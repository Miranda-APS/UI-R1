/// Diagnostica statistica delle firme 8D del lessico (Phase 69).
///
/// Legge `prometeo_topology_state.bin` direttamente e calcola:
///   1. Statistiche marginali per dimensione (mean, std, range)
///   2. Matrice di correlazione 8×8 (Pearson)
///   3. Coppie con |r| ≥ 0.5 (ridondanza forte)
///   4. Istogrammi di distribuzione per ogni dim
///   5. Coverage rispetto alla media (parole "tipiche")
///
/// Uso:
///     cargo run --release --bin signatures-stats
///
/// Output ASCII puro, niente dipendenze grafiche.

use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;
use std::path::Path;

const DIM_NAMES: [&str; 8] = [
    "potere", "materia", "ardore", "divenire",
    "spazio", "intreccio", "verita", "armonia",
];
const DIM_DESC: [&str; 8] = [
    "agency", "permanenza", "intensita", "tempo",
    "confine", "complessita", "definizione", "valenza",
];

fn main() {
    println!("Caricamento prometeo_topology_state.bin ...");
    let state = match PrometeoState::load_from_binary(Path::new("prometeo_topology_state.bin")) {
        Ok(s) => s,
        Err(e) => { eprintln!("Errore: {}", e); std::process::exit(2); }
    };
    let mut engine = PrometeoTopologyEngine::new();
    state.restore_lexicon(&mut engine);

    // Estrazione firme 8D
    let sigs: Vec<[f64; 8]> = engine.lexicon.patterns_iter()
        .map(|(_, p)| *p.signature.values())
        .collect();
    let n = sigs.len();
    if n == 0 { eprintln!("Nessuna firma."); std::process::exit(1); }

    println!("\n=== Analisi firme 8D — {} parole ===\n", n);

    // ---- 1) Stats marginali ----
    println!("[1/4] Statistiche per dimensione (valori 0-100)");
    println!("{:<2} {:<10} {:<14} {:>6} {:>6} {:>4} {:>4} {:>5}",
             "#", "nome", "desc", "mean", "std", "min", "max", "range");
    let mut means = [0.0; 8];
    let mut stds  = [0.0; 8];
    let mut mins  = [f64::INFINITY; 8];
    let mut maxs  = [f64::NEG_INFINITY; 8];
    for d in 0..8 {
        let vals: Vec<f64> = sigs.iter().map(|s| s[d] * 100.0).collect();
        let m: f64 = vals.iter().sum::<f64>() / n as f64;
        let v: f64 = vals.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n as f64;
        let s = v.sqrt();
        let lo = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        means[d] = m;
        stds[d]  = if s > 0.0 { s } else { 1.0 };
        mins[d]  = lo;
        maxs[d]  = hi;
        println!("{:<2} {:<10} {:<14} {:>6.1} {:>6.2} {:>4.0} {:>4.0} {:>5.0}",
                 d, DIM_NAMES[d], DIM_DESC[d], m, s, lo, hi, hi - lo);
    }

    let low_std: Vec<usize> = (0..8).filter(|&d| stds[d] < 18.0).collect();
    if !low_std.is_empty() {
        println!("\nWARN  dimensioni con std<18 (varianza schiacciata):");
        for &d in &low_std {
            println!("  - {} ({}) std={:.2}", DIM_NAMES[d], DIM_DESC[d], stds[d]);
        }
    } else {
        println!("\nOK   tutte le dim hanno std ≥ 18 (varianza sana).");
    }

    // ---- 2) Matrice correlazione ----
    println!("\n[2/4] Matrice di correlazione 8x8 (Pearson)");
    let mut corr = [[0.0f64; 8]; 8];
    for i in 0..8 {
        for j in 0..8 {
            let cov: f64 = sigs.iter()
                .map(|s| (s[i] * 100.0 - means[i]) * (s[j] * 100.0 - means[j]))
                .sum::<f64>() / n as f64;
            corr[i][j] = cov / (stds[i] * stds[j]);
        }
    }
    print!("       ");
    for n_ in DIM_NAMES.iter() { print!("{:>6.5}", n_); }
    println!();
    for i in 0..8 {
        print!("{:<6} ", DIM_NAMES[i]);
        for j in 0..8 { print!(" {:+.2}", corr[i][j]); }
        println!();
    }

    let high: f64 = 0.50;
    let med:  f64 = 0.30;
    let mut high_pairs = Vec::new();
    let mut med_pairs  = Vec::new();
    for i in 0..8 {
        for j in (i+1)..8 {
            let r = corr[i][j];
            if r.abs() >= high { high_pairs.push((i, j, r)); }
            else if r.abs() >= med { med_pairs.push((i, j, r)); }
        }
    }
    if !high_pairs.is_empty() {
        println!("\nWARN  Correlazioni FORTI |r| >= {} (ridondanza, una dim spiega l'altra):", high);
        let mut sorted = high_pairs.clone();
        sorted.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap());
        for (i, j, r) in sorted {
            println!("  {:<10} <-> {:<10} r={:+.3}", DIM_NAMES[i], DIM_NAMES[j], r);
        }
    }
    if !med_pairs.is_empty() {
        println!("\nINFO  Correlazioni medie {} <= |r| < {}:", med, high);
        let mut sorted = med_pairs.clone();
        sorted.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap());
        for (i, j, r) in sorted {
            println!("  {:<10} <-> {:<10} r={:+.3}", DIM_NAMES[i], DIM_NAMES[j], r);
        }
    }
    if high_pairs.is_empty() && med_pairs.is_empty() {
        println!("\nOK   Nessuna correlazione |r| >= {} — firme ben decorrelate.", med);
    }

    // ---- 3) Istogrammi ----
    println!("\n[3/4] Distribuzione per dim (10 bin, 0-100)");
    println!("           00-10 10-20 20-30 30-40 40-50 50-60 60-70 70-80 80-90 90-100");
    for d in 0..8 {
        let mut hist = [0usize; 10];
        for s in &sigs {
            let v = (s[d] * 100.0).clamp(0.0, 99.99);
            let b = (v / 10.0) as usize;
            hist[b] += 1;
        }
        print!("{:<10} ", DIM_NAMES[d]);
        for c in &hist { print!("{:>6}", c); }
        println!();
    }

    // ---- 4) Coverage ----
    println!("\n[4/4] Coverage: parole vicine alla media 8D");
    for tol in [10.0, 20.0, 30.0] {
        let mut count = 0;
        for s in &sigs {
            let d2: f64 = (0..8).map(|k| (s[k] * 100.0 - means[k]).powi(2)).sum::<f64>().sqrt();
            if d2 < tol { count += 1; }
        }
        let pct = 100.0 * count as f64 / n as f64;
        println!("  entro distanza euclidea {:>3.0}: {:>6} parole ({:.1}%)", tol, count, pct);
    }

    // ---- Verdetto ----
    println!("\n=== Verdetto ===");
    let mut issues = Vec::new();
    if !low_std.is_empty() { issues.push(format!("{} dim con varianza bassa", low_std.len())); }
    if !high_pairs.is_empty() { issues.push(format!("{} coppie fortemente correlate", high_pairs.len())); }
    if issues.is_empty() {
        println!("Le firme sono in buona salute: varianza ampia, dim decorrelate.");
    } else {
        println!("Le firme hanno questi problemi strutturali:");
        for i in &issues { println!("  - {}", i); }
        println!("\nProssimi passi:");
        if !high_pairs.is_empty() {
            println!("  1. ricalibrare derive_8d_from_kg per separare le coppie correlate");
            println!("  2. eseguire `cargo run --release --bin rederive-signatures`");
        }
        if !low_std.is_empty() {
            println!("  3. arricchire il KG sulle dim con varianza bassa");
        }
    }
}
