//! Prometeo — Sistema cognitivo topologico 8D
//!
//! CLI interattiva: il sistema apprende per esposizione,
//! sogna quando non perturbato, e risponde dal campo.

use std::io::{self, Write, BufRead};
use std::path::PathBuf;
use std::fs;

use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::composition::{compose_phrase, analyze_composition};
use prometeo::topology::dream::SleepPhase;
use prometeo::topology::locus::MovementKind;


/// File di stato in formato binario SimplDB (default, veloce).
const BINARY_STATE: &str = "prometeo_topology_state.bin";
/// File di stato in formato JSON (legacy, human-readable, per debug/backup).
const STATE_FILE: &str = "prometeo_topology_state.json";

fn main() {
    println!();
    println!("  PROMETEO — Topologia Cognitiva 8D");
    println!("  La macchina per cio che e.");
    println!();

    // Inizializza engine
    let mut engine = PrometeoTopologyEngine::new();

    // Caricamento stato: prova SimplDB (binario, veloce) poi JSON (legacy).
    let bin_path = PathBuf::from(BINARY_STATE);
    let json_path = PathBuf::from(STATE_FILE);

    if bin_path.exists() {
        // Formato binario: memory-mapped, avvio rapido
        match PrometeoState::load_from_binary(&bin_path) {
            Ok(state) => {
                let words_count = state.lexicon.words.len();
                let perturbations = state.total_perturbations;
                state.restore_lexicon(&mut engine);
                engine.lexicon.apply_curated_signatures();
                engine.lexicon.load_phenomenology_signatures();
                engine.recompute_all_word_affinities();
                engine.update_semantic_axes();
                engine.recalibrate_emergent_dimensions();
                println!("  [stato caricato (SimplDB): {} perturbazioni, {} parole]",
                    perturbations, words_count);
            }
            Err(e) => {
                println!("  [errore SimplDB: {}]", e);
                // Fallback al JSON se il binario è corrotto
                if json_path.exists() {
                    println!("  [tentativo fallback JSON...]");
                    match PrometeoState::load_from_file(&json_path) {
                        Ok(state) => {
                            let words_count = state.lexicon.words.len();
                            let perturbations = state.total_perturbations;
                            state.restore_lexicon(&mut engine);
                            engine.lexicon.apply_curated_signatures();
                            engine.recompute_all_word_affinities();
                            engine.update_semantic_axes();
                            engine.recalibrate_emergent_dimensions();
                            println!("  [stato caricato (JSON fallback): {} perturbazioni, {} parole]",
                                perturbations, words_count);
                        }
                        Err(e2) => println!("  [impossibile caricare stato: {}]", e2),
                    }
                }
            }
        }
    } else if json_path.exists() {
        // Primo avvio con file JSON legacy: carica e migra automaticamente a SimplDB
        match PrometeoState::load_from_file(&json_path) {
            Ok(state) => {
                let words_count = state.lexicon.words.len();
                let perturbations = state.total_perturbations;
                state.restore_lexicon(&mut engine);
                engine.lexicon.apply_curated_signatures();
                engine.lexicon.load_phenomenology_signatures();
                engine.recompute_all_word_affinities();
                engine.update_semantic_axes();
                engine.recalibrate_emergent_dimensions();
                println!("  [stato caricato (JSON): {} perturbazioni, {} parole]",
                    perturbations, words_count);
                // Auto-migrazione: crea il file binario per i prossimi avvii
                println!("  [migrazione a SimplDB in corso...]");
                let migrated = PrometeoState::capture(&engine);
                let est_kb = migrated.binary_size_estimate() / 1024;
                match migrated.save_to_binary(&bin_path) {
                    Ok(()) => println!("  [SimplDB creato: {} KB → avvii futuri più veloci]", est_kb),
                    Err(e) => println!("  [migrazione fallita: {}]", e),
                }
            }
            Err(e) => println!("  [impossibile caricare stato: {}]", e),
        }
    } else {
        println!("  [prima sessione — 36 parole cardinali]");
        // Auto-insegnamento: carica tutte le lezioni dalla cartella lessons/
        auto_teach_lessons(&mut engine);
    }

    // Carica il Knowledge Graph — la comprensione semantica del mondo.
    // Senza KG, la propagazione è cieca: i vicini vengono solo dai simplessi
    // (co-occorrenze cristallizzate), non dalle relazioni logiche (IS_A, CAUSES...).
    let kg_path = std::path::PathBuf::from("prometeo_kg.json");
    if kg_path.exists() {
        engine.load_kg_from_file(&kg_path);
        // CRITICO: ricostruire PF1 DOPO il KG — altrimenti i vicini nel campo
        // non includono le relazioni semantiche (IS_A, CAUSES...).
        engine.rebuild_pf_field();
        println!("  [KG caricato: {} archi → PF1 ricostruito con vicini semantici]",
            engine.kg.edge_count);
    }

    // Phase 43B — Narrativa fondativa: chiamata solo al primo avvio (is_born == false).
    // Se Prometeo è già nato (stato caricato), is_born = true e la funzione esce subito.
    if !engine.narrative_self.is_born {
        engine.initialize_founding_narrative();
        println!("  [narrativa fondativa cristallizzata — Prometeo nasce]");
    }

    let report = engine.report();
    println!("  [frattali: {}, simplessi: {}, vocabolario: {}]",
        report.fractal_count, report.simplex_count, report.vocabulary_size);
    println!();
    println!("  Comandi: :report :active :dream :vital :ask :homology :why :intro :dial");
    println!("           :where :see :inside :project  :reason <a> <b>  :abduce");
    println!("           :grow :confidence  :create <seme>  :metaphor <sorgente>");
    println!("           :nav <da> <a>  :analogy <a> <b> <c>  :analyze <frase>");
    println!("           :teach <frase>  :lesson/:learn <file>  :learn-all  :compact <file>  :will  :compound  :field  :phase");
    println!("           :emergent  :pop  :thoughts/:pensieri  :episodes  :vision  :echo  :percept  :operators");
    println!("           :know <fatto> [dominio]  :procedures  :save :quit");
    println!("           :read/:leggi <file>  — lettura attiva con analisi e curiosità");
    println!("           :perceive <svg>  — esperimento: percezione visiva SVG");
    println!("           :needs  :desires  — gerarchia bisogni e desideri attivi");
    println!();

    let stdin = io::stdin();
    let mut idle_ticks = 0u32;
    loop {
        // Mostra prompt con stato del sogno
        let phase_indicator = match engine.dream.phase {
            SleepPhase::Awake => "~",
            SleepPhase::WakefulDream { .. } => ".",
            SleepPhase::LightSleep { .. } => "z",
            SleepPhase::DeepSleep { .. } => "Z",
            SleepPhase::REM { .. } => "*",
        };
        let locus_indicator = match engine.where_am_i() {
            Some((name, horizon)) => format!("{}, {:.1}", name, horizon),
            None => "—".to_string(),
        };
        print!("  [{}] ({}) > ", phase_indicator, locus_indicator);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin.lock().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();

        if input.is_empty() {
            // Tick autonomo (sogno + vita autonoma)
            idle_ticks += 1;
            if idle_ticks % 3 == 0 {
                let prev_phase = engine.dream.phase;
                let result = engine.autonomous_tick();
                if engine.dream.phase != prev_phase {
                    println!("    [...fase: {:?}]", engine.dream.phase);
                }
                // Espressione spontanea: l'entita parla da sola
                if let Some(ref text) = result.spontaneous {
                    println!("\n    [spontaneo] {}\n", text.text);
                }
                // Curiosita spontanea: l'entita domanda da sola
                if let Some(ref q) = result.question {
                    println!("\n    [curiosita] {}\n", q.text);
                }
            }
            continue;
        }

        idle_ticks = 0;

        // Comandi speciali
        if input.starts_with(':') {
            match input {
                ":quit" | ":q" => {
                    // Salva stato prima di uscire
                    save_state(&engine);
                    println!("\n  [sessione terminata, stato salvato]\n");
                    break;
                }
                ":report" | ":r" => {
                    print_report(&engine);
                    continue;
                }
                ":active" | ":a" => {
                    print_active(&engine);
                    continue;
                }
                ":dream" | ":d" => {
                    print_dream(&mut engine);
                    continue;
                }
                ":vital" | ":v" => {
                    print_vital(&mut engine);
                    continue;
                }
                ":ask" => {
                    print_questions(&mut engine);
                    continue;
                }
                ":homology" | ":h" => {
                    print_homology(&engine);
                    continue;
                }
                ":why" | ":w" => {
                    print_why(&engine);
                    continue;
                }
                ":intro" | ":i" => {
                    print_introspection(&engine);
                    continue;
                }
                ":dial" | ":dialogue" => {
                    print_dialogue(&engine);
                    continue;
                }
                ":where" | ":pos" => {
                    print_where(&engine);
                    continue;
                }
                ":see" => {
                    print_see(&engine);
                    continue;
                }
                ":inside" | ":sub" => {
                    print_inside(&engine);
                    continue;
                }
                ":project" | ":holo" => {
                    print_projection(&engine);
                    continue;
                }
                ":abduce" => {
                    print_abductions(&engine);
                    continue;
                }
                ":grow" | ":g" => {
                    print_growth(&mut engine);
                    continue;
                }
                ":confidence" | ":conf" => {
                    print_confidence(&engine);
                    continue;
                }

                _ if input.starts_with(":create ") || input.starts_with(":cr ") => {
                    let seed_name = if input.starts_with(":create ") {
                        &input[8..]
                    } else {
                        &input[4..]
                    };
                    print_creative_session(&mut engine, seed_name.trim());
                    continue;
                }
                _ if input.starts_with(":metaphor ") || input.starts_with(":met ") => {
                    let source_name = if input.starts_with(":metaphor ") {
                        &input[10..]
                    } else {
                        &input[5..]
                    };
                    print_metaphors(&engine, source_name.trim());
                    continue;
                }
                _ if input.starts_with(":project ") || input.starts_with(":holo ") => {
                    let target_name = if input.starts_with(":project ") {
                        &input[9..]
                    } else {
                        &input[6..]
                    };
                    print_single_projection(&engine, target_name.trim());
                    continue;
                }
                _ if input.starts_with(":reason ") => {
                    let args: Vec<&str> = input[8..].split_whitespace().collect();
                    if args.len() >= 2 {
                        print_reasoning(&engine, args[0], args[1]);
                    } else {
                        println!("    [uso: :reason <da> <a>  — es. :reason spazio relazione]");
                    }
                    continue;
                }
                ":save" | ":s" => {
                    save_state(&engine);
                    println!("    [stato salvato (SimplDB)]");
                    continue;
                }
                ":identita" | ":identity" => {
                    let id = &engine.identity;
                    println!();
                    println!("  ◈ IDENTITÀ OLOGRAFICA");
                    println!("  ─────────────────────────────────────────");
                    println!("  Aggiornamenti REM: {}", id.update_count);
                    println!("  Continuità:        {:.3}", id.continuity);
                    if id.is_in_crisis()  { println!("  ⚠  CRISI identitaria (continuità < 0.65)"); }
                    if id.is_stagnant()   { println!("  ⚠  STAGNAZIONE (campo immobile)"); }

                    // Firma 8D
                    println!();
                    println!("  Firma del sé (8D):");
                    let dim_names = ["Agency","Permanenza","Intensità","Tempo","Confine","Complessità","Definizione","Valenza"];
                    for (i, &v) in id.self_signature.iter().enumerate() {
                        let bar: String = "█".repeat((v * 20.0) as usize);
                        println!("    {:12} {:5.3} {}", dim_names.get(i).unwrap_or(&"?"), v, bar);
                    }

                    // Helper: nome dal registry (copre tutti i 64 frattali)
                    let fractal_label = |fid: u32| -> String {
                        engine.registry.get(fid)
                            .map(|f| f.name.clone())
                            .unwrap_or_else(|| format!("#{}", fid))
                    };

                    // Frattale dominante
                    if let Some((fid, strength)) = id.dominant_fractal() {
                        println!();
                        println!("  Frattale dominante: {} — peso relativo {:.1}%",
                            fractal_label(fid), strength * 100.0);
                    }

                    // Direzione di movimento
                    if let Some((fid, delta)) = id.movement_direction() {
                        println!("  Verso:              {} — Δ {:.4}", fractal_label(fid), delta);
                    }

                    // Tensione primaria
                    match &id.primary_tension {
                        Some((a, b)) => println!("  Tensione primaria:  {} ↔ {} (persistenza: {} cicli)",
                            a, b, id.tension_persistence),
                        None => println!("  Tensione primaria:  non ancora consolidata"),
                    }

                    // Top frattali della proiezione personale — normalizzati al massimo per display
                    let mut proj: Vec<(usize, f64)> = id.personal_projection.iter()
                        .enumerate()
                        .filter(|(_, &v)| v > 0.0)
                        .map(|(i, &v)| (i, v))
                        .collect();
                    proj.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    if !proj.is_empty() {
                        let max_w = proj[0].1.max(1e-9);
                        println!();
                        println!("  Top frattali personali (relativo al massimo):");
                        for (fid, w) in proj.iter().take(10) {
                            let name = fractal_label(*fid as u32);
                            let rel = w / max_w;
                            let bar: String = "▓".repeat((rel * 24.0) as usize);
                            println!("    {:18} {:5.3}  {}", name, rel, bar);
                        }
                    }
                    println!();
                    continue;
                }
                _ if input.starts_with(":opinioni") => {
                    let path = {
                        let rest = input.trim_start_matches(":opinioni").trim();
                        if rest.is_empty() { "prometeo_opinions.md" } else { rest }
                    };
                    let doc = prometeo::topology::opinion::generate_opinion_document(&engine);
                    match std::fs::write(path, &doc) {
                        Ok(_) => println!("    [opinioni salvate in '{}' ({} caratteri)]", path, doc.len()),
                        Err(e) => println!("    [errore scrittura: {}]", e),
                    }
                    continue;
                }
                ":export-json" | ":json" => {
                    export_json(&engine);
                    continue;
                }
                ":will" => {
                    print_will(&engine);
                    continue;
                }
                ":infant" => {
                    engine = PrometeoTopologyEngine::new();
                    let report = engine.report();
                    println!();
                    println!("    [entita riavviata come infante]");
                    println!("    [vocabolario: {} parole cardinali]", report.vocabulary_size);
                    println!("    [usa :lesson <file> per insegnare]");
                    println!();
                    continue;
                }
                ":curriculum" | ":curr" => {
                    print_curriculum(&engine);
                    continue;
                }
                ":axes" | ":assi" => {
                    print_axes(&engine);
                    continue;
                }
                ":compound" | ":composti" => {
                    print_compounds(&engine);
                    continue;
                }
                ":field" | ":campo" => {
                    print_field(&engine);
                    continue;
                }
                _ if input.starts_with(":phase ") || input.starts_with(":fasi ") => {
                    let rest = if input.starts_with(":phase ") { &input[7..] } else { &input[6..] };
                    let args: Vec<&str> = rest.split_whitespace().collect();
                    if args.len() >= 2 {
                        print_phase_pair(&engine, args[0], args[1]);
                    } else {
                        print_phases(&engine);
                    }
                    continue;
                }
                ":phase" | ":fasi" | ":valence" | ":valenza" => {
                    print_phases(&engine);
                    continue;
                }
                ":bridges" | ":ponti" => {
                    print_bridges(&engine);
                    continue;
                }
                ":population" | ":pop" => {
                    print_population(&engine);
                    continue;
                }
                ":emergent" | ":emergenti" => {
                    print_emergent(&engine);
                    continue;
                }
                ":thoughts" | ":pensieri" => {
                    print_thoughts(&engine);
                    continue;
                }
                ":episodes" | ":episodi" => {
                    print_episodes(&engine);
                    continue;
                }
                ":vision" | ":visione" => {
                    print_vision(&engine);
                    continue;
                }
                ":echo" | ":eco" => {
                    print_echo(&engine);
                    continue;
                }
                ":percept" | ":percepisci" => {
                    print_perceptual_field(&engine);
                    continue;
                }
                ":operators" | ":operatori" => {
                    print_operators(&engine);
                    continue;
                }
                ":procedures" | ":procedure" | ":knowledge" | ":conoscenza" => {
                    print_knowledge(&engine);
                    continue;
                }
                _ if input.starts_with(":know ") || input.starts_with(":impara ") => {
                    let rest = if input.starts_with(":know ") {
                        &input[6..]
                    } else {
                        &input[8..]
                    }.trim();
                    // Formato: ":know <contenuto> [dominio]"
                    // Il dominio opzionale è l'ultima parola se è un identificatore corto
                    let domain_keywords = ["social", "dialogo", "procedurale", "epistemica",
                                          "dialogue", "procedural", "epistemic", "syntax"];
                    let (content, domain) = if let Some(last_space) = rest.rfind(' ') {
                        let last_word = &rest[last_space+1..];
                        if domain_keywords.iter().any(|&k| k == last_word) {
                            (&rest[..last_space], last_word)
                        } else {
                            (rest, "dialogo")
                        }
                    } else {
                        (rest, "dialogo")
                    };
                    engine.teach_knowledge(content, domain);
                    println!("  [conoscenza registrata: \"{}\" | dominio: {}]", content, domain);
                    println!("  [totale voci: {}]", engine.knowledge_base.entry_count());
                    continue;
                }
                ":reinforce" | ":rinforza" => {
                    print_reinforce(&mut engine);
                    continue;
                }
                ":reteach" => {
                    reteach_all(&mut engine);
                    continue;
                }
                ":learn-all" | ":la" => {
                    learn_all_pending(&mut engine);
                    continue;
                }
                _ if input.starts_with(":compact ") || input.starts_with(":c ") => {
                    let path = if input.starts_with(":compact ") {
                        &input[9..]
                    } else {
                        &input[3..]
                    };
                    run_compact_lesson(&mut engine, path.trim());
                    continue;
                }
                _ if input.starts_with(":axis ") || input.starts_with(":asse ") => {
                    let word = if input.starts_with(":axis ") {
                        &input[6..]
                    } else {
                        &input[6..]
                    };
                    print_word_on_axes(&engine, word.trim());
                    continue;
                }
                _ if input.starts_with(":tension ") || input.starts_with(":tensione ") => {
                    let rest = if input.starts_with(":tension ") {
                        &input[9..]
                    } else {
                        &input[10..]
                    };
                    let args: Vec<&str> = rest.split_whitespace().collect();
                    if args.len() >= 2 {
                        print_tension(&engine, args[0], args[1]);
                    } else {
                        println!("    [uso: :tension <polo_a> <polo_b>  — es. :tension caldo freddo]");
                    }
                    continue;
                }
                _ if input.starts_with(":teach ") || input.starts_with(":t ") => {
                    let text = if input.starts_with(":teach ") {
                        &input[7..]
                    } else {
                        &input[3..]
                    };
                    print_teach(&mut engine, text.trim());
                    continue;
                }
                _ if input.starts_with(":lesson ") || input.starts_with(":l ")
                    || input.starts_with(":learn ") => {
                    let path = if input.starts_with(":lesson ") {
                        &input[8..]
                    } else if input.starts_with(":learn ") {
                        &input[7..]
                    } else {
                        &input[3..]
                    };
                    run_lesson(&mut engine, path.trim());
                    continue;
                }
                _ if input.starts_with(":emergent ") || input.starts_with(":emergenti ") => {
                    let word = if input.starts_with(":emergent ") {
                        &input[10..]
                    } else {
                        &input[11..]
                    };
                    print_word_emergent(&engine, word.trim());
                    continue;
                }
                _ if input.starts_with(":analyze ") || input.starts_with(":an ") => {
                    let phrase_text = if input.starts_with(":analyze ") {
                        &input[9..]
                    } else {
                        &input[4..]
                    };
                    print_analysis(&mut engine, phrase_text);
                    continue;
                }
                _ if input.starts_with(":nav ") => {
                    let args: Vec<&str> = input[5..].split_whitespace().collect();
                    if args.len() >= 2 {
                        print_navigation(&engine, args[0], args[1]);
                    } else {
                        println!("    [uso: :nav <da> <a>  — es. :nav spazio relazione]");
                    }
                    continue;
                }
                _ if input.starts_with(":analogy ") => {
                    let args: Vec<&str> = input[9..].split_whitespace().collect();
                    if args.len() >= 3 {
                        print_analogy(&engine, args[0], args[1], args[2]);
                    } else {
                        println!("    [uso: :analogy <a> <b> <c>  — es. :analogy spazio tempo ego]");
                    }
                    continue;
                }
                _ if input.starts_with(":read ") || input.starts_with(":leggi ") => {
                    let path = if input.starts_with(":read ") { &input[6..] } else { &input[7..] };
                    run_read_command(&mut engine, path.trim());
                    continue;
                }
                _ if input.starts_with(":perceive ") || input.starts_with(":see-svg ") => {
                    let svg = if input.starts_with(":perceive ") {
                        &input[10..]
                    } else {
                        &input[9..]
                    };
                    run_perceive_svg(&mut engine, svg.trim());
                    continue;
                }
                _ => {
                    println!("    [comando sconosciuto: {}]", input);
                    continue;
                }
            }
        }

        // Input normale: perturba il campo e genera risposta
        let _response = engine.receive(input);

        // Mostra il movimento del locus
        if let Some(ref mov) = engine.last_movement {
            let dest_name = engine.registry.get(mov.to)
                .map(|f| f.name.as_str())
                .unwrap_or("?");
            match mov.kind {
                MovementKind::Origin => {
                    println!("    [@ {} (primo posizionamento)]", dest_name);
                }
                MovementKind::Traverse => {
                    println!("    [-> {} (traverse, {:.1})]", dest_name, mov.distance);
                }
                MovementKind::Jump => {
                    println!("    [=> {} (jump, {:.1})]", dest_name, mov.distance);
                }
                MovementKind::Drift => {
                    println!("    [~> {} (drift, {:.1})]", dest_name, mov.distance);
                }
            }
        }

        // Genera risposta testuale guidata dalla volonta (Phase 3 se possibile)
        let generated = engine.generate_willed();

        // Mostra intenzione + archetipo usato (una riga discreta)
        if let Some(will) = engine.current_will() {
            let intent_label = match &will.intention {
                prometeo::topology::will::Intention::Express { urgency, .. } =>
                    format!("esprimere ({:.0}%)", urgency * 100.0),
                prometeo::topology::will::Intention::Explore { pull, .. } =>
                    format!("esplorare ({:.0}%)", pull * 100.0),
                prometeo::topology::will::Intention::Question { urgency, .. } =>
                    format!("domandare ({:.0}%)", urgency * 100.0),
                prometeo::topology::will::Intention::Remember { resonance } =>
                    format!("ricordare ({:.0}%)", resonance * 100.0),
                prometeo::topology::will::Intention::Withdraw { .. } =>
                    "tacere".to_string(),
                prometeo::topology::will::Intention::Reflect =>
                    "riflettere".to_string(),
                prometeo::topology::will::Intention::Dream { .. } =>
                    "sognare".to_string(),
                prometeo::topology::will::Intention::Instruct { .. } =>
                    "istruire".to_string(),
            };
            // Indica se Phase 3 (traduzione strutturata) e attiva
            let gen_label = if generated.cluster_count == 1 {
                "strutturato".to_string()
            } else {
                format!("{} cluster", generated.cluster_count)
            };
            println!("    [volonta: {} | {}]", intent_label, gen_label);
        }

        println!();
        println!("    {}", generated.text);
        println!();
    }
}


fn print_report(engine: &PrometeoTopologyEngine) {
    let report = engine.report();
    println!();
    println!("    === STATO DEL SISTEMA ===");
    println!("    Frattali:       {}", report.fractal_count);
    println!("    Simplessi:      {}", report.simplex_count);
    println!("    Dim. massima:   {}", report.max_dimension);
    println!("    Componenti:     {}", report.connected_components);
    println!("    Vocabolario:    {} parole", report.vocabulary_size);
    println!("    Dim. emergenti: {}", report.emergent_dimensions);
    println!("    STM:            {} impronte", report.stm_count);
    println!("    MTM:            {} impronte", report.mtm_count);
    println!("    LTM:            {} impronte", report.ltm_count);
    println!("    Fase sogno:     {:?}", report.sleep_phase);
    println!("    Cicli sogno:    {}", report.dream_cycles);
    println!("    Perturbazioni:  {}", report.total_perturbations);
    println!("    --- Campo Parole ---");
    println!("    Vertici:        {}", report.word_field_vertices);
    println!("    Archi:          {}", report.word_field_edges);
    println!("    Energia:        {:.4}", report.word_field_energy);
    println!();
}

fn print_phases(engine: &PrometeoTopologyEngine) {
    use prometeo::topology::word_topology::WordTopology;

    println!();
    println!("    === FASI DEGLI ARCHI ===");

    // Opposizioni (fase > 2*PI/3 = ~120°)
    let oppositions = engine.word_topology.find_oppositions(2.0 * std::f64::consts::PI / 3.0);
    if oppositions.is_empty() {
        println!("    [nessuna opposizione rilevata (fase < 120°)]");
    } else {
        println!("    --- OPPOSIZIONI (fase > 120°) ---");
        for (word_a, word_b, phase) in oppositions.iter().take(20) {
            let degrees = phase.to_degrees();
            let label = WordTopology::phase_label(*phase);
            println!("    {:<15} <~> {:<15}  fase={:.1}°  [{}]", word_a, word_b, degrees, label);
        }
        println!("    Totale opposizioni: {}", oppositions.len());
    }
    println!();

    // Coppie contrastive note
    let known_pairs = [
        ("gioia", "tristezza"), ("luce", "buio"), ("caldo", "freddo"),
        ("vicino", "lontano"), ("forte", "debole"), ("dentro", "fuori"),
        ("grande", "piccolo"), ("alto", "basso"), ("veloce", "lento"),
        ("bello", "brutto"),
    ];
    println!("    --- COPPIE CONTRASTIVE ---");
    for (a, b) in &known_pairs {
        if let Some(phase) = engine.word_topology.edge_phase(a, b) {
            let degrees = phase.to_degrees();
            let label = WordTopology::phase_label(phase);
            println!("    {:<12} / {:<12}  fase={:>6.1}°  [{}]", a, b, degrees, label);
        } else {
            println!("    {:<12} / {:<12}  [nessun arco]", a, b);
        }
    }
    println!();

    // Risonanze (fase < PI/3 = ~60°)
    let resonances = engine.word_topology.find_resonances(std::f64::consts::PI / 3.0);
    if resonances.is_empty() {
        println!("    [nessuna risonanza forte rilevata (fase > 60°)]");
    } else {
        println!("    --- RISONANZE (fase < 60°) ---");
        for (word_a, word_b, phase) in resonances.iter().take(20) {
            let degrees = phase.to_degrees();
            let label = WordTopology::phase_label(*phase);
            println!("    {:<15} <=> {:<15}  fase={:.1}°  [{}]", word_a, word_b, degrees, label);
        }
        println!("    Totale risonanze: {}", resonances.len());
    }
    println!();
}

fn print_phase_pair(engine: &PrometeoTopologyEngine, word_a: &str, word_b: &str) {
    use prometeo::topology::word_topology::WordTopology;

    println!();
    println!("    === FASE: {} / {} ===", word_a, word_b);

    // Fase arco
    if let Some(phase) = engine.word_topology.edge_phase(word_a, word_b) {
        let degrees = phase.to_degrees();
        let label = WordTopology::phase_label(phase);
        println!("    Fase:   {:.1}°  [{}]", degrees, label);
    } else {
        println!("    [nessun arco diretto tra '{}' e '{}']", word_a, word_b);
    }

    // Dati operatore dal lessico
    let pat_a = engine.lexicon.get(word_a);
    let pat_b = engine.lexicon.get(word_b);
    match (pat_a, pat_b) {
        (Some(pa), Some(pb)) => {
            let neg_ab = pa.co_negated.get(word_b).copied().unwrap_or(0);
            let neg_ba = pb.co_negated.get(word_a).copied().unwrap_or(0);
            let aff_ab = pa.co_affirmed.get(word_b).copied().unwrap_or(0);
            let aff_ba = pb.co_affirmed.get(word_a).copied().unwrap_or(0);
            let total_neg = neg_ab + neg_ba;
            let total_aff = aff_ab + aff_ba;
            let total = total_neg + total_aff;
            let cooc = pa.co_occurrences.get(word_b).copied().unwrap_or(0)
                     + pb.co_occurrences.get(word_a).copied().unwrap_or(0);
            if total > 0 {
                let ratio = total_neg as f64 / total as f64;
                let label = if ratio > 0.7 { "opposizione" } else if ratio > 0.4 { "tensione" } else { "risonanza" };
                println!("    affirm={}  neg={}  ratio={:.2}  → {}  (cooc neutre: {})",
                         total_aff, total_neg, ratio, label, cooc);
            } else if cooc > 0 {
                println!("    [solo co-occorrenze neutre: {}  — nessun dato operatore]", cooc);
                println!("    [servi :reteach per popolare co_affirmed/co_negated]");
            } else {
                println!("    [nessun dato operatore ne co-occorrenze — parole rare o mai co-attivate]");
            }
        }
        (None, _) => println!("    ['{}' non e nel lessico]", word_a),
        (_, None) => println!("    ['{}' non e nel lessico]", word_b),
    }
    println!();
}

fn print_field(engine: &PrometeoTopologyEngine) {
    println!();
    println!("    === CAMPO TOPOLOGICO PAROLE ===");
    println!("    Vertici: {}  Archi: {}  Densita: {:.6}",
        engine.word_topology.vertex_count(),
        engine.word_topology.edge_count(),
        engine.word_topology.density());
    println!("    Grado medio: {:.1}  Energia: {:.4}",
        engine.word_topology.average_degree(),
        engine.word_topology.field_energy());

    let active = engine.word_topology.most_active(15);
    if active.is_empty() {
        println!("    [nessuna parola attiva nel campo]");
    } else {
        println!("    --- Parole attive ---");
        for v in &active {
            let neighbors = engine.word_topology.active_neighbors(&v.word);
            let neighbor_str = if neighbors.is_empty() {
                String::new()
            } else {
                let names: Vec<&str> = neighbors.iter().take(3).map(|(w, _)| *w).collect();
                format!("  → {}", names.join(", "))
            };
            println!("    {:<16} {:.3}{}", v.word, v.activation, neighbor_str);
        }
    }
    println!();
}

fn print_active(engine: &PrometeoTopologyEngine) {
    let active = engine.active_fractals();
    println!();
    if active.is_empty() {
        println!("    [nessun frattale attivo]");
    } else {
        println!("    === FRATTALI ATTIVI ===");
        for (name, score) in &active {
            let bar_len = (*score * 20.0) as usize;
            let bar: String = "#".repeat(bar_len);
            println!("    {:15} [{:<20}] {:.2}", name, bar, score);
        }
    }
    println!();
}

fn print_dream(engine: &mut PrometeoTopologyEngine) {
    println!();
    println!("    [entrando nel sogno...]");
    for i in 0..20 {
        let prev_phase = engine.dream.phase;
        let result = engine.autonomous_tick();
        if engine.dream.phase != prev_phase {
            println!("    tick {}: fase -> {:?}", i, engine.dream.phase);
        }
        if result.dream.dissolved_count > 0 {
            println!("    tick {}: dissolti {} simplessi fragili", i, result.dream.dissolved_count);
        }
        if !result.dream.new_connections.is_empty() {
            println!("    tick {}: scoperte {} connessioni (REM)", i, result.dream.new_connections.len());
        }
        if let Some(ref text) = result.spontaneous {
            println!("    tick {}: [spontaneo] {}", i, text.text);
        }
        if let Some(ref q) = result.question {
            println!("    tick {}: [curiosita] {}", i, q.text);
        }
    }
    println!("    [risveglio — fase: {:?}]", engine.dream.phase);
    println!();
}

fn print_analysis(engine: &mut PrometeoTopologyEngine, text: &str) {
    let phrase = compose_phrase(&mut engine.lexicon, text, &engine.registry);
    let analysis = analyze_composition(&phrase);

    println!();
    println!("    === ANALISI: \"{}\" ===", text);
    println!("    Parole note:     {}", analysis.known_words);
    println!("    Parole ignote:   {}", analysis.unknown_words);
    println!("    Forza:           {:.3}", analysis.strength);
    println!("    Frattali:");
    for (fid, score) in &analysis.dominant_fractals {
        let name = engine.registry.get(*fid)
            .map(|f| f.name.as_str())
            .unwrap_or("?");
        println!("      {:15} {:.3}", name, score);
    }
    println!("    Dimensioni salienti:");
    for (dim, deviation) in analysis.salient_dimensions.iter().take(4) {
        println!("      {:15} {:.3}", dim.name(), deviation);
    }
    println!();
}

fn print_vital(engine: &mut PrometeoTopologyEngine) {
    let state = engine.vital_state();
    println!();
    println!("    === STATO VITALE ===");
    println!("    Attivazione:   {:.2}  {}", state.activation, bar(state.activation));
    println!("    Saturazione:   {:.2}  {}", state.saturation, bar(state.saturation));
    println!("    Curiosita:     {:.2}  {}", state.curiosity, bar(state.curiosity));
    println!("    Fatica:        {:.2}  {}", state.fatigue, bar(state.fatigue));
    println!("    Tensione:      {:?}", state.tension);
    println!();
}

fn print_questions(engine: &mut PrometeoTopologyEngine) {
    let questions = engine.ask();
    println!();
    if questions.is_empty() {
        println!("    [nessuna domanda — il campo e completo]");
    } else {
        println!("    === DOMANDE (cosa non so) ===");
        for (i, q) in questions.iter().take(5).enumerate() {
            println!("    {}. [{:.2}] {}", i + 1, q.urgency, q.text);
        }
    }
    println!();
}

fn print_homology(engine: &PrometeoTopologyEngine) {
    let result = prometeo::topology::homology::compute_homology(&engine.complex);
    println!();
    println!("    === OMOLOGIA ===");
    println!("    b0 (componenti):  {}", result.betti_0);
    println!("    b1 (lacune):      {}", result.betti_1);
    println!("    b2 (cavita):      {}", result.betti_2);
    if !result.cycles.is_empty() {
        println!("    Cicli:");
        for c in &result.cycles {
            let names: Vec<String> = c.vertices.iter()
                .map(|&fid| engine.registry.get(fid)
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| format!("#{}", fid)))
                .collect();
            println!("      [{}]", names.join(" - "));
        }
    }
    if !result.dense_regions.is_empty() {
        println!("    Regioni dense:");
        for &(fid, count) in result.dense_regions.iter().take(3) {
            let name = engine.registry.get(fid)
                .map(|f| f.name.as_str())
                .unwrap_or("?");
            println!("      {:15} {} simplessi", name, count);
        }
    }
    if !result.sparse_regions.is_empty() {
        println!("    Regioni sparse:");
        for &(fid, count) in result.sparse_regions.iter().take(3) {
            let name = engine.registry.get(fid)
                .map(|f| f.name.as_str())
                .unwrap_or("?");
            println!("      {:15} {} simplessi", name, count);
        }
    }
    println!();
}

fn print_why(engine: &PrometeoTopologyEngine) {
    let trace = engine.why();
    println!();
    println!("    === PERCHE? (cammino topologico) ===");
    println!("    {}", trace.explanation);
    if !trace.fractal_sequence.is_empty() {
        println!("    Frattali coinvolti:");
        for (name, score) in &trace.fractal_sequence {
            println!("      {:15} {:.2}", name, score);
        }
    }
    if !trace.propagation_bridges.is_empty() {
        println!("    Ponti di propagazione:");
        for bridge in &trace.propagation_bridges {
            println!("      - {}", bridge);
        }
    }
    if !trace.active_path.is_empty() {
        println!("    Cammino ({} nodi):", trace.active_path.len());
        for node in &trace.active_path {
            println!("      dim={} att={:.2} [{}]",
                node.dimension, node.activation,
                node.fractals.join(", "));
        }
    }
    println!();
}

fn print_introspection(engine: &PrometeoTopologyEngine) {
    let intro = engine.introspect();
    println!();
    println!("    === INTROSPEZIONE ===");
    println!("    Frattali:          {}", intro.fractal_count);
    println!("    Simplessi:         {}", intro.simplex_count);
    println!("    Lacune (b1):       {}", intro.conceptual_gaps);
    println!("    Mondi separati:    {}", intro.disconnected_worlds);
    println!("    Energia campo:     {:.3}", intro.field_energy);
    println!("    Dim. emergenti:    {}", intro.emergent_dimensions);
    if let Some((name, count)) = &intro.densest_region {
        println!("    Regione densa:     {} ({} simplessi)", name, count);
    }
    if let Some((name, count)) = &intro.sparsest_region {
        println!("    Regione sparsa:    {} ({} simplessi)", name, count);
    }
    if let Some((name, count)) = &intro.most_experienced {
        println!("    Piu esperito:      {} ({} attivazioni)", name, count);
    }
    if let Some((name, count)) = &intro.least_experienced {
        println!("    Meno esperito:     {} ({} attivazioni)", name, count);
    }
    println!();
}

fn print_reasoning(engine: &PrometeoTopologyEngine, from_name: &str, to_name: &str) {
    let from = engine.find_fractal(from_name);
    let to = engine.find_fractal(to_name);

    match (from, to) {
        (Some(f), Some(t)) => {
            let imp = engine.implication(f, t);
            println!();
            println!("    === RAGIONAMENTO ===");
            println!("    {:?} (forza {:.2})", imp.kind, imp.strength);
            println!("    {}", imp.path.explanation);
            if imp.path.steps.len() > 2 {
                let intermediates: Vec<&str> = imp.path.steps[1..imp.path.steps.len()-1]
                    .iter().map(|s| s.fractal_name.as_str()).collect();
                println!("    Via: {}", intermediates.join(" -> "));
            }
            println!();
        }
        (None, _) => println!("    [frattale '{}' non trovato]", from_name),
        (_, None) => println!("    [frattale '{}' non trovato]", to_name),
    }
}

fn print_abductions(engine: &PrometeoTopologyEngine) {
    let abductions = engine.abduce();
    println!();
    if abductions.is_empty() {
        println!("    [campo silente — nessuna ipotesi]");
    } else {
        println!("    === ABDUZIONE (cosa spiegherebbe lo stato?) ===");
        for (i, abd) in abductions.iter().enumerate() {
            println!("    {}. {} (potere {:.2}, raggiunge {}, costo medio {:.2})",
                i + 1, abd.hypothesis_name, abd.explanatory_power, abd.reach, abd.mean_cost);
        }
    }
    println!();
}

fn print_growth(engine: &mut PrometeoTopologyEngine) {
    let events = engine.grow();
    println!();
    if events.is_empty() {
        println!("    [nessuna crescita — servono piu osservazioni]");
        println!("    Candidati in attesa: {}", engine.growth.pending_candidates());
        println!("    Frattali creati:     {}", engine.growth.created_fractal_count());
    } else {
        println!("    === CRESCITA ===");
        for event in &events {
            match event {
                prometeo::topology::growth::GrowthEvent::NewFractal { name, observation_count, .. } => {
                    println!("    + Nuovo frattale: {} ({} osservazioni)", name, observation_count);
                }
                prometeo::topology::growth::GrowthEvent::NewConnection { fractal_a, fractal_b } => {
                    let name_a = engine.registry.get(*fractal_a)
                        .map(|f| f.name.as_str()).unwrap_or("?");
                    let name_b = engine.registry.get(*fractal_b)
                        .map(|f| f.name.as_str()).unwrap_or("?");
                    println!("    + Nuova connessione: {} <-> {}", name_a, name_b);
                }
                prometeo::topology::growth::GrowthEvent::NewSubfractal { name, .. } => {
                    println!("    + Nuovo sotto-frattale: {}", name);
                }
            }
        }
        let report = engine.report();
        println!("    [totale: {} frattali, {} simplessi]", report.fractal_count, report.simplex_count);
    }
    println!();
}

fn print_dialogue(engine: &PrometeoTopologyEngine) {
    let state = engine.dialogue_state();
    println!();
    println!("    === DIALOGO ===");
    println!("    Turni:             {}", state.turn_count);
    if let Some(theme) = &state.dominant_theme {
        println!("    Tema dominante:    {}", theme);
    }
    println!("    Coerenza:          {:.2}  {}", state.thematic_coherence, bar(state.thematic_coherence));
    println!("    Novita:            {:.2}  {}", state.novelty, bar(state.novelty));
    let traj_label = if state.trajectory > 0.05 { "convergendo" }
        else if state.trajectory < -0.05 { "divergendo" }
        else { "stabile" };
    println!("    Traiettoria:       {:.2} ({})", state.trajectory, traj_label);
    if !state.salient_dimensions.is_empty() {
        println!("    Postura dimensionale:");
        for (dim, dev) in state.salient_dimensions.iter().take(4) {
            println!("      {:15} {:.3}", dim.name(), dev);
        }
    }
    // Mostra ultimi turni
    let turns = engine.conversation.turns();
    if !turns.is_empty() {
        println!("    Ultimi turni:");
        for turn in turns.iter().rev().take(5).collect::<Vec<_>>().into_iter().rev() {
            let preview = if turn.input.len() > 40 {
                format!("{}...", &turn.input[..40])
            } else {
                turn.input.clone()
            };
            println!("      [{}] \"{}\"", turn.turn_number, preview);
        }
    }
    println!();
}

fn print_navigation(engine: &PrometeoTopologyEngine, from_name: &str, to_name: &str) {
    let from = engine.find_fractal(from_name);
    let to = engine.find_fractal(to_name);

    match (from, to) {
        (Some(f), Some(t)) => {
            match engine.navigate(f, t) {
                Some(path) => {
                    println!();
                    println!("    === GEODETICA ===");
                    println!("    {}", path.explanation);
                    println!("    Costo totale: {:.3}", path.total_cost);
                    println!("    Profondita:   {}", path.max_depth);
                    println!("    Cammino:");
                    for (i, step) in path.steps.iter().enumerate() {
                        let via = if step.shared_structures.is_empty() {
                            String::new()
                        } else {
                            format!("  [via: {}]", step.shared_structures.join(", "))
                        };
                        if i == 0 {
                            println!("      {} (partenza)", step.fractal_name);
                        } else {
                            println!("      -> {} (costo: {:.3}){}", step.fractal_name, step.cumulative_cost, via);
                        }
                    }
                    println!();
                }
                None => println!("    [nessun cammino tra {} e {}]", from_name, to_name),
            }
        }
        (None, _) => println!("    [frattale '{}' non trovato]", from_name),
        (_, None) => println!("    [frattale '{}' non trovato]", to_name),
    }
}

fn print_analogy(engine: &PrometeoTopologyEngine, a_name: &str, b_name: &str, c_name: &str) {
    let a = engine.find_fractal(a_name);
    let b = engine.find_fractal(b_name);
    let c = engine.find_fractal(c_name);

    match (a, b, c) {
        (Some(a), Some(b), Some(c)) => {
            match engine.analogy(a, b, c) {
                Some(analogy) => {
                    println!();
                    println!("    === ANALOGIA ===");
                    println!("    {}", analogy.explanation);
                    println!("    Similitudine: {:.2}", analogy.structural_similarity);
                    if !analogy.shared_bridge_types.is_empty() {
                        println!("    Ponti comuni: {}", analogy.shared_bridge_types.join(", "));
                    }
                    println!();
                }
                None => println!("    [nessuna analogia trovata]"),
            }
        }
        (None, _, _) => println!("    [frattale '{}' non trovato]", a_name),
        (_, None, _) => println!("    [frattale '{}' non trovato]", b_name),
        (_, _, None) => println!("    [frattale '{}' non trovato]", c_name),
    }
}

fn bar(value: f64) -> String {
    let len = (value * 20.0) as usize;
    format!("[{:<20}]", "#".repeat(len))
}

fn print_creative_session(engine: &mut PrometeoTopologyEngine, seed_name: &str) {
    let seed = engine.find_fractal(seed_name);
    match seed {
        Some(sid) => {
            let session = engine.create_from(sid);
            println!();
            println!("    === SESSIONE CREATIVA ===");
            println!("    Seme: {}", session.seed_name);
            println!("    {}", session.explanation);

            if !session.insights.is_empty() {
                println!();
                println!("    Ispirazioni:");
                for (i, ins) in session.insights.iter().enumerate() {
                    println!("    {}. {} (novita {:.2}, distanza {:.1})",
                        i + 1, ins.discovery_name, ins.novelty, ins.distance);
                    println!("       {}", ins.explanation);
                }
            }

            if !session.metaphors.is_empty() {
                println!();
                println!("    Metafore:");
                for m in &session.metaphors {
                    println!("    - {} (tensione {:.2})", m.expression, m.tension);
                }
            }

            if session.connections_made_permanent > 0 {
                println!();
                println!("    [+{} connessioni rese permanenti]", session.connections_made_permanent);
            }
            println!();
        }
        None => println!("    [frattale '{}' non trovato]", seed_name),
    }
}

fn print_metaphors(engine: &PrometeoTopologyEngine, source_name: &str) {
    let source = engine.find_fractal(source_name);
    match source {
        Some(sid) => {
            let metaphors = engine.metaphor(sid);
            println!();
            if metaphors.is_empty() {
                println!("    [nessuna metafora trovata per {}]", source_name);
            } else {
                println!("    === METAFORE ===");
                for (i, m) in metaphors.iter().enumerate() {
                    let dim_names: Vec<&str> = m.shared_structure.iter().map(|d| d.name()).collect();
                    println!("    {}. {} (tensione {:.2})", i + 1, m.expression, m.tension);
                    println!("       Struttura condivisa: {}", dim_names.join(", "));
                }
            }
            println!();
        }
        None => println!("    [frattale '{}' non trovato]", source_name),
    }
}


fn print_where(engine: &PrometeoTopologyEngine) {
    println!();
    match engine.where_am_i() {
        Some((name, horizon)) => {
            println!("    === LOCUS ===");
            println!("    Posizione:   {}", name);
            println!("    Orizzonte:   {:.1}", horizon);
            println!("    Trail:       {} posizioni", engine.locus.trail.len());
            if !engine.locus.trail.is_empty() {
                let trail_names: Vec<String> = engine.locus.trail.iter()
                    .rev()
                    .take(5)
                    .filter_map(|&fid| engine.registry.get(fid).map(|f| f.name.clone()))
                    .collect();
                println!("    Percorso:    {}", trail_names.join(" <- "));
            }
        }
        None => {
            println!("    [non ancora posizionato — scrivi qualcosa per collocarmi]");
        }
    }
    println!();
}

fn print_see(engine: &PrometeoTopologyEngine) {
    println!();
    let visible = engine.what_i_see();
    if visible.is_empty() {
        println!("    [non vedo nulla — non sono ancora posizionato]");
    } else {
        println!("    === CAMPO VISIBILE ===");
        for (name, vis) in &visible {
            let bar_len = (*vis * 20.0) as usize;
            let bar_str: String = "#".repeat(bar_len);
            println!("    {:15} [{:<20}] {:.2}", name, bar_str, vis);
        }
    }
    println!();
}

fn print_inside(engine: &PrometeoTopologyEngine) {
    println!();
    match engine.where_inside() {
        Some(view) => {
            println!("    === SUB-LOCUS (dentro {}) ===", view.fractal_name);
            println!("    Gradi di liberta: {}", view.degrees_of_freedom);
            println!("    Coordinate:");
            for (dim, val) in &view.coordinates {
                let (low, high) = dim.poles();
                let bar_len = (*val * 20.0) as usize;
                let bar_str: String = "#".repeat(bar_len);
                println!("      {:12} [{:<20}] {:.2}  ({} .. {})",
                    dim.name(), bar_str, val, low, high);
            }
        }
        None => {
            println!("    [non ancora posizionato — scrivi qualcosa per collocarmi]");
        }
    }
    println!();
}

fn print_projection(engine: &PrometeoTopologyEngine) {
    println!();
    match engine.holographic_projection() {
        Some(proj) => {
            println!("    === PROIEZIONE OLOGRAFICA (da {}) ===", proj.from_name);
            for fp in proj.projections.iter().take(10) {
                println!("    {:15} prox={:.2}  ris={:.2}  dist={:.3}",
                    fp.name, fp.proximity, fp.dimensional_resonance, fp.distortion);
            }
        }
        None => {
            println!("    [non ancora posizionato]");
        }
    }
    println!();
}

fn print_single_projection(engine: &PrometeoTopologyEngine, target_name: &str) {
    let target = engine.find_fractal(target_name);
    match target {
        Some(tid) => {
            match engine.project_fractal(tid) {
                Some(fp) => {
                    println!();
                    println!("    === {} VISTO DA QUI ===", fp.name);
                    println!("    Prossimita:   {:.3}", fp.proximity);
                    println!("    Risonanza:    {:.3}", fp.dimensional_resonance);
                    println!("    Distorsione:  {:.3}", fp.distortion);
                    println!("    Centro apparente:");
                    for dim in prometeo::topology::primitive::Dim::ALL.iter() {
                        let val = fp.apparent_center.get(*dim);
                        let bar_len = (val * 20.0) as usize;
                        let bar_str: String = "#".repeat(bar_len);
                        println!("      {:12} [{:<20}] {:.2}", dim.name(), bar_str, val);
                    }
                    println!();
                }
                None => println!("    [non posso proiettare — non sono posizionato]"),
            }
        }
        None => println!("    [frattale '{}' non trovato]", target_name),
    }
}

fn print_confidence(engine: &PrometeoTopologyEngine) {
    let conf = engine.confidence();
    println!();
    println!("    === CONFIDENZA ===");
    println!("    Compreso:     {}", if conf.understood { "si" } else { "no" });
    println!("    Lacune:       {}", if conf.has_gaps { "si" } else { "no" });
    println!("    Frattali attivi: {}", conf.active_count);
    println!("    {}", conf.explanation);
    println!();
}

fn print_teach(engine: &mut PrometeoTopologyEngine, text: &str) {
    let result = engine.teach(text);
    println!();
    println!("    === INSEGNAMENTO ===");
    println!("    Parole processate: {}", result.words_processed.len());
    println!("    Gia note:         {}", result.known_count);
    println!("    Nuove:            {}", result.new_count);
    if !result.words_processed.is_empty() {
        for word in &result.words_processed {
            let status = if let Some(pat) = engine.lexicon.get(word) {
                let dom = pat.dominant_fractal()
                    .and_then(|(fid, aff)| {
                        engine.registry.get(fid)
                            .map(|f| format!("→ {} ({:.2})", f.name, aff))
                    })
                    .unwrap_or_else(|| "—".to_string());
                format!("stab={:.2} esp={} {}", pat.stability, pat.exposure_count, dom)
            } else {
                "?".to_string()
            };
            println!("      {:15} {}", word, status);
        }
    }
    if !result.fractal_affinities.is_empty() {
        println!("    Frattali coinvolti:");
        for (fid, score) in &result.fractal_affinities {
            let name = engine.registry.get(*fid)
                .map(|f| f.name.as_str())
                .unwrap_or("?");
            println!("      {:15} {:.3}", name, score);
        }
    }
    let report = engine.report();
    println!("    [vocabolario: {} parole]", report.vocabulary_size);
    println!();
}

fn print_will(engine: &PrometeoTopologyEngine) {
    println!();
    match engine.current_will() {
        Some(will) => {
            println!("    === VOLONTA ===");
            println!("    Intenzione:  {:?}", will.intention);
            println!("    Forza:       {:.2}  {}", will.drive, bar(will.drive));
            if !will.undercurrents.is_empty() {
                println!("    Correnti sotterranee:");
                for (intent, pressure) in &will.undercurrents {
                    let label = match intent {
                        prometeo::topology::will::Intention::Express { .. } => "Esprimere",
                        prometeo::topology::will::Intention::Explore { .. } => "Esplorare",
                        prometeo::topology::will::Intention::Question { .. } => "Domandare",
                        prometeo::topology::will::Intention::Remember { .. } => "Ricordare",
                        prometeo::topology::will::Intention::Withdraw { .. } => "Ritirarsi",
                        prometeo::topology::will::Intention::Reflect => "Riflettere",
                        prometeo::topology::will::Intention::Dream { .. } => "Sognare",
                        prometeo::topology::will::Intention::Instruct { .. } => "Istruire",
                    };
                    println!("      {:15} {:.2}", label, pressure);
                }
            }
        }
        None => {
            println!("    [nessuna volonta — il sistema non ha ancora ricevuto input]");
        }
    }
    println!();
}

fn print_compounds(engine: &PrometeoTopologyEngine) {
    println!();
    let compounds = engine.compound_states();
    if compounds.is_empty() {
        println!("    [nessun composto attivo — i frattali non co-attivano abbastanza]");
    } else {
        println!("    === COMPOSTI FRATTALI ATTIVI ===");
        for c in compounds {
            let fids: String = c.fractals.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let order_label = if c.order == 3 { " [ternario]" } else { "" };
            println!("    {:15} ({})  forza: {:.3}{}",
                c.name, fids, c.strength, order_label);
        }
    }
    println!();
}

fn print_bridges(engine: &PrometeoTopologyEngine) {
    println!();

    // 1. Ponti semantici: parole da frattali diversi vicine nello spazio 8D
    let bridges = engine.discover_bridges();
    if bridges.is_empty() {
        println!("    [nessun ponte semantico — insegna piu lezioni per far emergere connessioni]");
    } else {
        println!("    === PONTI SEMANTICI (connessioni inter-dominio) ===");
        println!("    Parole da frattali diversi ma vicine nello spazio 8D:");
        println!();
        for (i, b) in bridges.iter().enumerate().take(20) {
            let shared_names: String = b.shared_dims.iter()
                .take(3)
                .map(|(d, _, _)| format!("{:?}", d))
                .collect::<Vec<_>>()
                .join(", ");
            println!("    {:2}. {:12} ({}) <-> {:12} ({})  dist: {:.3}  [{}]",
                i + 1, b.word_a, b.fractal_a, b.word_b, b.fractal_b,
                b.distance, shared_names);
        }
        if bridges.len() > 20 {
            println!("    ... e altri {} ponti", bridges.len() - 20);
        }
    }

    println!();

    // 2. Affinita latenti: parole vicine a frattali non assegnati
    let latent = engine.discover_latent_affinities();
    if latent.is_empty() {
        println!("    [nessuna affinita latente — il lessico e ancora troppo giovane]");
    } else {
        println!("    === AFFINITA LATENTI (connessioni potenziali) ===");
        println!("    Parole topologicamente vicine a frattali non mappati:");
        println!();
        for (i, la) in latent.iter().enumerate().take(15) {
            println!("    {:2}. {:12} ({})  -->  {:12}  affinita: {:.2} (registrata: {:.2})",
                i + 1, la.word, la.current_fractal, la.latent_fractal,
                la.topological_affinity, la.registered_affinity);
        }
        if latent.len() > 15 {
            println!("    ... e altre {} affinita", latent.len() - 15);
        }
    }

    println!();
}

fn print_reinforce(engine: &mut PrometeoTopologyEngine) {
    println!();
    let result = engine.reinforce_bridges();

    if result.bridges_found == 0 && result.latent_found == 0 {
        println!("    [nessuna connessione da rinforzare — insegna piu lezioni]");
    } else {
        println!("    === RINFORZO CONNESSIONI ===");
        println!();
        if result.bridges_found > 0 {
            println!("    Ponti semantici:     {} trovati, {} rinforzati",
                result.bridges_found, result.bridges_reinforced);
            println!("    Simplessi creati:    {}", result.simplices_created);
        }
        if result.latent_found > 0 {
            println!("    Affinita latenti:    {} trovate, {} incrementate",
                result.latent_found, result.affinities_reinforced);
        }
        println!();
        println!("    Le connessioni sono ora iscritte nel complesso.");
        println!("    Usa :ponti per vedere i ponti aggiornati.");
    }
    println!();
}

fn run_lesson(engine: &mut PrometeoTopologyEngine, path: &str) {
    // Cerca il file nella directory corrente e in lessons/
    let file_path = if std::path::Path::new(path).exists() {
        PathBuf::from(path)
    } else {
        let lessons_path = PathBuf::from("lessons").join(path);
        if lessons_path.exists() {
            lessons_path
        } else {
            // Prova con estensione .txt
            let with_ext = PathBuf::from("lessons").join(format!("{}.txt", path));
            if with_ext.exists() {
                with_ext
            } else {
                println!();
                println!("    [file non trovato: {}]", path);
                println!("    [lezioni disponibili:]");
                if let Ok(entries) = fs::read_dir("lessons") {
                    let mut files: Vec<String> = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map_or(false, |ext| ext == "txt"))
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect();
                    files.sort();
                    for f in &files {
                        println!("      {}", f);
                    }
                }
                println!();
                return;
            }
        }
    };

    println!();
    println!("    === LEZIONE: {} ===", file_path.file_name().unwrap().to_string_lossy());

    match engine.teach_lesson_file(&file_path) {
        Ok(result) => {
            let report = engine.report();
            println!("    Parole processate: {}", result.words_processed.len());
            println!("    Parole nuove:      {}", result.new_count);
            println!("    Parole gia note:   {}", result.known_count);
            println!("    Vocabolario ora:   {} parole", report.vocabulary_size);
            if !engine.semantic_axes().is_empty() {
                println!("    Assi semantici:    {}", engine.semantic_axes().len());
            }
            println!();
        }
        Err(e) => {
            println!("    [{}]", e);
            println!();
        }
    }
}

/// Insegna un file in formato compatto (parola: ancore / negativi).
fn run_compact_lesson(engine: &mut PrometeoTopologyEngine, path: &str) {
    let file_path = if std::path::Path::new(path).exists() {
        PathBuf::from(path)
    } else {
        let lessons_path = PathBuf::from("lessons").join(path);
        if lessons_path.exists() {
            lessons_path
        } else {
            let with_ext = PathBuf::from("lessons").join(format!("{}.txt", path));
            if with_ext.exists() {
                with_ext
            } else {
                println!();
                println!("    [file non trovato: {}]", path);
                println!();
                return;
            }
        }
    };

    println!();
    println!("    === LEZIONE COMPATTA: {} ===", file_path.file_name().unwrap().to_string_lossy());

    match engine.teach_compact_file(&file_path) {
        Ok((result, sentences)) => {
            let report = engine.report();
            println!("    Parole processate: {}", result.words_processed.len());
            println!("    Parole nuove:      {}", result.new_count);
            println!("    Parole gia note:   {}", result.known_count);
            println!("    Vocabolario ora:   {} parole", report.vocabulary_size);
            println!("    Frasi generate:    {}", sentences.len());
            // Mostra prime 8 frasi come esempio
            if !sentences.is_empty() {
                println!("    --- esempio frasi generate ---");
                for (i, s) in sentences.iter().take(8).enumerate() {
                    println!("    {:2}. {}", i + 1, s);
                }
                if sentences.len() > 8 {
                    println!("    ... (+{} altre)", sentences.len() - 8);
                }
            }
            println!();
        }
        Err(e) => {
            println!("    [{}]", e);
            println!();
        }
    }
}

/// Auto-insegnamento: carica tutte le lezioni dalla cartella lessons/ in ordine.
/// Viene chiamato solo alla prima sessione (nessun file di stato trovato).
fn auto_teach_lessons(engine: &mut PrometeoTopologyEngine) {
    let lessons_dir = PathBuf::from("lessons");
    if !lessons_dir.exists() {
        println!("  [cartella lessons/ non trovata — usa :lesson per insegnare]");
        return;
    }

    let mut files: Vec<PathBuf> = match fs::read_dir(&lessons_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "txt" || ext == "lesson"))
            .collect(),
        Err(_) => {
            println!("  [impossibile leggere lessons/]");
            return;
        }
    };
    files.sort();

    if files.is_empty() {
        println!("  [nessuna lezione trovata in lessons/]");
        return;
    }

    println!("  [insegnamento automatico: {} lezioni]", files.len());
    println!();

    let mut total_new = 0usize;
    let mut total_lessons = 0usize;

    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy();
        match engine.teach_lesson_file(file) {
            Ok(result) => {
                total_new += result.new_count;
                total_lessons += 1;
                println!("    {:30} +{} parole", name, result.new_count);
            }
            Err(e) => {
                println!("    {:30} [errore: {}]", name, e);
            }
        }
    }

    println!();
    println!("  [completate {} lezioni, {} parole nuove apprese]", total_lessons, total_new);
}

fn print_curriculum(engine: &PrometeoTopologyEngine) {
    let curr = engine.curriculum();
    println!();
    if curr.lessons_completed.is_empty() {
        println!("    [nessuna lezione completata]");
    } else {
        println!("    === CURRICULUM ===");
        for (i, lesson) in curr.lessons_completed.iter().enumerate() {
            println!("    {}. {} — {} parole ({})",
                i + 1, lesson.name, lesson.words_taught.len(), lesson.timestamp);
        }
        println!("    Totale parole apprese: {}", curr.total_words_learned);
    }
    println!();
}

fn print_axes(engine: &PrometeoTopologyEngine) {
    let axes = engine.semantic_axes();
    println!();
    if axes.is_empty() {
        println!("    [nessun asse semantico rilevato]");
        println!("    [insegna piu parole con :lesson per rilevare coppie di contrasto]");
    } else {
        println!("    === ASSI SEMANTICI ({}) ===", axes.len());
        for (i, axis) in axes.iter().enumerate() {
            println!("    {}. {} ↔ {}  (forza: {:.3})",
                i + 1, axis.word_a, axis.word_b, axis.strength);
        }
    }
    println!();
}

fn print_word_on_axes(engine: &PrometeoTopologyEngine, word: &str) {
    println!();
    let positions = engine.word_on_axes(word);
    if positions.is_empty() {
        if engine.semantic_axes().is_empty() {
            println!("    [nessun asse semantico disponibile]");
        } else {
            println!("    [parola '{}' non trovata nel lessico]", word);
        }
    } else {
        println!("    === '{}' SUGLI ASSI ===", word);
        for (axis_name, pos) in &positions {
            // Barra visuale da -1 a +1
            let bar_pos = ((pos + 1.0) / 2.0 * 20.0) as usize;
            let bar: String = (0..21).map(|i| if i == bar_pos { '●' } else { '─' }).collect();
            println!("    {} [{:.2}]  {}", bar, pos, axis_name);
        }
    }
    println!();
}

fn reteach_all(engine: &mut PrometeoTopologyEngine) {
    println!();
    println!("    === RE-TEACHING (rinforzo co-occorrenze) ===");

    // Re-insegna lezioni dalla cartella lessons/
    let lessons_dir = PathBuf::from("lessons");
    if lessons_dir.exists() {
        print!("    Lezioni (lessons/)... ");
        io::stdout().flush().unwrap();
        match engine.reteach_all_in_dir(&lessons_dir) {
            Ok((files, known)) => println!("{} file, {} parole processate", files, known),
            Err(e) => println!("errore: {}", e),
        }
    }

    // Re-insegna candidati_batch nella directory corrente
    let cwd = PathBuf::from(".");
    let mut batch_files: Vec<PathBuf> = match fs::read_dir(&cwd) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map_or(false, |n| n.starts_with("candidati_batch_") && n.ends_with(".txt"))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    batch_files.sort();

    if !batch_files.is_empty() {
        println!("    Batch ({} file)...", batch_files.len());
        let mut total_known = 0usize;
        let mut total_files = 0usize;
        for (i, file) in batch_files.iter().enumerate() {
            match engine.reteach_lesson_file(file) {
                Ok(result) => {
                    total_known += result.known_count;
                    total_files += 1;
                    if (i + 1) % 50 == 0 {
                        println!("      ...{}/{} file processati", i + 1, batch_files.len());
                    }
                }
                Err(e) => {
                    println!("      [errore {}: {}]", file.display(), e);
                }
            }
        }
        // Ricalibra dopo tutti i batch — applica prima le firme curate
        engine.lexicon.apply_curated_signatures();
        engine.lexicon.load_phenomenology_signatures();
        engine.recompute_all_word_affinities();
        engine.update_semantic_axes();
        engine.recalibrate_emergent_dimensions();
        println!("    Batch completati: {} file, {} parole processate", total_files, total_known);
    }

    let report = engine.report();
    println!();
    println!("    Vocabolario: {} parole", report.vocabulary_size);
    println!("    Archi campo: {}", report.word_field_edges);
    println!("    Energia:     {:.4}", report.word_field_energy);
    println!();
}

/// Insegna tutte le lezioni pendenti (non ancora nel curriculum) da lessons/.
/// Include lessons/contrastive/ e lessons/translation/ automaticamente.
fn learn_all_pending(engine: &mut PrometeoTopologyEngine) {
    let lessons_dir = PathBuf::from("lessons");
    if !lessons_dir.exists() {
        println!();
        println!("    [cartella lessons/ non trovata]");
        println!();
        return;
    }

    let curriculum_size_before = engine.curriculum().lessons_completed.len();
    let vocab_before = engine.report().vocabulary_size;

    println!();
    println!("    === INSEGNAMENTO LEZIONI PENDENTI ===");
    println!("    (le lezioni gia nel curriculum vengono saltate)");
    println!();

    let mut progress_lines: Vec<(String, usize)> = Vec::new();

    match engine.teach_all_pending(&lessons_dir, &mut |name, new_w, _total| {
        progress_lines.push((name.to_string(), new_w));
    }) {
        Ok((taught, new_words, skipped)) => {
            // Mostra le lezioni insegnate
            for (name, new_w) in &progress_lines {
                println!("    + {:40} +{} parole", name, new_w);
            }
            if !progress_lines.is_empty() {
                println!();
            }

            let report = engine.report();
            println!("    Lezioni insegnate:  {}", taught);
            println!("    Lezioni saltate:    {} (gia nel curriculum)", skipped);
            println!("    Parole nuove:       {}", new_words);
            println!("    Vocabolario:        {} → {} parole", vocab_before, report.vocabulary_size);
            println!("    Curriculum:         {} → {} lezioni",
                curriculum_size_before, engine.curriculum().lessons_completed.len());
            if !engine.semantic_axes().is_empty() {
                println!("    Assi semantici:     {}", engine.semantic_axes().len());
            }
            println!();

            if taught == 0 {
                println!("    [nessuna lezione nuova — usa :reteach per forzare il re-learning]");
                println!();
            }
        }
        Err(e) => {
            println!("    [errore: {}]", e);
            println!();
        }
    }
}

fn print_population(engine: &PrometeoTopologyEngine) {
    println!();
    println!("    === DISTRIBUZIONE MULTI-FRATTALE ===");
    println!("    (ogni parola vive in tutti i frattali con intensita diversa)");
    println!();

    let mut ids = engine.registry.all_ids();
    ids.sort();

    // Per ogni frattale, conta quante parole hanno affinita forte (>= 0.7)
    // e quante hanno affinita moderata (>= 0.5)
    let total_words = engine.lexicon.word_count();

    for fid in &ids {
        if let Some(fractal) = engine.registry.get(*fid) {
            let mut strong = 0usize;
            let mut moderate = 0usize;
            let mut total_aff = 0.0f64;

            for (_word, pattern) in engine.lexicon.patterns_iter() {
                let aff = fractal.affinity(&pattern.signature);
                if aff >= 0.7 {
                    strong += 1;
                }
                if aff >= 0.5 {
                    moderate += 1;
                }
                total_aff += aff;
            }

            let avg_aff = if total_words > 0 { total_aff / total_words as f64 } else { 0.0 };
            let bar_len = (strong as f64 / total_words.max(1) as f64 * 40.0) as usize;
            let bar: String = "#".repeat(bar_len);
            println!("    {:15} [{}] forte:{:>4}  mod:{:>4}  media:{:.3}",
                fractal.name, format!("{:<40}", bar), strong, moderate, avg_aff);
        }
    }
    println!();
    println!("    Nota: 'forte' = affinita >= 0.7, 'mod' = affinita >= 0.5");
    println!("    Una parola puo essere forte in piu frattali simultaneamente.");
    println!();
}

fn print_emergent(engine: &PrometeoTopologyEngine) {
    println!();
    println!("    === DIMENSIONI EMERGENTI ===");

    let mut ids = engine.registry.all_ids();
    ids.sort();

    let mut total_calibrated = 0usize;
    let mut total_dims = 0usize;

    for fid in &ids {
        if let Some(fractal) = engine.registry.get(*fid) {
            if fractal.emergent_dimensions.is_empty() {
                continue;
            }
            let calibrated: Vec<_> = fractal.emergent_dimensions.iter()
                .filter(|d| d.is_calibrated())
                .collect();
            total_dims += fractal.emergent_dimensions.len();
            total_calibrated += calibrated.len();

            if calibrated.is_empty() {
                println!("    {:15} ({} dim, nessuna calibrata)", fractal.name, fractal.emergent_dimensions.len());
            } else {
                println!("    {:15} ({}/{} calibrate, {}w)",
                    fractal.name,
                    calibrated.len(),
                    fractal.emergent_dimensions.len(),
                    calibrated[0].calibration_population);
                for d in &calibrated {
                    let var_pct = d.explained_variance * 100.0;
                    println!("      {:20} std={:.3}  var={:.1}%  [{:.2}, {:.2}]",
                        d.name, d.std_dev, var_pct, d.range.0, d.range.1);
                }
            }
        }
    }
    println!();
    println!("    Totale: {}/{} dimensioni calibrate", total_calibrated, total_dims);
    println!();
}

fn print_thoughts(engine: &PrometeoTopologyEngine) {
    use prometeo::topology::thought::{generate_thoughts, ThoughtKind};

    println!();
    println!("    === PENSIERI TOPOLOGICI ===");
    println!("    (osservazioni strutturali grezze — non dialogo)");
    println!();

    let thoughts = generate_thoughts(engine);

    if thoughts.is_empty() {
        println!("    [nessun pensiero emergente — campo troppo quieto]");
        println!();
        return;
    }

    // Raggruppa per tipo
    let tensions:     Vec<_> = thoughts.iter().filter(|t| t.kind == ThoughtKind::Tension).collect();
    let gaps:         Vec<_> = thoughts.iter().filter(|t| t.kind == ThoughtKind::Gap).collect();
    let bridges:      Vec<_> = thoughts.iter().filter(|t| t.kind == ThoughtKind::MissingBridge).collect();
    let disconnects:  Vec<_> = thoughts.iter().filter(|t| t.kind == ThoughtKind::Disconnection).collect();
    let hypotheses:   Vec<_> = thoughts.iter().filter(|t| t.kind == ThoughtKind::Hypothesis).collect();

    if !tensions.is_empty() {
        println!("    TENSIONI ({}):", tensions.len());
        for t in tensions.iter().take(3) {
            let frattali = t.fractal_names.join(" ↔ ");
            let parole = t.words.iter().take(4).cloned().collect::<Vec<_>>().join(", ");
            println!("      [{:.2}] {} | {}", t.strength, frattali, parole);
        }
        println!();
    }

    if !gaps.is_empty() {
        println!("    LACUNE ({}):", gaps.len());
        for t in gaps.iter().take(3) {
            let frattali = t.fractal_names.join(", ");
            println!("      [{:.2}] {} — territorio inesplorato", t.strength, frattali);
        }
        println!();
    }

    if !bridges.is_empty() {
        println!("    PONTI MANCANTI ({}):", bridges.len());
        for t in bridges.iter().take(3) {
            let frattali = t.fractal_names.join(" — ");
            let parole = t.words.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
            println!("      [{:.2}] {} | potenziale: {}", t.strength, frattali, parole);
        }
        println!();
    }

    if !disconnects.is_empty() {
        println!("    DISCONNESSIONI ({}):", disconnects.len());
        for t in disconnects.iter().take(3) {
            let frattali = t.fractal_names.join(", ");
            println!("      [{:.2}] {} — isolato dal cluster principale", t.strength, frattali);
        }
        println!();
    }

    if !hypotheses.is_empty() {
        println!("    IPOTESI ({}):", hypotheses.len());
        for t in hypotheses.iter().take(3) {
            let parole = t.words.iter().take(4).cloned().collect::<Vec<_>>().join(", ");
            println!("      [{:.2}] {} — non ancora cristallizzato in LTM", t.strength, parole);
        }
        println!();
    }

    println!("    Totale: {} pensieri  (tensioni:{} lacune:{} ponti:{} isole:{} ipotesi:{})",
        thoughts.len(), tensions.len(), gaps.len(), bridges.len(), disconnects.len(), hypotheses.len());
    println!();
}

fn print_episodes(engine: &PrometeoTopologyEngine) {
    println!();
    println!("    === MEMORIA EPISODICA ===");

    let episodes = engine.memory.recent_episodes(10);

    if episodes.is_empty() {
        println!("    [nessun episodio memorizzato]");
        println!();
        return;
    }

    println!("    Ultimi {} episodi:", episodes.len());
    println!();

    for (i, ep) in episodes.iter().rev().enumerate() {
        let frattale = match ep.locus_fractal {
            Some(fid) => engine.registry.get(fid).map(|f| f.name.as_str()).unwrap_or("?"),
            None => "neutrale",
        };

        let input_preview = if ep.input_text.len() > 50 {
            format!("{}...", &ep.input_text[..50])
        } else {
            ep.input_text.clone()
        };

        println!("    [{}] turno {} @ tick {} ({})",
            i + 1, ep.turn_number, ep.timestamp, frattale);
        println!("        {} ({}): \"{}\"",
            ep.speaker,
            if ep.emotional_tone > 0.6 { "alto" }
            else if ep.emotional_tone > 0.3 { "medio" }
            else { "basso" },
            input_preview);
        println!();
    }

    let total = engine.memory.episodic_memory.len();
    let user_count = engine.memory.count_by_speaker("utente");
    let system_count = total - user_count;

    println!("    Statistiche: {} episodi totali ({} utente, {} sistema)",
        total, user_count, system_count);
    println!();
}

fn print_vision(engine: &PrometeoTopologyEngine) {
    println!();
    println!("    === VISIONE (parole attive nel campo) ===");
    println!();

    let vision = engine.perceive_vision(10);

    if vision.is_empty() {
        println!("    [campo spento — nessuna attivazione]");
        println!();
        return;
    }

    for (i, (word, activation)) in vision.iter().enumerate() {
        let intensity = if *activation > 0.7 {
            "████████"
        } else if *activation > 0.4 {
            "█████   "
        } else if *activation > 0.2 {
            "███     "
        } else {
            "█       "
        };
        println!("    {} {:12} {} ({:.3})", i + 1, word, intensity, activation);
    }
    println!();
}

fn print_echo(engine: &PrometeoTopologyEngine) {
    println!();
    println!("    === ECO (risonanze dalla memoria) ===");
    println!();

    let echo = engine.perceive_echo(8);

    if echo.is_empty() {
        println!("    [silenzio — nessuna risonanza dalla memoria]");
        println!();
        return;
    }

    for (i, (word, resonance)) in echo.iter().enumerate() {
        let echo_level = if *resonance > 0.6 {
            "forte"
        } else if *resonance > 0.3 {
            "medio"
        } else {
            "debole"
        };
        println!("    {} {:12} [{}] ({:.3})", i + 1, word, echo_level, resonance);
    }
    println!();
}

fn print_perceptual_field(engine: &PrometeoTopologyEngine) {
    println!();
    println!("    === CAMPO PERCETTIVO ===");
    println!();

    let field = engine.perceptual_field();

    // Posizione
    println!("    Posizione: {}", field.position);
    if let Some(sublocus) = &field.locus_sublocus {
        println!("        Sub-locus ({} gradi di liberta):", sublocus.degrees_of_freedom);
        for (dim, value) in &sublocus.coordinates {
            println!("            {} = {:.3}", dim.name(), value);
        }
    }
    println!();

    // Visione
    println!("    Visione ({} parole attive):", field.vision.len());
    for (word, act) in field.vision.iter().take(5) {
        println!("        {} ({:.3})", word, act);
    }
    println!();

    // Eco
    println!("    Eco ({} risonanze):", field.echo.len());
    for (word, res) in field.echo.iter().take(5) {
        println!("        {} ({:.3})", word, res);
    }
    println!();
}

fn print_word_emergent(engine: &PrometeoTopologyEngine, word: &str) {
    println!();
    match engine.word_emergent_position(word) {
        Some((fractal_name, projections)) => {
            if projections.is_empty() {
                println!("    [frattale {} non ha dimensioni emergenti calibrate]", fractal_name);
            } else {
                println!("    === '{}' — POSIZIONE EMERGENTE (in {}) ===", word, fractal_name);
                for (dim_name, val) in &projections {
                    // Barra da 0.0 a 1.0
                    let bar_pos = (val.clamp(0.0, 1.0) * 20.0) as usize;
                    let bar_str: String = (0..21).map(|i| if i == bar_pos { '●' } else { '─' }).collect();
                    println!("    {:20} {} {:.3}", dim_name, bar_str, val);
                }
            }
        }
        None => {
            println!("    [parola '{}' non trovata nel lessico]", word);
        }
    }
    println!();
}

fn print_tension(engine: &PrometeoTopologyEngine, pole_a: &str, pole_b: &str) {
    let tensions = engine.tension_words(pole_a, pole_b);

    println!();
    println!("    === TENSIONI: {} ↔ {} ===", pole_a, pole_b);

    if tensions.is_empty() {
        println!("    [nessuna parola di tensione trovata]");
        println!("    (assicurati che entrambi i poli siano nel lessico con stabilita sufficiente)");
        println!();
        return;
    }

    println!("    {:>5}  {:<18}  {:21}  {:>4}  {:>5}", "pos", "parola", "asse", "dist", "forza");
    println!("    {}", "─".repeat(62));

    let pole_a_label = format!("[{}]", pole_a);
    let pole_b_label = format!("[{}]", pole_b);
    let axis_width = 21usize;

    // Mostra i poli come riferimento
    println!("    {:>5.2}  {:<18}  {}{}  {:>4}  {:>5}",
        0.0, pole_a_label,
        "●".to_string() + &"─".repeat(axis_width - 1),
        "", 0.00, 1.00);

    for tw in &tensions {
        // Barra visuale della posizione sull'asse
        let bar: String = {
            let pos_clamped = tw.position.clamp(-0.15, 1.15);
            // Mappa [-0.15, 1.15] → [0, axis_width]
            let bar_idx = ((pos_clamped + 0.15) / 1.30 * axis_width as f64) as usize;
            let bar_idx = bar_idx.min(axis_width);
            (0..=axis_width).map(|i| {
                if i == bar_idx { '●' }
                else if i == 0 { '|' }
                else if i == axis_width { '|' }
                else { '─' }
            }).collect()
        };

        println!("    {:>+5.2}  {:<18}  {}  {:.2}  {:.2}",
            tw.position, tw.word, bar, tw.distance_to_axis, tw.strength);
    }

    println!("    {:>5.2}  {:<18}  {}{}  {:>4}  {:>5}",
        1.0, pole_b_label,
        "─".repeat(axis_width - 1) + "●",
        "", 0.00, 1.00);

    println!();
    println!("    {} tensioni trovate (posizione: 0.0={}, 1.0={})", tensions.len(), pole_a, pole_b);
    println!();
}

fn print_operators(engine: &PrometeoTopologyEngine) {
    println!();
    println!("    === OPERATORI STRUTTURALI (si + no + quanto = X) ===");

    // Raccogli tutte le coppie con dati operatore (neg o aff espliciti)
    // aff = co_affirmed (operatori "come", "simile"...), NON co_occurrences neutrali
    let mut pairs: Vec<(String, String, u64, u64)> = Vec::new(); // (word_a, word_b, aff, neg)

    for (word_a, pattern_a) in engine.lexicon.patterns_iter() {
        for (word_b, neg_count) in &pattern_a.co_negated {
            let aff_ab = pattern_a.co_affirmed.get(word_b).copied().unwrap_or(0);
            let neg_ab = *neg_count;
            // Aggiungi solo la coppia canonicamente ordinata (evita duplicati)
            if word_a.as_str() < word_b.as_str() {
                let neg_ba = engine.lexicon.get(word_b)
                    .and_then(|p| p.co_negated.get(word_a).copied())
                    .unwrap_or(0);
                let aff_ba = engine.lexicon.get(word_b)
                    .and_then(|p| p.co_affirmed.get(word_a).copied())
                    .unwrap_or(0);
                pairs.push((
                    word_a.clone(),
                    word_b.clone(),
                    aff_ab + aff_ba,
                    neg_ab + neg_ba,
                ));
            }
        }
    }

    if pairs.is_empty() {
        println!("    [nessuna co-occorrenza negata registrata]");
        println!("    Consiglio: ri-insegna lezioni contrastive dopo l'aggiornamento.");
        println!();
        return;
    }

    // Ordina per ratio negazione decrescente
    pairs.sort_by(|a, b| {
        let ratio_a = a.3 as f64 / (a.2 + a.3 + 1) as f64;
        let ratio_b = b.3 as f64 / (b.2 + b.3 + 1) as f64;
        ratio_b.partial_cmp(&ratio_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("    Coppie con dati operatore: {}", pairs.len());
    println!();
    println!("    {:<16} {:<16} {:>5} {:>5} {:>6}  →", "parola A", "parola B", "aff", "neg", "ratio");
    println!("    {}", "─".repeat(58));

    for (word_a, word_b, aff, neg) in pairs.iter().take(30) {
        let total = aff + neg;
        let ratio = *neg as f64 / total.max(1) as f64;
        let label = if ratio > 0.66 {
            "opposizione"
        } else if ratio > 0.33 {
            "tensione"
        } else {
            "risonanza"
        };
        println!("    {:<16} {:<16} {:>5} {:>5}  {:.2}   {}", word_a, word_b, aff, neg, ratio, label);
    }
    println!();
}

fn print_knowledge(engine: &PrometeoTopologyEngine) {
    let kb = &engine.knowledge_base;
    println!();
    println!("    === MEMORIA PROCEDURALE ===");
    println!();
    println!("    Le regole emergono dalla topologia — non sono template hardcodati.");
    println!("    :know <fatto> cristallizza il fatto nel campo (teach) e lo registra");
    println!("    per il recall contestuale (boost leggero quando pertinente).");
    println!();
    if kb.entry_count() == 0 {
        println!("    Nessuna conoscenza registrata.");
        println!("    Usa :know <fatto> [dominio] per insegnare.");
        println!("    Domini: social | dialogo | procedurale | epistemica | sintassi | sé");
    } else {
        println!("    Conoscenze registrate: {}", kb.entry_count());
        for entry in kb.user_entries().iter().take(20) {
            println!("      [{}] {}", entry.domain.as_str(), entry.content);
        }
    }
    println!();
}

fn save_state(engine: &PrometeoTopologyEngine) {
    let state = PrometeoState::capture(engine);
    let path = PathBuf::from(BINARY_STATE);
    if let Err(e) = state.save_to_binary(&path) {
        println!("    [errore salvataggio SimplDB: {}]", e);
    }
}

/// Esporta lo stato corrente in formato JSON (per debug/backup/ispezione).
fn export_json(engine: &PrometeoTopologyEngine) {
    let state = PrometeoState::capture(engine);
    let path = PathBuf::from(STATE_FILE);
    match state.save_to_file(&path) {
        Ok(()) => println!("    [stato esportato in JSON: {}]", STATE_FILE),
        Err(e) => println!("    [errore esportazione JSON: {}]", e),
    }
}


// ---------------------------------------------------------------------------
// Phase 31 — Lettura Attiva
// ---------------------------------------------------------------------------

/// Legge un file di testo e lo processa attivamente:
///   - ogni clausola (tra virgole) insegnata con teach() — co-attivazione stretta
///   - ogni frase ricevuta con receive() — attivazione campo completa
///   - mini-sogno ogni 5 paragrafi — consolidamento
///   - domande sulle parole sconosciute mostrate in tempo reale
fn run_read_command(engine: &mut PrometeoTopologyEngine, path: &str) {
    use std::collections::HashSet;
    use prometeo::topology::thought::generate_thoughts;

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => { println!("    [lettura: errore apertura file — {}]", e); return; }
    };

    // Normalizza fine-riga Windows (\r\n → \n) e divide in paragrafi
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let paragraphs: Vec<&str> = normalized
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if paragraphs.is_empty() {
        println!("    [lettura: file vuoto o formato non riconosciuto]");
        return;
    }

    let rep_before = engine.report();
    let simplici_prima = rep_before.simplex_count;
    let vocabolario_prima = rep_before.vocabulary_size;

    let mut tutte_sconosciute: HashSet<String> = HashSet::new();
    let mut domande_aperte: Vec<String> = Vec::new();
    let mut frase_globale: usize = 0;

    println!();
    println!("  ══════════════════════════════════════════");
    println!("  LETTURA ATTIVA — {} paragrafi", paragraphs.len());
    println!("  stato iniziale: {} simplici, {} parole", simplici_prima, vocabolario_prima);
    println!("  ══════════════════════════════════════════");
    println!();

    for (idx, paragraph) in paragraphs.iter().enumerate() {
        let frasi = split_frasi(paragraph);
        let mut para_sconosciute: Vec<String> = Vec::new();

        for frase in &frasi {
            let frase = frase.trim();
            if frase.len() < 3 { continue; }
            frase_globale += 1;

            // Ogni clausola: teach() — co-attivazione stretta (leggero)
            for clausola in split_clausole(frase) {
                let c = clausola.trim();
                if c.len() > 3 { engine.teach(c); }
            }

            // Ogni 10 frasi: receive() completo — punto di riflessione
            if frase_globale % 10 == 0 {
                engine.receive(frase);
                for raw in &engine.last_unknown_words.clone() {
                    let word: String = raw.chars()
                        .filter(|c| c.is_alphabetic() || *c == '\'')
                        .collect::<String>()
                        .to_lowercase();
                    if word.len() < 3 { continue; }
                    if tutte_sconosciute.insert(word.clone()) {
                        domande_aperte.push(word.clone());
                    }
                    if !para_sconosciute.contains(&word) {
                        para_sconosciute.push(word);
                    }
                }
            }
        }

        // Stampa paragrafo con feedback
        let preview: String = paragraph.chars().take(70).collect();
        let preview = if paragraph.len() > 70 {
            format!("{}…", preview)
        } else {
            preview.to_string()
        };
        print!("  [§{:02}] {}", idx + 1, preview);

        if !para_sconosciute.is_empty() {
            println!();
            println!("        Dubbi: {}", para_sconosciute.join(", "));
        } else {
            println!();
        }

        // Sogno ogni 10 paragrafi — consolidamento leggero
        if (idx + 1) % 10 == 0 {
            engine.autonomous_tick();
            let rep = engine.report();
            println!("  ··· sogno ··· simplici: {}", rep.simplex_count);
            println!();
        }
    }

    // Sogno finale — 2 tick di consolidamento
    engine.autonomous_tick();
    engine.autonomous_tick();

    let rep_dopo = engine.report();
    let simplici_dopo = rep_dopo.simplex_count;
    let vocabolario_dopo = rep_dopo.vocabulary_size;

    println!();
    println!("  ══════════════════════════════════════════");
    println!("  FINE LETTURA");
    println!("  Simplici:    {} → {}  (+{})",
        simplici_prima, simplici_dopo,
        simplici_dopo.saturating_sub(simplici_prima));
    println!("  Vocabolario: {} → {}  (+{})",
        vocabolario_prima, vocabolario_dopo,
        vocabolario_dopo.saturating_sub(vocabolario_prima));
    println!("  Parole nuove sconosciute: {}", tutte_sconosciute.len());

    if !domande_aperte.is_empty() {
        let mostrate: Vec<&String> = domande_aperte.iter().take(12).collect();
        println!("  Domande aperte: {}", mostrate.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
    }

    // Pensieri emersi dopo la lettura
    let thoughts = generate_thoughts(engine);
    if !thoughts.is_empty() {
        println!();
        println!("  Pensieri emersi dopo la lettura:");
        for t in thoughts.iter().take(4) {
            println!("    [{:?}] {} — {:?}",
                t.kind,
                t.words.iter().take(3).cloned().collect::<Vec<_>>().join(", "),
                t.fractal_names.iter().take(2).cloned().collect::<Vec<_>>());
        }
    }

    println!("  ══════════════════════════════════════════");
    println!();
}

/// Divide un testo in frasi usando `.`, `!`, `?` come delimitatori.
fn split_frasi(text: &str) -> Vec<String> {
    let mut frasi = Vec::new();
    let mut corrente = String::new();
    for ch in text.chars() {
        corrente.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let f = corrente.trim().to_string();
            if !f.is_empty() { frasi.push(f); }
            corrente.clear();
        }
    }
    let rest = corrente.trim().to_string();
    if !rest.is_empty() { frasi.push(rest); }
    frasi
}

/// Divide una frase in clausole usando `,`, `;`, `:` come delimitatori.
fn split_clausole(frase: &str) -> Vec<String> {
    frase.split(|c| c == ',' || c == ';' || c == ':')
        .map(|s| s.to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// Visual Perception — Esperimento SVG
// ---------------------------------------------------------------------------

/// Comando :perceive <svg> — Prometeo "vede" un'immagine SVG
fn run_perceive_svg(engine: &mut PrometeoTopologyEngine, svg: &str) {
    use prometeo::topology::visual_perception::parse_svg_simple;
    
    println!();
    println!("  === PERCEZIONE VISIVA ===");
    println!();
    
    // Parse SVG per mostrare cosa è stato rilevato
    let concepts = parse_svg_simple(svg);
    
    if concepts.is_empty() {
        println!("  [nessun elemento SVG riconosciuto]");
        println!("  [supportati: <circle>, <rect>, <line>]");
        println!();
        return;
    }
    
    println!("  Elementi rilevati: {}", concepts.len());
    for (i, concept) in concepts.iter().enumerate() {
        print!("    {}. {} ", i+1, concept.shape);
        if let Some(ref color) = concept.color {
            print!("{} ", color);
        }
        print!("({:.0}, {:.0}) ", concept.position.0, concept.position.1);
        if concept.size < 20.0 {
            print!("piccolo");
        } else if concept.size < 50.0 {
            print!("medio");
        } else {
            print!("grande");
        }
        if !concept.relations.is_empty() {
            print!(" [{}]", concept.relations.join(", "));
        }
        println!();
    }
    println!();
    
    // Percepisce con il campo topologico
    let response = engine.perceive_svg(svg);
    
    println!("  Parole attivate: {}", response.words_activated.join(", "));
    println!("  Frattali dominanti: {}", response.dominant_fractals.join(", "));
    println!("  Energia campo: {:.3}", response.field_energy);
    println!();
    println!("  Descrizione emergente:");
    println!("  \"{}\"", response.description);
    println!();
    
    // Diagnostica
    if response.description.is_empty() || response.description == "..." {
        println!("  ⚠ Descrizione vuota — possibile mancanza vocabolario geometrico");
        println!("  Verifica con: cerchio, quadrato, rosso, blu, sopra, sotto");
    } else {
        let geo_words = ["cerchio", "quadrato", "linea", "rettangolo",
                         "rosso", "blu", "verde", "giallo", "nero", "bianco",
                         "sopra", "sotto", "vicino", "grande", "piccolo", "qui"];
        let geo_count = geo_words.iter()
            .filter(|w| response.description.to_lowercase().contains(*w))
            .count();
        
        if geo_count > 0 {
            println!("  ✓ Usa {} parole geometriche", geo_count);
        } else {
            println!("  ⚠ Non usa parole geometriche nella descrizione");
        }
    }
    println!();
}
