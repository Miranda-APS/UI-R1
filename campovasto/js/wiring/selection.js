// Selezione e deselezione di una parola nel field attivo.
// Orchestratore delle chiamate a graph (highlight), sidebar (info/radar),
// breadcrumb, stats.

import { NEUTRAL_SIG } from '../constants.js';
import { computeOctalysis } from '../geometry.js';
import { getActive } from '../manager.js';
import { applyHighlight, clearHighlight } from '../graph.js';
import { showDimBars, showInfo, clearInfo, renderBreadcrumb, updateStats } from '../sidebar.js';
import { drawRadar } from '../editor.js';
import { isActive as filterIsActive, apply as reapplyFilters } from '../filters.js';
import { getFilterDirection, isRelTypeEnabled, getLinkFocus, clearLinkFocus, setLinkMode, isLinkMode } from '../ui-state.js';
import { renderTrail, setTrailHover } from '../components/exploration-trail.js';

// Cache di vicini pre-fetchati per parola (per sidebar / radar se serve).
const nbrCache = {};

export async function selectWord(word){
  const F = getActive();
  if(!F.hasWord(word)) return;
  if(word === F.selected) return;

  const idxInPath = F.navPath.indexOf(word);
  if(idxInPath >= 0){
    // Click su una parola già nel path: tronca lì (back-navigation).
    F.navPath = F.navPath.slice(0, idxInPath);
  } else if(F.selected){
    // Vasto: la catena è esplorazione strutturata attraverso le
    // connessioni — accumula solo se la nuova parola è nella rosa
    // (vicino diretto della selezione corrente).
    // Nuovo: la catena è lo storico dei click. L'utente vuole vedere
    // tutte le parole su cui è passato evidenziate, anche se non
    // sono connesse direttamente fra loro nel grafo.
    if(F.id === 'vasto'){
      if(F.currentRosa && F.currentRosa.has(word)){
        F.navPath.push(F.selected);
      } else {
        F.navPath = [];
      }
    } else {
      F.navPath.push(F.selected);
    }
  } else {
    F.navPath = [];
  }

  if(F.selected) clearHighlight(F);
  F.subHover = null;
  F.selected = word;
  renderBreadcrumb();

  const w = F.wordMap[word];
  if(w && !w.userOct) w.userOct = computeOctalysis(w.sig || NEUTRAL_SIG).slice();

  const direction = F.id === 'vasto' ? getFilterDirection() : 'both';
  const rosa = F.getRosa(word, direction, { filterByType: isRelTypeEnabled });
  applyHighlight(F, word, rosa);

  const btnHome = document.getElementById('btn-home');
  if(btnHome) btnHome.style.display = 'inline-flex';

  if(F.id === 'vasto' && !nbrCache[word]){
    try {
      const r = await fetch('/api/biennale/word?word=' + encodeURIComponent(word));
      if(r.ok) nbrCache[word] = await r.json();
    } catch(_){}
  }

  const sig = (w && w.sig) ? w.sig : NEUTRAL_SIG;
  // showInfo apre la .box.parola (.attivo) e gestisce body.parola-selezionata.
  // dim-bars + radar sono ora dentro la card, non hanno più wrapper toggleabili.
  showInfo(word);
  showDimBars(sig);
  drawRadar(sig);
  updateStats();
  // Trail in alto a destra: la nuova selezione (e il navPath troncato/esteso)
  // è già nello stato del Field — basta ridisegnare. L'hover viene azzerato
  // perché il click sostituisce il candidato in preview.
  setTrailHover(null);
}

export function deselectWord(){
  const F = getActive();
  F.selected = null;
  F.navPath = [];
  renderBreadcrumb();
  clearHighlight(F);
  if(F.id === 'vasto' && filterIsActive()) reapplyFilters();
  const btnHome = document.getElementById('btn-home');
  if(btnHome) btnHome.style.display = 'none';
  clearInfo();
  document.body.classList.remove('parola-selezionata');
  updateStats();
  // Empty click chiude anche linkFocus / linkMode: chi clicca su area vuota
  // sta uscendo dal flow "appena collegato A→B" o "sto cercando con cosa
  // collegare". Senza questo cleanup la vista restava bloccata.
  if(getLinkFocus()) clearLinkFocus();
  if(isLinkMode()) setLinkMode(false);
  setTrailHover(null);
  renderTrail();
}

export function refreshSelectedPanels(){
  const F = getActive();
  if(!F.selected) return;
  const w = F.wordMap[F.selected];
  const sig = (w && w.sig) || NEUTRAL_SIG;
  showInfo(F.selected);
  showDimBars(sig);
  drawRadar(sig);
}
