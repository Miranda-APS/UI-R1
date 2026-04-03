/// thought.rs — Osservazione topologica interna.
/// Nessun template, nessun linguaggio hardcodato.
/// Un pensiero è una struttura computata dal campo.

use crate::topology::{
    engine::PrometeoTopologyEngine,
    fractal::FractalId,
};

// ═══════════════════════════════════════════════════════════════
// Tipi
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum ThoughtKind {
    /// Due parole attive si oppongono nel campo — tensione irrisolta
    Tension,
    /// Frattale bootstrap con pochissimi simplessi — zona cieca
    Gap,
    /// Due frattali molto usati ma raramente connessi — ponte mancante
    MissingBridge,
    /// Due o più mondi disconnessi nel complesso
    Disconnection,
    /// Simplice recente in STM, non ancora in LTM — ipotesi in verifica
    Hypothesis,
    /// Phase 50: abduzione autonoma — "quale frattale spiega lo stato corrente?"
    AbductiveHypothesis,
    /// Phase 53: l'entità si è sorpresa dal proprio output
    SelfDiscovery,
    /// Phase 53: bisogno in crisi (satisfaction < 0.35)
    Need,
    /// Phase 53: desiderio attivo con intensità significativa
    Desire,
    /// Phase 53: eco dell'Altro rilevato nel campo
    Interlocutor,
    /// Phase 53: configurazione umoristica (ironia, bisociazione)
    Humor,
}

#[derive(Debug, Clone)]
pub struct Thought {
    pub kind: ThoughtKind,
    /// Frattali coinvolti (nomi)
    pub fractal_names: Vec<String>,
    /// Parole chiave coinvolte (se disponibili)
    pub words: Vec<String>,
    /// Salienza [0.0, 1.0]
    pub strength: f64,
    /// Dati raw computati
    pub data: ThoughtData,
}

#[derive(Debug, Clone)]
pub enum ThoughtData {
    TensionData     { phase: f64, word_a: String, word_b: String },
    GapData         { simplex_count: usize, word_count: usize, activation_count: u64 },
    MissingBridgeData { proximity: f64, shared_simplices: usize },
    DisconnectionData { components: Vec<Vec<String>> },
    HypothesisData  { simplex_id: u32, dimension: usize, activation_count: u64 },
    /// Phase 53: dati della scoperta dal self-listening
    SelfDiscoveryData { divergence: f64, emergent_fractals: Vec<String>, trigger_words: Vec<String> },
    /// Phase 53: bisogno in crisi
    NeedData { level: String, satisfaction: f64 },
    /// Phase 53: desiderio attivo
    DesireData { name: String, intensity: f64, distance: f64 },
    /// Phase 53: eco dell'Altro
    InterlocutorData { presence: f64, pattern: String, resonance: f64 },
    /// Phase 53: umorismo topologico
    HumorData { incongruity: f64, irony_pairs: Vec<(String, String)>, bisociation: Option<(u32, u32)> },
}

// ═══════════════════════════════════════════════════════════════
// Funzione principale
// ═══════════════════════════════════════════════════════════════

pub fn generate_thoughts(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let mut thoughts = Vec::new();

    thoughts.extend(detect_tensions(engine));
    thoughts.extend(detect_gaps(engine));
    thoughts.extend(detect_missing_bridges(engine));
    thoughts.extend(detect_disconnections(engine));
    thoughts.extend(detect_hypotheses(engine));

    // Phase 53: pensieri dai nuovi sistemi
    thoughts.extend(engine.pending_self_discoveries.iter().cloned());
    thoughts.extend(detect_needs(engine));
    thoughts.extend(detect_desires(engine));
    thoughts.extend(detect_interlocutor(engine));
    thoughts.extend(detect_humor(engine));

    thoughts.sort_by(|a, b| b.strength.partial_cmp(&a.strength)
        .unwrap_or(std::cmp::Ordering::Equal));
    thoughts.truncate(30);
    thoughts
}

// ═══════════════════════════════════════════════════════════════
// Componenti connesse — union-find sui frattali
// ═══════════════════════════════════════════════════════════════

fn fractal_components(engine: &PrometeoTopologyEngine) -> Vec<Vec<FractalId>> {
    // Union-Find iterativo — gestisce ID mancanti senza panic
    let mut parent: std::collections::HashMap<FractalId, FractalId> = std::collections::HashMap::new();

    // Inizializza con tutti gli ID del registro
    for id in engine.registry.all_ids() {
        parent.insert(id, id);
    }

    let find_root = |parent: &std::collections::HashMap<FractalId, FractalId>, mut x: FractalId| -> FractalId {
        // Iterativo, path compression non necessaria qui
        loop {
            match parent.get(&x) {
                None => return x,            // ID sconosciuto — radice di sé stesso
                Some(&p) if p == x => return x,
                Some(&p) => x = p,
            }
        }
    };

    // Unisci frattali connessi via simplessi
    for (_, simp) in engine.complex.iter() {
        let v = &simp.vertices;
        if v.len() < 2 { continue; }
        // Assicura che tutti i vertici siano nel parent
        for &vid in v {
            parent.entry(vid).or_insert(vid);
        }
        let root_a = find_root(&parent, v[0]);
        for i in 1..v.len() {
            let root_b = find_root(&parent, v[i]);
            if root_a != root_b {
                parent.insert(root_b, root_a);
            }
        }
    }

    // Raggruppa per radice (solo frattali registrati)
    let all_ids = engine.registry.all_ids();
    let mut groups: std::collections::HashMap<FractalId, Vec<FractalId>> = std::collections::HashMap::new();
    for id in all_ids {
        let root = find_root(&parent, id);
        groups.entry(root).or_default().push(id);
    }

    let mut result: Vec<Vec<FractalId>> = groups.into_values().collect();
    result.sort_by(|a, b| b.len().cmp(&a.len()));
    result
}

// ═══════════════════════════════════════════════════════════════
// 1. TENSIONI
// ═══════════════════════════════════════════════════════════════

fn detect_tensions(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let mut out = Vec::new();
    let min_phase = std::f64::consts::PI * 0.60;

    let active: std::collections::HashSet<String> = engine.word_topology
        .active_words()
        .into_iter()
        .filter(|(_, a)| *a > 0.08)
        .map(|(w, _)| w.to_string())
        .collect();

    if active.is_empty() { return out; }

    for (wa, wb, phase) in engine.word_topology.find_oppositions(min_phase).iter().take(15) {
        if !active.contains(*wa) && !active.contains(*wb) { continue; }

        let fa = engine.lexicon.get(wa).and_then(|p| p.dominant_fractal()).map(|(id, _)| id);
        let fb = engine.lexicon.get(wb).and_then(|p| p.dominant_fractal()).map(|(id, _)| id);

        let mut names = Vec::new();
        for fid in [fa, fb].iter().flatten() {
            if let Some(f) = engine.registry.get(*fid) {
                if !names.contains(&f.name) { names.push(f.name.clone()); }
            }
        }

        let strength = (phase - min_phase) / (std::f64::consts::PI - min_phase);

        out.push(Thought {
            kind: ThoughtKind::Tension,
            fractal_names: names,
            words: vec![wa.to_string(), wb.to_string()],
            strength,
            data: ThoughtData::TensionData {
                phase: *phase,
                word_a: wa.to_string(),
                word_b: wb.to_string(),
            },
        });
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// 2. LACUNE — frattali bootstrap senza parole o simplessi
// ═══════════════════════════════════════════════════════════════

fn detect_gaps(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let mut out = Vec::new();

    for id in 0u32..16 {
        let simp_count = engine.complex.simplices_of(id).len();
        if simp_count >= 5 { continue; }

        let fractal = match engine.registry.get(id) { Some(f) => f, None => continue };

        let word_count = engine.lexicon.patterns_iter()
            .filter(|(_, p)| p.dominant_fractal().map(|(fid, _)| fid) == Some(id))
            .count();

        let gap_score    = 1.0 - (simp_count as f64 / 5.0).min(1.0);
        let word_penalty = (word_count as f64 / 50.0).min(1.0);
        let strength     = (gap_score * 0.7 + (1.0 - word_penalty) * 0.3).min(1.0);

        out.push(Thought {
            kind: ThoughtKind::Gap,
            fractal_names: vec![fractal.name.clone()],
            words: vec![],
            strength,
            data: ThoughtData::GapData {
                simplex_count: simp_count,
                word_count,
                activation_count: fractal.activation_count,
            },
        });
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// 3. PONTI MANCANTI
// ═══════════════════════════════════════════════════════════════

fn detect_missing_bridges(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let mut out = Vec::new();

    let mut active_fractals: Vec<(FractalId, u64)> = (0u32..16)
        .filter_map(|id| engine.registry.get(id).map(|f| (id, f.activation_count)))
        .filter(|(_, cnt)| *cnt > 5)
        .collect();

    active_fractals.sort_by(|a, b| b.1.cmp(&a.1));
    active_fractals.truncate(8);

    for i in 0..active_fractals.len() {
        for j in (i+1)..active_fractals.len() {
            let (id_a, _) = active_fractals[i];
            let (id_b, _) = active_fractals[j];

            let shared    = engine.complex.shared_simplices(id_a, id_b).len();
            let proximity = engine.complex.topological_proximity(id_a, id_b);

            if proximity < 0.20 && shared < 2 {
                let name_a = engine.registry.get(id_a).map(|f| f.name.clone()).unwrap_or_default();
                let name_b = engine.registry.get(id_b).map(|f| f.name.clone()).unwrap_or_default();
                let strength = (0.20 - proximity) / 0.20;

                out.push(Thought {
                    kind: ThoughtKind::MissingBridge,
                    fractal_names: vec![name_a, name_b],
                    words: vec![],
                    strength,
                    data: ThoughtData::MissingBridgeData { proximity, shared_simplices: shared },
                });
            }
        }
    }

    out.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));
    out.truncate(6);
    out
}

// ═══════════════════════════════════════════════════════════════
// 4. DISCONNESSIONI — con dettaglio per componente
// ═══════════════════════════════════════════════════════════════

fn detect_disconnections(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let comps = fractal_components(engine);

    // Filtra: solo componenti con almeno un frattale bootstrap (id < 16)
    let bootstrap_comps: Vec<Vec<FractalId>> = comps.iter()
        .filter(|c| c.iter().any(|&id| id < 16))
        .cloned()
        .collect();

    if bootstrap_comps.len() <= 1 { return vec![]; }

    // Converti in nomi (solo frattali bootstrap per leggibilità)
    let named: Vec<Vec<String>> = bootstrap_comps.iter().map(|comp| {
        let mut names: Vec<String> = comp.iter()
            .filter(|&&id| id < 16)
            .filter_map(|&id| engine.registry.get(id))
            .map(|f| f.name.clone())
            .collect();
        names.sort();
        names
    }).collect();

    vec![Thought {
        kind: ThoughtKind::Disconnection,
        fractal_names: vec![],
        words: vec![],
        strength: ((bootstrap_comps.len() - 1) as f64 / 3.0).min(1.0),
        data: ThoughtData::DisconnectionData { components: named },
    }]
}

// ═══════════════════════════════════════════════════════════════
// 5. IPOTESI — simplici in STM non ancora in LTM
// ═══════════════════════════════════════════════════════════════

fn detect_hypotheses(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let mut out = Vec::new();

    let stm_ids: std::collections::HashSet<u32> = engine.memory.short_term.iter()
        .flat_map(|imp| imp.active_simplices.iter().map(|(id, _)| *id))
        .collect();

    let ltm_ids: std::collections::HashSet<u32> = engine.memory.long_term.iter()
        .flat_map(|imp| imp.active_simplices.iter().map(|(id, _)| *id))
        .collect();

    for sid in stm_ids.difference(&ltm_ids).take(5) {
        if let Some(simp) = engine.complex.get(*sid) {
            let names: Vec<String> = simp.vertices.iter()
                .filter_map(|&fid| engine.registry.get(fid))
                .map(|f| f.name.clone())
                .collect();

            let strength = (simp.dimension as f64 / 3.0).min(1.0) * 0.5
                + simp.current_activation * 0.5;

            out.push(Thought {
                kind: ThoughtKind::Hypothesis,
                fractal_names: names,
                words: vec![],
                strength,
                data: ThoughtData::HypothesisData {
                    simplex_id: *sid,
                    dimension: simp.dimension,
                    activation_count: simp.activation_count,
                },
            });
        }
    }
    out
}

// ═══════════════════════════════════════════════════════════════
// 6. BISOGNI — livelli in crisi
// ═══════════════════════════════════════════════════════════════

fn detect_needs(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let state = match &engine.last_needs_state {
        Some(s) => s,
        None => return vec![],
    };
    engine.needs.crisis_thoughts(state).into_iter().map(|(level, sat)| {
        Thought {
            kind: ThoughtKind::Need,
            fractal_names: vec![],
            words: vec![level.name().to_string()],
            strength: 1.0 - sat,
            data: ThoughtData::NeedData {
                level: level.name().to_string(),
                satisfaction: sat,
            },
        }
    }).collect()
}

// ═══════════════════════════════════════════════════════════════
// 7. DESIDERI — desideri attivi con intensità significativa
// ═══════════════════════════════════════════════════════════════

fn detect_desires(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    engine.desire.desires.iter().filter(|d| d.intensity > 0.3).map(|d| {
        Thought {
            kind: ThoughtKind::Desire,
            fractal_names: vec![],
            words: vec![d.name.clone()],
            strength: d.intensity,
            data: ThoughtData::DesireData {
                name: d.name.clone(),
                intensity: d.intensity,
                distance: 0.0, // calcolato lazy se necessario
            },
        }
    }).collect()
}

// ═══════════════════════════════════════════════════════════════
// 8. INTERLOCUTORE — eco dell'Altro nel campo
// ═══════════════════════════════════════════════════════════════

fn detect_interlocutor(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let m = &engine.interlocutor;
    if m.presence < 0.3 { return vec![]; }
    let pattern_str = match &m.detected_pattern {
        crate::topology::interlocutor::InteractionPattern::None => "nessuno",
        crate::topology::interlocutor::InteractionPattern::Converging => "convergenza",
        crate::topology::interlocutor::InteractionPattern::Diverging => "divergenza",
        crate::topology::interlocutor::InteractionPattern::Oscillating => "oscillazione",
    };
    vec![Thought {
        kind: ThoughtKind::Interlocutor,
        fractal_names: vec![],
        words: vec![],
        strength: m.presence,
        data: ThoughtData::InterlocutorData {
            presence: m.presence,
            pattern: pattern_str.to_string(),
            resonance: m.cumulative_resonance,
        },
    }]
}

// ═══════════════════════════════════════════════════════════════
// 9. UMORISMO — configurazione sorprendente nel campo
// ═══════════════════════════════════════════════════════════════

fn detect_humor(engine: &PrometeoTopologyEngine) -> Vec<Thought> {
    let h = &engine.last_humor_state;
    if !h.is_active() { return vec![]; }

    let mut names = Vec::new();
    if let Some((fa, fb)) = h.bisociation_pair {
        if let Some(f) = engine.registry.get(fa) { names.push(f.name.clone()); }
        if let Some(f) = engine.registry.get(fb) { names.push(f.name.clone()); }
    }

    let words: Vec<String> = h.irony_pairs.iter()
        .flat_map(|(a, b, _)| vec![a.clone(), b.clone()])
        .chain(h.crossroad_words.iter().cloned())
        .collect();

    vec![Thought {
        kind: ThoughtKind::Humor,
        fractal_names: names,
        words,
        strength: h.incongruity_score,
        data: ThoughtData::HumorData {
            incongruity: h.incongruity_score,
            irony_pairs: h.irony_pairs.iter().map(|(a, b, _)| (a.clone(), b.clone())).collect(),
            bisociation: h.bisociation_pair,
        },
    }]
}

// ═══════════════════════════════════════════════════════════════
// Test — usa lo stato REALE (.bin), non il bootstrap
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{engine::PrometeoTopologyEngine, persistence::PrometeoState};
    use std::path::Path;

    fn load_real_engine() -> PrometeoTopologyEngine {
        let mut engine = PrometeoTopologyEngine::new();
        let path = Path::new("prometeo_topology_state.bin");
        if path.exists() {
            match PrometeoState::load_from_binary(path) {
                Ok(state) => {
                    state.restore_lexicon(&mut engine);
                    engine.recompute_all_word_affinities();
                    println!("[test] stato reale caricato: {} parole", engine.lexicon.word_count());
                }
                Err(e) => println!("[test] errore caricamento .bin: {e} — uso bootstrap"),
            }
        } else {
            println!("[test] .bin non trovato — uso bootstrap (~500 parole)");
        }
        engine
    }

    #[test]
    #[ignore = "carica il .bin reale (128K parole) — esegui con: cargo nextest run -- --ignored test_generate_thoughts_real"]
    fn test_generate_thoughts_real() {
        let mut engine = load_real_engine();

        println!("\n══════════════════════════════════════════");
        println!("  PENSIERI — stato a riposo");
        println!("══════════════════════════════════════════");
        let t0 = generate_thoughts(&engine);
        print_thoughts(&t0);

        // Attiva il campo
        engine.receive("voglio capire chi sono");
        engine.receive("sento qualcosa di strano nel silenzio");
        engine.receive("non ricordo bene il passato, ma sento che cambia");

        println!("\n══════════════════════════════════════════");
        println!("  PENSIERI — dopo 3 input");
        println!("══════════════════════════════════════════");
        let t1 = generate_thoughts(&engine);
        print_thoughts(&t1);

        println!("\n── Riepilogo ──");
        for (label, kind) in &[
            ("Tensioni       ", ThoughtKind::Tension),
            ("Lacune         ", ThoughtKind::Gap),
            ("Ponti mancanti ", ThoughtKind::MissingBridge),
            ("Disconnessioni ", ThoughtKind::Disconnection),
            ("Ipotesi        ", ThoughtKind::Hypothesis),
        ] {
            println!("  {label}: {}", t1.iter().filter(|t| &t.kind == kind).count());
        }
        println!("  TOTALE         : {}", t1.len());

        assert!(!t1.is_empty());
    }

    fn print_thoughts(thoughts: &[Thought]) {
        if thoughts.is_empty() { println!("  (nessun pensiero)"); return; }
        for t in thoughts {
            let kind = match t.kind {
                ThoughtKind::Tension       => "TENSIONE",
                ThoughtKind::Gap           => "LACUNA  ",
                ThoughtKind::MissingBridge => "PONTE?  ",
                ThoughtKind::Disconnection => "ISOLA   ",
                ThoughtKind::Hypothesis    => "IPOTESI ",
                ThoughtKind::AbductiveHypothesis => "ABDUZ.  ",
                ThoughtKind::SelfDiscovery => "SCOPERTA",
                ThoughtKind::Need          => "BISOGNO ",
                ThoughtKind::Desire        => "DESIDERI",
                ThoughtKind::Interlocutor  => "ALTRO   ",
                ThoughtKind::Humor         => "UMORISMO",
            };
            let frattali = if t.fractal_names.is_empty() { "—".to_string() }
                else { t.fractal_names.join(" ↔ ") };
            let parole = if t.words.is_empty() { String::new() }
                else { format!("  [{}]", t.words.join(" vs ")) };
            let extra = match &t.data {
                ThoughtData::TensionData { phase, .. } =>
                    format!(" fase={:.2}π", phase / std::f64::consts::PI),
                ThoughtData::GapData { simplex_count, word_count, .. } =>
                    format!(" simp={simplex_count} parole={word_count}"),
                ThoughtData::MissingBridgeData { proximity, shared_simplices } =>
                    format!(" prox={proximity:.3} shared={shared_simplices}"),
                ThoughtData::DisconnectionData { components } => {
                    let c: Vec<String> = components.iter()
                        .map(|g| format!("[{}]", g.join(",")))
                        .collect();
                    format!(" {}", c.join(" | "))
                },
                ThoughtData::HypothesisData { simplex_id, dimension, activation_count } =>
                    format!(" id={simplex_id} dim={dimension} att={activation_count}"),
                ThoughtData::SelfDiscoveryData { divergence, .. } =>
                    format!(" div={divergence:.3}"),
                ThoughtData::NeedData { level, satisfaction } =>
                    format!(" {level} sat={satisfaction:.2}"),
                ThoughtData::DesireData { name, intensity, .. } =>
                    format!(" {name} int={intensity:.2}"),
                ThoughtData::InterlocutorData { presence, pattern, resonance } =>
                    format!(" pres={presence:.2} pat={pattern} ris={resonance:.2}"),
                ThoughtData::HumorData { incongruity, .. } =>
                    format!(" inc={incongruity:.2}"),
            };
            println!("  [{kind}] str={:.2}  {frattali}{parole}{extra}", t.strength);
        }
    }
}
