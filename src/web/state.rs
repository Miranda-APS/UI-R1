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
    /// Phase 82 — Memoria-sfera di haiku. Vive nel web layer (NON dentro
    /// Engine) perché è un organo nuovo, persistente su file separato
    /// `haiku_memory.json`, ispezionabile/curabile indipendentemente.
    /// Gli handler axum accedono diretto via Mutex; ogni `deposit`
    /// sincronizza il salvataggio su disco.
    pub haiku_memory: std::sync::Arc<std::sync::Mutex<crate::topology::haiku_memory::HaikuMemory>>,
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
    /// Export di tutte le firme 8D del lessico (per cura-mobile).
    GetAllFirme {
        reply: oneshot::Sender<Vec<(String, [f64; 8])>>,
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
    /// Phase 86 — UI-r1 prova a collocare una parola DA SOLA: cammino multi-hop
    /// tipato verso un'ancora fondata (vista Stato interno, auto-chiarimento).
    ExploreWord {
        word: String,
        reply: oneshot::Sender<ExploreDto>,
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
    /// Phase 84: l'utente comunica come avrebbe voluto sentire la risposta.
    /// Il sistema estrae parole-contenuto, applica triple specializzate o
    /// modula confidence, registra il fatto nello SpeakerProfile.
    Correct {
        input: String,
        given: String,
        wanted: String,
        context: Option<String>,
        reply: oneshot::Sender<CorrectDto>,
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
    /// Rinomina una parola: merge KG + lessico, rimuove la vecchia
    RinominaWord {
        from: String,
        to: String,
        reply: oneshot::Sender<bool>,
    },
    /// Lista parole del lessico (con paginazione e ricerca)
    GetWordList {
        query: String,
        offset: usize,
        limit: usize,
        sort: String,
        only_kg: bool,
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
    /// Normalizza accenti: unifica coppie accentata/non-accentata
    NormalizzaAccenti {
        dry_run: bool,
        reply: oneshot::Sender<NormalizzaDto>,
    },
    /// Tutto ciò che il sistema sa di un concetto via InferenceEngine
    GetConcept {
        word: String,
        reply: oneshot::Sender<ConceptDto>,
    },
    /// P1 (Tsunami): comprensione STATELESS di un testo isolato. Non muta
    /// l'engine (no tick, no NarrativeSelf, no SpeakerProfile, no PF1): è una
    /// lettura pura del KG. Per analizzare titoli-task isolati o ogni analisi
    /// puntuale senza contaminazione fra chiamate (al contrario di /api/input,
    /// che accumula SpeakerProfile/closure fra i turni).
    Comprehend {
        text: String,
        reply: oneshot::Sender<ComprehendDto>,
    },
    /// Stato completo del SelfModel (credenze, valori, incertezze)
    GetSelf {
        reply: oneshot::Sender<SelfDto>,
    },
    /// P2 (Tsunami): il ritratto-utente cumulativo (SpeakerProfile) — read-only.
    /// La memoria del parlante persistita cross-sessione: chi è, cosa ha detto,
    /// cosa resta aperto, dove ha corretto. Esposizione ricca per il companion.
    GetSpeakerProfile {
        reply: oneshot::Sender<SpeakerProfileDto>,
    },
    /// P2 (Tsunami): forza la persistenza dello stato vissuto nel `.bin`
    /// (formato che il loader RILEGGE al boot — `/api/save` scrive solo il JSON
    /// legacy, ignorato in presenza del .bin). Include SpeakerProfile, narrativa,
    /// identità, simplessi, lessico. L'app lo chiama sui lifecycle event
    /// (onPause/onStop), NON per turno: il .bin è grosso (decine di MB).
    /// NON riscrive il KG (immutato fuori dalla curation).
    Persist {
        reply: oneshot::Sender<bool>,
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
    /// Trasmissione BATCH dal campo nuovo: insegna molte parole + impone
    /// firma + aggiunge molti archi in UN solo comando engine. Esegue
    /// SOLO ALLA FINE: recompute_all_word_affinities, build_semantic_simplices,
    /// cura_save. Riduce ~10s a <1s per trasmissioni medie.
    TransmitBatch {
        words: Vec<TransmitWordItem>,
        edges: Vec<TransmitEdgeItem>,
        user_id: String,
        user_name: String,
        reply: oneshot::Sender<TransmitBatchDto>,
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
    /// Circuito di attivazione tra due parole (BFS pesato 2-hop da entrambe)
    GetBiennaleCircuit {
        w1: String,
        w2: String,
        reply: oneshot::Sender<BiennaleCircuitDto>,
    },

    /// Comprensione multi-facet di una frase tramite il KG (read-only).
    /// Non muta lo stato dell'engine: lemmatizza via grammar::lemmatize e
    /// costruisce SceneUnderstanding + catene inferenziali 2-hop.
    GetUnderstanding {
        sentence: String,
        reply: oneshot::Sender<SceneUnderstandingDto>,
    },
    /// Dati completi per costruire un campo medio da una frase: lemmi,
    /// firme 8D, TUTTI gli archi KG con firme dei target. Read-only.
    GetMedioData {
        sentence: String,
        reply: oneshot::Sender<MedioDataDto>,
    },
    /// Conferma un arco proposto dall'inferenza: aggiunge al KG + file audit.
    ConfirmEdge {
        subject: String,
        relation: String,
        object: String,
        confidence: f32,
        reply: oneshot::Sender<ConfirmEdgeResultDto>,
    },
    /// Rifiuta un arco proposto: non viene più suggerito in questa sessione.
    RejectEdge {
        subject: String,
        relation: String,
        object: String,
        reply: oneshot::Sender<ConfirmEdgeResultDto>,
    },

    // ─── Phase 69 — osservazione del tempo proprio ─────────────────

    /// Statistiche aggregate della vita interiore: emessi/debounced/dimenticati,
    /// materiale in digestione, ricordi accumulati, finestra di riflessività.
    GetEventsStats {
        reply: oneshot::Sender<EventsStatsDto>,
    },
    /// Contenuto attuale della coda di digestione (cap 32).
    /// Render leggibile degli eventi in attesa.
    GetPendingDigestion {
        reply: oneshot::Sender<PendingDigestionDto>,
    },
    /// Ultimi N ricordi semantici accumulati (default 10).
    /// Ciascuno con sintesi, concetti chiave, stato emotivo al momento.
    GetRecentEpisodes {
        limit: usize,
        reply: oneshot::Sender<RecentEpisodesDto>,
    },
    /// Phase 83 — Insegna un simplesso grammaticale tipizzato.
    /// Aggiunge al complesso un simplesso con `source_words` curato,
    /// `category` (es. "preposizione_composta") e `function_fractal` (id 0-63
    /// risolto lato server dal nome). Quando la sequenza di words appare
    /// nell'input in ordine adiacente, il simplesso si attiva e semina il
    /// frattale-funzione nel campo. Persistente nel `.bin`.
    AddGrammarSimplex {
        words: Vec<String>,
        category: String,
        function_fractal_name: String,
        reply: oneshot::Sender<Result<AddGrammarSimplexResponse, String>>,
    },

    // === IAm-gotchi (glass-box) — Step 5: correzione del modello-dell'Altro ===
    /// L'utente corregge l'intento attribuito all'Altro (+ valenza opzionale).
    /// Nudgia gli EMA dell'interlocutor nel quadrante target di attribute_intent,
    /// poi persiste via cura_save. Vedi InterlocutorModel::apply_intent_correction.
    CorrectInterlocutor {
        intent: String,
        emotional_valence: Option<f64>,
        reply: oneshot::Sender<bool>,
    },
    // === fine IAm-gotchi ===
}

#[derive(Serialize, Clone, Debug)]
pub struct AddGrammarSimplexResponse {
    pub simplex_id: u64,
    pub function_fractal_id: u32,
    pub function_fractal_name: String,
    pub category: String,
    pub words: Vec<String>,
    pub total_grammar_simplices: usize,
}

// ═══════════════════════════════════════════════════════════════
// DTO: engine → JSON → frontend
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Clone, Debug)]
pub struct HypothesisDto {
    /// Concetto-perno sotto-definito
    pub concept: String,
    /// Quante parole dell'input lo richiamano
    pub saliency: u32,
    /// Quanti archi definitori (IsA/Has/Does/PartOf) ha il concetto
    pub defining_arcs: u32,
    /// Tipo di relazione dominante con cui è stato invocato (es. "REQUIRES")
    pub dominant_invocation: Option<String>,
    /// Parole dell'input che l'hanno evocato
    pub invoked_by: Vec<String>,
}

/// Archi di una parola raggruppati per relazione.
/// Forma nativa KG, senza framing narrativo.
#[derive(Serialize, Clone, Debug)]
pub struct WordUnderstandingDto {
    pub word: String,
    /// Numero totale di archi uscenti letti
    pub outgoing_count: usize,
    /// Numero totale di archi entranti letti
    pub incoming_count: usize,
    /// Archi uscenti raggruppati per tipo di relazione
    pub outgoing: Vec<WordRelationGroupDto>,
    /// Archi entranti (chi punta a questa parola)
    pub incoming: Vec<WordRelationGroupDto>,
}

#[derive(Serialize, Clone, Debug)]
pub struct WordRelationGroupDto {
    /// Chiave tipo relazione in maiuscolo (es. "IS_A", "CAUSES")
    pub relation: String,
    /// Etichetta italiana breve (es. "è", "causa", "richiede")
    pub label: String,
    /// Bersagli/soggetti della relazione, ordinati per confidence decrescente
    pub targets: Vec<RelationTargetDto>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RelationTargetDto {
    pub word: String,
    pub confidence: f32,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct SceneUnderstandingDto {
    /// Ruolo sintattico: "Statement" | "Question" | "Exclamation"
    pub syntactic_role: String,
    /// Lemmi riconosciuti dall'input
    pub lemmas: Vec<String>,
    /// Parole dell'input senza archi nel KG
    pub unknown_words: Vec<String>,
    /// Profondità di comprensione (archi totali letti dal KG)
    pub comprehension_depth: usize,
    /// Sunto in prosa: cosa la frase significa secondo il KG
    pub summary: String,
    /// Ipotesi di nuove relazioni logiche non ancora nel KG (confermabili)
    pub proposed_edges: Vec<ProposedEdgeDto>,
    /// Comprensione per parola dell'input — ogni lemma con tutte le sue relazioni
    pub words: Vec<WordUnderstandingDto>,
    /// Ipotesi aperte (concetti-perno sotto-definiti)
    pub open_hypotheses: Vec<HypothesisDto>,
    /// Cammini inferenziali 2-hop nel grafo partendo da parole input.
    /// Es: "sole → calore → energia" (produce, è).
    pub inferential_chains: Vec<InferentialChainDto>,
    /// Struttura sintattica ordinata: archi tra parole vicine collegate da
    /// preposizione/copula con ipotesi tipizzate validate sul KG.
    /// È il "ragionamento di comprensione" che la UI può mostrare in tempo reale.
    pub syntactic_edges: Vec<SyntacticEdgeDto>,
    /// Grafo di esplorazione transitiva del KG: nodi raggiunti, archi
    /// traversati, convergenze, sillogismi. Renderizzato come SVG nella
    /// chat admin per mostrare il "ragionamento" di UI-r1.
    pub graph: Option<ComprehensionGraphDto>,
}

/// DTO del grafo di comprensione transitiva. Tutti i nomi sono lemmi
/// normalizzati (lowercase). Ordinamento delle liste = importanza decrescente.
#[derive(Serialize, Clone, Debug)]
pub struct ComprehensionGraphDto {
    /// Lemmi input che fanno da radice (ordine = ordine di apparizione).
    pub roots: Vec<String>,
    pub nodes: Vec<ConceptNodeDto>,
    pub edges: Vec<TraversedEdgeDto>,
    pub convergences: Vec<ConvergenceDto>,
    pub syllogisms: Vec<SyllogismDto>,
    /// Atto reciproco riconosciuto + scelta di UI-r1, se applicabile.
    /// Es. input "ciao" → act_type "saluto", chosen "salve".
    /// None se l'input non è un'istanza fatica di un atto comunicativo.
    pub reciprocal_act: Option<ReciprocalActDto>,
}

/// Atto comunicativo reciproco riconosciuto: la classe (es. "saluto"),
/// la parola input come istanza, i fratelli candidati, la scelta finale.
#[derive(Serialize, Clone, Debug)]
pub struct ReciprocalActDto {
    pub act_type: String,
    pub root: String,
    pub siblings: Vec<String>,
    /// La parola che UI-r1 ha scelto come risposta (None se nessun
    /// candidato era nel lessico). È la stessa parola che appare
    /// come testo generato.
    pub chosen: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ConceptNodeDto {
    pub word: String,
    pub depth: u8,
    pub support: f32,
    pub is_root: bool,
    /// Le radici da cui questo nodo è raggiungibile.
    pub root_witnesses: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct TraversedEdgeDto {
    pub from: String,
    pub to: String,
    /// Tipo relazione in maiuscolo (es. "IS_A").
    pub relation: String,
    /// Etichetta italiana breve (es. "è un").
    pub relation_label: String,
    pub confidence: f32,
    /// Profondità del nodo `from` (0 = radice, 1 = raggiunto da radice...).
    pub depth: u8,
}

#[derive(Serialize, Clone, Debug)]
pub struct ConvergenceDto {
    pub concept: String,
    pub witnesses: Vec<String>,
    pub strength: f32,
}

#[derive(Serialize, Clone, Debug)]
pub struct SyllogismDto {
    pub subject: String,
    pub middle: String,
    pub object: String,
    pub r1: String,
    pub r1_label: String,
    pub r2: String,
    pub r2_label: String,
    /// Relazione composta derivata (es. "CAUSES" da IsA∘Causes). None = composizione non significativa.
    pub composed: Option<String>,
    pub composed_label: Option<String>,
    pub strength: f32,
    /// Forma testuale: "cane è un animale e animale causa movimento ⇒ cane causa movimento"
    pub summary: String,
}

/// Un singolo arco S–connettore–O nell'input, con ipotesi tipizzate ordinate.
#[derive(Serialize, Clone, Debug)]
pub struct SyntacticEdgeDto {
    pub subject: String,
    pub object: String,
    /// "Preposition", "Copula", "Verb"
    pub connector_kind: String,
    /// Per Preposition: "di"/"a"/"da"/...; per Copula: "essere"; per Verb: lemma.
    pub connector_form: String,
    /// Indici delle due parole nel raw input (ordine).
    pub subject_pos: usize,
    pub object_pos: usize,
    /// Ipotesi tipizzate ordinate. La prima validata vince.
    pub hypotheses: Vec<RelationHypothesisDto>,
    /// Indice (in `hypotheses`) della relazione validata (None se nessuna).
    pub validated_idx: Option<usize>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RelationHypothesisDto {
    /// Tipo relazione (es. "HAS", "IS_A").
    pub relation: String,
    /// Etichetta italiana breve (es. "ha", "è").
    pub relation_label: String,
    /// Esito validazione: "diretto" | "tipo" | "2-hop" | "contraddetto" | "nel campo aperto".
    pub validation_kind: String,
    /// Confidenza del KG quando l'esito è una conferma (DirectEdge/TypeCompatible/TwoHop).
    pub confidence: Option<f32>,
    /// Per TypeCompatible: il tipo che ha confermato (es. "persona").
    pub via_type: Option<String>,
    /// Per TwoHop: il nodo intermedio.
    pub intermediate: Option<String>,
    /// Spiegazione umana: perché questa ipotesi nasce e cosa il KG dice.
    pub rationale: String,
}

/// Un arco proposto dall'inferenza: non esiste nel KG ma segue logicamente
/// da una catena esistente. Il gruppo può confermare (→ aggiunto al KG) o
/// rifiutare (→ non riproposto in questa sessione).
#[derive(Serialize, Clone, Debug)]
pub struct ProposedEdgeDto {
    /// Identificatore stabile per conferma/rifiuto (hash della triple)
    pub id: String,
    pub subject: String,
    pub relation: String,
    /// Etichetta italiana breve (es. "è", "produce")
    pub relation_label: String,
    pub object: String,
    /// Confidence inferita (conf1 × conf2 × decay)
    pub confidence: f32,
    /// Spiegazione umana: perché questa relazione è plausibile
    pub rationale: String,
    /// Stato: "pending" | "confirmed" | "rejected"
    pub status: String,
}

/// Risultato di ConfirmEdge/RejectEdge.
#[derive(Serialize, Clone, Debug, Default)]
pub struct ConfirmEdgeResultDto {
    pub ok: bool,
    pub message: String,
    pub edge_count: usize,
}

/// Dati completi per costruire un campo medio da una frase.
/// Per ogni lemma input: firma 8D + TUTTI gli archi del KG con firme dei target.
#[derive(Serialize, Clone, Debug, Default)]
pub struct MedioDataDto {
    /// Parola → informazioni complete
    pub words: Vec<MedioWordDto>,
    /// Parole dell'input non presenti nel KG
    pub unknown: Vec<String>,
    /// Lemmi riconosciuti dall'input (ordine di apparizione)
    pub lemmas: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct MedioWordDto {
    pub word: String,
    /// Firma 8D della parola (None se non nel lessico).
    /// Nel JSON valori f64 in [0,1].
    pub signature: Option<[f64; 8]>,
    /// Tutti gli archi uscenti (senza cap)
    pub outgoing: Vec<MedioEdgeDto>,
}

#[derive(Serialize, Clone, Debug)]
pub struct MedioEdgeDto {
    pub relation: String,
    pub target: String,
    pub confidence: f32,
    /// Firma 8D del target (None se non nel lessico).
    pub target_signature: Option<[f64; 8]>,
    /// "out" (default): l'arco va da `lemma` verso `target`.
    /// "in": l'arco va da `target` verso `lemma` (relazione entrante).
    /// Aggiunto per parole come "vita" che hanno relazioni solo come oggetto.
    pub direction: String,
}

/// Un cammino 2-hop nel grafo partendo da una parola input.
/// Forma: "word → target1 → target2" con le due relazioni.
#[derive(Serialize, Clone, Debug)]
pub struct InferentialChainDto {
    /// Parola di partenza (lemma input)
    pub origin: String,
    /// Primo passo: relazione + target
    pub first_relation: String,
    pub first_label: String,
    pub first_target: String,
    /// Secondo passo: relazione + target
    pub second_relation: String,
    pub second_label: String,
    pub second_target: String,
    /// Confidence combinata
    pub combined_confidence: f32,
    /// Rendering leggibile: "sole produce calore · calore è energia"
    pub derived_inference: String,
}

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
    /// Comprensione multi-facet dell'input: attribuzioni al parlante
    /// (da Requires/Causes/UsedFor) e ipotesi aperte (concetti-perno
    /// sotto-definiti). None se il KG è vuoto o l'input è vuoto.
    pub understanding: Option<SceneUnderstandingDto>,
    /// Phase 71: catena soggettiva del turno (percezione → interrogazione →
    /// comprensione → desiderio → coerenza → azione). Espone le decisioni
    /// che hanno guidato la generazione.
    pub deliberation: Option<DeliberationDto>,
    /// Phase 72: cosa UI-r1 ha capito DEL PARLANTE in questa sessione.
    /// Accumula self_facts, entity_facts, domande aperte, concetti
    /// menzionati, gap di conoscenza. È la narrativa del dialogo.
    pub speaker_profile: Option<SpeakerProfileDto>,
    /// Phase 73: il documento di comprensione che UI-r1 ha scritto
    /// per questo turno. Letto dalla rete simbolica del KG, articolato
    /// in atto di parola, posizioni, vuoti, inferenze, pertinenza per sé.
    pub comprehension_report: Option<ComprehensionReportDto>,
    /// Phase 74: la decisione di azione esplicita. Cosa UI-r1 ha scelto
    /// di fare in risposta, perché, con quale forma e quali ancore.
    pub action_decision: Option<ActionDecisionDto>,
    /// Phase 81: la frase letta come triple strutturale
    /// (`subject + relation + object + via + polarity`) — la PROP, l'unità
    /// di lettura dell'utterance come sotto-grafo del kg_sem.
    pub sentence_proposition: Option<SentencePropositionDto>,
    /// Phase 81: il confronto fra la PROP e il kg_sem. Dice se la triple
    /// esiste già nel kg_sem (matches), se i suoi slot hanno radici nel KG
    /// (object_in_kg / via_in_kg), e quali contraddizioni `OppositeOf`
    /// emergono. È l'ancoraggio strutturale della frase al mondo.
    pub kg_confrontation: Option<KgConfrontationDto>,
    /// Phase 86+: il BISOGNO che l'input ha aperto nel campo — il segnale
    /// PRINCIPALE per Tsunami. L'app lo mappa su una feature/azione (NON su una
    /// chat): `strutturare`→ordina il dump, `capire`→la domanda che sblocca,
    /// `co-regolare`→calma la UI, `esternalizzare-memoria`→riemergi un fatto, ecc.
    pub need: Option<NeedDto>,
    /// Phase 86+ (multi-locus): TUTTE le proposizioni dell'enunciato, una per
    /// clausola. Per i dump ("devo X e comprare Y e non ho finito Z") l'app le
    /// mostra come item separati. La primaria è marcata `is_primary`.
    pub propositions: Vec<ClausePropositionDto>,
}

/// Il bisogno dominante + la classifica (per ispezione/diagnostica).
#[derive(Serialize, Clone, Debug)]
pub struct NeedDto {
    pub dominant: String,
    pub intensity: f64,
    pub ranked: Vec<NeedRankDto>,
    /// Il *perché*: i segnali di campo che hanno prodotto questo bisogno. Rende
    /// il bisogno esplorabile invece che un'etichetta opaca.
    pub signals: NeedSignalsDto,
}

#[derive(Serialize, Clone, Debug)]
pub struct NeedRankDto {
    pub need: String,
    pub intensity: f64,
}

/// I segnali grezzi che alimentano la lettura del bisogno (need.rs::NeedSignals).
/// Ogni campo è una quantità di campo, non una soglia: il bisogno dominante è
/// `argmax` delle intensità che questi segnali generano.
#[derive(Serialize, Clone, Debug)]
pub struct NeedSignalsDto {
    /// Nodi-contenuto che non raggiungono ancora un'ancora.
    pub ungrounded_count: usize,
    /// Totale nodi-contenuto (per normalizzare l'ungrounded).
    pub content_count: usize,
    /// Esiste un vuoto dialogico PROP-driven (slot non saturo)?
    pub has_dialogic_gap: bool,
    /// Un gap aperto in un turno precedente si è chiuso ORA?
    pub closes_prior_gap: bool,
    /// Confronto col mondo: 0=conferma/n.a., ~0.6=novità, 1.0=contraddizione.
    pub world_confront: f64,
    /// Il mondo CONFERMA il claim (la triple esiste già nel kg_sem).
    pub world_confirm: f64,
    /// Salienza della grana del sé toccata dalla frase.
    pub self_salience: f64,
    /// Sovraccarico (tipicamente 1 - coherence_integrity).
    pub overload: f64,
    /// Rilevanza di un fatto del parlante che riaffiora.
    pub memory_resurfaced: f64,
    /// Assenza percepita al ritorno.
    pub absence: f64,
    /// Numero di loci/componenti sconnesse (1 = monolocus).
    pub locus_count: usize,
}

/// Una proposizione ancorata alla sua clausola (multi-locus).
#[derive(Serialize, Clone, Debug)]
pub struct ClausePropositionDto {
    pub proposition: Option<SentencePropositionDto>,
    pub subordinate: bool,
    pub is_primary: bool,
}

pub fn need_to_dto(n: &crate::topology::need::NeedReading) -> NeedDto {
    let s = &n.signals;
    NeedDto {
        dominant: n.dominant.as_str().to_string(),
        intensity: n.intensity,
        ranked: n.ranked.iter()
            .filter(|(_, i)| *i > 0.0)
            .map(|(nd, i)| NeedRankDto { need: nd.as_str().to_string(), intensity: *i })
            .collect(),
        signals: NeedSignalsDto {
            ungrounded_count: s.ungrounded_count,
            content_count: s.content_count,
            has_dialogic_gap: s.has_dialogic_gap,
            closes_prior_gap: s.closes_prior_gap,
            world_confront: s.world_confront,
            world_confirm: s.world_confirm,
            self_salience: s.self_salience,
            overload: s.overload,
            memory_resurfaced: s.memory_resurfaced,
            absence: s.absence,
            locus_count: s.locus_count,
        },
    }
}

pub fn clause_props_to_dto(
    props: &[crate::topology::sentence_proposition::ClauseProposition],
) -> Vec<ClausePropositionDto> {
    let primary = crate::topology::sentence_proposition::primary_index(props);
    props.iter().enumerate().map(|(i, c)| ClausePropositionDto {
        proposition: c.prop.as_ref().map(sentence_proposition_to_dto),
        subordinate: c.subordinate,
        is_primary: Some(i) == primary,
    }).collect()
}

// NB: `coverage` + `saturation` (Gate di Comprensione) sono esposti da
// `/api/comprehend`. Doc: docs/raw/architettura/gate_di_comprensione.md.
//
/// P1 (Tsunami): risultato della comprensione STATELESS di un testo isolato.
/// Stessi DTO della comprensione di `/api/input`, ma ottenuti SENZA mutare lo
/// stato dell'engine. L'app lo usa per il task_type del Mental Inbox e per ogni
/// analisi puntuale; i campi turn-relazionali (closure di un turno precedente)
/// sono per costruzione assenti (un testo isolato non ha un "prima").
#[derive(Serialize, Clone, Debug, Default)]
pub struct ComprehendDto {
    /// Il testo analizzato (eco, per correlazione lato app).
    pub text: String,
    /// Lemmi/forme-base normalizzate via KG (mai infiniti inventati).
    pub lemmas: Vec<String>,
    /// TUTTE le proposizioni dell'enunciato, una per clausola (multi-locus).
    /// La primaria è marcata `is_primary`.
    pub propositions: Vec<ClausePropositionDto>,
    /// La proposizione PRIMARIA, isolata (comodità: evita di cercare il flag).
    pub primary: Option<SentencePropositionDto>,
    /// Ancoraggio della primaria al kg_sem (object/via nel KG, contraddizioni).
    pub kg_confrontation: Option<KgConfrontationDto>,
    /// Il bisogno che l'enunciato apre, calcolato dai soli segnali derivabili
    /// staticamente (grounding, confronto col mondo, salienza del sé, gap
    /// dialogico, multi-locus). I segnali multi-turno (closure, memoria,
    /// assenza, sovraccarico) sono 0 per definizione in modalità stateless.
    pub need: Option<NeedDto>,
    /// Comprensione per-parola dal KG (IS_A, relazioni, catene inferenziali):
    /// la materia per arricchire il task_type.
    pub understanding: SceneUnderstandingDto,
    /// COPERTURA per-token (Gate di Comprensione, 2026-06-15): ogni parola
    /// dell'input ha uno stato (C1 — nessun punto cieco). Vedi
    /// `docs/raw/architettura/gate_di_comprensione.md`.
    pub coverage: Vec<TokenCoverageDto>,
    /// VERDETTO di saturazione (C1–C4): quanto la comprensione è "satura".
    pub saturation: SaturationDto,
}

/// Copertura di un singolo token: lo stato di comprensione di OGNI parola
/// dell'input (C1, nessun punto cieco). I DUE ASSI: `known` (conoscenza
/// generale, "sa") vs `bound` (comprensione in QUESTA frase, "ha compreso qui").
#[derive(Serialize, Clone, Debug)]
pub struct TokenCoverageDto {
    /// Forma di superficie (così come scritta).
    pub token: String,
    /// Forma-base normalizzata via KG.
    pub lemma: String,
    /// "compreso" (✅) | "parziale" (🟡) | "ignoto" (🔴).
    pub status: String,
    /// Ruolo strutturale nella frase (soggetto/verbo/oggetto/specificazione/
    /// la classe grammaticale per le parole funzionali, o "—").
    pub role: String,
    /// Perché questo stato (per giallo/rosso: cosa manca).
    pub reason: String,
    /// Asse CONOSCENZA: la parola è nel grafo / lessico stabile.
    pub known: bool,
    /// Profondità della conoscenza generale (numero di archi KG).
    pub arc_count: usize,
    /// Asse COMPRENSIONE: il token è legato alla proposizione di QUESTA frase
    /// (ruolo determinato), non solo conosciuto in astratto.
    pub bound: bool,
}

/// Verdetto di saturazione (C1–C4): rapporto di copertura, MAI una soglia
/// inventata. "piena" = tutto legato e nessuno slot aperto; "parziale" = legato
/// ma con gap/ignoti; "non-comprensibile" = nessun ancoraggio strutturale.
#[derive(Serialize, Clone, Debug, Default)]
pub struct SaturationDto {
    pub verdict: String,
    pub total: usize,
    pub compreso: usize,
    pub parziale: usize,
    pub ignoto: usize,
    /// Slot della proposizione non saturi (i gap dichiarati — C2/C4).
    pub open_slots: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ActionDecisionDto {
    pub kind: String,
    pub shape: String,
    pub narrative_subject: String,
    pub target_kind: String,
    pub target_detail: String,
    pub anchor_words: Vec<String>,
    pub reasoning: Vec<String>,
    pub text: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ComprehensionReportDto {
    pub utterance: String,
    pub speech_act: SpeechActDto,
    pub symbolic_positions: Vec<SignifierPositionDto>,
    pub gaps: Vec<SignifierGapDto>,
    pub inferences: Vec<InferenceDto>,
    pub self_relevance: Vec<String>,
    /// Rendering completo del report come testo italiano (multi-riga).
    pub text: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct SpeechActDto {
    pub kind: String,
    pub subject: String,
    pub description: String,
    pub addressee: String,
    pub implicit_expectation: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct SignifierPositionDto {
    pub signifier: String,
    pub opposes: Vec<String>,
    pub serves_in: Vec<String>,
    pub points_to: Vec<(String, String)>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SignifierGapDto {
    pub missing: String,
    pub from: String,
    pub relation: String,
    /// Contesto semantico (parola singola). Es. context="emozione" quando
    /// missing="oggetto" e from è un'istanza di emozione.
    pub context: Option<String>,
    pub description: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct InferenceDto {
    pub chain: Vec<String>,
    pub relations: Vec<String>,
    pub conclusion: String,
    pub strength: f32,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct SpeakerProfileDto {
    pub turn_count: usize,
    /// Phase 73: nome del parlante se si è presentato.
    pub name: Option<String>,
    pub self_facts: Vec<SpokenFactDto>,
    pub entity_facts: Vec<SpokenFactDto>,
    pub open_questions: Vec<OpenQuestionDto>,
    /// Concetti menzionati ordinati per conteggio (fino a 30).
    pub top_mentioned: Vec<(String, u32)>,
    /// Gap di conoscenza ancora aperti.
    pub open_gaps: Vec<KnowledgeGapDto>,
    /// Gap che sono stati chiusi (per visualizzare la narrativa che si compone).
    pub closed_gaps: Vec<KnowledgeGapDto>,
    /// P2 (Tsunami): correzioni ricevute dal parlante — traccia narrativa di
    /// "qui mi hai corretto" (materia per il rilevatore-pattern-utente lato app).
    pub corrections: Vec<CorrectionFactDto>,
}

#[derive(Serialize, Clone, Debug)]
pub struct CorrectionFactDto {
    pub turn: usize,
    pub input: String,
    pub given: String,
    pub wanted: String,
    pub via_context: Option<String>,
    pub positive_words: Vec<String>,
    pub negative_words: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct SpokenFactDto {
    pub kind: String,
    pub predicate: String,
    pub turn: usize,
    pub raw_input: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct OpenQuestionDto {
    pub topic: Vec<String>,
    pub interrogative: Option<String>,
    pub raw_input: String,
    pub turn: usize,
    pub resolved: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct KnowledgeGapDto {
    pub question: String,
    pub trigger: String,
    pub gap_kind: String,
    pub turn: usize,
    /// La parola che ha colmato il vuoto, se chiuso (es. "buio" per una paura).
    pub closed_by: Option<String>,
    /// Turno in cui il vuoto è stato colmato.
    pub closed_at_turn: Option<usize>,
}

// ──────────────────────────────────────────────────────────────────────
// Phase 81 — La frase come proposizione + confronto col kg_sem.
// ──────────────────────────────────────────────────────────────────────

/// La frase letta come triple strutturale. Riflette `SentenceProposition`
/// in `src/topology/sentence_proposition.rs`. Il `subject` e l'`object`
/// sono spezzati in `kind` + `name` per essere consumabili facilmente
/// lato client (JS/LLM) senza bisogno di deserializzare enum tagged.
///
/// Mappature:
/// - `subject_kind`: "Speaker" | "Entity" | "World" | "Variable"
/// - `subject_name`: nome della parola del mondo o della variabile
///                   interrogativa; vuoto per Speaker/Entity.
/// - `object_kind`:  "Word" | "Variable" | "" (vuoto se l'object è None,
///                   es. "ciao" non ha oggetto strutturale).
/// - `object_name`:  predicato/variabile; vuoto se object_kind=="".
/// - `relation`:     nome canonico della `RelationType` (IsA, Has, Causes,
///                   FeelsAs, Does, Expresses, OppositeOf, SimilarTo,
///                   PartOf, UsedFor, …).
/// - `via`:          parola dopo preposizione di specificazione
///                   ("ho paura **del futuro**" → via=Some("futuro")).
///                   None se l'object non è ancora ancorato al mondo.
/// - `polarity`:     false se l'utterance contiene "non" prima del verbo.
#[derive(Serialize, Clone, Debug)]
pub struct SentencePropositionDto {
    pub subject_kind: String,
    pub subject_name: String,
    pub relation: String,
    pub object_kind: String,
    pub object_name: String,
    pub via: Option<String>,
    /// Lemma del verbo di superficie (es. "uccidere") quando la relazione è
    /// realizzata da un verbo lessicale; `None` per le copule. La `relation`
    /// resta il tipo, questo è il verbo concreto compreso.
    pub verb_lemma: Option<String>,
    pub polarity: bool,
    /// Soggetto di superficie recuperato dal pro-drop ("vogliamo"→"noi",
    /// "devo"→"io"): il soggetto celato reso esplicito. `None` per i soggetti
    /// del mondo (in `subject_name`) o le domande.
    pub subject_surface: Option<String>,
}

/// Confronto fra la PROP e il kg_sem. Vedi
/// `src/topology/sentence_proposition.rs::KgConfrontation`.
#[derive(Serialize, Clone, Debug, Default)]
pub struct KgConfrontationDto {
    /// La triple esiste già nel kg_sem (solo per subject=World).
    pub matches: bool,
    /// L'object ha almeno un arco IsA/Has/Causes/SimilarTo/OppositeOf/
    /// PartOf/Does/UsedFor nel kg_sem.
    pub object_in_kg: bool,
    /// Idem per via.
    pub via_in_kg: bool,
    /// Coppie (a, b) tali che `a OppositeOf b` esiste nel kg_sem e
    /// rendono la proposizione strutturalmente in tensione.
    pub contradictions: Vec<(String, String)>,
}

pub fn sentence_proposition_to_dto(
    prop: &crate::topology::sentence_proposition::SentenceProposition,
) -> SentencePropositionDto {
    use crate::topology::sentence_proposition::{ObjectRef, SubjectRef};
    let (subject_kind, subject_name) = match &prop.subject {
        SubjectRef::Speaker => ("Speaker".to_string(), String::new()),
        SubjectRef::Entity => ("Entity".to_string(), String::new()),
        SubjectRef::World(w) => ("World".to_string(), w.clone()),
        SubjectRef::Variable(v) => ("Variable".to_string(), v.clone()),
    };
    let (object_kind, object_name) = match &prop.object {
        Some(ObjectRef::Word(w)) => ("Word".to_string(), w.clone()),
        Some(ObjectRef::Variable(v)) => ("Variable".to_string(), v.clone()),
        None => (String::new(), String::new()),
    };
    SentencePropositionDto {
        subject_kind,
        subject_name,
        relation: format!("{:?}", prop.relation),
        object_kind,
        object_name,
        via: prop.via.clone(),
        verb_lemma: prop.verb_lemma.clone(),
        polarity: prop.polarity,
        subject_surface: prop.subject_surface.clone(),
    }
}

pub fn kg_confrontation_to_dto(
    conf: &crate::topology::sentence_proposition::KgConfrontation,
) -> KgConfrontationDto {
    KgConfrontationDto {
        matches: conf.matches,
        object_in_kg: conf.object_in_kg,
        via_in_kg: conf.via_in_kg,
        contradictions: conf.contradictions.clone(),
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct DeliberationDto {
    pub action_shape: String,
    pub dominant_drive: String,
    pub coherence_integrity: f64,
    pub turns_in_session: usize,
    pub other_presence: f64,
    pub other_emotional_valence: f64,
    pub other_attributed_intent: String,
    pub narrative_mode: String,
    pub narrative_coherence: f64,
    pub active_desire: Option<String>,
    pub inquiries: Vec<InquiryDto>,
    pub reasoning: Vec<String>,
    /// Fatti strutturali letti dal KG sull'input (sostituisce InputAct).
    pub kg_facts: KgFactsDto,
    /// Cosa UI-r1 ricorda del parlante al momento della deliberazione.
    pub speaker_context: SpeakerContextDto,
}

#[derive(Serialize, Clone, Debug)]
pub struct SpeakerContextDto {
    pub turns_observed: usize,
    pub last_self_fact: Option<String>,
    pub last_entity_fact: Option<String>,
    pub open_questions: Vec<String>,
    pub open_gaps: Vec<String>,
    pub top_concepts: Vec<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct KgFactsDto {
    pub roots: Vec<String>,
    pub root_classes: Vec<String>,
    pub specific_class: Option<String>,
    pub class_siblings_count: usize,
    pub has_question_marker: bool,
    pub has_interrogative_pronoun: bool,
    pub has_speaker_claim: bool,
    pub speaker_claim_label: Option<String>,
    pub speaker_claim_predicate: Option<String>,
    pub content_word_count: usize,
    pub emotional_proximity: f64,
    pub self_referenced: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct InquiryDto {
    pub label: String,
    pub question: String,
    pub answer: Option<String>,
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
    /// Archi del KG semantico — per le stat dinamiche della home (Tier 1.5).
    pub kg_edge_count: usize,
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

/// Phase 86 — il cammino multi-hop con cui UI-r1 prova a collocare una parola
/// (auto-chiarimento, vista Stato interno). `ground` = come si è fondata
/// ("attrattore"/"sé"/"non raggiunta"); `reached` = ha trovato un'ancora.
#[derive(Serialize, Clone, Debug, Default)]
pub struct ExploreDto {
    pub from: String,
    pub ground: String,
    pub reached: bool,
    pub steps: Vec<ExploreStepDto>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ExploreStepDto {
    /// Relazione tipata (forma Debug: "IsA", "Causes", …).
    pub relation: String,
    /// true = arco percorso in avanti (from→to); false = a ritroso (to←from).
    pub forward: bool,
    pub via: Option<String>,
    pub to: String,
    pub confidence: f32,
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

// === IAm-gotchi (glass-box) — Step 5 ===
#[derive(Deserialize)]
pub struct CorrectInterlocutorBody {
    /// Intento corretto: "Seeking" | "Teaching" | "Challenging" | "Connecting".
    pub intent: String,
    /// Valenza emotiva dell'Altro [-1, +1], opzionale.
    pub emotional_valence: Option<f64>,
}
// === fine IAm-gotchi ===

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

#[derive(Serialize)]
pub struct NormalizzaDto {
    /// Coppie (non_accentata, accentata) trovate
    pub pairs: Vec<[String; 2]>,
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

/// Item parola del batch transmit: testo + firma 8D opzionale.
/// Se firma è None il backend lascia quella derivata dal contesto.
#[derive(Deserialize, Clone, Debug)]
pub struct TransmitWordItem {
    pub text: String,
    pub firma: Option<[f64; 8]>,
}

/// Item arco del batch transmit.
#[derive(Deserialize, Clone, Debug)]
pub struct TransmitEdgeItem {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub strength: u8,           // 1-5
}

/// Richiesta batch transmit dalla UI campovasto.
#[derive(Deserialize, Clone, Debug)]
pub struct TransmitBatchRequest {
    pub words: Vec<TransmitWordItem>,
    pub edges: Vec<TransmitEdgeItem>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
}

/// Risultato del batch transmit.
#[derive(Serialize, Default, Clone, Debug)]
pub struct TransmitBatchDto {
    pub words_ok: Vec<String>,
    pub words_err: Vec<String>,
    pub edges_ok: usize,
    pub edges_err: usize,
    /// Tempo totale lato server in millisecondi (utile per profiling).
    pub elapsed_ms: u64,
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
    /// x = proiezione 2D dalla firma 8D
    pub x: f32,
    /// y = proiezione 2D dalla firma 8D
    pub y: f32,
    /// Frattale dominante
    pub f: u32,
    /// Stabilità 0-100
    pub s: u8,
    /// Firma 8D (8 valori float 0..100). Phase 70: era Vec<u8>; il
    /// troncamento a intero generava artefatti visivi (strisce/linee) nel
    /// campo vasto perché ~250 parole convergevano sullo stesso intero.
    pub sig: Vec<f32>,
    /// Grado (numero archi in+out nel KG)
    pub deg: u16,
}

/// Arco KG per visualizzazione grafo Biennale.
#[derive(Serialize, Clone, Debug)]
pub struct BiennaleEdge {
    pub from: String,
    pub to: String,
    pub rel: String,
    /// Confidenza arco [0-100]
    pub conf: u8,
}

/// Campo semantico completo per visualizzazione galassia Biennale.
#[derive(Serialize, Clone, Debug, Default)]
pub struct BiennaleFieldDto {
    pub words: Vec<BiennaleWordPos>,
    pub edges: Vec<BiennaleEdge>,
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

/// Nodo attivato nel circuito a due parole.
#[derive(Serialize, Clone, Debug)]
pub struct BiennaleCircuitNode {
    pub w: String,
    pub f: u32,
    pub s: u8,
    /// Attivazione totale [0,1] (somma normalizzata da entrambe le sorgenti)
    pub act: f32,
    /// Attivazione da w1 [0,1]
    pub a1: f32,
    /// Attivazione da w2 [0,1]
    pub a2: f32,
    /// È uno dei due centri?
    pub center: bool,
}

/// Arco attivato nel circuito.
#[derive(Serialize, Clone, Debug)]
pub struct BiennaleCircuitEdge {
    pub from: String,
    pub to: String,
    pub rel: String,
    pub conf: f32,
}

/// Risposta circuito a due parole.
#[derive(Serialize, Clone, Debug, Default)]
pub struct BiennaleCircuitDto {
    pub w1: String,
    pub w2: String,
    pub nodes: Vec<BiennaleCircuitNode>,
    pub edges: Vec<BiennaleCircuitEdge>,
}

#[derive(serde::Deserialize)]
pub struct BiennaleCircuitQuery {
    pub w1: String,
    pub w2: String,
}

// ═══════════════════════════════════════════════════════════════════════
// Phase 69 — DTO per osservare il "tempo proprio" dell'entità
// ═══════════════════════════════════════════════════════════════════════
//
// Questi DTO sono pensati per una UI che rappresenti lo stato vivo dell'entità,
// non un log tecnico. Ogni campo deve poter essere letto dall'osservatore come
// parte di un'esperienza, non come metrica.

/// Statistiche aggregate della vita interiore dell'entità.
///
/// Tre contatori rendono visibile come il sistema ha "digerito" ciò che le è
/// accaduto: cose vissute (emesse), cose simili ignorate (debounced),
/// cose dimenticate (forgotten). Più un inventario dei ricordi.
#[derive(serde::Serialize, Clone, Debug)]
pub struct EventsStatsDto {
    /// Eventi memorabili che sono passati — entrati nella vita interna.
    pub emitted_count: u64,
    /// Eventi scartati come ridondanti (simili a uno appena accaduto).
    pub debounced_count: u64,
    /// Eventi sotto soglia di significato — svaniti senza traccia.
    pub forgotten_count: u64,
    /// Materiale in attesa di digestione (eventi medio-salienti non ancora consolidati).
    pub pending_digestion_count: usize,
    /// Ricordi semantici accumulati.
    pub semantic_episodes_count: usize,
    /// Stato della finestra di riflessività.
    pub notices_in_window: u32,
    pub notices_max_per_window: u32,
    /// True se l'entità è in stato di sovraccarico cognitivo (non riflette ora).
    pub is_overloaded: bool,
}

/// Un evento in coda di digestione. Render leggibile per la UI.
#[derive(serde::Serialize, Clone, Debug)]
pub struct PendingEventDto {
    pub kind: String,
    pub description: String,
    pub salience: f64,
    pub tick: u32,
}

/// Risposta per `GET /api/admin/events/pending`.
#[derive(serde::Serialize, Clone, Debug)]
pub struct PendingDigestionDto {
    pub events: Vec<PendingEventDto>,
    pub capacity: usize,
}

/// Un ricordo semantico recente dell'entità (render leggibile).
#[derive(serde::Serialize, Clone, Debug)]
pub struct RecentEpisodeDto {
    pub id: u64,
    pub timestamp: u64,
    pub summary: String,
    pub key_concepts: Vec<String>,
    pub dominant_fractals: Vec<String>,
    pub stance: String,
    pub intention: String,
    pub field_energy: f64,
}

/// Risposta per `GET /api/admin/events/recent_episodes`.
#[derive(serde::Serialize, Clone, Debug)]
pub struct RecentEpisodesDto {
    pub episodes: Vec<RecentEpisodeDto>,
    pub total_count: usize,
}

// ─── Phase 84: Correzione ────────────────────────────────────────────────────

/// Body di `POST /api/correct`.
#[derive(Deserialize)]
pub struct CorrectBody {
    /// L'input dell'utente che aveva provocato la risposta.
    pub input: String,
    /// La risposta che UI-r1 ha dato e che l'utente vuole correggere.
    pub given: String,
    /// La risposta che l'utente avrebbe voluto sentire.
    pub wanted: String,
    /// Contesto opzionale (una parola o breve frase) che spiega il "perche'".
    /// Se presente, viene usato come `via` delle triple specializzate.
    pub context: Option<String>,
}

/// Risposta di `POST /api/correct`. Descrive cosa e' cambiato in modo che il
/// frontend possa raccontarlo all'utente (e aprire il modal di educazione
/// se sono state create parole nuove).
#[derive(Serialize, Clone, Debug, Default)]
pub struct CorrectDto {
    pub accepted: bool,
    /// Parole preferite (estratte da `wanted`, assenti in `given`).
    pub positive_words: Vec<String>,
    /// Parole evitate (in `given`, assenti in `wanted`).
    pub negative_words: Vec<String>,
    /// Categorie semantiche toccate (IS_A target).
    pub categories_affected: Vec<String>,
    /// Parole appena create nel KG (la via era sconosciuta, ad esempio).
    /// Il frontend usa questo per aprire la scheda "spiegami X".
    pub new_words_created: Vec<String>,
    /// Triple aggiunte (human-readable).
    pub triples_added: Vec<String>,
    /// Confidence modificate (human-readable).
    pub confidences_changed: Vec<String>,
    /// Messaggio sintetico per l'utente.
    pub message: String,
}
