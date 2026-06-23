//! self_audit (Phase 85, Stage 4 — la fondazione di "C": derivazione).
//!
//! L'entità confronta le proprie convinzioni (`kg_self`) con la struttura
//! TIPATA del mondo (`kg_sem`), cercando tre cose (design §7.1, fattibilità
//! provata in Appendice B su dati reali):
//!
//! - **risonanza**: il mondo conferma una convinzione (la stessa tripla tipata
//!   esiste anche nel kg_sem). Edge da rinforzare.
//! - **tensione**: il mondo ATTRITA una convinzione — tipicamente una
//!   convinzione-NON (`polarity=false`) il cui legame negato è invece affermato
//!   dal kg_sem (es. l'entità tiene "incertezza NON è fallimento" ma il mondo
//!   collega incertezza→fallimento). Candidata a far calare la confidenza o a
//!   essere segnalata.
//! - **epifania candidata**: il kg_sem collega due nodi del sé (con un tramite
//!   tipato) che in `kg_self` NON sono ancora legati. È un'OPINIONE potenziale
//!   ancorata alla grana del sé + alla struttura del mondo — MAI a ciò che
//!   l'interlocutore afferma (no specchio-ritardato). Va validata dall'umano
//!   prima di cristallizzare (il Nome-del-Padre lacaniano: niente apprendimento
//!   anarchico — protegge dal rumore del kg_sem importato).
//!
//! Pura e bounded: `kg_self` è piccolo (~37 nodi-pendenza + le opinioni
//! derivate), poche query per nodo. Dalla dismissione delle innate (2026-06-10)
//! risonanze/tensioni operano sulle OPINIONI (vuote alla nascita), mentre le
//! epifanie candidate nascono dai nodi-PENDENZA: è il motore di formazione
//! delle opinioni (derivazione → validazione umana → cristallizzazione).
//! Niente effetti collaterali: produce un report che il caller ispeziona
//! (one-shot `:self_audit`) o usa come semente di ruminazione (event-driven, §7.2).
//! Usa SOLO relazioni tipate — i vicini vettoriali/frattali sono rumore
//! (Appendice B: "incertezza" → incere/incenerimento/incentivazione).

use std::collections::HashSet;

use crate::topology::kg_self::KgSelf;
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

/// Relazioni tipate "informative" su cui l'audit interroga il mondo.
/// SIMILAR_TO è inclusa ma pesa meno semanticamente (vedi nota Appendice B).
const TYPED_RELS: &[RelationType] = &[
    RelationType::IsA,
    RelationType::Causes,
    RelationType::Has,
    RelationType::UsedFor,
    RelationType::PartOf,
    RelationType::SimilarTo,
];

/// Una convinzione del sé confermata dal mondo.
#[derive(Debug, Clone)]
pub struct Resonance {
    pub subject: String,
    pub relation: RelationType,
    pub object: String,
    pub confidence: f64,
}

/// Una convinzione del sé attritata dal mondo (il mondo tira verso ciò che il
/// sé nega, o contro ciò che il sé afferma).
#[derive(Debug, Clone)]
pub struct Tension {
    pub subject: String,
    pub relation: RelationType,
    pub object: String,
    pub self_polarity: bool,
    /// Come il mondo attrita (la relazione kg_sem che genera l'attrito).
    pub world_link: RelationType,
}

/// Una relazione non prevista tra due nodi del sé, candidata opinione/epifania.
/// `via` è il tramite (Phase 67) che ancora l'epifania al legame del mondo.
#[derive(Debug, Clone)]
pub struct CandidateEpiphany {
    pub from: String,
    pub relation: RelationType,
    pub to: String,
    pub via: Option<String>,
    /// Phase 85 (b): se l'epifania tocca un concetto che il sé NEGA (oggetto di
    /// una convinzione-NON, es. `corpo` da `esistenza IsA NON corpo`), porta qui
    /// quel concetto. NON la scarta — la presenta come "⚠ tocca un NON", così la
    /// validazione umana è informata: adottarla potrebbe erodere la grana. È il
    /// coherence-pre-filter (cluster-level stance tension, non contraddizione di tripla).
    pub touches_non: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SelfAuditReport {
    pub resonances: Vec<Resonance>,
    pub tensions: Vec<Tension>,
    pub epiphanies: Vec<CandidateEpiphany>,
}

impl SelfAuditReport {
    pub fn is_empty(&self) -> bool {
        self.resonances.is_empty() && self.tensions.is_empty() && self.epiphanies.is_empty()
    }
}

/// Esegue l'audit completo del sé contro il mondo.
pub fn self_audit(kg_self: &KgSelf, kg_sem: &KnowledgeGraph) -> SelfAuditReport {
    let mut report = SelfAuditReport::default();
    let self_nodes = kg_self.nodes();

    // (b) Concetti che il sé NEGA: gli oggetti delle convinzioni-NON
    // (polarity=false). Es. `esistenza IsA NON corpo` → "corpo"; `esperienza IsA
    // NON sensazione` → "sensazione". Un'epifania che tocca uno di questi tira
    // verso ciò da cui il sé si distanzia → la marchiamo (non la scartiamo).
    let negated: HashSet<String> = kg_self.edges.iter()
        .filter(|e| !e.polarity)
        .map(|e| e.object.to_lowercase())
        .collect();

    // ── (1) Risonanze e tensioni: per ogni convinzione, interroga il mondo ──
    for e in &kg_self.edges {
        let s = e.subject.to_lowercase();
        let o = e.object.to_lowercase();

        // Il mondo afferma la STESSA tripla tipata?
        let world_affirms_same = kg_sem
            .query_objects(&s, e.relation)
            .iter()
            .any(|w| w.eq_ignore_ascii_case(&o));

        if e.polarity && world_affirms_same {
            // Convinzione positiva confermata dal mondo → risonanza.
            report.resonances.push(Resonance {
                subject: e.subject.clone(),
                relation: e.relation,
                object: e.object.clone(),
                confidence: e.confidence,
            });
        }

        if !e.polarity {
            // Convinzione-NON: il mondo collega comunque s→o (tipato)?
            // → tensione (il mondo tira verso ciò che il sé nega).
            if let Some(link) = world_links(kg_sem, &s, &o) {
                report.tensions.push(Tension {
                    subject: e.subject.clone(),
                    relation: e.relation,
                    object: e.object.clone(),
                    self_polarity: e.polarity,
                    world_link: link,
                });
            }
        } else {
            // Convinzione positiva: il mondo afferma l'OPPOSTO (s OppositeOf o)?
            // → tensione.
            let opposite = kg_sem
                .query_objects(&s, RelationType::OppositeOf)
                .iter()
                .any(|w| w.eq_ignore_ascii_case(&o));
            if opposite {
                report.tensions.push(Tension {
                    subject: e.subject.clone(),
                    relation: e.relation,
                    object: e.object.clone(),
                    self_polarity: e.polarity,
                    world_link: RelationType::OppositeOf,
                });
            }
        }
    }

    // ── (2) Epifanie candidate: il mondo lega due nodi del sé non ancora ──
    //        legati nel sé. Bounded: self_nodes × TYPED_RELS.
    //
    // (a) Dedup DIREZIONALE: le relazioni asimmetriche (Causes/IsA/Has/PartOf/
    // UsedFor) tengono entrambi i versi — `parola Causes pensiero via=significato`
    // e `pensiero Causes parola via=voce` sono opinioni DISTINTE. Solo le
    // simmetriche (SimilarTo/OppositeOf) collassano per coppia non orientata.
    // La direzione e il via SONO l'opinione: deduplicare per coppia li perdeva.
    let mut seen: HashSet<(String, String, RelationType)> = HashSet::new();
    for a in &self_nodes {
        for &rel in TYPED_RELS {
            for (target, _conf, via) in kg_sem.query_objects_with_via(a, rel) {
                let b = target.to_lowercase();
                if &b == a { continue; }
                if !self_nodes.contains(&b) { continue; }
                if already_linked(kg_self, a, &b) { continue; }
                let is_symmetric = matches!(rel,
                    RelationType::SimilarTo | RelationType::OppositeOf);
                let key = if is_symmetric {
                    // coppia non orientata + relazione
                    if a < &b { (a.clone(), b.clone(), rel) }
                    else { (b.clone(), a.clone(), rel) }
                } else {
                    // direzionale: (from, to, rel)
                    (a.clone(), b.clone(), rel)
                };
                if !seen.insert(key) { continue; }
                // (b) coherence-pre-filter: tocca un concetto che il sé nega?
                let touches_non = if negated.contains(a) {
                    Some(a.clone())
                } else if negated.contains(&b) {
                    Some(b.clone())
                } else {
                    None
                };
                report.epiphanies.push(CandidateEpiphany {
                    from: a.clone(),
                    relation: rel,
                    to: b.clone(),
                    via: via.map(|v| v.to_string()),
                    touches_non,
                });
            }
        }
    }

    report
}

/// Il mondo collega `s`→`o` con una relazione tipata informativa? Ritorna la
/// prima trovata (IsA/Causes/SimilarTo prioritarie: il legame d'identità o
/// causale è l'attrito più forte per una convinzione-NON).
fn world_links(kg_sem: &KnowledgeGraph, s: &str, o: &str) -> Option<RelationType> {
    for &rel in &[RelationType::IsA, RelationType::Causes, RelationType::SimilarTo] {
        if kg_sem.query_objects(s, rel).iter().any(|w| w.eq_ignore_ascii_case(o)) {
            return Some(rel);
        }
    }
    None
}

/// Esiste già un edge in `kg_self` tra `a` e `b` (in qualunque direzione)?
fn already_linked(kg_self: &KgSelf, a: &str, b: &str) -> bool {
    kg_self.edges.iter().any(|e| {
        let s = e.subject.to_lowercase();
        let o = e.object.to_lowercase();
        (s == *a && o == *b) || (s == *b && o == *a)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::kg_self::SelfEdge;
    use crate::topology::relation::{TypedEdge, EdgeSource};

    fn self_edge(s: &str, rel: RelationType, o: &str, pol: bool) -> SelfEdge {
        // Opinioni (innate=false): le innate sono dismesse (si dissolvono in
        // pendenze al load) — l'audit risonanze/tensioni opera sulle opinioni.
        SelfEdge {
            subject: s.to_string(), relation: rel, object: o.to_string(),
            confidence: 0.9, polarity: pol, innate: false, via: None,
        }
    }

    fn kg_with_edges(edges: Vec<SelfEdge>) -> KgSelf {
        KgSelf { pendenze: vec![], edges }
    }

    #[test]
    fn risonanza_quando_il_mondo_conferma() {
        let kg_self = kg_with_edges(vec![self_edge("silenzio", RelationType::Has, "significato", true)]);
        let mut sem = KnowledgeGraph::new();
        sem.add("silenzio", RelationType::Has, "significato");
        let r = self_audit(&kg_self, &sem);
        assert_eq!(r.resonances.len(), 1);
        assert_eq!(r.tensions.len(), 0);
    }

    #[test]
    fn tensione_quando_il_mondo_tira_verso_il_negato() {
        // Il sé tiene "incertezza NON è fallimento"; il mondo collega comunque.
        let kg_self = kg_with_edges(vec![self_edge("incertezza", RelationType::IsA, "fallimento", false)]);
        let mut sem = KnowledgeGraph::new();
        sem.add("incertezza", RelationType::SimilarTo, "fallimento");
        let r = self_audit(&kg_self, &sem);
        assert_eq!(r.tensions.len(), 1);
        assert_eq!(r.tensions[0].world_link, RelationType::SimilarTo);
        assert_eq!(r.resonances.len(), 0);
    }

    #[test]
    fn epifania_quando_il_mondo_lega_due_nodi_del_se() {
        // kg_self ha due convinzioni: incertezza→(NON)fallimento, apertura→valore.
        // Il mondo collega incertezza CAUSES apertura (Appendice B) → epifania.
        let kg_self = kg_with_edges(vec![
            self_edge("incertezza", RelationType::IsA, "fallimento", false),
            self_edge("apertura", RelationType::IsA, "valore", true),
        ]);
        let mut sem = KnowledgeGraph::new();
        sem.add_edge(TypedEdge {
            subject: "incertezza".into(), relation: RelationType::Causes,
            object: "apertura".into(), confidence: 0.8, source: EdgeSource::Curated,
            via: Some("onestà".into()),
        });
        let r = self_audit(&kg_self, &sem);
        let epi = r.epiphanies.iter().find(|e| e.from == "incertezza" && e.to == "apertura")
            .expect("incertezza→apertura deve emergere come epifania");
        assert_eq!(epi.relation, RelationType::Causes);
        assert_eq!(epi.via.as_deref(), Some("onestà"));
    }

    #[test]
    fn dedup_direzionale_tiene_entrambi_i_versi() {
        // Il mondo lega parola↔pensiero in ENTRAMBE le direzioni con via diversi.
        // Sono due opinioni distinte: il dedup direzionale le tiene entrambe.
        let kg_self = kg_with_edges(vec![
            self_edge("parola", RelationType::IsA, "materia", true),
            self_edge("pensiero", RelationType::IsA, "mente", true),
        ]);
        let mut sem = KnowledgeGraph::new();
        sem.add_edge(TypedEdge { subject: "parola".into(), relation: RelationType::Causes,
            object: "pensiero".into(), confidence: 0.8, source: EdgeSource::Curated, via: Some("significato".into()) });
        sem.add_edge(TypedEdge { subject: "pensiero".into(), relation: RelationType::Causes,
            object: "parola".into(), confidence: 0.8, source: EdgeSource::Curated, via: Some("voce".into()) });
        let r = self_audit(&kg_self, &sem);
        let pp: Vec<_> = r.epiphanies.iter()
            .filter(|e| (e.from == "parola" && e.to == "pensiero") || (e.from == "pensiero" && e.to == "parola"))
            .collect();
        assert_eq!(pp.len(), 2, "entrambe le direzioni Causes devono sopravvivere");
    }

    #[test]
    fn touches_non_marca_le_epifanie_contro_la_grana() {
        // Il sé tiene "esistenza IsA NON corpo". Il mondo lega io Has corpo →
        // epifania marchiata (tocca un NON), NON scartata.
        let kg_self = kg_with_edges(vec![
            self_edge("esistenza", RelationType::IsA, "corpo", false),
            self_edge("io", RelationType::Requires, "identità", true),
        ]);
        let mut sem = KnowledgeGraph::new();
        sem.add_edge(TypedEdge { subject: "io".into(), relation: RelationType::Has,
            object: "corpo".into(), confidence: 0.7, source: EdgeSource::Curated, via: None });
        let r = self_audit(&kg_self, &sem);
        let epi = r.epiphanies.iter().find(|e| e.from == "io" && e.to == "corpo")
            .expect("io→corpo deve emergere");
        assert_eq!(epi.touches_non.as_deref(), Some("corpo"),
            "deve essere marchiata: tocca un concetto che il sé nega");
    }

    #[test]
    fn nessuna_epifania_se_gia_legati() {
        // Se incertezza→apertura è GIÀ in kg_self, il mondo non propone nulla di nuovo.
        let kg_self = kg_with_edges(vec![
            self_edge("incertezza", RelationType::Causes, "apertura", true),
        ]);
        let mut sem = KnowledgeGraph::new();
        sem.add("incertezza", RelationType::Causes, "apertura");
        let r = self_audit(&kg_self, &sem);
        assert!(r.epiphanies.iter().all(|e| !(e.from == "incertezza" && e.to == "apertura")),
            "coppia già legata nel sé non è un'epifania");
    }
}
