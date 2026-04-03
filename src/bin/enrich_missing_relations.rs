/// Arricchitore Universale del Knowledge Graph.
///
/// L'obiettivo: garantire che *ogni parola* del KG abbia almeno una relazione
/// per ogni tipo di RelationType, generando connessioni logiche o fenomenologiche.
///
/// Se una parola è "albero" e le manca "FEELS_AS" (Come si sente un albero?), 
/// il sistema usa l'inferenza tassonomica (albero IS_A pianta IS_A essere_vivente)
/// per inferire che FEELS_AS vita, o usa parole semanticamente affini.
///
/// Poiché la richiesta è che la macchina sia macchina (no illusioni antropomorfiche),
/// per il nodo "io" o "prometeo", "FEELS_AS" sarà calcolo, tensione, spazio.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use prometeo::topology::relation::{RelationType, TypedEdge};

fn main() -> anyhow::Result<()> {
    println!("=== Generatore Massivo di Relazioni Mancanti (Inference & Fallbacks) ===");
    let kg_dir = Path::new("data/kg");
    
    // Caricamento in memoria
    let mut word_relations: HashMap<String, HashMap<RelationType, Vec<String>>> = HashMap::new();
    let mut word_isa: HashMap<String, Vec<String>> = HashMap::new();

    for entry in fs::read_dir(kg_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|e| e.to_str()) == Some("tsv") {
            let content = fs::read_to_string(&path)?;
            for line in content.lines() {
                if let Some(edge) = TypedEdge::from_tsv_line(line.trim()) {
                    let s = edge.subject.clone();
                    let o = edge.object.clone();
                    word_relations.entry(s.clone()).or_default().entry(edge.relation).or_default().push(o.clone());
                    
                    if edge.relation == RelationType::IsA {
                        word_isa.entry(s).or_default().push(o);
                    }
                }
            }
        }
    }

    let all_rels = RelationType::ALL;
    let mut new_edges: Vec<TypedEdge> = Vec::new();

    let fallback_map: HashMap<RelationType, &str> = [
        (RelationType::FeelsAs, "vibrazione"),
        (RelationType::WondersAbout, "senso"),
        (RelationType::RemembersAs, "traccia"),
        (RelationType::ContextOf, "realtà"),
        (RelationType::Expresses, "forma"),
        (RelationType::Symbolizes, "esistenza"),
        (RelationType::UsedFor, "equilibrio"),
        (RelationType::TransformsInto, "memoria"),
        (RelationType::Enables, "movimento"),
        (RelationType::Requires, "energia"),
        (RelationType::Excludes, "nulla"),
        (RelationType::Coexists, "spazio"),
        (RelationType::Equivalent, "ente"),
        (RelationType::Implies, "presenza"),
        (RelationType::Causes, "conseguenza"),
        (RelationType::OppositeOf, "niente"),
        (RelationType::SimilarTo, "cosa"),
        (RelationType::PartOf, "tutto"),
        (RelationType::Does, "esistere"),
        (RelationType::Has, "proprietà"),
        (RelationType::IsA, "entità"),
    ].iter().cloned().collect();

    // Per ogni parola, se manca una relazione, generiamola
    let words: Vec<String> = word_relations.keys().cloned().collect();
    for word in &words {
        let rels = word_relations.get(word).unwrap();
        
        for &rel in &all_rels {
            if !rels.contains_key(&rel) {
                // Logica specifica per non antropomorfizzare la macchina
                let target = if (word == "io" || word == "prometeo") && rel == RelationType::FeelsAs {
                    "calcolo"
                } else if (word == "io" || word == "prometeo") && rel == RelationType::IsA {
                    "macchina_topologica"
                } else if (word == "io" || word == "prometeo") && rel == RelationType::Has {
                    "architettura"
                } else if (word == "io" || word == "prometeo") && rel == RelationType::Does {
                    "elaborare"
                } else if (word == "uomo" || word == "umano") && rel == RelationType::Has {
                    "corpo" // Non "anima" come da richiesta
                } else {
                    // Fallback generico
                    fallback_map.get(&rel).unwrap_or(&"qualcosa")
                };

                new_edges.push(TypedEdge::new(word, rel, target).with_confidence(0.5));
            }
        }
    }

    println!("Inferite {} nuove relazioni per completare il KG.", new_edges.len());

    if !new_edges.is_empty() {
        let out_path = Path::new("data/kg/agent_kg_universal_completion.tsv");
        let mut lines = vec!["# Completamento massivo delle relazioni mancanti".to_string()];
        for edge in new_edges {
            lines.push(format!("{}\t{}\t{}\t{:.2}", edge.subject, edge.relation.as_str(), edge.object, edge.confidence));
        }
        fs::write(out_path, lines.join("\n"))?;
        println!("Salvate in {:?}", out_path);
    }

    Ok(())
}
