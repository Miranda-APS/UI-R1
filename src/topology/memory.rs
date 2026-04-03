/// Memoria Topologica — Contrazione del campo, non archiviazione.
///
/// La memoria non e un database. E il campo stesso a diversi livelli
/// di contrazione temporale.
/// - STM: la forma attuale del campo (simplessi appena attivati)
/// - MTM: la postura del campo (deformazioni che persistono)
/// - LTM: lo scheletro del campo (struttura topologica consolidata)

use std::collections::VecDeque;
use crate::topology::simplex::{SimplicialComplex, SimplexId};
use crate::topology::fractal::FractalId;
use crate::topology::composition::PhrasePattern;

/// Un'impronta: snapshot parziale dello stato del complesso in un momento.
#[derive(Debug, Clone)]
pub struct FieldImprint {
    /// Simplessi che erano attivi e il loro livello
    pub active_simplices: Vec<(SimplexId, f64)>,
    /// Frattali coinvolti
    pub involved_fractals: Vec<FractalId>,
    /// Timestamp (tick interno)
    pub tick: u64,
    /// Forza dell'impronta (decade nel tempo)
    pub strength: f64,
    /// Etichetta (da dove viene: input, sogno, etc.)
    pub origin: String,
}

/// Una risonanza: il passato deforma il presente.
#[derive(Debug, Clone)]
pub struct Resonance {
    /// L'impronta che risuona
    pub imprint: FieldImprint,
    /// Quanto forte e la risonanza [0.0, 1.0]
    pub strength: f64,
}

/// Una traccia episodica: memoria NARRATIVA di un evento.
/// A differenza di FieldImprint (struttura topologica),
/// EpisodicTrace memorizza la SEQUENZA e il SIGNIFICATO dell'evento.
#[derive(Debug, Clone)]
pub struct EpisodicTrace {
    /// Timestamp assoluto (u64 tick interno dell'engine)
    pub timestamp: u64,
    /// Numero del turno conversazionale
    pub turn_number: usize,
    /// Frattale in cui l'entita era posizionata al momento dell'evento
    pub locus_fractal: Option<FractalId>,
    /// Posizione concettuale 8D dell'evento (composite_signature della frase)
    pub conceptual_position: crate::topology::primitive::PrimitiveCore,
    /// La frase che ha generato l'evento
    pub phrase: PhrasePattern,
    /// Testo letterale dell'input
    pub input_text: String,
    /// Partecipanti (chi ha detto cosa: "io", "tu", "utente")
    pub speaker: String,
    /// Link causali: indici di tracce precedenti che hanno causato questa
    pub causal_links: Vec<usize>,
    /// Tono emotivo dell'evento (valenza vitale al momento)
    pub emotional_tone: f64,
    /// Quanto e stato importante l'evento (forza perturbazione campo)
    pub salience: f64,
}

impl EpisodicTrace {
    /// Crea una nuova traccia episodica da un input ricevuto.
    pub fn from_input(
        timestamp: u64,
        turn_number: usize,
        locus_fractal: Option<FractalId>,
        phrase: PhrasePattern,
        input_text: String,
        speaker: String,
        emotional_tone: f64,
        salience: f64,
    ) -> Self {
        let conceptual_position = phrase.composite_signature;
        Self {
            timestamp,
            turn_number,
            locus_fractal,
            conceptual_position,
            phrase,
            input_text,
            speaker,
            causal_links: Vec::new(),
            emotional_tone,
            salience,
        }
    }

    /// Distanza temporale da un altro evento (in tick).
    pub fn time_distance(&self, other: &EpisodicTrace) -> u64 {
        if self.timestamp > other.timestamp {
            self.timestamp - other.timestamp
        } else {
            other.timestamp - self.timestamp
        }
    }

    /// Distanza concettuale (posizione 8D) da un altro evento.
    pub fn conceptual_distance(&self, other: &EpisodicTrace) -> f64 {
        self.conceptual_position.distance(&other.conceptual_position)
    }
}

/// Memoria topologica stratificata.
#[derive(Debug)]
pub struct TopologicalMemory {
    /// STM: impronte recenti (finestra scorrevole)
    pub short_term: VecDeque<FieldImprint>,
    /// MTM: impronte consolidate (persistono per sessione)
    pub medium_term: Vec<FieldImprint>,
    /// LTM: impronte cristallizzate (permanenti)
    pub long_term: Vec<FieldImprint>,

    /// Memoria episodica: tracce narrative degli eventi
    pub episodic_memory: Vec<EpisodicTrace>,

    /// Configurazione
    pub stm_capacity: usize,
    pub consolidation_threshold: u64, // dopo N attivazioni → MTM
    pub crystallization_threshold: u64, // dopo N attivazioni → LTM

    /// Soglia di risonanza minima
    pub resonance_threshold: f64,

    /// Tick interno
    pub current_tick: u64,
}

impl TopologicalMemory {
    pub fn new() -> Self {
        Self {
            short_term: VecDeque::new(),
            medium_term: Vec::new(),
            long_term: Vec::new(),
            episodic_memory: Vec::new(),
            stm_capacity: 20,
            consolidation_threshold: 5,
            crystallization_threshold: 20,
            resonance_threshold: 0.3,
            current_tick: 0,
        }
    }

    /// Cattura lo stato corrente del complesso come impronta STM.
    pub fn capture(&mut self, complex: &SimplicialComplex, origin: &str) {
        self.current_tick += 1;

        let active: Vec<(SimplexId, f64)> = complex.active_simplices().iter()
            .map(|s| (s.id, s.current_activation))
            .collect();

        if active.is_empty() {
            return;
        }

        // Raccogli frattali coinvolti
        let mut fractals = Vec::new();
        for &(sid, _) in &active {
            if let Some(s) = complex.get(sid) {
                for &v in &s.vertices {
                    if !fractals.contains(&v) {
                        fractals.push(v);
                    }
                }
            }
        }

        let imprint = FieldImprint {
            active_simplices: active,
            involved_fractals: fractals,
            tick: self.current_tick,
            strength: 1.0,
            origin: origin.to_string(),
        };

        self.short_term.push_back(imprint);
        if self.short_term.len() > self.stm_capacity {
            self.short_term.pop_front();
        }
    }

    /// Risonanza: lascia che il campo presente risuoni col passato.
    /// Restituisce le impronte MTM/LTM che risuonano con lo stato attuale.
    pub fn resonate(&self, complex: &SimplicialComplex) -> Vec<Resonance> {
        let current_active: Vec<SimplexId> = complex.active_simplices()
            .iter().map(|s| s.id).collect();

        if current_active.is_empty() {
            return Vec::new();
        }

        let mut resonances = Vec::new();

        // Cerca risonanza nella MTM e LTM
        for imprint in self.medium_term.iter().chain(self.long_term.iter()) {
            let similarity = self.topological_similarity(&current_active, &imprint.active_simplices);
            let adjusted = similarity * imprint.strength;

            if adjusted > self.resonance_threshold {
                resonances.push(Resonance {
                    imprint: imprint.clone(),
                    strength: adjusted,
                });
            }
        }

        resonances.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
        resonances
    }

    /// Similarita topologica: quanti simplessi in comune, pesati per attivazione.
    fn topological_similarity(
        &self,
        current: &[SimplexId],
        past: &[(SimplexId, f64)],
    ) -> f64 {
        if current.is_empty() || past.is_empty() {
            return 0.0;
        }
        let matching: f64 = past.iter()
            .filter(|(sid, _)| current.contains(sid))
            .map(|(_, act)| act)
            .sum();
        let max_possible: f64 = past.iter().map(|(_, act)| act).sum();
        if max_possible <= 0.0 { 0.0 } else { matching / max_possible }
    }

    /// Consolidamento: promuovi impronte STM ricorrenti a MTM.
    /// Chiamato periodicamente o durante il sogno leggero.
    pub fn consolidate(&mut self) {
        // Conta quante volte ogni simplesso appare nella STM
        let mut simplex_count: std::collections::HashMap<SimplexId, u64> = std::collections::HashMap::new();
        for imprint in &self.short_term {
            for &(sid, _) in &imprint.active_simplices {
                *simplex_count.entry(sid).or_insert(0) += 1;
            }
        }

        // Trova simplessi ricorrenti
        let recurring: Vec<SimplexId> = simplex_count.iter()
            .filter(|(_, count)| **count >= self.consolidation_threshold)
            .map(|(sid, _)| *sid)
            .collect();

        if !recurring.is_empty() {
            // Crea un'impronta MTM dai simplessi ricorrenti
            let active: Vec<(SimplexId, f64)> = recurring.iter()
                .map(|sid| (*sid, *simplex_count.get(sid).unwrap() as f64 / self.stm_capacity as f64))
                .collect();

            let mut fractals = Vec::new();
            for imprint in &self.short_term {
                for &fid in &imprint.involved_fractals {
                    if !fractals.contains(&fid) {
                        fractals.push(fid);
                    }
                }
            }

            let consolidated = FieldImprint {
                active_simplices: active,
                involved_fractals: fractals,
                tick: self.current_tick,
                strength: 0.8,
                origin: "consolidamento".to_string(),
            };

            self.medium_term.push(consolidated);
        }
    }

    /// Cristallizzazione: promuovi impronte MTM molto stabili a LTM.
    /// Chiamato durante il sogno profondo.
    pub fn crystallize(&mut self) {
        let threshold = self.crystallization_threshold;
        let mut to_promote = Vec::new();

        for (i, imprint) in self.medium_term.iter().enumerate() {
            // Un'impronta MTM diventa LTM se e vecchia abbastanza
            // e ha forza residua alta
            if self.current_tick - imprint.tick > threshold && imprint.strength > 0.5 {
                to_promote.push(i);
            }
        }

        // Promuovi in ordine inverso per non invalidare gli indici
        for i in to_promote.into_iter().rev() {
            let mut imprint = self.medium_term.remove(i);
            imprint.origin = format!("{} → cristallizzato", imprint.origin);
            self.long_term.push(imprint);
        }
    }

    /// Phase 52: consolidamento leggero — promuove pattern STM ricorrenti (3+ occorrenze)
    /// senza attendere il DeepSleep completo. Apprendimento continuo durante la veglia.
    pub fn consolidate_light(&mut self) {
        if self.short_term.len() < 5 { return; }

        let mut simplex_count: std::collections::HashMap<SimplexId, u64> =
            std::collections::HashMap::new();
        for imprint in &self.short_term {
            for &(sid, _) in &imprint.active_simplices {
                *simplex_count.entry(sid).or_insert(0) += 1;
            }
        }

        // Soglia leggera: 3 occorrenze (vs 5 per il consolidamento pieno)
        let recurring: Vec<SimplexId> = simplex_count.iter()
            .filter(|(_, count)| **count >= 3)
            .map(|(sid, _)| *sid)
            .collect();

        if recurring.is_empty() { return; }

        // Evita duplicati: non creare MTM se esiste già un'impronta con gli stessi simplessi
        let existing_sids: std::collections::HashSet<SimplexId> = self.medium_term.iter()
            .flat_map(|imp| imp.active_simplices.iter().map(|(sid, _)| *sid))
            .collect();
        let truly_new: Vec<SimplexId> = recurring.into_iter()
            .filter(|sid| !existing_sids.contains(sid))
            .collect();

        if truly_new.is_empty() { return; }

        let active: Vec<(SimplexId, f64)> = truly_new.iter()
            .map(|sid| (*sid, *simplex_count.get(sid).unwrap() as f64 / self.stm_capacity as f64))
            .collect();

        let mut fractals = Vec::new();
        for imprint in &self.short_term {
            for &fid in &imprint.involved_fractals {
                if !fractals.contains(&fid) {
                    fractals.push(fid);
                }
            }
        }

        let consolidated = FieldImprint {
            active_simplices: active,
            involved_fractals: fractals,
            tick: self.current_tick,
            strength: 0.5, // più leggero del consolidamento pieno (0.8)
            origin: "consolidamento_leggero".to_string(),
        };

        self.medium_term.push(consolidated);
    }

    /// Decadimento: le impronte perdono forza nel tempo.
    pub fn decay(&mut self, rate: f64) {
        for imprint in self.medium_term.iter_mut() {
            imprint.strength = (imprint.strength - rate).max(0.0);
        }
        // Rimuovi impronte MTM con forza zero
        self.medium_term.retain(|i| i.strength > 0.01);

        // LTM decade molto piu lentamente
        for imprint in self.long_term.iter_mut() {
            imprint.strength = (imprint.strength - rate * 0.1).max(0.0);
        }
        self.long_term.retain(|i| i.strength > 0.01);
    }

    /// Statistiche.
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            stm_count: self.short_term.len(),
            mtm_count: self.medium_term.len(),
            ltm_count: self.long_term.len(),
            current_tick: self.current_tick,
        }
    }

    // === MEMORIA EPISODICA ===

    /// Registra un evento nella memoria episodica.
    pub fn record_episode(&mut self, trace: EpisodicTrace) {
        self.episodic_memory.push(trace);
        // Mantieni solo gli ultimi 100 episodi (configurabile)
        if self.episodic_memory.len() > 100 {
            self.episodic_memory.remove(0);
        }
    }

    /// Cerca episodi per speaker (chi ha parlato).
    pub fn find_by_speaker(&self, speaker: &str) -> Vec<&EpisodicTrace> {
        self.episodic_memory.iter()
            .filter(|t| t.speaker == speaker)
            .collect()
    }

    /// Cerca episodi in un range temporale (da timestamp_start a timestamp_end).
    pub fn find_by_time_range(&self, start: u64, end: u64) -> Vec<&EpisodicTrace> {
        self.episodic_memory.iter()
            .filter(|t| t.timestamp >= start && t.timestamp <= end)
            .collect()
    }

    /// Cerca episodi vicini a una posizione concettuale (entro una distanza 8D).
    pub fn find_near_position(&self, target_position: &crate::topology::primitive::PrimitiveCore, max_distance: f64) -> Vec<&EpisodicTrace> {
        self.episodic_memory.iter()
            .filter(|t| t.conceptual_position.distance(target_position) <= max_distance)
            .collect()
    }

    /// Cerca episodi avvenuti in un frattale specifico.
    pub fn find_by_fractal(&self, fractal_id: FractalId) -> Vec<&EpisodicTrace> {
        self.episodic_memory.iter()
            .filter(|t| t.locus_fractal == Some(fractal_id))
            .collect()
    }

    /// Cerca episodi simili a una frase (per contenuto frattale).
    pub fn find_similar_episodes(&self, query_phrase: &PhrasePattern, threshold: f64) -> Vec<&EpisodicTrace> {
        self.episodic_memory.iter()
            .filter(|t| {
                // Similarita basata su frattali coinvolti
                let shared_fractals: usize = query_phrase.fractal_involvement.keys()
                    .filter(|fid| t.phrase.fractal_involvement.contains_key(fid))
                    .count();
                let total_fractals = query_phrase.fractal_involvement.len()
                    .max(t.phrase.fractal_involvement.len());
                if total_fractals == 0 { return false; }
                (shared_fractals as f64 / total_fractals as f64) >= threshold
            })
            .collect()
    }

    /// Trova gli N episodi più recenti.
    pub fn recent_episodes(&self, n: usize) -> Vec<&EpisodicTrace> {
        let start = if self.episodic_memory.len() > n { self.episodic_memory.len() - n } else { 0 };
        self.episodic_memory[start..].iter().collect()
    }

    /// Trova l'episodio più recente che soddisfa un predicato.
    pub fn find_recent<F>(&self, predicate: F) -> Option<&EpisodicTrace>
    where
        F: Fn(&EpisodicTrace) -> bool,
    {
        self.episodic_memory.iter().rev().find(|t| predicate(*t))
    }

    /// Conta episodi per speaker.
    pub fn count_by_speaker(&self, speaker: &str) -> usize {
        self.episodic_memory.iter().filter(|t| t.speaker == speaker).count()
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub stm_count: usize,
    pub mtm_count: usize,
    pub ltm_count: usize,
    pub current_tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::fractal::bootstrap_fractals;
    use crate::topology::simplex::bootstrap_complex;

    fn setup() -> (SimplicialComplex, TopologicalMemory) {
        let reg = bootstrap_fractals();
        let mut ids = reg.all_ids(); ids.sort();
        let complex = bootstrap_complex(&ids);
        let memory = TopologicalMemory::new();
        (complex, memory)
    }

    #[test]
    fn test_capture_imprint() {
        let (mut complex, mut memory) = setup();
        complex.activate_region(0, 0.8); // Attiva SPAZIO

        memory.capture(&complex, "test");
        assert_eq!(memory.short_term.len(), 1);
        assert!(!memory.short_term[0].active_simplices.is_empty());
    }

    #[test]
    fn test_consolidation() {
        let (mut complex, mut memory) = setup();
        memory.consolidation_threshold = 3;

        // Attiva gli stessi simplessi molte volte
        for _ in 0..10 {
            complex.activate_region(0, 0.8);
            memory.capture(&complex, "ripetuto");
        }

        memory.consolidate();
        assert!(!memory.medium_term.is_empty(), "Dopo consolidamento deve esserci MTM");
    }

    #[test]
    fn test_decay() {
        let (mut complex, mut memory) = setup();
        complex.activate_region(0, 0.8);
        memory.capture(&complex, "test");

        memory.consolidate(); // Forza consolidamento
        // Aggiungi manualmente in MTM per test
        memory.medium_term.push(FieldImprint {
            active_simplices: vec![(0, 0.5)],
            involved_fractals: vec![0],
            tick: 0,
            strength: 0.5,
            origin: "test".to_string(),
        });

        memory.decay(0.1);
        assert!(memory.medium_term[0].strength < 0.5);
    }
}
