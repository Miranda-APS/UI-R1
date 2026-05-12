/// Migrazione una tantum: permuta le firme 8D dall'ordine Dim-enum legacy
/// all'ordine I Ching canonico (Cielo→Lago).
///
/// Permutazione (OLD pos → NEW pos):
///   Confine    0 → 4   (☶ Montagna)
///   Valenza    1 → 7   (☱ Lago)
///   Intensita  2 → 2   (☳ Tuono)
///   Definizione 3 → 6   (☲ Fuoco)
///   Complessita 4 → 5   (☴ Vento)
///   Permanenza 5 → 1   (☷ Terra)
///   Agency     6 → 0   (☰ Cielo)
///   Tempo      7 → 3   (☵ Acqua)
///
/// Inverso (NEW pos ← OLD pos): new[0]=old[6], new[1]=old[5], new[2]=old[2],
///                              new[3]=old[7], new[4]=old[0], new[5]=old[4],
///                              new[6]=old[3], new[7]=old[1]
///
/// DA ESEGUIRE UNA SOLA VOLTA prima del refactor dell'enum Dim.
/// Idempotenza non è garantita — non rieseguirlo.

use std::path::{Path, PathBuf};
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::primitive::PrimitiveCore;

const STATE_PATH:  &str = "prometeo_topology_state.bin";
const BACKUP_PATH: &str = "prometeo_topology_state.bin.pre_iching_ordering";

fn permute_old_to_iching(old: &[f64; 8]) -> [f64; 8] {
    // new[i] = old[source[i]]
    let source = [6usize, 5, 2, 7, 0, 4, 3, 1];
    let mut new = [0.0f64; 8];
    for i in 0..8 {
        new[i] = old[source[i]];
    }
    new
}

fn main() -> anyhow::Result<()> {
    let root = find_project_root();
    let bin_path = root.join(STATE_PATH);
    let backup   = root.join(BACKUP_PATH);

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  MIGRAZIONE ORDINAMENTO FIRME 8D — I CHING CANONICO ║");
    println!("║  Permuta OLD Dim-enum → NEW I Ching (Cielo→Lago)    ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();

    if !bin_path.exists() {
        eprintln!("ERRORE: {} non trovato.", bin_path.display());
        std::process::exit(1);
    }

    // 1. Carica stato
    println!("Carico stato da {}...", bin_path.display());
    let state = PrometeoState::load_from_binary(Path::new(&bin_path))
        .map_err(|e| anyhow::anyhow!(e))?;
    let n_words = state.lexicon.words.len();
    println!("  Lessico: {} parole", n_words);

    // 2. Ricostruisci engine con lessico originale
    let mut engine = PrometeoTopologyEngine::new();
    state.restore_lexicon(&mut engine);
    println!("  Engine pronto: {} parole nel lessico", engine.lexicon.word_count());

    // 3. Mostra un campione PRIMA della permutazione
    println!();
    println!("─── CAMPIONE PRIMA DELLA PERMUTAZIONE ───────────");
    println!("  (ordine legacy: [Confine, Valenza, Intens, Defin, Compl, Perm, Agency, Tempo])");
    for w in &["gioia", "tristezza", "paura", "io", "essere", "cane"] {
        if let Some(pat) = engine.lexicon.get(w) {
            let s = pat.signature.values();
            println!("  {:<12} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                w, s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]);
        }
    }

    // 4. Permuta tutte le firme
    let words: Vec<String> = engine.lexicon.patterns_iter()
        .map(|(w, _)| w.to_string())
        .collect();

    let mut permuted = 0usize;
    for word in &words {
        if let Some(pat) = engine.lexicon.get_mut(word) {
            let old_vals = *pat.signature.values();
            let new_vals = permute_old_to_iching(&old_vals);
            pat.signature = PrimitiveCore::new(new_vals);
            permuted += 1;
        }
    }
    println!();
    println!("Firme permutate: {}", permuted);

    // 5. Ricalcola affinità frattali dalla firma riorganizzata
    println!("Ricalcolo affinità frattali...");
    engine.recompute_all_word_affinities();

    // 6. Campione DOPO
    println!();
    println!("─── CAMPIONE DOPO LA PERMUTAZIONE ───────────────");
    println!("  (ordine I Ching: [Agency, Perm, Intens, Tempo, Confine, Compl, Defin, Valenza])");
    for w in &["gioia", "tristezza", "paura", "io", "essere", "cane"] {
        if let Some(pat) = engine.lexicon.get(w) {
            let s = pat.signature.values();
            println!("  {:<12} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                w, s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]);
        }
    }

    // 7. Backup + salva
    println!();
    std::fs::copy(&bin_path, &backup)?;
    println!("Backup salvato: {}", backup.display());

    let new_state = PrometeoState::capture(&engine);
    new_state.save_to_binary(Path::new(&bin_path))
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("Stato migrato salvato: {}", bin_path.display());

    println!();
    println!("✓ Migrazione completata.");
    println!("  {} firme permutate dall'ordine legacy all'ordine I Ching canonico.", permuted);
    println!("  ATTENZIONE: non rieseguire questo binario (non è idempotente).");

    Ok(())
}

fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..5 {
        if dir.join("Cargo.toml").exists() { return dir; }
        if let Some(p) = dir.parent() { dir = p.to_path_buf(); } else { break; }
    }
    PathBuf::from(".")
}
