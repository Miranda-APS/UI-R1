// Estrazione on-demand delle relazioni dal KG verso un Field.
// Usato quando il campo nuovo contiene parole "nude" (senza archi) e
// l'utente sceglie di importare le relazioni dal KG, eventualmente filtrando
// per gruppo di relazione (strutturale / causale / semantica / ...).
//
// Flusso:
//   1. fetchOutgoingForWord(word) chiama /api/biennale/word per i vicini KG
//   2. extractRelationsForWord(F, word, opts) filtra + aggiunge a F
//   3. extractRelationsForField(F, opts) itera su tutte le parole del field
//
// I target non presenti nel field vengono aggiunti come parole nuove
// (sig presa da vasto se disponibile, altrimenti neutra).

import { NEUTRAL_SIG, REL_GROUP } from './constants.js';
import { FIELDS } from './manager.js';
import { flagsForNewWord, flagsForNewEdge } from './policies/word.js';

// ---- Gruppi di relazione: label italiano per la UI di filtro ----
// Le chiavi corrispondono ai valori in REL_GROUP[rel].
export const GROUP_LABELS = {
  S: { label: 'strutturale', desc: 'è un, parte di' },
  C: { label: 'causale',     desc: 'causa, abilita, richiede, diventa' },
  M: { label: 'semantica',   desc: 'simile, opposto, simbolizza, contesto' },
  F: { label: 'fenomenologica', desc: 'ha, fa, sente, ricorda' },
  L: { label: 'logica',      desc: 'usato per, esprime, implica' },
};
export const ALL_GROUPS = ['S', 'C', 'M', 'F', 'L'];

function relPassesFilter(rel, allowedGroups){
  const g = REL_GROUP[rel] || 'L';
  return allowedGroups.has(g);
}

function confToInt(c){
  return Math.round(Math.min(1, Math.max(0, (c == null ? 0.5 : c))) * 100);
}

// ---- Fetch dal KG: relazioni uscenti di una parola ---------------------
// Risposta: array piatto { rel, target, conf } pronto da filtrare.
async function fetchOutgoingForWord(word){
  try {
    const r = await fetch('/api/biennale/word?word=' + encodeURIComponent(word));
    if(!r.ok) return [];
    const data = await r.json();
    // L'API restituisce neighbors uscenti + entranti (le entranti hanno
    // rel prefissato con "←"). Il personale deve contenere SOLO le
    // relazioni uscenti dalle parole — filtriamo le entranti qui.
    return (data.neighbors || [])
      .filter(n => typeof n.rel === 'string' && !n.rel.startsWith('←'))
      .map(n => ({ rel: n.rel, target: n.w, conf: n.conf }));
  } catch(e){
    console.warn('[extract] fetchOutgoingForWord failed', word, e);
    return [];
  }
}

// ---- Estrazione per UNA parola del field --------------------------------
// I target non presenti nel field vengono aggiunti se addNewTargets=true
// (default). La sig dei target viene presa da vasto se disponibile.
// Restituisce { addedWords, addedEdges, skippedRel } per feedback UI.
export async function extractRelationsForWord(F, word, opts = {}){
  const { allowedGroups = new Set(ALL_GROUPS), addNewTargets = true } = opts;
  if(!F.hasWord(word)) return { addedWords: 0, addedEdges: 0, skippedRel: 0 };

  const V = FIELDS.vasto;
  const outgoing = await fetchOutgoingForWord(word);

  let addedWords = 0, addedEdges = 0, skippedRel = 0;
  for(const e of outgoing){
    if(!relPassesFilter(e.rel, allowedGroups)){ skippedRel++; continue; }
    if(!F.hasWord(e.target)){
      if(!addNewTargets) continue;
      const vw = V?.wordMap?.[e.target];
      const sig = vw ? (vw.sig || NEUTRAL_SIG).slice() : NEUTRAL_SIG.slice();
      F.addWord({
        w: e.target, sig,
        flags: flagsForNewWord(F, { fromApi: !vw }),
      });
      addedWords++;
    }
    const key = `${word}|${e.target}|${e.rel}`;
    const before = F.edgeByKey[key];
    F.addEdge({
      from: word, to: e.target, rel: e.rel,
      conf: confToInt(e.conf),
      flags: flagsForNewEdge(F),
    });
    if(!before) addedEdges++;
  }
  return { addedWords, addedEdges, skippedRel };
}

// ---- Estrazione per TUTTE le parole del field ---------------------------
// Esegue le fetch in parallelo (il browser limita a ~6 connessioni per host).
// Per personale tipico (~10–50 parole) è istantaneo.
export async function extractRelationsForField(F, opts = {}){
  const words = F.words.map(w => w.w).filter(w => !String(w).startsWith('_'));
  const results = await Promise.all(words.map(w => extractRelationsForWord(F, w, opts)));
  const total = { addedWords: 0, addedEdges: 0, skippedRel: 0, perWord: words.length };
  for(const r of results){
    total.addedWords += r.addedWords;
    total.addedEdges += r.addedEdges;
    total.skippedRel += r.skippedRel;
  }
  return total;
}
