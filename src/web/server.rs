/// Server — Entry point del binario prometeo-web.
///
/// L'engine vive in un thread OS dedicato (non e Send).
/// Comunicazione via mpsc (comandi) e broadcast (aggiornamenti).

use std::collections::HashSet;
use tokio::sync::{mpsc, oneshot, broadcast};
use axum::{Router, routing::{get, post, delete}};
use tower_http::cors::CorsLayer;

use crate::topology::engine::PrometeoTopologyEngine;
use crate::topology::persistence::PrometeoState;
use crate::topology::vital::TensionState;
use crate::topology::dream::SleepPhase;
use crate::topology::valence::{Valence, DRIVE_NAMES};

/// Converte una Valence in ValenceDto per la UI.
fn valence_to_dto(v: &Valence) -> ValenceDto {
    ValenceDto {
        drives: v.drives.iter().enumerate().map(|(i, &val)| {
            ValenceDriveDto { name: DRIVE_NAMES[i].to_string(), value: val }
        }).collect(),
        label: v.derived_stance_label().to_string(),
        hedonic_tone: v.hedonic_tone(),
        intensity: v.intensity(),
        summary: v.summary(),
    }
}

/// Converte una Option<Valence> in Option<ValenceDto>.
fn opt_valence_to_dto(v: &Option<Valence>) -> Option<ValenceDto> {
    v.as_ref().map(valence_to_dto)
}


use super::state::*;
use super::api;
use super::ws;

/// Avvia il server web.
pub async fn run(port: u16) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<EngineCommand>(64);
    let (broadcast_tx, _) = broadcast::channel::<String>(128);

    let state = AppState {
        cmd_tx: cmd_tx.clone(),
        broadcast_tx: broadcast_tx.clone(),
        conv_store: std::sync::Arc::new(std::sync::Mutex::new(
            super::conversations::ConversationStore::load_or_new()
        )),
    };

    // Thread OS dedicato per l'engine (non e Send)
    let broadcast_tx_clone = broadcast_tx.clone();
    std::thread::spawn(move || {
        engine_loop(cmd_rx, broadcast_tx_clone);
    });

    // Auto-dream: tick autonomo ogni 3 secondi.
    // Skip (non Burst) per evitare accumulo di tick durante il caricamento iniziale.
    let dream_cmd_tx = cmd_tx.clone();
    let dream_broadcast = broadcast_tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let (tx, rx) = oneshot::channel();
            if dream_cmd_tx.send(EngineCommand::Dream { ticks: 1, reply: tx }).await.is_ok() {
                if let Ok(snapshot) = rx.await {
                    let update = serde_json::json!({
                        "type": "state_update",
                        "data": &snapshot,
                    });
                    let _ = dream_broadcast.send(update.to_string());
                }
            }
        }
    });

    let app = Router::new()
        .route("/", get(api::index))
        .route("/admin", get(api::admin_index))
        .route("/api/state", get(api::get_state))
        .route("/api/input", post(api::post_input))
        .route("/api/dream", post(api::post_dream))
        .route("/api/grow", post(api::post_grow))
        .route("/api/topology", get(api::get_topology))
        .route("/api/navigate/{from}/{to}", get(api::get_navigate))
        .route("/api/projection", get(api::get_projection))
        .route("/api/introspect", get(api::get_introspect))
        .route("/api/why", get(api::get_why))
        .route("/api/ask", get(api::get_ask))
        .route("/api/open-questions", get(api::get_open_questions))
        .route("/api/clarity", post(api::post_clarity))
        .route("/api/thought-chain", get(api::get_thought_chain))
        .route("/api/generate", get(api::get_generate))
        .route("/api/save", post(api::post_save))
        .route("/api/will", get(api::get_will))
        .route("/api/will/focus", post(api::post_will_focus))
        .route("/api/dream/report", get(api::get_dream_report))
        .route("/api/compounds", get(api::get_compounds))
        .route("/api/wordfield", get(api::get_wordfield))
        .route("/api/phase/{a}/{b}", get(api::get_phase))
        .route("/api/tension/{a}/{b}", get(api::get_tension))
        .route("/api/locus-simulate", post(api::post_locus_simulate))
        .route("/api/narrative", get(api::get_narrative))
        .route("/api/thoughts", get(api::get_thoughts))
        .route("/api/visuals", get(api::get_visuals))
        .route("/api/simpdb", get(api::get_simpdb))
        .route("/api/universe", get(api::get_universe))
        .route("/api/word_neighbors", get(api::get_word_neighbors))
        .route("/api/word_detail", get(api::get_word_detail))
        .route("/api/word_connect", post(api::post_word_connect))
        .route("/api/concept", get(api::get_concept))
        .route("/api/self", get(api::get_self))
        .route("/api/episodes", get(api::get_episodes))
        .route("/api/episodes/recall", get(api::recall_episodes))
        // Community session
        .route("/universo", get(api::universo_index))
        .route("/community", get(api::community_index))
        .route("/api/community/teach", post(api::post_community_teach))
        .route("/api/community/connect", post(api::post_community_connect))
        .route("/api/community/validate", post(api::post_community_validate))
        .route("/api/community/session", get(api::get_community_session))
        .route("/api/community/field", get(api::get_community_field))
        .route("/api/community/reset", post(api::post_community_reset))
        // Phase 52: Dialogo interiore
        .route("/api/inner-dialogue", get(api::get_inner_dialogue))
        .route("/api/respond", post(api::post_respond))
        // Gestione archi e relazioni
        .route("/api/relations", get(api::get_relations))
        .route("/api/edge", post(api::delete_edge))
        .route("/api/edge/confidence", post(api::patch_edge))
        // ── Biennale ─────────────────────────────────────────────
        .route("/biennale", get(api::biennale_index))
        .route("/dialogo", get(api::dialogo_index))
        .route("/curazione", get(api::curazione_index))
        .route("/api/cura/parole", get(api::get_word_list))
        .route("/api/cura/relazione", delete(api::delete_word_relation))
        .route("/api/cura/relazione/modifica", post(api::post_update_edge))
        .route("/api/cura/parola", delete(api::delete_word))
        .route("/api/cura/firma", post(api::post_update_firma))
        .route("/api/cura/categorie", get(api::get_categories))
        .route("/api/cura/pulizia-verbi", post(api::post_pulizia_verbi))
        .route("/api/biennale/field", get(api::get_biennale_field))
        .route("/api/biennale/word", get(api::get_biennale_word))
        .route("/api/biennale/journey", get(api::get_biennale_journey))
        .route("/ws", get(ws::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("╔══════════════════════════════════════════════╗");
    println!("║  PROMETEO — Topologia Cognitiva 8D          ║");
    println!("║  Web UI: http://localhost:{}               ║", port);
    println!("╚══════════════════════════════════════════════╝");

    let listener = tokio::net::TcpListener::bind(&addr).await
        .expect("Impossibile avviare il server");
    axum::serve(listener, app).await
        .expect("Errore nel server");
}

// ═══════════════════════════════════════════════════════════════
// Pulizia verbi coniugati
// ═══════════════════════════════════════════════════════════════

fn pulizia_verbi(engine: &mut crate::topology::engine::PrometeoTopologyEngine, dry_run: bool) -> crate::web::state::PuliziaDto {
    // Suffissi inequivocabilmente coniugati (non usati in nomi/aggettivi comuni)
    const SUFFIXES: &[&str] = &[
        // Gerundio
        "ando", "endo",
        // Imperfetto (forme lunghe sicure)
        "avamo", "avate", "avano",
        "evamo", "evate", "evano",
        "ivamo", "ivate", "ivano",
        // Futuro
        "eremo", "erete", "eranno",
        "iremo", "irete", "iranno",
        "aremo", "arete", "aranno",
        "erò", "erai", "erà",
        "irò", "irai", "irà",
        "arò", "arai", "arà",
        // Condizionale
        "erei", "eresti", "erebbe", "eremmo", "ereste", "erebbero",
        "irei", "iresti", "irebbe", "iremmo", "ireste", "irebbero",
        "arei", "aresti", "arebbe", "aremmo", "areste", "arebbero",
        // Passato remoto (forme lunghe)
        "ammo", "arono",
        "emmo", "erono",
        "immo", "irono",
        // Congiuntivo imperfetto
        "assi", "asse", "assimo", "assero",
        "essi", "esse", "essimo", "essero",
        "issi", "isse", "issimo", "issero",
        // Congiuntivo presente 1a (-are): tutte finiscono in -i ma troppo ambigue — skip
    ];

    // Parole da proteggere sempre (funzionali, eccezioni comuni)
    const PROTECTED: &[&str] = &[
        "quando", "quanto", "intanto", "frattanto", "mentre",
        "dentro", "attorno", "intorno", "secondo", "comando",
        "bando", "fondo", "mondo", "rotondo", "secondo",
        "grande", "prende", "rende", "vende", "pende",
        "ieri", "speri", "veri", "interi", "leggeri",
        "già", "più", "però", "però", "andrà", "sarà",
    ];

    let min_len = 5usize;

    let words: Vec<String> = engine.lexicon.patterns_iter()
        .map(|(w, _)| w.to_string())
        .collect();

    let mut to_delete: Vec<String> = Vec::new();
    for word in &words {
        if PROTECTED.contains(&word.as_str()) { continue; }
        // Non cancellare infiniti
        if word.ends_with("are") || word.ends_with("ere") || word.ends_with("ire") { continue; }
        if word.len() < min_len { continue; }
        let matched = SUFFIXES.iter().any(|suf| word.ends_with(suf) && word.len() > suf.len() + 1);
        if matched {
            to_delete.push(word.clone());
        }
    }

    to_delete.sort();
    let count = to_delete.len();

    if !dry_run {
        for word in &to_delete {
            engine.kg.remove_word(word);
            engine.lexicon.remove_word(word);
        }
        if count > 0 { cura_save(engine); }
    }

    crate::web::state::PuliziaDto { deleted: to_delete, count, dry_run }
}

// ═══════════════════════════════════════════════════════════════
// Salvataggio curazione: persiste il .bin dopo ogni mutazione KG/lessico
// ═══════════════════════════════════════════════════════════════

fn cura_save(engine: &crate::topology::engine::PrometeoTopologyEngine) {
    use std::path::Path;
    use crate::topology::persistence::PrometeoState;
    let state = PrometeoState::capture(engine);
    match state.save_to_binary(Path::new("prometeo_topology_state.bin")) {
        Ok(_) => {}
        Err(e) => eprintln!("[cura] Errore salvataggio .bin: {}", e),
    }
}

// ═══════════════════════════════════════════════════════════════
// Engine loop: gira nel thread OS dedicato
// ═══════════════════════════════════════════════════════════════

fn engine_loop(
    mut cmd_rx: mpsc::Receiver<EngineCommand>,
    broadcast_tx: broadcast::Sender<String>,
) {
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Sessione comunitaria: stato in-memory
    let mut session_log = SessionStateDto {
        community_name: String::new(),
        teach_entries: Vec::new(),
        community_edges: Vec::new(),
        founding_narrative: String::new(),
        total_words_taught: 0,
        total_connections: 0,
        active_participants: Vec::new(),
    };

    // Carica stato salvato o crea nuovo.
    // Priorita: SimplDB .bin (veloce, mmap) → JSON (legacy) → bootstrap seed
    // Su Android: controlla anche /sdcard/ per facilitare il trasferimento manuale del .bin
    // Nota: quando si carica da disco si usa new_empty() (skip bootstrap) + restore.
    // Il bootstrap completo (36 parole cardinali) è riservato al seed (primo avvio o --seed).
    let binary_paths = [
        "prometeo_topology_state.bin",
        "prometeo_state.bin",
        "/sdcard/prometeo_topology_state.bin",
        "/sdcard/prometeo_state.bin",
    ];
    let json_paths = [
        "prometeo_topology_state.json",
        "prometeo_state.json",
    ];
    let mut engine = {
        let mut loaded = None;
        // Prova prima il formato binario SimplDB
        for path_str in &binary_paths {
            if Path::new(path_str).exists() {
                match PrometeoState::load_from_binary(Path::new(path_str)) {
                    Ok(state) => {
                        println!("[engine] Stato .bin caricato da: {} ({} parole)",
                            path_str, state.lexicon.words.len());
                        let mut eng = PrometeoTopologyEngine::new_empty();
                        state.restore_lexicon(&mut eng);
                        // NON chiamare apply_curated_signatures() qui: sovrascrive le
                        // modifiche manuali alle firme salvate nel .bin. Le firme curate
                        // sono già nel .bin dal primo avvio o dall'ultima modifica UI.
                        eng.recompute_all_word_affinities();
                        eng.rebuild_pf_field();
                        eng.seed_conceptual_anchors();
                        loaded = Some(eng);
                        break;
                    }
                    Err(e) => eprintln!("[engine] Errore .bin {}: {}", path_str, e),
                }
            }
        }
        // Fallback JSON
        if loaded.is_none() {
            for path_str in &json_paths {
                if let Ok(state) = PrometeoState::load_from_file(Path::new(path_str)) {
                    println!("[engine] Stato .json caricato da: {} ({} parole)",
                        path_str, state.lexicon.words.len());
                    let mut eng = PrometeoTopologyEngine::new_empty();
                    state.restore_lexicon(&mut eng);
                    eng.lexicon.apply_curated_signatures();
                    eng.recompute_all_word_affinities();
                    eng.rebuild_pf_field();
                    eng.seed_conceptual_anchors();
                    loaded = Some(eng);
                    break;
                }
            }
        }
        loaded.unwrap_or_else(|| {
            println!("[engine] Nessuno stato trovato — bootstrap seed ({} parole cardinali)", 36);
            PrometeoTopologyEngine::new()
        })
    };

    // Carica il Knowledge Graph (se disponibile)
    engine.load_kg_from_file(Path::new("prometeo_kg.json"));

    // Phase 43B — Narrativa fondativa: solo al primo avvio (is_born == false).
    if !engine.narrative_self.is_born {
        engine.initialize_founding_narrative();
        println!("[engine] Narrativa fondativa cristallizzata — Prometeo nasce");
    }

    // Loop sincrono: ricevi comandi dal canale mpsc
    while let Some(cmd) = cmd_rx.blocking_recv() {
        match cmd {
            EngineCommand::Receive { input, reply } => {
                let response = engine.receive(&input);
                let generated = engine.generate_willed();
                let snapshot = build_snapshot(&mut engine);
                let stance = format!("{:?}", engine.narrative_self.stance);
                let valence_label = engine.narrative_self.valence.derived_stance_label().to_string();
                let intention = engine.narrative_self.pending_intention
                    .as_ref()
                    .map(|i| format!("{:?}", i))
                    .unwrap_or_else(|| "Express".to_string());
                let topic_continuity = engine.narrative_self.topic_continuity;
                let _ = reply.send(InputResponse {
                    generated_text: generated.text,
                    keywords: response.keywords,
                    state: snapshot,
                    stance,
                    valence_label,
                    intention,
                    topic_continuity,
                });
            }
            EngineCommand::GetState { reply } => {
                let snapshot = build_snapshot(&mut engine);
                let _ = reply.send(snapshot);
            }
            EngineCommand::GetTopology { reply } => {
                let dto = build_topology(&engine);
                let _ = reply.send(dto);
            }
            EngineCommand::Navigate { from, to, reply } => {
                let dto = build_navigation(&engine, &from, &to);
                let _ = reply.send(dto);
            }
            EngineCommand::Dream { ticks, reply } => {
                for _ in 0..ticks {
                    engine.autonomous_tick();
                }
                let snapshot = build_snapshot(&mut engine);
                let _ = reply.send(snapshot);
            }
            EngineCommand::Grow { reply } => {
                let events = engine.grow();
                let new_f = events.iter().filter(|e| matches!(e, crate::topology::growth::GrowthEvent::NewFractal { .. })).count();
                let new_c = events.iter().filter(|e| matches!(e, crate::topology::growth::GrowthEvent::NewConnection { .. })).count();
                let descs: Vec<String> = events.iter().map(|e| format!("{:?}", e)).collect();
                let _ = reply.send(GrowthDto {
                    events: descs,
                    new_fractals: new_f,
                    new_connections: new_c,
                });
            }
            EngineCommand::Introspect { reply } => {
                let intro = engine.introspect();
                let _ = reply.send(IntrospectionDto {
                    fractal_count: intro.fractal_count,
                    simplex_count: intro.simplex_count,
                    conceptual_gaps: intro.conceptual_gaps,
                    disconnected_worlds: intro.disconnected_worlds,
                    densest_region: intro.densest_region.map(|(n, c)| format!("{} ({})", n, c)),
                    sparsest_region: intro.sparsest_region.map(|(n, c)| format!("{} ({})", n, c)),
                    field_energy: intro.field_energy,
                    emergent_dimensions: intro.emergent_dimensions,
                    most_experienced: intro.most_experienced.map(|(n, c)| format!("{} ({})", n, c)),
                    least_experienced: intro.least_experienced.map(|(n, c)| format!("{} ({})", n, c)),
                });
            }
            EngineCommand::Why { reply } => {
                let trace = engine.why();
                let _ = reply.send(WhyDto {
                    explanation: trace.explanation,
                    fractal_sequence: trace.fractal_sequence.iter()
                        .map(|(name, act)| FractalActiveDto { name: name.clone(), activation: *act })
                        .collect(),
                    propagation_bridges: trace.propagation_bridges,
                });
            }
            EngineCommand::Ask { reply } => {
                // Restituisce le incertezze aperte come QuestionDto (compatibilità)
                let uncertainties = engine.open_uncertainties();
                let _ = reply.send(uncertainties.iter().map(|u| QuestionDto {
                    text: u.topic.clone(),
                    question_type: "Uncertainty".to_string(),
                    priority: u.tension,
                }).collect());
            }
            EngineCommand::GetOpenQuestions { reply } => {
                let uncertainties = engine.open_uncertainties();
                let _ = reply.send(uncertainties.into_iter().map(|u| UncertaintyDto {
                    topic: u.topic,
                    tension: u.tension,
                    emergence_count: u.emergence_count,
                }).collect());
            }
            EngineCommand::Clarity { topic, illumination, reply } => {
                engine.receive_clarity(&topic, &illumination);
                let _ = reply.send(true);
            }
            EngineCommand::GetLastThoughtChain { reply } => {
                use crate::topology::thought_chain::{ChainOutcome};
                let dto = engine.last_thought_chain.as_ref().map(|chain| {
                    let steps = chain.steps.iter().map(|s| ThoughtStepDto {
                        from_concept: s.from_concept.clone(),
                        relation: s.relation.clone(),
                        to_concept: s.to_concept.clone(),
                        confidence: s.confidence,
                        insight: s.insight.clone(),
                    }).collect();
                    let outcome = match &chain.outcome {
                        ChainOutcome::NewInsight { claim, .. } => ThoughtOutcomeDto {
                            kind: "insight".to_string(),
                            claim: Some(claim.clone()),
                            new_topic: None,
                        },
                        ChainOutcome::NewUncertainty { topic, .. } => ThoughtOutcomeDto {
                            kind: "new_question".to_string(),
                            claim: None,
                            new_topic: Some(topic.clone()),
                        },
                        ChainOutcome::DeadEnd => ThoughtOutcomeDto {
                            kind: "dead_end".to_string(),
                            claim: None,
                            new_topic: None,
                        },
                    };
                    ThoughtChainDto {
                        origin_description: chain.origin.description(),
                        steps,
                        outcome,
                        depth_reached: chain.depth_reached,
                    }
                });
                let _ = reply.send(dto);
            }
            EngineCommand::Projection { reply } => {
                let proj = engine.holographic_projection();
                let _ = reply.send(proj.map(|p| ProjectionDto {
                    from_name: p.from_name,
                    projections: p.projections.iter().map(|fp| ProjectionItemDto {
                        name: fp.name.clone(),
                        proximity: fp.proximity,
                        dimensional_resonance: fp.dimensional_resonance,
                        distortion: fp.distortion,
                        apparent_center: fp.apparent_center.values().to_vec(),
                    }).collect(),
                }));
            }
            EngineCommand::Generate { reply } => {
                let gen = engine.generate();
                let _ = reply.send(GenerateDto {
                    text: gen.text,
                    structure: format!("{:?}", gen.structure),
                    cluster_count: gen.cluster_count,
                });
            }
            EngineCommand::Save { reply } => {
                let state = PrometeoState::capture(&engine);
                let ok = state.save_to_file(Path::new("prometeo_topology_state.json")).is_ok();
                if ok { println!("[engine] Stato salvato su disco"); }
                let _ = reply.send(ok);
            }
            EngineCommand::GetWill { reply } => {
                let dto = build_will(&mut engine);
                let _ = reply.send(dto);
            }
            EngineCommand::GetCompounds { reply } => {
                let dto = build_compounds(&engine);
                let _ = reply.send(dto);
            }
            EngineCommand::GetWordField { reply } => {
                let dto = build_word_field(&engine);
                let _ = reply.send(dto);
            }
            EngineCommand::GetPhase { word_a, word_b, reply } => {
                let dto = build_phase(&engine, &word_a, &word_b);
                let _ = reply.send(dto);
            }
            EngineCommand::GetTension { pole_a, pole_b, reply } => {
                let dto = build_tension(&engine, &pole_a, &pole_b);
                let _ = reply.send(dto);
            }
            EngineCommand::GetNarrative { reply } => {
                use crate::web::state::{NarrativeDto, NarrativeTurnDto, NarrativePositionDto, CommitmentDto};
                use crate::topology::input_reading::InputAct;
                let ns = &engine.narrative_self;

                // Riconoscimento intento in linguaggio naturale con contesto specifico
                let describe_act = |act: &InputAct, turn: &crate::topology::narrative::NarrativeTurn| -> (String, Option<String>, Option<String>) {
                    let act_label = match act {
                        InputAct::Greeting => "Saluto".to_string(),
                        InputAct::SelfQuery => "Domanda su di sé".to_string(),
                        InputAct::Question => "Domanda".to_string(),
                        InputAct::EmotionalExpr => "Espressione emotiva".to_string(),
                        InputAct::Declaration => "Dichiarazione".to_string(),
                    };

                    // Intent riconosciuto: usa le parole specifiche dell'input
                    let words_ctx = if turn.input_words.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", turn.input_words.join(", "))
                    };
                    let salient = turn.salient_word.as_deref().unwrap_or("");

                    let recognized = match act {
                        InputAct::Greeting => Some(format!("Contatto sociale{}", words_ctx)),
                        InputAct::SelfQuery => {
                            if salient.is_empty() {
                                Some(format!("Chiede chi/cosa è Prometeo{}", words_ctx))
                            } else {
                                Some(format!("Chiede di '{}'{}", salient, words_ctx))
                            }
                        },
                        InputAct::Question => {
                            if salient.is_empty() {
                                Some(format!("Cerca chiarimento{}", words_ctx))
                            } else {
                                Some(format!("Domanda su '{}'{}", salient, words_ctx))
                            }
                        },
                        InputAct::EmotionalExpr => {
                            if salient.is_empty() {
                                Some(format!("Esprime stato emotivo{}", words_ctx))
                            } else {
                                Some(format!("Esprime emozione legata a '{}'{}", salient, words_ctx))
                            }
                        },
                        InputAct::Declaration => {
                            if turn.input_words.len() >= 2 {
                                Some(format!("Afferma relazione tra {}", turn.input_words.join(" e ")))
                            } else if !salient.is_empty() {
                                Some(format!("Afferma qualcosa su '{}'{}", salient, words_ctx))
                            } else {
                                Some(format!("Dichiarazione{}", words_ctx))
                            }
                        },
                    };

                    // Posizione formata: cosa Prometeo decide prima di rispondere
                    let stance_str = turn.stance.as_str();
                    let intention_str = format!("{:?}", turn.intention);
                    let formed = Some(format!("{} → {}", stance_str, intention_str));

                    (act_label, recognized, formed)
                };

                let recent: Vec<NarrativeTurnDto> = ns.turns.iter().rev().take(8).map(|t| {
                    let (act_label, recognized, formed) = describe_act(&t.received_act, t);
                    NarrativeTurnDto {
                        turn_id:    t.turn_id,
                        act:        act_label,
                        stance:     t.stance.as_str().to_string(),
                        intention:  format!("{:?}", t.intention),
                        intensity:  t.intensity,
                        awareness:  t.awareness.clone(),
                        crystallized: false,
                        recognized_intent: recognized,
                        formed_position: formed,
                        inner_state: t.inner_state_summary.clone(),
                        valence: opt_valence_to_dto(&t.valence),
                    }
                }).collect();
                let crys: Vec<NarrativeTurnDto> = ns.crystallized.iter().rev().map(|t| {
                    let (act_label, recognized, formed) = describe_act(&t.received_act, t);
                    NarrativeTurnDto {
                        turn_id:    t.turn_id,
                        act:        act_label,
                        stance:     t.stance.as_str().to_string(),
                        intention:  format!("{:?}", t.intention),
                        intensity:  t.intensity,
                        awareness:  t.awareness.clone(),
                        crystallized: true,
                        recognized_intent: recognized,
                        formed_position: formed,
                        inner_state: t.inner_state_summary.clone(),
                        valence: opt_valence_to_dto(&t.valence),
                    }
                }).collect();
                let pos: Vec<NarrativePositionDto> = ns.positions.iter().map(|(k, (s, i))| NarrativePositionDto {
                    act_key:   k.clone(),
                    stance:    s.as_str().to_string(),
                    intention: format!("{:?}", i),
                }).collect();
                let commitment_dto = ns.commitment.as_ref().map(|c| CommitmentDto {
                    intention:  format!("{:?}", c.intention),
                    strength:   c.strength,
                    turns_held: c.turns_held,
                    inertia:    c.inertia(),
                });
                let dto = NarrativeDto {
                    stance:            ns.stance.as_str().to_string(),
                    valence_label:     ns.valence.derived_stance_label().to_string(),
                    pending_intention: ns.pending_intention.as_ref().map(|i| format!("{:?}", i)),
                    topic_continuity:  ns.topic_continuity,
                    is_born:           ns.is_born,
                    turn_count:        ns.turns.len(),
                    valence:           Some(valence_to_dto(&ns.valence)),
                    commitment:        commitment_dto,
                    coherence_integrity: engine.identity.coherence_integrity,
                    attributed_intent: format!("{:?}", engine.interlocutor.attributed_intent),
                    recent_turns:      recent,
                    crystallized:      crys,
                    positions:         pos,
                };
                let _ = reply.send(dto);
            }
            EngineCommand::GetThoughts { reply } => {
                let thoughts = crate::topology::thought::generate_thoughts(&engine);
                let dto: Vec<api::ThoughtDto> = thoughts.into_iter().map(|t| {
                    use crate::topology::thought::{ThoughtData, ThoughtKind};
                    let kind = match t.kind {
                        ThoughtKind::Tension       => "tension",
                        ThoughtKind::Gap           => "gap",
                        ThoughtKind::MissingBridge => "missing_bridge",
                        ThoughtKind::Disconnection => "disconnection",
                        ThoughtKind::Hypothesis    => "hypothesis",
                        ThoughtKind::AbductiveHypothesis => "abductive_hypothesis",
                        ThoughtKind::SelfDiscovery => "self_discovery",
                        ThoughtKind::Need          => "need",
                        ThoughtKind::Desire        => "desire",
                        ThoughtKind::Interlocutor  => "interlocutor",
                        ThoughtKind::Humor         => "humor",
                    }.to_string();
                    let detail = match &t.data {
                        ThoughtData::TensionData { phase, word_a, word_b } =>
                            serde_json::json!({ "phase_pi": phase / std::f64::consts::PI, "word_a": word_a, "word_b": word_b }),
                        ThoughtData::GapData { simplex_count, word_count, activation_count } =>
                            serde_json::json!({ "simplex_count": simplex_count, "word_count": word_count, "activation_count": activation_count }),
                        ThoughtData::MissingBridgeData { proximity, shared_simplices } =>
                            serde_json::json!({ "proximity": proximity, "shared_simplices": shared_simplices }),
                        ThoughtData::DisconnectionData { components } =>
                            serde_json::json!({ "components": components }),
                        ThoughtData::HypothesisData { simplex_id, dimension, activation_count } =>
                            serde_json::json!({ "simplex_id": simplex_id, "dimension": dimension, "activation_count": activation_count }),
                        ThoughtData::SelfDiscoveryData { divergence, emergent_fractals, trigger_words } =>
                            serde_json::json!({ "divergence": divergence, "emergent_fractals": emergent_fractals, "trigger_words": trigger_words }),
                        ThoughtData::NeedData { level, satisfaction } =>
                            serde_json::json!({ "level": level, "satisfaction": satisfaction }),
                        ThoughtData::DesireData { name, intensity, distance } =>
                            serde_json::json!({ "name": name, "intensity": intensity, "distance": distance }),
                        ThoughtData::InterlocutorData { presence, pattern, resonance } =>
                            serde_json::json!({ "presence": presence, "pattern": pattern, "resonance": resonance }),
                        ThoughtData::HumorData { incongruity, irony_pairs, bisociation } =>
                            serde_json::json!({ "incongruity": incongruity, "irony_pairs": irony_pairs, "bisociation": bisociation }),
                    };
                    api::ThoughtDto { kind, fractal_names: t.fractal_names, words: t.words, strength: t.strength, detail }
                }).collect();
                let _ = reply.send(dto);
            }
            EngineCommand::GetVisuals { reply } => {
                use crate::topology::fractal_visuals::{fractal_svg_from_registry, compose_simplex_svg, FRACTAL_COUNT};
                use crate::topology::simplex::SharedStructureType;

                // Attivazioni correnti dai frattali nel campo PF1
                let acts = {
                    let scores = engine.pf_activation.emerge_fractal_activations(&engine.pf_field);
                    scores.iter().enumerate()
                        .filter(|(_, &s)| s > 0.01)
                        .map(|(id, &s)| (id as u32, s as f64))
                        .collect::<Vec<_>>()
                };
                let act_map: std::collections::HashMap<u32, f64> = acts.into_iter().collect();

                let fractals = (0..FRACTAL_COUNT as u32).filter_map(|id| {
                    let name = engine.registry.get(id)?.name.clone();
                    let svg = fractal_svg_from_registry(id, &engine.registry)?;
                    let activation = *act_map.get(&id).unwrap_or(&0.0);
                    Some(api::FractalVisualDto { id, name, svg, activation })
                }).collect();

                // Simplessi: prende tutti, ordina per activation desc, max 24
                let mut simplices: Vec<api::SimplexVisualDto> = engine.complex.iter()
                    .map(|(_, s)| {
                        let name = s.shared_faces.iter()
                            .find_map(|f| {
                                if let SharedStructureType::EmergentProperty(n) = &f.structure {
                                    Some(n.clone())
                                } else { None }
                            })
                            .unwrap_or_else(|| {
                                s.vertices.iter()
                                    .map(|&fid| engine.registry.get(fid)
                                        .map(|f| f.name.as_str()).unwrap_or("?"))
                                    .collect::<Vec<_>>().join("+")
                            });
                        let fractal_names: Vec<String> = s.vertices.iter()
                            .map(|&fid| engine.registry.get(fid)
                                .map(|f| f.name.clone())
                                .unwrap_or_default())
                            .collect();
                        let svg = compose_simplex_svg(&s.vertices, &name);
                        let strength = s.shared_faces.iter()
                            .map(|f| f.strength).sum::<f64>().min(1.0);
                        api::SimplexVisualDto {
                            name,
                            fractal_names,
                            svg,
                            strength,
                            activation: s.current_activation,
                        }
                    })
                    .collect();

                simplices.sort_by(|a, b| b.activation.partial_cmp(&a.activation)
                    .unwrap_or(std::cmp::Ordering::Equal));
                simplices.truncate(24);

                let _ = reply.send(api::VisualsDto { fractals, simplices });
            }
            EngineCommand::GetUniverse { reply } => {
                let dto = build_universe(&engine);
                let _ = reply.send(dto);
            }
            EngineCommand::GetWordNeighbors { word, reply } => {
                let dto = build_word_neighbors(&engine, &word);
                let _ = reply.send(dto);
            }
            EngineCommand::GetWordDetail { word, reply } => {
                let dto = build_word_detail(&engine, &word);
                let _ = reply.send(dto);
            }
            EngineCommand::AddWordConnect { from, relation, to, via, confidence, reply } => {
                let ok = add_word_connect(&mut engine, &from, &relation, &to, via, confidence);
                let _ = reply.send(ok);
            }
            EngineCommand::DeleteWordRelation { subject, relation, object, reply } => {
                use crate::topology::relation::RelationType;
                let ok = if let Some(rel) = RelationType::from_str(&relation) {
                    engine.kg.remove_edge(&subject, rel, &object);
                    engine.word_topology.remove_edge_between(&subject, &object);
                    true
                } else { false };
                if ok { cura_save(&engine); }
                let _ = reply.send(ok);
            }
            EngineCommand::DeleteWord { word, reply } => {
                engine.kg.remove_word(&word);
                engine.lexicon.remove_word(&word);
                cura_save(&engine);
                let _ = reply.send(true);
            }
            EngineCommand::GetWordList { query, offset, limit, sort, reply } => {
                let dto = build_word_list(&engine, &query, offset, limit, &sort);
                let _ = reply.send(dto);
            }
            EngineCommand::UpdateWordFirma { word, firma, reply } => {
                use crate::topology::primitive::PrimitiveCore;
                let ok = if let Some(pat) = engine.lexicon.get_mut(&word) {
                    pat.signature = PrimitiveCore::new(firma);
                    true
                } else { false };
                if ok {
                    engine.recompute_all_word_affinities();
                    cura_save(&engine);
                }
                let _ = reply.send(ok);
            }
            EngineCommand::UpdateEdge { subject, relation, object, confidence, via, reply } => {
                use crate::topology::relation::RelationType;
                let ok = if let Some(rel) = RelationType::from_str(&relation) {
                    engine.kg.update_edge(&subject, rel, &object, confidence, via)
                } else { false };
                if ok { cura_save(&engine); }
                let _ = reply.send(ok);
            }
            EngineCommand::PuliziaVerbi { dry_run, reply } => {
                let dto = pulizia_verbi(&mut engine, dry_run);
                let _ = reply.send(dto);
            }
            EngineCommand::GetCategories { relation, min_children, query, reply } => {
                let dto = build_categories(&engine, &relation, min_children, &query);
                let _ = reply.send(dto);
            }
            EngineCommand::GetConcept { word, reply } => {
                let dto = build_concept(&engine, &word);
                let _ = reply.send(dto);
            }
            EngineCommand::GetSelf { reply } => {
                let dto = build_self_dto(&engine);
                let _ = reply.send(dto);
            }
            EngineCommand::GetEpisodes { n, reply } => {
                let episodes: Vec<&crate::topology::semantic_episode::SemanticEpisode> =
                    engine.semantic_episodes.recent(n).iter().collect();
                let dtos = build_episode_dtos(&episodes);
                let _ = reply.send(dtos);
            }
            EngineCommand::RecallEpisodes { concepts, reply } => {
                let recalled = engine.semantic_episodes.recall_by_concepts(&concepts, 10);
                let episodes: Vec<&crate::topology::semantic_episode::SemanticEpisode> =
                    recalled.into_iter().map(|(ep, _)| ep).collect();
                let dtos = build_episode_dtos(&episodes);
                let _ = reply.send(dtos);
            }
            EngineCommand::SimulateLocus { locus_name, reply } => {
                let dto = engine.simulate_locus_view(&locus_name).map(|v| LociSimDto {
                    locus_name: v.locus_name,
                    visible_fractals: v.visible.iter()
                        .map(|(name, vis)| FractalActiveDto { name: name.clone(), activation: *vis })
                        .collect(),
                    active_fractals: v.active_fractals.iter()
                        .map(|(name, act)| FractalActiveDto { name: name.clone(), activation: *act })
                        .collect(),
                    generated_text: v.generated_text,
                });
                let _ = reply.send(dto);
            }

            // ─── Sessione Comunitaria ───────────────────────────────────────
            EngineCommand::CommunityTeach { text, user_id, user_name, user_context, reply } => {
                let energy_before = engine.vital_state().activation;
                let result = engine.teach(&text);
                let energy_after = engine.vital_state().activation;

                // Connessioni KG emergenti: parole nuove connesse a parole già note.
                // Filtra SIMILAR_TO e OPPOSITE_OF (archi Kaikki lessicografici, non semantici).
                use crate::topology::relation::RelationType;
                let connections_found: Vec<(String, String, String, f32)> = result.words_processed
                    .iter()
                    .flat_map(|w| engine.kg.all_outgoing(w.as_str())
                        .into_iter()
                        .filter(|(rel, _, _)| !matches!(rel,
                            RelationType::SimilarTo | RelationType::OppositeOf))
                        .take(2)
                        .map(|(rel, obj, conf)| (w.clone(), format!("{:?}", rel), obj.to_string(), conf))
                        .collect::<Vec<_>>())
                    .take(8)
                    .collect();

                // Frattali toccati
                let fractals_touched: Vec<(String, f64)> = engine.active_fractals()
                    .into_iter()
                    .filter(|(_, act)| *act > 0.1)
                    .take(5)
                    .map(|(name, act)| (name, act))
                    .collect();

                // Parole risonanti: le parole che il campo ha attivato in risposta
                // Escludi le parole che l'utente ha scritto — mostra solo la risonanza
                let user_words_set: std::collections::HashSet<&str> = result.words_processed
                    .iter().map(|w| w.as_str()).collect();
                let resonating_words: Vec<String> = engine.pf_activation
                    .hot_words(&engine.pf_field, 30)
                    .into_iter()
                    .filter(|(w, _)| !user_words_set.contains(w.as_str()))
                    .filter(|(w, _)| w.chars().count() >= 3)
                    .take(10)
                    .map(|(w, _)| w)
                    .collect();

                // Registra nel session log
                let ts = SystemTime::now().duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs()).unwrap_or(0);
                session_log.teach_entries.push(TeachEntry {
                    user_id: user_id.clone(),
                    user_name: user_name.clone(),
                    user_context: user_context.clone(),
                    text: text.clone(),
                    words_new: result.words_new.clone(),
                    timestamp: ts,
                });
                session_log.total_words_taught += result.words_processed.len();
                if !session_log.active_participants.contains(&user_name) {
                    session_log.active_participants.push(user_name.clone());
                }

                // Crea prima il DTO (possiede i dati), poi broadcast dal DTO
                let dto = CommunityTeachDto {
                    words_new: result.words_new,
                    words_known: result.words_known,
                    fractals_touched,
                    connections_found,
                    field_energy_delta: energy_after - energy_before,
                    resonating_words,
                };

                // Broadcast real-time a tutti i client WebSocket connessi
                let broadcast_msg = serde_json::json!({
                    "type": "campo_update",
                    "event": "narra",
                    "user_name": user_name,
                    "words_new": &dto.words_new,
                    "words_known": &dto.words_known,
                    "resonating_words": &dto.resonating_words,
                    "fractals_touched": &dto.fractals_touched,
                    "field_energy": energy_after,
                });
                let _ = broadcast_tx.send(broadcast_msg.to_string());

                let _ = reply.send(dto);
            }

            EngineCommand::CommunityValidateEdge { subject, relation, object, confidence, user_id, user_name, user_context, reply } => {
                use crate::topology::relation::{RelationType, TypedEdge};
                let rel = RelationType::from_str(&relation).unwrap_or(RelationType::SimilarTo);
                let edge = TypedEdge::new(&subject, rel, &object).with_confidence(confidence);
                engine.kg.add_edge(edge);
                // Rebuild semantic topology per il nuovo arco
                engine.build_semantic_simplices_from_kg();

                let ts = SystemTime::now().duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs()).unwrap_or(0);
                session_log.community_edges.push(CommunityEdge {
                    user_id: user_id.clone(),
                    user_name: user_name.clone(),
                    user_context: user_context.clone(),
                    subject: subject.clone(),
                    relation: relation.clone(),
                    object: object.clone(),
                    confidence,
                    timestamp: ts,
                });
                session_log.total_connections += 1;
                if !session_log.active_participants.contains(&user_name) {
                    session_log.active_participants.push(user_name.clone());
                }

                // Broadcast real-time a tutti i client WebSocket connessi
                let broadcast_msg = serde_json::json!({
                    "type": "campo_update",
                    "event": "connetti",
                    "user_name": user_name,
                    "subject": subject,
                    "relation": relation,
                    "object": object,
                });
                let _ = broadcast_tx.send(broadcast_msg.to_string());

                let _ = reply.send(true);
            }

            EngineCommand::GetSessionState { reply } => {
                let _ = reply.send(session_log.clone());
            }

            EngineCommand::ResetSession { community_name, reply } => {
                session_log = SessionStateDto {
                    community_name: community_name.clone(),
                    teach_entries: Vec::new(),
                    community_edges: Vec::new(),
                    founding_narrative: String::new(),
                    total_words_taught: 0,
                    total_connections: 0,
                    active_participants: Vec::new(),
                };
                let _ = reply.send(true);
            }

            // ── Volontà: focalizza su topic ──────────────────────────────
            EngineCommand::WillFocus { topic, reply } => {
                // Attiva la parola nel campo PF1 per modulare la volontà
                engine.pf_activation.activate_by_name(&engine.pf_field, &topic, 0.5);
                engine.word_topology.activate_word(&topic, 0.5);
                // Propaga un tick per far sentire l'effetto
                engine.pf_activation.propagate(&engine.pf_field);
                // Restituisci il will aggiornato
                let dto = build_will(&mut engine);
                let _ = reply.send(dto);
            }

            // ── Dream Report ────────────────────────────────────────────
            EngineCommand::GetDreamReport { reply } => {
                let report = engine.report();
                let (phase_str, depth) = match engine.dream.phase {
                    SleepPhase::Awake => ("Sveglio", 0.0),
                    SleepPhase::WakefulDream { depth } => ("Sogno vigile", depth),
                    SleepPhase::LightSleep { depth } => ("Sonno leggero", depth),
                    SleepPhase::DeepSleep { depth } => ("Sonno profondo", depth),
                    SleepPhase::REM { depth } => ("REM", depth),
                };

                // Descrizioni consolidamento basate sullo stato corrente
                let mut consolidations = Vec::new();
                if report.stm_count > 0 {
                    consolidations.push(ConsolidationDto {
                        description: format!("{} simplessi in memoria a breve termine", report.stm_count),
                        from_layer: "attivo".to_string(),
                        to_layer: "STM".to_string(),
                        strength: 0.3,
                    });
                }
                if report.mtm_count > 0 {
                    consolidations.push(ConsolidationDto {
                        description: format!("{} pattern consolidati in memoria intermedia", report.mtm_count),
                        from_layer: "STM".to_string(),
                        to_layer: "MTM".to_string(),
                        strength: 0.6,
                    });
                }
                if report.ltm_count > 0 {
                    consolidations.push(ConsolidationDto {
                        description: format!("{} strutture cristallizzate in memoria a lungo termine", report.ltm_count),
                        from_layer: "MTM".to_string(),
                        to_layer: "LTM".to_string(),
                        strength: 0.9,
                    });
                }

                // Sommario con contesto semantico (cosa è stato processato)
                let recent_concepts: Vec<String> = engine.semantic_episodes.recent(3)
                    .iter()
                    .flat_map(|ep| ep.key_concepts.iter().take(2).cloned())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .take(6)
                    .collect();
                let summary = if engine.dream.cycles_completed > 0 || !recent_concepts.is_empty() {
                    let mut s = format!(
                        "Cicli: {}. Perturbazioni: {}. Memoria: {} STM → {} MTM → {} LTM.",
                        engine.dream.cycles_completed, engine.total_perturbations,
                        report.stm_count, report.mtm_count, report.ltm_count
                    );
                    if !recent_concepts.is_empty() {
                        s.push_str(&format!(" Temi recenti: {}.", recent_concepts.join(", ")));
                    }
                    Some(s)
                } else {
                    None
                };

                let dto = DreamReportDto {
                    phase: phase_str.to_string(),
                    depth,
                    cycles_completed: engine.dream.cycles_completed,
                    total_perturbations: engine.total_perturbations as u64,
                    consolidations,
                    post_dream_summary: summary,
                    memory_stm: report.stm_count,
                    memory_mtm: report.mtm_count,
                    memory_ltm: report.ltm_count,
                };
                let _ = reply.send(dto);
            }

            // ── Phase 52: Dialogo Interiore ──────────────────────────────
            EngineCommand::GetInnerDialogue { reply } => {
                use crate::topology::thought::{generate_thoughts, ThoughtKind, ThoughtData};

                // Pensieri (dubbi)
                let thoughts = generate_thoughts(&engine);
                let thought_items: Vec<InnerDialogueItem> = thoughts.iter()
                    .enumerate()
                    .filter(|(_, t)| matches!(t.kind,
                        ThoughtKind::Tension | ThoughtKind::Gap | ThoughtKind::MissingBridge))
                    .map(|(i, t)| {
                        let cat = match t.kind {
                            ThoughtKind::Tension => "tensione",
                            ThoughtKind::Gap => "lacuna",
                            ThoughtKind::MissingBridge => "ponte_mancante",
                            _ => "altro",
                        };
                        let text = match &t.data {
                            ThoughtData::TensionData { word_a, word_b, .. } =>
                                format!("Sento tensione tra \"{}\" e \"{}\"", word_a, word_b),
                            ThoughtData::GapData { .. } =>
                                format!("La regione {} ha pochi contenuti", t.fractal_names.join(", ")),
                            ThoughtData::MissingBridgeData { .. } =>
                                format!("Non capisco il legame tra {} e {}",
                                    t.fractal_names.first().unwrap_or(&String::new()),
                                    t.fractal_names.last().unwrap_or(&String::new())),
                            _ => format!("{:?}", t.kind),
                        };
                        InnerDialogueItem {
                            id: i,
                            text,
                            category: cat.to_string(),
                            strength: t.strength,
                            detail: serde_json::json!({
                                "fractal_names": t.fractal_names,
                                "words": t.words,
                            }),
                        }
                    })
                    .collect();

                // Domande di curiosità
                let questions = engine.ask();
                let question_items: Vec<InnerDialogueItem> = questions.iter()
                    .enumerate()
                    .map(|(i, q)| InnerDialogueItem {
                        id: i,
                        text: q.text.clone(),
                        category: "domanda".to_string(),
                        strength: q.urgency,
                        detail: serde_json::json!({
                            "question_type": format!("{:?}", q.question_type),
                        }),
                    })
                    .collect();

                // Proposizioni (ragionamenti attivi)
                let prop_items: Vec<InnerDialogueItem> = engine.last_propositions.iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let via_text = p.via.as_ref()
                            .map(|v| format!(" (via {})", v))
                            .unwrap_or_default();
                        InnerDialogueItem {
                            id: i,
                            text: format!("{} {} {}{}", p.subject, p.relation.copula(), p.object, via_text),
                            category: if p.hops > 1 { "inferenza".to_string() } else { "proposizione".to_string() },
                            strength: p.strength,
                            detail: serde_json::json!({
                                "hops": p.hops,
                                "subject": p.subject,
                                "relation": p.relation.copula(),
                                "object": p.object,
                                "via": p.via,
                            }),
                        }
                    })
                    .collect();

                let _ = reply.send(InnerDialogueDto {
                    thoughts: thought_items,
                    questions: question_items,
                    propositions: prop_items,
                });
            }

            EngineCommand::RespondToInsight { item_type, item_id, response, action, reply } => {
                let effect = match action.as_str() {
                    "confirm" => {
                        // Insegna la risposta
                        engine.teach(&response);
                        // Se è una proposizione, cerca il simplesso corrispondente e boost
                        if item_type == "proposition" || item_type == "inferenza" {
                            if let Some(prop) = engine.last_propositions.get(item_id) {
                                let fid_s = engine.lexicon.get(&prop.subject)
                                    .and_then(|p| p.dominant_fractal())
                                    .map(|(fid, _)| fid);
                                let fid_o = engine.lexicon.get(&prop.object)
                                    .and_then(|p| p.dominant_fractal())
                                    .map(|(fid, _)| fid);
                                if let (Some(a), Some(b)) = (fid_s, fid_o) {
                                    let mut verts = vec![a, b];
                                    verts.sort();
                                    if let Some(sid) = engine.complex.find_simplex_with_vertices(&verts) {
                                        if let Some(s) = engine.complex.get_mut(sid) {
                                            s.persistence = (s.persistence + 0.1).min(1.0);
                                            s.activate(0.2);
                                        }
                                    }
                                }
                            }
                        }
                        format!("Confermato e insegnato: \"{}\"", response)
                    }
                    "deny" => {
                        engine.teach(&response);
                        if item_type == "proposition" || item_type == "inferenza" {
                            if let Some(prop) = engine.last_propositions.get(item_id) {
                                let fid_s = engine.lexicon.get(&prop.subject)
                                    .and_then(|p| p.dominant_fractal())
                                    .map(|(fid, _)| fid);
                                let fid_o = engine.lexicon.get(&prop.object)
                                    .and_then(|p| p.dominant_fractal())
                                    .map(|(fid, _)| fid);
                                if let (Some(a), Some(b)) = (fid_s, fid_o) {
                                    let mut verts = vec![a, b];
                                    verts.sort();
                                    if let Some(sid) = engine.complex.find_simplex_with_vertices(&verts) {
                                        if let Some(s) = engine.complex.get_mut(sid) {
                                            s.persistence = (s.persistence - 0.1).max(0.0);
                                            s.plasticity = (s.plasticity + 0.1).min(1.0);
                                        }
                                    }
                                }
                            }
                        }
                        format!("Negato e insegnato: \"{}\"", response)
                    }
                    _ => {
                        // "elaborate" o qualsiasi altro — insegna e basta
                        engine.teach(&response);
                        format!("Elaborazione insegnata: \"{}\"", response)
                    }
                };
                let _ = reply.send(RespondResult { success: true, effect });
            }

            EngineCommand::DeleteEdge { subject, relation, object, reply } => {
                use crate::topology::relation::RelationType;
                let success = if let Some(rel) = RelationType::from_str(&relation) {
                    engine.kg.remove_edge(&subject, rel, &object);
                    // Aggiorna anche la word_topology: rimuovi arco se presente
                    engine.word_topology.remove_edge_between(&subject, &object);
                    true
                } else {
                    false
                };
                let _ = reply.send(success);
            }

            EngineCommand::PatchEdgeConfidence { subject, relation, object, confidence, reply } => {
                use crate::topology::relation::RelationType;
                let success = if let Some(rel) = RelationType::from_str(&relation) {
                    engine.kg.update_confidence(&subject, rel, &object, confidence)
                } else {
                    false
                };
                let _ = reply.send(success);
            }

            EngineCommand::ConvReceive { conv_id: _, message, reply } => {
                engine.receive(&message);
                let response = engine.generate_willed();
                let _ = reply.send(response.text);
            }

            // ── Biennale ─────────────────────────────────────────────────────
            EngineCommand::GetBiennaleField { reply } => {
                let dto = build_biennale_field(&engine);
                let _ = reply.send(dto);
            }
            EngineCommand::GetBiennaleWord { word, reply } => {
                let dto = build_biennale_word(&engine, &word);
                let _ = reply.send(dto);
            }
            EngineCommand::GetBiennaleJourney { from, to, reply } => {
                let dto = build_biennale_journey(&engine, &from, &to);
                let _ = reply.send(dto);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Conversioni engine → DTO
// ═══════════════════════════════════════════════════════════════

fn build_snapshot(engine: &mut PrometeoTopologyEngine) -> StateSnapshot {
    let vital = engine.vital_state();
    let active = engine.active_fractals();
    let report = engine.report();

    let locus = if let Some((name, horizon)) = engine.where_am_i() {
        let trail: Vec<String> = engine.locus.trail.iter()
            .filter_map(|&fid| engine.registry.get(fid).map(|f| f.name.clone()))
            .collect();
        let sub_pos: Vec<SubDimDto> = engine.locus.sub_position.iter()
            .map(|(dim, &val)| SubDimDto { dim_index: dim.index() as u8, value: val })
            .collect();
        let visible: Vec<VisibleFractalDto> = engine.what_i_see().iter()
            .map(|(name, vis)| VisibleFractalDto { name: name.clone(), visibility: *vis })
            .collect();
        Some(LocusDto {
            fractal_name: name,
            fractal_id: engine.locus.position.unwrap_or(0),
            horizon,
            trail,
            sub_position: sub_pos,
            visible,
        })
    } else {
        None
    };

    // Firma campo: media pesata delle attivazioni
    let field_sig = engine.locus.full_position(&engine.registry)
        .map(|p| p.values().to_vec())
        .unwrap_or_else(|| vec![0.5; 8]);

    let (dream_phase, dream_depth) = match engine.dream.phase {
        SleepPhase::Awake => ("Awake".to_string(), 0.0),
        SleepPhase::WakefulDream { depth } => ("WakefulDream".to_string(), depth),
        SleepPhase::LightSleep { depth } => ("LightSleep".to_string(), depth),
        SleepPhase::DeepSleep { depth } => ("DeepSleep".to_string(), depth),
        SleepPhase::REM { depth } => ("REM".to_string(), depth),
    };

    StateSnapshot {
        vital: VitalDto {
            activation: vital.activation,
            saturation: vital.saturation,
            curiosity: vital.curiosity,
            fatigue: vital.fatigue,
            tension: match vital.tension {
                TensionState::Calm => "Calm",
                TensionState::Alert => "Alert",
                TensionState::Tense => "Tense",
                TensionState::Overloaded => "Overloaded",
            }.to_string(),
        },
        active_fractals: active.iter()
            .map(|(name, act)| FractalActiveDto { name: name.clone(), activation: *act })
            .collect(),
        locus,
        dream_phase,
        dream_depth,
        report: ReportDto {
            fractal_count: report.fractal_count,
            simplex_count: report.simplex_count,
            max_dimension: report.max_dimension,
            connected_components: report.connected_components,
            memory_stm: report.stm_count,
            memory_mtm: report.mtm_count,
            memory_ltm: report.ltm_count,
            dream_cycles: report.dream_cycles,
            total_perturbations: report.total_perturbations,
            vocabulary_size: report.vocabulary_size,
            emergent_dimensions: report.emergent_dimensions,
        },
        field_signature: field_sig,
    }
}

fn build_topology(engine: &PrometeoTopologyEngine) -> TopologyDto {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen_edges: HashSet<(u32, u32)> = HashSet::new();

    let bootstrap_ids: HashSet<u32> = [0u32, 1, 2, 3, 4, 5].into_iter().collect();

    for (&id, fractal) in engine.registry.iter() {
        let simplex_count = engine.complex.simplices_of(id).len();
        let activation: f64 = engine.complex.simplices_of(id)
            .iter()
            .filter_map(|sid| engine.complex.get(*sid))
            .map(|s| s.current_activation)
            .sum::<f64>();

        nodes.push(TopologyNode {
            id,
            name: fractal.name.clone(),
            activation: activation.min(1.0),
            is_locus: engine.locus.position == Some(id),
            is_bootstrap: bootstrap_ids.contains(&id),
            simplex_count,
        });
    }

    // Archi dai simplessi
    for (_, simplex) in engine.complex.iter() {
        let strength = simplex.shared_faces.iter()
            .map(|f| f.strength)
            .sum::<f64>()
            .min(1.0)
            .max(0.1);

        for i in 0..simplex.vertices.len() {
            for j in (i + 1)..simplex.vertices.len() {
                let a = simplex.vertices[i];
                let b = simplex.vertices[j];
                let edge = if a < b { (a, b) } else { (b, a) };
                if seen_edges.insert(edge) {
                    edges.push(TopologyEdge {
                        source: a,
                        target: b,
                        strength,
                    });
                }
            }
        }
    }

    TopologyDto { nodes, edges }
}

fn build_universe(engine: &PrometeoTopologyEngine) -> UniverseDto {
    // Mappa attivazioni correnti da PF1 (l'unico substrato di attivazione runtime)
    let act_map: std::collections::HashMap<String, f64> = engine.pf_activation
        .hot_words(&engine.pf_field, 2000)
        .into_iter()
        .map(|(w, a)| (w, a as f64))
        .collect();

    // Pre-calcola word_count per frattale: quante parole hanno questo frattale come dominante
    let mut fractal_word_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for (_, pattern) in engine.lexicon.patterns_iter() {
        if let Some((&dominant_fid, &max_aff)) = pattern.fractal_affinities.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            if max_aff >= 0.01 {
                *fractal_word_counts.entry(dominant_fid).or_insert(0) += 1;
            }
        }
    }

    // Frattali
    let fractals: Vec<UniverseFractal> = engine.registry.iter().map(|(&id, fractal)| {
        let activation: f64 = engine.complex.simplices_of(id)
            .iter()
            .filter_map(|sid| engine.complex.get(*sid))
            .map(|s| s.current_activation)
            .sum::<f64>();
        let lower = (id / 8) as u8;
        let upper = (id % 8) as u8;
        UniverseFractal {
            id,
            name: fractal.name.clone(),
            activation: activation.min(1.0),
            is_bootstrap: lower == upper,
            lower,
            upper,
            word_count: fractal_word_counts.get(&id).copied().unwrap_or(0),
        }
    }).collect();

    // Tutte le parole con affinità assegnata (no troncamento)
    let words: Vec<UniverseWord> = engine.lexicon.patterns_iter()
        .filter_map(|(_, pattern)| {
            let (dominant_fractal, max_aff) = pattern.fractal_affinities.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(&k, &v)| (k, v))
                .unwrap_or((0, 0.0));
            if max_aff < 0.01 { return None; } // scarta parole senza affinità frattale
            let activation = act_map.get(&pattern.word).copied().unwrap_or(0.0);
            Some(UniverseWord {
                w: pattern.word.clone(),
                f: dominant_fractal,
                s: (pattern.stability.min(1.0) * 100.0) as u8,
                a: (activation.min(1.0) * 100.0) as u8,
                a1: (max_aff.min(1.0) * 100.0) as u8,
            })
        })
        .collect();

    UniverseDto { fractals, words }
}

fn build_word_neighbors(engine: &PrometeoTopologyEngine, word: &str) -> WordNeighborsDto {
    let fractal_id = engine.lexicon.get(word)
        .and_then(|p| p.fractal_affinities.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&k, _)| k))
        .unwrap_or(0);

    let neighbors = if let Some(id) = engine.word_topology.word_id(word) {
        let adj = engine.word_topology.adjacency_list(id);
        let mut nbrs: Vec<WordNeighborDto> = adj.iter()
            .filter_map(|&nid| {
                let name = engine.word_topology.word_name(nid)?;
                let weight = engine.word_topology.edge_weight_between(word, name)?;
                let fid = engine.lexicon.get(name)
                    .and_then(|p| p.fractal_affinities.iter()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(&k, _)| k))
                    .unwrap_or(0);
                Some(WordNeighborDto { word: name.to_string(), weight, fractal_id: fid })
            })
            .collect();
        nbrs.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        nbrs.truncate(16);
        nbrs
    } else {
        Vec::new()
    };

    WordNeighborsDto { word: word.to_string(), fractal_id, neighbors }
}

fn build_word_detail(engine: &PrometeoTopologyEngine, word: &str) -> WordDetailDto {
    use crate::topology::relation::RelationType;

    let pattern = engine.lexicon.get(word);

    let (fractal_id, stability, exposure) = pattern
        .map(|p| {
            let dominant = p.fractal_affinities.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(&k, _)| k)
                .unwrap_or(0);
            (dominant, p.stability, p.exposure_count)
        })
        .unwrap_or((0, 0.0, 0));

    let fractal_name = engine.registry.get(fractal_id)
        .map(|f| f.name.clone())
        .unwrap_or_default();

    // Firma 8D dalla signature del pattern
    let firma_8d = pattern
        .map(|p| *p.signature.values())
        .unwrap_or([0.5; 8]);

    // Profilo Octalysis derivato dalla firma 8D
    let octalysis = OctalysisDto::from_firma(&firma_8d);

    // Top 5 affinità frattali
    let mut top_affinities: Vec<WordAffinityDto> = pattern
        .map(|p| {
            let mut aff: Vec<(u32, f64)> = p.fractal_affinities.iter()
                .map(|(&k, &v)| (k, v))
                .collect();
            aff.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            aff.truncate(5);
            aff.into_iter().map(|(fid, val)| WordAffinityDto {
                fractal_id: fid,
                fractal_name: engine.registry.get(fid).map(|f| f.name.clone()).unwrap_or_default(),
                value: val,
            }).collect()
        })
        .unwrap_or_default();
    top_affinities.retain(|a| a.value > 0.01);

    // Archi KG uscenti — con nome italiano, colore e via
    let kg_out: Vec<KgEdgeDto> = engine.kg.all_outgoing_full(word)
        .into_iter()
        .map(|(rel, target, conf, via)| KgEdgeDto {
            relation: rel.as_str().to_string(),
            nome: rel.nome().to_string(),
            colore: rel.colore().to_string(),
            target: target.to_string(),
            confidence: conf,
            via: via.map(|s| s.to_string()),
        })
        .collect();

    // Archi KG entranti
    let kg_in: Vec<KgEdgeDto> = engine.kg.all_incoming(word)
        .into_iter()
        .map(|(rel, subject, conf)| KgEdgeDto {
            relation: rel.as_str().to_string(),
            nome: rel.nome().to_string(),
            colore: rel.colore().to_string(),
            target: subject.to_string(),
            confidence: conf,
            via: None,
        })
        .collect();

    // Co-occorrenze statistiche filtrate: peso > 0.25, non già nel KG
    let statistical: Vec<WordNeighborDto> = if let Some(id) = engine.word_topology.word_id(word) {
        let adj = engine.word_topology.adjacency_list(id);
        let mut nbrs: Vec<WordNeighborDto> = adj.iter()
            .filter_map(|&nid| {
                let name = engine.word_topology.word_name(nid)?;
                let weight = engine.word_topology.edge_weight_between(word, name)?;
                if weight < 0.25 { return None; }
                if engine.kg.has_any_edge(word, name) { return None; }
                let fid = engine.lexicon.get(name)
                    .and_then(|p| p.fractal_affinities.iter()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(&k, _)| k))
                    .unwrap_or(0);
                Some(WordNeighborDto { word: name.to_string(), weight, fractal_id: fid })
            })
            .collect();
        nbrs.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        nbrs.truncate(8);
        nbrs
    } else {
        Vec::new()
    };

    WordDetailDto {
        word: word.to_string(),
        stability,
        exposure,
        fractal_id,
        fractal_name,
        firma_8d,
        octalysis,
        top_affinities,
        kg_out,
        kg_in,
        statistical,
    }
}

fn add_word_connect(
    engine: &mut PrometeoTopologyEngine,
    from: &str,
    relation: &str,
    to: &str,
    via: Option<String>,
    confidence: Option<f32>,
) -> bool {
    use crate::topology::relation::{RelationType, TypedEdge, EdgeSource};

    let rel = match RelationType::from_str(relation) {
        Some(r) => r,
        None => return false,
    };

    let edge = TypedEdge::new(from, rel, to)
        .with_source(EdgeSource::UserTaught)
        .with_via(via)
        .with_confidence(confidence.unwrap_or(1.0));
    engine.kg.add_edge(edge);

    // Propaga l'arco anche nella word_topology come arco KG
    engine.word_topology.add_edge_from_kg(from, to, rel);

    true
}

fn build_word_list(
    engine: &PrometeoTopologyEngine,
    query: &str,
    offset: usize,
    limit: usize,
    sort: &str,
) -> crate::web::state::WordListDto {
    use crate::web::state::{WordListDto, WordListItemDto};
    let q = query.to_lowercase();
    let mut items: Vec<WordListItemDto> = engine.lexicon.iter()
        .filter(|(word, _)| q.is_empty() || word.contains(q.as_str()))
        .map(|(word, pat)| {
            let dominant = pat.fractal_affinities.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(&k, _)| k)
                .unwrap_or(0);
            let fractal_name = engine.registry.get(dominant)
                .map(|f| f.name.clone())
                .unwrap_or_default();
            WordListItemDto {
                word: word.to_string(),
                stability: pat.stability,
                exposure: pat.exposure_count,
                fractal_name,
                out_degree: engine.kg.out_degree(word),
                in_degree: engine.kg.in_degree(word),
            }
        })
        .collect();
    match sort {
        "alpha_desc" => items.sort_by(|a, b| b.word.cmp(&a.word)),
        "out_asc"    => items.sort_by(|a, b| a.out_degree.cmp(&b.out_degree)),
        "out_desc"   => items.sort_by(|a, b| b.out_degree.cmp(&a.out_degree)),
        "in_asc"     => items.sort_by(|a, b| a.in_degree.cmp(&b.in_degree)),
        "in_desc"    => items.sort_by(|a, b| b.in_degree.cmp(&a.in_degree)),
        "stab_asc"   => items.sort_by(|a, b| a.stability.partial_cmp(&b.stability).unwrap_or(std::cmp::Ordering::Equal)),
        "stab_desc"  => items.sort_by(|a, b| b.stability.partial_cmp(&a.stability).unwrap_or(std::cmp::Ordering::Equal)),
        _            => items.sort_by(|a, b| a.word.cmp(&b.word)), // alpha_asc default
    }
    let total = items.len();
    let words = items.into_iter().skip(offset).take(limit).collect();
    WordListDto { words, total }
}

fn build_categories(
    engine: &PrometeoTopologyEngine,
    relation: &str,
    min_children: usize,
    query: &str,
) -> crate::web::state::CategoriesDto {
    use crate::topology::relation::RelationType;
    use crate::web::state::{CategoriesDto, CategoryItemDto};
    let rel = RelationType::from_str(relation).unwrap_or(RelationType::IsA);
    let q = query.to_lowercase();
    let mut cats: Vec<CategoryItemDto> = engine.kg.categories_for(rel, min_children)
        .into_iter()
        .filter(|w| q.is_empty() || w.contains(q.as_str()))
        .map(|word| {
            let children = engine.kg.query_subjects(&word, rel);
            let sample: Vec<String> = children.iter().take(10).map(|s| s.to_string()).collect();
            CategoryItemDto {
                word: word.clone(),
                children_count: children.len(),
                sample_children: sample,
            }
        })
        .collect();
    cats.sort_by(|a, b| b.children_count.cmp(&a.children_count));
    let total = cats.len();
    CategoriesDto { categories: cats, total }
}

fn build_concept(engine: &PrometeoTopologyEngine, word: &str) -> ConceptDto {
    use crate::topology::inference::InferenceEngine;
    use crate::topology::relation::RelationType;

    let inference = InferenceEngine::new(&engine.kg);

    let type_chain = inference.type_chain(word);
    let has = inference.what_has(word);
    let does = inference.what_does(word);
    let causes = inference.what_causes(word);
    let similar = inference.similar_to(word);
    let opposites = inference.opposites(word);
    let part_of = inference.part_of_what(word);
    // Istanze dirette: chi è IS_A questa parola (inverso)
    let instances: Vec<String> = engine.kg.query_subjects(word, RelationType::IsA)
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    // Densità ontologica: quante parole nel lessico condividono almeno un IS_A antenato con questa
    let my_ancestors: std::collections::HashSet<String> = type_chain.iter().cloned().collect();
    let ontology_density = if my_ancestors.is_empty() {
        0
    } else {
        engine.lexicon.patterns_iter()
            .filter(|(_, p)| p.word != word)
            .filter(|(_, p)| {
                inference.type_chain(&p.word).into_iter().any(|a| my_ancestors.contains(&a))
            })
            .count()
    };

    ConceptDto {
        word: word.to_string(),
        definition: inference.define(word),
        type_chain,
        instances,
        has,
        does,
        causes,
        similar,
        opposites,
        part_of,
        ontology_density,
    }
}

fn build_self_dto(engine: &PrometeoTopologyEngine) -> SelfDto {
    let beliefs: Vec<BeliefDto> = engine.self_model.beliefs.iter()
        .filter(|b| b.confidence > 0.1 || b.innate)
        .map(|b| BeliefDto {
            claim: b.claim.clone(),
            anchor_concepts: b.anchor_concepts.clone(),
            confidence: b.confidence,
            reinforcement_count: b.reinforcement_count,
            innate: b.innate,
        })
        .collect();

    let mut values: Vec<ValueDto> = engine.self_model.values.iter()
        .map(|v| ValueDto {
            name: v.name.clone(),
            weight: v.weight,
            associated_words: v.associated_words.clone(),
            innate: v.innate,
            activation_count: v.activation_count,
        })
        .collect();
    values.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));

    let uncertainties = engine.self_model.uncertainties.iter()
        .filter(|u| u.tension > 0.1)
        .map(|u| UncertaintyDto {
            topic: u.topic.clone(),
            tension: u.tension,
            emergence_count: u.emergence_count,
        })
        .collect();

    // Credenze attive: quelle i cui anchor_concepts si sovrappongono con le parole dell'ultimo input
    let input_words: HashSet<&str> = engine.last_input_words.iter().map(|s| s.as_str()).collect();
    let active_beliefs: Vec<ActiveBeliefDto> = engine.self_model.beliefs.iter()
        .filter(|b| b.confidence > 0.3)
        .filter_map(|b| {
            let overlap: Vec<String> = b.anchor_concepts.iter()
                .filter(|c| input_words.contains(c.as_str()))
                .cloned()
                .collect();
            if overlap.is_empty() { return None; }
            let influence = b.confidence * 0.05 * overlap.len() as f64;
            Some(ActiveBeliefDto {
                claim: b.claim.clone(),
                confidence: b.confidence,
                activated_words: overlap,
                influence_strength: influence,
            })
        })
        .collect();

    // Traccia influenza
    let mut influence_trace = Vec::new();
    for ab in &active_beliefs {
        influence_trace.push(format!(
            "Credenza \"{:.40}\" (conf {:.0}%) attiva su [{}] → boost {:.3}",
            ab.claim, ab.confidence * 100.0,
            ab.activated_words.join(", "),
            ab.influence_strength
        ));
    }
    // Valori attivi
    let boosts = engine.self_model.field_boosts(&engine.last_input_words);
    for (word, strength) in boosts.iter().take(5) {
        if *strength > 0.01 {
            influence_trace.push(format!("Valore → \"{}\" boost {:.3}", word, strength));
        }
    }

    SelfDto {
        beliefs,
        values,
        uncertainties,
        interaction_count: engine.self_model.interaction_count,
        active_beliefs,
        belief_influence_trace: influence_trace,
    }
}

fn build_episode_dtos(episodes: &[&crate::topology::semantic_episode::SemanticEpisode]) -> Vec<EpisodeDto> {
    episodes.iter().map(|ep| EpisodeDto {
        id: ep.id,
        timestamp: ep.timestamp,
        key_concepts: ep.key_concepts.clone(),
        dominant_fractals: ep.dominant_fractals.iter().map(|(id, name, act)| EpisodeFractalDto {
            id: *id,
            name: name.clone(),
            activation: *act,
        }).collect(),
        summary: ep.summary.clone(),
        stance: ep.stance.clone(),
        intention: ep.intention.clone(),
        active_values: ep.active_values.clone(),
        field_energy: ep.field_energy,
    }).collect()
}

fn build_navigation(engine: &PrometeoTopologyEngine, from: &str, to: &str) -> Option<NavigationDto> {
    let from_id = engine.find_fractal(from)?;
    let to_id = engine.find_fractal(to)?;
    let path = engine.navigate(from_id, to_id)?;

    Some(NavigationDto {
        from_name: engine.registry.get(from_id)?.name.clone(),
        to_name: engine.registry.get(to_id)?.name.clone(),
        steps: path.steps.iter().map(|s| NavStepDto {
            fractal_name: s.fractal_name.clone(),
            shared_structures: s.shared_structures.clone(),
            cumulative_cost: s.cumulative_cost,
        }).collect(),
        total_cost: path.total_cost,
        explanation: path.explanation,
    })
}

fn intention_name(i: &crate::topology::will::Intention) -> &'static str {
    use crate::topology::will::Intention;
    match i {
        Intention::Express { .. }  => "Express",
        Intention::Explore { .. }  => "Explore",
        Intention::Question { .. } => "Question",
        Intention::Remember { .. } => "Remember",
        Intention::Withdraw { .. } => "Withdraw",
        Intention::Reflect        => "Reflect",
        Intention::Dream { .. }   => "Dream",
        Intention::Instruct { .. } => "Instruct",
    }
}

fn build_will(engine: &mut PrometeoTopologyEngine) -> WillDto {
    use crate::topology::dream::SleepPhase;

    // Copia i dati dal will per evitare borrow conflict con vital_state()
    let will_data: Option<(String, f64, Vec<UndercurrentDto>, [usize; 2])> =
        engine.current_will().map(|will| {
            let name = intention_name(&will.intention).to_string();
            let mut under: Vec<UndercurrentDto> = will.undercurrents.iter()
                .map(|(i, p)| UndercurrentDto {
                    name: intention_name(i).to_string(),
                    pressure: *p,
                })
                .collect();
            under.sort_by(|a, b| b.pressure.partial_cmp(&a.pressure).unwrap_or(std::cmp::Ordering::Equal));
            (name, will.drive, under, will.codon)
        });

    let (intention, drive, undercurrents, codon, trigger_chain, forecast) = if let Some((name, drive, under, codon)) = will_data {
        // Trigger chain: ricostruisci il perché dall'engine state
        let mut triggers = Vec::new();
        let vital = engine.vital_state();
        if vital.curiosity > 0.3 {
            triggers.push(TriggerDto { cause: format!("curiosità {:.0}%", vital.curiosity * 100.0), value: vital.curiosity });
        }
        if vital.activation > 0.4 {
            triggers.push(TriggerDto { cause: format!("attivazione {:.0}%", vital.activation * 100.0), value: vital.activation });
        }
        if vital.fatigue > 0.3 {
            triggers.push(TriggerDto { cause: format!("fatica {:.0}%", vital.fatigue * 100.0), value: vital.fatigue });
        }
        if !engine.last_unknown_words.is_empty() {
            triggers.push(TriggerDto {
                cause: format!("{} parole ignote ({})", engine.last_unknown_words.len(),
                    engine.last_unknown_words.iter().take(3).cloned().collect::<Vec<_>>().join(", ")),
                value: engine.last_unknown_words.len() as f64 * 0.2,
            });
        }
        let topic_cont = engine.narrative_self.topic_continuity;
        if topic_cont > 0.6 {
            triggers.push(TriggerDto { cause: format!("continuità tematica {:.0}%", topic_cont * 100.0), value: topic_cont });
        } else if topic_cont < 0.3 {
            triggers.push(TriggerDto { cause: format!("tema nuovo (continuità {:.0}%)", topic_cont * 100.0), value: 1.0 - topic_cont });
        }

        // Valori SelfModel che influenzano la volontà
        for (val_name, weight) in engine.self_model.top_values(3) {
            if weight > 0.3 {
                triggers.push(TriggerDto {
                    cause: format!("valore: {} ({:.0}%)", val_name, weight * 100.0),
                    value: weight,
                });
            }
        }

        // Frattali attivi come contesto semantico
        let hot = engine.pf_activation.hot_words(&engine.pf_field, 5);
        if !hot.is_empty() {
            let campo: Vec<String> = hot.iter().map(|(w, _)| w.clone()).collect();
            triggers.push(TriggerDto {
                cause: format!("campo attivo: {}", campo.join(", ")),
                value: hot.first().map_or(0.0, |(_, a)| *a as f64).min(1.0),
            });
        }

        triggers.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal));

        // Forecast: seconda intenzione più forte — con contesto
        let forecast = under.get(1).map(|u| {
            if u.pressure > 0.3 {
                format!("{} (pressione {:.0}%)", u.name, u.pressure * 100.0)
            } else {
                u.name.clone()
            }
        });

        (name, drive, under, codon, triggers, forecast)
    } else {
        ("Dream".to_string(), 0.0, Vec::new(), [0usize, 1usize], Vec::new(), None)
    };

    let dream_phase = match engine.dream.phase {
        SleepPhase::Awake                => "Awake",
        SleepPhase::WakefulDream { .. }  => "WakefulDream",
        SleepPhase::LightSleep { .. }    => "LightSleep",
        SleepPhase::DeepSleep { .. }     => "DeepSleep",
        SleepPhase::REM { .. }           => "REM",
    }.to_string();

    WillDto { intention, drive, undercurrents, dream_phase, codon, trigger_chain, forecast }
}

fn build_compounds(engine: &PrometeoTopologyEngine) -> Vec<CompoundDto> {
    engine.compound_states().iter().map(|c| {
        let fractal_names: Vec<String> = c.fractals.iter()
            .filter_map(|&fid| engine.registry.get(fid).map(|f| f.name.clone()))
            .collect();
        CompoundDto {
            name: c.name.to_string(),
            fractals: fractal_names,
            strength: c.strength,
            order: c.order,
        }
    }).collect()
}

fn build_word_field(engine: &PrometeoTopologyEngine) -> WordFieldDto {
    let top = engine.pf_activation.hot_words(&engine.pf_field, 20)
        .into_iter().map(|(word, act)| (word, act as f64)).collect::<Vec<_>>();
    let top_words = top.iter().map(|(word, activation)| {
        // Frattale primario: l'affinita piu alta
        let fractal = engine.lexicon.get(word.as_str())
            .and_then(|p| {
                p.fractal_affinities.iter()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .and_then(|(&fid, _)| engine.registry.get(fid).map(|f| f.name.clone()))
            })
            .unwrap_or_else(|| "?".to_string());
        WordActivationDto {
            word: word.clone(),
            activation: *activation,
            fractal,
        }
    }).collect();

    WordFieldDto {
        top_words,
        total_energy: engine.pf_activation.field_energy() as f64,
        vertex_count: engine.word_topology.vertex_count(),
        edge_count: engine.word_topology.edge_count(),
    }
}

fn build_phase(engine: &PrometeoTopologyEngine, word_a: &str, word_b: &str) -> PhaseDto {
    use std::f64::consts::PI;

    let phase_rad = engine.word_topology.edge_phase(word_a, word_b)
        .unwrap_or(PI / 2.0);
    let phase_deg = phase_rad.to_degrees();
    let cos_value = phase_rad.cos();

    let label = if phase_deg < 60.0 {
        "Risonanza"
    } else if phase_deg < 120.0 {
        "Tensione"
    } else {
        "Opposizione"
    }.to_string();

    let (co_affirmed, co_negated) = engine.lexicon.get(word_a)
        .map(|p| (
            p.co_affirmed.get(word_b).copied().unwrap_or(0),
            p.co_negated.get(word_b).copied().unwrap_or(0),
        ))
        .unwrap_or((0, 0));

    PhaseDto {
        word_a: word_a.to_string(),
        word_b: word_b.to_string(),
        phase_rad,
        phase_deg,
        label,
        cos_value,
        co_affirmed,
        co_negated,
    }
}

fn build_tension(engine: &PrometeoTopologyEngine, pole_a: &str, pole_b: &str) -> Vec<TensionWordDto> {
    engine.lexicon.find_tension_words(pole_a, pole_b)
        .iter()
        .take(10)
        .map(|tw| TensionWordDto {
            word: tw.word.clone(),
            position: tw.position,
            distance_to_a: tw.distance_to_axis,
            distance_to_b: tw.distance_to_axis,
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════
// Biennale build functions
// ═══════════════════════════════════════════════════════════════

/// Calcola la posizione 2D di una parola dalla sua firma 8D.
/// x = valenza (dim[1]) jittered con confine (dim[0])
/// y = agency  (dim[6]) jittered con intensità (dim[2])
#[inline]
fn biennale_pos(sig: &[f64; 8]) -> (f32, f32) {
    let x = ((sig[1] + (sig[0] - 0.5) * 0.2) as f32).clamp(0.0, 1.0);
    let y = ((sig[6] + (sig[2] - 0.5) * 0.2) as f32).clamp(0.0, 1.0);
    (x, y)
}

fn build_biennale_field(engine: &PrometeoTopologyEngine) -> BiennaleFieldDto {
    let mut words: Vec<BiennaleWordPos> = engine.lexicon.patterns_iter()
        .filter_map(|(_, pattern)| {
            if pattern.stability < 0.35 { return None; }
            let (dominant_fid, max_aff) = pattern.fractal_affinities.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(&k, &v)| (k, v))
                .unwrap_or((0, 0.0));
            if max_aff < 0.01 { return None; }
            let sig = pattern.signature.values();
            let (x, y) = biennale_pos(sig);
            Some(BiennaleWordPos {
                w: pattern.word.clone(),
                x,
                y,
                f: dominant_fid,
                s: (pattern.stability.min(1.0) * 100.0) as u8,
            })
        })
        .collect();

    // Percentile normalization: distribute x and y uniformly across [0.02, 0.98]
    // so there are no empty voids. Preserves relative order (semantic meaning)
    // but fills the canvas evenly.
    let n = words.len();
    if n > 1 {
        let mut ix: Vec<usize> = (0..n).collect();
        ix.sort_by(|&a, &b| words[a].x.partial_cmp(&words[b].x).unwrap_or(std::cmp::Ordering::Equal));
        for (rank, &idx) in ix.iter().enumerate() {
            words[idx].x = 0.02 + (rank as f32 / (n - 1) as f32) * 0.96;
        }
        let mut iy: Vec<usize> = (0..n).collect();
        iy.sort_by(|&a, &b| words[a].y.partial_cmp(&words[b].y).unwrap_or(std::cmp::Ordering::Equal));
        for (rank, &idx) in iy.iter().enumerate() {
            words[idx].y = 0.02 + (rank as f32 / (n - 1) as f32) * 0.96;
        }
    }

    let fractal_names: Vec<(u32, String)> = engine.registry.iter()
        .map(|(&id, fractal)| (id, fractal.name.clone()))
        .collect();

    BiennaleFieldDto {
        words,
        fractal_names,
        axis_labels: [
            "negativo".to_string(),
            "positivo".to_string(),
            "passivo".to_string(),
            "attivo".to_string(),
        ],
    }
}

fn build_biennale_word(engine: &PrometeoTopologyEngine, word: &str) -> BiennaleWordDto {
    let pattern = engine.lexicon.get(word);

    let (firma, stability, dominant_fid) = if let Some(p) = pattern {
        let fid = p.fractal_affinities.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&k, _)| k)
            .unwrap_or(0);
        (*p.signature.values(), p.stability, fid)
    } else {
        ([0.5f64; 8], 0.0, 0)
    };

    let fractal_name = engine.registry.get(dominant_fid)
        .map(|f| f.name.clone())
        .unwrap_or_default();

    let (wx, wy) = biennale_pos(&firma);

    // Raccoglie vicini KG: uscenti + entranti, filtrati per stabilità >= 0.3
    let mut neighbors: Vec<BiennaleNeighborDto> = Vec::new();
    let mut rel_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    // Archi uscenti
    for (rel, target, conf) in engine.kg.all_outgoing(word) {
        if let Some(tp) = engine.lexicon.get(target) {
            if tp.stability >= 0.3 {
                let rel_str = rel.as_str().to_string();
                let count = rel_counts.entry(rel_str.clone()).or_insert(0);
                if *count < 8 {
                    let tsig = tp.signature.values();
                    let (tx, ty) = biennale_pos(tsig);
                    neighbors.push(BiennaleNeighborDto {
                        w: target.to_string(),
                        rel: rel_str,
                        conf,
                        x: tx,
                        y: ty,
                    });
                    *count += 1;
                }
            }
        }
    }

    // Archi entranti
    for (rel, subject, conf) in engine.kg.all_incoming(word) {
        if let Some(sp) = engine.lexicon.get(subject) {
            if sp.stability >= 0.3 {
                let rel_str = format!("←{}", rel.as_str());
                let count = rel_counts.entry(rel_str.clone()).or_insert(0);
                if *count < 8 {
                    let ssig = sp.signature.values();
                    let (sx, sy) = biennale_pos(ssig);
                    neighbors.push(BiennaleNeighborDto {
                        w: subject.to_string(),
                        rel: rel_str,
                        conf,
                        x: sx,
                        y: sy,
                    });
                    *count += 1;
                }
            }
        }
    }

    BiennaleWordDto {
        word: word.to_string(),
        firma,
        fractal_name,
        stability,
        x: wx,
        y: wy,
        neighbors,
    }
}

fn build_biennale_journey(engine: &PrometeoTopologyEngine, from: &str, to: &str) -> BiennaleJourneyDto {
    use std::collections::{HashMap, VecDeque};

    if from == to {
        // Percorso degenere: un solo nodo
        let (x, y) = engine.lexicon.get(from)
            .map(|p| biennale_pos(p.signature.values()))
            .unwrap_or((0.5, 0.5));
        return BiennaleJourneyDto {
            found: true,
            from: from.to_string(),
            to: to.to_string(),
            path: vec![BiennalePathStepDto { word: from.to_string(), relation: None, x, y }],
        };
    }

    // BFS con max depth 5
    const MAX_DEPTH: usize = 5;

    // parent_map: parola → (parola padre, relazione usata per arrivare qui)
    let mut parent_map: HashMap<String, (String, String)> = HashMap::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();

    visited.insert(from.to_string());
    queue.push_back((from.to_string(), 0));

    let mut found = false;

    'bfs: while let Some((current, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH { continue; }

        // Esplora archi uscenti
        for (rel, neighbor, _conf) in engine.kg.all_outgoing(&current) {
            // Filtra: il vicino deve esistere nel lessico con stabilità >= 0.3
            let ok = engine.lexicon.get(neighbor)
                .map(|p| p.stability >= 0.3)
                .unwrap_or(false);
            if !ok { continue; }
            if visited.contains(neighbor) { continue; }

            parent_map.insert(neighbor.to_string(), (current.clone(), rel.as_str().to_string()));
            visited.insert(neighbor.to_string());

            if neighbor == to {
                found = true;
                break 'bfs;
            }
            queue.push_back((neighbor.to_string(), depth + 1));
        }

        // Esplora anche archi entranti (grafo non orientato per navigazione)
        for (rel, subject, _conf) in engine.kg.all_incoming(&current) {
            let ok = engine.lexicon.get(subject)
                .map(|p| p.stability >= 0.3)
                .unwrap_or(false);
            if !ok { continue; }
            if visited.contains(subject) { continue; }

            parent_map.insert(subject.to_string(), (current.clone(), format!("←{}", rel.as_str())));
            visited.insert(subject.to_string());

            if subject == to {
                found = true;
                break 'bfs;
            }
            queue.push_back((subject.to_string(), depth + 1));
        }
    }

    if !found {
        return BiennaleJourneyDto {
            found: false,
            from: from.to_string(),
            to: to.to_string(),
            path: vec![],
        };
    }

    // Ricostruisci il percorso da `to` a `from`, poi invertilo
    let mut path_words: Vec<(String, Option<String>)> = Vec::new();
    let mut cur = to.to_string();
    loop {
        if let Some((parent, rel)) = parent_map.get(&cur) {
            path_words.push((cur.clone(), Some(rel.clone())));
            cur = parent.clone();
        } else {
            break;
        }
    }
    path_words.push((from.to_string(), None));
    path_words.reverse();

    // Converti in BiennalePathStepDto: la relazione di un passo punta al passo successivo
    // path_words[i].1 è la relazione usata da path_words[i-1] per arrivare a path_words[i]
    // Vogliamo che path[i].relation sia la relazione da i a i+1
    let n = path_words.len();
    let path: Vec<BiennalePathStepDto> = path_words.into_iter().enumerate().map(|(i, (word, _))| {
        // relazione verso il passo successivo: presa dal parent_map del passo i+1
        let relation = if i + 1 < n {
            // Il passo i+1 ha una relazione "come ci sono arrivato"
            // la recuperiamo dal parent_map direttamente
            None // placeholder — verrà ricalcolata sotto
        } else {
            None
        };
        let (x, y) = engine.lexicon.get(&word)
            .map(|p| biennale_pos(p.signature.values()))
            .unwrap_or((0.5, 0.5));
        BiennalePathStepDto { word, relation, x, y }
    }).collect();

    // Ricalcola le relazioni: path[i].relation = come si arriva a path[i+1]
    let mut path_final: Vec<BiennalePathStepDto> = path;
    for i in 0..path_final.len().saturating_sub(1) {
        let next_word = path_final[i + 1].word.clone();
        if let Some((_, rel)) = parent_map.get(&next_word) {
            path_final[i].relation = Some(rel.clone());
        }
    }

    BiennaleJourneyDto {
        found: true,
        from: from.to_string(),
        to: to.to_string(),
        path: path_final,
    }
}
