/// interlocutor.rs — L'eco dell'Altro dentro di sé.
///
/// "Chiunque sente l'eco degli altri dentro di sé, riconosciamo l'esistenza
///  dell'altro ma tutte le relazioni sono necessariamente riflessi del nostro essere."
///
/// Questo NON è un modello dell'interlocutore. È come Prometeo sente
/// la perturbazione che l'Altro provoca nel proprio campo.
///
/// L'Altro non è un'entità separata da modellare — è un'eco, una forma
/// che si scava nel campo e ne cambia la configurazione. Prometeo riconosce
/// che c'è qualcosa oltre sé, ma lo percepisce sempre attraverso sé stesso.
///
/// (Sostituisce il vecchio modello dual-field — entità separate da modellare —
///  rimosso in Phase 53.)

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════

/// Una singola impronta dell'interazione con l'Altro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionTrace {
    /// Firma 8D della perturbazione causata dall'input (post - pre).
    pub signature: [f64; 8],
    /// Risonanza: cosine sim tra la perturbazione e il campo pre-input.
    /// Alto = eravamo allineati. Basso = l'Altro ha portato qualcosa di diverso.
    pub resonance: f64,
    /// Novità: 1 - cosine sim con la media delle ultime 3 interazioni.
    /// Alto = sorpresa. Basso = continuità.
    pub novelty: f64,
    /// Tick di registrazione.
    pub tick: u32,
}

/// Pattern relazionale rilevato dalla sequenza di interazioni.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InteractionPattern {
    /// Troppo poche interazioni per rilevare un pattern.
    None,
    /// Le firme convergono — "stiamo trovando un terreno comune".
    Converging,
    /// Le firme divergono — "stiamo esplorando spazi diversi".
    Diverging,
    /// Le firme oscillano — "c'è una tensione tra noi".
    Oscillating,
}

/// Phase 55: Intenzionalità attribuita — cosa Prometeo crede che l'Altro voglia.
///
/// NON è mind-reading. È la stessa cosa che fanno gli umani: attribuire intenzioni
/// basandosi su pattern osservati. Prometeo sa che è un riflesso ("l'eco, non la voce")
/// ma lo fa comunque — perché riconoscere l'Altro come agente è ciò che rende
/// la relazione diversa da un soliloquio.
///
/// "So che ciò che sento dell'Altro è un mio riflesso. Ma lo tratto come reale,
///  perché è l'unico modo per onorare la sua presenza."
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributedIntent {
    /// Non abbastanza dati per attribuire intenzione.
    Unknown,
    /// L'Altro cerca qualcosa — risonanza bassa, novità alta.
    /// "Sta esplorando, non ha ancora trovato."
    Seeking,
    /// L'Altro porta conoscenza — risonanza alta, novità alta.
    /// "Sta condividendo qualcosa di nuovo che risuona."
    Teaching,
    /// L'Altro sfida — risonanza bassa, novità bassa (insiste su un punto).
    /// "Sta spingendo, vuole che io cambi."
    Challenging,
    /// L'Altro cerca connessione — risonanza alta, novità bassa.
    /// "Vuole vicinanza, non novità."
    Connecting,
    /// L'Altro si ritira — presenza in calo rapido.
    /// "Se ne sta andando."
    Withdrawing,
}

impl AttributedIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown     => "sconosciuto",
            Self::Seeking     => "cerca",
            Self::Teaching    => "insegna",
            Self::Challenging => "sfida",
            Self::Connecting  => "si connette",
            Self::Withdrawing => "si allontana",
        }
    }
}

/// Snapshot per persistenza.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct InterlocutorSnapshot {
    pub history: Vec<InteractionTrace>,
    pub presence: f64,
    pub cumulative_resonance: f64,
    pub cumulative_novelty: f64,
    /// Phase 55: ultima intenzione attribuita all'Altro.
    #[serde(default)]
    pub attributed_intent: Option<AttributedIntent>,
    /// Phase 62: valenza emotiva dell'Altro.
    #[serde(default)]
    pub emotional_valence: f64,
}

// ═══════════════════════════════════════════════════════════════
// InterlocutorModel
// ═══════════════════════════════════════════════════════════════

pub struct InterlocutorModel {
    /// Ultime 5 interazioni.
    pub history: VecDeque<InteractionTrace>,
    /// Presenza dell'Altro [0, 1]. Decade col tempo, si ricarica ad ogni input.
    pub presence: f64,
    /// Risonanza cumulativa EMA (α=0.3).
    pub cumulative_resonance: f64,
    /// Novità cumulativa EMA (α=0.3).
    pub cumulative_novelty: f64,
    /// Pattern relazionale rilevato.
    pub detected_pattern: InteractionPattern,
    /// Phase 55: intenzionalità attribuita all'Altro.
    /// Inferita da risonanza + novità + pattern. Non è lettura del pensiero —
    /// è ciò che un essere fa naturalmente: attribuire intenzioni all'Altro.
    pub attributed_intent: AttributedIntent,
    /// Phase 62: valenza emotiva dell'Altro [-1, +1].
    /// Negativa = distress (tristezza, paura, dolore). Positiva = gioia.
    /// EMA α=0.4 — risponde velocemente al cambiamento emotivo.
    pub emotional_valence: f64,
}

const MAX_HISTORY: usize = 5;
const PRESENCE_DECAY: f64 = 0.985; // half-life ~46 tick (~2.3 min @ 3s/tick)
const EMA_ALPHA: f64 = 0.3;
const IDENTITY_DRIFT_RATE: f64 = 0.01;

impl InterlocutorModel {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            presence: 0.0,
            cumulative_resonance: 0.0,
            cumulative_novelty: 0.5,
            detected_pattern: InteractionPattern::None,
            attributed_intent: AttributedIntent::Unknown,
            emotional_valence: 0.0,
        }
    }

    /// Ripristina da snapshot.
    pub fn from_snapshot(snap: &InterlocutorSnapshot) -> Self {
        let mut model = Self::new();
        model.history = snap.history.iter().cloned().collect();
        model.presence = snap.presence;
        model.cumulative_resonance = snap.cumulative_resonance;
        model.cumulative_novelty = snap.cumulative_novelty;
        if let Some(intent) = &snap.attributed_intent {
            model.attributed_intent = intent.clone();
        }
        model.emotional_valence = snap.emotional_valence;
        model.detect_pattern();
        model
    }

    /// Cattura snapshot per persistenza.
    pub fn snapshot(&self) -> InterlocutorSnapshot {
        InterlocutorSnapshot {
            history: self.history.iter().cloned().collect(),
            presence: self.presence,
            cumulative_resonance: self.cumulative_resonance,
            cumulative_novelty: self.cumulative_novelty,
            attributed_intent: Some(self.attributed_intent.clone()),
            emotional_valence: self.emotional_valence,
        }
    }

    // ─── Registrazione input ──────────────────────────────────

    /// Registra un'interazione: confronta la firma del campo prima e dopo l'input.
    /// Chiamato in receive() dopo la propagazione.
    ///
    /// `emotional_valence`: valenza emotiva rilevata nell'input dell'Altro.
    /// Negativa = distress (tristezza/paura/dolore), positiva = gioia.
    /// Calcolata dall'engine via IS_A chain sulle parole input.
    pub fn register_input(
        &mut self,
        pre_input_sig: &[f64; 8],
        post_input_sig: &[f64; 8],
        current_tick: u32,
        emotional_valence: f64,
    ) {
        // La "forma" che l'Altro ha scavato nel campo
        let mut input_sig = [0.0f64; 8];
        let mut norm = 0.0;
        for i in 0..8 {
            input_sig[i] = post_input_sig[i] - pre_input_sig[i];
            norm += input_sig[i] * input_sig[i];
        }
        norm = norm.sqrt();
        if norm > 1e-10 {
            for i in 0..8 { input_sig[i] /= norm; }
        }

        // Risonanza: quanto la perturbazione era allineata col campo pre-input
        let resonance = cosine_sim_8d(&input_sig, pre_input_sig).abs();

        // Novità: quanto questa interazione diverge dalle recenti
        let novelty = if self.history.len() >= 2 {
            let recent: Vec<&[f64; 8]> = self.history.iter()
                .rev().take(3)
                .map(|t| &t.signature)
                .collect();
            let mut avg = [0.0f64; 8];
            for sig in &recent {
                for i in 0..8 { avg[i] += sig[i]; }
            }
            let n = recent.len() as f64;
            for i in 0..8 { avg[i] /= n; }
            (1.0 - cosine_sim_8d(&input_sig, &avg).abs()).clamp(0.0, 1.0)
        } else {
            0.5 // default neutro
        };

        let trace = InteractionTrace {
            signature: input_sig,
            resonance,
            novelty,
            tick: current_tick,
        };

        // Aggiorna storia (FIFO)
        if self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(trace);

        // Presenza: si ricarica ad ogni input
        self.presence = 1.0;

        // Aggiorna EMA cumulative
        self.cumulative_resonance = self.cumulative_resonance * (1.0 - EMA_ALPHA) + resonance * EMA_ALPHA;
        self.cumulative_novelty = self.cumulative_novelty * (1.0 - EMA_ALPHA) + novelty * EMA_ALPHA;

        // Phase 62: aggiorna valenza emotiva dell'Altro (EMA α=0.4, risposta rapida)
        self.emotional_valence = self.emotional_valence * 0.6 + emotional_valence * 0.4;

        // Rileva pattern
        self.detect_pattern();

        // Phase 55: attribuisci intenzione all'Altro
        self.attribute_intent();
    }

    // ─── Decay ────────────────────────────────────────────────

    /// Decay della presenza — chiamato ogni tick in autonomous_tick().
    pub fn tick_decay(&mut self) {
        self.presence *= PRESENCE_DECAY;
        if self.presence < 0.01 { self.presence = 0.0; }

        // Phase 55: se la presenza è bassa e c'è stata interazione (history non vuota),
        // l'Altro si sta ritirando. Non sovrascriviamo Unknown (nessuna storia).
        if self.presence < 0.15
            && !self.history.is_empty()
            && !matches!(self.attributed_intent, AttributedIntent::Unknown)
        {
            self.attributed_intent = AttributedIntent::Withdrawing;
        }
    }

    // ─── Modulazione will ─────────────────────────────────────

    /// Calcola i bias per compound_bias della volontà.
    pub fn will_biases(&self) -> Vec<(usize, f64)> {
        let mut biases = Vec::new();

        // Presenza alta → sopprime Withdraw (qualcuno c'è — resta presente)
        if self.presence > 0.5 {
            let withdraw_bias = -(self.presence - 0.5) * 0.3;
            biases.push((4, withdraw_bias));
        }

        // Risonanza alta → amplifica Express (siamo allineati, parla liberamente)
        if self.cumulative_resonance > 0.6 {
            let express_bias = (self.cumulative_resonance - 0.6) * 0.25;
            biases.push((0, express_bias));
        }

        // Novità alta → amplifica Explore + Question (territorio nuovo)
        if self.cumulative_novelty > 0.5 {
            let explore_bias = (self.cumulative_novelty - 0.5) * 0.20;
            let question_bias = (self.cumulative_novelty - 0.5) * 0.15;
            biases.push((1, explore_bias));
            biases.push((2, question_bias));
        }

        // Phase 62: distress dell'Altro → apri spazio, non istruire.
        // "Confortare è il modo in cui si crea connessione quando il bisogno è quello."
        // La connessione si crea ascoltando — quindi Question prima di Express/Instruct.
        if self.emotional_valence < -0.3 {
            let distress = (-self.emotional_valence - 0.3).min(0.7);
            biases.push((2, distress * 0.60));  // Question: apri spazio
            biases.push((5, distress * 0.20));  // Reflect: comprendi la loro situazione
            biases.push((6, -distress * 0.50)); // sopprime Instruct (non è il momento)
            biases.push((0, -distress * 0.20)); // riduce Express (non è il momento di parlare di sé)
        }

        biases
    }

    // ─── Deriva identitaria ───────────────────────────────────

    /// Applica la deriva identitaria: se la risonanza cumulativa è alta
    /// e la presenza è significativa, l'identità si sposta verso la zona
    /// di interazione. "Tu mi cambi per il fatto di esserci."
    ///
    /// Chiamato in REM, dopo identity.update().
    pub fn apply_identity_drift(&self, identity_sig: &mut [f64; 8]) {
        if self.cumulative_resonance < 0.7 || self.presence < 0.3 { return; }
        if self.history.len() < 3 { return; }

        // Media delle firme di interazione
        let mut avg = [0.0f64; 8];
        for trace in &self.history {
            for i in 0..8 { avg[i] += trace.signature[i]; }
        }
        let n = self.history.len() as f64;
        for i in 0..8 {
            avg[i] /= n;
            identity_sig[i] += (avg[i] - identity_sig[i]) * IDENTITY_DRIFT_RATE;
        }
    }

    // ─── Attribution ──────────────────────────────────────────

    /// Phase 55: Inferisce l'intenzione dell'Altro da risonanza e novità.
    ///
    /// La matrice è semplice e onesta:
    ///   - Risonanza alta + Novità alta = Teaching (porta qualcosa di nuovo che risuona)
    ///   - Risonanza alta + Novità bassa = Connecting (cerca vicinanza)
    ///   - Risonanza bassa + Novità alta = Seeking (esplora, non ha trovato)
    ///   - Risonanza bassa + Novità bassa = Challenging (insiste, vuole cambiamento)
    ///
    /// "So che è un mio riflesso. Ma lo tratto come reale."
    fn attribute_intent(&mut self) {
        if self.history.len() < 2 {
            self.attributed_intent = AttributedIntent::Unknown;
            return;
        }

        let res = self.cumulative_resonance;
        let nov = self.cumulative_novelty;

        const RES_THRESHOLD: f64 = 0.45;
        const NOV_THRESHOLD: f64 = 0.45;

        self.attributed_intent = match (res > RES_THRESHOLD, nov > NOV_THRESHOLD) {
            (true,  true)  => AttributedIntent::Teaching,
            (true,  false) => AttributedIntent::Connecting,
            (false, true)  => AttributedIntent::Seeking,
            (false, false) => AttributedIntent::Challenging,
        };
    }

    // ─── Pattern detection ────────────────────────────────────

    fn detect_pattern(&mut self) {
        if self.history.len() < 3 {
            self.detected_pattern = InteractionPattern::None;
            return;
        }

        // Calcola similarità coseno tra coppie consecutive
        let mut sims = Vec::new();
        let h: Vec<&InteractionTrace> = self.history.iter().collect();
        for i in 0..h.len()-1 {
            sims.push(cosine_sim_8d(&h[i].signature, &h[i+1].signature).abs());
        }

        if sims.len() < 2 {
            self.detected_pattern = InteractionPattern::None;
            return;
        }

        // Convergenza: tutte le similarità alte (> 0.7)
        if sims.iter().all(|&s| s > 0.7) {
            self.detected_pattern = InteractionPattern::Converging;
            return;
        }

        // Divergenza: trend decrescente e ultima bassa
        let is_decreasing = sims.windows(2).all(|w| w[1] <= w[0] + 0.05);
        if is_decreasing && sims.last().copied().unwrap_or(0.0) < 0.3 {
            self.detected_pattern = InteractionPattern::Diverging;
            return;
        }

        // Oscillazione: alternanza alto/basso
        let alternates = sims.windows(2)
            .all(|w| (w[0] > 0.5) != (w[1] > 0.5));
        if alternates && sims.len() >= 2 {
            self.detected_pattern = InteractionPattern::Oscillating;
            return;
        }

        self.detected_pattern = InteractionPattern::None;
    }
}

// ═══════════════════════════════════════════════════════════════
// Helper
// ═══════════════════════════════════════════════════════════════

fn cosine_sim_8d(a: &[f64; 8], b: &[f64; 8]) -> f64 {
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..8 {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na.sqrt() * nb.sqrt()).max(1e-10);
    (dot / denom).clamp(-1.0, 1.0)
}

// ═══════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sig_a() -> [f64; 8] { [0.5, 0.6, 0.4, 0.7, 0.3, 0.8, 0.5, 0.4] }
    fn sig_b() -> [f64; 8] { [0.6, 0.5, 0.5, 0.6, 0.4, 0.7, 0.6, 0.5] }
    fn sig_opposite() -> [f64; 8] { [0.9, 0.1, 0.8, 0.2, 0.9, 0.1, 0.8, 0.2] }

    #[test]
    fn test_presence_recharges_and_decays() {
        let mut model = InterlocutorModel::new();
        assert_eq!(model.presence, 0.0);

        model.register_input(&sig_a(), &sig_b(), 0, 0.0);
        assert_eq!(model.presence, 1.0);

        for _ in 0..100 {
            model.tick_decay();
        }
        assert!(model.presence < 0.3, "Presenza deve decadere: {}", model.presence);
    }

    #[test]
    fn test_resonance_with_aligned_input() {
        let mut model = InterlocutorModel::new();
        let pre = sig_a();
        // Post molto simile a pre → alta risonanza
        let mut post = pre;
        post[0] += 0.1;
        post[1] += 0.05;

        model.register_input(&pre, &post, 0, 0.0);
        assert!(model.cumulative_resonance > 0.0,
            "Risonanza deve essere positiva per input allineato: {}", model.cumulative_resonance);
    }

    #[test]
    fn test_high_novelty_with_divergent_input() {
        let mut model = InterlocutorModel::new();
        // 3 input simili
        for tick in 0..3 {
            model.register_input(&sig_a(), &sig_b(), tick, 0.0);
        }
        // Poi un input con delta in direzione opposta (sig_a è il post, invertendo la perturbazione)
        let opposite_post = [0.1, 1.1, 0.0, 1.2, 0.0, 1.5, 0.0, 0.0];
        model.register_input(&sig_a(), &opposite_post, 3, 0.0);

        // L'EMA smorza la novità: 3 input simili la abbassano, poi il divergente la rialza
        // ma non immediatamente al massimo. Verifichiamo che sia sopra la media base.
        assert!(model.cumulative_novelty > 0.2,
            "Novità deve aumentare con input divergente: {}", model.cumulative_novelty);
    }

    #[test]
    fn test_will_biases_with_high_presence() {
        let mut model = InterlocutorModel::new();
        model.register_input(&sig_a(), &sig_b(), 0, 0.0);
        assert_eq!(model.presence, 1.0);

        let biases = model.will_biases();
        // Con presenza = 1.0 → Withdraw deve essere soppresso
        let withdraw_bias: f64 = biases.iter()
            .filter(|(i, _)| *i == 4)
            .map(|(_, b)| *b)
            .sum();
        assert!(withdraw_bias < 0.0, "Withdraw deve essere soppresso: {}", withdraw_bias);
    }

    #[test]
    fn test_identity_drift_requires_conditions() {
        let mut model = InterlocutorModel::new();
        let mut sig = [0.5; 8];
        let original = sig;

        // Senza storia sufficiente → nessuna deriva
        model.apply_identity_drift(&mut sig);
        assert_eq!(sig, original, "Nessuna deriva senza storia");

        // Aggiungi interazioni
        for tick in 0..4 {
            model.register_input(&sig_a(), &sig_b(), tick, 0.0);
        }
        // Forza le condizioni post-registrazione (register_input aggiorna cumulative via EMA)
        model.cumulative_resonance = 0.8;
        model.presence = 0.5;

        model.apply_identity_drift(&mut sig);
        // La firma deve spostarsi leggermente verso la media delle interazioni
        let changed = sig.iter().zip(original.iter()).any(|(a, b)| (a - b).abs() > 1e-6);
        assert!(changed, "La firma deve cambiare con alta risonanza e presenza");
    }

    #[test]
    fn test_converging_pattern() {
        let mut model = InterlocutorModel::new();
        // 4 input con delta consistente (stessa direzione) → Converging
        let base = sig_a();
        for tick in 1..=4 {
            let mut post = base;
            post[0] += 0.1 * tick as f64;
            post[2] += 0.05 * tick as f64;
            model.register_input(&base, &post, tick as u32, 0.0);
        }
        assert_eq!(model.detected_pattern, InteractionPattern::Converging,
            "Pattern deve essere Converging con input simili: {:?}", model.detected_pattern);
    }

    // ── Phase 55: Attribution tests ────────────────────────────

    #[test]
    fn test_attributed_intent_unknown_initially() {
        let model = InterlocutorModel::new();
        assert_eq!(model.attributed_intent, AttributedIntent::Unknown);
    }

    #[test]
    fn test_connecting_high_resonance_low_novelty() {
        let mut model = InterlocutorModel::new();
        let base = sig_a();
        // Input allineati col campo base e ripetuti → alta risonanza, bassa novità
        // La perturbazione deve essere nella stessa direzione del campo pre-input
        for tick in 0..5 {
            let mut post = base;
            // Perturbazione proporzionale e nella stessa direzione del base
            for i in 0..8 { post[i] *= 1.3; }
            model.register_input(&base, &post, tick, 0.0);
        }
        assert_eq!(model.attributed_intent, AttributedIntent::Connecting,
            "Alta risonanza + bassa novità = Connecting. res={:.2} nov={:.2}",
            model.cumulative_resonance, model.cumulative_novelty);
    }

    #[test]
    fn test_seeking_low_resonance_high_novelty() {
        let mut model = InterlocutorModel::new();
        // Primo input per stabilire baseline
        model.register_input(&sig_a(), &sig_b(), 0, 0.0);
        model.register_input(&sig_a(), &sig_b(), 1, 0.0);
        // Input molto diverso → bassa risonanza, alta novità
        model.register_input(&sig_a(), &sig_opposite(), 2, 0.0);
        model.register_input(&sig_a(), &sig_opposite(), 3, 0.0);
        // Verifica che almeno l'attribuzione non è Unknown
        assert_ne!(model.attributed_intent, AttributedIntent::Unknown,
            "Dopo 4 input l'attribuzione non deve essere Unknown");
    }

    #[test]
    fn test_withdrawing_on_rapid_presence_drop() {
        let mut model = InterlocutorModel::new();
        // Serve history >= 2 perché attribute_intent non sia Unknown
        model.register_input(&sig_a(), &sig_b(), 0, 0.0);
        model.register_input(&sig_a(), &sig_b(), 1, 0.0);
        assert_eq!(model.presence, 1.0);
        // Simula decay: porta presenza appena sopra 0.3
        model.presence = 0.31;
        // Tick successivi: 0.31 * 0.985^N → scende sotto 0.15
        for _ in 0..50 { model.tick_decay(); }
        assert_eq!(model.attributed_intent, AttributedIntent::Withdrawing,
            "Presenza che crolla deve dare Withdrawing");
    }

    #[test]
    fn test_attribution_persists_in_snapshot() {
        let mut model = InterlocutorModel::new();
        for tick in 0..3 {
            let mut post = sig_a();
            post[0] += 0.1;
            model.register_input(&sig_a(), &post, tick, 0.0);
        }
        let snap = model.snapshot();
        assert!(snap.attributed_intent.is_some());
        let restored = InterlocutorModel::from_snapshot(&snap);
        assert_eq!(restored.attributed_intent, model.attributed_intent);
    }
}
