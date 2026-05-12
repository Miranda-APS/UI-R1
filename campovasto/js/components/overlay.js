// Overlay sopra la canvas di vis-network: archi animati (lineDashOffset)
// e label dinamiche con risoluzione delle collisioni. Una sola chiamata a
// initOverlay(network), poi setActiveTraceEdges() e setActiveLabels()
// vengono invocate da graph.js in applyHighlight / sub-hover / restoreSelection.
//
// Render strategy: agganciamo l'evento `afterDrawing` di vis-network. Quel
// callback riceve il canvas 2D context dopo che vis ha disegnato nodi+archi.
// Ci disegniamo SOPRA gli archi animati e le label.
//
// Animazione: requestAnimationFrame chiama network.redraw() che ri-emette
// afterDrawing → ridipingiamo con offset aggiornato. Il loop si ferma da solo
// quando _activeEdges è vuoto, per non consumare CPU a riposo.

import { UI } from '../theme.js';

// ---- Parametri animazione archi ----
// Calmi: dash lunghi, scorrimento lento, propagazione dal sorgente verso il
// bersaglio. Bianco neutro — nessuna interferenza coi colori delle parole.
const DASH_LEN          = 14;
const GAP_LEN           = 6;
const DASH_PERIOD       = DASH_LEN + GAP_LEN;
const SPEED_PX_PER_SEC  = 28;     // lento → niente effetto strobo
const TRACE_LINE_WIDTH  = 2;
const TRACE_OPACITY     = 0.65;   // mai a 1 — lascia respirare
const TRACE_COLOR       = '#ffffff';
const PROPAGATION_MS    = 380;    // tempo di crescita arco from → to (ease-out)

// ---- Parametri label dinamiche ----
// Niente box. Testo grande nel colore della parola, con contorno nero
// spesso dietro per la leggibilità su qualsiasi sfondo.
const LABEL_FONT_SIZE   = 20;
const LABEL_FONT        = `600 ${LABEL_FONT_SIZE}px 'JetBrains Mono', monospace`;
const LABEL_NODE_OFFSET = 22;
const LABEL_STROKE_W    = 4;
const LABEL_STROKE      = 'rgba(0, 0, 0, 0.85)';

let _network = null;
let _activeEdges  = [];   // [{from, to, color?}]
let _activeLabels = [];   // [{nodeId, text, color}]
let _rafId        = null;
let _animStart    = 0;

function aabbCollide(a, b){
  return !(a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y);
}

// Helper per calcolare se siamo in rectangular layout.
// Importante: si applica SOLO se siamo nel campo 'nuovo', non in 'vasto'.
function isRectangularMode() {
  if (document.body.classList.contains('campo-vasto')) return false;
  if (window._getNuovoLayout) return window._getNuovoLayout() === 'rectangular';
  try { return localStorage.getItem('uir1_nuovo_layout') === 'rectangular'; } catch(e) { return false; }
}

// ---- Render archi animati ----
// La linea cresce dal nodo sorgente (e.from) verso il bersaglio (e.to) con
// ease-out cubic in PROPAGATION_MS. Sopra a questa crescita scorrono i dash
// (lineDashOffset) — l'effetto è un'onda che si propaga lungo l'arco.
// Pattern dash per tipo relazione (REL_GROUP). Coerenti con la legenda:
//   strutturale (S): dotted   →  [2, 4]
//   causale (C):     solid    →  []         (nessun dash)
//   semantica (M):   dashed   →  [3, 6]
//   fenomenologica (F): dotted fitto → [1, 3]
//   logica (L):      dashed largo → [12, 6]
const DASH_BY_GROUP = {
  S: [2, 4],
  C: [],
  M: [3, 6],
  F: [1, 3],
  L: [12, 6],
};

function drawTracingEdges(ctx){
  if(!_activeEdges.length || !_network) return;
  if(isRectangularMode()) return; // NIENTE archi animati in rectangular mode
  if(!_animStart) _animStart = performance.now();
  const elapsed = performance.now() - _animStart;
  // Offset negativo: i dash scorrono nella direzione di disegno (from → to).
  const offset  = -((elapsed / 1000) * SPEED_PX_PER_SEC) % DASH_PERIOD;
  // Propagazione: 0 → 1 con ease-out cubic (parte veloce, arriva in punta di piedi).
  const u = Math.min(1, elapsed / PROPAGATION_MS);
  const grow = 1 - Math.pow(1 - u, 3);

  const allIds = new Set();
  for(const e of _activeEdges){ allIds.add(e.from); allIds.add(e.to); }
  const positions = _network.getPositions([...allIds]);

  ctx.save();
  ctx.lineWidth      = TRACE_LINE_WIDTH;
  ctx.lineCap        = 'round';
  ctx.lineDashOffset = offset;
  ctx.globalAlpha    = TRACE_OPACITY;
  ctx.strokeStyle    = TRACE_COLOR;

  for(const e of _activeEdges){
    const fp = positions[e.from], tp = positions[e.to];
    if(!fp || !tp) continue;
    // Pattern dash specifico per il tipo di relazione (coerente con legenda).
    // Default fallback: il pattern animato originale.
    const pattern = (e.group && DASH_BY_GROUP[e.group] !== undefined)
      ? DASH_BY_GROUP[e.group]
      : [DASH_LEN, GAP_LEN];
    ctx.setLineDash(pattern);
    // Endpoint dinamico: durante la propagazione si ferma a `grow` del cammino.
    const endX = fp.x + (tp.x - fp.x) * grow;
    const endY = fp.y + (tp.y - fp.y) * grow;
    ctx.beginPath();
    ctx.moveTo(fp.x, fp.y);
    ctx.lineTo(endX, endY);
    ctx.stroke();
  }
  ctx.restore();
}

// ---- Render label dinamiche con anti-collisione ----
// Per ogni label prova 4 posizioni candidate (right, left, below, above) —
// la prima che non collide con label già piazzate vince. Niente box: il
// testo è nel colore della parola con un contorno nero spesso (strokeText
// → fillText) per restare leggibile su sfondi qualsiasi.
function drawLabels(ctx){
  if(!_activeLabels.length || !_network) return;

  ctx.save();
  ctx.font         = LABEL_FONT;
  ctx.textBaseline = 'middle';
  ctx.lineJoin     = 'round';

  const ids       = _activeLabels.map(l => l.nodeId);
  const positions = _network.getPositions(ids);
  const widths    = _activeLabels.map(l => ctx.measureText(l.text).width);

  const placed = [];   // bbox AABB già occupate
  const plan   = [];   // {lbl, chosen} per il disegno
  
  const isRect = isRectangularMode();

  for(let i = 0; i < _activeLabels.length; i++){
    const lbl = _activeLabels[i];
    const pos = positions[lbl.nodeId];
    if(!pos) continue;
    const w = widths[i];
    const h = LABEL_FONT_SIZE;

    // Nel layout rettangolare, NON disegniamo le label via overlay.
    // Sono già incastonate nei "box" nativi di vis-network, centrati e perfetti.
    // L'overlay in rect-mode disegnerà solo gli archi traccianti (se ci sono).
    if (isRect) {
      continue;
    }

    const candidates = [
      { x: pos.x + LABEL_NODE_OFFSET, y: pos.y, align: 'left' },
      { x: pos.x - LABEL_NODE_OFFSET, y: pos.y, align: 'right' },
      { x: pos.x, y: pos.y + LABEL_NODE_OFFSET + h/2, align: 'center' },
      { x: pos.x, y: pos.y - LABEL_NODE_OFFSET - h/2, align: 'center' },
    ];

    let chosen = null, chosenBox = null;
    for(const c of candidates){
      const left = c.align === 'left'  ? c.x
                 : c.align === 'right' ? c.x - w
                 : c.x - w/2;
      const box = { x: left, y: c.y - h/2, w, h };
      if(!placed.some(p => aabbCollide(p, box))){
        chosen = c; chosenBox = box;
        break;
      }
    }
    if(!chosen){
      chosen = candidates[0];
      const left = chosen.x;
      chosenBox = { x: left, y: chosen.y - h/2, w, h };
    }
    placed.push(chosenBox);
    plan.push({ lbl, chosen });
  }

  // Pass 1: contorno nero (su tutti). Garantisce che il contorno di un testo
  // non venga coperto dal contorno di un testo successivo.
  ctx.lineWidth   = LABEL_STROKE_W;
  ctx.strokeStyle = LABEL_STROKE;
  for(const { lbl, chosen } of plan){
    ctx.textAlign = chosen.align;
    ctx.strokeText(lbl.text, chosen.x, chosen.y);
  }
  // Pass 2: riempimento colorato.
  for(const { lbl, chosen } of plan){
    ctx.textAlign = chosen.align;
    ctx.fillStyle = lbl.color || UI.textBright;
    ctx.fillText(lbl.text, chosen.x, chosen.y);
  }

  ctx.restore();
}

// ---- Loop di animazione ----
// Gira SOLO durante la propagazione (PROPAGATION_MS + frame finale). Dopo
// si ferma da solo: gli archi restano dipinti come dashed statici (vis
// continua a chiamare afterDrawing su ogni suo redraw — pan, zoom — e noi
// li ridipingiamo in stato finale grow=1). Stop del loop = niente redraw
// forzati = il wheel di vis-network non viene mai interferito.
function tick(){
  if(!_activeEdges.length){ _rafId = null; return; }
  if(!_animStart) _animStart = performance.now();
  const elapsed = performance.now() - _animStart;
  // Buffer di 50ms oltre PROPAGATION_MS per garantire l'ultimo frame nitido.
  if(elapsed < PROPAGATION_MS + 50){
    _network?.redraw();
    _rafId = requestAnimationFrame(tick);
  } else {
    // Frame finale per cementare lo stato grow=1, poi stop.
    _network?.redraw();
    _rafId = null;
  }
}

function onAfterDrawing(ctx){
  drawTracingEdges(ctx);
  drawLabels(ctx);
}

// ---- API pubblica ----

export function initOverlay(network){
  _network = network;
  network.on('afterDrawing', onAfterDrawing);
}

// edges: [{from, to, color?}, ...] — color opzionale (default: edgeHover).
// Vuoto = stop animazione, redraw per pulire l'overlay.
export function setActiveTraceEdges(edges){
  const next = edges || [];
  // Reset fade-in solo quando la composizione effettivamente cambia (non
  // ad ogni mouseover su sub-rosa). Confronto strutturale leggero.
  const sameSet = next.length === _activeEdges.length
    && next.every((e, i) => _activeEdges[i] && e.from === _activeEdges[i].from && e.to === _activeEdges[i].to);
  _activeEdges = next;
  if(_activeEdges.length){
    if(!sameSet) _animStart = 0;
    if(!_rafId) _rafId = requestAnimationFrame(tick);
  } else {
    if(_rafId){ cancelAnimationFrame(_rafId); _rafId = null; }
    _network?.redraw();
  }
}

export function setActiveLabels(labels){
  _activeLabels = labels || [];
  _network?.redraw();
}

export function clearOverlay(){
  setActiveTraceEdges([]);
  setActiveLabels([]);
}
