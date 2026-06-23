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
    // Phase 75: KG procedurale — pattern grammaticali per il pattern matcher
    engine.load_kg_procedural_from_file(Path::new("prometeo_kg_procedurale.json"));
    // Verifica KG caricato (se "essere" ha relazioni, il KG è attivo)
    let kg_size = engine.kg.total_degree("essere");
    if kg_size > 0 {
        println!("  KG attivo (essere: {} relazioni)", kg_size);
    }
    let kg_proc_size = engine.kg_procedural.edge_count;
    if kg_proc_size > 0 {
        println!("  KG procedurale attivo ({} archi)", kg_proc_size);
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

fn show_scene_understanding(engine: &PrometeoTopologyEngine) {
    use prometeo::topology::understanding::SyntacticRole;
    println!("{}", "═".repeat(60));
    println!("  COMPRENSIONE DELLA SCENA");
    println!("{}", "═".repeat(60));
    let scene = match &engine.last_scene {
        None => {
            println!("  (nessuna scena disponibile — inserisci prima un input)");
            println!("{}", "═".repeat(60));
            return;
        }
        Some(s) => s,
    };
    let role = match scene.syntactic_role {
        SyntacticRole::Statement   => "dichiarazione",
        SyntacticRole::Question    => "domanda",
        SyntacticRole::Exclamation => "esclamazione",
    };
    println!("  Ruolo sintattico:     {}", role);
    println!("  Profondità compr.:    {} archi letti", scene.comprehension_depth);
    if !scene.unknown_words.is_empty() {
        println!("  Parole ignote al KG:  {}", scene.unknown_words.join(", "));
    }

    println!();
    println!("  PER PAROLA (archi uscenti):");
    for u in scene.per_word.iter() {
        if u.arc_count() == 0 { continue; }
        println!("    {} ({} archi uscenti, {} entranti)",
            u.word,
            u.forward.values().map(|v| v.len()).sum::<usize>(),
            u.reverse.values().map(|v| v.len()).sum::<usize>());
        for (_facet, edges) in u.forward.iter() {
            for e in edges.iter() {
                println!("        [{}] → {} ({:.2})",
                    e.relation.as_str().to_lowercase(),
                    e.target, e.confidence);
            }
        }
    }

    println!();
    println!("  IPOTESI APERTE (concetti-perno sotto-definiti):");
    if scene.open_hypotheses.is_empty() {
        println!("    (nessuna)");
    } else {
        for h in scene.open_hypotheses.iter().take(6) {
            let invoc = h.dominant_invocation
                .map(|r| r.as_str().to_string())
                .unwrap_or_else(|| "-".to_string());
            let invokers = h.invoked_by.join(", ");
            println!("    · {} [sal={} def={} via {}] richiamato da: {}",
                h.concept, h.saliency, h.defining_arcs, invoc, invokers);
        }
    }
    println!("{}", "═".repeat(60));
}

/// Phase 80: cosa UI-r1 ha capito del parlante (SpeakerProfile) e
/// cosa ha fatto lei stessa (SelfProfile). È la "memoria del dialogo".
fn show_speaker(engine: &PrometeoTopologyEngine) {
    let sp = &engine.speaker_profile;
    println!("  ── SpeakerProfile (cosa UI-r1 ha capito del parlante) ──");
    println!("    Turni osservati: {}", sp.turn_count);
    match &sp.name {
        Some(n) => println!("    Nome:            {}", n),
        None    => println!("    Nome:            (non dichiarato)"),
    }
    if sp.self_facts.is_empty() {
        println!("    Self-facts:      (nessuno — il parlante non si è posizionato)");
    } else {
        println!("    Self-facts ({}):", sp.self_facts.len());
        for f in sp.self_facts.iter().take(10) {
            println!("      [t{}] {:?}: \"{}\"", f.turn, f.kind, f.predicate);
        }
    }
    if !sp.entity_facts.is_empty() {
        println!("    Entity-facts ({}) — cosa il parlante dice di UI-r1:", sp.entity_facts.len());
        for f in sp.entity_facts.iter().take(5) {
            println!("      [t{}] {:?}: \"{}\"", f.turn, f.kind, f.predicate);
        }
    }
    let open_q: Vec<_> = sp.unresolved_questions().take(5).collect();
    if !open_q.is_empty() {
        println!("    Domande aperte ({}):", open_q.len());
        for q in open_q {
            println!("      [t{}] \"{}\"", q.turn, q.raw_input);
        }
    }
    let open_g: Vec<_> = sp.open_gaps().take(5).collect();
    if !open_g.is_empty() {
        println!("    Gap aperti ({}) — vuoti non articolati:", open_g.len());
        for g in open_g {
            println!("      [t{}] {} su \"{}\"", g.turn, g.gap_kind, g.trigger);
        }
    }
    let top = sp.top_mentioned(6);
    if !top.is_empty() {
        let s: Vec<String> = top.iter().map(|(w, c)| format!("{}×{}", w, c)).collect();
        println!("    Più menzionati: {}", s.join(", "));
    }

    println!("  ── SelfProfile (cosa UI-r1 ha FATTO nel dialogo) ──");
    let sf = &engine.self_profile;
    if sf.decisions.is_empty() {
        println!("    (nessuna decisione registrata)");
    } else {
        println!("    Decisioni registrate: {}", sf.decisions.len());
        for d in sf.decisions.iter().rev().take(6).collect::<Vec<_>>().iter().rev() {
            let gap = d.gap_attended.as_ref()
                .map(|g| format!(" gap={}/{}", g.from, g.missing))
                .unwrap_or_default();
            println!("      [t{}] {} ({:?}){}",
                d.turn, d.kind.as_str(), d.narrative_subject, gap);
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

/// Phase 86 (Stadio 1): mostra il grafo di comprensione — i cammini tipati che
/// la frase inscrive nel mondo, col confronto polarità-vincolato e il grounding.
fn show_comprehension_graph(g: &prometeo::topology::comprehension_path::ComprehensionGraph) {
    use prometeo::topology::comprehension_path::{GroundKind, TypedPath};

    fn render_path(p: &TypedPath) -> String {
        let mut s = p.from.clone();
        for step in &p.steps {
            let via = step.via.as_deref().map(|v| format!("·via={}", v)).unwrap_or_default();
            if step.forward {
                s.push_str(&format!(" —{:?}{}→ {}", step.relation, via, step.to));
            } else {
                s.push_str(&format!(" ←{:?}{}— {}", step.relation, via, step.to));
            }
        }
        s
    }
    fn ground_label(k: &GroundKind) -> &'static str {
        match k {
            GroundKind::PropositionNode => "→ nodo-frase",
            GroundKind::SelfNode => "→ SÉ",
            GroundKind::Attractor => "→ attrattore",
            GroundKind::AlreadyGround => "(già terra)",
            GroundKind::Unreached => "✗ non fondato",
        }
    }

    println!("\n  — Grafo di comprensione (Phase 86 Stadio 1) —");
    let root = g.root.as_deref().unwrap_or("(Sé/parlante)");
    let target = g.target.as_deref().unwrap_or("—");
    let via = g.via.as_deref().map(|v| format!(" via={}", v)).unwrap_or_default();
    let pol = if g.polarity { "+" } else { "−" };
    println!("  proposizione : {} —{:?}→ {}{}  [pol {}]", root, g.relation, target, via, pol);
    println!("  confronto    : {:?}", g.confront);
    match &g.claim_path {
        Some(p) => println!("  cammino sogg→ogg: {}", render_path(p)),
        None => println!("  cammino sogg→ogg: (nessuno — soggetto non-Mondo o non connesso)"),
    }
    if g.groundings.is_empty() {
        println!("  grounding    : (nessuno)");
    } else {
        println!("  grounding    :");
        for p in &g.groundings {
            println!("    {}   {}", render_path(p), ground_label(&p.ground));
        }
    }
    if !g.ungrounded.is_empty() {
        println!("  gap onesti   : {:?}  (non so cosa siano / come si leghino)", g.ungrounded);
    }
    // Phase 86 Stadio 3: il collasso del cammino saliente in frase articolata.
    match prometeo::topology::path_collapse::collapse(g) {
        Some(s) => println!("  collasso (S3): «{}»", s),
        None => println!("  collasso (S3): (nulla di dicibile — gap onesto)"),
    }
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

    // Ultimi candidati-epifania mostrati da `:self_audit`, numerati: `:crystallize N`
    // valida (Nome-del-Padre) l'N-esimo e lo cristallizza come opinione del sé.
    let mut last_epiphanies: Vec<prometeo::topology::self_audit::CandidateEpiphany> = vec![];

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
                ":speaker" | ":sp" => {
                    show_speaker(&engine);
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
                ":tick" => {
                    // Esegue N tick autonomi manualmente — utile per testare
                    // autoconsapevolezza e dream senza aspettare il server.
                    let n: usize = arg.parse().unwrap_or(15);
                    for _ in 0..n {
                        engine.autonomous_tick();
                    }
                    let sw_len = engine.narrative_self.self_witness.len();
                    println!("  {} tick eseguiti | auto-osservazioni: {}", n, sw_len);
                    if sw_len > 0 {
                        let words = engine.narrative_self.self_witness.recent_words(5);
                        println!("  Ultime parole osservate: {:?}", words);
                    }
                }
                ":witness" => {
                    // Mostra le auto-osservazioni accumulate
                    let sw = &engine.narrative_self.self_witness;
                    if sw.is_empty() {
                        println!("  Nessuna auto-osservazione ancora. Usa :tick N prima.");
                    } else {
                        println!("\n  — Il testimone silenzioso ({} osservazioni) —", sw.len());
                        for w in sw.recent_words(10) {
                            print!("  {} ", w);
                        }
                        println!();
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
                ":understanding" | ":u" | ":scene" => {
                    show_scene_understanding(&engine);
                }
                ":self_audit" | ":sa" => {
                    // Phase 86+ (riconcezione): l'entità confronta il sé col
                    // mondo (kg_sem) — risonanze/tensioni sulle OPINIONI,
                    // epifanie candidate dai nodi-PENDENZA (opinioni potenziali,
                    // da validare prima di cristallizzare).
                    let report = prometeo::topology::self_audit::self_audit(
                        &engine.kg_self, &engine.kg);
                    println!("\n  — Auto-audit del sé ({} pendenze, {} opinioni × kg_sem) —",
                        engine.kg_self.pendenze.len(), engine.kg_self.len());
                    println!("  RISONANZE ({}) — il mondo conferma:", report.resonances.len());
                    for r in report.resonances.iter().take(12) {
                        println!("    {} {:?} {} @{:.2}", r.subject, r.relation, r.object, r.confidence);
                    }
                    println!("  TENSIONI ({}) — il mondo attrita:", report.tensions.len());
                    for t in report.tensions.iter().take(12) {
                        let pol = if t.self_polarity { "" } else { "NON " };
                        println!("    sé: {} {:?} {}{}  |  mondo: {:?}",
                            t.subject, t.relation, pol, t.object, t.world_link);
                    }
                    println!("  EPIFANIE CANDIDATE ({}) — opinioni potenziali (`:crystallize N` per validare):",
                        report.epiphanies.len());
                    last_epiphanies = report.epiphanies.clone();
                    for (i, e) in last_epiphanies.iter().enumerate() {
                        let via = e.via.as_deref().map(|v| format!(" via={}", v)).unwrap_or_default();
                        let warn = e.touches_non.as_deref()
                            .map(|n| format!("  ⚠ tocca un NON: {}", n)).unwrap_or_default();
                        println!("    [{:>2}] {} {:?} {}{}{}", i, e.from, e.relation, e.to, via, warn);
                    }
                }
                ":crystallize" | ":cry" => {
                    // Nome-del-Padre: la validazione umana di un candidato di
                    // `:self_audit`. L'opinione entra nel sé (kg_self vivo) e si
                    // persiste nel JSON — l'unico modo in cui un'opinione nasce.
                    match arg.trim().parse::<usize>() {
                        Ok(n) if n < last_epiphanies.len() => {
                            let e = &last_epiphanies[n];
                            // Confidenza derivata < innata: l'opinione si guadagna,
                            // si rafforza nel dialogo, mai per decreto.
                            let edge = prometeo::topology::kg_self::SelfEdge {
                                subject: e.from.clone(), relation: e.relation, object: e.to.clone(),
                                confidence: 0.5, polarity: true, innate: false, via: e.via.clone(),
                            };
                            match engine.crystallize_opinion(edge) {
                                Ok(true) => println!("  ✓ cristallizzata: {} {:?} {} (conf 0.50). Il sé ora ha {} opinioni.",
                                    e.from, e.relation, e.to, engine.kg_self.len()),
                                Ok(false) => println!("  · già presente (i due nodi erano già legati) — niente doppioni."),
                                Err(err) => println!("  ✗ {}", err),
                            }
                        }
                        Ok(n) => println!("  Indice {} fuori range (0..{}). Esegui prima :self_audit.", n, last_epiphanies.len()),
                        Err(_) => println!("  Uso: :crystallize N  (N = numero del candidato da :self_audit)"),
                    }
                }
                ":explore" | ":ex" => {
                    // Phase 86 (Stadio 1): il grafo di comprensione (pathfinding
                    // tipato) della proposizione. Con argomento esegue prima la
                    // frase come turno reale, poi mostra il grafo; senza, usa
                    // l'ultimo turno.
                    if !arg.is_empty() {
                        let _ = engine.receive(arg);
                    }
                    match engine.comprehension_graph() {
                        Some(g) => show_comprehension_graph(&g),
                        None => println!("  Nessuna proposizione: l'ultimo input non ha prodotto una SentenceProposition (prova una frase con soggetto/verbo)."),
                    }
                    // Phase 86 Stadio 2: analisi logica — i complementi disambiguati.
                    if let Some(p) = engine.last_sentence_proposition.as_ref() {
                        if !p.complements.is_empty() {
                            println!("  complementi  :");
                            for c in &p.complements {
                                let rel = c.relation.map(|r| format!("{:?}", r)).unwrap_or_else(|| "?".into());
                                println!("    {} {}  →  {}", c.preposition, c.noun, rel);
                            }
                        }
                    }
                }
                _ => {
                    println!("Comando sconosciuto. Usa :quit, :field, :feelings, :narrative, :needs, :recall, :recurring, :introspect, :kg, :stats, :understanding, :explore, :self_audit, :crystallize, :save");
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

        // ── Phase 77 debug: ActionDecision corrente (pattern matcher) ──
        if let Some(d) = engine.last_action_decision.as_ref() {
            let target_str = match &d.target {
                prometeo::topology::action_reasoning::ActionTarget::Gap { signifier_missing, from } =>
                    format!("Gap{{from={}, missing={}}}", from, signifier_missing),
                prometeo::topology::action_reasoning::ActionTarget::OpenQuestion { question_text, .. } =>
                    format!("OpenQ{{{}}}", question_text),
                prometeo::topology::action_reasoning::ActionTarget::Claim { kind, predicate } =>
                    format!("Claim{{{}={}}}", kind, predicate),
                prometeo::topology::action_reasoning::ActionTarget::PhaticClass { class } =>
                    format!("Phatic{{{}}}", class),
                prometeo::topology::action_reasoning::ActionTarget::Signifier { word } =>
                    format!("Signif{{{}}}", word),
            };
            println!("  ╰ DECISIONE: {} | {} | {} | anchors=[{}]",
                d.kind.as_str(),
                d.shape.as_str(),
                target_str,
                d.anchor_words.iter().take(6).cloned().collect::<Vec<_>>().join(", "));
        }

        // ── Phase 81 debug: SentenceProposition (la frase come triple) ──
        // Mostra come l'utterance è stata letta strutturalmente prima dei nuclei.
        // Es. "ho paura del futuro" → Speaker FeelsAs paura via=futuro (+) [obj✓ via✓]
        if let Some(p) = engine.last_sentence_proposition.as_ref() {
            use prometeo::topology::sentence_proposition::{SubjectRef, ObjectRef};
            let subj = match &p.subject {
                SubjectRef::Speaker     => "Speaker".to_string(),
                SubjectRef::Entity      => "Entity".to_string(),
                SubjectRef::World(s)    => format!("World({})", s),
                SubjectRef::Variable(w) => format!("?{}", w),
            };
            let obj = match &p.object {
                None                        => "_".to_string(),
                Some(ObjectRef::Word(w))    => w.clone(),
                Some(ObjectRef::Variable(w)) => format!("?{}", w),
            };
            let via = p.via.as_deref().map(|v| format!(" via={}", v)).unwrap_or_default();
            let pol = if p.polarity { "+" } else { "-" };
            let conf = engine.last_kg_confrontation.as_ref()
                .map(|c| format!(" [obj{} via{}{}]",
                    if c.object_in_kg { "✓" } else { "✗" },
                    if c.via_in_kg    { "✓" } else { "✗" },
                    if !c.contradictions.is_empty() { " contra" } else { "" }))
                .unwrap_or_default();
            println!("  ╰ PROP: {} {:?} {}{} ({}){}",
                subj, p.relation, obj, via, pol, conf);
        }

        // Phase 85: confronto col sé (rifrazione) + continuità tematica (Stage 3).
        if let Some(sc) = engine.last_self_confrontation.as_ref() {
            if !sc.is_empty() {
                println!("  ╰ SÉ: conflitti={} risonanze={} (maxC={:.2} maxR={:.2})",
                    sc.conflitti.len(), sc.risonanze.len(),
                    sc.max_conflitto(), sc.max_risonanza());
            }
        }
        if engine.self_continuity > 0.0 {
            println!("  ╰ continuità-sé: {:.2}", engine.self_continuity);
        }

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

        // Comprensione dal KG (fatti strutturali, non narrazione)
        if let Some(scene) = &engine.last_scene {
            if !scene.per_word.is_empty() || !scene.open_hypotheses.is_empty() {
                // Riassunto per-parola: archi uscenti più forti
                for u in scene.per_word.iter().take(3) {
                    if u.arc_count() == 0 { continue; }
                    let mut highlights: Vec<String> = Vec::new();
                    for (facet, edges) in u.forward.iter() {
                        let _ = facet;
                        for e in edges.iter().take(2) {
                            highlights.push(format!("{}→{}({:.2})",
                                e.relation.as_str().to_lowercase().replace('_', ""),
                                e.target, e.confidence));
                            if highlights.len() >= 4 { break; }
                        }
                        if highlights.len() >= 4 { break; }
                    }
                    if !highlights.is_empty() {
                        println!("  ╰ {}: {}", u.word, highlights.join(" · "));
                    }
                }
                let hyps: Vec<String> = scene.open_hypotheses.iter().take(2)
                    .map(|h| format!("{}?[sal={} def={}]", h.concept, h.saliency, h.defining_arcs)).collect();
                if !hyps.is_empty() {
                    println!("  ╰ ipotesi: {}", hyps.join(" · "));
                }
            }
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
