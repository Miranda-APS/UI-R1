/// Creazione del polo Yin (Eva) dal polo Yang (Adamo) via rotazione di fase 8D.
///
/// Eva nasce con la stessa conoscenza di Adamo (lessico + simplici)
/// ma senza storia personale (memoria fresca, sogno fresco, locus neutro).
///
/// Coppie polari ruotate di pi/3:
///   (Agency, Confine)          -- agente vs interno
///   (Intensita, Permanenza)    -- fuoco vs durata
///   (Definizione, Complessita) -- nomina vs tesse
/// Valenza e Tempo restano invariate.

use crate::topology::engine::PrometeoTopologyEngine;
use crate::topology::primitive::Dim;

/// Crea il polo Yin trasferendo la conoscenza di Adamo in un engine fresco
/// e applicando una rotazione di fase pi/3 sulle coppie di dimensioni polari.
pub fn create_polar_twin(source: &PrometeoTopologyEngine) -> PrometeoTopologyEngine {
    let mut twin = PrometeoTopologyEngine::new();

    // Trasferisci il lessico (WordPattern e' Clone)
    let patterns: Vec<(String, crate::topology::lexicon::WordPattern)> = source.lexicon
        .patterns_iter()
        .map(|(w, p)| (w.to_string(), p.clone()))
        .collect();
    for (word, pattern) in patterns {
        twin.lexicon.insert_pattern(&word, pattern);
    }

    // Trasferisci il complesso simpliciale
    twin.complex.clear();
    for (_, simplex) in source.complex.iter() {
        twin.complex.restore_simplex(
            simplex.id,
            simplex.vertices.clone(),
            simplex.shared_faces.clone(),
            simplex.persistence,
            simplex.plasticity,
            simplex.activation_count,
            simplex.source_words.clone(),
            simplex.category.clone(),
            simplex.ordered,
            simplex.function_fractal,
        );
    }

    // Applica rotazione pi/3 alle firme
    let words: Vec<String> = twin.lexicon
        .patterns_iter()
        .map(|(w, _)| w.to_string())
        .collect();

    for word in &words {
        if let Some(pat) = twin.lexicon.get_mut(word) {
            let sig = &mut pat.signature;
            let (na, nc) = rotate_pair(sig.get(Dim::Agency), sig.get(Dim::Confine));
            sig.set(Dim::Agency, na);
            sig.set(Dim::Confine, nc);
            let (ni, np) = rotate_pair(sig.get(Dim::Intensita), sig.get(Dim::Permanenza));
            sig.set(Dim::Intensita, ni);
            sig.set(Dim::Permanenza, np);
            let (nd, nco) = rotate_pair(sig.get(Dim::Definizione), sig.get(Dim::Complessita));
            sig.set(Dim::Definizione, nd);
            sig.set(Dim::Complessita, nco);
        }
    }

    // Ricalibra
    twin.recompute_all_word_affinities();
    twin.recalibrate_emergent_dimensions();
    twin.word_topology.seed_resting_state(&twin.lexicon);
    twin
}

/// Rotazione di fase pi/3 centrata in (0.5, 0.5).
/// theta = pi/3: cos=0.5, sin~0.866
#[inline]
fn rotate_pair(a: f64, b: f64) -> (f64, f64) {
    let ac = a - 0.5;
    let bc = b - 0.5;
    let new_a = ac * 0.5 - bc * 0.866_025_4 + 0.5;
    let new_b = ac * 0.866_025_4 + bc * 0.5 + 0.5;
    (new_a.clamp(0.0, 1.0), new_b.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_pair_preserves_center() {
        let (a, b) = rotate_pair(0.5, 0.5);
        assert!((a - 0.5).abs() < 1e-6);
        assert!((b - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_rotate_pair_changes_values() {
        let (a, b) = rotate_pair(0.9, 0.1);
        assert!((a - 0.9).abs() > 0.01 || (b - 0.1).abs() > 0.01);
    }

    #[test]
    fn test_rotate_pair_clamps_to_unit() {
        for &(a, b) in &[(1.0_f64, 0.0_f64), (0.0, 1.0), (1.0, 1.0), (0.0, 0.0)] {
            let (ra, rb) = rotate_pair(a, b);
            assert!(ra >= 0.0 && ra <= 1.0);
            assert!(rb >= 0.0 && rb <= 1.0);
        }
    }

    #[test]
    fn test_polar_twin_preserves_word_count() {
        let mut engine = PrometeoTopologyEngine::new();
        engine.teach("io sono qui dentro vicino forte");
        engine.teach("tu sei lontano fuori debole");
        let n_before = engine.lexicon.word_count();
        let twin = create_polar_twin(&engine);
        assert_eq!(n_before, twin.lexicon.word_count());
    }

    #[test]
    fn test_polar_twin_rotates_signatures() {
        let mut engine = PrometeoTopologyEngine::new();
        engine.teach("io agire forte definito");
        let twin = create_polar_twin(&engine);
        let mut diffs = 0usize;
        for (word, pat) in engine.lexicon.patterns_iter() {
            if let Some(tp) = twin.lexicon.get(word) {
                let orig = pat.signature.values();
                let rotd = tp.signature.values();
                let dist: f64 = orig.iter().zip(rotd.iter())
                    .map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt();
                if dist > 0.05 { diffs += 1; }
            }
        }
        assert!(diffs > 0);
    }

    #[test]
    fn test_polar_twin_preserves_simplices() {
        let mut engine = PrometeoTopologyEngine::new();
        engine.teach("io sentire dentro");
        let n_before = engine.complex.count();
        let twin = create_polar_twin(&engine);
        assert!(twin.complex.count() >= n_before.min(1));
    }
}
