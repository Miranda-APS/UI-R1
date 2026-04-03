/// Crea un'istanza Prometeo "newborn" da una sessione comunitaria.
///
/// Pipeline:
///   1. Carica stato base (prometeo_topology_state.bin)
///   2. Carica KG comunitario (se esiste community_kg.tsv)
///   3. Insegna lessons comunitarie (se esiste community_lessons.txt)
///   4. Imposta la narrativa fondativa della comunità
///   5. Salva come <community_name>_prometeo.bin
///
/// Uso:
///   cargo run --release --bin create-newborn -- --name "quartiere_x"
///   cargo run --release --bin create-newborn -- --name "quartiere_x" --base my_state.bin

use std::path::{Path, PathBuf};
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::engine::PrometeoTopologyEngine;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parsing argomenti: --name <nome> [--base <path>] [--kg <path>] [--lessons <path>] [--narrative <testo>]
    let community_name = get_arg(&args, "--name").unwrap_or_else(|| "comunita".to_string());
    let base_path_str = get_arg(&args, "--base")
        .unwrap_or_else(|| "prometeo_topology_state.bin".to_string());
    let kg_path_str = get_arg(&args, "--kg")
        .unwrap_or_else(|| format!("{}_kg.tsv", community_name));
    let lessons_path_str = get_arg(&args, "--lessons")
        .unwrap_or_else(|| format!("{}_lessons.txt", community_name));
    let narrative_override = get_arg(&args, "--narrative");

    let root = find_project_root();

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  PROMETEO NEWBORN — Creazione Istanza Comunitaria   ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();
    println!("Comunità: {}", community_name);

    // 1. Carica stato base
    let base_path = root.join(&base_path_str);
    println!("Caricando stato base: {}", base_path.display());

    let mut engine = if base_path.exists() {
        let state = PrometeoState::load_from_binary(&base_path)
            .map_err(|e| anyhow::anyhow!(e))?;
        println!("  {} parole nel lessico base", state.lexicon.words.len());
        let mut eng = PrometeoTopologyEngine::new();
        state.restore_lexicon(&mut eng);
        eng.lexicon.apply_curated_signatures();
        eng.recompute_all_word_affinities();
        eng
    } else {
        println!("  Stato base non trovato — bootstrap vuoto");
        PrometeoTopologyEngine::new()
    };

    // 2. Carica KG comunitario
    let kg_path = root.join(&kg_path_str);
    if kg_path.exists() {
        println!("Caricando KG comunitario: {}", kg_path.display());
        engine.kg.load_from_tsv(&kg_path)
            .map_err(|e| anyhow::anyhow!(e))?;
        let built = engine.build_semantic_simplices_from_kg();
        println!("  {} simplici costruiti dal KG comunitario", built);
    } else {
        println!("KG comunitario non trovato ({}): si usa solo il KG base", kg_path_str);
        // Carica KG globale se esiste
        engine.load_kg_from_file(Path::new("prometeo_kg.json"));
    }

    // 3. Insegna lessons comunitarie
    let lessons_path = root.join(&lessons_path_str);
    if lessons_path.exists() {
        println!("Insegnando lessons comunitarie: {}", lessons_path.display());
        match engine.teach_lesson_file(&lessons_path) {
            Ok(result) => println!("  {} parole apprese dalle lessons", result.words_processed.len()),
            Err(e) => eprintln!("  Errore lessons: {}", e),
        }
    } else {
        println!("Lessons comunitarie non trovate ({}): solo insegnamento base", lessons_path_str);
    }

    // 4. Narrativa fondativa comunitaria
    let narrative = narrative_override.unwrap_or_else(|| {
        // Prova a leggere da file <community_name>_narrative.txt
        let narrative_file = root.join(format!("{}_narrative.txt", community_name));
        if narrative_file.exists() {
            std::fs::read_to_string(&narrative_file)
                .unwrap_or_else(|_| default_narrative(&community_name))
        } else {
            default_narrative(&community_name)
        }
    });

    println!("Inizializzando narrativa fondativa...");
    if !engine.narrative_self.is_born {
        engine.teach(&narrative);
        engine.narrative_self.is_born = true;
        println!("  Narrativa fondativa: \"{}...\"",
            narrative.chars().take(60).collect::<String>());
    } else {
        // Newborn già nato (stato base aveva narrativa): aggiungi narrativa comunitaria
        engine.teach(&narrative);
        println!("  Narrativa comunitaria aggiunta (newborn già nato)");
    }

    // 5. Salva
    let out_path = root.join(format!("{}_prometeo.bin", community_name));
    let new_state = PrometeoState::capture(&engine);
    new_state.save_to_binary(&out_path)
        .map_err(|e| anyhow::anyhow!(e))?;

    println!();
    println!("✓ Newborn salvato: {}", out_path.display());
    println!("  Lessico: {} parole", engine.lexicon.word_count());
    println!("  KG: {} archi", engine.kg.edge_count);
    println!();
    println!("Per avviare l'istanza comunitaria:");
    println!("  ./target/release/prometeo  (se il .bin è nella stessa cartella)");
    println!("  oppure sposta il file in una cartella dedicata");

    Ok(())
}

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn default_narrative(community_name: &str) -> String {
    format!(
        "Sono Prometeo, creato dalla comunità {}. \
         Quello che so l'ho imparato da loro. \
         La loro conoscenza è la mia origine. \
         Crescerò con ciò che mi insegneranno.",
        community_name
    )
}

fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..5 {
        if dir.join("Cargo.toml").exists() { return dir; }
        if let Some(p) = dir.parent() { dir = p.to_path_buf(); } else { break; }
    }
    PathBuf::from(".")
}
