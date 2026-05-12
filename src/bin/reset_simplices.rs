/// Azzera i simplessi cristallizzati e la memoria MTM/LTM,
/// preservando lessico, identità, narrativa, episodi e conoscenza.
///
/// Uso: cargo run --release --bin reset-simplices
///
/// Questo risolve la "saturazione di fondo": simplessi accumulati
/// da sessioni filosofiche che rendono il campo sordo agli input,
/// con INTRECCIO/LINGUAGGIO sempre a 1.0 indipendentemente dall'input.

use std::path::Path;
use prometeo::topology::persistence::PrometeoState;

const STATE_PATH: &str = "prometeo_topology_state.bin";
const BACKUP_PATH: &str = "prometeo_topology_state.bin.pre_reset";

fn main() -> anyhow::Result<()> {
    println!("=== Reset Simplessi ===");
    println!("Carico stato: {}", STATE_PATH);

    let mut state = PrometeoState::load_from_binary(Path::new(STATE_PATH))
        .map_err(|e| anyhow::anyhow!(e))?;

    // Statistiche prima del reset
    let n_simplices = state.complex.simplices.len();
    let n_mtm = state.memory.mtm.len();
    let n_ltm = state.memory.ltm.len();
    let n_words = state.lexicon.words.len();
    let perturbations = state.total_perturbations;

    println!("\nStato attuale:");
    println!("  Parole lessico: {}", n_words);
    println!("  Perturbazioni totali: {}", perturbations);
    println!("  Simplessi: {}", n_simplices);
    println!("  MTM imprints: {}", n_mtm);
    println!("  LTM imprints: {}", n_ltm);

    // Backup
    println!("\nBackup → {}", BACKUP_PATH);
    std::fs::copy(STATE_PATH, BACKUP_PATH)?;

    // Reset: azzera simplessi e memoria contestuale
    // I simplessi codificano pattern co-occorrenza delle sessioni precedenti.
    // MTM/LTM codificano imprints di memoria a medio/lungo termine.
    // Entrambi contribuiscono alla saturazione di fondo.
    state.complex.simplices.clear();
    state.memory.mtm.clear();
    state.memory.ltm.clear();

    // Preserviamo: lessico, identità, narrativa, episodi, conoscenza, curriculum
    println!("\nPreservati:");
    println!("  Lessico: {} parole", n_words);
    if let Some(id) = &state.identity {
        println!("  Identità: continuità {:.2}, {} aggiornamenti", id.continuity, id.update_count);
    }
    if let Some(narr) = &state.narrative {
        println!("  Narrativa: {} turni cristallizzati", narr.crystallized.len());
    }
    if let Some(eps) = &state.episodes {
        println!("  Episodi semantici: {}", eps.episodes.len());
    }

    // Salva
    println!("\nSalvo stato pulito → {}", STATE_PATH);
    state.save_to_binary(Path::new(STATE_PATH))
        .map_err(|e| anyhow::anyhow!(e))?;

    println!("\n✓ Reset completato.");
    println!("  Simplessi azzerati: {} → 0", n_simplices);
    println!("  MTM azzerati: {} → 0", n_mtm);
    println!("  LTM azzerati: {} → 0", n_ltm);
    println!("\nIl campo ora risponde agli input senza memoria contestuale di sfondo.");
    println!("Nota: i simplessi si ricristallizzeranno naturalmente durante la conversazione.");

    Ok(())
}
