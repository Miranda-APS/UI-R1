//! Prometeo v8.0 — Campo Topologico Computazionale 8D
//!
//! Architettura frattale 8D + Complessi Simpliciali.
//! La macchina per ciò che è.
//!
//! ## Struttura
//! - `topology/` — L'intero sistema: primitivi 8D, frattali, simplessi,
//!   contesto, memoria, sogno, lessico, composizione, generazione, persistenza, engine.
//!
//! ## Le 8 Dimensioni Primitive (generative, come RGB)
//! Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo

pub mod topology;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "android")]
pub mod android;

// Re-export principali per convenienza
pub use topology::{
    PrimitiveCore, Dim,
    Fractal, FractalId, FractalRegistry, bootstrap_fractals,
    Simplex, SimplexId, SimplicialComplex, bootstrap_complex,
    Context, Perturbation, EmergentResponse,
    TopologicalMemory,
    DreamEngine, SleepPhase, DreamResult,
    Lexicon, WordPattern,
    PrometeoState,
    compute_homology, HomologyResult,
    VitalCore, VitalState, TensionState,
    CuriosityEngine, CuriosityQuestion, QuestionType,
    generate_from_field, generate_with_will, GeneratedText, SentenceStructure,
    CovariationTracker, Covariation, DimensionalEvent,
    WillCore, WillResult, Intention, WithdrawReason, DialogueContext,
    WordTopology, WordVertex, WordEdge, WordId,
    PrometeoTopologyEngine, SystemReport, TeachResult, CompoundState,
    SemanticBridge, LatentAffinity, BridgeReinforcement, AutonomousResult,
    PerceptualField, TensionWord,
    TranslatedExpression, translate_state, translate_or_raw, IdentityContext,
};
