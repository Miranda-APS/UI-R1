/// Knowledge Graph — Grafo di conoscenza con archi tipati.
///
/// Questo è il livello semantico che mancava: le parole non sono solo
/// punti in uno spazio 8D con co-occorrenze statistiche. Hanno relazioni
/// logiche esplicite che definiscono il loro significato.
///
/// Struttura:
///   - Nodi: concetti (parole lowercase)
///   - Archi: relazioni tipate (IS_A, HAS, DOES, PART_OF, CAUSES, ...)
///   - Doppio indice: outgoing[soggetto] + incoming[oggetto]
///
/// Separazione dal campo topologico:
///   - KG = conoscenza del mondo (fatti stabili)
///   - WordTopology = stato attuale del campo (attivazioni)
///   - In receive(): KG informa il campo con boost grounded

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use crate::topology::relation::{RelationType, TypedEdge, EdgeSource};
use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════════════════
// KgNode — informazioni aggregate su un concetto
// ═══════════════════════════════════════════════════════════════════════════

/// Target di un arco: oggetto + confidenza + via opzionale.
#[derive(Debug, Clone)]
pub struct KgTarget {
    pub object: String,
    pub confidence: f32,
    pub source: EdgeSource,
    /// Tramite/mezzo della relazione (opzionale).
    pub via: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// KnowledgeGraph — il grafo
// ═══════════════════════════════════════════════════════════════════════════

/// Knowledge Graph con archi logici tipati.
/// Accesso O(1) per query dirette. Query inverse (chi IS-A X?) O(k) con k=archi.
pub struct KnowledgeGraph {
    /// outgoing[soggetto][relazione] = Vec<KgTarget>
    outgoing: HashMap<String, HashMap<RelationType, Vec<KgTarget>>>,
    /// incoming[oggetto][relazione] = Vec<soggetto>
    incoming: HashMap<String, HashMap<RelationType, Vec<String>>>,
    /// Numero totale di archi
    pub edge_count: usize,
    /// Numero di nodi unici
    pub node_count: usize,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
            edge_count: 0,
            node_count: 0,
        }
    }

    // ─── Inserimento ────────────────────────────────────────────────────────

    /// Aggiunge un arco al grafo. Deduplicato per (soggetto, relazione, oggetto).
    pub fn add_edge(&mut self, edge: TypedEdge) {
        let subj = edge.subject.clone();
        let obj = edge.object.clone();
        let rel = edge.relation;

        // Conta nodi nuovi
        let is_new_subj = !self.outgoing.contains_key(&subj);
        let is_new_obj = !self.incoming.contains_key(&obj);

        // Outgoing
        let out_map = self.outgoing.entry(subj.clone()).or_default();
        let out_vec = out_map.entry(rel).or_default();

        // Deduplicazione
        if !out_vec.iter().any(|t| t.object == obj) {
            out_vec.push(KgTarget {
                object: obj.clone(),
                confidence: edge.confidence,
                source: edge.source,
                via: edge.via.clone(),
            });
            self.edge_count += 1;

            // Incoming
            let in_map = self.incoming.entry(obj.clone()).or_default();
            in_map.entry(rel).or_default().push(subj.clone());

            if is_new_subj { self.node_count += 1; }
            if is_new_obj && obj != subj { self.node_count += 1; }
        }
    }

    /// Aggiunge un arco semplice (subject, rel, object) con confidenza 1.0.
    pub fn add(&mut self, subject: &str, rel: RelationType, object: &str) {
        self.add_edge(TypedEdge::new(subject, rel, object));
    }

    // ─── Query dirette ───────────────────────────────────────────────────────

    /// Tutti gli oggetti connessi a `subject` con la relazione `rel`.
    /// Es: query_objects("cane", IsA) → ["animale", "mammifero"]
    pub fn query_objects<'a>(&'a self, subject: &str, rel: RelationType) -> Vec<&'a str> {
        self.outgoing.get(subject)
            .and_then(|m| m.get(&rel))
            .map(|v| v.iter().map(|t| t.object.as_str()).collect())
            .unwrap_or_default()
    }

    /// Come query_objects ma restituisce anche la confidence per-arco.
    /// Es: query_objects_weighted("cane", IsA) → [("animale", 0.95), ("mammifero", 1.0)]
    pub fn query_objects_weighted<'a>(&'a self, subject: &str, rel: RelationType) -> Vec<(&'a str, f32)> {
        self.outgoing.get(subject)
            .and_then(|m| m.get(&rel))
            .map(|v| v.iter().map(|t| (t.object.as_str(), t.confidence)).collect())
            .unwrap_or_default()
    }

    /// Tutti i soggetti che hanno `rel` verso `object`.
    /// Es: query_subjects("animale", IsA) → ["cane", "gatto", "uccello", ...]
    pub fn query_subjects<'a>(&'a self, object: &str, rel: RelationType) -> Vec<&'a str> {
        self.incoming.get(object)
            .and_then(|m| m.get(&rel))
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Tutti gli archi uscenti da `subject` (qualunque relazione).
    pub fn all_outgoing(&self, subject: &str) -> Vec<(RelationType, &str, f32)> {
        match self.outgoing.get(subject) {
            None => vec![],
            Some(m) => {
                let mut result = Vec::new();
                for (rel, targets) in m {
                    for t in targets {
                        result.push((*rel, t.object.as_str(), t.confidence));
                    }
                }
                result
            }
        }
    }

    /// Come all_outgoing ma include anche `via`. Usato dalla UI di curation.
    pub fn all_outgoing_full(&self, subject: &str) -> Vec<(RelationType, &str, f32, Option<&str>)> {
        match self.outgoing.get(subject) {
            None => vec![],
            Some(m) => {
                let mut result = Vec::new();
                for (rel, targets) in m {
                    for t in targets {
                        result.push((*rel, t.object.as_str(), t.confidence, t.via.as_deref()));
                    }
                }
                result
            }
        }
    }

    /// Rimuove tutte le relazioni di una parola dal grafo (sia uscenti che entranti).
    pub fn remove_word(&mut self, word: &str) {
        // 1. Rimuovi archi uscenti: word → Y
        if let Some(rel_map) = self.outgoing.remove(word) {
            for (rel, targets) in &rel_map {
                for t in targets {
                    if let Some(in_map) = self.incoming.get_mut(&t.object) {
                        if let Some(subjects) = in_map.get_mut(rel) {
                            subjects.retain(|s| s != word);
                        }
                    }
                    self.edge_count = self.edge_count.saturating_sub(1);
                }
            }
        }
        // 2. Rimuovi archi entranti: Z → word
        if let Some(rel_map) = self.incoming.remove(word) {
            for (rel, subjects) in &rel_map {
                for subj in subjects {
                    if let Some(out_map) = self.outgoing.get_mut(subj.as_str()) {
                        if let Some(targets) = out_map.get_mut(rel) {
                            let before = targets.len();
                            targets.retain(|t| t.object != word);
                            self.edge_count = self.edge_count.saturating_sub(before - targets.len());
                        }
                    }
                }
            }
        }
        // 3. Ricalcola node_count
        let all: std::collections::HashSet<&String> =
            self.outgoing.keys().chain(self.incoming.keys()).collect();
        self.node_count = all.len();
    }

    /// Migra tutti gli archi di `from` su `into_word`, poi rimuove `from`.
    /// Usato per normalizzare coppie accentata/non-accentata.
    /// Se un arco identico (stessa rel+oggetto) esiste già in `into_word`, lo salta.
    pub fn merge_word_into(&mut self, from: &str, into_word: &str) {
        // Raccogli archi uscenti di `from` (snapshot owned)
        let out_edges: Vec<(RelationType, String, f32, Option<String>)> = self.outgoing
            .get(from)
            .map(|rel_map| rel_map.iter().flat_map(|(&rel, targets)| {
                targets.iter().map(move |t| (rel, t.object.clone(), t.confidence, t.via.clone()))
            }).collect())
            .unwrap_or_default();

        // Raccogli archi entranti: lista di (rel, soggetto) — confidence/via recuperate dopo
        let in_pairs: Vec<(RelationType, String)> = self.incoming
            .get(from)
            .map(|rel_map| rel_map.iter().flat_map(|(&rel, subjects)| {
                subjects.iter().map(move |s| (rel, s.clone()))
            }).collect())
            .unwrap_or_default();

        // Per ogni in_pair, recupera confidence e via dall'outgoing del soggetto
        let in_edges: Vec<(RelationType, String, f32, Option<String>)> = in_pairs.iter()
            .filter_map(|(rel, subj)| {
                self.outgoing.get(subj.as_str())
                    .and_then(|m| m.get(rel))
                    .and_then(|v| v.iter().find(|t| t.object == from))
                    .map(|t| (*rel, subj.clone(), t.confidence, t.via.clone()))
            })
            .collect();

        // Inserisci archi uscenti in into_word
        for (rel, obj, conf, via) in out_edges {
            let target_obj = if obj == from { into_word.to_string() } else { obj };
            self.add_edge(crate::topology::relation::TypedEdge {
                subject: into_word.to_string(),
                relation: rel,
                object: target_obj,
                confidence: conf,
                source: crate::topology::relation::EdgeSource::Curated,
                via,
            });
        }

        // Inserisci archi entranti come (subj → into_word)
        for (rel, subj, conf, via) in in_edges {
            self.add_edge(crate::topology::relation::TypedEdge {
                subject: subj,
                relation: rel,
                object: into_word.to_string(),
                confidence: conf,
                source: crate::topology::relation::EdgeSource::Curated,
                via,
            });
        }

        // Rimuovi `from` completamente
        self.remove_word(from);
    }

    /// Tutti i nodi presenti nel grafo (come soggetti o oggetti).
    pub fn all_nodes(&self) -> Vec<&str> {
        let mut nodes: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for k in self.outgoing.keys() { nodes.insert(k.as_str()); }
        for k in self.incoming.keys() { nodes.insert(k.as_str()); }
        nodes.into_iter().collect()
    }

    /// Tutti gli archi entranti verso `object` (qualunque relazione).
    /// Restituisce (relazione, soggetto, confidenza).
    pub fn all_incoming(&self, object: &str) -> Vec<(RelationType, &str, f32)> {
        match self.incoming.get(object) {
            None => vec![],
            Some(m) => {
                let mut result = Vec::new();
                for (rel, subjects) in m {
                    for subj in subjects {
                        // Recupera confidenza dal lato outgoing
                        let conf = self.outgoing.get(subj.as_str())
                            .and_then(|om| om.get(rel))
                            .and_then(|targets| targets.iter().find(|t| t.object == object))
                            .map(|t| t.confidence)
                            .unwrap_or(1.0);
                        result.push((*rel, subj.as_str(), conf));
                    }
                }
                result
            }
        }
    }

    /// Esiste un arco KG in qualunque direzione tra `a` e `b`?
    pub fn has_any_edge(&self, a: &str, b: &str) -> bool {
        self.outgoing.get(a).map(|m| m.values().any(|v| v.iter().any(|t| t.object == b))).unwrap_or(false)
        || self.outgoing.get(b).map(|m| m.values().any(|v| v.iter().any(|t| t.object == a))).unwrap_or(false)
    }

    /// Rimuove un arco specifico (soggetto, relazione, oggetto).
    pub fn remove_edge(&mut self, subject: &str, rel: RelationType, object: &str) {
        let mut removed = false;
        if let Some(rel_map) = self.outgoing.get_mut(subject) {
            if let Some(targets) = rel_map.get_mut(&rel) {
                let before = targets.len();
                targets.retain(|t| t.object != object);
                if targets.len() < before {
                    removed = true;
                }
            }
        }
        if let Some(rel_map) = self.incoming.get_mut(object) {
            if let Some(subjects) = rel_map.get_mut(&rel) {
                subjects.retain(|s| s != subject);
            }
        }
        if removed {
            self.edge_count = self.edge_count.saturating_sub(1);
        }
    }

    /// Aggiorna il campo `via` di un arco esistente.
    pub fn update_edge_via(&mut self, subject: &str, rel: RelationType, object: &str, new_via: Option<String>) -> bool {
        if let Some(rel_map) = self.outgoing.get_mut(subject) {
            if let Some(targets) = rel_map.get_mut(&rel) {
                if let Some(target) = targets.iter_mut().find(|t| t.object == object) {
                    target.via = new_via;
                    return true;
                }
            }
        }
        false
    }

    /// Aggiorna confidence e/o via di un arco. Ritorna true se trovato.
    pub fn update_edge(&mut self, subject: &str, rel: RelationType, object: &str, new_confidence: Option<f32>, new_via: Option<Option<String>>) -> bool {
        if let Some(rel_map) = self.outgoing.get_mut(subject) {
            if let Some(targets) = rel_map.get_mut(&rel) {
                if let Some(target) = targets.iter_mut().find(|t| t.object == object) {
                    if let Some(c) = new_confidence { target.confidence = c.clamp(0.0, 1.0); }
                    if let Some(v) = new_via { target.via = v; }
                    return true;
                }
            }
        }
        false
    }

    /// Aggiorna la confidence di un arco. Ritorna true se trovato e aggiornato.
    pub fn update_confidence(&mut self, subject: &str, rel: RelationType, object: &str, new_confidence: f32) -> bool {
        if let Some(rel_map) = self.outgoing.get_mut(subject) {
            if let Some(targets) = rel_map.get_mut(&rel) {
                if let Some(target) = targets.iter_mut().find(|t| t.object == object) {
                    target.confidence = new_confidence.clamp(0.0, 1.0);
                    return true;
                }
            }
        }
        false
    }

    /// Il nodo esiste nel grafo?
    pub fn contains(&self, word: &str) -> bool {
        self.outgoing.contains_key(word) || self.incoming.contains_key(word)
    }

    /// Grado uscente totale di un nodo (tutte le relazioni).
    pub fn out_degree(&self, word: &str) -> usize {
        self.outgoing.get(word)
            .map(|m| m.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }

    /// Grado entrante totale di un nodo (tutte le relazioni).
    pub fn in_degree(&self, word: &str) -> usize {
        self.incoming.get(word)
            .map(|m| m.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }

    /// Grado totale (entrante + uscente) di un nodo.
    pub fn total_degree(&self, word: &str) -> usize {
        self.out_degree(word) + self.in_degree(word)
    }

    // ─── Caricamento da TSV ──────────────────────────────────────────────────

    /// Carica triple da un file TSV.
    /// Formato: soggetto\tRELAZIONE\toggetto[\tconfidenza]
    /// Linee che iniziano con # sono commenti e vengono ignorate.
    pub fn load_from_tsv(&mut self, path: &Path) -> anyhow::Result<usize> {
        let content = fs::read_to_string(path)?;
        let mut count = 0usize;
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            match TypedEdge::from_tsv_line(trimmed) {
                Some(edge) => {
                    self.add_edge(edge);
                    count += 1;
                }
                None => {
                    // Segnala solo righe malformate non-commento
                    eprintln!("[KG] riga {} ignorata: {:?}", line_num + 1, trimmed);
                }
            }
        }
        Ok(count)
    }

    /// Carica tutti i file .tsv da una directory.
    pub fn load_from_dir(&mut self, dir: &Path) -> anyhow::Result<usize> {
        let mut total = 0usize;
        if !dir.exists() { return Ok(0); }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("tsv") {
                match self.load_from_tsv(&path) {
                    Ok(n) => { total += n; }
                    Err(e) => eprintln!("[KG] errore caricando {:?}: {}", path, e),
                }
            }
        }
        Ok(total)
    }

    // ─── Query strutturali ───────────────────────────────────────────────────

    /// Nodi che sono target di almeno `min_children` archi di tipo `rel`.
    /// Utile per trovare categorie: `categories_for(IsA, 2)` → ["animale", "nazione", ...]
    pub fn categories_for(&self, rel: RelationType, min_children: usize) -> Vec<String> {
        self.incoming.iter()
            .filter_map(|(node, rel_map)| {
                rel_map.get(&rel)
                    .filter(|children| children.len() >= min_children)
                    .map(|_| node.clone())
            })
            .collect()
    }

    /// Corteccia prefrontale topologica: IS_A upward = riconoscimento categoria, CAUSES = intento.
    ///
    /// Per ogni parola input, risale la catena IS_A (fino a 2 hop).
    /// Un nodo diventa attrattore se ha almeno `min_isa_children` figli IS_A entranti
    /// (cioè è una vera categoria, non un nodo terminale).
    /// I `causes` dell'attrattore dicono cosa l'entità dovrebbe fare.
    ///
    /// Esempio:
    ///   "ciao" → IS_A → "saluto" (20 figli IS_A) → CAUSES → "benvenuto"
    ///   "paura" → IS_A → "emozione" (80 figli IS_A) → CAUSES → "cautela"
    pub fn find_activated_attractors(
        &self,
        input_words: &[&str],
        min_isa_children: usize,
    ) -> Vec<AttractorHit> {
        use std::collections::HashMap;

        // concept → (score, source_words)
        let mut attractor_map: HashMap<&str, (f64, Vec<String>)> = HashMap::new();

        for &word in input_words {
            if word.len() < 3 { continue; }

            // Hop 1: direct IS_A parents
            for parent in self.query_objects(word, RelationType::IsA) {
                let n_children = self.query_subjects(parent, RelationType::IsA).len();
                if n_children >= min_isa_children {
                    let e = attractor_map.entry(parent).or_insert((0.0, Vec::new()));
                    e.0 += 1.0;
                    if !e.1.contains(&word.to_string()) { e.1.push(word.to_string()); }
                }
                // Hop 2: grandparent IS_A
                for grandparent in self.query_objects(parent, RelationType::IsA) {
                    let n_gc = self.query_subjects(grandparent, RelationType::IsA).len();
                    if n_gc >= min_isa_children {
                        let e = attractor_map.entry(grandparent).or_insert((0.0, Vec::new()));
                        e.0 += 0.6;  // decay per hop
                        if !e.1.contains(&word.to_string()) { e.1.push(word.to_string()); }
                    }
                }
            }
        }

        let mut attractors: Vec<AttractorHit> = attractor_map
            .into_iter()
            .map(|(concept, (score, sources))| {
                let causes = self.query_objects(concept, RelationType::Causes)
                    .into_iter()
                    .take(4)
                    .map(|s| s.to_string())
                    .collect();
                AttractorHit {
                    concept: concept.to_string(),
                    activation_score: score,
                    source_words: sources,
                    causes,
                }
            })
            .collect();

        attractors.sort_by(|a, b| b.activation_score.partial_cmp(&a.activation_score)
            .unwrap_or(std::cmp::Ordering::Equal));
        attractors
    }

    /// Nodi che hanno almeno `min_targets` archi uscenti di tipo `rel`.
    /// Utile per trovare cluster di similitudine: `nodes_with_min_outgoing(SimilarTo, 2)`.
    pub fn nodes_with_min_outgoing(&self, rel: RelationType, min_targets: usize) -> Vec<String> {
        self.outgoing.iter()
            .filter_map(|(node, rel_map)| {
                rel_map.get(&rel)
                    .filter(|targets| targets.len() >= min_targets)
                    .map(|_| node.clone())
            })
            .collect()
    }

    // ─── Serializzazione ─────────────────────────────────────────────────────

    /// Snapshot serializzabile.
    pub fn to_snapshot(&self) -> KgSnapshot {
        let mut edges = Vec::with_capacity(self.edge_count);
        for (subj, rel_map) in &self.outgoing {
            for (rel, targets) in rel_map {
                for t in targets {
                    edges.push(TypedEdge {
                        subject: subj.clone(),
                        relation: *rel,
                        object: t.object.clone(),
                        confidence: t.confidence,
                        source: t.source,
                        via: t.via.clone(),
                    });
                }
            }
        }
        KgSnapshot { edges }
    }

    pub fn from_snapshot(snap: KgSnapshot) -> Self {
        let mut kg = Self::new();
        for edge in snap.edges {
            kg.add_edge(edge);
        }
        kg
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self { Self::new() }
}

// ═══════════════════════════════════════════════════════════════════════════
// KgSnapshot — persistenza
// ═══════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════
// AttractorHit — risultato di IS_A traversal upward
// ═══════════════════════════════════════════════════════════════════════════

/// Un nodo attrattore raggiunto dall'input via catena IS_A.
/// Rappresenta la categoria pragmatica che l'entità ha riconosciuto.
/// I `causes` dicono cosa FARE in risposta.
#[derive(Debug, Clone)]
pub struct AttractorHit {
    /// Il concetto-categoria raggiunto (es. "saluto", "emozione", "domanda")
    pub concept: String,
    /// Score di attivazione: somma delle attivazioni delle parole sorgente
    pub activation_score: f64,
    /// Parole input che hanno raggiunto questo attrattore
    pub source_words: Vec<String>,
    /// CAUSES targets da questo attrattore — cosa l'entità dovrebbe fare
    pub causes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KgSnapshot {
    pub edges: Vec<TypedEdge>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::relation::RelationType;

    fn build_test_kg() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        kg.add("cane", RelationType::IsA, "animale");
        kg.add("gatto", RelationType::IsA, "animale");
        kg.add("animale", RelationType::IsA, "essere_vivente");
        kg.add("cane", RelationType::Does, "abbaiare");
        kg.add("cane", RelationType::Has, "pelo");
        kg.add("animale", RelationType::Does, "mangiare");
        kg.add("animale", RelationType::Does, "dormire");
        kg.add("germania", RelationType::IsA, "nazione");
        kg.add("nazione", RelationType::Has, "confine");
        kg.add("nazione", RelationType::Has, "capitale");
        kg.add("caldo", RelationType::OppositeOf, "freddo");
        kg
    }

    #[test]
    fn test_add_and_query() {
        let kg = build_test_kg();
        let is_a = kg.query_objects("cane", RelationType::IsA);
        assert!(is_a.contains(&"animale"), "cane IS-A animale");
        let does = kg.query_objects("cane", RelationType::Does);
        assert!(does.contains(&"abbaiare"), "cane DOES abbaiare");
    }

    #[test]
    fn test_inverse_query() {
        let kg = build_test_kg();
        let animals = kg.query_subjects("animale", RelationType::IsA);
        assert!(animals.contains(&"cane"));
        assert!(animals.contains(&"gatto"));
    }

    #[test]
    fn test_edge_count() {
        let kg = build_test_kg();
        assert!(kg.edge_count >= 10, "deve avere almeno 10 archi: {}", kg.edge_count);
    }

    #[test]
    fn test_no_duplicate_edges() {
        let mut kg = KnowledgeGraph::new();
        kg.add("cane", RelationType::IsA, "animale");
        kg.add("cane", RelationType::IsA, "animale"); // duplicato
        assert_eq!(kg.edge_count, 1, "non deve duplicare archi");
    }

    #[test]
    fn test_all_outgoing() {
        let kg = build_test_kg();
        let out = kg.all_outgoing("cane");
        assert!(out.len() >= 3, "cane ha almeno 3 archi: IS_A, DOES, HAS");
    }

    #[test]
    fn test_contains() {
        let kg = build_test_kg();
        assert!(kg.contains("cane"));
        assert!(kg.contains("animale"));
        assert!(!kg.contains("unicorno"));
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let kg = build_test_kg();
        let snap = kg.to_snapshot();
        let count = kg.edge_count;
        let restored = KnowledgeGraph::from_snapshot(snap);
        assert_eq!(restored.edge_count, count);
        let is_a = restored.query_objects("cane", RelationType::IsA);
        assert!(is_a.contains(&"animale"));
    }

    #[test]
    fn test_tsv_parse_inline() {
        let mut kg = KnowledgeGraph::new();
        // Simula lettura TSV linea per linea
        let lines = [
            "sole\tDOES\tbrillare\t1.0",
            "sole\tCAUSES\tluce",
            "# questo è un commento",
            "",
            "luna\tIS_A\tsatellite",
        ];
        for line in &lines {
            if let Some(edge) = TypedEdge::from_tsv_line(line) {
                kg.add_edge(edge);
            }
        }
        assert_eq!(kg.edge_count, 3);
        assert!(kg.query_objects("sole", RelationType::Does).contains(&"brillare"));
        assert!(kg.query_objects("sole", RelationType::Causes).contains(&"luce"));
    }
}
