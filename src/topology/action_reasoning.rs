//! ActionReasoning — il ragionamento esplicito sull'azione.
//!
//! Francesco (conversazione 2026-04-26): "ui-r1 dovrebbe come prima cosa
//! tradurre cognitivamente l'input e scrivere cosa ha capito, poi dovrebbe
//! ragionare su come si reagisce a quell'input".
//!
//! Phase 73 ha costruito il primo livello — il `ComprehensionReport`
//! dove UI-r1 SCRIVE quello che ha capito. Phase 74 costruisce il secondo:
//! il documento dove UI-r1 SCRIVE come risponderà — quale azione fare,
//! cosa indirizzare, quale forma scegliere, perché.
//!
//! Niente silenzio (Francesco, 2026-04-26: "fai attenzione con il 'tacere'
//! perché è una funzione che per ora non possiamo permetterci"). Niente
//! enum-driven dispatch — la decisione è derivata STRUTTURALMENTE dal
//! report e dallo stato.
//!
//! ## Le 5 azioni possibili
//!
//! 1. **InviteToArticulate** — il parlante ha aperto un vuoto significativo
//!    (Requires non articolato). Inviti a portare al significante.
//! 2. **AnswerOpenQuestion** — il parlante ha posto una domanda. Tenti
//!    una risposta dal KG / da te stessa.
//! 3. **RecognizeClaim** — il parlante si è posizionato. Restituisci
//!    riconoscimento simbolico (lacaniano: "ho ricevuto la tua parola").
//! 4. **PhaticReturn** — atto fatico (saluto/congedo/cortesia). Ricambi.
//! 5. **Elaborate** — l'enunciato non ha claim né gap salienti. Elabori
//!    sulla materia portata.

use serde::{Serialize, Deserialize};

use crate::topology::comprehension_report::ComprehensionReport;
use crate::topology::deliberation::ActionShape;
use crate::topology::speaker_profile::SpeakerProfile;

// ═══════════════════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    InviteToArticulate,
    AnswerOpenQuestion,
    RecognizeClaim,
    PhaticReturn,
    Elaborate,
}

impl ActionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InviteToArticulate => "invitare-ad-articolare",
            Self::AnswerOpenQuestion => "rispondere-alla-domanda",
            Self::RecognizeClaim => "riconoscere-il-posizionamento",
            Self::PhaticReturn => "ricambio-fatico",
            Self::Elaborate => "elaborare",
        }
    }
}

/// L'oggetto su cui l'azione si concentra.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionTarget {
    /// Un vuoto specifico nella catena del report.
    Gap {
        /// Il significante mancante (es. "oggetto", "causa").
        signifier_missing: String,
        /// La parola da cui nasce il vuoto (es. "paura").
        from: String,
    },
    /// Una domanda aperta del parlante (dal SpeakerProfile o dal turno).
    OpenQuestion {
        question_text: String,
        topic: Vec<String>,
    },
    /// Un claim del parlante da riconoscere.
    Claim {
        kind: String,
        predicate: String,
    },
    /// La classe di un atto fatico (es. "saluto").
    PhaticClass {
        class: String,
    },
    /// Un significante centrale dell'enunciato.
    Signifier {
        word: String,
    },
}

/// Soggetto narrativo dell'azione.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NarrativeSubject {
    /// Parla del parlante.
    Speaker,
    /// Parla di UI-r1.
    Self_,
    /// Parla del mondo / concetti.
    World,
    /// Parla del rapporto (atto fatico, riconoscimento).
    Relation,
}

impl NarrativeSubject {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Speaker  => "parlante",
            Self::Self_    => "UI-r1",
            Self::World    => "mondo",
            Self::Relation => "rapporto",
        }
    }
}

/// La decisione di azione esplicita per il turno corrente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDecision {
    /// Tipo di azione.
    pub kind: ActionKind,
    /// L'oggetto su cui agire.
    pub target: ActionTarget,
    /// Forma fisica dell'uscita.
    pub shape: ActionShape,
    /// Soggetto narrativo: chi parla, di chi.
    pub narrative_subject: NarrativeSubject,
    /// Parole-chiave che devono guidare la generazione (anchor).
    /// Sono i significanti che la risposta DEVE toccare per essere
    /// coerente con la decisione presa.
    pub anchor_words: Vec<String>,
    /// Note testuali del ragionamento (perché questa decisione).
    pub reasoning: Vec<String>,
}

impl ActionDecision {
    /// Render testuale del documento di decisione (italiano leggibile).
    pub fn compose_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("AZIONE SCELTA\n"));
        out.push_str(&format!("  tipo: {}\n", self.kind.as_str()));
        out.push_str(&format!("  forma: {}\n", self.shape.as_str()));
        out.push_str(&format!("  soggetto narrativo: {}\n", self.narrative_subject.as_str()));
        out.push_str("  oggetto:\n");
        match &self.target {
            ActionTarget::Gap { signifier_missing, from } =>
                out.push_str(&format!(
                    "    vuoto: \"{}\" rinvia a \"{}\" (mancante)\n",
                    from, signifier_missing,
                )),
            ActionTarget::OpenQuestion { question_text, topic } =>
                out.push_str(&format!(
                    "    domanda aperta: \"{}\" (su: {})\n",
                    question_text, topic.join(", "),
                )),
            ActionTarget::Claim { kind, predicate } =>
                out.push_str(&format!(
                    "    posizionamento del parlante: {} \"{}\"\n", kind, predicate)),
            ActionTarget::PhaticClass { class } =>
                out.push_str(&format!("    classe fatica: \"{}\"\n", class)),
            ActionTarget::Signifier { word } =>
                out.push_str(&format!("    significante: \"{}\"\n", word)),
        }
        if !self.anchor_words.is_empty() {
            out.push_str(&format!("  parole-ancora: [{}]\n", self.anchor_words.join(", ")));
        }
        out.push_str("\nRAGIONAMENTO\n");
        for r in &self.reasoning {
            out.push_str(&format!("  - {}\n", r));
        }
        out
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Costruzione della decisione
// ═══════════════════════════════════════════════════════════════════════════

/// Decide l'azione data la comprensione del report e lo stato del parlante.
///
/// Ordine di priorità (da più specifico a più generico):
///  1. Se l'atto è interrogazione → AnswerOpenQuestion
///  2. Se l'atto è atto-fatico breve in classe specifica → PhaticReturn
///  3. Se c'è uno SpeakerClaim e c'è almeno un vuoto strutturale → InviteToArticulate
///     (il claim apre la domanda; il vuoto la specifica)
///  4. Se c'è uno SpeakerClaim e nessun vuoto → RecognizeClaim
///  5. Altrimenti → Elaborate (asserzione del parlante o input non classificato)
pub fn decide_action(
    report: &ComprehensionReport,
    speaker_profile: &SpeakerProfile,
) -> ActionDecision {
    let mut reasoning: Vec<String> = Vec::new();

    // Phase 79: la closure NON è più un dispatch if/then a RecognizeClaim.
    // È un percetto ("chiusura") seminato nel campo del kg_proc da
    // `seed_from_comprehension`, che attiva i concetti `restituire` e
    // `posizione`; il pattern `riconoscimento` (UsedFor restituire via=posizione)
    // vince per risonanza in `select_pattern_by_resonance`. Il rendering
    // legge `report.closes_prior_gap` direttamente per estrarre trigger e
    // closing_word. Qui annotiamo solo la percezione nel reasoning per
    // trasparenza, senza forzare la decisione.
    if let Some(c) = &report.closes_prior_gap {
        reasoning.push(format!(
            "percezione: il parlante ha colmato il vuoto che io avevo aperto \
             al turno {} — \"{}\" articola il {} di \"{}\". \
             Il percetto \"chiusura\" attiva il campo procedurale; il pattern \
             di riconoscimento emerge per risonanza.",
            c.opened_at_turn, c.closing_word, c.role, c.trigger,
        ));
    }

    // ── 1. Interrogazione → AnswerOpenQuestion ────────────────────────
    if report.speech_act.kind == "interrogazione" {
        reasoning.push(
            "l'atto di parola è interrogativo: il parlante attende informazione".to_string(),
        );
        // Il topic della domanda è l'enunciato stesso (es. "chi sei?")
        let topic: Vec<String> = report.symbolic_positions.iter()
            .map(|p| p.signifier.clone())
            .collect();
        let target = ActionTarget::OpenQuestion {
            question_text: report.utterance.clone(),
            topic: topic.clone(),
        };
        // Soggetto narrativo: la domanda è rivolta all'entità se:
        //  (a) compare "tu" come pronome di riferimento, oppure
        //  (b) "chi"/"io" è una radice topic (es. "chi sei?"), oppure
        //  (c) l'enunciato contiene un verbo coniugato in 2a singolare
        //      (es. "come stai?", "stai bene?") — l'entità è il soggetto
        //      grammaticale del verbo del parlante.
        let utt_lower = report.utterance.to_lowercase();
        let has_2sg_verb = {
            use crate::topology::grammar::{lemmatize, Person};
            utt_lower.split(|c: char| !c.is_alphabetic())
                .filter(|t| !t.is_empty())
                .any(|t| lemmatize(t).map(|r| matches!(r.person, Person::Second)).unwrap_or(false))
        };
        let self_ref = utt_lower.contains("tu")
            || topic.iter().any(|t| t == "chi" || t == "io")
            || has_2sg_verb;
        let narrative_subject = if self_ref {
            reasoning.push(
                "la domanda è rivolta a me (presenza di \"chi\" / \"tu\") — \
                 il soggetto della risposta sono io".to_string(),
            );
            NarrativeSubject::Self_
        } else {
            reasoning.push(
                "la domanda è sul mondo — il soggetto della risposta è la materia chiesta".to_string(),
            );
            NarrativeSubject::World
        };
        // Anchor words: dipende dal soggetto.
        let anchor_words = if narrative_subject == NarrativeSubject::Self_ {
            // Quando rispondo su di me: parole che mi descrivono.
            // Per ora: signifier dell'enunciato + concetti emergenti dalle inferenze.
            let mut words: Vec<String> = topic.clone();
            for inf in &report.inferences {
                for w in &inf.chain {
                    if !words.contains(w) { words.push(w.clone()); }
                }
            }
            words
        } else {
            // Quando rispondo sul mondo: i significanti dell'enunciato + le loro
            // conseguenze/composizioni dal report.
            let mut words: Vec<String> = topic.clone();
            for pos in &report.symbolic_positions {
                for (_, t) in pos.points_to.iter().take(2) {
                    if !words.contains(t) { words.push(t.clone()); }
                }
            }
            words
        };
        return ActionDecision {
            kind: ActionKind::AnswerOpenQuestion,
            target,
            shape: ActionShape::Sentence,
            narrative_subject,
            anchor_words,
            reasoning,
        };
    }

    // ── 2. Atto fatico breve in classe → PhaticReturn ────────────────
    if report.speech_act.kind == "atto-fatico" {
        reasoning.push(
            "l'atto è fatico: la risposta appropriata è in registro fatico simmetrico".to_string(),
        );
        // Estrai la classe (es. "saluto") dalla descrizione del speech_act.
        // È sempre presente per atti fatici.
        let class = extract_class_from_phatic_description(&report.speech_act.description)
            .unwrap_or_else(|| "atto".to_string());
        // Anchor words: i fratelli (simili) della classe + significanti correlati
        let mut anchor_words: Vec<String> = Vec::new();
        for pos in &report.symbolic_positions {
            // Aggiungi i "simile a" come fratelli
            for (rel, target) in &pos.points_to {
                if rel == "simile a" && !anchor_words.contains(target) {
                    anchor_words.push(target.clone());
                }
            }
        }
        return ActionDecision {
            kind: ActionKind::PhaticReturn,
            target: ActionTarget::PhaticClass { class },
            shape: ActionShape::Word,
            narrative_subject: NarrativeSubject::Relation,
            anchor_words,
            reasoning,
        };
    }

    // ── 3+4. Posizionamento / Feeling / Action — gestiamo i claim ────
    let speaker_claim_kind = report.speech_act.kind.as_str();
    let claim_predicates: Vec<String> = speaker_profile.self_facts.iter()
        .filter(|f| f.turn == speaker_profile.turn_count)
        .map(|f| f.predicate.clone())
        .collect();

    let is_claim_act = matches!(
        speaker_claim_kind,
        "posizionamento" | "denominazione" | "dichiarazione-di-azione"
    );

    if is_claim_act {
        let predicate = claim_predicates.first().cloned()
            .unwrap_or_else(|| {
                // Fallback: il primo significante del report
                report.symbolic_positions.first()
                    .map(|p| p.signifier.clone())
                    .unwrap_or_default()
            });

        // ── 3. Vuoti aperti → InviteToArticulate ──────────────────────
        if !report.gaps.is_empty() {
            let gap = &report.gaps[0];
            reasoning.push(format!(
                "il parlante si è posizionato come \"{}\" — c'è un vuoto strutturale: \
                 \"{}\" rinvia a \"{}\" che non è stato articolato",
                predicate, gap.from, gap.missing,
            ));
            reasoning.push(
                "lacanianamente, questo vuoto è una soglia di desiderio: \
                 invito il parlante a portarlo al significante".to_string(),
            );
            // Anchor: il significante mancante + il significante del claim +
            // pronomi interrogativi che permettono la formulazione.
            let mut anchor_words: Vec<String> = vec![
                gap.missing.clone(),
                gap.from.clone(),
            ];
            // Aggiungi pronome interrogativo coerente con la classe del gap
            let pronoun = match gap.missing.as_str() {
                "oggetto" | "causa" | "ragione" | "bisogno" => "cosa",
                "luogo" | "spazio" => "dove",
                "tempo" => "quando",
                "persona" | "soggetto" => "chi",
                "modo" | "modalità" => "come",
                _ => "cosa",
            };
            if !anchor_words.contains(&pronoun.to_string()) {
                anchor_words.push(pronoun.to_string());
            }
            return ActionDecision {
                kind: ActionKind::InviteToArticulate,
                target: ActionTarget::Gap {
                    signifier_missing: gap.missing.clone(),
                    from: gap.from.clone(),
                },
                shape: ActionShape::Question,
                narrative_subject: NarrativeSubject::Speaker,
                anchor_words,
                reasoning,
            };
        }

        // ── 4. Nessun vuoto → RecognizeClaim ──────────────────────────
        reasoning.push(format!(
            "il parlante si è posizionato come \"{}\" — niente vuoti strutturali da invitare \
             ad articolare. Restituisco riconoscimento simbolico.",
            predicate,
        ));
        let mut anchor_words: Vec<String> = vec![predicate.clone()];
        // Aggiungi opposti / simili per dare colore senza imitare empatia
        for pos in &report.symbolic_positions {
            for s in pos.serves_in.iter().take(1) {
                if !anchor_words.contains(s) { anchor_words.push(s.clone()); }
            }
        }
        return ActionDecision {
            kind: ActionKind::RecognizeClaim,
            target: ActionTarget::Claim {
                kind: speaker_claim_kind.to_string(),
                predicate,
            },
            shape: ActionShape::Sentence,
            narrative_subject: NarrativeSubject::Speaker,
            anchor_words,
            reasoning,
        };
    }

    // ── 5. Default → Elaborate ────────────────────────────────────────
    reasoning.push(
        "l'enunciato è un'asserzione senza claim né domanda: \
         elaboro sulla materia portata".to_string(),
    );
    let mut anchor_words: Vec<String> = report.symbolic_positions.iter()
        .map(|p| p.signifier.clone())
        .collect();
    // Aggiungi le conseguenze più rilevanti
    for pos in &report.symbolic_positions {
        for (rel, t) in pos.points_to.iter() {
            if (rel == "causa" || rel == "ha") && !anchor_words.contains(t) {
                anchor_words.push(t.clone());
            }
        }
    }
    let target = match report.symbolic_positions.first() {
        Some(p) => ActionTarget::Signifier { word: p.signifier.clone() },
        None => ActionTarget::Signifier { word: report.utterance.clone() },
    };
    ActionDecision {
        kind: ActionKind::Elaborate,
        target,
        shape: ActionShape::Sentence,
        narrative_subject: NarrativeSubject::World,
        anchor_words,
        reasoning,
    }
}

/// Estrae la classe da una descrizione del tipo
/// "il parlante apre un atto comunicativo della classe \"saluto\"".
fn extract_class_from_phatic_description(s: &str) -> Option<String> {
    // Cerca ciò che è tra le virgolette doppie.
    let mut chars = s.chars();
    let mut found_open = false;
    let mut acc = String::new();
    while let Some(c) = chars.next() {
        if c == '"' {
            if found_open { return Some(acc); }
            found_open = true;
        } else if found_open {
            acc.push(c);
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::comprehension_report::*;
    use crate::topology::speaker_profile::SpeakerProfile;

    fn empty_report(utterance: &str, kind: &str) -> ComprehensionReport {
        ComprehensionReport {
            utterance: utterance.to_string(),
            speech_act: SpeechAct {
                kind: kind.to_string(),
                subject: "Speaker".to_string(),
                description: "test".to_string(),
                addressee: "UI-r1".to_string(),
                implicit_expectation: "test".to_string(),
            },
            symbolic_positions: vec![],
            gaps: vec![],
            inferences: vec![],
            self_relevance: vec![],
            closes_prior_gap: None,
        }
    }

    #[test]
    fn question_to_self_yields_answer_open_question_with_self() {
        let mut r = empty_report("chi sei?", "interrogazione");
        r.symbolic_positions.push(SignifierPosition {
            signifier: "chi".to_string(),
            opposes: vec![],
            serves_in: vec!["identità".to_string()],
            points_to: vec![],
        });
        r.symbolic_positions.push(SignifierPosition {
            signifier: "essere".to_string(),
            opposes: vec![],
            serves_in: vec!["entità".to_string()],
            points_to: vec![],
        });
        let sp = SpeakerProfile::new();
        let dec = decide_action(&r, &sp);
        assert_eq!(dec.kind, ActionKind::AnswerOpenQuestion);
        assert_eq!(dec.narrative_subject, NarrativeSubject::Self_);
    }

    #[test]
    fn phatic_act_yields_phatic_return_word() {
        let r = empty_report("ciao", "atto-fatico");
        let mut r2 = r;
        r2.speech_act.description =
            "il parlante apre un atto comunicativo della classe \"saluto\"".to_string();
        let sp = SpeakerProfile::new();
        let dec = decide_action(&r2, &sp);
        assert_eq!(dec.kind, ActionKind::PhaticReturn);
        assert_eq!(dec.shape, ActionShape::Word);
        if let ActionTarget::PhaticClass { class } = &dec.target {
            assert_eq!(class, "saluto");
        } else {
            panic!("expected PhaticClass target");
        }
    }

    #[test]
    fn claim_with_gap_yields_invite_to_articulate() {
        let mut r = empty_report("ho paura", "posizionamento");
        r.gaps.push(SignifierGap {
            missing: "oggetto".to_string(),
            from: "paura".to_string(),
            relation: "Requires".to_string(),
            context: Some("emozione".to_string()),
            description: "vuoto".to_string(),
        });
        let mut sp = SpeakerProfile::new();
        sp.turn_count = 1;
        sp.self_facts.push(crate::topology::speaker_profile::SpokenFact {
            kind: crate::topology::speaker_profile::FactKind::Feeling,
            predicate: "paura".to_string(),
            turn: 1,
            raw_input: "ho paura".to_string(),
        });
        let dec = decide_action(&r, &sp);
        assert_eq!(dec.kind, ActionKind::InviteToArticulate);
        assert_eq!(dec.shape, ActionShape::Question);
        assert!(dec.anchor_words.contains(&"cosa".to_string()));
        assert!(dec.anchor_words.contains(&"oggetto".to_string()));
    }

    #[test]
    fn claim_without_gap_yields_recognize() {
        let r = empty_report("io sono felice", "posizionamento");
        let mut sp = SpeakerProfile::new();
        sp.turn_count = 1;
        sp.self_facts.push(crate::topology::speaker_profile::SpokenFact {
            kind: crate::topology::speaker_profile::FactKind::Feeling,
            predicate: "felice".to_string(),
            turn: 1,
            raw_input: "io sono felice".to_string(),
        });
        let dec = decide_action(&r, &sp);
        assert_eq!(dec.kind, ActionKind::RecognizeClaim);
    }

    #[test]
    fn assertion_yields_elaborate() {
        let r = empty_report("il sole è caldo", "asserzione");
        let sp = SpeakerProfile::new();
        let dec = decide_action(&r, &sp);
        assert_eq!(dec.kind, ActionKind::Elaborate);
        assert_eq!(dec.shape, ActionShape::Sentence);
    }

    #[test]
    fn extract_class_works() {
        let s = "il parlante apre un atto comunicativo della classe \"saluto\"";
        assert_eq!(extract_class_from_phatic_description(s), Some("saluto".to_string()));
    }

    // ─── Phase 79: closure perception è ora un percetto, non un dispatch ──
    //
    // Prima (Phase 78): la closure innescava un if/then in decide_action
    // che ritornava RecognizeClaim con anchors=[trigger, closing_word].
    // Ora: la closure è un percetto ("chiusura") seminato nel campo del
    // kg_proc; il pattern "riconoscimento" emerge per risonanza nel
    // pattern_matcher, e render_riconoscimento legge trigger/closing_word
    // direttamente da `report.closes_prior_gap`. Qui verifichiamo solo che
    // decide_action annoti la percezione nel reasoning per trasparenza.

    #[test]
    fn closure_annota_percezione_nel_reasoning() {
        let mut r = empty_report("del buio", "asserzione");
        r.closes_prior_gap = Some(crate::topology::comprehension_report::PriorGapClosure {
            trigger: "paura".to_string(),
            role: "oggetto".to_string(),
            closing_word: "buio".to_string(),
            opened_at_turn: 1,
        });
        let mut sp = SpeakerProfile::new();
        sp.turn_count = 2;
        let dec = decide_action(&r, &sp);
        // Il reasoning deve esplicitare la closure (così il log mostra
        // perché il pattern di riconoscimento poi vince per risonanza).
        assert!(dec.reasoning.iter().any(|s| s.contains("turno 1")),
            "il reasoning non cita il turno di apertura: {:?}", dec.reasoning);
        assert!(dec.reasoning.iter().any(|s| s.contains("chiusura") || s.contains("risonanza")),
            "il reasoning non cita il percetto/risonanza: {:?}", dec.reasoning);
    }

    #[test]
    fn closure_non_forza_piu_recognize_claim_via_dispatch() {
        // Phase 79: senza dispatch if/then, "del buio" classificato come
        // asserzione cade nel default Elaborate (non c'è SpeakerClaim, non
        // c'è interrogazione, non c'è atto-fatico). Il pattern di
        // riconoscimento emergerà dal pattern_matcher per risonanza.
        let mut r = empty_report("del buio", "asserzione");
        r.closes_prior_gap = Some(crate::topology::comprehension_report::PriorGapClosure {
            trigger: "paura".to_string(),
            role: "oggetto".to_string(),
            closing_word: "buio".to_string(),
            opened_at_turn: 1,
        });
        let sp = SpeakerProfile::new();
        let dec = decide_action(&r, &sp);
        // decide_action non dispatcha: kind = Elaborate (fall-through default).
        assert_eq!(dec.kind, ActionKind::Elaborate);
        // L'architettura: chi sceglie il pattern è il pattern_matcher per
        // risonanza, vedi `kg_proc_field::tests::closure_dal_report_attiva_chiusura_e_riconoscimento`.
    }
}
