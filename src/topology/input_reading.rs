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

/// Soggetto del claim nell'input.
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimAgent {
    /// "io sono/ho/sento X" — il parlante dichiara qualcosa di sé
    Speaker,
    /// "tu sei/hai/senti X" — il parlante dice qualcosa di Prometeo
    Entity,
}

/// Tipo di claim strutturale rilevato nell'input.
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimKind {
    /// "io sono X" / "tu sei X" — identità o appartenenza
    Identity,
    /// "io ho X" / "sento X" / "provo X" — possesso o stato interno
    Feeling,
    /// "io faccio/voglio/penso X" — azione o intenzione
    Action,
}

/// Un claim strutturale rilevato nell'input: chi dice cosa di chi.
///
/// "io sono triste" → SpeakerClaim { agent: Speaker, kind: Feeling, predicate: "triste" }
/// "tu sei bello"   → SpeakerClaim { agent: Entity, kind: Identity, predicate: "bello" }
/// "io voglio capire" → SpeakerClaim { agent: Speaker, kind: Action, predicate: "capire" }
///
/// Non è parsing sintattico completo — è pattern matching sui costrutti
/// più comuni dell'italiano che esprimono relazioni soggetto-predicato.
/// Robusto per frasi semplici; sufficiente per orientare la risposta.
#[derive(Debug, Clone)]
pub struct SpeakerClaim {
    pub agent: ClaimAgent,
    pub kind: ClaimKind,
    /// La parola che porta il predicato (la prima parola-contenuto dopo il verbo)
    pub predicate: String,
}

/// Lettura strutturata dell'input corrente.
#[derive(Debug, Clone)]
pub struct InputReading {
    pub act: InputAct,
    /// Intensità dell'atto comunicativo (0..1) — media top-3 delta frattali assoluti
    pub intensity: f64,
    /// Parola più stabile dell'input (se presente nel lessico)
    pub salient_word: Option<String>,
    /// Claim strutturale rilevato (se presente).
    /// "io sono triste" → Some(SpeakerClaim { Speaker, Feeling, "triste" })
    /// "il cane abbaia" → None (nessun soggetto grammaticale rilevante)
    pub speaker_claim: Option<SpeakerClaim>,
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

    // ── Rilevamento claim strutturale soggetto/predicato ────────────────────
    // Rileva "io sono/ho/sento X" e "tu sei/hai X" come claim espliciti.
    // Non è parsing completo: è pattern matching sui costrutti più frequenti.
    // Robusto per frasi semplici; sufficiente per orientare stance e risposta.
    let speaker_claim = detect_speaker_claim(raw_words, lexicon, kg);

    // ── Classificazione (ordine di priorità) ─────────────────────────────────
    // I claim emozionali del parlante ("io sono triste") elevano Declaration
    // a EmotionalExpr anche se il KG non ha riconosciuto la parola via IS_A.
    // Questo permette risposte empatiche a stati interni espressi strutturalmente.
    let has_emotional_claim = speaker_claim.as_ref()
        .map(|c| c.agent == ClaimAgent::Speaker && c.kind == ClaimKind::Feeling)
        .unwrap_or(false);

    let act = if has_greeting {
        InputAct::Greeting
    } else if has_question_mark && has_self_ref {
        InputAct::SelfQuery
    } else if has_question_mark {
        InputAct::Question
    } else if has_emotional || has_emotional_claim {
        InputAct::EmotionalExpr
    } else {
        InputAct::Declaration
    };

    InputReading { act, intensity, salient_word, speaker_claim }
}

/// Rileva un claim strutturale soggetto/predicato nell'input.
///
/// Pattern riconosciuti (ordine di ricerca: io/tu → verbo → predicato):
///   "io sono/ero/mi sento/sento/provo/ho X"  → Speaker claim
///   "tu sei/eri/hai/senti X"                 → Entity claim
///   "io voglio/penso/credo X"                → Speaker Action
///
/// Restituisce None se nessun pattern viene riconosciuto.
pub fn detect_speaker_claim(
    raw_words: &[String],
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
) -> Option<SpeakerClaim> {
    if raw_words.is_empty() { return None; }

    let words_lc: Vec<&str> = raw_words.iter().map(|w| w.as_str()).collect();

    // Verbi copulativi/di stato per il parlante (1a persona)
    const SPEAKER_IDENTITY: &[&str] = &["sono", "ero", "sarò"];
    const SPEAKER_FEELING:  &[&str] = &["sento", "provo", "ho", "mi", "sto"];
    const SPEAKER_ACTION:   &[&str] = &["voglio", "penso", "credo", "so", "capisco", "devo"];

    // Verbi per l'entità (2a persona)
    const ENTITY_VERBS: &[&str] = &["sei", "eri", "sarai", "hai", "senti", "provi"];

    // Cerca "io" come soggetto esplicito
    let has_io = words_lc.iter().any(|&w| w == "io");
    // Cerca "mi" (mi sento, mi chiamo) — soggetto implicito
    let has_mi = words_lc.iter().any(|&w| w == "mi");
    // Cerca "tu" o "ti" come riferimento all'entità
    let has_tu = words_lc.iter().any(|&w| w == "tu" || w == "ti");
    // Cerca verbi di prima persona che implicano il soggetto parlante senza pronome esplicito.
    // In italiano "ho paura" è inequivocabilmente prima persona — "ho" non esiste in 3a persona.
    // Questi verbi sono sufficienti come indicatori del soggetto senza "io".
    const IMPLICIT_SPEAKER_VERBS: &[&str] = &["ho", "sono", "sto", "voglio", "penso", "credo", "sento", "provo", "devo", "so"];
    let has_implicit_speaker = !has_io && !has_mi && !has_tu
        && words_lc.iter().any(|&w| IMPLICIT_SPEAKER_VERBS.contains(&w));

    if !has_io && !has_mi && !has_tu && !has_implicit_speaker {
        return None;
    }

    // Trova la posizione del verbo e classifica il claim
    let (agent, kind, verb_pos) = if has_io || has_mi || has_implicit_speaker {
        // Cerca verbi del parlante
        if let Some(pos) = words_lc.iter().position(|&w| SPEAKER_IDENTITY.contains(&w)) {
            (ClaimAgent::Speaker, ClaimKind::Identity, pos)
        } else if let Some(pos) = words_lc.iter().position(|&w| SPEAKER_FEELING.contains(&w)) {
            (ClaimAgent::Speaker, ClaimKind::Feeling, pos)
        } else if let Some(pos) = words_lc.iter().position(|&w| SPEAKER_ACTION.contains(&w)) {
            (ClaimAgent::Speaker, ClaimKind::Action, pos)
        } else if has_implicit_speaker {
            // Verbo implicito speaker ma non in nessuna lista → usa posizione del verbo implicito
            if let Some(pos) = words_lc.iter().position(|&w| IMPLICIT_SPEAKER_VERBS.contains(&w)) {
                (ClaimAgent::Speaker, ClaimKind::Feeling, pos)
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        // has_tu: cerca verbi dell'entità
        if let Some(pos) = words_lc.iter().position(|&w| ENTITY_VERBS.contains(&w)) {
            (ClaimAgent::Entity, ClaimKind::Identity, pos)
        } else {
            return None;
        }
    };

    // Prendi la prima parola-contenuto dopo il verbo come predicato
    let predicate = words_lc.iter()
        .skip(verb_pos + 1)
        .find(|&&w| {
            // Salta articoli e parole funzionali brevi
            !matches!(w, "un" | "una" | "uno" | "il" | "la" | "lo" | "i" | "gli" | "le"
                         | "del" | "della" | "di" | "a" | "da" | "in" | "per" | "e")
        })
        .map(|&w| w.to_string());

    let predicate = predicate?;

    // Raffina il kind in base al predicato: se il predicato è nel dominio
    // emozionale (KG o lista), upgraded a Feeling indipendentemente dal verbo.
    // "io sono triste" (Identity verbo + emozione) → kind = Feeling
    let final_kind = if kind == ClaimKind::Identity {
        let is_emotional = is_emotional_word(&predicate, lexicon, kg);
        if is_emotional { ClaimKind::Feeling } else { kind }
    } else {
        kind
    };

    Some(SpeakerClaim { agent, kind: final_kind, predicate })
}

/// Verifica se una parola è nel dominio emozionale.
/// Usa KG (IS_A "emozione") se disponibile, poi lista diretta come fallback.
fn is_emotional_word(word: &str, lexicon: &Lexicon, kg: Option<&KnowledgeGraph>) -> bool {
    // Lista diretta di parole emozionali comuni — bootstrap per parole non ancora nel KG
    const EMOTIONAL_WORDS: &[&str] = &[
        "triste", "tristezza", "felice", "felicità", "arrabbiato", "arrabbiata",
        "paura", "spaventato", "spaventata", "ansioso", "ansiosa", "ansia",
        "gioioso", "gioiosa", "contento", "contenta", "depresso", "depressa",
        "solo", "sola", "solitudine", "amato", "amata", "odiato", "odiata",
        "stanco", "stanca", "stanchezza", "eccitato", "eccitata", "nervoso", "nervosa",
        "calmo", "calma", "sereno", "serena", "preoccupato", "preoccupata",
        "deluso", "delusa", "sorpreso", "sorpresa", "confuso", "confusa",
        "bene", "male", "meglio", "peggio", "vuoto", "pieno", "perso", "persa",
    ];

    if EMOTIONAL_WORDS.contains(&word) { return true; }

    // Check KG: word IS_A emozione/sentimento?
    if let Some(kg) = kg {
        let parents = kg.query_objects(word, RelationType::IsA);
        for p in parents {
            let p_lc = p.to_lowercase();
            if p_lc == "emozione" || p_lc == "sentimento" || p_lc == "stato_d_animo"
                || p_lc == "sensazione" || p_lc == "affetto" || p_lc == "umore" {
                return true;
            }
        }
    }

    // Ultima risorsa: il lessico sa che è un aggettivo (emozioni sono spesso aggettivi)?
    // Solo se ha POS = Adjective E fa parte del lessico con stabilità > 0
    if let Some(pat) = lexicon.get(word) {
        if matches!(pat.pos, Some(crate::topology::grammar::PartOfSpeech::Adjective))
            && pat.stability > 0.3 {
            // Gli aggettivi stabili nel lessico che hanno POS Adjective probabilmente
            // descrivono stati — ma è un'euristica debole, non usarla come unico segnale
            // Controllo dimensionale: alta Valenza (dim 7) = parola valutativa
            if pat.signature.get(crate::topology::primitive::Dim::Valenza) > 0.6 {
                return true;
            }
        }
    }

    false
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
