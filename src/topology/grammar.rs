/// Grammatica Italiana — Coniugazione e lemmatizzazione morfologica.
///
/// Questo modulo non importa nulla dal resto di Prometeo — e autonomo.
/// Fornisce:
///   - `PartOfSpeech`: categoria grammaticale (per WordPattern)
///   - `conjugate()`: infinito + persona + tempo → forma coniugata
///   - `lemmatize()`: forma coniugata → infinito + persona + tempo
///   - `detect_pos_from_word()`: rilevamento POS da forma (suffissi + liste dirette)

use serde::{Serialize, Deserialize};

// ─── Tipi pubblici ───────────────────────────────────────────────────────────

/// Categoria grammaticale di una parola nel lessico.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    /// Verbo (forma infinita nel lessico)
    Verb,
    /// Nome
    Noun,
    /// Aggettivo
    Adjective,
    /// Avverbio
    Adverb,
    /// Pronome (io, tu, noi — con peso semantico pieno)
    Pronoun,
}

/// Persona grammaticale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Person {
    First,        // io
    Second,       // tu
    Third,        // lui/lei
    FirstPlural,  // noi
    SecondPlural, // voi
    ThirdPlural,  // loro
}

/// Tempo verbale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tense {
    Present,     // presente indicativo
    Imperfect,   // imperfetto indicativo
    Future,      // futuro semplice
    Conditional, // condizionale presente
}

/// Risultato della lemmatizzazione.
#[derive(Debug, Clone)]
pub struct LemmaResult {
    pub infinitive: String,
    pub person: Person,
    pub tense: Tense,
}

// ─── Coniugazione ────────────────────────────────────────────────────────────

/// Coniuga un verbo italiano.
/// Prima cerca nei verbi irregolari, poi applica i pattern regolari.
/// Se il verbo non e riconoscibile, restituisce l'infinito invariato.
pub fn conjugate(infinitive: &str, person: Person, tense: Tense) -> String {
    if let Some(form) = conjugate_irregular(infinitive, person, tense) {
        return form;
    }
    conjugate_regular(infinitive, person, tense)
}

fn conjugate_irregular(inf: &str, person: Person, tense: Tense) -> Option<String> {
    use Person::*;
    use Tense::*;

    let form: &str = match (inf, tense) {
        // ── essere ──────────────────────────────────────────────────────────
        ("essere", Present) => match person {
            First => "sono", Second => "sei", Third => "è",
            FirstPlural => "siamo", SecondPlural => "siete", ThirdPlural => "sono",
        },
        ("essere", Imperfect) => match person {
            First => "ero", Second => "eri", Third => "era",
            FirstPlural => "eravamo", SecondPlural => "eravate", ThirdPlural => "erano",
        },
        ("essere", Future) => match person {
            First => "sarò", Second => "sarai", Third => "sarà",
            FirstPlural => "saremo", SecondPlural => "sarete", ThirdPlural => "saranno",
        },
        ("essere", Conditional) => match person {
            First => "sarei", Second => "saresti", Third => "sarebbe",
            FirstPlural => "saremmo", SecondPlural => "sareste", ThirdPlural => "sarebbero",
        },
        // ── avere ───────────────────────────────────────────────────────────
        ("avere", Present) => match person {
            First => "ho", Second => "hai", Third => "ha",
            FirstPlural => "abbiamo", SecondPlural => "avete", ThirdPlural => "hanno",
        },
        ("avere", Imperfect) => match person {
            First => "avevo", Second => "avevi", Third => "aveva",
            FirstPlural => "avevamo", SecondPlural => "avevate", ThirdPlural => "avevano",
        },
        ("avere", Future) => match person {
            First => "avrò", Second => "avrai", Third => "avrà",
            FirstPlural => "avremo", SecondPlural => "avrete", ThirdPlural => "avranno",
        },
        ("avere", Conditional) => match person {
            First => "avrei", Second => "avresti", Third => "avrebbe",
            FirstPlural => "avremmo", SecondPlural => "avreste", ThirdPlural => "avrebbero",
        },
        // ── fare ────────────────────────────────────────────────────────────
        ("fare", Present) => match person {
            First => "faccio", Second => "fai", Third => "fa",
            FirstPlural => "facciamo", SecondPlural => "fate", ThirdPlural => "fanno",
        },
        ("fare", Imperfect) => match person {
            First => "facevo", Second => "facevi", Third => "faceva",
            FirstPlural => "facevamo", SecondPlural => "facevate", ThirdPlural => "facevano",
        },
        ("fare", Future) => match person {
            First => "farò", Second => "farai", Third => "farà",
            FirstPlural => "faremo", SecondPlural => "farete", ThirdPlural => "faranno",
        },
        ("fare", Conditional) => match person {
            First => "farei", Second => "faresti", Third => "farebbe",
            FirstPlural => "faremmo", SecondPlural => "fareste", ThirdPlural => "farebbero",
        },
        // ── andare ──────────────────────────────────────────────────────────
        ("andare", Present) => match person {
            First => "vado", Second => "vai", Third => "va",
            FirstPlural => "andiamo", SecondPlural => "andate", ThirdPlural => "vanno",
        },
        ("andare", Imperfect) => match person {
            First => "andavo", Second => "andavi", Third => "andava",
            FirstPlural => "andavamo", SecondPlural => "andavate", ThirdPlural => "andavano",
        },
        ("andare", Future) => match person {
            First => "andrò", Second => "andrai", Third => "andrà",
            FirstPlural => "andremo", SecondPlural => "andrete", ThirdPlural => "andranno",
        },
        ("andare", Conditional) => match person {
            First => "andrei", Second => "andresti", Third => "andrebbe",
            FirstPlural => "andremmo", SecondPlural => "andreste", ThirdPlural => "andrebbero",
        },
        // ── volere ──────────────────────────────────────────────────────────
        ("volere", Present) => match person {
            First => "voglio", Second => "vuoi", Third => "vuole",
            FirstPlural => "vogliamo", SecondPlural => "volete", ThirdPlural => "vogliono",
        },
        ("volere", Imperfect) => match person {
            First => "volevo", Second => "volevi", Third => "voleva",
            FirstPlural => "volevamo", SecondPlural => "volevate", ThirdPlural => "volevano",
        },
        ("volere", Future) => match person {
            First => "vorrò", Second => "vorrai", Third => "vorrà",
            FirstPlural => "vorremo", SecondPlural => "vorrete", ThirdPlural => "vorranno",
        },
        ("volere", Conditional) => match person {
            First => "vorrei", Second => "vorresti", Third => "vorrebbe",
            FirstPlural => "vorremmo", SecondPlural => "vorreste", ThirdPlural => "vorrebbero",
        },
        // ── potere ──────────────────────────────────────────────────────────
        ("potere", Present) => match person {
            First => "posso", Second => "puoi", Third => "può",
            FirstPlural => "possiamo", SecondPlural => "potete", ThirdPlural => "possono",
        },
        ("potere", Imperfect) => match person {
            First => "potevo", Second => "potevi", Third => "poteva",
            FirstPlural => "potevamo", SecondPlural => "potevate", ThirdPlural => "potevano",
        },
        ("potere", Future) => match person {
            First => "potrò", Second => "potrai", Third => "potrà",
            FirstPlural => "potremo", SecondPlural => "potrete", ThirdPlural => "potranno",
        },
        ("potere", Conditional) => match person {
            First => "potrei", Second => "potresti", Third => "potrebbe",
            FirstPlural => "potremmo", SecondPlural => "potreste", ThirdPlural => "potrebbero",
        },
        // ── sapere ──────────────────────────────────────────────────────────
        ("sapere", Present) => match person {
            First => "so", Second => "sai", Third => "sa",
            FirstPlural => "sappiamo", SecondPlural => "sapete", ThirdPlural => "sanno",
        },
        ("sapere", Imperfect) => match person {
            First => "sapevo", Second => "sapevi", Third => "sapeva",
            FirstPlural => "sapevamo", SecondPlural => "sapevate", ThirdPlural => "sapevano",
        },
        ("sapere", Future) => match person {
            First => "saprò", Second => "saprai", Third => "saprà",
            FirstPlural => "sapremo", SecondPlural => "saprete", ThirdPlural => "sapranno",
        },
        ("sapere", Conditional) => match person {
            First => "saprei", Second => "sapresti", Third => "saprebbe",
            FirstPlural => "sapremmo", SecondPlural => "sapreste", ThirdPlural => "saprebbero",
        },
        // ── venire ──────────────────────────────────────────────────────────
        ("venire", Present) => match person {
            First => "vengo", Second => "vieni", Third => "viene",
            FirstPlural => "veniamo", SecondPlural => "venite", ThirdPlural => "vengono",
        },
        ("venire", Imperfect) => match person {
            First => "venivo", Second => "venivi", Third => "veniva",
            FirstPlural => "venivamo", SecondPlural => "venivate", ThirdPlural => "venivano",
        },
        ("venire", Future) => match person {
            First => "verrò", Second => "verrai", Third => "verrà",
            FirstPlural => "verremo", SecondPlural => "verrete", ThirdPlural => "verranno",
        },
        ("venire", Conditional) => match person {
            First => "verrei", Second => "verresti", Third => "verrebbe",
            FirstPlural => "verremmo", SecondPlural => "verreste", ThirdPlural => "verrebbero",
        },
        // ── dire ────────────────────────────────────────────────────────────
        ("dire", Present) => match person {
            First => "dico", Second => "dici", Third => "dice",
            FirstPlural => "diciamo", SecondPlural => "dite", ThirdPlural => "dicono",
        },
        ("dire", Imperfect) => match person {
            First => "dicevo", Second => "dicevi", Third => "diceva",
            FirstPlural => "dicevamo", SecondPlural => "dicevate", ThirdPlural => "dicevano",
        },
        ("dire", Future) => match person {
            First => "dirò", Second => "dirai", Third => "dirà",
            FirstPlural => "diremo", SecondPlural => "direte", ThirdPlural => "diranno",
        },
        ("dire", Conditional) => match person {
            First => "direi", Second => "diresti", Third => "direbbe",
            FirstPlural => "diremmo", SecondPlural => "direste", ThirdPlural => "direbbero",
        },
        // ── dare ────────────────────────────────────────────────────────────
        ("dare", Present) => match person {
            First => "do", Second => "dai", Third => "dà",
            FirstPlural => "diamo", SecondPlural => "date", ThirdPlural => "danno",
        },
        ("dare", Imperfect) => match person {
            First => "davo", Second => "davi", Third => "dava",
            FirstPlural => "davamo", SecondPlural => "davate", ThirdPlural => "davano",
        },
        ("dare", Future) => match person {
            First => "darò", Second => "darai", Third => "darà",
            FirstPlural => "daremo", SecondPlural => "darete", ThirdPlural => "daranno",
        },
        ("dare", Conditional) => match person {
            First => "darei", Second => "daresti", Third => "darebbe",
            FirstPlural => "daremmo", SecondPlural => "dareste", ThirdPlural => "darebbero",
        },
        // ── stare ───────────────────────────────────────────────────────────
        ("stare", Present) => match person {
            First => "sto", Second => "stai", Third => "sta",
            FirstPlural => "stiamo", SecondPlural => "state", ThirdPlural => "stanno",
        },
        ("stare", Imperfect) => match person {
            First => "stavo", Second => "stavi", Third => "stava",
            FirstPlural => "stavamo", SecondPlural => "stavate", ThirdPlural => "stavano",
        },
        ("stare", Future) => match person {
            First => "starò", Second => "starai", Third => "starà",
            FirstPlural => "staremo", SecondPlural => "starete", ThirdPlural => "staranno",
        },
        ("stare", Conditional) => match person {
            First => "starei", Second => "staresti", Third => "starebbe",
            FirstPlural => "staremmo", SecondPlural => "sareste", ThirdPlural => "starebbero",
        },
        _ => return None,
    };
    Some(form.to_string())
}

fn conjugate_regular(inf: &str, person: Person, tense: Tense) -> String {
    use Person::*;
    use Tense::*;

    if inf.ends_with("are") {
        let stem = &inf[..inf.len() - 3];
        // Ortografia -care/-gare: il suono duro si conserva → "h" davanti a
        // i/e ("mancare"→manchi/manchiamo/mancherò, "pagare"→paghi/pagherai).
        // Regola grafica deterministica, vale SOLO per la 1ª coniugazione.
        let hard_cg = stem.ends_with('c') || stem.ends_with('g');
        let join = |suf: &str| -> String {
            // Ortografia -iare: la `i` atona del tema si FONDE con la desinenza
            // che inizia per i ("studiare"→studi/studiamo, "cambiare"→cambi,
            // "mangiare"→mangi/mangiamo), non si raddoppia (mai "studii").
            if stem.ends_with('i') && suf.starts_with('i') {
                format!("{}{}", stem, &suf[1..])
            } else if hard_cg && (suf.starts_with('i') || suf.starts_with('e')) {
                format!("{}h{}", stem, suf)
            } else {
                format!("{}{}", stem, suf)
            }
        };
        match tense {
            Present => {
                let suf = match person {
                    First => "o", Second => "i", Third => "a",
                    FirstPlural => "iamo", SecondPlural => "ate", ThirdPlural => "ano",
                };
                join(suf)
            }
            Imperfect => {
                let suf = match person {
                    First => "avo", Second => "avi", Third => "ava",
                    FirstPlural => "avamo", SecondPlural => "avate", ThirdPlural => "avano",
                };
                join(suf)
            }
            Future => {
                // "amare" → stem "am" → futuro stem "amer" ("mancare" → "mancher")
                let fstem = join("er");
                let suf = match person {
                    First => "ò", Second => "ai", Third => "à",
                    FirstPlural => "emo", SecondPlural => "ete", ThirdPlural => "anno",
                };
                format!("{}{}", fstem, suf)
            }
            Conditional => {
                let cstem = join("er");
                let suf = match person {
                    First => "ei", Second => "esti", Third => "ebbe",
                    FirstPlural => "emmo", SecondPlural => "este", ThirdPlural => "ebbero",
                };
                format!("{}{}", cstem, suf)
            }
        }
    } else if inf.ends_with("ere") {
        let stem = &inf[..inf.len() - 3];
        match tense {
            Present => {
                let suf = match person {
                    First => "o", Second => "i", Third => "e",
                    FirstPlural => "iamo", SecondPlural => "ete", ThirdPlural => "ono",
                };
                format!("{}{}", stem, suf)
            }
            Imperfect => {
                let suf = match person {
                    First => "evo", Second => "evi", Third => "eva",
                    FirstPlural => "evamo", SecondPlural => "evate", ThirdPlural => "evano",
                };
                format!("{}{}", stem, suf)
            }
            Future => {
                // "credere" → rimuovi "e" finale → "creder"
                let fstem = &inf[..inf.len() - 1];
                let suf = match person {
                    First => "ò", Second => "ai", Third => "à",
                    FirstPlural => "emo", SecondPlural => "ete", ThirdPlural => "anno",
                };
                format!("{}{}", fstem, suf)
            }
            Conditional => {
                let cstem = &inf[..inf.len() - 1];
                let suf = match person {
                    First => "ei", Second => "esti", Third => "ebbe",
                    FirstPlural => "emmo", SecondPlural => "este", ThirdPlural => "ebbero",
                };
                format!("{}{}", cstem, suf)
            }
        }
    } else if inf.ends_with("ire") {
        let stem = &inf[..inf.len() - 3];
        let finire_type = is_finire_type(inf);
        match tense {
            Present => {
                if finire_type {
                    let suf = match person {
                        First => "isco", Second => "isci", Third => "isce",
                        FirstPlural => "iamo", SecondPlural => "ite", ThirdPlural => "iscono",
                    };
                    format!("{}{}", stem, suf)
                } else {
                    let suf = match person {
                        First => "o", Second => "i", Third => "e",
                        FirstPlural => "iamo", SecondPlural => "ite", ThirdPlural => "ono",
                    };
                    format!("{}{}", stem, suf)
                }
            }
            Imperfect => {
                let suf = match person {
                    First => "ivo", Second => "ivi", Third => "iva",
                    FirstPlural => "ivamo", SecondPlural => "ivate", ThirdPlural => "ivano",
                };
                format!("{}{}", stem, suf)
            }
            Future => {
                let fstem = &inf[..inf.len() - 1];
                let suf = match person {
                    First => "ò", Second => "ai", Third => "à",
                    FirstPlural => "emo", SecondPlural => "ete", ThirdPlural => "anno",
                };
                format!("{}{}", fstem, suf)
            }
            Conditional => {
                let cstem = &inf[..inf.len() - 1];
                let suf = match person {
                    First => "ei", Second => "esti", Third => "ebbe",
                    FirstPlural => "emmo", SecondPlural => "este", ThirdPlural => "ebbero",
                };
                format!("{}{}", cstem, suf)
            }
        }
    } else {
        // Infinito non riconosciuto: restituisce invariato
        inf.to_string()
    }
}

/// Verbi -ire che usano -isco al presente (tipo "finire").
fn is_finire_type(inf: &str) -> bool {
    matches!(inf,
        "finire" | "capire" | "preferire" | "costruire" | "pulire" |
        "restituire" | "agire" | "definire" | "garantire" | "riferire" |
        "unire" | "obbedire" | "suggerire" | "proibire" | "eseguire" |
        "contribuire" | "distribuire" | "istituire" | "costituire" |
        "stabilire" | "subire" | "nutrire" | "reagire" | "istruire" |
        "inserire" | "gestire" | "condire" | "guarire" | "punire" |
        "impedire" | "chiarire" | "ferire" | "colpire" | "investire" |
        "digerire" | "svanire" | "fiorire" | "colorire" | "esaurire" |
        "abbellire" | "arricchire" | "indebolire" | "ingrandire" |
        "fornire" | "tradire" | "rapire" | "stupire" | "sparire"
    )
}

// ─── Lemmatizzazione ─────────────────────────────────────────────────────────

/// Lemmatizza una forma verbale italiana.
/// Restituisce l'infinito, la persona e il tempo se riconoscibile.
/// Restituisce None se la parola non e riconoscibile come verbo coniugato.
///
/// Strategia (dal piu specifico al meno):
///   1. Irregolari (tabella completa)
///   2. Imperfetto -are/-ere/-ire (suffissi molto distintivi)
///   3. Presente finire-type (-isco/-isci/-isce/-iscono)
///   4. Condizionale -ire (-irei/-iresti/-irebbe/-iremmo/-ireste/-irebbero)
///   5. Futuro -ire (-iro/-irai/-ira/-iremo/-irete/-iranno)
/// Phase 86 (#3): infinito di un PARTICIPIO PASSATO IRREGOLARE, o `None`.
/// Tabella-dato (come i verbi irregolari): la forma NON si ricostruisce per
/// stripping (-ato/-uto/-ito). Match per *stem* = participio senza la vocale di
/// concordanza (o/a/i/e), così "preso/presa/presi/prese" → "prendere". Stem ≥4
/// per ridurre l'ambiguità coi nomi (il seeding è additivo, quindi tollera
/// qualche falso positivo raro).
pub fn irregular_participle(word: &str) -> Option<String> {
    let w = word.to_lowercase();
    const IRREG: &[(&str, &str)] = &[
        ("pres", "prendere"), ("mess", "mettere"), ("scritt", "scrivere"),
        ("vist", "vedere"), ("apert", "aprire"), ("chius", "chiudere"),
        ("accolt", "accogliere"), ("scelt", "scegliere"), ("tolt", "togliere"),
        ("rispost", "rispondere"), ("chiest", "chiedere"), ("decis", "decidere"),
        ("spint", "spingere"), ("dipint", "dipingere"), ("strett", "stringere"),
        ("vint", "vincere"), ("giunt", "giungere"), ("offert", "offrire"),
        ("soffert", "soffrire"), ("copert", "coprire"), ("rott", "rompere"),
        ("cott", "cuocere"), ("fritt", "friggere"), ("vissut", "vivere"),
        ("conclus", "concludere"), ("divis", "dividere"), ("espress", "esprimere"),
        ("discuss", "discutere"), ("pers", "perdere"), ("cors", "correre"),
        ("mort", "morire"), ("nascost", "nascondere"), ("rimast", "rimanere"),
        ("vissut", "vivere"), ("prodott", "produrre"), ("condott", "condurre"),
        ("ridott", "ridurre"), ("tradott", "tradurre"), ("distrutt", "distruggere"),
        ("propost", "proporre"), ("compost", "comporre"), ("espost", "esporre"),
        ("impost", "imporre"), ("dispost", "disporre"), ("suppost", "supporre"),
        ("post", "porre"), ("assunt", "assumere"), ("presunt", "presumere"),
        ("svolt", "svolgere"), ("risolt", "risolvere"),
    ];
    let stem = if w.ends_with(|c| matches!(c, 'o' | 'a' | 'i' | 'e')) {
        &w[..w.len() - 1]
    } else {
        &w[..]
    };
    if stem.len() < 4 { return None; }
    IRREG.iter().find(|(s, _)| *s == stem).map(|(_, inf)| inf.to_string())
}

pub fn lemmatize(word: &str) -> Option<LemmaResult> {
    use Person::*;
    use Tense::*;

    let w = word.to_lowercase();
    let w = w.as_str();

    // 0. Enclitico pronominale/riflessivo su INFINITO (Phase 86 §2-bis):
    //    "abbandonarsi"→"abbandonare", "andarci"→"andare", "darsene"→"dare".
    //    L'infinito perde la "e" finale e si attacca il clitico; lo ricostruiamo.
    //    Guard anti-falso-positivo: dopo lo strip il residuo deve finire in "r" e
    //    il candidato (+"e") deve essere un infinito valido (-are/-ere/-ire) —
    //    così "corsi"→"cor"→"core"(ore) è RIFIUTATO, "abbandonar"→"abbandonare" no.
    //    Cliti più lunghi prima (sene/glielo… prima di si/ne/lo).
    const ENCLITICS: &[&str] = &[
        "glielo", "gliela", "glieli", "gliele", "gliene",
        "sene", "cene", "tene", "mene", "vene",
        "cela", "cele", "celo", "celi",
        "mela", "mele", "melo", "meli", "tela", "tele", "telo", "teli",
        "si", "ci", "mi", "ti", "vi", "ne", "lo", "la", "li", "le", "gli",
    ];
    for cl in ENCLITICS {
        if let Some(stem) = w.strip_suffix(cl) {
            if stem.ends_with('r') && stem.len() >= 3 {
                let inf = format!("{stem}e");
                if inf.ends_with("are") || inf.ends_with("ere") || inf.ends_with("ire") {
                    return Some(LemmaResult { infinitive: inf, person: Third, tense: Present });
                }
            }
        }
    }

    // 1. Irregolari
    if let Some(r) = lemmatize_irregular(w) {
        return Some(r);
    }

    // 1b. Participio passato IRREGOLARE (Phase 86 #3): "preso"→"prendere",
    //     "accolto"→"accogliere". Non ricostruibile per stripping → tabella-dato.
    if let Some(inf) = irregular_participle(w) {
        return Some(LemmaResult { infinitive: inf, person: Third, tense: Present });
    }

    // 2. Imperfetto -are (avano/avate/avamo/ava/avi/avo)
    for (suf, person) in &[
        ("avano", ThirdPlural), ("avate", SecondPlural), ("avamo", FirstPlural),
        ("ava", Third), ("avi", Second), ("avo", First),
    ] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() >= 2 {
                return Some(LemmaResult {
                    infinitive: format!("{}are", stem),
                    person: *person,
                    tense: Imperfect,
                });
            }
        }
    }

    // 2b. Imperfetto -ere (evano/evate/evamo/eva/evi/evo)
    for (suf, person) in &[
        ("evano", ThirdPlural), ("evate", SecondPlural), ("evamo", FirstPlural),
        ("eva", Third), ("evi", Second), ("evo", First),
    ] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() >= 2 {
                return Some(LemmaResult {
                    infinitive: format!("{}ere", stem),
                    person: *person,
                    tense: Imperfect,
                });
            }
        }
    }

    // 2c. Imperfetto -ire (ivano/ivate/ivamo/iva/ivi/ivo)
    for (suf, person) in &[
        ("ivano", ThirdPlural), ("ivate", SecondPlural), ("ivamo", FirstPlural),
        ("iva", Third), ("ivi", Second), ("ivo", First),
    ] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() >= 2 {
                return Some(LemmaResult {
                    infinitive: format!("{}ire", stem),
                    person: *person,
                    tense: Imperfect,
                });
            }
        }
    }

    // 3. Presente finire-type (molto specifico)
    for (suf, person) in &[
        ("iscono", ThirdPlural), ("isce", Third), ("isci", Second), ("isco", First),
    ] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() >= 2 {
                return Some(LemmaResult {
                    infinitive: format!("{}ire", stem),
                    person: *person,
                    tense: Present,
                });
            }
        }
    }

    // 3b. Presente regolare (-iamo, -ate, -ete, -ite, -ano, -ono, -o, -i, -a, -e)
    // Usiamo una priorità cauta e limitiamo ai suffissi lunghi per non sovrascrivere troppo.
    // L'italiano ha molte ambiguità qui (es. "mani" -> nome plurale o tu mani?)
    // Pertanto, applichiamo solo per radici abbastanza lunghe (>= 3 chars).
    for (suf, person, end_vowel) in &[
        ("iamo", FirstPlural, "a"), ("iamo", FirstPlural, "e"), ("iamo", FirstPlural, "i"),
        ("ate", SecondPlural, "a"), ("ete", SecondPlural, "e"), ("ite", SecondPlural, "i"),
        ("ano", ThirdPlural, "a"), ("ono", ThirdPlural, "e"), ("ono", ThirdPlural, "i"),
    ] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() >= 2 {
                return Some(LemmaResult {
                    infinitive: format!("{}re", format!("{}{}", stem, end_vowel)),
                    person: *person,
                    tense: Present,
                });
            }
        }
    }
    
    // Per "vivi", "ami", "senti" (2a persona)
    if let Some(stem) = w.strip_suffix("i") {
        if stem.len() >= 3 {
            // È un'euristica grezza: proviamo -are, -ere, -ire in ordine
            // L'ideale sarebbe controllare il lessico, ma qui non abbiamo accesso
            // Restituiamo una forma fittizia o preferiamo "ere"/"ire" se finisce con certe lettere.
            // Semplifichiamo per i verbi più comuni come "vivere".
            if stem.ends_with("iv") { // vivi -> vivere
                return Some(LemmaResult {
                    infinitive: format!("{}ere", stem),
                    person: Second,
                    tense: Present,
                });
            } else if stem.ends_with("am") { // ami -> amare
                return Some(LemmaResult {
                    infinitive: format!("{}are", stem),
                    person: Second,
                    tense: Present,
                });
            }
            // Fallback generico
            return Some(LemmaResult {
                infinitive: format!("{}are", stem), // Assumiamo 1a coniugazione per default
                person: Second,
                tense: Present,
            });
        }
    }

    // 4. Condizionale -ire (irei/iresti/irebbe/iremmo/ireste/irebbero — distintivo)
    for (suf, person) in &[
        ("irebbero", ThirdPlural), ("ireste", SecondPlural), ("iremmo", FirstPlural),
        ("irebbe", Third), ("iresti", Second), ("irei", First),
    ] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() >= 2 {
                return Some(LemmaResult {
                    infinitive: format!("{}ire", stem),
                    person: *person,
                    tense: Conditional,
                });
            }
        }
    }

    // 5. Futuro -ire (iranno/irete/iremo + accento: ira/irai/iro)
    // Nota: "iro/ira" hanno accento in italiano ma la forma e spesso scritta senza
    for (suf, person) in &[
        ("iranno", ThirdPlural), ("irete", SecondPlural), ("iremo", FirstPlural),
        ("irà", Third), ("irai", Second), ("irò", First),
    ] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() >= 2 {
                return Some(LemmaResult {
                    infinitive: format!("{}ire", stem),
                    person: *person,
                    tense: Future,
                });
            }
        }
    }

    None
}

/// Collasso morfologico GENERICO verso il/i lemma-candidati (Phase 86 §11).
///
/// A differenza di `lemmatize` (verbo-only, consumato dalla comprensione:
/// `pattern_matcher`, `action_reasoning`, `input_reading`, `sentence_proposition`),
/// questa funzione copre anche NOMI (plurale→singolare) e AGGETTIVI
/// (genere/numero/grado→base) — è il collasso usato al *seeding* del campo
/// (`engine::receive`) e dal *gate di cura* (`bin/check_lemma`).
///
/// L'italiano è ambiguo (plurale `-i` → `-o` *o* `-e`; `-e` → `-a`): per questo
/// **sovra-genera** un insieme di candidati invece di sceglierne uno solo. Chi
/// chiama disambigua: il seeding tiene il primo candidato che esiste nel lessico
/// (il lessico è il ponte, §11.2); il gate verifica se il lemma-bersaglio `into`
/// è fra i candidati. Funzione PURA: nessun accesso a lessico/KG.
///
/// I candidati verbali (via `lemmatize` + gerundio + 1ª sing.) vengono prima,
/// poi le riduzioni nominali/aggettivali. L'ordine conta solo come tie-break:
/// il filtro lessicale del chiamante è il vero disambiguatore.
pub fn lemma_candidates(word: &str) -> Vec<String> {
    let w = word.to_lowercase();
    let mut out: Vec<String> = Vec::new();
    let mut add = |c: String, out: &mut Vec<String>| {
        if c != w && c.chars().count() >= 2 && !out.contains(&c) {
            out.push(c);
        }
    };

    // ── 1. VERBO ───────────────────────────────────────────────────────────
    // 1a. Tutte le forme già coperte da lemmatize (irregolari, imperfetto,
    //     presente plurale, finire-type, condizionale, futuro, enclitici).
    if let Some(r) = lemmatize(&w) {
        add(r.infinitive, &mut out);
    }
    // 1b. Participio passato irregolare (idempotente con lemmatize).
    if let Some(inf) = irregular_participle(&w) {
        add(inf, &mut out);
    }
    // 1c. Gerundio: -ando → -are ; -endo → -ere | -ire (amando→amare, agendo→agire).
    if let Some(stem) = w.strip_suffix("ando") {
        if stem.chars().count() >= 2 { add(format!("{stem}are"), &mut out); }
    }
    if let Some(stem) = w.strip_suffix("endo") {
        if stem.chars().count() >= 2 {
            add(format!("{stem}ere"), &mut out);
            add(format!("{stem}ire"), &mut out);
        }
    }
    // 1d. 1ª singolare presente -o (abbacchio→abbacchiare): coniugazione ignota
    //     → candidati per tutte e tre. Stem ≥3 per ridurre il rumore sui nomi.
    if let Some(stem) = w.strip_suffix('o') {
        if stem.chars().count() >= 3 {
            add(format!("{stem}are"), &mut out);
            add(format!("{stem}ere"), &mut out);
            add(format!("{stem}ire"), &mut out);
        }
    }

    // ── 2. NOME / AGGETTIVO ──────────────────────────────────────────────────
    // Mutazione velare ortografica: uno stem in "ch"/"gh" davanti a -i/-e maschera
    // una base in "c"/"g" (antico→antichi, cronaca→cronache, amico→amici). Per ogni
    // stem proviamo anche la variante "ammorbidita".
    let soften = |stem: &str| -> Option<String> {
        if let Some(s) = stem.strip_suffix("ch") { return Some(format!("{s}c")); }
        if let Some(s) = stem.strip_suffix("gh") { return Some(format!("{s}g")); }
        None
    };
    // 2a. Superlativo assoluto -issimo/a/i/e → base (-o e -e): altissimo→alto,
    //     grandissimo→grande, agitatissimo→agitato, antichissimo→antico (velare).
    for suf in &["issimo", "issima", "issimi", "issime"] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.chars().count() >= 2 {
                for base in std::iter::once(stem.to_string()).chain(soften(stem)) {
                    add(format!("{base}o"), &mut out);
                    add(format!("{base}e"), &mut out);
                }
            }
        }
    }
    // 2b. Flessione genere/numero verso il lemma base.
    //   plurale -i → -o (libri→libro) | -e (api→ape, accidenti→accidente) |
    //               -a (poeti→poeta) | -io (binari→binario, edifici→edificio) | velare
    if let Some(stem) = w.strip_suffix('i') {
        if stem.chars().count() >= 2 {
            for base in std::iter::once(stem.to_string()).chain(soften(stem)) {
                add(format!("{base}o"), &mut out);
                add(format!("{base}e"), &mut out);
                add(format!("{base}a"), &mut out);
            }
            add(format!("{stem}io"), &mut out); // nomi in -io: -io → -i
        }
    }
    //   forma in -e → -a (femm. plur.: case→casa, aquile→aquila) |
    //              -o (agg. femm. plur.: diverse→diverso, solide→solido) | velare ;
    //              + 3ª sing. verbo -ere/-ire (affligge→affliggere, sente→sentire)
    if let Some(stem) = w.strip_suffix('e') {
        if stem.chars().count() >= 2 {
            for base in std::iter::once(stem.to_string()).chain(soften(stem)) {
                add(format!("{base}a"), &mut out);
                add(format!("{base}o"), &mut out);
            }
            add(format!("{stem}ere"), &mut out);
            add(format!("{stem}ire"), &mut out);
        }
    }
    //   femminile -a → -o (aggettivo: ampia→ampio) ; + verbo -are/-ere/-ire
    //              (3ª sing. e congiuntivo: ama→amare, discerna→discernere, risponda→rispondere)
    if let Some(stem) = w.strip_suffix('a') {
        if stem.chars().count() >= 2 {
            add(format!("{stem}o"), &mut out);
            add(format!("{stem}are"), &mut out);
            add(format!("{stem}ere"), &mut out);
            add(format!("{stem}ire"), &mut out);
        }
    }

    out
}

/// Lemma VALIDATO contro un dizionario esterno (KG/lessico, via la closure
/// `known`). Risolve il bug dell'"infinito inventato" (Tsunami, fase archetipo):
/// `lemmatize` è solo-verbi → forzava nomi/aggettivi in infiniti inesistenti
/// (`mondi→mondare`, `possibili→possibilare`), gonfiando `unknown_words`.
///
/// Qui: prova i candidati morfologici (verbo + nome/aggettivo, via
/// `lemma_candidates`) confermati dal dizionario. Tre esiti, ONESTI:
///   - la parola stessa è nota → è già lemma;
///   - ESATTAMENTE UN candidato confermato → disambiguo, lo usa (cani→cane);
///   - PIÙ candidati confermati (es. "mondi"→{mondo, mondare} — "mondare" È un
///     verbo reale) → **NON indovina**: deferisce alla forma di superficie. La
///     disambiguazione nome-vs-verbo è CONTESTUALE (il ruolo nella clausola,
///     analisi logica), non un trucco morfologico cieco;
///   - zero conferme → superficie.
/// MAI un infinito speculativo (mondi↛mondare). `grammar` resta autonomo: il
/// dizionario entra come closure. NB: interim — la versione piena lemmatizzerà
/// per RUOLO (verbo→infinito, argomento→singolare) quando il chunker clausa-aware
/// assegna i ruoli; questa funzione resterà la riduzione nominale validata.
pub fn kg_validated_lemma(word: &str, known: impl Fn(&str) -> bool) -> String {
    kg_validated_with(word, known, lemma_candidates)
}

/// Lemmatizzazione NOMINALE validata: come `kg_validated_lemma` ma genera **solo
/// candidati nome/aggettivo** (`nominal_lemma_candidates`), mai infiniti verbali.
/// È la lemmatizzazione PER RUOLO (Phase 86+): quando il chunker clausa-aware ha
/// marcato un token come argomento/attributo sappiamo che è un nome → "mondi"→
/// "mondo" anche se "mondare" esiste nel KG (la disambiguazione nome-vs-verbo
/// l'ha fatta il RUOLO, non un trucco morfologico — [[feedback-no-tricks…]]).
pub fn kg_validated_nominal(word: &str, known: impl Fn(&str) -> bool) -> String {
    kg_validated_with(word, known, nominal_lemma_candidates)
}

/// Come `kg_validated_nominal` ma con il GENERE noto dall'articolo (accordo
/// grammaticale): scioglie la falsa-ambiguità dei plurali `-i` ("i gatti"→"gatto"
/// invece di deferire perché anche "gatta" è nel KG). `masc=None` → identico a
/// `kg_validated_nominal`. È disambiguazione contestuale, non un trucco
/// morfologico ([[feedback-no-tricks-toward-reality]]).
pub fn kg_validated_nominal_gendered(
    word: &str,
    masc: Option<bool>,
    known: impl Fn(&str) -> bool,
) -> String {
    kg_validated_with(word, known, |w| nominal_lemma_candidates_gendered(w, masc))
}

/// Cuore comune: prova i candidati prodotti da `candidates`, ritorna l'unico
/// confermato dal dizionario; deferisce alla superficie se ambiguo o assente.
fn kg_validated_with(
    word: &str,
    known: impl Fn(&str) -> bool,
    candidates: impl Fn(&str) -> Vec<String>,
) -> String {
    let w = word.to_lowercase();
    if w.chars().count() < 2 {
        return w;
    }
    if known(&w) {
        return w; // la forma stessa è nel dizionario → è già lemma
    }
    let mut found: Option<String> = None;
    for c in candidates(&w) {
        if !known(&c) {
            continue;
        }
        match &found {
            None => found = Some(c),
            Some(prev) if *prev != c => return w, // ambiguo → deferisci al contesto
            _ => {}
        }
    }
    found.unwrap_or(w) // disambiguo → lemma; zero conferme → superficie
}

/// Candidati di lemma NOMINALE (nome/aggettivo): solo riduzioni di genere/numero
/// (-issimo→base, plurale -i→-o/-e/-a/-io, -e→-a/-o, -a→-o, con mutazione velare
/// ch/gh→c/g). MAI infiniti verbali — quello lo fa `lemma_candidates` quando la
/// classe è ignota. Qui la classe (nome) è già nota dal ruolo.
pub fn nominal_lemma_candidates(word: &str) -> Vec<String> {
    nominal_lemma_candidates_gendered(word, None)
}

/// Come `nominal_lemma_candidates`, ma se il genere è NOTO (dall'articolo che
/// precede il nome — accordo grammaticale, non un trucco morfologico) filtra i
/// candidati del plurale `-i`: un maschile (`i/gli gatti`) non viene mai da `-a`
/// (gatta→gatte, non gatti); un femminile (`le armi`) non viene mai da `-o`.
/// Questo scioglie la falsa-ambiguità che faceva deferire "gatti" (→{gatto,gatta}
/// entrambi nel KG). `masc = Some(true)` maschile, `Some(false)` femminile,
/// `None` = genere ignoto → comportamento invariato (tutti i candidati).
pub fn nominal_lemma_candidates_gendered(word: &str, masc: Option<bool>) -> Vec<String> {
    let w = word.to_lowercase();
    let mut out: Vec<String> = Vec::new();
    let mut add = |c: String, out: &mut Vec<String>| {
        if c != w && c.chars().count() >= 2 && !out.contains(&c) {
            out.push(c);
        }
    };
    let soften = |stem: &str| -> Option<String> {
        if let Some(s) = stem.strip_suffix("ch") { return Some(format!("{s}c")); }
        if let Some(s) = stem.strip_suffix("gh") { return Some(format!("{s}g")); }
        None
    };
    // superlativo assoluto -issimo/a/i/e → base (-o/-e)
    for suf in &["issimo", "issima", "issimi", "issime"] {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.chars().count() >= 2 {
                for base in std::iter::once(stem.to_string()).chain(soften(stem)) {
                    add(format!("{base}o"), &mut out);
                    add(format!("{base}e"), &mut out);
                }
            }
        }
    }
    // plurale -i → -o | -e | -a | -io (+velare).
    // Filtro di genere (se noto): maschile mai -a, femminile mai -o/-io.
    if let Some(stem) = w.strip_suffix('i') {
        if stem.chars().count() >= 2 {
            let allow_o = masc != Some(false); // -o è maschile
            let allow_a = masc != Some(true);  // -a è femminile
            for base in std::iter::once(stem.to_string()).chain(soften(stem)) {
                if allow_o { add(format!("{base}o"), &mut out); }
                add(format!("{base}e"), &mut out); // -e: entrambi i generi
                if allow_a { add(format!("{base}a"), &mut out); }
            }
            if allow_o { add(format!("{stem}io"), &mut out); }
        }
    }
    // plurale -e → -a | -o (+velare). "le piante"→pianta (femminile -a);
    // il filtro di genere evita la falsa-ambiguità con "pianto" (-o maschile).
    if let Some(stem) = w.strip_suffix('e') {
        if stem.chars().count() >= 2 {
            let allow_o = masc != Some(false);
            let allow_a = masc != Some(true);
            for base in std::iter::once(stem.to_string()).chain(soften(stem)) {
                if allow_a { add(format!("{base}a"), &mut out); }
                if allow_o { add(format!("{base}o"), &mut out); }
            }
        }
    }
    // femminile -a → -o (aggettivo: ampia→ampio)
    if let Some(stem) = w.strip_suffix('a') {
        if stem.chars().count() >= 2 {
            add(format!("{stem}o"), &mut out);
        }
    }
    out
}

/// Reverse lookup per verbi irregolari.
fn lemmatize_irregular(w: &str) -> Option<LemmaResult> {
    use Person::*;
    use Tense::*;

    let (inf, person, tense): (&str, Person, Tense) = match w {
        // ── essere ──────────────────────────────────────────────────────────
        "sono"     => ("essere", First,        Present),   // ambiguo: anche ThirdPlural
        "sei"      => ("essere", Second,       Present),
        "è"        => ("essere", Third,        Present),
        "siamo"    => ("essere", FirstPlural,  Present),
        "siete"    => ("essere", SecondPlural, Present),
        "ero"      => ("essere", First,        Imperfect),
        "eri"      => ("essere", Second,       Imperfect),
        "era"      => ("essere", Third,        Imperfect),
        "eravamo"  => ("essere", FirstPlural,  Imperfect),
        "eravate"  => ("essere", SecondPlural, Imperfect),
        "erano"    => ("essere", ThirdPlural,  Imperfect),
        "sarò"     => ("essere", First,        Future),
        "sarai"    => ("essere", Second,       Future),
        "sarà"     => ("essere", Third,        Future),
        "saremo"   => ("essere", FirstPlural,  Future),
        "sarete"   => ("essere", SecondPlural, Future),
        "saranno"  => ("essere", ThirdPlural,  Future),
        "sarei"    => ("essere", First,        Conditional),
        "saresti"  => ("essere", Second,       Conditional),
        "sarebbe"  => ("essere", Third,        Conditional),
        "saremmo"  => ("essere", FirstPlural,  Conditional),
        "sareste"  => ("essere", SecondPlural, Conditional),
        "sarebbero"=> ("essere", ThirdPlural,  Conditional),
        // ── avere ───────────────────────────────────────────────────────────
        "ho"       => ("avere", First,        Present),
        "hai"      => ("avere", Second,       Present),
        "ha"       => ("avere", Third,        Present),
        "abbiamo"  => ("avere", FirstPlural,  Present),
        "hanno"    => ("avere", ThirdPlural,  Present),
        "avevo"    => ("avere", First,        Imperfect),
        "avevi"    => ("avere", Second,       Imperfect),
        "aveva"    => ("avere", Third,        Imperfect),
        "avevamo"  => ("avere", FirstPlural,  Imperfect),
        "avevate"  => ("avere", SecondPlural, Imperfect),
        "avevano"  => ("avere", ThirdPlural,  Imperfect),
        "avrò"     => ("avere", First,        Future),
        "avrai"    => ("avere", Second,       Future),
        "avrà"     => ("avere", Third,        Future),
        "avremo"   => ("avere", FirstPlural,  Future),
        "avrete"   => ("avere", SecondPlural, Future),
        "avranno"  => ("avere", ThirdPlural,  Future),
        "avrei"    => ("avere", First,        Conditional),
        "avresti"  => ("avere", Second,       Conditional),
        "avrebbe"  => ("avere", Third,        Conditional),
        "avremmo"  => ("avere", FirstPlural,  Conditional),
        "avreste"  => ("avere", SecondPlural, Conditional),
        "avrebbero"=> ("avere", ThirdPlural,  Conditional),
        // ── fare ────────────────────────────────────────────────────────────
        "faccio"   => ("fare", First,        Present),
        "fai"      => ("fare", Second,       Present),
        "facciamo" => ("fare", FirstPlural,  Present),
        "fanno"    => ("fare", ThirdPlural,  Present),
        "facevo"   => ("fare", First,        Imperfect),
        "facevi"   => ("fare", Second,       Imperfect),
        "faceva"   => ("fare", Third,        Imperfect),
        "facevamo" => ("fare", FirstPlural,  Imperfect),
        "facevate" => ("fare", SecondPlural, Imperfect),
        "facevano" => ("fare", ThirdPlural,  Imperfect),
        "farò"     => ("fare", First,        Future),
        "farai"    => ("fare", Second,       Future),
        "farà"     => ("fare", Third,        Future),
        "faremo"   => ("fare", FirstPlural,  Future),
        "farete"   => ("fare", SecondPlural, Future),
        "faranno"  => ("fare", ThirdPlural,  Future),
        "farei"    => ("fare", First,        Conditional),
        "faresti"  => ("fare", Second,       Conditional),
        "farebbe"  => ("fare", Third,        Conditional),
        "faremmo"  => ("fare", FirstPlural,  Conditional),
        "fareste"  => ("fare", SecondPlural, Conditional),
        "farebbero"=> ("fare", ThirdPlural,  Conditional),
        // ── andare ──────────────────────────────────────────────────────────
        "vado"     => ("andare", First,        Present),
        "vai"      => ("andare", Second,       Present),
        "va"       => ("andare", Third,        Present),
        "andiamo"  => ("andare", FirstPlural,  Present),
        "vanno"    => ("andare", ThirdPlural,  Present),
        "andavo"   => ("andare", First,        Imperfect),
        "andavi"   => ("andare", Second,       Imperfect),
        "andava"   => ("andare", Third,        Imperfect),
        "andavamo" => ("andare", FirstPlural,  Imperfect),
        "andavate" => ("andare", SecondPlural, Imperfect),
        "andavano" => ("andare", ThirdPlural,  Imperfect),
        "andrò"    => ("andare", First,        Future),
        "andrai"   => ("andare", Second,       Future),
        "andrà"    => ("andare", Third,        Future),
        "andremo"  => ("andare", FirstPlural,  Future),
        "andrete"  => ("andare", SecondPlural, Future),
        "andranno" => ("andare", ThirdPlural,  Future),
        "andrei"   => ("andare", First,        Conditional),
        "andresti" => ("andare", Second,       Conditional),
        "andrebbe" => ("andare", Third,        Conditional),
        "andremmo" => ("andare", FirstPlural,  Conditional),
        "andreste" => ("andare", SecondPlural, Conditional),
        "andrebbero"=> ("andare", ThirdPlural, Conditional),
        // ── riuscire (presente irregolare: riesc-) ───────────────────────────
        "riesco"    => ("riuscire", First,        Present),
        "riesci"    => ("riuscire", Second,       Present),
        "riesce"    => ("riuscire", Third,        Present),
        "riusciamo" => ("riuscire", FirstPlural,  Present),
        "riuscite"  => ("riuscire", SecondPlural, Present),
        "riescono"  => ("riuscire", ThirdPlural,  Present),
        // ── dovere ──────────────────────────────────────────────────────────
        "devo"     => ("dovere", First,        Present),
        "devi"     => ("dovere", Second,       Present),
        "deve"     => ("dovere", Third,        Present),
        "dobbiamo" => ("dovere", FirstPlural,  Present),
        "dovete"   => ("dovere", SecondPlural, Present),
        "devono"   => ("dovere", ThirdPlural,  Present),
        "dovevo"   => ("dovere", First,        Imperfect),
        "dovevi"   => ("dovere", Second,       Imperfect),
        "doveva"   => ("dovere", Third,        Imperfect),
        "dovevamo" => ("dovere", FirstPlural,  Imperfect),
        "dovevate" => ("dovere", SecondPlural, Imperfect),
        "dovevano" => ("dovere", ThirdPlural,  Imperfect),
        "dovrò"    => ("dovere", First,        Future),
        "dovrai"   => ("dovere", Second,       Future),
        "dovrà"    => ("dovere", Third,        Future),
        "dovremo"  => ("dovere", FirstPlural,  Future),
        "dovrete"  => ("dovere", SecondPlural, Future),
        "dovranno" => ("dovere", ThirdPlural,  Future),
        "dovrei"   => ("dovere", First,        Conditional),
        "dovresti" => ("dovere", Second,       Conditional),
        "dovrebbe" => ("dovere", Third,        Conditional),
        "dovremmo" => ("dovere", FirstPlural,  Conditional),
        "dovreste" => ("dovere", SecondPlural, Conditional),
        "dovrebbero"=> ("dovere", ThirdPlural, Conditional),
        // ── volere ──────────────────────────────────────────────────────────
        "voglio"   => ("volere", First,        Present),
        "vuoi"     => ("volere", Second,       Present),
        "vuole"    => ("volere", Third,        Present),
        "vogliamo" => ("volere", FirstPlural,  Present),
        "vogliono" => ("volere", ThirdPlural,  Present),
        "volevo"   => ("volere", First,        Imperfect),
        "volevi"   => ("volere", Second,       Imperfect),
        "voleva"   => ("volere", Third,        Imperfect),
        "volevamo" => ("volere", FirstPlural,  Imperfect),
        "volevate" => ("volere", SecondPlural, Imperfect),
        "volevano" => ("volere", ThirdPlural,  Imperfect),
        "vorrò"    => ("volere", First,        Future),
        "vorrai"   => ("volere", Second,       Future),
        "vorrà"    => ("volere", Third,        Future),
        "vorremo"  => ("volere", FirstPlural,  Future),
        "vorrete"  => ("volere", SecondPlural, Future),
        "vorranno" => ("volere", ThirdPlural,  Future),
        "vorrei"   => ("volere", First,        Conditional),
        "vorresti" => ("volere", Second,       Conditional),
        "vorrebbe" => ("volere", Third,        Conditional),
        "vorremmo" => ("volere", FirstPlural,  Conditional),
        "vorreste" => ("volere", SecondPlural, Conditional),
        "vorrebbero"=> ("volere", ThirdPlural, Conditional),
        // ── potere ──────────────────────────────────────────────────────────
        "posso"    => ("potere", First,        Present),
        "puoi"     => ("potere", Second,       Present),
        "può"      => ("potere", Third,        Present),
        "possiamo" => ("potere", FirstPlural,  Present),
        "possono"  => ("potere", ThirdPlural,  Present),
        "potevo"   => ("potere", First,        Imperfect),
        "potevi"   => ("potere", Second,       Imperfect),
        "poteva"   => ("potere", Third,        Imperfect),
        "potevamo" => ("potere", FirstPlural,  Imperfect),
        "potevate" => ("potere", SecondPlural, Imperfect),
        "potevano" => ("potere", ThirdPlural,  Imperfect),
        "potrò"    => ("potere", First,        Future),
        "potrai"   => ("potere", Second,       Future),
        "potrà"    => ("potere", Third,        Future),
        "potremo"  => ("potere", FirstPlural,  Future),
        "potrete"  => ("potere", SecondPlural, Future),
        "potranno" => ("potere", ThirdPlural,  Future),
        "potrei"   => ("potere", First,        Conditional),
        "potresti" => ("potere", Second,       Conditional),
        "potrebbe" => ("potere", Third,        Conditional),
        "potremmo" => ("potere", FirstPlural,  Conditional),
        "potreste" => ("potere", SecondPlural, Conditional),
        "potrebbero"=> ("potere", ThirdPlural, Conditional),
        // ── sapere ──────────────────────────────────────────────────────────
        "so"       => ("sapere", First,        Present),
        "sappiamo" => ("sapere", FirstPlural,  Present),
        "sanno"    => ("sapere", ThirdPlural,  Present),
        "sapevo"   => ("sapere", First,        Imperfect),
        "sapevi"   => ("sapere", Second,       Imperfect),
        "sapeva"   => ("sapere", Third,        Imperfect),
        "sapevamo" => ("sapere", FirstPlural,  Imperfect),
        "sapevate" => ("sapere", SecondPlural, Imperfect),
        "sapevano" => ("sapere", ThirdPlural,  Imperfect),
        "saprò"    => ("sapere", First,        Future),
        "saprai"   => ("sapere", Second,       Future),
        "saprà"    => ("sapere", Third,        Future),
        "sapremo"  => ("sapere", FirstPlural,  Future),
        "saprete"  => ("sapere", SecondPlural, Future),
        "sapranno" => ("sapere", ThirdPlural,  Future),
        "saprei"   => ("sapere", First,        Conditional),
        "sapresti" => ("sapere", Second,       Conditional),
        "saprebbe" => ("sapere", Third,        Conditional),
        "sapremmo" => ("sapere", FirstPlural,  Conditional),
        "sapreste" => ("sapere", SecondPlural, Conditional),
        "saprebbero"=> ("sapere", ThirdPlural, Conditional),
        // ── venire ──────────────────────────────────────────────────────────
        "vengo"    => ("venire", First,        Present),
        "vieni"    => ("venire", Second,       Present),
        "viene"    => ("venire", Third,        Present),
        "veniamo"  => ("venire", FirstPlural,  Present),
        "vengono"  => ("venire", ThirdPlural,  Present),
        "venivo"   => ("venire", First,        Imperfect),
        "venivi"   => ("venire", Second,       Imperfect),
        "veniva"   => ("venire", Third,        Imperfect),
        "venivamo" => ("venire", FirstPlural,  Imperfect),
        "venivate" => ("venire", SecondPlural, Imperfect),
        "venivano" => ("venire", ThirdPlural,  Imperfect),
        "verrò"    => ("venire", First,        Future),
        "verrai"   => ("venire", Second,       Future),
        "verrà"    => ("venire", Third,        Future),
        "verremo"  => ("venire", FirstPlural,  Future),
        "verrete"  => ("venire", SecondPlural, Future),
        "verranno" => ("venire", ThirdPlural,  Future),
        "verrei"   => ("venire", First,        Conditional),
        "verresti" => ("venire", Second,       Conditional),
        "verrebbe" => ("venire", Third,        Conditional),
        "verremmo" => ("venire", FirstPlural,  Conditional),
        "verreste" => ("venire", SecondPlural, Conditional),
        "verrebbero"=> ("venire", ThirdPlural, Conditional),
        // ── dire ────────────────────────────────────────────────────────────
        "dico"     => ("dire", First,        Present),
        "dici"     => ("dire", Second,       Present),
        "dice"     => ("dire", Third,        Present),
        "diciamo"  => ("dire", FirstPlural,  Present),
        "dicono"   => ("dire", ThirdPlural,  Present),
        "dicevo"   => ("dire", First,        Imperfect),
        "dicevi"   => ("dire", Second,       Imperfect),
        "diceva"   => ("dire", Third,        Imperfect),
        "dicevamo" => ("dire", FirstPlural,  Imperfect),
        "dicevate" => ("dire", SecondPlural, Imperfect),
        "dicevano" => ("dire", ThirdPlural,  Imperfect),
        "dirò"     => ("dire", First,        Future),
        "dirai"    => ("dire", Second,       Future),
        "dirà"     => ("dire", Third,        Future),
        "diremo"   => ("dire", FirstPlural,  Future),
        "direte"   => ("dire", SecondPlural, Future),
        "diranno"  => ("dire", ThirdPlural,  Future),
        "direi"    => ("dire", First,        Conditional),
        "diresti"  => ("dire", Second,       Conditional),
        "direbbe"  => ("dire", Third,        Conditional),
        "diremmo"  => ("dire", FirstPlural,  Conditional),
        "direste"  => ("dire", SecondPlural, Conditional),
        "direbbero"=> ("dire", ThirdPlural,  Conditional),
        // ── dare ────────────────────────────────────────────────────────────
        "do"       => ("dare", First,        Present),
        "dà"       => ("dare", Third,        Present),
        "diamo"    => ("dare", FirstPlural,  Present),
        "danno"    => ("dare", ThirdPlural,  Present),
        "davo"     => ("dare", First,        Imperfect),
        "davi"     => ("dare", Second,       Imperfect),
        "dava"     => ("dare", Third,        Imperfect),
        "davamo"   => ("dare", FirstPlural,  Imperfect),
        "davate"   => ("dare", SecondPlural, Imperfect),
        "davano"   => ("dare", ThirdPlural,  Imperfect),
        "darò"     => ("dare", First,        Future),
        "darai"    => ("dare", Second,       Future),
        "darà"     => ("dare", Third,        Future),
        "daremo"   => ("dare", FirstPlural,  Future),
        "darete"   => ("dare", SecondPlural, Future),
        "daranno"  => ("dare", ThirdPlural,  Future),
        "darei"    => ("dare", First,        Conditional),
        "daresti"  => ("dare", Second,       Conditional),
        "darebbe"  => ("dare", Third,        Conditional),
        "daremmo"  => ("dare", FirstPlural,  Conditional),
        "dareste"  => ("dare", SecondPlural, Conditional),
        "darebbero"=> ("dare", ThirdPlural,  Conditional),
        // ── stare ───────────────────────────────────────────────────────────
        "sto"      => ("stare", First,        Present),
        "stai"     => ("stare", Second,       Present),
        "sta"      => ("stare", Third,        Present),
        "stiamo"   => ("stare", FirstPlural,  Present),
        "stanno"   => ("stare", ThirdPlural,  Present),
        "stavo"    => ("stare", First,        Imperfect),
        "stavi"    => ("stare", Second,       Imperfect),
        "stava"    => ("stare", Third,        Imperfect),
        "stavamo"  => ("stare", FirstPlural,  Imperfect),
        "stavate"  => ("stare", SecondPlural, Imperfect),
        "stavano"  => ("stare", ThirdPlural,  Imperfect),
        "starò"    => ("stare", First,        Future),
        "starai"   => ("stare", Second,       Future),
        "starà"    => ("stare", Third,        Future),
        "staremo"  => ("stare", FirstPlural,  Future),
        "starete"  => ("stare", SecondPlural, Future),
        "staranno" => ("stare", ThirdPlural,  Future),
        "starei"   => ("stare", First,        Conditional),
        "staresti" => ("stare", Second,       Conditional),
        "starebbe" => ("stare", Third,        Conditional),
        "staremmo" => ("stare", FirstPlural,  Conditional),
        "stareste" => ("stare", SecondPlural, Conditional),
        "starebbero"=> ("stare", ThirdPlural, Conditional),
        _ => return None,
    };

    Some(LemmaResult {
        infinitive: inf.to_string(),
        person,
        tense,
    })
}

// ─── Rilevamento POS ─────────────────────────────────────────────────────────

/// Rileva se una parola e probabilmente un verbo all'infinito.
/// Euristica: lunghezza >= 5 e suffisso -are/-ere/-ire.
/// Rileva la categoria grammaticale di una parola dalla sua forma.
///
/// Ordine di priorità: Pronoun → Adverb → Verb → Noun → Adjective.
/// Alta precisione sui suffissi italiani + liste dirette per parole ad alta frequenza.
/// Restituisce None solo se nessuna regola si applica con sufficiente confidenza.
pub fn detect_pos_from_word(word: &str) -> Option<PartOfSpeech> {
    let len = word.chars().count();

    // ── Pronomi (lista diretta — forma invariabile) ──────────────────────────
    const PRONOUNS: &[&str] = &[
        "io", "tu", "lui", "lei", "noi", "voi", "loro",
        "me", "te", "se", "ci", "vi", "si", "mi", "ti", "ne",
        "egli", "ella", "essi", "esse", "lo", "gli",
    ];
    if PRONOUNS.contains(&word) {
        return Some(PartOfSpeech::Pronoun);
    }

    // ── Avverbi: suffisso -mente (≥8 char) — quasi 100% precisione ──────────
    if len >= 8 && word.ends_with("mente") {
        return Some(PartOfSpeech::Adverb);
    }

    // ── Avverbi: lista diretta ───────────────────────────────────────────────
    const ADVERBS: &[&str] = &[
        "molto", "poco", "tanto", "troppo", "sempre", "mai", "ancora",
        "anche", "spesso", "presto", "tardi", "bene", "male", "meglio", "peggio",
        "insieme", "subito", "forse", "davvero", "invece", "però",
        "ora", "poi", "dentro", "fuori", "sopra", "sotto", "prima", "dopo",
        "quasi", "circa", "certo", "già", "adesso", "oggi",
        "ieri", "domani", "lì", "qui", "là", "qua",
        // Eccezioni a suffissi che sembrano nomi (-anza, -enza)
        "abbastanza", "abbondanza",
    ];
    if ADVERBS.contains(&word) {
        return Some(PartOfSpeech::Adverb);
    }

    // ── Verbi: infinito in -are/-ere/-ire (≥5 char) ─────────────────────────
    if len >= 5
        && (word.ends_with("are") || word.ends_with("ere") || word.ends_with("ire"))
    {
        return Some(PartOfSpeech::Verb);
    }

    // ── Sostantivi: suffissi ad alta affidabilità ────────────────────────────
    // -zione/-sione: nazione, passione, emozione — 98%+ precisione
    if len >= 7 && (word.ends_with("zione") || word.ends_with("sione")) {
        return Some(PartOfSpeech::Noun);
    }
    // -tà/-ità: libertà, qualità, identità, realtà — 99%+
    if len >= 3 && word.ends_with("tà") {
        return Some(PartOfSpeech::Noun);
    }
    // -mento: sentimento, movimento, pensiero — 95%+
    if len >= 7 && word.ends_with("mento") {
        return Some(PartOfSpeech::Noun);
    }
    // -ezza: bellezza, dolcezza, grandezza — 98%+
    if len >= 6 && word.ends_with("ezza") {
        return Some(PartOfSpeech::Noun);
    }
    // -ismo: realismo, simbolismo — 98%+
    if len >= 6 && word.ends_with("ismo") {
        return Some(PartOfSpeech::Noun);
    }
    // -tura: natura, scultura, struttura — 90%+ (>6 char per evitare "tura" da solo)
    if len >= 7 && word.ends_with("tura") {
        return Some(PartOfSpeech::Noun);
    }
    // -anza/-enza: speranza, presenza, differenza — 90%+
    if len >= 7 && (word.ends_with("anza") || word.ends_with("enza")) {
        return Some(PartOfSpeech::Noun);
    }
    // -tore/-trice: scrittore, pittrice, attore — 90%+
    if len >= 7 && (word.ends_with("tore") || word.ends_with("trice")) {
        return Some(PartOfSpeech::Noun);
    }
    // -aggio: coraggio, viaggio, paesaggio — 95%+
    if len >= 7 && word.ends_with("aggio") {
        return Some(PartOfSpeech::Noun);
    }
    // -ione (generica, più ampia): azione, emozione, opinione — 95%+
    if len >= 7 && word.ends_with("ione") {
        return Some(PartOfSpeech::Noun);
    }

    // ── Sostantivi: lista diretta (alta frequenza nel lessico Prometeo) ───────
    const NOUNS: &[&str] = &[
        // corpo e percezione
        "corpo", "mente", "anima", "cuore", "occhio", "occhi", "mano", "mani",
        "testa", "voce", "pelle", "sangue", "carne", "osso", "ossa",
        "respiro", "fiato", "sguardo", "gesto", "passo",
        // spazio e tempo
        "tempo", "spazio", "luogo", "posto", "mondo", "terra", "cielo",
        "acqua", "fuoco", "aria", "luce", "buio", "ombra", "notte", "giorno",
        "vita", "morte", "sogno", "realtà", "momento", "istante",
        "inizio", "fine", "centro", "confine", "limite", "punto",
        // emozioni e stati (forma nominale)
        "paura", "gioia", "dolore", "rabbia", "amore", "odio", "tristezza",
        "quiete", "silenzio", "pace", "caos", "forza", "energia",
        "calore", "freddo", "peso", "vuoto", "pienezza",
        // relazione e identità
        "nome", "parola", "cosa", "idea", "pensiero", "senso", "significato",
        "forma", "figura", "campo", "rete", "nodo", "filo", "legame",
        "radice", "origine", "fonte", "seme", "fiore", "frutto",
        // struttura cognitiva
        "struttura", "schema", "modello", "sistema", "ordine", "insieme",
        "parte", "tutto", "uno", "numero", "grado", "livello",
    ];
    if NOUNS.contains(&word) {
        return Some(PartOfSpeech::Noun);
    }

    // ── Aggettivi: suffissi ad alta affidabilità ─────────────────────────────
    // -ibile/-abile: possibile, amabile, credibile — 99%+
    if len >= 7 && (word.ends_with("ibile") || word.ends_with("abile")) {
        return Some(PartOfSpeech::Adjective);
    }
    // -oso/-osa: famoso, misterioso, gioioso (≥6 evita "viso", "caso")
    if len >= 6 && (word.ends_with("oso") || word.ends_with("osa")) {
        return Some(PartOfSpeech::Adjective);
    }
    // -ivo/-iva: attivo, passivo, creativo (≥6 evita "ivo" da solo)
    if len >= 6 && (word.ends_with("ivo") || word.ends_with("iva")) {
        return Some(PartOfSpeech::Adjective);
    }
    // -ale (≥9 riduce falsi positivi tipo "animale", "canale")
    if len >= 9 && word.ends_with("ale") {
        return Some(PartOfSpeech::Adjective);
    }

    // ── Aggettivi: lista diretta (alta frequenza, forme invariabili o basi) ──
    const ADJECTIVES: &[&str] = &[
        // dimensione fisica
        "grande", "piccolo", "piccola", "piccoli", "piccole",
        "alto", "alta", "alti", "alte", "basso", "bassa", "bassi", "basse",
        "lungo", "lunga", "lunghi", "lunghe", "breve", "brevi",
        "largo", "larga", "larghi", "larghe",
        "pieno", "piena", "pieni", "piene", "vuoto", "vuota", "vuoti", "vuote",
        "aperto", "aperta", "aperti", "aperte",
        "chiuso", "chiusa", "chiusi", "chiuse",
        // qualità sensoriale
        "caldo", "calda", "caldi", "calde",
        "freddo", "fredda", "freddi", "fredde",
        "forte", "forti", "debole", "deboli",
        "dolce", "dolci", "amaro", "amara", "acuto", "acuta",
        "lento", "lenta", "lenti", "lente",
        "duro", "dura", "duri", "dure",
        "morbido", "morbida", "pesante", "leggero", "leggera",
        // valutazione
        "bello", "bella", "belli", "belle",
        "brutto", "brutta", "brutti", "brutte",
        "buono", "buona", "buoni", "buone",
        "cattivo", "cattiva", "cattivi", "cattive",
        "giusto", "giusta", "giusti", "giuste",
        "nuovo", "nuova", "nuovi", "nuove",
        "vecchio", "vecchia", "vecchi", "vecchie",
        "vero", "vera", "veri", "vere",
        "falso", "falsa", "falsi", "false",
        // stati e condizioni
        "vivo", "viva", "vivi", "vive",
        "morto", "morta", "morti", "morte",
        "sicuro", "sicura", "sicuri", "sicure",
        "libero", "libera", "liberi", "libere",
        "facile", "facili", "difficile", "difficili",
        "puro", "pura", "puri", "pure",
        "intero", "intera", "interi", "intere",
        "solo", "sola", "soli", "sole",
        "profondo", "profonda", "profondi", "profonde",
        "oscuro", "oscura", "oscuri", "oscure",
        "luminoso", "luminosa",
        "silenzioso", "silenziosa",
        "antico", "antica", "antichi", "antiche",
        "giovane", "giovani",
        "umano", "umana", "umani", "umane",
        "eterno", "eterna", "eterni", "eterne",
        "infinito", "infinita", "infiniti", "infinite",
        // spaziale
        "vicino", "vicina", "vicini", "vicine",
        "lontano", "lontana", "lontani", "lontane",
        // relazione e grado
        "diverso", "diversa", "diversi", "diverse",
        "uguale", "uguali",
        "stesso", "stessa", "stessi", "stesse",
        "altro", "altra", "altri", "altre",
        "primo", "prima", "primi", "prime",
        "ultimo", "ultima", "ultimi", "ultime",
        "certo", "certa", "certi", "certe",
        "proprio", "propria", "propri", "proprie",
    ];
    if ADJECTIVES.contains(&word) {
        return Some(PartOfSpeech::Adjective);
    }

    None
}

// ─── Genere e Numero ─────────────────────────────────────────────────────────

/// Genere grammaticale italiano.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gender {
    Maschile,
    Femminile,
}

/// Numero grammaticale italiano.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Singolare,
    Plurale,
}

/// Inferisce genere e numero da una parola italiana usando euristica morfologica.
///
/// Precisione ~85-90% su lessico comune. Suffissi prioritari:
///   -o/-i → maschile (sing/plur)
///   -a/-e → femminile (sing/plur)
///   -ione/-zione/-sione → femminile singolare
///   -tà/-ità → femminile singolare
///   -ore/-tore → maschile singolare
///   -ura → femminile singolare
///   -mento/-aggio → maschile singolare
///   -ezza → femminile singolare
/// Ambiguo (termina in -e o -i indistinti) → default maschile singolare.
pub fn detect_gender_number(word: &str) -> (Gender, Number) {
    let len = word.chars().count();
    if len == 0 {
        return (Gender::Maschile, Number::Singolare);
    }

    // Casi speciali ad alta frequenza
    match word {
        "paura" | "gioia" | "tristezza" | "rabbia" | "pace" | "forza" |
        "vita" | "morte" | "mente" | "anima" | "voce" | "notte" | "luce" |
        "terra" | "acqua" | "aria" | "casa" | "mano" | "mani" |
        "realtà" | "identità" | "libertà" | "verità" | "qualità" |
        "presenza" | "assenza" | "speranza" | "distanza" | "fiducia" => {
            // Determina il numero dalla desinenza anche per i casi speciali
            if word.ends_with('i') && !["mani"].contains(&word) {
                return (Gender::Femminile, Number::Plurale);
            }
            return (Gender::Femminile, Number::Singolare);
        }
        "campo" | "tempo" | "spazio" | "mondo" | "sogno" | "corpo" |
        "cuore" | "silenzio" | "sangue" | "pensiero" | "fuoco" | "vento" |
        "calore" | "dolore" | "amore" | "pericolo" | "piacere" |
        "coraggio" | "viaggio" | "paesaggio" | "senso" | "confine" |
        "male" | "bene" | "nome" | "piede" | "dente" | "ponte" | "sole" |
        "mare" | "padre" | "fiore" | "fiume" | "monte" | "leone" | "pane" |
        "timore" | "tremore" | "ardore" | "terrore" | "errore" | "orrore" |
        "valore" | "colore" | "odore" | "sapore" | "sapere" | "volere" => {
            return (Gender::Maschile, Number::Singolare);
        }
        _ => {}
    }

    // Suffissi femminili con alta affidabilità
    if len >= 5 && (word.ends_with("ione") || word.ends_with("zione") || word.ends_with("sione")) {
        return (Gender::Femminile, Number::Singolare);
    }
    // Nomi astratti in -tà / -tù accentata sono femminili (libertà, virtù,
    // gioventù, schiavitù). Regola morfologica, non lista.
    if word.ends_with("tà") || word.ends_with("tù") {
        return (Gender::Femminile, Number::Singolare);
    }
    if len >= 5 && word.ends_with("ezza") {
        return (Gender::Femminile, Number::Singolare);
    }
    if len >= 6 && word.ends_with("tura") {
        return (Gender::Femminile, Number::Singolare);
    }
    if len >= 6 && (word.ends_with("anza") || word.ends_with("enza")) {
        return (Gender::Femminile, Number::Singolare);
    }
    if len >= 5 && word.ends_with("trice") {
        return (Gender::Femminile, Number::Singolare);
    }

    // Suffissi maschili con alta affidabilità
    if len >= 5 && word.ends_with("mento") {
        return (Gender::Maschile, Number::Singolare);
    }
    if len >= 5 && word.ends_with("aggio") {
        return (Gender::Maschile, Number::Singolare);
    }
    if len >= 5 && (word.ends_with("tore") || word.ends_with("sore")) {
        return (Gender::Maschile, Number::Singolare);
    }
    if len >= 5 && word.ends_with("ismo") {
        return (Gender::Maschile, Number::Singolare);
    }
    // -ore: tremore, dolore, calore, ardore, terrore — 95%+ maschile
    if len >= 5 && word.ends_with("ore") {
        return (Gender::Maschile, Number::Singolare);
    }
    // -ere: potere, piacere, volere — maschile singolare
    if len >= 5 && word.ends_with("ere") && !word.ends_with("iere") {
        return (Gender::Maschile, Number::Singolare);
    }

    // Desinenze standard
    let last = word.chars().last().unwrap();
    match last {
        'o' => (Gender::Maschile, Number::Singolare),
        'a' => (Gender::Femminile, Number::Singolare),
        'i' => {
            // -i può essere maschile plurale o femminile plurale
            // Euristica: se penultima è vocale → più spesso femminile (-ai non tipico)
            // Default: maschile plurale (gatti, campi, sogni)
            (Gender::Maschile, Number::Plurale)
        }
        'e' => {
            // -e ambiguo: notte (f), amore (m), torre (f), nome (m)
            // Default: femminile singolare (leggermente più frequente nel lessico comune)
            (Gender::Femminile, Number::Singolare)
        }
        _ => (Gender::Maschile, Number::Singolare),
    }
}

/// Articolo determinativo italiano per una parola.
///
/// Regole di selezione dell'articolo:
///   Maschile singolare:
///     "lo" → davanti a s+consonante, z, gn, ps, x, y, pn
///     "l'" → davanti a vocale (eliso: "l'amore", non "l' amore")
///     "il" → altrimenti
///   Maschile plurale:
///     "gli" → davanti a vocale, s+consonante, z, gn, ps, x, y, pn
///     "i"   → altrimenti
///   Femminile singolare:
///     "l'" → davanti a vocale (eliso)
///     "la" → altrimenti
///   Femminile plurale:
///     "le"  → sempre
pub fn with_definite_article(word: &str) -> String {
    let (gender, number) = detect_gender_number(word);
    let article = italian_definite_article(word, gender, number);
    // "l'" si elide direttamente con la parola (no spazio): "l'amore"
    if article == "l'" {
        format!("{}{}", article, word)
    } else {
        format!("{} {}", article, word)
    }
}

/// Calcola solo l'articolo determinativo (senza la parola).
pub fn definite_article(word: &str) -> &'static str {
    let (gender, number) = detect_gender_number(word);
    italian_definite_article(word, gender, number)
}

/// Preposizione `di` contratta con l'articolo determinativo della parola:
/// del / dello / della / dell' / dei / degli / delle. Restituisce la forma
/// completa "preposizione + parola" (con elisione per dell'). Phase 84 (2b):
/// risolve "di buio" → "del buio", "di futuro" → "del futuro".
pub fn di_articulated(word: &str) -> String {
    let prep = match definite_article(word) {
        "il" => "del",
        "lo" => "dello",
        "la" => "della",
        "l'" => "dell'",
        "i" => "dei",
        "gli" => "degli",
        "le" => "delle",
        _ => "di",
    };
    if prep == "dell'" {
        format!("{}{}", prep, word)
    } else {
        format!("{} {}", prep, word)
    }
}

fn italian_definite_article(word: &str, gender: Gender, number: Number) -> &'static str {
    let starts_vowel = word.chars().next()
        .map(|c| "aeiouàèéìòóùAEIOU".contains(c))
        .unwrap_or(false);

    // Controlla se inizia con s+consonante, z, gn, ps, x, y, pn
    let needs_lo = {
        let mut chars = word.chars();
        match chars.next() {
            Some('s') | Some('S') => {
                // s + consonante → "lo"
                chars.next().map(|c| !"aeiouàèéìòóù".contains(c)).unwrap_or(false)
            }
            Some('z') | Some('Z') | Some('x') | Some('X') | Some('y') | Some('Y') => true,
            Some('g') | Some('G') => {
                // gn → "lo"
                chars.next().map(|c| c == 'n' || c == 'N').unwrap_or(false)
            }
            Some('p') | Some('P') => {
                // ps, pn → "lo"
                chars.next().map(|c| c == 's' || c == 'S' || c == 'n' || c == 'N').unwrap_or(false)
            }
            _ => false,
        }
    };

    match (gender, number) {
        (Gender::Maschile, Number::Singolare) => {
            if starts_vowel { "l'" }
            else if needs_lo { "lo" }
            else { "il" }
        }
        (Gender::Maschile, Number::Plurale) => {
            if starts_vowel || needs_lo { "gli" }
            else { "i" }
        }
        (Gender::Femminile, Number::Singolare) => {
            if starts_vowel { "l'" }
            else { "la" }
        }
        (Gender::Femminile, Number::Plurale) => "le",
    }
}

/// Articolo indeterminativo italiano per una parola.
pub fn with_indefinite_article(word: &str) -> String {
    let (gender, number) = detect_gender_number(word);
    let article = italian_indefinite_article(word, gender, number);
    // "un'" si elide direttamente con la parola (no spazio): "un'essenza"
    if article.ends_with('\'') {
        format!("{}{}", article, word)
    } else {
        format!("{} {}", article, word)
    }
}

fn italian_indefinite_article(word: &str, gender: Gender, number: Number) -> &'static str {
    let starts_vowel = word.chars().next()
        .map(|c| "aeiouàèéìòóùAEIOU".contains(c))
        .unwrap_or(false);

    let needs_uno = {
        let mut chars = word.chars();
        match chars.next() {
            Some('s') | Some('S') => {
                chars.next().map(|c| !"aeiouàèéìòóù".contains(c)).unwrap_or(false)
            }
            Some('z') | Some('Z') | Some('x') | Some('X') | Some('y') | Some('Y') => true,
            Some('g') | Some('G') => chars.next().map(|c| c == 'n' || c == 'N').unwrap_or(false),
            Some('p') | Some('P') => chars.next().map(|c| c == 's' || c == 'S' || c == 'n' || c == 'N').unwrap_or(false),
            _ => false,
        }
    };

    match (gender, number) {
        (Gender::Maschile, Number::Singolare) => {
            // "uno" davanti a s+consonante / z / gn / ps / pn / x / y;
            // "un" altrimenti (incluso davanti a vocale: "un amico").
            if needs_uno && !starts_vowel { "uno" } else { "un" }
        }
        (Gender::Maschile, Number::Plurale) => "dei",
        (Gender::Femminile, Number::Singolare) => {
            if starts_vowel { "un'" } else { "una" }
        }
        (Gender::Femminile, Number::Plurale) => "delle",
    }
}

/// Preposizione articolata: "di" + articolo + parola.
/// Es: with_articulated_preposition("di", "cane") → "del cane"
///     with_articulated_preposition("a", "amore") → "all'amore"
pub fn with_articulated_preposition(prep: &str, word: &str) -> String {
    let (gender, number) = detect_gender_number(word);
    let art = italian_definite_article(word, gender, number);

    // Contrazione preposizione + articolo
    let contracted = match (prep, art) {
        ("di", "il")  => "del".to_string(),
        ("di", "lo")  => "dello".to_string(),
        ("di", "la")  => "della".to_string(),
        ("di", "l'")  => format!("dell'{}", word),
        ("di", "i")   => "dei".to_string(),
        ("di", "gli") => "degli".to_string(),
        ("di", "le")  => "delle".to_string(),
        ("a",  "il")  => "al".to_string(),
        ("a",  "lo")  => "allo".to_string(),
        ("a",  "la")  => "alla".to_string(),
        ("a",  "l'")  => format!("all'{}", word),
        ("a",  "i")   => "ai".to_string(),
        ("a",  "gli") => "agli".to_string(),
        ("a",  "le")  => "alle".to_string(),
        ("da", "il")  => "dal".to_string(),
        ("da", "lo")  => "dallo".to_string(),
        ("da", "la")  => "dalla".to_string(),
        ("da", "l'")  => format!("dall'{}", word),
        ("da", "i")   => "dai".to_string(),
        ("da", "gli") => "dagli".to_string(),
        ("da", "le")  => "dalle".to_string(),
        ("in", "il")  => "nel".to_string(),
        ("in", "lo")  => "nello".to_string(),
        ("in", "la")  => "nella".to_string(),
        ("in", "l'")  => format!("nell'{}", word),
        ("in", "i")   => "nei".to_string(),
        ("in", "gli") => "negli".to_string(),
        ("in", "le")  => "nelle".to_string(),
        ("su", "il")  => "sul".to_string(),
        ("su", "lo")  => "sullo".to_string(),
        ("su", "la")  => "sulla".to_string(),
        ("su", "l'")  => format!("sull'{}", word),
        ("su", "i")   => "sui".to_string(),
        ("su", "gli") => "sugli".to_string(),
        ("su", "le")  => "sulle".to_string(),
        // Preposizioni non contratte (con, per, tra, fra)
        _ => return format!("{} {} {}", prep, art, word),
    };

    // Per l'apostrofo, la parola è già inclusa nella contrazione
    if contracted.contains('\'') {
        contracted
    } else {
        format!("{} {}", contracted, word)
    }
}

/// Inflessione aggettivo per genere e numero.
///
/// Gestisce i quattro pattern principali degli aggettivi italiani:
///   tipo 1 (bello/bella/belli/belle): -o/-a/-i/-e
///   tipo 2 (grande/grandi): -e/-i (invariante per genere)
///   tipo 3 (rosa, blu, viola): invariante (fine vocale/consonante atipica)
pub fn inflect_adjective(adj: &str, gender: Gender, number: Number) -> String {
    // Aggettivi invarianti (non cambiano forma)
    const INVARIANTI: &[&str] = &[
        "rosa", "blu", "viola", "beige", "bordeaux", "lilla", "marrone",
        "arancione", "pari", "dispari", "impari",
    ];
    if INVARIANTI.contains(&adj) {
        return adj.to_string();
    }

    let len = adj.chars().count();
    if len < 2 {
        return adj.to_string();
    }

    // Aggettivi in -e/-i: stessa forma per maschile e femminile
    // grande → grande (sg), grandi (pl)
    if adj.ends_with('e') {
        return match number {
            Number::Singolare => adj.to_string(),
            Number::Plurale => {
                let base = &adj[..adj.len()-1];
                format!("{}i", base)
            }
        };
    }

    // Aggettivi in -o/-a/-i/-e (tipo bello)
    if adj.ends_with('o') {
        let base = &adj[..adj.len()-1];
        return match (gender, number) {
            (Gender::Maschile, Number::Singolare) => adj.to_string(),
            (Gender::Femminile, Number::Singolare) => format!("{}a", base),
            (Gender::Maschile, Number::Plurale)   => format!("{}i", base),
            (Gender::Femminile, Number::Plurale)   => format!("{}e", base),
        };
    }

    // Aggettivi in -a (raro: belga → belga/belghe)
    if adj.ends_with('a') {
        let base = &adj[..adj.len()-1];
        return match (gender, number) {
            (Gender::Maschile, Number::Singolare) => adj.to_string(),
            (Gender::Femminile, Number::Singolare) => adj.to_string(),
            (Gender::Maschile, Number::Plurale)   => format!("{}i", base),
            (Gender::Femminile, Number::Plurale)   => format!("{}he", base),
        };
    }

    // Aggettivi in -i (già plurale, tipo: semplici, facili)
    if adj.ends_with('i') {
        // Se è già plurale, usalo direttamente per le forme plurali
        let base = &adj[..adj.len()-1];
        return match (gender, number) {
            (_, Number::Plurale) => adj.to_string(),
            (Gender::Maschile, Number::Singolare) => format!("{}e", base),
            (Gender::Femminile, Number::Singolare) => format!("{}e", base),
        };
    }

    // Invariante fallback
    adj.to_string()
}

// ─── Test ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conjugate_essere() {
        assert_eq!(conjugate("essere", Person::First, Tense::Present), "sono");
        assert_eq!(conjugate("essere", Person::Third, Tense::Present), "è");
        assert_eq!(conjugate("essere", Person::First, Tense::Imperfect), "ero");
        assert_eq!(conjugate("essere", Person::Third, Tense::Imperfect), "era");
        assert_eq!(conjugate("essere", Person::First, Tense::Future), "sarò");
        assert_eq!(conjugate("essere", Person::First, Tense::Conditional), "sarei");
    }

    #[test]
    fn test_conjugate_avere() {
        assert_eq!(conjugate("avere", Person::First, Tense::Present), "ho");
        assert_eq!(conjugate("avere", Person::Third, Tense::Present), "ha");
        assert_eq!(conjugate("avere", Person::ThirdPlural, Tense::Present), "hanno");
    }

    #[test]
    fn test_conjugate_regular_are() {
        assert_eq!(conjugate("amare", Person::First, Tense::Present), "amo");
        assert_eq!(conjugate("amare", Person::Second, Tense::Present), "ami");
        assert_eq!(conjugate("amare", Person::Third, Tense::Present), "ama");
        assert_eq!(conjugate("amare", Person::FirstPlural, Tense::Present), "amiamo");
        assert_eq!(conjugate("amare", Person::First, Tense::Imperfect), "amavo");
        assert_eq!(conjugate("amare", Person::Third, Tense::Imperfect), "amava");
        assert_eq!(conjugate("amare", Person::First, Tense::Future), "amerò");
        assert_eq!(conjugate("parlare", Person::First, Tense::Present), "parlo");
        assert_eq!(conjugate("sentire", Person::First, Tense::Present), "sento");
        assert_eq!(conjugate("correre", Person::First, Tense::Present), "corro");
    }

    #[test]
    fn test_conjugate_regular_ere() {
        assert_eq!(conjugate("credere", Person::First, Tense::Present), "credo");
        assert_eq!(conjugate("credere", Person::Third, Tense::Present), "crede");
        assert_eq!(conjugate("credere", Person::First, Tense::Imperfect), "credevo");
        assert_eq!(conjugate("credere", Person::First, Tense::Future), "crederò");
    }

    #[test]
    fn test_conjugate_regular_ire_dormire() {
        assert_eq!(conjugate("dormire", Person::First, Tense::Present), "dormo");
        assert_eq!(conjugate("dormire", Person::Third, Tense::Present), "dorme");
        assert_eq!(conjugate("dormire", Person::First, Tense::Imperfect), "dormivo");
    }

    #[test]
    fn test_conjugate_regular_ire_finire() {
        assert_eq!(conjugate("finire", Person::First, Tense::Present), "finisco");
        assert_eq!(conjugate("finire", Person::Second, Tense::Present), "finisci");
        assert_eq!(conjugate("finire", Person::Third, Tense::Present), "finisce");
        assert_eq!(conjugate("capire", Person::First, Tense::Present), "capisco");
    }

    #[test]
    fn test_lemma_candidates_nome_plurale() {
        // -i → -e / -o / -a : il candidato giusto è fra quelli generati (gate §11)
        assert!(lemma_candidates("accidenti").contains(&"accidente".to_string()));
        assert!(lemma_candidates("api").contains(&"ape".to_string()));
        assert!(lemma_candidates("libri").contains(&"libro".to_string()));
        // -e → -a (femminile plurale)
        assert!(lemma_candidates("aquile").contains(&"aquila".to_string()));
        assert!(lemma_candidates("case").contains(&"casa".to_string()));
    }

    #[test]
    fn test_kg_validated_lemma_onesto_no_trucchi() {
        // DISAMBIGUO: un solo candidato nel dizionario → lemmatizza.
        let solo_nomi = |w: &str| ["cane", "idea", "opera", "amare"].contains(&w);
        assert_eq!(kg_validated_lemma("cani", solo_nomi), "cane");
        assert_eq!(kg_validated_lemma("idee", solo_nomi), "idea");
        assert_eq!(kg_validated_lemma("opere", solo_nomi), "opera");
        assert_eq!(kg_validated_lemma("amano", solo_nomi), "amare"); // verbo confermato

        // AMBIGUO: "mondare"/"possibilare" SONO nel dizionario (verbi reali) →
        // NON si indovina (niente trucco di distanza): si DEFERISCE alla
        // superficie, deciderà il contesto (analisi logica clausa-aware).
        let con_verbi = |w: &str| {
            ["mondo", "mondare", "possibile", "possibilare"].contains(&w)
        };
        assert_eq!(kg_validated_lemma("mondi", con_verbi), "mondi");
        assert_eq!(kg_validated_lemma("possibili", con_verbi), "possibili");

        // Già lemma → invariata. Zero conferme → superficie, MAI infinito.
        assert_eq!(kg_validated_lemma("mondo", con_verbi), "mondo");
        assert_eq!(kg_validated_lemma("mondi", |_| false), "mondi");
    }

    #[test]
    fn test_kg_validated_nominal_disambigua_per_ruolo() {
        // Stesso dizionario del test sopra: "mondo" E "mondare" entrambi noti.
        // Ma SAPENDO che il token è un argomento (ruolo nominale), si generano
        // SOLO candidati nome → "mondare" non è nemmeno considerato → "mondi"
        // si riduce a "mondo" senza ambiguità. È la disambiguazione PER RUOLO,
        // non un trucco morfologico: il chunker ha già detto "questo è un nome".
        let con_verbi = |w: &str| {
            ["mondo", "mondare", "possibile", "possibilare"].contains(&w)
        };
        assert_eq!(kg_validated_nominal("mondi", con_verbi), "mondo");
        assert_eq!(kg_validated_nominal("possibili", con_verbi), "possibile");
        // niente candidati verbali: nominal_lemma_candidates non produce infiniti
        assert!(!nominal_lemma_candidates("mondi").iter().any(|c| c.ends_with("are")));
        // zero conferme → superficie (mai infinito inventato)
        assert_eq!(kg_validated_nominal("mondi", |_| false), "mondi");
    }

    #[test]
    fn test_kg_validated_nominal_gendered_articolo() {
        // "gatto" E "gatta" entrambi nel KG → senza genere, "gatti" è ambiguo
        // (gatto vs gatta) → deferisce. Col genere dall'articolo si scioglie.
        let dict = |w: &str| ["gatto", "gatta", "arma", "armo"].contains(&w);
        // senza genere: ambiguo → superficie
        assert_eq!(kg_validated_nominal("gatti", dict), "gatti");
        // "i/gli gatti" → maschile → mai -a → solo "gatto" confermato → riduce
        assert_eq!(kg_validated_nominal_gendered("gatti", Some(true), dict), "gatto");
        // "le armi" → femminile → mai -o → solo "arma" confermato → riduce
        assert_eq!(kg_validated_nominal_gendered("armi", Some(false), dict), "arma");
        // plurale -e: "le piante" → femminile → "pianta" (non l'ambiguo "pianto")
        let dict2 = |w: &str| ["pianta", "pianto"].contains(&w);
        assert_eq!(kg_validated_nominal("piante", dict2), "piante"); // ambiguo senza genere
        assert_eq!(kg_validated_nominal_gendered("piante", Some(false), dict2), "pianta");
        // genere ignoto (None) = comportamento invariato (deferisce)
        assert_eq!(kg_validated_nominal_gendered("gatti", None, dict), "gatti");
        // mai infiniti inventati e mai conferme dal nulla
        assert_eq!(kg_validated_nominal_gendered("gatti", Some(true), |_| false), "gatti");
    }

    #[test]
    fn test_lemma_candidates_aggettivo() {
        // superlativo assoluto → base
        assert!(lemma_candidates("altissimo").contains(&"alto".to_string()));
        assert!(lemma_candidates("agitatissimo").contains(&"agitato".to_string()));
        assert!(lemma_candidates("grandissimo").contains(&"grande".to_string()));
        // femminile → maschile (base)
        assert!(lemma_candidates("ampia").contains(&"ampio".to_string()));
    }

    #[test]
    fn test_lemma_candidates_verbo_finito() {
        // 1ª singolare presente -o
        assert!(lemma_candidates("abbacchio").contains(&"abbacchiare".to_string()));
        // gerundio
        assert!(lemma_candidates("amando").contains(&"amare".to_string()));
        assert!(lemma_candidates("agendo").contains(&"agire".to_string()));
        // 3ª singolare presente -ere
        assert!(lemma_candidates("affligge").contains(&"affliggere".to_string()));
        // participio irregolare (coerente con lemmatize)
        assert!(lemma_candidates("accolta").contains(&"accogliere".to_string()));
    }

    #[test]
    fn test_lemma_candidates_non_riduce_se_stesso() {
        // la forma non deve mai comparire fra i propri candidati
        assert!(!lemma_candidates("casa").contains(&"casa".to_string()));
        assert!(!lemma_candidates("amare").contains(&"amare".to_string()));
    }

    #[test]
    fn test_lemmatize_imperfect() {
        let r = lemmatize("sentivo").unwrap();
        assert_eq!(r.infinitive, "sentire");
        assert_eq!(r.tense, Tense::Imperfect);
        assert_eq!(r.person, Person::First);

        let r = lemmatize("correvo").unwrap();
        assert_eq!(r.infinitive, "correre");
        assert_eq!(r.tense, Tense::Imperfect);

        let r = lemmatize("parlavo").unwrap();
        assert_eq!(r.infinitive, "parlare");
        assert_eq!(r.tense, Tense::Imperfect);

        let r = lemmatize("affermavo").unwrap();
        assert_eq!(r.infinitive, "affermare");
    }

    #[test]
    fn test_lemmatize_irregular() {
        let r = lemmatize("sono").unwrap();
        assert_eq!(r.infinitive, "essere");

        let r = lemmatize("ho").unwrap();
        assert_eq!(r.infinitive, "avere");

        let r = lemmatize("faccio").unwrap();
        assert_eq!(r.infinitive, "fare");

        let r = lemmatize("voglio").unwrap();
        assert_eq!(r.infinitive, "volere");

        let r = lemmatize("posso").unwrap();
        assert_eq!(r.infinitive, "potere");
    }

    #[test]
    fn test_lemmatize_finire_type() {
        let r = lemmatize("finisco").unwrap();
        assert_eq!(r.infinitive, "finire");

        let r = lemmatize("capisce").unwrap();
        assert_eq!(r.infinitive, "capire");
    }

    #[test]
    fn test_lemmatize_non_verb() {
        assert!(lemmatize("bello").is_none());
        assert!(lemmatize("casa").is_none());
        assert!(lemmatize("felice").is_none());
        assert!(lemmatize("io").is_none());
    }

    #[test]
    fn test_lemmatize_participio_irregolare() {
        // Phase 86 #3
        assert_eq!(lemmatize("preso").unwrap().infinitive, "prendere");
        assert_eq!(lemmatize("presa").unwrap().infinitive, "prendere");
        assert_eq!(lemmatize("scritto").unwrap().infinitive, "scrivere");
        assert_eq!(lemmatize("accolto").unwrap().infinitive, "accogliere");
        assert_eq!(lemmatize("aperto").unwrap().infinitive, "aprire");
    }

    #[test]
    fn test_lemmatize_enclitico_riflessivo() {
        // Phase 86 §2-bis
        assert_eq!(lemmatize("abbandonarsi").unwrap().infinitive, "abbandonare");
        assert_eq!(lemmatize("andarci").unwrap().infinitive, "andare");
        assert_eq!(lemmatize("abbellirsi").unwrap().infinitive, "abbellire");
        // guard anti-falso-positivo: "corsi"→"cor"→"core"(ore) NON è infinito → rifiutato dall'enclitico
        assert_ne!(lemmatize("corsi").map(|r| r.infinitive), Some("core".to_string()));
    }

    #[test]
    fn test_detect_pos() {
        // Verbi (infinito)
        assert_eq!(detect_pos_from_word("sentire"), Some(PartOfSpeech::Verb));
        assert_eq!(detect_pos_from_word("correre"), Some(PartOfSpeech::Verb));
        assert_eq!(detect_pos_from_word("amare"), Some(PartOfSpeech::Verb));
        // Aggettivi (lista diretta)
        assert_eq!(detect_pos_from_word("bello"), Some(PartOfSpeech::Adjective));
        // Aggettivi (suffisso -oso)
        assert_eq!(detect_pos_from_word("famoso"), Some(PartOfSpeech::Adjective));
        // Aggettivi (suffisso -ibile)
        assert_eq!(detect_pos_from_word("possibile"), Some(PartOfSpeech::Adjective));
        // Sostantivi (lista diretta)
        assert_eq!(detect_pos_from_word("corpo"), Some(PartOfSpeech::Noun));
        assert_eq!(detect_pos_from_word("acqua"), Some(PartOfSpeech::Noun));
        // Sostantivi (suffisso -zione)
        assert_eq!(detect_pos_from_word("emozione"), Some(PartOfSpeech::Noun));
        // Sostantivi (suffisso -tà)
        assert_eq!(detect_pos_from_word("libertà"), Some(PartOfSpeech::Noun));
        // Avverbi (lista diretta)
        assert_eq!(detect_pos_from_word("dentro"), Some(PartOfSpeech::Adverb));
        assert_eq!(detect_pos_from_word("abbastanza"), Some(PartOfSpeech::Adverb));
        // Avverbi (suffisso -mente)
        assert_eq!(detect_pos_from_word("rapidamente"), Some(PartOfSpeech::Adverb));
        // Pronomi
        assert_eq!(detect_pos_from_word("io"), Some(PartOfSpeech::Pronoun));
        assert_eq!(detect_pos_from_word("noi"), Some(PartOfSpeech::Pronoun));
        // Nessun tag (parola ambigua breve o non classificabile)
        assert_eq!(detect_pos_from_word("casa"), None);  // non in lista diretta, nessun suffisso
    }
}
