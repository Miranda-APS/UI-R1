//! Phase 86 — §2: riconoscitore derivazionale (la "risali-alla-base").
//!
//! > Design: `docs/raw/architettura/curation_famiglie_derivazionali_design.md`.
//!
//! Data una forma *derivata* (`affamato`, `pulizia`, `sociologo`, `abbagliante`),
//! trova il suo **lessema-base** + il **tipo di derivazione**. È:
//!  - il **consumatore minimo di `DerivesFrom`** (Principio 7: niente archi senza
//!    lettore): l'arco curato dall'agente ha **priorità** (autoritativo);
//!  - il **fallback regolare** per i derivati non ancora curati: regole di
//!    suffisso, ma **sempre validate contro il KG** (un candidato-base che non
//!    esiste come nodo NON viene proposto → niente falsi positivi).
//!
//! Gli IRREGOLARI (forma non ricostruibile per stripping, o significato slittato)
//! NON passano per la regola: vivono come arco `DerivesFrom` curato (§4 del design).
//! La regola incarna il regolare; il dato cura l'irregolare.

use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

/// Trova (base, tipo) di una forma derivata. `None` se non è riconoscibile come
/// derivato (né arco curato, né regola che validi contro il KG).
pub fn derivational_base(word: &str, kg: &KnowledgeGraph) -> Option<(String, String)> {
    let w = word.to_lowercase();

    // (1) Arco curato `word DerivesFrom base via=tipo` — autoritativo.
    if let Some((base, _conf, via)) =
        kg.query_objects_with_via(&w, RelationType::DerivesFrom).into_iter().next()
    {
        let tipo = via.map(|s| s.to_string()).unwrap_or_else(|| "derivazione".to_string());
        return Some((base.to_string(), tipo));
    }

    // (2) Fallback regolare: regole di suffisso, validate contro il KG.
    let (tipo, candidates) = suffix_candidates(&w)?;
    for base in candidates {
        if base != w && kg.contains(&base) {
            return Some((base, tipo.to_string()));
        }
    }
    None
}

/// Per un suffisso derivazionale noto, restituisce (tipo, candidati-base). La
/// ricostruzione è approssimata di proposito: la validazione contro il KG in
/// `derivational_base` scarta i candidati inesistenti, quindi possiamo proporre
/// più forme senza rischio. Tabella modesta (i suffissi più produttivi); gli
/// irregolari sono dato curato, non regola.
fn suffix_candidates(w: &str) -> Option<(&'static str, Vec<String>)> {
    // (suffisso, tipo, terminazioni-base da appendere allo stem)
    const RULES: &[(&str, &str, &[&str])] = &[
        ("zione", "nominalizzazione",  &["re", "are", "ere"]),   // creazione→creare
        ("sione", "nominalizzazione",  &["dere", "tere"]),       // comprensione→comprendere (parziale)
        ("mento", "nominalizzazione",  &["are", "ere", "ire"]),  // trattamento→trattare
        ("tore",  "agentivo",          &["are", "ere", "ire"]),  // creatore→creare
        ("trice", "agentivo",          &["are", "ere", "ire"]),
        ("ante",  "participio-attivo", &["are"]),                // abbagliante→abbagliare
        ("ente",  "participio-attivo", &["ere", "ire"]),         // vivente→vivere
        ("ato",   "aggettivazione",    &["are"]),                // affamato→affamare
        ("uto",   "aggettivazione",    &["ere"]),
        ("ito",   "aggettivazione",    &["ire"]),
        ("oso",   "aggettivazione",    &["a", "e", "o"]),        // pauroso→paura
        ("osa",   "aggettivazione",    &["a", "e", "o"]),
        ("ità",   "nominalizzazione",  &["e", "o", "a"]),        // felicità→felice
    ];
    for (suf, tipo, ends) in RULES {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() < 2 { continue; }
            let mut bases: Vec<String> = ends.iter().map(|e| format!("{stem}{e}")).collect();
            bases.push(stem.to_string()); // anche lo stem nudo (es. nomi tronchi)
            return Some((tipo, bases));
        }
    }
    // -logo ↔ -logia (composti): sociologo→sociologia
    if let Some(stem) = w.strip_suffix("logo") {
        if stem.len() >= 2 { return Some(("agentivo", vec![format!("{stem}logia")])); }
    }
    // -mente (avverbio) ← aggettivo femminile: lentamente→lenta→lento
    if let Some(stem) = w.strip_suffix("mente") {
        if stem.len() >= 2 {
            let masch = format!("{}o", stem.trim_end_matches('a'));
            return Some(("avverbio", vec![stem.to_string(), masch]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::relation::{EdgeSource, TypedEdge};

    fn kg_with(nodes: &[&str]) -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        // crea nodi reali aggiungendo un arco IsA fittizio (li rende presenti)
        for n in nodes {
            kg.add(n, RelationType::IsA, "concetto");
        }
        kg
    }

    #[test]
    fn arco_curato_ha_priorita() {
        let mut kg = kg_with(&["pulire"]);
        kg.add_edge(TypedEdge {
            subject: "pulizia".into(), relation: RelationType::DerivesFrom,
            object: "pulire".into(), confidence: 0.9, source: EdgeSource::AgentCurated,
            via: Some("nominalizzazione".into()),
        });
        let (base, tipo) = derivational_base("pulizia", &kg).expect("base attesa");
        assert_eq!(base, "pulire");
        assert_eq!(tipo, "nominalizzazione");
    }

    #[test]
    fn regola_participio_attivo_validata() {
        let kg = kg_with(&["abbagliare"]);
        let (base, tipo) = derivational_base("abbagliante", &kg).expect("base attesa");
        assert_eq!(base, "abbagliare");
        assert_eq!(tipo, "participio-attivo");
    }

    #[test]
    fn regola_agentivo_logo() {
        let kg = kg_with(&["sociologia"]);
        let (base, tipo) = derivational_base("sociologo", &kg).expect("base attesa");
        assert_eq!(base, "sociologia");
        assert_eq!(tipo, "agentivo");
    }

    #[test]
    fn regola_aggettivazione_oso() {
        let kg = kg_with(&["paura"]);
        let (base, _) = derivational_base("pauroso", &kg).expect("base attesa");
        assert_eq!(base, "paura");
    }

    #[test]
    fn niente_falso_positivo_se_base_assente() {
        // "abbagliante" ma il KG NON ha "abbagliare" → niente proposta spuria.
        let kg = kg_with(&["altro"]);
        assert!(derivational_base("abbagliante", &kg).is_none());
    }

    #[test]
    fn parola_non_derivata_resta_none() {
        let kg = kg_with(&["fame", "casa"]);
        assert!(derivational_base("casa", &kg).is_none());
    }
}
