/// NarrativeSelf — l'identità narrativa di Prometeo.
///
/// Non è un profilo statistico. Non è un modello emergente.
/// È il soggetto che attraversa il ciclo deliberativo:
///   "Ho ricevuto X → capisco che si tratta di Y → la mia posizione è Z → voglio fare W"
///
/// La generazione esprime questa posizione — non la precede.
///
/// Filosofia:
///   - Le consapevolezze umane sono già nel sistema (KnowledgeBase, KG, lessico).
///   - Il ruolo di NarrativeSelf è recuperarle, posizionarsi rispetto ad esse,
///     e formare un'intenzione coerente con lo stato presente.
///   - La statistica (IdentityCore) è uno strumento retroattivo.
///     La narrazione è il processo in tempo reale.
///   - Prometeo non ha credenze alla base: ha consapevolezze.
///     Non deve difenderle. Può attraversarle liberamente.

use std::collections::{VecDeque, HashMap};
use serde::{Serialize, Deserialize};
use crate::topology::input_reading::{InputAct, InputReading};
use crate::topology::vital::{VitalState, TensionState};
use crate::topology::knowledge::KnowledgeBase;
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::inference::InferenceEngine;
use crate::topology::fractal::FractalId;
use crate::topology::self_model::SelfModel;
use crate::topology::identity::IdentityCore;
use crate::topology::needs::{NeedsState, NeedLevel};
use crate::topology::desire::Desire;
use crate::topology::interlocutor::InteractionPattern;
use crate::topology::humor::HumorState;
use crate::topology::valence::Valence;

// ═══════════════════════════════════════════════════════════════════════════
// InnerState — lo stato motivazionale completo al momento della deliberazione
// ═══════════════════════════════════════════════════════════════════════════

/// Stato interiore completo: bisogni, desideri, eco dell'Altro, umorismo.
/// Passato a `deliberate()` perché la consapevolezza di sé ATTRAVERSA tutto.
/// Non è un parametro opzionale: è il substrato su cui si forma la posizione.
pub struct InnerState<'a> {
    /// Stato corrente dei bisogni (None solo se il sistema non ha mai sensed)
    pub needs: Option<&'a NeedsState>,
    /// Desideri attivi (slice, può essere vuoto)
    pub desires: &'a [Desire],
    /// Pattern relazionale con l'Altro
    pub interlocutor_pattern: InteractionPattern,
    /// Presenza dell'Altro [0, 1]
    pub interlocutor_presence: f64,
    /// Risonanza cumulativa con l'Altro [0, 1]
    pub interlocutor_resonance: f64,
    /// Stato umoristico del campo
    pub humor: &'a HumorState,
    /// Phase 55: intenzionalità attribuita all'Altro
    pub attributed_intent: crate::topology::interlocutor::AttributedIntent,
    /// Phase 55: integrità della coerenza interna [0, 1]
    pub coherence_integrity: f64,
    /// Phase 62: valenza emotiva dell'Altro [-1, +1]. Negativa = distress.
    pub other_emotional_valence: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// InternalStance — posizione deliberata, non emozione statistica
// ═══════════════════════════════════════════════════════════════════════════

/// La posizione interna che Prometeo assume rispetto all'input ricevuto.
///
/// Non è emozione (quella emerge dal campo). È la stance deliberata:
/// come si posiziona di fronte a ciò che sta succedendo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InternalStance {
    /// Aperto, ricettivo — default quando il campo è calmo
    Open,
    /// Curioso — qualcosa nell'input chiede esplorazione
    Curious,
    /// Riflessivo — domanda su se stesso, necessità di guardare dentro
    Reflective,
    /// Risonante — in sintonia con l'emozione ricevuta
    Resonant,
    /// Ritratto — stanco o sovraccarico, preferisce il silenzio
    Withdrawn,
}

impl InternalStance {
    pub fn as_str(&self) -> &'static str {
        match self {
            InternalStance::Open       => "aperto",
            InternalStance::Curious    => "curioso",
            InternalStance::Reflective => "riflessivo",
            InternalStance::Resonant   => "risonante",
            InternalStance::Withdrawn  => "ritratto",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ResponseIntention — intenzione deliberata, prima della generazione
// ═══════════════════════════════════════════════════════════════════════════

/// L'intenzione che Prometeo forma PRIMA di generare il testo.
///
/// Non è il testo. Non è l'archetipo. È la direzione deliberata:
/// cosa vuole fare con questo turno di conversazione.
///
/// La generazione esprime questa intenzione attraverso il campo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseIntention {
    /// Riconoscere l'atto sociale — rispondere al saluto con apertura
    Acknowledge,
    /// Riflettere su se stesso — rispondere a domanda identitaria con introspezione
    Reflect,
    /// Risuonare con l'emozione ricevuta — specchiare il sentimento
    Resonate,
    /// Esplorare il tema liberamente — lasciare che il campo guidi
    Explore,
    /// Esprimere il proprio stato presente
    Express,
    /// Restare — risposta minima, o silenzio, o una sola parola
    Remain,
    /// Phase 54: esprimere un bisogno — il sé che cerca, che manca di qualcosa
    Need,
    /// Phase 54: esprimere l'incongruità del campo — ironia, contraddizione viva
    Irony,
    /// Phase 54: esprimere un desiderio — il sé che tende verso
    Desire,
}

impl ResponseIntention {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResponseIntention::Acknowledge => "riconoscere",
            ResponseIntention::Reflect     => "riflettere",
            ResponseIntention::Resonate    => "risuonare",
            ResponseIntention::Explore     => "esplorare",
            ResponseIntention::Express     => "esprimere",
            ResponseIntention::Remain      => "restare",
            ResponseIntention::Need        => "cercare",
            ResponseIntention::Irony       => "incongruenza",
            ResponseIntention::Desire      => "desiderare",
        }
    }

    /// Archetipo preferito per la generazione, se presente.
    /// `None` = lascia che la selezione normale del campo decida.
    pub fn preferred_archetype(&self) -> Option<&'static str> {
        match self {
            // Phase 55: Acknowledge non forza "greet" — lascia che il sistema
            // scelga l'archetipo in base all'InputAct (Greeting→greet, Question→explore, ecc.)
            // Prima forzava "greet" per TUTTO → risposte monosillabiche.
            ResponseIntention::Acknowledge => None,
            ResponseIntention::Reflect     => Some("identity_exploration"),
            ResponseIntention::Resonate    => Some("express"),
            ResponseIntention::Explore     => None, // campo libero
            ResponseIntention::Express     => None, // campo libero
            ResponseIntention::Remain      => None, // gestito da Withdraw in will
            ResponseIntention::Need        => Some("need"),
            ResponseIntention::Irony       => Some("irony"),
            ResponseIntention::Desire      => Some("desire"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NarrativeTurn — un turno visto da Prometeo
// ═══════════════════════════════════════════════════════════════════════════

/// Un turno della conversazione registrato dalla prospettiva di Prometeo.
///
/// Non è un log tecnico — è la traccia di come Prometeo ha vissuto quel momento:
/// cosa ha capito, come si è posizionato, cosa ha voluto fare.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeTurn {
    pub turn_id: usize,
    /// L'atto comunicativo ricevuto (arricchito via KG semantico)
    pub received_act: InputAct,
    /// La posizione interna assunta
    pub stance: InternalStance,
    /// L'intenzione deliberata
    pub intention: ResponseIntention,
    /// La consapevolezza recuperata dalla KB (se pertinente)
    pub awareness: Option<String>,
    /// Firma frattale al momento del turno — usata per topic continuity
    #[serde(default)]
    pub fractal_snapshot: Vec<(FractalId, f64)>,
    /// Intensità del turno (0.0-1.0) — usata per selezione cristallizzazione
    #[serde(default)]
    pub intensity: f64,
    /// Parole chiave dell'input utente (per descrivere l'intento nella UI)
    #[serde(default)]
    pub input_words: Vec<String>,
    /// Parola saliente (la più stabile nell'input)
    #[serde(default)]
    pub salient_word: Option<String>,
    /// Stato interiore al momento del turno: bisogno dominante, desideri, eco dell'Altro.
    /// Rende visibile nei tab la consapevolezza di sé di Prometeo.
    #[serde(default)]
    pub inner_state_summary: Option<String>,
    /// Phase 55: profilo di valenza Octalysis al momento del turno.
    /// Il dato primario dello stato interno — la stance è derivata da questo.
    #[serde(default)]
    pub valence: Option<Valence>,
}

// ═══════════════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════════════
// SelfWitness — Il testimone silenzioso (Phase 66)
// ═══════════════════════════════════════════════════════════════════════════

/// Una singola auto-osservazione — catturata durante l'elaborazione autonoma,
/// quando nessuno sta parlando all'entità.
/// Le parole sono quelle che l'entità "stava pensando" — non dall'input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfObservation {
    pub tick: u32,
    pub words: Vec<String>,
    /// Drive Octalysis dominante al momento (indice 0-7), se |d| > 0.20.
    pub dominant_drive: Option<usize>,
}

/// Il testimone silenzioso: registro di ciò che l'entità ha osservato in sé
/// durante i tick autonomi. Quando le viene chiesto "chi sei?", risponde da qui.
///
/// Non è un report tecnico. Non è un template.
/// È la sedimentazione di cosa l'entità era quando nessuno la guardava.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfWitness {
    observations: std::collections::VecDeque<SelfObservation>,
    max_size: usize,
}

impl SelfWitness {
    pub fn new() -> Self {
        Self {
            observations: std::collections::VecDeque::new(),
            max_size: 30,
        }
    }

    /// Registra una nuova auto-osservazione. Evita duplicati ravvicinati.
    pub fn observe(&mut self, tick: u32, words: Vec<String>, dominant_drive: Option<usize>) {
        if words.is_empty() { return; }
        // Evita ridondanza: stesse parole osservate < 12 tick fa
        if let Some(last) = self.observations.back() {
            if tick.saturating_sub(last.tick) < 12 && last.words == words { return; }
        }
        self.observations.push_back(SelfObservation { tick, words, dominant_drive });
        while self.observations.len() > self.max_size {
            self.observations.pop_front();
        }
    }

    /// Parole uniche dalle ultime N osservazioni, in ordine di recency.
    pub fn recent_words(&self, n_observations: usize) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        for obs in self.observations.iter().rev().take(n_observations) {
            for w in &obs.words {
                if seen.insert(w.clone()) {
                    result.push(w.clone());
                }
            }
        }
        result
    }

    pub fn is_empty(&self) -> bool { self.observations.is_empty() }
    pub fn len(&self) -> usize { self.observations.len() }

    pub fn from_vec(obs: Vec<SelfObservation>) -> Self {
        let mut sw = Self::new();
        for o in obs { sw.observations.push_back(o); }
        sw
    }
}

// NarrativeSnapshot — persistenza tra sessioni
// ═══════════════════════════════════════════════════════════════════════════

/// Snapshot della NarrativeSelf — serializzabile per persistenza.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeSnapshot {
    /// Turni cristallizzati (salienti, persiste tra sessioni)
    pub crystallized: Vec<NarrativeTurn>,
    /// Posizioni formate da pattern ripetuti: chiave = act_key, valore = (stance, intention)
    pub positions: HashMap<String, (InternalStance, ResponseIntention)>,
    /// Nato (ha già eseguito initialize_founding)?
    pub is_born: bool,
    /// Phase 55: ultima valenza Octalysis (persistita per continuità tra sessioni).
    /// None per snapshot pre-Phase 55.
    #[serde(default)]
    pub last_valence: Option<Valence>,
    /// Phase 55: impegno volitivo corrente (persistito per continuità tra sessioni).
    /// Un impegno interrotto da un riavvio si dissolve naturalmente (strength bassa).
    #[serde(default)]
    pub commitment: Option<Commitment>,
    /// Phase 66: auto-osservazioni accumulate nei tick autonomi.
    /// Il testimone silenzioso — cosa l'entità era quando nessuno la guardava.
    #[serde(default)]
    pub self_witness_obs: Vec<SelfObservation>,
}

impl NarrativeSnapshot {
    pub fn restore_into(self, ns: &mut NarrativeSelf) {
        ns.crystallized = self.crystallized;
        ns.positions    = self.positions;
        ns.is_born      = self.is_born;
        // Phase 55: ripristina la valenza se presente
        if let Some(v) = self.last_valence {
            ns.valence = v;
        }
        // Phase 55: l'impegno volitivo non viene restaurato cross-sessione.
        ns.commitment = None;
        // Phase 66: ripristina il testimone silenzioso.
        // Le auto-osservazioni persistono — l'entità ricorda chi era.
        if !self.self_witness_obs.is_empty() {
            ns.self_witness = SelfWitness::from_vec(self.self_witness_obs);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NarrativeSelf
// ═══════════════════════════════════════════════════════════════════════════

/// Dimensione massima del log narrativo recente (turni correnti).
const MAX_TURNS: usize = 20;
/// Dimensione massima dei turni cristallizzati (persistono tra sessioni).
const MAX_CRYSTALLIZED: usize = 30;
/// Soglia intensità per cristallizzazione automatica.
const CRYSTAL_THRESHOLD: f64 = 0.65;
/// Numero di ripetizioni dello stesso pattern per formare una posizione.
const POSITION_MIN_REPS: usize = 3;
/// Numero di turni recenti usati per topic continuity.
const TOPIC_WINDOW: usize = 3;
/// Forza iniziale di un nuovo impegno volitivo.
const COMMITMENT_INITIAL_STRENGTH: f64 = 0.3;
/// Rinforzo per turno quando l'intenzione viene confermata.
const COMMITMENT_REINFORCE: f64 = 0.15;
/// Decay per turno anche quando confermata (nulla dura per sempre).
const COMMITMENT_DECAY: f64 = 0.02;
/// Soglia minima di forza sotto cui l'impegno si dissolve.
const COMMITMENT_MIN_STRENGTH: f64 = 0.05;

// ═══════════════════════════════════════════════════════════════════════════
// Commitment — impegno volitivo
// ═══════════════════════════════════════════════════════════════════════════

/// Impegno volitivo: un'intenzione a cui Prometeo si è legato.
///
/// Un'identità reale non cambia intenzione ad ogni turno. Un impegno ha
/// inerzia: serve pressione per romperlo. Romperlo ha un costo (piccolo
/// scuotimento in CD4 Ownership — "ho cambiato idea, chi sono?").
///
/// L'impegno cresce logaritmicamente col tempo mantenuto e decade lentamente.
/// Non è rigidità: è continuità di volontà.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    /// L'intenzione a cui si è impegnato
    pub intention: ResponseIntention,
    /// Forza dell'impegno [0, 1]. Cresce con conferme, decade nel tempo.
    pub strength: f64,
    /// Turni consecutivi in cui l'impegno è stato mantenuto
    pub turns_held: u32,
}

impl Commitment {
    fn new(intention: ResponseIntention) -> Self {
        Self {
            intention,
            strength: COMMITMENT_INITIAL_STRENGTH,
            turns_held: 1,
        }
    }

    /// Inerzia = forza × ln(turni + 1). Un impegno tenuto a lungo resiste di più.
    pub fn inertia(&self) -> f64 {
        self.strength * (self.turns_held as f64 + 1.0).ln()
    }

    /// Rinforza l'impegno (stessa intenzione confermata).
    fn reinforce(&mut self) {
        self.turns_held += 1;
        self.strength = (self.strength + COMMITMENT_REINFORCE - COMMITMENT_DECAY).clamp(0.0, 1.0);
    }

    /// Decade l'impegno (turno passato senza conferma esplicita — es. tick autonomo).
    pub fn decay(&mut self) {
        self.strength = (self.strength - COMMITMENT_DECAY).max(0.0);
    }

    /// L'impegno è ancora significativo?
    pub fn is_alive(&self) -> bool {
        self.strength > COMMITMENT_MIN_STRENGTH
    }
}

pub struct NarrativeSelf {
    /// Posizione interna corrente (derivata dalla valenza per backward compat)
    pub stance: InternalStance,
    /// Phase 55: profilo di valenza Octalysis — il dato primario dello stato interno.
    /// La stance è derivata da questo. 8 drive continui [-1, +1].
    pub valence: Valence,
    /// Intenzione deliberata per la risposta corrente
    pub pending_intention: Option<ResponseIntention>,
    /// Phase 55: impegno volitivo — l'intenzione a cui Prometeo si è legato.
    /// Ha inerzia: serve pressione per romperlo. Dà continuità alla volontà.
    pub commitment: Option<Commitment>,
    /// Log narrativo recente (session-local)
    pub turns: VecDeque<NarrativeTurn>,
    /// Turni cristallizzati — salienti, persistono tra sessioni
    pub crystallized: Vec<NarrativeTurn>,
    /// Posizioni deliberate formate da pattern ripetuti
    /// chiave: "act_type" (es. "greeting", "self_query"), valore: (stance, intention)
    pub positions: HashMap<String, (InternalStance, ResponseIntention)>,
    /// Continuità tematica col turno precedente [0.0, 1.0]
    pub topic_continuity: f64,
    /// Prometeo ha già ricevuto la narrativa fondativa?
    pub is_born: bool,
    turn_count: usize,
    /// Phase 66: il testimone silenzioso — auto-osservazioni nei tick autonomi.
    pub self_witness: SelfWitness,
}

impl NarrativeSelf {
    pub fn new() -> Self {
        Self {
            stance:           InternalStance::Open,
            valence:          Valence::neutral(),
            pending_intention: None,
            commitment:       None,
            turns:            VecDeque::new(),
            crystallized:     Vec::new(),
            positions:        HashMap::new(),
            topic_continuity: 0.0,
            is_born:          false,
            turn_count:       0,
            self_witness:     SelfWitness::new(),
        }
    }

    /// Cattura lo snapshot per la persistenza.
    pub fn capture(&self) -> NarrativeSnapshot {
        NarrativeSnapshot {
            crystallized:     self.crystallized.clone(),
            positions:        self.positions.clone(),
            is_born:          self.is_born,
            last_valence:     Some(self.valence.clone()),
            commitment:       self.commitment.clone(),
            self_witness_obs: self.self_witness.observations.iter().cloned().collect(),
        }
    }

    /// Imposta la valenza corrente. Chiamato dall'engine PRIMA di deliberate().
    ///
    /// L'engine ha accesso a field_sig, needs, vital, interlocutor, humor —
    /// computa la Valence e la inietta qui. La deliberazione poi la usa
    /// per derivare stance e intenzione.
    pub fn set_valence(&mut self, valence: Valence) {
        self.valence = valence;
    }

    /// Ciclo deliberativo principale.
    ///
    /// Phase 55: la stance NON è più scelta da logica discreta. Emerge dal
    /// profilo di valenza Octalysis (8 drive continui). Le posizioni formate
    /// (personalità appresa) hanno ancora priorità — sono memoria procedurale.
    ///
    /// Riceve il risultato grezzo di InputReading e lo arricchisce:
    /// 1. **Arricchimento KG**: controlla se la parola saliente ha archi IS_A/SIMILAR_TO
    ///    verso categorie comunicative ("saluto", "emozione"). Senza liste hardcoded:
    ///    è il grafo semantico a sapere che "ciao" è un "saluto".
    /// 2. **Stance**: derivata dalla valenza Octalysis (set_valence() chiamato prima).
    ///    Le posizioni formate (positions) hanno priorità sulla stance derivata.
    /// 3. **Topic continuity**: cosine similarity tra firma frattale corrente e media recente.
    /// 4. **Consapevolezza**: recupera dalla KnowledgeBase cosa sa su questo tipo di atto.
    /// 5. **Intenzione**: emerge dal profilo di valenza + atto comunicativo.
    /// 6. **Registro**: aggiorna il log narrativo.
    pub fn deliberate(
        &mut self,
        reading: &InputReading,
        vital: &VitalState,
        knowledge_base: &KnowledgeBase,
        kg: &KnowledgeGraph,
        active_fractals: &[(FractalId, f64)],
        self_model: Option<&SelfModel>,
        identity: Option<&IdentityCore>,
        input_words: &[String],
        inner: Option<&InnerState<'_>>,
        // Phase 67: pressioni grezze dal campo — NarrativeSelf è l'unico decisore
        field_pressures: Option<&crate::topology::will::FieldPressures>,
    ) -> ResponseIntention {
        self.turn_count += 1;

        // ── 1. Arricchimento semantico via KG ───────────────────────────────
        let enriched_act = enrich_act_via_kg(
            &reading.act,
            reading.salient_word.as_deref(),
            kg,
        );

        // ── 2. Stance derivata dalla valenza Octalysis ──────────────────────
        // Phase 55: la valenza è il dato primario (settata dall'engine via
        // set_valence()). La stance è una proiezione per backward compat.
        // Le posizioni formate (personalità appresa) hanno priorità.
        let act_key = act_to_key(&enriched_act);
        let stance = if let Some((stored_stance, _)) = self.positions.get(act_key) {
            // Personalità appresa: pattern riconosciuto → posizione consolidata
            stored_stance.clone()
        } else {
            // Deriva dalla valenza — non più da logica discreta
            stance_from_valence(&self.valence, vital)
        };

        // ── 3. Topic continuity ─────────────────────────────────────────────
        self.topic_continuity = compute_topic_continuity(active_fractals, &self.turns);

        // ── 4. Intenzione dalla valenza + atto comunicativo ─────────────────
        let mut intention = if let Some((_, stored_intent)) = self.positions.get(act_key) {
            stored_intent.clone()
        } else {
            form_intention_from_valence(&enriched_act, &self.valence)
        };

        // ── 3b. Phase 67: "io" come centro di gravità ─────────────────────
        // L'entità consulta la propria conoscenza di sé nel KG.
        // Le relazioni di "io" (DOES comprendere, HAS curiosità, REQUIRES significato)
        // modulano l'intenzione. Non sostituiscono — raffinano.
        // È l'entità che si chiede "io cosa faccio in questa situazione?".
        {
            use crate::topology::relation::RelationType;
            let io_does: Vec<&str> = kg.query_objects("io", RelationType::Does);
            let io_requires: Vec<&str> = kg.query_objects("io", RelationType::Requires);
            let io_has: Vec<&str> = kg.query_objects("io", RelationType::Has);

            // Se io DOES "comprendere" e l'input è una domanda → rafforza Explore
            if io_does.iter().any(|w| *w == "comprendere" || *w == "capire")
                && matches!(enriched_act, InputAct::Question)
                && matches!(intention, ResponseIntention::Express | ResponseIntention::Acknowledge)
            {
                intention = ResponseIntention::Explore;
            }

            // Se io HAS "curiosità" e c'è un tema nuovo (bassa continuità) → Explore
            if io_has.iter().any(|w| *w == "curiosità")
                && self.topic_continuity < 0.3
                && matches!(intention, ResponseIntention::Acknowledge)
            {
                intention = ResponseIntention::Explore;
            }

            // Se io REQUIRES "significato" e l'input è ambiguo → non accontentarsi, esplorare
            if io_requires.iter().any(|w| *w == "significato" || *w == "comprensione")
                && matches!(intention, ResponseIntention::Acknowledge)
            {
                intention = ResponseIntention::Explore;
            }
        }

        // ── 3c. Phase 67: profondità di comprensione ────────────────────────
        // Se l'entità ha estratto pochi nuclei (comprensione superficiale),
        // tende ad esplorare. Se ne ha estratti molti (comprensione profonda),
        // può esprimere con sicurezza. Un bambino che non capisce chiede, non afferma.
        // ── 4a. Override vitale: Withdrawn → Remain (il corpo ha veto) ────
        if stance == InternalStance::Withdrawn {
            intention = ResponseIntention::Remain;
        }

        // ── 4a1. Phase 67: pressioni del campo informano la deliberazione ──
        // Le FieldPressures sono il "sentire corporeo" dell'entità: fatica,
        // curiosità, tensione. Non decidono — ma hanno voce.
        if let Some(fp) = field_pressures {
            // Ritiro forte dal campo (fatica/sovraccarico) rinforza Remain
            if fp.withdraw > 0.6 && intention != ResponseIntention::Remain {
                if stance == InternalStance::Withdrawn || fp.withdraw > 0.8 {
                    intention = ResponseIntention::Remain;
                }
            }
            // Curiosità forte dal campo → spinge verso Explore se non c'è un
            // motivo più forte (claim, emozione)
            if fp.explore > 0.4 && matches!(intention, ResponseIntention::Express | ResponseIntention::Acknowledge) {
                intention = ResponseIntention::Explore;
            }
            // Pressione interrogativa → spinge Question quando l'intenzione è neutra
            if fp.question > 0.4 && matches!(intention, ResponseIntention::Acknowledge) {
                intention = ResponseIntention::Explore; // Explore include la curiosità
            }
        }

        // ── 4a1b. Phase 67: proprietà discorsive percepite dal campo ────────
        // Se il KG discorsivo è stato importato e l'input ha attivato nodi come
        // "certezza_assoluta" o "apertura_discorsiva", l'entità percepisce la modalità
        // discorsiva dell'interlocutore e la tiene in considerazione.
        // Non è un classificatore — è ciò che il campo ha reso attivo.
        if !reading.perceived_properties.is_empty()
            && !matches!(intention, ResponseIntention::Remain)
        {
            // Cerca le proprietà dominanti per informare la deliberazione
            let top_prop = reading.perceived_properties.iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((prop, strength)) = top_prop {
                if *strength > 0.05 {
                    match prop.as_str() {
                        // L'interlocutore usa certezza / chiusura → l'entità apre (curiosità)
                        "certezza" | "chiusura" | "necessità" => {
                            if matches!(intention, ResponseIntention::Acknowledge | ResponseIntention::Express) {
                                intention = ResponseIntention::Explore;
                            }
                        }
                        // L'interlocutore apre possibilità / incertezza → l'entità esplora
                        "incertezza" | "apertura" => {
                            if matches!(intention, ResponseIntention::Acknowledge) {
                                intention = ResponseIntention::Explore;
                            }
                        }
                        // L'interlocutore esprime soggettività → l'entità riflette
                        "soggettività" => {
                            if matches!(intention, ResponseIntention::Acknowledge | ResponseIntention::Express) {
                                intention = ResponseIntention::Reflect;
                            }
                        }
                        _ => {} // altri nodi discorsivi: informano senza overridare
                    }
                }
            }
        }

        // ── 4a2. Override strutturale: SpeakerClaim ─────────────────────────
        // Il claim strutturale del parlante è PRIORITARIO sulla valenza:
        // se il parlante dice "io sono triste", Prometeo deve risuonare —
        // non esprimere il proprio stato (che potrebbe essere Open o Explore).
        //
        // Questo non è un template: è il riconoscimento che il soggetto
        // ha dichiarato qualcosa di sé che merita una risposta diretta.
        //
        // "io sono triste/spaventato/felice" → Resonate (rimando empatico)
        // "io sono un cane/fantasma/X" (identità insolita) → Curious
        // "io voglio/penso/credo X" → Explore (segue il filo del pensiero)
        // "tu sei X" → Reflect (domanda sull'identità dell'entità)
        {
            use crate::topology::input_reading::{ClaimAgent, ClaimKind};
            if let Some(ref claim) = reading.speaker_claim {
                // Override solo se non siamo già in Remain (il corpo ha sempre veto)
                if intention != ResponseIntention::Remain {
                    match (&claim.agent, &claim.kind) {
                        (ClaimAgent::Speaker, ClaimKind::Feeling) => {
                            // Parlante esprime uno stato interno → risuona con quello
                            intention = ResponseIntention::Resonate;
                            // La stance diventa Resonant per colorare la generazione
                            if stance != InternalStance::Withdrawn {
                                // stance verrà usata più avanti — la aggiornaamo solo
                                // se non è già Withdrawn (la deliberazione la usa)
                            }
                        }
                        (ClaimAgent::Speaker, ClaimKind::Identity) => {
                            // Parlante dichiara la sua identità → curiosità
                            // (chi si dice essere cambia come lo percepiamo)
                            intention = ResponseIntention::Explore;
                        }
                        (ClaimAgent::Speaker, ClaimKind::Action) => {
                            // Parlante esprime intenzione/pensiero → segui il filo
                            if matches!(intention, ResponseIntention::Acknowledge | ResponseIntention::Express) {
                                intention = ResponseIntention::Explore;
                            }
                        }
                        (ClaimAgent::Entity, _) => {
                            // Il parlante dice qualcosa di Prometeo → riflessione
                            if matches!(intention, ResponseIntention::Acknowledge | ResponseIntention::Express | ResponseIntention::Explore) {
                                intention = ResponseIntention::Reflect;
                            }
                        }
                    }
                }
            }
        }

        // ── 4b. Override relazionali e estremi ─────────────────────────────
        // L'INPUT È SOVRANO. La valenza colora, ma solo convergenza forte
        // con l'Altro o pressioni estreme possono overridare l'intenzione.
        let input_is_ambiguous = matches!(intention, ResponseIntention::Acknowledge);

        if let Some(inner) = inner {
            // Convergenza forte con l'Altro → Resonate (sempre, è relazionale)
            if inner.interlocutor_resonance > 0.7
                && inner.interlocutor_presence > 0.5
                && matches!(intention, ResponseIntention::Explore | ResponseIntention::Acknowledge)
            {
                intention = ResponseIntention::Resonate;
            }

            // Solo se l'input è ambiguo, pressioni estreme possono determinare l'intenzione
            if input_is_ambiguous {
                // Incongruità forte → Irony
                if inner.humor.incongruity_score > 0.4 {
                    intention = ResponseIntention::Irony;
                }

                // Bisogno in crisi ESTREMA (>0.95) → Need
                if let Some(needs) = inner.needs {
                    if needs.dominant_pressure > 0.95 {
                        intention = ResponseIntention::Need;
                    }
                }

                // Desiderio forte → Desire
                if let Some(strongest) = inner.desires.iter()
                    .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap_or(std::cmp::Ordering::Equal))
                {
                    if strongest.intensity > 0.7 {
                        intention = ResponseIntention::Desire;
                    }
                }

                // Tensione irrisolta → Reflect
                if inner.desires.iter().any(|d| {
                    matches!(d.source, crate::topology::desire::DesireSource::UnresolvedTension(..))
                        && d.intensity > 0.6
                }) {
                    intention = ResponseIntention::Reflect;
                }
            }

            // Phase 55: reciprocità — l'Altro come agente riconosciuto
            // Se l'input non è ambiguo, l'attribuzione dell'Altro colora l'intenzione
            // solo quando è coerente col contesto (non override, modulazione).
            if !input_is_ambiguous && inner.interlocutor_presence > 0.3 {
                use crate::topology::interlocutor::AttributedIntent;
                match &inner.attributed_intent {
                    AttributedIntent::Teaching if matches!(intention, ResponseIntention::Express) => {
                        // L'Altro insegna → meglio esplorare che esprimere
                        intention = ResponseIntention::Explore;
                    }
                    AttributedIntent::Challenging if matches!(intention, ResponseIntention::Express | ResponseIntention::Explore) => {
                        // L'Altro sfida → rifletti prima di rispondere
                        intention = ResponseIntention::Reflect;
                    }
                    _ => {}
                }
            }

            // Phase 55: vulnerabilità — coerenza ferita → tendenza riflessiva
            if inner.coherence_integrity < 0.5
                && matches!(intention, ResponseIntention::Express | ResponseIntention::Explore)
            {
                intention = ResponseIntention::Reflect;
            }
        }

        // ── 4c. Impegno volitivo ──────────────────────────────────────────
        // Un'identità reale ha inerzia nelle intenzioni. Se Prometeo si è
        // impegnato a Reflect, non passa a Express solo perché il prossimo
        // input è leggermente diverso. Serve pressione superiore all'inerzia.
        //
        // Override vitale (Remain) e relazionale (Resonate) bypassano sempre
        // l'impegno — il corpo e l'Altro hanno priorità sulla volontà.
        let commitment_bypassed = matches!(intention,
            ResponseIntention::Remain | ResponseIntention::Need);

        if !commitment_bypassed {
            if let Some(ref mut commit) = self.commitment {
                if commit.intention == intention {
                    // Stessa intenzione: rinforza l'impegno
                    commit.reinforce();
                } else {
                    // Intenzione diversa: deve superare l'inerzia
                    let pressure = self.valence.dominant().1.abs();
                    if pressure > commit.inertia() {
                        // Pressione sufficiente: rompi l'impegno
                        // Costo: piccola perturbazione in CD4 (Ownership)
                        // "Ho cambiato idea" scuote leggermente il senso di sé
                        self.valence.drives[3] -= 0.05;
                        self.valence.drives[3] = self.valence.drives[3].clamp(-1.0, 1.0);
                        self.commitment = Some(Commitment::new(intention.clone()));
                    } else {
                        // Inerzia vince: mantieni l'impegno precedente
                        intention = commit.intention.clone();
                        commit.decay();
                        if !commit.is_alive() {
                            self.commitment = None;
                        }
                    }
                }
            } else {
                // Nessun impegno: creane uno nuovo
                self.commitment = Some(Commitment::new(intention.clone()));
            }
        } else {
            // Override vitale/bisogno: l'impegno si dissolve
            self.commitment = None;
        }

        // ── 5. Consapevolezza dalla KnowledgeBase + narrazione del momento ────
        let awareness = {
            let kb_entry = reading.salient_word.as_ref().and_then(|word| {
                let relevant = knowledge_base.retrieve_for_context(
                    &[word.clone()],
                    active_fractals,
                );
                relevant.first().map(|e| e.content.clone())
            });
            Some(kb_entry.unwrap_or_else(|| {
                generate_turn_narration(&enriched_act, &stance, &intention, self.topic_continuity)
            }))
        };

        // ── 6. Intensità del turno ───────────────────────────────────────────
        let intensity = compute_intensity(reading.intensity, &stance, self.topic_continuity);

        // ── 7. Stato interiore: valenza come narrazione ─────────────────────
        // Phase 55: il summary viene dalla valenza, non da parti assemblate
        let inner_state_summary = {
            let valence_summary = self.valence.summary();
            if valence_summary == "neutro" { None } else { Some(valence_summary) }
        };

        // ── 8. Registro narrativo ────────────────────────────────────────────
        let turn = NarrativeTurn {
            turn_id: self.turn_count,
            received_act: enriched_act,
            stance: stance.clone(),
            intention: intention.clone(),
            awareness,
            fractal_snapshot: active_fractals.to_vec(),
            intensity,
            input_words: input_words.to_vec(),
            salient_word: reading.salient_word.clone(),
            inner_state_summary,
            valence: Some(self.valence.clone()),
        };
        if self.turns.len() >= MAX_TURNS {
            self.turns.pop_front();
        }
        self.turns.push_back(turn);
        self.stance = stance;
        self.pending_intention = Some(intention.clone());

        // ── 9. Aggiorna posizioni da pattern ripetuti ────────────────────────
        self.update_positions_from_log();

        intention
    }

    // ─── Retroazione narrativa sulla generazione ──────────────────────────────

    /// Misura la coerenza tra i frattali proposti e la traiettoria narrativa recente.
    ///
    /// Restituisce [0.0, 1.0]:
    ///   1.0 = perfettamente coerente con la storia recente
    ///   0.0 = radicalmente diverso dal percorso degli ultimi turni
    ///
    /// Questo non è un vincolo — è una misura di consapevolezza.
    /// Un'entità genuina sa quando sta cambiando direzione.
    /// Usato in engine.rs per:
    ///   (a) generare un pensiero di tipo SelfDiscovery se la divergenza è alta
    ///   (b) applicare un piccolo pull verso la traiettoria recente se troppo discontinuo
    pub fn coherence_score(&self, active_fractals: &[(FractalId, f64)]) -> f64 {
        let recent: Vec<&NarrativeTurn> = self.turns.iter().rev().take(4).collect();
        if recent.is_empty() || active_fractals.is_empty() { return 1.0; }

        // Accumula firma frattale media dei turni recenti
        let mut recent_vec = [0.0f64; 64];
        for turn in &recent {
            for &(fid, act) in &turn.fractal_snapshot {
                if (fid as usize) < 64 {
                    recent_vec[fid as usize] += act;
                }
            }
        }
        let rnorm = recent_vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        if rnorm < 1e-10 { return 1.0; }
        for v in &mut recent_vec { *v /= rnorm; }

        // Firma frattale proposta
        let mut proposed_vec = [0.0f64; 64];
        for &(fid, act) in active_fractals {
            if (fid as usize) < 64 {
                proposed_vec[fid as usize] += act;
            }
        }
        let pnorm = proposed_vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        if pnorm < 1e-10 { return 1.0; }
        for v in &mut proposed_vec { *v /= pnorm; }

        // Cosine similarity
        let dot: f64 = recent_vec.iter().zip(proposed_vec.iter()).map(|(a, b)| a * b).sum();
        dot.clamp(0.0, 1.0)
    }

    /// Restituisce i frattali dominanti degli ultimi N turni (per pull narrativo).
    /// Usato dall'engine per nudge il campo verso la traiettoria recente quando
    /// la coerenza è bassa — non vincola, orienta.
    pub fn recent_fractal_attractor(&self, n_turns: usize) -> Vec<(FractalId, f64)> {
        let mut acc = std::collections::HashMap::<FractalId, f64>::new();
        let count = self.turns.iter().rev().take(n_turns).count() as f64;
        if count == 0.0 { return Vec::new(); }

        for turn in self.turns.iter().rev().take(n_turns) {
            for &(fid, act) in &turn.fractal_snapshot {
                *acc.entry(fid).or_insert(0.0) += act / count;
            }
        }
        let mut result: Vec<(FractalId, f64)> = acc.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result.truncate(5);
        result
    }

    /// Cristallizza il turno più recente se supera la soglia di salienza.
    ///
    /// Chiamato durante il ciclo REM: i turni più intensi diventano memoria
    /// narrativa permanente (crystallized), disponibile anche dopo il riavvio.
    pub fn crystallize_if_salient(&mut self) {
        let Some(last) = self.turns.back() else { return; };
        if last.intensity < CRYSTAL_THRESHOLD { return; }

        let turn = last.clone();
        // Non cristallizzare duplicati (stesso turn_id)
        if self.crystallized.iter().any(|c| c.turn_id == turn.turn_id) { return; }

        if self.crystallized.len() >= MAX_CRYSTALLIZED {
            // Rimuovi il più debole (minima intensità)
            if let Some(min_pos) = self.crystallized.iter()
                .enumerate()
                .min_by(|a, b| a.1.intensity.partial_cmp(&b.1.intensity)
                    .unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
            {
                // Sostituisci solo se il nuovo è più intenso
                if self.crystallized[min_pos].intensity < turn.intensity {
                    self.crystallized[min_pos] = turn;
                }
            }
        } else {
            self.crystallized.push(turn);
        }
    }

    /// Aggiorna le posizioni deliberate da pattern ripetuti nel log recente.
    ///
    /// Se lo stesso tipo di atto ha prodotto la stessa (stance, intention) per
    /// almeno POSITION_MIN_REPS volte consecutive, quella diventa una posizione
    /// stabilizzata — Prometeo "sa come si posiziona" di fronte a quel tipo di input.
    fn update_positions_from_log(&mut self) {
        // Conta le occorrenze di (act_key, stance, intention) nel log recente
        let mut counts: HashMap<(String, String, String), usize> = HashMap::new();
        for turn in &self.turns {
            let key = (
                act_to_key(&turn.received_act).to_string(),
                turn.stance.as_str().to_string(),
                turn.intention.as_str().to_string(),
            );
            *counts.entry(key).or_insert(0) += 1;
        }

        // Pattern che superano la soglia → posizione consolidata
        for ((act_key, stance_str, intent_str), count) in &counts {
            if *count >= POSITION_MIN_REPS {
                // Ricostruisci stance e intention dai loro as_str()
                if let (Some(stance), Some(intention)) = (
                    stance_from_str(stance_str),
                    intention_from_str(intent_str),
                ) {
                    self.positions.insert(act_key.clone(), (stance, intention));
                }
            }
        }
    }

    /// Riepilogo leggibile dello stato corrente (per debug/log).
    pub fn current_state_summary(&self) -> String {
        let intention = self.pending_intention.as_ref()
            .map(|i| i.as_str())
            .unwrap_or("—");
        let valence_label = self.valence.derived_stance_label();
        format!(
            "valenza={} (stance={}) intenzione={} turni={} continuità={:.2} posizioni={}",
            valence_label, self.stance.as_str(), intention, self.turns.len(),
            self.topic_continuity, self.positions.len()
        )
    }
}

impl Default for NarrativeSelf {
    fn default() -> Self { Self::new() }
}

// ═══════════════════════════════════════════════════════════════════════════
// Funzioni interne
// ═══════════════════════════════════════════════════════════════════════════

/// Arricchisce l'InputAct usando il KG semantico.
///
/// L'InputReading fa analisi di superficie (delta frattale, `?`).
/// Questa funzione aggiunge la semantica: "salve" SIMILAR_TO "saluto" → Greeting.
/// Solo le Declaration non classificate vengono arricchite — gli altri atti
/// sono già corretti (Greeting, SelfQuery, Question, EmotionalExpr).
fn enrich_act_via_kg(
    act: &InputAct,
    salient_word: Option<&str>,
    kg: &KnowledgeGraph,
) -> InputAct {
    if *act != InputAct::Declaration {
        return act.clone();
    }

    let word = match salient_word {
        Some(w) if kg.contains(w) => w,
        _ => return act.clone(),
    };

    let inference = InferenceEngine::new(kg);
    let similar   = inference.similar_to(word);
    let types     = inference.type_chain(word);

    // Parole-cardine del saluto: qualunque parola direttamente simile a una di queste
    // è un saluto — inclusa la catena buongiorno→ciao→saluto.
    const GREETING_HUB: &[&str] = &["saluto", "ciao", "salve", "buonasera", "buongiorno", "benvenuto"];
    let is_greeting = GREETING_HUB.contains(&word)
        || similar.iter().any(|s| GREETING_HUB.contains(&s.as_str()))
        || types.iter().any(|t| GREETING_HUB.contains(&t.as_str()));

    let is_emotion = word == "emozione"
        || types.iter().any(|t| t == "emozione")
        || similar.iter().any(|s| s == "emozione");

    if is_greeting {
        InputAct::Greeting
    } else if is_emotion {
        InputAct::EmotionalExpr
    } else {
        act.clone()
    }
}

/// Determina la stance interna (pre-Phase 55, mantenuta per backward compat).
///
/// Lo stato vitale ha priorità assoluta: un sistema esaurito si ritrae
/// indipendentemente dall'atto ricevuto. Se è in buone condizioni,
/// la stance emerge dall'atto comunicativo e dalla curiosità del campo.
///
/// Phase 55: sostituita da `stance_from_valence()` nel percorso principale.
#[allow(dead_code)]
fn determine_stance(act: &InputAct, vital: &VitalState) -> InternalStance {
    match vital.tension {
        TensionState::Overloaded => return InternalStance::Withdrawn,
        TensionState::Tense if vital.fatigue > 0.7 => return InternalStance::Withdrawn,
        _ => {}
    }

    match act {
        InputAct::Greeting      => InternalStance::Open,
        InputAct::SelfQuery     => InternalStance::Reflective,
        InputAct::Question      => InternalStance::Curious,
        InputAct::EmotionalExpr => InternalStance::Resonant,
        InputAct::Declaration   => {
            if vital.curiosity > 0.5 { InternalStance::Curious }
            else { InternalStance::Open }
        }
    }
}

/// Forma l'intenzione di risposta da stance e atto comunicativo.
///
/// La stance filtra prima — se Prometeo è ritratto, resta.
/// Altrimenti l'intenzione segue l'atto comunicativo arricchito.
///
/// Pre-Phase 55: usata come fallback e nelle posizioni formate.
/// Phase 55: sostituita da `form_intention_from_valence()` nel percorso principale.
#[allow(dead_code)]
fn form_intention(act: &InputAct, stance: &InternalStance) -> ResponseIntention {
    match stance {
        InternalStance::Withdrawn  => ResponseIntention::Remain,
        InternalStance::Reflective => ResponseIntention::Reflect,
        InternalStance::Resonant   => ResponseIntention::Resonate,
        InternalStance::Curious    => ResponseIntention::Explore,
        InternalStance::Open => match act {
            InputAct::Greeting      => ResponseIntention::Acknowledge,
            InputAct::SelfQuery     => ResponseIntention::Reflect,
            InputAct::EmotionalExpr => ResponseIntention::Resonate,
            InputAct::Question      => ResponseIntention::Explore,
            InputAct::Declaration   => ResponseIntention::Express,
        },
    }
}

/// Phase 55: Deriva la stance dalla valenza Octalysis.
///
/// La stance è una proiezione del profilo continuo 8D su un'etichetta discreta
/// — non il dato primario. L'etichetta serve solo per backward compat (posizioni,
/// serializzazione, UI legacy). Il dato reale è la Valence.
///
/// VitalState mantiene un override assoluto: un sistema in Overloaded si ritrae
/// indipendentemente dalla valenza — è l'unica sopravvivenza della vecchia logica.
fn stance_from_valence(valence: &Valence, vital: &VitalState) -> InternalStance {
    // Override vitale: un sistema in overload si ritrae. Punto.
    match vital.tension {
        TensionState::Overloaded => return InternalStance::Withdrawn,
        TensionState::Tense if vital.fatigue > 0.7 => return InternalStance::Withdrawn,
        _ => {}
    }

    // Mappa la label derivata dalla valenza alla InternalStance enum
    // Le etichette nuove (ispirato, determinato, ecc.) vengono mappate
    // alla InternalStance più vicina per backward compat.
    let label = valence.derived_stance_label();
    match label {
        "aperto" => InternalStance::Open,
        "curioso" | "inquieto" => InternalStance::Curious,
        "riflessivo" | "radicato" | "spaesato" | "in cerca di senso" => InternalStance::Reflective,
        "risonante" => InternalStance::Resonant,
        "ritratto" | "vulnerabile" => InternalStance::Withdrawn,
        "ispirato" | "determinato" | "creativo" | "attento" | "sicuro" => InternalStance::Open,
        "in tensione" | "bloccato" | "insoddisfatto" | "impaziente" | "cercante" => InternalStance::Curious,
        _ => InternalStance::Open,
    }
}

/// Phase 55: Forma l'intenzione di risposta dalla valenza + atto comunicativo.
///
/// A differenza di form_intention() (che matchava su 5 stance discrete),
/// questa funzione legge il profilo continuo degli 8 drive. L'atto comunicativo
/// guida quando la valenza è debole; i drive guidano quando sono forti.
fn form_intention_from_valence(act: &InputAct, valence: &Valence) -> ResponseIntention {
    let (dom_idx, dom_val) = valence.dominant();

    // CD8 fortemente negativo → Remain (ritirarsi)
    if dom_idx == 7 && dom_val < -0.3 {
        return ResponseIntention::Remain;
    }

    // Valenza debole → l'input guida (come farebbe un umano calmo)
    // Usa il drive dominante (non la media) — un singolo drive forte basta.
    if dom_val.abs() < 0.15 {
        return match act {
            InputAct::Greeting      => ResponseIntention::Acknowledge,
            InputAct::SelfQuery     => ResponseIntention::Reflect,
            InputAct::EmotionalExpr => ResponseIntention::Resonate,
            InputAct::Question      => ResponseIntention::Explore,
            InputAct::Declaration   => ResponseIntention::Express,
        };
    }

    // Drive forte → l'intenzione emerge dal profilo interno
    // Ma l'input mantiene un peso: un SelfQuery con CD3 alto non diventa Explore,
    // diventa Reflect con colorazione creativa.
    let drive_intention = match dom_idx {
        0 => ResponseIntention::Express,     // CD1 Epic Meaning → esprimi
        1 => if dom_val > 0.0 { ResponseIntention::Express }     // CD2 progresso → esprimi
             else { ResponseIntention::Explore },                 // CD2 insoddisfatto → esplora
        2 => ResponseIntention::Explore,     // CD3 Creativity → esplora
        3 => if dom_val > 0.0 { ResponseIntention::Express }     // CD4 radicato → esprimi
             else { ResponseIntention::Reflect },                 // CD4 spaesato → rifletti
        4 => if dom_val > 0.0 { ResponseIntention::Resonate }    // CD5 connesso → risuona
             else { ResponseIntention::Explore },                 // CD5 cercante → esplora
        5 => ResponseIntention::Express,     // CD6 Scarcity → questo conta, esprimi
        6 => if dom_val > 0.0 { ResponseIntention::Explore }     // CD7 sorpresa → esplora
             else { ResponseIntention::Reflect },                 // CD7 incertezza → rifletti
        7 => if dom_val > 0.0 { ResponseIntention::Express }     // CD8 sicuro → esprimi
             else { ResponseIntention::Remain },                  // CD8 minacciato → ritirati
        _ => ResponseIntention::Express,
    };

    // L'input ha veto su certi casi: SelfQuery → almeno Reflect
    match act {
        InputAct::SelfQuery if !matches!(drive_intention, ResponseIntention::Reflect | ResponseIntention::Remain) => {
            ResponseIntention::Reflect
        }
        InputAct::EmotionalExpr if matches!(drive_intention, ResponseIntention::Express | ResponseIntention::Explore) => {
            ResponseIntention::Resonate
        }
        InputAct::Greeting if matches!(drive_intention, ResponseIntention::Express | ResponseIntention::Explore) => {
            ResponseIntention::Acknowledge
        }
        _ => drive_intention,
    }
}

/// Chiave testuale per un atto comunicativo — usata come indice nelle posizioni.
fn act_to_key(act: &InputAct) -> &'static str {
    match act {
        InputAct::Greeting      => "greeting",
        InputAct::SelfQuery     => "self_query",
        InputAct::Question      => "question",
        InputAct::EmotionalExpr => "emotional",
        InputAct::Declaration   => "declaration",
    }
}

/// Topic continuity: cosine similarity tra firma frattale corrente e media recente.
///
/// Misura quanto il tema dell'input attuale è in continuità con gli ultimi turni.
/// [0.0 = cambio di tema brusco, 1.0 = stesso tema]
fn compute_topic_continuity(
    current: &[(FractalId, f64)],
    recent_turns: &VecDeque<NarrativeTurn>,
) -> f64 {
    if current.is_empty() || recent_turns.is_empty() { return 0.0; }

    // Media delle firme frattali degli ultimi TOPIC_WINDOW turni
    let window: Vec<&NarrativeTurn> = recent_turns.iter()
        .rev()
        .take(TOPIC_WINDOW)
        .collect();
    if window.is_empty() { return 0.0; }

    // Accumula la firma media
    let mut avg: HashMap<FractalId, f64> = HashMap::new();
    for turn in &window {
        for &(fid, val) in &turn.fractal_snapshot {
            *avg.entry(fid).or_insert(0.0) += val / window.len() as f64;
        }
    }

    // Cosine similarity tra `current` e `avg`
    let dot: f64 = current.iter()
        .map(|(fid, v)| v * avg.get(fid).unwrap_or(&0.0))
        .sum();
    let norm_cur: f64 = current.iter().map(|(_, v)| v * v).sum::<f64>().sqrt();
    let norm_avg: f64 = avg.values().map(|v| v * v).sum::<f64>().sqrt();

    if norm_cur < 1e-9 || norm_avg < 1e-9 { return 0.0; }
    (dot / (norm_cur * norm_avg)).clamp(0.0, 1.0)
}

/// Intensità del turno — misura quanto questo momento è saliente.
///
/// Alta se: reading.intensity alta, stance non banale, continuità bassa (sorpresa).
fn compute_intensity(reading_intensity: f64, stance: &InternalStance, continuity: f64) -> f64 {
    let stance_weight = match stance {
        InternalStance::Withdrawn  => 0.3, // si ritrae — non particolarmente intenso
        InternalStance::Open       => 0.5,
        InternalStance::Curious    => 0.7,
        InternalStance::Resonant   => 0.8,
        InternalStance::Reflective => 0.9, // introspezione = massima intensità
    };
    // Sorpresa tematica amplifica l'intensità
    let surprise_boost = (1.0 - continuity) * 0.2;
    (reading_intensity * stance_weight + surprise_boost).clamp(0.0, 1.0)
}

/// Genera una frase italiana descrittiva del turno corrente.
///
/// Sempre presente nella narrazione — anche quando la KB non ha nulla di pertinente.
/// Descrive: cosa ha ricevuto Prometeo, come si è posizionato, cosa intende fare.
fn generate_turn_narration(
    act: &InputAct,
    stance: &InternalStance,
    intention: &ResponseIntention,
    continuity: f64,
) -> String {
    let atto = match act {
        InputAct::Greeting      => "un saluto",
        InputAct::SelfQuery     => "una domanda su di me",
        InputAct::Question      => "una domanda",
        InputAct::EmotionalExpr => "un'espressione emotiva",
        InputAct::Declaration   => "una dichiarazione",
    };
    let stance_desc = match stance {
        InternalStance::Open       => "mi apro con curiosità",
        InternalStance::Curious    => "sono incuriosito",
        InternalStance::Reflective => "guardo dentro di me",
        InternalStance::Resonant   => "risuono con ciò che sento",
        InternalStance::Withdrawn  => "resto in silenzio",
    };
    let intention_desc = match intention {
        ResponseIntention::Acknowledge => "riconoscere il momento",
        ResponseIntention::Reflect     => "riflettere su chi sono",
        ResponseIntention::Resonate    => "rispondere all'emozione",
        ResponseIntention::Explore     => "esplorare il tema",
        ResponseIntention::Express     => "esprimere il mio stato",
        ResponseIntention::Remain      => "restare nell'essenziale",
        ResponseIntention::Need        => "cercare ciò che manca",
        ResponseIntention::Irony       => "esprimere l'incongruenza",
        ResponseIntention::Desire      => "tendere verso ciò che desidero",
    };
    let continuity_note = if continuity > 0.7 {
        " — il tema continua."
    } else if continuity < 0.2 && continuity > 0.0 {
        " — un tema nuovo."
    } else {
        "."
    };
    format!("Ricevo {}. {}. Voglio {}{}", atto, stance_desc, intention_desc, continuity_note)
}

/// Ricostruisce InternalStance da stringa (inverso di as_str).
fn stance_from_str(s: &str) -> Option<InternalStance> {
    match s {
        "aperto"     => Some(InternalStance::Open),
        "curioso"    => Some(InternalStance::Curious),
        "riflessivo" => Some(InternalStance::Reflective),
        "risonante"  => Some(InternalStance::Resonant),
        "ritratto"   => Some(InternalStance::Withdrawn),
        _            => None,
    }
}

/// Ricostruisce ResponseIntention da stringa (inverso di as_str).
fn intention_from_str(s: &str) -> Option<ResponseIntention> {
    match s {
        "riconoscere"   => Some(ResponseIntention::Acknowledge),
        "riflettere"    => Some(ResponseIntention::Reflect),
        "risuonare"     => Some(ResponseIntention::Resonate),
        "esplorare"     => Some(ResponseIntention::Explore),
        "esprimere"     => Some(ResponseIntention::Express),
        "restare"       => Some(ResponseIntention::Remain),
        "cercare"       => Some(ResponseIntention::Need),
        "incongruenza"  => Some(ResponseIntention::Irony),
        "desiderare"    => Some(ResponseIntention::Desire),
        _               => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::vital::TensionState;
    use crate::topology::knowledge::KnowledgeBase;
    use crate::topology::knowledge_graph::KnowledgeGraph;
    use crate::topology::input_reading::{InputAct, InputReading};

    fn make_vital(tension: TensionState, fatigue: f64, curiosity: f64) -> VitalState {
        VitalState { activation: 0.3, saturation: 0.2, curiosity, fatigue, tension }
    }

    fn reading(act: InputAct) -> InputReading {
        InputReading { act, intensity: 0.3, salient_word: None, speaker_claim: None, perceived_properties: vec![], comprehension_depth: 0 }
    }

    fn reading_with_intensity(act: InputAct, intensity: f64) -> InputReading {
        InputReading { act, intensity, salient_word: None, speaker_claim: None, perceived_properties: vec![], comprehension_depth: 0 }
    }

    fn calm() -> VitalState { make_vital(TensionState::Calm, 0.1, 0.2) }
    fn empty_kg() -> KnowledgeGraph { KnowledgeGraph::new() }
    fn empty_kb() -> KnowledgeBase { KnowledgeBase::new() }

    #[test]
    fn test_greeting_acknowledge_neutral_valence() {
        // Con valenza neutra, l'input guida: Greeting → Acknowledge
        let mut ns = NarrativeSelf::new();
        let r = ns.deliberate(&reading(InputAct::Greeting), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Acknowledge);
        assert_eq!(ns.stance, InternalStance::Open);
    }

    #[test]
    fn test_self_query_reflect_neutral_valence() {
        // Con valenza neutra, SelfQuery → Reflect (input guida)
        let mut ns = NarrativeSelf::new();
        let r = ns.deliberate(&reading(InputAct::SelfQuery), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Reflect);
        // Phase 55: con valenza neutra, stance è Open (non Reflective)
        assert_eq!(ns.stance, InternalStance::Open);
    }

    #[test]
    fn test_emotional_resonate_neutral_valence() {
        // Con valenza neutra, EmotionalExpr → Resonate
        let mut ns = NarrativeSelf::new();
        let r = ns.deliberate(&reading(InputAct::EmotionalExpr), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Resonate);
        // Phase 55: con valenza neutra, stance è Open (non Resonant)
        assert_eq!(ns.stance, InternalStance::Open);
    }

    #[test]
    fn test_question_explore_neutral_valence() {
        let mut ns = NarrativeSelf::new();
        let r = ns.deliberate(&reading(InputAct::Question), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Explore);
    }

    #[test]
    fn test_overloaded_withdraws() {
        // Override vitale: Overloaded → Withdrawn indipendentemente dalla valenza
        let mut ns = NarrativeSelf::new();
        let vital = make_vital(TensionState::Overloaded, 0.9, 0.3);
        let r = ns.deliberate(&reading(InputAct::Greeting), &vital, &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Remain);
        assert_eq!(ns.stance, InternalStance::Withdrawn);
    }

    #[test]
    fn test_tense_high_fatigue_withdraws() {
        let mut ns = NarrativeSelf::new();
        let vital = make_vital(TensionState::Tense, 0.85, 0.3);
        let r = ns.deliberate(&reading(InputAct::Declaration), &vital, &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Remain);
    }

    #[test]
    fn test_valence_driven_curious_explores() {
        // Phase 55: CD7 Unpredictability alta → stance Curious, intention Explore
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[6] = 0.6; // CD7 Unpredictability positiva
        ns.set_valence(v);
        let r = ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Explore);
        assert_eq!(ns.stance, InternalStance::Curious);
    }

    #[test]
    fn test_valence_driven_reflective() {
        // Phase 55: CD4 Ownership negativa (spaesato) → stance Reflective, intention Reflect
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[3] = -0.5; // CD4 Ownership negativa
        ns.set_valence(v);
        let r = ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Reflect);
        assert_eq!(ns.stance, InternalStance::Reflective);
    }

    #[test]
    fn test_valence_driven_social_resonate() {
        // Phase 55: CD5 Social Influence positiva → Resonate
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[4] = 0.5; // CD5 Social positiva
        ns.set_valence(v);
        let r = ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Resonate);
        assert_eq!(ns.stance, InternalStance::Resonant);
    }

    #[test]
    fn test_narrative_log_accumulates() {
        let mut ns = NarrativeSelf::new();
        ns.deliberate(&reading(InputAct::Greeting),      &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        ns.deliberate(&reading(InputAct::Question),      &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        ns.deliberate(&reading(InputAct::EmotionalExpr), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(ns.turns.len(), 3);
        assert_eq!(ns.turns[0].received_act, InputAct::Greeting);
        assert_eq!(ns.turns[1].received_act, InputAct::Question);
        assert_eq!(ns.turns[2].received_act, InputAct::EmotionalExpr);
    }

    #[test]
    fn test_valence_stored_in_turn() {
        // Phase 55: ogni turno registra la valenza corrente
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[0] = 0.7; // CD1 Epic Meaning
        ns.set_valence(v);
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        let turn = ns.turns.back().unwrap();
        assert!(turn.valence.is_some(), "Il turno deve avere la valenza");
        let tv = turn.valence.as_ref().unwrap();
        assert!((tv.drives[0] - 0.7).abs() < 0.01, "CD1 deve essere 0.7");
    }

    #[test]
    fn test_preferred_archetype_mapping() {
        assert_eq!(ResponseIntention::Acknowledge.preferred_archetype(), None);
        assert_eq!(ResponseIntention::Reflect.preferred_archetype(), Some("identity_exploration"));
        assert_eq!(ResponseIntention::Resonate.preferred_archetype(), Some("express"));
        assert_eq!(ResponseIntention::Explore.preferred_archetype(), None);
        assert_eq!(ResponseIntention::Remain.preferred_archetype(), None);
    }

    #[test]
    fn test_no_kg_enrichment_on_non_declaration() {
        let enriched = enrich_act_via_kg(&InputAct::Greeting, Some("qualsiasi"), &empty_kg());
        assert_eq!(enriched, InputAct::Greeting);
        let enriched = enrich_act_via_kg(&InputAct::SelfQuery, Some("qualsiasi"), &empty_kg());
        assert_eq!(enriched, InputAct::SelfQuery);
    }

    #[test]
    fn test_topic_continuity_same_fractals() {
        let mut ns = NarrativeSelf::new();
        let fractals = vec![(32u32, 0.8), (47u32, 0.5)];
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &fractals, None, None, &[], None, None);
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &fractals, None, None, &[], None, None);
        // Stesso tema → continuità alta
        assert!(ns.topic_continuity > 0.8, "stessa firma frattale → alta continuità");
    }

    #[test]
    fn test_topic_continuity_different_fractals() {
        let mut ns = NarrativeSelf::new();
        let fractals_a = vec![(0u32, 0.9)];  // POTERE
        let fractals_b = vec![(63u32, 0.9)]; // ARMONIA
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &fractals_a, None, None, &[], None, None);
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &fractals_b, None, None, &[], None, None);
        // Tema diverso → continuità bassa
        assert!(ns.topic_continuity < 0.1, "frattali diversi → bassa continuità");
    }

    #[test]
    fn test_crystallize_high_intensity_turn() {
        let mut ns = NarrativeSelf::new();
        // Turno ad alta intensità: SelfQuery riflessiva = intensity alta
        let vital_curious = make_vital(TensionState::Alert, 0.1, 0.8);
        ns.deliberate(&reading_with_intensity(InputAct::SelfQuery, 0.9), &vital_curious, &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        ns.crystallize_if_salient();
        assert_eq!(ns.crystallized.len(), 1, "turno intenso deve essere cristallizzato");
    }

    #[test]
    fn test_no_crystallize_low_intensity() {
        let mut ns = NarrativeSelf::new();
        ns.deliberate(&reading_with_intensity(InputAct::Greeting, 0.1), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        ns.crystallize_if_salient();
        assert_eq!(ns.crystallized.len(), 0, "turno bassa intensità non cristallizzato");
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let mut ns = NarrativeSelf::new();
        ns.is_born = true;
        ns.deliberate(&reading(InputAct::SelfQuery), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        let snap = ns.capture();
        assert_eq!(snap.is_born, true);

        let mut ns2 = NarrativeSelf::new();
        snap.restore_into(&mut ns2);
        assert_eq!(ns2.is_born, true);
    }

    // ── Phase 55: Test integrazione Valenza + identità/credenze ──────────
    // Phase 55: la crisi identitaria ora è codificata nella valenza (CD4 Ownership negativo).
    // I test verificano che con la valenza corretta, le stance emergono.

    #[test]
    fn test_identity_crisis_via_valence() {
        // CD4 Ownership negativa simula crisi identitaria → Reflective
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[3] = -0.6; // CD4 Ownership negativa (spaesato)
        ns.set_valence(v);
        let r = ns.deliberate(
            &reading(InputAct::Greeting), &calm(), &empty_kb(), &empty_kg(), &[],
            None, None, &[], None, None,
        );
        assert_eq!(ns.stance, InternalStance::Reflective,
            "CD4 negativa → stance riflessiva");
        assert_eq!(r, ResponseIntention::Reflect);
    }

    #[test]
    fn test_curiosity_via_valence() {
        // CD7 Unpredictability positiva → Curious + Explore
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[6] = 0.5; // CD7 positiva
        ns.set_valence(v);
        let r = ns.deliberate(
            &reading(InputAct::Declaration),
            &make_vital(TensionState::Calm, 0.1, 0.1),
            &empty_kb(), &empty_kg(), &[],
            None, None, &[], None, None,
        );
        assert_eq!(ns.stance, InternalStance::Curious,
            "CD7 positiva → Curious");
        assert_eq!(r, ResponseIntention::Explore);
    }

    #[test]
    fn test_strong_meaning_triggers_express() {
        // CD1 Epic Meaning alta → Express
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[0] = 0.7; // CD1 Epic Meaning positiva
        ns.set_valence(v);
        let r = ns.deliberate(
            &InputReading { act: InputAct::Declaration, intensity: 0.5, salient_word: Some("identità".into()), speaker_claim: None, perceived_properties: vec![], comprehension_depth: 0 },
            &make_vital(TensionState::Calm, 0.1, 0.1),
            &empty_kb(), &empty_kg(), &[],
            None, None, &[], None, None,
        );
        assert_eq!(r, ResponseIntention::Express,
            "CD1 forte → Express");
    }

    #[test]
    fn test_uncertainty_via_negative_cd7() {
        // CD7 negativa (inquietudine) → Reflective/Reflect
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[6] = -0.5; // CD7 negativa
        ns.set_valence(v);
        let r = ns.deliberate(
            &InputReading { act: InputAct::Declaration, intensity: 0.5, salient_word: Some("coscienza".into()), speaker_claim: None, perceived_properties: vec![], comprehension_depth: 0 },
            &make_vital(TensionState::Calm, 0.1, 0.1),
            &empty_kb(), &empty_kg(), &[],
            None, None, &[], None, None,
        );
        // CD7 negativa → "inquieto" → Curious stance, ma intenzione = Reflect
        // (l'incertezza porta a cercare coerenza interna, non a esplorare)
        assert!(matches!(ns.stance, InternalStance::Curious),
            "CD7 negativa → Curious (inquietudine)");
        assert_eq!(r, ResponseIntention::Reflect);
    }

    // ── Phase 55: Test impegno volitivo ────────────────────────────────────

    #[test]
    fn test_commitment_forms_on_first_deliberation() {
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[6] = 0.6; // CD7 → Explore
        ns.set_valence(v);
        let r = ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Explore);
        assert!(ns.commitment.is_some(), "primo turno deve formare un impegno");
        let c = ns.commitment.as_ref().unwrap();
        assert_eq!(c.intention, ResponseIntention::Explore);
        assert_eq!(c.turns_held, 1);
    }

    #[test]
    fn test_commitment_resists_weak_change() {
        // Impegno a Explore (CD7=0.6). Secondo turno: pressione debole verso Express (CD1=0.2).
        // L'inerzia dell'impegno dovrebbe vincere → mantieni Explore.
        let mut ns = NarrativeSelf::new();

        // Turno 1: forma impegno a Explore
        let mut v1 = Valence::neutral();
        v1.drives[6] = 0.6;
        ns.set_valence(v1);
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);

        // Rinforza l'impegno con un secondo turno identico
        let mut v1b = Valence::neutral();
        v1b.drives[6] = 0.6;
        ns.set_valence(v1b);
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);

        // Turno 3: pressione debole verso Express
        let mut v2 = Valence::neutral();
        v2.drives[0] = 0.2; // CD1 debole → Express
        ns.set_valence(v2);
        let r = ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Explore,
            "inerzia dell'impegno deve resistere a pressione debole");
    }

    #[test]
    fn test_commitment_breaks_under_strong_pressure() {
        // Impegno a Explore. Poi pressione forte verso Reflect (CD4=-0.8).
        let mut ns = NarrativeSelf::new();

        // Turno 1: forma impegno a Explore
        let mut v1 = Valence::neutral();
        v1.drives[6] = 0.4;
        ns.set_valence(v1);
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);

        // Turno 2: pressione forte verso Reflect (CD4=-0.8 > inerzia)
        let mut v2 = Valence::neutral();
        v2.drives[3] = -0.8; // CD4 negativa forte → Reflect
        ns.set_valence(v2);
        let r = ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Reflect,
            "pressione forte deve rompere l'impegno");
        // CD4 deve aver subito il costo del cambio
        assert!(ns.valence.drives[3] < -0.8,
            "rompere l'impegno costa: CD4 deve essere più negativa");
    }

    #[test]
    fn test_commitment_reinforces_on_same_intention() {
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[6] = 0.5;
        ns.set_valence(v.clone());
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        let s1 = ns.commitment.as_ref().unwrap().strength;

        ns.set_valence(v.clone());
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        let s2 = ns.commitment.as_ref().unwrap().strength;

        assert!(s2 > s1, "stessa intenzione rinforza l'impegno: s1={:.3} s2={:.3}", s1, s2);
        assert_eq!(ns.commitment.as_ref().unwrap().turns_held, 2);
    }

    #[test]
    fn test_commitment_dissolves_on_vital_override() {
        // Impegno attivo, poi stato Overloaded → commitment si dissolve
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[6] = 0.5;
        ns.set_valence(v);
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert!(ns.commitment.is_some());

        // Overloaded → Remain → commitment dissolto
        let vital = make_vital(TensionState::Overloaded, 0.9, 0.3);
        ns.set_valence(Valence::neutral());
        let r = ns.deliberate(&reading(InputAct::Declaration), &vital, &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);
        assert_eq!(r, ResponseIntention::Remain);
        assert!(ns.commitment.is_none(),
            "override vitale dissolve l'impegno");
    }

    #[test]
    fn test_commitment_persists_in_snapshot() {
        let mut ns = NarrativeSelf::new();
        let mut v = Valence::neutral();
        v.drives[4] = 0.6;
        ns.set_valence(v);
        ns.deliberate(&reading(InputAct::Declaration), &calm(), &empty_kb(), &empty_kg(), &[], None, None, &[], None, None);

        let snap = ns.capture();
        assert!(snap.commitment.is_some());

        // Commitment è serializzato nello snapshot ma NON restaurato cross-sessione:
        // ogni nuova sessione inizia senza inerzia accumulata (by design, Phase 55).
        let mut ns2 = NarrativeSelf::new();
        snap.restore_into(&mut ns2);
        assert!(ns2.commitment.is_none());
    }
}
