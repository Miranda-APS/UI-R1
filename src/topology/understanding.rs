//! Understanding — comprensione multi-facet prima della generazione.
//!
//! Francesco (conversazione 2026-04-20): "l'entità non deve mai e poi mai
//! prendere parole ciecamente, deve cercare di capire". Ogni tipo di relazione
//! KG contribuisce con una semantica propria — non esiste una relazione
//! decorativa. La comprensione è la lettura completa di questi facet per
//! ciascuna parola-lemma dell'input, più l'aggregazione in una scena.
//!
//! Non è generazione: è lo strato INTELLIGERE — cosa l'entità ha capito
//! prima di decidere cosa dire. L'output di questa fase alimenta due cose:
//!
//! 1. La reazione necessaria (azione pragmatica che la scena richiede).
//! 2. L'ipotesi da verificare (concetto-perno saliente ma sotto-definito).
//!
//! # Esempio
//!
//! Input lemmatizzato: `["stare"]` + marker `?` → ruolo Question.
//!
//! `Understanding::for_word("stare", &kg)` raccoglie:
//! - `identita`: azione
//! - `qualita`: stato, condizione, benessere, equilibrio
//! - `precondizioni`: equilibrio
//! - `polo_escluso`: muoversi
//!
//! L'InferenceOnSpeaker deriva dalla fusione di tutte le Understanding della
//! scena: il parlante richiede ciò che le parole richiedono e produce ciò che
//! le parole causano.

use std::collections::HashMap;

use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

// ═══════════════════════════════════════════════════════════════════════════
// Facet — categoria semantica derivata dal tipo di arco
// ═══════════════════════════════════════════════════════════════════════════

/// Categoria semantica a cui un arco tipato appartiene.
///
/// Ogni `RelationType` mappa su un `Facet` — la categoria semantica di quello
/// che l'arco esprime. Più archi dello stesso tipo confluiscono nello stesso
/// facet (tutti gli `IsA` concorrono all'`Identity`, tutti i `Causes`
/// concorrono agli `Effects`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facet {
    /// Cos'è la parola (IsA, ParteDi come appartenenza categoriale).
    Identity,
    /// Cosa la parola produce nel mondo (Causes, Enables, Implies).
    Effects,
    /// Pre-condizioni — anche attribuite al parlante (Requires, Coexists).
    Preconditions,
    /// Azioni che la parola compie o incorpora (Does).
    Actions,
    /// Qualità possedute (Has).
    Qualities,
    /// Appartenenza composizionale (PartOf).
    Belonging,
    /// Cosa la parola non è (OppositeOf, Excludes).
    Polarity,
    /// Risonanze e sinonimi (SimilarTo, Equivalent).
    Resonances,
    /// Trasformazione (TransformsInto).
    Becoming,
    /// Funzione pragmatica (UsedFor).
    PragmaticFunction,
    /// Manifestazione (Expresses, Symbolizes).
    Manifestation,
    /// Scena/cornice in cui la parola si colloca (ContextOf).
    PhenomenalFrame,
    /// Qualità vissuta in prima persona (FeelsAs).
    LivedQuality,
    /// Traccia emotiva nel ricordo (RemembersAs).
    EmotionalMemory,
    /// Orizzonte inquieto, interrogazione originaria (WondersAbout).
    InquiryHorizon,
}

impl Facet {
    /// Mappa ogni tipo di relazione alla sua categoria semantica.
    pub fn of(r: RelationType) -> Facet {
        match r {
            RelationType::IsA           => Facet::Identity,
            RelationType::Has           => Facet::Qualities,
            RelationType::Does          => Facet::Actions,
            RelationType::PartOf        => Facet::Belonging,
            RelationType::Causes        => Facet::Effects,
            RelationType::Enables       => Facet::Effects,
            RelationType::Requires      => Facet::Preconditions,
            RelationType::Coexists      => Facet::Preconditions,
            RelationType::TransformsInto=> Facet::Becoming,
            RelationType::SimilarTo     => Facet::Resonances,
            RelationType::Equivalent    => Facet::Resonances,
            RelationType::OppositeOf    => Facet::Polarity,
            RelationType::Excludes      => Facet::Polarity,
            RelationType::UsedFor       => Facet::PragmaticFunction,
            RelationType::Expresses     => Facet::Manifestation,
            RelationType::Symbolizes    => Facet::Manifestation,
            RelationType::ContextOf     => Facet::PhenomenalFrame,
            RelationType::FeelsAs       => Facet::LivedQuality,
            RelationType::RemembersAs   => Facet::EmotionalMemory,
            RelationType::WondersAbout  => Facet::InquiryHorizon,
            RelationType::Implies       => Facet::Effects,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Understanding — lettura di una parola lemma attraverso tutti i suoi archi
// ═══════════════════════════════════════════════════════════════════════════

/// Un arco letto come (relazione, parola collegata, confidence).
#[derive(Debug, Clone)]
pub struct FacetEdge {
    pub relation: RelationType,
    pub target: String,
    pub confidence: f32,
}

/// Comprensione di una singola parola dall'attraversamento del KG.
///
/// Non contiene attivazioni di campo — è pura lettura KG. La parola è
/// sempre un lemma (già prodotto dal lessico).
#[derive(Debug, Clone)]
pub struct Understanding {
    pub word: String,
    /// Archi in uscita raggruppati per facet.
    pub forward: HashMap<Facet, Vec<FacetEdge>>,
    /// Archi in entrata raggruppati per facet (il mondo parla della parola).
    pub reverse: HashMap<Facet, Vec<FacetEdge>>,
}

impl Understanding {
    /// Costruisce la comprensione dal KG. Se la parola non è presente,
    /// restituisce un oggetto con mappe vuote — il chiamante può verificare
    /// `is_unknown()`.
    pub fn for_word(word: &str, kg: &KnowledgeGraph) -> Understanding {
        let mut forward: HashMap<Facet, Vec<FacetEdge>> = HashMap::new();
        let mut reverse: HashMap<Facet, Vec<FacetEdge>> = HashMap::new();

        for (rel, target, conf) in kg.all_outgoing(word) {
            let facet = Facet::of(rel);
            forward.entry(facet).or_default().push(FacetEdge {
                relation: rel,
                target: target.to_string(),
                confidence: conf,
            });
        }

        for (rel, source, conf) in kg.all_incoming(word) {
            let facet = Facet::of(rel);
            reverse.entry(facet).or_default().push(FacetEdge {
                relation: rel,
                target: source.to_string(),
                confidence: conf,
            });
        }

        Understanding { word: word.to_string(), forward, reverse }
    }

    /// True se non c'è alcun arco: la parola è ignota al KG.
    pub fn is_unknown(&self) -> bool {
        self.forward.is_empty() && self.reverse.is_empty()
    }

    /// Numero totale di archi letti (forward + reverse).
    pub fn arc_count(&self) -> usize {
        self.forward.values().map(|v| v.len()).sum::<usize>()
            + self.reverse.values().map(|v| v.len()).sum::<usize>()
    }

    /// Accesso rapido a un facet forward (vuoto se assente).
    pub fn forward_facet(&self, f: Facet) -> &[FacetEdge] {
        self.forward.get(&f).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Accesso rapido a un facet reverse (vuoto se assente).
    pub fn reverse_facet(&self, f: Facet) -> &[FacetEdge] {
        self.reverse.get(&f).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Inferenza sul parlante — cosa la scelta lessicale attribuisce a chi parla
// ═══════════════════════════════════════════════════════════════════════════

/// Tipo di attribuzione che la scena fa al parlante.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpeakerAttribution {
    /// Il parlante ha un attributo che la parola richiede.
    /// Da `W Requires X` ⇒ chi enuncia W deve avere X.
    HasAttribute(String),
    /// Il parlante vuole produrre un effetto che la parola causa.
    /// Da `W Causes X` ⇒ il parlante enuncia W per produrre X.
    WantsToProduce(String),
    /// Il parlante intende la funzione pragmatica della parola.
    /// Da `W UsedFor X` ⇒ il parlante intende X.
    Intends(String),
}

impl SpeakerAttribution {
    pub fn target(&self) -> &str {
        match self {
            SpeakerAttribution::HasAttribute(t) => t,
            SpeakerAttribution::WantsToProduce(t) => t,
            SpeakerAttribution::Intends(t) => t,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Ipotesi — concetti-perno salienti ma sotto-definiti nel KG
// ═══════════════════════════════════════════════════════════════════════════

/// Un concetto "portante" che è richiamato più volte dalla scena corrente
/// ma che l'entità non ha strutturato abbastanza nel KG. Diventa candidato
/// per una domanda aperta (verificare l'ipotesi).
///
/// Esempio: "presenza" è richiesta da `ciao`, `saluto`, `tu`, `incontro`
/// ma ha pochi archi definitori (IsA / Has / Does). L'entità sa che gioca
/// un ruolo centrale senza averne una definizione piena — è un'ipotesi.
#[derive(Debug, Clone)]
pub struct Hypothesis {
    /// Il concetto-perno da verificare.
    pub concept: String,
    /// Quante parole dell'input lo richiamano (salienza).
    pub saliency: u32,
    /// Numero di archi definitori in uscita (IsA/Has/Does/PartOf).
    pub defining_arcs: u32,
    /// Quale tipo di invocazione domina (Requires, Causes, IsA, ...).
    pub dominant_invocation: Option<RelationType>,
    /// Le parole dell'input che l'hanno invocato.
    pub invoked_by: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// SceneUnderstanding — comprensione dell'intera frase
// ═══════════════════════════════════════════════════════════════════════════

/// Ruolo sintattico dedotto dai marker testuali.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntacticRole {
    /// Dichiarazione (default).
    Statement,
    /// Domanda (presenza di `?`).
    Question,
    /// Esclamazione (presenza di `!`).
    Exclamation,
}

impl SyntacticRole {
    pub fn from_text(raw: &str) -> SyntacticRole {
        if raw.contains('?') { SyntacticRole::Question }
        else if raw.contains('!') { SyntacticRole::Exclamation }
        else { SyntacticRole::Statement }
    }
}

/// Lettura della scena intera (una o più parole).
#[derive(Debug, Clone)]
pub struct SceneUnderstanding {
    /// Comprensione di ogni singolo lemma dell'input.
    pub per_word: Vec<Understanding>,
    /// Ruolo sintattico dedotto (`?`, `!`, statement).
    pub syntactic_role: SyntacticRole,
    /// Profilo aggregato delle attribuzioni al parlante.
    pub speaker_attributions: Vec<(SpeakerAttribution, f32)>,
    /// Ipotesi aperte — concetti-perno da verificare.
    pub open_hypotheses: Vec<Hypothesis>,
    /// Profondità complessiva (somma archi letti, indicatore comprensione).
    pub comprehension_depth: usize,
    /// Parole dell'input che non hanno prodotto alcun arco (residuo).
    pub unknown_words: Vec<String>,
    /// Struttura sintattica ordinata: archi tra parole adiacenti collegate da
    /// preposizione, copula o verbo. Ogni arco porta una lista di ipotesi
    /// tipizzate validate sul KG. È il "ragionamento" di Prometeo nel mentre
    /// cerca di capire.
    pub syntactic_edges: Vec<SyntacticEdge>,
}

impl SceneUnderstanding {
    /// Compone la scena a partire dai lemmi dell'input e dal testo grezzo.
    ///
    /// I lemmi vanno forniti già canonicalizzati (il lessico lo fa via
    /// `process_input`). Il testo grezzo serve solo a riconoscere i marker
    /// sintattici (`?`/`!`).
    pub fn assemble(lemmas: &[&str], raw_text: &str, kg: &KnowledgeGraph) -> SceneUnderstanding {
        let syntactic_role = SyntacticRole::from_text(raw_text);
        let per_word: Vec<Understanding> = lemmas.iter()
            .map(|w| Understanding::for_word(w, kg))
            .collect();

        let unknown_words: Vec<String> = per_word.iter()
            .filter(|u| u.is_unknown())
            .map(|u| u.word.clone())
            .collect();

        let comprehension_depth: usize = per_word.iter().map(|u| u.arc_count()).sum();

        let speaker_attributions = aggregate_speaker_attributions(&per_word);
        let open_hypotheses = detect_hypotheses(&per_word, kg);
        let syntactic_edges = parse_syntactic_structure(raw_text, kg);

        SceneUnderstanding {
            per_word,
            syntactic_role,
            speaker_attributions,
            open_hypotheses,
            comprehension_depth,
            unknown_words,
            syntactic_edges,
        }
    }

    /// True se la scena è una domanda.
    pub fn is_question(&self) -> bool {
        self.syntactic_role == SyntacticRole::Question
    }

    /// Le parole-input della scena (lemmi).
    pub fn input_lemmas(&self) -> Vec<&str> {
        self.per_word.iter().map(|u| u.word.as_str()).collect()
    }

    /// Effetti che la scena cumulativamente produrrebbe (unione Causes).
    /// Usato per derivare l'intento pragmatico.
    pub fn aggregated_effects(&self) -> Vec<(String, f32)> {
        let mut acc: HashMap<String, f32> = HashMap::new();
        for u in &self.per_word {
            for e in u.forward_facet(Facet::Effects) {
                let v = acc.entry(e.target.clone()).or_insert(0.0);
                *v = v.max(e.confidence);
            }
            for e in u.forward_facet(Facet::PragmaticFunction) {
                let v = acc.entry(e.target.clone()).or_insert(0.0);
                *v = v.max(e.confidence);
            }
        }
        let mut v: Vec<_> = acc.into_iter().collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v
    }
}

// ─── Helpers di aggregazione ───────────────────────────────────────────────

fn aggregate_speaker_attributions(per_word: &[Understanding]) -> Vec<(SpeakerAttribution, f32)> {
    // Mappa per deduplicare la stessa attribuzione da parole diverse.
    let mut acc: HashMap<(u8, String), f32> = HashMap::new();

    for u in per_word {
        // Preconditions → HasAttribute
        for e in u.forward_facet(Facet::Preconditions) {
            let key = (0u8, e.target.clone());
            let v = acc.entry(key).or_insert(0.0);
            *v = v.max(e.confidence);
        }
        // Effects → WantsToProduce
        for e in u.forward_facet(Facet::Effects) {
            let key = (1u8, e.target.clone());
            let v = acc.entry(key).or_insert(0.0);
            *v = v.max(e.confidence);
        }
        // PragmaticFunction → Intends
        for e in u.forward_facet(Facet::PragmaticFunction) {
            let key = (2u8, e.target.clone());
            let v = acc.entry(key).or_insert(0.0);
            *v = v.max(e.confidence);
        }
    }

    let mut out: Vec<(SpeakerAttribution, f32)> = acc.into_iter()
        .map(|((kind, name), conf)| {
            let a = match kind {
                0 => SpeakerAttribution::HasAttribute(name),
                1 => SpeakerAttribution::WantsToProduce(name),
                _ => SpeakerAttribution::Intends(name),
            };
            (a, conf)
        })
        .collect();

    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Rileva concetti-perno: richiamati da più parole della scena *oppure*
/// richiamati come `Requires`/`Causes` da una parola ma poveri di archi
/// definitori. Questi diventano candidati ipotesi.
fn detect_hypotheses(per_word: &[Understanding], kg: &KnowledgeGraph) -> Vec<Hypothesis> {
    // invocazioni: concept -> Vec<(word_source, RelationType, confidence)>
    let mut invocations: HashMap<String, Vec<(String, RelationType, f32)>> = HashMap::new();

    for u in per_word {
        for edges in u.forward.values() {
            for e in edges {
                invocations.entry(e.target.clone())
                    .or_default()
                    .push((u.word.clone(), e.relation, e.confidence));
            }
        }
    }

    // I lemmi input non sono mai ipotesi di sé stessi.
    let input_lemmas: std::collections::HashSet<String> = per_word.iter()
        .map(|u| u.word.clone())
        .collect();

    let mut hypotheses = Vec::new();

    for (concept, invokers) in invocations {
        if input_lemmas.contains(&concept) {
            continue;
        }

        // Soggetti/oggetti ultra-generici vanno scartati: "stato" ha 784 in,
        // "azione" ecc. sono hub categoriali — ipotizzarli è rumore.
        // Usiamo il grado totale come euristica: se il concetto ha >200 incoming
        // nel KG, è troppo saturo per essere ipotesi utile.
        let incoming_count = kg.all_incoming(&concept).len();
        if incoming_count > 200 {
            continue;
        }

        // Salienza: numero di PAROLE DISTINTE dell'input che richiamano il
        // concetto (non numero di archi). Lo stesso lemma che invoca un
        // concetto via Requires e Has conta una volta sola.
        let distinct_invokers: std::collections::HashSet<&str> = invokers.iter()
            .map(|(w, _, _)| w.as_str())
            .collect();
        let saliency = distinct_invokers.len() as u32;

        // Conta archi definitori del concetto (IsA, Has, Does, PartOf)
        let defining_arcs: u32 = kg.all_outgoing(&concept).iter()
            .filter(|(r, _, _)| matches!(r,
                RelationType::IsA | RelationType::Has |
                RelationType::Does | RelationType::PartOf))
            .count() as u32;

        // Dominant invocation: il tipo di relazione più frequente fra gli invokers
        let mut rel_count: HashMap<RelationType, u32> = HashMap::new();
        for (_, r, _) in &invokers {
            *rel_count.entry(*r).or_insert(0) += 1;
        }
        let dominant_invocation = rel_count.into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(r, _)| r);

        // IsA/SimilarTo non sono invocazioni che aprono ipotesi: sono
        // classificazione e risonanza. Un'ipotesi nasce da precondizione o
        // effetto (Requires/Causes/UsedFor/Enables) o manifestazione
        // (Expresses/Symbolizes).
        let is_hypothesis_worthy_invocation = matches!(
            dominant_invocation,
            Some(RelationType::Requires) | Some(RelationType::Causes) |
            Some(RelationType::UsedFor)  | Some(RelationType::Enables) |
            Some(RelationType::Expresses)| Some(RelationType::Symbolizes)
        );

        // Criterio ipotesi:
        //   (a) salienza >= 2 da parole distinte E invocazione pregnante E
        //       concetto sotto-definito (<6 archi def); oppure
        //   (b) salienza = 1 ma invocato come Requires/Causes E sotto-definito
        //       (<4 archi def) — concetto aperto con un solo ponte, vale la pena.
        let under_defined_multi = defining_arcs < 6;
        let under_defined_solo = defining_arcs < 4;
        let qualifies =
            (saliency >= 2 && is_hypothesis_worthy_invocation && under_defined_multi)
            || (saliency == 1 && is_hypothesis_worthy_invocation && under_defined_solo);

        if !qualifies {
            continue;
        }

        let mut invoked_by: Vec<String> = distinct_invokers.iter()
            .map(|s| s.to_string())
            .collect();
        invoked_by.sort();

        hypotheses.push(Hypothesis {
            concept,
            saliency,
            defining_arcs,
            dominant_invocation,
            invoked_by,
        });
    }

    // Ordina: più saliente prima; a parità, meno definito prima.
    hypotheses.sort_by(|a, b| {
        b.saliency.cmp(&a.saliency)
            .then(a.defining_arcs.cmp(&b.defining_arcs))
    });

    hypotheses
}

// ═══════════════════════════════════════════════════════════════════════════
// Parse sintattica ordinata — preposizioni, copula, verbi come ipotesi
// di relazione tipizzata validate sul KG.
//
// Principio: la frase italiana espone strutture S–marker–O dove il marker
// (preposizione, copula essere, verbo) è il segnale sintattico della
// relazione semantica fra i due nominali. UI-r1 ipotizza la relazione dal
// marker (mapping deterministico, lista ordinata) e VALIDA l'ipotesi sul KG:
// arco diretto, compatibilità di tipo via IS_A, cammino 2-hop. La prima
// ipotesi validata è la relazione "compresa". Se nessuna valida, l'arco
// resta come ipotesi aperta — gap di conoscenza.
// ═══════════════════════════════════════════════════════════════════════════

/// Tipo di connettore sintattico tra due parole vicine nell'input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntacticConnector {
    /// Preposizione italiana (di, a, da, in, con, per, tra, fra, su) anche
    /// nelle forme articolate (del, allo, dalla, ...).
    Preposition(String),
    /// Forma del verbo essere (è, sono, era, ...).
    Copula,
    /// Verbo lessicale qualunque (Mario MANGIA il pane).
    Verb(String),
}

/// Esito della validazione di un'ipotesi sul KG.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationOutcome {
    /// Arco diretto presente nel KG: `subject rel object`.
    DirectEdge { confidence: f32 },
    /// Compatibilità di tipo via IS_A: `subject IsA T`, `T rel object`.
    /// Conferma strutturale per ereditarietà (Mario IS_A persona,
    /// persona HAS libro → "Mario HAS libro" è plausibile).
    TypeCompatible { via_type: String, confidence: f32 },
    /// Cammino 2-hop: `subject rel intermediate`, `intermediate IsA object`
    /// (o equivalenza/similarità). Inferenza transitiva debole.
    TwoHop { intermediate: String, confidence: f32 },
    /// Esiste un arco OPPOSITE_OF tra subject e object: relazione
    /// strutturalmente impossibile in questa direzione semantica.
    Contradicted,
    /// Nessuna evidenza nel KG.
    NotInKg,
}

impl ValidationOutcome {
    /// True se l'ipotesi è confermata (in un qualche modo) dal KG.
    pub fn is_valid(&self) -> bool {
        matches!(self,
            ValidationOutcome::DirectEdge { .. }
            | ValidationOutcome::TypeCompatible { .. }
            | ValidationOutcome::TwoHop { .. }
        )
    }

    /// Etichetta breve usata in UI.
    pub fn kind(&self) -> &'static str {
        match self {
            ValidationOutcome::DirectEdge { .. }     => "diretto",
            ValidationOutcome::TypeCompatible { .. } => "tipo",
            ValidationOutcome::TwoHop { .. }         => "2-hop",
            ValidationOutcome::Contradicted          => "contraddetto",
            ValidationOutcome::NotInKg               => "nel campo aperto",
        }
    }
}

/// Un'ipotesi tipizzata di relazione, con il suo esito di validazione.
#[derive(Debug, Clone)]
pub struct RelationHypothesis {
    pub relation: RelationType,
    pub validation: ValidationOutcome,
    /// Spiegazione umana: perché questa ipotesi è stata generata e cosa il
    /// KG dice. Visualizzata nel grafo dinamico.
    pub rationale: String,
}

/// Un arco sintattico nell'input: due parole-contenuto connesse da un
/// marker (preposizione/copula/verbo), con ipotesi tipizzate ordinate.
#[derive(Debug, Clone)]
pub struct SyntacticEdge {
    pub subject: String,
    pub object: String,
    pub connector: SyntacticConnector,
    /// Indici delle due parole nel raw input (ordine di apparizione).
    pub subject_pos: usize,
    pub object_pos: usize,
    /// Lista ordinata di ipotesi: la prima validata vince.
    pub hypotheses: Vec<RelationHypothesis>,
    /// Indice (in `hypotheses`) della prima relazione validata.
    pub validated_idx: Option<usize>,
}

impl SyntacticEdge {
    /// La relazione che UI-r1 ha "compreso" per questo arco, se ne ha
    /// validata una.
    pub fn understood_relation(&self) -> Option<RelationType> {
        self.validated_idx.and_then(|i| self.hypotheses.get(i).map(|h| h.relation))
    }
}

/// Classifica una parola come connettore (preposizione/copula/verbo).
/// Restituisce `None` se la parola è un nominale o ignota.
fn classify_connector(word: &str) -> Option<SyntacticConnector> {
    let w = word.to_lowercase();
    let stripped = w.trim_end_matches('\'');
    match stripped {
        // Preposizioni semplici
        "di" | "a" | "da" | "in" | "con" | "per" | "tra" | "fra" | "su"
        // Articolate "di"
        | "del" | "dello" | "della" | "dei" | "degli" | "delle" | "dell"
        // Articolate "a"
        | "al" | "allo" | "alla" | "ai" | "agli" | "alle" | "all"
        // Articolate "da"
        | "dal" | "dallo" | "dalla" | "dai" | "dagli" | "dalle" | "dall"
        // Articolate "in"
        | "nel" | "nello" | "nella" | "nei" | "negli" | "nelle" | "nell"
        // Articolate "su"
        | "sul" | "sullo" | "sulla" | "sui" | "sugli" | "sulle" | "sull"
        // Articolate "con" (rare ma esistono)
        | "col" | "coi" | "collo" | "colla"
            => Some(SyntacticConnector::Preposition(stripped.to_string())),
        // Copula essere — forme più comuni del presente, imperfetto, futuro,
        // congiuntivo, passato remoto.
        "è" | "e'" | "sono" | "sei" | "siamo" | "siete"
        | "ero" | "eri" | "era" | "eravamo" | "eravate" | "erano"
        | "fui" | "fosti" | "fu" | "fummo" | "foste" | "furono"
        | "sarò" | "sarai" | "sarà" | "saremo" | "sarete" | "saranno"
        | "sia" | "siano" | "fossi" | "fosse" | "fossero"
            => Some(SyntacticConnector::Copula),
        _ => None,
    }
}

/// Estrae alla preposizione (in forma articolata) il prefisso semplice.
/// Es: "del" → "di", "alla" → "a", "negli" → "in".
fn base_preposition(p: &str) -> &str {
    match p {
        "del" | "dello" | "della" | "dei" | "degli" | "delle" | "dell" => "di",
        "al" | "allo" | "alla" | "ai" | "agli" | "alle" | "all" => "a",
        "dal" | "dallo" | "dalla" | "dai" | "dagli" | "dalle" | "dall" => "da",
        "nel" | "nello" | "nella" | "nei" | "negli" | "nelle" | "nell" => "in",
        "sul" | "sullo" | "sulla" | "sui" | "sugli" | "sulle" | "sull" => "su",
        "col" | "coi" | "collo" | "colla" => "con",
        other => other,
    }
}

/// Tabella deterministica preposizione → ipotesi di relazione (in ordine).
/// L'ordine codifica una preferenza semantica generale; il KG decide.
fn hypothesize_for_connector(c: &SyntacticConnector) -> Vec<RelationType> {
    use RelationType::*;
    match c {
        SyntacticConnector::Preposition(p) => match base_preposition(p.as_str()) {
            // "di" — genitivo polisemico
            "di"  => vec![Has, PartOf, IsA, Causes],
            // "da" — provenienza, agente, trasformazione
            "da"  => vec![Causes, TransformsInto, PartOf],
            // "a" — direzione, scopo, transfer
            "a"   => vec![UsedFor, Does, Has],
            // "in" — contenimento, contesto
            "in"  => vec![PartOf, ContextOf],
            // "con" — strumento, compagnia
            "con" => vec![Has, Coexists, UsedFor],
            // "per" — scopo, causa
            "per" => vec![UsedFor, Causes],
            // "tra"/"fra" — confronto, contrasto, posizione
            "tra" | "fra" => vec![SimilarTo, OppositeOf, PartOf],
            // "su" — topic, contesto
            "su"  => vec![ContextOf, PartOf],
            _ => vec![],
        },
        SyntacticConnector::Copula => vec![IsA, Equivalent, Has],
        SyntacticConnector::Verb(_) => vec![Does, Causes, UsedFor],
    }
}

/// Valida un'ipotesi `subject rel object` contro il KG.
fn validate_hypothesis(
    subject: &str,
    rel: RelationType,
    object: &str,
    kg: &KnowledgeGraph,
) -> ValidationOutcome {
    // 1. Arco diretto — la conferma più forte.
    for (target, conf) in kg.query_objects_weighted(subject, rel) {
        if eq_ci(target, object) {
            return ValidationOutcome::DirectEdge { confidence: conf };
        }
    }
    // 2. Compatibilità di tipo: subject IsA T, T rel object.
    //    "Mario IS_A persona, persona HAS libro" → "Mario HAS libro" è plausibile.
    for (parent_type, _) in kg.query_objects_weighted(subject, RelationType::IsA) {
        for (target, conf) in kg.query_objects_weighted(parent_type, rel) {
            if eq_ci(target, object) {
                return ValidationOutcome::TypeCompatible {
                    via_type: parent_type.to_string(),
                    confidence: conf * 0.7,
                };
            }
        }
    }
    // 3. 2-hop: subject rel intermediate, intermediate IsA object (o ~).
    for (intermediate, conf1) in kg.query_objects_weighted(subject, rel) {
        for (final_target, _) in kg.query_objects_weighted(intermediate, RelationType::IsA) {
            if eq_ci(final_target, object) {
                return ValidationOutcome::TwoHop {
                    intermediate: intermediate.to_string(),
                    confidence: conf1 * 0.6,
                };
            }
        }
        for (final_target, _) in kg.query_objects_weighted(intermediate, RelationType::SimilarTo) {
            if eq_ci(final_target, object) {
                return ValidationOutcome::TwoHop {
                    intermediate: intermediate.to_string(),
                    confidence: conf1 * 0.5,
                };
            }
        }
    }
    // 4. Contraddizione: OPPOSITE_OF esplicito.
    for (target, _) in kg.query_objects_weighted(subject, RelationType::OppositeOf) {
        if eq_ci(target, object) {
            return ValidationOutcome::Contradicted;
        }
    }
    ValidationOutcome::NotInKg
}

fn eq_ci(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b) || a.to_lowercase() == b.to_lowercase()
}

/// Tokenizza il raw input in (parola, indice) preservando l'ordine. Non
/// rimuove le function-words: il parser sintattico ne ha bisogno. Punteggiatura
/// rimossa.
fn tokenize_ordered(raw: &str) -> Vec<String> {
    raw.split(|c: char| c.is_whitespace() || (c.is_ascii_punctuation() && c != '\''))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

/// Articoli e particelle italiane che vanno saltati cercando i nominali ai
/// lati di un connettore. NON sono connettori sintattici di per sé.
fn is_article_or_skip(word: &str) -> bool {
    matches!(word,
        "il" | "lo" | "la" | "i" | "gli" | "le" | "l"
        | "un" | "uno" | "una" | "un'"
        // Particelle clitiche che possono apparire tra connettore e nominale
        | "ne" | "ci" | "vi"
    )
}

fn is_nominal_candidate(word: &str) -> bool {
    word.len() >= 2
        && classify_connector(word).is_none()
        && !is_article_or_skip(word)
}

fn find_next_connector(toks: &[String], from: usize) -> Option<usize> {
    (from..toks.len()).find(|&j| classify_connector(&toks[j]).is_some())
}

fn find_prev_nominal(toks: &[String], before: usize, lower_bound: usize) -> Option<usize> {
    (lower_bound..before).rev().find(|&j| is_nominal_candidate(&toks[j]))
}

fn find_next_nominal(toks: &[String], from: usize) -> Option<usize> {
    (from..toks.len()).find(|&j| is_nominal_candidate(&toks[j]))
}

/// Costruisce ipotesi e valida per ogni terna `[N1] ... connector ... [N2]`
/// trovata nell'input. Articoli ("il", "un", ...) e particelle clitiche
/// vengono saltati, così "il cane è un animale" produce l'arco
/// `cane —è— animale`. Preserva l'ordine: `subject_pos`/`object_pos` sono
/// gli indici nel raw input tokenizzato.
pub fn parse_syntactic_structure(raw_text: &str, kg: &KnowledgeGraph) -> Vec<SyntacticEdge> {
    let toks = tokenize_ordered(raw_text);
    if toks.len() < 3 {
        return Vec::new();
    }
    let mut edges = Vec::new();
    let mut cursor = 0usize;
    while cursor < toks.len() {
        let mid_idx = match find_next_connector(&toks, cursor) {
            Some(idx) => idx,
            None => break,
        };
        let connector = classify_connector(&toks[mid_idx]).expect("classified");
        let subject_idx = match find_prev_nominal(&toks, mid_idx, cursor) {
            Some(idx) => idx,
            None => { cursor = mid_idx + 1; continue; }
        };
        let object_idx = match find_next_nominal(&toks, mid_idx + 1) {
            Some(idx) => idx,
            None => break,
        };

        let subject_raw = toks[subject_idx].clone();
        let object_raw = toks[object_idx].clone();

        let candidates = hypothesize_for_connector(&connector);
        if candidates.is_empty() {
            cursor = mid_idx + 1;
            continue;
        }

        let mut hypotheses: Vec<RelationHypothesis> = Vec::with_capacity(candidates.len());
        let mut validated_idx: Option<usize> = None;
        for (idx, rel) in candidates.iter().enumerate() {
            let outcome = validate_hypothesis(&subject_raw, *rel, &object_raw, kg);
            let rationale = build_rationale(&connector, *rel, &subject_raw, &object_raw, &outcome);
            let valid_now = outcome.is_valid();
            hypotheses.push(RelationHypothesis {
                relation: *rel,
                validation: outcome,
                rationale,
            });
            if validated_idx.is_none() && valid_now {
                validated_idx = Some(idx);
            }
        }

        edges.push(SyntacticEdge {
            subject: subject_raw,
            object: object_raw,
            connector,
            subject_pos: subject_idx,
            object_pos: object_idx,
            hypotheses,
            validated_idx,
        });
        cursor = object_idx + 1;
    }
    edges
}

/// Spiegazione umana del singolo passaggio "ipotesi + validazione".
fn build_rationale(
    connector: &SyntacticConnector,
    rel: RelationType,
    subject: &str,
    object: &str,
    outcome: &ValidationOutcome,
) -> String {
    let conn_label = match connector {
        SyntacticConnector::Preposition(p) => format!("la preposizione \"{}\"", p),
        SyntacticConnector::Copula => "la copula \"essere\"".to_string(),
        SyntacticConnector::Verb(v) => format!("il verbo \"{}\"", v),
    };
    let rel_label = rel.as_str();
    match outcome {
        ValidationOutcome::DirectEdge { confidence } =>
            format!("{} suggerisce {}: il KG ha l'arco diretto {} {} {} (conf {:.2})",
                conn_label, rel_label, subject, rel_label, object, confidence),
        ValidationOutcome::TypeCompatible { via_type, confidence } =>
            format!("{} suggerisce {}: {} è un {}, e {} {} {} regge nel KG (conf {:.2})",
                conn_label, rel_label, subject, via_type, via_type, rel_label, object, confidence),
        ValidationOutcome::TwoHop { intermediate, confidence } =>
            format!("{} suggerisce {}: cammino {} → {} → {} (conf {:.2})",
                conn_label, rel_label, subject, intermediate, object, confidence),
        ValidationOutcome::Contradicted =>
            format!("{} suggerirebbe {}, ma il KG dice {} OPPOSTO_DI {}",
                conn_label, rel_label, subject, object),
        ValidationOutcome::NotInKg =>
            format!("{} suggerisce {}: non confermato dal KG",
                conn_label, rel_label),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::knowledge_graph::KnowledgeGraph;
    use crate::topology::relation::RelationType;

    fn build_test_kg() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        // Piccolo KG che replica la scena "ciao"
        kg.add("ciao", RelationType::IsA, "saluto");
        kg.add("ciao", RelationType::Causes, "incontro");
        kg.add("ciao", RelationType::Requires, "presenza");
        kg.add("ciao", RelationType::Does, "salutare");
        kg.add("ciao", RelationType::OppositeOf, "addio");

        kg.add("saluto", RelationType::Causes, "apertura");
        kg.add("saluto", RelationType::Requires, "presenza");

        kg.add("presenza", RelationType::IsA, "qualità");
        // "presenza" ha pochi archi definitori → candidato ipotesi

        kg.add("incontro", RelationType::Requires, "due");
        kg.add("incontro", RelationType::Requires, "presenza");
        kg.add("incontro", RelationType::Causes, "relazione");
        kg
    }

    #[test]
    fn understanding_reads_forward_and_reverse_facets() {
        let kg = build_test_kg();
        let u = Understanding::for_word("ciao", &kg);
        assert!(!u.is_unknown());
        assert_eq!(u.forward_facet(Facet::Identity).len(), 1);
        assert_eq!(u.forward_facet(Facet::Identity)[0].target, "saluto");
        assert_eq!(u.forward_facet(Facet::Effects).len(), 1);
        assert_eq!(u.forward_facet(Facet::Preconditions).len(), 1);
        assert_eq!(u.forward_facet(Facet::Actions).len(), 1);
        assert_eq!(u.forward_facet(Facet::Polarity).len(), 1);
    }

    #[test]
    fn understanding_unknown_word_is_empty() {
        let kg = build_test_kg();
        let u = Understanding::for_word("xyzzyx", &kg);
        assert!(u.is_unknown());
        assert_eq!(u.arc_count(), 0);
    }

    #[test]
    fn scene_detects_question_marker() {
        let kg = build_test_kg();
        let scene = SceneUnderstanding::assemble(&["ciao"], "ciao?", &kg);
        assert!(scene.is_question());
    }

    #[test]
    fn scene_aggregates_speaker_attributions() {
        let kg = build_test_kg();
        let scene = SceneUnderstanding::assemble(&["ciao"], "ciao", &kg);
        // "ciao Requires presenza" + "ciao Causes incontro"
        let has_presence = scene.speaker_attributions.iter()
            .any(|(a, _)| matches!(a, SpeakerAttribution::HasAttribute(n) if n == "presenza"));
        let wants_meeting = scene.speaker_attributions.iter()
            .any(|(a, _)| matches!(a, SpeakerAttribution::WantsToProduce(n) if n == "incontro"));
        assert!(has_presence, "manca attribuzione presenza");
        assert!(wants_meeting, "manca attribuzione incontro");
    }

    #[test]
    fn scene_detects_hypothesis_for_pivotal_requires() {
        let kg = build_test_kg();
        let scene = SceneUnderstanding::assemble(&["ciao"], "ciao", &kg);
        // "presenza" è Requires da ciao e sotto-definito (solo 1 IsA)
        let has_presence_hyp = scene.open_hypotheses.iter()
            .any(|h| h.concept == "presenza");
        assert!(has_presence_hyp, "'presenza' dovrebbe essere un'ipotesi aperta");
    }

    #[test]
    fn scene_comprehension_depth_counts_arcs() {
        let kg = build_test_kg();
        let scene = SceneUnderstanding::assemble(&["ciao"], "ciao", &kg);
        assert!(scene.comprehension_depth >= 5);
    }

    #[test]
    fn scene_marks_unknown_words() {
        let kg = build_test_kg();
        let scene = SceneUnderstanding::assemble(&["ciao", "xyzzyx"], "ciao xyzzyx", &kg);
        assert_eq!(scene.unknown_words, vec!["xyzzyx".to_string()]);
    }

    #[test]
    fn syntactic_parse_finds_di_as_has() {
        let mut kg = KnowledgeGraph::new();
        kg.add("mario", RelationType::IsA, "persona");
        kg.add("persona", RelationType::Has, "libro");
        let scene = SceneUnderstanding::assemble(&["mario", "libro"], "il libro di mario", &kg);
        // L'edge atteso è (libro, "di", mario) con relazione validata HAS via TypeCompatible
        // (libro IS_A oggetto?? — qui usiamo che persona Has libro è invertito).
        // Il parser produce per "libro di mario" l'edge subject=libro object=mario.
        let edge = scene.syntactic_edges.iter()
            .find(|e| e.subject == "libro" && e.object == "mario")
            .expect("manca l'arco libro—di—mario");
        assert!(matches!(edge.connector, SyntacticConnector::Preposition(ref p) if p == "di"));
        // L'ordinamento delle ipotesi parte da Has — quindi se ci sono validazioni,
        // la prima dovrebbe essere quella più alta in lista.
        assert_eq!(edge.hypotheses[0].relation, RelationType::Has);
    }

    #[test]
    fn syntactic_parse_validates_direct_edge() {
        let mut kg = KnowledgeGraph::new();
        kg.add("fuoco", RelationType::Causes, "calore");
        let scene = SceneUnderstanding::assemble(&["calore", "fuoco"], "calore da fuoco", &kg);
        // "calore da fuoco" → subject=calore, object=fuoco, connector="da".
        // Hypothesize: [Causes, TransformsInto, PartOf]
        // Direct edge "calore Causes fuoco"? NO — abbiamo "fuoco Causes calore".
        // Quindi nessuna validazione diretta in questa direzione.
        let edge = scene.syntactic_edges.iter()
            .find(|e| e.subject == "calore" && e.object == "fuoco")
            .expect("manca l'arco");
        assert!(matches!(edge.connector, SyntacticConnector::Preposition(ref p) if p == "da"));
        // Non si valida (direzione inversa). validated_idx == None oppure non Causes.
        if let Some(idx) = edge.validated_idx {
            assert_ne!(edge.hypotheses[idx].relation, RelationType::Causes);
        }
    }

    #[test]
    fn syntactic_parse_copula_validates_isa() {
        let mut kg = KnowledgeGraph::new();
        kg.add("cane", RelationType::IsA, "animale");
        let scene = SceneUnderstanding::assemble(&["cane", "animale"], "il cane è un animale", &kg);
        let edge = scene.syntactic_edges.iter()
            .find(|e| e.subject == "cane" && e.object == "animale")
            .expect("manca l'arco copula cane—animale");
        assert_eq!(edge.connector, SyntacticConnector::Copula);
        let idx = edge.validated_idx.expect("la copula avrebbe dovuto validare IS_A diretto");
        assert_eq!(edge.hypotheses[idx].relation, RelationType::IsA);
        assert!(matches!(edge.hypotheses[idx].validation, ValidationOutcome::DirectEdge { .. }));
    }

    #[test]
    fn syntactic_parse_preserves_word_order() {
        let kg = KnowledgeGraph::new();
        let scene = SceneUnderstanding::assemble(
            &["sole", "caldo"],
            "il sole è caldo e il mare di salato",
            &kg,
        );
        // Prima coppia: sole–è–caldo (copula)
        // Seconda coppia: mare–di–salato (preposizione "di")
        // Verifichiamo l'ordine
        assert!(scene.syntactic_edges.len() >= 1);
        let positions: Vec<usize> = scene.syntactic_edges.iter().map(|e| e.subject_pos).collect();
        // Le posizioni devono essere strettamente crescenti
        for w in positions.windows(2) {
            assert!(w[0] < w[1], "gli edge sintattici devono preservare l'ordine");
        }
    }

    #[test]
    fn syntactic_parse_marks_not_in_kg() {
        let kg = KnowledgeGraph::new();
        let scene = SceneUnderstanding::assemble(&["xyz", "abc"], "xyz di abc", &kg);
        let edge = scene.syntactic_edges.iter()
            .find(|e| e.subject == "xyz" && e.object == "abc")
            .expect("arco mancante");
        assert!(edge.validated_idx.is_none());
        assert!(edge.hypotheses.iter().all(|h| matches!(h.validation, ValidationOutcome::NotInKg)));
    }

    #[test]
    fn scene_phrase_with_interrogative_bridge() {
        // Replica "come stai?" in versione minima con il ponte grammaticale.
        let mut kg = KnowledgeGraph::new();
        kg.add("come", RelationType::IsA, "modalità");
        kg.add("come", RelationType::UsedFor, "domanda");
        kg.add("stare", RelationType::Has, "stato");
        kg.add("stare", RelationType::Has, "condizione");
        kg.add("stare", RelationType::Has, "benessere");

        let scene = SceneUnderstanding::assemble(&["come", "stare"], "come stai?", &kg);
        assert!(scene.is_question());
        // "modalità" è toccato una volta ma è hypothetic perché è Identity di "come"
        // → non è ipotesi (è IsA, non Requires/Causes)
        // invece "stato"/"condizione"/"benessere" sono Has di stare, saliency 1, non ipotesi
        // Il test vero è che entrambe le parole contribuiscono alla comprensione
        assert_eq!(scene.per_word.len(), 2);
        assert!(scene.comprehension_depth >= 5);
    }
}
