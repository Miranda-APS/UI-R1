/// Topologia — L'architettura frattale 8D di Prometeo.
///
/// Moduli:
/// - `primitive`: Le 8 dimensioni primitive (l'RGB della semantica)
/// - `fractal`: Frattali, sotto-frattali, generazione dimensionale
/// - `simplex`: Complessi simpliciali, connessioni topologiche
/// - `context`: Attivazione contestuale e perturbazione
/// - `memory`: Memoria topologica stratificata (STM/MTM/LTM)
/// - `dream`: Sogno come digestione topologica
/// - `lexicon`: Lessico apprendibile (parole come pattern topologici)
/// - `composition`: Composizione frasale (frase come operazione topologica)
/// - `homology`: Calcolo omologico (numeri di Betti, cicli, lacune)
/// - `generation`: Generazione testo dalla configurazione topologica
/// - `dimensional`: Generazione dimensionale (co-variazioni → dimensioni emergenti)
/// - `metacognition`: Il sistema osserva la propria topologia
/// - `navigation`: Navigazione geodetica (cammini minimi, analogie)
/// - `dialogue`: Dialogo multi-turno (contesto conversazionale, anafore)
/// - `reasoning`: Ragionamento come navigazione topologica
/// - `growth`: Crescita strutturale (nuovi frattali e connessioni)
/// - `creativity`: Creativita come sogno guidato (REM intenzionale, metafore)
/// - `locus`: Posizione del sistema nel suo universo concettuale
/// - `engine`: Orchestratore leggero del sistema

pub mod primitive;
pub mod fractal;
pub mod simplex;
pub mod context;
pub mod memory;
pub mod dream;
pub mod lexicon;
pub mod composition;
pub mod persistence;
pub mod homology;
pub mod vital;
pub mod curiosity;
pub mod generation;
pub mod dimensional;
pub mod metacognition;
pub mod navigation;
pub mod dialogue;
pub mod reasoning;
pub mod growth;
pub mod creativity;
pub mod locus;
pub mod will;
pub mod word_topology;
pub mod pf1;
pub mod grammar;
pub mod syntax_center;
pub mod simpdb;
pub mod state_translation;
pub mod knowledge;
pub mod engine;
pub mod thought;
pub mod episodic;
pub mod polar_twin;
pub mod synthesis;

pub mod visual_perception;
pub mod fractal_visuals;
pub mod environment;
pub mod opinion;
pub mod identity;
pub mod provenance;
pub mod relation;
pub mod knowledge_graph;
pub mod inference;
pub mod input_reading;
pub mod narrative;
pub mod self_model;
pub mod semantic_episode;
pub mod inquiry;
pub mod thought_chain;
pub mod proposition;
pub mod needs;
pub mod desire;
pub mod interlocutor;
pub mod humor;
pub mod valence;
pub mod expression;

// LLM Substrate (feature-gated)
#[cfg(feature = "llm-substrate")]
pub mod llm_substrate;

#[cfg(feature = "llm-substrate")]
pub mod llm_substrate_qwen35;

pub use primitive::{PrimitiveCore, Dim};
pub use fractal::{
    Fractal, FractalId, FractalRegistry, DimConstraint, EmergentDimension,
    bootstrap_fractals,
};
pub use simplex::{
    Simplex, SimplexId, SimplicialComplex, SharedFace, SharedStructureType,
    bootstrap_complex,
};
pub use context::{
    Context, ActivationResult, Perturbation, EmergentResponse,
    activate_context, create_perturbation, apply_perturbation, emerge_response,
};
pub use memory::{TopologicalMemory, FieldImprint, Resonance, MemoryStats};
pub use dream::{DreamEngine, SleepPhase, DreamResult};
pub use lexicon::{Lexicon, WordPattern, WordActivation, SemanticAxis, SemanticAxisSnapshot, TensionWord};
pub use composition::{PhrasePattern, compose_phrase, inscribe_phrase, analyze_composition};
pub use persistence::{PrometeoState, CurriculumProgress, LessonRecord};
pub use homology::{compute_homology, HomologyResult, Cycle};
pub use vital::{VitalCore, VitalState, TensionState};
pub use curiosity::{CuriosityEngine, CuriosityQuestion, QuestionType};
pub use generation::{generate_from_field, generate_from_field_with_locus, generate_with_will, GeneratedText, SentenceStructure};
pub use dimensional::{CovariationTracker, Covariation, DimensionalEvent};
pub use metacognition::{introspect, trace_response, compute_delta, Introspection, ResponseTrace, FieldDelta};
pub use navigation::{find_geodesic, geodesic_distance, distance_map, find_analogy, GeodesicPath, GeodesicStep, TopologicalAnalogy};
pub use dialogue::{ConversationContext, ConversationTurn, DialogueState, dialogue_state};
pub use reasoning::{evaluate_implication, abduce, find_contradictions, reason, Implication, ImplicationType, Abduction, Contradiction, ReasoningResult};
pub use growth::{GrowthTracker, GrowthEvent};
pub use creativity::{create, find_metaphors, assess_confidence, CreativeSession, CreativeInsight, Metaphor, FieldConfidence};
pub use locus::{Locus, Movement, MovementKind, SubLocusView, HolographicProjection, FractalProjection, project_universe, project_from_locus};
pub use will::{WillCore, WillResult, Intention, WithdrawReason, DialogueContext};
pub use word_topology::{WordTopology, WordVertex, WordEdge, WordId};
pub use grammar::{PartOfSpeech, Person, Tense, LemmaResult, conjugate, lemmatize, detect_pos_from_word};
pub use state_translation::{TranslatedExpression, translate_state, translate_or_raw, IdentityContext};
pub use knowledge::{KnowledgeBase, KnowledgeEntry, KnowledgeDomain, KnowledgeSnapshot};
pub use engine::{PrometeoTopologyEngine, SystemReport, TeachResult, CompoundState, SemanticBridge, LatentAffinity, BridgeReinforcement, AutonomousResult, PerceptualField};
pub use episodic::{EpisodeStore, Episode, EpisodeSnapshot, PHI_INV, RECALL_BLEND, RECALL_THRESHOLD};
pub use identity::{IdentityCore, IdentitySnapshot};
pub use self_model::{SelfModel, SelfBelief, SelfValue, SelfUncertainty, SelfModelSnapshot};
pub use semantic_episode::{SemanticEpisodeLog, SemanticEpisode};
pub use provenance::{ActivationSource, ProvenanceMap};
pub use relation::{RelationType, TypedEdge, EdgeSource};
pub use knowledge_graph::{KnowledgeGraph, KgSnapshot};
pub use inference::InferenceEngine;
pub use proposition::{Proposition, PropRelation, extract_propositions};
pub use needs::{NeedsHierarchy, NeedsState, NeedsPressure, NeedLevel, FieldMetrics};
pub use desire::{DesireCore, Desire, DesireSource, DesireSnapshot, SatisfactionEvent};
pub use interlocutor::{InterlocutorModel, InteractionTrace, InteractionPattern, InterlocutorSnapshot};
pub use humor::{HumorSense, HumorState};
pub use valence::{Valence, ValenceInput, DRIVE_NAMES, DRIVE_DIM};
pub use visual_perception::{VisualConcept, PerceptualResponse, parse_svg_simple};
pub use fractal_visuals::{fractal_svg, fractal_svg_from_registry, fractal_name, all_fractal_svgs, all_fractal_svgs_from_registry, compose_simplex_svg, FRACTAL_NAMES, FRACTAL_COUNT};
