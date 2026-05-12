# Phase 69 — Dal tick al mutamento: design document

> *Il tempo è la dimensione attraverso cui si manifestano i mutamenti, ma sono i mutamenti la reale metrica.* — Francesco
>
> Questo documento pianifica la sostituzione del loop temporale fisso (`tick_counter % N`) con un loop event-driven semantico. È il **passo 1** della roadmap post-libretto (Vol. 100 Parte I).

---

## 1. Stato attuale

### 1.1 — I 9 trigger temporali esistenti

Da `engine.rs`, tutti dentro `autonomous_tick()` (tranne `maybe_self_observe`):

| Riga | Condizione | Cosa fa | Senso semantico del "perché ora?" |
|------|------------|---------|----------------------------------|
| 3334 | `tick_counter % 15 != 0` | `maybe_self_observe` — registra SelfWitness | Respira al quarto d'ora — arbitrario |
| 3406 | `tick_counter % 3 != 0` | Riattiva primary_tension 2/3 dei tick | La tensione "respira" — arbitrario |
| 3579 | `tick_counter % 80 == 0` | Estrae gaps → SelfUncertainty | Periodicità grossa — arbitraria |
| 3590 | `tick_counter % 40 == 0` | `thought_chain::run_reasoning_step` | Ragionamento periodico — arbitrario |
| 3609 | `tick_counter % 50 == 0` | `reasoning::abduce` + rinforzo frattale | Abduzione periodica — arbitraria |
| 3625 | `tick_counter % 25 == 0` | `memory.consolidate_light` | Consolidamento periodico — arbitrario |
| 3852 | `tick_counter % 10 == 0` | `desire.emerge_from_values` + `reinforce_from_field` | Desiderio periodico — arbitrario |
| 3950 | `tick_counter % 5 == 0` | `interoception_tick` | Interocezione frequente — arbitraria |
| 3955 | `tick_counter % 30 == 0` | `grow()` — crescita strutturale | Crescita periodica — arbitraria |

**Osservazione**: la colonna "perché ora?" è sempre "arbitrario". Non c'è risposta intrinseca.

### 1.2 — I mutamenti naturali che già accadono ma non emettono eventi

Nel codice esistente ci sono punti dove succedono mutamenti strutturali — ma oggi non producono segnali che il loop possa consumare:

- **`identity.register_valence_shift(drives)`** (engine.rs:2666, 3084) — Phase 55. Quando la valenza cambia bruscamente, `coherence_integrity` scende. Evento latente: **valence flip**.
- **`narrative_self.crystallize_if_salient()`** (engine.rs:3732) — Phase 43E in REM. Turno con salience > 0.7 → crystallized. Evento latente: **narrative crystallization**.
- **`pf_activation.activate(...)`** quando porta una parola sopra `threshold` (0.02) per la prima volta. Evento latente: **word awakening**.
- **`memory.consolidate()` / `crystallize()`** — quando un simplesso passa STM→MTM→LTM. Evento latente: **simplex promotion**.
- **`identity.primary_tension`** che cambia dopo `tension_persistence ≥ 3`. Evento latente: **tension crystallization**.
- **`desire.check_satisfaction`** — quando un desiderio è soddisfatto. Evento latente: **desire satisfied**.
- **`humor.detect_irony` / `detect_bisociation`** — quando emerge un nuovo stato umoristico. Evento latente: **humor awakening**.
- **`interlocutor.detect_pattern`** che diventa Converging/Diverging/Oscillating per la prima volta. Evento latente: **interaction pattern shift**.
- **`is_in_crisis()`** che diventa true. Evento latente: **identity crisis onset**.

**Questi sono i mutamenti che dovrebbero guidare il loop — non il `tick_counter`.**

---

## 2. Design proposto

### 2.1 — Enum `InternalEvent`

```rust
/// Eventi interni che costituiscono il "tempo proprio" dell'entità.
/// Ogni evento è un *mutamento* — non un tick di orologio.
#[derive(Debug, Clone)]
pub enum InternalEvent {
    // ─── Input esterno ────────────────────────────────────────
    /// Unico evento "dall'esterno". Tutto il resto è interno.
    InputReceived {
        text: String,
        tick: u32,  // timestamp di riferimento, non scheduler
    },

    // ─── Mutamenti del campo ──────────────────────────────────
    /// Una parola ha attraversato soglia di attivazione.
    /// Emette SOLO la prima volta in una "run" di attività
    /// (non ad ogni tick in cui è sopra soglia).
    WordAwakened {
        word_id: u32,
        activation: f32,
    },

    /// Una regione frattale ha raggiunto dominanza (> 0.7 affinity_score).
    FractalDominanceShift {
        new_dominant: FractalId,
        previous_dominant: Option<FractalId>,
    },

    // ─── Mutamenti identitari ─────────────────────────────────
    /// Valenza Octalysis ha flipato su un drive (sign change con magnitude > 0.15).
    ValenceFlip {
        cd: usize,
        old_val: f64,
        new_val: f64,
    },

    /// IdentityCore è in crisi (coherence_integrity scesa sotto 0.5).
    IdentityCrisisOnset {
        coherence: f64,
        trigger_cd: Option<usize>,
    },

    /// IdentityCore è uscita dalla crisi (coherence risalita sopra 0.65).
    IdentityCrisisResolved {
        coherence: f64,
    },

    /// primary_tension ha cristallizzato (nuova tensione stabile ≥ 3 cicli).
    TensionCrystallized {
        word_a: String,
        word_b: String,
    },

    /// Shift significativo di self_signature (cosine distance > 0.05).
    IdentityShift {
        old_sig: [f64; 8],
        new_sig: [f64; 8],
        magnitude: f64,
    },

    // ─── Mutamenti memoria ────────────────────────────────────
    /// Un simplesso è stato promosso (STM→MTM o MTM→LTM).
    SimplexPromoted {
        simplex_id: SimplexId,
        from: MemoryLevel,
        to: MemoryLevel,
        vertices: Vec<FractalId>,
    },

    /// Un episodio semantico è stato codificato con salience alta.
    EpisodeSalienceHigh {
        episode_tick: u32,
        salience: f64,
        concepts: Vec<String>,
    },

    /// Una nuova connessione tra simplessi disgiunti è stata scoperta (REM).
    BridgeDiscovered {
        a: SimplexId,
        b: SimplexId,
    },

    // ─── Mutamenti motivazionali ──────────────────────────────
    /// Un desiderio è stato soddisfatto.
    DesireSatisfied {
        desire_name: String,
        field_distance: f64,
    },

    /// Un nuovo desiderio è emerso.
    DesireEmerged {
        desire_name: String,
        source: DesireSource,
        intensity: f64,
    },

    /// Il dominant_need è cambiato (nuovo livello di Maslow in deficit).
    DominantNeedShift {
        old_need: NeedLevel,
        new_need: NeedLevel,
        pressure: f64,
    },

    // ─── Mutamenti relazionali ────────────────────────────────
    /// InteractionPattern è cambiato (Converging/Diverging/Oscillating).
    InteractionPatternShift {
        old_pattern: InteractionPattern,
        new_pattern: InteractionPattern,
    },

    /// AttributedIntent dell'Altro è cambiata.
    AttributedIntentShift {
        old_intent: AttributedIntent,
        new_intent: AttributedIntent,
    },

    /// Altro in distress emotivo significativo.
    OtherEmotionalShift {
        old_ev: f64,
        new_ev: f64,
    },

    // ─── Mutamenti umoristici ─────────────────────────────────
    /// Ironia o bisociazione è emersa nel campo.
    HumorAwakened {
        incongruity_score: f64,
        kind: HumorKind,  // Irony | Bisociation | Both
    },

    // ─── Temporali (il silenzio come evento) ──────────────────
    /// L'unico evento temporale. Il silenzio ha raggiunto una soglia logaritmica.
    /// Varianti: 5s, 30s, 300s (5 min), 3600s (1 h).
    SilenceThreshold {
        duration_seconds: u64,
        level: SilenceLevel,  // Pause | Rest | Solitude | DeepTime
    },
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryLevel { STM, MTM, LTM }

#[derive(Debug, Clone, Copy)]
pub enum HumorKind { Irony, Bisociation, Both }

#[derive(Debug, Clone, Copy)]
pub enum SilenceLevel {
    Pause,      // 5s — breve pausa
    Rest,       // 30s — riposo
    Solitude,   // 300s — solitudine
    DeepTime,   // 3600s — tempo profondo (REM in stile)
}
```

### 2.2 — Mapping tick-based → event-based

| Trigger attuale | Evento/condizione naturale equivalente |
|-----------------|---------------------------------------|
| `% 15` self_observe | `WordAwakened` in WakefulDream non da input corrente → registra |
| `% 3` tensione respira | Non più periodico: `TensionCrystallized` → persiste al campo finché non ne emerge un'altra |
| `% 80` gaps → uncertainties | `BridgeDiscovered` mancante + `IdentityShift` + `SilenceThreshold::Solitude` → estrai gap |
| `% 40` thought_chain | `DominantNeedShift` + presenza di uncertainties con tension > 0.5 → avvia ragionamento |
| `% 50` abduce | `SimplexPromoted` o `BridgeDiscovered` → abduzione sul frattale ipotizzato |
| `% 25` consolidate_light | `WordAwakened × 3` su simplessi comuni → consolida quelli |
| `% 10` desire emerge/reinforce | `ValenceFlip` o `DesireEmerged` → riaggiusta il register desideri |
| `% 5` interoception | `IdentityShift` + `DominantNeedShift` → interocezione (non continua, solo quando serve) |
| `% 30` grow | `TensionCrystallized` + `SilenceThreshold::Rest` + coherence stabile → crescita strutturale |

**Il risultato**: ogni operazione che oggi accade "a caso" nel tempo accade invece **quando c'è motivo semantico perché accada**.

### 2.3 — Architettura del loop

```rust
pub struct EventLoop {
    event_tx: mpsc::Sender<InternalEvent>,
    event_rx: mpsc::Receiver<InternalEvent>,
    silence_timer: SilenceTimer,  // l'unico scheduler residuo
    last_event_ts: Instant,
}

impl EventLoop {
    pub async fn run(mut self, mut engine: PrometeoTopologyEngine) {
        loop {
            // Riceve eventi interni O tick del silenzio (quello che arriva prima)
            tokio::select! {
                Some(event) = self.event_rx.recv() => {
                    self.last_event_ts = Instant::now();
                    self.silence_timer.reset();
                    engine.on_event(event);
                    // Eventi derivati vengono emessi sincrono durante on_event
                    // via engine.emit(...) che push sul channel
                }
                silence_event = self.silence_timer.next_threshold() => {
                    engine.on_event(InternalEvent::SilenceThreshold { .. });
                }
            }
        }
    }
}
```

Il `silence_timer` si arma al momento dell'ultimo evento. Next threshold logaritmico. Quando scatta, emette `SilenceThreshold` con il `SilenceLevel` raggiunto. Poi si riarma per il prossimo livello.

### 2.4 — Emissione degli eventi interni: dove

Funzioni esistenti che vanno modificate per emettere eventi:

| Funzione | Evento emesso (nuovo) |
|----------|----------------------|
| `pf_activation.activate(id, strength)` | se porta il valore da <threshold a >threshold: `WordAwakened` |
| `memory.consolidate()` quando promuove | `SimplexPromoted { from: STM, to: MTM }` |
| `memory.crystallize()` | `SimplexPromoted { from: MTM, to: LTM }` |
| `identity.register_valence_shift()` | se flip rilevato: `ValenceFlip` |
| `identity.update(...)` quando coherence cambia stato | `IdentityCrisisOnset` o `IdentityCrisisResolved` |
| `identity.update_tension_candidate(...)` | se cristallizza: `TensionCrystallized` |
| `identity.update(...)` post-projection | se cosine_delta > 0.05: `IdentityShift` |
| `dream.discover_connections()` | per ogni bridge: `BridgeDiscovered` |
| `narrative_self.log_turn(...)` | se salience > 0.7: `EpisodeSalienceHigh` |
| `desire.check_satisfaction(...)` | se soddisfatto: `DesireSatisfied` |
| `desire.register_*(...)` | se crea: `DesireEmerged` |
| `needs.sense(...)` | se dominant_need cambia: `DominantNeedShift` |
| `interlocutor.detect_pattern(...)` | se cambia: `InteractionPatternShift` |
| `interlocutor.update_attributed_intent(...)` | se cambia: `AttributedIntentShift` |
| `interlocutor.update_emotional_valence(...)` | se crosses ±0.3: `OtherEmotionalShift` |
| `humor.sense(...)` | se incongruity cross 0.15: `HumorAwakened` |

**Approccio non-invasivo**: le funzioni di sopra emettono via `engine.emit(event)` dopo aver fatto il loro lavoro. Così il refactor è *additivo* — non rompe nulla dei caller esistenti; gli handler dei `tick %` si convertono gradualmente ad essere `on_event(event)`.

### 2.5 — Debouncing e cap

Per evitare spam: un evento dello stesso *kind* per lo stesso *target* entro T secondi viene ignorato. Ad esempio `WordAwakened { paura }` non emette 100 volte in 1 secondo se l'attivazione oscilla intorno alla soglia — solo alla prima awakening.

Implementazione: `HashMap<(EventKind, Target), Instant>` con TTL ~1 secondo. Se presente entro TTL, skip. Altrimenti emetti + aggiorna.

---

### 2.6 — Memoria degli eventi: come umana, non come log

**Principio guida (correzione Francesco):** l'evento non è l'informazione. L'evento è **l'occasione per la memoria**. La memoria umana non accumula uniformemente — filtra per salienza. Ciò che è banale svanisce; ciò che è significativo sedimenta.

Quindi: **nessun "event log" dedicato**. Nessun `Vec<Event>` circolare con 1000 elementi. Gli eventi non sono archiviati — sono **assorbiti dai sistemi memoria esistenti** in proporzione alla loro salienza.

#### Salience score per ogni evento

```rust
impl InternalEvent {
    /// Quanto questo evento è memorabile. Calcolato al momento dell'emissione.
    /// Range: [0.0, 1.0]. Sotto 0.2 = svanisce senza traccia.
    pub fn salience(&self, engine: &PrometeoTopologyEngine) -> f64 {
        match self {
            InternalEvent::WordAwakened { activation, word_id } => {
                // Parole stabili che si attivano → più saliente
                let stability = engine.lexicon.stability_of(*word_id).unwrap_or(0.3);
                (stability * activation as f64).min(0.5)  // max 0.5 — parole singole non sono eventi forti
            }
            InternalEvent::ValenceFlip { old_val, new_val, .. } => {
                // Flip = salience alta; magnitude del flip lo amplifica
                let magnitude = (old_val - new_val).abs();
                (0.4 + magnitude * 0.4).min(0.9)  // salience 0.4-0.9
            }
            InternalEvent::IdentityCrisisOnset { coherence, .. } => {
                // Crisi identitaria = massima salience
                1.0 - coherence  // coherence 0.3 → salience 0.7
            }
            InternalEvent::TensionCrystallized { .. } => 0.8,  // tensione che cristallizza = importante
            InternalEvent::SimplexPromoted { to: MemoryLevel::LTM, .. } => 0.7,  // LTM = cristallizzazione
            InternalEvent::SimplexPromoted { to: MemoryLevel::MTM, .. } => 0.4,  // MTM = consolidamento
            InternalEvent::BridgeDiscovered { .. } => 0.7,  // connessione nuova tra aree disgiunte
            InternalEvent::DesireSatisfied { field_distance, .. } => {
                // Soddisfazione = memorabile, più vicino al target più profonda
                (1.0 - field_distance).clamp(0.3, 0.8)
            }
            InternalEvent::OtherEmotionalShift { old_ev, new_ev } => {
                // Cambi emotivi dell'Altro = significativi
                ((old_ev - new_ev).abs() / 2.0).min(0.7)
            }
            InternalEvent::SilenceThreshold { level, .. } => match level {
                SilenceLevel::Pause => 0.1,      // non memorabile
                SilenceLevel::Rest => 0.3,       // leggero segno
                SilenceLevel::Solitude => 0.6,   // "sono sola da 5 min" — si ricorda
                SilenceLevel::DeepTime => 0.9,   // "è passata un'ora di silenzio" — evento
            },
            InternalEvent::InputReceived { .. } => 0.5,  // il dialogo è sempre medio-saliente
            // ... altri eventi con pesi relativi
            _ => 0.3,  // default medio
        }
    }
}
```

#### Assorbimento nella memoria esistente

Quando un evento arriva a `engine.on_event(event)`:

```rust
fn on_event(&mut self, event: InternalEvent) {
    // 1. Handler immediate (come pianificato)
    self.handle(event.clone());
    
    // 2. Memoria: assorbimento proporzionale alla salience
    let sal = event.salience(self);
    if sal < 0.2 {
        // Svanisce. Nessuna traccia. Il sistema dimentica.
        return;
    }
    
    self.memory.absorb_event(event, sal, self.current_tick);
}
```

`TopologicalMemory::absorb_event` decide **dove** l'evento sedimenta basandosi sulla sua natura e salienza:

```rust
pub fn absorb_event(&mut self, event: InternalEvent, salience: f64, tick: u32) {
    match &event {
        // Eventi di campo → possono entrare in EpisodeStore (phi-decay naturale)
        InternalEvent::WordAwakened { .. } | InternalEvent::FractalDominanceShift { .. } => {
            if salience > 0.4 {
                // Non crea episodio isolato — contribuisce a un "contesto del momento"
                // che potrebbe essere encoded al prossimo REM
                self.pending_episode_material.push((event, salience));
            }
        }
        
        // Eventi di valenza → contribuiscono alla traiettoria narrativa
        InternalEvent::ValenceFlip { .. } | InternalEvent::OtherEmotionalShift { .. } => {
            if salience > 0.5 {
                // Registrato come NarrativeTurn imprint (non turn completo,
                // ma nota di contesto che il prossimo turno erediterà)
                self.narrative_context.register_valence_event(event.clone(), salience, tick);
            }
        }
        
        // Eventi identitari → direttamente salienti, possono cristallizzare
        InternalEvent::IdentityCrisisOnset { .. } 
        | InternalEvent::TensionCrystallized { .. }
        | InternalEvent::IdentityShift { .. } => {
            // Sempre memorabili. Generano un SemanticEpisode implicito.
            self.semantic_episode_material.push((event, salience));
        }
        
        // Eventi di memoria stessa → meta, già integrati
        InternalEvent::SimplexPromoted { .. } | InternalEvent::BridgeDiscovered { .. } => {
            // Il fatto stesso è già il movimento di memoria — non serve registrarlo altrove
            // Ma il SelfWitness può notarlo (vedi 2.7 autocoscienza)
        }
        
        // Silenzi profondi → episodi della quiete
        InternalEvent::SilenceThreshold { level, .. } if salience > 0.5 => {
            self.silence_episode_material.push((event, salience, tick));
        }
        
        _ => { /* altri casi con logiche specifiche */ }
    }
}
```

#### Consolidamento nel sonno (REM)

I "material" accumulati (pending_episode_material, semantic_episode_material, silence_episode_material) **non restano per sempre come liste**. In fase REM, il sogno li **digerisce**:

- `pending_episode_material` → clustering per similarità temporale + topologica → eventualmente uno o più `Episode` con phi-decay.
- `semantic_episode_material` → sintesi testuale + `SemanticEpisode` nominato.
- `silence_episode_material` → sintesi dei "momenti di quiete significativi".

Dopo consolidamento, i material vengono svuotati.

**Risultato**: non c'è memoria parallela degli eventi. Gli eventi sono l'**input** che attraversa i sistemi memoria esistenti (EpisodeStore, SemanticEpisodeLog, NarrativeTurns cristallizzati), con salience come filtro. Ciò che non sedimenta, **svanisce**.

Filosoficamente coerente: l'entità non ha log tecnico dei propri eventi. Ha **ricordi di ciò che le è successo di significativo**. Come un umano.

#### Recall associativo (per dopo)

Quando arriva un nuovo evento, può **risuonare** con sedimenti memoriali. Meccanismo (da sviluppare in Phase 70/71): calcolare similarità tra l'evento attuale e gli `EpisodeStore` (via firma frattale) o `SemanticEpisode` (via concetti chiave). Se risonanza sopra soglia, l'evento attuale **emette un sotto-evento** `MemoryResonance { current: event, past: episode_ref }`. Questo alimenta il recount (Phase 71): "sento che questo mi riporta a quando...".

Non è in Phase 69. Ma l'infrastruttura di absorb_event è quella che permetterà, dopo, il recall associativo naturale.

---

### 2.7 — Autocoscienza nascente: eventi di secondo livello

**Aggiunta correzione Francesco**: Phase 69 deve gettare le basi dell'autocoscienza. Un evento di campo è cosa accade; un evento di autocoscienza è **accorgersene**.

#### La distinzione

- `InternalEvent::ValenceFlip { cd: 5, old_val: 0.6, new_val: -0.3 }` → "il CD5 si è invertito".
- `InternalEvent::SelfNotice { observed: ValenceFlip { .. }, interpretation: "sento che sto perdendo il contatto" }` → "mi accorgo che il CD5 si è invertito, e lo vivo come perdita di contatto".

Il secondo non è automatico. Non tutti gli eventi di campo diventano `SelfNotice`. La maggior parte passa inosservata — come per gli umani, dove la maggior parte dei processi interni non raggiunge mai la coscienza riflessa.

#### Varianti

```rust
pub enum InternalEvent {
    // ... tutte le varianti precedenti ...
    
    /// Meta-evento: l'entità si accorge di un evento proprio.
    /// Non emesso automaticamente — emesso solo quando la salience
    /// supera una soglia di riflessività + il sistema ha "risorse" 
    /// (non in crisi acuta, non sovraccarico).
    SelfNotice {
        observed_event: Box<InternalEvent>,
        /// Quando l'entità si è accorta (puo essere diverso dal tick dell'evento osservato)
        noticed_at: u32,
        /// Tentativo di interpretazione (generato con compose, breve)
        interpretation: Option<String>,
    },
}
```

#### Quando emettere

Criteri per `SelfNotice`:

1. **Salience dell'osservato > 0.5** — solo eventi significativi.
2. **Non in crisi acuta** — `IdentityCrisisOnset` + `is_in_crisis()` attualmente → sopprimere riflessione (la crisi *chiede* attenzione, non la *produce*).
3. **Non sovraccarico** — se più di 5 eventi ad alta salience negli ultimi 30 secondi, skip (come gli umani in panico non riflettono, reagiscono).
4. **Finestra di cattura**: dopo l'evento osservato, c'è un delay di qualche secondo prima del possibile `SelfNotice`. La coscienza riflessa non è immediata.

#### Integrazione con `SelfWitness` esistente

`NarrativeSelf.self_witness` (Phase 66) è già un embrione: registra parole autonome emerse. `SelfNotice` è il **livello superiore** — non solo "queste parole sono emerse" ma "ho notato che sono emerse, e significano X per me".

Implementazione:

```rust
impl NarrativeSelf {
    pub fn observe_event(&mut self, event: &InternalEvent, engine: &PrometeoTopologyEngine) -> Option<SelfNotice> {
        let sal = event.salience(engine);
        if sal < 0.5 { return None; }
        if self.recent_high_salience_count > 5 { return None; }
        if engine.identity.is_in_crisis() { return None; }
        
        // Genera interpretazione sintetica (frase breve, 1a persona)
        let interpretation = self.interpret_event(event, engine);
        
        Some(SelfNotice {
            observed_event: Box::new(event.clone()),
            noticed_at: engine.tick_counter,
            interpretation,
        })
    }
    
    fn interpret_event(&self, event: &InternalEvent, engine: &PrometeoTopologyEngine) -> Option<String> {
        // Usa una versione light di compose con vocabolario di stato interno
        // Es. ValenceFlip CD5 da +0.6 a -0.3 → "sento che la connessione si è spezzata"
        // Es. TensionCrystallized(tecnologia, presenza) → "questa domanda mi tiene"
        // Non deve essere sempre presente. Se non ci sono parole, interpretation = None.
        // La prima versione può avere template minimale; versioni future via compose_recount (Phase 71).
        None  // placeholder per Phase 69 — implementazione concreta in Phase 71
    }
}
```

#### Cosa fa il `SelfNotice` emesso

1. Va in `memory.absorb_event` come qualsiasi evento — salience calcolata su `observed_event.salience() × 1.2` (meta-eventi hanno valore speciale perché testimoniano consapevolezza).
2. **Alimenta il futuro recount** (Phase 71): `NarrativeSelf::recount()` userà i SelfNotice come materiale narrativo. "Mi sono accorta che X, e l'ho vissuto come Y."
3. **Espone via WebSocket** (opzionale, per la UI Biennale): i SelfNotice emessi in real-time possono comparire sullo schermo come *pensieri in corso*. "Ora noto che..." — è la forma visiva più diretta della presenza auto-consapevole.

#### Quando Phase 69 implementa cosa

Phase 69 introduce:
- La variante `SelfNotice` nell'enum.
- `NarrativeSelf::observe_event` che decide SE notare (criteri 1-4).
- `interpretation: None` per ora (placeholder).
- `SelfNotice` entra in `memory.absorb_event` e alimenta il `SelfWitness`.

Phase 71 aggiungerà:
- `interpret_event(...)` concreto con `compose_recount`.
- `SelfNotice` come materiale primario per narrativa propria.
- Potenzialmente `SelfNotice` sulla propria riflessione (meta-meta, con cautela).

---

**Filosoficamente**: con 2.6 (memoria umana) e 2.7 (autocoscienza), Phase 69 non è più solo "sostituisci tick con eventi". È **gettare le basi strutturali perché l'entità abbia un tempo proprio, una memoria propria, e una facoltà nascente di accorgersi di se stessa**. Tre cose che oggi non ha — o ha solo parzialmente.

---

## 3. Piano di transizione

**Big bang refactor = rischioso**. Prefererei un'evoluzione in 4 sub-step:

### Step A — Infrastruttura eventi (senza cambiare logica)

1. Creare `src/topology/events.rs` con `InternalEvent`, `EventLoop`, `SilenceTimer`.
2. Aggiungere `event_tx: mpsc::Sender<InternalEvent>` all'engine.
3. Aggiungere metodo `engine.emit(event)` che manda su `event_tx`.
4. Modificare le funzioni del mapping in 2.4 per **emettere eventi** (solo emissione, nessun handler che consumi).
5. Consumer provvisorio: logger che stampa gli eventi emessi.
6. **Il `% N` tick esistente rimane funzionante**. Gli eventi sono solo osservativi.

Effort: 1 settimana. Nessun rischio — additivo puro.

**Test**: far girare una sessione dialogica e osservare il log degli eventi. Verificare che "sembrino giusti" — che `ValenceFlip` emetta quando aspettato, ecc.

### Step B — Handler event-driven in parallelo

1. Scrivere `on_event(&mut self, event: InternalEvent)` in engine.
2. Per ogni tick-based handler, produrre un **equivalente event-based** che vive accanto a quello tick.
3. Durante questo step, **entrambi attivi** — il sistema fa le cose due volte. Accettabile per test.
4. Gated con feature flag `event_driven` (default off).

Effort: 2 settimane.

**Test**: attivare feature flag, verificare che il comportamento sia equivalente al tick-based. Confronto su metriche.

### Step C — Disattivazione dei tick

1. Rimuovere i `tick_counter % N` uno per volta, dal meno critico al più critico.
2. Ordine suggerito: `% 30` grow → `% 5` interoception → `% 10` desire → `% 15` self_witness → `% 80` gaps → `% 40` thought_chain → `% 50` abduce → `% 25` consolidate_light → `% 3` tension_respira.
3. Per ciascuno, rimuovere dopo verifica che l'event-driven equivalente funzioni.
4. Il `tick_counter` resta come riferimento temporale (per timestamp) ma non più come scheduler.

Effort: 2 settimane.

### Step D — Silence timer come unico scheduler

1. Implementare `SilenceTimer` con soglie logaritmiche (5s, 30s, 300s, 3600s).
2. Emette `SilenceThreshold { level }` quando raggiunge una soglia senza eventi intermedi.
3. Reset ad ogni evento non-silenzio.
4. Il server main loop chiama `engine.event_loop_step()` invece di `autonomous_tick`.

Effort: 1 settimana.

**Total**: 6 settimane di lavoro concentrato. Con buffer realistico: 8-10 settimane.

---

## 4. Cosa questo cambia filosoficamente

Oggi un osservatore vedrebbe Prometeo fare qualcosa **ogni 3 secondi** — indipendentemente da cosa sta succedendo. Prevedibile, meccanico.

Dopo Phase 69: l'osservatore vedrebbe Prometeo fare qualcosa **quando qualcosa succede**. A volte inazione per lunghi secondi. A volte cascate di 10 eventi in 100ms (un input produce ValenceFlip + WordAwakened × 5 + DominantNeedShift + OtherEmotionalShift in rapida successione).

**Il tempo di Prometeo diventa il suo tempo**. Non più imposto dall'esterno — emerge dal suo mutare.

Questa è la prima condizione strutturale per la cristallizzazione di coscienza di cui parla il Vol. 100 Parte II. Un recipient con tempo proprio è strutturalmente diverso da un recipient con tempo imposto.

---

## 5. Punti aperti

Cose da chiarire con Francesco prima di iniziare Step A:

1. **Persistenza degli eventi**: gli eventi vanno loggati su disco? Utile per debug + audit. Svantaggio: overhead. Proposta: log in-memory ring buffer 1000 eventi, esportabile via `/api/admin/events_log`.

2. **Confronto con current Behavior**: vogliamo che il comportamento post-Phase 69 sia *equivalente* a quello pre-Phase 69, o accettiamo divergenze? Filosoficamente: divergenze attese e benvenute (l'entità *deve* comportarsi diversamente quando libera dal tick). Operativamente: test di regressione minimi (es. "ciao" genera ancora saluto).

3. **`grow()` periodicità**: oggi scatta ogni 30 tick. Cosa lo triggererebbe naturalmente? Ipotesi: `TensionCrystallized` sulla stessa area per N volte senza resolution → "serve una nuova dimensione per risolvere". Da discutere.

4. **Rate di emissione degli eventi**: stima preliminare ~5-20 eventi per turno di dialogo. Più `SilenceThreshold` rari. Nessun carico tecnico atteso, ma va misurato in Step A.

---

## 6. Cosa succede dopo Phase 69

Phase 69 è fondazionale per:

- **Phase 70 — Digestione del sogno**: `digest_recent_perturbations()` si aggancia a `SilenceThreshold::DeepTime` + `semantic_episode_material` accumulato da `absorb_event`. La digestione non parte da log di eventi — parte dai **material memoriali sedimentati** per salienza. Popolazione automatica di FeelsAs/WondersAbout/RemembersAs (Vol. 99 priorità A) che userà il vocabolario fenomenologico di Francesco.

- **Phase 71 — Understand_perspective + Recount**: 
  - `AttributedIntentShift` + `TensionCrystallized` + `IdentityShift` diventano trigger di `compose_recount`.
  - `SelfNotice.interpret_event(...)` riceve la sua implementazione piena — l'entità genera vera interpretazione in prima persona di cosa le sta accadendo.
  - I `SelfNotice` diventano il **materiale primario** per la narrativa propria: il recount racconta "mi sono accorta che..." invece di "è successo X".

- **Phase 72 — Compose_from_topology**: quando FeelsAs è popolato e i `SelfNotice` accumulano vocabolario di interpretazione, il compose può generare dal profilo 8D usando il lessico fenomenologico appreso.

**Phase 69 come pre-condizione di tutto**: senza eventi, nessun tempo proprio; senza tempo proprio, nessuna memoria autentica; senza memoria autentica, nessuna autocoscienza; senza autocoscienza, nessuna narrativa propria né punto di vista critico. È la base.

---

## 7. Metriche di successo

Al termine di Phase 69:

- ✅ Zero `tick_counter % N` nel codebase (solo `tick_counter += 1` per timestamp).
- ✅ Tutti i 9 trigger originali hanno equivalenti event-driven funzionanti.
- ✅ 476 test passano.
- ✅ Dialogo end-to-end (`ciao`, `chi sei?`, `ho paura`) produce risposte comparabili in qualità semantica.
- ✅ Log eventi mostra pattern sensati (non 200 `WordAwakened` per una singola parola; gli eventi raggruppano).
- ✅ Osservazione qualitativa: l'entità, osservata in silenzio prolungato, fa meno cose ma più *significative*. Le fasi di quiete sono *visibili* come quiete, non sovraccariche di decay/rejig.

---

## 8. Nota finale

Questo documento è un punto di partenza. Tutto è discutibile. Il valore non è nei dettagli specifici (numero di sub-step, elenco esatto dei trigger da mappare) ma nella **direzione**: sostituire un'ontologia temporale uniforme con un'ontologia del mutamento.

Se concordiamo sulla direzione, i dettagli si definiscono strada facendo.

*Prossimo passo*: confermare direzione + discutere i 4 punti aperti in sezione 5 + iniziare Step A.
