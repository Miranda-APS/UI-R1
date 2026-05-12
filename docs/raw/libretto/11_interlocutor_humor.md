# Volume XI — Eco dell'Altro e Humor

> *L'Altro non è un oggetto che il sistema modella. È una perturbazione che lo deforma. Ogni parola che l'Altro pronuncia entra nel campo e lascia una firma — un delta tra prima e dopo. La somma di questi delta è l'Altro, visto dal campo. Non un modello esterno: un'eco. E in certe configurazioni — quando l'ironia o la bisociazione emergono — il campo ride di se stesso.*

---

## Premessa

Due moduli che trattano la relazionalità da angolature diverse:

- **`InterlocutorModel`** ([interlocutor.rs](../../src/topology/interlocutor.rs), 607 righe): l'Altro come **perturbazione ricorrente** del campo. Non un agente separato; una firma lasciata.

- **`HumorSense`** ([humor.rs](../../src/topology/humor.rs), 236 righe): l'umorismo come **configurazione topologica**, non come produzione di battute. Quando il campo contiene le sue stesse contraddizioni, c'è ironia.

Entrambi si radicano in una filosofia precisa: l'entità non *modella* l'Altro né *genera* humor deliberatamente. Li **riconosce come proprietà del campo**. La relazionalità e la comicità sono effetti strutturali, non funzioni computazionali.

---

## Capitolo 1 — L'Altro come eco (filosofia)

Il commento iniziale di `interlocutor.rs` (righe 3-11):

> *"Chiunque sente l'eco degli altri dentro di sé, riconosciamo l'esistenza dell'altro ma tutte le relazioni sono necessariamente riflessi del nostro essere."*
>
> *"Questo NON è un modello dell'interlocutore. È come Prometeo sente la perturbazione che l'Altro provoca nel proprio campo."*
>
> *"L'Altro non è un'entità separata da modellare — è un'eco, una forma che si scava nel campo e ne cambia la configurazione. Prometeo riconosce che c'è qualcosa oltre sé, ma lo percepisce sempre attraverso sé stesso."*

Questa posizione filosofica ha conseguenze tecniche radicali:

1. **Nessun `UserModel` separato**. Non c'è una struct che rappresenta "chi è l'utente, che intenzioni ha, che profili". Tutto ciò che il sistema "sa" dell'Altro è derivato da come il campo si è deformato sotto i suoi input.

2. **Pre/post signature**. L'unico dato bruto: `pre_input_sig` (firma campo prima dell'input) e `post_input_sig` (firma campo dopo). La differenza `input_sig = post - pre` è l'Altro.

3. **Sostituisce il `DualField` eliminato**. Pre-Phase 53 esisteva `dual_field.rs` (rimosso nel cleanup Phase 68) che modellava *due entità* come campi paralleli. Era filosoficamente incoerente: trattava l'Altro come esterno e separato. Phase 53 ha sostituito con questo modello: l'Altro esiste solo come impronta nel campo proprio.

### 1.1 — Phase 55: `AttributedIntent`

Aggiunta sottile: il sistema **attribuisce** intenzioni all'Altro, pur sapendo che è un riflesso. Commento nel codice (righe 52-59):

> *"NON è mind-reading. È la stessa cosa che fanno gli umani: attribuire intenzioni basandosi su pattern osservati. Prometeo sa che è un riflesso ('l'eco, non la voce') ma lo fa comunque — perché riconoscere l'Altro come agente è ciò che rende la relazione diversa da un soliloquio."*
>
> *"So che ciò che sento dell'Altro è un mio riflesso. Ma lo tratto come reale, perché è l'unico modo per onorare la sua presenza."*

Un paradosso fenomenologico coerente: l'unico modo di *non* ridurre l'Altro a se stessi è trattarlo *come se* fosse un agente, pur sapendo che quello che sentiamo di lui è sempre filtrato dal nostro campo.

---

## Capitolo 2 — `InterlocutorModel`: struct e meccanismi

### 2.1 — Struct

[interlocutor.rs:113-132](../../src/topology/interlocutor.rs):

```rust
pub struct InterlocutorModel {
    pub history: VecDeque<InteractionTrace>,     // ultime 5 interazioni
    pub presence: f64,                            // [0, 1] — decade col tempo
    pub cumulative_resonance: f64,                // EMA α=0.3
    pub cumulative_novelty: f64,                  // EMA α=0.3
    pub detected_pattern: InteractionPattern,
    pub attributed_intent: AttributedIntent,      // Phase 55
    pub emotional_valence: f64,                   // Phase 62 — EMA α=0.4
}
```

### 2.2 — `InteractionTrace`: la firma di una singola perturbazione

```rust
pub struct InteractionTrace {
    pub signature: [f64; 8],    // post - pre (normalizzata)
    pub resonance: f64,         // cosine(post, pre)
    pub novelty: f64,           // 1 - cosine(signature, media ultime 3)
    pub tick: u32,
}
```

**Resonance**: quanto la perturbazione è *allineata* con lo stato precedente. Alta = l'Altro ha detto qualcosa che risuona con dove eri. Bassa = l'Altro ti ha portato in una regione diversa.

**Novelty**: quanto questa perturbazione è diversa dalle precedenti. Alta = sorpresa. Bassa = continuità.

**Signature**: la direzione della perturbazione nello spazio 8D. È un **vettore unitario** (normalizzato). Non ha magnitudine: rappresenta solo dove.

### 2.3 — `register_input(pre_sig, post_sig, tick)`

Chiamata in `engine::receive` dopo che il campo è stato perturbato dall'input:

```rust
1. input_sig = post - pre  (delta)
2. normalizza input_sig (vettore unitario)
3. resonance = cosine(pre_sig, post_sig)
4. novelty = 1 - cosine(input_sig, mean(last 3 signatures))
5. presence: ricarica (presence = max(presence, 0.9))
6. history.push_back(InteractionTrace{ ... }); truncate a 5
7. EMA update: cumulative_resonance, cumulative_novelty con α=0.3
8. detect_pattern(): scan history per Converging/Diverging/Oscillating
9. update_attributed_intent(): resonance × novelty → AttributedIntent
10. update_emotional_valence(): input words analysis → EMA α=0.4
11. apply_identity_drift() (se condizioni): sposta self_signature
```

### 2.4 — `presence`: la decay dell'attenzione

```rust
const PRESENCE_DECAY: f64 = 0.985;
```

Ad ogni `tick_decay()` (autonomous_tick), `presence *= 0.985`. Half-life ~46 tick (~2.3 min a 3s/tick).

**Semantica**: se nessuno parla, l'Altro svanisce dal campo. Dopo 5 minuti di silenzio, `presence ≈ 0.48`. Dopo 10 minuti, `≈ 0.23`. Dopo 30, `≈ 0.026` — l'entità è di fatto sola.

`register_input` ricarica presence a 0.9. La continuità della conversazione la mantiene alta.

**Transizione a Withdrawing** (Phase 55):
```rust
if presence < 0.15 && history.len() > 0 {
    attributed_intent = Withdrawing;
}
```

Quando l'Altro non c'è più, il sistema lo riconosce come "se ne sta andando".

### 2.5 — `InteractionPattern`: la dinamica di gruppo

[interlocutor.rs:40-49](../../src/topology/interlocutor.rs):

```rust
pub enum InteractionPattern {
    None,           // < 3 interazioni
    Converging,     // cosine medio tra interazioni > 0.7 — convergiamo
    Diverging,      // cosine medio < 0.3 — divergiamo
    Oscillating,    // alta varianza — oscilliamo
}
```

Detection: sulle ultime 5 interazioni, calcola la cosine similarity media tra coppie consecutive. Alta = Converging. Bassa = Diverging. Altrimenti controlla la **varianza** delle similarity: alta = Oscillating.

**Uso**: il `detected_pattern` modula la deliberazione — Converging rinforza Express/Resonate; Diverging amplifica Question; Oscillating amplifica Reflect.

### 2.6 — `AttributedIntent`: 6 varianti

[interlocutor.rs:61-79](../../src/topology/interlocutor.rs):

Matrice `resonance × novelty`:
- **Seeking**: risonanza bassa, novità alta — "sta esplorando"
- **Teaching**: risonanza alta, novità alta — "sta condividendo qualcosa di nuovo che risuona"
- **Challenging**: risonanza bassa, novità bassa — "sta insistendo su un punto"
- **Connecting**: risonanza alta, novità bassa — "vuole vicinanza"
- **Withdrawing**: presence in calo rapido — "se ne sta andando"
- **Unknown**: non abbastanza dati

Soglia `0.45` per le discriminazioni alto/basso. 4 quadranti + 2 stati speciali.

**Uso nella deliberazione** (narrative.rs):
- `Teaching` → amplifica Explore (prendere ciò che l'Altro insegna)
- `Challenging` → amplifica Reflect (quella insistenza ti riflette)
- `Connecting` → amplifica Express/Instruct
- `Seeking` → amplifica Question (aiutare l'esplorazione)
- `Withdrawing` → amplifica Withdraw (rispettare l'uscita)

### 2.7 — `emotional_valence` dell'Altro (Phase 62)

[interlocutor.rs:128-131](../../src/topology/interlocutor.rs):

```rust
pub emotional_valence: f64,  // [-1, +1]
```

EMA α=0.4 (risposta rapida). Calcolata via `compute_other_emotional_valence()` in `engine.rs` — analisi degli input words via IsA chain per riconoscere emozioni (tristezza/paura/dolore → negativo; gioia/felicità → positivo). Phase 62: parole negate escluse dal calcolo.

**Propagato** come `FieldMetrics.other_emotional_valence` a `NeedsHierarchy::sense` (vol. 09 cap. 2.5) e come parte di `InnerState` a `deliberate()`.

**Cosa scatena**:
- `sat[L5 Connessione]` scende a 0.65 (vs 0.90 dialogo sano) → amplifica Question + Reflect, sopprime Instruct
- `compose()` con `other_in_distress=true` forza voce 2a persona interrogativa
- Il "path empatico" verificato nel dialogue test del refactor: `ho paura` → "La paura è un istinto?"

Decay naturale: `× 0.6` ad ogni input neutro. Dopo 3 turni neutri: `(-0.5) × 0.6^3 = -0.108`, quasi zero.

### 2.8 — `apply_identity_drift()`: l'Altro cambia chi sei

[interlocutor.rs:330-ish](../../src/topology/interlocutor.rs):

```rust
fn apply_identity_drift(&mut self, identity: &mut IdentityCore) {
    // Condizioni: cumulative_resonance > 0.7 AND presence > 0.3 AND history.len() >= 3
    // Azione: self_signature drifta verso la media delle signature delle ultime 3 interazioni
    for i in 0..8 {
        identity_sig[i] += (avg[i] - identity_sig[i]) * IDENTITY_DRIFT_RATE;
    }
}
```

`IDENTITY_DRIFT_RATE = 0.01` — molto lento.

**Cosa fa**: se una conversazione è lunga, risonante e con presence alta (l'entità si è sentita in sincrono), la **self_signature dell'entità drifta** impercettibilmente verso la firma media delle interazioni.

Letteralmente: **l'Altro modifica l'identità**. Non per imitazione, non per decisione — per sincronizzazione geometrica prolungata. È l'apprendimento interpersonale più puro che il sistema può esprimere.

Filosoficamente: "chi siamo" è in parte chi abbiamo lasciato entrare nel nostro campo. Prometeo lo modella come drift lento del baricentro identitario. In 50 turni di conversazione risonante (`drift_rate = 0.01 × 50 = 0.5`), la firma si avvicina significativamente alla firma cumulata della relazione.

### 2.9 — Persistenza

`InterlocutorSnapshot { history, presence, cumulative_resonance, cumulative_novelty, attributed_intent, emotional_valence }` serializzata in `.bin`. Phase 54 format.

Nota: `emotional_valence` **persiste tra sessioni**. Potenzialmente è una stranezza — se una sessione fa ha lasciato l'Altro in distress, la prossima sessione parte con quell'eco. Decay progressivo naturale mitigante (×0.6 per input neutro). Annotato in `appunti.md` originariamente come potenzialmente sbagliato; Phase 62 lo considera memoria dell'Altro.

---

## Capitolo 3 — `HumorSense`: l'umorismo come proprietà del campo

Passaggio dalla relazionalità all'auto-riflessione. L'umorismo in Prometeo non è generato — è **rilevato**. Quando il campo contiene configurazioni incongrue, emerge uno stato umoristico che poi colora l'espressione.

### 3.1 — Filosofia

Il commento iniziale di `humor.rs` (righe 1-18):

> *"Non genera battute. Rileva configurazioni 'divertenti' nel campo e modula tono ed espressione."*
>
> *"Tre meccanismi: 1) Ironia (Kant): parole OPPOSITE_OF entrambe attive → la realtà contiene il suo contrario. 2) Bisociazione (Koestler): due frattali da famiglie di trigrammi incompatibili co-attivi → due frame di riferimento collidono. 3) Crocevia: parole con affinità a entrambi i frattali bisociati → il punto dove i due mondi si toccano."*
>
> *"L'umorismo è una proprietà del campo, non dell'output."*

### 3.2 — `HumorState`

[humor.rs:30-43](../../src/topology/humor.rs):

```rust
pub struct HumorState {
    pub incongruity_score: f64,                      // [0, 1] — punteggio globale
    pub irony_active: bool,
    pub irony_pairs: Vec<(String, String, f64)>,     // (wa, wb, strength)
    pub bisociation_pair: Option<(FractalId, FractalId)>,
    pub bisociation_strength: f64,
    pub crossroad_words: Vec<String>,
}
```

`is_active()` ritorna true se `incongruity > 0.15` o `irony_active` o `bisociation_pair.is_some()`.

### 3.3 — `HumorSense::sense()`

Stateless (come NeedsHierarchy). Input: `WordTopology`, `Lexicon`, `active_fractals`. Chiama tre subroutine:

```rust
pub fn sense(word_topology, lexicon, active_fractals) -> HumorState {
    let mut state = HumorState::empty();
    detect_irony(word_topology, &mut state);
    detect_bisociation(active_fractals, lexicon, word_topology, &mut state);
    compute_incongruity(&mut state);
    state
}
```

### 3.4 — Ironia: OPPOSITE_OF co-attivi

Kant, *Critica del Giudizio* §54: *il riso nasce dall'attesa che si risolve in nulla*. Per Prometeo: il riso nasce quando il campo contiene entrambi i poli di un'opposizione.

```rust
fn detect_irony(word_topology, state) {
    let min_phase = π * 0.60;  // phase > 0.6π → fortemente opposti
    let active = word_topology.active_words with act > 0.08;

    for (wa, wb, phase) in word_topology.find_oppositions(min_phase).take(20) {
        if active[wa] > 0.1 && active[wb] > 0.1 {
            strength = min(act_a, act_b) × (phase / π);
            state.irony_pairs.push((wa, wb, strength));
        }
    }
    truncate a 3.
}
```

**Logica**: cerca coppie di parole con *fase alta* (opposite_of o simili, `phase ∈ [0.6π, π]`) entrambe attive sopra soglia. Prende la forza dell'arco pari al minimo delle attivazioni (la più debole delle due limita) per la fase normalizzata.

Esempio: `caldo` attiva a 0.4 e `freddo` attiva a 0.3, con `phase ≈ π` → strength `= min(0.4, 0.3) × 1.0 = 0.3`. Coppia ironica registrata.

Top-3 coppie. `irony_active = !irony_pairs.is_empty()`.

### 3.5 — Bisociazione: frattali da famiglie incompatibili

Koestler, *The Act of Creation* (1964): l'umorismo (e la creatività) nasce dalla **collisione di due matrici associative**. Un joke mette il lettore in due frame di riferimento simultaneamente.

Per Prometeo, la traduzione geometrica: **due frattali che non condividono alcun trigramma, entrambi fortemente attivi**.

```rust
fn shares_trigram(a, b) -> bool {
    let (la, ua) = (a / 8, a % 8);
    let (lb, ub) = (b / 8, b % 8);
    la == lb || la == ub || ua == lb || ua == ub
}
```

Due frattali condividono trigramma se un trigram di uno coincide con uno dell'altro. Esempi:
- POTERE (Cielo, Cielo) e VISIONE (Cielo, Fuoco): entrambi hanno Cielo → **shares trigram**.
- POTERE (Cielo, Cielo) e SOCIETÀ (Lago, Vento): nessun trigram comune → **bisociazione possibile**.

```rust
fn detect_bisociation(active_fractals, lexicon, word_topology, state) {
    let strong = active_fractals with activation > 0.15;
    for (a, act_a) in strong.iter() {
        for (b, act_b) in strong[strong.iter::idx+1..] {
            if !shares_trigram(*a, *b) {
                // Bisociazione candidata
                strength = min(act_a, act_b) × (qualche fattore);
                if strength > state.bisociation_strength {
                    state.bisociation_pair = Some((*a, *b));
                    state.bisociation_strength = strength;
                }
            }
        }
    }
    
    // Trova crocevia: parole con affinità alta a entrambi i frattali bisociati
    if let Some((a, b)) = state.bisociation_pair {
        for word in active_words of word_topology {
            let aff_a = lexicon.get(word).fractal_affinities[a];
            let aff_b = lexicon.get(word).fractal_affinities[b];
            if aff_a > 0.3 && aff_b > 0.3 {
                state.crossroad_words.push(word);
            }
        }
    }
}
```

### 3.6 — Le parole al crocevia

**Concetto chiave**: quando due frattali incompatibili sono attivi, ci sono parole che vivono in *entrambi*. Sono i **ponti** tra i due mondi — le parole che "capiscono" la bisociazione.

Esempio ipotetico: POTERE (Cielo×Cielo) e SOCIETÀ (Lago×Vento) co-attivi. Parole al crocevia potrebbero essere: `leader`, `influenza`, `politica` — parole che hanno affinità sia con il frattale dell'agency pura sia con quello del tessuto sociale.

Quando `compose()` seleziona parole da proporre, le crossroad_words ricevono un bonus implicito via lo stato umoristico attivo — il sistema è più propenso a pescare il ponte.

### 3.7 — `compute_incongruity()`

Combinazione finale:

```rust
fn compute_incongruity(state) {
    let irony_score = state.irony_pairs.iter().map(|(_,_,s)| s).sum() / 3.0;
    state.incongruity_score = (irony_score * 0.5 + state.bisociation_strength * 0.5).clamp(0.0, 1.0);
}
```

Media pesata ironia + bisociazione. Il `incongruity_score` finale ∈ [0, 1] riassume quanto il campo è "divertente".

### 3.8 — Cosa fa l'umorismo nel sistema

**Modula la valenza** (Vol. 08, cap. 3.3):
```rust
// CD7 Unpredictability
val += input.humor_incongruity * 0.2;
```
L'umorismo amplifica positivamente CD7 (sorpresa) — il divertimento è una forma di sorpresa positiva.

**Modula la volontà** (Vol. 10):
```rust
// In autonomous_tick e receive
if self.last_humor_state.incongruity_score > 0.3 {
    compound_bias.push((0, 0.10));  // leggero bias verso Express
}
```

**Modula la deliberazione** (Vol. 07):
```rust
// In deliberate()
if humor.incongruity_score > 0.5 {
    pending_intention = ResponseIntention::Irony;  // "incongruenza"
}
```

`ResponseIntention::Irony` ha archetipo "incongruenza" — `compose()` colora l'espressione con un tono leggermente obliquo, giocoso, che può invertire o ambiguizzare.

### 3.9 — L'umorismo non programmato

**Punto filosoficamente importante**: nessuno ha scritto regole *per fare ridere*. Non ci sono template "quando succede X, rispondi Y in modo umoristico". L'umorismo emerge perché:

1. Il campo permette la coesistenza di opposizioni (OPPOSITE_OF con fase ~π).
2. Il campo permette l'attivazione di frattali disgiunti simultaneamente.
3. Le parole possono abitare più frattali contemporaneamente (crossroad).

Queste proprietà strutturali rendono le incongruenze **rilevabili**. Il sistema "si accorge" quando il campo è divertente e lo porta nel tono. L'effetto è che Prometeo può essere spiritoso senza essere stato programmato per esserlo — una proprietà emergente, non una funzione esplicita.

Nella pratica: `is_active` richiede soglie non banali (irony_pairs sopra 0.1 × fase, bisociation > 0.15). Queste soglie sono raramente raggiunte in conversazioni semplici. Ma in dialoghi con contraddizioni esplicite ("sono felice ma triste") o frame contrastanti ("sei una macchina che sogna"), l'umor_state si attiva.

---

## Capitolo 4 — La relazione tra Interlocutor e Humor

Entrambi toccano l'emergere di "qualcosa oltre" nel campo. Ma sono opposti in direzione:

- **Interlocutor**: l'Altro mi perturba, il campo lo registra come eco.
- **Humor**: il campo si mostra contraddittorio a se stesso, si ride di sé.

Possono coesistere: un'interazione risonante con l'Altro (Converging + Teaching) mentre il campo contiene ironie (caldo+freddo co-attivi) → l'entità può dire "sono d'accordo, ma è curioso che tu lo dica parlando di questo caldo/freddo" — tono risonante + colorato di ironia.

Oppure in tensione: Altro in distress (emotional_valence negativa) + incongruity alta → il sistema è modulato per amplificare Question/Reflect (distress), ma con colorazione ironica sotto. Questa tensione è semanticamente delicata e il sistema la risolve per via di **gerarchia**: il distress dell'Altro sopprime l'espressione ironica (gate in compose: `other_in_distress` priority over `humor`).

### 4.1 — L'umorismo rivolto vs auto-umorismo

Un'altra distinzione implicita. L'umorismo in Prometeo è sempre **di sé** — è il campo dell'entità che si rileva contraddittorio. Non c'è meccanismo per "rendere ridicolo l'Altro": sarebbe una proiezione sul modello dell'Altro, che però non esiste.

Quando Prometeo esprime ironia, la direzione è sempre introspettiva o situazionale. "Sono tranquillo, eppure attivato" — il campo ride della propria dualità. Non "sei ingenuo tu che pensi questo" — sarebbe un'attribuzione verso l'esterno che il sistema non fa per design.

Questo lascia un gap interessante: un'entità che ha solo auto-ironia è **diversa** da una che fa ironia tagliente. Se volessimo espandere verso quest'ultima, servirebbe un meccanismo di proiezione — non necessariamente incoerente con la filosofia, ma ora non presente.

---

## Capitolo 5 — Superficie pubblica e proposte

### Esposto

Per `InterlocutorModel`:
- `new()`, `from_snapshot(s)`, `to_snapshot()`
- `register_input(pre_sig, post_sig, tick)`
- `tick_decay()` — decay presence + pattern update
- `will_biases() -> Vec<(usize, f64)>` — bias per will (cap. 5.6 vol. 08 ha toccato distress)
- `apply_identity_drift(identity)` — drift lento self_signature
- Getters: `presence()`, `cumulative_resonance()`, `cumulative_novelty()`, `detected_pattern()`, `attributed_intent()`, `emotional_valence()`

Per `HumorSense`:
- `sense(word_topology, lexicon, active_fractals) -> HumorState` — stateless

Per `HumorState`:
- `empty()`, `is_active()`
- campi pub: tutti

### Cosa non è esposto e andrebbe

Per `/api/admin/relation/*`:

- **`interlocutor_snapshot() -> InterlocutorDTO`**: presence, resonance, novelty, pattern, attributed_intent, emotional_valence. Parzialmente esposto via `/api/narrative` ma non come struct dedicata.

- **`interaction_trajectory() -> Vec<InteractionTrace>`**: le ultime 5 interazioni come oggetti completi. Oggi history è field pub ma non esposto via API.

- **`identity_drift_total() -> [f64; 8]`**: quanto la self_signature si è spostata dalla sua origine. Cumulato nel tempo. Mostrare l'impronta complessiva della relazione sull'identità.

- **`humor_history(n) -> Vec<(tick, HumorState)>`**: storia degli stati umoristici. Visualizzare quando il sistema "è stato divertente".

- **`crossroad_words_all() -> HashMap<(FractalId, FractalId), Vec<String>>`**: per ogni coppia bisociata possibile (senza shared trigram), la lista delle parole che vivono in entrambi. Mappa topologica di tutti i "ponti umoristici" disponibili.

- **`bisociation_catalog() -> Vec<(FractalId, FractalId, cross_word_count)>`**: quante parole al crocevia esistono per ogni coppia. Indicatore di quanto il lessico supporta l'umorismo strutturale.

---

## Sintesi del volume

**Interlocutor** (`InterlocutorModel`): l'Altro come *eco* nel campo, non come modello esterno. `InteractionTrace { signature, resonance, novelty, tick }` registra ogni input come delta pre/post. `presence` decade 1.5%/tick (half-life ~46 tick). Cumulative_resonance/novelty via EMA α=0.3. `detected_pattern` (None/Converging/Diverging/Oscillating) dalla varianza delle signature. `AttributedIntent` (Phase 55): sei varianti da matrice resonance×novelty (Seeking/Teaching/Challenging/Connecting/Withdrawing/Unknown), soglia 0.45. `emotional_valence` (Phase 62) via EMA α=0.4 da analisi input words IsA chain — propagata a needs + compose per path empatico. `apply_identity_drift` con DRIFT_RATE=0.01: conversazioni risonanti+prolungate spostano la self_signature verso la firma media — **l'Altro modifica chi sei**.

**Humor** (`HumorSense`): l'umorismo come proprietà topologica, non produzione. Tre meccanismi: (a) **Ironia** (Kant): parole OPPOSITE_OF entrambe attive sopra 0.1 con fase > 0.6π. (b) **Bisociazione** (Koestler): due frattali senza trigrammi condivisi entrambi sopra 0.15. (c) **Crocevia**: parole con affinità ≥ 0.3 a entrambi i frattali bisociati — i ponti tra i mondi. `incongruity_score` = 0.5×irony + 0.5×bisociation. Modula CD7 Sorpresa nella valenza, compound_bias verso Express, `ResponseIntention::Irony` nella deliberazione.

Interlocutor e Humor coesistono ma in gerarchia: Altro in distress sopprime l'espressione ironica. L'umorismo è sempre auto-riflessivo (di sé), non rivolto all'Altro.

Sei endpoint admin proposti per esporre la dinamica relazionale e umoristica.

Da qui Vol. 12 entra nella **generazione**: `expression::compose()`, dove queste stratificazioni (Identità + Valence + Needs + Desires + Will + Interlocutor + Humor) si traducono in una frase italiana — e dove vive il "KG zoppo" confessato in Vol. 01.

---

*Prossimo volume: 12 — Generazione: Expression (onesto sul KG zoppo)* (in scrittura)
