/// socratic_educator — Educazione socratica: Prometeo fa domande
///
/// Invece di insegnare passivamente, Prometeo CHIEDE e tu RISPONDI.
/// Questo attiva la curiosità epistemica e permette a Prometeo di
/// guidare il proprio apprendimento verso le lacune che percepisce.
///
/// Ciclo:
/// 1. Prometeo identifica lacune (homology gaps, parole instabili)
/// 2. Prometeo fa una domanda
/// 3. Tu rispondi
/// 4. Prometeo apprende dalla risposta
/// 5. Prometeo verifica comprensione (genera)
/// 6. Tu validi
///
/// Uso:
///   cargo run --release --bin socratic-educator

use std::io::{self, Write};
use std::path::Path;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::curiosity::QuestionType;

fn load_or_create_engine(bin_path: &Path) -> PrometeoTopologyEngine {
    if bin_path.exists() {
        match PrometeoState::load_from_binary(bin_path) {
            Ok(state) => {
                let mut engine = PrometeoTopologyEngine::new();
                state.restore_lexicon(&mut engine);
                println!("✓ Stato caricato: {} parole", engine.lexicon.word_count());
                engine
            }
            Err(e) => {
                eprintln!("✗ Errore caricamento: {}", e);
                // Non creare un nuovo engine vuoto se fallisce il caricamento. Fallisci esplicitamente.
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("✗ Errore: file di stato non trovato: {:?}", bin_path);
        std::process::exit(1);
    }
}

fn save_state(engine: &PrometeoTopologyEngine, bin_path: &Path) {
    let state = PrometeoState::capture(engine);
    state.save_to_binary(bin_path).ok();
}

fn generate_question(engine: &mut PrometeoTopologyEngine) -> Option<String> {
    // Usa il sistema di curiosità per generare domande
    let questions = engine.ask();
    
    if questions.is_empty() {
        return None;
    }
    
    // Prendi la domanda con urgency più alto
    let best = questions.iter()
        .max_by(|a, b| a.urgency.partial_cmp(&b.urgency)
            .unwrap_or(std::cmp::Ordering::Equal))?;
    
    // Formatta la domanda
    let question = match &best.question_type {
        QuestionType::ConceptualGap { vertices } => {
            let names: Vec<String> = vertices.iter()
                .filter_map(|fid| engine.registry.get(*fid).map(|f| f.name.clone()))
                .collect();
            if names.len() >= 2 {
                format!("cosa unifica {}?", names.join(", "))
            } else {
                format!("cosa unifica questi concetti?")
            }
        }
        QuestionType::SparseRegion { fractal } => {
            if let Some(f) = engine.registry.get(*fractal) {
                format!("cosa c'è intorno a {}?", f.name)
            } else {
                format!("cosa c'è intorno al frattale {}?", fractal)
            }
        }
        QuestionType::Isolated { fractal } => {
            if let Some(f) = engine.registry.get(*fractal) {
                format!("a cosa si collega {}?", f.name)
            } else {
                format!("a cosa si collega il frattale {}?", fractal)
            }
        }
        QuestionType::Disconnection { component_a, component_b } => {
            let name_a = engine.registry.get(*component_a).map(|f| f.name.as_str()).unwrap_or("A");
            let name_b = engine.registry.get(*component_b).map(|f| f.name.as_str()).unwrap_or("B");
            format!("come si collegano {} e {}?", name_a, name_b)
        }
    };
    
    Some(question)
}

fn find_unstable_words(engine: &PrometeoTopologyEngine, limit: usize) -> Vec<String> {
    // Trova parole con bassa stabilità (< 0.3) e poche esposizioni (< 5)
    let mut unstable: Vec<(String, f64, usize)> = Vec::new();
    
    for (word, pat) in engine.lexicon.patterns_iter() {
        if pat.stability < 0.3 && pat.exposure_count < 5 && pat.exposure_count > 0 {
            unstable.push((word.to_string(), pat.stability, pat.exposure_count as usize));
        }
    }
    
    // Ordina per stabilità (più instabili prima)
    unstable.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    
    unstable.into_iter()
        .take(limit)
        .map(|(w, _, _)| w)
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut state_path = Path::new("prometeo_topology_state.bin").to_path_buf();
    
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

    println!("{}", "═".repeat(70));
    println!("  PROMETEO — Educazione Socratica");
    println!("{}", "═".repeat(70));
    println!();
    
    let mut engine = load_or_create_engine(&state_path);
    let mut question_count = 0usize;
    let mut answer_count = 0usize;
    
    println!("  Prometeo guida il proprio apprendimento facendo domande.");
    println!("  Tu rispondi, lui apprende, poi verifica la comprensione.");
    println!();
    println!("Comandi: :quit, :stats, :manual (passa a modalità manuale)");
    println!();
    println!("{}", "─".repeat(70));
    
    loop {
        // FASE 1: Prometeo identifica lacune e fa domanda
        println!("\n[Prometeo analizza lacune...]");
        
        // Trova parole instabili
        let unstable = find_unstable_words(&engine, 3);
        
        if !unstable.is_empty() {
            println!("  Parole instabili: {}", unstable.join(", "));
        }
        
        // Genera domanda dal sistema di curiosità
        let question = generate_question(&mut engine);
        
        if let Some(q) = question {
            question_count += 1;
            println!("\n[Prometeo] Domanda #{}: {}", question_count, q);
        } else {
            println!("\n[Prometeo] Non ho domande specifiche.");
            println!("  Vuoi insegnare qualcosa? (scrivi frase o :quit)");
        }
        
        // FASE 2: Tu rispondi
        print!("\n[Tu] Risposta > ");
        io::stdout().flush().ok();
        
        let mut answer = String::new();
        match io::stdin().read_line(&mut answer) {
            Ok(0) => break,
            Ok(_) => {},
            Err(e) => {
                eprintln!("Errore: {}", e);
                continue;
            }
        }
        
        let answer = answer.trim();
        if answer.is_empty() {
            continue;
        }
        
        // Comandi
        if answer.starts_with(':') {
            match answer {
                ":quit" | ":q" => {
                    save_state(&engine, &state_path);
                    println!("\nDomande: {} | Risposte: {}", question_count, answer_count);
                    break;
                }
                ":stats" => {
                    println!("\n{}", "═".repeat(60));
                    println!("  Parole lessico:     {}", engine.lexicon.word_count());
                    println!("  Domande fatte:      {}", question_count);
                    println!("  Risposte date:      {}", answer_count);
                    println!("{}", "═".repeat(60));
                }
                ":manual" => {
                    println!("\n  Modalità manuale: scrivi frasi da insegnare");
                    print!("  > ");
                    io::stdout().flush().ok();
                    let mut manual_input = String::new();
                    if io::stdin().read_line(&mut manual_input).is_ok() {
                        let manual_input = manual_input.trim();
                        if !manual_input.is_empty() {
                            engine.teach(manual_input);
                            println!("  ✓ Appreso");
                        }
                    }
                }
                _ => {
                    println!("  Comando sconosciuto");
                }
            }
            continue;
        }
        
        // FASE 3: Prometeo apprende dalla risposta
        answer_count += 1;
        let teach_result = engine.teach(answer);
        
        println!("  ✓ Risposta appresa");
        if teach_result.new_count > 0 {
            println!("    Nuove parole: {}", teach_result.words_new.join(", "));
        }
        
        // FASE 4: Prometeo verifica comprensione
        println!("\n  [Prometeo verifica comprensione...]");
        let _resp = engine.receive(answer);
        let verification = engine.generate_willed();
        
        println!("  Prometeo verifica: \"{}\"", verification.text);
        
        // FASE 5: Validazione rapida
        print!("  Corretto? (ok/no) > ");
        io::stdout().flush().ok();
        
        let mut validation = String::new();
        if io::stdin().read_line(&mut validation).is_ok() {
            let validation = validation.trim().to_lowercase();
            
            if validation == "ok" || validation == "si" || validation == "sì" {
                println!("  ✓ Comprensione verificata");
                // Rinforza
                engine.teach(&verification.text);
            } else if validation == "no" {
                println!("  ✗ Comprensione errata");
                print!("    Correzione > ");
                io::stdout().flush().ok();
                let mut corr = String::new();
                if io::stdin().read_line(&mut corr).is_ok() {
                    let corr = corr.trim();
                    if !corr.is_empty() {
                        engine.teach(corr);
                        println!("  ✓ Correzione applicata");
                    }
                }
            }
        }
        
        // Auto-save ogni 3 cicli
        if question_count % 3 == 0 {
            save_state(&engine, &state_path);
        }
    }
    
    println!("\n{}", "═".repeat(70));
}
