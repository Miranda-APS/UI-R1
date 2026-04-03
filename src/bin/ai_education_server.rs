/// ai_education_server — Server JSON per educazione via IA
///
/// Espone API JSON per permettere a un'IA esterna di educare Prometeo.
/// Legge comandi da stdin (JSON), esegue azioni, ritorna risultati (JSON).
///
/// Protocollo:
///   Input:  {"action": "teach", "sentence": "io corpo qui"}
///   Output: {"success": true, "new_words": ["corpo"], "known_words": ["io", "qui"]}
///
/// Azioni:
///   - teach: Insegna frase
///   - verify: Insegna + genera comprensione
///   - info: Info su parola
///   - stats: Statistiche
///   - save: Salva stato
///   - quit: Termina
///
/// Uso:
///   cargo run --release --bin ai-education-server

use std::io::{self, BufRead, Write};
use std::path::Path;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
#[serde(tag = "action")]
enum Command {
    #[serde(rename = "teach")]
    Teach { sentence: String },
    
    #[serde(rename = "verify")]
    Verify { sentence: String },
    
    #[serde(rename = "info")]
    Info { word: String },
    
    #[serde(rename = "stats")]
    Stats,
    
    #[serde(rename = "save")]
    Save,
    
    #[serde(rename = "quit")]
    Quit,
}

#[derive(Serialize)]
struct TeachResponse {
    success: bool,
    new_words: Vec<String>,
    known_words: Vec<String>,
    new_count: usize,
    known_count: usize,
}

#[derive(Serialize)]
struct VerifyResponse {
    success: bool,
    taught: bool,
    understanding: String,
    new_words: Vec<String>,
}

#[derive(Serialize)]
struct InfoResponse {
    found: bool,
    word: Option<String>,
    signature: Option<Vec<f64>>,
    stability: Option<f64>,
    exposure_count: Option<u64>,
    dominant_fractal: Option<(u32, f64)>,
}

#[derive(Serialize)]
struct StatsResponse {
    word_count: usize,
    edge_count: usize,
    simplex_count: usize,
    lessons_completed: usize,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Response {
    Teach(TeachResponse),
    Verify(VerifyResponse),
    Info(InfoResponse),
    Stats(StatsResponse),
    Simple { success: bool, message: Option<String> },
}

fn load_or_create_engine(bin_path: &Path) -> PrometeoTopologyEngine {
    if bin_path.exists() {
        match PrometeoState::load_from_binary(bin_path) {
            Ok(state) => {
                let mut engine = PrometeoTopologyEngine::new();
                state.restore_lexicon(&mut engine);
                engine
            }
            Err(_) => PrometeoTopologyEngine::new()
        }
    } else {
        PrometeoTopologyEngine::new()
    }
}

fn save_state(engine: &PrometeoTopologyEngine, bin_path: &Path) -> bool {
    let state = PrometeoState::capture(engine);
    state.save_to_binary(bin_path).is_ok()
}

fn handle_command(
    engine: &mut PrometeoTopologyEngine,
    state_path: &Path,
    cmd: Command,
) -> (Response, bool) {
    match cmd {
        Command::Teach { sentence } => {
            let result = engine.teach(&sentence);
            (
                Response::Teach(TeachResponse {
                    success: true,
                    new_words: result.words_new,
                    known_words: result.words_known,
                    new_count: result.new_count,
                    known_count: result.known_count,
                }),
                false
            )
        }
        
        Command::Verify { sentence } => {
            // Teach + receive + generate
            let teach_result = engine.teach(&sentence);
            let _resp = engine.receive(&sentence);
            let generated = engine.generate_willed();
            
            (
                Response::Verify(VerifyResponse {
                    success: true,
                    taught: true,
                    understanding: generated.text,
                    new_words: teach_result.words_new,
                }),
                false
            )
        }
        
        Command::Info { word } => {
            if let Some(pat) = engine.lexicon.get(&word) {
                (
                    Response::Info(InfoResponse {
                        found: true,
                        word: Some(word),
                        signature: Some(pat.signature.values().to_vec()),
                        stability: Some(pat.stability),
                        exposure_count: Some(pat.exposure_count),
                        dominant_fractal: pat.dominant_fractal(),
                    }),
                    false
                )
            } else {
                (
                    Response::Info(InfoResponse {
                        found: false,
                        word: Some(word),
                        signature: None,
                        stability: None,
                        exposure_count: None,
                        dominant_fractal: None,
                    }),
                    false
                )
            }
        }
        
        Command::Stats => {
            (
                Response::Stats(StatsResponse {
                    word_count: engine.lexicon.word_count(),
                    edge_count: engine.word_topology.edge_count(),
                    simplex_count: engine.complex.count(),
                    lessons_completed: engine.curriculum.lessons_completed.len(),
                }),
                false
            )
        }
        
        Command::Save => {
            let success = save_state(engine, state_path);
            (
                Response::Simple {
                    success,
                    message: if success { Some("Saved".to_string()) } else { Some("Save failed".to_string()) },
                },
                false
            )
        }
        
        Command::Quit => {
            save_state(engine, state_path);
            (
                Response::Simple {
                    success: true,
                    message: Some("Goodbye".to_string()),
                },
                true
            )
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut state_path = Path::new("prometeo_ai_edu.bin").to_path_buf();
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--state" | "-s" if i + 1 < args.len() => {
                state_path = Path::new(&args[i + 1]).to_path_buf();
                i += 2;
            }
            _ => { i += 1; }
        }
    }
    
    // Carica engine
    let mut engine = load_or_create_engine(&state_path);
    
    // Segnala ready
    let ready = serde_json::json!({
        "status": "ready",
        "state_path": state_path.to_string_lossy(),
        "word_count": engine.lexicon.word_count(),
    });
    println!("{}", serde_json::to_string(&ready).unwrap());
    io::stdout().flush().ok();
    
    // Loop: leggi comandi JSON da stdin
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Parse comando
        let cmd: Command = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let error = serde_json::json!({
                    "error": format!("Parse error: {}", e)
                });
                println!("{}", serde_json::to_string(&error).unwrap());
                io::stdout().flush().ok();
                continue;
            }
        };
        
        // Esegui comando
        let (response, should_quit) = handle_command(&mut engine, &state_path, cmd);
        
        // Ritorna risposta JSON
        println!("{}", serde_json::to_string(&response).unwrap());
        io::stdout().flush().ok();
        
        if should_quit {
            break;
        }
    }
}
