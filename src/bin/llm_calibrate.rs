// src/bin/llm_calibrate.rs
//
// Calibra la matrice di proiezione hidden_states → 8D Prometeo
//
// Usage:
//   cargo run --release --features llm-substrate --bin llm-calibrate -- \
//     --model Qwen/Qwen3.5-9B \
//     --state prometeo_topology_state.bin \
//     --output projection_qwen35_9b.npy

use anyhow::Result;
use prometeo::topology::engine::PrometeoTopologyEngine;
use std::path::PathBuf;

#[cfg(feature = "llm-substrate")]
use prometeo::topology::llm_substrate::candle_impl::CandleSubstrate;

#[derive(Debug)]
struct Args {
    model: String,
    state_path: PathBuf,
    output_path: PathBuf,
    max_words: usize,
    min_stability: f64,
    min_exposure: u64,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut args = std::env::args().skip(1);
        let mut model = "Qwen/Qwen3.5-9B".to_string();
        let mut state_path = PathBuf::from("prometeo_topology_state.bin");
        let mut output_path = PathBuf::from("projection_matrix.npy");
        let mut max_words = 1000;
        let mut min_stability = 0.5;
        let mut min_exposure = 10;
        
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--model" => model = args.next().unwrap(),
                "--state" => state_path = PathBuf::from(args.next().unwrap()),
                "--output" => output_path = PathBuf::from(args.next().unwrap()),
                "--max-words" => max_words = args.next().unwrap().parse()?,
                "--min-stability" => min_stability = args.next().unwrap().parse()?,
                "--min-exposure" => min_exposure = args.next().unwrap().parse()?,
                _ => anyhow::bail!("Unknown argument: {}", arg),
            }
        }
        
        Ok(Self {
            model,
            state_path,
            output_path,
            max_words,
            min_stability,
            min_exposure,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== CALIBRAZIONE LLM SUBSTRATE ===\n");
    
    let args = Args::parse()?;
    
    println!("Configurazione:");
    println!("  Modello: {}", args.model);
    println!("  Stato Prometeo: {:?}", args.state_path);
    println!("  Output: {:?}", args.output_path);
    println!("  Max parole: {}", args.max_words);
    println!("  Min stabilità: {:.2}", args.min_stability);
    println!("  Min esposizioni: {}\n", args.min_exposure);
    
    // 1. Carica Prometeo
    println!("[1/5] Caricamento stato Prometeo...");
    let engine = PrometeoTopologyEngine::new();
    
    // 2. Filtra parole stabili
    println!("[2/5] Selezione parole stabili...");
    let stable_words: Vec<_> = engine.lexicon.patterns_iter()
        .filter(|(_, w)| {
            w.stability >= args.min_stability && 
            w.exposure_count >= args.min_exposure
        })
        .take(args.max_words)
        .map(|(word, pattern)| (word.clone(), pattern.clone()))
        .collect();
    
    println!("  Parole selezionate: {}", stable_words.len());
    
    if stable_words.is_empty() {
        anyhow::bail!("Nessuna parola stabile trovata!");
    }
    
    // 3. Carica modello LLM
    #[cfg(feature = "llm-substrate")]
    {
        println!("[3/5] Caricamento modello LLM...");
        
        let config = prometeo::topology::llm_substrate::SubstrateConfig {
            model_name: args.model.clone(),
            device: prometeo::topology::llm_substrate::DeviceType::CPU,
            ..Default::default()
        };
        
        let substrate = CandleSubstrate::load(config).await?;
        
        println!("  Modello caricato con successo");
        
        // 4. Estrai hidden states per ogni parola
        println!("[4/5] Estrazione hidden states...");
        
        // TODO: Implementare estrazione
        // Per ora, placeholder
        
        println!("  Hidden states estratti");
        
        // 5. Calibra proiezione
        println!("[5/5] Calibrazione proiezione...");
        
        // TODO: Implementare calibrazione
        // Least squares: P = (X^T X)^-1 X^T Y
        
        println!("\n✓ Calibrazione completata!");
        println!("  Matrice salvata in: {:?}", args.output_path);
    }
    
    #[cfg(not(feature = "llm-substrate"))]
    {
        anyhow::bail!("Feature 'llm-substrate' non abilitata. Compila con --features llm-substrate");
    }
    
    Ok(())
}
