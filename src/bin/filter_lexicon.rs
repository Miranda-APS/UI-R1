/// Script per filtrare il master_lexicon_review.tsv usando un dizionario italiano.
/// Mantiene:
/// 1. I frattali base
/// 2. Le parole sartoriali già curate
/// 3. Le parole PENDING che esistono nel dizionario italiano scaricato (anche composte).

use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("=== Filtraggio del Master Lexicon ===");

    // 1. Caricamento del dizionario italiano
    let dict_path = Path::new("data/kg/dizionario_italiano.txt");
    if !dict_path.exists() {
        anyhow::bail!("Dizionario non trovato in {:?}. Assicurati di averlo scaricato.", dict_path);
    }

    let dict_content = fs::read_to_string(dict_path)?;
    let mut dizionario: HashSet<String> = HashSet::new();
    for line in dict_content.lines() {
        let w = line.trim().to_lowercase();
        if !w.is_empty() {
            dizionario.insert(w);
        }
    }
    println!("Caricate {} parole dal dizionario italiano.", dizionario.len());

    // 2. Lettura del master lexicon
    let in_path = Path::new("data/kg/master_lexicon_review.tsv");
    let out_path = Path::new("data/kg/master_lexicon_cleaned.tsv");

    if !in_path.exists() {
        anyhow::bail!("File {:?} non trovato. Esegui prima export_master_signatures.", in_path);
    }

    let tsv_content = fs::read_to_string(in_path)?;
    let mut out_lines = Vec::new();
    let mut kept_pending = 0;
    let mut discarded_pending = 0;

    for line in tsv_content.lines() {
        // Manteniamo intatti gli header
        if line.starts_with('#') || line.starts_with("WORD\t") || line.is_empty() {
            out_lines.push(line.to_string());
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let word = parts[0];
            let wtype = parts[1];
            // let sig = parts[2];

            if wtype == "FRACTAL" || wtype == "CURATED" {
                // Tieni sempre frattali e parole sartoriali
                out_lines.push(line.to_string());
            } else if wtype == "PENDING" {
                // Per le PENDING, verifichiamo col dizionario
                let mut is_valid = true;
                
                // Ignora parole che contengono numeri o caratteri strani
                if word.chars().any(|c| c.is_numeric() || (!c.is_alphabetic() && c != ' ' && c != '\'')) {
                    is_valid = false;
                } else {
                    // Controlla ogni sotto-parola (se è composta da spazi o apostrofi)
                    let sub_words: Vec<&str> = word.split(|c| c == ' ' || c == '\'').collect();
                    for sw in sub_words {
                        let sw_lower = sw.to_lowercase();
                        // Alcune stop words molto brevi o apostrofate potrebbero mancare, ma in genere le parole valide di 2+ lettere ci sono
                        if sw_lower.len() > 1 && !dizionario.contains(&sw_lower) {
                            // Non trovata
                            is_valid = false;
                            break;
                        }
                    }
                }

                if is_valid {
                    out_lines.push(line.to_string());
                    kept_pending += 1;
                } else {
                    discarded_pending += 1;
                }
            }
        }
    }

    // 3. Salvataggio
    fs::write(out_path, out_lines.join("\n"))?;
    
    println!("Filtro completato con successo!");
    println!("Parole PENDING mantenute: {}", kept_pending);
    println!("Parole PENDING scartate: {}", discarded_pending);
    println!("Il nuovo file pulito si trova in: {}", out_path.display());

    Ok(())
}
