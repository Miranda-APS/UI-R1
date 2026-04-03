/// Analizzatore del Knowledge Graph: Conta le relazioni mancanti per tipo.
/// Questo script serve per farsi un'idea di quante parole non hanno almeno 
/// una relazione per ogni RelationType, prima di generarle massivamente.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use prometeo::topology::relation::{RelationType, TypedEdge};

fn main() -> anyhow::Result<()> {
    println!("=== Analisi Completezza Relazioni KG ===");
    let kg_dir = Path::new("data/kg");
    
    // Mappa: Parola -> Set di Relazioni uscenti presenti
    let mut word_relations: HashMap<String, HashSet<RelationType>> = HashMap::new();

    for entry in fs::read_dir(kg_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|e| e.to_str()) == Some("tsv") {
            let content = fs::read_to_string(&path)?;
            for line in content.lines() {
                if let Some(edge) = TypedEdge::from_tsv_line(line.trim()) {
                    word_relations.entry(edge.subject).or_default().insert(edge.relation);
                }
            }
        }
    }

    let total_words = word_relations.len();
    println!("Totale parole uniche trovate come Soggetto: {}", total_words);

    let all_rels = RelationType::ALL;
    let mut missing_counts: HashMap<RelationType, usize> = HashMap::new();

    for rels in word_relations.values() {
        for &rel in &all_rels {
            if !rels.contains(&rel) {
                *missing_counts.entry(rel).or_insert(0) += 1;
            }
        }
    }

    println!("\nParole MANCANTI per tipo di relazione (su {} totali):", total_words);
    for rel in &all_rels {
        let missing = missing_counts.get(rel).unwrap_or(&0);
        let perc = (*missing as f64 / total_words as f64) * 100.0;
        println!("  {:<15} : {} mancano ({:.1}%)", rel.as_str(), missing, perc);
    }

    Ok(())
}
