//! dialogue_bench — strumento di MISURA dei dialoghi (Phase 86).
//!
//! Lo strumento prima del giudizio. Dato un file di input (un'enunciazione per
//! riga), per ogni turno emette in forma COMPATTA e DIFFABILE:
//!   - PROP   : la SentenceProposition (chi-dice-cosa-su-chi, ruoli + polarità)
//!   - CONFR  : il confronto col mondo (Confirm/Contradict/Novelty/NotApplicable)
//!   - CLAIM  : il cammino diretto soggetto→oggetto trovato dal pathfinding
//!   - GROUND : i cammini di grounding (verso attrattore/sé/nodo-frase)
//!   - GAP    : i nodi non fondati (gap onesti)
//!   - S3     : il collasso del cammino saliente (path_collapse, Stadio 3)
//!   - OUT    : l'output REALE (la pipeline compose attuale)
//!
//! Il divario fra S3/grafo e OUT è ciò che misura "comprensione vs espressione".
//! I VERDETTI umani (la risposta è buona? quale asse manca?) si annotano sopra
//! questo output — lo strumento non giudica, mostra.
//!
//! Formato file input:
//!   - una enunciazione per riga;
//!   - righe che iniziano con `#` = etichette di sezione (stampate, non inviate);
//!   - righe vuote ignorate.
//! Sessione UNICA e continua (SpeakerProfile/narrativa si accumulano → la
//! continuità multi-turno è osservabile). `--teach` per simulare l'apprendimento.
//!
//! Uso:
//!   cargo run --release --bin dialogue_bench -- corpus.txt
//!   cargo run --release --bin dialogue_bench -- corpus.txt --state s.bin --teach

use std::path::Path;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::comprehension_path::{ComprehensionGraph, GroundKind, TypedPath};
use prometeo::topology::sentence_proposition::{ObjectRef, SubjectRef};
use prometeo::topology::analisi_logica::{analizza, Analisi};

fn load_engine(bin_path: &Path) -> PrometeoTopologyEngine {
    let mut engine = if bin_path.exists() {
        match PrometeoState::load_from_binary(bin_path) {
            Ok(state) => {
                let mut eng = PrometeoTopologyEngine::new_empty();
                state.restore_lexicon(&mut eng);
                eprintln!("✓ stato: {} parole", eng.lexicon.word_count());
                eng
            }
            Err(e) => {
                eprintln!("✗ load: {} — nuovo stato", e);
                PrometeoTopologyEngine::new()
            }
        }
    } else {
        eprintln!("  nessun .bin — nuovo stato");
        PrometeoTopologyEngine::new()
    };
    engine.lexicon.apply_curated_signatures();
    engine.recompute_all_word_affinities();
    engine.rebuild_pf_field();
    engine.load_kg_from_file(Path::new("prometeo_kg.json"));
    engine.load_kg_procedural_from_file(Path::new("prometeo_kg_procedurale.json"));
    eprintln!("  kg: {} archi | kg_proc: {} archi", engine.kg.edge_count, engine.kg_procedural.edge_count);
    engine
}

fn render_path(p: &TypedPath) -> String {
    let mut s = p.from.clone();
    for step in &p.steps {
        let via = step.via.as_deref().map(|v| format!("·via={v}")).unwrap_or_default();
        if step.forward {
            s.push_str(&format!(" —{:?}{}→ {}", step.relation, via, step.to));
        } else {
            s.push_str(&format!(" ←{:?}{}— {}", step.relation, via, step.to));
        }
    }
    s
}

fn ground_tag(k: &GroundKind) -> &'static str {
    match k {
        GroundKind::PropositionNode => "nodo-frase",
        GroundKind::SelfNode => "SÉ",
        GroundKind::Attractor => "attrattore",
        GroundKind::AlreadyGround => "già-terra",
        GroundKind::Unreached => "✗",
    }
}

fn show_prop(engine: &PrometeoTopologyEngine) {
    match engine.last_sentence_proposition.as_ref() {
        None => println!("  PROP   : —  (nessuna proposizione estratta)"),
        Some(p) => {
            let subj = match &p.subject {
                SubjectRef::Speaker => "Speaker".into(),
                SubjectRef::Entity => "Entity".into(),
                SubjectRef::World(s) => format!("World({s})"),
                SubjectRef::Variable(w) => format!("?{w}"),
            };
            let obj = match &p.object {
                None => "_".into(),
                Some(ObjectRef::Word(w)) => w.clone(),
                Some(ObjectRef::Variable(w)) => format!("?{w}"),
            };
            let via = p.via.as_deref().map(|v| format!(" via={v}")).unwrap_or_default();
            let pol = if p.polarity { "+" } else { "−" };
            println!("  PROP   : {} {:?} {}{} ({})", subj, p.relation, obj, via, pol);
            if !p.complements.is_empty() {
                let cs: Vec<String> = p.complements.iter()
                    .map(|c| format!("{} {}→{}", c.preposition, c.noun,
                        c.relation.map(|r| format!("{r:?}")).unwrap_or_else(|| "?".into())))
                    .collect();
                println!("  COMPL  : {}", cs.join(" | "));
            }
        }
    }
}

/// Analisi logica clausa-aware (Phase 86+): le clausole con il ruolo di OGNI
/// token. Osservativa — non ancora cablata nella PROP/OUT.
fn show_analisi(raw: &[String], a: &Analisi) {
    let clausole: Vec<String> = a.clausole.iter().map(|c| {
        let toks: Vec<String> = c.range.clone()
            .map(|i| format!("{}/{}", raw[i], a.funzioni[i].sigla()))
            .collect();
        let pred = c.predicato_lemma.as_deref()
            .map(|l| format!(" ⟨{l}⟩")).unwrap_or_default();
        format!("[{}{}]", toks.join(" "), pred)
    }).collect();
    println!("  ANALISI: {}", clausole.join("  |  "));
}

/// Multi-locus (Phase 86+): le proposizioni per clausola, con quale è la
/// primaria e quali subordinate. Visibile solo se >1 clausola.
fn show_loci(engine: &PrometeoTopologyEngine) {
    let props = &engine.last_sentence_propositions;
    if props.len() <= 1 { return; }
    let primary = prometeo::topology::sentence_proposition::primary_index(props);
    let loci: Vec<String> = props.iter().enumerate().map(|(i, c)| {
        let mark = if Some(i) == primary { "▶" } else if c.subordinate { "└sub" } else { "·" };
        let body = match &c.prop {
            None => "—".to_string(),
            Some(p) => {
                let subj = match &p.subject {
                    SubjectRef::Speaker => "Speaker".into(),
                    SubjectRef::Entity => "Entity".into(),
                    SubjectRef::World(s) => format!("World({s})"),
                    SubjectRef::Variable(w) => format!("?{w}"),
                };
                let obj = match &p.object {
                    None => "_".into(),
                    Some(ObjectRef::Word(w)) => w.clone(),
                    Some(ObjectRef::Variable(w)) => format!("?{w}"),
                };
                let via = p.via.as_deref().map(|v| format!(" via={v}")).unwrap_or_default();
                format!("{} {:?} {}{}", subj, p.relation, obj, via)
            }
        };
        format!("{mark} {body}")
    }).collect();
    let indep = prometeo::topology::sentence_proposition::independent_locus_count(props);
    println!("  LOCI   : {}   (indipendenti={indep})", loci.join("   "));
}

fn show_graph(g: &ComprehensionGraph) {
    println!("  CONFR  : {:?}", g.confront);
    match &g.claim_path {
        Some(p) => println!("  CLAIM  : {}", render_path(p)),
        None => println!("  CLAIM  : —"),
    }
    let grounded: Vec<String> = g.groundings.iter()
        .filter(|p| !p.steps.is_empty())
        .map(|p| format!("{} [{}]", render_path(p), ground_tag(&p.ground)))
        .collect();
    if !grounded.is_empty() {
        println!("  GROUND : {}", grounded.join("  ·  "));
    }
    if !g.ungrounded.is_empty() {
        println!("  GAP    : {:?}", g.ungrounded);
    }
}

fn main() {
    let mut input_file: Option<String> = None;
    let mut state_path = Path::new("prometeo_topology_state.bin").to_path_buf();
    let mut teach = false;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--state" | "-s" if i + 1 < args.len() => { state_path = Path::new(&args[i + 1]).into(); i += 2; }
            "--teach" => { teach = true; i += 1; }
            other if !other.starts_with('-') => { input_file = Some(other.to_string()); i += 1; }
            _ => { i += 1; }
        }
    }

    let Some(input_file) = input_file else {
        eprintln!("Uso: dialogue_bench <corpus.txt> [--state s.bin] [--teach]");
        std::process::exit(2);
    };
    let corpus = std::fs::read_to_string(&input_file)
        .unwrap_or_else(|e| { eprintln!("✗ {input_file}: {e}"); std::process::exit(2); });

    let mut engine = load_engine(&state_path);
    println!("# bench: {input_file} | teach={teach}");
    println!("{}", "═".repeat(72));

    let mut turn = 0usize;
    for raw in corpus.lines() {
        let line = raw.trim();
        if line.is_empty() { continue; }
        if let Some(label) = line.strip_prefix('#') {
            println!("\n## {}", label.trim());
            continue;
        }

        turn += 1;
        if teach { engine.teach(line); }
        let _ = engine.receive(line);
        let generated = engine.generate_willed();

        println!("\n[{turn}] «{line}»");
        {
            let a = analizza(&engine.last_input_tokens_full, &engine.kg_procedural, &engine.kg);
            show_analisi(&engine.last_input_tokens_full, &a);
        }
        show_prop(&engine);
        show_loci(&engine);
        if let Some(g) = engine.comprehension_graph() {
            show_graph(&g);
            let ss = prometeo::topology::comprehension_path::self_salience(&g, &engine.kg_self);
            println!("  SELF-SAL: {:.2}", ss);
            match prometeo::topology::path_collapse::collapse(&g) {
                Some(s) => println!("  S3     : «{s}»"),
                None => println!("  S3     : —  (gap onesto / soggetto non-Mondo)"),
            }
        }
        {
            let residuo = prometeo::topology::sentence_proposition::unaccounted_tokens(
                &engine.last_input_words,
                engine.last_sentence_proposition.as_ref(),
                Some(&engine.kg_procedural),
                Some(&engine.kg),
            );
            if residuo.is_empty() {
                println!("  RESIDUO: 0  ✓ ogni parola ha un ruolo");
            } else {
                println!("  RESIDUO: {} → [{}]", residuo.len(), residuo.join(", "));
            }
        }
        match &engine.last_need {
            Some(n) => {
                let others: Vec<String> = n.ranked.iter().skip(1)
                    .filter(|(_, i)| *i > 0.0)
                    .map(|(nd, i)| format!("{}={:.2}", nd.as_str(), i))
                    .collect();
                println!("  BISOGNO: {} ({:.2}){}", n.dominant.as_str(), n.intensity,
                    if others.is_empty() { String::new() } else { format!("  · {}", others.join(" ")) });
            }
            None => println!("  BISOGNO: —  (nessuna tensione)"),
        }
        println!("  OUT    : «{}»", generated.text);
    }

    println!("\n{}", "═".repeat(72));
    println!("# {turn} turni misurati");
}
