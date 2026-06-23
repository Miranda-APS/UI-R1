/// Persistenza — Serializzazione dello stato topologico.
///
/// Il sistema "cresce" sessione dopo sessione.
/// Questo modulo salva e carica l'intero stato del complesso,
/// la memoria e il lessico.
///
/// Formati supportati:
/// - JSON (legacy, human-readable, lento su file grandi)
/// - SimplDB (binario, ~5x più veloce, ~3x più piccolo, memory-mapped)

use std::path::Path;
use std::collections::HashMap;

use crate::topology::simpdb::{SimplDB, SIMPDB_MAGIC_V2, SIMPDB_MAGIC_V3};
use serde::{Serialize, Deserialize};

use crate::topology::primitive::{PrimitiveCore, Dim};
use crate::topology::fractal::FractalId;
use crate::topology::simplex::SimplexId;
use crate::topology::lexicon::SemanticAxisSnapshot;
use crate::topology::knowledge::KnowledgeSnapshot;
use crate::topology::grammar::PartOfSpeech;
use crate::topology::episodic::EpisodeSnapshot;
use crate::topology::identity::IdentitySnapshot;
use crate::topology::narrative::NarrativeSnapshot;
use crate::topology::self_model::SelfModelSnapshot;
use crate::topology::semantic_episode::SemanticEpisodeLog;

/// Snapshot serializzabile del lessico.
#[derive(Serialize, Deserialize, Debug)]
pub struct LexiconSnapshot {
    pub words: Vec<WordSnapshot>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WordSnapshot {
    pub word: String,
    pub signature: [f64; 8],
    pub fractal_affinities: Vec<(FractalId, f64)>,
    pub exposure_count: u64,
    pub stability: f64,
    /// Co-occorrenze raw — presenti solo se la parola non è in PF1 (nuove, non ancora distillate).
    /// Per parole in PF1 con vicini: svuotato in capture(), sostituito da neighbors_pf1.
    pub co_occurrences: Vec<(String, u64)>,
    /// Co-occorrenze negate (operatore strutturale "non").
    /// Default vuoto per retrocompatibilita con salvataggi precedenti.
    #[serde(default)]
    pub co_negated: Vec<(String, u64)>,
    /// Co-occorrenze affermate esplicitamente (operatori "come", "simile", "uguale"...).
    /// Usate come denominatore nella fase degli archi. Default vuoto per retrocompat.
    #[serde(default)]
    pub co_affirmed: Vec<(String, u64)>,
    /// Categoria grammaticale (opzionale, retrocompatibile).
    #[serde(default)]
    pub pos: Option<PartOfSpeech>,
    /// Top-8 vicini PF1 (nome_vicino, peso, fase) — la topologia ROM distillata.
    ///
    /// Presente quando la parola è in PF1 con almeno 1 vicino. In questo caso
    /// co_occurrences è vuoto: PF1 è il substrato, le co_occ sono l'input di build.
    /// Default vuoto per retrocompatibilità con file salvati prima di questa versione.
    #[serde(default)]
    pub neighbors_pf1: Vec<(String, f32, f32)>,
}

/// Snapshot serializzabile di una SharedFace — ricostruibile.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SharedFaceSnapshot {
    /// Tipo: "dim" (PrimitiveDim) o "property" (EmergentProperty) o "covariation"
    pub structure_type: String,
    /// Valore: nome della dimensione, oppure nome della proprietà
    pub structure_value: String,
    /// Per CovariationPattern: le dimensioni coinvolte (indici)
    #[serde(default)]
    pub covariation_dims: Vec<u8>,
    /// Forza della condivisione [0.0, 1.0]
    pub strength: f64,
    /// Manifestazioni: (fractal_id, descrizione)
    #[serde(default)]
    pub manifestations: Vec<(FractalId, String)>,
}

/// Snapshot serializzabile del complesso simpliciale.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComplexSnapshot {
    pub simplices: Vec<SimplexSnapshot>,
    pub next_id: SimplexId,
    pub activation_threshold: f64,
}

/// Phase 83 fallback — snapshot del complesso PRIMA dell'aggiunta dei
/// campi grammaticali ai simplessi (`category`, `ordered`, `function_fractal`).
/// Usato per deserializzare .bin pre-P83 e convertirli al formato corrente
/// con tutti i simplessi default-semantici.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComplexSnapshotPreP83 {
    pub simplices: Vec<SimplexSnapshotPreP83>,
    pub next_id: SimplexId,
    pub activation_threshold: f64,
}

/// Phase 83 fallback — snapshot simplesso pre-P83 (senza category/ordered/
/// function_fractal). bincode è strict-positional: deserializzare un .bin
/// vecchio con la struct nuova fallisce sui campi aggiunti. Questo struct
/// rispecchia il formato pre-P83 esatto.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimplexSnapshotPreP83 {
    pub id: SimplexId,
    pub vertices: Vec<FractalId>,
    pub dimension: usize,
    pub persistence: f64,
    pub plasticity: f64,
    pub activation_count: u64,
    #[serde(default)]
    pub faces: Vec<SharedFaceSnapshot>,
    #[serde(default)]
    pub face_descriptions: Vec<String>,
    #[serde(default)]
    pub source_words: Option<Vec<String>>,
}

impl From<ComplexSnapshotPreP83> for ComplexSnapshot {
    fn from(l: ComplexSnapshotPreP83) -> Self {
        let simplices = l.simplices.into_iter().map(|s| SimplexSnapshot {
            id: s.id,
            vertices: s.vertices,
            dimension: s.dimension,
            persistence: s.persistence,
            plasticity: s.plasticity,
            activation_count: s.activation_count,
            faces: s.faces,
            face_descriptions: s.face_descriptions,
            source_words: s.source_words,
            // Phase 83 — backward compat: simplessi del .bin pre-P83 sono
            // tutti semantici (Phase 49-52), nessuno grammaticale.
            category: None,
            ordered: false,
            function_fractal: None,
        }).collect();
        ComplexSnapshot {
            simplices,
            next_id: l.next_id,
            activation_threshold: l.activation_threshold,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimplexSnapshot {
    pub id: SimplexId,
    pub vertices: Vec<FractalId>,
    pub dimension: usize,
    pub persistence: f64,
    pub plasticity: f64,
    pub activation_count: u64,
    /// Facce condivise — formato strutturato ricostruibile (v1.3+)
    #[serde(default)]
    pub faces: Vec<SharedFaceSnapshot>,
    /// Legacy: descrizioni testuali (retrocompat v1.2)
    #[serde(default)]
    pub face_descriptions: Vec<String>,
    /// Phase 52: parole sorgente (proposizioni inscritte). Default None per retrocompat.
    #[serde(default)]
    pub source_words: Option<Vec<String>>,
    /// Phase 83: categoria del simplesso (es. "preposizione_composta",
    /// "locuzione_fatica"). None = semantico (default, backward compat).
    #[serde(default)]
    pub category: Option<String>,
    /// Phase 83: se true, source_words devono attivarsi in ordine adiacente.
    /// Default false per backward compat (simplessi semantici Phase 49-52).
    #[serde(default)]
    pub ordered: bool,
    /// Phase 83: frattale-funzione attivato dal simplesso quando emerge.
    /// None = simplesso semantico (no effetto strutturale al campo).
    #[serde(default)]
    pub function_fractal: Option<FractalId>,
}

/// Snapshot serializzabile della memoria.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemorySnapshot {
    pub mtm: Vec<ImprintSnapshot>,
    pub ltm: Vec<ImprintSnapshot>,
    pub current_tick: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImprintSnapshot {
    pub active_simplices: Vec<(SimplexId, f64)>,
    pub involved_fractals: Vec<FractalId>,
    pub tick: u64,
    pub strength: f64,
    pub origin: String,
}

/// Snapshot serializzabile del locus.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocusSnapshot {
    /// Posizione corrente (None = non ancora posizionato)
    pub position: Option<FractalId>,
    /// Raggio dell'orizzonte percettivo
    pub horizon: f64,
    /// Soglia per jump vs traverse
    pub jump_threshold: f64,
    /// Trail delle posizioni recenti
    pub trail: Vec<FractalId>,
    /// Sub-locus: posizione nelle dimensioni libere (indice dim → valore)
    #[serde(default)]
    pub sub_position: Vec<(u8, f64)>,
}

/// Record di una lezione completata.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LessonRecord {
    /// Nome della lezione (es. "00_corpo", "01_spazio")
    pub name: String,
    /// Parole insegnate in questa lezione
    pub words_taught: Vec<String>,
    /// Timestamp ISO 8601
    pub timestamp: String,
}

/// Progresso del curriculum: quali lezioni sono state fatte.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CurriculumProgress {
    /// Lezioni completate in ordine
    pub lessons_completed: Vec<LessonRecord>,
    /// Totale parole apprese (lessico - parole cardinali)
    pub total_words_learned: usize,
}

impl CurriculumProgress {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra una lezione completata.
    pub fn record_lesson(&mut self, name: &str, words: Vec<String>) {
        let timestamp = chrono_now();
        self.total_words_learned += words.len();
        self.lessons_completed.push(LessonRecord {
            name: name.to_string(),
            words_taught: words,
            timestamp,
        });
    }

    /// La lezione e gia stata fatta?
    pub fn has_lesson(&self, name: &str) -> bool {
        self.lessons_completed.iter().any(|l| l.name == name)
    }
}

/// Timestamp corrente come stringa ISO-like (senza dipendenza chrono).
fn chrono_now() -> String {
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("t+{}s", dur.as_secs())
}

/// Stato completo del sistema.
#[derive(Serialize, Deserialize, Debug)]
pub struct PrometeoState {
    pub version: String,
    pub total_perturbations: u64,
    pub dream_cycles: u64,
    pub lexicon: LexiconSnapshot,
    pub complex: ComplexSnapshot,
    pub memory: MemorySnapshot,
    /// Locus (opzionale per retrocompatibilita con vecchi salvataggi)
    #[serde(default)]
    pub locus: Option<LocusSnapshot>,
    /// Curriculum: lezioni fatte (opzionale per retrocompatibilita)
    #[serde(default)]
    pub curriculum: Option<CurriculumProgress>,
    /// Assi semantici rilevati (opzionale per retrocompatibilita)
    #[serde(default)]
    pub semantic_axes: Option<Vec<SemanticAxisSnapshot>>,
    /// Memoria procedurale: voci insegnate dall'utente (opzionale per retrocompat)
    #[serde(default)]
    pub knowledge: Option<KnowledgeSnapshot>,
    /// Memoria episodica — Phase 28 (opzionale per retrocompat con salvataggi precedenti)
    #[serde(default)]
    pub episodes: Option<EpisodeSnapshot>,
    /// Unix timestamp della prima creazione dell'istanza (opzionale per retrocompat).
    /// Se assente in salvataggi vecchi, viene impostato al momento del caricamento.
    #[serde(default)]
    pub instance_born: Option<u64>,
    /// Nucleo identitario olografico — Phase 34 (opzionale per retrocompat).
    #[serde(default)]
    pub identity: Option<IdentitySnapshot>,
    /// Identità narrativa — Phase 42/43 (opzionale per retrocompat).
    #[serde(default)]
    pub narrative: Option<NarrativeSnapshot>,
    /// SelfModel — credenze, valori, incertezze esplicite (opzionale per retrocompat).
    #[serde(default)]
    pub self_model: Option<SelfModelSnapshot>,
    /// Episodi semantici — log navigabile delle interazioni (opzionale per retrocompat).
    #[serde(default)]
    pub semantic_episodes: Option<SemanticEpisodeLog>,
    /// Phase 54: desideri attivi (opzionale per retrocompat).
    #[serde(default)]
    pub desire: Option<crate::topology::desire::DesireSnapshot>,
    /// Phase 54: eco dell'Altro (opzionale per retrocompat).
    #[serde(default)]
    pub interlocutor: Option<crate::topology::interlocutor::InterlocutorSnapshot>,
    /// P2 (Tsunami): memoria del parlante — il "ritratto-utente" (self_facts,
    /// entity_facts, open_questions, gaps, mentioned, name, corrections).
    /// Persiste cross-sessione la continuità dell'essere-conosciuto.
    /// Opzionale per retrocompat con salvataggi precedenti.
    #[serde(default)]
    pub speaker_profile: Option<crate::topology::speaker_profile::SpeakerProfile>,
}

// ═══════════════════════════════════════════════════════════════
// Conversioni Engine → Snapshot
// ═══════════════════════════════════════════════════════════════

impl PrometeoState {
    /// Cattura lo stato corrente dell'engine.
    pub fn capture(engine: &crate::topology::engine::PrometeoTopologyEngine) -> Self {
        // Lessico — PF1 come ROM: per parole in PF1 con vicini, sostituisce co_occurrences.
        // Questo è il cuore della riduzione: 5M co_occ → 1M vicini PF1 → .bin da 167MB → ~65MB.
        let words: Vec<WordSnapshot> = engine.lexicon.most_stable(usize::MAX).iter()
            .map(|pat| {
                // Controlla se la parola ha vicini in PF1 (topologia ROM già costruita)
                let pf1_nbrs: Vec<(String, f32, f32)> =
                    engine.pf_field.word_id(&pat.word)
                        .map(|pf1_id| {
                            let record = engine.pf_field.record(pf1_id);
                            if record.neighbor_count > 0 {
                                (0..record.neighbor_count as usize)
                                    .map(|i| {
                                        let nid = record.neighbors[i];
                                        let nword = engine.pf_field.word_name(nid).to_string();
                                        (nword, record.neighbor_weights[i], record.neighbor_phases[i])
                                    })
                                    .collect()
                            } else {
                                vec![]
                            }
                        })
                        .unwrap_or_default();

                // Se PF1 ha vicini per questa parola → usa PF1 come ROM, svuota co_occ.
                // Altrimenti (parola nuova non ancora distillata in PF1) → tieni co_occ.
                let use_pf1 = !pf1_nbrs.is_empty();

                WordSnapshot {
                    word: pat.word.clone(),
                    signature: *pat.signature.values(),
                    fractal_affinities: pat.fractal_affinities.iter()
                        .map(|(&k, &v)| (k, v)).collect(),
                    exposure_count: pat.exposure_count,
                    stability: pat.stability,
                    co_occurrences: if use_pf1 { vec![] } else {
                        pat.co_occurrences.iter().map(|(k, &v)| (k.clone(), v)).collect()
                    },
                    co_negated: if use_pf1 { vec![] } else {
                        pat.co_negated.iter().map(|(k, &v)| (k.clone(), v)).collect()
                    },
                    co_affirmed: if use_pf1 { vec![] } else {
                        pat.co_affirmed.iter().map(|(k, &v)| (k.clone(), v)).collect()
                    },
                    pos: pat.pos.clone(),
                    neighbors_pf1: pf1_nbrs,
                }
            })
            .collect();

        // Complesso — salva le facce in formato strutturato ricostruibile
        let simplices: Vec<SimplexSnapshot> = engine.complex.iter()
            .map(|(&id, s)| {
                let faces: Vec<SharedFaceSnapshot> = s.shared_faces.iter()
                    .map(|f| {
                        let (stype, svalue, covdims) = match &f.structure {
                            crate::topology::simplex::SharedStructureType::PrimitiveDim(dim) => {
                                ("dim".to_string(), format!("{:?}", dim), vec![])
                            }
                            crate::topology::simplex::SharedStructureType::EmergentProperty(name) => {
                                ("property".to_string(), name.clone(), vec![])
                            }
                            crate::topology::simplex::SharedStructureType::CovariationPattern { dims, description } => {
                                let dim_indices: Vec<u8> = dims.iter()
                                    .map(|d| d.index() as u8)
                                    .collect();
                                ("covariation".to_string(), description.clone(), dim_indices)
                            }
                        };
                        SharedFaceSnapshot {
                            structure_type: stype,
                            structure_value: svalue,
                            covariation_dims: covdims,
                            strength: f.strength,
                            manifestations: f.manifestations.iter()
                                .map(|(&fid, desc)| (fid, desc.clone()))
                                .collect(),
                        }
                    })
                    .collect();

                SimplexSnapshot {
                    id,
                    vertices: s.vertices.clone(),
                    dimension: s.dimension,
                    persistence: s.persistence,
                    plasticity: s.plasticity,
                    activation_count: s.activation_count,
                    faces,
                    face_descriptions: vec![], // legacy vuoto
                    source_words: s.source_words.clone(),
                    // Phase 83 — simplessi grammaticali persistenti
                    category: s.category.clone(),
                    ordered: s.ordered,
                    function_fractal: s.function_fractal,
                }
            })
            .collect();

        // Memoria (solo MTM e LTM, la STM e effimera)
        let mtm: Vec<ImprintSnapshot> = engine.memory.medium_term.iter()
            .map(|imp| ImprintSnapshot {
                active_simplices: imp.active_simplices.clone(),
                involved_fractals: imp.involved_fractals.clone(),
                tick: imp.tick,
                strength: imp.strength,
                origin: imp.origin.clone(),
            })
            .collect();

        let ltm: Vec<ImprintSnapshot> = engine.memory.long_term.iter()
            .map(|imp| ImprintSnapshot {
                active_simplices: imp.active_simplices.clone(),
                involved_fractals: imp.involved_fractals.clone(),
                tick: imp.tick,
                strength: imp.strength,
                origin: imp.origin.clone(),
            })
            .collect();

        // Locus (incluso sub-locus)
        let sub_pos: Vec<(u8, f64)> = engine.locus.sub_position.iter()
            .map(|(dim, &val)| (dim.index() as u8, val))
            .collect();
        let locus = Some(LocusSnapshot {
            position: engine.locus.position,
            horizon: engine.locus.horizon,
            jump_threshold: engine.locus.jump_threshold,
            trail: engine.locus.trail.clone(),
            sub_position: sub_pos,
        });

        PrometeoState {
            version: "8D-v1.3".to_string(),
            total_perturbations: engine.total_perturbations,
            dream_cycles: engine.dream.cycles_completed,
            lexicon: LexiconSnapshot { words },
            complex: ComplexSnapshot {
                simplices,
                next_id: engine.complex.count() as u32 + 100, // margine
                activation_threshold: engine.complex.activation_threshold,
            },
            memory: MemorySnapshot {
                mtm,
                ltm,
                current_tick: engine.memory.current_tick,
            },
            locus,
            curriculum: Some(engine.curriculum.clone()),
            semantic_axes: Some(engine.semantic_axes.iter()
                .map(|a| a.to_snapshot())
                .collect()),
            knowledge: Some(KnowledgeSnapshot::capture(&engine.knowledge_base)),
            episodes: Some(engine.episode_store.snapshot()),
            instance_born: Some(engine.instance_born),
            identity: Some(engine.identity.to_snapshot()),
            narrative: Some(engine.narrative_self.capture()),
            self_model: Some(engine.self_model.to_snapshot()),
            semantic_episodes: Some(engine.semantic_episodes.clone()),
            desire: Some(engine.desire.snapshot()),
            interlocutor: Some(engine.interlocutor.snapshot()),
            speaker_profile: Some(engine.speaker_profile.clone()),
        }
    }

    /// Salva su file JSON (formato legacy, human-readable).
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Errore serializzazione: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("Errore scrittura file: {}", e))
    }

    /// Carica da file JSON (formato legacy).
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Errore lettura file: {}", e))?;
        serde_json::from_str(&json)
            .map_err(|e| format!("Errore deserializzazione: {}", e))
    }

    /// Salva in formato SimplDB v3 (sezionato, portabile, topologicamente strutturato).
    ///
    /// Formato: header 128 bytes + sezione LEXICON (sorted CSR) + GRAPH (CSR) + META (bincode).
    /// Leggibile da Kotlin/Swift senza Rust. Tutte le query topologiche native.
    pub fn save_to_binary(&self, path: &Path) -> Result<(), String> {
        let db = SimplDB::from_state(self);
        db.save(path)
    }

    /// Carica da file SimplDB con auto-detection del formato.
    ///
    /// - SIMPDB03 (v3, sezionato): usa SimplDB::open()
    /// - SIMPDB02 (v2, bincode blob): caricamento legacy con bincode + mmap
    pub fn load_from_binary(path: &Path) -> Result<Self, String> {
        use memmap2::Mmap;
        use std::fs::File;

        // Legge i magic bytes per scegliere il parser
        let file = File::open(path)
            .map_err(|e| format!("Errore apertura file: {}", e))?;
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| format!("Errore mmap: {}", e))?;

        if mmap.len() < 8 {
            return Err("File binario corrotto (< 8 bytes)".to_string());
        }

        match &mmap[..8] {
            m if m == SIMPDB_MAGIC_V3 => {
                // Formato v3: usa SimplDB strutturato
                drop(mmap); // rilascia il primo mmap prima di riaprire
                SimplDB::open(path)?.to_state()
            }
            m if m == SIMPDB_MAGIC_V2 => {
                // Formato v2 legacy: bincode blob
                bincode::deserialize(&mmap[8..])
                    .map_err(|e| format!("Errore deserializzazione SIMPDB02: {}", e))
            }
            magic => Err(format!(
                "Magic sconosciuto: {:?} — non è un file SimplDB valido",
                magic
            )),
        }
    }

    /// Dimensione stimata del file binario v3 (pre-scrittura, in bytes).
    pub fn binary_size_estimate(&self) -> usize {
        SimplDB::from_state(self).estimated_file_size()
    }

    /// Ripristina il lessico nell'engine.
    /// Usa insert_pattern() per ripristinare lo stato ESATTO salvato:
    /// firma, affinita, stabilita, esposizioni, co-occorrenze.
    pub fn restore_lexicon(&self, engine: &mut crate::topology::engine::PrometeoTopologyEngine) {
        for ws in &self.lexicon.words {
            let sig = PrimitiveCore::new(ws.signature);
            let mut pat = crate::topology::lexicon::WordPattern::new_known(
                &ws.word,
                sig,
                ws.fractal_affinities.clone(),
            );
            // Ripristina lo stato esatto (new_known imposta exposure=10, stability=0.6)
            pat.exposure_count = ws.exposure_count;
            pat.stability = ws.stability;
            // Ripristina co-occorrenze affermate (salvate come Vec<(String, u64)>)
            for (other_word, count) in &ws.co_occurrences {
                for _ in 0..*count {
                    pat.register_co_occurrence(other_word);
                }
            }
            // Ripristina co-occorrenze negate (operatori strutturali)
            for (other_word, count) in &ws.co_negated {
                for _ in 0..*count {
                    pat.register_co_negation(other_word);
                }
            }
            // Ripristina co-occorrenze affermate esplicitamente (operatori "come", "simile"...)
            for (other_word, count) in &ws.co_affirmed {
                for _ in 0..*count {
                    pat.register_co_affirmation(other_word);
                }
            }
            // Ripristina POS se salvata
            pat.pos = ws.pos.clone();
            // Inserisci con stato completo, sovrascrivendo quello bootstrap
            engine.lexicon.insert_pattern(&ws.word, pat);
        }
        // Tagga come Verb tutte le parole infinitive senza POS (stato file precedente)
        engine.lexicon.tag_pos_from_forms();

        engine.total_perturbations = self.total_perturbations;
        engine.memory.current_tick = self.memory.current_tick;

        // Ripristina il locus (incluso sub-locus)
        if let Some(ref locus_snap) = self.locus {
            engine.locus.position = locus_snap.position;
            engine.locus.horizon = locus_snap.horizon;
            engine.locus.jump_threshold = locus_snap.jump_threshold;
            engine.locus.trail = locus_snap.trail.clone();
            // Ripristina sub-locus
            engine.locus.sub_position.clear();
            for &(dim_idx, val) in &locus_snap.sub_position {
                if let Some(&dim) = Dim::ALL.get(dim_idx as usize) {
                    engine.locus.sub_position.insert(dim, val);
                }
            }
        }

        // Ripristina curriculum
        if let Some(ref curr) = self.curriculum {
            engine.curriculum = curr.clone();
        }

        // Ripristina assi semantici
        if let Some(ref axes) = self.semantic_axes {
            engine.semantic_axes = axes.iter()
                .map(|a| crate::topology::lexicon::SemanticAxis::from_snapshot(a))
                .collect();
        }

        // Ripristina memoria procedurale (solo le voci insegnate dall'utente;
        // i template di default vengono sempre ricreati dal codice)
        if let Some(ref snap) = self.knowledge {
            engine.knowledge_base = snap.clone().restore();
        }

        // ═══════════════════════════════════════════════════════════
        // Ripristina il complesso simpliciale
        // ═══════════════════════════════════════════════════════════
        // Svuota il complesso bootstrap e ricostruisce da snapshot.
        // Questo è il pezzo cruciale: senza questo, OGNI restart
        // perde tutte le connessioni topologiche (simplessi, bridges,
        // compound inscritti) e il sistema "riforma da zero".
        engine.complex.clear();

        for ss in &self.complex.simplices {
            let shared_faces = Self::reconstruct_faces(ss);
            engine.complex.restore_simplex(
                ss.id,
                ss.vertices.clone(),
                shared_faces,
                ss.persistence,
                ss.plasticity,
                ss.activation_count,
                ss.source_words.clone(),
                ss.category.clone(),
                ss.ordered,
                ss.function_fractal,
            );
        }

        // Pulisce simplici dinamici inattivi accumulati nelle sessioni precedenti.
        // I simplici bootstrap (primi ~120) vengono preservati sempre.
        // Rimuove solo simplici con attivazione sotto soglia e persistenza < 0.5.
        // Questo previene il blocco O(N^2) in propagate_activation con complessi gonfiati.
        engine.complex.prune_low_activity(200);

        // Ricostruisce il campo topologico delle parole.
        //
        // STRATEGIA DUALE:
        //   • File post-corpus (Phase 39+): usa neighbors_pf1 — la topologia ROM distillata.
        //     PF1 ha i top-8 vicini per parola → O(N×8) invece di O(5M co_occ). Veloce.
        //   • File legacy (senza neighbors_pf1): usa co_occurrences → build_from_lexicon().
        //     Retrocompatibilità con salvataggi precedenti.
        let has_pf1_topology = self.lexicon.words.iter().any(|ws| !ws.neighbors_pf1.is_empty());

        if has_pf1_topology {
            // PATH VELOCE: ricostruisce word_topology dai vicini PF1 (max 8 per parola).
            // Non serve iterare 5M co_occ — il campo è già distillato nella ROM.
            let mut topo = crate::topology::word_topology::WordTopology::new();
            // Fase 1: tutti i vertici
            for ws in &self.lexicon.words {
                topo.add_word(&ws.word);
            }
            // Fase 2: archi dai vicini PF1
            for ws in &self.lexicon.words {
                for (nbr, weight, phase) in &ws.neighbors_pf1 {
                    if let (Some(wa), Some(wb)) = (topo.word_id(&ws.word), topo.word_id(nbr)) {
                        if wa < wb {
                            topo.add_edge_with_phase(wa, wb, *weight as f64, 1, *phase as f64);
                        }
                    }
                }
                // Parole senza PF1 (nuove, non ancora distillate): usa co_occurrences se presenti
                if ws.neighbors_pf1.is_empty() && !ws.co_occurrences.is_empty() {
                    for (other, &count) in ws.co_occurrences.iter()
                        .map(|(k, v)| (k, v))
                    {
                        if let (Some(wa), Some(wb)) = (topo.word_id(&ws.word), topo.word_id(other)) {
                            if wa < wb {
                                let w = ((count as f64).ln().max(0.01)).min(1.0);
                                topo.add_edge_with_phase(wa, wb, w, count, std::f64::consts::FRAC_PI_2);
                            }
                        }
                    }
                }
            }
            engine.word_topology = topo;
        } else {
            // PATH LEGACY: co_occurrenze → word_topology (funziona ma lento con file grandi)
            engine.word_topology = crate::topology::word_topology::WordTopology::build_from_lexicon(&engine.lexicon);
        }

        // Stato di riposo: seme iniziale di attivazione proporzionale alla stability.
        // L'entita "si sveglia" con le sue parole piu familiari gia presenti nel campo.
        // Senza questo, il campo e topologicamente ricco ma "buio" — zero attivazione
        // finche l'utente non parla. Con questo, i concetti piu stabili (io, sentire,
        // gioia...) sono gia presenti, e il saluto emerge senza warm-up.
        engine.word_topology.seed_resting_state(&engine.lexicon);

        // Locus iniziale: se il locus non e stato salvato (None), posiziona l'entita
        // nel frattale dominante del campo resting. Senza questo, il locus resta None
        // finche l'entita non riceve un input con parola sufficientemente stabile.
        if engine.locus.position.is_none() {
            let fractal_activations = engine.word_topology
                .emerge_fractal_activations(&engine.lexicon);
            if let Some((fid, _)) = fractal_activations.iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                engine.locus.move_to(*fid, &engine.complex, &engine.registry);
            }
        }

        // Ripristina la memoria episodica — Phase 28.
        // Gli episodi decadono con φ — riattivano pattern passati nel campo presente.
        if let Some(snap) = self.episodes.clone() {
            engine.episode_store.restore(snap);
        }

        // Ripristina instance_born — preserva l'età dell'entità tra le sessioni.
        // Se assente (salvataggio vecchio), usa il timestamp corrente come fallback.
        if let Some(born) = self.instance_born {
            engine.instance_born = born;
        }
        // last_interaction_ts viene sempre resettato al boot: il silenzio
        // tra le sessioni non si accumula — ogni avvio è un risveglio fresco.

        // Ri-calibra le dimensioni emergenti dal lessico ripristinato.
        // Le emergenti si ricalcolano dalla distribuzione reale delle parole.
        engine.recalibrate_emergent_dimensions();

        // Ripristina IdentityCore — Phase 34.
        // Se esiste uno snapshot, ripristina storia + tensione primaria.
        // Poi aggiorna con il lessico corrente per ricalcolare la proiezione fresca.
        engine.identity = if let Some(ref snap) = self.identity {
            let mut id = crate::topology::identity::IdentityCore::from_snapshot(snap);
            id.update(&engine.lexicon, &engine.word_topology);
            id
        } else {
            crate::topology::identity::IdentityCore::build(&engine.lexicon, &engine.word_topology)
        };

        // Ripristina NarrativeSelf — Phase 42/43.
        // Cristallizzato + posizioni + is_born persistono tra sessioni.
        if let Some(snap) = self.narrative.clone() {
            snap.restore_into(&mut engine.narrative_self);
        }

        // Ripristina SelfModel — credenze, valori, incertezze esplicite.
        // Se assente (salvataggio vecchio), mantiene il bootstrap di default.
        if let Some(snap) = self.self_model.clone() {
            engine.self_model = crate::topology::self_model::SelfModel::from_snapshot(snap);
        }

        // Ripristina episodi semantici.
        if let Some(log) = self.semantic_episodes.clone() {
            engine.semantic_episodes = log;
        }

        // Phase 54: ripristina desideri e interlocutore.
        if let Some(snap) = self.desire.clone() {
            engine.desire = crate::topology::desire::DesireCore::from_snapshot(&snap);
        }
        if let Some(snap) = self.interlocutor.clone() {
            engine.interlocutor = crate::topology::interlocutor::InterlocutorModel::from_snapshot(&snap);
        }

        // P2 (Tsunami): ripristina la memoria del parlante (ritratto-utente).
        if let Some(profile) = self.speaker_profile.clone() {
            engine.speaker_profile = profile;
        }
    }

    /// Ricostruisce le SharedFace da uno snapshot.
    /// Supporta sia il formato v1.3 (faces strutturate) sia il legacy v1.2
    /// (face_descriptions come stringhe) con best-effort parsing.
    fn reconstruct_faces(ss: &SimplexSnapshot) -> Vec<crate::topology::simplex::SharedFace> {
        use crate::topology::simplex::{SharedFace, SharedStructureType};

        if !ss.faces.is_empty() {
            // Formato v1.3: ricostruzione completa
            ss.faces.iter().filter_map(|fs| {
                let structure = match fs.structure_type.as_str() {
                    "dim" => {
                        Dim::from_name(&fs.structure_value)
                            .map(SharedStructureType::PrimitiveDim)?
                    }
                    "property" => {
                        SharedStructureType::EmergentProperty(fs.structure_value.clone())
                    }
                    "covariation" => {
                        let dims: Vec<Dim> = fs.covariation_dims.iter()
                            .filter_map(|&idx| Dim::ALL.get(idx as usize).copied())
                            .collect();
                        SharedStructureType::CovariationPattern {
                            dims,
                            description: fs.structure_value.clone(),
                        }
                    }
                    _ => return None,
                };
                let mut face = SharedFace {
                    structure,
                    manifestations: std::collections::HashMap::new(),
                    strength: fs.strength,
                };
                for (fid, desc) in &fs.manifestations {
                    face.manifestations.insert(*fid, desc.clone());
                }
                Some(face)
            }).collect()
        } else if !ss.face_descriptions.is_empty() {
            // Legacy v1.2: best-effort parsing da stringhe
            ss.face_descriptions.iter().filter_map(|desc| {
                // Formato: "PrimitiveDim(Intensita) (str=0.60)"
                //       o: "EmergentProperty(\"variazione\") (str=0.70)"
                let strength = desc.rsplit("str=").next()
                    .and_then(|s| s.trim_end_matches(')').parse::<f64>().ok())
                    .unwrap_or(0.5);

                if desc.starts_with("PrimitiveDim(") {
                    let dim_name = desc.trim_start_matches("PrimitiveDim(")
                        .split(')').next().unwrap_or("");
                    Dim::from_name(dim_name)
                        .map(|d| SharedFace::from_dim(d, strength))
                } else if desc.starts_with("EmergentProperty(") {
                    let prop = desc.trim_start_matches("EmergentProperty(\"")
                        .split('"').next().unwrap_or("unknown");
                    Some(SharedFace::from_property(prop, strength))
                } else {
                    // Fallback: proprietà emergente con descrizione completa
                    Some(SharedFace::from_property(desc, strength))
                }
            }).collect()
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::engine::PrometeoTopologyEngine;
    use std::path::PathBuf;

    #[test]
    fn test_capture_state() {
        let mut engine = PrometeoTopologyEngine::new();
        engine.receive("io penso al tempo");
        engine.receive("la gioia del colore rosso");

        let state = PrometeoState::capture(&engine);
        assert_eq!(state.total_perturbations, 2);
        assert!(!state.lexicon.words.is_empty());
        assert!(!state.complex.simplices.is_empty());
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut engine = PrometeoTopologyEngine::new();
        engine.receive("ciao mondo");

        let state = PrometeoState::capture(&engine);
        let json = serde_json::to_string(&state).unwrap();
        let restored: PrometeoState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.total_perturbations, state.total_perturbations);
        assert_eq!(restored.version, "8D-v1.3");
    }

    #[test]
    fn test_save_and_load() {
        let mut engine = PrometeoTopologyEngine::new();
        engine.receive("io sento il tempo");

        let state = PrometeoState::capture(&engine);

        let tmp = std::env::temp_dir().join("prometeo_test_state.json");
        state.save_to_file(&tmp).unwrap();

        let loaded = PrometeoState::load_from_file(&tmp).unwrap();
        assert_eq!(loaded.total_perturbations, 1);

        // Cleanup
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_restore_lexicon() {
        let mut engine1 = PrometeoTopologyEngine::new();
        // Insegna una parola nuova
        for _ in 0..10 {
            engine1.receive("il quixplat brillante illumina");
        }

        let state = PrometeoState::capture(&engine1);

        // Crea un engine nuovo e ripristina
        let mut engine2 = PrometeoTopologyEngine::new();
        state.restore_lexicon(&mut engine2);

        assert!(engine2.lexicon.knows("quixplat"),
            "La parola appresa deve essere ripristinata");
    }

    #[test]
    fn test_restore_preserves_signatures_and_co_occurrences() {
        let mut engine1 = PrometeoTopologyEngine::new();
        // Insegna parole con contesto per creare co-occorrenze
        for _ in 0..8 {
            engine1.receive("quixplat brillante luminoso forte");
            engine1.receive("quixplat caldo vicino dentro");
        }

        // Cattura firma e co-occorrenze prima del salvataggio
        let pat1 = engine1.lexicon.get("quixplat").unwrap();
        let sig1 = *pat1.signature.values();
        let exp1 = pat1.exposure_count;
        let stab1 = pat1.stability;
        let cooc_brillante = *pat1.co_occurrences.get("brillante").unwrap_or(&0);
        assert!(cooc_brillante > 0, "Devono esserci co-occorrenze con 'brillante'");

        let state = PrometeoState::capture(&engine1);

        // Ripristina in un engine nuovo
        let mut engine2 = PrometeoTopologyEngine::new();
        state.restore_lexicon(&mut engine2);

        let pat2 = engine2.lexicon.get("quixplat").unwrap();
        // Firma identica
        assert_eq!(*pat2.signature.values(), sig1, "Firma deve essere identica dopo restore");
        // Esposizioni identiche
        assert_eq!(pat2.exposure_count, exp1, "Esposizioni devono essere identiche");
        // Stabilita identica
        assert!((pat2.stability - stab1).abs() < 0.001, "Stabilita deve essere identica");
        // Co-occorrenze ripristinate
        let cooc2 = *pat2.co_occurrences.get("brillante").unwrap_or(&0);
        assert_eq!(cooc2, cooc_brillante, "Co-occorrenze devono essere identiche dopo restore");
    }

    #[test]
    fn test_restore_complex_round_trip() {
        let mut engine1 = PrometeoTopologyEngine::new();
        // Insegna frasi per creare simplessi da compound inscription
        for _ in 0..15 {
            engine1.receive("io sento il tempo che passa nel confine");
            engine1.receive("la gioia del colore luminoso e caldo");
            engine1.receive("noi pensiamo alla forza del gesto");
        }

        // Conta simplessi prima del salvataggio
        let complex_count_1 = engine1.complex.count();
        assert!(complex_count_1 > 0, "Deve avere simplessi dopo le frasi");

        // Verifica che ci siano facce strutturate
        let mut total_faces_1 = 0usize;
        for (_, simplex) in engine1.complex.iter() {
            total_faces_1 += simplex.shared_faces.len();
        }

        // Cattura e salva
        let state = PrometeoState::capture(&engine1);

        // Verifica che le faces siano salvate nel formato strutturato
        for ss in &state.complex.simplices {
            assert!(!ss.faces.is_empty() || ss.vertices.is_empty(),
                "Ogni simplesso con vertici deve avere faces strutturate");
        }

        // Ripristina in un engine nuovo
        let mut engine2 = PrometeoTopologyEngine::new();
        state.restore_lexicon(&mut engine2);

        // Verifica: stesso numero di simplessi
        let complex_count_2 = engine2.complex.count();
        assert_eq!(complex_count_2, complex_count_1,
            "Il numero di simplessi deve essere identico dopo restore: era {} ora {}",
            complex_count_1, complex_count_2);

        // Verifica: le facce sono state ricostruite
        let mut total_faces_2 = 0usize;
        for (_, simplex) in engine2.complex.iter() {
            total_faces_2 += simplex.shared_faces.len();
        }
        assert_eq!(total_faces_2, total_faces_1,
            "Il numero totale di facce deve essere identico: era {} ora {}",
            total_faces_1, total_faces_2);
    }

    #[test]
    fn test_restore_complex_preserves_topology() {
        let mut engine1 = PrometeoTopologyEngine::new();
        for _ in 0..10 {
            engine1.receive("io penso al tempo");
        }

        // Misura prossimità topologica tra frattali 0 e 1 (SPAZIO e TEMPO)
        let prox_1 = engine1.complex.topological_proximity(0, 1);

        let state = PrometeoState::capture(&engine1);

        let mut engine2 = PrometeoTopologyEngine::new();
        state.restore_lexicon(&mut engine2);

        let prox_2 = engine2.complex.topological_proximity(0, 1);
        assert!((prox_2 - prox_1).abs() < 0.001,
            "Prossimità topologica deve essere identica: era {:.4} ora {:.4}",
            prox_1, prox_2);
    }

    #[test]
    fn test_restore_full_lifecycle_save_load_file() {
        let mut engine1 = PrometeoTopologyEngine::new();
        for _ in 0..10 {
            engine1.receive("la luce brilla nel cielo sereno");
            engine1.receive("io sento il battere del cuore");
        }

        let state = PrometeoState::capture(&engine1);
        let tmp = std::env::temp_dir().join("prometeo_test_lifecycle.json");
        state.save_to_file(&tmp).unwrap();

        // Carica da file e ripristina
        let loaded = PrometeoState::load_from_file(&tmp).unwrap();
        let mut engine2 = PrometeoTopologyEngine::new();
        loaded.restore_lexicon(&mut engine2);

        // Lessico, perturbazioni, complesso tutti preservati
        assert_eq!(engine2.total_perturbations, engine1.total_perturbations);
        assert_eq!(engine2.complex.count(), engine1.complex.count(),
            "Complex count deve sopravvivere al ciclo save/load");

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_binary_round_trip() {
        // Verifica che il formato SimplDB produca uno stato identico al JSON.
        let mut engine = PrometeoTopologyEngine::new();
        engine.receive("io sento il tempo");
        engine.receive("la luce brilla nel cielo");

        let state = PrometeoState::capture(&engine);

        let tmp = std::env::temp_dir().join("prometeo_test_binary.bin");
        state.save_to_binary(&tmp).unwrap();

        // Il file deve iniziare con i magic bytes corretti (v3)
        let bytes = std::fs::read(&tmp).unwrap();
        assert_eq!(&bytes[..8], b"SIMPDB03", "Magic bytes errati");

        // La dimensione binaria deve essere inferiore al JSON
        let json = serde_json::to_vec(&state).unwrap();
        assert!(bytes.len() < json.len(),
            "Binario ({}) deve essere più piccolo del JSON ({})", bytes.len(), json.len());

        // Round-trip: stato caricato deve essere identico
        let loaded = PrometeoState::load_from_binary(&tmp).unwrap();
        assert_eq!(loaded.total_perturbations, state.total_perturbations);
        assert_eq!(loaded.version, state.version);
        assert_eq!(loaded.lexicon.words.len(), state.lexicon.words.len());

        // Ogni parola deve avere la stessa firma (SimplDB ordina alfabeticamente, non per inserzione)
        let word_map: std::collections::HashMap<&str, &WordSnapshot> = loaded.lexicon.words.iter()
            .map(|ws| (ws.word.as_str(), ws))
            .collect();
        for orig in &state.lexicon.words {
            let restored = word_map.get(orig.word.as_str())
                .expect(&format!("Parola '{}' mancante nel round-trip", orig.word));
            assert_eq!(orig.signature, restored.signature,
                "Firma diversa per '{}'", orig.word);
            assert_eq!(orig.exposure_count, restored.exposure_count,
                "Esposizioni diverse per '{}'", orig.word);
        }

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_binary_magic_detection() {
        // Un file JSON deve essere rifiutato dal loader binario
        let mut engine = PrometeoTopologyEngine::new();
        engine.receive("test");
        let state = PrometeoState::capture(&engine);

        let tmp_json = std::env::temp_dir().join("prometeo_test_magic.json");
        state.save_to_file(&tmp_json).unwrap();

        let result = PrometeoState::load_from_binary(&tmp_json);
        assert!(result.is_err(), "Il loader binario deve rifiutare un file JSON");
        // L'errore può menzionare "SimplDB" o "Magic sconosciuto"
        let err = result.unwrap_err();
        assert!(err.contains("Magic") || err.contains("SimplDB"),
            "Errore deve descrivere il formato non riconosciuto: {}", err);

        let _ = std::fs::remove_file(&tmp_json);
    }

    #[test]
    fn test_binary_restore_full_lifecycle() {
        // Ciclo completo: save binary → load binary → restore engine
        let mut engine1 = PrometeoTopologyEngine::new();
        for _ in 0..5 {
            engine1.receive("io sento il tempo che passa");
            engine1.receive("la gioia del colore rosso e caldo");
        }

        let state = PrometeoState::capture(&engine1);
        let tmp = std::env::temp_dir().join("prometeo_test_binary_lifecycle.bin");
        state.save_to_binary(&tmp).unwrap();

        let loaded = PrometeoState::load_from_binary(&tmp).unwrap();
        let mut engine2 = PrometeoTopologyEngine::new();
        loaded.restore_lexicon(&mut engine2);

        assert_eq!(engine2.total_perturbations, engine1.total_perturbations,
            "Perturbazioni devono essere identiche dopo restore binario");
        assert_eq!(engine2.complex.count(), engine1.complex.count(),
            "Simplessi devono essere identici dopo restore binario");

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_resting_state_after_restore() {
        // Dopo restore, le parole stabili devono avere attivazione non-zero
        // (resting state = stability × 0.003, sotto soglia active_words 0.02).
        // Il resting è infrastruttura dormiente: presente ma non "attiva"
        // fino a quando l'input non la sveglia.
        let mut engine1 = PrometeoTopologyEngine::new();
        for _ in 0..15 {
            engine1.receive("io sento la gioia nel cuore");
            engine1.receive("la luce porta calore e vita");
        }

        let state = PrometeoState::capture(&engine1);
        let mut engine2 = PrometeoTopologyEngine::new();
        state.restore_lexicon(&mut engine2);

        // Il campo deve avere parole con attivazione > 0 (resting state)
        // anche se sotto soglia active_words — il resting è dormiente, non morto.
        let all_acts = engine2.word_topology.all_activations();
        let nonzero = all_acts.iter().filter(|(_, a)| *a > 0.0).count();
        assert!(nonzero > 0,
            "Dopo restore almeno alcune parole devono avere attivazione > 0 (resting state)");

        let max_activation = all_acts.iter().map(|(_, a)| *a).fold(0.0f64, f64::max);
        assert!(max_activation > 0.0,
            "L'attivazione massima a riposo deve essere > 0, trovato: {}", max_activation);
    }
}
