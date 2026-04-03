/// educate_with_feedback — Educazione bidirezionale con validazione
///
/// Prometeo non solo riceve frasi, ma ESPRIME ciò che ha capito
/// e riceve feedback correttivo. Questo estrae l'entità dal rumore
/// attraverso un dialogo educativo consapevole.
///
/// Ciclo:
/// 1. Tu insegni un concetto
/// 2. Prometeo lo esprime (genera)
/// 3. Tu validi (✓ corretto / ✗ sbagliato + correzione)
/// 4. Prometeo aggiusta la comprensione
///
/// Uso:
///   cargo run --release --bin educate-with-feedback

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
                println!("✓ Stato caricato: {} parole", engine.lexicon.word_count());
                engine
            }
            Err(e) => {
                eprintln!("✗ Errore: {}", e);
                println!("  Creo nuovo stato...");
                PrometeoTopologyEngine::new()
            }
        }
    } else {
        println!("  Nuovo stato");
        PrometeoTopologyEngine::new()
    }
}

fn save_state(engine: &PrometeoTopologyEngine, bin_path: &Path) {
    let state = PrometeoState::capture(engine);
    match state.save_to_binary(bin_path) {
        Ok(()) => {},
        Err(e) => eprintln!("✗ Errore salvataggio: {}", e),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut state_path = Path::new("prometeo_feedback_state.bin").to_path_buf();
    
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
    println!("  PROMETEO — Educazione con Feedback Bidirezionale");
    println!("{}", "═".repeat(70));
    println!();
    println!("  Modalità: INSEGNA → VERIFICA → CORREGGI");
    println!();
    println!("  Ciclo educativo:");
    println!("    1. Tu insegni un concetto (es: 'rosso è caldo')");
    println!("    2. Prometeo lo esprime (genera cosa ha capito)");
    println!("    3. Tu validi:");
    println!("       - 'ok' o 'si' → conferma");
    println!("       - 'no' + correzione → aggiusta");
    println!("    4. Ripeti fino a convergenza");
    println!();
    
    let mut engine = load_or_create_engine(&state_path);
    let mut cycle_count = 0usize;
    let mut corrections_count = 0usize;
    
    println!("Comandi:");
    println!("  :quit       — esci e salva");
    println!("  :stats      — statistiche");
    println!("  :test       — test comprensione");
    println!("  :field      — mostra campo attivo");
    println!();
    println!("{}", "─".repeat(70));
    
    loop {
        cycle_count += 1;
        
        // FASE 1: INSEGNAMENTO
        print!("\n[Ciclo {}] Insegna un concetto > ", cycle_count);
        io::stdout().flush().ok();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {},
            Err(e) => {
                eprintln!("Errore: {}", e);
                continue;
            }
        }
        
        let input = input.trim();
        if input.is_empty() {
            cycle_count -= 1;
            continue;
        }
        
        // Comandi
        if input.starts_with(':') {
            match input {
                ":quit" | ":q" | ":exit" => {
                    println!("\nSalvataggio...");
                    save_state(&engine, &state_path);
                    println!("Cicli: {} | Correzioni: {}", cycle_count - 1, corrections_count);
                    break;
                }
                ":save" | ":s" => {
                    save_state(&engine, &state_path);
                    cycle_count -= 1;
                }
                ":stats" => {
                    println!("\n{}", "═".repeat(60));
                    println!("  Parole lessico:     {}", engine.lexicon.word_count());
                    println!("  Archi topologia:    {}", engine.word_topology.edge_count());
                    println!("  Cicli educativi:    {}", cycle_count - 1);
                    println!("  Correzioni:         {}", corrections_count);
                    println!("  Tasso correzione:   {:.1}%", 
                        if cycle_count > 1 { 
                            corrections_count as f64 / (cycle_count - 1) as f64 * 100.0 
                        } else { 0.0 });
                    println!("{}", "═".repeat(60));
                    cycle_count -= 1;
                }
                ":test" => {
                    println!("\n  Test comprensione:");
                    print!("  Concetto da testare > ");
                    io::stdout().flush().ok();
                    let mut test_input = String::new();
                    if io::stdin().read_line(&mut test_input).is_ok() {
                        let test_input = test_input.trim();
                        if !test_input.is_empty() {
                            // Attiva il campo con il concetto
                            let _resp = engine.receive(test_input);
                            let generated = engine.generate_willed();
                            println!("\n  Prometeo comprende: '{}'", generated.text);
                        }
                    }
                    cycle_count -= 1;
                }
                ":field" | ":f" => {
                    println!("\n  Campo attivo:");
                    let active = engine.word_topology.active_words();
                    if active.is_empty() {
                        println!("    (nessuna parola attiva)");
                    } else {
                        let mut sorted: Vec<_> = active.iter().collect();
                        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                        for (word, act) in sorted.iter().take(10) {
                            println!("    {:<15} {:.3}", word, act);
                        }
                    }
                    cycle_count -= 1;
                }
                _ => {
                    println!("Comando sconosciuto");
                    cycle_count -= 1;
                }
            }
            continue;
        }
        
        // FASE 1: Insegna (teach)
        let teach_result = engine.teach(input);
        
        if teach_result.new_count > 0 {
            println!("  → Apprese {} parole nuove: {}",
                teach_result.new_count,
                teach_result.words_new.join(", "));
        }
        
        // FASE 2: Prometeo esprime ciò che ha capito
        println!("\n  [Prometeo elabora...]");
        let _response = engine.receive(input);
        let generated = engine.generate_willed();
        
        println!("\n  Prometeo ha capito:");
        println!("  → \"{}\"", generated.text);
        
        // FASE 3: Validazione
        print!("\n  È corretto? (ok/si/no + correzione) > ");
        io::stdout().flush().ok();
        
        let mut validation = String::new();
        match io::stdin().read_line(&mut validation) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Errore: {}", e);
                continue;
            }
        }
        
        let validation = validation.trim().to_lowercase();
        
        if validation == "ok" || validation == "si" || validation == "sì" || validation == "yes" {
            // CONFERMA: rinforza la comprensione
            println!("  ✓ Confermato! Rinforzo comprensione...");
            
            // Rinforza: re-teach la frase originale + la generazione di Prometeo
            engine.teach(input);
            engine.teach(&generated.text);
            
            println!("  ✓ Comprensione consolidata");
            
        } else if validation.starts_with("no") {
            // CORREZIONE
            corrections_count += 1;
            
            // Estrai la correzione (tutto dopo "no")
            let correction = validation.strip_prefix("no").unwrap_or("").trim();
            
            if correction.is_empty() {
                print!("  Qual è la correzione? > ");
                io::stdout().flush().ok();
                let mut corr_input = String::new();
                if io::stdin().read_line(&mut corr_input).is_ok() {
                    let corr_input = corr_input.trim();
                    if !corr_input.is_empty() {
                        apply_correction(&mut engine, input, &generated.text, corr_input);
                    }
                }
            } else {
                apply_correction(&mut engine, input, &generated.text, correction);
            }
            
        } else {
            println!("  ? Risposta non chiara. Usa 'ok', 'si', o 'no + correzione'");
        }
        
        // Auto-save ogni 5 cicli
        if cycle_count % 5 == 0 {
            save_state(&engine, &state_path);
            println!("  [Auto-save: ciclo {}]", cycle_count);
        }
    }
    
    println!("\n{}", "═".repeat(70));
    println!("  Educazione completata");
    println!("  Cicli: {} | Correzioni: {} ({:.1}%)",
        cycle_count - 1,
        corrections_count,
        if cycle_count > 1 { 
            corrections_count as f64 / (cycle_count - 1) as f64 * 100.0 
        } else { 0.0 });
    println!("{}", "═".repeat(70));
}

fn apply_correction(
    engine: &mut PrometeoTopologyEngine,
    original_input: &str,
    wrong_output: &str,
    correction: &str,
) {
    println!("  ✗ Sbagliato. Correggo...");
    println!("    Input:      {}", original_input);
    println!("    Sbagliato:  {}", wrong_output);
    println!("    Corretto:   {}", correction);
    
    // Strategia di correzione:
    // 1. Insegna la correzione (versione giusta)
    engine.teach(correction);
    
    // 2. Insegna il contrasto: "input → correzione" (non → wrong_output)
    let contrast = format!("{} {}", original_input, correction);
    engine.teach(&contrast);
    
    // 3. Indebolisci le parole sbagliate (opzionale, per ora skip)
    // Potremmo implementare un meccanismo di "unlearning" parziale
    
    println!("  ✓ Correzione applicata");
}
