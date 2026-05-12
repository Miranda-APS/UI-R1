/// Verifica della distribuzione delle firme 8D.
/// Controlla se le parole chiave e le loro propagazioni hanno 
/// firme polarizzate anziché piatte (attorno allo 0.5).

use prometeo::topology::lexicon::Lexicon;

fn print_signature(word: &str, sig: &[f64; 8]) {
    println!("{:<15}: [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}]",
        word, sig[0], sig[1], sig[2], sig[3], sig[4], sig[5], sig[6], sig[7]
    );
}

fn main() {
    println!("=== Verifica Polarizzazione Firme 8D nel Lexicon ===");
    println!("Dimensioni (I Ching canonico): [Agency, Permanenza, Intensità, Tempo, Confine, Complessità, Definizione, Valenza]\n");

    let mut lexicon = Lexicon::bootstrap();
    lexicon.load_phenomenology_signatures();
    
    let words_to_check = vec![
        // Poli Hardcoded
        "io", "macchina", "vuoto", "paura", "calma", "calcolo",
        // Parole che dovrebbero essere state propagate via IS_A o SIMILAR_TO
        "angoscia", "terrore", "tristezza", "intelligenza", "software",
        "silenzio", "abisso", "pace", "quiete"
    ];

    let mut found = 0;
    for word in &words_to_check {
        if let Some(pat) = lexicon.get(*word) {
            print_signature(word, pat.signature.values());
            found += 1;
        } else {
            println!("{:<15}: (Non trovato nel Lexicon corrente)", word);
        }
    }

    println!("\nParole verificate: {}/{}", found, words_to_check.len());
}
