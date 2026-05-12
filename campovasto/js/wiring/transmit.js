// Trasmissione del campo nuovo verso il campo vasto.
// + invio della frase del personale al campo community.

import { FIELDS, activeId, transmitToVasto, pendingCount } from '../manager.js';
import { isActive as filterIsActive, apply as reapplyFilters } from '../filters.js';

export function updateTransmitButton(){
  const btn = document.getElementById('btnTransmit');
  if(!btn) return;
  if(activeId !== 'nuovo'){ btn.style.display = 'none'; return; }
  const pending = pendingCount();
  btn.style.display = '';
  btn.textContent = pending ? `↗ trasmetti (${pending})` : '↗ trasmesso';
  btn.disabled = pending === 0;

  const btnC = document.getElementById('btnToCommunity');
  if(btnC){
    const show = activeId === 'nuovo' && FIELDS.nuovo?.sentence;
    btnC.style.display = show ? '' : 'none';
  }
}

export async function doTransmit(){
  if(activeId !== 'nuovo') return;
  const btn = document.getElementById('btnTransmit');
  if(btn){ btn.disabled = true; btn.textContent = '↗ trasmissione…'; }
  let result = null;
  let netError = null;
  try {
    result = await transmitToVasto();
  } catch(e){
    netError = e;
    console.error('[transmit]', e);
  } finally {
    // Sempre: ripristina lo stato del bottone, anche se transmitToVasto
    // ha fallito a metà loop. Senza finally il bottone restava bloccato
    // su "trasmissione…" e l'utente non sapeva cosa fosse successo.
    updateTransmitButton();
  }
  if(netError){
    showTransmitToast('errore di rete: ' + (netError.message || netError), 'err');
    return;
  }
  const { words, edges, errors } = result || { words: 0, edges: 0, errors: 0 };
  if(words || edges){
    const parts = [];
    if(words) parts.push(`${words} parol${words === 1 ? 'a' : 'e'}`);
    if(edges) parts.push(`${edges} relazion${edges === 1 ? 'e' : 'i'}`);
    showTransmitToast(`trasmesso al campo vasto: ${parts.join(' + ')}`, 'ok');
  } else if(!errors){
    showTransmitToast('nulla da trasmettere', 'ok');
  }
  if(errors) showTransmitToast(`${errors} elementi non trasmessi (vedi console)`, 'err');
  if(filterIsActive()) reapplyFilters();
}

// Toast effimero per segnalare il risultato della trasmissione.
function showTransmitToast(msg, kind = 'ok'){
  const t = document.createElement('div');
  t.className = 'transmit-toast ' + kind;
  t.textContent = msg;
  document.body.appendChild(t);
  requestAnimationFrame(() => t.classList.add('show'));
  setTimeout(() => {
    t.classList.remove('show');
    setTimeout(() => t.remove(), 300);
  }, 2400);
}

// Invia frase + vicini al campo community (via sessionStorage).
export function sendToCommunity(){
  const P = FIELDS.nuovo;
  if(!P || !P.sentence) return;

  const sentenceWords = P.words.filter(w => w.flags?.fromSentence).map(w => w.w);
  const wordsWithNeighbors = sentenceWords.map(sw => {
    const nbs = [];
    (P.edgesByWord[sw] || []).forEach(key => {
      const e = P.edgeByKey[key];
      if(!e) return;
      const other = e.from === sw ? e.to : e.from;
      if(other === sw) return;
      const direction = e.from === sw ? 'out' : 'in';
      nbs.push({ word: other, rel: e.rel, conf: e.conf || 50, direction });
    });
    nbs.sort((a, b) => b.conf - a.conf);
    return { w: sw, neighbors: nbs };
  });

  const payload = {
    sentence: P.sentence,
    words: wordsWithNeighbors,
    stamp: Date.now(),
  };
  try {
    sessionStorage.setItem('uir1_nuovo_to_community', JSON.stringify(payload));
    window.location.href = 'community.html?from=nuovo';
  } catch(e){
    console.warn('Storage bloccato:', e);
  }
}
