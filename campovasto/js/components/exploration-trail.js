// Trail di esplorazione — pannello in alto a destra del grafo che mostra la
// catena di parole che l'utente ha cliccato, con la relazione (e l'eventuale
// "tramite") che lega ogni passo al successivo. Sull'hover di un vicino della
// parola corrente, una preview semi-trasparente anticipa il prossimo passo.
//
// È un puro renderer: legge `F.navPath` + `F.selected` (già mantenuti da
// selection.js) + una parola hover passata esplicitamente. Nessuno stato
// duplicato. Click su un elemento del trail → seleziona quella parola
// (selectWord truncia automaticamente il navPath fino a quel punto).
//
// Vedi CLAUDE.md §1 (no hex literals) e §3 (un file, una responsabilità).

import { RL } from '../constants.js';
import { colorForSig, UI } from '../theme.js';
import { getActive } from '../manager.js';
import { esc } from '../geometry.js';

let _root = null;
let _onWordPick = null;
let _hoverWord = null;

export function initTrail(graphContainer, opts = {}){
  if(_root) return;
  _root = document.createElement('div');
  _root.id = 'explorationTrail';
  _root.className = 'exploration-trail';
  _root.style.display = 'none';
  graphContainer.appendChild(_root);
  _onWordPick = opts.onWordPick || null;
}

// Aggiorna la parola in hover e ridisegna. Passa null per togliere la preview.
export function setTrailHover(word){
  _hoverWord = word || null;
  renderTrail();
}

// Renderer: chiamato a ogni cambio di selezione/hover/campo.
export function renderTrail(){
  if(!_root) return;
  const F = getActive();
  if(!F || !F.selected){
    _root.style.display = 'none';
    _root.innerHTML = '';
    return;
  }

  const path = [...(F.navPath || []), F.selected];

  // Preview valida solo se l'hover è su una parola diversa dalla testa
  // ED è collegata da un arco diretto (in qualsiasi direzione).
  let previewStep = null;
  if(_hoverWord && _hoverWord !== F.selected && !path.includes(_hoverWord)){
    const step = _findEdgeStep(F, F.selected, _hoverWord);
    if(step) previewStep = { word: _hoverWord, step };
  }

  const parts = [];
  for(let i = 0; i < path.length; i++){
    if(i > 0){
      const step = _findEdgeStep(F, path[i - 1], path[i]);
      parts.push(_renderStep(step, false));
    }
    parts.push(_renderWord(F, path[i], i, false));
  }
  if(previewStep){
    parts.push(_renderStep(previewStep.step, true));
    parts.push(_renderWord(F, previewStep.word, path.length, true));
  }

  parts.push(`<button type="button" class="trail-close" title="azzera trail">×</button>`);

  _root.innerHTML = parts.join('');
  _root.style.display = 'flex';

  // Wire click sui pillole-parola (solo le cristallizzate, non la preview).
  _root.querySelectorAll('.trail-word:not(.preview)').forEach(el => {
    el.addEventListener('click', () => {
      const w = el.dataset.word;
      if(w) _onWordPick?.(w);
    });
  });
  _root.querySelector('.trail-close').addEventListener('click', () => {
    // Deseleziona tutto: l'effetto visivo è "azzera la catena". La selezione
    // non è strettamente del trail — toglierla è il modo coerente di pulire.
    _onWordPick?.(null);
  });
}

// Cerca un arco diretto fra `a` e `b` (qualsiasi direzione). Restituisce
// { rel, via, direction } dove direction = 'out' se a→b, 'in' se b→a.
function _findEdgeStep(F, a, b){
  const keys = F.edgesByWord?.[a];
  if(!keys) return null;
  for(const key of keys){
    const e = F.edgeByKey[key];
    if(!e) continue;
    if(e.from === a && e.to === b){
      return { rel: e.rel, via: e.via || null, direction: 'out' };
    }
    if(e.from === b && e.to === a){
      return { rel: e.rel, via: e.via || null, direction: 'in' };
    }
  }
  return null;
}

function _renderWord(F, word, idx, isPreview){
  const w = F.wordMap?.[word];
  const col = w ? colorForSig(w.sig) : UI.textDim;
  const cls = `trail-word${isPreview ? ' preview' : ''}`;
  return `<span class="${cls}" data-idx="${idx}" data-word="${esc(word)}" `
       + `style="border-left-color:${col}">${esc(word)}</span>`;
}

function _renderStep(step, isPreview){
  if(!step){
    // Niente arco diretto fra due parole consecutive del navPath: succede in
    // 'nuovo' (il path accumula i click anche senza arco). Mostriamo un
    // separatore neutro.
    return `<span class="trail-step empty${isPreview ? ' preview' : ''}">›</span>`;
  }
  const lab = RL[step.rel] || step.rel;
  const arrow = step.direction === 'in' ? '←' : '→';
  const cls = `trail-step${isPreview ? ' preview' : ''}`;
  let html = `<span class="${cls}">`
           + `<span class="trail-arrow">${arrow}</span>`
           + `<span class="trail-rel">${esc(lab)}</span>`;
  if(step.via){
    html += `<span class="trail-via">tramite <em>${esc(step.via)}</em></span>`;
  }
  html += `</span>`;
  return html;
}
