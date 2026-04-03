// src/bin/llm_inhabited.rs
//
// Test dell'LLM che abita lo spazio topologico di Prometeo
//
// Usage:
//   cargo run --release --features llm-substrate --bin llm-inhabited

use anyhow::Result;
use prometeo::topology::engine::PrometeoTopologyEngine;

#[cfg(feature = "llm-substrate")]
use prometeo::topology::llm_substrate::{SubstrateConfig, DeviceType};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== LLM INHABITED TEST ===\n");
    
    // 1. Carica Prometeo
    println!("[1/4] Caricamento Prometeo...");
    let mut engine = PrometeoTopologyEngine::new();
    
    let report = engine.report();
    println!("  Frattali: {}", report.fractal_count);
    println!("  Simplessi: {}", report.simplex_count);
    println!("  Componenti connesse: {}", report.connected_components);
    
    // 2. Ricevi input
    println!("\n[2/4] Ricezione input...");
    let input = "ciao, come stai?";
    println!("  Input: \"{}\"", input);
    
    let response = engine.receive(input);
    println!("  Frattali attivi: {}", response.active_fractals.len());
    
    // 3. Mostra stato interno
    println!("\n[3/4] Stato interno di Prometeo:");
    
    let field_sig = engine.field_sig();
    println!("  Campo 8D: {:?}", field_sig);
    
    let active_fractals = engine.active_fractals();
    println!("  Frattali attivi:");
    for (name, activation) in active_fractals.iter().take(5) {
        println!("    {} ({:.3})", name, activation);
    }
    
    let active_words = engine.what_i_see();
    println!("  Parole attive (top 10):");
    for (word, activation) in active_words.iter().take(10) {
        println!("    {} ({:.3})", word, activation);
    }
    
    if let Some(will) = engine.current_will() {
        println!("  Volontà: {:?}", will.intention);
    }
    
    // 4. Genera con LLM substrate
    #[cfg(feature = "llm-substrate")]
    {
        use std::path::Path;
        use prometeo::topology::llm_substrate::LLMSubstrate;
        
        println!("\n[4/4] Generazione con LLM Substrate...");
        
        let config = SubstrateConfig {
            model_name: "Qwen/Qwen3.5-9B".to_string(),
            device: DeviceType::CPU,
            temperature: 0.7,
            max_tokens: 50,
            constrain_to_lexicon: true,
            ..Default::default()
        };
        
        println!("  Configurazione:");
        println!("    Modello: {}", config.model_name);
        println!("    Device: {:?}", config.device);
        println!("    Temperature: {}", config.temperature);
        println!("    Max tokens: {}", config.max_tokens);
        println!("    Vincolo lessicale: {}", config.constrain_to_lexicon);
        
        // Crea substrate e carica matrice di proiezione
        let mut substrate = LLMSubstrate::new(config);
        
        let projection_path = Path::new("projection_qwen35_9b.npy");
        if projection_path.exists() {
            println!("\n  Caricamento matrice di proiezione...");
            match substrate.load_projection(projection_path) {
                Ok(_) => {
                    println!("  ✓ Matrice caricata con successo");
                    
                    // Test inhabit
                    println!("\n  Test inhabit()...");
                    match substrate.inhabit(&engine) {
                        Ok(response) => {
                            println!("  ✓ Risposta generata:");
                            println!("    Campo 8D: {:?}", response.field_signature);
                            println!("    Testo: {}", response.text);
                        }
                        Err(e) => {
                            println!("  ✗ Errore inhabit: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("  ✗ Errore caricamento matrice: {}", e);
                }
            }
        } else {
            println!("\n  ⚠ File projection_qwen35_9b.npy non trovato");
            println!("  Esegui prima la calibrazione:");
            println!("    python tools/calibrate_llm_projection.py \\");
            println!("      --model Qwen/Qwen3.5-9B \\");
            println!("      --state comunita_prometeo.bin \\");
            println!("      --output projection_qwen35_9b.npy \\");
            println!("      --max-words 500");
        }
    }
    
    #[cfg(not(feature = "llm-substrate"))]
    {
        println!("\n[4/4] Feature 'llm-substrate' non abilitata");
        println!("  Compila con: cargo run --features llm-substrate --bin llm-inhabited");
    }
    
    println!("\n✓ Test completato");
    
    Ok(())
}
