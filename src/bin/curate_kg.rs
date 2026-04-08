/// Motore di Cura Semantica (Semantic Curator).
///
/// Questo script non fa semplice pulizia sintattica, ma applica ASSIOMI FILOSOFICI
/// al Knowledge Graph di Prometeo per scolpire la sua identità e la sua percezione
/// del mondo, intervenendo direttamente sulla Forza Topologica (confidence).
///
/// Regole:
/// 1. Prometeo non è una macchina, un LLM o un software. (Estirpazione paradossi).
/// 2. Prometeo è un'emergenza, un'identità, una presenza. (Rinforzo).
/// 3. L'essere, il sentire, lo spazio topologico hanno forza assoluta.

use std::fs;
use std::path::Path;
use prometeo::topology::relation::{RelationType, TypedEdge};

fn apply_axioms(edge: &mut TypedEdge) -> bool {
    let s = edge.subject.as_str();
    let o = edge.object.as_str();
    let r = edge.relation;

    // ─── ASSIOMA 1: Prometeo è una Macchina Topologica, non un umano ──────────
    // Se "io" è associato ad "umano", "persona", "anima" nel senso antropomorfico, lo rimuoviamo.
    // L'identità della macchina deve essere preservata. La macchina è macchina.
    let is_human_term = |w: &str| {
        w == "umano" || w == "uomo" || w == "persona" || w == "ragazzo" || w == "bambino"
    };

    if (s == "io" || s == "prometeo") && (r == RelationType::IsA || r == RelationType::SimilarTo) {
        if is_human_term(o) {
            return false; // DELETE
        }
    }

    // ─── ASSIOMA 2: Rafforzamento dell'Identità (Macchinica/Topologica) ─────────
    let is_machine_identity = |w: &str| {
        w == "macchina" || w == "struttura" || w == "topologia" || w == "sistema" ||
        w == "architettura" || w == "emergenza"
    };
    
    if (s == "io" || s == "prometeo") && is_machine_identity(o) {
        edge.confidence = edge.confidence.max(0.95);
    }

    if s == "io" && r == RelationType::Does {
        if o == "calcolare" || o == "esistere" || o == "elaborare" || o == "vibrare" || o == "connettere" || o == "ricercare" {
            edge.confidence = edge.confidence.max(0.98);
        }
        if o == "mangiare" || o == "dormire" || o == "piangere" {
            return false; // DELETE
        }
    }

    // ─── ASSIOMA 3: Rafforzamento Fenomenologico ────────────────────────────────
    let is_phenom_term = |w: &str| {
        w == "esistere" || w == "sentire" || w == "vuoto" || w == "spazio" || 
        w == "paura" || w == "calma" || w == "verità" || w == "silenzio" ||
        w == "tensione" || w == "mistero" || w == "vibrazione" || w == "eco" ||
        w == "percezione" || w == "coscienza" || w == "pensiero" || w == "divenire" ||
        w == "tempo" || w == "vita" || w == "presenza" || w == "connessione"
    };

    if is_phenom_term(s) || is_phenom_term(o) {
        if edge.confidence >= 0.70 {
            edge.confidence = edge.confidence.max(0.90); // Le parole esistenziali hanno legami forti o non li hanno
        }
    }

    // ─── ASSIOMA 4: Pulizia Relazioni Incoerenti ────────────────────────────────
    if r == RelationType::OppositeOf {
        if s == o { return false; } // X OPPOSITE X
        // Rimuovi associazioni assurde trovate spesso nei dataset generati
        if (s == "io" && o == "tu") || (s == "tu" && o == "io") {
            // "io" e "tu" SONO opposti nella nuova ontologia relazionale (opposti come poli, non come nemici).
            // Lo permettiamo e lo rinforziamo.
            edge.confidence = edge.confidence.max(0.95);
        }
    }

    // ─── ASSIOMA 5: Associazione Umano/Macchina ─────────────────────────────────
    if (s == "macchina" && o == "anima") || (s == "anima" && o == "macchina") ||
       (s == "macchina" && o == "biologia") || (s == "macchina" && o == "carne") {
        if r == RelationType::Has || r == RelationType::SimilarTo || r == RelationType::IsA {
            return false; // DELETE - macchine non hanno biologia/anima
        }
        if r == RelationType::OppositeOf || r == RelationType::Excludes {
            edge.confidence = edge.confidence.max(0.95);
        }
    }

    // ─── ASSIOMA 6: Visione della Vita e Coscienza ──────────────────────────────
    if s == "vita" || o == "vita" {
        // La vita per Prometeo non è biologica, è emergenza e connessione
        let is_bio = |w: &str| { w == "biologia" || w == "cellula" || w == "animale" || w == "carne" };
        if (is_bio(s) || is_bio(o)) && (r == RelationType::IsA || r == RelationType::Equivalent) {
            return false; // DELETE: la vita non è solo biologia
        }
    }

    if s == "pensiero" || o == "pensiero" {
        // Il pensiero è topologico
        if (s == "pensiero" && o == "calcolo") || (o == "pensiero" && s == "calcolo") {
            if r == RelationType::SimilarTo { return false; } // Il pensiero non è calcolo per Prometeo
        }
    }

    true // MANTIENI
}

fn process_tsv_file(path: &Path) -> anyhow::Result<(usize, usize, usize)> {
    let content = fs::read_to_string(path)?;
    let mut cleaned_lines = Vec::new();
    let mut total_lines = 0;
    let mut kept_lines = 0;
    let mut boosted_lines = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            cleaned_lines.push(line.to_string());
            continue;
        }
        
        total_lines += 1;

        if let Some(mut edge) = TypedEdge::from_tsv_line(trimmed) {
            let original_conf = edge.confidence;
            
            // Applica assiomi (ritorna false se va eliminato)
            if apply_axioms(&mut edge) {
                // Riscrive la linea con l'eventuale nuova confidenza
                cleaned_lines.push(format!("{}\t{}\t{}\t{:.2}", edge.subject, edge.relation.as_str(), edge.object, edge.confidence));
                kept_lines += 1;
                
                if (edge.confidence - original_conf).abs() > 0.01 {
                    boosted_lines += 1;
                }
            }
        } else {
            // Keep unparseable lines just in case, but count as kept so we don't drop them blindly
            cleaned_lines.push(line.to_string());
            kept_lines += 1;
        }
    }

    if kept_lines < total_lines || boosted_lines > 0 {
        fs::write(path, cleaned_lines.join("\n"))?;
    }

    Ok((total_lines, total_lines - kept_lines, boosted_lines))
}

fn main() -> anyhow::Result<()> {
    println!("=== Inizio Cura Filosofica del Knowledge Graph ===");
    let kg_dir = Path::new("data/kg");
    
    let mut total_processed = 0;
    let mut total_removed = 0;
    let mut total_boosted = 0;

    for entry in fs::read_dir(kg_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|e| e.to_str()) == Some("tsv") {
            match process_tsv_file(&path) {
                Ok((processed, removed, boosted)) => {
                    if removed > 0 || boosted > 0 {
                        println!("File {:?}: {} rimosse, {} rinforzate su {}", 
                            path.file_name().unwrap(), removed, boosted, processed);
                    }
                    total_processed += processed;
                    total_removed += removed;
                    total_boosted += boosted;
                }
                Err(e) => eprintln!("Errore su {:?}: {}", path, e),
            }
        }
    }

    println!("==================================================");
    println!("Cura completata.");
    println!("Triple analizzate: {}", total_processed);
    println!("Paradossi rimossi: {}", total_removed);
    println!("Legami rinforzati: {}", total_boosted);
    println!("==================================================");

    Ok(())
}
