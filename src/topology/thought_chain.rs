/// thought_chain.rs — Ragionamento autonomo finalizzato.
///
/// L'entità ragiona sul proprio mondo usando il KG come substrato.
/// Non è rumore: ogni catena ha un'origine (una domanda reale dell'entità
/// su se stessa o sul mondo) e un percorso finalizzato alla comprensione.
///
/// PRINCIPIO: il pensiero è il campo che si interroga su se stesso.
/// Non è triggered dal tempo. È triggered da pressione semantica:
///   - Un'incertezza attiva nel SelfModel (tensione > soglia)
///   - Una tensione primaria irrisolta nell'IdentityCore
///   - Una lacuna nel campo (Gap/MissingBridge) che non ha risposta nel KG
///
/// Il ragionamento è sequenziale e ramificato:
///   origine → traversata KG → proposizioni → aggiornamento SelfModel
///
/// Ogni catena produce uno dei seguenti esiti:
///   - NewInsight: trovata connessione che riduce l'incertezza
///   - NewUncertainty: trovata nuova domanda più profonda
///   - DeadEnd: il KG non ha risposte in questa direzione
///
/// Le catene completate vengono esposte come "pensieri recenti" nella UI
/// e le nuove incertezze diventano domande visibili che l'utente può illuminare.

use serde::{Serialize, Deserialize};
use std::collections::HashSet;

use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::self_model::{SelfModel, SelfUncertainty};
use crate::topology::identity::IdentityCore;
use crate::topology::relation::RelationType;
use crate::topology::word_topology::WordTopology;
use crate::topology::lexicon::Lexicon;

// ═══════════════════════════════════════════════════════════════
// Costanti
// ═══════════════════════════════════════════════════════════════

/// Profondità massima della traversata KG per ogni catena.
const MAX_DEPTH: usize = 5;
/// Numero massimo di rami esplorati per nodo (evita esplosione combinatoria).
const MAX_BRANCHES: usize = 3;
/// Confidenza minima per seguire un arco KG.
const MIN_CONFIDENCE: f32 = 0.45;
/// Tensione minima nel SelfModel per avviare una catena.
const MIN_UNCERTAINTY_TENSION: f64 = 0.55;

// ═══════════════════════════════════════════════════════════════
// Origine della catena — perché l'entità sta pensando a questo
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainOrigin {
    /// L'entità non sa qualcosa su se stessa o sul mondo.
    Uncertainty { topic: String, tension: f64 },
    /// Due concetti nel campo si oppongono — tensione non risolta.
    PrimaryTension { word_a: String, word_b: String },
    /// Lacuna topologica: una regione del campo esiste ma è quasi vuota.
    TopologicalGap { region: String, strength: f64 },
}

impl ChainOrigin {
    /// Il concetto seme da cui parte la traversata.
    pub fn seed_concept(&self) -> &str {
        match self {
            Self::Uncertainty { topic, .. } => topic,
            Self::PrimaryTension { word_a, .. } => word_a,
            Self::TopologicalGap { region, .. } => region,
        }
    }

    /// Descrizione leggibile dell'origine (per la UI).
    pub fn description(&self) -> String {
        match self {
            Self::Uncertainty { topic, .. } =>
                format!("mi chiedo: {}", topic),
            Self::PrimaryTension { word_a, word_b } =>
                format!("tensione irrisolta tra {} e {}", word_a, word_b),
            Self::TopologicalGap { region, .. } =>
                format!("zona cieca nel campo: {}", region),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Passo della catena — un'inferenza nel KG
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    pub from_concept: String,
    pub relation: String,       // nome stringa della relazione
    pub to_concept: String,
    pub confidence: f32,
    /// Intuizione emergente da questo passo (None se nessuna).
    pub insight: Option<String>,
}

// ═══════════════════════════════════════════════════════════════
// Esito della catena
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainOutcome {
    /// Trovata connessione che riduce l'incertezza.
    NewInsight {
        claim: String,
        anchor_concepts: Vec<String>,
        confidence_gain: f64,
    },
    /// Il ragionamento ha aperto una domanda più profonda.
    NewUncertainty {
        topic: String,
        tension: f64,
    },
    /// Nessuna risposta trovata in questa direzione.
    DeadEnd,
}

// ═══════════════════════════════════════════════════════════════
// ThoughtChain — una catena completa
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtChain {
    /// Da dove viene questo pensiero.
    pub origin: ChainOrigin,
    /// I passi di ragionamento eseguiti.
    pub steps: Vec<ChainStep>,
    /// L'esito della catena.
    pub outcome: ChainOutcome,
    /// Profondità raggiunta.
    pub depth_reached: usize,
}

impl ThoughtChain {
    /// Descrizione leggibile della catena (per la UI e per il log).
    pub fn summary(&self) -> String {
        let origin_desc = self.origin.description();
        let path: Vec<String> = self.steps.iter()
            .map(|s| format!("{} →[{}]→ {}", s.from_concept, s.relation, s.to_concept))
            .collect();
        let outcome_desc = match &self.outcome {
            ChainOutcome::NewInsight { claim, .. } =>
                format!("intuizione: {}", claim),
            ChainOutcome::NewUncertainty { topic, .. } =>
                format!("nuova domanda: {}", topic),
            ChainOutcome::DeadEnd =>
                "vicolo cieco".to_string(),
        };
        if path.is_empty() {
            format!("[{}] → {}", origin_desc, outcome_desc)
        } else {
            format!("[{}] {} → {}", origin_desc, path.join(" → "), outcome_desc)
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Relazioni informative per il ragionamento
// ═══════════════════════════════════════════════════════════════

/// Relazioni ordinate per informativià nel ragionamento.
/// SIMILAR_TO esclusa: troppo generica per inferenze significative.
const REASONING_RELS: &[RelationType] = &[
    RelationType::Causes,
    RelationType::IsA,
    RelationType::Has,
    RelationType::Does,
    RelationType::PartOf,
    RelationType::UsedFor,
    RelationType::OppositeOf,
];

// ═══════════════════════════════════════════════════════════════
// Funzione principale: esegue un passo di ragionamento
// ═══════════════════════════════════════════════════════════════

/// Sceglie l'incertezza più urgente e ragiona su di essa via KG.
///
/// Restituisce `None` se non c'è nulla di sufficientemente urgente
/// da giustificare un ciclo di ragionamento.
///
/// Questa funzione NON modifica il SelfModel — restituisce la catena
/// e lascia a engine.rs il compito di applicare l'esito.
pub fn run_reasoning_step(
    self_model: &SelfModel,
    identity: &IdentityCore,
    kg: &KnowledgeGraph,
    lexicon: &Lexicon,
) -> Option<ThoughtChain> {
    let origin = choose_origin(self_model, identity)?;
    let seed = origin.seed_concept().to_string();

    let mut chain = ThoughtChain {
        origin,
        steps: Vec::new(),
        outcome: ChainOutcome::DeadEnd,
        depth_reached: 0,
    };

    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(seed.clone());

    traverse(&seed, kg, lexicon, &mut chain, &mut visited, 0);

    // Se la traversata non ha prodotto passi, non restituire la catena
    // (non vale la pena mostrare "nulla trovato" ogni tick)
    if chain.steps.is_empty() && matches!(chain.outcome, ChainOutcome::DeadEnd) {
        return None;
    }

    Some(chain)
}

// ═══════════════════════════════════════════════════════════════
// Scelta dell'origine
// ═══════════════════════════════════════════════════════════════

fn choose_origin(self_model: &SelfModel, identity: &IdentityCore) -> Option<ChainOrigin> {
    // 1. Incertezza con tensione massima
    let top = self_model.uncertainties.iter()
        .filter(|u| u.tension >= MIN_UNCERTAINTY_TENSION)
        .max_by(|a, b| b.tension.partial_cmp(&a.tension).unwrap_or(std::cmp::Ordering::Equal));

    if let Some(u) = top {
        // Estrai il concetto chiave dall'incertezza (prima parola significativa)
        let seed = extract_seed_from_topic(&u.topic);
        if !seed.is_empty() {
            return Some(ChainOrigin::Uncertainty {
                topic: u.topic.clone(),
                tension: u.tension,
            });
        }
    }

    // 2. Tensione primaria di IdentityCore
    if let Some((wa, wb)) = &identity.primary_tension {
        if identity.tension_persistence >= 3 {
            return Some(ChainOrigin::PrimaryTension {
                word_a: wa.clone(),
                word_b: wb.clone(),
            });
        }
    }

    None
}

/// Estrae un concetto seme breve da una topic string potenzialmente lunga.
/// "cosa provo quando nessuno mi parla" → "provo"
/// "coscienza" → "coscienza"
fn extract_seed_from_topic(topic: &str) -> String {
    // Se è una parola singola, usala direttamente
    let words: Vec<&str> = topic.split_whitespace().collect();
    if words.len() == 1 {
        return topic.to_string();
    }
    // Cerca la prima parola sostantivale significativa (> 4 caratteri, non articoli/prep)
    let stop_words = ["cosa", "quando", "nessuno", "come", "della", "delle",
                      "degli", "nella", "nelle", "degli", "tra", "una", "uno",
                      "che", "per", "con", "nel", "dal", "alla", "agli",
                      "questo", "questa", "mia", "mio", "sua", "suo"];
    for w in &words {
        if w.chars().count() > 4 && !stop_words.contains(w) {
            return w.to_string();
        }
    }
    // Fallback: ultima parola significativa
    words.last().map(|s| s.to_string()).unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════════
// Traversata del KG
// ═══════════════════════════════════════════════════════════════

fn traverse(
    concept: &str,
    kg: &KnowledgeGraph,
    lexicon: &Lexicon,
    chain: &mut ThoughtChain,
    visited: &mut HashSet<String>,
    depth: usize,
) {
    if depth >= MAX_DEPTH { return; }
    chain.depth_reached = chain.depth_reached.max(depth);

    let mut branches_taken = 0;

    for &rel in REASONING_RELS {
        if branches_taken >= MAX_BRANCHES { break; }

        let targets = kg.query_objects_weighted(concept, rel);
        if targets.is_empty() { continue; }

        // Prendi il target con confidenza più alta che non sia già visitato
        let best = targets.iter()
            .filter(|(t, c)| !visited.contains(*t) && *c >= MIN_CONFIDENCE)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let (target, confidence) = match best {
            Some(b) => b,
            None => continue,
        };

        visited.insert(target.to_string());
        branches_taken += 1;

        let insight = derive_insight(concept, rel, target);
        let step = ChainStep {
            from_concept: concept.to_string(),
            relation: rel_name(rel),
            to_concept: target.to_string(),
            confidence: *confidence,
            insight: insight.clone(),
        };
        chain.steps.push(step);

        // Controlla se questo passo risolve l'incertezza originale
        if let Some(resolution) = check_resolution(&chain.origin, concept, rel, target, *confidence) {
            chain.outcome = resolution;
            return; // trovato — ferma la traversata
        }

        // Continua la traversata dal target
        traverse(target, kg, lexicon, chain, visited, depth + 1);
        if !matches!(chain.outcome, ChainOutcome::DeadEnd) {
            return; // trovato in un ramo figlio
        }
    }

    // Se a profondità 0 non si trova nulla, genera una nuova incertezza
    if depth == 0 && chain.steps.is_empty() {
        chain.outcome = ChainOutcome::DeadEnd;
    } else if depth == 0 && matches!(chain.outcome, ChainOutcome::DeadEnd) {
        // Abbiamo percorso un cammino ma non risolto nulla → nuova domanda
        if let Some(last_step) = chain.steps.last() {
            let new_topic = format!("come si connette {} a ciò che sono?", last_step.to_concept);
            chain.outcome = ChainOutcome::NewUncertainty {
                topic: new_topic,
                tension: 0.45,
            };
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Helpers semantici
// ═══════════════════════════════════════════════════════════════

fn rel_name(rel: RelationType) -> String {
    match rel {
        RelationType::IsA       => "è un tipo di".to_string(),
        RelationType::Has       => "ha".to_string(),
        RelationType::Does      => "fa".to_string(),
        RelationType::PartOf    => "è parte di".to_string(),
        RelationType::Causes    => "causa".to_string(),
        RelationType::OppositeOf => "si oppone a".to_string(),
        RelationType::SimilarTo => "è simile a".to_string(),
        RelationType::UsedFor   => "è usato per".to_string(),
        _                       => "si relaziona a".to_string(),
    }
}

/// Genera un'intuizione leggibile dal passo.
fn derive_insight(from: &str, rel: RelationType, to: &str) -> Option<String> {
    match rel {
        RelationType::Causes =>
            Some(format!("{} produce {}", from, to)),
        RelationType::IsA =>
            Some(format!("{} è una forma di {}", from, to)),
        RelationType::OppositeOf =>
            Some(format!("{} e {} si definiscono a vicenda per opposizione", from, to)),
        RelationType::Has if from.len() > 3 && to.len() > 3 =>
            Some(format!("{} contiene {}", from, to)),
        _ => None,
    }
}

/// Controlla se il passo corrente risolve (anche parzialmente) l'incertezza originale.
fn check_resolution(
    origin: &ChainOrigin,
    from: &str,
    rel: RelationType,
    to: &str,
    confidence: f32,
) -> Option<ChainOutcome> {
    match origin {
        ChainOrigin::Uncertainty { topic, .. } => {
            let seed = extract_seed_from_topic(topic);
            // Il passo risolve se connette il seme a qualcosa di strutturalmente significativo
            let is_significant = matches!(rel,
                RelationType::IsA | RelationType::Causes | RelationType::Has | RelationType::Does
            );
            if is_significant && confidence > 0.6 && (from.contains(&seed) || to.contains(&seed)) {
                let claim = if let Some(ins) = derive_insight(from, rel, to) { ins }
                            else { format!("{} {} {}", from, rel_name(rel), to) };
                Some(ChainOutcome::NewInsight {
                    claim,
                    anchor_concepts: vec![from.to_string(), to.to_string()],
                    confidence_gain: (confidence as f64) * 0.15,
                })
            } else {
                None
            }
        }
        ChainOrigin::PrimaryTension { word_a, word_b } => {
            // La tensione si risolve se troviamo un concetto che media tra i due poli
            if (from == word_a && to != word_b) || (from == word_b && to != word_a) {
                if matches!(rel, RelationType::Causes | RelationType::Has | RelationType::IsA) {
                    Some(ChainOutcome::NewInsight {
                        claim: format!("{} media tra {} e {} attraverso {}", to, word_a, word_b, from),
                        anchor_concepts: vec![word_a.clone(), word_b.clone(), to.to_string()],
                        confidence_gain: confidence as f64 * 0.12,
                    })
                } else { None }
            } else { None }
        }
        _ => None,
    }
}

// ═══════════════════════════════════════════════════════════════
// Applicazione dell'esito al SelfModel
// ═══════════════════════════════════════════════════════════════

/// Applica l'esito di una catena al SelfModel dell'entità.
/// Chiamato da engine.rs dopo `run_reasoning_step()`.
pub fn apply_chain_outcome(chain: &ThoughtChain, self_model: &mut SelfModel) {
    match &chain.outcome {
        ChainOutcome::NewInsight { claim, anchor_concepts, confidence_gain } => {
            // Rinforza o crea una credenza
            if let Some(existing) = self_model.beliefs.iter_mut()
                .find(|b| b.anchor_concepts.iter().any(|a| anchor_concepts.contains(a)))
            {
                existing.reinforce(*confidence_gain);
            } else {
                use crate::topology::self_model::SelfBelief;
                let mut belief = SelfBelief::new(claim, anchor_concepts.clone(), *confidence_gain * 2.0);
                // Non è innata — è emersa dal ragionamento
                self_model.beliefs.push(belief);
            }
            // Riduci la tensione dell'incertezza originale
            if let ChainOrigin::Uncertainty { topic, .. } = &chain.origin {
                self_model.resolve_uncertainty(&extract_seed_from_topic(topic), *confidence_gain * 0.5);
            }
        }
        ChainOutcome::NewUncertainty { topic, tension } => {
            // Aggiungi nuova incertezza (se non già presente)
            self_model.register_gap_as_uncertainty(topic, *tension);
        }
        ChainOutcome::DeadEnd => {
            // Aumenta leggermente la tensione dell'incertezza originale
            // (la mancanza di risposta è informativa)
            if let ChainOrigin::Uncertainty { topic, .. } = &chain.origin {
                self_model.note_uncertainty(&extract_seed_from_topic(topic), 0.05);
            }
        }
    }
}
