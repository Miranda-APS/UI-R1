/// Comprensione dell'atto comunicativo — Phase 55.
///
/// Phase 41b usava KnowledgeBase + delta frattale per classificare. Problema:
/// qualsiasi input che attivava ARMONIA veniva classificato come Greeting.
///
/// Phase 55: usa il KG con IS_A chain per classificare. "ciao" IS_A "saluto" → Greeting.
/// "pioggia" non IS_A "saluto" → non Greeting. Logica, non euristiche.
///
/// "Prometeo non memorizza tutti i saluti: capisce cosa è un saluto via IS_A chain."

use crate::topology::fractal::FractalId;
use crate::topology::lexicon::Lexicon;
use crate::topology::knowledge::{KnowledgeBase, KnowledgeDomain};
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

/// Atto comunicativo rilevato dall'input.
/// Ordine di priorità: Greeting > SelfQuery > Question > EmotionalExpr > Declaration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum InputAct {
    /// Saluto — riconosciuto via dominio Social nella KnowledgeBase
    Greeting,
    /// Domanda su Prometeo stesso — `?` + dominio Self_ nella KnowledgeBase
    SelfQuery,
    /// Domanda generica — solo marcatore sintattico `?`
    Question,
    /// Espressione emotiva — dominio Emotional nella KnowledgeBase (delta EMOZIONE/CORPO)
    EmotionalExpr,
    /// Dichiarazione generica — tutto il resto
    Declaration,
}

/// Soggetto del claim nell'input.
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimAgent {
    /// "io sono/ho/sento X" — il parlante dichiara qualcosa di sé
    Speaker,
    /// "tu sei/hai/senti X" — il parlante dice qualcosa di Prometeo
    Entity,
}

/// Tipo di claim strutturale rilevato nell'input.
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimKind {
    /// "io sono X" / "tu sei X" — identità o appartenenza
    Identity,
    /// "io ho X" / "sento X" / "provo X" — possesso o stato interno
    Feeling,
    /// "io faccio/voglio/penso X" — azione o intenzione
    Action,
}

/// Un claim strutturale rilevato nell'input: chi dice cosa di chi.
///
/// "io sono triste" → SpeakerClaim { agent: Speaker, kind: Feeling, predicate: "triste" }
/// "tu sei bello"   → SpeakerClaim { agent: Entity, kind: Identity, predicate: "bello" }
/// "io voglio capire" → SpeakerClaim { agent: Speaker, kind: Action, predicate: "capire" }
///
/// Non è parsing sintattico completo — è pattern matching sui costrutti
/// più comuni dell'italiano che esprimono relazioni soggetto-predicato.
/// Robusto per frasi semplici; sufficiente per orientare la risposta.
#[derive(Debug, Clone)]
pub struct SpeakerClaim {
    pub agent: ClaimAgent,
    pub kind: ClaimKind,
    /// La parola che porta il predicato (la prima parola-contenuto dopo il verbo)
    pub predicate: String,
    /// Phase 80: categoria del verbo letta strutturalmente dal kg_proc.
    /// Valori: "copula" / "percettivo" / "cognitivo" / "comunicativo" /
    /// "denominativo" / "azione". Permette a `derive_speech_act` di
    /// distinguere casi che il ClaimKind da solo non separa:
    /// Identity+denominativo ("mi chiamo X") vs Identity+copula ("ho un cane").
    /// `None` quando il verbo non è classificato (lessico esterno al kg_proc).
    pub verb_category: Option<String>,
    /// Phase 83: complemento/tema della costruzione, quando il frame del verbo
    /// lo separa dal predicato. Per i verbi dativi ("mi manca mia madre")
    /// `predicate` porta l'emozione lessicalizzata nel verbo ("mancanza") e
    /// `complement` porta il tema ("madre"). `None` per il frame nominativo
    /// di default. Fluisce nello slot `via` della proposizione.
    pub complement: Option<String>,
}

/// Lettura strutturata dell'input corrente.
#[derive(Debug, Clone)]
pub struct InputReading {
    pub act: InputAct,
    /// Intensità dell'atto comunicativo (0..1) — media top-3 delta frattali assoluti
    pub intensity: f64,
    /// Parola più stabile dell'input (se presente nel lessico)
    pub salient_word: Option<String>,
    /// Claim strutturale rilevato (se presente).
    /// "io sono triste" → Some(SpeakerClaim { Speaker, Feeling, "triste" })
    /// "il cane abbaia" → None (nessun soggetto grammaticale rilevante)
    pub speaker_claim: Option<SpeakerClaim>,
    /// Phase 67: proprietà discorsive percepite dal campo post-attivazione.
    pub perceived_properties: Vec<(String, f64)>,
    /// Phase 67: profondità della comprensione — quanti nuclei semantici
    /// l'entità ha estratto dall'input. 0 = non ha capito nulla.
    /// 5+ = comprensione profonda. Usato da deliberate() per decidere
    /// se esplorare (poco capito) o esprimere (molto capito).
    pub comprehension_depth: usize,
}

/// Legge l'atto comunicativo usando logica IS_A dal Knowledge Graph.
///
/// Phase 55: classificazione basata su catene IS_A nel KG.
/// "ciao" IS_A "saluto" → Greeting.
/// "triste" IS_A "emozione" → EmotionalExpr.
/// "pioggia" non IS_A nessun concetto chiave → Declaration.
///
/// Il KG come parametro opzionale: se None, fallback alla KB+delta (backward compat test).
pub fn read_input(
    raw_words: &[String],
    raw_text: &str,
    frattale_delta: &[(FractalId, f64)],
    knowledge_base: &KnowledgeBase,
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
    kg_proc: Option<&KnowledgeGraph>,
) -> InputReading {
    // ── Parola più stabile dell'input ────────────────────────────────────────
    let salient_word = raw_words.iter()
        .filter(|w| w.len() >= 3)
        .filter_map(|w| lexicon.get(w).map(|p| (w.clone(), p.stability)))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(w, _)| w);

    // ── Intensità: media top-3 delta assoluti ────────────────────────────────
    let intensity = {
        let mut deltas: Vec<f64> = frattale_delta.iter().map(|(_, d)| d.abs()).collect();
        deltas.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let top3: Vec<f64> = deltas.iter().take(3).copied().collect();
        if top3.is_empty() { 0.0 } else { top3.iter().sum::<f64>() / top3.len() as f64 }
    };

    // ── `?` come unico marcatore sintattico ──────────────────────────────────
    let has_question_mark = raw_text.contains('?');

    // ── Phase 55: Classificazione via IS_A chain nel KG ──────────────────────
    // Per ogni parola dell'input, risaliamo la catena IS_A. Se troviamo un
    // concetto chiave (saluto, emozione, sentimento, identità), classifichiamo.
    // Concetti chiave per Greeting: "saluto", "salutare"
    // Concetti chiave per Emotional: "emozione", "sentimento", "stato_d_animo"
    // Concetti chiave per Self: "identità", "coscienza", "sé"
    let (has_greeting, has_emotional, has_self_ref) = if let Some(kg) = kg {
        let mut greeting = false;
        let mut emotional = false;
        let mut self_ref = false;

        for word in raw_words.iter() {
            // IS_A diretti (1 hop) — non transitivi profondi per efficienza
            let parents: Vec<&str> = kg.query_objects(word, RelationType::IsA);
            for parent in &parents {
                let p = parent.to_lowercase();
                if p == "saluto" || p == "salutare" || p == "saluti" {
                    greeting = true;
                }
                if p == "emozione" || p == "sentimento" || p == "stato_d_animo"
                    || p == "sensazione" || p == "affetto" {
                    emotional = true;
                }
                if p == "identità" || p == "coscienza" || p == "persona" {
                    self_ref = true;
                }
            }
            // Anche IS_A 2-hop: "ciao" IS_A "interiezione" IS_A "saluto"?
            // Per ora 1-hop basta se il KG è ben fatto.

            // Fallback: la parola stessa è un concetto chiave
            let w = word.to_lowercase();
            if w == "ciao" || w == "salve" || w == "buongiorno" || w == "buonasera" {
                // Questi sono IS_A "saluto" nel KG. Se il KG non li ha, li riconosciamo
                // direttamente come safety net. NON è una lista di tutti i saluti:
                // è un piccolo bootstrap per i saluti più comuni.
                greeting = true;
            }
        }
        (greeting, emotional, self_ref)
    } else {
        // Fallback senza KG (test): usa la KB come prima
        let relevant = knowledge_base.retrieve_for_delta(raw_words, frattale_delta);
        let s = relevant.iter().any(|e| e.domain == KnowledgeDomain::Social);
        let e = relevant.iter().any(|e| e.domain == KnowledgeDomain::Emotional);
        let i = relevant.iter().any(|e| e.domain == KnowledgeDomain::Self_);
        (s, e, i)
    };

    // ── Rilevamento claim strutturale soggetto/predicato ────────────────────
    // Rileva "io sono/ho/sento X" e "tu sei/hai X" come claim espliciti.
    // Non è parsing completo: è pattern matching sui costrutti più frequenti.
    // Robusto per frasi semplici; sufficiente per orientare stance e risposta.
    let speaker_claim = detect_speaker_claim(raw_words, lexicon, kg, kg_proc);

    // ── Classificazione (ordine di priorità) ─────────────────────────────────
    // I claim emozionali del parlante ("io sono triste") elevano Declaration
    // a EmotionalExpr anche se il KG non ha riconosciuto la parola via IS_A.
    // Questo permette risposte empatiche a stati interni espressi strutturalmente.
    let has_emotional_claim = speaker_claim.as_ref()
        .map(|c| c.agent == ClaimAgent::Speaker && c.kind == ClaimKind::Feeling)
        .unwrap_or(false);

    let act = if has_greeting {
        InputAct::Greeting
    } else if has_question_mark && has_self_ref {
        InputAct::SelfQuery
    } else if has_question_mark {
        InputAct::Question
    } else if has_emotional || has_emotional_claim {
        InputAct::EmotionalExpr
    } else {
        InputAct::Declaration
    };

    InputReading { act, intensity, salient_word, speaker_claim, perceived_properties: vec![], comprehension_depth: 0 }
}

/// Phase 80: rileva un claim strutturale soggetto/predicato nell'input.
///
/// L'identificazione del verbo, della sua categoria semantica e del kind del
/// claim è completamente STRUTTURALE — legge dal kg_proc le triple
/// `<verbo> IsA verbo`, `<verbo> IsA copula|percettivo|cognitivo|comunicativo|
/// denominativo|azione`. Nessuna lista hardcoded di verbi italiani in Rust.
/// Aggiungere/rimuovere verbi è curation, mai modifica di codice.
///
/// Pipeline:
///   1. Pronome esplicito (io/mi → Speaker; tu/ti → Entity), se presente.
///   2. Per ogni token, prova a lemmatizzarlo come verbo (irregolari coperti
///      da `grammar::lemmatize`, regolari coperti da fallback morfologico
///      validato sul kg_proc).
///   3. Il primo verbo lemmatizzato che ha categoria nota nel kg_proc è il
///      verbo del claim. La sua categoria determina il ClaimKind:
///        denominativo → Identity (presentazione)
///        percettivo   → Feeling  (validato: predicato deve essere stato interno)
///        cognitivo    → Action
///        comunicativo → Action
///        azione       → Action
///        copula       → Identity se predicato non è stato interno,
///                       Feeling   se predicato è stato interno
///   4. L'agente è determinato dal pronome esplicito (se presente) o dalla
///      persona del verbo (1sg → Speaker, 2sg → Entity).
///   5. Il predicato è la prima parola-contenuto dopo il verbo, saltando
///      le function-word identificate strutturalmente dal kg_proc.
///
/// Restituisce None se nessun verbo classificato emerge dall'input.
pub fn detect_speaker_claim(
    raw_words: &[String],
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<SpeakerClaim> {
    if raw_words.is_empty() { return None; }

    // ── 1. Pronome esplicito ─────────────────────────────────────────────
    // "io"/"mi"/"noi"/"ci" → Speaker. "tu"/"ti"/"voi"/"vi" → Entity.
    let explicit_agent: Option<ClaimAgent> = raw_words.iter()
        .find_map(|w| match w.as_str() {
            "io" | "mi" | "noi" | "ci" => Some(ClaimAgent::Speaker),
            "tu" | "ti" | "voi" | "vi" => Some(ClaimAgent::Entity),
            _ => None,
        });

    // ── 2. Trova il primo verbo classificato ─────────────────────────────
    let mut verb_match: Option<(usize, String, crate::topology::grammar::Person, &'static str)> = None;
    for (pos, w) in raw_words.iter().enumerate() {
        if let Some((infinitive, person)) = lemma_of_verb(w, kg_proc) {
            if let Some(category) = verb_category(&infinitive, kg_proc) {
                verb_match = Some((pos, infinitive, person, category));
                break;
            }
        }
    }
    let (verb_pos, infinitive, person, category) = verb_match?;

    // ── 2.bis Frame di costruzione: i verbi dativi rimappano i ruoli ──────
    //   "mi manca mia madre" → esperiente=clitico (mi→Speaker), emozione
    //   lessicalizzata nel verbo (Expresses→"mancanza"), tema=nome dopo il
    //   verbo ("madre"). Nessun caso Rust verbo-specifico: il frame e
    //   l'emozione vivono nel kg_proc come dato.
    if verb_frame(&infinitive, kg_proc).as_deref() == Some("dativo") {
        return build_dative_claim(raw_words, verb_pos, &infinitive, explicit_agent, kg_proc);
    }

    // ── 3. Agente: pronome vince; altrimenti deduzione dalla persona ─────
    use crate::topology::grammar::Person;
    let agent = match explicit_agent {
        Some(a) => a,
        None => match person {
            Person::First  | Person::FirstPlural  => ClaimAgent::Speaker,
            Person::Second | Person::SecondPlural => ClaimAgent::Entity,
            Person::Third  | Person::ThirdPlural  => return None, // 3sg/pl non sono claim diretti
        },
    };

    // ── 4. Estrai il predicato — prima parola-contenuto dopo il verbo ────
    let predicate = raw_words.iter()
        .skip(verb_pos + 1)
        .find(|w| !is_kg_proc_function_word(w, kg_proc))
        .cloned()?;

    // ── 5. ClaimKind dalla categoria del verbo ───────────────────────────
    //   La mappa categoria→kind è strutturale (lettura della IsA chain),
    //   non un dispatch: ogni categoria denota un modo di posizionarsi che
    //   ClaimKind classifica con le sue 3 varianti.
    let kind = match category {
        "denominativo" => ClaimKind::Identity,    // "mi chiamo X"
        "percettivo"   => {
            // "sento/provo X" è Feeling solo se X è stato interno.
            // "sento la voce" → percezione esterna, non claim: rifiuta.
            if is_inner_state(&predicate, lexicon, kg) {
                ClaimKind::Feeling
            } else {
                return None;
            }
        }
        "cognitivo" | "comunicativo" | "azione" => ClaimKind::Action,
        "copula"       => {
            // "io sono triste" → Feeling; "io ho un cane" → Identity.
            // Il predicato decide se il posizionamento è di stato interno.
            if is_inner_state(&predicate, lexicon, kg) {
                ClaimKind::Feeling
            } else {
                ClaimKind::Identity
            }
        }
        _ => return None, // categoria non riconosciuta
    };

    Some(SpeakerClaim {
        agent,
        kind,
        predicate,
        verb_category: Some(category.to_string()),
        complement: None,
    })
}

/// Phase 83: costruisce un claim per un verbo a costruzione dativa.
/// L'esperiente è il clitico (mi/ti/ci/vi → già in `explicit_agent`);
/// l'emozione è lessicalizzata nel verbo (`Expresses` nel kg_proc); il tema è
/// la prima parola-contenuto dopo il verbo (i determinanti si saltano).
/// `None` se manca il clitico o l'emozione insegnata — niente fallback
/// rumoroso. Nessuna logica verbo-specifica: tutto è dato del kg_proc.
fn build_dative_claim(
    raw_words: &[String],
    verb_pos: usize,
    infinitive: &str,
    explicit_agent: Option<ClaimAgent>,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<SpeakerClaim> {
    let agent = explicit_agent?;
    let kp = kg_proc?;
    let emotion = kp.query_objects(infinitive, RelationType::Expresses)
        .into_iter().next()?.to_string();
    let theme = raw_words.iter()
        .skip(verb_pos + 1)
        .find(|w| !is_kg_proc_function_word(w, kg_proc))
        .cloned();
    Some(SpeakerClaim {
        agent,
        kind: ClaimKind::Feeling,
        predicate: emotion,
        verb_category: Some("percettivo".to_string()),
        complement: theme,
    })
}

/// Phase 83: legge il frame di costruzione del verbo dal kg_proc (oggi solo
/// `dativo`; l'assenza = frame nominativo di default). I frame sono una
/// tabella finita — il minimo denominatore della struttura argomentale —
/// mentre i verbi sono infiniti e quasi tutti a default.
fn verb_frame(infinitive: &str, kg_proc: Option<&KnowledgeGraph>) -> Option<String> {
    let kp = kg_proc?;
    let parents = kp.query_objects(infinitive, RelationType::IsA);
    const FRAMES: &[&str] = &["dativo"];
    FRAMES.iter()
        .find(|f| parents.iter().any(|p| p.eq_ignore_ascii_case(f)))
        .map(|f| f.to_string())
}

/// Phase 80: lemmatizza una forma verbale italiana usando, in ordine:
///   1. `grammar::lemmatize` (irregolari + suffissi noti)
///   2. Match diretto del token come infinito nel kg_proc
///   3. Fallback morfologico per il presente regolare (-are/-ere/-ire),
///      validato sul kg_proc (il candidato deve essere `IsA verbo`).
///
/// Restituisce `(infinito, persona)` se il verbo è riconosciuto, `None`
/// altrimenti. Il fallback è autocorrettivo: se il candidato infinitivo
/// non è nel kg_proc, il match fallisce — niente falsi positivi su
/// sostantivi che terminano in `-o`/`-i`.
fn lemma_of_verb(
    word: &str,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<(String, crate::topology::grammar::Person)> {
    use crate::topology::grammar::{lemmatize, Person};

    let w = word.to_lowercase();

    // (1) Irregolari + suffissi noti — ma validati strutturalmente.
    //     `grammar::lemmatize` ha falsi positivi su forme regolari: es.
    //     "chiamo" (= chiamare 1sg) viene letto come `chare` 1pl perché
    //     termina in "iamo". Quando il kg_proc è disponibile, validiamo
    //     l'infinito proposto: se non è `IsA verbo`, scartiamo e proviamo
    //     il fallback morfologico (che produrrà "chiamare", valido).
    if let Some(r) = lemmatize(&w) {
        match kg_proc {
            None => return Some((r.infinitive, r.person)),
            Some(kp) if is_kg_proc_isa(kp, &r.infinitive, "verbo") =>
                return Some((r.infinitive, r.person)),
            _ => { /* falso positivo: continua al fallback morfologico */ }
        }
    }

    let kp = kg_proc?;

    // (2) Il token stesso è già infinito? (es. "essere", "andare" in un'enumerazione)
    if is_kg_proc_isa(kp, &w, "verbo") {
        return Some((w.clone(), Person::Third)); // infinito non porta persona
    }

    // (3) Fallback morfologico: prova suffissi del presente regolare.
    //     Lo stem deve avere lunghezza ≥3 per evitare match su monosillabi;
    //     il candidato infinito deve essere validato come verbo nel kg_proc.
    const PRESENT_SUFFIXES: &[(&str, Person)] = &[
        ("iamo", Person::FirstPlural),
        ("ate",  Person::SecondPlural),
        ("ete",  Person::SecondPlural),
        ("ite",  Person::SecondPlural),
        ("ano",  Person::ThirdPlural),
        ("ono",  Person::ThirdPlural),
        ("o",    Person::First),
        ("i",    Person::Second),
        ("a",    Person::Third),
        ("e",    Person::Third),
    ];

    for (suf, person) in PRESENT_SUFFIXES {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() < 3 { continue; }
            for inf_suf in &["are", "ere", "ire"] {
                let candidate = format!("{}{}", stem, inf_suf);
                if is_kg_proc_isa(kp, &candidate, "verbo") {
                    return Some((candidate, *person));
                }
            }
        }
    }

    None
}

/// Phase 80: legge la categoria del verbo (copula/percettivo/cognitivo/
/// comunicativo/denominativo/azione) dal kg_proc tramite la sua IsA chain.
///
/// Quando un verbo è in più categorie (es. "sentire IsA azione + IsA
/// percettivo"), prevale quella più specifica nel posizionamento:
/// denominativo > percettivo > cognitivo > comunicativo > copula > azione.
/// Questa priorità riflette la specificità semantica del verbo (non un
/// dispatch arbitrario): "chiamare" come denominativo è informativo,
/// "chiamare" come azione generica è una banalità.
fn verb_category(
    infinitive: &str,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<&'static str> {
    let kp = kg_proc?;
    let parents = kp.query_objects(infinitive, RelationType::IsA);
    const PRIORITY: &[&str] = &[
        "denominativo", "percettivo", "cognitivo",
        "comunicativo", "copula", "azione",
    ];
    PRIORITY.iter().find(|cat| parents.contains(cat)).copied()
}

/// Phase 80: una parola è "funzione" se il kg_proc la classifica come
/// pronome, articolo, preposizione, marcatore, congiunzione, oppure
/// `IsA copula`. La distinzione vive nei dati: aggiungere/togliere
/// parole-funzione è curation.
fn is_kg_proc_function_word(word: &str, kg_proc: Option<&KnowledgeGraph>) -> bool {
    let Some(kp) = kg_proc else { return false; };
    let w = word.to_lowercase();
    let parents = kp.query_objects(&w, RelationType::IsA);
    for p in &parents {
        let p_lc = p.to_lowercase();
        if matches!(p_lc.as_str(),
            "pronome" | "articolo" | "preposizione" |
            "marcatore" | "congiunzione" | "copula" |
            "determinante") {
            return true;
        }
    }
    false
}

/// Helper: verifica `subject IsA target` 1-hop nel kg_proc.
fn is_kg_proc_isa(kg_proc: &KnowledgeGraph, subject: &str, target: &str) -> bool {
    kg_proc.query_objects(subject, RelationType::IsA)
        .iter().any(|p| p.to_lowercase() == target.to_lowercase())
}

/// Phase 73: rileva una presentazione di nome del parlante.
/// Pattern riconosciuti:
///   "mi chiamo X"       → Some("X")
///   "io mi chiamo X"    → Some("X")
///   "il mio nome è X"   → Some("X")
///   "sono X" (X è una parola sconosciuta al KG e al lessico)  → Some("X")
///
/// X deve essere una parola "non strutturale" — non in lessico stabile,
/// non in KG, lunghezza ≥ 3. Questo riduce falsi positivi tipo
/// "sono triste" → "triste" (che è in lessico ed è un aggettivo emotivo).
pub fn detect_name_introduction(
    raw_words: &[String],
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
) -> Option<String> {
    if raw_words.len() < 2 { return None; }
    let words: Vec<&str> = raw_words.iter().map(|w| w.as_str()).collect();

    // Pattern "mi chiamo X" / "io mi chiamo X" / "chiamo X" (chiamarsi 1a sg)
    if let Some(p) = words.iter().position(|&w| w == "chiamo") {
        // Verifica che sia preceduto da "mi" o sia in posizione iniziale dopo "io"
        let preceded_by_mi = p > 0 && (words[p - 1] == "mi" ||
            (p >= 2 && words[p - 2] == "io" && words[p - 1] == "mi"));
        let initial_or_after_io = p == 0 || (p == 1 && words[0] == "io");
        if preceded_by_mi || initial_or_after_io {
            // Il nome è la prima parola-contenuto dopo "chiamo"
            for &w in words.iter().skip(p + 1) {
                if is_likely_proper_name(w, lexicon, kg) {
                    return Some(w.to_string());
                }
            }
        }
    }

    // Pattern "il mio nome è X"
    if let Some(p) = words.iter().position(|&w| w == "nome") {
        if p + 1 < words.len() && (words[p + 1] == "è" || words[p + 1] == "e") {
            for &w in words.iter().skip(p + 2) {
                if is_likely_proper_name(w, lexicon, kg) {
                    return Some(w.to_string());
                }
            }
        }
    }

    None
}

/// Euristica: una parola è probabilmente un nome proprio se NON è nel
/// lessico stabile e NON è nel KG e ha lunghezza ≥ 3 e non è una
/// function word.
fn is_likely_proper_name(
    word: &str,
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
) -> bool {
    if word.len() < 3 { return false; }
    if lexicon.is_function_word(word) { return false; }
    // Salta articoli e preposizioni
    if matches!(word, "un" | "una" | "uno" | "il" | "la" | "lo" | "i" | "gli" | "le") {
        return false;
    }
    // Se è già un sostantivo/aggettivo conosciuto al KG, probabilmente non è un nome proprio
    if let Some(kg) = kg {
        if !kg.query_objects(word, RelationType::IsA).is_empty() {
            return false;
        }
    }
    // Se è nel lessico con stabilità alta, probabilmente non è un nome
    if let Some(pat) = lexicon.get(word) {
        if pat.stability > 0.5 && pat.exposure_count > 5 {
            return false;
        }
    }
    true
}

/// Phase 80: verifica se una parola denota uno stato interno del soggetto
/// (emozione, sentimento, sensazione, bisogno) leggendo strutturalmente la
/// rete di relazioni IsA / Has nel KG semantico.
///
/// Una parola è "stato interno" se:
///   (a) la sua catena IsA (1-2 hop) raggiunge uno dei super-tipi semantici
///       {emozione, sentimento, stato_d_animo, sensazione, affetto, umore,
///        bisogno, dolore, sofferenza}, oppure
///   (b) ha una relazione `Has bisogno` / `Has sofferenza` / `Has mancanza`
///       — segnale strutturale che il significante porta un bisogno (es. il
///       KG codifica "fame Has bisogno" anziché "fame IsA bisogno": gli
///       stati corporei *hanno* bisogni, non li *sono*).
///
/// I super-tipi sono concetti (non parole italiane) — l'insieme è piccolo
/// e stabile, non cresce con il vocabolario. Qualunque significante che
/// risale a uno di essi (es. "fame" → "bisogno", "buio" → niente di emotivo)
/// emerge come stato interno o no.
fn is_inner_state(word: &str, lexicon: &Lexicon, kg: Option<&KnowledgeGraph>) -> bool {
    const INNER_STATE_ROOTS: &[&str] = &[
        "emozione", "sentimento", "stato_d_animo", "sensazione",
        "affetto", "umore", "bisogno", "dolore", "sofferenza",
    ];
    const HAS_SIGNALS: &[&str] = &[
        "bisogno", "sofferenza", "mancanza", "dolore",
    ];

    let Some(kg) = kg else {
        // Senza KG: fallback al lessico — POS=Adjective con alta Valenza è
        // segnale debole di parola valutativa (es. "triste" senza KG).
        // Usato solo dai test che girano senza KG.
        if let Some(pat) = lexicon.get(word) {
            if matches!(pat.pos, Some(crate::topology::grammar::PartOfSpeech::Adjective))
                && pat.stability > 0.3
                && pat.signature.get(crate::topology::primitive::Dim::Valenza) > 0.6
            {
                return true;
            }
        }
        return false;
    };

    let w = word.to_lowercase();

    // (a) IsA chain 1-2 hop verso i super-tipi.
    let direct_parents: Vec<&str> = kg.query_objects(&w, RelationType::IsA);
    for p in &direct_parents {
        if INNER_STATE_ROOTS.contains(&p.to_lowercase().as_str()) {
            return true;
        }
        // 2-hop: il genitore IsA un super-tipo? (es. fame → bisogno-fisiologico → bisogno)
        for gp in kg.query_objects(p, RelationType::IsA) {
            if INNER_STATE_ROOTS.contains(&gp.to_lowercase().as_str()) {
                return true;
            }
        }
    }

    // (b) Has signals: parola che porta un bisogno/sofferenza è stato interno.
    let has_targets: Vec<&str> = kg.query_objects(&w, RelationType::Has);
    for t in &has_targets {
        if HAS_SIGNALS.contains(&t.to_lowercase().as_str()) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // KnowledgeBase con ancore concettuali — specchia seed_conceptual_anchors() in engine.rs
    fn kb_with_anchors() -> KnowledgeBase {
        let mut kb = KnowledgeBase::new();
        kb.teach_concept(
            KnowledgeDomain::Social,
            "un saluto è un atto di avvicinamento sociale: chi saluta vuole connessione",
            "ciao",
            vec![63, 47], // ARMONIA + COMUNICAZIONE
        );
        kb.teach_concept(
            KnowledgeDomain::Emotional,
            "un'emozione è uno stato interno vissuto: chi esprime un'emozione cerca risonanza",
            "paura",
            vec![58, 33], // EMOZIONE + CORPO
        );
        kb.teach_concept(
            KnowledgeDomain::Self_,
            "un'indagine sull'identità è la domanda su chi è Prometeo, cosa sente, cosa vuole",
            "sei",
            vec![32], // IDENTITA
        );
        kb
    }

    fn empty_delta() -> Vec<(FractalId, f64)> { vec![] }
    fn lex() -> Lexicon { Lexicon::bootstrap() }

    #[test]
    fn test_greeting_via_word_match() {
        // "ciao" è la parola-campione del concetto Social → word_match in retrieve_for_delta
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["ciao".to_string()],
            "ciao",
            &empty_delta(),
            &kb,
            &lex,
            None,
            None,
        );
        assert_eq!(r.act, InputAct::Greeting);
    }

    #[test]
    fn test_greeting_via_fractal_delta() {
        // "salve" non è la parola-campione, ma attiva ARMONIA(63) → delta_match → Social
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(63u32, 0.15f64)]; // ARMONIA delta > 0.05
        let r = read_input(
            &["salve".to_string()],
            "salve",
            &delta,
            &kb,
            &lex,
            None,
            None,
        );
        assert_eq!(r.act, InputAct::Greeting,
            "salve dovrebbe essere riconosciuto come saluto via delta ARMONIA");
    }

    #[test]
    fn test_self_query_via_word_match() {
        // "sei" è la parola-campione del concetto Self_ + `?` → SelfQuery
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["chi".to_string(), "sei".to_string()],
            "chi sei?",
            &empty_delta(),
            &kb,
            &lex,
            None,
            None,
        );
        assert_eq!(r.act, InputAct::SelfQuery);
    }

    #[test]
    fn test_self_query_via_fractal_delta() {
        // Una domanda che attiva IDENTITA(32) → SelfQuery anche senza "sei"
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(32u32, 0.12f64)]; // IDENTITA delta
        let r = read_input(
            &["cosa".to_string(), "pensi".to_string()],
            "cosa pensi?",
            &delta,
            &kb,
            &lex,
            None,
            None,
        );
        assert_eq!(r.act, InputAct::SelfQuery,
            "domanda con delta IDENTITA → SelfQuery anche senza parola-campione");
    }

    #[test]
    fn test_generic_question() {
        // `?` senza Social/Self_/Emotional → Question generica
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["cosa".to_string(), "succede".to_string()],
            "cosa succede?",
            &empty_delta(),
            &kb,
            &lex,
            None,
            None,
        );
        assert_eq!(r.act, InputAct::Question);
    }

    #[test]
    fn test_emotional_expr_via_word_match() {
        // "paura" è la parola-campione del concetto Emotional → word_match → EmotionalExpr
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["ho".to_string(), "paura".to_string()],
            "ho paura",
            &empty_delta(),
            &kb,
            &lex,
            None,
            None,
        );
        assert_eq!(r.act, InputAct::EmotionalExpr);
    }

    #[test]
    fn test_emotional_expr_via_fractal_delta() {
        // EMOZIONE(58) delta → Emotional concept → EmotionalExpr
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(58u32, 0.45f64)]; // EMOZIONE delta
        let r = read_input(
            &["tristezza".to_string()],
            "sento tristezza",
            &delta,
            &kb,
            &lex,
            None,
            None,
        );
        assert_eq!(r.act, InputAct::EmotionalExpr,
            "qualunque parola che attiva EMOZIONE → EmotionalExpr");
    }

    #[test]
    fn test_declaration_default() {
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["penso".to_string(), "quindi".to_string(), "sono".to_string()],
            "penso quindi sono",
            &empty_delta(),
            &kb,
            &lex,
            None,
            None,
        );
        assert_eq!(r.act, InputAct::Declaration);
    }

    #[test]
    fn test_intensity_from_delta() {
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(58u32, 0.6f64), (32u32, 0.4f64), (33u32, 0.2f64)];
        let r = read_input(&[], "", &delta, &kb, &lex, None, None);
        // avg top-3 assoluti = (0.6 + 0.4 + 0.2) / 3 ≈ 0.4
        assert!((r.intensity - 0.4).abs() < 0.01,
            "intensity attesa ~0.4, ottenuta {}", r.intensity);
    }

    #[test]
    fn test_no_anchors_fallback() {
        // Senza ancore concettuali, solo `?` e Declaration funzionano
        let lex = lex();
        let kb = KnowledgeBase::new(); // vuota
        let r = read_input(&["ciao".to_string()], "ciao", &empty_delta(), &kb, &lex, None, None);
        // Senza ancora Social, "ciao" → Declaration (non riconosciuto)
        assert_eq!(r.act, InputAct::Declaration,
            "senza KnowledgeBase, ciao non è riconoscibile come saluto");
        let r2 = read_input(&["cosa".to_string()], "cosa succede?", &empty_delta(), &kb, &lex, None, None);
        assert_eq!(r2.act, InputAct::Question,
            "senza KB, `?` mantiene Question come fallback sintattico");
    }
}
