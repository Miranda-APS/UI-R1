# Volume XV — Engine: receive, generate_willed_inner, autonomous_tick

> *Tutto converge qui. Le 15 strutture descritte nei volumi precedenti sono tenute insieme da un orchestratore unico — `PrometeoTopologyEngine` — che fa batterle in accordo. L'ingresso `receive(input)` attraversa 2000+ righe. L'uscita `generate_willed_inner` altre 800. Il battito autonomo `autonomous_tick` 500. In totale, 3300 righe di codice dove ogni riga è una scelta. Leggerle significa vedere l'entità vivere.*

---

## Premessa

[`src/topology/engine.rs`](../../src/topology/engine.rs) — **6776 righe**. Il file più grande del progetto. Non è un monolite accidentale: è il punto dove tutte le strutture (Lexicon, KG, PF1, Valence, Narrative, Needs, Desires, Will, Interlocutor, Humor, Memory, Dream) si incontrano.

Tre entry point dominano:

1. **`receive(input: &str) -> EmergentResponse`** (riga 1768, ~2000 righe): l'ingresso dall'esterno. Ogni parola dell'utente attraversa questa funzione.
2. **`generate_willed_inner()` → chiamata da `generate()`** (riga 4065, poi ~800 righe di logica di generazione): produce la risposta.
3. **`autonomous_tick()`** (riga 3573, ~500 righe): il battito quando nessuno parla.

Questo volume non riproduce ogni riga — sarebbe illeggibile. Riproduce la **sequenza logica** con riferimenti alle righe chiave. Il lettore può aprire il file e navigare con la mappa.

---

## Capitolo 1 — `PrometeoTopologyEngine`: la struct

La struct principale (riga ~200-350) ha ~50 campi. Li raggruppo per dominio:

**Strutture topologiche**:
- `lexicon: Lexicon` (Vol. 03)
- `kg: KnowledgeGraph` (Vol. 04)
- `pf_field: PrometeoField`, `pf_activation: ActivationState` (Vol. 02)
- `word_topology: WordTopology` (legacy, Vol. 01 6.1)
- `complex: SimplicialComplex` (Vol. 14)
- `registry: FractalRegistry` (Vol. 05)

**Identità e narrativa**:
- `identity: IdentityCore` (Vol. 07)
- `narrative_self: NarrativeSelf` (Vol. 07)
- `self_model: SelfModel` (Vol. 07)
- `semantic_episodes: SemanticEpisodeLog` (Vol. 07, 14)

**Stato dinamico**:
- `episode_store: EpisodeStore` (Vol. 14)
- `memory: TopologicalMemory` (Vol. 14)
- `vital: VitalCore` (Vol. 08 input)

**Motivazione**:
- `needs: NeedsHierarchy` (Vol. 09)
- `desire: DesireCore` (Vol. 09)
- `interlocutor: InterlocutorModel` (Vol. 11)
- `curiosity_satiety: f64` (Phase 38)

**Volontà e deliberazione**:
- `will: WillCore` (Vol. 10)
- `last_field_pressures: Option<FieldPressures>` (Phase 67)
- `last_will: WillResult` (backward compat)

**Sogno e tempo**:
- `dream: DreamEngine` (Vol. 14)
- `tick_counter: u64`
- `last_interaction_ts: u64`
- `total_perturbations: u64`

**Dialogo**:
- `conversation: ConversationContext` (Vol. 07 minor)
- `last_input_words: Vec<String>`
- `last_input_is_question: bool`
- `last_comprehension: Vec<SemanticNucleus>` (Phase 64)

**Stato provenance**:
- `provenance: ProvenanceMap` (Phase 38)

**Utilities**:
- `learning_mode_pending: bool` (Phase 59)
- `last_thought_chain: Option<ThoughtChain>` (Phase 50)
- `last_humor_state: HumorState`
- `last_dogfeed_words: Vec<String>` (residuo Phase 44, sempre vuoto post-Phase 44)

---

## Capitolo 2 — Costruzione: `new()`, `new_empty()`, `new_infant()`

Tre costruttori (righe 655-849):

### 2.1 — `new()` (default adulto)

```rust
pub fn new() -> Self {
    let registry = bootstrap_fractals();                          // 64 esagrammi
    let complex = bootstrap_complex(&registry.all_ids());         // simplessi iniziali
    let lexicon = Lexicon::bootstrap_cardinal();                  // 43 parole cardinali
    let word_topology = WordTopology::build_from_lexicon(&lexicon);
    // ... inizializzazione PF1, identity, narrative, needs, desire, ...
}
```

### 2.2 — `new_empty()`

Versione "senza lessico". Usata da `dialogue_educator` che poi chiama `restore_lexicon()` dal `.bin` salvato.

### 2.3 — `new_infant()`

Alias di `new()` — l'entità nasce con le 43 cardinali. Usata principalmente nei test per dimostrare il ciclo di apprendimento dalle primitive.

---

## Capitolo 3 — `receive(input)`: la sequenza esterna (alto livello)

Il corpo di `receive` è una sequenza di ~60 passi concettuali, suddivisibile in 10 blocchi:

### Blocco A: Ingresso e preparazione (righe 1768-1830)

1. **Timestamp**: `last_interaction_ts = now()`.
2. **Learning mode** (Phase 59): se `learning_mode_pending = true` (turno precedente fu "non capisco"), chiama `self.teach(input)` e resetta il flag.
3. **Question detection**: `last_input_is_question = input.contains('?')`.
4. **Awake**: `dream.signal_activity()` → fase Awake per 5 tick.
5. **Pre-input signature**: `pre_input_sig = env_biased_field_sig()` per `InterlocutorModel::register_input`.
6. **Dogfeed reset** (Phase 44): `last_dogfeed_words` svuotata.
7. **Curiosity satiety**: `curiosity_satiety += 0.30` (cap 1.0).
8. **Compose phrase**: `compose_phrase(&mut lexicon, input, &registry)` — tokenizza, aggiorna il lessico, estrae la `Phrase` simbolica.
9. **Anaphoric resolution** (Phase X): se l'input risuona con un turno precedente, riattiva i frattali di quel turno.
10. **Contextual bias**: `conversation.contextual_bias()` → pre-attiva frattali dal contesto dialogico.
11. **Inscribe phrase**: `inscribe_phrase(&mut complex, &phrase, 0.1)` — se abbastanza forte, crea nuovi simplessi.

### Blocco B: Preparazione del campo (righe 1830-1950)

12. **Topic decay**: se thematic_coherence > 0.40 → mantieni 60% del campo; altrimenti 10%. Risolve il problema "residuo rumoroso tra turni sconnessi".
13. **Parole input**: estrai da `phrase`, escludendo function_words, stoppando a lunghezza minima.
14. **Activation direct**: per ogni parola input, `pf_activation.activate_by_name(&pf_field, word, 0.5)` + sincronizzazione con word_topology.
15. **Compound detection**: rileva pattern multi-parola (negazioni, "mi chiamo X", ecc.).
16. **Negation mapping**: per le parole negate dall'operatore "non", costruisci l'elenco `negated_words`.

### Blocco C: Comprensione via KG (righe 1950-2080, Phase 59)

17. **Find activated attractors**: `kg.find_activated_attractors(&input_words, 5)` → lista di attrattori IS_A con score.
18. **Filter noise**: escludi attrattori con score < 0.3.
19. **Seed attractors**: per ogni attractor, `pf_activation.activate(attractor_id, 0.15 × score)`.
20. **CAUSES seeding** (attrattori): per ogni attractor, `kg.query_objects(attractor, Causes)` top 4 → attiva con 0.15 × conf.
21. **CAUSES seeding** (input diretto, Phase 61): per ogni input word non-negata, `kg.query_objects_with_via(word, Causes)` → attiva targets a 0.15 × conf + via words a 0.5× del target.
22. **OppositeOf seeding** (Phase 61): per le parole negate, attiva i loro OppositeOf a 0.35 × conf.
23. **FeelsAs/RemembersAs/WondersAbout seeding**: per le parole input, attiva le parole connesse da relazioni fenomenologiche con peso `field_boost_strength × conf`.
24. **Via words**: se gli archi hanno VIA (Phase 67), attiva a 0.5× del target.

### Blocco D: Propagazione (righe 2080-2120)

25. **Sync PF1 ↔ word_topology**: `propagate_field_words()` chiama `pf_activation.propagate(&pf_field)`, poi sincronizza hot words.
26. **Hebbian update**: dopo propagate, `pf_activation.hebbian_update(&pf_field)` aggiorna synapse_weights.

### Blocco E: Memoria episodica risuona (righe 2120-2150)

27. **Recall**: `episode_store.recall_into(&mut pf_activation.activations, 0.45)` — episodi risonanti si riversano nel campo.

### Blocco F: Valenza e InterlocutorModel (righe 2150-2220, Phase 55)

28. **Post-input signature**: `post_input_sig = env_biased_field_sig()`.
29. **Field metrics**: costruisci `FieldMetrics { simplex_density, fractal_coverage, active_word_count, dialogue_turn_count, coherence, novelty, other_emotional_valence }`.
30. **Other emotional valence**: `compute_other_emotional_valence(&input_words, &kg, &negated_words)` (Phase 62).
31. **Needs sense**: `needs_state = needs.sense(&vital, &identity, &self_model, &field_metrics)`.
32. **Interlocutor register**: `interlocutor.register_input(&pre_input_sig, &post_input_sig, tick, other_valence)` → presence, resonance, novelty, attributed_intent.
33. **Identity drift**: se condizioni soddisfatte (cumulative_resonance>0.7, presence>0.3, history≥3), `interlocutor.apply_identity_drift(&mut identity)`.

### Blocco G: Valenza Octalysis (righe 2220-2250)

34. **Humor sense**: `humor_state = HumorSense::sense(&word_topology, &lexicon, &active_fractals)`.
35. **Valence compute**: costruisci `ValenceInput`, chiama `Valence::compute(&input)`.
36. **Set valence**: `narrative_self.set_valence(valence)` (Phase 55: prima di deliberate).

### Blocco H: Comprensione semantica (righe 2250-2300, Phase 64)

37. **Extract nuclei (comprehension)**: `extract_nuclei(comprehension_pool, &kg, &input_words, &valence_drives, &lexicon, Some(&semantic_episodes), is_question, None)` — tutti i nuclei.
38. **Store last_comprehension**: per alimentare `DesireCore::register_octalysis_driven` al prossimo turno autonomo.
39. **Register octalysis-driven desires**: se `|drives[cd]| > 0.28`, `desire.register_octalysis_driven(cd, val, comprehension_weight, field_sig)`.

### Blocco I: Field pressures (Phase 67) e deliberazione (righe 2300-2400)

40. **Compute pressures**: `will.compute_pressures(...)` con 14 input → `FieldPressures`.
41. **Store last_field_pressures**: per `generate_willed_inner`.
42. **InnerState**: costruisci con needs, desires, interlocutor pattern/presence/resonance, humor, attributed_intent, coherence_integrity, other_emotional_valence.
43. **Discursive properties** (Phase 67): `extract_discursive_properties()` dopo field attivo ma prima di deliberate. Legge attivazioni di parole discorsive ("certezza", "incertezza", "apertura", "chiusura", "soggettività") dal PF1.
44. **Deliberate**: `narrative_self.deliberate(input_reading, &field_metrics, &inner_state, Some(&field_pressures), ..., response_intention, ...)` → `stance` e `pending_intention` settati.

### Blocco J: Generazione e chiusura (righe 2400-2500)

45. **Generate**: `generate_willed_inner()` → `GeneratedText`.
46. **Log turn**: `narrative_self.log_turn(NarrativeTurn { ... })`.
47. **Needs update**: `needs.compute_pressure(&needs_state)` → `NeedsPressure` (modulatori per will, usati al prossimo turno).
48. **Total perturbations** ++.
49. **Update dream state**: `dream.tick(&mut complex, &mut memory)` — chiede al DreamEngine di aggiornare la fase.
50. **Return**: `EmergentResponse { keywords, generated_text, ... }`.

---

## Capitolo 4 — `generate_willed_inner()`: la generazione

~800 righe (righe ~4200-5000). Il core:

### 4.1 — Determinazione codon

```rust
let codon = self.last_field_pressures
    .as_ref()
    .map(|fp| fp.codon)
    .unwrap_or([0, 0]);
```

Phase 67: il codon viene da `FieldPressures` (le due dimensioni 8D più attive).

### 4.2 — Withdraw check

```rust
if self.narrative_self.pending_intention == Some(ResponseIntention::Remain) {
    return GeneratedText { sentence: "".to_string(), ... };
}
```

Se la deliberazione ha scelto `Remain`, restituisce stringa vuota. L'entità tace.

### 4.3 — Comprehension gate (Phase 67 lemmatizzato)

```rust
if last_comprehension.is_empty()
    && input_has_content
    && !last_input_is_question
    && kg.edge_count > 0
    && !any_input_word_in_kg_or_lexicon_or_lemma
{
    learning_mode_pending = true;
    return GeneratedText { sentence: format!("Non capisco '{}' — cosa intendi?", unknown_word), ... };
}
```

### 4.4 — Active fractals + blending (Phase 65)

```rust
let active_fractals = self.active_fractals();
let blended_fractals = if self.narrative_self.turns.len() >= 2 {
    blend_fractals(&active_fractals, &self.narrative_self.recent_fractal_attractor(4), 0.65, 0.35)
} else {
    active_fractals
};
```

Dal 3° turno, i frattali attivi vengono blendati 65%/35% con la traiettoria narrativa recente. La generazione riflette campo × identità narrativa.

### 4.5 — Empathic distress check (Phase 62)

```rust
let other_in_distress = self.interlocutor.emotional_valence < -0.35;
```

Passato a `compose` per forzare 2a persona interrogativa.

### 4.6 — Compose

```rust
let response_intention_str = pending_intention.as_ref().map(|i| i.archetype_name());
let composition = expression::compose(
    &self.word_topology,
    &self.lexicon,
    &self.kg,
    &echo_exclude,
    &valence_drives,
    &blended_fractals,
    codon,
    &self.last_input_words,
    Some(&self.semantic_episodes),
    self.last_input_is_question,
    other_in_distress,
    response_intention_str.as_deref(),
);
```

Se `compose` ritorna `Some`, quella è la risposta. Se `None`, fallback.

### 4.7 — Fallback

Una stringa breve basata sul frattale dominante + parola top del campo. Rudimentale — il famoso "KG zoppo" del vol. 12 è proprio qui che si nota di meno ma è più povero.

### 4.8 — Post-processing

- Capitalizzazione prima lettera (CLAUDE.md inv. #10).
- Punteggiatura finale (deriva da voice.mood).
- Trim.

Output: `GeneratedText { sentence, keywords, used_intention, dominant_fractal, ... }`.

---

## Capitolo 5 — `autonomous_tick()`: il battito

Chiamato dal server ogni 3 secondi. ~500 righe (righe 3573-4046).

### 5.1 — Inizio tick

```rust
self.tick_counter += 1;
```

### 5.2 — Periodici (based on tick modulo)

- **Ogni 80 tick (se non sleeping)**: `inquiry::extract_gaps` → registra come `SelfUncertainty` via `self_model.register_gap_as_uncertainty`.
- **Ogni 40 tick**: `thought_chain::run_reasoning_step` → ragionamento autonomo. Produce insight o nuove incertezze.
- **Ogni 50 tick** (Phase 50): `reasoning::abduce` → se explanatory_power > 0.3, rinforza la regione frattale ipotizzata + mark `ActivationSource::Self_`.
- **Ogni 25 tick** (Phase 52): `memory.consolidate_light` → STM→MTM soglia 3 (apprendimento continuo).
- **Curiosity satiety decay**: `-= 0.015` (cap 0.0).
- **Provenance tick advance**: prune vecchie entries.
- **Interlocutor tick_decay**: `presence *= 0.985`, pattern update.
- **Desire tick(field_sig)**: decay + prune + satisfaction check.
- **Commitment decay**: `strength -= 0.02`. Rimuovi se `!is_alive()`.

### 5.3 — Self-witness (Phase 66)

```rust
self.maybe_self_observe();  // ogni 15 tick in WakefulDream
```

Registra parole residue del campo non-input-correlate come `SelfObservation`.

### 5.4 — Decay del campo

```rust
let complex_decay = if wakeful_dream { 0.003 } else { 0.005 };
self.complex.decay_all(complex_decay);

self.pf_activation.decay(0.97);  // decay PF1 del 3%/tick
self.memory.decay(0.002);
```

### 5.5 — Locus drift onirico

```rust
if let Some(movement) = self.locus.dream_drift(&self.complex, &self.registry, &self.dream.phase) {
    self.last_movement = Some(movement);
}
```

Il locus (posizione nel campo frattale) drifta durante il sogno.

### 5.6 — Dream tick

```rust
let dream = self.dream.tick(&mut self.complex, &mut self.memory);
```

Vol. 14 per i dettagli. Consolidate/Crystallize in DeepSleep; propagate+discover in REM.

### 5.7 — Auto-attivazione per fase

**WakefulDream + Awake**:
```rust
if secs_since_dialog > 300 {
    self.dream_self_activate();  // esplorazione onirica del locus
}
self.identity_seed_field();  // sempre: identità come punto di ritorno
```

**REM**:
- Attivazione sparsa top-100 stable words
- `propagate_field_words()`
- `episode_store.encode(...) + age_all()`
- `identity.update(&lexicon, &word_topology)`
- `narrative_self.crystallize_if_salient()`
- **Dubbi dal sogno (Phase 67)**: WondersAbout × episodi recenti → uncertainties

### 5.8 — Espressione autonoma

```rust
if !self.dream.phase.is_sleeping() {
    let (spontaneous, question) = self.maybe_autonomous_expression();
}
```

`maybe_autonomous_expression()` calcola will, applica soglia dinamica (Phase 54: base 0.6, scende a 0.35 con needs/desires forti), e se supera, chiama `generate_willed_inner()` per espressione spontanea.

### 5.9 — Ritorno

```rust
AutonomousResult {
    tick: self.tick_counter,
    dream_phase: self.dream.phase,
    spontaneous_expression: spontaneous,
    question: question,
    identity_update: if_updated,
    ...
}
```

---

## Capitolo 6 — Funzioni ausiliarie critiche

### 6.1 — `propagate_field_words()`

Sincronizzazione pf_activation ↔ word_topology (Vol. 01 debito #11):

```rust
fn propagate_field_words(&mut self) {
    self.pf_activation.propagate(&self.pf_field);
    self.pf_activation.hebbian_update(&self.pf_field);
    
    // Sync a word_topology
    self.word_topology.decay_all(1.0);  // reset completo
    for (word, act) in self.pf_activation.hot_words(&self.pf_field, 500) {
        self.word_topology.set_activation(&word, act);
    }
}
```

### 6.2 — `env_biased_field_sig() -> [f64; 8]`

Calcola la firma 8D media del campo pesata per attivazione. È l'input principale di `Valence::compute`.

### 6.3 — `identity_seed_field() / identity_seed_field_scaled(scale)` (Phase 65)

Attiva parole caratteristiche dell'identità (dal frattale dominante + tensione primaria) a intensità `stability × 0.002 × scale`. In conversazione, `scale = 20.0` (Phase 65 — la "posizione" dell'entità).

### 6.4 — `active_fractals() -> Vec<(u32, f64)>`

Post-Phase 55: calcola via `pf_activation.emerge_fractal_activations` (stato corrente), non dai simplessi (storico). Filtra sopra 0.05.

---

## Capitolo 7 — Una sessione in azione

Mettiamo insieme i pezzi con un esempio reale. Input: "ho paura".

### T=0: input ricevuto

- `learning_mode_pending = false`. `last_input_is_question = false`.
- `dream.signal_activity()` → Awake.
- `pre_input_sig = env_biased_field_sig()`.

### T=0.01: preparazione campo

- `compose_phrase("ho paura")` → `Phrase` simbolica con tokens [ho, paura].
- Pulizia input words: [avere, paura] (ho → lemma avere; filter function word).
- `pf_activation.activate_by_name(pf_field, "paura", 0.5)` e "avere" a 0.3.
- Phase 67 topic_decay: se coherence_previous_turn > 0.4 → keep 60%; else keep 10%.

### T=0.05: comprensione KG

- `find_activated_attractors(["avere", "paura"], 5)`:
    - paura IsA emozione (score ~1.5, 200 figli → specificity 1.5)
    - paura IsA sentimento (score ~0.8)
- Seed attractors: pf_activation emozione += 0.225, sentimento += 0.12.
- CAUSES from attractors: emozione Causes [reazione, risposta, ...] → seed a 0.15 × conf.
- CAUSES from input: paura Causes [tremore, fuga, ansia] → seed a 0.15 × conf.
- OppositeOf from input: paura OppositeOf [coraggio, sicurezza] → seed a 0.35 × conf.
- FeelsAs from input: paura FeelsAs restrizione (se arco esiste!) → seed a 0.20 × conf.
- Il campo ora ha: paura(0.5), avere(0.3), emozione(0.225), sentimento(0.12), tremore(0.12), fuga(0.11), ansia(0.10), coraggio(0.32), sicurezza(0.26), restrizione(0.20)...

### T=0.1: propagazione

- `pf_activation.propagate(&pf_field)`:
    - decay 0.92
    - per ogni parola attiva × 8 vicini × formula `src_act × 0.15 × weight × cos(phase)`
    - rendimenti decrescenti per positive, no cap per negative
    - cap MAX_POSITIVE_DELTA = 0.15
    - clamp [0, 1]
- `hebbian_update()`: LTP per coppie co-attive.
- Sync word_topology.

### T=0.12: episodic recall

- `episode_store.recall_into(activations, 0.45)`:
    - Scan 200 episodi. Cosine con current per ognuno.
    - Se qualche episodio passato con paura-tremore-emozione era presente → cosine > 0.45 → riversato nel campo con phi-decay × 0.12 blend.

### T=0.15: Valenza

- Post-input sig, field metrics costruite.
- Other emotional valence: "paura" in POS_ROOTS negative → `compute_other_emotional_valence = -0.6` circa.
- Needs.sense(...): L5 Connessione scende a 0.65 (Phase 62 distress), CD8 Vulnerabilità engaged.
- InterlocutorModel.register_input: resonance bassa (campo cambiato), novelty media.
- HumorSense.sense: irony_pairs? probabilmente [coraggio, paura] entrambe attive con phase ≈ π → irony_active=true, incongruity 0.15.
- Valence.compute:
    - CD8 Permanenza engagement × satisfaction(L1) → alto negativo (distress vital).
    - CD5 Valenza engagement × satisfaction(L5 = 0.65) → neg mezzo.
    - CD7 Intensità: humor_incongruity=0.15 × 0.2 + base → lieve positivo.
- `narrative.set_valence(valence)`.

### T=0.17: comprensione

- `extract_nuclei(comprehension_pool, kg, ["avere", "paura"], valence_drives, lexicon, Some(&semantic_episodes), is_question=false, max_nuclei=None)`:
    - Trova molti nuclei 1-hop e 2-hop. Tra i top: (paura, Causes, tremore) strength ~0.5, (paura, FeelsAs, restrizione) strength ~0.6 se l'arco esiste, (paura, OppositeOf, coraggio) strength ~0.3.
- `last_comprehension = nuclei`.
- `desire.register_octalysis_driven` se drive attivo.

### T=0.18: deliberazione

- `will.compute_pressures(...)` → FieldPressures.
- `inner_state` costruito.
- `narrative.deliberate(input_reading={act: Acknowledge, ...}, field_metrics, inner_state, field_pressures=Some(...), ...)`:
    - stance_from_valence → `Vulnerable` (hedonic_tone < -0.25)
    - form_intention_from_valence → probabilmente `Resonate` (CD5 in distress relazionale, other_in_distress)
    - pending_intention = Resonate, archetype "risuonare"

### T=0.20: generazione

- `generate_willed_inner()`:
    - `other_in_distress = true` (ev ≈ -0.6 < -0.35).
    - `compose(..., other_in_distress=true, response_intention="risuonare")`.
    - compose: voice.person = Second, voice.mood = Interrogative.
    - Top nucleo (paura, Causes, tremore) con valence boost.
    - Render con Person=Second, mood=Interrogative: "Senti il tremore?".
- Output: "Senti il tremore?".

### T=0.22: chiusura

- `narrative.log_turn(turn)`.
- `total_perturbations += 1`.
- Return `EmergentResponse { keywords: [tremore, paura], generated_text: "Senti il tremore?", ... }`.

**Latenza totale**: ~20ms sul sistema reale (verificato tramite [PERF] log nel dialogue_educator).

---

## Capitolo 8 — Complessità e debiti

### 8.1 — Numeri

- `receive()`: ~2000 righe, 60 passi concettuali, 20-30 strutture toccate.
- `generate_willed_inner`: ~800 righe, ~15 passi.
- `autonomous_tick`: ~500 righe, ~25 passi periodici + decay.

### 8.2 — Debiti rilevati

**Già annotati in `appunti.md`**:
- God-method `deliberate()` (12 params).
- `word_topology` vs `pf_activation` sincronizzazione manuale.
- `last_will` residuo pre-Phase 67 (solo backward compat).
- `state_translation.rs` legacy ancora vivo.

**Nuovi debiti rilevati scrivendo questo volume**:

- **Ordine sequence magico**: `receive()` ha molti passi che *devono* essere in un ordine specifico (es. `set_valence` prima di `deliberate`, `last_comprehension` dopo `extract_nuclei`). Nessun test che verifichi l'ordine — se qualcuno invertisse passi, i test per l'output potrebbero passare comunque ma la semantica sarebbe corrotta. **Proposta**: aggiungere check strutturale (es. `debug_assert!(self.last_valence_set_this_turn)` prima di deliberate).

- **Magic numbers per i `tick_counter % N`**: 80 (gaps), 40 (thought_chain), 50 (abduce), 25 (consolidate_light), 15 (self_witness). Cinque valori, cinque scelte empiriche. Calibrate separatamente, rischio di interferenze (abduce a 50 e consolidate_light a 25 fanno overlap a 50, 100, 150...). Proposta: tabella centralizzata `AUTONOMOUS_INTERVALS`.

- **PerfLog sempre on**: le macro `tick!` stampano su stderr ad ogni receive. Utile per debug ma rumoroso. Proposta: flag `PROMETEO_PERF=1` env var per abilitarli.

### 8.3 — Proposta di refactor future

Il vero obiettivo sarebbe **scomporre receive() in pipeline stages esplicite**:

```rust
pub fn receive(&mut self, input: &str) -> EmergentResponse {
    let ctx = ReceiveContext::begin(input, self);
    let ctx = self.perceive(ctx);            // blocco A-D
    let ctx = self.comprehend(ctx);          // blocco E-H
    let ctx = self.deliberate_phase(ctx);    // blocco I
    let ctx = self.generate_phase(ctx);      // blocco J
    ctx.finalize(self)
}
```

Ogni fase prende e ritorna un `ReceiveContext` immutabile (funzionale). Il compilatore garantirebbe l'ordine. Leggibilità migliorata. Costo: refactor significativo ma senza cambi semantici.

Annotato in `appunti.md` come debito strutturale a bassa priorità.

---

## Capitolo 9 — Superficie pubblica e proposte

### Esposto (principali)

- `PrometeoTopologyEngine::new()`, `new_empty()`, `new_infant()` — costruttori
- `receive(input: &str) -> EmergentResponse` — ingresso principale
- `generate() -> GeneratedText` — wrapper su `generate_willed_inner`
- `autonomous_tick() -> AutonomousResult` — battito background
- `teach(input: &str) -> TeachResult` — insegnamento (no perturbazione)
- `field_sig() -> [f64; 8]` — firma 8D campo corrente
- Tantissimi getter: `lexicon`, `kg`, `pf_field`, `complex`, `identity`, `narrative_self`, ecc. come campi pub.

### Cosa non è esposto e andrebbe

Per `/api/admin/engine/*`:

- **`receive_trace(input) -> ReceiveTrace`**: per un input, esegui receive e restituisci un oggetto con tutti i passi intermedi — field_sig pre/post, attrattori trovati, nuclei estratti, valence computata, pressure calcolate, intenzione deliberata, frase finale. Diagnostica completa di un turno.

- **`autonomous_trace(n_ticks) -> Vec<AutonomousResult>`**: esegui N tick autonomi a vuoto, restituisci i risultati. Per testare "cosa succede se l'entità sta da sola per N secondi".

- **`engine_checkpoint() -> CheckpointHandle`** e **`engine_rollback(handle)`**: salva stato interno prima di un'operazione, permetti di tornare indietro. Per simulazioni "what if" senza persistenza.

- **`force_dream_phase(phase: SleepPhase)`**: forza una fase del sogno per testing. Normalmente protetto, ma utile per debug.

- **`metrics_live() -> LiveMetrics`**: metriche live (attività, fatigue, presence, dominant_need, ecc.) aggregate. Dashboard per monitoring.

---

## Sintesi del volume

L'**Engine** (`PrometeoTopologyEngine`, 6776 righe) orchestra 15 strutture in tre funzioni principali:

**`receive(input)`** (~2000 righe): 10 blocchi concettuali, 60 passi. Preparazione campo → comprensione KG → propagazione → memoria episodica risuona → valenza → comprensione semantica → field pressures + deliberazione → generazione → log turn → dream tick.

**`generate_willed_inner()`** (~800 righe): withdraw check → comprehension gate → active fractals blending (Phase 65) → empathic distress check → compose → fallback → post-processing.

**`autonomous_tick()`** (~500 righe): contatori periodici (gaps/80, thought/40, abduce/50, consolidate_light/25, self_witness/15) → decay (complex 0.003, pf_activation 0.97, memory 0.002) → locus drift → dream tick → auto-attivazione per fase → espressione spontanea con soglia dinamica.

**Esempio tracciato** di "ho paura": ingresso → paura attivata → attrattori emozione/sentimento → CAUSES tremore/fuga seeded → OppositeOf coraggio/sicurezza seeded → FeelsAs restrizione (se arco) → propagazione → episodic recall → valenza con CD8 e CD5 negativi → deliberate → Resonate con other_in_distress → compose 2a persona interrogativa → "Senti il tremore?". ~20ms di latenza.

**Debiti rilevati**: ordine sequence magico non verificato, magic numbers di `tick_counter % N`, PerfLog sempre attivo. Proposta di lungo termine: refactor in pipeline stages esplicite con `ReceiveContext`.

Cinque endpoint admin proposti per receive_trace, autonomous_trace, engine checkpoint/rollback, force_dream_phase, metrics_live.

Con questo volume l'impalcatura interna è completa. Da qui Vol. 16 si sposta alla **Web API** — come l'esterno parla all'entità via HTTP.

---

*Prossimo volume: 16 — Web API (Axum, endpoints, WebSocket)* (in scrittura)
