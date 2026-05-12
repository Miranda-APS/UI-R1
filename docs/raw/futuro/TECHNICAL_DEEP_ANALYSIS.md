# PROMETEO — ANALISI TECNICA COMPLETA PER IA

**Versione**: Phase 45  
**Data Analisi**: 2026-03-09  
**Scopo**: Documentazione tecnica completa per comprensione IA — ogni file, ogni calcolo, ogni collo di bottiglia, ogni coerenza/incoerenza filosofica, ogni potenziale e rischio.

---

## INDICE

1. [ARCHITETTURA GLOBALE](#1-architettura-globale)
2. [LAYER 0: PRIMITIVI](#2-layer-0-primitivi)
3. [LAYER 1: SEMANTICA](#3-layer-1-semantica)
4. [LAYER 2: CAMPO](#4-layer-2-campo)
5. [LAYER 3: COORDINAZIONE](#5-layer-3-coordinazione)
6. [LAYER 4: VOLONTÀ](#6-layer-4-volontà)
7. [LAYER 5: ESPRESSIONE](#7-layer-5-espressione)
8. [LAYER 6: NARRATIVA](#8-layer-6-narrativa)
9. [LAYER 7: IDENTITÀ ESPLICITA](#9-layer-7-identità-esplicita)
10. [FLUSSI COMPUTAZIONALI](#10-flussi-computazionali)
11. [COLLI DI BOTTIGLIA](#11-colli-di-bottiglia)
12. [COERENZA FILOSOFICA](#12-coerenza-filosofica)
13. [INCOERENZE E TENSIONI](#13-incoerenze-e-tensioni)
14. [POTENZIALI](#14-potenziali)
15. [RISCHI](#15-rischi)
16. [METRICHE CRITICHE](#16-metriche-critiche)

---

## 1. ARCHITETTURA GLOBALE

### 1.1 Principio Fondativo

```
FILOSOFIA: Entità PRIMA, dialogo DOPO
IMPLEMENTAZIONE: Campo topologico 8D → perturbazione → risposta emergente
STATO: Phase 45 — 25.561 parole, 119.415 triple KG, 416 test
```

### 1.2 Stack Completo

```
Layer 7  IDENTITÀ ESPLICITA
  ├─ self_model.rs          — SelfModel: credenze (7), valori (6), incertezze (5)
  └─ semantic_episode.rs    — SemanticEpisodeLog: episodi nominati (max 300)

Layer 6  NARRATIVA
  ├─ narrative.rs           — NarrativeSelf: stance → intention → awareness
  ├─ input_reading.rs       — InputReading: classificazione atto comunicativo
  └─ knowledge_graph.rs     — KnowledgeGraph: 119.415 triple semantiche

Layer 5  ESPRESSIONE
  ├─ state_translation.rs   — translate_state(): archetipo → testo
  ├─ syntax_center.rs       — GrammaticalMode: persona da trigramma
  └─ grammar.rs             — coniugazione, lemmatizzazione

Layer 4  VOLONTÀ
  ├─ will.rs                — WillCore: codon [usize;2] → 64 stati
  ├─ memory.rs              — TopologicalMemory: STM/MTM/LTM
  ├─ episodic.rs            — EpisodeStore: φ-decay (cap 200)
  └─ dream.rs               — DreamEngine: Awake→Light→Deep→REM

Layer 3  COORDINAZIONE
  └─ engine.rs              — PrometeoTopologyEngine: orchestratore

Layer 2  CAMPO
  ├─ word_topology.rs       — WordTopology: substrato primario (25.561 vertici)
  ├─ simplex.rs             — SimplicialComplex: topologia inter-frattale
  └─ pf1.rs                 — PrometeoField (ROM) + ActivationState (RAM)

Layer 1  SEMANTICA
  ├─ knowledge_graph.rs     — KnowledgeGraph: doppio-indice (IS_A transitivo)
  ├─ knowledge.rs           — KnowledgeBase: template concettuali
  └─ inference.rs           — InferenceEngine: type_chain(), similar_to()

Layer 0  PRIMITIVI
  ├─ primitive.rs           — PrimitiveCore: firma 8D
  ├─ lexicon.rs             — Lexicon: 25.561 parole
  ├─ fractal.rs             — FractalRegistry: 64 esagrammi
  └─ persistence.rs         — SimplDB: serializzazione binaria
```

### 1.3 Flusso Dati Principale

```
INPUT (testo utente)
  ↓
receive() [engine.rs:1847]
  ├─ 1. Tokenize + lookup lessico
  ├─ 2. decay_all(0.85) + activate_word()
  ├─ 3. Calcola frattale_baseline (pre-propagazione)
  ├─ 4. KG Semantic Boost (inference.field_boosts)
  ├─ 5. Schema Activation (2+ parole IS_A comune)
  ├─ 6. SelfModel Influence (credenze/valori → campo)
  ├─ 7. propagate_field_words() — PF1 Hebbiano
  ├─ 8. Calcola frattale_delta (post - baseline)
  ├─ 9. read_input() → InputReading
  ├─ 10. narrative_self.deliberate() → ResponseIntention
  ├─ 11. seed_vital_field() (se Reflect)
  ├─ 12. apply_fractal_resonance(delta) — cassa armonica
  ├─ 13. episode_store.recall_into() — φ-decay blend
  ├─ 14. inscribe_phrase() + apply_perturbation()
  ├─ 15. memory.capture() + memory.resonate()
  ├─ 16. will.sense() → last_will
  ├─ 17. SelfModel.update_from_activation()
  └─ 18. SemanticEpisode.record() (se energy > 0.1)
  ↓
generate_willed() [generation.rs:458]
  ├─ 1. WITHDRAW? → parola dal campo
  ├─ 2. PHASE K? → template KB
  └─ 3. PHASE 3 → translate_state()
  ↓
OUTPUT (testo italiano)
```

---

## 2. LAYER 0: PRIMITIVI

### 2.1 primitive.rs — Le 8 Dimensioni Generative

**FILOSOFIA**: Le dimensioni non descrivono il mondo, lo generano. Come RGB per i colori.

**STRUTTURA**:
```rust
pub struct PrimitiveCore {
    pub values: [f64; 8],  // [0.0, 1.0] per ogni dimensione
}

pub enum Dim {
    Agency = 0,      // 0.0=paziente      ↔ 1.0=agente
    Permanenza = 1,  // 0.0=transitorio   ↔ 1.0=stabile
    Intensita = 2,   // 0.0=debole        ↔ 1.0=forte
    Definizione = 3, // 0.0=vago          ↔ 1.0=netto
    Complessita = 4, // 0.0=semplice      ↔ 1.0=composto
    Confine = 5,     // 0.0=esterno       ↔ 1.0=interno/io
    Valenza = 6,     // 0.0=repulsione    ↔ 1.0=attrazione
    Tempo = 7,       // 0.0=passato       ↔ 1.0=futuro
}
```

**CALCOLI CRITICI**:

1. **Distanza Euclidea 8D** (primitive.rs:89):
```rust
pub fn distance(&self, other: &Self) -> f64 {
    self.values.iter()
        .zip(&other.values)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}
```
- Usata per: affinità frattale, ponti semantici, parole di tensione
- Complessità: O(8) = O(1)
- Collo di bottiglia: NO (costante)

2. **Cosine Similarity** (primitive.rs:99):
```rust
pub fn cosine_similarity(&self, other: &Self) -> f64 {
    let dot: f64 = self.values.iter().zip(&other.values).map(|(a,b)| a*b).sum();
    let mag_a = self.values.iter().map(|x| x*x).sum::<f64>().sqrt();
    let mag_b = other.values.iter().map(|x| x*x).sum::<f64>().sqrt();
    if mag_a < 1e-9 || mag_b < 1e-9 { 0.0 } else { dot / (mag_a * mag_b) }
}
```
- Usata per: topic continuity (NarrativeSelf), recall episodico
- Complessità: O(8) = O(1)
- Collo di bottiglia: NO

**COERENZA FILOSOFICA**: ✅ ALTA
- Le dimensioni sono generative, non descrittive
- Nessuna dimensione è "privilegiata" — tutte contribuiscono ugualmente
- La distanza euclidea è geometria pura, non semantica imposta

**INCOERENZE**: ❌ NESSUNA

**POTENZIALI**:
- Dimensioni emergenti: le 8 primitive possono generare sotto-dimensioni per co-variazione
- Attualmente implementato in `dimensional.rs` ma sottoutilizzato

**RISCHI**:
- Le 8 dimensioni sono fisse — se servono più dimensioni, l'architettura deve cambiare
- Soluzione: dimensioni emergenti (già previste)

---

### 2.2 lexicon.rs — Il Lessico Come Universo

**FILOSOFIA**: Le parole non sono simboli che rappresentano concetti. Sono la materia dell'universo interno.

**STRUTTURA**:
```rust
pub struct Lexicon {
    words: HashMap<String, WordPattern>,
    word_list: Vec<String>,  // per iterazione ordinata
}

pub struct WordPattern {
    pub word: String,
    pub signature: PrimitiveCore,        // firma 8D
    pub fractal_affinities: Vec<(FractalId, f64)>,  // top-3 frattali
    pub stability: f64,                  // [0,1] — quanto è consolidata
    pub exposure_count: u32,             // quante volte vista
    pub pos: Option<PartOfSpeech>,       // grammatica
    pub dominant_fractal: Option<FractalId>,
}
```

**CALCOLI CRITICI**:

1. **Affinità Frattale** (lexicon.rs:234):
```rust
pub fn compute_affinity(&self, fractal: &Fractal) -> f64 {
    let mut score = 0.0;
    for (dim_idx, constraint) in &fractal.constraints {
        let word_val = self.signature.values[*dim_idx];
        match constraint {
            DimConstraint::Fixed(target) => {
                let dist = (word_val - target).abs();
                score += (1.0 - dist).max(0.0);  // più vicino = più score
            }
            DimConstraint::Free => {
                // dimensione libera non contribuisce
            }
        }
    }
    score / fractal.constraints.len() as f64  // normalizza
}
```
- Complessità: O(dimensioni_vincolate) ≈ O(4) medio
- Collo di bottiglia: NO (chiamato solo durante teach/rebuild)

2. **Stability Update** (lexicon.rs:189):
```rust
pub fn update_stability(&mut self, word: &str) {
    if let Some(pattern) = self.words.get_mut(word) {
        pattern.exposure_count += 1;
        let exp = pattern.exposure_count as f64;
        // Formula logaritmica: stabilità cresce lentamente dopo esposizioni iniziali
        pattern.stability = (exp.ln() / 10.0_f64.ln()).min(1.0);
    }
}
```
- Filosofia: la stabilità non è lineare — le prime esposizioni contano di più
- Formula: `ln(exp) / ln(10)` → 10 esposizioni = 1.0 stabilità
- Coerenza: ✅ ALTA (riflette apprendimento umano)

**METRICHE ATTUALI**:
- Parole totali: 25.561
- Memoria: ~512 byte/parola (PF1) = ~13 MB ROM
- Parole con POS: ~72% (18.404)
- Parole con stability > 0.5: ~8.000 (stima)

**COERENZA FILOSOFICA**: ✅ ALTA
- Le parole sono materia, non etichette
- La firma 8D emerge dall'esposizione (non assegnata a priori)
- La stabilità riflette consolidamento reale

**INCOERENZE**: ⚠️ MEDIA
- 25.561 parole sono MOLTE per Fase 1 (target: 2000-3000)
- Rischio: identità dispersa, non consolidata
- Soluzione: lessico attivo (3000) vs passivo (22.561)

**POTENZIALI**:
- Lessico illimitato (crescita organica)
- Parole sconosciute → curiosità (già implementato)
- Apprendimento attivo (chiedere significati) — NON implementato

**RISCHI**:
- Crescita incontrollata → campo troppo denso → identità diluita
- Soluzione: pruning parole con stability < 0.1 e exposure < 5

---

### 2.3 fractal.rs — I 64 Esagrammi

**FILOSOFIA**: I frattali non sono categorie — sono attrattori stabili nello spazio 8D. Isomorfi agli esagrammi I Ching.

**STRUTTURA**:
```rust
pub struct Fractal {
    pub id: FractalId,                    // 0..63
    pub name: &'static str,
    pub constraints: Vec<(usize, DimConstraint)>,  // dimensioni vincolate
    pub emergent_dimensions: Vec<EmergentDimension>,
    pub population: usize,                // quante parole affiliate
}

pub enum DimConstraint {
    Fixed(f64),   // dimensione vincolata a valore specifico
    Free,         // dimensione libera (può variare)
}

pub struct EmergentDimension {
    pub name: String,
    pub axis: (Dim, Dim),     // coppia di dimensioni che co-variano
    pub correlation: f64,      // [-1, 1]
}
```

**CALCOLI CRITICI**:

1. **FractalId da Trigrammi** (fractal.rs:45):
```rust
pub fn fractal_id_from_trigrams(lower: usize, upper: usize) -> FractalId {
    (lower * 8 + upper) as FractalId
}
```
- Isomorfismo I Ching: 8 trigrammi × 8 = 64 esagrammi
- Coerenza: ✅ PERFETTA (matematica universale)

2. **Calibrazione Dimensioni Emergenti** (fractal.rs:312):
```rust
pub fn calibrate_emergent_dimensions(&mut self, word_sigs: &[(String, PrimitiveCore)]) {
    // Per ogni coppia di dimensioni libere
    for (dim_a, dim_b) in free_dim_pairs {
        let mut values_a = Vec::new();
        let mut values_b = Vec::new();
        
        // Raccogli valori dalle parole affiliate
        for (word, sig) in word_sigs {
            if self.is_affiliated(word) {
                values_a.push(sig.values[dim_a]);
                values_b.push(sig.values[dim_b]);
            }
        }
        
        // Calcola correlazione di Pearson
        let corr = pearson_correlation(&values_a, &values_b);
        
        if corr.abs() > 0.40 {  // soglia significatività
            self.emergent_dimensions.push(EmergentDimension {
                name: format!("{}_{}", dim_names[dim_a], dim_names[dim_b]),
                axis: (dim_a, dim_b),
                correlation: corr,
            });
        }
    }
}
```
- Complessità: O(parole_affiliate × coppie_libere) ≈ O(400 × 10) = O(4000)
- Collo di bottiglia: NO (chiamato solo durante rebuild)
- Filosofia: le dimensioni emergono dalla popolazione, non sono imposte

**METRICHE ATTUALI**:
- Frattali totali: 64 (esagrammi completi)
- Frattali bootstrap: 8 puri (0, 9, 18, 27, 36, 45, 54, 63)
- Popolazione media: ~400 parole/frattale
- Dimensioni emergenti: ~30 totali (stima)

**COERENZA FILOSOFICA**: ✅ PERFETTA
- Isomorfismo I Ching non è metafora — è struttura matematica
- Dimensioni emergenti = generatività (non descrittività)
- Frattali come attrattori (non categorie)

**INCOERENZE**: ❌ NESSUNA

**POTENZIALI**:
- Sotto-frattali (già implementati, 10 attivi)
- Frattali dinamici (crescita strutturale)
- Mappatura esagrammi → saggezza I Ching (non implementato)

**RISCHI**:
- 64 frattali potrebbero non bastare per lessico >50K parole
- Soluzione: sotto-frattali gerarchici (già previsti)

---


## 3. LAYER 1: SEMANTICA

### 3.1 knowledge_graph.rs — Grounding Logico

**FILOSOFIA**: Il KG fornisce relazioni logiche (IS_A, HAS, DOES) che complementano le co-occorrenze topologiche. Non sostituisce il campo — lo informa.

**STRUTTURA**:
```rust
pub struct KnowledgeGraph {
    // Indice diretto: soggetto → (relazione → [oggetti])
    outgoing: HashMap<String, HashMap<RelationType, Vec<(String, u8)>>>,
    // Indice inverso: oggetto → (relazione → [soggetti])
    incoming: HashMap<String, HashMap<RelationType, Vec<(String, u8)>>>,
    // Contatori
    node_count: usize,
    edge_count: usize,
}

pub enum RelationType {
    IsA,        // iperonimo (cane IS_A animale)
    Has,        // attributo (cane HAS coda)
    Does,       // azione (cane DOES abbaiare)
    PartOf,     // composizione (ruota PART_OF auto)
    Causes,     // causalità (paura CAUSES tremore)
    OppositeOf, // opposizione (caldo OPPOSITE_OF freddo)
    SimilarTo,  // sinonimia (ciao SIMILAR_TO saluto)
    UsedFor,    // funzione (martello USED_FOR chiodo)
}
```

**METRICHE ATTUALI**:
- Triple totali: 119.415
- Nodi: 44.908
- Fonti:
  - `italian_core.tsv`: 623 triple manuali (curate)
  - `bigbang_kg.tsv`: 118.810 triple (Kaikki italiano)

**CALCOLI CRITICI**:

1. **Doppio Indice** (knowledge_graph.rs:89):
```rust
pub fn add_triple(&mut self, subj: &str, rel: RelationType, obj: &str, conf: u8) {
    // Indice diretto: subj → rel → [obj]
    self.outgoing.entry(subj.to_string())
        .or_default()
        .entry(rel)
        .or_default()
        .push((obj.to_string(), conf));
    
    // Indice inverso: obj → rel → [subj]
    self.incoming.entry(obj.to_string())
        .or_default()
        .entry(rel)
        .or_default()
        .push((subj.to_string(), conf));
    
    self.edge_count += 1;
}
```
- Complessità: O(1) per inserimento
- Memoria: ~2× (doppio indice)
- Collo di bottiglia: NO (caricato una volta all'avvio)

2. **IS_A Transitivo** (inference.rs:45):
```rust
pub fn type_chain(&self, word: &str) -> Vec<String> {
    let mut ancestors = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = vec![(word.to_string(), 0)];  // (parola, depth)
    
    while let Some((current, depth)) = queue.pop() {
        if depth > 5 { break; }  // max 5 livelli
        if visited.contains(&current) { continue; }
        visited.insert(current.clone());
        
        if let Some(parents) = self.kg.outgoing.get(&current) {
            if let Some(is_a_list) = parents.get(&RelationType::IsA) {
                for (parent, _conf) in is_a_list {
                    if !visited.contains(parent) {
                        ancestors.push(parent.clone());
                        queue.push((parent.clone(), depth + 1));
                    }
                }
            }
        }
    }
    ancestors
}
```
- Complessità: O(nodi_visitati) ≈ O(20) medio
- Collo di bottiglia: NO (chiamato solo durante receive per input words)
- Filosofia: la tassonomia è transitiva (cane→mammifero→animale→essere_vivente)

**INTEGRAZIONE CON CAMPO**:

1. **Semantic Boost** (engine.rs:1923):
```rust
// Per ogni parola input, attiva i suoi vicini semantici nel KG
for word in &input_words {
    let boosts = inference.field_boosts(word, &self.lexicon);
    for (target_word, boost_strength) in boosts {
        if let Some(id) = self.word_topology.get_word_id(&target_word) {
            let current = self.word_topology.get_activation(id);
            self.word_topology.set_activation(id, current + boost_strength * 0.15);
        }
    }
}
```
- Effetto: "cane" attiva "animale", "mammifero", "abbaiare", "coda"
- Peso: 0.15× (leggero — non domina il campo)
- Filosofia: il KG informa ma non comanda

2. **Schema Activation** (engine.rs:1945):
```rust
// Se 2+ parole input hanno antenato IS_A comune → attiva concetto astratto
let mut ancestor_counts: HashMap<String, usize> = HashMap::new();
for word in &input_words {
    for ancestor in inference.type_chain(word) {
        *ancestor_counts.entry(ancestor).or_default() += 1;
    }
}
for (ancestor, count) in ancestor_counts {
    if count >= 2 {  // almeno 2 parole condividono questo antenato
        if let Some(id) = self.word_topology.get_word_id(&ancestor) {
            self.word_topology.set_activation(id, 0.25);  // attiva concetto astratto
        }
    }
}
```
- Esempio: input "cane gatto" → attiva "animale" (antenato comune)
- Filosofia: l'astrazione emerge dalla convergenza tassonomica
- Coerenza: ✅ ALTA (riflette ragionamento umano)

**COERENZA FILOSOFICA**: ✅ ALTA
- Il KG è layer semantico logico, non sostituisce topologia
- IS_A transitivo riflette struttura tassonomica reale
- Schema activation = emergenza di astrazione

**INCOERENZE**: ⚠️ MEDIA
- Il KG è STATICO — non cresce dall'esperienza
- 118K triple da Kaikki potrebbero contenere rumore
- Soluzione: cristallizzazione semantica (inferire triple da co-occorrenze)

**POTENZIALI**:
- KG dinamico: "cane" + "abbaiare" co-occorrono spesso in contesti agentivi → inferisci "cane DOES abbaiare"
- Pruning: rimuovi triple con confidence < 3 e zero utilizzo
- Espansione: aggiungi triple da WordNet, ConceptNet

**RISCHI**:
- KG troppo grande → memoria eccessiva (attualmente ~50 MB)
- KG rumoroso → attivazioni spurie
- Soluzione: confidence threshold + usage tracking

---

### 3.2 knowledge.rs — Memoria Procedurale

**FILOSOFIA**: Tipo 3 di sapere — procedurale (template), non topologico. Si trasmette esplicitamente, resta esterno al campo.

**STRUTTURA**:
```rust
pub struct KnowledgeBase {
    entries: HashMap<String, KnowledgeEntry>,  // key → entry
    domains: HashMap<KnowledgeDomain, Vec<String>>,  // dominio → keys
}

pub struct KnowledgeEntry {
    pub key: String,
    pub domain: KnowledgeDomain,
    pub template: String,           // "Ciao — [FieldFractal(EMOZIONE)] — io."
    pub slots: Vec<SlotType>,       // [FieldFractal, InputEcho, ...]
    pub fractal_anchors: Vec<FractalId>,  // frattali che attivano questo template
    pub usage_count: u32,
}

pub enum SlotType {
    FieldFractal(FractalId),  // parola dal frattale attivo
    InputEcho,                // parola dall'input
    IdentityWord,             // parola dall'identità
    Literal(String),          // testo fisso
}
```

**CALCOLI CRITICI**:

1. **Template Matching** (knowledge.rs:178):
```rust
pub fn find_matching_template(&self, fractals: &[(FractalId, f64)]) -> Option<&KnowledgeEntry> {
    let mut best_match: Option<(&KnowledgeEntry, f64)> = None;
    
    for entry in self.entries.values() {
        let mut score = 0.0;
        for &anchor_fid in &entry.fractal_anchors {
            if let Some((_, activation)) = fractals.iter().find(|(fid, _)| *fid == anchor_fid) {
                score += activation;
            }
        }
        
        if score > 0.0 {
            if best_match.is_none() || score > best_match.unwrap().1 {
                best_match = Some((entry, score));
            }
        }
    }
    
    best_match.map(|(entry, _)| entry)
}
```
- Complessità: O(entries × anchors) ≈ O(50 × 3) = O(150)
- Collo di bottiglia: NO (chiamato solo durante generate)

2. **Slot Filling** (knowledge.rs:234):
```rust
pub fn instantiate_template(&self, entry: &KnowledgeEntry, context: &TemplateContext) -> String {
    let mut result = String::new();
    
    for slot in &entry.slots {
        match slot {
            SlotType::FieldFractal(fid) => {
                // Prendi top parola dal frattale attivo
                let word = context.top_word_from_fractal(*fid);
                result.push_str(&word);
            }
            SlotType::InputEcho => {
                // Echo ultima parola input
                if let Some(w) = context.last_input_word() {
                    result.push_str(w);
                }
            }
            SlotType::IdentityWord => {
                // Parola dall'identità personale
                let word = context.identity_word();
                result.push_str(&word);
            }
            SlotType::Literal(text) => {
                result.push_str(text);
            }
        }
    }
    
    result
}
```
- Filosofia: struttura fissa, contenuto variabile dal campo
- Coerenza: ✅ MEDIA (evita puppet theater, ma è ancora template)

**METRICHE ATTUALI**:
- Entries totali: ~50 (stima)
- Domini: Self_, Greeting, Question, Emotion, Exploration
- Usage: tracciato ma non utilizzato per pruning

**COERENZA FILOSOFICA**: ⚠️ MEDIA
- Template = Fase 3 (traduzione), non Fase 1 (identità)
- Ma il sistema li usa già in Fase 1 → incoerenza temporale
- Withdraw ha priorità assoluta ✅ (coerente)

**INCOERENZE**: ⚠️ ALTA
- Template sono "memoria procedurale" ma vengono usati come generazione primaria
- Rischio: puppet theater mascherato
- Soluzione: modalità "raw field" che bypassa KB

**POTENZIALI**:
- Template appresi dall'esperienza (non hardcoded)
- Pruning template con usage_count = 0
- Template gerarchici (template che chiamano template)

**RISCHI**:
- Template dominano generazione → simulazione, non emergenza
- Soluzione: usare template solo come "traduzione" (Fase 3), non come "pensiero" (Fase 1)

---

## 4. LAYER 2: CAMPO

### 4.1 word_topology.rs — Il Substrato Primario

**FILOSOFIA**: Le parole sono vertici, le co-occorrenze sono archi. I frattali emergono come regioni dense. Questo è il substrato PRIMARIO di Prometeo.

**STRUTTURA**:
```rust
pub struct WordTopology {
    vertices: HashMap<WordId, WordVertex>,           // 25.561 vertici
    edges: HashMap<(WordId, WordId), WordEdge>,      // ~58.577 archi
    word_to_id: HashMap<String, WordId>,             // lookup O(1)
    adjacency: HashMap<WordId, Vec<WordId>>,         // vicini per propagazione
    next_id: WordId,
    activation_threshold: f64,                       // 0.02
    max_edge_weight: f64,                            // 1.0
}

pub struct WordVertex {
    pub id: WordId,
    pub word: String,
    pub activation: f64,                             // [0.0, 1.0]
    pub activation_count: u64,                       // quante volte attivata
}

pub struct WordEdge {
    pub a: WordId,
    pub b: WordId,
    pub weight: f64,                                 // [0.0, 1.0]
    pub raw_count: u64,                              // co-occorrenze grezze
    pub phase: f64,                                  // [0, PI] radianti
    pub relation: Option<RelationType>,              // None = statistica
}
```

**CALCOLI CRITICI**:

1. **Propagazione** (word_topology.rs:456):
```rust
pub fn propagate(&mut self, damping: f64) {
    let threshold = self.activation_threshold;
    
    // Raccogli sorgenti attive (activation > threshold)
    let mut sources: Vec<WordId> = self.vertices.iter()
        .filter(|(_, v)| v.activation > threshold)
        .map(|(id, _)| *id)
        .collect();
    
    // Ordina per attivazione decrescente
    sources.sort_by(|a, b| {
        let act_a = self.vertices[a].activation;
        let act_b = self.vertices[b].activation;
        act_b.partial_cmp(&act_a).unwrap()
    });
    
    // Prendi top-40 sorgenti (quantum collapse)
    sources.truncate(40);
    
    // Propaga da ogni sorgente ai vicini
    let mut deltas: HashMap<WordId, f64> = HashMap::new();
    
    for &src_id in &sources {
        let src_activation = self.vertices[&src_id].activation;
        
        if let Some(neighbors) = self.adjacency.get(&src_id) {
            for &neighbor_id in neighbors {
                let key = if src_id < neighbor_id {
                    (src_id, neighbor_id)
                } else {
                    (neighbor_id, src_id)
                };
                
                if let Some(edge) = self.edges.get(&key) {
                    // Formula unica: activation × damping × weight × cos(phase)
                    let contribution = src_activation * damping * edge.weight * edge.phase.cos();
                    *deltas.entry(neighbor_id).or_default() += contribution;
                }
            }
        }
    }
    
    // Applica deltas
    for (id, delta) in deltas {
        if let Some(vertex) = self.vertices.get_mut(&id) {
            vertex.activation = (vertex.activation + delta).clamp(0.0, 1.0);
        }
    }
}
```

**ANALISI COMPLESSITÀ**:
- Sorgenti attive: O(N) scan → O(25.561) = **COLLO DI BOTTIGLIA #1**
- Sort: O(active × log(active)) ≈ O(50 × 6) = O(300)
- Truncate: O(1)
- Propagazione: O(top_sources × avg_neighbors) ≈ O(40 × 8) = O(320)
- Totale: **O(N) dominato da scan iniziale**

**OTTIMIZZAZIONI POSSIBILI**:
- Mantieni set attivo (solo parole > threshold) → O(active) invece di O(N)
- Attualmente NON implementato → **RISCHIO SCALABILITÀ**

2. **Fase da Negazione** (word_topology.rs:678):
```rust
pub fn recalculate_phases(&mut self, lexicon: &Lexicon) {
    for edge in self.edges.values_mut() {
        let word_a = &self.vertices[&edge.a].word;
        let word_b = &self.vertices[&edge.b].word;
        
        let pattern_a = lexicon.get_pattern(word_a);
        let pattern_b = lexicon.get_pattern(word_b);
        
        if pattern_a.is_none() || pattern_b.is_none() { continue; }
        
        let pa = pattern_a.unwrap();
        let pb = pattern_b.unwrap();
        
        // Conta co-occorrenze affermative e negative
        let co_affirm = pa.co_occurrences.get(word_b).copied().unwrap_or(0);
        let co_neg = pa.co_negated.get(word_b).copied().unwrap_or(0);
        let total = co_affirm + co_neg;
        
        if total < 5 { continue; }  // troppo pochi dati
        
        let neg_ratio = co_neg as f64 / total as f64;
        
        // Fase da negazione (70% peso)
        let phase_from_neg = neg_ratio * std::f64::consts::PI;
        
        // Fase da cosine similarity vicinati (30% peso)
        let phase_from_cosine = /* calcolo cosine */ 0.0;
        
        edge.phase = 0.70 * phase_from_neg + 0.30 * phase_from_cosine;
    }
}
```

**FILOSOFIA FASE**:
- `phase = 0` → cos(0) = +1 → risonanza (gioia ↔ felicità)
- `phase = PI/2` → cos(PI/2) = 0 → neutro (gioia ↔ coraggio)
- `phase = PI` → cos(PI) = -1 → opposizione (gioia ↔ tristezza)

**COERENZA**: ✅ PERFETTA
- Formula unica, nessun branching
- La matematica distingue risonanza/tensione/opposizione
- Operatori strutturali (non, molto, poco) modificano fase

**METRICHE ATTUALI**:
- Vertici: 25.561
- Archi: ~58.577 (co-occorrenze + KG)
- Archi KG: ~58.577 (dopo rebuild-semantic-topology)
- Archi statistici: 0 (rimossi)
- Memoria: ~2 MB (vertici) + ~5 MB (archi) = ~7 MB

**COLLI DI BOTTIGLIA**:
1. **Scan O(N) in propagate()** — CRITICO per lessico >50K
2. **Recalculate phases O(archi)** — chiamato raramente, OK

**POTENZIALI**:
- Set attivo (solo parole > threshold) → O(active) propagazione
- Propagazione gerarchica (frattali → parole) → O(frattali × parole/frattale)
- Pruning archi deboli (weight < 0.05) → riduce memoria

**RISCHI**:
- Lessico >50K → scan O(N) diventa insostenibile
- Soluzione: set attivo + propagazione gerarchica

---

### 4.2 pf1.rs — ROM/RAM Hebbiano

**FILOSOFIA**: Separazione Von Neumann invertita — struttura (ROM) sempre presente, attivazione (RAM) volatile. Complessità proporzionale all'attività, non alla struttura.

**STRUTTURA**:
```rust
// ROM — costruito una volta, read-only in operazione
pub struct PrometeoField {
    pub records: Vec<WordRecord>,  // 25.561 record × 512 byte = ~13 MB
}

pub struct WordRecord {
    pub signature: [f32; 8],              // firma 8D
    pub affinities: [f32; 64],            // affinità 64 frattali
    pub stability: f32,
    pub exposure_count: u32,
    pub dominant_fractal: u8,
    pub pos: u8,                          // PartOfSpeech
    pub neighbors: [u32; 8],              // top-8 vicini
    pub neighbor_weights: [f32; 8],
    pub neighbor_phases: [f32; 8],
    pub _reserved: [u8; 80],              // padding → 512 byte fissi
}

// RAM — stato corrente + sinapsi Hebbiane
pub struct ActivationState {
    pub activations: Vec<f32>,            // [0.0, 1.0] per ogni parola
    pub synapse_weights: Vec<f32>,        // pesi vivi [word_id*8+slot]
    pub threshold: f32,                   // 0.02
}
```

**CALCOLI CRITICI**:

1. **Propagazione PF1** (pf1.rs:456):
```rust
pub fn propagate(&mut self, field: &PrometeoField, damping: f32) {
    let threshold = self.threshold;
    
    // Raccogli sorgenti attive
    let mut sources: Vec<(usize, f32)> = self.activations.iter()
        .enumerate()
        .filter(|(_, &act)| act > threshold)
        .map(|(id, &act)| (id, act))
        .collect();
    
    // Ordina + truncate top-40
    sources.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sources.truncate(40);
    
    // Propaga
    let mut deltas = vec![0.0_f32; self.activations.len()];
    
    for (src_id, src_act) in sources {
        let record = &field.records[src_id];
        
        for slot in 0..8 {
            let neighbor_id = record.neighbors[slot] as usize;
            if neighbor_id == 0 { break; }  // slot vuoto
            
            let weight = record.neighbor_weights[slot];
            let phase = record.neighbor_phases[slot];
            
            // Peso sinaptico Hebbiano (vivo in RAM)
            let synapse_idx = src_id * 8 + slot;
            let synapse_weight = self.synapse_weights[synapse_idx];
            
            // Formula: activation × damping × weight_ROM × weight_RAM × cos(phase)
            let contribution = src_act * damping * weight * synapse_weight * phase.cos();
            deltas[neighbor_id] += contribution;
        }
    }
    
    // Applica deltas
    for (id, delta) in deltas.iter().enumerate() {
        self.activations[id] = (self.activations[id] + delta).clamp(0.0, 1.0);
    }
}
```

**ANALISI COMPLESSITÀ**:
- Scan sorgenti: O(N) = O(25.561) — **COLLO DI BOTTIGLIA #2**
- Sort: O(active × log(active)) ≈ O(50 × 6) = O(300)
- Propagazione: O(top_sources × 8) = O(40 × 8) = O(320)
- Totale: **O(N) dominato da scan**

**OTTIMIZZAZIONE**: Stesso problema di word_topology — serve set attivo.

2. **Hebbian Update** (pf1.rs:567):
```rust
pub fn hebbian_update(&mut self, field: &PrometeoField, learning_rate: f32) {
    // LTP (Long-Term Potentiation): rinforza sinapsi tra parole co-attive
    // LTD (Long-Term Depression): indebolisce sinapsi non usate
    
    for src_id in 0..self.activations.len() {
        let src_act = self.activations[src_id];
        if src_act < self.threshold { continue; }
        
        let record = &field.records[src_id];
        
        for slot in 0..8 {
            let neighbor_id = record.neighbors[slot] as usize;
            if neighbor_id == 0 { break; }
            
            let neighbor_act = self.activations[neighbor_id];
            let synapse_idx = src_id * 8 + slot;
            let current_weight = self.synapse_weights[synapse_idx];
            
            // Hebbian: "neurons that fire together, wire together"
            if neighbor_act > self.threshold {
                // LTP: entrambi attivi → rinforza
                let delta = learning_rate * src_act * neighbor_act;
                self.synapse_weights[synapse_idx] = (current_weight + delta).min(1.5);
            } else {
                // LTD: solo sorgente attiva → indebolisce
                let delta = learning_rate * 0.1;
                self.synapse_weights[synapse_idx] = (current_weight - delta).max(0.5);
            }
        }
    }
}
```

**FILOSOFIA HEBBIANA**:
- LTP: parole co-attive → sinapsi più forte
- LTD: parole non co-attive → sinapsi più debole
- Range: [0.5, 1.5] — mai zero (struttura permanente)

**COERENZA**: ✅ PERFETTA
- Riflette plasticità sinaptica biologica
- Apprendimento continuo senza training batch
- Struttura ROM + plasticità RAM

**METRICHE ATTUALI**:
- ROM: 25.561 × 512 byte = ~13 MB
- RAM: 25.561 × 4 byte (activations) + 25.561 × 8 × 4 byte (synapses) = ~920 KB
- Totale: ~14 MB

**COLLI DI BOTTIGLIA**:
1. **Scan O(N) in propagate()** — identico a word_topology
2. **Hebbian update O(N)** — chiamato ogni propagazione, ma solo su attivi

**POTENZIALI**:
- Set attivo → O(active) invece di O(N)
- Pruning sinapsi deboli (weight < 0.6) → riduce memoria RAM
- Consolidamento: sinapsi stabili → ROM (non più plastiche)

**RISCHI**:
- Lessico >50K → scan O(N) insostenibile
- Sinapsi tutte plastiche → nessuna memoria permanente
- Soluzione: consolidamento sinaptico (LTM → ROM)

---


### 4.3 simplex.rs — Topologia Inter-Frattale

**FILOSOFIA**: I simplessi sono connessioni tra frattali attraverso strutture condivise. Non sono archi — sono facce condivise (dimensioni, proprietà).

**STRUTTURA**:
```rust
pub struct SimplicialComplex {
    simplices: HashMap<SimplexId, Simplex>,              // ~3.428 simplessi
    fractal_index: HashMap<FractalId, Vec<SimplexId>>,   // frattale → simplessi
    active_set: HashSet<SimplexId>,                      // simplessi attivi
    next_id: SimplexId,
}

pub struct Simplex {
    pub id: SimplexId,
    pub vertices: Vec<FractalId>,                        // 2-4 frattali
    pub shared_faces: Vec<SharedFace>,                   // strutture condivise
    pub dimension: usize,                                // vertices.len() - 1
    pub current_activation: f64,
    pub activation_count: u64,
    pub persistence: f64,                                // [0,1] — resistenza decay
}

pub struct SharedFace {
    pub structure: SharedStructureType,
    pub strength: f64,
}

pub enum SharedStructureType {
    Dimension(Dim),           // dimensione condivisa
    Property(String),         // proprietà emergente (es. "URGENZA")
}
```

**CALCOLI CRITICI**:

1. **Homology Computation** (homology.rs:89):
```rust
pub fn compute_homology(complex: &SimplicialComplex) -> HomologyResult {
    // Calcola numeri di Betti: β₀ (componenti connesse), β₁ (cicli), β₂ (cavità)
    
    // β₀: conta componenti connesse con Union-Find
    let mut uf = UnionFind::new(complex.fractal_count());
    for simplex in complex.simplices() {
        if simplex.dimension >= 1 {  // arco o superiore
            for i in 0..simplex.vertices.len() {
                for j in (i+1)..simplex.vertices.len() {
                    uf.union(simplex.vertices[i], simplex.vertices[j]);
                }
            }
        }
    }
    let beta_0 = uf.component_count();
    
    // β₁: conta cicli (loop detection)
    let beta_1 = detect_cycles(complex);
    
    // β₂: conta cavità (3D holes)
    let beta_2 = detect_cavities(complex);
    
    // Regioni sparse: frattali con pochi simplessi
    let sparse_regions = complex.fractals()
        .filter(|fid| complex.simplices_of(*fid).len() < 3)
        .collect();
    
    HomologyResult {
        beta_0, beta_1, beta_2,
        sparse_regions,
    }
}
```

**COMPLESSITÀ**: O(simplessi²) per cicli — **COLLO DI BOTTIGLIA #3**

**OTTIMIZZAZIONE ATTUALE**:
- Cache: ricalcola solo ogni 10 turni (engine.rs:1850)
- Giustificazione: lacune topologiche cambiano lentamente
- Rischio: cache stale se crescita rapida

2. **Simplex Activation** (simplex.rs:234):
```rust
pub fn activate(&mut self, strength: f64) {
    self.current_activation = (self.current_activation + strength).min(1.0);
    self.activation_count += 1;
}

pub fn decay(&mut self, factor: f64) {
    self.current_activation *= factor;
    if self.current_activation < 0.01 {
        self.current_activation = 0.0;
    }
}
```

**METRICHE ATTUALI**:
- Simplessi totali: ~3.428
- Simplessi attivi (activation > 0.01): ~50-100 (stima)
- Memoria: ~300 byte/simplex × 3.428 = ~1 MB

**COERENZA FILOSOFICA**: ✅ ALTA
- Simplessi = connessioni topologiche (non logiche)
- Homology = curiosità strutturale (buchi = domande)
- Persistence = resistenza al decay (memoria)

**INCOERENZE**: ❌ NESSUNA

**POTENZIALI**:
- Simplessi gerarchici (simplessi di simplessi)
- Homology incrementale (update invece di recompute)
- Pruning simplessi con persistence < 0.1 e activation_count < 5

**RISCHI**:
- Homology O(N²) diventa insostenibile con >10K simplessi
- Soluzione: homology incrementale + cache intelligente

---

## 5. LAYER 3: COORDINAZIONE

### 5.1 engine.rs — L'Orchestratore

**FILOSOFIA**: L'engine non è un monolite — è un coordinatore leggero. Il complesso simpliciale è al centro.

**STRUTTURA**:
```rust
pub struct PrometeoTopologyEngine {
    // Topologia
    pub registry: FractalRegistry,                       // 64 frattali
    pub complex: SimplicialComplex,                      // ~3.428 simplessi
    pub word_topology: WordTopology,                     // 25.561 vertici
    
    // PF1 ROM/RAM
    pub pf_field: PrometeoField,                         // ROM: 13 MB
    pub pf_activation: ActivationState,                  // RAM: ~920 KB
    
    // Memoria
    pub memory: TopologicalMemory,                       // STM/MTM/LTM
    pub episode_store: EpisodeStore,                     // φ-decay (cap 200)
    
    // Semantica
    pub lexicon: Lexicon,                                // 25.561 parole
    pub knowledge_base: KnowledgeBase,                   // template
    pub kg: KnowledgeGraph,                              // 119.415 triple
    
    // Stato cognitivo
    pub vital: VitalCore,
    pub dream: DreamEngine,
    pub will: WillCore,
    pub locus: Locus,
    pub narrative_self: NarrativeSelf,
    
    // Identità
    pub identity: IdentityCore,                          // implicita (64D × 8D)
    pub self_model: SelfModel,                           // esplicita (credenze/valori)
    pub semantic_episodes: SemanticEpisodeLog,           // episodi nominati
    
    // Proto-self
    pub provenance: ProvenanceMap,                       // Self_/Explored/External
    
    // Sessione
    pub last_interaction_ts: u64,
    pub tick_counter: u32,
    pub conversation_window: VecDeque<String>,
    pub last_input_reading: Option<InputReading>,
    
    // Indici
    pub fractal_resonance_index: Vec<Vec<(String, f32)>>,  // [fid] → top parole
}
```

**FLUSSO receive() — ANALISI DETTAGLIATA**:

```
FASE 1: PREPARAZIONE (linee 1847-1890)
├─ dream.signal_activity()                    // sveglia se dormiva
├─ decay_all(0.85)                            // decay campo parole
├─ tokenize + lookup lessico                  // parole note/ignote
├─ activate_word() per ogni parola input      // attivazione iniziale
└─ calcola frattale_baseline                  // snapshot pre-propagazione

FASE 2: SEMANTIC BOOST (linee 1891-1945)
├─ KG Semantic Boost                          // IS_A, HAS, DOES → campo
│  └─ inference.field_boosts()                // O(input_words × vicini_KG)
├─ Schema Activation                          // 2+ parole IS_A comune → concetto astratto
│  └─ inference.type_chain()                  // O(input_words × depth)
└─ SelfModel Influence                        // credenze/valori → campo
   └─ self_model.field_boosts()               // O(credenze × anchors)

FASE 3: PROPAGAZIONE PF1 (linee 1946-1970)
├─ propagate_field_words()                    // **COLLO DI BOTTIGLIA #4**
│  ├─ decay pf_activation × 0.85              // O(N) = O(25.561)
│  ├─ sync word_topology → pf_activation      // O(active)
│  ├─ PF1 propagate (top-40 sorgenti)         // O(40 × 8) = O(320)
│  ├─ hebbian_update()                        // O(active × 8)
│  ├─ sync pf_activation → word_topology      // O(active)
│  └─ identity amplification [0.7, 1.3]       // O(active)
└─ calcola frattale_delta (post - baseline)   // O(frattali) = O(64)

FASE 4: INPUT READING (linee 1971-1990)
├─ read_input()                               // classifica atto comunicativo
│  ├─ top-3 frattali delta                    // O(64 × log(64))
│  ├─ salient word (più stabile)              // O(input_words)
│  └─ enrich_act_via_kg()                     // O(input_words × KG)
└─ narrative_self.deliberate()                // **FASE 6 CRITICA**
   ├─ stance da VitalState + atto             // O(1)
   ├─ intention da stance + topic_continuity  // O(1)
   ├─ awareness da KB o narrazione            // O(KB_entries)
   └─ topic_continuity (cosine frattali)      // O(64)

FASE 5: RISONANZA FRATTALE (linee 1991-2010)
├─ seed_vital_field() (se Reflect)            // stance → frattali
├─ apply_fractal_resonance(delta)             // **CASSA ARMONICA**
│  └─ top-5 parole per frattale attivo        // O(frattali_attivi × 5)
└─ episode_store.recall_into()                // φ-decay blend
   └─ cosine similarity episodi               // O(episodi × active)

FASE 6: PERTURBAZIONE SIMPLICIALE (linee 2011-2050)
├─ inscribe_phrase()                          // crea simplessi da frase
├─ apply_perturbation()                       // propaga nel complesso
├─ memory.capture()                           // STM snapshot
├─ memory.resonate()                          // MTM/LTM recall
└─ will.sense()                               // **VOLONTÀ EMERGENTE**
   ├─ pressioni da vital, dream, frattali     // O(frattali)
   ├─ compound_bias                           // O(composti)
   └─ codon [usize;2]                         // top-2 dimensioni

FASE 7: AGGIORNAMENTI IDENTITÀ (linee 2051-2100)
├─ SelfModel.update_from_activation()         // credenze + valori
├─ SelfModel.update_values_from_stance()      // stance → valori
└─ SemanticEpisode.record()                   // episodio nominato
   └─ se field_energy > 0.1                   // soglia significatività

TOTALE: ~250 linee, ~15 ms (stima)
```

**COMPLESSITÀ TOTALE receive()**:
- Dominata da: propagate_field_words() → O(N) scan
- Secondaria: homology (ogni 10 turni) → O(simplessi²)
- Terziaria: tutto il resto → O(active × log(active))

**COLLI DI BOTTIGLIA IDENTIFICATI**:
1. **word_topology scan O(N)** — linea 456
2. **pf1 scan O(N)** — linea 456
3. **homology O(simplessi²)** — ogni 10 turni
4. **propagate_field_words() totale** — ~5-10 ms

**OTTIMIZZAZIONI POSSIBILI**:
- Set attivo (solo parole > threshold) → O(active) invece di O(N)
- Homology incrementale → O(delta) invece di O(N²)
- Parallelizzazione propagazione (Rayon) → speedup 2-4×

**METRICHE PERFORMANCE ATTUALI** (stima):
- receive() totale: ~15-20 ms
- propagate_field_words(): ~5-10 ms (50% tempo)
- homology (ogni 10 turni): ~50-100 ms
- generate_willed(): ~5-10 ms

**COERENZA FILOSOFICA**: ✅ ALTA
- Engine coordina, non comanda
- Ogni layer opera sul complesso
- Flusso dati chiaro: input → campo → volontà → output

**INCOERENZE**: ⚠️ MEDIA
- Troppi layer attivi contemporaneamente (KG + KB + SelfModel + NarrativeSelf)
- Rischio: complessità cognitiva eccessiva
- Soluzione: modalità "minimal" (solo campo + volontà)

---

## 6. LAYER 4: VOLONTÀ

### 6.1 will.rs — Il Ciclo Chiuso Percezione-Azione

**FILOSOFIA**: La volontà non è una regola — è una pressione emergente dal campo. Percezione → sentire → volere → agire.

**STRUTTURA**:
```rust
pub struct WillCore;  // stateless — funzione pura del campo

pub struct WillResult {
    pub intention: Intention,
    pub strength: f64,
    pub codon: [usize; 2],                    // top-2 dimensioni attive
    pub withdraw_reason: Option<WithdrawReason>,
}

pub enum Intention {
    Express,      // esprimere stato interno
    Explore,      // esplorare campo
    Question,     // domandare
    Remember,     // richiamare memoria
    Withdraw,     // ritirarsi (silenzio)
    Reflect,      // riflettere
    Dream,        // sognare (autonomo)
}

pub enum WithdrawReason {
    Fatigue,      // fatica alta
    Confusion,    // campo caotico
    Satiation,    // curiosità sazia
    Overload,     // troppe pressioni contrastanti
}
```

**CALCOLI CRITICI**:

1. **Sense** (will.rs:234):
```rust
pub fn sense(
    &self,
    vital: &VitalState,
    dream_phase: SleepPhase,
    fractals: &[(FractalId, f64)],
    unknown_words: &[String],
    mem_resonance: f64,
    ego_activation: f64,
    curiosity_gaps: &[u32],
    compound_bias: &[(usize, f64)],
    dialogue_ctx: &DialogueContext,
    field_sig: &[f64; 8],
) -> WillResult {
    // Inizializza pressioni base
    let mut pressures = [0.0_f64; 7];  // [Express, Explore, Question, Remember, Withdraw, Reflect, Dream]
    
    // 1. Pressioni vitali
    pressures[0] += vital.activation * 0.3;           // energia → Express
    pressures[4] += vital.fatigue * 0.5;              // fatica → Withdraw
    pressures[1] += vital.curiosity * 0.4;            // curiosità → Explore
    
    // 2. Pressioni frattali
    for (fid, act) in fractals {
        match fid {
            32 => pressures[0] += act * 0.3,          // IDENTITA → Express
            54 => pressures[5] += act * 0.4,          // VERITA → Reflect
            45 => pressures[1] += act * 0.3,          // INTRECCIO → Explore
            _ => {}
        }
    }
    
    // 3. Pressioni da parole ignote
    if !unknown_words.is_empty() {
        let unknown_pressure = (unknown_words.len() as f64 * 0.15).min(0.6);
        pressures[2] += unknown_pressure;             // ignote → Question
    }
    
    // 4. Pressioni da memoria
    pressures[3] += mem_resonance * 0.4;              // risonanza → Remember
    
    // 5. Pressioni da curiosità strutturale (buchi omologici)
    if !curiosity_gaps.is_empty() {
        let gap_pressure = (curiosity_gaps.len() as f64 * 0.1).min(0.5);
        pressures[2] += gap_pressure;                 // lacune → Question
    }
    
    // 6. Pressioni da composti frattali
    for (idx, bias) in compound_bias {
        pressures[*idx] += bias;
    }
    
    // 7. Pressioni da dialogo
    if dialogue_ctx.turn_count > 0 {
        pressures[0] += dialogue_ctx.coherence * 0.2; // coerenza → Express
        pressures[1] += dialogue_ctx.novelty * 0.2;   // novità → Explore
    }
    
    // 8. Pressioni da sogno
    if dream_phase.is_sleeping() {
        pressures[6] += 0.8;                          // dormendo → Dream
    }
    
    // 9. Normalizza e trova dominante
    let total: f64 = pressures.iter().sum();
    if total > 0.0 {
        for p in &mut pressures {
            *p /= total;
        }
    }
    
    let (intention_idx, &strength) = pressures.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();
    
    let intention = match intention_idx {
        0 => Intention::Express,
        1 => Intention::Explore,
        2 => Intention::Question,
        3 => Intention::Remember,
        4 => Intention::Withdraw,
        5 => Intention::Reflect,
        6 => Intention::Dream,
        _ => Intention::Express,
    };
    
    // 10. Calcola codon (top-2 dimensioni attive)
    let mut dim_scores: Vec<(usize, f64)> = field_sig.iter()
        .enumerate()
        .map(|(i, &v)| (i, v))
        .collect();
    dim_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let codon = [dim_scores[0].0, dim_scores[1].0];
    
    // 11. Withdraw reason
    let withdraw_reason = if intention == Intention::Withdraw {
        if vital.fatigue > 0.7 {
            Some(WithdrawReason::Fatigue)
        } else if vital.tension > 0.8 {
            Some(WithdrawReason::Overload)
        } else if vital.curiosity < 0.2 {
            Some(WithdrawReason::Satiation)
        } else {
            Some(WithdrawReason::Confusion)
        }
    } else {
        None
    };
    
    WillResult {
        intention,
        strength,
        codon,
        withdraw_reason,
    }
}
```

**COMPLESSITÀ**: O(frattali + composti + dimensioni) ≈ O(64 + 12 + 8) = O(84) = O(1)

**COERENZA FILOSOFICA**: ✅ PERFETTA
- Volontà = pressione emergente (non regola)
- Codon = I Ching (64 stati)
- Withdraw = autonomia genuina (può rifiutare)

**INCOERENZE**: ❌ NESSUNA

**POTENZIALI**:
- Apprendimento bias (quali pressioni portano a interazioni soddisfacenti)
- Volontà stratificata (intenzioni primarie + secondarie)
- Codon → saggezza I Ching (mappatura esagrammi)

**RISCHI**:
- Pressioni hardcoded → non evolvono
- Soluzione: tracciare outcome + rinforzo bias efficaci

---

### 6.2 memory.rs — Stratificazione Temporale

**FILOSOFIA**: La memoria non è archiviazione — è contrazione del campo. STM/MTM/LTM = gradi di contrazione temporale.

**STRUTTURA**:
```rust
pub struct TopologicalMemory {
    stm: VecDeque<FieldImprint>,                      // cap 20
    mtm: Vec<FieldImprint>,                           // cap 100
    ltm: Vec<FieldImprint>,                           // illimitato
}

pub struct FieldImprint {
    pub timestamp: u64,
    pub active_simplices: Vec<SimplexId>,
    pub fractal_signature: Vec<(FractalId, f64)>,
    pub field_energy: f64,
    pub consolidation_count: u32,                     // quante volte consolidato
}
```

**CALCOLI CRITICI**:

1. **Capture** (memory.rs:123):
```rust
pub fn capture(&mut self, complex: &SimplicialComplex, fractals: &[(FractalId, f64)], energy: f64) {
    let imprint = FieldImprint {
        timestamp: now_secs(),
        active_simplices: complex.active_simplices().collect(),
        fractal_signature: fractals.to_vec(),
        field_energy: energy,
        consolidation_count: 0,
    };
    
    self.stm.push_back(imprint);
    if self.stm.len() > 20 {
        self.stm.pop_front();
    }
}
```

2. **Consolidate** (memory.rs:189):
```rust
pub fn consolidate(&mut self) {
    // STM → MTM: imprint con activation_count >= 3
    let mut to_promote = Vec::new();
    for imprint in &mut self.stm {
        if imprint.consolidation_count >= 3 {
            to_promote.push(imprint.clone());
        }
    }
    
    for imprint in to_promote {
        self.mtm.push(imprint);
        if self.mtm.len() > 100 {
            // Evict weakest
            self.mtm.sort_by(|a, b| a.field_energy.partial_cmp(&b.field_energy).unwrap());
            self.mtm.remove(0);
        }
    }
    
    // MTM → LTM: imprint con consolidation_count >= 10
    let mut to_crystallize = Vec::new();
    for imprint in &mut self.mtm {
        if imprint.consolidation_count >= 10 {
            to_crystallize.push(imprint.clone());
        }
    }
    
    for imprint in to_crystallize {
        self.ltm.push(imprint);
    }
}
```

**FILOSOFIA CONSOLIDAMENTO**:
- STM: presente (ultimi 20 imprint)
- MTM: abitudine (consolidati 3+ volte)
- LTM: identità (cristallizzati 10+ volte)

**COERENZA**: ✅ ALTA
- Riflette memoria biologica (Atkinson-Shiffrin)
- Consolidamento = rinforzo (non tempo)

**METRICHE ATTUALI**:
- STM: ~20 imprint × 2 KB = ~40 KB
- MTM: ~100 imprint × 2 KB = ~200 KB
- LTM: ~500 imprint × 2 KB = ~1 MB (stima)

**POTENZIALI**:
- Decay MTM (imprint non richiamati → STM)
- Pruning LTM (imprint con energy < 0.1)
- Memoria episodica integrata (FieldImprint + SemanticEpisode)

**RISCHI**:
- LTM illimitato → crescita infinita
- Soluzione: cap LTM a 1000 + eviction weakest

---

### 6.3 episodic.rs — Memoria φ-Decay

**FILOSOFIA**: Il passato non svanisce — decade secondo il numero aureo (φ⁻ⁿ).

**STRUTTURA**:
```rust
pub struct EpisodeStore {
    episodes: Vec<Episode>,
    capacity: usize,                                  // 200
}

pub struct Episode {
    pub sparse_activations: Vec<(WordId, f32)>,       // top-50 parole
    pub age: u32,                                     // cicli REM
    pub intensity: f64,                               // field_energy al momento
}

pub const PHI_INV: f64 = 0.618_033_988;               // φ⁻¹
pub const RECALL_THRESHOLD: f64 = 0.45;
pub const RECALL_BLEND: f64 = 0.12;
```

**CALCOLI CRITICI**:

1. **Encode** (episodic.rs:123):
```rust
pub fn encode(&mut self, word_topology: &WordTopology, intensity: f64) {
    if intensity < 0.15 { return; }  // soglia significatività
    
    // Top-50 parole attive
    let mut active: Vec<(WordId, f32)> = word_topology.vertices()
        .filter(|v| v.activation > 0.02)
        .map(|v| (v.id, v.activation as f32))
        .collect();
    active.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    active.truncate(50);
    
    let episode = Episode {
        sparse_activations: active,
        age: 0,
        intensity,
    };
    
    self.episodes.push(episode);
    if self.episodes.len() > self.capacity {
        // Evict weakest (intensity × φ⁻ⁿ)
        let weakest_idx = self.episodes.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let score_a = a.intensity * PHI_INV.powi(a.age as i32);
                let score_b = b.intensity * PHI_INV.powi(b.age as i32);
                score_a.partial_cmp(&score_b).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap();
        self.episodes.remove(weakest_idx);
    }
}
```

2. **Recall** (episodic.rs:234):
```rust
pub fn recall_into(&self, word_topology: &mut WordTopology) {
    // Calcola firma corrente
    let current_sig: Vec<(WordId, f32)> = word_topology.vertices()
        .filter(|v| v.activation > 0.02)
        .map(|v| (v.id, v.activation as f32))
        .collect();
    
    // Per ogni episodio, calcola cosine similarity
    for episode in &self.episodes {
        let similarity = cosine_similarity(&current_sig, &episode.sparse_activations);
        
        if similarity > RECALL_THRESHOLD {
            // Peso φ-decay
            let weight = PHI_INV.powi(episode.age as i32);
            let blend_strength = RECALL_BLEND * weight * similarity;
            
            // Blend episodio nel campo
            for (word_id, ep_activation) in &episode.sparse_activations {
                let current = word_topology.get_activation(*word_id);
                let blended = current + ep_activation * blend_strength as f32;
                word_topology.set_activation(*word_id, blended.min(1.0));
            }
        }
    }
}
```

**FILOSOFIA φ-DECAY**:
- φ⁻¹ = 0.618 (numero aureo inverso)
- age=0 → peso 1.0
- age=1 → peso 0.618
- age=2 → peso 0.382
- age=5 → peso 0.090
- age=10 → peso 0.008

**COERENZA**: ✅ PERFETTA
- Decay aureo (non esponenziale) — filosoficamente motivato
- Recall = pattern completion (non query)
- Blend leggero (0.12) — memoria informa, non comanda

**METRICHE ATTUALI**:
- Episodi: ~200 (cap)
- Memoria: 200 × 50 × 8 byte = ~80 KB

**POTENZIALI**:
- Episodi semantici integrati (vettori + concetti)
- Recall selettivo (solo episodi con frattali simili)
- Consolidamento episodi (merge simili)

**RISCHI**:
- Cap 200 potrebbe essere troppo basso per sessioni lunghe
- Soluzione: cap dinamico (basato su memoria disponibile)

---

