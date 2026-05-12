// Undo/redo per il field editabile (personale). Modulo foglia: nessun
// import dai moduli che lo chiamano (vedi CLAUDE.md §6 — niente cicli).
//
// Modello: il field ha tre stati — past[], last, future[]. Ogni mutazione
// commit-tata via saveField chiama record() che spinge `last` su past, vuota
// future, aggiorna `last`. undo() ribalta past → last → future; redo() viceversa.
//
// L'applicazione effettiva di una snapshot al field è iniettata dall'esterno
// via setApplyFn() — qui non sappiamo cosa sia un Field, e va benissimo così.

const STACKS = {
  nuovo: { past: [], future: [], last: null },
};
const MAX_DEPTH = 50;

let _replaying = false;
let _applyFn   = null;
let _onChange  = null;

export function setApplyFn(fn){ _applyFn = fn; }
export function setOnChange(fn){ _onChange = fn; }
export function isReplaying(){ return _replaying; }

// Inizializza la baseline di un field. Chiamato dopo creazione/idratazione:
// la prima mutazione successiva avrà la baseline come stato di "annulla".
export function initBaseline(id, snapshot){
  const s = STACKS[id];
  if(!s) return;
  s.past = [];
  s.future = [];
  s.last = snapshot;
  _onChange?.();
}

// Registra una nuova snapshot post-mutazione. Skippa durante il replay
// (undo/redo non devono creare nuovo storico).
export function record(id, snapshot){
  if(_replaying) return;
  const s = STACKS[id];
  if(!s) return;
  if(s.last !== null){
    s.past.push(s.last);
    if(s.past.length > MAX_DEPTH) s.past.shift();
  }
  s.future = [];
  s.last = snapshot;
  _onChange?.();
}

export function canUndo(id){ return !!STACKS[id]?.past.length; }
export function canRedo(id){ return !!STACKS[id]?.future.length; }

export function undo(id){
  const s = STACKS[id];
  if(!s || !s.past.length) return false;
  s.future.push(s.last);
  s.last = s.past.pop();
  _replaying = true;
  try { _applyFn?.(id, s.last); } finally { _replaying = false; }
  _onChange?.();
  return true;
}

export function redo(id){
  const s = STACKS[id];
  if(!s || !s.future.length) return false;
  s.past.push(s.last);
  s.last = s.future.pop();
  _replaying = true;
  try { _applyFn?.(id, s.last); } finally { _replaying = false; }
  _onChange?.();
  return true;
}
