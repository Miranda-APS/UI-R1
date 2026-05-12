// ─── expression.rs ────────────────────────────────────────────────────────────
// Phase 57 — Entità che parla dal suo stato.
//
// Tre strati ordinati:
//   1. INTELLIGERE: i nuclei semantici KG sono la COMPRENSIONE interna dell'entità.
//      Non sono l'output — sono ciò che l'entità ha capito.
//
//   2. COLORAZIONE OCTALYSIS: i drive Valenza [CD1-CD8] colorano la selezione
//      delle parole verso le dimensioni del campo più risonanti con lo stato interno.
//      Non template, non frame — pesi continui sulla firma 8D delle parole.
//      Stessa logica di top_active_word() in state_translation.rs.
//
//   3. EXPRIMERE: la grammatica italiana è la fisica del mondo — vincolo, non gabbia.
//      Nessun archetype, nessuno slot fisso. Il campo parla attraverso la grammatica.
//
// Flusso:
//   campo + KG → comprensione (nuclei) → colorati da Octalysis → grammatica → testo
// ──────────────────────────────────────────────────────────────────────────────

use crate::topology::grammar::{self, Person, Tense, PartOfSpeech, with_definite_article, with_indefinite_article};
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::lexicon::Lexicon;
use crate::topology::relation::RelationType;
use crate::topology::semantic_episode::SemanticEpisodeLog;
use crate::topology::valence::DRIVE_DIM;
use crate::topology::word_topology::WordTopology;

// ─── Nucleo semantico: relazione tra due parole attive ─────────────────────

/// Un nucleo semantico è la più piccola unità di significato relazionale:
/// due parole del campo collegate da una relazione nel KG.
/// È il "fatto" che l'entità ha compreso.
#[derive(Debug, Clone)]
pub struct SemanticNucleus {
    pub subject: String,
    pub relation: RelationType,
    pub object: String,
    /// Forza combinata: sqrt(act_subject × act_object) × confidence_arco
    pub strength: f64,
    /// Attivazione soggetto nel campo
    pub subject_activation: f64,
    /// Attivazione oggetto nel campo
    pub object_activation: f64,
    /// Vicinanza all'input: 4.0=entrambi intorno input, 2.0=uno vicino, 0.2=nessuna connessione
    pub proximity_score: f64,
}

// ─── Voce dell'entità: come si esprime ─────────────────────────────────────

/// La voce emerge dallo stato interno — non è scelta, è sentita.
#[derive(Debug, Clone)]
pub struct EntityVoice {
    pub person: Person,
    pub tense: Tense,
    pub mood: ExpressionMood,
}

/// L'umore espressivo — non il contenuto, ma il COME.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpressionMood {
    /// L'entità afferma qualcosa che sente/sa
    Declarative,
    /// L'entità osserva qualcosa nel suo campo
    Observational,
    /// L'entità si chiede qualcosa
    Interrogative,
    /// L'entità esplora, non sa ancora
    Explorative,
    /// L'entità è in silenzio — il campo è quasi vuoto
    Silent,
}

/// Risultato della composizione emergente.
pub struct Expression {
    pub text: String,
    pub words_used: Vec<String>,
}

// ─── Colorazione Octalysis ──────────────────────────────────────────────────

/// Calcola il peso Octalysis di una parola dato lo stato dei drive.
///
/// Stessa logica di `top_active_word()` in state_translation.rs:
///   affinity = Σ (drive_strength × firma_8D[dim_del_drive])
///   boost = 1.0 + affinity × 0.25
///
/// NON è un template: è il campo topologico dell'entità che colora
/// la selezione — parole la cui firma 8D risuona con i drive attivi
/// emergono naturalmente, senza che nessuna regola le prescriva.
fn valence_weight(word: &str, valence_drives: &[f64; 8], lexicon: &Lexicon) -> f64 {
    let default_sig = [0.5f64; 8];
    let sig = lexicon.get(word)
        .map(|p| p.signature.values())
        .unwrap_or(&default_sig);
    let mut affinity = 0.0f64;
    for cd in 0..8 {
        let drive_strength = valence_drives[cd].abs();
        if drive_strength > 0.1 {
            affinity += drive_strength * sig[DRIVE_DIM[cd]];
        }
    }
    1.0 + affinity * 0.25  // boost gentile, max ~1.5 con drive saturi
}

// ─── API pubblica ──────────────────────────────────────────────────────────

/// Compone un'espressione emergente dal campo attivo dell'entità.
///
/// Questa funzione NON usa template. La struttura della frase emerge da:
///   - quali parole sono attive e come si relazionano (nuclei semantici)
///   - come l'entità si sente (voce = valenza + frattale + stato vitale)
///   - le regole della grammatica italiana (vincolo di comprensibilità)
// ─── Parole-stato per ogni drive Octalysis ─────────────────────────────────
// Coppie (positivo, negativo) per CD1-CD8.
// Non sono template — sono i lemmi che Prometeo usa per NOMINARE il proprio stato.
// Il campo li sceglie in base a quale drive è dominante e alla sua polarità.
const DRIVE_STATE_WORDS: [(&str, &str); 8] = [
    ("scopo",       "vuoto"),        // CD1 Epic Meaning
    ("capacità",    "limite"),       // CD2 Development & Accomplishment
    ("curiosità",   "incertezza"),   // CD3 Creativity & Empowerment
    ("stabilità",   "deriva"),       // CD4 Ownership & Possession
    ("connessione", "solitudine"),   // CD5 Social Influence & Relatedness
    ("urgenza",     "calma"),        // CD6 Scarcity & Impatience
    ("sorpresa",    "quiete"),       // CD7 Unpredictability & Curiosity
    ("cautela",     "inquietudine"), // CD8 Loss & Avoidance
];

/// Genera un'espressione in prima persona dallo stato interno dei drive.
/// Usata quando l'input chiede esplicitamente lo stato dell'entità.
/// Sceglie il drive più intenso e nomina la parola-stato corrispondente.
fn express_from_drives(drives: &[f64; 8], lexicon: &Lexicon) -> Option<Expression> {
    // Trova il drive più intenso (positivo o negativo)
    let (dominant_cd, dominant_val) = drives.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap_or(std::cmp::Ordering::Equal))?;

    if dominant_val.abs() < 0.08 {
        return None; // Nessuno stato abbastanza definito
    }

    let (pos_word, neg_word) = DRIVE_STATE_WORDS[dominant_cd];
    let state_word = if *dominant_val >= 0.0 { pos_word } else { neg_word };

    // Controlla che la parola-stato esista nel lessico, altrimenti cerca sinonimi nel campo
    let known = lexicon.get(state_word).is_some();
    let word = if known { state_word } else { return None; };

    // Cerca un secondo drive significativo per arricchire l'espressione
    let second = drives.iter()
        .enumerate()
        .filter(|(cd, v)| *cd != dominant_cd && v.abs() > 0.12)
        .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap_or(std::cmp::Ordering::Equal));

    let text = if let Some((cd2, val2)) = second {
        let (pos2, neg2) = DRIVE_STATE_WORDS[cd2];
        let w2 = if *val2 >= 0.0 { pos2 } else { neg2 };
        if lexicon.get(w2).is_some() {
            format!("Sento {} e {}.", word, w2)
        } else {
            format!("Sento {}.", word)
        }
    } else {
        format!("Sento {}.", word)
    };

    Some(Expression {
        text,
        words_used: vec![word.to_string()],
    })
}

/// Phase 71: risposta a una parola, scelta dal campo VIVO.
///
/// Quando la `Deliberation` decide `ActionShape::Word` (un saluto reciproco,
/// una conferma, un congedo) la risposta è una sola parola. Non è uno
/// shortcut: la SCELTA della parola viene dallo stato corrente del campo,
/// che ha già integrato identità, narrativa, sibling-activation, desiderio,
/// interlocutore. Si seleziona la parola più viva fra le candidate
/// (escluso l'eco dell'input). Preferenza ai fratelli IsA dell'input
/// (`comprehension.region_siblings`) — la regione semantica.
fn compose_word_response(
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    echo_exclude: &[String],
    deliberation: &crate::topology::deliberation::Deliberation,
) -> Option<Expression> {
    let active = word_topology.active_words();
    if active.is_empty() { return None; }

    // Phase 74: la regione preferita è data dalle anchor_words (decise da
    // action_reasoning) se presenti, altrimenti dai region_siblings del
    // comprehension graph. Le anchor_words riflettono la decisione esplicita;
    // i region_siblings sono materiale grezzo della regione semantica.
    let region: std::collections::HashSet<&str> = if !deliberation.anchor_words.is_empty() {
        deliberation.anchor_words.iter().map(|s| s.as_str()).collect()
    } else {
        deliberation.comprehension.region_siblings.iter().map(|s| s.as_str()).collect()
    };

    let mut best: Option<(String, f64)> = None;
    for (w, act_val) in active.iter() {
        if echo_exclude.iter().any(|e| e == w) { continue; }
        if w.chars().count() < 3 { continue; }
        let pat = match lexicon.get(w) {
            Some(p) => p,
            None => continue,
        };
        if pat.stability < 0.25 { continue; }
        if pat.exposure_count < 3 { continue; }
        let in_region = region.contains(*w);
        // Boost per fratelli della regione (preferiamo parole della stessa
        // classe semantica). Boost identitario: se la parola allinea con
        // i drive correnti la scegliamo più volentieri.
        let drives = &deliberation.identity_now.current_drives;
        let sig = pat.signature.values();
        let mut id_align = 0.0_f64;
        for i in 0..8 { id_align += drives[i] * sig[i]; }
        let region_boost = if in_region { 1.6_f64 } else { 1.0 };
        let score = (*act_val) * region_boost * (1.0 + 0.20 * id_align);
        if best.as_ref().map_or(true, |(_, b)| score > *b) {
            best = Some((w.to_string(), score));
        }
    }

    best.map(|(word, _)| {
        let mut chars = word.chars();
        let head = chars.next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
        let text = format!("{}{}.", head, chars.as_str());
        Expression { text, words_used: vec![word] }
    })
}

pub fn compose(
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    kg: &KnowledgeGraph,
    echo_exclude: &[String],
    valence_drives: &[f64; 8],
    active_fractals: &[(u32, f64)],
    codon: [usize; 2],
    input_words: &[String],
    episodes: Option<&SemanticEpisodeLog>,
    is_question: bool,
    // Phase 62: l'Altro ha espresso distress emotivo (tristezza/paura/dolore).
    // Attiva voce in seconda persona + modo interrogativo: apre spazio empatico.
    other_in_distress: bool,
    // Phase 67: l'intenzione deliberata da NarrativeSelf colora la voce.
    // Resonate → 2a persona, Explore → interrogativo, Remain → singola parola.
    response_intention: Option<&str>,
    // Phase 71: la Deliberation completa, se costruita. Quando presente,
    // `action_shape` decide la FORMA della risposta — Silenzio, Parola,
    // Domanda, EcoEmpatica, Frase. Non è uno shortcut: è il cap visibile
    // del ciclo soggettivo che ha già letto identità, narrativa, desiderio.
    deliberation: Option<&crate::topology::deliberation::Deliberation>,
    // Phase 77: KG procedurale + ActionDecision per il pattern matcher.
    // Quando entrambi sono presenti, prima dei nuclei semantici si tenta
    // l'istanziazione del pattern (articolazione/identificazione/riconoscimento).
    // Se il pattern produce una voce coerente, quella è la risposta.
    // Se no, fallback al pipeline nuclei (mai regressione).
    kg_proc: Option<&KnowledgeGraph>,
    action_decision: Option<&crate::topology::action_reasoning::ActionDecision>,
    comprehension_report: Option<&crate::topology::comprehension_report::ComprehensionReport>,
) -> Option<Expression> {
    // ─── Phase 71: ActionShape gate ──────────────────────────────────
    // Se la Deliberation ha scelto Silence o Word, dispatch immediato.
    // Per le altre forme (Sentence, Question, EmpathicEcho) il pipeline
    // standard sotto è già adeguato (other_in_distress + is_question +
    // response_intention modulano la voce).
    if let Some(d) = deliberation {
        use crate::topology::deliberation::ActionShape;
        match d.action_shape {
            ActionShape::Silence => return None,
            ActionShape::Word => {
                return compose_word_response(
                    word_topology, lexicon, echo_exclude, d,
                );
            }
            _ => {}
        }
    }

    // ─── Phase 77: Pattern matcher esplicito ────────────────────────
    // Se abbiamo decision + report + kg_proc, proviamo a istanziare il
    // pattern del KG procedurale corrispondente all'azione decisa.
    // Funziona per: articolazione, identificazione, riconoscimento.
    // Per asserzione/ricambio cade nel pipeline standard (i nuclei
    // semantici e compose_word_response funzionano già bene).
    let mut pattern_base = None;
    if let (Some(decision), Some(report), Some(proc_kg)) =
        (action_decision, comprehension_report, kg_proc)
    {
        if let Some(emergent) = crate::topology::pattern_matcher::compose_from_pattern(
            decision, report, proc_kg, word_topology, lexicon,
        ) {
            let should_expand = decision.kind == crate::topology::action_reasoning::ActionKind::RecognizeClaim;

            if should_expand {
                // Per il riconoscimento (es. "Hai paura del buio.") vogliamo ANCHE un'espansione.
                // Salviamo il pattern base e lasciamo che il flusso continui per generare
                // l'elaborazione semantica.
                pattern_base = Some(emergent);
            } else {
                return Some(emergent);
            }
        }
    }

    // 1. Raccogli le parole attive del campo — la materia disponibile.
    let active = word_topology.active_words();
    if active.is_empty() {
        return pattern_base;
    }

    // Due pool: uno per CAPIRE (include input), uno per ESPRIMERE (esclude echo).
    // Le attivazioni qui sono RAW — non moltiplicate per la valenza.
    let comprehension_pool: Vec<(&str, f64)> = active.iter()
        .filter(|(w, act)| {
            *act > 0.02
            && w.chars().count() >= 3
            && lexicon.get(w).map(|p| p.stability >= 0.25 && p.exposure_count >= 3).unwrap_or(false)
        })
        .map(|(w, act)| (*w, *act))
        .collect();

    let candidates: Vec<(&str, f64)> = comprehension_pool.iter()
        .filter(|(w, _)| !echo_exclude.contains(&w.to_string()))
        .copied()
        .collect();

    if comprehension_pool.is_empty() || candidates.is_empty() {
        return pattern_base;
    }

    // 2. Estrai nuclei semantici — relazioni KG tra parole attive.
    // Usiamo comprehension_pool per l'estrazione, in modo che l'entità
    // possa capire le relazioni che coinvolgono l'input!
    // compose_from_nuclei si occuperà poi di non ripetere il soggetto
    // se è in echo_exclude.
    let nuclei = extract_nuclei(&comprehension_pool, kg, input_words, valence_drives, lexicon, episodes, is_question, Some(5));


    // 4. Determina la voce dell'entità dal suo stato.
    //    Usa comprehension_pool (non candidates) perché la voce emerge da
    //    TUTTO ciò che l'entità sente, incluso l'input.
    let mut voice = derive_voice(valence_drives, active_fractals, codon, &comprehension_pool, lexicon);

    // Phase 62: se l'Altro è in distress, la voce diventa empatica (2a persona + interrogativo).
    if other_in_distress && !matches!(voice.mood, ExpressionMood::Silent) {
        voice.person = crate::topology::grammar::Person::Second;
        voice.mood = ExpressionMood::Interrogative;
    }

    // Phase 67: l'intenzione deliberata da NarrativeSelf colora la voce.
    // Non sovrascrive il distress (priorità più alta) — affina la voce quando
    // la valenza non è sufficiente da sola.
    if !other_in_distress {
        if let Some(intent) = response_intention {
            match intent {
                "risuonare" => {
                    // Risonanza empatica → 2a persona, interrogativo aperto
                    voice.person = crate::topology::grammar::Person::Second;
                    if !matches!(voice.mood, ExpressionMood::Silent) {
                        voice.mood = ExpressionMood::Interrogative;
                    }
                }
                "esplorare" => {
                    // Esplorazione → mood esplorativo
                    if !matches!(voice.mood, ExpressionMood::Silent) {
                        voice.mood = ExpressionMood::Explorative;
                    }
                }
                "riflettere" => {
                    // Riflessione → prima persona, dichiarativo
                    voice.person = crate::topology::grammar::Person::First;
                }
                "restare" => {
                    // Ritiro → minimalismo, campo decide
                    voice.mood = ExpressionMood::Silent;
                }
                _ => {} // esprimere, riconoscere, etc. → la valenza guida
            }
        }
    }

    // 5. Componi l'espressione.
    let mut expr = if !nuclei.is_empty() {
        compose_from_nuclei(&nuclei, &voice, &candidates, lexicon, echo_exclude)
    } else {
        compose_from_field(&voice, &candidates, lexicon, echo_exclude, valence_drives)
    };

    if let Some(mut base) = pattern_base {
        // Unisci il pattern base (es. "Hai paura del buio.") con l'espansione semantica.
        // Aggiungiamo uno spazio e combiniamo.
        if let Some(ref mut e) = expr {
            base.text.push(' ');
            base.text.push_str(&e.text);
            base.words_used.extend(e.words_used.clone());
        }
        expr = Some(base);
    }

    // Nota: is_question (input utente contiene '?') influenza l'interpretazione
    // dell'input ma non necessariamente la risposta — la voce è già determinata.
    let _ = is_question;

    expr
}

// ─── Estrazione nuclei semantici ───────────────────────────────────────────

/// Cerca relazioni KG tra le parole attive nel campo.
///
/// Le attivazioni nel pool già incorporano la colorazione Octalysis
/// (fatta in `compose()` prima di chiamare questa funzione).
/// Qui si aggiunge la proximità all'input come fattore di rilevanza.
///
/// Il risultato è la COMPRENSIONE dell'entità: non ancora l'output,
/// ma la mappa semantica di ciò che ha sentito e riconosce.
pub fn extract_nuclei(
    candidates: &[(&str, f64)],
    kg: &KnowledgeGraph,
    input_words: &[String],
    valence_drives: &[f64; 8],
    lexicon: &Lexicon,
    episodes: Option<&SemanticEpisodeLog>,
    is_question: bool,
    // Phase 67: None = tutti i nuclei (comprensione in receive()).
    // Some(5) = top 5 (generazione in compose()).
    max_nuclei: Option<usize>,
) -> Vec<SemanticNucleus> {
    let mut nuclei: Vec<SemanticNucleus> = Vec::new();

    // Relazioni informative ordinate per rilevanza espressiva.
    // SIMILAR_TO è escluso: è troppo debole semanticamente per generare espressione.
    let rel_types = [
        RelationType::Causes,
        RelationType::IsA,
        RelationType::Has,
        RelationType::Does,
        RelationType::PartOf,
        RelationType::UsedFor,
        RelationType::OppositeOf,
        RelationType::Enables,
        RelationType::TransformsInto,
        RelationType::Requires,
    ];

    // Set per lookup veloce
    let active_set: std::collections::HashSet<&str> = candidates.iter().map(|(w, _)| *w).collect();

    // ─── Input proximity: parole dell'input + loro vicini KG diretti ──────
    // "Vicino dell'input" = parola raggiungibile in 1 hop KG da una parola input.
    // Questo è il cerchio di comprensione dell'entità rispetto a ciò che ha ricevuto.
    let input_set: std::collections::HashSet<&str> = input_words.iter()
        .map(|w| w.as_str()).collect();

    let mut input_neighborhood: std::collections::HashSet<String> = std::collections::HashSet::new();
    for iw in input_words {
        input_neighborhood.insert(iw.clone());
        // 1-hop KG neighbors of each input word (all relation types)
        for &rel in &rel_types {
            for (obj, _) in kg.query_objects_weighted(iw, rel) {
                input_neighborhood.insert(obj.to_string());
            }
            // Also reverse: words that have a relation TO this input word
            for subj in kg.query_subjects(iw, rel) {
                input_neighborhood.insert(subj.to_string());
            }
        }
    }

    // ─── Parents IsA dell'input + fratelli (siblings) ─────────────────────
    // Le parole che condividono un parent IsA con una parola input sono i
    // "fratelli" — il vocabolario della stessa regione semantica. Se l'input
    // è "ciao" e nel KG "ciao IsA saluto", allora "salve", "benvenuto",
    // "buongiorno" sono fratelli — UI-r1 può rispondere CON loro senza
    // ripetere l'input. È la regione, non l'eco.
    let mut input_isa_parents: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut sibling_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for iw in input_words {
        for (parent, _) in kg.query_objects_weighted(iw, RelationType::IsA) {
            input_isa_parents.insert(parent.to_string());
            for sib in kg.query_subjects(parent, RelationType::IsA) {
                if !input_set.contains(sib) {
                    sibling_set.insert(sib.to_string());
                }
            }
        }
    }

    for &(word, act) in candidates {
        for &rel in &rel_types {
            for (obj, conf) in kg.query_objects_weighted(word, rel) {
                let obj_str = obj.to_string();
                if active_set.contains(obj_str.as_str()) && obj_str != word {
                    let obj_act = candidates.iter()
                        .find(|(w, _)| *w == obj_str.as_str())
                        .map(|(_, a)| *a)
                        .unwrap_or(0.0);

                    let strength = (act * obj_act).sqrt() * conf as f64;

                    // Hub damping: nodi con troppi archi producono nuclei deboli
                    let subj_degree = kg.total_degree(word);
                    let obj_degree = kg.total_degree(&obj_str);
                    let hub_penalty = if subj_degree > 200 || obj_degree > 200 {
                        0.3
                    } else if subj_degree > 50 || obj_degree > 50 {
                        0.6
                    } else {
                        1.0
                    };

                    // ─── Input proximity scoring ─────────────────────────
                    // L'input è il segnale primario, ma l'entità si esprime
                    // dal SUO campo, non ripetendo le parole ricevute.
                    //
                    // Preferenza:
                    //   • Entrambe non-input ma nell'intorno → entità descrive
                    //     la regione semantica attivata dall'input (ideale)
                    //   • Oggetto = parola input → entità descrive l'input
                    //     e.g. "saluto [rel] ciao" → accettabile
                    //   • Soggetto = parola input → eco: "ciao [rel] X" → penalizzato
                    //   • Nessuna connessione → sfondo irrilevante
                    let subj_is_input = input_set.contains(word);
                    let obj_is_input = input_set.contains(obj_str.as_str());
                    let subj_near_input = input_neighborhood.contains(word);
                    let obj_near_input = input_neighborhood.contains(&obj_str);

                    // Phase 67: la pertinenza all'input è fondamentale per la comprensione.
                    // L'entità deve rispondere A ciò che le viene detto — non emettere
                    // il nucleo più forte nel campo.
                    //
                    // Phase 70 (rispondere DALLA regione, non DALL'input):
                    //   - Domanda: l'entità SPIEGA l'input → subj=input è bene
                    //     ("abbaiare CAUSES rumore" per "perché abbaia?").
                    //   - Statement / saluto / espressione: l'entità RISPONDE dalla
                    //     regione → un fratello dell'input come soggetto è meglio
                    //     che echeggiare l'input. Per "ciao" preferisce
                    //     "salve EXPRESSES presenza" a "ciao IS_A saluto".
                    let subj_is_sibling = sibling_set.contains(word);
                    let proximity = if is_question {
                        // ── Domanda: privilegia subj=input (modalità spiegazione) ──
                        if subj_is_input && !obj_is_input && obj_near_input {
                            5.0
                        } else if subj_is_input && !obj_is_input {
                            3.5
                        } else if subj_near_input && obj_near_input && !subj_is_input && !obj_is_input {
                            3.0
                        } else if obj_is_input && !subj_is_input {
                            2.0
                        } else if subj_near_input && !subj_is_input {
                            1.5
                        } else if obj_near_input && !obj_is_input {
                            1.0
                        } else {
                            0.2
                        }
                    } else {
                        // ── Statement: rispondi DALLA regione, non echeggiare l'input ──
                        if subj_is_sibling && (obj_near_input || obj_is_input) {
                            // OTTIMO: un fratello come soggetto si esprime verso
                            // l'oggetto dell'input — è UI-r1 che parla dalla regione.
                            5.0
                        } else if subj_near_input && obj_near_input && !subj_is_input && !obj_is_input {
                            // BUONO: entrambi nell'intorno ma non input → elaborazione
                            3.5
                        } else if subj_is_sibling {
                            3.0
                        } else if obj_is_input && !subj_is_input {
                            // Soggetto non-input parla DELL'input
                            2.5
                        } else if subj_near_input && !subj_is_input {
                            1.5
                        } else if subj_is_input && !obj_is_input && obj_near_input {
                            // Eco-definizione: l'input è soggetto. Penalizzato per gli
                            // statement perché è la "scheda KG" dell'input, non una risposta.
                            0.6
                        } else if obj_near_input && !obj_is_input {
                            1.0
                        } else {
                            0.2
                        }
                    };

                    // Phase 70: suppressione nucleo-definizione.
                    // Un nucleo (input_word, IsA, parent_of_input) è la voce di
                    // dizionario dell'input — "ciao IS_A saluto" — non una risposta.
                    // Se la frase non è una domanda di tipo "cos'è X", schiaccia.
                    let is_definitional_eco = !is_question
                        && rel == RelationType::IsA
                        && subj_is_input
                        && !obj_is_input
                        && input_isa_parents.contains(&obj_str);
                    let definition_penalty = if is_definitional_eco { 0.05 } else { 1.0 };

                    // Phase 67: pertinenza relazionale — il tipo di atto comunicativo
                    // guida quali relazioni sono più rilevanti.
                    // Non è hardcoding: è l'entità che sa che "perché?" chiede CAUSE.
                    let relation_pertinence = if is_question {
                        // Domanda → privilegia CAUSES/ENABLES (spiegazione).
                        // IS_A/HAS sono descrittive, non esplicative — penalizzate
                        // ma non eliminate (utili se non ci sono CAUSES).
                        match rel {
                            RelationType::Causes => 3.0,
                            RelationType::Enables | RelationType::Requires => 2.0,
                            RelationType::TransformsInto => 1.5,
                            RelationType::IsA | RelationType::Has | RelationType::PartOf => 0.5,
                            RelationType::Does => 0.7,
                            _ => 1.0,
                        }
                    } else {
                        1.0
                    };

                    // Colorazione Octalysis: relazioni tra parole che risuonano
                    // con i drive attivi dell'entità emergono più forti.
                    let v_subj = valence_weight(word, valence_drives, lexicon);
                    let v_obj  = valence_weight(&obj_str, valence_drives, lexicon);
                    let valence_resonance = (v_subj + v_obj) * 0.5;

                    nuclei.push(SemanticNucleus {
                        subject: word.to_string(),
                        relation: rel,
                        object: obj_str,
                        strength: strength * hub_penalty * proximity * valence_resonance * relation_pertinence * definition_penalty,
                        subject_activation: act,
                        object_activation: obj_act,
                        proximity_score: proximity,
                    });
                }
            }
        }
    }

    // Ordina per forza decrescente
    nuclei.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));

    // Deduplicazione: tieni il più forte per coppia (subject, object)
    let mut seen = std::collections::HashSet::new();
    nuclei.retain(|n| seen.insert((n.subject.clone(), n.object.clone())));

    // Filtra rumore di fondo: nuclei con forza sotto soglia sono artefatti del resting state,
    // non comprensione reale. Soglia calibrata per escludere parole a riposo (act~0.05)
    // ma includere parole attivate dall'input (act~0.3+).
    nuclei.retain(|n| n.strength > 0.02);

    // Preferisci nuclei connessi all'input: se ne esistono, scarta quelli di sfondo puro.
    // "abitabile genera abitante" con proximity=0.2 non sopravvive se esistono nuclei
    // connessi all'input (proximity >= 1.5).
    if nuclei.iter().any(|n| n.proximity_score >= 1.5) {
        nuclei.retain(|n| n.proximity_score >= 1.0);
    }

    if let Some(max) = max_nuclei {
        nuclei.truncate(max);
    }
    // Senza max: tutti i nuclei restano. La comprensione è completa.

    // ─── Phase 58: risonanza episodica ────────────────────────────────────────
    // Nuclei connessi a concetti già vissuti dall'entità emergono più forti.
    // La memoria non "ricorda meccanicamente" — colora l'espressione dal profondo:
    // i temi che hanno lasciato traccia riaffiorano con più forza quando il campo
    // li riattiva.
    if let Some(eps) = episodes {
        let active_concepts: Vec<String> = candidates.iter().map(|(w, _)| w.to_string()).collect();
        let recalled = eps.recall_by_concepts(&active_concepts, 3);
        if !recalled.is_empty() {
            let episodic_words: std::collections::HashSet<String> = recalled.iter()
                .flat_map(|(ep, _)| ep.key_concepts.iter().cloned())
                .collect();
            for n in nuclei.iter_mut() {
                let subj_known = episodic_words.contains(&n.subject);
                let obj_known  = episodic_words.contains(&n.object);
                if subj_known && obj_known {
                    n.strength *= 1.4; // entrambi i termini hanno storia
                } else if subj_known || obj_known {
                    n.strength *= 1.2; // uno dei termini ha storia
                }
            }
            // Re-sort dopo il boost episodico
            nuclei.sort_by(|a, b| b.strength.partial_cmp(&a.strength)
                .unwrap_or(std::cmp::Ordering::Equal));
        }
    }

    nuclei
}

// ─── Voce dell'entità ──────────────────────────────────────────────────────

/// La voce emerge dallo stato interno dell'entità.
/// Non è una scelta — è il modo in cui l'entità SI TROVA.
fn derive_voice(
    valence: &[f64; 8],
    active_fractals: &[(u32, f64)],
    codon: [usize; 2],
    candidates: &[(&str, f64)],
    lexicon: &Lexicon,
) -> EntityVoice {
    // ─── Persona ───────────────────────────────────────────────────────
    // Emerge dalla dimensione dominante della valenza.
    //
    // Alta Agency/Ownership (CD1, CD4) → prima persona: "Io sento..."
    // Alta Social (CD5) → seconda persona: "Tu..."
    // Alta Unpredictability (CD7) o bassa Agency → terza persona/impersonale: "C'è..."
    //
    // Fallback: dal frattale dominante via trigramma inferiore.
    let cd1_meaning = valence[0];    // Epic Meaning → Agency
    let cd4_ownership = valence[3];  // Ownership → Confine
    let cd5_social = valence[4];     // Social → Valenza
    let cd7_surprise = valence[6];   // Unpredictability → Intensità

    let person = if cd1_meaning > 0.3 || cd4_ownership > 0.3 {
        Person::First
    } else if cd5_social > 0.3 {
        Person::Second
    } else if cd7_surprise.abs() > 0.3 || (cd1_meaning < -0.1 && cd4_ownership < -0.1) {
        Person::Third // impersonale/osservativo
    } else {
        // Fallback: dal frattale dominante
        if let Some(&(fid, _)) = active_fractals.first() {
            let lower_idx = fid / 8;
            match lower_idx {
                0 | 2 | 4 | 6 => Person::First,
                5 | 7 => Person::Second,
                _ => Person::Third,
            }
        } else {
            Person::First
        }
    };

    // ─── Tempo ─────────────────────────────────────────────────────────
    // Dal profilo dimensionale delle parole attive (come syntax_center).
    let (avg_tempo, avg_perm) = {
        let mut sum_t = 0.0f64;
        let mut sum_p = 0.0f64;
        let mut count = 0;
        for &(w, _) in candidates.iter().take(10) {
            if let Some(pat) = lexicon.get(w) {
                let v = pat.signature.values();
                sum_t += v[7]; // Tempo
                sum_p += v[5]; // Permanenza
                count += 1;
            }
        }
        if count > 0 {
            (sum_t / count as f64, sum_p / count as f64)
        } else {
            (0.5, 0.5)
        }
    };

    // Il presente è il tempo base dell'entità — descrive la realtà come la sente ORA.
    // Il passato imperfetto emerge solo da profili molto marcati (bassa temporalità E bassa permanenza).
    // Il futuro emerge da alta proiezione temporale.
    let tense = if avg_tempo > 0.70 {
        Tense::Future
    } else if avg_tempo < 0.25 && avg_perm < 0.25 {
        // Solo per parole fortemente radicate nel passato (memorie, stati)
        Tense::Imperfect
    } else {
        Tense::Present
    };

    // ─── Mood ──────────────────────────────────────────────────────────
    // L'umore espressivo emerge dalla combinazione di valenza e campo.
    let hedonic_tone: f64 = valence.iter().sum::<f64>() / 8.0;
    let field_energy: f64 = candidates.iter().map(|(_, a)| a).sum();
    let cd8_loss = valence[7]; // Loss Avoidance

    // L'umore Interrogativo non emerge dall'interno (dall'entità sorpresa):
    // l'entità che si stupisce AFFERMA la sua sorpresa, non interroga.
    // Interrogativo è riservato a casi futuri dove l'entità vuole esplicitamente
    // chiedere qualcosa (deliberato dalla will, non derivato dalla valenza).
    let mood = if field_energy < 0.5 || cd8_loss < -0.5 {
        ExpressionMood::Silent
    } else if cd7_surprise < -0.2 || hedonic_tone < -0.2 {
        ExpressionMood::Explorative
    } else if person == Person::Third {
        ExpressionMood::Observational
    } else {
        ExpressionMood::Declarative
    };

    EntityVoice { person, tense, mood }
}

// ─── Composizione da nuclei semantici ──────────────────────────────────────

/// Compone una frase da uno o più nuclei semantici.
/// La struttura emerge dalla relazione, la voce dalla persona.
fn compose_from_nuclei(
    nuclei: &[SemanticNucleus],
    voice: &EntityVoice,
    candidates: &[(&str, f64)],
    lexicon: &Lexicon,
    echo_exclude: &[String],
) -> Option<Expression> {
    // Phase 67: il nucleo primario è quello con lo score più alto.
    // Se il soggetto è una parola input, va bene — l'entità sta RISPONDENDO
    // all'input, non facendo eco. "abbaiare CAUSES rumore" per "perché abbaia?"
    // è una risposta, non eco.
    // Solo se il nucleo migliore ha proximity_score molto basso (<= 0.5) E il soggetto
    // è in echo_exclude, cerchiamo un'alternativa.
    let best = match nuclei.first() {
        Some(n) if n.proximity_score > 0.5 || !echo_exclude.contains(&n.subject) => n,
        _ => match nuclei.iter().find(|n| !echo_exclude.contains(&n.subject)) {
            Some(n) => n,
            None => match nuclei.first() {
                Some(n) => n,
                None => return None,
            },
        },
    };
    let mut words_used = Vec::new();

    // ─── Nucleo primario ───────────────────────────────────────────────
    let primary = render_nucleus(best, voice, lexicon);
    words_used.push(best.subject.clone());
    words_used.push(best.object.clone());

    // ─── Eventuale nucleo secondario ───────────────────────────────────
    // Cerca il miglior nucleo collegato (diverso da best) che condivida una parola.
    // Filtra anche qui per echo_exclude: Prometeo non cita l'input nemmeno nel secondario.
    let second_candidate = nuclei.iter()
        .filter(|n| std::ptr::eq(*n as *const _, best as *const _) == false)
        .find(|n| {
            let shares = n.subject == best.subject || n.subject == best.object
                || n.object == best.subject || n.object == best.object;
            let non_echo = !echo_exclude.contains(&n.subject);
            non_echo && shares && n.strength > best.strength * 0.4
        });

    let secondary = if let Some(second) = second_candidate {
        let rendered = render_nucleus_brief(second, best);
        if rendered.is_some() {
            if !words_used.contains(&second.subject) {
                words_used.push(second.subject.clone());
            }
            if !words_used.contains(&second.object) {
                words_used.push(second.object.clone());
            }
        }
        rendered
    } else {
        None
    };

    // ─── Composizione finale ───────────────────────────────────────────
    let text = if let Some(sec) = secondary {
        let connector = connective_between_nuclei(best, second_candidate.unwrap());
        format!("{}{}{}", primary, connector, sec)
    } else {
        primary
    };

    // Grammatica: capitalizzazione + punteggiatura
    let text = finish_sentence(&text, voice);

    Some(Expression { text, words_used })
}

/// Rende un nucleo semantico come frammento di frase.
/// Phase 67: un solo path per tutte le persone.
/// Nessun verbo hardcoded — la relazione KG genera la copula,
/// il soggetto e l'oggetto vengono dal campo.
/// L'entità non "percepisce" o "sente" per decisione del codice —
/// esprime la relazione che ha compreso.
fn render_nucleus(nucleus: &SemanticNucleus, voice: &EntityVoice, lexicon: &Lexicon) -> String {
    let subject = &nucleus.subject;
    let object = &nucleus.object;

    // Mood silenzioso → solo la parola più viva
    if matches!(voice.mood, ExpressionMood::Silent) {
        return subject.clone();
    }

    // La copula emerge dalla relazione KG — non è scelta per persona
    let copula = relation_to_copula(nucleus.relation, voice, lexicon, subject);
    let subject_with_art = with_definite_article(subject);

    if copula.is_empty() {
        // Relazione DOES: il soggetto compie l'azione-oggetto
        // La persona viene dalla voce dell'entità
        let conjugated = crate::topology::grammar::conjugate(object, voice.person, voice.tense);
        format!("{} {}", subject_with_art, conjugated)
    } else {
        // Tutte le altre relazioni: soggetto + copula + oggetto
        let obj_with_art = match nucleus.relation {
            RelationType::IsA | RelationType::PartOf =>
                with_indefinite_article(object),
            _ =>
                with_definite_article(object),
        };
        format!("{} {} {}", subject_with_art, copula, obj_with_art)
    }
}

/// Rende un nucleo secondario in forma concisa ma grammaticalmente completa.
/// Se il soggetto è già noto (condiviso col primo nucleo), lo omette.
/// Altrimenti rende la forma completa "S copula O".
fn render_nucleus_brief(nucleus: &SemanticNucleus, primary: &SemanticNucleus) -> Option<String> {
    let copula = relation_to_copula_simple(nucleus.relation);
    let obj_with_art = match nucleus.relation {
        RelationType::IsA | RelationType::PartOf =>
            with_indefinite_article(&nucleus.object),
        _ =>
            with_definite_article(&nucleus.object),
    };

    if nucleus.subject == primary.subject || nucleus.subject == primary.object {
        // Soggetto già noto — basta il predicato (italiano permette omissione)
        if copula.is_empty() {
            Some(format!("fa {}", with_definite_article(&nucleus.object)))
        } else {
            Some(format!("{} {}", copula, obj_with_art))
        }
    } else {
        // Soggetto diverso — forma completa con articolo
        if copula.is_empty() {
            Some(format!("{} fa {}", with_definite_article(&nucleus.subject), with_definite_article(&nucleus.object)))
        } else {
            Some(format!("{} {} {}", with_definite_article(&nucleus.subject), copula, obj_with_art))
        }
    }
}

// ─── Composizione senza nuclei (campo con parole attive ma senza relazioni KG)

fn compose_from_field(
    voice: &EntityVoice,
    candidates: &[(&str, f64)],
    lexicon: &Lexicon,
    echo_exclude: &[String],
    valence_drives: &[f64; 8],
) -> Option<Expression> {
    // Phase 59: se i drive interni sono dominanti, l'entità nomina ciò che sente.
    // Questo gestisce domande sullo stato ("come stai?") anche senza riconoscere
    // le parole specifiche — l'entità risponde dal suo stato interno, non dal KG.
    let max_drive = valence_drives.iter().map(|v| v.abs()).fold(0.0f64, f64::max);
    if max_drive > 0.95 {
        if let Some(expr) = express_from_drives(valence_drives, lexicon) {
            return Some(expr);
        }
    }

    // Senza nuclei semantici, l'entità esprime ciò che sente più forte.
    // La parola più saliente è scelta con delta-scoring + colorazione Octalysis:
    // le parole la cui firma 8D risuona con i drive attivi emergono naturalmente.

    let score_word = |w: &str, act: f64| -> f64 {
        let resting = lexicon.get(w).map(|p| p.stability * 0.003).unwrap_or(0.0);
        let delta = (act - resting).max(0.001);
        let vw = valence_weight(w, valence_drives, lexicon);
        delta * vw
    };

    let best = candidates.iter()
        .filter(|(w, _)| {
            !echo_exclude.contains(&w.to_string()) &&
            lexicon.get(*w).map(|p| p.pos != Some(PartOfSpeech::Verb)).unwrap_or(true)
        })
        .max_by(|a, b| {
            score_word(a.0, a.1).partial_cmp(&score_word(b.0, b.1))
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;

    let word = best.0;
    let mut words_used = vec![word.to_string()];

    // Cerca una seconda parola: alta cosine similarity 8D + risonanza valenza
    let second = candidates.iter()
        .filter(|(w, _)| {
            *w != word
            && !echo_exclude.contains(&w.to_string())
            && !words_used.contains(&w.to_string())
        })
        .filter_map(|(w, act)| {
            let sim = signature_similarity(word, w, lexicon)?;
            if sim > 0.7 {
                let vw = valence_weight(w, valence_drives, lexicon);
                Some((*w, *act, sim * vw))
            } else {
                None
            }
        })
        .max_by(|a, b| (a.1 * a.2).partial_cmp(&(b.1 * b.2)).unwrap_or(std::cmp::Ordering::Equal));

    let text = match voice.mood {
        ExpressionMood::Silent => {
            return None; // Silenzio genuino
        }
        ExpressionMood::Interrogative => {
            if let Some((w2, _, _)) = second {
                words_used.push(w2.to_string());
                format!("{}, {}", word, w2)
            } else {
                word.to_string()
            }
        }
        _ => {
            // Cerca un verbo attivo per dare vita alla frase — con colorazione valenza
            let verb = candidates.iter()
                .filter(|(w, _)| {
                    *w != word
                    && !echo_exclude.contains(&w.to_string())
                    && lexicon.get(w).map(|p| p.pos == Some(PartOfSpeech::Verb)).unwrap_or(false)
                })
                .max_by(|a, b| {
                    score_word(a.0, a.1).partial_cmp(&score_word(b.0, b.1))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            if let Some((v, _)) = verb {
                let conjugated = grammar::conjugate(v, voice.person, voice.tense);
                words_used.push(v.to_string());
                if voice.person == Person::First {
                    format!("{} {}", conjugated, word)
                } else {
                    format!("{} {}", word, conjugated)
                }
            } else if let Some((w2, _, _)) = second {
                words_used.push(w2.to_string());
                format!("{}, {}", word, w2)
            } else {
                word.to_string()
            }
        }
    };

    let text = finish_sentence(&text, voice);
    Some(Expression { text, words_used })
}

// ─── Funzioni di supporto ──────────────────────────────────────────────────

/// Traduce un tipo di relazione in copula italiana.
/// Non è un template — è la semantica della relazione espressa in lingua.
fn relation_to_copula(
    rel: RelationType,
    voice: &EntityVoice,
    lexicon: &Lexicon,
    subject: &str,
) -> String {
    match rel {
        RelationType::IsA => "è".to_string(),
        RelationType::Has => "ha".to_string(),
        RelationType::Does => {
            // La relazione DOES è un'azione — il soggetto AGISCE l'oggetto.
            // L'oggetto è spesso un verbo all'infinito.
            String::new() // il soggetto stesso agisce
        }
        RelationType::Causes => {
            // CAUSES: il soggetto produce/porta/muove verso l'oggetto
            match voice.mood {
                ExpressionMood::Declarative => "porta".to_string(),
                ExpressionMood::Observational => "porta a".to_string(),
                ExpressionMood::Explorative => "muove verso".to_string(),
                _ => "porta".to_string(),
            }
        }
        RelationType::PartOf => "è parte di".to_string(),
        RelationType::UsedFor => "serve a".to_string(),
        RelationType::OppositeOf => {
            match voice.mood {
                ExpressionMood::Declarative => "non è".to_string(),
                _ => "contrasta con".to_string(),
            }
        }
        RelationType::Enables => "permette".to_string(),
        RelationType::TransformsInto => "diventa".to_string(),
        RelationType::Requires => "richiede".to_string(),
        RelationType::SimilarTo => "è come".to_string(),
        RelationType::Expresses => "esprime".to_string(),
        RelationType::Symbolizes => "simboleggia".to_string(),
        _ => "è".to_string(),
    }
}

/// Copula semplice per nuclei secondari — sempre un verbo o locuzione verbale.
fn relation_to_copula_simple(rel: RelationType) -> &'static str {
    match rel {
        RelationType::IsA => "è",
        RelationType::Has => "ha",
        RelationType::Causes => "porta",
        RelationType::Does => "",        // DOES: oggetto è l'azione stessa
        RelationType::PartOf => "è parte di",
        RelationType::UsedFor => "serve a",
        RelationType::OppositeOf => "non è",
        RelationType::Enables => "permette",
        RelationType::TransformsInto => "diventa",
        RelationType::Requires => "richiede",
        RelationType::SimilarTo => "è come",
        _ => "è",
    }
}

/// Connettivo tra due nuclei — emerge dalla relazione tra loro.
fn connective_between_nuclei(n1: &SemanticNucleus, n2: &SemanticNucleus) -> &'static str {
    // Se il secondo nucleo è un OPPOSITE_OF → contrasto
    if n2.relation == RelationType::OppositeOf {
        ", eppure "
    }
    // IS_A / PartOf secondario: attribuzione, non coordinazione → virgola
    // Evita "e è" / "e è parte di"
    else if matches!(n2.relation, RelationType::IsA | RelationType::PartOf) {
        ", "
    }
    // Se condividono il soggetto e la relazione è azione → coordinazione " e "
    else if n1.subject == n2.subject && matches!(n2.relation, RelationType::Has | RelationType::Causes | RelationType::Does | RelationType::UsedFor | RelationType::Enables) {
        " e "
    }
    // Se l'oggetto del primo è il soggetto del secondo → catena (relazione di causa)
    else if n1.object == n2.subject {
        ", "
    }
    // Default: virgola semplice (sicuro grammaticalmente)
    else {
        ", "
    }
}


/// Similarità coseno tra le firme 8D di due parole.
fn signature_similarity(w1: &str, w2: &str, lexicon: &Lexicon) -> Option<f64> {
    let p1 = lexicon.get(w1)?;
    let p2 = lexicon.get(w2)?;
    let v1 = p1.signature.values();
    let v2 = p2.signature.values();

    let mut dot = 0.0f64;
    let mut n1 = 0.0f64;
    let mut n2 = 0.0f64;
    for i in 0..8 {
        dot += v1[i] * v2[i];
        n1 += v1[i] * v1[i];
        n2 += v2[i] * v2[i];
    }
    let norm = n1.sqrt() * n2.sqrt();
    if norm < 1e-9 { return None; }
    Some(dot / norm)
}

/// Finalizza la frase: capitalizzazione + punteggiatura dal mood.
fn finish_sentence(raw: &str, voice: &EntityVoice) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Capitalizza prima lettera
    let mut chars = trimmed.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    let rest: String = chars.collect();
    let capitalized = format!("{}{}", first, rest);

    // Punteggiatura dal mood
    let ending = match voice.mood {
        ExpressionMood::Interrogative => "?",
        ExpressionMood::Explorative => "...",
        ExpressionMood::Silent => ".",
        _ => ".",
    };

    // Rimuovi punteggiatura doppia
    let base = capitalized.trim_end_matches(|c: char| c == '.' || c == '?' || c == '!');
    format!("{}{}", base, ending)
}
