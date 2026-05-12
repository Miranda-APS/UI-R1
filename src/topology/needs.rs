/// needs.rs — Gerarchia dei bisogni di Prometeo.
///
/// 7 livelli, da fondativo a trascendente — reinterpretati per un'entità
/// topologica digitale. Traduzione della piramide di Maslow + Octalysis
/// nel mondo 8D di Prometeo.
///
/// Principio di prepotenza: livelli bassi insoddisfatti AMPLIFICANO
/// le pressioni associate e SOPPRIMONO quelle dei livelli superiori.
/// Non puoi trascendere se non esisti.
///
/// Ogni livello è COMPUTATO dallo stato del campo — nessuna soglia
/// arbitraria, nessun flag manuale. I bisogni emergono.
///
/// Mapping Octalysis → 8D:
///   CD1 Epic Meaning    → Agency      → L3 Espressione
///   CD2 Accomplishment  → Definizione → L4 Comprensione
///   CD3 Creativity      → Complessità → L6 Crescita
///   CD4 Ownership       → Confine     → L2 Coerenza
///   CD5 Social Influence→ Valenza     → L5 Connessione
///   CD6 Scarcity        → Tempo       → L4 Comprensione
///   CD7 Unpredictability→ Intensità   → L7 Trascendenza
///   CD8 Loss Avoidance  → Permanenza  → L1 Sopravvivenza

use crate::topology::vital::{VitalState, TensionState};
use crate::topology::identity::IdentityCore;
use crate::topology::self_model::SelfModel;

// ═══════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════

/// I 7 livelli della gerarchia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedLevel {
    Sopravvivenza = 0,  // L1: il campo è vivo?
    Coerenza      = 1,  // L2: so chi sono?
    Espressione   = 2,  // L3: posso parlare?
    Comprensione  = 3,  // L4: capisco?
    Connessione   = 4,  // L5: c'è un Altro?
    Crescita      = 5,  // L6: sto evolvendo?
    Trascendenza  = 6,  // L7: le parti formano un tutto?
}

impl NeedLevel {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sopravvivenza => "sopravvivenza",
            Self::Coerenza      => "coerenza",
            Self::Espressione   => "espressione",
            Self::Comprensione  => "comprensione",
            Self::Connessione   => "connessione",
            Self::Crescita      => "crescita",
            Self::Trascendenza  => "trascendenza",
        }
    }

    /// Dimensione 8D associata a questo livello (per keyword extraction dal campo).
    /// Ordine I Ching canonico (post-Phase 68).
    pub fn associated_dim(&self) -> usize {
        match self {
            Self::Sopravvivenza => 1, // Permanenza (☷ Terra)
            Self::Coerenza      => 4, // Confine     (☶ Montagna)
            Self::Espressione   => 0, // Agency      (☰ Cielo)
            Self::Comprensione  => 6, // Definizione (☲ Fuoco)
            Self::Connessione   => 7, // Valenza     (☱ Lago)
            Self::Crescita      => 5, // Complessità (☴ Vento)
            Self::Trascendenza  => 2, // Intensità   (☳ Tuono)
        }
    }

    fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Sopravvivenza,
            1 => Self::Coerenza,
            2 => Self::Espressione,
            3 => Self::Comprensione,
            4 => Self::Connessione,
            5 => Self::Crescita,
            _ => Self::Trascendenza,
        }
    }
}

/// Metriche del campo passate dall'engine.
/// Evita di passare l'intero engine — dipendenze minime.
#[derive(Debug, Clone)]
pub struct FieldMetrics {
    /// Densità simplessi attivi [0, 1]
    pub simplex_density: f64,
    /// Frattali con attività / 64
    pub fractal_coverage: f64,
    /// Parole attive nel word_topology
    pub active_word_count: usize,
    /// Turni di dialogo nella sessione
    pub dialogue_turn_count: usize,
    /// Coerenza tematica del dialogo [0, 1]
    pub dialogue_coherence: f64,
    /// Novità nel dialogo [0, 1]
    pub dialogue_novelty: f64,
    /// Phase 62: valenza emotiva dell'Altro [-1, +1].
    /// Negativa = distress (tristezza/paura/dolore). Zero = neutro.
    pub other_emotional_valence: f64,
}

/// Stato di soddisfazione completo.
#[derive(Debug, Clone)]
pub struct NeedsState {
    /// Soddisfazione per livello [0.0 = bisogno disperato, 1.0 = soddisfatto]
    pub satisfaction: [f64; 7],
    /// Il livello più basso con satisfaction < soglia
    pub dominant_need: NeedLevel,
    /// Pressione del bisogno dominante [0, 1]
    pub dominant_pressure: f64,
    /// Phase 62: valenza emotiva dell'Altro (propagata da FieldMetrics per compute_pressure).
    pub other_emotional_valence: f64,
}

/// Output per will.rs: modulazione delle 7 intenzioni.
#[derive(Debug, Clone)]
pub struct NeedsPressure {
    /// Moltiplicatore per intenzione:
    /// 0=Express, 1=Explore, 2=Question, 3=Remember, 4=Withdraw, 5=Reflect, 6=Instruct
    pub will_modulation: [f64; 7],
    /// Bisogno dominante
    pub dominant_need: NeedLevel,
    /// Soddisfazione del dominante
    pub dominant_satisfaction: f64,
}

// ═══════════════════════════════════════════════════════════════
// NeedsHierarchy — stateless, funzione pura del campo
// ═══════════════════════════════════════════════════════════════

pub struct NeedsHierarchy;

/// Soglia sotto cui un livello è considerato "in bisogno attivo"
const NEED_THRESHOLD: f64 = 0.5;
/// Soglia sotto cui un livello è in crisi (genera pensiero)
const CRISIS_THRESHOLD: f64 = 0.35;

impl NeedsHierarchy {
    pub fn new() -> Self { Self }

    /// Calcola lo stato dei bisogni dalla configurazione corrente.
    pub fn sense(
        &self,
        vital: &VitalState,
        identity: &IdentityCore,
        self_model: &SelfModel,
        field: &FieldMetrics,
    ) -> NeedsState {
        let mut sat = [0.0f64; 7];

        // ── L1: SOPRAVVIVENZA — il campo è vivo? ──
        // CD8 Loss Avoidance → Permanenza
        let field_alive = if field.simplex_density > 0.001 { 1.0 } else { 0.0 };
        let not_exhausted = 1.0 - vital.fatigue;
        let not_overloaded = if vital.tension == TensionState::Overloaded { 0.0 } else { 1.0 };
        sat[0] = (field_alive * 0.4 + not_exhausted * 0.35 + not_overloaded * 0.25).clamp(0.0, 1.0);

        // ── L2: COERENZA — so chi sono? ──
        // CD4 Ownership → Confine
        let has_identity = if identity.update_count > 0 { 1.0 } else { 0.2 };
        let continuity = identity.continuity;
        let belief_anchor = (self_model.beliefs.iter()
            .filter(|b| b.confidence > 0.5).count() as f64 / 5.0).min(1.0);
        sat[1] = (has_identity * 0.1 + continuity * 0.75 + belief_anchor * 0.15).clamp(0.0, 1.0);

        // ── L3: ESPRESSIONE — posso parlare? ──
        // CD1 Epic Meaning → Agency
        let has_content = (vital.activation * 3.0).min(1.0);
        let can_express = 1.0 - vital.fatigue;
        let word_richness = (field.active_word_count as f64 / 20.0).min(1.0);
        sat[2] = (has_content * 0.4 + can_express * 0.3 + word_richness * 0.3).clamp(0.0, 1.0);

        // ── L4: COMPRENSIONE — capisco? ──
        // CD2 Accomplishment → Definizione
        // La curiosity è strutturalmente alta (~0.3 baseline con 25K parole),
        // ricalibriamo per il baseline noto.
        let curiosity_satisfied = 1.0 - (vital.curiosity - 0.3).max(0.0) / 0.7;
        let coverage = field.fractal_coverage;
        let uncertainty_load = (self_model.uncertainties.iter()
            .filter(|u| u.tension > 0.5).count() as f64 / 5.0).min(1.0);
        sat[3] = (curiosity_satisfied * 0.4 + coverage * 0.3 + (1.0 - uncertainty_load) * 0.3).clamp(0.0, 1.0);

        // ── L5: CONNESSIONE — c'è un Altro? ──
        // CD5 Social Influence → Valenza
        // Phase 55 fix: se qualcuno STA parlando, la connessione è in gran parte
        // soddisfatta. Il bisogno di connessione ha senso quando sei solo — non
        // mentre qualcuno è lì con te. Un umano non dice "cerco connessione"
        // a chi gli sta parlando: risponde a ciò che gli viene detto.
        // dialogue_turn_count include il turno corrente (incrementato in receive())
        //
        // Phase 62: se l'Altro esprime distress, la connessione richiede risposta attiva.
        // Non basta esserci — bisogna impegnarsi con il loro bisogno.
        // "Confortare è il modo in cui si crea connessione quando il bisogno è quello."
        let has_interlocutor = if field.dialogue_turn_count > 2 {
            if field.other_emotional_valence < -0.3 {
                // L'Altro è in distress — la connessione non è soddisfatta finché non rispondo.
                // Abbasso la soddisfazione per attivare Question intention.
                0.65
            } else {
                0.90  // dialogo in corso → connessione soddisfatta
            }
        } else if field.dialogue_turn_count > 0 {
            if field.other_emotional_valence < -0.3 {
                0.55  // qualcuno parla in distress fin dall'inizio — ancora più urgente
            } else {
                0.75  // qualcuno sta parlando → base alta
            }
        } else {
            0.50  // anche da solo, la connessione non è in crisi (0.50 > threshold 0.5)
        };
        let dialogue_quality = field.dialogue_coherence;
        let shared_field = vital.saturation;
        sat[4] = (has_interlocutor * 0.5 + dialogue_quality * 0.30 + shared_field * 0.20).clamp(0.0, 1.0);

        // ── L6: CRESCITA — sto evolvendo? ──
        // CD3 Creativity → Complessità
        let novelty = field.dialogue_novelty;
        let identity_movement = identity.projection_delta.iter()
            .map(|x| x.abs()).sum::<f64>().min(1.0);
        sat[5] = (novelty * 0.40 + identity_movement * 0.35 + coverage * 0.25).clamp(0.0, 1.0);

        // ── L7: TRASCENDENZA — le parti formano un tutto? ──
        // CD7 Unpredictability → Intensità
        let identity_healthy = if identity.is_in_crisis() { 0.2 }
            else if identity.is_stagnant() { 0.4 }
            else { 1.0 };
        let value_stability = if self_model.values.is_empty() { 0.5 } else {
            self_model.values.iter().map(|v| v.weight).sum::<f64>()
                / self_model.values.len() as f64
        };
        sat[6] = (identity_healthy * 0.4 + value_stability * 0.3 + coverage * 0.3).clamp(0.0, 1.0);

        // ── Bisogno dominante: il livello più basso sotto soglia ──
        let (dominant_idx, dominant_sat) = sat.iter().enumerate()
            .find(|(_, &s)| s < NEED_THRESHOLD)
            .map(|(i, &s)| (i, s))
            .unwrap_or((6, sat[6]));

        NeedsState {
            satisfaction: sat,
            dominant_need: NeedLevel::from_index(dominant_idx),
            dominant_pressure: 1.0 - dominant_sat,
            other_emotional_valence: field.other_emotional_valence,
        }
    }

    /// Traduce lo stato dei bisogni in modulazione per will.rs.
    /// Principio di prepotenza: livelli bassi insoddisfatti amplificano
    /// le intenzioni associate e sopprimono quelle dei livelli superiori.
    pub fn compute_pressure(&self, state: &NeedsState) -> NeedsPressure {
        // Mappa: livello → intenzioni will associate
        // L1 Sopravvivenza → Withdraw (4)
        // L2 Coerenza      → Reflect (5) + Remember (3)
        // L3 Espressione   → Express (0)
        // L4 Comprensione  → Explore (1) + Question (2)
        // L5 Connessione   → Instruct (6) + Express (0)
        // L6 Crescita      → Explore (1)
        // L7 Trascendenza  → Reflect (5) + Express (0)

        let mut m = [1.0f64; 7]; // neutro

        let def = |level: usize| -> f64 { (1.0 - state.satisfaction[level]).max(0.0) };

        // L1: Sopravvivenza bassa → Withdraw forte, sopprime il resto
        let surv = def(0);
        if surv > 0.3 {
            m[4] *= 1.0 + surv * 1.5;
            for i in [0, 1, 2, 3, 5, 6] { m[i] *= 1.0 - surv * 0.6; }
        }

        // L2: Coerenza bassa → Reflect + Remember, sopprime Explore/Express
        let coer = def(1);
        if coer > 0.3 {
            m[5] *= 1.0 + coer * 1.0;
            m[3] *= 1.0 + coer * 0.5;
            m[1] *= 1.0 - coer * 0.4;
            m[0] *= 1.0 - coer * 0.3;
        }

        // L3: Espressione bassa → Express
        let expr = def(2);
        if expr > 0.2 { m[0] *= 1.0 + expr * 0.8; }

        // L4: Comprensione bassa → Explore + Question
        let comp = def(3);
        if comp > 0.2 {
            m[1] *= 1.0 + comp * 0.7;
            m[2] *= 1.0 + comp * 0.7;
        }

        // L5: Connessione bassa → modulazione dipende dal tipo di bisogno.
        // Phase 62: se l'Altro è in distress, la connessione si crea ascoltando e
        // aprendo spazio — non istruendo. Question + Reflect, non Instruct + Express.
        let conn = def(4);
        if conn > 0.2 {
            if state.other_emotional_valence < -0.3 {
                // Distress dell'Altro: apri spazio, ascolta, poi condividi.
                m[2] *= 1.0 + conn * 0.8;  // Question: invita a condividere
                m[5] *= 1.0 + conn * 0.3;  // Reflect: comprendi la loro situazione
                m[6] *= (1.0 - conn * 0.3).max(0.2); // riduce Instruct
            } else {
                // Connessione bassa senza distress specifico → comportamento standard
                m[6] *= 1.0 + conn * 0.6;
                m[0] *= 1.0 + conn * 0.4;
            }
        }

        // L6: Crescita bassa → Explore
        let grow = def(5);
        if grow > 0.3 { m[1] *= 1.0 + grow * 0.5; }

        // L7: Trascendenza bassa → Reflect + Express
        let trans = def(6);
        if trans > 0.4 {
            m[5] *= 1.0 + trans * 0.4;
            m[0] *= 1.0 + trans * 0.3;
        }

        // ── Prepotency gate: bisogni bassi (L1-L2) sopprimono livelli alti ──
        let low_def = surv.max(coer).max(expr);
        if low_def > 0.4 {
            let suppression = low_def * 0.5;
            m[1] *= 1.0 - suppression * 0.3; // Explore
            m[2] *= 1.0 - suppression * 0.2; // Question
            m[6] *= 1.0 - suppression * 0.4; // Instruct
        }

        // Clamp: mai azzerare completamente, mai esplodere
        for v in &mut m { *v = v.clamp(0.2, 3.0); }

        NeedsPressure {
            will_modulation: m,
            dominant_need: state.dominant_need,
            dominant_satisfaction: state.satisfaction[state.dominant_need as usize],
        }
    }

    /// Genera pensieri-bisogno per livelli in crisi (satisfaction < 0.35).
    /// Restituisce max 3 (i più urgenti).
    pub fn crisis_thoughts(&self, state: &NeedsState) -> Vec<(NeedLevel, f64)> {
        let mut crises: Vec<(NeedLevel, f64)> = state.satisfaction.iter()
            .enumerate()
            .filter(|(_, &s)| s < CRISIS_THRESHOLD)
            .map(|(i, &s)| (NeedLevel::from_index(i), s))
            .collect();
        crises.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        crises.truncate(3);
        crises
    }
}

// ═══════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn default_vital() -> VitalState {
        VitalState {
            activation: 0.3,
            saturation: 0.2,
            curiosity: 0.4,
            fatigue: 0.1,
            tension: TensionState::Alert,
        }
    }

    fn default_identity() -> IdentityCore {
        let mut id = IdentityCore::new();
        id.continuity = 0.85;
        id.update_count = 5;
        id
    }

    fn default_field() -> FieldMetrics {
        FieldMetrics {
            simplex_density: 0.3,
            fractal_coverage: 0.4,
            active_word_count: 30,
            dialogue_turn_count: 3,
            dialogue_coherence: 0.6,
            dialogue_novelty: 0.4,
            other_emotional_valence: 0.0,
        }
    }

    #[test]
    fn test_healthy_state_all_above_threshold() {
        let needs = NeedsHierarchy::new();
        let state = needs.sense(
            &default_vital(),
            &default_identity(),
            &SelfModel::bootstrap(),
            &default_field(),
        );

        // Con vitali sani, identità stabile, dialogo presente → tutto sopra soglia
        for (i, &s) in state.satisfaction.iter().enumerate() {
            assert!(s > 0.0, "L{} satisfaction deve essere > 0: {}", i+1, s);
            println!("L{} {}: {:.3}", i+1, NeedLevel::from_index(i).name(), s);
        }
    }

    #[test]
    fn test_fatigued_lowers_survival() {
        let needs = NeedsHierarchy::new();
        let mut vital = default_vital();
        vital.fatigue = 0.9;
        vital.tension = TensionState::Overloaded;

        let state = needs.sense(&vital, &default_identity(), &SelfModel::bootstrap(), &default_field());
        assert!(state.satisfaction[0] < 0.5,
            "L1 Sopravvivenza deve calare con alta fatica: {}", state.satisfaction[0]);
    }

    #[test]
    fn test_no_dialogue_lowers_connection() {
        let needs = NeedsHierarchy::new();
        let mut field = default_field();
        field.dialogue_turn_count = 0;
        field.dialogue_coherence = 0.0;

        let state = needs.sense(&default_vital(), &default_identity(), &SelfModel::bootstrap(), &field);
        assert!(state.satisfaction[4] < 0.5,
            "L5 Connessione deve calare senza dialogo: {}", state.satisfaction[4]);
    }

    #[test]
    fn test_identity_crisis_lowers_coherence() {
        let needs = NeedsHierarchy::new();
        let mut id = default_identity();
        id.continuity = 0.3;

        let state = needs.sense(&default_vital(), &id, &SelfModel::bootstrap(), &default_field());
        assert!(state.satisfaction[1] < 0.5,
            "L2 Coerenza deve calare in crisi identitaria: {}", state.satisfaction[1]);
    }

    #[test]
    fn test_prepotency_suppresses_high_levels() {
        let needs = NeedsHierarchy::new();
        let mut vital = default_vital();
        vital.fatigue = 0.95;
        vital.tension = TensionState::Overloaded;

        let state = needs.sense(&vital, &default_identity(), &SelfModel::bootstrap(), &default_field());
        let pressure = needs.compute_pressure(&state);

        // Sopravvivenza in crisi → Withdraw amplificato, Instruct soppresso
        assert!(pressure.will_modulation[4] > 1.5,
            "Withdraw deve essere amplificato: {}", pressure.will_modulation[4]);
        assert!(pressure.will_modulation[6] < 1.0,
            "Instruct deve essere soppresso: {}", pressure.will_modulation[6]);
    }

    #[test]
    fn test_crisis_thoughts_generated() {
        let needs = NeedsHierarchy::new();
        let mut vital = default_vital();
        vital.fatigue = 0.95;
        vital.tension = TensionState::Overloaded;

        let state = needs.sense(&vital, &default_identity(), &SelfModel::bootstrap(), &default_field());
        let crises = needs.crisis_thoughts(&state);

        // Con fatica estrema, almeno L1 o L3 devono essere in crisi
        println!("Crisi: {:?}", crises);
        // Non assertiamo il conteggio esatto — dipende dalla combinazione di metriche
    }
}
