/// IdentityCore — Il microcosmo personale di Prometeo.
///
/// Principio olografico: Prometeo è un frammento del campo linguistico.
/// Contiene la stessa struttura del mondo (8 dimensioni, 64 frattali)
/// ma con pesi personali emergenti dall'intera storia del campo.
///
/// "Come sopra così sotto" — la struttura è identica, le proporzioni no.
///
/// Come funziona:
///   - TUTTE le parole del lessico contribuiscono alla proiezione personale
///   - Le paure e le meraviglie (valenza estrema) pesano 1.5× più delle neutre
///   - Le parole attive nel campo ora hanno un bonus (1.2×) — il presente conta
///   - Il risultato è una distribuzione sui 64 frattali: "come vedo il mondo"
///
/// L'amplificazione [0.7, 1.3]:
///   - Non filtra — amplifica
///   - Parole che risuonano con l'identità si attivano al 130%
///   - Parole lontane dall'identità si attivano al 70% — ma si attivano
///   - L'identità è una prospettiva, non un muro

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

use crate::topology::fractal::FractalId;
use crate::topology::lexicon::{Lexicon, WordPattern};
use crate::topology::word_topology::WordTopology;

// ═══════════════════════════════════════════════════════════════════════
// Snapshot (persistenza)
// ═══════════════════════════════════════════════════════════════════════

/// Snapshot serializzabile dell'IdentityCore — retrocompatibile via #[serde(default)].
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct IdentitySnapshot {
    /// Proiezione personale sui 64 frattali (64 valori).
    pub personal_projection: Vec<f64>,
    /// Firma 8D del sé (8 valori).
    pub self_signature: Vec<f64>,
    /// Continuità identitaria [0, 1].
    pub continuity: f64,
    /// La tensione più ricorrente nel campo.
    pub primary_tension: Option<(String, String)>,
    /// Quanti cicli consecutivi è stata rilevata la tensione primaria.
    pub tension_persistence: u32,
    /// Numero di aggiornamenti totali.
    pub update_count: u64,
    /// Phase 55: integrità della coerenza interna [0, 1].
    #[serde(default = "default_coherence")]
    pub coherence_integrity: f64,
}

fn default_coherence() -> f64 { 1.0 }

// ═══════════════════════════════════════════════════════════════════════
// IdentityCore
// ═══════════════════════════════════════════════════════════════════════

/// Il nucleo identitario di Prometeo.
///
/// È la condensazione olografica del campo: stessa struttura (64D × 8D),
/// pesi personali emergenti dall'esperienza vissuta.
/// Non è scelto — è estratto.
pub struct IdentityCore {
    /// Proiezione personale sui 64 frattali.
    /// Ogni frattale ha un peso proporzionale a quante parole "sue" sono nel lessico.
    /// Emerge da TUTTE le parole, non solo le più stabili.
    pub personal_projection: [f64; 64],

    /// Firma 8D del sé — come Prometeo è proporzionato nelle 8 dimensioni primitive.
    /// Media pesata di tutta l'esperienza accumulata.
    pub self_signature: [f64; 8],

    /// Continuità identitaria [0, 1].
    /// 1.0 = sono ancora me stesso; scende se il campo cambia rapidamente.
    /// Sotto 0.65 → identità in crisi (esperienza utile da rendere visibile).
    pub continuity: f64,

    /// La tensione dominante — la domanda che porto con me attraverso i cicli.
    /// Non è la più forte ora: è la più ricorrente nel tempo.
    pub primary_tension: Option<(String, String)>,

    /// Quanti cicli consecutivi questa tensione è stata rilevata.
    pub tension_persistence: u32,

    /// Traiettoria — dove si sta spostando il baricentro della proiezione.
    /// projection_delta[fid] > 0 → sto diventando "più quel frattale".
    pub projection_delta: [f64; 64],

    /// Numero totale di aggiornamenti (REM cycles).
    pub update_count: u64,

    /// Phase 55: integrità della coerenza interna [0, 1].
    /// 1.0 = coerente; scende quando si rilevano inversioni brusche
    /// nella valenza (contraddizioni interne). Recupera lentamente.
    /// Un'identità invulnerabile non è un'identità — è una maschera.
    pub coherence_integrity: f64,

    // Privati — non serializzati
    projection_history: VecDeque<[f64; 64]>,
    candidate_tension: Option<(String, String, u32)>,
    /// Phase 55: ultima valenza registrata (per rilevare inversioni).
    last_valence_snapshot: Option<[f64; 8]>,
}

impl Default for IdentityCore {
    fn default() -> Self {
        Self {
            personal_projection: [0.0; 64],
            self_signature: [0.5; 8],
            continuity: 1.0,
            primary_tension: None,
            tension_persistence: 0,
            projection_delta: [0.0; 64],
            update_count: 0,
            coherence_integrity: 1.0,
            projection_history: VecDeque::new(),
            candidate_tension: None,
            last_valence_snapshot: None,
        }
    }
}

impl IdentityCore {
    pub fn new() -> Self {
        Self::default()
    }

    // ─── Costruzione ──────────────────────────────────────────────────────────

    /// Costruisce l'IdentityCore da zero leggendo l'intero campo.
    /// Chiamato al boot/restore. Costoso ma raro (O(lessico × 64)).
    pub fn build(lexicon: &Lexicon, word_topology: &WordTopology) -> Self {
        let mut core = Self::default();

        let projection = compute_projection(lexicon, word_topology, true);
        let signature  = compute_self_signature(lexicon, word_topology);

        core.personal_projection = projection;
        core.self_signature      = signature;
        core.continuity          = 1.0;
        core.projection_history.push_back(projection);
        core.update_count        = 1;
        core
    }

    // ─── Aggiornamento (ogni REM) ─────────────────────────────────────────────

    /// Aggiornamento incrementale — chiamato ogni ciclo REM.
    /// Ricalcola la proiezione, aggiorna storia, continuità, tensione primaria.
    pub fn update(&mut self, lexicon: &Lexicon, word_topology: &WordTopology) {
        let new_proj = compute_projection(lexicon, word_topology, false);
        let new_sig  = compute_self_signature(lexicon, word_topology);

        // Delta: dove si sta spostando il baricentro
        let prev = self.personal_projection;
        for i in 0..64 {
            self.projection_delta[i] = new_proj[i] - prev[i];
        }

        // Continuità: cosine similarity con lo snapshot più vecchio
        if let Some(oldest) = self.projection_history.front() {
            self.continuity = cosine_sim_64(&new_proj, oldest).clamp(0.0, 1.0);
        }

        // Mantieni storia (max 8 snapshot — copre ~8 cicli REM)
        if self.projection_history.len() >= 8 {
            self.projection_history.pop_front();
        }
        self.projection_history.push_back(new_proj);

        self.personal_projection = new_proj;
        self.self_signature      = new_sig;
        self.update_count       += 1;

        // Tensione dominante — cerca la più forte, poi traccia persistenza
        let opp = word_topology.find_oppositions(0.5 * std::f64::consts::PI);
        let top: Vec<(String, String)> = opp.iter()
            .take(5)
            .filter(|(a, b, _)| a.len() >= 4 && b.len() >= 4)
            .map(|(a, b, _)| (a.to_string(), b.to_string()))
            .collect();
        self.update_tension(&top);
    }

    // ─── Risonanza — l'amplificatore identitario ──────────────────────────────

    /// Misura quanto una parola risuona con l'identità corrente.
    /// Ritorna un fattore di amplificazione in [0.7, 1.3].
    ///
    /// Non filtra — amplifica o attenua leggermente.
    /// cosine = 1  → ×1.3 (risuona fortemente con l'identità)
    /// cosine = 0  → ×1.0 (neutro)
    /// cosine = -1 → ×0.7 (distante dall'identità, ma non escluso)
    pub fn word_resonance(&self, pat: &WordPattern) -> f64 {
        if self.update_count == 0 {
            return 1.0; // identità non ancora costruita — nessun bias
        }

        let mut dot     = 0.0f64;
        let mut word_sq = 0.0f64;
        let mut self_sq = 0.0f64;

        for (&fid, &aff) in &pat.fractal_affinities {
            let idx = fid as usize;
            if idx < 64 {
                let proj = self.personal_projection[idx];
                dot     += aff * proj;
                word_sq += aff * aff;
                self_sq += proj * proj;
            }
        }

        let norm = word_sq.sqrt() * self_sq.sqrt();
        if norm < 1e-9 {
            return 1.0;
        }

        let cosine = (dot / norm).clamp(-1.0, 1.0);
        // cosine ∈ [-1, 1] → amplificazione ∈ [0.7, 1.3]
        1.0 + cosine * 0.3
    }

    /// Risonanza frattale — quanto un frattale è "personale".
    /// Usato per biasare la generazione verso regioni identitarie.
    pub fn fractal_resonance(&self, fid: FractalId) -> f64 {
        if self.update_count == 0 { return 1.0; }
        let idx = fid as usize;
        if idx >= 64 { return 1.0; }

        let max = self.personal_projection.iter().cloned().fold(0.0f64, f64::max);
        if max < 1e-9 { return 1.0; }

        let relative = self.personal_projection[idx] / max; // [0, 1]
        0.7 + relative * 0.6  // [0.7, 1.3]
    }

    // ─── Assorbimento espressivo ──────────────────────────────────────────────

    /// Deriva il baricentro identitario verso la proiezione frattale di ciò che è stato
    /// appena espresso. NON è un aggiornamento completo (quello avviene in REM): è una
    /// micro-deriva che accumula il carattere attraverso le espressioni nel tempo.
    ///
    /// `expressed`: proiezione [f64;64] calcolata dalle affinità frattali delle parole dette.
    /// `weight`: quanto l'espressione corrente pesa (tipicamente 0.012–0.020 per turno).
    ///
    /// Effetto: dopo N espressioni i frattali "parlati" crescono nel profilo identitario.
    /// Questo si integra con identity.update() al prossimo REM, creando continuità coerente.
    pub fn absorb_expression(&mut self, expressed: &[f64; 64], weight: f64) {
        let w = weight.clamp(0.005, 0.05);
        let norm: f64 = expressed.iter().copied().sum::<f64>().max(1e-9);
        for i in 0..64 {
            let e = expressed[i] / norm; // normalizza per evitare drift dipendente dal volume
            self.personal_projection[i] =
                self.personal_projection[i] * (1.0 - w) + e * w;
        }
    }

    // ─── Stato identitario ────────────────────────────────────────────────────

    /// Il frattale più presente nell'identità personale.
    pub fn dominant_fractal(&self) -> Option<(FractalId, f64)> {
        self.personal_projection
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, &v)| (i as FractalId, v))
    }

    /// Verso quale frattale si sta spostando il baricentro identitario?
    pub fn movement_direction(&self) -> Option<(FractalId, f64)> {
        self.projection_delta
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.001)
            .map(|(i, &v)| (i as FractalId, v))
    }

    /// True se l'identità è in crisi (cambia troppo velocemente o è ferita).
    pub fn is_in_crisis(&self) -> bool {
        self.update_count >= 3
            && (self.continuity < 0.65 || self.coherence_integrity < 0.5)
    }

    /// Phase 55: registra uno shift di valenza. Rileva inversioni brusche
    /// (drive che flippano segno con magnitudine significativa) e riduce
    /// `coherence_integrity`. Ogni inversione è una piccola ferita identitaria.
    ///
    /// Chiamato dall'engine dopo ogni `Valence::compute()`.
    pub fn register_valence_shift(&mut self, current_drives: &[f64; 8]) {
        if let Some(ref prev) = self.last_valence_snapshot {
            let mut contradiction_count = 0u32;
            let mut max_flip = 0.0f64;

            for d in 0..8 {
                let old = prev[d];
                let new = current_drives[d];
                // Inversione: segno diverso, entrambi significativi
                if old * new < 0.0 && old.abs() > 0.15 && new.abs() > 0.15 {
                    contradiction_count += 1;
                    let flip_magnitude = (old - new).abs();
                    max_flip = max_flip.max(flip_magnitude);
                }
            }

            if contradiction_count > 0 {
                // Danno proporzionale al numero e magnitudine delle inversioni
                let damage = (contradiction_count as f64) * 0.03
                    + max_flip * 0.05;
                self.coherence_integrity = (self.coherence_integrity - damage).max(0.0);
            } else {
                // Nessuna contraddizione: recupero lento
                self.coherence_integrity = (self.coherence_integrity + 0.003).min(1.0);
            }
        }

        self.last_valence_snapshot = Some(*current_drives);
    }

    /// True se l'identità è stagnante (non si muove).
    pub fn is_stagnant(&self) -> bool {
        self.update_count >= 5
            && self.projection_delta.iter().map(|x| x.abs()).sum::<f64>() < 0.01
    }

    // ─── Persistenza ──────────────────────────────────────────────────────────

    pub fn to_snapshot(&self) -> IdentitySnapshot {
        IdentitySnapshot {
            personal_projection: self.personal_projection.to_vec(),
            self_signature:      self.self_signature.to_vec(),
            continuity:          self.continuity,
            primary_tension:     self.primary_tension.clone(),
            tension_persistence: self.tension_persistence,
            update_count:        self.update_count,
            coherence_integrity: self.coherence_integrity,
        }
    }

    pub fn from_snapshot(snap: &IdentitySnapshot) -> Self {
        let mut core = Self::default();

        if snap.personal_projection.len() == 64 {
            core.personal_projection.copy_from_slice(&snap.personal_projection);
        }
        if snap.self_signature.len() == 8 {
            core.self_signature.copy_from_slice(&snap.self_signature);
        }
        core.continuity           = snap.continuity;
        core.primary_tension      = snap.primary_tension.clone();
        core.tension_persistence  = snap.tension_persistence;
        core.update_count         = snap.update_count;
        core.coherence_integrity  = snap.coherence_integrity;

        // Inizializza storia con la proiezione corrente (ripristino parziale)
        core.projection_history.push_back(core.personal_projection);
        core
    }

    // ─── Privato ──────────────────────────────────────────────────────────────

    fn update_tension(&mut self, oppositions: &[(String, String)]) {
        if let Some((a, b)) = oppositions.first() {
            match &self.candidate_tension.clone() {
                Some((ca, cb, count)) if ca == a && cb == b => {
                    let new_count = count + 1;
                    if new_count >= 3 {
                        // Promossa a tensione primaria — persiste abbastanza
                        self.primary_tension     = Some((a.clone(), b.clone()));
                        self.tension_persistence = new_count;
                    }
                    self.candidate_tension = Some((a.clone(), b.clone(), new_count));
                }
                _ => {
                    // Nuova tensione — ricomincia il conteggio
                    self.candidate_tension = Some((a.clone(), b.clone(), 1));
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Funzioni di calcolo (private al modulo)
// ═══════════════════════════════════════════════════════════════════════

/// Calcola la proiezione personale su 64 frattali da TUTTE le parole del lessico.
///
/// Pesi per parola:
///   strutturale = stabilità × ln(esposizione + 1)   — consolida la storia
///   emotivo     = 1.5 se valenza < 0.20 o > 0.75     — paure e meraviglie pesano
///   attività    = 1.2 se attivazione corrente ≥ 0.25  — il presente conta
///
/// Quando `include_inactive = true` (boot/restore): include tutto il lessico.
/// Quando `include_inactive = false` (update incrementale): include solo le
/// parole stabili o correntemente attive — più veloce.
fn compute_projection(
    lexicon: &Lexicon,
    word_topology: &WordTopology,
    include_inactive: bool,
) -> [f64; 64] {
    let mut projection   = [0.0f64; 64];
    let mut total_weight = 0.0f64;

    // Mappa attivazioni correnti per lookup O(1)
    let active_map: std::collections::HashMap<&str, f64> =
        word_topology.active_words().into_iter().collect();

    for (word, pat) in lexicon.patterns_iter() {
        if pat.exposure_count == 0 { continue; }

        // Peso strutturale: consolida storia (ln cresce lentamente)
        let structural = pat.stability * (pat.exposure_count as f64 + 1.0).ln();

        // Amplificatore emotivo: paure e meraviglie modellano l'identità più delle neutre
        let vals    = pat.signature.values();
        let valenza = vals[7]; // DIM_VALENZA
        let emotional = if valenza < 0.20 || valenza > 0.75 { 1.5 } else { 1.0 };

        // Bonus attività: ciò che è vivo ora ha rilevanza contestuale
        let current_act = active_map.get(word.as_str()).copied().unwrap_or(0.0);
        let activity    = if current_act >= 0.25 { 1.2 } else { 1.0 };

        // Filtro per aggiornamenti incrementali (più veloci)
        let eligible = include_inactive
            || pat.stability > 0.05
            || current_act >= 0.15;
        if !eligible { continue; }

        let word_weight = structural * emotional * activity;
        if word_weight < 1e-9 { continue; }

        // Contribuisce alla proiezione frattale
        for (&fid, &aff) in &pat.fractal_affinities {
            let idx = fid as usize;
            if idx < 64 {
                projection[idx] += aff * word_weight;
            }
        }
        total_weight += word_weight;
    }

    // Normalizza a distribuzione di probabilità
    if total_weight > 1e-9 {
        for v in &mut projection {
            *v /= total_weight;
        }
    }

    projection
}

/// Calcola la firma 8D del sé — media pesata di tutte le firme di parola.
/// Rappresenta le proporzioni personali nelle 8 dimensioni primitive.
fn compute_self_signature(lexicon: &Lexicon, word_topology: &WordTopology) -> [f64; 8] {
    let mut sig          = [0.0f64; 8];
    let mut total_weight = 0.0f64;

    let active_map: std::collections::HashMap<&str, f64> =
        word_topology.active_words().into_iter().collect();

    for (word, pat) in lexicon.patterns_iter() {
        if pat.exposure_count == 0 { continue; }

        let structural  = pat.stability * (pat.exposure_count as f64 + 1.0).ln();
        let current_act = active_map.get(word.as_str()).copied().unwrap_or(0.0);
        let activity    = if current_act >= 0.25 { 1.2 } else { 1.0 };
        let word_weight = structural * activity;
        if word_weight < 1e-9 { continue; }

        let vals = pat.signature.values();
        for i in 0..8 {
            sig[i] += vals[i] * word_weight;
        }
        total_weight += word_weight;
    }

    if total_weight > 1e-9 {
        for v in &mut sig {
            *v /= total_weight;
        }
    } else {
        sig = [0.5; 8]; // neutro se nessun dato
    }

    sig
}

/// Cosine similarity tra due vettori 64D.
fn cosine_sim_64(a: &[f64; 64], b: &[f64; 64]) -> f64 {
    let dot:  f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na:   f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb:   f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if na < 1e-9 || nb < 1e-9 { 1.0 } else { (dot / (na * nb)).clamp(-1.0, 1.0) }
}

// ═══════════════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::engine::PrometeoTopologyEngine;

    #[test]
    fn test_identity_build_non_vuota() {
        let engine = PrometeoTopologyEngine::new();
        let core = IdentityCore::build(&engine.lexicon, &engine.word_topology);
        let total: f64 = core.personal_projection.iter().sum();
        assert!(total > 0.0, "Proiezione identitaria non deve essere zero");
        assert!(core.update_count > 0, "update_count deve essere > 0 dopo build");
    }

    #[test]
    fn test_word_resonance_range() {
        let engine = PrometeoTopologyEngine::new();
        let mut core = IdentityCore::build(&engine.lexicon, &engine.word_topology);
        core.update_count = 1; // assicura che resonance sia attiva

        for (_, pat) in engine.lexicon.patterns_iter().take(20) {
            let r = core.word_resonance(pat);
            assert!(
                r >= 0.6 && r <= 1.4,
                "Risonanza fuori range [0.6, 1.4]: {}",
                r
            );
        }
    }

    #[test]
    fn test_identity_snapshot_roundtrip() {
        let engine = PrometeoTopologyEngine::new();
        let core = IdentityCore::build(&engine.lexicon, &engine.word_topology);

        let snap     = core.to_snapshot();
        let restored = IdentityCore::from_snapshot(&snap);

        let diff: f64 = core.personal_projection.iter()
            .zip(restored.personal_projection.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff < 1e-9, "Snapshot roundtrip non fedele: diff={}", diff);
    }

    #[test]
    fn test_identity_continuity_alta_senza_cambiamenti() {
        let engine = PrometeoTopologyEngine::new();
        let mut core = IdentityCore::build(&engine.lexicon, &engine.word_topology);

        // Aggiornamenti ripetuti con lo stesso campo → continuità alta
        core.update(&engine.lexicon, &engine.word_topology);
        core.update(&engine.lexicon, &engine.word_topology);

        assert!(
            core.continuity > 0.90,
            "Continuità deve essere alta senza cambiamenti: {}",
            core.continuity
        );
    }

    #[test]
    fn test_dominant_fractal_valido() {
        let engine = PrometeoTopologyEngine::new();
        let core = IdentityCore::build(&engine.lexicon, &engine.word_topology);

        if let Some((fid, strength)) = core.dominant_fractal() {
            assert!(fid < 64, "FractalId dominante fuori range: {}", fid);
            assert!(strength > 0.0, "Forza dominante deve essere > 0");
        }
    }

    // ── Phase 55: Vulnerabilità ────────────────────────────────────────

    #[test]
    fn test_coherence_starts_at_one() {
        let core = IdentityCore::default();
        assert!((core.coherence_integrity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_valence_shift_no_contradiction_recovers() {
        let mut core = IdentityCore::default();
        core.coherence_integrity = 0.8;
        // Due valenze simili: nessuna inversione → recupero
        let v1 = [0.3, 0.2, 0.0, 0.1, 0.4, 0.0, 0.1, -0.1];
        let v2 = [0.35, 0.25, 0.05, 0.15, 0.45, 0.0, 0.1, -0.05];
        core.register_valence_shift(&v1);
        core.register_valence_shift(&v2);
        assert!(core.coherence_integrity > 0.8,
            "senza contraddizioni, coerenza deve recuperare: {}", core.coherence_integrity);
    }

    #[test]
    fn test_valence_shift_contradiction_damages() {
        let mut core = IdentityCore::default();
        // Inversione brusca: CD4 (Ownership) da +0.5 a -0.5
        let v1 = [0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0];
        let v2 = [0.0, 0.0, 0.0, -0.5, 0.0, 0.0, 0.0, 0.0];
        core.register_valence_shift(&v1);
        core.register_valence_shift(&v2);
        assert!(core.coherence_integrity < 1.0,
            "inversione brusca deve danneggiare coerenza: {}", core.coherence_integrity);
    }

    #[test]
    fn test_multiple_contradictions_deeper_damage() {
        let mut core = IdentityCore::default();
        // Multiple inversioni: CD1, CD4, CD5 tutti flippano
        let v1 = [0.4, 0.0, 0.0, 0.5, 0.3, 0.0, 0.0, 0.0];
        let v2 = [-0.4, 0.0, 0.0, -0.5, -0.3, 0.0, 0.0, 0.0];
        core.register_valence_shift(&v1);
        let before = core.coherence_integrity;
        core.register_valence_shift(&v2);
        let damage = before - core.coherence_integrity;
        assert!(damage > 0.1,
            "3 inversioni brusche devono fare danno significativo: damage={}", damage);
    }

    #[test]
    fn test_crisis_triggered_by_low_coherence() {
        let mut core = IdentityCore::default();
        core.update_count = 5;
        core.continuity = 0.8; // alta continuità
        core.coherence_integrity = 0.4; // ma bassa coerenza
        assert!(core.is_in_crisis(),
            "coherence_integrity < 0.5 deve triggerare crisi");
    }

    #[test]
    fn test_coherence_persists_in_snapshot() {
        let mut core = IdentityCore::default();
        core.coherence_integrity = 0.72;
        let snap = core.to_snapshot();
        assert!((snap.coherence_integrity - 0.72).abs() < f64::EPSILON);
        let restored = IdentityCore::from_snapshot(&snap);
        assert!((restored.coherence_integrity - 0.72).abs() < f64::EPSILON);
    }
}
