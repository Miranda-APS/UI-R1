// Overlay HTML delle 8 etichette dimensione (POTERE, MATERIA, ARDORE, …).
// Restano FISSE nel viewport durante zoom/pan, a differenza delle dim-label
// che vivevano come nodi nel canvas vis-network e scomparivano fuori
// quadro al primo zoom-in. Richiesto dalla collega: "le otto dimensioni
// nel campo spariscono fuori se zoommi, devono stare su un altro livello
// e restare sempre nel quadro".
//
// Posizione: 8 ancore intorno al perimetro del #graph, con offset interno
// piccolo. Allineamento angolare coerente con DIM_ANGLES (lo stesso che
// piazza i nodi nel campo): chi guarda capisce dove "punta" ciascuna
// dimensione anche quando i nodi non sono visibili.

import { DIM_NAMES, DIM_ANGLES } from '../constants.js';
import { DIM_COLORS } from '../theme.js';

let _root = null;

// Mapping angolo → posizione viewport (CSS top/left/transform). Usiamo
// posizionamento in % rispetto al container #graph, con un margin interno
// per non incollare le label al bordo.
function positionForAngle(angleRad){
  // -π/2 = top (y -1), 0 = right (x +1), π/2 = bottom, ±π = left.
  const cosA = Math.cos(angleRad);
  const sinA = Math.sin(angleRad);
  // Distanza dal centro = quasi al bordo (47%) per restare sopra al canvas.
  const r = 47;
  const x = 50 + cosA * r;  // %
  const y = 50 + sinA * r;
  // Allineamento testo: spinge il box verso il bordo, non lo centra al
  // perimetro (così non si sovrappone agli angoli del canvas).
  let tx = '-50%', ty = '-50%';
  if(cosA >  0.5) tx = '-100%';
  if(cosA < -0.5) tx = '0%';
  if(sinA >  0.5) ty = '-100%';
  if(sinA < -0.5) ty = '0%';
  return {
    left: x + '%',
    top:  y + '%',
    transform: `translate(${tx}, ${ty})`,
  };
}

export function buildDimOverlay(){
  const host = document.getElementById('graph');
  if(!host || _root) return;
  const wrap = document.createElement('div');
  wrap.id = 'dimOverlay';
  wrap.className = 'dim-overlay';
  for(let i = 0; i < 8; i++){
    const el = document.createElement('span');
    el.className = 'dim-overlay-label';
    el.textContent = DIM_NAMES[i].toUpperCase();
    el.style.color = DIM_COLORS[i];
    const pos = positionForAngle(DIM_ANGLES[i]);
    el.style.left = pos.left;
    el.style.top  = pos.top;
    el.style.transform = pos.transform;
    wrap.appendChild(el);
  }
  host.appendChild(wrap);
  _root = wrap;
}

// Visibilità: vogliamo l'overlay solo nel campo vasto (dove la geometria
// 8D è la cifra). Negli altri layout (rectangular del nuovo) le dimensioni
// non corrispondono a posizioni → sarebbero fuorvianti.
export function setDimOverlayVisible(visible){
  if(!_root) return;
  _root.style.display = visible ? 'block' : 'none';
}
