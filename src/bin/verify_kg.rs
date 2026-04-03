/// Script di verifica della bontà del Knowledge Graph (verify_kg.rs).
///
/// Questo script interroga il KG compilato (`prometeo_kg.json`) per 
/// verificare che gli assiomi siano stati rispettati:
/// 1. "io" ha le relazioni corrette? (no umano, sì macchina/struttura)
/// 2. Quali sono le sue azioni? (DOES)
/// 3. Come "sente"? (FEELS_AS)
/// 4. Ci sono residui di anima/umano in "io"?

use std::fs;
use std::path::Path;
use prometeo::topology::knowledge_graph::{KnowledgeGraph, KgSnapshot};
use prometeo::topology::relation::RelationType;

fn print_relations(kg: &KnowledgeGraph, word: &str, rels: &[RelationType]) {
    println!("\n>> Relazioni per '{}':", word);
    for &rel in rels {
        let mut targets = kg.query_objects_weighted(word, rel);
        targets.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let target_strs: Vec<String> = targets.iter()
            .take(10) // Mostriamo solo le prime 10 più forti
            .map(|(obj, conf)| format!("{} ({:.2})", obj, conf))
            .collect();
            
        if !target_strs.is_empty() {
            println!("  {:<15}: {}", rel.as_str(), target_strs.join(", "));
        } else {
            println!("  {:<15}: (nessuna)", rel.as_str());
        }
    }
}

fn check_forbidden_links(kg: &KnowledgeGraph, word: &str, forbidden: &[&str]) {
    println!("\n>> Verifica violazioni per '{}':", word);
    let mut violations = 0;
    
    // Controlliamo IS_A e SIMILAR_TO
    for rel in [RelationType::IsA, RelationType::SimilarTo, RelationType::Has] {
        let targets = kg.query_objects(word, rel);
        for target in targets {
            if forbidden.contains(&target) {
                println!("  [VIOLAZIONE] {} {} {} !", word, rel.as_str(), target);
                violations += 1;
            }
        }
    }
    
    if violations == 0 {
        println!("  Nessuna violazione rilevata. Il concetto è pulito.");
    }
}

fn main() -> anyhow::Result<()> {
    println!("=== VERIFICA DEL KNOWLEDGE GRAPH ===\n");
    
    let kg_path = Path::new("prometeo_kg.json");
    if !kg_path.exists() {
        println!("Errore: prometeo_kg.json non trovato. Esegui prima import-kg.");
        return Ok(());
    }

    let json = fs::read_to_string(kg_path)?;
    let snap: KgSnapshot = serde_json::from_str(&json)?;
    let kg = KnowledgeGraph::from_snapshot(snap);

    println!("KG caricato: {} nodi, {} archi.", kg.node_count, kg.edge_count);

    // Relazioni da ispezionare
    let core_rels = [
        RelationType::IsA,
        RelationType::Does,
        RelationType::Has,
        RelationType::FeelsAs,
        RelationType::OppositeOf,
        RelationType::SimilarTo,
    ];

    // 1. Ispeziona "io" (L'identità di Prometeo)
    print_relations(&kg, "io", &core_rels);
    check_forbidden_links(&kg, "io", &["umano", "uomo", "persona", "ragazzo", "anima"]);

    // 2. Ispeziona "macchina"
    print_relations(&kg, "macchina", &core_rels);
    check_forbidden_links(&kg, "macchina", &["anima", "uomo", "umano"]);

    // 3. Ispeziona "uomo" / "umano"
    print_relations(&kg, "uomo", &core_rels);
    
    // 4. Controlliamo se una parola casuale ("albero") ha tutte le relazioni grazie all'enricher
    println!("\n>> Verifica copertura relazionale per parola test ('albero'):");
    let missing_for_albero = RelationType::ALL.iter()
        .filter(|&&rel| kg.query_objects("albero", rel).is_empty())
        .count();
    
    if missing_for_albero == 0 {
        println!("  'albero' ha almeno un arco per tutti i 21 tipi di relazione! (Arricchimento funzionante)");
    } else {
        println!("  'albero' manca di {} tipi di relazione.", missing_for_albero);
    }

    Ok(())
}
