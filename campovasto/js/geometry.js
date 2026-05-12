// Pure functions: firma ↔ spazio, Octalysis, reference frame.
// Nessun colore, nessun DOM — solo matematica del layout.

import { DIM_ANGLES, R, NEUTRAL_SIG } from './constants.js';

export function esc(s){
  return String(s == null ? '' : s)
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

// Octalysis 8 → 8 derivato dalla firma dimensionale.
// sig = [agency, permanenza, intensità, tempo, confine, complessità, definizione, valenza] (0..100)
export function computeOctalysis(sig){
  const d = (sig || NEUTRAL_SIG).map(v => (v || 0) / 100);
  return [
    Math.sqrt(d[0] * d[1]),                                  // 0 significato epico
    Math.sqrt(d[5] * d[0]),                                  // 1 empowerment
    Math.sqrt(d[7] * Math.max(0, 1 - d[4])),                 // 2 influenza sociale
    Math.sqrt(d[5] * Math.max(0, 1 - d[6])),                 // 3 imprevedibilità
    Math.sqrt(Math.max(0, 1 - d[7]) * Math.max(0, 1 - d[1])),// 4 evitamento
    Math.cbrt(d[3] * d[2] * Math.max(0, 1 - d[1])),          // 5 scarsità
    Math.sqrt(d[4] * d[1]),                                  // 6 possesso
    Math.sqrt(d[2] * d[6])                                   // 7 realizzazione
  ];
}

// Proietta una firma 8D → coordinate 2D usando il reference frame passato.
// frame = { dimMean: [8], dimStd: [8] }
export function sigToXY(sig, frame){
  let x = 0, y = 0;
  for(let i = 0; i < 8; i++){
    const std = frame.dimStd[i];
    const z = std > 0.01 ? ((sig[i] || 50) - frame.dimMean[i]) / std : 0;
    x += z * Math.cos(DIM_ANGLES[i]);
    y += z * Math.sin(DIM_ANGLES[i]);
  }
  return { x, y };
}

// Deriva medie + deviazioni standard da un array di parole con firma.
export function deriveFrame(words){
  const sums = [0,0,0,0,0,0,0,0], sq = [0,0,0,0,0,0,0,0];
  let cnt = 0;
  words.forEach(w => {
    if(w.sig){
      for(let i = 0; i < 8; i++){
        sums[i] += w.sig[i];
        sq[i] += w.sig[i] * w.sig[i];
      }
      cnt++;
    }
  });
  const dimMean = new Array(8).fill(0);
  const dimStd  = new Array(8).fill(1);
  if(cnt){
    for(let i = 0; i < 8; i++){
      dimMean[i] = sums[i] / cnt;
      dimStd[i] = Math.sqrt(Math.max(0.01, sq[i] / cnt - dimMean[i] * dimMean[i]));
    }
  }
  return { dimMean, dimStd };
}

// Normalizza ciascuna dimensione per rank percentuale (in-place).
// Dopo l'invocazione, w.sig è normalizzato 0..100 per dim.
export function rankNormalizeInPlace(words){
  if(words.length <= 1) return;
  for(let d = 0; d < 8; d++){
    const sorted = [...words].sort((a, b) => (a.sig?.[d] || 0) - (b.sig?.[d] || 0));
    sorted.forEach((w, rank) => {
      if(!w._nsig) w._nsig = new Array(8).fill(50);
      w._nsig[d] = Math.round((rank / (words.length - 1)) * 100);
    });
  }
  words.forEach(w => { w._rawSig = w.sig; w.sig = w._nsig || w.sig; });
}

// Calcola posizione 2D per una parola a partire dal suo rank-per-magnitudine.
export function placeByRank(sig, frame, rank, totalWords, jitter = 0){
  const p = sigToXY(sig, frame);
  const angle = Math.atan2(p.y, p.x);
  const mag = Math.sqrt(p.x * p.x + p.y * p.y);
  const normR = totalWords > 1 ? 0.12 + (rank / (totalWords - 1)) * 0.83 : 0.5;
  const r = normR * R * (1 + (Math.random() - 0.5) * jitter);
  return {
    x: Math.cos(angle) * r,
    y: Math.sin(angle) * r,
    angle, mag, normR,
  };
}

// Rank per magnitudine in un array di parole.
export function rankOfMag(mag, words){
  const mags = words.map(w => w.position?.mag || 0).sort((a, b) => a - b);
  let rank = mags.findIndex(m => m >= mag);
  return rank < 0 ? mags.length : rank;
}
