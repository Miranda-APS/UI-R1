/// Crescita Strutturale — Il sistema crea nuovi frattali e connessioni.
///
/// Il complesso non e statico. Se un concetto ricorrente non rientra
/// in nessun frattale esistente, il sistema ne crea uno nuovo.
/// Se due concetti vengono attivati spesso insieme, nasce un simplesso.
///
/// La crescita e lenta e conservativa: servono molte osservazioni
/// prima che il sistema "decida" di creare qualcosa di nuovo.
/// Questo previene la proliferazione incontrollata.

use std::collections::HashMap;
use crate::topology::fractal::{FractalId, FractalRegistry, DimConstraint, EmergentDimension};
use crate::topology::simplex::{SimplicialComplex, SharedFace};
use crate::topology::primitive::{PrimitiveCore, Dim};
use crate::topology::lexicon::Lexicon;

/// Un concetto candidato a diventare un nuovo frattale.
#[derive(Debug, Clone)]
struct ConceptCandidate {
    /// Firma dimensionale media delle osservazioni
    signature: [f64; 8],
    /// Quante volte e stato osservato
    observation_count: u64,
    /// Da quali input e emerso
    origins: Vec<String>,
    /// Affinita massima con frattali esistenti (bassa = veramente nuovo)
    max_affinity: f64,
}

/// Evento di crescita: cosa e successo nel sistema.
#[derive(Debug, Clone)]
pub enum GrowthEvent {
    /// Un nuovo frattale e stato creato
    NewFractal {
        id: FractalId,
        name: String,
        observation_count: u64,
    },
    /// Un nuovo simplesso e stato creato tra frattali co-attivati
    NewConnection {
        fractal_a: FractalId,
        fractal_b: FractalId,
    },
    /// Un sotto-frattale e stato creato
    NewSubfractal {
        parent_id: FractalId,
        child_id: FractalId,
        name: String,
    },
}

/// Il tracker della crescita strutturale.
#[derive(Debug)]
pub struct GrowthTracker {
    /// Candidati: firme dimensionali che non rientrano nei frattali esistenti
    candidates: Vec<ConceptCandidate>,
    /// Co-attivazioni: coppie di frattali attivati insieme
    coactivations: HashMap<(FractalId, FractalId), u64>,
    /// Soglia minima di osservazioni per creare un frattale
    pub min_observations: u64,
    /// Soglia massima di affinita per considerare un concetto "nuovo"
    pub novelty_threshold: f64,
    /// Soglia di co-attivazione per creare un simplesso
    pub coactivation_threshold: u64,
    /// Massimo di frattali creabili (per evitare proliferazione)
    pub max_created: usize,
    /// Frattali gia creati
    created_count: usize,
}

impl GrowthTracker {
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            coactivations: HashMap::new(),
            min_observations: 10,
            novelty_threshold: 0.5, // Se affinita < 0.5, e "nuovo"
            coactivation_threshold: 8,
            max_created: 20,
            created_count: 0,
        }
    }

    /// Osserva un input: se la firma non rientra nei frattali esistenti,
    /// accumula come candidato.
    pub fn observe(
        &mut self,
        signature: &PrimitiveCore,
        input: &str,
        registry: &FractalRegistry,
    ) {
        // Trova l'affinita massima con i frattali esistenti
        let max_affinity = registry.iter()
            .map(|(_, f)| f.affinity(signature))
            .fold(0.0f64, f64::max);

        if max_affinity < self.novelty_threshold {
            // Questo input non rientra bene in nessun frattale → candidato
            // Cerca se esiste gia un candidato simile
            let mut found = false;
            for candidate in &mut self.candidates {
                let sim = signature_similarity(&candidate.signature, signature.values());
                if sim > 0.7 {
                    // Aggiorna candidato esistente
                    candidate.observation_count += 1;
                    // Media mobile della firma
                    let n = candidate.observation_count as f64;
                    for d in 0..8 {
                        candidate.signature[d] = (candidate.signature[d] * (n - 1.0) + signature.values()[d]) / n;
                    }
                    if candidate.origins.len() < 5 {
                        candidate.origins.push(input.to_string());
                    }
                    candidate.max_affinity = candidate.max_affinity.min(max_affinity);
                    found = true;
                    break;
                }
            }

            if !found {
                let mut sig = [0.0; 8];
                sig.copy_from_slice(signature.values());
                self.candidates.push(ConceptCandidate {
                    signature: sig,
                    observation_count: 1,
                    origins: vec![input.to_string()],
                    max_affinity,
                });
            }
        }
    }

    /// Osserva co-attivazione di frattali.
    pub fn observe_coactivation(&mut self, active_fractals: &[FractalId]) {
        for i in 0..active_fractals.len() {
            for j in (i + 1)..active_fractals.len() {
                let a = active_fractals[i].min(active_fractals[j]);
                let b = active_fractals[i].max(active_fractals[j]);
                *self.coactivations.entry((a, b)).or_insert(0) += 1;
            }
        }
    }

    /// Tenta di far crescere il sistema: crea nuovi frattali e connessioni
    /// se le soglie sono raggiunte.
    pub fn try_grow(
        &mut self,
        registry: &mut FractalRegistry,
        complex: &mut SimplicialComplex,
        lexicon: &Lexicon,
    ) -> Vec<GrowthEvent> {
        let mut events = Vec::new();

        // 1. Crea nuovi frattali da candidati maturi
        if self.created_count < self.max_created {
            let mature: Vec<usize> = self.candidates.iter().enumerate()
                .filter(|(_, c)| c.observation_count >= self.min_observations)
                .map(|(i, _)| i)
                .collect();

            // Processa in ordine inverso per non invalidare gli indici
            for &idx in mature.iter().rev() {
                if self.created_count >= self.max_created {
                    break;
                }

                let candidate = self.candidates.remove(idx);
                let event = create_fractal_from_candidate(
                    &candidate, registry, complex,
                );
                if let Some(e) = event {
                    self.created_count += 1;
                    events.push(e);
                }
            }
        }

        // 2. Crea simplessi da co-attivazioni ricorrenti
        let strong_pairs: Vec<((FractalId, FractalId), u64)> = self.coactivations.iter()
            .filter(|(_, &count)| count >= self.coactivation_threshold)
            .map(|(&pair, &count)| (pair, count))
            .collect();

        for ((a, b), count) in strong_pairs {
            // Controlla se esiste gia un simplesso tra loro
            if complex.shared_simplices(a, b).is_empty() {
                // Crea un nuovo simplesso basato sulla co-attivazione
                let strength = (count as f64 / 20.0).min(1.0);
                complex.add_simplex(
                    vec![a, b],
                    vec![SharedFace::from_property("co-attivazione", strength)],
                );
                events.push(GrowthEvent::NewConnection {
                    fractal_a: a,
                    fractal_b: b,
                });
            }
            // Reset il contatore (per evitare creazioni multiple)
            self.coactivations.remove(&(a, b));
        }

        // 3. Pulizia candidati troppo vecchi o deboli
        self.candidates.retain(|c| c.observation_count < self.min_observations * 5);

        events
    }

    /// Quanti candidati ci sono in attesa.
    pub fn pending_candidates(&self) -> usize {
        self.candidates.len()
    }

    /// Quanti frattali sono stati creati.
    pub fn created_fractal_count(&self) -> usize {
        self.created_count
    }

    /// Statistiche sulle co-attivazioni piu forti.
    pub fn top_coactivations(&self) -> Vec<((FractalId, FractalId), u64)> {
        let mut pairs: Vec<_> = self.coactivations.iter()
            .map(|(&pair, &count)| (pair, count))
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(10);
        pairs
    }
}

/// Crea un frattale da un candidato maturo.
fn create_fractal_from_candidate(
    candidate: &ConceptCandidate,
    registry: &mut FractalRegistry,
    complex: &mut SimplicialComplex,
) -> Option<GrowthEvent> {
    // Determina quali dimensioni sono "salienti" (lontane dal neutro 0.5)
    let mut constraints = [DimConstraint::Free; 8];
    let mut salient_dims = Vec::new();

    for d in 0..8 {
        let deviation = (candidate.signature[d] - 0.5).abs();
        if deviation > 0.15 {
            constraints[d] = DimConstraint::Fixed(candidate.signature[d]);
            salient_dims.push(Dim::ALL[d]);
        }
    }

    // Serve almeno una dimensione saliente per creare un frattale
    if salient_dims.is_empty() {
        return None;
    }

    // Genera un nome dal candidato
    let name = generate_fractal_name(&salient_dims, &candidate.origins);

    let id = registry.register(&name, constraints);

    // Trova il frattale esistente piu vicino e crea un simplesso
    let point = PrimitiveCore::new(candidate.signature);
    if let Some(nearest) = registry.nearest(&point) {
        if nearest != id {
            // Crea una connessione col frattale piu vicino
            let shared_dims: Vec<_> = salient_dims.iter()
                .filter(|d| {
                    if let Some(f) = registry.get(nearest) {
                        matches!(f.signature[d.index()], DimConstraint::Fixed(_))
                    } else {
                        false
                    }
                })
                .collect();

            let faces = if shared_dims.is_empty() {
                vec![SharedFace::from_property("emergenza", 0.5)]
            } else {
                vec![SharedFace::from_dim(*shared_dims[0], 0.6)]
            };

            complex.add_simplex(vec![id, nearest], faces);
        }
    }

    Some(GrowthEvent::NewFractal {
        id,
        name: name.clone(),
        observation_count: candidate.observation_count,
    })
}

/// Genera un nome per un nuovo frattale dalle dimensioni salienti.
fn generate_fractal_name(dims: &[Dim], origins: &[String]) -> String {
    // Se abbiamo un'origine testuale, usala come base
    if let Some(first_origin) = origins.first() {
        let word = first_origin.split_whitespace()
            .filter(|w| w.len() > 3)
            .next()
            .unwrap_or("concetto");
        return word.to_uppercase();
    }

    // Altrimenti genera dal profilo dimensionale
    let dim_names: Vec<&str> = dims.iter().map(|d| d.name()).collect();
    format!("EMERGENTE_{}", dim_names.join("_"))
}

/// Similitudine tra due firme dimensionali.
fn signature_similarity(a: &[f64; 8], b: &[f64; 8]) -> f64 {
    let sum_sq: f64 = a.iter().zip(b.iter())
        .map(|(va, vb)| (va - vb) * (va - vb))
        .sum();
    1.0 - (sum_sq.sqrt() / 8.0f64.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::fractal::bootstrap_fractals;
    use crate::topology::simplex::bootstrap_complex;
    use crate::topology::lexicon::Lexicon;

    fn setup() -> (FractalRegistry, SimplicialComplex, Lexicon) {
        let reg = bootstrap_fractals();
        let mut ids = reg.all_ids(); ids.sort();
        let complex = bootstrap_complex(&ids);
        let lexicon = Lexicon::bootstrap();
        (reg, complex, lexicon)
    }

    #[test]
    fn test_tracker_creation() {
        let tracker = GrowthTracker::new();
        assert_eq!(tracker.pending_candidates(), 0);
        assert_eq!(tracker.created_fractal_count(), 0);
    }

    #[test]
    fn test_observe_novel_concept() {
        let (reg, _, _) = setup();
        let mut tracker = GrowthTracker::new();
        tracker.novelty_threshold = 0.75; // Soglia alta per il test

        // Ordine I Ching: [Agency, Perm, Intens, Tempo, Confine, Compl, Defin, Valenza]
        // Firma lontana da tutti i 64 esagrammi (nessuna dim fissa vicina).
        // Agency=0.3, Perm=0.85 (lontano da 0.10), Intens=0.0, Tempo=0.0,
        // Confine=0.03 (lontano da 0.30), Compl=0.1, Defin=0.0, Val=0.1
        let novel = PrimitiveCore::new([0.3, 0.85, 0.0, 0.0, 0.03, 0.1, 0.0, 0.1]);
        tracker.observe(&novel, "concetto strano", &reg);

        assert!(tracker.pending_candidates() > 0,
            "Un concetto nuovo deve diventare candidato");
    }

    #[test]
    fn test_observe_familiar_concept() {
        let (reg, _, _) = setup();
        let mut tracker = GrowthTracker::new();

        // Ordine I Ching: Confine=4. Firma vicina a SPAZIO (Confine alto).
        let familiar = PrimitiveCore::new([0.5, 0.5, 0.5, 0.5, 0.9, 0.5, 0.5, 0.5]);
        tracker.observe(&familiar, "qualcosa di spaziale", &reg);

        // Non dovrebbe diventare candidato (troppo simile a SPAZIO)
        assert_eq!(tracker.pending_candidates(), 0,
            "Un concetto familiare non deve diventare candidato");
    }

    #[test]
    fn test_grow_after_observations() {
        let (mut reg, mut complex, lexicon) = setup();
        let mut tracker = GrowthTracker::new();
        tracker.min_observations = 3; // Soglia bassa per il test
        tracker.novelty_threshold = 0.75; // Soglia alta per il test

        // Ordine I Ching — stessa firma lontana del test sopra.
        let novel = PrimitiveCore::new([0.3, 0.85, 0.0, 0.0, 0.03, 0.1, 0.0, 0.1]);

        // Osserva lo stesso concetto molte volte
        for i in 0..5 {
            tracker.observe(&novel, &format!("strano {}", i), &reg);
        }

        let initial_count = reg.count();
        let events = tracker.try_grow(&mut reg, &mut complex, &lexicon);

        assert!(!events.is_empty(), "Dopo abbastanza osservazioni deve crescere");
        assert!(reg.count() > initial_count, "Deve esserci un nuovo frattale");
    }

    #[test]
    fn test_coactivation_creates_connection() {
        let (mut reg, mut complex, lexicon) = setup();
        let mut tracker = GrowthTracker::new();
        tracker.coactivation_threshold = 3; // Soglia bassa per test

        // Trova due frattali non connessi (se possibile)
        // Usiamo ID alti (sotto-frattali) che potrebbero non essere connessi
        let ids = reg.all_ids();
        if ids.len() >= 8 {
            let a = ids[6]; // Sotto-frattale
            let b = ids[7]; // Altro sotto-frattale

            // Se non sono connessi, testa la co-attivazione
            if complex.shared_simplices(a, b).is_empty() {
                for _ in 0..5 {
                    tracker.observe_coactivation(&[a, b]);
                }

                let events = tracker.try_grow(&mut reg, &mut complex, &lexicon);
                let has_connection = events.iter().any(|e| matches!(e, GrowthEvent::NewConnection { .. }));
                assert!(has_connection, "Co-attivazione ripetuta deve creare connessione");
            }
        }
    }

    #[test]
    fn test_max_created_limit() {
        let (mut reg, mut complex, lexicon) = setup();
        let mut tracker = GrowthTracker::new();
        tracker.min_observations = 2;
        tracker.max_created = 2;

        // Crea molti candidati diversi
        for i in 0..5 {
            let v = 0.1 * i as f64;
            let novel = PrimitiveCore::new([v, v, 0.1, 0.1, 0.9, 0.9, v, 0.1]);
            for _ in 0..3 {
                tracker.observe(&novel, &format!("diverso {}", i), &reg);
            }
        }

        let events = tracker.try_grow(&mut reg, &mut complex, &lexicon);
        let new_fractals = events.iter()
            .filter(|e| matches!(e, GrowthEvent::NewFractal { .. }))
            .count();

        assert!(new_fractals <= 2, "Non deve superare max_created: {}", new_fractals);
    }

    #[test]
    fn test_signature_similarity() {
        let a = [0.5; 8];
        let b = [0.5; 8];
        assert!((signature_similarity(&a, &b) - 1.0).abs() < 0.01);

        let c = [0.0; 8];
        let d = [1.0; 8];
        assert!(signature_similarity(&c, &d) < 0.5);
    }
}
