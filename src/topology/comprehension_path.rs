//! Phase 86 — Stadio 1: la comprensione come pathfinding tipato DIRETTO.
//!
//! > Design: `docs/raw/architettura/comprensione_esplorativa_design.md`.
//!
//! Modulo **additivo e ispezionabile**: NON tocca `compose`/nuclei. Prende la
//! [`SentenceProposition`] (Phase 81) — già strutturata in soggetto/relazione/
//! oggetto/via/polarità — e ne deriva un [`ComprehensionGraph`]: i cammini
//! tipati che connettono i nodi-contenuto della frase tra loro e al *terreno
//! fondato*. Serve a MISURARE prima di toccare l'output (principio "uno alla
//! volta, misurando, reversibile").
//!
//! ## L'invariante anti-«sacco di parole» (design §3.1)
//!
//! Il pathfinding NON parte da un *insieme* di nodi: parte dalla proposizione
//! STRUTTURATA. `io ho fame` (`Speaker Has fame +`), `io non ho fame` (pol −) e
//! `ho fame di io` (`World(fame) … via=io`) condividono `{io, fame}` ma sono
//! **tre grafi diversi**, perché ruolo, ordine, preposizione e polarità *sono*
//! comprensione. La polarità VINCOLA il confronto (non è un'etichetta a valle).
//!
//! ## Grounding (design §3.3)
//!
//! Un ramo si ferma quando raggiunge: (a) un altro nodo della frase
//! [connessione], (b) un **attrattore** (categoria-substrato), (c) un **nodo del
//! sé** (`kg_self`), (d) un nodo già visitato. Più un backstop di profondità.
//! Gli attrattori si fermano anche il flooding: le mega-categorie SONO ground,
//! quindi il BFS non le attraversa mai (niente hub-damping numerico).

use std::collections::{HashMap, HashSet, VecDeque};

use crate::topology::kg_self::KgSelf;
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;
use crate::topology::sentence_proposition::{ObjectRef, SentenceProposition, SubjectRef};

/// Soglia strutturale (NON un gate sul significato): un nodo è "attrattore /
/// categoria-substrato" se almeno questo numero di nodi sono `IsA` esso. È la
/// stessa nozione di specificità di `find_activated_attractors` (Phase 59/61).
/// **Provvisorio Stadio 1**: la versione principale userà l'insieme-attrattori
/// dei 64 stati (nucleus.tsv). Qui basta a fondare e a fermare il flooding.
const ATTRACTOR_MIN_CHILDREN: usize = 25;

/// Backstop di sicurezza sulla profondità (design §3.3: cap DICHIARATO, non
/// gate sul significato — i cammini reali sono 1-3 hop).
const MAX_DEPTH: usize = 4;

/// Come un ramo del pathfinding ha toccato terra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundKind {
    /// Ha raggiunto un altro nodo-contenuto della frase (connessione trovata).
    PropositionNode,
    /// Ha raggiunto un nodo del sé (`kg_self`).
    SelfNode,
    /// Ha raggiunto un attrattore (categoria-substrato).
    Attractor,
    /// Il nodo di partenza era già fondato (attrattore / sé).
    AlreadyGround,
    /// Nessuna ancora raggiunta entro il backstop → gap onesto.
    Unreached,
}

/// Un passo di un cammino tipato. `forward=false` significa che l'arco è stato
/// percorso al contrario (entrante): `to <-relation- from`.
#[derive(Debug, Clone, PartialEq)]
pub struct PathStep {
    pub relation: RelationType,
    pub forward: bool,
    pub via: Option<String>,
    pub to: String,
    pub confidence: f32,
}

/// Un cammino tipato da un nodo a un'ancora (o a un altro nodo della frase).
#[derive(Debug, Clone, PartialEq)]
pub struct TypedPath {
    pub from: String,
    pub steps: Vec<PathStep>,
    pub ground: GroundKind,
}

impl TypedPath {
    pub fn endpoint(&self) -> &str {
        self.steps.last().map(|s| s.to.as_str()).unwrap_or(&self.from)
    }
}

/// Esito del confronto fra ciò che la proposizione ASSERISCE (relazione +
/// polarità) e ciò che il mondo TIENE. La polarità entra qui, nel *segno* del
/// confronto, non come flag cosmetico (design §3.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Confront {
    /// Il mondo afferma la stessa relazione e le polarità concordano.
    Confirm,
    /// Il mondo afferma la relazione, ma la proposizione la nega (o viceversa).
    Contradict,
    /// La proposizione propone una relazione che il mondo (ancora) non ha.
    Novelty,
    /// Soggetto non-Mondo (Speaker/Entity/Variable): confronto col mondo non
    /// applicabile a questo livello (è confront_with_self, stadio successivo).
    NotApplicable,
}

/// Il grafo di comprensione di una frase: l'ologramma strutturato (design §2).
#[derive(Debug, Clone)]
pub struct ComprehensionGraph {
    /// Nodo-soggetto se è del Mondo; `None` se Speaker/Entity/Variable.
    pub root: Option<String>,
    pub relation: RelationType,
    /// Nodo-oggetto concreto, se presente.
    pub target: Option<String>,
    pub via: Option<String>,
    /// Lemma del verbo di superficie che ha realizzato la relazione (dalla PROP).
    /// Per un `Does` del Mondo è il verbo concreto ("uccidere"), così il collasso
    /// lo realizza invece del generico "compie"; il `target` è il paziente.
    pub verb_lemma: Option<String>,
    pub polarity: bool,
    /// Il cammino diretto soggetto→oggetto (la relazione asserita), se i due
    /// nodi sono del Mondo e connessi.
    pub claim_path: Option<TypedPath>,
    /// Confronto della relazione asserita col mondo (polarità inclusa).
    pub confront: Confront,
    /// Per ogni nodo-contenuto: il cammino al terreno fondato più vicino.
    pub groundings: Vec<TypedPath>,
    /// Nodi-contenuto che non raggiungono alcuna ancora → gap onesti
    /// (design §3.5: "non so cosa sia X" è un atto, non un crash).
    pub ungrounded: Vec<String>,
}

/// Insieme degli attrattori (categorie-substrato) del kg_sem.
fn attractor_set(kg: &KnowledgeGraph) -> HashSet<String> {
    kg.categories_for(RelationType::IsA, ATTRACTOR_MIN_CHILDREN)
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect()
}

/// Un nodo è "fondato" (ferma un ramo del BFS)? Restituisce come.
fn ground_kind(
    node: &str,
    attractors: &HashSet<String>,
    self_nodes: &HashSet<String>,
    prop_nodes: &HashSet<String>,
) -> Option<GroundKind> {
    if prop_nodes.contains(node) {
        Some(GroundKind::PropositionNode)
    } else if self_nodes.contains(node) {
        Some(GroundKind::SelfNode)
    } else if attractors.contains(node) {
        Some(GroundKind::Attractor)
    } else {
        None
    }
}

/// Vicini tipati di un nodo, in entrambi i sensi (la comprensione segue le
/// relazioni in entrambe le direzioni, registrando il senso percorso).
/// Registro attivo per la disambiguazione dei sensi (polisemia). Quando un nodo
/// ha archi concorrenti — più sensi, ognuno con un `via`-registro (metodo
/// dell'agente kg: `solo FeelsAs solitudine via=emozione` / `solo SimilarTo
/// assolo via=musica`) — preferiamo l'arco il cui `via` è "lit" (combacia con
/// una parola co-attiva della frase) o la cui relazione combacia con la
/// COSTRUZIONE (la PROP: "mi sento solo" è FeelsAs → il senso emotivo di "solo",
/// non l'assolo). Data-driven: nessuna lista di parole in Rust.
struct RegisterHint<'a> {
    relation: RelationType,
    active: &'a HashSet<String>,
}

/// Punteggio di registro di un arco: 2 se il suo `via` è lit (registro attivo),
/// 1 se la relazione combacia con la costruzione, 0 altrimenti.
fn register_score(via: &Option<String>, rel: RelationType, hint: &RegisterHint) -> u8 {
    if let Some(v) = via {
        if hint.active.contains(&v.to_lowercase()) {
            return 2;
        }
    }
    if rel == hint.relation {
        return 1;
    }
    0
}

fn neighbors(
    kg: &KnowledgeGraph,
    node: &str,
    hint: Option<&RegisterHint>,
) -> Vec<(RelationType, bool, Option<String>, String, f32)> {
    let mut out = Vec::new();
    for (rel, to, conf, via) in kg.all_outgoing_full(node) {
        out.push((rel, true, via.map(|s| s.to_string()), to.to_lowercase(), conf));
    }
    for (rel, from, conf) in kg.all_incoming(node) {
        out.push((rel, false, None, from.to_lowercase(), conf));
    }
    // Ordine: PRIMA il registro (disambiguazione del senso per polisemia), poi
    // — DETERMINISMO (bug critico 2026-06-08: le query KG iterano HashMap in
    // ordine randomizzato → la BFS prendeva un ancoraggio diverso a ogni run) —
    // confidenza decrescente, relazione, nodo come spareggio stabile.
    out.sort_by(|a, b| {
        let ra = hint.map(|h| register_score(&a.2, a.0, h)).unwrap_or(0);
        let rb = hint.map(|h| register_score(&b.2, b.0, h)).unwrap_or(0);
        rb.cmp(&ra)
            .then_with(|| b.4.partial_cmp(&a.4).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| (a.0 as u8).cmp(&(b.0 as u8)))
            .then_with(|| a.3.cmp(&b.3))
    });
    out
}

/// BFS diretto `from → to`: il cammino più corto che connette i due nodi della
/// frase, **senza attraversare** nodi fondati (che fermano il ramo, evitando il
/// flooding attraverso le mega-categorie). Atterrare su `to` è permesso.
fn find_path(
    from: &str,
    to: &str,
    kg: &KnowledgeGraph,
    attractors: &HashSet<String>,
    self_nodes: &HashSet<String>,
) -> Option<TypedPath> {
    if from == to {
        return None;
    }
    // prev: nodo -> (predecessore, step usato per arrivarci)
    let mut prev: HashMap<String, (String, PathStep)> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(from.to_string());
    let mut q: VecDeque<(String, usize)> = VecDeque::new();
    q.push_back((from.to_string(), 0));

    while let Some((cur, depth)) = q.pop_front() {
        if depth >= MAX_DEPTH {
            continue;
        }
        // Non attraversare un nodo fondato (tranne il punto di partenza): le
        // categorie-substrato fermano il ramo. `to` è gestito al landing sotto.
        if cur != from && cur != to {
            // i prop_nodes qui sono solo {to}; ground su attrattore/sé ferma.
            let mut single = HashSet::new();
            single.insert(to.to_string());
            if ground_kind(&cur, attractors, self_nodes, &single).is_some() {
                continue;
            }
        }
        for (rel, fwd, via, nb, conf) in neighbors(kg, &cur, None) {
            if visited.contains(&nb) {
                continue;
            }
            let step = PathStep { relation: rel, forward: fwd, via, to: nb.clone(), confidence: conf };
            prev.insert(nb.clone(), (cur.clone(), step));
            if nb == to {
                return Some(reconstruct(from, to, &prev, GroundKind::PropositionNode));
            }
            visited.insert(nb.clone());
            q.push_back((nb, depth + 1));
        }
    }
    None
}

/// BFS dal nodo all'ancora-fondata. **Multi-candidato (2026-06-10)**: invece di
/// fermarsi al PRIMO ground raggiunto (che, ordinato per confidenza, era spesso
/// un meta-edge — `paura Requires oggetto` → "La paura ha bisogno dell'oggetto"),
/// raccoglie TUTTI i ground raggiungibili (terminali, non attraversati: la
/// frontiera resta limitata) e sceglie il migliore con preferenza STRUTTURALE
/// (`grounding_preferred`): sé > attrattore > nodo-frase, poi TASSONOMICO (primo
/// passo `IsA`: "che cos'è X" — Quillian — batte "di cosa X ha bisogno"), poi
/// più corto, poi confidenza. Niente numeri-magici: spareggio ordinato e stabile.
/// `hint` (registro attivo) disambigua i sensi degli omonimi.
fn ground_node(
    node: &str,
    kg: &KnowledgeGraph,
    attractors: &HashSet<String>,
    self_nodes: &HashSet<String>,
    prop_nodes: &HashSet<String>,
    hint: Option<&RegisterHint>,
) -> TypedPath {
    // Il nodo stesso è già terra?
    if let Some(k) = ground_kind(node, attractors, self_nodes, &HashSet::new()) {
        let k = if k == GroundKind::PropositionNode { GroundKind::AlreadyGround } else { k };
        return TypedPath { from: node.to_string(), steps: vec![], ground: GroundKind::AlreadyGround.min_with(k) };
    }
    let mut prev: HashMap<String, (String, PathStep)> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(node.to_string());
    let mut q: VecDeque<(String, usize)> = VecDeque::new();
    q.push_back((node.to_string(), 0));
    let mut candidates: Vec<TypedPath> = Vec::new();

    while let Some((cur, depth)) = q.pop_front() {
        if depth >= MAX_DEPTH {
            continue;
        }
        for (rel, fwd, via, nb, conf) in neighbors(kg, &cur, hint) {
            if visited.contains(&nb) {
                continue;
            }
            let step = PathStep { relation: rel, forward: fwd, via, to: nb.clone(), confidence: conf };
            // Raggiunta un'ancora? È TERMINALE — registra il candidato (cammino
            // più corto verso QUESTO ground, perché BFS) e NON la attraversare.
            if let Some(k) = ground_kind(&nb, attractors, self_nodes, prop_nodes) {
                prev.insert(nb.clone(), (cur.clone(), step));
                visited.insert(nb.clone());
                candidates.push(reconstruct(node, &nb, &prev, k));
                continue;
            }
            prev.insert(nb.clone(), (cur.clone(), step));
            visited.insert(nb.clone());
            q.push_back((nb, depth + 1));
        }
    }
    candidates
        .into_iter()
        .max_by(|a, b| grounding_preferred(a, b))
        .unwrap_or(TypedPath { from: node.to_string(), steps: vec![], ground: GroundKind::Unreached })
}

/// Preferenza strutturale fra due cammini di grounding (ordinamento crescente:
/// `max_by` prende il migliore). UNICA sorgente di verità dello spareggio, usata
/// sia per scegliere il grounding di un nodo (`ground_node`) sia il più saliente
/// del grafo (`path_collapse::salient_grounding` la richiama).
pub(crate) fn grounding_preferred(a: &TypedPath, b: &TypedPath) -> std::cmp::Ordering {
    let rank = |g: &GroundKind| match g {
        GroundKind::SelfNode => 3u8,
        GroundKind::Attractor => 2,
        GroundKind::PropositionNode => 1,
        _ => 0,
    };
    let taxonomic = |p: &TypedPath| -> u8 {
        u8::from(matches!(p.steps.first().map(|s| s.relation), Some(RelationType::IsA)))
    };
    let avg_conf = |p: &TypedPath| -> f32 {
        if p.steps.is_empty() { return 0.0; }
        p.steps.iter().map(|s| s.confidence).sum::<f32>() / p.steps.len() as f32
    };
    rank(&a.ground)
        .cmp(&rank(&b.ground))
        .then(taxonomic(a).cmp(&taxonomic(b)))
        .then(b.steps.len().cmp(&a.steps.len())) // più corto = migliore → inverti
        .then(avg_conf(a).partial_cmp(&avg_conf(b)).unwrap_or(std::cmp::Ordering::Equal))
        .then(b.endpoint().cmp(a.endpoint())) // spareggio stabile per nome
}

/// Tenta di fondare un nodo VIA la sua base derivazionale (Phase 86 §2). Se
/// `node` è una forma derivata la cui base è nota/fondata, costruisce un cammino
/// `node —DerivesFrom·via=tipo→ base —…→ ancora`. Niente rescue se il riconoscitore
/// non trova una base (validata contro il KG dentro `derivational_base`).
fn ground_via_derivation(
    node: &str,
    kg: &KnowledgeGraph,
    attractors: &HashSet<String>,
    self_nodes: &HashSet<String>,
    prop_nodes: &HashSet<String>,
) -> Option<TypedPath> {
    let (base, tipo) = crate::topology::derivation::derivational_base(node, kg)?;
    let step = PathStep {
        relation: RelationType::DerivesFrom,
        forward: true,
        via: Some(tipo),
        to: base.clone(),
        confidence: 1.0,
    };
    // Fonda la base; il suo cammino diventa la coda del nostro. (Nessun hint di
    // registro qui: la rescue morfologica non disambigua sensi.)
    let base_path = ground_node(&base, kg, attractors, self_nodes, prop_nodes, None);
    let ground = if base_path.ground == GroundKind::Unreached {
        // La base esiste (validata) ma non raggiunge ancore: è comunque un
        // ancoraggio morfologico — il derivato È compreso come forma della base.
        GroundKind::AlreadyGround
    } else {
        base_path.ground.clone()
    };
    let mut steps = vec![step];
    steps.extend(base_path.steps);
    Some(TypedPath { from: node.to_string(), steps, ground })
}

fn reconstruct(
    from: &str,
    to: &str,
    prev: &HashMap<String, (String, PathStep)>,
    ground: GroundKind,
) -> TypedPath {
    let mut steps = Vec::new();
    let mut cur = to.to_string();
    while let Some((p, step)) = prev.get(&cur) {
        steps.push(step.clone());
        cur = p.clone();
        if cur == from {
            break;
        }
    }
    steps.reverse();
    TypedPath { from: from.to_string(), steps, ground }
}

impl GroundKind {
    /// Helper interno: preferenza fra due ground-kind triviali (non usato in BFS).
    fn min_with(self, other: GroundKind) -> GroundKind {
        // Sé e attrattore valgono più di "già nodo della frase".
        match (&self, &other) {
            (_, GroundKind::SelfNode) | (GroundKind::SelfNode, _) => GroundKind::SelfNode,
            (_, GroundKind::Attractor) | (GroundKind::Attractor, _) => GroundKind::Attractor,
            _ => GroundKind::AlreadyGround,
        }
    }
}

/// Estrae il nodo-contenuto del Mondo da un soggetto (se è del Mondo).
fn subject_node(s: &SubjectRef) -> Option<String> {
    match s {
        SubjectRef::World(w) => Some(w.to_lowercase()),
        _ => None,
    }
}

fn object_node(o: &Option<ObjectRef>) -> Option<String> {
    match o {
        Some(ObjectRef::Word(w)) => Some(w.to_lowercase()),
        _ => None,
    }
}

/// Calcola il confronto della relazione ASSERITA col mondo (polarità inclusa).
fn compute_confront(
    root: &Option<String>,
    target: &Option<String>,
    relation: RelationType,
    polarity: bool,
    kg: &KnowledgeGraph,
    connected: bool,
) -> Confront {
    let (Some(r), Some(t)) = (root, target) else {
        return Confront::NotApplicable;
    };
    // Il mondo afferma DIRETTAMENTE la relazione asserita r --relation--> t ?
    let direct = kg
        .query_objects_with_via(r, relation)
        .iter()
        .any(|(o, _, _)| o.eq_ignore_ascii_case(t));
    match (direct, polarity) {
        (true, true) => Confront::Confirm,    // il mondo conferma, polarità concorde
        (true, false) => Confront::Contradict, // il mondo tiene ciò che la frase nega
        (false, true) => {
            // Relazione asserita non diretta: se i due nodi sono comunque
            // connessi è una novità inferibile; altrimenti pura novità.
            let _ = connected;
            Confront::Novelty
        }
        (false, false) => Confront::Confirm, // entrambi negano la stessa cosa
    }
}

/// Phase 86 — auto-collocazione di UN nodo (vista Stato interno). Il cammino
/// tipato multi-hop più saliente dal nodo a un'ancora fondata (attrattore / nodo
/// del sé), con la STESSA logica del grounding di comprensione (`ground_node`):
/// non attraversa le mega-categorie, sceglie il ground per preferenza strutturale
/// (sé > attrattore, poi tassonomico, poi più corto). Read-only — è il TENTATIVO
/// dell'entità di capirsi da sola una parola prima di chiedere all'umano.
pub fn ground_word(word: &str, kg_sem: &KnowledgeGraph, kg_self: &KgSelf) -> TypedPath {
    let attractors = attractor_set(kg_sem);
    let self_nodes = kg_self.nodes();
    let prop_nodes: HashSet<String> = HashSet::new();
    ground_node(&word.to_lowercase(), kg_sem, &attractors, &self_nodes, &prop_nodes, None)
}

/// Il cuore: dalla proposizione strutturata al grafo di comprensione.
pub fn explore(prop: &SentenceProposition, kg_sem: &KnowledgeGraph, kg_self: &KgSelf) -> ComprehensionGraph {
    let attractors = attractor_set(kg_sem);
    let self_nodes = kg_self.nodes();

    let root = subject_node(&prop.subject);
    let target = object_node(&prop.object);
    let via = prop.via.as_ref().map(|s| s.to_lowercase());

    // I nodi-contenuto della frase: soggetto/oggetto/via + i complementi
    // (Phase 86 Stadio 2: l'analisi logica completa entra nel grafo). Ognuno è
    // "terra" per il grounding degli altri.
    let mut content: Vec<String> = [root.clone(), target.clone(), via.clone()]
        .into_iter()
        .flatten()
        .collect();
    for c in &prop.complements {
        let n = c.noun.to_lowercase();
        if !content.contains(&n) {
            content.push(n);
        }
    }

    // Cammino della relazione asserita: soggetto → oggetto (se entrambi del Mondo).
    let claim_path = match (&root, &target) {
        (Some(r), Some(t)) => find_path(r, t, kg_sem, &attractors, &self_nodes),
        _ => None,
    };

    let confront = compute_confront(
        &root,
        &target,
        prop.relation,
        prop.polarity,
        kg_sem,
        claim_path.is_some(),
    );

    // Grounding di ogni nodo-contenuto verso l'ancora più vicina. Gli ALTRI
    // nodi-contenuto contano come terra (connessione interna alla frase).
    let mut groundings = Vec::new();
    let mut ungrounded = Vec::new();
    for node in &content {
        let others: HashSet<String> = content.iter().filter(|n| *n != node).cloned().collect();
        // Registro attivo: le altre parole-contenuto della frase (co-attive) +
        // la relazione della costruzione. Disambigua gli omonimi del nodo.
        let hint = RegisterHint { relation: prop.relation, active: &others };
        let path = ground_node(node, kg_sem, &attractors, &self_nodes, &others, Some(&hint));
        if path.ground == GroundKind::Unreached {
            // Rescue derivazionale (Phase 86 §2): una forma derivata che non è
            // un nodo (nessun arco da seguire) può fondarsi VIA la sua base —
            // "pauroso" si capisce come derivato di "paura". L'arco DerivesFrom
            // *curato* è già attraversato sopra (è un arco KG); questo copre i
            // derivati REGOLARI non ancora curati, via il riconoscitore.
            match ground_via_derivation(node, kg_sem, &attractors, &self_nodes, &others) {
                Some(dpath) => groundings.push(dpath),
                None => ungrounded.push(node.clone()),
            }
        } else {
            groundings.push(path);
        }
    }

    ComprehensionGraph {
        root,
        relation: prop.relation,
        target,
        via,
        verb_lemma: prop.verb_lemma.clone(),
        polarity: prop.polarity,
        claim_path,
        confront,
        groundings,
        ungrounded,
    }
}

/// La **salienza del sé** su un grafo di comprensione ∈ [0,1]: quanto la
/// comprensione di questa frase tocca le *pendenze* del sé (Phase 86+, design
/// `comprensione_bisogno_atto.md`). È il segnale che dice "questa frase chiama il
/// sé a prendere posizione" — alimenta `NeedSignals.self_salience`. La grana
/// **deforma quale cammino è saliente, non si renderizza** (reframe §3.6): qui si
/// LEGGE soltanto. = massimo `pendenza_weight` fra tutti i nodi toccati dai
/// cammini (claim + grounding) e dalla terna soggetto/oggetto/via. `max` (non
/// somma) → nessun bias sulla lunghezza del cammino: conta la posta più forte.
pub fn self_salience(g: &ComprehensionGraph, kg_self: &KgSelf) -> f64 {
    let mut best = 0.0_f64;
    let mut consider = |node: &str| {
        let w = kg_self.pendenza_weight(node);
        if w > best {
            best = w;
        }
    };
    for n in [g.root.as_deref(), g.target.as_deref(), g.via.as_deref()]
        .into_iter()
        .flatten()
    {
        consider(n);
    }
    let mut scan = |p: &TypedPath| {
        consider(&p.from);
        for s in &p.steps {
            consider(&s.to);
        }
    };
    if let Some(cp) = &g.claim_path {
        scan(cp);
    }
    for gp in &g.groundings {
        scan(gp);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::relation::{EdgeSource, TypedEdge};

    fn edge(kg: &mut KnowledgeGraph, s: &str, r: RelationType, o: &str) {
        kg.add_edge(TypedEdge {
            subject: s.into(),
            relation: r,
            object: o.into(),
            confidence: 0.9,
            source: EdgeSource::Curated,
            via: None,
        });
    }

    fn empty_self() -> KgSelf {
        KgSelf::default()
    }

    fn prop(subject: SubjectRef, relation: RelationType, object: Option<ObjectRef>, via: Option<&str>, polarity: bool) -> SentenceProposition {
        SentenceProposition { subject, relation, object, via: via.map(|s| s.to_string()), verb_lemma: None, polarity, complements: vec![], subject_surface: None }
    }

    #[test]
    fn cammino_diretto_confermato_dal_mondo() {
        let mut kg = KnowledgeGraph::new();
        edge(&mut kg, "tradimento", RelationType::Requires, "fiducia");
        let p = prop(SubjectRef::World("tradimento".into()), RelationType::Requires, Some(ObjectRef::Word("fiducia".into())), None, true);
        let g = explore(&p, &kg, &empty_self());
        assert_eq!(g.confront, Confront::Confirm);
        assert!(g.claim_path.is_some(), "tradimento e fiducia devono connettersi");
    }

    #[test]
    fn negazione_e_polarita_vincolano_il_confronto() {
        // Il mondo tiene `incertezza IsA fallimento`.
        let mut kg = KnowledgeGraph::new();
        edge(&mut kg, "incertezza", RelationType::IsA, "fallimento");
        // "l'incertezza è un fallimento" (+) → il mondo conferma.
        let pos = prop(SubjectRef::World("incertezza".into()), RelationType::IsA, Some(ObjectRef::Word("fallimento".into())), None, true);
        assert_eq!(explore(&pos, &kg, &empty_self()).confront, Confront::Confirm);
        // "l'incertezza NON è un fallimento" (−) → il mondo contraddice la negazione.
        let neg = prop(SubjectRef::World("incertezza".into()), RelationType::IsA, Some(ObjectRef::Word("fallimento".into())), None, false);
        assert_eq!(explore(&neg, &kg, &empty_self()).confront, Confront::Contradict);
    }

    #[test]
    fn invariante_sacco_di_parole() {
        // Stessi nodi {io, fame}, tre strutture → tre grafi diversi.
        let kg = KnowledgeGraph::new();
        let s = empty_self();
        // "io ho fame" — Speaker Has fame (+)
        let g1 = explore(&prop(SubjectRef::Speaker, RelationType::Has, Some(ObjectRef::Word("fame".into())), None, true), &kg, &s);
        // "io non ho fame" — Speaker Has fame (−)
        let g2 = explore(&prop(SubjectRef::Speaker, RelationType::Has, Some(ObjectRef::Word("fame".into())), None, false), &kg, &s);
        // "ho fame di io" — World(fame) ... via=io
        let g3 = explore(&prop(SubjectRef::World("fame".into()), RelationType::Has, None, Some("io"), true), &kg, &s);

        // g1 vs g2: stessa struttura, polarità opposta.
        assert_eq!(g1.polarity, true);
        assert_eq!(g2.polarity, false);
        assert_ne!(g1.polarity, g2.polarity, "io ho fame ≠ io non ho fame");
        // g3: il soggetto è il Mondo (fame), non il parlante; ha un via.
        assert_eq!(g3.root.as_deref(), Some("fame"));
        assert_eq!(g1.root, None, "in 'io ho fame' il soggetto non è un nodo del Mondo");
        assert_eq!(g3.via.as_deref(), Some("io"));
        assert_eq!(g1.via, None);
        // I tre grafi NON sono lo stesso (root/polarità/via differiscono).
        assert!(!(g1.root == g3.root && g1.via == g3.via), "ruoli diversi → grafi diversi");
    }

    #[test]
    fn grounding_si_ferma_su_attrattore() {
        let mut kg = KnowledgeGraph::new();
        // `emozione` è una categoria-substrato (≥ ATTRACTOR_MIN_CHILDREN figli IsA).
        edge(&mut kg, "paura", RelationType::IsA, "emozione");
        for i in 0..ATTRACTOR_MIN_CHILDREN {
            edge(&mut kg, &format!("e{i}"), RelationType::IsA, "emozione");
        }
        let p = prop(SubjectRef::World("paura".into()), RelationType::IsA, Some(ObjectRef::Word("emozione".into())), None, true);
        let g = explore(&p, &kg, &empty_self());
        // "paura" si fonda: emozione è la sua ancora (ed è anche l'oggetto della frase).
        assert!(g.ungrounded.is_empty() || !g.ungrounded.contains(&"paura".to_string()));
    }

    #[test]
    fn nodo_isolato_resta_gap_onesto() {
        // 'caffè' senza archi → non raggiunge ancore → gap onesto.
        let kg = KnowledgeGraph::new();
        let p = prop(SubjectRef::World("caffè".into()), RelationType::IsA, Some(ObjectRef::Word("bevanda".into())), None, true);
        let g = explore(&p, &kg, &empty_self());
        assert!(g.ungrounded.contains(&"caffè".to_string()), "caffè isolato deve restare gap, non crash");
    }

    #[test]
    fn rescue_derivazionale_fonda_il_derivato_via_base() {
        // "pauroso" NON è un nodo (nessun arco), ma "paura" sì e si fonda su un
        // attrattore. Il rescue §2 lo capisce come derivato di paura.
        let mut kg = KnowledgeGraph::new();
        edge(&mut kg, "paura", RelationType::IsA, "emozione");
        for i in 0..ATTRACTOR_MIN_CHILDREN {
            edge(&mut kg, &format!("e{i}"), RelationType::IsA, "emozione");
        }
        let p = prop(SubjectRef::World("pauroso".into()), RelationType::IsA, None, None, true);
        let g = explore(&p, &kg, &empty_self());
        assert!(!g.ungrounded.contains(&"pauroso".to_string()), "pauroso deve fondarsi via base");
        let dpath = g.groundings.iter().find(|p| p.from == "pauroso").expect("cammino per pauroso");
        assert_eq!(dpath.steps.first().map(|s| s.relation), Some(RelationType::DerivesFrom));
        assert_eq!(dpath.steps.first().map(|s| s.to.clone()), Some("paura".to_string()));
    }

    #[test]
    fn relazione_non_tenuta_dal_mondo_e_novita() {
        // Il mondo NON ha denaro→felicità: affermarlo è una novità, non conferma.
        let kg = KnowledgeGraph::new();
        let p = prop(SubjectRef::World("denaro".into()), RelationType::Causes, Some(ObjectRef::Word("felicità".into())), None, true);
        assert_eq!(explore(&p, &kg, &empty_self()).confront, Confront::Novelty);
    }
}
