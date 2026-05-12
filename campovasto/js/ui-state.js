// Stato UI transient condiviso tra moduli che non appartiene a un Field
// specifico. Single source of truth — sostituisce il vecchio uso di
// window._uir* (vedi CLAUDE.md §4).

import { LS } from './constants.js';

// Filtri attivi sul campo vasto (solo vasto: nuovo/medio non hanno filtri).
const filterState = {
  // Set<string> | null — parole che passano dim/range (null = niente filtro)
  matchedWords: null,
  // Set<edgeId> | null — archi da mostrare quando il filtro è attivo
  allowedEdges: null,
  // 'both' | 'in' | 'out' — modificatore della direzione archi al hover.
  // Default 'out': con entrambi entranti+uscenti il vasto diventa illeggibile
  // (parole hub come "essere", "qualità" hanno migliaia di archi entranti).
  // Mostrare solo gli uscenti dà priorità alla narrazione che parte dalla
  // parola selezionata.
  direction: 'out',
};

export function getMatchedWords(){ return filterState.matchedWords; }
export function setMatchedWords(s){ filterState.matchedWords = s; }
export function isFilterActive(){ return filterState.matchedWords != null; }

export function getAllowedEdges(){ return filterState.allowedEdges; }
export function setAllowedEdges(s){ filterState.allowedEdges = s; }

export function getFilterDirection(){ return filterState.direction; }
export function setFilterDirection(d){ filterState.direction = d; }

// Reset totale: usato quando l'utente azzera i filtri.
export function resetFilterState(){
  filterState.matchedWords = null;
  filterState.allowedEdges = null;
  filterState.direction = 'both';
}

// Indica se una parola (id) passa il filtro corrente. Se nessun filtro è
// attivo, tutte le parole "passano" (true).
export function wordPassesFilter(id){
  if(!filterState.matchedWords) return true;
  return filterState.matchedWords.has(id);
}

// ---- Filtro per tipo di relazione (legenda interattiva) ------------------
// Set<string> dei tipi (RelationType, es. 'IS_A', 'CAUSES') attualmente
// abilitati. null = stato iniziale (tutti abilitati). Un tipo NON in set
// significa che gli archi di quel tipo non vengono visualizzati al hover
// né nell'overlay animato.
let _enabledRelTypes = null;

export function isRelTypeEnabled(rel){
  if(_enabledRelTypes == null) return true;
  return _enabledRelTypes.has(rel);
}

export function getEnabledRelTypes(){ return _enabledRelTypes; }

export function setEnabledRelTypes(set){
  _enabledRelTypes = set instanceof Set ? new Set(set) : null;
}

// Toggle di un singolo tipo: se era enabled, lo rimuove; altrimenti aggiunge.
// Inizializza il set con TUTTI i tipi alla prima toggle (passandoli come
// `allTypes`) così la transizione da "tutti enabled" a "uno disabled" è naturale.
export function toggleRelType(rel, allTypes){
  if(_enabledRelTypes == null){
    _enabledRelTypes = new Set(allTypes || []);
  }
  if(_enabledRelTypes.has(rel)) _enabledRelTypes.delete(rel);
  else _enabledRelTypes.add(rel);
}

// Toggle di un'intera categoria (es. tutti i tipi 'S'). Se almeno un tipo del
// gruppo è disabilitato, abilita tutti; altrimenti disabilita tutti. Più
// intuitivo del classico XOR: il click "ripristina sempre la categoria piena"
// se è parzialmente attiva.
export function toggleRelGroup(groupTypes, allTypes){
  if(_enabledRelTypes == null) _enabledRelTypes = new Set(allTypes || []);
  const allOn = groupTypes.every(t => _enabledRelTypes.has(t));
  if(allOn) groupTypes.forEach(t => _enabledRelTypes.delete(t));
  else      groupTypes.forEach(t => _enabledRelTypes.add(t));
}

// ---- Modalità di layout del campo nuovo ---------------------------------
// 'dimensional': layout originale, parole posizionate dalle 8 dimensioni.
// 'rectangular': parole-frase in alto, 5 fasce per gruppo relazione sotto
// ciascuna, 6° fascia in basso per le orfane.
// Persistito in LS.NUOVO_LAYOUT — la scelta dell'utente sopravvive al refresh.
let _nuovoLayout = (() => {
  try {
    const v = localStorage.getItem(LS.NUOVO_LAYOUT);
    return (v === 'rectangular' || v === 'dimensional') ? v : 'dimensional';
  } catch(_){ return 'dimensional'; }
})();

export function getNuovoLayout(){ return _nuovoLayout; }

export function setNuovoLayout(mode){
  if(mode !== 'dimensional' && mode !== 'rectangular') return;
  _nuovoLayout = mode;
  try { localStorage.setItem(LS.NUOVO_LAYOUT, mode); } catch(_){}
}

// ---- Modalità collegamento (linkMode) -----------------------------------
// Stato UI transient attivato dal tab "collega" della modale aggiungi
// parola. Mentre attivo, il campo locale (nuovo) mostra le ETICHETTE di
// TUTTE le parole (non solo della frase) — la collega ha chiesto:
// "devono vedersi le etichette delle parole sul campo se non non so con
// cosa collegare". Il drag-to-connect (graph.js) è il flow principale.
let _linkMode = false;
const _linkModeListeners = new Set();
export function isLinkMode(){ return _linkMode; }
export function setLinkMode(on){
  const next = !!on;
  if(next === _linkMode) return;
  _linkMode = next;
  _linkModeListeners.forEach(fn => { try { fn(next); } catch(_){} });
}
export function onLinkModeChange(fn){ _linkModeListeners.add(fn); return () => _linkModeListeners.delete(fn); }

// ---- Focus dopo collegamento (linkFocus) --------------------------------
// Dopo che A→B è stato creato, restano visibili SOLO: parole della frase,
// A, B, e l'arco A→B. Tutto il resto è hidden. Si esce con click vuoto o
// con cambio campo. Richiesto dalla collega: "Dopo aver collegato la parola
// A e B vedrò sul campo solo le parole della frase (senza archi) e le
// parole che ho appena collegato unite dal loro arco".
let _linkFocus = null;  // { from: string, to: string, edgeKey: string } | null
const _linkFocusListeners = new Set();
export function getLinkFocus(){ return _linkFocus; }
export function setLinkFocus(focus){
  _linkFocus = focus || null;
  _linkFocusListeners.forEach(fn => { try { fn(_linkFocus); } catch(_){} });
}
export function clearLinkFocus(){ setLinkFocus(null); }
export function onLinkFocusChange(fn){ _linkFocusListeners.add(fn); return () => _linkFocusListeners.delete(fn); }
