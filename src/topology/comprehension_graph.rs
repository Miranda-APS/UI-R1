//! ComprehensionGraph — l'esplorazione transitiva dell'input.
//!
//! Francesco (conversazione 2026-04-25): "deve prendere tutte le relazioni
//! del mio input e fare tutti gli hop che servono anche usando i sillogismi
//! per capire esattamente cosa gli viene detto. capire significa sapere come
//! ci si comporta di conseguenza ad un input".
//!
//! Questo modulo costruisce, per ogni input dell'utente, un grafo di
//! esplorazione transitiva nel KG: BFS dai lemmi input, depth ≤ 3, con
//! hub-damping. Lungo l'esplorazione ricava convergenze (concetti raggiunti
//! da più radici) e sillogismi (cammini A→B→C dove la composizione delle
//! relazioni è logicamente significativa).
//!
//! Il grafo è la base per due cose:
//!
//! 1. **Comportamento**: la regione del grafo con maggiore convergenza è
//!    "ciò di cui si sta parlando davvero". Le parole-fratello (sibling)
//!    dell'input sotto le radici di convergenza sono il vocabolario
//!    dal quale UI-r1 può rispondere senza echeggiare l'input.
//!
//! 2. **Visualizzazione**: nodi + archi del grafo sono renderizzati nella
//!    chat amministrativa come SVG, mostrando in tempo reale il
//!    "ragionamento" di UI-r1 mentre cerca di capire.
//!
//! Niente template — solo struttura del KG attraversata con regole chiare.

use std::collections::HashMap;

use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

// ═══════════════════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════════════════

/// Profondità massima dell'esplorazione (numero di hop dal root).
pub const DEFAULT_MAX_DEPTH: u8 = 3;

/// Soglia oltre la quale un concetto è considerato "hub": non viene
/// espanso per evitare che la BFS esploda su nodi saturati (essere, fare, ...).
pub const HUB_DEGREE_THRESHOLD: usize = 200;

/// Confidence minima per traversare un arco. Sotto questa soglia, il segnale
/// è troppo debole per contribuire alla comprensione.
pub const MIN_EDGE_CONFIDENCE: f32 = 0.20;

/// Decadimento per profondità: un arco a depth=2 contribuisce 0.6× rispetto
/// a depth=1. Penalizza esplorazioni lontane.
pub const DEPTH_DECAY: f32 = 0.6;

/// Un concetto raggiunto durante l'esplorazione.
#[derive(Debug, Clone)]
pub struct ConceptNode {
    /// Lemma normalizzato del concetto.
    pub word: String,
    /// Hop minimo da una radice (0 = parola input).
    pub depth: u8,
    /// Quanto il concetto è "supportato" dalla scena: somma delle confidenze
    /// degli archi che lo raggiungono, pesata per profondità.
    pub support: f32,
    /// Le radici (parole input) da cui questo concetto è raggiungibile.
    /// Più radici → punto di convergenza.
    pub root_witnesses: Vec<String>,
}

/// Un arco letto dal KG durante l'esplorazione.
#[derive(Debug, Clone)]
pub struct TraversedEdge {
    pub from: String,
    pub relation: RelationType,
    pub to: String,
    pub confidence: f32,
    /// Profondità del nodo `from` (0 = è una radice).
    pub depth: u8,
}

/// Un concetto che converge da più parole input — ciò di cui la frase
/// "parla davvero" emerge qui.
#[derive(Debug, Clone)]
pub struct Convergence {
    pub concept: String,
    pub witnesses: Vec<String>,
    pub strength: f32,
}

/// Un sillogismo dedotto: A r1 B, B r2 C ⇒ A r* C.
/// La composizione `r* = compose(r1, r2)` è derivata da regole esplicite —
/// non tutte le coppie di relazioni si compongono.
#[derive(Debug, Clone)]
pub struct Syllogism {
    pub subject: String,
    pub r1: RelationType,
    pub middle: String,
    pub r2: RelationType,
    pub object: String,
    /// La relazione composta (se la composizione è significativa).
    pub composed: Option<RelationType>,
    pub strength: f32,
}

/// Il grafo di comprensione completo per un input.
#[derive(Debug, Clone)]
pub struct ComprehensionGraph {
    /// Le parole input che fanno da radice. Ordine = ordine di apparizione.
    pub roots: Vec<String>,
    /// Tutti i concetti raggiunti, indicizzati per nome.
    pub nodes: HashMap<String, ConceptNode>,
    /// Tutti gli archi traversati.
    pub edges: Vec<TraversedEdge>,
    /// Concetti raggiunti da ≥2 radici, ordinati per strength.
    pub convergences: Vec<Convergence>,
    /// Sillogismi dedotti, ordinati per strength.
    pub syllogisms: Vec<Syllogism>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Composizione di relazioni (sillogismi)
// ═══════════════════════════════════════════════════════════════════════════

/// Regole di composizione tra relazioni: dato `A r1 B, B r2 C`, qual è la
/// relazione composta `A r* C`? Solo le composizioni logicamente forti
/// vengono dichiarate qui — il resto resta come cammino non sillogizzato.
///
/// I tipi:
/// - **Transitive dirette**: IsA∘IsA = IsA, PartOf∘PartOf = PartOf, ...
/// - **Type-inheritance**: IsA∘<r> = <r> (se sono persona, e una persona ha
///   libri, allora io ho un libro — ereditarietà strutturale).
/// - **Equivalenza**: Equivalent∘<r> = <r> e <r>∘Equivalent = <r>.
/// - **Doppia negazione**: OppositeOf∘OppositeOf = SimilarTo.
pub fn compose_relations(r1: RelationType, r2: RelationType) -> Option<RelationType> {
    use RelationType::*;
    match (r1, r2) {
        // Transitività diretta
        (IsA, IsA)             => Some(IsA),
        (PartOf, PartOf)       => Some(PartOf),
        (Causes, Causes)       => Some(Causes),
        (Enables, Enables)     => Some(Enables),
        (Requires, Requires)   => Some(Requires),
        (TransformsInto, TransformsInto) => Some(TransformsInto),

        // Type-inheritance: se X IsA T, T r Y allora X r Y
        (IsA, Has)             => Some(Has),
        (IsA, Does)            => Some(Does),
        (IsA, Causes)          => Some(Causes),
        (IsA, Enables)         => Some(Enables),
        (IsA, Requires)        => Some(Requires),
        (IsA, UsedFor)         => Some(UsedFor),
        (IsA, Expresses)       => Some(Expresses),
        (IsA, Symbolizes)      => Some(Symbolizes),
        (IsA, FeelsAs)         => Some(FeelsAs),
        (IsA, OppositeOf)      => Some(OppositeOf),

        // Concatenamento causale
        (Causes, Enables)      => Some(Enables),
        (Enables, Causes)      => Some(Causes),
        (Causes, Implies)      => Some(Implies),

        // Whole-part inverso non vale: PartOf ∘ Has è ambiguo (mano è parte
        // di corpo, corpo ha sangue ⇏ mano ha sangue) — saltiamo.

        // Equivalenza è trasparente in entrambe le direzioni
        (Equivalent, r)        => Some(r),
        (r, Equivalent)        => Some(r),

        // Similarità è debole: propaga solo IsA (sono simili a un cane,
        // un cane è un mammifero ⇒ siamo simili a un mammifero).
        (SimilarTo, IsA)       => Some(IsA),

        // Doppia negazione
        (OppositeOf, OppositeOf) => Some(SimilarTo),

        // Causale + Manifestazione: X causa Y, Y esprime Z → X esprime Z
        (Causes, Expresses)    => Some(Expresses),
        (Causes, Symbolizes)   => Some(Symbolizes),

        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Build — esplorazione BFS
// ═══════════════════════════════════════════════════════════════════════════

impl ComprehensionGraph {
    /// Costruisce il grafo per un set di lemmi input.
    pub fn build(input_lemmas: &[&str], kg: &KnowledgeGraph) -> Self {
        Self::build_with_depth(input_lemmas, kg, DEFAULT_MAX_DEPTH)
    }

    /// Variante con profondità configurabile (utile nei test).
    pub fn build_with_depth(input_lemmas: &[&str], kg: &KnowledgeGraph, max_depth: u8) -> Self {
        let mut nodes: HashMap<String, ConceptNode> = HashMap::new();
        let mut edges: Vec<TraversedEdge> = Vec::new();

        let roots: Vec<String> = input_lemmas.iter()
            .map(|s| s.to_lowercase())
            .filter(|s| s.len() >= 2)
            .collect();

        // Ogni elemento del frontier rappresenta un nodo da espandere, con
        // la radice da cui proviene per tracciare la convergenza.
        let mut frontier: Vec<(String, u8, String)> = Vec::new();
        for root in &roots {
            nodes.insert(root.clone(), ConceptNode {
                word: root.clone(),
                depth: 0,
                support: 1.0,
                root_witnesses: vec![root.clone()],
            });
            frontier.push((root.clone(), 0, root.clone()));
        }

        while let Some((current, depth, root_witness)) = frontier.pop() {
            if depth >= max_depth { continue; }

            // Hub-damping: non espandere oltre nodi con grado uscente molto alto
            // (depth>0 — le radici si espandono sempre, anche se sono hub).
            let degree = kg.all_outgoing(&current).len();
            if depth > 0 && degree > HUB_DEGREE_THRESHOLD { continue; }

            for (rel, target_str, conf) in kg.all_outgoing(&current) {
                if conf < MIN_EDGE_CONFIDENCE { continue; }
                let target = target_str.to_lowercase();
                if target == current { continue; }
                // Salta se l'arco va a una radice (a meno che sia un nuovo path
                // di convergenza — gestito sotto).

                // Registra l'arco
                edges.push(TraversedEdge {
                    from: current.clone(),
                    relation: rel,
                    to: target.clone(),
                    confidence: conf,
                    depth,
                });

                let support_increment = conf * DEPTH_DECAY.powi(depth as i32);

                let entry = nodes.entry(target.clone()).or_insert_with(|| ConceptNode {
                    word: target.clone(),
                    depth: depth + 1,
                    support: 0.0,
                    root_witnesses: Vec::new(),
                });
                entry.support += support_increment;
                if !entry.root_witnesses.iter().any(|w| w == &root_witness) {
                    entry.root_witnesses.push(root_witness.clone());
                }
                // Se il nodo era già stato raggiunto a depth maggiore, aggiorna
                // alla depth minore (per BFS la prima visita è la più breve, ma
                // diversi cammini possono raggiungerlo a depth diverse).
                if entry.depth > depth + 1 {
                    entry.depth = depth + 1;
                }

                // Espandi a meno che il target sia già una radice (evita cicli).
                if depth + 1 < max_depth && !roots.contains(&target) {
                    frontier.push((target, depth + 1, root_witness.clone()));
                }
            }
        }

        // Convergenze: nodi raggiunti da ≥2 radici (oppure raggiunti da un'unica
        // radice ma con support alto e depth>0 — escludiamo le radici stesse).
        let mut convergences: Vec<Convergence> = nodes.values()
            .filter(|n| n.depth > 0 && n.root_witnesses.len() >= 2)
            .map(|n| Convergence {
                concept: n.word.clone(),
                witnesses: n.root_witnesses.clone(),
                strength: n.support,
            })
            .collect();
        convergences.sort_by(|a, b| {
            b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Sillogismi: cammini depth-0 + depth-1 con composizione esplicita.
        let syllogisms = detect_syllogisms(&edges);

        ComprehensionGraph {
            roots,
            nodes,
            edges,
            convergences,
            syllogisms,
        }
    }

    /// Numero totale di concetti raggiunti (incluse radici).
    pub fn node_count(&self) -> usize { self.nodes.len() }

    /// Numero di archi traversati.
    pub fn edge_count(&self) -> usize { self.edges.len() }

    /// Le radici come slice di stringhe.
    pub fn root_set(&self) -> std::collections::HashSet<&str> {
        self.roots.iter().map(|s| s.as_str()).collect()
    }

    /// Concetti che NON sono radici, ordinati per support decrescente.
    /// Sono i candidati naturali per una risposta dalla regione.
    pub fn explored_concepts_by_support(&self) -> Vec<&ConceptNode> {
        let mut v: Vec<&ConceptNode> = self.nodes.values()
            .filter(|n| n.depth > 0)
            .collect();
        v.sort_by(|a, b| {
            b.support.partial_cmp(&a.support).unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }

    /// Trova "fratelli" delle radici: concetti che condividono un parent
    /// IsA con almeno una radice. Sono il vocabolario per rispondere DALLA
    /// REGIONE dell'input senza echeggiare l'input stesso.
    pub fn siblings_of_roots(&self, kg: &KnowledgeGraph, max: usize) -> Vec<(String, f32)> {
        let mut sibling_scores: HashMap<String, f32> = HashMap::new();
        let root_set: std::collections::HashSet<&str> = self.root_set();

        for root in &self.roots {
            let parents = kg.query_objects_weighted(root, RelationType::IsA);
            for (parent, conf_isa) in parents {
                // Per ogni parent, recupera tutti i suoi figli (fratelli del root)
                let siblings = kg.query_subjects(parent, RelationType::IsA);
                for sib in siblings {
                    if root_set.contains(sib) { continue; }
                    if sib == root { continue; }
                    let entry = sibling_scores.entry(sib.to_string()).or_insert(0.0);
                    *entry += conf_isa * 0.5;
                }
            }
        }

        let mut v: Vec<(String, f32)> = sibling_scores.into_iter().collect();
        v.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        v.truncate(max);
        v
    }
}

/// Rileva sillogismi nel set di archi traversati. Solo le coppie (depth-0, depth-1)
/// sono considerate — composizioni più lunghe sono troppo deboli.
fn detect_syllogisms(edges: &[TraversedEdge]) -> Vec<Syllogism> {
    let depth0: Vec<&TraversedEdge> = edges.iter().filter(|e| e.depth == 0).collect();
    let depth1: Vec<&TraversedEdge> = edges.iter().filter(|e| e.depth == 1).collect();

    let mut sylls: Vec<Syllogism> = Vec::new();
    for e1 in &depth0 {
        for e2 in &depth1 {
            if e1.to != e2.from { continue; }
            if e1.from == e2.to { continue; }
            let composed = compose_relations(e1.relation, e2.relation);
            if composed.is_none() { continue; }
            let strength = e1.confidence * e2.confidence * 0.7;
            sylls.push(Syllogism {
                subject: e1.from.clone(),
                r1: e1.relation,
                middle: e1.to.clone(),
                r2: e2.relation,
                object: e2.to.clone(),
                composed,
                strength,
            });
        }
    }
    sylls.sort_by(|a, b| {
        b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal)
    });
    sylls.truncate(20);
    sylls
}

// ═══════════════════════════════════════════════════════════════════════════
// ReciprocalAct — atto comunicativo che chiama una risposta della stessa
// classe. Per un saluto si risponde con un saluto, per un ringraziamento
// con accoglienza/cortesia, per un congedo con un congedo. Non è template:
// è riconoscimento strutturale dal KG (parent IsA con ≥3 figli) + scelta
// per allineamento di firma 8D con lo stato corrente di UI-r1.
// ═══════════════════════════════════════════════════════════════════════════

/// Numero minimo di figli IsA richiesti perché un parent sia considerato
/// "classe di atto comunicativo" (cioè abbia abbastanza istanze per
/// supportare una scelta di risposta).
pub const MIN_SIBLINGS_FOR_ACT: usize = 3;

/// Un atto comunicativo riconosciuto come istanza di una classe (saluto,
/// ringraziamento, scusa, congedo, ...) con i fratelli-istanza che sono
/// candidati naturali per la risposta.
#[derive(Debug, Clone)]
pub struct ReciprocalAct {
    /// Il parent IsA — la classe (es. "saluto").
    pub act_type: String,
    /// La parola input riconosciuta come istanza (es. "ciao").
    pub root: String,
    /// Le altre istanze della stessa classe nel KG (es. ["salve", "benvenuto", "buongiorno"]).
    pub siblings: Vec<String>,
    /// Confidence dell'arco IsA root → act_type (quanto è certa la classificazione).
    pub classification_confidence: f32,
}

impl ReciprocalAct {
    /// Rileva se l'input è un atto comunicativo che chiama una risposta
    /// della stessa classe. Condizioni:
    ///  - input ha 1 sola parola contenuto (root unica)
    ///  - quella parola ha un parent IsA con ≥ MIN_SIBLINGS_FOR_ACT figli
    ///  - se più parents qualificano, sceglie quello con MENO figli
    ///    (più specifico — "saluto" batte "atto" se entrambi qualificano)
    pub fn detect(graph: &ComprehensionGraph, kg: &KnowledgeGraph) -> Option<Self> {
        if graph.roots.len() != 1 { return None; }
        let root = graph.roots[0].clone();

        let parents = kg.query_objects_weighted(&root, RelationType::IsA);
        if parents.is_empty() { return None; }

        let mut best: Option<(String, Vec<String>, f32)> = None;
        for (parent, conf) in parents {
            let all_children = kg.query_subjects(parent, RelationType::IsA);
            if all_children.len() < MIN_SIBLINGS_FOR_ACT { continue; }
            if all_children.len() > HUB_DEGREE_THRESHOLD { continue; }
            let siblings: Vec<String> = all_children.iter()
                .filter(|s| **s != root.as_str())
                .map(|s| s.to_string())
                .collect();
            if siblings.is_empty() { continue; }
            let take = match &best {
                None => true,
                Some((_, b, _)) => siblings.len() < b.len(),
            };
            if take {
                best = Some((parent.to_string(), siblings, conf));
            }
        }

        best.map(|(act_type, siblings, classification_confidence)| ReciprocalAct {
            act_type,
            root,
            siblings,
            classification_confidence,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::knowledge_graph::KnowledgeGraph;
    use crate::topology::relation::RelationType;

    fn build_test_kg() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        // Scena "ciao"
        kg.add("ciao", RelationType::IsA, "saluto");
        kg.add("ciao", RelationType::Causes, "incontro");
        kg.add("ciao", RelationType::Requires, "presenza");
        kg.add("ciao", RelationType::OppositeOf, "addio");
        // Saluti fratelli (sotto saluto)
        kg.add("salve", RelationType::IsA, "saluto");
        kg.add("benvenuto", RelationType::IsA, "saluto");
        kg.add("buongiorno", RelationType::IsA, "saluto");
        // 2-hop: saluto → presenza, saluto → apertura
        kg.add("saluto", RelationType::Requires, "presenza");
        kg.add("saluto", RelationType::Causes, "apertura");
        // 2-hop: incontro → presenza, incontro → relazione
        kg.add("incontro", RelationType::Requires, "presenza");
        kg.add("incontro", RelationType::Causes, "relazione");
        kg
    }

    #[test]
    fn graph_collects_all_one_hop_edges_from_root() {
        let kg = build_test_kg();
        let g = ComprehensionGraph::build(&["ciao"], &kg);
        // Almeno IsA, Causes, Requires, OppositeOf da "ciao"
        let depth0: Vec<&TraversedEdge> = g.edges.iter().filter(|e| e.depth == 0).collect();
        assert!(depth0.len() >= 4, "depth-0 edges = {}", depth0.len());
    }

    #[test]
    fn graph_explores_two_hops() {
        let kg = build_test_kg();
        let g = ComprehensionGraph::build(&["ciao"], &kg);
        // "presenza" è raggiunto sia da ciao (depth 1) che da saluto/incontro (depth 2)
        let presenza = g.nodes.get("presenza").expect("presenza nel grafo");
        assert_eq!(presenza.depth, 1, "presenza è 1-hop diretto da ciao");
    }

    #[test]
    fn graph_detects_convergence_on_presenza() {
        let kg = build_test_kg();
        let g = ComprehensionGraph::build(&["ciao"], &kg);
        // "presenza" è richiamato da ciao (Requires) E saluto (Requires) E incontro (Requires)
        // Tutte le invocazioni hanno radice "ciao", quindi nel grafo presenza ha 1 root_witness.
        // Per convergenza con ≥2 root serve un secondo input.
        let g2 = ComprehensionGraph::build(&["ciao", "incontro"], &kg);
        let conv: Vec<&Convergence> = g2.convergences.iter()
            .filter(|c| c.concept == "presenza")
            .collect();
        assert!(!conv.is_empty(), "presenza dovrebbe essere convergenza tra ciao e incontro");
        assert!(conv[0].witnesses.contains(&"ciao".to_string()));
        assert!(conv[0].witnesses.contains(&"incontro".to_string()));
    }

    #[test]
    fn graph_detects_syllogism_isa_causes() {
        let mut kg = KnowledgeGraph::new();
        // X IsA T, T Causes Y → X Causes Y
        kg.add("cane", RelationType::IsA, "animale");
        kg.add("animale", RelationType::Causes, "movimento");
        let g = ComprehensionGraph::build(&["cane"], &kg);
        let syll = g.syllogisms.iter().find(|s|
            s.subject == "cane" && s.middle == "animale" && s.object == "movimento"
        ).expect("manca il sillogismo cane→animale→movimento");
        assert_eq!(syll.r1, RelationType::IsA);
        assert_eq!(syll.r2, RelationType::Causes);
        assert_eq!(syll.composed, Some(RelationType::Causes));
    }

    #[test]
    fn graph_finds_siblings_of_root_under_isa() {
        let kg = build_test_kg();
        let g = ComprehensionGraph::build(&["ciao"], &kg);
        let sibs = g.siblings_of_roots(&kg, 10);
        let names: Vec<&str> = sibs.iter().map(|(s,_)| s.as_str()).collect();
        // I fratelli di ciao sotto "saluto" sono: salve, benvenuto, buongiorno
        assert!(names.contains(&"salve"), "fratelli: {:?}", names);
        assert!(names.contains(&"benvenuto"), "fratelli: {:?}", names);
        assert!(names.contains(&"buongiorno"), "fratelli: {:?}", names);
        // ciao stesso non deve essere nella lista
        assert!(!names.contains(&"ciao"));
    }

    #[test]
    fn hub_damping_skips_oversaturated_nodes() {
        // KG con un hub artificiale (>HUB_DEGREE_THRESHOLD figli)
        let mut kg = KnowledgeGraph::new();
        kg.add("specifico", RelationType::IsA, "hub");
        for i in 0..(HUB_DEGREE_THRESHOLD + 50) {
            kg.add("hub", RelationType::Has, &format!("attr_{}", i));
        }
        let g = ComprehensionGraph::build(&["specifico"], &kg);
        // hub raggiunto a depth=1 ma NON espanso → niente attr_* nei nodi.
        let attr_count = g.nodes.keys().filter(|k| k.starts_with("attr_")).count();
        assert_eq!(attr_count, 0, "hub damping fallito: {} attr_* presenti", attr_count);
    }

    #[test]
    fn reciprocal_act_detected_for_greeting() {
        let kg = build_test_kg();
        let g = ComprehensionGraph::build(&["ciao"], &kg);
        let act = ReciprocalAct::detect(&g, &kg).expect("ciao dovrebbe attivare un atto reciproco");
        assert_eq!(act.act_type, "saluto");
        assert_eq!(act.root, "ciao");
        // I fratelli sono salve, benvenuto, buongiorno (escluso ciao stesso)
        assert_eq!(act.siblings.len(), 3);
        assert!(act.siblings.contains(&"salve".to_string()));
        assert!(act.siblings.contains(&"benvenuto".to_string()));
        assert!(act.siblings.contains(&"buongiorno".to_string()));
    }

    #[test]
    fn reciprocal_act_skips_when_too_few_siblings() {
        let mut kg = KnowledgeGraph::new();
        kg.add("solo", RelationType::IsA, "categoria_unica");
        // Solo 1 figlio per categoria_unica → non qualifica.
        let g = ComprehensionGraph::build(&["solo"], &kg);
        assert!(ReciprocalAct::detect(&g, &kg).is_none());
    }

    #[test]
    fn reciprocal_act_skips_for_multi_root_input() {
        let kg = build_test_kg();
        let g = ComprehensionGraph::build(&["ciao", "incontro"], &kg);
        // Più di una radice → non è atto fatico semplice.
        assert!(ReciprocalAct::detect(&g, &kg).is_none());
    }

    #[test]
    fn reciprocal_act_picks_most_specific_parent() {
        let mut kg = KnowledgeGraph::new();
        // ciao IsA saluto IsA atto_comunicativo — e ciao IsA atto IsA atto_comunicativo
        // Entrambi qualificano, saluto è più specifico (3 figli vs 5 figli).
        kg.add("ciao", RelationType::IsA, "saluto");
        kg.add("salve", RelationType::IsA, "saluto");
        kg.add("benvenuto", RelationType::IsA, "saluto");
        kg.add("buongiorno", RelationType::IsA, "saluto");
        kg.add("ciao", RelationType::IsA, "atto");
        kg.add("salve", RelationType::IsA, "atto");
        kg.add("ringraziare", RelationType::IsA, "atto");
        kg.add("scusare", RelationType::IsA, "atto");
        kg.add("congedare", RelationType::IsA, "atto");

        let g = ComprehensionGraph::build(&["ciao"], &kg);
        let act = ReciprocalAct::detect(&g, &kg).expect("atto dovrebbe essere rilevato");
        // Deve scegliere "saluto" (più specifico) non "atto"
        assert_eq!(act.act_type, "saluto");
    }


    #[test]
    fn graph_supports_emerge_from_multi_path() {
        let kg = build_test_kg();
        let g = ComprehensionGraph::build(&["ciao"], &kg);
        // "presenza" ha support proveniente da almeno due cammini:
        // ciao→presenza diretto (depth 0→1) E ciao→saluto→presenza (0→1→1).
        // Il support di presenza deve essere > del support di "addio" (solo 1 path).
        let presenza_sup = g.nodes.get("presenza").map(|n| n.support).unwrap_or(0.0);
        let addio_sup = g.nodes.get("addio").map(|n| n.support).unwrap_or(0.0);
        assert!(presenza_sup > addio_sup, "presenza={} addio={}", presenza_sup, addio_sup);
    }
}
