// Cablaggio dei bottoni della sidebar (header nav, tabs, footer azioni, card parola).
// I bottoni non conoscono logica: delegano a view-switcher/transmit/editor/filters.

import { openAddWord, openAddRel, openEditDims } from '../editor.js';
import { switchMode, fitCurrent } from './view-switcher.js';
import { doTransmit, sendToCommunity } from './transmit.js';
import { deselectWord } from './selection.js';
import { openSentencePanel } from './sentence-panel.js';
import { getActive, saveField, FIELDS } from '../manager.js';
import { openConfirm } from '../components/confirm-panel.js';

export function wireButtons(){
  document.querySelectorAll('button.tab').forEach(btn => {
    btn.onclick = () => {
      if(btn.disabled) return;
      const view = btn.dataset.view;
      const sp = document.getElementById('sentencePanel');
      const sentenceOpen = sp && sp.style.display !== 'none' && sp.innerHTML.trim().length > 0;

      // Tab nuovo+empty: il campo è vuoto. Toggle del pannello frase: se è
      // già aperto, lo chiude (per uscire dalla creazione e tornare a vasto
      // basta cliccare la tab vasto, ma se l'utente clicca la stessa tab
      // ancora chiudiamo per coerenza).
      if(view === 'nuovo' && btn.classList.contains('empty')){
        const N = FIELDS.nuovo;
        if(!N || (N.words.length === 0 && !N.sentence)){
          if(sentenceOpen) sp.style.display = 'none';
          else openSentencePanel();
          return;
        }
      }
      // Click su qualsiasi altra tab: chiude il pannello frase se aperto,
      // così l'utente non resta bloccato dentro la creazione.
      if(sentenceOpen) sp.style.display = 'none';
      switchMode(view);
    };
  });

  const addWordBtn  = document.querySelector('[data-action="add-word"]');
  const addRelBtn   = document.querySelector('[data-action="add-rel"]');
  const sentenceBtn = document.querySelector('[data-action="sentence"]');
  const communityBtn= document.querySelector('[data-action="community"]');
  const transmitBtn = document.getElementById('btnTransmit');
  const toCommunity = document.getElementById('btnToCommunity');
  const homeBtn     = document.getElementById('btn-home');
  const backBtn     = document.getElementById('btn-back');

  if(addWordBtn)   addWordBtn.onclick   = () => openAddWord();
  if(addRelBtn)    addRelBtn.onclick    = () => openAddRel();
  if(sentenceBtn)  sentenceBtn.onclick  = openSentencePanel;
  if(transmitBtn)  transmitBtn.onclick  = doTransmit;
  if(toCommunity)  toCommunity.onclick  = sendToCommunity;
  if(homeBtn)      homeBtn.onclick      = () => { deselectWord(); fitCurrent(); };
  if(backBtn)      backBtn.onclick      = deselectWord;
  if(communityBtn) communityBtn.onclick = () => { window.location.href = 'community.html'; };

  // Bottoni della card parola (.box.parola .azioni) — agiscono sulla parola
  // selezionata. Sono dentro la card che è nascosta finché niente è selezionato,
  // quindi non servono guardie: se sono cliccabili, F.selected esiste.
  const parolaModifica  = document.getElementById('parolaModifica');
  const parolaRelazione = document.getElementById('parolaRelazione');
  const parolaElimina   = document.getElementById('parolaElimina');

  if(parolaModifica) parolaModifica.onclick = () => {
    const F = getActive();
    if(F?.selected) openEditDims(F.selected);
  };
  if(parolaRelazione) parolaRelazione.onclick = () => {
    const F = getActive();
    if(F?.selected) openAddRel(F.selected);
  };
  if(parolaElimina) parolaElimina.onclick = async () => {
    const F = getActive();
    if(!F?.selected) return;
    const w = F.selected;
    const ok = await openConfirm({
      title: 'Eliminare la parola?',
      message: `"${w}" verrà rimossa dal campo insieme a tutti i suoi archi.`,
      confirmLabel: 'elimina',
      cancelLabel: 'annulla',
      danger: true,
    });
    if(!ok) return;
    deselectWord();
    F.removeWord(w);
    saveField(F.id);
  };
}
