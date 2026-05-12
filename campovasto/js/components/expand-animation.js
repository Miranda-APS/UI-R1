// Animazione del personale (creato da frase) — "lettura → espansione semantica".
//
// Roadmap UX §2. Sotto l'animazione c'è il vasto montato in vis-network. Le
// fasi dell'animazione, in ordine:
//
//   FASE 0 — DIM:  i nodi del vasto vengono attenuati a opacity 0.4 (sfumatura
//                  di sfondo, le glow dell'animazione risaltano sopra).
//   FASE 1 — CYCLE: per ogni parola della frase, in sequenza —
//      1. la parola si illumina alla sua posizione 8D nel vasto
//      2. archi animati la collegano ai vicini (rose)
//      3. la rosa tiene un breve hold
//      4. la rosa si chiude (archi + glow svaniscono)
//      5. parte la parola successiva
//   FASE 2 — FADEOUT: i nodi del vasto NON rilevanti (cioè non parole della
//      frase né loro vicini) sfumano da opacity 0.4 a 0; quelli rilevanti
//      salgono da 0.4 a 1.
//   FASE 3 — HANDOVER: emettiamo `uir1:expansion-completed`. view-switcher.js
//      smonta il vasto e monta il personale: i puntini delle parole della
//      frase + vicini sono già alle posizioni dove l'utente li ha visti
//      restare (stesso frame 8D, nessun salto).
//
// Le coordinate dei punti sono lette dal vasto e proiettate sul canvas via
// `network.canvasToDOM(pos)` ad ogni frame, così l'animazione segue eventuali
// pan/zoom. Per parole non presenti nel vasto, fallback al personale.

import { UI, colorForSig } from '../theme.js';
import { get as getField } from '../manager.js';
import { network } from '../graph.js';
import { buildNodeSpec } from '../node-style.js';

// ---- Tempi (ms) ----------------------------------------------------------

const TIMING = {
  intro:     500,   // delay iniziale: campo dim visibile, niente glow ancora
  appear:    500,   // fade-in del glow della parola (era 250: troppo rapido)
  perSat:    280,   // delay fra satelliti consecutivi durante l'apertura
  hold:      1100,  // pause con tutti i satelliti visibili (leggibile)
  close:     1100,  // ritrazione della rosa
  gap:       500,   // pausa fra una parola della frase e la successiva
  fadeout:   2000,  // fade-out dei nodi non rilevanti (fase 2)
  settle:    400,   // pausa dopo fadeout, prima del commit (no salto a freddo)
};

// Opacity dei nodi del vasto durante l'animazione iniziale (cycle).
// I nodi rilevanti (parole della frase + vicini) salgono a 1 durante fadeout;
// i non rilevanti scendono a 0. Tenuto abbastanza alto perché il campo
// resti leggibile come contesto durante l'animazione (richiesta UX:
// "puntini più luminosi").
const DIM_VASTO = 0.45;

// ---- Layout / rendering --------------------------------------------------

const STYLE = {
  satR:         8,     // raggio del pallino satellite (era 5: troppo timido sopra il campo)
  archW:        2.0,   // larghezza dell'arco genitore→satellite
  labelOffY:    -18,   // offset Y della label della parola sopra il nodo attivo
  bgPad:        4,
  fontWord:     '600 14px "JetBrains Mono", monospace',
  fontSat:      '500 12.5px "JetBrains Mono", monospace',
  fontCrumb:    '500 13.5px "JetBrains Mono", monospace',
  crumbY:       30,    // distanza dal top del canvas
  crumbRight:   28,    // distanza dal bordo destro del canvas
  crumbMaxW:    0.55,  // larghezza max della breadcrumb come % del canvas
};

// ---- Stato modulo --------------------------------------------------------

let _state           = null;
let _canvas          = null;
let _ctx             = null;
let _rafId           = null;
let _resizeListener  = null;

// ---- API pubblica --------------------------------------------------------

export function startExpandAnimation(field, container){
  if(!field || !container) return;
  stopExpandAnimation();

  const sentence = field.sentence || '';
  // Ordina per sentenceIndex (set in buildNuovo): l'animazione segue l'ordine
  // di lettura della frase, non l'ordine di inserzione nel field.
  const sentenceWords = field.words
    .filter(w => w.flags?.fromSentence)
    .slice()
    .sort((a, b) => {
      const ai = typeof a.sentenceIndex === 'number' ? a.sentenceIndex : 999;
      const bi = typeof b.sentenceIndex === 'number' ? b.sentenceIndex : 999;
      return ai - bi;
    });
  if(sentenceWords.length === 0) return;

  _setupCanvas(container);
  _state = _buildState(field, sentenceWords, sentence);
  _state.field = field;
  _state.startTime = performance.now();
  _state.completionEmitted = false;
  _state.fadeoutInitDone = false;
  // Sfumatura iniziale: porta tutti i nodi del personale a DIM_VASTO così la
  // glow dell'animazione sopra risalta.
  _dimVasto();
  _rafId = requestAnimationFrame(_loop);
}

export function stopExpandAnimation(){
  if(_rafId){ cancelAnimationFrame(_rafId); _rafId = null; }
  if(_resizeListener){ window.removeEventListener('resize', _resizeListener); _resizeListener = null; }
  if(_canvas){ _canvas.remove(); _canvas = null; _ctx = null; }
  // Se l'utente interrompe a metà cycle, alcune parole potrebbero avere lo
  // spec 'active' applicato — ripuliscile prima di restore.
  _clearActiveVisuals();
  // _restoreVasto SOLO se l'utente esce dall'animazione PRIMA del completamento:
  // serve a non lasciare nodi invisibili in caso di abort. Se invece il fadeout
  // è terminato regolarmente (completionEmitted), NON restore: i nodi non-preserved
  // sono a opacity=0 e devono RESTARE invisibili nei pochi millisecondi tra
  // commitExpansion (che li rimuove fisicamente) e il remount del field pulito.
  // Senza questo gate vedi un flash di tutti i 3000 cloni a piena opacità.
  if(_state && !_state.completionEmitted) _restoreVasto();
  _state = null;
}

// Sfumatura sui nodi del personale (popolato con clone vasto) durante
// l'animazione iniziale. Le parole della frase + vicini emergeranno con
// l'animazione di glow sopra.
function _dimVasto(){
  if(!_state?.field) return;
  const P = _state.field;
  _state.allNodeIds = P.words.map(w => w.w);
  _state.activeIds = new Set();
  const batch = _state.allNodeIds.map(id => ({ id, opacity: DIM_VASTO }));
  try { P.nodesDS.update(batch); } catch(_){}
}

// Restituisce true se due Set hanno gli stessi elementi.
function _setEqual(a, b){
  if(a.size !== b.size) return false;
  for(const v of a) if(!b.has(v)) return false;
  return true;
}

// Spec per il nodo nello stato "attivo" del cycle (parola corrente della
// frase: bordo bianco, dimensione maggiore, opacity piena). Riusa la variante
// 'active' di node-style — niente colori inline qui (CLAUDE.md §2). Rimuove
// le chiavi di posizione: vis-network non deve riposizionare il nodo durante
// l'update.
function _activeSpecForCycle(wordId){
  const w = _state.field.wordMap[wordId];
  if(!w) return null;
  const spec = buildNodeSpec(w, 'active', { fieldId: 'nuovo' });
  delete spec.x; delete spec.y; delete spec.fixed;
  // L'animazione disegna la label su canvas (_drawLabel): la label vis viene
  // tenuta vuota dalla variante 'active', quindi nessun doppione.
  return spec;
}

// Spec per il nodo nello stato "non attivo" del cycle (dim allo sfondo).
// Riapplica la variante 'normal' ma sovrascrive opacity al livello DIM_VASTO,
// e nasconde la label (showLabel:false) — durante il cycle solo il canvas
// disegna il testo della parola attiva.
function _dimSpecForCycle(wordId){
  const w = _state.field.wordMap[wordId];
  if(!w) return null;
  const spec = buildNodeSpec(w, 'normal', { fieldId: 'nuovo', showLabel: false });
  delete spec.x; delete spec.y; delete spec.fixed;
  spec.opacity = DIM_VASTO;
  return spec;
}

// Aggiorna lo stato "attivo/dim" dei nodi della frase in base al tempo. Diff
// minimale verso _state.activeIds: un solo update per transizione, non per frame.
// Sostituisce il vecchio `_drawGlow` (cerchione) con un cambio di spec del
// nodo stesso — stesso effetto di un mouseover/select, niente halo.
function _updateActiveVisuals(now){
  if(!_state?.field) return;
  const want = new Set();
  for(let i = 0; i < _state.parole.length; i++){
    const ws = _autoWordState(i, now);
    if(ws.phase === 'appearing' || ws.phase === 'opening' || ws.phase === 'hold'){
      want.add(_state.parole[i].word);
    }
  }
  const have = _state.activeIds || new Set();
  if(_setEqual(want, have)) return;
  const batch = [];
  for(const id of have){
    if(!want.has(id)){
      const s = _dimSpecForCycle(id);
      if(s) batch.push(s);
    }
  }
  for(const id of want){
    if(!have.has(id)){
      const s = _activeSpecForCycle(id);
      if(s) batch.push(s);
    }
  }
  if(batch.length){
    try { _state.field.nodesDS.update(batch); } catch(_){}
  }
  _state.activeIds = want;
}

// Riporta tutti i nodi correntemente "attivi" allo stato dim del cycle. Da
// chiamare al passaggio cycle→fadeout (fadeout possiede l'opacity) e in
// stopExpandAnimation per pulizia idempotente.
function _clearActiveVisuals(){
  if(!_state?.field || !_state.activeIds || _state.activeIds.size === 0) return;
  const batch = [];
  for(const id of _state.activeIds){
    const s = _dimSpecForCycle(id);
    if(s) batch.push(s);
  }
  if(batch.length){
    try { _state.field.nodesDS.update(batch); } catch(_){}
  }
  _state.activeIds = new Set();
}

function _restoreVasto(){
  // Non chiamato di norma: lasciamo la fase fadeout gestire l'opacity finale
  // (preserved=1, non-preserved=0) e poi commitExpansion rimuove fisicamente
  // i non-preserved. Resta come fallback se serve riportare a stato pulito.
  if(!_state?.field || !_state.allNodeIds) return;
  const batch = _state.allNodeIds.map(id => ({ id, opacity: 1 }));
  try { _state.field.nodesDS.update(batch); } catch(_){}
}

// ---- Setup canvas --------------------------------------------------------

function _setupCanvas(container){
  _canvas = document.createElement('canvas');
  _canvas.id = 'medio-anim-canvas';
  container.appendChild(_canvas);
  _resizeCanvas();
  _resizeListener = () => { _resizeCanvas(); };
  window.addEventListener('resize', _resizeListener);
}

function _resizeCanvas(){
  if(!_canvas) return;
  const dpr = window.devicePixelRatio || 1;
  const rect = _canvas.getBoundingClientRect();
  _canvas.width  = Math.max(1, Math.round(rect.width  * dpr));
  _canvas.height = Math.max(1, Math.round(rect.height * dpr));
  _ctx = _canvas.getContext('2d');
  _ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
}

// ---- Build state ---------------------------------------------------------

function _buildState(field, sentenceWords, sentence){
  // Posizione di una parola: vasto prima, personale come fallback. Le coordinate
  // sono "vis-network space" (~ [-550..550]); proietteremo su DOM via
  // network.canvasToDOM ad ogni frame (così pan/zoom funzionano).
  const vasto = getField('vasto');
  const senSet = new Set(sentenceWords.map(w => w.w));
  const lookupPos = (w) => vasto?.wordMap?.[w]?.position || field.wordMap?.[w]?.position || null;
  const lookupColor = (w) => {
    const sig = vasto?.wordMap?.[w]?.sig || field.wordMap?.[w]?.sig;
    return sig ? colorForSig(sig) : UI.textDim;
  };

  const parole = sentenceWords.map(sw => {
    const targets = [];
    (field.edgesByWord[sw.w] || []).forEach(key => {
      const e = field.edgeByKey[key];
      if(!e || e.from !== sw.w) return;
      if(e.to === sw.w) return;
      if(senSet.has(e.to)) return;
      if(targets.includes(e.to)) return;
      targets.push(e.to);
    });
    const limited = targets.slice(0, 6);

    const sats = limited.map((t, j) => {
      const pos = lookupPos(t);
      return {
        word:  t,
        color: lookupColor(t),
        pos,                                         // null = fallback semicircolare
        fallbackAngle: Math.PI * (0.18 + 0.64 * (j / Math.max(1, limited.length - 1))),
      };
    });

    return {
      word:        sw.w,
      displayName: sw.displayName || sw.w,
      color:       colorForSig(sw.sig),
      pos:         lookupPos(sw.w) || sw.position || null,
      targets:     sats,
    };
  });

  return { sentence, parole };
}

// Proietta una posizione vis-network su DOM. Se network non è ancora pronto,
// torna null (lo skip nel rendering).
function _toDOM(pos){
  if(!pos || !network) return null;
  try { return network.canvasToDOM({ x: pos.x, y: pos.y }); }
  catch(_){ return null; }
}

// ---- Loop ----------------------------------------------------------------

function _totalAutoDur(){
  if(!_state) return 0;
  let total = 0;
  for(let i = 0; i < _state.parole.length; i++) total += _wordCycleDur(i);
  return total;
}

function _loop(now){
  if(!_state || !_ctx){ _rafId = null; return; }

  const elapsed = now - _state.startTime;
  // Cycle dur include intro iniziale (TIMING.intro è dentro _wordStartOffset).
  const cycleDur = TIMING.intro + _totalAutoDur();
  const fadeoutEnd = cycleDur + TIMING.fadeout;
  const settleEnd  = fadeoutEnd + TIMING.settle;

  // FASE FINALE — settle terminato: dispatch handover.
  if(elapsed > settleEnd){
    if(!_state.completionEmitted){
      _state.completionEmitted = true;
      window.dispatchEvent(new CustomEvent('uir1:expansion-completed'));
    }
    return;
  }

  const w = _canvas.width  / (window.devicePixelRatio || 1);
  const h = _canvas.height / (window.devicePixelRatio || 1);
  _ctx.clearRect(0, 0, w, h);

  if(elapsed <= cycleDur){
    // FASE 1 — cycle: la parola corrente passa allo spec 'active' del nodo
    // (stesso feedback di selezione/mouseover, niente cerchione canvas).
    // Su canvas restano label + satelliti + archi animati.
    _updateActiveVisuals(now);
    let activeCrumbIdx = -1;
    for(let i = 0; i < _state.parole.length; i++){
      const ws = _autoWordState(i, now);
      if(ws.phase === 'hidden' || ws.phase === 'done') continue;
      if(activeCrumbIdx === -1) activeCrumbIdx = i;

      const parola = _state.parole[i];
      const cdom = _toDOM(parola.pos);
      if(!cdom) continue;

      // Label della parola sopra il nodo (alpha guidata da glowAlpha = curva
      // di apparizione/scomparsa).
      _drawLabel(cdom.x, cdom.y + STYLE.labelOffY, (parola.displayName || parola.word), ws.glowAlpha, STYLE.fontWord, parola.color);

      // Satelliti (durante opening / hold / closing)
      if(ws.perSatT){
        _drawSatellites(parola, cdom.x, cdom.y, ws.perSatT);
      }
    }
    // Breadcrumb della frase in alto al canvas: l'utente vede la sequenza di
    // lettura e dove sta in quel momento. Senza questo, l'attivazione delle
    // parole nel campo sembra "sparsa" perché le posizioni 8D sono per
    // definizione spaziali, non lineari.
    _drawBreadcrumb(w, activeCrumbIdx);
  } else if(elapsed <= fadeoutEnd){
    // FASE 2 — fadeout: i nodi non rilevanti svaniscono, i rilevanti emergono.
    // Prima di lasciare al fadeout il controllo dell'opacity, ripristina ogni
    // nodo eventualmente ancora "attivo" allo spec dim — così il fadeout parte
    // da uno stato uniforme (DIM_VASTO ovunque).
    if(!_state.fadeoutInitDone){
      _clearActiveVisuals();
      _initFadeout();
    }
    const fadeT = _easeOut(Math.min(1, (elapsed - cycleDur) / TIMING.fadeout));
    _renderFadeout(fadeT);
  }
  // FASE settle: né disegno né update opacity. Il network resta nello stato
  // finale del fadeout (preserved 1, others 0). L'utente "respira" prima del
  // commit/remount che avviene al dispatch.

  _rafId = requestAnimationFrame(_loop);
}

// Costruisce le liste di nodi del personale preserved (parole frase + vicini)
// e non-preserved (vasto-clone non rilevanti). Da usare per la fase fadeout.
function _initFadeout(){
  if(!_state?.field) return;
  const preserved = new Set();
  for(const p of _state.parole){
    preserved.add(p.word);
    for(const sat of p.targets) preserved.add(sat.word);
  }
  _state.preservedIds = [];
  _state.fadeOutIds   = [];
  for(const id of (_state.allNodeIds || [])){
    if(preserved.has(id)) _state.preservedIds.push(id);
    else                   _state.fadeOutIds.push(id);
  }
  _state.fadeoutInitDone = true;
}

// t in [0..1]: 0 = stato cycle (tutti a DIM_VASTO), 1 = stato finale
// (preserved a 1, non-preserved a 0).
function _renderFadeout(t){
  if(!_state?.field) return;
  const preservedOpacity = DIM_VASTO + (1 - DIM_VASTO) * t;
  const fadeOutOpacity   = DIM_VASTO * (1 - t);
  const batch = [];
  for(const id of _state.preservedIds || []) batch.push({ id, opacity: preservedOpacity });
  for(const id of _state.fadeOutIds   || []) batch.push({ id, opacity: fadeOutOpacity });
  if(batch.length){
    try { _state.field.nodesDS.update(batch); } catch(_){}
  }
}

// ---- State machine per parola --------------------------------------------

function _wordCycleDur(i){
  const n = _state.parole[i].targets.length;
  return TIMING.appear + n * TIMING.perSat + TIMING.hold + TIMING.close;
}

function _wordStartOffset(i){
  let t = TIMING.intro;  // intro iniziale: il campo si "stabilizza" prima del cycle
  for(let k = 0; k < i; k++) t += _wordCycleDur(k);
  // Pausa di TIMING.gap fra parola e parola: il glow della precedente è
  // completamente svanito prima che la prossima inizi ad apparire.
  return t + i * TIMING.gap;
}

// Restituisce { phase, glowAlpha, perSatT? } per la parola i al tempo `now`.
// Phase: 'hidden' | 'appearing' | 'opening' | 'hold' | 'closing' | 'done'
function _autoWordState(i, now){
  const elapsed = now - _state.startTime - _wordStartOffset(i);
  if(elapsed < 0) return { phase: 'hidden', glowAlpha: 0 };

  let cursor = 0;

  // appearing — solo glow, satelliti non ancora
  if(elapsed < TIMING.appear){
    const t = _easeOut(elapsed / TIMING.appear);
    return { phase: 'appearing', glowAlpha: t };
  }
  cursor += TIMING.appear;

  const n = _state.parole[i].targets.length;
  const openingDur = n * TIMING.perSat;

  // opening — un satellite per volta
  if(elapsed < cursor + openingDur){
    const opE = elapsed - cursor;
    return { phase: 'opening', glowAlpha: 1, perSatT: _perSatProgress(n, opE) };
  }
  cursor += openingDur;

  // hold — tutti aperti
  if(elapsed < cursor + TIMING.hold){
    return { phase: 'hold', glowAlpha: 1, perSatT: _state.parole[i].targets.map(() => 1) };
  }
  cursor += TIMING.hold;

  // closing — ritrazione + glow svanisce
  if(elapsed < cursor + TIMING.close){
    const closeT = _easeOut((elapsed - cursor) / TIMING.close);
    return {
      phase: 'closing',
      glowAlpha: 1 - closeT,
      perSatT: _state.parole[i].targets.map(() => 1 - closeT),
    };
  }

  return { phase: 'done', glowAlpha: 0 };
}

function _perSatProgress(n, opElapsed){
  const arr = new Array(n);
  for(let k = 0; k < n; k++){
    const e = opElapsed - k * TIMING.perSat;
    arr[k] = e <= 0 ? 0 : _easeOut(Math.min(1, e / TIMING.perSat));
  }
  return arr;
}

// ---- Drawing -------------------------------------------------------------

// color: tinta del testo. Se omesso, default a textBright. Per coerenza
// con il vasto, le label nell'animazione usano il colore del puntino.
function _drawLabel(x, y, text, alpha, font, color){
  if(alpha <= 0 || !text) return;
  const ctx = _ctx;
  ctx.save();
  ctx.font = font;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  // Sfondo etichetta per leggibilità sopra il campo
  const w = ctx.measureText(text).width;
  ctx.globalAlpha = alpha * 0.7;
  ctx.fillStyle = UI.bg;
  ctx.fillRect(x - w/2 - STYLE.bgPad, y - 9, w + STYLE.bgPad * 2, 16);
  // Testo nel colore del puntino (firma)
  ctx.globalAlpha = alpha;
  ctx.fillStyle = color || UI.textBright || UI.text;
  ctx.fillText(text, x, y);
  ctx.restore();
}

// Disegna in alto-destra al canvas la frase letta con la parola attiva
// evidenziata. Le parole già lette sono dim, quella attiva piena, le
// future ancora dim. Il colore di ogni voce è quello della firma del
// puntino corrispondente (continuità visiva con il campo sotto).
//
// Posizione top-right perché:
//   - top-center sovrapponeva i nodi-frase quando le loro posizioni 8D
//     cadevano in alto al campo.
//   - bottom-* è già occupato dall'understanding-panel.
function _drawBreadcrumb(canvasW, activeIdx){
  if(!_state?.parole?.length) return;
  const ctx = _ctx;
  ctx.save();
  ctx.font = STYLE.fontCrumb;
  ctx.textBaseline = 'middle';

  const sep = ' → ';
  const parts = _state.parole.map(p => p.displayName || p.word);
  const sepW = ctx.measureText(sep).width;
  const partWs = parts.map(s => ctx.measureText(s).width);
  const fullW = partWs.reduce((a, b) => a + b, 0) + sepW * Math.max(0, parts.length - 1);

  // Tronca dall'inizio se eccede la larghezza max — la parola attiva e
  // quelle successive devono restare visibili.
  const maxW = Math.floor(canvasW * STYLE.crumbMaxW);
  let startIdx = 0;
  let totalW = fullW;
  if(fullW > maxW && activeIdx > 0){
    // Drop le prime parole finché non sta nel maxW, ma sempre tenendo
    // almeno la parola attiva e quelle successive.
    while(startIdx < activeIdx && totalW > maxW){
      totalW -= partWs[startIdx] + sepW;
      startIdx++;
    }
  }
  const visibleParts  = parts.slice(startIdx);
  const visibleWidths = partWs.slice(startIdx);
  const visibleW = visibleWidths.reduce((a, b) => a + b, 0)
                 + sepW * Math.max(0, visibleParts.length - 1);

  // Ancoraggio destra: x parte da (right - visibleW)
  const right = canvasW - STYLE.crumbRight;
  let x = right - visibleW;
  const y = STYLE.crumbY;

  // Background scuro per contrasto sopra il campo dim
  ctx.globalAlpha = 0.72;
  ctx.fillStyle = UI.bg;
  ctx.fillRect(x - STYLE.bgPad * 2, y - 13, visibleW + STYLE.bgPad * 4, 26);

  // "…" davanti se abbiamo troncato
  ctx.textAlign = 'left';
  if(startIdx > 0){
    ctx.globalAlpha = 0.5;
    ctx.fillStyle = UI.textDim || UI.textBright;
    ctx.fillText('… ', x - ctx.measureText('… ').width, y);
  }

  for(let i = 0; i < visibleParts.length; i++){
    const realIdx  = i + startIdx;
    const isActive = realIdx === activeIdx;
    const isPast   = activeIdx >= 0 && realIdx < activeIdx;
    ctx.globalAlpha = isActive ? 1.0 : (isPast ? 0.55 : 0.35);
    ctx.fillStyle = _state.parole[realIdx].color || UI.textBright;
    ctx.fillText(visibleParts[i], x, y);
    x += visibleWidths[i];
    if(i < visibleParts.length - 1){
      ctx.globalAlpha = 0.4;
      ctx.fillStyle = UI.textDim || UI.textBright;
      ctx.fillText(sep, x, y);
      x += sepW;
    }
  }
  ctx.restore();
}

function _drawSatellites(parola, cx, cy, perSatT){
  const ctx = _ctx;
  ctx.save();
  for(let k = 0; k < parola.targets.length; k++){
    const t = perSatT[k] || 0;
    if(t <= 0) continue;
    const sat = parola.targets[k];

    // Posizione DOM target del satellite
    let tx, ty;
    const tdom = _toDOM(sat.pos);
    if(tdom){
      tx = tdom.x; ty = tdom.y;
    } else {
      // Fallback: distribuzione semicircolare a raggio 100 dal centro
      tx = cx + Math.cos(sat.fallbackAngle) * 100;
      ty = cy + Math.sin(sat.fallbackAngle) * 100;
    }

    // Animazione: dal centro (cx, cy) al target (tx, ty), guidata da t in [0..1].
    const sx = cx + (tx - cx) * t;
    const sy = cy + (ty - cy) * t;

    // Linea genitore → satellite
    ctx.globalAlpha = 0.55 * t;
    ctx.strokeStyle = UI.edgeHover;
    ctx.lineWidth = STYLE.archW;
    ctx.beginPath();
    ctx.moveTo(cx, cy);
    ctx.lineTo(sx, sy);
    ctx.stroke();

    // Pallino satellite
    ctx.globalAlpha = t;
    ctx.beginPath();
    ctx.arc(sx, sy, STYLE.satR, 0, Math.PI * 2);
    ctx.fillStyle = sat.color;
    ctx.fill();

    // Etichetta verso fine espansione
    if(t > 0.6){
      const labelAlpha = (t - 0.6) / 0.4;
      _drawLabel(sx, sy + STYLE.satR + 10, sat.word, labelAlpha, STYLE.fontSat, sat.color);
    }
  }
  ctx.restore();
}

// ---- Helpers --------------------------------------------------------------

function _easeOut(t){
  t = Math.max(0, Math.min(1, t));
  return 1 - Math.pow(1 - t, 3);
}
