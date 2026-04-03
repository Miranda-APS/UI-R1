/// Script di purificazione del Knowledge Graph (clean_kg).
///
/// Legge tutti i file TSV in `data/kg/`, applica regole severe di filtraggio,
/// e sovrascrive i file puliti, stampando un log delle righe rimosse.
///
/// Regole di pulizia:
/// 1. Nessun carattere speciale o numero (solo a-z, spazi, apostrofi permessi).
/// 2. Lunghezza delle parole ragionevole (min 2, max 25), eccezioni: "o", "a", "e".
/// 3. Confidenza minima 0.5.
/// 4. Relazioni valide (già gestite da TypedEdge::from_tsv_line).
/// 5. Nessuna parola palesemente errata o inglese ("abb", "about", ecc.).

use std::fs;
use std::path::{Path, PathBuf};
use prometeo::topology::relation::{RelationType, TypedEdge};

fn is_valid_word(word: &str) -> bool {
    let w = word.trim();
    if w.is_empty() { return false; }
    
    // Eccezioni cortissime ammesse
    if w == "o" || w == "a" || w == "e" || w == "i" { return true; }
    
    // Lunghezza
    if w.chars().count() < 2 { return false; }
    if w.chars().count() > 25 { return false; }

    // Caratteri validi: alfabetici, spazi, apostrofi, trattini
    for c in w.chars() {
        if !c.is_alphabetic() && c != ' ' && c != '\'' && c != '-' {
            return false;
        }
    }

    // Blacklist di parole palesemente non italiane / monnezza apparsa nei log
    let blacklist = [
        "abb", "about", "multiplayere", "multidisciplinarire", "criaturo", "message",
        "the", "cross", "italiere", "belga", "tinta", "pinastro"
    ];
    if blacklist.contains(&w.to_lowercase().as_str()) { return false; }

    true
}

fn process_tsv_file(path: &Path) -> anyhow::Result<(usize, usize)> {
    let content = fs::read_to_string(path)?;
    let mut cleaned_lines = Vec::new();
    let mut total_lines = 0;
    let mut kept_lines = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            cleaned_lines.push(line.to_string()); // mantieni commenti e righe vuote
            continue;
        }
        
        total_lines += 1;

        // Prova a parsare la riga
        if let Some(edge) = TypedEdge::from_tsv_line(trimmed) {
            // Filtri semantici e sintattici
            if is_valid_word(&edge.subject) && is_valid_word(&edge.object) {
                if edge.confidence >= 0.5 {
                    cleaned_lines.push(line.to_string());
                    kept_lines += 1;
                }
            }
        }
    }

    // Sovrascrive il file
    fs::write(path, cleaned_lines.join("\n"))?;

    Ok((total_lines, total_lines - kept_lines))
}

fn main() -> anyhow::Result<()> {
    println!("=== Inizio purificazione Knowledge Graph ===");
    let kg_dir = Path::new("data/kg");
    
    if !kg_dir.exists() {
        eprintln!("Directory data/kg non trovata!");
        std::process::exit(1);
    }

    let mut total_processed = 0;
    let mut total_removed = 0;

    for entry in fs::read_dir(kg_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|e| e.to_str()) == Some("tsv") {
            // Escludi file specifici se necessario
            match process_tsv_file(&path) {
                Ok((processed, removed)) => {
                    if removed > 0 {
                        println!("File {:?}: {} triple rimosse su {}", path.file_name().unwrap(), removed, processed);
                    }
                    total_processed += processed;
                    total_removed += removed;
                }
                Err(e) => {
                    eprintln!("Errore processando {:?}: {}", path, e);
                }
            }
        }
    }

    println!("============================================");
    println!("Pulizia completata.");
    println!("Totale triple analizzate: {}", total_processed);
    println!("Totale triple rimosse:    {}", total_removed);
    println!("============================================");

    Ok(())
}
