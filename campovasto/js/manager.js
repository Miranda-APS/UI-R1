// Registry dei due field + persistenza localStorage + transmit personale→vasto.
// Unica sorgente di verità su "quale field è attivo" (vedi CLAUDE.md §4).
//
// Modello: due campi.
//   - vasto: la galassia (~3000 nodi top-degree dal KG, read-only)
//   - personale: il campo dell'utente — può essere vuoto, o avere una
//                `sentence` se è stato creato da una frase. La sentence
//                attiva la modalità "comprensione frase" (vedi roadmap UX §1).

import { LS, LS_SCHEMA_VERSION, NEUTRAL_SIG } from './constants.js';
import { initBaseline, record, isReplaying } from './history.js';

export const FIELDS = { vasto: null, nuovo: null };
export let activeId = 'vasto';

export function get(id){ return FIELDS[id]; }
export function getActive(){ return FIELDS[activeId]; }
export function setActive(id){ activeId = id; }

// Aggancia un Field al registry e inizializza la sua baseline di history.
// Usare al posto di `FIELDS[id] = F` per garantire che ogni nuovo field
// editabile abbia uno stato di partenza per l'undo (vedi history.js).
export function registerField(id, F){
  FIELDS[id] = F;
  if(id === 'nuovo'){
    initBaseline(id, F ? F.toSnapshot() : null);
  }
}

// ---- Persistenza ----

function lsKeyFor(id){
  return id === 'nuovo' ? LS.NUOVO : null;
}

// opts.silent: true → niente record nello storico (usato dall'apply di
// undo/redo per persistere senza creare un nuovo entry).
// Wrapper schema-versioned: il payload va sempre dentro `{ v, data, ts }`
// così loadField può scartare versioni vecchie senza ambiguità.
export function saveField(id, opts = {}){
  const F = FIELDS[id];
  const key = lsKeyFor(id);
  if(!F || !key) return;
  try {
    const envelope = { v: LS_SCHEMA_VERSION, ts: Date.now(), data: F.toJSON() };
    localStorage.setItem(key, JSON.stringify(envelope));
  } catch(_){}
  if(!opts.silent && !isReplaying() && id === 'nuovo'){
    record(id, F.toSnapshot());
  }
}

export function loadField(id){
  const key = lsKeyFor(id);
  if(!key) return null;
  try {
    const raw = localStorage.getItem(key);
    if(!raw) return null;
    const parsed = JSON.parse(raw);
    // Envelope schema-versioned: scarta payload con version mismatch.
    if(parsed && typeof parsed === 'object' && 'v' in parsed){
      if(parsed.v !== LS_SCHEMA_VERSION){
        console.info(`[manager] localStorage ${key} schema v${parsed.v} → atteso v${LS_SCHEMA_VERSION}, scartato.`);
        try { localStorage.removeItem(key); } catch(_){}
        return null;
      }
      return parsed.data || null;
    }
    // Payload legacy senza envelope: lo scartiamo per non riportare a galla
    // stati con flag obsoleti (fromExpansion=true salvato per sbaglio, ecc.).
    console.info(`[manager] localStorage ${key} legacy senza envelope, scartato.`);
    try { localStorage.removeItem(key); } catch(_){}
    return null;
  } catch(_){ return null; }
}

// ---- Transmit ----

// Trasmette al KG vero del server le parole/archi non ancora trasmessi.
// UN SOLO endpoint: POST /api/community/transmit_batch — il server processa
// tutto in un solo comando engine ed esegue le operazioni costose
// (recompute_all_word_affinities, build_semantic_simplices, cura_save)
// UNA volta sola alla fine. Drasticamente più veloce del flusso 1-by-1
// (5 parole + 10 archi: >10s → <1s).
//
// La parte "in-memory" (V.addWord, V.addEdge) resta come ottimizzazione
// visiva: l'utente vede subito la parola nel vasto senza dover ri-fetchare.
// Verità però è server-side; al prossimo reload il vasto la carica dal KG.
export async function transmitToVasto(){
  const S = FIELDS.nuovo;
  const V = FIELDS.vasto;
  if(!S || !V) return { words: 0, edges: 0, errors: 0 };

  // Filtra: solo parole utente non trasmesse, non vasto-clone, non unknown.
  const wordsToSend = S.words.filter(sw =>
    !sw.flags.transmitted && !sw.flags.unknown
    && !sw.flags.fromExpansion && sw.flags.userCreated
  );
  const edgesToSend = S.edges.filter(se =>
    !se.flags.transmitted && !se.flags.fromExpansion
  );

  if(wordsToSend.length === 0 && edgesToSend.length === 0){
    return { words: 0, edges: 0, errors: 0 };
  }

  // Payload batch: parole con firma 8D normalizzata [0..1], archi con
  // strength 1..5 (scala backend).
  const payload = {
    words: wordsToSend.map(sw => ({
      text: sw.w,
      firma: (sw.sig || NEUTRAL_SIG).map(v => Math.max(0, Math.min(1, v / 100))),
    })),
    edges: edgesToSend.map(se => ({
      subject: se.from,
      relation: se.rel,
      object: se.to,
      strength: Math.max(1, Math.min(5, Math.round((se.conf || 80) / 20))),
    })),
    user_name: 'campovasto',
  };

  let result;
  try {
    const r = await fetch('/api/community/transmit_batch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    if(!r.ok) throw new Error('transmit_batch HTTP ' + r.status);
    result = await r.json();
  } catch(e){
    console.error('[transmit] batch failed:', e);
    return { words: 0, edges: 0, errors: wordsToSend.length + edgesToSend.length };
  }

  // Mirror in-memory + flag transmitted in base a quanto il server ha
  // accettato. words_ok contiene i testi delle parole salvate.
  const wordsOkSet = new Set(result.words_ok || []);
  let wn = 0, en = 0, errors = 0;
  for(const sw of wordsToSend){
    if(!wordsOkSet.has(sw.w)){ errors++; continue; }
    if(V.hasWord(sw.w)){
      V.updateWordSig(sw.w, sw.sig || NEUTRAL_SIG);
    } else {
      V.addWord({
        w: sw.w,
        sig: (sw.sig || NEUTRAL_SIG).slice(),
        flags: { userCreated: true, transmitted: true },
        deg: 0,
      });
    }
    sw.flags.transmitted = true;
    wn++;
  }
  // edges_ok dal server è un conteggio (non lista); applichiamo
  // ottimisticamente a tutti gli archi inviati che hanno entrambi gli
  // estremi nel vasto. Server-side errors_count è in result.edges_err.
  const expectedEdgesOk = result.edges_ok || 0;
  let edgeMirrored = 0;
  for(const se of edgesToSend){
    if(edgeMirrored >= expectedEdgesOk) break;
    if(!V.hasWord(se.from) || !V.hasWord(se.to)) continue;
    V.addEdge({ from: se.from, to: se.to, rel: se.rel, conf: se.conf || 80 });
    se.flags.transmitted = true;
    en++;
    edgeMirrored++;
  }
  errors += result.edges_err || 0;
  if(result.elapsed_ms !== undefined){
    console.info(`[transmit] server: ${result.elapsed_ms}ms — ${wn} parole, ${en} archi`);
  }

  saveField('nuovo');
  return { words: wn, edges: en, errors };
}

// Conteggio di elementi ancora da trasmettere nel personale. Allineato ai
// filtri del transmit loop: fromExpansion e parole non userCreated NON
// contano (sono cloni del vasto, non contributo dell'utente).
export function pendingCount(){
  const S = FIELDS.nuovo;
  if(!S) return 0;
  let n = 0;
  S.words.forEach(w => {
    if(w.flags.transmitted || w.flags.unknown) return;
    if(w.flags.fromExpansion) return;
    if(!w.flags.userCreated) return;
    n++;
  });
  S.edges.forEach(e => {
    if(e.flags.transmitted) return;
    if(e.flags.fromExpansion) return;
    n++;
  });
  return n;
}
