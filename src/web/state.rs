/// State — Tipi condivisi tra API, WebSocket e engine thread.

use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, oneshot, broadcast};

// ═══════════════════════════════════════════════════════════════
// AppState: condiviso tra tutti gli handler axum
// ═══════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct AppState {
    /// Canale per inviare comandi all'engine thread
    pub cmd_tx: mpsc::Sender<EngineCommand>,
    /// Canale broadcast per notificare i client WebSocket
    pub broadcast_tx: broadcast::Sender<String>,
    /// Store conversazioni — accessibile sia dagli handler che dall'engine loop
    pub conv_store: std::sync::Arc<std::sync::Mutex<super::conversations::ConversationStore>>,
}

// ═══════════════════════════════════════════════════════════════
// Comandi: main thread → engine thread
// ═══════════════════════════════════════════════════════════════

pub enum EngineCommand {
    /// Ricevi input testuale
    Receive {
        input: String,
        reply: oneshot::Sender<InputResponse>,
    },
    /// Stato volontà corrente
    GetWill {
        reply: oneshot::Sender<WillDto>,
    },
    /// Composti frattali attivi
    GetCompounds {
        reply: oneshot::Sender<Vec<CompoundDto>>,
    },
    /// Campo parole: top attive + energia
    GetWordField {
        reply: oneshot::Sender<WordFieldDto>,
    },
    /// Fase tra due parole
    GetPhase {
        word_a: String,
        word_b: String,
        reply: oneshot::Sender<PhaseDto>,
    },
    /// Parole di tensione tra due poli
    GetTension {
        pole_a: String,
        pole_b: String,
        reply: oneshot::Sender<Vec<TensionWordDto>>,
    },
    /// Snapshot stato corrente
    GetState {
        reply: oneshot::Sender<StateSnapshot>,
    },
    /// Grafo completo per visualizzazione
    GetTopology {
        reply: oneshot::Sender<TopologyDto>,
    },
    /// Navigazione geodetica tra due frattali
    Navigate {
        from: String,
        to: String,
        reply: oneshot::Sender<Option<NavigationDto>>,
    },
    /// Forza sogno
    Dream {
        ticks: u32,
        reply: oneshot::Sender<StateSnapshot>,
    },
    /// Crescita strutturale
    Grow {
        reply: oneshot::Sender<GrowthDto>,
    },
    /// Introspezione
    Introspect {
        reply: oneshot::Sender<IntrospectionDto>,
    },
    /// Perche ultimo output
    Why {
        reply: oneshot::Sender<WhyDto>,
    },
    /// Incertezze aperte — le domande reali che l'entità non ha risolto da sola
    Ask {
        reply: oneshot::Sender<Vec<QuestionDto>>,
    },
    /// L'utente illumina un'incertezza aperta dell'entità
    Clarity {
        topic: String,
        illumination: String,
        reply: oneshot::Sender<bool>,
    },
    /// Recupera le incertezze aperte dell'entità (per la UI)
    GetOpenQuestions {
        reply: oneshot::Sender<Vec<UncertaintyDto>>,
    },
    /// Recupera l'ultima catena di ragionamento autonomo
    GetLastThoughtChain {
        reply: oneshot::Sender<Option<ThoughtChainDto>>,
    },
    /// Proiezione olografica
    Projection {
        reply: oneshot::Sender<Option<ProjectionDto>>,
    },
    /// Genera testo
    Generate {
        reply: oneshot::Sender<GenerateDto>,
    },
    /// Salva stato su disco
    Save {
        reply: oneshot::Sender<bool>,
    },
    /// Simula generazione dal punto di vista di un altro locus
    SimulateLocus {
        locus_name: String,
        reply: oneshot::Sender<Option<LociSimDto>>,
    },
    /// Stato NarrativeSelf — ciclo deliberativo
    GetNarrative {
        reply: oneshot::Sender<NarrativeDto>,
    },
    /// Osservazioni topologiche interne (pensieri)
    GetThoughts {
        reply: oneshot::Sender<Vec<super::api::ThoughtDto>>,
    },
    /// Grammatica visiva: SVG dei 16 frattali + simplessi attivi composti
    GetVisuals {
        reply: oneshot::Sender<super::api::VisualsDto>,
    },
    /// Universo esplorabile: frattali + parole con posizione
    GetUniverse {
        reply: oneshot::Sender<UniverseDto>,
    },
    /// Vicini di una parola nella word topology
    GetWordNeighbors {
        word: String,
        reply: oneshot::Sender<WordNeighborsDto>,
    },
    /// Dettaglio completo di una parola: KG, affinità, statistiche
    GetWordDetail {
        word: String,
        reply: oneshot::Sender<WordDetailDto>,
    },
    /// Aggiunge connessione semantica curata dall'utente
    AddWordConnect {
        from: String,
        relation: String,
        to: String,
        via: Option<String>,
        confidence: Option<f32>,
        reply: oneshot::Sender<bool>,
    },
    /// Rimuove una relazione specifica (soggetto, tipo, oggetto)
    DeleteWordRelation {
        subject: String,
        relation: String,
        object: String,
        reply: oneshot::Sender<bool>,
    },
    /// Rimuove una parola e tutte le sue relazioni dal KG
    DeleteWord {
        word: String,
        reply: oneshot::Sender<bool>,
    },
    /// Lista parole del lessico (con paginazione e ricerca)
    GetWordList {
        query: String,
        offset: usize,
        limit: usize,
        sort: String,
        reply: oneshot::Sender<WordListDto>,
    },
    /// Aggiorna la firma 8D di una parola
    UpdateWordFirma {
        word: String,
        firma: [f64; 8],
        reply: oneshot::Sender<bool>,
    },
    /// Aggiorna confidence e/o via di una relazione esistente
    UpdateEdge {
        subject: String,
        relation: String,
        object: String,
        confidence: Option<f32>,
        via: Option<Option<String>>,
        reply: oneshot::Sender<bool>,
    },
    /// Lista parole-categoria (hub con molti figli IS_A o altra relazione)
    GetCategories {
        relation: String,
        min_children: usize,
        query: String,
        reply: oneshot::Sender<CategoriesDto>,
    },
    /// Pulizia lessico: rimuove forme verbali coniugate
    PuliziaVerbi {
        dry_run: bool,
        reply: oneshot::Sender<PuliziaDto>,
    },
    /// Tutto ciò che il sistema sa di un concetto via InferenceEngine
    GetConcept {
        word: String,
        reply: oneshot::Sender<ConceptDto>,
    },
    /// Stato completo del SelfModel (credenze, valori, incertezze)
    GetSelf {
        reply: oneshot::Sender<SelfDto>,
    },
    /// Episodi semantici recenti
    GetEpisodes {
        n: usize,
        reply: oneshot::Sender<Vec<EpisodeDto>>,
    },
    /// Episodi simili per concetti chiave
    RecallEpisodes {
        concepts: Vec<String>,
        reply: oneshot::Sender<Vec<EpisodeDto>>,
    },
    /// Sessione comunitaria: insegna testo e registra contributo
    CommunityTeach {
        text: String,
        user_id: String,
        user_name: String,
        user_context: String,
        reply: oneshot::Sender<CommunityTeachDto>,
    },
    /// Sessione comunitaria: valida/aggiusta confidenza di un arco KG
    CommunityValidateEdge {
        subject: String,
        relation: String,
        object: String,
        confidence: f32,
        user_id: String,
        user_name: String,
        user_context: String,
        reply: oneshot::Sender<bool>,
    },
    /// Stato sessione comunitaria corrente
    GetSessionState {
        reply: oneshot::Sender<SessionStateDto>,
    },
    /// Reset sessione comunitaria (nuova sessione)
    ResetSession {
        community_name: String,
        reply: oneshot::Sender<bool>,
    },
    /// Modula la volontà: focalizza su un topic specifico
    WillFocus {
        topic: String,
        reply: oneshot::Sender<WillDto>,
    },
    /// Report dettagliato del sogno (consolidamenti, perturbazioni)
    GetDreamReport {
        reply: oneshot::Sender<DreamReportDto>,
    },
    // ── Phase 52: Dialogo Interiore ──────────────────────────────────
    /// Aggrega pensieri, domande di curiosità e proposizioni per il dialogo interiore
    GetInnerDialogue {
        reply: oneshot::Sender<InnerDialogueDto>,
    },
    /// L'utente risponde a un item del dialogo interiore
    RespondToInsight {
        item_type: String,
        item_id: usize,
        response: String,
        action: String,
        reply: oneshot::Sender<RespondResult>,
    },
    // ── Conversazioni ──────────────────────────────────────────────────────
    /// Nuova conversazione utente: processa il messaggio e risponde
    ConvReceive {
        conv_id: String,
        message: String,
        reply: oneshot::Sender<String>,  // testo risposta entità
    },

    /// Cancella un arco KG specifico
    DeleteEdge {
        subject: String,
        relation: String,
        object: String,
        reply: oneshot::Sender<bool>,
    },
    /// Modifica la confidence di un arco KG
    PatchEdgeConfidence {
        subject: String,
        relation: String,
        object: String,
        confidence: f32,
        reply: oneshot::Sender<bool>,
    },
    // ── Biennale endpoints ────────────────────────────────────────
    /// Campo semantico 2D per visualizzazione galassia
    GetBiennaleField {
        reply: oneshot::Sender<BiennaleFieldDto>,
    },
    /// Dettaglio parola con vicini KG tipati
    GetBiennaleWord {
        word: String,
        reply: oneshot::Sender<BiennaleWordDto>,
    },
    /// Percorso BFS tra due parole nel KG
    GetBiennaleJourney {
        from: String,
        to: String,
        reply: oneshot::Sender<BiennaleJourneyDto>,
    },
}

// ═══════════════════════════════════════════════════════════════
// DTO: engine → JSON → frontend
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Clone, Debug)]
pub struct InputResponse {
    /// Testo generato dal campo
    pub generated_text: String,
    /// Parole chiave dall'emergenza
    pub keywords: Vec<String>,
    /// Stato aggiornato
    pub state: StateSnapshot,
    /// Postura interna (backward compat: Open, Curious, Reflective, Resonant, Withdrawn)
    pub stance: String,
    /// Phase 55: etichetta derivata dalla valenza Octalysis (più ricca di stance)
    pub valence_label: String,
    /// Intenzione di risposta (Acknowledge, Reflect, Resonate, Explore, Express, Remain)
    pub intention: String,
    /// Continuità tematica rispetto ai turni recenti [0,1]
    pub topic_continuity: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct StateSnapshot {
    /// Vitali
    pub vital: VitalDto,
    /// Frattali attivi (nome, attivazione)
    pub active_fractals: Vec<FractalActiveDto>,
    /// Posizione locus
    pub locus: Option<LocusDto>,
    /// Fase sogno
    pub dream_phase: String,
    /// Profondita sogno
    pub dream_depth: f64,
    /// Report sistema
    pub report: ReportDto,
    /// Firma campo corrente (8 valori)
    pub field_signature: Vec<f64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct VitalDto {
    pub activation: f64,
    pub saturation: f64,
    pub curiosity: f64,
    pub fatigue: f64,
    pub tension: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct FractalActiveDto {
    pub name: String,
    pub activation: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct LocusDto {
    pub fractal_name: String,
    pub fractal_id: u32,
    pub horizon: f64,
    pub trail: Vec<String>,
    pub sub_position: Vec<SubDimDto>,
    pub visible: Vec<VisibleFractalDto>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SubDimDto {
    pub dim_index: u8,
    pub value: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct VisibleFractalDto {
    pub name: String,
    pub visibility: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct ReportDto {
    pub fractal_count: usize,
    pub simplex_count: usize,
    pub max_dimension: usize,
    pub connected_components: usize,
    pub memory_stm: usize,
    pub memory_mtm: usize,
    pub memory_ltm: usize,
    pub dream_cycles: u64,
    pub total_perturbations: u64,
    pub vocabulary_size: usize,
    pub emergent_dimensions: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct TopologyDto {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
}

#[derive(Serialize, Clone, Debug)]
pub struct TopologyNode {
    pub id: u32,
    pub name: String,
    pub activation: f64,
    pub is_locus: bool,
    pub is_bootstrap: bool,
    pub simplex_count: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct TopologyEdge {
    pub source: u32,
    pub target: u32,
    pub strength: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct NavigationDto {
    pub from_name: String,
    pub to_name: String,
    pub steps: Vec<NavStepDto>,
    pub total_cost: f64,
    pub explanation: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct NavStepDto {
    pub fractal_name: String,
    pub shared_structures: Vec<String>,
    pub cumulative_cost: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct GrowthDto {
    pub events: Vec<String>,
    pub new_fractals: usize,
    pub new_connections: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct IntrospectionDto {
    pub fractal_count: usize,
    pub simplex_count: usize,
    pub conceptual_gaps: usize,
    pub disconnected_worlds: usize,
    pub densest_region: Option<String>,
    pub sparsest_region: Option<String>,
    pub field_energy: f64,
    pub emergent_dimensions: usize,
    pub most_experienced: Option<String>,
    pub least_experienced: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct WhyDto {
    pub explanation: String,
    pub fractal_sequence: Vec<FractalActiveDto>,
    pub propagation_bridges: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct QuestionDto {
    pub text: String,
    pub question_type: String,
    pub priority: f64,
}

/// Un'incertezza aperta dell'entità — domanda reale che non ha saputo rispondersi.
/// Mostrata nella UI perché l'utente possa scegliere di illuminarla.
#[derive(Serialize, Clone, Debug)]
pub struct UncertaintyDto {
    /// Il topic dell'incertezza (la domanda).
    pub topic: String,
    /// Urgenza [0, 1] — quanto questa domanda preme sull'entità.
    pub tension: f64,
    /// Quante volte è emersa senza trovare risposta.
    pub emergence_count: u32,
}

/// Un passo di ragionamento nella catena di pensiero autonomo.
#[derive(Serialize, Clone, Debug)]
pub struct ThoughtStepDto {
    pub from_concept: String,
    pub relation: String,
    pub to_concept: String,
    pub confidence: f32,
    pub insight: Option<String>,
}

/// Esito della catena di ragionamento.
#[derive(Serialize, Clone, Debug)]
pub struct ThoughtOutcomeDto {
    pub kind: String,           // "insight" | "new_question" | "dead_end"
    pub claim: Option<String>,  // per insight
    pub new_topic: Option<String>, // per new_question
}

/// Ultima catena di ragionamento autonomo dell'entità.
#[derive(Serialize, Clone, Debug)]
pub struct ThoughtChainDto {
    pub origin_description: String,
    pub steps: Vec<ThoughtStepDto>,
    pub outcome: ThoughtOutcomeDto,
    pub depth_reached: usize,
}

/// Risposta dell'endpoint /api/clarity.
#[derive(Serialize, Clone, Debug)]
pub struct ClarityResponseDto {
    pub acknowledged: bool,
    pub topic: String,
    pub message: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ProjectionDto {
    pub from_name: String,
    pub projections: Vec<ProjectionItemDto>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ProjectionItemDto {
    pub name: String,
    pub proximity: f64,
    pub dimensional_resonance: f64,
    pub distortion: f64,
    pub apparent_center: Vec<f64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GenerateDto {
    pub text: String,
    pub structure: String,
    pub cluster_count: usize,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct WillDto {
    pub intention: String,
    pub drive: f64,
    pub undercurrents: Vec<UndercurrentDto>,
    pub dream_phase: String,
    /// Codone 8D: [dim_a, dim_b] — top-2 dimensioni attive nel campo.
    pub codon: [usize; 2],
    /// Catena causale: perché questa intenzione (causa → contributo)
    pub trigger_chain: Vec<TriggerDto>,
    /// Prossima intenzione probabile se le condizioni cambiano
    pub forecast: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct UndercurrentDto {
    pub name: String,
    pub pressure: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct TriggerDto {
    pub cause: String,
    pub value: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct CompoundDto {
    pub name: String,
    pub fractals: Vec<String>,
    pub strength: f64,
    pub order: usize,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct WordFieldDto {
    pub top_words: Vec<WordActivationDto>,
    pub total_energy: f64,
    pub vertex_count: usize,
    pub edge_count: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct WordActivationDto {
    pub word: String,
    pub activation: f64,
    pub fractal: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct PhaseDto {
    pub word_a: String,
    pub word_b: String,
    pub phase_rad: f64,
    pub phase_deg: f64,
    pub label: String,        // "Risonanza", "Tensione", "Opposizione"
    pub cos_value: f64,
    pub co_affirmed: u64,
    pub co_negated: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct TensionWordDto {
    pub word: String,
    pub position: f64,
    pub distance_to_a: f64,
    pub distance_to_b: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct LociSimDto {
    /// Frattale simulato come locus
    pub locus_name: String,
    /// Frattali visibili da questa prospettiva
    pub visible_fractals: Vec<FractalActiveDto>,
    /// Frattali attivi nel word_topology
    pub active_fractals: Vec<FractalActiveDto>,
    /// Testo generato dalla prospettiva di questo locus
    pub generated_text: String,
}

// ─── Universo esplorabile ───────────────────────────────────────

#[derive(Serialize, Clone, Debug, Default)]
pub struct UniverseDto {
    pub fractals: Vec<UniverseFractal>,
    /// Top parole per stabilità (chiavi corte per JSON compatto)
    pub words: Vec<UniverseWord>,
}

#[derive(Serialize, Clone, Debug)]
pub struct UniverseFractal {
    pub id: u32,
    pub name: String,
    pub activation: f64,
    pub is_bootstrap: bool,
    /// Trigramma inferiore (id / 8)
    pub lower: u8,
    /// Trigramma superiore (id % 8)
    pub upper: u8,
    /// Numero di parole nel lessico con questo frattale come dominante
    pub word_count: usize,
}

/// Parola compressa per il payload universo.
/// Chiavi brevi per ridurre dimensione JSON (~40 byte/parola).
#[derive(Serialize, Clone, Debug)]
pub struct UniverseWord {
    /// Parola
    pub w: String,
    /// Frattale dominante (argmax affinità)
    pub f: u32,
    /// Stabilità 0-100
    pub s: u8,
    /// Attivazione corrente 0-100
    pub a: u8,
    /// Affinità dominante 0-100 (raggio orbita: alta = vicino al sole)
    pub a1: u8,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct WordNeighborsDto {
    pub word: String,
    pub fractal_id: u32,
    pub neighbors: Vec<WordNeighborDto>,
}

#[derive(Serialize, Clone, Debug)]
pub struct WordNeighborDto {
    pub word: String,
    pub weight: f64,
    pub fractal_id: u32,
}

#[derive(Deserialize)]
pub struct WordQuery {
    pub word: String,
}

// ─── Word Detail ────────────────────────────────────────────────

/// Dettaglio completo di una parola per il pannello UI.
#[derive(Serialize, Clone, Debug, Default)]
pub struct WordDetailDto {
    pub word: String,
    pub stability: f64,
    pub exposure: u64,
    pub fractal_id: u32,
    pub fractal_name: String,
    /// Firma 8D della parola [confine, valenza, intensità, definizione, complessità, permanenza, agency, tempo]
    pub firma_8d: [f64; 8],
    /// Profilo Octalysis: 8 core drive derivati dalla firma 8D
    pub octalysis: OctalysisDto,
    /// Top 5 frattali per affinità
    pub top_affinities: Vec<WordAffinityDto>,
    /// Archi KG uscenti (soggetto → oggetto)
    pub kg_out: Vec<KgEdgeDto>,
    /// Archi KG entranti (oggetto ← soggetto)
    pub kg_in: Vec<KgEdgeDto>,
    /// Co-occorrenze statistiche di qualità (peso > 0.25, non nel KG)
    pub statistical: Vec<WordNeighborDto>,
}

/// Profilo Octalysis — 8 core drive motivazionali derivati dalla firma 8D.
/// Ogni drive è [0.0, 1.0]. Calcolato come funzione delle dimensioni primitive.
#[derive(Serialize, Clone, Debug, Default)]
pub struct OctalysisDto {
    /// CD1: Significato Epico — Agency↑ × Permanenza↑
    pub significato_epico: f64,
    /// CD2: Realizzazione — Intensità↑ × Definizione↑
    pub realizzazione: f64,
    /// CD3: Creatività — Complessità↑ × Agency↑
    pub creativita: f64,
    /// CD4: Possesso — Confine↑ × Permanenza↑
    pub possesso: f64,
    /// CD5: Influenza Sociale — Valenza↑ × (1-Confine)↑
    pub influenza_sociale: f64,
    /// CD6: Scarsità — Tempo↑ × Intensità↑ × (1-Permanenza)
    pub scarsita: f64,
    /// CD7: Curiosità — Complessità↑ × (1-Definizione)
    pub curiosita: f64,
    /// CD8: Evitamento Perdita — (1-Valenza) × (1-Permanenza)
    pub evitamento_perdita: f64,
}

impl OctalysisDto {
    /// Calcola il profilo Octalysis dalla firma 8D.
    /// dims: [confine, valenza, intensità, definizione, complessità, permanenza, agency, tempo]
    pub fn from_firma(d: &[f64; 8]) -> Self {
        Self {
            significato_epico:  (d[6] * d[5]).sqrt().clamp(0.0, 1.0),          // agency × permanenza
            realizzazione:      (d[2] * d[3]).sqrt().clamp(0.0, 1.0),          // intensità × definizione
            creativita:         (d[4] * d[6]).sqrt().clamp(0.0, 1.0),          // complessità × agency
            possesso:           (d[0] * d[5]).sqrt().clamp(0.0, 1.0),          // confine × permanenza
            influenza_sociale:  (d[1] * (1.0 - d[0])).sqrt().clamp(0.0, 1.0), // valenza × (1-confine)
            scarsita:           (d[7] * d[2] * (1.0 - d[5])).cbrt().clamp(0.0, 1.0), // tempo × intensità × (1-permanenza)
            curiosita:          (d[4] * (1.0 - d[3])).sqrt().clamp(0.0, 1.0),  // complessità × (1-definizione)
            evitamento_perdita: ((1.0 - d[1]) * (1.0 - d[5])).sqrt().clamp(0.0, 1.0), // (1-valenza) × (1-permanenza)
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct WordAffinityDto {
    pub fractal_id: u32,
    pub fractal_name: String,
    pub value: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct KgEdgeDto {
    /// Chiave interna (IS_A, CAUSES, ENABLES, IMPLIES...)
    pub relation: String,
    /// Nome italiano per display ("è un", "causa", "abilita", "implica"...)
    pub nome: String,
    /// Colore CSS per l'arco nel grafo
    pub colore: String,
    pub target: String,
    pub confidence: f32,
    /// Tramite/mezzo della relazione (opzionale)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

#[derive(Deserialize)]
pub struct WordConnectBody {
    pub from: String,
    pub relation: String,
    pub to: String,
    pub via: Option<String>,
    pub confidence: Option<f32>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct WordListDto {
    pub words: Vec<WordListItemDto>,
    pub total: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct WordListItemDto {
    pub word: String,
    pub stability: f64,
    pub exposure: u64,
    pub fractal_name: String,
    pub out_degree: usize,
    pub in_degree: usize,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct CategoriesDto {
    pub categories: Vec<CategoryItemDto>,
    pub total: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct CategoryItemDto {
    pub word: String,
    pub children_count: usize,
    /// Primi 10 figli (per preview)
    pub sample_children: Vec<String>,
}

#[derive(Deserialize)]
pub struct UpdateEdgeBody {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub confidence: Option<f32>,
    pub via: Option<String>,
    pub clear_via: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateFirmaBody {
    pub word: String,
    pub firma: [f64; 8],
}

#[derive(Serialize)]
pub struct PuliziaDto {
    pub deleted: Vec<String>,
    pub count: usize,
    pub dry_run: bool,
}

// ═══════════════════════════════════════════════════════════════
// DTO: NarrativeSelf
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Clone, Debug)]
pub struct NarrativeTurnDto {
    pub turn_id: usize,
    pub act: String,
    pub stance: String,
    pub intention: String,
    pub intensity: f64,
    pub awareness: Option<String>,
    pub crystallized: bool,
    /// Intento riconosciuto in linguaggio naturale (es. "L'utente chiede chiarimento su X")
    pub recognized_intent: Option<String>,
    /// Posizione formata prima della generazione
    pub formed_position: Option<String>,
    /// Phase 54: stato interiore (bisogni, desideri, eco dell'Altro, umorismo)
    pub inner_state: Option<String>,
    /// Phase 55: profilo di valenza Octalysis (8 drive [-1, +1])
    pub valence: Option<ValenceDto>,
}

/// Phase 55: Profilo di valenza Octalysis per la UI.
#[derive(Serialize, Clone, Debug)]
pub struct ValenceDto {
    /// 8 drive con nome e valore [-1, +1]
    pub drives: Vec<ValenceDriveDto>,
    /// Etichetta derivata (es. "curioso", "ispirato", "vulnerabile")
    pub label: String,
    /// Tono edonico globale [-1, +1]
    pub hedonic_tone: f64,
    /// Intensità globale [0, 1]
    pub intensity: f64,
    /// Summary compatto
    pub summary: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ValenceDriveDto {
    pub name: String,
    pub value: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct NarrativePositionDto {
    pub act_key: String,
    pub stance: String,
    pub intention: String,
}

/// Phase 55: Impegno volitivo — l'intenzione a cui Prometeo si è legato.
#[derive(Serialize, Clone, Debug)]
pub struct CommitmentDto {
    /// Intenzione a cui è impegnato
    pub intention: String,
    /// Forza dell'impegno [0, 1]
    pub strength: f64,
    /// Turni consecutivi mantenuto
    pub turns_held: u32,
    /// Inerzia calcolata (forza × ln(turni+1))
    pub inertia: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct NarrativeDto {
    pub stance: String,
    /// Phase 55: etichetta derivata dalla valenza (più ricca di stance)
    pub valence_label: String,
    pub pending_intention: Option<String>,
    pub topic_continuity: f64,
    pub is_born: bool,
    pub turn_count: usize,
    /// Phase 55: profilo di valenza corrente
    pub valence: Option<ValenceDto>,
    /// Phase 55: impegno volitivo corrente (se presente)
    pub commitment: Option<CommitmentDto>,
    /// Phase 55: integrità coerenza identitaria [0, 1] — scende su contraddizioni interne
    pub coherence_integrity: f64,
    /// Phase 55: intenzione attribuita all'Altro (Unknown/Seeking/Teaching/Challenging/Connecting/Withdrawing)
    pub attributed_intent: String,
    /// Ultimi turni recenti (max 8, non cristallizzati)
    pub recent_turns: Vec<NarrativeTurnDto>,
    /// Turni cristallizzati — salienti, persistono tra sessioni
    pub crystallized: Vec<NarrativeTurnDto>,
    /// Posizioni deliberate formate da pattern ripetuti
    pub positions: Vec<NarrativePositionDto>,
}

// ═══════════════════════════════════════════════════════════════
// DTO: Concept — tutto ciò che InferenceEngine sa di una parola
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Clone, Debug, Default)]
pub struct ConceptDto {
    pub word: String,
    /// Definizione in linguaggio naturale (es. "cane è un mammifero, ha pelo, abbaia")
    pub definition: Option<String>,
    /// Catena IS_A transitiva: tipi di questa parola
    pub type_chain: Vec<String>,
    /// Istanze dirette: parole che sono IS_A questa (inversione)
    pub instances: Vec<String>,
    /// Proprietà (HAS dirette + ereditate)
    pub has: Vec<String>,
    /// Azioni (DOES dirette + ereditate)
    pub does: Vec<String>,
    /// Effetti causali
    pub causes: Vec<String>,
    /// Concetti simili/sinonimi
    pub similar: Vec<String>,
    /// Opposti (tensione concettuale)
    pub opposites: Vec<String>,
    /// Di cosa è parte questa parola
    pub part_of: Vec<String>,
    /// Quante parole nel lessico condividono un IS_A con questa (densità ontologica)
    pub ontology_density: usize,
}

// ═══════════════════════════════════════════════════════════════
// DTO: SelfModel — identità esplicita
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Clone, Debug, Default)]
pub struct SelfDto {
    pub beliefs: Vec<BeliefDto>,
    pub values: Vec<ValueDto>,
    pub uncertainties: Vec<UncertaintyDto>,
    pub interaction_count: u64,
    /// Credenze attive nell'ultima interazione (anchor overlap con input)
    pub active_beliefs: Vec<ActiveBeliefDto>,
    /// Traccia influenza: come le credenze hanno modulato il campo
    pub belief_influence_trace: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ActiveBeliefDto {
    pub claim: String,
    pub confidence: f64,
    pub activated_words: Vec<String>,
    pub influence_strength: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct BeliefDto {
    pub claim: String,
    pub anchor_concepts: Vec<String>,
    pub confidence: f64,
    pub reinforcement_count: u32,
    pub innate: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct ValueDto {
    pub name: String,
    pub weight: f64,
    pub associated_words: Vec<String>,
    pub innate: bool,
    pub activation_count: u64,
}

// ═══════════════════════════════════════════════════════════════
// DTO: SemanticEpisode — memoria episodica navigabile
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Clone, Debug)]
pub struct EpisodeDto {
    pub id: u64,
    pub timestamp: u64,
    pub key_concepts: Vec<String>,
    pub dominant_fractals: Vec<EpisodeFractalDto>,
    pub summary: String,
    pub stance: String,
    pub intention: String,
    pub active_values: Vec<String>,
    pub field_energy: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct EpisodeFractalDto {
    pub id: u32,
    pub name: String,
    pub activation: f64,
}

// ═══════════════════════════════════════════════════════════════
// DTO: Sessione Comunitaria
// ═══════════════════════════════════════════════════════════════

/// Risultato di un insegnamento comunitario
#[derive(Serialize, Clone, Debug)]
pub struct CommunityTeachDto {
    /// Parole nuove aggiunte al lessico
    pub words_new: Vec<String>,
    /// Parole già conosciute
    pub words_known: Vec<String>,
    /// Frattali toccati dall'insegnamento (nome, delta attivazione)
    pub fractals_touched: Vec<(String, f64)>,
    /// Connessioni emergenti trovate (soggetto, relazione, oggetto, confidenza)
    pub connections_found: Vec<(String, String, String, f32)>,
    /// Energia del campo prima e dopo
    pub field_energy_delta: f64,
    /// Parole che il campo ha attivato in risposta (risonanza semantica)
    pub resonating_words: Vec<String>,
}

/// Singolo contributo testuale di un partecipante
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TeachEntry {
    pub user_id: String,
    pub user_name: String,
    pub user_context: String,
    pub text: String,
    pub words_new: Vec<String>,
    pub timestamp: u64,
}

/// Singola connessione KG creata da un partecipante
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommunityEdge {
    pub user_id: String,
    pub user_name: String,
    pub user_context: String,
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub confidence: f32,
    pub timestamp: u64,
}

/// Stato completo della sessione comunitaria corrente
#[derive(Serialize, Clone, Debug, Default)]
pub struct SessionStateDto {
    pub community_name: String,
    pub teach_entries: Vec<TeachEntry>,
    pub community_edges: Vec<CommunityEdge>,
    pub founding_narrative: String,
    pub total_words_taught: usize,
    pub total_connections: usize,
    pub active_participants: Vec<String>,
}

/// Richiesta di connessione KG dalla UI community
#[derive(Deserialize, Clone, Debug)]
pub struct CommunityConnectRequest {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub strength: Option<u8>,   // 1-5, convertito in confidenza 0.2-1.0
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub user_context: Option<String>,
}

/// Richiesta di insegnamento dalla UI community
#[derive(Deserialize, Clone, Debug)]
pub struct CommunityTeachRequest {
    pub text: String,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub user_context: Option<String>,
}

/// Richiesta di validazione connessione dalla UI community
#[derive(Deserialize, Clone, Debug)]
pub struct CommunityValidateRequest {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub resonance: u8,          // 1-5, convertito in confidenza 0.2-1.0
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub user_context: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Phase 52: Dialogo Interiore — DTOs
// ═══════════════════════════════════════════════════════════════

/// Aggregato del dialogo interiore: dubbi, domande, proposizioni
#[derive(Serialize, Clone, Debug)]
pub struct InnerDialogueDto {
    pub thoughts: Vec<InnerDialogueItem>,
    pub questions: Vec<InnerDialogueItem>,
    pub propositions: Vec<InnerDialogueItem>,
}

/// Singolo item del dialogo interiore
#[derive(Serialize, Clone, Debug)]
pub struct InnerDialogueItem {
    pub id: usize,
    pub text: String,
    pub category: String,
    pub strength: f64,
    pub detail: serde_json::Value,
}

/// Risultato di una risposta utente a un item
#[derive(Serialize, Clone, Debug)]
pub struct RespondResult {
    pub success: bool,
    pub effect: String,
}

// ═══════════════════════════════════════════════════════════════
// DTO: Dream Report — dettagli consolidamento e perturbazioni
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Clone, Debug, Default)]
pub struct DreamReportDto {
    pub phase: String,
    pub depth: f64,
    pub cycles_completed: u64,
    pub total_perturbations: u64,
    /// Consolidamenti recenti: cosa è passato tra layer di memoria
    pub consolidations: Vec<ConsolidationDto>,
    /// Report post-sogno (dopo ciclo completo)
    pub post_dream_summary: Option<String>,
    /// Conteggi memoria correnti
    pub memory_stm: usize,
    pub memory_mtm: usize,
    pub memory_ltm: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct ConsolidationDto {
    pub description: String,
    pub from_layer: String,
    pub to_layer: String,
    pub strength: f64,
}

/// Richiesta di risposta a un item del dialogo interiore
#[derive(Deserialize, Clone, Debug)]
pub struct RespondRequest {
    pub item_type: String,
    pub item_id: usize,
    pub response: String,
    pub action: String,
}

// ═══════════════════════════════════════════════════════════════
// Biennale DTOs
// ═══════════════════════════════════════════════════════════════

/// Posizione 2D di una parola nel campo semantico (Biennale).
#[derive(Serialize, Clone, Debug)]
pub struct BiennaleWordPos {
    /// Parola
    pub w: String,
    /// x = valenza (dim[1]) jittered con confine (dim[0])
    pub x: f32,
    /// y = agency (dim[6]) jittered con intensità (dim[2])
    pub y: f32,
    /// Frattale dominante
    pub f: u32,
    /// Stabilità 0-100
    pub s: u8,
}

/// Campo semantico completo per visualizzazione galassia Biennale.
#[derive(Serialize, Clone, Debug, Default)]
pub struct BiennaleFieldDto {
    pub words: Vec<BiennaleWordPos>,
    pub fractal_names: Vec<(u32, String)>,
    /// ["negativo", "positivo", "passivo", "attivo"]
    pub axis_labels: [String; 4],
}

/// Vicino KG con tipo relazione e posizione.
#[derive(Serialize, Clone, Debug)]
pub struct BiennaleNeighborDto {
    pub w: String,
    pub rel: String,
    pub conf: f32,
    pub x: f32,
    pub y: f32,
}

/// Dettaglio parola per Biennale: firma 8D + vicini KG tipati.
#[derive(Serialize, Clone, Debug, Default)]
pub struct BiennaleWordDto {
    pub word: String,
    pub firma: [f64; 8],
    pub fractal_name: String,
    pub stability: f64,
    pub x: f32,
    pub y: f32,
    pub neighbors: Vec<BiennaleNeighborDto>,
}

/// Un passo nel percorso semantico (Biennale Journey).
#[derive(Serialize, Clone, Debug)]
pub struct BiennalePathStepDto {
    pub word: String,
    /// Relazione verso il passo successivo (None per l'ultimo)
    pub relation: Option<String>,
    pub x: f32,
    pub y: f32,
}

/// Percorso BFS tra due parole nel KG.
#[derive(Serialize, Clone, Debug, Default)]
pub struct BiennaleJourneyDto {
    pub found: bool,
    pub from: String,
    pub to: String,
    pub path: Vec<BiennalePathStepDto>,
}

/// Query params per gli endpoint biennale word/journey.
#[derive(serde::Deserialize)]
pub struct BiennaleWordQuery {
    pub word: String,
}

#[derive(serde::Deserialize)]
pub struct BiennaleJourneyQuery {
    pub from: String,
    pub to: String,
}
