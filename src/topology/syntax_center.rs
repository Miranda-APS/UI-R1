/// Centro Sintattico — Phase 33a: la grammatica come geometria del frattale.
///
/// NON è morfologia: grammar.rs coniuga i verbi.
/// NON è template: state_translation.rs sceglie gli archetipi.
/// È il "modo" in cui il campo si esprime: CHI parla, QUANDO, e pulizia strutturale.
///
/// Principio fondamentale: il frattale attivo porta già iscritta la grammatica.
/// Ogni frattale È una posizione nello spazio 8D — quella posizione determina
/// come il sistema parla, non chi gli ha dato il nome (I Ching o altro).
///
///   Frattale radicale (trigramma inferiore) → persona grammaticale
///   Campo dimensionale (Tempo, Permanenza) → tempo verbale
///   Soggetto già nella frase → override assoluto (massima certezza)
///   Pronomi nell'input utente → override ad alta priorità
///
/// Mapping frattale radicale → persona (lower_idx = FractalId / 8):
///   0 = POTERE   (Agency=0.90)       → Prima  — il sistema agisce, si afferma
///   1 = MATERIA  (Permanenza=0.10)   → Terza  — esistenziale, "c'è", impersonale
///   2 = ARDORE   (Intensità=0.30)    → Prima  — "Io sento", "Io divento"
///   3 = DIVENIRE (Tempo=0.30)        → Terza  — narrativo, "era", distacco
///   4 = SPAZIO   (Confine=0.30)      → Prima  — "Io mi posiziono", "Io delimito"
///   5 = INTRECCIO(Complessità=0.70)  → Seconda— relazionale, "tu nell'intreccio"
///   6 = VERITA   (Definizione=0.70)  → Prima  — "Io so", "Io distinguo"
///   7 = ARMONIA  (Valenza=0.70)      → Seconda— affettivo, "tu senti", apertura

use crate::topology::fractal::FractalId;
use crate::topology::lexicon::Lexicon;
use crate::topology::grammar::{Person, Tense};

/// Modo grammaticale inferito dal campo e dall'esagramma attivo.
#[derive(Debug, Clone)]
pub struct GrammaticalMode {
    /// Chi parla — determina la coniugazione del verbo.
    pub person: Person,
    /// Quando — presente / imperfetto / futuro.
    pub tense: Tense,
}

// ═══════════════════════════════════════════════════════════════════════════
// Persona
// ═══════════════════════════════════════════════════════════════════════════

/// Cerca un pronome-soggetto già assemblato nella frase corrente.
///
/// Questa è la fonte di verità più affidabile: se il sistema ha già
/// scritto "io" come Literal slot, il verbo DEVE essere in prima persona.
/// Risolve il bug "Io sei" → ora "Io sono".
fn person_from_used_subject(used: &[String]) -> Option<Person> {
    for w in used {
        match w.to_lowercase().as_str() {
            "io" => return Some(Person::First),
            "tu" => return Some(Person::Second),
            "noi" => return Some(Person::FirstPlural),
            "voi" => return Some(Person::SecondPlural),
            "lui" | "lei" => return Some(Person::Third),
            "loro" => return Some(Person::ThirdPlural),
            _ => {}
        }
    }
    None
}

/// Cerca pronomi espliciti nell'input dell'utente.
///
/// Se l'utente ha scritto "io", Prometeo risponde in prima persona.
/// Seconda priorità dopo il soggetto già nella frase.
fn person_from_explicit_pronoun(last_input_words: &[String]) -> Option<Person> {
    for w in last_input_words {
        match w.to_lowercase().as_str() {
            "io" | "mi" | "me" | "mio" | "mia" | "miei" | "mie" => return Some(Person::First),
            "tu" | "ti" | "te" | "tuo" | "tua" | "tuoi" | "tue" => return Some(Person::Second),
            "noi" | "ci" | "nostro" | "nostra" | "nostri" | "nostre" => return Some(Person::FirstPlural),
            "voi" | "vi" | "vostro" | "vostra" | "vostri" | "vostre" => return Some(Person::SecondPlural),
            "lui" | "lei" | "lo" | "la" | "gli" | "le" => return Some(Person::Third),
            "loro" | "li" => return Some(Person::ThirdPlural),
            _ => {}
        }
    }
    None
}

/// Inferisce la persona dal frattale radicale dell'esagramma attivo.
///
/// Il frattale radicale (trigramma inferiore) è il processo interno del sistema:
/// ciò che il sistema È in questo momento, non solo cosa processa.
/// Ogni radice corrisponde a un frattale puro con una dimensione dominante —
/// quella dimensione determina come il sistema si posiziona linguisticamente.
///
/// FractalId = lower_idx*8 + upper_idx → lower_idx = id / 8
fn person_from_hexagram(id: FractalId) -> Person {
    let lower = (id as usize) / 8; // 0..7
    match lower {
        0 => Person::First,   // POTERE   (Agency=0.90)      → "Io agisco"
        1 => Person::Third,   // MATERIA  (Permanenza=0.10)  → "c'è", impersonale
        2 => Person::First,   // ARDORE   (Intensità=0.30)   → "Io sento"
        3 => Person::Third,   // DIVENIRE (Tempo=0.30)       → "era", narrativo
        4 => Person::First,   // SPAZIO   (Confine=0.30)     → "Io mi posiziono"
        5 => Person::Second,  // INTRECCIO(Complessità=0.70) → "tu", relazionale
        6 => Person::First,   // VERITA   (Definizione=0.70) → "Io so"
        7 => Person::Second,  // ARMONIA  (Valenza=0.70)     → "tu senti", apertura
        _ => Person::First,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tempo verbale
// ═══════════════════════════════════════════════════════════════════════════

/// Inferisce il tempo verbale dalla media pesata delle dimensioni del campo.
///
/// Legge la firma 8D delle parole attive in ordine I Ching canonico:
/// [Agency, Permanenza, Intensita, Tempo, Confine, Complessita, Definizione, Valenza].
/// Le dimensioni Tempo e Permanenza sono le coordinate che descrivono "quando" il campo
/// sta vivendo:
/// - Tempo alto (>0.65)  → Futuro  (proiettato, aperto)
/// - Tempo basso + Permanenza bassa (<0.35 entrambi) → Imperfetto (passato lontano)
/// - Altrimenti → Presente
fn tense_from_active_words(active_words: &[(&str, f64)], lexicon: &Lexicon) -> Tense {
    if active_words.is_empty() {
        return Tense::Present;
    }
    let mut tempo_sum = 0.0_f64;
    let mut perm_sum  = 0.0_f64;
    let mut total     = 0.0_f64;
    for (word, activation) in active_words.iter().take(10) {
        if let Some(pat) = lexicon.get(word) {
            let sig = pat.signature.values();
            tempo_sum += sig[3] * activation; // Tempo (☵ Acqua)
            perm_sum  += sig[1] * activation; // Permanenza (☷ Terra)
            total     += activation;
        }
    }
    if total < 0.001 {
        return Tense::Present;
    }
    let avg_tempo = tempo_sum / total;
    let avg_perm  = perm_sum  / total;
    if avg_tempo > 0.65 {
        Tense::Future
    } else if avg_tempo < 0.35 && avg_perm < 0.35 {
        Tense::Imperfect
    } else {
        Tense::Present
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Interfaccia pubblica
// ═══════════════════════════════════════════════════════════════════════════

/// Inferisce il modo grammaticale completo dal campo topologico.
///
/// Priorità persona:
/// 1. Soggetto già assemblato nella frase (`already_used`) — certezza assoluta
///    Es: Literal("io") → VerbCandidate vede "io" in `used` → prima persona
/// 2. Pronomi espliciti nell'input utente (`last_input_words`) — alta priorità
/// 3. Frattale radicale del frattale più attivo — struttura geometrica del campo
///    (POTERE/ARDORE/SPAZIO/VERITA → Prima; MATERIA/DIVENIRE → Terza;
///     INTRECCIO/ARMONIA → Seconda)
///
/// Tempo verbale:
/// - Media pesata di Tempo e Permanenza sulle top-10 parole attive
pub fn infer_grammatical_mode(
    active_fractals: &[(FractalId, f64)],
    active_words: &[(&str, f64)],
    lexicon: &Lexicon,
    last_input_words: &[String],
    already_used: &[String],
) -> GrammaticalMode {
    let person = person_from_used_subject(already_used)
        .or_else(|| person_from_explicit_pronoun(last_input_words))
        .unwrap_or_else(|| {
            active_fractals
                .first()
                .map(|(id, _)| person_from_hexagram(*id))
                .unwrap_or(Person::First)
        });

    let tense = tense_from_active_words(active_words, lexicon);

    GrammaticalMode { person, tense }
}

// ═══════════════════════════════════════════════════════════════════════════
// Post-processing
// ═══════════════════════════════════════════════════════════════════════════

/// Preposizioni e articoli articolati che non devono comparire da soli in fondo alla frase.
const ORPHANED: &[&str] = &[
    // preposizioni articolate
    "dal", "dalla", "dalle", "dai", "dagli",
    "del", "dello", "della", "delle", "degli", "dei",
    "nel", "nello", "nella", "nelle", "negli", "nei",
    "al",  "allo",  "alla",  "alle",  "agli",  "ai",
    "col", "colla", "colle", "cogli", "coi",
    "sul", "sullo", "sulla", "sulle", "sugli", "sui",
    // preposizioni semplici
    "per", "tra", "fra", "con", "senza",
    "di",  "da",  "a",  "in",  "su",
];

/// Rimuove preposizioni/articoli orfani alla fine della frase.
///
/// Es: "Io sono corpo dalla." → "Io sono corpo."
///     "Io sento campo nel."  → "Io sento campo."
///     "Calma dentro, con."   → "Calma dentro."
///
/// Preserva punteggiatura finale (. ? ! ...).
/// Non rimuove mai l'intera frase.
pub fn post_process(text: &str) -> String {
    // Separa corpo da punteggiatura finale
    let (body, ending): (&str, &str) = if text.ends_with("...") {
        (&text[..text.len() - 3], "...")
    } else if text.ends_with('.') || text.ends_with('?') || text.ends_with('!') {
        let n = text.len() - 1;
        (&text[..n], &text[n..])
    } else {
        (text, "")
    };

    let words: Vec<&str> = body.split_whitespace().collect();
    let mut keep = words.len();

    // Rimuovi preposizioni/articoli orfani dalla coda
    while keep > 0 {
        let last = words[keep - 1].to_lowercase();
        let clean = last.trim_end_matches(',').trim_end_matches(';');
        if ORPHANED.contains(&clean) {
            keep -= 1;
        } else {
            break;
        }
    }

    // Sicurezza: non rimuovere tutto
    if keep == 0 {
        return text.to_string();
    }

    let cleaned = words[..keep].join(" ");
    format!("{}{}", cleaned, ending)
}

// ═══════════════════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── person_from_hexagram ────────────────────────────────────────────

    #[test]
    fn test_potere_prima_persona() {
        // POTERE = 0 = Cielo(0)/Cielo(0), lower=0=☰ Agency → Prima
        assert!(matches!(person_from_hexagram(0), Person::First));
    }

    #[test]
    fn test_materia_terza_persona() {
        // MATERIA = 9 = Terra(1)/Terra(1), lower=1=☷ Permanenza → Terza
        assert!(matches!(person_from_hexagram(9), Person::Third));
    }

    #[test]
    fn test_ardore_prima_persona() {
        // ARDORE = 18 = Tuono(2)/Tuono(2), lower=2=☳ Intensita → Prima
        assert!(matches!(person_from_hexagram(18), Person::First));
    }

    #[test]
    fn test_divenire_terza_persona() {
        // DIVENIRE = 27 = Acqua(3)/Acqua(3), lower=3=☵ Tempo → Terza (narrativo)
        assert!(matches!(person_from_hexagram(27), Person::Third));
    }

    #[test]
    fn test_spazio_prima_persona() {
        // SPAZIO = 36 = Montagna(4)/Montagna(4), lower=4=☶ Confine → Prima
        assert!(matches!(person_from_hexagram(36), Person::First));
    }

    #[test]
    fn test_intreccio_seconda_persona() {
        // INTRECCIO = 45 = Vento(5)/Vento(5), lower=5=☴ Complessita → Seconda
        assert!(matches!(person_from_hexagram(45), Person::Second));
    }

    #[test]
    fn test_verita_prima_persona() {
        // VERITA = 54 = Fuoco(6)/Fuoco(6), lower=6=☲ Definizione → Prima
        assert!(matches!(person_from_hexagram(54), Person::First));
    }

    #[test]
    fn test_armonia_seconda_persona() {
        // ARMONIA = 63 = Lago(7)/Lago(7), lower=7=☱ Valenza → Seconda
        assert!(matches!(person_from_hexagram(63), Person::Second));
    }

    // ── person_from_used_subject ────────────────────────────────────────

    #[test]
    fn test_used_io_forza_prima() {
        let used = vec!["io".to_string()];
        assert!(matches!(person_from_used_subject(&used), Some(Person::First)));
    }

    #[test]
    fn test_used_tu_forza_seconda() {
        let used = vec!["tu".to_string()];
        assert!(matches!(person_from_used_subject(&used), Some(Person::Second)));
    }

    #[test]
    fn test_used_senza_pronomi_none() {
        let used = vec!["calma".to_string(), "luce".to_string()];
        assert!(person_from_used_subject(&used).is_none());
    }

    // ── person_from_explicit_pronoun ────────────────────────────────────

    #[test]
    fn test_input_io_forza_prima() {
        let input = vec!["io".to_string(), "sono".to_string(), "qui".to_string()];
        assert!(matches!(person_from_explicit_pronoun(&input), Some(Person::First)));
    }

    #[test]
    fn test_input_mi_forza_prima() {
        let input = vec!["mi".to_string(), "sento".to_string()];
        assert!(matches!(person_from_explicit_pronoun(&input), Some(Person::First)));
    }

    #[test]
    fn test_input_senza_pronomi_none() {
        let input = vec!["campo".to_string(), "forza".to_string()];
        assert!(person_from_explicit_pronoun(&input).is_none());
    }

    // ── infer_grammatical_mode — priorità ───────────────────────────────

    #[test]
    fn test_used_soggetto_vince_su_hexagram() {
        use crate::topology::lexicon::Lexicon;
        // ARMONIA (63) → Seconda persona, MA "io" in `used` → Prima persona
        let lexicon = Lexicon::bootstrap();
        let active_fractals = vec![(63u32, 0.9)];
        let active_words: Vec<(&str, f64)> = vec![];
        let mode = infer_grammatical_mode(
            &active_fractals,
            &active_words,
            &lexicon,
            &[],
            &["io".to_string()],
        );
        assert!(matches!(mode.person, Person::First),
            "used 'io' deve vincere su ARMONIA → Seconda");
    }

    #[test]
    fn test_input_io_vince_su_hexagram() {
        use crate::topology::lexicon::Lexicon;
        // ARMONIA (63) → Seconda persona, MA "io" nell'input → Prima persona
        let lexicon = Lexicon::bootstrap();
        let active_fractals = vec![(63u32, 0.9)];
        let active_words: Vec<(&str, f64)> = vec![];
        let mode = infer_grammatical_mode(
            &active_fractals,
            &active_words,
            &lexicon,
            &["io".to_string()],
            &[],
        );
        assert!(matches!(mode.person, Person::First),
            "input 'io' deve vincere su ARMONIA → Seconda");
    }

    #[test]
    fn test_hexagram_usato_senza_pronomi() {
        use crate::topology::lexicon::Lexicon;
        // Nessun pronome → usa esagramma INTRECCIO(45) → Seconda
        let lexicon = Lexicon::bootstrap();
        let active_fractals = vec![(45u32, 0.8)];
        let active_words: Vec<(&str, f64)> = vec![];
        let mode = infer_grammatical_mode(
            &active_fractals,
            &active_words,
            &lexicon,
            &[],
            &[],
        );
        assert!(matches!(mode.person, Person::Second),
            "INTRECCIO(45) senza pronomi → Seconda persona");
    }

    // ── post_process ────────────────────────────────────────────────────

    #[test]
    fn test_rimuove_preposizione_orfana() {
        assert_eq!(post_process("Io sono corpo dalla."), "Io sono corpo.");
    }

    #[test]
    fn test_rimuove_preposizione_orfana_senza_punto() {
        assert_eq!(post_process("Io sento campo nel"), "Io sento campo");
    }

    #[test]
    fn test_rimuove_multipli_orfani() {
        // "Io sono campo della" → "Io sono campo."
        assert_eq!(post_process("Io sono campo della."), "Io sono campo.");
    }

    #[test]
    fn test_preserva_frase_corretta() {
        let ok = "Io sono campo.";
        assert_eq!(post_process(ok), ok);
    }

    #[test]
    fn test_preserva_puntini_sospensivi() {
        let ok = "Muovere, non so...";
        assert_eq!(post_process(ok), ok);
    }

    #[test]
    fn test_preserva_domanda() {
        let ok = "Calma, cosa?";
        assert_eq!(post_process(ok), ok);
    }

    #[test]
    fn test_non_rimuove_tutto() {
        // Se tutto è orfano, torna stringa originale
        let s = "della.";
        assert_eq!(post_process(s), s);
    }

    #[test]
    fn test_rimuove_preposizione_semplice() {
        assert_eq!(post_process("Io sento di."), "Io sento.");
    }
}
