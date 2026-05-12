// Un Field possiede il proprio stato topologico: parole, archi, DataSets,
// selezione, breadcrumb. Più istanze (vasto/nuovo/medio) coesistono senza
// condividere memoria: si condivide solo il reference frame.
//
// Nessuno stile nodo/arco vive qui — tutto passa per node-style.js
// (vedi CLAUDE.md §2).

import { NEUTRAL_SIG, DIM_ANGLES } from './constants.js';
import { sigToXY, placeByRank, rankOfMag } from './geometry.js';
import { buildNodeSpec, buildEdgeSpec, buildDimLabelSpec, nodeColorUpdate } from './node-style.js';
import { tokens } from './theme.js';

// Chiave stabile di un arco: usata come id nel DataSet.
export function edgeKey(e){ return `${e.from}|${e.to}|${e.rel}`; }

// ---- Normalizzazione input ----
// Accetta sia formato canonico { w, sig, flags: {...} } che formato legacy
// { w, sig, _userWord, _unknown, _fromSentence, _fromApi, _noSignature,
//   transmitted }. Restituisce sempre il formato nuovo. Usato in hydrate
// (per backwards compat di salvataggi server) e in addWord da sentence.js.
function normalizeWord(input){
  const flags = {
    userCreated:   !!(input.flags?.userCreated   ?? input._userWord),
    unknown:       !!(input.flags?.unknown       ?? input._unknown),
    fromSentence:  !!(input.flags?.fromSentence  ?? input._fromSentence),
    fromApi:       !!(input.flags?.fromApi       ?? input._fromApi),
    noSignature:   !!(input.flags?.noSignature   ?? input._noSignature),
    transmitted:   !!(input.flags?.transmitted   ?? input.transmitted),
    // fromExpansion: marker per parole vasto-clone temporanee nel personale
    // durante la fase comprensione frase. Rimosse da commitExpansion.
    fromExpansion: !!(input.flags?.fromExpansion),
  };
  const position = input.position
    ? { ...input.position }
    : (input._px != null ? { x: input._px, y: input._py,
                             angle: input._angle, mag: input._mag, normR: input._normR }
                         : null);
  return {
    w: input.w,
    sig: (input.sig || NEUTRAL_SIG).slice(),
    flags,
    position,
    deg: input.deg || 0,
    localDegree: 0,
    userOct: input.userOct || input._userOct || null,
    // displayName: forma originale (con articoli) per parole della frase.
    // Se assente, label cade su `w` (lemma).
    displayName: input.displayName || null,
    // Indice di apparizione nella frase originale (per ordine animazione).
    sentenceIndex: typeof input.sentenceIndex === 'number' ? input.sentenceIndex : null,
    // Posizione manuale (drag o creazione via right-click). Se true, i
    // layout (rectangular/dimensional) NON sovrascrivono w.position.
    // Persistito in toJSON così il drag dell'utente sopravvive al refresh.
    _userPositioned: !!input._userPositioned,
    // Snapshot delle coordinate dimensionali: serve a applyDimensionalLayout
    // per ripristinarle quando si torna dal layout rectangular. Persistito
    // nello snapshot di history così l'undo non perde il riferimento.
    _dimPos: input._dimPos ? { x: input._dimPos.x, y: input._dimPos.y } : null,
  };
}

// ---- Classe Field ---------------------------------------------------------

export class Field {
  // id: 'vasto' | 'nuovo'
  // frame: { dimMean, dimStd } — condiviso, letto, mai modificato
  constructor(id, frame){
    this.id = id;
    this.frame = frame;
    this.words = [];
    this.edges = [];
    this.wordMap = Object.create(null);
    this.edgesByWord = Object.create(null);
    this.edgeByKey = Object.create(null);
    this.nodesDS = new vis.DataSet();
    this.edgesDS = new vis.DataSet();
    this.selected = null;
    this.currentRosa = null;
    this.isDimmed = false;
    this.subHover = null;
    this.navPath = [];
    this.sentence = null;       // valorizzato se il personale è stato creato da frase
    this.expansionShown = false; // true dopo che l'animazione iniziale è stata vista
  }

  // ---- Parole ----

  hasWord(w){ return !!this.wordMap[w]; }

  // Accetta sia un word object "nuovo formato" sia un word "legacy" con _prefissi.
  addWord(input){
    const existing = this.wordMap[input.w];
    if(existing) return existing;

    const word = normalizeWord(input);
    this.wordMap[word.w] = word;
    this.words.push(word);
    this.edgesByWord[word.w] = new Set();

    this._placeIfNeeded(word);
    const spec = buildNodeSpec(word, 'normal', { fieldId: this.id });
    this.nodesDS.add(spec);
    return word;
  }

  // Posiziona la parola nel campo se non ha ancora coordinate.
  _placeIfNeeded(word){
    if(word.position?.x != null) return;
    const sig = word.sig;
    const p = sigToXY(sig, this.frame);
    const mag = Math.sqrt(p.x * p.x + p.y * p.y);
    const others = this.words.filter(x => x !== word);
    const rank = rankOfMag(mag, others);
    const placed = placeByRank(sig, this.frame, rank, Math.max(2, this.words.length));
    word.position = {
      x: placed.x, y: placed.y,
      angle: placed.angle, mag: placed.mag, normR: placed.normR,
    };
  }

  // Carica in batch parole + archi: una sola .add() su nodesDS e una su
  // edgesDS, placement per rank in un'unica passata. Evita le ~27K add()
  // individuali del boot (vedi CLAUDE.md §3 e analisi performance).
  // Le posizioni in input sono rispettate; solo le parole senza position
  // vengono piazzate qui.
  bulkLoad({ words = [], edges = [] } = {}){
    if(!words.length && !edges.length) return;

    // 1. Normalizza e registra le parole (filter dei duplicati).
    const newWords = [];
    for(const input of words){
      if(this.wordMap[input.w]) continue;
      const word = normalizeWord(input);
      this.wordMap[word.w] = word;
      this.words.push(word);
      this.edgesByWord[word.w] = new Set();
      newWords.push(word);
    }

    // 2. Placement per rank in un'unica passata su quelle senza posizione.
    const toPlace = newWords.filter(w => w.position?.x == null);
    if(toPlace.length) this._bulkPlaceByRank(toPlace);

    // 3. Costruisci tutte le node spec e una sola .add().
    if(newWords.length){
      const nodeBatch = newWords.map(w => buildNodeSpec(w, 'normal', { fieldId: this.id }));
      this.nodesDS.add(nodeBatch);
    }

    // 4. Archi: registra + costruisci spec, una sola .add() finale.
    const edgeBatch = [];
    for(const edge of edges){
      const key = edgeKey(edge);
      if(this.edgeByKey[key]) continue;
      const stored = {
        from: edge.from, to: edge.to, rel: edge.rel,
        conf: edge.conf || 50,
        via: edge.via || null,
        flags: {
          userCreated:   !!(edge.flags?.userCreated ?? edge._userEdge),
          transmitted:   !!(edge.flags?.transmitted ?? edge.transmitted),
          fromExpansion: !!(edge.flags?.fromExpansion),
        },
        key,
      };
      this.edges.push(stored);
      this.edgeByKey[key] = stored;
      if(!this.edgesByWord[edge.from]) this.edgesByWord[edge.from] = new Set();
      if(!this.edgesByWord[edge.to])   this.edgesByWord[edge.to]   = new Set();
      this.edgesByWord[edge.from].add(key);
      this.edgesByWord[edge.to].add(key);
      edgeBatch.push(buildEdgeSpec(stored));
    }
    if(edgeBatch.length) this.edgesDS.add(edgeBatch);

    // 5. localDegree: set una sola volta dopo gli archi.
    for(const w of this.words){
      w.localDegree = (this.edgesByWord[w.w] || new Set()).size;
    }

    // 6. In nuovo/medio la dimensione nodo dipende da localDegree: refresh
    // delle spec post-grado. In vasto la dimensione usa word.deg (esterno),
    // già riflesso nella spec iniziale → nessun refresh necessario.
    if(this.id !== 'vasto' && newWords.length){
      const refreshBatch = newWords.map(w => buildNodeSpec(w, 'normal', { fieldId: this.id }));
      this.nodesDS.update(refreshBatch);
    }
  }

  // Placement per rank: ogni parola ottiene una posizione 2D basata sulla
  // sua magnitudine ordinata. Coerente con app.js prima della Fase 1.
  _bulkPlaceByRank(words){
    const items = words.map(w => {
      const p = placeByRank(w.sig || NEUTRAL_SIG, this.frame, 0, 1);
      return { word: w, mag: p.mag };
    }).sort((a, b) => a.mag - b.mag);

    const total = Math.max(2, items.length);
    items.forEach((entry, i) => {
      const placed = placeByRank(entry.word.sig || NEUTRAL_SIG, this.frame, i, total, 0.06);
      entry.word.position = {
        x: placed.x, y: placed.y,
        angle: placed.angle, mag: placed.mag, normR: placed.normR,
      };
    });
  }

  removeWord(w){
    const word = this.wordMap[w];
    if(!word) return;
    [...(this.edgesByWord[w] || [])].forEach(k => this.removeEdge(k));
    try { this.nodesDS.remove(w); } catch(_){}
    delete this.wordMap[w];
    delete this.edgesByWord[w];
    this.words = this.words.filter(x => x.w !== w);
  }

  updateWordSig(w, sig){
    const word = this.wordMap[w];
    if(!word) return;
    const wasUnknown = word.flags.unknown;
    word.sig = sig.slice();

    // Riposiziona dal nuovo sig — UNLESS l'utente l'ha trascinata o
    // posizionata a mano. Il drag wins: l'intento utente non viene
    // sovrascritto da un cambio firma. La posizione manuale resta dov'è.
    let placed;
    if(word._userPositioned && word.position?.x != null){
      placed = { x: word.position.x, y: word.position.y };
    } else {
      const p = sigToXY(sig, this.frame);
      const mag = Math.sqrt(p.x * p.x + p.y * p.y);
      const others = this.words.filter(x => x.w !== w);
      const rank = rankOfMag(mag, others);
      placed = placeByRank(sig, this.frame, rank, this.words.length);
      word.position = {
        x: placed.x, y: placed.y,
        angle: placed.angle, mag: placed.mag, normR: placed.normR,
      };
    }

    if(wasUnknown){
      word.flags.unknown = false;
      word.flags.userCreated = true;
    }

    // Update parziale: posizione + colore. La firma cambia → la dimensione
    // dominante può cambiare → il colore deve seguirla, altrimenti il pallino
    // resta del vecchio colore mentre la sidebar mostra il nuovo (incongruenza
    // segnalata dalla collega). Per i nodi unknown promossi, ricostruiamo
    // l'intero spec (cambia sfondo, font, ecc).
    if(wasUnknown){
      this.nodesDS.update(buildNodeSpec(word, 'normal', { fieldId: this.id }));
    } else {
      // Posizione + colore. Il color block lo costruisce node-style.js
      // (regola §2: nessuno spec inline fuori da quel modulo).
      const colorPatch = nodeColorUpdate(word.w, sig);
      this.nodesDS.update({ ...colorPatch, x: placed.x, y: placed.y });
    }
  }

  // ---- Archi ----

  addEdge(edge){
    const key = edgeKey(edge);
    if(this.edgeByKey[key]){
      // Edge già presente: se l'utente passa un nuovo via, aggiorna in place.
      // La chiave non include via — un solo arco from|to|rel, via è metadato.
      if(edge.via && !this.edgeByKey[key].via) this.edgeByKey[key].via = edge.via;
      return this.edgeByKey[key];
    }
    const stored = {
      from: edge.from, to: edge.to, rel: edge.rel,
      conf: edge.conf || 50,
      via: edge.via || null,
      flags: {
        userCreated:   !!(edge.flags?.userCreated ?? edge._userEdge),
        transmitted:   !!(edge.flags?.transmitted ?? edge.transmitted),
        fromExpansion: !!(edge.flags?.fromExpansion),
      },
      key,
    };
    this.edges.push(stored);
    this.edgeByKey[key] = stored;
    if(!this.edgesByWord[edge.from]) this.edgesByWord[edge.from] = new Set();
    if(!this.edgesByWord[edge.to])   this.edgesByWord[edge.to]   = new Set();
    this.edgesByWord[edge.from].add(key);
    this.edgesByWord[edge.to].add(key);

    const fromWord = this.wordMap[edge.from];
    const toWord   = this.wordMap[edge.to];
    if(fromWord) fromWord.localDegree = (this.edgesByWord[edge.from] || new Set()).size;
    if(toWord)   toWord.localDegree   = (this.edgesByWord[edge.to]   || new Set()).size;

    try { this.edgesDS.add(buildEdgeSpec(stored)); } catch(_){}
    return stored;
  }

  removeEdge(key){
    const edge = this.edgeByKey[key];
    if(!edge) return;
    try { this.edgesDS.remove(key); } catch(_){}
    this.edges = this.edges.filter(e => edgeKey(e) !== key);
    delete this.edgeByKey[key];
    this.edgesByWord[edge.from]?.delete(key);
    this.edgesByWord[edge.to]?.delete(key);

    const fromWord = this.wordMap[edge.from];
    const toWord   = this.wordMap[edge.to];
    if(fromWord) fromWord.localDegree = (this.edgesByWord[edge.from] || new Set()).size;
    if(toWord)   toWord.localDegree   = (this.edgesByWord[edge.to]   || new Set()).size;
  }

  edgesForWord(w){
    return [...(this.edgesByWord[w] || [])].map(k => this.edgeByKey[k]).filter(Boolean);
  }

  edgesForWordIds(w){
    return [...(this.edgesByWord[w] || [])];
  }

  // Vicini diretti della parola, limitati alle parole presenti nel DataSet.
  // direction='both' (default) include tutti i vicini.
  // direction='out' include solo i vicini puntati DA w (w → other).
  // direction='in'  include solo i vicini che puntano A w (other → w).
  // opts.filterByType: function(rel) → bool. Se passata, esclude dalla rosa
  // i vicini raggiungibili SOLO tramite archi di tipo rifiutato. Coerenza
  // con la legenda interattiva: arco e vicino-tramite-arco sono lo stesso
  // fatto, non possono diverger.
  getRosa(w, direction = 'both', opts = {}){
    const rosa = new Set();
    const typeOK = opts.filterByType || (() => true);
    (this.edgesByWord[w] || []).forEach(key => {
      const e = this.edgeByKey[key];
      if(!e) return;
      if(direction === 'out' && e.from !== w) return;
      if(direction === 'in'  && e.to   !== w) return;
      if(!typeOK(e.rel)) return;
      const other = e.from === w ? e.to : e.from;
      if(this.nodesDS.get(other)) rosa.add(other);
    });
    return rosa;
  }

  // ---- Etichette dimensioni (il "compass") ----
  //
  // No-op: le 8 etichette POTERE/MATERIA/… sono ora un overlay HTML in
  // components/dim-overlay.js che resta ancorato al viewport durante zoom
  // e pan (richiesta della collega). Il metodo è preservato per retrocompat
  // dei caller (app.js, modals.js, sentence.js, view-switcher.js): nessuno
  // di loro va aggiornato perché aggiungere le label al canvas non serve
  // più. Se in futuro qualcuno vorrà di nuovo le label dentro al canvas
  // (per export immagine, screenshot, ecc.), basterà ripristinare il body.
  addDimLabels(){ /* moved to dim-overlay.js */ }

  // ---- Snapshot degli archi in stato "normal" (nascosti, baseline) ----
  // Usato dal restore post-highlight/filter senza memorizzare una copia.
  baselineEdgeBatch(){
    // layoutMode ci serve per non perdere lo smooth e le opacità giuste
    const layoutMode = (this.id === 'nuovo' && window._getNuovoLayout) ? window._getNuovoLayout() : 
                       (this.id === 'nuovo' && localStorage.getItem('uir1_nuovo_layout') === 'rectangular' ? 'rectangular' : undefined);
    return this.edges.map(e => buildEdgeSpec(e, { variant: 'normal', layoutMode }));
  }

  // ---- Refresh size (dopo bulk add) ----
  refreshSizes(){
    const batch = [];
    this.words.forEach(w => {
      w.localDegree = (this.edgesByWord[w.w] || new Set()).size;
      batch.push(buildNodeSpec(w, 'normal', { fieldId: this.id }));
    });
    if(batch.length) this.nodesDS.update(batch);
  }

  // ---- Posizione manuale (drag o creazione) ----
  // Marca una parola come "posizionata dall'utente" e aggiorna i campi
  // x/y. I layout (rectangular/dimensional) skipperanno questa parola, e
  // updateWordSig non sovrascriverà la posizione al cambio firma.
  markPositionUser(w, x, y){
    const word = this.wordMap[w];
    if(!word) return;
    word.position = { ...(word.position || {}), x, y };
    word._userPositioned = true;
  }

  // Posiziona un set di NUOVE parole in cluster ANULARE attorno a una
  // parola "host". Strategia multi-ring: il primo anello (~10 parole) sta
  // a baseRadius dall'host, poi anelli concentrici sempre più larghi.
  // Capacità di un anello = ⌈2π·r / spacing⌉, così la spaziatura angolare
  // resta leggibile anche con 30+ parole. Senza questo, l'estrazione di
  // un satellite molto connesso impilava tutte le parole sopra l'host
  // rendendo il campo illeggibile (segnalato dalla collega).
  // Le parole estratte sono _userPositioned → non vengono spostate dal
  // layout successivo.
  clusterAroundWord(hostWord, newWordIds, opts = {}){
    const host = this.wordMap[hostWord];
    if(!host || !host.position) return;
    const { baseRadius = 140, ringSpacing = 90, slotSpacing = 110, startAngle = -Math.PI / 2 } = opts;
    const ids = newWordIds.filter(id => this.wordMap[id] && id !== hostWord);
    if(!ids.length) return;

    // Distribuzione su anelli successivi. Anello k ha raggio
    // baseRadius + k*ringSpacing, capacità ⌈2π·r / slotSpacing⌉.
    let placed = 0;
    let ring = 0;
    let angleOffset = startAngle;
    while(placed < ids.length){
      const r = baseRadius + ring * ringSpacing;
      const capacity = Math.max(6, Math.floor((2 * Math.PI * r) / slotSpacing));
      const count = Math.min(capacity, ids.length - placed);
      const step = (Math.PI * 2) / count;
      // Stagger fra anelli successivi così le parole non si allineano radialmente.
      const offset = angleOffset + (ring % 2 === 1 ? step / 2 : 0);
      for(let i = 0; i < count; i++){
        const a = offset + step * i;
        const x = host.position.x + Math.cos(a) * r;
        const y = host.position.y + Math.sin(a) * r;
        this.markPositionUser(ids[placed + i], x, y);
      }
      placed += count;
      ring++;
    }
  }

  // ---- Link mode: forza visibilità label su TUTTE le parole ----
  // Usato dal "tab collega" della modale aggiungi parola: l'utente deve
  // vedere le etichette di tutte le parole del campo per sapere con cosa
  // collegare (la collega: "se non non so con cosa collegare").
  applyLinkMode(on){
    const batch = [];
    for(const w of this.words){
      if(w.flags?.fromExpansion) continue;  // non illuminare i 27K cloni vasto
      const spec = buildNodeSpec(w, 'normal', { fieldId: this.id, forceLabel: !!on });
      batch.push(spec);
    }
    if(batch.length) this.nodesDS.update(batch);
  }

  // ---- Link focus: vista isolata dopo "A collegata a B" ----
  // Mostra SOLO: parole della frase + A + B + l'arco A→B. Tutto il resto
  // (parole e archi) viene nascosto via hidden:true. clearLinkFocus()
  // ripristina la visibilità.
  applyLinkFocus(focus){
    if(!focus){ this.clearLinkFocus(); return; }
    const keep = new Set();
    keep.add(focus.from);
    keep.add(focus.to);
    for(const w of this.words){
      if(w.flags?.fromSentence) keep.add(w.w);
    }
    const nodeBatch = this.words.map(w => ({ id: w.w, hidden: !keep.has(w.w) }));
    if(nodeBatch.length) this.nodesDS.update(nodeBatch);
    // Archi: mostra SOLO l'arco A→B, nascondi tutto il resto.
    const edgeBatch = this.edges.map(e => ({
      id: e.key,
      hidden: e.key !== focus.edgeKey,
    }));
    if(edgeBatch.length) this.edgesDS.update(edgeBatch);
  }

  clearLinkFocus(){
    const nodeBatch = this.words.map(w => ({ id: w.w, hidden: false }));
    if(nodeBatch.length) this.nodesDS.update(nodeBatch);
    // Archi: ripristina baseline (nascosti — visibili solo on hover/select).
    const baseline = this.baselineEdgeBatch();
    if(baseline.length) this.edgesDS.update(baseline);
  }

  // ---- Spreading iterativo per evitare sovrapposizioni ----
  // respectUserPositioned (default true): le parole con _userPositioned
  // restano FERME — sono i centri attorno a cui le altre si spostano per
  // fare spazio. Coerente con drag manuale e cluster delle estrazioni KG.
  spreadNonOverlapping({
    minDist = tokens.spread.minDist,
    iterations = tokens.spread.iterations,
    stepFactor = 0.5,
    respectUserPositioned = true,
  } = {}){
    const ws = this.words;
    if(ws.length < 2) return;

    for(let iter = 0; iter < iterations; iter++){
      let moved = 0;
      for(let i = 0; i < ws.length; i++){
        const a = ws[i];
        const ap = a.position;
        if(!ap) continue;
        for(let j = i + 1; j < ws.length; j++){
          const b = ws[j];
          const bp = b.position;
          if(!bp) continue;
          let dx = bp.x - ap.x;
          let dy = bp.y - ap.y;
          let d = Math.sqrt(dx * dx + dy * dy);
          if(d < 0.01){
            const ang = Math.random() * Math.PI * 2;
            dx = Math.cos(ang); dy = Math.sin(ang); d = 1;
          }
          if(d < minDist){
            const push = (minDist - d) * stepFactor * 0.5;
            const nx = dx / d, ny = dy / d;
            const aLocked = respectUserPositioned && a._userPositioned;
            const bLocked = respectUserPositioned && b._userPositioned;
            if(aLocked && bLocked){
              continue;  // entrambe fissate dall'utente: non toccare
            } else if(aLocked){
              bp.x += nx * push * 2; bp.y += ny * push * 2;  // tutto il push su b
            } else if(bLocked){
              ap.x -= nx * push * 2; ap.y -= ny * push * 2;  // tutto il push su a
            } else {
              ap.x -= nx * push; ap.y -= ny * push;
              bp.x += nx * push; bp.y += ny * push;
            }
            moved++;
          }
        }
      }
      if(moved === 0) break;
    }

    const batch = ws
      .filter(w => w.position)
      .map(w => ({ id: w.w, x: w.position.x, y: w.position.y }));
    if(batch.length) this.nodesDS.update(batch);
  }

  // ---- Serializzazione ----
  toJSON(){
    return {
      sentence: this.sentence || null,
      expansionShown: !!this.expansionShown,
      words: this.words.map(w => ({
        w: w.w, sig: w.sig, flags: { ...w.flags },
        userOct: w.userOct || null,
        displayName: w.displayName || null,
        sentenceIndex: w.sentenceIndex !== undefined ? w.sentenceIndex : null,
        // Persistiamo posizione + flag manuale così il drag dell'utente
        // sopravvive al refresh. Senza questi campi, il prossimo load
        // ricalcolerebbe tutto da firma → l'utente perde il setup.
        position: w.position ? { x: w.position.x, y: w.position.y } : null,
        _userPositioned: !!w._userPositioned,
      })),
      edges: this.edges.map(e => ({
        from: e.from, to: e.to, rel: e.rel, conf: e.conf,
        via: e.via || null,
        flags: { ...e.flags },
      })),
    };
  }

  // Snapshot completo: include posizioni 2D delle parole (toJSON le ricalcola).
  // Usato dall'undo/redo perché la posizione fa parte dello stato visivo.
  toSnapshot(){
    return {
      sentence: this.sentence || null,
      expansionShown: !!this.expansionShown,
      words: this.words.map(w => ({
        w: w.w,
        sig: w.sig.slice(),
        flags: { ...w.flags },
        userOct: w.userOct ? w.userOct.slice() : null,
        position: w.position ? { ...w.position } : null,
        displayName: w.displayName || null,
        // sentenceIndex deve sopravvivere a undo/redo: senza, le parole
        // della frase finiscono con index null → 999 nel sort di
        // applyRectangularLayout → ordine "vogliamo futuro bello" si
        // scombina in modo arbitrario.
        sentenceIndex: w.sentenceIndex !== undefined ? w.sentenceIndex : null,
        _userPositioned: !!w._userPositioned,
        _dimPos: w._dimPos ? { x: w._dimPos.x, y: w._dimPos.y } : null,
      })),
      edges: this.edges.map(e => ({
        from: e.from, to: e.to, rel: e.rel, conf: e.conf,
        via: e.via || null,
        flags: { ...e.flags },
      })),
    };
  }

  // Resetta lo stato del field preservando dim labels (gli unici nodi non
  // associati a parole). Usato prima di replaceFromSnapshot per non duplicare
  // dim labels e mantenere il network bound agli stessi DataSet.
  clear(){
    this.selected = null;
    this.subHover = null;
    this.currentRosa = null;
    this.isDimmed = false;
    this.navPath = [];
    this.sentence = null;
    this.expansionShown = false;
    if(this.words.length){
      try { this.nodesDS.remove(this.words.map(w => w.w)); } catch(_){}
    }
    if(this.edges.length){
      try { this.edgesDS.remove(this.edges.map(e => e.key)); } catch(_){}
    }
    this.words = [];
    this.edges = [];
    this.wordMap = Object.create(null);
    this.edgesByWord = Object.create(null);
    this.edgeByKey = Object.create(null);
  }

  // Sostituisce lo stato con quello di una snapshot (toSnapshot output).
  // Il network resta bound agli stessi DataSet — solo i contenuti cambiano.
  replaceFromSnapshot(data){
    this.clear();
    if(!data) return;
    if(data.sentence) this.sentence = data.sentence;
    if(data.expansionShown) this.expansionShown = true;
    this.bulkLoad({ words: data.words || [], edges: data.edges || [] });
  }

  // Rimuove in batch tutte le parole NON in `keepSet`, e tutti gli archi
  // con almeno un estremo non preservato. Usato dopo l'animazione di
  // comprensione frase per "potare" il personale dai nodi vasto-clone non
  // rilevanti (vedi roadmap UX §1 Fase C). Operazione one-shot: una sola
  // chiamata a nodesDS.remove(...)/edgesDS.remove(...) per le ~3K rimozioni.
  removeNonExpansionWords(keepSet){
    const wordsToRemove = this.words.filter(w => !keepSet.has(w.w));
    const edgesToRemove = this.edges.filter(e => !keepSet.has(e.from) || !keepSet.has(e.to));

    if(edgesToRemove.length){
      const ids = edgesToRemove.map(e => e.key);
      try { this.edgesDS.remove(ids); } catch(_){}
      const remSet = new Set(ids);
      for(const e of edgesToRemove){
        delete this.edgeByKey[e.key];
        this.edgesByWord[e.from]?.delete(e.key);
        this.edgesByWord[e.to]?.delete(e.key);
      }
      this.edges = this.edges.filter(e => !remSet.has(e.key));
    }
    if(wordsToRemove.length){
      const ids = wordsToRemove.map(w => w.w);
      try { this.nodesDS.remove(ids); } catch(_){}
      const remSet = new Set(ids);
      for(const w of wordsToRemove){
        delete this.wordMap[w.w];
        delete this.edgesByWord[w.w];
      }
      this.words = this.words.filter(w => !remSet.has(w.w));
    }
    // Ricalcola localDegree dopo le rimozioni.
    for(const w of this.words){
      w.localDegree = (this.edgesByWord[w.w] || new Set()).size;
    }
  }

  // Carica stato serializzato (accetta sia formato nuovo che legacy).
  hydrate(data){
    if(data.sentence) this.sentence = data.sentence;
    if(data.expansionShown) this.expansionShown = true;
    (data.words || []).forEach(w => this.addWord(w));
    (data.edges || []).forEach(e => {
      if(!this.wordMap[e.from]){
        this.addWord({ w: e.from, sig: NEUTRAL_SIG.slice(), flags: { userCreated: true } });
      }
      if(!this.wordMap[e.to]){
        this.addWord({ w: e.to, sig: NEUTRAL_SIG.slice(), flags: { userCreated: true } });
      }
      this.addEdge(e);
    });
  }
}
