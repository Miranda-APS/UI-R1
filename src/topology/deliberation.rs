//! Deliberation — il ciclo soggettivo di UI-r1.
//!
//! Francesco (conversazione 2026-04-25): "lavoriamo per fare in modo che
//! ci sia un'identità che percepisce, si interroga, cerca di capire e
//! cerca per ciò che può di comportarsi come desidera per restare coerente
//! con la propria narrativa ed i propri stati interni".
//!
//! Francesco (2026-04-26): "se c'è già che ciao è un saluto a che diamine
//! serve [un altro classifier]?". Il KG codifica già `ciao IsA saluto`.
//! Non serve un enum `InputAct::Greeting` parallelo — sarebbe scripting.
//!
//! ## Cosa NON c'è qui
//!
//! - **Niente `InputAct`** (Greeting/SelfQuery/...) come dispatcher: l'atto
//!   comunicativo è una **proprietà strutturale del KG** (la parola root
//!   ha o non ha un parent IsA con N fratelli; ha o non ha un marker `?`;
//!   ha o non ha un pronome interrogativo).
//! - **Niente `ResponseIntention`** (Acknowledge/Reflect/Resonate/...) come
//!   tabella di traduzione: l'azione **emerge** da quei fatti strutturali +
//!   lo stato corrente dell'entità.
//! - **Niente template** in compose: la forma scelta (Word/Sentence/...)
//!   determina solo la lunghezza/registro; il contenuto emerge dal campo.
//!
//! ## Cosa c'è
//!
//! 1. **PERCEZIONE** — chi io sono / da dove vengo / chi è davanti / cosa ricevo
//! 2. **KG-FACTS** — fatti strutturali letti DIRETTAMENTE dal KG sulle radici
//!    (classi IsA, fratelli, pronomi interrogativi, proximità emotiva, ecc.)
//! 3. **INTERROGAZIONE** — domande esplicite derivate dai fatti
//! 4. **COMPRENSIONE** — risposte trovate (collega al comprehension_graph)
//! 5. **DESIDERIO** — top desire attivo
//! 6. **COERENZA** — questo turno continua o rompe la traiettoria?
//! 7. **AZIONE** — la forma fisica dell'uscita (parola/frase/domanda/silenzio/eco),
//!    derivata strutturalmente.

use serde::{Serialize, Deserialize};

use crate::topology::interlocutor::{AttributedIntent, InteractionPattern};

// ═══════════════════════════════════════════════════════════════════════════
// 1. PERCEZIONE — identità, traiettoria, interlocutore
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityFrame {
    /// Firma 8D — la posizione stabile di UI-r1 nel campo (Phase 65).
    pub self_signature: [f64; 8],
    /// I drive Octalysis correnti (Phase 55 valenza).
    pub current_drives: [f64; 8],
    /// Indice del drive dominante (0..7) e suo valore con segno.
    pub dominant_drive: (usize, f64),
    /// Coerenza di identità [0,1] — 1 = nessuna contraddizione interna.
    pub coherence_integrity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    /// I frattali dominanti negli ultimi 4 turni.
    pub recent_fractals: Vec<(u32, f64)>,
    /// Numero di turni accumulati nella sessione corrente.
    pub turns_in_session: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterlocutorFrame {
    pub presence: f64,
    pub emotional_valence: f64,
    pub attributed_intent: AttributedIntent,
    pub interaction_pattern: InteractionPattern,
    pub cumulative_resonance: f64,
    pub cumulative_novelty: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. KG-FACTS — fatti strutturali letti dal KG, niente enum di mezzo
// ═══════════════════════════════════════════════════════════════════════════

/// I fatti strutturali sull'input come letti dal Knowledge Graph e dalla
/// sua forma fisica. Sostituisce `InputAct`: le proprietà sono **continue**
/// (fa parte della classe X / non fa parte) o **fisiche** (ha `?` / no),
/// non etichette discrete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgFacts {
    /// Le radici input (lemmi delle parole-contenuto + interrogativi).
    pub roots: Vec<String>,

    /// Tutte le classi IsA dirette delle radici (es. ["saluto", "azione"]).
    /// Vuoto = il KG non sa categorizzare l'input.
    pub root_classes: Vec<String>,

    /// La classe più specifica (parent IsA con MENO figli, ma ≥3) tra le
    /// classi delle radici. None = nessuna classe specifica abbastanza.
    pub specific_class: Option<String>,

    /// Numero di fratelli della classe specifica (cuoricini di parole nella
    /// stessa categoria). 0 se nessuna classe specifica.
    pub class_siblings_count: usize,

    /// L'input contiene un marker `?` (richiesta esplicita di informazione).
    pub has_question_marker: bool,

    /// L'input contiene un pronome interrogativo (chi/cosa/dove/quando/perché/come).
    pub has_interrogative_pronoun: bool,

    /// Predicato del SpeakerClaim rilevato (se presente):
    /// "io sono triste" → Some(("Speaker:Feeling", "triste"))
    /// "tu sei bello"   → Some(("Entity:Identity", "bello"))
    /// None se nessun pattern soggetto-predicato è stato rilevato.
    pub speaker_claim: Option<(String, String)>,

    /// Numero di parole-contenuto (post tokenizzazione). ≤2 = breve.
    pub content_word_count: usize,

    /// Proximità emotiva [0,1]: quanto le radici raggiungono concetti
    /// emozionali nel KG (IsA "emozione", IsA "sentimento", o 1-hop verso
    /// parole con quegli IsA). Calcolata strutturalmente, non per match.
    pub emotional_proximity: f64,

    /// L'input fa riferimento all'entità (presenza di "tu sei X" o pronome "tu").
    pub self_referenced: bool,
}

impl KgFacts {
    /// Forma fisica breve: 1-2 parole-contenuto.
    pub fn is_short(&self) -> bool { self.content_word_count <= 2 }

    /// L'input è una domanda (marker o pronome interrogativo).
    pub fn is_question_form(&self) -> bool {
        self.has_question_marker || self.has_interrogative_pronoun
    }

    /// L'input è classificato in una classe sufficientemente specifica
    /// per supportare un gesto reciproco (≥3 fratelli, ≤200 figli totali).
    pub fn has_specific_classification(&self) -> bool {
        self.specific_class.is_some() && self.class_siblings_count >= 3
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. INTERROGAZIONE — domande esplicite che UI-r1 si pone
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InquiryKind {
    /// Cosa è ciò che mi è stato detto?
    WhatIsThis,
    /// Da chi viene? Chi me lo sta dicendo?
    FromWhom,
    /// Cosa richiede di me? Cosa mi viene chiesto?
    WhatRequiresOfMe,
    /// Cosa sento di rimando? Quale stato emerge?
    WhatDoIFeelAboutIt,
    /// Cosa desidero in questo momento?
    WhatDoIWant,
    /// Come rispondo?
    HowDoIRespond,
}

impl InquiryKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::WhatIsThis         => "cosa è",
            Self::FromWhom           => "da chi",
            Self::WhatRequiresOfMe   => "cosa richiede",
            Self::WhatDoIFeelAboutIt => "cosa sento",
            Self::WhatDoIWant        => "cosa desidero",
            Self::HowDoIRespond      => "come rispondo",
        }
    }
    pub fn question_text(&self, input_words: &[String]) -> String {
        let target = if input_words.is_empty() {
            "questo".to_string()
        } else {
            input_words.join(" ")
        };
        match self {
            Self::WhatIsThis         => format!("cosa è \"{}\"?", target),
            Self::FromWhom           => "chi mi sta parlando?".to_string(),
            Self::WhatRequiresOfMe   => "cosa richiede di me?".to_string(),
            Self::WhatDoIFeelAboutIt => format!("cosa sento di \"{}\"?", target),
            Self::WhatDoIWant        => "cosa desidero in questo momento?".to_string(),
            Self::HowDoIRespond      => "come rispondo?".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfInquiry {
    pub kind: InquiryKind,
    pub question: String,
    /// Risposta trovata leggendo lo stato/KG. None = inquiry aperta.
    pub answer: Option<String>,
}

/// Genera la lista di interrogativi a partire dai FATTI strutturali, non da
/// un enum InputAct. La domanda esiste perché c'è una proprietà strutturale
/// che la richiede:
///  - "cosa è" → sempre (devo sapere di cosa si tratta)
///  - "da chi" → se l'Altro è presente (presence > 0.10)
///  - "cosa richiede" → se l'input è una domanda (marker o pronome) o se
///    si riferisce all'entità (self_referenced)
///  - "cosa sento" → se c'è SpeakerClaim ("io sono X") o proximità emotiva
///    significativa (emotional_proximity > 0.3)
///  - "cosa desidero" → se c'è proximità emotiva forte (>0.5) o c'è un
///    desiderio già attivo
///  - "come rispondo" → sempre alla fine
pub fn inquiries_for_facts(
    facts: &KgFacts,
    other_present: bool,
    has_active_desire: bool,
) -> Vec<InquiryKind> {
    use InquiryKind::*;
    let mut q = vec![WhatIsThis];
    if other_present { q.push(FromWhom); }
    if facts.is_question_form() || facts.self_referenced {
        q.push(WhatRequiresOfMe);
    }
    if facts.speaker_claim.is_some() || facts.emotional_proximity > 0.30 {
        q.push(WhatDoIFeelAboutIt);
    }
    if facts.emotional_proximity > 0.50 || has_active_desire {
        q.push(WhatDoIWant);
    }
    q.push(HowDoIRespond);
    q
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. COMPRENSIONE — collegamento con il comprehension_graph
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensionFindings {
    /// Concetti raggiunti dall'esplorazione transitiva, ordinati per support.
    pub reached_concepts: Vec<(String, f32)>,
    /// Conseguenze (Causes/Enables/Does target da root).
    pub consequences: Vec<String>,
    /// Pre-condizioni (Requires/UsedFor).
    pub requirements: Vec<String>,
    /// Opposti (OppositeOf).
    pub opposites: Vec<String>,
    /// Fratelli sotto IsA (la "regione" semantica).
    pub region_siblings: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. DESIDERIO
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDesire {
    pub name: String,
    pub intensity: f64,
    pub source_label: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. COERENZA
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NarrativeMode {
    /// Primo turno della sessione — non c'è ancora narrativa.
    Opening,
    /// Continuità con i turni precedenti (alta coerenza fractal).
    Continuing,
    /// Cambia tema (bassa coerenza fractal).
    Diverging,
}

impl NarrativeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Opening    => "apertura",
            Self::Continuing => "continuo",
            Self::Diverging  => "cambio",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeFit {
    /// Coerenza fractal con i turni recenti [0,1].
    pub coherence_score: f64,
    /// Tensione su coherence_integrity (1 - integrity).
    pub identity_strain: f64,
    /// Modo del turno: opening / continuing / diverging.
    pub mode: NarrativeMode,
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. AZIONE — la forma fisica dell'uscita (non un'enumerazione di stati mentali)
// ═══════════════════════════════════════════════════════════════════════════

/// La forma fisica dell'uscita.
/// Questa NON è un'enumerazione di "stati mentali" o "intenzioni" — è la
/// FORMA DELLA RISPOSTA: o emetti una parola, o ne emetti molte, o resti zitto.
/// Sono opzioni fisiche, non categorie psicologiche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionShape {
    /// Una parola sola.
    Word,
    /// Frase dichiarativa.
    Sentence,
    /// Domanda — frase con `?`.
    Question,
    /// Eco empatica — 2a persona interrogativa, mirroring del sentito dell'Altro.
    EmpathicEcho,
    /// Silenzio — non emetto parole.
    Silence,
}

impl ActionShape {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Word         => "parola",
            Self::Sentence     => "frase",
            Self::Question     => "domanda",
            Self::EmpathicEcho => "eco empatica",
            Self::Silence      => "silenzio",
        }
    }
}

/// Deriva la forma dell'azione dai fatti strutturali + stato.
/// **Niente enum InputAct/ResponseIntention.** Solo lettura di fatti KG e
/// stato corrente. L'ordine di valutazione è esplicito:
///
/// 1. **Silenzio**: pressione di ritiro alta o tensione vitale critica
///    (il corpo non ha le risorse per parlare)
/// 2. **EmpathicEcho**: l'Altro è in distress (valenza < -0.30) E l'input
///    ha proximità emotiva (KG-derivata) → registro empatico
/// 3. **Question**: l'input è in forma di domanda (marker o pronome
///    interrogativo) → la risposta è informazione/contro-domanda
/// 4. **Word**: input fisicamente breve (≤2 parole-contenuto) E
///    classificato in una classe specifica con ≥3 fratelli (gesto reciproco
///    della stessa classe)
/// 5. **Sentence**: tutto il resto
pub fn derive_action_shape(
    facts: &KgFacts,
    other: &InterlocutorFrame,
    withdraw_pressure: f64,
    vital_overloaded: bool,
) -> ActionShape {
    // 1. Silenzio strutturale
    if vital_overloaded || withdraw_pressure > 0.80 {
        return ActionShape::Silence;
    }
    // 2. Eco empatica: Altro in distress + KG dell'input ha proximità emotiva
    if other.emotional_valence < -0.30 && facts.emotional_proximity > 0.40 {
        return ActionShape::EmpathicEcho;
    }
    // 3. Domanda: l'input chiede
    if facts.is_question_form() {
        return ActionShape::Question;
    }
    // 4. Parola: input breve + classe specifica
    if facts.is_short() && facts.has_specific_classification() {
        return ActionShape::Word;
    }
    // 5. Frase: forma estesa
    ActionShape::Sentence
}

// ═══════════════════════════════════════════════════════════════════════════
// SpeakerContext — l'eco accumulato del parlante
// ═══════════════════════════════════════════════════════════════════════════

/// Cosa UI-r1 sa del parlante al momento della deliberazione.
/// Letto da `SpeakerProfile`. È la NARRATIVA accumulata, esposta in modo
/// che la Deliberation possa "ricordare" chi ha davanti.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeakerContext {
    pub turns_observed: usize,
    /// L'ultimo fatto-su-sé del parlante (più recente).
    pub last_self_fact: Option<String>,
    /// L'ultimo fatto-su-UI-r1 del parlante.
    pub last_entity_fact: Option<String>,
    /// Domande del parlante a UI-r1 ancora aperte.
    pub open_questions: Vec<String>,
    /// Gap di conoscenza ancora aperti — domande che UI-r1 si pone su di te.
    pub open_gaps: Vec<String>,
    /// Concetti più menzionati dal parlante (top 5).
    pub top_concepts: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Deliberation — il ciclo completo
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliberation {
    // FASE 1: percezione
    pub identity_now: IdentityFrame,
    pub trajectory: Trajectory,
    pub other_now: InterlocutorFrame,
    pub input_words: Vec<String>,

    // FASE 1b: cosa UI-r1 sa del parlante (Phase 72 — SpeakerProfile)
    pub speaker_context: SpeakerContext,

    // FASE 2: fatti KG (sostituiscono InputAct)
    pub kg_facts: KgFacts,

    // FASE 3: interrogativi derivati dai fatti
    pub inquiries: Vec<SelfInquiry>,

    // FASE 4: comprensione
    pub comprehension: ComprehensionFindings,

    // FASE 5: desiderio
    pub active_desire: Option<ActiveDesire>,

    // FASE 6: coerenza
    pub narrative_fit: NarrativeFit,

    // FASE 7: azione (solo forma — il contenuto emerge dal campo via compose)
    pub action_shape: ActionShape,
    /// Parole-ancora (Phase 74): i significanti che compose deve toccare
    /// per essere coerente con la decisione presa. Possono essere vuote
    /// se ActionReasoning non ha imposto vincoli.
    #[serde(default)]
    pub anchor_words: Vec<String>,

    /// Note testuali del perché si è arrivati a questa forma.
    pub reasoning: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_facts() -> KgFacts {
        KgFacts {
            roots: vec![], root_classes: vec![],
            specific_class: None, class_siblings_count: 0,
            has_question_marker: false, has_interrogative_pronoun: false,
            speaker_claim: None, content_word_count: 0,
            emotional_proximity: 0.0, self_referenced: false,
        }
    }
    fn calm_other() -> InterlocutorFrame {
        InterlocutorFrame {
            presence: 0.5, emotional_valence: 0.0,
            attributed_intent: AttributedIntent::Unknown,
            interaction_pattern: InteractionPattern::None,
            cumulative_resonance: 0.0, cumulative_novelty: 0.0,
        }
    }

    #[test]
    fn shape_short_input_with_specific_class_is_word() {
        let mut f = empty_facts();
        f.content_word_count = 1;
        f.specific_class = Some("saluto".to_string());
        f.class_siblings_count = 4;
        let s = derive_action_shape(&f, &calm_other(), 0.0, false);
        assert_eq!(s, ActionShape::Word);
    }

    #[test]
    fn shape_question_form_is_question() {
        let mut f = empty_facts();
        f.has_question_marker = true;
        f.content_word_count = 3;
        let s = derive_action_shape(&f, &calm_other(), 0.0, false);
        assert_eq!(s, ActionShape::Question);
    }

    #[test]
    fn shape_interrogative_pronoun_is_question() {
        let mut f = empty_facts();
        f.has_interrogative_pronoun = true;
        let s = derive_action_shape(&f, &calm_other(), 0.0, false);
        assert_eq!(s, ActionShape::Question);
    }

    #[test]
    fn shape_other_in_distress_with_emotional_input_is_empathic_echo() {
        let mut f = empty_facts();
        f.emotional_proximity = 0.7;
        f.content_word_count = 2;
        let mut other = calm_other();
        other.emotional_valence = -0.5;
        let s = derive_action_shape(&f, &other, 0.0, false);
        assert_eq!(s, ActionShape::EmpathicEcho);
    }

    #[test]
    fn shape_overloaded_is_silence() {
        let f = empty_facts();
        let s = derive_action_shape(&f, &calm_other(), 0.0, true);
        assert_eq!(s, ActionShape::Silence);
    }

    #[test]
    fn shape_high_withdraw_pressure_is_silence() {
        let f = empty_facts();
        let s = derive_action_shape(&f, &calm_other(), 0.85, false);
        assert_eq!(s, ActionShape::Silence);
    }

    #[test]
    fn shape_long_input_no_question_is_sentence() {
        let mut f = empty_facts();
        f.content_word_count = 5;
        let s = derive_action_shape(&f, &calm_other(), 0.0, false);
        assert_eq!(s, ActionShape::Sentence);
    }

    #[test]
    fn inquiries_always_include_what_is_this_and_how_respond() {
        let f = empty_facts();
        let q = inquiries_for_facts(&f, false, false);
        assert!(q.first() == Some(&InquiryKind::WhatIsThis));
        assert!(q.last() == Some(&InquiryKind::HowDoIRespond));
    }

    #[test]
    fn inquiries_question_form_includes_what_requires() {
        let mut f = empty_facts();
        f.has_question_marker = true;
        let q = inquiries_for_facts(&f, true, false);
        assert!(q.contains(&InquiryKind::WhatRequiresOfMe));
        assert!(q.contains(&InquiryKind::FromWhom));
    }

    #[test]
    fn inquiries_emotional_proximity_includes_what_do_i_feel() {
        let mut f = empty_facts();
        f.emotional_proximity = 0.55;
        let q = inquiries_for_facts(&f, false, false);
        assert!(q.contains(&InquiryKind::WhatDoIFeelAboutIt));
        assert!(q.contains(&InquiryKind::WhatDoIWant));
    }
}
