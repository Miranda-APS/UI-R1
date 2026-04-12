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

    /// Come query_objects_weighted ma restituisce anche il campo `via` (ponte contestuale).
    /// Es: query_objects_with_via("fuoco", Causes) → [("calore", 0.90, Some("combustione"))]
    pub fn query_objects_with_via<'a>(&'a self, subject: &str, rel: RelationType) -> Vec<(&'a str, f32, Option<&'a str>)> {
        self.outgoing.get(subject)
            .and_then(|m| m.get(&rel))
            .map(|v| v.iter().map(|t| (t.object.as_str(), t.confidence, t.via.as_deref())).collect())
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

    /// Grado totale massimo su tutti i nodi del KG.
    /// Usato come normalizzatore nella derivazione firme 8D (dim Complessità).
    pub fn max_total_degree(&self) -> usize {
        let all_nodes: std::collections::HashSet<&str> = self.outgoing.keys()
            .map(|s| s.as_str())
            .chain(self.incoming.keys().map(|s| s.as_str()))
            .collect();
        all_nodes.iter()
            .map(|w| self.total_degree(w))
            .max()
            .unwrap_or(1)
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

            // Fattore specificità: attrattori con POCHI figli IS_A (specifici) vengono
            // preferiti a mega-attrattori come "azione" (5906 figli) o "qualita" (3503).
            // sweet_spot = 300 → "emozione" (209 figli) riceve punteggio pieno (>1.0),
            // "qualita" riceve ~0.086, "azione" riceve ~0.051.
            // Formula: specificity(n) = min(2.0, 300.0 / max(n, 1))
            let specificity = |n: usize| -> f64 {
                (300.0_f64 / (n.max(1) as f64)).min(2.0)
            };

            // Hop 1: direct IS_A parents
            for parent in self.query_objects(word, RelationType::IsA) {
                let n_children = self.query_subjects(parent, RelationType::IsA).len();
                if n_children >= min_isa_children {
                    let e = attractor_map.entry(parent).or_insert((0.0, Vec::new()));
                    e.0 += specificity(n_children);
                    if !e.1.contains(&word.to_string()) { e.1.push(word.to_string()); }
                }
                // Hop 2: grandparent IS_A
                for grandparent in self.query_objects(parent, RelationType::IsA) {
                    let n_gc = self.query_subjects(grandparent, RelationType::IsA).len();
                    if n_gc >= min_isa_children {
                        let e = attractor_map.entry(grandparent).or_insert((0.0, Vec::new()));
                        e.0 += 0.6 * specificity(n_gc);  // decay per hop + specificità
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

    // ─── Firma 8D KG-derivata ─────────────────────────────────────────────────

    /// Phase 63: Calcola la firma 8D di una parola dalla sua struttura nel grafo.
    ///
    /// Principio: la geometria IS il significato quando la "luce" è coerente.
    /// Le 8 dimensioni I Ching emergono dalle proprietà relazionali della parola —
    /// non dalle co-occorrenze statistiche con altri token testuali.
    ///
    /// Restituisce None se la parola non esiste nel KG.
    /// `max_degree`: grado massimo osservato nel grafo (per normalizzare Complessità).
    /// `valence_scores`: mappa parola → valenza pre-calcolata via BFS dalle radici emotive.
    pub fn derive_8d_from_kg(
        &self,
        word: &str,
        max_degree: usize,
        valence_scores: &std::collections::HashMap<String, f64>,
    ) -> Option<[f64; 8]> {
        if !self.contains(word) { return None; }

        let mut sig = [0.5f64; 8];

        // Contatori strutturali
        let causes_out = self.query_objects(word, crate::topology::relation::RelationType::Causes).len();
        let causes_in  = self.query_subjects(word, crate::topology::relation::RelationType::Causes).len();
        let isa_parents = self.query_objects(word, crate::topology::relation::RelationType::IsA).len();
        let isa_children = self.query_subjects(word, crate::topology::relation::RelationType::IsA).len();
        let has_opposite = !self.query_objects(word, crate::topology::relation::RelationType::OppositeOf).is_empty();
        let total_deg = self.total_degree(word);

        // ── Dim 0: Agency (☰ Cielo) ──────────────────────────────────────────
        // Quanto questa parola è AGENTE di cambiamento vs. passiva ricevente?
        // CAUSES outgoing = la parola produce effetti nel mondo → alto Agency
        // CAUSES incoming = la parola è effetto di altro → basso Agency
        // Le pure categorie (animale, qualità) hanno agency media — non causano, non sono causate
        let causes_total = causes_out + causes_in;
        sig[0] = if causes_total > 0 {
            (causes_out as f64 / causes_total as f64).clamp(0.05, 0.95)
        } else if isa_children > 5 {
            0.20  // categoria astratta con molti figli: permanente, non agente
        } else {
            0.50  // parola senza relazioni causali: agency neutra
        };

        // ── Dim 1: Permanenza (☷ Terra) ──────────────────────────────────────
        // Quanto questo concetto è stabile/immutabile nel mondo?
        // Categorie astratte con molti figli IS_A (emozione, animale) → permanenti
        // Concetti specifici, eventi, azioni → transitori
        sig[1] = if isa_children > 50 {
            0.85  // mega-categoria: molto permanente
        } else if isa_children > 10 {
            0.65  // categoria media
        } else if isa_children > 0 {
            0.40  // categoria piccola
        } else if causes_out > 3 {
            0.20  // agente attivo: poco permanente
        } else {
            0.35  // concetto specifico: transitorio
        };

        // ── Dim 2: Intensità (☳ Tuono) ────────────────────────────────────────
        // Quanto questa parola porta energia/forza/urgenza?
        // Proxy: parole con molti CAUSES outgoing + bassa permanenza → intense
        // Anche: parole che sono SIMILAR_TO ad altri nodi ad alta intensità
        // (propagato via valence_scores come secondo canale)
        let intensity_from_causes = if causes_out > 0 {
            ((causes_out as f64) / (causes_out as f64 + 3.0)).min(0.9)
        } else { 0.2 };
        // Boost se è un'emozione forte (prossima alle radici emotive)
        let valence = valence_scores.get(word).copied().unwrap_or(0.5);
        let emotional_intensity = (valence - 0.5).abs() * 2.0; // 0=neutro, 1=carico
        sig[2] = (intensity_from_causes * 0.6 + emotional_intensity * 0.4).clamp(0.05, 0.95);

        // ── Dim 3: Tempo (☵ Acqua) ────────────────────────────────────────────
        // Quanto questa parola è radicata in processi/flussi temporali?
        // Parole in catene causali (sia source che target) → alto Tempo
        // Categorie pure (IS_A hierarchy) → basso Tempo
        sig[3] = if causes_total > 0 {
            ((causes_total as f64) / (causes_total as f64 + 5.0)).min(0.9)
        } else if isa_children > 20 {
            0.15  // categoria statica: fuori dal tempo
        } else {
            0.35  // concetto con poca storia causale
        };

        // ── Dim 4: Confine (☶ Montagna) ───────────────────────────────────────
        // Quanto questo concetto è preciso/delimitato?
        // Pochi figli IS_A → specificità alta → alto Confine
        // Mega-categorie → molte cose ci appartengono → basso Confine
        // Avere OPPOSITE_OF = confine netto (sa cosa non è)
        let specificity = if isa_children == 0 {
            0.80  // foglia del grafo IS_A: massimamente specifico
        } else {
            (5.0 / (isa_children as f64 + 1.0)).min(0.75)
        };
        let polarity_bonus = if has_opposite { 0.15 } else { 0.0 };
        sig[4] = (specificity + polarity_bonus).clamp(0.05, 0.95);

        // ── Dim 5: Complessità (☴ Vento) ──────────────────────────────────────
        // Quanto questo nodo è connesso/intrecciato con altri?
        // Hub con molte relazioni → alta Complessità
        // Foglie con poche relazioni → bassa Complessità
        let max_deg_f = (max_degree.max(1)) as f64;
        sig[5] = if total_deg > 0 {
            ((total_deg as f64).ln() / max_deg_f.ln()).clamp(0.05, 0.95)
        } else { 0.05 };

        // ── Dim 6: Definizione (☲ Fuoco) ──────────────────────────────────────
        // Quanto questo concetto è ben definito/chiaro?
        // Molti genitori IS_A = localizzato precisamente nella tassonomia
        // Avere OPPOSITE_OF = polarità netta → alta Definizione
        // Parole prive di relazioni = indefinite
        let parents_contribution = (isa_parents as f64 / (isa_parents as f64 + 3.0)).min(0.7);
        let opposite_contribution = if has_opposite { 0.3 } else { 0.0 };
        sig[6] = (parents_contribution + opposite_contribution).clamp(0.05, 0.95);

        // ── Dim 7: Valenza (☱ Lago) ───────────────────────────────────────────
        // Carica emotiva/sociale della parola [0=negativo, 0.5=neutro, 1=positivo]
        // Derivata da BFS dalle radici emotive (pre-calcolato in valence_scores)
        sig[7] = valence_scores.get(word).copied().unwrap_or(0.5);

        Some(sig)
    }

    /// Phase 63: Propagazione BFS della valenza emotiva dal KG.
    ///
    /// Parte da radici semantiche positive e negative, propaga via SIMILAR_TO/IS_A/CAUSES
    /// con decadimento per hop. Restituisce una mappa parola → [0,1] dove:
    ///   0.0 = fortemente negativo (dolore/paura)
    ///   0.5 = neutro
    ///   1.0 = fortemente positivo (gioia/amore)
    ///
    /// Non è una lista hardcoded di sentimenti — è la geometria della rete
    /// che propaga la carica dalle radici conosciute verso la periferia.
    pub fn compute_valence_scores(&self) -> std::collections::HashMap<String, f64> {
        use std::collections::{HashMap, VecDeque};

        // Radici semantiche — non "lista di parole emotive" ma ancoraggi KG.
        // Le parole vicine ereditano la carica proporzionalmente alla distanza.
        const POS_ROOTS: &[(&str, f64)] = &[
            ("gioia", 1.0), ("felicità", 1.0), ("amore", 0.95), ("speranza", 0.90),
            ("piacere", 0.85), ("entusiasmo", 0.85), ("serenità", 0.85),
            ("gratitudine", 0.80), ("armonia", 0.80), ("fiducia", 0.80),
        ];
        const NEG_ROOTS: &[(&str, f64)] = &[
            ("dolore", -1.0), ("sofferenza", -1.0), ("paura", -0.95), ("tristezza", -0.95),
            ("angoscia", -0.95), ("rabbia", -0.85), ("ansia", -0.85),
            ("disperazione", -0.90), ("odio", -0.90), ("lutto", -0.85),
        ];

        // Propagazione: SIMILAR_TO (forte), IS_A (media), CAUSES (debole, bidirezionale)
        const SIMILAR_DECAY: f64 = 0.85;
        const ISA_DECAY:     f64 = 0.60;
        const CAUSES_DECAY:  f64 = 0.40;
        const MAX_HOPS: usize = 4;

        // scores: parola → carica cumulativa (somma pesata da tutti i cammini)
        // counts: parola → numero di cammini (per fare media)
        let mut scores: HashMap<String, f64> = HashMap::new();
        let mut counts: HashMap<String, usize> = HashMap::new();

        // BFS da ogni radice
        let all_roots: Vec<(&str, f64)> = POS_ROOTS.iter().chain(NEG_ROOTS.iter())
            .map(|&(w, v)| (w, v))
            .collect();

        for (root, charge) in &all_roots {
            // queue: (parola, carica_attuale, hop)
            let mut queue: VecDeque<(String, f64, usize)> = VecDeque::new();
            let mut visited: HashMap<String, f64> = HashMap::new();

            queue.push_back((root.to_string(), *charge, 0));
            visited.insert(root.to_string(), *charge);

            while let Some((word, current_charge, hop)) = queue.pop_front() {
                // Accumula questo cammino nella parola
                *scores.entry(word.clone()).or_insert(0.0) += current_charge;
                *counts.entry(word.clone()).or_insert(0) += 1;

                if hop >= MAX_HOPS { continue; }

                // Espandi via SIMILAR_TO (bidirezionale — la similarità non ha direzione)
                for neighbor in self.query_objects(&word, crate::topology::relation::RelationType::SimilarTo) {
                    let new_charge = current_charge * SIMILAR_DECAY;
                    if new_charge.abs() < 0.05 { continue; }
                    if !visited.contains_key(neighbor) {
                        visited.insert(neighbor.to_string(), new_charge);
                        queue.push_back((neighbor.to_string(), new_charge, hop + 1));
                    }
                }
                for neighbor in self.query_subjects(&word, crate::topology::relation::RelationType::SimilarTo) {
                    let new_charge = current_charge * SIMILAR_DECAY;
                    if new_charge.abs() < 0.05 { continue; }
                    if !visited.contains_key(neighbor) {
                        visited.insert(neighbor.to_string(), new_charge);
                        queue.push_back((neighbor.to_string(), new_charge, hop + 1));
                    }
                }

                // Espandi via IS_A verso i figli (gli specializzati ereditano dalla categoria)
                for child in self.query_subjects(&word, crate::topology::relation::RelationType::IsA) {
                    let new_charge = current_charge * ISA_DECAY;
                    if new_charge.abs() < 0.05 { continue; }
                    if !visited.contains_key(child) {
                        visited.insert(child.to_string(), new_charge);
                        queue.push_back((child.to_string(), new_charge, hop + 1));
                    }
                }

                // Espandi via CAUSES in avanti (le cause trasmettono valenza agli effetti)
                for effect in self.query_objects(&word, crate::topology::relation::RelationType::Causes) {
                    let new_charge = current_charge * CAUSES_DECAY;
                    if new_charge.abs() < 0.05 { continue; }
                    if !visited.contains_key(effect) {
                        visited.insert(effect.to_string(), new_charge);
                        queue.push_back((effect.to_string(), new_charge, hop + 1));
                    }
                }
            }
        }

        // OPPOSITE_OF inverte la valenza: se "gioia" è 1.0, "tristezza" via OPPOSITE_OF è -1.0
        // (già catturato dalle radici dirette, ma gestisce casi non in radici)
        let opposite_words: Vec<String> = scores.keys().cloned().collect();
        for word in &opposite_words {
            let word_score = *scores.get(word.as_str()).unwrap_or(&0.0);
            if word_score.abs() < 0.1 { continue; }
            for opp in self.query_objects(word, crate::topology::relation::RelationType::OppositeOf) {
                let inv = -word_score * 0.9;
                let existing = scores.get(opp).copied().unwrap_or(0.0);
                // Prendi il valore più estremo (non fare media se già opposto)
                if inv.abs() > existing.abs() {
                    scores.insert(opp.to_string(), inv);
                    counts.insert(opp.to_string(), 1);
                }
            }
        }

        // Normalizza: media dei cammini → [0, 1] (da [-1, +1])
        let mut result = HashMap::new();
        for (word, total) in &scores {
            let n = counts.get(word).copied().unwrap_or(1) as f64;
            let avg = (total / n).clamp(-1.0, 1.0);
            // Mappa [-1, +1] → [0, 1]
            result.insert(word.clone(), (avg + 1.0) / 2.0);
        }

        result
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

#[derive(Debug, Clone, Serialize, Default)]
pub struct KgSnapshot {
    pub edges: Vec<TypedEdge>,
}

impl<'de> serde::Deserialize<'de> for KgSnapshot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use crate::topology::relation::{EdgeSource, RelationType};
        use serde::Deserialize as _;

        // Struttura intermedia con `relation` come stringa — tollera varianti sconosciute.
        #[derive(serde::Deserialize)]
        struct RawEdge {
            subject: String,
            relation: String,
            object: String,
            #[serde(default = "default_conf")]
            confidence: f32,
            #[serde(default)]
            source: EdgeSource,
            #[serde(default)]
            via: Option<String>,
        }
        fn default_conf() -> f32 { 1.0 }

        #[derive(serde::Deserialize)]
        struct RawSnap { edges: Vec<RawEdge> }

        let raw = RawSnap::deserialize(deserializer)?;
        let mut skipped = 0usize;
        let mut edges = Vec::with_capacity(raw.edges.len());
        for e in raw.edges {
            match RelationType::from_str(&e.relation) {
                Some(rel) => edges.push(crate::topology::relation::TypedEdge {
                    subject:    e.subject,
                    relation:   rel,
                    object:     e.object,
                    confidence: e.confidence,
                    source:     e.source,
                    via:        e.via,
                }),
                None => { skipped += 1; }
            }
        }
        if skipped > 0 {
            eprintln!("[KG] {} archi con relazione sconosciuta saltati", skipped);
        }
        Ok(KgSnapshot { edges })
    }
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
