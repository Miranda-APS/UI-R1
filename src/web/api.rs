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
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use super::state::*;

// ═══════════════════════════════════════════════════════════════
// GET / — Serve la dashboard HTML
// ═══════════════════════════════════════════════════════════════

static COMMUNITY_HTML: &str = include_str!("community/index.html");
static BIENNALE_HOME_HTML: &str = include_str!("biennale/home.html");
static DIALOGO_HTML: &str = include_str!("biennale/dialogo.html");
static CURAZIONE_HTML: &str = include_str!("biennale/curazione.html");
static CURA_MOBILE_HTML: &str = include_str!("biennale/cura_mobile.html");
static COMPRENSIONE_HTML: &str = include_str!("comprensione.html");

/// Gate di Comprensione — pagina dedicata, leggera, ISOLATA da campovasto.
/// Vedi docs/raw/architettura/gate_di_comprensione.md.
pub async fn comprensione_index() -> Html<&'static str> {
    Html(COMPRENSIONE_HTML)
}

pub async fn index() -> Html<&'static str> {
    Html(BIENNALE_HOME_HTML)
}

/// Viste dedicate (hub/stato-interno/frattali) servite DA DISCO a runtime:
/// URL pulito + editabili senza ricompilare. Cartella top-level `viste/` (come
/// `campovasto/`), isolate dall'esploratore ma con design-system condiviso
/// (importano theme.js/font da /campovasto via path assoluto).
/// Vedi docs/raw/architettura/piano_ritiro_moduli.md.
async fn serve_vista(file: &str) -> Response {
    let path = std::path::Path::new("viste").join(file);
    match std::fs::read_to_string(&path) {
        Ok(s) => Html(s).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, format!("vista non trovata: {}", e)).into_response(),
    }
}

pub async fn admin_index() -> Response { serve_vista("hub.html").await }
pub async fn stato_interno_index() -> Response { serve_vista("stato-interno.html").await }
pub async fn frattali_index() -> Response { serve_vista("frattali.html").await }

pub async fn dialogo_index() -> Html<&'static str> {
    Html(DIALOGO_HTML)
}

pub async fn curazione_index() -> Html<&'static str> {
    Html(CURAZIONE_HTML)
}

// ═══════════════════════════════════════════════════════════════
// /cura-mobile — App offline per curazione (PWA)
// ═══════════════════════════════════════════════════════════════

pub async fn cura_mobile_index() -> Html<&'static str> {
    Html(CURA_MOBILE_HTML)
}

/// GET /cura-mobile/kg.json — serve il file kg.json corrente.
pub async fn cura_mobile_kg() -> impl axum::response::IntoResponse {
    let path = std::path::Path::new("prometeo_kg.json");
    match std::fs::read_to_string(path) {
        Ok(s) => ([(axum::http::header::CONTENT_TYPE, "application/json")], s).into_response(),
        Err(e) => (axum::http::StatusCode::NOT_FOUND, format!("kg.json non trovato: {}", e)).into_response(),
    }
}

/// GET /cura-mobile/firme.tsv — serve hub_signatures.tsv (solo gli hub curati a mano).
pub async fn cura_mobile_firme() -> impl axum::response::IntoResponse {
    let path = std::path::Path::new("data/anchors/hub_signatures.tsv");
    match std::fs::read_to_string(path) {
        Ok(s) => ([(axum::http::header::CONTENT_TYPE, "text/tab-separated-values; charset=utf-8")], s).into_response(),
        Err(e) => (axum::http::StatusCode::NOT_FOUND, format!("firme.tsv non trovato: {}", e)).into_response(),
    }
}

/// GET /cura-mobile/standalone.html — HTML autosufficiente con TUTTI i dati
/// incorporati. Da scaricare una volta (online), salvare sul telefono, aprire
/// come file qualsiasi. Funziona da `file://` senza server.
pub async fn cura_mobile_standalone(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    // 1. Leggi kg.json
    let kg_text = match std::fs::read_to_string("prometeo_kg.json") {
        Ok(s) => s,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("kg.json non trovato: {}", e)).into_response(),
    };
    let kg: serde_json::Value = match serde_json::from_str(&kg_text) {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("kg.json invalido: {}", e)).into_response(),
    };
    let edges_json = serde_json::to_string(&kg["edges"]).unwrap_or_else(|_| "[]".into());

    // 2. Prendi tutte le firme dal motore
    let (tx, rx) = oneshot::channel();
    if state.cmd_tx.send(EngineCommand::GetAllFirme { reply: tx }).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "engine offline").into_response();
    }
    let firme = match rx.await {
        Ok(v) => v,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "no reply").into_response(),
    };
    let firme_json: String = {
        let pairs: Vec<serde_json::Value> = firme.into_iter()
            .map(|(w, sig)| {
                let arr: Vec<f64> = sig.iter().map(|v| (v * 1000.0).round() / 1000.0).collect();
                serde_json::json!([w, arr])
            })
            .collect();
        serde_json::to_string(&pairs).unwrap_or_else(|_| "[]".into())
    };

    // 3. Inietta lo <script id="bundled-data"> nell'HTML
    let bundled_script = format!(
        r##"<script id="bundled-data" type="application/json">{{"edges":{},"firme":{}}}</script>"##,
        edges_json, firme_json
    );
    let html = CURA_MOBILE_HTML.replace("</body>", &format!("{}\n</body>", bundled_script));

    ([(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response()
}

/// GET /cura-mobile/all_firme.tsv — esporta TUTTE le firme dal `.bin` corrente.
/// Include sia gli hub curati che le 17.384 propagate da Phase 70 v4.
pub async fn cura_mobile_all_firme(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let (tx, rx) = oneshot::channel();
    if state.cmd_tx.send(EngineCommand::GetAllFirme { reply: tx }).await.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "engine offline").into_response();
    }
    let firme = match rx.await {
        Ok(v) => v,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "no reply").into_response(),
    };
    let mut body = String::with_capacity(firme.len() * 80);
    body.push_str("word\tagency\tpermanenza\tintensita\ttempo\tconfine\tcomplessita\tdefinizione\tvalenza\n");
    for (w, sig) in firme {
        body.push_str(&w);
        for v in sig.iter() {
            body.push('\t');
            body.push_str(&format!("{:.3}", v));
        }
        body.push('\n');
    }
    ([(axum::http::header::CONTENT_TYPE, "text/tab-separated-values; charset=utf-8")], body).into_response()
}

/// GET /cura-mobile/manifest.json — PWA manifest minimo.
pub async fn cura_mobile_manifest() -> impl axum::response::IntoResponse {
    // start_url e scope con trailing slash: il service worker copre tutto sotto.
    let body = r##"{
  "name": "Cura KG · Prometeo",
  "short_name": "Cura",
  "start_url": "/cura-mobile/",
  "scope": "/cura-mobile/",
  "display": "standalone",
  "orientation": "portrait",
  "background_color": "#0a0a14",
  "theme_color": "#0a0a14",
  "icons": [
    {"src": "/cura-mobile/icon.svg", "sizes": "512x512", "type": "image/svg+xml", "purpose": "any maskable"}
  ]
}"##;
    ([(axum::http::header::CONTENT_TYPE, "application/manifest+json")], body).into_response()
}

/// GET /cura-mobile/sw.js — service worker minimo (cache statica + navigate fallback).
pub async fn cura_mobile_sw() -> impl axum::response::IntoResponse {
    let body = r#"// Service worker: cache shell offline + navigate fallback per qualunque /cura-mobile/*.
const CACHE = 'cura-shell-v4';
const SHELL = ['/cura-mobile/', '/cura-mobile/manifest.json', '/cura-mobile/icon.svg'];

self.addEventListener('install', (e) => {
  e.waitUntil(caches.open(CACHE).then(c => c.addAll(SHELL)));
  self.skipWaiting();
});

self.addEventListener('activate', (e) => {
  e.waitUntil(caches.keys().then(ks => Promise.all(ks.filter(k => k !== CACHE).map(k => caches.delete(k)))));
  self.clients.claim();
});

self.addEventListener('fetch', (e) => {
  const req = e.request;
  const u = new URL(req.url);

  // Navigate (apertura PWA o reload): serve la shell cached, anche offline.
  if(req.mode === 'navigate' && u.pathname.startsWith('/cura-mobile')){
    e.respondWith(
      caches.match('/cura-mobile/').then(cached => cached || fetch(req).catch(() => caches.match('/cura-mobile/')))
    );
    return;
  }

  // Asset statici della shell: cache-first.
  if(SHELL.includes(u.pathname)){
    e.respondWith(caches.match(req).then(r => r || fetch(req)));
    return;
  }

  // kg.json e all_firme.tsv: network-first con fallback. L'app salva in IndexedDB
  // dopo il primo fetch — quindi non serve cacharli qui (sarebbe spreco di MB).
});
"#;
    ([(axum::http::header::CONTENT_TYPE, "application/javascript")], body).into_response()
}

/// GET /cura-mobile/icon.svg — icona SVG per PWA.
pub async fn cura_mobile_icon() -> impl axum::response::IntoResponse {
    let body = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><rect width="512" height="512" fill="#0a0a14"/><circle cx="256" cy="256" r="170" fill="none" stroke="#6a8fff" stroke-width="14"/><circle cx="256" cy="256" r="60" fill="#b07cff"/><circle cx="256" cy="120" r="22" fill="#80ff9a"/><circle cx="256" cy="392" r="22" fill="#ffb060"/><circle cx="120" cy="256" r="22" fill="#ff7878"/><circle cx="392" cy="256" r="22" fill="#5AAFE8"/><line x1="256" y1="142" x2="256" y2="196" stroke="#6a8fff" stroke-width="6"/><line x1="256" y1="316" x2="256" y2="370" stroke="#6a8fff" stroke-width="6"/><line x1="142" y1="256" x2="196" y2="256" stroke="#6a8fff" stroke-width="6"/><line x1="316" y1="256" x2="370" y2="256" stroke="#6a8fff" stroke-width="6"/></svg>"##;
    ([(axum::http::header::CONTENT_TYPE, "image/svg+xml")], body).into_response()
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
            understanding: None,
            deliberation: None,
            speaker_profile: None,
            comprehension_report: None,
            action_decision: None,
            sentence_proposition: None,
            kg_confrontation: None,
            need: None,
            propositions: Vec::new(),
        }),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/comprehend — P1 (Tsunami): comprensione STATELESS
// Ritorna lemmi + proposizioni multi-locus + concetti (+ need) SENZA
// mutare lo stato dell'engine. Per analizzare titoli-task isolati e ogni
// analisi puntuale senza che N chiamate si contaminino (al contrario di
// /api/input, stateful). Non fa broadcast WS (non cambia nulla).
// ═══════════════════════════════════════════════════════════════

pub async fn post_comprehend(
    State(state): State<AppState>,
    Json(req): Json<InputRequest>,
) -> Json<ComprehendDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Comprehend {
        text: req.text,
        reply: tx,
    }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(ComprehendDto::default()),
    }
}

// ═══════════════════════════════════════════════════════════════
// POST /api/analyze — MODALITÀ OSSERVATORE (analisi di testi di terzi)
// Segmenta un testo (anche lungo) in frasi, comprende ciascuna in modo
// STATELESS e COMPATTO, e aggrega. Niente cornice "io sono il destinatario",
// niente mutazione di stato. Per trascrizioni/verbali/testi lunghi.
// ═══════════════════════════════════════════════════════════════

pub async fn post_analyze(
    State(state): State<AppState>,
    Json(req): Json<InputRequest>,
) -> Json<AnalyzeDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Analyze {
        text: req.text,
        reply: tx,
    }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(AnalyzeDto::default()),
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
// GET /api/explore?word=X — UI-r1 prova a collocare la parola DA SOLA
// (cammino multi-hop tipato verso un'ancora; vista Stato interno).
// ═══════════════════════════════════════════════════════════════

pub async fn get_explore(
    State(state): State<AppState>,
    Query(params): Query<WordQuery>,
) -> Json<crate::web::state::ExploreDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::ExploreWord {
        word: params.word,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// POST /api/clarity — l'utente FONDA un'incertezza aperta
// engine.receive_clarity → insegna il testo + abbassa la tensione
// (resolve_uncertainty). È il modo reale di "chiudere" un'incertezza
// dalla vista Stato interno: l'entità chiede, l'utente risponde.
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct ClarityBody {
    pub topic: String,
    pub illumination: String,
}

pub async fn post_clarity(
    State(state): State<AppState>,
    Json(body): Json<ClarityBody>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Clarity {
        topic: body.topic,
        illumination: body.illumination,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}

// === IAm-gotchi (glass-box) — Step 5: correzione del modello-dell'Altro ===
pub async fn post_correct_interlocutor(
    State(state): State<AppState>,
    Json(body): Json<crate::web::state::CorrectInterlocutorBody>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::CorrectInterlocutor {
        intent: body.intent,
        emotional_valence: body.emotional_valence,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(false))
}
// === fine IAm-gotchi ===

// ═══════════════════════════════════════════════════════════════
// POST /api/correct — Phase 84: l'utente corregge la risposta del sistema
// ═══════════════════════════════════════════════════════════════

pub async fn post_correct(
    State(state): State<AppState>,
    Json(body): Json<crate::web::state::CorrectBody>,
) -> Json<crate::web::state::CorrectDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Correct {
        input: body.input,
        given: body.given,
        wanted: body.wanted,
        context: body.context,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_else(|_| crate::web::state::CorrectDto {
        accepted: false,
        message: "Errore di canale interno".to_string(),
        ..Default::default()
    }))
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
    pub only_kg: Option<bool>,
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
        only_kg: params.only_kg.unwrap_or(true),  // default ON: niente forme flesse del lessico
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
// POST /api/cura/rinomina — Rinomina parola (merge KG + lessico)
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct RinominaBody {
    pub from: String,
    pub to: String,
}

pub async fn post_rinomina(
    State(state): State<AppState>,
    Json(body): Json<RinominaBody>,
) -> Json<bool> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::RinominaWord {
        from: body.from.trim().to_lowercase(),
        to: body.to.trim().to_lowercase(),
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
// POST /api/cura/normalizza-accenti?dry_run=true
// ═══════════════════════════════════════════════════════════════

pub async fn post_normalizza_accenti(
    State(state): State<AppState>,
    Query(params): Query<PuliziaQuery>,
) -> Json<crate::web::state::NormalizzaDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::NormalizzaAccenti {
        dry_run: params.dry_run.unwrap_or(false),
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or(crate::web::state::NormalizzaDto {
        pairs: vec![], count: 0, dry_run: true
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
// GET /api/speaker — P2 (Tsunami): ritratto-utente (SpeakerProfile)
// Persistito cross-sessione: name, self_facts, entity_facts,
// open_questions, gaps (open+closed con closed_by), mentioned, corrections.
// ═══════════════════════════════════════════════════════════════

pub async fn get_speaker_profile(State(state): State<AppState>) -> Json<SpeakerProfileDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetSpeakerProfile { reply: tx }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// POST /api/persist — P2 (Tsunami): forza il salvataggio del .bin
// L'app lo chiama sui lifecycle event (onPause/onStop) per persistere
// il vissuto (SpeakerProfile, narrativa, identità, simplessi). Ritorna {ok}.
// ═══════════════════════════════════════════════════════════════

pub async fn post_persist(State(state): State<AppState>) -> Json<serde_json::Value> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::Persist { reply: tx }).await;
    let ok = rx.await.unwrap_or(false);
    Json(serde_json::json!({ "ok": ok }))
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

/// POST /api/community/transmit_batch — trasmissione batch dal campo nuovo
/// al KG. Insegna molte parole + impone firme + aggiunge molti archi in
/// UN solo comando engine. Drasticamente più veloce del flusso 1-by-1.
pub async fn post_community_transmit_batch(
    State(state): State<AppState>,
    Json(req): Json<crate::web::state::TransmitBatchRequest>,
) -> Json<crate::web::state::TransmitBatchDto> {
    let user_id = req.user_id.unwrap_or_else(|| "anonimo".to_string());
    let user_name = req.user_name.unwrap_or_else(|| "Anonimo".to_string());
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::TransmitBatch {
        words: req.words,
        edges: req.edges,
        user_id,
        user_name,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
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
                kg_edge_count: 0,
            },
            field_signature: vec![0.5; 8],
        }
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

// GET /api/biennale/field_all — TUTTO il lessico (nessun filtro), per campovastotest
pub async fn get_biennale_field_all(State(state): State<AppState>) -> Json<BiennaleFieldDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetBiennaleFieldAll { reply: tx }).await;
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

// ═══════════════════════════════════════════════════════════════
// GET /api/biennale/circuit?w1=X&w2=Y — Circuito di attivazione
// ═══════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════
// GET /api/understanding?sentence=... — Comprensione di una frase
// ═══════════════════════════════════════════════════════════════
//
// Legge il KG per ogni parola della frase (read-only, non muta il lessico).
// Restituisce attribuzioni al parlante, ipotesi aperte, catene inferenziali
// 2-hop. Pensato per community: guida la curazione mostrando cosa l'entità
// deduce e cosa resta sotto-definito.

#[derive(serde::Deserialize)]
pub struct UnderstandingQuery {
    pub sentence: String,
}

pub async fn get_understanding(
    State(state): State<AppState>,
    Query(params): Query<UnderstandingQuery>,
) -> Json<super::state::SceneUnderstandingDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetUnderstanding {
        sentence: params.sentence,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_else(|_| super::state::SceneUnderstandingDto {
        syntactic_role: "Statement".to_string(),
        lemmas: vec![],
        unknown_words: vec![],
        comprehension_depth: 0,
        summary: String::new(),
        proposed_edges: vec![],
        words: vec![],
        open_hypotheses: vec![],
        inferential_chains: vec![],
        syntactic_edges: vec![],
        graph: None,
    }))
}

// ═══════════════════════════════════════════════════════════════
// POST /api/kg/confirm_edge + /api/kg/reject_edge
// ═══════════════════════════════════════════════════════════════

#[derive(serde::Deserialize)]
pub struct EdgeProposal {
    pub subject: String,
    pub relation: String,
    pub object: String,
    #[serde(default)]
    pub confidence: Option<f32>,
}

pub async fn post_confirm_edge(
    State(state): State<AppState>,
    Json(body): Json<EdgeProposal>,
) -> Json<super::state::ConfirmEdgeResultDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::ConfirmEdge {
        subject: body.subject,
        relation: body.relation,
        object: body.object,
        confidence: body.confidence.unwrap_or(0.7),
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// Save/Load collective fields (campo nuovo / campo medio)
// ═══════════════════════════════════════════════════════════════
//
// Il gruppo può salvare un campo sul server per ripenderlo poi. Ogni save
// ha un nome visibile (e uno slug derivato) + una password. Il payload è
// qualsiasi JSON (la UI salva l'output di Field.toJSON()).
//
// Storage: data/saved_fields/<slug>.json. Non-cryptographic: hash SHA-256
// della password + salt fisso (sufficiente per l'uso community, non per
// contesti ad alta sicurezza).

use sha2::{Sha256, Digest};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SavedFieldEntry {
    pub slug: String,
    pub name: String,
    pub field_id: String,
    pub created_at: u64,
    pub password_hash: String,
    pub data: serde_json::Value,
}

#[derive(serde::Serialize)]
pub struct SavedFieldMeta {
    pub slug: String,
    pub name: String,
    pub field_id: String,
    pub created_at: u64,
}

fn password_hash(password: &str) -> String {
    let salt = "uir1-community-2026";
    let mut h = Sha256::new();
    h.update(salt.as_bytes());
    h.update(password.as_bytes());
    format!("{:x}", h.finalize())
}

fn slugify(name: &str) -> String {
    let mut s = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() { s.push(c.to_ascii_lowercase()); }
        else if c == ' ' || c == '-' || c == '_' { s.push('_'); }
    }
    let s = s.trim_matches('_').to_string();
    if s.is_empty() { "campo".to_string() } else { s }
}

fn saved_fields_dir() -> std::path::PathBuf {
    let p = std::path::PathBuf::from("data/saved_fields");
    let _ = std::fs::create_dir_all(&p);
    p
}

#[derive(serde::Deserialize)]
pub struct SaveFieldRequest {
    pub name: String,
    pub password: String,
    pub field_id: String,
    pub data: serde_json::Value,
}

#[derive(serde::Serialize)]
pub struct SaveFieldResponse {
    pub ok: bool,
    pub message: String,
    pub slug: Option<String>,
}

pub async fn post_save_field(Json(body): Json<SaveFieldRequest>) -> Json<SaveFieldResponse> {
    if body.name.trim().is_empty() || body.password.is_empty() {
        return Json(SaveFieldResponse {
            ok: false,
            message: "nome e password sono obbligatori".to_string(),
            slug: None,
        });
    }
    // Slug univoco: se esiste già, incrementa suffisso numerico
    let base_slug = slugify(&body.name);
    let dir = saved_fields_dir();
    let mut slug = base_slug.clone();
    let mut n = 2;
    while dir.join(format!("{}.json", slug)).exists() {
        slug = format!("{}-{}", base_slug, n);
        n += 1;
        if n > 999 {
            return Json(SaveFieldResponse {
                ok: false,
                message: "troppi campi con nome simile".to_string(),
                slug: None,
            });
        }
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let entry = SavedFieldEntry {
        slug: slug.clone(),
        name: body.name.trim().to_string(),
        field_id: body.field_id,
        created_at: ts,
        password_hash: password_hash(&body.password),
        data: body.data,
    };
    let path = dir.join(format!("{}.json", slug));
    match serde_json::to_vec_pretty(&entry).and_then(|v| { std::fs::write(&path, &v).map_err(|e| serde_json::Error::io(e)) }) {
        Ok(_) => Json(SaveFieldResponse {
            ok: true,
            message: format!("campo salvato come '{}'", slug),
            slug: Some(slug),
        }),
        Err(e) => Json(SaveFieldResponse {
            ok: false,
            message: format!("errore salvataggio: {}", e),
            slug: None,
        }),
    }
}

pub async fn get_saved_fields_list() -> Json<Vec<SavedFieldMeta>> {
    let dir = saved_fields_dir();
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") { continue; }
            if let Ok(bytes) = std::fs::read(&p) {
                if let Ok(entry) = serde_json::from_slice::<SavedFieldEntry>(&bytes) {
                    out.push(SavedFieldMeta {
                        slug: entry.slug,
                        name: entry.name,
                        field_id: entry.field_id,
                        created_at: entry.created_at,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Json(out)
}

#[derive(serde::Deserialize)]
pub struct LoadFieldRequest {
    pub slug: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoadFieldResponse {
    pub ok: bool,
    pub message: String,
    pub name: Option<String>,
    pub field_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
pub struct DeleteFieldRequest {
    pub slug: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct DeleteFieldResponse {
    pub ok: bool,
    pub message: String,
}

pub async fn post_delete_field(Json(body): Json<DeleteFieldRequest>) -> Json<DeleteFieldResponse> {
    let dir = saved_fields_dir();
    let path = dir.join(format!("{}.json", body.slug));
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return Json(DeleteFieldResponse {
            ok: false, message: "campo non trovato".to_string(),
        }),
    };
    let entry: SavedFieldEntry = match serde_json::from_slice(&bytes) {
        Ok(e) => e,
        Err(_) => return Json(DeleteFieldResponse {
            ok: false, message: "errore lettura campo".to_string(),
        }),
    };
    if entry.password_hash != password_hash(&body.password) {
        return Json(DeleteFieldResponse {
            ok: false, message: "password errata".to_string(),
        });
    }
    match std::fs::remove_file(&path) {
        Ok(_) => Json(DeleteFieldResponse {
            ok: true, message: format!("campo '{}' eliminato", entry.name),
        }),
        Err(e) => Json(DeleteFieldResponse {
            ok: false, message: format!("errore eliminazione: {}", e),
        }),
    }
}

pub async fn post_load_field(Json(body): Json<LoadFieldRequest>) -> Json<LoadFieldResponse> {
    let dir = saved_fields_dir();
    let path = dir.join(format!("{}.json", body.slug));
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return Json(LoadFieldResponse {
            ok: false, message: "campo non trovato".to_string(),
            name: None, field_id: None, data: None,
        }),
    };
    let entry: SavedFieldEntry = match serde_json::from_slice(&bytes) {
        Ok(e) => e,
        Err(e) => return Json(LoadFieldResponse {
            ok: false, message: format!("errore lettura: {}", e),
            name: None, field_id: None, data: None,
        }),
    };
    if entry.password_hash != password_hash(&body.password) {
        return Json(LoadFieldResponse {
            ok: false, message: "password errata".to_string(),
            name: None, field_id: None, data: None,
        });
    }
    Json(LoadFieldResponse {
        ok: true,
        message: "campo caricato".to_string(),
        name: Some(entry.name),
        field_id: Some(entry.field_id),
        data: Some(entry.data),
    })
}

pub async fn post_reject_edge(
    State(state): State<AppState>,
    Json(body): Json<EdgeProposal>,
) -> Json<super::state::ConfirmEdgeResultDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::RejectEdge {
        subject: body.subject,
        relation: body.relation,
        object: body.object,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════
// GET /api/medio?sentence=... — Dati completi per il campo medio
// ═══════════════════════════════════════════════════════════════
//
// Per ogni lemma della frase restituisce firma 8D + TUTTI gli archi KG
// (nessun cap, nessun filtro vasto) con firme dei target. Usato dalla
// frontend buildMedio per costruire un campo medio completo, non parziale.

pub async fn get_medio_data(
    State(state): State<AppState>,
    Query(params): Query<UnderstandingQuery>,
) -> Json<super::state::MedioDataDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetMedioData {
        sentence: params.sentence,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

pub async fn get_biennale_circuit(
    State(state): State<AppState>,
    Query(params): Query<BiennaleCircuitQuery>,
) -> Json<BiennaleCircuitDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetBiennaleCircuit {
        w1: params.w1,
        w2: params.w2,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_default())
}

// ═══════════════════════════════════════════════════════════════════
// Phase 69 — endpoint di osservazione del tempo proprio dell'entità
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/events/stats
///
/// Statistiche aggregate: quanti eventi sono entrati nella vita interna,
/// quanti sono stati ignorati come simili (debounced), quanti dimenticati
/// (sotto soglia), il materiale in digestione, i ricordi accumulati,
/// lo stato della finestra di riflessività.
pub async fn get_events_stats(State(state): State<AppState>) -> Json<EventsStatsDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetEventsStats { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(EventsStatsDto {
            emitted_count: 0,
            debounced_count: 0,
            forgotten_count: 0,
            pending_digestion_count: 0,
            semantic_episodes_count: 0,
            notices_in_window: 0,
            notices_max_per_window: 5,
            is_overloaded: false,
        }),
    }
}

/// GET /api/admin/events/pending
///
/// La coda di digestione corrente (fino a 32 eventi medio-salienti
/// in attesa di essere consolidati in REM).
pub async fn get_pending_digestion(State(state): State<AppState>) -> Json<PendingDigestionDto> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::GetPendingDigestion { reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(PendingDigestionDto {
            events: Vec::new(),
            capacity: 32,
        }),
    }
}

/// GET /api/admin/events/recent_episodes?limit=10
///
/// Gli ultimi N ricordi semantici dell'entità. Ciascuno con sintesi,
/// concetti chiave, stato emotivo e frattali dominanti al momento
/// della formazione del ricordo.
#[derive(serde::Deserialize)]
pub struct RecentEpisodesQuery {
    pub limit: Option<usize>,
}

pub async fn get_recent_episodes(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<RecentEpisodesQuery>,
) -> Json<RecentEpisodesDto> {
    let (tx, rx) = oneshot::channel();
    let limit = q.limit.unwrap_or(10).min(50);
    let _ = state.cmd_tx.send(EngineCommand::GetRecentEpisodes { limit, reply: tx }).await;
    match rx.await {
        Ok(dto) => Json(dto),
        Err(_) => Json(RecentEpisodesDto {
            episodes: Vec::new(),
            total_count: 0,
        }),
    }
}

// ═══════════════════════════════════════════════════════════════════
// Phase 82 — Memoria-sfera di haiku
// ═══════════════════════════════════════════════════════════════════
//
// Ogni evento cognitivo (osservazione, comprensione, atto di
// nominazione) può essere cristallizzato come haiku: tre versi densi
// posizionati su uno dei 64 attrattori frattali I Ching. Le tangenze
// (cerchi che si toccano sulla sfera) emergono per ancore lessicali
// condivise (≥2) o per trigramma I Ching in comune.
//
// La memoria è PERSISTENTE su `haiku_memory.json` accanto al `.bin`.
// Vive nel web layer (AppState), non dentro Engine — è organo nuovo,
// ispezionabile/curabile/cancellabile indipendentemente dal sostrato
// cognitivo principale. Ogni `deposit` sincronizza il salvataggio.

use crate::topology::haiku_memory::{HaikuCristallizzato, HaikuMemoryStats};

const HAIKU_MEMORY_PATH: &str = "haiku_memory.json";

#[derive(Deserialize)]
pub struct DepositHaikuBody {
    pub verses: [String; 3],
    pub fractal_id: u32,
    pub anchors: Vec<String>,
    pub source: Option<String>,
    pub note: Option<String>,
}

#[derive(Serialize)]
pub struct DepositHaikuResponse {
    pub id: String,
    pub tangencies: Vec<String>,
    pub total: usize,
}

/// POST /api/haiku/deposit
pub async fn post_haiku_deposit(
    State(state): State<AppState>,
    Json(body): Json<DepositHaikuBody>,
) -> Json<DepositHaikuResponse> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let haiku = HaikuCristallizzato {
        id: String::new(),
        verses: body.verses,
        fractal_id: body.fractal_id & 0x3F, // clamp 0..=63
        anchors: body.anchors,
        tangencies: Vec::new(),
        timestamp: ts,
        source: body.source.unwrap_or_else(|| "claude".to_string()),
        note: body.note,
    };
    let (id, tangencies, total) = {
        let mut mem = state.haiku_memory.lock().unwrap();
        let id = mem.deposit(haiku);
        let tangencies = mem
            .get(&id)
            .map(|h| h.tangencies.clone())
            .unwrap_or_default();
        let total = mem.len();
        // Save sincrono — file piccolo, write veloce.
        if let Err(e) = mem.save_to_file(std::path::Path::new(HAIKU_MEMORY_PATH)) {
            eprintln!("[haiku-memory] save failed: {}", e);
        }
        (id, tangencies, total)
    };
    Json(DepositHaikuResponse {
        id,
        tangencies,
        total,
    })
}

#[derive(Deserialize)]
pub struct RecallHaikuBody {
    pub fractal_id: u32,
    #[serde(default)]
    pub anchors: Vec<String>,
    pub n: Option<usize>,
}

#[derive(Serialize)]
pub struct RecallHaikuResponse {
    pub crystals: Vec<HaikuCristallizzato>,
    pub total_in_memory: usize,
}

/// POST /api/haiku/recall
///
/// Recall geometrico sulla sfera. Combina distanza frattale + ancore
/// condivise (le ancore dominano: β=5.0 vs α=1.0).
pub async fn post_haiku_recall(
    State(state): State<AppState>,
    Json(body): Json<RecallHaikuBody>,
) -> Json<RecallHaikuResponse> {
    let n = body.n.unwrap_or(5).min(50);
    let mem = state.haiku_memory.lock().unwrap();
    let crystals: Vec<HaikuCristallizzato> = mem
        .recall_by_proximity(body.fractal_id & 0x3F, &body.anchors, n)
        .into_iter()
        .cloned()
        .collect();
    Json(RecallHaikuResponse {
        crystals,
        total_in_memory: mem.len(),
    })
}

/// GET /api/haiku/stats
pub async fn get_haiku_stats(State(state): State<AppState>) -> Json<HaikuMemoryStats> {
    let mem = state.haiku_memory.lock().unwrap();
    Json(mem.snapshot_stats())
}

#[derive(Deserialize)]
pub struct HaikuListQuery {
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct HaikuListResponse {
    pub crystals: Vec<HaikuCristallizzato>,
    pub total: usize,
}

// ═══════════════════════════════════════════════════════════════════
// Phase 83 — Simplessi grammaticali (educazione live)
// ═══════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct AddGrammarSimplexBody {
    /// Sequenza di parole-perno (es. ["rispetto", "a"]).
    pub words: Vec<String>,
    /// Categoria libera (es. "preposizione_composta", "locuzione_fatica").
    /// Non è un dispatch — è etichetta per ispezione/curation.
    pub category: String,
    /// Nome del frattale-funzione che il simplesso attiva quando emerge
    /// (es. "RELAZIONE", "SALUTO", "POSSIBILITA"). Risolto via FractalRegistry.
    pub function_fractal_name: String,
}

/// POST /api/grammar_simplex — insegna un simplesso grammaticale.
/// La modifica è persistente: viene salvata al prossimo `save_to_binary`.
/// Vedi `Engine::add_grammar_simplex` e Phase 83 in CLAUDE.md.
pub async fn post_grammar_simplex(
    State(state): State<AppState>,
    Json(body): Json<AddGrammarSimplexBody>,
) -> Json<Result<AddGrammarSimplexResponse, String>> {
    let (tx, rx) = oneshot::channel();
    let _ = state.cmd_tx.send(EngineCommand::AddGrammarSimplex {
        words: body.words,
        category: body.category,
        function_fractal_name: body.function_fractal_name,
        reply: tx,
    }).await;
    Json(rx.await.unwrap_or_else(|_| Err("engine channel closed".into())))
}

/// GET /api/haiku/all?limit=N — dump cristalli (più recenti prima).
pub async fn get_haiku_all(
    State(state): State<AppState>,
    Query(q): Query<HaikuListQuery>,
) -> Json<HaikuListResponse> {
    let limit = q.limit.unwrap_or(50).min(500);
    let mem = state.haiku_memory.lock().unwrap();
    let mut all: Vec<HaikuCristallizzato> = mem.haikus.clone();
    all.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    let total = all.len();
    all.truncate(limit);
    Json(HaikuListResponse {
        crystals: all,
        total,
    })
}
