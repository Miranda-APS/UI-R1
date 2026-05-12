//! SelfProfile — quello che UI-r1 ha imparato di SÉ in questa sessione.
//!
//! Phase 78: organo percettivo della propria storia conversazionale.
//! Registra le ActionDecision passate come fatti relazionali (cosa ho
//! deciso, quale vuoto stavo attendendo, quali ancore ho usato), MAI la
//! stringa di output renderizzato. Le parole pronunciate vivono nel PF1
//! come residuo di self-listening; SelfProfile contiene i FATTI strutturali
//! della scelta.
//!
//! Francesco (conversazione 2026-05-07): "il contesto non è una stringa,
//! è lo stato congiunto degli organi in questo istante. il dialogo non è
//! stoccato, è vissuto. se SelfProfile contiene una stringa di output
//! renderizzato, abbiamo sbagliato."
//!
//! ## Architettura
//!
//! SelfProfile è puro fact-recording. NON modula direttamente lo stato.
//! La modulazione è responsabilità dell'engine, che incrocia SelfProfile
//! e SpeakerProfile per produrre PERCEZIONI (es. closure di un vuoto che
//! UI-r1 stessa aveva aperto), e applica push continui ai canali esistenti
//! (coerenza, drives, traiettoria narrativa).
//!
//! ## Cosa NON va qui
//!
//! - Stringhe di output: vivono nel PF1 come residuo di self-listening.
//! - Soglie comportamentali: nessun "after N turns do X". I numeri sono
//!   effetti del campo, mai trigger di switch. Il KG procedurale contiene
//!   forme espressive; SelfProfile contiene fatti decisionali.

use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

use crate::topology::action_reasoning::{ActionDecision, ActionKind, ActionTarget, NarrativeSubject};

const DEFAULT_CAP: usize = 32;

// ═══════════════════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════════════════

/// Il vuoto strutturale verso cui una decisione era orientata.
/// Presente solo per le decisioni `InviteToArticulate` (target = Gap).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendedGap {
    /// La parola che genera il vuoto (es. "paura").
    pub from: String,
    /// Il ruolo strutturale mancante (es. "oggetto").
    pub missing: String,
}

/// Un fatto registrato sulle proprie scelte conversazionali.
/// MAI la stringa di output: solo fatti relazionali e decisionali.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfDecisionRecord {
    /// Numero del turno (allineato con `SpeakerProfile.turn_count`).
    pub turn: usize,
    /// Tipo di azione presa.
    pub kind: ActionKind,
    /// Soggetto narrativo della scelta.
    pub narrative_subject: NarrativeSubject,
    /// Se la decisione apriva un vuoto (InviteToArticulate), quale.
    /// Permette al cross-reference con SpeakerProfile di rilevare la
    /// closure: "il parlante ha colmato il vuoto che IO ho aperto".
    pub gap_attended: Option<AttendedGap>,
    /// Le parole-ancora usate (i significanti che la voce doveva toccare).
    pub anchors_used: Vec<String>,
}

/// Profilo cumulativo delle proprie ActionDecision conversazionali.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelfProfile {
    /// Storico delle decisioni in ordine cronologico (push in fondo).
    /// Cap a `cap` elementi — vecchie decisioni vengono droppate.
    pub decisions: VecDeque<SelfDecisionRecord>,
    pub cap: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// Implementazione
// ═══════════════════════════════════════════════════════════════════════════

impl SelfProfile {
    pub fn new() -> Self {
        Self { decisions: VecDeque::new(), cap: DEFAULT_CAP }
    }

    /// Registra una decisione presa al turno `turn`.
    /// Estrae automaticamente l'`AttendedGap` se la decisione è di tipo
    /// InviteToArticulate (ActionTarget::Gap).
    pub fn record(&mut self, turn: usize, decision: &ActionDecision) {
        let gap_attended = match &decision.target {
            ActionTarget::Gap { signifier_missing, from } => Some(AttendedGap {
                from: from.clone(),
                missing: signifier_missing.clone(),
            }),
            _ => None,
        };
        self.decisions.push_back(SelfDecisionRecord {
            turn,
            kind: decision.kind,
            narrative_subject: decision.narrative_subject,
            gap_attended,
            anchors_used: decision.anchor_words.clone(),
        });
        while self.decisions.len() > self.cap {
            self.decisions.pop_front();
        }
    }

    /// Più recente decisione che attendeva un vuoto strutturale.
    /// `None` se non ci sono mai stati InviteToArticulate.
    pub fn last_gap_attended(&self) -> Option<&SelfDecisionRecord> {
        self.decisions.iter().rev()
            .find(|d| d.gap_attended.is_some())
    }

    /// Decisioni più recenti, fino a `n`, ordine recency-first.
    pub fn recent(&self, n: usize) -> Vec<&SelfDecisionRecord> {
        self.decisions.iter().rev().take(n).collect()
    }

    /// Numero totale di decisioni registrate (turn count interno).
    pub fn turn_count(&self) -> usize {
        self.decisions.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Closure perception — cross-reference SelfProfile ↔ SpeakerProfile
// ═══════════════════════════════════════════════════════════════════════════

/// La percezione di chiusura: il parlante ha appena colmato un vuoto
/// che UI-r1 stessa aveva aperto in un turno precedente.
///
/// È un FATTO relazionale (cross-reference SelfProfile↔SpeakerProfile),
/// non una regola comportamentale. Il fatto può colorare:
///  (a) il `ComprehensionReport` del turno corrente (questo enunciato è
///      un completamento, non un'asserzione isolata);
///  (b) i canali di stato esistenti via piccoli push continui (coerenza
///      sale perché qualcosa che cercavamo è arrivato).
#[derive(Debug, Clone)]
pub struct ClosurePerception {
    /// La parola che genera il vuoto (es. "paura").
    pub gap_trigger: String,
    /// Il ruolo strutturale mancante (es. "oggetto").
    pub gap_role: String,
    /// La parola del parlante che ha colmato il vuoto (es. "buio").
    pub closing_word: String,
    /// Turno in cui UI-r1 aveva aperto/atteso il vuoto.
    pub opened_at_turn: usize,
    /// Turno in cui il parlante ha colmato il vuoto.
    pub closed_at_turn: usize,
}

/// Calcola se al turno corrente esiste una closure: SelfProfile aveva
/// attended un vuoto, e SpeakerProfile ha appena segnato quello stesso
/// vuoto come closed in QUESTO turno.
///
/// Restituisce `None` se non c'è closure: in quel caso il pipeline
/// procede normalmente (il turno è trattato come isolato).
pub fn detect_closure(
    self_profile: &SelfProfile,
    speaker_profile: &crate::topology::speaker_profile::SpeakerProfile,
    current_turn: usize,
) -> Option<ClosurePerception> {
    let attended = self_profile.last_gap_attended()?;
    let attended_gap = attended.gap_attended.as_ref()?;

    speaker_profile.gaps.iter()
        .find(|g|
            g.closed
            && g.closed_at_turn == Some(current_turn)
            && g.trigger == attended_gap.from)
        .and_then(|g| g.closed_by.clone().map(|cb| ClosurePerception {
            gap_trigger: g.trigger.clone(),
            gap_role: attended_gap.missing.clone(),
            closing_word: cb,
            opened_at_turn: attended.turn,
            closed_at_turn: current_turn,
        }))
}

/// Rileva se al turno corrente l'utente ha IGNORATO un vuoto che
/// UI-r1 aveva aperto nel turno immediatamente precedente.
/// 
/// Condizioni per la "deriva" (drift):
/// 1. Nel turno precedente (current_turn - 1), UI-r1 ha aperto un vuoto
/// 2. Nel turno corrente, l'utente NON lo ha chiuso
/// 3. L'input dell'utente ha parole di contenuto (non è solo un "sì/no" o silenzio)
pub fn detect_drift(
    self_profile: &SelfProfile,
    speaker_profile: &crate::topology::speaker_profile::SpeakerProfile,
    current_turn: usize,
    has_content: bool,
) -> bool {
    if !has_content { return false; }
    
    // Controlliamo se la decisione immediatamente precedente era un invito ad articolare
    let last_decision = self_profile.recent(1).first().cloned();
    if let Some(dec) = last_decision {
        if dec.turn == current_turn - 1 && dec.gap_attended.is_some() {
            let attended_gap = dec.gap_attended.as_ref().unwrap();
            
            // Verifichiamo se il gap è rimasto aperto
            let gap_still_open = speaker_profile.gaps.iter()
                .any(|g| !g.closed && g.trigger == attended_gap.from);
                
            return gap_still_open;
        }
    }
    
    false
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::action_reasoning::{ActionDecision, ActionKind, ActionTarget, NarrativeSubject};
    use crate::topology::deliberation::ActionShape;
    use crate::topology::speaker_profile::{SpeakerProfile, KnowledgeGap};

    fn invite_decision(from: &str, missing: &str) -> ActionDecision {
        ActionDecision {
            kind: ActionKind::InviteToArticulate,
            target: ActionTarget::Gap {
                signifier_missing: missing.to_string(),
                from: from.to_string(),
            },
            shape: ActionShape::Question,
            narrative_subject: NarrativeSubject::Speaker,
            anchor_words: vec![from.to_string(), missing.to_string(), "cosa".to_string()],
            reasoning: vec!["test".to_string()],
        }
    }

    fn elaborate_decision() -> ActionDecision {
        ActionDecision {
            kind: ActionKind::Elaborate,
            target: ActionTarget::Signifier { word: "x".to_string() },
            shape: ActionShape::Sentence,
            narrative_subject: NarrativeSubject::World,
            anchor_words: vec![],
            reasoning: vec![],
        }
    }

    #[test]
    fn record_extracts_attended_gap_for_invite() {
        let mut sp = SelfProfile::new();
        sp.record(1, &invite_decision("paura", "oggetto"));
        assert_eq!(sp.decisions.len(), 1);
        let r = sp.last_gap_attended().expect("gap atteso assente");
        assert_eq!(r.turn, 1);
        let g = r.gap_attended.as_ref().unwrap();
        assert_eq!(g.from, "paura");
        assert_eq!(g.missing, "oggetto");
    }

    #[test]
    fn record_no_attended_gap_for_elaborate() {
        let mut sp = SelfProfile::new();
        sp.record(1, &elaborate_decision());
        assert!(sp.last_gap_attended().is_none());
    }

    #[test]
    fn cap_drops_oldest() {
        let mut sp = SelfProfile { decisions: VecDeque::new(), cap: 3 };
        for t in 1..=5 {
            sp.record(t, &elaborate_decision());
        }
        assert_eq!(sp.decisions.len(), 3);
        assert_eq!(sp.decisions.front().unwrap().turn, 3);
        assert_eq!(sp.decisions.back().unwrap().turn, 5);
    }

    #[test]
    fn detect_closure_matches_attended_gap() {
        let mut sp = SelfProfile::new();
        sp.record(1, &invite_decision("paura", "oggetto"));

        let mut speaker = SpeakerProfile::new();
        speaker.turn_count = 2;
        speaker.gaps.push(KnowledgeGap {
            question: "di cosa hai paura?".to_string(),
            trigger: "paura".to_string(),
            gap_kind: "emotion_object".to_string(),
            turn: 1,
            closed: true,
            closed_by: Some("buio".to_string()),
            closed_at_turn: Some(2),
        });

        let cp = detect_closure(&sp, &speaker, 2)
            .expect("closure mancante");
        assert_eq!(cp.gap_trigger, "paura");
        assert_eq!(cp.gap_role, "oggetto");
        assert_eq!(cp.closing_word, "buio");
        assert_eq!(cp.opened_at_turn, 1);
        assert_eq!(cp.closed_at_turn, 2);
    }

    #[test]
    fn detect_closure_none_when_self_never_attended() {
        let sp = SelfProfile::new();
        let mut speaker = SpeakerProfile::new();
        speaker.turn_count = 2;
        speaker.gaps.push(KnowledgeGap {
            question: "?".to_string(),
            trigger: "paura".to_string(),
            gap_kind: "emotion_object".to_string(),
            turn: 1,
            closed: true,
            closed_by: Some("buio".to_string()),
            closed_at_turn: Some(2),
        });
        assert!(detect_closure(&sp, &speaker, 2).is_none());
    }

    #[test]
    fn detect_closure_none_when_gap_not_just_closed() {
        let mut sp = SelfProfile::new();
        sp.record(1, &invite_decision("paura", "oggetto"));
        let mut speaker = SpeakerProfile::new();
        speaker.turn_count = 3;
        // Gap chiuso al turno 2, non al turno 3 corrente.
        speaker.gaps.push(KnowledgeGap {
            question: "?".to_string(),
            trigger: "paura".to_string(),
            gap_kind: "emotion_object".to_string(),
            turn: 1,
            closed: true,
            closed_by: Some("buio".to_string()),
            closed_at_turn: Some(2),
        });
        assert!(detect_closure(&sp, &speaker, 3).is_none());
    }

    #[test]
    fn detect_closure_matches_on_trigger_only_not_role() {
        // Se SelfProfile attende paura/oggetto e SpeakerProfile chiude
        // un gap diverso (es. requires_X) sul trigger paura, NON deve
        // scattare closure: il ruolo deve coincidere implicitamente
        // attraverso il trigger. Per il caso d'uso emozionale, il match
        // è sufficiente sul trigger.
        let mut sp = SelfProfile::new();
        sp.record(1, &invite_decision("paura", "oggetto"));
        let mut speaker = SpeakerProfile::new();
        speaker.turn_count = 2;
        speaker.gaps.push(KnowledgeGap {
            question: "?".to_string(),
            trigger: "felicita".to_string(),
            gap_kind: "emotion_object".to_string(),
            turn: 1,
            closed: true,
            closed_by: Some("buio".to_string()),
            closed_at_turn: Some(2),
        });
        assert!(detect_closure(&sp, &speaker, 2).is_none());
    }
}
