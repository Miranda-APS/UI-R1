/// Script per esportare l'intero vocabolario per la revisione manuale.
///
/// Produce un file `master_lexicon_review.tsv` contenente:
/// 1. I 64 frattali base (I-Ching) con le loro firme esatte.
/// 2. Le parole sartoriali già curate (da phenomenology.tsv).
/// 3. Tutte le rimanenti parole del Knowledge Graph (con firma neutra 0.5).

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use prometeo::topology::knowledge_graph::KgSnapshot;
use prometeo::topology::fractal::HEXAGRAMS;

fn main() -> anyhow::Result<()> {
    println!("=== Generazione Master Lexicon per Revisione ===");

    let out_path = Path::new("data/kg/master_lexicon_review.tsv");
    let mut lines = vec![
        "# Master Lexicon per Revisione Sartoriale".to_string(),
        "# Dimensioni: [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo]".to_string(),
        "WORD\tTYPE\tSIG".to_string()
    ];

    // 1. I 64 Frattali
    println!("Aggiungo i 64 frattali I-Ching...");
    let mut fractal_names = HashSet::new();
    for (_i, (lower, upper, name)) in HEXAGRAMS.iter().enumerate() {
        let mut sig = [0.5; 8];
        sig[lower.dim().index()] = lower.value();
        sig[upper.dim().index()] = upper.value();
        
        // Mappatura indici interni -> Ordine Standard
        // Indici Dim: Agency=0, Permanenza=1, Intensita=2, Tempo=3, Confine=4, Complessita=5, Definizione=6, Valenza=7
        let remapped_sig = [
            sig[4], // Confine
            sig[7], // Valenza
            sig[2], // Intensita
            sig[6], // Definizione
            sig[5], // Complessita
            sig[1], // Permanenza
            sig[0], // Agency
            sig[3], // Tempo
        ];
        
        let sig_str: Vec<String> = remapped_sig.iter().map(|v| format!("{:.2}", v)).collect();
        let lower_name = name.to_lowercase();
        lines.push(format!("{}\tFRACTAL\t{}", lower_name, sig_str.join(",")));
        fractal_names.insert(lower_name);
    }

    // 2. Le parole sartoriali curate (da phenomenology.tsv)
    let pheno_path = Path::new("data/kg/phenomenology.tsv");
    let mut curated = HashSet::new();
    if pheno_path.exists() {
        println!("Aggiungo le parole sartoriali curate...");
        let content = fs::read_to_string(pheno_path)?;
        for line in content.lines() {
            if line.starts_with('#') || line.is_empty() { continue; }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let word = parts[0];
                let sig = parts[2];
                if !fractal_names.contains(word) {
                    lines.push(format!("{}\tCURATED\t{}", word, sig));
                    curated.insert(word.to_string());
                }
            }
        }
    }

    // 3. Tutte le parole del KG
    let kg_path = Path::new("prometeo_kg.json");
    if kg_path.exists() {
        println!("Estraggo il resto del vocabolario dal Knowledge Graph...");
        let json = fs::read_to_string(kg_path)?;
        let snap: KgSnapshot = serde_json::from_str(&json)?;
        let mut all_words = HashSet::new();
        for edge in snap.edges {
            all_words.insert(edge.subject);
            all_words.insert(edge.object);
        }
        
        let mut sorted_words: Vec<String> = all_words.into_iter().collect();
        sorted_words.sort();
        
        let mut pending_count = 0;
        for word in sorted_words {
            if !curated.contains(&word) && !fractal_names.contains(&word) {
                // Default signature 0.5
                lines.push(format!("{}\tPENDING\t0.50,0.50,0.50,0.50,0.50,0.50,0.50,0.50", word));
                pending_count += 1;
            }
        }
        println!("Aggiunte {} parole da calibrare.", pending_count);
    } else {
        println!("prometeo_kg.json non trovato!");
    }

    fs::write(out_path, lines.join("\n"))?;
    println!("File esportato con successo in: {}", out_path.display());
    Ok(())
}
