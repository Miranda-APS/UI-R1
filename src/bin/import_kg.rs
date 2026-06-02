/// Import Knowledge Graph — carica le triple TSV nel KG di Prometeo.
///
/// Legge tutti i file .tsv da data/kg/ e li salva come snapshot JSON
/// nel file prometeo_kg.json (caricato all'avvio del motore).
///
/// Uso:
///   cargo run --release --bin import-kg

use std::path::{Path, PathBuf};
use prometeo::topology::knowledge_graph::{KnowledgeGraph, KgSnapshot};

fn main() -> anyhow::Result<()> {
    let root = find_project_root();
    let kg_dir = root.join("data").join("kg");
    let output_path = root.join("prometeo_kg.json");

    println!("=== Import Knowledge Graph ===");
    println!("Sorgente: {}", kg_dir.display());

    if !kg_dir.exists() {
        eprintln!("ERRORE: directory {} non trovata", kg_dir.display());
        eprintln!("Crea data/kg/ e aggiungi file .tsv con triple");
        std::process::exit(1);
    }

    let mut kg = KnowledgeGraph::new();
    let count = kg.load_from_dir(&kg_dir)?;

    println!("Triple caricate: {}", count);
    println!("Nodi: {}", kg.node_count);
    println!("Archi: {}", kg.edge_count);

    // Salva snapshot JSON
    let snap = kg.to_snapshot();
    let json = serde_json::to_string_pretty(&snap)?;
    std::fs::write(&output_path, &json)?;

    println!("KG salvato: {}", output_path.display());
    println!("");
    println!("Esempi di inferenza:");

    use prometeo::topology::inference::InferenceEngine;
    let engine = InferenceEngine::new(&kg);

    // Test "cane"
    let types = engine.type_chain("cane");
    if !types.is_empty() {
        println!("  cane IS-A: {}", types.join(", "));
    }
    let actions = engine.what_does("cane");
    if !actions.is_empty() {
        println!("  cane DOES: {}", actions.join(", "));
    }

    // Test "ciao"
    let ciao_boosts = engine.field_boosts("ciao");
    if !ciao_boosts.is_empty() {
        let words: Vec<&str> = ciao_boosts.iter().take(5).map(|(w, _)| w.as_str()).collect();
        println!("  ciao boost: {}", words.join(", "));
    }

    // Test "germania"
    let ger_types = engine.type_chain("germania");
    if !ger_types.is_empty() {
        println!("  germania IS-A: {}", ger_types.join(", "));
    }
    let ger_has = engine.what_has("germania");
    if !ger_has.is_empty() {
        println!("  germania HAS: {}", ger_has.join(", "));
    }

    println!("");
    println!("✓ Import completato");
    Ok(())
}

fn find_project_root() -> PathBuf {
    // Override esplicito via env var. Usata da progetti wrapper (es. IAm-a-gotchi)
    // che vivono fuori dal repo e vogliono leggere data/kg/ + scrivere
    // prometeo_kg.json dentro la loro cartella, senza che il fallback per
    // current_exe() li trascini dentro prometeo_standalone/.
    if let Ok(p) = std::env::var("PROMETEO_PROJECT_ROOT") {
        let path = PathBuf::from(p);
        if path.exists() {
            return path;
        }
    }
    // Cerca il Cargo.toml risalendo dalla directory corrente
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..5 {
        if dir.join("Cargo.toml").exists() {
            return dir;
        }
        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            break;
        }
    }
    // Fallback: usa il path dell'eseguibile
    if let Ok(exe) = std::env::current_exe() {
        // target/release/import-kg → progetto è 3 livelli su
        if let Some(p) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            if p.join("Cargo.toml").exists() {
                return p.to_path_buf();
            }
        }
    }
    PathBuf::from(".")
}
