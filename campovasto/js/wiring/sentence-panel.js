// Pannello "crea il tuo campo da frase" + pannello "UI-R1 legge".
// Callback iniettato da view-switcher per evitare dipendenze circolari.

import { REL_LABEL_IT } from '../constants.js';
import { UI } from '../theme.js';
import { FIELDS } from '../manager.js';
import { buildNuovo } from '../sentence.js';
import { esc } from '../geometry.js';

let _onNuovoCreated = null;  // () => void — iniettato da view-switcher
export function setSentencePanelCallbacks({ onNuovoCreated }){
  _onNuovoCreated = onNuovoCreated;
}

export function openSentencePanel(){
  const p = document.getElementById('sentencePanel');
  if(!p) return;
  const hasPersonale = FIELDS.nuovo && (FIELDS.nuovo.words.length > 0 || FIELDS.nuovo.sentence);
  p.innerHTML = `
    <div class="intesta">
      <h3>crea il tuo campo da frase</h3>
      <button class="chiudi" id="sentenceClose" title="chiudi">×</button>
    </div>
    <textarea id="sentenceInput" rows="4" placeholder="scrivi una frase..."></textarea>
    <div class="azioni">
      <button class="edit-btn primary" id="sentenceBuild">crea</button>
      ${hasPersonale ? '<button class="edit-btn" id="sentenceClear">azzera campo</button>' : ''}
    </div>
    <div class="sentence-hint">Tutte le parole della frase diventano punti nel campo: quelle senza relazioni nel KG prendono una firma neutra.</div>
  `;
  p.style.display = '';
  setTimeout(() => document.getElementById('sentenceInput')?.focus(), 50);
  document.getElementById('sentenceClose').onclick = () => p.style.display = 'none';
  document.getElementById('sentenceBuild').onclick = async () => {
    const text = document.getElementById('sentenceInput').value.trim();
    if(!text) return;
    const btn = document.getElementById('sentenceBuild');
    btn.disabled = true; btn.textContent = 'ricerca in corso…';
    try {
      await buildNuovo(text);
    } catch(e){
      console.error('buildNuovo:', e);
      btn.disabled = false; btn.textContent = 'crea';
      return;
    }
    p.style.display = 'none';
    _onNuovoCreated?.();
    fetchUnderstandingAndRender(text);
  };
  const clrBtn = document.getElementById('sentenceClear');
  if(clrBtn){
    clrBtn.onclick = () => {
      window.dispatchEvent(new CustomEvent('uir1:clear-nuovo'));
      p.style.display = 'none';
    };
  }
}

// ---- Pannello "UI-R1 legge" -------------------------------------------------

const _hypothesisStatus = new Map();

export async function fetchUnderstandingAndRender(sentence){
  if(!sentence || sentence.trim().length < 2){ hideUnderstandingPanel(); return; }
  try {
    const r = await fetch('/api/understanding?sentence=' + encodeURIComponent(sentence));
    if(!r.ok){ hideUnderstandingPanel(); return; }
    const data = await r.json();
    renderUnderstandingPanel(sentence, data);
  } catch(e){ console.warn('[understanding] fetch fallito', e); }
}

export function hideUnderstandingPanel(){
  const p = document.getElementById('understandingPanel');
  if(p) p.classList.add('empty');
}

function renderUnderstandingPanel(sentence, u){
  const panel = document.getElementById('understandingPanel');
  const body = document.getElementById('upBody');
  const header = document.getElementById('upHeader');
  const snippet = document.getElementById('upHeaderSnippet');
  const badgeDepth = document.getElementById('upBadgeDepth');
  const badgeHyp = document.getElementById('upBadgeHyp');
  if(!panel || !body || !header) return;
  const parts = [];

  if(u.summary && u.summary.trim()){
    parts.push(`<div class="up-section">
      <div class="up-section-title">sunto</div>
      <div class="up-summary">${esc(u.summary)}</div>
    </div>`);
  }

  if(u.proposed_edges && u.proposed_edges.length){
    const items = u.proposed_edges.map(p => {
      const status = _hypothesisStatus.get(p.id) || p.status || 'pending';
      const statusClass = status === 'confirmed' ? 'confirmed'
                        : status === 'rejected' ? 'rejected' : 'pending';
      const buttons = status === 'pending'
        ? `<button class="up-btn confirm" data-id="${esc(p.id)}"
             data-s="${esc(p.subject)}" data-r="${esc(p.relation)}"
             data-o="${esc(p.object)}" data-c="${p.confidence.toFixed(2)}">✓ conferma</button>
           <button class="up-btn reject" data-id="${esc(p.id)}"
             data-s="${esc(p.subject)}" data-r="${esc(p.relation)}"
             data-o="${esc(p.object)}">✗ rifiuta</button>`
        : status === 'confirmed'
          ? `<span class="up-btn-done confirmed">✓ aggiunto al KG</span>`
          : `<span class="up-btn-done rejected">✗ rifiutato</span>`;
      return `<div class="up-hyp-row ${statusClass}" data-id="${esc(p.id)}">
        <div class="up-hyp-claim">
          <span class="up-hyp-word">${esc(p.subject)}</span>
          <span class="up-hyp-rel">${esc(p.relation_label || REL_LABEL_IT[p.relation] || p.relation)}</span>
          <span class="up-hyp-word">${esc(p.object)}</span>
          <span class="up-conf">${p.confidence.toFixed(2)}</span>
        </div>
        <div class="up-hyp-rationale">${esc(p.rationale)}</div>
        <div class="up-hyp-actions">${buttons}</div>
      </div>`;
    }).join('');
    parts.push(`<div class="up-section">
      <div class="up-section-title">ipotesi — relazioni non tracciate</div>
      ${items}
    </div>`);
  }

  if(u.unknown_words && u.unknown_words.length){
    parts.push(`<div class="up-section">
      <div class="up-section-title">parole ignote al KG</div>
      <div class="up-unknown">${u.unknown_words.map(esc).join(', ')}</div>
    </div>`);
  }

  parts.push(`<div class="up-section">
    <div class="up-section-title">diagnostica</div>
    <div style="font-size:10px;color:${UI.textDim};font-family:'JetBrains Mono', monospace">
      ruolo: ${esc((u.syntactic_role || '—').toLowerCase())} · ${u.comprehension_depth || 0} archi · lemmi: ${(u.lemmas || []).map(esc).join(', ')}
    </div>
  </div>`);

  body.innerHTML = parts.join('');
  if(snippet) snippet.textContent = '"' + sentence + '"';
  if(badgeDepth) badgeDepth.textContent = (u.comprehension_depth || 0) + ' archi';
  const pending = (u.proposed_edges || []).filter(p => (_hypothesisStatus.get(p.id) || 'pending') === 'pending').length;
  if(badgeHyp){
    if(pending > 0){ badgeHyp.textContent = pending + ' ipotesi'; badgeHyp.style.display = ''; }
    else badgeHyp.style.display = 'none';
  }
  panel.classList.remove('empty');
  panel.classList.add('collapsed');

  if(!header._bound){
    header._bound = true;
    header.addEventListener('click', () => panel.classList.toggle('collapsed'));
  }

  body.querySelectorAll('.up-btn.confirm').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        const r = await fetch('/api/kg/confirm_edge', {
          method: 'POST', headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            subject: btn.dataset.s, relation: btn.dataset.r, object: btn.dataset.o,
            confidence: parseFloat(btn.dataset.c || '0.7'),
          }),
        });
        const d = await r.json();
        _hypothesisStatus.set(btn.dataset.id, d.ok ? 'confirmed' : 'pending');
      } catch(err){ console.warn(err); }
      renderUnderstandingPanel(sentence, u);
    });
  });
  body.querySelectorAll('.up-btn.reject').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        await fetch('/api/kg/reject_edge', {
          method: 'POST', headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ subject: btn.dataset.s, relation: btn.dataset.r, object: btn.dataset.o }),
        });
        _hypothesisStatus.set(btn.dataset.id, 'rejected');
      } catch(err){ console.warn(err); }
      renderUnderstandingPanel(sentence, u);
    });
  });
}
