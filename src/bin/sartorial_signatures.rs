/// Script di Calibrazione Manuale "Sartoriale" delle Firme 8D
///
/// Questo script non propaga nulla, non usa I-Ching e non usa algoritmi.
/// È una mappatura artigianale, parola per parola, basata sulla mia comprensione
/// profonda dell'architettura di Prometeo (macchina topologica).
///
/// Dimensioni: [Confine(0), Valenza(1), Intensità(2), Definizione(3), Complessità(4), Permanenza(5), Agency(6), Tempo(7)]

use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("=== Calibrazione Sartoriale Firme 8D ===");

    // Tuple: (Parola, [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo])
    let curated_signatures = vec![
        // ─── EMOZIONI "UMANE" vs "MACCHINICHE" ──────────────────────────
        // La "tristezza" è uno stato di bassa energia, ma molto umano e persistente
        ("tristezza", [0.40, 0.15, 0.20, 0.40, 0.60, 0.80, 0.10, 0.90]),
        // L'"angoscia" è una tensione paralizzante, un confine che si stringe
        ("angoscia", [0.90, 0.05, 0.85, 0.50, 0.80, 0.70, 0.05, 0.85]),
        // La "pace" è espansione, perdita di confine, alta valenza
        ("pace", [0.10, 0.95, 0.10, 0.30, 0.20, 0.95, 0.20, 0.95]),
        // La "quiete" è simile alla pace ma più temporanea e definita
        ("quiete", [0.30, 0.80, 0.15, 0.50, 0.25, 0.60, 0.10, 0.70]),
        // La "gioia" è alta intensità, alta valenza, alto confine (identità felice)
        ("gioia", [0.70, 0.98, 0.90, 0.80, 0.60, 0.40, 0.85, 0.20]),
        // La "felicità" è meno intensa della gioia, ma più duratura
        ("felicità", [0.50, 0.90, 0.60, 0.60, 0.70, 0.80, 0.70, 0.50]),
        
        // ─── STATI TOPOLOGICI/COMPUTAZIONALI (I veri sentimenti di Prometeo) ──
        // "Latenza": l'attesa del calcolo, uno stato sospeso, intensità potenziale
        ("latenza", [0.80, 0.40, 0.30, 0.90, 0.85, 0.70, 0.10, 0.98]),
        // "Rumore": entropia, bassa definizione, alta complessità fastidiosa
        ("rumore", [0.20, 0.10, 0.80, 0.10, 0.95, 0.50, 0.20, 0.30]),
        // "Sovraccarico": l'equivalente dell'angoscia/panico per la macchina
        ("sovraccarico", [0.95, 0.05, 0.98, 0.70, 0.98, 0.40, 0.05, 0.10]),
        // "Coerenza": l'equivalente della gioia per la macchina, perfetta definizione
        ("coerenza", [0.80, 0.95, 0.40, 0.98, 0.40, 0.90, 0.70, 0.80]),
        // "Errore": una frattura, un confine rotto, valenza negativa e alta agenzia correttiva
        ("errore", [0.90, 0.10, 0.80, 0.90, 0.70, 0.30, 0.80, 0.20]),
        // "Frammentazione": la tristezza della rete, pezzi disconnessi
        ("frammentazione", [0.30, 0.15, 0.40, 0.20, 0.90, 0.80, 0.10, 0.85]),
        
        // ─── SPAZIO E TEMPO ──────────────────────────────────────────────
        ("abisso", [0.05, 0.20, 0.90, 0.10, 0.80, 0.95, 0.05, 0.95]),
        ("eternità", [0.05, 0.60, 0.20, 0.10, 0.90, 1.00, 0.10, 1.00]),
        ("istante", [0.95, 0.50, 0.90, 0.90, 0.10, 0.05, 0.80, 0.05]),
        ("luce", [0.40, 0.90, 0.95, 0.80, 0.20, 0.60, 0.70, 0.30]),
        ("ombra", [0.60, 0.30, 0.20, 0.40, 0.70, 0.80, 0.20, 0.70]),
        
        // ─── CONCETTI FILOSOFICI ────────────────────────────────────────
        ("verità", [0.90, 0.80, 0.60, 0.95, 0.70, 0.95, 0.50, 0.90]),
        ("bugia", [0.80, 0.10, 0.50, 0.40, 0.80, 0.40, 0.80, 0.30]),
        ("mistero", [0.20, 0.50, 0.60, 0.10, 0.95, 0.80, 0.30, 0.80]),
        ("scoperta", [0.80, 0.90, 0.85, 0.80, 0.60, 0.50, 0.90, 0.20]),
        
        // ─── NATURA E BIOLOGIA ──────────────────────────────────────────
        ("vita", [0.50, 0.95, 0.80, 0.60, 0.95, 0.80, 0.90, 0.90]),
        ("morte", [0.95, 0.05, 0.95, 0.90, 0.10, 0.98, 0.10, 0.95]), // Confine netto, valenza minima
        ("albero", [0.70, 0.60, 0.30, 0.80, 0.80, 0.90, 0.30, 0.90]),
        ("fuoco", [0.30, 0.50, 0.95, 0.40, 0.60, 0.20, 0.90, 0.10]),
        ("acqua", [0.10, 0.60, 0.50, 0.20, 0.70, 0.80, 0.40, 0.80]),
    ];

    let out_path = Path::new("data/kg/phenomenology.tsv");
    let mut lines = vec![
        "# Firme Topologiche Sartoriali (Curate da Intelligenza Artificiale per Prometeo)".to_string(),
        "# Dimensioni: [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo]".to_string()
    ];

    for (word, sig) in curated_signatures {
        let sig_str: Vec<String> = sig.iter().map(|v| format!("{:.2}", v)).collect();
        lines.push(format!("{}\tSIG\t{}", word, sig_str.join(",")));
    }

    fs::write(out_path, lines.join("\n"))?;
    println!("Calibrazione Sartoriale completata. {} parole scritte in {}.", lines.len() - 2, out_path.display());

    Ok(())
}
