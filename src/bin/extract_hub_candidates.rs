/// Estrae i candidati hub per la curazione manuale delle firme (Phase 70).
///
/// Per ogni parola nel lessico con grado nel KG, calcola:
///   - grado totale (in + out)
///   - frattale dominante e affinità massima
///   - firma 8D corrente (Phase 63 statistica, da rivedere)
///   - 4 archi rappresentativi (uno per relazione più informativa)
///
/// Output: data/anchors/hub_candidates.tsv (TOP_N parole, ordinato per grado).
///         Ognuna è un suggerimento per la curazione: leggi gli archi,
///         decidi una firma migliore, riscrivila nel TSV (o copiala in
///         hub_signatures.tsv finale).
///
/// Filtro: degree ≥ 5 (le parole davvero "hub" del KG attuale).
///
/// Uso:
///     cargo run --release --bin extract-hub-candidates -- [N=80]

use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;
use prometeo::topology::relation::RelationType;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const OUT_PATH: &str = "data/anchors/hub_candidates.tsv";
const DEFAULT_N: usize = 80;
const MIN_DEG: usize = 5;
// Massimo archi rappresentativi per parola
const MAX_EDGES_PREVIEW: usize = 4;

fn main() {
    let n: usize = std::env::args().nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_N);

    println!("Caricamento prometeo_topology_state.bin ...");
    let state = match PrometeoState::load_from_binary(Path::new("prometeo_topology_state.bin")) {
        Ok(s) => s,
        Err(e) => { eprintln!("Errore: {}", e); std::process::exit(2); }
    };
    let mut engine = PrometeoTopologyEngine::new();
    state.restore_lexicon(&mut engine);
    // Lo state non include il KG: va caricato esplicitamente.
    println!("Caricamento prometeo_kg.json ...");
    engine.load_kg_from_file(Path::new("prometeo_kg.json"));

    // Costruisci ranking per grado
    let mut entries: Vec<(String, usize, [f64; 8], u32, f64)> = Vec::new();
    for (word, pattern) in engine.lexicon.patterns_iter() {
        let deg = engine.kg.out_degree(word) + engine.kg.in_degree(word);
        if deg < MIN_DEG { continue; }
        let sig = *pattern.signature.values();
        let (dom_fid, max_aff) = pattern.fractal_affinities.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(&k, &v)| (k, v))
            .unwrap_or((0, 0.0));
        entries.push((word.clone(), deg, sig, dom_fid, max_aff));
    }
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    entries.truncate(n);

    // Prepara output
    if let Some(p) = Path::new(OUT_PATH).parent() { fs::create_dir_all(p).unwrap(); }
    let mut f = File::create(OUT_PATH).unwrap();

    writeln!(f, "# Candidati HUB per la curazione manuale delle firme (Phase 70).").unwrap();
    writeln!(f, "# Top {} parole del KG ordinate per grado decrescente (in + out).", entries.len()).unwrap();
    writeln!(f, "#").unwrap();
    writeln!(f, "# Per ogni hub trovi:").unwrap();
    writeln!(f, "#   - grado totale, frattale dominante (ID + nome + affinità)").unwrap();
    writeln!(f, "#   - firma 8D ATTUALE (Phase 63, da rivedere)").unwrap();
    writeln!(f, "#   - 4 archi rappresentativi del KG").unwrap();
    writeln!(f, "#").unwrap();
    writeln!(f, "# Workflow di curazione:").unwrap();
    writeln!(f, "#   1) leggi parola + archi rappresentativi").unwrap();
    writeln!(f, "#   2) decidi i valori 8D (0.0=Yin estremo, 1.0=Yang estremo, 0.5=neutro)").unwrap();
    writeln!(f, "#   3) copia la riga 'sig:' in data/anchors/hub_signatures.tsv con i nuovi valori").unwrap();
    writeln!(f, "#").unwrap();
    writeln!(f, "# Riferimento dimensioni:").unwrap();
    writeln!(f, "#   agency      = agisce o subisce (alto = agente)").unwrap();
    writeln!(f, "#   permanenza  = persiste o svanisce").unwrap();
    writeln!(f, "#   intensita   = movimento interno (alto = ardente)").unwrap();
    writeln!(f, "#   tempo       = futuro o passato").unwrap();
    writeln!(f, "#   confine     = grande o piccolo / netto o sfumato").unwrap();
    writeln!(f, "#   complessita = intricato o semplice").unwrap();
    writeln!(f, "#   definizione = preciso o vago").unwrap();
    writeln!(f, "#   valenza     = attrae (alto) o respinge (basso)").unwrap();
    writeln!(f, "#").unwrap();
    writeln!(f, "# === Lista hub === ({} candidati)", entries.len()).unwrap();
    writeln!(f).unwrap();

    // Carica nomi frattali per ID
    let registry = prometeo::topology::fractal::bootstrap_fractals();

    for (rank, (word, deg, sig, dom_fid, aff)) in entries.iter().enumerate() {
        let frac_name = registry.get(*dom_fid).map(|f| f.name.clone()).unwrap_or_else(|| format!("F{}", dom_fid));
        writeln!(f, "## [{}] {} — grado {} — frattale {} ({}, aff {:.2})",
            rank + 1, word, deg, dom_fid, frac_name, aff).unwrap();

        // 4 archi rappresentativi: prendi primi MAX_EDGES_PREVIEW per varietà di RelationType
        let mut seen_rels: HashMap<RelationType, usize> = HashMap::new();
        let mut rendered = 0usize;
        let outs: Vec<_> = engine.kg.all_outgoing(word);
        let ins:  Vec<_> = engine.kg.all_incoming(word);

        for (rel, target, conf) in outs.iter() {
            if rendered >= MAX_EDGES_PREVIEW { break; }
            let count = seen_rels.entry(*rel).or_insert(0);
            if *count >= 2 { continue; }
            *count += 1;
            writeln!(f, "#   → {} {} (conf {:.2})", rel.as_str(), target, conf).unwrap();
            rendered += 1;
        }
        for (rel, source, conf) in ins.iter() {
            if rendered >= MAX_EDGES_PREVIEW { break; }
            let count = seen_rels.entry(*rel).or_insert(0);
            if *count >= 2 { continue; }
            *count += 1;
            writeln!(f, "#   ← {} {} (conf {:.2})", source, rel.as_str(), conf).unwrap();
            rendered += 1;
        }

        // Firma corrente come riga TSV (commentata, da copiare e modificare)
        writeln!(f, "# sig attuale (Phase 63):").unwrap();
        writeln!(f, "#sig:\t{}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
            word, sig[0], sig[1], sig[2], sig[3], sig[4], sig[5], sig[6], sig[7]).unwrap();
        writeln!(f).unwrap();
    }

    println!("Generato {} con {} candidati hub.", OUT_PATH, entries.len());
    println!();
    println!("Top 10 hub:");
    for (i, (w, d, _, fid, _)) in entries.iter().take(10).enumerate() {
        let frac_name = registry.get(*fid).map(|f| f.name.as_str()).unwrap_or("?");
        println!("  {:>2}. {:<20} grado {:>4}  frattale {} ({})", i + 1, w, d, fid, frac_name);
    }
}
