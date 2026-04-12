/// Motore di Inferenza — Ragionamento logico sul Knowledge Graph.
///
/// I computer sono macchine logiche: usiamo la logica.
/// Nessuna statistica, nessuna co-occorrenza. Solo regole.
///
/// Regole implementate (Horn clauses):
///   1. Transitività IS_A:    X IS_A Y, Y IS_A Z  ⟹  X IS_A Z
///   2. Ereditarietà HAS:     X IS_A Y, Y HAS P   ⟹  X HAS P
///   3. Ereditarietà DOES:    X IS_A Y, Y DOES A  ⟹  X DOES A
///   4. Causalità transitiva: X CAUSES Y, Y CAUSES Z ⟹ X CAUSES Z (1 hop)
///
/// Uso principale: field_boosts(word) → Vec<(word, strength)>
/// Per ogni parola attiva nel campo, restituisce le parole logicamente
/// correlate da iniettare come boost nel campo topologico.

use std::collections::{HashMap, HashSet, VecDeque};
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

// ═══════════════════════════════════════════════════════════════════════════
// Costanti
// ═══════════════════════════════════════════════════════════════════════════

/// Profondità massima di traversal IS-A (evita loop infiniti su ontologie cicliche)
const MAX_ISA_DEPTH: usize = 5;
/// Profondità massima per catene causali
const MAX_CAUSE_DEPTH: usize = 2;

// ═══════════════════════════════════════════════════════════════════════════
// InferenceEngine
// ═══════════════════════════════════════════════════════════════════════════

/// Motore di inferenza: naviga il KG con regole logiche.
/// Stateless: borrow del KG, nessun caching (il KG è piccolo).
pub struct InferenceEngine<'a> {
    kg: &'a KnowledgeGraph,
}

impl<'a> InferenceEngine<'a> {
    pub fn new(kg: &'a KnowledgeGraph) -> Self {
        Self { kg }
    }

    // ─── Tassonomia ──────────────────────────────────────────────────────────

    /// Catena IS_A: tutti i tipi di `word` (transitivo).
    /// "cane" → ["animale", "essere_vivente", "mammifero"]
    pub fn type_chain(&self, word: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        self.isa_bfs(word, &mut visited, &mut result, 0);
        result
    }

    /// BFS iterativo su IS_A (evita stack overflow su grafi profondi).
    fn isa_bfs(&self, start: &str, visited: &mut HashSet<String>, result: &mut Vec<String>, depth: usize) {
        if depth >= MAX_ISA_DEPTH { return; }
        let parents = self.kg.query_objects(start, RelationType::IsA);
        for parent in parents {
            if !visited.contains(parent) {
                visited.insert(parent.to_string());
                result.push(parent.to_string());
                self.isa_bfs(parent, visited, result, depth + 1);
            }
        }
    }

    /// I tipi diretti di `word` (solo IS_A immediati, non transitivi).
    pub fn direct_types(&self, word: &str) -> Vec<String> {
        self.kg.query_objects(word, RelationType::IsA)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    // ─── Proprietà ────────────────────────────────────────────────────────────

    /// Proprietà dirette + ereditate: cosa HA questa parola?
    /// "cane" → ["pelo", "zampe", "confine"(da nazione? no — solo catena IS_A corretta)]
    ///
    /// Regola: X IS_A Y, Y HAS P ⟹ X HAS P
    pub fn what_has(&self, word: &str) -> Vec<String> {
        let mut result = HashSet::new();
        // Dirette
        for p in self.kg.query_objects(word, RelationType::Has) {
            result.insert(p.to_string());
        }
        // Ereditate via IS_A
        for ancestor in self.type_chain(word) {
            for p in self.kg.query_objects(&ancestor, RelationType::Has) {
                result.insert(p.to_string());
            }
        }
        result.into_iter().collect()
    }

    // ─── Azioni ──────────────────────────────────────────────────────────────

    /// Azioni dirette + ereditate: cosa FA questa parola/concetto?
    ///
    /// Regola: X IS_A Y, Y DOES A ⟹ X DOES A
    pub fn what_does(&self, word: &str) -> Vec<String> {
        let mut result = HashSet::new();
        // Dirette
        for a in self.kg.query_objects(word, RelationType::Does) {
            result.insert(a.to_string());
        }
        // Ereditate via IS_A
        for ancestor in self.type_chain(word) {
            for a in self.kg.query_objects(&ancestor, RelationType::Does) {
                result.insert(a.to_string());
            }
        }
        result.into_iter().collect()
    }

    // ─── Causalità ───────────────────────────────────────────────────────────

    /// Cosa CAUSA questa cosa (direttamente)?
    pub fn what_causes(&self, word: &str) -> Vec<String> {
        self.kg.query_objects(word, RelationType::Causes)
            .into_iter().map(|s| s.to_string()).collect()
    }

    /// Catena causale BFS (max MAX_CAUSE_DEPTH hop).
    pub fn causal_chain(&self, word: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((word.to_string(), 0usize));
        while let Some((current, depth)) = queue.pop_front() {
            if depth >= MAX_CAUSE_DEPTH { continue; }
            for effect in self.kg.query_objects(&current, RelationType::Causes) {
                if visited.insert(effect.to_string()) {
                    result.push(effect.to_string());
                    queue.push_back((effect.to_string(), depth + 1));
                }
            }
        }
        result
    }

    // ─── Relazioni varie ─────────────────────────────────────────────────────

    /// Parti di cui è composta questa cosa (PART_OF inverso: cosa è parte di X?).
    pub fn parts_of(&self, word: &str) -> Vec<String> {
        self.kg.query_subjects(word, RelationType::PartOf)
            .into_iter().map(|s| s.to_string()).collect()
    }

    /// Di cosa è parte questa cosa (PART_OF diretto).
    pub fn part_of_what(&self, word: &str) -> Vec<String> {
        self.kg.query_objects(word, RelationType::PartOf)
            .into_iter().map(|s| s.to_string()).collect()
    }

    /// Opposti di questa cosa.
    pub fn opposites(&self, word: &str) -> Vec<String> {
        let mut res: Vec<String> = self.kg.query_objects(word, RelationType::OppositeOf)
            .into_iter().map(|s| s.to_string()).collect();
        // OPPOSITE_OF è simmetrica
        for s in self.kg.query_subjects(word, RelationType::OppositeOf) {
            if !res.contains(&s.to_string()) { res.push(s.to_string()); }
        }
        res
    }

    /// Simili/sinonimi di questa cosa.
    pub fn similar_to(&self, word: &str) -> Vec<String> {
        let mut res: Vec<String> = self.kg.query_objects(word, RelationType::SimilarTo)
            .into_iter().map(|s| s.to_string()).collect();
        // SIMILAR_TO è simmetrica
        for s in self.kg.query_subjects(word, RelationType::SimilarTo) {
            if !res.contains(&s.to_string()) { res.push(s.to_string()); }
        }
        res
    }

    // ─── API principale — boost per il campo ─────────────────────────────────

    /// Calcola i boost semantici da iniettare nel campo topologico.
    ///
    /// Dato una parola attiva (es. "cane"), restituisce:
    ///   - i suoi tipi IS_A ("animale", "mammifero") con forza alta
    ///   - le sue azioni DOES ("abbaiare", "correre") con forza media
    ///   - le sue proprietà HAS ("pelo", "zampe") con forza media
    ///   - i suoi effetti CAUSES con forza media
    ///   - i suoi simili SIMILAR_TO con forza alta
    ///   - gli opposti OPPOSITE_OF con forza bassa (tensione, non risonanza)
    ///
    /// Forza = confidence × field_boost_strength × decay_per_hop
    pub fn field_boosts(&self, word: &str) -> Vec<(String, f32)> {
        if !self.kg.contains(word) { return vec![]; }

        // Calcola type_chain UNA SOLA VOLTA e riusa per IS_A, DOES, HAS
        let isa_chain = self.type_chain(word);

        // Boosts diretti per DOES e HAS (per distinguere forza diretta vs ereditata)
        let direct_does: HashSet<String> = self.kg.query_objects(word, RelationType::Does)
            .into_iter().map(|s| s.to_string()).collect();
        let direct_has: HashSet<String> = self.kg.query_objects(word, RelationType::Has)
            .into_iter().map(|s| s.to_string()).collect();

        // HashMap per deduplicazione O(1) — mantieni il boost massimo
        let mut map: HashMap<String, f32> = HashMap::new();
        let mut push = |k: String, v: f32| {
            if k != word {
                let e = map.entry(k).or_insert(0.0);
                if v > *e { *e = v; }
            }
        };

        // Phase 48: ogni boost usa confidence per-arco, non costante per tipo.
        // Forza = field_boost_strength(tipo) × confidence_arco × decay_hop

        // IS_A diretti — con confidence per-arco
        let isa_base = RelationType::IsA.field_boost_strength();
        for (t, conf) in self.kg.query_objects_weighted(word, RelationType::IsA) {
            push(t.to_string(), isa_base * conf);
        }
        // IS_A transitivi con decay per hop
        for (i, ancestor) in isa_chain.iter().enumerate().skip(1) {
            let decay = 0.7_f32.powi(i as i32);
            push(ancestor.clone(), isa_base * decay);
        }

        // DOES diretti + ereditati — con confidence per-arco
        let does_base = RelationType::Does.field_boost_strength();
        for (a, conf) in self.kg.query_objects_weighted(word, RelationType::Does) {
            push(a.to_string(), does_base * conf);
        }
        for ancestor in &isa_chain {
            for a in self.kg.query_objects(ancestor, RelationType::Does) {
                if !direct_does.contains(a) {
                    push(a.to_string(), does_base * 0.6);
                }
            }
        }

        // HAS diretti + ereditati — con confidence per-arco
        let has_base = RelationType::Has.field_boost_strength();
        for (p, conf) in self.kg.query_objects_weighted(word, RelationType::Has) {
            push(p.to_string(), has_base * conf);
        }
        for ancestor in &isa_chain {
            for p in self.kg.query_objects(ancestor, RelationType::Has) {
                if !direct_has.contains(p) {
                    push(p.to_string(), has_base * 0.6);
                }
            }
        }

        // CAUSES, SIMILAR_TO, OPPOSITE_OF, PART_OF, USED_FOR — con confidence per-arco
        // Phase 67: via words attivate come ponti contestuali (0.5× forza target)
        let via_relations = [
            RelationType::Causes, RelationType::SimilarTo, RelationType::OppositeOf,
            RelationType::PartOf, RelationType::FeelsAs, RelationType::WondersAbout,
            RelationType::RemembersAs,
        ];
        for rel in &via_relations {
            let base = rel.field_boost_strength();
            for (obj, conf, via) in self.kg.query_objects_with_via(word, *rel) {
                push(obj.to_string(), base * conf);
                // Via: il ponte contestuale entra nel campo a forza ridotta
                if let Some(via_word) = via {
                    if via_word != word {
                        push(via_word.to_string(), base * conf * 0.5);
                    }
                }
            }
        }

        let mut result: Vec<(String, f32)> = map.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result.truncate(20);
        result
    }

    /// Risposta a "cosa è X?": tipo diretto + caratteristiche principali.
    /// Usato per generare risposte a domande definitorie.
    pub fn define(&self, word: &str) -> Option<String> {
        let types = self.direct_types(word);
        if types.is_empty() { return None; }
        let main_type = &types[0];
        let props = self.kg.query_objects(word, RelationType::Has);
        let actions = self.kg.query_objects(word, RelationType::Does);

        let mut parts = vec![format!("{} è un/a {}", word, main_type)];
        if !props.is_empty() {
            parts.push(format!("ha {}", props[0]));
        }
        if !actions.is_empty() {
            parts.push(format!("{}", actions[0]));
        }
        Some(parts.join(", ") + ".")
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::knowledge_graph::KnowledgeGraph;
    use crate::topology::relation::RelationType;

    fn build_kg() -> KnowledgeGraph {
        let mut kg = KnowledgeGraph::new();
        // Tassonomia animali
        kg.add("cane", RelationType::IsA, "mammifero");
        kg.add("mammifero", RelationType::IsA, "animale");
        kg.add("animale", RelationType::IsA, "essere_vivente");
        // Azioni
        kg.add("cane", RelationType::Does, "abbaiare");
        kg.add("animale", RelationType::Does, "mangiare");
        kg.add("animale", RelationType::Does, "dormire");
        kg.add("essere_vivente", RelationType::Does, "respirare");
        // Proprietà
        kg.add("cane", RelationType::Has, "pelo");
        kg.add("mammifero", RelationType::Has, "sangue_caldo");
        // Causalità
        kg.add("paura", RelationType::Causes, "tremore");
        kg.add("tremore", RelationType::Causes, "agitazione");
        // Contrari
        kg.add("caldo", RelationType::OppositeOf, "freddo");
        // Geopolitica
        kg.add("germania", RelationType::IsA, "nazione");
        kg.add("nazione", RelationType::Has, "confine");
        kg.add("nazione", RelationType::Has, "capitale");
        // Saluto
        kg.add("ciao", RelationType::SimilarTo, "saluto");
        kg.add("ciao", RelationType::SimilarTo, "benvenuto");
        kg
    }

    #[test]
    fn test_type_chain_transitivo() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let chain = engine.type_chain("cane");
        assert!(chain.contains(&"mammifero".to_string()), "cane IS-A mammifero");
        assert!(chain.contains(&"animale".to_string()), "cane IS-A animale (transitivo)");
        assert!(chain.contains(&"essere_vivente".to_string()), "cane IS-A essere_vivente (transitivo)");
    }

    #[test]
    fn test_what_does_eredita() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let actions = engine.what_does("cane");
        assert!(actions.contains(&"abbaiare".to_string()), "cane DOES abbaiare (diretto)");
        assert!(actions.contains(&"mangiare".to_string()), "cane DOES mangiare (da animale)");
        assert!(actions.contains(&"respirare".to_string()), "cane DOES respirare (da essere_vivente)");
    }

    #[test]
    fn test_what_has_eredita() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let props = engine.what_has("cane");
        assert!(props.contains(&"pelo".to_string()), "cane HAS pelo (diretto)");
        assert!(props.contains(&"sangue_caldo".to_string()), "cane HAS sangue_caldo (da mammifero)");
    }

    #[test]
    fn test_causal_chain() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let chain = engine.causal_chain("paura");
        assert!(chain.contains(&"tremore".to_string()), "paura CAUSES tremore");
        assert!(chain.contains(&"agitazione".to_string()), "paura CAUSES agitazione (transitivo)");
    }

    #[test]
    fn test_opposites_simmetrico() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let opp_caldo = engine.opposites("caldo");
        let opp_freddo = engine.opposites("freddo");
        assert!(opp_caldo.contains(&"freddo".to_string()), "caldo ↔ freddo");
        assert!(opp_freddo.contains(&"caldo".to_string()), "freddo ↔ caldo (simmetria)");
    }

    #[test]
    fn test_similar_to_simmetrico() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let sim = engine.similar_to("ciao");
        assert!(sim.contains(&"saluto".to_string()));
        let sim2 = engine.similar_to("saluto");
        assert!(sim2.contains(&"ciao".to_string()), "SIMILAR_TO è simmetrica");
    }

    #[test]
    fn test_field_boosts_cane() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let boosts = engine.field_boosts("cane");
        assert!(!boosts.is_empty(), "cane deve produrre boost");
        let words: Vec<&str> = boosts.iter().map(|(w, _)| w.as_str()).collect();
        assert!(words.contains(&"animale"), "boost deve includere animale");
        assert!(words.contains(&"abbaiare"), "boost deve includere abbaiare");
    }

    #[test]
    fn test_field_boosts_ciao() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let boosts = engine.field_boosts("ciao");
        let words: Vec<&str> = boosts.iter().map(|(w, _)| w.as_str()).collect();
        assert!(words.contains(&"saluto"), "ciao → saluto (SIMILAR_TO)");
    }

    #[test]
    fn test_field_boosts_germania_non_contamina() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let boosts_ciao = engine.field_boosts("ciao");
        let words: Vec<&str> = boosts_ciao.iter().map(|(w, _)| w.as_str()).collect();
        // "germania" NON deve apparire nei boost di "ciao"
        assert!(!words.contains(&"germania"), "ciao NON deve attivare germania");
    }

    #[test]
    fn test_no_boosts_for_unknown_word() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let boosts = engine.field_boosts("unicorno_inesistente");
        assert!(boosts.is_empty(), "parola sconosciuta = nessun boost");
    }

    #[test]
    fn test_geopolitica() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        // germania è una nazione → ha confine, ha capitale (ereditati)
        let props = engine.what_has("germania");
        assert!(props.contains(&"confine".to_string()), "germania HAS confine (da nazione)");
        assert!(props.contains(&"capitale".to_string()), "germania HAS capitale (da nazione)");
    }

    #[test]
    fn test_define() {
        let kg = build_kg();
        let engine = InferenceEngine::new(&kg);
        let def = engine.define("cane");
        assert!(def.is_some());
        let s = def.unwrap();
        assert!(s.contains("mammifero") || s.contains("animale"));
    }
}
