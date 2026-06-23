
> Per: agente di frontend/design. Da: Fable, su direttiva di Francesco Mancuso
> (2026-06-15). Contesto teorico: [gate_di_comprensione.md](gate_di_comprensione.md).
> **Confine netto**: questo piano riguarda SOLO la struttura/UI dell'admin e
> delle viste. Il *backend della comprensione* (`/api/comprehend`, copertura
> per-token, saturazione) lo sta facendo Fable in Rust — NON toccarlo, consumalo.

## Il problema

`/admin` (`src/web/index.html`, ~150KB) è un monolite a 9 tab (CHAT, CAMPO,
FRATTALI, VOLONTÀ, SOGNO, NARRATIVA, IDENTITÀ, EPISODI, DIALOGO) — "mare di cose
obsolete o rotte". In parallelo esistono 11 pagine HTML con forti duplicazioni
(biennale/index ≈ doppione di campovasto; due pagine community; tre strumenti di
cura). L'unico frontend solido è **`campovasto/`** (motore grafo vis-network,
`node-style` isolato, theme centralizzato, `view-switcher`, `sentence.js`).

## Il principio guida (di Francesco)

> Ciò che ruba spazio → **endpoint dedicato**. Ciò che è compatto → **in-page**
> nell'hub. Design language unico = **campovasto** (riusare theme + node-style,
> mai nuovi CSS monolitici).

## Architettura target

### `/admin` diventa un HUB / launcher, non un contenitore

L'admin tiene SOLO due cose:

1. **Cruscotto vitali compatto** (in-page, O(1), già esistono nell'header):
   `parole`, `simplessi`, `perturbazioni`, `energia` + il dot di connessione WS.
   Sono leggere e danno il polso del sistema a colpo d'occhio — restano.
2. **Griglia di card-launcher** verso i moduli (ognuno una vista dedicata).

Tutto il resto (le 9 tab) ESCE dall'admin e diventa o vista dedicata o viene
ritirato.

### Layout dell'hub (proposta)

```
┌─ UI-r1 · admin ───────────────────────────────  [vitali: parole · simplessi · energia · ●] ─┐
│                                                                                              │
│  ┌──────────────────────────┐  ┌───────────────────┐  ┌───────────────────┐                │
│  │  ★ GATE DI COMPRENSIONE   │  │  CAMPO / FRATTALI  │  │  STATO INTERNO    │                │
│  │  (card primaria, grande)  │  │  → campovasto      │  │  valenza·narrativa │                │
│  │  l'analisi di un input    │  │                    │  │  ·identità·episodi │                │
│  │  → /comprensione          │  └───────────────────┘  └───────────────────┘                │
│  └──────────────────────────┘                                                                │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐                        │
│  │  CURA KG          │  │  CHAT (sperim.)   │  │  COMUNITÀ          │                        │
│  └───────────────────┘  └───────────────────┘  └───────────────────┘                        │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

**DOVE VA IL GATE DI COMPRENSIONE**: è la **card primaria dell'hub** — in alto a
sinistra, visivamente dominante (è il cuore del progetto). NON è una tab dentro
l'admin: è una **pagina dedicata e indipendente** (`/comprensione`, costruita da
Fable). La card dell'admin ci **linka** (apre la pagina), NON la incolla dentro.
Il vecchio `comp-panel` dentro la tab CHAT va **rimosso** (superato dalla pagina
dedicata).

> **Chiarimento (l'admin non embedda nulla)**: tutte le card dell'hub sono
> **link** a pagine dedicate (campovasto, `/comprensione`, ecc.). L'admin NON
> incolla campovasto né il gate al proprio interno — è un launcher leggero.
> Campovasto è un'app satura (~40 moduli + vis-network 629KB): resta una pagina
> a sé, intoccata.

### Riallocazione delle 9 tab attuali

| Tab attuale | Destinazione |
|---|---|
| CHAT | Split: link a **Gate di Comprensione** (primario) + **Chat sperimentale** (declassata, reattiva) |
| CAMPO, FRATTALI | Coperti da **campovasto** → card-link |
| VOLONTÀ, SOGNO, NARRATIVA, IDENTITÀ, EPISODI | Unificare in una vista **"Stato Interno"** dedicata (sono tutte facce del sé). Ritirare i widget morti. |
| DIALOGO | Assorbito dalla chat sperimentale |

### Sfoltimento pagine (richiede verifica di Fable prima di cancellare)

| Pagina | Azione probabile |
|---|---|
| `campovasto/*` | **TENERE** — fondamenta + design language |
| `biennale/index.html` (`/biennale`,`/campo-vasto`) | consolidare in campovasto |
| `/community` + `campovasto/community.html` | tenerne **UNA** |
| `/curazione`, `/cura-mobile`, `/api/cura/*` | consolidare in **UNO** strumento di cura |
| `/universo`, `/diffrazione`, `/ui-r1`, `/dialogo` | verificare → quasi certo retire |
| `/` (`home.html`) | tenere come launcher pubblico |

> NB: l'elenco "retire" va confermato da Fable con un giro di verifica veloce
> prima della rimozione — non cancellare alla cieca.

## Regole di stile (vincolanti)

1. Riusa il theme centralizzato e `node-style` di **campovasto**. Nessun nuovo
   CSS monolitico, nessun font diverso (JetBrains Mono ovunque).
2. I-Ching = struttura interna, MAI estetica esibita all'utente: nessun
   trigramma/numero-di-frattale in vista utente; etichette in parole italiane.
3. Ogni vista dedicata ≤ un file leggibile; logica isolata per modulo (la
   lezione di campovasto: app.js ≤150 righe, node-style isolato).
4. L'admin NON deve più contenere logica di rendering pesante: solo launcher +
   vitali + fetch leggere.

## Endpoint dati disponibili (da consumare, già esistenti)

- vitali: `GET /api/state`
- campo: `GET /api/wordfield`, `GET /api/visuals` (frattali)
- stato interno: `GET /api/narrative`, `GET /api/self`, `GET /api/episodes`, `GET /api/will`
- comprensione (in arrivo, di Fable): `POST /api/comprehend` → arricchito con
  `coverage[]` (copertura per-token) + `saturation` (verdetto) + i due assi.
  **Questa è la fonte del Gate** — la vista dedicata `/comprensione` la consuma.
- cura: `GET /api/cura/parole`, `/api/relations`, ecc.

## Non-goal

- Non toccare il backend della comprensione (Rust, di Fable).
- Non inventare nuovi endpoint dati senza concordarli.
- Non costruire `/comprensione` (lo fa Fable come pagina dedicata e
  indipendente) — l'admin ci LINKA soltanto.
