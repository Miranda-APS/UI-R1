//! Phase 86 — Stadio 3: il collasso del cammino in frase italiana articolata.
//!
//! > Design: `docs/raw/architettura/comprensione_esplorativa_design.md` §3.7.
//!
//! Modulo **additivo e ispezionabile** (gemello di [`comprehension_path`]): NON
//! tocca `compose`/nuclei. Prende il [`ComprehensionGraph`] (Stadio 1) e ne
//! collassa il cammino saliente in italiano, articolato con le preposizioni
//! della mappa rel→prep (la metà-OUT del ponte la cui metà-IN è [`prepositions`]).
//! Serve a MISURARE prima di cablare in `compose` (principio "uno alla volta,
//! misurando, reversibile").
//!
//! ## La linea da non superare (design §8)
//!
//! Una *realizzazione grammaticale* (`Requires` → "ha bisogno di") **non è un
//! template sull'output**: è una regola di verbalizzazione *composta coi nodi
//! reali del cammino* (come `grammar::conjugate`). Resta dalla parte della lente
//! finché (a) è lessicalizzata dai nodi del cammino, non canned, e (b) la scelta
//! fra realizzazioni valide è uno **spareggio** colorato dal campo — l'unico
//! numero legittimo dell'output. L'antidoto alla rigidità è la *ricchezza* di
//! realizzazioni (più verbi per relazione), non un frame unico.
//!
//! Tabella in Rust come *seed* (come `prepositions.rs`): è grammatica ("fisica
//! del mondo"), non trigger comportamentale. Potrà migrare nel kg_proc.

use crate::topology::comprehension_path::{ComprehensionGraph, GroundKind, TypedPath};
use crate::topology::grammar::{
    conjugate, with_articulated_preposition, with_definite_article, with_indefinite_article,
    Person, Tense,
};
use crate::topology::relation::RelationType;

/// Realizzazione grammaticale di una relazione (grammatica-come-dato, seed Rust
/// migratabile al kg_proc). Ogni voce: `(verbo 3ª sing. presente SENZA la prep,
/// preposizione del complemento)`. `prep=None` → oggetto diretto (articolo
/// determinativo); `prep=Some(p)` → complemento introdotto da `p` (articolato:
/// "ha bisogno" + di → "ha bisogno della fiducia"). `prep=Some("come")` non si
/// articola ("si vive come restrizione"). Più realizzazioni per relazione =
/// ricchezza, antidoto alla rigidità (design §3.7).
fn realizations(rel: RelationType) -> &'static [(&'static str, Option<&'static str>)] {
    use RelationType::*;
    match rel {
        IsA            => &[("è", None)], // copula: oggetto con articolo indeterminativo
        Has            => &[("ha", None)],
        Does           => &[("compie", None)], // gestito a parte: l'oggetto è un verbo
        PartOf         => &[("fa parte", Some("di")), ("è parte", Some("di"))],
        Causes         => &[("causa", None), ("genera", None), ("porta", Some("a")), ("provoca", None)],
        Enables        => &[("permette", None), ("rende possibile", None)],
        Requires       => &[("ha bisogno", Some("di")), ("richiede", None), ("presuppone", None)],
        TransformsInto => &[("diventa", None), ("si trasforma", Some("in"))],
        SimilarTo      => &[("somiglia", Some("a"))],
        OppositeOf     => &[("è l'opposto", Some("di")), ("contrasta", Some("con"))],
        UsedFor        => &[("serve", Some("a")), ("si usa", Some("per"))],
        Expresses      => &[("esprime", None)],
        Symbolizes     => &[("simboleggia", None)],
        ContextOf      => &[("riguarda", None), ("fa da cornice", Some("a"))],
        FeelsAs        => &[("si vive", Some("come")), ("è vissuto", Some("come"))],
        WondersAbout   => &[("si interroga", Some("su"))],
        RemembersAs    => &[("ricorda", None)],
        Implies        => &[("implica", None)],
        Equivalent     => &[("equivale", Some("a"))],
        Excludes       => &[("esclude", None)],
        Coexists       => &[("convive", Some("con"))],
        DerivesFrom    => &[("deriva", Some("da"))],
    }
}

/// Sceglie la realizzazione di una relazione. Lo *spareggio* fra realizzazioni
/// ugualmente valide è il solo numero legittimo qui (design §3.7) e sarà colorato
/// dal campo (`valence_weight`) quando il collasso entra in `compose`. Per ora
/// deterministico: la prima (la più piana). `idx` permette di variare in test/futuro.
fn pick_realization(rel: RelationType, idx: usize) -> (&'static str, Option<&'static str>) {
    let r = realizations(rel);
    r[idx % r.len()]
}

/// Articola un singolo legame `soggetto REL oggetto` (polarità inclusa) in una
/// clausola italiana. È l'unità del collasso. `subject`/`object` sono nodi reali
/// del cammino; la frase si *compone* con loro, non è canned.
pub fn render_relation(subject: &str, rel: RelationType, object: &str, polarity: bool) -> String {
    let subj = with_definite_article(subject);
    let neg = if polarity { "" } else { "non " };

    // IsA: copula. Predicato NOMINALE → articolo indeterminativo ("la paura è
    // un'emozione"); predicato AGGETTIVALE/PARTICIPIALE → niente articolo ("il
    // padre è morto", "la libertà è sopravvalutata"). La distinzione è morfologica
    // (grammatica-come-dato, seed migratabile al kg_proc), non un dizionario.
    if rel == RelationType::IsA {
        if is_adjectival_predicate(object) {
            return format!("{} {}è {}", capitalize(&subj), neg, object);
        }
        return format!("{} {}è {}", capitalize(&subj), neg, with_indefinite_article(object));
    }
    // Does: l'oggetto è un VERBO → si coniuga alla 3ª sing.; nessun articolo.
    // ("il tradimento rompe" — il paziente, se c'è, arriva dal via, vedi render_claim).
    if rel == RelationType::Does {
        let v = conjugate(object, Person::Third, Tense::Present);
        return format!("{} {}{}", capitalize(&subj), neg, v);
    }

    let (verb, prep) = pick_realization(rel, 0);
    let comp = match prep {
        Some("come") => format!("come {object}"), // il paragone non si articola
        Some(p) => with_articulated_preposition(p, object),
        None => with_definite_article(object),
    };
    format!("{} {}{} {}", capitalize(&subj), neg, verb, comp)
}

/// Collassa il *claim* della frase (soggetto-Mondo REL oggetto [via]) — il
/// cammino saliente per eccellenza. Restituisce `None` se il soggetto non è del
/// Mondo (Speaker/Entity = territorio di `confront_with_self`, Stadio 4) o manca
/// l'oggetto: onesto, non questo stadio.
pub fn render_claim(g: &ComprehensionGraph) -> Option<String> {
    let root = g.root.as_deref()?;
    let neg = if g.polarity { "" } else { "non " };
    let subj = capitalize(&with_definite_article(root));

    // Does: il VERBO di superficie compreso ("uccidere", "iniziare") realizza
    // l'azione — non il generico "compie", né l'oggetto coniugato per sbaglio.
    // Quando il `verb_lemma` è presente (estrazione del Mondo), il `target` è il
    // PAZIENTE (oggetto diretto): "il tradimento uccide la fiducia". Senza
    // verb_lemma (shape legacy "Does rompere via=fiducia"), l'oggetto È il verbo
    // e il paziente è il via. Intransitivo (nessun paziente) → "Il giorno inizia."
    if g.relation == RelationType::Does {
        let (verb, patient) = match &g.verb_lemma {
            Some(vl) => (vl.clone(), g.target.clone()),       // verbo esplicito; target=paziente
            None => (g.target.clone()?, g.via.clone()),       // legacy: oggetto=verbo, via=paziente
        };
        let v = conjugate(&verb, Person::Third, Tense::Present);
        let tail = match patient {
            Some(p) => format!(" {}", with_definite_article(&p)),
            None => String::new(),
        };
        return Some(end(format!("{subj} {neg}{v}{tail}")));
    }

    let target = g.target.as_deref()?;
    let mut s = render_relation(root, g.relation, target, g.polarity);
    // Il via come tramite, per le relazioni causali/trasformative (dove è genuino).
    if let Some(via) = &g.via {
        if matches!(
            g.relation,
            RelationType::Causes | RelationType::TransformsInto | RelationType::Requires | RelationType::Enables
        ) {
            s.push_str(&format!(" attraverso {}", with_definite_article(via)));
        }
    }
    Some(end(s))
}

/// Collassa un cammino multi-hop in una catena di clausole (design §3.7:
/// `A —R1→ B —R2·via=C→ D`). Ogni hop si verbalizza con la sua realizzazione e i
/// nodi reali; le clausole si concatenano con la virgola (attribuzione). `forward`
/// dello step dice il senso dell'arco percorso: `false` = arco entrante, la
/// relazione lega `to → prev` (non `prev → to`).
pub fn render_path(path: &TypedPath) -> Option<String> {
    if path.steps.is_empty() {
        return None;
    }
    let mut clauses: Vec<String> = Vec::new();
    let mut prev = path.from.clone();
    for step in &path.steps {
        let clause = if step.forward {
            render_relation(&prev, step.relation, &step.to, true)
        } else {
            // arco entrante: l'edge reale è `to --rel--> prev`
            render_relation(&step.to, step.relation, &prev, true)
        };
        clauses.push(clause);
        prev = step.to.clone();
    }
    // Decapitalizza le clausole successive alla prima (sono continuazioni).
    let joined = clauses
        .iter()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.clone() } else { decapitalize(c) })
        .collect::<Vec<_>>()
        .join(", ");
    Some(end(joined))
}

/// Il punto d'ingresso: dal grafo di comprensione alla frase saliente.
///
/// Priorità (design §3.7 "appropriato nel dire"): il *claim* (la relazione
/// asserita) se c'è ed è del Mondo; altrimenti il cammino di grounding più
/// saliente (più corto, fondato su sé/attrattore). Se nulla è dicibile → `None`
/// (gap onesto, non un'insalata).
pub fn collapse(g: &ComprehensionGraph) -> Option<String> {
    if let Some(s) = render_claim(g) {
        return Some(s);
    }
    // Nessun claim del Mondo: prova il cammino di grounding più saliente.
    salient_grounding(g).and_then(render_path)
}

/// Collassa SOLO il cammino di grounding più saliente, saltando il claim.
/// È la voce della posizione su un claim del Mondo che l'entità NON conosce
/// (`KgConfrontation.matches == false`): ricalcarlo sarebbe assenso per eco —
/// la posizione onesta è ciò che il SUO grafo sostiene sul tema (il cammino
/// deformato dalla grana: la preferenza premia i nodi-pendenza del sé).
/// `None` se nulla è dicibile (gap onesto → fallback del caller).
pub fn collapse_grounding(g: &ComprehensionGraph) -> Option<String> {
    salient_grounding(g).and_then(render_path)
}

/// Collasso strutturale di un claim dello SPEAKER (l'utente parla di sé), guidato
/// dall'ATTO (dal bisogno, via kg_proc: `bisogno UsedFor atto via=locus`) e
/// dall'INTERROGATIVO del locus (`locus UsedFor chiedere via=parola`). È la
/// macchina GENERALE comprensione→posizione→output: non conosce intenti
/// specifici, applica modi.
///
/// DEISSI (shifter): nella voce di UI-r1 verso l'utente, l'«io» dell'Altro è
/// «tu» → il verbo di superficie si coniuga alla persona spostata. Riusa la
/// grammatica-come-dato (`grammar::conjugate`), zero template per-intento.
///
/// `act`: "chiedere" (gap) | "esplorare" (causa) | "confermare" (specchio).
/// `interrog`: la parola interrogativa del locus, dal kg_proc (cosa/perché/come).
pub fn collapse_speaker(
    prop: &crate::topology::sentence_proposition::SentenceProposition,
    act: &str,
    interrog: Option<&str>,
    obj_display: Option<&str>,
) -> Option<String> {
    use crate::topology::sentence_proposition::{ObjectRef, SubjectRef};
    use crate::topology::grammar::{conjugate, Person, Tense, with_articulated_preposition};

    // Solo claim dello Speaker/Entity: il Mondo è territorio di `collapse`.
    if matches!(prop.subject, SubjectRef::World(_) | SubjectRef::Variable(_)) {
        return None;
    }
    let obj = match &prop.object {
        Some(ObjectRef::Word(w)) => w.clone(),
        _ => return None,
    };
    // Serve il VERBO di superficie reale (avere/essere/sentire/volere…): è ciò
    // che coniughiamo. Senza, lasciamo il fallback al caller (gap onesto).
    let verb_lemma = prop.verb_lemma.as_deref()?;

    // Deissi: «io»→«tu» (2ª sing.), «noi»→«voi» (2ª pl.), «tu»→«io» (1ª sing.).
    let person = match prop.subject_surface.as_deref().map(|s| s.to_lowercase()).as_deref() {
        Some("noi") | Some("voi") => Person::SecondPlural,
        Some("tu") => Person::First,
        _ => Person::Second, // default: l'Altro è «tu»
    };
    let verb = conjugate(verb_lemma, person, Tense::Present);
    let neg = if prop.polarity { "" } else { "non " };

    // Per un `Does` INTRANSITIVO l'oggetto È il verbo stesso ("devo studiare" →
    // Does studiare, obj=studiare): il verbo coniugato lo realizza già, NON lo
    // ripetiamo ("Perché studi?", non "Perché studii studiare?"). Transitivo
    // (obj≠verbo, "voglio cambiare vita") o altre relazioni → l'oggetto resta.
    let obj_is_verb = prop.relation == RelationType::Does && obj.eq_ignore_ascii_case(verb_lemma);
    // L'oggetto con l'articolo che l'utente HA usato ("un cane", "la tesi"); se
    // l'utente non ne ha messo (stati: "ho paura"), resta nudo — corretto.
    let obj_shown = obj_display.unwrap_or(&obj);
    let obj_part = if obj_is_verb { String::new() } else { format!(" {obj_shown}") };

    // Coda del complemento (via): infinito → "di lavorare"; nome → "del futuro".
    let via_tail = prop.via.as_ref().map(|v| {
        let inf = v.ends_with("are") || v.ends_with("ere") || v.ends_with("ire");
        if inf { format!(" di {v}") } else { format!(" {}", with_articulated_preposition("di", v)) }
    }).unwrap_or_default();

    let text = match act {
        // ESPLORARE (posizionarsi): interroga la CAUSA — "Perché non hai voglia di lavorare?"
        "esplorare" => {
            let q = interrog.unwrap_or("perché");
            format!("{} {neg}{verb}{obj_part}{via_tail}?", capitalize(q))
        }
        // CHIEDERE (capire): interroga lo slot mancante — "Di cosa hai paura?"
        "chiedere" => {
            let q = interrog.unwrap_or("cosa");
            format!("Di {q} {neg}{verb}{obj_part}?")
        }
        // CONFERMARE / specchio (default): "Hai sonno." / "Senti paura del futuro."
        _ => end(capitalize(&format!("{neg}{verb}{obj_part}{via_tail}"))),
    };
    Some(text)
}

/// Il grounding più saliente del grafo. Usa la STESSA preferenza strutturale di
/// `ground_node` (`comprehension_path::grounding_preferred`): sé > attrattore >
/// nodo-frase, poi TASSONOMICO (primo passo `IsA`: "che cos'è X" — Quillian —
/// batte "di cosa X ha bisogno", il meta-edge), poi più corto, poi confidenza.
/// Una sola sorgente di verità per lo spareggio, niente soglie.
fn salient_grounding(g: &ComprehensionGraph) -> Option<&TypedPath> {
    g.groundings
        .iter()
        .filter(|p| !p.steps.is_empty() && p.ground != GroundKind::Unreached)
        .max_by(|a, b| crate::topology::comprehension_path::grounding_preferred(a, b))
}

// ── helper di superficie ─────────────────────────────────────────────────────

fn end(mut s: String) -> String {
    let t = s.trim_end();
    if !t.ends_with(['.', '?', '!']) {
        s = format!("{t}.");
    }
    s
}

/// Il predicato di una copula è aggettivale/participiale (→ niente articolo)?
/// Morfologia, non dizionario (seed migratabile): participi regolari (-ato/-uto/
/// -ito + flessioni), suffissi aggettivali produttivi (-oso/-ale/-ile/-ico/-ivo/
/// -ante/-ente) e un piccolo nucleo di participi irregolari comuni. Conservativo:
/// nel dubbio (es. "-o" generico, che può essere nome) ritorna false → articolo.
fn is_adjectival_predicate(word: &str) -> bool {
    let w = word.to_lowercase();
    const IRREGULAR: &[&str] = &[
        "morto", "morta", "morti", "morte",
        "rotto", "rotta", "rotti", "rotte",
        "aperto", "aperta", "aperti", "aperte",
        "chiuso", "chiusa", "chiusi", "chiuse",
        "perso", "persa", "persi", "perse",
        "vivo", "viva", "vivi", "vive",
    ];
    if IRREGULAR.contains(&w.as_str()) {
        return true;
    }
    const SUFFIXES: &[&str] = &[
        "ato", "ata", "ati", "ate", // participi -are
        "uto", "uta", "uti", "ute", // participi -ere
        "ito", "ita", "iti", "ite", // participi -ire
        "oso", "osa", "osi", "ose", // -oso
        "ale", "ali", "ile", "ili", // -ale/-ile
        "ico", "ica", "ici", "iche", // -ico
        "ivo", "iva", "ivi", "ive", // -ivo
        "ante", "anti", "ente", "enti", // participi presenti / -ante·-ente
    ];
    // stem ≥3 esclude monosillabi ambigui ("stato"→st, già escluso altrove).
    SUFFIXES.iter().any(|suf| w.strip_suffix(suf).map(|s| s.len() >= 3).unwrap_or(false))
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn decapitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::comprehension_path::{Confront, PathStep};

    fn graph(
        root: Option<&str>,
        rel: RelationType,
        target: Option<&str>,
        via: Option<&str>,
        polarity: bool,
    ) -> ComprehensionGraph {
        ComprehensionGraph {
            root: root.map(|s| s.to_string()),
            relation: rel,
            target: target.map(|s| s.to_string()),
            via: via.map(|s| s.to_string()),
            verb_lemma: None,
            polarity,
            claim_path: None,
            confront: Confront::NotApplicable,
            groundings: vec![],
            ungrounded: vec![],
        }
    }

    /// Variante con verb_lemma esplicito, per il collasso di un `Does` del Mondo
    /// che porta il verbo di superficie (es. "uccidere") oltre al paziente.
    fn graph_with_verb(
        root: &str, target: Option<&str>, via: Option<&str>, verb: &str, polarity: bool,
    ) -> ComprehensionGraph {
        ComprehensionGraph {
            root: Some(root.to_string()),
            relation: RelationType::Does,
            target: target.map(|s| s.to_string()),
            via: via.map(|s| s.to_string()),
            verb_lemma: Some(verb.to_string()),
            polarity,
            claim_path: None,
            confront: Confront::NotApplicable,
            groundings: vec![],
            ungrounded: vec![],
        }
    }

    #[test]
    fn collasso_requires_articolato() {
        // La verifica-target del design §9.3.
        let g = graph(Some("tradimento"), RelationType::Requires, Some("fiducia"), None, true);
        assert_eq!(collapse(&g).as_deref(), Some("Il tradimento ha bisogno della fiducia."));
    }

    #[test]
    fn collasso_isa_articolo_indeterminativo() {
        let g = graph(Some("paura"), RelationType::IsA, Some("emozione"), None, true);
        assert_eq!(collapse(&g).as_deref(), Some("La paura è un'emozione."));
    }

    #[test]
    fn collasso_causes_con_via() {
        let g = graph(Some("separazione"), RelationType::Causes, Some("dolore"), Some("perdita"), true);
        assert_eq!(
            collapse(&g).as_deref(),
            Some("La separazione causa il dolore attraverso la perdita.")
        );
    }

    #[test]
    fn collasso_does_con_paziente() {
        // "tradimento Does rompere via=fiducia" → l'oggetto-verbo regge il paziente.
        let g = graph(Some("tradimento"), RelationType::Does, Some("rompere"), Some("fiducia"), true);
        assert_eq!(collapse(&g).as_deref(), Some("Il tradimento rompe la fiducia."));
    }

    #[test]
    fn collasso_does_realizza_verbo_di_superficie() {
        // Shape estrazione-Mondo: verb_lemma="uccidere", target=PAZIENTE (fiducia).
        // Il verbo concreto si realizza — non il generico "compie".
        let g = graph_with_verb("tradimento", Some("fiducia"), None, "uccidere", true);
        assert_eq!(collapse(&g).as_deref(), Some("Il tradimento uccide la fiducia."));
    }

    #[test]
    fn collasso_does_intransitivo_senza_paziente() {
        // "il giorno inizia (dal mattino)" → niente oggetto diretto → solo il verbo.
        let g = graph_with_verb("giorno", None, Some("mattino"), "iniziare", true);
        assert_eq!(collapse(&g).as_deref(), Some("Il giorno inizia."));
    }

    #[test]
    fn polarita_negativa_inserisce_non() {
        let g = graph(Some("pensiero"), RelationType::IsA, Some("calcolo"), None, false);
        assert_eq!(collapse(&g).as_deref(), Some("Il pensiero non è un calcolo."));
    }

    #[test]
    fn soggetto_non_mondo_non_collassa_qui() {
        // Speaker/Entity → root=None → render_claim None (territorio confront_with_self).
        let g = graph(None, RelationType::FeelsAs, Some("paura"), None, true);
        assert!(render_claim(&g).is_none());
        assert!(collapse(&g).is_none(), "senza claim né grounding non c'è nulla da dire (gap onesto)");
    }

    #[test]
    fn cammino_multi_hop_si_concatena() {
        // tradimento --Requires--> fiducia (fwd), poi fiducia <--Has-- relazione (inv)
        let path = TypedPath {
            from: "tradimento".into(),
            steps: vec![
                PathStep { relation: RelationType::Requires, forward: true, via: None, to: "fiducia".into(), confidence: 0.9 },
                PathStep { relation: RelationType::Has, forward: false, via: None, to: "relazione".into(), confidence: 0.8 },
            ],
            ground: GroundKind::Attractor,
        };
        // hop1: "Il tradimento ha bisogno della fiducia"; hop2 entrante: "la relazione ha la fiducia"
        let s = render_path(&path).unwrap();
        assert!(s.starts_with("Il tradimento ha bisogno della fiducia,"), "got: {s}");
        assert!(s.contains("la relazione ha la fiducia"), "got: {s}");
    }
}
