/// inquiry.rs — Rilevamento lacune topologiche interne.
///
/// In precedenza questo modulo chiamava Qwen3 via Ollama per rispondere alle lacune.
/// Quella dipendenza esterna è stata rimossa: UI-r1 non chiede a nessuno fuori di sé.
///
/// Il modulo ora espone solo `extract_gaps()` — usato da engine.rs per convertire
/// le lacune in `SelfUncertainty` nel SelfModel dell'entità. Le lacune irrisolte
/// vengono esposte nella UI come domande aperte che l'utente può scegliere di
/// illuminare, restituendo comprensione all'entità tramite `/api/clarity`.

use crate::topology::thought::{generate_thoughts, ThoughtKind};
use crate::topology::engine::PrometeoTopologyEngine;

// ═══════════════════════════════════════════════════════════════
// Helper per l'integrazione con l'engine
// ═══════════════════════════════════════════════════════════════

/// Raccoglie le lacune topologiche con strength >= threshold dalla topologia corrente.
/// Ritorna `(nome_concetto, strength)` ordinati per forza decrescente.
/// Solo Gap e MissingBridge — i tipi che indicano conoscenza assente, non tensione.
pub fn extract_gaps(engine: &PrometeoTopologyEngine, threshold: f64) -> Vec<(String, f64)> {
    let mut gaps: Vec<(String, f64)> = generate_thoughts(engine)
        .into_iter()
        .filter(|t| {
            matches!(t.kind, ThoughtKind::Gap | ThoughtKind::MissingBridge)
                && t.strength >= threshold
        })
        .filter_map(|t| {
            // Per Gap: usa il nome del frattale.
            // Per MissingBridge: usa le parole coinvolte se disponibili.
            let label = if !t.words.is_empty() {
                t.words.iter().take(2).cloned().collect::<Vec<_>>().join(" — ")
            } else {
                t.fractal_names.into_iter().next().unwrap_or_default()
            };
            if label.is_empty() { None } else { Some((label, t.strength)) }
        })
        .collect();

    gaps.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    gaps.dedup_by(|a, b| a.0 == b.0);
    gaps
}
