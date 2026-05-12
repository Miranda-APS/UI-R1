// Layout "a relazioni" del campo nuovo (v2: flow grid, niente fasce rigide).
//
// Visione:
//   - Riga superiore: parole-frase (ordine `sentenceIndex`), pallini grandi
//     e label prominenti — sono le ANCORE della frase.
//   - Sotto ciascuna parola-frase: una colonna di larghezza PROPORZIONALE al
//     numero di vicini di quella parola. I vicini fluiscono in grid wrap
//     (sinistra→destra, top→bottom), ORDINATI PER GRUPPO (S→C→M→F→L) così
//     parole della stessa famiglia appaiono contigue. Niente fasce orizzontali
//     rigide — sprecavano spazio in alcune zone e saturavano in altre.
//   - Pallini dei vicini ridotti a piccoli ancoraggi: la LABEL è il contenuto,
//     il colore della firma vive sulla scritta.
//   - Riga inferiore: orfane (parole utente senza archi), full-width.
//
// La logica del Field NON cambia: questo modulo legge `F.words`, `F.edges`,
// `F.edgesByWord`, `F.edgeByKey`, e scrive solo `word.position` + flag locali
// (`_layoutShared`, `_dimPos` snapshot per il rollback al layout dimensionale).

import { REL_GROUP, REL_WEIGHT, REL_GROUPS_INFO, DIM_ANGLES } from '../constants.js';
import { buildNodeSpec } from '../node-style.js';

// Calcola le coordinate canvas (xHalf, yHalf) in modo che il bounding box
// dei nodi abbia LO STESSO ASPECT RATIO del container del grafo. Così
// `network.fit()` riempie sia l'asse X che l'asse Y, niente spazio vuoto
// sopra/sotto.
function _computeCanvas(){
  const container = document.getElementById('graph');
  const cw = container?.clientWidth  || 1300;
  const ch = container?.clientHeight || 900;
  const xHalf = 600;
  const yHalf = Math.max(200, Math.round(xHalf * (ch / cw)));
  return { xHalf, yHalf };
}

// Ordine S/C/M/F/L per ordinamento dei vicini dentro una colonna.
const GROUP_ORDER = REL_GROUPS_INFO.map(g => g.id);
const GROUP_RANK = Object.fromEntries(GROUP_ORDER.map((g, i) => [g, i]));

// Proporzioni verticali (in unità canvas). Tenute compatte per spingere
// la riga frase verso il top del bounding box e l'ultima riga vicini verso
// il bottom — vis-network fit usa così tutta l'altezza del viewport.
const SENTENCE_ROW_H   = 55;   // header parole-frase: pallino + label
const ORPHAN_ROW_H_MAX = 60;   // riga orfane (compressa se vuota)
const ROW_PADDING_TOP  = 15;
const ROW_PADDING_BOT  = 15;

// Spacing tipico per la grid wrap dei vicini.
const NEIGHBOR_CELL_W  = 75;   // larghezza orizzontale ideale per cella label
const COL_PAD_X        = 15;   // padding orizzontale interno alla colonna

// ---- API pubblica --------------------------------------------------------

export function applyRectangularLayout(F){
  if(!F || !F.words?.length) return;

  _snapshotDimensionalPositions(F);

  const sentenceWords = F.words
    .filter(w => w.flags?.fromSentence)
    .slice()
    .sort((a, b) => _sentenceIndexOf(a) - _sentenceIndexOf(b));

  const { xHalf, yHalf } = _computeCanvas();

  // Caso degenere: nessuna parola della frase. Grid uniforme.
  if(sentenceWords.length === 0){
    _layoutGridFallback(F, xHalf, yHalf);
    _hideDimLabels(F, true);
    _pushPositionUpdates(F);
    _pushNodeStyleUpdates(F, 'rectangular');
    return;
  }

  const sentenceSet = new Set(sentenceWords.map(w => w.w));
  const assignment = _assignNeighbors(F, sentenceWords, sentenceSet);

  // Orfane: parole non-frase senza assegnazione (nessun arco a una parola-frase).
  const orphans = F.words.filter(w =>
    !sentenceSet.has(w.w) && !assignment.byNeighbor.has(w.w)
  );

  // ---- Larghezze colonne uguali ----------------
  const totalW = 2 * xHalf;
  const nCols = sentenceWords.length;
  const colW = totalW / Math.max(1, nCols);

  const colX = [];
  for(let i = 0; i < nCols; i++){
    colX.push(-xHalf + colW * (i + 0.5));
  }

  // ---- Verticale: riga frase (alto), area vicini (mid), orfane (basso) ----
  const sentenceY = -yHalf + ROW_PADDING_TOP + SENTENCE_ROW_H / 2;
  const neighborsTop = -yHalf + ROW_PADDING_TOP + SENTENCE_ROW_H + 30;

  // ---- Posiziona parole-frase --------------------------------------------
  // Skip parole con _userPositioned (drag manuale dell'utente): la sua
  // intenzione vince sul layout automatico.
  sentenceWords.forEach((sw, i) => {
    if(!sw._userPositioned){
      sw.position = { x: colX[i], y: sentenceY };
    }
    sw._layoutShared = false;
  });

  // ---- Posiziona vicini per ciascuna colonna -----------------------------
  let maxNeighborsBot = neighborsTop;
  for(let i = 0; i < sentenceWords.length; i++){
    const list = assignment.byHostList[sentenceWords[i].w];
    if(!list.length) continue;
    const bot = _placeColumnFlow(list, F.wordMap, colX[i], neighborsTop, colW);
    if(bot > maxNeighborsBot) maxNeighborsBot = bot;
  }

  // ---- Orfane: riga full-width in basso, grid wrap libero ----------------
  if(orphans.length > 0){
    const orphanTop = maxNeighborsBot + 60;
    _placeColumnFlow(
      orphans.map(w => ({ word: w.w })),
      F.wordMap, 0, orphanTop, totalW
    );
  }

  // ---- Marca i shared (vicini di più parole-frase) -----------------------
  for(const wId of assignment.sharedSet){
    const w = F.wordMap[wId];
    if(w) w._layoutShared = true;
  }

  _hideDimLabels(F, true);
  _pushPositionUpdates(F);
  _pushNodeStyleUpdates(F, 'rectangular');
}

// Layout dimensionale: ricalcola le posizioni dalle firme 8D (placeByRank
// sul frame del campo). NON ripristina `_dimPos` snapshot precedenti perché
// quegli snapshot erano post-spread con minDist alti e tendevano a spingere
// i nodi fuori dal grafo. La firma è la sola sorgente di verità della
// posizione semantica; lo spread serve solo per separare overlap residui.
//
// _userPositioned (drag manuale) vince comunque: quei nodi restano dove
// l'utente li ha messi.
export function applyDimensionalLayout(F){
  if(!F || !F.words?.length) return;

  // Ricalcola posizioni via firme per le parole non bloccate dal drag.
  const toPlace = F.words.filter(w => !w._userPositioned);
  if(toPlace.length) F._bulkPlaceByRank(toPlace);

  // Spread LEGGERO solo per separare sovrapposizioni residue. minDist
  // generoso (~110) faceva esplodere il campo per i 30-50 nodi tipici di
  // un personale; minDist 55 mantiene i nodi vicini alla loro firma
  // permettendo una leggibilità accettabile.
  F.spreadNonOverlapping({ minDist: 55, iterations: 40 });

  for(const w of F.words){ w._layoutShared = false; }

  _hideDimLabels(F, false);
  _pushPositionUpdates(F);
  _pushNodeStyleUpdates(F, 'dimensional');
}

// ---- Assegnazione vicini -------------------------------------------------

// Per ogni vicino non-frase: trova la parola-frase con cui ha l'arco di forza
// massima (forza = REL_WEIGHT[rel] × confidence) e ricorda il gruppo. I vicini
// connessi a >1 parola-frase finiscono `shared` (rendering a stellina).
//
// Output:
//   - byHostList[hostWord] = [{ word, group, strength }, ...]  ordinato per
//     gruppo (S→C→M→F→L) e dentro per strength desc.
//   - byNeighbor: Map(neighborId → bestHostInfo)
//   - sharedSet: Set di neighborIds con >1 host
function _assignNeighbors(F, sentenceWords, sentenceSet){
  const best  = new Map();   // neighbor → { hostWord, group, strength }
  const hosts = new Map();   // neighbor → Set<hostWord>

  for(const sw of sentenceWords){
    const keys = F.edgesByWord[sw.w];
    if(!keys) continue;
    for(const key of keys){
      const e = F.edgeByKey[key];
      if(!e) continue;
      const other = e.from === sw.w ? e.to : e.from;
      if(!other || other === sw.w) continue;
      if(sentenceSet.has(other)) continue;
      if(!F.wordMap[other]) continue;

      const group = REL_GROUP[e.rel] || 'L';
      const weight = REL_WEIGHT[e.rel] ?? 0.5;
      const conf = (e.conf ?? 50) / 100;
      const strength = weight * conf;

      if(!hosts.has(other)) hosts.set(other, new Set());
      hosts.get(other).add(sw.w);

      const cur = best.get(other);
      if(!cur || strength > cur.strength){
        best.set(other, { hostWord: sw.w, group, strength });
      }
    }
  }

  const byHostList = {};
  for(const sw of sentenceWords) byHostList[sw.w] = [];
  for(const [neighborId, info] of best){
    byHostList[info.hostWord].push({
      word: neighborId, group: info.group, strength: info.strength,
    });
  }

  // Ordinamento: prima per gruppo (S→C→M→F→L), poi per forza decrescente.
  // Risultato: vicini visivamente contigui per famiglia, le relazioni più
  // salienti in testa.
  for(const host in byHostList){
    byHostList[host].sort((a, b) => {
      const dr = (GROUP_RANK[a.group] ?? 99) - (GROUP_RANK[b.group] ?? 99);
      if(dr !== 0) return dr;
      return b.strength - a.strength;
    });
  }

  const sharedSet = new Set();
  for(const [neighborId, hostSet] of hosts){
    if(hostSet.size > 1) sharedSet.add(neighborId);
  }

  return { byHostList, byNeighbor: best, sharedSet };
}

// ---- Posizionamento grid-wrap dentro una colonna -------------------------

// `items` è un array di { word, group?, strength? } già ordinato.
function _placeColumnFlow(items, wordMap, cx, top, colW){
  if(!items.length) return top;
  const innerW = Math.max(1, colW - 2 * COL_PAD_X);

  const CELL_W = 120; // Più largo per accogliere i box rettangolari
  const ROW_H = 50;   // Altezza fissa per riga

  // Quanti per riga?
  const perRow = Math.max(1, Math.min(items.length, Math.floor(innerW / CELL_W) || 1));
  const rows = Math.ceil(items.length / perRow);

  for(let i = 0; i < items.length; i++){
    const r = Math.floor(i / perRow);
    const c = i % perRow;
    const inRow = (r === rows - 1) ? (items.length - r * perRow) : perRow;
    const cellSpacing = innerW / inRow;
    const w = wordMap[items[i].word];
    if(!w) continue;
    if(w._userPositioned) continue;  // posizione manuale: lasciala dov'è
    w.position = {
      x: cx - innerW / 2 + cellSpacing * (c + 0.5),
      y: top + r * ROW_H,
    };
  }
  return top + rows * ROW_H;
}

// ---- Internals -----------------------------------------------------------

function _sentenceIndexOf(w){
  return typeof w.sentenceIndex === 'number' ? w.sentenceIndex : 999;
}

function _snapshotDimensionalPositions(F){
  for(const w of F.words){
    if(!w._dimPos && w.position?.x != null){
      w._dimPos = { x: w.position.x, y: w.position.y };
    }
  }
}

// Fallback grid quando nessuna parola-frase: griglia uniforme.
function _layoutGridFallback(F, xHalf, yHalf){
  const ws = F.words;
  const n = ws.length;
  const cols = Math.max(1, Math.ceil(Math.sqrt(n * (xHalf / yHalf))));
  const rows = Math.ceil(n / cols);
  const cellW = (2 * xHalf) / cols;
  const cellH = (2 * yHalf) / rows;
  ws.forEach((w, i) => {
    if(w._userPositioned) { w._layoutShared = false; return; }
    const r = Math.floor(i / cols);
    const c = i % cols;
    w.position = {
      x: -xHalf + cellW * (c + 0.5),
      y: -yHalf + cellH * (r + 0.5),
    };
    w._layoutShared = false;
  });
}

// Mostra/nasconde le 8 dim-label "POTERE/MATERIA/...".
//
// Le dim-label vivono ora come overlay HTML (components/dim-overlay.js),
// non più come nodi del canvas vis-network. Questo no-op evita un bug
// pernicioso: prima `nodesDS.update({id: '_dim_label_*', x, y})` su nodi
// inesistenti faceva sì che vis-network li CREASSE da zero senza spec
// (forma/colore/font) → comparivano come grossi pallini blu di default
// nelle posizioni cardinali. Visibili nello screenshot della collega.
//
// La show/hide del dim-overlay HTML in funzione del layout è gestita
// da app.js via setDimOverlayVisible() (vedi onLayoutChange).
function _hideDimLabels(_F, _hide){ /* moved to dim-overlay.js (HTML) */ }

function _pushPositionUpdates(F){
  // niente `fixed: true`: in nuovo i nodi sono draggabili (gestito da
  // node-style.js). vis-network con physics off NON sposta da solo i nodi
  // senza fixed → comportamento identico al precedente, ma drag abilitato.
  const batch = F.words
    .filter(w => w.position?.x != null)
    .map(w => ({ id: w.w, x: w.position.x, y: w.position.y }));
  if(batch.length){
    try { F.nodesDS.update(batch); } catch(_){}
  }
}

function _pushNodeStyleUpdates(F, layoutMode){
  const opts = { fieldId: F.id };
  if(layoutMode) opts.layoutMode = layoutMode;
  const batch = F.words.map(w => {
    const variant = w._layoutShared ? 'shared' : 'normal';
    return buildNodeSpec(w, variant, opts);
  });
  if(!batch.length) return;
  // vis.DataSet.update fa shallow merge: tornando a dimensional, campi
  // rect-only (margin, shapeProperties) restano cached e contaminano il
  // rendering. Remove+add ricostruisce la spec da zero — niente residui.
  if(layoutMode === 'dimensional'){
    const ids = batch.map(s => s.id);
    try { F.nodesDS.remove(ids); F.nodesDS.add(batch); } catch(_){}
  } else {
    try { F.nodesDS.update(batch); } catch(_){}
  }
}
