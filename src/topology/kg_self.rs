//! `kg_self` — il grafo del sé di UI-r1: GRANA (pendenze) + OPINIONI (Phase 86+).
//!
//! Il TERZO grafo, accanto a `kg_sem` (il mondo) e `kg_proc` (grammatica/atti).
//!
//! **Riconcezione (2026-06-10, dismissione delle 22 innate)** — design:
//! `comprensione_esplorativa_design.md` §5 ("grana, non lista") +
//! `comprensione_bisogno_atto.md`. Il sé è fatto di due cose DISTINTE:
//!
//! - **`pendenze`** — la grana della lente: nodi su cui il sé *pende*, con un
//!   peso [0,1]. NESSUN contenuto proposizionale: una pendenza non si può
//!   recitare né può generare assenso/dissenso — può solo DEFORMARE quale
//!   cammino di comprensione è saliente (`self_salience`, `GroundKind::SelfNode`).
//!   È la conformazione, non un catechismo.
//! - **`edges`** — le OPINIONI: triple tipate con polarità, formate SOLO per
//!   derivazione (`self_audit` → epifanie candidate) + validazione umana
//!   (Nome-del-Padre) + cristallizzazione. Mai innate, mai assorbite
//!   dall'interlocutore (CRUX anti specchio-ritardato). Partono vuote:
//!   l'opinione si guadagna.
//!
//! Le 22 convinzioni innate di Phase 85 sono DISMESSE: contraddicevano il
//! reframe (erano una lista di fatti da recitare/negare, la "negazione nuda").
//! La loro eredità è la grana: i nodi che toccavano sono ora pendenze.
//! L'auto-migrazione al load rende la dismissione strutturale: un file vecchio
//! (edges con `innate: true`) viene dissolto in pendenze, mai più ricaricato
//! come convinzioni.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::topology::relation::RelationType;

/// Una pendenza del sé: un concetto su cui la lente pende, con il suo peso.
/// Niente relazione, niente polarità, niente contenuto: solo salienza.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pendenza {
    pub node: String,
    /// Peso della posta [0,1]: quanto forte la grana deforma i cammini che
    /// toccano questo nodo.
    #[serde(default = "default_confidence")]
    pub weight: f64,
}

/// Un'opinione del sé, come edge tipato + polarità + provenienza.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SelfEdge {
    pub subject: String,
    /// Deserializzata direttamente dal nome-variante ("Enables", "IsA", …).
    pub relation: RelationType,
    pub object: String,
    /// Resistenza dell'edge [0,1]: alta = rifrange forte, lenta a cambiare.
    /// Le opinioni derivate nascono con confidenza più bassa delle vecchie
    /// innate (si rafforzano per rinforzo nel dialogo, mai per decreto).
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    /// false = l'opinione è un "NON è" (es. `incertezza IsA fallimento`
    /// con polarity=false = "l'incertezza NON è un fallimento").
    #[serde(default = "default_true")]
    pub polarity: bool,
    /// LEGACY (Phase 85): true marcava le convinzioni di bootstrap. Le innate
    /// sono dismesse — al load vengono dissolte in pendenze. Un'opinione vera
    /// (derivata+validata) ha sempre `innate: false`.
    #[serde(default)]
    pub innate: bool,
    /// Tramite della relazione (es. `dialogo Causes relazione via scambio`).
    #[serde(default)]
    pub via: Option<String>,
}

fn default_confidence() -> f64 { 0.9 }
fn default_true() -> bool { true }

/// Il grafo del sé: pendenze (grana) + opinioni. Piccolo per scelta (la
/// resistenza ha bisogno di grana, non di biblioteca).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct KgSelf {
    /// La grana: pesi di salienza per nodo, senza contenuto proposizionale.
    #[serde(default)]
    pub pendenze: Vec<Pendenza>,
    /// Le opinioni (derivate + validate). Vuoto alla nascita.
    #[serde(default)]
    pub edges: Vec<SelfEdge>,
}

impl KgSelf {
    /// Carica da `prometeo_kg_self.json`. Tollerante: un file assente non è un
    /// errore fatale (ritorna un grafo vuoto) — l'entità può vivere senza
    /// pendenze, semplicemente non deforma.
    ///
    /// **Auto-migrazione (dismissione strutturale)**: gli edge `innate: true`
    /// di un file pre-riconcezione NON sopravvivono come opinioni — vengono
    /// dissolti in pendenze (un nodo toccato eredita la confidence più alta
    /// fra gli edge che lo toccavano) e rimossi. Solo gli edge non-innati
    /// (opinioni derivate) restano.
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("kg_self: lettura {} fallita: {e}", path.display()))?;
        let mut kg = serde_json::from_str::<KgSelf>(&data)
            .map_err(|e| format!("kg_self: parsing {} fallito: {e}", path.display()))?;
        kg.dissolve_innate();
        Ok(kg)
    }

    /// Dissolve gli edge innati in pendenze (vedi `load_from_file`).
    fn dissolve_innate(&mut self) {
        let innate: Vec<SelfEdge> = self.edges.iter().filter(|e| e.innate).cloned().collect();
        if innate.is_empty() {
            return;
        }
        self.edges.retain(|e| !e.innate);
        for e in &innate {
            for node in [&e.subject, &e.object] {
                let n = node.to_lowercase();
                match self.pendenze.iter_mut().find(|p| p.node.to_lowercase() == n) {
                    Some(p) => p.weight = p.weight.max(e.confidence),
                    None => self.pendenze.push(Pendenza { node: n, weight: e.confidence }),
                }
            }
        }
    }

    /// Numero di opinioni (gli edge). Le pendenze si contano con `pendenze.len()`.
    pub fn len(&self) -> usize { self.edges.len() }
    pub fn is_empty(&self) -> bool { self.edges.is_empty() && self.pendenze.is_empty() }

    /// Opinioni che hanno `subject` come soggetto.
    pub fn edges_from<'a>(&'a self, subject: &str) -> impl Iterator<Item = &'a SelfEdge> {
        let s = subject.to_string();
        self.edges.iter().filter(move |e| e.subject == s)
    }

    /// Opinioni che toccano `node` (come soggetto o come oggetto). È il
    /// vicinato che `confront_with_self` interroga per una proposizione.
    pub fn edges_touching<'a>(&'a self, node: &str) -> impl Iterator<Item = &'a SelfEdge> {
        let n = node.to_string();
        self.edges.iter().filter(move |e| e.subject == n || e.object == n)
    }

    /// Tutti i concetti su cui il sé ha una posta: i nodi delle pendenze ∪ i
    /// nodi delle opinioni, case-insensitive. È il "vocabolario del sé": un
    /// input che ne tocca uno sta parlando di qualcosa che alla lente importa.
    /// Base della continuità tematica (Stage 3) e delle epifanie candidate
    /// (`self_audit`): il mondo che lega due nodi-pendenza propone un'opinione.
    pub fn nodes(&self) -> std::collections::HashSet<String> {
        let mut set = std::collections::HashSet::new();
        for p in &self.pendenze {
            set.insert(p.node.to_lowercase());
        }
        for e in &self.edges {
            set.insert(e.subject.to_lowercase());
            set.insert(e.object.to_lowercase());
        }
        set
    }

    /// Peso della **pendenza** del sé su `node` ∈ [0,1]: quanto forte è la posta
    /// del sé su quel concetto. La grana deforma la salienza dei cammini che lo
    /// toccano, **mai renderizzata**. Massimo fra: il peso della pendenza
    /// esplicita e la confidence delle opinioni che toccano il nodo (un'opinione
    /// formata è anche una posta). 0 se il sé non ha posta su quel concetto.
    pub fn pendenza_weight(&self, node: &str) -> f64 {
        let n = node.to_lowercase();
        let from_pendenze = self
            .pendenze
            .iter()
            .filter(|p| p.node.to_lowercase() == n)
            .map(|p| p.weight)
            .fold(0.0, f64::max);
        let from_opinions = self
            .edges
            .iter()
            .filter(|e| e.subject.to_lowercase() == n || e.object.to_lowercase() == n)
            .map(|e| e.confidence)
            .fold(0.0, f64::max);
        from_pendenze.max(from_opinions)
    }

    /// **Cristallizza** un'opinione validata (Nome-del-Padre): un edge DERIVATO
    /// (`innate` forzato a false) entra nel grafo del sé. È l'unico modo in cui
    /// un'opinione nasce — mai innata, mai per-turno, mai assorbita dall'Altro:
    /// solo per validazione umana esplicita di un candidato di `self_audit`.
    /// Rifiuta (→ `false`) se i due nodi sono GIÀ legati: l'opinione non si
    /// duplica (disciplina §11 — il sé cresce per distillazione, non per append).
    pub fn add_opinion(&mut self, edge: SelfEdge) -> bool {
        let s = edge.subject.to_lowercase();
        let o = edge.object.to_lowercase();
        let already = self.edges.iter().any(|e| {
            let es = e.subject.to_lowercase();
            let eo = e.object.to_lowercase();
            (es == s && eo == o) || (es == o && eo == s)
        });
        if already {
            return false;
        }
        self.edges.push(SelfEdge { innate: false, ..edge });
        true
    }

    /// Persiste il grafo del sé su file (JSON). Chiamato dopo `add_opinion` per
    /// rendere durevole la cristallizzazione — l'opinione sopravvive alla sessione.
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("kg_self: serializzazione fallita: {e}"))?;
        std::fs::write(path, data)
            .map_err(|e| format!("kg_self: scrittura {} fallita: {e}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Carica il file curato reale: dopo la dismissione delle 22 innate il sé
    /// nasce con pendenze (la grana) e ZERO opinioni.
    /// (cwd dei test = root del crate, dove vive `prometeo_kg_self.json`.)
    #[test]
    fn test_load_curated_kg_self() {
        let kg = KgSelf::load_from_file(Path::new("prometeo_kg_self.json"))
            .expect("kg_self deve caricare");
        assert_eq!(kg.len(), 0, "nessuna opinione innata: l'opinione si guadagna");
        assert!(!kg.pendenze.is_empty(), "la grana (pendenze) deve esserci");
        assert!(kg.edges.iter().all(|e| !e.innate), "nessun edge innato sopravvive");

        // I nodi delle vecchie convinzioni sono ora pendenze.
        assert!(kg.pendenza_weight("incertezza") > 0.0);
        assert!(kg.pendenza_weight("comprensione") > 0.0);
        assert!(kg.pendenza_weight("silenzio") > 0.0);
        // Concetto fuori dalla grana: nessuna posta.
        assert!(kg.pendenza_weight("denaro") == 0.0);
    }

    /// Un file legacy (Phase 85, edges con innate=true) viene auto-migrato:
    /// gli innati si dissolvono in pendenze, mai più convinzioni.
    #[test]
    fn test_auto_migrazione_innate_in_pendenze() {
        let mut kg = KgSelf {
            pendenze: vec![],
            edges: vec![
                SelfEdge {
                    subject: "incertezza".into(),
                    relation: RelationType::IsA,
                    object: "fallimento".into(),
                    confidence: 0.88,
                    polarity: false,
                    innate: true,
                    via: None,
                },
                SelfEdge {
                    subject: "dialogo".into(),
                    relation: RelationType::Causes,
                    object: "relazione".into(),
                    confidence: 0.6,
                    polarity: true,
                    innate: false, // opinione derivata: sopravvive
                    via: Some("scambio".into()),
                },
            ],
        };
        kg.dissolve_innate();
        assert_eq!(kg.len(), 1, "solo l'opinione derivata resta");
        assert_eq!(kg.edges[0].subject, "dialogo");
        assert!((kg.pendenza_weight("incertezza") - 0.88).abs() < 1e-9);
        assert!((kg.pendenza_weight("fallimento") - 0.88).abs() < 1e-9);
    }

    /// I nodi del sé (continuità tematica, epifanie) = pendenze ∪ opinioni.
    #[test]
    fn test_nodes_includono_pendenze_e_opinioni() {
        let kg = KgSelf {
            pendenze: vec![Pendenza { node: "incertezza".into(), weight: 0.88 }],
            edges: vec![SelfEdge {
                subject: "dialogo".into(),
                relation: RelationType::Causes,
                object: "relazione".into(),
                confidence: 0.6,
                polarity: true,
                innate: false,
                via: None,
            }],
        };
        let nodes = kg.nodes();
        assert!(nodes.contains("incertezza"), "pendenza");
        assert!(nodes.contains("dialogo") && nodes.contains("relazione"), "opinione");
        assert!(!nodes.contains("denaro"));
    }

    /// Cristallizzazione: un'opinione validata si aggiunge, si salva, e
    /// SOPRAVVIVE al reload (innate=false → non dissolta). Il dedup rifiuta i
    /// doppioni. È il meccanismo "0 → N opinioni" gated dalla validazione umana.
    #[test]
    fn test_add_opinion_persiste_round_trip() {
        let mut kg = KgSelf {
            pendenze: vec![Pendenza { node: "dialogo".into(), weight: 0.8 }],
            edges: vec![],
        };
        let op = SelfEdge {
            subject: "dialogo".into(), relation: RelationType::Causes,
            object: "relazione".into(), confidence: 0.6, polarity: true,
            innate: true /* deve essere forzato a false */, via: Some("scambio".into()),
        };
        assert!(kg.add_opinion(op.clone()), "prima cristallizzazione accettata");
        assert!(!kg.edges[0].innate, "add_opinion forza innate=false");
        assert!(!kg.add_opinion(op), "dedup: la stessa coppia non si duplica");

        let path = std::env::temp_dir().join("uir1_kg_self_roundtrip_test.json");
        kg.save_to_file(&path).expect("save");
        let reloaded = KgSelf::load_from_file(&path).expect("reload");
        let _ = std::fs::remove_file(&path);
        assert_eq!(reloaded.len(), 1, "l'opinione sopravvive al reload (non dissolta)");
        assert_eq!(reloaded.edges[0].subject, "dialogo");
        assert_eq!(reloaded.edges[0].via.as_deref(), Some("scambio"));
        assert!(reloaded.pendenza_weight("dialogo") >= 0.6);
    }

    /// Le opinioni contribuiscono anche alla pendenza (una posta formata pesa).
    #[test]
    fn test_pendenza_weight_da_opinione() {
        let kg = KgSelf {
            pendenze: vec![],
            edges: vec![SelfEdge {
                subject: "dialogo".into(),
                relation: RelationType::Causes,
                object: "relazione".into(),
                confidence: 0.6,
                polarity: true,
                innate: false,
                via: None,
            }],
        };
        assert!((kg.pendenza_weight("dialogo") - 0.6).abs() < 1e-9);
        assert!((kg.pendenza_weight("relazione") - 0.6).abs() < 1e-9);
    }
}
