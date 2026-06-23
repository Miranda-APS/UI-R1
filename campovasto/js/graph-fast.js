// graph-fast.js — DROP-IN per graph.js (clone "scheggia").
//
// Stessa interfaccia pubblica di graph.js:
//   initNetwork, mountField, setHandlers, applyHighlight, clearHighlight,
//   restoreSelection  +  export let network  (shim coi metodi vis usati dalla chrome).
//
// Renderer EVENT-DRIVEN (l'insight di Francesco): non si ridisegna a orologeria.
//   - FONDALE: tutti i ~21k dot rasterizzati UNA volta in un bitmap offscreen.
//     Pan/zoom = un solo drawImage (O(1)). Si rigenera SOLO su evento
//     (cambio dati/filtri via F.nodesDS, cambio campo).
//   - LIVE: solo il vicinato (selezione/hover) sul layer vivo, sopra un velo
//     che attenua il fondale a costo O(1) (niente update dei 21k nodi).
//   - overlay.js (archi animati + label) riusato INTATTO: si aggancia al nostro
//     `afterDrawing` (ctx in coordinate-mondo, come fa vis-network).
//
// Le posizioni sono quelle già calcolate da Field (look IDENTICO a campovasto).
// Colore/taglia da node-style.js + theme.js (fonte unica, CLAUDE.md §1/§2).

import { getActive, setActive, FIELDS, saveField } from './manager.js';
import { colorForSig, UI } from './theme.js';
import { sizeFor } from './node-style.js';
import { REL_GROUP, GROUP_DASH } from './constants.js';
import {
  getMatchedWords, isFilterActive, getFilterDirection, isRelTypeEnabled, getNuovoLayout,
} from './ui-state.js';
import {
  initOverlay, setActiveTraceEdges, setActiveLabels, clearOverlay,
} from './components/overlay.js';
import { openCtxMenu } from './components/ctx-menu.js';

export let network = null;
let _handlers = {};

// ---- Canvas / contesti / container ----------------------------------------
let _container = null;
let _back = null, _live = null;   // fondale bitmap / layer vivo
let _bx = null, _lx = null;
let W = 0, H = 0;                  // dimensioni canvas in px

// ---- View: schermo(px locali) = mondo*k + t -------------------------------
const view = { k: 1, tx: 0, ty: 0 };
const worldToScreen = (wx, wy) => [wx * view.k + view.tx, wy * view.k + view.ty];
const screenToWorld = (sx, sy) => [(sx - view.tx) / view.k, (sy - view.ty) / view.k];

// ---- Bounds mondo (quadrati) + bitmap fondale -----------------------------
let minX = 0, maxX = 1, minY = 0, maxY = 1;
const BW = 4096, BH = 4096;
let _bmp = null;            // canvas offscreen del fondale
let _bmpFieldId = null;     // per quale campo è valido il bitmap
const worldToBmp = (wx, wy) => [ (wx - minX) / (maxX - minX) * BW, (wy - minY) / (maxY - minY) * BH ];

// ---- Picking: grid spaziale (broad-phase in mondo) ------------------------
const GN = 160;
const _grid = new Map();
let _gridFieldId = null;
const _gk = (gx, gy) => gx + ',' + gy;

// ---- Stato del layer vivo (cosa illuminare) -------------------------------
const _liveState = { path: [], rosa: null, sub: null, subRosa: null, edges: null };

// Vista relazioni (pack): stato dedicato, screen-space.
let _rectScrollY = 0;
let _rectBoxes = [];     // [{id, x, y, w, h, isSentence}] — y PRE-scroll
let _rectContentH = 0;

// Stato vista nuovo
let _lastModeKey = '';         // refit/refresh al cambio campo/vista
let _nuovoHideLabels = false;  // dimensioni: label solo su hover/click (come vasto)
let _dragCurX = 0, _dragCurY = 0;  // cursore durante un drag (per il tether di connessione)
let _fieldHasClones = false;   // nuovo con cloni vasto (fase comprensione) — evita il flash dimensionale

// ---- Sottoscrizione ai cambi dati del campo (filtri → rebuild bitmap) -----
let _nodesSub = null;       // { ds, cb }
let _rebuildQueued = false;

// =====================================================================
//  API pubblica
// =====================================================================

export function initNetwork(container){
  _container = container;
  try { _nuovoHideLabels = localStorage.getItem('uir1_nuovo_hide_labels') === '1'; } catch(_){}
  // I canvas vanno SOTTO gli overlay HTML di #graph (stats, breadcrumb,
  // dim-overlay, toolbar, trail). Li inseriamo come PRIMI figli così
  // vengono dipinti prima dei fratelli successivi.
  _back = document.createElement('canvas');
  _live = document.createElement('canvas');
  for(const c of [_back, _live]){ c.style.position = 'absolute'; c.style.inset = '0'; c.style.zIndex = '0'; }
  _live.style.cursor = 'crosshair';   // puntamento preciso (feedback)
  container.prepend(_back);
  _back.after(_live);
  _bx = _back.getContext('2d');
  _lx = _live.getContext('2d');

  _resizeCanvases();
  _wireEvents();

  network = _makeShim();
  initOverlay(network);   // archi animati + label dinamiche su afterDrawing

  // Il #graph cambia larghezza (collassa/ridimensiona sidebar, resize finestra):
  // adatta i px del canvas E ri-fitta la vista, così il grafo si prende lo
  // spazio disponibile quando la sidebar si chiude e si ricomprime quando si
  // riapre (segue anche la transizione CSS, tick per tick).
  const onResize = () => { _resizeCanvases(); _fitView(); redraw(); };
  try {
    const ro = new ResizeObserver(onResize);
    ro.observe(container);
  } catch(_){ window.addEventListener('resize', onResize); }

  return network;
}

// Monta un campo: ricalcola bounds, grid, bitmap, fit, redraw.
export function mountField(id, { fit = true, animate = true } = {}){
  const F = FIELDS[id];
  if(!F) return;
  setActive(id);

  // Reset stato vivo al cambio campo.
  _liveState.path = []; _liveState.rosa = null; _liveState.sub = null; _liveState.subRosa = null; _liveState.edges = null;
  _rectScrollY = 0;
  clearOverlay();

  // Sottoscrizione ai cambi del DataSet del campo attivo: filtri.js
  // (e ogni mutazione dati) → rigenera il fondale su EVENTO.
  if(_nodesSub){ try { _nodesSub.ds.off('*', _nodesSub.cb); } catch(_){} _nodesSub = null; }
  const cb = () => _scheduleBitmapRebuild();
  try { F.nodesDS.on('*', cb); _nodesSub = { ds: F.nodesDS, cb }; } catch(_){}

  _fieldHasClones = F.words.some(w => w.flags?.fromExpansion);
  _computeBounds(F);
  _buildGrid(F);
  _buildBitmap(F);
  if(fit) _fitView();
  redraw();
}

export function setHandlers(h){ _handlers = { ..._handlers, ...h }; }

// =====================================================================
//  Highlight / clear / restore  (feed del layer vivo + overlay)
//  NB: NON tocca F.nodesDS (è il guadagno: niente update dei 21k nodi).
// =====================================================================

export function applyHighlight(F, activeWord, rosa){
  if(F.id === 'vasto' && isFilterActive() && !(getMatchedWords()?.has(activeWord))) return;
  _liveState.path = [...F.navPath, activeWord];
  _liveState.rosa = rosa || new Set();
  _liveState.sub = null; _liveState.subRosa = null;
  F.currentRosa = rosa;
  F.isDimmed = true;
  // Archi statici (catena+frontiera) SEMPRE in `_liveState.edges`: la vista
  // dimensioni li disegna con `_drawEdge` in coordinate-SCHERMO, col tratteggio
  // per famiglia (GROUP_DASH) coerente con la legenda. L'overlay animato (in
  // coordinate-MONDO, dash scalati da view.k → apparivano continui) resta solo
  // come flusso colorato sopra. Fix: prima nella vista dimensioni `_liveState
  // .edges` era null → nessun arco tratteggiato, solo il flusso "pieno".
  _liveState.edges = _visibleEdges(F, _liveState.path, _liveState.rosa, 'both');
  if(_rectActive()) clearOverlay();
  else _feedOverlay(F, _liveState.path, _liveState.rosa);
  redraw();
}

export function clearHighlight(F){
  _cancelClear();
  _liveState.path = []; _liveState.rosa = null; _liveState.sub = null; _liveState.subRosa = null; _liveState.edges = null;
  F.currentRosa = null;
  F.isDimmed = false;
  F.subHover = null;
  clearOverlay();
  redraw();
}

export function restoreSelection(F){
  if(!F.selected || !F.currentRosa) return;
  _liveState.path = [...F.navPath, F.selected];
  _liveState.rosa = F.currentRosa;
  _liveState.sub = null; _liveState.subRosa = null;
  F.subHover = null;
  // Vedi nota in applyHighlight: archi statici tratteggiati (legenda) + overlay.
  _liveState.edges = _visibleEdges(F, _liveState.path, _liveState.rosa, 'both');
  if(_rectActive()) clearOverlay();
  else _feedOverlay(F, _liveState.path, _liveState.rosa);
  redraw();
}

// Sub-hover: anteprima delle connessioni di un vicino della selezione.
function _subHoverPreview(F, node){
  const direction = F.id === 'vasto' ? getFilterDirection() : 'both';
  const subRosa = F.getRosa(node, direction, { filterByType: isRelTypeEnabled });
  _liveState.sub = node;
  _liveState.subRosa = subRosa;
  F.subHover = node;
  // archi del nodo sotto il cursore.
  const edges = _visibleEdges(F, [node], subRosa, direction);
  _liveState.edges = edges;
  if(_rectActive()){
    clearOverlay();   // board: tutto in screen-space, niente overlay world
  } else {
    setActiveTraceEdges(_overlayEdgesFrom(F, edges));
    setActiveLabels([]);   // label le disegna _drawNuovoField/_drawFocusLabels
  }
  redraw();
}

// =====================================================================
//  Rendering
// =====================================================================

function redraw(){
  // Al cambio campo/vista: assicura px canvas correnti e ri-fit. Risolve il
  // "relazioni appare piccola/sfocata" passando da dimensioni a relazioni
  // (prima si sistemava solo cliccando 'riorganizza').
  const F0 = getActive();
  const modeKey = F0 ? `${F0.id}:${_rectActive() ? 'r' : 'd'}` : '';
  if(modeKey !== _lastModeKey){
    _lastModeKey = modeKey;
    _resizeCanvases();
    _rectScrollY = 0;
    if(!_rectActive()) _fitView();
  }
  _drawBackdrop();
  _drawLive();
  // overlay (archi animati + label) sul ctx live, in COORDINATE-MONDO —
  // identico a come vis-network passa il ctx trasformato in afterDrawing.
  _lx.save();
  _lx.setTransform(view.k, 0, 0, view.k, view.tx, view.ty);
  (_events['afterDrawing'] || []).forEach(fn => { try { fn(_lx); } catch(_){} });
  _lx.restore();
  if(_dragNode && _moved) _drawDragTether();
}

// Tether di connessione: linea tratteggiata + freccia dal nodo sorgente al
// cursore mentre si trascina, e anello sul bersaglio sotto il cursore. Dà il
// feedback "stai collegando" (richiesta di Francesco) in entrambe le viste.
function _drawDragTether(){
  const F = getActive(); if(!F) return;
  let ax, ay;
  if(_rectActive()){
    const box = _boxById.get(_dragNode); if(!box) return;
    ax = box.x + box.w / 2; ay = box.y + box.h / 2 - _rectScrollY;
  } else {
    const w = F.wordMap[_dragNode];
    const o = _dragNodeOrig || (w && w.position); if(!o) return;
    [ax, ay] = worldToScreen(o.x, o.y);
  }
  const cx = _dragCurX, cy = _dragCurY;
  const tgt = _rectActive() ? _pickRect(cx, cy)
            : (F.id === 'nuovo' ? _pickNuovo(F, cx, cy) : _pickFromGrid(F, cx, cy, _dragNode));
  const overTarget = !!(tgt && tgt !== _dragNode);
  const col = overTarget ? (UI.sentenceGlow || '#7ad') : (UI.edgeHover || '#cfd2e0');
  _lx.save();
  _lx.setTransform(1, 0, 0, 1, 0, 0);
  _lx.globalAlpha = 0.9; _lx.strokeStyle = col; _lx.lineWidth = 2;
  _lx.setLineDash([6, 5]);
  _lx.beginPath(); _lx.moveTo(ax, ay); _lx.lineTo(cx, cy); _lx.stroke();
  _lx.setLineDash([]);
  const ang = Math.atan2(cy - ay, cx - ax), ah = 9;
  _lx.beginPath(); _lx.moveTo(cx, cy);
  _lx.lineTo(cx - Math.cos(ang - 0.4) * ah, cy - Math.sin(ang - 0.4) * ah);
  _lx.lineTo(cx - Math.cos(ang + 0.4) * ah, cy - Math.sin(ang + 0.4) * ah);
  _lx.closePath(); _lx.fillStyle = col; _lx.fill();
  if(overTarget){
    _lx.strokeStyle = UI.sentenceGlow || '#7ad'; _lx.lineWidth = 2.5;
    const tb = _rectActive() ? _boxById.get(tgt) : null;
    if(tb){ _roundRect(tb.x - 2, tb.y - _rectScrollY - 2, tb.w + 4, tb.h + 4, 7); _lx.stroke(); }
    else {
      const tw = F.wordMap[tgt];
      if(tw?.position){
        const [tx, ty] = worldToScreen(tw.position.x, tw.position.y);
        _lx.beginPath(); _lx.arc(tx, ty, Math.max(10, sizeFor(tw, F.id) * view.k + 5), 0, 6.28318); _lx.stroke();
      }
    }
  }
  _lx.restore();
}

function _drawBackdrop(){
  _bx.fillStyle = UI.bg || '#0f0f1a';
  _bx.fillRect(0, 0, W, H);
  // Il bitmap (21k dot) serve a vasto e durante la comprensione frase (cloni).
  // Il campo nuovo a riposo si disegna tutto sul layer vivo (dot/box) → niente
  // dot-fantasma dietro i box della vista relazioni.
  const F = getActive();
  const useBitmap = F && (F.id === 'vasto' || document.body.classList.contains('comprensione-frase'));
  if(!useBitmap || !_bmp) return;
  const [sx, sy] = worldToScreen(minX, minY);
  const [ex, ey] = worldToScreen(maxX, maxY);
  _bx.imageSmoothingEnabled = true;
  _bx.drawImage(_bmp, sx, sy, ex - sx, ey - sy);   // un solo drawImage per tutto il fondale
}

function _drawLive(){
  _lx.setTransform(1, 0, 0, 1, 0, 0);
  _lx.clearRect(0, 0, W, H);
  const F = getActive();
  if(!F) return;

  // _fieldHasClones: il nuovo è "gonfio" di cloni vasto = siamo nel flusso
  // comprensione anche se la classe CSS non è ancora stata messa → evita il
  // flash del campo dimensionale prima che parta l'animazione.
  const inComprehension = document.body.classList.contains('comprensione-frase') || _fieldHasClones;
  const isRect = F.id === 'nuovo' && getNuovoLayout() === 'rectangular';

  // Velo O(1): attenua il fondale su selezione/hover, OPPURE durante la
  // comprensione frase. NON in vista relazioni (lì l'attenuazione è per-box).
  if((F.isDimmed && !isRect) || inComprehension){
    _lx.fillStyle = 'rgba(15,15,26,0.62)';
    _lx.fillRect(0, 0, W, H);
  }

  // Durante la comprensione frase i visual li possiede il canvas dell'animazione
  // (label animate via canvasToDOM): niente baseline qui (no label doppie).
  if(inComprehension) return;

  // Campo nuovo, VISTA RELAZIONI: pack a righe + archi.
  if(isRect){ _drawRectBoard(F); return; }

  // Campo nuovo, VISTA DIMENSIONI: una sola funzione per riposo + hover
  // (sfondo attenuato con opacity, non sparito; label solo dove servono).
  if(F.id === 'nuovo'){ _drawNuovoField(F); return; }

  // --- da qui solo VASTO ---
  if(!F.isDimmed) return;

  // Archi stilizzati del set visibile (catena+frontiera o sub-hover), SOTTO i
  // nodi. L'overlay (afterDrawing) aggiunge il flusso animato colorato sopra.
  if(_liveState.edges){ for(const e of _liveState.edges) _drawEdge(F, e); }

  const pathSet = new Set(_liveState.path);
  const sub = _liveState.sub;
  const subRosa = _liveState.subRosa;

  // Rosa (e sub-rosa) — dot pieni nel colore della parola.
  if(_liveState.rosa){
    _liveState.rosa.forEach(id => { if(!pathSet.has(id)) _drawNodeDot(F, id, 'rosa'); });
  }
  if(subRosa){
    subRosa.forEach(id => { if(!pathSet.has(id) && id !== sub) _drawNodeDot(F, id, 'rosa'); });
  }
  // Catena (path) — dot + anello bianco.
  for(const id of _liveState.path) _drawNodeDot(F, id, 'active');
  if(sub) _drawNodeDot(F, sub, 'active');

  // Label a dimensione FISSA leggibile (path/rosa/sub): i rosa nel proprio
  // colore, la selezione in bianco-bold. (Non più via overlay, che con field_all
  // le rendeva microscopiche.)
  _drawFocusLabels(F);
}

// Disegna le label del set in evidenza a dimensione fissa (screen-space).
function _drawFocusLabels(F){
  const drawn = new Set();
  const one = (id, big) => {
    if(drawn.has(id)) return; drawn.add(id);
    const w = F.wordMap[id]; if(!w || !w.position || w.position.x == null) return;
    const [sx, sy] = worldToScreen(w.position.x, w.position.y);
    if(sx < -40 || sx > W + 40 || sy < -20 || sy > H + 20) return;
    const label = w.displayName || w.w;
    const fs = big ? 16 : 14;
    _lx.save();
    _lx.font = `${big ? '700' : '600'} ${fs}px 'JetBrains Mono', monospace`;
    _lx.textBaseline = 'middle'; _lx.textAlign = 'left';
    const tx = sx + Math.max(3, sizeFor(w, F.id) * view.k) + 5;
    _lx.lineWidth = 3.5; _lx.strokeStyle = 'rgba(0,0,0,0.9)'; _lx.strokeText(label, tx, sy);
    _lx.fillStyle = big ? (UI.textBright || '#fff') : colorForSig(w.sig);
    _lx.fillText(label, tx, sy);
    _lx.restore();
  };
  for(const id of _liveState.path) one(id, true);
  if(_liveState.sub) one(_liveState.sub, true);
  if(_liveState.rosa) _liveState.rosa.forEach(id => one(id, false));
  if(_liveState.subRosa) _liveState.subRosa.forEach(id => one(id, false));
}

// Campo nuovo, vista dimensioni — riposo E hover in un'unica passata.
// A riposo: tutte le parole piene (label secondo _nuovoHideLabels).
// In hover/selezione: la catena (path) + l'adiacenza del nodo corrente in
// evidenza; TUTTO il resto resta VISIBILE ma ATTENUATO (opacity), non sparisce.
// Le label dello sfondo si spengono in hover (solo il focus resta etichettato)
// → niente label "riscritte" inutilmente.
function _drawNuovoField(F){
  const dimmed = F.isDimmed;
  const path = _liveState.path || [];
  const pathSet = new Set(path);
  const focusSet = new Set(path);
  if(_liveState.rosa)    _liveState.rosa.forEach(id => focusSet.add(id));
  if(_liveState.subRosa) _liveState.subRosa.forEach(id => focusSet.add(id));
  if(_liveState.sub)     focusSet.add(_liveState.sub);
  const hasFocus = dimmed && focusSet.size > 0;

  // Archi catena+frontiera (solo in focus).
  if(hasFocus && _liveState.edges){ for(const e of _liveState.edges) _drawEdge(F, e); }

  for(const w of F.words){
    if(w.flags?.fromExpansion || !w.position || w.position.x == null) continue;
    const inFocus = focusSet.has(w.w);
    const isChain = pathSet.has(w.w) || w.w === _liveState.sub;
    const faded = hasFocus && !inFocus;
    _lx.save();
    _lx.globalAlpha = faded ? 0.16 : 1;   // sfondo attenuato, NON sparito
    _drawNodeDot(F, w.w, isChain ? 'active' : 'rosa');
    _lx.restore();
    // Label: in hover solo il focus; a riposo secondo _nuovoHideLabels.
    const showLabel = hasFocus ? inFocus : !_nuovoHideLabels;
    if(!showLabel) continue;
    const [sx, sy] = worldToScreen(w.position.x, w.position.y);
    const label = w.displayName || w.w;
    const big = isChain;
    _lx.save();
    _lx.font = `${big ? '700' : '600'} ${big ? 15 : 13}px 'JetBrains Mono', monospace`;
    _lx.textBaseline = 'middle';
    const tx = sx + Math.max(3, sizeFor(w, F.id) * view.k) + 5;
    _lx.lineWidth = 3; _lx.strokeStyle = 'rgba(0,0,0,.85)';
    _lx.strokeText(label, tx, sy);
    _lx.fillStyle = big ? (UI.textBright || '#fff') : colorForSig(w.sig);
    _lx.fillText(label, tx, sy);
    _lx.restore();
  }
}

function _drawNodeDot(F, id, kind){
  const w = F.wordMap[id];
  if(!w || !w.position || w.position.x == null) return;
  const [sx, sy] = worldToScreen(w.position.x, w.position.y);
  const r = Math.max(kind === 'active' ? 4.5 : 3, sizeFor(w, F.id) * view.k);
  const col = colorForSig(w.sig);
  _lx.beginPath(); _lx.arc(sx, sy, r, 0, 6.28318); _lx.fillStyle = col; _lx.fill();
  if(kind === 'active'){
    _lx.lineWidth = 2; _lx.strokeStyle = UI.textBright || '#fff'; _lx.stroke();
  }
}

// Arco stilizzato come vis nell'originale: tratteggio per famiglia (REL_GROUP →
// GROUP_DASH), spessore per confidenza, freccia di direzione (from → to). Il
// colore è quello degli archi dell'originale (UI.edgeHover); il flusso animato
// colorato lo aggiunge l'overlay sopra.
function _drawEdge(F, e){
  const fw = F.wordMap[e.from], tw = F.wordMap[e.to];
  if(!fw?.position || !tw?.position || fw.position.x == null || tw.position.x == null) return;
  const [fx, fy] = worldToScreen(fw.position.x, fw.position.y);
  const [tx, ty] = worldToScreen(tw.position.x, tw.position.y);
  const dx = tx - fx, dy = ty - fy;
  const len = Math.hypot(dx, dy); if(len < 1) return;
  const ang = Math.atan2(dy, dx);
  const dash = GROUP_DASH[REL_GROUP[e.rel] || 'L'];
  const width = Math.max(1, ((e.conf || 50) / 100) * 1.6);
  const col = UI.edgeHover || '#cfd2e0';
  // accorcia all'orlo del nodo target così la freccia non entra nel dot
  const tr = Math.max(3, sizeFor(tw, F.id) * view.k) + 1.5;
  const ex = tx - Math.cos(ang) * tr, ey = ty - Math.sin(ang) * tr;
  _lx.save();
  _lx.globalAlpha = 0.78;
  _lx.strokeStyle = col;
  _lx.lineWidth = width;
  _lx.setLineDash(Array.isArray(dash) ? dash : []);
  _lx.beginPath(); _lx.moveTo(fx, fy); _lx.lineTo(ex, ey); _lx.stroke();
  _lx.setLineDash([]);
  const ah = 6 + width;   // testa di freccia
  _lx.beginPath();
  _lx.moveTo(ex, ey);
  _lx.lineTo(ex - Math.cos(ang - 0.42) * ah, ey - Math.sin(ang - 0.42) * ah);
  _lx.lineTo(ex - Math.cos(ang + 0.42) * ah, ey - Math.sin(ang + 0.42) * ah);
  _lx.closePath();
  _lx.fillStyle = col;
  _lx.fill();
  _lx.restore();
}

// ---- Vista relazioni (layout rettangolare del nuovo): box etichettati ------
// Le posizioni le calcola layouts/rectangular.js (parole-frase in alto, vicini
// in colonne per famiglia). Qui le rendiamo come box come l'originale: sfondo
// scuro, bordo nel colore della firma (glow per le parole-frase), label dentro.
function _roundRect(x, y, w, h, r){
  _lx.beginPath();
  _lx.moveTo(x + r, y);
  _lx.arcTo(x + w, y, x + w, y + h, r);
  _lx.arcTo(x + w, y + h, x, y + h, r);
  _lx.arcTo(x, y + h, x, y, r);
  _lx.arcTo(x, y, x + w, y, r);
  _lx.closePath();
}

// ---- Vista relazioni: PACK COMPATTO a righe + archi ------------------------
// Per sessioni collettive: tutte le parole come box a dimensione FISSA (sempre
// leggibili), impacchettati in righe che riempiono la larghezza (niente colonne
// sbilanciate). Le parole-frase in cima, evidenziate; le altre in ordine
// alfabetico. Gli ARCHI mostrano le relazioni: sottili a riposo, accesi su
// hover/selezione. Chiudere la sidebar allarga (più box per riga). Scroll
// verticale con la rotella solo se serve.
const RB = {
  pad: 24, padTop: 66, gapX: 8, gapY: 8, sectionGap: 18,
  fontHeader: 15, fontChip: 13, hHeader: 32, hChip: 26,
  padHeader: 13, padChip: 10,
};

let _boxById = new Map();   // id → box, per disegnare gli archi fra i box

function _rectActive(){
  const F = getActive();
  return !!(F && F.id === 'nuovo' && getNuovoLayout() === 'rectangular');
}

// Pack a righe (flow). Parole-frase prima (evidenziate), poi le altre
// alfabetiche. Box larghi quanto il testo. Riempiono la larghezza, vanno a capo.
function _computeBoard(F){
  _rectBoxes = []; _boxById = new Map();
  const innerW = Math.max(120, W - 2 * RB.pad);
  const words = F.words.filter(w => !w.flags?.fromExpansion);
  const sentence = words.filter(w => w.flags?.fromSentence)
    .slice().sort((a, b) => (a.sentenceIndex ?? 999) - (b.sentenceIndex ?? 999));
  const others = words.filter(w => !w.flags?.fromSentence)
    .slice().sort((a, b) => (a.displayName || a.w).localeCompare(b.displayName || b.w, 'it'));

  let x = RB.pad, y = RB.padTop, rowMaxH = 0;   // padTop: spazio per le icone in alto
  const place = (w, isSentence) => {
    const fs = isSentence ? RB.fontHeader : RB.fontChip;
    _lx.font = `${isSentence ? '700' : '600'} ${fs}px 'JetBrains Mono', monospace`;
    const label = w.displayName || w.w;
    const cw = _lx.measureText(label).width + (isSentence ? RB.padHeader : RB.padChip) * 2;
    const h = isSentence ? RB.hHeader : RB.hChip;
    if(x + cw > RB.pad + innerW){ x = RB.pad; y += rowMaxH + RB.gapY; rowMaxH = 0; }
    const box = { id: w.w, x, y, w: cw, h, isSentence };
    _rectBoxes.push(box); _boxById.set(w.w, box);
    x += cw + RB.gapX; rowMaxH = Math.max(rowMaxH, h);
  };
  for(const w of sentence) place(w, true);
  if(sentence.length){ x = RB.pad; y += rowMaxH + RB.sectionGap; rowMaxH = 0; }   // a capo dopo la frase
  for(const w of others) place(w, false);

  _rectContentH = y + rowMaxH + RB.pad;
  const maxScroll = Math.max(0, _rectContentH - H);
  _rectScrollY = Math.max(0, Math.min(_rectScrollY, maxScroll));
}

// Arco fra due box (screen, y PRE-scroll applicato). Sottile a riposo, acceso
// se tocca la parola sotto il cursore / selezionata.
function _drawPackEdge(a, b, active, hasFocus){
  const ax = a.x + a.w / 2, ay = a.y + a.h / 2 - _rectScrollY;
  const bx = b.x + b.w / 2, by = b.y + b.h / 2 - _rectScrollY;
  if((ay < 0 && by < 0) || (ay > H && by > H)) return;   // cull
  _lx.save();
  if(active){ _lx.strokeStyle = UI.edgeHover || '#cfd2e0'; _lx.globalAlpha = 0.85; _lx.lineWidth = 1.6; }
  else if(hasFocus){ _lx.strokeStyle = UI.edgeDefault || '#555'; _lx.globalAlpha = 0.06; _lx.lineWidth = 1; }
  else { _lx.strokeStyle = UI.edgeDefault || '#6a6a80'; _lx.globalAlpha = 0.20; _lx.lineWidth = 1; }
  _lx.beginPath(); _lx.moveTo(ax, ay); _lx.lineTo(bx, by); _lx.stroke();
  _lx.restore();
}

function _drawBoardChip(F, b, { active, faded, isFocus }){
  const w = F.wordMap[b.id]; if(!w) return;
  const y = b.y - _rectScrollY;
  if(y + b.h < -4 || y > H + 4) return;   // cull fuori viewport
  const isSentence = b.isSentence;
  const label = w.displayName || w.w;
  const col = colorForSig(w.sig);
  // Bordo SEMPRE nel colore della firma (i rosa NON perdono il colore); glow
  // per le parole-frase. Il focus si distingue solo per spessore e sfondo.
  const border = isSentence ? (UI.sentenceGlow || col) : col;
  _lx.save();
  _lx.globalAlpha = faded ? 0.28 : 1;
  _roundRect(b.x, y, b.w, b.h, 6);
  _lx.fillStyle = isFocus ? 'rgba(60,60,80,0.97)' : (isSentence ? 'rgba(34,34,52,0.97)' : 'rgba(22,22,34,0.97)');
  _lx.fill();
  _lx.lineWidth = isFocus ? 3 : (isSentence ? 2.4 : 1.5);
  _lx.strokeStyle = border;
  _lx.stroke();
  _lx.font = `${isSentence ? '700' : '600'} ${isSentence ? RB.fontHeader : RB.fontChip}px 'JetBrains Mono', monospace`;
  _lx.fillStyle = UI.textBright || '#fff';
  _lx.textAlign = 'center'; _lx.textBaseline = 'middle';
  _lx.fillText(label, b.x + b.w / 2, y + b.h / 2);
  _lx.textAlign = 'start';
  _lx.restore();
}

function _drawRectBoard(F){
  _computeBoard(F);

  // Stato di navigazione (come campovasto): la CATENA cliccata (path) resta
  // evidenziata; si aggiunge l'adiacenza del nodo corrente (rosa) + sub-hover.
  // Gli archi mostrati = catena+frontiera (_liveState.edges), non gli archi di
  // tutti i vicini.
  const path = _liveState.path || [];
  const pathSet = new Set(path);
  const focusSet = new Set(path);
  if(_liveState.rosa)    _liveState.rosa.forEach(id => focusSet.add(id));
  if(_liveState.subRosa) _liveState.subRosa.forEach(id => focusSet.add(id));
  if(_liveState.sub)     focusSet.add(_liveState.sub);
  const hasFocus = F.isDimmed && focusSet.size > 0;

  if(!hasFocus){
    // A riposo: tutti gli archi sottili (struttura), niente in evidenza.
    for(const e of F.edges){
      if(!isRelTypeEnabled(e.rel)) continue;
      const a = _boxById.get(e.from), b = _boxById.get(e.to);
      if(a && b) _drawPackEdge(a, b, false, false);
    }
  } else if(_liveState.edges){
    // In interazione: SOLO catena+frontiera, accesi.
    for(const e of _liveState.edges){
      const a = _boxById.get(e.from), b = _boxById.get(e.to);
      if(a && b) _drawPackEdge(a, b, true, true);
    }
  }

  // Box: catena (path) + adiacenza in colore pieno; il resto sbiadito.
  for(const box of _rectBoxes){
    const isFocus = pathSet.has(box.id) || box.id === _liveState.sub;
    const active = !hasFocus ? false : focusSet.has(box.id);
    const faded = hasFocus && !active;
    _drawBoardChip(F, box, { active, faded, isFocus });
  }

  // Indicatore di scroll se il contenuto eccede il viewport.
  if(_rectContentH > H){
    const trackH = H - 2 * RB.pad;
    const thumbH = Math.max(24, trackH * (H / _rectContentH));
    const thumbY = RB.pad + (trackH - thumbH) * (_rectScrollY / (_rectContentH - H));
    _lx.save();
    _lx.globalAlpha = 0.5;
    _lx.fillStyle = UI.textDim || '#8a8a99';
    _roundRect(W - 8, thumbY, 4, thumbH, 2);
    _lx.fill();
    _lx.restore();
  }
}

// Hit-test del tabellone: tutta la riga del box (anche se il chip è più corto).
function _pickRect(localX, localY){
  for(const b of _rectBoxes){
    const y = b.y - _rectScrollY;
    if(localX >= b.x && localX <= b.x + b.w && localY >= y && localY <= y + b.h) return b.id;
  }
  return null;
}

// =====================================================================
//  Costruzione bounds / grid / bitmap
// =====================================================================

function _computeBounds(F){
  let aX = Infinity, bX = -Infinity, aY = Infinity, bY = -Infinity;
  for(const w of F.words){
    const p = w.position; if(!p || p.x == null) continue;
    if(p.x < aX) aX = p.x; if(p.x > bX) bX = p.x;
    if(p.y < aY) aY = p.y; if(p.y > bY) bY = p.y;
  }
  if(!isFinite(aX)){ minX = 0; maxX = 1; minY = 0; maxY = 1; return; }
  const cx = (aX + bX) / 2, cy = (aY + bY) / 2;
  const half = (Math.max(bX - aX, bY - aY) / 2) * 1.08 || 1;
  minX = cx - half; maxX = cx + half; minY = cy - half; maxY = cy + half;   // quadrato
}

function _buildGrid(F){
  _grid.clear();
  _gridFieldId = F.id;
  const spanX = maxX - minX, spanY = maxY - minY;
  for(const w of F.words){
    const p = w.position; if(!p || p.x == null) continue;
    const gx = Math.min(GN - 1, Math.max(0, Math.floor((p.x - minX) / spanX * GN)));
    const gy = Math.min(GN - 1, Math.max(0, Math.floor((p.y - minY) / spanY * GN)));
    const k = _gk(gx, gy);
    let arr = _grid.get(k); if(!arr){ arr = []; _grid.set(k, arr); }
    arr.push(w.w);
  }
}

function _buildBitmap(F){
  const off = document.createElement('canvas'); off.width = BW; off.height = BH;
  const ox = off.getContext('2d');
  ox.fillStyle = UI.bg || '#0f0f1a'; ox.fillRect(0, 0, BW, BH);
  const sc = BW / (maxX - minX);
  const filtering = F.id === 'vasto' && isFilterActive();
  const matched = filtering ? getMatchedWords() : null;
  for(const w of F.words){
    const p = w.position; if(!p || p.x == null) continue;
    const [bxp, byp] = worldToBmp(p.x, p.y);
    const on = !filtering || (matched && matched.has(w.w));
    ox.fillStyle = colorForSig(w.sig);
    ox.globalAlpha = on ? 0.85 : 0.10;
    const r = Math.max(1.2, sizeFor(w, F.id) * sc);
    ox.beginPath(); ox.arc(bxp, byp, r, 0, 6.28318); ox.fill();
  }
  ox.globalAlpha = 1;
  _bmp = off;
  _bmpFieldId = F.id;
}

function _scheduleBitmapRebuild(){
  if(_rebuildQueued) return;
  _rebuildQueued = true;
  requestAnimationFrame(() => {
    _rebuildQueued = false;
    const F = getActive(); if(!F) return;
    _fieldHasClones = F.words.some(w => w.flags?.fromExpansion);
    _computeBounds(F); _buildGrid(F); _buildBitmap(F);
    redraw();
  });
}

// =====================================================================
//  View fit + picking
// =====================================================================

function _fitView(){
  if(!W || !H) return;
  const k = 0.92 * Math.min(W / (maxX - minX), H / (maxY - minY));
  view.k = k;
  view.tx = W / 2 - (minX + maxX) / 2 * k;
  view.ty = H / 2 - (minY + maxY) / 2 * k;
}

// pick(localX, localY) → id parola sotto il cursore, o null.
// Con una selezione attiva nel vasto, SOLO path/rosa/selezione sono
// interattivi (i bright in primo piano): cerca esclusivamente tra quelli, con
// soglia generosa. Così i punti collegati sono cliccabili e lo sfondo denso
// non "ruba" il click/hover (che faceva risultare parola sbagliata in trail e
// breadcrumb, e gli archi lampeggianti). Senza selezione: broad-phase su grid.
function pick(localX, localY){
  const F = getActive(); if(!F) return null;
  if(_rectActive()) return _pickRect(localX, localY);
  if(F.id === 'vasto' && F.selected){
    return _pickFromSet(F, localX, localY, _interactiveSet(F));
  }
  if(F.id === 'nuovo') return _pickNuovo(F, localX, localY);   // dot+label cliccabili
  return _pickFromGrid(F, localX, localY);
}

// Pick nel nuovo (vista dimensioni): l'INTERA parola è cliccabile (dot + label),
// così è facile afferrarla per spostarla/collegarla. Con le label nascoste,
// area generosa attorno al pallino.
function _pickNuovo(F, localX, localY){
  _lx.font = "600 13px 'JetBrains Mono', monospace";
  let best = null, bestD = Infinity;
  for(const w of F.words){
    if(w.flags?.fromExpansion || !w.position || w.position.x == null) continue;
    const [sx, sy] = worldToScreen(w.position.x, w.position.y);
    const r = Math.max(3, sizeFor(w, F.id) * view.k);
    if(_nuovoHideLabels){
      const hit = Math.max(12, r + 6);
      const d = Math.hypot(sx - localX, sy - localY);
      if(d <= hit && d < bestD){ bestD = d; best = w.w; }
      continue;
    }
    const lw = _lx.measureText(w.displayName || w.w).width;
    if(localX >= sx - r - 3 && localX <= sx + 9 + lw + 3 && localY >= sy - 11 && localY <= sy + 11){
      const d = Math.abs(localX - sx) + Math.abs(localY - sy);
      if(d < bestD){ bestD = d; best = w.w; }
    }
  }
  return best;
}

function _interactiveSet(F){
  const s = new Set(_liveState.path);
  if(_liveState.rosa) _liveState.rosa.forEach(id => s.add(id));
  if(F.selected) s.add(F.selected);
  return s;
}

// Con una selezione attiva, è cliccabile SOLO il mondo adiacente: rosa +
// selezione (+ nel nuovo anche le parole del breadcrumb, per back/troncamento).
// Senza selezione: tutto cliccabile. Vale per vasto E nuovo (prima il vincolo
// era solo nel vasto; l'area-parola estesa nel nuovo lo aveva smascherato).
function _clickAllowed(F, id){
  if(!id || !F.selected || !F.currentRosa) return true;
  if(id === F.selected || F.currentRosa.has(id)) return true;
  if(F.id !== 'vasto' && F.navPath && F.navPath.includes(id)) return true;
  return false;
}

function _pickFromSet(F, localX, localY, ids){
  let best = null, bestD = Infinity;
  for(const id of ids){
    const w = F.wordMap[id]; if(!w || !w.position || w.position.x == null) continue;
    const [sx, sy] = worldToScreen(w.position.x, w.position.y);
    const d = Math.hypot(sx - localX, sy - localY);
    const thr = Math.max(12, sizeFor(w, F.id) * view.k + 8);   // target generoso
    if(d < thr && d < bestD){ bestD = d; best = id; }
  }
  return best;
}

function _pickFromGrid(F, localX, localY, exclude){
  const [wx, wy] = screenToWorld(localX, localY);
  const spanX = maxX - minX, spanY = maxY - minY;
  const gx = Math.floor((wx - minX) / spanX * GN);
  const gy = Math.floor((wy - minY) / spanY * GN);
  let best = null, bestD = Infinity;
  for(let dx = -1; dx <= 1; dx++) for(let dy = -1; dy <= 1; dy++){
    const arr = _grid.get(_gk(gx + dx, gy + dy)); if(!arr) continue;
    for(const id of arr){
      if(exclude && id === exclude) continue;
      const w = F.wordMap[id]; if(!w || !w.position) continue;
      const [sx, sy] = worldToScreen(w.position.x, w.position.y);
      const d = Math.hypot(sx - localX, sy - localY);
      const thr = Math.max(6, sizeFor(w, F.id) * view.k + 4);
      if(d < thr && d < bestD){ bestD = d; best = id; }
    }
  }
  return best;
}

// =====================================================================
//  Eventi DOM sul layer vivo (replica la logica di graph.js::_wireEvents)
// =====================================================================

let _pressing = false, _moved = false, _lastX = 0, _lastY = 0;
let _dragNode = null, _dragNodeOrig = null;   // drag di un nodo nel campo nuovo
let _hovered = null;
let _pendingClear = null, _pendingClearField = null;

function _localXY(e){
  const r = _container.getBoundingClientRect();
  return [e.clientX - r.left, e.clientY - r.top];
}
function _cancelClear(){ if(_pendingClear){ clearTimeout(_pendingClear); _pendingClear = null; _pendingClearField = null; } }
function _scheduleClear(F){
  _pendingClearField = F;
  if(_pendingClear) clearTimeout(_pendingClear);
  _pendingClear = setTimeout(() => {
    _pendingClear = null; const FF = _pendingClearField; _pendingClearField = null;
    if(FF) clearHighlight(FF);
  }, 60);
}

function _wireEvents(){
  const el = _live;

  el.addEventListener('mousedown', e => {
    if(e.button !== 0) return;
    _pressing = true; _moved = false; _lastX = e.clientX; _lastY = e.clientY;
    _dragNode = null; _dragNodeOrig = null;
    const F = getActive(); if(!F) return;
    // Solo nel campo editabile (nuovo) il drag di un nodo lo MUOVE o lo CONNETTE.
    // In vasto i nodi sono fissi (read-only) e il drag non fa nulla.
    if(F.id !== 'vasto'){
      const [lx, ly] = _localXY(e);
      const id = pick(lx, ly);
      if(id && !String(id).startsWith('_')){
        const w = F.wordMap[id];
        if(w?.position){ _dragNode = id; _dragNodeOrig = { x: w.position.x, y: w.position.y }; }
      }
    }
  });

  // NB: NIENTE pan-by-drag. Trascinare lo sfondo NON sposta il grafo (richiesta
  // di Francesco); la vista si muove SOLO con la rotella. Nel nuovo, trascinare
  // un NODO lo sposta (move) o lo connette (rilascio su un altro nodo).
  window.addEventListener('mousemove', e => {
    if(!_pressing) return;
    if(Math.abs(e.clientX - _lastX) > 3 || Math.abs(e.clientY - _lastY) > 3) _moved = true;
    if(_dragNode){
      const [lx, ly] = _localXY(e);
      _dragCurX = lx; _dragCurY = ly;
      // Vista dimensioni: il nodo segue il cursore (sposta). Vista relazioni:
      // posizioni automatiche → niente move, solo il tether di connessione.
      if(!_rectActive()){
        const F = getActive(); const w = F?.wordMap[_dragNode];
        if(w?.position){ const [wx, wy] = screenToWorld(lx, ly); w.position.x = wx; w.position.y = wy; }
      }
      redraw();   // ridisegna anche il tether (freccia di connessione)
    }
  });

  // Hover (solo quando il cursore è sul canvas e non si sta premendo/trascinando).
  el.addEventListener('mousemove', e => {
    if(_pressing) return;
    const [lx, ly] = _localXY(e);
    const id = pick(lx, ly);
    if(id === _hovered) return;
    _hovered = id;
    _onHover(id);
  });

  window.addEventListener('mouseup', e => {
    if(!_pressing) return;
    _pressing = false;
    const F = getActive(); if(!F){ _dragNode = null; return; }
    const [lx, ly] = _localXY(e);

    // --- Drag di un nodo nel nuovo: CONNECT (rilascio su altro nodo) o MOVE ---
    if(_dragNode){
      const src = _dragNode, orig = _dragNodeOrig;
      _dragNode = null; _dragNodeOrig = null;
      if(!_moved){ if(_clickAllowed(F, src)) _handlers.onSelectWord?.(src); return; }   // press = click (vincolo rosa)
      const rect = _rectActive();
      const target = rect ? _pickRect(lx, ly) : _pickFromGrid(F, lx, ly, src);
      if(target && target !== src && !String(target).startsWith('_')){
        // CONNECT: (vista dimensioni) rimetti il source dov'era; apri il picker.
        if(!rect){
          const w = F.wordMap[src];
          if(w?.position && orig){ w.position.x = orig.x; w.position.y = orig.y; }
          _scheduleBitmapRebuild();
        }
        _handlers.onConnect?.(src, target);
      } else if(!rect){
        // MOVE (solo vista dimensioni): fissa la posizione finale.
        const w = F.wordMap[src];
        if(w?.position) F.markPositionUser(src, w.position.x, w.position.y);
        _scheduleBitmapRebuild();
        try { saveField(F.id); } catch(_){}
      }
      return;
    }

    // --- Click normale (nessun node-drag) — come graph.js::on('click') ---
    if(_moved) return;                 // trascinamento a vuoto: niente
    const id = pick(lx, ly);
    if(id){
      if(String(id).startsWith('_')) return;
      if(!_clickAllowed(F, id)) return;
      _handlers.onSelectWord?.(id);
      return;
    }
    if(F.selected) _handlers.onDeselect?.();
  });

  el.addEventListener('mouseleave', () => {
    if(_pressing) return;
    _hovered = null;
    _handlers.onNodeHover?.(null);
    const F = getActive(); if(!F) return;
    if(!F.selected) _scheduleClear(F);
    else if(F.subHover) restoreSelection(F);
  });

  el.addEventListener('wheel', e => {
    e.preventDefault();
    // Vista relazioni (tabellone): la rotella SCORRE verticalmente, non zooma.
    if(_rectActive()){
      const maxScroll = Math.max(0, _rectContentH - H);
      _rectScrollY = Math.max(0, Math.min(_rectScrollY + e.deltaY, maxScroll));
      redraw();
      return;
    }
    const [lx, ly] = _localXY(e);
    const [wx, wy] = screenToWorld(lx, ly);
    view.k *= (e.deltaY < 0 ? 1.15 : 1 / 1.15);
    view.k = Math.max(0.02, Math.min(4000, view.k));
    view.tx = lx - wx * view.k; view.ty = ly - wy * view.k;
    redraw();
  }, { passive: false });

  el.addEventListener('contextmenu', e => {
    e.preventDefault();
    const F = getActive(); if(!F) return;
    const [lx, ly] = _localXY(e);
    const id = pick(lx, ly);
    const screenX = e.clientX, screenY = e.clientY;
    if(id){
      if(String(id).startsWith('_')){ _handlers.onCtxEdge?.(null, 0, 0); return; }
      if(F.id === 'vasto' && F.selected && F.currentRosa && !F.currentRosa.has(id) && id !== F.selected) return;
      _handlers.onCtxNode?.(id, screenX, screenY);
      return;
    }
    // Nuovo, vista dimensioni: menu con toggle label (come vasto: visibili solo
    // su hover/click) + crea parola. Solo lato clone (riusa il componente menu).
    if(F.id === 'nuovo' && !_rectActive()){
      openCtxMenu({ x: screenX, y: screenY, items: [
        { kind: 'title', label: 'vista' },
        { kind: 'item', label: _nuovoHideLabels ? 'mostra parole' : 'nascondi parole', action: 'toggle-labels',
          onClick: () => {
            _nuovoHideLabels = !_nuovoHideLabels;
            try { localStorage.setItem('uir1_nuovo_hide_labels', _nuovoHideLabels ? '1' : '0'); } catch(_){}
            redraw();
          } },
        { kind: 'sep' },
        { kind: 'item', label: 'crea parola qui', action: 'create',
          onClick: () => { const [wx, wy] = screenToWorld(lx, ly); _handlers.onCtxEmpty?.(screenX, screenY, wx, wy); } },
      ]});
      return;
    }
    const [wx, wy] = screenToWorld(lx, ly);
    _handlers.onCtxEmpty?.(screenX, screenY, wx, wy);
  });
}

function _onHover(id){
  const F = getActive(); if(!F) return;
  _cancelClear();
  _handlers.onNodeHover?.(id);          // trail di esplorazione
  if(!id){
    if(!F.selected) _scheduleClear(F);
    else if(F.subHover) restoreSelection(F);
    return;
  }
  if(String(id).startsWith('_')) return;
  // Con selezione attiva: interagisci solo col mondo adiacente (rosa/breadcrumb).
  if(!_clickAllowed(F, id)) return;
  const direction = F.id === 'vasto' ? getFilterDirection() : 'both';
  if(!F.selected){
    applyHighlight(F, id, F.getRosa(id, direction, { filterByType: isRelTypeEnabled }));
    return;
  }
  // Selezione attiva: sub-hover su un vicino → anteprima delle sue connessioni.
  if(id === F.selected){ if(F.subHover) restoreSelection(F); return; }
  if(F.currentRosa && F.currentRosa.has(id)) _subHoverPreview(F, id);
}

// =====================================================================
//  Feed overlay (archi animati + label) — porting da graph.js
// =====================================================================

function _overlayLabel(F, id){
  const w = F.wordMap[id];
  if(!w) return null;
  return { nodeId: id, text: w.displayName || id, color: colorForSig(w.sig) };
}

// Archi visibili reali: catena (path consecutivi) + frontiera (ultima → rosa),
// filtrati per direzione e tipo (legenda). Restituisce gli oggetti-arco veri
// (con from/to/rel/conf) → direzione e tratteggio corretti nel rendering.
function _visibleEdges(F, path, rosa, direction){
  const keys = new Set();
  for(let i = 0; i < path.length - 1; i++){
    const a = path[i], b = path[i + 1];
    for(const id of F.edgesForWordIds(a)){
      const e = F.edgeByKey[id]; if(!e || !isRelTypeEnabled(e.rel)) continue;
      const other = e.from === a ? e.to : e.from;
      if(other === b) keys.add(id);
    }
  }
  if(path.length){
    const last = path[path.length - 1];
    for(const id of F.edgesForWordIds(last)){
      const e = F.edgeByKey[id]; if(!e || !isRelTypeEnabled(e.rel)) continue;
      if(direction === 'out' && e.from !== last) continue;
      if(direction === 'in'  && e.to   !== last) continue;
      const other = e.from === last ? e.to : e.from;
      if(rosa && rosa.has(other)) keys.add(id);
    }
  }
  return [...keys].map(k => F.edgeByKey[k]).filter(Boolean);
}

// overlay animato (flusso): colore = parola sorgente, dash per famiglia.
function _overlayEdgesFrom(F, edges){
  return edges.map(e => {
    const fw = F.wordMap[e.from];
    return { from: e.from, to: e.to, color: fw ? colorForSig(fw.sig) : UI.edgeHover, group: REL_GROUP[e.rel] || 'L' };
  });
}

// Catena + frontiera: archi stilizzati (live) + animati (overlay).
// Le label NON le facciamo più all'overlay: lì sono in coordinate-mondo e con
// field_all (fit molto stretto) diventerebbero microscopiche. Le disegna
// _drawFocusLabels a dimensione FISSA leggibile.
function _feedOverlay(F, path, rosa){
  const direction = F.id === 'vasto' ? getFilterDirection() : 'both';
  const edges = _visibleEdges(F, path, rosa, direction);
  _liveState.edges = edges;
  setActiveTraceEdges(_overlayEdgesFrom(F, edges));
  setActiveLabels([]);
}

// =====================================================================
//  Shim `network` (solo i metodi vis usati dalla chrome)
// =====================================================================

const _events = {};   // name -> [cb]  (in pratica solo 'afterDrawing')

function _resizeCanvases(){
  if(!_container) return;
  W = _container.clientWidth || window.innerWidth;
  H = _container.clientHeight || window.innerHeight;
  for(const c of [_back, _live]){ if(c){ c.width = W; c.height = H; } }
}

function _makeShim(){
  return {
    get body(){ return { container: _container }; },
    on(name, cb){ (_events[name] ||= []).push(cb); },
    off(name, cb){ const a = _events[name]; if(a){ const i = a.indexOf(cb); if(i >= 0) a.splice(i, 1); } },
    setData(){ /* lo swap dei dati avviene via mountField */ },
    fit(){ _fitView(); redraw(); },
    redraw(){ redraw(); },
    getPositions(ids){
      const F = getActive(); const out = {};
      const list = ids || (F ? F.words.map(w => w.w) : []);
      for(const id of list){ const w = F?.wordMap[id]; if(w?.position && w.position.x != null) out[id] = { x: w.position.x, y: w.position.y }; }
      return out;
    },
    getNodeAt(dom){ return (dom ? pick(dom.x, dom.y) : null) || undefined; },
    DOMtoCanvas(dom){ const [x, y] = screenToWorld(dom.x, dom.y); return { x, y }; },
    canvasToDOM(pos){ const [x, y] = worldToScreen(pos.x, pos.y); return { x, y }; },
  };
}
