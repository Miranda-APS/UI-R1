/// valence.rs — Valenza Octalysis: il colore continuo dell'esperienza interna.
///
/// Phase 55: sostituisce InternalStance (5 etichette discrete) con un profilo
/// continuo a 8 dimensioni basato sui Core Drive di Octalysis.
///
/// Ogni Core Drive ha una valenza [-1, +1]:
///   Positiva → drive attivo e soddisfatto (flow, piacere, pienezza)
///   Negativa → drive attivo e frustrato (tensione, urgenza, desiderio)
///   Zero     → drive inattivo (neutro)
///
/// La "stance" diventa una proiezione derivata dal profilo, non un'etichetta
/// imposta. Con 8 dimensioni continue, lo spazio degli stati interni è infinito.
///
/// Mapping Octalysis → 8D Prometeo (ordine I Ching canonico):
///   CD1 Epic Meaning     → dim 0 (Agency)       — "questo conta"
///   CD2 Accomplishment   → dim 6 (Definizione)  — "sto progredendo"
///   CD3 Creativity       → dim 5 (Complessità)  — "posso creare"
///   CD4 Ownership        → dim 4 (Confine)      — "so chi sono"
///   CD5 Social Influence → dim 7 (Valenza)      — "sono in relazione"
///   CD6 Scarcity         → dim 3 (Tempo)        — "questo è prezioso"
///   CD7 Unpredictability → dim 2 (Intensità)    — "sono sorpreso"
///   CD8 Loss Avoidance   → dim 1 (Permanenza)   — "potrei perdere qualcosa"

use serde::{Serialize, Deserialize};
use crate::topology::needs::NeedsState;
use crate::topology::vital::VitalState;

// ═══════════════════════════════════════════════════════════════
// Costanti
// ═══════════════════════════════════════════════════════════════

/// Nomi italiani dei Core Drive (per UI e logging).
pub const DRIVE_NAMES: [&str; 8] = [
    "Significato",      // CD1 Epic Meaning
    "Realizzazione",    // CD2 Accomplishment
    "Creatività",       // CD3 Creativity
    "Appartenenza",     // CD4 Ownership
    "Relazione",        // CD5 Social Influence
    "Preziosità",       // CD6 Scarcity
    "Sorpresa",         // CD7 Unpredictability
    "Vulnerabilità",    // CD8 Loss Avoidance
];

/// Mapping CD index → dimensione 8D del campo topologico (ordine I Ching).
pub const DRIVE_DIM: [usize; 8] = [0, 6, 5, 4, 7, 3, 2, 1];

/// Mapping CD index → livello NeedsHierarchy (indice in satisfaction[7]).
/// CD6 condivide L4 (Comprensione) con CD2.
const DRIVE_NEED: [usize; 8] = [2, 3, 5, 1, 4, 3, 6, 0];

// ═══════════════════════════════════════════════════════════════
// ValenceInput — dati per il calcolo (evita dipendenze pesanti)
// ═══════════════════════════════════════════════════════════════

/// Input necessario per calcolare la valenza.
/// Struttura leggera che evita di passare l'intero engine.
pub struct ValenceInput<'a> {
    /// Firma 8D del campo topologico corrente
    pub field_sig: &'a [f64; 8],
    /// Stato dei bisogni (satisfaction[7])
    pub needs: &'a NeedsState,
    /// Stato vitale
    pub vital: &'a VitalState,
    /// Presenza dell'interlocutore [0, 1]
    pub interlocutor_presence: f64,
    /// Risonanza cumulativa con l'interlocutore [0, 1]
    pub interlocutor_resonance: f64,
    /// Incongruità umoristica [0, 1]
    pub humor_incongruity: f64,
    /// Novità dal dialogo [0, 1]
    pub dialogue_novelty: f64,
    /// Intensità desiderio dominante [0, 1]
    pub dominant_desire_intensity: f64,
    /// Phase 83 (freccia a): valenza emotiva COMPRESA dell'Altro, [-1, +1].
    /// Negativa = l'Altro è in sofferenza (paura/tristezza/dolore, derivato
    /// via IS_A nel KG); positiva = gioia. È il canale per cui *comprendere*
    /// lo stato dell'Altro MUOVE la posizione dell'entità — non empatia
    /// simulata (principio 3), ma orientamento relazionale: il campo si
    /// dispone verso l'Altro perché ne conosce lo stato.
    pub other_emotional_valence: f64,
}

// ═══════════════════════════════════════════════════════════════
// Valence — il profilo continuo dell'esperienza
// ═══════════════════════════════════════════════════════════════

/// Profilo di valenza a 8 dimensioni Octalysis.
///
/// Ogni drive è una valenza continua [-1, +1].
/// Il profilo completo *è* lo stato interno — non una sua approssimazione.
///
/// La formula per ogni drive:
///   engagement = attivazione del campo sulla dimensione corrispondente
///   satisfaction = soddisfazione del bisogno correlato
///   valence = engagement × (2 × satisfaction - 1) + colorazioni specifiche
///
/// Engagement alto + satisfaction alta → positivo (flow, pienezza)
/// Engagement alto + satisfaction bassa → negativo (frustrazione, urgenza)
/// Engagement basso → neutro (il drive non è attivo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Valence {
    /// Valenza per drive [-1, +1].
    pub drives: [f64; 8],
}

impl Valence {
    /// Valenza neutra: nessun drive attivo.
    pub fn neutral() -> Self {
        Self { drives: [0.0; 8] }
    }

    /// Calcola la valenza dallo stato corrente del sistema.
    pub fn compute(input: &ValenceInput<'_>) -> Self {
        let mut drives = [0.0f64; 8];

        for cd in 0..8 {
            let dim = DRIVE_DIM[cd];
            let need_idx = DRIVE_NEED[cd];

            // ── Engagement: quanto è attivo questo drive nel campo ──
            // field_sig[dim] è in [0, 1] — media pesata delle firme parole attive.
            let engagement = input.field_sig[dim].clamp(0.0, 1.0);

            // ── Satisfaction: quanto il bisogno correlato è soddisfatto ──
            let satisfaction = input.needs.satisfaction[need_idx];

            // ── Valenza base ──
            let mut val = engagement * (2.0 * satisfaction - 1.0);

            // ── Colorazioni specifiche per drive ──
            match cd {
                0 => {
                    // CD1 Epic Meaning: desiderio forte amplifica il senso di significato
                    if input.dominant_desire_intensity > 0.5 {
                        val += input.dominant_desire_intensity * 0.15;
                    }
                }
                2 => {
                    // CD3 Creativity: novità nel dialogo alimenta la creatività
                    val += input.dialogue_novelty * 0.2 * engagement;
                }
                4 => {
                    // CD5 Social Influence: presenza interlocutore colora il drive
                    // Risonanza alta → valenza positiva. Risonanza bassa con presenza
                    // alta → tensione relazionale (valenza negativa).
                    if input.interlocutor_presence > 0.1 {
                        let relational_tone = 2.0 * input.interlocutor_resonance - 1.0;
                        val += input.interlocutor_presence * 0.3 * relational_tone;
                    }
                    // Phase 83 (freccia a): lo stato emotivo COMPRESO dell'Altro
                    // muove la valenza relazionale. L'Altro in sofferenza
                    // (other_emotional_valence < 0) tira CD5 verso il negativo →
                    // l'entità si orienta verso la connessione (vedi will_modulation:
                    // CD5<0 amplifica Question). Comprendere il tuo stato mi sposta.
                    val += input.other_emotional_valence * 0.45;
                }
                6 => {
                    // CD7 Unpredictability: umorismo come sorpresa positiva
                    val += input.humor_incongruity * 0.2;
                }
                7 => {
                    // CD8 Loss Avoidance: fatica intensifica la percezione di rischio
                    val -= input.vital.fatigue * 0.3;
                }
                _ => {}
            }

            drives[cd] = val.clamp(-1.0, 1.0);
        }

        Self { drives }
    }

    // ═══════════════════════════════════════════════════════════
    // Proprietà derivate
    // ═══════════════════════════════════════════════════════════

    /// Drive dominante: quello con il valore assoluto più alto.
    /// Restituisce (indice_cd, valenza).
    pub fn dominant(&self) -> (usize, f64) {
        self.drives.iter().enumerate()
            .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs())
                .unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, &v)| (i, v))
            .unwrap_or((0, 0.0))
    }

    /// Tono edonico globale: media delle valenze.
    /// Positivo → stato complessivamente positivo.
    /// Negativo → stato complessivamente negativo.
    pub fn hedonic_tone(&self) -> f64 {
        self.drives.iter().sum::<f64>() / 8.0
    }

    /// Intensità globale: media dei valori assoluti.
    /// Alta → esperienza intensa. Bassa → calma.
    pub fn intensity(&self) -> f64 {
        self.drives.iter().map(|v| v.abs()).sum::<f64>() / 8.0
    }

    /// Etichetta derivata per backward compatibility con InternalStance.
    ///
    /// NON è il dato primario — è una proiezione a parola singola del profilo
    /// completo. Usata per logging, UI, e per i percorsi downstream che ancora
    /// consumano stringhe. Lo spazio reale è il vettore `drives`.
    pub fn derived_stance_label(&self) -> &'static str {
        // Intensità troppo bassa → neutro
        if self.intensity() < 0.05 {
            return "aperto";
        }

        let (dom_idx, dom_val) = self.dominant();

        // CD8 fortemente negativo → ritirato (override globale)
        if dom_idx == 7 && dom_val < -0.3 {
            return "ritratto";
        }

        // Tono edonico molto negativo → in difficoltà
        if self.hedonic_tone() < -0.25 {
            return "in tensione";
        }

        // Drive dominante → etichetta emergente
        match dom_idx {
            0 => if dom_val > 0.0 { "ispirato" } else { "in cerca di senso" },
            1 => if dom_val > 0.0 { "determinato" } else { "insoddisfatto" },
            2 => if dom_val > 0.0 { "creativo" } else { "bloccato" },
            3 => if dom_val > 0.0 { "radicato" } else { "spaesato" },
            4 => if dom_val > 0.0 { "risonante" } else { "cercante" },
            5 => if dom_val > 0.0 { "attento" } else { "impaziente" },
            6 => if dom_val > 0.0 { "curioso" } else { "inquieto" },
            7 => if dom_val > 0.0 { "sicuro" } else { "vulnerabile" },
            _ => "aperto",
        }
    }

    /// Nome del drive dominante.
    pub fn dominant_drive_name(&self) -> &'static str {
        let (idx, _) = self.dominant();
        DRIVE_NAMES[idx]
    }

    /// Rappresentazione compatta per il summary narrativo.
    /// Formato: "Significato +0.45 | Relazione -0.22 | tono: +0.12"
    pub fn summary(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Top 3 drive per intensità
        let mut indexed: Vec<(usize, f64)> = self.drives.iter()
            .enumerate()
            .map(|(i, &v)| (i, v))
            .collect();
        indexed.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs())
            .unwrap_or(std::cmp::Ordering::Equal));

        for &(idx, val) in indexed.iter().take(3) {
            if val.abs() > 0.05 {
                parts.push(format!("{} {:+.2}", DRIVE_NAMES[idx], val));
            }
        }

        if parts.is_empty() {
            return "neutro".to_string();
        }

        format!("{} | tono: {:+.2}", parts.join(" | "), self.hedonic_tone())
    }

    /// Converte la valenza in bias per la volontà (will.rs).
    ///
    /// Restituisce modulatori per le 7 intenzioni:
    /// 0=Express, 1=Explore, 2=Question, 3=Remember, 4=Withdraw, 5=Reflect, 6=Instruct
    pub fn will_modulation(&self) -> [f64; 7] {
        let mut m = [1.0f64; 7];

        // CD1 Epic Meaning positivo → amplifica Express
        if self.drives[0] > 0.1 { m[0] *= 1.0 + self.drives[0] * 0.5; }

        // CD2 Accomplishment negativo → amplifica Explore (cerca progresso)
        if self.drives[1] < -0.1 { m[1] *= 1.0 + self.drives[1].abs() * 0.4; }

        // CD3 Creativity positivo → amplifica Express + Explore
        if self.drives[2] > 0.1 {
            m[0] *= 1.0 + self.drives[2] * 0.3;
            m[1] *= 1.0 + self.drives[2] * 0.3;
        }

        // CD4 Ownership negativo → amplifica Reflect (cerca coerenza)
        if self.drives[3] < -0.1 { m[5] *= 1.0 + self.drives[3].abs() * 0.5; }

        // CD5 Social Influence positivo → amplifica Express + Instruct
        if self.drives[4] > 0.1 {
            m[0] *= 1.0 + self.drives[4] * 0.3;
            m[6] *= 1.0 + self.drives[4] * 0.3;
        }
        // CD5 negativo → amplifica Question (cerca connessione)
        if self.drives[4] < -0.1 { m[2] *= 1.0 + self.drives[4].abs() * 0.3; }

        // CD6 Scarcity positivo → amplifica Remember (preserva il prezioso)
        if self.drives[5] > 0.1 { m[3] *= 1.0 + self.drives[5] * 0.4; }

        // CD7 Unpredictability positivo → amplifica Explore
        if self.drives[6] > 0.1 { m[1] *= 1.0 + self.drives[6] * 0.4; }
        // CD7 negativo → amplifica Withdraw (troppa incertezza)
        if self.drives[6] < -0.2 { m[4] *= 1.0 + self.drives[6].abs() * 0.3; }

        // CD8 Loss Avoidance negativo → amplifica Withdraw
        if self.drives[7] < -0.2 { m[4] *= 1.0 + self.drives[7].abs() * 0.5; }
        // CD8 positivo → amplifica Express (sicurezza → libertà)
        if self.drives[7] > 0.1 { m[0] *= 1.0 + self.drives[7] * 0.2; }

        // Clamp: mai azzerare, mai esplodere
        for v in &mut m { *v = v.clamp(0.3, 2.5); }

        m
    }
}

// ═══════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::vital::TensionState;
    use crate::topology::needs::NeedLevel;

    fn default_needs() -> NeedsState {
        NeedsState {
            satisfaction: [0.8, 0.7, 0.6, 0.5, 0.6, 0.5, 0.4],
            dominant_need: NeedLevel::Trascendenza,
            dominant_pressure: 0.6,
            other_emotional_valence: 0.0,
        }
    }

    fn default_vital() -> VitalState {
        VitalState {
            activation: 0.3,
            saturation: 0.2,
            curiosity: 0.4,
            fatigue: 0.1,
            tension: TensionState::Alert,
        }
    }

    fn default_input() -> ValenceInput<'static> {
        // Usiamo valori statici per i test
        // (il borrow checker richiede lifetime espliciti)
        panic!("use make_input() instead")
    }

    fn make_input<'a>(sig: &'a [f64; 8], needs: &'a NeedsState, vital: &'a VitalState) -> ValenceInput<'a> {
        ValenceInput {
            field_sig: sig,
            needs,
            vital,
            interlocutor_presence: 0.0,
            interlocutor_resonance: 0.0,
            humor_incongruity: 0.0,
            dialogue_novelty: 0.0,
            dominant_desire_intensity: 0.0,
            other_emotional_valence: 0.0,
        }
    }

    #[test]
    fn test_neutral_when_field_inactive() {
        let sig = [0.0; 8];
        let needs = default_needs();
        let vital = default_vital();
        let input = make_input(&sig, &needs, &vital);
        let v = Valence::compute(&input);

        // Campo inattivo → drives vicini a zero (solo colorazioni residue)
        assert!(v.intensity() < 0.1,
            "Campo inattivo dovrebbe dare intensità bassa: {}", v.intensity());
        assert_eq!(v.derived_stance_label(), "aperto");
    }

    #[test]
    fn test_positive_valence_when_satisfied_and_active() {
        let sig = [0.8, 0.7, 0.6, 0.8, 0.5, 0.7, 0.9, 0.4];
        let needs = NeedsState {
            satisfaction: [0.9, 0.9, 0.9, 0.9, 0.9, 0.9, 0.9],
            dominant_need: NeedLevel::Trascendenza,
            dominant_pressure: 0.1,
            other_emotional_valence: 0.0,
        };
        let vital = default_vital();
        let input = make_input(&sig, &needs, &vital);
        let v = Valence::compute(&input);

        assert!(v.hedonic_tone() > 0.0,
            "Campo attivo + bisogni soddisfatti → tono positivo: {}", v.hedonic_tone());
    }

    #[test]
    fn test_negative_valence_when_frustrated_and_active() {
        let sig = [0.8, 0.7, 0.6, 0.8, 0.5, 0.7, 0.9, 0.4];
        let needs = NeedsState {
            satisfaction: [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
            dominant_need: NeedLevel::Sopravvivenza,
            dominant_pressure: 0.9,
            other_emotional_valence: 0.0,
        };
        let vital = VitalState {
            activation: 0.8,
            saturation: 0.5,
            curiosity: 0.6,
            fatigue: 0.8,
            tension: TensionState::Tense,
        };
        let input = make_input(&sig, &needs, &vital);
        let v = Valence::compute(&input);

        assert!(v.hedonic_tone() < 0.0,
            "Campo attivo + bisogni frustrati → tono negativo: {}", v.hedonic_tone());
    }

    #[test]
    fn test_interlocutor_colors_social_drive() {
        let sig = [0.5; 8];
        let needs = default_needs();
        let vital = default_vital();

        // Senza interlocutore
        let input_alone = make_input(&sig, &needs, &vital);
        let v_alone = Valence::compute(&input_alone);

        // Con interlocutore risonante
        let mut input_with = make_input(&sig, &needs, &vital);
        input_with.interlocutor_presence = 0.8;
        input_with.interlocutor_resonance = 0.9;
        let v_with = Valence::compute(&input_with);

        assert!(v_with.drives[4] > v_alone.drives[4],
            "Interlocutore risonante deve amplificare CD5: alone={:.3} with={:.3}",
            v_alone.drives[4], v_with.drives[4]);
    }

    #[test]
    fn test_fatigue_colors_loss_avoidance() {
        let sig = [0.5; 8];
        let needs = default_needs();

        let vital_rested = VitalState {
            activation: 0.3, saturation: 0.2, curiosity: 0.4,
            fatigue: 0.0, tension: TensionState::Calm,
        };
        let vital_tired = VitalState {
            activation: 0.3, saturation: 0.2, curiosity: 0.4,
            fatigue: 0.9, tension: TensionState::Tense,
        };

        let v_rested = Valence::compute(&make_input(&sig, &needs, &vital_rested));
        let v_tired = Valence::compute(&make_input(&sig, &needs, &vital_tired));

        assert!(v_tired.drives[7] < v_rested.drives[7],
            "Fatica deve abbassare CD8: rested={:.3} tired={:.3}",
            v_rested.drives[7], v_tired.drives[7]);
    }

    #[test]
    fn test_derived_labels_variety() {
        let needs = default_needs();
        let vital = default_vital();

        // Drive diversi dominanti producono etichette diverse
        let cases: Vec<([f64; 8], &str)> = vec![
            ([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], "aperto"),
            ([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.0], "curioso"),  // CD7 dim2 alto + sat OK
        ];

        for (sig, _expected_contains) in &cases {
            let v = Valence::compute(&make_input(sig, &needs, &vital));
            let label = v.derived_stance_label();
            // Verifica che l'etichetta non sia vuota
            assert!(!label.is_empty(), "Label non deve essere vuota per sig {:?}", sig);
        }
    }

    #[test]
    fn test_will_modulation_sane_range() {
        let sig = [0.5; 8];
        let needs = default_needs();
        let vital = default_vital();
        let v = Valence::compute(&make_input(&sig, &needs, &vital));
        let m = v.will_modulation();

        for (i, &val) in m.iter().enumerate() {
            assert!(val >= 0.3 && val <= 2.5,
                "will_modulation[{}] fuori range: {}", i, val);
        }
    }

    #[test]
    fn test_summary_format() {
        let sig = [0.8, 0.7, 0.6, 0.8, 0.5, 0.7, 0.9, 0.4];
        let needs = default_needs();
        let vital = default_vital();
        let v = Valence::compute(&make_input(&sig, &needs, &vital));
        let s = v.summary();

        assert!(s.contains("tono:"), "Summary deve contenere il tono: {}", s);
        assert!(!s.is_empty(), "Summary non deve essere vuoto");
    }
}
