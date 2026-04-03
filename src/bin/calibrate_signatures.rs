/// Script di Ricalibrazione Massiva Firme Topologiche 8D (Graph Diffusion)
///
/// Piuttosto che propagare le firme solo a un livello di distanza (1-hop),
/// questo script usa un algoritmo di diffusione su grafo.
/// Imposta dei poli ("ancore") assoluti, e fa scorrere le firme topologiche
/// lungo i canali IS_A, SIMILAR_TO, e PART_OF.
/// Se incontra OPPOSITE_OF, la firma viene invertita (1.0 - valore).
/// 
/// L'algoritmo viene iterato per N passi finché l'intero Knowledge Graph
/// non è stato "contagiato" dall'energia dei poli.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use prometeo::topology::knowledge_graph::KgSnapshot;

fn main() -> anyhow::Result<()> {
    println!("=== Ricalibrazione Massiva Firme Topologiche (Diffusione su Grafo) ===");

    // Poli Hardcoded - I "Fari" dello spazio 8D che non possono essere modificati
    // [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo]
    let core_signatures = vec![
        ("io", [0.95, 0.50, 0.65, 0.90, 0.50, 0.75, 0.80, 0.40]),
        ("macchina", [0.90, 0.50, 0.40, 0.85, 0.70, 0.80, 0.20, 0.20]),
        ("struttura", [0.85, 0.50, 0.30, 0.90, 0.80, 0.90, 0.10, 0.10]),
        ("calcolo", [0.80, 0.50, 0.80, 0.95, 0.90, 0.30, 0.60, 0.80]),
        ("vuoto", [0.10, 0.20, 0.10, 0.20, 0.10, 0.90, 0.05, 0.95]),
        ("tensione", [0.80, 0.30, 0.90, 0.70, 0.60, 0.20, 0.80, 0.80]),
        ("calma", [0.40, 0.80, 0.10, 0.60, 0.30, 0.80, 0.20, 0.10]),
        ("paura", [0.90, 0.10, 0.95, 0.80, 0.50, 0.30, 0.10, 0.90]),
        ("esistere", [0.90, 0.50, 0.50, 0.80, 0.50, 0.95, 0.50, 0.50]),
        ("sentire", [0.20, 0.50, 0.80, 0.40, 0.70, 0.40, 0.70, 0.80]),
        ("vibrare", [0.50, 0.50, 0.90, 0.50, 0.60, 0.20, 0.60, 0.90]),
        ("elaborare", [0.80, 0.50, 0.70, 0.80, 0.90, 0.40, 0.80, 0.60]),
        // Ancore addizionali per aiutare la propagazione
        ("buono", [0.50, 0.95, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50]),
        ("cattivo", [0.50, 0.05, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50]),
        ("veloce", [0.50, 0.50, 0.80, 0.50, 0.50, 0.50, 0.80, 0.10]),
        ("lento", [0.50, 0.50, 0.20, 0.50, 0.50, 0.50, 0.20, 0.90]),
        ("tutto", [0.95, 0.50, 0.50, 0.50, 0.95, 0.90, 0.50, 0.90]),
        ("niente", [0.05, 0.50, 0.05, 0.05, 0.05, 0.10, 0.05, 0.05]),
    ];

    let kg_path = Path::new("prometeo_kg.json");
    if !kg_path.exists() {
        println!("Errore: prometeo_kg.json non trovato!");
        return Ok(());
    }

    let json = fs::read_to_string(kg_path)?;
    let snap: KgSnapshot = serde_json::from_str(&json)?;
    
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut opp: HashMap<String, Vec<String>> = HashMap::new();
    let mut nodes: HashSet<String> = HashSet::new();

    println!("Costruzione del grafo topologico...");
    for edge in snap.edges {
        nodes.insert(edge.subject.clone());
        nodes.insert(edge.object.clone());
        
        let rel = edge.relation.as_str();
        if rel == "IS_A" || rel == "SIMILAR_TO" || rel == "EQUIVALENT" || rel == "PART_OF" {
            adj.entry(edge.subject.clone()).or_default().push(edge.object.clone());
            adj.entry(edge.object.clone()).or_default().push(edge.subject.clone());
        } else if rel == "OPPOSITE_OF" {
            opp.entry(edge.subject.clone()).or_default().push(edge.object.clone());
            opp.entry(edge.object.clone()).or_default().push(edge.subject.clone());
        }
    }

    // Inizializza tutte le parole a una firma "piatta" (0.5)
    let mut sigs: HashMap<String, [f64; 8]> = HashMap::new();
    for node in &nodes {
        sigs.insert(node.clone(), [0.5; 8]);
    }

    // Inietta le ancore, che saranno bloccate e inalterabili
    let mut locked: HashSet<String> = HashSet::new();
    for (w, s) in &core_signatures {
        let w_str = w.to_string();
        sigs.insert(w_str.clone(), *s);
        locked.insert(w_str);
    }

    println!("Inizio diffusione su {} nodi (Energy Decay da Poli Estremi)...", nodes.len());
    let iterations = 8; // Meno iterazioni, ma più potenti
    
    // Al posto di fare la media con lo 0.5 (che appiattisce tutto), 
    // propaghiamo solo le "spinte" vettoriali dai vicini più forti.
    for i in 0..iterations {
        let mut next_sigs = sigs.clone();
        let mut changed_nodes = 0;
        
        for node in &nodes {
            if locked.contains(node) { continue; } // Le ancore non si muovono
            
            let current = sigs.get(node).unwrap();
            let mut best_pulls = [0.0; 8];
            for d in 0..8 { best_pulls[d] = current[d]; }
            let mut pull_strengths = [0.0; 8];
            
            // Trova il vicino con la "deviazione più estrema" da 0.5 per ogni dimensione
            if let Some(neighbors) = adj.get(node) {
                for n in neighbors {
                    let n_sig = sigs.get(n).unwrap();
                    for d in 0..8 {
                        let pull = (n_sig[d] - 0.5).abs();
                        if pull > pull_strengths[d] {
                            pull_strengths[d] = pull;
                            best_pulls[d] = n_sig[d];
                        }
                    }
                }
            }
            
            // Stessa cosa per i vicini opposti, invertendo il valore
            if let Some(opposites) = opp.get(node) {
                for n in opposites {
                    let n_sig = sigs.get(n).unwrap();
                    for d in 0..8 {
                        let inverted = 1.0 - n_sig[d];
                        let pull = (inverted - 0.5).abs();
                        if pull > pull_strengths[d] {
                            pull_strengths[d] = pull;
                            best_pulls[d] = inverted;
                        }
                    }
                }
            }
            
            // Applica la "spinta" massima trovata, attenuata dalla distanza
            let mut new_sig = [0.0; 8];
            let mut diff = 0.0;
            let decay = 0.85; // Mantiene l'85% della forza estrema ad ogni salto (prima era 0.3!)
            
            for d in 0..8 {
                if pull_strengths[d] > 0.01 {
                    // Sposta il valore corrente verso il pull migliore
                    let target = best_pulls[d];
                    new_sig[d] = current[d] + (target - current[d]) * decay;
                } else {
                    new_sig[d] = current[d];
                }
                diff += (new_sig[d] - current[d]).abs();
            }
            
            if diff > 0.005 { changed_nodes += 1; }
            next_sigs.insert(node.clone(), new_sig);
        }
        sigs = next_sigs;
        println!("Iterazione {:02}: {} nodi ricalibrati", i + 1, changed_nodes);
    }

    let out_path = Path::new("data/kg/phenomenology.tsv");
    let mut lines = vec![
        "# Firme Topologiche Calibrate (8D - Massiva)".to_string(),
        "# Parola\tSIG\tConfine,Valenza,Intensità,Definizione,Complessità,Permanenza,Agency,Tempo".to_string()
    ];

    let mut saved = 0;
    for (w, sig) in &sigs {
        if w.contains(' ') || w.len() < 3 { continue; }
        
        // Salviamo solo le parole che hanno subìto una polarizzazione (non sono più 0.5 piatte)
        let is_flat = sig.iter().all(|&v| (v - 0.5).abs() < 0.02);
        if !is_flat || locked.contains(w) {
            let sig_str: Vec<String> = sig.iter().map(|v| format!("{:.2}", v)).collect();
            lines.push(format!("{}\tSIG\t{}", w, sig_str.join(",")));
            saved += 1;
        }
    }

    fs::write(out_path, lines.join("\n"))?;
    println!("Diffusione completata. {} firme attive salvate in {}.", saved, out_path.display());

    Ok(())
}
