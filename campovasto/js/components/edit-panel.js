// Pannello di edit (#editPanel) — struttura fissa: title + body + actions.
// Tutte le funzioni openX di editor.js passano per qui. Non costruiscono
// più innerHTML a mano. Coerenza visuale per definizione (vedi CLAUDE.md §3).
//
// API:
//   const handle = openPanel({
//     title: 'Aggiungi parola',           // string o HTML safe
//     build: (body) => { ... },           // callback che popola il body
//     actions: [                          // bottoni in basso
//       { label: 'aggiungi', primary: true, onClick: () => {...} },
//       { label: 'annulla',                onClick: closePanel    },
//     ],
//     onClose: () => { ... },             // chiamato a closePanel()
//   });
//   // handle.body  → riferimento al div del body se serve (es. per setSig)
//   // handle.close → alias di closePanel
//
//   closePanel();

let _activeOnClose = null;
let _backdropEl = null;
let _keyHandler = null;

function ensureBackdrop(){
  if(_backdropEl) return _backdropEl;
  const el = document.createElement('div');
  el.id = 'editPanelBackdrop';
  el.className = 'edit-panel-backdrop';
  el.addEventListener('mousedown', (e) => {
    // Click sul backdrop (non sul panel sopra) → chiusura.
    if(e.target === el) closePanel();
  });
  document.body.appendChild(el);
  _backdropEl = el;
  return el;
}

export function closePanel(){
  const p = document.getElementById('editPanel');
  if(!p) return;
  p.classList.remove('open');
  if(_backdropEl) _backdropEl.classList.remove('open');
  if(_keyHandler){
    document.removeEventListener('keydown', _keyHandler);
    _keyHandler = null;
  }
  const cb = _activeOnClose;
  _activeOnClose = null;
  cb?.();
}

export function openPanel({ title = '', build = null, actions = [], onClose = null } = {}){
  const panel = document.getElementById('editPanel');
  if(!panel) return null;

  // Se c'è già un onClose pendente, eseguilo ora (chiusura implicita).
  if(_activeOnClose){
    const prev = _activeOnClose;
    _activeOnClose = null;
    prev();
  }

  panel.innerHTML = '';

  const titleEl = document.createElement('h3');
  titleEl.className = 'edit-title';
  titleEl.innerHTML = title;
  panel.appendChild(titleEl);

  const body = document.createElement('div');
  body.className = 'edit-body';
  panel.appendChild(body);

  build?.(body);

  if(actions.length){
    const actionsEl = document.createElement('div');
    actionsEl.className = 'edit-actions';
    actions.forEach(a => {
      const btn = document.createElement('button');
      let cls = 'edit-btn';
      if(a.primary) cls += ' primary';
      if(a.danger)  cls += ' danger';
      btn.className = cls;
      btn.textContent = a.label;
      btn.onclick = () => a.onClick?.();
      actionsEl.appendChild(btn);
    });
    panel.appendChild(actionsEl);
  }

  panel.classList.add('open');
  ensureBackdrop().classList.add('open');
  _activeOnClose = onClose;

  // ESC per chiudere — coerente con il backdrop click. Singola registrazione.
  _keyHandler = (e) => {
    if(e.key === 'Escape') closePanel();
  };
  document.addEventListener('keydown', _keyHandler);

  return { panel, body, close: closePanel };
}
