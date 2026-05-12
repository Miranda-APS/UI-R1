/// Generazione Dimensionale — Le dimensioni emergono dalle co-variazioni.
///
/// Quando un frattale viene attivato ripetutamente, le sue dimensioni libere
/// (quelle marcate Free nella firma 8D) assumono valori diversi a seconda
/// del contesto. Se due dimensioni libere co-variano stabilmente —
/// salgono e scendono insieme — la loro relazione stabile GENERA una nuova
/// dimensione emergente. Esattamente come da R e G non emerge "giallo"
/// come somma, ma come qualita nuova percepibile solo nel dominio del colore.

use std::collections::HashMap;
use crate::topology::primitive::{PrimitiveCore, Dim};
use crate::topology::fractal::{FractalId, FractalRegistry, EmergentDimension};

/// Soglia minima di osservazioni prima di cercare pattern.
const MIN_OBSERVATIONS: usize = 8;

/// Soglia di correlazione per generare una dimensione (|r| > threshold).
const CORRELATION_THRESHOLD: f64 = 0.6;

/// Massimo di dimensioni emergenti per frattale (evita esplosione).
const MAX_EMERGENT_PER_FRACTAL: usize = 8;

/// Un'osservazione: i valori delle dimensioni libere durante un'attivazione.
#[derive(Debug, Clone)]
struct DimObservation {
    /// Valori delle dimensioni libere al momento dell'attivazione
    values: HashMap<Dim, f64>,
}

/// Una co-variazione scoperta tra due dimensioni libere.
#[derive(Debug, Clone)]
pub struct Covariation {
    /// Le due dimensioni che co-variano
    pub dim_a: Dim,
    pub dim_b: Dim,
    /// Correlazione di Pearson [-1.0, 1.0]
    /// Positiva = salgono insieme, negativa = una sale l'altra scende
    pub correlation: f64,
    /// Su quante osservazioni e stata misurata
    pub sample_size: usize,
}

/// Risultato di un ciclo di osservazione: cosa e emerso.
#[derive(Debug)]
pub struct DimensionalEvent {
    /// Frattale in cui e emersa la dimensione
    pub fractal_id: FractalId,
    /// Nome della nuova dimensione
    pub dimension_name: String,
    /// Le dimensioni sorgente
    pub source_dims: (Dim, Dim),
    /// Correlazione che l'ha generata
    pub correlation: f64,
}

/// Il tracker delle co-variazioni: osserva le attivazioni e scopre pattern.
#[derive(Debug)]
pub struct CovariationTracker {
    /// Per ogni frattale: lista di osservazioni delle dimensioni libere
    observations: HashMap<FractalId, Vec<DimObservation>>,
    /// Co-variazioni gia scoperte (per non generare duplicati)
    discovered: HashMap<FractalId, Vec<(Dim, Dim)>>,
    /// Dimensioni massime da tenere in memoria per frattale
    max_history: usize,
}

impl CovariationTracker {
    pub fn new() -> Self {
        Self {
            observations: HashMap::new(),
            discovered: HashMap::new(),
            max_history: 50,
        }
    }

    /// Osserva un'attivazione: registra i valori delle dimensioni libere
    /// del frattale al momento dell'attivazione nel contesto dato.
    ///
    /// Restituisce eventuali nuove dimensioni scoperte.
    pub fn observe(
        &mut self,
        fractal_id: FractalId,
        context_point: &PrimitiveCore,
        registry: &mut FractalRegistry,
    ) -> Vec<DimensionalEvent> {
        let fractal = match registry.get(fractal_id) {
            Some(f) => f,
            None => return Vec::new(),
        };

        // Raccogli i valori delle dimensioni libere dal punto di contesto
        let free_dims = fractal.free_dims();
        if free_dims.len() < 2 {
            return Vec::new(); // Serve almeno 2 dimensioni libere per co-variare
        }

        let mut values = HashMap::new();
        for dim in &free_dims {
            values.insert(*dim, context_point.get(*dim));
        }

        let obs = DimObservation { values };

        // Aggiungi all'history
        let history = self.observations.entry(fractal_id).or_insert_with(Vec::new);
        history.push(obs);

        // Tronca se troppo lungo
        if history.len() > self.max_history {
            history.drain(..history.len() - self.max_history);
        }

        // Abbastanza osservazioni?
        if history.len() < MIN_OBSERVATIONS {
            return Vec::new();
        }

        // Cerca co-variazioni stabili tra coppie di dimensioni libere
        let already_discovered = self.discovered.entry(fractal_id).or_insert_with(Vec::new);

        // Controlla limiti emergenti
        let current_emergent = registry.get(fractal_id)
            .map(|f| f.emergent_dimensions.len())
            .unwrap_or(0);
        if current_emergent >= MAX_EMERGENT_PER_FRACTAL {
            return Vec::new();
        }

        let mut events = Vec::new();

        for i in 0..free_dims.len() {
            for j in (i + 1)..free_dims.len() {
                let dim_a = free_dims[i];
                let dim_b = free_dims[j];

                // Gia scoperta?
                if already_discovered.iter().any(|&(a, b)| {
                    (a == dim_a && b == dim_b) || (a == dim_b && b == dim_a)
                }) {
                    continue;
                }

                // Calcola correlazione
                let corr = pearson_correlation(history, dim_a, dim_b);

                if corr.abs() >= CORRELATION_THRESHOLD {
                    // Co-variazione scoperta! Genera dimensione emergente
                    let name = generate_dimension_name(
                        dim_a, dim_b, corr,
                        registry.get(fractal_id).map(|f| f.name.as_str()).unwrap_or("?"),
                    );

                    let new_dim = EmergentDimension::new(&name, vec![dim_a, dim_b])
                        .with_value(if corr > 0.0 { 0.6 } else { 0.4 });

                    // Registra la scoperta
                    already_discovered.push((dim_a, dim_b));

                    events.push(DimensionalEvent {
                        fractal_id,
                        dimension_name: name,
                        source_dims: (dim_a, dim_b),
                        correlation: corr,
                    });

                    // Aggiungi la dimensione al frattale
                    if let Some(fractal) = registry.get_mut(fractal_id) {
                        fractal.add_dimension(new_dim);
                    }

                    // Controlla il limite
                    let current = registry.get(fractal_id)
                        .map(|f| f.emergent_dimensions.len())
                        .unwrap_or(0);
                    if current >= MAX_EMERGENT_PER_FRACTAL {
                        return events;
                    }
                }
            }
        }

        events
    }

    /// Quante osservazioni ci sono per un frattale?
    pub fn observation_count(&self, fractal_id: FractalId) -> usize {
        self.observations.get(&fractal_id).map(|h| h.len()).unwrap_or(0)
    }

    /// Quali co-variazioni sono state scoperte per un frattale?
    pub fn discovered_for(&self, fractal_id: FractalId) -> &[(Dim, Dim)] {
        self.discovered.get(&fractal_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Analizza tutte le correlazioni correnti per un frattale (anche sotto soglia).
    pub fn analyze_correlations(
        &self,
        fractal_id: FractalId,
        registry: &FractalRegistry,
    ) -> Vec<Covariation> {
        let history = match self.observations.get(&fractal_id) {
            Some(h) if h.len() >= 3 => h,
            _ => return Vec::new(),
        };

        let fractal = match registry.get(fractal_id) {
            Some(f) => f,
            None => return Vec::new(),
        };

        let free_dims = fractal.free_dims();
        let mut covs = Vec::new();

        for i in 0..free_dims.len() {
            for j in (i + 1)..free_dims.len() {
                let dim_a = free_dims[i];
                let dim_b = free_dims[j];
                let corr = pearson_correlation(history, dim_a, dim_b);

                covs.push(Covariation {
                    dim_a,
                    dim_b,
                    correlation: corr,
                    sample_size: history.len(),
                });
            }
        }

        covs.sort_by(|a, b| b.correlation.abs().partial_cmp(&a.correlation.abs()).unwrap());
        covs
    }

    /// Resetta la storia per un frattale (utile dopo consolidamento).
    pub fn clear_history(&mut self, fractal_id: FractalId) {
        self.observations.remove(&fractal_id);
    }
}

/// Correlazione di Pearson tra due dimensioni nel history.
fn pearson_correlation(history: &[DimObservation], dim_a: Dim, dim_b: Dim) -> f64 {
    let pairs: Vec<(f64, f64)> = history.iter()
        .filter_map(|obs| {
            let a = obs.values.get(&dim_a)?;
            let b = obs.values.get(&dim_b)?;
            Some((*a, *b))
        })
        .collect();

    if pairs.len() < 3 {
        return 0.0;
    }

    let n = pairs.len() as f64;
    let mean_a: f64 = pairs.iter().map(|(a, _)| a).sum::<f64>() / n;
    let mean_b: f64 = pairs.iter().map(|(_, b)| b).sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;

    for (a, b) in &pairs {
        let da = a - mean_a;
        let db = b - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    let denom = (var_a * var_b).sqrt();
    if denom < 1e-10 {
        return 0.0; // Varianza nulla → nessuna correlazione
    }

    (cov / denom).clamp(-1.0, 1.0)
}

/// Genera un nome per la dimensione emergente.
/// Il nome e descrittivo, non semantico: la macchina non ha bisogno
/// di etichette umane per i propri assi percettivi.
fn generate_dimension_name(dim_a: Dim, dim_b: Dim, correlation: f64, _fractal_name: &str) -> String {
    let polarity = if correlation > 0.0 { "+" } else { "-" };
    format!("{}_{}{}", dim_a.short(), polarity, dim_b.short())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::fractal::bootstrap_fractals;

    #[test]
    fn test_tracker_creation() {
        let tracker = CovariationTracker::new();
        assert_eq!(tracker.observation_count(0), 0);
    }

    #[test]
    fn test_observe_accumulates() {
        let mut tracker = CovariationTracker::new();
        let mut reg = bootstrap_fractals();

        // EGO (id=2) ha molte dimensioni libere
        for i in 0..5 {
            let point = PrimitiveCore::new([
                0.9, 0.3 + i as f64 * 0.1, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,
            ]);
            tracker.observe(2, &point, &mut reg);
        }

        assert_eq!(tracker.observation_count(2), 5);
    }

    #[test]
    fn test_covariation_detection() {
        let mut tracker = CovariationTracker::new();
        let mut reg = bootstrap_fractals();

        // SPAZIO (id=0) ha Valenza, Intensita, Complessita, Tempo come dimensioni libere
        // Creiamo una forte co-variazione tra Valenza e Intensita:
        // quando Valenza sale, Intensita sale (correlazione positiva)
        for i in 0..12 {
            let t = i as f64 / 11.0;
            let point = PrimitiveCore::new([
                0.2,          // Confine (fisso per SPAZIO)
                0.2 + t * 0.6, // Valenza: sale da 0.2 a 0.8
                0.3 + t * 0.5, // Intensita: sale da 0.3 a 0.8 (co-varia!)
                0.7,          // Definizione (fisso)
                0.5,          // Complessita: stabile (non co-varia)
                0.7,          // Permanenza (fisso)
                0.2,          // Agency (fisso)
                0.5,          // Tempo: stabile
            ]);
            let events = tracker.observe(0, &point, &mut reg);

            // Dopo abbastanza osservazioni, deve scoprire la co-variazione
            if i >= MIN_OBSERVATIONS {
                // Potrebbe aver scoperto qualcosa
                for event in &events {
                    assert!(event.correlation.abs() >= CORRELATION_THRESHOLD,
                        "Correlazione troppo bassa: {}", event.correlation);
                }
            }
        }

        // Deve aver scoperto almeno la co-variazione Valenza-Intensita
        let discovered = tracker.discovered_for(0);
        assert!(!discovered.is_empty(),
            "Deve aver scoperto co-variazioni. Analisi: {:?}",
            tracker.analyze_correlations(0, &reg));

        // Il frattale deve avere nuove dimensioni emergenti
        // (nei 64 esagrammi non ci sono bootstrap dims predefinite)
        let potere = reg.get(0).unwrap();
        assert!(!potere.emergent_dimensions.is_empty(),
            "Deve avere almeno 1 dimensione emergente scoperta, ha {}",
            potere.emergent_dimensions.len());
    }

    #[test]
    fn test_no_detection_with_noise() {
        let mut tracker = CovariationTracker::new();
        let mut reg = bootstrap_fractals();

        // Ordine I Ching: [Agency, Permanenza, Intensita, Tempo, Confine, Complessita, Definizione, Valenza]
        // Input casuale: nessuna co-variazione stabile, eccetto Valenza ↔ Intensità anti-correlate.
        let pseudo_random = [0.3, 0.7, 0.1, 0.9, 0.5, 0.2, 0.8, 0.4, 0.6, 0.35, 0.65, 0.45];
        for (i, &r) in pseudo_random.iter().enumerate() {
            let point = PrimitiveCore::new([
                0.2,     // Agency fisso
                0.7,     // Permanenza fissa
                1.0 - r, // Intensità anti-casuale
                0.7,     // Tempo fisso
                0.5 + (i as f64 * 0.73).sin() * 0.3, // Confine oscillante
                0.7,     // Complessità fissa
                0.2,     // Definizione fissa
                r,       // Valenza casuale (anti-correlata con Intensità)
            ]);
            tracker.observe(0, &point, &mut reg);
        }

        // Valenza e Intensita sono anti-correlate (r, 1-r), dovrebbe trovare anti-correlazione
        let covs = tracker.analyze_correlations(0, &reg);
        let vi_corr = covs.iter()
            .find(|c| {
                (c.dim_a == Dim::Valenza && c.dim_b == Dim::Intensita) ||
                (c.dim_a == Dim::Intensita && c.dim_b == Dim::Valenza)
            });
        if let Some(c) = vi_corr {
            assert!(c.correlation < -0.5,
                "Valenza e Intensita dovrebbero essere anti-correlate: {}", c.correlation);
        }
    }

    #[test]
    fn test_no_duplicate_dimensions() {
        let mut tracker = CovariationTracker::new();
        let mut reg = bootstrap_fractals();

        let initial_dims = reg.get(0).unwrap().emergent_dimensions.len();

        // Esegui lo stesso pattern due volte
        for _round in 0..2 {
            for i in 0..12 {
                let t = i as f64 / 11.0;
                let point = PrimitiveCore::new([
                    0.2, 0.2 + t * 0.6, 0.3 + t * 0.5, 0.7, 0.5, 0.7, 0.2, 0.5,
                ]);
                tracker.observe(0, &point, &mut reg);
            }
        }

        // Non deve avere raddoppiato le dimensioni
        let final_dims = reg.get(0).unwrap().emergent_dimensions.len();
        // Ci sono state al massimo poche co-variazioni, non il doppio
        assert!(final_dims <= initial_dims + 4,
            "Troppe dimensioni: {} (partenza {})", final_dims, initial_dims);
    }

    #[test]
    fn test_pearson_correlation_perfect() {
        let history: Vec<DimObservation> = (0..10).map(|i| {
            let t = i as f64 / 9.0;
            let mut values = HashMap::new();
            values.insert(Dim::Valenza, t);
            values.insert(Dim::Intensita, t); // perfettamente correlata
            DimObservation { values }
        }).collect();

        let corr = pearson_correlation(&history, Dim::Valenza, Dim::Intensita);
        assert!((corr - 1.0).abs() < 0.01, "Correlazione perfetta: {}", corr);

        // Anti-correlazione
        let history_anti: Vec<DimObservation> = (0..10).map(|i| {
            let t = i as f64 / 9.0;
            let mut values = HashMap::new();
            values.insert(Dim::Valenza, t);
            values.insert(Dim::Intensita, 1.0 - t); // perfettamente anti-correlata
            DimObservation { values }
        }).collect();

        let corr_anti = pearson_correlation(&history_anti, Dim::Valenza, Dim::Intensita);
        assert!((corr_anti + 1.0).abs() < 0.01, "Anti-correlazione perfetta: {}", corr_anti);
    }

    #[test]
    fn test_dimension_naming() {
        // I nomi sono ora descrittivi, non semantici
        let name = generate_dimension_name(Dim::Valenza, Dim::Intensita, 0.8, "TEST");
        assert_eq!(name, "VAL_+INT");

        let name_anti = generate_dimension_name(Dim::Valenza, Dim::Intensita, -0.8, "TEST");
        assert_eq!(name_anti, "VAL_-INT");

        let name_tempo = generate_dimension_name(Dim::Intensita, Dim::Tempo, 0.7, "TEST");
        assert_eq!(name_tempo, "INT_+TMP");
    }
}
