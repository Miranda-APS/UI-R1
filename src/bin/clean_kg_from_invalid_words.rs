/// Script per rimuovere definitivamente dal Knowledge Graph (prometeo_kg.json)
/// tutte le parole che sono state scartate durante la pulizia (e le loro relazioni).

use prometeo::topology::knowledge_graph::KgSnapshot;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("=== Pulizia del Knowledge Graph ===");

    // 1. Carica le parole valide dal file master appena pulito
    let tsv_path = Path::new("data/kg/master_lexicon_review.tsv");
    if !tsv_path.exists() {
        anyhow::bail!("File {:?} non trovato.", tsv_path);
    }
    
    let tsv_content = fs::read_to_string(tsv_path)?;
    let mut valid_words = HashSet::new();
    
    for line in tsv_content.lines() {
        if line.starts_with('#') || line.starts_with("WORD\t") || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let word = parts[0].trim();
            if !word.is_empty() {
                valid_words.insert(word.to_string());
            }
        }
    }
    println!("Parole valide caricate dalla revisione: {}", valid_words.len());

    // 2. Carica il Knowledge Graph
    let kg_path = Path::new("prometeo_kg.json");
    if !kg_path.exists() {
        anyhow::bail!("File {:?} non trovato.", kg_path);
    }
    
    let json = fs::read_to_string(kg_path)?;
    let mut snap: KgSnapshot = serde_json::from_str(&json)?;
    
    let initial_edges = snap.edges.len();
    println!("Relazioni (edges) totali iniziali nel KG: {}", initial_edges);

    // 3. Filtra le relazioni: mantieni solo quelle in cui sia soggetto che oggetto sono parole valide
    snap.edges.retain(|edge| {
        valid_words.contains(&edge.subject) && valid_words.contains(&edge.object)
    });

    let final_edges = snap.edges.len();
    let removed_edges = initial_edges - final_edges;
    println!("Relazioni rimosse (perché contenevano parole non valide): {}", removed_edges);
    println!("Relazioni rimanenti nel KG: {}", final_edges);

    // 4. Salva il KG aggiornato
    let out_json = serde_json::to_string_pretty(&snap)?;
    fs::write(kg_path, out_json)?;
    println!("Knowledge Graph (prometeo_kg.json) aggiornato con successo!");

    Ok(())
}
