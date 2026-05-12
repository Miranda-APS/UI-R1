// Nomi, costanti e mapping del dominio. Nessun colore o hex — quelli stanno
// in theme.js (vedi CLAUDE.md §1).

// Le 8 dimensioni fenomenologiche (nome canonico).
export const DIM_NAMES  = ['potere','materia','ardore','divenire','spazio','intreccio','verità','armonia'];

// Descrizione leggibile di ogni dimensione.
export const DIM_DESC   = ['agisce o subisce','permanenza o evanescenza','movimento o inerzia','futuro o passato','grande o piccolo','complesso o semplice','definito o vago','attrae o respinge'];

// Trigram I Ching associato a ogni dimensione.
export const DIM_ICHING = ['☰ cielo','☷ terra','☳ tuono','☵ acqua','☶ montagna','☴ vento','☲ fuoco','☱ lago'];

// Disposizione angolare (clockwise dall'alto) di ogni dimensione nel campo 2D.
// Indicizzato come sig: [agency, permanenza, intensità, tempo, confine, complessità, definizione, valenza]
export const DIM_ANGLES = [
  -Math.PI/2,    // 0 potere    → top
   Math.PI/4,    // 1 materia   → bottom-right
  -Math.PI/4,    // 2 ardore    → top-right
   0,            // 3 divenire  → right
   3*Math.PI/4,  // 4 spazio    → bottom-left
  -3*Math.PI/4,  // 5 intreccio → top-left
   Math.PI,      // 6 verità    → left
   Math.PI/2,    // 7 armonia   → bottom
];

// Octalysis — nomi dei core drive (ordine fisso CD1..CD8).
export const CD_NAMES = ['significato epico','empowerment','influenza sociale','imprevedibilità','evitamento','scarsità','possesso','realizzazione'];

// Gruppi di relazione e stile tratto (S/C/M/F/L).
export const REL_GROUP = {
  IS_A:'S', PART_OF:'S',
  CAUSES:'C', ENABLES:'C', REQUIRES:'C', TRANSFORMS_INTO:'C',
  SIMILAR_TO:'M', OPPOSITE_OF:'M', SYMBOLIZES:'M', CONTEXT_OF:'M',
  HAS:'F', DOES:'F', FEELS_AS:'F', WONDERS_ABOUT:'F', REMEMBERS_AS:'F',
  USED_FOR:'L', EXPRESSES:'L', IMPLIES:'L', EQUIVALENT:'L', EXCLUDES:'L', COEXISTS:'L',
};
export const GROUP_DASH = { S:[5,5], C:false, M:[3,6], F:[2,4], L:[12,6] };

// Metadati delle 5 famiglie di relazione: label italiana + classe CSS della
// linea della legenda (vedi style.css `.legend-line.<class>`). Ordine di
// presentazione nella sidebar.
export const REL_GROUPS_INFO = [
  { id: 'S', label: 'strutturale',     cssClass: 'structural' },
  { id: 'C', label: 'causale',         cssClass: 'causal' },
  { id: 'M', label: 'semantica',       cssClass: 'semantic' },
  { id: 'F', label: 'fenomenologica',  cssClass: 'phenomenological' },
  { id: 'L', label: 'logica',          cssClass: 'logical' },
];

// Inverso di REL_GROUP: gruppo → array di tipi (preservando l'ordine di
// dichiarazione di REL_GROUP). Usato dalla legenda interattiva per popolare
// i sotto-toggle per tipo specifico.
export const TYPES_BY_GROUP = (() => {
  const out = {};
  for(const [rel, group] of Object.entries(REL_GROUP)){
    if(!out[group]) out[group] = [];
    out[group].push(rel);
  }
  return out;
})();

// Etichette italiane per relazioni mostrate nel menu utente.
export const RL = {
  IS_A:'è un', HAS:'ha', DOES:'fa', PART_OF:'parte di',
  CAUSES:'causa', ENABLES:'abilita', REQUIRES:'richiede', TRANSFORMS_INTO:'diventa',
  SIMILAR_TO:'simile a', OPPOSITE_OF:'opposto di',
  USED_FOR:'usato per', EXPRESSES:'esprime',
};

// Mappa estesa relazione → verbo leggibile (usata dal pannello "UI-R1 legge").
export const REL_LABEL_IT = {
  IS_A: 'è', HAS: 'ha', DOES: 'fa', PART_OF: 'parte di',
  CAUSES: 'causa', ENABLES: 'abilita', REQUIRES: 'richiede', TRANSFORMS_INTO: 'diventa',
  SIMILAR_TO: 'simile a', OPPOSITE_OF: 'opposto di', USED_FOR: 'usato per',
  EXPRESSES: 'esprime', SYMBOLIZES: 'simbolizza', CONTEXT_OF: 'contesto di',
  FEELS_AS: 'sente come', WONDERS_ABOUT: 'si chiede di', REMEMBERS_AS: 'ricorda come',
};

// Etichette per la legenda interattiva (#relLegend). Italiano completo per
// TUTTI i tipi noti — l'utente vede le pill cliccabili e deve capire al
// volo che relazione sta filtrando.
export const REL_LEGEND_LABEL_IT = {
  IS_A: 'è un', PART_OF: 'parte di',
  CAUSES: 'causa', ENABLES: 'abilita', REQUIRES: 'richiede', TRANSFORMS_INTO: 'diventa',
  SIMILAR_TO: 'simile a', OPPOSITE_OF: 'opposto di', SYMBOLIZES: 'simbolizza', CONTEXT_OF: 'contesto di',
  HAS: 'ha', DOES: 'fa', FEELS_AS: 'sente come', WONDERS_ABOUT: 'si chiede', REMEMBERS_AS: 'ricorda',
  USED_FOR: 'usato per', EXPRESSES: 'esprime', IMPLIES: 'implica',
  EQUIVALENT: 'equivale a', EXCLUDES: 'esclude', COEXISTS: 'coesiste con',
};

// Scala spaziale del campo (raggio massimo del layout).
export const R = 550;

// Firma neutra: tutte le dimensioni a metà.
export const NEUTRAL_SIG = [50,50,50,50,50,50,50,50];

// Chiavi di persistenza (localStorage).
export const LS = {
  NUOVO:      'uir1_nuovo_v1',
  SIDEBAR_W:      'uir1_sidebar_w',
  SIDEBAR_COLLAPSED: 'uir1_sidebar_collapsed',
  NUOVO_LAYOUT:   'uir1_nuovo_layout',  // 'dimensional' | 'rectangular'
};

// Schema version del payload di un Field salvato in LS.NUOVO. Bumpare ogni
// volta che la struttura serializzata cambia in modo incompatibile. Al boot,
// loadField scarta payload con version diverso (il campo torna vuoto, ma
// l'utente non vede UI rotta da uno stato inconsistente).
export const LS_SCHEMA_VERSION = 2;

// Pesi tipo-relazione per "forza" di un arco quando un satellite è
// connesso a più parole-frase (layout rettangolare). Coerente con la
// gerarchia usata per la generazione testuale (vedi CLAUDE.md §21):
// CAUSES dominante, IS_A/DOES forti, SIMILAR_TO debole.
export const REL_WEIGHT = {
  CAUSES: 1.0, ENABLES: 0.95, REQUIRES: 0.9, TRANSFORMS_INTO: 0.9,
  IS_A: 0.9, DOES: 0.9, HAS: 0.85,
  USED_FOR: 0.8, PART_OF: 0.8,
  EXPRESSES: 0.75, SYMBOLIZES: 0.75, IMPLIES: 0.75, EQUIVALENT: 0.75,
  CONTEXT_OF: 0.7, OPPOSITE_OF: 0.7,
  FEELS_AS: 0.65, WONDERS_ABOUT: 0.65, REMEMBERS_AS: 0.65,
  EXCLUDES: 0.6, COEXISTS: 0.55,
  SIMILAR_TO: 0.4,
};
