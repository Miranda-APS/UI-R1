//! Campo di attivazione del KG procedurale — il substrato della selezione
//! dei pattern espressivi per risonanza.
//!
//! Phase 79. Sostituisce il dispatch `pattern_name_for(decision)` (che era
//! una mappa hardcoded `ActionKind → pattern_name`) con un meccanismo
//! emergente: i percetti del `ComprehensionReport` seminano l'attivazione
//! di concetti nel kg_proc tramite triple `<percetto> Causes <concetto>`,
//! e il pattern che vince è quello la cui pertinenza (`UsedFor X via Y`)
//! risuona di più con il campo seminato.
//!
//! ## Pipeline
//!
//! ```text
//! ComprehensionReport            ─→  seed_from_comprehension(report)
//!   .closes_prior_gap                  attiva percetto "chiusura"
//!   .speech_act.kind = "saluto"        attiva percetto "saluto"
//!   .speech_act.kind = "interrogazione" attiva percetto "domanda"
//!   .gaps non vuoti + posizionamento   attiva percetto "apertura"
//!   ...                                                │
//!                                                      ▼
//! per ogni percetto: kg_proc.query("Causes")      KgProcActivation
//!   chiusura Causes restituire (0.7)              { restituire: 0.7,
//!   chiusura Causes posizione   (0.5)               posizione:  0.5,
//!   chiusura Causes completamento (0.4)             completamento: 0.4 }
//!                                                      │
//!                                                      ▼
//!                                         select_pattern_by_resonance
//!   per ogni nodo IsA pattern:
//!     score = Σ activation[target] su tutti gli UsedFor X via Y
//!   riconoscimento UsedFor restituire via=posizione → 0.7+0.5 = 1.2 ← vince
//!   ricambio       UsedFor restituire via=saluto    → 0.7+0   = 0.7
//!   articolazione  UsedFor chiedere via=vuoto       → 0+0     = 0
//!                                                      │
//!                                                      ▼
//!                                         "riconoscimento"
//!
//! Il bridge percetto→nodo è in Rust (legge proprietà tipizzate del report);
//! le mappe percetto→concetto sono in dati (kg_proc, triple Causes).
//! ```
//!
//! ## Cosa NON fa questo modulo
//!
//! - **Nessuna propagazione di attivazione nel kg_proc**: il seeding è
//!   diretto sui target di `Causes`. Aggiungere propagazione (vicini di
//!   vicini) è un'estensione futura se serve. Per i 10 pattern attuali e
//!   i 9 percetti, l'attivazione diretta è sufficiente.
//! - **Nessun decay**: il campo è ricostruito ad ogni `receive()` dal
//!   ComprehensionReport del turno corrente. La continuità tra turni vive
//!   negli organi (SpeakerProfile, SelfProfile), non in questo campo.
//! - **Nessun dispatch su pattern_name**: i pattern sono solo nodi del
//!   kg_proc. Aggiungere un nuovo pattern = aggiungere triple, mai Rust.

use std::collections::HashMap;

use crate::topology::comprehension_report::ComprehensionReport;
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

/// Attivazione corrente di concetti nel KG procedurale.
/// Le chiavi sono nomi di nodi (parole italiane atomiche).
#[derive(Debug, Clone, Default)]
pub struct KgProcActivation {
    activations: HashMap<String, f64>,
}

impl KgProcActivation {
    pub fn new() -> Self { Self::default() }

    /// Lettura diretta. 0.0 se il nodo non è attivo.
    pub fn get(&self, node: &str) -> f64 {
        self.activations.get(node).copied().unwrap_or(0.0)
    }

    /// Iter per debug/log.
    pub fn iter(&self) -> impl Iterator<Item = (&str, f64)> {
        self.activations.iter().map(|(k, v)| (k.as_str(), *v))
    }

    /// Aggiunge attivazione (additivo, capped a 1.0).
    pub fn add(&mut self, node: &str, weight: f64) {
        let entry = self.activations.entry(node.to_string()).or_insert(0.0);
        *entry = (*entry + weight).min(1.0);
    }

    /// Semina un percetto: per ogni `<percetto> Causes <target>` nel kg_proc,
    /// somma `confidence × intensity` all'attivazione del target.
    /// L'intensity è il "gain" del percetto (1.0 di default; più basso se
    /// il percetto è solo parzialmente presente).
    pub fn seed_percetto(&mut self, percetto: &str, intensity: f64, kg_proc: &KnowledgeGraph) {
        for (target, conf, _via) in kg_proc.query_objects_with_via(percetto, RelationType::Causes) {
            let weight = (conf as f64) * intensity;
            self.add(target, weight);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Bridge: ComprehensionReport → percetti seeded nel kg_proc
// ═══════════════════════════════════════════════════════════════════════════

/// Mappa le proprietà tipizzate del ComprehensionReport sui percetti del
/// kg_proc. È il bridge tipato I/O — il PRIMO punto in cui il report
/// (Rust struct) entra nel campo del kg_proc (dati). Le mappe percetto→concetto
/// vivono come triple `Causes` nel kg_proc, non qui.
///
/// Le combinazioni sono inferenziali, non comportamentali: descrivono CHE TIPO
/// di evento conversazionale stiamo percependo, non COSA fare in risposta.
pub fn seed_from_comprehension(
    activation: &mut KgProcActivation,
    report: &ComprehensionReport,
    kg_proc: &KnowledgeGraph,
) {
    let kind = report.speech_act.kind.as_str();

    // Closure di un vuoto aperto da UI-r1: il parlante ha portato l'oggetto
    // mancante. Percetto: "chiusura".
    if report.closes_prior_gap.is_some() {
        activation.seed_percetto("chiusura", 1.0, kg_proc);
    }

    // Posizionamento del parlante (claim di sentimento) con vuoto aperto:
    // il parlante ha portato un'emozione senza il suo oggetto. Percetto:
    // "apertura" — chiama articolazione.
    if kind == "posizionamento"
        && !report.gaps.is_empty()
        && report.closes_prior_gap.is_none()
    {
        activation.seed_percetto("apertura", 1.0, kg_proc);
    }

    // Posizionamento senza vuoto e senza closure: claim completo del parlante
    // → riconoscimento simbolico. Percetto: "posizione".
    if kind == "posizionamento"
        && report.gaps.is_empty()
        && report.closes_prior_gap.is_none()
    {
        activation.seed_percetto("posizione", 1.0, kg_proc);
    }

    // Interrogazione rivolta a UI-r1 stessa (chi sei? come stai? cosa pensi?).
    // Percetti: "domanda" (rispondere) + "identità" come boost diretto.
    // Per domande sul mondo (subject senza Entity) NON seminiamo: nessun
    // pattern espressivo è appropriato — il caller cadrà nei nuclei semantici.
    //
    // `derive_speech_act` codifica la self-reference così:
    //   subject = "Speaker (su Entity)" se la domanda riguarda UI-r1
    //   subject = "Speaker"             se la domanda riguarda il mondo
    //
    // In aggiunta: se l'utterance contiene un verbo coniugato in 2a singolare
    // ("sei", "stai", "pensi", "vivi"), l'interrogazione si rivolge a UI-r1
    // (Lei è il "tu"). Questo cattura "chi sei?" / "come stai?" anche quando
    // la self-reference non è stata rilevata a monte da `facts.self_referenced`.
    if kind == "interrogazione" {
        let asks_self = report.speech_act.subject.contains("Entity")
            || report.speech_act.subject == "Self_"
            || report.speech_act.description.contains("identità")
            || utterance_has_second_singular(&report.utterance);
        if asks_self {
            activation.seed_percetto("domanda", 1.0, kg_proc);
            activation.add("identità", 0.5);
        }
    }

    // Atto fatico: saluto/congedo. Percetto: "saluto".
    if kind == "saluto" || kind == "atto-fatico" || kind == "fatico" {
        activation.seed_percetto("saluto", 1.0, kg_proc);
    }

    // Asserzione sul mondo (no claim al parlante). Percetto: "affermazione".
    if kind == "asserzione" {
        activation.seed_percetto("affermazione", 1.0, kg_proc);
    }

    // Denominazione: "mi chiamo X". Percetto: "introduzione".
    if kind == "denominazione" || kind == "presentazione" {
        activation.seed_percetto("introduzione", 1.0, kg_proc);
    }
}

/// Detecta se l'utterance contiene un verbo coniugato in 2a singolare (presente).
/// Usa `grammar::lemmatize` per riconoscere le forme verbali. È euristico:
/// non distingue 2sg da forme nominali ambigue (es. "vivi" può essere verbo o
/// nome). In pratica, in contesto interrogativo italiano, la presenza di un
/// 2sg è un segnale forte di destinatario = TU (= UI-r1 nell'enunciato).
fn utterance_has_second_singular(utterance: &str) -> bool {
    use crate::topology::grammar::{lemmatize, Person};
    let normalized = utterance.to_lowercase();
    for token in normalized.split(|c: char| !c.is_alphabetic()) {
        if token.is_empty() { continue; }
        if let Some(result) = lemmatize(token) {
            if matches!(result.person, Person::Second) {
                return true;
            }
        }
    }
    false
}

// ═══════════════════════════════════════════════════════════════════════════
// Selezione del pattern per risonanza
// ═══════════════════════════════════════════════════════════════════════════

/// Punteggio di risonanza di un pattern: somma delle attivazioni dei suoi
/// `UsedFor X via Y` — sia X (l'azione) sia Y (il ruolo). Un pattern "vince"
/// quando il campo del kg_proc ha attivato sia la sua azione caratteristica
/// sia il suo ruolo specifico.
pub fn pattern_score(
    pattern_name: &str,
    activation: &KgProcActivation,
    kg_proc: &KnowledgeGraph,
) -> f64 {
    let mut score = 0.0;
    for (target, _conf, via) in kg_proc.query_objects_with_via(pattern_name, RelationType::UsedFor) {
        score += activation.get(target);
        if let Some(v) = via {
            score += activation.get(v);
        }
    }
    score
}

/// Seleziona il pattern dal kg_proc che meglio risuona col campo attivo.
/// Restituisce `None` se nessun pattern ha risonanza > 0 (campo sotto
/// soglia di significatività — il caller cadrà nel fallback).
pub fn select_pattern_by_resonance(
    activation: &KgProcActivation,
    kg_proc: &KnowledgeGraph,
) -> Option<String> {
    let patterns = kg_proc.query_subjects("pattern", RelationType::IsA);
    let mut best: Option<(String, f64)> = None;
    for p in patterns {
        let score = pattern_score(p, activation, kg_proc);
        if score > 0.0 && best.as_ref().map_or(true, |(_, b)| score > *b) {
            best = Some((p.to_string(), score));
        }
    }
    best.map(|(name, _)| name)
}

/// Variante diagnostica: ritorna il punteggio di TUTTI i pattern ordinati,
/// utile per debug e introspezione (es. log "DECISIONE" in dialogue_educator).
pub fn pattern_scores(
    activation: &KgProcActivation,
    kg_proc: &KnowledgeGraph,
) -> Vec<(String, f64)> {
    let patterns = kg_proc.query_subjects("pattern", RelationType::IsA);
    let mut scores: Vec<(String, f64)> = patterns
        .into_iter()
        .map(|p| (p.to_string(), pattern_score(p, activation, kg_proc)))
        .collect();
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scores
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::comprehension_report::{ComprehensionReport, SpeechAct, SignifierGap, PriorGapClosure};
    use crate::topology::relation::{TypedEdge, EdgeSource};

    /// Mini KG procedurale con 3 percetti e 3 pattern, per test atomici.
    fn build_minimal_kg() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();

        // Pattern: articolazione UsedFor chiedere via=vuoto
        kg.add("articolazione", RelationType::IsA, "pattern");
        kg.add_edge(TypedEdge {
            subject: "articolazione".to_string(),
            relation: RelationType::UsedFor,
            object: "chiedere".to_string(),
            confidence: 0.95,
            source: EdgeSource::Curated,
            via: Some("vuoto".to_string()),
        });

        // Pattern: riconoscimento UsedFor restituire via=posizione
        kg.add("riconoscimento", RelationType::IsA, "pattern");
        kg.add_edge(TypedEdge {
            subject: "riconoscimento".to_string(),
            relation: RelationType::UsedFor,
            object: "restituire".to_string(),
            confidence: 0.95,
            source: EdgeSource::Curated,
            via: Some("posizione".to_string()),
        });

        // Pattern: ricambio UsedFor restituire via=saluto
        kg.add("ricambio", RelationType::IsA, "pattern");
        kg.add_edge(TypedEdge {
            subject: "ricambio".to_string(),
            relation: RelationType::UsedFor,
            object: "restituire".to_string(),
            confidence: 0.95,
            source: EdgeSource::Curated,
            via: Some("saluto".to_string()),
        });

        // Percetti
        kg.add("apertura", RelationType::IsA, "percetto");
        kg.add_edge(TypedEdge {
            subject: "apertura".to_string(),
            relation: RelationType::Causes,
            object: "chiedere".to_string(),
            confidence: 0.7,
            source: EdgeSource::Curated,
            via: None,
        });
        kg.add_edge(TypedEdge {
            subject: "apertura".to_string(),
            relation: RelationType::Causes,
            object: "vuoto".to_string(),
            confidence: 0.5,
            source: EdgeSource::Curated,
            via: None,
        });

        kg.add("chiusura", RelationType::IsA, "percetto");
        kg.add_edge(TypedEdge {
            subject: "chiusura".to_string(),
            relation: RelationType::Causes,
            object: "restituire".to_string(),
            confidence: 0.7,
            source: EdgeSource::Curated,
            via: None,
        });
        kg.add_edge(TypedEdge {
            subject: "chiusura".to_string(),
            relation: RelationType::Causes,
            object: "posizione".to_string(),
            confidence: 0.5,
            source: EdgeSource::Curated,
            via: None,
        });

        kg.add("saluto", RelationType::IsA, "percetto");
        kg.add_edge(TypedEdge {
            subject: "saluto".to_string(),
            relation: RelationType::Causes,
            object: "restituire".to_string(),
            confidence: 0.7,
            source: EdgeSource::Curated,
            via: None,
        });
        kg.add_edge(TypedEdge {
            subject: "saluto".to_string(),
            relation: RelationType::Causes,
            object: "saluto".to_string(),
            confidence: 0.6,
            source: EdgeSource::Curated,
            via: None,
        });

        kg
    }

    fn report(kind: &str) -> ComprehensionReport {
        ComprehensionReport {
            utterance: "test".to_string(),
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
    fn apertura_attiva_articolazione() {
        let kg = build_minimal_kg();
        let mut act = KgProcActivation::new();
        act.seed_percetto("apertura", 1.0, &kg);
        // chiedere ≈ 0.7, vuoto ≈ 0.5 → articolazione = 1.2
        assert!((pattern_score("articolazione", &act, &kg) - 1.2).abs() < 0.01);
        assert_eq!(select_pattern_by_resonance(&act, &kg).as_deref(), Some("articolazione"));
    }

    #[test]
    fn chiusura_attiva_riconoscimento_non_ricambio() {
        // chiusura attiva restituire(0.7) + posizione(0.5)
        // riconoscimento (UsedFor restituire via=posizione) = 0.7 + 0.5 = 1.2
        // ricambio       (UsedFor restituire via=saluto)    = 0.7 + 0   = 0.7
        let kg = build_minimal_kg();
        let mut act = KgProcActivation::new();
        act.seed_percetto("chiusura", 1.0, &kg);
        assert_eq!(select_pattern_by_resonance(&act, &kg).as_deref(), Some("riconoscimento"));
        assert!(pattern_score("riconoscimento", &act, &kg) > pattern_score("ricambio", &act, &kg));
    }

    #[test]
    fn saluto_attiva_ricambio_non_riconoscimento() {
        // saluto attiva restituire(0.7) + saluto(0.6)
        // ricambio       (UsedFor restituire via=saluto)    = 0.7 + 0.6 = 1.3
        // riconoscimento (UsedFor restituire via=posizione) = 0.7 + 0   = 0.7
        let kg = build_minimal_kg();
        let mut act = KgProcActivation::new();
        act.seed_percetto("saluto", 1.0, &kg);
        assert_eq!(select_pattern_by_resonance(&act, &kg).as_deref(), Some("ricambio"));
    }

    #[test]
    fn campo_vuoto_nessun_pattern() {
        let kg = build_minimal_kg();
        let act = KgProcActivation::new();
        assert!(select_pattern_by_resonance(&act, &kg).is_none());
    }

    #[test]
    fn closure_dal_report_attiva_chiusura_e_riconoscimento() {
        // Bridge end-to-end: ComprehensionReport con closure → seed → riconoscimento.
        let kg = build_minimal_kg();
        let mut r = report("posizionamento");
        r.closes_prior_gap = Some(PriorGapClosure {
            trigger: "paura".to_string(),
            role: "oggetto".to_string(),
            closing_word: "buio".to_string(),
            opened_at_turn: 1,
        });
        let mut act = KgProcActivation::new();
        seed_from_comprehension(&mut act, &r, &kg);
        assert_eq!(select_pattern_by_resonance(&act, &kg).as_deref(), Some("riconoscimento"));
    }

    #[test]
    fn posizionamento_con_gap_attiva_apertura_e_articolazione() {
        // Bridge end-to-end: posizionamento + gap aperto → seed apertura → articolazione.
        let kg = build_minimal_kg();
        let mut r = report("posizionamento");
        r.gaps.push(SignifierGap {
            missing: "oggetto".to_string(),
            from: "paura".to_string(),
            relation: "Requires".to_string(),
            context: None,
            description: "test".to_string(),
        });
        let mut act = KgProcActivation::new();
        seed_from_comprehension(&mut act, &r, &kg);
        assert_eq!(select_pattern_by_resonance(&act, &kg).as_deref(), Some("articolazione"));
    }

    #[test]
    fn saluto_dal_report() {
        let kg = build_minimal_kg();
        let r = report("saluto");
        let mut act = KgProcActivation::new();
        seed_from_comprehension(&mut act, &r, &kg);
        assert_eq!(select_pattern_by_resonance(&act, &kg).as_deref(), Some("ricambio"));
    }
}
