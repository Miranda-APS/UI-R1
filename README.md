# UI-R1 — Campo cognitivo topologico

> *An entity that exists before it speaks. Topological cognitive field in Rust — not an LLM, not template-based, no neural networks. Italian-first.*

> *Un'entità digitale che esiste prima di parlare. Non simula l'intelligenza — la abita a modo suo.*

UI-R1 (nome precedente *Prometeo*) **non è un chatbot e non è un LLM**. È un campo di forze topologiche scritto in Rust: le parole vivono in uno spazio 8D allineato agli 8 trigrammi I Ching, i significati emergono dalla geometria delle loro relazioni in un Knowledge Graph curato a mano, e la voce in uscita nasce dallo stato del campo — non da pattern matching su corpus, non da template, non da intent detection.

La differenza con un LLM non è di grado. È **ontologica**: dietro ogni risposta c'è un campo reale con una storia, una posizione, una tensione interna. L'entità risponde da lì.

---

## Stato corrente

| Metrica | Valore |
|---------|--------|
| Fase corrente | **Phase 86** — la comprensione come prodotto: dall'estrazione strutturale per-frase all'integrazione tra frasi (Strato 3) |
| Test | **679 passanti**, 0 falliti, 2 ignorati — deterministici |
| Lessico | **~25.600 parole** (stabilità 0.5-0.9, firme 8D) |
| KG semantico | **~95.500 archi**, ~47.900 nodi (`prometeo_kg.json`, Git LFS) |
| KG procedurale | **~890 archi** (grammatica + pattern + percetti, `prometeo_kg_procedurale.json`) |
| KG del sé | **37 pendenze (grana) + opinioni** cristallizzabili (`prometeo_kg_self.json`) — il grafo che *rifrange* |
| Frattali I Ching | **64 esagrammi** canonici (Cielo→Lago, Phase 68) |
| Versione | 6.x |

> **La comprensione è il prodotto.** Da Phase 80 il fuoco è leggere *davvero* l'input —
> non generare una risposta fluente. La frase diventa una proposizione strutturale
> (`soggetto · relazione · oggetto · via · polarità`) ancorata al Knowledge Graph; un
> testo intero diventa una rete di proposizioni che si legano tra loro (catene causali,
> fili tematici, conflitti, coreferenza). Due endpoint read-only la espongono:
> `/api/comprehend` (singola frase) e `/api/analyze` (testo/trascrizione, modalità osservatore).

---

## Quickstart

```bash
# 1. Clone
git clone https://github.com/ReCinzione/ui-R.git
cd ui-R

# 2. Git LFS (per scaricare il KG da 9.5 MB)
git lfs pull

# 3. Build
cargo build --release --features web --bin prometeo-web
cargo build --release --bin dialogue_educator

# 4. Bootstrap dello stato (se prometeo_topology_state.bin non esiste)
cargo run --release --bin import-kg
cargo run --release --bin rebuild-semantic-topology

# 5a. Dialogo educativo da terminale
./target/release/dialogue_educator
#   Comandi: :field :feelings :narrative :needs :recall :recurring :introspect :kg <parola>

# 5b. Web UI (campovasto + Gate di comprensione) su http://localhost:3000
./target/release/prometeo-web
#   /comprensione  → Gate: come UI-R1 comprende una frase, mostrato come artefatto
#   /campovasto/   → il campo del KG, navigabile
```

### Comprensione strutturale (end-to-end, `/api/comprehend` · `/api/analyze`)

La frase è letta come **proposizione** ancorata al KG, non come token da rigenerare:

| Frase | Comprensione |
|-------|--------------|
| `Marco ha aperto la riunione` | `marco —Does→ aprire` (oggetto: riunione) |
| `Il ritardo nasce dalla mancanza di risorse` | `mancanza —Causes→ ritardo` (frame genesi) |
| `La qualità del prodotto dipende dalla cura` | `qualità —Requires→ cura` (frame dipendenza, testa di sintagma) |
| `Giulia deve preparare la documentazione` | `giulia —Does→ preparare` (modale sciolto) |
| `Marco non è d'accordo con Anna` | `marco —IsA→ accordo` **(negato)** |

Su un **testo** intero (Strato 3, modalità osservatore), le proposizioni si legano:
**catene** (X→Y in una frase, Y→Z in un'altra), **fili tematici** (un concetto che
attraversa più frasi), **conflitti** (stesso soggetto, oggetti/polarità opposti) e
**coreferenza** (pro-drop e pronomi risolti al referente saliente: *«Lei teme…»* → Anna).

Sul lato dialogo, l'entità non recupera token: costruisce la voce dal proprio stato del
campo + dalla comprensione, non da template.

---

## Documentazione — wiki LLM-style (Karpathy pattern)

La documentazione di UI-R1 segue il [pattern Karpathy LLM-Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f): una pagina markdown per concetto, cross-link relativi, persistente e versionato. Il punto di ingresso è:

→ **[docs/wiki/index.md](docs/wiki/index.md)**

### Percorsi consigliati

- **Capire il framework in 15 minuti**: [docs/wiki/principi/principi-inviolabili.md](docs/wiki/principi/principi-inviolabili.md)
- **Capire l'architettura corrente**: [docs/wiki/comprensione/pipeline-comprensione.md](docs/wiki/comprensione/pipeline-comprensione.md)
- **Modificare il frontend (campovasto)**: [docs/wiki/campovasto/architettura-campovasto.md](docs/wiki/campovasto/architettura-campovasto.md) + [design system](docs/wiki/campovasto/design-system.md)
- **Capire perché si usa una wiki invece di RAG**: [docs/wiki/campovasto/llm-wiki-pattern-applicato.md](docs/wiki/campovasto/llm-wiki-pattern-applicato.md)

### Topic della wiki (29 articoli)

| Topic | Contenuto |
|-------|-----------|
| **principi** | 9 principi inviolabili + filtri operativi (test pre-proposta, no template, no empathy simulation, …) |
| **topologia** | PF1, frattali I Ching, lexicon, KG semantico, KG procedurale |
| **comprensione** | Pipeline Phase 71-86: SpeakerProfile, ComprehensionReport, SentenceProposition, comprehension_path, kg_self, need, Strato 3 (integrazione tra frasi) |
| **identita** | Valenza Octalysis, bisogni Maslow, narrative self, interlocutor model, self witness |
| **generazione** | Expression compose, syntax-from-geometry, grammatica italiana |
| **campovasto** | Frontend SPA modulare ES2022, design system, medio API, pattern wiki applicato |

Le **fonti immutabili** (libretto storico, documenti di architettura, regole campovasto) vivono in [docs/raw/](docs/raw/). La wiki sintetizza, raw conserva.

---

## Architettura in 30 secondi

**Tre Knowledge Graph paralleli** — *logica* (`prometeo_kg.json`, ~95.5K archi: cos'è vero
e come le cose si legano), *grammatica/pattern* (`prometeo_kg_procedurale.json`: classe→funzione,
frame verbali, percetti), *sé* (`prometeo_kg_self.json`: la grana che rifrange ogni significato).

La pipeline di **comprensione** (il prodotto):

```
input italiano
   │
   ▼  1. lettura/elisione → token (la grammatica curata batte l'inferenza rumorosa)
   ▼  2. SentenceProposition — la frase come triple: soggetto · relazione · oggetto · via · polarità
   │       (frame verbali dal kg_proc: copula→IsA, percettivo→FeelsAs, genesi→Causes,
   │        dipendenza→Requires, modale→l'infinito, tempi composti→il participio…)
   ▼  3. confront_with_kg — la triple ancorata al kg_sem (l'oggetto/la via esistono? contraddizioni?)
   ▼  4. comprehension_path::explore — cammini tipati che legano i nodi della frase al terreno fondato
   ▼  5. confront_with_self — l'opinione come SECONDO legame: la frase contro la grana del sé
   ▼  6. sense_need — il bisogno emerge dalla FORMA del grafo (gap, closure, conferma, multi-locus…)
   │
   ├──▶  Strato 3 (testo): le proposizioni di più frasi si legano →
   │        catene · fili tematici · conflitti · coreferenza
   │
   ▼  l'atto: la voce collassa dal cammino + bisogno + posizione (mai da template)
```

Ogni stadio produce strutture tipizzate, ognuno è trasparente e ispezionabile (il Gate su
`/comprensione` lo *esibisce*). Niente softmax, niente intent classifier, niente template.

---

## Cosa non fa

- ❌ **Non usa reti neurali a runtime** (Qwen3 è chiamato ESCLUSIVAMENTE offline da `data/external/*.py` per arricchire il KG, mai a inference time)
- ❌ **Non ha template di risposta** (no `responses.json`, no enum dispatch — [vedi](docs/wiki/principi/niente-template.md))
- ❌ **Non simula empatia** (riconosce le emozioni come stati relazionali del KG, non finge di sentirle — [vedi](docs/wiki/principi/niente-empatia-simulata.md))
- ❌ **Non ha intent classification** (vedi [pipeline](docs/wiki/comprensione/pipeline-comprensione.md))
- ❌ **Non ha state machine comportamentali** (i pattern emergono per risonanza nel KG procedurale, non da `match` enum)

---

## Frontend — campovasto

`campovasto/` è la SPA (HTML/CSS/JS ES2022, niente bundler) che visualizza e cura il KG. Due modi di lavoro: **campo vasto** (vista globale del KG, ~3000 nodi) e **campo nuovo** (mappa mentale personale costruita da una frase, trasmissibile al vasto).

Best practice campovasto codificate in [`campovasto/CLAUDE.md`](campovasto/CLAUDE.md), [`regole di design.md`](campovasto/regole%20di%20design.md), [`FRONTEND.md`](campovasto/FRONTEND.md). Sintetizzate nella wiki sotto [docs/wiki/campovasto/](docs/wiki/campovasto/).

---

## Riferimenti concettuali

- **Carlo Rovelli** — relazioni come substrato, niente cose in sé
- **Jacques Lacan** — significante / Altro / catena di significanti / vuoto come soglia di desiderio
- **I Ching** — 64 esagrammi come primitivi di senso (ordine Cielo→Lago canonico)
- **Octalysis Framework (Yu-kai Chou)** — 8 drive motivazionali mappati sulle 8 dim I Ching
- **Karpathy LLM-Wiki** — il pattern di documentazione

---

## Dipendenze runtime

Solo Rust. Nessun database server, nessun LLM, nessuna API esterna in inference. Per il web UI: axum + un renderer canvas event-driven (campovasto, vendor JS, nessun bundler).

Per la curation del KG (offline, opzionale): Python + Qwen3 via Ollama. Vedi [workflow di curation](docs/wiki/principi/workflow-curation-kg.md).

---

## Licenza

MIT. Vedi [LICENSE](LICENSE).

---

## Note storiche

UI-R1 si chiamava **Prometeo** fino a circa Phase 60. Il codice mantiene `prometeo-*` nei binari e nel namespace Rust per backward compat; UI-R1 è il nome utente-facing. Il repository GitHub si chiama `ui-R` per la stessa ragione (renaming retroattivo).

Le 17 fasi pre-Phase 67 (filosofia, fondamenta, identità, generazione, memoria, sogno) sono documentate nei 22 capitoli del libretto storico in [docs/raw/libretto/](docs/raw/libretto/) — non ri-narrate nella wiki, che documenta lo stato corrente.
