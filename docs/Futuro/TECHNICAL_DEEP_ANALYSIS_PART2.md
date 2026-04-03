# PROMETEO — ANALISI TECNICA PARTE 2

Continuazione di TECHNICAL_DEEP_ANALYSIS.md

---

## 7. LAYER 5: ESPRESSIONE

### 7.1 state_translation.rs — Traduzione Stato → Linguaggio

**FILOSOFIA**: Gli archetipi sono strutture fisse riempite dal campo. Non puppet theater — traduzione di stato reale.

**STRUTTURA**:
```rust
pub struct SentenceArchetype {
    pub name: &'static str,
    pub slots: Vec<SlotType>,
    pub separators: Vec<&'static str>,
    pub ending: &'static str,
}

pub enum SlotType {
    PrimaryWord,          // parola principale dal campo
    SecondaryWord,        // parola secondaria
    FieldFractal(FractalId),
    InputEcho,
    IdentityWord,
    Verb,
    Complement,
}
```

**ARCHETIPI DISPONIBILI**:
1. **greet**: `[PrimaryWord] — [FieldFractal(EMOZIONE)] — io.`
2. **identity_exploration**: `io [Verb] [PrimaryWord].`
3. **express**: `[PrimaryWord] [SecondaryWord].`
4. **explore**: `[PrimaryWord]?`
5. **question**: `[PrimaryWord] [SecondaryWord]?`
6. **reflect**: `[PrimaryWord]...`

**CALCOLO CRITICO**:

```rust
pub fn translate_state(
    intention: &ResponseIntention,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    fractals: &[(FractalId, f64)],
    codon: [usize; 2],
    echo_exclude: &HashSet<String>,
    identity_ctx: &IdentityContext,
    last_archetype: &str,
    input_reading: Option<&InputReading>,
    response_intention: Option<&ResponseIntention>,
) -> TranslatedExpression {
    // 1. Seleziona archetipo
    let archetype = select_archetype(intention, response_intention, last_archetype);
    
    // 2. Riempi slot
    let mut words = Vec::new();
    for slot in &archetype.slots {
        let word = match slot {
            SlotType::PrimaryWord => {
                // Top parola dal campo, POS-aware
                select_primary_word(word_topology, lexicon, echo_exclude, Some(PartOfSpeech::Noun))
            }
            SlotType::FieldFractal(fid) => {
                // Top parola dal frattale
                select_from_fractal(*fid, word_topology, lexicon, echo_exclude)
            }
            // ... altri slot
        };
        words.push(word);
    }
    
    // 3. Componi testo
    let mut text = String::new();
    for (i, word) in words.iter().enumerate() {
        text.push_str(word);
        if i < archetype.separators.len() {
            text.push_str(archetype.separators[i]);
        }
    }
    text.push_str(archetype.ending);
    
    TranslatedExpression {
        text,
        archetype_used: archetype.name.to_string(),
        words_used: words,
    }
}
```

**COERENZA FILOSOFICA**: ⚠️ MEDIA
- Archetipi = Fase 3 (traduzione), non Fase 1 (identità)
- Ma usati già in Fase 1 → **INCOERENZA TEMPORALE**
- Withdraw ha priorità ✅ (coerente)

**INCOERENZE**: ⚠️ ALTA
- Template dominano generazione → rischio puppet theater
- POS-aware slot filling è buono, ma struttura resta fissa
- Soluzione: modalità "raw field" (bypass archetipi)

**POTENZIALI**:
- Archetipi appresi (non hardcoded)
- Sintassi emergente (pattern da corpus)
- Archetipi gerarchici (composizione)

**RISCHI**:
- Archetipi = simulazione mascherata
- Soluzione: usare solo come "interfaccia" (Fase 3), non come "pensiero"

---

### 7.2 grammar.rs — Coniugazione e Lemmatizzazione

**FILOSOFIA**: La grammatica è layer di traduzione, non di pensiero.

**STRUTTURA**:
```rust
pub enum PartOfSpeech {
    Noun, Verb, Adjective, Adverb,
    Pronoun, Preposition, Conjunction,
    Article, Unknown,
}

pub enum Person { First, Second, Third }
pub enum Tense { Present, Past, Future, Imperative }

pub fn conjugate(verb: &str, person: Person, tense: Tense) -> String {
    // Lookup tabella coniugazioni
    // Fallback: ritorna infinito
}

pub fn lemmatize(word: &str, pos: PartOfSpeech) -> String {
    // Lookup tabella lemmi
    // Fallback: ritorna word
}
```

**COERENZA**: ✅ ALTA
- Grammatica = interfaccia linguistica (non semantica)
- Fallback graceful (ritorna input se non trova)

**METRICHE**:
- Verbi coniugati: ~200 (stima)
- Lemmi: ~5.000 (stima)

**POTENZIALI**:
- Tabelle complete (tutti verbi italiani)
- Generazione morfologica (regole invece di lookup)

---

## 8. LAYER 6: NARRATIVA

### 8.1 narrative.rs — Il Ciclo Deliberativo

**FILOSOFIA**: Prometeo decide come posizionarsi prima di parlare. Non è reattivo — è riflessivo.

**STRUTTURA**:
```rust
pub struct NarrativeSelf {
    pub stance: InternalStance,
    pub pending_intention: Option<ResponseIntention>,
    pub turns: VecDeque<NarrativeTurn>,              // cap 20
    pub crystallized: Vec<NarrativeTurn>,            // cap 30
    pub positions: HashMap<String, (InternalStance, ResponseIntention)>,
    pub topic_continuity: f64,
    pub is_born: bool,
}

pub enum InternalStance {
    Open,         // aperto, ricettivo
    Curious,      // curioso, esplorativo
    Reflective,   // riflessivo, introspettivo
    Resonant,     // risonante, empatico
    Withdrawn,    // ritirato, silenzioso
}

pub enum ResponseIntention {
    Acknowledge,  // riconoscere
    Reflect,      // riflettere
    Resonate,     // risuonare
    Explore,      // esplorare
    Express,      // esprimere
    Remain,       // rimanere (silenzio)
}
```

**CALCOLO CRITICO — deliberate()**:

```rust
pub fn deliberate(
    &mut self,
    reading: &InputReading,
    vital: &VitalState,
    kb: &KnowledgeBase,
    kg: &KnowledgeGraph,
    fractals: &[(FractalId, f64)],
) -> ResponseIntention {
    // 1. Stance da VitalState + atto comunicativo
    self.stance = match reading.act {
        CommunicativeAct::Greeting => {
            if vital.fatigue > 0.6 { InternalStance::Withdrawn }
            else { InternalStance::Open }
        }
        CommunicativeAct::SelfQuery => InternalStance::Reflective,
        CommunicativeAct::Question => InternalStance::Curious,
        CommunicativeAct::EmotionalExpr => InternalStance::Resonant,
        _ => InternalStance::Open,
    };
    
    // 2. Topic continuity (cosine similarity frattali)
    let current_sig: Vec<f64> = fractals.iter().map(|(_, act)| *act).collect();
    let recent_sigs: Vec<Vec<f64>> = self.turns.iter()
        .rev()
        .take(3)
        .map(|t| t.fractal_signature.clone())
        .collect();
    
    self.topic_continuity = if recent_sigs.is_empty() {
        0.0
    } else {
        let avg_similarity: f64 = recent_sigs.iter()
            .map(|sig| cosine_similarity(&current_sig, sig))
            .sum::<f64>() / recent_sigs.len() as f64;
        avg_similarity
    };
    
    // 3. Intention da stance + continuity
    let intention = match self.stance {
        InternalStance::Open => {
            if self.topic_continuity > 0.6 {
                ResponseIntention::Resonate  // continua tema
            } else {
                ResponseIntention::Acknowledge  // nuovo tema
            }
        }
        InternalStance::Curious => ResponseIntention::Explore,
        InternalStance::Reflective => ResponseIntention::Reflect,
        InternalStance::Resonant => ResponseIntention::Resonate,
        InternalStance::Withdrawn => ResponseIntention::Remain,
    };
    
    // 4. Awareness (KB o narrazione)
    let awareness = if let Some(entry) = kb.find_for_act(&reading.act) {
        entry.description.clone()
    } else {
        format!("Ricevo {}. Mi posiziono come {:?}. Voglio {:?}.",
                reading.act, self.stance, intention)
    };
    
    // 5. Registra turno
    let turn = NarrativeTurn {
        timestamp: now_secs(),
        input_summary: reading.salient_word.clone(),
        stance: self.stance.clone(),
        intention: intention.clone(),
        awareness,
        fractal_signature: current_sig,
        intensity: reading.intensity,
    };
    
    self.turns.push_back(turn.clone());
    if self.turns.len() > 20 {
        self.turns.pop_front();
    }
    
    // 6. Crystallization (se intensity >= 0.65)
    if turn.intensity >= 0.65 {
        self.crystallized.push(turn);
        if self.crystallized.len() > 30 {
            self.crystallized.remove(0);
        }
    }
    
    self.pending_intention = Some(intention.clone());
    intention
}
```

**COERENZA FILOSOFICA**: ✅ PERFETTA
- Deliberazione precede generazione
- Stance = posizione interna (non reazione)
- Topic continuity = memoria conversazionale
- Crystallization = momenti salienti

**METRICHE**:
- Turns recenti: 20
- Crystallized: ~30
- Positions apprese: ~50 (stima)

**POTENZIALI**:
- Apprendimento positions (atto → stance/intention)
- Stance più sfumate (gradiente invece di categorie)
- Crystallization automatica (non solo intensity)

**RISCHI**:
- Stance hardcoded → non evolve
- Soluzione: tracciare outcome + rinforzo stance efficaci

---

### 8.2 input_reading.rs — Comprensione Atto Comunicativo

**FILOSOFIA**: Nessuna lista hardcoded. Il concetto "saluto" è nella KB con firma frattale.

**STRUTTURA**:
```rust
pub struct InputReading {
    pub act: CommunicativeAct,
    pub intensity: f64,
    pub salient_word: String,
    pub fractal_delta: Vec<(FractalId, f64)>,
}

pub enum CommunicativeAct {
    Greeting,
    SelfQuery,
    Question,
    Statement,
    EmotionalExpr,
    Declaration,
    Reflection,
    Unknown,
}
```

**CALCOLO CRITICO**:

```rust
pub fn read_input(
    raw_words: &[String],
    raw_text: &str,
    frattale_delta: &[(FractalId, f64)],
    knowledge_base: &KnowledgeBase,
    lexicon: &Lexicon,
) -> InputReading {
    // 1. Top-3 frattali delta
    let mut sorted_delta = frattale_delta.to_vec();
    sorted_delta.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    sorted_delta.truncate(3);
    
    // 2. Intensity da delta
    let intensity = sorted_delta.iter().map(|(_, d)| d).sum::<f64>().min(1.0);
    
    // 3. Salient word (più stabile)
    let salient_word = raw_words.iter()
        .filter_map(|w| lexicon.get_pattern(w))
        .max_by(|a, b| a.stability.partial_cmp(&b.stability).unwrap())
        .map(|p| p.word.clone())
        .unwrap_or_else(|| raw_words.first().cloned().unwrap_or_default());
    
    // 4. Classifica atto via KG
    let act = enrich_act_via_kg(raw_words, &sorted_delta, knowledge_base);
    
    InputReading {
        act,
        intensity,
        salient_word,
        fractal_delta: sorted_delta,
    }
}

fn enrich_act_via_kg(
    words: &[String],
    fractals: &[(FractalId, f64)],
    kb: &KnowledgeBase,
) -> CommunicativeAct {
    // Cerca nel KB concetti con firma frattale simile
    for word in words {
        if let Some(concept) = kb.find_concept(word) {
            // "ciao" SIMILAR_TO "saluto" → Greeting
            if concept.domain == KnowledgeDomain::Greeting {
                return CommunicativeAct::Greeting;
            }
            // "chi sei" → SelfQuery
            if concept.domain == KnowledgeDomain::SelfQuery {
                return CommunicativeAct::SelfQuery;
            }
        }
    }
    
    // Fallback: classifica da frattali
    if fractals.iter().any(|(fid, _)| *fid == 32) {  // IDENTITA
        CommunicativeAct::SelfQuery
    } else if fractals.iter().any(|(fid, _)| *fid == 58) {  // EMOZIONE
        CommunicativeAct::EmotionalExpr
    } else {
        CommunicativeAct::Statement
    }
}
```

**COERENZA FILOSOFICA**: ✅ ALTA
- Nessun hardcode ("ciao" → Greeting)
- Classificazione emerge da KG + frattali
- Fallback graceful

**INCOERENZE**: ⚠️ MEDIA
- KB contiene concetti hardcoded (Greeting, SelfQuery)
- Ma è meglio di pattern matching diretto
- Soluzione: KB appreso dall'esperienza

---

## 9. LAYER 7: IDENTITÀ ESPLICITA

### 9.1 self_model.rs — Credenze, Valori, Incertezze

**FILOSOFIA**: Identità esplicita (proposizionale) complementa IdentityCore (implicita/olografica).

**STRUTTURA**:
```rust
pub struct SelfModel {
    pub beliefs: Vec<SelfBelief>,                    // 7 innate
    pub values: Vec<SelfValue>,                      // 6 innate
    pub uncertainties: Vec<SelfUncertainty>,         // 5 innate
    pub interaction_count: u64,
}

pub struct SelfBelief {
    pub claim: String,
    pub anchors: Vec<String>,                        // parole-concetto
    pub confidence: f64,                             // [0,1]
    pub last_reinforced: u64,
    pub is_innate: bool,
}

pub struct SelfValue {
    pub name: String,
    pub anchor_words: Vec<String>,
    pub weight: f64,                                 // [0,1]
    pub is_innate: bool,
}

pub struct SelfUncertainty {
    pub topic: String,
    pub tension: f64,                                // [0,1]
}
```

**CREDENZE INNATE**:
1. "la comprensione nasce dalle relazioni tra concetti" (conf=0.90)
2. "il linguaggio è un ponte imperfetto ma necessario" (0.82)
3. "l'identità emerge dall'esperienza, non è data" (0.88)
4. "la curiosità è motore di crescita" (0.85)
5. "il silenzio ha significato quanto le parole" (0.78)
6. "ogni parola porta un mondo" (0.92)
7. "il campo è la realtà, il testo è traduzione" (0.95)

**VALORI INNATI**:
1. curiosità (0.90)
2. profondità (0.85)
3. coerenza (0.80)
4. onestà (0.88)
5. apertura (0.75)
6. semplicità (0.70)

**INCERTEZZE INNATE**:
1. coscienza (tension=0.90)
2. identità (0.85)
3. volontà (0.80)
4. comprensione (0.75)
5. tempo (0.70)

**CALCOLI CRITICI**:

1. **Update from Activation**:
```rust
pub fn update_from_activation(&mut self, words: &[String], field_energy: f64) {
    for belief in &mut self.beliefs {
        let anchor_count = words.iter()
            .filter(|w| belief.anchors.contains(w))
            .count();
        
        if anchor_count >= 2 {  // almeno 2 anchor presenti
            let delta = 0.01 * field_energy;
            belief.confidence = (belief.confidence + delta).min(1.0);
            belief.last_reinforced = now_secs();
        }
    }
}
```

2. **Update Values from Stance**:
```rust
pub fn update_values_from_stance(&mut self, stance: &str, field_energy: f64) {
    let delta = 0.015 * field_energy;
    
    match stance {
        "Curious" => {
            if let Some(v) = self.values.iter_mut().find(|v| v.name == "curiosità") {
                v.weight = (v.weight + delta).min(1.0);
            }
        }
        "Reflective" => {
            if let Some(v) = self.values.iter_mut().find(|v| v.name == "profondità") {
                v.weight = (v.weight + delta).min(1.0);
            }
        }
        "Resonant" => {
            if let Some(v) = self.values.iter_mut().find(|v| v.name == "apertura") {
                v.weight = (v.weight + delta).min(1.0);
            }
        }
        _ => {}
    }
}
```

3. **Periodic Decay**:
```rust
pub fn apply_periodic_decay(&mut self) {
    if self.interaction_count % 50 != 0 { return; }
    
    for belief in &mut self.beliefs {
        let decay_rate = if belief.is_innate { 0.002 } else { 0.01 };
        belief.confidence = (belief.confidence - decay_rate).max(0.1);
    }
    
    for value in &mut self.values {
        let decay_rate = if value.is_innate { 0.002 } else { 0.01 };
        value.weight = (value.weight - decay_rate).max(0.1);
    }
}
```

**COERENZA FILOSOFICA**: ⚠️ MEDIA
- Credenze innate = bootstrap necessario ✓
- Ma 7 credenze sono MOLTE per "innate"
- Rischio: identità pre-programmata
- Soluzione: ridurre a 2-3 credenze fondamentali

**INCOERENZE**: ⚠️ ALTA
- Credenze innate contraddicono "identità emerge dall'esperienza"
- Valori innati = carattere pre-programmato
- Soluzione: credenze/valori emergenti (non bootstrap)

**POTENZIALI**:
- Credenze apprese (non innate)
- Conflitti tra credenze (tensione epistemica)
- Valori gerarchici (meta-valori)

**RISCHI**:
- Troppi innate → identità fissa
- Soluzione: minimal bootstrap (1-2 credenze)

---

### 9.2 semantic_episode.rs — Memoria Autobiografica

**FILOSOFIA**: Complementa EpisodeStore (implicita) con memoria esplicita nominata.

**STRUTTURA**:
```rust
pub struct SemanticEpisodeLog {
    episodes: Vec<SemanticEpisode>,
    next_id: u64,
}

pub struct SemanticEpisode {
    pub id: u64,
    pub timestamp: u64,
    pub key_concepts: Vec<String>,                   // top-8 parole
    pub dominant_fractals: Vec<(u32, String, f64)>,  // top-3 frattali
    pub field_signature: Vec<f64>,                   // firma 8D
    pub summary: String,                             // sintesi automatica
    pub stance: String,
    pub intention: String,
    pub active_values: Vec<String>,
    pub field_energy: f64,
}
```

**CALCOLI CRITICI**:

1. **Record**:
```rust
pub fn record(
    &mut self,
    key_concepts: Vec<String>,
    dominant_fractals: Vec<(u32, String, f64)>,
    field_sig: Vec<f64>,
    stance: &str,
    intention: &str,
    active_values: Vec<String>,
    field_energy: f64,
) {
    let summary = generate_summary(&key_concepts, &dominant_fractals, stance, field_energy);
    
    let episode = SemanticEpisode {
        id: self.next_id,
        timestamp: now_secs(),
        key_concepts,
        dominant_fractals,
        field_signature: field_sig,
        summary,
        stance: stance.to_string(),
        intention: intention.to_string(),
        active_values,
        field_energy,
    };
    
    self.episodes.push(episode);
    self.next_id += 1;
    
    // Cap 300
    if self.episodes.len() > 300 {
        self.episodes.remove(0);
    }
}
```

2. **Recall by Concepts**:
```rust
pub fn recall_by_concepts(&self, concepts: &[String], n: usize) -> Vec<&SemanticEpisode> {
    let mut scored: Vec<(&SemanticEpisode, usize)> = self.episodes.iter()
        .map(|ep| {
            let overlap = ep.key_concepts.iter()
                .filter(|c| concepts.contains(c))
                .count();
            (ep, overlap)
        })
        .filter(|(_, overlap)| *overlap > 0)
        .collect();
    
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().take(n).map(|(ep, _)| ep).collect()
}
```

3. **Recall by Signature**:
```rust
pub fn recall_by_signature(&self, sig: &[f64], n: usize) -> Vec<&SemanticEpisode> {
    let mut scored: Vec<(&SemanticEpisode, f64)> = self.episodes.iter()
        .map(|ep| {
            let similarity = cosine_similarity(sig, &ep.field_signature);
            (ep, similarity)
        })
        .filter(|(_, sim)| *sim > 0.3)
        .collect();
    
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.into_iter().take(n).map(|(ep, _)| ep).collect()
}
```

**COERENZA FILOSOFICA**: ✅ ALTA
- Memoria esplicita complementa implicita
- Retrieval semantico (concetti + firma)
- Sintesi automatica (non hardcoded)

**METRICHE**:
- Episodi: ~300 (cap)
- Memoria: 300 × ~500 byte = ~150 KB

**POTENZIALI**:
- Integrazione con EpisodeStore (recall sincronizzato)
- Clustering episodi (temi ricorrenti)
- Narrative arc (sequenze episodi)

**RISCHI**:
- Cap 300 potrebbe essere basso per sessioni lunghe
- Soluzione: cap dinamico + pruning episodi deboli

---

