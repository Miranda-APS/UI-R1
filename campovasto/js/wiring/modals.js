// Modali "salva / carica campo dal server" (collettivi).
// Tutto agisce sul campo nuovo: in pre-launch i salvataggi sono visti
// come un unico spazio condiviso (niente filtro per field_id sul client).

import { FIELDS, saveField, registerField } from '../manager.js';
import { Field } from '../field.js';
import { esc } from '../geometry.js';
import { UI, tokens } from '../theme.js';

let _onFieldLoaded = null;  // (fieldId) => void — iniettato da view-switcher
export function setModalCallbacks({ onFieldLoaded }){
  _onFieldLoaded = onFieldLoaded;
}

export function closeCollectiveModal(){
  document.getElementById('collectiveModal').style.display = 'none';
  document.getElementById('collectiveBox').innerHTML = '';
}

export async function openSaveModal(){
  const F = FIELDS.nuovo;
  if(!F || F.words.length === 0) return;
  const overlay = document.getElementById('collectiveModal');
  const box = document.getElementById('collectiveBox');
  overlay.style.display = 'flex';
  const defaultName = F.sentence ? F.sentence.slice(0, 30) : 'campo nuovo';
  box.innerHTML = `
    <h3>Salva il tuo campo sul server</h3>
    <div class="modal-subtitle">Il gruppo potrà riprenderlo conoscendo nome e password.</div>
    <label>nome</label>
    <input type="text" id="saveName" value="${esc(defaultName)}" maxlength="80">
    <label>password</label>
    <input type="password" id="savePassword" placeholder="scegli una password">
    <div id="existingSaves" style="margin-top:12px"></div>
    <div id="saveStatus"></div>
    <div class="modal-actions">
      <button id="saveCancel">annulla</button>
      <button class="primary" id="saveConfirm">salva</button>
    </div>
  `;
  document.getElementById('saveCancel').onclick = closeCollectiveModal;
  overlay.onclick = (e) => { if(e.target === overlay) closeCollectiveModal(); };

  try {
    const r = await fetch('/api/saved_fields');
    const list = await r.json();
    const listEl = document.getElementById('existingSaves');
    if(list.length){
      listEl.innerHTML = `
        <label>campi già salvati</label>
        <div class="saved-list">${list.map(x => {
          const date = new Date(x.created_at * 1000).toLocaleString('it-IT', { dateStyle: 'short', timeStyle: 'short' });
          return `<div class="saved-item" style="cursor:default">
            <span class="name">${esc(x.name)}</span>
            <span class="meta">${date}</span>
          </div>`;
        }).join('')}</div>
      `;
    }
  } catch(_){}

  document.getElementById('saveConfirm').onclick = async () => {
    const name = document.getElementById('saveName').value.trim();
    const password = document.getElementById('savePassword').value;
    const status = document.getElementById('saveStatus');
    if(!name || !password){
      status.innerHTML = '<div class="modal-status err">nome e password obbligatori</div>';
      return;
    }
    const data = F.toJSON();
    try {
      const r = await fetch('/api/saved_fields/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, password, field_id: 'nuovo', data }),
      });
      const d = await r.json();
      status.innerHTML = d.ok
        ? `<div class="modal-status ok">${esc(d.message)}</div>`
        : `<div class="modal-status err">${esc(d.message)}</div>`;
      if(d.ok) setTimeout(closeCollectiveModal, 1200);
    } catch(err){
      status.innerHTML = `<div class="modal-status err">errore di rete: ${esc(String(err))}</div>`;
    }
  };
  setTimeout(() => document.getElementById('saveName')?.focus(), 50);
}

export async function openLoadModal(){
  const overlay = document.getElementById('collectiveModal');
  const box = document.getElementById('collectiveBox');
  overlay.style.display = 'flex';
  box.innerHTML = `
    <h3>Carica un campo dal server</h3>
    <div class="modal-subtitle">Scegli un campo salvato e inserisci la password.</div>
    <div class="saved-list" id="savedList">caricamento…</div>
    <label>password</label>
    <input type="password" id="loadPassword" placeholder="password del campo selezionato">
    <div id="loadStatus"></div>
    <div class="modal-actions">
      <button id="loadCancel">annulla</button>
      <button class="primary" id="loadConfirm" disabled>carica</button>
    </div>
  `;
  document.getElementById('loadCancel').onclick = closeCollectiveModal;
  overlay.onclick = (e) => { if(e.target === overlay) closeCollectiveModal(); };
  let selectedSlug = null;

  try {
    const r = await fetch('/api/saved_fields');
    const list = await r.json();
    const listEl = document.getElementById('savedList');
    if(list.length === 0){
      listEl.innerHTML = `<div style="padding:12px;color:${UI.textDim};font-size:12px">Nessun campo salvato.</div>`;
    } else {
      listEl.innerHTML = list.map(x => {
        const date = new Date(x.created_at * 1000).toLocaleString('it-IT', { dateStyle: 'short', timeStyle: 'short' });
        return `<div class="saved-item" data-slug="${esc(x.slug)}" data-name="${esc(x.name)}">
          <span class="name">${esc(x.name)}</span>
          <span style="display:flex; gap:8px; align-items:center">
            <span class="meta">${date}</span>
            <button class="saved-delete" data-slug="${esc(x.slug)}" data-name="${esc(x.name)}" title="elimina">✕</button>
          </span>
        </div>`;
      }).join('');
      listEl.querySelectorAll('.saved-item').forEach(it => {
        it.onclick = (e) => {
          if(e.target.classList.contains('saved-delete')) return;
          listEl.querySelectorAll('.saved-item').forEach(x => x.classList.remove('selected'));
          it.classList.add('selected');
          selectedSlug = it.dataset.slug;
          document.getElementById('loadConfirm').disabled = false;
        };
      });
      listEl.querySelectorAll('.saved-delete').forEach(btn => {
        btn.onclick = (e) => {
          e.stopPropagation();
          const slug = btn.dataset.slug;
          const pw = document.getElementById('loadPassword')?.value || '';
          if(!pw){
            document.getElementById('loadStatus').innerHTML =
              '<div class="modal-status err">inserisci la password per eliminare</div>';
            return;
          }
          fetch('/api/saved_fields/delete', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ slug, password: pw }),
          }).then(r => r.json()).then(d => {
            const status = document.getElementById('loadStatus');
            if(d.ok){
              status.innerHTML = `<div class="modal-status ok">${esc(d.message)}</div>`;
              btn.closest('.saved-item')?.remove();
              if(selectedSlug === slug){
                selectedSlug = null;
                document.getElementById('loadConfirm').disabled = true;
              }
            } else {
              status.innerHTML = `<div class="modal-status err">${esc(d.message)}</div>`;
            }
          }).catch(err => {
            document.getElementById('loadStatus').innerHTML =
              `<div class="modal-status err">errore: ${esc(String(err))}</div>`;
          });
        };
      });
    }
  } catch(err){
    // Modale potrebbe essere stata chiusa durante la fetch — guard sull'elemento.
    const el = document.getElementById('savedList');
    if(el) el.innerHTML =
      `<div style="padding:12px;color:${UI.textDim};font-size:12px">errore: ${esc(String(err))}</div>`;
  }

  document.getElementById('loadConfirm').onclick = async () => {
    if(!selectedSlug) return;
    const password = document.getElementById('loadPassword').value;
    const status = document.getElementById('loadStatus');
    if(!password){
      status.innerHTML = '<div class="modal-status err">inserisci la password</div>';
      return;
    }
    try {
      const r = await fetch('/api/saved_fields/load', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ slug: selectedSlug, password }),
      });
      const d = await r.json();
      if(d.ok && d.data){
        const V = FIELDS.vasto;
        const newF = new Field('nuovo', V.frame);
        newF.addDimLabels();
        newF.hydrate(d.data);
        newF.spreadNonOverlapping();
        registerField('nuovo', newF);
        saveField('nuovo');
        status.innerHTML = `<div class="modal-status ok">campo "${esc(d.name)}" caricato</div>`;
        closeCollectiveModal();
        _onFieldLoaded?.('nuovo');
      } else {
        status.innerHTML = `<div class="modal-status err">${esc(d.message || 'errore')}</div>`;
      }
    } catch(err){
      status.innerHTML = `<div class="modal-status err">errore: ${esc(String(err))}</div>`;
    }
  };
}
