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

/// Un complemento della frase: una preposizione + il suo nome, con la relazione
/// kg_sem disambiguata (Phase 86 Stadio 2). "ho paura **del futuro**" →
/// Complement{prep:"del", noun:"futuro", relation: <disambiguata dal KG>}.
/// È l'analisi logica *completa*: oltre il singolo `via`, ogni sintagma
/// preposizionale diventa un legame tipato.
#[derive(Debug, Clone, PartialEq)]
pub struct Complement {
    pub preposition: String,
    pub noun: String,
    /// Relazione disambiguata contro il KG (None se la preposizione non ha
    /// ipotesi-contenuto, es. "a" dativo).
    pub relation: Option<RelationType>,
    /// Ruolo logico del complemento (paragone/termine/specificazione/fine/…),
    /// dalla preposizione + categoria del verbo. È il "secondo argomento"
    /// dell'analisi logica (Stadio 2): "preferisco X **a Y**" → Y = paragone.
    pub role: Option<String>,
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
    /// Lemma del verbo di SUPERFICIE che ha realizzato la relazione (es.
    /// "uccidere" in "il tradimento uccide la fiducia", "iniziare" in "il giorno
    /// inizia"). La `relation` porta il TIPO (Does/IsA/FeelsAs…); questo porta il
    /// TOKEN concreto, così la comprensione non perde *quale* azione è e
    /// l'espressione può realizzarla invece del generico "compie". `None` quando
    /// non c'è un verbo lessicale di contenuto (copula "è", o frame senza verbo).
    /// Letto dalla stessa lemmatizzazione che categorizza il verbo — generale, mai
    /// per-verbo.
    pub verb_lemma: Option<String>,
    /// Polarità della proposizione (false se l'utterance contiene una negazione
    /// che precede il verbo). Per ora rilevata via parola letterale "non".
    pub polarity: bool,
    /// Tutti i complementi preposizionali (Phase 86 Stadio 2), ciascuno con la
    /// relazione disambiguata. `via` resta il complemento di specificazione
    /// primario (compat); `complements` è l'analisi logica completa.
    pub complements: Vec<Complement>,
    /// Il soggetto di superficie quando è CELATO (pro-drop italiano):
    /// "vogliamo"→"noi", "devo"→"io". Recuperato dalla desinenza del verbo.
    /// `None` per soggetti del mondo (in `subject`) o domande.
    pub subject_surface: Option<String>,
}

/// Una proposizione ancorata alla sua CLAUSOLA (multi-locus, Phase 86+).
/// Un enunciato è un `Vec<ClauseProposition>`; la primaria (prima indipendente)
/// è quella da cui la voce a turno singolo risponde, il numero di loci
/// indipendenti con proposizione alimenta il bisogno `Strutturare` (i dump).
#[derive(Debug, Clone)]
pub struct ClauseProposition {
    /// La proposizione della clausola. `None` quando la clausola non porta una
    /// proposizione estraibile (onesto: meglio nessuna che una sbagliata —
    /// la subordinata NON deve diventare primaria al posto della principale).
    pub prop: Option<SentenceProposition>,
    /// Intervallo di token della clausola nell'enunciato.
    pub range: std::ops::Range<usize>,
    /// La clausola è subordinata (mai candidabile a primaria).
    pub subordinate: bool,
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
/// Riduzione nominale validata di un token in ruolo ARGOMENTO (oggetto/via): al
/// singolare se il KG lo conferma, altrimenti superficie (deferisce sull'ambiguo).
/// Senza KG, identità. È la lemmatizzazione per ruolo applicata agli argomenti.
fn reduce_nominal(w: &str, kg: Option<&KnowledgeGraph>) -> String {
    reduce_nominal_gendered(w, kg, None)
}

/// Come `reduce_nominal` ma col genere noto dall'articolo (accordo grammaticale):
/// scioglie la falsa-ambiguità dei plurali ("i gatti"→"gatto", non deferito).
fn reduce_nominal_gendered(w: &str, kg: Option<&KnowledgeGraph>, masc: Option<bool>) -> String {
    match kg {
        Some(k) => crate::topology::grammar::kg_validated_nominal_gendered(w, masc, |c| k.contains(c)),
        None => w.to_string(),
    }
}

/// Genere del nome `target` dedotto dall'articolo che lo precede in `raw_words`
/// (accordo grammaticale, non morfologia indovinata). i/gli/il/lo/un/uno →
/// maschile; le/la/una → femminile; `l'`/articolo assente/altro → None (ambiguo:
/// non si indovina, [[feedback-no-tricks-toward-reality]]). Usato per ridurre i
/// plurali argomento al singolare giusto ("le piante"→pianta, "i gatti"→gatto).
fn article_gender(raw_words: &[String], target: &str) -> Option<bool> {
    let t = target.to_lowercase();
    let idx = raw_words.iter().position(|w| w.to_lowercase() == t)?;
    if idx == 0 {
        return None;
    }
    match raw_words[idx - 1].to_lowercase().as_str() {
        "i" | "gli" | "il" | "lo" | "un" | "uno" => Some(true),
        "le" | "la" | "una" => Some(false),
        _ => None,
    }
}

pub fn extract_proposition(
    raw_words: &[String],
    claim: Option<&SpeakerClaim>,
    kg_proc: Option<&KnowledgeGraph>,
    kg: Option<&KnowledgeGraph>,
) -> Option<SentenceProposition> {
    // ── 1. Polarità: cerchiamo "non" prima del primo verbo riconosciuto ────
    let polarity = !raw_words.iter().any(|w| w.eq_ignore_ascii_case("non"));

    // ── 2. Caso domanda: la frase contiene un pronome interrogativo ────────
    //    "chi sei?" → Variable("chi") + IsA + Entity (la persona è 2sg)
    //    "cosa pensi?" → Variable("cosa") + ... + Entity
    if let Some(interrog) = find_interrogative(raw_words) {
        let (subject, object) = build_question(&interrog, raw_words, kg_proc);
        let _ = object;
        return Some(SentenceProposition {
            subject,
            relation: RelationType::IsA, // default per domande aperte (chi/cosa/quale)
            object: Some(ObjectRef::Variable(interrog)),
            via: None,
            verb_lemma: None, // domanda aperta: il verbo è copula/da-riempire
            polarity,
            complements: vec![],
            subject_surface: None,
        }).map(|mut p| {
            p.object = Some(ObjectRef::Variable(interrog_label(&p)));
            p.complements = extract_complements(raw_words, kg_proc, kg, &p);
            p
        });
    }

    // ── 3.bis Caso soggetto-Mondo (Phase 84, 2c-D): nessun claim Speaker/
    //    Entity, ma una frase in 3a persona sul mondo ("il mare è profondo").
    //    → World(soggetto) IsA oggetto. Solo copula per ora; i verbi d'azione
    //    in 3a persona ("il sole splende") richiederanno il kg_sem qui.
    if claim.is_none() {
        if let Some(mut p) = extract_world_proposition(raw_words, kg_proc, kg, polarity) {
            p.complements = extract_complements(raw_words, kg_proc, kg, &p);
            return Some(p);
        }
    }

    // ── 3. Caso con SpeakerClaim (Phase 80): la frase ha un agente+verbo+predicato ─
    let claim = claim?;
    let subject = match claim.agent {
        ClaimAgent::Speaker => SubjectRef::Speaker,
        ClaimAgent::Entity  => SubjectRef::Entity,
    };
    let relation = relation_from_verb_category(claim);
    // Phase 86+: lemmatizzazione PER RUOLO. L'oggetto e la via sono ARGOMENTI
    // (nomi) → riduzione nominale validata ("biciclette"→"bicicletta"), mai un
    // infinito (il ruolo ha disambiguato nome-vs-verbo). Deferisce sull'ambiguo.
    let object = Some(ObjectRef::Word(reduce_nominal_gendered(
        &claim.predicate, kg, article_gender(raw_words, &claim.predicate))));
    // Phase 83: se il frame del verbo ha già separato un complemento (il tema
    // dativo, es. "madre" in "mi manca mia madre"), QUELLO è la via — non
    // serve ricavarlo dalle preposizioni. Altrimenti, frame nominativo: via
    // dalla preposizione di specificazione ("del futuro").
    let via = claim.complement.clone()
        .or_else(|| extract_via(raw_words, &claim.predicate, kg_proc))
        .map(|v| reduce_nominal_gendered(&v, kg, article_gender(raw_words, &v)));

    let mut prop = SentenceProposition {
        subject, relation, object, via,
        verb_lemma: claim.verb_lemma.clone(),
        polarity, complements: vec![],
        subject_surface: claim.subject_surface.clone(),
    };
    prop.complements = extract_complements(raw_words, kg_proc, kg, &prop);
    Some(prop)
}

/// Multi-locus (Phase 86+): una proposizione per CLAUSOLA dell'enunciato.
///
/// Usa il chunker clausa-aware (`analisi_logica::analizza`) per segmentare, poi
/// estrae la proposizione di ogni clausola sul suo *slice* di token, con il
/// claim rilevato PER CLAUSOLA. Risolve la frase multi-clausola: "mi sento solo
/// da quando me ne sono andato" → la principale "mi sento solo" non viene più
/// scavalcata dalla subordinata. Le clausole senza proposizione restano nel Vec
/// (con `prop=None`) così la selezione della primaria resta strutturale.
///
/// Senza entrambi i grafi (path di test) ricade su una proposizione singola
/// sull'intero enunciato — backward-compatible con `extract_proposition`.
pub fn extract_propositions(
    raw_words: &[String],
    lexicon: &crate::topology::lexicon::Lexicon,
    kg_proc: Option<&KnowledgeGraph>,
    kg: Option<&KnowledgeGraph>,
) -> Vec<ClauseProposition> {
    let (Some(kp), Some(k)) = (kg_proc, kg) else {
        let claim = crate::topology::input_reading::detect_speaker_claim(
            raw_words, lexicon, kg, kg_proc,
        );
        let prop = extract_proposition(raw_words, claim.as_ref(), kg_proc, kg);
        return vec![ClauseProposition { prop, range: 0..raw_words.len(), subordinate: false }];
    };
    let analisi = crate::topology::analisi_logica::analizza(raw_words, kp, k);
    let mut out = Vec::with_capacity(analisi.clausole.len());
    for c in &analisi.clausole {
        let mut slice = &raw_words[c.range.clone()];
        // "che" completivo/relativo che APRE una subordinata è un complementatore
        // trasparente, non un interrogativo: "ha spiegato CHE il lancio dipende…"
        // → estrai sul contenuto ("il lancio dipende…" → lancio Requires
        // completamento), non su un "?che" fantasma. Vale solo per le subordinate
        // (al top-level "Che fai?" il "che" resta interrogativo, Phase 83).
        if c.subordinate && slice.first().map_or(false, |w| w.eq_ignore_ascii_case("che")) {
            slice = &slice[1..];
        }
        if slice.is_empty() { continue; }
        let claim = crate::topology::input_reading::detect_speaker_claim(
            slice, lexicon, kg, kg_proc,
        );
        let prop = extract_proposition(slice, claim.as_ref(), kg_proc, kg);
        out.push(ClauseProposition { prop, range: c.range.clone(), subordinate: c.subordinate });
    }

    // Phase 86+: ELLISSI DEL SOGGETTO nelle coordinate. In un dump "devo chiamare
    // il dottore E prendere le medicine" la 2ª clausola non ha soggetto esplicito
    // e il suo verbo è all'infinito → niente claim. Ma il soggetto è ELISO e
    // co-riferisce a quello della principale (Speaker/Entity). Lo rendiamo
    // esplicito: re-estraiamo la clausola coordinata con il pronome iniettato.
    // Solo coordinate (non subordinate), solo se la principale ha un agente.
    let inherited_pron: Option<&str> = primary_index(&out)
        .and_then(|i| out[i].prop.as_ref())
        .and_then(|p| match p.subject {
            SubjectRef::Speaker => Some("io"),
            SubjectRef::Entity => Some("tu"),
            _ => None,
        });
    if let Some(pron) = inherited_pron {
        let primary = primary_index(&out);
        for idx in 0..out.len() {
            if out[idx].prop.is_some() || out[idx].subordinate || Some(idx) == primary {
                continue;
            }
            let mut injected: Vec<String> = Vec::with_capacity(out[idx].range.len() + 1);
            injected.push(pron.to_string());
            injected.extend_from_slice(&raw_words[out[idx].range.clone()]);
            let claim = crate::topology::input_reading::detect_speaker_claim(
                &injected, lexicon, kg, kg_proc,
            );
            if let Some(p) = extract_proposition(&injected, claim.as_ref(), kg_proc, kg) {
                out[idx].prop = Some(p);
            }
        }
    }
    out
}

/// L'indice della proposizione PRIMARIA: la prima clausola INDIPENDENTE; in
/// mancanza, la prima clausola. La voce a turno singolo risponde da qui (le
/// subordinate sono circostanza, non il cuore dell'enunciato).
pub fn primary_index(props: &[ClauseProposition]) -> Option<usize> {
    if props.is_empty() { return None; }
    Some(props.iter().position(|c| !c.subordinate).unwrap_or(0))
}

/// Quanti loci INDIPENDENTI con una proposizione reale (≥2 ⇒ dump → bisogno
/// `Strutturare`). Le subordinate (circostanza di un'unica idea) non contano.
pub fn independent_locus_count(props: &[ClauseProposition]) -> usize {
    props.iter().filter(|c| !c.subordinate && c.prop.is_some()).count()
}

/// Phase 86 Stadio 2: estrae TUTTI i complementi preposizionali della frase e
/// disambigua la relazione di ciascuno contro il KG. La "testa" a cui il
/// complemento si lega è l'oggetto-contenuto (il predicato) se presente,
/// altrimenti il soggetto-Mondo. Senza kg, nessun complemento (niente
/// disambiguazione affidabile).
fn extract_complements(
    raw_words: &[String],
    kg_proc: Option<&KnowledgeGraph>,
    kg: Option<&KnowledgeGraph>,
    prop: &SentenceProposition,
) -> Vec<Complement> {
    let Some(kg) = kg else { return vec![] };
    // testa: oggetto-parola, oppure soggetto-Mondo
    let head = match &prop.object {
        Some(ObjectRef::Word(w)) => Some(w.to_lowercase()),
        _ => None,
    }.or_else(|| match &prop.subject {
        SubjectRef::World(w) => Some(w.to_lowercase()),
        _ => None,
    });
    let Some(head) = head else { return vec![] };

    // Categoria del verbo reggente (per il ruolo: "a" dopo un valutativo =
    // paragone). Letta dalla stessa lemmatizzazione che categorizza il verbo.
    let vcat = prop.verb_lemma.as_deref()
        .and_then(|v| crate::topology::input_reading::verb_category(v, kg_proc));

    let mut out: Vec<Complement> = Vec::new();
    for (i, w) in raw_words.iter().enumerate() {
        let prep = w.to_lowercase();
        // È una preposizione? (di/da/per/con/su/in/contro/…/a) — "a" inclusa
        // (Stadio 2): introduce il secondo argomento anche senza relazione-
        // contenuto pulita ("preferisco X **a** Y", "giocare **a** calcio").
        if !crate::topology::prepositions::is_preposition(&prep) {
            continue;
        }
        // Prima parola-contenuto dopo la preposizione.
        let Some(noun) = raw_words[i + 1..].iter()
            .find(|x| !is_function_word_simple(&x.to_lowercase(), kg_proc))
            .map(|x| x.to_lowercase())
        else { continue };
        if noun == head || noun == prep { continue; }
        // Evita duplicati sullo stesso nome.
        if out.iter().any(|c| c.noun == noun) { continue; }
        // Relazione disambiguata dal KG (None per "a" e per le prep senza
        // ipotesi-contenuto); ruolo logico dalla preposizione + categoria verbo.
        let relation = crate::topology::prepositions::disambiguate(&head, &prep, &noun, kg)
            .map(|r| r.relation);
        let role = crate::topology::prepositions::complement_role(&prep, vcat)
            .map(|s| s.to_string());
        out.push(Complement { preposition: prep, noun, relation, role });
    }
    out
}

/// Cerca un pronome interrogativo nei token. Il pronome è interrogativo se
/// `IsA interrogativo` nel kg_proc; in alternativa per i casi base usiamo
/// la lista canonica (chi/cosa/che/dove/quando/perché/come/quale/quanto).
pub(crate) fn find_interrogative(raw_words: &[String]) -> Option<String> {
    // Interrogativi "forti": interrogativi quando presenti — MA non quando
    // introdotti da una preposizione, che li rende subordinatori (temporali/
    // modali): "da quando", "di come", "per quanto". Phase 86 (A2, bench
    // 2026-06-08): senza, "mi sento solo da quando me ne sono andato" veniva
    // letto come domanda fantasma su "quando", perdendo `Speaker FeelsAs solo`.
    const STRONG: &[&str] = &[
        "chi", "cosa", "dove", "quando", "perché", "perche", "come", "quale", "qual", "quanto",
    ];
    const SUBORDINATING_PREP: &[&str] = &["da", "di", "per", "in", "con", "su", "a", "fin", "fino"];
    // Un interrogativo preceduto da articolo/determinante è in realtà la TESTA
    // di un sintagma nominale ("una cosa", "la cosa", "quella cosa") — un nome,
    // non una domanda. Stesso chiuso-classe di SUBORDINATING_PREP: cattura anche
    // "la quale" (pronome relativo, non interrogativo). Senza, "mi aiuti a
    // ragionare su una cosa?" veniva letto come domanda fantasma "che è UI-r1".
    const ARTICLE_DET: &[&str] = &[
        "il", "lo", "la", "i", "gli", "le", "un", "uno", "una",
        "questo", "questa", "questi", "queste", "quello", "quella", "quei", "quegli", "quelle",
        "codesto", "ogni", "alcun", "alcuna", "qualche", "nessun", "nessuna",
    ];
    if let Some((_, w)) = raw_words.iter().enumerate()
        .filter(|(_, w)| STRONG.contains(&w.to_lowercase().as_str()))
        .find(|(i, _)| {
            if *i == 0 { return true; }
            let prev = raw_words[i - 1].to_lowercase();
            let prev = prev.trim_end_matches('\'');
            !SUBORDINATING_PREP.contains(&prev) && !ARTICLE_DET.contains(&prev)
        })
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

/// Phase 84 (2c-D): costruisce una proposizione sul Mondo da una frase in 3a
/// persona con copula ("il mare è profondo" → `World(mare) IsA profondo`).
/// La copula è individuata leggendo il kg_proc (`<lemma> IsA copula`); il
/// soggetto è l'ultima parola-contenuto prima della copula, l'oggetto la prima
/// dopo. Niente liste di verbi: copula = dato del kg_proc, soggetto/oggetto =
/// prime parole non-funzionali. I verbi d'azione 3a persona ("il sole splende")
/// non sono coperti qui (servirebbe il kg_sem per la verbità) — `None`.
fn extract_world_proposition(
    raw_words: &[String],
    kg_proc: Option<&KnowledgeGraph>,
    kg: Option<&KnowledgeGraph>,
    polarity: bool,
) -> Option<SentenceProposition> {
    use crate::topology::grammar::{lemmatize, Person};
    let kp = kg_proc?;
    let is_copula = |w: &str| -> bool {
        let lw = w.to_lowercase();
        let lemma = lemmatize(&lw).map(|r| r.infinitive).unwrap_or_else(|| lw.clone());
        kp.query_objects(&lemma, RelationType::IsA)
            .iter().any(|p| p.eq_ignore_ascii_case("copula"))
            || kp.query_objects(&lw, RelationType::IsA)
                .iter().any(|p| p.eq_ignore_ascii_case("copula"))
    };
    // Pivot della frase-mondo. PRIMO tentativo: la copula ("è" → IsA) oppure
    // "avere" ("ha"/"hanno" → Has). È la traduzione uno-a-uno categoria→relazione
    // applicata a un soggetto del Mondo: "il mare è profondo" → World(mare) IsA
    // profondo; "il silenzio ha un significato" → World(silenzio) Has significato.
    // "avere" prima della copula generica: nel kg_proc è `IsA copula`, ma con un
    // oggetto-contenuto sul Mondo significa possesso (Has), non identità.
    //
    // FALLBACK (Phase 86 #2): un verbo d'azione/transitivo in 3a persona
    // ("il tradimento uccide la fiducia"). NON è un claim Speaker/Entity (3a
    // persona → detect_speaker_claim ritorna None), ma è una proposizione sul
    // Mondo. Richiede il kg_sem per la verbità (`lemma_of_verb` usa la catena
    // IsA del kg_sem); la relazione viene dalla categoria del verbo (default
    // `Does` per un transitivo non categorizzato). Senza kg, niente fallback
    // → nessuna regressione rispetto a prima.
    // Cattura anche il LEMMA del verbo (non solo posizione+relazione): copula e
    // avere sono grammaticali → verb_lemma None; un verbo d'azione porta il suo
    // lemma di superficie ("uccidere", "iniziare") così non si perde *quale*
    // azione è. Generale: lo stesso `inf` che sceglie la relazione.
    // Pivot = (subject_end, obj_anchor, relation, verb_lemma). `subject_end` è il
    // confine sinistro della ricerca soggetto (il VERBO, o l'AUSILIARE nei composti);
    // `obj_anchor` è la testa verbale dopo cui cercare l'oggetto (il VERBO, o il
    // PARTICIPIO nei composti). Coincidono nei tempi semplici, divergono nei composti.
    let (verb_pos, obj_anchor, relation, verb_lemma) = raw_words.iter().enumerate().find_map(|(i, w)| {
        let lw = w.to_lowercase();
        let lemma = lemmatize(&lw).map(|r| r.infinitive).unwrap_or_else(|| lw.clone());
        // Tempo COMPOSTO sul Mondo (ausiliare + participio): "Marco ha aperto la
        // riunione" → World(marco) Does aprire (ogg=riunione), NON "marco Has
        // aperto". Speculare al path Speaker (`find_past_participle`). Il PARTICIPIO
        // decide: se manca — avere+nome ("ha un significato"), essere+aggettivo
        // ("è profondo") — restiamo su Has/IsA come prima. Il soggetto si cerca PRIMA
        // dell'ausiliare (subject_end=i), l'oggetto DOPO il participio (obj_anchor=pp).
        let composite = || crate::topology::input_reading::find_past_participle(raw_words, i + 1, kg_proc)
            .map(|(pp, inf)| {
                let rel = world_relation_for_verb(&inf, kg_proc);
                (i, pp, rel, Some(inf))
            });
        if lemma == "avere" || lw == "ha" || lw == "hanno" {
            return composite().or(Some((i, i, RelationType::Has, None::<String>)));
        }
        if is_copula(w) {
            return composite().or(Some((i, i, RelationType::IsA, None::<String>)));
        }
        None
    }).or_else(|| {
        // Un candidato-verbo preceduto da ARTICOLO è testa NOMINALE, non il verbo:
        // "il risultato" = nome (non "risultare"), "il mondo" ≠ "mondare". SOLO
        // l'articolo è introduttore-nominale non ambiguo: un quantificatore/
        // dimostrativo ("tutto", "questo") può essere pronome-soggetto ("tutto
        // dipende") oltre che determinante ("tutto il mondo") — quelli NO.
        // Il contesto disambigua l'omografo nome/participio: pondera, non indovina.
        let nominal_intro = |word: &str| {
            matches!(word, "il"|"lo"|"la"|"i"|"gli"|"le"|"un"|"uno"|"una"|"l'"|"un'")
            || kg_proc.map_or(false, |kp| kp.query_objects(word, RelationType::IsA)
                .iter().any(|p| p.eq_ignore_ascii_case("articolo")))
        };
        raw_words.iter().enumerate().find_map(|(i, w)| {
            // Testa nominale dopo articolo/determinante, o NOME retto da
            // PREPOSIZIONE → non è il verbo: "del prodotto" → prodotto è nome, non
            // produrre (in italiano la preposizione regge nome/infinito, mai un
            // verbo finito). Stesso principio del path Speaker.
            if i > 0 {
                let prev = raw_words[i-1].to_lowercase();
                if nominal_intro(&prev) || crate::topology::prepositions::is_preposition(&prev) {
                    return None;
                }
            }
            let (inf, person) = crate::topology::input_reading::lemma_of_verb(w, kg_proc, kg)?;
            // Nome-proprio/verbo (strutturale): un candidato DEBOLMENTE attestato
            // (non curato nel kg_proc) seguito da un verbo CURATO è il soggetto,
            // non il verbo ("Marco preferisce" → marco soggetto, preferire verbo).
            if crate::topology::input_reading::verb_category(&inf, kg_proc).is_none() {
                if let Some(next) = raw_words.get(i + 1) {
                    if let Some((ni, _)) = crate::topology::input_reading::lemma_of_verb(next, kg_proc, kg) {
                        if crate::topology::input_reading::verb_category(&ni, kg_proc).is_some() {
                            return None;
                        }
                    }
                }
            }
            // Solo 3a persona: 1a/2a sono claim Speaker/Entity, già gestiti a monte.
            if !matches!(person, Person::Third | Person::ThirdPlural) { return None; }
            // Modale + infinito sul Mondo: "Giulia deve preparare la documentazione"
            // → il CONTENUTO è l'infinito (preparare), non il modale. Speculare al
            // frame modale del path Speaker. Soggetto resta prima del modale (i),
            // oggetto dopo l'infinito (obj_anchor = ip).
            if kg_proc.map_or(false, |kp|
                crate::topology::input_reading::is_kg_proc_isa(kp, &inf, "modale"))
            {
                if let Some((ip, inf2)) = crate::topology::input_reading::infinitive_after(raw_words, i + 1, kg_proc, kg) {
                    let rel = world_relation_for_verb(&inf2, kg_proc);
                    return Some((i, ip, rel, Some(inf2)));
                }
            }
            let rel = world_relation_for_verb(&inf, kg_proc);
            Some((i, i, rel, Some(inf)))
        })
    })?;

    let pre = &raw_words[..verb_pos];
    let subject = pre.iter().enumerate().rev()
        // Testa del sintagma soggetto: l'ultimo NOME non retto da preposizione.
        // In "la qualità del prodotto", "prodotto" è retto da "del"
        // (specificazione che modifica "qualità") → la testa è "qualità". A
        // ritroso, salta i modificatori preceduti da preposizione.
        .find(|(j, w)| {
            !is_function_word_simple(&w.to_lowercase(), kg_proc)
                && (*j == 0 || !crate::topology::prepositions::is_preposition(&pre[j - 1].to_lowercase()))
        })
        .map(|(_, w)| w.to_lowercase())
        .or_else(|| {
            // Quantificatore / pronome indefinito SENZA un nome che determina
            // ("tutto dipende dal caso", "niente cambia") → uso PRONOMINALE, è il
            // soggetto. Se determinasse un nome ("tutto il mondo è bello") il nome
            // sarebbe già stato trovato sopra: questo è solo il fallback. La
            // struttura decide articolo-determinante vs pronome, non una lista.
            pre.iter().rev().find(|w| {
                kg_proc.map_or(false, |kp| {
                    kp.query_objects(&w.to_lowercase(), RelationType::IsA)
                        .iter().any(|p| matches!(p.to_lowercase().as_str(),
                            "quantificatore" | "indefinito"))
                })
            }).map(|w| w.to_lowercase())
        })
        .or_else(|| {
            // Soggetto ANAFORICO (Strato 3): nessun soggetto nominale esplicito, ma
            // il verbo è di 3a persona → l'agente vive altrove nel discorso. Marcato
            // qui, RISOLTO nello strato discorsivo (`build_analysis`), non indovinato:
            //  - pronome personale di 3a in posizione soggetto ("Lei teme…") →
            //    World(pronome): porta il pronome per il match al referente saliente;
            //  - pro-drop (solo parole-funzione prima del verbo, tipicamente vuoto —
            //    "Ha spiegato…") → World(""): elisione, riempita col soggetto saliente.
            // I pronomi di 3a sono classe CHIUSA (come io/tu altrove), non una lista
            // di nomi. Senza referente (frase isolata) il soggetto resta vuoto: onesto.
            const PRON3: &[&str] = &["lui","lei","loro","egli","ella","esso","essa","essi","esse"];
            if let Some(p) = pre.iter().rev().map(|w| w.to_lowercase())
                .find(|w| PRON3.contains(&w.as_str())) {
                return Some(p);
            }
            if pre.iter().all(|w| is_function_word_simple(&w.to_lowercase(), kg_proc)) {
                return Some(String::new()); // pro-drop
            }
            None
        })?;

    // Oggetto DIRETTO vs COMPLEMENTO. L'oggetto diretto si raggiunge dal verbo
    // SOLO attraverso articoli/determinanti. Una parola-contenuto introdotta da
    // una PREPOSIZIONE è un complemento (provenienza/luogo/oblige), non
    // l'oggetto: "il giorno inizia DAL mattino" → verbo intransitivo, mattino è
    // complemento — NON "giorno compie mattino". Direct objects in italiano non
    // prendono preposizione: la disambiguazione è strutturale.
    // `prepositions::is_preposition` normalizza anche le forme ARTICOLATE
    // (dall'/dal/della/nel/sul/…) che il check kg_proc `IsA preposizione` mancava
    // — perché il kg_proc elenca solo le forme semplici. Senza questo, "dall'" non
    // era riconosciuto e "ignoto" diventava un falso oggetto diretto. Fallback al
    // kg_proc per eventuali preposizioni fuori dal canon chiuso.
    let is_prep = |w: &str| crate::topology::prepositions::is_preposition(w)
        || crate::topology::input_reading::is_kg_proc_isa(kp, w, "preposizione");
    let mut direct: Option<String> = None;
    let mut complement: Option<String> = None;
    let mut saw_prep = false;
    for w in raw_words.iter().skip(obj_anchor + 1) {
        let lw = w.to_lowercase();
        if is_prep(&lw) { saw_prep = true; continue; }
        if is_function_word_simple(&lw, kg_proc) { continue; }
        if saw_prep { complement = Some(lw); } else { direct = Some(lw); }
        break;
    }

    // Frame di GENESI / DIPENDENZA (kg_proc, dato): quando l'unico argomento è un
    // complemento di preposizione, la relazione della proposizione viene dalla
    // PREPOSIZIONE (famiglia-ipotesi) raffinata dalla CLASSE del verbo, non dal
    // `Does` di default. "da" propone la famiglia causale; il frame sceglie:
    //   genesi     → Causes, soggetto = la FONTE ("la paura nasce dall'ignoto" →
    //                World(ignoto) Causes paura);
    //   dipendenza → Requires, soggetto invariato ("tutto dipende dal caso" →
    //                World(tutto) Requires caso).
    // Gated dal frame: un verbo senza frame lascia "da" come circostanza (via),
    // niente over-generalizzazione su "da" non-causali ("inizia dal mattino").
    if direct.is_none() {
        if let (Some(comp), Some(inf)) = (complement.as_ref(), verb_lemma.as_deref()) {
            let frames = kp.query_objects(inf, RelationType::IsA);
            let has = |f: &str| frames.iter().any(|p| p.eq_ignore_ascii_case(f));
            if has("genesi") {
                return Some(SentenceProposition {
                    subject: SubjectRef::World(comp.clone()),
                    relation: RelationType::Causes,
                    object: Some(ObjectRef::Word(subject.clone())),
                    via: None,
                    verb_lemma,
                    polarity,
                    complements: vec![],
                    subject_surface: None,
                });
            }
            if has("dipendenza") {
                return Some(SentenceProposition {
                    subject: SubjectRef::World(subject.clone()),
                    relation: RelationType::Requires,
                    object: Some(ObjectRef::Word(comp.clone())),
                    via: None,
                    verb_lemma,
                    polarity,
                    complements: vec![],
                    subject_surface: None,
                });
            }
        }
    }

    // Via: la specificazione dopo l'oggetto ("...uccide la fiducia PER
    // vigliaccheria" → via=vigliaccheria), o il complemento di provenienza/
    // luogo quando il verbo è intransitivo ("inizia dal mattino" → via=mattino).
    let via = direct.as_ref()
        .and_then(|o| extract_via(raw_words, o, kg_proc))
        .or_else(|| complement.clone());

    // Oggetto CLITICO proclitico: "mia moglie mi capisce" → il paziente è il
    // clitico (mi) PRIMA del verbo, non dopo. Un pronome immediatamente prima del
    // verbo (kg_proc `IsA pronome`/`IsA riflessivo`) è l'oggetto della transitiva.
    // Allora la proposizione è valida anche senza oggetto post-verbale: cattura
    // World(soggetto) Does <verbo> [polarità], il verbo non si perde più.
    let has_proclitic_object = verb_pos > 0 && {
        let prev = raw_words[verb_pos - 1].to_lowercase();
        crate::topology::input_reading::is_kg_proc_isa(kp, &prev, "pronome")
            || crate::topology::input_reading::is_kg_proc_isa(kp, &prev, "riflessivo")
    };

    // Conservativo: con un oggetto diretto, proposizione piena come prima. Con
    // SOLO un complemento, proposizione senza oggetto (intransitivo + via). Con un
    // oggetto clitico proclitico, proposizione transitiva (paziente cliticizzato).
    // Con nulla, nessuna proposizione (come prima — niente regressione).
    let object = match (&direct, &complement) {
        (Some(o), _) => Some(ObjectRef::Word(o.clone())),
        (None, Some(_)) => None,
        (None, None) if has_proclitic_object => None,
        (None, None) => return None,
    };

    Some(SentenceProposition {
        subject: SubjectRef::World(subject),
        relation,
        object,
        via,
        verb_lemma,
        polarity,
        complements: vec![],
        subject_surface: None,
    })
}

/// Mappa la categoria di un verbo (kg_proc) alla relazione kg_sem, per un
/// soggetto del Mondo. Speculare a `relation_from_verb_category` ma senza
/// `ClaimKind` (qui il soggetto è un nodo del mondo, non il parlante). Un verbo
/// transitivo non categorizzato → `Does` (compie un'azione sull'oggetto): la
/// struttura S-V-O è catturata; il pathfinding troverà comunque la relazione
/// reale fra i due nodi nel kg_sem.
fn world_relation_for_verb(inf: &str, kg_proc: Option<&KnowledgeGraph>) -> RelationType {
    match crate::topology::input_reading::verb_category(inf, kg_proc).as_deref() {
        Some("copula") => RelationType::IsA,
        Some("cognitivo") | Some("comunicativo") => RelationType::Expresses,
        Some("percettivo") => RelationType::FeelsAs,
        _ => RelationType::Does, // azione, denominativo, non categorizzato
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

/// Phase 86+ (analisi logica, strumento RESIDUO): i token dell'input che NON
/// risultano assegnati ad alcun ruolo dall'analisi corrente — la misura di
/// "nessuna parola saltata" (design `analisi_logica_grammatica_kg_proc.md`).
/// Un token è ASSEGNATO se: (a) è una parola-funzione (classe chiusa dal kg_proc),
/// (b) è una forma verbale riconosciuta (presente o participio — il predicato o
/// un verbo), (c) compare come contenuto della proposizione (soggetto-Mondo /
/// oggetto / via / complemento + la sua preposizione). Tutto il resto è RESIDUO:
/// oggi tipicamente attributi (aggettivi), complementi di tempo non-preposizionali,
/// marcatori di subordinata. **Sotto-stima i verbi** (accetta ogni forma verbale,
/// anche di subordinate) per non avere falsi positivi — è un floor onesto del gap.
pub fn unaccounted_tokens(
    raw_words: &[String],
    prop: Option<&SentenceProposition>,
    kg_proc: Option<&KnowledgeGraph>,
    kg: Option<&KnowledgeGraph>,
) -> Vec<String> {
    use std::collections::HashSet;
    let mut accounted: HashSet<String> = HashSet::new();
    if let Some(p) = prop {
        if let SubjectRef::World(w) = &p.subject {
            accounted.insert(w.to_lowercase());
        }
        if let Some(ObjectRef::Word(w)) = &p.object {
            accounted.insert(w.to_lowercase());
        }
        if let Some(v) = &p.via {
            accounted.insert(v.to_lowercase());
        }
        for c in &p.complements {
            accounted.insert(c.noun.to_lowercase());
            accounted.insert(c.preposition.to_lowercase());
        }
    }
    // Funzioni assegnate dal chunker (analisi logica): attributi + circostanze.
    let mut assegnati = std::collections::HashSet::new();
    if let (Some(kp), Some(k)) = (kg_proc, kg) {
        assegnati.extend(crate::topology::analisi_logica::attributo_indices(raw_words, kp, k));
    }
    if let Some(kp) = kg_proc {
        assegnati.extend(crate::topology::analisi_logica::circostanza_indices(raw_words, kp));
    }
    let attributi = assegnati;
    let mut out = Vec::new();
    for (i, w) in raw_words.iter().enumerate() {
        let lw = w.to_lowercase();
        if lw.is_empty() || !lw.chars().any(|c| c.is_alphabetic()) {
            continue;
        }
        if accounted.contains(&lw) {
            continue;
        }
        if attributi.contains(&i) {
            continue; // ruolo: attributo (gruppo nominale)
        }
        if is_function_word_simple(&lw, kg_proc) {
            continue;
        }
        if crate::topology::input_reading::lemma_of_verb(w, kg_proc, kg).is_some()
            || is_participle_form(&lw)
        {
            continue;
        }
        out.push(lw);
    }
    out
}

/// Forma di participio passato (suffisso regolare, stem ≥3). Usato solo dallo
/// strumento RESIDUO per non flaggare i verbi composti.
fn is_participle_form(w: &str) -> bool {
    const SUF: &[&str] = &[
        "ato", "ata", "ati", "ate", "uto", "uta", "uti", "ute", "ito", "ita", "iti", "ite",
    ];
    SUF.iter().any(|s| w.strip_suffix(s).map(|st| st.len() >= 3).unwrap_or(false))
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
            "marcatore" | "congiunzione" | "copula" |
            // Phase 86 (triage bench 2026-06-08): un AVVERBIO è un modificatore,
            // mai un soggetto/oggetto di contenuto. Senza questo, "forse è colpa
            // mia" → soggetto-Mondo="forse" (già `IsA avverbio` nel kg_proc).
            "avverbio" |
            // Phase 86 (A1, 2026-06-08): un DETERMINANTE (possessivo/dimostrativo/
            // quantificatore: mia/mio/questa/alcuna) non è mai testa di contenuto.
            // Allinea questa funzione a `is_kg_proc_function_word`: senza, "con
            // mia sorella" → complemento/via = "mia" invece di "sorella".
            "determinante" |
            // Un'interiezione (boh/mah) è atto espressivo a sé, non argomento.
            "interiezione")
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
        (Some("copula"), ClaimKind::Feeling)  => RelationType::FeelsAs,
        // copula + non-stato: ESSERE ≠ AVERE. "sono Marco" → identità (IsA);
        // "ho un cane" / "ho sonno" → possesso/stato posseduto (Has), MAI "io è
        // cane". La distinzione la porta il verbo di superficie (verb_lemma).
        (Some("copula"), _) if claim.verb_lemma.as_deref() == Some("avere")
                                              => RelationType::Has,
        (Some("copula"), _)                   => RelationType::IsA,
        // "amo/preferisco/desidero X" — attitudine valutativa verso l'oggetto:
        // un posizionamento affettivo, non un'espressione. FeelsAs, NON Expresses
        // (che era la discarica dei verbi mentali). La relazione vive nel verbo
        // (frame `valutativo` nel kg_proc), non in un bucket grammaticale grezzo.
        (Some("valutativo"), _)   => RelationType::FeelsAs,
        // "penso/credo X" — espressione cognitiva (credenza). Qui Expresses è
        // appropriato: è davvero un'asserzione di pensiero.
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

            // INCONGRUENZA strutturale: l'input asserisce un'IDENTITÀ o
            // SOMIGLIANZA (X è/come Y) tra due concetti che il grafo conosce
            // come OPPOSTI ("la paura è coraggio" ↔ paura OppositeOf coraggio).
            // È la tensione fra ciò che il mondo dell'input dice e ciò che il
            // MIO grafo sostiene — da evidenziare, non da nascondere.
            if matches!(prop.relation, RelationType::IsA | RelationType::SimilarTo) {
                let opposite = kg_sem.query_objects(&s_lc, RelationType::OppositeOf)
                    .iter().any(|t| t.eq_ignore_ascii_case(&o_lc))
                    || kg_sem.query_objects(&o_lc, RelationType::OppositeOf)
                        .iter().any(|t| t.eq_ignore_ascii_case(&s_lc));
                if opposite {
                    out.contradictions.push((s.clone(), o.clone()));
                }
            }
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
// confront_with_self — la frase confrontata col SÉ (Phase 85, Livello 1)
// ═══════════════════════════════════════════════════════════════════════════

/// Un edge del sé colpito dalla proposizione, con la sua magnitudine
/// (= confidenza dell'edge: la sua resistenza). Continua, mai soglia.
#[derive(Debug, Clone)]
pub struct SelfHit {
    pub subject:  String,
    pub relation: RelationType,
    pub object:   String,
    pub magnitude: f64,
    /// Polarità dell'edge del SÉ (= l'impegno dell'entità su questa tripla).
    /// È ciò che l'entità *tiene*: il rendering del posizionamento articola
    /// questa polarità ("non è" se false), non quella della PROP in ingresso.
    pub polarity: bool,
}

/// Esito del confronto fra una SentenceProposition e `kg_self` — l'OPINIONE
/// come secondo legame (gemello di `KgConfrontation`). Non è una decisione: è
/// una struttura percettiva che i decisori a valle leggeranno per risonanza
/// (seminerà `dissonanza`/`conferma` nel kg_proc — increment successivo).
///
/// Livello 1 (questo increment): solo conflitto/risonanza per match-tripla
/// esatto su soggetti `World`. `estensione` è differita (design §4.3).
#[derive(Debug, Clone, Default)]
pub struct SelfConfrontation {
    /// La PROP afferma ciò che il sé nega (o viceversa): polarità discordi.
    pub conflitti: Vec<SelfHit>,
    /// La PROP conferma un impegno del sé: polarità concordi.
    pub risonanze: Vec<SelfHit>,
}

impl SelfConfrontation {
    pub fn is_empty(&self) -> bool {
        self.conflitti.is_empty() && self.risonanze.is_empty()
    }
    /// Magnitudine del conflitto più forte (0.0 se nessuno). A valle diventa
    /// l'intensità del percetto `dissonanza`.
    pub fn max_conflitto(&self) -> f64 {
        self.conflitti.iter().map(|h| h.magnitude).fold(0.0, f64::max)
    }
    /// Magnitudine della risonanza più forte (intensità di `conferma`).
    pub fn max_risonanza(&self) -> f64 {
        self.risonanze.iter().map(|h| h.magnitude).fold(0.0, f64::max)
    }
}

/// Confronta la proposizione col grafo del sé. **Livello 1**: match-tripla
/// esatto (`subject=World(s)` ∧ `relation` ∧ `object=Word(o)`, identità
/// case-insensitive); polarità **concordi → risonanza**, **discordi →
/// conflitto**; magnitudine = `edge.confidence`. Soggetti `Speaker`/`Entity` e
/// le opposizioni via kg_sem sono **Livello 2** (differito).
/// Vedi `docs/raw/architettura/kg_self_design.md` §4.3.
pub fn confront_with_self(
    prop: &SentenceProposition,
    kg_self: &crate::topology::kg_self::KgSelf,
) -> SelfConfrontation {
    let mut out = SelfConfrontation::default();

    // Livello 1: solo (World(s), R, Word(o)). Altri casi → Livello 2.
    let (subj, obj) = match (&prop.subject, &prop.object) {
        (SubjectRef::World(s), Some(ObjectRef::Word(o))) => (s, o),
        _ => return out,
    };

    for e in kg_self.edges.iter() {
        if e.relation == prop.relation
            && e.subject.eq_ignore_ascii_case(subj)
            && e.object.eq_ignore_ascii_case(obj)
        {
            let hit = SelfHit {
                subject:  e.subject.clone(),
                relation: e.relation,
                object:   e.object.clone(),
                magnitude: e.confidence,
                polarity:  e.polarity,
            };
            // Polarità concordi → la PROP conferma il sé; discordi → conflitto.
            if e.polarity == prop.polarity {
                out.risonanze.push(hit);
            } else {
                out.conflitti.push(hit);
            }
        }
    }
    out
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
            verb_lemma: None,
            subject_surface: None,
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
    fn test_confront_with_self_level1() {
        use crate::topology::kg_self::{KgSelf, SelfEdge};
        // Il confronto opera sulle OPINIONI (derivate+validate, innate=false):
        // le innate sono dismesse — al load si dissolvono in pendenze.
        let edge = |s: &str, r: RelationType, o: &str, conf: f64, pol: bool| SelfEdge {
            subject: s.to_string(), relation: r, object: o.to_string(),
            confidence: conf, polarity: pol, innate: false, via: None,
        };
        let kg_self = KgSelf { pendenze: vec![], edges: vec![
            edge("incertezza", RelationType::IsA,  "fallimento",  0.88, false), // NON-opinione
            edge("silenzio",   RelationType::Has,  "significato", 0.89, true),
        ]};
        let prop = |subj: &str, r: RelationType, obj: &str, pol: bool| SentenceProposition {
            subject: SubjectRef::World(subj.to_string()),
            relation: r,
            object: Some(ObjectRef::Word(obj.to_string())),
            via: None, verb_lemma: None, polarity: pol, complements: vec![],
            subject_surface: None,
        };

        // "l'incertezza è un fallimento" → afferma (+) ciò che il sé nega (−) → CONFLITTO
        let c = confront_with_self(&prop("incertezza", RelationType::IsA, "fallimento", true), &kg_self);
        assert_eq!(c.conflitti.len(), 1, "polarità discordi → conflitto");
        assert!(c.risonanze.is_empty());
        assert!((c.max_conflitto() - 0.88).abs() < 1e-9, "magnitudine = confidenza dell'edge");

        // "il silenzio ha un significato" → concorda (+/+) → RISONANZA
        let r = confront_with_self(&prop("silenzio", RelationType::Has, "significato", true), &kg_self);
        assert_eq!(r.risonanze.len(), 1, "polarità concordi → risonanza");
        assert!(r.conflitti.is_empty());

        // input che non tocca il sé → vuoto
        assert!(confront_with_self(&prop("mare", RelationType::IsA, "profondo", true), &kg_self).is_empty());

        // soggetto-sé (Entity) → Livello 2, differito → vuoto in Livello 1
        let p_self = SentenceProposition {
            subject: SubjectRef::Entity, relation: RelationType::IsA,
            object: Some(ObjectRef::Word("calcolo".to_string())), via: None, verb_lemma: None, polarity: true,
            complements: vec![],
            subject_surface: None,
        };
        assert!(confront_with_self(&p_self, &kg_self).is_empty(), "self-subject è Livello 2");
    }

    #[test]
    fn io_sono_triste_yields_speaker_feelsas_triste() {
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Feeling, "triste", Some("copula"));
        let p = extract_proposition(&words("io sono triste"), Some(&c), Some(&kg_proc), None)
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
        let p = extract_proposition(&words("ho paura del futuro"), Some(&c), Some(&kg_proc), None)
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
        let p = extract_proposition(&words("mi chiamo francesco"), Some(&c), Some(&kg_proc), None)
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::IsA);
        assert_eq!(p.object, Some(ObjectRef::Word("francesco".to_string())));
    }

    #[test]
    fn vado_al_mare_yields_does_with_predicate_mare() {
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Action, "mare", Some("azione"));
        let p = extract_proposition(&words("vado al mare"), Some(&c), Some(&kg_proc), None)
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
        let p = extract_proposition(&words("chi sei"), None, Some(&kg_proc), None)
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Entity,
            "domanda con verbo 2sg → soggetto = UI-r1 (Entity)");
        assert!(matches!(p.object, Some(ObjectRef::Variable(ref v)) if v == "chi"));
    }

    #[test]
    fn non_ho_paura_yields_polarity_false() {
        let kg_proc = kg_proc_minimal();
        let c = claim(ClaimAgent::Speaker, ClaimKind::Feeling, "paura", Some("copula"));
        let p = extract_proposition(&words("non ho paura"), Some(&c), Some(&kg_proc), None)
            .expect("proposizione attesa");
        assert!(!p.polarity, "polarità doveva essere false in presenza di `non`");
    }

    #[test]
    fn no_claim_no_interrog_yields_none() {
        let kg_proc = kg_proc_minimal();
        let p = extract_proposition(&words("il sole splende"), None, Some(&kg_proc), None);
        assert!(p.is_none(),
            "senza claim e senza pronome interrogativo, niente proposizione (per ora)");
    }

    #[test]
    fn transitivo_mondo_phase86() {
        // Phase 86 #2: "il tradimento uccide la fiducia" → World(tradimento) Does
        // fiducia. Verbo 3a persona transitivo, riconosciuto via kg_sem
        // (uccidere IsA azione → verb-concept); relazione default Does (verbo non
        // categorizzato nel kg_proc); soggetto prima del verbo, oggetto dopo.
        let kg_proc = kg_proc_minimal();
        let mut kg = KnowledgeGraph::new();
        kg.add("uccidere", RelationType::IsA, "azione");
        let p = extract_proposition(
            &words("il tradimento uccide la fiducia"),
            None, Some(&kg_proc), Some(&kg),
        ).expect("proposizione transitiva sul Mondo attesa");
        assert_eq!(p.subject, SubjectRef::World("tradimento".to_string()));
        assert_eq!(p.object, Some(ObjectRef::Word("fiducia".to_string())));
        assert_eq!(p.relation, RelationType::Does);
    }

    #[test]
    fn complementi_disambiguati_phase86_stadio2() {
        // "ho paura del futuro": il complemento "del futuro" è disambiguato
        // contro il KG. Il mondo tiene "futuro Causes paura" → la preposizione
        // "di" (candidate [PartOf,Has,IsA,Causes]) si fissa su Causes.
        let kg_proc = kg_proc_minimal();
        let mut kg = KnowledgeGraph::new();
        kg.add("futuro", RelationType::Causes, "paura");
        let c = claim(ClaimAgent::Speaker, ClaimKind::Feeling, "paura", Some("percettivo"));
        let p = extract_proposition(
            &words("ho paura del futuro"),
            Some(&c), Some(&kg_proc), Some(&kg),
        ).expect("proposizione attesa");
        let comp = p.complements.iter().find(|c| c.noun == "futuro")
            .expect("complemento 'futuro' atteso");
        assert_eq!(comp.preposition, "del");
        assert_eq!(comp.relation, Some(RelationType::Causes),
            "il KG (futuro Causes paura) disambigua 'di' in Causes");
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
            verb_lemma: None,
            polarity: true,
            complements: vec![],
            subject_surface: None,
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
            verb_lemma: None,
            polarity: true,
            complements: vec![],
            subject_surface: None,
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
            Some(&c), Some(&kg_proc), None,
        ).expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::FeelsAs);
        assert_eq!(p.object, Some(ObjectRef::Word("rabbia".to_string())));
    }

    #[test]
    fn che_iniziale_resta_interrogativo() {
        // "che fai?" — "che" in posizione iniziale è interrogativo.
        let kg_proc = kg_proc_minimal();
        let p = extract_proposition(&words("che fai"), None, Some(&kg_proc), None)
            .expect("proposizione attesa");
        assert!(matches!(p.object, Some(ObjectRef::Variable(ref v)) if v == "che"));
    }

    #[test]
    fn che_cosa_preferisce_cosa() {
        // "che cosa rende vivo un pensiero?" — l'interrogativo forte "cosa"
        // vince sul "che", che da solo non è più catturato.
        let kg_proc = kg_proc_minimal();
        let p = extract_proposition(&words("che cosa rende vivo un pensiero"), None, Some(&kg_proc), None)
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
        let p = extract_proposition(&words("mi manca mia madre"), Some(&c), Some(&kg_proc), None)
            .expect("proposizione attesa");
        assert_eq!(p.subject, SubjectRef::Speaker);
        assert_eq!(p.relation, RelationType::FeelsAs);
        assert_eq!(p.object, Some(ObjectRef::Word("mancanza".to_string())));
        assert_eq!(p.via, Some("madre".to_string()),
            "il tema dativo deve riempire lo slot via");
    }
}
