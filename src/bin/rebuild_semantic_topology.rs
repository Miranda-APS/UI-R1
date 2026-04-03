/// Ricostruzione Semantica della Topologia delle Parole.
///
/// Questo binary fa la transizione fondamentale:
///   DA: co-occorrenze statistiche Wikipedia (ciao ↔ germania = rumore)
///   A:  relazioni logiche tipate dal KG    (ciao ↔ saluto = significato)
///
/// Operazione:
///   1. Carica lo stato Prometeo (.bin)
///   2. Carica il Knowledge Graph (prometeo_kg.json)
///   3. Rimuove tutti gli archi statistici (co-occorrenze)
///   4. Costruisce archi semantici tipati da KG (IS_A, HAS, DOES, CAUSES, ...)
///   5. Salva il nuovo stato
///
/// La rete risultante è:
///   - Pulita: nessun rumore Wikipedia
///   - Logica: ogni arco ha un significato preciso
///   - Ramificata: IS_A transitivo connette le categorie
///   - Esplorabile: puoi vedere PERCHÉ due parole sono connesse
///
/// Uso:
///   cargo run --release --bin rebuild-semantic-topology
///
/// Backup automatico del .bin originale prima di sovrascrivere.

use std::path::{Path, PathBuf};
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::knowledge_graph::{KnowledgeGraph, KgSnapshot};

fn main() -> anyhow::Result<()> {
    let root = find_project_root();
    let bin_path  = root.join("prometeo_topology_state.bin");
    let kg_path   = root.join("prometeo_kg.json");
    let backup    = root.join("prometeo_topology_state.bin.pre_semantic");

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  RICOSTRUZIONE TOPOLOGIA SEMANTICA                  ║");
    println!("║  Da co-occorrenze Wikipedia → relazioni logiche KG  ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    // 1. Carica il KG
    if !kg_path.exists() {
        eprintln!("ERRORE: prometeo_kg.json non trovato.");
        eprintln!("Esegui prima: cargo run --release --bin import-kg");
        std::process::exit(1);
    }
    let kg_json = std::fs::read_to_string(&kg_path)?;
    let kg_snap: KgSnapshot = serde_json::from_str(&kg_json)?;
    let kg = KnowledgeGraph::from_snapshot(kg_snap);
    println!("KG caricato: {} archi, {} nodi", kg.edge_count, kg.node_count);

    // 2. Carica lo stato Prometeo
    if !bin_path.exists() {
        eprintln!("ERRORE: prometeo_topology_state.bin non trovato.");
        std::process::exit(1);
    }
    println!("Caricando stato da {}...", bin_path.display());
    let state = PrometeoState::load_from_binary(&bin_path).map_err(|e| anyhow::anyhow!(e))?;
    println!("Stato caricato: {} parole nel lessico", state.lexicon.words.len());

    // 3. Ricostruisce engine
    let mut engine = PrometeoTopologyEngine::new();
    state.restore_lexicon(&mut engine);
    engine.lexicon.apply_curated_signatures();
    engine.recompute_all_word_affinities();

    println!();
    println!("─── STATO PRECEDENTE ─────────────────────────────────");
    let (total_before, semantic_before, stat_before) = engine.word_topology.edge_stats();
    println!("  Archi totali:      {}", total_before);
    println!("  Archi semantici:   {} ({:.1}%)", semantic_before,
        if total_before > 0 { semantic_before as f64 / total_before as f64 * 100.0 } else { 0.0 });
    println!("  Archi statistici:  {} (Wikipedia/testo)", stat_before);

    // 4. Rimuove archi statistici (co-occorrenze Wikipedia)
    println!();
    println!("─── RIMOZIONE CO-OCCORRENZE STATISTICHE ──────────────");
    let removed = engine.word_topology.clear_statistical_edges();
    println!("  Rimossi: {} archi statistici", removed);

    // 5. Costruisce archi semantici dal KG
    println!();
    println!("─── COSTRUZIONE ARCHI SEMANTICI ──────────────────────");
    engine.kg = kg;
    let (added, _) = engine.word_topology.build_from_knowledge_graph(&engine.kg);
    let (total_after, semantic_after, stat_after) = engine.word_topology.edge_stats();
    println!("  Archi aggiunti:    {}", added);
    println!("  Archi totali:      {}", total_after);
    println!("  Archi semantici:   {} ({:.1}%)", semantic_after,
        if total_after > 0 { semantic_after as f64 / total_after as f64 * 100.0 } else { 0.0 });
    println!("  Archi statistici:  {} (dovrebbe essere 0)", stat_after);

    // 6. Mostra esempi di connessioni semantiche
    println!();
    println!("─── ESEMPI CONNESSIONI ───────────────────────────────");
    for word in &["ciao", "cane", "germania", "sole", "amore"] {
        let neighbors = engine.word_topology.top_active_neighbors(word, 5);
        if !neighbors.is_empty() {
            let nn: Vec<String> = neighbors.iter().map(|(w, _)| w.clone()).collect();
            println!("  {} → [{}]", word, nn.join(", "));
        } else {
            println!("  {} → (nessun vicino nel KG)", word);
        }
    }

    // 7. Backup + salva
    println!();
    if bin_path.exists() {
        std::fs::copy(&bin_path, &backup)?;
        println!("Backup salvato: {}", backup.display());
    }

    let new_state = PrometeoState::capture(&engine);
    new_state.save_to_binary(&bin_path).map_err(|e| anyhow::anyhow!(e))?;
    println!("Stato salvato: {}", bin_path.display());

    println!();
    println!("✓ Topologia semantica ricostruita.");
    println!("  La rete ora parla logica, non statistica.");
    Ok(())
}

fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..5 {
        if dir.join("Cargo.toml").exists() { return dir; }
        if let Some(p) = dir.parent() { dir = p.to_path_buf(); } else { break; }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(p) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            if p.join("Cargo.toml").exists() { return p.to_path_buf(); }
        }
    }
    PathBuf::from(".")
}
