/// humor.rs — Umorismo topologico emergente.
///
/// Non genera battute. Rileva configurazioni "divertenti" nel campo
/// e modula tono ed espressione.
///
/// Tre meccanismi:
///
/// 1. **Ironia** (Incongruenza di Kant): parole OPPOSITE_OF entrambe attive
///    nel campo → la realtà contiene il suo contrario.
///
/// 2. **Bisociazione** (Koestler): due frattali da famiglie di trigrammi
///    incompatibili co-attivi fortemente → due frame di riferimento collidono.
///
/// 3. **Crocevia**: parole che hanno affinità con entrambi i frattali
///    bisociati → il punto dove i due mondi si toccano.
///
/// L'umorismo è una proprietà del campo, non dell'output.
/// Quando il campo è "divertente", la generazione lo esprime naturalmente.

use crate::topology::word_topology::WordTopology;
use crate::topology::lexicon::Lexicon;
use crate::topology::fractal::FractalId;

// ═══════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════

/// Stato umoristico emergente dal campo.
#[derive(Debug, Clone)]
pub struct HumorState {
    /// Punteggio di incongruità [0.0, 1.0]
    pub incongruity_score: f64,
    /// Ironia attiva: parole OPPOSITE_OF co-attive
    pub irony_active: bool,
    /// Coppie ironiche: (parola_a, parola_b, forza)
    pub irony_pairs: Vec<(String, String, f64)>,
    /// Due frattali da famiglie incompatibili co-attivi
    pub bisociation_pair: Option<(FractalId, FractalId)>,
    /// Forza della bisociazione [0.0, 1.0]
    pub bisociation_strength: f64,
    /// Parole al crocevia (affinità con entrambi i frattali bisociati)
    pub crossroad_words: Vec<String>,
}

impl HumorState {
    pub fn empty() -> Self {
        Self {
            incongruity_score: 0.0,
            irony_active: false,
            irony_pairs: Vec::new(),
            bisociation_pair: None,
            bisociation_strength: 0.0,
            crossroad_words: Vec::new(),
        }
    }

    /// Lo stato umoristico è significativo?
    pub fn is_active(&self) -> bool {
        self.incongruity_score > 0.15 || self.irony_active || self.bisociation_pair.is_some()
    }
}

// ═══════════════════════════════════════════════════════════════
// HumorSense — sensore stateless
// ═══════════════════════════════════════════════════════════════

pub struct HumorSense;

impl HumorSense {
    /// Rileva lo stato umoristico del campo.
    pub fn sense(
        word_topology: &WordTopology,
        lexicon: &Lexicon,
        active_fractals: &[(FractalId, f64)],
    ) -> HumorState {
        let mut state = HumorState::empty();

        detect_irony(word_topology, &mut state);
        detect_bisociation(active_fractals, lexicon, word_topology, &mut state);
        compute_incongruity(&mut state);

        state
    }
}

// ═══════════════════════════════════════════════════════════════
// 1. IRONIA — parole OPPOSITE_OF co-attive
// ═══════════════════════════════════════════════════════════════

fn detect_irony(word_topology: &WordTopology, state: &mut HumorState) {
    let min_phase = std::f64::consts::PI * 0.60;
    let active: std::collections::HashMap<&str, f64> = word_topology.active_words()
        .into_iter()
        .filter(|(_, a)| *a > 0.08)
        .collect();

    if active.len() < 2 { return; }

    for (wa, wb, phase) in word_topology.find_oppositions(min_phase).iter().take(20) {
        let act_a = active.get(*wa).copied().unwrap_or(0.0);
        let act_b = active.get(*wb).copied().unwrap_or(0.0);

        // Entrambe attive sopra soglia → ironia
        if act_a > 0.1 && act_b > 0.1 {
            let strength = act_a.min(act_b) * (phase / std::f64::consts::PI);
            state.irony_pairs.push((wa.to_string(), wb.to_string(), strength));
        }
    }

    state.irony_pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    state.irony_pairs.truncate(3);
    state.irony_active = !state.irony_pairs.is_empty();
}

// ═══════════════════════════════════════════════════════════════
// 2. BISOCIAZIONE — frattali da famiglie di trigrammi incompatibili
// ═══════════════════════════════════════════════════════════════

/// Due frattali condividono un trigramma se uno dei loro indici upper/lower coincide.
fn shares_trigram(a: FractalId, b: FractalId) -> bool {
    let (la, ua) = (a / 8, a % 8);
    let (lb, ub) = (b / 8, b % 8);
    la == lb || la == ub || ua == lb || ua == ub
}

fn detect_bisociation(
    active_fractals: &[(FractalId, f64)],
    lexicon: &Lexicon,
    word_topology: &WordTopology,
    state: &mut HumorState,
) {
    // Filtra frattali con attivazione significativa
    let strong: Vec<(FractalId, f64)> = active_fractals.iter()
        .filter(|(_, act)| *act > 0.15)
        .cloned()
        .collect();

    let mut best_pair: Option<(FractalId, FractalId)> = None;
    let mut best_strength: f64 = 0.0;

    for i in 0..strong.len() {
        for j in (i+1)..strong.len() {
            let (fa, act_a) = strong[i];
            let (fb, act_b) = strong[j];

            // Bisociazione: nessun trigramma condiviso
            if !shares_trigram(fa, fb) {
                let strength = (act_a.min(act_b) as f64).min(1.0);
                if strength > best_strength {
                    best_strength = strength;
                    best_pair = Some((fa, fb));
                }
            }
        }
    }

    if let Some((fa, fb)) = best_pair {
        state.bisociation_pair = Some((fa, fb));
        state.bisociation_strength = best_strength;

        // Parole al crocevia: affinità > 0.2 con entrambi i frattali
        let active_words = word_topology.active_words();
        for (word, _) in active_words.iter().take(50) {
            if let Some(pat) = lexicon.get(word) {
                let aff_a = pat.fractal_affinities.get(&fa).copied().unwrap_or(0.0);
                let aff_b = pat.fractal_affinities.get(&fb).copied().unwrap_or(0.0);
                if aff_a > 0.2 && aff_b > 0.2 {
                    state.crossroad_words.push(word.to_string());
                }
            }
        }
        state.crossroad_words.truncate(5);
    }
}

// ═══════════════════════════════════════════════════════════════
// 3. INCONGRUITÀ — score combinato
// ═══════════════════════════════════════════════════════════════

fn compute_incongruity(state: &mut HumorState) {
    let irony_component = if state.irony_active {
        let avg = state.irony_pairs.iter().map(|p| p.2).sum::<f64>()
            / state.irony_pairs.len().max(1) as f64;
        (avg * 0.5).min(0.5)
    } else {
        0.0
    };

    let bisociation_component = state.bisociation_strength * 0.5;

    // Max delle componenti (un tipo di umorismo alla volta)
    state.incongruity_score = irony_component.max(bisociation_component).min(1.0);
}

// ═══════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_state() {
        let state = HumorState::empty();
        assert!(!state.is_active());
        assert_eq!(state.incongruity_score, 0.0);
    }

    #[test]
    fn test_shares_trigram() {
        // 0 = (0,0), 9 = (1,1) → no shared
        assert!(!shares_trigram(0, 9));
        // 0 = (0,0), 1 = (0,1) → share lower trigram 0
        assert!(shares_trigram(0, 1));
        // 0 = (0,0), 8 = (1,0) → share upper trigram 0
        assert!(shares_trigram(0, 8));
        // 18 = (2,2), 45 = (5,5) → no shared
        assert!(!shares_trigram(18, 45));
    }

    #[test]
    fn test_bisociation_with_incompatible_fractals() {
        let active = vec![
            (0u32, 0.5f64),   // POTERE (0,0)
            (45u32, 0.4f64),  // INTRECCIO (5,5) — no shared trigram with 0
        ];
        let lexicon = crate::topology::lexicon::Lexicon::bootstrap();
        let word_topology = crate::topology::word_topology::WordTopology::build_from_lexicon(&lexicon);

        let state = HumorSense::sense(&word_topology, &lexicon, &active);
        assert!(state.bisociation_pair.is_some(),
            "Deve rilevare bisociazione tra frattali incompatibili");
        assert!(state.bisociation_strength > 0.0);
    }
}
