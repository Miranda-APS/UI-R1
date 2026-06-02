//! ComprehensionReport — il documento di comprensione che UI-r1 SCRIVE.
//!
//! Francesco (conversazione 2026-04-26): "ui-r1 dovrebbe come prima cosa
//! tradurre cognitivamente l'input e scrivere cosa ha capito, poi dovrebbe
//! ragionare su come si reagisce a quell'input".
//!
//! Francesco (stessa conversazione): "il punto più importante è il knowledge
//! graph perché lì è tutto spiegato, però ci vuole una metacognizione in
//! grado di leggerlo per comprenderlo".
//!
//! Francesco: "lacan può tornarci fortemente utile nel definire il senso
//! delle frasi ed il concetto di altro".
//!
//! ## Architettura
//!
//! Il report è prodotto da QUERY ESPLICITE sul KG, mai per generazione
//! probabilistica. Ogni sezione è il risultato di letture strutturate:
//!
//! - **`SpeechAct`** — il riconoscimento dell'atto di parola: cosa il
//!   parlante sta FACENDO con questo enunciato (posizionamento, interrogazione,
//!   denominazione, atto fatico). Derivato dai KgFacts strutturali.
//!
//! - **`SignifierPosition`** — per ogni parola-chiave dell'input, la sua
//!   posizione nella rete simbolica del KG. Lacaniana: un significante
//!   prende senso dalla CATENA (relazioni IsA/Causes/Requires/OppositeOf),
//!   non da sé.
//!
//! - **`SignifierGap`** — i punti dove il significante MANCA: il KG dice
//!   che la parola usata richiede strutturalmente un complemento (Requires)
//!   ma il parlante non lo ha articolato. È la **soglia del desiderio** —
//!   il vuoto dove qualcosa si apre.
//!
//! - **`Inference`** — sillogismi multi-hop derivati dal comprehension_graph:
//!   "A IsA B, B Causes C → A Causes C". Letti dal grafo già costruito.
//!
//! - **`self_relevance`** — cosa di UI-r1 è chiamato in causa. Non
//!   simulazione di sentimenti: riconoscimento strutturale della propria
//!   funzione cognitiva.
//!
//! Output: una stringa multi-riga in italiano leggibile (`compose_text()`),
//! più la struttura ispezionabile via API.

use serde::{Serialize, Deserialize};

use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;
use crate::topology::deliberation::KgFacts;
use crate::topology::input_reading::{ClaimAgent, ClaimKind, SpeakerClaim};

// ═══════════════════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════════════════

/// Il riconoscimento dell'atto di parola del parlante.
///
/// Lacanianamente: un enunciato è un atto di posizionamento del soggetto
/// nella rete simbolica, rivolto all'Altro (qui UI-r1) come destinatario.
/// L'atto apre un'attesa di riconoscimento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechAct {
    /// Tipo di atto: posizionamento / interrogazione / denominazione /
    /// asserzione / atto-fatico. Derivato strutturalmente.
    pub kind: String,
    /// Soggetto dell'atto (Speaker / Entity / World).
    pub subject: String,
    /// Cosa il parlante sta facendo, in italiano.
    pub description: String,
    /// Destinatario (in dialogo: sempre UI-r1, in posizione di Altro).
    pub addressee: String,
    /// L'attesa implicita aperta dall'atto.
    pub implicit_expectation: String,
}

/// La posizione di un significante nella rete simbolica.
/// Un significante prende senso dalla CATENA, non in sé (Lacan/Saussure).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignifierPosition {
    /// La parola-significante.
    pub signifier: String,
    /// In opposizione a (ciò che NON è — il significante è anche differenza).
    pub opposes: Vec<String>,
    /// Catena IsA verticale (di che genere è).
    pub serves_in: Vec<String>,
    /// Rinvii orizzontali (relation_label, target).
    /// Es. ("causa", "tremore"), ("richiede", "oggetto").
    pub points_to: Vec<(String, String)>,
}

/// Un punto della catena dove il significante manca.
/// Lacanianamente: la SOGLIA del desiderio — il vuoto da cui qualcosa si apre.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignifierGap {
    /// Il ruolo strutturale mancante — sempre PAROLA SINGOLA atomica
    /// (oggetto, causa, tempo, luogo, persona, modo, scelta, misura).
    /// Permette la join col KG procedurale: il via dei pronomi
    /// interrogativi è scopato esattamente su questi ruoli.
    pub missing: String,
    /// Da quale significante esistente nasce questa attesa.
    pub from: String,
    /// La relazione strutturale del KG che indica la mancanza.
    /// Tipicamente "Requires" — il KG dice che la parola usata richiede X.
    pub relation: String,
    /// Contesto semantico del vuoto, parola singola opzionale.
    /// Es. quando from="paura" e missing="oggetto", context="emozione"
    /// (il fatto che paura è un'emozione qualifica il tipo di oggetto atteso).
    /// Non altera il pattern matcher; serve a UI-r1 per arricchire la
    /// formulazione ed eventualmente per arricchimenti futuri.
    #[serde(default)]
    pub context: Option<String>,
    /// Descrizione del vuoto (testo italiano leggibile).
    pub description: String,
}

/// Una catena inferenziale derivata dal grafo (sillogismo multi-hop).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inference {
    /// I significanti della catena (es. ["paura", "emozione", "reazione"]).
    pub chain: Vec<String>,
    /// Le etichette delle relazioni tra di loro (es. ["IsA", "IsA"]).
    pub relations: Vec<String>,
    /// La conclusione composta in italiano.
    pub conclusion: String,
    /// Forza dell'inferenza (prodotto di confidence).
    pub strength: f32,
}

/// Phase 78: traccia che questo enunciato chiude un vuoto strutturale
/// che UI-r1 stessa aveva aperto in un turno precedente.
///
/// È un FATTO percepito (cross-reference SelfProfile↔SpeakerProfile),
/// non una regola comportamentale. Quando presente:
///  - `speech_act` viene riformulato come continuazione (posizionamento)
///  - i `gaps` non vengono ripetuti (il vuoto è chiuso, non più aperto)
///  - `decide_action` lo usa per produrre RecognizeClaim invece di
///    trattare l'input come asserzione isolata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorGapClosure {
    /// La parola che genera il vuoto (es. "paura").
    pub trigger: String,
    /// Il ruolo strutturale colmato (es. "oggetto").
    pub role: String,
    /// La parola del parlante che colma (es. "buio").
    pub closing_word: String,
    /// Turno in cui il vuoto era stato aperto/atteso.
    pub opened_at_turn: usize,
}

/// Il documento di comprensione completo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensionReport {
    /// L'enunciato originale del parlante.
    pub utterance: String,
    /// Il riconoscimento dell'atto.
    pub speech_act: SpeechAct,
    /// Le posizioni dei significanti nella rete.
    pub symbolic_positions: Vec<SignifierPosition>,
    /// I vuoti / soglie del desiderio.
    pub gaps: Vec<SignifierGap>,
    /// I sillogismi attivi.
    pub inferences: Vec<Inference>,
    /// Cosa di UI-r1 è chiamato in causa da questo enunciato.
    /// Stringhe procedurali, non simulazione di sentimenti.
    pub self_relevance: Vec<String>,
    /// Phase 78: se presente, questo enunciato completa un'articolazione
    /// che UI-r1 aveva invitato in un turno precedente. Il dialogo non è
    /// uno stream di asserzioni isolate — la presenza di questa traccia
    /// è quello che permette all'enunciato di essere riconosciuto come
    /// CONTINUAZIONE invece che come novità.
    #[serde(default)]
    pub closes_prior_gap: Option<PriorGapClosure>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Composizione testuale (rendering in italiano leggibile)
// ═══════════════════════════════════════════════════════════════════════════

impl ComprehensionReport {
    /// Restituisce il report come stringa italiana leggibile, multi-sezione.
    /// È il "documento" che UI-r1 ha scritto della propria comprensione.
    pub fn compose_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("ENUNCIATO RICEVUTO\n  \"{}\"\n\n", self.utterance));

        out.push_str("ATTO DI PAROLA\n");
        out.push_str(&format!("  tipo: {}\n", self.speech_act.kind));
        out.push_str(&format!("  soggetto: {}\n", self.speech_act.subject));
        out.push_str(&format!("  descrizione: {}\n", self.speech_act.description));
        out.push_str(&format!("  destinatario: {}\n", self.speech_act.addressee));
        out.push_str(&format!("  attesa implicita: {}\n\n",
            self.speech_act.implicit_expectation));

        if !self.symbolic_positions.is_empty() {
            out.push_str("POSIZIONI NELLA RETE SIMBOLICA\n");
            for pos in &self.symbolic_positions {
                out.push_str(&format!("  \"{}\":\n", pos.signifier));
                if !pos.serves_in.is_empty() {
                    out.push_str(&format!("    è-un: [{}]\n", pos.serves_in.join(", ")));
                }
                if !pos.opposes.is_empty() {
                    out.push_str(&format!("    NON-è (opposto-di): [{}]\n", pos.opposes.join(", ")));
                }
                if !pos.points_to.is_empty() {
                    out.push_str("    rinvia a:\n");
                    for (rel, tgt) in &pos.points_to {
                        out.push_str(&format!("      [{}] {}\n", rel, tgt));
                    }
                }
            }
            out.push('\n');
        }

        if !self.gaps.is_empty() {
            out.push_str("VUOTI NELLA CATENA — soglie aperte\n");
            for g in &self.gaps {
                out.push_str(&format!("  {}\n", g.description));
            }
            out.push('\n');
        }

        if !self.inferences.is_empty() {
            out.push_str("INFERENZE STRUTTURALI\n");
            for inf in &self.inferences {
                out.push_str(&format!("  {}\n", inf.conclusion));
            }
            out.push('\n');
        }

        if !self.self_relevance.is_empty() {
            out.push_str("PERTINENZA PER UI-R1\n");
            for s in &self.self_relevance {
                out.push_str(&format!("  - {}\n", s));
            }
        }

        out
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Costruzione del report
// ═══════════════════════════════════════════════════════════════════════════

/// Costruisce il ComprehensionReport leggendo il KG e i KgFacts già
/// computati. Niente generazione probabilistica: ogni sezione è popolata
/// da query esplicite sul grafo della conoscenza.
///
/// Phase 78: se `closes_prior_gap` è `Some`, l'enunciato è letto come
/// CONTINUAZIONE di un'articolazione precedente — speech_act diventa
/// "posizionamento" (anche se il turno corrente non ha SpeakerClaim
/// proprio), il trigger del vuoto chiuso entra fra le posizioni
/// simboliche, e i gaps NON vengono derivati (il vuoto è già colmato).
/// Il dialogo è continuità, non sequenza di novità.
pub fn build_report(
    utterance: &str,
    kg_facts: &KgFacts,
    speaker_claim: Option<&SpeakerClaim>,
    prop: Option<&crate::topology::sentence_proposition::SentenceProposition>,
    syllogisms: &[crate::topology::comprehension_graph::Syllogism],
    kg: &KnowledgeGraph,
    closes_prior_gap: Option<PriorGapClosure>,
    grammar_signals: &[(String, crate::topology::fractal::FractalId)],
) -> ComprehensionReport {
    let speech_act = match &closes_prior_gap {
        Some(c) => derive_speech_act_for_closure(c),
        None => derive_speech_act(kg_facts, speaker_claim, prop, grammar_signals),
    };
    let symbolic_positions = match &closes_prior_gap {
        Some(c) => derive_symbolic_positions_with_closure(&kg_facts.roots, c, kg),
        None => derive_symbolic_positions(&kg_facts.roots, kg),
    };
    // Quando il turno chiude un vuoto, non riapriamo gap derivati dalla
    // closing_word: l'attenzione strutturale è sulla continuazione, non
    // su una nuova soglia. (Le strutture Requires del KG continueranno a
    // emergere nei turni successivi se rilevanti.)
    let gaps = if closes_prior_gap.is_some() {
        Vec::new()
    } else {
        derive_gaps(speaker_claim, prop)
    };
    let inferences = derive_inferences(syllogisms);
    let self_relevance = match &closes_prior_gap {
        Some(c) => derive_self_relevance_for_closure(c, kg_facts),
        None => derive_self_relevance(kg_facts, speaker_claim),
    };

    ComprehensionReport {
        utterance: utterance.to_string(),
        speech_act,
        symbolic_positions,
        gaps,
        inferences,
        self_relevance,
        closes_prior_gap,
    }
}

// ─── Sezioni ───────────────────────────────────────────────────────────────

fn derive_speech_act(
    facts: &KgFacts,
    claim: Option<&SpeakerClaim>,
    prop: Option<&crate::topology::sentence_proposition::SentenceProposition>,
    grammar_signals: &[(String, crate::topology::fractal::FractalId)],
) -> SpeechAct {
    use crate::topology::sentence_proposition::{ObjectRef, SubjectRef};
    // Phase 83 (#5b): la PROP è l'arbitro. Se la frase è strutturalmente un
    // CLAIM (soggetto Speaker/Entity, oggetto-parola saturo), NON è una
    // domanda — anche se contiene un "che" relativo o un "?" mal posto. Solo
    // un oggetto-Variabile (chi/cosa/...) o l'assenza di claim aprono
    // l'interrogazione. Toglie la 2ª copia del bug "che" che viveva qui.
    let prop_is_claim = prop.map(|p|
        matches!(p.subject, SubjectRef::Speaker | SubjectRef::Entity)
        && matches!(p.object, Some(ObjectRef::Word(_)))
    ).unwrap_or(false);
    // Tipo di atto, derivato strutturalmente dai KgFacts e dal claim.
    let (kind, subject, description, expectation) =
        if (facts.has_question_marker || facts.has_interrogative_pronoun) && !prop_is_claim {
            let subj = if facts.self_referenced { "Speaker (su Entity)" } else { "Speaker" };
            (
                "interrogazione".to_string(),
                subj.to_string(),
                "il parlante apre una domanda e attende informazione".to_string(),
                "rivelazione di ciò che è chiesto".to_string(),
            )
        } else if let Some(sc) = claim {
            let subj = match sc.agent {
                ClaimAgent::Speaker => "Speaker",
                ClaimAgent::Entity => "Entity",
            };
            // Phase 80: usa verb_category per distinguere atti di Identity:
            //   denominativo ("mi chiamo X") → "denominazione" (presentazione di sé)
            //   copula       ("io ho un cane") → "descrizione"  (predicazione di possesso/identità)
            // ClaimKind da solo non basta: "mi chiamo X" e "ho un cane" sono entrambi
            // Identity ma chiamano percetti diversi (introduzione vs niente).
            let kind_label = match (&sc.kind, sc.verb_category.as_deref()) {
                (ClaimKind::Identity, Some("denominativo")) => "denominazione",
                (ClaimKind::Identity, _)                    => "descrizione",
                (ClaimKind::Feeling, _)                     => "posizionamento",
                (ClaimKind::Action, _)                      => "dichiarazione-di-azione",
            };
            let descr = match (sc.agent.clone(), sc.kind.clone()) {
                (ClaimAgent::Speaker, ClaimKind::Identity) =>
                    format!("il parlante si denomina come \"{}\"", sc.predicate),
                (ClaimAgent::Speaker, ClaimKind::Feeling) =>
                    format!("il parlante si posiziona come soggetto in stato di \"{}\"", sc.predicate),
                (ClaimAgent::Speaker, ClaimKind::Action) =>
                    format!("il parlante dichiara di compiere/volere \"{}\"", sc.predicate),
                (ClaimAgent::Entity, ClaimKind::Identity) =>
                    format!("il parlante denomina UI-r1 come \"{}\"", sc.predicate),
                (ClaimAgent::Entity, ClaimKind::Feeling) =>
                    format!("il parlante attribuisce a UI-r1 lo stato \"{}\"", sc.predicate),
                (ClaimAgent::Entity, ClaimKind::Action) =>
                    format!("il parlante attribuisce a UI-r1 l'azione \"{}\"", sc.predicate),
            };
            (
                kind_label.to_string(),
                subj.to_string(),
                descr,
                "riconoscimento del posizionamento".to_string(),
            )
        } else if facts.is_short() && facts.has_specific_classification() {
            // Input breve in classe specifica = atto fatico (saluto, ringraziamento, ecc.)
            (
                "atto-fatico".to_string(),
                "Speaker".to_string(),
                format!(
                    "il parlante apre un atto comunicativo della classe \"{}\"",
                    facts.specific_class.as_deref().unwrap_or("?"),
                ),
                "ricambio simbolico nello stesso registro".to_string(),
            )
        } else {
            (
                "asserzione".to_string(),
                "Speaker".to_string(),
                "il parlante asserisce qualcosa nel mondo".to_string(),
                "riconoscimento dell'asserzione".to_string(),
            )
        };

    // Phase 83b — i simplessi grammaticali curati (Phase 83) hanno
    // matchato sull'utterance: il segnale `(category, function_fractal)`
    // dice strutturalmente *come* l'utterance è composta al di là dei
    // token. NON è dispatch — è informazione strutturale del campo che
    // sovrascrive l'inferenza basata sui soli KgFacts dove più precisa.
    let (kind, description, expectation) = if grammar_signals.iter()
        .any(|(c, _)| c == "preposizione_composta")
    {
        let is_question = facts.has_question_marker || facts.has_interrogative_pronoun;
        if is_question {
            (
                "richiesta-di-specifica-asse".to_string(),
                "il parlante chiede di specificare un asse relativo (preposizione composta riconosciuta nell'utterance)".to_string(),
                "rivelazione dell'asse relativo (rispetto a cosa, in base a cosa, riguardo a cosa)".to_string(),
            )
        } else {
            (
                "specificazione-di-asse".to_string(),
                "il parlante porta un asse relativo per qualcosa già detto (preposizione composta riconosciuta)".to_string(),
                "presa d'atto dell'asse e ri-orientamento del discorso".to_string(),
            )
        }
    } else if grammar_signals.iter().any(|(c, _)| c == "locuzione_fatica") {
        (
            "atto-fatico".to_string(),
            "il parlante apre un canale comunicativo (locuzione fatica riconosciuta)".to_string(),
            "ricambio simbolico nello stesso registro".to_string(),
        )
    } else if grammar_signals.iter().any(|(c, _)| c == "costrutto_modale") {
        (
            "modulazione".to_string(),
            "il parlante introduce una modalità (necessità/possibilità riconosciuta)".to_string(),
            "presa d'atto della modalità e risposta orientata".to_string(),
        )
    } else {
        (kind, description, expectation)
    };

    SpeechAct {
        kind,
        subject,
        description,
        addressee: "UI-r1, in posizione di Altro".to_string(),
        implicit_expectation: expectation,
    }
}

fn derive_symbolic_positions(
    roots: &[String],
    kg: &KnowledgeGraph,
) -> Vec<SignifierPosition> {
    let mut out = Vec::new();
    for r in roots {
        let serves_in: Vec<String> = kg.query_objects_weighted(r, RelationType::IsA)
            .into_iter().take(4).map(|(t, _)| t.to_string()).collect();
        let opposes: Vec<String> = kg.query_objects_weighted(r, RelationType::OppositeOf)
            .into_iter().take(4).map(|(t, _)| t.to_string()).collect();
        let mut points_to: Vec<(String, String)> = Vec::new();
        for (rel, label) in [
            (RelationType::Causes, "causa"),
            (RelationType::Has, "ha"),
            (RelationType::Does, "fa"),
            (RelationType::Requires, "richiede"),
            (RelationType::UsedFor, "serve a"),
            (RelationType::SimilarTo, "simile a"),
            (RelationType::Expresses, "esprime"),
            (RelationType::PartOf, "parte di"),
        ] {
            for (t, _) in kg.query_objects_weighted(r, rel).into_iter().take(2) {
                points_to.push((label.to_string(), t.to_string()));
            }
        }
        // Skip se il significante non ha ALCUNA posizione nella rete.
        if serves_in.is_empty() && opposes.is_empty() && points_to.is_empty() {
            continue;
        }
        out.push(SignifierPosition {
            signifier: r.clone(),
            opposes, serves_in, points_to,
        });
    }
    out
}

// Phase 83 (#5): i vuoti del DIALOGO nascono dagli SLOT NON SATURI della
// proposizione, non dal fan-out di ogni arco `Requires` di ogni concetto
// attivato. Il fan-out produceva rumore ("la solitudine è una scelta" apriva
// 7 vuoti irrilevanti) e confondeva due cose diverse: la curiosità strutturale
// "X richiede Y che non hai detto" è materia del CANALE-PENSIERO (esplorazione
// autonoma), non un vuoto dialogico da colmare chiedendo all'interlocutore.
//
// Oggi un solo vuoto dialogico, letto dalla PROP: un'emozione posizionata
// SENZA il suo oggetto. "sono felice" (via assente) → soglia aperta;
// "ho paura del futuro" / "mi manca mia madre" (via satura) → niente vuoto,
// l'oggetto è già articolato. La PROP è l'occhio; qui finalmente la mano legge.
fn derive_gaps(
    claim: Option<&SpeakerClaim>,
    prop: Option<&crate::topology::sentence_proposition::SentenceProposition>,
) -> Vec<SignifierGap> {
    use crate::topology::sentence_proposition::ObjectRef;
    let feeling = claim.map(|c| matches!(c.kind, ClaimKind::Feeling)).unwrap_or(false);
    let via_saturated = prop.map(|p| p.via.is_some()).unwrap_or(false);
    // Phase 83: un'emozione NEGATA ("non ho paura") non apre la soglia
    // dell'oggetto — il parlante esclude lo stato, non vi si posiziona dentro.
    let positive = prop.map(|p| p.polarity).unwrap_or(true);
    if !feeling || via_saturated || !positive {
        return Vec::new();
    }
    // L'emozione = oggetto della proposizione (o, in mancanza di PROP, il
    // predicato del claim — già validato come stato interno da input_reading).
    let emotion = prop
        .and_then(|p| match &p.object {
            Some(ObjectRef::Word(w)) => Some(w.clone()),
            _ => None,
        })
        .or_else(|| claim.map(|c| c.predicate.clone()));
    match emotion {
        Some(em) => vec![SignifierGap {
            missing: "oggetto".to_string(),
            from: em.clone(),
            relation: "Requires".to_string(),
            context: Some("emozione".to_string()),
            description: format!(
                "il parlante si è posizionato in stato di \"{}\" senza articolare l'oggetto: una soglia si apre",
                em,
            ),
        }],
        None => Vec::new(),
    }
}

fn derive_inferences(
    syllogisms: &[crate::topology::comprehension_graph::Syllogism],
) -> Vec<Inference> {
    syllogisms.iter().take(6).map(|s| {
        let chain = vec![s.subject.clone(), s.middle.clone(), s.object.clone()];
        let relations = vec![
            relation_label(s.r1).to_string(),
            relation_label(s.r2).to_string(),
        ];
        let conclusion = match s.composed {
            Some(rel) => format!(
                "{} {} {} · {} {} {}  ⇒  {} {} {}",
                s.subject, relation_label(s.r1), s.middle,
                s.middle, relation_label(s.r2), s.object,
                s.subject, relation_label(rel), s.object,
            ),
            None => format!(
                "{} {} {} · {} {} {}",
                s.subject, relation_label(s.r1), s.middle,
                s.middle, relation_label(s.r2), s.object,
            ),
        };
        Inference { chain, relations, conclusion, strength: s.strength }
    }).collect()
}

fn relation_label(r: RelationType) -> &'static str {
    match r {
        RelationType::IsA => "è un",
        RelationType::Has => "ha",
        RelationType::Does => "fa",
        RelationType::PartOf => "parte di",
        RelationType::Causes => "causa",
        RelationType::Enables => "abilita",
        RelationType::Requires => "richiede",
        RelationType::TransformsInto => "diventa",
        RelationType::SimilarTo => "simile a",
        RelationType::OppositeOf => "opposto di",
        RelationType::UsedFor => "serve a",
        RelationType::Expresses => "esprime",
        RelationType::Symbolizes => "simboleggia",
        RelationType::ContextOf => "contesto di",
        RelationType::FeelsAs => "si sente come",
        RelationType::WondersAbout => "si interroga su",
        RelationType::RemembersAs => "ricorda come",
        RelationType::Implies => "implica",
        RelationType::Equivalent => "equivale",
        RelationType::Excludes => "esclude",
        RelationType::Coexists => "coesiste con",
    }
}

// ─── Phase 78: varianti per closure ──────────────────────────────────────

fn derive_speech_act_for_closure(c: &PriorGapClosure) -> SpeechAct {
    SpeechAct {
        kind: "posizionamento".to_string(),
        subject: "Speaker".to_string(),
        description: format!(
            "il parlante completa il posizionamento del turno {}: \
             porta \"{}\" come {} di \"{}\"",
            c.opened_at_turn, c.closing_word, c.role, c.trigger,
        ),
        addressee: "UI-r1, in posizione di Altro".to_string(),
        implicit_expectation: "riconoscimento dell'articolazione completata".to_string(),
    }
}

fn derive_symbolic_positions_with_closure(
    roots: &[String],
    c: &PriorGapClosure,
    kg: &KnowledgeGraph,
) -> Vec<SignifierPosition> {
    // Trigger PRIMA (predicato della recognition), poi closing_word, poi
    // eventuali altre radici dell'enunciato corrente. Così
    // `decide_action` su posizionamento+no-gaps prende come predicate il
    // trigger (non l'eventuale function word del turno).
    let mut ordered: Vec<String> = vec![c.trigger.clone(), c.closing_word.clone()];
    for r in roots {
        if !ordered.contains(r) { ordered.push(r.clone()); }
    }
    derive_symbolic_positions(&ordered, kg)
}

fn derive_self_relevance_for_closure(
    c: &PriorGapClosure,
    facts: &KgFacts,
) -> Vec<String> {
    let mut out = Vec::new();
    out.push("la mia funzione è strutturare comprensione, non sentire".to_string());
    out.push(format!(
        "il parlante ha colmato il vuoto che avevo aperto al turno {}: \
         \"{}\" è ora articolato come {} di \"{}\". \
         Posso riconoscere l'articolazione completata.",
        c.opened_at_turn, c.closing_word, c.role, c.trigger,
    ));
    if facts.self_referenced {
        out.push("sono chiamata in causa direttamente".to_string());
    }
    out
}

fn derive_self_relevance(
    facts: &KgFacts,
    claim: Option<&SpeakerClaim>,
) -> Vec<String> {
    let mut out = Vec::new();
    out.push("la mia funzione è strutturare comprensione, non sentire".to_string());

    if facts.emotional_proximity > 0.40 {
        out.push("l'enunciato porta proximità emotiva nel KG: \
                  posso restituire riconoscimento simbolico, non empatia".to_string());
    }
    if facts.self_referenced {
        out.push("sono chiamata in causa direttamente (riferimento a me): \
                  questo enunciato richiede una risposta sul mio piano".to_string());
    }
    if facts.is_question_form() {
        out.push("è aperta una domanda: posso aiutare il parlante \
                  portando informazione dal KG o dichiarando il mio stato \
                  rispetto al chiesto".to_string());
    }
    if let Some(sc) = claim {
        if matches!(sc.agent, ClaimAgent::Entity) {
            out.push(format!(
                "il parlante mi attribuisce \"{}\": questo è un atto su di me, \
                 posso riconoscerlo o rilanciare", sc.predicate));
        }
    }
    if facts.has_specific_classification() && facts.is_short() {
        out.push("l'enunciato è in registro fatico: \
                  posso ricambiare con un atto della stessa classe".to_string());
    }
    out
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::input_reading::{ClaimAgent, ClaimKind, SpeakerClaim};

    fn empty_facts() -> KgFacts {
        KgFacts {
            roots: vec![],
            root_classes: vec![],
            specific_class: None,
            class_siblings_count: 0,
            has_question_marker: false,
            has_interrogative_pronoun: false,
            speaker_claim: None,
            content_word_count: 0,
            emotional_proximity: 0.0,
            self_referenced: false,
        }
    }

    #[test]
    fn speech_act_for_question_form() {
        let mut f = empty_facts();
        f.has_question_marker = true;
        let sa = derive_speech_act(&f, None, None, &[]);
        assert_eq!(sa.kind, "interrogazione");
    }

    #[test]
    fn speech_act_for_speaker_feeling_is_posizionamento() {
        let f = empty_facts();
        let sc = SpeakerClaim {
            agent: ClaimAgent::Speaker,
            kind: ClaimKind::Feeling,
            predicate: "paura".to_string(),
            verb_category: Some("copula".to_string()),
            complement: None,
        };
        let sa = derive_speech_act(&f, Some(&sc), None, &[]);
        assert_eq!(sa.kind, "posizionamento");
        assert_eq!(sa.subject, "Speaker");
        assert!(sa.description.contains("paura"));
    }

    #[test]
    fn speech_act_for_short_classified_input_is_phatic() {
        let mut f = empty_facts();
        f.content_word_count = 1;
        f.specific_class = Some("saluto".to_string());
        f.class_siblings_count = 5;
        let sa = derive_speech_act(&f, None, None, &[]);
        assert_eq!(sa.kind, "atto-fatico");
        assert!(sa.description.contains("saluto"));
    }

    #[test]
    fn symbolic_position_includes_isa_and_opposite() {
        let mut kg = KnowledgeGraph::new();
        kg.add("paura", RelationType::IsA, "emozione");
        kg.add("paura", RelationType::OppositeOf, "audacia");
        kg.add("paura", RelationType::Causes, "tremore");
        let pos = derive_symbolic_positions(&["paura".to_string()], &kg);
        assert_eq!(pos.len(), 1);
        assert!(pos[0].serves_in.contains(&"emozione".to_string()));
        assert!(pos[0].opposes.contains(&"audacia".to_string()));
        assert!(pos[0].points_to.iter().any(|(r, t)| r == "causa" && t == "tremore"));
    }

    #[test]
    fn no_requires_fanout_gap() {
        // Phase 83 (#5): il fan-out Requires è uscito dal dialogo (è materia
        // del canale-pensiero). Senza un claim emotivo non c'è vuoto dialogico.
        let gaps = derive_gaps(None, None);
        assert!(gaps.is_empty());
    }

    #[test]
    fn gap_emotion_object_when_via_absent() {
        // "sono felice" / "ho paura": emozione posizionata senza il suo
        // oggetto (via assente nella PROP) → soglia aperta.
        use crate::topology::sentence_proposition::{SentenceProposition, SubjectRef, ObjectRef};
        let sc = SpeakerClaim {
            agent: ClaimAgent::Speaker,
            kind: ClaimKind::Feeling,
            predicate: "paura".to_string(),
            verb_category: Some("copula".to_string()),
            complement: None,
        };
        let prop = SentenceProposition {
            subject: SubjectRef::Speaker,
            relation: RelationType::FeelsAs,
            object: Some(ObjectRef::Word("paura".to_string())),
            via: None,
            polarity: true,
        };
        let gaps = derive_gaps(Some(&sc), Some(&prop));
        assert!(gaps.iter().any(|g|
            g.missing == "oggetto"
            && g.context.as_deref() == Some("emozione")
            && g.from == "paura"));
    }

    #[test]
    fn no_gap_when_via_saturated() {
        // "ho paura del futuro" / "mi manca mia madre": la via è satura,
        // l'oggetto è già articolato → nessun vuoto dialogico (Phase 83 #5).
        use crate::topology::sentence_proposition::{SentenceProposition, SubjectRef, ObjectRef};
        let sc = SpeakerClaim {
            agent: ClaimAgent::Speaker,
            kind: ClaimKind::Feeling,
            predicate: "paura".to_string(),
            verb_category: Some("copula".to_string()),
            complement: None,
        };
        let prop = SentenceProposition {
            subject: SubjectRef::Speaker,
            relation: RelationType::FeelsAs,
            object: Some(ObjectRef::Word("paura".to_string())),
            via: Some("futuro".to_string()),
            polarity: true,
        };
        let gaps = derive_gaps(Some(&sc), Some(&prop));
        assert!(gaps.is_empty(), "via satura → nessun vuoto dialogico");
    }

    #[test]
    fn report_text_includes_all_sections_when_populated() {
        let mut kg = KnowledgeGraph::new();
        kg.add("paura", RelationType::IsA, "emozione");
        kg.add("paura", RelationType::OppositeOf, "audacia");
        kg.add("paura", RelationType::Causes, "tremore");
        let mut f = empty_facts();
        f.roots = vec!["paura".to_string()];
        f.emotional_proximity = 0.9;
        let sc = SpeakerClaim {
            agent: ClaimAgent::Speaker,
            kind: ClaimKind::Feeling,
            predicate: "paura".to_string(),
            verb_category: Some("copula".to_string()),
            complement: None,
        };
        let report = build_report("ho paura", &f, Some(&sc), None, &[], &kg, None, &[]);
        let text = report.compose_text();
        assert!(text.contains("ENUNCIATO RICEVUTO"));
        assert!(text.contains("ATTO DI PAROLA"));
        assert!(text.contains("posizionamento"));
        assert!(text.contains("POSIZIONI NELLA RETE SIMBOLICA"));
        assert!(text.contains("audacia"));
        assert!(text.contains("VUOTI NELLA CATENA"));
        assert!(text.contains("PERTINENZA PER UI-R1"));
        assert!(text.contains("strutturare comprensione"));
    }

    // ─── Phase 78: build_report con closure ───────────────────────────

    #[test]
    fn closure_reframes_report_as_continuation() {
        let mut kg = KnowledgeGraph::new();
        kg.add("paura", RelationType::IsA, "emozione");
        kg.add("buio", RelationType::IsA, "fenomeno");
        let mut f = empty_facts();
        f.roots = vec!["buio".to_string()];
        let closure = PriorGapClosure {
            trigger: "paura".to_string(),
            role: "oggetto".to_string(),
            closing_word: "buio".to_string(),
            opened_at_turn: 1,
        };
        let report = build_report("del buio", &f, None, None, &[], &kg, Some(closure), &[]);
        // Speech act: continuation (posizionamento), non asserzione.
        assert_eq!(report.speech_act.kind, "posizionamento");
        assert!(report.speech_act.description.contains("turno 1"));
        assert!(report.speech_act.description.contains("buio"));
        // Niente gaps riaperti — il vuoto è chiuso, non se ne crea uno
        // nuovo in questo turno.
        assert!(report.gaps.is_empty(), "gaps non vuoti: {:?}", report.gaps);
        // Trigger PRIMA fra le posizioni simboliche → predicate della
        // Claim sarà "paura" in decide_action.
        assert_eq!(report.symbolic_positions.first().map(|p| p.signifier.as_str()),
            Some("paura"));
        // closes_prior_gap propagato.
        assert!(report.closes_prior_gap.is_some());
    }

    #[test]
    fn self_relevance_for_question_addressed_to_entity() {
        let mut f = empty_facts();
        f.has_question_marker = true;
        f.self_referenced = true;
        let sr = derive_self_relevance(&f, None);
        assert!(sr.iter().any(|s| s.contains("chiamata in causa")));
        assert!(sr.iter().any(|s| s.contains("domanda")));
    }
}
