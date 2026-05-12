// Context menu — fonte unica per nodi/archi/area-vuota.
// Stessa struttura, stesso comportamento di chiusura, stessa accessibilità.
//
// Voci ammesse:
//   { kind: 'title', label, color? }    — non cliccabile, header colorato
//   { kind: 'title', html }              — header HTML grezzo (es. multi-colore)
//   { kind: 'sep' }                      — separatore
//   { kind: 'item', label, action, danger?, onClick } — voce attiva
//
// Esempio:
//   openCtxMenu({ x, y, items: [
//     { kind: 'title', label: 'parola', color: '#5AAFE8' },
//     { kind: 'sep' },
//     { kind: 'item', label: 'modifica dimensioni', action: 'edit-dims', onClick: () => {...} },
//     { kind: 'item', label: 'elimina', action: 'delete', danger: true, onClick: () => {...} },
//   ]})

import { esc } from '../geometry.js';

let _outsideHandler = null;

export function closeCtxMenu(){
  document.getElementById('ctxMenu')?.classList.remove('open');
  if(_outsideHandler){
    document.removeEventListener('mousedown', _outsideHandler);
    _outsideHandler = null;
  }
}

export function openCtxMenu({ x, y, items }){
  const m = document.getElementById('ctxMenu');
  if(!m) return;

  m.innerHTML = items.map(it => {
    if(it.kind === 'sep') return '<div class="ctx-sep"></div>';
    if(it.kind === 'title'){
      const style = it.color ? ` style="color:${it.color}"` : '';
      const inner = it.html ?? esc(it.label || '');
      return `<div class="ctx-item ctx-title"${style}>${inner}</div>`;
    }
    const cls = 'ctx-item' + (it.danger ? ' danger' : '');
    return `<div class="${cls}" data-action="${esc(it.action)}">${esc(it.label)}</div>`;
  }).join('');

  m.style.left = x + 'px';
  m.style.top  = y + 'px';
  m.classList.add('open');

  m.querySelectorAll('.ctx-item[data-action]').forEach(el => {
    el.onclick = () => {
      const action = el.dataset.action;
      const item = items.find(i => i.kind === 'item' && i.action === action);
      closeCtxMenu();
      item?.onClick?.();
    };
  });

  // Chiudi al click fuori. Single handler per istanza, rimosso a closeCtxMenu().
  if(_outsideHandler) document.removeEventListener('mousedown', _outsideHandler);
  _outsideHandler = (e) => {
    if(!m.contains(e.target)) closeCtxMenu();
  };
  setTimeout(() => document.addEventListener('mousedown', _outsideHandler), 0);
}
