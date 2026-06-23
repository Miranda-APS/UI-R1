// Stile nodi e archi per vis-network. FONTE UNICA. Nessun altro modulo
// deve costruire oggetti { color, font, shadow, borderWidth } per vis
// (vedi CLAUDE.md §2). Se serve un nuovo look, aggiungi qui una variante.

import { DIM_COLORS, UI, tokens, colorForSig } from './theme.js';
import { DIM_NAMES, REL_GROUP, GROUP_DASH } from './constants.js';

// ---- Helper interni ------------------------------------------------------

// Colore di una word basato sulla firma. Memoizzato per riferimento di firma
// (WeakMap): la dimensione dominante non cambia finché la firma è la stessa
// array, e ricostruire lo spec di molti nodi non deve ricalcolare il colore
// ogni volta. Se la firma viene sostituita (editing), entra una nuova chiave.
const _colorBySig = new WeakMap();
function colorFor(word){
  const sig = word.sig;
  if(!sig) return colorForSig(sig);
  let c = _colorBySig.get(sig);
  if(c === undefined){ c = colorForSig(sig); _colorBySig.set(sig, c); }
  return c;
}

// Dimensione base di un nodo in base al Field e ai flag.
// Le parole `fromExpansion` (vasto-clone temporaneo nel personale durante
// la fase di comprensione frase) usano la formula del vasto: grado globale
// del KG, non grado locale del personale. Solo le parole "vere" del personale
// (frase + vicini importati + create dall'utente) usano la formula locale.
export function sizeFor(word, fieldId){
  if(word.flags?.unknown) return tokens.nodeSize.unknown;
  if(fieldId === 'vasto' || word.flags?.fromExpansion){
    const deg = word.deg || 1;
    const s = 2 + Math.sqrt(deg) * 0.5;
    return Math.min(tokens.nodeSize.max, Math.max(tokens.nodeSize.min, s));
  }
  // Nel personale la dimensione è proporzionale al grado locale.
  const localDeg = word.localDegree || 0;
  return Math.min(20, Math.max(6, 4 + Math.sqrt(localDeg) * 2.5));
}

// Spec "color block" per vis — stesso colore in background/border/highlight/hover.
function colorBlock(bg, border){
  return {
    background: bg, border,
    highlight: { background: bg, border },
    hover:     { background: bg, border },
  };
}

// Font di una label standard (parola non selezionata).
// Color = colore della firma del nodo (stessa logica del vasto via overlay):
// la label si legge nello stesso colore del pallino, con stroke scuro per
// leggibilità su sfondo scuro.
function labelFont(isSentenceWord, color){
  if(isSentenceWord){
    return {
      size: tokens.font.sentenceLabel,
      color: UI.textBright,
      face: 'JetBrains Mono',
      strokeWidth: tokens.stroke.textStrong,
      strokeColor: UI.sentenceTextStroke,
      vadjust: tokens.vadjust.sentenceLabel,
      mod: 'bold',
    };
  }
  return {
    size: tokens.font.labeled,
    color: color || UI.textLabel,
    face: 'JetBrains Mono',
    strokeWidth: tokens.stroke.textSoft,
    strokeColor: UI.sentenceTextStrokeSoft,
    vadjust: tokens.vadjust.label,
  };
}

// Font "boldfaced" per nodo attivo (selezionato).
function activeFont(isRect = false){
  const sz = isRect ? tokens.font.labeled : tokens.font.activeLabel;
  return {
    size: sz,
    color: UI.textBright,
    strokeWidth: isRect ? 0 : 3,
    strokeColor: isRect ? 'transparent' : UI.bgStroke,
    bold: {
      size: sz,
      face: 'JetBrains Mono',
      color: UI.textBright,
      vadjust: 0,
      mod: 'bold',
    },
  };
}

// Font "boldfaced" per marker "!" (parola unknown).
function unknownFont(activeSize = false){
  const sz = activeSize ? tokens.font.unknownActive : tokens.font.unknownLabel;
  return {
    size: sz,
    color: UI.unknownText,
    bold: {
      size: sz, face: 'JetBrains Mono', color: UI.unknownText, mod: 'bold',
    },
  };
}

// Shadow per parole "from sentence".
function sentenceShadow(which = 'high'){
  const colorMap = {
    high:    UI.sentenceShadowHigh,
    low:     UI.sentenceShadowLow,
    strong:  UI.sentenceShadowStrong,
  };
  const size = which === 'low' ? tokens.shadow.sentenceDimmed : tokens.shadow.sentenceActive;
  return { enabled: true, color: colorMap[which] || colorMap.high, size, x: 0, y: 0 };
}

// ---- Build spec nodo ----------------------------------------------------

// buildNodeSpec(word, variant, opts)
//   word   : { w, sig, position: {x,y}, flags: {unknown, fromSentence, ...} }
//   variant: 'normal' | 'active' | 'rosa' | 'dimmed' | 'filterDim' | 'dragTarget'
//            (la variante 'dim-label' usa addDimLabel separato, vedi sotto)
//   opts   : { fieldId, showLabel?, layoutMode? }
// - fieldId: obbligatorio per sizing.
  // - layoutMode='rectangular': scale-up nodi/font (proiezione, leggibilità).
// Ritorna l'oggetto pronto per vis.DataSet.add()/update().
export function buildNodeSpec(word, variant, opts = {}){
  const { fieldId, showLabel, layoutMode, forceLabel, inPath } = opts;
  // Modalità rectangular del nuovo: scale ASIMMETRICO.
  // Si applica SOLO al campo nuovo, mai al campo vasto.
  const isRect = fieldId === 'nuovo' && layoutMode === 'rectangular';
  const isSentence_ = !!word.flags?.fromSentence;
  const SCALE      = isRect ? (isSentence_ ? 1.5 : 0.45) : 1.0;
  // Nel nuovo dimensionale le label a riposo risultavano più piccole rispetto
  // allo stato 'active' visto durante l'animazione di creazione (font.activeLabel
  // 30 vs labeled 16 / sentenceLabel 22): boost di leggibilità, solo nuovo (il
  // vasto a 9k nodi resta invariato).
  const FONT_SCALE = isRect ? (isSentence_ ? 1.3 : 1.0)
                   : (fieldId === 'nuovo' ? 1.25 : 1.0);
  const flags = word.flags || {};
  const color = colorFor(word);
  const size  = sizeFor(word, fieldId);
  // Label visibili solo nel personale, MA non per le parole vasto-clone
  // (fromExpansion=true) durante la fase di comprensione frase: 3000 etichette
  // sovrapposte sarebbero illeggibili e nasconderebbero l'animazione.
  // forceLabel: override per linkMode — durante il "collega" l'utente
  // deve vedere le etichette di TUTTE le parole per sapere con cosa
  // collegare (le sole parole della frase non bastano).
  const labelVisible = forceLabel ? true
                      : (showLabel !== undefined ? showLabel
                      : (fieldId === 'nuovo' && !flags.fromExpansion));

  // Posizione (solo per add iniziale; gli update non la ritoccano)
  const position = word.position || {};

  // --- Variant: unknown ---
  if(flags.unknown){
    switch(variant){
      case 'active':
        return {
          id: word.w, label: '!', hidden: false,
          color: colorBlock(UI.unknownBg, UI.textBright),
          size: tokens.nodeSize.activeUnknown,
          borderWidth: tokens.border.activeUnknown,
          opacity: tokens.opacity.full,
          font: unknownFont(true),
        };
      case 'dimmed':
        return {
          id: word.w, label: '!',
          color: colorBlock(UI.unknownBgDimmed, UI.unknownBorderDimmed),
          size: tokens.nodeSize.dimmedUnknown,
          borderWidth: tokens.border.dimmedUnknown,
          opacity: tokens.opacity.dimmedUnknown,
          font: { size: tokens.font.dimmedUnknown, color: UI.unknownText },
        };
      // normal | rosa | altri fall through al marker giallo base
      default: {
        const spec = {
          id: word.w, label: '!', shape: 'square',
          size: tokens.nodeSize.unknown,
          color: {
            background: UI.unknownBg, border: UI.unknownBorder,
            highlight: { background: UI.unknownBg,      border: UI.unknownBorder },
            hover:     { background: UI.unknownBgHover, border: UI.unknownBorder },
          },
          font: unknownFont(false),
          borderWidth: tokens.border.unknown,
          opacity: tokens.opacity.full,
          hidden: false,
        };
        if(variant === 'normal' && position.x != null){
          // fixed: true solo in vasto (read-only). In nuovo i nodi sono
          // draggabili dall'utente per riorganizzare il layout.
          spec.x = position.x; spec.y = position.y; spec.fixed = (fieldId === 'vasto');
        }
        return spec;
      }
    }
  }

  // --- Variant: dragTarget (feedback temporaneo durante drag-to-connect) ---
  if(variant === 'dragTarget'){
    return {
      id: word.w,
      color: colorBlock(color, UI.sentenceGlow),
      borderWidth: tokens.border.dragTarget,
      shadow: { enabled: true, color: UI.sentenceShadowStrong, size: tokens.shadow.dragTarget, x: 0, y: 0 },
    };
  }

  // --- Variant: active (selezionato) ---
  // label='': il testo viene disegnato dall'overlay (components/overlay.js)
  // con sfondo + anti-collisione. Evita doppioni con vis (tranne in rectangular mode).
  if(variant === 'active'){
    const actScale = isRect ? 1.0 : SCALE;
    const baseSpec = {
      id: word.w, label: isRect ? (word.displayName || word.w) : '', hidden: false,
      color: colorBlock(color, UI.textBright),
      size: Math.round(tokens.nodeSize.active * actScale),
      borderWidth: tokens.border.active,
      opacity: tokens.opacity.full,
      font: activeFont(isRect),
    };
    if (isRect) {
      const borderCol = inPath ? UI.pathGlow : (isSentence_ ? UI.sentenceGlow : color);
      baseSpec.shape = 'box';
      baseSpec.margin = isSentence_ ? { top: 8, bottom: 8, left: 12, right: 12 } : { top: 6, bottom: 6, left: 10, right: 10 };
      baseSpec.shapeProperties = { borderRadius: 6 };
      baseSpec.color.background = 'rgba(60, 60, 80, 0.95)';
      baseSpec.color.border = borderCol;
      baseSpec.font.color = UI.textBright;
      baseSpec.font.strokeWidth = 0;
      baseSpec.size = Math.round(tokens.nodeSize.normal * SCALE); // NIENTE INGRANDIMENTO
      if(inPath){
        baseSpec.borderWidth = 3;
        baseSpec.shadow = { enabled: true, color: UI.pathShadow, size: 14, x: 0, y: 0 };
      }
    } else if(inPath){
      // Modalità non-rectangular: alone verde sopra l'active default per
      // segnalare "questa parola fa parte della catena di click".
      baseSpec.shadow = { enabled: true, color: UI.pathShadow, size: 16, x: 0, y: 0 };
    }
    return baseSpec;
  }

  // --- Variant: rosa (vicini della parola attiva) ---
  // label='' per la stessa ragione di 'active' — overlay-driven.
  if(variant === 'rosa'){
    const spec = {
      id: word.w, label: isRect ? (word.displayName || word.w) : '',
      color: colorBlock(color, color),
      size: tokens.nodeSize.rosa,
      borderWidth: tokens.border.normal,
      opacity: tokens.opacity.full,
      font: { size: isRect ? tokens.font.labeled : tokens.font.sentenceLabel, color: UI.textBright },
    };
    if(flags.fromSentence){
      spec.color = colorBlock(color, UI.sentenceGlow);
      spec.borderWidth = tokens.border.fromSentenceRosa;
      spec.shadow = sentenceShadow('low');
    }
    if (isRect) {
      spec.shape = 'box';
      spec.margin = isSentence_ ? { top: 8, bottom: 8, left: 12, right: 12 } : { top: 6, bottom: 6, left: 10, right: 10 };
      spec.shapeProperties = { borderRadius: 6, borderDashes: opts.borderDashes || false };
      spec.color.background = 'rgba(22, 22, 34, 0.95)';
      spec.font.color = UI.textBright;
      spec.font.strokeWidth = 0;
      spec.size = Math.round(tokens.nodeSize.normal * SCALE); // NIENTE INGRANDIMENTO
      spec.borderWidth = 3; // Bordo leggermente più spesso per evidenziare il tratteggio
    }
    return spec;
  }

  // --- Variant: dimmed (tutte le altre parole quando una è selezionata) ---
  if(variant === 'dimmed'){
    if(flags.fromSentence){
      return {
        id: word.w, label: '',
        color: {
          background: color, border: UI.sentenceGlow,
          hover: { background: color, border: UI.sentenceGlow },
        },
        borderWidth: tokens.border.dimmedSentence,
        opacity: tokens.opacity.dimmedSentence,
      };
    }
    return {
      id: word.w, label: '',
      color: {
        background: color, border: color,
        hover: { background: color, border: color },
      },
      borderWidth: tokens.border.dimmed,
      opacity: tokens.opacity.dimmed,
    };
  }

  // --- Variant: filterDim (visibile ma attenuato perché non passa il filtro) ---
  if(variant === 'filterDim'){
    return {
      id: word.w, label: '', hidden: false,
      color: colorBlock(color, color),
      borderWidth: tokens.border.filterDim,
      opacity: tokens.opacity.filterDim,
      font: { size: tokens.font.filterHidden, color: UI.transparentText },
    };
  }

  // --- Variant: normal (default) ---
  // 'shared' = vicino di più parole-frase nel layout rettangolare (stellina).
  // Stesso colore della firma, label normale, shape 'star' al posto di 'dot'.
  const isSentence = !!flags.fromSentence;
  const isShared   = variant === 'shared';
  const borderCol = isSentence ? UI.sentenceGlow : color;
  const dotSize = isSentence ? Math.max(size, tokens.nodeSize.fromSentence) : size;

  const baseSize = isShared ? Math.max(dotSize + 2, 10) : dotSize;
  const baseFont = labelVisible
    ? labelFont(isSentence, color)
    : { size: tokens.font.unlabeled, color: UI.textBright };
  // Applica scale rectangular su size + font.size (font è oggetto, copia).
  const scaledFont = SCALE !== 1.0 || FONT_SCALE !== 1.0
    ? { ...baseFont, size: Math.round((baseFont.size || 14) * FONT_SCALE) }
    : baseFont;

  const spec = {
    id: word.w,
    // Display: forma originale come scritta dall'utente (con articoli) per
    // parole della frase, lemma per il resto.
    label: labelVisible ? (word.displayName || word.w) : '',
    size: Math.round(baseSize * SCALE),
    color: colorBlock(color, borderCol),
    // Perf vasto: per i dot a riposo il bordo è già === fill (invisibile) →
    // borderWidth 0 elimina lo stroke per-nodo (9k stroke()/frame in meno) a
    // parità di resa. Le parole-frase tengono il loro bordo glow.
    borderWidth: isSentence ? tokens.border.fromSentence
               : (fieldId === 'vasto' ? tokens.border.vastoDot : tokens.border.normal),
    opacity: tokens.opacity.normal,
    font: scaledFont,
  };

  if(isRect) {
    spec.shape = 'box';
    spec.margin = isSentence ? { top: 8, bottom: 8, left: 12, right: 12 } : { top: 6, bottom: 6, left: 10, right: 10 };
    spec.shapeProperties = { borderRadius: 6, borderDashes: false }; // Default continuo
    
    spec.color = {
      background: 'rgba(22, 22, 34, 0.95)',
      border: borderCol,
      highlight: { background: 'rgba(40, 40, 60, 0.95)', border: borderCol },
      hover: { background: 'rgba(40, 40, 60, 0.95)', border: borderCol },
    };
    
    spec.font = {
      ...scaledFont,
      color: UI.textBright, // Testo SEMPRE bianco per contrasto col bordo colorato
      strokeWidth: 0,
      vadjust: 0,
    };
    
    // In modalità rectangular, non vogliamo ingrandimenti / cambi di bordo esagerati all'hover.
    // L'evidenziazione la fa il focus o il dimming del resto.
    if(variant === 'active') {
      spec.borderWidth = (spec.borderWidth || 1) + 2;
      spec.color.background = 'rgba(60, 60, 80, 0.95)';
    } else if (variant === 'dimmed') {
      spec.opacity = tokens.opacity.dimmed;
      spec.color.border = UI.dimmedBorder;
    }
    
    // Forza la label ad esserci sempre nei box rettangolari (anche se è 'active' o 'rosa'),
    // perché l'overlay label non la disegnerà più per questa modalità.
    spec.label = word.displayName || word.w;
  } else {
    if(isShared) spec.shape = 'star';
    else spec.shape = 'dot';
  }

  if(isSentence && !isRect) spec.shadow = sentenceShadow('high');
  if(position.x != null){
    // fixed: true solo in vasto (read-only). In nuovo i nodi sono
    // draggabili → physics off + niente fixed = restano fermi finché
    // non li trascini.
    spec.x = position.x; spec.y = position.y;
    spec.fixed = (fieldId === 'vasto');
    spec.hidden = false;
  }
  return spec;
}

// Etichetta "compass" ai bordi del campo (POTERE, MATERIA, ecc.).
// Non è una parola — è una label fissa di riferimento.
export function buildDimLabelSpec(idx, x, y){
  return {
    id: '_dim_label_' + idx,
    x, y, fixed: true, physics: false, chosen: false,
    shape: 'text',
    label: DIM_NAMES[idx].toUpperCase(),
    font: {
      size: tokens.font.dimLabel,
      color: DIM_COLORS[idx],
      face: 'JetBrains Mono',
      bold: '700',
    },
    _dimLabel: true,
  };
}

// ---- Build spec arco -----------------------------------------------------

// buildEdgeSpec(edge, opts)
//   edge: { from, to, rel, conf }  (key = from|to|rel, usata come id)
//   opts: { hidden?, variant? }
// variant: 'normal' (default, nascosto) | 'hover' | 'hoverSub' | 'focus' | 'filterShown'
export function buildEdgeSpec(edge, opts = {}){
  const key = `${edge.from}|${edge.to}|${edge.rel}`;
  const group = REL_GROUP[edge.rel] || 'L';
  const dash = GROUP_DASH[group] || false;
  const width = Math.max(tokens.edge.widthMin, ((edge.conf || 50) / 100) * 0.4 * 4);
  const variant = opts.variant || 'normal';
  const isRect = opts.fieldId === 'nuovo' && opts.layoutMode === 'rectangular';

  // Layout rettangolare: la baseline degli archi resta nascosta (le colonne
  // sono già una rappresentazione delle relazioni — gli archi sopra
  // diventerebbero un labirinto). MA su hover/selection vanno mostrati,
  // altrimenti la collega non vede che "angoscia" è connessa a "futuro"
  // quando ci passa sopra. Quindi: short-circuit a hidden SOLO per variant
  // 'normal'; tutti gli altri variant (hover, subFocus, hoverSub, selection,
  // focus, dim, filterShown) cadono nel render normale sotto.
  if (isRect && variant === 'normal') {
    return { id: key, hidden: true, color: { opacity: 0 } };
  }

  const arrows = group === 'C'
    ? { to: { enabled: true, scaleFactor: tokens.edge.arrowCausal + width * tokens.edge.arrowCausalFactor } }
    : { to: { enabled: true, scaleFactor: tokens.edge.arrowOther } };

  // Nel layout rectangular, rendiamo gli archi curvi (cubicBezier verticale)
  // per ridurre l'intersezione tra le colonne e li facciamo più tenui di default.
  const smooth = isRect 
    ? { type: 'cubicBezier', forceDirection: 'vertical', roundness: 0.6 }
    : false;

  const base = {
    id: key,
    from: edge.from,
    to: edge.to,
    width,
    dashes: dash,
    arrows,
    smooth,
    font: { size: 0 },
    label: '',
  };

  if(variant === 'hover'){
    return { ...base, hidden: false, color: { color: UI.edgeHover, opacity: tokens.edge.opacityHover }, width: tokens.edge.widthHover };
  }
  if(variant === 'hoverSub'){
    return { ...base, hidden: false, color: { color: UI.edgeSub, opacity: tokens.edge.opacityFocus }, width: tokens.edge.widthBase };
  }
  if(variant === 'focus'){
    return { ...base, hidden: false, color: { color: UI.edgeFocus, opacity: tokens.edge.opacityFocus }, width: tokens.edge.widthFocus };
  }
  if(variant === 'subFocus'){
    return { ...base, hidden: false, color: { color: UI.edgeDefault, opacity: tokens.edge.opacitySub }, width: tokens.edge.widthSub };
  }
  if(variant === 'dim'){
    return { ...base, hidden: false, color: { color: UI.edgeDim, opacity: tokens.edge.opacityDim }, width: tokens.edge.widthMin };
  }
  if(variant === 'filterShown'){
    return { ...base, hidden: false, color: { color: UI.edgeHover, opacity: isRect ? 0.15 : tokens.edge.opacityFiltered }, width: tokens.edge.widthBase };
  }
  if(variant === 'selection'){
    return { ...base, hidden: false, color: { color: UI.edgeHover, opacity: tokens.edge.opacityFiltered }, width: tokens.edge.widthHover };
  }

  // 'normal' — baseline: nascosto, visibile solo su mouseover
  return {
    ...base,
    hidden: opts.hidden !== undefined ? opts.hidden : true,
    color: { color: UI.edgeDefault, opacity: isRect ? 0.15 : tokens.edge.opacityBase, highlight: UI.edgeHighlight },
  };
}

// ---- Semplici helper di update parziale -----------------------------------
// Quando si vuole solo cambiare hidden/opacity senza ricostruire l'intero spec.

export function edgeVisibilityUpdate(key, { hidden, variant }){
  if(variant){
    return buildEdgeSpec({ from: '_', to: '_', rel: '_' }, { variant });
  }
  return { id: key, hidden };
}

// Update parziale per il SOLO colore di un nodo (background+border+highlight).
// Usato da Field.updateWordSig dopo che la firma è cambiata: la dimensione
// dominante si sposta, il colore deve seguirla in tempo reale.
// Posizione e size vengono aggiornati separatamente da chi conosce il rank.
export function nodeColorUpdate(wordId, sig){
  const c = colorForSig(sig);
  return {
    id: wordId,
    color: colorBlock(c, c),
  };
}
