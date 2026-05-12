# Design system — regole di stile e codice campovasto

> Sources: Francesco Mancuso, 2026-05-12 (campovasto/CLAUDE.md, regole di design.md, FRONTEND.md)
> Raw: [CLAUDE_campovasto](../../raw/frontend/CLAUDE_campovasto.md); [regole_di_design](../../raw/frontend/regole_di_design.md); [FRONTEND.md](../../raw/frontend/FRONTEND.md)

## Overview

Il design system di campovasto è codificato in 3 documenti operativi (`CLAUDE.md`, `regole di design.md`, `FRONTEND.md`) che sono **regole non consigli**. Trattano: theme centralizzato, isolamento del node-style, limiti dimensionali dei moduli, naming, font, color management, persistenza state. Questo articolo è la sintesi normativa.

## Le 10 regole inviolabili (campovasto/CLAUDE.md)

1. **Theme centralizzato**: tutte le variabili colore/spacing in `js/theme.js` come CSS custom properties. Mai colori hardcoded in CSS o JS.
2. **Node-style isolato**: lo styling dei nodi vis-network vive ESCLUSIVAMENTE in `js/node-style.js`. Mai mescolato con il theme generale (i nodi sono il dominio della libreria, non del theme).
3. **app.js ≤ 150 righe**: solo bootstrap + wiring. Tutta la logica vive nei moduli ES.
4. **Naming**: kebab-case per file (`expand-animation.js`), camelCase per identifier (`expandAnimation`).
5. **Font**: JetBrains Mono ovunque (400/500/600/700, italic 400). Mai Courier New, -apple-system, IBM Plex Mono, system-ui generico.
6. **Persistenza state**: in `js/ui-state.js` via localStorage. Mai sessionStorage. Mai cookies.
7. **Moduli ES2022 nativi**: niente bundler. `<script type="module">` con import relativi.
8. **No framework UI** (no React/Vue/Svelte). Vanilla JS + vis-network per il grafo.
9. **Animazioni dichiarative**: CSS keyframes preferiti, JS solo per orchestrazione di sequenze (vedi `expand-animation.js`).
10. **Refactor as you go**: se un modulo supera 800 righe, splitarlo in `components/` o `wiring/`.

## Theme (theme.js)

Esempi di variabili:
```js
export function applyTheme(root) {
  set('--spazio-medio', '8px');
  set('--colore-sfondo', '#0f0f1a');
  set('--colore-testo', '#e8e8f5');
  set('--colore-accent-isa', '#7e57c2');
  set('--colore-accent-causes', '#ef5350');
  // ...
}
```

`set('--var', value)` scrive in `:root` come custom property, ereditata da tutto il documento.

## Node-style (node-style.js)

Funziona in modo parallelo al theme ma per vis-network:
```js
export function styleNode(node, ctx) {
  // colore in base al frattale dominante,
  // size proporzionale a stability,
  // border per stato (selected, focused),
  // …
}
```

**Da non mescolare** con il theme: i nodi sono entità vis-network, non DOM. Le variabili CSS non li raggiungono.

## Tipografia

JetBrains Mono — un'unica famiglia, 4 pesi + italico, embedded come woff2 in `fonts/`. Linee guida da `regole di design.md`:

- **Display headlines**: 700, tracking -1%
- **Body**: 400, tracking 0
- **Caption / label**: 500
- **Inline code / variabili**: 400, sempre `font-feature-settings: "calt" on` per legature
- **Mai mix**: il monospace deve essere coerente, anche per testo "human".

Motivazione: l'identità visiva di UI-R1 è **uniforme e severa**, contro l'estetica AI di pulizia troppo levigata.

## Colori

Sistema base a 3 layer:
- **Sfondo**: scuro/blu profondo (`#0f0f1a`)
- **Testo**: chiaro/leggermente sfumato (`#e8e8f5`, mai puro #fff)
- **Accenti**: 8 tonalità per i tipi di relazione (IsA viola, Causes rosso, OppositeOf magenta, SimilarTo verde acqua, …) mappate ai trigrammi I Ching

Le mappature precise sono in `theme.js`. Vedi anche `rel-legend.js` per la legenda visibile.

## Spacing scale

```
--spazio-mini: 4px
--spazio-medio: 8px
--spazio-grande: 16px
--spazio-extra: 32px
--spazio-vasto: 64px
```

Niente valori arbitrari in CSS — sempre via variabile.

## Lo strumento libera

Quote da `regole di design.md`:
> "UI-R1 non deve essere un'esperienza coinvolgente. Deve essere uno strumento che si fa attraversare. Niente notifiche, niente badge counter, niente streak, niente engagement metrics."

Questa è la versione frontend del [principio 4](../principi/principi-inviolabili.md) (lo strumento deve liberare, non creare bisogno).

## Persistenza state

`ui-state.js` espone:
- `getState(key, default)` / `setState(key, value)` — wrapper localStorage
- Una chiave per modulo (es. `ui-state.field.layout-positions`, `ui-state.sidebar.collapsed`)
- Mai dati sensibili (token, credenziali) — solo preferenze UI

## Dipendency graph (FRONTEND.md)

Schema delle import:
```
app.js
 ├─ manager.js
 ├─ theme.js
 ├─ node-style.js
 ├─ graph.js
 │   ├─ vendor/vis-network.min.js
 │   └─ node-style.js
 ├─ field.js
 │   ├─ manager.js, graph.js, layouts/rectangular.js
 ├─ sentence.js
 │   ├─ manager.js, relations-extract.js
 ├─ wiring/* (lifecycle attach)
 └─ components/* (lazy load on demand)
```

Cicli sono vietati. Se un componente serve a più moduli → spostarlo in `components/`.

## Aggiungere un nuovo pannello

Procedura:
1. Crea `js/components/<nome>.js` esportando `initPanel(root, ctx)`
2. Aggiungi `<div id="panel-<nome>">` in `index.html`
3. Registra il wiring in `js/wiring/<nome>.js`
4. Importa il wiring da `app.js`
5. Verifica naming kebab-case + camelCase identifier
6. Refresh — no rebuild

## See Also

- [Architettura campovasto](architettura-campovasto.md) — topologia file
- [LLM Wiki pattern applicato](llm-wiki-pattern-applicato.md) — questa wiki segue Karpathy
- [Principi inviolabili](../principi/principi-inviolabili.md)
