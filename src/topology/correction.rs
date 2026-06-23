//! Phase 84 — la **correzione** come segnale strutturale.
//!
//! Un endpoint dell'engine: l'utente comunica "avresti dovuto dire questo,
//! magari in *questo contesto*". UI-r1 NON registra la stringa come template.
//! Estrae le parole-contenuto, le confronta col KG, modifica triple e
//! confidence. Il prossimo turno con input simile produce naturalmente la
//! risposta corretta perche' il campo e' cambiato — niente if/then.
//!
//! Due livelli operativi:
//!
//! 1. **Con `via_context`** (l'utente specifica il perche'): aggiunge triple
//!    specializzate `parola IsA categoria via context` con confidence 1.0.
//!    Lascia intatto il significato generico — "ciao" e "benvenuto" sono
//!    entrambi saluti, ma con *via* diverse.
//!
//! 2. **Senza `via_context`** (fallback): alza la confidence delle triple
//!    `IsA` delle parole preferite (in `wanted`, non in `given`) di `+δ`,
//!    abbassa quelle delle parole evitate (in `given`, non in `wanted`) di
//!    `-δ` con floor 0.05. Mai a zero — le parole evitate restano usabili
//!    in altri contesti.
//!
//! In entrambi i casi: registra un `CorrectionFact` nello SpeakerProfile per
//! memoria narrativa, e crea il nodo della via se non esiste.

use crate::topology::engine::PrometeoTopologyEngine;
use crate::topology::input_reading::is_kg_proc_function_word;
use crate::topology::relation::RelationType;
use crate::topology::speaker_profile::CorrectionFact;

/// Forza con cui la correzione modula le confidence in modalita' fallback
/// (senza via). 0.15 = 3 correzioni saturano un pattern stabile.
const CONFIDENCE_DELTA: f32 = 0.15;
/// Confidence assegnata alle nuove triple specializzate (con via).
const NEW_TRIPLE_CONFIDENCE: f32 = 1.0;
/// Floor sotto cui non scende mai la confidence di un arco esistente:
/// una parola evitata oggi puo' essere richiamata domani in altro contesto.
const CONFIDENCE_FLOOR: f32 = 0.05;

/// Risultato di una correzione. Descrive cosa e' cambiato perche' l'utente
/// possa vederlo (e il frontend possa visualizzare l'effetto).
#[derive(Debug, Clone, Default)]
pub struct CorrectionResult {
    /// Parole-contenuto preferite (estratte da `wanted`, non in `given`).
    pub positive_words: Vec<String>,
    /// Parole-contenuto evitate (in `given`, non in `wanted`).
    pub negative_words: Vec<String>,
    /// Categorie semantiche (IS_A target) toccate dalla correzione.
    pub categories_affected: Vec<String>,
    /// Parole nuove create nel KG (es. il `via_context` non esisteva).
    /// Il frontend usa questa lista per aprire il modale "spiegami X"
    /// dopo la correzione, in modo che l'utente possa educare il nuovo nodo.
    pub new_words_created: Vec<String>,
    /// Triple aggiunte (forma human-readable per ispezione/debug).
    pub triples_added: Vec<String>,
    /// Triple il cui peso e' stato modificato (vecchio → nuovo).
    pub confidences_changed: Vec<String>,
    /// Messaggio sintetico in italiano leggibile sull'esito.
    pub message: String,
}

/// Applica una correzione al sistema. Vedi doc del modulo per la semantica.
///
/// - `input`: cio' che l'utente aveva detto (es. "ciao")
/// - `given`: la risposta che UI-r1 ha prodotto (es. "Benvenuto.")
/// - `wanted`: la risposta che l'utente avrebbe voluto (es. "Ciao!")
/// - `via_context`: parola/breve frase opzionale che specifica il contesto
///   pragmatico (es. "amico"). Se presente, sara' usata come `via` delle
///   triple specializzate.
pub fn apply_correction(
    engine: &mut PrometeoTopologyEngine,
    input: &str,
    given: &str,
    wanted: &str,
    via_context: Option<&str>,
) -> CorrectionResult {
    let mut result = CorrectionResult::default();

    // --- 1. Estrazione parole-contenuto -------------------------------------
    let kg_proc = Some(&engine.kg_procedural);
    let wanted_words = content_words(wanted, kg_proc);
    let given_words  = content_words(given,  kg_proc);

    // Preferite = in wanted, non in given.
    let positive: Vec<String> = wanted_words.iter()
        .filter(|w| !given_words.iter().any(|g| g == *w))
        .cloned().collect();
    // Evitate = in given, non in wanted.
    let negative: Vec<String> = given_words.iter()
        .filter(|w| !wanted_words.iter().any(|p| p == *w))
        .cloned().collect();

    result.positive_words = positive.clone();
    result.negative_words = negative.clone();

    // --- 2. Estrazione del via_word dal contesto ----------------------------
    // Prendiamo la PRIMA parola-contenuto del contesto, se fornito.
    let via_word: Option<String> = via_context
        .map(|ctx| content_words(ctx, kg_proc).into_iter().next())
        .flatten();

    // --- 3. Branch principale: con-via vs fallback-confidence ----------------
    let mut categories: Vec<String> = Vec::new();

    if let Some(via) = via_word.as_ref() {
        // Se la via non esiste come nodo del KG (nessun arco in/out), la
        // creiamo aggiungendo un arco placeholder `via IsA contesto`. Il
        // frontend usera' `new_words_created` per chiedere all'utente di
        // spiegarla subito dopo.
        if !engine.kg.has_node(via) {
            engine.kg.add(via, RelationType::IsA, "contesto");
            result.new_words_created.push(via.clone());
            result.triples_added.push(format!("{} IsA contesto", via));
        }

        // Per ogni parola preferita, aggiungi triple `parola IsA cat via via_word`.
        for w in &positive {
            // Cerca le IS_A esistenti per dedurre la categoria.
            let parents: Vec<String> = engine.kg
                .query_objects(w, RelationType::IsA)
                .into_iter().map(|s| s.to_string()).collect();
            if parents.is_empty() {
                // Nessuna IS_A: aggiungiamo almeno `w IsA contesto via via_word`
                // come ancoraggio minimo. Sarebbe meglio se l'utente educasse `w`.
                engine.kg.add_via(w, RelationType::IsA, "contesto", via,
                                   NEW_TRIPLE_CONFIDENCE);
                result.triples_added.push(format!("{} IsA contesto via {}", w, via));
            } else {
                for cat in &parents {
                    // Triple specializzata con via: memoria di "in questo
                    // contesto, questa e' la parola giusta".
                    engine.kg.add_via(w, RelationType::IsA, cat, via,
                                      NEW_TRIPLE_CONFIDENCE);
                    result.triples_added.push(format!("{} IsA {} via {}", w, cat, via));
                    // ALLO STESSO TEMPO modula la confidence generica:
                    // senza questa, il pattern matcher continuerebbe a usare
                    // l'anchor vecchia (es. "benvenuto") perche' la sua
                    // triple senza via vince per scoring naturale. Solo
                    // alzando "w IsA cat" diamo effetto immediato.
                    if let Some((old, new)) = bump_confidence(engine, w, cat, CONFIDENCE_DELTA) {
                        result.confidences_changed.push(
                            format!("{} IsA {}: {:.2} -> {:.2}", w, cat, old, new));
                    }
                    if !categories.contains(cat) { categories.push(cat.clone()); }
                }
            }
        }
        // E abbassiamo la confidence delle parole evitate sulle stesse categorie.
        // La parola non sparisce — resta usabile in altri contesti.
        for w in &negative {
            let parents: Vec<String> = engine.kg
                .query_objects(w, RelationType::IsA)
                .into_iter().map(|s| s.to_string()).collect();
            for cat in &parents {
                if !categories.contains(cat) { continue; }
                if let Some((old, new)) = bump_confidence(engine, w, cat, -CONFIDENCE_DELTA) {
                    result.confidences_changed.push(
                        format!("{} IsA {}: {:.2} -> {:.2}", w, cat, old, new));
                }
            }
        }
        result.message = format!(
            "Capito. {} {} il modo {} di {} (ho aggiunto {} triple{}).",
            positive.first().map(|s| format!("'{}'", s)).unwrap_or_else(|| "questo".to_string()),
            if positive.len() == 1 { "e'" } else { "sono" },
            via,
            categories.first().cloned().unwrap_or_else(|| "rispondere".to_string()),
            result.triples_added.len(),
            if result.confidences_changed.is_empty() {
                String::new()
            } else {
                format!(", aggiustate {} confidence", result.confidences_changed.len())
            },
        );
    } else {
        // Fallback senza via: modula confidence delle triple IS_A esistenti.
        for w in &positive {
            let parents: Vec<String> = engine.kg
                .query_objects(w, RelationType::IsA)
                .into_iter().map(|s| s.to_string()).collect();
            for cat in &parents {
                if let Some((old, new)) = bump_confidence(engine, w, cat, CONFIDENCE_DELTA) {
                    result.confidences_changed.push(
                        format!("{} IsA {}: {:.2} -> {:.2}", w, cat, old, new));
                    if !categories.contains(cat) { categories.push(cat.clone()); }
                }
            }
        }
        for w in &negative {
            let parents: Vec<String> = engine.kg
                .query_objects(w, RelationType::IsA)
                .into_iter().map(|s| s.to_string()).collect();
            for cat in &parents {
                if let Some((old, new)) = bump_confidence(engine, w, cat, -CONFIDENCE_DELTA) {
                    result.confidences_changed.push(
                        format!("{} IsA {}: {:.2} -> {:.2}", w, cat, old, new));
                    if !categories.contains(cat) { categories.push(cat.clone()); }
                }
            }
        }
        result.message = if result.confidences_changed.is_empty() {
            "Non avevo abbastanza informazioni per imparare da questa correzione.".to_string()
        } else {
            format!("Ho aggiustato {} relazioni nel mio modo di rispondere.",
                    result.confidences_changed.len())
        };
    }

    result.categories_affected = categories;

    // --- 4. Registra il fatto narrativo nello SpeakerProfile -----------------
    let turn = engine.speaker_profile.turn_count;
    engine.speaker_profile.register_correction(CorrectionFact {
        turn,
        input: input.to_string(),
        given: given.to_string(),
        wanted: wanted.to_string(),
        via_context: via_word,
        positive_words: result.positive_words.clone(),
        negative_words: result.negative_words.clone(),
    });

    result
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

/// Tokenizza una stringa libera in parole-contenuto pulite.
///
/// - split su whitespace + punteggiatura
/// - lowercase
/// - filtra via parole-funzione (pronomi, articoli, preposizioni…) via kg_proc
fn content_words(text: &str, kg_proc: Option<&crate::topology::knowledge_graph::KnowledgeGraph>)
    -> Vec<String>
{
    let mut out: Vec<String> = Vec::new();
    for raw in text.split(|c: char| !c.is_alphabetic() && c != '\'') {
        let w = raw.trim_matches('\'').to_lowercase();
        if w.len() < 2 { continue; }
        if is_kg_proc_function_word(&w, kg_proc) { continue; }
        if !out.iter().any(|x| x == &w) { out.push(w); }
    }
    out
}

/// Aggiusta la confidence di una specifica triple `subject IsA object` di `delta`,
/// cappata in `[CONFIDENCE_FLOOR, 1.0]`. Ritorna `(old, new)` se la triple
/// esisteva, `None` altrimenti.
fn bump_confidence(
    engine: &mut PrometeoTopologyEngine,
    subject: &str,
    object: &str,
    delta: f32,
) -> Option<(f32, f32)> {
    let old = engine.kg.edge_confidence(subject, RelationType::IsA, object)?;
    let mut new = old + delta;
    if new > 1.0 { new = 1.0; }
    if new < CONFIDENCE_FLOOR { new = CONFIDENCE_FLOOR; }
    if (new - old).abs() < 0.0001 { return None; }
    engine.kg.update_confidence(subject, RelationType::IsA, object, new);
    Some((old, new))
}
