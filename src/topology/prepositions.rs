//! Phase 86 — Stadio 2: preposizioni come ipotesi di relazione (validate dal KG).
//!
//! > Design: `docs/raw/architettura/comprensione_esplorativa_design.md` §3.4 +
//! > `curation_famiglie_derivazionali_design.md`. Memoria: preposizioni = ipotesi
//! > DETERMINISTICHE, non distribuzioni.
//!
//! Una preposizione, in "X *prep* Y", non determina la relazione: ne propone un
//! **insieme di candidate**; il KG valida quale regge fra i due nomi. Verificato
//! sui dati: lo stesso "di" si disambigua in `Causes` (dolore *della* separazione),
//! `PartOf` (senso *della* vita), `Requires` (fiducia *del* tempo).
//!
//! È la metà-IN del ponte; la metà-OUT (relazione→preposizione, per il collasso
//! articolato, Stadio 3) riusa la stessa mappa al contrario.
//!
//! Tabella in Rust come *seed* (come la tabella suffissi/participi): è grammatica
//! ("fisica del mondo"), non trigger comportamentale. Potrà migrare nel kg_proc.

use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

use RelationType::*;

/// Le relazioni-ipotesi che una preposizione propone, in ordine di preferenza.
/// La preposizione è normalizzata (anche le forme articolate: del/dello/…→di).
pub fn hypotheses(prep: &str) -> &'static [RelationType] {
    match normalize(prep).as_str() {
        // specificazione / possesso / materia / origine-causale
        "di"     => &[PartOf, Has, IsA, Causes],
        // origine / agente / fine ("qualcosa da bere") / causa ("tremo dalla paura")
        "da"     => &[Causes, UsedFor],
        // fine / causa
        "per"    => &[UsedFor, Causes],
        // strumento / compagnia
        "con"    => &[UsedFor, Coexists],
        // tema / sfondo
        "su"     => &[ContextOf],
        "in"     => &[ContextOf, PartOf],
        // opposizione
        "contro" => &[OppositeOf, Excludes],
        "senza"  => &[Excludes],
        // relazione reciproca
        "tra"    => &[Coexists],
        "fra"    => &[Coexists],
        _ => &[],
    }
}

/// È una preposizione (anche articolata)? Più ampio di `hypotheses`: include "a"
/// (dativo/moto/paragone) che NON ha una relazione-contenuto pulita ma introduce
/// pur sempre un complemento da catturare (Stadio 2: il secondo argomento).
/// Senza questo, "preferisco la pizza **alla** pasta" e "giocare **a** calcio"
/// perdevano del tutto il loro secondo termine.
pub fn is_preposition(prep: &str) -> bool {
    matches!(normalize(prep).as_str(),
        "di" | "da" | "per" | "con" | "su" | "in" |
        "contro" | "senza" | "tra" | "fra" | "a")
}

/// Il RUOLO logico del complemento, dalla preposizione (normalizzata) + la
/// categoria del verbo reggente. Grammaticale e deterministico (chiuso-classe),
/// non un'inferenza: "a" dopo un verbo VALUTATIVO è un termine di **paragone**
/// ("preferisco X a Y"), altrove un **termine** (dativo/destinazione).
pub fn complement_role(prep: &str, verb_category: Option<&str>) -> Option<&'static str> {
    Some(match normalize(prep).as_str() {
        "a"      => if verb_category == Some("valutativo") { "paragone" } else { "termine" },
        "di"     => "specificazione",
        "da"     => "origine",
        "per"    => "fine",
        "con"    => "compagnia",
        "su" | "in" => "ambito",
        "contro" => "opposizione",
        "senza"  => "esclusione",
        "tra" | "fra" => "relazione",
        _ => return None,
    })
}

/// Normalizza una preposizione (articolate → semplice).
fn normalize(prep: &str) -> String {
    match prep.to_lowercase().as_str() {
        "del" | "dello" | "della" | "dei" | "degli" | "delle" | "dell'" | "d'" => "di".into(),
        "dal" | "dallo" | "dalla" | "dai" | "dagli" | "dalle" | "dall'" => "da".into(),
        "nel" | "nello" | "nella" | "nei" | "negli" | "nelle" | "nell'" => "in".into(),
        "sul" | "sullo" | "sulla" | "sui" | "sugli" | "sulle" | "sull'" => "su".into(),
        "col" | "coi" | "collo" | "colla" => "con".into(),
        "al" | "allo" | "alla" | "ai" | "agli" | "alle" | "all'" => "a".into(),
        other => other.to_string(),
    }
}

/// Direzione del legame trovato fra testa e complemento.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir { HeadToComplement, ComplementToHead }

/// Esito della disambiguazione di "head *prep* complement".
#[derive(Debug, Clone)]
pub struct PrepResolution {
    pub relation: RelationType,
    pub dir: Dir,
    /// true se il KG conferma la relazione; false se è la sola ipotesi di default
    /// (nessuna candidata trovata nel mondo → l'ipotesi resta non validata).
    pub validated: bool,
}

/// Disambigua "head *prep* complement" contro il KG: sceglie la prima relazione
/// candidata della preposizione che il mondo CONFERMA fra i due nomi (in una
/// delle due direzioni). Se nessuna è confermata, ritorna la prima ipotesi
/// (default, `validated=false`) — onesto: "ho un'ipotesi, il mondo non la regge".
pub fn disambiguate(
    head: &str,
    prep: &str,
    complement: &str,
    kg: &KnowledgeGraph,
) -> Option<PrepResolution> {
    let cands = hypotheses(prep);
    if cands.is_empty() { return None; }
    let h = head.to_lowercase();
    let c = complement.to_lowercase();
    for &rel in cands {
        // head --rel--> complement ?
        if kg.query_objects(&h, rel).iter().any(|o| o.eq_ignore_ascii_case(&c)) {
            return Some(PrepResolution { relation: rel, dir: Dir::HeadToComplement, validated: true });
        }
        // complement --rel--> head ? ("dolore della separazione": separazione Causes dolore)
        if kg.query_objects(&c, rel).iter().any(|o| o.eq_ignore_ascii_case(&h)) {
            return Some(PrepResolution { relation: rel, dir: Dir::ComplementToHead, validated: true });
        }
    }
    // Nessuna candidata confermata: ipotesi di default, non validata.
    Some(PrepResolution { relation: cands[0], dir: Dir::HeadToComplement, validated: false })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::relation::{EdgeSource, TypedEdge};

    fn kg() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        let mut add = |s: &str, r: RelationType, o: &str| {
            kg.add_edge(TypedEdge { subject: s.into(), relation: r, object: o.into(),
                confidence: 0.9, source: EdgeSource::Curated, via: None });
        };
        add("gamba", PartOf, "tavolo");          // la gamba del tavolo
        add("separazione", Causes, "dolore");     // il dolore della separazione
        add("coltello", UsedFor, "tagliare");     // taglio con il coltello
        add("uomo", Has, "valore");               // un uomo di valore
        kg
    }

    #[test]
    fn di_si_disambigua_in_partof() {
        let r = disambiguate("gamba", "del", "tavolo", &kg()).unwrap();
        assert_eq!(r.relation, PartOf);
        assert!(r.validated);
        assert_eq!(r.dir, Dir::HeadToComplement);
    }

    #[test]
    fn di_si_disambigua_in_causes_direzione_inversa() {
        // "il dolore della separazione": testa=dolore, ma è separazione→Causes→dolore
        let r = disambiguate("dolore", "della", "separazione", &kg()).unwrap();
        assert_eq!(r.relation, Causes);
        assert_eq!(r.dir, Dir::ComplementToHead);
        assert!(r.validated);
    }

    #[test]
    fn di_si_disambigua_in_has() {
        let r = disambiguate("uomo", "di", "valore", &kg()).unwrap();
        assert_eq!(r.relation, Has);
    }

    #[test]
    fn con_strumento_usedfor() {
        let r = disambiguate("coltello", "con", "tagliare", &kg()).unwrap();
        assert_eq!(r.relation, UsedFor);
        assert!(r.validated);
    }

    #[test]
    fn ipotesi_default_se_mondo_non_conferma() {
        // "di" fra due nomi senza legame nel KG → prima ipotesi (PartOf), non validata.
        let r = disambiguate("pippo", "di", "pluto", &kg()).unwrap();
        assert_eq!(r.relation, PartOf);
        assert!(!r.validated);
    }

    #[test]
    fn preposizione_senza_ipotesi() {
        // "a" (dativo/direzione) non ha relazione-contenuto → None.
        assert!(disambiguate("vado", "a", "roma", &kg()).is_none());
    }
}
