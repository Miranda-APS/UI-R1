/// Phase 63 — Riderivazione firme 8D dal Knowledge Graph.
///
/// Il problema: le firme 8D delle parole erano calibrate da co-occorrenze
/// statistiche nel corpus. Parole co-occorrenti con token grammaticali
/// (articoli, verbi ausiliari) convergono verso la regione LINGUAGGIO/INTRECCIO,
/// indipendentemente dal loro significato semantico.
///
/// La soluzione: ogni dimensione 8D mappa a una proprietà strutturale della
/// posizione della parola nel KG — le relazioni SONO il significato.
///
///   Dim 0 Agency     (☰ Cielo)   — ratio CAUSES outgoing / totale
///   Dim 1 Permanenza (☷ Terra)   — conteggio IS_A children
///   Dim 2 Intensità  (☳ Tuono)   — forza causale + carica emotiva
///   Dim 3 Tempo      (☵ Acqua)   — partecipazione a catene causali
///   Dim 4 Confine    (☶ Montagna)— specificità IS_A + OPPOSITE_OF
///   Dim 5 Complessità(☴ Vento)   — log grado / log max_grado
///   Dim 6 Definizione(☲ Fuoco)   — genitori IS_A + OPPOSITE_OF
///   Dim 7 Valenza    (☱ Lago)    — BFS da radici emotive
///
/// Uso:
///   cargo run --release --bin rederive-signatures

use std::path::{Path, PathBuf};
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::knowledge_graph::{KnowledgeGraph, KgSnapshot};
use prometeo::topology::primitive::PrimitiveCore;

const STATE_PATH: &str = "prometeo_topology_state.bin";
const BACKUP_PATH: &str = "prometeo_topology_state.bin.pre_p63";
const KG_PATH:    &str = "prometeo_kg.json";

fn main() -> anyhow::Result<()> {
    let root = find_project_root();
    let bin_path = root.join(STATE_PATH);
    let kg_path  = root.join(KG_PATH);
    let backup   = root.join(BACKUP_PATH);

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  PHASE 63 — RIDERIVAZIONE FIRME 8D DA KG            ║");
    println!("║  Geometria = Relazioni (non statistica)              ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    // 1. Carica KG
    if !kg_path.exists() {
        eprintln!("ERRORE: prometeo_kg.json non trovato.");
        eprintln!("Esegui prima: cargo run --release --bin import-kg");
        std::process::exit(1);
    }
    println!("Carico KG da {}...", kg_path.display());
    let kg_json = std::fs::read_to_string(&kg_path)?;
    let kg_snap: KgSnapshot = serde_json::from_str(&kg_json)?;
    let kg = KnowledgeGraph::from_snapshot(kg_snap);
    println!("  KG caricato: {} archi, {} nodi", kg.edge_count, kg.node_count);

    // 2. Carica stato Prometeo
    if !bin_path.exists() {
        eprintln!("ERRORE: {} non trovato.", bin_path.display());
        std::process::exit(1);
    }
    println!("Carico stato da {}...", bin_path.display());
    let state = PrometeoState::load_from_binary(Path::new(&bin_path))
        .map_err(|e| anyhow::anyhow!(e))?;
    let n_words = state.lexicon.words.len();
    println!("  Lessico: {} parole", n_words);

    // 3. Pre-calcoli KG (costosi, fatti una volta)
    println!();
    println!("─── PRE-CALCOLI KG ───────────────────────────────────");
    let t0 = std::time::Instant::now();
    let max_degree = kg.max_total_degree();
    println!("  Grado massimo KG: {} ({}ms)", max_degree, t0.elapsed().as_millis());

    let t1 = std::time::Instant::now();
    let valence_scores = kg.compute_valence_scores();
    println!("  Valenze calcolate: {} parole con carica emotiva ({}ms)", valence_scores.len(), t1.elapsed().as_millis());

    let t1b = std::time::Instant::now();
    let temporal_scores = kg.compute_temporal_scores();
    println!("  Posizioni temporali calcolate: {} parole ({}ms)", temporal_scores.len(), t1b.elapsed().as_millis());

    // 4. Ricostruisce engine e ripristina lessico
    println!();
    println!("─── RIPRISTINO ENGINE ────────────────────────────────");
    let mut engine = PrometeoTopologyEngine::new();
    state.restore_lexicon(&mut engine);
    engine.kg = kg;
    println!("  Engine pronto: {} parole nel lessico", engine.lexicon.word_count());

    // 5. Rideriva le firme 8D
    println!();
    println!("─── RIDERIVAZIONE FIRME 8D ───────────────────────────");
    let words: Vec<String> = engine.lexicon.patterns_iter()
        .map(|(w, _)| w.to_string())
        .collect();

    let mut updated = 0usize;
    let mut skipped_no_kg = 0usize;

    for word in &words {
        // Deriva solo parole presenti nel KG — le altre mantengono la firma precedente
        if let Some(sig) = engine.kg.derive_8d_from_kg(word, max_degree, &valence_scores, &temporal_scores) {
            if let Some(pat) = engine.lexicon.get_mut(word) {
                pat.signature = PrimitiveCore::new(sig);
                updated += 1;
            }
        } else {
            skipped_no_kg += 1;
        }
    }

    println!("  Firme aggiornate:  {} / {}", updated, words.len());
    println!("  Parole senza KG:   {} (mantengono firma precedente)", skipped_no_kg);

    // 6. Ricalcola le affinità frattali da tutte le firme aggiornate
    println!();
    println!("─── RICALCOLO AFFINITÀ FRATTALI ──────────────────────");
    let t2 = std::time::Instant::now();
    engine.recompute_all_word_affinities();
    println!("  Ricalcolo completato ({}ms)", t2.elapsed().as_millis());

    // 7. Mostra distribuzione firme — campione diagnostico
    println!();
    println!("─── VERIFICA CAMPIONE ────────────────────────────────");
    println!("  {:15} {:5} {:5} {:5} {:5} {:5} {:5} {:5} {:5}  (Agency Perm Int Tempo Conf Compl Def Val)",
        "parola", "Ag", "Pe", "In", "Te", "Co", "Cp", "De", "Va");
    for word in &["tristezza", "gioia", "paura", "cane", "essere", "correre", "pietra", "amore", "domanda", "io"] {
        if let Some(pat) = engine.lexicon.get(word) {
            let s = pat.signature.values();
            println!("  {:15} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                word, s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]);
        } else {
            println!("  {:15} (non nel lessico)", word);
        }
    }

    // 8. Distribuzione valenza — verifica che emozioni si disperdano correttamente
    println!();
    println!("─── TOP FRATTALI PER ALCUNE PAROLE ──────────────────");
    for word in &["tristezza", "gioia", "paura", "cane"] {
        if let Some(pat) = engine.lexicon.get(word) {
            let top: Vec<String> = pat.fractal_affinities.iter()
                .take(3)
                .map(|(fid, af)| format!("{}({:.2})", fid, af))
                .collect();
            println!("  {} → [{}]", word, top.join(", "));
        }
    }

    // 9. Backup + salva
    println!();
    if bin_path.exists() {
        std::fs::copy(&bin_path, &backup)?;
        println!("Backup salvato: {}", backup.display());
    }

    let new_state = PrometeoState::capture(&engine);
    new_state.save_to_binary(Path::new(&bin_path))
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("Stato salvato: {}", bin_path.display());

    println!();
    println!("✓ Phase 63 completata.");
    println!("  {} firme riderivate da relazioni KG (non statistica).", updated);
    println!("  La geometria ora riflette il significato relazionale.");

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
