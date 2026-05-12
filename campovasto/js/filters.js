// Filtri per il campo vasto: dimensione dominante (multi-select) +
// range per ogni componente della firma. Pannello a scomparsa.
//
// La struttura statica del pannello vive in index.html (regole di design.md §11).
// Qui restano solo le parti dinamiche: 8 chip dimensione, 8 righe range firma,
// e il wiring eventi.
//
// Lo stato dei filtri condiviso con graph.js vive in ui-state.js (CLAUDE.md §4).

import { DIM_NAMES, DIM_DESC } from './constants.js';
import { DIM_COLORS, dominantDim } from './theme.js';
import { get } from './manager.js';
import { applyHighlight, clearHighlight } from './graph.js';
import { buildNodeSpec } from './node-style.js';
import {
  setMatchedWords, setAllowedEdges, setFilterDirection, resetFilterState,
  isFilterActive as _isFilterActive, isRelTypeEnabled,
} from './ui-state.js';

// Stato dei controlli (non dei nodi filtrati).
const state = {
  dims: new Set(),
  ranges: new Array(8).fill(null).map(() => [0, 100]),
  direction: 'both',
  active: false,
};

export function isActive(){ return _isFilterActive() || state.direction !== 'both'; }

// Parsing dei poli da DIM_DESC. Formato canonico: "POLO_HIGH o POLO_LOW".
// Esempio: "movimento o inerzia" → ['movimento', 'inerzia']  (high, low)
function poliFromDesc(desc){
  const parts = (desc || '').split(' o ');
  if(parts.length !== 2) return ['', ''];
  return [parts[0], parts[1]];
}

export function buildFilterPanel(){
  const panel = document.getElementById('filtri');
  if(!panel) return;

  // Direzione: 3 chip vivono in #relazioni (sezione separata). Il wiring sta
  // qui perché filters.js è il padrone della direzione filtro, ma il selettore
  // attraversa due section diverse — usare document, non panel.
  document.querySelectorAll('#dirFilter button.dir').forEach(b => {
    b.onclick = () => {
      state.direction = b.dataset.dir;
      document.querySelectorAll('#dirFilter button.dir').forEach(x =>
        x.classList.toggle('attivo', x.dataset.dir === state.direction));
      setFilterDirection(state.direction);
      if(anyConstraint()) applyDebounced();
      // Se c'è una parola selezionata in vasto, ricalcola l'highlight subito —
      // altrimenti la nuova direzione si vedrebbe solo al prossimo hover.
      const F = get('vasto');
      if(F && F.selected){
        applyHighlight(F, F.selected, F.getRosa(F.selected, state.direction, { filterByType: isRelTypeEnabled }));
      }
    };
  });

  // Iniezione delle 8 righe range firma (replica per dato).
  // Layout per riga: [nome] [switch] [cursore range] [lettura editabile].
  // Lo switch è il toggle "filtra per dimensione dominante" (rimpiazza i
  // chip della precedente sezione "dimensione dominante").
  // I poli del cursore vengono dal formato "POLO_HIGH o POLO_LOW" di DIM_DESC.
  const sigEl = panel.querySelector('#sigRanges');
  for(let i = 0; i < 8; i++){
    const [lo, hi] = state.ranges[i];
    const [poloHigh, poloLow] = poliFromDesc(DIM_DESC[i]);
    // Al caricamento: nessuna dimensione esclusa → tutti i toggle ON (attivo).
    const attivo = state.dims.has(i) ? '' : ' attivo';
    const row = document.createElement('div');
    row.className = 'riga firma';
    row.style.setProperty('--dim-color', DIM_COLORS[i]);
    row.innerHTML = `
      <span class="nome">${DIM_NAMES[i]}</span>
      <button class="switch${attivo}" data-dim="${i}" title="filtra per dimensione dominante"></button>
      <div class="cursore range" data-dim="${i}">
        <span class="polo sinistra">${poloLow}</span>
        <span class="polo destra">${poloHigh}</span>
        <div class="riempi" id="trackfill_${i}" style="left:${lo}%; width:${hi-lo}%"></div>
      </div>
      <span class="lettura range">
        <input type="number" min="0" max="100" data-dim="${i}" data-kind="lo" value="${lo}">–<input type="number" min="0" max="100" data-dim="${i}" data-kind="hi" value="${hi}">
      </span>
    `;
    sigEl.appendChild(row);
  }
  _wireRangeDrag(panel);
  _wireRangeInputs(panel);
  _wireSwitch(panel);

  // Azzera (esiste già in index.html).
  panel.querySelector('#filterReset').onclick = () => reset();
}

// Debounce: più cambi ravvicinati → una sola apply.
let _applyTimer = null;
function applyDebounced(){
  if(_applyTimer) clearTimeout(_applyTimer);
  _applyTimer = setTimeout(() => { _applyTimer = null; apply(); }, 70);
}

// Wiring del toggle .switch nelle righe firma — replica con click delegation
// la logica dei vecchi chip dimensione (state.dims add/delete + apply).
let _switchWired = false;
function _wireSwitch(panel){
  if(_switchWired) return;
  _switchWired = true;
  panel.addEventListener('click', (e) => {
    const sw = e.target.closest('button.switch');
    if(!sw || !panel.contains(sw)) return;
    const dim = parseInt(sw.dataset.dim);
    if(isNaN(dim)) return;
    const row = sw.closest('.riga.firma');
    if(state.dims.has(dim)){
      // Era esclusa: riattiva
      state.dims.delete(dim);
      sw.classList.add('attivo');
      if(row) row.classList.remove('inattivo');
    } else {
      // Era attiva: escludi
      state.dims.add(dim);
      sw.classList.remove('attivo');
      if(row) row.classList.add('inattivo');
    }
    applyDebounced();
  });
}

// Sincronizza i due input numerici di una dimensione con lo stato corrente.
function _syncRangeInputs(dim){
  const [lo, hi] = state.ranges[dim];
  document.querySelectorAll(`#sigRanges .lettura input[data-dim="${dim}"]`).forEach(inp => {
    const target = inp.dataset.kind === 'lo' ? lo : hi;
    if(document.activeElement !== inp && parseInt(inp.value) !== target){
      inp.value = target;
    }
  });
}

// Drag su .cursore.range — replica il pattern del .cursore.singolo
// (dim-editor.js): mousedown sul container avvia il drag dell'estremo lo o
// hi più vicino al click; mousemove aggiorna; mouseup commit-ta. Listener
// globali una sola volta per istanza, delegation sul mousedown.
let _rangeDragWired = false;
function _wireRangeDrag(panel){
  if(_rangeDragWired) return;
  _rangeDragWired = true;

  let active = null;   // { dim, kind, cur, fill }

  const updateFromEvent = (e) => {
    if(!active) return;
    const rect = active.cur.getBoundingClientRect();
    const px = Math.min(100, Math.max(0, ((e.clientX - rect.left) / rect.width) * 100));
    const v = Math.round(px);
    let [lo, hi] = state.ranges[active.dim];
    if(active.kind === 'lo') lo = Math.min(v, hi);
    else                     hi = Math.max(v, lo);
    state.ranges[active.dim] = [lo, hi];
    active.fill.style.left  = lo + '%';
    active.fill.style.width = (hi - lo) + '%';
    _syncRangeInputs(active.dim);
    applyDebounced();
  };

  panel.addEventListener('mousedown', (e) => {
    // Skip se mousedown è su un input editabile (lasciamo il focus al campo).
    if(e.target.closest('.lettura input')) return;
    const cur = e.target.closest('.cursore.range');
    if(!cur || !panel.contains(cur)) return;
    const dim = parseInt(cur.dataset.dim);
    if(isNaN(dim)) return;
    const rect = cur.getBoundingClientRect();
    const px = ((e.clientX - rect.left) / rect.width) * 100;
    const [lo, hi] = state.ranges[dim];
    // Aggancio l'estremo più vicino al click — jump-to anche senza drag.
    active = {
      dim,
      kind: Math.abs(px - lo) <= Math.abs(px - hi) ? 'lo' : 'hi',
      cur,
      fill: cur.querySelector('.riempi'),
    };
    updateFromEvent(e);
    e.preventDefault();
  });
  window.addEventListener('mousemove', updateFromEvent);
  window.addEventListener('mouseup', () => { active = null; });
}

// Wiring degli input numerici editabili in .lettura.range — si comportano come
// la lettura del .cursore.singolo (vedi dim-editor.js): valori clampati allo
// stato, l'altro estremo limita il movimento di questo.
let _rangeInputsWired = false;
function _wireRangeInputs(panel){
  if(_rangeInputsWired) return;
  _rangeInputsWired = true;

  panel.addEventListener('input', (e) => {
    const inp = e.target.closest('.lettura input');
    if(!inp) return;
    const dim = parseInt(inp.dataset.dim);
    if(isNaN(dim)) return;
    let v = parseInt(inp.value);
    if(isNaN(v)) return;
    v = Math.min(100, Math.max(0, v));
    let [lo, hi] = state.ranges[dim];
    if(inp.dataset.kind === 'lo') lo = Math.min(v, hi);
    else                          hi = Math.max(v, lo);
    state.ranges[dim] = [lo, hi];
    const fill = document.getElementById('trackfill_' + dim);
    if(fill){ fill.style.left = lo + '%'; fill.style.width = (hi - lo) + '%'; }
    _syncRangeInputs(dim);
    applyDebounced();
  });

  // Al blur, normalizza il valore mostrato (es. "abc" → torna allo stato).
  panel.addEventListener('change', (e) => {
    const inp = e.target.closest('.lettura input');
    if(!inp) return;
    const dim = parseInt(inp.dataset.dim);
    if(isNaN(dim)) return;
    _syncRangeInputs(dim);
  });
}

function passes(w){
  // state.dims = set di dimensioni ESCLUSE (switch off → aggiunta).
  // Una parola con dim dominante in state.dims viene filtrata via.
  if(state.dims.has(dominantDim(w.sig))) return false;
  for(let i = 0; i < 8; i++){
    const v = w.sig?.[i] ?? 50;
    const [lo, hi] = state.ranges[i];
    if(v < lo || v > hi) return false;
  }
  return true;
}

function anyConstraint(){
  if(state.dims.size) return true;
  return state.ranges.some(([lo, hi]) => lo > 0 || hi < 100);
}

export function apply(){
  const F = get('vasto');
  if(!F) return;

  const hasNodeFilter = state.dims.size > 0 || state.ranges.some(([lo, hi]) => lo > 0 || hi < 100);
  state.active = hasNodeFilter;
  setFilterDirection(state.direction);

  if(!hasNodeFilter){
    // Reset: tutti i nodi al variant 'normal'; archi come baseline.
    setMatchedWords(null);
    setAllowedEdges(null);
    const batch = F.words.map(w => buildNodeSpec(w, 'normal', { fieldId: F.id }));
    if(batch.length) F.nodesDS.update(batch);
    const edgeBaseline = F.baselineEdgeBatch();
    if(edgeBaseline.length) F.edgesDS.update(edgeBaseline);
    return;
  }

  const matched = new Set();
  F.words.forEach(w => { if(passes(w)) matched.add(w.w); });
  setMatchedWords(matched);

  F.selected = null;
  F.currentRosa = null;
  F.subHover = null;

  // Matched → normal; non-matched → filterDim (visibili ma attenuati).
  const batch = F.words.map(w => {
    const variant = matched.has(w.w) ? 'normal' : 'filterDim';
    return buildNodeSpec(w, variant, { fieldId: F.id });
  });
  F.nodesDS.update(batch);

  // Archi: baseline (nascosti, on-hover); il modificatore di direzione agisce
  // solo quando l'utente passa sopra una parola.
  const edgeBaseline = F.baselineEdgeBatch();
  if(edgeBaseline.length) F.edgesDS.update(edgeBaseline);
}

export function reset(silent = false){
  const F = get('vasto');
  if(!F) return;
  state.dims.clear();
  state.ranges = new Array(8).fill(null).map(() => [0, 100]);
  state.direction = 'both';
  state.active = false;
  resetFilterState();

  if(!silent){
    // Tutti ON di default: state.dims vuoto = nessuna esclusione → switch attivi.
    document.querySelectorAll('#sigRanges button.switch').forEach(o => o.classList.add('attivo'));
    document.querySelectorAll('#sigRanges .riga.firma').forEach(r => r.classList.remove('inattivo'));
    document.querySelectorAll('#dirFilter button.dir').forEach(c => c.classList.toggle('attivo', c.dataset.dir === 'both'));
    document.querySelectorAll('#sigRanges .riempi').forEach(fill => {
      fill.style.left = '0%';
      fill.style.width = '100%';
    });
    for(let i = 0; i < 8; i++) _syncRangeInputs(i);
  }
  clearHighlight(F);
}
