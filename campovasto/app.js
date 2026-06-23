// Entry point: fetch vasto, crea i due Field, monta la network, cabla tutti
// i wiring. Target: ≤150 righe (vedi CLAUDE.md §3). Nessuna logica —
// solo orchestrazione del boot.

import { applyThemeToCss } from './js/theme.js';
import { deriveFrame, rankNormalizeInPlace } from './js/geometry.js';
import { Field } from './js/field.js';
import { FIELDS, getActive, activeId, loadField, registerField, saveField } from './js/manager.js';
import { initNetwork, mountField, setHandlers } from './js/graph.js';
import {
  openNodeCtxMenu, openEdgeCtxMenu, openQuickEdge, openConnect,
  setEditorCallbacks,
} from './js/editor.js';
import { setupSearch, setOnSelectWord } from './js/sidebar.js';
import { buildFilterPanel } from './js/filters.js';
import { buildRelLegend } from './js/rel-legend.js';

import { selectWord, deselectWord, refreshSelectedPanels } from './js/wiring/selection.js';
import {
  refreshFieldUI, openEmptyCtxMenu, initViewSwitcherWiring, applyNuovoLayout,
} from './js/wiring/view-switcher.js';
import {
  getNuovoLayout,
  onLinkModeChange, onLinkFocusChange, clearLinkFocus, setLinkMode,
} from './js/ui-state.js';
import { wireButtons } from './js/wiring/buttons.js';
import { wireKeyboard } from './js/wiring/keyboard.js';
import { wireSidebarLayout } from './js/wiring/sidebar-layout.js';
import { setApplyFn } from './js/history.js';
import { buildGraphToolbar } from './js/components/graph-toolbar.js';
import { buildDimOverlay, setDimOverlayVisible } from './js/components/dim-overlay.js';
import { initTrail, setTrailHover } from './js/components/exploration-trail.js';
import { clearInfo } from './js/sidebar.js';

async function init(){
  applyThemeToCss();

  const loading = document.getElementById('loading');
  loading.textContent = 'caricamento campo…';

  let data;
  try {
    // cache: 'no-store' per evitare che il browser serva una versione vecchia
    // del campo dopo modifiche server-side. Il payload è grande (~3K nodi) ma
    // il bottleneck è il rendering, non la fetch.
    const resp = await fetch('/api/biennale/field_all', { cache: 'no-store' });
    if(!resp.ok) throw new Error('HTTP ' + resp.status);
    data = await resp.json();
  } catch(e){
    console.error('Errore caricamento campo:', e);
    const empty = document.getElementById('info-empty');
    if(empty) empty.innerHTML =
      '<span class="word-empty">Errore: impossibile caricare i dati. Verifica che il server sia attivo.</span>';
    loading.style.display = 'none';
    return;
  }

  const filterRe = /[\d_]/;
  const allWords = (data.words || []).filter(w => w.w.length >= 2 && !filterRe.test(w.w));
  const allEdges = data.edges || [];

  rankNormalizeInPlace(allWords);
  const frame = deriveFrame(allWords);

  loading.textContent = 'costruzione campo vasto…';
  // Yield al browser: lascia ridipingere il loader prima del lavoro pesante.
  await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));

  const V = new Field('vasto', frame);
  V.addDimLabels();
  V.bulkLoad({ words: allWords, edges: allEdges });
  registerField('vasto', V);

  // Personale: idratato da localStorage se esiste, altrimenti vuoto.
  const P = new Field('nuovo', frame);
  P.addDimLabels();
  const savedP = loadField('nuovo');
  if(savedP){ P.hydrate(savedP); P.spreadNonOverlapping(); }
  registerField('nuovo', P);

  loading.style.display = 'none';

  buildFilterPanel();
  buildRelLegend();
  setupSearch();

  setOnSelectWord(selectWord);
  setEditorCallbacks({
    onWordChanged: (w) => {
      const F = getActive();
      if(F?.selected === w) refreshSelectedPanels();
      if(F && F.id === 'nuovo'){
        // Layout dimensionale: spread per evitare sovrapposizioni dopo add/edit.
        // Layout rettangolare: ricalcolo completo (la nuova parola va in 6° fascia
        // se orfana, o nella fascia/colonna corretta se ha già un arco).
        if(getNuovoLayout() === 'rectangular') applyNuovoLayout();
        else F.spreadNonOverlapping({ iterations: 30 });
      }
      refreshFieldUI();
    },
    onEdgeChanged: () => {
      // Aggiungere/rimuovere archi cambia la struttura visiva del rectangular
      // (colonne dei vicini per host) e la densità del dimensional. Senza
      // riapplicare il layout, le colonne svuotate restano vuote e quelle
      // dense non si ricompattano.
      const F = getActive();
      if(F && F.id === 'nuovo'){
        if(getNuovoLayout() === 'rectangular') applyNuovoLayout();
        else F.spreadNonOverlapping({ iterations: 30 });
      }
      refreshFieldUI();
    },
    onSelect:   (w) => { selectWord(w); refreshFieldUI(); },
    onDeselect: ()  => { deselectWord(); refreshFieldUI(); },
  });

  // Undo/redo: applica la snapshot al field corrispondente, persiste senza
  // creare nuovi entry, deseleziona e refresh UI. Registrato qui perché è il
  // solo punto che ha visibilità su FIELDS + saveField + UI helpers.
  setApplyFn((id, snap) => {
    const F = FIELDS[id];
    if(!F) return;
    F.replaceFromSnapshot(snap);
    saveField(id, { silent: true });
    document.body.classList.remove('parola-selezionata');
    clearInfo();
    // replaceFromSnapshot ricostruisce le parole con buildNodeSpec senza
    // layoutMode → stile sempre dimensional. Se il toggle dell'utente è su
    // rectangular, riapplichiamo il layout corrente per riallineare stili
    // e posizioni al modo attivo.
    if(id === 'nuovo' && activeId === 'nuovo') applyNuovoLayout();
    refreshFieldUI();
  });

  initNetwork(document.getElementById('graph'));
  setHandlers({
    onSelectWord: selectWord,
    onDeselect:   deselectWord,
    onCtxNode:    openNodeCtxMenu,
    onCtxEdge:    openEdgeCtxMenu,
    onCtxEmpty:   (sx, sy, cx, cy) => {
      // Menu di creazione solo nel personale.
      if(activeId === 'vasto') return;
      openEmptyCtxMenu(sx, sy, cx, cy);
    },
    onConnect: (from, to) => {
      if(activeId === 'vasto') return;
      openConnect(from, to);
    },
    onEditEdge: openQuickEdge,
    onNodeHover: (word) => setTrailHover(word),
  });
  // animate:false sul mount iniziale: la fit-animation ridisegnerebbe i 9k
  // nodi a ogni frame per ~500ms al boot. Fit istantaneo → caricamento scheggia.
  mountField('vasto', { fit: true, animate: false });
  // Body class iniziale per il gating CSS (es. #filtri visibile solo in vasto).
  document.body.classList.add('campo-vasto');

  initViewSwitcherWiring();
  buildGraphToolbar();
  buildDimOverlay();
  setDimOverlayVisible(true);
  // Trail di esplorazione (top-right del grafo). onWordPick: null = deseleziona,
  // word = naviga lì (selectWord trunca il navPath se la parola era già nel path).
  initTrail(document.getElementById('graph'), {
    onWordPick: (w) => { if(w) selectWord(w); else deselectWord(); },
  });
  refreshFieldUI();

  // Wiring linkMode + linkFocus → applica al field corrente. Solo "nuovo"
  // accetta queste modalità (vasto è read-only). Cambio campo: lo stato
  // viene resettato per non lasciare il "nuovo" in linkMode mentre l'utente
  // guarda il vasto.
  onLinkModeChange((on) => {
    const F = getActive();
    if(!F || F.id !== 'nuovo') return;
    F.applyLinkMode(on);
    document.body.classList.toggle('link-mode', on);
  });
  onLinkFocusChange((focus) => {
    const F = getActive();
    if(!F || F.id !== 'nuovo'){ return; }
    F.applyLinkFocus(focus);
    document.body.classList.toggle('link-focus', !!focus);
  });
  wireButtons();
  wireKeyboard();
  wireSidebarLayout();
}

init();
