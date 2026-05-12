//! Pattern Matcher — istanzia i pattern del KG procedurale come voce.
//!
//! Phase 77 — il bridge tra `ActionDecision` (cosa fare, su cosa) e i
//! pattern grammaticali del KG procedurale (`articolazione`, `identificazione`,
//! `riconoscimento`, `ricambio`, `asserzione`).
//!
//! Francesco (2026-04-26): "le regole grammaticali dovremmo spiegargliele,
//! non infilargliele a forza nel codice". I pattern vivono nel KG come
//! triple `pattern Requires <ruolo> via <funzione>`. Questo modulo li
//! LEGGE e li istanzia. Niente template hardcoded — la struttura emerge
//! dal grafo, il contenuto emerge dal campo + anchor_words.
//!
//! ## Pipeline
//!
//! ```text
//! ActionDecision.kind ──→ pattern_name (es. "articolazione")
//!                          │
//!                          ▼
//!                     load_pattern_schema(kg_proc)
//!                          │  legge "X UsedFor <fine> via <target>" + tutti i "X Requires <ruolo> via <funzione>"
//!                          ▼
//!                     instantiate(schema, decision, report, kg_proc, kg_sem, lexicon, field)
//!                          │  per ogni slot trova il candidato migliore (anchor + via match + field)
//!                          ▼
//!                     render(instance, decision, report)
//!                          │  ordine sintattico italiano per famiglia di pattern
//!                          ▼
//!                     Expression ("Di cosa hai paura?")
//! ```
//!
//! ## Cosa fa il KG procedurale, cosa fa il KG semantico
//!
//! - **kg_proc**: pattern, ruoli grammaticali, classi delle parole funzionali
//!   (pronome/preposizione/verbo/marcatore + sotto-categorie).
//!   Es. `cosa IsA pronome`, `cosa IsA interrogativo`, `cosa UsedFor chiedere via=oggetto`.
//!
//! - **kg_sem**: parole-contenuto del mondo (paura, sole, casa, ...) e loro
//!   relazioni semantiche. Es. `paura IsA emozione`, `paura Causes tremore`.
//!
//! Il pattern matcher legge **entrambi**: gli slot "pronome", "verbo", "marcatore"
//! da kg_proc; il `target` del gap (la parola del claim del parlante,
//! es. "paura") è una parola-contenuto in kg_sem.

use std::collections::HashSet;

use crate::topology::action_reasoning::{ActionDecision, ActionKind, ActionTarget, NarrativeSubject};
use crate::topology::comprehension_report::ComprehensionReport;
use crate::topology::expression::Expression;
use crate::topology::grammar::{self, PartOfSpeech, Person, Tense};
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::lexicon::Lexicon;
use crate::topology::relation::RelationType;
use crate::topology::word_topology::WordTopology;

// ═══════════════════════════════════════════════════════════════════════════
// Schema del pattern (letto dal KG procedurale)
// ═══════════════════════════════════════════════════════════════════════════

/// Specifica di uno slot di un pattern.
/// `role` è il ruolo grammaticale (pronome, verbo, marcatore...).
/// `via` è la qualificazione specifica (interrogativo, cognitivo, modale...).
#[derive(Debug, Clone)]
pub struct SlotSpec {
    pub role: String,
    pub via: Option<String>,
}

/// Schema del pattern come letto dal KG procedurale:
/// `pattern UsedFor <purpose> via <purpose_target>` +
/// `pattern Requires <role> via <via>` per ogni slot.
#[derive(Debug, Clone)]
pub struct PatternSchema {
    pub name: String,
    pub purpose: Option<String>,
    pub purpose_target: Option<String>,
    pub slots: Vec<SlotSpec>,
}

/// Risultato dell'istanziazione: per ogni slot, la parola scelta.
#[derive(Debug, Clone)]
pub struct InstantiatedPattern {
    pub name: String,
    pub fillers: Vec<SlotFill>,
}

#[derive(Debug, Clone)]
pub struct SlotFill {
    pub spec: SlotSpec,
    pub word: String,
}

impl InstantiatedPattern {
    fn find(&self, role: &str) -> Option<&str> {
        self.fillers.iter()
            .find(|f| f.spec.role == role)
            .map(|f| f.word.as_str())
    }
    fn find_with_via(&self, role: &str, via: &str) -> Option<&str> {
        self.fillers.iter()
            .find(|f| f.spec.role == role && f.spec.via.as_deref() == Some(via))
            .map(|f| f.word.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Selezione del pattern: per RISONANZA nel campo del kg_proc
// ═══════════════════════════════════════════════════════════════════════════
//
// Phase 79: la selezione del pattern non è più una mappa hardcoded
// `ActionKind → pattern_name`. È risonanza emergente: il
// `ComprehensionReport` semina percetti nel campo del kg_proc, e il
// pattern la cui pertinenza (`UsedFor X via Y`) ha più attivazione
// dei suoi target vince. Vedi `kg_proc_field.rs`.
//
// Il vantaggio: aggiungere un pattern al KG procedurale è una modifica
// di dati, non di codice. I 10 pattern esistenti (articolazione,
// identificazione, ricambio, asserzione, presentazione, riconoscimento,
// posizionamento, specchio, esplorazione, esitazione) sono già tutti
// raggiungibili.

// ═══════════════════════════════════════════════════════════════════════════
// 1. Caricamento dello schema dal KG procedurale
// ═══════════════════════════════════════════════════════════════════════════

/// Legge `name UsedFor <purpose> via <target>` + `name Requires <role> via <fn>`
/// dal KG procedurale e produce uno schema di pattern.
/// Restituisce `None` se il pattern non è nel KG (caso normale: niente kg_proc
/// caricato → fallback al path nuclei).
pub fn load_pattern_schema(name: &str, kg_proc: &KnowledgeGraph) -> Option<PatternSchema> {
    // Verifica che il nodo esista come pattern.
    let is_pattern = kg_proc.query_objects(name, RelationType::IsA)
        .iter().any(|o| *o == "pattern");
    if !is_pattern { return None; }

    // Purpose: prima triple UsedFor con eventuale via.
    let (purpose, purpose_target) = kg_proc.query_objects_with_via(name, RelationType::UsedFor)
        .into_iter().next()
        .map(|(p, _, v)| (Some(p.to_string()), v.map(|s| s.to_string())))
        .unwrap_or((None, None));

    // Slots: tutti i Requires con eventuale via, in ordine di lettura.
    let slots: Vec<SlotSpec> = kg_proc.query_objects_with_via(name, RelationType::Requires)
        .into_iter()
        .map(|(role, _, via)| SlotSpec {
            role: role.to_string(),
            via: via.map(|s| s.to_string()),
        })
        .collect();

    if slots.is_empty() { return None; }

    Some(PatternSchema {
        name: name.to_string(),
        purpose,
        purpose_target,
        slots,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Slot-filler — sceglie la parola per ogni slot
// ═══════════════════════════════════════════════════════════════════════════

/// Trova candidati per uno slot leggendo il KG procedurale:
/// parole con `IsA <role>`, preferendo quelle con anche `IsA <via>`.
fn candidates_for_slot(slot: &SlotSpec, kg_proc: &KnowledgeGraph) -> Vec<String> {
    // Strict: IsA role AND IsA via
    if let Some(via) = &slot.via {
        let by_role: HashSet<&str> = kg_proc.query_subjects(&slot.role, RelationType::IsA)
            .into_iter().collect();
        let by_via: HashSet<&str> = kg_proc.query_subjects(via, RelationType::IsA)
            .into_iter().collect();
        let strict: Vec<String> = by_role.intersection(&by_via)
            .map(|s| s.to_string())
            .collect();
        if !strict.is_empty() { return strict; }
    }
    // Fallback: solo IsA role
    kg_proc.query_subjects(&slot.role, RelationType::IsA)
        .into_iter().map(|s| s.to_string()).collect()
}

/// Per un pronome interrogativo, sceglie quello il cui `UsedFor chiedere via=X`
/// matcha il `target` (gap.missing). Es. target="oggetto" → "cosa".
/// Se più candidati matchano (es. "cosa" e "che" entrambi via=oggetto),
/// preferisce quello che è già in `anchors` (deciso da action_reasoning).
/// Se nessun match → None e si lascia il fallback generico.
fn interrogative_for_target(
    target: &str,
    kg_proc: &KnowledgeGraph,
    anchors: &HashSet<&str>,
) -> Option<String> {
    let pronouns: HashSet<&str> = kg_proc.query_subjects("pronome", RelationType::IsA)
        .into_iter().collect();
    let interrogatives: HashSet<&str> = kg_proc.query_subjects("interrogativo", RelationType::IsA)
        .into_iter().collect();
    let mut matches: Vec<String> = Vec::new();
    for p in pronouns.intersection(&interrogatives) {
        for (_, _, via) in kg_proc.query_objects_with_via(p, RelationType::UsedFor) {
            if via.map(|v| v == target).unwrap_or(false) {
                matches.push(p.to_string());
                break;
            }
        }
    }
    if matches.is_empty() { return None; }
    // Preferenza: quello in anchors (action_reasoning ha già scelto)
    if let Some(in_anchor) = matches.iter().find(|p| anchors.contains(p.as_str())) {
        return Some(in_anchor.clone());
    }
    // Altrimenti: il primo (HashMap order). Sufficiente per pari merito.
    matches.into_iter().next()
}

/// Per una preposizione, sceglie quella il cui `UsedFor introdurre via=X`
/// è semanticamente coerente col contesto. Default "di" (specificazione)
/// quando l'articolazione chiede "Di cosa...".
fn preposition_for_context(_context: &str, kg_proc: &KnowledgeGraph) -> Option<String> {
    // Per ora privilegia "di" (specificazione) come scelta di default per
    // il pattern articolazione. Logiche più sofisticate (mapping context→via)
    // possono crescere come triple nel KG procedurale.
    let candidates: HashSet<&str> = kg_proc.query_subjects("preposizione", RelationType::IsA)
        .into_iter().collect();
    if candidates.contains("di") { return Some("di".to_string()); }
    candidates.into_iter().next().map(|s| s.to_string())
}

/// I ruoli "grammaticali" — quelli che hanno parole-funzione classificate
/// come `IsA <role>` nel KG procedurale (pronome, verbo, preposizione, ecc.).
/// Tutti gli altri ruoli (predicato, soggetto, oggetto, nome, parola) sono
/// "contenutistici" — vanno riempiti dal campo + anchor_words, non dal kg_proc.
fn is_grammatical_role(role: &str) -> bool {
    matches!(role,
        "pronome" | "articolo" | "preposizione" | "marcatore"
        | "verbo" | "avverbio" | "congiunzione" | "interiezione")
}

/// Riconosce le parole-funzione consultando il KG procedurale: una parola
/// è "funzionale" se la sua catena IsA porta a categorie di pura macchina
/// grammaticale (pronome, articolo, preposizione, marcatore, congiunzione)
/// oppure è una copula (essere/avere/stare/fare). I verbi "azione/percettivo/
/// cognitivo/comunicativo/denominativo" NON sono funzionali — sono verbi
/// con contenuto semantico.
///
/// Phase 79: prima era una lista hardcoded di parole italiane in Rust.
/// Ora è una proprietà strutturale del kg_proc: aggiungere/togliere parole
/// di funzione = curare le triple `IsA pronome/articolo/...` o `IsA copula`,
/// mai più toccare Rust.
fn is_function_word(w: &str, kg_proc: &KnowledgeGraph) -> bool {
    // Forme elise (un', d', l') si riconducono alla forma base.
    let base: String = if w.ends_with('\'') {
        w.trim_end_matches('\'').to_string()
    } else {
        w.to_string()
    };

    let isa: HashSet<&str> = kg_proc
        .query_objects(&base, RelationType::IsA)
        .into_iter()
        .collect();

    // Verbi: solo le copule (essere/avere/stare/fare) sono funzionali.
    if isa.contains("verbo") {
        return isa.contains("copula");
    }

    // Categorie grammaticali pure: pronome, articolo, preposizione,
    // marcatore, congiunzione → funzionale.
    isa.iter().any(|cat| matches!(*cat,
        "pronome" | "articolo" | "preposizione" | "marcatore" | "congiunzione"
    ))
}

/// Sceglie la parola per uno slot grammaticale (pronome non interrogativo,
/// verbo generico, marcatore...) usando candidati IsA role+via,
/// score = anchor_word + field_activation. Restituisce None se nessun
/// candidato accettabile.
fn pick_slot_word(
    slot: &SlotSpec,
    decision: &ActionDecision,
    kg_proc: &KnowledgeGraph,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
) -> Option<String> {
    // Slot contenutistico: prendilo dalle anchor_words (parola-contenuto).
    // Es. slot "predicato+identità" per identificazione → "entità", "fondamento".
    if !is_grammatical_role(&slot.role) {
        return decision.anchor_words.iter()
            .find(|w| !is_function_word(w, kg_proc) && w.chars().count() >= 3)
            .cloned();
    }

    let candidates = candidates_for_slot(slot, kg_proc);
    if candidates.is_empty() { return None; }

    let anchors: HashSet<&str> = decision.anchor_words.iter().map(|s| s.as_str()).collect();
    let active: std::collections::HashMap<String, f64> = word_topology.active_words()
        .into_iter().map(|(w, a)| (w.to_string(), a)).collect();

    let mut best: Option<(String, f64)> = None;
    for cand in &candidates {
        let mut score = 1.0_f64;
        if anchors.contains(cand.as_str()) { score += 3.0; }
        if let Some(act) = active.get(cand.as_str()) { score += 1.5 * act; }
        // Bonus minimo se la parola esiste nel lessico (è "viva" per UI-r1)
        if lexicon.get(cand.as_str()).is_some() { score += 0.2; }
        if best.as_ref().map_or(true, |(_, b)| score > *b) {
            best = Some((cand.clone(), score));
        }
    }
    best.map(|(w, _)| w)
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Istanziazione del pattern
// ═══════════════════════════════════════════════════════════════════════════

/// Istanzia il pattern: per ogni slot sceglie la parola che lo riempie,
/// usando informazione del decision (anchor_words, target) e del campo.
pub fn instantiate(
    schema: &PatternSchema,
    decision: &ActionDecision,
    kg_proc: &KnowledgeGraph,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
) -> Option<InstantiatedPattern> {
    // target_role: per `articolazione` deriva dal `gap.missing` (es. "oggetto").
    // Per altri pattern resta None — useremo altri segnali.
    let gap_target: Option<String> = match &decision.target {
        ActionTarget::Gap { signifier_missing, .. } => Some(signifier_missing.clone()),
        _ => None,
    };

    let anchors_set: HashSet<&str> = decision.anchor_words.iter().map(|s| s.as_str()).collect();

    let mut fillers: Vec<SlotFill> = Vec::new();
    for slot in &schema.slots {
        let word = match (slot.role.as_str(), slot.via.as_deref()) {
            // Pronome interrogativo → cerca quello che chiede il ruolo del gap
            ("pronome", Some("interrogativo")) => {
                gap_target.as_deref()
                    .and_then(|t| interrogative_for_target(t, kg_proc, &anchors_set))
                    .or_else(|| pick_slot_word(slot, decision, kg_proc, word_topology, lexicon))
            }
            // Preposizione di contesto → "di" di default per articolazione
            ("preposizione", Some("specificazione")) => {
                gap_target.as_deref()
                    .and_then(|t| preposition_for_context(t, kg_proc))
                    .or_else(|| pick_slot_word(slot, decision, kg_proc, word_topology, lexicon))
            }
            // Pronome personale → derivato dal narrative_subject
            ("pronome", Some("personale")) => {
                Some(match decision.narrative_subject {
                    NarrativeSubject::Self_   => "io".to_string(),
                    NarrativeSubject::Speaker => "tu".to_string(),
                    NarrativeSubject::World   => "esso".to_string(),
                    NarrativeSubject::Relation=> "noi".to_string(),
                })
            }
            // Verbo copula → "essere" (default), oppure "avere" se anchor lo include
            ("verbo", Some("copula")) => {
                if decision.anchor_words.iter().any(|w| w == "avere") {
                    Some("avere".to_string())
                } else {
                    Some("essere".to_string())
                }
            }
            // Tutti gli altri slot: pick_slot_word
            _ => pick_slot_word(slot, decision, kg_proc, word_topology, lexicon),
        };

        match word {
            Some(w) => fillers.push(SlotFill { spec: slot.clone(), word: w }),
            None => {
                // Slot non riempito: alcuni pattern possono fallire qui.
                // Politica: se è un marcatore mancante, lo creiamo logicamente
                // (sarà reso come "?" o "." in fase di rendering).
                if slot.role == "marcatore" {
                    fillers.push(SlotFill {
                        spec: slot.clone(),
                        word: slot.via.clone().unwrap_or_else(|| "dichiarativo".to_string()),
                    });
                } else {
                    return None; // pattern non istanziabile
                }
            }
        }
    }

    Some(InstantiatedPattern {
        name: schema.name.clone(),
        fillers,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Rendering — ordine sintattico italiano per famiglia di pattern
// ═══════════════════════════════════════════════════════════════════════════

/// Marcatore → glifo finale (la punteggiatura come fisica della grammatica).
fn marker_glyph(marker: &str) -> &'static str {
    match marker {
        "interrogativo" => "?",
        "esclamativo"   => "!",
        _               => ".",
    }
}

/// Capitalizza la prima lettera della stringa.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ─── Pattern: ARTICOLAZIONE ──────────────────────────────────────────
// Es. "Di cosa hai paura?" / "Perché hai paura?" (no preposizione)
// Slot canonici: pronome+interrogativo, preposizione+contesto, verbo+predicato, marcatore+interrogativo
fn render_articolazione(
    inst: &InstantiatedPattern,
    decision: &ActionDecision,
    report: &ComprehensionReport,
) -> Option<Expression> {
    // pronome interrogativo (obbligatorio)
    let pron = inst.find_with_via("pronome", "interrogativo")
        .or_else(|| inst.find("pronome"))?;
    // preposizione (opzionale: solo per "cosa"/"chi"; "perché"/"dove"/"quando" non la prendono)
    let prep = inst.find_with_via("preposizione", "specificazione")
        .or_else(|| inst.find("preposizione"));
    let needs_prep = matches!(pron, "cosa" | "che" | "chi" | "quale");

    // Verbo: priorità (1) verbo del claim estratto dall'utterance del parlante,
    // (2) verbo dello slot dal kg_proc. Il principio: l'articolazione richiama
    // il verbo che il parlante ha usato — "ho paura" → "hai paura?" non
    // "sei paura?". L'utterance è la fonte della forma reciproca.
    let verb_from_utterance: Option<String> = extract_main_verb(&report.utterance);
    let slot_verb = inst.find_with_via("verbo", "predicato")
        .or_else(|| inst.find("verbo"))?;
    let verb_lemma: String = verb_from_utterance.unwrap_or_else(|| slot_verb.to_string());

    // target del claim (gap.from): ciò di cui parla il claim ("paura")
    let claim_target: Option<String> = match &decision.target {
        ActionTarget::Gap { from, .. } => Some(from.clone()),
        _ => None,
    };
    // marcatore
    let mark = inst.find("marcatore").unwrap_or("interrogativo");

    let verb_conj = grammar::conjugate(&verb_lemma, Person::Second, Tense::Present);

    // Composizione: [Prep] [Pron] [Verbo-2sg] [Target]
    let mut text = String::new();
    let mut words_used: Vec<String> = Vec::new();
    if needs_prep && prep.is_some() {
        text.push_str(prep.unwrap());
        text.push(' ');
        text.push_str(pron);
        words_used.push(pron.to_string());
    } else {
        text.push_str(pron);
        words_used.push(pron.to_string());
    }
    text.push(' ');
    text.push_str(&verb_conj);
    words_used.push(verb_lemma.to_string());
    if let Some(t) = claim_target {
        text.push(' ');
        text.push_str(&t);
        words_used.push(t);
    }
    text = capitalize(text.trim());
    text.push_str(marker_glyph(mark));

    Some(Expression { text, words_used })
}

/// Estrae il verbo principale dall'utterance del parlante via lemmatize.
/// `lemmatize` riconosce SOLO verbi (italiano), restituisce l'infinito.
/// Usato dall'articolazione per coniugare in 2a singolare il verbo che il
/// parlante ha usato. (es. "ho paura" → "avere" → "hai paura".)
fn extract_main_verb(utterance: &str) -> Option<String> {
    use crate::topology::grammar::lemmatize;
    let normalized = utterance.to_lowercase();
    for token in normalized.split(|c: char| !c.is_alphabetic()) {
        if token.is_empty() { continue; }
        if let Some(result) = lemmatize(token) {
            return Some(result.infinitive);
        }
    }
    None
}

// ─── Pattern: IDENTIFICAZIONE ────────────────────────────────────────
// Es. "Sono un'entità." / "Sono un fondamento."
fn render_identificazione(
    inst: &InstantiatedPattern,
    decision: &ActionDecision,
    _report: &ComprehensionReport,
    lexicon: &Lexicon,
    kg_proc: &KnowledgeGraph,
) -> Option<Expression> {
    let _pron = inst.find_with_via("pronome", "personale").unwrap_or("io");
    let verb_lemma = inst.find_with_via("verbo", "copula")
        .or_else(|| inst.find("verbo")).unwrap_or("essere");
    // Predicato: prima parola-ancora contenutistica che NON sia:
    //  (a) function word (controllo strutturale via kg_proc: pronomi/articoli/
    //      preposizioni/marcatori/congiunzioni + copule)
    //  (b) verbo del lessico (gli infiniti non sono predicati di "Sono X")
    let is_verb = |w: &str| -> bool {
        lexicon.get(w).map(|p| matches!(p.pos, Some(PartOfSpeech::Verb))).unwrap_or(false)
    };
    let predicato = decision.anchor_words.iter()
        .find(|w| !is_function_word(w, kg_proc) && w.chars().count() >= 3 && !is_verb(w))
        .cloned()?;

    // Italiano: nel parlato standard si elide il pronome soggetto.
    // "Sono un fondamento." è più naturale di "Io sono fondamento."
    // L'articolo indeterminativo si usa per nomi comuni (identificazione
    // di tipo). Per nomi propri (capitalizzati) o predicati astratti molto
    // generici ("entità", "essenza") si potrebbe omettere — per ora applichiamo
    // sempre indeterminativo: il KG produce nomi comuni nei suoi anchor.
    let verb_conj = grammar::conjugate(verb_lemma, Person::First, Tense::Present);
    let predicato_with_art = grammar::with_indefinite_article(&predicato);
    let text = format!("{} {}.",
        capitalize(&verb_conj),
        predicato_with_art);

    Some(Expression {
        text,
        words_used: vec![verb_lemma.to_string(), predicato],
    })
}

// ─── Pattern: RICONOSCIMENTO ─────────────────────────────────────────
// Es. "Hai paura." / "Hai paura del buio."
// Lacaniano: restituisco al parlante il suo posizionamento, in 2a persona.
fn render_riconoscimento(
    inst: &InstantiatedPattern,
    decision: &ActionDecision,
    report: &ComprehensionReport,
) -> Option<Expression> {
    let verb_lemma = inst.find_with_via("verbo", "percettivo")
        .or_else(|| inst.find("verbo")).unwrap_or("sentire");

    // Phase 79: closure-aware. Se il report contiene closure
    // (`closes_prior_gap`), il predicato è il trigger (es. "paura") e lo
    // specifier è la closing_word (es. "buio") — letti direttamente dal
    // percetto, non passati attraverso la pre-shaping di action_reasoning.
    // Per claim normali (senza closure), si cade nel comportamento standard.
    let (predicate, specifier) = if let Some(c) = &report.closes_prior_gap {
        (c.trigger.clone(), Some(c.closing_word.clone()))
    } else {
        let p = match &decision.target {
            ActionTarget::Claim { predicate, .. } => predicate.clone(),
            _ => decision.anchor_words.first().cloned()?,
        };
        let s = if decision.anchor_words.len() >= 2 && decision.anchor_words[0] == p {
            Some(decision.anchor_words[1].clone())
        } else {
            None
        };
        (p, s)
    };
    
    // Controlla se il pattern ha istanziato una preposizione per lo specifier
    let prep = inst.find_with_via("preposizione", "specificazione")
        .or_else(|| inst.find("preposizione"));

    let verb_conj = grammar::conjugate(verb_lemma, Person::Second, Tense::Present);
    
    let mut text = String::new();
    let mut words_used: Vec<String> = vec![verb_lemma.to_string(), predicate.clone()];
    
    text.push_str(&capitalize(&verb_conj));
    text.push(' ');
    text.push_str(&predicate);
    
    if let Some(spec) = specifier {
        text.push(' ');
        if let Some(p) = prep {
            text.push_str(p);
            text.push(' ');
            words_used.push(p.to_string());
        } else {
            // Default preposizione (specificazione)
            text.push_str("di ");
        }
        // Idealmente useremmo preposizioni articolate (del/dello/della/...) ma
        // per ora la preposizione base è sufficiente per "Hai paura del buio."
        // (richiederà il pattern_matcher di leggere prep_articolate da kg_proc).
        text.push_str(&spec);
        words_used.push(spec);
    }
    
    text.push('.');
    
    Some(Expression {
        text,
        words_used,
    })
}

/// Dispatch del rendering in base al nome del pattern.
pub fn render(
    inst: &InstantiatedPattern,
    decision: &ActionDecision,
    report: &ComprehensionReport,
    lexicon: &Lexicon,
    kg_proc: &KnowledgeGraph,
) -> Option<Expression> {
    match inst.name.as_str() {
        "articolazione"  => render_articolazione(inst, decision, report),
        "identificazione" => render_identificazione(inst, decision, report, lexicon, kg_proc),
        "riconoscimento" => render_riconoscimento(inst, decision, report),
        // ricambio + asserzione + altri: lasciati al path esistente
        // (compose_word_response per ricambio, nuclei semantici per asserzione).
        // Restituisce None ⇒ fallback al pipeline corrente di compose().
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. API pubblica unificata
// ═══════════════════════════════════════════════════════════════════════════

/// Punto d'ingresso unico: dato il decision e il report, costruisce il
/// campo procedurale, seleziona il pattern per risonanza, lo istanzia,
/// e lo rende. Restituisce `None` se nessun pattern risuona o non è
/// istanziabile — il caller userà il fallback nuclei.
pub fn compose_from_pattern(
    decision: &ActionDecision,
    report: &ComprehensionReport,
    kg_proc: &KnowledgeGraph,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
) -> Option<Expression> {
    use crate::topology::kg_proc_field::{KgProcActivation, seed_from_comprehension, select_pattern_by_resonance};

    // Costruzione del campo procedurale dal report (percetti → concetti).
    let mut activation = KgProcActivation::new();
    seed_from_comprehension(&mut activation, report, kg_proc);

    // Selezione per risonanza (sostituisce il dispatch pattern_name_for).
    let pattern_name = select_pattern_by_resonance(&activation, kg_proc)?;

    let schema = load_pattern_schema(&pattern_name, kg_proc)?;
    let inst = instantiate(&schema, decision, kg_proc, word_topology, lexicon)?;
    render(&inst, decision, report, lexicon, kg_proc)
}

/// Variante diagnostica: ritorna il pattern selezionato + i punteggi di
/// tutti i pattern, per il log "DECISIONE" in dialogue_educator.
pub fn compose_from_pattern_with_trace(
    decision: &ActionDecision,
    report: &ComprehensionReport,
    kg_proc: &KnowledgeGraph,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
) -> (Option<Expression>, Option<String>, Vec<(String, f64)>) {
    use crate::topology::kg_proc_field::{KgProcActivation, seed_from_comprehension, select_pattern_by_resonance, pattern_scores};

    let mut activation = KgProcActivation::new();
    seed_from_comprehension(&mut activation, report, kg_proc);

    let scores = pattern_scores(&activation, kg_proc);
    let pattern_name = select_pattern_by_resonance(&activation, kg_proc);

    let expr = pattern_name.as_deref()
        .and_then(|name| load_pattern_schema(name, kg_proc))
        .and_then(|schema| instantiate(&schema, decision, kg_proc, word_topology, lexicon))
        .and_then(|inst| render(&inst, decision, report, lexicon, kg_proc));

    (expr, pattern_name, scores)
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::action_reasoning::{ActionKind, ActionTarget, NarrativeSubject};
    use crate::topology::comprehension_report::SpeechAct;
    use crate::topology::deliberation::ActionShape;

    fn build_proc_kg_minimal() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        // Categorie
        kg.add("pronome", RelationType::IsA, "categoria");
        kg.add("preposizione", RelationType::IsA, "categoria");
        kg.add("verbo", RelationType::IsA, "categoria");
        kg.add("marcatore", RelationType::IsA, "categoria");
        // Sottocategorie
        kg.add("interrogativo", RelationType::IsA, "qualificatore");
        kg.add("personale", RelationType::IsA, "qualificatore");
        kg.add("copula", RelationType::IsA, "qualificatore");
        kg.add("predicato", RelationType::IsA, "qualificatore");
        kg.add("specificazione", RelationType::IsA, "qualificatore");
        kg.add("percettivo", RelationType::IsA, "qualificatore");
        // Pronomi interrogativi (con UsedFor chiedere via=X)
        for (p, ruolo) in [("cosa", "oggetto"), ("perché", "causa"), ("dove", "luogo"),
                           ("chi", "persona"), ("come", "modo")] {
            kg.add(p, RelationType::IsA, "pronome");
            kg.add(p, RelationType::IsA, "interrogativo");
            kg.add_edge(crate::topology::relation::TypedEdge {
                subject: p.to_string(),
                relation: RelationType::UsedFor,
                object: "chiedere".to_string(),
                confidence: 0.95,
                source: crate::topology::relation::EdgeSource::Curated,
                via: Some(ruolo.to_string()),
            });
        }
        // Pronomi personali
        kg.add("io", RelationType::IsA, "pronome");
        kg.add("io", RelationType::IsA, "personale");
        kg.add("tu", RelationType::IsA, "pronome");
        kg.add("tu", RelationType::IsA, "personale");
        // Preposizioni
        kg.add("di", RelationType::IsA, "preposizione");
        kg.add("di", RelationType::IsA, "semplice");
        // Verbi base
        kg.add("essere", RelationType::IsA, "verbo");
        kg.add("essere", RelationType::IsA, "copula");
        kg.add("avere", RelationType::IsA, "verbo");
        kg.add("avere", RelationType::IsA, "copula");
        kg.add("sentire", RelationType::IsA, "verbo");
        kg.add("sentire", RelationType::IsA, "percettivo");
        kg.add("temere", RelationType::IsA, "verbo");
        // Marcatori
        kg.add("interrogativo", RelationType::IsA, "marcatore");
        kg.add("dichiarativo", RelationType::IsA, "marcatore");
        // ── Percetti (per il nuovo flow di selezione per risonanza) ──
        kg.add("apertura", RelationType::IsA, "percetto");
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "apertura".to_string(),
            relation: RelationType::Causes,
            object: "chiedere".to_string(),
            confidence: 0.7,
            source: crate::topology::relation::EdgeSource::Curated,
            via: None,
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "apertura".to_string(),
            relation: RelationType::Causes,
            object: "vuoto".to_string(),
            confidence: 0.5,
            source: crate::topology::relation::EdgeSource::Curated,
            via: None,
        });
        kg.add("posizione", RelationType::IsA, "percetto");
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "posizione".to_string(),
            relation: RelationType::Causes,
            object: "restituire".to_string(),
            confidence: 0.4,
            source: crate::topology::relation::EdgeSource::Curated,
            via: None,
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "posizione".to_string(),
            relation: RelationType::Causes,
            object: "posizione".to_string(),
            confidence: 0.4,
            source: crate::topology::relation::EdgeSource::Curated,
            via: None,
        });
        kg.add("domanda", RelationType::IsA, "percetto");
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "domanda".to_string(),
            relation: RelationType::Causes,
            object: "rispondere".to_string(),
            confidence: 0.7,
            source: crate::topology::relation::EdgeSource::Curated,
            via: None,
        });

        // Pattern: articolazione
        kg.add("articolazione", RelationType::IsA, "pattern");
        // Pertinenza: UsedFor chiedere via=vuoto (risuona con percetto "apertura")
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "articolazione".to_string(),
            relation: RelationType::UsedFor,
            object: "chiedere".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("vuoto".to_string()),
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "articolazione".to_string(),
            relation: RelationType::Requires,
            object: "pronome".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("interrogativo".to_string()),
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "articolazione".to_string(),
            relation: RelationType::Requires,
            object: "preposizione".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("specificazione".to_string()),
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "articolazione".to_string(),
            relation: RelationType::Requires,
            object: "verbo".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("predicato".to_string()),
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "articolazione".to_string(),
            relation: RelationType::Requires,
            object: "marcatore".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("interrogativo".to_string()),
        });
        // Pattern: identificazione
        kg.add("identificazione", RelationType::IsA, "pattern");
        // Pertinenza: UsedFor rispondere via=identità (risuona con percetto "domanda" + boost identità)
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "identificazione".to_string(),
            relation: RelationType::UsedFor,
            object: "rispondere".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("identità".to_string()),
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "identificazione".to_string(),
            relation: RelationType::Requires,
            object: "pronome".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("personale".to_string()),
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "identificazione".to_string(),
            relation: RelationType::Requires,
            object: "verbo".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("copula".to_string()),
        });
        // Pattern: riconoscimento
        kg.add("riconoscimento", RelationType::IsA, "pattern");
        // Pertinenza: UsedFor restituire via=posizione (risuona con percetti "posizione" e "chiusura")
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "riconoscimento".to_string(),
            relation: RelationType::UsedFor,
            object: "restituire".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("posizione".to_string()),
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "riconoscimento".to_string(),
            relation: RelationType::Requires,
            object: "pronome".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("personale".to_string()),
        });
        kg.add_edge(crate::topology::relation::TypedEdge {
            subject: "riconoscimento".to_string(),
            relation: RelationType::Requires,
            object: "verbo".to_string(),
            confidence: 0.95,
            source: crate::topology::relation::EdgeSource::Curated,
            via: Some("percettivo".to_string()),
        });
        kg
    }

    fn make_report(utt: &str, kind: &str, subject: &str) -> ComprehensionReport {
        ComprehensionReport {
            utterance: utt.to_string(),
            speech_act: SpeechAct {
                kind: kind.to_string(),
                subject: subject.to_string(),
                description: "test".to_string(),
                addressee: "UI-r1".to_string(),
                implicit_expectation: "test".to_string(),
            },
            symbolic_positions: vec![],
            gaps: vec![],
            inferences: vec![],
            self_relevance: vec![],
            closes_prior_gap: None,
        }
    }

    fn add_gap(report: &mut ComprehensionReport, missing: &str, from: &str) {
        report.gaps.push(crate::topology::comprehension_report::SignifierGap {
            missing: missing.to_string(),
            from: from.to_string(),
            relation: "Requires".to_string(),
            context: None,
            description: format!("vuoto {}", missing),
        });
    }

    #[test]
    fn load_articolazione_schema_from_kg() {
        let kg = build_proc_kg_minimal();
        let schema = load_pattern_schema("articolazione", &kg).unwrap();
        assert_eq!(schema.name, "articolazione");
        assert_eq!(schema.slots.len(), 4);
        assert!(schema.slots.iter().any(|s| s.role == "pronome"
            && s.via.as_deref() == Some("interrogativo")));
        assert!(schema.slots.iter().any(|s| s.role == "preposizione"
            && s.via.as_deref() == Some("specificazione")));
    }

    #[test]
    fn load_pattern_returns_none_for_non_pattern_node() {
        let kg = build_proc_kg_minimal();
        // "cosa" è IsA pronome, non IsA pattern → None
        assert!(load_pattern_schema("cosa", &kg).is_none());
    }

    #[test]
    fn interrogative_for_object_returns_cosa() {
        let kg = build_proc_kg_minimal();
        let anchors: HashSet<&str> = ["cosa", "perché", "dove"].iter().copied().collect();
        assert_eq!(interrogative_for_target("oggetto", &kg, &anchors), Some("cosa".to_string()));
        assert_eq!(interrogative_for_target("causa", &kg, &anchors), Some("perché".to_string()));
        assert_eq!(interrogative_for_target("luogo", &kg, &anchors), Some("dove".to_string()));
    }

    #[test]
    fn render_articolazione_for_paura_yields_di_cosa_hai_paura() {
        let kg = build_proc_kg_minimal();
        let lex = Lexicon::bootstrap();
        let wt = WordTopology::new();
        let decision = ActionDecision {
            kind: ActionKind::InviteToArticulate,
            target: ActionTarget::Gap {
                signifier_missing: "oggetto".to_string(),
                from: "paura".to_string(),
            },
            shape: ActionShape::Question,
            narrative_subject: NarrativeSubject::Speaker,
            anchor_words: vec!["oggetto".to_string(), "paura".to_string(),
                              "cosa".to_string(), "avere".to_string()],
            reasoning: vec![],
        };
        // Phase 79: il report deve contenere il vuoto strutturale per
        // attivare il percetto "apertura" (che risuona con articolazione).
        let mut report = make_report("ho paura", "posizionamento", "Speaker");
        add_gap(&mut report, "oggetto", "paura");
        let expr = compose_from_pattern(&decision, &report, &kg, &wt, &lex).unwrap();
        // Atteso: "Di cosa hai paura?" (oppure simile con verbo coerente)
        assert!(expr.text.starts_with("Di cosa"), "Got: {}", expr.text);
        assert!(expr.text.contains("paura"));
        assert!(expr.text.ends_with("?"));
    }

    #[test]
    fn render_articolazione_for_causa_no_preposition() {
        let kg = build_proc_kg_minimal();
        let lex = Lexicon::bootstrap();
        let wt = WordTopology::new();
        let decision = ActionDecision {
            kind: ActionKind::InviteToArticulate,
            target: ActionTarget::Gap {
                signifier_missing: "causa".to_string(),
                from: "tristezza".to_string(),
            },
            shape: ActionShape::Question,
            narrative_subject: NarrativeSubject::Speaker,
            anchor_words: vec!["causa".to_string(), "tristezza".to_string(),
                              "perché".to_string(), "essere".to_string()],
            reasoning: vec![],
        };
        let mut report = make_report("sono triste", "posizionamento", "Speaker");
        add_gap(&mut report, "causa", "tristezza");
        let expr = compose_from_pattern(&decision, &report, &kg, &wt, &lex).unwrap();
        // "perché" non vuole preposizione: deve iniziare direttamente con "Perché"
        assert!(expr.text.starts_with("Perché"), "Got: {}", expr.text);
        assert!(expr.text.ends_with("?"));
    }

    #[test]
    fn render_identificazione_for_chi_sei_yields_sono_predicato() {
        let kg = build_proc_kg_minimal();
        let lex = Lexicon::bootstrap();
        let wt = WordTopology::new();
        let decision = ActionDecision {
            kind: ActionKind::AnswerOpenQuestion,
            target: ActionTarget::OpenQuestion {
                question_text: "chi sei?".to_string(),
                topic: vec!["chi".to_string()],
            },
            shape: ActionShape::Sentence,
            narrative_subject: NarrativeSubject::Self_,
            anchor_words: vec!["chi".to_string(), "essere".to_string(),
                              "entità".to_string()],
            reasoning: vec![],
        };
        // Phase 79: subject=Self_ attiva il boost identità → identificazione vince.
        let report = make_report("chi sei?", "interrogazione", "Self_");
        let expr = compose_from_pattern(&decision, &report, &kg, &wt, &lex).unwrap();
        assert!(expr.text.starts_with("Sono"), "Got: {}", expr.text);
        assert!(expr.text.contains("entità"));
        // Articolo indeterminativo: "Sono un'entità." (vocale → un')
        assert!(expr.text.contains("un'entità") || expr.text.contains("un entità")
            || expr.text.contains("una entità"),
            "Got: {}", expr.text);
        assert!(expr.text.ends_with("."));
    }

    #[test]
    fn render_riconoscimento_for_io_sono_felice_yields_senti_felice() {
        let kg = build_proc_kg_minimal();
        let lex = Lexicon::bootstrap();
        let wt = WordTopology::new();
        let decision = ActionDecision {
            kind: ActionKind::RecognizeClaim,
            target: ActionTarget::Claim {
                kind: "posizionamento".to_string(),
                predicate: "felice".to_string(),
            },
            shape: ActionShape::Sentence,
            narrative_subject: NarrativeSubject::Speaker,
            anchor_words: vec!["felice".to_string()],
            reasoning: vec![],
        };
        // Phase 79: posizionamento senza vuoto → percetto "posizione" → riconoscimento.
        let report = make_report("io sono felice", "posizionamento", "Speaker");
        let expr = compose_from_pattern(&decision, &report, &kg, &wt, &lex).unwrap();
        assert!(expr.text.contains("felice"), "Got: {}", expr.text);
        // Verbo coniugato in 2a singolare presente
        assert!(expr.text.starts_with("Senti") || expr.text.starts_with("Hai"),
                "Got: {}", expr.text);
    }
}
