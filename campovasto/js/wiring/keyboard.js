// Scorciatoie tastiera globali:
// — Escape: chiude editPanel/sentencePanel o deseleziona la parola attiva.
// — Ctrl/Cmd+Z: undo nel field editabile attivo (nuovo/medio).
// — Ctrl/Cmd+Shift+Z: redo nel field editabile attivo.

import { getActive } from '../manager.js';
import { closeEdit } from '../editor.js';
import { deselectWord } from './selection.js';
import { doUndo, doRedo } from '../components/graph-toolbar.js';

function isEditableTarget(target){
  const tag = (target?.tagName || '').toLowerCase();
  if(tag === 'input' || tag === 'textarea') return true;
  return target?.isContentEditable === true;
}

export function wireKeyboard(){
  document.addEventListener('keydown', e => {
    // Undo / redo (prima di Escape: anche con search non focusata).
    if((e.metaKey || e.ctrlKey) && (e.key === 'z' || e.key === 'Z')){
      if(isEditableTarget(e.target)) return;   // lascia agire il browser sui field di input
      e.preventDefault();
      if(e.shiftKey) doRedo(); else doUndo();
      return;
    }

    if(e.key !== 'Escape') return;
    const si = document.getElementById('search');
    if(document.activeElement === si) return;

    const panel = document.getElementById('editPanel');
    if(panel?.classList.contains('open')){ closeEdit(); return; }

    const sp = document.getElementById('sentencePanel');
    if(sp && sp.style.display !== 'none'){ sp.style.display = 'none'; return; }

    const F = getActive();
    if(F?.selected) deselectWord();
  });
}
