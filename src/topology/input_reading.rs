/// Comprensione dell'atto comunicativo — Phase 55.
///
/// Phase 41b usava KnowledgeBase + delta frattale per classificare. Problema:
/// qualsiasi input che attivava ARMONIA veniva classificato come Greeting.
///
/// Phase 55: usa il KG con IS_A chain per classificare. "ciao" IS_A "saluto" → Greeting.
/// "pioggia" non IS_A "saluto" → non Greeting. Logica, non euristiche.
///
/// "Prometeo non memorizza tutti i saluti: capisce cosa è un saluto via IS_A chain."

use crate::topology::fractal::FractalId;
use crate::topology::lexicon::Lexicon;
use crate::topology::knowledge::{KnowledgeBase, KnowledgeDomain};
use crate::topology::knowledge_graph::KnowledgeGraph;
use crate::topology::relation::RelationType;

/// Atto comunicativo rilevato dall'input.
/// Ordine di priorità: Greeting > SelfQuery > Question > EmotionalExpr > Declaration.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum InputAct {
    /// Saluto — riconosciuto via dominio Social nella KnowledgeBase
    Greeting,
    /// Domanda su Prometeo stesso — `?` + dominio Self_ nella KnowledgeBase
    SelfQuery,
    /// Domanda generica — solo marcatore sintattico `?`
    Question,
    /// Espressione emotiva — dominio Emotional nella KnowledgeBase (delta EMOZIONE/CORPO)
    EmotionalExpr,
    /// Dichiarazione generica — tutto il resto
    Declaration,
}

/// Lettura strutturata dell'input corrente.
#[derive(Debug, Clone)]
pub struct InputReading {
    pub act: InputAct,
    /// Intensità dell'atto comunicativo (0..1) — media top-3 delta frattali assoluti
    pub intensity: f64,
    /// Parola più stabile dell'input (se presente nel lessico)
    pub salient_word: Option<String>,
}

/// Legge l'atto comunicativo usando logica IS_A dal Knowledge Graph.
///
/// Phase 55: classificazione basata su catene IS_A nel KG.
/// "ciao" IS_A "saluto" → Greeting.
/// "triste" IS_A "emozione" → EmotionalExpr.
/// "pioggia" non IS_A nessun concetto chiave → Declaration.
///
/// Il KG come parametro opzionale: se None, fallback alla KB+delta (backward compat test).
pub fn read_input(
    raw_words: &[String],
    raw_text: &str,
    frattale_delta: &[(FractalId, f64)],
    knowledge_base: &KnowledgeBase,
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
) -> InputReading {
    // ── Parola più stabile dell'input ────────────────────────────────────────
    let salient_word = raw_words.iter()
        .filter(|w| w.len() >= 3)
        .filter_map(|w| lexicon.get(w).map(|p| (w.clone(), p.stability)))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(w, _)| w);

    // ── Intensità: media top-3 delta assoluti ────────────────────────────────
    let intensity = {
        let mut deltas: Vec<f64> = frattale_delta.iter().map(|(_, d)| d.abs()).collect();
        deltas.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let top3: Vec<f64> = deltas.iter().take(3).copied().collect();
        if top3.is_empty() { 0.0 } else { top3.iter().sum::<f64>() / top3.len() as f64 }
    };

    // ── `?` come unico marcatore sintattico ──────────────────────────────────
    let has_question_mark = raw_text.contains('?');

    // ── Phase 55: Classificazione via IS_A chain nel KG ──────────────────────
    // Per ogni parola dell'input, risaliamo la catena IS_A. Se troviamo un
    // concetto chiave (saluto, emozione, sentimento, identità), classifichiamo.
    // Concetti chiave per Greeting: "saluto", "salutare"
    // Concetti chiave per Emotional: "emozione", "sentimento", "stato_d_animo"
    // Concetti chiave per Self: "identità", "coscienza", "sé"
    let (has_greeting, has_emotional, has_self_ref) = if let Some(kg) = kg {
        let mut greeting = false;
        let mut emotional = false;
        let mut self_ref = false;

        for word in raw_words.iter() {
            // IS_A diretti (1 hop) — non transitivi profondi per efficienza
            let parents: Vec<&str> = kg.query_objects(word, RelationType::IsA);
            for parent in &parents {
                let p = parent.to_lowercase();
                if p == "saluto" || p == "salutare" || p == "saluti" {
                    greeting = true;
                }
                if p == "emozione" || p == "sentimento" || p == "stato_d_animo"
                    || p == "sensazione" || p == "affetto" {
                    emotional = true;
                }
                if p == "identità" || p == "coscienza" || p == "persona" {
                    self_ref = true;
                }
            }
            // Anche IS_A 2-hop: "ciao" IS_A "interiezione" IS_A "saluto"?
            // Per ora 1-hop basta se il KG è ben fatto.

            // Fallback: la parola stessa è un concetto chiave
            let w = word.to_lowercase();
            if w == "ciao" || w == "salve" || w == "buongiorno" || w == "buonasera" {
                // Questi sono IS_A "saluto" nel KG. Se il KG non li ha, li riconosciamo
                // direttamente come safety net. NON è una lista di tutti i saluti:
                // è un piccolo bootstrap per i saluti più comuni.
                greeting = true;
            }
        }
        (greeting, emotional, self_ref)
    } else {
        // Fallback senza KG (test): usa la KB come prima
        let relevant = knowledge_base.retrieve_for_delta(raw_words, frattale_delta);
        let s = relevant.iter().any(|e| e.domain == KnowledgeDomain::Social);
        let e = relevant.iter().any(|e| e.domain == KnowledgeDomain::Emotional);
        let i = relevant.iter().any(|e| e.domain == KnowledgeDomain::Self_);
        (s, e, i)
    };

    // ── Classificazione (ordine di priorità) ─────────────────────────────────
    let act = if has_greeting {
        InputAct::Greeting
    } else if has_question_mark && has_self_ref {
        InputAct::SelfQuery
    } else if has_question_mark {
        InputAct::Question
    } else if has_emotional {
        InputAct::EmotionalExpr
    } else {
        InputAct::Declaration
    };

    InputReading { act, intensity, salient_word }
}

#[cfg(test)]
mod tests {
    use super::*;

    // KnowledgeBase con ancore concettuali — specchia seed_conceptual_anchors() in engine.rs
    fn kb_with_anchors() -> KnowledgeBase {
        let mut kb = KnowledgeBase::new();
        kb.teach_concept(
            KnowledgeDomain::Social,
            "un saluto è un atto di avvicinamento sociale: chi saluta vuole connessione",
            "ciao",
            vec![63, 47], // ARMONIA + COMUNICAZIONE
        );
        kb.teach_concept(
            KnowledgeDomain::Emotional,
            "un'emozione è uno stato interno vissuto: chi esprime un'emozione cerca risonanza",
            "paura",
            vec![58, 33], // EMOZIONE + CORPO
        );
        kb.teach_concept(
            KnowledgeDomain::Self_,
            "un'indagine sull'identità è la domanda su chi è Prometeo, cosa sente, cosa vuole",
            "sei",
            vec![32], // IDENTITA
        );
        kb
    }

    fn empty_delta() -> Vec<(FractalId, f64)> { vec![] }
    fn lex() -> Lexicon { Lexicon::bootstrap() }

    #[test]
    fn test_greeting_via_word_match() {
        // "ciao" è la parola-campione del concetto Social → word_match in retrieve_for_delta
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["ciao".to_string()],
            "ciao",
            &empty_delta(),
            &kb,
            &lex,
            None,
        );
        assert_eq!(r.act, InputAct::Greeting);
    }

    #[test]
    fn test_greeting_via_fractal_delta() {
        // "salve" non è la parola-campione, ma attiva ARMONIA(63) → delta_match → Social
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(63u32, 0.15f64)]; // ARMONIA delta > 0.05
        let r = read_input(
            &["salve".to_string()],
            "salve",
            &delta,
            &kb,
            &lex,
            None,
        );
        assert_eq!(r.act, InputAct::Greeting,
            "salve dovrebbe essere riconosciuto come saluto via delta ARMONIA");
    }

    #[test]
    fn test_self_query_via_word_match() {
        // "sei" è la parola-campione del concetto Self_ + `?` → SelfQuery
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["chi".to_string(), "sei".to_string()],
            "chi sei?",
            &empty_delta(),
            &kb,
            &lex,
            None,
        );
        assert_eq!(r.act, InputAct::SelfQuery);
    }

    #[test]
    fn test_self_query_via_fractal_delta() {
        // Una domanda che attiva IDENTITA(32) → SelfQuery anche senza "sei"
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(32u32, 0.12f64)]; // IDENTITA delta
        let r = read_input(
            &["cosa".to_string(), "pensi".to_string()],
            "cosa pensi?",
            &delta,
            &kb,
            &lex,
            None,
        );
        assert_eq!(r.act, InputAct::SelfQuery,
            "domanda con delta IDENTITA → SelfQuery anche senza parola-campione");
    }

    #[test]
    fn test_generic_question() {
        // `?` senza Social/Self_/Emotional → Question generica
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["cosa".to_string(), "succede".to_string()],
            "cosa succede?",
            &empty_delta(),
            &kb,
            &lex,
            None,
        );
        assert_eq!(r.act, InputAct::Question);
    }

    #[test]
    fn test_emotional_expr_via_word_match() {
        // "paura" è la parola-campione del concetto Emotional → word_match → EmotionalExpr
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["ho".to_string(), "paura".to_string()],
            "ho paura",
            &empty_delta(),
            &kb,
            &lex,
            None,
        );
        assert_eq!(r.act, InputAct::EmotionalExpr);
    }

    #[test]
    fn test_emotional_expr_via_fractal_delta() {
        // EMOZIONE(58) delta → Emotional concept → EmotionalExpr
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(58u32, 0.45f64)]; // EMOZIONE delta
        let r = read_input(
            &["tristezza".to_string()],
            "sento tristezza",
            &delta,
            &kb,
            &lex,
            None,
        );
        assert_eq!(r.act, InputAct::EmotionalExpr,
            "qualunque parola che attiva EMOZIONE → EmotionalExpr");
    }

    #[test]
    fn test_declaration_default() {
        let lex = lex();
        let kb = kb_with_anchors();
        let r = read_input(
            &["penso".to_string(), "quindi".to_string(), "sono".to_string()],
            "penso quindi sono",
            &empty_delta(),
            &kb,
            &lex,
            None,
        );
        assert_eq!(r.act, InputAct::Declaration);
    }

    #[test]
    fn test_intensity_from_delta() {
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(58u32, 0.6f64), (32u32, 0.4f64), (33u32, 0.2f64)];
        let r = read_input(&[], "", &delta, &kb, &lex, None);
        // avg top-3 assoluti = (0.6 + 0.4 + 0.2) / 3 ≈ 0.4
        assert!((r.intensity - 0.4).abs() < 0.01,
            "intensity attesa ~0.4, ottenuta {}", r.intensity);
    }

    #[test]
    fn test_no_anchors_fallback() {
        // Senza ancore concettuali, solo `?` e Declaration funzionano
        let lex = lex();
        let kb = KnowledgeBase::new(); // vuota
        let r = read_input(&["ciao".to_string()], "ciao", &empty_delta(), &kb, &lex, None);
        // Senza ancora Social, "ciao" → Declaration (non riconosciuto)
        assert_eq!(r.act, InputAct::Declaration,
            "senza KnowledgeBase, ciao non è riconoscibile come saluto");
        let r2 = read_input(&["cosa".to_string()], "cosa succede?", &empty_delta(), &kb, &lex, None);
        assert_eq!(r2.act, InputAct::Question,
            "senza KB, `?` mantiene Question come fallback sintattico");
    }
}
