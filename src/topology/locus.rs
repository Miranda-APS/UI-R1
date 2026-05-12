/// Locus — La posizione del sistema nel suo universo concettuale.
///
/// Prometeo non osserva il campo dall'alto. Ha una posizione.
/// Da quella posizione vede il mondo con chiarezza decrescente.
/// Ogni input lo muove: a volte scivola lungo una geodetica (continuazione),
/// a volte salta in una regione lontana (cambio tema).
/// Nel sogno, il locus drifta seguendo i gradienti residui.

use std::collections::HashMap;
use crate::topology::primitive::{PrimitiveCore, Dim};
use crate::topology::fractal::{FractalId, FractalRegistry, DimConstraint};
use crate::topology::simplex::SimplicialComplex;
use crate::topology::composition::PhrasePattern;
use crate::topology::navigation::{geodesic_distance, distance_map, find_geodesic};
use crate::topology::dream::SleepPhase;

/// La posizione del sistema nel suo spazio concettuale.
pub struct Locus {
    /// Frattale in cui il sistema si trova (None = non ancora collocato)
    pub position: Option<FractalId>,
    /// Raggio di percezione: quanto lontano vede il sistema
    pub horizon: f64,
    /// Mappa delle distanze dal locus corrente (cache)
    distances: HashMap<FractalId, f64>,
    /// Traccia del percorso recente (ultimi N loci visitati)
    pub trail: Vec<FractalId>,
    /// Soglia di salto: oltre questa distanza, e un cambio tema
    pub jump_threshold: f64,
    /// Capacita massima del trail
    trail_capacity: usize,
    /// Sub-locus: posizione nelle dimensioni libere del frattale corrente.
    /// Se il frattale ha N dimensioni libere, il sub-locus e un vettore
    /// N-dimensionale — la "stanza" dentro il concetto.
    pub sub_position: HashMap<Dim, f64>,
}

// ═══════════════════════════════════════════════════════════════
// SUB-LOCUS: posizione dentro il frattale
// ═══════════════════════════════════════════════════════════════

/// Il sub-locus descrive dove il sistema si trova DENTRO un frattale.
/// Un frattale con 4 dimensioni libere e una stanza con 4 gradi di
/// liberta — il sub-locus dice dove il sistema sta nella stanza.
#[derive(Debug, Clone)]
pub struct SubLocusView {
    /// Il frattale in cui siamo
    pub fractal_id: FractalId,
    /// Nome del frattale
    pub fractal_name: String,
    /// Posizione in ogni dimensione libera
    pub coordinates: Vec<(Dim, f64)>,
    /// Gradi di liberta (numero dimensioni libere)
    pub degrees_of_freedom: usize,
}

// ═══════════════════════════════════════════════════════════════
// PROIEZIONE OLOGRAFICA: l'universo visto da dentro un frattale
// ═══════════════════════════════════════════════════════════════

/// Proiezione olografica: come appare l'intero universo visto da
/// dentro un frattale. Ogni frattale contiene una vista compressa
/// di tutto — TEMPO visto da dentro SPAZIO e diverso da TEMPO
/// visto da dentro EGO.
#[derive(Debug, Clone)]
pub struct HolographicProjection {
    /// Frattale da cui si proietta
    pub from: FractalId,
    /// Nome del frattale sorgente
    pub from_name: String,
    /// Come appare ogni altro frattale dalla prospettiva di `from`.
    /// Prossimita, distorsione, rilevanza — tutto relativo.
    pub projections: Vec<FractalProjection>,
}

/// Come appare un singolo frattale nella proiezione olografica.
#[derive(Debug, Clone)]
pub struct FractalProjection {
    /// Frattale proiettato
    pub fractal_id: FractalId,
    /// Nome del frattale
    pub name: String,
    /// Prossimita topologica (da distance_map) [0.0 = lontanissimo, 1.0 = adiacente]
    pub proximity: f64,
    /// Risonanza dimensionale: quante dimensioni condividono lo stesso vincolo
    pub dimensional_resonance: f64,
    /// Distorsione: quanto il frattale "appare diverso" visto da qui
    /// rispetto al suo centro oggettivo. Dipende dalle dimensioni
    /// libere del frattale sorgente.
    pub distortion: f64,
    /// Il centro apparente: come appare il frattale target
    /// filtrato attraverso le dimensioni libere del sorgente
    pub apparent_center: PrimitiveCore,
}

/// Risultato di un movimento nel campo.
#[derive(Debug)]
pub struct Movement {
    /// Da dove siamo partiti
    pub from: Option<FractalId>,
    /// Dove siamo arrivati
    pub to: FractalId,
    /// Tipo di movimento
    pub kind: MovementKind,
    /// Cammino percorso (vuoto se salto)
    pub path: Vec<FractalId>,
    /// Distanza percorsa
    pub distance: f64,
}

/// Tipo di movimento nel campo concettuale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementKind {
    /// Primo posizionamento (il sistema non era ancora da nessuna parte)
    Origin,
    /// Continuazione tematica — il sistema attraversa lo spazio
    Traverse,
    /// Cambio tema — il sistema si rilocalizza
    Jump,
    /// Deriva onirica — il sistema si muove durante il sogno
    Drift,
}

impl Locus {
    pub fn new() -> Self {
        Self {
            position: None,
            horizon: 3.0,
            distances: HashMap::new(),
            trail: Vec::new(),
            jump_threshold: 4.0,
            trail_capacity: 20,
            sub_position: HashMap::new(),
        }
    }

    /// Calcola dove un input vuole portare il sistema.
    /// Restituisce il frattale con la massima affinita composita dall'input.
    pub fn compute_destination(
        phrase: &PhrasePattern,
        _registry: &FractalRegistry,
    ) -> Option<FractalId> {
        phrase.fractal_involvement.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&fid, _)| fid)
    }

    /// Muovi il sistema verso una destinazione.
    /// Decide automaticamente se e Traverse o Jump.
    pub fn move_to(
        &mut self,
        destination: FractalId,
        complex: &SimplicialComplex,
        registry: &FractalRegistry,
    ) -> Movement {
        let from = self.position;

        match from {
            None => {
                // Primo posizionamento
                self.position = Some(destination);
                self.trail.push(destination);
                self.recalculate_distances(complex);
                self.recalculate_sub_position(registry);
                Movement {
                    from: None,
                    to: destination,
                    kind: MovementKind::Origin,
                    path: Vec::new(),
                    distance: 0.0,
                }
            }
            Some(current) => {
                if current == destination {
                    // Gia li — nessun movimento
                    return Movement {
                        from: Some(current),
                        to: destination,
                        kind: MovementKind::Traverse,
                        path: Vec::new(),
                        distance: 0.0,
                    };
                }

                let dist = geodesic_distance(complex, current, destination)
                    .unwrap_or(self.jump_threshold + 1.0);

                let (kind, path) = if dist > self.jump_threshold {
                    // Cambio tema: salto diretto
                    (MovementKind::Jump, Vec::new())
                } else {
                    // Continuazione: attraversa lo spazio
                    let geodesic_path = find_geodesic(complex, registry, current, destination);
                    let waypoints: Vec<FractalId> = geodesic_path
                        .map(|p| p.steps.iter().map(|s| s.fractal_id).collect())
                        .unwrap_or_default();
                    (MovementKind::Traverse, waypoints)
                };

                self.position = Some(destination);
                self.trail.push(destination);
                if self.trail.len() > self.trail_capacity {
                    self.trail.remove(0);
                }
                self.recalculate_distances(complex);
                self.recalculate_sub_position(registry);

                Movement { from: Some(current), to: destination, kind, path, distance: dist }
            }
        }
    }

    /// La visibilita di un frattale dal locus corrente.
    /// 1.0 = prossimale (vivido), 0.0 = oltre l'orizzonte (invisibile).
    pub fn visibility(&self, fractal: FractalId) -> f64 {
        if self.position.is_none() {
            return 0.5; // Nessun locus → visibilita neutra
        }
        if Some(fractal) == self.position {
            return 1.0; // Al locus → massima visibilita
        }
        if let Some(&dist) = self.distances.get(&fractal) {
            if dist > self.horizon {
                return 0.0;
            }
            // Gaussiana: decade con la distanza
            let sigma = self.horizon / 3.0;
            (-dist * dist / (2.0 * sigma * sigma)).exp()
        } else {
            0.0 // Non raggiungibile
        }
    }

    /// Tutti i frattali entro l'orizzonte, con la loro visibilita.
    pub fn visible_fractals(&self) -> Vec<(FractalId, f64)> {
        if self.position.is_none() {
            return Vec::new();
        }
        let mut result: Vec<(FractalId, f64)> = self.distances.iter()
            .filter_map(|(&fid, &dist)| {
                if dist <= self.horizon {
                    Some((fid, self.visibility(fid)))
                } else {
                    None
                }
            })
            .collect();

        // Aggiungi il locus stesso
        if let Some(pos) = self.position {
            if !result.iter().any(|(fid, _)| *fid == pos) {
                result.push((pos, 1.0));
            }
        }

        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    /// Drift onirico: muovi il locus lungo il gradiente di attivazione residua.
    pub fn dream_drift(
        &mut self,
        complex: &SimplicialComplex,
        registry: &FractalRegistry,
        phase: &SleepPhase,
    ) -> Option<Movement> {
        let _current = self.position?;

        match phase {
            SleepPhase::Awake => None, // Da sveglio, il locus si muove solo con input
            SleepPhase::DeepSleep { .. } => None, // Consolidamento locale, non si muove
            SleepPhase::LightSleep { .. } => {
                // Piccoli drift: vicino piu attivo entro distanza 1.0
                self.drift_to_most_active(complex, registry, 1.0)
            }
            SleepPhase::REM { depth } => {
                // Orizzonte espanso, drift verso regioni lontane
                let old_horizon = self.horizon;
                self.horizon *= 1.0 + depth;
                let result = self.drift_to_most_active(complex, registry, self.horizon * 0.5);
                self.horizon = old_horizon;
                result
            }
            SleepPhase::WakefulDream { .. } => {
                // Piccoli drift come LightSleep
                self.drift_to_most_active(complex, registry, 1.5)
            }
        }
    }

    /// Drift verso il frattale piu attivo entro un raggio.
    fn drift_to_most_active(
        &mut self,
        complex: &SimplicialComplex,
        registry: &FractalRegistry,
        max_distance: f64,
    ) -> Option<Movement> {
        let current = self.position?;

        // Trova il frattale piu attivo entro il raggio
        let mut best: Option<(FractalId, f64)> = None;
        for (&fid, &dist) in &self.distances {
            if dist > max_distance || fid == current {
                continue;
            }
            // Attivazione media dei simplessi che toccano questo frattale
            let activation = complex.simplices_of(fid)
                .iter()
                .filter_map(|sid| complex.get(*sid))
                .map(|s| s.current_activation)
                .sum::<f64>();

            if activation > 0.0 {
                if best.is_none() || activation > best.unwrap().1 {
                    best = Some((fid, activation));
                }
            }
        }

        if let Some((target, _)) = best {
            let dist = self.distances.get(&target).copied().unwrap_or(0.0);
            self.position = Some(target);
            self.trail.push(target);
            if self.trail.len() > self.trail_capacity {
                self.trail.remove(0);
            }
            self.recalculate_distances(complex);
            self.recalculate_sub_position(registry);

            Some(Movement {
                from: Some(current),
                to: target,
                kind: MovementKind::Drift,
                path: Vec::new(),
                distance: dist,
            })
        } else {
            None
        }
    }

    /// Ricalcola le distanze dal locus corrente.
    fn recalculate_distances(&mut self, complex: &SimplicialComplex) {
        if let Some(pos) = self.position {
            self.distances = distance_map(complex, pos);
        }
    }

    /// Ricalcola la posizione nelle dimensioni libere del frattale corrente.
    /// Il sub-locus inizia al centro (0.5) per ogni dimensione libera.
    fn recalculate_sub_position(&mut self, registry: &FractalRegistry) {
        self.sub_position.clear();
        if let Some(pos) = self.position {
            if let Some(fractal) = registry.get(pos) {
                for dim in fractal.free_dims() {
                    self.sub_position.insert(dim, 0.5);
                }
            }
        }
    }

    /// Aggiorna il sub-locus in base all'input: le dimensioni libere
    /// vengono spostate verso i valori della firma composita dell'input.
    /// Forza di spostamento proporzionale alla perturbazione.
    pub fn update_sub_position(&mut self, input_signature: &PrimitiveCore, strength: f64) {
        let s = strength.clamp(0.0, 0.5); // Non troppo aggressivo
        for (dim, current) in self.sub_position.iter_mut() {
            let target = input_signature.get(*dim);
            // Spostamento elastico verso il valore dell'input
            *current = *current * (1.0 - s) + target * s;
        }
    }

    /// Vista del sub-locus corrente: dove siamo dentro il frattale.
    pub fn sub_locus_view(&self, registry: &FractalRegistry) -> Option<SubLocusView> {
        let pos = self.position?;
        let fractal = registry.get(pos)?;
        let coordinates: Vec<(Dim, f64)> = fractal.free_dims().iter()
            .filter_map(|d| self.sub_position.get(d).map(|&v| (*d, v)))
            .collect();
        Some(SubLocusView {
            fractal_id: pos,
            fractal_name: fractal.name.clone(),
            degrees_of_freedom: coordinates.len(),
            coordinates,
        })
    }

    /// Il punto 8D completo del locus: dimensioni fisse dal frattale +
    /// dimensioni libere dal sub-locus. E la posizione esatta nel campo.
    pub fn full_position(&self, registry: &FractalRegistry) -> Option<PrimitiveCore> {
        let pos = self.position?;
        let fractal = registry.get(pos)?;
        let mut values = [0.5; 8];
        for d in &Dim::ALL {
            match fractal.signature[d.index()] {
                DimConstraint::Fixed(v) => values[d.index()] = v,
                DimConstraint::Free => {
                    values[d.index()] = self.sub_position
                        .get(d)
                        .copied()
                        .unwrap_or(0.5);
                }
            }
        }
        Some(PrimitiveCore::new(values))
    }
}

// ═══════════════════════════════════════════════════════════════
// PROIEZIONE OLOGRAFICA
// ═══════════════════════════════════════════════════════════════

/// Proietta l'intero universo dal punto di vista di un frattale.
/// Ogni frattale vede tutti gli altri, ma li vede *diversamente*:
/// - Prossimita topologica (da distance_map) → quanto e vicino
/// - Risonanza dimensionale → quante dimensioni condividono
/// - Distorsione → quanto il target appare "deformato" dalle
///   dimensioni libere del sorgente
pub fn project_universe(
    from: FractalId,
    complex: &SimplicialComplex,
    registry: &FractalRegistry,
) -> Option<HolographicProjection> {
    let source = registry.get(from)?;
    let distances = distance_map(complex, from);
    let _source_free = source.free_dims();
    let _source_center = source.center();

    let mut projections = Vec::new();

    for (&target_id, target) in registry.iter() {
        if target_id == from {
            continue;
        }

        // 1. Prossimita topologica: inversamente proporzionale alla distanza
        let dist = distances.get(&target_id).copied().unwrap_or(10.0);
        let proximity = if dist < 0.01 { 1.0 } else { 1.0 / (1.0 + dist) };

        // 2. Risonanza dimensionale: quante dimensioni sono vincolate
        //    allo stesso valore (o vicino) in entrambi i frattali
        let mut resonance_sum = 0.0;
        let mut resonance_count = 0;
        for d in &Dim::ALL {
            let idx = d.index();
            if let (DimConstraint::Fixed(v1), DimConstraint::Fixed(v2)) =
                (source.signature[idx], target.signature[idx])
            {
                resonance_sum += 1.0 - (v1 - v2).abs();
                resonance_count += 1;
            }
        }
        let dimensional_resonance = if resonance_count > 0 {
            resonance_sum / resonance_count as f64
        } else {
            0.0
        };

        // 3. Centro apparente: il centro del target visto attraverso
        //    le dimensioni libere del sorgente.
        //    Le dimensioni fisse del sorgente "colorano" la percezione:
        //    il target viene tirato verso i valori del sorgente nelle
        //    sue dimensioni fisse, e lasciato libero dove il sorgente e libero.
        let target_center = target.center();
        let mut apparent_values = [0.5; 8];
        for d in &Dim::ALL {
            let idx = d.index();
            let target_val = target_center.get(*d);
            match source.signature[idx] {
                DimConstraint::Fixed(src_val) => {
                    // Dimensione fissa del sorgente: il target viene
                    // "visto" parzialmente attraverso questa lente.
                    // Blend tra il valore reale e quello del sorgente.
                    apparent_values[idx] = target_val * 0.6 + src_val * 0.4;
                }
                DimConstraint::Free => {
                    // Dimensione libera: il target appare come e realmente
                    apparent_values[idx] = target_val;
                }
            }
        }
        let apparent_center = PrimitiveCore::new(apparent_values);

        // 4. Distorsione: quanto il centro apparente differisce dal centro reale
        let distortion = apparent_center.distance(&target_center);

        projections.push(FractalProjection {
            fractal_id: target_id,
            name: target.name.clone(),
            proximity,
            dimensional_resonance,
            distortion,
            apparent_center,
        });
    }

    // Ordina per prossimita (i piu vicini prima)
    projections.sort_by(|a, b| b.proximity.partial_cmp(&a.proximity)
        .unwrap_or(std::cmp::Ordering::Equal));

    Some(HolographicProjection {
        from,
        from_name: source.name.clone(),
        projections,
    })
}

/// Proietta un singolo frattale dal punto di vista del locus corrente.
/// Tiene conto anche del sub-locus (posizione nelle dimensioni libere).
pub fn project_from_locus(
    locus: &Locus,
    target_id: FractalId,
    complex: &SimplicialComplex,
    registry: &FractalRegistry,
) -> Option<FractalProjection> {
    let from_id = locus.position?;
    let source = registry.get(from_id)?;
    let target = registry.get(target_id)?;

    let dist = geodesic_distance(complex, from_id, target_id).unwrap_or(10.0);
    let proximity = if dist < 0.01 { 1.0 } else { 1.0 / (1.0 + dist) };

    // Risonanza
    let mut resonance_sum = 0.0;
    let mut resonance_count = 0;
    for d in &Dim::ALL {
        let idx = d.index();
        if let (DimConstraint::Fixed(v1), DimConstraint::Fixed(v2)) =
            (source.signature[idx], target.signature[idx])
        {
            resonance_sum += 1.0 - (v1 - v2).abs();
            resonance_count += 1;
        }
    }
    let dimensional_resonance = if resonance_count > 0 {
        resonance_sum / resonance_count as f64
    } else {
        0.0
    };

    // Centro apparente — usa il sub-locus come lente aggiuntiva
    let target_center = target.center();
    let full_pos = locus.full_position(registry).unwrap_or_else(PrimitiveCore::neutral);

    let mut apparent_values = [0.5; 8];
    for d in &Dim::ALL {
        let idx = d.index();
        let target_val = target_center.get(*d);
        match source.signature[idx] {
            DimConstraint::Fixed(_) => {
                // Dimensione fissa: il sub-locus non influenza
                apparent_values[idx] = target_val * 0.6 + full_pos.get(*d) * 0.4;
            }
            DimConstraint::Free => {
                // Dimensione libera: il sub-locus la colora
                let sub_val = locus.sub_position.get(d).copied().unwrap_or(0.5);
                // Piu il sub-locus e lontano dal centro, piu distorce
                let sub_influence = (sub_val - 0.5).abs() * 2.0; // [0, 1]
                apparent_values[idx] = target_val * (1.0 - sub_influence * 0.3)
                    + sub_val * (sub_influence * 0.3);
            }
        }
    }
    let apparent_center = PrimitiveCore::new(apparent_values);
    let distortion = apparent_center.distance(&target_center);

    Some(FractalProjection {
        fractal_id: target_id,
        name: target.name.clone(),
        proximity,
        dimensional_resonance,
        distortion,
        apparent_center,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::fractal::bootstrap_fractals;
    use crate::topology::simplex::bootstrap_complex;
    use crate::topology::lexicon::Lexicon;
    use crate::topology::composition::compose_phrase;

    fn setup() -> (FractalRegistry, SimplicialComplex) {
        let registry = bootstrap_fractals();
        let mut ids = registry.all_ids();
        ids.sort();
        let complex = bootstrap_complex(&ids);
        (registry, complex)
    }

    #[test]
    fn test_initial_state() {
        let locus = Locus::new();
        assert!(locus.position.is_none());
        assert!(locus.trail.is_empty());
        assert!((locus.horizon - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_first_placement() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        let ids = registry.all_ids();

        let movement = locus.move_to(ids[0], &complex, &registry);
        assert_eq!(movement.kind, MovementKind::Origin);
        assert_eq!(locus.position, Some(ids[0]));
        assert_eq!(locus.trail.len(), 1);
    }

    #[test]
    fn test_traverse_nearby() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        let ids = registry.all_ids();

        // Prima posizione
        locus.move_to(ids[0], &complex, &registry);

        // Muovi a un vicino — dovrebbe essere Traverse
        let movement = locus.move_to(ids[1], &complex, &registry);
        assert!(movement.kind == MovementKind::Traverse || movement.kind == MovementKind::Jump,
            "Movimento verso vicino deve essere Traverse o Jump");
        assert_eq!(locus.position, Some(ids[1]));
        assert_eq!(locus.trail.len(), 2);
    }

    #[test]
    fn test_visibility_at_locus() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        let ids = registry.all_ids();

        locus.move_to(ids[0], &complex, &registry);

        // Al locus: visibilita 1.0
        let vis = locus.visibility(ids[0]);
        assert!((vis - 1.0).abs() < f64::EPSILON, "Visibilita al locus deve essere 1.0");
    }

    #[test]
    fn test_visibility_decays() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        let ids = registry.all_ids();

        locus.move_to(ids[0], &complex, &registry);

        // I vicini devono avere visibilita tra 0 e 1
        for &fid in &ids[1..] {
            let vis = locus.visibility(fid);
            assert!(vis >= 0.0 && vis <= 1.0,
                "Visibilita deve essere in [0, 1], trovato {}", vis);
        }
    }

    #[test]
    fn test_visible_fractals() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        let ids = registry.all_ids();

        locus.move_to(ids[0], &complex, &registry);

        let visible = locus.visible_fractals();
        assert!(!visible.is_empty(), "Ci devono essere frattali visibili");

        // Il primo deve essere il locus stesso
        assert_eq!(visible[0].0, ids[0], "Il locus deve essere il piu visibile");
        assert!((visible[0].1 - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_trail_capacity() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        locus.trail_capacity = 3;
        let ids = registry.all_ids();

        for &id in ids.iter().take(5) {
            locus.move_to(id, &complex, &registry);
        }

        assert!(locus.trail.len() <= 3,
            "Il trail non deve superare la capacita, len={}", locus.trail.len());
    }

    #[test]
    fn test_compute_destination() {
        let (registry, _complex) = setup();
        let mut lexicon = Lexicon::bootstrap();

        let phrase = compose_phrase(&mut lexicon, "io penso al tempo futuro", &registry);
        let dest = Locus::compute_destination(&phrase, &registry);
        assert!(dest.is_some(), "Deve trovare una destinazione per 'io penso al tempo futuro'");
    }

    #[test]
    fn test_no_drift_when_awake() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        let ids = registry.all_ids();
        locus.move_to(ids[0], &complex, &registry);

        let drift = locus.dream_drift(&complex, &registry, &SleepPhase::Awake);
        assert!(drift.is_none(), "Da sveglio il locus non drifta");
    }

    // === Test Sub-Locus ===

    #[test]
    fn test_sub_position_initialized_on_move() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        let ids = registry.all_ids();

        // SPAZIO (id=0) ha Valenza, Intensita, Complessita, Tempo libere
        locus.move_to(ids[0], &complex, &registry);

        assert!(!locus.sub_position.is_empty(),
            "Il sub-locus deve avere coordinate dopo move_to");

        // Tutte le dimensioni libere devono essere a 0.5 (centro)
        for (&_dim, &val) in &locus.sub_position {
            assert!((val - 0.5).abs() < f64::EPSILON,
                "Sub-posizione iniziale deve essere 0.5, trovato {}", val);
        }
    }

    #[test]
    fn test_sub_position_matches_free_dims() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();

        // SPAZIO ha 4 dimensioni libere: Valenza, Intensita, Complessita, Tempo
        locus.move_to(0, &complex, &registry);
        let spazio = registry.get(0).unwrap();
        let free = spazio.free_dims();

        assert_eq!(locus.sub_position.len(), free.len(),
            "Sub-locus deve avere {} coordinate, ha {}",
            free.len(), locus.sub_position.len());

        for dim in &free {
            assert!(locus.sub_position.contains_key(dim),
                "Dimensione libera {:?} assente dal sub-locus", dim);
        }
    }

    #[test]
    fn test_update_sub_position() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        locus.move_to(0, &complex, &registry); // SPAZIO

        // Ordine I Ching: Valenza=7. Signature con Valenza alta (0.9) e Intensità bassa (0.1).
        let sig = PrimitiveCore::new([0.2, 0.7, 0.1, 0.5, 0.2, 0.7, 0.5, 0.9]);
        locus.update_sub_position(&sig, 0.5);

        // Valenza (libera in SPAZIO) deve essersi spostata verso 0.9
        if let Some(&val) = locus.sub_position.get(&Dim::Valenza) {
            assert!(val > 0.5, "Valenza deve spostarsi verso 0.9, trovato {}", val);
        }
    }

    #[test]
    fn test_sub_locus_view() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        locus.move_to(0, &complex, &registry); // POTERE (Agency=0.90 fixed)

        let view = locus.sub_locus_view(&registry);
        assert!(view.is_some());
        let view = view.unwrap();
        assert_eq!(view.fractal_name, "POTERE");
        assert!(view.degrees_of_freedom > 0,
            "POTERE deve avere gradi di liberta");
    }

    #[test]
    fn test_full_position() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        locus.move_to(0, &complex, &registry); // POTERE (Agency=0.90 fixed)

        let full = locus.full_position(&registry);
        assert!(full.is_some());
        let full = full.unwrap();

        // Agency e fisso a 0.90 in POTERE (trigramma ☰☰)
        assert!((full.get(Dim::Agency) - 0.90).abs() < 1e-6,
            "Agency fisso deve essere 0.90, trovato {}", full.get(Dim::Agency));

        // Confine e libero, deve essere 0.5 (centro iniziale)
        assert!((full.get(Dim::Confine) - 0.5).abs() < f64::EPSILON,
            "Confine libero deve essere 0.5, trovato {}", full.get(Dim::Confine));
    }

    #[test]
    fn test_sub_position_changes_on_fractal_change() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();

        // Muovi a POTERE(0) — 1 dim fissa (Agency), 7 libere
        locus.move_to(0, &complex, &registry);
        let potere_free = locus.sub_position.len();

        // Muovi a IDENTITA(32) — 2 dim fisse (Confine+Agency), 6 libere
        locus.move_to(32, &complex, &registry);
        let identita_free = locus.sub_position.len();

        assert_ne!(potere_free, identita_free,
            "POTERE(1 fixed) e IDENTITA(2 fixed) devono avere diversi gradi di liberta: {} vs {}",
            potere_free, identita_free);
    }

    // === Test Proiezione Olografica ===

    #[test]
    fn test_project_universe() {
        let (registry, complex) = setup();
        let projection = super::project_universe(0, &complex, &registry);

        assert!(projection.is_some());
        let proj = projection.unwrap();
        assert_eq!(proj.from_name, "POTERE");
        assert!(!proj.projections.is_empty(),
            "La proiezione deve contenere altri frattali");

        // Tutti i frattali proiettati devono avere prossimita in [0, 1]
        for fp in &proj.projections {
            assert!(fp.proximity >= 0.0 && fp.proximity <= 1.0,
                "Prossimita deve essere in [0,1], trovato {} per {}", fp.proximity, fp.name);
        }
    }

    #[test]
    fn test_projection_is_perspective_dependent() {
        let (registry, complex) = setup();

        // Proietta da POTERE(0) e da ARMONIA(63)
        let proj_potere = super::project_universe(0, &complex, &registry).unwrap();
        let proj_armonia = super::project_universe(63, &complex, &registry).unwrap();

        // Trova come appare TERRA(9) da POTERE e da ARMONIA
        // Entrambi sono vicini a TERRA nel ring
        let terra_da_potere = proj_potere.projections.iter()
            .find(|p| p.fractal_id == 9);
        let terra_da_armonia = proj_armonia.projections.iter()
            .find(|p| p.fractal_id == 9);

        assert!(terra_da_potere.is_some() && terra_da_armonia.is_some(),
            "TERRA(9) deve essere visibile da POTERE e da ARMONIA");

        let ts = terra_da_potere.unwrap();
        let te = terra_da_armonia.unwrap();

        // La distorsione deve essere diversa (prospettive diverse)
        // o almeno i centri apparenti devono differire
        let center_diff = ts.apparent_center.distance(&te.apparent_center);
        assert!(center_diff > 0.01 || (ts.distortion - te.distortion).abs() > 0.01,
            "TEMPO visto da SPAZIO e da EGO deve apparire diverso. \
             Diff centri: {:.4}, diff distorsione: {:.4}",
            center_diff, (ts.distortion - te.distortion).abs());
    }

    #[test]
    fn test_project_from_locus() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        locus.move_to(0, &complex, &registry); // POTERE

        // MATERIA (9) e vicino diretto di POTERE nel ring
        let proj = super::project_from_locus(&locus, 9, &complex, &registry);
        assert!(proj.is_some());
        let proj = proj.unwrap();
        assert_eq!(proj.name, "MATERIA");
        assert!(proj.proximity > 0.0);
    }

    #[test]
    fn test_sub_locus_affects_projection() {
        let (registry, complex) = setup();
        let mut locus = Locus::new();
        locus.move_to(0, &complex, &registry); // SPAZIO

        // Proiezione dal centro del sub-locus
        let proj_center = super::project_from_locus(&locus, 1, &complex, &registry).unwrap();

        // Sposta il sub-locus verso Valenza alta
        let sig = PrimitiveCore::new([0.2, 1.0, 1.0, 0.7, 1.0, 0.7, 0.2, 1.0]);
        locus.update_sub_position(&sig, 0.5);

        // Proiezione dal sub-locus spostato
        let proj_shifted = super::project_from_locus(&locus, 1, &complex, &registry).unwrap();

        // I centri apparenti devono differire
        let diff = proj_center.apparent_center.distance(&proj_shifted.apparent_center);
        assert!(diff > 0.001,
            "Spostare il sub-locus deve cambiare la proiezione, diff={:.6}", diff);
    }
}
