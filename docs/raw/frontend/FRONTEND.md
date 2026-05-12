# Campovasto — Guida frontend

Tutto il frontend del visualizzatore Prometeo (porta 3000 di
`194.116.73.38`, oppure `localhost:8080/campovasto/` in locale) vive in
**questa cartella `campovasto/`**.

Modificare HTML / CSS / JS qui e ricaricare il browser è sufficiente: il
backend Rust serve la cartella a runtime (via `ServeDir`), quindi **non
serve ricompilare** per cambi puramente di interfaccia.

---

## Struttura

```
campovasto/
├── index.html            ← pagina principale (un solo file)
├── community.html        ← variante "campo community"
├── style.css             ← unico foglio di stile
├── app.js                ← bootstrap (≤ 150 righe — solo init)
├── vendor/
│   └── vis-network.min.js   ← motore di rendering del grafo (canvas)
├── fonts/                ← JetBrains Mono (web-font self-hosted)
└── js/                   ← codice modulare (ES modules)
    ├── theme.js          ← FONTE UNICA di colori e token (vedi §1)
    ├── node-style.js     ← FONTE UNICA degli stili nodo/arco (vedi §2)
    ├── constants.js      ← nomi dimensioni, chiavi localStorage, …
    ├── geometry.js       ← matematica posizioni (firma 8D → x,y)
    ├── manager.js        ← stato globale dei 3 campi (vasto/nuovo/medio)
    ├── field.js          ← classe Field (parole, archi, DataSet vis)
    ├── graph.js          ← interazione col canvas (hover, click, highlight)
    ├── editor.js         ← pannelli "modifica dimensioni/relazioni" + ctx menu
    ├── sidebar.js        ← sidebar destra (info parola, dim bars, radar)
    ├── filters.js        ← filtri del campo vasto
    ├── sentence.js       ← creazione campo medio + frase nel campo nuovo
    ├── relations-extract.js  ← estrazione on-demand dal KG
    ├── medio-animation.js    ← animazione di entrata della frase
    ├── ui-state.js       ← stato UI transient (filtri attivi, ecc.)
    ├── components/       ← widget riusabili (vedi §3)
    │   ├── dim-editor.js     ← editor 8 dimensioni (sidebar + panel)
    │   ├── edit-panel.js     ← struttura del pannello modale
    │   ├── ctx-menu.js       ← menu contestuali
    │   ├── extract-dialog.js ← dialog filtri estrazione
    │   └── overlay.js        ← canvas overlay (archi animati + label)
    ├── policies/
    │   └── word.js       ← regole "chi può, come, con quali flag"
    └── wiring/           ← cablaggi (bottoni, tastiera, transmit, ecc.)
```

---

## Avvio in locale

### Opzione A — backend in locale (consigliato per test completi)

Richiede Rust installato (rustup.rs).

```bash
cd progetto-prometeo/
cargo build --release --features web --bin prometeo-web
./target/release/prometeo-web 8080
# Apri http://localhost:8080/campovasto/
```

Modifichi un file in `campovasto/` → ricarichi la pagina → vedi il cambio.
Non serve ricompilare.

### Opzione B — usa il backend remoto

Se non vuoi installare Rust, lavori direttamente sul server di sviluppo
(porta 3001 di `194.116.73.38`):

```bash
# Modifichi in locale
# Carichi sul server di dev
scp -r campovasto/ miranda@194.116.73.38:~/prometeo_standalone_v3/
# Apri http://194.116.73.38:3001/campovasto/
```

⚠ Non tocchi mai `~/UI-r1/` su quel server — è la stable di produzione.

### Cache busting

Il browser cachea pesantemente JS e CSS. Quando carichi una nuova versione,
**bump in `index.html`** dei due `?v=NN` di `style.css` e `app.js`:

```html
<link rel="stylesheet" href="style.css?v=25">
<script src="app.js?v=25" type="module"></script>
```

Ad ogni rilascio incrementa `25 → 26`. Senza bump il browser potrebbe
ignorare le modifiche.

---

## Le quattro regole non negoziabili

Questi sono i contratti del progetto. Se una modifica li viola, *o la
modifica cambia, o la regola cambia* — mai tacitamente entrambe.
(Il file `CLAUDE.md` in questa cartella ha la versione completa.)

### §1. Tutti i colori in `js/theme.js`

Nessun valore esadecimale (`#ffffff`, `rgba(...)`, `hsl(...)`) deve apparire
fuori da `js/theme.js`. Mai. Né nei `.js` né nel `.css`.

`theme.js` esporta:
- `DIM_COLORS[0..7]` — gli 8 colori delle dimensioni I Ching
- `CD_COLORS[0..7]` — gli 8 colori dei drive Octalysis
- `UI` — colori semantici di interfaccia (`glow`, `textDim`, `linkNeutral`, …)
- `tokens` — dimensioni, durate, z-index (`nodeSizeMin`, `anim.fast`, …)

Le **CSS custom properties** (`--color-dim-0`, `--color-ui-glow`, …) sono
generate al boot da `theme.js` iniettando su `document.documentElement`. Le
trovi in `style.css` solo come `var(--color-...)`, mai scritte a mano.

**Voglio cambiare un colore.**
1. Apri `js/theme.js`
2. Trova il token (es. `UI.glow` o `DIM_COLORS[3]`)
3. Cambia il valore lì
4. Tutto il resto si aggiorna automaticamente (JS e CSS)

**Voglio aggiungere un colore nuovo.**
1. Lo aggiungi a `theme.js` con un nome semantico (`UI.myThing`)
2. Lo usi via `import { UI } from './theme.js'` nel JS, oppure via
   `var(--color-ui-mything)` nel CSS (la custom property è generata).

### §2. Tutti gli stili nodo/arco in `js/node-style.js`

`node-style.js` esporta `buildNodeSpec(word, variant, opts?)` e
`buildEdgeSpec(edge, opts?)`. **Nessun altro file** costruisce oggetti vis
con `color`, `font`, `shadow`, `borderWidth` inline.

Varianti supportate (parametro `variant`):
- `'normal'` — stato di riposo
- `'unknown'` — parola sconosciuta (fondo grigio)
- `'from-sentence'` — parola creata da una frase (alone dorato)
- `'highlighted'` / `'active'` — selezionata
- `'rosa'` — vicini della selezionata
- `'dimmed'` — sbiadita (le altre quando una è selezionata)
- `'drag-target'` — feedback durante drag-to-connect

**Voglio cambiare l'aspetto di un nodo selezionato.**
- Apri `js/node-style.js`, blocco `if(variant === 'active')`. Modifica `size`,
  `borderWidth`, `font`, ecc. (i colori vengono dai token di `theme.js`).

**Voglio una nuova variante "pulsante".**
1. In `node-style.js`, aggiungi `if(variant === 'pulsing'){ return { ... }; }`
2. Dal sito d'uso (es. `graph.js`) chiamala:
   `F.nodesDS.update(buildNodeSpec(word, 'pulsing', {fieldId: F.id}))`

### §3. Un file = una responsabilità

- `app.js` è SOLO bootstrap. Limite ≤ 150 righe.
- Wiring (bottoni, tastiera, transmit, sidebar layout) → `js/wiring/<nome>.js`.
- Componenti riusabili → `js/components/<nome>.js`.
- Un file > ~400 righe = probabile mix di responsabilità → dividi.

### §4. State management

- `js/manager.js` è la **sola** sorgente di verità per i 3 campi:
  `FIELDS = { vasto, nuovo, medio }` e `activeId`.
- Stato persistente → chiave in `js/constants.js` sotto `LS`, salvato in
  `localStorage` (vasto è eccezione: ricaricato da API ad ogni boot, mai
  persistito lato client).
- Stato UI transient cross-modulo → `js/ui-state.js`.

---

## Concetti di dominio

Per leggere il codice senza perdersi, due nozioni:

**Firma 8D.** Ogni parola ha una `sig` di 8 numeri in `[0,1]`. Le 8
dimensioni sono I Ching canoniche: Agency, Permanenza, Intensità, Tempo,
Confine, Complessità, Definizione, Valenza. La **posizione** del puntino sul
canvas è derivata dalla firma via `geometry.js::sigToXY` + `placeByRank`.
Cambiando la firma di una parola, il puntino si sposta. Cambiando i token
colore in `theme.js`, il colore di tutte le parole si aggiorna.

**Tre campi:**
- **vasto** — il KG globale (~27.000 parole). Read-mostly, voce stabile dal
  server. Le modifiche locali sono effimere (non persistono).
- **nuovo** — sandbox personale dell'utente (parole proprie, frasi proprie).
  Persistito in `localStorage`. Si trasmette al vasto con `↗ trasmetti`.
- **medio** — campo derivato da una singola frase (entra animata). Anche
  questo persistito in `localStorage`.

**Rosa.** I "vicini" di una parola selezionata: i nodi con cui ha un arco.
Quando clicchi una parola, viene calcolata la sua `rosa` (`Field.getRosa`)
e applicata via `graph.js::applyHighlight`.

---

## Compiti comuni — dove mettere le mani

| Voglio… | File principale |
|---|---|
| Cambiare un colore | `js/theme.js` |
| Cambiare l'aspetto di un nodo (taglia, bordo, ombra) | `js/node-style.js` |
| Modificare l'aspetto di un arco | `js/node-style.js` (`buildEdgeSpec`) |
| Cambiare il layout della sidebar | `index.html` + `style.css` |
| Cambiare quale info appare quando clicchi una parola | `js/sidebar.js::showInfo` |
| Cambiare l'animazione degli archi (line-dash) | `js/components/overlay.js` |
| Cambiare le label dinamiche dei vicini | `js/components/overlay.js::drawLabels` |
| Aggiungere una voce al menu contestuale (right-click) | `js/editor.js::openNodeCtxMenu` |
| Cambiare il pannello "modifica dimensioni" | `js/components/dim-editor.js` |
| Cambiare il bottone "trasmetti" | `js/wiring/transmit.js` |
| Cambiare il messaggio "campo vuoto — premi…" | `js/wiring/view-switcher.js::updateEmptyHint` |
| Cambiare il radar Octalysis | `js/editor.js::drawRadar` |
| Cambiare i filtri del campo vasto | `js/filters.js` |

---

## Workflow consigliato

1. Lavori in locale con backend (Opzione A) o con backend remoto (Opzione B).
2. Modifichi i file. Niente build, ricarichi la pagina.
3. Prima di "rilasciare": bump della versione cache in `index.html`.
4. Sincronizzi sul server. Per la **stable** (porta 3000) chiedi a Francesco
   o segui `IA di quartiere/REDEPLOY.md`. Per la **dev** (porta 3001) basta
   `scp -r campovasto/ miranda@194.116.73.38:~/prometeo_standalone_v3/`.

---

## Tre cose che spesso fanno perdere tempo

1. **`vis-network` ignora il CSS.** Il canvas è un canvas, non DOM. Non puoi
   stilare i nodi con `:hover` o `.classlist`. Tutto lo stile passa da
   `node-style.js`. Il CSS serve a sidebar, pannelli, modali, breadcrumb,
   overlay SVG di drag-to-connect — non al grafo.

2. **Cache aggressiva del browser.** Se modifichi un file e non vedi
   cambiare niente: bump del `?v=NN` in `index.html`, oppure Ctrl+Shift+R
   (force refresh).

3. **`vasto` ha ~27.000 nodi.** Operazioni O(n²) nel ciclo di rendering = la
   pagina si pianta. Se aggiungi feature, controlla `js/field.js::bulkLoad`
   come template per operazioni batch (una sola `nodesDS.add(array)` invece
   di 27K `add(node)`).
