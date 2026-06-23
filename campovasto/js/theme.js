// Fonte unica di verità per colori, token, animazioni.
// Ogni hex o rgba di campovasto/ vive qui e solo qui (vedi CLAUDE.md §1).
// Le custom property CSS vengono generate all'avvio da applyThemeToCss().

// ---- Colori delle 8 dimensioni I Ching (ordine DIM_NAMES canonico) ----
// Indicizzato con DIM_NAMES: potere, materia, ardore, divenire, spazio,
// intreccio, verità, armonia.
export const DIM_COLORS = [
  '#5AAFE8',  // 0 potere    — trigram Cielo
  '#E89A3A',  // 1 materia   — trigram Terra
  '#E04848',  // 2 ardore    — trigram Fuoco
  '#A86EDB',  // 3 divenire  — trigram Acqua
  '#C8C4BC',  // 4 spazio    — trigram Pietra
  '#4AC880',  // 5 intreccio — trigram Foresta
  '#E8D040',  // 6 verità    — trigram Sole
  '#E87AAA',  // 7 armonia   — trigram Tramonto
];

// ---- Colori Octalysis (CD1..CD8), layout distinto dalle dimensioni ----
export const CD_COLORS = [
  '#5AAFE8',  // significato epico
  '#4AC880',  // empowerment
  '#E87AAA',  // influenza sociale
  '#A86EDB',  // imprevedibilità
  '#C8C4BC',  // evitamento
  '#E04848',  // scarsità
  '#E8D040',  // possesso
  '#E89A3A',  // realizzazione
];

// ---- Colori dei trigrammi (indicizzati per id binario, NON per DIM_NAMES) --
// Usati dalla lente 3D (frattali.html): un FractalId = lower×8 + upper, e il
// colore viene dal trigramma `lower`. Distinti da DIM_COLORS, che è indicizzato
// per dimensione-firma (permutazione diversa — vedi Phase 68). Non confonderli.
// Ordine binario: 0 Cielo, 1 Terra, 2 Tuono, 3 Acqua, 4 Montagna, 5 Vento,
// 6 Fuoco, 7 Lago. Sono anche la palette delle relazioni KG (isa=Vento/verde,
// ha=Terra/blu, fa=Tuono/giallo, causa=Fuoco/arancio, simile=Acqua/ciano…).
export const TRIG_COLORS = [
  '#ff6b6b',  // 0 Cielo    ☰
  '#5aadff',  // 1 Terra    ☷
  '#ffd04a',  // 2 Tuono    ☳
  '#4affda',  // 3 Acqua    ☵
  '#a06eff',  // 4 Montagna ☶
  '#70d060',  // 5 Vento    ☴
  '#ff9a5a',  // 6 Fuoco    ☲
  '#ff6eb0',  // 7 Lago     ☱
];

// ---- Colori di interfaccia (nomi semantici, mai usare hex diretti) ----
export const UI = {
  // Sfondi e superfici
  bg:            '#0f0f1a',
  surface:       '#1a1a2e',
  surfaceHover:  '#3a3a6e',
  border:        '#2a2a4e',
  borderHover:   '#5a5a8e',
  separator:     '#333',

  // Testo
  text:          '#e0e0e0',
  textBright:    '#fff',
  textDim:       '#888',
  textLabel:     '#d8d8d8',
  textMuted:     '#aaa',
  textVeryDim:   '#666',

  // Ancore visive (marker "unknown": quadrato giallo con "!")
  unknownBg:        '#ffcc00',
  unknownBorder:    '#ff9900',
  unknownBgHover:   '#ffdd33',
  unknownBgDimmed:  '#997700',
  unknownBorderDimmed: '#664400',
  unknownText:      '#000',

  // "Sentence-word" (parola della frase nel campo medio): alone dorato
  sentenceGlow:        '#ffd560',
  sentenceShadowHigh:  'rgba(255, 213, 96, 0.85)',
  sentenceShadowLow:   'rgba(255, 213, 96, 0.55)',
  sentenceShadowDrag:  'rgba(255, 213, 96, 0.8)',
  sentenceShadowStrong:'rgba(255, 213, 96, 0.9)',
  sentenceTextStroke:  'rgba(0,0,0,0.9)',
  sentenceTextStrokeSoft: 'rgba(0,0,0,0.8)',

  // "Path glow" — alone delle parole del breadcrumb attivo (catena di click).
  // Distinto dal dorato delle sentence-words per evitare ambiguità: il giallo
  // dice "questa parola viene dalla frase", il verde "questa parola è nel
  // mio percorso di click corrente".
  pathGlow:            '#4AC880',
  pathShadow:          'rgba(74, 200, 128, 0.75)',

  // Archi
  edgeDefault:   '#555',
  edgeHover:     '#aaa',
  edgeHighlight: '#999',
  edgeDim:       '#444',
  edgeFallback:  '#ccc',
  edgeFocus:     '#bbb',
  edgeSub:       '#ccc',

  // Radar Octalysis
  radarRingOuter:  'rgba(255,255,255,0.55)',
  radarRingInner:  'rgba(255,255,255,0.30)',
  radarSpokes:     'rgba(255,255,255,0.35)',
  radarFill:       'rgba(90,175,232,0.32)',
  radarStroke:     'rgba(140,200,245,1)',
  radarGlow:       'rgba(90,175,232,0.8)',

  // Stati interattivi
  savedItemHover:   'rgba(255,255,255,0.04)',
  savedItemActive:  'rgba(90,175,232,0.12)',

  // Sidebar h3 (heading di sezione)
  h3Color:          'rgba(120,140,200,.5)',

  // Toolbar grafo (overlay #graphToolbar)
  toolbarBg:        'rgba(18, 20, 40, 0.88)',
  toolbarBorder:    'rgba(255,255,255,0.12)',
  toolbarHover:     'rgba(255,255,255,0.10)',
  toolbarActive:    'rgba(255,255,255,0.18)',

  // Pannelli laterali (#filterPanel, #sentencePanel, …)
  pannelloSfondo:   'rgba(18, 18, 34, 0.85)',
  bordoHover:       '#444',
  testoTenue:       '#555',
  chipAttivoSfondo: 'rgba(255,255,255,0.12)',
  cursorePista:     'rgba(255,255,255,0.10)',
  cursoreOmbra:     'rgba(0,0,0,0.3)',

  // Cursore (controllo lineare unificato — vedi regole di design.md §3)
  poloTesto:        '#fff',
  poloOmbra:        'rgba(0,0,0,0.95)',

  // Fallback node/edge (quando non abbiamo un colore dimensione)
  fallbackNode:  '#666',

  // Overlay invisibile (testo nascosto)
  transparentText: 'rgba(0,0,0,0)',
};

// ---- Dimensioni / spaziature / tempi (dimensional tokens) ----
export const tokens = {
  nodeSize: {
    min:              3,
    max:              14,
    unknown:          14,
    fromSentence:     14,    // floor quando la parola è della frase
    active:           20,
    rosa:             14,
    dimmedUnknown:    12,
    activeUnknown:    18,
  },
  border: {
    normal:           1.5,
    vastoDot:         0,      // dot a riposo nel vasto: bordo===fill (invisibile) → 0 evita 9k stroke()/frame
    fromSentence:     4,
    fromSentenceRosa: 3,
    unknown:          2.5,
    active:           3,
    activeUnknown:    4,
    dimmed:           0.8,
    dimmedSentence:   2,
    dimmedUnknown:    2,
    dragTarget:       4,
    filterDim:        1,
  },
  font: {
    unlabeled:        18,    // nodo senza label visibile (vasto)
    labeled:          16,    // label standard (nuovo/medio)
    sentenceLabel:    22,    // label parola della frase
    unknownLabel:     22,    // "!" piccolo
    unknownActive:    26,    // "!" attivo
    activeLabel:      30,    // label di una parola selezionata
    dimLabel:         20,    // etichette delle dimensioni (compass)
    dimmedUnknown:    18,
    filterHidden:     1,
  },
  stroke: {
    textSoft:         4,
    textStrong:       5,
  },
  vadjust: {
    label:            -6,
    sentenceLabel:    -8,
  },
  opacity: {
    normal:           0.92,
    full:             1,
    dimmed:           0.22,
    dimmedSentence:   0.55,
    dimmedUnknown:    0.65,
    filterDim:        0.20,
  },
  shadow: {
    sentenceActive:   18,    // size del glow
    sentenceDimmed:   10,
    dragTarget:       18,
  },
  edge: {
    widthMin:         0.5,
    widthBase:        1,     // highlight post-filter
    widthHover:       1.2,
    widthSub:         0.6,
    widthFocus:       1.1,
    opacityBase:      0.5,
    opacityHover:     0.7,
    opacityFocus:     0.7,
    opacitySub:       0.25,
    opacityFiltered:  0.6,
    opacityDim:       0.15,
    arrowCausal:      0.4,
    arrowCausalFactor:0.08,
    arrowOther:       0.25,
  },
  anim: {
    mountFit:         500,    // ms per fit dopo mount
    overlayShow:      900,
    overlayFade:      400,
    autoplayStep:     1800,
  },
  spread: {
    minDist:          55,     // distanza minima per spreadNonOverlapping
    iterations:       50,
  },
};

// ---- Applica il tema come CSS custom properties ---------------------------
// Da chiamare una volta all'avvio, prima del primo render.
// Genera: --color-dim-<name>, --color-cd-<i>, --color-<ui-key>, --space-*,
//         --transition-smooth, ecc.
import { DIM_NAMES } from './constants.js';

// Mappa DIM_NAMES[i] → nome I Ching usato come alias nelle CSS var storiche.
// Il CSS di campovasto usa var(--colore-cielo) per la dimensione "potere", ecc.
// Mantiene il naming italiano per non riscrivere ~40 selettori — il mapping
// è esplicito e documentato, fonte di verità resta questo file.
const DIM_TO_ICHING = ['cielo','terra','fuoco','acqua','pietra','foresta','sole','tramonto'];

export function applyThemeToCss(){
  const root = document.documentElement;
  const set = (k, v) => root.style.setProperty(k, v);

  // Colori dimensioni: ogni dimensione ha DUE alias — canonico (per-name) e
  // storico trigram (usato dal CSS esistente).
  DIM_COLORS.forEach((c, i) => {
    set(`--color-dim-${DIM_NAMES[i]}`, c);
    set(`--colore-${DIM_TO_ICHING[i]}`, c);
  });
  CD_COLORS.forEach((c, i)  => set(`--color-cd-${i}`, c));

  // UI chrome: naming italiano storico (usato dal CSS).
  set('--sfondo-scuro',  UI.bg);
  set('--superficie',    UI.surface);
  set('--bordo',         UI.border);
  set('--testo-chiaro',  UI.text);
  set('--testo-basso',   UI.textDim);

  // UI chrome: naming tecnico (nuovi riferimenti).
  set('--color-bg',       UI.bg);
  set('--color-surface',  UI.surface);
  set('--color-border',   UI.border);
  set('--color-text',     UI.text);
  set('--color-text-dim', UI.textDim);

  // Spaziature.
  set('--spazio-piccolo',       '4px');
  set('--spazio-medio',         '8px');
  set('--spazio-grande',        '16px');
  set('--spazio-molto-grande',  '24px');
  set('--space-xs', '4px');
  set('--space-sm', '8px');
  set('--space-md', '16px');
  set('--space-lg', '24px');

  // Transizioni.
  set('--movimento-dolce',      '0.25s cubic-bezier(0.2, 0.9, 0.4, 1.1)');
  set('--transition-smooth',    '0.25s cubic-bezier(0.2, 0.9, 0.4, 1.1)');

  // Sidebar h3
  set('--h3-color',             UI.h3Color);

  // Toolbar grafo
  set('--color-toolbar-bg',     UI.toolbarBg);
  set('--color-toolbar-border', UI.toolbarBorder);
  set('--color-toolbar-hover',  UI.toolbarHover);
  set('--color-toolbar-active', UI.toolbarActive);

  // Pannelli laterali
  set('--pannello-sfondo',    UI.pannelloSfondo);
  set('--bordo-hover',        UI.bordoHover);
  set('--testo-tenue',        UI.testoTenue);
  set('--chip-attivo-sfondo', UI.chipAttivoSfondo);
  set('--cursore-pista',      UI.cursorePista);
  set('--cursore-ombra',      UI.cursoreOmbra);

  // Cursore (poli)
  set('--polo-testo',         UI.poloTesto);
  set('--polo-ombra',         UI.poloOmbra);

  // Tratti relazioni (stile dash).
  set('--rel-strutturale',      '5px dotted ' + UI.textDim);
  set('--rel-causale',          '3px solid '  + UI.textDim);
  set('--rel-semantica',        '4px dashed ' + UI.textDim);
  set('--rel-fenomenologica',   '2px dotted ' + UI.textDim);
  set('--rel-logica',           '2px dashed ' + UI.textDim);
}

// ---- Helper: colore di una firma 8D ---------------------------------------
// Usato dappertutto al posto di memorizzare `word._color`.
// Restituisce il colore della dimensione dominante.
export function colorForSig(sig){
  if(!sig || sig.length < 8) return DIM_COLORS[0];
  let best = 0, bestV = -1;
  for(let i = 0; i < 8; i++){
    const v = sig[i] || 0;
    if(v > bestV){ bestV = v; best = i; }
  }
  return DIM_COLORS[best];
}

// Indice (0..7) della dimensione dominante.
export function dominantDim(sig){
  if(!sig || sig.length < 8) return 0;
  let best = 0, bestV = -1;
  for(let i = 0; i < 8; i++){
    const v = sig[i] || 0;
    if(v > bestV){ bestV = v; best = i; }
  }
  return best;
}
