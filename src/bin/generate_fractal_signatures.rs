/// Genera le 64 firme canoniche dei frattali I Ching come ancore di
/// propagazione (Phase 70).
///
/// Usa l'API esistente: HEXAGRAMS + Trigram::dim()/value() in fractal.rs.
/// Per ogni esagramma:
///   - le 2 dimensioni associate ai trigrammi (lower, upper) ricevono il
///     valore canonico (0.10/0.30/0.70/0.90 secondo il Yang-content)
///   - le 6 dimensioni rimanenti restano a 0.50 (neutro)
///
/// Output: data/anchors/fractal_signatures.tsv
///
/// Uso:
///     cargo run --release --bin generate-fractal-signatures

use prometeo::topology::fractal::{HEXAGRAMS, Trigram};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const NEUTRAL: f64 = 0.50;
const OUT_PATH: &str = "data/anchors/fractal_signatures.tsv";

fn fractal_signature(lower: Trigram, upper: Trigram) -> [f64; 8] {
    let mut sig = [NEUTRAL; 8];
    sig[lower.dim().index()] = lower.value();
    // Se lower e upper hanno la stessa dim (esagramma puro) il valore è
    // identico — idempotente, niente conflitto.
    sig[upper.dim().index()] = upper.value();
    sig
}

fn main() {
    if let Some(parent) = Path::new(OUT_PATH).parent() {
        fs::create_dir_all(parent).expect("create_dir_all");
    }
    let mut f = File::create(OUT_PATH).expect("create file");

    writeln!(f, "# Firme canoniche dei 64 frattali I Ching (Phase 70).").unwrap();
    writeln!(f, "# Generato da: cargo run --release --bin generate-fractal-signatures").unwrap();
    writeln!(f, "#").unwrap();
    writeln!(f, "# ID = lower.index() * 8 + upper.index()").unwrap();
    writeln!(f, "# Ogni trigramma fissa la dim associata al suo valore canonico Yang:").unwrap();
    writeln!(f, "#   ☰ Cielo=0.90 (Agency)    ☷ Terra=0.10 (Permanenza)").unwrap();
    writeln!(f, "#   ☳ Tuono=0.30 (Intensità) ☵ Acqua=0.30 (Tempo)").unwrap();
    writeln!(f, "#   ☶ Montagna=0.30 (Confine) ☴ Vento=0.70 (Complessità)").unwrap();
    writeln!(f, "#   ☲ Fuoco=0.70 (Definizione) ☱ Lago=0.70 (Valenza)").unwrap();
    writeln!(f, "# Le dim non controllate dai due trigrammi restano a 0.50 (neutro).").unwrap();
    writeln!(f, "#").unwrap();
    writeln!(f, "# Header dim: agency=0 permanenza=1 intensita=2 tempo=3").unwrap();
    writeln!(f, "#             confine=4 complessita=5 definizione=6 valenza=7").unwrap();
    writeln!(f, "id\tlower\tupper\tname\tagency\tpermanenza\tintensita\ttempo\tconfine\tcomplessita\tdefinizione\tvalenza").unwrap();

    for (idx, (lower, upper, name)) in HEXAGRAMS.iter().enumerate() {
        let sig = fractal_signature(*lower, *upper);
        let id = lower.index() * 8 + upper.index();
        debug_assert_eq!(id, idx, "ordine HEXAGRAMS non sequenziale");
        writeln!(f,
            "{}\t{}{}\t{}{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
            id,
            lower.symbol(), trigram_short(*lower),
            upper.symbol(), trigram_short(*upper),
            name,
            sig[0], sig[1], sig[2], sig[3], sig[4], sig[5], sig[6], sig[7]
        ).unwrap();
    }

    println!("Generato {} con 64 firme canoniche.", OUT_PATH);
    println!();
    println!("Esempi di lettura:");
    println!("  ID 0  ☰☰ POTERE      → agency=0.90, resto neutro 0.50");
    println!("  ID 9  ☷☷ MATERIA     → permanenza=0.10, resto neutro");
    println!("  ID 36 ☶☶ SPAZIO      → confine=0.30, resto neutro");
    println!("  ID 63 ☱☱ ARMONIA     → valenza=0.70, resto neutro");
    println!("  ID 1  ☰☷ CREAZIONE   → agency=0.90, permanenza=0.10, resto neutro");
}

fn trigram_short(t: Trigram) -> &'static str {
    match t {
        Trigram::Cielo => "Cielo", Trigram::Terra => "Terra",
        Trigram::Tuono => "Tuono", Trigram::Acqua => "Acqua",
        Trigram::Montagna => "Mont.", Trigram::Vento => "Vento",
        Trigram::Fuoco => "Fuoco", Trigram::Lago => "Lago",
    }
}
