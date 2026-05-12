# STEERING — campovasto/

Regole di stile e struttura per il codice in questa cartella. Si applicano a ogni
modifica, senza eccezioni. Se una modifica le viola, **o la modifica cambia, o
la regola cambia** — mai tacitamente entrambe.

Questo file è il **contratto tra sessioni**. Leggilo prima di modificare
qualsiasi cosa in `campovasto/`.

---

## Perché queste regole esistono

Questo endpoint è cresciuto un pezzo alla volta. Il risultato: hex sparsi ovunque,
stili nodo costruiti inline in più file, `app.js` da 1000+ righe che mischia
bootstrap/wiring/transmit, convenzioni di naming miste. Queste regole chiudono
quei rubinetti. Se non le rispetti, in tre sessioni sei di nuovo nel caos.

---

## 1. Una sola fonte di verità per colori e token

- **Tutti gli hex vivono in [`js/theme.js`](js/theme.js) e solo lì.**
- `theme.js` esporta:
  - `DIM_COLORS[0..7]` — i colori delle 8 dimensioni I Ching.
  - `CD_COLORS[0..7]` — Octalysis.
  - `UI` — colori di interfaccia con nomi semantici (`glow`, `textDim`,
    `unknownBg`, `linkNeutral`, `shadowSoft`, …).
  - `tokens` — dimensioni, durate, z-index (`nodeSizeMin`, `nodeSizeMax`,
    `shadowSizeStrong`, `anim.fast`, …).
- **Zero hex letterali** in qualunque altro file `.js`/`.css` di questa cartella.
  Se serve un colore nuovo, aggiungilo a `theme.js` con un nome semantico, poi
  usa quel nome. Non c'è alternativa.
- Le custom property CSS (`--color-dim-0`, `--color-ui-glow`, …) sono **generate
  all'avvio da `theme.js`** iniettando su `document.documentElement`. Non si
  scrivono mai a mano in `style.css`. Una fonte, due proiezioni (JS e CSS).
- Eccezione: valori **geometrici** del dominio (`R = 550`, `DIM_ANGLES`,
  fattori di rank-normalize) non sono tema — stanno in `constants.js`/`geometry.js`.

## 2. Stile nodi/archi solo in `node-style.js`

- [`js/node-style.js`](js/node-style.js) esporta `buildNodeSpec(word, variant, opts?)`
  e `buildEdgeSpec(edge, opts?)`.
- Varianti nodo supportate: `'normal'`, `'unknown'`, `'from-sentence'`,
  `'dim-label'`, `'highlighted'`, `'dimmed'`, `'drag-target'`.
- **Nessun altro file** costruisce oggetti vis con `color`/`font`/`shadow`/
  `borderWidth` inline. Se trovi `{ color: { background: '...' } }` fuori da
  `node-style.js`, è un bug — spostalo.
- Se serve un nuovo look (es. "pulsante", "faded"), aggiungi la variante in
  `node-style.js` e chiamala dal sito d'uso.

## 3. Un file, una responsabilità

- `app.js` è **solo** bootstrap. Hard limit: ≤ 150 righe.
- Wiring (bottoni, tastiera, sidebar, transmit, keyboard shortcuts) vive in
  `js/wiring/<nome>.js`.
- Un file che supera ~400 righe è un segnale: probabilmente ospita più
  responsabilità. Dividi prima di aggiungere altro.
- Un file = un concetto esprimibile in 5 parole ("stile di nodi e archi",
  "persistenza localStorage dei campi", "filtri del campo vasto").

## 4. State management

- [`js/manager.js`](js/manager.js) è la **sola** sorgente di verità per
  `FIELDS = { vasto, nuovo, medio }` e `activeId`.
- Nessun modulo tiene stato globale in variabili a modulo-livello. Eccezioni
  ammesse: cache locali immutabili (es. `nbrCache` in `app.js`) — documentarle.
- UI transient cross-modulo (hover globale, drag-in-progress, modale aperto) va
  in un eventuale `js/ui-state.js`. UI transient interno a un modulo resta lì.
- Aggiungere stato senza decidere la **persistenza** è vietato. Ogni campo
  nuovo di uno stato risponde a: "localStorage? session? niente?". Se
  localStorage, la chiave sta in `constants.js` sotto `LS`.

## 5. Naming

- **Inglese per tecnica**: `node`, `edge`, `field`, `color`, `size`, `position`,
  `selected`.
- **Italiano per dominio**: `parola`, `firma`, `dimensione`, `frase`, `campo`,
  `rosa` (neighbors di una parola).
- Mai mischiare nello stesso identificatore. `wordColor` ok; `parolaColor` no;
  `coloreParola` ok se contesto è puramente di dominio.
- Prefisso `_` **solo** per campi veramente privati (non serializzati in
  `toJSON`, non letti da altri moduli). I campi persistiti non hanno `_`.
- I campi dei `word` object (signature, posizione, flag) usano forma piena:
  `sig`, `position.{x,y}`, `flags.{userCreated, unknown, fromSentence, transmitted}`.
  I campi calcolati (`color`, `size`, `angle`, `magnitude`) NON si memorizzano
  sul word — si derivano al volo da `theme` + `sig`.

## 6. Dipendenze tra moduli

Grafo target delle import:

```
app.js                  → wiring/*, graph, manager, editor, sidebar, filters, sentence
wiring/*                → manager, editor, sidebar, graph
graph.js                → manager, node-style, theme
manager.js              → field, constants
field.js                → theme, node-style, geometry, constants
node-style.js           → theme, constants
editor.js, sidebar.js   → manager (read-only), theme
filters.js              → manager (read-only), theme
sentence.js             → manager, field, theme
```

- **Nessuna dipendenza circolare.** Gli handler sono iniettati via
  `setHandlers({...})`, non importati.
- I moduli di rendering/UI **non fanno `fetch`.** `app.js` (bootstrap) fa i
  fetch e passa i dati giù. Unica eccezione: `editor.js` può chiamare API
  specifiche di editing (es. firma di una parola nuova) perché è il suo scopo.

## 7. Grafo: vis-network resta

- Il campo vasto carica ~27.000 nodi dal KG. `vis-network` su canvas è la scelta
  corretta per questa scala.
- Non introdurre ReactFlow, D3-force, Cytoscape, Pixi, SVG-based libs senza
  un benchmark misurato che mostri che servono. "I nodi non si stilizzano in
  CSS" non è un motivo — lo stile nodi passa per `node-style.js` (vedi §2).
- Il canvas ignora il CSS: accettato. Il CSS serve a sidebar, modali, pannelli,
  breadcrumb, overlay SVG di drag-to-connect. Non serve al grafo.

## 8. Solo relazioni uscenti dalle parole della frase

Ogni parola del campo nuovo mostra le sue **relazioni uscenti**, non quelle
entranti. In particolare nessuna parola-frase deve raccogliere vicini
bidirezionali: un arco `X IS_A io` farebbe apparire `X` come vicino di `io`
anche se la freccia è l'opposto. Questa regola è già stata violata e
ripristinata più volte — ogni regressione produce parole hub come "io" con
cataste di vicini bidirezionali.

La regola NON significa "tieni solo gli archi della frase": gli archi fra
satelliti (target del KG che hanno relazioni reciproche) vanno tenuti, sono
parte del campo che il KG descrive.

Punti dove la regola si applica:

- [`sentence.js`](js/sentence.js) `commitExpansion`: `keepEdges` esclude
  solo gli archi `non-frase → frase` (entranti puri verso parole-frase).
  Tiene `frase→satellite`, `satellite→satellite`, `frase→frase`.
- [`sentence.js`](js/sentence.js) `applyInterpretation`: l'opzione
  `includeIncoming` resta `false` per default. Mai impostarla a `true` nel
  flusso "frase → personale".
- [`relations-extract.js`](js/relations-extract.js) `fetchOutgoingForWord`:
  filtra `n.rel.startsWith('←')` perché `/api/biennale/word` restituisce
  entranti prefissate con `←`.

Il server è bidirezionale per scelta (vedi `build_biennale_word`,
`build_medio_data_for_sentence`). Il filtro è responsabilità del client.

## 9. Checklist pre-modifica

Prima di aggiungere/modificare qualcosa:

1. Il colore/token che mi serve esiste in `theme.js`? → se no, aggiungilo lì.
2. Lo stile nodo/arco rientra in una variante di `node-style.js`? → se no,
   aggiungi la variante lì.
3. Lo stato nuovo è per-field o globale? → metti il campo al posto giusto
   (`Field` o `manager`/`ui-state`).
4. Se persisti: chiave in `constants.js:LS`.
5. `app.js` cresce sopra 150 righe? → estrai in `wiring/`.
6. Sto scrivendo un `#` o un `rgba(…)` fuori da `theme.js`? → **stop**, torna a (1).

Se rispondi "lo sistemo dopo" a una qualunque: fermati e sistema adesso.

## 10. Come modificare queste regole

Questo file è il contratto. Se una sessione ha un motivo valido per violare una
regola, **o** aggiorna la regola (con il motivo) **o** sistema il codice che la
viola. Mai lasciare la dissonanza implicita — è lì che il caos torna a crescere.

Le regole non sono aspirazionali. Sono lo stato del codice che deve essere vero
a fine di ogni modifica.
