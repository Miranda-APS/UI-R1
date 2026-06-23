// prometeo-mcp — Phase 82
//
// Server MCP (Model Context Protocol) che espone UI-r1 come substrato
// cognitivo strutturato a un client LLM (Claude Desktop, Claude Code,
// qualunque client MCP via stdio).
//
// Architettura (decisione 1A): HTTP-wrapper. Questo processo parla MCP
// con il client via stdio e fa requests REST al server prometeo-web in
// esecuzione su PROMETEO_WEB_URL (default http://127.0.0.1:3000). Un
// solo engine, un solo .bin, una sola sessione viva — condivisa tra la
// UI campovasto e il client MCP. Quando l'LLM interroga UI-r1 mentre
// la web UI è aperta, vedono lo stesso UI-r1 nello stesso momento.
//
// Semantica della chiamata (decisione 2A): ogni `comprehend(input)` è
// un turno reale di dialogo. L'engine incrementa tick, aggiorna
// NarrativeSelf, scrive in SpeakerProfile, modula PF1. L'LLM non è
// spettatore — è interlocutore. Il parametro `speaker_id` è oggi
// future-proof (engine ignora) e diventerà la chiave per multi-speaker
// quando SpeakerProfile sarà multi-istanza.
//
// Avvio:
//   1. Lancia prometeo-web (cargo run --release --bin prometeo-web)
//   2. cargo run --release --features mcp --bin prometeo-mcp
//      (parla con stdin/stdout — usare un client MCP, non un terminale)
//
// Variabili d'ambiente:
//   PROMETEO_WEB_URL   base URL del server prometeo-web (default 127.0.0.1:8080)
//   PROMETEO_MCP_LOG   se "1", log diagnostico su stderr

use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────
// Tipi di request — schemi JSON-Schema generati automaticamente
// ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ComprehendRequest {
    /// Frase italiana che UI-r1 deve comprendere. È un turno reale di
    /// dialogo: modificherà lo stato vivo (tick, NarrativeSelf,
    /// SpeakerProfile, PF1). Non è una query read-only.
    pub input: String,
    /// Identificativo opzionale dell'interlocutore (oggi ignorato lato
    /// engine; diventerà la chiave per multi-speaker quando SpeakerProfile
    /// supporterà più istanze). Convenzione: "claude", "user", "guest", ecc.
    pub speaker_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct WordRequest {
    /// La parola da interrogare nel knowledge graph / lessico.
    pub word: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct QueryKgRequest {
    /// Parola-soggetto su cui interrogare il knowledge graph.
    pub word: String,
    /// Numero massimo di vicini da restituire (default 30).
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DepositHaikuRequest {
    /// I tre versi (5-7-5 sillabe — chiarezza > poesia, la conta non
    /// viene verificata: è una forma-vincolo di densità).
    pub verses: [String; 3],
    /// Frattale dominante I Ching al deposito (0..=63). 64 attrattori,
    /// 8 trigrammi inferiori × 8 superiori. Posiziona il cristallo
    /// sulla sfera. Recuperabile dallo stato corrente via
    /// `get_active_fractals`.
    pub fractal_id: u32,
    /// Parole-perno (2-6). Le tangenze con altri cristalli si formano
    /// per ancore condivise (≥2, case-insensitive).
    pub anchors: Vec<String>,
    /// Chi cristallizza ("claude", "user", "uir1", "system"). Default "claude".
    pub source: Option<String>,
    /// Annotazione libera, ≤280 caratteri raccomandati.
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AddGrammarSimplexRequest {
    /// Sequenza di parole-perno in ordine (es. ["rispetto", "a"]). Quando
    /// la sequenza appare nell'input in adiacenza nell'ordine indicato, il
    /// simplesso si attiva.
    pub words: Vec<String>,
    /// Categoria libera per ispezione/curation (es. "preposizione_composta",
    /// "locuzione_fatica", "costrutto_modale", "tempo_composto"). NON è un
    /// dispatch — è etichetta informativa. La funzione vera del simplesso
    /// è data dal `function_fractal_name`.
    pub category: String,
    /// Nome del frattale-funzione che il simplesso attiva quando emerge
    /// (es. "RELAZIONE" per `[rispetto, a]`, "SALUTO" per `[come, stai]`,
    /// "POSSIBILITA" per `[bisogna, che]`). Risolto via FractalRegistry
    /// (case-insensitive). Phase 81 `extract_proposition` legge i frattali
    /// attivi per decidere ruoli grammaticali — niente lookup di triple.
    pub function_fractal_name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RecallHaikuRequest {
    /// Frattale corrente (0..=63), o 0 se non rilevante (si peserà
    /// solo sulle ancore).
    pub fractal_id: u32,
    /// Ancore lessicali correnti (parole "calde" nel campo o nel
    /// pensiero corrente). Vuoto se si vuole recall puramente
    /// geometrica.
    pub anchors: Option<Vec<String>>,
    /// Numero massimo di cristalli da restituire (default 5).
    pub n: Option<usize>,
}

// ──────────────────────────────────────────────────────────────────────
// Server
// ──────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct PrometeoMcp {
    base_url: String,
    http: reqwest::Client,
    tool_router: ToolRouter<Self>,
}

impl PrometeoMcp {
    pub fn new(base_url: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client init");
        Self {
            base_url,
            http,
            tool_router: Self::tool_router(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    async fn get_json(&self, path: &str) -> Result<serde_json::Value, McpError> {
        let url = self.url(path);
        let resp = self.http.get(&url).send().await.map_err(|e| {
            McpError::internal_error(format!("GET {} failed: {}", url, e), None)
        })?;
        let status = resp.status();
        if !status.is_success() {
            return Err(McpError::internal_error(
                format!("GET {} returned HTTP {}", url, status),
                None,
            ));
        }
        resp.json::<serde_json::Value>().await.map_err(|e| {
            McpError::internal_error(format!("GET {} JSON parse: {}", url, e), None)
        })
    }

    async fn post_json(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let url = self.url(path);
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(format!("POST {} failed: {}", url, e), None)
            })?;
        let status = resp.status();
        if !status.is_success() {
            return Err(McpError::internal_error(
                format!("POST {} returned HTTP {}", url, status),
                None,
            ));
        }
        resp.json::<serde_json::Value>().await.map_err(|e| {
            McpError::internal_error(format!("POST {} JSON parse: {}", url, e), None)
        })
    }

    fn ok(value: serde_json::Value) -> Result<CallToolResult, McpError> {
        let content = Content::json(value).map_err(|e| {
            McpError::internal_error(format!("Content::json failed: {}", e), None)
        })?;
        Ok(CallToolResult::success(vec![content]))
    }

    // ──────────────────────────────────────────────────────────────────
    // Compattatori payload — il server REST restituisce payload molto
    // grandi (state con 64 frattali visibili, trail lungo, ecc). Per un
    // client LLM via MCP il payload completo brucia context senza
    // valore aggiunto. I compattatori riducono i campi voluminosi
    // mantenendo le informazioni significative (top-N per attivazione,
    // ultimi N per cronologia). Lavorano in-place su `serde_json::Value`.
    //
    // Schema: ogni tool che ritorna stato cumulativo applica il proprio
    // compattatore. La logica è semplice e idempotente: se i campi
    // attesi non esistono, no-op.
    // ──────────────────────────────────────────────────────────────────

    /// Trim del payload `InputResponse` di `/api/input` (e affini che
    /// includono `state`). Riduce:
    /// - `state.active_fractals` a top-12 (per attivazione descrescente,
    ///   già ordinato lato server)
    /// - `state.locus.visible` a top-12 (per visibilità descrescente)
    /// - `state.locus.trail` agli ultimi 10
    /// - rimuove `state.locus.sub_position` (debug spaziale verboso)
    fn compact_input_response(mut v: serde_json::Value) -> serde_json::Value {
        const TOP_FRACTALS: usize = 12;
        const TOP_VISIBLE: usize = 12;
        const TRAIL_LAST: usize = 10;
        if let Some(state) = v.get_mut("state").and_then(|s| s.as_object_mut()) {
            if let Some(arr) = state.get_mut("active_fractals").and_then(|a| a.as_array_mut()) {
                arr.truncate(TOP_FRACTALS);
            }
            if let Some(locus) = state.get_mut("locus").and_then(|l| l.as_object_mut()) {
                if let Some(arr) = locus.get_mut("visible").and_then(|a| a.as_array_mut()) {
                    arr.truncate(TOP_VISIBLE);
                }
                if let Some(arr) = locus.get_mut("trail").and_then(|a| a.as_array_mut()) {
                    let n = arr.len();
                    if n > TRAIL_LAST {
                        *arr = arr.split_off(n - TRAIL_LAST);
                    }
                }
                locus.remove("sub_position");
            }
        }
        v
    }

    /// Trim del payload `/api/visuals` (frattali + simplici). Riduce:
    /// - `fractals` a top-12 (per activation descrescente, ordinato lato
    ///   binario perché il server NON ordina)
    /// - `simplices` a top-30 (per intensity descrescente)
    fn compact_visuals(mut v: serde_json::Value) -> serde_json::Value {
        const TOP_FRACTALS: usize = 12;
        const TOP_SIMPLICES: usize = 30;
        if let Some(obj) = v.as_object_mut() {
            if let Some(arr) = obj.get_mut("fractals").and_then(|a| a.as_array_mut()) {
                arr.sort_by(|a, b| {
                    let av = a.get("activation").and_then(|x| x.as_f64()).unwrap_or(0.0);
                    let bv = b.get("activation").and_then(|x| x.as_f64()).unwrap_or(0.0);
                    bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
                });
                arr.truncate(TOP_FRACTALS);
            }
            if let Some(arr) = obj.get_mut("simplices").and_then(|a| a.as_array_mut()) {
                arr.sort_by(|a, b| {
                    let av = a.get("intensity").and_then(|x| x.as_f64()).unwrap_or(0.0);
                    let bv = b.get("intensity").and_then(|x| x.as_f64()).unwrap_or(0.0);
                    bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
                });
                arr.truncate(TOP_SIMPLICES);
            }
        }
        v
    }

    /// Trim del payload `/api/self` (SelfDto). Riduce:
    /// - `uncertainties` a top-15 (per tension descrescente)
    /// - `belief_influence_trace` a primi 10
    fn compact_self_profile(mut v: serde_json::Value) -> serde_json::Value {
        const TOP_UNCERTAINTIES: usize = 15;
        const TOP_TRACE: usize = 10;
        if let Some(obj) = v.as_object_mut() {
            if let Some(arr) = obj.get_mut("uncertainties").and_then(|a| a.as_array_mut()) {
                arr.sort_by(|a, b| {
                    let av = a.get("tension").and_then(|x| x.as_f64()).unwrap_or(0.0);
                    let bv = b.get("tension").and_then(|x| x.as_f64()).unwrap_or(0.0);
                    bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
                });
                arr.truncate(TOP_UNCERTAINTIES);
            }
            if let Some(arr) = obj.get_mut("belief_influence_trace").and_then(|a| a.as_array_mut()) {
                arr.truncate(TOP_TRACE);
            }
        }
        v
    }

    /// Trim del payload `/api/wordfield` (WordFieldDto). Limita le
    /// parole attive ai top-30 per attivazione.
    fn compact_field_state(mut v: serde_json::Value) -> serde_json::Value {
        const TOP_WORDS: usize = 30;
        if let Some(obj) = v.as_object_mut() {
            // Il DTO può avere campi come "words" o "active_words"; trim
            // entrambi se presenti.
            for key in &["words", "active_words", "top_active"] {
                if let Some(arr) = obj.get_mut(*key).and_then(|a| a.as_array_mut()) {
                    arr.truncate(TOP_WORDS);
                }
            }
        }
        v
    }
}

// ──────────────────────────────────────────────────────────────────────
// Tool definitions
// ──────────────────────────────────────────────────────────────────────

#[tool_router]
impl PrometeoMcp {
    /// **Turno reale di dialogo con UI-r1.** Invia l'input italiano
    /// all'engine. UI-r1 lo legge, costruisce ComprehensionReport
    /// (Phase 73), estrae SentenceProposition se disponibile (Phase 81),
    /// decide un'ActionDecision (Phase 74) e genera una risposta.
    ///
    /// Il response include la voce italiana di UI-r1 più tutti gli
    /// organi strutturati: cosa ha capito (`comprehension_report`),
    /// cosa ha deciso di fare (`action_decision`), la memoria del
    /// parlante (`speaker_profile`), la catena deliberativa
    /// (`deliberation`), la stance valenziale e l'intenzione.
    ///
    /// Lo stato di UI-r1 viene modificato: tick, NarrativeSelf,
    /// SpeakerProfile, PF1. Non è una query passiva.
    #[tool(
        description = "Send an Italian sentence to UI-r1 as a real dialogue turn. Returns its full structured comprehension: spoken response, comprehension_report (Phase 73), action_decision (Phase 74), speaker_profile (Phase 72), deliberation chain, stance and intention. Modifies live engine state."
    )]
    async fn comprehend(
        &self,
        Parameters(req): Parameters<ComprehendRequest>,
    ) -> Result<CallToolResult, McpError> {
        let body = serde_json::json!({ "text": req.input });
        let value = self.post_json("/api/input", body).await?;
        Self::ok(Self::compact_input_response(value))
    }

    /// **Modalità OSSERVATORE** — analizza un testo di TERZI (trascrizione,
    /// verbale, paragrafo), NON un turno di dialogo rivolto a UI-r1. Segmenta in
    /// frasi e comprende ciascuna in modo STATELESS e COMPATTO: nessuna cornice
    /// "io sono il destinatario" (niente addressee/self_relevance), nessuna
    /// mutazione di stato (tick/NarrativeSelf/SpeakerProfile/PF1 intatti).
    ///
    /// Per ogni frase: `speech_act`, `claim` (soggetto-relazione-oggetto-via-
    /// complementi = chi-dice-cosa-su-cosa), `anchor_concepts` (vicinato KG per
    /// il tagging tematico), `inferences` (catene 2-hop), `contradictions`.
    /// Più un `aggregate`: concetti ricorrenti, distribuzione degli atti,
    /// contraddizioni. USARE QUESTO (non `comprehend`) per analizzare riunioni/
    /// conversazioni altrui o testi lunghi.
    #[tool(
        description = "OBSERVER MODE — analyze a third-party text/transcript (read-only, stateless, compact). Splits text into sentences and comprehends each WITHOUT treating UI-r1 as the addressee (no addressee/self_relevance, no state mutation). Per sentence: speech_act, claim (subject-relation-object-via-complements = who-says-what-about-what), anchor_concepts (KG neighborhood for thematic tagging), inferences (2-hop chains), contradictions. Plus an aggregate (recurring concepts, speech-act distribution). Use THIS, not comprehend, for meetings/conversations of others or long texts."
    )]
    async fn analyze(
        &self,
        Parameters(req): Parameters<ComprehendRequest>,
    ) -> Result<CallToolResult, McpError> {
        let body = serde_json::json!({ "text": req.input });
        let value = self.post_json("/api/analyze", body).await?;
        Self::ok(value)
    }

    /// Stato vivo del **campo PF1**: parole attualmente attive con
    /// le loro attivazioni, decay rate, neighborhood. È la
    /// fotografia del "pensiero" presente di UI-r1 — quello che è
    /// vivo nel campo in questo istante.
    #[tool(
        description = "Read the live PF1 field state: which words are currently activated, their activations, neighborhood. Snapshot of UI-r1's present 'thought-field'. Read-only."
    )]
    async fn get_field_state(&self) -> Result<CallToolResult, McpError> {
        let value = self.get_json("/api/wordfield").await?;
        Self::ok(Self::compact_field_state(value))
    }

    /// **Stato narrativo dell'entità**: stance, valenza Octalysis
    /// (8 drives), coherence_integrity, intention deliberata,
    /// attrattore frattale recente, commitment volitivo, ultima
    /// stance attribuita all'Altro. È la posizione interna da cui
    /// UI-r1 sta parlando in questo momento.
    #[tool(
        description = "Read UI-r1's internal narrative state: stance, Octalysis drives, coherence_integrity, deliberated intention, recent fractal attractor, volitional commitment, attributed intent toward the Other. The position from which UI-r1 is currently speaking."
    )]
    async fn get_narrative_state(&self) -> Result<CallToolResult, McpError> {
        let value = self.get_json("/api/narrative").await?;
        Self::ok(value)
    }

    /// **Frattali I Ching attivi**: per ciascuno dei 64 attrattori
    /// regionali (8 trigrammi inferiori × 8 superiori) il punteggio
    /// di attivazione corrente. È la geometria attuale della sfera —
    /// dove UI-r1 si trova sulla superficie I Ching.
    #[tool(
        description = "Read currently active fractals (64 I Ching attractors, 8x8 grid). Returns activation score per fractal. The geometry of where UI-r1 is on the I Ching sphere right now."
    )]
    async fn get_active_fractals(&self) -> Result<CallToolResult, McpError> {
        // /api/visuals espone fractal activations + simplexes. Il compattatore
        // ordina i frattali per activation desc (il server non ordina) e
        // tronca a top-12, più top-30 simplici per intensity.
        let value = self.get_json("/api/visuals").await?;
        Self::ok(Self::compact_visuals(value))
    }

    /// **Pensieri attivi**: i diversi tipi di pensiero che l'engine
    /// ha generato (Gap, MissingBridge, Disconnection, Hypothesis,
    /// AbductiveHypothesis, SelfDiscovery, Need, Desire,
    /// Interlocutor, Humor, Tension). Mostra cosa UI-r1 sta
    /// pensando, non solo cosa sta dicendo.
    #[tool(
        description = "Read active thoughts UI-r1 is currently entertaining: Gap, MissingBridge, Hypothesis, AbductiveHypothesis, SelfDiscovery, Need, Desire, Interlocutor, Humor, Tension. Shows what UI-r1 is thinking, not just what it's saying."
    )]
    async fn get_thoughts(&self) -> Result<CallToolResult, McpError> {
        let value = self.get_json("/api/thoughts").await?;
        Self::ok(value)
    }

    /// **Profilo del Sé**: identità nucleare, model di Sé, frattali
    /// dominanti, episodi semantici recenti, valori e incertezze
    /// esplicite. La continuità identitaria di UI-r1 attraverso le
    /// sessioni.
    #[tool(
        description = "Read UI-r1's self-profile: IdentityCore (holographic), SelfModel (beliefs, values, uncertainties), dominant fractals, recent semantic episodes. The identity continuity across sessions."
    )]
    async fn get_self_profile(&self) -> Result<CallToolResult, McpError> {
        let value = self.get_json("/api/self").await?;
        Self::ok(Self::compact_self_profile(value))
    }

    /// **Vicinato KG di una parola**: triple uscenti ed entranti dal
    /// kg_sem (e dal kg_proc se applicabile), con confidence e via.
    /// Per esplorare la rete relazionale attorno a un concetto.
    #[tool(
        description = "Query the knowledge graph neighborhood of a word: outgoing and incoming triples from kg_sem (and kg_proc when applicable), with confidence and via. For exploring the relational network around a concept."
    )]
    async fn query_kg(
        &self,
        Parameters(req): Parameters<QueryKgRequest>,
    ) -> Result<CallToolResult, McpError> {
        let _limit = req.limit.unwrap_or(30); // endpoint REST non accetta limit oggi; future-proof
        let path = format!(
            "/api/word_neighbors?word={}",
            urlencoding::encode_or_simple(&req.word)
        );
        let value = self.get_json(&path).await?;
        Self::ok(value)
    }

    /// **Dettaglio di una parola**: firma 8D (Agency, Permanenza,
    /// Intensita, Tempo, Confine, Complessita, Definizione, Valenza —
    /// ordine I Ching canonico Phase 68), stability, exposure, POS,
    /// frattali con cui ha affinità. Per esibire la geometria di un
    /// concetto.
    #[tool(
        description = "Read full word details: 8D signature (I Ching canonical order: Agency, Permanenza, Intensita, Tempo, Confine, Complessita, Definizione, Valenza), stability, exposure, POS tags, fractal affinities. Exhibits the geometry of a single concept."
    )]
    async fn get_word_detail(
        &self,
        Parameters(req): Parameters<WordRequest>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!(
            "/api/word_detail?word={}",
            urlencoding::encode_or_simple(&req.word)
        );
        let value = self.get_json(&path).await?;
        Self::ok(value)
    }

    /// **Concetto associato a una parola**: ancestori IS_A nel KG,
    /// discendenti campione, relazioni che caratterizzano il concetto.
    /// Per posizionare la parola nella tassonomia semantica.
    #[tool(
        description = "Read the conceptual context of a word: IS_A ancestors in the KG, sample descendants, characterizing relations. Places the word in the semantic taxonomy."
    )]
    async fn get_concept(
        &self,
        Parameters(req): Parameters<WordRequest>,
    ) -> Result<CallToolResult, McpError> {
        let path = format!(
            "/api/concept?word={}",
            urlencoding::encode_or_simple(&req.word)
        );
        let value = self.get_json(&path).await?;
        Self::ok(value)
    }

    // ──────────────────────────────────────────────────────────────────
    // Phase 82 / FOND4 — Memoria sferica di haiku (cerchi sulla sfera)
    // ──────────────────────────────────────────────────────────────────

    /// **Deposita un cristallo** (haiku) nella memoria-sfera di UI-r1.
    /// Tre versi 5-7-5 (chiarezza > poesia), frattale dominante I Ching
    /// al deposito, ancore lessicali (parole-perno emergenti). Le
    /// tangenze con altri cristalli si calcolano automaticamente:
    /// due cristalli sono tangenti se condividono ≥2 ancore OPPURE
    /// se condividono uno dei due trigrammi I Ching.
    ///
    /// Il deposito è PERSISTENTE su `haiku_memory.json`. Sopravvive
    /// alle sessioni. È il modo in cui un LLM può lasciare cristalli
    /// di pensiero nella memoria di UI-r1, e ritrovarli per prossimità
    /// geometrica nelle sessioni future.
    #[tool(
        description = "Deposit a haiku-crystal into UI-r1's spherical memory. Three verses (5-7-5 syllables — clarity over poetry), a dominant I Ching fractal (0-63), anchor words (2-6). Tangencies with other crystals compute automatically: ≥2 shared anchors OR shared lower/upper trigram. PERSISTENT across sessions. This is how an LLM leaves traces in UI-r1's memory and finds them again by geometric proximity in future sessions."
    )]
    async fn deposit_haiku(
        &self,
        Parameters(req): Parameters<DepositHaikuRequest>,
    ) -> Result<CallToolResult, McpError> {
        let body = serde_json::json!({
            "verses": req.verses,
            "fractal_id": req.fractal_id,
            "anchors": req.anchors,
            "source": req.source.unwrap_or_else(|| "claude".to_string()),
            "note": req.note,
        });
        let value = self.post_json("/api/haiku/deposit", body).await?;
        Self::ok(value)
    }

    /// **Recall geometrico** dalla memoria-sfera: dati un frattale
    /// corrente e/o ancore correnti, restituisce i cristalli più
    /// prossimi sulla sfera. Il punteggio combina distanza frattale e
    /// ancore condivise, con le ancore lessicali che dominano
    /// (β=5.0) sulla geometria di sfondo (α=1.0).
    #[tool(
        description = "Recall crystals from UI-r1's spherical memory by proximity. Given a current fractal (0-63) and/or current anchor words, returns the geometrically closest crystals. Anchor words dominate (β=5.0) over background fractal geometry (α=1.0): 2 shared anchors beat identical fractal."
    )]
    async fn recall_haiku_near(
        &self,
        Parameters(req): Parameters<RecallHaikuRequest>,
    ) -> Result<CallToolResult, McpError> {
        let body = serde_json::json!({
            "fractal_id": req.fractal_id,
            "anchors": req.anchors.unwrap_or_default(),
            "n": req.n.unwrap_or(5),
        });
        let value = self.post_json("/api/haiku/recall", body).await?;
        Self::ok(value)
    }

    /// **Statistiche sulla sfera**: totale cristalli, frattali più
    /// popolati (top-8), ancore più ricorrenti (top-12), densità
    /// tangenziale media. Per ispezionare la forma della memoria.
    #[tool(
        description = "Read statistics about UI-r1's haiku-sphere: total crystals, top-8 most populated fractals, top-12 most recurrent anchor words, average tangency density. Inspects the shape of the memory."
    )]
    async fn get_haiku_stats(&self) -> Result<CallToolResult, McpError> {
        let value = self.get_json("/api/haiku/stats").await?;
        Self::ok(value)
    }

    // ──────────────────────────────────────────────────────────────────
    // Phase 83 — Educazione grammaticale live (simplessi tipizzati)
    // ──────────────────────────────────────────────────────────────────

    /// **Insegna un simplesso grammaticale.** Sequenza di parole
    /// (ordered) che, quando appare adiacente nell'input, attiva un
    /// frattale-funzione nel campo (es. RELAZIONE, SALUTO, POSSIBILITA).
    /// Non è template: i token restano separati nel parser; è la
    /// geometria del campo che cambia quando il simplesso emerge,
    /// e Phase 81 legge i frattali attivi per decidere ruoli.
    ///
    /// Esempi:
    /// - `["rispetto","a"]` + `RELAZIONE` → "rispetto a X" → asse-relativo
    /// - `["come","stai"]` + `SALUTO` → atto fatico, non identificazione
    /// - `["bisogna","che"]` + `POSSIBILITA` → costrutto modale
    /// - `["ho","mangiato"]` + `DIVENIRE` → tempo composto passato
    ///
    /// PERSISTENTE: salvato nel `.bin` al prossimo save. Vedi Phase 83 in CLAUDE.md.
    #[tool(
        description = "Teach UI-r1 a grammatical simplex: an ordered word sequence that, when it appears adjacent in input, activates a function fractal in the field (RELATION, GREETING, POSSIBILITY, etc.). Not a template lookup — Phase 81 parser reads active fractals from the field to decide grammatical roles. Examples: [rispetto,a]+RELAZIONE for prepositional 'in relation to', [come,stai]+SALUTO for phatic greeting, [bisogna,che]+POSSIBILITA for modal. PERSISTENT across sessions."
    )]
    async fn add_grammar_simplex(
        &self,
        Parameters(req): Parameters<AddGrammarSimplexRequest>,
    ) -> Result<CallToolResult, McpError> {
        let body = serde_json::json!({
            "words": req.words,
            "category": req.category,
            "function_fractal_name": req.function_fractal_name,
        });
        let value = self.post_json("/api/grammar_simplex", body).await?;
        Self::ok(value)
    }
}

// ──────────────────────────────────────────────────────────────────────
// ServerHandler — get_info espone la natura del server al client MCP
// ──────────────────────────────────────────────────────────────────────

#[tool_handler]
impl ServerHandler for PrometeoMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "UI-r1 cognitive substrate (Phase 82). Each tool reads from \
                 a live Engine via HTTP. `comprehend` is a REAL dialogue \
                 turn that modifies engine state (tick, NarrativeSelf, \
                 SpeakerProfile, PF1). All other tools are read-only. \
                 Use `get_field_state`/`get_narrative_state`/`get_thoughts` \
                 to read UI-r1's live cognitive state before responding \
                 from its position rather than from your statistical \
                 distribution alone.",
            )
    }
}

// ──────────────────────────────────────────────────────────────────────
// URL-encoding minimo per parole italiane con accenti / spazi
// (evita di tirare giù una crate solo per questo)
// ──────────────────────────────────────────────────────────────────────

mod urlencoding {
    /// URL-encode percent-style for a path segment. Ammette ASCII
    /// alfanumerici e i caratteri non-riservati `-._~`. Tutto il resto
    /// viene percent-encoded byte-per-byte UTF-8. Coerente con RFC 3986.
    pub fn encode_or_simple(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for b in s.as_bytes() {
            let c = *b;
            let is_unreserved = c.is_ascii_alphanumeric()
                || c == b'-'
                || c == b'.'
                || c == b'_'
                || c == b'~';
            if is_unreserved {
                out.push(c as char);
            } else {
                out.push('%');
                out.push_str(&format!("{:02X}", c));
            }
        }
        out
    }
}

// ──────────────────────────────────────────────────────────────────────
// Entrypoint
// ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Default port 3000 (matching prometeo-web's banner). Override with
    // PROMETEO_WEB_URL if the server is bound elsewhere.
    let base_url = std::env::var("PROMETEO_WEB_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    let log_enabled = std::env::var("PROMETEO_MCP_LOG").as_deref() == Ok("1");
    if log_enabled {
        eprintln!("[prometeo-mcp] starting; base_url = {}", base_url);
    }

    let server = PrometeoMcp::new(base_url);
    let service = server.serve(stdio()).await?;
    if log_enabled {
        eprintln!("[prometeo-mcp] handshake complete; waiting on stdio…");
    }
    service.waiting().await?;
    Ok(())
}
