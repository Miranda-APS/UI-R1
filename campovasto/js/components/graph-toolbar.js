// Toolbar overlay nel #graph (bottom-right): strumenti di manipolazione del
// grafo per i field editabili (nuovo, medio). Visibile solo quando il field
// attivo accetta modifiche utente — in vasto è nascosta.
//
// Bottoni: riorganizza, undo (Ctrl+Z), redo (Ctrl+Shift+Z).

import { undo, redo, canUndo, canRedo, setOnChange } from '../history.js';
import { activeId, getActive, saveField } from '../manager.js';
import { getNuovoLayout } from '../ui-state.js';
import { applyRectangularLayout, applyDimensionalLayout } from '../layouts/rectangular.js';
import { network } from '../graph.js';

let _root = null;
let _onReorganize = null;

// Permette ad app.js di passare un hook custom (es. refreshFieldUI) dopo
// la riorganizzazione. Se non impostato, refresh implicito via fit.
export function setOnReorganize(fn){ _onReorganize = fn; }

export function buildGraphToolbar(){
  const host = document.getElementById('graph');
  if(!host || _root) return;
  const bar = document.createElement('div');
  bar.id = 'graphToolbar';
  bar.innerHTML = `
    <button id="gtReorg" type="button" title="riorganizza il grafo" aria-label="riorganizza">⤧</button>
    <button id="gtUndo" type="button" title="annulla (Ctrl+Z)" aria-label="annulla">↶</button>
    <button id="gtRedo" type="button" title="ripeti (Ctrl+Shift+Z)" aria-label="ripeti">↷</button>
  `;
  host.appendChild(bar);
  _root = bar;

  bar.querySelector('#gtReorg').addEventListener('click', doReorganize);
  bar.querySelector('#gtUndo').addEventListener('click', doUndo);
  bar.querySelector('#gtRedo').addEventListener('click', doRedo);

  setOnChange(refreshGraphToolbar);
  refreshGraphToolbar();
}

// Riorganizza il campo nuovo: rilascia i flag _userPositioned (l'utente
// ha chiesto un reset esplicito, vince la geometria sul drag manuale) e
// riapplica il layout corrente con uno spread aggressivo. Centro la
// vista con fit() per tornare a una scena leggibile dopo estrazioni KG
// che hanno gonfiato il campo.
export function doReorganize(){
  if(activeId !== 'nuovo') return;
  const F = getActive();
  if(!F || !F.words?.length) return;
  // Rilascia i lock manuali: il "riorganizza" è un atto deliberato
  // dell'utente per ripartire pulito. Senza questo, le parole bloccate
  // dal drag restano dove sono e bloccano la riorganizzazione.
  for(const w of F.words){ w._userPositioned = false; }
  if(getNuovoLayout() === 'rectangular'){
    applyRectangularLayout(F);
  } else {
    // applyDimensionalLayout ricalcola le posizioni via firme 8D + uno
    // spread leggero (minDist:55). Niente spread aggressivo aggiuntivo:
    // farebbe uscire i nodi dal viewport.
    applyDimensionalLayout(F);
  }
  saveField('nuovo');
  try { network?.fit({ animation: { duration: 400, easingFunction: 'easeInOutQuad' } }); } catch(_){}
  _onReorganize?.();
}

// Esposti perché chiamati anche dalle scorciatoie tastiera (vedi keyboard.js).
export function doUndo(){
  if(activeId !== 'nuovo') return;
  undo(activeId);
}
export function doRedo(){
  if(activeId !== 'nuovo') return;
  redo(activeId);
}

// Aggiorna visibilità della toolbar e enable/disable dei bottoni in base
// al field attivo e allo stato dello stack. Da chiamare anche al cambio field.
export function refreshGraphToolbar(){
  if(!_root) return;
  const id = activeId;
  const editable = id === 'nuovo';
  // Durante la "comprensione frase" l'animazione canvas guida la scena —
  // undo/redo competono con il flusso e disorientano. Si nascondono finché
  // l'utente non esce dalla modalità (commitExpansion o switch di campo).
  const inComprehension = document.body.classList.contains('comprensione-frase');
  _root.style.display = (editable && !inComprehension) ? 'flex' : 'none';
  const u = _root.querySelector('#gtUndo');
  const r = _root.querySelector('#gtRedo');
  const reorg = _root.querySelector('#gtReorg');
  if(u) u.disabled = !editable || !canUndo(id);
  if(r) r.disabled = !editable || !canRedo(id);
  // Riorganizza disabilitato se non c'è niente da riorganizzare.
  const F = editable ? getActive() : null;
  if(reorg) reorg.disabled = !editable || !F || !F.words?.length;
}
