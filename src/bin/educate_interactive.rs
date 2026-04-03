/// educate_interactive — Educazione one-to-one con Prometeo
///
/// Modalità interattiva per insegnare frasi direttamente a Prometeo
/// e vedere come l'entità apprende in tempo reale.
///
/// Uso:
///   cargo run --release --bin educate-interactive
///   cargo run --release --bin educate-interactive -- --state custom_state.bin

use std::io::{self, Write};
use std::path::Path;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;

fn load_or_create_engine(bin_path: &Path) -> PrometeoTopologyEngine {
    if bin_path.exists() {
        match PrometeoState::load_from_binary(bin_path) {
            Ok(state) => {
                let mut engine = PrometeoTopologyEngine::new();
                state.restore_lexicon(&mut engine);
                println!("✓ Stato caricato: {} parole, {} archi",
                    engine.lexicon.word_count(),
                    engine.word_topology.edge_count());
                engine
            }
            Err(e) => {
                eprintln!("✗ Errore caricamento: {}", e);
                println!("  Creo nuovo stato...");
                PrometeoTopologyEngine::new()
            }
        }
    } else {
        println!("  Nuovo stato (file non esistente)");
        PrometeoTopologyEngine::new()
    }
}

fn save_state(engine: &PrometeoTopologyEngine, bin_path: &Path) {
    let state = PrometeoState::capture(engine);
    match state.save_to_binary(bin_path) {
        Ok(()) => println!("✓ Stato salvato: {:?}", bin_path),
        Err(e) => eprintln!("✗ Errore salvataggio: {}", e),
    }
}

fn show_word_info(engine: &PrometeoTopologyEngine, word: &str) {
    if let Some(pat) = engine.lexicon.get(word) {
        println!("\n  Parola: {}", word);
        println!("  Firma 8D: {:?}", pat.signature.values());
        println!("  Stabilità: {:.3}", pat.stability);
        println!("  Esposizioni: {}", pat.exposure_count);
        
        // Frattale dominante
        if let Some((fid, aff)) = pat.dominant_fractal() {
            println!("  Frattale: {} (affinità {:.3})", fid, aff);
        }
        
        // Co-occorrenze top 5
        let mut coocs: Vec<_> = pat.co_occurrences.iter().collect();
        coocs.sort_by(|a, b| b.1.cmp(a.1));
        if !coocs.is_empty() {
            println!("  Co-occorrenze:");
            for (w, count) in coocs.iter().take(5) {
                println!("    {} ({})", w, count);
            }
        }
    } else {
        println!("  Parola '{}' non conosciuta", word);
    }
}

fn show_stats(engine: &PrometeoTopologyEngine) {
    println!("\n{}", "═".repeat(60));
    println!("  STATISTICHE PROMETEO");
    println!("{}", "═".repeat(60));
    println!("  Parole lessico:     {}", engine.lexicon.word_count());
    println!("  Archi topologia:    {}", engine.word_topology.edge_count());
    println!("  Simplessi:          {}", engine.complex.count());
    println!("  Lezioni completate: {}", engine.curriculum.lessons_completed.len());
    println!("  Parole apprese:     {}", engine.curriculum.total_words_learned);
    println!("{}", "═".repeat(60));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut state_path = Path::new("prometeo_state.bin").to_path_buf();
    
    // Parse args
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

    println!("{}", "═".repeat(60));
    println!("  PROMETEO — Educazione Interattiva One-to-One");
    println!("{}", "═".repeat(60));
    println!();
    
    // Carica o crea engine
    let mut engine = load_or_create_engine(&state_path);
    let mut sentence_count = 0usize;
    
    println!();
    println!("Comandi disponibili:");
    println!("  - Scrivi una frase per insegnarla");
    println!("  - :info <parola>  — mostra info su una parola");
    println!("  - :stats          — mostra statistiche");
    println!("  - :save           — salva stato");
    println!("  - :lesson <file>  — insegna file lezione");
    println!("  - :quit           — esci e salva");
    println!();
    
    loop {
        print!("\n[Insegna] > ");
        io::stdout().flush().ok();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {},
            Err(e) => {
                eprintln!("Errore lettura: {}", e);
                continue;
            }
        }
        
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        
        // Comandi speciali
        if input.starts_with(':') {
            let parts: Vec<&str> = input[1..].split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "quit" | "exit" | "q" => {
                    println!("\nSalvataggio e uscita...");
                    save_state(&engine, &state_path);
                    println!("Frasi insegnate questa sessione: {}", sentence_count);
                    break;
                }
                "save" | "s" => {
                    save_state(&engine, &state_path);
                }
                "stats" => {
                    show_stats(&engine);
                }
                "info" | "i" if parts.len() > 1 => {
                    let word = parts[1].to_lowercase();
                    show_word_info(&engine, &word);
                }
                "lesson" | "l" if parts.len() > 1 => {
                    let lesson_path = Path::new(parts[1]);
                    if !lesson_path.exists() {
                        println!("✗ File non trovato: {:?}", lesson_path);
                        continue;
                    }
                    
                    println!("Insegno lezione: {:?}", lesson_path);
                    match engine.teach_lesson_file(lesson_path) {
                        Ok(result) => {
                            println!("✓ Lezione completata!");
                            println!("  Parole processate: {}", result.words_processed.len());
                            println!("  Nuove: {} | Conosciute: {}", result.new_count, result.known_count);
                            sentence_count += result.words_processed.len();
                            
                            // Salva dopo ogni lezione
                            save_state(&engine, &state_path);
                        }
                        Err(e) => {
                            println!("✗ Errore: {}", e);
                        }
                    }
                }
                "help" | "h" => {
                    println!("\nComandi:");
                    println!("  :info <parola>   — info su parola");
                    println!("  :stats           — statistiche");
                    println!("  :save            — salva stato");
                    println!("  :lesson <file>   — insegna file lezione");
                    println!("  :quit            — esci");
                }
                _ => {
                    println!("Comando sconosciuto. Usa :help per aiuto.");
                }
            }
            continue;
        }
        
        // Insegna la frase
        let result = engine.teach(input);
        sentence_count += 1;
        
        println!("✓ Appreso! (frase #{})", sentence_count);
        
        if result.new_count > 0 {
            println!("  Parole NUOVE ({}): {}", result.new_count, result.words_new.join(", "));
        }
        if result.known_count > 0 {
            println!("  Parole note ({}): {}", result.known_count, result.words_known.join(", "));
        }
        
        // Mostra frattali coinvolti
        if !result.fractal_affinities.is_empty() {
            let mut sorted = result.fractal_affinities.clone();
            sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let top3: Vec<String> = sorted.iter()
                .take(3)
                .map(|(fid, aff)| format!("{}({:.2})", fid, aff))
                .collect();
            println!("  Frattali: {}", top3.join(", "));
        }
        
        // Auto-save ogni 20 frasi
        if sentence_count % 20 == 0 {
            save_state(&engine, &state_path);
            println!("  [Auto-save]");
        }
    }
    
    println!("\n{}", "═".repeat(60));
    println!("  Sessione completata");
    println!("{}", "═".repeat(60));
}
