// Switch tra i due campi (vasto, personale): mount, overlay di feedback,
// hint di campo vuoto, azioni contestuali, cancellazione campo.

import { LS, REL_GROUP } from '../constants.js';
import { tokens } from '../theme.js';
import { FIELDS, activeId, setActive, saveField, registerField } from '../manager.js';
import { Field } from '../field.js';
import { mountField, network } from '../graph.js';
import { clearInfo, renderBreadcrumb, updateModeIndicators, updateStats } from '../sidebar.js';
import { openAddWord } from '../editor.js';
import { isActive as filterIsActive, apply as reapplyFilters } from '../filters.js';
// Animazione "lettura → espansione semantica" (vedi roadmap UX §2).
import { startExpandAnimation, stopExpandAnimation } from '../components/expand-animation.js';
import {
  openSentencePanel, fetchUnderstandingAndRender, hideUnderstandingPanel,
  setSentencePanelCallbacks,
} from './sentence-panel.js';
import { updateTransmitButton, doTransmit } from './transmit.js';
import { pendingCount } from '../manager.js';
import { openSaveModal, openLoadModal, setModalCallbacks } from './modals.js';
import { openCtxMenu } from '../components/ctx-menu.js';
import { openConfirm, openChoice } from '../components/confirm-panel.js';
import { openPanel, closePanel } from '../components/edit-panel.js';
import { openExtractDialog } from '../components/extract-dialog.js';
import { refreshGraphToolbar } from '../components/graph-toolbar.js';
import { addSentenceToNuovo, commitExpansion } from '../sentence.js';
import { extractRelationsForField } from '../relations-extract.js';
import { getNuovoLayout, setNuovoLayout, setLinkMode, clearLinkFocus } from '../ui-state.js';
import { applyRectangularLayout, applyDimensionalLayout } from '../layouts/rectangular.js';
import { setDimOverlayVisible } from '../components/dim-overlay.js';
import { renderTrail as _renderExplorationTrail } from '../components/exploration-trail.js';

// ---- Remount / switch -----------------------------------------------------

// Imposta body.campo-{id} per il gating CSS di sezioni vincolate al campo
// (es. #filtri visibile solo in vasto). Lo togglo qui invece che a livello
// di setActive per tenere la responsabilità nello switching UI.
function setCampoBodyClass(id){
  const cls = document.body.classList;
  ['campo-vasto', 'campo-nuovo'].forEach(c => cls.remove(c));
  cls.add('campo-' + id);
}

// Modalità "comprensione frase": personale CON frase E animazione iniziale
// non ancora vista → il NUOVO è montato (popolato con clone di tutto il
// vasto da buildNuovo). L'animazione canvas sopra evidenzia ciascuna
// parola della frase + rose; in fase fadeout, le parole vasto-clone non
// rilevanti svaniscono. Al completamento (uir1:expansion-completed)
// commitExpansion() pota il personale tenendo solo frase + vicini.
function _isComprehensionMode(id){
  const P = FIELDS.nuovo;
  return id === 'nuovo' && !!P?.sentence && !P?.expansionShown;
}

function _enterComprehension(){
  document.body.classList.add('comprensione-frase');
}
function _exitComprehension(){
  document.body.classList.remove('comprensione-frase');
}

// Rimonta un field già attivo (es. dopo buildNuovo o load dal server).
export function remountActive(id){
  setActive(id);
  setCampoBodyClass(id);
  if(_isComprehensionMode(id)){
    // Il personale stesso fa da sfondo (opacity ridotta via comprensione-frase).
    // Il personale è montato (popolato con clone vasto + frase). L'animazione
    // canvas sopra evidenzia frase+rose, fadeout fa svanire le parole vasto
    // non rilevanti, commitExpansion pota a fine animazione.
    mountField('nuovo', { fit: true, animate: false });
    _enterComprehension();
  } else {
    _exitComprehension();
    mountField(id, { fit: true });
    if(id === 'nuovo') applyNuovoLayout();
    else setDimOverlayVisible(true);  // vasto: bussola sempre visibile
  }
  clearInfo();
  renderBreadcrumb();
  updateModeIndicators();
  refreshFieldUI();
  if(_isComprehensionMode(id)){
    fetchUnderstandingAndRender(FIELDS.nuovo.sentence);
    setTimeout(() => startExpandAnimation(FIELDS.nuovo, document.getElementById('graph')), 300);
  } else {
    hideUnderstandingPanel();
    stopExpandAnimation();
  }
}

export function switchMode(id){
  if(activeId === id){
    remountActive(id);
    return;
  }
  // Reset stati di interazione transitori che non hanno senso al cambio
  // campo (linkMode/linkFocus sono nuovo-only).
  setLinkMode(false);
  clearLinkFocus();
  // Se stiamo uscendo dalla comprensione frase prima della fine
  // dell'animazione, finalizzala subito (commitExpansion pota i nodi
  // vasto-clone). Evita di lasciare il personale gonfio in localStorage.
  if(activeId === 'nuovo' && _isComprehensionMode('nuovo')){
    commitExpansion();
    stopExpandAnimation();
    hideUnderstandingPanel();
  }
  const prev = FIELDS[activeId];
  if(prev){
    prev.selected = null;
    prev.subHover = null;
    prev.currentRosa = null;
    prev.isDimmed = false;
  }
  const btnHome = document.getElementById('btn-home');
  if(btnHome) btnHome.style.display = 'none';
  document.body.classList.remove('parola-selezionata');
  setCampoBodyClass(id);

  // In comprensione-frase montiamo il personale stesso come sfondo (opacity
  // ridotta via comprensione-frase); altrimenti il field corrispondente all'id.
  if(_isComprehensionMode(id)){
    // Il personale è montato (popolato con clone vasto + frase). L'animazione
    // canvas sopra evidenzia frase+rose, fadeout fa svanire le parole vasto
    // non rilevanti, commitExpansion pota a fine animazione.
    mountField('nuovo', { fit: true, animate: false });
    _enterComprehension();
  } else {
    _exitComprehension();
    mountField(id, { fit: true });
    if(id === 'nuovo') applyNuovoLayout();
    else setDimOverlayVisible(true);  // vasto: bussola sempre visibile
  }
  clearInfo();
  renderBreadcrumb();
  showModeOverlay(id);
  updateModeIndicators();
  refreshFieldUI();

  if(_isComprehensionMode(id)){
    fetchUnderstandingAndRender(FIELDS.nuovo.sentence);
    setTimeout(() => startExpandAnimation(FIELDS.nuovo, document.getElementById('graph')), 400);
  } else {
    hideUnderstandingPanel();
    stopExpandAnimation();
  }

  // #filtri ora è permanente in vasto (CSS lo nasconde negli altri campi).
  // Riapplica i filtri se attivi quando si torna su vasto.
  if(id === 'vasto' && filterIsActive()) reapplyFilters();
}

function showModeOverlay(id){
  const label = id === 'vasto' ? 'campo vasto' : 'campo nuovo';
  document.getElementById('mode-overlay')?.remove();
  const ov = document.createElement('div');
  ov.id = 'mode-overlay';
  ov.textContent = label;
  document.body.appendChild(ov);
  requestAnimationFrame(() => ov.classList.add('show'));
  setTimeout(() => {
    ov.classList.remove('show');
    setTimeout(() => ov.remove(), tokens.anim.overlayFade);
  }, tokens.anim.overlayShow);
}

// ---- Refresh UI del field corrente ----------------------------------------

export function refreshFieldUI(){
  updateStats();
  updateTransmitButton();
  updateEmptyHint();
  updateOverlayCtaVisibility();
  updateTabPersonaleVisibility();
  updateViewActions();
  refreshGraphToolbar();
  // Trail di esplorazione: ridisegna anche al cambio campo (F.selected
  // diverso fra vasto e nuovo, navPath idem). Import lazy via dynamic
  // avrebbe complicato — qui è un import diretto perfettamente safe.
  _renderExplorationTrail();
}

// La tab del nuovo cambia label e stato in base al contenuto:
//   - vuoto:    classe "empty",  label "crea il tuo campo"
//   - popolato: classe normale,  label "nuovo"
// Click handler intelligente in buttons.js: se vuoto apre il pannello frase,
// altrimenti switch al campo nuovo.
function updateTabPersonaleVisibility(){
  const tab = document.querySelector('button.tab[data-view="nuovo"]');
  if(!tab) return;
  const P = FIELDS.nuovo;
  const hasContent = P && (P.words.length > 0 || P.sentence);
  tab.style.display = '';  // sempre visibile
  if(hasContent){
    tab.classList.remove('empty');
    tab.textContent = 'nuovo';
    tab.title = 'il tuo campo';
  } else {
    tab.classList.add('empty');
    tab.textContent = 'crea il tuo campo';
    tab.title = 'crea un campo nuovo da una frase';
  }
}

function updateEmptyHint(){
  document.getElementById('empty-hint')?.remove();
  const F = FIELDS[activeId];
  if(!F || F.id === 'vasto' || F.words.length > 0) return;
  const hint = document.createElement('div');
  hint.id = 'empty-hint';
  hint.innerHTML = 'Campo vuoto — premi con il <strong>tasto destro del mouse</strong> per iniziare,'
    + ' oppure usa <strong>✎ crea da frase</strong> qui sopra.<br>'
    + '<span class="hint-sub">Trascina per spostare un rettangolo. <strong>Shift+drag</strong> per collegare.</span>';
  document.getElementById('graph').appendChild(hint);
}

// Bottoni overlay "crea il tuo campo" e "carica un campo": visibili solo
// nel vasto, nascosti quando l'utente seleziona una parola (declutter Fase B).
function updateOverlayCtaVisibility(){
  const wrap = document.getElementById('vastoOverlayActions');
  if(!wrap) return;
  const inVasto = activeId === 'vasto';
  const wordSelected = document.body.classList.contains('parola-selezionata');
  wrap.style.display = (inVasto && !wordSelected) ? '' : 'none';
}

function updateViewActions(){
  const host = document.getElementById('viewActions');
  if(!host) return;
  host.innerHTML = '';

  // In vasto: solo "carica un campo" (riprendere un campo salvato sul server).
  if(activeId === 'vasto'){
    host.appendChild(mkViewBtn('📂 carica un campo', '', () => openLoadModal('nuovo')));
    return;
  }
  if(activeId !== 'nuovo') return;
  const F = FIELDS.nuovo;
  const hasContent = F && F.words && F.words.length > 0;

  // Campo nuovo VUOTO → "crea da frase" (entry point per costruirlo).
  // Campo nuovo POPOLATO → toggle layout (dimensioni / relazioni). "Crea da frase"
  // sparisce: il campo è già lì e creare di nuovo lo cancellerebbe.
  if(!hasContent){
    host.appendChild(mkViewBtn('✎ crea da frase', 'primary', () => openSentencePanel()));
  } else {
    host.appendChild(mkLayoutToggle());
  }
  host.appendChild(mkViewBtn('💾 salva',  '', () => openSaveModal('nuovo')));
  host.appendChild(mkViewBtn('📂 carica', '', () => openLoadModal('nuovo')));
  if(hasContent){
    host.appendChild(mkViewBtn('svuota campo', 'danger', () => clearNuovoWithPrompt()));
  }
}

// Wrap di clearNuovo che chiede prima all'utente se vuole trasmettere il
// lavoro pendente al campo vasto. Tre opzioni:
//   - "trasmetti e svuota" → doTransmit() poi clearNuovo()
//   - "svuota e basta"      → clearNuovo() diretto
//   - "annulla"             → no-op
// Se non c'è nulla di pendente, salta la domanda e svuota direttamente.
async function clearNuovoWithPrompt(){
  const pending = pendingCount();
  if(pending === 0){
    // Niente da perdere: conferma minima per evitare svuotamenti accidentali.
    const ok = await openConfirm({
      title: 'Svuotare il campo nuovo?',
      message: 'Tutte le parole e gli archi del tuo campo verranno rimossi.',
      confirmLabel: 'svuota',
      cancelLabel: 'annulla',
      danger: true,
    });
    if(!ok) return;
    clearNuovo();
    return;
  }
  // Tre opzioni esplicite: trasmetti+svuota / svuota / annulla. La modale
  // interna openChoice rimpiazza il confirm() nativo a doppia conferma.
  const word = pending === 1 ? 'elemento' : 'elementi';
  const adj  = pending === 1 ? 'trasmesso' : 'trasmessi';
  const choice = await openChoice({
    title: `Hai ${pending} ${word} non ${adj}`,
    message: 'Cosa vuoi fare prima di svuotare il campo?',
    choices: [
      { key: 'transmit', label: 'trasmetti al vasto e poi svuota', primary: true },
      { key: 'discard',  label: 'svuota senza trasmettere', danger: true },
      { key: 'cancel',   label: 'annulla' },
    ],
  });
  if(!choice || choice === 'cancel') return;
  if(choice === 'transmit'){
    try { await doTransmit(); } catch(e){ console.warn('[clearNuovoWithPrompt] transmit failed:', e); }
  } else if(choice === 'discard'){
    // Conferma extra per "discard": l'azione è irreversibile e l'utente
    // sta scegliendo di scartare lavoro che il vasto non vedrà mai.
    const sure = await openConfirm({
      title: 'Confermi?',
      message: `Stai per svuotare il campo SENZA trasmettere ${pending} ${word}.\nQuesto lavoro andrà perso.`,
      confirmLabel: 'svuota senza trasmettere',
      cancelLabel: 'annulla',
      danger: true,
    });
    if(!sure) return;
  }
  clearNuovo();
}

function mkViewBtn(label, variant, onClick){
  const b = document.createElement('button');
  b.className = 'view-action-btn' + (variant ? ' ' + variant : '');
  b.textContent = label;
  b.addEventListener('click', onClick);
  return b;
}

// Segmented control "dimensioni / relazioni" — sceglie il layout del campo
// nuovo. Stato corrente da ui-state, persistito in localStorage.
function mkLayoutToggle(){
  const wrap = document.createElement('div');
  wrap.className = 'view-action-toggle';
  const cur = getNuovoLayout();
  const mk = (mode, label, title) => {
    const b = document.createElement('button');
    b.className = 'view-action-btn toggle-seg' + (cur === mode ? ' attivo' : '');
    b.textContent = label;
    b.title = title;
    b.addEventListener('click', () => {
      if(getNuovoLayout() === mode) return;
      setNuovoLayout(mode);
      applyNuovoLayout();
      updateViewActions();
    });
    return b;
  };
  wrap.appendChild(mk('dimensional', '◐ dimensioni', 'parole posizionate dalle 8 dimensioni'));
  wrap.appendChild(mk('rectangular', '▦ relazioni',  'parole-frase in alto, vicini in fasce per gruppo'));
  return wrap;
}

// Applica al campo nuovo il layout corrente (dimensional/rectangular). Usato
// dal toggle e al mount del campo (dopo registerField/load).
export function applyNuovoLayout(){
  const F = FIELDS.nuovo;
  if(!F) return;
  const isRect = getNuovoLayout() === 'rectangular';
  if(isRect){
    applyRectangularLayout(F);
  } else {
    applyDimensionalLayout(F);
  }
  // Dim-overlay HTML: in rectangular le firme 8D non determinano la
  // posizione — la bussola "POTERE/MATERIA/..." è fuorviante e va via.
  if(activeId === 'nuovo') setDimOverlayVisible(!isRect);
  if(network && activeId === 'nuovo'){
    try { network.fit({ animation: false }); } catch(_){}
  }
}

// ---- Menu contestuale su area vuota (solo personale) ----------------------
// Voci attive: "+ aggiungi parola" e "+ aggiungi frase".

export function openEmptyCtxMenu(screenX, screenY, canvasX, canvasY){
  if(activeId !== 'nuovo') return;
  const items = [
    {
      kind: 'item', label: '+ aggiungi parola', action: 'new-word',
      onClick: () => openAddWord({ position: { x: canvasX, y: canvasY } }),
    },
    {
      kind: 'item', label: '+ aggiungi frase', action: 'new-sentence',
      onClick: () => openAddSentencePanel(),
    },
  ];
  // Estrazione di massa: solo se ci sono parole nel personale.
  const P = FIELDS.nuovo;
  if(P && P.words.length){
    items.push({
      kind: 'item', label: '⤓ estrai tutte le relazioni dal KG', action: 'extract-all',
      onClick: () => openExtractDialog({ kind: 'field' }, async (allowedGroups) => {
        await extractRelationsForField(P, { allowedGroups });
        // Spread aggressivo: l'estrazione massiva può aggiungere centinaia
        // di parole tutte concentrate nelle stesse regioni semantiche 8D.
        P.spreadNonOverlapping({ minDist: 100, iterations: 150 });
        saveField('nuovo');
        // Riapplica il layout corrente: senza questa chiamata, in modalità
        // rectangular le nuove parole restano puntini con posizioni 8D
        // finché un reload della pagina non rigenera le spec con il
        // layoutMode corretto. Vedi anche editor.js#extract-rel.
        applyNuovoLayout();
        refreshFieldUI();
      }),
    });
  }
  if(P && P.edges.length){
    items.push({
      kind: 'item', label: '🗙 cancella relazioni filtrate', action: 'delete-rels',
      onClick: () => openExtractDialog({ kind: 'field' }, async (allowedGroups) => {
        deleteRelationsForField(P, allowedGroups);
        saveField('nuovo');
        refreshFieldUI();
      }, { mode: 'delete' }),
    });
  }
  openCtxMenu({ x: screenX, y: screenY, items });
}

// Rimuove dal field gli archi il cui gruppo (REL_GROUP[edge.rel]) è in
// `allowedGroups`, e poi rimuove anche le parole-target rimaste orfane
// (senza più archi e non appartenenti alla frase originale). Le parole
// della frase (flags.fromSentence) sono SEMPRE preservate, anche se restano
// isolate.
function deleteRelationsForField(F, allowedGroups){
  if(!F || !allowedGroups || !allowedGroups.size) return;
  const edgesToRemove = [];
  for(const e of F.edges){
    const group = REL_GROUP[e.rel] || 'L';
    if(allowedGroups.has(group)) edgesToRemove.push(e.key);
  }
  for(const key of edgesToRemove) F.removeEdge(key);

  // Pulisci le parole orfane: nessun arco residuo + non appartenenti alla
  // frase. Una parola "fromSentence" resta nel campo anche senza archi.
  const wordsToRemove = [];
  for(const w of F.words){
    if(w.flags?.fromSentence) continue;
    const localDeg = (F.edgesByWord[w.w] || new Set()).size;
    if(localDeg === 0) wordsToRemove.push(w.w);
  }
  for(const word of wordsToRemove) F.removeWord(word);
}

// Mini-pannello: scrivi una frase, viene interpretata via /api/medio e i
// suoi punti vengono aggiunti al personale. Le posizioni derivano dalle
// firme 8D (placeByRank) — la frase non si "ancora al click", ogni parola
// va dove la sua firma la chiama.
function openAddSentencePanel(){
  openPanel({
    title: 'Aggiungi frase al personale',
    build: (body) => {
      body.innerHTML = `
        <textarea id="addSentenceInput" rows="3" class="edit-input"
                  placeholder="scrivi una frase..."
                  style="width:100%;resize:vertical"></textarea>
        <div class="sentence-hint" style="margin-top:8px;font-size:11px;opacity:0.75">
          Le parole della frase vengono piazzate qui: quelle conosciute prendono
          la firma del campo vasto, le altre una firma neutra (potrai modificarla).
        </div>
      `;
      setTimeout(() => body.querySelector('#addSentenceInput')?.focus(), 50);
    },
    actions: [
      { label: 'aggiungi', primary: true, onClick: async () => {
          const text = document.getElementById('addSentenceInput').value.trim();
          if(!text){ closePanel(); return; }
          const btnHost = document.querySelector('#editPanel .edit-actions .primary');
          if(btnHost){ btnHost.disabled = true; btnHost.textContent = 'interpreto…'; }
          try {
            await addSentenceToNuovo(text);
            closePanel();
          } catch(e){
            console.error('addSentenceToNuovo:', e);
            if(btnHost){ btnHost.disabled = false; btnHost.textContent = 'aggiungi'; }
            return;
          }
          // refreshFieldUI rimuove anche il "campo vuoto" hint via updateEmptyHint.
          refreshFieldUI();
        } },
      { label: 'annulla', onClick: closePanel },
    ],
  });
}

// ---- Cancellazione campo nuovo ----------------------------------------

export function clearNuovo(){
  const wasActive = activeId === 'nuovo';
  const V = FIELDS.vasto;
  try { localStorage.removeItem(LS.NUOVO); } catch(_){}
  if(V){
    const P = new Field('nuovo', V.frame);
    P.addDimLabels();
    registerField('nuovo', P);
  }
  stopExpandAnimation();
  hideUnderstandingPanel();
  if(wasActive){
    // Dopo cancellazione, riportiamo l'utente al vasto: il campo nuovo è
    // vuoto e non ha senso mostrarlo. La tab "crea il tuo campo" ricomparirà
    // automaticamente in posizione del nuovo.
    switchMode('vasto');
  } else {
    refreshFieldUI();
  }
}

// ---- Inizializzazione: registra i callback iniettati ----------------------

export function initViewSwitcherWiring(){
  setSentencePanelCallbacks({
    onNuovoCreated: () => remountActive('nuovo'),
  });
  setModalCallbacks({
    // switchMode (non remountActive) gestisce correttamente la transizione
    // anche quando l'utente è ancora nel vasto (caricamento iniziale): reset
    // della selezione precedente, body class campo-{id}, fit del network.
    onFieldLoaded: (id) => switchMode(id),
  });

  // Custom event per "azzera personale" emesso da sentence-panel (evita import
  // circolare panel→view-switcher→panel).
  window.addEventListener('uir1:clear-nuovo', () => clearNuovo());

  // Bottone overlay sul canvas vasto: carica un campo salvato sul server.
  // La creazione di un campo nuovo è ora la tab "crea il tuo campo" in sidebar.
  const ctaCarica = document.getElementById('caricaCampoOverlay');
  if(ctaCarica) ctaCarica.addEventListener('click', () => openLoadModal());

  // Custom event emesso da expand-animation.js quando l'utente clicca un
  // nodo del vasto sotto l'overlay (in modalità comprensione frase): si
  // esce dalla comprensione passando a vasto e si seleziona la parola.
  window.addEventListener('uir1:open-vasto-word', async (e) => {
    const word = e.detail?.word;
    if(!word) return;
    switchMode('vasto');
    // Aspetta che vasto sia pronto e la parola sia individuabile
    const { selectWord } = await import('./selection.js');
    setTimeout(() => selectWord(word), 200);
  });

  // Custom event emesso da expand-animation.js al termine del fadeout:
  // l'animazione ha potato visivamente le parole non rilevanti (a opacity 0).
  // commitExpansion rimuove davvero le parole vasto-clone dal personale e
  // setta expansionShown=true. Poi remount per uscire dal CSS comprensione.
  window.addEventListener('uir1:expansion-completed', () => {
    if(!FIELDS.nuovo) return;
    commitExpansion();
    stopExpandAnimation();
    hideUnderstandingPanel();
    if(activeId === 'nuovo') remountActive('nuovo');
  });
}

// ---- Helper di fit per "home button" --------------------------------------
export function fitCurrent(){
  try { network?.fit({ animation: true }); } catch(_){}
}
