// Network vis condivisa + mount/unmount dei field + highlight/selezione.
// Nessuno stile inline: ogni spec nodo/arco passa per node-style.js
// (vedi CLAUDE.md §2).

import { getActive, setActive, FIELDS, saveField } from './manager.js';
import { buildNodeSpec, buildEdgeSpec } from './node-style.js';
import { UI, tokens, colorForSig } from './theme.js';
import { REL_GROUP, GROUP_DASH } from './constants.js';
import {
  getMatchedWords, isFilterActive, getFilterDirection, getAllowedEdges,
  isRelTypeEnabled, getNuovoLayout,
} from './ui-state.js';
import {
  initOverlay, setActiveTraceEdges, setActiveLabels, clearOverlay,
} from './components/overlay.js';

export let network = null;
let _handlers = {};
let _justDragged = false;  // True per ~50ms dopo un drag-to-connect riuscito.

export function initNetwork(container){
  network = new vis.Network(container, { nodes: new vis.DataSet([]), edges: new vis.DataSet([]) }, {
    physics: { enabled: false },
    // improvedLayout=false: tutti i nodi hanno già posizione esplicita
    // (fixed: true via _placeIfNeeded → placeByRank). Il layout engine non
    // serve, e quando il numero di nodi cambia bruscamente (es. rimozione di
    // ~3000 nodi vasto-clone dopo l'animazione di comprensione frase) il suo
    // tentativo di "ridisporre" produce un cerchio fantasma esterno.
    layout: { improvedLayout: false },
    interaction: {
      hover: true,
      hoverConnectedEdges: false, // Evita instabilità e sfarfallii su hover
      tooltipDelay: 50,
      zoomView: true,
      dragView: true,
      zoomSpeed: 0.6,
    },
    nodes: { shape: 'dot', borderWidth: tokens.border.normal },
    edges: { smooth: false },
  });
  _wireEvents();
  initOverlay(network);   // archi animati + label dinamiche on hover/select
  container.style.cursor = 'grab';   // segnala panning all'utente (default canvas)
  return network;
}

// ---- Helper overlay: deduce label spec per un nodo ----
function overlayLabel(F, id){
  const w = F.wordMap[id];
  if(!w) return null;
  return { nodeId: id, text: w.displayName || id, color: colorForSig(w.sig) };
}

// Trace edges: gli archi del nodo, filtrati dalla direction corrente.
// Il colore dell'arco animato segue il colore della parola SORGENTE
// (e.from): l'arco "scorre" nel colore di chi lo emette.
function traceEdgesFor(F, word, direction){
  const out = [];
  for(const id of F.edgesForWordIds(word)){
    const e = F.edgeByKey[id];
    if(!e) continue;
    if(direction === 'out' && e.from !== word) continue;
    if(direction === 'in'  && e.to   !== word) continue;
    if(!isRelTypeEnabled(e.rel)) continue;  // filtro legenda interattiva
    const fromW = F.wordMap[e.from];
    const color = fromW ? colorForSig(fromW.sig) : UI.edgeHover;
    // group della relazione (S/C/M/F/L) per pattern dash coerente con legenda.
    const group = REL_GROUP[e.rel] || 'L';
    out.push({ from: e.from, to: e.to, color, group });
  }
  return out;
}

// Monta un field specifico nel network (swap di DataSets).
export function mountField(id, { fit = true, animate = true } = {}){
  const F = FIELDS[id];
  if(!F) return;
  setActive(id);
  network.setData({ nodes: F.nodesDS, edges: F.edgesDS });
  if(fit){
    const doFit = () => network.fit(animate ? { animation: { duration: tokens.anim.mountFit, easingFunction: 'easeInOutQuad' } } : {});
    setTimeout(doFit, 60);
  }
}

// Handlers iniettati dall'esterno (evita dipendenze circolari).
export function setHandlers(h){ _handlers = { ..._handlers, ...h }; }

// ---- Drag unificato: move oppure connect, deciso al rilascio --------------
// In nuovo/medio, qualunque drag (senza modificatori) di un singolo
// rettangolo ha due esiti possibili decisi al RILASCIO in base a dove
// sta il cursore:
//   - rilasciato sopra un altro rettangolo → CONNECT: il source torna
//     alla posizione originale, si apre il picker relazione. Il nodo
//     non si sposta — l'unico effetto è la creazione dell'arco.
//   - rilasciato in area vuota → MOVE: il rettangolo resta dove
//     l'utente l'ha portato, marca _userPositioned, persistito.
//
// Implementazione: vis-network gestisce nativamente il movimento
// visivo del rettangolo durante il drag (con dragNodes:true di
// default). Noi salviamo la posizione iniziale al `dragStart` e
// decidiamo il finale al `dragEnd`.
//
// Il drag su area vuota è il pan del canvas (dragView), gestito da vis.

let _dragOrigPos = null;     // { nodeId: {x,y} } — posizione iniziale dei nodi del drag corrente
let _dragWasSelected = null; // selezione attiva al dragStart, da ripristinare al dragEnd
let _isDragging = false;     // true fra dragStart e dragEnd — l'hover non deve perturbare il drag

// ---- Events ---------------------------------------------------------------

function _wireEvents(){
  const container = network.body.container;

  // Pan cursor: 'grabbing' durante drag su canvas vuota, 'grab' a riposo.
  // Indipendente dal drag-to-connect (che parte solo se mousedown su un nodo).
  container.addEventListener('mousedown', e => {
    if(e.button !== 0) return;
    const rect = container.getBoundingClientRect();
    const nid = network.getNodeAt({ x: e.clientX - rect.left, y: e.clientY - rect.top });
    if(!nid) container.style.cursor = 'grabbing';
  });
  container.addEventListener('mouseup', () => {
    if(container.style.cursor === 'grabbing') container.style.cursor = 'grab';
  });

  network.on('hoverNode', p => {
    // Durante il drag, l'hover non deve riapplicare highlight/dimming —
    // andrebbe a sovrascrivere lo stato "tutte le label visibili" che il
    // dragStart ha impostato per permettere di scegliere un target.
    if(_isDragging) return;
    if(String(p.node).startsWith('_')) return;
    const F = getActive(); if(!F) return;
    // Trail di esplorazione: l'hover su un vicino mostra la preview del
    // prossimo passo. Cablato come handler iniettato (vedi setHandlers).
    _handlers.onNodeHover?.(p.node);
    // In vasto la rosa è il vincolo di navigazione (focus sui vicini della
    // selezione). In nuovo/medio ogni nodo deve restare libero al click —
    // se la rosa è vuota (nodo senza archi), il vincolo bloccherebbe tutto.
    if(F.id === 'vasto' && F.selected && F.currentRosa && !F.currentRosa.has(p.node) && p.node !== F.selected) return;
    container.style.cursor = 'pointer';
    const direction = F.id === 'vasto' ? getFilterDirection() : 'both';
    if(!F.selected){
      applyHighlight(F, p.node, F.getRosa(p.node, direction, { filterByType: isRelTypeEnabled }));
      return;
    }
    // Sub-hover su un nodo della rosa: evidenzia le sue connessioni.
    if(F.currentRosa && (F.currentRosa.has(p.node) || p.node === F.selected)){
      if(p.node === F.selected) return;
      F.subHover = p.node;
      const subRosa = F.getRosa(p.node, direction, { filterByType: isRelTypeEnabled });
      const path = [...F.navPath, F.selected];
      const pathSet = new Set(path);
      const hover = F.wordMap[p.node];
      const batch = [];
      const layoutMode = (F.id === 'nuovo' && getNuovoLayout() === 'rectangular') ? 'rectangular' : undefined;
      const isChain = path.length > 1;
      if(hover) batch.push(buildNodeSpec(hover, 'active', { fieldId: F.id, layoutMode }));
      // Tutto il path resta 'active' anche durante il sub-hover, con shadow
      // verde di catena se path.length>1.
      for(const pw of path){
        const w = F.wordMap[pw];
        if(w) batch.push(buildNodeSpec(w, 'active', { fieldId: F.id, layoutMode, inPath: isChain }));
      }
      subRosa.forEach(ww => {
        if(pathSet.has(ww)) return;  // path word vince su rosa
        if(F.nodesDS.get(ww)){
          // Trova tratteggio arco
          let borderDashes = false;
          const edIds = F.edgesForWordIds([p.node]);
          for(const id of edIds) {
            const e = F.edgeByKey[id];
            if(e && (e.from === ww || e.to === ww)) {
              const group = REL_GROUP[e.rel] || null;
              if(group) {
                 borderDashes = GROUP_DASH[group] || false;
              }
              break;
            }
          }
          const w = F.wordMap[ww];
          if(w) batch.push(buildNodeSpec(w, 'rosa', { fieldId: F.id, layoutMode, borderDashes }));
        }
      });
      F.currentRosa.forEach(ww => {
        if(ww !== p.node && !subRosa.has(ww) && !pathSet.has(ww)){
          const w = F.wordMap[ww];
          if(w) batch.push(buildNodeSpec(w, 'dimmed', { fieldId: F.id, layoutMode }));
        }
      });
      F.nodesDS.update(batch);

      const dirFilter = (word) => (id) => {
        const e = F.edgeByKey[id];
        if(!e) return true;
        if(!isRelTypeEnabled(e.rel)) return false;  // filtro legenda
        if(direction === 'out') return e.from === word;
        if(direction === 'in')  return e.to   === word;
        return true;
      };
      const selIds   = F.edgesForWordIds(F.selected).filter(dirFilter(F.selected));
      const hoverIds = F.edgesForWordIds(p.node).filter(dirFilter(p.node));
      F.edgesDS.update(selIds.map(id => ({
        ...buildEdgeSpec(F.edgeByKey[id], { variant: 'subFocus', layoutMode }),
      })));
      F.edgesDS.update(hoverIds.map(id => ({
        ...buildEdgeSpec(F.edgeByKey[id], { variant: 'hoverSub', layoutMode }),
      })));

      // Overlay: archi animati SOLO della parola sotto il cursore (sub-hover).
      // La collega ha chiesto: "ho cliccato su 'confine' e sono in mouseover
      // su 'esterno'. Devo vedere solo tutti gli archi di esterno." Senza
      // questo isolamento il campo restava affollato dagli archi della
      // selezione precedente, rendendo difficile leggere le relazioni del
      // nodo che si sta esplorando.
      setActiveTraceEdges(traceEdgesFor(F, p.node, direction));
      const labels = [];
      const sl = overlayLabel(F, F.selected); if(sl) labels.push(sl);
      const hl = overlayLabel(F, p.node);     if(hl) labels.push(hl);
      subRosa.forEach(w => {
        if(w !== F.selected && w !== p.node){
          const l = overlayLabel(F, w);
          if(l) labels.push(l);
        }
      });
      setActiveLabels(labels);
    }
  });

  network.on('blurNode', () => {
    // Durante il drag, il blur non deve toccare lo stato visivo: il
    // dragStart ha già impostato "tutte le label visibili", lo stato
    // tornerà al normale via dragEnd → restoreHighlightIfNeeded.
    if(_isDragging) return;
    container.style.cursor = 'grab';
    _handlers.onNodeHover?.(null);
    const F = getActive(); if(!F) return;
    if(!F.selected) clearHighlight(F);
    else if(F.subHover) restoreSelection(F);
  });
  container.addEventListener('mouseleave', () => {
    if(_isDragging) return;
    _handlers.onNodeHover?.(null);
    const F = getActive(); if(!F) return;
    if(!F.selected) clearHighlight(F);
    else if(F.subHover) restoreSelection(F);
  });

  network.on('click', p => {
    if(_justDragged) return;
    const F = getActive(); if(!F) return;
    if(p.nodes.length > 0){
      const nid = p.nodes[0];
      if(String(nid).startsWith('_')) return;
      // Vincolo rosa solo in vasto (vedi commento in hoverNode).
      if(F.id === 'vasto' && F.selected && F.currentRosa && !F.currentRosa.has(nid) && nid !== F.selected) return;
      _handlers.onSelectWord?.(nid);
      return;
    }
    if(p.edges.length > 0){
      _handlers.onEditEdge?.(p.edges[0]);
      return;
    }
    if(F.selected) _handlers.onDeselect?.();
  });

  // dragStart: salva le posizioni iniziali dei nodi che stanno per essere
  // trascinati. Servono per (a) rimettere il source al suo posto in caso
  // di connect, (b) escludere il source dal getNodeAt al rilascio.
  // Inoltre, se c'è una selezione attiva con i nodi non-rosa in 'dimmed'
  // (label vuota), tolgo l'highlight per la durata del drag — l'utente
  // deve poter leggere TUTTE le label per scegliere a cosa connettersi.
  // Lo ripristino al dragEnd se non si è creata una connessione.
  network.on('dragStart', (params) => {
    if(!params.nodes || params.nodes.length === 0) return;
    const F = getActive();
    if(!F || F.id === 'vasto') return;
    const positions = network.getPositions(params.nodes);
    _dragOrigPos = {};
    for(const nid of params.nodes){
      if(String(nid).startsWith('_')) continue;
      const pos = positions[nid];
      if(pos) _dragOrigPos[nid] = { x: pos.x, y: pos.y };
    }
    // Il dimming si attiva sia su selezione (selectWord → applyHighlight)
    // sia su hover senza selezione (hoverNode → applyHighlight su un nodo
    // qualsiasi). Quindi controllo F.isDimmed indipendentemente da
    // F.selected: anche senza selezione attiva, le label dei nodi non-rosa
    // sono nascoste e vanno riportate visibili per il drag.
    if(F.isDimmed){
      _dragWasSelected = F.selected;  // può essere null (hover senza select)
      clearHighlight(F);
    }
    _isDragging = true;
  });

  // dragEnd: decide tra CONNECT (rilasciato sopra altro nodo) e MOVE
  // (rilasciato in area vuota). Ramo single-node only — il drag di gruppo
  // resta un puro move.
  network.on('dragEnd', (params) => {
    const F = getActive();
    const orig = _dragOrigPos; _dragOrigPos = null;
    const wasSel = _dragWasSelected; _dragWasSelected = null;
    _isDragging = false;
    // Helper: ripristina l'highlight della selezione precedente al drag.
    // Da chiamare a fine logica connect/move (ogni return path).
    const restoreHighlightIfNeeded = () => {
      if(!wasSel || !F || F.selected !== wasSel) return;
      const direction = F.id === 'vasto' ? getFilterDirection() : 'both';
      const rosa = F.getRosa(wasSel, direction, { filterByType: isRelTypeEnabled });
      applyHighlight(F, wasSel, rosa);
    };
    if(!F || F.id === 'vasto'){ restoreHighlightIfNeeded(); return; }
    if(!params.nodes || params.nodes.length === 0){ restoreHighlightIfNeeded(); return; }

    // Single-node drag in nuovo: connect vs move.
    if(params.nodes.length === 1 && params.pointer?.DOM){
      const source = params.nodes[0];
      if(!String(source).startsWith('_')){
        // Pos finale dove vis ha messo il nodo (rilascio mouse)
        const finalPositions = network.getPositions([source]);
        const finalPos = finalPositions[source];
        const origPos  = orig?.[source];

        // Per identificare il target sotto il cursore devo prima togliere
        // il source da lì: vis l'ha appena lasciato sotto il puntatore.
        // Lo riporto temporaneamente alla pos originale, leggo, decido.
        if(origPos){
          try { F.nodesDS.update({ id: source, x: origPos.x, y: origPos.y }); } catch(_){}
        }
        let target = network.getNodeAt(params.pointer.DOM);
        if(target === source) target = null;
        if(target && String(target).startsWith('_')) target = null;

        if(target){
          // CONNECT: source resta alla pos originale (già impostata),
          // sincronizza word.position e apre il picker relazione.
          // Niente restoreHighlight qui: onConnect aprirà la modale e,
          // a conferma utente, _onEdgeChanged riapplicherà il layout
          // (che tornerebbe a sovrascrivere comunque l'highlight).
          const w = F.wordMap?.[source];
          if(w?.position && origPos){
            w.position.x = origPos.x; w.position.y = origPos.y;
          }
          _handlers.onConnect?.(source, target);
          _justDragged = true;
          setTimeout(() => { _justDragged = false; }, 80);
          return;
        }

        // MOVE: rimetti il source dove l'utente l'ha lasciato (annullo
        // il ripristino temporaneo) e marca _userPositioned così i
        // layout non lo sovrascrivono al prossimo refresh.
        if(finalPos){
          try { F.nodesDS.update({ id: source, x: finalPos.x, y: finalPos.y }); } catch(_){}
          F.markPositionUser(source, finalPos.x, finalPos.y);
        }
        saveField(F.id);
        restoreHighlightIfNeeded();
        return;
      }
    }

    // Multi-node move (selezione di gruppo) o nodi speciali: comportamento
    // standard — registra le posizioni finali, niente connect.
    const positions = network.getPositions(params.nodes);
    let touched = false;
    for(const nid of params.nodes){
      if(String(nid).startsWith('_')) continue;
      const pos = positions[nid];
      if(!pos) continue;
      F.markPositionUser(nid, pos.x, pos.y);
      touched = true;
    }
    if(touched) saveField(F.id);
    restoreHighlightIfNeeded();
  });

  network.on('oncontext', p => {
    p.event.preventDefault();
    const F = getActive(); if(!F) return;
    const nid = network.getNodeAt(p.pointer.DOM);
    const screenX = p.event.clientX || p.pointer.DOM.x;
    const screenY = p.event.clientY || p.pointer.DOM.y;
    if(nid){
      if(String(nid).startsWith('_')){ _handlers.onCtxEdge?.(null, 0, 0); return; }
      // Vincolo rosa solo in vasto (vedi commento in hoverNode).
      if(F.id === 'vasto' && F.selected && F.currentRosa && !F.currentRosa.has(nid) && nid !== F.selected) return;
      // Right-click NON cambia la selezione: il breadcrumb (navPath) deve
      // restare intatto mentre l'utente apre il menu su un nodo qualsiasi.
      // Le voci del ctx-menu (modifica/elimina/aggiungi rel) operano sul
      // nodeId passato, non su F.selected — quindi nessuna regressione.
      _handlers.onCtxNode?.(nid, screenX, screenY);
    } else {
      const canvasPt = network.DOMtoCanvas(p.pointer.DOM);
      _handlers.onCtxEmpty?.(screenX, screenY, canvasPt.x, canvasPt.y);
    }
  });
}

// ---- Highlight/clear/restore ----------------------------------------------
// Funzioni pure che agiscono sul DataSet del Field passato.
//
// Se un filtro è attivo e la parola non passa, usa la variante 'filterDim'
// (visibile ma attenuata) — mai 'dimmed' standard, che nasconderebbe la
// parola e confonderebbe l'utente.

function nodeVariantFor(F, id, variantIfPasses, borderDashes = false, extraOpts = {}){
  const word = F.wordMap[id];
  if(!word) return null;

  // Assicurati che passiamo il layoutMode corrente ('rectangular' o 'dimensional')
  // così buildNodeSpec preserva la forma 'box' durante gli highlight/hover.
  const layoutMode = (F.id === 'nuovo' && getNuovoLayout() === 'rectangular') ? 'rectangular' : undefined;

  if(F.id === 'vasto' && isFilterActive() && !getMatchedWords().has(id)){
    return buildNodeSpec(word, 'filterDim', { fieldId: F.id, layoutMode, borderDashes });
  }
  return buildNodeSpec(word, variantIfPasses, { fieldId: F.id, layoutMode, borderDashes, ...extraOpts });
}

// Helper: archi visibili per un path. Restituisce due insiemi:
//   - chainEdgeIds: archi tra parole CONSECUTIVE del path (la catena vera).
//   - frontierEdgeIds: archi dall'ULTIMA parola del path verso le parole rosa.
// L'unione è ciò che va mostrato sul grafo. Tutti gli archi che puntano
// ad altre parole (storiche o sconnesse) restano hidden — così non vediamo
// più archi che terminano in "rettangoli vuoti" come segnalato dalla collega.
function computeVisibleEdgesForPath(F, path, rosa, direction){
  const chain = new Set();
  const frontier = new Set();
  // (a) catena: archi consecutivi
  for(let i = 0; i < path.length - 1; i++){
    const a = path[i], b = path[i + 1];
    for(const id of F.edgesForWordIds(a)){
      const e = F.edgeByKey[id];
      if(!e || !isRelTypeEnabled(e.rel)) continue;
      const other = e.from === a ? e.to : e.from;
      if(other === b) chain.add(id);
    }
  }
  // (b) frontiera: ultima parola → sua rosa
  if(path.length > 0){
    const last = path[path.length - 1];
    for(const id of F.edgesForWordIds(last)){
      const e = F.edgeByKey[id];
      if(!e || !isRelTypeEnabled(e.rel)) continue;
      if(direction === 'out' && e.from !== last) continue;
      if(direction === 'in'  && e.to   !== last) continue;
      const other = e.from === last ? e.to : e.from;
      if(rosa && rosa.has(other)) frontier.add(id);
    }
  }
  return { chain, frontier };
}

// Path-aware: deriva il percorso dal F.navPath + activeWord. Tutte le
// parole del path vengono renderizzate come 'active' (la collega: "vorrei
// che mi mantenessi attive tutte le parole su cui ho cliccato"). La rosa
// è quella dell'ULTIMA parola cliccata (ciò che si può cliccare dopo).
// Tutto il resto (parole non in path e non in rosa) resta dimmed.
//
// Empty-click → deselectWord → clear navPath → si torna a vista normale.
// Right-click su qualunque nodo: il context menu funziona indipendentemente
// dal path (vedi oncontext handler più sopra).
export function applyHighlight(F, activeWord, rosa){
  // Se c'è un filtro e activeWord non passa, ignora l'highlight.
  if(F.id === 'vasto' && isFilterActive() && !getMatchedWords().has(activeWord)) return;

  const path = [...F.navPath, activeWord];
  const pathSet = new Set(path);
  const isChain = path.length > 1;
  const layoutMode = (F.id === 'nuovo' && getNuovoLayout() === 'rectangular') ? 'rectangular' : undefined;

  const batch = [];
  // (1) Tutte le parole del path → 'active' con shadow verde se è una catena
  // (path.length>1). Una sola parola → 'active' standard.
  for(const pw of path){
    const spec = nodeVariantFor(F, pw, 'active', false, { inPath: isChain });
    if(spec) batch.push(spec);
  }
  // (2) Rosa dell'ultima parola → 'rosa' (escluse quelle già nel path)
  rosa.forEach(w => {
    if(pathSet.has(w)) return;
    let borderDashes = false;
    const eIds = F.edgesForWordIds([activeWord]);
    for(const id of eIds) {
      const e = F.edgeByKey[id];
      if(e && (e.from === w || e.to === w)) {
        const group = REL_GROUP[e.rel] || null;
        if(group) borderDashes = GROUP_DASH[group] || false;
        break;
      }
    }
    const s = nodeVariantFor(F, w, 'rosa', borderDashes);
    if(s) batch.push(s);
  });
  // (3) Tutto il resto → dimmed
  F.nodesDS.forEach(n => {
    if(String(n.id).startsWith('_')) return;
    if(pathSet.has(n.id) || rosa.has(n.id)) return;
    const s = nodeVariantFor(F, n.id, 'dimmed');
    if(s) batch.push(s);
  });
  F.nodesDS.update(batch);

  // (4) Archi visibili: SOLO catena (consecutivi nel path) + frontiera
  // (ultima parola → rosa). Niente archi verso parole storiche dimmed —
  // altrimenti vediamo "archi che puntano a rettangoli vuoti".
  const direction = F.id === 'vasto' ? getFilterDirection() : 'both';
  const { chain, frontier } = computeVisibleEdgesForPath(F, path, rosa, direction);
  const edgeBatch = [];
  F.edgesDS.forEach(eRow => {
    const id = eRow.id;
    const e = F.edgeByKey[id];
    if(!e){ edgeBatch.push({ id, hidden: true }); return; }
    if(chain.has(id)){
      // Archi della catena: variant 'selection' (più marcato del hover).
      edgeBatch.push(buildEdgeSpec(e, { variant: 'selection', layoutMode, fieldId: F.id }));
    } else if(frontier.has(id)){
      edgeBatch.push(buildEdgeSpec(e, { variant: 'hover', layoutMode, fieldId: F.id }));
    } else {
      edgeBatch.push({ id, hidden: true });
    }
  });
  if(edgeBatch.length) F.edgesDS.update(edgeBatch);

  F.currentRosa = rosa;
  F.isDimmed = true;

  // Overlay: archi animati SOLO catena+frontiera (coerente con quanto sopra).
  const overlayEdges = [];
  // Archi catena: per ogni coppia consecutiva, animazione "fluisce" tra di loro.
  for(let i = 0; i < path.length - 1; i++){
    const a = path[i], b = path[i + 1];
    const fromW = F.wordMap[a];
    overlayEdges.push({
      from: a, to: b,
      color: fromW ? colorForSig(fromW.sig) : UI.edgeHover,
      group: 'C',
    });
  }
  // Archi frontiera: ultima → rosa (riusa traceEdgesFor che già filtra).
  if(path.length){
    overlayEdges.push(...traceEdgesFor(F, path[path.length - 1], direction));
  }
  setActiveTraceEdges(overlayEdges);
  // Label visibili: solo parole del path + rosa attuale (le storiche dimmed
  // non mostrano label).
  const labels = [];
  for(const pw of path){ const l = overlayLabel(F, pw); if(l) labels.push(l); }
  rosa.forEach(w => {
    if(pathSet.has(w)) return;
    const l = overlayLabel(F, w); if(l) labels.push(l);
  });
  setActiveLabels(labels);
}

export function clearHighlight(F){
  const batch = [];
  F.nodesDS.forEach(n => {
    if(String(n.id).startsWith('_')) return;
    const s = nodeVariantFor(F, n.id, 'normal');
    if(s) batch.push(s);
  });
  F.nodesDS.update(batch);

  const allowed = getAllowedEdges();
  if(isFilterActive() && allowed){
    const eBatch = [];
    F.edgesDS.forEach(e => {
      if(allowed.has(e.id)){
        eBatch.push(buildEdgeSpec(F.edgeByKey[e.id], { variant: 'filterShown', layoutMode: F.id === 'nuovo' && window._getNuovoLayout ? window._getNuovoLayout() : undefined }));
      } else {
        eBatch.push({ id: e.id, hidden: true });
      }
    });
    if(eBatch.length) F.edgesDS.update(eBatch);
  } else {
    // Baseline: tutti gli archi al variant 'normal' (nascosti).
    const baseline = F.baselineEdgeBatch();
    if(baseline.length) F.edgesDS.update(baseline);
  }

  F.currentRosa = null;
  F.isDimmed = false;
  F.subHover = null;
  clearOverlay();
}

// Path-aware: dopo un sub-hover (mouse esce dal vicino) ripristina la
// vista path con tutte le parole del navPath + selected come 'active'.
// Stesso edge filter di applyHighlight: solo catena + frontiera.
export function restoreSelection(F){
  if(!F.selected || !F.currentRosa) return;
  const path = [...F.navPath, F.selected];
  const pathSet = new Set(path);
  const isChain = path.length > 1;
  const layoutMode = (F.id === 'nuovo' && getNuovoLayout() === 'rectangular') ? 'rectangular' : undefined;
  const batch = [];

  // Path → 'active' (con shadow verde se catena)
  for(const pw of path){
    const w = F.wordMap[pw];
    if(w) batch.push(buildNodeSpec(w, 'active', { fieldId: F.id, layoutMode, inPath: isChain }));
  }
  // Rosa dell'ultima → 'rosa' (escluse parole path)
  F.currentRosa.forEach(w => {
    if(pathSet.has(w)) return;
    let borderDashes = false;
    const eIds = F.edgesForWordIds([F.selected]);
    for(const id of eIds) {
      const e = F.edgeByKey[id];
      if(e && (e.from === w || e.to === w)) {
        const group = REL_GROUP[e.rel] || null;
        if(group) borderDashes = GROUP_DASH[group] || false;
        break;
      }
    }
    const word = F.wordMap[w];
    if(word) batch.push(buildNodeSpec(word, 'rosa', { fieldId: F.id, layoutMode, borderDashes }));
  });
  // Resto → dimmed
  F.nodesDS.forEach(n => {
    if(String(n.id).startsWith('_')) return;
    if(pathSet.has(n.id) || F.currentRosa.has(n.id)) return;
    const s = nodeVariantFor(F, n.id, 'dimmed');
    if(s) batch.push(s);
  });
  F.nodesDS.update(batch);

  // Archi: catena + frontiera (stesso filtro di applyHighlight).
  const direction = F.id === 'vasto' ? getFilterDirection() : 'both';
  const { chain, frontier } = computeVisibleEdgesForPath(F, path, F.currentRosa, direction);
  const edgeBatch = [];
  F.edgesDS.forEach(eRow => {
    const id = eRow.id;
    const e = F.edgeByKey[id];
    if(!e){ edgeBatch.push({ id, hidden: true }); return; }
    if(chain.has(id)){
      edgeBatch.push(buildEdgeSpec(e, { variant: 'selection', layoutMode, fieldId: F.id }));
    } else if(frontier.has(id)){
      edgeBatch.push(buildEdgeSpec(e, { variant: 'selection', layoutMode, fieldId: F.id }));
    } else {
      edgeBatch.push({ id, hidden: true });
    }
  });
  if(edgeBatch.length) F.edgesDS.update(edgeBatch);
  F.subHover = null;

  // Overlay: archi animati catena + frontiera (no archi storici).
  const overlayEdges = [];
  for(let i = 0; i < path.length - 1; i++){
    const a = path[i], b = path[i + 1];
    const fromW = F.wordMap[a];
    overlayEdges.push({
      from: a, to: b,
      color: fromW ? colorForSig(fromW.sig) : UI.edgeHover,
      group: 'C',
    });
  }
  if(path.length){
    overlayEdges.push(...traceEdgesFor(F, path[path.length - 1], direction));
  }
  setActiveTraceEdges(overlayEdges);

  const labels = [];
  for(const pw of path){ const l = overlayLabel(F, pw); if(l) labels.push(l); }
  F.currentRosa.forEach(w => {
    if(pathSet.has(w)) return;
    const l = overlayLabel(F, w); if(l) labels.push(l);
  });
  setActiveLabels(labels);
}
