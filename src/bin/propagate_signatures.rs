/// Phase 70 v4 — Propagazione firme 8D: cascata IS_A + coerenza vicinato.
///
/// IDEA (post-discussione utente):
///
///   Fase A — Cascata IS_A (BFS dagli hub)
///     Hub: firma curata, fissa, intoccabile.
///     Parola con padre IS_A firmato: COPIA il padre più specifico
///     (degree minore = più informativo). Niente media, niente drift.
///     Iterativa fino a fixed-point.
///
///   Fase B — Coerenza con il vicinato (UNA passata, content-aware)
///     Ogni parola firmata in Fase A viene tirata verso le firme dei suoi
///     vicini, con peso per tipo di relazione. NIENTE +0.04 ciechi: la firma
///     si sposta verso COSA è connessa, non per il fatto di esserlo.
///     Sonno → riposo (passivo) tira sonno verso passività.
///     Fuoco → distruzione (intenso) tira fuoco verso intensità.
///
///       SIMILAR_TO   → 0.30 (tira tutte 8 dim)
///       OPPOSITE_OF  → 0.20 (mirror valenza/agency)
///       PART_OF out  → 0.15 / in 0.10
///       TRANSFORMS_INTO → 0.10
///       CAUSES out   → 0.10 / in 0.05
///       DOES out     → 0.10 / in 0.05
///       HAS out      → 0.08 / in 0.05
///       USED_FOR out → 0.08 / in 0.05
///
///     Cap finale: lo spostamento in norma L2 ≤ 0.30.
///
///   Parole orfane (no cammino IS_A → hub):
///     1. SIMILAR_TO firmato → COPIA (vicino con confidence massima)
///     2. OPPOSITE_OF firmato → MIRROR (vicino con confidence massima)
///     3. Sennò: [0.5]×8 + Fase B.
///
/// Backup: prometeo_topology_state.bin.pre_p70_v4
///
/// Uso: cargo run --release --bin propagate-signatures

use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::knowledge_graph::{KnowledgeGraph, KgSnapshot};
use prometeo::topology::primitive::PrimitiveCore;
use prometeo::topology::relation::RelationType;

const STATE_PATH:  &str = "prometeo_topology_state.bin";
const BACKUP_PATH: &str = "prometeo_topology_state.bin.pre_p70_v4";
const KG_PATH:     &str = "prometeo_kg.json";
const HUB_PATH:    &str = "data/anchors/hub_signatures.tsv";
const SIGNED_LIST: &str = "data/phase70_signed_words.txt";

const NEUTRAL: f64 = 0.50;
const PHASE_A_MAX_ROUNDS: usize = 50;

/// Cap dello spostamento in Fase B (norma L2 del vettore-delta).
/// Una parola non si allontana più di questo dalla sua firma di Fase A.
/// Preserva l'eredità tassonomica e impedisce regressione al centro.
const PHASE_B_CAP: f64 = 0.30;

/// Forza di tiro per tipo di relazione. outgoing = parola è soggetto.
fn pull_strength(rel: RelationType, outgoing: bool) -> f64 {
    use RelationType::*;
    match rel {
        SimilarTo               => 0.30,
        OppositeOf              => 0.20,
        PartOf if outgoing      => 0.15,
        PartOf                  => 0.10,
        TransformsInto          => 0.10,
        Causes if outgoing      => 0.10,
        Causes                  => 0.05,
        Does   if outgoing      => 0.10,
        Does                    => 0.05,
        Has    if outgoing      => 0.08,
        Has                     => 0.05,
        UsedFor if outgoing     => 0.08,
        UsedFor                 => 0.05,
        Enables if outgoing     => 0.06,
        Enables                 => 0.04,
        Requires if outgoing    => 0.06,
        Requires                => 0.04,
        IsA                     => 0.0,  // gestita in Fase A, no residuo
        _                       => 0.05,
    }
}

/// Applica modulazione content-aware: tira `base` verso le firme dei vicini.
/// Cap L2 dello spostamento totale a `PHASE_B_CAP`.
fn modulate(
    base: [f64; 8],
    neighbors: &[(RelationType, [f64; 8], f32, bool)],
) -> [f64; 8] {
    let mut delta = [0.0f64; 8];

    for (rel, other_sig, conf, outgoing) in neighbors {
        let pull = pull_strength(*rel, *outgoing) * (*conf as f64).clamp(0.1, 1.0);
        if pull == 0.0 { continue; }

        if *rel == RelationType::OppositeOf {
            // Mirror: target_v = 1 - other.valence; target_a = 1 - other.agency.
            let target_v = 1.0 - other_sig[7];
            let target_a = 1.0 - other_sig[0];
            delta[7] += (target_v - base[7]) * pull;
            delta[0] += (target_a - base[0]) * pull * 0.5;  // agency più morbida
        } else {
            // Tira tutte 8 dim verso il vicino (proporzionale alla distanza).
            for i in 0..8 {
                delta[i] += (other_sig[i] - base[i]) * pull;
            }
        }
    }

    // Cap norma L2 dello spostamento.
    let mag = delta.iter().map(|d| d * d).sum::<f64>().sqrt();
    let scale = if mag > PHASE_B_CAP { PHASE_B_CAP / mag } else { 1.0 };

    let mut new_sig = [0.0f64; 8];
    for i in 0..8 {
        new_sig[i] = (base[i] + delta[i] * scale).clamp(0.0, 1.0);
    }
    new_sig
}

// ---- Caricamento hub ------------------------------------------------------

fn load_hub_signatures(path: &Path) -> HashMap<String, [f64; 8]> {
    let mut out = HashMap::new();
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => { eprintln!("WARN: {} non trovato.", path.display()); return out; }
    };
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if line.starts_with("word\t") { continue; }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 9 { continue; }
        let word = parts[0].trim().to_string();
        let mut sig = [NEUTRAL; 8];
        let mut ok = true;
        for i in 0..8 {
            match parts[i + 1].trim().parse::<f64>() {
                Ok(v) => sig[i] = v.clamp(0.0, 1.0),
                Err(_) => { ok = false; break; }
            }
        }
        if ok { out.insert(word, sig); }
    }
    out
}

// ---- MAIN -----------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    let root = find_project_root();
    let bin_path = root.join(STATE_PATH);
    let kg_path  = root.join(KG_PATH);
    let backup   = root.join(BACKUP_PATH);

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  PHASE 70 v4 — IS_A CASCADE + COERENZA VICINATO         ║");
    println!("║  Single most-specific parent · content-aware pull        ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    if !kg_path.exists() {
        eprintln!("ERRORE: {} non trovato.", kg_path.display());
        std::process::exit(1);
    }
    println!("Carico KG da {}...", kg_path.display());
    let kg_json = std::fs::read_to_string(&kg_path)?;
    let kg_snap: KgSnapshot = serde_json::from_str(&kg_json)?;
    let kg = KnowledgeGraph::from_snapshot(kg_snap);
    println!("  KG: {} archi, {} nodi", kg.edge_count, kg.node_count);

    println!("Carico stato da {}...", bin_path.display());
    let state = PrometeoState::load_from_binary(Path::new(&bin_path))
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("  Lessico: {} parole", state.lexicon.words.len());

    let mut engine = PrometeoTopologyEngine::new();
    state.restore_lexicon(&mut engine);
    engine.kg = kg;

    let hub_path = root.join(HUB_PATH);
    let hub_sigs = load_hub_signatures(&hub_path);
    let hub_set: HashSet<String> = hub_sigs.keys().cloned().collect();
    let mut hub_in_lex = 0usize;
    for w in &hub_set {
        if engine.lexicon.get(w).is_some() { hub_in_lex += 1; }
        else { eprintln!("  WARN: hub '{}' non nel lessico", w); }
    }
    println!("\n{}/{} hub presenti nel lessico", hub_in_lex, hub_set.len());

    let words: Vec<String> = engine.lexicon.patterns_iter()
        .map(|(w, _)| w.to_string())
        .collect();

    // Init: hub fissi, tutte le altre None.
    let mut sigs: HashMap<String, Option<[f64; 8]>> = HashMap::with_capacity(words.len());
    for word in &words {
        if let Some(s) = hub_sigs.get(word) {
            sigs.insert(word.clone(), Some(*s));
        } else {
            sigs.insert(word.clone(), None);
        }
    }
    println!("Inizializzazione: {} hub fissi, {} a None.\n",
             hub_in_lex, words.len() - hub_in_lex);

    // ─── FASE A: cascata IS_A (copia da padre più specifico) ──────────────
    println!("─── FASE A: CASCATA IS_A (copia da padre più specifico) ──");
    for round in 1..=PHASE_A_MAX_ROUNDS {
        let mut changed = 0usize;
        let snapshot = sigs.clone();
        for word in &words {
            if hub_set.contains(word) { continue; }
            if snapshot[word].is_some() { continue; }

            // Cerca il padre IS_A firmato con degree minore (più specifico).
            let mut best: Option<(usize, [f64; 8])> = None;
            for (rel, target, _conf) in engine.kg.all_outgoing(word) {
                if rel != RelationType::IsA { continue; }
                if let Some(Some(p_sig)) = snapshot.get(target) {
                    let deg = engine.kg.out_degree(target) + engine.kg.in_degree(target);
                    if best.map_or(true, |(d, _)| deg < d) {
                        best = Some((deg, *p_sig));
                    }
                }
            }

            if let Some((_, p_sig)) = best {
                sigs.insert(word.clone(), Some(p_sig));  // COPIA esatta
                changed += 1;
            }
        }
        let firmed = sigs.values().filter(|v| v.is_some()).count();
        println!("  round {:>2}: +{:>5} firmate (totale {} / {})",
                 round, changed, firmed, words.len());
        if changed == 0 {
            println!("  ✓ Fase A stabile al round {}", round);
            break;
        }
    }

    let firmed_after_a = sigs.values().filter(|v| v.is_some()).count();
    let none_after_a = words.len() - firmed_after_a;
    println!("\n  Fine Fase A: {} firmate, {} ancora None", firmed_after_a, none_after_a);

    // ─── FASE B: coerenza con vicinato (single-pass content-aware) ───────
    println!("\n─── FASE B: COERENZA VICINATO (single-pass) ──────────────");

    // Snapshot post-A: Fase B legge tutto stabile.
    let snapshot = sigs.clone();
    let mut new_sigs = sigs.clone();
    let mut newly_firmed = 0usize;
    let mut modulated = 0usize;
    let mut total_displacement = 0.0f64;
    let mut still_orphan = 0usize;

    for word in &words {
        if hub_set.contains(word) { continue; }

        // Raccogli vicini con firma (escludendo IS_A — già gestita).
        let mut neighbors: Vec<(RelationType, [f64; 8], f32, bool)> = Vec::new();
        for (rel, target, conf) in engine.kg.all_outgoing(word) {
            if rel == RelationType::IsA { continue; }
            if let Some(Some(s)) = snapshot.get(target) {
                neighbors.push((rel, *s, conf, true));
            }
        }
        for (rel, source, conf) in engine.kg.all_incoming(word) {
            if rel == RelationType::IsA { continue; }
            if let Some(Some(s)) = snapshot.get(source) {
                neighbors.push((rel, *s, conf, false));
            }
        }

        let base_sig = snapshot[word];

        if base_sig.is_none() {
            // Parola orfana: cerca prima SIMILAR_TO, poi OPPOSITE_OF, poi neutro+modula.
            if neighbors.is_empty() {
                still_orphan += 1;
                continue;
            }

            // 1. SIMILAR_TO firmato con confidence massima → COPIA.
            let best_similar = neighbors.iter()
                .filter(|(r, _, _, _)| *r == RelationType::SimilarTo)
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((_, sig, _, _)) = best_similar {
                new_sigs.insert(word.clone(), Some(*sig));
                newly_firmed += 1;
                continue;
            }

            // 2. OPPOSITE_OF firmato con confidence massima → MIRROR.
            let best_opposite = neighbors.iter()
                .filter(|(r, _, _, _)| *r == RelationType::OppositeOf)
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((_, sig, _, _)) = best_opposite {
                let mut m = *sig;
                m[7] = 1.0 - m[7];          // valenza specchiata
                m[0] = 1.0 - m[0];          // agency specchiata
                new_sigs.insert(word.clone(), Some(m));
                newly_firmed += 1;
                continue;
            }

            // 3. Sennò: parti da neutro e applica le modulazioni dei vicini.
            let neutral = [NEUTRAL; 8];
            let modulated_sig = modulate(neutral, &neighbors);
            new_sigs.insert(word.clone(), Some(modulated_sig));
            newly_firmed += 1;
        } else {
            // Parola con firma da Fase A: applica content-aware pull.
            let cur = base_sig.unwrap();
            if neighbors.is_empty() { continue; }

            let new_sig = modulate(cur, &neighbors);

            let disp = (0..8).map(|i| (new_sig[i] - cur[i]).powi(2))
                .sum::<f64>().sqrt();
            if disp > 1e-6 {
                new_sigs.insert(word.clone(), Some(new_sig));
                total_displacement += disp;
                modulated += 1;
            }
        }
    }

    sigs = new_sigs;
    let avg_disp = if modulated > 0 { total_displacement / modulated as f64 } else { 0.0 };
    println!("  Single-pass: +{} nuove firme (orfane), {} modulate, spostamento medio {:.4}",
             newly_firmed, modulated, avg_disp);
    println!("  Parole isolate (nessun arco): {}", still_orphan);

    let firmed_after_b = sigs.values().filter(|v| v.is_some()).count();
    let still_none = words.len() - firmed_after_b;
    println!("\n  Fine Fase B: {} firmate, {} ancora None", firmed_after_b, still_none);

    // ─── Diagnostica: dove finiscono le parole non firmate? ────────────
    let mut zero_arcs = 0usize;     // parole davvero senza alcun arco KG
    let mut only_isa_unsigned = 0usize;  // solo IS_A verso parole non firmate
    let mut subgraph_isolated = 0usize;  // archi solo verso parole non firmate
    for word in &words {
        if sigs[word].is_some() { continue; }
        let outs = engine.kg.all_outgoing(word);
        let ins  = engine.kg.all_incoming(word);
        if outs.is_empty() && ins.is_empty() {
            zero_arcs += 1;
            continue;
        }
        let non_isa = outs.iter().filter(|(r, _, _)| *r != RelationType::IsA).count()
                    + ins.iter().filter(|(r, _, _)| *r != RelationType::IsA).count();
        if non_isa == 0 {
            only_isa_unsigned += 1;
        } else {
            subgraph_isolated += 1;
        }
    }
    println!("\n  Breakdown delle {} non firmate:", still_none);
    println!("    {:>5} parole senza alcun arco KG (true isolates)", zero_arcs);
    println!("    {:>5} parole con solo IS_A verso parole non firmate", only_isa_unsigned);
    println!("    {:>5} parole con altri archi solo verso parole non firmate", subgraph_isolated);

    // ─── Applica firme al lessico ─────────────────────────────────────────
    println!("\n─── APPLICAZIONE FIRME ───────────────────────────");
    let mut applied = 0usize;
    let mut left_neutral = 0usize;
    for (word, sig_opt) in &sigs {
        if let Some(pat) = engine.lexicon.get_mut(word) {
            if let Some(sig) = sig_opt {
                pat.signature = PrimitiveCore::new(*sig);
                applied += 1;
            } else {
                left_neutral += 1;
            }
        }
    }
    println!("  {} firme aggiornate, {} parole isolate (firma precedente preservata).",
             applied, left_neutral);

    // ─── Ricalcola affinità frattali ──────────────────────────────────────
    println!("\n─── RICALCOLO AFFINITÀ FRATTALI ──────────────────");
    let t = std::time::Instant::now();
    engine.recompute_all_word_affinities();
    println!("  Completato in {}ms", t.elapsed().as_millis());

    // ─── Verifica campione ────────────────────────────────────────────────
    println!("\n─── VERIFICA CAMPIONE ────────────────────────────");
    println!("  {:15} Ag   Pe   In   Te   Co   Cp   De   Va", "parola");
    for w in &["tristezza", "gioia", "paura", "cane", "essere", "correre",
               "pietra", "amore", "dormire", "azione", "tempo", "vita",
               "cesoia", "forbici", "lupo", "sonno", "fuoco", "vaso",
               "riposo", "distruzione"] {
        if let Some(p) = engine.lexicon.get(w) {
            let s = p.signature.values();
            println!("  {:15} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} {:.2}",
                     w, s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]);
        } else {
            println!("  {:15} (non nel lessico)", w);
        }
    }

    // ─── Backup + salva ──────────────────────────────────────────────────
    println!();
    if bin_path.exists() && !backup.exists() {
        std::fs::copy(&bin_path, &backup)?;
        println!("Backup salvato: {}", backup.display());
    } else if backup.exists() {
        println!("Backup già esistente: {}", backup.display());
    }
    let new_state = PrometeoState::capture(&engine);
    new_state.save_to_binary(Path::new(&bin_path))
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("Stato salvato: {}", bin_path.display());

    // ─── Esporta lista parole firmate (per filtro server-side campovasto) ──
    let signed_path = root.join(SIGNED_LIST);
    let mut signed_words: Vec<&String> = sigs.iter()
        .filter_map(|(w, s)| if s.is_some() { Some(w) } else { None })
        .collect();
    signed_words.sort();
    let body: String = signed_words.iter().map(|w| w.as_str()).collect::<Vec<_>>().join("\n");
    if let Some(parent) = signed_path.parent() { std::fs::create_dir_all(parent).ok(); }
    std::fs::write(&signed_path, body)?;
    println!("Lista firmate salvata: {} ({} parole)", signed_path.display(), signed_words.len());

    println!("\n✓ Phase 70 v4 completata.");
    Ok(())
}

fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for _ in 0..5 {
        if dir.join("Cargo.toml").exists() { return dir; }
        if let Some(p) = dir.parent() { dir = p.to_path_buf(); } else { break; }
    }
    PathBuf::from(".")
}
