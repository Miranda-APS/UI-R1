/// Complessi Simpliciali — La topologia delle connessioni tra frattali.
///
/// I frattali non sono isolati. Sono connessi tramite strutture condivise.
/// Piu strutture condividono, piu profonda e la connessione.
/// La topologia risultante E il sapere di Prometeo.

use std::collections::{HashMap, HashSet};
use crate::topology::fractal::FractalId;
use crate::topology::primitive::Dim;

/// Identificatore univoco di un simplesso.
pub type SimplexId = u32;

/// Tipo di struttura condivisa tra frattali (faccia del simplesso).
#[derive(Debug, Clone, PartialEq)]
pub enum SharedStructureType {
    /// Condividono una dimensione primitiva 8D con ruolo simile
    PrimitiveDim(Dim),
    /// Condividono una proprieta emergente (per nome)
    EmergentProperty(String),
    /// Condividono un pattern di co-variazione
    CovariationPattern {
        dims: Vec<Dim>,
        description: String,
    },
}

/// Una struttura condivisa tra frattali — una "faccia" del simplesso.
#[derive(Debug, Clone)]
pub struct SharedFace {
    /// Che tipo di struttura e condivisa
    pub structure: SharedStructureType,
    /// Come si manifesta in ciascun frattale (puo differire)
    pub manifestations: HashMap<FractalId, String>,
    /// Forza della condivisione [0.0, 1.0]
    pub strength: f64,
}

impl SharedFace {
    pub fn from_dim(dim: Dim, strength: f64) -> Self {
        Self {
            structure: SharedStructureType::PrimitiveDim(dim),
            manifestations: HashMap::new(),
            strength: strength.clamp(0.0, 1.0),
        }
    }

    pub fn from_property(name: &str, strength: f64) -> Self {
        Self {
            structure: SharedStructureType::EmergentProperty(name.to_string()),
            manifestations: HashMap::new(),
            strength: strength.clamp(0.0, 1.0),
        }
    }

    pub fn with_manifestation(mut self, fractal: FractalId, desc: &str) -> Self {
        self.manifestations.insert(fractal, desc.to_string());
        self
    }
}

/// Un simplesso: connessione tra N frattali attraverso strutture condivise.
#[derive(Debug, Clone)]
pub struct Simplex {
    /// Identificatore univoco
    pub id: SimplexId,
    /// I vertici: frattali connessi
    pub vertices: Vec<FractalId>,
    /// Le facce condivise: strutture in comune
    pub shared_faces: Vec<SharedFace>,
    /// Dimensione topologica: (n vertici - 1)
    pub dimension: usize,
    /// Persistenza: quanto e stabile [0.0, 1.0]
    pub persistence: f64,
    /// Plasticita: quanto puo cambiare [0.0, 1.0]
    pub plasticity: f64,
    /// Contatore attivazioni
    pub activation_count: u64,
    /// Attivazione corrente [0.0, 1.0] — quanto e "illuminato" in questo momento
    pub current_activation: f64,
    /// Phase 52: parole sorgente che hanno generato questo simplesso (proposizioni inscritte).
    /// Usato dalla risonanza per riattivare parole dal passato nella generazione.
    pub source_words: Option<Vec<String>>,
    /// Phase 83 — categoria del simplesso. `None` = simplesso semantico
    /// (default, Phase 49-52, proposizioni estratte dal KG).
    /// `Some(s)` = simplesso grammaticale tipizzato (es. "preposizione_composta",
    /// "locuzione_fatica", "costrutto_modale", "tempo_composto"). La categoria
    /// non è un dispatch — è etichetta informativa per ispezione/curation.
    pub category: Option<String>,
    /// Phase 83 — se `true`, le `source_words` del simplesso devono attivarsi
    /// nell'**ordine** specificato (e in adiacenza, vedi `engine.receive`)
    /// affinché il simplesso emerga. Default `false` (semantico — qualsiasi
    /// co-attivazione vale). I simplessi grammaticali usano `true`: "rispetto a"
    /// ≠ "a rispetto".
    pub ordered: bool,
    /// Phase 83 — frattale-funzione che il simplesso ATTIVA quando emerge.
    /// È il meccanismo di "significato senza etichetta": un simplesso
    /// grammaticale non ha una regola semantica scritta — porta un
    /// orientamento del campo verso un attrattore specifico
    /// (es. RELAZIONE per `[rispetto, a]`, SALUTO per `[come, stai]`,
    /// POSSIBILITA per `[bisogna, che]`). Il parser di Phase 81 legge
    /// quali frattali sono attivi durante il parsing e decide ruoli
    /// dalla geometria del campo, non da lookup.
    pub function_fractal: Option<FractalId>,
}

impl Simplex {
    pub fn new(id: SimplexId, vertices: Vec<FractalId>, shared_faces: Vec<SharedFace>) -> Self {
        let dimension = if vertices.is_empty() { 0 } else { vertices.len() - 1 };
        Self {
            id,
            vertices,
            shared_faces,
            dimension,
            persistence: 0.3,
            plasticity: 0.9,
            activation_count: 0,
            current_activation: 0.0,
            source_words: None,
            category: None,
            ordered: false,
            function_fractal: None,
        }
    }

    /// Phase 83 — costruisce un simplesso grammaticale già configurato.
    /// I `vertices` sono i frattali su cui poggia (tipicamente derivati
    /// dalle firme delle source_words); `function_fractal` è il frattale
    /// che il simplesso attiva quando emerge. `persistence` parte alta
    /// (0.7) perché un simplesso grammaticale è insegnamento curato, non
    /// emergenza da co-occorrenza ripetuta — è cristallizzato d'origine.
    pub fn new_grammatical(
        id: SimplexId,
        vertices: Vec<FractalId>,
        shared_faces: Vec<SharedFace>,
        source_words: Vec<String>,
        category: String,
        function_fractal: FractalId,
    ) -> Self {
        let dimension = if vertices.is_empty() { 0 } else { vertices.len() - 1 };
        Self {
            id,
            vertices,
            shared_faces,
            dimension,
            persistence: 0.7,
            plasticity: 0.3,
            activation_count: 0,
            current_activation: 0.0,
            source_words: Some(source_words),
            category: Some(category),
            ordered: true,
            function_fractal: Some(function_fractal),
        }
    }

    /// Phase 83 — è un simplesso grammaticale?
    pub fn is_grammatical(&self) -> bool {
        self.category.is_some() && self.function_fractal.is_some()
    }

    /// Questo simplesso contiene un dato frattale?
    pub fn contains(&self, fractal: FractalId) -> bool {
        self.vertices.contains(&fractal)
    }

    /// Quante facce condivise ha?
    pub fn face_count(&self) -> usize {
        self.shared_faces.len()
    }

    /// Forza complessiva della connessione (media delle forze delle facce).
    pub fn connection_strength(&self) -> f64 {
        if self.shared_faces.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.shared_faces.iter().map(|f| f.strength).sum();
        sum / self.shared_faces.len() as f64
    }

    /// Attiva il simplesso: incrementa contatore, consolida.
    pub fn activate(&mut self, strength: f64) {
        self.activation_count += 1;
        self.current_activation = (self.current_activation + strength).min(1.0);
        // Cristallizzazione progressiva
        self.plasticity = (self.plasticity * 0.995).max(0.05);
        self.persistence = (self.persistence + 0.003).min(1.0);
    }

    /// Decadimento dell'attivazione corrente (per passo temporale).
    pub fn decay(&mut self, rate: f64) {
        self.current_activation = (self.current_activation - rate).max(0.0);
    }
}

/// Il complesso simpliciale: l'intera topologia delle connessioni.
#[derive(Debug, Clone)]
pub struct SimplicialComplex {
    /// Tutti i simplessi
    simplices: HashMap<SimplexId, Simplex>,
    /// Indice: frattale → simplessi che lo contengono
    fractal_index: HashMap<FractalId, Vec<SimplexId>>,
    /// Prossimo ID
    next_id: SimplexId,
    /// Soglia di attivazione globale (veglia vs sogno)
    pub activation_threshold: f64,
    /// Set sparso dei simplessi attualmente attivi (sopra soglia).
    /// Aggiornato incrementalmente: O(active) per active_simplices() invece di O(19K).
    active_set: HashSet<SimplexId>,
}

impl SimplicialComplex {
    pub fn new() -> Self {
        Self {
            simplices: HashMap::new(),
            fractal_index: HashMap::new(),
            next_id: 0,
            activation_threshold: 0.15,
            active_set: HashSet::new(),
        }
    }

    /// Trova un simplesso con esattamente questi vertici, o restituisce None.
    pub fn find_simplex_with_vertices(&self, vertices: &[FractalId]) -> Option<SimplexId> {
        if vertices.is_empty() { return None; }
        // Parti dal frattale con meno simplessi (minimizza iterazioni)
        let start_fid = vertices.iter()
            .min_by_key(|&&fid| self.fractal_index.get(&fid).map(|v| v.len()).unwrap_or(0))?;
        let candidates = self.fractal_index.get(start_fid)?;
        for &sid in candidates {
            if let Some(s) = self.simplices.get(&sid) {
                if s.vertices.len() == vertices.len()
                    && vertices.iter().all(|v| s.vertices.contains(v))
                {
                    return Some(sid);
                }
            }
        }
        None
    }

    /// Aggiunge un simplesso al complesso.
    pub fn add_simplex(&mut self, vertices: Vec<FractalId>, shared_faces: Vec<SharedFace>) -> SimplexId {
        let id = self.next_id;
        self.next_id += 1;

        // Aggiorna indice
        for &v in &vertices {
            self.fractal_index.entry(v).or_default().push(id);
        }

        let simplex = Simplex::new(id, vertices, shared_faces);
        self.simplices.insert(id, simplex);
        id
    }

    /// Ripristina un simplesso con ID, persistenza, plasticità e activation_count specifici.
    /// Usato dalla persistenza per ricostruire il complesso esatto salvato.
    /// Phase 83: ultimi 3 parametri (category, ordered, function_fractal)
    /// sono `Option<...>`/`bool` per backward compat con snapshot pre-P83.
    #[allow(clippy::too_many_arguments)]
    pub fn restore_simplex(&mut self, id: SimplexId, vertices: Vec<FractalId>,
                           shared_faces: Vec<SharedFace>,
                           persistence: f64, plasticity: f64, activation_count: u64,
                           source_words: Option<Vec<String>>,
                           category: Option<String>,
                           ordered: bool,
                           function_fractal: Option<FractalId>) {
        // Aggiorna indice
        for &v in &vertices {
            self.fractal_index.entry(v).or_default().push(id);
        }

        let mut simplex = Simplex::new(id, vertices, shared_faces);
        simplex.persistence = persistence;
        simplex.plasticity = plasticity;
        simplex.activation_count = activation_count;
        simplex.source_words = source_words;
        simplex.category = category;
        simplex.ordered = ordered;
        simplex.function_fractal = function_fractal;
        self.simplices.insert(id, simplex);

        // Assicura che next_id sia sempre oltre l'id più alto
        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }

    /// Svuota completamente il complesso.
    pub fn clear(&mut self) {
        self.simplices.clear();
        self.fractal_index.clear();
        self.active_set.clear();
        self.next_id = 0;
    }

    /// Accesso a un simplesso.
    pub fn get(&self, id: SimplexId) -> Option<&Simplex> {
        self.simplices.get(&id)
    }

    /// Accesso mutabile.
    pub fn get_mut(&mut self, id: SimplexId) -> Option<&mut Simplex> {
        self.simplices.get_mut(&id)
    }

    /// Tutti i simplessi che contengono un frattale.
    pub fn simplices_of(&self, fractal: FractalId) -> Vec<SimplexId> {
        self.fractal_index.get(&fractal).cloned().unwrap_or_default()
    }

    /// Top-N simplessi per persistenza decrescente per un frattale.
    /// Pre-computa la persistenza prima del sort (evita HashMap lookup per ogni confronto).
    pub fn top_persistent_simplices_of(&self, fractal: FractalId, limit: usize) -> Vec<SimplexId> {
        let sids = match self.fractal_index.get(&fractal) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let mut with_p: Vec<(SimplexId, f32)> = sids.iter()
            .filter_map(|&id| self.simplices.get(&id).map(|s| (id, s.persistence as f32)))
            .collect();
        with_p.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        with_p.into_iter().take(limit).map(|(id, _)| id).collect()
    }

    /// Precomputa una cache (frattale → top-N simplessi ordinati per persistenza).
    /// Costo: O(N_frattali × N_simplessi_per_frattale × log).
    /// Usato da build_from_lexicon per evitare 75K sort ridondanti durante il loop parole.
    pub fn build_fractal_simplex_cache(&self, limit: usize) -> HashMap<FractalId, Vec<SimplexId>> {
        self.fractal_index.keys().map(|&fid| {
            (fid, self.top_persistent_simplices_of(fid, limit))
        }).collect()
    }

    /// Tutti i simplessi condivisi tra due frattali.
    /// O(sa + sb) via HashSet — evita O(sa × sb) della versione naive.
    pub fn shared_simplices(&self, a: FractalId, b: FractalId) -> Vec<SimplexId> {
        let sa = match self.fractal_index.get(&a) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let sb = match self.fractal_index.get(&b) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let set_a: HashSet<SimplexId> = sa.iter().copied().collect();
        sb.iter().filter(|id| set_a.contains(id)).copied().collect()
    }

    /// Vicinanza topologica tra due frattali:
    /// quanti simplessi condividono, pesati per dimensione e forza.
    pub fn topological_proximity(&self, a: FractalId, b: FractalId) -> f64 {
        let shared = self.shared_simplices(a, b);
        if shared.is_empty() {
            return 0.0;
        }
        let score: f64 = shared.iter()
            .filter_map(|id| self.simplices.get(id))
            .map(|s| {
                let dim_bonus = 1.0 + s.dimension as f64 * 0.5;
                let strength = s.connection_strength();
                dim_bonus * strength * s.persistence
            })
            .sum();
        // Normalizza (approssimativo)
        (score / 10.0).min(1.0)
    }

    /// Attiva una regione del complesso centrata su un frattale.
    /// Restituisce i simplessi illuminati.
    pub fn activate_region(&mut self, center: FractalId, strength: f64) -> Vec<SimplexId> {
        let mut activated = Vec::new();
        let simplex_ids = self.simplices_of(center);

        for sid in simplex_ids {
            if let Some(simplex) = self.simplices.get_mut(&sid) {
                // L'attivazione dipende da:
                // - la forza richiesta
                // - la persistenza del simplesso (stabile = reagisce di piu)
                // - il numero di facce condivise (piu facce = piu rilevante)
                let relevance = strength * simplex.persistence * (1.0 + simplex.face_count() as f64 * 0.1);
                if relevance > self.activation_threshold {
                    simplex.activate(relevance.min(1.0));
                    self.active_set.insert(sid);
                    activated.push(sid);
                }
            }
        }

        activated
    }

    /// Propaga l'attivazione ai simplessi adiacenti (simplessi che condividono vertici
    /// con quelli gia attivi).
    /// La propagazione e proporzionale all'attivazione sorgente e decade con ogni step.
    /// Cosi il campo differenzia: attivazioni forti si diffondono, deboli restano locali.
    pub fn propagate_activation(&mut self, steps: usize) {
        for step in 0..steps {
            // Fattore di smorzamento: decade con ogni step
            let step_damping = 0.15 / (1.0 + step as f64);

            // Raccogli simplessi attivi con le loro attivazioni.
            // Cap a 100 sorgenti (le piu attive) per evitare O(N^2) con complessi grandi.
            let mut active_sources: Vec<(Vec<FractalId>, f64)> = self.simplices.values()
                .filter(|s| s.current_activation > self.activation_threshold)
                .map(|s| (s.vertices.clone(), s.current_activation))
                .collect();
            if active_sources.len() > 100 {
                active_sources.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                active_sources.truncate(100);
            }

            // Per ogni sorgente, propaga ai vicini proporzionalmente.
            // OTTIMIZZAZIONE: split borrow su simplices + fractal_index per evitare
            // Vec<SimplexId>.clone() (era 750KB/chiamata con 3428 simplici).
            // fractal_index.get() restituisce &[SimplexId] — zero allocazioni.
            let activation_threshold = self.activation_threshold;
            let SimplicialComplex { ref simplices, ref fractal_index, .. } = *self;

            let mut to_activate: Vec<(SimplexId, f64)> = Vec::new();
            for (vertices, src_activation) in &active_sources {
                for &v in vertices {
                    if let Some(sids) = fractal_index.get(&v) {
                        for &sid in sids {
                            if let Some(s) = simplices.get(&sid) {
                                if s.current_activation < activation_threshold {
                                    // Propagazione = sorgente * smorzamento * persistenza target
                                    let propagated = src_activation * step_damping * s.persistence;
                                    to_activate.push((sid, propagated));
                                }
                            }
                        }
                    }
                }
            }

            for (sid, strength) in to_activate {
                if let Some(s) = self.simplices.get_mut(&sid) {
                    s.activate(strength);
                    self.active_set.insert(sid);
                }
            }
        }
    }

    /// Decadimento globale: riduce l'attivazione di tutti i simplessi.
    pub fn decay_all(&mut self, rate: f64) {
        let threshold = self.activation_threshold;
        for simplex in self.simplices.values_mut() {
            simplex.decay(rate);
        }
        // Aggiorna active_set: rimuovi quelli scesi sotto soglia
        self.active_set.retain(|id| {
            self.simplices.get(id).map_or(false, |s| s.current_activation > threshold)
        });
    }

    /// Elimina i simplessi dinamici (id >= bootstrap_count) con attivazione e persistenza basse.
    /// Preserva sempre i simplessi bootstrap (id < bootstrap_count) e quelli ad alta persistenza.
    /// Chiama dopo il restore per ripulire complessi gonfiati da sessioni precedenti.
    pub fn prune_low_activity(&mut self, bootstrap_count: usize) {
        // Soglia: rimuove solo simplici MAI stati attivati (activation_count == 0)
        // e con persistence ancora al valore di fabbrica (< 0.31).
        // Con PF1 (Phase 27) la propagazione è O(attive × 8), non O(N²):
        // non è più necessario tagliare aggressivamente.
        // I simplici da conversazione (activation_count >= 1) vengono sempre preservati.
        let to_remove: Vec<SimplexId> = self.simplices.iter()
            .filter(|(&id, s)| {
                (id as usize) >= bootstrap_count
                && s.current_activation < self.activation_threshold
                && s.activation_count == 0
                && s.persistence < 0.31
            })
            .map(|(&id, _)| id)
            .collect();
        for id in &to_remove {
            if let Some(s) = self.simplices.remove(id) {
                for v in &s.vertices {
                    if let Some(list) = self.fractal_index.get_mut(v) {
                        list.retain(|sid| sid != id);
                    }
                }
                self.active_set.remove(id);
            }
        }
    }

    /// Restituisce i simplessi attualmente attivi (sopra soglia).
    /// O(active) grazie all'active_set — invece di O(19K) scansione completa.
    pub fn active_simplices(&self) -> Vec<&Simplex> {
        self.active_set.iter()
            .filter_map(|id| self.simplices.get(id))
            .filter(|s| s.current_activation > self.activation_threshold)
            .collect()
    }

    /// Restituisce i simplessi piu attivi, ordinati per attivazione decrescente.
    pub fn most_active(&self, limit: usize) -> Vec<&Simplex> {
        let mut active: Vec<&Simplex> = self.simplices.values()
            .filter(|s| s.current_activation > 0.0)
            .collect();
        active.sort_by(|a, b| b.current_activation.partial_cmp(&a.current_activation).unwrap());
        active.into_iter().take(limit).collect()
    }

    /// Numero totale di simplessi.
    pub fn count(&self) -> usize {
        self.simplices.len()
    }

    /// Dimensione massima dei simplessi nel complesso.
    pub fn max_dimension(&self) -> usize {
        self.simplices.values().map(|s| s.dimension).max().unwrap_or(0)
    }

    /// Iteratore su tutti i simplessi.
    pub fn iter(&self) -> impl Iterator<Item = (&SimplexId, &Simplex)> {
        self.simplices.iter()
    }

    /// Dissolvi simplessi con persistenza troppo bassa e attivazione zero.
    /// Restituisce il numero di simplessi rimossi.
    /// Incrementa la persistenza strutturale dei simplessi in un frattale senza attivarli.
    /// Usato dalla self-resonance: ciò che Prometeo esprime si cristallizza topologicamente,
    /// ma non crea eco nel campo di attivazione.
    /// `delta` tipicamente 0.003–0.005 per espressione.
    pub fn nudge_persistence_at(&mut self, center: FractalId, delta: f64) {
        let ids = self.simplices_of(center);
        for sid in ids {
            if let Some(s) = self.simplices.get_mut(&sid) {
                s.persistence = (s.persistence + delta).min(1.0);
            }
        }
    }

    pub fn dissolve_weak(&mut self, min_persistence: f64) -> usize {
        let to_remove: Vec<SimplexId> = self.simplices.iter()
            .filter(|(_, s)| s.persistence < min_persistence && s.activation_count < 3)
            .map(|(id, _)| *id)
            .collect();

        let count = to_remove.len();
        for id in &to_remove {
            if let Some(simplex) = self.simplices.remove(id) {
                // Rimuovi dall'indice
                for v in &simplex.vertices {
                    if let Some(ids) = self.fractal_index.get_mut(v) {
                        ids.retain(|sid| sid != id);
                    }
                }
                self.active_set.remove(id);
            }
        }
        count
    }

    // === OMOLOGIA (semplificata) ===

    /// Conta i "buchi" di dimensione 0: componenti connesse.
    /// Frattali non connessi da nessun simplesso sono isolati.
    pub fn connected_components(&self) -> usize {
        let all_fractals: Vec<FractalId> = self.fractal_index.keys().copied().collect();
        if all_fractals.is_empty() {
            return 0;
        }

        let mut visited = std::collections::HashSet::new();
        let mut components = 0;

        for &start in &all_fractals {
            if visited.contains(&start) {
                continue;
            }
            components += 1;
            // BFS
            let mut queue = vec![start];
            while let Some(current) = queue.pop() {
                if !visited.insert(current) {
                    continue;
                }
                // Trova tutti i frattali connessi via simplessi
                for sid in self.simplices_of(current) {
                    if let Some(s) = self.simplices.get(&sid) {
                        for &v in &s.vertices {
                            if !visited.contains(&v) {
                                queue.push(v);
                            }
                        }
                    }
                }
            }
        }

        components
    }

    /// Frattali "isolati": presenti nell'indice ma in nessun simplesso con altri.
    pub fn isolated_fractals(&self) -> Vec<FractalId> {
        self.fractal_index.iter()
            .filter(|(_, sids)| {
                sids.iter().all(|sid| {
                    self.simplices.get(sid).map(|s| s.vertices.len() <= 1).unwrap_or(true)
                })
            })
            .map(|(fid, _)| *fid)
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════
// Bootstrap: connessioni tra i 64 esagrammi
// ═══════════════════════════════════════════════════════════════

// ID esagrammi puri (stesso trigramma inferiore e superiore)
// Trigramma: Cielo=0 Terra=1 Tuono=2 Acqua=3 Montagna=4 Vento=5 Fuoco=6 Lago=7
// ID puro = t*8+t → 0, 9, 18, 27, 36, 45, 54, 63
const PURE_POTERE:    FractalId = 0;  // ☰☰
const PURE_ARMONIA_T: FractalId = 9;  // ☷☷
const PURE_IMPULSO:   FractalId = 18; // ☳☳
const PURE_DIVENIRE:  FractalId = 27; // ☵☵
const PURE_SPAZIO:    FractalId = 36; // ☶☶
const PURE_RESPIRO:   FractalId = 45; // ☴☴
const PURE_VERITA:    FractalId = 54; // ☲☲
const PURE_ARMONIA:   FractalId = 63; // ☱☱

/// Crea il complesso simpliciale iniziale con le connessioni
/// tra i 64 esagrammi I Ching.
///
/// Struttura di bootstrap:
/// 1. Ring degli 8 esagrammi puri (stessa qualità superiore e inferiore)
///    → spina dorsale del sistema — risonanza primordiale
/// 2. Triangoli di affinità tra esagrammi adiacenti nel ring
///    → prime strutture 2D — esperienza, dialogo, trasformazione
pub fn bootstrap_complex(fractal_ids: &[FractalId]) -> SimplicialComplex {
    let mut complex = SimplicialComplex::new();

    // Ci aspettiamo almeno 64 frattali
    if fractal_ids.len() < 64 {
        return complex;
    }

    // --- Ring degli 8 esagrammi puri (1-simplessi) ---
    // Ciascun esagramma puro connette dimensioni primordiali al successivo nel ring:
    // POTERE(0) ↔ TERRA(9) ↔ IMPULSO(18) ↔ DIVENIRE(27) ↔
    // SPAZIO(36) ↔ RESPIRO(45) ↔ VERITA(54) ↔ ARMONIA(63) ↔ POTERE(0)

    let ring: [(FractalId, FractalId, &str, &str); 8] = [
        (PURE_POTERE,    PURE_ARMONIA_T, "forza_che_sostiene",   "dall'azione al radicamento"),
        (PURE_ARMONIA_T, PURE_IMPULSO,   "radicamento_che_agisce","dalla terra al movimento"),
        (PURE_IMPULSO,   PURE_DIVENIRE,  "impulso_che_scorre",   "dal lampo al flusso"),
        (PURE_DIVENIRE,  PURE_SPAZIO,    "flusso_che_si_forma",  "dall'acqua alla montagna"),
        (PURE_SPAZIO,    PURE_RESPIRO,   "forma_che_respira",    "dalla montagna al vento"),
        (PURE_RESPIRO,   PURE_VERITA,    "respiro_che_illumina", "dal vento al fuoco"),
        (PURE_VERITA,    PURE_ARMONIA,   "luce_che_risuona",     "dal fuoco al lago"),
        (PURE_ARMONIA,   PURE_POTERE,    "risonanza_primordiale","dal lago al cielo"),
    ];

    for (fa, fb, prop, descr) in &ring {
        complex.add_simplex(
            vec![*fa, *fb],
            vec![
                SharedFace::from_property(prop, 0.3)
                    .with_manifestation(*fa, descr)
                    .with_manifestation(*fb, descr),
            ],
        );
    }

    // --- Triangoli fondamentali tra esagrammi puri adiacenti (2-simplessi) ---

    // POTERE × TERRA × IMPULSO → l'azione radicata che si muove (manifestazione)
    complex.add_simplex(
        vec![PURE_POTERE, PURE_ARMONIA_T, PURE_IMPULSO],
        vec![
            SharedFace::from_property("manifestazione", 0.4)
                .with_manifestation(PURE_POTERE,    "la forza che si muove")
                .with_manifestation(PURE_ARMONIA_T, "la terra che risponde")
                .with_manifestation(PURE_IMPULSO,   "il movimento che emerge"),
        ],
    );

    // DIVENIRE × SPAZIO × RESPIRO → il flusso nello spazio aperto (presenza)
    complex.add_simplex(
        vec![PURE_DIVENIRE, PURE_SPAZIO, PURE_RESPIRO],
        vec![
            SharedFace::from_property("presenza", 0.4)
                .with_manifestation(PURE_DIVENIRE, "il tempo che scorre")
                .with_manifestation(PURE_SPAZIO,   "il luogo che contiene")
                .with_manifestation(PURE_RESPIRO,  "il respiro che connette"),
        ],
    );

    // VERITA × ARMONIA × POTERE → la luce che risuona con la forza (coscienza)
    complex.add_simplex(
        vec![PURE_VERITA, PURE_ARMONIA, PURE_POTERE],
        vec![
            SharedFace::from_property("coscienza_primordiale", 0.4)
                .with_manifestation(PURE_VERITA,   "la chiarezza")
                .with_manifestation(PURE_ARMONIA,  "la risonanza")
                .with_manifestation(PURE_POTERE,   "la forza creatrice"),
        ],
    );

    complex
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::fractal::bootstrap_fractals;

    #[test]
    fn test_bootstrap_complex() {
        let reg = bootstrap_fractals();
        let ids = reg.all_ids();
        let complex = bootstrap_complex(&ids);

        // 8 archi ring + 3 triangoli = 11 simplessi minimi
        assert!(complex.count() >= 11, "Almeno 11 simplessi iniziali, trovati: {}", complex.count());
        assert!(complex.max_dimension() >= 2, "Almeno un 2-simplesso");
    }

    #[test]
    fn test_topological_proximity() {
        let reg = bootstrap_fractals();
        let ids = reg.all_ids();
        let complex = bootstrap_complex(&ids);

        // POTERE(0) e TERRA(9) sono connessi nel ring
        let prox = complex.topological_proximity(0, 9);
        assert!(prox > 0.0, "POTERE(0) e TERRA(9) devono essere connessi, prox={}", prox);

        // SPAZIO(36) e RESPIRO(45) sono connessi nel ring
        let prox2 = complex.topological_proximity(36, 45);
        assert!(prox2 > 0.0, "SPAZIO(36) e RESPIRO(45) devono essere connessi, prox={}", prox2);
    }

    #[test]
    fn test_activate_region() {
        let reg = bootstrap_fractals();
        let ids = reg.all_ids();
        let mut complex = bootstrap_complex(&ids);

        let activated = complex.activate_region(36, 0.8); // Attiva SPAZIO(☶☶)
        assert!(!activated.is_empty(), "Qualche simplesso deve attivarsi");

        let active = complex.active_simplices();
        assert!(!active.is_empty());
    }

    #[test]
    fn test_connected_components() {
        let reg = bootstrap_fractals();
        let ids = reg.all_ids();
        let complex = bootstrap_complex(&ids);

        let components = complex.connected_components();
        // Gli 8 esagrammi puri sono tutti connessi nel ring → 1 componente
        assert!(components >= 1, "Almeno una componente connessa");
    }

    #[test]
    fn test_decay_and_dissolve() {
        let reg = bootstrap_fractals();
        let ids = reg.all_ids();
        let mut complex = bootstrap_complex(&ids);

        // Attiva gli 8 esagrammi puri del ring
        for &id in &[0u32, 9, 18, 27, 36, 45, 54, 63] {
            complex.activate_region(id, 0.5);
        }

        // Decadimento
        for _ in 0..100 {
            complex.decay_all(0.01);
        }

        // Dopo 100 passi di decadimento, tutto dovrebbe essere quasi spento
        let active = complex.active_simplices();
        assert!(active.is_empty() || active.len() < complex.count());
    }
}
