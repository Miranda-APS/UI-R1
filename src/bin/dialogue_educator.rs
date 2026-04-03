/// dialogue_educator — Educazione dialogica con UI-r1
///
/// Interfaccia interattiva per dialogare, educare e osservare l'entità.
/// Ogni messaggio viene prima insegnato (teach), poi ricevuto (receive),
/// così UI-r1 apprende dal dialogo stesso.
///
/// Comandi introspettivi:
///   :field         — parole attive nel campo (top 15)
///   :feelings      — valenza Octalysis 8D corrente
///   :narrative     — stance, intenzione, impegno volitivo
///   :needs         — stato dei bisogni (7 livelli Maslow)
///   :recall [n]    — ultimi N episodi semantici (default 5)
///   :recurring     — concetti più ricorrenti nella storia
///   :introspect    — dump completo dello stato interno
///   :kg <word>     — relazioni KG per una parola
///   :stats         — statistiche sistema
///   :save          — salva stato
///   :quit          — esci e salva
///
/// Uso:
///   cargo run --release --bin dialogue-educator
///   cargo run --release --bin dialogue-educator -- --state mia_sessione.bin

use std::io::{self, Write};
use std::path::Path;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::valence::DRIVE_NAMES;
use prometeo::topology::knowledge_graph::KnowledgeGraph;


fn load_or_create_engine(bin_path: &Path) -> PrometeoTopologyEngine {
    let mut engine = if bin_path.exists() {
        match PrometeoState::load_from_binary(bin_path) {
            Ok(state) => {
                let mut eng = PrometeoTopologyEngine::new_empty();
                state.restore_lexicon(&mut eng);
                let words = eng.lexicon.word_count();
                let eps = eng.semantic_episodes.len();
                println!("✓ Stato caricato: {} parole, {} episodi semantici", words, eps);
                eng
            }
            Err(e) => {
                eprintln!("✗ Errore caricamento: {}", e);
                println!("  Creo nuovo stato...");
                PrometeoTopologyEngine::new()
            }
        }
    } else {
        println!("  Nuovo stato (nessun .bin trovato)");
        PrometeoTopologyEngine::new()
    };

    // Setup post-restore (stesso ordine del server)
    engine.lexicon.apply_curated_signatures();
    engine.recompute_all_word_affinities();
    engine.rebuild_pf_field();
    // Carica il KG da prometeo_kg.json (necessario per :kg, relazioni KG, expression nuclei)
    engine.load_kg_from_file(Path::new("prometeo_kg.json"));
    // Verifica KG caricato (se "essere" ha relazioni, il KG è attivo)
    let kg_size = engine.kg.total_degree("essere");
    if kg_size > 0 {
        println!("  KG attivo (essere: {} relazioni)", kg_size);
    }

    engine
}

fn save_state(engine: &PrometeoTopologyEngine, bin_path: &Path) {
    let state = PrometeoState::capture(engine);
    match state.save_to_binary(bin_path) {
        Ok(()) => println!("  [Salvato in {}]", bin_path.display()),
        Err(e) => eprintln!("✗ Errore salvataggio: {}", e),
    }
}

fn show_field(engine: &PrometeoTopologyEngine, top_n: usize) {
    let active = engine.word_topology.active_words();
    if active.is_empty() {
        println!("  (campo vuoto — nessuna parola attiva)");
        return;
    }
    let mut sorted: Vec<_> = active.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    println!("  Campo attivo ({} parole):", sorted.len());
    for (word, act) in sorted.iter().take(top_n) {
        let resting = engine.lexicon.get(word)
            .map(|p| p.stability * 0.003).unwrap_or(0.0);
        let delta = (*act - resting).max(0.0);
        println!("    {:<18} act={:.3}  Δ={:.3}", word, act, delta);
    }
}

fn show_feelings(engine: &PrometeoTopologyEngine) {
    let drives = &engine.narrative_self.valence.drives;
    println!("  Valenza Octalysis 8D:");
    for (i, &d) in drives.iter().enumerate() {
        let bar_len = ((d.abs() * 12.0).round() as usize).min(12);
        println!("    CD{} {:<14} {:>+.3}  |{}",
            i + 1, DRIVE_NAMES[i], d, bar_visual(d));
    }
}

fn bar_visual(v: f64) -> String {
    let width = 20usize;
    let mid = width / 2;
    let magnitude = (v.abs() * mid as f64).round() as usize;
    let magnitude = magnitude.min(mid);
    let mut bar = vec![' '; width];
    bar[mid] = '|';
    if v >= 0.0 {
        for i in mid..mid + magnitude {
            bar[i] = '█';
        }
    } else {
        for i in (mid - magnitude)..mid {
            bar[i] = '█';
        }
    }
    bar.iter().collect()
}

fn show_narrative(engine: &PrometeoTopologyEngine) {
    let ns = &engine.narrative_self;
    println!("  Stato narrativo:");
    println!("    Stance:     {}", ns.stance.as_str());
    if let Some(ref intent) = ns.pending_intention {
        println!("    Intenzione: {}", intent.as_str());
    } else {
        println!("    Intenzione: (nessuna)");
    }
    if let Some(ref c) = ns.commitment {
        println!("    Impegno:    {} (forza={:.2}, turni={})",
            c.intention.as_str(), c.strength, c.turns_held);
    }
    println!("    Continuità: {:.2}", ns.topic_continuity);
    println!("    Nato:       {}", if ns.is_born { "sì" } else { "no" });
    println!("    Episodi:    {}", engine.semantic_episodes.len());
}

fn show_needs(engine: &PrometeoTopologyEngine) {
    let level_names = [
        "L1 Sopravvivenza",
        "L2 Sicurezza",
        "L3 Appartenenza",
        "L4 Stima/Comprensione",
        "L5 Realizzazione",
        "L6 Autotrascendenza",
        "L7 Spiritualità",
    ];
    if let Some(ref ns) = engine.last_needs_state {
        println!("  Bisogni (soddisfazione):");
        for (i, &sat) in ns.satisfaction.iter().enumerate() {
            let name = level_names.get(i).copied().unwrap_or("?");
            let pct = (sat * 100.0).round() as u32;
            let bar_len = (sat * 20.0).round() as usize;
            let bar = "█".repeat(bar_len) + &" ".repeat(20 - bar_len);
            println!("    {:<24} {:>3}%  [{}]", name, pct, bar);
        }
    } else {
        println!("  (stato bisogni non ancora calcolato — fai receive() prima)");
    }
}

fn show_recall(engine: &PrometeoTopologyEngine, n: usize) {
    let recent = engine.semantic_episodes.recent(n);
    if recent.is_empty() {
        println!("  (nessun episodio semantico ancora)");
        return;
    }
    println!("  Ultimi {} episodi semantici:", recent.len());
    for ep in recent.iter().rev() {
        println!("  ─");
        println!("    [ep.{}]  Stance: {}  |  Intenzione: {}",
            ep.id, ep.stance, ep.intention);
        println!("    Concetti: {}",
            ep.key_concepts.iter().take(6).cloned().collect::<Vec<_>>().join(", "));
        println!("    Riassunto: {}", ep.summary);
        if !ep.active_values.is_empty() {
            println!("    Valori: {}", ep.active_values.join(", "));
        }
    }
}

fn show_recurring(engine: &PrometeoTopologyEngine) {
    let top = engine.semantic_episodes.top_recurring_concepts(15);
    if top.is_empty() {
        println!("  (nessun concetto ricorrente ancora)");
        return;
    }
    println!("  Concetti più ricorrenti nella storia di UI-r1:");
    for (concept, count) in &top {
        let bar = "█".repeat((*count).min(20));
        println!("    {:<20} {:>3}  {}", concept, count, bar);
    }
}

fn show_kg(kg: &KnowledgeGraph, word: &str) {
    let outgoing = kg.all_outgoing(word);
    let incoming = kg.all_incoming(word);

    if outgoing.is_empty() && incoming.is_empty() {
        println!("  (nessuna relazione KG trovata per '{}')", word);
        return;
    }

    let total = kg.total_degree(word);
    println!("  KG per '{}' (grado totale = {}):", word, total);

    if !outgoing.is_empty() {
        println!("  → Uscenti:");
        let mut shown = 0;
        for (rel, obj, conf) in &outgoing {
            if shown >= 15 { println!("    ... ({} altri)", outgoing.len() - 15); break; }
            println!("    {:?}  →  {}  (conf={:.2})", rel, obj, conf);
            shown += 1;
        }
    }

    if !incoming.is_empty() {
        println!("  ← Entranti:");
        let mut shown = 0;
        for (rel, subj, conf) in &incoming {
            if shown >= 10 { println!("    ... ({} altri)", incoming.len() - 10); break; }
            println!("    {}  →{:?}→  {}  (conf={:.2})", subj, rel, word, conf);
            shown += 1;
        }
    }
}

fn show_introspect(engine: &mut PrometeoTopologyEngine) {
    println!("{}", "═".repeat(60));
    println!("  STATO INTERNO — UI-r1");
    println!("{}", "═".repeat(60));
    println!();
    show_field(engine, 10);
    println!();
    show_feelings(engine);
    println!();
    show_narrative(engine);
    println!();
    show_needs(engine);
    println!();
    let vital = engine.vital_state();
    println!("  Stato vitale:");
    println!("    Attivazione: {:.3}  Curiosità: {:.3}  Fatica: {:.3}",
        vital.activation, vital.curiosity, vital.fatigue);
    println!("{}", "═".repeat(60));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut state_path = Path::new("prometeo_topology_state.bin").to_path_buf();
    let mut teach_mode = true; // default: insegna + ricevi

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--state" | "-s" if i + 1 < args.len() => {
                state_path = Path::new(&args[i + 1]).to_path_buf();
                i += 2;
            }
            "--no-teach" => {
                teach_mode = false;
                i += 1;
            }
            _ => { i += 1; }
        }
    }

    println!("{}", "═".repeat(70));
    println!("  UI-r1 — Dialogo Educativo");
    println!("{}", "═".repeat(70));
    println!();
    if teach_mode {
        println!("  Modalità: DIALOGO + APPRENDIMENTO");
        println!("  Ogni messaggio viene prima insegnato, poi ricevuto.");
    } else {
        println!("  Modalità: SOLO DIALOGO (--no-teach)");
    }
    println!();

    let mut engine = load_or_create_engine(&state_path);
    let mut turn_count = 0usize;
    let mut words_learned = 0usize;

    println!("Comandi:");
    println!("  :quit / :q       — esci e salva");
    println!("  :save            — salva stato");
    println!("  :field           — campo attivo (top 15)");
    println!("  :feelings        — valenza Octalysis 8D");
    println!("  :narrative       — stance, intenzione, impegno");
    println!("  :needs           — bisogni (7 livelli)");
    println!("  :recall [n]      — ultimi N episodi (default 5)");
    println!("  :recurring       — concetti più ricorrenti");
    println!("  :introspect      — dump completo stato interno");
    println!("  :kg <parola>     — relazioni KG per una parola");
    println!("  :stats           — statistiche sistema");
    println!();
    println!("{}", "─".repeat(70));

    loop {
        print!("\n[Tu] > ");
        io::stdout().flush().ok();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {},
            Err(e) => {
                eprintln!("Errore: {}", e);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // ─── Comandi ───────────────────────────────────────────────────────
        if input.starts_with(':') {
            let parts: Vec<&str> = input.splitn(2, ' ').collect();
            let cmd = parts[0];
            let arg = parts.get(1).copied().unwrap_or("");

            match cmd {
                ":quit" | ":q" | ":exit" => {
                    println!("\nSalvataggio stato...");
                    save_state(&engine, &state_path);
                    println!("Turni: {} | Parole apprese: {} | Episodi: {}/{}",
                        turn_count, words_learned,
                        engine.semantic_episodes.len(),
                        engine.semantic_episodes.total_recorded());
                    break;
                }
                ":save" => {
                    save_state(&engine, &state_path);
                }
                ":field" | ":f" => {
                    show_field(&engine, 15);
                }
                ":feelings" => {
                    show_feelings(&engine);
                }
                ":narrative" | ":n" => {
                    show_narrative(&engine);
                }
                ":needs" => {
                    show_needs(&engine);
                }
                ":recall" | ":r" => {
                    let n: usize = arg.parse().unwrap_or(5);
                    show_recall(&engine, n);
                }
                ":recurring" => {
                    show_recurring(&engine);
                }
                ":introspect" | ":i" => {
                    show_introspect(&mut engine);
                }
                ":kg" => {
                    if arg.is_empty() {
                        println!("  Uso: :kg <parola>");
                    } else {
                        show_kg(&engine.kg, arg);
                    }
                }
                ":stats" => {
                    println!("\n{}", "═".repeat(60));
                    println!("  Parole lessico:     {}", engine.lexicon.word_count());
                    println!("  Archi topologia:    {}", engine.word_topology.edge_count());
                    println!("  Simplessi:          {}", engine.complex.count());
                    println!("  Episodi semantici:  {}/{} (finestra/totale)",
                        engine.semantic_episodes.len(),
                        engine.semantic_episodes.total_recorded());
                    println!("  Turni dialogo:      {}", turn_count);
                    println!("  Parole apprese:     {}", words_learned);
                    let recurring = engine.semantic_episodes.top_recurring_concepts(3);
                    if !recurring.is_empty() {
                        let themes: Vec<String> = recurring.iter()
                            .map(|(w, c)| format!("{}({})", w, c)).collect();
                        println!("  Temi ricorrenti:    {}", themes.join(", "));
                    }
                    println!("{}", "═".repeat(60));
                }
                _ => {
                    println!("Comando sconosciuto. Usa :quit, :field, :feelings, :narrative, :needs, :recall, :recurring, :introspect, :kg, :stats, :save");
                }
            }
            continue;
        }

        // ─── FASE 1: INSEGNA (opzionale) ───────────────────────────────────
        if teach_mode {
            let teach_result = engine.teach(input);
            words_learned += teach_result.new_count;
            if teach_result.new_count > 0 {
                println!("  [Apprese {} parole: {}]",
                    teach_result.new_count,
                    teach_result.words_new.iter().take(5).cloned().collect::<Vec<_>>().join(", "));
            }
        }

        // ─── FASE 2: RICEVI ────────────────────────────────────────────────
        let _response = engine.receive(input);

        // ─── FASE 3: GENERA ────────────────────────────────────────────────
        let generated = engine.generate_willed();
        turn_count += 1;

        // ─── Output principale ─────────────────────────────────────────────
        println!("\n[UI-r1] > {}", generated.text);

        // Hint interno: stance + drives dominanti (formato compatto)
        let stance = engine.narrative_self.stance.as_str();
        let drives = &engine.narrative_self.valence.drives;
        let dom_drives: Vec<String> = drives.iter().enumerate()
            .filter(|(_, &d)| d.abs() > 0.15)
            .map(|(i, &d)| format!("{}{:+.2}", DRIVE_NAMES[i], d))
            .collect();
        let intent_str = engine.narrative_self.pending_intention.as_ref()
            .map(|i| i.as_str()).unwrap_or("-");
        let ep_total = engine.semantic_episodes.total_recorded();
        let ep_window = engine.semantic_episodes.len();

        if dom_drives.is_empty() {
            println!("  ╰ {} | {} | ep.{}/{}", stance, intent_str, ep_window, ep_total);
        } else {
            println!("  ╰ {} | {} | {} | ep.{}/{}",
                stance, intent_str,
                dom_drives.iter().take(3).cloned().collect::<Vec<_>>().join(" "),
                ep_window, ep_total);
        }

        // Auto-save ogni 20 turni
        if turn_count % 20 == 0 {
            save_state(&engine, &state_path);
            println!("  [Auto-save: turno {}]", turn_count);
        }
    }

    println!("\n{}", "═".repeat(70));
    println!("  Dialogo terminato — turni: {}", turn_count);
    println!("{}", "═".repeat(70));
}
