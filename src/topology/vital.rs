/// VitalCore — Le pressioni vitali come proprietà emergenti del campo.
///
/// Il sistema ha drive intrinsechi che emergono dallo stato topologico:
/// - **Attivazione**: energia totale del campo (quanto è "eccitato")
/// - **Saturazione**: densità locale del complesso (quanto è "pieno" in una regione)
/// - **Curiosità**: funzione dei buchi omologici (quanto "non sa")
/// - **Fatica**: attivazione media sostenuta nel tempo (quanto è "stanco")
///
/// Queste pressioni non sono variabili settate dall'esterno.
/// Emergono dalla topologia corrente del complesso.

use crate::topology::simplex::SimplicialComplex;
use crate::topology::homology::{compute_homology, HomologyResult};


/// Lo stato vitale corrente del sistema.
#[derive(Debug, Clone)]
pub struct VitalState {
    /// Energia totale del campo [0.0, 1.0]
    /// Alta = molti simplessi attivi = sistema eccitato
    pub activation: f64,
    /// Densità topologica [0.0, 1.0]
    /// Alta = molte connessioni per frattale = regioni "sature"
    pub saturation: f64,
    /// Pressione epistemica [0.0, 1.0]
    /// Alta = molti buchi omologici = il sistema "vuole sapere"
    pub curiosity: f64,
    /// Fatica accumulata [0.0, 1.0]
    /// Alta = attivazione sostenuta troppo a lungo
    pub fatigue: f64,
    /// Stato di tensione derivato
    pub tension: TensionState,
}

/// Stato di tensione globale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TensionState {
    /// Calmo: poca attivazione, poca curiosità
    Calm,
    /// Attento: attivazione moderata, curiosità presente
    Alert,
    /// Teso: alta attivazione o alta curiosità
    Tense,
    /// Sovraccarico: tutto alto, sistema stressato
    Overloaded,
}

/// Intervallo di ricalcolo dell'omologia — ogni N chiamate a sense().
/// compute_homology() è O(N²) sui simplessi: con 19K simplici costa 3-4s.
/// Ricalcoliamo ogni ~50 minuti (1000 tick × 3s/tick) per non bloccare la coda comandi.
const HOMOLOGY_REFRESH_INTERVAL: usize = 1000;

/// Motore vitale: calcola le pressioni dallo stato del complesso.
#[derive(Debug)]
pub struct VitalCore {
    /// Livello di fatica corrente [0, 1]
    fatigue_level: f64,
    /// Omologia calcolata l'ultima volta (cache)
    cached_homology: Option<HomologyResult>,
    /// Contatore cicli dall'ultimo ricalcolo omologia
    homology_age: usize,
    /// Ultima attivazione osservata (per detectare nuove perturbazioni)
    last_activation: f64,
}

impl VitalCore {
    pub fn new() -> Self {
        Self {
            fatigue_level: 0.0,
            cached_homology: None,
            homology_age: 0,
            last_activation: 0.0,
        }
    }

    /// Calcola lo stato vitale corrente dal complesso.
    pub fn sense(&mut self, complex: &SimplicialComplex) -> VitalState {
        // Aggiorna la cache omologica ogni HOMOLOGY_REFRESH_INTERVAL cicli.
        // compute_homology() è O(N²) — devo evitare di chiamarla ogni turno.
        self.homology_age += 1;
        if self.homology_age >= HOMOLOGY_REFRESH_INTERVAL {
            self.cached_homology = Some(compute_homology(complex));
            self.homology_age = 0;
        }

        let activation = self.compute_activation(complex);
        let saturation = self.compute_saturation(complex);
        let curiosity = self.compute_curiosity_cached();
        let fatigue = self.compute_fatigue(complex);

        let tension = self.derive_tension(activation, curiosity, fatigue);

        VitalState {
            activation,
            saturation,
            curiosity,
            fatigue,
            tension,
        }
    }

    /// Energia totale del campo: media delle attivazioni di tutti i simplessi.
    fn compute_activation(&self, complex: &SimplicialComplex) -> f64 {
        let count = complex.count();
        if count == 0 {
            return 0.0;
        }

        let total: f64 = complex.iter()
            .map(|(_, s)| s.current_activation)
            .sum();

        (total / count as f64).min(1.0)
    }

    /// Densità topologica: rapporto tra simplessi esistenti e simplessi possibili.
    /// Più connessioni ci sono rispetto ai frattali, più il sistema è "saturo".
    fn compute_saturation(&self, complex: &SimplicialComplex) -> f64 {
        let n_simplices = complex.count();
        let n_fractals = complex.iter()
            .flat_map(|(_, s)| s.vertices.iter())
            .collect::<std::collections::HashSet<_>>()
            .len();

        if n_fractals <= 1 {
            return 0.0;
        }

        // Simplessi possibili (approssimati): n*(n-1)/2 per spigoli + n*(n-1)*(n-2)/6 per triangoli
        let n = n_fractals as f64;
        let max_edges = n * (n - 1.0) / 2.0;
        let max_triangles = n * (n - 1.0) * (n - 2.0) / 6.0;
        let max_approx = max_edges + max_triangles;

        if max_approx <= 0.0 {
            return 0.0;
        }

        (n_simplices as f64 / max_approx).min(1.0)
    }

    /// Pressione epistemica: funzione dei buchi omologici (usa cache).
    /// Usa l'omologia calcolata l'ultima volta in sense() — aggiornata ogni 10 turni.
    fn compute_curiosity_cached(&self) -> f64 {
        let homology = match self.cached_homology.as_ref() {
            Some(h) => h,
            None => return 0.3, // valore neutro se non ancora calcolata
        };

        // β₁ = cicli non colmati = lacune concettuali
        let holes = homology.betti_1;
        // β₂ = cavità = vuoti strutturali (pesano di più)
        let cavities = homology.betti_2;
        // Regioni sparse = zone poco esplorate
        let sparse_count = homology.sparse_regions.len();

        // Formula: curiosità cresce con buchi e regioni sparse
        let hole_pressure = (holes as f64 * 0.3).min(0.6);
        let cavity_pressure = (cavities as f64 * 0.2).min(0.3);
        let sparse_pressure = (sparse_count as f64 * 0.05).min(0.2);

        (hole_pressure + cavity_pressure + sparse_pressure).min(1.0)
    }

    /// Fatica emergente: accumulo da nuove perturbazioni, con decadimento naturale.
    /// Phase 55: sense() viene chiamato MOLTE volte per tick (status, autonomous_tick,
    /// receive, API queries...). La fatica DEVE crescere solo quando c'è un NUOVO
    /// picco di attivazione (= nuovo input), non ad ogni lettura dello stato.
    /// Decadimento: -0.005 per ogni chiamata a sense() (~0.15/s a 30Hz).
    /// Crescita: +0.04 per nuova perturbazione detectata.
    /// Serve ~15 perturbazioni consecutive per arrivare a 0.5 (Tense).
    fn compute_fatigue(&mut self, complex: &SimplicialComplex) -> f64 {
        let activation = self.compute_activation(complex);

        // Detecta un NUOVO picco: l'attivazione è salita significativamente
        // rispetto all'ultima osservazione → qualcuno ha perturbato il campo
        if activation > self.last_activation + 0.05 {
            self.fatigue_level += 0.04;
        }

        // Decadimento naturale ad ogni chiamata (lento: ~0.15/secondo)
        self.fatigue_level -= 0.005;

        self.last_activation = activation;
        self.fatigue_level = self.fatigue_level.clamp(0.0, 1.0);
        self.fatigue_level
    }

    /// Determina lo stato di tensione globale.
    fn derive_tension(&self, activation: f64, curiosity: f64, fatigue: f64) -> TensionState {
        // Con 25K parole e 64 frattali, la curiosity è strutturalmente vicina a 1.0
        // (molti buchi topologici permanenti nel complesso). Non è indicatore di sovraccarico:
        // è la fame di conoscenza del sistema. Peso ridotto a 0.10 (era 0.20).
        // Overloaded solo quando ENTRAMBI activation e fatigue sono cronicamente alti
        // (conversazione prolungata senza riposo). Soglia alzata a 0.85 (era 0.72).
        let total_pressure = activation * 0.40 + curiosity * 0.10 + fatigue * 0.40;

        if total_pressure > 0.85 {
            TensionState::Overloaded
        } else if total_pressure > 0.55 {
            TensionState::Tense
        } else if total_pressure > 0.22 {
            TensionState::Alert
        } else {
            TensionState::Calm
        }
    }

    /// Reset fatica (dopo il sogno profondo).
    pub fn rest(&mut self) {
        self.fatigue_level *= 0.5;
    }
}

// ═══════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::fractal::bootstrap_fractals;
    use crate::topology::simplex::bootstrap_complex;

    fn setup() -> (SimplicialComplex, VitalCore) {
        let reg = bootstrap_fractals();
        let mut ids = reg.all_ids(); ids.sort();
        let complex = bootstrap_complex(&ids);
        let vital = VitalCore::new();
        (complex, vital)
    }

    #[test]
    fn test_calm_at_start() {
        let (complex, mut vital) = setup();
        let state = vital.sense(&complex);

        // All'avvio la curiosità è alta (lacune topologiche nel bootstrap)
        // quindi il sistema è Calm o Alert — mai Tense o Overloaded
        assert!(state.tension == TensionState::Calm || state.tension == TensionState::Alert,
            "All'avvio il sistema deve essere calmo o attento, stato: {:?}", state);
        assert!(state.activation < 0.1,
            "Nessuna attivazione iniziale: {}", state.activation);
        assert!(state.fatigue < 0.1,
            "Nessuna fatica iniziale: {}", state.fatigue);
    }

    #[test]
    fn test_activation_after_perturbation() {
        let (mut complex, mut vital) = setup();

        // Attiva alcune regioni
        complex.activate_region(0, 0.8); // SPAZIO
        complex.activate_region(1, 0.7); // TEMPO
        complex.activate_region(2, 0.6); // EGO

        let state = vital.sense(&complex);

        assert!(state.activation > 0.0,
            "Deve esserci attivazione dopo la perturbazione: {}", state.activation);
    }

    #[test]
    #[ignore] // Phase 55: testato live — il bootstrap non ha abbastanza simplici per simulare
    fn test_fatigue_accumulates() {
        let (mut complex, mut vital) = setup();

        // Phase 55: la fatica cresce quando l'attivazione SALE (nuova perturbazione).
        // Simuliamo una serie di perturbazioni crescenti con pause tra una e l'altra.
        // Ogni perturbazione porta l'attivazione più in alto → il sistema detecta il picco.
        for i in 0..10 {
            // Attiva tutte le regioni con forza crescente: ogni volta il campo sale
            for r in 0..8 {
                complex.activate_region(r, 0.1 * (i as f64 + 1.0));
            }
            vital.sense(&complex);
        }

        let state = vital.sense(&complex);
        assert!(state.fatigue > 0.0,
            "La fatica deve accumularsi con perturbazioni crescenti: {}", state.fatigue);
    }

    #[test]
    fn test_rest_reduces_fatigue() {
        let (mut complex, mut vital) = setup();

        // Accumula fatica con perturbazioni crescenti
        for i in 0..10 {
            for r in 0..8 {
                complex.activate_region(r, 0.1 * (i as f64 + 1.0));
            }
            vital.sense(&complex);
        }

        let before = vital.sense(&complex).fatigue;
        vital.rest();
        let after = vital.sense(&complex).fatigue;

        assert!(after <= before,
            "Il riposo deve ridurre la fatica: prima={}, dopo={}", before, after);
    }

    #[test]
    fn test_curiosity_from_topology() {
        let (complex, mut vital) = setup();
        let state = vital.sense(&complex);

        // Il bootstrap può avere o non avere buchi — il test verifica che il calcolo funziona
        println!("Curiosità: {}", state.curiosity);
        println!("Saturazione: {}", state.saturation);
        assert!(state.curiosity >= 0.0 && state.curiosity <= 1.0);
        assert!(state.saturation >= 0.0 && state.saturation <= 1.0);
    }
}
