// Dialog di estrazione/cancellazione relazioni filtrate per gruppo.
// Mostra le 5 famiglie di relazioni (REL_GROUP) come checkbox: l'utente
// sceglie i gruppi su cui agire e conferma.
//
// scope: { kind: 'word', word: 'cane' } | { kind: 'field', label?: '...' }
// mode: 'extract' (default) — estrae dal KG verso il field
//       'delete' — rimuove dal field gli archi dei gruppi selezionati
// onConfirm(allowedGroups: Set<string>) → callback async che esegue l'azione.

import { openPanel, closePanel } from './edit-panel.js';
import { GROUP_LABELS, ALL_GROUPS } from '../relations-extract.js';
import { esc } from '../geometry.js';

export function openExtractDialog(scope, onConfirm, opts = {}){
  const mode = opts.mode || 'extract';
  const verb = mode === 'delete' ? 'Cancella' : 'Estrai';
  const verbLower = mode === 'delete' ? 'cancella' : 'estrai';
  const verbProgress = mode === 'delete' ? 'cancello…' : 'estraggo…';
  const hint = mode === 'delete'
    ? 'Verranno rimossi dal campo TUTTI gli archi dei tipi selezionati. I nodi che restano senza archi rimarranno nel campo.'
    : 'I target delle relazioni non ancora presenti nel campo verranno aggiunti (con la firma dal campo vasto se disponibile).';
  const title = scope.kind === 'word'
    ? `${verb} relazioni: ${esc(scope.word)}`
    : `${verb} tutte le relazioni`;

  openPanel({
    title,
    build: (body) => {
      let html = '<div class="edit-section-title">tipi di relazione</div>';
      html += '<div class="extract-groups">';
      for(const g of ALL_GROUPS){
        const { label, desc } = GROUP_LABELS[g];
        html += `<label class="extract-group-row">
          <input type="checkbox" data-group="${g}" checked>
          <span class="extract-group-label">${esc(label)}</span>
          <span class="extract-group-desc">${esc(desc)}</span>
        </label>`;
      }
      html += '</div>';
      html += `<div class="extract-actions-row">
        <button class="edit-btn small" id="extractAll">tutto</button>
        <button class="edit-btn small" id="extractNone">niente</button>
      </div>`;
      html += `<div class="sentence-hint" style="margin-top:10px;font-size:11px;opacity:0.75">${esc(hint)}</div>`;
      body.innerHTML = html;

      body.querySelector('#extractAll').onclick = () => {
        body.querySelectorAll('input[data-group]').forEach(c => c.checked = true);
      };
      body.querySelector('#extractNone').onclick = () => {
        body.querySelectorAll('input[data-group]').forEach(c => c.checked = false);
      };
    },
    actions: [
      { label: verbLower, primary: true, danger: mode === 'delete', onClick: async () => {
          const checked = new Set();
          document.querySelectorAll('#editPanel input[data-group]:checked').forEach(c => {
            checked.add(c.dataset.group);
          });
          if(!checked.size){ closePanel(); return; }
          const btn = document.querySelector('#editPanel .edit-actions .primary');
          if(btn){ btn.disabled = true; btn.textContent = verbProgress; }
          try {
            await onConfirm(checked);
          } catch(e){
            console.error('[extract-dialog]', e);
          } finally {
            closePanel();
          }
        } },
      { label: 'annulla', onClick: closePanel },
    ],
  });
}
