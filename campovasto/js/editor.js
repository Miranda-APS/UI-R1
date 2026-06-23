// Pannelli: aggiungi parola, aggiungi relazione, modifica dimensioni/relazioni,
// context menu, radar Octalysis. Operano sul field attivo.
//
// Tutta la struttura passa per i componenti dedicati:
//   - DimEditor   → editor delle 8 dimensioni (sidebar + panel)
//   - EditPanel   → struttura {title, body, actions} dell'#editPanel
//   - CtxMenu     → menu contestuali su nodi/archi/area-vuota
//
// Niente innerHTML manuale per ctx-menu o titoli/azioni del panel
// (vedi CLAUDE.md §3 "una responsabilità per file").

import { CD_NAMES, RL, REL_LEGEND_LABEL_IT, NEUTRAL_SIG } from './constants.js';
import { CD_COLORS, UI, colorForSig } from './theme.js';
import { computeOctalysis, esc } from './geometry.js';
import { getActive, saveField, FIELDS } from './manager.js';
import { DimEditor } from './components/dim-editor.js';
import { openPanel, closePanel } from './components/edit-panel.js';
import { openCtxMenu, closeCtxMenu } from './components/ctx-menu.js';
import { setLinkMode } from './ui-state.js';
import {
  flagsForNewWord, flagsForNewEdge,
  applyEditFlagsToWord, applyEditFlagsToEdge,
} from './policies/word.js';
import { openExtractDialog } from './components/extract-dialog.js';
import { extractRelationsForWord } from './relations-extract.js';

// Callback iniettati: evita dipendenze circolari con app.js.
let _onWordChanged = null;
let _onEdgeChanged = null;
let _onSelect = null;
let _onDeselect = null;
export function setEditorCallbacks({ onWordChanged, onEdgeChanged, onSelect, onDeselect }){
  _onWordChanged = onWordChanged;
  _onEdgeChanged = onEdgeChanged;
  _onSelect = onSelect;
  _onDeselect = onDeselect;
}

// L'editor dimensioni dentro l'editPanel (uno alla volta). Distrutto a closeEdit.
let _activeDimEditor = null;

// closeEdit è la chiusura "ufficiale" del panel — gestisce anche teardown
// dell'editor dimensioni se attivo.
export function closeEdit(){
  closePanel();  // attiva l'onClose registrato a openPanel, che chiude DimEditor
}

// ---- Aggiungi parola -----------------------------------------------------
//
// Modale a tab. Tab disponibili (richieste della collega):
//   - dimensioni: editor 8D (default)
//   - relazioni:  multi-row di archi che partono dalla parola corrente
//   - octalisys:  preview dei drive Octalysis derivati dalla firma
//   - collega:    chiude la modale e attiva linkMode sul campo locale
//                 (drag-to-connect con etichette di tutte le parole visibili)
//
// La parola viene salvata appena valida (input.blur), così il cambio di tab
// non perde lo stato. Salvataggio idempotente (riusa updateWordSig se esiste).

const ADD_WORD_TABS = [
  { id: 'dimensioni', label: 'dimensioni' },
  { id: 'relazioni',  label: 'relazioni' },
  { id: 'octalisys',  label: 'octalisys' },
  { id: 'collega',    label: 'collega' },
];

let _addWordCtx = null;  // { word, sig, prefill, dimEditor, body }

export function openAddWord(prefill = {}){
  const initialSig = (prefill.sig || NEUTRAL_SIG).slice();
  _addWordCtx = {
    word: (prefill.word || '').toLowerCase(),
    sig: initialSig,
    prefill,
    dimEditor: null,
    body: null,
    activeTab: 'dimensioni',
  };

  openPanel({
    title: `Aggiungi parola${prefill.word ? ' — ' + esc(prefill.word) : ''}`,
    build: (body) => {
      _addWordCtx.body = body;
      body.innerHTML = `
        <div class="edit-row">
          <label>parola</label>
          <input type="text" id="newWordInput" class="edit-input"
                 value="${esc(prefill.word || '')}" placeholder="scrivi..."
                 ${prefill.locked ? 'disabled' : ''}>
        </div>
        <div class="add-word-tabs" role="tablist">
          ${ADD_WORD_TABS.map(t =>
            `<button type="button" class="add-word-tab${t.id === 'dimensioni' ? ' attivo' : ''}" data-tab="${t.id}">${esc(t.label)}</button>`
          ).join('')}
        </div>
        <div class="add-word-tab-host" id="addWordTabHost"></div>
      `;
      // Salva la parola al blur (autosave): chi cambia tab non perde lo
      // stato, e a fine modale non serve un bottone "salva" dedicato.
      const wi = body.querySelector('#newWordInput');
      wi.addEventListener('blur', _autosaveWord);
      // Tab switching.
      body.querySelectorAll('.add-word-tab').forEach(btn => {
        btn.addEventListener('click', () => _switchAddWordTab(btn.dataset.tab));
      });
      _renderAddWordTab('dimensioni');
    },
    actions: [
      { label: 'fine', primary: true, onClick: () => { _autosaveWord(); closeEdit(); } },
    ],
    onClose: () => {
      _addWordCtx?.dimEditor?.destroy();
      _addWordCtx = null;
    },
  });

  const wi = document.getElementById('newWordInput');
  if(wi && !prefill.locked) setTimeout(() => wi.focus(), 50);
}

function _switchAddWordTab(tabId){
  if(!_addWordCtx) return;
  const body = _addWordCtx.body;
  body.querySelectorAll('.add-word-tab').forEach(b => {
    b.classList.toggle('attivo', b.dataset.tab === tabId);
  });
  // "collega" non ha un pannello: chiude la modale e attiva linkMode.
  if(tabId === 'collega'){
    _autosaveWord();
    closeEdit();
    setLinkMode(true);
    return;
  }
  _addWordCtx.activeTab = tabId;
  _renderAddWordTab(tabId);
}

function _renderAddWordTab(tabId){
  if(!_addWordCtx) return;
  const host = _addWordCtx.body.querySelector('#addWordTabHost');
  if(!host) return;
  // Distruggi il dim editor precedente se esiste (re-creato solo per "dimensioni").
  if(_addWordCtx.dimEditor){ _addWordCtx.dimEditor.destroy(); _addWordCtx.dimEditor = null; }
  host.innerHTML = '';

  if(tabId === 'dimensioni'){
    const sub = document.createElement('div');
    sub.innerHTML = `
      <div class="edit-section-title">valore delle dimensioni</div>
      <div id="dimEditorHost"></div>
    `;
    host.appendChild(sub);
    _addWordCtx.dimEditor = new DimEditor(sub.querySelector('#dimEditorHost'), {
      sig: _addWordCtx.sig,
      onChange: (sig) => { _addWordCtx.sig = sig; },
      onCommit: (sig) => { _addWordCtx.sig = sig; _autosaveWord(); },
    });
  } else if(tabId === 'octalisys'){
    const oct = computeOctalysis(_addWordCtx.sig);
    host.innerHTML = `
      <div class="edit-section-title">Octalysis derivato dalle dimensioni</div>
      <div class="oct-preview">${
        CD_NAMES.map((n, i) =>
          `<span style="color:${CD_COLORS[i]}">${esc(n)}: ${Math.round(oct[i] * 100)}%</span>`
        ).join(' · ')
      }</div>
    `;
  } else if(tabId === 'relazioni'){
    // Salva la parola prima di mostrare le relazioni: addEdge ha bisogno
    // che il soggetto esista nel field.
    _autosaveWord();
    const word = _addWordCtx.word;
    if(!word){
      host.innerHTML = `<div class="edit-section-title">scrivi prima la parola</div>`;
      return;
    }
    host.innerHTML = `
      <div class="edit-section-title">relazioni che partono da "${esc(word)}"</div>
      <div id="addWordRelsHost"></div>
      <div class="edit-actions-row" style="margin-top:6px">
        <button type="button" class="edit-btn small" id="addWordRelsAdd">+ aggiungi riga</button>
      </div>
    `;
    const rh = host.querySelector('#addWordRelsHost');
    rh.innerHTML = makeRelRowHtml(word);
    _wireRelRow(rh.querySelector('.rel-row-multi'));
    host.querySelector('#addWordRelsAdd').onclick = () => {
      rh.insertAdjacentHTML('beforeend', makeRelRowHtml(word));
      const newRow = rh.querySelector('.rel-row-multi:last-child');
      _wireRelRow(newRow);
      newRow.querySelector('.rel-to').focus();
    };
  }
}

// Salva la parola (add o update sig) usando lo stato corrente del ctx.
// Idempotente: chiamabile più volte senza side-effect.
//
// Lookup KG: se l'utente digita una parola che è già nel campo vasto
// (riconosciuta dal KG) e NON ha toccato manualmente le dimensioni, la
// firma viene presa dal vasto. La parola entra nel campo già con la sua
// posizione semantica corretta — le relazioni del KG restano latenti
// finché l'utente non clicca "⤓ estrai relazioni dal KG" sul nodo.
function _autosaveWord(){
  if(!_addWordCtx) return;
  const F = getActive();
  if(!F) return;
  const wi = document.getElementById('newWordInput');
  const raw = (wi?.value || _addWordCtx.word || '').trim().toLowerCase();
  if(!raw || !/^[a-zàèéìòóùü' -]+$/i.test(raw)) return;
  _addWordCtx.word = raw;

  // Se l'utente non ha toccato il dim editor (sig ancora neutra) e la
  // parola esiste nel vasto, adotta la firma vasto. updateDimEditor
  // riflette il cambio nella UI così l'utente la vede e può ancora
  // modificarla prima di "fine".
  const V = FIELDS.vasto;
  const vw = V?.wordMap?.[raw];
  const sigIsNeutral = _addWordCtx.sig.every((v, i) => v === NEUTRAL_SIG[i]);
  if(vw && sigIsNeutral){
    _addWordCtx.sig = (vw.sig || NEUTRAL_SIG).slice();
    if(_addWordCtx.dimEditor && typeof _addWordCtx.dimEditor.setSig === 'function'){
      try { _addWordCtx.dimEditor.setSig(_addWordCtx.sig); } catch(_){}
    }
  }
  const sig = _addWordCtx.sig;

  const wasNew = !F.hasWord(raw);
  if(!wasNew){
    F.updateWordSig(raw, sig);
    if(F.id !== 'vasto') applyEditFlagsToWord(F.wordMap[raw]);
  } else {
    const newWord = { w: raw, sig: sig.slice(), flags: flagsForNewWord(F) };
    if(_addWordCtx.prefill?.position){
      // L'utente ha fatto right-click su un punto preciso del canvas:
      // rispetta quella posizione (e tienila al riparo dai layout).
      newWord.position = { ..._addWordCtx.prefill.position };
      newWord._userPositioned = true;
    }
    F.addWord(newWord);
  }
  saveField(F.id);
  // Notifica il cambio: in rectangular ricalcola il layout così la parola
  // appena aggiunta appare subito come rettangolo, non come puntino.
  if(wasNew) _onWordChanged?.(raw);
}

// ---- Aggiungi relazione --------------------------------------------------
// Multi-row con autosave: ogni riga è una relazione (da/tipo/a/forza). Il
// tasto + aggiunge una riga vuota in coda. Il tipo è un datalist: l'utente
// può scegliere uno dei tipi noti o digitarne uno nuovo (alias per ora;
// la validazione lato KG resta lato server). Le righe complete vengono
// salvate man mano (autosave intelligente) — chi chiude la modale non
// perde lavoro.

let _relRowSeq = 0;

function makeRelRowHtml(prefillFrom = ''){
  const id = ++_relRowSeq;
  const opts = Object.keys(RL).map(r =>
    `<option value="${esc(r)}">${esc(r)} — ${esc(RL[r])}</option>`
  ).join('');
  return `
    <div class="edit-row rel-row-multi" data-row="${id}" data-saved="0">
      <input type="text" class="edit-input rel-from" value="${esc(prefillFrom)}" placeholder="da..." style="flex:1">
      <input type="text" class="edit-input rel-type" list="relTypeList" placeholder="tipo" style="flex:0 0 9em">
      <input type="text" class="edit-input rel-to" placeholder="a..." style="flex:1">
      <input type="range" class="rel-conf" min="10" max="100" value="80" style="flex:0 0 6em">
      <span class="edit-val rel-conf-val">80</span>
      <span class="rel-saved-mark" title="salvata">·</span>
      <button type="button" class="rel-row-del" title="elimina riga">×</button>
    </div>
  `;
}

export function openAddRel(prefillFrom = ''){
  const datalist = `<datalist id="relTypeList">${
    Object.keys(RL).map(r => `<option value="${esc(r)}">${esc(RL[r])}</option>`).join('')
  }</datalist>`;

  openPanel({
    title: 'Aggiungi relazioni',
    build: (body) => {
      body.innerHTML = `
        ${datalist}
        <div class="edit-section-title">tipo: scegli dalla lista o scrivi un alias nuovo</div>
        <div id="relRowsHost">${makeRelRowHtml(prefillFrom)}</div>
        <div class="edit-actions-row">
          <button type="button" class="edit-btn small" id="addRelRowBtn">+ aggiungi riga</button>
          <span class="rel-autosave-hint" style="margin-left:auto;font-size:11px;color:var(--testo-basso)">
            autosalva quando da/tipo/a sono compilati
          </span>
        </div>
      `;
      const host = body.querySelector('#relRowsHost');
      _wireRelRow(host.querySelector('.rel-row-multi'));
      body.querySelector('#addRelRowBtn').onclick = () => {
        host.insertAdjacentHTML('beforeend', makeRelRowHtml(''));
        const newRow = host.querySelector('.rel-row-multi:last-child');
        _wireRelRow(newRow);
        newRow.querySelector('.rel-from').focus();
      };
    },
    actions: [
      { label: 'fine', primary: true, onClick: closeEdit },
    ],
  });
  // Focus sul primo campo libero della prima riga.
  const firstRow = document.querySelector('#relRowsHost .rel-row-multi');
  const firstFocus = firstRow?.querySelector(prefillFrom ? '.rel-type' : '.rel-from');
  if(firstFocus) setTimeout(() => firstFocus.focus(), 50);
}

// Cabla una singola riga: live-update del valore conf, autosave su blur dei
// campi quando la triade da/tipo/a è completa, delete riga, evita doppi
// salvataggi (data-saved="1").
function _wireRelRow(row){
  if(!row) return;
  const from = row.querySelector('.rel-from');
  const type = row.querySelector('.rel-type');
  const to   = row.querySelector('.rel-to');
  const conf = row.querySelector('.rel-conf');
  const confVal = row.querySelector('.rel-conf-val');
  const delBtn = row.querySelector('.rel-row-del');

  conf.oninput = () => { confVal.textContent = conf.value; };

  const trySave = () => {
    const f = from.value.trim().toLowerCase();
    const r = type.value.trim().toUpperCase();   // KG usa identificatori uppercase
    const t = to.value.trim().toLowerCase();
    const c = parseInt(conf.value);
    if(!f || !r || !t || f === t) return;
    // Edge dedup: chiave from|to|rel. Se esiste già, non salviamo di nuovo.
    const F = getActive();
    if(!F) return;
    const key = `${f}|${t}|${r}`;
    if(F.edgeByKey?.[key]){
      row.dataset.saved = '1';
      row.classList.add('rel-row-saved');
      return;
    }
    const newWordFlags = flagsForNewWord(F);
    if(!F.hasWord(f)) F.addWord({ w: f, sig: NEUTRAL_SIG.slice(), flags: newWordFlags });
    if(!F.hasWord(t)) F.addWord({ w: t, sig: NEUTRAL_SIG.slice(), flags: newWordFlags });
    F.addEdge({ from: f, to: t, rel: r, conf: c, flags: flagsForNewEdge(F) });
    saveField(F.id);
    row.dataset.saved = '1';
    row.classList.add('rel-row-saved');
  };

  // Autosave: blur dei campi triade + change del tipo. Niente save su input
  // per non spammare addEdge ad ogni keystroke.
  [from, type, to].forEach(el => el.addEventListener('blur', trySave));
  type.addEventListener('change', trySave);

  delBtn.onclick = () => {
    // Se la riga era già salvata, l'utente sta esprimendo l'intenzione di
    // rimuovere l'arco appena creato. Lo togliamo dal field.
    if(row.dataset.saved === '1'){
      const F = getActive();
      const f = from.value.trim().toLowerCase();
      const r = type.value.trim().toUpperCase();
      const t = to.value.trim().toLowerCase();
      const key = `${f}|${t}|${r}`;
      if(F?.edgeByKey?.[key]){
        F.removeEdge(key);
        saveField(F.id);
        _onEdgeChanged?.();
      }
    }
    row.remove();
  };
}

// ---- Modifica dimensioni -------------------------------------------------

export function openEditDims(word){
  const F = getActive();
  const w = F.wordMap[word];
  if(!w) return;
  const initialSig = (w.sig || NEUTRAL_SIG).slice();

  openPanel({
    title: `Dimensioni: ${esc(word)}`,
    build: (body) => {
      const host = document.createElement('div');
      body.appendChild(host);
      _activeDimEditor = new DimEditor(host, { sig: initialSig });
    },
    actions: [
      { label: 'applica', primary: true, onClick: () => {
          const newSig = _activeDimEditor.getSig();
          F.updateWordSig(word, newSig);
          applyEditFlagsToWord(w);
          saveField(F.id);
          closeEdit();
          _onWordChanged?.(word);
        } },
      { label: 'annulla', onClick: closeEdit },
    ],
    onClose: () => { _activeDimEditor?.destroy(); _activeDimEditor = null; },
  });
}

// ---- Modifica relazioni della parola -------------------------------------

export function openEditRels(word){
  const F = getActive();
  if(!F.wordMap[word]) return;
  const rels = F.edgesForWord(word);
  const relTypes = Object.keys(RL);

  openPanel({
    title: `Relazioni: ${esc(word)}`,
    build: (body) => {
      let html = '<div class="rels-list">';
      if(!rels.length) html += '<p class="empty-rels">nessuna relazione</p>';
      rels.forEach((e, idx) => {
        const other = e.from === word ? e.to : e.from;
        const dir = e.from === word ? '→' : '←';
        html += `<div class="edit-row rel-edit-row">`
              + `<span class="rel-dir">${dir}</span>`
              + `<span class="rel-other">${esc(other)}</span>`
              + `<select data-eidx="${idx}" class="edit-select">${relTypes.map(r => `<option value="${r}"${e.rel === r ? ' selected' : ''}>${RL[r]}</option>`).join('')}</select>`
              + `<input type="text" data-vidx="${idx}" class="edit-input rel-via-input" value="${esc(e.via || '')}" placeholder="tramite (opzionale)">`
              + `<input type="range" min="10" max="100" value="${e.conf || 50}" data-cidx="${idx}">`
              + `<span class="edit-val" id="rv_${idx}">${e.conf || 50}%</span>`
              + `<button class="delete-rel" data-key="${esc(e.key)}">✕</button>`
              + `</div>`;
      });
      html += `</div><div class="edit-section-title">aggiungi nuova</div>`;
      html += `<div class="edit-row rel-edit-row">`
            + `<input id="newRelWord" class="edit-input" placeholder="parola">`
            + `<select id="newRelType" class="edit-select">${relTypes.map(r => `<option value="${r}">${RL[r]}</option>`).join('')}</select>`
            + `<input id="newRelVia" class="edit-input rel-via-input" placeholder="tramite">`
            + `<button class="edit-btn" id="addRelBtn">+</button>`
            + `</div>`;
      body.innerHTML = html;

      body.querySelectorAll('input[type=range][data-cidx]').forEach(sl => {
        sl.oninput = function(){
          document.getElementById('rv_' + this.dataset.cidx).textContent = this.value + '%';
        };
      });
      body.querySelectorAll('.delete-rel').forEach(btn => {
        btn.onclick = () => {
          F.removeEdge(btn.dataset.key);
          saveField(F.id);
          _onEdgeChanged?.();
          openEditRels(word);
        };
      });
      body.querySelector('#addRelBtn').onclick = () => {
        const target = body.querySelector('#newRelWord').value.trim().toLowerCase();
        const rel = body.querySelector('#newRelType').value;
        const via = (body.querySelector('#newRelVia').value || '').trim().toLowerCase() || null;
        if(!target || target === word) return;
        if(!F.hasWord(target)){
          F.addWord({
            w: target, sig: NEUTRAL_SIG.slice(),
            flags: flagsForNewWord(F),
          });
        }
        F.addEdge({ from: word, to: target, rel, conf: 80, via, flags: flagsForNewEdge(F) });
        saveField(F.id);
        _onEdgeChanged?.();
        openEditRels(word);
      };
    },
    actions: [
      { label: 'applica', primary: true, onClick: () => {
          const rels2 = F.edgesForWord(word);
          document.querySelectorAll('#editPanel select[data-eidx]').forEach(sel => {
            const idx = parseInt(sel.dataset.eidx);
            if(rels2[idx]) rels2[idx].rel = sel.value;
          });
          document.querySelectorAll('#editPanel input[data-cidx]').forEach(rng => {
            const idx = parseInt(rng.dataset.cidx);
            if(rels2[idx]) rels2[idx].conf = parseInt(rng.value);
          });
          document.querySelectorAll('#editPanel input[data-vidx]').forEach(inp => {
            const idx = parseInt(inp.dataset.vidx);
            if(rels2[idx]){
              const v = (inp.value || '').trim().toLowerCase();
              rels2[idx].via = v || null;
            }
          });
          saveField(F.id);
          closeEdit();
          _onWordChanged?.(word);
        } },
      { label: 'chiudi', onClick: closeEdit },
    ],
  });
}

// ---- Quick edit di un singolo arco ---------------------------------------

export function openQuickEdge(edgeKey){
  const F = getActive();
  const edge = F.edgeByKey[edgeKey];
  if(!edge) return;
  const relTypes = Object.keys(RL);
  const fromCol = F.wordMap[edge.from] ? colorForSig(F.wordMap[edge.from].sig) : UI.edgeFallback;
  const toCol   = F.wordMap[edge.to]   ? colorForSig(F.wordMap[edge.to].sig)   : UI.edgeFallback;

  openPanel({
    title: 'modifica relazione',
    build: (body) => {
      body.innerHTML = `
        <div class="rel-header">
          <span style="color:${fromCol}">${esc(edge.from)}</span>
          <span class="rel-arrow">→</span>
          <span style="color:${toCol}">${esc(edge.to)}</span>
        </div>
        <div class="edit-row"><label>tipo</label><select id="qeType" class="edit-select">${relTypes.map(r => `<option value="${r}"${edge.rel === r ? ' selected' : ''}>${RL[r]}</option>`).join('')}</select></div>
        <div class="edit-row"><label>tramite</label><input type="text" id="qeVia" class="edit-input rel-via-input" value="${esc(edge.via || '')}" placeholder="parola che fa da ponte"></div>
        <div class="edit-row"><label>forza</label><input type="range" id="qeConf" min="10" max="100" value="${edge.conf || 80}"><span class="edit-val">${edge.conf || 80}</span></div>
      `;
      body.querySelector('#qeConf').oninput = function(){ this.nextElementSibling.textContent = this.value; };
    },
    actions: [
      { label: 'applica', primary: true, onClick: () => {
          const newRel = document.getElementById('qeType').value;
          const newConf = parseInt(document.getElementById('qeConf').value);
          const newVia = (document.getElementById('qeVia').value || '').trim().toLowerCase() || null;
          if(newRel !== edge.rel){
            F.removeEdge(edgeKey);
            F.addEdge({
              from: edge.from, to: edge.to, rel: newRel, conf: newConf, via: newVia,
              flags: flagsForNewEdge(F),
            });
          } else {
            edge.conf = newConf;
            edge.via = newVia;
            applyEditFlagsToEdge(edge);
            const width = Math.max(0.5, (newConf / 100) * 1.6);
            F.edgesDS.update({ id: edgeKey, width });
          }
          saveField(F.id);
          closeEdit();
          _onWordChanged?.(edge.from);
        } },
      { label: 'elimina', danger: true, onClick: () => {
          F.removeEdge(edgeKey);
          saveField(F.id);
          closeEdit();
          _onWordChanged?.(edge.from);
        } },
      { label: 'annulla', onClick: closeEdit },
    ],
  });
}

// ---- Drag-to-connect: chiede tipo relazione + forza ----------------------

export function openConnect(from, to){
  const F = getActive();
  if(from === to) return;
  // TUTTE le 21 relazioni disponibili nel drag-to-connect (non più solo le 12 di
  // RL): la gente le ha tutte sotto mano e sceglie. Le etichette complete vivono
  // in REL_LEGEND_LABEL_IT (constants.js), ordinate per famiglia.
  const LBL = REL_LEGEND_LABEL_IT;
  const relTypes = Object.keys(LBL);
  const fromCol = F.wordMap[from] ? colorForSig(F.wordMap[from].sig) : UI.edgeFallback;
  const toCol   = F.wordMap[to]   ? colorForSig(F.wordMap[to].sig)   : UI.edgeFallback;

  // Risolve l'input dell'utente al codice canonico KG. Permette:
  // identificatore ("IS_A", "is_a"), etichetta italiana ("è un"),
  // o un alias nuovo (uppercase verbatim, validato dal server).
  const resolveRel = (raw) => {
    const s = (raw || '').trim();
    if(!s) return '';
    const up = s.toUpperCase();
    if(LBL[up]) return up;
    const norm = s.toLowerCase();
    for(const k of relTypes){
      if(LBL[k].toLowerCase() === norm) return k;
    }
    return up;
  };

  openPanel({
    title: 'collega parole',
    build: (body) => {
      // Etichetta italiana come value: il datalist filtra in base a quella
      // ("è" → "è un", "ca" → "causa"). Più amichevole per chi non conosce
      // gli identificatori KG. resolveRel rimappa al codice prima di salvare.
      body.innerHTML = `
        <datalist id="connectRelList">${
          relTypes.map(r => `<option value="${esc(LBL[r])}"></option>`).join('')
        }</datalist>
        <div class="rel-header">
          <span style="color:${fromCol}">${esc(from)}</span>
          <span class="rel-arrow">→</span>
          <span style="color:${toCol}">${esc(to)}</span>
        </div>
        <div class="edit-row">
          <label>tipo</label>
          <input type="text" id="cType" class="edit-input" list="connectRelList"
                 value="${esc(LBL[relTypes[0]])}" autocomplete="off"
                 placeholder="scrivi o scorri con la rotellina">
        </div>
        <div class="edit-row">
          <label>tramite</label>
          <input type="text" id="cVia" class="edit-input" autocomplete="off"
                 placeholder="parola che fa da ponte (opzionale)">
        </div>
        <div class="edit-row">
          <label>forza</label>
          <input type="range" id="cConf" min="10" max="100" value="80">
          <span class="edit-val">80</span>
        </div>
      `;
      const typeInput = body.querySelector('#cType');
      const confInput = body.querySelector('#cConf');
      confInput.oninput = function(){ this.nextElementSibling.textContent = this.value; };

      // Rotellina sull'input tipo: cicla nella lista delle relazioni note
      // (etichette italiane) senza scrollare il pannello. Se l'utente ha
      // digitato un alias non in RL, il ciclo riparte dall'inizio.
      const labels = relTypes.map(r => LBL[r]);
      typeInput.addEventListener('wheel', (e) => {
        e.preventDefault();
        const cur = typeInput.value.trim().toLowerCase();
        let idx = labels.findIndex(l => l.toLowerCase() === cur);
        if(idx < 0) idx = e.deltaY > 0 ? 0 : labels.length - 1;
        else idx = (idx + (e.deltaY > 0 ? 1 : -1) + labels.length) % labels.length;
        typeInput.value = labels[idx];
      }, { passive: false });

      setTimeout(() => { typeInput.focus(); typeInput.select(); }, 50);
    },
    actions: [
      { label: 'collega', primary: true, onClick: () => {
          const rel = resolveRel(document.getElementById('cType').value);
          const via = (document.getElementById('cVia').value || '').trim().toLowerCase();
          const conf = parseInt(document.getElementById('cConf').value);
          if(!rel) return;
          F.addEdge({ from, to, rel, conf, via: via || null, flags: flagsForNewEdge(F) });
          saveField(F.id);
          _onEdgeChanged?.();
          closeEdit();
          // Esce da linkMode (la connessione è fatta) ma lascia visibile
          // tutto il campo — l'utente vuole continuare a lavorare nel
          // contesto, non in una vista isolata.
          setLinkMode(false);
        } },
      { label: 'annulla', onClick: closeEdit },
    ],
  });
}

// ---- Context menu su un nodo --------------------------------------------

export function openNodeCtxMenu(nodeId, mx, my){
  const F = getActive();
  const w = F.wordMap[nodeId];
  const col = w ? colorForSig(w.sig) : UI.textDim;
  const tagUnknown = w?.flags?.unknown ? ' <span class="unknown-badge">sconosciuta</span>' : '';
  const items = [
    { kind: 'title', html: `${esc(nodeId)}${tagUnknown}`, color: col },
    { kind: 'sep' },
  ];
  if(w?.flags?.unknown){
    items.push({ kind: 'item', label: 'descrivi parola', action: 'describe', onClick: () => openEditDims(nodeId) });
  } else {
    items.push({ kind: 'item', label: 'modifica dimensioni', action: 'edit-dims', onClick: () => openEditDims(nodeId) });
  }
  items.push(
    { kind: 'item', label: 'modifica relazioni', action: 'edit-rels', onClick: () => openEditRels(nodeId) },
    { kind: 'item', label: '+ aggiungi relazione', action: 'add-rel', onClick: () => openAddRel(nodeId) },
  );
  // Estrazione relazioni dal KG: solo nel personale.
  if(F.id === 'nuovo'){
    items.push({
      kind: 'item', label: '⤓ estrai relazioni dal KG', action: 'extract-rel',
      onClick: () => openExtractDialog({ kind: 'word', word: nodeId }, async (allowedGroups) => {
        const F2 = getActive();
        // Snapshot: parole già nel campo PRIMA dell'estrazione, così
        // possiamo identificare le nuove e clusterizzarle attorno alla
        // parola satellite — invece di lasciarle finire in posizioni
        // 8D arbitrarie (placeByRank) lontane dal contesto.
        const before = new Set(F2.words.map(w => w.w));
        await extractRelationsForWord(F2, nodeId, { allowedGroups });
        const newWords = F2.words.map(w => w.w).filter(id => !before.has(id));
        if(newWords.length){
          // Cluster anulare attorno a nodeId: anelli concentrici per
          // accomodare anche estrazioni grandi (30+ parole) senza
          // sovrapposizioni. Le nuove sono _userPositioned così il
          // layout rectangular/dimensional non le sparpaglia.
          F2.clusterAroundWord(nodeId, newWords);
        }
        // spreadNonOverlapping con minDist generoso per separare le nuove
        // dalle parole pre-esistenti che potrebbero essere finite vicino.
        // Iterazioni alte per stabilizzare campi gonfi post-extract: senza
        // abbastanza iterazioni, i rettangoli del rectangular si sovrappongono
        // e l'hover triggera oscillazioni visive.
        F2.spreadNonOverlapping({ minDist: 100, iterations: 150 });
        saveField(F2.id);
        // L'estrazione ha aggiunto archi e parole: serve riapplicare il
        // layout corrente perché in modalità rectangular i nuovi nodi
        // appaiono come puntini in posizioni 8D arbitrarie finché non
        // vengono ricostruiti come rettangoli con la spec layoutMode
        // corretta. _onEdgeChanged riapplica applyNuovoLayout + refresh.
        _onEdgeChanged?.();
        // Re-seleziona per ricalcolare la rosa dei vicini con i nuovi archi.
        if(F2.selected === nodeId){ F2.selected = null; _onSelect?.(nodeId); }
        else _onWordChanged?.(nodeId);
      }),
    });
  }
  items.push(
    { kind: 'sep' },
    { kind: 'item', label: 'elimina parola', action: 'delete', danger: true, onClick: () => {
        const F2 = getActive();
        if(F2.selected === nodeId) _onDeselect?.();
        F2.removeWord(nodeId);
        saveField(F2.id);
      } },
  );
  openCtxMenu({ x: mx, y: my, items });
}

// ---- Context menu su un arco --------------------------------------------

export function openEdgeCtxMenu(edgeId, mx, my){
  if(!edgeId){ closeCtxMenu(); return; }
  const F = getActive();
  const edge = F.edgeByKey[edgeId];
  if(!edge) return;
  const relLabel = RL[edge.rel] || edge.rel;
  const fromCol = F.wordMap[edge.from] ? colorForSig(F.wordMap[edge.from].sig) : UI.edgeFallback;
  const toCol   = F.wordMap[edge.to]   ? colorForSig(F.wordMap[edge.to].sig)   : UI.edgeFallback;

  const titleHtml = `<span style="color:${fromCol}">${esc(edge.from)}</span>`
                  + `<span style="color:${UI.textDim};margin:0 4px">${esc(relLabel)}</span>`
                  + `<span style="color:${toCol}">${esc(edge.to)}</span>`;

  openCtxMenu({
    x: mx, y: my,
    items: [
      { kind: 'title', html: titleHtml },
      { kind: 'sep' },
      { kind: 'item', label: 'modifica relazione', action: 'edit', onClick: () => openEditRels(edge.from) },
      { kind: 'item', label: 'elimina relazione', action: 'delete', danger: true, onClick: () => {
          F.removeEdge(edge.key); saveField(F.id); _onEdgeChanged?.();
        } },
    ],
  });
}

// ---- Radar Octalysis (drag per modificare i drive) ----------------------

const RADAR = {
  W: 480, H: 320, R: 100,
  angles: CD_NAMES.map((_, i) => (Math.PI * 2 / 8) * i - Math.PI / 2),
  points: [], dragIdx: -1, handlersAttached: false,
};
RADAR.cx = RADAR.W / 2;
RADAR.cy = RADAR.H / 2;

export function drawRadar(sig){
  const F = getActive();
  const w = F.selected ? F.wordMap[F.selected] : null;
  const oct = (w && w.userOct) ? w.userOct : computeOctalysis(sig);
  const canvas = document.getElementById('radar');
  if(!canvas) return;
  const ctx = canvas.getContext('2d');
  const { W, H, cx, cy, R, angles } = RADAR;
  ctx.clearRect(0, 0, W, H);

  for(let r = 0.25; r <= 1; r += 0.25){
    ctx.beginPath();
    for(let i = 0; i <= 8; i++){
      const a = angles[i % 8];
      const x = cx + Math.cos(a) * R * r, y = cy + Math.sin(a) * R * r;
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    }
    ctx.closePath();
    ctx.strokeStyle = r === 1 ? UI.radarRingOuter : UI.radarRingInner;
    ctx.lineWidth = r === 1 ? 1.5 : 1;
    ctx.stroke();
  }

  const LABEL_OFFSET = R + 24, LINE_HEIGHT = 18, FONT_SIZE = 15;
  ctx.font = `500 ${FONT_SIZE}px 'JetBrains Mono',sans-serif`;
  for(let i = 0; i < 8; i++){
    const a = angles[i], cosA = Math.cos(a), sinA = Math.sin(a);
    ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(cx + cosA * R, cy + sinA * R);
    ctx.strokeStyle = UI.radarSpokes; ctx.stroke();
    ctx.textAlign = cosA > 0.3 ? 'left' : cosA < -0.3 ? 'right' : 'center';
    ctx.textBaseline = 'top';
    ctx.fillStyle = CD_COLORS[i];
    const lx = cx + cosA * LABEL_OFFSET, ly = cy + sinA * LABEL_OFFSET;
    const lines = CD_NAMES[i].split(' ');
    const blockH = (lines.length - 1) * LINE_HEIGHT + FONT_SIZE;
    let yStart;
    if(sinA >  0.3) yStart = ly;
    else if(sinA < -0.3) yStart = ly - blockH;
    else yStart = ly - blockH / 2;
    lines.forEach((line, k) => { ctx.fillText(line, lx, yStart + k * LINE_HEIGHT); });
  }

  ctx.beginPath();
  for(let i = 0; i < 8; i++){
    const a = angles[i], v = Math.min(1, oct[i]);
    const x = cx + Math.cos(a) * R * v, y = cy + Math.sin(a) * R * v;
    i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
  }
  ctx.closePath();
  ctx.fillStyle = UI.radarFill; ctx.fill();
  ctx.shadowColor = UI.radarGlow; ctx.shadowBlur = 8;
  ctx.strokeStyle = UI.radarStroke; ctx.lineWidth = 3; ctx.stroke();
  ctx.shadowBlur = 0;

  RADAR.points = [];
  for(let i = 0; i < 8; i++){
    const a = angles[i], v = Math.min(1, oct[i]);
    const px = cx + Math.cos(a) * R * v, py = cy + Math.sin(a) * R * v;
    RADAR.points.push({ i, px, py, v, a });
    ctx.beginPath(); ctx.arc(px, py, 3.5, 0, Math.PI * 2);
    ctx.fillStyle = CD_COLORS[i]; ctx.fill();
    ctx.strokeStyle = UI.textBright; ctx.lineWidth = 1; ctx.stroke();
    ctx.font = "600 12px 'JetBrains Mono', monospace";
    ctx.textAlign = Math.cos(a) > 0.3 ? 'left' : Math.cos(a) < -0.3 ? 'right' : 'center';
    ctx.textBaseline = Math.sin(a) > 0.3 ? 'top' : 'bottom';
    ctx.fillStyle = UI.textBright;
    ctx.fillText(Math.round(v * 100) + '%',
      px + (Math.cos(a) > 0.3 ? 10 : Math.cos(a) < -0.3 ? -10 : 0),
      py + (Math.sin(a) > 0.3 ? 10 : -10));
  }

  if(!RADAR.handlersAttached) attachRadarHandlers(canvas);
}

function attachRadarHandlers(canvas){
  function canvasXY(e){
    const rect = canvas.getBoundingClientRect();
    const scaleX = RADAR.W / rect.width, scaleY = RADAR.H / rect.height;
    return { x: (e.clientX - rect.left) * scaleX, y: (e.clientY - rect.top) * scaleY };
  }
  canvas.addEventListener('mousedown', e => {
    const F = getActive();
    if(!F.selected) return;
    const p = canvasXY(e);
    for(const pt of RADAR.points){
      if(Math.hypot(p.x - pt.px, p.y - pt.py) < 14){
        RADAR.dragIdx = pt.i;
        canvas.style.cursor = 'grabbing';
        e.preventDefault();
        break;
      }
    }
  });
  canvas.addEventListener('mousemove', e => {
    const F = getActive();
    if(RADAR.dragIdx < 0 || !F.selected) return;
    const p = canvasXY(e);
    const a = RADAR.angles[RADAR.dragIdx];
    const dx = p.x - RADAR.cx, dy = p.y - RADAR.cy;
    const proj = (dx * Math.cos(a) + dy * Math.sin(a)) / RADAR.R;
    const newV = Math.max(0, Math.min(1, proj));
    const w = F.wordMap[F.selected]; if(!w) return;
    if(!w.userOct) w.userOct = computeOctalysis(w.sig || NEUTRAL_SIG).slice();
    w.userOct[RADAR.dragIdx] = newV;
    drawRadar(w.sig);
  });
  function endDrag(){
    const F = getActive();
    if(RADAR.dragIdx >= 0 && F.selected){
      const w = F.wordMap[F.selected];
      if(w?.userOct && F.id !== 'vasto') saveField(F.id);
    }
    RADAR.dragIdx = -1;
    canvas.style.cursor = 'default';
  }
  canvas.addEventListener('mouseup', endDrag);
  canvas.addEventListener('mouseleave', endDrag);
  window.addEventListener('mouseup', endDrag);
  RADAR.handlersAttached = true;
}
