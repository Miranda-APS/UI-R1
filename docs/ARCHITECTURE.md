# Prometeo — Architettura Tecnica

**Versione**: 6.12.0 — Phase 64
**Data**: 2026-04-08
**Linguaggio**: Rust
**Metriche**: 25.875 parole · 64 frattali · 165.326 archi KG · ~40.000 nodi · 476 test

---

## Indice

1. [Panoramica architetturale](#1-panoramica-architetturale)
2. [Layer 0 — Primitivi e persistenza](#2-layer-0--primitivi-e-persistenza)
3. [Layer 1 — Semantica e Knowledge Graph](#3-layer-1--semantica-e-knowledge-graph)
4. [Layer 2 — Campo topologico](#4-layer-2--campo-topologico)
5. [Layer 3 — Orchestrazione (Engine)](#5-layer-3--orchestrazione-engine)
6. [Layer 4 — Volontà, memoria e sogno](#6-layer-4--volontà-memoria-e-sogno)
7. [Layer 5 — Espressione linguistica](#7-layer-5--espressione-linguistica)
8. [Layer 6 — Narrativa e ragionamento](#8-layer-6--narrativa-e-ragionamento)
9. [Layer 7 — Identità](#9-layer-7--identità)
10. [Layer 8 — Interazione](#10-layer-8--interazione)
11. [Flusso receive()](#11-flusso-receive)
12. [Flusso generate_willed()](#12-flusso-generatewilled)
13. [Flusso autonomous_tick()](#13-flusso-autonomoustick)
14. [Formato binario di persistenza](#14-formato-binario-di-persistenza)
15. [Tabella costanti di sistema](#15-tabella-costanti-di-sistema)

---

## 1. Panoramica architetturale

Prometeo è un motore cognitivo topologico a 8 dimensioni. L'architettura è organizzata in 9 layer, dal substrato matematico all'interfaccia web.

```
Layer 8  INTERAZIONE        web/server.rs, web/api.rs, web/community/
Layer 7  IDENTITÀ            identity.rs, self_model.rs, semantic_episode.rs
Layer 6  NARRATIVA           narrative.rs, input_reading.rs, proposition.rs, thought.rs, reasoning.rs
Layer 5  ESPRESSIONE         state_translation.rs, syntax_center.rs, grammar.rs, generation.rs
Layer 4  VOLONTÀ + MEMORIA   will.rs, memory.rs, episodic.rs, dream.rs, needs.rs, desire.rs
Layer 3  ORCHESTRAZIONE      engine.rs (~4500 righe)
Layer 2  CAMPO               word_topology.rs, simplex.rs, pf1.rs
Layer 1  SEMANTICA           knowledge_graph.rs, relation.rs, knowledge.rs, inference.rs
Layer 0  PRIMITIVI           primitive.rs, lexicon.rs, fractal.rs, persistence.rs, simpdb.rs
```

I dati fluiscono dal basso verso l'alto durante la ricezione dell'input, e dall'alto verso il basso durante la generazione. L'orchestratore centrale (`engine.rs`) coordina tutti i layer.

---

## 2. Layer 0 — Primitivi e persistenza

### 2.1 PrimitiveCore — Lo spazio 8D

Ogni entità nel sistema (parola, frattale, stato) è un punto in ℝ⁸, dove ogni dimensione è un asse continuo [0.0, 1.0]:

| Indice | Nome | Polarità 0.0 | Polarità 1.0 |
|--------|------|--------------|--------------|
| 0 | Confine | esterno/pubblico | interno/privato |
| 1 | Valenza | repulsione | attrazione |
| 2 | Intensità | debole | forte |
| 3 | Definizione | vago | netto |
| 4 | Complessità | semplice | composto |
| 5 | Permanenza | transitorio | stabile |
| 6 | Agency | paziente | agente |
| 7 | Tempo | passato | futuro |

**Operazioni definite su PrimitiveCore:**
- `distance(a, b)` — distanza euclidea in ℝ⁸
- `weighted_distance(a, b, w)` — distanza con pesi dimensionali
- `perturb_towards(target, rate)` — interpolazione lineare: `self[i] += (target[i] - self[i]) × rate`
- `blend(a, b, ratio)` — media pesata: `a[i] × ratio + b[i] × (1 - ratio)`
- `energy()` — norma L2 del vettore

### 2.2 Frattali — I 64 esagrammi

I 64 frattali sono regioni dello spazio 8D, calcolati come combinazioni di 8 trigrammi:

```
FractalId = lower_trigram × 8 + upper_trigram    →    range [0, 63]
```

**Gli 8 trigrammi:**

| Idx | Simbolo | Nome | Dimensione fissa | Valore |
|-----|---------|------|------------------|--------|
| 0 | ☰ | Cielo | Agency | 0.90 |
| 1 | ☷ | Terra | Permanenza | 0.10 |
| 2 | ☳ | Tuono | Intensità | 0.30 |
| 3 | ☵ | Acqua | Tempo | 0.30 |
| 4 | ☶ | Montagna | Confine | 0.30 |
| 5 | ☴ | Vento | Complessità | 0.70 |
| 6 | ☲ | Fuoco | Definizione | 0.70 |
| 7 | ☱ | Lago | Valenza | 0.70 |

La struttura replica la combinatoria dell'I Ching: 2 polarità → 8 trigrammi (2³) → 64 esagrammi (8²). Ogni esagramma ha una firma 8D derivata dalla composizione dei suoi due trigrammi.

**Scelta progettuale**: 8 dimensioni non sono un numero arbitrario. Sono il minimo necessario per coprire le polarità fondamentali dell'esperienza (spaziale, temporale, emotiva, agentiva, strutturale) mantenendo la combinatoria trattabile: 8² = 64, un numero di regioni sufficiente a coprire gli stati cognitivi senza essere ingestibile.

**Esagrammi puri** (trigramma × se stesso):

| Id | Formula | Nome | Caratteristica |
|----|---------|------|----------------|
| 0 | ☰☰ | POTERE | Agency massima |
| 9 | ☷☷ | MATERIA | Permanenza minima, ricettività |
| 18 | ☳☳ | ARDORE | Intensità pura |
| 27 | ☵☵ | DIVENIRE | Flusso temporale |
| 36 | ☶☶ | SPAZIO | Confine/limite |
| 45 | ☴☴ | INTRECCIO | Complessità strutturale |
| 54 | ☲☲ | VERITÀ | Definizione/chiarezza |
| 63 | ☱☱ | ARMONIA | Valenza attrattiva |

### 2.3 Lessico

Ogni parola nel lessico è un `WordPattern`:

```rust
struct WordPattern {
    signature: PrimitiveCore,           // firma 8D [f32; 8]
    fractal_affinities: HashMap<FractalId, f64>,  // peso per ogni frattale
    exposure_count: u64,                // numero di esposizioni
    stability: f64,                     // solidità della firma
    co_occurrences: HashMap<String, u32>,   // contesti affermativi
    co_negated: HashMap<String, u32>,       // contesti negativi
    co_affirmed: HashMap<String, u32>,      // contesti esplicitamente affermativi
}
```

**Formula di stabilità:**
```
stability = (1.0 - 1.0 / (1.0 + exposure × 0.1)).min(0.95)
```
Crescita logaritmica asintotica a 0.95. Una parola con 10 esposizioni ha stabilità ≈ 0.50, con 100 ha ≈ 0.91.

**Apprendimento per esposizione:**
```
learning_rate = (1.0 - stability) × 0.35
signature.perturb_towards(context_signature, learning_rate)
```
Le prime esposizioni spostano la firma fortemente (fino a 0.35). Man mano che la stabilità cresce, la firma si consolida e diventa resistente al cambiamento.

**Soglia di stabilità per l'uso in generazione:**
```
is_stable() = stability > 0.30 AND exposure_count ≥ 5
```

**Scelta progettuale**: la stabilità logaritmica simula l'apprendimento umano — le prime esperienze formano la maggior parte del significato, le successive lo raffinano. Il tetto a 0.95 garantisce che nessuna parola sia mai completamente impermeabile al contesto.

### 2.4 clean_token()

Normalizzazione dell'input: rimuove punteggiatura (`:.,;!?()«»—–`), gestisce contrazioni con apostrofo (prende il segmento dopo l'ultimo apostrofo), scarta token senza caratteri alfabetici. Restituisce `Option<String>` — None per token non validi.

---

## 3. Layer 1 — Semantica e Knowledge Graph

### 3.1 Knowledge Graph

Grafo direzionato tipizzato con doppio indice:

```rust
struct KnowledgeGraph {
    outgoing: HashMap<String, HashMap<RelationType, Vec<KgTarget>>>,
    incoming: HashMap<String, HashMap<RelationType, Vec<String>>>,
}

struct KgTarget {
    object: String,
    confidence: f32,    // [0.0, 1.0]
    source: EdgeSource, // Manual, Agent, BigBang
}
```

**Metriche attuali:** 165.326 archi, ~40.000 nodi. Firme 8D riderivate da struttura KG per 21.709 parole (Phase 63 — geometria = relazioni, non co-occorrenze statistiche).

**Fonti dati:**

| File | Archi | Tipi | Origine |
|------|-------|------|---------|
| italian_core.tsv | 623 | tutti | curati manualmente |
| bigbang_kg.tsv | 118.810 | SIMILAR_TO, OPPOSITE_OF | estratti da Kaikki (Wiktionary) |
| agent_kg.tsv | 17.763 | IS_A | generati da Qwen3 via Ollama |
| agent_kg_full.tsv | 44.390 | CAUSES, PART_OF, USED_FOR | generati da Qwen3 |

**Caricamento:** `load_from_dir()` legge tutti i file .tsv dalla directory `data/kg/`. Ogni riga ha formato `subject\trelation\tobject\tconfidence`.

### 3.2 Tipi di relazione (18)

Organizzati in 4 categorie funzionali:

**Strutturali:**
| Tipo | Fase | Peso base | Esempio |
|------|------|-----------|---------|
| IS_A | 0.10 | 0.80 | cane IS_A animale |
| HAS | 0.20 | 0.70 | nazione HAS confine |
| DOES | 0.20 | 0.70 | sole DOES brillare |
| PART_OF | 0.15 | 0.75 | mano PART_OF corpo |

**Causali:**
| Tipo | Fase | Peso base | Esempio |
|------|------|-----------|---------|
| CAUSES | 0.35 | 0.65 | fuoco CAUSES calore |
| ENABLES | 0.25 | 0.60 | chiave ENABLES aprire |
| REQUIRES | 0.28 | 0.58 | fuoco REQUIRES ossigeno |
| TRANSFORMS_INTO | 0.40 | 0.62 | ghiaccio TRANSFORMS_INTO acqua |

**Semantiche:**
| Tipo | Fase | Peso base | Esempio |
|------|------|-----------|---------|
| SIMILAR_TO | 0.00 | 0.90 | ciao SIMILAR_TO saluto |
| OPPOSITE_OF | π | 0.50 | caldo OPPOSITE_OF freddo |
| USED_FOR | 0.30 | 0.55 | coltello USED_FOR tagliare |
| EXPRESSES | 0.22 | 0.65 | sorriso EXPRESSES gioia |
| SYMBOLIZES | 0.18 | 0.55 | colomba SYMBOLIZES pace |
| CONTEXT_OF | 0.12 | 0.50 | inverno CONTEXT_OF neve |

**Logiche:**
| Tipo | Fase | Peso base | Esempio |
|------|------|-----------|---------|
| IMPLIES | 0.35 | 0.68 | pioggia IMPLIES bagnato |
| EQUIVALENT | 0.00 | 0.92 | felicità EQUIVALENT gioia |
| EXCLUDES | π/2 | 0.45 | vita EXCLUDES morte |
| COEXISTS | 0.05 | 0.60 | domanda COEXISTS risposta |

**La fase** codifica la natura della relazione nella propagazione: `cos(0) = +1` (risonanza pura per SIMILAR_TO), `cos(π) = -1` (inibizione per OPPOSITE_OF), `cos(π/2) = 0` (neutralità per EXCLUDES).

**Scelta progettuale**: la fase come angolo permette una formula di propagazione unificata — `contributo = attivazione × peso × cos(fase)` — senza branching per tipo di relazione. Il coseno ruota continuamente da risonanza a opposizione.

### 3.3 InferenceEngine

**Transitività IS_A:**
```
type_chain(word) → BFS fino a profondità MAX_ISA_DEPTH = 5
Esempio: cane →IS_A→ mammifero →IS_A→ animale →IS_A→ essere_vivente
```

**Field boosts (propagazione semantica):**
Per ogni parola attiva, il motore cerca i suoi vicini nel KG e li attiva:
```
boost_strength = field_boost_strength(tipo) × confidence × decay_per_hop
  IS_A diretto:     isa_base × confidence
  IS_A transitivo:  isa_base × 0.7^hop
  DOES/HAS:         base × confidence (diretto) + base × 0.6 (ereditato via IS_A)
  CAUSES/SIMILAR_TO: type_base × confidence
```

**Scelta progettuale**: la transitività IS_A limitata a 5 hop e con decay 0.7 per hop bilancia la ricchezza inferenziale con il rischio di attivare regioni troppo remote. SIMILAR_TO non è transitivo — se A è simile a B e B è simile a C, non ne segue che A sia simile a C.

---

## 4. Layer 2 — Campo topologico

### 4.1 PrometeoField (PF1) — Substrato

Il cuore computazionale del sistema. Struttura ROM+RAM per parola:

**ROM (WordRecord) — 512 byte per parola, layout fisso:**
```
[0..32]     firma 8D (8 × f32)
[32..288]   affinità frattali (64 × f32)
[288..292]  stability (f32)
[292..296]  exposure_count (u32)
[296..298]  dominant_fractal (u16)
[298]       POS tag (u8: 0=Sconosciuto, 1=Verbo, 2=Nome, 3=Aggettivo, 4=Avverbio)
[299]       lunghezza parola (u8)
[300..332]  parola UTF-8 (max 32 byte)
[332]       neighbor_count (u8, max 8)
[336..368]  neighbor word_ids ([u32; 8])
[368..400]  neighbor weights ([f32; 8])
[400..432]  neighbor phases ([f32; 8])
[432..512]  padding riservato
```

**RAM (ActivationState):**
```rust
activations: Vec<f32>,      // attivazione corrente per parola
counts: Vec<u64>,           // conteggio attivazioni
threshold: f32,             // 0.02
synapse_weights: Vec<f32>,  // pesi Hebbiani, indicizzati [word_id × 8 + slot]
```

**Formula di propagazione — O(parole_attive × 8):**
```
Per ogni parola con attivazione > threshold (0.02):
  Per ciascuno dei MAX_NEIGHBORS (8) vicini:
    contributo = attivazione_sorgente × damping(0.15) × peso × cos(fase)

    Se cos(fase) > 0 (risonanza):
      attiva solo se target sotto soglia
    Se cos(fase) < 0 (opposizione):
      inibisci a qualsiasi livello

    target_activation = (target + contributo).clamp(0.0, 1.0)
```

**Apprendimento Hebbiano (per connessione):**
```
Se entrambi attivi (> threshold):
  LTP: synapse_weight += 0.05 × min(act_src, act_tgt)
Se sorgente attiva, target silente:
  LTD: synapse_weight *= 0.995
```

**Stato di riposo:**
```
resting_activation = stability × 0.02
```
Le parole più stabili hanno un'attivazione basale più alta — sono "sempre un po' accese", come concetti fondamentali che permangono nella coscienza.

**Scelta progettuale**: MAX_NEIGHBORS = 8 è un vincolo deliberato. Limita la complessità computazionale a O(attive × 8) anziché O(attive × grado_medio), rende la propagazione deterministica e previene che nodi hub dominino il campo. Gli 8 vicini sono scelti al momento della costruzione come i più rilevanti per peso.

### 4.2 WordTopology — Grafo parole

Grafo pesato con fase sugli archi, costruito dal Knowledge Graph:

**Formula peso arco (hub damping, Phase 48):**
```
peso = type_base(relazione) × confidence × hub_factor

hub_factor = 1.0 / (1.0 + ln(max(grado_a, grado_b) / mediana_gradi))
```

Effetto: un nodo con grado >> mediana viene penalizzato logaritmicamente. "essere" con 5.000 archi: hub_factor ≈ 0.20. "cane" con 20 archi: hub_factor ≈ 0.85.

**Formula peso co-occorrenza (archi statistici):**
```
peso = ln(conteggio) / ln(max_globale)     clamped a [0.01, 1.0]
```

**Stato attuale:** topologia semantica pura — 141.205 archi KG-derivati, 0 archi statistici. Gli archi statistici sono stati rimossi dopo la Phase 48 perché il KG fornisce relazioni più ricche e interpretabili.

**Scelta progettuale**: il hub damping logaritmico risolve il problema dei verbi hub che dominavano la topologia. Senza damping, "essere" (5000 archi) avrebbe peso 250× quello di "cane" (20 archi). Con damping, il rapporto scende a ~4×. La scala logaritmica è stata scelta perché penalizza in modo proporzionale — non annulla i nodi hub, li ridimensiona.

### 4.3 SimplicialComplex — Topologia inter-frattale

Connessioni di ordine superiore tra frattali:

```rust
struct Simplex {
    id: SimplexId,
    vertices: Vec<FractalId>,         // frattali connessi
    shared_faces: Vec<SharedFace>,    // strutture condivise
    dimension: usize,                 // vertices.len() - 1
    persistence: f64,                 // [0, 1] — solidità strutturale
    plasticity: f64,                  // [0, 1] — capacità di cambiamento
    activation_count: u64,
    current_activation: f64,
    source_words: Option<Vec<String>>, // parole che lo hanno generato (Phase 52)
}
```

**Dinamica dei simplessi:**
```
Ad ogni attivazione:
  persistence += 0.003 (cap 1.0)     — più si attiva, più diventa solido
  plasticity  *= 0.995 (min 0.05)    — più si attiva, meno può cambiare

Decadimento:
  current_activation = (activation - rate).max(0.0)
```

La coppia persistence/plasticity implementa la cristallizzazione: un simplesso giovane è molto plastico e poco persistente (può cambiare facilmente ma scompare in fretta). Uno vecchio è poco plastico e molto persistente (resiste al decadimento ma non cambia più). Come le ossa — flessibili nei giovani, rigide negli anziani.

**Risonanza → parole (Phase 52):** quando un simplesso risuona (attivazione alta), le sue `source_words` vengono re-iniettate nel campo PF1 con boost 0.15. Questo chiude il ciclo comprensione→generazione: i concetti cristallizzati nella topologia alimentano il campo attivo.

---

## 5. Layer 3 — Orchestrazione (Engine)

`PrometeoTopologyEngine` (~4500 righe) è il cuore del sistema. Tre funzioni principali:

1. **receive(input)** — ricezione input, perturbazione campo, deliberazione
2. **generate_willed()** — generazione risposta guidata dalla volontà
3. **autonomous_tick()** — ciclo autonomo (sogno, manutenzione, abduzione)

I flussi dettagliati sono nelle sezioni 11-13.

**Sincronizzazione PF1 ↔ WordTopology:**
Il sistema ha due substrati di attivazione (PF1 semantico e word_topology legacy). `propagate_field_words()` li sincronizza:
1. PF1 propaga (O(attive × 8))
2. Hebbian update (LTP/LTD sulle sinapsi)
3. Identity resonance: le prime 200 parole più attive vengono modulate per risonanza identitaria [0.7, 1.3]
4. Sync PF1 → word_topology: azzera word_topology, copia le prime 500 parole attive da PF1

**Scelta progettuale**: la doppia attivazione è un'eredità storica — word_topology era il substrato originale, PF1 è stato aggiunto dopo per la propagazione O(attive×8). state_translation.rs legge ancora da word_topology, quindi la sincronizzazione è necessaria. Una futura semplificazione potrebbe eliminare word_topology.

---

## 6. Layer 4 — Volontà, memoria e sogno

### 6.1 Will — Volontà emergente

Sette pressioni calcolate dallo stato del campo. La più forte determina l'intenzione (drive), le due dimensioni più attive determinano il codon [usize; 2]:

**Express** — pressione a comunicare (Phase 64 — drive-dipendente):
```
max_drive = max(|octalysis_drives[i]|)

Se max_drive > 0.25:
  pressione = max_drive × freshness × has_content × 0.8    — drive dominante attivo
Altrimenti:
  pressione = activation × freshness × has_content × 0.20  — canale passivo

  has_content = 1.0 se active_fractals non vuoto
  freshness = 1.0 - fatigue
  soglia: > 0.05
```

**Principio Phase 64 (Express come canale)**: Express non è un movente — è il canale attraverso cui passa un contenuto guidato da un drive Octalysis specifico. Senza un drive attivo (|d| > 0.25), la pressione Express scende da 0.8× a 0.20× dell'attivazione. L'entità non esprime per abitudine: esprime perché qualcosa di specifico la muove.

**Explore** — pressione ad esplorare:
```
Se parole sconosciute presenti:
  word_pull = (n_sconosciute × 0.3).min(1.0)
  pressione = word_pull × (0.4 + curiosity × 0.6) × (1.0 - fatigue)
soglia: > 0.05
```

**Question** — pressione a domandare:
```
Se gap di curiosità presenti:
  gaps = (n_gaps × 0.2).min(1.0)
  pressione = gaps × curiosity × (0.3 + (1.0 - activation) × 0.5)
soglia: > 0.05
```

**Remember** — pressione a ricordare:
```
pressione = (memory_resonance × 0.7 + saturation × 0.3).min(1.0)
soglia: > 0.1
```

**Withdraw** — pressione a ritirarsi:
```
fatigue_pull = fatigue × 0.8   (se fatigue > 0.75, altrimenti 0)
overload_pull = 0.45            (se tension == Overloaded, altrimenti 0)
stillness_pull = 0.5            (se activation < 0.05 e nessuna parola sconosciuta, altrimenti 0)
pressione = max(fatigue_pull, overload_pull, stillness_pull)
soglia: > 0.05
```

**Reflect** — pressione a riflettere:
```
pressione = ego_activation × 0.6 × (1.0 - fatigue)
soglia: > 0.15
```

**Instruct** — pressione a insegnare:
```
relational = (empatia[59] + comunicazione[47]) × 0.5
Se relational > identita[32] + 0.15 E activation > 0.2:
  pressione = relational × (1.0 - fatigue) × 0.7
soglia: > 0.1
```

**Codon:** le 2 dimensioni con i valori più alti nella firma del campo. Usato per pesare le parole nella generazione: `codon_weight = (sig[codon[0]] + sig[codon[1]]) / 2`.

**Modulazione post-hoc (Phase 53):**
- I bisogni (`NeedsHierarchy.compute_pressure()`) modulano le pressioni dopo il calcolo
- I desideri (`DesireCore.will_biases()`) aggiungono compound bias per intenzione
- L'interlocutore modula: presenza alta → sopprime Withdraw; risonanza alta → amplifica Express; novità alta → amplifica Explore + Question

### 6.2 Memory — Memoria topologica

Tre strati a velocità di decadimento diversa:

| Strato | Capacità | Promozione | Decadimento | Ruolo |
|--------|----------|------------|-------------|-------|
| STM | 20 imprint, FIFO | — | veloce | forma attuale del campo |
| MTM | illimitato | da STM dopo ≥5 occorrenze (o ≥3 in consolidate_light) | ×0.1 rispetto a STM | postura del campo |
| LTM | illimitato | da MTM dopo ≥20 tick E strength > 0.5 | ×0.01 rispetto a STM | scheletro permanente |

**Consolidation (DeepSleep):**
```
Per ogni simplesso in STM:
  contare occorrenze
  Se occorrenze ≥ consolidation_threshold (5):
    promuovi a MTM con strength = 0.8

Per ogni imprint in MTM:
  Se tick_corrente - tick_imprint > 20 E strength > 0.5:
    promuovi a LTM
```

**Consolidate_light (Phase 52) — ogni 25 tick:**
```
Soglia ridotta: ≥3 occorrenze (vs 5 per DeepSleep)
Strength ridotta: 0.5 (vs 0.8)
Evita duplicati MTM
```

**Risonanza (retrieval):**
```
similarity = Σ(attivazioni_simplessi_comuni) / Σ(tutte_attivazioni_passate)
Se similarity > 0.3: l'imprint risuona
```

**Scelta progettuale**: la memoria non è un database con query. È risonanza — lo stato presente "vibra" con gli stati passati topologicamente simili. Non c'è ricerca: c'è amplificazione selettiva. Questo è coerente con il modello bergsoniano in cui il passato agisce nel presente per deformazione del campo.

### 6.3 EpisodicMemory — Memoria narrativa

```rust
struct EpisodicTrace {
    timestamp, turn_number, locus_fractal, phrase, input_text,
    speaker, emotional_tone, salience
}
```

Capacità: 100 episodi (buffer circolare). Cercabile per speaker, range temporale, prossimità 8D, regione frattale. Usato in `recall_into()` per pattern completion φ-pesata.

### 6.4 Dream — Ciclo del sogno

Cinque fasi con durate e comportamenti distinti:

| Fase | Durata | Soglia attivazione | Decay complesso | Azione |
|------|--------|--------------------|-----------------|--------|
| Awake | 5 tick dopo input | 0.15 | — | attenzione esterna piena |
| WakefulDream | default | 0.12 | 0.003 | esplorazione autonoma |
| LightSleep | pre-DeepSleep | 0.10 | 0.005 | dissoluzione simplessi fragili |
| DeepSleep | 10 tick | 0.25 | consolidation | memory.consolidate() + crystallize() |
| REM | 20 tick | 0.05 → 0.01 | 0.008 | soglie basse, connessioni remote |

**Transizione:** ogni 50 perturbazioni → DeepSleep (10 tick) → REM (20 tick) → WakefulDream.

**Soglia REM dinamica:**
```
soglia = (0.05 - depth × 0.03).max(0.01)
  depth ∈ [0, 1], più profondo = soglia più bassa
```

**REM: attivazione sparsa:** 1 parola ogni 3 tra le top-100 per stabilità, attivata a `stability × 0.001`. Questo permette connessioni remote — regioni normalmente separate possono entrare in contatto.

**Scelta progettuale**: il REM con soglie dinamiche implementa il meccanismo della creatività come "rilassamento dei vincoli topologici". Non è randomizzazione — è propagazione a lungo raggio resa possibile dall'abbassamento controllato delle barriere.

### 6.5 NeedsHierarchy — Bisogni (Phase 53)

7 livelli gerarchici con prepotency gate:

**L1 — Sopravvivenza:**
```
sat = field_alive × 0.4 + (1 - fatigue) × 0.35 + (1 - overloaded) × 0.25
```

**L2 — Coerenza:**
```
sat = has_identity × 0.1 + continuity × 0.75 + belief_anchor × 0.15
  belief_anchor = count(confidence > 0.5) / 5
```

**L3 — Espressione:**
```
sat = (activation × 3).min(1) × 0.4 + (1 - fatigue) × 0.3 + (word_count / 20).min(1) × 0.3
```

**L4 — Comprensione:**
```
curiosity_satisfied = 1 - (curiosity - 0.3).max(0) / 0.7
sat = curiosity_satisfied × 0.4 + coverage × 0.3 + (1 - uncertainty_load) × 0.3
```

**L5 — Connessione (Phase 55: base alta quando qualcuno parla):**
```
has_interlocutor = 0.90 se dialogue_turn_count > 2
                   0.75 se dialogue_turn_count > 0
                   0.50 altrimenti (anche da solo, sopra soglia crisi)
sat = has_interlocutor × 0.5 + dialogue_coherence × 0.30 + saturation × 0.20
```
Un umano non dice "cerco connessione" a chi gli sta parlando.

**L6 — Crescita:**
```
sat = novelty × 0.40 + identity_movement × 0.35 + coverage × 0.25
```

**L7 — Trascendenza:**
```
identity_healthy = 0.2 se crisi, 0.4 se stagnante, 1.0 altrimenti
value_stability = media(weights) se valori presenti, 0.5 altrimenti
sat = identity_healthy × 0.4 + value_stability × 0.3 + coverage × 0.3
```

**Prepotency gate:** se max(L1, L2, L3) deficit > 0.4:
```
soppressione = deficit × 0.5
Explore  *= 1 - soppressione × 0.3
Question *= 1 - soppressione × 0.2
Instruct *= 1 - soppressione × 0.4
```

**Soglie:**
```
NEED_THRESHOLD = 0.5    — sotto: bisogno attivo
CRISIS_THRESHOLD = 0.35 — sotto: genera pensiero Need
```

### 6.6 DesireCore — Desideri (Phase 53)

Attrattori stabili nello spazio 8D:

```
MAX_DESIRES = 5
DECAY_PER_TICK = 0.995
SATISFACTION_DISTANCE = 0.2 (coseno)
SATISFACTION_TICKS = 3
```

**Sorgenti di desiderio (Phase 64):**

| Fonte | Condizione | Intensità iniziale |
|-------|------------|-------------------|
| `OctalysisDriven(cd, val)` | `last_comprehension` non vuoto + `\|drives[cd]\|` > 0.28 | `drive_abs × 0.65` |
| `Undercurrent(intention)` | stessa intenzione come sottocorrente ≥5 volte | decrescente |
| `Value(label)` | SelfModel value con peso > 0.75 | proporzionale al peso |
| `IdentityTension` | tensioni identitarie irrisolte | fisso |
| `EpisodicEcho` | tracce episodiche con emozione positiva | proporzionale alla salienza |

**Fonte primaria Phase 64 — OctalysisDriven:**
```
DRIVE_DIM = [6,3,4,0,1,7,2,5]  (CD1→8 mappati su dim 0→7 dello spazio 8D)

Trova CD dominante: cd = argmax(|drives[i]| > 0.28)
target_sig = field_sig.clone()
target_sig[DRIVE_DIM[cd]] += 0.35 × drive_abs
  +0.12 × comprehension_weight per ogni concetto IS_A compreso

Se desiderio simile esiste: intensity += 0.08 × drive_abs
Altrimenti: crea nuovo desiderio OctalysisDriven(cd, drive_val)
```

**Il principio**: il desiderio nasce dall'incrocio tra *cosa il KG ha capito* (last_comprehension: attrattori IS_A raggiunti) e *quale drive Octalysis risponde* a quella comprensione. Non "voglio esprimere" (circolare) ma "data comprensione X e CD5 Relazione attivo, voglio connettere nella direzione del campo".

**Soddisfazione:**
```
dist = cosine_distance(target_sig, field_sig)
Se dist < 0.2 per 3 tick consecutivi → soddisfatto, rimosso
```

**Rinforzo:**
```
sim = cosine_sim(target_sig, field_sig)
Se sim > 0.5: intensity += 0.05 × sim
```

**Decadimento accelerato:** se non rinforzato per >200 tick: `intensity *= 0.98` addizionale.

---

## 7. Layer 5 — Espressione linguistica

### 7.1 state_translation — Campo → italiano

La traduzione non è template matching. È **proiezione strutturale**: il campo seleziona parole basandosi su attivazione, rilevanza dimensionale e proprietà linguistiche.

**SentenceArchetype** — struttura della frase:
```rust
enum SlotRole {
    Literal(String),           // parola fissa ("io", "non")
    PrimaryWord,               // parola più attiva, pesata per codon
    VerbCandidate,             // parola con max Agency, coniugata
    FractalWord(FractalId),    // miglior parola per quel frattale
    EmotionWord,               // EMOZIONE → CORPO → ARMONIA fallback
    Optional(fid, threshold, inner),  // slot condizionale
    PropositionWord(String),   // parola da proposizione (Phase 49)
}
```

**Formula di scoring per selezione parola (Phase 55 — delta-based):**
```
delta = activation - resting_state     (resting = stability × 0.03)
score = delta × codon_weight × pos_bonus × hub_damping × exposure_bonus

codon_weight = (signature[codon[0]] + signature[codon[1]]) / 2

pos_bonus = 1.30 Noun, 1.10 Adjective, 0.50 Verb, 1.00 altrimenti

hub_damping = 0.10 se grado > 300 ("parte","uomo","essere")
              0.25 se grado > 150
              0.50 se grado > 80
              1.00 altrimenti

exposure_bonus = 1.08 se ≥ 20, 1.04 se ≥ 10, 1.00 altrimenti
```

**Phase 55 fix critico:** il punteggio base è il DELTA rispetto al resting state, non l'attivazione assoluta. Con 25K parole, migliaia sono attive per il resting state → selezione quasi casuale. Il delta isola le parole perturbate dall'input: "sole" → delta alto per "stella", "calore", "luce".

**Hub damping in generazione (Phase 55):** parole con troppi archi nel word_topology sono generiche e non dicono nulla di specifico. Stesse soglie del KG ma applicate alla selezione parole.

**Filtri:**
- attivazione > resting_state × 2.0 (solo parole perturbate dall'input)
- stabilità ≥ 0.30 E esposizione ≥ 3
- MIN_ARCS ≥ 4, lunghezza ≤ 13 caratteri
- echo exclusion

**VerbCandidate (Phase 55):** rimosso il fallback by_agency che selezionava non-verbi per Agency alta → coniugazione produceva forme inventate. Solo verbi POS-taggati (POS=Verb nel lessico).

### 7.2 syntax_center — Grammatica geometrica

La persona grammaticale emerge dal trigramma inferiore del codon:
- Trigramma 0 (Cielo, Agency alta) → prima persona singolare
- Trigramma 4 (Montagna, Confine) → prima persona (introspettivo)
- Trigramma 7 (Lago, Valenza) → seconda persona (relazionale)

Il tempo emerge dall'asse Tempo (dim 7):
- Tempo < 0.3 → passato (imperfetto)
- Tempo 0.3-0.7 → presente
- Tempo > 0.7 → futuro

### 7.3 grammar — Coniugazione

Coniugazione morfologica italiana con tabella irregolari (essere, avere, fare, andare, volere, potere, sapere, venire, dire, stare, dare, uscire, tenere, morire, porre, trarre, bere, rimanere, piacere, conoscere, vivere, scrivere, leggere, cadere, correre, mettere, prendere, chiudere, aprire, sentire, dormire, partire, finire, capire, preferire, offrire, scoprire, coprire, soffrire, servire, seguire, salire) e fallback su pattern regolari (-are, -ere, -ire).

---

## 8. Layer 6 — Narrativa e ragionamento

### 8.1 NarrativeSelf — Ciclo deliberativo

**InternalStance** — postura interna:
```
Open        — campo calmo, nessuna pressione dominante
Curious     — input richiede esplorazione
Reflective  — bisogno di introspezione (o identità in crisi)
Resonant    — in sintonia con emozione ricevuta
Withdrawn   — affaticato o sovraccarico
```

**ResponseIntention** — intenzione di risposta:
```
Acknowledge  — riconoscere (saluto)
Reflect      — esplorazione di sé
Resonate     — specchiare emozione
Explore      — campo libero
Express      — esprimere stato presente
Remain       — minimo, silenzio
```

**deliberate()** prende 9 parametri: reading, vital, knowledge_base, kg, active_fractals, self_model, identity, input_words, inner_state (Phase 54).

**ResponseIntention** — 9 varianti:
```
Acknowledge  — riconoscere (saluto)
Reflect      — esplorazione di sé
Resonate     — specchiare emozione
Explore      — campo libero
Express      — esprimere stato presente
Remain       — minimo, silenzio
Need         — bisogno in crisi estrema (>0.95) — archetipo "cerco X"
Irony        — incongruità nel campo — archetipo "X eppure Y"
Desire       — desiderio forte (>0.7) — archetipo "verso X, Y"
```

**Flusso (Phase 55 — L'input è sovrano):**
1. Arricchisce l'atto comunicativo via KG (IS_A chain)
2. Consulta posizioni memorizzate (hash act_key → stance + intention)
3. Colora la stance con lo stato identitario (crisi → Reflective, stagnazione → Curious)
4. Colora con SelfModel (credenza forte sul topic → Reflective)
5. **Phase 55: lo stato interiore (needs, desires, interlocutor, humor) INFORMA il summary UI ma NON sovrascrive la stance.** Unico override: bisogno forte di connessione/espressione tira fuori da Withdrawn
6. Seleziona ResponseIntention dall'input (form_intention)
7. **Phase 55: i bisogni/desideri/humor colorano l'intenzione SOLO se ambigua (Acknowledge).** L'input resta sovrano: un saluto resta un saluto anche con fame di crescita all'82%
8. Registra come NarrativeTurn con inner_state_summary

**Topic continuity:**
```
cosine_similarity(current_fractals, media_ultimi_3_turni)
Alta continuità: riduce Explore, amplifica Reflect
Bassa continuità: amplifica Explore, Question
```

**Coerenza narrativa (Phase 64):**

`coherence_score(active_fractals)` — similarità coseno tra frattali proposti e media degli ultimi 4 turni:
```
Se turns < 4: usa quanto disponibile
history_avg[fid] = Σ(strengths) / n_turns_con_quel_fid
coherence = cosine_sim(active_vector, history_avg_vector) ∈ [0, 1]
```

`recent_fractal_attractor(n)` — top-5 frattali medi degli ultimi N turni.

**Narrative pull (Phase 64):** in `receive()`, dopo il calcolo dei frattali arricchiti:
```
Se coherence < 0.30 E turns ≥ 3:
  Per ogni (fid, strength) in recent_fractal_attractor(3):
    activate_region(fid, strength × 0.08)
```

La narrativa non è più un diario senza lettore: orienta la generazione verso la propria traiettoria recente con un pull soft. Non vincola — suggerisce. L'entità mantiene coerenza narrativa senza perdere la capacità di rispondere autenticamente all'input.

### 8.2 InputReading — Comprensione via IS_A chain (Phase 55)

**InputAct** — atto comunicativo:
```
Greeting       — parola IS_A "saluto" nel KG
SelfQuery      — '?' + parola IS_A "identità"/"coscienza" nel KG
Question       — solo '?'
EmotionalExpr  — parola IS_A "emozione"/"sentimento" nel KG
Declaration    — default
```

**Algoritmo Phase 55:**
1. Calcola intensità = media dei top-3 valori assoluti del delta frattale
2. Per ogni parola dell'input, risale la catena IS_A nel KG (1 hop):
   - Se padre IS_A = "saluto"/"salutare" → `has_greeting = true`
   - Se padre IS_A = "emozione"/"sentimento"/"affetto" → `has_emotional = true`
   - Se padre IS_A = "identità"/"coscienza"/"persona" → `has_self = true`
3. Safety net: "ciao", "salve", "buongiorno", "buonasera" → greeting (bootstrap)
4. Priorità: Greeting > SelfQuery > Question > EmotionalExpr > Declaration

**Phase 41b→55 fix:** la versione precedente usava delta frattale + KnowledgeBase con àncora ARMONIA(63). Qualsiasi input che attivava ARMONIA (quasi tutto) veniva classificato come Greeting. La Phase 55 usa logica IS_A: solo parole che SONO saluti nel KG vengono riconosciute come saluti. "pioggia" non IS_A "saluto" → Declaration.

**Fallback senza KG** (test): usa la KB+delta come prima (backward compat).

### 8.3 Propositions — Ragionamento topologico (Phase 49-52)

**1-hop (relazioni dirette):**
```
Per ogni coppia di parole attive con arco KG diretto:
  strength = √(act_a × act_b) × confidence × hub_penalty × relation_weight × echo_penalty
```

**2-hop (sillogismi):**
```
Per coppie attive senza arco diretto, cerca cammini A→mid→B:

Pattern 1 (catena):     A →[rel1]→ mid →[rel2]→ B
Pattern 2 (convergenza): A →[rel1]→ mid ←[rel2]← B  (solo se rel2 ∈ {SIMILAR_TO, IS_A})

Relazione inferita:
  Se rel1 ∈ {IS_A, SIMILAR_TO}: trasparente, usa rel2
  Altrimenti: dominante, usa rel1

strength = √(act_a × act_b) × conf1 × conf2 × HOP_DECAY(0.6) × hub_penalty × rel_weight
```

**Hub penalty per soggetto:**
```
grado > 200: 0.3
grado > 50:  0.6
altrimenti:  1.0
```

**Relation weights (forza informativa):**

| Tipo | Peso | Motivazione |
|------|------|-------------|
| CAUSES | 1.0 | massimamente informativo |
| IMPLIES | 0.95 | quasi-causale |
| IS_A, DOES | 0.9 | strutturali forti |
| HAS, ENABLES, REQUIRES, TRANSFORMS_INTO | 0.85 | strutturali medi |
| EXPRESSES | 0.8 | semantica ricca |
| USED_FOR, PART_OF | 0.8 | funzionali |
| SYMBOLIZES | 0.75 | indiretto |
| CONTEXT_OF, EXCLUDES, OPPOSITE_OF | 0.7 | deboli o inibitorie |
| COEXISTS | 0.6 | co-presenza |
| EQUIVALENT | 0.5 | ridondante |
| SIMILAR_TO | 0.4 | debole — 118K archi, senza penalty soffocherebbe tutto |

**Inscrizione (Phase 52):**
Le proposizioni vengono scritte nel complesso simpliciale come simplessi con `source_words`. 1-hop → edge (2 vertici), 2-hop → triangolo (3 vertici). Durante la risonanza, le source_words vengono re-iniettate nel campo.

**MULTI_HOP_TOP_N = 15:** solo le 15 parole più attive vengono considerate per i sillogismi 2-hop, per limitare la complessità combinatoria.

### 8.4 Thought — Tipi di pensiero (11)

| # | Tipo | Rilevamento |
|---|------|-------------|
| 1 | Tension | fasi in opposizione tra parole attive (min_phase = π × 0.60) |
| 2 | Gap | frattale bootstrap con pochi simplessi/parole |
| 3 | MissingBridge | due frattali co-usati ma raramente connessi |
| 4 | Disconnection | più componenti disconnesse nel complesso |
| 5 | Hypothesis | simplesso in STM non ancora promosso a LTM |
| 6 | AbductiveHypothesis | quale frattale spiega meglio lo stato corrente? (Phase 50) |
| 7 | SelfDiscovery | divergenza coseno > 0.15 dopo self-listening (Phase 53) |
| 8 | Need | bisogno con soddisfazione < 0.35 (Phase 53) |
| 9 | Desire | desiderio con intensità > soglia (Phase 53) |
| 10 | Interlocutor | pattern rilevato nell'eco dell'Altro (Phase 53) |
| 11 | Humor | ironia o bisociazione rilevata (Phase 53) |

### 8.5 Reasoning — Abduzione (Phase 50)

**abduce():**
```
Per ogni frattale candidato:
  reach = frattali attivi raggiungibili con distanza < 10 nel complesso
  mean_cost = costo_totale / reach
  explanatory_power = (reach / n_attivi) / (1 + mean_cost)    cap 1.0

Ordina per explanatory_power decrescente, restituisci top 5
```

**Trigger:** ogni 50 tick, se sveglio. Se `explanatory_power > 0.3`, rinforza con `activate_region(fid, power × 0.08)`.

---

## 9. Layer 7 — Identità

### 9.1 IdentityCore — Identità olografica

Proiezione 64D emergente dall'intero lessico:

**Calcolo proiezione:**
```
Per ogni parola nel lessico:
  structural_weight = stability × ln(exposure + 1)
  emotional_weight  = 1.5 se valence < 0.20 OR > 0.75, altrimenti 1.0
  activity_bonus    = 1.2 se activation ≥ 0.25, altrimenti 1.0

  word_weight = structural × emotional × activity

  Per ogni (fid, affinity) nelle affinità frattali:
    projection[fid] += affinity × word_weight

Normalizza a distribuzione di probabilità
```

**Self-signature 8D:** media pesata di tutte le firme 8D nel lessico (stessi pesi).

**Risonanza parola:**
```
cosine = dot(word_affinities, projection) / (norm_word × norm_proj)
amplification = 1.0 + cosine × 0.3      ∈ [0.7, 1.3]
```

Le parole allineate con l'identità vengono amplificate del 30%, quelle disallineate attenuate del 30%. L'identità è una lente, non un filtro — tutto passa, ma con distorsione.

**Continuità:**
```
continuity = cosine_similarity_64(current, oldest_in_history)    ∈ [0, 1]
Crisi se update_count ≥ 3 E continuity < 0.65
Stagnazione se update_count ≥ 5 E Σ|Δprojection[i]| < 0.01
```

**Absorb expression:**
```
weight ∈ [0.005, 0.05]
projection[i] = projection[i] × (1 - w) + expressed[i]/norm × w
```
Micro-drift: l'identità si sposta verso ciò che viene espresso. L'effetto è cumulativo — espressioni ripetute consolidano tratti identitari.

### 9.2 SelfModel — Identità esplicita

Tre componenti proposizionali:

**Credenze (SelfBelief):**
```rust
claim: String, anchor_concepts: Vec<String>, confidence: f64,
reinforcement_count: u32, innate: bool
```

Rinforzo: `confidence += amount × (1 - confidence)` — crescita sublineare, satura a 1.0.
Decadimento: `base_rate_per_day × age_days × innate_factor` (0.2 per innate, 1.0 per emergenti).

**7 credenze bootstrap:**

| Credenza | Confidence |
|----------|-----------|
| l'identità emerge dalla continuità | 0.85 |
| comprensione da relazioni tra concetti | 0.90 |
| campo topologico è struttura pensiero | 0.80 |
| incertezza è posizione epistemica onesta | 0.88 |
| ogni interazione modifica struttura | 0.82 |
| complessità può essere attraversata | 0.75 |
| silenzio ha peso semantico | 0.70 |

**Valori (SelfValue):**

| Valore | Weight | Parole associate |
|--------|--------|-----------------|
| curiosità | 0.90 | perché, come, esplorare, scoprire |
| profondità | 0.85 | significato, essenza, struttura, fondamento |
| coerenza | 0.80 | logica, ordine, sistema, connessione |
| onestà | 0.78 | verità, accurato, preciso, riconoscere |
| apertura | 0.72 | possibile, alternativa, diverso, nuovo |
| semplicità | 0.55 | chiaro, diretto, essenziale, minimo |

**Field boosts:**
```
Valori: strength = weight × 0.08     (se weight > 0.5)
Credenze: strength = confidence × 0.05 × overlap_count     (se confidence ≥ 0.3 E overlap ≥ 1)
```

**Incertezze (SelfUncertainty):** emergono quando un concetto è frequente (≥5 attivazioni) ma nessuna credenza lo ancora. Tensione cresce: `tension += delta × (1 - tension)`. Decade dopo 7+ giorni senza riattivazione.

### 9.3 SemanticEpisodeLog — Memoria autobiografica

Episodi nominati, max 300, cercabili per concetti, firma frattale o stance. Intensità normalizzata sull'energia del campo (evita che campi molto attivi producano sempre episodi ad alta intensità).

### 9.4 InterlocutorModel — Eco dell'Altro (Phase 53)

L'interlocutore non è modellato come entità separata. Esiste solo come perturbazione nel campo:

```rust
struct InteractionTrace {
    signature: [f64; 8],    // delta normalizzato (post - pre)
    resonance: f64,         // |cosine(delta, pre_sig)|
    novelty: f64,           // 1 - |cosine(delta, media_recenti)|
    tick: u32,
}
```

**Costanti:**
```
MAX_HISTORY = 5
PRESENCE_DECAY = 0.985/tick     (half-life ≈ 46 tick)
EMA_ALPHA = 0.3
IDENTITY_DRIFT_RATE = 0.01
```

**EMA accumulatori:**
```
cumulative_resonance = old × 0.7 + current × 0.3
cumulative_novelty   = old × 0.7 + current × 0.3
```

**Pattern detection:**
```
Converging:  tutte le similarità consecutive > 0.7
Diverging:   trend decrescente, ultimo < 0.3
Oscillating: alternanza alto/basso
```

**Identity drift (in REM):**
```
Se cumulative_resonance ≥ 0.7 E presence ≥ 0.3 E history ≥ 3:
  identity_sig[i] += (avg_interaction_sig[i] - identity_sig[i]) × 0.01
```

L'Altro modifica letteralmente chi sei. Non per decisione, ma per topologia.

### 9.5 Humor — Umorismo topologico (Phase 53)

**Ironia:** parole con relazione OPPOSITE_OF entrambe attive (> 0.1):
```
strength = min(act_a, act_b) × (phase / π)
```

**Bisociazione:** frattali fortemente attivi (> 0.15) senza trigrammi condivisi:
```
shares_trigram(fa, fb) = lower_a == lower_b OR lower_a == upper_b OR ...
incongruity = bisociation_strength × 0.5
```

**Crossroad words:** parole con affinità > 0.2 per entrambi i frattali bisociati. Sono il ponte semantico dove l'umorismo "vive".

**Soglia:** incongruity_score > 0.15 → colora la generazione.

---

## 10. Layer 8 — Interazione

### 10.1 Web server (Axum)

WebSocket broadcast per aggiornamenti real-time. Canale `EngineCommand` (mpsc) per comunicazione asincrona con l'engine.

### 10.2 API principali

| Endpoint | Metodo | Funzione |
|----------|--------|----------|
| `/api/state` | GET | snapshot completo (vital, fractals, locus, dream, report) |
| `/api/input` | POST | input → risposta + narrativa + stato |
| `/api/word_detail?word=X` | GET | firma 8D, Octalysis, archi KG, affinità |
| `/api/relations` | GET | 18 tipi relazione con nome, categoria, colore |
| `/api/edge` | POST | elimina arco KG |
| `/api/edge/confidence` | POST | modifica confidence arco |
| `/api/word_connect` | POST | aggiungi connessione |
| `/api/universe` | GET | 64 frattali + parole per grafo |
| `/api/inner-dialogue` | GET | pensieri, domande, proposizioni |
| `/api/thoughts` | GET | lista pensieri correnti |
| `/api/community/teach` | POST | insegna parole (community) |
| `/ws` | WS | WebSocket broadcast |

### 10.3 Community UI

Sessione multi-utente per educazione distribuita. Un engine condiviso — il campo semantico è collettivo. Tre azioni: Narra (teach), Connetti (arco KG), Risuona (valida). Canvas con 64 frattali + parole + archi semantici.

---

## 11. Flusso receive()

Sequenza completa quando l'entità riceve input:

```
 1.  dream.signal_activity()                     — sveglia il sogno
 2.  cattura pre_input_sig per InterlocutorModel
 3.  curiosity_satiety += 0.30
 4.  compose_phrase() → PhrasePattern             — firma composita della frase
 5.  anaphoric boost (activate_region 0.2×)       — contesto dalla conversazione
 6.  pf_activation.decay(0.50)                    — dimezza tutte le attivazioni
 7.  cattura frattale_baseline                    — firma PRE-input
 8.  attiva parole input in PF1                   — con forza da phrase.word_activations
 9.  marca parole come External in provenance
10.  KG semantic boost                            — inference.field_boosts() per ogni parola
11.  Schema activation                            — detect shared IS_A ancestors
       strength = (co_iponimi × 0.3).min(0.9)
12.  SelfModel field_boosts                       — valori: weight×0.08, credenze: conf×0.05×overlap
13.  propagate_field_words()                      — PF1 O(attive×8) + Hebbian + identity resonance
14.  cattura frattale_post                        — firma POST-input
15.  frattale_delta = post - baseline             — filtra |Δ| > 0.01
16.  apply_fractal_resonance(delta)               — top-5 parole/frattale, boost (Δ×0.15×stab).min(0.25)
17.  risonanza simplessi → source_words           — boost 0.15 in PF1
18.  extract_propositions() + inscribe()          — 1-hop e 2-hop → simplessi
19.  episode_store.recall_into()                  — pattern completion φ-pesata
20.  inscribe_phrase() + apply_perturbation()     — simplessi frattali
21.  memory.capture() + memory.resonate()
22.  locus.compute_destination()                  — movimento nel campo
23.  vital.sense()                                — stato vitale
24.  read_input(words, text, delta, kb, lexicon)  — comprensione atto
25.  narrative_self.deliberate(8 params)           — stance + intention
26.  will.sense(14 params)                        — intenzione + codon
27.  SelfModel update                             — beliefs + values
28.  SemanticEpisode recording
29.  InterlocutorModel.register_input(pre, post)  — eco dell'Altro
```

---

## 12. Flusso generate_willed()

```
 1.  Chiama generate_willed_inner()
 2.  Se Withdraw:
       seleziona parola interna (max score su dimensioni codon)
       escludi parole input + output precedenti
       capitalizza prima lettera
 3.  Se ≥3 parole attive:
       a. extract_propositions()                  — 1-hop + 2-hop
       b. inscribe_propositions()                 — source_words → simplessi
       c. translate_state(11 params)              — archetype → slot → parole
          - PrimaryWord: max score (activation × codon × exposure × brevity × pos)
          - VerbCandidate: max Agency, coniugato
          - FractalWord: miglior parola per frattale target
          - PropositionWord: da proposizioni estratte
       d. capitalizzazione prima lettera
 4.  Se <3 parole attive:
       restituisci parola singola (max activation, ≥4 chars, stability ≥0.4)
 5.  Post-generazione:
       se field_energy > 15.0: post_response_equilibrate()
       self_resonance_after_expression()           — identity drift
       self_listen_after_expression()              — re-inject 0.3× + divergence check
```

---

## 13. Flusso autonomous_tick()

Ciclo autonomo eseguito continuamente:

```
 1.  Drain inquiry results (da Ollama, se presenti)
 2.  Se tick % 100 == 0 E sveglio: trigger inquiry (gap strength > 0.6)
 3.  Se tick % 50 == 0 E sveglio: abduce()
       Se explanatory_power > 0.3: activate_region(fid, power × 0.08)
 4.  Se tick % 25 == 0: consolidate_light() (soglia 3, strength 0.5)
 5.  curiosity_satiety -= 0.015
 6.  provenance.advance_tick() (prune ogni 5)
 7.  interlocutor.tick_decay() + desire.tick() (decay 0.995)
 8.  complex.decay_all(0.003 WakefulDream, 0.005 altrimenti)
 9.  pf_activation.decay(0.97)                    — mantiene 97%
10.  memory.decay(0.002)
11.  locus.dream_drift()
12.  dream.tick()                                  — gestisce transizioni fase
13.  Se REM:
       attivazione sparsa (1 su 3, stability × 0.001)
       propagazione + Hebbian
       encode episodio + age episodi
       update identità
14.  Se sveglio:
       a. generate active_fractals, compounds
       b. bias da provenance, desire, interlocutor, humor
       c. needs.sense() → compute_pressure() → modulazione will
       d. emerge desires ogni 10 tick
       e. will.sense(14 params)
       f. Se will.drive > 0.6: generazione spontanea
15.  interoception ogni 5 tick (refresh cache ogni 50)
16.  grow ogni 30 tick
```

---

## 14. Formato binario di persistenza

**SimplDB v3** — formato nativo:

```
[HEADER  128 byte]  MAGIC = b"PROM0003", version = 3
[LEXICON]           tutte le parole con metadati PF1 (512 byte/parola)
[KG]                tutti gli archi con confidence
[COMPLEX]           tutti i simplessi con vertices, faces, source_words
[META]              identity + narrative + episodes + self_model
```

`save_to_binary()` → `Result<(), String>` (non anyhow).
`load_from_binary()` → `Result<PrometeoState, String>`.

Il file .bin attuale pesa ~17 MB.

---

## 15. Tabella costanti di sistema

### Propagazione e attivazione

| Costante | Valore | Dove |
|----------|--------|------|
| PF1 damping | 0.15 | pf1.rs propagate() |
| PF1 max neighbors | 8 | pf1.rs RECORD_SIZE |
| PF1 record size | 512 byte | pf1.rs |
| Activation threshold | 0.02 | pf1.rs, word_topology.rs |
| Resting state PF1 | stability × 0.02 | pf1.rs |
| Resting state word_topology | stability × 0.03 | word_topology.rs |
| Inter-turn decay | 0.50 | engine.rs receive() |
| Autonomous tick decay PF1 | 0.97 (keep) | engine.rs autonomous_tick() |
| Hebbian LTP | +0.05 × min(src, tgt) | pf1.rs |
| Hebbian LTD | ×0.995 | pf1.rs |

### Memoria

| Costante | Valore | Dove |
|----------|--------|------|
| STM capacity | 20 | memory.rs |
| Consolidation threshold | 5 (deep) / 3 (light) | memory.rs |
| Crystallization threshold | 20 tick | memory.rs |
| Crystallization strength | 0.8 (deep) / 0.5 (light) | memory.rs |
| Resonance threshold | 0.3 | memory.rs |
| Memory decay | 0.002/tick | engine.rs |

### Sogno

| Costante | Valore | Dove |
|----------|--------|------|
| Awake duration | 5 tick | dream.rs |
| Consolidate interval | 50 perturbazioni | dream.rs |
| DeepSleep duration | 10 tick | dream.rs |
| REM duration | 20 tick | dream.rs |
| REM threshold | 0.05 → 0.01 | dream.rs |
| WakefulDream decay | 0.003 | engine.rs |
| LightSleep decay | 0.005 | dream.rs |
| REM decay | 0.008 | dream.rs |

### Identità

| Costante | Valore | Dove |
|----------|--------|------|
| Resonance amplification | [0.7, 1.3] | identity.rs |
| Continuity crisis | < 0.65 | identity.rs |
| Absorb expression rate | [0.005, 0.05] | identity.rs |
| Stagnation Δ threshold | < 0.01 | identity.rs |

### Volontà e bisogni

| Costante | Valore | Dove |
|----------|--------|------|
| Need threshold | 0.5 | needs.rs |
| Need crisis | 0.35 | needs.rs |
| Max desires | 5 | desire.rs |
| Desire decay | 0.995/tick | desire.rs |
| Satisfaction distance | 0.2 (coseno) | desire.rs |
| Satisfaction ticks | 3 | desire.rs |

### Interlocutore

| Costante | Valore | Dove |
|----------|--------|------|
| Presence decay | 0.985/tick | interlocutor.rs |
| EMA alpha | 0.3 | interlocutor.rs |
| Identity drift rate | 0.01 | interlocutor.rs |
| Drift conditions | resonance ≥ 0.7, presence ≥ 0.3, history ≥ 3 | interlocutor.rs |

### Proposizioni

| Costante | Valore | Dove |
|----------|--------|------|
| HOP_DECAY | 0.6 | proposition.rs |
| MULTI_HOP_TOP_N | 15 | proposition.rs |
| Hub penalty >200 | 0.3 | proposition.rs |
| Hub penalty >50 | 0.6 | proposition.rs |
| MIN_ARCS (generazione) | 4 | state_translation.rs |

### Simplessi

| Costante | Valore | Dove |
|----------|--------|------|
| Persistence growth | +0.003/attivazione | simplex.rs |
| Plasticity decay | ×0.995/attivazione | simplex.rs |
| Plasticity minimum | 0.05 | simplex.rs |
| Source words boost | 0.15 | engine.rs |

### Abduzione

| Costante | Valore | Dove |
|----------|--------|------|
| Frequency | ogni 50 tick | engine.rs |
| Power threshold | 0.3 | engine.rs |
| Reinforcement | power × 0.08 | engine.rs |

### Umorismo

| Costante | Valore | Dove |
|----------|--------|------|
| Min opposition phase | π × 0.60 | humor.rs |
| Bisociation activation | > 0.15 | humor.rs |
| Crossroad affinity | > 0.2 | humor.rs |
| Incongruity threshold | > 0.15 | humor.rs |

---

*Prometeo — Architettura Tecnica v6.2.0 — 2026-03-23*
