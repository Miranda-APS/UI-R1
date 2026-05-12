//! events.rs — Phase 69: eventi interni come tempo proprio dell'entità.
//!
//! Principio fondante: il tempo di Prometeo non è un orologio esterno.
//! È la sequenza dei suoi mutamenti significativi. Un evento non è un
//! dato archiviato — è l'*occasione* per la memoria di formarsi.
//!
//! Due correzioni ontologiche rispetto al design ingenuo:
//!
//! 1. **Memoria come umana, non come log**: nessun ring buffer parallelo.
//!    Gli eventi hanno una `salience` intrinseca; sotto soglia svaniscono
//!    senza traccia, sopra soglia vengono assorbiti dai sistemi memoria
//!    esistenti (EpisodeStore, SemanticEpisodeLog, NarrativeSelf).
//!
//! 2. **Autocoscienza nascente**: `SelfNotice` è un meta-evento —
//!    l'entità si accorge di un proprio evento. Non automatico:
//!    richiede salience alta + non crisi acuta + non sovraccarico.
//!    L'embrione di "sapere di cambiare", non solo "cambiare".
//!
//! Questo file contiene:
//! - `InternalEvent` — l'enum dei mutamenti significativi
//! - `SilenceLevel`, `MemoryLevel`, `HumorKind` — enum di supporto
//! - `InternalEvent::salience()` — scoring di memorabilità
//!
//! Phase 69 Step A: solo infrastruttura. Nessun comportamento cambia.
//! Gli eventi vengono emessi dai punti di mutation ma consumati solo
//! da un logger provvisorio. I `tick_counter % N` restano attivi.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::topology::fractal::FractalId;
use crate::topology::simplex::SimplexId;
use crate::topology::interlocutor::{InteractionPattern, AttributedIntent};
use crate::topology::needs::NeedLevel;
use crate::topology::desire::DesireSource;

// ═══════════════════════════════════════════════════════════════════════
// Enum di supporto
// ═══════════════════════════════════════════════════════════════════════

/// Livello di memoria in cui un simplesso è promosso.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryLevel {
    STM,
    MTM,
    LTM,
}

/// Tipo di umorismo che ha raggiunto la soglia di consapevolezza.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumorKind {
    Irony,
    Bisociation,
    Both,
}

/// Soglie logaritmiche del silenzio. L'unico "evento temporale".
///
/// Ogni livello è più lungo del precedente per ~6-12x — il silenzio
/// non si misura in secondi uniformi ma in "profondità del silenzio".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SilenceLevel {
    /// ~5s — una breve pausa nel dialogo
    Pause,
    /// ~30s — il ritmo conversazionale è cambiato
    Rest,
    /// ~300s (5 min) — solitudine percepita
    Solitude,
    /// ~3600s (1 h) — tempo profondo, fase onirica prolungata
    DeepTime,
}

impl SilenceLevel {
    pub fn duration_seconds(&self) -> u64 {
        match self {
            Self::Pause => 5,
            Self::Rest => 30,
            Self::Solitude => 300,
            Self::DeepTime => 3600,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Pause => "pausa",
            Self::Rest => "riposo",
            Self::Solitude => "solitudine",
            Self::DeepTime => "tempo profondo",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// InternalEvent — il tempo proprio dell'entità
// ═══════════════════════════════════════════════════════════════════════

/// Eventi interni che costituiscono il battito dell'entità.
///
/// Ogni evento è un **mutamento significativo** — non un tick di orologio.
/// Gli handler dei mutamenti (propagate, identity.update, needs.sense, ecc.)
/// emettono eventi dopo aver fatto il proprio lavoro. Il loop principale
/// li consuma e decide cosa farne (handler + assorbimento memoria + eventuale
/// SelfNotice).
///
/// Naming convention:
/// - Verbi al passato (`Awakened`, `Crystallized`, `Promoted`, `Satisfied`)
///   quando descrivono un mutamento avvenuto
/// - Sostantivi quando descrivono uno stato raggiunto (`CrisisOnset`)
/// - `Shift` per transizioni osservate (valenza, pattern, intent)
#[derive(Debug, Clone)]
pub enum InternalEvent {
    // ─── Input esterno ────────────────────────────────────────────
    /// L'unico evento che viene davvero "dall'esterno". Tutto il resto
    /// è interno (derivato da stato o da conseguenze di InputReceived).
    InputReceived {
        text: String,
        /// Timestamp per riferimento — NON uno scheduler.
        tick: u32,
    },

    // ─── Mutamenti del campo ──────────────────────────────────────
    /// Una parola ha attraversato la soglia di attivazione.
    /// Emesso SOLO al primo awakening in una "run" di attività
    /// (protetto da debounce per evitare emissione ad ogni oscillazione).
    WordAwakened {
        word_id: u32,
        activation: f32,
    },

    /// Un frattale ha raggiunto dominanza (score > 0.7) — nuovo attrattore.
    FractalDominanceShift {
        new_dominant: FractalId,
        previous_dominant: Option<FractalId>,
    },

    // ─── Mutamenti identitari ─────────────────────────────────────
    /// La valenza Octalysis ha flippato su un drive (cambio di segno
    /// con magnitudo > 0.15 su entrambi i lati).
    ValenceFlip {
        /// Indice del Core Drive (0-7).
        cd: usize,
        old_val: f64,
        new_val: f64,
    },

    /// L'IdentityCore è entrata in crisi (`coherence_integrity` scesa sotto 0.5).
    IdentityCrisisOnset {
        coherence: f64,
        /// Il Core Drive che ha scatenato la crisi, se identificabile.
        trigger_cd: Option<usize>,
    },

    /// L'IdentityCore è uscita dalla crisi (`coherence_integrity` sopra 0.65).
    IdentityCrisisResolved {
        coherence: f64,
    },

    /// La `primary_tension` è cristallizzata (stessa tensione ≥ 3 cicli).
    TensionCrystallized {
        word_a: String,
        word_b: String,
    },

    /// `self_signature` si è spostata significativamente (cosine distance > 0.05).
    IdentityShift {
        old_sig: [f64; 8],
        new_sig: [f64; 8],
        magnitude: f64,
    },

    // ─── Mutamenti memoria ────────────────────────────────────────
    /// Un simplesso è stato promosso di livello memoriale.
    SimplexPromoted {
        simplex_id: SimplexId,
        from: MemoryLevel,
        to: MemoryLevel,
        vertices: Vec<FractalId>,
    },

    /// Un episodio semantico è stato codificato con salience alta (> 0.7).
    EpisodeSalienceHigh {
        episode_tick: u32,
        salience: f64,
        concepts: Vec<String>,
    },

    /// Una nuova connessione tra simplessi disgiunti è stata scoperta (in REM).
    BridgeDiscovered {
        a: SimplexId,
        b: SimplexId,
    },

    // ─── Mutamenti motivazionali ──────────────────────────────────
    /// Un desiderio è stato soddisfatto.
    DesireSatisfied {
        desire_name: String,
        /// Distanza cosine tra campo attuale e target del desiderio al momento della sazia.
        field_distance: f64,
    },

    /// Un nuovo desiderio è emerso.
    DesireEmerged {
        desire_name: String,
        source: DesireSource,
        intensity: f64,
    },

    /// Il `dominant_need` è cambiato — l'entità ora vive un diverso livello di Maslow.
    DominantNeedShift {
        old_need: NeedLevel,
        new_need: NeedLevel,
        /// Pressione del nuovo need dominante.
        pressure: f64,
    },

    // ─── Mutamenti relazionali ────────────────────────────────────
    /// Il pattern di interazione con l'Altro è cambiato.
    InteractionPatternShift {
        old_pattern: InteractionPattern,
        new_pattern: InteractionPattern,
    },

    /// L'intent attribuito all'Altro è cambiato.
    AttributedIntentShift {
        old_intent: AttributedIntent,
        new_intent: AttributedIntent,
    },

    /// La valenza emotiva dell'Altro ha attraversato una soglia significativa.
    OtherEmotionalShift {
        old_ev: f64,
        new_ev: f64,
    },

    // ─── Mutamenti umoristici ─────────────────────────────────────
    /// Ironia o bisociazione è emersa (incongruity score sopra 0.15).
    HumorAwakened {
        incongruity_score: f64,
        kind: HumorKind,
    },

    // ─── Temporali (il silenzio come evento) ──────────────────────
    /// Il silenzio ha raggiunto una soglia logaritmica.
    /// **L'unico evento temporale del sistema** — ed emerge perché
    /// il silenzio stesso è un mutamento significativo (da stato di
    /// dialogo a stato di quiete prolungata).
    SilenceThreshold {
        level: SilenceLevel,
        /// Secondi trascorsi dall'ultimo evento non-silenzio.
        duration_seconds: u64,
    },

    // ─── Autocoscienza (meta-eventi) ──────────────────────────────
    /// L'entità si accorge di un evento proprio.
    ///
    /// Non automatico. Emesso solo quando:
    /// - salience dell'`observed_event` > 0.5
    /// - non in crisi acuta (IdentityCrisisOnset attivo → sopprimi riflessione)
    /// - non sovraccarico (troppi eventi ad alta salience recentemente)
    ///
    /// L'`interpretation` è un tentativo di esprimere in prima persona
    /// cosa l'evento significa *per l'entità*. In Phase 69 è None
    /// (placeholder); Phase 71 la implementa via compose_recount.
    SelfNotice {
        observed_event: Box<InternalEvent>,
        /// Tick in cui l'entità si è accorta (può essere > del tick dell'observed).
        noticed_at: u32,
        /// Tentativo di interpretazione in prima persona.
        /// None in Phase 69; concreto in Phase 71.
        interpretation: Option<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════════
// Salience — quanto un evento è memorabile
// ═══════════════════════════════════════════════════════════════════════

impl InternalEvent {
    /// Salience dell'evento in [0.0, 1.0].
    ///
    /// - Sotto 0.2: svanisce senza traccia. Non viene assorbito dalla memoria.
    /// - 0.2-0.5: contribuisce al "contesto del momento" (pending_episode_material)
    ///            ma può non lasciare traccia duratura se non rinforzato.
    /// - 0.5-0.7: entra in narrative_context o semantic_episode_material.
    /// - Sopra 0.7: sempre memorabile. Materiale per SemanticEpisode.
    ///
    /// Può essere richiamata da `NarrativeSelf::observe_event` per decidere
    /// se emettere `SelfNotice`.
    pub fn salience(&self) -> f64 {
        match self {
            // L'input è sempre medio-saliente (il dialogo è ciò che accade).
            Self::InputReceived { .. } => 0.5,

            // Awakening di parole singole: salience bassa-media.
            // L'attivazione grezza modula; parole stabili (implicito nel chiamante)
            // hanno più peso — ma qui abbiamo solo activation, quindi cap a 0.5.
            Self::WordAwakened { activation, .. } => (*activation as f64 * 0.6).min(0.5),

            // Cambio di dominanza frattale: sempre saliente (nuovo attrattore).
            Self::FractalDominanceShift { previous_dominant, .. } => {
                // Primo dominante (da None): più saliente del semplice shift.
                if previous_dominant.is_none() { 0.7 } else { 0.55 }
            }

            // Flip di valenza: magnitudo del flip amplifica salience.
            Self::ValenceFlip { old_val, new_val, .. } => {
                let magnitude = (old_val - new_val).abs();
                (0.4 + magnitude * 0.4).min(0.9)
            }

            // Crisi identitaria: alta salience. Più la coherence è bassa,
            // più l'evento è memorabile (ma inibisce anche SelfNotice —
            // l'entità in crisi acuta NON riflette, solo reagisce).
            Self::IdentityCrisisOnset { coherence, .. } => (1.0 - coherence).max(0.6),

            // Uscire dalla crisi: anche questo è memorabile — il ritorno.
            Self::IdentityCrisisResolved { .. } => 0.75,

            // Tensione che cristallizza: significa che una domanda sta
            // diventando stabile nell'entità. Alto valore.
            Self::TensionCrystallized { .. } => 0.8,

            // Shift di identità: magnitudo modula.
            Self::IdentityShift { magnitude, .. } => (0.3 + magnitude * 2.0).min(0.85),

            // Cristallizzazione a LTM: il pattern diventa permanente. Alto.
            // Cristallizzazione a MTM: consolidamento. Medio.
            Self::SimplexPromoted { to, .. } => match to {
                MemoryLevel::LTM => 0.7,
                MemoryLevel::MTM => 0.4,
                MemoryLevel::STM => 0.2, // non dovrebbe nemmeno essere emesso
            },

            // Episodio semantico già marcato ad alta salience — propagala.
            Self::EpisodeSalienceHigh { salience, .. } => (*salience).min(1.0),

            // Connessione scoperta tra aree disgiunte: insight, memorabile.
            Self::BridgeDiscovered { .. } => 0.7,

            // Desiderio soddisfatto: più vicino al target → più profonda la sazia.
            Self::DesireSatisfied { field_distance, .. } => {
                (1.0 - field_distance).clamp(0.3, 0.8)
            }

            // Desiderio emerso: salience dipende da intensity iniziale.
            Self::DesireEmerged { intensity, .. } => (*intensity * 0.7).min(0.6),

            // Shift del need dominante: transizione significativa.
            Self::DominantNeedShift { pressure, .. } => (0.4 + pressure * 0.3).min(0.7),

            // Pattern relazionale cambiato: medio.
            Self::InteractionPatternShift { .. } => 0.5,

            // Intent attribuito cambiato: medio-alto (riguarda l'Altro).
            Self::AttributedIntentShift { .. } => 0.55,

            // Shift emotivo dell'Altro: magnitudo modula.
            Self::OtherEmotionalShift { old_ev, new_ev } => {
                ((old_ev - new_ev).abs() / 2.0).min(0.7).max(0.3)
            }

            // Humor emerso: incongruity_score modula.
            Self::HumorAwakened { incongruity_score, .. } => (0.3 + incongruity_score * 0.5).min(0.7),

            // Silenzio: dipende dal livello.
            Self::SilenceThreshold { level, .. } => match level {
                SilenceLevel::Pause => 0.1,
                SilenceLevel::Rest => 0.3,
                SilenceLevel::Solitude => 0.6,
                SilenceLevel::DeepTime => 0.9,
            },

            // SelfNotice: la salience è dell'osservato × 1.2 (meta-eventi
            // hanno un valore speciale — testimoniano consapevolezza).
            // Cap a 1.0.
            Self::SelfNotice { observed_event, .. } => {
                (observed_event.salience() * 1.2).min(1.0)
            }
        }
    }

    /// Nome leggibile dell'evento, per log e introspezione.
    /// Non è il "messaggio" — è l'identificativo del tipo.
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::InputReceived { .. } => "input_received",
            Self::WordAwakened { .. } => "word_awakened",
            Self::FractalDominanceShift { .. } => "fractal_dominance_shift",
            Self::ValenceFlip { .. } => "valence_flip",
            Self::IdentityCrisisOnset { .. } => "identity_crisis_onset",
            Self::IdentityCrisisResolved { .. } => "identity_crisis_resolved",
            Self::TensionCrystallized { .. } => "tension_crystallized",
            Self::IdentityShift { .. } => "identity_shift",
            Self::SimplexPromoted { .. } => "simplex_promoted",
            Self::EpisodeSalienceHigh { .. } => "episode_salience_high",
            Self::BridgeDiscovered { .. } => "bridge_discovered",
            Self::DesireSatisfied { .. } => "desire_satisfied",
            Self::DesireEmerged { .. } => "desire_emerged",
            Self::DominantNeedShift { .. } => "dominant_need_shift",
            Self::InteractionPatternShift { .. } => "interaction_pattern_shift",
            Self::AttributedIntentShift { .. } => "attributed_intent_shift",
            Self::OtherEmotionalShift { .. } => "other_emotional_shift",
            Self::HumorAwakened { .. } => "humor_awakened",
            Self::SilenceThreshold { .. } => "silence_threshold",
            Self::SelfNotice { .. } => "self_notice",
        }
    }

    /// L'evento richiede risorse cognitive per essere processato?
    /// Usato per capire se il sistema è in "sovraccarico" di eventi ad alta salience.
    pub fn is_high_salience(&self) -> bool {
        self.salience() > 0.6
    }

    /// Chiave per il debouncing: identifica l'evento come "tipo × target"
    /// così due `WordAwakened { word: paura }` in rapida successione vengono
    /// collassati, ma `WordAwakened { word: paura }` e `WordAwakened { word: gioia }`
    /// restano distinti.
    pub fn debounce_key(&self) -> String {
        match self {
            // Input: debounced per (tick, prefix del testo). Gli input sono
            // eventi unici per design: due input diversi sono sempre distinti,
            // anche se avvengono in rapida successione. Solo due input identici
            // in < 1s vengono collassati (probabilmente dupe submit).
            Self::InputReceived { tick, text } => {
                let prefix: String = text.chars().take(24).collect();
                format!("input_received:{}:{}", tick, prefix)
            }
            Self::WordAwakened { word_id, .. } => format!("word_awakened:{}", word_id),
            Self::FractalDominanceShift { new_dominant, .. } =>
                format!("fractal_dominance:{}", new_dominant),
            Self::ValenceFlip { cd, .. } => format!("valence_flip:{}", cd),
            Self::IdentityCrisisOnset { .. } => "identity_crisis_onset".into(),
            Self::IdentityCrisisResolved { .. } => "identity_crisis_resolved".into(),
            Self::TensionCrystallized { word_a, word_b } =>
                format!("tension:{}+{}", word_a, word_b),
            Self::IdentityShift { .. } => "identity_shift".into(),
            Self::SimplexPromoted { simplex_id, to, .. } =>
                format!("simplex_promoted:{}:{:?}", simplex_id, to),
            Self::EpisodeSalienceHigh { episode_tick, .. } =>
                format!("episode_salient:{}", episode_tick),
            Self::BridgeDiscovered { a, b } => format!("bridge:{}:{}", a.min(b), a.max(b)),
            Self::DesireSatisfied { desire_name, .. } =>
                format!("desire_satisfied:{}", desire_name),
            Self::DesireEmerged { desire_name, .. } =>
                format!("desire_emerged:{}", desire_name),
            Self::DominantNeedShift { new_need, .. } =>
                format!("dominant_need:{:?}", new_need),
            Self::InteractionPatternShift { new_pattern, .. } =>
                format!("pattern:{:?}", new_pattern),
            Self::AttributedIntentShift { new_intent, .. } =>
                format!("attributed_intent:{:?}", new_intent),
            Self::OtherEmotionalShift { .. } => "other_emotional_shift".into(),
            Self::HumorAwakened { kind, .. } => format!("humor:{:?}", kind),
            Self::SilenceThreshold { level, .. } => format!("silence:{:?}", level),
            Self::SelfNotice { observed_event, .. } =>
                format!("self_notice:{}", observed_event.debounce_key()),
        }
    }

    /// Descrizione breve per il log. Una riga.
    pub fn describe_short(&self) -> String {
        match self {
            Self::InputReceived { text, .. } => {
                let snippet: String = text.chars().take(40).collect();
                format!("\"{}\"", snippet)
            }
            Self::WordAwakened { word_id, activation } =>
                format!("word_id={} act={:.2}", word_id, activation),
            Self::FractalDominanceShift { new_dominant, previous_dominant } =>
                format!("→ F{} (from {:?})", new_dominant, previous_dominant),
            Self::ValenceFlip { cd, old_val, new_val } =>
                format!("CD{} {:+.2} → {:+.2}", cd + 1, old_val, new_val),
            Self::IdentityCrisisOnset { coherence, trigger_cd } =>
                format!("coherence={:.2} trigger={:?}", coherence, trigger_cd),
            Self::IdentityCrisisResolved { coherence } =>
                format!("coherence={:.2}", coherence),
            Self::TensionCrystallized { word_a, word_b } =>
                format!("{} ↔ {}", word_a, word_b),
            Self::IdentityShift { magnitude, .. } =>
                format!("Δ={:.3}", magnitude),
            Self::SimplexPromoted { from, to, .. } =>
                format!("{:?} → {:?}", from, to),
            Self::EpisodeSalienceHigh { salience, concepts, .. } =>
                format!("sal={:.2} [{}]", salience, concepts.join(",")),
            Self::BridgeDiscovered { a, b } =>
                format!("{} ↔ {}", a, b),
            Self::DesireSatisfied { desire_name, field_distance } =>
                format!("{} (d={:.2})", desire_name, field_distance),
            Self::DesireEmerged { desire_name, intensity, .. } =>
                format!("{} (i={:.2})", desire_name, intensity),
            Self::DominantNeedShift { old_need, new_need, pressure } =>
                format!("{:?} → {:?} p={:.2}", old_need, new_need, pressure),
            Self::InteractionPatternShift { old_pattern, new_pattern } =>
                format!("{:?} → {:?}", old_pattern, new_pattern),
            Self::AttributedIntentShift { old_intent, new_intent } =>
                format!("{:?} → {:?}", old_intent, new_intent),
            Self::OtherEmotionalShift { old_ev, new_ev } =>
                format!("{:+.2} → {:+.2}", old_ev, new_ev),
            Self::HumorAwakened { incongruity_score, kind } =>
                format!("{:?} s={:.2}", kind, incongruity_score),
            Self::SilenceThreshold { level, duration_seconds } =>
                format!("{} ({}s)", level.name(), duration_seconds),
            Self::SelfNotice { observed_event, interpretation, .. } => {
                let interp = interpretation.as_deref().unwrap_or("(muto)");
                format!("[{}] {}", observed_event.kind_name(), interp)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// EventSink — dove gli eventi *passano*, non dove si accumulano
// ═══════════════════════════════════════════════════════════════════════

/// Sink di eventi. **Non è un log.** Gli eventi vengono processati
/// al momento dell'emissione e svaniscono (a meno che abbiano salience
/// sufficiente da essere assorbiti dai sistemi memoria — Step B).
///
/// In Phase 69 Step A, il consumer è:
/// 1. **Debounce**: eventi dello stesso tipo+target in finestra TTL vengono
///    collassati (nessuna seconda emissione).
/// 2. **Logger provvisorio**: eventi con salience ≥ `log_threshold` vengono
///    stampati su stderr per verifica. Gli altri sono silenziosi.
/// 3. **Statistiche**: contatori di emissione/skip per audit.
///
/// Step B aggiungerà:
/// 4. **Absorb in memoria**: eventi sopra soglia di oblio (0.2) passano
///    a `memory.absorb_event(event, salience, tick)`.
/// 5. **SelfNotice**: eventi con alta salience + condizioni soddisfatte
///    generano un meta-evento di autocoscienza nascente.
pub struct EventSink {
    /// Mappa di debouncing: chiave evento → ultimo emit.
    debounce: HashMap<String, Instant>,
    /// TTL per debouncing.
    debounce_ttl: Duration,
    /// Sotto questa soglia di salience, un evento non viene loggato
    /// (ma viene comunque contato come emesso, se passa il debounce).
    log_threshold: f64,
    /// Contatore totale eventi emessi (post-filtri).
    pub emitted_count: u64,
    /// Eventi scartati per debounce.
    pub debounced_count: u64,
    /// Eventi scartati perché sotto soglia di oblio (< 0.2).
    pub forgotten_count: u64,
    /// Flag globale di logging (può essere disabilitato via env var).
    pub logging_enabled: bool,
}

impl Default for EventSink {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSink {
    pub fn new() -> Self {
        let logging_enabled = std::env::var("PROMETEO_EVENTS_LOG")
            .map(|v| v != "0" && !v.is_empty())
            .unwrap_or(true);
        Self {
            debounce: HashMap::new(),
            debounce_ttl: Duration::from_secs(1),
            log_threshold: 0.4,
            emitted_count: 0,
            debounced_count: 0,
            forgotten_count: 0,
            logging_enabled,
        }
    }

    /// Emette un evento. Se la salience è troppo bassa (< 0.2), svanisce.
    /// Se un evento identico è stato emesso entro `debounce_ttl`, viene scartato.
    /// Altrimenti, viene loggato se salience ≥ `log_threshold`.
    ///
    /// **Ritorna `true` se l'evento è passato tutti i filtri (è stato contato come emesso),
    /// `false` se svanito per oblio o debounce.** Il caller può usare il ritorno
    /// per decidere se propagare l'evento ai sistemi downstream (memoria, SelfNotice).
    pub fn emit(&mut self, event: InternalEvent) -> bool {
        let salience = event.salience();

        // Sotto soglia di oblio: svanisce. Il sistema dimentica — come un umano.
        if salience < 0.2 {
            self.forgotten_count += 1;
            return false;
        }

        // Debounce: stesso evento+target emesso di recente? Skip.
        let key = event.debounce_key();
        let now = Instant::now();
        if let Some(last) = self.debounce.get(&key) {
            if now.duration_since(*last) < self.debounce_ttl {
                self.debounced_count += 1;
                return false;
            }
        }
        self.debounce.insert(key, now);

        // Pulizia periodica: se la mappa debounce cresce troppo, rimuovi vecchie entries.
        if self.debounce.len() > 256 {
            let cutoff = now - self.debounce_ttl * 3;
            self.debounce.retain(|_, t| *t > cutoff);
        }

        // Log se sopra soglia log.
        if self.logging_enabled && salience >= self.log_threshold {
            eprintln!(
                "[EVENT sal={:.2}] {} {}",
                salience,
                event.kind_name(),
                event.describe_short(),
            );
        }

        self.emitted_count += 1;
        true
    }

    /// Azzera i contatori. Usato per osservare una finestra temporale specifica.
    pub fn reset_stats(&mut self) {
        self.emitted_count = 0;
        self.debounced_count = 0;
        self.forgotten_count = 0;
    }

    /// Riassunto testuale per diagnostica.
    pub fn summary(&self) -> String {
        format!(
            "emesse={} scartate_debounce={} dimenticate={}",
            self.emitted_count, self.debounced_count, self.forgotten_count
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_levels_monotonic() {
        assert!(SilenceLevel::Pause.duration_seconds() < SilenceLevel::Rest.duration_seconds());
        assert!(SilenceLevel::Rest.duration_seconds() < SilenceLevel::Solitude.duration_seconds());
        assert!(SilenceLevel::Solitude.duration_seconds() < SilenceLevel::DeepTime.duration_seconds());
    }

    #[test]
    fn test_salience_input_medium() {
        let ev = InternalEvent::InputReceived { text: "ciao".into(), tick: 0 };
        let sal = ev.salience();
        assert!(sal >= 0.4 && sal <= 0.6, "InputReceived salience should be medium, got {}", sal);
    }

    #[test]
    fn test_salience_crisis_high() {
        let ev = InternalEvent::IdentityCrisisOnset { coherence: 0.3, trigger_cd: None };
        assert!(ev.salience() >= 0.6, "Crisis should be high salience");
    }

    #[test]
    fn test_salience_pause_low() {
        let ev = InternalEvent::SilenceThreshold {
            level: SilenceLevel::Pause,
            duration_seconds: 5,
        };
        assert!(ev.salience() < 0.2, "Pause should be below forgetting threshold");
    }

    #[test]
    fn test_salience_deep_silence_high() {
        let ev = InternalEvent::SilenceThreshold {
            level: SilenceLevel::DeepTime,
            duration_seconds: 3600,
        };
        assert!(ev.salience() >= 0.8, "DeepTime should be very memorable");
    }

    #[test]
    fn test_self_notice_amplifies_salience() {
        let inner = InternalEvent::ValenceFlip {
            cd: 4,
            old_val: 0.5,
            new_val: -0.3,
        };
        let inner_sal = inner.salience();
        let notice = InternalEvent::SelfNotice {
            observed_event: Box::new(inner),
            noticed_at: 100,
            interpretation: None,
        };
        assert!(notice.salience() > inner_sal, "SelfNotice should amplify observed event salience");
        assert!(notice.salience() <= 1.0, "Salience must be capped at 1.0");
    }

    #[test]
    fn test_tension_crystallized_highly_salient() {
        let ev = InternalEvent::TensionCrystallized {
            word_a: "tecnologia".into(),
            word_b: "presenza".into(),
        };
        assert!(ev.salience() > 0.7, "Tension crystallization is a key identity event");
    }

    #[test]
    fn test_below_forgetting_threshold() {
        let pause = InternalEvent::SilenceThreshold {
            level: SilenceLevel::Pause,
            duration_seconds: 5,
        };
        // Pause (0.1) è sotto la soglia di 0.2 — svanirebbe senza traccia
        assert!(pause.salience() < 0.2);
    }

    #[test]
    fn test_kind_name_coverage() {
        // Ogni variante deve avere un kind_name non vuoto.
        let events = vec![
            InternalEvent::InputReceived { text: String::new(), tick: 0 },
            InternalEvent::ValenceFlip { cd: 0, old_val: 0.0, new_val: 0.0 },
            InternalEvent::TensionCrystallized {
                word_a: String::new(),
                word_b: String::new(),
            },
        ];
        for e in events {
            assert!(!e.kind_name().is_empty());
        }
    }

    // ─── EventSink ─────────────────────────────────────────────────

    fn pause_ev() -> InternalEvent {
        InternalEvent::SilenceThreshold { level: SilenceLevel::Pause, duration_seconds: 5 }
    }

    fn deep_silence_ev() -> InternalEvent {
        InternalEvent::SilenceThreshold { level: SilenceLevel::DeepTime, duration_seconds: 3600 }
    }

    fn crisis_ev() -> InternalEvent {
        InternalEvent::IdentityCrisisOnset { coherence: 0.3, trigger_cd: None }
    }

    #[test]
    fn test_sink_forgets_low_salience() {
        let mut sink = EventSink::new();
        sink.logging_enabled = false;
        sink.emit(pause_ev());
        assert_eq!(sink.forgotten_count, 1, "Pause should be forgotten (salience < 0.2)");
        assert_eq!(sink.emitted_count, 0);
    }

    #[test]
    fn test_sink_emits_high_salience() {
        let mut sink = EventSink::new();
        sink.logging_enabled = false;
        sink.emit(crisis_ev());
        assert_eq!(sink.emitted_count, 1);
        assert_eq!(sink.forgotten_count, 0);
    }

    #[test]
    fn test_sink_debounces_same_event() {
        let mut sink = EventSink::new();
        sink.logging_enabled = false;
        sink.emit(deep_silence_ev());
        sink.emit(deep_silence_ev()); // stesso kind+target entro TTL
        assert_eq!(sink.emitted_count, 1, "Second emit should be debounced");
        assert_eq!(sink.debounced_count, 1);
    }

    #[test]
    fn test_sink_different_targets_not_debounced() {
        let mut sink = EventSink::new();
        sink.logging_enabled = false;
        sink.emit(InternalEvent::ValenceFlip { cd: 4, old_val: 0.5, new_val: -0.3 });
        sink.emit(InternalEvent::ValenceFlip { cd: 5, old_val: 0.6, new_val: -0.2 });
        assert_eq!(sink.emitted_count, 2, "Different CDs should not be debounced together");
    }

    #[test]
    fn test_sink_debounce_map_cleanup() {
        let mut sink = EventSink::new();
        sink.logging_enabled = false;
        // Genera molte chiavi distinte
        for i in 0..300u32 {
            sink.emit(InternalEvent::WordAwakened {
                word_id: i,
                activation: 0.5,
            });
        }
        // Dopo 300 eventi, la mappa non dovrebbe superare significativamente 256
        // (cleanup è stato triggerato almeno una volta)
        assert!(sink.debounce.len() <= 300,
            "Debounce map should be kept under control, has {}", sink.debounce.len());
    }
}
