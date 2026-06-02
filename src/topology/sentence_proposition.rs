//! Phase 81 — la frase come proposizione, non come sequenza di token.
//!
//! Francesco (2026-05-15): "bisogna mettere l'accento sulla comprensione
//! delle frasi non solo più delle parole".
//!
//! ## Cosa risolve
//!
//! Fino a Phase 80, `detect_speaker_claim` classifica ciascun token
//! (verbo / pronome / preposizione / predicato) ma la frase nel suo insieme
//! non viene mai vista come unità. Conseguenze:
//!  - "Ho paura del futuro" → gap "oggetto" su paura viene aperto, anche
//!    se "futuro" è strutturalmente già l'oggetto (legato dalla preposizione
//!    "del" che il kg_proc classifica `IsA preposizione + IsA specificazione`).
//!  - "Vado al mare" → predicato="mare", ma `mare` è il *via* del movimento,
//!    non l'oggetto di un'azione astratta.
//!  - Il KG semantico (rete di triple) non viene mai confrontato con la
//!    frase: UI-r1 sa che "paura Causes pianto" ma non collega l'asserzione
//!    "ho paura" a quella mappa.
//!
//! ## Architettura
//!
//! Una frase = una piccola rete di proposizioni che si sovrappone al
//! kg_sem. L'enunciato inscrive un sotto-grafo nel grafo del mondo, e
//! UI-r1 capisce nella misura in cui sa dire come quel sotto-grafo si
//! appoggia a (o devia da) ciò che già sa.
//!
//! ```text
//! raw_words + speaker_claim + kg_proc
//!   │
//!   ▼  extract_proposition
//! SentenceProposition {
//!     subject:  Speaker | Entity | World(s) | Variable(w),
//!     relation: derivata da verb_category + via del verbo nel kg_proc
//!               (essere/identità → IsA, avere/stato + inner → FeelsAs,
//!                chiamare/denominare → IsA, sentire/percepire → FeelsAs,
//!                pensare/credere → Expresses, andare/spostare → Does, ...)
//!     object:   Word(predicato) | Variable(pronome interrogativo) | None,
//!     via:      parola dopo una preposizione di specificazione, se presente
//!               ("ho paura del futuro" → via=Some("futuro"))
//!     polarity: !negated
//! }
//!   │
//!   ▼  confront_with_kg(prop, kg_sem)
//! KgConfrontation {
//!     matches:        la triple esiste già nel kg_sem?
//!     inferences:     catene 2-hop di vicini coerenti
//!     contradictions: archi OppositeOf rilevanti
//!     slot_filled:    quali slot della proposizione sono ancorati al kg_sem
//! }
//! ```
//!
//! La proposizione è quello su cui `derive_speech_act` e `derive_gaps`
//! agiranno (Phase 81b → vedi integrazione in ComprehensionReport).

use crate::topology::input_reading::{ClaimAgent, ClaimKind, SpeakerClaim};
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

// ═══════════════════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════════════════

/// Il soggetto di una proposizione enunciata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubjectRef {
    /// Il parlante ("io", "mi", "noi", "ci"; verbo in 1a persona).
    Speaker,
    /// UI-r1 stessa ("tu", "ti"; verbo in 2a persona).
    Entity,
    /// Un soggetto del mondo di 3a persona ("il sole", "Marco", ...).
    World(String),
    /// Una variabile interrogativa ("chi", "cosa", "quale", ...).
    Variable(String),
}

/// L'oggetto di una proposizione enunciata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectRef {
    /// Una parola concreta (predicato, complemento).
    Word(String),
    /// Una variabile interrogativa nell'oggetto ("chi sei?" → "chi").
    Variable(String),
}

/// La proposizione che una frase porta.
///
/// È la lettura *strutturale* dell'enunciato come triple, prima di qualsiasi
/// dispatch comportamentale. Tutte le decisioni a valle (`derive_speech_act`,
/// `derive_gaps`, pattern matcher) leggono da qui anziché ri-parsare token.
#[derive(Debug, Clone)]
pub struct SentenceProposition {
    pub subject:  SubjectRef,
    pub relation: RelationType,
    pub object:   Option<ObjectRef>,
    /// Parola dopo una preposizione di specificazione, se presente.
    /// Es. "ho paura **del futuro**" → via=Some("futuro").
    /// È il complemento che ANCORA l'oggetto al mondo: senza il via, il claim
    /// resta interno ("ho paura"); con il via, è già articolato.
    pub via: Option<String>,
    /// Polarità della proposizione (false se l'utterance contiene una negazione
    /// che precede il verbo). Per ora rilevata via parola letterale "non".
    pub polarity: bool,
}

/// Risultato del confronto fra una SentenceProposition e il kg_sem.
///
/// Phase 81 introduce solo i campi base; le inferenze 2-hop riusano la
/// pipeline esistente (`comprehension_graph::syllogisms`) e arriveranno
/// gradualmente come slot ancorati.
#[derive(Debug, Clone, Default)]
pub struct KgConfrontation {
    /// La triple (subject, relation, object) esiste già nel kg_sem?
    /// Quando il subject è Speaker/Entity (ossia non un nodo del mondo),
    /// il match riguarda solo (object, relation, world).
    pub matches: bool,
    /// Archi OppositeOf che rendono la proposizione strutturalmente in
    /// tensione col KG (es. "non ho paura" ↔ "paura OppositeOf coraggio").
    pub contradictions: Vec<(String, String)>,
    /// Quali slot della proposizione hanno un riscontro nel kg_sem
    /// (almeno 1 arco IsA o uno tra Has/Causes/...). Permette di dire:
    /// "object ha radici nel KG, via no" — utile per scegliere come
    /// rispondere.
    pub object_in_kg: bool,
    pub via_in_kg:    bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Estrazione
// ═══════════════════════════════════════════════════════════════════════════

/// Estrae la proposizione strutturale che la frase porta.
///
/// Riceve il `SpeakerClaim` già rilevato da `detect_speaker_claim` (che
/// fornisce verbo+agente+predicato) e lo arricchisce leggendo i restanti
/// token strutturalmente:
///   - `via`: prima parola-contenuto dopo una preposizione di specificazione
///     (di/del/della/per/su/con/...) — `IsA specificazione` nel kg_proc.
///   - `polarity`: false se "non" precede il verbo nel raw_words.
///   - `relation`: dedotta dalla categoria del verbo + classe del predicato.
///
/// Restituisce None se nessun SpeakerClaim è stato rilevato e nessuna
/// proposizione di terza persona può essere ricostruita.
pub fn extract_proposition(
    raw_words: &[String],
    claim: Option<&SpeakerClaim>,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<SentenceProposition> {
    // ── 1. Polarità: cerchiamo "non" prima del primo verbo riconosciuto ────
    let polarity = !raw_words.iter().any(|w| w.eq_ignore_ascii_case("non"));

    // ── 2. Caso domanda: la frase contiene un pronome interrogativo ────────
    //    "chi sei?" → Variable("chi") + IsA + Entity (la persona è 2sg)
    //    "cosa pensi?" → Variable("cosa") + ... + Entity
    if let Some(interrog) = find_interrogative(raw_words) {
        let (subject, object) = build_question(&interrog, raw_words, kg_proc);
        return Some(SentenceProposition {
            subject,
            relation: RelationType::IsA, // default per domande aperte (chi/cosa/quale)
            object: Some(ObjectRef::Variable(interrog)),
            via: None,
            polarity,
            // (signature: tener conto del campo via in costruzioni del tipo
            //  "di cosa hai paura?" → la variable è nell'object, il via è
            //  "paura"; per ora teniamo semplice e lasciamo questo caso a
            //  iterazioni successive.)
        }).map(|mut p| { p.object = Some(ObjectRef::Variable(interrog_label(&p))); p });
    }

    // ── 3. Caso con SpeakerClaim (Phase 80): la frase ha un agente+verbo+predicato ─
    let claim = claim?;
    let subject = match claim.agent {
        ClaimAgent::Speaker => SubjectRef::Speaker,
        ClaimAgent::Entity  => SubjectRef::Entity,
    };
    let relation = relation_from_verb_category(claim);
    let object = Some(ObjectRef::Word(claim.predicate.clone()));
    // Phase 83: se il frame del verbo ha già separato un complemento (il tema
    // dativo, es. "madre" in "mi manca mia madre"), QUELLO è la via — non
    // serve ricavarlo dalle preposizioni. Altrimenti, frame nominativo: via
    // dalla preposizione di specificazione ("del futuro").
    let via = claim.complement.clone()
        .or_else(|| extract_via(raw_words, &claim.predicate, kg_proc));

    Some(SentenceProposition { subject, relation, object, via, polarity })
}

/// Cerca un pronome interrogativo nei token. Il pronome è interrogativo se
/// `IsA interrogativo` nel kg_proc; in alternativa per i casi base usiamo
/// la lista canonica (chi/cosa/che/dove/quando/perché/come/quale/quanto).
fn find_interrogative(raw_words: &[String]) -> Option<String> {
    // Interrogativi "forti": sempre interrogativi quando presenti.
    const STRONG: &[&str] = &[
        "chi", "cosa", "dove", "quando", "perché", "perche", "come", "quale", "quanto",
    ];
    if let Some(w) = raw_words.iter()
        .find(|w| STRONG.contains(&w.to_lowercase().as_str()))
    {
        return Some(w.clone());
    }
    // "che" è ambiguo: interrogativo ("Che fai?") vs relativo ("la rabbia
    // che non so spiegare"). Il "che" relativo non apre mai l'enunciato; lo
    // accettiamo come interrogativo solo in posizione iniziale. Così "provo
    // una rabbia che non so spiegare" non viene più letto come una domanda
    // fantasma su "che" (Phase 83 — fix corruzione diffusa).
    if raw_words.first().map(|w| w.eq_ignore_ascii_case("che")).unwrap_or(false) {
        return Some(raw_words[0].clone());
    }
    None
}

/// Per una domanda, deduce subject e object dal contesto:
///   "chi sei?" → subject=Entity (2sg di "sei"), object=Variable("chi")
///   "cosa pensi?" → subject=Entity, object=Variable("cosa")
///   "chi è marco?" → subject=World("marco"), object=Variable("chi")
fn build_question(
    _interrog: &str,
    raw_words: &[String],
    _kg_proc: Option<&KnowledgeGraph>,
) -> (SubjectRef, Option<ObjectRef>) {
    use crate::topology::grammar::{lemmatize, Person};
    // Se l'enunciato contiene un verbo coniugato in 2a singolare, la domanda è rivolta a UI-r1.
    let has_2sg = raw_words.iter().any(|w| {
        lemmatize(&w.to_lowercase())
            .map(|r| matches!(r.person, Person::Second))
            .unwrap_or(false)
    });
    if has_2sg {
        (SubjectRef::Entity, None)
    } else {
        // Default: subject World (anonimo) — il rendering downstream lo
        // gestirà come "qualcosa nel mondo". Iterazioni successive potranno
        // ancorarlo a un sostantivo concreto della frase.
        (SubjectRef::World(String::new()), None)
    }
}

fn interrog_label(p: &SentenceProposition) -> String {
    match &p.object {
        Some(ObjectRef::Variable(w)) => w.clone(),
        _ => String::new(),
    }
}

/// Estrae la `via`: la prima parola-contenuto che segue una preposizione
/// di specificazione, ESCLUDENDO il predicato già preso. Una preposizione è
/// di specificazione se `IsA preposizione + IsA specificazione` nel kg_proc,
/// oppure (fallback) appartiene al canon ("di", "del", "della", "dei", "degli",
/// "delle", "per", "su", "dell'").
///
/// Esempio: "ho paura del futuro" con predicate="paura":
///   loop sui token: trova "del" come preposizione → la parola successiva
///   "futuro" è la via.
fn extract_via(
    raw_words: &[String],
    predicate: &str,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<String> {
    let preds_lc = predicate.to_lowercase();
    let mut iter = raw_words.iter().enumerate().peekable();
    while let Some((i, w)) = iter.next() {
        let w_lc = w.to_lowercase();
        if !is_specification_preposition(&w_lc, kg_proc) { continue; }
        // Prendi la parola successiva non-funzionale e diversa dal predicato.
        for j in (i + 1)..raw_words.len() {
            let cand = raw_words[j].to_lowercase();
            if cand == preds_lc { continue; }
            if is_function_word_simple(&cand, kg_proc) { continue; }
            return Some(raw_words[j].clone());
        }
    }
    None
}

/// Una preposizione è "di specificazione" se il kg_proc la classifica come
/// tale. Fallback canon per le preposizioni più comuni (di / del / della /
/// dei / degli / delle / per / con / su / dell').
fn is_specification_preposition(w: &str, kg_proc: Option<&KnowledgeGraph>) -> bool {
    const CANON: &[&str] = &[
        "di", "del", "della", "dei", "degli", "delle", "dello",
        "dal", "dalla", "dai", "dagli", "dalle",
        "per", "con", "su", "sul", "sulla",
        "dell'", "all'", "nell'",
    ];
    if CANON.contains(&w) { return true; }
    let Some(kp) = kg_proc else { return false; };
    let parents = kp.query_objects(w, RelationType::IsA);
    let has_prep = parents.iter().any(|p| p.eq_ignore_ascii_case("preposizione"));
    let has_spec = parents.iter().any(|p| p.eq_ignore_ascii_case("specificazione"));
    has_prep && has_spec
}

fn is_function_word_simple(w: &str, kg_proc: Option<&KnowledgeGraph>) -> bool {
    const CANON: &[&str] = &[
        "il", "lo", "la", "i", "gli", "le", "l'",
        "un", "una", "uno", "un'",
        "che", "in", "tra", "fra",
        "e", "o", "ma",
    ];
    if CANON.contains(&w) { return true; }
    let Some(kp) = kg_proc else { return false; };
    let parents = kp.query_objects(w, RelationType::IsA);
    parents.iter().any(|p| {
        matches!(p.to_lowercase().as_str(),
            "pronome" | "articolo" | "preposizione" |
            "marcatore" | "congiunzione" | "copula")
    })
}

/// Mappa categoria del verbo + classe del predicato → RelationType.
///
/// Non è un dispatch comportamentale: è la *traduzione* della categoria
/// grammaticale (che vive nel kg_proc come dato) verso la rete di relazioni
/// semantiche (che vive nel kg_sem come dato). Aggiungere una categoria al
/// kg_proc richiederà l'aggiornamento di un caso qui, ma è uno-a-uno: nessuna
/// regola di transizione comportamentale.
fn relation_from_verb_category(claim: &SpeakerClaim) -> RelationType {
    match (claim.verb_category.as_deref(), &claim.kind) {
        // "mi chiamo X" — denominazione: il nome è un'etichetta del Sé.
        // Trattata come IsA per ora (Lacanianamente: il nome È il Sé in superficie).
        (Some("denominativo"), _) => RelationType::IsA,
        // "sento/provo X" — percezione di stato interno → FeelsAs.
        (Some("percettivo"), _)   => RelationType::FeelsAs,
        // "ho fame" / "sono triste" — copula + stato interno → FeelsAs.
        // "io sono Marco" — copula + non-inner → IsA.
        (Some("copula"), ClaimKind::Feeling)  => RelationType::FeelsAs,
        (Some("copula"), _)                   => RelationType::IsA,
        // "penso/credo/voglio X" — espressione cognitiva.
        (Some("cognitivo"), _)    => RelationType::Expresses,
        // "dico/chiedo/parlo X" — espressione comunicativa.
        (Some("comunicativo"), _) => RelationType::Expresses,
        // "vado/cerco/lavoro X" — azione.
        (Some("azione"), _)       => RelationType::Does,
        // Fallback prudente: senza categoria, IsA per Identity/Feeling, Does per Action.
        (_, ClaimKind::Action)    => RelationType::Does,
        (_, ClaimKind::Feeling)   => RelationType::FeelsAs,
        (_, ClaimKind::Identity)  => RelationType::IsA,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Confronto col KG semantico
// ═══════════════════════════════════════════════════════════════════════════

/// Confronta la proposizione col kg_sem.
///
/// Phase 81 (minimo viable): controlla se l'oggetto/via sono ancorati al
/// kg_sem (hanno almeno una relazione IsA, Has, o Causes); rileva
/// contraddizioni leggere via OppositeOf su via↔object.
///
/// Estensioni future:
///  - match diretto della triple (X relation Y) nel KG → "eco";
///  - inferenze 2-hop coerenti con (subject, relation, object);
///  - integrazione con syllogisms di comprehension_graph.
pub fn confront_with_kg(
    prop: &SentenceProposition,
    kg_sem: &KnowledgeGraph,
) -> KgConfrontation {
    let mut out = KgConfrontation::default();

    let object_word = match &prop.object {
        Some(ObjectRef::Word(w)) => Some(w.clone()),
        _ => None,
    };

    if let Some(obj) = &object_word {
        let obj_lc = obj.to_lowercase();
        out.object_in_kg = node_has_any_kg_edge(kg_sem, &obj_lc);
    }
    if let Some(via) = &prop.via {
        let via_lc = via.to_lowercase();
        out.via_in_kg = node_has_any_kg_edge(kg_sem, &via_lc);

        // Contraddizione strutturale leggera: l'oggetto e il via sono in
        // OppositeOf reciproco? Es. "ho coraggio del pericolo" (?). È raro
        // ma se accade, vale la pena marcarlo.
        if let Some(obj) = &object_word {
            for opposite in kg_sem.query_objects(&obj.to_lowercase(), RelationType::OppositeOf) {
                if opposite.eq_ignore_ascii_case(&via_lc) {
                    out.contradictions.push((obj.clone(), via.clone()));
                }
            }
        }
    }

    // Match esatto della triple: caso più semplice — Speaker/Entity non
    // sono nodi del kg_sem (sono ruoli enunciativi), quindi il check
    // significativo è quando subject è World(s).
    if let SubjectRef::World(s) = &prop.subject {
        if let Some(ObjectRef::Word(o)) = &prop.object {
            let s_lc = s.to_lowercase();
            let o_lc = o.to_lowercase();
            out.matches = kg_sem.query_objects(&s_lc, prop.relation)
                .iter().any(|t| t.eq_ignore_ascii_case(&o_lc));
        }
    }

    out
}

fn node_has_any_kg_edge(kg: &KnowledgeGraph, word: &str) -> bool {
    use RelationType as R;
    for r in [R::IsA, R::Has, R::Causes, R::SimilarTo, R::OppositeOf, R::PartOf, R::Does, R::UsedFor] {
        if !kg.query_objects(word, r).is_empty() {
            return true;
        }
    }
    false
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::input_reading::{ClaimAgent, ClaimKind, SpeakerClaim};

    fn words(s: &str) -> Vec<String> {
        s.split_whitespace().map(|w| w.to_lowercase()).collect()
    }

    fn claim(agent: ClaimAgent, kind: ClaimKind, pred: &str, vc: Option<&str>) -> SpeakerClaim {
        SpeakerClaim {
            agent, kind,
            predicate: pred.to_string(),
            verb_category: vc.map(|s| s.to_string()),
            complement: None,
        }
    }

    fn kg_proc_minimal() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        // Preposizioni di specificazione
        for prep in ["di", "del", "della", "dei", "delle", "per"] {
            kg.add(prep, RelationType::IsA, "preposizione");
            kg.add(prep, RelationType::IsA, "specificazione");
        }
        kg.add("il", RelationType::IsA, "articolo");
        kg.add("mi", RelationType::IsA, "pronome");
        kg.add("io", RelationType::IsA, "pronome");
        kg
    }

    #[test]
    fn io_sono_triste_yields_speaker_feelsas_triste() {
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Feeling, "triste", Some("copula"));
        let p = extract_proposition(&words("io sono triste"), Some(&c), Some(&kg_proc))
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::FeelsAs);
        assert_eq!(p.object, Some(ObjectRef::Word("triste".to_string())));
        assert_eq!(p.via, None);
        assert!(p.polarity);
    }

    #[test]
    fn ho_paura_del_futuro_yields_via_futuro() {
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Feeling, "paura", Some("copula"));
        let p = extract_proposition(&words("ho paura del futuro"), Some(&c), Some(&kg_proc))
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::FeelsAs);
        assert_eq!(p.object, Some(ObjectRef::Word("paura".to_string())));
        assert_eq!(p.via, Some("futuro".to_string()),
            "via doveva essere `futuro` (oggetto della preposizione `del`)");
    }

    #[test]
    fn mi_chiamo_francesco_yields_isa_francesco() {
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Identity, "francesco", Some("denominativo"));
        let p = extract_proposition(&words("mi chiamo francesco"), Some(&c), Some(&kg_proc))
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::IsA);
        assert_eq!(p.object, Some(ObjectRef::Word("francesco".to_string())));
    }

    #[test]
    fn vado_al_mare_yields_does_with_predicate_mare() {
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Action, "mare", Some("azione"));
        let p = extract_proposition(&words("vado al mare"), Some(&c), Some(&kg_proc))
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::Does);
        // `mare` resta come object (predicato del claim); `al` non è una
        // preposizione di specificazione → via=None.
        assert_eq!(p.object, Some(ObjectRef::Word("mare".to_string())));
        assert_eq!(p.via, None);
    }

    #[test]
    fn chi_sei_yields_variable_object_with_entity_subject() {
        let kg_proc = kg_proc_minimal();
        // Nessun SpeakerClaim per le domande senza claim esplicito.
        let p = extract_proposition(&words("chi sei"), None, Some(&kg_proc))
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Entity,
            "domanda con verbo 2sg → soggetto = UI-r1 (Entity)");
        assert!(matches!(p.object, Some(ObjectRef::Variable(ref v)) if v == "chi"));
    }

    #[test]
    fn non_ho_paura_yields_polarity_false() {
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Feeling, "paura", Some("copula"));
        let p = extract_proposition(&words("non ho paura"), Some(&c), Some(&kg_proc))
            .expect("proposizione attesa");
        assert!(!p.polarity, "polarità doveva essere false in presenza di `non`");
    }

    #[test]
    fn no_claim_no_interrog_yields_none() {
        let kg_proc = kg_proc_minimal();
        let p = extract_proposition(&words("il sole splende"), None, Some(&kg_proc));
        assert!(p.is_none(),
            "senza claim e senza pronome interrogativo, niente proposizione (per ora)");
    }

    // ─── Confronto con kg_sem ─────────────────────────────────────────────

    fn kg_sem_minimal() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        kg.add("paura", RelationType::IsA, "emozione");
        kg.add("paura", RelationType::Causes, "tremore");
        kg.add("futuro", RelationType::IsA, "tempo");
        kg.add("triste", RelationType::OppositeOf, "felice");
        kg
    }

    #[test]
    fn confronto_paura_futuro_anchora_entrambi_al_kg() {
        let kg_sem = kg_sem_minimal();
        let prop = SentenceProposition {
            subject: SubjectRef::Speaker,
            relation: RelationType::FeelsAs,
            object: Some(ObjectRef::Word("paura".to_string())),
            via: Some("futuro".to_string()),
            polarity: true,
        };
        let conf = confront_with_kg(&prop, &kg_sem);
        assert!(conf.object_in_kg, "paura ha archi nel kg_sem");
        assert!(conf.via_in_kg,    "futuro ha archi nel kg_sem");
        assert!(conf.contradictions.is_empty());
    }

    #[test]
    fn confronto_oggetto_sconosciuto_non_ancorato() {
        let kg_sem = kg_sem_minimal();
        let prop = SentenceProposition {
            subject: SubjectRef::Speaker,
            relation: RelationType::IsA,
            object: Some(ObjectRef::Word("xyzzy".to_string())),
            via: None,
            polarity: true,
        };
        let conf = confront_with_kg(&prop, &kg_sem);
        assert!(!conf.object_in_kg);
        assert!(!conf.via_in_kg);
    }

    // ─── Phase 83: "che" relativo non dirotta il claim ────────────────────

    #[test]
    fn relative_che_does_not_hijack_claim() {
        // "provo una rabbia che non so spiegare": il "che" è relativo, non
        // interrogativo. Prima di Phase 83 veniva letto come domanda fantasma
        // (World IsA 'che'); ora la frase deve leggere il claim percettivo.
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Feeling, "rabbia", Some("percettivo"));
        let p = extract_proposition(
            &words("provo una rabbia che non so spiegare"),
            Some(&c), Some(&kg_proc),
        ).expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::FeelsAs);
        assert_eq!(p.object, Some(ObjectRef::Word("rabbia".to_string())));
    }

    #[test]
    fn che_iniziale_resta_interrogativo() {
        // "che fai?" — "che" in posizione iniziale è interrogativo.
        let kg_proc = kg_proc_minimal();
        let p = extract_proposition(&words("che fai"), None, Some(&kg_proc))
            .expect("proposizione attesa");
        assert!(matches!(p.object, Some(ObjectRef::Variable(ref v)) if v == "che"));
    }

    #[test]
    fn che_cosa_preferisce_cosa() {
        // "che cosa rende vivo un pensiero?" — l'interrogativo forte "cosa"
        // vince sul "che", che da solo non è più catturato.
        let kg_proc = kg_proc_minimal();
        let p = extract_proposition(&words("che cosa rende vivo un pensiero"), None, Some(&kg_proc))
            .expect("proposizione attesa");
        assert!(matches!(p.object, Some(ObjectRef::Variable(ref v)) if v == "cosa"));
    }

    #[test]
    fn dative_complement_becomes_via() {
        // Frame dativo: il claim porta predicate=emozione, complement=tema.
        // La proposizione mette il tema nello slot `via` — forma a specchio
        // di "ho paura del futuro" → Speaker FeelsAs mancanza via=madre.
        let kg_proc = kg_proc_minimal();
        let mut c = claim(ClaimAgent::Speaker, ClaimKind::Feeling, "mancanza", Some("percettivo"));
        c.complement = Some("madre".to_string());
        let p = extract_proposition(&words("mi manca mia madre"), Some(&c), Some(&kg_proc))
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::FeelsAs);
        assert_eq!(p.object, Some(ObjectRef::Word("mancanza".to_string())));
        assert_eq!(p.via, Some("madre".to_string()),
            "il tema dativo deve riempire lo slot via");
    }
}
