# REGOLE DI DESIGN — campovasto/

Contratto formale per il design visivo dei tre campi (vasto, nuovo, medio).
Si applica a ogni modifica all'interfaccia, senza eccezioni. Stesso peso di
`CLAUDE.md` — leggilo prima di toccare CSS, HTML o gli spec di vis-network
costruiti in `js/node-style.js`.

> **Scope**: solo `campovasto/` (vasto + nuovo + medio).
> Vale per: chrome HTML/CSS della pagina **e** spec JS che costruiscono
> nodi/archi del grafo. Logica applicativa (data, persistence, network)
> è fuori scope.

---

## §0 — Pensiero per oggetti, non per atomi

Ogni elemento dell'interfaccia è un **oggetto** con identità. Ogni oggetto
ha un kit di base (forma, sfondo, bordo, font, spaziatura). Le varianti e
gli stati lo modificano **per delta** — non lo riscrivono.

**Regola del delta**: una classe variante o di stato dichiara solo le
proprietà che:
- **modificano** una caratteristica già impostata sopra (oggetto base o
  ereditata)
- **introducono** una caratteristica non ancora impostata nella catena

Se la variante non cambia nulla rispetto al kit base + ereditarietà, il
blocco CSS è **vuoto** (e di solito non si scrive).

```css
button           { padding: 6px; border: 1px solid var(--bordo); background: var(--superficie); }
button.undo      { /* vuoto: undo non si distingue dal bott base */ }
button.attivo    { background: var(--superficie-alt); /* solo il delta */ }
button.bloccato  { opacity: 0.32; pointer-events: none; }
```

**Quando il tag HTML identifica già l'oggetto, è il tag il selettore — niente
classe di oggetto.** La classe d'oggetto si usa solo quando il tag DOM è
generico (`<div>`, `<span>`).

| Oggetto                              | Identità DOM                  | Selettore CSS                            |
|--------------------------------------|--------------------------------|------------------------------------------|
| `bott`                               | `<button>`                     | `button`                                 |
| `input`                              | `<input>`, `<textarea>`        | `input`, `textarea`                      |
| `box`, `etichetta`, `segnale`, `tip` | `<div>` / `<span>` generico    | `.box`, `.etichetta`, `.segnale`, `.tip` |

L'HTML compone classi atomiche per varianti e stati:

- **eventuale classe d'oggetto** (solo se tag generico) = oggetto
- **classi successive** = variante e/o stato
- **spazio** = composizione; mai trattini per concatenare ruoli diversi

Esempio HTML:

```html
<button class="undo">↶</button>                  <!-- button è il bott -->
<button class="undo bloccato">↶</button>
<div class="box modale attivo">…</div>           <!-- div generico → classe box -->
<input class="ricerca">                          <!-- input è il input -->
```

Ogni nuovo bordo, sfondo, font, ombra → **nuova classe**. Mai valori inline.

---

## §1 — Tema: Origine

C'è un solo tema attivo: **Origine** (scuro + freddo). I valori vivono in
`js/theme.js` ed escono come CSS custom property dal root via
`applyThemeToCss()` (vedi `CLAUDE.md §1`).

L'architettura è pronta per altri temi: aggiungerne uno = nuova tabella in
`theme.js` + selettore al root. Nessuna regola CSS deve mai consumare un
colore/spaziatura/font letterale: sempre `var(--…)`.

```css
/* sì */
button { background: var(--superficie); color: var(--testo-chiaro); }

/* no — letterale */
button { background: #1a1d2e; color: #e0e0e0; }
```

I numeri non-tema (raggi geometrici del campo, fattori di rank-normalize,
angoli I Ching) non sono temi — restano in `constants.js` / `geometry.js`.

---

## §2 — Oggetti

Vocabolario chiuso. Ogni elemento visivo deve essere riconducibile a uno
di questi.

| Oggetto      | Cosa è                                  | Identità                                |
|--------------|------------------------------------------|------------------------------------------|
| `nodo`       | pallino + etichetta-parola               | spec JS in `node-style.js`               |
| `arco`       | linea fra due nodi (+ verso, + tipo)     | spec JS in `node-style.js`               |
| `bott`       | bottone/icona di azione                  | tag `<button>`                           |
| `input`      | campo testuale o numerico                | tag `<input>`, `<textarea>`              |
| `sezione`    | segmento verticale di contenuto correlato (testo, controlli, dati raggruppati) | tag `<section>` |
| `cursore`    | controllo lineare per valore singolo o intervallo | classe HTML (vedi §3 per varianti) |
| `box`        | qualsiasi contenitore                    | classe HTML (su `<div>`)                 |
| `etichetta`  | testo libero che identifica qualcosa     | classe HTML (su `<span>` o `<div>`)      |
| `segnale`    | badge / indicatore di stato              | classe HTML                              |
| `tip`        | tooltip / suggerimento                   | classe HTML                              |

Aggiungere un oggetto nuovo = nuova riga in questa tabella, **prima** del
codice. Se un elemento visivo non rientra qui, prima si estende il
vocabolario, poi si scrive.

---

## §3 — Varianti

Una variante è una specializzazione di un oggetto. Si esprime come **classe
atomica**, italiano corto, senza prefisso. Vive solo in combinazione con
un oggetto.

```html
<button class="undo">↶</button>           <!-- variante undo del button -->
<div class="box modale">…</div>           <!-- variante modale del box -->
<div class="box barra">…</div>            <!-- variante barra del box -->
```

Selettori CSS sempre **specifici per oggetto**: `button.undo`, `.box.modale`.
Mai `.undo {…}` o `.modale {…}` da soli — una variante non ha senso fuori
dal suo oggetto.

Vale la **regola del delta** (§0): la variante dichiara solo le proprietà
che modificano il kit base o ne aggiungono di nuove.

Varianti già definite (lista non esaustiva — si estende qui prima del
codice):

| Variante  | Oggetto    | Cosa indica                                            |
|-----------|------------|---------------------------------------------------------|
| `undo`    | `button`   | bottone "annulla"                                       |
| `redo`    | `button`   | bottone "ripeti"                                        |
| `switch`  | `button`   | toggle di visibilità — pattern unificato per qualsiasi UI dove l'utente accende/spegne la presenza di qualcosa (es. una dimensione attiva nel filtro). Icona switch (SVG in `/icons/switch-off.svg` e `/icons/switch-on.svg`) applicata via CSS mask: pallino a sinistra = inattivo, a destra = attivo. Il colore (background-color sotto la mask) arriva dal tema — segue `--dim-color` quando attivo, `--testo-basso` a riposo |
| `modale`  | `box`      | finestra sovrapposta bloccante                          |
| `barra`   | `box`      | striscia di comandi raggruppati (toolbar, breadcrumb)   |
| `parola`  | `box`      | card della parola selezionata in sidebar (titolo + dim-bars + radar + azioni) |
| `campo`   | `segnale`  | etichetta che indica da quale campo arriva qualcosa (`dal vasto`, `dal nuovo`, `dal medio`) |
| `singolo` | `cursore`  | un valore [0..100] (es. dim-bars sidebar)               |
| `range`   | `cursore`  | due valori [lo..hi] (es. filtro firma del campo vasto)  |

> **Classi strutturali interne** (non in §2): `intesta`, `corpo`, `sezione`,
> `azioni`, `griglia`, `riga` sono parti di un `box`/`pannello`/`parola`.
> Non sono oggetti del vocabolario — sono il "tessuto" interno di un box.
> Si applicano direttamente come classe (`<div class="azioni">`) e si
> consumano in CSS senza qualificatore di oggetto. Non si elencano qui.

### Anatomia del `cursore`

Pattern visivo unificato per tutta l'app: **un solo modo di rappresentare
controlli lineari**, varia solo dove parte il riempimento.

```html
<!-- valore singolo -->
<div class="cursore singolo">
  <span class="polo sinistra">…polo low…</span>
  <span class="polo destra">…polo high…</span>
  <div class="riempi"></div>
</div>

<!-- intervallo -->
<div class="cursore range">
  <span class="polo sinistra">…polo low…</span>
  <span class="polo destra">…polo high…</span>
  <div class="riempi"></div>
  <input type="range" data-kind="lo">
  <input type="range" data-kind="hi">
</div>
```

| Sub-elemento         | Ruolo                                                                  |
|----------------------|------------------------------------------------------------------------|
| `.cursore`           | container = track scuro a tutta larghezza, h:18px, cursor `ew-resize`  |
| `.cursore .riempi`   | fill colorato — full height; per `singolo` parte da sinistra, per `range` parte da `lo%` |
| `.cursore .polo`     | etichetta polo sovrapposta, pointer-events:none (`.sinistra` / `.destra`) |
| `.cursore.range input[type="range"]` | sliders nativi sopra il container, thumb sottile per il drag agli estremi |

Ogni nuovo controllo lineare nell'app deve usare `cursore` con una di
queste due varianti. Aggiungere una terza variante richiede motivazione
qui prima del codice.

---

## §4 — Stati

Vocabolario chiuso, applicato a qualsiasi oggetto.

| Stato       | Quando                          | Applicato come                                      |
|-------------|----------------------------------|------------------------------------------------------|
| *(default)* | nessuna classe di stato         | implicito                                            |
| `hover`     | sorvolato col cursore           | `:hover` per chrome HTML; classe `.hover` per spec JS|
| `attivo`    | selezionato / in uso            | classe                                               |
| `inattivo`  | non in uso / attenuato (dimmed) | classe                                               |
| `bloccato`  | disabled / non interagibile     | classe                                               |
| `ignoto`    | parola senza firma (solo `nodo`)| classe / flag                                        |

Estensioni ammesse solo se servono: `carica`, `errato`. Aggiunte qui prima
del codice.

**Doppio livello di scrittura** (segue il cascade CSS naturale):

1. **Globale** — regola di stato senza qualificatore di oggetto. Fissa il
   comportamento di default valido in tutta l'app.
   ```css
   .attivo   { opacity: 1; }
   .inattivo { opacity: 0.5; }
   .bloccato { opacity: 0.32; pointer-events: none; }
   ```
2. **Contestuale** — la stessa classe combinata con oggetto/variante/area
   sovrascrive il default. Vale la regola del delta (§0): si dichiara solo
   ciò che differisce.
   ```css
   .barra button.inattivo  { opacity: 0.7; }   /* nei comandi di una barra il dimmed è meno marcato */
   .box.modale.attivo      { /* delta specifico del modale aperto */ }
   button.attivo           { background: var(--superficie-alt); }
   ```

Lo stato non ha aspetto proprio assoluto: ha un default globale **e** un
comportamento relativo al contesto in cui appare.

---

## §5 — Aree

Le aree sono i grandi contenitori della pagina. Hanno **ID univoco**,
nessuna classe. Sono il punto di ancoraggio delle catene DOM brevi.

Aree esistenti (legacy in inglese — preservate per non innescare refactor
diffusi in HTML/CSS/JS):

| ID legacy             | Cosa contiene                                       |
|-----------------------|------------------------------------------------------|
| `#graph`              | canvas del grafo + overlay (toolbar, hint, animBar) |
| `#sidebar`            | sezioni della sidebar                               |
| `#editPanel`          | pannello di editing                                 |
| `#sentencePanel`      | pannello frase (medio)                              |
| `#understandingPanel` | pannello "UI-R1 legge" (medio)                      |

**Aree nuove**: ID in **italiano breve**. Aggiungere un'area = nuova riga
in una tabella sotto questa, mai una classe. La motivazione si scrive qui
prima del DOM.

---

## §6 — Catene DOM

**Massimo 3 livelli.** Preferenza al più corto possibile.

```css
/* sì */
#sidebar .etichetta              { … }
button.attivo                    { … }
#graph .barra button             { … }

/* no — troppo profondo */
.app .pane .body .item .etichetta { … }
```

L'ID di area ancora la catena. Dentro l'area, le classi degli oggetti (o i
tag, vedi §0) identificano da sole il target. Si scende di un livello solo
se la ragione è strutturale (es. una `.barra` dentro `#graph`).

---

## §7 — Eredità

Ogni `box` (e le sue varianti `modale`, `barra`) e ogni **area** definisce
le proprietà di contesto: font, spaziatura, tinta neutra. Gli oggetti figli
**ereditano** — non ridefiniscono.

```css
.box.modale            { font-size: 14px; padding: 16px; }
/* dentro .box.modale, le .etichetta sono 14px senza dichiararlo */

.box.modale .etichetta { /* solo se serve un override puntuale */ }
```

**Vale per tutti gli oggetti, anche `button` e `input`**: hanno un kit base
globale al selettore di tag (`button { … }`, `input { … }`) e regole
locali che lo sovrascrivono **via cascade solo quando il contesto lo
richiede**. Se la regola locale dichiarerebbe gli stessi valori del default,
non si scrive (regola del delta, §0).

```css
button                  { /* kit base: bordo, sfondo, font, padding */ }
.barra button           { /* delta: solo cosa cambia nella barra */ }
.box.modale button      { /* delta: solo cosa cambia in un modale */ }
```

---

## §8 — Prossimità

I comandi vivono nel **wrapper DOM dell'oggetto su cui agiscono**.

```html
<!-- sì: il comando edit è dentro la card della parola -->
<div class="box parola">
  <span class="etichetta">…</span>
  <button class="edit">✎</button>
</div>
```

Le `box.barra` globali contengono solo comandi che agiscono sull'**area
intera** (es. la toolbar undo/redo del grafo agisce sul field, non su una
parola specifica → vive in `#graph` come barra dell'area).

Se un comando finisce lontano dall'oggetto su cui agisce, è un bug
strutturale: si sposta il comando, non si aggiungono indicatori per
"spiegarlo".

**Salvaguardia — spostamenti radicali richiedono consenso esplicito**:
quando applicare la prossimità implica spostare un comando attraverso aree
diverse (es. da `#sidebar` a `#graph`, o da una `box.modale` al wrapper di
una parola), **prima di muovere chiedi l'ok all'utente**. Le
riorganizzazioni grandi non si fanno per inerzia di regola: si propongono
e si conferma.

---

## §9 — Naming

- **italiano breve**, parole singole quando possibile
- **camelCase** per acronimi/sigle (raro)
- **trattini** solo dentro la stessa classe atomica per legare due termini
- **spazio** in HTML separa classi atomiche distinte

| Esempio                          | Verdetto                        |
|----------------------------------|---------------------------------|
| `bott`, `box`, `attivo`, `modale`| ok (atomici)                    |
| `box modale attivo`              | ok (tre atomi composti)         |
| `edit-conferma`                  | ok (un atomo, due parole legate)|
| `box-modale-attivo`              | no (tre ruoli in una classe)    |
| `commandButton`, `boxAttivo`     | no (verboso, non italiano)      |

---

## §10 — Dimensioni responsive

Le dimensioni si esprimono in funzione di cosa rappresentano:

| Tipo dimensione | Unità | Perché |
|---|---|---|
| Testo o contenuto (larghezza label, padding di un input, gap tipografico) | `em`, `ch` | Scala col font corrente — coerente con browser zoom, accessibility, cambio di tema tipografico |
| Viewport / pagina intera (canvas wrapper, modali, sidebar full-height) | `dvh`, `dvw`, `dvmin` + `env(safe-area-inset-*)` | Stabile su WebView mobile (barra URL, gesture bar), compatibile con tastiera virtuale via `visualViewport` |
| Costanti visive atomiche (border, border-radius, gap di ritmo, ombre) | `px` | Pixel "fisici" del linguaggio visivo, indipendenti da font e viewport |

**Preferire `dvh` a `vh`** — `dvh` (dynamic viewport height) gestisce
correttamente la barra URL mobile. Per layout che devono "respirare" attorno
alla tastiera virtuale, abbinare a un listener su `window.visualViewport`
che aggiorna una CSS var (`--vv-h`) consumata dal CSS.

Niente `100%` su altezze che dipendono dal viewport — usare `dvh/dvw`.
Niente `vh` puro per layout full-screen.

**Esempi**:

```css
/* Layout legato al testo: em */
.riga.firma { grid-template-columns: 7em 1fr 5em; }

/* Pagina intera: dvh + safe area */
body {
  min-height: 100dvh;
  padding-top:    env(safe-area-inset-top);
  padding-bottom: env(safe-area-inset-bottom);
}

/* Costanti visive: px */
.box { border: 1px solid var(--bordo); border-radius: 8px; }
```

**Eccezione canvas vis-network**: il canvas del grafo è opaco al CSS — la
libreria gestisce internamente le sue dimensioni (autoResize, pan/zoom).
Questa regola si applica solo al **container esterno** (`#graph`) e agli
**overlay** (toolbar, hint, animBar). Niente `em`/`dvh` *dentro* gli spec
JS di nodi/archi: lì usiamo i numeri del dominio (raggi, angoli) come da §1.

**Migrazione**: la regola si applica al codice nuovo e al CSS che si tocca
per altre ragioni. Niente refactor in massa del CSS esistente — fino a
nuovo ordine.

---

## §11 — Bordi (caso pilota del sistema)

Tre classi base, ognuna consumata dal CSS via `var(--…)` di tema:

| Classe   | Peso              | Uso                                                |
|----------|-------------------|----------------------------------------------------|
| `b-soft` | 1px neutro tenue  | aree, input, divisori                              |
| `b-mid`  | 1px neutro pieno  | box, modali, card                                  |
| `b-hard` | 2px accento       | stati attivi, bordi che chiamano l'occhio          |

Ogni bordo nuovo eredita una di queste **+** classe specifica per
oggetto/variante (regola del delta, §0). Mai `border: 1px solid #fff`
inline. Mai una quarta classe base senza motivare l'aggiunta qui.

Lo stesso pattern (3 classi base + composizione per oggetto) è il
**modello** per le altre famiglie visive che verranno aggiunte (sfondi,
ombre, raggi). Vanno introdotte una alla volta in questo file, nello
stesso formato.

---

## §12 — HTML/CSS prima, JS solo dove serve

La struttura statica dell'interfaccia vive in `index.html`. Lo stile vive
in `style.css` (più i token di `theme.js`). Il JavaScript entra solo
quando serve **comportamento**: interazione, stato dinamico, contenuto
che si replica in N istanze.

**Regola operativa**:

- Se un elemento può essere scritto direttamente in HTML/CSS → in HTML/CSS
- Se richiede JS (contenuto dinamico, replica per dato, gestione di
  stato) → JS, ma **via injection collegata a eventi**, mai come
  `<script>` inline o handler `onclick="…"` nell'HTML
- Eventi DOM si attaccano via `addEventListener` da moduli JS, non come
  attributi HTML

Esempio:

```html
<!-- HTML: struttura statica -->
<div id="filterPanel" class="box pannello">
  <div class="intesta">
    <span class="etichetta titolo">filtri campo vasto</span>
    <button class="chiudi">×</button>
  </div>
  <div class="sezione">
    <div id="dimFilter" class="griglia"></div>   <!-- popolato da JS -->
  </div>
</div>
```

```js
// JS: iniezione dinamica + wiring eventi
const grid = document.getElementById('dimFilter');
for (const d of DIM_NAMES) grid.appendChild(buildChip(d));
document.querySelector('#filterPanel .chiudi').addEventListener('click', closeFilter);
```

**Niente JS hardcoded nell'HTML**: nessun `onclick`, `onload`,
`onsubmit`; nessun `<script>` con logica di pagina inline. Solo lo
`<script type="module" src="app.js">` di entry point è autorizzato.

---

## §13 — Come modificare queste regole

Questo file è il contratto. Se una sessione ha un motivo valido per
violare una regola, **o** aggiorna la regola (con motivo) **o** sistema il
codice che la viola. Mai lasciare la dissonanza implicita — è lì che il
caos torna a crescere.

Le regole non sono aspirazionali: sono lo stato del codice che deve
essere vero a fine di ogni modifica.

> *Nota di scope*: i nodi e gli archi del grafo vivono nel canvas di
> vis-network, non nel DOM. Le classi `nodo`/`arco` non hanno effetto CSS
> diretto: il loro kit passa per gli spec JS in `js/node-style.js`. Il
> **vocabolario** (oggetti, varianti, stati) resta lo stesso —
> uniformiamo il pensiero, non la lingua di esecuzione.