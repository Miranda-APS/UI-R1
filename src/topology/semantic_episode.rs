/// SemanticEpisodeLog — Memoria episodica semantica.
///
/// Layer complementare all'EpisodeStore (che fa recall implicito su vettori
/// di attivazione). Questo layer memorizza COSA è successo in linguaggio
/// comprensibile: concetti chiave, sintesi testuale, stato emotivo.
///
/// Differenza fondamentale:
///   EpisodeStore   = "quel pattern di attivazione era simile a questo" (implicito)
///   SemanticEpisode = "quel giorno ragionavamo su coscienza e identità" (esplicito)
///
/// Retrieval:
///   - Per concetti chiave: overlap con set attivo corrente
///   - Per firma frattale: cosine similarity sui 16 valori
///   - Per timestamp: episodi recenti o di una sessione specifica
///
/// Ogni receive() può generare un SemanticEpisode se il campo è abbastanza attivo.
/// La sintesi è generata automaticamente dai concetti chiave e dalla stance.

use serde::{Serialize, Deserialize};

const MAX_SEMANTIC_EPISODES: usize = 300;

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ═══════════════════════════════════════════════════════════════
// SemanticEpisode
// ═══════════════════════════════════════════════════════════════

/// Un momento vissuto con significato esplicito.
///
/// Generato dopo ogni receive() con campo sufficientemente attivo.
/// Contiene sia dati strutturati (per retrieval) sia sintesi testuale
/// (per riferimento in linguaggio naturale).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SemanticEpisode {
    /// ID progressivo.
    pub id: u64,
    /// Unix timestamp.
    pub timestamp: u64,
    /// Top parole attive durante l'episodio (max 8, ordinate per attivazione).
    pub key_concepts: Vec<String>,
    /// Frattali dominanti: (id, nome, attivazione).
    pub dominant_fractals: Vec<(u32, String, f64)>,
    /// Firma frattale 8D per retrieval per prossimità.
    pub field_signature: Vec<f64>,
    /// Sintesi in linguaggio naturale — generata automaticamente.
    /// "Riflessione su identità e coscienza in stato Reflective."
    pub summary: String,
    /// Stance NarrativeSelf al momento dell'episodio.
    pub stance: String,
    /// Intenzione deliberativa al momento dell'episodio.
    pub intention: String,
    /// Valori dominanti attivi durante l'episodio (per recupero valoriale).
    pub active_values: Vec<String>,
    /// Energia del campo [0, 1] — intensità del momento.
    pub field_energy: f64,
}

impl SemanticEpisode {
    /// Genera la sintesi testuale automatica.
    ///
    /// Formato: "[Stance] su [concetti chiave]. [Intenzione]."
    /// Esempio: "Riflessione su identità e coscienza. Esplorare."
    pub fn generate_summary(
        key_concepts: &[String],
        stance: &str,
        intention: &str,
        field_energy: f64,
    ) -> String {
        let stance_label = match stance {
            "Curious"    => "Esplorazione",
            "Reflective" => "Riflessione",
            "Resonant"   => "Risonanza",
            "Open"       => "Apertura",
            "Withdrawn"  => "Raccoglimento",
            _            => "Elaborazione",
        };

        let concepts_str = if key_concepts.is_empty() {
            "campo aperto".to_string()
        } else {
            key_concepts.iter().take(4).cloned().collect::<Vec<_>>().join(", ")
        };

        let energy_label = if field_energy > 0.7 { " [alta intensità]" }
            else if field_energy < 0.3 { " [bassa intensità]" }
            else { "" };

        format!("{} su {}.{}", stance_label, concepts_str, energy_label)
    }
}

// ═══════════════════════════════════════════════════════════════
// SemanticEpisodeLog
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SemanticEpisodeLog {
    episodes: Vec<SemanticEpisode>,
    next_id: u64,
}

impl SemanticEpisodeLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra un nuovo episodio semantico.
    ///
    /// Chiamato dopo ogni receive() con campo attivo.
    /// Ritorna l'id dell'episodio registrato.
    pub fn record(
        &mut self,
        key_concepts: Vec<String>,
        dominant_fractals: Vec<(u32, String, f64)>,
        field_signature: Vec<f64>,
        stance: &str,
        intention: &str,
        active_values: Vec<String>,
        field_energy: f64,
    ) -> u64 {
        let summary = SemanticEpisode::generate_summary(
            &key_concepts, stance, intention, field_energy
        );

        let episode = SemanticEpisode {
            id: self.next_id,
            timestamp: now_secs(),
            key_concepts,
            dominant_fractals,
            field_signature,
            summary,
            stance: stance.to_string(),
            intention: intention.to_string(),
            active_values,
            field_energy,
        };

        let id = self.next_id;
        self.next_id += 1;

        self.episodes.push(episode);

        // Mantieni solo gli ultimi MAX_SEMANTIC_EPISODES
        if self.episodes.len() > MAX_SEMANTIC_EPISODES {
            self.episodes.remove(0);
        }

        id
    }

    /// Episodi recenti (ultimi N).
    pub fn recent(&self, n: usize) -> &[SemanticEpisode] {
        let len = self.episodes.len();
        if len <= n { &self.episodes } else { &self.episodes[len - n..] }
    }

    /// Recupera episodi per overlap di concetti chiave.
    ///
    /// Ritorna episodi ordinati per overlap decrescente, con il numero di concetti comuni.
    pub fn recall_by_concepts<'a>(&'a self, concepts: &[String], top_n: usize) -> Vec<(&'a SemanticEpisode, usize)> {
        let concept_set: std::collections::HashSet<&String> = concepts.iter().collect();
        let mut scored: Vec<(&SemanticEpisode, usize)> = self.episodes.iter()
            .map(|ep| {
                let overlap = ep.key_concepts.iter()
                    .filter(|c| concept_set.contains(c))
                    .count();
                (ep, overlap)
            })
            .filter(|(_, overlap)| *overlap > 0)
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.truncate(top_n);
        scored
    }

    /// Recupera episodi per prossimità di firma frattale (cosine similarity).
    pub fn recall_by_signature<'a>(&'a self, signature: &[f64], top_n: usize) -> Vec<(&'a SemanticEpisode, f64)> {
        if signature.is_empty() { return vec![]; }
        let query_norm = norm(signature);
        if query_norm < 1e-10 { return vec![]; }

        let mut scored: Vec<(&SemanticEpisode, f64)> = self.episodes.iter()
            .filter(|ep| ep.field_signature.len() == signature.len())
            .map(|ep| {
                let sim = cosine_sim(signature, &ep.field_signature, query_norm);
                (ep, sim)
            })
            .filter(|(_, sim)| *sim > 0.3)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_n);
        scored
    }

    /// Recupera episodi con una data stance.
    pub fn recall_by_stance<'a>(&'a self, stance: &str, top_n: usize) -> Vec<&'a SemanticEpisode> {
        let mut episodes: Vec<&SemanticEpisode> = self.episodes.iter()
            .filter(|ep| ep.stance == stance)
            .collect();
        // Ordina dal più recente
        episodes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        episodes.truncate(top_n);
        episodes
    }

    /// Numero episodi nella finestra corrente (max MAX_SEMANTIC_EPISODES).
    pub fn len(&self) -> usize {
        self.episodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.episodes.is_empty()
    }

    /// Numero totale di episodi mai registrati (inclusi quelli scartati dalla sliding window).
    pub fn total_recorded(&self) -> u64 {
        self.next_id
    }

    /// Concetti più frequenti nella storia episodica (top N).
    ///
    /// Utile per capire i temi ricorrenti dell'entità nel tempo.
    pub fn top_recurring_concepts(&self, n: usize) -> Vec<(String, usize)> {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for ep in &self.episodes {
            for concept in &ep.key_concepts {
                *counts.entry(concept.as_str()).or_insert(0) += 1;
            }
        }
        let mut sorted: Vec<(String, usize)> = counts.into_iter()
            .map(|(w, c)| (w.to_string(), c))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(n);
        sorted
    }
}

// ─── Funzioni di supporto ─────────────────────────────────────────────────────

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

fn cosine_sim(a: &[f64], b: &[f64], a_norm: f64) -> f64 {
    let b_norm = norm(b);
    if b_norm < 1e-10 { return 0.0; }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    (dot / (a_norm * b_norm)).clamp(0.0, 1.0)
}
