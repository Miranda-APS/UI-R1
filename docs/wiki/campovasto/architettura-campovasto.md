# Architettura campovasto

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md campovasto, FRONTEND.md)
> Raw: [CLAUDE_campovasto](../../raw/frontend/CLAUDE_campovasto.md); [FRONTEND.md](../../raw/frontend/FRONTEND.md)

## Overview

**Campovasto** è il frontend di esplorazione di UI-R1: una single-page application modulare (HTML/CSS/JS ES modules, niente bundler) che visualizza il [KG semantico](../topologia/knowledge-graph-semantico.md) come grafo navigabile. Due modi di lavoro: **campo vasto** (vista globale del KG, ~3000 nodi) e **campo nuovo** (mappa mentale personale costruita da una frase, trasmissibile al vasto). Servito dal backend Rust su `/campovasto/` con cache `no-cache, must-revalidate` (gli ES module imports non vedono `?v=N`).

## Topologia file

```
campovasto/
├─ index.html        # entry point, 2 tab visibili: vasto + nuovo
├─ app.js            # bootstrap (~7KB)
├─ style.css         # 62KB, theme centralizzato
├─ community.html    # vista alternativa community
├─ js/
│  ├─ manager.js     # registro FIELDS (vasto / nuovo)
│  ├─ graph.js       # vis-network bindings
│  ├─ field.js       # field state machine (27KB)
│  ├─ editor.js      # editing nodi/archi (36KB)
│  ├─ sentence.js    # creazione campo nuovo da frase (23KB)
│  ├─ sidebar.js     # pannelli laterali
│  ├─ filters.js, geometry.js, history.js
│  ├─ theme.js       # variabili CSS centralizzate
│  ├─ node-style.js  # styling nodi (isolato dal theme)
│  ├─ rel-legend.js  # legenda dei tipi di relazione
│  ├─ relations-extract.js  # estrazione archi da nodi nudi
│  ├─ ui-state.js    # localStorage state
│  ├─ constants.js
│  ├─ components/
│  │   ├─ confirm-panel.js
│  │   ├─ ctx-menu.js
│  │   ├─ dim-overlay.js
│  │   ├─ dim-editor.js
│  │   ├─ edit-panel.js
│  │   ├─ expand-animation.js  # 24KB — animazioni di espansione
│  │   ├─ exploration-trail.js # traccia esplorazione
│  │   ├─ extract-dialog.js
│  │   ├─ graph-toolbar.js
│  │   └─ overlay.js
│  ├─ layouts/
│  │   └─ rectangular.js       # 13KB — flow grid no-overlap
│  ├─ policies/
│  │   └─ word.js              # politiche di parola (filtri, accent normalize)
│  └─ wiring/
│      ├─ buttons.js, keyboard.js, modals.js
│      ├─ selection.js, sentence-panel.js, sidebar-layout.js
│      ├─ transmit.js          # trasmissione campo nuovo → vasto
│      └─ view-switcher.js
├─ fonts/ (JetBrains Mono 400/500/600/700)
├─ icons/ (switch-on/off SVG)
└─ vendor/vis-network.min.js
```

## Backend endpoints consumati

Vedi [medio API](medio-api.md) per il dettaglio di `/api/medio`. Lista completa degli endpoint chiamati:

| Endpoint | Method | Uso |
|----------|--------|-----|
| `/api/biennale/field` | GET | popola il campo vasto (~3000 nodi) |
| `/api/biennale/word` | GET | dettagli singola parola |
| `/api/medio?sentence=...` | GET | crea campo nuovo da frase |
| `/api/community/teach` | POST | insegna parola al sistema |
| `/api/community/connect` | POST | aggiunge arco al KG |
| `/api/community/validate` | POST | valida/aggiusta confidence |
| `/api/community/transmit_batch` | POST | bulk teach + edges (campo nuovo → vasto) |
| `/api/community/session` | GET | stato sessione |
| `/api/community/field` | GET | campo sessione |
| `/api/community/reset` | POST | reset sessione |
| `/api/word_connect` | POST | connetti due parole |
| `/api/state` | GET | stato corrente entità |
| `/api/kg/confirm_edge` | POST | conferma proposta arco |
| `/api/kg/reject_edge` | POST | rifiuta proposta arco |
| `/api/saved_fields[/save\|load\|delete]` | mixed | persistenza campi |
| `/ws` | WS | broadcast real-time (teach/connect events) |

## Convenzioni di codice (CLAUDE.md campovasto)

Vedi [design system](design-system.md) per i dettagli completi. Punti chiave:

- **Theme centralizzato**: variabili in `theme.js`, mai colori hardcoded inline.
- **Node-style isolato**: lo styling dei nodi vis-network vive in `node-style.js`, NON nel theme generale.
- **app.js ≤ 150 righe**: solo bootstrap. La logica vive nei moduli ES.
- **Naming**: kebab-case per file, camelCase per identifier.
- **Font**: JetBrains Mono ovunque. Mai Courier New, -apple-system, IBM Plex Mono.

## State machine FIELDS

```
FIELDS = {
  vasto: { data: …, layout: …, ui_state: … },
  nuovo: { data: …, layout: …, ui_state: …, sentence: ... },
}
```

Switch via `setActive(id)` in `manager.js`. View-switcher disabilita/abilita pannelli specifici per modo.

## Trasmissione "campo nuovo → vasto"

Workflow:
1. Utente apre tab "nuovo", inserisce frase
2. `sentence.js` chiama `/api/medio?sentence=...` → riceve lemmi + archi
3. Costruisce il campo nuovo come grafo personale
4. Utente edita (aggiungi nodi/archi, modifica relazioni)
5. Click "Trasmetti" → `wiring/transmit.js` invia POST a `/api/community/transmit_batch`
6. Backend salva atomicamente (`cura_save_kg`) e invalida la cache biennale
7. Al refresh del vasto, le nuove parole sono visibili

## Cache strategy

Server: `Cache-Control: no-cache, must-revalidate` per `/campovasto/*`. Motivo: gli ES module imports non vedono `?v=N` sull'HTML, quindi senza no-cache, gli aggiornamenti incrementali ai moduli non arrivano al browser. Trade-off: 1 roundtrip in più al boot, sempre allineato.

## See Also

- [Design system](design-system.md) — theme, naming, regole UI
- [Medio API](medio-api.md) — l'endpoint che crea il campo nuovo
- [LLM Wiki pattern applicato](llm-wiki-pattern-applicato.md) — collegamento al pattern Karpathy
- [Knowledge graph semantico](../topologia/knowledge-graph-semantico.md) — il KG mostrato
