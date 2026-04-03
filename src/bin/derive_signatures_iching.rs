/// Script di Derivazione Firme 8D basato su I-Ching (Frattali)
///
/// Invece di propagare le firme sul grafo delle parole (che rischia di
/// far collassare tutto verso un polo dominante come "paura"),
/// questo script sfrutta le affinità frattali intrinseche di ogni parola.
///
/// Ogni parola nel Lexicon ha delle `fractal_affinities` verso i 64 esagrammi.
/// Ogni esagramma è composto da 2 trigrammi.
/// Ogni trigramma controlla una Dimensione specifica con un Valore specifico (Yin/Yang).
///
/// La firma 8D della parola diventa quindi la proiezione esatta delle sue
/// risonanze frattali, garantendo una diversità geometrica perfetta e coerente
/// con la logica fondativa di Prometeo.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use prometeo::topology::lexicon::Lexicon;
use prometeo::topology::fractal::{Trigram, HEXAGRAMS};

fn main() -> anyhow::Result<()> {
    println!("=== Derivazione Firme 8D via Logica I-Ching (Frattali) ===");

    // Inizializza il Lexicon base (senza phenomenology)
    let mut lexicon = Lexicon::bootstrap();
    
    // Mappa l'ID del frattale (0..63) alle sue dimensioni fisse derivate dai trigrammi
    // HEXAGRAMS ha indice = lower.index()*8 + upper.index()
    let mut fractal_signatures: HashMap<u32, [(usize, f64); 2]> = HashMap::new();
    
    for (i, (lower, upper, _name)) in HEXAGRAMS.iter().enumerate() {
        let dim_lower = lower.dim().index();
        let val_lower = lower.value();
        
        let dim_upper = upper.dim().index();
        let val_upper = upper.value();
        
        fractal_signatures.insert(i as u32, [(dim_lower, val_lower), (dim_upper, val_upper)]);
    }

    let out_path = Path::new("data/kg/phenomenology.tsv");
    let mut lines = vec![
        "# Firme Topologiche Calibrate (8D - Derivate da I-Ching)".to_string(),
        "# Dimensioni: Confine(4), Valenza(7), Intensita(2), Definizione(6), Complessita(5), Permanenza(1), Agency(0), Tempo(3)".to_string(),
        "# Parola\tSIG\tConfine,Valenza,Intensità,Definizione,Complessità,Permanenza,Agency,Tempo".to_string()
    ];

    let mut generated = 0;
    
    // Elenco delle parole per cui forziamo la sovrascrittura manuale per questioni di tuning
    // (come fatto in apply_curated_signatures(), ma qui le escludiamo dalla derivazione 
    // per non rovinare le 200 parole curate).
    let curated_words = vec!["io", "paura", "gioia", "luce", "vita", "morte", "macchina", "vuoto"];

    // Estraiamo tutte le parole e le loro affinità dal Lexicon
    // Non possiamo mutare e iterare contemporaneamente, quindi cloniamo i dati necessari
    let words_with_affinities: Vec<(String, HashMap<u32, f64>)> = lexicon.patterns_iter()
        .map(|(w, pat)| (w.to_string(), pat.fractal_affinities.clone()))
        .collect();

    for (word, affinities) in words_with_affinities {
        if curated_words.contains(&word.as_str()) { continue; }
        if affinities.is_empty() { continue; }
        if word.contains(' ') || word.len() < 3 { continue; }

        let mut sig_accum = [0.0_f64; 8];
        let mut sig_weight = [0.0_f64; 8];

        // Per ogni frattale a cui la parola è affine
        for (fid, affinity) in affinities {
            if let Some(constraints) = fractal_signatures.get(&fid) {
                // Il frattale fissa 2 dimensioni (trigramma inferiore e superiore)
                for (dim_idx, val) in constraints {
                    sig_accum[*dim_idx] += val * affinity;
                    sig_weight[*dim_idx] += affinity;
                }
            }
        }

        // Calcola la firma finale. Se una dimensione non è stata toccata da nessun frattale affine,
        // rimane a 0.5 (libera/neutra).
        let mut final_sig = [0.5_f64; 8];
        let mut is_flat = true;
        
        for d in 0..8 {
            if sig_weight[d] > 0.0 {
                final_sig[d] = sig_accum[d] / sig_weight[d];
                if (final_sig[d] - 0.5).abs() > 0.05 {
                    is_flat = false; // La parola ha una polarizzazione reale in almeno una dimensione
                }
            }
        }

        // Salviamo solo se la parola ha una vera forma geometrica derivata
        if !is_flat {
            // Mapping degli indici dell'I-Ching al formato standard 
            // Standard: [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo]
            // Indici Dim: Agency=0, Permanenza=1, Intensita=2, Tempo=3, Confine=4, Complessita=5, Definizione=6, Valenza=7
            let remapped_sig = [
                final_sig[4], // Confine
                final_sig[7], // Valenza
                final_sig[2], // Intensita
                final_sig[6], // Definizione
                final_sig[5], // Complessita
                final_sig[1], // Permanenza
                final_sig[0], // Agency
                final_sig[3], // Tempo
            ];

            let sig_str: Vec<String> = remapped_sig.iter().map(|v| format!("{:.2}", v)).collect();
            lines.push(format!("{}\tSIG\t{}", word, sig_str.join(",")));
            generated += 1;
        }
    }

    fs::write(out_path, lines.join("\n"))?;
    println!("Derivazione completata. {} firme fenomenologiche generate in base all'I-Ching.", generated);

    Ok(())
}
