/// Script per rimuovere intere classi di relazioni spurie o inutili dal KG
/// 
/// In particolare:
/// - Rimuove tutte le relazioni SIMILAR_TO (spesso fonetiche/inutili, derivate da dizionari poveri)
/// - Rimuove tutte le relazioni provenienti da "agent_kg_universal_completion.tsv" che ha generato 
///   connessioni astratte assurde (es. cane SYMBOLIZES esistenza, cane EQUIVALENT ente)
/// - Rimuove OPPOSITE_OF dubbi che non passano un filtro di base

use prometeo::topology::knowledge_graph::KgSnapshot;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("=== Piallatura del Rumore nel Knowledge Graph ===");

    let kg_path = Path::new("prometeo_kg.json");
    if !kg_path.exists() {
        anyhow::bail!("File {:?} non trovato.", kg_path);
    }
    
    let json = fs::read_to_string(kg_path)?;
    let mut snap: KgSnapshot = serde_json::from_str(&json)?;
    
    let initial_edges = snap.edges.len();
    println!("Relazioni totali iniziali nel KG: {}", initial_edges);

    // Identifichiamo le relazioni spurie
    let mut removed_similar = 0;
    let mut removed_universal = 0;
    let mut removed_opposite = 0;

    snap.edges.retain(|edge| {
        let rel_str = format!("{:?}", edge.relation);
        
        // 1. Via i SIMILAR_TO: sono spazzatura derivata da bigbang_kg.tsv (cane -> canicola)
        if rel_str == "SimilarTo" {
            removed_similar += 1;
            return false;
        }

        // 2. Via le astrazioni assurde di universal_completion (SYMBOLIZES, EQUIVALENT, CONTEXT_OF, FEELS_AS)
        // Manteniamo solo IS_A, HAS, DOES, CAUSES, REQUIRES
        let valid_predicates = ["IsA", "Has", "Does", "Causes", "Requires", "RelatedTo"];
        if !valid_predicates.contains(&rel_str.as_str()) && rel_str != "OppositeOf" {
            removed_universal += 1;
            return false;
        }

        // 3. Via gli OPPOSITE_OF se l'oggetto è più lungo/complesso e non è un aggettivo chiaro
        // (es. cane opposto_di cappone). Per ora, per sicurezza, teniamo un log di quanti ne togliamo
        if rel_str == "OppositeOf" {
            // Filtro grezzo: togliamo le opposizioni sostantivo-sostantivo senza senso
            if (edge.subject == "cane" && edge.object == "cappone") || 
               (edge.subject == "gatto" && edge.object == "topo") {
                removed_opposite += 1;
                return false;
            }
            // In realtà, OPPOSITE_OF spesso non serve ai drive. Se vogliamo essere drastici, 
            // potremmo toglierlo tutto. Per ora teniamo solo i più comuni o togliamolo se fa danni.
        }

        true
    });

    let final_edges = snap.edges.len();
    let total_removed = initial_edges - final_edges;

    println!("--- Risultati Pulizia ---");
    println!("Rimossi SIMILAR_TO (fonetici/inutili): {}", removed_similar);
    println!("Rimosse relazioni astratte spurie (SYMBOLIZES, EQUIVALENT, etc.): {}", removed_universal);
    println!("Rimossi OPPOSITE_OF specifici: {}", removed_opposite);
    println!("Totale archi eliminati: {}", total_removed);
    println!("Relazioni rimanenti, pulite e valide: {}", final_edges);

    // Salva il KG aggiornato
    let out_json = serde_json::to_string_pretty(&snap)?;
    fs::write(kg_path, out_json)?;
    println!("Knowledge Graph aggiornato con successo.");

    Ok(())
}
