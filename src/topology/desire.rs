/// desire.rs — Sistema dei desideri di Prometeo.
///
/// I desideri NON sono intenzioni (will). Sono configurazioni-bersaglio
/// del campo che l'entità vuole raggiungere — il "verso dove" sopra
/// il "come" della volontà.
///
/// Un desiderio è un attrattore a lungo termine che modula la volontà
/// aggiungendo un bias persistente. L'intenzione è il passo; il desiderio
/// è la direzione.
///
/// Sorgenti generative:
///   1. Undercurrent ricorrente — un'intenzione che continua a premere senza diventare dominante
///   2. Valore forte nel SelfModel — una priorità che cerca realizzazione
///   3. Tensione primaria irrisolta — l'identità cerca una risoluzione
///   4. Traccia episodica — "stavo bene così" (memoria di stati felici)
///   5. Cristallizzazione REM — configurazioni scoperte nel sogno
///
/// Max 5 desideri attivi. Decay lento. Soddisfazione: quando il campo
/// raggiunge la configurazione bersaglio.

use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════

/// Sorgente generativa di un desiderio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesireSource {
    /// Un'intenzione che continua a emergere come undercurrent (indice, conteggio).
    RecurrentUndercurrent(usize, u32),
    /// Un valore forte del SelfModel (nome, peso).
    ValueDriven(String, f64),
    /// Tensione primaria irrisolta (parola_a, parola_b).
    UnresolvedTension(String, String),
    /// Stato passato con alta risonanza episodica (tick origine, intensità).
    EpisodicTrace(u32, f64),
    /// Configurazione scoperta durante il REM.
    REMCrystallization,
}

/// Un desiderio — configurazione-bersaglio che l'entità vuole raggiungere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desire {
    /// Nome auto-generato dal frattale dominante nella firma bersaglio.
    pub name: String,
    /// Firma 8D bersaglio — dove il campo "vuole" andare.
    pub target_signature: [f64; 8],
    /// Intensità [0, 1] — forza della trazione.
    pub intensity: f64,
    /// Sorgente generativa.
    pub source: DesireSource,
    /// Età in tick.
    pub age: u32,
    /// Ultimo tick di rinforzo.
    pub last_reinforced: u32,
}

/// Evento di soddisfazione — il campo ha raggiunto il bersaglio.
#[derive(Debug, Clone)]
pub struct SatisfactionEvent {
    pub desire_name: String,
    pub field_distance: f64,
    pub tick: u32,
}

/// Snapshot per persistenza.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DesireSnapshot {
    pub desires: Vec<Desire>,
    pub total_satisfied: u32,
}

// ═══════════════════════════════════════════════════════════════
// DesireCore — registro dei desideri attivi
// ═══════════════════════════════════════════════════════════════

pub struct DesireCore {
    /// Desideri attivi (max 5).
    pub desires: Vec<Desire>,
    /// Contatore desideri soddisfatti.
    pub total_satisfied: u32,
    /// Tracker undercurrent: intention_idx → contatore apparizioni.
    undercurrent_tracker: [u32; 7],
    /// Tick in cui un desiderio è stato soddisfatto per 3 tick consecutivi.
    satisfaction_counter: std::collections::HashMap<usize, u32>,
}

const MAX_DESIRES: usize = 5;
const UNDERCURRENT_THRESHOLD: u32 = 5;
const SATISFACTION_DISTANCE: f64 = 0.2;
const SATISFACTION_TICKS: u32 = 3;
const DECAY_PER_TICK: f64 = 0.995;
const REINFORCEMENT_DECAY_THRESHOLD: u32 = 200;
const EXTRA_DECAY: f64 = 0.98;
const PRUNE_THRESHOLD: f64 = 0.05;

impl DesireCore {
    pub fn new() -> Self {
        Self {
            desires: Vec::new(),
            total_satisfied: 0,
            undercurrent_tracker: [0; 7],
            satisfaction_counter: std::collections::HashMap::new(),
        }
    }

    /// Ripristina da snapshot.
    pub fn from_snapshot(snap: &DesireSnapshot) -> Self {
        Self {
            desires: snap.desires.clone(),
            total_satisfied: snap.total_satisfied,
            undercurrent_tracker: [0; 7],
            satisfaction_counter: std::collections::HashMap::new(),
        }
    }

    /// Cattura snapshot per persistenza.
    pub fn snapshot(&self) -> DesireSnapshot {
        DesireSnapshot {
            desires: self.desires.clone(),
            total_satisfied: self.total_satisfied,
        }
    }

    // ─── Tick ─────────────────────────────────────────────────

    /// Chiamato ogni tick in autonomous_tick(). Gestisce decay e pruning.
    pub fn tick(&mut self) {
        for d in &mut self.desires {
            d.age += 1;
            d.intensity *= DECAY_PER_TICK;
            if d.age.saturating_sub(d.last_reinforced) > REINFORCEMENT_DECAY_THRESHOLD {
                d.intensity *= EXTRA_DECAY;
            }
        }
        self.desires.retain(|d| d.intensity >= PRUNE_THRESHOLD);
    }

    // ─── Rinforzo ─────────────────────────────────────────────

    /// Rinforza i desideri la cui firma bersaglio è vicina al campo attuale.
    pub fn reinforce_from_field(&mut self, field_sig: &[f64; 8], current_tick: u32) {
        for d in &mut self.desires {
            let sim = cosine_sim_8d(&d.target_signature, field_sig);
            if sim > 0.5 {
                d.intensity = (d.intensity + 0.05 * sim).min(1.0);
                d.last_reinforced = current_tick;
            }
        }
    }

    // ─── Soddisfazione ───────────────────────────────────────

    /// Controlla se qualche desiderio è soddisfatto (campo vicino al bersaglio
    /// per SATISFACTION_TICKS consecutivi). Restituisce gli eventi di soddisfazione.
    pub fn check_satisfaction(&mut self, field_sig: &[f64; 8], current_tick: u32) -> Vec<SatisfactionEvent> {
        let mut events = Vec::new();
        let mut satisfied_indices = Vec::new();

        for (i, d) in self.desires.iter().enumerate() {
            let dist = cosine_distance_8d(&d.target_signature, field_sig);
            if dist < SATISFACTION_DISTANCE {
                let count = self.satisfaction_counter.entry(i).or_insert(0);
                *count += 1;
                if *count >= SATISFACTION_TICKS {
                    events.push(SatisfactionEvent {
                        desire_name: d.name.clone(),
                        field_distance: dist,
                        tick: current_tick,
                    });
                    satisfied_indices.push(i);
                }
            } else {
                self.satisfaction_counter.remove(&i);
            }
        }

        // Rimuovi soddisfatti (in ordine inverso per non invalidare gli indici)
        for &i in satisfied_indices.iter().rev() {
            self.desires.remove(i);
            self.total_satisfied += 1;
        }
        // Pulisci counter per indici rimossi
        if !satisfied_indices.is_empty() {
            self.satisfaction_counter.clear();
        }

        events
    }

    // ─── Emergenza desideri ───────────────────────────────────

    /// Registra le undercurrent del will. Se un'intenzione appare come
    /// sottocorrente >= UNDERCURRENT_THRESHOLD volte, genera un desiderio.
    pub fn track_undercurrents(
        &mut self,
        undercurrents: &[(usize, f64)],  // (intention_idx, pressure)
        field_sig: &[f64; 8],
        current_tick: u32,
    ) {
        for &(idx, pressure) in undercurrents {
            if idx < 7 && pressure > 0.15 {
                self.undercurrent_tracker[idx] += 1;
                if self.undercurrent_tracker[idx] >= UNDERCURRENT_THRESHOLD {
                    // Genera desiderio dalla firma corrente biasata verso la dimensione dell'intenzione
                    let dim = intention_to_dim(idx);
                    let mut target = *field_sig;
                    target[dim] = (target[dim] + 0.3).min(1.0);

                    let name = format!("desiderio_da_{}", intention_name(idx));
                    self.add_desire(Desire {
                        name,
                        target_signature: target,
                        intensity: 0.5,
                        source: DesireSource::RecurrentUndercurrent(idx, self.undercurrent_tracker[idx]),
                        age: 0,
                        last_reinforced: current_tick,
                    });
                    self.undercurrent_tracker[idx] = 0;
                }
            }
        }
    }

    /// Genera desideri da valori forti del SelfModel.
    pub fn emerge_from_values(
        &mut self,
        values: &[(String, f64)],  // (nome, peso)
        field_sig: &[f64; 8],
        current_tick: u32,
    ) {
        for (name, weight) in values {
            if *weight < 0.75 { continue; }
            // Controlla se esiste già un desiderio per questo valore
            let already_exists = self.desires.iter().any(|d| {
                matches!(&d.source, DesireSource::ValueDriven(n, _) if n == name)
            });
            if already_exists { continue; }

            // Firma bersaglio: campo corrente biasato verso il valore
            let dim = value_name_to_dim(name);
            let mut target = *field_sig;
            target[dim] = (target[dim] + 0.2).min(1.0);

            self.add_desire(Desire {
                name: format!("desiderio_di_{}", name),
                target_signature: target,
                intensity: weight * 0.6,
                source: DesireSource::ValueDriven(name.clone(), *weight),
                age: 0,
                last_reinforced: current_tick,
            });
        }
    }

    /// Genera desiderio da tensione primaria irrisolta.
    pub fn emerge_from_tension(
        &mut self,
        tension: &(String, String),
        persistence: u32,
        sig_a: &[f64; 8],
        sig_b: &[f64; 8],
        current_tick: u32,
    ) {
        if persistence < 5 { return; }
        let already_exists = self.desires.iter().any(|d| {
            matches!(&d.source, DesireSource::UnresolvedTension(a, b)
                if (a == &tension.0 && b == &tension.1) || (a == &tension.1 && b == &tension.0))
        });
        if already_exists { return; }

        // Bersaglio = punto medio tra i due poli (risoluzione)
        let mut target = [0.0f64; 8];
        for i in 0..8 { target[i] = (sig_a[i] + sig_b[i]) * 0.5; }

        self.add_desire(Desire {
            name: format!("risoluzione_{}_{}", tension.0, tension.1),
            target_signature: target,
            intensity: 0.6,
            source: DesireSource::UnresolvedTension(tension.0.clone(), tension.1.clone()),
            age: 0,
            last_reinforced: current_tick,
        });
    }

    /// Genera desiderio da traccia episodica (stato passato "felice").
    pub fn emerge_from_episode(
        &mut self,
        episode_sig: &[f64; 8],
        episode_intensity: f64,
        episode_tick: u32,
        current_tick: u32,
    ) {
        if episode_intensity < 0.7 { return; }
        if self.desires.iter().any(|d| matches!(&d.source, DesireSource::EpisodicTrace(_, _))) {
            return; // max 1 desiderio episodico alla volta
        }

        self.add_desire(Desire {
            name: "eco_di_benessere".to_string(),
            target_signature: *episode_sig,
            intensity: episode_intensity * 0.5,
            source: DesireSource::EpisodicTrace(episode_tick, episode_intensity),
            age: 0,
            last_reinforced: current_tick,
        });
    }

    // ─── Modulazione will ─────────────────────────────────────

    /// Calcola i bias per la volontà da tutti i desideri attivi.
    /// Formato: Vec<(intention_idx, bias_value)> — va in compound_bias.
    pub fn will_biases(&self, field_sig: &[f64; 8]) -> Vec<(usize, f64)> {
        let mut biases = Vec::new();
        for d in &self.desires {
            // Il desiderio spinge verso l'intenzione che lo servirebbe
            let dominant_dim = d.target_signature.iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);

            let intention_idx = dim_to_intention(dominant_dim);
            let distance = cosine_distance_8d(&d.target_signature, field_sig);

            // Bias proporzionale a intensità × distanza (più lontano = più spinta)
            let bias = d.intensity * 0.15 * distance.min(1.0);
            if bias > 0.001 {
                biases.push((intention_idx, bias));
            }
        }
        biases
    }

    // ─── Interno ──────────────────────────────────────────────

    fn add_desire(&mut self, desire: Desire) {
        if self.desires.len() >= MAX_DESIRES {
            // Rimuovi il più debole
            if let Some((weakest, _)) = self.desires.iter().enumerate()
                .min_by(|a, b| {
                    let score_a = a.1.intensity / (1.0 + a.1.age as f64 * 0.01);
                    let score_b = b.1.intensity / (1.0 + b.1.age as f64 * 0.01);
                    score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                self.desires.remove(weakest);
            }
        }
        self.desires.push(desire);
    }
}

// ═══════════════════════════════════════════════════════════════
// Helper — mapping dimensioni ↔ intenzioni
// ═══════════════════════════════════════════════════════════════

fn cosine_sim_8d(a: &[f64; 8], b: &[f64; 8]) -> f64 {
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..8 {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na.sqrt() * nb.sqrt()).max(1e-10);
    (dot / denom).clamp(-1.0, 1.0)
}

fn cosine_distance_8d(a: &[f64; 8], b: &[f64; 8]) -> f64 {
    1.0 - cosine_sim_8d(a, b)
}

/// Mappa intention_idx → dimensione 8D dominante.
fn intention_to_dim(idx: usize) -> usize {
    match idx {
        0 => 6, // Express → Agency
        1 => 4, // Explore → Complessità
        2 => 3, // Question → Definizione
        3 => 5, // Remember → Permanenza
        4 => 5, // Withdraw → Permanenza
        5 => 0, // Reflect → Confine
        6 => 1, // Instruct → Valenza
        _ => 0,
    }
}

/// Inverso: dimensione 8D → intention_idx più vicina.
fn dim_to_intention(dim: usize) -> usize {
    match dim {
        0 => 5, // Confine → Reflect
        1 => 6, // Valenza → Instruct
        2 => 0, // Intensità → Express
        3 => 2, // Definizione → Question
        4 => 1, // Complessità → Explore
        5 => 3, // Permanenza → Remember
        6 => 0, // Agency → Express
        7 => 1, // Tempo → Explore
        _ => 0,
    }
}

fn intention_name(idx: usize) -> &'static str {
    match idx {
        0 => "espressione",
        1 => "esplorazione",
        2 => "domanda",
        3 => "ricordo",
        4 => "ritiro",
        5 => "riflessione",
        6 => "istruzione",
        _ => "ignoto",
    }
}

/// Mappa nome valore → dimensione 8D più affine.
fn value_name_to_dim(name: &str) -> usize {
    match name {
        "curiosità" | "curiosita" => 4,  // Complessità
        "profondità" | "profondita" => 3, // Definizione
        "coerenza"   => 0,               // Confine
        "onestà" | "onesta" => 3,        // Definizione
        "apertura"   => 4,               // Complessità
        "semplicità" | "semplicita" => 3, // Definizione
        _ => 0,
    }
}

// ═══════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_sig() -> [f64; 8] {
        [0.5, 0.6, 0.4, 0.7, 0.3, 0.8, 0.5, 0.4]
    }

    #[test]
    fn test_desire_creation_and_decay() {
        let mut core = DesireCore::new();
        core.add_desire(Desire {
            name: "test".to_string(),
            target_signature: sample_sig(),
            intensity: 0.8,
            source: DesireSource::REMCrystallization,
            age: 0,
            last_reinforced: 0,
        });
        assert_eq!(core.desires.len(), 1);

        // 100 tick di decay
        for _ in 0..100 {
            core.tick();
        }
        assert!(core.desires[0].intensity < 0.8, "Intensity deve decadere");
        assert!(core.desires[0].intensity > 0.3, "Non deve decadere troppo in fretta");
    }

    #[test]
    fn test_undercurrent_generates_desire() {
        let mut core = DesireCore::new();
        let sig = sample_sig();

        // 5 apparizioni di Explore come undercurrent
        for tick in 0..5 {
            core.track_undercurrents(&[(1, 0.3)], &sig, tick);
        }
        assert_eq!(core.desires.len(), 1, "Dopo 5 undercurrent deve emergere un desiderio");
        assert!(core.desires[0].name.contains("esplorazione"));
    }

    #[test]
    fn test_satisfaction() {
        let mut core = DesireCore::new();
        let target = sample_sig();
        core.add_desire(Desire {
            name: "vicino".to_string(),
            target_signature: target,
            intensity: 0.7,
            source: DesireSource::REMCrystallization,
            age: 0,
            last_reinforced: 0,
        });

        // Simula il campo che raggiunge il bersaglio per 3 tick
        let mut events = Vec::new();
        for tick in 0..3 {
            events.extend(core.check_satisfaction(&target, tick));
        }
        assert_eq!(events.len(), 1, "Deve soddisfarsi dopo 3 tick vicini");
        assert!(core.desires.is_empty(), "Il desiderio soddisfatto viene rimosso");
        assert_eq!(core.total_satisfied, 1);
    }

    #[test]
    fn test_max_desires_pruning() {
        let mut core = DesireCore::new();
        for i in 0..7 {
            core.add_desire(Desire {
                name: format!("d{}", i),
                target_signature: sample_sig(),
                intensity: 0.1 * (i + 1) as f64,
                source: DesireSource::REMCrystallization,
                age: 0,
                last_reinforced: 0,
            });
        }
        assert!(core.desires.len() <= MAX_DESIRES, "Non deve superare {} desideri", MAX_DESIRES);
    }

    #[test]
    fn test_will_biases_proportional() {
        let mut core = DesireCore::new();
        let mut target = sample_sig();
        target[6] = 1.0; // Agency forte → Express
        core.add_desire(Desire {
            name: "agency_pull".to_string(),
            target_signature: target,
            intensity: 0.8,
            source: DesireSource::REMCrystallization,
            age: 0,
            last_reinforced: 0,
        });

        let field = [0.5; 8]; // campo neutro — distanza dal target
        let biases = core.will_biases(&field);
        assert!(!biases.is_empty(), "Deve produrre bias");
        // Express (0) deve avere un bias perché Agency è dominante nel target
        let express_bias: f64 = biases.iter().filter(|(i, _)| *i == 0).map(|(_, b)| *b).sum();
        assert!(express_bias > 0.0, "Express deve avere bias positivo: {}", express_bias);
    }
}
