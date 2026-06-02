//! SpeakerProfile — quello che UI-r1 sa del parlante.
//!
//! Francesco (conversazione 2026-04-26): "ui-r1 deve poter capire cose di me
//! in base a quello che gli dico e soprattutto deve poter interessargli e
//! deve poter decidere come rispondere al meglio alle mie parole. fino a
//! che ui-r1 non capisce gli input non provare nemmeno a farmi leggere gli
//! output. non mi interessano fino a che lui non capisce cosa gli dico".
//!
//! Francesco (stessa conversazione): "come facciamo a far continuare il
//! dialogo tramite la narrativa e non tramite la persistenza di uno stato
//! che potrebbe voler dire tutto o niente?"
//!
//! La narrativa di una sessione non è uno stato che decade. È quello che
//! UI-r1 ha **imparato di te** turno per turno e che resta: i fatti che
//! hai affermato di te, le domande che le hai rivolto e che restano aperte
//! finché non senti di avere risposta, i concetti che hai menzionato (e che
//! continuerai a menzionare se sono importanti), i gap di conoscenza che
//! UI-r1 si porta dietro come materia di curiosità.
//!
//! Senza questo modello, UI-r1 può solo reagire a stimoli istantanei.
//! Con questo modello, può portare avanti un dialogo come una mente che
//! tiene a memoria chi ha davanti.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::topology::input_reading::{ClaimAgent, ClaimKind, SpeakerClaim};
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;
use crate::topology::deliberation::KgFacts;

// ═══════════════════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════════════════

/// Tipo di affermazione fatta dal parlante.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FactKind {
    /// "io sono X" / "tu sei X" — identità o appartenenza.
    Identity,
    /// "io ho X" / "sento X" / "provo X" — stato/possesso/sensazione.
    Feeling,
    /// "io faccio X" / "voglio X" / "penso X" — azione/intento.
    Action,
}

impl FactKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Feeling  => "feeling",
            Self::Action   => "action",
        }
    }
}

/// Un fatto che il parlante ha affermato (su sé o su UI-r1) in un turno.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpokenFact {
    pub kind: FactKind,
    /// La parola portatrice del predicato (es. "triste", "paura", "capire").
    pub predicate: String,
    /// Numero del turno in cui è stato affermato (1-based).
    pub turn: usize,
    /// Testo originale dell'input (per contesto/visualizzazione).
    pub raw_input: String,
}

/// Una domanda rivolta a UI-r1 ancora aperta — resta aperta finché non
/// viene marcata come risolta. Può sopravvivere fra turni.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenQuestion {
    /// Le radici dell'input al momento della domanda — il "topic".
    pub topic: Vec<String>,
    /// Tipo (interrogative pronoun se rilevato): chi/cosa/dove/quando/perché/come.
    pub interrogative: Option<String>,
    /// Testo originale.
    pub raw_input: String,
    pub turn: usize,
    /// True quando UI-r1 considera la domanda risposta.
    pub resolved: bool,
}

/// Un gap di conoscenza: il KG dice che la parola del parlante
/// strutturalmente RICHIEDE qualcosa che il parlante non ha specificato.
/// È materia di curiosità: domanda strutturale che UI-r1 può farsi/fare.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    /// Domanda formulata (in italiano) che colmerebbe il gap.
    pub question: String,
    /// Concetto che ha generato il gap (root dell'input).
    pub trigger: String,
    /// Tipo di gap (es. "emotion_object", "requires_X", "action_object").
    pub gap_kind: String,
    pub turn: usize,
    /// True se il gap è stato chiuso da un input successivo.
    pub closed: bool,
    /// Phase 78: la parola che ha colmato il vuoto, se chiuso.
    /// Permette al cross-reference SelfProfile↔SpeakerProfile di sapere
    /// CHE COSA ha articolato il parlante (es. "buio" per una paura).
    #[serde(default)]
    pub closed_by: Option<String>,
    /// Phase 78: turno in cui il vuoto è stato colmato.
    /// Distinguere il turno di apertura (`turn`) dal turno di chiusura
    /// permette di rilevare closure-percepibili nello stesso turno.
    #[serde(default)]
    pub closed_at_turn: Option<usize>,
}

/// Profilo cumulativo del parlante della sessione.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeakerProfile {
    /// Phase 73: nome del parlante se si è presentato ("mi chiamo X").
    /// Una sola identificazione per sessione — la prima rilevata.
    pub name: Option<String>,
    /// Fatti che il parlante ha affermato di SE STESSO.
    /// Ordine cronologico (push in fondo).
    pub self_facts: Vec<SpokenFact>,
    /// Fatti che il parlante ha affermato su UI-r1 ("tu sei X").
    pub entity_facts: Vec<SpokenFact>,
    /// Domande aperte rivolte a UI-r1.
    pub open_questions: Vec<OpenQuestion>,
    /// Concetti menzionati (parola → conteggio cumulativo).
    pub mentioned: HashMap<String, u32>,
    /// Gap di conoscenza accumulati — quello che UI-r1 vorrebbe sapere.
    pub gaps: Vec<KnowledgeGap>,
    /// Numero di turni del parlante osservati.
    pub turn_count: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// Implementazione
// ═══════════════════════════════════════════════════════════════════════════

impl SpeakerProfile {
    pub fn new() -> Self { Self::default() }

    /// Phase 73: registra il nome del parlante se non è ancora noto.
    /// La prima presentazione vince — identità del parlante stabile per sessione.
    pub fn set_name_if_unset(&mut self, name: &str) {
        if self.name.is_none() {
            self.name = Some(name.to_string());
        }
    }

    /// Osserva un nuovo turno e aggiorna il profilo.
    ///
    /// Letture necessarie:
    ///  - `raw_input`: testo originale per contesto
    ///  - `kg_facts`: fatti strutturali già calcolati (radici, marker, classi)
    ///  - `speaker_claim`: pattern S-V-P se rilevato dall'input
    ///  - `kg`: per derivare gap strutturali (Requires non specificati, ecc.)
    pub fn observe_turn(
        &mut self,
        raw_input: &str,
        kg_facts: &KgFacts,
        speaker_claim: Option<&SpeakerClaim>,
        kg: &KnowledgeGraph,
    ) {
        self.turn_count += 1;
        let turn = self.turn_count;

        // ── 1. Mentioned — incrementa per ogni radice non-funzionale ──────
        for r in &kg_facts.roots {
            *self.mentioned.entry(r.clone()).or_insert(0) += 1;
        }

        // ── 2. SpokenFact — dal SpeakerClaim se presente ──────────────────
        if let Some(sc) = speaker_claim {
            let kind = match sc.kind {
                ClaimKind::Identity => FactKind::Identity,
                ClaimKind::Feeling  => FactKind::Feeling,
                ClaimKind::Action   => FactKind::Action,
            };
            let fact = SpokenFact {
                kind,
                predicate: sc.predicate.clone(),
                turn,
                raw_input: raw_input.to_string(),
            };
            match sc.agent {
                ClaimAgent::Speaker => self.self_facts.push(fact),
                ClaimAgent::Entity  => self.entity_facts.push(fact),
            }
        }

        // ── 3. OpenQuestion — se l'input ha forma interrogativa ───────────
        if kg_facts.has_question_marker || kg_facts.has_interrogative_pronoun {
            // Cerca il pronome interrogativo nelle radici (se presente).
            let interrogatives = ["chi", "cosa", "che", "dove", "quando",
                                  "perché", "perche", "come", "quale",
                                  "quali", "quanto", "quanta", "quanti", "quante"];
            let interrogative = kg_facts.roots.iter()
                .find(|r| interrogatives.contains(&r.as_str()))
                .cloned();
            // Topic = radici NON-interrogative (la cosa di cui si chiede)
            let topic: Vec<String> = kg_facts.roots.iter()
                .filter(|r| !interrogatives.contains(&r.as_str()))
                .cloned()
                .collect();
            self.open_questions.push(OpenQuestion {
                topic, interrogative,
                raw_input: raw_input.to_string(),
                turn, resolved: false,
            });
        }

        // ── 4. KnowledgeGap — derivati strutturalmente dal KG ─────────────
        // Per ogni radice, cerca le sue Requires e i suoi parent IsA.
        // Se la radice è un'emozione e c'è uno SpeakerClaim Feeling,
        // il gap è "di cosa?".
        // Se la radice ha Requires nel KG, il gap è "qual è X per te?".
        for r in &kg_facts.roots {
            // Gap emozionale: emozione affermata senza oggetto
            let parent_classes = kg.query_objects_weighted(r, RelationType::IsA);
            let is_emotion = parent_classes.iter().any(|(p, _)|
                ["emozione", "sentimento", "stato_d_animo", "sensazione", "affetto"]
                    .contains(p));
            if is_emotion && speaker_claim
                .map(|sc| matches!(sc.kind, ClaimKind::Feeling))
                .unwrap_or(false)
            {
                let q = format!("di cosa hai {}?", r);
                let kind_label = "emotion_object";
                if !self.gaps.iter().any(|g|
                    g.trigger == *r && g.gap_kind == kind_label && !g.closed)
                {
                    self.gaps.push(KnowledgeGap {
                        question: q,
                        trigger: r.clone(),
                        gap_kind: kind_label.to_string(),
                        turn, closed: false,
                        closed_by: None,
                        closed_at_turn: None,
                    });
                }
            }

            // Gap strutturale: Requires nel KG non specificato dal parlante
            let reqs = kg.query_objects_weighted(r, RelationType::Requires);
            for (req, _conf) in reqs.iter().take(2) {
                // Se il parlante ha menzionato `req` in un turno precedente
                // o nello stesso input, considera il gap già coperto.
                if self.mentioned.contains_key(*req)
                    || kg_facts.roots.iter().any(|root| root == req)
                {
                    continue;
                }
                let kind_label = format!("requires_{}", req);
                if self.gaps.iter().any(|g|
                    g.trigger == *r && g.gap_kind == kind_label && !g.closed)
                {
                    continue;
                }
                let q = format!("cosa è {} per te in \"{}\"?", req, r);
                self.gaps.push(KnowledgeGap {
                    question: q,
                    trigger: r.clone(),
                    gap_kind: kind_label,
                    turn, closed: false,
                    closed_by: None,
                    closed_at_turn: None,
                });
            }
        }

        // ── 5. Chiusura gap: se le radici di QUESTO turno coprono un gap
        //    aperto da un turno precedente, marcalo come chiuso. ──────────
        // Phase 78: cattura la parola che ha colmato il vuoto e il turno —
        // serve al cross-reference SelfProfile↔SpeakerProfile per la
        // closure perception.
        let current_roots: std::collections::HashSet<&str> = kg_facts.roots
            .iter().map(|s| s.as_str()).collect();
        for gap in self.gaps.iter_mut() {
            if gap.closed { continue; }
            // Se il gap era "di cosa hai paura?" e l'input corrente menziona
            // qualcosa che potrebbe essere l'oggetto della paura, chiudi.
            // Euristica: se il gap.kind è "emotion_object" e l'input ha root
            // diversa dal trigger (paura) e contiene una parola-contenuto,
            // consideralo chiuso.
            if gap.gap_kind == "emotion_object" {
                // Phase 80: se il turno corrente porta un PROPRIO claim del
                // parlante (un nuovo posizionamento), non è una chiusura del
                // gap precedente — è un cambio di tema. Il vecchio gap resta
                // aperto (verrà scartato dalla finestra temporale), ma questo
                // turno NON è la sua articolazione.
                // Caso d'uso: "sono triste" → "mi chiamo francesco":
                //   il gap "oggetto-di-triste" non va chiuso da "francesco".
                if speaker_claim.is_some() {
                    continue;
                }
                let has_other = current_roots.iter().any(|r| **r != gap.trigger);
                if has_other && kg_facts.content_word_count >= 1 {
                    gap.closed = true;
                    let closing = kg_facts.roots.iter()
                        .find(|r| r.as_str() != gap.trigger.as_str())
                        .cloned();
                    gap.closed_by = closing;
                    gap.closed_at_turn = Some(turn);
                }
            }
            // Se il gap è "requires_X" e il parlante ora menziona X
            if gap.gap_kind.starts_with("requires_") {
                let req = &gap.gap_kind[9..];
                if current_roots.contains(req) {
                    gap.closed = true;
                    gap.closed_by = Some(req.to_string());
                    gap.closed_at_turn = Some(turn);
                }
            }
        }
    }

    // ─── Query / accessori ─────────────────────────────────────────────

    /// Numero di self_facts.
    pub fn self_fact_count(&self) -> usize { self.self_facts.len() }

    /// Domande aperte non risolte.
    pub fn unresolved_questions(&self) -> impl Iterator<Item = &OpenQuestion> {
        self.open_questions.iter().filter(|q| !q.resolved)
    }

    /// Gap aperti.
    pub fn open_gaps(&self) -> impl Iterator<Item = &KnowledgeGap> {
        self.gaps.iter().filter(|g| !g.closed)
    }

    /// Concetti menzionati ordinati per conteggio decrescente.
    pub fn top_mentioned(&self, n: usize) -> Vec<(String, u32)> {
        let mut v: Vec<(String, u32)> = self.mentioned.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v.truncate(n);
        v
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::input_reading::{SpeakerClaim, ClaimAgent, ClaimKind};
    use crate::topology::knowledge_graph::KnowledgeGraph;

    fn empty_kg() -> KnowledgeGraph {
        KnowledgeGraph::new()
    }

    fn facts(roots: Vec<&str>, has_q: bool, has_pron: bool, content_count: usize) -> KgFacts {
        KgFacts {
            roots: roots.iter().map(|s| s.to_string()).collect(),
            root_classes: vec![],
            specific_class: None,
            class_siblings_count: 0,
            has_question_marker: has_q,
            has_interrogative_pronoun: has_pron,
            speaker_claim: None,
            content_word_count: content_count,
            emotional_proximity: 0.0,
            self_referenced: false,
        }
    }

    #[test]
    fn observes_speaker_self_claim() {
        let mut p = SpeakerProfile::new();
        let kg = empty_kg();
        let f = facts(vec!["paura"], false, false, 1);
        let sc = SpeakerClaim {
            agent: ClaimAgent::Speaker,
            kind: ClaimKind::Feeling,
            predicate: "paura".to_string(),
            verb_category: Some("copula".to_string()),
            complement: None,
        };
        p.observe_turn("ho paura", &f, Some(&sc), &kg);
        assert_eq!(p.self_facts.len(), 1);
        assert_eq!(p.self_facts[0].predicate, "paura");
        assert_eq!(p.self_facts[0].kind, FactKind::Feeling);
    }

    #[test]
    fn observes_entity_claim_into_entity_facts() {
        let mut p = SpeakerProfile::new();
        let kg = empty_kg();
        let f = facts(vec!["bello"], false, false, 1);
        let sc = SpeakerClaim {
            agent: ClaimAgent::Entity,
            kind: ClaimKind::Identity,
            predicate: "bello".to_string(),
            verb_category: Some("copula".to_string()),
            complement: None,
        };
        p.observe_turn("tu sei bello", &f, Some(&sc), &kg);
        assert_eq!(p.entity_facts.len(), 1);
        assert_eq!(p.self_facts.len(), 0);
    }

    #[test]
    fn question_marker_creates_open_question() {
        let mut p = SpeakerProfile::new();
        let kg = empty_kg();
        let f = facts(vec!["essere", "chi"], true, true, 1);
        p.observe_turn("chi sei?", &f, None, &kg);
        assert_eq!(p.open_questions.len(), 1);
        assert_eq!(p.open_questions[0].interrogative.as_deref(), Some("chi"));
        assert_eq!(p.open_questions[0].topic, vec!["essere".to_string()]);
    }

    #[test]
    fn emotion_claim_creates_gap_of_cause() {
        let mut p = SpeakerProfile::new();
        let mut kg = empty_kg();
        kg.add("paura", RelationType::IsA, "emozione");
        let f = facts(vec!["paura"], false, false, 1);
        let sc = SpeakerClaim {
            agent: ClaimAgent::Speaker,
            kind: ClaimKind::Feeling,
            predicate: "paura".to_string(),
            verb_category: Some("copula".to_string()),
            complement: None,
        };
        p.observe_turn("ho paura", &f, Some(&sc), &kg);
        assert!(p.open_gaps().any(|g| g.trigger == "paura" && g.gap_kind == "emotion_object"),
            "gap mancante: {:?}", p.gaps);
    }

    #[test]
    fn subsequent_input_closes_emotion_gap() {
        let mut p = SpeakerProfile::new();
        let mut kg = empty_kg();
        kg.add("paura", RelationType::IsA, "emozione");
        // Turn 1: "ho paura" — apre gap emotion_object
        let f1 = facts(vec!["paura"], false, false, 1);
        let sc = SpeakerClaim {
            agent: ClaimAgent::Speaker,
            kind: ClaimKind::Feeling,
            predicate: "paura".to_string(),
            verb_category: Some("copula".to_string()),
            complement: None,
        };
        p.observe_turn("ho paura", &f1, Some(&sc), &kg);
        // Turn 2: "del buio" — gap dovrebbe chiudersi (l'oggetto della paura è specificato)
        let f2 = facts(vec!["buio"], false, false, 1);
        p.observe_turn("del buio", &f2, None, &kg);
        let still_open: Vec<&KnowledgeGap> = p.open_gaps()
            .filter(|g| g.trigger == "paura" && g.gap_kind == "emotion_object")
            .collect();
        assert!(still_open.is_empty(), "gap ancora aperto: {:?}", still_open);
    }

    #[test]
    fn requires_gap_closes_when_speaker_mentions_required() {
        let mut p = SpeakerProfile::new();
        let mut kg = empty_kg();
        kg.add("fuoco", RelationType::Requires, "ossigeno");
        // Turn 1: "il fuoco" — apre gap requires_ossigeno
        let f1 = facts(vec!["fuoco"], false, false, 1);
        p.observe_turn("il fuoco", &f1, None, &kg);
        assert!(p.open_gaps().any(|g| g.gap_kind == "requires_ossigeno"));
        // Turn 2: "ossigeno" — chiude
        let f2 = facts(vec!["ossigeno"], false, false, 1);
        p.observe_turn("ossigeno", &f2, None, &kg);
        assert!(!p.open_gaps().any(|g| g.gap_kind == "requires_ossigeno"));
    }

    #[test]
    fn mentioned_accumulates_across_turns() {
        let mut p = SpeakerProfile::new();
        let kg = empty_kg();
        let f = facts(vec!["mare"], false, false, 1);
        p.observe_turn("il mare", &f, None, &kg);
        p.observe_turn("il mare", &f, None, &kg);
        p.observe_turn("il mare", &f, None, &kg);
        assert_eq!(p.mentioned.get("mare"), Some(&3));
    }

    #[test]
    fn turn_count_increments_per_observation() {
        let mut p = SpeakerProfile::new();
        let kg = empty_kg();
        let f = facts(vec!["x"], false, false, 1);
        p.observe_turn("x", &f, None, &kg);
        p.observe_turn("x", &f, None, &kg);
        assert_eq!(p.turn_count, 2);
    }
}
