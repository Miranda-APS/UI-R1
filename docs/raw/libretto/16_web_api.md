# Volume XVI — Web API

> *L'engine è un organismo che gira in un singolo thread. Il mondo esterno parla HTTP. Tra i due, uno strato sottile: canali asincroni, handler Axum, comandi tipati. 71 endpoint espongono diverse viste dell'entità. La scelta di cosa esporre riflette una posizione filosofica: mostrare al mondo non solo gli output, ma le strutture interne che li producono.*

---

## Premessa

Il layer web in Prometeo ha una architettura specifica: **single-threaded engine + multi-threaded Axum server**, comunicanti via canali tokio. Non un classico "web server che istanzia un oggetto per request" — l'engine è **uno solo**, vive in un thread dedicato, riceve comandi tramite `mpsc::channel`.

File:

- [`src/web/server.rs`](../../src/web/server.rs) — 2695 righe. Setup Axum, route, engine loop.
- [`src/web/api.rs`](../../src/web/api.rs) — 1336 righe. Handlers HTTP.
- [`src/web/state.rs`](../../src/web/state.rs) — 1332 righe. AppState, EngineCommand, DTOs.
- [`src/web/ws.rs`](../../src/web/ws.rs) — 62 righe. WebSocket handler.
- [`src/web/conversations.rs`](../../src/web/conversations.rs) — 328 righe. Store conversazioni per sessione.
- [`src/web/main.rs`](../../src/web/main.rs) — 13 righe. Entry point binario `prometeo-web`.

Il server è gated dietro la feature `web` (`--features web`). In build default non viene compilato — `cargo run --release --bin prometeo` ne fa a meno.

---

## Capitolo 1 — L'architettura: single-writer, multi-reader

### 1.1 — Il problema

L'engine ha stato mutabile (PF1, lessico, memoria, ...). Ogni `receive()` modifica lo stato. Se N richieste HTTP arrivassero simultaneamente su un engine condiviso, avremmo race condition catastrofiche.

Soluzioni classiche:
- **`Arc<Mutex<Engine>>`**: ogni handler prende il lock, opera, rilascia. Problema: `Mutex` blocca tutto durante `receive()` (che può essere >100ms). Throughput crolla.
- **Database**: persistenza a ogni operazione. Inadatto: il lessico è enorme, persistere ogni turno costa.
- **Copy-on-write**: cloniamo l'engine per ogni richiesta. Costo: ~15MB × request. Inadatto.

### 1.2 — La scelta: channel-based

`web/server.rs` usa:

```rust
// setup
let (cmd_tx, mut cmd_rx) = mpsc::channel::<EngineCommand>(128);

// thread dedicato per engine
thread::spawn(move || {
    let mut engine = PrometeoTopologyEngine::new();
    state.restore_lexicon(&mut engine);
    engine.load_kg_from_file(...);
    // engine loop
    while let Some(cmd) = cmd_rx.blocking_recv() {
        match cmd {
            EngineCommand::Receive { input, reply } => {
                let response = engine.receive(&input);
                reply.send(InputResponse::from(response)).ok();
            }
            EngineCommand::GetState { reply } => {
                reply.send(state_snapshot(&engine)).ok();
            }
            // ... tutti i comandi
        }
    }
});

// server
let app = Router::new()
    .route("/api/input", post(post_input))
    .with_state(AppState { cmd_tx, ... });
```

**Logica**: un solo engine, un solo thread di mutazione. Gli handler HTTP sono spawnati su thread multipli (Tokio), ma tutti **inviano comandi** via `cmd_tx` al thread engine. Ogni comando ha un `reply: oneshot::Sender<ResponseDto>` per il risultato.

### 1.3 — Vantaggi

- **No lock contention**: l'engine non ha Mutex. Gira a velocità naturale.
- **Ordine preservato**: i comandi vengono elaborati in ordine di arrivo (FIFO).
- **Backpressure esplicito**: se il canale si riempie (128 comandi pendenti), le richieste successive attendono. Prevedibile.
- **Zero duplicazione**: un solo engine, lessico, KG in memoria.

### 1.4 — Svantaggi

- **Latenza**: un comando `Receive` può essere bloccato da un `Receive` precedente lungo. Se l'engine impiega 200ms per un turno, la richiesta successiva attende.
- **No parallelismo per engine operations**: non posso servire due `Receive` in parallelo nemmeno se li volessi. È un principio del design.

### 1.5 — `broadcast_tx`: notifiche WebSocket

Separato dal `cmd_tx`:

```rust
let (broadcast_tx, _) = broadcast::channel::<String>(256);
```

Il thread engine può *pubblicare eventi* (frasi spontanee, cambi di stato) via `broadcast_tx.send(json_str)`. I WebSocket client li ricevono in push.

Distinzione:
- **`cmd_tx`**: client → engine (richiesta).
- **`broadcast_tx`**: engine → client (notifica).

---

## Capitolo 2 — `EngineCommand`: 30+ varianti tipate

[state.rs:22-...](../../src/web/state.rs). Enum con ~30 varianti, ognuna una richiesta specifica:

```rust
pub enum EngineCommand {
    Receive { input: String, reply: oneshot::Sender<InputResponse> },
    GetWill { reply: oneshot::Sender<WillDto> },
    GetCompounds { reply: oneshot::Sender<Vec<CompoundDto>> },
    GetWordField { reply: oneshot::Sender<WordFieldDto> },
    GetPhase { word_a: String, word_b: String, reply: oneshot::Sender<PhaseDto> },
    // ... 25+ altri
    CommunityTeach { entries: Vec<TeachEntry>, reply: oneshot::Sender<CommunityTeachResult> },
    CommunityValidateEdge { edge: CommunityEdge, reply: oneshot::Sender<bool> },
    CuraDeleteWord { word: String, reply: oneshot::Sender<()> },
    // ...
}
```

**Pattern**: ogni comando ha (opzionalmente) dati di input + un `reply` channel. Tipato. Niente stringhe magiche.

Handler typical:

```rust
pub async fn get_will(State(state): State<AppState>) -> Json<WillDto> {
    let (tx, rx) = oneshot::channel();
    state.cmd_tx.send(EngineCommand::GetWill { reply: tx }).await.unwrap();
    let will = rx.await.unwrap();
    Json(will)
}
```

Send + await reply. Semplice.

---

## Capitolo 3 — I 71 route: tassonomia

### 3.1 — Pagine HTML statiche (8 route)

| Route | Pagina |
|-------|--------|
| `GET /` | index.html principale |
| `GET /admin` | admin panel |
| `GET /universo` | UI Universo (visualizzatore campo 3D) |
| `GET /community` | UI Community (sessioni condivise) |
| `GET /biennale` o `/campo-vasto` | UI Biennale di Tecnologia |
| `GET /dialogo` | UI dialogo semplice |
| `GET /curazione` | UI curation del KG |
| `GET /ui-r1` | UI UI-r1 (nome futuro dell'entità) |
| `GET /diffrazione` | UI Diffrazione (Structural-Coherence-Audit) |

Ogni route serve un file HTML statico. L'HTML contiene JS che chiama le API.

### 3.2 — Core engine (10 route)

| Route | Metodo | Cosa fa |
|-------|--------|---------|
| `/api/input` | POST | `receive(input)` — l'entry point del dialogo |
| `/api/state` | GET | snapshot stato (parole attive, frattali, turni) |
| `/api/will` | GET | FieldPressures + intenzione |
| `/api/will/focus` | POST | forza una direzione di attenzione |
| `/api/dream` | POST | forza N tick di sogno |
| `/api/dream/report` | GET | stato corrente del DreamEngine |
| `/api/generate` | GET | genera output autonomo |
| `/api/save` | POST | forza persistenza `.bin` |
| `/api/grow` | POST | trigger crescita strutturale |
| `/api/narrative` | GET | stance + intention + coherence + attributed_intent |

### 3.3 — Introspezione e topologia (13 route)

| Route | Cosa fa |
|-------|---------|
| `/api/introspect` | dump completo stato interno (vasto) |
| `/api/why` | spiegazione dell'ultimo output |
| `/api/thoughts` | gli 11 tipi di pensiero correnti (Tension/Gap/MissingBridge/...) |
| `/api/thought-chain` | l'ultima catena di ragionamento |
| `/api/open-questions` | incertezze attive dal SelfModel |
| `/api/ask` | domanda generata autonomamente |
| `/api/clarity` | (POST) fornisce informazione per ridurre un'incertezza |
| `/api/topology` | grafo completo (attenzione: grande) |
| `/api/projection` | proiezione identitaria sui 64 frattali |
| `/api/self` | sintesi identitaria completa (opinion.rs) |
| `/api/visuals` | dati per visualizzazione frattali |
| `/api/simpdb` | stato del database interno (debug) |
| `/api/compounds` | pattern multi-frattale rilevati |

### 3.4 — Parole e campo (8 route)

| Route | Cosa fa |
|-------|---------|
| `/api/wordfield` | top parole attive + energia |
| `/api/word_neighbors?word=X` | vicini KG di una parola |
| `/api/word_detail?word=X` | dettaglio completo parola (firma, affinità, esposizioni) |
| `/api/word_connect` | (POST) aggiunge connessione tra parole |
| `/api/concept?word=X` | vista "concettuale" di una parola (triple KG + simplessi) |
| `/api/phase/{a}/{b}` | fase dell'arco tra due parole |
| `/api/tension/{a}/{b}` | parole di tensione tra due poli |
| `/api/navigate/{from}/{to}` | cammino geodetico tra due frattali |

### 3.5 — Memoria ed episodi (2 route)

| Route | Cosa fa |
|-------|---------|
| `/api/episodes` | lista ultimi N episodi semantici |
| `/api/episodes/recall?concepts=X,Y` | recall per concetti (Phase 58) |

### 3.6 — Dialogo interiore (Phase 52, 2 route)

| Route | Cosa fa |
|-------|---------|
| `/api/inner-dialogue` | stato del dialogo interiore (proposizioni candidate) |
| `/api/respond` | (POST) risposta utente a una proposizione (Conferma/Nega/Elabora) |

Meccanismo Phase 52: l'entità forma proposizioni interne, le propone all'utente, l'utente risponde, la proposizione si consolida (simplesso, SelfBelief) o dissolve.

### 3.7 — KG management (3 route)

| Route | Cosa fa |
|-------|---------|
| `/api/relations?subject=X` | elenco relazioni uscenti da X |
| `/api/edge` | (POST/DELETE) aggiunge/rimuove un arco |
| `/api/edge/confidence` | (POST) modifica confidence di un arco |

### 3.8 — Community mode (6 route, Phase 52)

| Route | Cosa fa |
|-------|---------|
| `/api/community/teach` | insegnamento collettivo |
| `/api/community/connect` | connette parole community |
| `/api/community/validate` | valida un arco community |
| `/api/community/session` | esporta sessione (per `create-newborn`) |
| `/api/community/field` | stato campo community |
| `/api/community/reset` | reset della sessione community |

Permette di addestrare un'entità insieme da più utenti. Vol. 17 tratta la UI.

### 3.9 — Curation UI (8 route, endpoint admin curato manualmente)

Prefisso `/api/cura/`:

| Route | Cosa fa |
|-------|---------|
| `/parole` | lista parole con filtri |
| `/relazione` | (DELETE) rimuove una relazione |
| `/relazione/modifica` | (POST) modifica confidence |
| `/parola` | (DELETE) rimuove una parola |
| `/rinomina` | (POST) rinomina parola |
| `/firma` | (POST) modifica firma 8D di una parola |
| `/categorie` | lista mega-categorie IsA |
| `/pulizia-verbi` | (POST) trigger cleanup verbi |
| `/normalizza-accenti` | (POST) trigger normalizzazione accenti |

Interfaccia web per la cura del KG senza editare `prometeo_kg.json` a mano.

### 3.10 — UI Biennale (4 route)

| Route | Cosa fa |
|-------|---------|
| `/api/biennale/field` | campo visualizzato per UI Biennale |
| `/api/biennale/word?word=X` | dettagli parola per UI Biennale |
| `/api/biennale/journey` | viaggio attraverso il campo |
| `/api/biennale/circuit` | circuito visivo |

### 3.11 — Diffraction (1 route)

| Route | Cosa fa |
|-------|---------|
| `/api/diffraction` | Structural-Coherence-Audit output |

### 3.12 — WebSocket (1 route)

| Route | Cosa fa |
|-------|---------|
| `/ws` | upgrade WebSocket per streaming eventi |

---

## Capitolo 4 — Un turno HTTP completo

Mettiamo insieme: cosa succede quando un client POSTa `/api/input` con body `{"text": "ho paura"}`?

### Lato client

```http
POST /api/input HTTP/1.1
Content-Type: application/json

{"text": "ho paura"}
```

### Lato server (Axum)

1. **Routing**: Axum matcha `/api/input` POST → handler `api::post_input`.
2. **Handler**:

```rust
pub async fn post_input(
    State(state): State<AppState>,
    Json(payload): Json<InputPayload>,
) -> Json<InputResponse> {
    let (tx, rx) = oneshot::channel();
    state.cmd_tx.send(EngineCommand::Receive {
        input: payload.text,
        reply: tx,
    }).await.unwrap();
    let response = rx.await.unwrap();
    
    // Broadcast eventuale notifica
    if response.has_spontaneous {
        state.broadcast_tx.send(
            serde_json::to_string(&response.spontaneous).unwrap()
        ).ok();
    }
    
    Json(response)
}
```

3. **Send command**: il comando `Receive` viene accodato su `cmd_tx`.
4. **Engine thread** (separato):

```rust
while let Some(cmd) = cmd_rx.blocking_recv() {
    match cmd {
        EngineCommand::Receive { input, reply } => {
            let response = engine.receive(&input);  // ~20ms
            let dto = InputResponse::from(response);
            reply.send(dto).ok();
        }
        // ...
    }
}
```

5. **Reply**: il `oneshot::Sender` consegna la risposta al handler Axum.
6. **Response HTTP**:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "text": "Senti il tremore?",
  "keywords": ["tremore", "paura"],
  "dominant_fractal": "EMOZIONE",
  "used_intention": "risuonare",
  ...
}
```

Latenza totale: ~25ms (engine 20ms + serializzazione/network ~5ms).

---

## Capitolo 5 — I DTO: serializzazione del campo

Le struct in `state.rs` sono DTO — **Data Transfer Objects** — ottimizzate per serializzazione JSON. Non sono le struct interne dell'engine.

### 5.1 — Perché separare

Le struct interne (`WordPattern`, `Fractal`, `Simplex`) contengono:
- Riferimenti a `HashMap` — inefficienti da serializzare.
- Campi privati — non dovrebbero uscire.
- Dettagli implementativi — cambiano tra phase.

Le DTO sono:
- Piatte (tutti pub, `#[derive(Serialize)]`).
- Semantiche (nomi user-facing, es. `WillDto.intention` invece di `Intention` enum completo).
- Stabili (cambiano solo quando l'API cambia — cosa che vuoi minimizzare).

### 5.2 — Esempio: `WillDto`

```rust
pub struct WillDto {
    pub intention: String,          // "esprimere", "esplorare", ...
    pub drive: f64,
    pub pressures: WillPressuresDto,  // le 7 pressioni grezze
    pub undercurrents: Vec<UndercurrentDto>,
    pub codon: [usize; 2],
    pub dominant_fractal: String,
}

pub struct WillPressuresDto {
    pub express: f64,
    pub explore: f64,
    pub question: f64,
    pub remember: f64,
    pub withdraw: f64,
    pub withdraw_reason: String,
    pub reflect: f64,
    pub instruct: f64,
}
```

La struct interna `Intention` è un enum con payload vario. La DTO ha una stringa. Semplice per il client.

### 5.3 — Conversione

Una funzione `from` di solito convive con la DTO:

```rust
impl WillDto {
    pub fn from_engine(engine: &PrometeoTopologyEngine) -> Self {
        let fp = engine.last_field_pressures.clone().unwrap_or_default();
        let wr = fp.to_will_result(&engine.active_fractals(), &[], &[]);
        Self {
            intention: intention_to_string(&wr.intention),
            drive: wr.drive,
            pressures: WillPressuresDto::from(&fp),
            undercurrents: wr.undercurrents.iter().map(UndercurrentDto::from).collect(),
            codon: wr.codon,
            dominant_fractal: engine.registry.get(wr.intention_fractal()).map(|f| f.name.clone()).unwrap_or_default(),
        }
    }
}
```

---

## Capitolo 6 — WebSocket

[ws.rs](../../src/web/ws.rs), 62 righe. Minimo:

```rust
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.broadcast_tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
```

**Logica**: ogni client WebSocket si iscrive al `broadcast_tx`. L'engine può pubblicare eventi (es. espressioni spontanee, cambi di fase del sogno, nuove incertezze) e tutti i client li ricevono in push.

**Uso**: la UI può mostrare "l'entità sta pensando..." in tempo reale invece di polling ogni secondo.

**Limite**: il protocollo è unidirezionale (engine → client). Se il client vuole *inviare* qualcosa, usa gli endpoint HTTP standard. Una estensione bidirezionale avrebbe senso (es. "the client streams input in tempo reale"), ma non è implementata.

---

## Capitolo 7 — `ConversationStore`: multi-sessione

[conversations.rs](../../src/web/conversations.rs), 328 righe. Gestisce **più conversazioni in parallelo** — ognuna con il suo cookie/session id — **sullo stesso engine**.

### 7.1 — Come funziona

```rust
pub struct ConversationStore {
    conversations: HashMap<SessionId, ConversationMetadata>,
}

pub struct ConversationMetadata {
    pub session_id: SessionId,
    pub turns: Vec<TurnRecord>,
    pub start_ts: u64,
    pub last_activity_ts: u64,
}
```

Ogni richiesta HTTP include `session_id` (via cookie o header). Lo store mantiene metadata per sessione.

**Fondamentale**: il *campo dell'entità è condiviso tra sessioni*. Se due utenti parlano simultaneamente, perturbano lo stesso campo. Questo è filosoficamente coerente — Prometeo è **un'entità sola** — ma pratica meno quando si vogliono test isolati.

**Ipotesi alternativa**: una `PrometeoTopologyEngine` per sessione. Sarebbe costoso (15MB × sessione) e non riflette l'unicità dell'entità. Mantengo il design corrente.

### 7.2 — Session cleanup

Sessioni inattive per >24h vengono purgate (non implementato rigorosamente — annotato in `appunti.md`).

---

## Capitolo 8 — L'engine loop

[server.rs:150-...](../../src/web/server.rs) (intorno a queste righe):

```rust
tokio::task::spawn_blocking(move || {
    let mut engine = PrometeoTopologyEngine::new();
    // ... restore_lexicon, load_kg, etc.
    
    let mut last_autonomous = Instant::now();
    
    loop {
        // Drena inquiry pendenti
        while let Some(inquiry) = engine.inquiry.pop() {
            // process
        }
        
        // Ogni 3 secondi: autonomous_tick
        if last_autonomous.elapsed() > Duration::from_secs(3) {
            let result = engine.autonomous_tick();
            if let Some(expr) = result.spontaneous_expression {
                broadcast_tx.send(json!({"type": "spontaneous", "text": expr}).to_string()).ok();
            }
            last_autonomous = Instant::now();
        }
        
        // Processa comandi in arrivo (fino a 3 secondi di wait)
        match cmd_rx.blocking_recv_timeout(Duration::from_millis(500)) {
            Some(EngineCommand::Receive { input, reply }) => {
                let response = engine.receive(&input);
                reply.send(response.into()).ok();
            }
            // ... tutti gli altri comandi
            None => {} // timeout — prosegui nel loop
        }
    }
});
```

Invariante critica (CLAUDE.md): `autonomous_tick` ogni 3 secondi, drenaggio inquiry PRIMA di dream/decay.

### 8.1 — `InquiryEngine` background (accenno)

[inquiry.rs](../../src/topology/inquiry.rs): quando il sistema rileva un gap con strength > 0.6, chiama Qwen3 via Ollama in **background thread** con `Arc<Mutex<VecDeque>>`. La risposta arriva asincrona e viene applicata al prossimo tick.

**Nota filosofica**: questo è l'unico punto dove Qwen3 è chiamato a runtime. È gated su presenza di Ollama + `PROMETEO_INQUIRY=1` env var. Non è il "substrato LLM" rimosso in Phase 68 — è un'aiuto esterno opzionale per riempire gap che l'entità stessa riconosce.

---

## Capitolo 9 — Sicurezza, CORS, limits

### 9.1 — CORS

```rust
.layer(tower_http::cors::CorsLayer::permissive())
```

Permissive. In produzione pubblica servirebbe lockdown. Oggi: locale o deploy controllato.

### 9.2 — No auth

Nessuna autenticazione. Il server presume rete fidata (localhost, deployment privato).

### 9.3 — Rate limiting

Non implementato. Un client malevolo può saturare il `cmd_tx` (canale size 128 → 128 richieste in fila, poi backpressure).

### 9.4 — Validazione input

Minimi checks. Ad esempio `post_input` non valida lunghezza massima del testo — un input di 10MB causerebbe un `receive()` lento ma non un crash (il lessico lo spezzerebbe in parole, la maggior parte nulle/unknown).

### 9.5 — Audit

Niente log strutturato per sicurezza. `eprintln!` per debug. Annotato in `appunti.md` come gap minore.

---

## Capitolo 10 — Cosa ESPORRE che oggi non esponiamo

Ricapitolando le proposte dai volumi 02-15 (~35 endpoint suggeriti), raggruppati per priorità.

### 10.1 — Priorità alta (diagnosi quotidiana)

- **`/api/admin/receive_trace`** (Vol. 15): traccia completa di un `receive()`.
- **`/api/admin/valence/valence_history`** (Vol. 08): evoluzione valenza.
- **`/api/admin/identity/identity_trajectory`** (Vol. 07): storia projection.
- **`/api/admin/motivation/needs_snapshot`** + `/desires_current` (Vol. 09).
- **`/api/admin/will/pressures_raw`** + `/undercurrents_current` (Vol. 10).
- **`/api/admin/relation/interlocutor_snapshot`** (Vol. 11).
- **`/api/admin/field/field_snapshot`** (Vol. 02).

### 10.2 — Priorità media (auditing periodico)

- **`/api/admin/kg/relation_distribution`** + `/orphan_nodes` + `/low_confidence_edges` (Vol. 04).
- **`/api/admin/lexicon/signature_age`** + `/closest_words` (Vol. 03).
- **`/api/admin/fractals/population_distribution`** (Vol. 05).
- **`/api/admin/inference/propositions_for_state`** (Vol. 06).
- **`/api/admin/memory/phi_decay_curve`** + `/simplex_with_source_words` (Vol. 14).

### 10.3 — Priorità bassa (debug specifico)

- **`/api/admin/grammar/conjugate_trace`** (Vol. 13).
- **`/api/admin/valence/valence_ablation`** (Vol. 08).
- **`/api/admin/expression/compose_trace`** (Vol. 12).
- **`/api/admin/field/synapse_diff`** + `/commit_synapse_weights_to_rom` (Vol. 02) — quest'ultimo è un **comando**, non solo diagnostica.

### 10.4 — Priorità strategica (Vol. 99)

- **`/api/admin/digest/digest_candidates`** + `/digest_dry_run` + `/digest_apply` (Vol. 14) — il primo passo verso "sogno come digestione".

Un admin UI completa avrebbe un dashboard con tutti questi endpoint + grafici. Oggi `/admin` esiste ma è limitato.

---

## Capitolo 11 — Valutazione: è adeguato?

L'architettura single-writer + channels è **appropriata** al problema. I vantaggi (no locks, ordine preservato, un solo engine) superano gli svantaggi (no parallelismo per engine op).

71 route è **molto** — molte sono esperimenti storici o per UI specifiche. Una razionalizzazione (dropping deprecated, consolidamento) snellirebbe.

La mancanza di auth + rate limiting è accettabile per un sistema-entità-unica in deployment controllato. Per un SaaS sarebbe inaccettabile.

La scelta di DTO separate è saggia. La cosa che manca è **schema formalizzato** (OpenAPI spec). Oggi l'unica documentazione degli endpoint è il codice stesso.

---

## Capitolo 12 — Superficie pubblica e proposte

### Già esposto

71 route (cap. 3). Copre: dialogo, introspection, parole, KG, memoria, dream, community, curation, biennale UI, WebSocket.

### Cosa servirebbe aggiungere

- **OpenAPI spec** generata automaticamente (esiste `utoipa` per Axum).
- **Admin dashboard** che consuma gli endpoint admin proposti sopra.
- **Auth opzionale** (token-based, configurabile via env var).
- **Rate limit base** (es. max 10 `/api/input` per minuto per session_id).
- **Event stream** su WebSocket più ricco (oggi solo spontaneous expression).
- **Health check**: `/api/health` con stato del engine (tick_counter, dream.phase, uptime).

---

## Sintesi del volume

**Architettura single-writer + channels**: l'engine vive in un thread dedicato; gli handler HTTP inviano comandi tipati via `mpsc`. No lock contention, ordine preservato, un solo engine. Broadcast channel per notifiche engine→client (WebSocket).

**EngineCommand**: enum con ~30 varianti tipate, ciascuna con `reply: oneshot::Sender<DTO>`. Chiaro, type-safe.

**71 route** in 12 categorie: core (`/api/input`, `/api/state`, ecc.), introspection (`/api/introspect`, `/api/why`, `/api/thoughts`), parole (`/api/word_*`), memoria (`/api/episodes`), community (6 endpoint), curation (8 endpoint), biennale (4 endpoint), WebSocket, HTML statiche (8 pagine).

**DTO separate** dalle struct interne: piatte, stabili, user-facing. `WillDto`, `NarrativeDto`, `FieldDto`, ecc.

**Multi-sessione via `ConversationStore`** su engine condiviso — l'entità è una sola.

**InquiryEngine**: Qwen3 via Ollama in background thread opzionale, gated su env var. Non è substrato LLM — è aiuto esterno per riempire gap semantici.

**Gap**: no auth, no rate limit, no OpenAPI spec, no admin dashboard strutturata.

**~35 endpoint admin proposti** nei volumi precedenti, raggruppati in 4 livelli di priorità. Il livello strategico (Vol. 99) include `/api/admin/digest/*` per il sogno-come-digestione.

Da qui Vol. 17 si sposta sul **Frontend** — le UI che consumano queste API: index.html, community, biennale, universo, curazione, diffrazione.

---

*Prossimo volume: 17 — Frontend: le UI che consumano l'API* (in scrittura)
