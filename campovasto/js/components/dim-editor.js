// Editor delle 8 dimensioni — fonte unica. Usato:
//   - in sidebar (modalità inline): mostra/edita la firma della parola selezionata
//   - in editPanel (modalità panel):  aggiunta parola + modifica dimensioni
//
// Stesso markup, stessa interazione, stesso colore — coerente per definizione
// (vedi CLAUDE.md §3 "una responsabilità per file").
//
// API:
//   const ed = new DimEditor(host, { sig, onChange, onCommit, readonly });
//   ed.setSig(newSig);   // aggiorna senza ridisegnare
//   ed.getSig();         // restituisce una copia
//   ed.destroy();        // rimuove i listener globali
//
// Eventi:
//   onChange(sig)  → ad ogni movimento (drag/typing) — per anteprima posizione
//   onCommit(sig)  → al rilascio drag o blur input — per persistere

import { DIM_NAMES, DIM_DESC, NEUTRAL_SIG } from '../constants.js';

export class DimEditor {
  constructor(host, { sig = NEUTRAL_SIG.slice(), onChange = null, onCommit = null, readonly = false } = {}){
    this.host = host;
    this.sig = (sig || NEUTRAL_SIG).slice();
    this._onChange = onChange;
    this._onCommit = onCommit;
    this.readonly = !!readonly;
    this._cleanup = null;
    this._render();
  }

  _render(){
    this.host.innerHTML = '';
    this.host.classList.add('dim-editor');

    for(let i = 0; i < 8; i++){
      this.host.appendChild(this._buildRow(i));
    }
    if(!this.readonly) this._wireDrag();
  }

  _buildRow(i){
    const v = this.sig[i] || 0;
    const dimName = DIM_NAMES[i];

    const row = document.createElement('div');
    row.className = 'dim-row';

    const nameSpan = document.createElement('span');
    nameSpan.className = 'dim-name';
    nameSpan.setAttribute('data-dim', dimName);
    nameSpan.textContent = dimName;

    const wrap = document.createElement('div');
    wrap.className = 'dim-bar-wrap';
    wrap.setAttribute('data-dim', i);

    const bar = document.createElement('div');
    bar.className = 'dim-bar';
    bar.setAttribute('data-dim', dimName);
    bar.style.width = v + '%';
    wrap.appendChild(bar);

    const parts = DIM_DESC[i].split(' o ');
    if(parts.length === 2){
      const l = document.createElement('span'); l.className = 'dim-extreme left';  l.textContent = parts[1];
      const r = document.createElement('span'); r.className = 'dim-extreme right'; r.textContent = parts[0];
      wrap.appendChild(l); wrap.appendChild(r);
    } else {
      const s = document.createElement('span'); s.className = 'dim-extreme single'; s.textContent = DIM_DESC[i];
      wrap.appendChild(s);
    }

    const valInput = document.createElement('input');
    valInput.type = 'number';
    valInput.className = 'dim-val';
    valInput.value = v;
    valInput.min = 0;
    valInput.max = 100;
    if(this.readonly) valInput.disabled = true;

    if(!this.readonly){
      valInput.addEventListener('input', () => {
        let nv = parseInt(valInput.value);
        if(!Number.isFinite(nv)) nv = 0;
        nv = Math.min(100, Math.max(0, nv));
        this.sig[i] = nv;
        bar.style.width = nv + '%';
        this._onChange?.(this.sig.slice());
      });
      valInput.addEventListener('change', () => {
        valInput.value = this.sig[i];
        this._onCommit?.(this.sig.slice());
      });
    }

    row.appendChild(nameSpan);
    row.appendChild(wrap);
    row.appendChild(valInput);
    return row;
  }

  // Un solo set di listener globali (mousemove/mouseup) per istanza — non 8.
  // Delegation sul host per il mousedown.
  _wireDrag(){
    let activeIdx = -1;
    let activeBar = null;
    let activeWrap = null;
    let activeInput = null;

    const updateFromEvent = (e) => {
      if(activeIdx < 0 || !activeWrap) return;
      const rect = activeWrap.getBoundingClientRect();
      const percent = Math.min(100, Math.max(0, ((e.clientX - rect.left) / rect.width) * 100));
      const nv = Math.round(percent);
      activeBar.style.width = nv + '%';
      activeInput.value = nv;
      this.sig[activeIdx] = nv;
      this._onChange?.(this.sig.slice());
    };

    const onMouseMove = (e) => updateFromEvent(e);
    const onMouseUp = () => {
      if(activeIdx >= 0){
        this._onCommit?.(this.sig.slice());
        activeIdx = -1;
        activeBar = null;
        activeWrap = null;
        activeInput = null;
      }
    };

    const onMouseDown = (e) => {
      const wrap = e.target.closest('.dim-bar-wrap');
      if(!wrap || !this.host.contains(wrap)) return;
      activeIdx = parseInt(wrap.dataset.dim);
      activeBar = wrap.querySelector('.dim-bar');
      activeWrap = wrap;
      activeInput = wrap.parentElement.querySelector('.dim-val');
      updateFromEvent(e);  // jump-to: click anche senza drag aggiorna
      e.preventDefault();
    };

    this.host.addEventListener('mousedown', onMouseDown);
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);

    this._cleanup = () => {
      this.host.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };
  }

  // Aggiorna i valori senza ricostruire il DOM (preserva focus, evita flicker).
  setSig(newSig){
    const next = (newSig || NEUTRAL_SIG).slice();
    for(let i = 0; i < 8; i++){
      this.sig[i] = next[i] || 0;
      const bar = this.host.querySelector(`.dim-bar-wrap[data-dim="${i}"] .dim-bar`);
      const inp = this.host.querySelectorAll('.dim-val')[i];
      if(bar) bar.style.width = (this.sig[i] || 0) + '%';
      if(inp && document.activeElement !== inp) inp.value = this.sig[i] || 0;
    }
  }

  getSig(){ return this.sig.slice(); }

  destroy(){
    this._cleanup?.();
    this._cleanup = null;
    this.host.classList.remove('dim-editor');
  }
}
