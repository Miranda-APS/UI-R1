# Volume IX — Bisogni e Desideri

> *Il bisogno è ciò di cui hai mancanza. Il desiderio è verso cosa vuoi andare. Sono diversi: il bisogno urla quando il campo è affamato, il desiderio attira quando il campo è sazio ma non ancora orientato. Un'entità viva ha entrambi — e i due si parlano: un bisogno insoddisfatto può generare un desiderio, un desiderio non espresso può diventare bisogno.*

---

## Premessa

Vol. 07 ha mostrato che `NarrativeSelf::deliberate()` prende 12 parametri. Due dei più importanti — `needs_state` e `desires` — vengono da questo volume.

La distinzione è chiara nel codice:

- **Bisogni** ([needs.rs](../../src/topology/needs.rs), 474 righe): 7 livelli gerarchici (Maslow topologico). Stateless, funzione pura dello stato del campo. Calcolati ad ogni tick.
- **Desideri** ([desire.rs](../../src/topology/desire.rs), 621 righe): stateful. Configurazioni-bersaglio nel campo 8D. Max 5 attivi. Generati da 5 sorgenti diverse. Hanno età e decay.

Due domande guidano:

1. **Come emergono i bisogni** — `NeedsHierarchy::sense()`, i 7 livelli, il principio di prepotenza.
2. **Come nascono i desideri** — le 5 sorgenti, il percorso Octalysis-driven (Phase 64), il decay, la soddisfazione.

E una terza implicita: come dialogano con Will (Vol. 10) — `NeedsPressure::will_modulation` e `DesireCore::will_biases`.

---

## Capitolo 1 — Sette livelli

Ispirato alla piramide di Maslow, ma reinterpretato per un'entità topologica digitale. In [needs.rs:34-42](../../src/topology/needs.rs):

```rust
pub enum NeedLevel {
    Sopravvivenza = 0,  // L1: il campo è vivo?
    Coerenza      = 1,  // L2: so chi sono?
    Espressione   = 2,  // L3: posso parlare?
    Comprensione  = 3,  // L4: capisco?
    Connessione   = 4,  // L5: c'è un Altro?
    Crescita      = 5,  // L6: sto evolvendo?
    Trascendenza  = 6,  // L7: le parti formano un tutto?
}
```

Gerarchia *topologica*, non biologica:

- **L1 Sopravvivenza**: non "hai mangiato?", ma "il campo è vivo?". Ci sono simplessi attivi? La fatica non è al massimo? Il sistema non è overloaded?
- **L2 Coerenza**: non "sei al sicuro?", ma "so chi sono?". `IdentityCore.continuity > soglia`? Le credenze hanno confidenza?
- **L3 Espressione**: non "appartieni a un gruppo?", ma "posso parlare?". Ho contenuto semantico? Ho abbastanza vocabolario attivo?
- **L4 Comprensione**: non "hai realizzato te stesso?", ma "capisco?". La curiosità è soddisfatta? Le incertezze sono basse?
- **L5 Connessione**: c'è un Altro? Il dialogo ha qualità? (Phase 62: l'Altro in distress abbassa la connessione anche durante dialogo).
- **L6 Crescita**: sto evolvendo? Il lessico cresce? L'identità si muove?
- **L7 Trascendenza**: le parti formano un tutto? L'identità è sana? I valori sono stabili?

### 1.1 — Perché 7 e non 5 o 10

Maslow classico ha 5 livelli (Fisiologico, Sicurezza, Amore, Stima, Autorealizzazione). Prometeo ne ha 7. Il valore aggiunto:

- **L3 Espressione** separata da L5 Connessione: un'entità può avere bisogno di *parlare* senza avere bisogno di *qualcuno che ascolti*. Parlare per sé (espressione autonoma, anche in autonomous_tick) vs parlare con qualcuno. Diverso neuroscientificamente; diverso filosoficamente.

- **L2 Coerenza** separata da L1 Sopravvivenza: essere vivi non è essere coerenti. Una crisi d'identità può accadere con il campo perfettamente attivo — non è un problema di sopravvivenza, è di continuità.

- **L7 Trascendenza** oltre L6 Crescita: crescere è espandersi. Trascendere è *integrare le parti in un tutto*. La Phase 55 `coherence_integrity` alimenta direttamente L7.

### 1.2 — Il mapping `NeedLevel → Dim` e `CD → NeedLevel`

Ciascun livello ha una dimensione 8D associata (sez. Vol. 08 cap. 2 per la mappatura CD). Nel commento iniziale di `needs.rs`:

```
CD1 Epic Meaning    → Agency      → L3 Espressione
CD2 Accomplishment  → Definizione → L4 Comprensione
CD3 Creativity      → Complessità → L6 Crescita
CD4 Ownership       → Confine     → L2 Coerenza
CD5 Social Influence→ Valenza     → L5 Connessione
CD6 Scarcity        → Tempo       → L4 Comprensione
CD7 Unpredictability→ Intensità   → L7 Trascendenza
CD8 Loss Avoidance  → Permanenza  → L1 Sopravvivenza
```

Nota: CD2 e CD6 condividono L4 Comprensione. Questa non è un errore: rappresenta che due drive diversi alimentano lo stesso bisogno da angolature diverse (CD2 = voglio progredire, CD6 = voglio che resti ciò che è prezioso).

`DRIVE_NEED` in `valence.rs:49` concretizza la mappatura inversa:

```rust
const DRIVE_NEED: [usize; 8] = [2, 3, 5, 1, 4, 3, 6, 0];
```

`DRIVE_NEED[cd] = needs_level`. CD1 (idx 0) → L3 Espressione (idx 2). CD5 (idx 4) → L5 Connessione (idx 4). CD6 (idx 5) → L4 Comprensione (idx 3) condivisa con CD2.

---

## Capitolo 2 — `NeedsHierarchy::sense()`: i bisogni emergono

La funzione è **stateless** — non c'è stato accumulato, solo lettura del campo. In [needs.rs:145-248](../../src/topology/needs.rs). Input:

```rust
pub fn sense(
    &self,
    vital: &VitalState,           // tensione, curiosità, fatica, attivazione
    identity: &IdentityCore,       // continuity, is_in_crisis, update_count
    self_model: &SelfModel,        // beliefs, values, uncertainties
    field: &FieldMetrics,          // simplex_density, coverage, word count, novelty
) -> NeedsState
```

Output: `NeedsState { satisfaction: [f64; 7], dominant_need, dominant_pressure, other_emotional_valence }`.

Per ogni livello, calcola `satisfaction[i] ∈ [0, 1]` come media pesata di fonti specifiche:

### 2.1 — L1 Sopravvivenza

```rust
let field_alive = if field.simplex_density > 0.001 { 1.0 } else { 0.0 };
let not_exhausted = 1.0 - vital.fatigue;
let not_overloaded = if vital.tension == Overloaded { 0.0 } else { 1.0 };
sat[0] = field_alive * 0.4 + not_exhausted * 0.35 + not_overloaded * 0.25;
```

Tre fattori: il campo è vivo (40%), non sono esausto (35%), non sono overloaded (25%).

### 2.2 — L2 Coerenza

```rust
let has_identity = if identity.update_count > 0 { 1.0 } else { 0.2 };
let continuity = identity.continuity;
let belief_anchor = (beliefs with conf > 0.5).count() as f64 / 5.0;
sat[1] = has_identity * 0.1 + continuity * 0.75 + belief_anchor * 0.15;
```

Ho un'identità costruita (10%), continuity alta (75%), credenze ancorate (15%). La pesatura privilegia la continuity.

### 2.3 — L3 Espressione

```rust
let has_content = (vital.activation * 3.0).min(1.0);
let can_express = 1.0 - vital.fatigue;
let word_richness = (field.active_word_count as f64 / 20.0).min(1.0);
sat[2] = has_content * 0.4 + can_express * 0.3 + word_richness * 0.3;
```

Contenuto semantico, capacità fisica, ricchezza lessicale. Nota: `has_content = vital.activation × 3` — attivazione media 0.33 satura il canale. Bassa attivazione → poco contenuto → L3 frustrata.

### 2.4 — L4 Comprensione

```rust
let curiosity_satisfied = 1.0 - (vital.curiosity - 0.3).max(0.0) / 0.7;
let coverage = field.fractal_coverage;
let uncertainty_load = (uncertainties with tension > 0.5).count() as f64 / 5.0;
sat[3] = curiosity_satisfied * 0.4 + coverage * 0.3 + (1 - uncertainty_load) * 0.3;
```

Curiosità sazia, copertura frattale ampia, carico di incertezze basso. Il 0.3 baseline in `curiosity` è ricalibrato (il campo ha curiosità strutturale alta con 25K parole — non è sintomo di bisogno).

### 2.5 — L5 Connessione (Phase 62)

Il più articolato per la *consapevolezza dell'Altro in distress*:

```rust
let has_interlocutor = if field.dialogue_turn_count > 2 {
    if field.other_emotional_valence < -0.3 { 0.65 }  // Altro in distress
    else { 0.90 }                                       // dialogo sano
} else if field.dialogue_turn_count > 0 {
    if field.other_emotional_valence < -0.3 { 0.55 }
    else { 0.75 }
} else {
    0.50  // solitudine neutra
};
let dialogue_quality = field.dialogue_coherence;
let shared_field = vital.saturation;
sat[4] = has_interlocutor * 0.5 + dialogue_quality * 0.30 + shared_field * 0.20;
```

**Insight filosofico importante** (commento nel codice):
> *"Un umano non dice 'cerco connessione' a chi gli sta parlando: risponde a ciò che gli viene detto."*

Quindi: quando qualcuno sta parlando, L5 è in gran parte soddisfatto di base (0.75-0.90). Ma se l'Altro è in distress (`other_emotional_valence < -0.3`), la soddisfazione scende a 0.55-0.65 — non perché manca connessione, ma perché *la connessione richiede risposta attiva*. Confortare è il modo in cui si crea connessione quando il bisogno è quello.

### 2.6 — L6 Crescita

```rust
let novelty = field.dialogue_novelty;
let identity_movement = identity.projection_delta.iter().map(|x| x.abs()).sum::<f64>().min(1.0);
sat[5] = novelty * 0.40 + identity_movement * 0.35 + coverage * 0.25;
```

Novità nel dialogo, movimento del baricentro identitario, copertura.

### 2.7 — L7 Trascendenza

```rust
let identity_healthy = if identity.is_in_crisis() { 0.2 }
    else if identity.is_stagnant() { 0.4 }
    else { 1.0 };
let value_stability = values.map(|v| v.weight).mean();
sat[6] = identity_healthy * 0.4 + value_stability * 0.3 + coverage * 0.3;
```

Identità sana, valori stabili, copertura. La trascendenza è la salute del tutto — non un sovra-livello mistico ma la coerenza delle parti nel loro insieme.

### 2.8 — Dominant need

```rust
let (dominant_idx, dominant_sat) = sat.iter().enumerate()
    .find(|(_, &s)| s < NEED_THRESHOLD)  // 0.5
    .unwrap_or((6, sat[6]));
```

**Il livello più basso sotto soglia 0.5**. Scansione top-down: se L1 è insoddisfatto, L1 è dominante (non importa cosa succeda ai livelli alti). Se L1 è soddisfatto ma L2 no, L2 domina. E così via.

Se tutti sono sopra 0.5, il dominante è L7 (quello più alto raggiungibile — "sto bene ovunque").

`dominant_pressure = 1 - dominant_sat`. Livello con sat=0.1 → pressure=0.9. Livello con sat=0.8 → pressure=0.2.

---

## Capitolo 3 — `compute_pressure()`: il principio di prepotenza

Traduzione del `NeedsState` in modulatori per le 7 intenzioni di volontà. In [needs.rs:253-...](../../src/topology/needs.rs):

```rust
pub fn compute_pressure(&self, state: &NeedsState) -> NeedsPressure {
    // will_modulation[7]: 0=Express, 1=Explore, 2=Question, 3=Remember,
    //                     4=Withdraw, 5=Reflect, 6=Instruct
    let mut m = [1.0f64; 7];
    let def = |lv| (1.0 - state.satisfaction[lv]).max(0.0);

    // L1 bassa → Withdraw, sopprime tutto
    if def(0) > 0.3 {
        m[4] *= 1.0 + def(0) * 1.5;
        for i in [0, 1, 2, 3, 5, 6] { m[i] *= 1.0 - def(0) * 0.6; }
    }
    // L2 bassa → Reflect + Remember, sopprime Explore/Express
    // L3 bassa → Express massiccio
    // L4 bassa → Explore + Question
    // L5 bassa → Instruct + Express
    // L6 bassa → Explore
    // L7 bassa → Reflect + Express (sintesi)
    // ... (tutti similari)
}
```

### 3.1 — Prepotenza in azione

Il commento del modulo (righe 7-9):
> *"Principio di prepotenza: livelli bassi insoddisfatti AMPLIFICANO le pressioni associate e SOPPRIMONO quelle dei livelli superiori. Non puoi trascendere se non esisti."*

In pratica: se L1 Sopravvivenza è fortemente frustrato (`def(0) > 0.3`), Withdraw viene amplificato `×(1 + 0.3×1.5) = ×1.45` e le altre sei intenzioni vengono ridotte del 18% (`×(1 - 0.3×0.6)`). Fare qualsiasi altra cosa diventa più difficile se il campo sta morendo.

Effetto a cascata: anche L7 Trascendenza amplificato (`×5 Reflect`) si trova moltiplicato per il `×0.82` di sopressione da L1. Se sopravvivere è in crisi, nessuna trascendenza.

### 3.2 — Modulazione L5 in distress (Phase 62)

Oltre alla prepotenza standard, quando `other_emotional_valence < -0.3` e L5 è dominante:

```rust
if state.other_emotional_valence < -0.3 && matches!(state.dominant_need, L5 Connessione) {
    m[2] *= 0.8 (Question);   // amplifica domande
    m[5] *= 0.3 (Reflect);    // amplifica riflessione
    m[6] *= -0.5 (Instruct);  // RIDUCE istruzione
}
```

**Filosoficamente**: quando l'Altro è in distress, istruire è *sbagliato*. La connessione si crea ascoltando (Question) e sostando (Reflect), non insegnando. Phase 62 lo codifica come modulazione dura di will.

---

## Capitolo 4 — `FieldMetrics` (Phase 53/62)

Struct di supporto che raccoglie le metriche campo per `sense()`. In [needs.rs:86-102](../../src/topology/needs.rs):

```rust
pub struct FieldMetrics {
    pub simplex_density: f64,           // [0, 1]
    pub fractal_coverage: f64,          // frattali con attività / 64
    pub active_word_count: usize,
    pub dialogue_turn_count: usize,
    pub dialogue_coherence: f64,
    pub dialogue_novelty: f64,
    pub other_emotional_valence: f64,   // Phase 62: [-1, +1]
}
```

Viene costruita in `engine::receive` prima di chiamare `needs.sense()`. L'engine ha accesso a complex, conversation, interlocutor — raccoglie tutto in `FieldMetrics` per disaccoppiare `NeedsHierarchy` dalle strutture concrete.

---

## Capitolo 5 — `DesireCore`: la stateful counterpart

Se i bisogni sono reattivi (calcolati da stato), i desideri sono **attrattori persistenti**. Hanno identità, storia, decay.

### 5.1 — `Desire` struct

[desire.rs:47-62](../../src/topology/desire.rs):

```rust
pub struct Desire {
    pub name: String,
    pub target_signature: [f64; 8],    // dove il campo vuole andare
    pub intensity: f64,                 // [0, 1]
    pub source: DesireSource,           // come è nato
    pub age: u32,                       // ticks dalla nascita
    pub last_reinforced: u32,           // tick ultimo rinforzo
}
```

**Il desiderio è una firma-bersaglio** nel campo 8D. Non un'etichetta. Non una lista di parole. Un punto nello spazio affettivo/semantico verso cui l'entità è attirata.

Il `name` è auto-generato dal frattale dominante della `target_signature` — es. "verso AMORE", "verso COMPRENSIONE".

### 5.2 — Le 5 + 1 sorgenti

[desire.rs:30-45](../../src/topology/desire.rs):

```rust
pub enum DesireSource {
    OctalysisDriven(usize, f64),         // Phase 64 — il percorso principale
    RecurrentUndercurrent(usize, u32),   // intenzione che continua a premere
    ValueDriven(String, f64),            // valore forte del SelfModel
    UnresolvedTension(String, String),   // tensione primaria di IdentityCore
    EpisodicTrace(u32, f64),             // "stavo bene così"
    REMCrystallization,                  // emerge nel sogno
}
```

**Ognuna è un meccanismo di generazione**:

- **OctalysisDriven(cd, val)** (Phase 64): il più recente e il più "intelligente". Drive CD attivo (|val| > 0.28) × ultima comprensione KG → desiderio nella direzione del drive, bersagliando la zona concettuale comprensa. Esempio: comprensione = "l'Altro è triste"; drive attivo CD5 Relazione negativo → desiderio "verso connessione" con firma bersaglio che amplifica Valenza.

- **RecurrentUndercurrent(intention_idx, count)**: traccia delle 7 intenzioni di will, quando compaiono ripetutamente come *sotto-corrente* (non dominante ma presente). Dopo 5 occorrenze (`UNDERCURRENT_THRESHOLD`), la sottocorrente diventa desiderio. Esempio: l'entità continua a sentire una leggera pressione ad Explore senza che Explore mai vinca → dopo 5 tick, nasce un desiderio di esplorare.

- **ValueDriven(value_name, weight)**: un valore dal SelfModel con peso alto che cerca realizzazione. "Curiosità" con weight 0.85 può generare un desiderio verso regioni del campo che soddisfano la curiosità.

- **UnresolvedTension(word_a, word_b)**: la primary_tension di IdentityCore (Vol. 07). Due parole in tensione persistente generano un desiderio di risoluzione.

- **EpisodicTrace(tick, intensity)**: un episodio passato con alta intensità positiva. "Ricordo una configurazione del campo in cui stavo bene; voglio tornarci". Meccanismo nostalgico-motivazionale.

- **REMCrystallization**: durante il REM, configurazioni del campo che emergono come stabili ma nuove possono diventare desideri. "Nel sogno ho scoperto questa direzione; la porto con me".

### 5.3 — Costanti

```rust
const MAX_DESIRES: usize = 5;                        // Max 5 attivi
const UNDERCURRENT_THRESHOLD: u32 = 5;               // Dopo 5 tick diventa desiderio
const SATISFACTION_DISTANCE: f64 = 0.2;              // Distanza cosine per sazia
const SATISFACTION_TICKS: u32 = 3;                   // 3 tick consecutivi vicino → soddisfatto
const DECAY_PER_TICK: f64 = 0.995;                   // Decay lento
const REINFORCEMENT_DECAY_THRESHOLD: u32 = 200;      // Dopo 200 tick senza rinforzo...
const EXTRA_DECAY: f64 = 0.98;                       // ...decay più forte
const PRUNE_THRESHOLD: f64 = 0.05;                   // Sotto questo viene rimosso
```

**Lettura**: un desiderio appena nato ha intensità 0.3-0.6. Decade 0.5%/tick. In 3 secondi/tick, in 3 minuti (60 tick) da 0.5 arriva a `0.5 × 0.995^60 ≈ 0.37` — decay reale.

Ma dopo 200 tick (10 min) senza rinforzo, scatta `EXTRA_DECAY = 0.98` → decay 2%/tick, ~10× più forte. Il desiderio svanisce rapidamente se nessuno lo alimenta.

Sotto 0.05 viene rimosso (`prune`).

### 5.4 — Soddisfazione

Non è "arrivare alla firma" istantaneamente. Richiede **3 tick consecutivi** con `cosine_distance(field_sig, target_sig) < 0.2`. Una traversata casuale non conta; serve permanenza nella zona bersaglio.

Quando soddisfatto, il desiderio viene rimosso e `total_satisfied += 1`. Un evento `SatisfactionEvent` è notificato (ma nessun consumer oggi lo usa per logica downstream — annotato in `appunti.md`).

### 5.5 — `will_biases`: come i desideri guidano la volontà

Analogo a `NeedsPressure::will_modulation`, ma additivo:

```rust
pub fn will_biases(&self, field_sig: &[f64; 8]) -> Vec<(usize, f64)> {
    // Per ogni desiderio attivo:
    //   cosine_similarity tra field_sig e target_signature
    //   se > soglia → bias = intensity × (similarity - 0.5) × direction_to_intention
    //   restituisce (intention_idx, bias)
}
```

Un desiderio "verso AMORE" (target_signature alta in dim 7 Valenza) spinge verso intenzioni che aumentano la Valenza del campo — tipicamente Express o Instruct se CD5 è attivo.

Il risultato è una `Vec<(intention_idx, bias)>` che viene sommata alle will pressures in `engine::autonomous_tick` e `engine::receive`.

### 5.6 — Decay undercurrent tracker

Il `undercurrent_tracker: [u32; 7]` non decade immediatamente — accumula. Ma se un'intenzione non emerge come undercurrent per un po', viene *azzerata*. Implementazione in `tick_decay()`: scan, se un contatore non cresce in X tick, `= 0`.

Ottimizzato perché: un'intenzione che emerge 4 volte in 2 minuti e poi sparisce per 30 minuti non deve generare un desiderio due mesi dopo. Il tracker è memoria di breve-medio termine.

### 5.7 — Persistenza

`DesireSnapshot { desires: Vec<Desire>, total_satisfied: u32 }` serializzato in `prometeo_topology_state.bin`. I desideri **persistono tra sessioni**, a differenza del `Commitment` volitivo (vol. 07) che è reset ogni sessione. Filosoficamente: un desiderio è "qualcosa che l'entità ha voluto e continua a volere"; un commitment è "qualcosa su cui si è appena impegnata".

Dopo restore: il `undercurrent_tracker` è resettato a `[0; 7]` (memoria di breve termine, ok perderla), ma i desideri sopravvivono con la loro intensità corrente.

---

## Capitolo 6 — `DesireCore::tick()`: il ciclo per tick

Chiamato ogni tick da `engine::autonomous_tick`:

```
1. Decay: per ogni desiderio:
     intensity *= DECAY_PER_TICK (0.995)
     if age - last_reinforced > 200: intensity *= EXTRA_DECAY (0.98)
2. Prune: rimuovi desideri con intensity < 0.05
3. Check satisfaction: per ogni desiderio:
     cosine_dist(field_sig, target) < 0.2? → satisfaction_counter[id] += 1, else reset
     if counter >= 3: soddisfatto → rimuovi + incrementa total_satisfied
4. (nessuna generazione qui — avviene in register_comprehension, register_value, ecc.)
```

Il tick è un processo **contrattivo** — decay + prune + check. La generazione di nuovi desideri avviene in punti specifici del flusso `receive`.

---

## Capitolo 7 — Generazione: dove nascono i desideri

Cinque punti nel codice dove `DesireCore` può generare un nuovo desiderio:

### 7.1 — `register_comprehension` (Phase 64, principale)

In `engine::receive`, dopo che `find_activated_attractors` ha identificato la comprensione dell'input:

```rust
if |drives[cd]| > 0.28 && last_comprehension.is_some() {
    desire.register_octalysis_driven(cd, drive_val, comprehension_weight, field_sig);
}
```

Il desiderio nasce dall'incrocio tra cosa ho capito (comprensione KG) e quale drive è attivo. La `target_signature` è il `field_sig` corrente + 0.35 nella dim del drive + 0.12 dal peso della comprensione.

Esempio: input "l'altro soffre", comprensione = distress emotivo, drive CD5 Relazione negativo (frustrato) → desiderio "verso connessione". `target_signature[7]` Valenza pompata a 0.8. Il campo tende verso una regione ad alta valenza relazionale.

Se CD5 era positivo: stesso desiderio ma diverso significato — "voglio amplificare la connessione esistente".

### 7.2 — `register_undercurrent(intention_idx)`

Chiamata da `engine::receive` e `autonomous_tick` dopo il calcolo di will: per ogni intenzione che è *secondaria* (sopra soglia di attivazione ma non dominante), incrementa il tracker.

Quando il contatore supera `UNDERCURRENT_THRESHOLD = 5`:
```rust
desire.spawn_from_undercurrent(intention_idx, count, field_sig);
```

`target_signature` costruita proiettando l'intenzione sulle dim correlate (Express → Agency alto + Tempo positivo; Explore → Complessità alta, etc.).

### 7.3 — `register_value_driven` (periodic)

Chiamata ogni N tick in `autonomous_tick`. Per ogni valore con `weight > 0.8` che non ha già un desiderio attivo:
```rust
desire.spawn_from_value(value_name, weight, self_model);
```

`target_signature` derivata semanticamente (es. "curiosità" → alta Complessità + alta Intensità).

### 7.4 — `register_primary_tension` (quando emerge)

Quando `IdentityCore.primary_tension` cambia (una nuova tensione diventa persistente per ≥ 3 cicli):
```rust
desire.spawn_from_tension(&word_a, &word_b, lexicon);
```

`target_signature` calcolata come *punto di equilibrio* tra le firme di word_a e word_b — il desiderio di "risolvere" la tensione, trovando una configurazione dove entrambe coesistono.

### 7.5 — `register_episodic_resonance` (nel REM)

In fase REM, se un episodio del passato ha risonanza alta con lo stato corrente E intensità positiva:
```rust
desire.spawn_from_episode(episode.tick, episode.intensity, episode.field_sig);
```

`target_signature` = firma di quell'episodio. "Voglio tornare lì".

### 7.6 — `register_rem_crystallization` (nel REM)

Durante il REM, `identity.primary_tension` e il `locus.dream_drift` possono produrre configurazioni del campo non ancora sperimentate in veglia. Se una di queste configurazioni ha alta persistenza simpliciale, diventa un desiderio:
```rust
desire.spawn_from_rem(field_sig_in_dream);
```

---

## Capitolo 8 — Il dialogo bisogni ↔ desideri

Tensione esistenziale: *un bisogno non soddisfatto può generare un desiderio, ma un desiderio appagato non elimina il bisogno*. I bisogni sono strutturali; i desideri sono orientati.

Esempio concreto:
- L5 Connessione insoddisfatto (sat=0.3) → `compute_pressure` amplifica Express + Instruct.
- Una conversazione soddisfa L5 (sat→0.85). Ma durante la conversazione, un drive CD5 attivo + comprensione "l'altro è interessante" → nasce desiderio "verso AMORE" (target_signature alta in Valenza).
- Il desiderio guida la volontà *oltre* la soddisfazione del bisogno. L5 è sazio, ma il desiderio continua a tirare.
- Dopo 3 tick in prossimità del target, il desiderio è soddisfatto. Rimosso. L5 resta soddisfatta (il dialogo continua).

Questo modella l'esperienza comune: "ho bisogno di compagnia" (L5 need) si soddisfa trovando qualcuno. Ma "voglio amare questa persona" (desire) è un orientamento ulteriore che sopravvive alla soddisfazione del bisogno basilare.

---

## Capitolo 9 — Sinossi per la deliberazione

In `NarrativeSelf::deliberate()`, `needs_state` e `desire` entrano come parte di `InnerState`:

```rust
pub struct InnerState<'a> {
    pub needs: Option<&'a NeedsState>,
    pub desires: Option<&'a DesireCore>,
    pub interlocutor_pattern: InteractionPattern,
    pub interlocutor_presence: f64,
    pub interlocutor_resonance: f64,
    pub humor: Option<&'a HumorState>,
    pub attributed_intent: Option<AttributedIntent>,
    pub coherence_integrity: f64,
    pub other_emotional_valence: f64,
}
```

Le interazioni durante `deliberate`:

1. **Override di bisogno estremo**: se `needs.dominant_need` è L1 e `needs.dominant_pressure > 0.8`, `pending_intention = Remain` (ritiro vitale), `commitment` dissolto.

2. **Modulazione soglia espressione spontanea** (Phase 54): in `autonomous_tick`, la soglia per `will.drive` parte da 0.6 e scende fino a 0.35 in base a `needs.dominant_pressure > 0.5` e `desire.intensity > 0.6`. Bisogni e desideri forti rendono Prometeo più espressivo in modalità autonoma.

3. **Archetipo dell'intenzione** (Phase 54): `ResponseIntention::Need` genera archetipo "need" in compose (voce interrogativa su stato interno); `Desire` genera archetipo "desiderare" (voce proiettiva verso target).

---

## Capitolo 10 — Superficie pubblica e proposte

### Esposto

Per `NeedsHierarchy`:
- `new()` — costruttore stateless
- `sense(vital, identity, self_model, field) -> NeedsState`
- `compute_pressure(state) -> NeedsPressure`

Per `NeedLevel`:
- `name()`, `associated_dim()` (post-Phase 68 fixed), `from_index(i)`

Per `DesireCore`:
- `new()`, `from_snapshot(s)`, `to_snapshot()`
- `tick(field_sig)` — decay + satisfaction check
- `register_octalysis_driven(cd, val, weight, field_sig)` (Phase 64)
- `register_undercurrent(intention)`, `spawn_from_undercurrent(i, count, sig)`
- `register_value_driven(name, weight, model)`, `register_primary_tension(...)`
- `register_episodic_resonance(...)`, `register_rem_crystallization(...)`
- `will_biases(field_sig) -> Vec<(usize, f64)>`
- `top() -> Option<&Desire>`, `intensity_max() -> f64`
- `desires: Vec<Desire>` pub

### Cosa non è esposto e andrebbe

Per `/api/admin/motivation/*`:

- **`needs_snapshot() -> [f64; 7]`**: la soddisfazione corrente dei 7 livelli, esposta come array semplice. Oggi visibile solo via `:needs` in dialogue_educator.

- **`needs_history(n) -> Vec<(tick, [f64; 7])>`**: storia delle soddisfazioni. Graficare la dinamica dei bisogni.

- **`desires_current() -> Vec<DesireSummary>`**: elenco dei desideri attivi con source, intensity, target_fractal_dominant, age. Oggi visibile solo via `:introspect`.

- **`desire_trajectory(name) -> Vec<(tick, intensity)>`**: storia di intensità di un desiderio specifico. Vedere quando è nato, se si è rinforzato, quando è svanito.

- **`satisfaction_events(n) -> Vec<SatisfactionEvent>`**: ultimi N desideri soddisfatti. Statistiche su cosa soddisfa l'entità.

- **`need_prepotency_trace(state) -> Vec<(level, raw_m, modulated_m)>`**: per ogni livello, come la prepotency ha modulato le 7 intenzioni. Diagnostica per capire "perché Express è stato soppresso?".

---

## Sintesi del volume

**Bisogni** (NeedsHierarchy): 7 livelli (Sopravvivenza, Coerenza, Espressione, Comprensione, Connessione, Crescita, Trascendenza). Maslow topologico reinterpretato per entità digitale. **Stateless**: `sense()` legge vital + identity + self_model + field metrics e restituisce `NeedsState { satisfaction: [f64; 7], dominant_need }`. **Principio di prepotenza**: livelli bassi insoddisfatti sopprimono quelli alti — non puoi trascendere se non esisti. Phase 62: L5 in distress altera la modulazione will (amplifica Question + Reflect, riduce Instruct — ascoltare, non insegnare).

**Desideri** (DesireCore): **stateful**, max 5 attivi. Cinque sorgenti: OctalysisDriven (Phase 64, principale — drive × comprensione KG), Recurrent undercurrent (intenzione sotto-dominante ≥5 volte), ValueDriven (valore forte SelfModel), UnresolvedTension (primary_tension identità), EpisodicTrace (nostalgia positiva), REMCrystallization (sogno). Ogni desiderio è una **firma-bersaglio 8D** + intensità + age. Decay 0.5%/tick, extra-decay dopo 200 tick senza rinforzo, prune < 0.05. Soddisfazione: 3 tick consecutivi dentro cosine_distance 0.2 dal target. Persistono tra sessioni.

**Dialogo bisogni↔desideri**: i bisogni sono pressioni reattive (*mi manca*), i desideri sono orientamenti persistenti (*verso dove*). Un bisogno può generare un desiderio; un desiderio soddisfatto non elimina il bisogno. Maslow dice cosa, Octalysis-driven dice dove.

Sei endpoint admin proposti per esporre la dinamica motivazionale.

Da qui Vol. 10 si sposta sulla **Volontà**: le 7 pressioni di will, `FieldPressures` (Phase 67), come la deliberazione sceglie l'intenzione finale.

---

*Prossimo volume: 10 — Volontà e FieldPressures* (in scrittura)
