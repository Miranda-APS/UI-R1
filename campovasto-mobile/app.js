const DIM_COLORS = ['#74d6ff', '#ffb366', '#ff7d7d', '#b780ff', '#d7d3cb', '#56d18f', '#ffd36d', '#ff98c0'];
const PERSONAL_STORAGE_KEY = 'campovasto-mobile-personale-v1';
const NEUTRAL_SIG = [50, 50, 50, 50, 50, 50, 50, 50];
const SCENE_EMPTY_HTML = '<div class="scene-empty">Tocca <strong>interpreta</strong> per generare la prima scena mobile del tuo campo.</div>';

const state = {
  currentView: 'vasto',
  personalMode: 'scene',
  allWords: [],
  allEdges: [],
  wordMap: new Map(),
  outAdj: new Map(),
  inAdj: new Map(),
  vastoNetwork: null,
  personalNetwork: null,
  sceneData: null,
  personal: {
    words: new Map(),
    edges: [],
    edgeKeys: new Set(),
    outAdj: new Map(),
    inAdj: new Map(),
    sentences: [],
    lastFocusNames: [],
  },
};

const refs = {
  primaryCta: document.getElementById('primaryCta'),
  goPersonal: document.getElementById('goPersonal'),
  searchToggle: document.getElementById('searchToggle'),
  searchPanel: document.getElementById('searchPanel'),
  graphHost: document.getElementById('graph'),
  graphOverlay: document.getElementById('graphOverlay'),
  personalGraphHost: document.getElementById('personalGraph'),
  personalGraphOverlay: document.getElementById('personalGraphOverlay'),
  searchInput: document.getElementById('searchInput'),
  searchResults: document.getElementById('searchResults'),
  resetVasto: document.getElementById('resetVasto'),
  sentenceInput: document.getElementById('sentenceInput'),
  composeToggle: document.getElementById('composeToggle'),
  composePanel: document.getElementById('composePanel'),
  buildScene: document.getElementById('buildScene'),
  focusLatest: document.getElementById('focusLatest'),
  clearPersonal: document.getElementById('clearPersonal'),
  sentenceMeta: document.getElementById('sentenceMeta'),
  personalStats: document.getElementById('personalStats'),
  phraseHistory: document.getElementById('phraseHistory'),
  medioScene: document.getElementById('medioScene'),
  sceneCard: document.getElementById('sceneCard'),
  personalGraphCard: document.getElementById('personalGraphCard'),
  personalModeButtons: Array.from(document.querySelectorAll('#personalModeToggle [data-mode]')),
  navButtons: Array.from(document.querySelectorAll('.nav-button')),
  views: {
    vasto: document.getElementById('view-vasto'),
    personale: document.getElementById('view-personale'),
  },
  sheet: document.getElementById('sheet'),
  sheetKicker: document.getElementById('sheetKicker'),
  sheetTitle: document.getElementById('sheetTitle'),
  sheetSubtitle: document.getElementById('sheetSubtitle'),
  sheetContent: document.getElementById('sheetContent'),
  sheetActions: document.getElementById('sheetActions'),
};

init();

function init() {
  bindUi();
  hydratePersonalState();
  loadVasto();
}

function bindUi() {
  refs.primaryCta.addEventListener('click', () => {
    switchView('personale');
    setPanelOpen(refs.composePanel, true);
    refs.sentenceInput.focus();
  });

  refs.goPersonal.addEventListener('click', () => {
    switchView('personale');
    switchPersonalMode('graph');
    setPanelOpen(refs.composePanel, false);
  });

  refs.searchToggle.addEventListener('click', () => {
    const nextState = refs.searchPanel.classList.contains('is-hidden');
    setPanelOpen(refs.searchPanel, nextState);
    if (nextState) refs.searchInput.focus();
  });

  refs.composeToggle.addEventListener('click', () => {
    const nextState = refs.composePanel.classList.contains('is-hidden');
    setPanelOpen(refs.composePanel, nextState);
    if (nextState) refs.sentenceInput.focus();
  });

  refs.navButtons.forEach((button) => {
    button.addEventListener('click', () => {
      const view = button.dataset.view;
      if (view === 'scrivi') {
        switchView('personale');
        switchPersonalMode('scene');
        setPanelOpen(refs.composePanel, true);
        refs.sentenceInput.focus();
        return;
      }
      switchView(view);
    });
  });

  refs.personalModeButtons.forEach((button) => {
    button.addEventListener('click', () => switchPersonalMode(button.dataset.mode));
  });

  refs.searchInput.addEventListener('input', handleSearchInput);
  refs.resetVasto.addEventListener('click', buildInitialGalaxy);
  refs.buildScene.addEventListener('click', handleBuildScene);
  refs.focusLatest.addEventListener('click', focusLatestSentence);
  refs.clearPersonal.addEventListener('click', clearPersonalField);

  refs.sheet.addEventListener('click', (event) => {
    if (event.target === refs.sheet) closeSheet();
  });
}

function switchView(view) {
  state.currentView = view;
  Object.entries(refs.views).forEach(([key, element]) => {
    element.classList.toggle('is-active', key === view);
  });
  refs.navButtons.forEach((button) => {
    const isActive = button.dataset.view === view;
    button.classList.toggle('is-active', isActive);
  });
  if (view !== 'vasto') setPanelOpen(refs.searchPanel, false);
  requestAnimationFrame(() => {
    state.vastoNetwork?.redraw();
    state.personalNetwork?.redraw();
  });
}

function switchPersonalMode(mode) {
  state.personalMode = mode;
  refs.personalModeButtons.forEach((button) => {
    button.classList.toggle('is-active', button.dataset.mode === mode);
  });
  refs.sceneCard.classList.toggle('is-hidden', mode !== 'scene');
  refs.personalGraphCard.classList.toggle('is-hidden', mode !== 'graph');
  if (mode === 'graph') {
    requestAnimationFrame(() => {
      renderPersonalGraph({ focusNames: state.personal.lastFocusNames, fit: true });
    });
  }
}

async function loadVasto() {
  setOverlay(refs.graphOverlay, 'caricamento campo vasto…');
  try {
    const response = await fetch('/api/biennale/field', { cache: 'no-store' });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();
    buildVastoIndexes(data);
    buildInitialGalaxy();
  } catch (error) {
    console.error(error);
    setOverlay(refs.graphOverlay, 'Errore di caricamento. Verifica che il backend sia attivo.');
  }
}

function buildVastoIndexes(data) {
  const filteredWords = (data.words || []).filter((word) => word?.w && word.w.length >= 2);
  const edgeList = Array.isArray(data.edges) ? data.edges : [];

  state.allWords = filteredWords;
  state.allEdges = edgeList;
  state.wordMap = new Map(filteredWords.map((word) => [word.w, word]));
  state.outAdj = new Map();
  state.inAdj = new Map();

  for (const edge of edgeList) {
    pushAdjacency(state.outAdj, edge.from, edge);
    pushAdjacency(state.inAdj, edge.to, edge);
  }
}

function buildInitialGalaxy() {
  if (!state.allWords.length) return;

  renderVastoGraph(state.allWords, state.allEdges, { fit: true, dense: true });
  setOverlay(refs.graphOverlay, '');
  closeSheet();
}

function handleSearchInput() {
  const query = refs.searchInput.value.trim().toLowerCase();
  if (!query) {
    refs.searchResults.innerHTML = '';
    return;
  }

  const matches = state.allWords
    .filter((word) => word.w.toLowerCase().includes(query))
    .sort((a, b) => getVastoDegree(b.w) - getVastoDegree(a.w))
    .slice(0, 12);

  refs.searchResults.innerHTML = matches
    .map((word) => `<button class="search-chip" type="button" data-word="${escapeHtml(word.w)}">${escapeHtml(word.w)}</button>`)
    .join('');

  refs.searchResults.querySelectorAll('[data-word]').forEach((button) => {
    button.addEventListener('click', () => {
      const word = button.dataset.word;
      refs.searchInput.value = word;
      setPanelOpen(refs.searchPanel, false);
      showNeighborhood(word);
    });
  });
}

function showNeighborhood(wordName) {
  const center = state.wordMap.get(wordName);
  if (!center) return;

  const names = new Set([wordName]);
  const outgoing = (state.outAdj.get(wordName) || []).slice(0, 20);
  const incoming = (state.inAdj.get(wordName) || []).slice(0, 10);

  for (const edge of outgoing) names.add(edge.to);
  for (const edge of incoming) names.add(edge.from);

  const nodes = Array.from(names)
    .map((name) => state.wordMap.get(name))
    .filter(Boolean)
    .sort((a, b) => {
      if (a.w === wordName) return -1;
      if (b.w === wordName) return 1;
      return getVastoDegree(b.w) - getVastoDegree(a.w);
    });

  const edges = state.allEdges
    .filter((edge) => names.has(edge.from) && names.has(edge.to))
    .slice(0, 220);

  renderVastoGraph(nodes, edges, { focusWord: wordName, fit: true, dense: false });
  openVastoWordSheet(wordName);
}

function renderVastoGraph(words, edges, opts = {}) {
  const visNodes = words.map((word) => buildVisNode(word, {
    degree: getVastoDegree(word.w),
    isFocus: opts.focusWord && word.w === opts.focusWord,
    dense: !!opts.dense,
  }));
  const visEdges = edges.map((edge) => buildVisEdge(edge, { dense: !!opts.dense }));
  const options = buildNetworkOptions({ dense: !!opts.dense });

  if (!state.vastoNetwork) {
    state.vastoNetwork = new vis.Network(refs.graphHost, { nodes: visNodes, edges: visEdges }, options);
    state.vastoNetwork.on('click', (params) => {
      if (!params.nodes.length) {
        closeSheet();
        return;
      }
      openVastoWordSheet(params.nodes[0]);
    });
    state.vastoNetwork.on('dragStart', closeSheet);
  } else {
    state.vastoNetwork.setData({ nodes: visNodes, edges: visEdges });
    state.vastoNetwork.setOptions(options);
  }

  if (opts.fit) {
    if (opts.dense) {
      requestAnimationFrame(() => {
        state.vastoNetwork.fit({ animation: { duration: 220, easingFunction: 'easeInOutQuad' } });
      });
    } else {
      state.vastoNetwork.once('stabilizationIterationsDone', () => {
        state.vastoNetwork.fit({ animation: { duration: 280, easingFunction: 'easeInOutQuad' } });
      });
    }
  }

  setOverlay(refs.graphOverlay, '');
}

function openVastoWordSheet(wordName) {
  const word = state.wordMap.get(wordName);
  if (!word) return;
  const outgoing = sortByConfidence(state.outAdj.get(wordName) || []).slice(0, 6);
  const incoming = sortByConfidence(state.inAdj.get(wordName) || []).slice(0, 4);

  renderSheet({
    kicker: 'campo vasto',
    title: wordName,
    subtitle: buildSignatureSummary(word),
    contentHtml: `
      <div class="sheet-grid">
        <div class="sheet-stat">
          <span class="sheet-stat-label">grado</span>
          <strong>${getVastoDegree(wordName)}</strong>
        </div>
        <div class="sheet-stat">
          <span class="sheet-stat-label">traiettoria</span>
          <strong>${outgoing.length} uscenti / ${incoming.length} entranti</strong>
        </div>
      </div>
      <div class="sheet-list">
        ${renderRelationItems(outgoing, 'Nessuna relazione uscente nel vicinato corrente.')}
      </div>
    `,
    actions: [
      { label: 'porta nel tuo campo', onClick: () => addVastoWordToPersonal(wordName) },
      { label: 'apri vicinato', onClick: () => showNeighborhood(wordName) },
    ],
  });
}

async function handleBuildScene() {
  const sentence = refs.sentenceInput.value.trim();
  if (!sentence) {
    refs.sentenceMeta.textContent = 'Scrivi una frase per costruire il campo.';
    return;
  }

  refs.sentenceMeta.textContent = 'Interpretazione in corso…';
  refs.buildScene.disabled = true;

  try {
    const response = await fetch(`/api/medio?sentence=${encodeURIComponent(sentence)}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();
    const focusNames = mergeInterpretationIntoPersonal(sentence, data);
    state.sceneData = { sentence, data, focusNames };
    refs.sentenceMeta.textContent = `${(data.lemmas || []).length} lemmi riconosciuti · ${(data.unknown || []).length} parole sconosciute`;
    renderMedioScene(sentence, data);
    renderPersonalUi();
    renderPersonalGraph({ focusNames, fit: true });
    switchView('personale');
    switchPersonalMode('scene');
    setPanelOpen(refs.composePanel, false);
  } catch (error) {
    console.error(error);
    refs.sentenceMeta.textContent = 'Errore: impossibile generare la scena.';
  } finally {
    refs.buildScene.disabled = false;
  }
}

function mergeInterpretationIntoPersonal(sentence, data) {
  const wordsByName = new Map((data.words || []).map((word) => [word.word, word]));
  const unknownSet = new Set(data.unknown || []);
  const focusNames = [];

  for (const lemma of data.lemmas || []) {
    const found = wordsByName.get(lemma);
    if (found) {
      upsertPersonalWord({
        w: found.word,
        sig: normalizeSig(found.signature),
        source: 'sentence',
      });
      focusNames.push(found.word);
      continue;
    }
    if (unknownSet.has(lemma)) {
      upsertPersonalWord({
        w: lemma,
        sig: NEUTRAL_SIG.slice(),
        source: 'unknown',
      });
      focusNames.push(lemma);
    }
  }

  for (const word of data.words || []) {
    upsertPersonalWord({
      w: word.word,
      sig: normalizeSig(word.signature),
      source: 'sentence',
    });

    for (const edge of sortByConfidence(word.outgoing || []).slice(0, 8)) {
      upsertPersonalWord({
        w: edge.target,
        sig: normalizeSig(edge.target_signature),
        source: 'satellite',
      });
      addPersonalEdge({
        from: word.word,
        to: edge.target,
        rel: edge.relation,
        conf: Math.round((edge.confidence || 0) * 100),
      });
    }
  }

  const entry = {
    text: sentence,
    focusNames,
    createdAt: Date.now(),
  };
  state.personal.sentences = [entry, ...state.personal.sentences.filter((item) => item.text !== sentence)].slice(0, 18);
  state.personal.lastFocusNames = focusNames;
  persistPersonalState();
  return focusNames;
}

function addVastoWordToPersonal(wordName) {
  const word = state.wordMap.get(wordName);
  if (!word) return;

  upsertPersonalWord({
    w: word.w,
    sig: Array.isArray(word.sig) ? word.sig.slice() : NEUTRAL_SIG.slice(),
    source: 'vasto',
  });

  for (const edge of sortByConfidence(state.outAdj.get(wordName) || []).slice(0, 6)) {
    const target = state.wordMap.get(edge.to);
    upsertPersonalWord({
      w: edge.to,
      sig: Array.isArray(target?.sig) ? target.sig.slice() : NEUTRAL_SIG.slice(),
      source: 'satellite',
    });
    addPersonalEdge({
      from: edge.from,
      to: edge.to,
      rel: edge.rel,
      conf: edge.conf || 50,
    });
  }

  state.personal.lastFocusNames = [wordName];
  persistPersonalState();
  renderPersonalUi();
  switchView('personale');
  switchPersonalMode('graph');
  setPanelOpen(refs.composePanel, false);
  renderPersonalGraph({ focusNames: [wordName], fit: true });
  openPersonalWordSheet(wordName);
}

function focusLatestSentence() {
  if (!state.personal.lastFocusNames.length) {
    refs.sentenceMeta.textContent = 'Non ci sono ancora parole recenti nel tuo campo.';
    return;
  }
  switchView('personale');
  switchPersonalMode('graph');
  setPanelOpen(refs.composePanel, false);
  renderPersonalGraph({ focusNames: state.personal.lastFocusNames, fit: true });
}

function clearPersonalField() {
  if (!confirm('Svuotare il tuo campo mobile?')) return;
  state.personal = {
    words: new Map(),
    edges: [],
    edgeKeys: new Set(),
    outAdj: new Map(),
    inAdj: new Map(),
    sentences: [],
    lastFocusNames: [],
  };
  state.sceneData = null;
  refs.sentenceInput.value = '';
  refs.sentenceMeta.textContent = 'Nessuna frase attiva.';
  refs.medioScene.innerHTML = SCENE_EMPTY_HTML;
  setPanelOpen(refs.composePanel, false);
  persistPersonalState();
  renderPersonalUi();
  renderPersonalGraph({ fit: false });
  closeSheet();
}

function renderMedioScene(sentence, data) {
  const sceneWords = (data.words || []).slice();
  const satellites = collectSatellites(sceneWords);
  const unknown = (data.unknown || []).slice(0, 4);

  const stage = document.createElement('div');
  stage.className = 'scene-stage';
  stage.innerHTML = '<div class="scene-haze"></div>';

  sceneWords.forEach((word, index) => {
    stage.appendChild(buildSceneCard({
      label: word.word,
      meta: `${(word.outgoing || []).length} relazioni`,
      kind: 'lemma',
      x: mapIndex(index, sceneWords.length, -34, 34),
      y: 18 + (index % 2) * 8,
      z: 120 - index * 14,
      rotateY: mapIndex(index, sceneWords.length, -10, 10),
      payload: { type: 'lemma', word },
    }));
  });

  satellites.forEach((item, index) => {
    stage.appendChild(buildSceneCard({
      label: item.target,
      meta: `${item.relation} · ${Math.round(item.confidence * 100)}%`,
      kind: 'satellite',
      x: mapIndex(index, satellites.length, -42, 42),
      y: -18 + ((index % 3) - 1) * 18,
      z: -80 - (index % 4) * 42,
      rotateY: mapIndex(index, satellites.length, -18, 18),
      payload: { type: 'satellite', item },
    }));
  });

  unknown.forEach((label, index) => {
    stage.appendChild(buildSceneCard({
      label,
      meta: 'non riconosciuta nel KG',
      kind: 'unknown',
      x: -28 + index * 18,
      y: 34,
      z: 42 - index * 16,
      rotateY: -8 + index * 5,
      payload: { type: 'unknown', label },
    }));
  });

  const ribbon = document.createElement('div');
  ribbon.className = 'scene-ribbon';
  const tokens = sentence.split(/\s+/).filter(Boolean).slice(0, 18);
  ribbon.innerHTML = tokens.map((token) => `<span class="scene-token">${escapeHtml(token)}</span>`).join('');
  stage.appendChild(ribbon);

  refs.medioScene.innerHTML = '';
  refs.medioScene.appendChild(stage);
}

function buildSceneCard({ label, meta, kind, x, y, z, rotateY, payload }) {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = `scene-card-node is-${kind}`;
  button.style.transform = `translate3d(calc(${x}% - 50%), calc(${y}% - 50%), ${z}px) rotateY(${rotateY}deg)`;
  button.innerHTML = `<strong>${escapeHtml(label)}</strong><span>${escapeHtml(meta)}</span>`;
  button.addEventListener('click', () => openSceneSheet(payload));
  return button;
}

function openSceneSheet(payload) {
  if (payload.type === 'lemma') {
    const word = payload.word;
    const items = (word.outgoing || []).slice(0, 5);
    renderSheet({
      kicker: 'campo medio',
      title: word.word,
      subtitle: 'lemma emerso dalla frase',
      contentHtml: `
        <div class="sheet-grid">
          <div class="sheet-stat">
            <span class="sheet-stat-label">firma</span>
            <strong>${word.signature ? '8D disponibile' : 'firma assente'}</strong>
          </div>
          <div class="sheet-stat">
            <span class="sheet-stat-label">outgoing</span>
            <strong>${items.length} relazioni mostrate</strong>
          </div>
        </div>
        <div class="sheet-list">
          ${items.map((item) => `
            <div class="sheet-list-item">
              <div>${escapeHtml(item.target)}</div>
              <small>${escapeHtml(item.relation)} · ${escapeHtml(item.direction || 'out')}</small>
            </div>
          `).join('') || '<div class="sheet-list-item">Nessuna relazione uscente disponibile.</div>'}
        </div>
      `,
      actions: [
        { label: 'apri nel tuo campo', onClick: () => focusWordInPersonal(word.word) },
        { label: 'apri nel vasto', onClick: () => openInVasto(word.word) },
      ],
    });
    return;
  }

  if (payload.type === 'satellite') {
    renderSheet({
      kicker: 'campo medio',
      title: payload.item.target,
      subtitle: 'satellite semantico',
      contentHtml: `
        <div class="sheet-list">
          <div class="sheet-list-item">
            <div>relazione</div>
            <small>${escapeHtml(payload.item.relation)}</small>
          </div>
          <div class="sheet-list-item">
            <div>confidenza</div>
            <small>${Math.round(payload.item.confidence * 100)}%</small>
          </div>
        </div>
      `,
      actions: [
        { label: 'apri nel tuo campo', onClick: () => focusWordInPersonal(payload.item.target) },
        { label: 'apri nel vasto', onClick: () => openInVasto(payload.item.target) },
      ],
    });
    return;
  }

  renderSheet({
    kicker: 'campo medio',
    title: payload.label,
    subtitle: 'parola non riconosciuta',
    contentHtml: '<div class="sheet-list-item">Questa parola non e\' nel lessico attuale del grafo.</div>',
    actions: [
      { label: 'aggiungi neutra al tuo campo', onClick: () => addUnknownToPersonal(payload.label) },
    ],
  });
}

function renderPersonalUi() {
  const wordCount = state.personal.words.size;
  const edgeCount = state.personal.edges.length;
  const sentenceCount = state.personal.sentences.length;

  refs.personalStats.innerHTML = `
    <div class="meta-chip"><span>parole</span><strong>${wordCount}</strong></div>
    <div class="meta-chip"><span>archi</span><strong>${edgeCount}</strong></div>
    <div class="meta-chip"><span>frasi</span><strong>${sentenceCount}</strong></div>
  `;

  refs.phraseHistory.innerHTML = state.personal.sentences
    .slice(0, 8)
    .map((entry, index) => `<button class="search-chip" type="button" data-history-index="${index}">${escapeHtml(trimText(entry.text, 28))}</button>`)
    .join('');

  refs.phraseHistory.querySelectorAll('[data-history-index]').forEach((button) => {
    button.addEventListener('click', () => {
      const entry = state.personal.sentences[Number(button.dataset.historyIndex)];
      if (!entry) return;
      refs.sentenceInput.value = entry.text;
      refs.sentenceMeta.textContent = 'Frase recuperata dal tuo campo.';
      switchView('personale');
      switchPersonalMode('graph');
      setPanelOpen(refs.composePanel, false);
      renderPersonalGraph({ focusNames: entry.focusNames, fit: true });
    });
  });

  setOverlay(
    refs.personalGraphOverlay,
    wordCount ? '' : 'Il tuo campo e\' vuoto. Scrivi una frase o aggiungi parole dal vasto.'
  );
}

function renderPersonalGraph({ focusNames = [], fit = true } = {}) {
  const words = Array.from(state.personal.words.values());
  const edges = state.personal.edges.slice();

  if (!words.length) {
    if (state.personalNetwork) {
      state.personalNetwork.setData({ nodes: [], edges: [] });
    }
    setOverlay(refs.personalGraphOverlay, 'Il tuo campo e\' vuoto. Scrivi una frase o aggiungi parole dal vasto.');
    return;
  }

  const focusSet = new Set(focusNames || []);
  const visNodes = words.map((word) => buildVisNode(word, {
    degree: getPersonalDegree(word.w),
    isFocus: focusSet.has(word.w),
    dense: false,
  }));
  const visEdges = edges.map((edge) => buildVisEdge(edge, { dense: false }));
  const options = buildNetworkOptions({ dense: false });

  if (!state.personalNetwork) {
    state.personalNetwork = new vis.Network(refs.personalGraphHost, { nodes: visNodes, edges: visEdges }, options);
    state.personalNetwork.on('click', (params) => {
      if (!params.nodes.length) {
        closeSheet();
        return;
      }
      openPersonalWordSheet(params.nodes[0]);
    });
    state.personalNetwork.on('dragStart', closeSheet);
  } else {
    state.personalNetwork.setData({ nodes: visNodes, edges: visEdges });
    state.personalNetwork.setOptions(options);
  }

  requestAnimationFrame(() => {
    state.personalNetwork.redraw();
    if (fit) {
      const validFocus = focusNames.filter((name) => state.personal.words.has(name));
      if (validFocus.length) {
        state.personalNetwork.fit({
          nodes: validFocus,
          animation: { duration: 280, easingFunction: 'easeInOutQuad' },
        });
      } else {
        state.personalNetwork.fit({ animation: { duration: 280, easingFunction: 'easeInOutQuad' } });
      }
    }
  });

  setOverlay(refs.personalGraphOverlay, '');
}

function openPersonalWordSheet(wordName) {
  const word = state.personal.words.get(wordName);
  if (!word) return;
  const outgoing = sortByConfidence(state.personal.outAdj.get(wordName) || []).slice(0, 6);
  const incoming = sortByConfidence(state.personal.inAdj.get(wordName) || []).slice(0, 4);

  renderSheet({
    kicker: 'tuo campo',
    title: wordName,
    subtitle: buildSignatureSummary(word),
    contentHtml: `
      <div class="sheet-grid">
        <div class="sheet-stat">
          <span class="sheet-stat-label">grado</span>
          <strong>${getPersonalDegree(wordName)}</strong>
        </div>
        <div class="sheet-stat">
          <span class="sheet-stat-label">origine</span>
          <strong>${escapeHtml(word.source || 'personale')}</strong>
        </div>
      </div>
      <div class="sheet-list">
        ${renderRelationItems(outgoing, 'Nessuna relazione uscente nel tuo campo.')}
      </div>
    `,
    actions: [
      { label: 'metti al centro', onClick: () => focusWordInPersonal(wordName) },
      { label: 'apri nel vasto', onClick: () => openInVasto(wordName) },
      { label: 'rimuovi', onClick: () => removePersonalWord(wordName) },
    ],
  });
}

function focusWordInPersonal(wordName) {
  switchView('personale');
  switchPersonalMode('graph');
  renderPersonalGraph({ focusNames: [wordName], fit: true });
  if (state.personal.words.has(wordName)) {
    requestAnimationFrame(() => openPersonalWordSheet(wordName));
  }
}

function openInVasto(wordName) {
  if (!state.wordMap.has(wordName)) return;
  switchView('vasto');
  setPanelOpen(refs.searchPanel, false);
  refs.searchInput.value = wordName;
  showNeighborhood(wordName);
}

function addUnknownToPersonal(wordName) {
  upsertPersonalWord({ w: wordName, sig: NEUTRAL_SIG.slice(), source: 'unknown' });
  state.personal.lastFocusNames = [wordName];
  persistPersonalState();
  renderPersonalUi();
  focusWordInPersonal(wordName);
}

function removePersonalWord(wordName) {
  if (!state.personal.words.has(wordName)) return;
  state.personal.words.delete(wordName);
  state.personal.edges = state.personal.edges.filter((edge) => edge.from !== wordName && edge.to !== wordName);
  rebuildPersonalIndexes();
  persistPersonalState();
  renderPersonalUi();
  renderPersonalGraph({ focusNames: [], fit: true });
  closeSheet();
}

function hydratePersonalState() {
  refs.medioScene.innerHTML = SCENE_EMPTY_HTML;
  try {
    const raw = localStorage.getItem(PERSONAL_STORAGE_KEY);
    if (!raw) {
      renderPersonalUi();
      return;
    }
    const parsed = JSON.parse(raw);
    state.personal.words = new Map((parsed.words || []).map((word) => [word.w, word]));
    state.personal.edges = Array.isArray(parsed.edges) ? parsed.edges : [];
    state.personal.sentences = Array.isArray(parsed.sentences) ? parsed.sentences : [];
    state.personal.lastFocusNames = Array.isArray(parsed.lastFocusNames) ? parsed.lastFocusNames : [];
    rebuildPersonalIndexes();
  } catch (error) {
    console.error('Errore ripristino campo personale mobile:', error);
    state.personal.words = new Map();
    state.personal.edges = [];
    state.personal.sentences = [];
    state.personal.lastFocusNames = [];
    rebuildPersonalIndexes();
  }
  renderPersonalUi();
}

function persistPersonalState() {
  const payload = {
    words: Array.from(state.personal.words.values()),
    edges: state.personal.edges,
    sentences: state.personal.sentences,
    lastFocusNames: state.personal.lastFocusNames,
  };
  localStorage.setItem(PERSONAL_STORAGE_KEY, JSON.stringify(payload));
}

function rebuildPersonalIndexes() {
  state.personal.edgeKeys = new Set();
  state.personal.outAdj = new Map();
  state.personal.inAdj = new Map();

  for (const edge of state.personal.edges) {
    state.personal.edgeKeys.add(edgeKey(edge));
    pushAdjacency(state.personal.outAdj, edge.from, edge);
    pushAdjacency(state.personal.inAdj, edge.to, edge);
  }
}

function upsertPersonalWord(word) {
  const existing = state.personal.words.get(word.w);
  if (existing) {
    if ((!existing.sig || !existing.sig.length || isNeutralSig(existing.sig)) && word.sig?.length) {
      existing.sig = word.sig.slice();
    }
    if (word.source && (existing.source === 'unknown' || existing.source === 'satellite')) {
      existing.source = word.source;
    }
    state.personal.words.set(existing.w, existing);
    return existing;
  }
  const record = {
    w: word.w,
    sig: Array.isArray(word.sig) && word.sig.length ? word.sig.slice() : NEUTRAL_SIG.slice(),
    source: word.source || 'personale',
  };
  state.personal.words.set(record.w, record);
  return record;
}

function addPersonalEdge(edge) {
  const key = edgeKey(edge);
  if (state.personal.edgeKeys.has(key)) return;
  state.personal.edgeKeys.add(key);
  state.personal.edges.push(edge);
  pushAdjacency(state.personal.outAdj, edge.from, edge);
  pushAdjacency(state.personal.inAdj, edge.to, edge);
}

function collectSatellites(words) {
  const seen = new Set(words.map((word) => word.word));
  const satellites = [];
  for (const word of words) {
    for (const edge of sortByConfidence(word.outgoing || [])) {
      if (seen.has(edge.target)) continue;
      satellites.push(edge);
      seen.add(edge.target);
      if (satellites.length >= 12) return satellites;
    }
  }
  return satellites;
}

function renderSheet({ kicker, title, subtitle, contentHtml, actions = [] }) {
  refs.sheetKicker.textContent = kicker;
  refs.sheetTitle.textContent = title;
  refs.sheetSubtitle.textContent = subtitle;
  refs.sheetContent.innerHTML = contentHtml;
  refs.sheetActions.innerHTML = actions
    .map((action, index) => `<button class="ghost-button" type="button" data-sheet-action="${index}">${escapeHtml(action.label)}</button>`)
    .join('');

  refs.sheetActions.querySelectorAll('[data-sheet-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const action = actions[Number(button.dataset.sheetAction)];
      if (action) action.onClick();
    });
  });
  openSheet();
}

function setPanelOpen(panel, isOpen) {
  panel.classList.toggle('is-hidden', !isOpen);
}

function openSheet() {
  refs.sheet.classList.add('is-open');
  refs.sheet.setAttribute('aria-hidden', 'false');
}

function closeSheet() {
  refs.sheet.classList.remove('is-open');
  refs.sheet.setAttribute('aria-hidden', 'true');
}

function setOverlay(element, text) {
  element.textContent = text;
  element.classList.toggle('is-hidden', !text);
}

function buildVisNode(word, { degree, isFocus, dense }) {
  const color = pickWordColor(word);
  const shouldShowLabel = isFocus || (!dense && degree >= 4);
  const hasFixedPos = Number.isFinite(word.x) && Number.isFinite(word.y);
  const nodeX = hasFixedPos ? (word.x - 0.5) * 2200 : undefined;
  const nodeY = hasFixedPos ? (word.y - 0.5) * 2200 : undefined;
  return {
    id: word.w,
    label: shouldShowLabel ? word.w : '',
    shape: 'dot',
    size: isFocus
      ? 20
      : dense
        ? clamp(2 + degree * 0.055, 2, 6)
        : clamp(6 + degree * 0.14, 7, 14),
    font: {
      face: 'JetBrains Mono',
      color: '#eef2ff',
      size: isFocus ? 18 : dense ? 0 : 13,
      strokeWidth: 0,
    },
    borderWidth: isFocus ? 2 : dense ? 0 : 1,
    color: {
      background: color,
      border: isFocus ? '#ffffff' : dense ? 'rgba(255,255,255,0.06)' : 'rgba(255,255,255,0.22)',
      highlight: { background: color, border: '#ffffff' },
    },
    x: hasFixedPos ? nodeX : undefined,
    y: hasFixedPos ? nodeY : undefined,
    fixed: hasFixedPos && dense ? { x: true, y: true } : false,
  };
}

function buildVisEdge(edge, { dense }) {
  return {
    id: `${edge.from}-${edge.rel}-${edge.to}`,
    from: edge.from,
    to: edge.to,
    arrows: 'to',
    label: dense ? '' : shortRelation(edge.rel),
    font: {
      face: 'JetBrains Mono',
      size: dense ? 0 : 11,
      color: '#93a0c0',
      strokeWidth: 0,
      align: 'middle',
    },
    width: dense ? 0.35 : clamp((edge.conf || 40) / 28, 1, 3),
    color: {
      color: dense ? 'rgba(255,255,255,0.085)' : 'rgba(255,255,255,0.18)',
      highlight: '#74d6ff',
      opacity: 0.8,
    },
    smooth: {
      enabled: !dense,
      type: dense ? 'continuous' : 'dynamic',
    },
  };
}

function buildNetworkOptions({ dense }) {
  return {
    autoResize: true,
    interaction: {
      hover: false,
      zoomView: true,
      dragView: true,
      multiselect: false,
      hideEdgesOnDrag: dense,
      hideEdgesOnZoom: dense,
    },
    physics: dense ? false : {
      stabilization: { iterations: 120 },
      barnesHut: {
        gravitationalConstant: -3500,
        centralGravity: 0.18,
        springLength: 110,
        springConstant: 0.025,
        damping: 0.82,
      },
    },
    layout: {
      improvedLayout: !dense,
    },
  };
}

function renderRelationItems(items, emptyMessage) {
  if (!items.length) return `<div class="sheet-list-item">${emptyMessage}</div>`;
  return items.map((edge) => `
    <div class="sheet-list-item">
      <div>${escapeHtml(edge.to)}</div>
      <small>${escapeHtml(edge.rel)} · conf ${Math.round(edge.conf || 0)}</small>
    </div>
  `).join('');
}

function getVastoDegree(wordName) {
  return (state.outAdj.get(wordName)?.length || 0) + (state.inAdj.get(wordName)?.length || 0);
}

function getPersonalDegree(wordName) {
  return (state.personal.outAdj.get(wordName)?.length || 0) + (state.personal.inAdj.get(wordName)?.length || 0);
}

function pushAdjacency(map, key, value) {
  if (!map.has(key)) map.set(key, []);
  map.get(key).push(value);
}

function sortByConfidence(edges) {
  return edges.slice().sort((a, b) => (b.conf ?? b.confidence ?? 0) - (a.conf ?? a.confidence ?? 0));
}

function edgeKey(edge) {
  return `${edge.from}::${edge.rel}::${edge.to}`;
}

function normalizeSig(sig) {
  if (!Array.isArray(sig) || sig.length !== 8) return NEUTRAL_SIG.slice();
  return sig.map((value) => {
    const numeric = Number(value) || 0;
    return numeric <= 1 ? Math.round(numeric * 100) : Math.round(numeric);
  });
}

function isNeutralSig(sig) {
  return Array.isArray(sig) && sig.every((value) => Number(value) === 50);
}

function pickWordColor(word) {
  const sig = Array.isArray(word.sig) ? word.sig : null;
  if (!sig || !sig.length) return '#74d6ff';
  let maxIndex = 0;
  let maxValue = -Infinity;
  sig.forEach((value, index) => {
    const current = Number(value) || 0;
    if (current > maxValue) {
      maxValue = current;
      maxIndex = index;
    }
  });
  return DIM_COLORS[maxIndex] || '#74d6ff';
}

function buildSignatureSummary(word) {
  const sig = Array.isArray(word.sig) ? word.sig : [];
  if (!sig.length) return 'firma 8D non disponibile';
  const pairs = sig.map((value, index) => ({ index, value: Number(value) || 0 }));
  pairs.sort((a, b) => b.value - a.value);
  const top = pairs.slice(0, 2).map((pair) => `d${pair.index + 1}:${Math.round(pair.value)}`).join(' · ');
  return `dominanti ${top}`;
}

function shortRelation(relation) {
  if (!relation) return '';
  return relation.length > 12 ? relation.slice(0, 12) : relation;
}

function mapIndex(index, total, min, max) {
  if (total <= 1) return 0;
  const ratio = index / (total - 1);
  return min + (max - min) * ratio;
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function trimText(value, maxLength) {
  if (value.length <= maxLength) return value;
  return `${value.slice(0, maxLength - 1)}…`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');
}
