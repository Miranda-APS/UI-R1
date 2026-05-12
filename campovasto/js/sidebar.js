// Sidebar: card parola (.box.parola), dim-bars (slider per dimensione), breadcrumb, search,
// statistiche, mode indicators. Tutto opera sul field attivo.

import { DIM_NAMES, DIM_DESC, DIM_ICHING, NEUTRAL_SIG } from './constants.js';
import { colorForSig, dominantDim, UI } from './theme.js';
import { sigToXY, placeByRank, rankOfMag, esc } from './geometry.js';
import { getActive, activeId, saveField } from './manager.js';
import { DimEditor } from './components/dim-editor.js';
import { isReadonly, applyEditFlagsToWord } from './policies/word.js';

let _onSelectWord = null;
export function setOnSelectWord(fn){ _onSelectWord = fn; }

// ---- Dim bars (editor inline, fonte unica via DimEditor) ----

let _dimEditor = null;

export function showDimBars(sig){
  const el = document.getElementById('dim-bars');
  if(!el) return;
  _dimEditor?.destroy();
  _dimEditor = new DimEditor(el, {
    sig,
    onChange: (newSig) => applyInlineSigChange(newSig),
    onCommit: (newSig) => commitInlineSig(newSig),
    readonly: isReadonly(getActive()),
  });
}

// Anteprima live della modifica firma: aggiorna posizione + colore titolo,
// senza persistere. Persistenza avviene in commitInlineSig (mouseup/blur).
function applyInlineSigChange(sig){
  const F = getActive();
  if(!F || !F.selected) return;
  const w = F.wordMap[F.selected];
  if(!w) return;
  const title = document.getElementById('info-title');
  if(title){
    title.setAttribute('data-dim', DIM_NAMES[dominantDim(sig)]);
    title.style.setProperty('--dim-color', colorForSig(sig));
  }

  // Aggiorna posizione in tempo reale (non persiste finché non confermi).
  const p = sigToXY(sig, F.frame);
  const mag = Math.sqrt(p.x * p.x + p.y * p.y);
  const rank = rankOfMag(mag, F.words.filter(x => x.w !== w.w));
  const placed = placeByRank(sig, F.frame, rank, F.words.length);
  w.position = {
    x: placed.x, y: placed.y,
    angle: placed.angle, mag: placed.mag, normR: placed.normR,
  };
  F.nodesDS.update({ id: w.w, x: placed.x, y: placed.y });
}

function commitInlineSig(sig){
  const F = getActive();
  if(!F || !F.selected || isReadonly(F)) return;
  F.updateWordSig(F.selected, sig);
  applyEditFlagsToWord(F.wordMap[F.selected]);
  saveField(F.id);
}

// ---- Breadcrumb ----

export function renderBreadcrumb(){
  const el = document.getElementById('breadcrumb');
  if(!el) return;
  el.innerHTML = '';
  const F = getActive();
  if(!F.selected) return;
  const items = [...F.navPath, F.selected];
  items.forEach((w, idx) => {
    if(idx > 0){
      const sep = document.createElement('span');
      sep.className = 'breadcrumb-sep';
      sep.textContent = '›';
      el.appendChild(sep);
    }
    const isCurrent = idx === items.length - 1;
    const item = document.createElement(isCurrent ? 'span' : 'a');
    item.className = 'breadcrumb-item' + (isCurrent ? ' current' : '');
    item.textContent = w;
    const ww = F.wordMap[w];
    if(ww){
      item.style.borderLeft = `3px solid ${colorForSig(ww.sig)}`;
      item.style.paddingLeft = '9px';
    }
    if(!isCurrent){
      item.href = '#';
      item.addEventListener('click', e => { e.preventDefault(); _onSelectWord?.(w); });
    }
    el.appendChild(item);
  });
}

// ---- Card parola (#info-parola) ----
// La section #info-parola è gemella di #dimensioni e #spinte: tutte e tre
// vivono in #sidebar e si mostrano insieme via body.parola-selezionata
// (vedi regole di design.md §5 + style.css). Qui solo il populamento dei
// campi e il toggle dello stato globale.

export function showInfo(word){
  const F = getActive();
  const w = F.wordMap[word];
  const sig = (w && w.sig && w.sig.length >= 8) ? w.sig : NEUTRAL_SIG;
  const col = colorForSig(sig);

  const title = document.getElementById('info-title');
  const campo = document.getElementById('info-campo');
  if(!title || !campo) return;

  const unknown = w?.flags?.unknown
    ? ' <span class="unknown-badge" title="Parola sconosciuta — clicca qui per descriverla">?</span>'
    : '';
  title.innerHTML = esc(word) + unknown;
  title.setAttribute('data-dim', DIM_NAMES[dominantDim(sig)]);
  title.style.setProperty('--dim-color', col);

  // Badge "dal vasto" / "dal nuovo" / "dal medio"
  campo.textContent = `dal ${F.id}`;

  document.body.classList.add('parola-selezionata');
}

export function clearInfo(){
  const F = getActive();
  const empty = document.getElementById('info-empty');
  document.body.classList.remove('parola-selezionata');

  if(empty){
    if(F && F.id === 'nuovo' && F.sentence){
      empty.innerHTML = `<div class="medio-sentence">
        <div class="medio-sentence-label">frase del campo</div>
        <div class="medio-sentence-text">"${esc(F.sentence)}"</div>
      </div>`;
    } else {
      empty.innerHTML = '<span class="word-empty">Cerca o clicca una parola</span>';
    }
  }
}

// ---- Statistiche ----

export function updateStats(){
  const stats = document.getElementById('stats');
  if(!stats) return;
  const F = getActive();
  const visibleNodes = F.nodesDS.get().filter(n => !n.hidden && !String(n.id).startsWith('_')).length;
  const visibleEdges = F.edgesDS.get().filter(e => !e.hidden).length;
  const label = F.id === 'vasto' ? 'campo vasto' : 'campo nuovo';
  let text = `<strong>${label}</strong> · ${visibleNodes} parole · ${visibleEdges} relazioni`;
  if(F.id !== 'vasto'){
    const pending = F.words.filter(w => !w.flags.transmitted && !w.flags.unknown).length
                  + F.edges.filter(e => !e.flags.transmitted).length;
    if(pending) text += ` · <em>${pending} da trasmettere</em>`;
    const unknown = F.words.filter(w => w.flags.unknown).length;
    if(unknown) text += ` · <em class="unknown-mini">${unknown} sconosciute</em>`;
  }
  stats.innerHTML = text;
  stats.className = 'mode-' + F.id;
}

// ---- Mode indicators (piccoli punti sui bottoni) ----

export function updateModeIndicators(){
  document.querySelectorAll('button.tab').forEach(btn => {
    btn.classList.toggle('attivo', btn.dataset.view === activeId);
  });
}

// ---- Search ----

export function setupSearch(){
  const searchInput = document.getElementById('search');
  const searchResults = document.getElementById('search-results');
  if(!searchInput || !searchResults) return;

  function hide(){ searchResults.innerHTML = ''; searchResults.style.display = 'none'; }

  function render(matches){
    searchResults.innerHTML = '';
    if(!matches.length){ hide(); return; }
    const frag = document.createDocumentFragment();
    matches.forEach(w => {
      const div = document.createElement('div');
      div.className = 'search-item';
      div.setAttribute('data-dim', DIM_NAMES[dominantDim(w.sig)]);
      div.textContent = w.w;
      div.addEventListener('click', () => {
        searchInput.value = ''; hide();
        _onSelectWord?.(w.w);
      });
      frag.appendChild(div);
    });
    searchResults.appendChild(frag);
    searchResults.style.display = 'block';
  }

  function run(query){
    const q = query.toLowerCase().trim();
    if(!q){ hide(); return; }
    const F = getActive();
    const pool = F.words;
    const token = q.split(/\s+/).filter(Boolean)[0] || q;
    // Match SOLO per prefisso: "gno" non deve restituire "romagnolo".
    // Il match per substring pescava troppo rumore (sillabe centrali) e
    // confondeva l'utente che cercava "comincia per...".
    const starts = [];
    for(const w of pool){
      if(!w.w) continue;
      if(w.w.startsWith(token)) starts.push(w);
      if(starts.length >= 15) break;
    }
    render(starts);
  }

  searchInput.addEventListener('input', () => run(searchInput.value));
  searchInput.addEventListener('keydown', e => {
    if(e.key === 'Enter'){
      e.preventDefault();
      const q = searchInput.value.toLowerCase().trim();
      if(!q) return;
      const first = searchResults.querySelector('.search-item');
      const target = first ? first.textContent : q;
      searchInput.value = ''; hide(); collapseSearch();
      _onSelectWord?.(target);
    } else if(e.key === 'Escape'){
      searchInput.value = ''; hide(); collapseSearch(); searchInput.blur();
    }
  });
  searchInput.addEventListener('blur', () => {
    // Se l'utente ha lasciato il campo vuoto, ricollassa alla lente.
    if(!searchInput.value.trim()) setTimeout(collapseSearch, 150);
  });
  document.addEventListener('click', e => {
    if(e.target !== searchInput && !searchResults.contains(e.target)) hide();
  });

  // Toggle lente: collassato di default, click espande il campo input.
  const ricercaSection = document.getElementById('ricerca');
  const toggleBtn = document.getElementById('search-toggle');
  function expandSearch(){
    ricercaSection?.classList.add('aperto');
    searchInput.hidden = false;
    setTimeout(() => searchInput.focus(), 30);
  }
  function collapseSearch(){
    ricercaSection?.classList.remove('aperto');
    searchInput.hidden = true;
    hide();
  }
  if(toggleBtn) toggleBtn.addEventListener('click', expandSearch);
}
