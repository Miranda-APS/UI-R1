/// WordTopology — Il campo topologico fatto di parole.
///
/// Le parole sono i vertici. Le co-occorrenze sono gli archi.
/// I frattali emergono come regioni dense del campo.
/// La propagazione attraversa le connessioni tra parole.
///
/// Questa struttura e il substrato PRIMARIO di Prometeo:
/// l'input attiva parole → la propagazione illumina il vicinato →
/// le attivazioni frattali emergono per aggregazione → il resto del sistema
/// (navigazione, volonta, composti) lavora sulle attivazioni emergenti.

use std::collections::{HashMap, HashSet};
use crate::topology::fractal::FractalId;
use crate::topology::lexicon::Lexicon;
use crate::topology::relation::RelationType;
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::inference::InferenceEngine;

/// Identificatore unico per un vertice-parola nel campo.
pub type WordId = u32;

/// Un vertice nel campo topologico: una parola con la sua attivazione corrente.
#[derive(Debug, Clone)]
pub struct WordVertex {
    pub id: WordId,
    /// La parola (lowercase)
    pub word: String,
    /// Attivazione corrente [0.0, 1.0]
    pub activation: f64,
    /// Quante volte e stata attivata nel campo
    pub activation_count: u64,
}

/// Un arco nel campo: connessione tra due parole.
///
/// Un arco può nascere da due fonti:
///   - Co-occorrenza statistica (da testo — rumorosa, Wikipedia)
///   - Relazione logica tipata dal KG (IS_A, HAS, DOES, CAUSES, ...)
///
/// La FASE codifica il tipo di relazione:
///   0     = risonanza pura  (SIMILAR_TO: ciao ↔ saluto)
///   ~0.1  = molto vicini    (IS_A: cane ↔ animale)
///   ~0.2  = vicini          (HAS/DOES: cane ↔ abbaiare)
///   ~0.35 = relati          (CAUSES: paura ↔ tremore)
///   PI/2  = neutro          (co-occorrenza generica)
///   PI    = opposizione     (OPPOSITE_OF: caldo ↔ freddo)
///
/// La propagazione usa cos(phase): formula unica, nessun branching.
#[derive(Debug, Clone)]
pub struct WordEdge {
    pub a: WordId,
    pub b: WordId,
    /// Peso della connessione [0.0, 1.0]
    pub weight: f64,
    /// Conteggio grezzo (co-occorrenze) o 0 per archi KG
    pub raw_count: u64,
    /// Fase della relazione [0, PI] radianti
    pub phase: f64,
    /// Tipo di relazione logica (None = co-occorrenza statistica)
    pub relation: Option<RelationType>,
}

/// Il campo topologico delle parole.
///
/// Struttura dati principale: un grafo pesato dove i vertici sono parole
/// e gli archi rappresentano relazioni di co-occorrenza apprese.
/// Le attivazioni si propagano lungo gli archi.
#[derive(Debug)]
pub struct WordTopology {
    /// Vertici: word_id → vertice
    vertices: HashMap<WordId, WordVertex>,
    /// Archi: (id_minore, id_maggiore) → arco
    edges: HashMap<(WordId, WordId), WordEdge>,
    /// Indice: parola → word_id
    word_to_id: HashMap<String, WordId>,
    /// Indice inverso: word_id → lista archi (vicini)
    adjacency: HashMap<WordId, Vec<WordId>>,
    /// Prossimo ID disponibile
    next_id: WordId,
    /// Soglia di attivazione minima per propagazione
    pub activation_threshold: f64,
    /// Peso massimo tra tutti gli archi (per normalizzazione)
    max_edge_weight: f64,
}

impl WordTopology {
    /// Crea un campo vuoto.
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: HashMap::new(),
            word_to_id: HashMap::new(),
            adjacency: HashMap::new(),
            next_id: 0,
            activation_threshold: 0.02,
            max_edge_weight: 1.0,
        }
    }

    /// Costruisce il campo topologico dal lessico esistente.
    /// Converte parole in vertici e co-occorrenze in archi.
    pub fn build_from_lexicon(lexicon: &Lexicon) -> Self {
        let mut topo = Self::new();

        // Fase 1: crea tutti i vertici
        for (word, _pattern) in lexicon.patterns_iter() {
            topo.add_word(word);
        }

        // Fase 2: crea gli archi dalle co-occorrenze
        // Raccogli prima il max per normalizzazione
        let mut global_max_count: u64 = 1;
        for (_word, pattern) in lexicon.patterns_iter() {
            for (_other, &count) in &pattern.co_occurrences {
                if count > global_max_count {
                    global_max_count = count;
                }
            }
        }

        let log_max = (global_max_count as f64).ln().max(1.0);

        for (word, pattern) in lexicon.patterns_iter() {
            let word_id = match topo.word_to_id.get(word) {
                Some(&id) => id,
                None => continue,
            };

            for (other, &count) in &pattern.co_occurrences {
                // Crea arco solo se l'altra parola esiste come vertice
                let other_id = match topo.word_to_id.get(other) {
                    Some(&id) => id,
                    None => continue,
                };

                // Evita duplicati: arco solo nella direzione id_minore → id_maggiore
                if word_id >= other_id {
                    continue;
                }

                // Peso IDF-like: co-occorrenze rare pesano proporzionalmente di piu
                // count=1 → ~0, count=max → ~1.0
                let weight = if count > 0 {
                    ((count as f64).ln() / log_max).clamp(0.01, 1.0)
                } else {
                    0.0
                };

                if weight > 0.0 {
                    topo.add_edge(word_id, other_id, weight, count);
                }
            }
        }

        topo.max_edge_weight = 1.0; // gia normalizzato
        topo
    }

    /// Costruisce archi SEMANTICI dai tipi di relazione del Knowledge Graph.
    ///
    /// Ogni relazione logica diventa un arco con fase precisa:
    ///   SIMILAR_TO  → phase 0.00 (risonanza pura — quasi sinonimi)
    ///   IS_A        → phase 0.10 (molto vicini — iperonimo)
    ///   PART_OF     → phase 0.15 (vicini — composizione)
    ///   HAS         → phase 0.20 (vicini — attributo)
    ///   DOES        → phase 0.20 (vicini — azione)
    ///   CAUSES      → phase 0.35 (relati — causalità)
    ///   USED_FOR    → phase 0.30 (relati — funzione)
    ///   OPPOSITE_OF → phase PI   (opposizione — inibitorio)
    ///
    /// IS_A è transitivo: cane→mammifero (0.80), cane→animale (0.55),
    /// cane→essere_vivente (0.36). La distanza tassonomica riduce il peso.
    ///
    /// Gli archi KG SOVRASCRIVONO archi co-occorrenza se il peso KG è maggiore.
    /// Questo rende il KG l'autorità semantica senza perdere dati pregressi.
    ///
    /// Ritorna (nuovi_archi, archi_rafforzati).
    pub fn build_from_knowledge_graph(&mut self, kg: &KnowledgeGraph) -> (usize, usize) {
        let inference = InferenceEngine::new(kg);
        let mut added = 0usize;
        let mut strengthened = 0usize;

        // Fase per ogni tipo di relazione
        let phase_for = |rel: RelationType| -> f64 {
            match rel {
                RelationType::SimilarTo | RelationType::Equivalent => 0.00,
                RelationType::IsA        => 0.10,
                RelationType::PartOf     => 0.15,
                RelationType::Has        => 0.20,
                RelationType::Does       => 0.20,
                RelationType::UsedFor    => 0.30,
                RelationType::Causes | RelationType::Implies => 0.35,
                RelationType::Enables    => 0.25,
                RelationType::Requires   => 0.28,
                RelationType::TransformsInto => 0.40,
                RelationType::Expresses  => 0.22,
                RelationType::Symbolizes => 0.18,
                RelationType::ContextOf  => 0.12,
                RelationType::Coexists   => 0.05,
                RelationType::Excludes   => std::f64::consts::FRAC_PI_2,
                RelationType::OppositeOf => std::f64::consts::PI,
                // Fenomenologiche
                RelationType::FeelsAs    => 0.05,  // Intima risonanza
                RelationType::WondersAbout => 0.15, // Tensione esplorativa
                RelationType::RemembersAs  => 0.10, // Risonanza episodica
                RelationType::DerivesFrom  => 0.08, // Famiglia derivazionale: vicina
            }
        };

        // Peso base per tipo — modulato poi da confidence e hub_damping per-arco
        let type_base = |rel: RelationType| -> f64 {
            match rel {
                RelationType::SimilarTo  => 0.90,
                RelationType::Equivalent => 0.92,
                RelationType::IsA        => 0.80,
                RelationType::PartOf     => 0.75,
                RelationType::Has        => 0.70,
                RelationType::Does       => 0.70,
                RelationType::UsedFor    => 0.55,
                RelationType::Causes     => 0.65,
                RelationType::Enables    => 0.60,
                RelationType::Requires   => 0.58,
                RelationType::TransformsInto => 0.62,
                RelationType::Expresses  => 0.65,
                RelationType::Symbolizes => 0.55,
                RelationType::ContextOf  => 0.50,
                RelationType::Implies    => 0.68,
                RelationType::Coexists   => 0.60,
                RelationType::Excludes   => 0.45,
                RelationType::OppositeOf => 0.50,
                // Fenomenologiche
                RelationType::FeelsAs    => 0.85,
                RelationType::WondersAbout => 0.70,
                RelationType::RemembersAs  => 0.80,
                RelationType::DerivesFrom  => 0.82, // Legame forte alla base
            }
        };

        // ── Phase 48: Hub damping ──────────────────────────────────
        // Calcola il grado totale di ogni nodo nel KG.
        // Nodi hub (essere, avere, fare) con migliaia di archi vengono
        // smorzati logaritmicamente: peso_effettivo = base × confidence × hub_factor.
        // hub_factor = 1.0 / (1.0 + ln(max(deg_a, deg_b) / median_degree).max(0))
        // Questo dà peso diverso ad OGNI singolo arco, non costante per tipo.
        let all_words: Vec<String> = self.word_to_id.keys().cloned().collect();

        // Pre-calcola gradi per tutte le parole nel lessico
        let degrees: Vec<(String, usize)> = all_words.iter()
            .map(|w| (w.clone(), kg.total_degree(w)))
            .filter(|(_, d)| *d > 0)
            .collect();

        // Mediana dei gradi (solo parole con archi KG)
        let median_degree = if degrees.is_empty() {
            1.0_f64
        } else {
            let mut sorted_degrees: Vec<usize> = degrees.iter().map(|(_, d)| *d).collect();
            sorted_degrees.sort_unstable();
            sorted_degrees[sorted_degrees.len() / 2] as f64
        };

        // Lookup rapido grado → f64
        let degree_map: std::collections::HashMap<&str, f64> = degrees.iter()
            .map(|(w, d)| (w.as_str(), *d as f64))
            .collect();

        // Hub damping: penalizza nodi con grado >> mediana
        let hub_factor = |word_a: &str, word_b: &str| -> f64 {
            let deg_a = degree_map.get(word_a).copied().unwrap_or(1.0);
            let deg_b = degree_map.get(word_b).copied().unwrap_or(1.0);
            let max_deg = deg_a.max(deg_b);
            // Solo nodi sopra la mediana vengono penalizzati
            let ratio = (max_deg / median_degree).max(1.0);
            1.0 / (1.0 + ratio.ln())
        };

        for word in &all_words {
            let id_a = match self.word_to_id.get(word.as_str()) {
                Some(&id) => id,
                None => continue,
            };

            // 1. Archi diretti per ogni relazione
            for (rel, target, confidence) in kg.all_outgoing(word) {
                // Phase 86 (decisione #4a): `DerivesFrom` è navigazione morfologica,
                // NON propagazione di campo. Non genera un arco topologico: attivare
                // "pulire" non deve accendere "pulizia/pulito" via la famiglia (sporcherebbe
                // il campo). Resta attraversabile dal pathfinding (comprehension_path
                // legge il KG direttamente), ma non semina la spreading-activation.
                if rel == RelationType::DerivesFrom { continue; }
                let id_b = match self.word_to_id.get(target) {
                    Some(&id) => id,
                    None => continue, // target non nel lessico: skip
                };
                if id_a == id_b { continue; }

                // Phase 48: peso = base_tipo × confidence_arco × hub_damping
                let hf = hub_factor(word, target);
                let weight = (type_base(rel) * confidence as f64 * hf).clamp(0.01, 1.0);
                let phase = phase_for(rel);
                let key = if id_a < id_b { (id_a, id_b) } else { (id_b, id_a) };

                let existing = self.edges.get(&key).map(|e| e.weight).unwrap_or(0.0);
                if weight > existing {
                    if existing > 0.0 { strengthened += 1; } else { added += 1; }
                    self.edges.insert(key, WordEdge {
                        a: key.0, b: key.1,
                        weight, raw_count: 0,
                        phase, relation: Some(rel),
                    });
                    if let Some(adj) = self.adjacency.get_mut(&key.0) {
                        if !adj.contains(&key.1) { adj.push(key.1); }
                    }
                    if let Some(adj) = self.adjacency.get_mut(&key.1) {
                        if !adj.contains(&key.0) { adj.push(key.0); }
                    }
                }
            }

            // 2. IS_A transitivo: cane→animale (2hop), cane→essere_vivente (3hop)
            let ancestors = inference.type_chain(word);
            for (depth, ancestor) in ancestors.iter().enumerate() {
                let depth = depth + 1; // depth 0 = diretto (già gestito sopra), 1 = nonno...
                if depth == 0 { continue; } // diretto già sopra
                let id_anc = match self.word_to_id.get(ancestor.as_str()) {
                    Some(&id) => id,
                    None => continue,
                };
                if id_a == id_anc { continue; }

                // Peso decade con profondità × hub damping
                let hf = hub_factor(word, ancestor);
                let weight = (0.80_f64 * 0.65_f64.powi(depth as i32) * hf).clamp(0.01, 1.0);
                // Fase leggermente meno risonante con profondità
                let phase = (0.10 + 0.05 * depth as f64).min(std::f64::consts::FRAC_PI_2);
                let key = if id_a < id_anc { (id_a, id_anc) } else { (id_anc, id_a) };

                let existing = self.edges.get(&key).map(|e| e.weight).unwrap_or(0.0);
                if weight > existing {
                    if existing > 0.0 { strengthened += 1; } else { added += 1; }
                    self.edges.insert(key, WordEdge {
                        a: key.0, b: key.1,
                        weight, raw_count: 0,
                        phase, relation: Some(RelationType::IsA),
                    });
                    if let Some(adj) = self.adjacency.get_mut(&key.0) {
                        if !adj.contains(&key.1) { adj.push(key.1); }
                    }
                    if let Some(adj) = self.adjacency.get_mut(&key.1) {
                        if !adj.contains(&key.0) { adj.push(key.0); }
                    }
                }
            }
        }

        self.max_edge_weight = 1.0;
        (added, strengthened)
    }

    /// Rimuove tutti gli archi derivati da co-occorrenze statistiche.
    /// Mantiene solo gli archi con relazione KG esplicita.
    /// Usato da `rebuild-semantic-topology` per pulizia Wikipedia.
    pub fn clear_statistical_edges(&mut self) -> usize {
        let before = self.edges.len();
        self.edges.retain(|_, e| e.relation.is_some());

        // Ricostruisce l'indice di adiacenza da zero
        for adj in self.adjacency.values_mut() {
            adj.clear();
        }
        for (&(a, b), _) in &self.edges {
            if let Some(adj) = self.adjacency.get_mut(&a) {
                if !adj.contains(&b) { adj.push(b); }
            }
            if let Some(adj) = self.adjacency.get_mut(&b) {
                if !adj.contains(&a) { adj.push(a); }
            }
        }

        before - self.edges.len()
    }

    /// Statistiche sugli archi: totali, semantici (KG), statistici (co-occorrenza).
    pub fn edge_stats(&self) -> (usize, usize, usize) {
        let semantic = self.edges.values().filter(|e| e.relation.is_some()).count();
        let total = self.edges.len();
        (total, semantic, total - semantic)
    }

    /// Aggiunge una parola come vertice. Restituisce il WordId assegnato.
    /// Se la parola esiste gia, restituisce il suo ID.
    pub fn add_word(&mut self, word: &str) -> WordId {
        let lower = word.to_lowercase();
        if let Some(&existing) = self.word_to_id.get(&lower) {
            return existing;
        }

        let id = self.next_id;
        self.next_id += 1;

        self.vertices.insert(id, WordVertex {
            id,
            word: lower.clone(),
            activation: 0.0,
            activation_count: 0,
        });

        self.word_to_id.insert(lower, id);
        self.adjacency.insert(id, Vec::new());
        id
    }

    /// Aggiunge o aggiorna un arco tra due parole (fase default = PI/2, neutro).
    fn add_edge(&mut self, a: WordId, b: WordId, weight: f64, raw_count: u64) {
        self.add_edge_with_phase(a, b, weight, raw_count, std::f64::consts::FRAC_PI_2);
    }

    /// Aggiunge o aggiorna un arco con fase esplicita.
    /// phase in [0, PI]: 0 = risonanza, PI/2 = neutro, PI = opposizione.
    /// Pubblico per permettere la ricostruzione del campo da vicini PF1 (Phase 39).
    pub fn add_edge_with_phase(&mut self, a: WordId, b: WordId, weight: f64, raw_count: u64, phase: f64) {
        let key = if a < b { (a, b) } else { (b, a) };

        // Se l'arco esiste gia, preserva la fase se viene passato il default (PI/2)
        let neutral = std::f64::consts::FRAC_PI_2;
        let existing_phase = if (phase - neutral).abs() < 0.001 {
            self.edges.get(&key).map(|e| e.phase).unwrap_or(neutral)
        } else {
            phase
        };

        // Preserva la relazione logica se esistente
        let existing_relation = self.edges.get(&key).and_then(|e| e.relation);

        self.edges.insert(key, WordEdge {
            a: key.0,
            b: key.1,
            weight,
            raw_count,
            phase: existing_phase,
            relation: existing_relation,
        });

        // Aggiorna lista adiacenza
        if let Some(adj) = self.adjacency.get_mut(&key.0) {
            if !adj.contains(&key.1) {
                adj.push(key.1);
            }
        }
        if let Some(adj) = self.adjacency.get_mut(&key.1) {
            if !adj.contains(&key.0) {
                adj.push(key.0);
            }
        }
    }

    /// Aggiunge (o rinforza) un arco KG tipato da connessione curata dall'utente.
    /// Se la parola non è nel lessico viene aggiunta. Se l'arco KG è più forte del
    /// corrente, lo sovrascrive mantenendo tipo e fase semantica corretti.
    pub fn add_edge_from_kg(&mut self, word_a: &str, word_b: &str, rel: crate::topology::relation::RelationType) {
        use crate::topology::relation::RelationType as RT;
        let phase = match rel {
            RT::SimilarTo | RT::Equivalent => 0.00,
            RT::IsA        => 0.10,
            RT::PartOf     => 0.15,
            RT::Has        => 0.20,
            RT::Does       => 0.20,
            RT::UsedFor    => 0.30,
            RT::Causes | RT::Implies => 0.35,
            RT::Enables    => 0.25,
            RT::Requires   => 0.28,
            RT::TransformsInto => 0.40,
            RT::Expresses  => 0.22,
            RT::Symbolizes => 0.18,
            RT::ContextOf  => 0.12,
            RT::Coexists   => 0.05,
            RT::Excludes   => std::f64::consts::FRAC_PI_2,
            RT::OppositeOf => std::f64::consts::PI,
            // Fenomenologiche
            RT::FeelsAs    => 0.05,  // Molto intimo, risonanza forte
            RT::WondersAbout => 0.15, // Tensione esplorativa leggera
            RT::RemembersAs  => 0.10, // Risonanza della memoria
            RT::DerivesFrom  => 0.08, // Famiglia derivazionale
        };
        let weight = match rel {
            RT::SimilarTo  => 0.90,
            RT::Equivalent => 0.92,
            RT::IsA        => 0.80,
            RT::PartOf     => 0.75,
            RT::Has        => 0.70,
            RT::Does       => 0.70,
            RT::UsedFor    => 0.55,
            RT::Causes     => 0.65,
            RT::Enables    => 0.60,
            RT::Requires   => 0.58,
            RT::TransformsInto => 0.62,
            RT::Expresses  => 0.65,
            RT::Symbolizes => 0.55,
            RT::ContextOf  => 0.50,
            RT::Implies    => 0.68,
            RT::Coexists   => 0.60,
            RT::Excludes   => 0.45,
            RT::OppositeOf => 0.50,
            // Fenomenologiche
            RT::FeelsAs    => 0.85,
            RT::WondersAbout => 0.70,
            RT::RemembersAs  => 0.80,
            RT::DerivesFrom  => 0.82,
        };
        let id_a = self.add_word(word_a);
        let id_b = self.add_word(word_b);
        if id_a == id_b { return; }
        let key = if id_a < id_b { (id_a, id_b) } else { (id_b, id_a) };
        let existing_w = self.edges.get(&key).map(|e| e.weight).unwrap_or(0.0);
        if weight > existing_w {
            self.edges.insert(key, WordEdge { a: key.0, b: key.1, weight, raw_count: 0, phase, relation: Some(rel) });
            if let Some(adj) = self.adjacency.get_mut(&key.0) { if !adj.contains(&key.1) { adj.push(key.1); } }
            if let Some(adj) = self.adjacency.get_mut(&key.1) { if !adj.contains(&key.0) { adj.push(key.0); } }
        }
    }

    /// Aggiorna un arco da nuova co-occorrenza (dopo teach).
    pub fn update_edge_from_cooccurrence(&mut self, word_a: &str, word_b: &str, count: u64) {
        let id_a = match self.word_to_id.get(&word_a.to_lowercase()) {
            Some(&id) => id,
            None => return,
        };
        let id_b = match self.word_to_id.get(&word_b.to_lowercase()) {
            Some(&id) => id,
            None => return,
        };

        // Ricalcola peso — usiamo la scala corrente
        let weight = if count > 0 {
            ((count as f64).ln() / (self.max_edge_weight.max(1.0)).ln().max(1.0)).clamp(0.01, 1.0)
        } else {
            0.0
        };

        self.add_edge(id_a, id_b, weight, count);
    }

    // ==================== ATTIVAZIONE ====================

    /// Attiva una parola nel campo.
    pub fn activate_word(&mut self, word: &str, strength: f64) {
        let lower = word.to_lowercase();
        if let Some(&id) = self.word_to_id.get(&lower) {
            if let Some(vertex) = self.vertices.get_mut(&id) {
                vertex.activation = (vertex.activation + strength).min(1.0);
                vertex.activation_count += 1;
            }
        }
    }

    /// Attiva una parola per ID.
    pub fn activate_word_id(&mut self, id: WordId, strength: f64) {
        if let Some(vertex) = self.vertices.get_mut(&id) {
            vertex.activation = (vertex.activation + strength).min(1.0);
            vertex.activation_count += 1;
        }
    }

    /// Propagazione dell'attivazione nel campo.
    /// L'attivazione si diffonde dalle parole attive ai loro vicini,
    /// pesata dal peso dell'arco e smorzata ad ogni passo.
    ///
    /// La FASE dell'arco determina il tipo di propagazione tramite cos(phase):
    ///   cos(0)    = +1 → risonanza (propagazione concorde)
    ///   cos(PI/2) =  0 → tensione creativa (nessuna propagazione)
    ///   cos(PI)   = -1 → opposizione (propagazione inversa)
    ///
    /// Una formula unica, nessun branching.
    pub fn propagate(&mut self, steps: usize) {
        for step in 0..steps {
            let step_damping = 0.15 / (1.0 + step as f64);

            // Raccogli sorgenti attive
            let active_sources: Vec<(WordId, f64)> = self.vertices.values()
                .filter(|v| v.activation > self.activation_threshold)
                .map(|v| (v.id, v.activation))
                .collect();

            let mut deltas: Vec<(WordId, f64)> = Vec::new();

            for (src_id, src_activation) in &active_sources {
                if let Some(neighbors) = self.adjacency.get(src_id) {
                    for &neighbor_id in neighbors {
                        let key = if *src_id < neighbor_id {
                            (*src_id, neighbor_id)
                        } else {
                            (neighbor_id, *src_id)
                        };

                        let edge = match self.edges.get(&key) {
                            Some(e) => e,
                            None => continue,
                        };

                        // Formula unica: cos(phase) determina segno e intensita
                        let propagated = src_activation * step_damping * edge.weight * edge.phase.cos();

                        if propagated.abs() < 0.001 {
                            continue;
                        }

                        if propagated > 0.0 {
                            // Risonanza: attiva solo vertici sotto soglia (evita retroazione)
                            if let Some(n) = self.vertices.get(&neighbor_id) {
                                if n.activation < self.activation_threshold {
                                    deltas.push((neighbor_id, propagated));
                                }
                            }
                        } else {
                            // Opposizione: deattiva (funziona a qualsiasi livello)
                            deltas.push((neighbor_id, propagated));
                        }
                    }
                }
            }

            // Applica tutti i delta in un colpo
            for (id, delta) in deltas {
                if let Some(vertex) = self.vertices.get_mut(&id) {
                    vertex.activation = (vertex.activation + delta).clamp(0.0, 1.0);
                }
            }
        }
    }

    /// Stato di riposo: seme iniziale di attivazione proporzionale alla stability.
    ///
    /// Chiamato dopo build_from_lexicon() al restore dello stato salvato.
    /// L'entita "si sveglia" con le sue parole piu familiari gia presenti
    /// nel campo — non aspetta input esterno per esistere.
    ///
    /// Formula: activation = stability * 0.08
    ///   stability=0.25 → 0.020 (minimo percepibile — parola incontrata poche volte)
    ///   stability=0.50 → 0.040 (moderatamente stabile)
    ///   stability=0.70 → 0.056 (ben presente)
    ///   stability=0.95 → 0.076 (massimo per stato di riposo)
    ///
    /// Senza propagazione: ogni parola ha attivazione proporzionale alla propria
    /// stabilità. Il campo è discriminante — non saturo.
    /// La propagazione avviene naturalmente al primo input reale.
    pub fn seed_resting_state(&mut self, lexicon: &Lexicon) {
        // Semina solo parole ben integrate nel grafo semantico.
        // MIN_SEED_ARCS=1: funziona sia con topologia PF1 (max 8 archi/parola)
        // sia con topologia co_occ (molti archi). Il filtro qualità è la stability > 0.20.
        // Con 5M archi e 128K parole (media ~39/parola), la soglia era 100 (rumore wiki);
        // fanno parte del vocabolario "vissuto" di Prometeo.
        // MIN_SEED_ARCS=1: parole con almeno 1 arco (evita isolati puri).
        // Con topologia PF1 (max 8 archi) o co_occ (>100), la stability è il filtro principale.
        const MIN_SEED_ARCS: usize = 1;
        for (word, pattern) in lexicon.patterns_iter() {
            if pattern.stability > 0.20 {
                if let Some(id) = self.word_id(word) {
                    let arc_count = self.adjacency.get(&id)
                        .map_or(0, |ns| ns.len());
                    if arc_count < MIN_SEED_ARCS {
                        continue;
                    }
                } else {
                    continue;
                }
                let initial_activation = pattern.stability * 0.003;
                self.activate_word(word, initial_activation);
            }
        }
    }

    /// Decadimento globale delle attivazioni.
    pub fn decay_all(&mut self, rate: f64) {
        for vertex in self.vertices.values_mut() {
            vertex.activation *= 1.0 - rate;
            if vertex.activation < 0.001 {
                vertex.activation = 0.0;
            }
        }
    }

    // ==================== QUERY ====================

    /// Parole piu attive nel campo.
    pub fn most_active(&self, limit: usize) -> Vec<&WordVertex> {
        let mut active: Vec<&WordVertex> = self.vertices.values()
            .filter(|v| v.activation > self.activation_threshold)
            .collect();
        active.sort_by(|a, b| b.activation.partial_cmp(&a.activation).unwrap());
        active.truncate(limit);
        active
    }

    /// Parole attive con le loro attivazioni, come coppie (parola, attivazione).
    pub fn active_words(&self) -> Vec<(&str, f64)> {
        self.vertices.values()
            .filter(|v| v.activation > self.activation_threshold)
            .map(|v| (v.word.as_str(), v.activation))
            .collect()
    }

    /// Vicini attivi di una parola.
    pub fn active_neighbors(&self, word: &str) -> Vec<(&str, f64)> {
        let lower = word.to_lowercase();
        let id = match self.word_to_id.get(&lower) {
            Some(&id) => id,
            None => return Vec::new(),
        };

        let neighbors = match self.adjacency.get(&id) {
            Some(n) => n,
            None => return Vec::new(),
        };

        neighbors.iter()
            .filter_map(|&nid| {
                let v = self.vertices.get(&nid)?;
                if v.activation > self.activation_threshold {
                    Some((v.word.as_str(), v.activation))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Vicini con peso più alto (per diagnosi e display). Non usa attivazione.
    pub fn top_active_neighbors(&self, word: &str, limit: usize) -> Vec<(String, f64)> {
        let lower = word.to_lowercase();
        let id = match self.word_to_id.get(&lower) {
            Some(&id) => id,
            None => return Vec::new(),
        };
        let neighbors = match self.adjacency.get(&id) {
            Some(n) => n,
            None => return Vec::new(),
        };
        let mut result: Vec<(String, f64)> = neighbors.iter().filter_map(|&nid| {
            let key = if id < nid { (id, nid) } else { (nid, id) };
            let edge = self.edges.get(&key)?;
            let word_str = self.vertices.get(&nid)?.word.clone();
            Some((word_str, edge.weight))
        }).collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result.truncate(limit);
        result
    }

    /// Restituisce il WordId di una parola.
    pub fn word_id(&self, word: &str) -> Option<WordId> {
        self.word_to_id.get(&word.to_lowercase()).copied()
    }

    /// Restituisce la parola dato un WordId.
    pub fn word_name(&self, id: WordId) -> Option<&str> {
        self.vertices.get(&id).map(|v| v.word.as_str())
    }

    /// Lista di adiacenza per un WordId — i vicini diretti nel campo.
    /// Restituisce slice vuoto se l'ID non esiste.
    pub fn adjacency_list(&self, id: WordId) -> &[WordId] {
        self.adjacency.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Peso dell'arco tra due parole per nome [0, 1].
    /// Restituisce None se l'arco non esiste.
    pub fn edge_weight_between(&self, a: &str, b: &str) -> Option<f64> {
        let id_a = self.word_to_id.get(&a.to_lowercase())?;
        let id_b = self.word_to_id.get(&b.to_lowercase())?;
        let key = if id_a < id_b { (*id_a, *id_b) } else { (*id_b, *id_a) };
        self.edges.get(&key).map(|e| e.weight)
    }

    /// Rimuove un arco tra due parole (se esiste).
    pub fn remove_edge_between(&mut self, a: &str, b: &str) {
        let id_a = match self.word_to_id.get(&a.to_lowercase()) { Some(&id) => id, None => return };
        let id_b = match self.word_to_id.get(&b.to_lowercase()) { Some(&id) => id, None => return };
        let key = if id_a < id_b { (id_a, id_b) } else { (id_b, id_a) };
        self.edges.remove(&key);
        if let Some(adj) = self.adjacency.get_mut(&key.0) { adj.retain(|&id| id != key.1); }
        if let Some(adj) = self.adjacency.get_mut(&key.1) { adj.retain(|&id| id != key.0); }
    }

    /// Tutte le parole con attivazione > soglia minima.
    /// Usato da PF1 per sincronizzare le attivazioni prima della propagazione.
    pub fn all_activations(&self) -> Vec<(&str, f64)> {
        self.vertices.values()
            .filter(|v| v.activation > 0.001)
            .map(|v| (v.word.as_str(), v.activation))
            .collect()
    }

    /// Imposta l'attivazione di una parola direttamente (senza incrementare il contatore).
    /// Usato da PF1 per sincronizzare i risultati di propagazione verso word_topology.
    pub fn set_activation(&mut self, word: &str, activation: f64) {
        if let Some(&id) = self.word_to_id.get(&word.to_lowercase()) {
            if let Some(vertex) = self.vertices.get_mut(&id) {
                vertex.activation = activation.clamp(0.0, 1.0);
            }
        }
    }

    /// Numero di vertici (parole).
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Numero di archi (connessioni).
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Densita del grafo: archi / archi_possibili.
    pub fn density(&self) -> f64 {
        let n = self.vertices.len() as f64;
        if n < 2.0 {
            return 0.0;
        }
        let max_edges = n * (n - 1.0) / 2.0;
        self.edges.len() as f64 / max_edges
    }

    /// Grado medio: quanti vicini ha in media una parola.
    pub fn average_degree(&self) -> f64 {
        if self.vertices.is_empty() {
            return 0.0;
        }
        let total_degree: usize = self.adjacency.values().map(|adj| adj.len()).sum();
        total_degree as f64 / self.vertices.len() as f64
    }

    /// Energia del campo: somma delle attivazioni di tutti i vertici.
    pub fn field_energy(&self) -> f64 {
        self.vertices.values().map(|v| v.activation).sum()
    }

    // ==================== AGGREGAZIONE EMERGENTE ====================

    /// Deriva le attivazioni frattali dallo stato del campo parole.
    ///
    /// I frattali NON sono vertici del campo — sono REGIONI emergenti.
    /// Ogni parola attiva contribuisce ai frattali in base alle sue affinita
    /// lessicali. Il risultato e una mappa FractalId → attivazione emergente.
    ///
    /// Questo SOSTITUISCE phrase.fractal_involvement nel flusso di attivazione:
    /// stesso tipo di dato, ma derivato dal CAMPO, non dal lessico direttamente.
    pub fn emerge_fractal_activations(&self, lexicon: &Lexicon) -> Vec<(FractalId, f64)> {
        let mut fractal_scores: HashMap<FractalId, (f64, f64)> = HashMap::new(); // (somma, conteggio)

        for vertex in self.vertices.values() {
            if vertex.activation <= self.activation_threshold {
                continue;
            }

            if let Some(pattern) = lexicon.get(&vertex.word) {
                for (&fid, &affinity) in &pattern.fractal_affinities {
                    let entry = fractal_scores.entry(fid).or_insert((0.0, 0.0));
                    entry.0 += vertex.activation * affinity;
                    entry.1 += 1.0;
                }
            }
        }

        let mut result: Vec<(FractalId, f64)> = fractal_scores.into_iter()
            .map(|(fid, (sum, count))| (fid, sum / count.max(1.0)))
            .filter(|(_, score)| *score > 0.03)
            .collect();

        // Ordina per attivazione decrescente
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result
    }

    /// Reset completo delle attivazioni (utile per test o reset).
    pub fn reset_activations(&mut self) {
        for vertex in self.vertices.values_mut() {
            vertex.activation = 0.0;
        }
    }

    // ==================== FASE DEGLI ARCHI ====================

    /// Ricalcola la fase di tutti gli archi basandosi sulla similarita
    /// degli intorni nel lessico.
    ///
    /// Il principio: due parole con intorni DIVERSI (co-occorrenze diverse)
    /// hanno fase alta (→ PI, opposizione). Intorni SIMILI → fase bassa (→ 0, risonanza).
    ///
    /// Usa la cosine similarity dei vettori co-occorrenza, mappata a fase:
    ///   cosine = 1.0 → phase = 0 (risonanza)
    ///   cosine = 0.5 → phase = PI/2 (tensione)
    ///   cosine = 0.0 → phase = PI (opposizione)
    pub fn recalculate_phases(&mut self, lexicon: &Lexicon) {
        let edge_keys: Vec<(WordId, WordId)> = self.edges.keys().copied().collect();

        for key in edge_keys {
            let (id_a, id_b) = key;

            let word_a = match self.vertices.get(&id_a) {
                Some(v) => v.word.clone(),
                None => continue,
            };
            let word_b = match self.vertices.get(&id_b) {
                Some(v) => v.word.clone(),
                None => continue,
            };

            let phase = Self::compute_edge_phase(lexicon, &word_a, &word_b);

            if let Some(edge) = self.edges.get_mut(&key) {
                edge.phase = phase;
            }
        }
    }

    /// Calcola la fase di un singolo arco [0, PI] radianti.
    ///
    /// DUE FONTI di segnale, combinate:
    /// 1. Rapporto negazione diretta (co_negated / totale) — operatori strutturali
    /// 2. Cosine similarity dei vicinati — struttura latente del campo
    ///
    /// Fonte 1 prioritaria (70%) se presente con dati sufficienti.
    /// Fonte 2 come fallback o contributo (30%).
    fn compute_edge_phase(lexicon: &Lexicon, word_a: &str, word_b: &str) -> f64 {
        use std::f64::consts::{PI, FRAC_PI_2};

        let pattern_a = match lexicon.get(word_a) {
            Some(p) => p,
            None => return FRAC_PI_2,
        };
        let pattern_b = match lexicon.get(word_b) {
            Some(p) => p,
            None => return FRAC_PI_2,
        };

        // === Fonte 1: rapporto negazione diretta (operatori) ===
        // Usa co_affirmed (solo affermazioni esplicite) come denominatore,
        // NON co_occurrences (che include contesti neutrali e diluerebbe il segnale).
        let neg_ab = pattern_a.co_negated.get(word_b).copied().unwrap_or(0);
        let neg_ba = pattern_b.co_negated.get(word_a).copied().unwrap_or(0);
        let aff_ab = pattern_a.co_affirmed.get(word_b).copied().unwrap_or(0);
        let aff_ba = pattern_b.co_affirmed.get(word_a).copied().unwrap_or(0);

        let total_neg = (neg_ab + neg_ba) as f64;
        let total_aff = (aff_ab + aff_ba) as f64;
        let total_direct = total_neg + total_aff;

        let operator_phase_opt = if total_direct >= 4.0 {
            // Rapporto negazione: 0.0=risonanza, 0.5=tensione, 1.0=opposizione
            let neg_ratio = total_neg / total_direct;
            let raw_phase = PI * neg_ratio;
            // Signal strength: quanto fidarci del dato operatore
            let signal = (total_direct / 10.0).min(1.0);
            Some(FRAC_PI_2 + (raw_phase - FRAC_PI_2) * signal)
        } else {
            None
        };

        // === Fonte 2: cosine similarity dei vicinati ===
        let cosine_phase_opt = Self::compute_cosine_phase(pattern_a, pattern_b, word_a, word_b);

        // === Blend ===
        match (operator_phase_opt, cosine_phase_opt) {
            (Some(op), Some(cos)) => op * 0.7 + cos * 0.3,
            (Some(op), None) => op,
            (None, Some(cos)) => cos,
            (None, None) => FRAC_PI_2,
        }
    }

    /// Calcola la fase dalla cosine similarity dei vicinati di co-occorrenza.
    /// Ritorna None se i dati sono insufficienti (vicinati poveri, ratio basso...).
    fn compute_cosine_phase(
        pattern_a: &crate::topology::lexicon::WordPattern,
        pattern_b: &crate::topology::lexicon::WordPattern,
        word_a: &str,
        word_b: &str,
    ) -> Option<f64> {
        use std::f64::consts::{PI, FRAC_PI_2};

        // Serve un minimo di co-occorrenze per calcolare
        if pattern_a.co_occurrences.len() < 3 || pattern_b.co_occurrences.len() < 3 {
            return None;
        }

        // Co-occorrenza reciproca (solo affermate, non negate)
        let co_ab = pattern_a.co_occurrences.get(word_b).copied().unwrap_or(0);
        let co_ba = pattern_b.co_occurrences.get(word_a).copied().unwrap_or(0);
        let mutual_co = (co_ab + co_ba) as f64;

        if mutual_co < 4.0 {
            return None;
        }

        // Rapporto co-occorrenza reciproca / totale
        let total_a: u64 = pattern_a.co_occurrences.values().sum();
        let total_b: u64 = pattern_b.co_occurrences.values().sum();
        let ratio_a = mutual_co / total_a.max(1) as f64;
        let ratio_b = mutual_co / total_b.max(1) as f64;
        let min_ratio = ratio_a.min(ratio_b);

        if min_ratio < 0.03 {
            return None;
        }

        // Filtro parole ubique (operatori ora esclusi gia dalla raccolta)
        let ubiquitous: &[&str] = &["io", "dentro", "fuori", "forte", "tu",
            "essere", "sentire", "avere", "di", "il", "la", "un", "una",
            "con", "per", "si", "mi", "ti", "ci", "lo", "le",
            "a", "da", "in", "su"];

        // Vicinati significativi
        let neighbors_a: usize = pattern_a.co_occurrences.keys()
            .filter(|w| w.as_str() != word_b && !ubiquitous.contains(&w.as_str()))
            .count();
        let neighbors_b: usize = pattern_b.co_occurrences.keys()
            .filter(|w| w.as_str() != word_a && !ubiquitous.contains(&w.as_str()))
            .count();

        if neighbors_a < 8 || neighbors_b < 8 {
            return None;
        }

        // Unione dei vicinati
        let mut all_words: Vec<&str> = Vec::new();
        for (w, _) in &pattern_a.co_occurrences {
            if w != word_b && !ubiquitous.contains(&w.as_str()) {
                all_words.push(w.as_str());
            }
        }
        for (w, _) in &pattern_b.co_occurrences {
            if w != word_a && !ubiquitous.contains(&w.as_str()) {
                if !all_words.contains(&w.as_str()) {
                    all_words.push(w.as_str());
                }
            }
        }

        // Servono parole condivise e esclusive
        let mut shared_count = 0usize;
        let mut exclusive_a = 0usize;
        let mut exclusive_b = 0usize;
        for w in &all_words {
            let va = pattern_a.co_occurrences.get(*w).copied().unwrap_or(0);
            let vb = pattern_b.co_occurrences.get(*w).copied().unwrap_or(0);
            if va > 0 && vb > 0 {
                shared_count += 1;
            } else if va > 0 {
                exclusive_a += 1;
            } else {
                exclusive_b += 1;
            }
        }

        if shared_count < 3 || exclusive_a < 3 || exclusive_b < 3 {
            return None;
        }

        // Cosine similarity sui vettori di co-occorrenza
        let mut dot = 0.0_f64;
        let mut norm_a = 0.0_f64;
        let mut norm_b = 0.0_f64;
        for w in &all_words {
            let va = pattern_a.co_occurrences.get(*w).copied().unwrap_or(0) as f64;
            let vb = pattern_b.co_occurrences.get(*w).copied().unwrap_or(0) as f64;
            dot += va * vb;
            norm_a += va * va;
            norm_b += vb * vb;
        }

        let cosine = if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a.sqrt() * norm_b.sqrt())
        } else {
            return None;
        };

        // cosine = 1.0 → phase = 0 (risonanza), cosine = 0.0 → phase = PI (opposizione)
        let raw_phase = PI * (1.0 - cosine);
        let signal_strength = (mutual_co / 20.0).min(1.0);
        Some(FRAC_PI_2 + (raw_phase - FRAC_PI_2) * signal_strength)
    }

    /// Setta la fase di un arco tra due parole [0, PI].
    /// Utile per bootstrap di opposizioni note o per correzioni manuali.
    pub fn set_edge_phase(&mut self, word_a: &str, word_b: &str, phase: f64) {
        let id_a = match self.word_to_id.get(&word_a.to_lowercase()) {
            Some(&id) => id,
            None => return,
        };
        let id_b = match self.word_to_id.get(&word_b.to_lowercase()) {
            Some(&id) => id,
            None => return,
        };

        let key = if id_a < id_b { (id_a, id_b) } else { (id_b, id_a) };

        if let Some(edge) = self.edges.get_mut(&key) {
            edge.phase = phase.clamp(0.0, std::f64::consts::PI);
        }
    }

    /// Restituisce la fase dell'arco tra due parole [0, PI].
    pub fn edge_phase(&self, word_a: &str, word_b: &str) -> Option<f64> {
        let id_a = self.word_to_id.get(&word_a.to_lowercase())?;
        let id_b = self.word_to_id.get(&word_b.to_lowercase())?;

        let key = if id_a < id_b { (*id_a, *id_b) } else { (*id_b, *id_a) };
        self.edges.get(&key).map(|e| e.phase)
    }

    /// Etichetta leggibile per una fase.
    pub fn phase_label(phase: f64) -> &'static str {
        use std::f64::consts::PI;
        if phase < PI / 3.0 { "risonanza" }
        else if phase > 2.0 * PI / 3.0 { "opposizione" }
        else { "tensione" }
    }

    /// Trova opposizioni: archi con fase alta (vicini a PI).
    /// min_phase: soglia minima (es. 2*PI/3 = 2.094)
    pub fn find_oppositions(&self, min_phase: f64) -> Vec<(&str, &str, f64)> {
        let mut oppositions: Vec<(&str, &str, f64)> = Vec::new();

        for edge in self.edges.values() {
            if edge.phase > min_phase {
                if let (Some(va), Some(vb)) = (
                    self.vertices.get(&edge.a),
                    self.vertices.get(&edge.b),
                ) {
                    oppositions.push((&va.word, &vb.word, edge.phase));
                }
            }
        }

        // Ordina per fase decrescente (opposizioni piu forti prima)
        oppositions.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        oppositions
    }

    /// Trova risonanze: archi con fase bassa (vicini a 0).
    /// max_phase: soglia massima (es. PI/3 = 1.047)
    pub fn find_resonances(&self, max_phase: f64) -> Vec<(&str, &str, f64)> {
        let mut resonances: Vec<(&str, &str, f64)> = Vec::new();

        for edge in self.edges.values() {
            if edge.phase < max_phase {
                if let (Some(va), Some(vb)) = (
                    self.vertices.get(&edge.a),
                    self.vertices.get(&edge.b),
                ) {
                    resonances.push((&va.word, &vb.word, edge.phase));
                }
            }
        }

        // Ordina per fase crescente (risonanze piu forti prima)
        resonances.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        resonances
    }

    // ==================== ARRICCHIMENTO EMERGENTE ====================

    /// Arricchisce i pesi degli archi con la distanza emergente.
    ///
    /// Per ogni coppia di parole connesse che condividono lo stesso frattale,
    /// il peso dell'arco viene MODULATO dalla vicinanza nelle dimensioni emergenti:
    /// - Parole vicine in emergente (es. gioia↔felicita) → peso aumenta
    /// - Parole lontane in emergente (es. gioia↔tristezza) → peso diminuisce
    ///
    /// Questo rende la propagazione SEMANTICAMENTE consapevole:
    /// non propaga uniformemente tra tutte le parole connesse,
    /// ma preferisce le parole che occupano regioni emergenti simili.
    pub fn enrich_with_emergent_distances(
        &mut self,
        lexicon: &Lexicon,
        registry: &crate::topology::fractal::FractalRegistry,
    ) {
        use crate::topology::fractal::FractalId;

        // Pre-calcola frattale primario per ogni parola
        let mut word_fractal: HashMap<WordId, FractalId> = HashMap::new();
        for (id, vertex) in &self.vertices {
            if let Some(pattern) = lexicon.get(&vertex.word) {
                if let Some((&fid, _)) = pattern.fractal_affinities.iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                {
                    word_fractal.insert(*id, fid);
                }
            }
        }

        // Modula ogni arco con la distanza emergente
        let edge_keys: Vec<(WordId, WordId)> = self.edges.keys().copied().collect();

        for key in edge_keys {
            let (id_a, id_b) = key;

            // Entrambe le parole devono avere un frattale primario
            let fid_a = match word_fractal.get(&id_a) {
                Some(&f) => f,
                None => continue,
            };
            let fid_b = match word_fractal.get(&id_b) {
                Some(&f) => f,
                None => continue,
            };

            // Solo per parole nello STESSO frattale: modula con distanza emergente
            if fid_a != fid_b {
                continue;
            }

            // Recupera le firme 8D
            let sig_a = match self.vertices.get(&id_a)
                .and_then(|v| lexicon.get(&v.word))
            {
                Some(pat) => pat.signature,
                None => continue,
            };
            let sig_b = match self.vertices.get(&id_b)
                .and_then(|v| lexicon.get(&v.word))
            {
                Some(pat) => pat.signature,
                None => continue,
            };

            let em_dist = registry.emergent_distance(fid_a, &sig_a, &sig_b);

            if em_dist > 0.01 {
                // Modulazione: vicinanza emergente → boost peso
                // em_dist = 0 → moltiplicatore = 1.5 (molto vicine)
                // em_dist = 1 → moltiplicatore = 0.75 (lontane)
                // em_dist = 2+ → moltiplicatore = 0.5 (molto lontane)
                let multiplier = 1.5 / (1.0 + em_dist);
                let multiplier = multiplier.clamp(0.3, 2.0);

                if let Some(edge) = self.edges.get_mut(&key) {
                    edge.weight = (edge.weight * multiplier).clamp(0.01, 1.0);
                }
            }
        }
    }
}

// ==================== TEST ====================

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_small_topology() -> WordTopology {
        let mut topo = WordTopology::new();
        let id_a = topo.add_word("gioia");
        let id_b = topo.add_word("felicita");
        let id_c = topo.add_word("tristezza");
        let id_d = topo.add_word("dolore");

        // gioia ↔ felicita (forte, risonanza — fase ≈ 0.2)
        topo.add_edge_with_phase(id_a, id_b, 0.8, 10, 0.2);
        // tristezza ↔ dolore (forte, risonanza — fase ≈ 0.3)
        topo.add_edge_with_phase(id_c, id_d, 0.7, 8, 0.3);
        // gioia ↔ tristezza (OPPOSIZIONE — fase ≈ 2.8, quasi PI)
        topo.add_edge_with_phase(id_a, id_c, 0.5, 12, 2.8);

        topo
    }

    #[test]
    fn test_add_word() {
        let mut topo = WordTopology::new();
        let id1 = topo.add_word("ciao");
        let id2 = topo.add_word("mondo");
        let id3 = topo.add_word("ciao"); // duplicato

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 0); // stesso ID
        assert_eq!(topo.vertex_count(), 2);
    }

    #[test]
    fn test_activate_and_query() {
        let mut topo = setup_small_topology();

        topo.activate_word("gioia", 0.8);
        topo.activate_word("felicita", 0.3);

        let active = topo.most_active(10);
        assert_eq!(active.len(), 2);
        assert_eq!(active[0].word, "gioia"); // piu attiva
    }

    #[test]
    fn test_propagation_affinity() {
        let mut topo = setup_small_topology();

        // Attiva solo gioia
        topo.activate_word("gioia", 0.8);

        // Propaga 1 step
        topo.propagate(1);

        // felicita (risonanza, fase ≈ 0.2) dovrebbe attivarsi
        let felicita = topo.vertices.values()
            .find(|v| v.word == "felicita")
            .unwrap();
        assert!(felicita.activation > 0.0, "felicita non attivata dalla propagazione");

        // dolore (non collegato a gioia) non dovrebbe attivarsi
        let dolore = topo.vertices.values()
            .find(|v| v.word == "dolore")
            .unwrap();
        assert_eq!(dolore.activation, 0.0, "dolore non dovrebbe attivarsi al primo step");
    }

    #[test]
    fn test_propagation_dualism() {
        let mut topo = setup_small_topology();

        // Attiva gioia E tristezza (entrambe presenti)
        topo.activate_word("gioia", 0.8);
        topo.activate_word("tristezza", 0.5);

        // Propaga: gioia dovrebbe DISATTIVARE tristezza (fase ≈ 2.8, opposizione)
        topo.propagate(1);

        // Tristezza dovrebbe essere diminuita (cos(2.8) < 0 → propagazione inversa)
        let tristezza = topo.vertices.values()
            .find(|v| v.word == "tristezza")
            .unwrap();
        assert!(tristezza.activation < 0.5,
            "tristezza ({}) dovrebbe diminuire (era 0.5) per opposizione con gioia",
            tristezza.activation);

        // felicita dovrebbe essere attivata (fase ≈ 0.2, risonanza)
        let felicita = topo.vertices.values()
            .find(|v| v.word == "felicita")
            .unwrap();
        assert!(felicita.activation > 0.0, "felicita deve attivarsi per risonanza con gioia");
    }

    #[test]
    fn test_propagation_dualism_inactive_target() {
        let mut topo = setup_small_topology();

        // Attiva SOLO gioia (tristezza parte da 0)
        topo.activate_word("gioia", 0.8);

        // Propaga: gioia prova a disattivare tristezza, ma e gia a 0
        topo.propagate(1);

        // Tristezza resta a 0 (non puo andare sotto 0)
        let tristezza = topo.vertices.values()
            .find(|v| v.word == "tristezza")
            .unwrap();
        assert_eq!(tristezza.activation, 0.0,
            "tristezza deve restare a 0 (gia inattiva, dualismo non la rende negativa)");
    }

    #[test]
    fn test_phase_query() {
        let topo = setup_small_topology();

        // Gioia ↔ felicita: risonanza (fase bassa, < PI/3)
        let phase_gf = topo.edge_phase("gioia", "felicita").unwrap();
        assert!(phase_gf < std::f64::consts::PI / 3.0,
            "gioia-felicita devono avere fase bassa (risonanza): {}", phase_gf);

        // Gioia ↔ tristezza: opposizione (fase alta, > 2*PI/3)
        let phase_gt = topo.edge_phase("gioia", "tristezza").unwrap();
        assert!(phase_gt > 2.0 * std::f64::consts::PI / 3.0,
            "gioia-tristezza devono avere fase alta (opposizione): {}", phase_gt);
    }

    #[test]
    fn test_propagation_tension() {
        // Fase = PI/2 → cos(PI/2) = 0 → nessuna propagazione
        let mut topo = WordTopology::new();
        let id_a = topo.add_word("coraggio");
        let id_b = topo.add_word("paura");
        topo.add_edge_with_phase(id_a, id_b, 0.8, 10, std::f64::consts::FRAC_PI_2);

        topo.activate_word("coraggio", 0.8);
        topo.propagate(1);

        // Paura non deve ne attivarsi ne disattivarsi (tensione creativa)
        let paura = topo.vertices.values()
            .find(|v| v.word == "paura")
            .unwrap();
        assert_eq!(paura.activation, 0.0,
            "con fase PI/2 (tensione) non deve esserci propagazione, ma activation={}", paura.activation);
    }

    #[test]
    fn test_propagation_two_steps() {
        let mut topo = setup_small_topology();

        // Attiva tristezza (non gioia — per testare propagazione positiva a dolore)
        topo.activate_word("tristezza", 0.8);
        topo.propagate(2);

        // Dopo step: dolore (collegato a tristezza con risonanza, fase=0.3) dovrebbe attivarsi
        let dolore = topo.vertices.values()
            .find(|v| v.word == "dolore")
            .unwrap();
        assert!(dolore.activation > 0.0,
            "dolore ({}) deve attivarsi dalla propagazione di tristezza (affinita)",
            dolore.activation);
    }

    #[test]
    fn test_decay() {
        let mut topo = setup_small_topology();

        topo.activate_word("gioia", 0.5);
        let before = topo.vertices.values()
            .find(|v| v.word == "gioia").unwrap().activation;

        topo.decay_all(0.1);

        let after = topo.vertices.values()
            .find(|v| v.word == "gioia").unwrap().activation;

        assert!(after < before, "attivazione deve diminuire dopo decay");
        assert!((after - before * 0.9).abs() < 0.001, "decay 10%: {} -> {}", before, after);
    }

    #[test]
    fn test_build_from_lexicon() {
        let lexicon = Lexicon::bootstrap();
        let topo = WordTopology::build_from_lexicon(&lexicon);

        assert_eq!(topo.vertex_count(), lexicon.word_count(),
            "ogni parola del lessico deve essere un vertice");

        // Il bootstrap ha co-occorrenze solo dopo teach, non nel bootstrap stesso.
        // Quindi gli archi potrebbero essere 0 con un lessico appena creato.
        // Ma i vertici devono corrispondere.
        println!("Vertici: {}, Archi: {}, Densita: {:.6}",
            topo.vertex_count(), topo.edge_count(), topo.density());
    }

    #[test]
    fn test_emerge_fractal_activations() {
        let lexicon = Lexicon::bootstrap();
        let mut topo = WordTopology::build_from_lexicon(&lexicon);

        // Attiva parole spaziali
        topo.activate_word("qui", 0.8);
        topo.activate_word("vicino", 0.6);
        topo.activate_word("lontano", 0.5);

        let fractals = topo.emerge_fractal_activations(&lexicon);

        // SPAZIO (id=36, ☶☶) dovrebbe essere il frattale piu attivo
        assert!(!fractals.is_empty(), "devono emergere frattali dalle parole spaziali");

        // Verifica che SPAZIO (id=36) sia tra i risultati
        let spazio = fractals.iter().find(|(fid, _)| *fid == 36);
        assert!(spazio.is_some(), "SPAZIO(36) deve emergere da parole spaziali. Frattali: {:?}", fractals);
    }

    #[test]
    fn test_density_and_stats() {
        let topo = setup_small_topology();

        assert_eq!(topo.vertex_count(), 4);
        assert_eq!(topo.edge_count(), 3);
        assert!(topo.density() > 0.0);
        assert!(topo.average_degree() > 0.0);
        assert_eq!(topo.field_energy(), 0.0); // nessuna attivazione
    }

    #[test]
    fn test_active_neighbors() {
        let mut topo = setup_small_topology();

        topo.activate_word("gioia", 0.8);
        topo.activate_word("felicita", 0.5);

        let neighbors = topo.active_neighbors("gioia");
        assert_eq!(neighbors.len(), 1); // solo felicita e attiva tra i vicini
        assert_eq!(neighbors[0].0, "felicita");
    }

    #[test]
    fn test_hub_damping_reduces_hub_weight() {
        // Phase 48: archi da/verso nodi hub devono avere peso minore
        // rispetto ad archi tra nodi con pochi archi
        use crate::topology::knowledge_graph::KnowledgeGraph;
        use crate::topology::relation::RelationType;

        let mut kg = KnowledgeGraph::new();

        // "essere" è un hub — connesso a molte cose
        for i in 0..100 {
            kg.add(&format!("cosa_{}", i), RelationType::IsA, "essere");
        }
        // "cane" è specifico — pochi archi
        kg.add("cane", RelationType::IsA, "animale");
        kg.add("cane", RelationType::Does, "abbaiare");

        let mut topo = WordTopology::new();
        topo.add_word("essere");
        topo.add_word("cane");
        topo.add_word("animale");
        topo.add_word("abbaiare");
        for i in 0..100 {
            topo.add_word(&format!("cosa_{}", i));
        }

        let (added, _) = topo.build_from_knowledge_graph(&kg);
        assert!(added > 0, "deve aver aggiunto archi");

        // L'arco cane→animale (nodi specifici) deve avere peso maggiore
        // di cosa_0→essere (nodo hub)
        let id_cane = *topo.word_to_id.get("cane").unwrap();
        let id_animale = *topo.word_to_id.get("animale").unwrap();
        let id_cosa0 = *topo.word_to_id.get("cosa_0").unwrap();
        let id_essere = *topo.word_to_id.get("essere").unwrap();

        let key_spec = if id_cane < id_animale { (id_cane, id_animale) } else { (id_animale, id_cane) };
        let key_hub = if id_cosa0 < id_essere { (id_cosa0, id_essere) } else { (id_essere, id_cosa0) };

        let w_spec = topo.edges.get(&key_spec).map(|e| e.weight).unwrap_or(0.0);
        let w_hub = topo.edges.get(&key_hub).map(|e| e.weight).unwrap_or(0.0);

        assert!(w_spec > w_hub,
            "arco specifico ({:.4}) deve pesare più di arco hub ({:.4})",
            w_spec, w_hub);
    }

    #[test]
    fn test_per_edge_confidence_varies_weight() {
        // Phase 48: due archi IS_A con confidence diversa → peso diverso
        use crate::topology::knowledge_graph::KnowledgeGraph;
        use crate::topology::relation::{RelationType, TypedEdge};

        let mut kg = KnowledgeGraph::new();
        // cane IS_A animale con confidence alta
        kg.add_edge(TypedEdge::new("cane", RelationType::IsA, "animale").with_confidence(0.95));
        // tavolo IS_A oggetto con confidence bassa
        kg.add_edge(TypedEdge::new("tavolo", RelationType::IsA, "oggetto").with_confidence(0.40));

        let mut topo = WordTopology::new();
        for w in &["cane", "animale", "tavolo", "oggetto"] {
            topo.add_word(w);
        }

        topo.build_from_knowledge_graph(&kg);

        let id = |w: &str| *topo.word_to_id.get(w).unwrap();
        let key1 = { let (a, b) = (id("cane"), id("animale")); if a < b { (a, b) } else { (b, a) } };
        let key2 = { let (a, b) = (id("tavolo"), id("oggetto")); if a < b { (a, b) } else { (b, a) } };

        let w1 = topo.edges.get(&key1).map(|e| e.weight).unwrap_or(0.0);
        let w2 = topo.edges.get(&key2).map(|e| e.weight).unwrap_or(0.0);

        assert!(w1 > w2,
            "cane→animale (conf=0.95, peso={:.4}) deve pesare più di tavolo→oggetto (conf=0.40, peso={:.4})",
            w1, w2);
    }
}
