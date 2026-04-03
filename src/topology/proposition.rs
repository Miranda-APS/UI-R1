/// Proposizioni topologiche — il pensiero strutturato che la grammatica esprime.
///
/// Una proposizione è una relazione semantica (Soggetto + Relazione + Oggetto)
/// che emerge dall'incrocio tra campo attivo e Knowledge Graph.
/// NON è parsing dell'input: è lettura del campo post-propagazione.
///
/// Phase 49: proposizioni dirette (1-hop KG).
/// Phase 51: proposizioni multi-hop (2-hop KG) — sillogismi.
///   Esempio: "il sole è caldo" → sole CAUSES calore, calore SIMILAR_TO caldo
///   → proposizione inferita "sole CAUSES caldo" (via "calore", 2 hop).

use std::collections::{HashMap, HashSet};
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;
use crate::topology::word_topology::WordTopology;
use crate::topology::lexicon::Lexicon;

// ═══════════════════════════════════════════════════════════════════════════
// Costanti
// ═══════════════════════════════════════════════════════════════════════════

/// Decay di forza per ogni hop aggiuntivo
const HOP_DECAY: f64 = 0.6;

/// Max parole attive da considerare per multi-hop (evita esplosione combinatoria)
const MULTI_HOP_TOP_N: usize = 15;

/// Peso della relazione nella forza della proposizione.
/// Relazioni forti (causali, tassonomiche) producono proposizioni più informative
/// di quelle deboli (SIMILAR_TO) che sono mera vicinanza lessicale.
fn relation_weight(rel: RelationType) -> f64 {
    match rel {
        RelationType::Causes => 1.0,         // Molto informativo: "X genera Y"
        RelationType::Implies => 0.95,       // Logico: "X implica Y"
        RelationType::IsA => 0.9,            // Tassonomico: "X è un Y"
        RelationType::Does => 0.9,           // Azione: "X fa Y"
        RelationType::Has => 0.85,           // Attributo: "X ha Y"
        RelationType::Enables => 0.85,       // Abilitazione: "X abilita Y"
        RelationType::Requires => 0.85,      // Prerequisito: "X richiede Y"
        RelationType::TransformsInto => 0.85,// Trasformazione: "X diventa Y"
        RelationType::Expresses => 0.8,      // Espressione: "X esprime Y"
        RelationType::UsedFor => 0.8,        // Funzione: "X serve per Y"
        RelationType::PartOf => 0.8,         // Composizione: "X è parte di Y"
        RelationType::Symbolizes => 0.75,    // Simbolico: "X simboleggia Y"
        RelationType::ContextOf => 0.7,      // Contesto: "X contesto di Y"
        RelationType::Excludes => 0.7,       // Esclusione: più forte di opposto
        RelationType::OppositeOf => 0.7,     // Contrasto: informativo ma non costruttivo
        RelationType::Coexists => 0.6,       // Coesistenza: complementarietà
        RelationType::Equivalent => 0.5,     // Equivalenza: come SimilarTo ma forte
        RelationType::SimilarTo => 0.4,      // Debolissimo: vicinanza lessicale generica
        // Fenomenologiche
        RelationType::FeelsAs => 1.2,        // Intima risonanza
        RelationType::WondersAbout => 1.1,   // Tensione esplorativa
        RelationType::RemembersAs => 1.1,    // Memoria emotiva
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════════════════

/// Relazione semantica nella proposizione.
#[derive(Debug, Clone, PartialEq)]
pub enum PropRelation {
    /// X è Y (predicazione identitaria / copula)
    IsA,
    /// X ha Y (attributo/proprietà)
    Has,
    /// X fa Y (azione)
    Does,
    /// X causa Y (causalità)
    Causes,
    /// X è parte di Y
    PartOf,
    /// X è usato per Y
    UsedFor,
    /// X è simile a Y
    SimilarTo,
    /// X è opposto a Y
    OppositeOf,
    /// X abilita Y
    Enables,
    /// X richiede Y
    Requires,
    /// X diventa Y
    TransformsInto,
    /// X esprime Y
    Expresses,
    /// X simboleggia Y
    Symbolizes,
    /// X contesto di Y
    ContextOf,
    /// X implica Y
    Implies,
    /// X equivale a Y
    Equivalent,
    /// X esclude Y
    Excludes,
    /// X coesiste con Y
    Coexists,
    /// X si sente come Y (fenomenologico)
    FeelsAs,
    /// X si interroga su Y (esplorativo)
    WondersAbout,
    /// X ricorda Y come Z (memoria emotiva)
    RemembersAs,
    /// Prossimità topologica (nessun arco KG esplicito, ma vicini nel campo 8D)
    FieldProximity,
}

impl PropRelation {
    fn from_relation_type(rt: RelationType) -> Self {
        match rt {
            RelationType::IsA => Self::IsA,
            RelationType::Has => Self::Has,
            RelationType::Does => Self::Does,
            RelationType::Causes => Self::Causes,
            RelationType::PartOf => Self::PartOf,
            RelationType::UsedFor => Self::UsedFor,
            RelationType::SimilarTo => Self::SimilarTo,
            RelationType::OppositeOf => Self::OppositeOf,
            RelationType::Enables => Self::Enables,
            RelationType::Requires => Self::Requires,
            RelationType::TransformsInto => Self::TransformsInto,
            RelationType::Expresses => Self::Expresses,
            RelationType::Symbolizes => Self::Symbolizes,
            RelationType::ContextOf => Self::ContextOf,
            RelationType::Implies => Self::Implies,
            RelationType::Equivalent => Self::Equivalent,
            RelationType::Excludes => Self::Excludes,
            RelationType::Coexists => Self::Coexists,
            RelationType::FeelsAs => Self::FeelsAs,
            RelationType::WondersAbout => Self::WondersAbout,
            RelationType::RemembersAs => Self::RemembersAs,
        }
    }

    /// Copula da usare nella generazione grammaticale.
    /// Restituisce la parola-ponte tra soggetto e oggetto.
    pub fn copula(&self) -> &'static str {
        match self {
            Self::IsA => "è",
            Self::Has => "ha",
            Self::Does => "",
            Self::Causes => "genera",
            Self::PartOf => "è parte di",
            Self::UsedFor => "serve per",
            Self::SimilarTo => "è come",
            Self::OppositeOf => "non è",
            Self::Enables => "abilita",
            Self::Requires => "richiede",
            Self::TransformsInto => "diventa",
            Self::Expresses => "esprime",
            Self::Symbolizes => "simboleggia",
            Self::ContextOf => "è contesto di",
            Self::Implies => "implica",
            Self::Equivalent => "equivale a",
            Self::Excludes => "esclude",
            Self::Coexists => "coesiste con",
            Self::FieldProximity => "e",
            Self::FeelsAs => "si sente come",
            Self::WondersAbout => "si interroga su",
            Self::RemembersAs => "ricorda come",
        }
    }
}

/// Una proposizione topologica: il "pensiero" che la grammatica esprimerà.
#[derive(Debug, Clone)]
pub struct Proposition {
    /// Soggetto
    pub subject: String,
    /// Relazione semantica
    pub relation: PropRelation,
    /// Oggetto/predicato
    pub object: String,
    /// Forza della proposizione [0.0, 1.0]
    pub strength: f64,
    /// Confidence combinata degli archi KG
    pub kg_confidence: f32,
    /// Numero di hop (1=diretto, 2=inferito)
    pub hops: u8,
    /// Parola intermedia per proposizioni multi-hop
    pub via: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Multi-hop: ricerca cammini a 2 hop nel KG
// ═══════════════════════════════════════════════════════════════════════════

/// Un cammino a 2 hop nel KG: from →[rel1]→ mid →[rel2]→ to
#[derive(Debug)]
struct TwoHopPath {
    rel1: RelationType,
    intermediate: String,
    rel2: RelationType,
    conf1: f32,
    conf2: f32,
}

impl TwoHopPath {
    /// Relazione inferita: IS_A e SIMILAR_TO sono "trasparenti" (passano la relazione successiva).
    /// Altre relazioni sono "dominanti" (il soggetto le eredita).
    ///
    /// Esempi:
    ///   cane IS_A animale, animale HAS zampe → cane HAS zampe (IS_A trasparente)
    ///   sole CAUSES calore, calore SIMILAR_TO caldo → sole CAUSES caldo (CAUSES dominante)
    ///   cane SIMILAR_TO lupo, lupo DOES ululare → cane DOES ululare (SIMILAR_TO trasparente)
    fn inferred_relation(&self) -> RelationType {
        match self.rel1 {
            RelationType::IsA | RelationType::SimilarTo => self.rel2,
            _ => self.rel1,
        }
    }

    fn combined_confidence(&self) -> f32 {
        self.conf1 * self.conf2
    }
}

/// Trova cammini a 2 hop tra `from` e `to` nel KG.
///
/// Pattern 1 (forward chain): from →[rel1]→ mid →[rel2]→ to
///   Esempio: sole →CAUSES→ calore →SIMILAR_TO→ caldo
///
/// Pattern 2 (shared target): from →[rel1]→ mid ←[rel2]← to
///   Solo se rel2 ∈ {SIMILAR_TO, IS_A} (to ≈ mid).
///   Esempio: sole →CAUSES→ calore ←SIMILAR_TO← caldo
fn find_two_hop_paths(kg: &KnowledgeGraph, from: &str, to: &str) -> Vec<TwoHopPath> {
    let mut paths = Vec::new();

    let out_from = kg.all_outgoing(from);

    // Pattern 1: from→mid→to
    // Cerco intermediari che sono target di `from` E source di edge verso `to`
    let in_to = kg.all_incoming(to);
    let in_to_map: HashMap<&str, (RelationType, f32)> = in_to.iter()
        .map(|(r, s, c)| (*s, (*r, *c)))
        .collect();

    for (rel1, mid, conf1) in &out_from {
        if *mid == to { continue; } // sarebbe diretto, non 2-hop
        if let Some((rel2, conf2)) = in_to_map.get(mid) {
            paths.push(TwoHopPath {
                rel1: *rel1,
                intermediate: mid.to_string(),
                rel2: *rel2,
                conf1: *conf1,
                conf2: *conf2,
            });
        }
    }

    // Pattern 2: from→mid←to (entrambi puntano a mid)
    // Solo se to→mid è SIMILAR_TO o IS_A (to ≈ mid)
    let out_to = kg.all_outgoing(to);
    let out_to_map: HashMap<&str, (RelationType, f32)> = out_to.iter()
        .map(|(r, t, c)| (*t, (*r, *c)))
        .collect();

    for (rel1, mid, conf1) in &out_from {
        if *mid == to { continue; }
        if let Some((rel2, conf2)) = out_to_map.get(mid) {
            // to→mid: ha senso solo se rel2 è "trasparente" (to ≈ mid)
            if matches!(rel2, RelationType::SimilarTo | RelationType::IsA) {
                // Evita duplicati con Pattern 1
                let already = paths.iter().any(|p| p.intermediate == *mid);
                if !already {
                    paths.push(TwoHopPath {
                        rel1: *rel1,
                        intermediate: mid.to_string(),
                        rel2: *rel2,
                        conf1: *conf1,
                        conf2: *conf2,
                    });
                }
            }
        }
    }

    // Ordina per confidence combinata decrescente
    paths.sort_by(|a, b| {
        let ca = a.combined_confidence();
        let cb = b.combined_confidence();
        cb.partial_cmp(&ca).unwrap_or(std::cmp::Ordering::Equal)
    });
    paths.truncate(3);
    paths
}

// ═══════════════════════════════════════════════════════════════════════════
// Estrazione proposizioni dal campo
// ═══════════════════════════════════════════════════════════════════════════

/// Estrae proposizioni strutturate dall'incrocio tra parole attive e KG.
///
/// Algoritmo:
/// 1. Prende le top N parole attive dal campo
/// 2. Per ogni coppia, cerca archi KG diretti (1-hop)
/// 3. Per coppie senza archi diretti, cerca cammini 2-hop (sillogismi)
/// 4. Forza = sqrt(act_i × act_j) × confidence × hop_decay^(hops-1)
/// 5. Ordina per forza e restituisce le top K
///
/// `echo_exclude`: parole dell'input da NON usare come soggetto (anti-eco).
pub fn extract_propositions(
    word_topology: &WordTopology,
    kg: &KnowledgeGraph,
    _lexicon: &Lexicon,
    echo_exclude: &[String],
    max_propositions: usize,
) -> Vec<Proposition> {
    // Top parole attive dal campo
    let active = word_topology.most_active(30);
    if active.len() < 2 {
        return vec![];
    }

    let mut propositions: Vec<Proposition> = Vec::new();

    // ── Fase 1: proposizioni dirette (1-hop) ────────────────────────────────

    // Hub damping per proposizioni: soggetti con troppi archi KG producono
    // proposizioni generiche ("essere è fondamento"). Penalizziamo.
    let hub_penalty = |word: &str| -> f64 {
        let deg = kg.total_degree(word);
        if deg > 200 { 0.3 } // mega-hub: penalizza fortemente
        else if deg > 50 { 0.6 }
        else { 1.0 }
    };

    for i in 0..active.len() {
        let word_a = &active[i].word;
        let act_a = active[i].activation;

        if word_a.len() < 3 { continue; }

        // Archi uscenti da word_a
        let hp_a = hub_penalty(word_a);
        for (rel, target, confidence) in kg.all_outgoing(word_a) {
            if let Some(entry_b) = active.iter().find(|e| e.word == target) {
                let act_b = entry_b.activation;
                let rw = relation_weight(rel);
                let strength = (act_a as f64 * act_b as f64).sqrt() * confidence as f64 * hp_a * rw;
                if strength < 0.01 { continue; }

                let echo_penalty = if echo_exclude.iter().any(|e| e == word_a) { 0.5 } else { 1.0 };

                propositions.push(Proposition {
                    subject: word_a.clone(),
                    relation: PropRelation::from_relation_type(rel),
                    object: target.to_string(),
                    strength: strength * echo_penalty,
                    kg_confidence: confidence,
                    hops: 1,
                    via: None,
                });
            }
        }

        // Archi entranti
        for (rel, source, confidence) in kg.all_incoming(word_a) {
            if let Some(entry_s) = active.iter().find(|e| e.word == source) {
                let already = propositions.iter().any(|p| p.subject == source && p.object == *word_a);
                if already { continue; }

                let act_s = entry_s.activation;
                let hp_s = hub_penalty(source);
                let rw = relation_weight(rel);
                let strength = (act_s as f64 * act_a as f64).sqrt() * confidence as f64 * hp_s * rw;
                if strength < 0.01 { continue; }

                let echo_penalty = if echo_exclude.iter().any(|e| e == source) { 0.5 } else { 1.0 };

                propositions.push(Proposition {
                    subject: source.to_string(),
                    relation: PropRelation::from_relation_type(rel),
                    object: word_a.clone(),
                    strength: strength * echo_penalty,
                    kg_confidence: confidence,
                    hops: 1,
                    via: None,
                });
            }
        }
    }

    // ── Fase 2: proposizioni multi-hop (2-hop, sillogismi) ──────────────────

    // Coppie già coperte da proposizioni dirette
    let direct_pairs: HashSet<(String, String)> = propositions.iter()
        .map(|p| (p.subject.clone(), p.object.clone()))
        .collect();

    let n = active.len().min(MULTI_HOP_TOP_N);
    for i in 0..n {
        let word_a = &active[i].word;
        let act_a = active[i].activation;
        if word_a.len() < 3 { continue; }

        for j in 0..n {
            if i == j { continue; }
            let word_b = &active[j].word;
            if word_b.len() < 3 { continue; }

            // Skip se già coperto da proposizione diretta
            if direct_pairs.contains(&(word_a.clone(), word_b.clone())) { continue; }

            let paths = find_two_hop_paths(kg, word_a, word_b);
            if let Some(best) = paths.first() {
                let act_b = active[j].activation;
                let combined_conf = best.combined_confidence();
                let inferred_rel = best.inferred_relation();
                let hp_a = hub_penalty(word_a);
                let rw = relation_weight(inferred_rel);
                let strength = (act_a as f64 * act_b as f64).sqrt()
                    * combined_conf as f64
                    * HOP_DECAY
                    * hp_a
                    * rw;
                if strength < 0.01 { continue; }

                // Per multi-hop l'echo penalty è ridotto: l'inferenza è contenuto nuovo,
                // non ripetizione dell'input. Penalità 0.8 invece di 0.5.
                let echo_penalty = if echo_exclude.iter().any(|e| e == word_a) { 0.8 } else { 1.0 };

                propositions.push(Proposition {
                    subject: word_a.clone(),
                    relation: PropRelation::from_relation_type(inferred_rel),
                    object: word_b.clone(),
                    strength: strength * echo_penalty,
                    kg_confidence: combined_conf,
                    hops: 2,
                    via: Some(best.intermediate.clone()),
                });
            }
        }
    }

    // Deduplica: per ogni coppia (subject, object) tieni solo la proposizione più forte
    propositions.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
    let mut seen: HashSet<(String, String)> = HashSet::new();
    propositions.retain(|p| seen.insert((p.subject.clone(), p.object.clone())));
    propositions.truncate(max_propositions);
    propositions
}

// ═══════════════════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_kg_with_edges() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        kg.add("sole", RelationType::Causes, "calore");
        kg.add("sole", RelationType::IsA, "stella");
        kg.add("cane", RelationType::IsA, "animale");
        kg.add("cane", RelationType::Does, "abbaiare");
        kg.add("fuoco", RelationType::Causes, "calore");
        kg
    }

    fn setup_kg_multihop() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        // Catena: sole CAUSES calore, calore SIMILAR_TO caldo
        kg.add("sole", RelationType::Causes, "calore");
        // Ereditarietà: cane IS_A animale, animale HAS zampe
        kg.add("cane", RelationType::IsA, "animale");
        kg.add("animale", RelationType::Has, "zampe");
        // Ereditarietà azione: cane IS_A animale, animale DOES mangiare
        kg.add("animale", RelationType::Does, "mangiare");
        // Similarità bidirezionale: caldo SIMILAR_TO calore (Pattern 2)
        kg.add("caldo", RelationType::SimilarTo, "calore");
        kg
    }

    fn setup_topology_with_active(words: &[(&str, f64)]) -> (WordTopology, Lexicon) {
        let lexicon = Lexicon::bootstrap();
        let mut topo = WordTopology::new();
        for (word, _) in words {
            topo.add_word(word);
        }
        for (word, strength) in words {
            topo.activate_word(word, *strength);
        }
        (topo, lexicon)
    }

    #[test]
    fn test_extract_propositions_basic() {
        let kg = setup_kg_with_edges();
        let (mut topo, lex) = setup_topology_with_active(&[
            ("sole", 0.8),
            ("calore", 0.6),
        ]);
        topo.add_word("stella");

        let props = extract_propositions(&topo, &kg, &lex, &[], 5);

        assert!(!props.is_empty(), "deve trovare almeno una proposizione");
        let sole_calore = props.iter().find(|p| p.subject == "sole" && p.object == "calore");
        assert!(sole_calore.is_some(), "deve trovare sole CAUSES calore. Props: {:?}", props);
        assert_eq!(sole_calore.unwrap().relation, PropRelation::Causes);
        assert_eq!(sole_calore.unwrap().hops, 1);
        assert!(sole_calore.unwrap().via.is_none());
    }

    #[test]
    fn test_extract_no_active_overlap() {
        let kg = setup_kg_with_edges();
        let (topo, lex) = setup_topology_with_active(&[
            ("tempo", 0.8),
            ("spazio", 0.6),
        ]);

        let props = extract_propositions(&topo, &kg, &lex, &[], 5);
        assert!(props.is_empty() || props.iter().all(|p| p.relation == PropRelation::FieldProximity),
            "senza archi KG tra parole attive, nessuna proposizione diretta");
    }

    #[test]
    fn test_echo_penalty() {
        let kg = setup_kg_with_edges();
        let (mut topo, lex) = setup_topology_with_active(&[
            ("sole", 0.8),
            ("calore", 0.6),
        ]);
        topo.add_word("stella");

        let props_no_echo = extract_propositions(&topo, &kg, &lex, &[], 5);
        let props_with_echo = extract_propositions(
            &topo, &kg, &lex,
            &["sole".to_string()],
            5,
        );

        if let (Some(p1), Some(p2)) = (
            props_no_echo.iter().find(|p| p.subject == "sole"),
            props_with_echo.iter().find(|p| p.subject == "sole"),
        ) {
            assert!(p2.strength < p1.strength,
                "echo penalty deve ridurre la forza: {} vs {}", p2.strength, p1.strength);
        }
    }

    #[test]
    fn test_proposition_strength_ordering() {
        let kg = setup_kg_with_edges();
        let (mut topo, lex) = setup_topology_with_active(&[
            ("sole", 0.9),
            ("calore", 0.8),
            ("cane", 0.3),
            ("animale", 0.2),
        ]);
        topo.add_word("stella");
        topo.add_word("abbaiare");
        topo.add_word("fuoco");

        let props = extract_propositions(&topo, &kg, &lex, &[], 10);

        if props.len() >= 2 {
            assert!(props[0].strength >= props[1].strength,
                "proposizioni devono essere ordinate per forza");
        }
    }

    // ── Test multi-hop (Phase 51) ───────────────────────────────────────────

    #[test]
    fn test_multihop_causes_similar() {
        // Sillogismo: sole CAUSES calore, caldo SIMILAR_TO calore → sole CAUSES caldo
        let kg = setup_kg_multihop();
        let (topo, lex) = setup_topology_with_active(&[
            ("sole", 0.8),
            ("caldo", 0.6),
        ]);

        let props = extract_propositions(&topo, &kg, &lex, &[], 5);

        let sole_caldo = props.iter().find(|p| p.subject == "sole" && p.object == "caldo");
        assert!(sole_caldo.is_some(),
            "deve inferire sole→caldo via 2-hop. Props: {:?}", props);
        let p = sole_caldo.unwrap();
        assert_eq!(p.relation, PropRelation::Causes, "relazione inferita deve essere CAUSES");
        assert_eq!(p.hops, 2, "deve essere 2-hop");
        assert_eq!(p.via.as_deref(), Some("calore"), "via deve essere 'calore'");
    }

    #[test]
    fn test_multihop_isa_inheritance() {
        // Sillogismo: cane IS_A animale, animale HAS zampe → cane HAS zampe
        let kg = setup_kg_multihop();
        let (topo, lex) = setup_topology_with_active(&[
            ("cane", 0.8),
            ("zampe", 0.6),
        ]);

        let props = extract_propositions(&topo, &kg, &lex, &[], 5);

        let cane_zampe = props.iter().find(|p| p.subject == "cane" && p.object == "zampe");
        assert!(cane_zampe.is_some(),
            "deve inferire cane HAS zampe via IS_A animale. Props: {:?}", props);
        let p = cane_zampe.unwrap();
        assert_eq!(p.relation, PropRelation::Has, "IS_A trasparente: relazione ereditata = HAS");
        assert_eq!(p.hops, 2);
        assert_eq!(p.via.as_deref(), Some("animale"));
    }

    #[test]
    fn test_multihop_isa_does_inheritance() {
        // Sillogismo: cane IS_A animale, animale DOES mangiare → cane DOES mangiare
        let kg = setup_kg_multihop();
        let (topo, lex) = setup_topology_with_active(&[
            ("cane", 0.8),
            ("mangiare", 0.5),
        ]);

        let props = extract_propositions(&topo, &kg, &lex, &[], 5);

        let cane_mangiare = props.iter().find(|p| p.subject == "cane" && p.object == "mangiare");
        assert!(cane_mangiare.is_some(),
            "deve inferire cane DOES mangiare via IS_A animale. Props: {:?}", props);
        assert_eq!(cane_mangiare.unwrap().relation, PropRelation::Does);
    }

    #[test]
    fn test_multihop_weaker_than_direct() {
        // Una proposizione 2-hop deve avere forza minore di una 1-hop equivalente
        let kg = setup_kg_multihop();
        let (topo, lex) = setup_topology_with_active(&[
            ("sole", 0.8),
            ("calore", 0.7),  // target diretto
            ("caldo", 0.7),   // target 2-hop (stessa attivazione di calore per confronto equo)
        ]);

        let props = extract_propositions(&topo, &kg, &lex, &[], 10);

        let direct = props.iter().find(|p| p.subject == "sole" && p.object == "calore");
        let inferred = props.iter().find(|p| p.subject == "sole" && p.object == "caldo");

        if let (Some(d), Some(inf)) = (direct, inferred) {
            assert!(d.strength > inf.strength,
                "proposizione diretta ({:.3}) deve essere più forte della inferita ({:.3})",
                d.strength, inf.strength);
        }
    }

    #[test]
    fn test_find_two_hop_paths_basic() {
        let kg = setup_kg_multihop();
        let paths = find_two_hop_paths(&kg, "sole", "caldo");
        assert!(!paths.is_empty(), "deve trovare cammino sole→calore→caldo");
        assert_eq!(paths[0].intermediate, "calore");
    }

    #[test]
    fn test_find_two_hop_no_path() {
        let kg = setup_kg_multihop();
        let paths = find_two_hop_paths(&kg, "sole", "zampe");
        assert!(paths.is_empty(), "nessun cammino 2-hop tra sole e zampe");
    }
}
