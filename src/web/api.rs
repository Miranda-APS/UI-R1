/// API REST — Handler per tutti gli endpoint.
/// UI refresh: theme toggle + font sizes

use axum::{
    extract::State,
    extract::Path,
    extract::Query,
    response::{Html, IntoResponse, Response},
    body::Body,
    Json,
};
use axum::http::{StatusCode, header};
use serde::Deserialize;
use tokio::sync::oneshot;

use super::state::*;

// ═══════════════════════════════════════════════════════════════
// GET / — Serve la dashboard HTML
// ═══════════════════════════════════════════════════════════════

static INDEX_HTML: &str = include_str!("index.html");
static COMMUNITY_HTML: &str = include_str!("community/index.html");
static UNIVERSO_HTML: &str = include_str!("universo/index.html");
static BIENNALE_HTML: &str = include_str!("biennale/index.html");
static BIENNALE_HOME_HTML: &str = include_str!("biennale/home.html");
static DIALOGO_HTML: &str = include_str!("biennale/dialogo.html");
static CURAZIONE_HTML: &str = include_str!("biennale/curazione.html");

pub async fn index() -> Html<&'static str> {
    Html(BIENNALE_HOME_HTML)
}

pub async fn admin_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn universo_index() -> Html<&'static str> {
    Html(UNIVERSO_HTML)
}

pub async fn biennale_index() -> Html<&'static str> {
    Html(BIENNALE_HTML)
}

pub async fn dialogo_index() -> Html<&'static str> {
    Html(DIALOGO_HTML)
}

pub async fn curazione_index() -> Html<&'static str> {
    Html(CURAZIONE_HTML)
}

// ═══════════════════════════════════════════════════════════════
// GET /api/state — Snapshot completo
// ═══════════════════════════════════════════════════════════════

pub async fn get_state(State(state): State<AppState>) -> Json<StateSnapshot> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetState { reply: tx }).await;
    match rx.await {
        Ok(snapshot) => Json(snapshot),
        Err(_) => Json(StateSnapshot::default()),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/input — Invia testo all'engine
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct InputRequest {
    pub text: String,
}

pub async fn post_input(
    State(state): State<AppState>,
    Json(req): Json<InputRequest>,
) -> Json<InputResponse> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Receive {
        input: req.text,
        reply: tx,
    }).await;
    match rx.await {
        Ok(response) => {
            // Broadcast stato aggiornato ai WebSocket
            let update = serde_json::json!({
                "type": "state_update",
                "data": &response.state,
            });
            let _ = state.broadcast_tx.send(update.to_string());
            Json(response)
        }
        Err(_) => Json(InputResponse {
            generated_text: String::new(),
            keywords: Vec::new(),
            state: StateSnapshot::default(),
            stance: "Open".to_string(),
            valence_label: "aperto".to_string(),
            intention: "Express".to_string(),
            topic_continuity: 0.5,
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/dream — Forza sogno
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct DreamRequest {
    pub ticks: Option<u32>,
}

pub async fn post_dream(
    State(state): State<AppState>,
    Json(req): Json<DreamRequest>,
) -> Json<StateSnapshot> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Dream {
        ticks: req.ticks.unwrap_or(20),
        reply: tx,
    }).await;
    match rx.await {
        Ok(snapshot) => {
            let update = serde_json::json!({
                "type": "state_update",
                "data": &snapshot,
            });
            let _ = state.broadcast_tx.send(update.to_string());
            Json(snapshot)
        }
        Err(_) => Json(StateSnapshot::default()),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/grow — Crescita strutturale
// ═══════════════════════════════════════════════════════════════

pub async fn post_grow(State(state): State<AppState>) -> Json<GrowthDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Grow { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(GrowthDto {
            events: Vec::new(),
            new_fractals: 0,
            new_connections: 0,
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/topology — Grafo completo
// ═══════════════════════════════════════════════════════════════

pub async fn get_topology(State(state): State<AppState>) -> Json<TopologyDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetTopology { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(TopologyDto {
            nodes: Vec::new(),
            edges: Vec::new(),
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/navigate/:from/:to — Geodetica
// ═══════════════════════════════════════════════════════════════

pub async fn get_navigate(
    State(state): State<AppState>,
    Path((from, to)): Path<(String, String)>,
) -> Json<Option<NavigationDto>> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Navigate {
        from,
        to,
        reply: tx,
    }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(None),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/projection — Proiezione olografica
// ═══════════════════════════════════════════════════════════════

pub async fn get_projection(State(state): State<AppState>) -> Json<Option<ProjectionDto>> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Projection { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(None),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/introspect — Introspezione
// ═══════════════════════════════════════════════════════════════

pub async fn get_introspect(State(state): State<AppState>) -> Json<IntrospectionDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Introspect { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(IntrospectionDto {
            fractal_count: 0,
            simplex_count: 0,
            conceptual_gaps: 0,
            disconnected_worlds: 0,
            densest_region: None,
            sparsest_region: None,
            field_energy: 0.0,
            emergent_dimensions: 0,
            most_experienced: None,
            least_experienced: None,
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/why — Spiegazione ultimo output
// ═══════════════════════════════════════════════════════════════

pub async fn get_why(State(state): State<AppState>) -> Json<WhyDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Why { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(WhyDto {
            explanation: String::new(),
            fractal_sequence: Vec::new(),
            propagation_bridges: Vec::new(),
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/ask — Incertezze aperte (domande reali dell'entità)
// ═══════════════════════════════════════════════════════════════

pub async fn get_ask(State(state): State<AppState>) -> Json<Vec<QuestionDto>> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Ask { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(Vec::new()),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/open-questions — Incertezze aperte (versione diretta)
// ═══════════════════════════════════════════════════════════════

pub async fn get_open_questions(State(state): State<AppState>) -> Json<Vec<UncertaintyDto>> {
    let (tx, rx) = oneshot::channel::<Vec<UncertaintyDto>>();
    let _ = state.cmd_tx.send(EngineCommand::GetOpenQuestions { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(Vec::new()),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/thought-chain — Ultima catena di ragionamento autonomo
// ═══════════════════════════════════════════════════════════════

pub async fn get_thought_chain(State(state): State<AppState>) -> Json<Option<ThoughtChainDto>> {
    let (tx, rx) = oneshot::channel::<Option<ThoughtChainDto>>();
    let _ = state.cmd_tx.send(EngineCommand::GetLastThoughtChain { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(None),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/clarity — L'utente illumina un'incertezza dell'entità
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct ClarityRequest {
    pub topic: String,
    pub illumination: String,
}

pub async fn post_clarity(
    State(state): State<AppState>,
    Json(req): Json<ClarityRequest>,
) -> Json<ClarityResponseDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Clarity {
        topic: req.topic.clone(),
        illumination: req.illumination.clone(),
        reply: tx,
    }).await;
    match rx.await {
        Ok(true) => Json(ClarityResponseDto {
            acknowledged: true,
            topic: req.topic.clone(),
            message: format!("Comprensione ricevuta su '{}'. Il campo si aggiorna.", req.topic),
        }),
        _ => Json(ClarityResponseDto {
            acknowledged: false,
            topic: req.topic,
            message: "Impossibile elaborare la comprensione.".to_string(),
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/generate — Genera testo dal campo
// ═══════════════════════════════════════════════════════════════

pub async fn get_generate(State(state): State<AppState>) -> Json<GenerateDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Generate { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(GenerateDto {
            text: String::new(),
            structure: String::new(),
            cluster_count: 0,
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/save — Salva stato
// ═══════════════════════════════════════════════════════════════

pub async fn post_save(State(state): State<AppState>) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Save { reply: tx }).await;
    match rx.await {
        Ok(ok) => Json(ok),
        Err(_) => Json(false),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/will — Stato volontà corrente
// ═══════════════════════════════════════════════════════════════

pub async fn get_will(State(state): State<AppState>) -> Json<WillDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetWill { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(WillDto::default()),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/compounds — Composti frattali attivi
// ═══════════════════════════════════════════════════════════════

pub async fn get_compounds(State(state): State<AppState>) -> Json<Vec<CompoundDto>> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetCompounds { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(Vec::new()),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/wordfield — Campo parole top attive
// ═══════════════════════════════════════════════════════════════

pub async fn get_wordfield(State(state): State<AppState>) -> Json<WordFieldDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetWordField { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(WordFieldDto::default()),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/phase/:a/:b — Fase tra due parole
// ═══════════════════════════════════════════════════════════════

pub async fn get_phase(
    State(state): State<AppState>,
    Path((a, b)): Path<(String, String)>,
) -> Json<PhaseDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetPhase { word_a: a, word_b: b, reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(PhaseDto::default()),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/tension/:a/:b — Parole di tensione tra due poli
// ═══════════════════════════════════════════════════════════════

pub async fn get_tension(
    State(state): State<AppState>,
    Path((a, b)): Path<(String, String)>,
) -> Json<Vec<TensionWordDto>> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetTension { pole_a: a, pole_b: b, reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(Vec::new()),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/locus-simulate — Simula dal punto di vista di un locus
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct LocusSimRequest {
    pub locus: String,
}

pub async fn post_locus_simulate(
    State(state): State<AppState>,
    Json(req): Json<LocusSimRequest>,
) -> Json<Option<LociSimDto>> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::SimulateLocus {
        locus_name: req.locus,
        reply: tx,
    }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(None),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/simpdb — Serve il file SimplDB binario per download mobile
// ═══════════════════════════════════════════════════════════════

pub async fn get_simpdb() -> Response {
    // Cerca prima il formato v3 (.bin), poi il legacy JSON
    let paths = [
        "prometeo_topology_state.bin",
        "prometeo_state.bin",
    ];

    for path in &paths {
        match tokio::fs::read(path).await {
            Ok(bytes) => {
                return (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, "application/octet-stream"),
                        (header::CONTENT_DISPOSITION, "attachment; filename=\"prometeo_state.bin\""),
                    ],
                    bytes,
                ).into_response();
            }
            Err(_) => continue,
        }
    }

    (StatusCode::NOT_FOUND, "SimplDB non disponibile — usa :save prima").into_response()
}

// ═══════════════════════════════════════════════════════════════
// /api/thoughts — osservazioni topologiche interne
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Serialize)]
pub struct ThoughtDto {
    pub kind: String,
    pub fractal_names: Vec<String>,
    pub words: Vec<String>,
    pub strength: f64,
    pub detail: serde_json::Value,
}

pub async fn get_thoughts(State(state): State<AppState>) -> Json<Vec<ThoughtDto>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetThoughts { reply: tx }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/narrative — stato NarrativeSelf
// ═══════════════════════════════════════════════════════════════

pub async fn get_narrative(State(state): State<AppState>) -> Json<super::state::NarrativeDto> {
    use super::state::{NarrativeDto, NarrativeTurnDto, NarrativePositionDto};
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetNarrative { reply: tx }).await;
    Json(rx.await.unwrap_or(NarrativeDto {
        stance: "aperto".into(),
        valence_label: "aperto".into(),
        pending_intention: None,
        topic_continuity: 0.5,
        is_born: false,
        turn_count: 0,
        valence: None,
        commitment: None,
        coherence_integrity: 1.0,
        attributed_intent: "Unknown".to_string(),
        recent_turns: vec![],
        crystallized: vec![],
        positions: vec![],
    }))
}

// ═══════════════════════════════════════════════════════════════
// /api/visuals — grammatica visiva: SVG dei frattali + simplessi
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Serialize)]
pub struct FractalVisualDto {
    pub id: u32,
    pub name: String,
    pub svg: String,
    pub activation: f64,
}

#[derive(serde::Serialize)]
pub struct SimplexVisualDto {
    pub name: String,
    pub fractal_names: Vec<String>,
    pub svg: String,
    pub strength: f64,
    pub activation: f64,
}

#[derive(serde::Serialize)]
pub struct VisualsDto {
    pub fractals: Vec<FractalVisualDto>,
    pub simplices: Vec<SimplexVisualDto>,
}

pub async fn get_visuals(State(state): State<AppState>) -> Json<VisualsDto> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetVisuals { reply: tx }).await;
    Json(rx.await.unwrap_or(VisualsDto { fractals: vec![], simplices: vec![] }))
}

// ═══════════════════════════════════════════════════════════════
// GET /api/universe — Galassia esplorabile: frattali + parole
// ═══════════════════════════════════════════════════════════════

pub async fn get_universe(State(state): State<AppState>) -> Json<UniverseDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetUniverse { reply: tx }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/word_neighbors?word=xxx — Vicini di una parola
// ═══════════════════════════════════════════════════════════════

pub async fn get_word_neighbors(
    State(state): State<AppState>,
    Query(params): Query<WordQuery>,
) -> Json<WordNeighborsDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetWordNeighbors {
        word: params.word,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/word_detail?word=xxx — Dettaglio completo parola
// ═══════════════════════════════════════════════════════════════

pub async fn get_word_detail(
    State(state): State<AppState>,
    Query(params): Query<WordQuery>,
) -> Json<WordDetailDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetWordDetail {
        word: params.word,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// POST /api/word_connect — Aggiunge connessione curata
// ═══════════════════════════════════════════════════════════════

pub async fn post_word_connect(
    State(state): State<AppState>,
    Json(body): Json<WordConnectBody>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::AddWordConnect {
        from: body.from,
        relation: body.relation,
        to: body.to,
        via: body.via,
        confidence: body.confidence,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

// ═══════════════════════════════════════════════════════════════
// GET /api/cura/parole?q=&offset=&limit= — Lista parole paginata
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct WordListQuery {
    pub q: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub sort: Option<String>,
}

pub async fn get_word_list(
    State(state): State<AppState>,
    Query(params): Query<WordListQuery>,
) -> Json<crate::web::state::WordListDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetWordList {
        query: params.q.unwrap_or_default(),
        offset: params.offset.unwrap_or(0),
        limit: params.limit.unwrap_or(50),
        sort: params.sort.unwrap_or_else(|| "alpha_asc".to_string()),
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// DELETE /api/cura/relazione — Rimuove una relazione specifica
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct DeleteRelationBody {
    pub subject: String,
    pub relation: String,
    pub object: String,
}

pub async fn delete_word_relation(
    State(state): State<AppState>,
    Json(body): Json<DeleteRelationBody>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::DeleteWordRelation {
        subject: body.subject,
        relation: body.relation,
        object: body.object,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

// ═══════════════════════════════════════════════════════════════
// DELETE /api/cura/parola?word= — Rimuove una parola dal KG
// ═══════════════════════════════════════════════════════════════

pub async fn delete_word(
    State(state): State<AppState>,
    Query(params): Query<WordQuery>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::DeleteWord {
        word: params.word,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

// ═══════════════════════════════════════════════════════════════
// POST /api/cura/firma — Aggiorna firma 8D di una parola
// ═══════════════════════════════════════════════════════════════

pub async fn post_update_firma(
    State(state): State<AppState>,
    Json(body): Json<crate::web::state::UpdateFirmaBody>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::UpdateWordFirma {
        word: body.word,
        firma: body.firma,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

// ═══════════════════════════════════════════════════════════════
// POST /api/cura/relazione/modifica — Aggiorna confidence e/o via
// ═══════════════════════════════════════════════════════════════

pub async fn post_update_edge(
    State(state): State<AppState>,
    Json(body): Json<crate::web::state::UpdateEdgeBody>,
) -> Json<bool> {
    let via_update: Option<Option<String>> = if body.clear_via == Some(true) {
        Some(None)
    } else if let Some(v) = body.via {
        if v.trim().is_empty() { None } else { Some(Some(v)) }
    } else {
        None
    };
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::UpdateEdge {
        subject: body.subject,
        relation: body.relation,
        object: body.object,
        confidence: body.confidence,
        via: via_update,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

// ═══════════════════════════════════════════════════════════════
// GET /api/cura/categorie?rel=IS_A&min=3&q=
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct CategoriesQuery {
    pub rel: Option<String>,
    pub min: Option<usize>,
    pub q: Option<String>,
}

pub async fn get_categories(
    State(state): State<AppState>,
    Query(params): Query<CategoriesQuery>,
) -> Json<crate::web::state::CategoriesDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetCategories {
        relation: params.rel.unwrap_or_else(|| "IS_A".to_string()),
        min_children: params.min.unwrap_or(3),
        query: params.q.unwrap_or_default(),
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// POST /api/cura/pulizia-verbi?dry_run=true — Rimuove forme verbali coniugate
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct PuliziaQuery {
    pub dry_run: Option<bool>,
}

pub async fn post_pulizia_verbi(
    State(state): State<AppState>,
    Query(params): Query<PuliziaQuery>,
) -> Json<crate::web::state::PuliziaDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::PuliziaVerbi {
        dry_run: params.dry_run.unwrap_or(false),
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(crate::web::state::PuliziaDto {
        deleted: vec![], count: 0, dry_run: true
    }))
}

// ═══════════════════════════════════════════════════════════════
// GET /api/concept?word=xxx — Tutto ciò che il sistema sa di un concetto
// ═══════════════════════════════════════════════════════════════

pub async fn get_concept(
    State(state): State<AppState>,
    Query(params): Query<WordQuery>,
) -> Json<ConceptDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetConcept {
        word: params.word,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/self — Identità esplicita: credenze, valori, incertezze
// ═══════════════════════════════════════════════════════════════

pub async fn get_self(State(state): State<AppState>) -> Json<SelfDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetSelf { reply: tx }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/episodes?n=20 — Episodi semantici recenti
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct EpisodeQuery { pub n: Option<usize> }

pub async fn get_episodes(
    State(state): State<AppState>,
    Query(params): Query<EpisodeQuery>,
) -> Json<Vec<EpisodeDto>> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetEpisodes {
        n: params.n.unwrap_or(20),
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/episodes/recall?concepts=cane,animale — Recall per concetti
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct RecallQuery { pub concepts: String }

pub async fn recall_episodes(
    State(state): State<AppState>,
    Query(params): Query<RecallQuery>,
) -> Json<Vec<EpisodeDto>> {
    let concepts: Vec<String> = params.concepts
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::RecallEpisodes { concepts, reply: tx }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// Community Session API
// ═══════════════════════════════════════════════════════════════

/// GET /community — UI community HTML
pub async fn community_index() -> Html<&'static str> {
    Html(COMMUNITY_HTML)
}

/// POST /api/community/teach — insegna testo (da partecipante)
pub async fn post_community_teach(
    State(state): State<AppState>,
    Json(req): Json<CommunityTeachRequest>,
) -> Json<CommunityTeachDto> {
    let user_id = req.user_id.unwrap_or_else(|| "anonimo".to_string());
    let user_name = req.user_name.unwrap_or_else(|| "Anonimo".to_string());
    let user_context = req.user_context.unwrap_or_default();
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::CommunityTeach {
        text: req.text,
        user_id,
        user_name,
        user_context,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_else(|_| CommunityTeachDto {
        words_new: Vec::new(),
        words_known: Vec::new(),
        resonating_words: Vec::new(),
        fractals_touched: Vec::new(),
        connections_found: Vec::new(),
        field_energy_delta: 0.0,
    }))
}

/// POST /api/community/connect — aggiungi connessione KG curata
pub async fn post_community_connect(
    State(state): State<AppState>,
    Json(req): Json<CommunityConnectRequest>,
) -> Json<bool> {
    let user_id = req.user_id.unwrap_or_else(|| "anonimo".to_string());
    let user_name = req.user_name.unwrap_or_else(|| "Anonimo".to_string());
    let user_context = req.user_context.unwrap_or_default();
    // Converti strength 1-5 in confidenza 0.2-1.0
    let confidence = req.strength
        .map(|s| (s.clamp(1, 5) as f32) / 5.0)
        .unwrap_or(0.8);
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::CommunityValidateEdge {
        subject: req.subject,
        relation: req.relation,
        object: req.object,
        confidence,
        user_id,
        user_name,
        user_context,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

/// POST /api/community/validate — valida/aggiusta confidenza connessione
pub async fn post_community_validate(
    State(state): State<AppState>,
    Json(req): Json<CommunityValidateRequest>,
) -> Json<bool> {
    let user_id = req.user_id.unwrap_or_else(|| "anonimo".to_string());
    let user_name = req.user_name.unwrap_or_else(|| "Anonimo".to_string());
    let user_context = req.user_context.unwrap_or_default();
    let confidence = (req.resonance.clamp(1, 5) as f32) / 5.0;
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::CommunityValidateEdge {
        subject: req.subject,
        relation: req.relation,
        object: req.object,
        confidence,
        user_id,
        user_name,
        user_context,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

/// GET /api/community/session — stato sessione corrente
pub async fn get_community_session(State(state): State<AppState>) -> Json<SessionStateDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetSessionState { reply: tx }).await;
    Json(rx.await.unwrap_or_default())
}

/// GET /api/community/field — campo parole sessione (parole insegnate)
pub async fn get_community_field(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Restituisce le parole attive + stato sessione combinati
    let (tx_field, rx_field) = oneshot::channel();
    let (tx_session, rx_session) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetWordField { reply: tx_field }).await;
    let _ = state.cmd_tx.send(EngineCommand::GetSessionState { reply: tx_session }).await;
    let field = rx_field.await.unwrap_or_default();
    let session = rx_session.await.unwrap_or_default();
    Json(serde_json::json!({
        "top_words": field.top_words.iter().map(|w| &w.word).collect::<Vec<_>>(),
        "total_energy": field.total_energy,
        "session_words": session.teach_entries.iter()
            .flat_map(|e| e.words_new.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter().collect::<Vec<_>>(),
        "connections": session.community_edges,
    }))
}

/// POST /api/community/reset — nuova sessione
pub async fn post_community_reset(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Json<bool> {
    let name = body.get("community_name")
        .and_then(|v| v.as_str())
        .unwrap_or("comunita")
        .to_string();
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::ResetSession { community_name: name, reply: tx }).await;
    Json(rx.await.unwrap_or(false))
}

// ═══════════════════════════════════════════════════════════════
// Default per StateSnapshot (usato in caso di errore)
// ═══════════════════════════════════════════════════════════════

impl Default for StateSnapshot {
    fn default() -> Self {
        Self {
            vital: VitalDto {
                activation: 0.0,
                saturation: 0.0,
                curiosity: 0.0,
                fatigue: 0.0,
                tension: "Calm".to_string(),
            },
            active_fractals: Vec::new(),
            locus: None,
            dream_phase: "Awake".to_string(),
            dream_depth: 0.0,
            report: ReportDto {
                fractal_count: 0,
                simplex_count: 0,
                max_dimension: 0,
                connected_components: 0,
                memory_stm: 0,
                memory_mtm: 0,
                memory_ltm: 0,
                dream_cycles: 0,
                total_perturbations: 0,
                vocabulary_size: 0,
                emergent_dimensions: 0,
            },
            field_signature: vec![0.5; 8],
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Phase 52: Dialogo Interiore
// ═══════════════════════════════════════════════════════════════

/// GET /api/inner-dialogue — Aggregato di pensieri, domande e proposizioni
pub async fn get_inner_dialogue(
    State(state): State<AppState>,
) -> Json<InnerDialogueDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetInnerDialogue { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(InnerDialogueDto {
            thoughts: vec![],
            questions: vec![],
            propositions: vec![],
        }),
    }
}

/// POST /api/respond — L'utente risponde a un item del dialogo interiore
pub async fn post_respond(
    State(state): State<AppState>,
    Json(req): Json<RespondRequest>,
) -> Json<RespondResult> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::RespondToInsight {
        item_type: req.item_type,
        item_id: req.item_id,
        response: req.response,
        action: req.action,
        reply: tx,
    }).await;
    match rx.await {
        Ok(result) => Json(result),
        Err(_) => Json(RespondResult {
            success: false,
            effect: "Errore comunicazione con engine".to_string(),
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/will/focus — Modula la volontà focalizzando su un topic
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct WillFocusRequest {
    pub topic: String,
}

pub async fn post_will_focus(
    State(state): State<AppState>,
    Json(req): Json<WillFocusRequest>,
) -> Json<WillDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::WillFocus {
        topic: req.topic,
        reply: tx,
    }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(WillDto::default()),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/dream/report — Report dettagliato del sogno
// ═══════════════════════════════════════════════════════════════

pub async fn get_dream_report(State(state): State<AppState>) -> Json<DreamReportDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetDreamReport { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(DreamReportDto::default()),
    }
}

// ═══════════════════════════════════════════════════════════════
// GET /api/relations — Lista tutti i tipi di relazione (per menu)
// ═══════════════════════════════════════════════════════════════

pub async fn get_relations() -> Json<Vec<serde_json::Value>> {
    use crate::topology::relation::RelationType;
    let list: Vec<serde_json::Value> = RelationType::ALL.iter().map(|r| {
        serde_json::json!({
            "key": r.as_str(),
            "nome": r.nome(),
            "categoria": r.categoria(),
            "colore": r.colore(),
        })
    }).collect();
    Json(list)
}

// ═══════════════════════════════════════════════════════════════
// DELETE /api/edge — Cancella un arco KG
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct EdgeDeleteRequest {
    pub subject: String,
    pub relation: String,
    pub object: String,
}

pub async fn delete_edge(
    State(state): State<AppState>,
    Json(req): Json<EdgeDeleteRequest>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::DeleteEdge {
        subject: req.subject,
        relation: req.relation,
        object: req.object,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

// ═══════════════════════════════════════════════════════════════
// PATCH /api/edge — Modifica confidence di un arco KG
// ═══════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct EdgePatchRequest {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub confidence: f32,
}

pub async fn patch_edge(
    State(state): State<AppState>,
    Json(req): Json<EdgePatchRequest>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::PatchEdgeConfidence {
        subject: req.subject,
        relation: req.relation,
        object: req.object,
        confidence: req.confidence,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

// ═══════════════════════════════════════════════════════════════
// GET /api/biennale/field — Campo semantico 2D (galassia)
// ═══════════════════════════════════════════════════════════════

pub async fn get_biennale_field(State(state): State<AppState>) -> Json<BiennaleFieldDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetBiennaleField { reply: tx }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/biennale/word?word=X — Dettaglio parola con vicini KG
// ═══════════════════════════════════════════════════════════════

pub async fn get_biennale_word(
    State(state): State<AppState>,
    Query(params): Query<BiennaleWordQuery>,
) -> Json<BiennaleWordDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetBiennaleWord {
        word: params.word,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/biennale/journey?from=X&to=Y — Percorso BFS nel KG
// ═══════════════════════════════════════════════════════════════

pub async fn get_biennale_journey(
    State(state): State<AppState>,
    Query(params): Query<BiennaleJourneyQuery>,
) -> Json<BiennaleJourneyDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetBiennaleJourney {
        from: params.from,
        to: params.to,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}
