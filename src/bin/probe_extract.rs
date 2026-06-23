//! probe_extract — diagnostica offline della catena claim→PROP→confronto.
//!
//! Replica esattamente la lettura dell'engine (tokenizzazione `clean_token`,
//! `detect_speaker_claim`, `extract_propositions`, `confront_with_kg`) sui KG
//! reali, senza server né stato. Uso:
//!   cargo run --release --bin probe_extract -- "il silenzio ha un significato"
//! Senza argomenti legge le frasi da stdin (una per riga) — comodo per batterie.

use prometeo::topology::knowledge_graph::{KgSnapshot, KnowledgeGraph};
use prometeo::topology::lexicon::Lexicon;

fn load(path: &str) -> KnowledgeGraph {
    let json = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let snap: KgSnapshot = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{path}: {e}"));
    KnowledgeGraph::from_snapshot(snap)
}

fn tokenize(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter_map(prometeo::topology::lexicon::clean_token)
        .filter(|w| !w.is_empty())
        .collect()
}

fn analyze(s: &str, lex: &Lexicon, kg: &KnowledgeGraph, kg_proc: &KnowledgeGraph) {
    let tokens = tokenize(s);
    println!("\n>>> {s}");
    let claim = prometeo::topology::input_reading::detect_speaker_claim(
        &tokens, lex, Some(kg), Some(kg_proc),
    );
    println!("    CLAIM: {claim:?}");
    let props = prometeo::topology::sentence_proposition::extract_propositions(
        &tokens, lex, Some(kg_proc), Some(kg),
    );
    for c in &props {
        match &c.prop {
            Some(p) => {
                let confr = prometeo::topology::sentence_proposition::confront_with_kg(p, kg);
                println!(
                    "    PROP[{}{}]: {:?} {:?} {:?} via={:?} pol={} | matches={} obj_kg={} via_kg={}",
                    if c.subordinate { "sub" } else { "ind" },
                    c.range.start,
                    p.subject, p.relation, p.object, p.via, p.polarity,
                    confr.matches, confr.object_in_kg, confr.via_in_kg,
                );
            }
            None => println!("    PROP[{}]: None", if c.subordinate { "sub" } else { "ind" }),
        }
    }
}

fn main() {
    let kg_proc = load("prometeo_kg_procedurale.json");
    let kg = load("prometeo_kg.json");
    let lex = Lexicon::bootstrap();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        use std::io::BufRead;
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            analyze(line, &lex, &kg, &kg_proc);
        }
    } else {
        for s in &args {
            analyze(s, &lex, &kg, &kg_proc);
        }
    }
}
