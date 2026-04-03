/// SimplDB v3 — Database Simpliciale Binario
///
/// Formato sezionato con struttura topologica nativa.
/// Ogni sezione è indirizzabile indipendentemente tramite offset nell'header.
///
/// Layout file:
///   [HEADER:  128 bytes]  magic + word_count + edge_counts + section offsets
///   [LEXICON: variabile]  parole ordinate + firme 8D + affinità frattali
///   [GRAPH:   variabile]  archi co-occorrenza in formato CSR
///   [META:    variabile]  complesso + memoria + curriculum (bincode)
///
/// Tutti i valori numerici: little-endian (portabile su x86/ARM/mobile).
/// Leggibile da Kotlin/Swift senza dipendenze Rust.
///
/// Query topologiche native:
///   word_id("gioia")           → O(log n) binary search nel lessico ordinato
///   neighbors(id)              → O(1) slice CSR
///   top_words_in_fractal(fid)  → O(n) su affinità
///   topological_neighborhood   → BFS nel grafo CSR

use std::path::Path;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::topology::fractal::FractalId;
use crate::topology::grammar::PartOfSpeech;
use crate::topology::persistence::{
    PrometeoState, LexiconSnapshot, WordSnapshot,
    ComplexSnapshot, SimplexSnapshot, SharedFaceSnapshot, MemorySnapshot, LocusSnapshot,
    CurriculumProgress,
};
use crate::topology::simplex::SimplexId;
use crate::topology::lexicon::SemanticAxisSnapshot;
use crate::topology::knowledge::KnowledgeSnapshot;

// ═══════════════════════════════════════════════════════════════
// Costanti formato
// ═══════════════════════════════════════════════════════════════

/// Magic bytes del formato SimplDB v3 (sezionato, portabile).
pub const SIMPDB_MAGIC_V3: &[u8; 8] = b"SIMPDB03";

/// Magic bytes del formato SimplDB v2 (bincode blob, legacy).
pub const SIMPDB_MAGIC_V2: &[u8; 8] = b"SIMPDB02";

/// Dimensione fissa dell'header in bytes.
const HEADER_SIZE: usize = 128;

/// Identificatore numerico di una parola (posizione nel lessico ordinato).
pub type WordId = u32;

// ═══════════════════════════════════════════════════════════════
// Sezione META — dati non topologici serializzati via bincode
// ═══════════════════════════════════════════════════════════════

/// MetaSection corrente (Phase 58+): include episodi semantici nominati.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetaSection {
    version: String,
    total_perturbations: u64,
    dream_cycles: u64,
    complex: ComplexSnapshot,
    memory: MemorySnapshot,
    locus: Option<LocusSnapshot>,
    curriculum: Option<CurriculumProgress>,
    semantic_axes: Option<Vec<SemanticAxisSnapshot>>,
    knowledge: Option<KnowledgeSnapshot>,
    episodes: Option<crate::topology::episodic::EpisodeSnapshot>,
    instance_born: Option<u64>,
    identity: Option<crate::topology::identity::IdentitySnapshot>,
    narrative: Option<crate::topology::narrative::NarrativeSnapshot>,
    /// Phase 54: desideri attivi.
    desire: Option<crate::topology::desire::DesireSnapshot>,
    /// Phase 54: eco dell'Altro.
    interlocutor: Option<crate::topology::interlocutor::InterlocutorSnapshot>,
    /// Phase 54: stato SelfModel (credenze, valori, incertezze).
    self_model: Option<crate::topology::self_model::SelfModelSnapshot>,
    /// Phase 58: episodi semantici nominati (la storia vissuta dell'entità).
    semantic_episodes: Option<crate::topology::semantic_episode::SemanticEpisodeLog>,
}

/// MetaSection pre-Phase58 — senza semantic_episodes.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetaSectionPreP58 {
    version: String,
    total_perturbations: u64,
    dream_cycles: u64,
    complex: ComplexSnapshot,
    memory: MemorySnapshot,
    locus: Option<LocusSnapshot>,
    curriculum: Option<CurriculumProgress>,
    semantic_axes: Option<Vec<SemanticAxisSnapshot>>,
    knowledge: Option<KnowledgeSnapshot>,
    episodes: Option<crate::topology::episodic::EpisodeSnapshot>,
    instance_born: Option<u64>,
    identity: Option<crate::topology::identity::IdentitySnapshot>,
    narrative: Option<crate::topology::narrative::NarrativeSnapshot>,
    desire: Option<crate::topology::desire::DesireSnapshot>,
    interlocutor: Option<crate::topology::interlocutor::InterlocutorSnapshot>,
    self_model: Option<crate::topology::self_model::SelfModelSnapshot>,
}

impl From<MetaSectionPreP58> for MetaSection {
    fn from(l: MetaSectionPreP58) -> Self {
        MetaSection {
            version: l.version,
            total_perturbations: l.total_perturbations,
            dream_cycles: l.dream_cycles,
            complex: l.complex,
            memory: l.memory,
            locus: l.locus,
            curriculum: l.curriculum,
            semantic_axes: l.semantic_axes,
            knowledge: l.knowledge,
            episodes: l.episodes,
            instance_born: l.instance_born,
            identity: l.identity,
            narrative: l.narrative,
            desire: l.desire,
            interlocutor: l.interlocutor,
            self_model: l.self_model,
            semantic_episodes: None,
        }
    }
}

/// MetaSection pre-Phase54 — senza desire/interlocutor/self_model.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetaSectionPreP54 {
    version: String,
    total_perturbations: u64,
    dream_cycles: u64,
    complex: ComplexSnapshot,
    memory: MemorySnapshot,
    locus: Option<LocusSnapshot>,
    curriculum: Option<CurriculumProgress>,
    semantic_axes: Option<Vec<SemanticAxisSnapshot>>,
    knowledge: Option<KnowledgeSnapshot>,
    episodes: Option<crate::topology::episodic::EpisodeSnapshot>,
    instance_born: Option<u64>,
    identity: Option<crate::topology::identity::IdentitySnapshot>,
    narrative: Option<crate::topology::narrative::NarrativeSnapshot>,
}

impl From<MetaSectionPreP54> for MetaSection {
    fn from(l: MetaSectionPreP54) -> Self {
        MetaSection {
            version: l.version,
            total_perturbations: l.total_perturbations,
            dream_cycles: l.dream_cycles,
            complex: l.complex,
            memory: l.memory,
            locus: l.locus,
            curriculum: l.curriculum,
            semantic_axes: l.semantic_axes,
            knowledge: l.knowledge,
            episodes: l.episodes,
            instance_born: l.instance_born,
            identity: l.identity,
            narrative: l.narrative,
            desire: None,
            interlocutor: None,
            self_model: None,
            semantic_episodes: None,
        }
    }
}

/// Versione legacy di MetaSection senza instance_born.
/// Usata come fallback di deserializzazione per file .bin creati prima
/// dell'introduzione del campo — bincode non supporta campi opzionali a fine stream.
/// Usa ComplexSnapshotPreP52 perché i file legacy non hanno source_words.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetaSectionLegacy {
    version: String,
    total_perturbations: u64,
    dream_cycles: u64,
    complex: ComplexSnapshotPreP52,
    memory: MemorySnapshot,
    locus: Option<LocusSnapshot>,
    curriculum: Option<CurriculumProgress>,
    semantic_axes: Option<Vec<SemanticAxisSnapshot>>,
    knowledge: Option<KnowledgeSnapshot>,
    episodes: Option<crate::topology::episodic::EpisodeSnapshot>,
}

impl From<MetaSectionLegacy> for MetaSection {
    fn from(l: MetaSectionLegacy) -> Self {
        // Prima converti in MetaSectionPreP52, poi in MetaSection
        let pre = MetaSectionPreP52 {
            version: l.version,
            total_perturbations: l.total_perturbations,
            dream_cycles: l.dream_cycles,
            complex: l.complex,
            memory: l.memory,
            locus: l.locus,
            curriculum: l.curriculum,
            semantic_axes: l.semantic_axes,
            knowledge: l.knowledge,
            episodes: l.episodes,
            instance_born: None,
            identity: None,
            narrative: None,
        };
        MetaSection::from(pre)
    }
}

/// Pre-Phase52: SimplexSnapshot senza source_words.
/// Bincode è posizionale — i file salvati prima di Phase 52 non contengono il campo.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SimplexSnapshotPreP52 {
    pub id: SimplexId,
    pub vertices: Vec<FractalId>,
    pub dimension: usize,
    pub persistence: f64,
    pub plasticity: f64,
    pub activation_count: u64,
    pub faces: Vec<SharedFaceSnapshot>,
    pub face_descriptions: Vec<String>,
    // NO source_words
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ComplexSnapshotPreP52 {
    pub simplices: Vec<SimplexSnapshotPreP52>,
    pub next_id: SimplexId,
    pub activation_threshold: f64,
}

/// MetaSection pre-Phase52: stessi campi top-level di MetaSection corrente,
/// ma ComplexSnapshot senza source_words nei simplessi.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetaSectionPreP52 {
    version: String,
    total_perturbations: u64,
    dream_cycles: u64,
    complex: ComplexSnapshotPreP52,
    memory: MemorySnapshot,
    locus: Option<LocusSnapshot>,
    curriculum: Option<CurriculumProgress>,
    semantic_axes: Option<Vec<SemanticAxisSnapshot>>,
    knowledge: Option<KnowledgeSnapshot>,
    episodes: Option<crate::topology::episodic::EpisodeSnapshot>,
    instance_born: Option<u64>,
    identity: Option<crate::topology::identity::IdentitySnapshot>,
    narrative: Option<crate::topology::narrative::NarrativeSnapshot>,
}

impl From<MetaSectionPreP52> for MetaSection {
    fn from(l: MetaSectionPreP52) -> Self {
        // Converte i simplessi aggiungendo source_words: None
        let simplices = l.complex.simplices.into_iter().map(|s| SimplexSnapshot {
            id: s.id,
            vertices: s.vertices,
            dimension: s.dimension,
            persistence: s.persistence,
            plasticity: s.plasticity,
            activation_count: s.activation_count,
            faces: s.faces,
            face_descriptions: s.face_descriptions,
            source_words: None,
        }).collect();
        MetaSection {
            version: l.version,
            total_perturbations: l.total_perturbations,
            dream_cycles: l.dream_cycles,
            complex: ComplexSnapshot {
                simplices,
                next_id: l.complex.next_id,
                activation_threshold: l.complex.activation_threshold,
            },
            memory: l.memory,
            locus: l.locus,
            curriculum: l.curriculum,
            semantic_axes: l.semantic_axes,
            knowledge: l.knowledge,
            episodes: l.episodes,
            instance_born: l.instance_born,
            identity: l.identity,
            narrative: l.narrative,
            desire: None,
            interlocutor: None,
            self_model: None,
            semantic_episodes: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// SimplDB — il database
// ═══════════════════════════════════════════════════════════════

/// Database Simpliciale Binario.
///
/// Lessico ordinato lessicograficamente → binary search O(log n).
/// Grafo co-occorrenza in formato CSR → neighborood O(1).
/// Sezione META in bincode → complesso + memoria + curriculum.
/// Sezione PF1 (Phase 39+) → vicini ROM distillati, sostituisce co_occurrences.
pub struct SimplDB {
    // ── Lessico (ordinato lessicograficamente per binary search) ─────────
    /// Parole ordinate. words[word_id] = stringa.
    pub words: Vec<String>,
    /// Firme 8D. signatures[word_id] = [f64; 8].
    pub signatures: Vec<[f64; 8]>,
    /// Stabilità nel lessico [0..1].
    pub stability: Vec<f32>,
    /// Conteggio esposizioni.
    pub exposure: Vec<u32>,
    /// Categoria grammaticale (opzionale).
    pub pos: Vec<Option<PartOfSpeech>>,
    /// Affinità frattali per parola: [(fractal_id, affinity)].
    pub affinities: Vec<Vec<(u8, f32)>>,

    // ── Grafo co-occorrenza (CSR) ────────────────────────────────────────
    /// row_ptr[word_id]..row_ptr[word_id+1] = range in co_col/co_val.
    pub co_row_ptr: Vec<u32>,
    pub co_col: Vec<u32>,
    pub co_val: Vec<u32>,
    /// Co-occorrenze negate (operatore "non").
    pub neg_row_ptr: Vec<u32>,
    pub neg_col: Vec<u32>,
    pub neg_val: Vec<u32>,
    /// Co-occorrenze affermate (operatori "come", "simile").
    pub aff_row_ptr: Vec<u32>,
    pub aff_col: Vec<u32>,
    pub aff_val: Vec<u32>,

    // ── Meta (bincode blob) ──────────────────────────────────────────────
    meta_data: Vec<u8>,

    // ── PF1 topology (Phase 39+) ─────────────────────────────────────────
    /// Vicini PF1 per parola, indicizzati per SimplDB word_id.
    /// pf1_neighbors[word_id] = Vec<(neighbor_simpdb_id, weight, phase)>
    /// Bincode serializzato. Vuoto per file legacy (pre-Phase39).
    pf1_data: Vec<u8>,
}

// ═══════════════════════════════════════════════════════════════
// Costruzione da PrometeoState
// ═══════════════════════════════════════════════════════════════

impl SimplDB {
    /// Costruisce un SimplDB da uno stato Prometeo.
    /// Le parole vengono ordinate lessicograficamente per abilitare il binary search.
    pub fn from_state(state: &PrometeoState) -> Self {
        let words_snap = &state.lexicon.words;

        // Ordina le parole lessicograficamente → word_id = posizione nell'ordine
        let mut sorted: Vec<&WordSnapshot> = words_snap.iter().collect();
        sorted.sort_by(|a, b| a.word.cmp(&b.word));

        // Mappa parola → word_id per la risoluzione degli archi
        let word_to_id: HashMap<&str, WordId> = sorted.iter()
            .enumerate()
            .map(|(i, ws)| (ws.word.as_str(), i as WordId))
            .collect();

        let n = sorted.len();

        // ── Arrays paralleli del lessico ─────────────────────────────────
        let mut words     = Vec::with_capacity(n);
        let mut signatures = Vec::with_capacity(n);
        let mut stability  = Vec::with_capacity(n);
        let mut exposure   = Vec::with_capacity(n);
        let mut pos        = Vec::with_capacity(n);
        let mut affinities = Vec::with_capacity(n);

        for ws in &sorted {
            words.push(ws.word.clone());
            signatures.push(ws.signature);
            stability.push(ws.stability as f32);
            exposure.push(ws.exposure_count.min(u32::MAX as u64) as u32);
            pos.push(ws.pos.clone());
            let affs: Vec<(u8, f32)> = ws.fractal_affinities.iter()
                .map(|&(fid, val)| (fid as u8, val as f32))
                .collect();
            affinities.push(affs);
        }

        // ── Grafo CSR ────────────────────────────────────────────────────
        let (co_row_ptr, co_col, co_val) =
            build_csr(&sorted, &word_to_id, |ws| &ws.co_occurrences);
        let (neg_row_ptr, neg_col, neg_val) =
            build_csr(&sorted, &word_to_id, |ws| &ws.co_negated);
        let (aff_row_ptr, aff_col, aff_val) =
            build_csr(&sorted, &word_to_id, |ws| &ws.co_affirmed);

        // ── Meta section ─────────────────────────────────────────────────
        let meta = MetaSection {
            version: state.version.clone(),
            total_perturbations: state.total_perturbations,
            dream_cycles: state.dream_cycles,
            complex: state.complex.clone(),
            memory: state.memory.clone(),
            locus: state.locus.clone(),
            curriculum: state.curriculum.clone(),
            semantic_axes: state.semantic_axes.clone(),
            knowledge: state.knowledge.clone(),
            episodes: state.episodes.clone(),
            instance_born: state.instance_born,
            identity: state.identity.clone(),
            narrative: state.narrative.clone(),
            desire: state.desire.clone(),
            interlocutor: state.interlocutor.clone(),
            self_model: state.self_model.clone(),
            semantic_episodes: state.semantic_episodes.clone(),
        };
        let meta_data = bincode::serialize(&meta)
            .expect("MetaSection deve essere serializzabile");

        // ── PF1 neighbors (Phase 39+) ─────────────────────────────────────
        // Costruisce Vec<Vec<(simpdb_id, weight, phase)>> dai neighbors_pf1 dei WordSnapshot.
        // Traduce word_name → simpdb_id (ordine alfabetico) per compattezza.
        let pf1_data: Vec<u8> = {
            let any_pf1 = sorted.iter().any(|ws| !ws.neighbors_pf1.is_empty());
            if any_pf1 {
                let pf1_neighbors: Vec<Vec<(u32, f32, f32)>> = sorted.iter()
                    .map(|ws| {
                        ws.neighbors_pf1.iter()
                            .filter_map(|(nbr_name, w, ph)| {
                                word_to_id.get(nbr_name.as_str())
                                    .map(|&nbr_id| (nbr_id, *w, *ph))
                            })
                            .collect()
                    })
                    .collect();
                bincode::serialize(&pf1_neighbors)
                    .expect("pf1_neighbors deve essere serializzabile")
            } else {
                vec![]
            }
        };

        SimplDB {
            words, signatures, stability, exposure, pos, affinities,
            co_row_ptr, co_col, co_val,
            neg_row_ptr, neg_col, neg_val,
            aff_row_ptr, aff_col, aff_val,
            meta_data,
            pf1_data,
        }
    }

    /// Converte il SimplDB in PrometeoState (per compatibilità con restore_lexicon).
    pub fn to_state(&self) -> Result<PrometeoState, String> {
        // Catena fallback — bincode è posizionale, #[serde(default)] non basta.
        // 1. Formato corrente (Phase 58+: semantic_episodes)
        // 2. Pre-Phase58 (Phase 54+: desire/interlocutor/self_model, senza semantic_episodes)
        // 3. Pre-Phase54 (Phase 52+: source_words, senza desire/interlocutor/self_model)
        // 4. Pre-Phase52 (con instance_born/identity/narrative, senza source_words)
        // 5. Legacy originale (senza instance_born/identity/narrative)
        let meta: MetaSection = bincode::deserialize(&self.meta_data)
            .or_else(|_| {
                bincode::deserialize::<MetaSectionPreP58>(&self.meta_data)
                    .map(MetaSection::from)
            })
            .or_else(|_| {
                bincode::deserialize::<MetaSectionPreP54>(&self.meta_data)
                    .map(MetaSection::from)
            })
            .or_else(|_| {
                bincode::deserialize::<MetaSectionPreP52>(&self.meta_data)
                    .map(MetaSection::from)
            })
            .or_else(|_| {
                bincode::deserialize::<MetaSectionLegacy>(&self.meta_data)
                    .map(MetaSection::from)
            })
            .map_err(|e| format!("Errore deserializzazione MetaSection: {}", e))?;

        // Decodifica PF1 neighbors se presenti (Phase 39+)
        let pf1_neighbors_decoded: Option<Vec<Vec<(u32, f32, f32)>>> =
            if !self.pf1_data.is_empty() {
                bincode::deserialize(&self.pf1_data).ok()
            } else {
                None
            };

        let words: Vec<WordSnapshot> = self.words.iter().enumerate().map(|(i, word)| {
            // Ricostruisce neighbors_pf1 traducendo simpdb_id → nome_parola
            let neighbors_pf1: Vec<(String, f32, f32)> = pf1_neighbors_decoded.as_ref()
                .and_then(|v| v.get(i))
                .map(|nbrs| {
                    nbrs.iter()
                        .filter_map(|&(nbr_id, w, ph)| {
                            self.words.get(nbr_id as usize)
                                .map(|nbr_word| (nbr_word.clone(), w, ph))
                        })
                        .collect()
                })
                .unwrap_or_default();

            WordSnapshot {
                word: word.clone(),
                signature: self.signatures[i],
                fractal_affinities: self.affinities[i].iter()
                    .map(|&(fid, val)| (fid as FractalId, val as f64))
                    .collect(),
                exposure_count: self.exposure[i] as u64,
                stability: self.stability[i] as f64,
                co_occurrences: self.csr_to_vec(i as WordId,
                    &self.co_row_ptr, &self.co_col, &self.co_val),
                co_negated: self.csr_to_vec(i as WordId,
                    &self.neg_row_ptr, &self.neg_col, &self.neg_val),
                co_affirmed: self.csr_to_vec(i as WordId,
                    &self.aff_row_ptr, &self.aff_col, &self.aff_val),
                pos: self.pos[i].clone(),
                neighbors_pf1,
            }
        }).collect();

        Ok(PrometeoState {
            version: meta.version,
            total_perturbations: meta.total_perturbations,
            dream_cycles: meta.dream_cycles,
            lexicon: LexiconSnapshot { words },
            complex: meta.complex,
            memory: meta.memory,
            locus: meta.locus,
            curriculum: meta.curriculum,
            semantic_axes: meta.semantic_axes,
            knowledge: meta.knowledge,
            episodes: meta.episodes,
            instance_born: meta.instance_born,
            identity: meta.identity,
            narrative: meta.narrative,
            self_model: meta.self_model,
            semantic_episodes: meta.semantic_episodes,
            desire: meta.desire,
            interlocutor: meta.interlocutor,
        })
    }

    /// Ricostruisce un Vec<(String, u64)> da una riga CSR (per to_state).
    fn csr_to_vec(&self, id: WordId, row_ptr: &[u32], col: &[u32], val: &[u32])
        -> Vec<(String, u64)>
    {
        let start = row_ptr[id as usize] as usize;
        let end   = row_ptr[id as usize + 1] as usize;
        col[start..end].iter().zip(val[start..end].iter())
            .map(|(&c, &v)| (self.words[c as usize].clone(), v as u64))
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════
// Query API topologiche
// ═══════════════════════════════════════════════════════════════

impl SimplDB {
    /// Numero di parole nel lessico.
    pub fn word_count(&self) -> usize { self.words.len() }

    /// Cerca una parola → word_id via binary search O(log n).
    pub fn word_id(&self, word: &str) -> Option<WordId> {
        self.words.binary_search_by(|w| w.as_str().cmp(word))
            .ok()
            .map(|i| i as WordId)
    }

    /// Stringa di una parola da word_id.
    pub fn word_str(&self, id: WordId) -> Option<&str> {
        self.words.get(id as usize).map(|s| s.as_str())
    }

    /// Firma 8D di una parola.
    pub fn signature(&self, id: WordId) -> Option<[f64; 8]> {
        self.signatures.get(id as usize).copied()
    }

    /// Affinità di una parola verso un frattale specifico.
    pub fn affinity(&self, id: WordId, fractal: FractalId) -> f32 {
        self.affinities.get(id as usize)
            .and_then(|affs| affs.iter().find(|&&(fid, _)| fid == fractal as u8))
            .map(|&(_, val)| val)
            .unwrap_or(0.0)
    }

    /// Frattale primario di una parola (massima affinità).
    pub fn primary_fractal(&self, id: WordId) -> Option<FractalId> {
        self.affinities.get(id as usize)
            .and_then(|affs| affs.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()))
            .map(|&(fid, _)| fid as FractalId)
    }

    /// Vicini nel grafo co-occorrenza: iteratore su (word_id, count).
    pub fn neighbors(&self, id: WordId) -> impl Iterator<Item=(WordId, u32)> + '_ {
        let start = self.co_row_ptr[id as usize] as usize;
        let end   = self.co_row_ptr[id as usize + 1] as usize;
        self.co_col[start..end].iter()
            .zip(self.co_val[start..end].iter())
            .map(|(&c, &v)| (c, v))
    }

    /// Parole con affinità massima verso un frattale, ordinate per affinità.
    pub fn top_words_in_fractal(&self, fractal: FractalId, top_n: usize) -> Vec<(WordId, f32)> {
        let fid = fractal as u8;
        let mut result: Vec<(WordId, f32)> = self.affinities.iter().enumerate()
            .filter_map(|(i, affs)| {
                affs.iter().find(|&&(f, _)| f == fid)
                    .map(|&(_, val)| (i as WordId, val))
            })
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result.truncate(top_n);
        result
    }

    /// Neighborhood topologico: BFS nel grafo co-occorrenza fino a `hops` passi.
    /// Restituisce tutti i word_id raggiunti (escluso il nodo di partenza).
    pub fn topological_neighborhood(&self, id: WordId, hops: usize) -> Vec<WordId> {
        let mut visited = std::collections::HashSet::new();
        let mut frontier = vec![id];
        visited.insert(id);

        for _ in 0..hops {
            let mut next = Vec::new();
            for node in frontier {
                for (neighbor, _) in self.neighbors(node) {
                    if visited.insert(neighbor) {
                        next.push(neighbor);
                    }
                }
            }
            frontier = next;
            if frontier.is_empty() { break; }
        }

        visited.into_iter().filter(|&x| x != id).collect()
    }

    /// Distanza topologica tra due parole (BFS, -1 se non connesse).
    pub fn topological_distance(&self, a: WordId, b: WordId) -> i32 {
        if a == b { return 0; }
        let mut visited = std::collections::HashSet::new();
        let mut frontier = vec![a];
        visited.insert(a);
        let mut dist = 0i32;

        while !frontier.is_empty() {
            dist += 1;
            let mut next = Vec::new();
            for node in frontier {
                for (neighbor, _) in self.neighbors(node) {
                    if neighbor == b { return dist; }
                    if visited.insert(neighbor) {
                        next.push(neighbor);
                    }
                }
            }
            frontier = next;
        }
        -1 // non connessi
    }

    /// Parole sulla frontiera tra due frattali (affinità simile verso entrambi).
    pub fn frontier_words(&self, fa: FractalId, fb: FractalId, top_n: usize) -> Vec<WordId> {
        let fa = fa as u8;
        let fb = fb as u8;
        let mut scored: Vec<(WordId, f32)> = self.affinities.iter().enumerate()
            .filter_map(|(i, affs)| {
                let va = affs.iter().find(|&&(f, _)| f == fa).map(|&(_, v)| v).unwrap_or(0.0);
                let vb = affs.iter().find(|&&(f, _)| f == fb).map(|&(_, v)| v).unwrap_or(0.0);
                let min = va.min(vb);
                if min > 0.1 { Some((i as WordId, min)) } else { None }
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_n);
        scored.iter().map(|&(id, _)| id).collect()
    }

    /// Dimensione stimata del file che verrà scritto (in bytes).
    pub fn estimated_file_size(&self) -> usize {
        let n = self.words.len();
        let str_pool: usize = self.words.iter().map(|w| w.len() + 1).sum();
        let lex = 4 + str_pool + 4 * n + 64 * n + 4 * n + 4 * n + n
            + 4 + 4 * (n + 1)
            + self.affinities.iter().map(|a| a.len()).sum::<usize>() * 5;
        let graph = 12
            + 4 * (n + 1) * 3
            + (self.co_col.len() + self.neg_col.len() + self.aff_col.len()) * 8;
        HEADER_SIZE + lex + graph + self.meta_data.len() + self.pf1_data.len()
    }
}

// ═══════════════════════════════════════════════════════════════
// Serializzazione (save)
// ═══════════════════════════════════════════════════════════════

impl SimplDB {
    /// Scrive il file SimplDB v3 sul disco.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        use std::io::Write;

        let n = self.words.len();

        // ── LEXICON ──────────────────────────────────────────────────────
        let mut lex = Vec::new();

        // String pool (null-terminated, ordinato)
        let mut str_pool = Vec::<u8>::new();
        let mut str_offsets = Vec::<u32>::new();
        for word in &self.words {
            str_offsets.push(str_pool.len() as u32);
            str_pool.extend_from_slice(word.as_bytes());
            str_pool.push(0u8);
        }
        push_u32(&mut lex, str_pool.len() as u32);
        lex.extend_from_slice(&str_pool);
        for &off in &str_offsets { push_u32(&mut lex, off); }

        // Firme 8D
        for sig in &self.signatures {
            for &v in sig { push_f64(&mut lex, v); }
        }

        // Stabilità (f32), esposizioni (u32), POS (u8)
        for &v in &self.stability  { push_f32(&mut lex, v); }
        for &v in &self.exposure   { push_u32(&mut lex, v); }
        for p in &self.pos { lex.push(pos_to_u8(p)); }

        // Allineamento a 4 byte
        align4(&mut lex);

        // Affinità frattali (CSR-like: ptr + fractal_ids + values)
        let total_aff: u32 = self.affinities.iter().map(|a| a.len() as u32).sum();
        push_u32(&mut lex, total_aff);
        let mut ptr = 0u32;
        push_u32(&mut lex, 0);
        for affs in &self.affinities {
            ptr += affs.len() as u32;
            push_u32(&mut lex, ptr);
        }
        for affs in &self.affinities {
            for &(fid, _) in affs { lex.push(fid); }
        }
        align4(&mut lex);
        for affs in &self.affinities {
            for &(_, val) in affs { push_f32(&mut lex, val); }
        }

        // ── GRAPH ────────────────────────────────────────────────────────
        let mut graph = Vec::new();
        push_u32(&mut graph, self.co_col.len() as u32);
        push_u32(&mut graph, self.neg_col.len() as u32);
        push_u32(&mut graph, self.aff_col.len() as u32);

        for &v in &self.co_row_ptr  { push_u32(&mut graph, v); }
        for &v in &self.co_col      { push_u32(&mut graph, v); }
        for &v in &self.co_val      { push_u32(&mut graph, v); }
        for &v in &self.neg_row_ptr { push_u32(&mut graph, v); }
        for &v in &self.neg_col     { push_u32(&mut graph, v); }
        for &v in &self.neg_val     { push_u32(&mut graph, v); }
        for &v in &self.aff_row_ptr { push_u32(&mut graph, v); }
        for &v in &self.aff_col     { push_u32(&mut graph, v); }
        for &v in &self.aff_val     { push_u32(&mut graph, v); }

        // ── HEADER ───────────────────────────────────────────────────────
        let lex_off   = HEADER_SIZE as u64;
        let graph_off = lex_off + lex.len() as u64;
        let meta_off  = graph_off + graph.len() as u64;
        // Sezione PF1 (Phase 39+): subito dopo meta.
        // Offset=0 e size=0 → sezione assente (retrocompat con lettori vecchi).
        let pf1_off  = meta_off + self.meta_data.len() as u64;
        let pf1_size = self.pf1_data.len() as u64;

        let mut header = vec![0u8; HEADER_SIZE];
        header[0..8].copy_from_slice(SIMPDB_MAGIC_V3);
        header[8..12].copy_from_slice(&3u32.to_le_bytes());       // version
        header[12..16].copy_from_slice(&(n as u32).to_le_bytes()); // word_count
        header[16..20].copy_from_slice(&(self.co_col.len() as u32).to_le_bytes());
        header[20..24].copy_from_slice(&(self.neg_col.len() as u32).to_le_bytes());
        header[24..28].copy_from_slice(&(self.aff_col.len() as u32).to_le_bytes());
        // [28..32] reserved
        header[32..40].copy_from_slice(&lex_off.to_le_bytes());
        header[40..48].copy_from_slice(&(lex.len() as u64).to_le_bytes());
        header[48..56].copy_from_slice(&graph_off.to_le_bytes());
        header[56..64].copy_from_slice(&(graph.len() as u64).to_le_bytes());
        header[64..72].copy_from_slice(&meta_off.to_le_bytes());
        header[72..80].copy_from_slice(&(self.meta_data.len() as u64).to_le_bytes());
        header[80..88].copy_from_slice(&(pf1_off + pf1_size).to_le_bytes()); // eof
        // [88..96] pf1_offset — nuovo (Phase 39+), 0 se assente
        header[88..96].copy_from_slice(&pf1_off.to_le_bytes());
        // [96..104] pf1_size — nuovo (Phase 39+), 0 se assente
        header[96..104].copy_from_slice(&pf1_size.to_le_bytes());
        // [104..128] reserved

        // ── WRITE ────────────────────────────────────────────────────────
        let mut file = std::fs::File::create(path)
            .map_err(|e| format!("Errore creazione file SimplDB: {}", e))?;
        file.write_all(&header).map_err(|e| format!("header: {}", e))?;
        file.write_all(&lex).map_err(|e| format!("lexicon: {}", e))?;
        file.write_all(&graph).map_err(|e| format!("graph: {}", e))?;
        file.write_all(&self.meta_data).map_err(|e| format!("meta: {}", e))?;
        if !self.pf1_data.is_empty() {
            file.write_all(&self.pf1_data).map_err(|e| format!("pf1: {}", e))?;
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
// Deserializzazione (open)
// ═══════════════════════════════════════════════════════════════

impl SimplDB {
    /// Apre un file SimplDB v3 con memory-mapping.
    pub fn open(path: &Path) -> Result<Self, String> {
        use memmap2::Mmap;
        use std::fs::File;

        let file = File::open(path)
            .map_err(|e| format!("Errore apertura SimplDB: {}", e))?;
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| format!("Errore mmap: {}", e))?;

        if mmap.len() < HEADER_SIZE {
            return Err("File SimplDB corrotto (< 128 bytes)".to_string());
        }
        if &mmap[0..8] != SIMPDB_MAGIC_V3 {
            return Err(format!("Magic errato: {:?} (atteso SIMPDB03)", &mmap[0..8]));
        }

        // ── Parse header ─────────────────────────────────────────────────
        let word_count = rd_u32(&mmap, 12) as usize;
        let e_co  = rd_u32(&mmap, 16) as usize;
        let e_neg = rd_u32(&mmap, 20) as usize;
        let e_aff = rd_u32(&mmap, 24) as usize;
        let lex_off   = rd_u64(&mmap, 32) as usize;
        let _lex_size = rd_u64(&mmap, 40) as usize;
        let graph_off = rd_u64(&mmap, 48) as usize;
        let meta_off  = rd_u64(&mmap, 64) as usize;
        let meta_size = rd_u64(&mmap, 72) as usize;

        // ── Parse LEXICON ─────────────────────────────────────────────────
        let mut c = lex_off;

        let str_pool_size = rd_u32(&mmap, c) as usize; c += 4;
        let str_pool = &mmap[c..c + str_pool_size]; c += str_pool_size;

        let str_offsets: Vec<usize> = (0..word_count).map(|_| {
            let v = rd_u32(&mmap, c) as usize; c += 4; v
        }).collect();

        let words: Vec<String> = str_offsets.iter().map(|&off| {
            let end = str_pool[off..].iter().position(|&b| b == 0).unwrap_or(0);
            std::str::from_utf8(&str_pool[off..off + end])
                .unwrap_or("")
                .to_string()
        }).collect();

        let signatures: Vec<[f64; 8]> = (0..word_count).map(|_| {
            let mut sig = [0f64; 8];
            for j in 0..8 { sig[j] = rd_f64(&mmap, c); c += 8; }
            sig
        }).collect();

        let stability: Vec<f32> = (0..word_count).map(|_| { let v = rd_f32(&mmap, c); c += 4; v }).collect();
        let exposure:  Vec<u32> = (0..word_count).map(|_| { let v = rd_u32(&mmap, c); c += 4; v }).collect();
        let pos: Vec<Option<PartOfSpeech>> = (0..word_count).map(|_| { let v = u8_to_pos(mmap[c]); c += 1; v }).collect();

        c += (4 - (c % 4)) % 4; // allineamento

        let total_aff = rd_u32(&mmap, c) as usize; c += 4;
        let aff_ptrs: Vec<usize> = (0..=word_count).map(|_| { let v = rd_u32(&mmap, c) as usize; c += 4; v }).collect();
        let aff_fids = &mmap[c..c + total_aff]; c += total_aff;
        c += (4 - (c % 4)) % 4;
        let aff_vals: Vec<f32> = (0..total_aff).map(|_| { let v = rd_f32(&mmap, c); c += 4; v }).collect();

        let affinities: Vec<Vec<(u8, f32)>> = (0..word_count).map(|i| {
            (aff_ptrs[i]..aff_ptrs[i + 1])
                .map(|j| (aff_fids[j], aff_vals[j]))
                .collect()
        }).collect();

        // ── Parse GRAPH ───────────────────────────────────────────────────
        let mut gc = graph_off + 12; // salta edge counts (già noti dall'header)

        let co_row_ptr  = read_u32_arr(&mmap, &mut gc, word_count + 1);
        let co_col      = read_u32_arr(&mmap, &mut gc, e_co);
        let co_val      = read_u32_arr(&mmap, &mut gc, e_co);
        let neg_row_ptr = read_u32_arr(&mmap, &mut gc, word_count + 1);
        let neg_col     = read_u32_arr(&mmap, &mut gc, e_neg);
        let neg_val     = read_u32_arr(&mmap, &mut gc, e_neg);
        let aff_row_ptr = read_u32_arr(&mmap, &mut gc, word_count + 1);
        let aff_col     = read_u32_arr(&mmap, &mut gc, e_aff);
        let aff_val     = read_u32_arr(&mmap, &mut gc, e_aff);

        // ── META ──────────────────────────────────────────────────────────
        let meta_data = mmap[meta_off..meta_off + meta_size].to_vec();

        // ── PF1 (Phase 39+) — presenti solo se pf1_size > 0 ─────────────
        // Header bytes [88..96] = pf1_offset, [96..104] = pf1_size.
        // In file legacy i byte [88..104] sono zero → pf1 assente.
        let pf1_data: Vec<u8> = if mmap.len() >= 104 {
            let pf1_off  = rd_u64(&mmap, 88) as usize;
            let pf1_size = rd_u64(&mmap, 96) as usize;
            if pf1_size > 0 && pf1_off + pf1_size <= mmap.len() {
                mmap[pf1_off..pf1_off + pf1_size].to_vec()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        Ok(SimplDB {
            words, signatures, stability, exposure, pos, affinities,
            co_row_ptr, co_col, co_val,
            neg_row_ptr, neg_col, neg_val,
            aff_row_ptr, aff_col, aff_val,
            meta_data,
            pf1_data,
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// Helper: scrittura little-endian
// ═══════════════════════════════════════════════════════════════

#[inline] fn push_u32(buf: &mut Vec<u8>, v: u32) { buf.extend_from_slice(&v.to_le_bytes()); }
#[inline] fn push_u64(buf: &mut Vec<u8>, v: u64) { buf.extend_from_slice(&v.to_le_bytes()); }
#[inline] fn push_f32(buf: &mut Vec<u8>, v: f32) { buf.extend_from_slice(&v.to_le_bytes()); }
#[inline] fn push_f64(buf: &mut Vec<u8>, v: f64) { buf.extend_from_slice(&v.to_le_bytes()); }
#[inline] fn align4(buf: &mut Vec<u8>) { while buf.len() % 4 != 0 { buf.push(0); } }

// ═══════════════════════════════════════════════════════════════
// Helper: lettura little-endian
// ═══════════════════════════════════════════════════════════════

#[inline] fn rd_u32(data: &[u8], off: usize) -> u32 { u32::from_le_bytes(data[off..off+4].try_into().unwrap()) }
#[inline] fn rd_u64(data: &[u8], off: usize) -> u64 { u64::from_le_bytes(data[off..off+8].try_into().unwrap()) }
#[inline] fn rd_f32(data: &[u8], off: usize) -> f32 { f32::from_le_bytes(data[off..off+4].try_into().unwrap()) }
#[inline] fn rd_f64(data: &[u8], off: usize) -> f64 { f64::from_le_bytes(data[off..off+8].try_into().unwrap()) }

fn read_u32_arr(data: &[u8], cursor: &mut usize, count: usize) -> Vec<u32> {
    let mut v = Vec::with_capacity(count);
    for _ in 0..count { v.push(rd_u32(data, *cursor)); *cursor += 4; }
    v
}

// ═══════════════════════════════════════════════════════════════
// Helper: PartOfSpeech ↔ u8
// ═══════════════════════════════════════════════════════════════

fn pos_to_u8(pos: &Option<PartOfSpeech>) -> u8 {
    match pos {
        None                             => 0,
        Some(PartOfSpeech::Verb)        => 1,
        Some(PartOfSpeech::Noun)        => 2,
        Some(PartOfSpeech::Adjective)   => 3,
        Some(PartOfSpeech::Adverb)      => 4,
        Some(PartOfSpeech::Pronoun)     => 5,
    }
}

fn u8_to_pos(v: u8) -> Option<PartOfSpeech> {
    match v {
        1 => Some(PartOfSpeech::Verb),
        2 => Some(PartOfSpeech::Noun),
        3 => Some(PartOfSpeech::Adjective),
        4 => Some(PartOfSpeech::Adverb),
        5 => Some(PartOfSpeech::Pronoun),
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════
// Helper: costruzione CSR da WordSnapshot
// ═══════════════════════════════════════════════════════════════

fn build_csr(
    sorted: &[&WordSnapshot],
    word_to_id: &HashMap<&str, WordId>,
    get_edges: impl Fn(&WordSnapshot) -> &Vec<(String, u64)>,
) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let n = sorted.len();
    let mut row_ptr = vec![0u32; n + 1];
    let mut col_all = Vec::new();
    let mut val_all = Vec::new();

    for (i, ws) in sorted.iter().enumerate() {
        // Aggrega co-occorrenze verso lo stesso vicino (HashMap per dedup)
        let mut edge_map: HashMap<u32, u32> = HashMap::new();
        for (other, count) in get_edges(ws) {
            if let Some(&id) = word_to_id.get(other.as_str()) {
                let cnt = (*count).min(u32::MAX as u64) as u32;
                *edge_map.entry(id).or_insert(0) += cnt;
            }
        }
        // Ordina per col_id (CSR richiede righe ordinate per binary search)
        let mut row: Vec<(u32, u32)> = edge_map.into_iter().collect();
        row.sort_by_key(|&(id, _)| id);

        row_ptr[i + 1] = row_ptr[i] + row.len() as u32;
        for (c, v) in row {
            col_all.push(c);
            val_all.push(v);
        }
    }

    (row_ptr, col_all, val_all)
}

// ═══════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::engine::PrometeoTopologyEngine;

    fn engine_with_data() -> PrometeoTopologyEngine {
        let mut e = PrometeoTopologyEngine::new();
        for _ in 0..5 {
            e.receive("io sento la gioia nel cuore");
            e.receive("la luce porta calore e vita");
            e.receive("il tempo scorre come acqua");
        }
        e
    }

    #[test]
    fn test_simpdb_build_and_query() {
        let engine = engine_with_data();
        let state = PrometeoState::capture(&engine);
        let db = SimplDB::from_state(&state);

        assert!(db.word_count() > 0, "Il lessico non può essere vuoto");

        // binary search deve trovare le parole insegnate
        assert!(db.word_id("gioia").is_some(), "gioia deve essere nel lessico");
        assert!(db.word_id("luce").is_some(), "luce deve essere nel lessico");
        assert!(db.word_id("xxxxxxx").is_none(), "parola inesistente deve ritornare None");
    }

    #[test]
    fn test_simpdb_neighbors_csr() {
        let engine = engine_with_data();
        let state = PrometeoState::capture(&engine);
        let db = SimplDB::from_state(&state);

        if let Some(id) = db.word_id("gioia") {
            let neighbors: Vec<_> = db.neighbors(id).collect();
            // gioia co-occorre con cuore, luce, ecc.
            assert!(!neighbors.is_empty(), "gioia deve avere vicini nel grafo");
            // Ogni vicino è un word_id valido
            for (nid, _count) in &neighbors {
                assert!((*nid as usize) < db.word_count(), "word_id vicino fuori range");
            }
        }
    }

    #[test]
    fn test_simpdb_save_open_round_trip() {
        let engine = engine_with_data();
        let state = PrometeoState::capture(&engine);
        let db = SimplDB::from_state(&state);

        let tmp = std::env::temp_dir().join("prometeo_test_simpdb_v3.bin");
        db.save(&tmp).unwrap();

        // Magic bytes corretti
        let bytes = std::fs::read(&tmp).unwrap();
        assert_eq!(&bytes[0..8], b"SIMPDB03", "Magic SIMPDB03 errato");

        // Riapri e verifica identità
        let db2 = SimplDB::open(&tmp).unwrap();
        assert_eq!(db2.word_count(), db.word_count(), "word_count deve essere identico");

        // Ogni parola deve essere identica (stesso ordine lessicografico)
        for (i, word) in db.words.iter().enumerate() {
            assert_eq!(db2.words[i], *word, "parola #{} diversa", i);
            assert_eq!(db2.signatures[i], db.signatures[i], "firma #{} diversa", i);
        }

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_simpdb_to_state_round_trip() {
        let mut engine1 = PrometeoTopologyEngine::new();
        for _ in 0..3 {
            engine1.receive("io sento il tempo che passa");
            engine1.receive("la gioia del colore rosso e caldo");
        }

        let state1 = PrometeoState::capture(&engine1);
        let db = SimplDB::from_state(&state1);
        let state2 = db.to_state().unwrap();

        assert_eq!(state2.total_perturbations, state1.total_perturbations);
        assert_eq!(state2.lexicon.words.len(), state1.lexicon.words.len());

        // Le parole devono essere tutte presenti (in ordine diverso — le nostre sono alfabetiche)
        let words1: std::collections::HashSet<_> = state1.lexicon.words.iter().map(|w| &w.word).collect();
        let words2: std::collections::HashSet<_> = state2.lexicon.words.iter().map(|w| &w.word).collect();
        assert_eq!(words1, words2, "Il set di parole deve essere identico");
    }

    #[test]
    fn test_simpdb_topological_neighborhood() {
        let engine = engine_with_data();
        let state = PrometeoState::capture(&engine);
        let db = SimplDB::from_state(&state);

        if let Some(id) = db.word_id("gioia") {
            let neighbors_1hop = db.topological_neighborhood(id, 1);
            let neighbors_2hop = db.topological_neighborhood(id, 2);
            // 2 hop deve includere almeno tutti quelli a 1 hop
            assert!(neighbors_2hop.len() >= neighbors_1hop.len(),
                "2 hop deve coprire almeno tutti i vicini a 1 hop");
        }
    }

    #[test]
    fn test_simpdb_full_lifecycle() {
        // Ciclo completo: state → SimplDB → file → open → to_state → restore engine
        let mut engine1 = PrometeoTopologyEngine::new();
        for _ in 0..5 {
            engine1.receive("io sento il tempo che passa");
        }

        let state1 = PrometeoState::capture(&engine1);
        let db = SimplDB::from_state(&state1);

        let tmp = std::env::temp_dir().join("prometeo_test_simpdb_lifecycle.bin");
        db.save(&tmp).unwrap();

        let db2 = SimplDB::open(&tmp).unwrap();
        let state2 = db2.to_state().unwrap();
        let mut engine2 = PrometeoTopologyEngine::new();
        state2.restore_lexicon(&mut engine2);

        assert_eq!(engine2.total_perturbations, engine1.total_perturbations,
            "Perturbazioni devono sopravvivere al ciclo SimplDB");
        assert_eq!(engine2.complex.count(), engine1.complex.count(),
            "Simplessi devono sopravvivere al ciclo SimplDB");

        let _ = std::fs::remove_file(&tmp);
    }
}
