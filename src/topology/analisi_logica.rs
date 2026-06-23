//! Phase 86+ — Analisi logica: il chunker generico guidato dalla grammatica del
//! kg_proc. Design: `docs/raw/architettura/analisi_logica_grammatica_kg_proc.md`.
//!
//! Obiettivo: ogni parola dell'input riceve una FUNZIONE (o resta nel residuo,
//! misurabile). La mappa classe→funzione è DATO nel kg_proc (`aggettivo UsedFor
//! attributo`, `attributo Requires nome`, …); qui vive solo il MECCANISMO
//! generico — niente regole frase-specifiche (sarebbe un parser hardcoded).
//!
//! Primo mattone: **`attributo`**. Le classi aperte (nome/aggettivo) non sono
//! enumerabili → la testa-nome e i suoi modificatori si riconoscono per
//! STRUTTURA del gruppo nominale, non per dizionario: le parole-funzione
//! (articolo, copula, preposizione…) spezzano i gruppi; dentro un gruppo, un
//! content-word che segue un altro content-word è un attributo della testa
//! ("un caffè *freddo*", "l'anno *scorso*", "una bicicletta *nuova*"). La copula
//! spezza, quindi "è morto" NON dà attributo (morto è predicato).

use std::collections::HashSet;

use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

/// Un token "porta contenuto" (candidato nome/aggettivo): alfabetico, non
/// parola-funzione (classe chiusa dal kg_proc), non verbo di modo finito
/// (presente — il predicato). I participi NON sono esclusi: in posizione
/// attributiva sono attributi ("una porta *chiusa*"), e l'adiacenza con la
/// copula li separa quando sono predicati.
fn is_content(token: &str, kg_proc: &KnowledgeGraph, kg: &KnowledgeGraph) -> bool {
    let w = token.to_lowercase();
    if !w.chars().any(|c| c.is_alphabetic()) {
        return false;
    }
    if crate::topology::input_reading::is_kg_proc_function_word(&w, Some(kg_proc)) {
        return false;
    }
    // Verbo di modo finito presente → predicato, non contenuto nominale.
    crate::topology::input_reading::lemma_of_verb(token, Some(kg_proc), Some(kg)).is_none()
}

/// Il kg_proc dichiara la funzione `attributo` (dato)? Gate generico: senza il
/// dato, il meccanismo non assegna nulla (la grammatica vive nel kg_proc).
fn attributo_is_declared(kg_proc: &KnowledgeGraph) -> bool {
    kg_proc
        .query_objects("aggettivo", RelationType::UsedFor)
        .iter()
        .any(|o| o.eq_ignore_ascii_case("attributo"))
}

/// Indici dei token che ricoprono la funzione `attributo`: un content-word
/// preceduto, nel flusso, da un altro content-word (= modificatore della testa
/// nominale che lo precede). Meccanismo generico, nessuna parola hardcoded.
pub fn attributo_indices(
    raw_words: &[String],
    kg_proc: &KnowledgeGraph,
    kg: &KnowledgeGraph,
) -> HashSet<usize> {
    let mut out = HashSet::new();
    if !attributo_is_declared(kg_proc) {
        return out;
    }
    let content: Vec<bool> = raw_words
        .iter()
        .map(|w| is_content(w, kg_proc, kg))
        .collect();
    for i in 1..content.len() {
        if content[i] && content[i - 1] {
            out.insert(i);
        }
    }
    out
}

/// Indici dei token che ricoprono la funzione `circostanza`: gli AVVERBI
/// (classe `IsA avverbio` nel kg_proc — semi-chiusa, curata in §H/§H.ter) che il
/// dato dichiara `avverbio UsedFor circostanza`. Ruolo reale (modificatore del
/// verbo: tempo/grado/modo), non "parola-funzione da saltare". Gate sul dato.
pub fn circostanza_indices(
    raw_words: &[String],
    kg_proc: &KnowledgeGraph,
) -> HashSet<usize> {
    let mut out = HashSet::new();
    let declared = kg_proc
        .query_objects("avverbio", RelationType::UsedFor)
        .iter()
        .any(|o| o.eq_ignore_ascii_case("circostanza"));
    if !declared {
        return out;
    }
    for (i, w) in raw_words.iter().enumerate() {
        let is_avv = kg_proc
            .query_objects(&w.to_lowercase(), RelationType::IsA)
            .iter()
            .any(|o| o.eq_ignore_ascii_case("avverbio"));
        if is_avv {
            out.insert(i);
        }
    }
    out
}

// ═══════════════════════════════════════════════════════════════════════════
// Chunker clausa-aware (Phase 86+) — design:
//   docs/raw/architettura/chunker_clausa_aware_design.md
//
// Un enunciato è una o più CLAUSOLE; in ogni clausola ogni token riceve una
// FUNZIONE (o resta nel residuo, misurabile). La mappa classe→funzione è DATO
// nel kg_proc (§J: `aggettivo UsedFor attributo`, `nome UsedFor argomento`,
// `preposizione UsedFor complemento`, …); qui vive solo il MECCANISMO generico —
// niente regole frase-specifiche. La lemmatizzazione, la PROP e il bisogno sono
// CONSEGUENZE dei ruoli; questo primo strato OSSERVA (visibile nel bench), non
// tocca ancora PROP/OUT (additivo e reversibile).
// ═══════════════════════════════════════════════════════════════════════════

use crate::topology::grammar;
use crate::topology::input_reading::lemma_of_verb;

/// La funzione sintattica che un token ricopre in una clausola.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Funzione {
    /// Gruppo verbale (ausiliare/copula + verbo/participio): il nucleo della clausola.
    Predicato,
    /// Testa nominale (soggetto/oggetto): il contenuto di un gruppo nominale nudo.
    Argomento,
    /// Aggettivo/participio attributivo adiacente alla testa nominale.
    Attributo,
    /// Articolo / determinante: determina un nome senza esserne il contenuto.
    Determinazione,
    /// Gruppo preposizionale (la preposizione + il suo contenuto).
    Complemento,
    /// Avverbio (modificatore del verbo: tempo/grado/modo).
    Circostanza,
    /// Congiunzione/subordinante: lega clausole o argomenti.
    Connettivo,
    /// Marcatore o interiezione: atto espressivo a sé ("secondo me", "boh").
    Marcatore,
    /// Nessun ruolo assegnato — la spia (numero da portare a zero).
    Residuo,
}

impl Funzione {
    /// Sigla compatta per l'osservabilità nel bench.
    pub fn sigla(&self) -> &'static str {
        match self {
            Funzione::Predicato      => "Pred",
            Funzione::Argomento      => "Arg",
            Funzione::Attributo      => "Attr",
            Funzione::Determinazione => "Det",
            Funzione::Complemento    => "Compl",
            Funzione::Circostanza    => "Circ",
            Funzione::Connettivo     => "Conn",
            Funzione::Marcatore      => "Marc",
            Funzione::Residuo        => "·",
        }
    }
}

/// Una clausola = un predicato + i suoi argomenti/circostanze. Conosce il proprio
/// intervallo di token e il lemma del verbo che la regge (per la PROP futura).
#[derive(Debug, Clone)]
pub struct Clausola {
    /// Intervallo di token [start, end) nell'enunciato.
    pub range: std::ops::Range<usize>,
    /// L'infinito del verbo che regge la clausola (None se verbless).
    pub predicato_lemma: Option<String>,
    /// La clausola è SUBORDINATA — introdotta da un connettivo subordinante
    /// (temporale/causale/concessiva/…: "da quando…", "perché…"). Le coordinate
    /// (e/o/ma) e la prima clausola sono indipendenti (`false`). La PROP primaria
    /// si sceglie fra le indipendenti.
    pub subordinate: bool,
}

/// L'analisi logica di un enunciato: la funzione di OGNI token + le clausole.
#[derive(Debug, Clone)]
pub struct Analisi {
    /// Una funzione per ogni token dell'enunciato (allineata a `raw_words`).
    pub funzioni: Vec<Funzione>,
    /// Le clausole in cui l'enunciato è segmentato.
    pub clausole: Vec<Clausola>,
}

/// `token IsA classe` (1 hop) nel kg_proc.
fn isa(token: &str, classe: &str, kg_proc: &KnowledgeGraph) -> bool {
    kg_proc
        .query_objects(&token.to_lowercase(), RelationType::IsA)
        .iter()
        .any(|p| p.eq_ignore_ascii_case(classe))
}

/// Il connettivo è SUBORDINANTE? Letto dal kg_proc come DATO (`<conj> UsedFor
/// legare via=<ruolo>`): i ruoli subordinanti (temporale/causale/concessiva/…)
/// aprono una clausola dipendente; i coordinanti (additiva/avversativa/…) no.
fn is_subordinante(token: &str, kg_proc: &KnowledgeGraph) -> bool {
    const SUB_ROLES: &[&str] = &[
        "ipotetica", "causale", "subordinante", "temporale",
        "concessiva", "finale", "condizionale", "comparativa",
    ];
    kg_proc
        .query_objects_with_via(&token.to_lowercase(), RelationType::UsedFor)
        .iter()
        .any(|(obj, _, via)| {
            obj.eq_ignore_ascii_case("legare")
                && via
                    .map(|v| SUB_ROLES.contains(&v.to_lowercase().as_str()))
                    .unwrap_or(false)
        })
}

/// La funzione che una classe grammaticale ricopre, letta dal kg_proc come DATO
/// (§J: `classe UsedFor funzione`). Curare la grammatica = modificare il JSON,
/// mai il Rust (Principio 6). `None` se la classe non dichiara una funzione: il
/// token cadrà nel residuo, la spia. Il *meccanismo* di attacco (testa vs
/// modificatore, gruppo preposizionale) resta generico nel walk — quella è
/// sintassi universale, non una regola frase-specifica.
fn funzione_di_classe(classe: &str, kg_proc: &KnowledgeGraph) -> Option<Funzione> {
    for o in kg_proc.query_objects(classe, RelationType::UsedFor) {
        let f = match o.to_lowercase().as_str() {
            "attributo" => Funzione::Attributo,
            "argomento" => Funzione::Argomento,
            "complemento" => Funzione::Complemento,
            "circostanza" => Funzione::Circostanza,
            "determinazione" => Funzione::Determinazione,
            "connessione" => Funzione::Connettivo,
            _ => continue,
        };
        return Some(f);
    }
    None
}

/// L'infinito di un participio passato: irregolari come dato (`grammar`), poi i
/// suffissi regolari (-ato→are, -uto→ere, -ito→ire). `None` se non è participio.
fn participle_lemma(w: &str) -> Option<String> {
    let lw = w.to_lowercase();
    if let Some(inf) = grammar::irregular_participle(&lw) {
        return Some(inf);
    }
    for (suf, inf_suf) in [
        ("ato", "are"), ("ata", "are"), ("ati", "are"), ("ate", "are"),
        ("uto", "ere"), ("uta", "ere"), ("uti", "ere"), ("ute", "ere"),
        ("ito", "ire"), ("ita", "ire"), ("iti", "ire"), ("ite", "ire"),
    ] {
        if let Some(stem) = lw.strip_suffix(suf) {
            if stem.chars().count() >= 3 {
                return Some(format!("{stem}{inf_suf}"));
            }
        }
    }
    None
}

/// Un gruppo verbale: i token che compongono il predicato + il suo infinito.
struct GruppoVerbale {
    tokens: Vec<usize>,
    lemma: String,
}

/// La classe chiusa NON-verbale del token (articolo/determinante/preposizione/
/// congiunzione/pronome/avverbio/marcatore/interiezione), se ne ha una. Copula e
/// ausiliare sono ESCLUSE di proposito: sono verbi, non si saltano. Le forme
/// coniugate (ho/è/sono) non sono nodi del kg_proc → non matchano qui, cadono
/// correttamente al controllo verbale.
fn classe_chiusa_non_verbale(w: &str, kg_proc: &KnowledgeGraph) -> Option<&'static str> {
    const CLASSI: &[&str] = &[
        "congiunzione", "preposizione", "articolo", "determinante",
        "pronome", "avverbio", "marcatore", "interiezione",
    ];
    CLASSI.iter().copied().find(|c| isa(w, c, kg_proc))
}

/// Il participio passato subito dopo `from` (saltando clitici/avverbi, finestra
/// breve), col suo infinito. Per i tempi composti "ho lavorato"/"me ne sono andato".
fn participio_dopo(
    raw: &[String],
    aux_pos: usize,
    kg_proc: &KnowledgeGraph,
) -> Option<(usize, String)> {
    let n = raw.len();
    let mut j = aux_pos + 1;
    while j < n && j <= aux_pos + 3 {
        let wj = raw[j].to_lowercase();
        if let Some(lemma) = participle_lemma(&wj) {
            return Some((j, lemma));
        }
        if isa(&wj, "pronome", kg_proc) || isa(&wj, "avverbio", kg_proc) {
            j += 1;
            continue;
        }
        break;
    }
    None
}

/// Trova i gruppi verbali (i nuclei delle clausole) con disambiguazione
/// CONTESTUALE nome-vs-verbo (mai morfologica cieca, [[feedback-no-tricks…]]).
/// Scansione unica con due stati:
///   - `expect_noun`: dopo un articolo/determinante la testa è un NOME, non un
///     verbo (così "una"/"il dolore" non diventano `unire`/`dolorare`);
///   - `verb_in_run`: un secondo verbo finito "nudo" (senza un connettivo che lo
///     separi dal primo) è un nome mal-lemmatizzato — in italiano due predicati
///     richiedono un connettivo, salvo ausiliare+participio e modale+infinito,
///     che si raggruppano. Il connettivo azzera `verb_in_run`.
fn gruppi_verbali(
    raw: &[String],
    kg_proc: &KnowledgeGraph,
    kg: &KnowledgeGraph,
) -> Vec<GruppoVerbale> {
    let mut groups = Vec::new();
    let n = raw.len();
    let mut i = 0;
    let mut verb_in_run = false;
    let mut expect_noun = false;
    while i < n {
        let w = raw[i].to_lowercase();
        if let Some(c) = classe_chiusa_non_verbale(&w, kg_proc) {
            match c {
                "congiunzione" => { verb_in_run = false; expect_noun = false; }
                // un determinante introduce la sua testa nominale (mai un verbo)
                "articolo" | "determinante" => { expect_noun = true; }
                _ => {}
            }
            i += 1;
            continue;
        }
        if let Some((inf, _)) = lemma_of_verb(&raw[i], Some(kg_proc), Some(kg)) {
            // testa attesa dopo un determinante → è il NOME, non un verbo
            if expect_noun {
                expect_noun = false;
                i += 1;
                continue;
            }
            // secondo verbo nudo senza connettivo → nome mal-lemmatizzato
            if verb_in_run {
                i += 1;
                continue;
            }
            // ausiliare/copula + participio → gruppo composto (contenuto = participio)
            if isa(&inf, "ausiliare", kg_proc) {
                if let Some((p, lemma)) = participio_dopo(raw, i, kg_proc) {
                    groups.push(GruppoVerbale { tokens: vec![i, p], lemma });
                    verb_in_run = true;
                    i = p + 1;
                    continue;
                }
                // ausiliare senza participio → copula ("sono triste", "ho un cane")
            }
            // modale + infinito → gruppo (contenuto = l'infinito, "voglio capire")
            if isa(&inf, "modale", kg_proc) {
                if let Some((p, lemma)) = infinito_dopo(raw, i, kg_proc, kg) {
                    groups.push(GruppoVerbale { tokens: vec![i, p], lemma });
                    verb_in_run = true;
                    i = p + 1;
                    continue;
                }
            }
            groups.push(GruppoVerbale { tokens: vec![i], lemma: inf });
            verb_in_run = true;
            i += 1;
        } else {
            // parola-contenuto non verbale → consuma l'attesa-nome
            expect_noun = false;
            i += 1;
        }
    }
    groups
}

/// L'infinito subito dopo un modale (saltando la preposizione "a"/"di" di un
/// catenativo), col suo lemma. Per "voglio andare", "devo capire".
fn infinito_dopo(
    raw: &[String],
    modal_pos: usize,
    kg_proc: &KnowledgeGraph,
    kg: &KnowledgeGraph,
) -> Option<(usize, String)> {
    let n = raw.len();
    let mut j = modal_pos + 1;
    while j < n && j <= modal_pos + 2 {
        let wj = raw[j].to_lowercase();
        // un infinito vero finisce in -are/-ere/-ire ed è un verbo
        if (wj.ends_with("are") || wj.ends_with("ere") || wj.ends_with("ire"))
            && lemma_of_verb(&raw[j], Some(kg_proc), Some(kg)).is_some()
        {
            return Some((j, wj));
        }
        if isa(&wj, "preposizione", kg_proc) {
            j += 1;
            continue;
        }
        break;
    }
    None
}

/// Segmenta l'enunciato in intervalli di clausola, uno per gruppo verbale.
/// Il confine fra due gruppi cade sul connettivo (congiunzione/subordinante) che
/// li separa — risalendo su un'eventuale preposizione di testa ("da quando").
/// Se non c'è connettivo, il confine è l'inizio del gruppo successivo.
fn clausole_ranges(
    n: usize,
    groups: &[GruppoVerbale],
    raw: &[String],
    kg_proc: &KnowledgeGraph,
) -> Vec<std::ops::Range<usize>> {
    if groups.len() <= 1 {
        return vec![0..n];
    }
    let mut boundaries = vec![0usize];
    for pair in groups.windows(2) {
        let end_prev = *pair[0].tokens.last().unwrap() + 1;
        let start_next = *pair[1].tokens.first().unwrap();
        // Cerca un connettivo nel gap (congiunzione/subordinante).
        let mut boundary = start_next;
        for k in end_prev..start_next {
            if isa(&raw[k], "congiunzione", kg_proc) {
                boundary = k;
                // risali su una preposizione di testa ("da quando")
                if k > end_prev && isa(&raw[k - 1], "preposizione", kg_proc) {
                    boundary = k - 1;
                }
                break;
            }
        }
        boundaries.push(boundary);
    }
    let mut ranges = Vec::new();
    for w in boundaries.windows(2) {
        ranges.push(w[0]..w[1]);
    }
    ranges.push(*boundaries.last().unwrap()..n);
    ranges
}

/// Analizza un enunciato: segmenta in clausole e assegna a OGNI token una
/// funzione, leggendo le classi dal kg_proc (gate sul dato) e attaccando per
/// adiacenza. Meccanismo generico, nessuna parola hardcoded.
pub fn analizza(
    raw_words: &[String],
    kg_proc: &KnowledgeGraph,
    kg: &KnowledgeGraph,
) -> Analisi {
    let n = raw_words.len();
    let mut funzioni = vec![Funzione::Residuo; n];

    let groups = gruppi_verbali(raw_words, kg_proc, kg);
    let ranges = clausole_ranges(n, &groups, raw_words, kg_proc);

    // Indici di tutti i token-predicato (per il lookup nel walk dei ruoli).
    let predicato_set: HashSet<usize> =
        groups.iter().flat_map(|g| g.tokens.iter().copied()).collect();

    let mut clausole = Vec::with_capacity(ranges.len());
    for range in ranges {
        // Il lemma del gruppo che cade in questa clausola (se c'è).
        let predicato_lemma = groups
            .iter()
            .find(|g| g.tokens.iter().any(|&t| range.contains(&t)))
            .map(|g| g.lemma.clone());
        // Subordinata se introdotta (entro i primi token) da un connettivo
        // subordinante (eventualmente preceduto da una preposizione: "da quando").
        let subordinate = range.clone().take(2)
            .any(|i| is_subordinante(&raw_words[i], kg_proc));

        // Walk dei ruoli: stato del gruppo nominale corrente.
        let mut in_prep = false;
        let mut have_head = false;
        for i in range.clone() {
            let w = raw_words[i].to_lowercase();
            if !w.chars().any(|c| c.is_alphabetic()) {
                funzioni[i] = Funzione::Residuo;
                continue;
            }
            if predicato_set.contains(&i) {
                funzioni[i] = Funzione::Predicato;
                in_prep = false;
                have_head = false;
                continue;
            }
            // Congiunzione: lega (la funzione è dato §J); reset del gruppo nominale.
            if isa(&w, "congiunzione", kg_proc) {
                funzioni[i] = funzione_di_classe("congiunzione", kg_proc)
                    .unwrap_or(Funzione::Connettivo);
                in_prep = false;
                have_head = false;
                continue;
            }
            // Marcatore/interiezione: atto a sé, non un ruolo §J (strutturale).
            if isa(&w, "marcatore", kg_proc) || isa(&w, "interiezione", kg_proc) {
                funzioni[i] = Funzione::Marcatore;
                continue;
            }
            // Preposizione: apre un gruppo preposizionale (funzione §J = complemento).
            if isa(&w, "preposizione", kg_proc) {
                funzioni[i] = funzione_di_classe("preposizione", kg_proc)
                    .unwrap_or(Funzione::Complemento);
                in_prep = true;
                have_head = false;
                continue;
            }
            // Articolo/determinante: determinazione (funzione §J).
            if isa(&w, "determinante", kg_proc) {
                if let Some(f) = funzione_di_classe("determinante", kg_proc) {
                    funzioni[i] = f;
                    continue;
                }
            }
            if isa(&w, "articolo", kg_proc) {
                if let Some(f) = funzione_di_classe("articolo", kg_proc) {
                    funzioni[i] = f;
                    continue;
                }
            }
            // Avverbio: circostanza (funzione §J).
            if isa(&w, "avverbio", kg_proc) {
                if let Some(f) = funzione_di_classe("avverbio", kg_proc) {
                    funzioni[i] = f;
                    continue;
                }
            }
            // Pronome soggetto/oggetto o clitico: vale come argomento (strutturale).
            if isa(&w, "pronome", kg_proc) {
                funzioni[i] = Funzione::Argomento;
                continue;
            }
            // Arrivati qui il token NON è né predicato né una classe chiusa →
            // è contenuto nominale (nome/aggettivo), ANCHE se mal-lemmatizzabile
            // come verbo: il contesto (non in predicato_set) lo ha già escluso dai
            // verbi. Il MECCANISMO di attacco (gruppo preposizionale, testa vs
            // modificatore) è generico; quale funzione la classe ricopre resta
            // dato §J (nome→argomento, aggettivo→attributo).
            if w.chars().any(|c| c.is_alphabetic()) {
                funzioni[i] = if in_prep {
                    Funzione::Complemento
                } else if !have_head {
                    have_head = true;
                    funzione_di_classe("nome", kg_proc).unwrap_or(Funzione::Argomento)
                } else {
                    funzione_di_classe("aggettivo", kg_proc).unwrap_or(Funzione::Attributo)
                };
                continue;
            }
            // non-alfabetico residuo: la spia.
            funzioni[i] = Funzione::Residuo;
        }
        clausole.push(Clausola { range, predicato_lemma, subordinate });
    }

    Analisi { funzioni, clausole }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::relation::{EdgeSource, TypedEdge};

    fn kg_proc() -> KnowledgeGraph {
        let mut kp = KnowledgeGraph::new();
        let mut add = |s: &str, r: RelationType, o: &str| {
            kp.add_edge(TypedEdge {
                subject: s.into(), relation: r, object: o.into(),
                confidence: 0.95, source: EdgeSource::Curated, via: None,
            });
        };
        // funzione dichiarata
        add("aggettivo", RelationType::UsedFor, "attributo");
        add("attributo", RelationType::Requires, "nome");
        // parole-funzione chiuse
        add("un", RelationType::IsA, "articolo");
        add("una", RelationType::IsA, "articolo");
        add("articolo", RelationType::IsA, "categoria");
        kp
    }

    fn words(s: &str) -> Vec<String> {
        s.split_whitespace().map(|w| w.to_string()).collect()
    }

    #[test]
    fn modificatore_dopo_la_testa_e_attributo() {
        let kp = kg_proc();
        let kg = KnowledgeGraph::new();
        // "una bicicletta nuova": un(art) bicicletta(nome) nuova(attributo)
        let idx = attributo_indices(&words("una bicicletta nuova"), &kp, &kg);
        assert!(idx.contains(&2), "nuova deve essere attributo");
        assert!(!idx.contains(&1), "bicicletta è la testa, non attributo");
    }

    #[test]
    fn la_copula_spezza_il_gruppo() {
        let mut kp = kg_proc();
        kp.add_edge(TypedEdge {
            subject: "è".into(), relation: RelationType::IsA, object: "copula".into(),
            confidence: 0.95, source: EdgeSource::Curated, via: None,
        });
        let kg = KnowledgeGraph::new();
        // "padre è morto": padre, è(copula→spezza), morto → morto NON è attributo
        let idx = attributo_indices(&words("padre è morto"), &kp, &kg);
        assert!(idx.is_empty(), "dopo la copula il predicato non è attributo");
    }

    #[test]
    fn senza_dato_nessuna_assegnazione() {
        let kp = KnowledgeGraph::new(); // funzione NON dichiarata
        let kg = KnowledgeGraph::new();
        assert!(attributo_indices(&words("una bicicletta nuova"), &kp, &kg).is_empty());
    }

    // ── chunker clausa-aware ────────────────────────────────────────────────

    /// kg_proc ricco: classi chiuse + verbi (`IsA verbo`, così `lemma_of_verb`
    /// valida) + la mappa classe→funzione §J. Niente kg_sem (verbità via §verbo).
    fn kp_full() -> KnowledgeGraph {
        let mut kp = KnowledgeGraph::new();
        let mut add = |s: &str, r: RelationType, o: &str| {
            kp.add_edge(TypedEdge {
                subject: s.into(), relation: r, object: o.into(),
                confidence: 0.95, source: EdgeSource::Curated, via: None,
            });
        };
        // verbi (validati da is_verb_form via `IsA verbo`)
        for v in ["avere", "essere", "sentire", "andare", "vedere"] {
            add(v, RelationType::IsA, "verbo");
        }
        add("avere", RelationType::IsA, "ausiliare");
        add("essere", RelationType::IsA, "ausiliare");
        // classi chiuse
        for p in ["con", "da", "di"] { add(p, RelationType::IsA, "preposizione"); }
        for c in ["quando", "e"] { add(c, RelationType::IsA, "congiunzione"); }
        for pr in ["mi", "me", "ne"] { add(pr, RelationType::IsA, "pronome"); }
        add("mia", RelationType::IsA, "determinante");
        // §J — classe → funzione (dato)
        add("nome", RelationType::UsedFor, "argomento");
        add("aggettivo", RelationType::UsedFor, "attributo");
        add("preposizione", RelationType::UsedFor, "complemento");
        add("avverbio", RelationType::UsedFor, "circostanza");
        add("congiunzione", RelationType::UsedFor, "connessione");
        add("determinante", RelationType::UsedFor, "determinazione");
        add("articolo", RelationType::UsedFor, "determinazione");
        // ruolo del connettivo (subordinante vs coordinante) — richiede `via`,
        // dopo gli `add` per non tenere `kp` mutabilmente prestato due volte.
        drop(add);
        kp.add_edge(TypedEdge {
            subject: "quando".into(), relation: RelationType::UsedFor, object: "legare".into(),
            confidence: 0.95, source: EdgeSource::Curated, via: Some("temporale".into()),
        });
        kp.add_edge(TypedEdge {
            subject: "e".into(), relation: RelationType::UsedFor, object: "legare".into(),
            confidence: 0.95, source: EdgeSource::Curated, via: Some("additiva".into()),
        });
        kp
    }

    fn residuo_alfabetico(raw: &[String], a: &Analisi) -> usize {
        raw.iter().zip(&a.funzioni)
            .filter(|(w, f)| w.chars().any(|c| c.is_alphabetic()) && **f == Funzione::Residuo)
            .count()
    }

    #[test]
    fn composto_una_clausola() {
        let kp = kp_full();
        let kg = KnowledgeGraph::new();
        // ho(aux) litigato(part→litigare) con(prep) mia(det) sorella(nome)
        let raw = words("ho litigato con mia sorella");
        let a = analizza(&raw, &kp, &kg);
        assert_eq!(a.clausole.len(), 1, "una sola clausola");
        assert_eq!(a.clausole[0].predicato_lemma.as_deref(), Some("litigare"));
        assert_eq!(a.funzioni[0], Funzione::Predicato);   // ho
        assert_eq!(a.funzioni[1], Funzione::Predicato);   // litigato
        assert_eq!(a.funzioni[2], Funzione::Complemento); // con
        assert_eq!(a.funzioni[3], Funzione::Determinazione); // mia
        assert_eq!(a.funzioni[4], Funzione::Complemento); // sorella (in gruppo prep.)
        assert_eq!(residuo_alfabetico(&raw, &a), 0);
    }

    #[test]
    fn due_clausole_subordinata() {
        let kp = kp_full();
        let kg = KnowledgeGraph::new();
        // CLAUSOLA A: mi sento solo  |  CLAUSOLA B: da quando me ne sono andato di casa
        let raw = words("mi sento solo da quando me ne sono andato di casa");
        let a = analizza(&raw, &kp, &kg);
        assert_eq!(a.clausole.len(), 2, "principale + subordinata");
        // confine su "da" (preposizione che precede il subordinante "quando")
        assert_eq!(a.clausole[0].range, 0..3);
        assert_eq!(a.clausole[1].range, 3..11);
        assert_eq!(a.clausole[0].predicato_lemma.as_deref(), Some("sentire"));
        assert_eq!(a.clausole[1].predicato_lemma.as_deref(), Some("andare"));
        // la principale è indipendente; "da quando…" è subordinata
        assert!(!a.clausole[0].subordinate, "la principale è indipendente");
        assert!(a.clausole[1].subordinate, "'da quando…' è subordinata");
        // ruoli clausola A
        assert_eq!(a.funzioni[0], Funzione::Argomento);   // mi
        assert_eq!(a.funzioni[1], Funzione::Predicato);   // sento
        assert_eq!(a.funzioni[2], Funzione::Argomento);   // solo (testa, predicativo)
        // ruoli clausola B
        assert_eq!(a.funzioni[3], Funzione::Complemento); // da
        assert_eq!(a.funzioni[4], Funzione::Connettivo);  // quando
        assert_eq!(a.funzioni[7], Funzione::Predicato);   // sono
        assert_eq!(a.funzioni[8], Funzione::Predicato);   // andato
        assert_eq!(a.funzioni[9], Funzione::Complemento); // di
        assert_eq!(a.funzioni[10], Funzione::Complemento); // casa
        assert_eq!(residuo_alfabetico(&raw, &a), 0);
    }

    #[test]
    fn coordinazione_argomenti_resta_una_clausola() {
        let kp = kp_full();
        let kg = KnowledgeGraph::new();
        // "ho visto mondi e idee": un solo gruppo verbale → una clausola; la "e"
        // coordina ARGOMENTI (la testa post-e "idee" è un nome, non un verbo).
        let raw = words("ho visto mondi e idee");
        let a = analizza(&raw, &kp, &kg);
        assert_eq!(a.clausole.len(), 1, "argomenti coordinati = una clausola");
        assert_eq!(a.funzioni[1], Funzione::Predicato);  // visto
        assert_eq!(a.funzioni[2], Funzione::Argomento);  // mondi (testa)
        assert_eq!(a.funzioni[3], Funzione::Connettivo); // e
        assert_eq!(a.funzioni[4], Funzione::Argomento);  // idee (testa coordinata)
    }
}
