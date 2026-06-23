// Frase → Campo Personale. Lemmatizzazione euristica italiana client-side.
// Per ogni token: prova varianti canoniche contro vasto.wordMap; se trovato
// aggiungi il lemma + archi uscenti dal KG; se sconosciuto, placeholder.

import { NEUTRAL_SIG } from './constants.js';
import { Field } from './field.js';
import { FIELDS, saveField, registerField } from './manager.js';
import { flagsForNewWord, flagsForNewEdge } from './policies/word.js';
import { setNuovoLayout } from './ui-state.js';

// Stopwords italiane (selettive).
const STOPWORDS = new Set([
  'il','lo','la','i','gli','le','un','uno','una',
  'di','del','dello','della','dei','degli','delle',
  'a','al','allo','alla','ai','agli','alle',
  'da','dal','dallo','dalla','dai','dagli','dalle',
  'in','nel','nello','nella','nei','negli','nelle',
  'su','sul','sullo','sulla','sui','sugli','sulle',
  'per','con','tra','fra','e','o','ma','se','che','ed','od',
  'non','mi','ti','si','ci','vi','ne','lo','la','li','le',
  'è','sono','sei','siamo','siete','era','erano','sia','stato','stata',
  'ho','hai','ha','abbiamo','avete','hanno','aveva','avevano',
  'questo','questa','quello','quella','questi','queste','quelli','quelle',
  'del','anche','come','quando','dove','perché','chi','cosa','quale',
  'molto','poco','più','meno','sempre','mai','già','ancora','ormai',
  'solo','stesso','tutto','tutti','tutte','tutta','niente','nulla','ogni',
  'ovviamente','tendenzialmente','certamente','probabilmente','veramente',
]);

// (suffisso → sostituzione). Un token può generare più candidati.
const ENDING_SWAPS = [
  // Verbi (suffisso più lungo prima).
  ['iamo','are'], ['iamo','ere'], ['iamo','ire'],
  ['ate','are'],  ['ete','ere'],  ['ite','ire'],
  ['ano','are'],  ['ono','ere'],  ['ono','ire'],
  ['avamo','are'], ['evamo','ere'], ['ivamo','ire'],
  ['avate','are'], ['evate','ere'], ['ivate','ire'],
  ['avano','are'], ['evano','ere'], ['ivano','ire'],
  ['ava','are'],  ['eva','ere'],  ['iva','ire'],
  ['avo','are'],  ['evo','ere'],  ['ivo','ire'],
  ['avi','are'],  ['evi','ere'],  ['ivi','ire'],
  ['arono','are'], ['erono','ere'], ['irono','ire'],
  ['asti','are'],  ['esti','ere'],  ['isti','ire'],
  ['erò','are'], ['erò','ere'], ['irò','ire'],
  ['erai','are'], ['erai','ere'], ['irai','ire'],
  ['erà','are'],  ['erà','ere'],  ['irà','ire'],
  ['eremo','are'],['eremo','ere'],['iremo','ire'],
  ['erete','are'],['erete','ere'],['irete','ire'],
  ['eranno','are'],['eranno','ere'],['iranno','ire'],
  ['ando','are'],
  ['endo','ere'], ['endo','ire'],
  ['ato','are'], ['ati','are'], ['ata','are'], ['ate','are'],
  ['uto','ere'], ['uti','ere'], ['uta','ere'], ['ute','ere'],
  ['ito','ire'], ['iti','ire'], ['ita','ire'], ['ite','ire'],
  // Nomi/aggettivi: plurali → singolari.
  ['zioni','zione'], ['sioni','sione'],
  ['chi','co'], ['ghi','go'], ['che','ca'], ['ghe','ga'],
  ['sci','sco'], ['sce','sca'], ['gli','glio'],
  ['oni','one'], ['ini','ino'], ['eni','ene'], ['ani','ano'],
  ['osi','oso'], ['ose','osa'],
  ['esi','ese'], ['ali','ale'], ['ili','ile'], ['oli','olo'],
  ['ici','ico'], ['iche','ica'],
  ['atori','atore'], ['atrici','atrice'],
  ['ore','ora'],
  ['i','o'], ['i','e'], ['e','a'],
];

export function tokenize(sentence){
  return sentence
    .toLowerCase()
    .replace(/[^\p{L}'\s-]/gu, ' ')
    .split(/\s+/)
    .map(t => t.trim())
    .filter(t => t.length > 1 && !STOPWORDS.has(t));
}

// Stopwords "sicure": articoli, preposizioni, congiunzioni base.
// Lista breve e conservativa — solo parole che il server NON tratta come
// lemmi. "tutti", "molto", ecc. non sono qui perché il server le considera
// parole piene (e quindi appariranno nel display in posizione propria).
const SURE_STOPWORDS = new Set([
  'il','lo','la','i','gli','le','un','uno','una',
  'di','a','da','in','con','su','per','tra','fra',
  'e','o','ma','se','che','ed','od','né',
  'del','dello','della','dei','degli','delle',
  'al','allo','alla','ai','agli','alle',
  'dal','dallo','dalla','dai','dagli','dalle',
  'nel','nello','nella','nei','negli','nelle',
  'sul','sullo','sulla','sui','sugli','sulle',
]);

// Filtra dai lemmi server le stopwords client. Il server può includere
// "un", "il", ecc. nei lemmi (anche se poi finiscono in `unknown`); per
// allineare client/server e non shiftare il match posizionale, le rimuoviamo
// qui. Esportata per uso in buildNuovo.
export function filterServerLemmas(lemmas){
  return (lemmas || []).filter(l => !SURE_STOPWORDS.has(l));
}

// Costruisce la mappa lemma → forma originale visualizzabile per ciascuna
// parola di significato. Display = raw token come scritto, SENZA articoli
// precedenti accorpati (l'utente vuole "futuro", non "un futuro"):
//   "ciao a tutti"             → { ciao: "ciao", tutti: "tutti" }
//   "vogliamo un futuro bello" → { volere: "vogliamo", futuro: "futuro", bello: "bello" }
//   "viva l'olio d'oliva"      → { viva/vivere: "viva", olio: "olio", oliva: "oliva" }
//
// Apostrofi: spezzati in fase di split (`l'olio` → `l`, `olio`; `d'oliva` →
// `d`, `oliva`). I monoletterali risultanti sono filtrati come stopword.
//
// Match per candidate-lemma (non posizionale): per ogni raw token, troviamo
// il primo lemma server, non ancora consumato, che sia un suo candidato
// lemmatizzato (lemmaCandidates include il token stesso). Questo evita lo
// shift quando il server dedupa lemmi ripetuti (es. "viva viva" → un solo
// lemma) o aggiunge/toglie token per via di clean_token sul server.
export function buildDisplayMap(sentence, lemmas){
  const map = new Map();
  const filtered = filterServerLemmas(lemmas);
  if(filtered.length === 0) return map;
  const raw = _tokenizeRaw(sentence);

  const consumed = new Set();
  // Raw token già matchato a un lemma server: una sua ripetizione è solo
  // ridondanza ("viva viva" → un solo lemma sul server) e NON deve consumare
  // il fallback verso un lemma diverso. Senza questo skip, il secondo "viva"
  // si appropriava del posto di "olio", e a cascata "oliva" finiva fuori range.
  const matchedRaw = new Set();
  for(const tok of raw){
    if(SURE_STOPWORDS.has(tok) || tok.length <= 1) continue;
    if(matchedRaw.has(tok)) continue;

    const cands = lemmaCandidates(tok);  // include tok stesso
    let matchedIdx = -1, matchedLemma = null;
    for(const c of cands){
      const idx = filtered.indexOf(c);
      if(idx >= 0 && !consumed.has(idx)){
        matchedIdx = idx; matchedLemma = c; break;
      }
    }
    if(matchedIdx < 0){
      // Fallback per token "nuovi" senza match diretto: primo lemma libero.
      // Utile per lemmatizzazioni server che divergono dai nostri suffissi
      // (es. "vogliamo" → "volere" via grammar irregolare).
      for(let i = 0; i < filtered.length; i++){
        if(!consumed.has(i)){ matchedIdx = i; matchedLemma = filtered[i]; break; }
      }
    }
    if(matchedIdx >= 0){
      consumed.add(matchedIdx);
      matchedRaw.add(tok);
      if(!map.has(matchedLemma)) map.set(matchedLemma, tok);
    }
  }
  return map;
}

// Tokenizzazione coerente con la logica di clean_token lato server: gli
// apostrofi spezzano i token (così "l'olio" → ["l", "olio"]). I single-char
// vengono filtrati a valle.
function _tokenizeRaw(sentence){
  return sentence.toLowerCase()
    .replace(/[^\p{L}\s-]/gu, ' ')   // niente apostrofi nella whitelist: spezzano
    .split(/\s+/)
    .map(t => t.trim())
    .filter(t => t.length > 0);
}

export function lemmaCandidates(token){
  const cands = [];
  const seen = new Set();
  const push = (c) => { if(c && !seen.has(c)){ seen.add(c); cands.push(c); } };
  push(token);
  for(const [suf, rep] of ENDING_SWAPS){
    if(token.endsWith(suf) && token.length > suf.length + 1){
      push(token.slice(0, -suf.length) + rep);
    }
  }
  const stem = token.replace(/[aeiou]$/, '');
  for(const end of ['o','a','e','are','ere','ire']) push(stem + end);
  return cands;
}

export function tryLemma(token, vastoWordMap){
  for(const c of lemmaCandidates(token)){
    if(vastoWordMap[c]) return { lemma: c, found: true };
  }
  return { lemma: token, found: false };
}

// Converte firma 8D da [0,1] float a [0,100] intero.
function sigF64ToInt(sig){
  if(!sig || sig.length !== 8) return NEUTRAL_SIG.slice();
  return sig.map(v => Math.round((v || 0.5) * 100));
}

// ---- Fase 1: interpretazione frase ---------------------------------------
// Chiama /api/medio e restituisce solo l'interpretazione grezza, riusabile
// sia per il rebuild iniziale del personale sia per la frase aggiuntiva.
// NON tocca i Field.
//
// Return: { sentence, lemmas, words: [{word, signature, outgoing}], unknown }
export async function interpretSentence(sentence){
  const r = await fetch('/api/medio?sentence=' + encodeURIComponent(sentence));
  if(!r.ok) throw new Error('api /api/medio failed: ' + r.status);
  const data = await r.json();
  return {
    sentence,
    lemmas:  data.lemmas || [],
    words:   data.words || [],
    unknown: data.unknown || [],
  };
}

// ---- Fase 2: applicazione al field --------------------------------------
// Aggiunge le parole + (opzionalmente) archi dell'interpretazione al field.
// La POSIZIONE delle nuove parole deriva sempre dalla firma 8D, via
// Field._placeIfNeeded (sigToXY + placeByRank). Coerenza fissa: la firma
// è la verità, la posizione segue.
//
// opts.markFromSentence:  parole della frase ricevono flag fromSentence=true
//                          (alone dorato).
// opts.importEdges:       se false, gli archi del KG NON vengono importati
//                          e i target degli archi NON vengono creati.
// opts.markTransmitted:   se true, parole/archi nuovi sono già "transmitted"
//                          (la creazione iniziale del personale da frase
//                          considera le parole già nel KG come allineate).
//                          Se false, la frase aggiuntiva lascia tutto da
//                          trasmettere (l'utente preme ↗ trasmetti).
//
// Return: { added: [parole nuove], skipped: [già presenti], unknown: [...] }
export function applyInterpretation(F, interp, {
  markFromSentence = true,
  importEdges = true,
  markTransmitted = false,
  displayMap = null,
  // Se false (default): solo relazioni uscenti dal lemma (direction='out').
  // Se true: include anche le entranti (direction='in', il KG le restituisce
  // per parole come "vita" che sono oggetto di molte relazioni). Le entranti
  // generano archi direzionati `subject → lemma`.
  includeIncoming = false,
} = {}){
  const V = FIELDS.vasto;
  const added = [], skipped = [];

  const addWord = (name, sigFromApi, fromSentence) => {
    if(F.hasWord(name)){
      // La parola è già nel field (es. clonata dal vasto). Aggiorna però
      // displayName se è una parola della frase: vogliamo la forma originale
      // (incluso eventuali articoli) anziché il lemma.
      if(fromSentence && displayMap?.has(name)){
        F.wordMap[name].displayName = displayMap.get(name);
      }
      skipped.push(name);
      return;
    }
    const inVasto = V?.wordMap?.[name];
    let sig;
    const flagOpts = { fromSentence, transmitted: markTransmitted };
    if(inVasto){
      sig = (inVasto.sig || NEUTRAL_SIG).slice();
    } else if(sigFromApi){
      sig = sigF64ToInt(sigFromApi);
      flagOpts.fromApi = true;
    } else {
      sig = NEUTRAL_SIG.slice();
      flagOpts.noSignature = true;
    }
    // Niente position: F.addWord → _placeIfNeeded piazza la parola in base
    // alla sua firma 8D (sigToXY + placeByRank), come ovunque altrove.
    const w = { w: name, sig, flags: flagsForNewWord(F, flagOpts) };
    if(fromSentence && displayMap?.has(name)) w.displayName = displayMap.get(name);
    F.addWord(w);
    added.push(name);
  };

  // 1. Parole della frase, in ordine (data.lemmas).
  const wordDataByLemma = {};
  interp.words.forEach(w => { wordDataByLemma[w.word] = w; });
  const unknownSet = new Set(interp.unknown);

  for(const lemma of interp.lemmas){
    const wData = wordDataByLemma[lemma];
    if(wData){
      addWord(wData.word, wData.signature, markFromSentence);
    } else if(unknownSet.has(lemma)){
      addWord(lemma, null, markFromSentence);
    }
  }

  // 2. Target degli archi + archi stessi (le parole-bersaglio non sono
  // della frase, quindi fromSentence=false). Saltato se importEdges=false:
  // in quel caso le parole entrano nel campo "nude" e le relazioni si
  // estraggono on-demand via relations-extract.js.
  if(importEdges){
    for(const wd of interp.words){
      for(const e of (wd.outgoing || [])){
        const isIncoming = e.direction === 'in';
        if(isIncoming && !includeIncoming) continue;  // solo outgoing
        // Niente SIMILAR_TO alla creazione: sono molti (i saluti di "ciao"
        // erano 6) e si ammassano nella stessa regione 8D rompendo il layout.
        // L'utente può sempre aggiungerli on-demand con "estrai relazioni".
        if(e.relation === 'SIMILAR_TO') continue;
        addWord(e.target, e.target_signature, false);
        F.addEdge({
          from: isIncoming ? e.target : wd.word,
          to:   isIncoming ? wd.word : e.target,
          rel: e.relation,
          conf: Math.round((e.confidence || 0.5) * 100),
          flags: flagsForNewEdge(F, { transmitted: markTransmitted }),
        });
      }
    }
  }

  return { added, skipped, unknown: interp.unknown };
}

// ---- Genera/rigenera il personale da una frase --------------------------
// Rebuild completo: il personale viene popolato CLONANDO il vasto (tutti i
// ~3000 nodi + archi) per la fase di "comprensione frase" dell'animazione.
// L'animazione fa svanire le parole non rilevanti; al termine
// `commitExpansion()` rimuove le parole vasto-clone restando solo con
// frase + vicini. Vedi roadmap UX §1 Fase C "il vasto si ritira, lo schermo
// diventa il proprio campo in costruzione".
export async function buildNuovo(sentence){
  const V = FIELDS.vasto;
  if(!V) throw new Error('vasto non inizializzato');

  const interpRaw = await interpretSentence(sentence);
  // Filtra le sicure stopwords ovunque: il server può includere "un", "una",
  // "il", … sia in `lemmas` sia in `unknown` sia in `words`. Se passassimo
  // l'interp grezza ad applyInterpretation, "una" verrebbe aggiunta come
  // parola della frase (fromSentence=true), MA non riceverebbe sentenceIndex
  // (perché filteredLemmas la salta) → l'animazione la mette in fondo. Filtrare
  // qui in un punto solo elimina la sorgente del bug.
  const interp = {
    ...interpRaw,
    lemmas:  filterServerLemmas(interpRaw.lemmas),
    unknown: (interpRaw.unknown || []).filter(u => !SURE_STOPWORDS.has(u)),
    words:   (interpRaw.words   || []).filter(w => !SURE_STOPWORDS.has(w.word)),
  };

  const P = new Field('nuovo', V.frame);
  P.addDimLabels();

  // 1. Clona TUTTO il vasto nel personale. Le parole/archi clonati sono
  // marcati `flags.fromExpansion: true` — verranno rimossi a fine animazione,
  // tranne quelli appartenenti alla frase o ai suoi vicini diretti.
  const vastoWords = V.words.map(vw => ({
    w: vw.w,
    sig: vw.sig.slice(),
    flags: { fromExpansion: true },
    position: vw.position ? { ...vw.position } : null,
    deg: vw.deg || 0,
  }));
  const vastoEdges = V.edges.map(e => ({
    from: e.from, to: e.to, rel: e.rel, conf: e.conf,
    flags: { fromExpansion: true },
  }));
  P.bulkLoad({ words: vastoWords, edges: vastoEdges });

  // 2. Applica l'interpretazione: aggiunge i lemmi del server con sig+outgoing
  // dal KG, displayName = forma originale (con articoli precedenti accorpati).
  // displayMap è costruita dai lemmi RAW (non filtrati) per il match posizionale
  // sui token raw. Il filtro stopword avviene dentro buildDisplayMap.
  const displayMap = buildDisplayMap(sentence, interpRaw.lemmas);
  applyInterpretation(P, interp, {
    markFromSentence: true, markTransmitted: true, displayMap,
  });

  // 3. Marca i lemmi della frase + i loro target/edges come fromExpansion=false
  // (vengono preservati dal commit), e setta displayName + sentenceIndex.
  // Iteriamo i lemmi (già filtrati): "un" e simili non producono un puntino
  // dedicato — sono semplicemente saltati. sentenceIndex riflette l'ordine
  // di apparizione tra i lemmi non-stopword, così l'animazione segue la frase.
  const filteredLemmas = interp.lemmas;
  filteredLemmas.forEach((lemma, idx) => {
    let w = P.wordMap[lemma];
    if(!w){
      // Il server riconosce il lemma ma non c'è un word object (parola senza
      // sig nel lessico, raro). Aggiungiamolo come puntino nudo perché la
      // parola della frase non sparisca dal grafo.
      P.addWord({
        w: lemma,
        sig: NEUTRAL_SIG.slice(),
        flags: flagsForNewWord(P, { fromSentence: true, noSignature: true }),
        displayName: displayMap.get(lemma) || lemma,
      });
      w = P.wordMap[lemma];
    }
    if(w){
      w.flags.fromSentence = true;
      w.flags.fromExpansion = false;
      w.displayName = displayMap.get(lemma) || lemma;
      // Assegna sentenceIndex se non è già stato assegnato un valore inferiore
      // (caso di parole ripetute, teniamo la prima occorrenza per l'ordine).
      if (typeof w.sentenceIndex !== 'number' || idx < w.sentenceIndex) {
        w.sentenceIndex = idx;
      }
    }
  });
  for(const wd of interp.words){
    for(const e of (wd.outgoing || [])){
      // applyInterpretation salta direction='in' di default → niente edge per
      // quelle. Saltiamo anche qui per evitare di marcare target/edge inesistenti.
      if(e.direction === 'in') continue;
      const tw = P.wordMap[e.target];
      if(tw) tw.flags.fromExpansion = false;
      const edge = P.edgeByKey[`${wd.word}|${e.target}|${e.relation}`];
      if(edge) edge.flags.fromExpansion = false;
    }
  }

  P.sentence = sentence;
  P.expansionShown = false;
  registerField('nuovo', P);
  // Ogni nuova creazione da frase parte SEMPRE in layout dimensionale —
  // l'animazione di comprensione si gioca sulle posizioni 8D del vasto. Il
  // toggle "relazioni" resta una scelta successiva dell'utente.
  setNuovoLayout('dimensional');
  // Niente saveField qui: il personale è "gonfio" di nodi vasto-clone che
  // verranno rimossi da commitExpansion al termine dell'animazione.

  const known = [], fromApi = [];
  for(const w of interp.words){
    if(V.wordMap[w.word]) known.push(w.word);
    else if(w.signature) fromApi.push({ original: w.word, lemma: w.word });
  }
  return {
    known,
    unknown: interp.unknown.map(u => ({ original: u, lemma: u })),
    fromApi,
    field: P,
  };
}

// Termine animazione comprensione: ricostruisce il personale da zero con solo
// le parole/archi rilevanti (non fromExpansion). Sostituisce il Field gonfio
// con uno pulito — più solido che "potare" il vecchio (vis-network mantiene
// cache interne sul DataSet originale, anche dopo remove batch).
// Idempotente: se expansionShown è già true, no-op.
export function commitExpansion(){
  const P = FIELDS.nuovo;
  if(!P || P.expansionShown) return;
  const V = FIELDS.vasto;
  if(!V) return;

  const keepWords = P.words.filter(w => !w.flags?.fromExpansion);
  const keepSet = new Set(keepWords.map(w => w.w));
  const sentenceSet = new Set(
    keepWords.filter(w => w.flags?.fromSentence).map(w => w.w)
  );
  // Regola "solo uscenti": ogni parola del personale mostra le sue
  // relazioni uscenti, non le entranti. In particolare, non vogliamo che
  // una parola-frase come "io" raccolga vicini bidirezionali ("X IS_A io"
  // farebbe apparire "X" come vicino di "io", anche se la freccia è
  // l'opposto). Filtro: escludi solo gli archi che entrano in una
  // parola-frase da una non-frase. Tieni tutto il resto:
  //   - frase → satellite      (uscenti dalla frase, nucleo del campo)
  //   - satellite → satellite  (relazioni proprie fra i target del KG)
  //   - frase  → frase         (struttura interna alla frase)
  const keepEdges = P.edges.filter(e =>
    keepSet.has(e.from) && keepSet.has(e.to) &&
    !(sentenceSet.has(e.to) && !sentenceSet.has(e.from))
  );

  const cleanP = new Field('nuovo', V.frame);
  cleanP.addDimLabels();
  cleanP.sentence = P.sentence;
  cleanP.expansionShown = true;
  cleanP.bulkLoad({
    words: keepWords.map(w => ({
      w: w.w,
      sig: w.sig.slice(),
      flags: { ...w.flags, fromExpansion: false },
      position: w.position ? { ...w.position } : null,
      deg: w.deg,
      userOct: w.userOct ? w.userOct.slice() : null,
      displayName: w.displayName || null,
      sentenceIndex: w.sentenceIndex,
    })),
    edges: keepEdges.map(e => ({
      from: e.from, to: e.to, rel: e.rel, conf: e.conf,
      flags: { ...e.flags, fromExpansion: false },
    })),
  });
  // Ricalcola posizioni dalle firme 8D del set ridotto (frase + vicini).
  // Le posizioni ereditate dal vasto-clone erano calcolate per 27K nodi e
  // diventano illeggibilmente fitte con 30-50 parole; uno spread con
  // minDist alto le faceva esplodere fuori dal grafo. placeByRank sul set
  // locale dà anelli arieggiati e coerenti con le firme.
  cleanP._bulkPlaceByRank(cleanP.words);
  cleanP.spreadNonOverlapping({ minDist: 55, iterations: 40 });
  registerField('nuovo', cleanP);  // sostituisce il Field gonfio
  saveField('nuovo');
}

// ---- Aggiunge una frase al personale (merge, no rebuild) ----------------
// Le parole della frase vengono aggiunte al personale come fromSentence +
// userCreated, senza azzerare quello che c'era prima. Posizione = funzione
// della firma (placeByRank), come ovunque nel campo. Chiamato dal menu
// "+ aggiungi frase" sul personale già popolato.
export async function addSentenceToNuovo(sentence){
  const V = FIELDS.vasto;
  const P = FIELDS.nuovo;
  if(!V || !P) throw new Error('campi non inizializzati');

  const interp = await interpretSentence(sentence);
  const filteredLemmas = filterServerLemmas(interp.lemmas || []);
  const displayMap = buildDisplayMap(sentence, interp.lemmas);
  // Frase aggiuntiva: importEdges=false. Le parole entrano nel campo "nude":
  // l'utente decide se e quando estrarre le relazioni dal KG, con quale filtro.
  const result = applyInterpretation(P, { ...interp, lemmas: filteredLemmas }, {
    markFromSentence: true, importEdges: false, markTransmitted: false,
    displayMap,
  });

  // Setta sentenceIndex sulle parole appena aggiunte. Base = max esistente + 1
  // così le frasi aggiunte successivamente si concatenano in ordine di
  // inserimento, senza collidere con gli indici di frasi precedenti.
  // Senza questo, il rectangular layout (che ordina per sentenceIndex) ordina
  // in modo casuale le parole-frase nuove.
  let baseIdx = -1;
  for(const w of P.words){
    if(typeof w.sentenceIndex === 'number' && w.sentenceIndex > baseIdx){
      baseIdx = w.sentenceIndex;
    }
  }
  filteredLemmas.forEach((lemma, i) => {
    const w = P.wordMap[lemma];
    if(!w) return;
    w.flags.fromSentence = true;
    if(displayMap?.has(lemma)) w.displayName = displayMap.get(lemma);
    if(typeof w.sentenceIndex !== 'number'){
      w.sentenceIndex = baseIdx + 1 + i;
    }
  });

  P.spreadNonOverlapping({ iterations: 30 });
  saveField('nuovo');
  return result;
}
