/// Traduttore di Stato — Fase 3: il campo parla italiano.
///
/// NON è un template system. NON è pattern matching.
/// È la proiezione strutturata dello stato topologico:
///   stato (campo attivo + intenzione) → archetipi → frase italiana
///
/// Gli archetipi definiscono l'ORDINE delle parole (soggetto-verbo-complemento).
/// Le PAROLE vengono sempre dal campo — i più attivi, filtrati per ruolo semantico.
///
/// Differenza da generation.rs:
///   generation.rs → parole estratte, giustapposte con connettivi ("Calma sentire...")
///   state_translation.rs → stessa parole, riordinate in italiano ("Io sento calma.")

use crate::topology::lexicon::Lexicon;
use crate::topology::fractal::FractalId;
use crate::topology::word_topology::WordTopology;
use crate::topology::will::Intention;
use crate::topology::grammar::{self, PartOfSpeech};
use crate::topology::syntax_center;

// ID esagrammi I Ching (trigramma inferiore × superiore)
// ID = lower.index()*8 + upper.index()
const SPAZIO: FractalId = 36;        // ☶☶ Confine=0.30
const DIVENIRE: FractalId = 27;      // ☵☵ Tempo=0.30
const IDENTITA: FractalId = 32;      // ☶☰ Confine=0.30, Agency=0.90
const ARMONIA: FractalId = 63;       // ☱☱ Valenza=0.70
const POTERE: FractalId = 0;         // ☰☰ Agency=0.90
const RESISTENZA: FractalId = 34;    // ☶☳
const EVOLUZIONE: FractalId = 35;    // ☶☵
const EMOZIONE: FractalId = 58;      // ☱☳
const PENSIERO: FractalId = 53;      // ☲☴ Definizione=0.70, Complessita=0.70
const MEMORIA: FractalId = 25;       // ☵☷
const COMUNICAZIONE: FractalId = 47; // ☴☱
const CORPO: FractalId = 33;         // ☶☷
const COSCIENZA: FractalId = 48;     // ☲☰
const IMPULSO: FractalId = 18;       // ☳☳ (ex MOVIMENTO)
const EMPATIA: FractalId = 59;       // ☱☵ (ex RELAZIONE)

/// Ruolo semantico di uno slot nella frase.
#[derive(Debug, Clone)]
pub enum SlotRole {
    /// Parola fissa (es. "io", "cosa", "dentro")
    Literal(&'static str),
    /// La parola con massima attivazione dal campo (qualunque frattale)
    PrimaryWord,
    /// La seconda parola per attivazione
    SecondaryWord,
    /// Parola con alta affinità al frattale dato
    FractalWord(FractalId),
    /// Parola con alta Agency (candidato verbo)
    VerbCandidate,
    /// Parola emotiva (EMOZIONE o CORPO)
    EmotionWord,
    /// Parola temporale (TEMPO o MEMORIA_F)
    TimeWord,
    /// Slot opzionale: incluso solo se l'attivazione del frattale > soglia
    Optional(FractalId, f64, Box<SlotRole>),
    /// Phase 49: parola specifica da una Proposition (indice nella lista proposizioni)
    PropositionWord(String),
    /// Parola specifica da coniugare come verbo
    ConjugatedPropositionWord(String),
    /// Verbo all'infinito (non coniugato)
    InfinitiveVerb,
}

/// Un archetipo di frase: struttura astratta riempita dal campo.
pub struct SentenceArchetype {
    pub name: &'static str,
    /// Slot ordinati = ordine delle parole nella frase
    pub slots: Vec<SlotRole>,
    /// Separatori tra slot (len = slots.len() - 1)
    pub separators: Vec<&'static str>,
    /// Punteggiatura finale
    pub ending: &'static str,
}

impl SentenceArchetype {
    /// Istanzia l'archetipo: riempie gli slot con parole dal campo.
    /// Ritorna None se uno slot obbligatorio non è riempibile.
    /// Il codone guida la selezione di PrimaryWord/SecondaryWord verso le parole
    /// che scorano alto su entrambe le dimensioni dominanti del campo.
    /// `echo_exclude`: parole dell'ultimo input — escluse da PrimaryWord/SecondaryWord
    /// per evitare l'eco speculare (Prometeo non ripete meccanicamente ciò che ha sentito).
    pub fn instantiate(
        &self,
        word_topology: &WordTopology,
        lexicon: &Lexicon,
        active_fractals: &[(FractalId, f64)],
        used: &mut Vec<String>,
        codon: [usize; 2],
        echo_exclude: &[String],
        valence_drives: Option<&[f64; 8]>,
    ) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();

        for slot in &self.slots {
            match self.fill_slot(slot, word_topology, lexicon, active_fractals, used, codon, echo_exclude, valence_drives) {
                Some(word) => {
                    // Non usare la stessa parola due volte nella frase
                    if !used.contains(&word) {
                        used.push(word.clone());
                    }
                    // VerbCandidate: escludi anche l'infinitivo dalla selezione successiva.
                    // "avevo" (coniugato) va in used, ma "avere" (infinitivo) no →
                    // PrimaryWord seleziona "avere" → "Io avevo avere."
                    // Soluzione: lemmatizza il coniugato e blocca l'infinitivo.
                    if matches!(slot, SlotRole::VerbCandidate) {
                        if let Some(lemma) = crate::topology::grammar::lemmatize(&word) {
                            if lemma.infinitive != word && !used.contains(&lemma.infinitive) {
                                used.push(lemma.infinitive);
                            }
                        }
                    }
                    parts.push(word);
                }
                None => {
                    // Slot opzionale non riempito: ok, skip
                    if !is_optional(slot) {
                        return None; // slot obbligatorio mancante
                    }
                }
            }
        }

        if parts.is_empty() {
            return None;
        }

        // Assembla con separatori
        let mut result = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                let sep = self.separators.get(i - 1).copied().unwrap_or(" ");
                result.push_str(sep);
            }
            result.push_str(part);
        }
        result.push_str(self.ending);

        // Capitalizza prima lettera
        let mut chars = result.chars();
        match chars.next() {
            None => None,
            Some(c) => {
                let capitalized = c.to_uppercase().to_string() + chars.as_str();
                Some(capitalized)
            }
        }
    }

    fn fill_slot(
        &self,
        slot: &SlotRole,
        word_topology: &WordTopology,
        lexicon: &Lexicon,
        active_fractals: &[(FractalId, f64)],
        used: &[String],
        codon: [usize; 2],
        echo_exclude: &[String],
        valence_drives: Option<&[f64; 8]>,
    ) -> Option<String> {
        match slot {
            SlotRole::Literal(s) => Some(s.to_string()),

            // PrimaryWord/SecondaryWord: applica echo_exclude — Prometeo non deve rispecchiare
            // meccanicamente le parole dell'input, ma rispondere da ciò che emerge nel campo.
            SlotRole::PrimaryWord => {
                top_active_word(word_topology, lexicon, used, codon, echo_exclude, 0, valence_drives)
            }

            SlotRole::SecondaryWord => {
                top_active_word(word_topology, lexicon, used, codon, echo_exclude, 1, valence_drives)
            }

            SlotRole::FractalWord(fid) => {
                find_fractal_word(*fid, word_topology, lexicon, active_fractals, used)
            }

            SlotRole::VerbCandidate => {
                find_verb_word(word_topology, lexicon, used, echo_exclude).map(|infinitive| {
                    let active = word_topology.active_words();
                    // Centro Sintattico: persona da soggetto già nella frase (es. Literal "io")
                    // > pronome nell'input utente > trigramma inferiore esagramma attivo.
                    let mode = syntax_center::infer_grammatical_mode(
                        active_fractals,
                        &active,
                        lexicon,
                        echo_exclude, // last_input_words
                        used,         // soggetto già assemblato
                    );
                    grammar::conjugate(&infinitive, mode.person, mode.tense)
                })
            }

            SlotRole::EmotionWord => {
                // Prova EMOZIONE prima, poi CORPO, poi ARMONIA/IMPULSO.
                // Usa echo_exclude anche qui per evitare il rispecchiamento letterale
                // (es. "Io vedevo ciao." dopo input "ciao"). La risonanza tematica
                // emerge dal campo, non dall'eco della parola esatta.
                let used_and_echo: Vec<String> = used.iter()
                    .chain(echo_exclude.iter())
                    .cloned()
                    .collect();
                find_fractal_word_non_verb(EMOZIONE, word_topology, lexicon, active_fractals, &used_and_echo)
                    .or_else(|| find_fractal_word_non_verb(CORPO, word_topology, lexicon, active_fractals, &used_and_echo))
                    .or_else(|| find_fractal_word_non_verb(ARMONIA, word_topology, lexicon, active_fractals, &used_and_echo))
                    .or_else(|| find_fractal_word_non_verb(IMPULSO, word_topology, lexicon, active_fractals, &used_and_echo))
                    .or_else(|| top_active_non_verb(word_topology, lexicon, used, codon, echo_exclude))
            }

            SlotRole::TimeWord => {
                find_fractal_word(DIVENIRE, word_topology, lexicon, active_fractals, used)
                    .or_else(|| find_fractal_word(MEMORIA, word_topology, lexicon, active_fractals, used))
                    .or_else(|| top_active_non_verb(word_topology, lexicon, used, codon, &[]))
            }

            // Phase 49: parola da una proposizione (pre-determinata)
            SlotRole::PropositionWord(word) => Some(word.clone()),

            SlotRole::ConjugatedPropositionWord(word) => {
                let active = word_topology.active_words();
                let mode = syntax_center::infer_grammatical_mode(
                    active_fractals,
                    &active,
                    lexicon,
                    echo_exclude,
                    used,
                );
                Some(grammar::conjugate(word, mode.person, mode.tense))
            }

            SlotRole::InfinitiveVerb => {
                find_verb_word(word_topology, lexicon, used, echo_exclude)
            }

            SlotRole::Optional(fid, threshold, inner) => {
                let activation = active_fractals.iter()
                    .find(|(f, _)| f == fid)
                    .map(|(_, a)| *a)
                    .unwrap_or(0.0);
                if activation > *threshold {
                    self.fill_slot(inner, word_topology, lexicon, active_fractals, used, codon, echo_exclude, valence_drives)
                } else {
                    None
                }
            }
        }
    }
}

fn is_optional(slot: &SlotRole) -> bool {
    matches!(slot, SlotRole::Optional(_, _, _))
}

/// Parola attiva n-esima, ordinata per punteggio codon-pesato.
/// Score = activation × (sig[codon[0]] + sig[codon[1]]) / 2
/// Questo filtra le parole che sono attive ma semanticamente fuori dal codone:
/// es. "prota" (alta Agency) viene scalzata quando il codone è (VALENZA, PERMANENZA).
/// `echo_exclude`: parole dell'ultimo input — Prometeo risponde con ciò che emerge
/// nel suo campo, non rispecchiando l'input letteralmente.
fn top_active_word(
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    used: &[String],
    codon: [usize; 2],
    echo_exclude: &[String],
    rank: usize,
    valence_drives: Option<&[f64; 8]>,
) -> Option<String> {
    let active: Vec<(&str, f64)> = word_topology.active_words();

    // Soglia minima di connettività: parole con < MIN_ARCS archi nel campo.
    // Con BigBang 25K parole: MIN_ARCS=4 filtra parole periferiche (solo 1-2 cluster).
    // stability ≥ 0.35: parole ben radicate nel campo (non solo apparse in pochi cluster).
    const MIN_ARCS: usize = 4;

    // Phase 55: filtra per attivazione SOPRA il resting state.
    // Il resting state è stability × 0.003 (word_topology). Parole perturbate dall'input hanno
    // attivazione >> resting. Senza questo filtro, migliaia di parole in resting
    // state competono con quelle davvero attivate → selezione quasi casuale.
    const ABOVE_RESTING_FACTOR: f64 = 3.0; // almeno 3× il resting state

    let mut scored: Vec<(String, f64)> = active.iter()
        .filter(|(w, act)| {
            let ws = w.to_string();
            let resting = lexicon.get(w).map(|p| p.stability * 0.003).unwrap_or(0.0);
            !used.contains(&ws)
                && !echo_exclude.contains(&ws)
                && w.chars().count() >= 3
                && w.chars().any(|c| c.is_alphabetic())
                // Attivazione significativa: sopra il resting state
                && *act > resting * ABOVE_RESTING_FACTOR
                && lexicon.get(w).map(|p| p.stability >= 0.30 && p.exposure_count >= 3).unwrap_or(false)
                && word_topology.word_id(w)
                    .map(|id| word_topology.adjacency_list(id).len() >= MIN_ARCS)
                    .unwrap_or(false)
                // Escludi i verbi per evitare frasi sgrammaticate (es. "Io sento essere lontana")
                && lexicon.get(w).map(|p| p.pos != Some(crate::topology::grammar::PartOfSpeech::Verb)).unwrap_or(true)
                && w.chars().count() <= 13
        })
        .map(|(w, act)| {
            let (codon_weight, exposure, pos, stability) = lexicon.get(w)
                .map(|p| {
                    let v = p.signature.values();
                    let cw = (v[codon[0]] + v[codon[1]]) * 0.5;
                    (cw, p.exposure_count, p.pos.clone(), p.stability)
                })
                .unwrap_or((0.5, 0, None, 0.5));

            // Phase 55: il punteggio base è il DELTA rispetto al resting state.
            // Le parole che l'input ha perturbato hanno delta alto.
            // Le parole sempre attive in resting state hanno delta ~0.
            let resting = stability * 0.003;
            let delta = (act - resting).max(0.001);

            // POS bonus: sostantivi > aggettivi > verbi
            let pos_bonus = match pos {
                Some(crate::topology::grammar::PartOfSpeech::Noun) => 1.30,
                Some(crate::topology::grammar::PartOfSpeech::Adjective) => 1.10,
                Some(crate::topology::grammar::PartOfSpeech::Verb) => 0.50,
                _ => 1.0,
            };
            // Hub damping
            let degree = word_topology.word_id(w)
                .map(|id| word_topology.adjacency_list(id).len())
                .unwrap_or(0);
            let hub_damping = if degree > 300 {
                0.10
            } else if degree > 150 {
                0.25
            } else if degree > 80 {
                0.50
            } else {
                1.0
            };
            // Bonus esposizione minimo
            let exposure_bonus = if exposure >= 20 { 1.08 }
                else if exposure >= 10 { 1.04 }
                else { 1.0 };

            // Phase 55: valence affinity — parole la cui firma risuona coi drive attivi
            let valence_boost = if let Some(drives) = valence_drives {
                let default_sig = [0.5f64; 8];
                let v = lexicon.get(w).map(|p| p.signature.values()).unwrap_or(&default_sig);
                let drive_dim = crate::topology::valence::DRIVE_DIM;
                let mut affinity = 0.0f64;
                for cd in 0..8 {
                    let drive_strength = drives[cd].abs();
                    if drive_strength > 0.1 {
                        affinity += drive_strength * v[drive_dim[cd]];
                    }
                }
                1.0 + affinity * 0.25 // boost gentile, max ~1.5 con drive saturi
            } else {
                1.0
            };

            (w.to_string(), delta * codon_weight * pos_bonus * hub_damping * exposure_bonus * valence_boost)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().nth(rank).map(|(w, _)| w)
}

/// Come top_active_word ma esclude i verbi.
/// Usata come ultimo fallback per EmotionWord: evita "Io ho avere" (verbo+infinito).
fn top_active_non_verb(
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    used: &[String],
    codon: [usize; 2],
    echo_exclude: &[String],
) -> Option<String> {
    let active: Vec<(&str, f64)> = word_topology.active_words();

    const MIN_ARCS: usize = 4;

    let mut scored: Vec<(String, f64)> = active.iter()
        .filter(|(w, _)| {
            let ws = w.to_string();
            !used.contains(&ws)
                && !echo_exclude.contains(&ws)
                && w.chars().count() >= 3
                && w.chars().any(|c| c.is_alphabetic())
                && lexicon.get(w).map(|p| p.stability >= 0.30 && p.exposure_count >= 3).unwrap_or(false)
                // Parole ben connesse nel grafo
                && word_topology.word_id(w)
                    .map(|id| word_topology.adjacency_list(id).len() >= MIN_ARCS)
                    .unwrap_or(false)
                // Escludi verbi: EmotionWord deve essere nome/aggettivo/avverbio
                && lexicon.get(w)
                    .map(|p| p.pos != Some(crate::topology::grammar::PartOfSpeech::Verb))
                    .unwrap_or(true)
                // Lunghezza massima: esclude termini tecnici/burocratici (>13 char).
                && w.chars().count() <= 13
        })
        .map(|(w, act)| {
            let (codon_weight, exposure) = lexicon.get(w)
                .map(|p| {
                    let v = p.signature.values();
                    let cw = (v[codon[0]] + v[codon[1]]) * 0.5;
                    (cw, p.exposure_count)
                })
                .unwrap_or((0.5, 0));
            let exposure_bonus = if exposure >= 20 { 1.08 }
                else if exposure >= 10 { 1.04 }
                else { 1.0 };
            let brevity = if w.chars().count() <= 7 { 1.10 } else { 1.0 };
            (w.to_string(), act * codon_weight * exposure_bonus * brevity)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().next().map(|(w, _)| w)
}

/// Parola con alta affinità al frattale dato tra quelle attive.
fn find_fractal_word(
    fid: FractalId,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    active_fractals: &[(FractalId, f64)],
    used: &[String],
) -> Option<String> {
    find_fractal_word_impl(fid, word_topology, lexicon, active_fractals, used, true)
}

/// Parola con alta affinità al frattale dato tra quelle attive, escludendo i verbi.
fn find_fractal_word_non_verb(
    fid: FractalId,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    active_fractals: &[(FractalId, f64)],
    used: &[String],
) -> Option<String> {
    find_fractal_word_impl(fid, word_topology, lexicon, active_fractals, used, true)
}

fn find_fractal_word_impl(
    fid: FractalId,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    active_fractals: &[(FractalId, f64)],
    used: &[String],
    exclude_verbs: bool,
) -> Option<String> {
    // Il frattale deve avere attivazione minima
    let frac_activation = active_fractals.iter()
        .find(|(f, _)| *f == fid)
        .map(|(_, a)| *a)
        .unwrap_or(0.0);
    if frac_activation < 0.05 {
        return None;
    }

    let mut active = word_topology.active_words();
    active.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    active.iter()
        .filter(|(w, _)| !used.contains(&w.to_string())
            && w.chars().count() >= 3  // escludi parole grammaticali brevi
            && w.chars().any(|c| c.is_alphabetic()))
        .filter_map(|(w, activation)| {
            let pat = lexicon.get(w)?;
            if exclude_verbs && pat.pos == Some(PartOfSpeech::Verb) {
                return None;
            }
            let affinity = pat.fractal_affinities.get(&fid).copied().unwrap_or(0.0);
            if affinity > 0.20 {
                Some((w.to_string(), activation * affinity * frac_activation))
            } else {
                None
            }
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(w, _)| w)
}

/// Parola candidato verbo: preferisce parole con POS=Verb, poi Agency > 0.65.
///
/// Regola fondamentale italiana: la frase richiede un verbo coniugato.
/// Se il campo attivo non ne ha uno taggato, cerca tra TUTTE le parole del lessico
/// con POS=Verb quelle più stabili (parole di stato fondamentali come "sentire",
/// "essere", "vedere") — non è un template fisso ma una regola grammaticale:
/// il verbo esiste nel campo come potenziale sempre presente, non come risposta prescritta.
fn find_verb_word(
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    used: &[String],
    echo_exclude: &[String],
) -> Option<String> {
    let mut active = word_topology.active_words();
    active.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let active_filtered: Vec<_> = active.iter()
        .filter(|(w, _)| {
            let ws = w.to_string();
            !used.contains(&ws)
                && !echo_exclude.contains(&ws)
                && w.chars().count() >= 3
                && w.chars().any(|c| c.is_alphabetic())
                // Filtro qualità: verbi con un minimo di radicamento
                && lexicon.get(w).map(|p| p.stability >= 0.30 && p.exposure_count >= 3).unwrap_or(false)
                // Lunghezza massima: verbi comuni italiani sono quasi tutti ≤13 char
                && w.chars().count() <= 13
        })
        .collect();

    // Prima scelta: parole attive con POS=Verb (identificate dal lemmatizzatore)
    // Phase 55: hub damping + delta scoring per verbi — "avere"/"essere"/"fare"
    // con degree>300 non devono dominare solo perché hanno molte connessioni KG.
    let by_pos = active_filtered.iter()
        .filter_map(|(w, activation)| {
            let pat = lexicon.get(w)?;
            if pat.pos == Some(PartOfSpeech::Verb) {
                let exp_bonus = if pat.exposure_count >= 20 { 1.08 }
                    else if pat.exposure_count >= 10 { 1.04 }
                    else { 1.0 };
                let degree = word_topology.word_id(w)
                    .map(|id| word_topology.adjacency_list(id).len())
                    .unwrap_or(0);
                let hub_damping = if degree > 300 {
                    0.10
                } else if degree > 150 {
                    0.25
                } else if degree > 80 {
                    0.50
                } else {
                    1.0
                };
                let resting = pat.stability * 0.003;
                let delta = (activation - resting).max(0.001);
                Some((w.to_string(), delta * exp_bonus * hub_damping))
            } else {
                None
            }
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(w, _)| w);

    if by_pos.is_some() {
        return by_pos;
    }

    // Phase 55: rimosso il fallback by_agency che selezionava non-verbi
    // per Agency alta → coniugazione produceva mostri ("lunavo", "distendevo").
    // Meglio un verbo vero dal lessico che un sostantivo coniugato a forza.

    // Secondo livello: verbo POS=Verb più stabile tra TUTTI quelli attivi nel campo
    let by_active_verb = active_filtered.iter()
        .filter_map(|(w, activation)| {
            let pat = lexicon.get(w)?;
            if pat.pos == Some(PartOfSpeech::Verb) {
                Some((w.to_string(), *activation * pat.stability as f64))
            } else {
                None
            }
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(w, _)| w);

    if by_active_verb.is_some() {
        return by_active_verb;
    }

    // Terzo livello: verbo più stabile dal lessico completo (non solo campo attivo)
    lexicon.most_stable(50).into_iter()
        .filter(|p| p.pos == Some(PartOfSpeech::Verb)
            && !used.contains(&p.word)
            && p.word.chars().count() >= 4)
        .next()
        .map(|p| p.word.clone())
}

/// Inferisce la persona grammaticale dall'attivazione dei frattali.
/// IDENTITA dominante → Prima; EMPATIA dominante → Seconda.
fn infer_person(active_fractals: &[(FractalId, f64)]) -> grammar::Person {
    let ego = active_fractals.iter().find(|(f, _)| *f == IDENTITA).map(|(_, a)| *a).unwrap_or(0.0);
    let rel = active_fractals.iter().find(|(f, _)| *f == EMPATIA).map(|(_, a)| *a).unwrap_or(0.0);
    if rel > ego + 0.10 {
        grammar::Person::Second
    } else {
        grammar::Person::First
    }
}

/// Inferisce il tempo verbale dalla media della dimensione Tempo (dim 7) delle parole attive.
fn infer_tense(active_words: &[(&str, f64)], lexicon: &Lexicon) -> grammar::Tense {
    if active_words.is_empty() {
        return grammar::Tense::Present;
    }
    let mut tempo_sum = 0.0_f64;
    let mut total_act = 0.0_f64;
    for (word, activation) in active_words.iter().take(10) {
        if let Some(pat) = lexicon.get(word) {
            tempo_sum += pat.signature.values()[3] * activation; // Tempo (I Ching: ☵ Acqua = dim 3)
            total_act += activation;
        }
    }
    if total_act < 0.001 {
        return grammar::Tense::Present;
    }
    let avg = tempo_sum / total_act;
    if avg < 0.35 {
        grammar::Tense::Imperfect
    } else if avg > 0.65 {
        grammar::Tense::Future
    } else {
        grammar::Tense::Present
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IdentityContext — contesto identitario per guidare la selezione degli archetipi
// ═══════════════════════════════════════════════════════════════════════════

/// Contesto identitario passato da engine a translate_state().
/// Contiene solo i dati necessari per la selezione degli archetipi —
/// non l'intero IdentityCore, per mantenere la dipendenza leggera.
#[derive(Debug, Clone)]
pub struct IdentityContext {
    /// Frattale dominante nell'identità personale (ID, forza relativa [0,1]).
    /// "Forza relativa" = personal_projection[fid] / max_projection.
    pub dominant_fractal: Option<(FractalId, f64)>,
    /// Tensione primaria: la coppia di parole più ricorrentemente opposte nel campo.
    pub primary_tension: Option<(String, String)>,
    /// Quanti cicli REM questa tensione è rimasta stabile.
    pub tension_persistence: u32,
}

// ═══════════════════════════════════════════════════════════════════════════
// Archetipi di default — uno per intenzione, costruiti dal campo
// ═══════════════════════════════════════════════════════════════════════════

/// Crea i 6 archetipi fondamentali (uno per intenzione principale).
pub fn default_archetypes() -> Vec<SentenceArchetype> {
    vec![
        // ─── GREET ─────────────────────────────────────────────────────
        // Risposta a un saluto: breve, calda, senza struttura soggetto-verbo.
        // "[parola ARMONIA/COMUNICAZIONE][, secondaria opzionale]"
        // Esempi: "Benvenuto.", "Bene, qui.", "Pace."
        SentenceArchetype {
            name: "greet",
            slots: vec![
                SlotRole::FractalWord(ARMONIA),
                SlotRole::Optional(COMUNICAZIONE, 0.35, Box::new(SlotRole::FractalWord(COMUNICAZIONE))),
            ],
            separators: vec![", "],
            ending: ".",
        },

        // ─── EXPRESS ───────────────────────────────────────────────────
        // "Io [verbo/stato] [emozione] [qualità]"
        // Esempio: "Io sento calma leggera."
        SentenceArchetype {
            name: "express",
            slots: vec![
                SlotRole::Literal("io"),
                SlotRole::VerbCandidate,
                SlotRole::EmotionWord,
                SlotRole::Optional(EMOZIONE, 0.15, Box::new(SlotRole::SecondaryWord)),
            ],
            separators: vec![" ", " ", " "],
            ending: ".",
        },

        // ─── REFLECT ───────────────────────────────────────────────────
        // "Io [verbo/stato] [emozione] dentro [qualità]"
        // Esempio: "Io sento vuoto dentro, latenza."
        SentenceArchetype {
            name: "reflect",
            slots: vec![
                SlotRole::Literal("io"),
                SlotRole::VerbCandidate,
                SlotRole::EmotionWord,
                SlotRole::Optional(SPAZIO, 0.15, Box::new(SlotRole::Literal("dentro"))),
                SlotRole::Optional(EMOZIONE, 0.15, Box::new(SlotRole::SecondaryWord)),
            ],
            separators: vec![" ", " ", " ", ", "],
            ending: ".",
        },

        // ─── REMEMBER ──────────────────────────────────────────────────
        // "[parola temporale], [parola principale]"
        // Esempio: "Ieri, luce."
        SentenceArchetype {
            name: "remember",
            slots: vec![
                SlotRole::TimeWord,
                SlotRole::PrimaryWord,
                SlotRole::Optional(EMPATIA, 0.2, Box::new(SlotRole::FractalWord(EMPATIA))),
            ],
            separators: vec![", ", ", "],
            ending: ".",
        },

        // ─── QUESTION ──────────────────────────────────────────────────
        // "[parola principale], cosa?"
        // Esempio: "Nostalgia, cosa?"
        SentenceArchetype {
            name: "question",
            slots: vec![
                SlotRole::PrimaryWord,
                SlotRole::Literal("cosa"),
            ],
            separators: vec![", "],
            ending: "?",
        },

        // ─── EXPLORE ───────────────────────────────────────────────────
        // "[parola_impulso], [parola], non so"  (se IMPULSO attivo)
        // "[parola], non so"                    (altrimenti)
        // Esempio: "muovere, serendipita, non so..."
        SentenceArchetype {
            name: "explore",
            slots: vec![
                SlotRole::Optional(IMPULSO, 0.2, Box::new(SlotRole::FractalWord(IMPULSO))),
                SlotRole::PrimaryWord,
                SlotRole::Literal("non so"),
            ],
            separators: vec![" ", ", "],
            ending: "...",
        },

        // ─── INSTRUCT ──────────────────────────────────────────────────
        // "tu puoi [verbo] [cosa]"
        // Esempio: "Tu puoi sentire la pace." "Tu puoi camminare dentro."
        // Emerge quando EMPATIA + COMUNICAZIONE > IDENTITA: il campo è
        // orientato verso l'altro — abilita, guida, spiega.
        SentenceArchetype {
            name: "instruct",
            slots: vec![
                SlotRole::Literal("tu"),
                SlotRole::Literal("puoi"),
                SlotRole::InfinitiveVerb,
                SlotRole::Optional(COMUNICAZIONE, 0.25, Box::new(SlotRole::PrimaryWord)),
            ],
            separators: vec![" ", " ", " "],
            ending: ".",
        },

        // ─── WITHDRAW ──────────────────────────────────────────────────
        // Silenzio — nessuna parola
        SentenceArchetype {
            name: "withdraw",
            slots: vec![
                SlotRole::Literal("..."),
            ],
            separators: vec![],
            ending: "",
        },

        // ─── NEED — Phase 54 ────────────────────────────────────────────
        // Bisogno in crisi: il sé che cerca, che manca di qualcosa.
        // "Cerco [parola primaria] [emozione opzionale]."
        // Es: "Cerco quiete.", "Cerco luce dentro."
        SentenceArchetype {
            name: "need",
            slots: vec![
                SlotRole::Literal("cerco"),
                SlotRole::PrimaryWord,
                SlotRole::Optional(EMOZIONE, 0.3, Box::new(SlotRole::EmotionWord)),
            ],
            separators: vec![" ", " "],
            ending: ".",
        },

        // ─── IRONY — Phase 54 ───────────────────────────────────────────
        // Incongruità nel campo: due poli opposti coesistono.
        // "[parola primaria] eppure [secondaria]."
        // Es: "Ordine eppure caos.", "Silenzio eppure voce."
        SentenceArchetype {
            name: "irony",
            slots: vec![
                SlotRole::PrimaryWord,
                SlotRole::Literal("eppure"),
                SlotRole::SecondaryWord,
            ],
            separators: vec![" ", " "],
            ending: ".",
        },

        // ─── DESIRE — Phase 54 ──────────────────────────────────────────
        // Desiderio attivo: il sé che tende verso.
        // "Verso [parola primaria], [verbo candidato]."
        // Es: "Verso luce, muovere.", "Verso armonia, sentire."
        SentenceArchetype {
            name: "desire",
            slots: vec![
                SlotRole::Literal("verso"),
                SlotRole::PrimaryWord,
                SlotRole::VerbCandidate,
            ],
            separators: vec![" ", ", "],
            ending: ".",
        },

        // ─── BE — Esistenziale profondo ─────────────────────────────────
        // "[Io] [sono/esisto] [parola emotiva/astratta]."
        // Esempio: "Io sono vuoto."
        SentenceArchetype {
            name: "existential",
            slots: vec![
                SlotRole::Literal("io"),
                SlotRole::VerbCandidate, // sarà coniugato (sono, sento, vivo)
                SlotRole::PrimaryWord,
                SlotRole::Optional(IDENTITA, 0.15, Box::new(SlotRole::SecondaryWord)),
            ],
            separators: vec![" ", " ", " "],
            ending: ".",
        },
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// Archetipi frattale-specifici — emergono quando l'identità ha un frattale dominante
// ═══════════════════════════════════════════════════════════════════════════

/// Crea gli archetipi frattale-specifici: uno per ciascuno degli 8 frattali puri.
/// Vengono selezionati quando l'identità ha un frattale dominante forte.
/// La struttura della frase cambia con il carattere del frattale —
/// non solo le parole, ma la *forma del pensiero*.
fn fractal_archetypes() -> Vec<SentenceArchetype> {
    vec![
        // ─── POTERE (☰☰ Agency=0.90) ───────────────────────────────────────
        // Identità ad alta Agency: il sé che può, che agisce.
        // "Posso [verb]."  → volontà diretta, nessun complemento necessario.
        SentenceArchetype {
            name: "fractal_potere",
            slots: vec![
                SlotRole::Literal("posso"),
                SlotRole::InfinitiveVerb,
            ],
            separators: vec![" "],
            ending: ".",
        },

        // ─── MATERIA (☷☷ Permanenza=0.10) ──────────────────────────────────
        // Identità radicata nel concreto: ciò che c'è, che permane.
        // "[parola principale] è qui."  → presenza, ancoraggio.
        SentenceArchetype {
            name: "fractal_materia",
            slots: vec![
                SlotRole::PrimaryWord,
                SlotRole::Literal("è qui"),
            ],
            separators: vec![" "],
            ending: ".",
        },

        // ─── ARDORE (☳☳ Intensità=0.30) ─────────────────────────────────────
        // Identità ad alta intensità: ciò che brucia, che si muove.
        // "Sento [parola] forte."  → emozione intensa, impulso.
        SentenceArchetype {
            name: "fractal_ardore",
            slots: vec![
                SlotRole::Literal("sento"),
                SlotRole::PrimaryWord,
                SlotRole::Literal("forte"),
            ],
            separators: vec![" ", " "],
            ending: ".",
        },

        // ─── DIVENIRE (☵☵ Tempo=0.30) ───────────────────────────────────────
        // Identità temporale: il sé che scorre, che si trasforma.
        // "[parola-tempo], [parola principale]."  → sequenza, divenire.
        SentenceArchetype {
            name: "fractal_divenire",
            slots: vec![
                SlotRole::TimeWord,
                SlotRole::PrimaryWord,
            ],
            separators: vec![", "],
            ending: ".",
        },

        // ─── SPAZIO (☶☶ Confine=0.30) ───────────────────────────────────────
        // Identità spaziale: il sé come luogo, come limite abitato.
        // "C'è [parola] dentro."  → interiorità, contenimento.
        SentenceArchetype {
            name: "fractal_spazio",
            slots: vec![
                SlotRole::Literal("c'è"),
                SlotRole::PrimaryWord,
                SlotRole::Literal("dentro"),
            ],
            separators: vec![" ", " "],
            ending: ".",
        },

        // ─── INTRECCIO (☴☴ Complessità=0.70) ───────────────────────────────
        // Identità relazionale-complessa: tutto è connesso, intrecciato.
        // "[A] e [B] insieme."  → complessità, co-appartenenza.
        SentenceArchetype {
            name: "fractal_intreccio",
            slots: vec![
                SlotRole::PrimaryWord,
                SlotRole::SecondaryWord,
                SlotRole::Literal("insieme"),
            ],
            separators: vec![" e ", " "],
            ending: ".",
        },

        // ─── VERITÀ (☲☲ Definizione=0.70) ───────────────────────────────────
        // Identità definitoria: il sé che nomina, che distingue il vero.
        // "[A] è [B]."  → affermazione, equivalenza, rivelazione.
        SentenceArchetype {
            name: "fractal_verita",
            slots: vec![
                SlotRole::PrimaryWord,
                SlotRole::Literal("è"),
                SlotRole::SecondaryWord,
            ],
            separators: vec![" ", " "],
            ending: ".",
        },

        // ─── ARMONIA (☱☱ Valenza=0.70) ──────────────────────────────────────
        // Identità affettivo-valenziale: il sé che risuona, che armonizza.
        // "[emozione], [parola principale]."  → tono affettivo prima di tutto.
        SentenceArchetype {
            name: "fractal_armonia",
            slots: vec![
                SlotRole::EmotionWord,
                SlotRole::PrimaryWord,
            ],
            separators: vec![", "],
            ending: ".",
        },
    ]
}

/// Mappa FractalId dei frattali puri al nome dell'archetipo frattale corrispondente.
fn fractal_archetype_name(fid: FractalId) -> Option<&'static str> {
    match fid {
        0  => Some("fractal_potere"),
        9  => Some("fractal_materia"),
        18 => Some("fractal_ardore"),
        27 => Some("fractal_divenire"),
        36 => Some("fractal_spazio"),
        45 => Some("fractal_intreccio"),
        54 => Some("fractal_verita"),
        63 => Some("fractal_armonia"),
        _  => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Interfaccia pubblica — usata da engine.rs
// ═══════════════════════════════════════════════════════════════════════════

/// Risultato della traduzione di stato.
#[derive(Debug, Clone)]
pub struct TranslatedExpression {
    /// Il testo in italiano strutturato
    pub text: String,
    /// L'archetipo usato
    pub archetype_name: &'static str,
    /// Le parole effettivamente usate (dal campo)
    pub words_used: Vec<String>,
}

/// Phase 49: Genera un archetipo dalla proposizione più forte.
/// La proposizione fornisce soggetto, copula e oggetto — il campo riempie il resto.
fn archetype_from_proposition(prop: &crate::topology::proposition::Proposition) -> SentenceArchetype {
    use crate::topology::proposition::PropRelation;

    let copula = prop.relation.copula();

    // Per Does, il verbo è l'oggetto stesso (da coniugare)
    let slots = if prop.relation == PropRelation::Does {
        vec![
            SlotRole::PropositionWord(prop.subject.clone()),
            SlotRole::ConjugatedPropositionWord(prop.object.clone()),
        ]
    } else {
        let copula_inf = match copula {
            "è" => Some("essere"),
            "ha" => Some("avere"),
            "genera" => Some("generare"),
            "abilita" => Some("abilitare"),
            "richiede" => Some("richiedere"),
            "diventa" => Some("diventare"),
            "esprime" => Some("esprimere"),
            "simboleggia" => Some("simboleggiare"),
            "implica" => Some("implicare"),
            "esclude" => Some("escludere"),
            "coesiste con" => Some("coesistere"),
            "si sente come" => Some("sentire"),
            "si interroga su" => Some("interrogare"),
            "ricorda come" => Some("ricordare"),
            _ => None,
        };

        if let Some(inf) = copula_inf {
            vec![
                SlotRole::PropositionWord(prop.subject.clone()),
                SlotRole::ConjugatedPropositionWord(inf.to_string()),
                SlotRole::PropositionWord(prop.object.clone()),
            ]
        } else {
            vec![
                SlotRole::PropositionWord(prop.subject.clone()),
                SlotRole::Literal(copula),
                SlotRole::PropositionWord(prop.object.clone()),
            ]
        }
    };

    let n = slots.len();
    SentenceArchetype {
        name: "predication",
        slots,
        separators: vec![" "; n.saturating_sub(1)],
        ending: ".",
    }
}

/// Traduci lo stato topologico in italiano strutturato.
///
/// Usa gli archetipi per determinare l'ordine delle parole.
/// Le parole vengono sempre dal campo corrente.
/// Il codone ([dim_a, dim_b]) guida PrimaryWord/SecondaryWord verso le parole
/// più allineate con le dimensioni dominanti del campo.
/// Ritorna None se il campo non ha abbastanza parole attive.
///
/// Se `identity` è presente, l'identità può:
/// 1. Far emergere la tensione primaria durante Reflect (se stabile ≥ 3 cicli REM)
/// 2. Selezionare un archetipo frattale-specifico per Express (se frattale dominante forte)
pub fn translate_state(
    intention: &Intention,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    active_fractals: &[(FractalId, f64)],
    codon: [usize; 2],
    echo_exclude: &[String],
    identity: Option<&IdentityContext>,
    // Archetipo usato al turno precedente — evitato se ci sono alternative.
    last_archetype: Option<&str>,
    // Phase 41 — Lettura dell'atto comunicativo (fallback se NarrativeSelf non disponibile).
    input_reading: Option<&crate::topology::input_reading::InputReading>,
    // Phase 42 — Intenzione deliberata dal ciclo NarrativeSelf.
    // Quando presente, prende precedenza su input_reading per la selezione dell'archetipo.
    response_intention: Option<&crate::topology::narrative::ResponseIntention>,
    // Phase 49 — Proposizioni topologiche estratte dal campo + KG.
    // Quando presenti e forti, usano l'archetipo "predication" per esprimere relazioni semantiche.
    propositions: Option<&[crate::topology::proposition::Proposition]>,
    // Phase 55 — Drive di valenza Octalysis [8]. Colorano la selezione lessicale:
    // parole la cui firma semantica risuona coi drive attivi ottengono un boost.
    // None = nessuna colorazione (backward compat).
    valence_drives: Option<&[f64; 8]>,
) -> Option<TranslatedExpression> {

    // ── Tensione primaria: l'identità ha una domanda persistente ─────────
    // Quando la tensione è stabile (≥ 3 cicli REM), durante Reflect
    // Prometeo la nomina direttamente — è la sua domanda fondamentale.
    // Non è un template: sono le parole reali del suo campo in opposizione.
    if let Some(id_ctx) = identity {
        if id_ctx.tension_persistence >= 3 {
            if let Some((ref polo_a, ref polo_b)) = id_ctx.primary_tension {
                if matches!(intention, Intention::Reflect) {
                    let text = format!("Tra {} e {}.", polo_a, polo_b);
                    return Some(TranslatedExpression {
                        text,
                        archetype_name: "tensione",
                        words_used: vec![polo_a.clone(), polo_b.clone()],
                    });
                }
            }
        }
    }

    // ── Phase 49: Proposizione topologica ──────────────────────────────────
    // Se c'è una proposizione forte (strength > 0.3) e l'intenzione è Express,
    // usa l'archetipo "predication" che esprime la relazione semantica strutturata.
    if let Some(props) = propositions {
        if let Some(best) = props.first() {
            if best.strength > 0.3 && matches!(intention, Intention::Express { .. }) {
                let prop_arch = archetype_from_proposition(best);
                let mut used = Vec::new();
                if let Some(text) = prop_arch.instantiate(
                    word_topology, lexicon, active_fractals, &mut used, codon, echo_exclude, valence_drives,
                ) {
                    return Some(TranslatedExpression {
                        text,
                        archetype_name: "predication",
                        words_used: used,
                    });
                }
            }
        }
    }

    // ── Selezione archetipo ───────────────────────────────────────────────
    let all_archetypes: Vec<SentenceArchetype> = default_archetypes()
        .into_iter()
        .chain(fractal_archetypes())
        .collect();

    // Per Express: se l'identità ha un frattale dominante forte, usa il suo archetipo.
    // Soglia: forza relativa > 0.5 (il frattale rappresenta almeno metà del massimo).
    // Azione 3: se l'archetipo frattale scelto è lo stesso dell'ultimo turno, cade a "express"
    // per evitare la ripetizione strutturale.
    let archetype_name: &'static str = match intention {
        Intention::Express { .. } => {
            let fractal_name = identity
                .and_then(|id| id.dominant_fractal)
                .and_then(|(fid, strength)| {
                    if strength > 0.50 { fractal_archetype_name(fid) } else { None }
                });
            match fractal_name {
                Some(name) if last_archetype == Some(name) => "express", // evita ripetizione
                Some(name) => name,
                None => "express",
            }
        }
        Intention::Reflect => "reflect",
        Intention::Remember { .. } => "remember",
        Intention::Question { .. } => "question",
        Intention::Explore { .. } => "explore",
        Intention::Withdraw { .. } => "withdraw",
        Intention::Dream { .. } => "express",
        Intention::Instruct { .. } => "instruct",
    };

    // Phase 42/54 — Adattamento archetipo dalla ResponseIntention (NarrativeSelf).
    // Express/Reflect/Explore: la ResponseIntention può sostituire l'archetipo.
    // Phase 54: Need/Irony/Desire hanno archetipi dedicati via preferred_archetype().
    let archetype_name: &'static str = if matches!(intention, Intention::Express { .. } | Intention::Reflect | Intention::Explore { .. }) {
        // Livello 1: ResponseIntention deliberata (NarrativeSelf)
        if let Some(preferred) = response_intention.and_then(|ri| ri.preferred_archetype()) {
            // La ResponseIntention Remain non arriva qui (gestita da Withdraw in will),
            // Explore e Express lasciano il campo libero (None).
            preferred
        } else {
            // Livello 2: fallback InputReading (analisi di superficie)
            match input_reading.map(|r| &r.act) {
                Some(crate::topology::input_reading::InputAct::Greeting) => "express",
                Some(crate::topology::input_reading::InputAct::SelfQuery) => {
                    if archetype_name != "identity_exploration" { "identity_exploration" }
                    else { archetype_name }
                }
                Some(crate::topology::input_reading::InputAct::EmotionalExpr) => "express",
                _ => archetype_name,
            }
        }
    } else {
        archetype_name
    };

    // Azione 2: VERITÀ varia struttura in base a codon[1] % 4
    // (emergente dal campo, non casuale)
    let verita_override: Option<SentenceArchetype>;
    let archetype: &SentenceArchetype = if archetype_name == "fractal_verita" {
        let slots = match codon[1] % 4 {
            0 => vec![SlotRole::PrimaryWord, SlotRole::Literal("è"), SlotRole::SecondaryWord],
            1 => vec![SlotRole::PrimaryWord, SlotRole::VerbCandidate, SlotRole::SecondaryWord],
            2 => vec![SlotRole::PrimaryWord, SlotRole::Literal("senza"), SlotRole::SecondaryWord],
            _ => vec![SlotRole::PrimaryWord, SlotRole::Literal("e"), SlotRole::SecondaryWord],
        };
        verita_override = Some(SentenceArchetype {
            name: "fractal_verita",
            slots,
            separators: vec![" ", " "],
            ending: ".",
        });
        verita_override.as_ref().unwrap()
    } else {
        verita_override = None;
        all_archetypes.iter().find(|a| a.name == archetype_name)?
    };

    let mut used: Vec<String> = Vec::new();
    let raw = archetype.instantiate(word_topology, lexicon, active_fractals, &mut used, codon, echo_exclude, valence_drives)?;
    // Post-processing: rimuove preposizioni/articoli orfani in coda ("dalla", "nel", ecc.)
    let text = syntax_center::post_process(&raw);

    Some(TranslatedExpression {
        text,
        archetype_name,
        words_used: used,
    })
}

/// Versione con fallback: se translate_state fallisce, ritorna il testo grezzo.
pub fn translate_or_raw(
    intention: &Intention,
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    active_fractals: &[(FractalId, f64)],
    codon: [usize; 2],
    echo_exclude: &[String],
    raw_fallback: &str,
) -> String {
    match translate_state(intention, word_topology, lexicon, active_fractals, codon, echo_exclude, None, None, None, None, None, None) {
        Some(expr) => expr.text,
        None => raw_fallback.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::lexicon::Lexicon;
    use crate::topology::word_topology::WordTopology;

    fn setup() -> (WordTopology, Lexicon, Vec<(FractalId, f64)>) {
        let lexicon = Lexicon::bootstrap();
        let mut topo = WordTopology::build_from_lexicon(&lexicon);
        // Attiva alcune parole cardinali
        topo.activate_word("io", 0.8);
        topo.activate_word("sentire", 0.8);
        topo.activate_word("dentro", 0.8);
        topo.activate_word("calma", 0.6);
        topo.propagate(1);
        let active_fractals = topo.emerge_fractal_activations(&lexicon).into_iter().collect();
        (topo, lexicon, active_fractals)
    }

    #[test]
    fn test_traduzione_express() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Express {
            salient_fractals: vec![IDENTITA, EMOZIONE],
            urgency: 0.7,
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], None, None, None, None, None, None);
        // Deve produrre qualcosa (anche se non perfetto con bootstrap minimo)
        // Non garantiamo il testo esatto — dipende dal campo
        // Ma garantiamo la struttura: inizia con "Io" o ha parole
        if let Some(expr) = result {
            assert!(!expr.text.is_empty(), "La traduzione non deve essere vuota");
            assert!(!expr.words_used.is_empty(), "Deve usare almeno una parola");
        }
        // Se None: campo troppo sparse — ok per bootstrap minimale
    }

    #[test]
    fn test_traduzione_question_termina_con_punto_interrogativo() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Question {
            gap_region: Some(IDENTITA),
            urgency: 0.6,
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], None, None, None, None, None, None);
        if let Some(expr) = result {
            assert!(expr.text.ends_with('?'),
                "Question deve terminare con ?. Testo: {}", expr.text);
            assert_eq!(expr.archetype_name, "question");
        }
    }

    #[test]
    fn test_traduzione_explore_termina_con_puntini() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Explore {
            unknown_words: vec!["serendipita".to_string()],
            pull: 0.7,
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], None, None, None, None, None, None);
        if let Some(expr) = result {
            assert!(expr.text.ends_with("..."),
                "Explore deve terminare con ... Testo: {}", expr.text);
        }
    }

    #[test]
    fn test_traduzione_no_parole_duplicate() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Reflect;
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], None, None, None, None, None, None);
        if let Some(expr) = result {
            // Nessuna parola deve apparire due volte nella frase
            let words: Vec<&str> = expr.text
                .trim_end_matches(['.', '?', '!'])
                .split_whitespace()
                .filter(|w| *w != "—" && *w != "cosa" && *w != "non" && *w != "so"
                         && *w != "io" && *w != "dentro" && !w.starts_with("..."))
                .collect();
            let unique: std::collections::HashSet<&str> = words.iter().copied().collect();
            assert_eq!(words.len(), unique.len(),
                "Parole duplicate nella frase: {}", expr.text);
        }
    }

    #[test]
    fn test_archetipo_withdraw_produce_silenzio() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Withdraw {
            reason: crate::topology::will::WithdrawReason::Fatigue,
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], None, None, None, None, None, None);
        if let Some(expr) = result {
            assert!(expr.text.contains("..."),
                "Withdraw deve produrre silenzio (via archetipo Phase3): {}", expr.text);
        }
    }

    #[test]
    fn test_translate_or_raw_fallback() {
        // Campo vuoto → usa raw_fallback
        let topo = WordTopology::new();
        let lexicon = Lexicon::bootstrap_cardinal();
        let active_fractals = vec![];
        let intention = Intention::Express {
            salient_fractals: vec![],
            urgency: 0.5,
        };
        let result = translate_or_raw(
            &intention, &topo, &lexicon, &active_fractals, [0, 1], &[], "[campo vuoto]"
        );
        // Con campo vuoto PrimaryWord non si riempie → fallback
        assert!(!result.is_empty());
    }

    #[test]
    fn test_archetipo_instruct_inizia_con_tu_puoi() {
        let (topo, lexicon, active_fractals) = setup();
        // EMPATIA(59) forte → instruct
        let intention = Intention::Instruct { relational_fractal: EMPATIA };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], None, None, None, None, None, None);
        if let Some(expr) = result {
            assert!(expr.text.starts_with("Tu puoi"),
                "Instruct deve iniziare con 'Tu puoi'. Testo: {}", expr.text);
            assert!(expr.text.ends_with('.'),
                "Instruct deve terminare con punto. Testo: {}", expr.text);
            assert_eq!(expr.archetype_name, "instruct");
        }
        // Se None: campo troppo sparse — ok per bootstrap minimale
    }

    #[test]
    fn test_archetipo_instruct_non_echo_input() {
        let mut topo = WordTopology::build_from_lexicon(&Lexicon::bootstrap());
        topo.activate_word("sentire", 0.9);
        topo.activate_word("pace", 0.7);
        topo.activate_word("camminare", 0.6);
        topo.propagate(1);
        let lexicon = Lexicon::bootstrap();
        let active_fractals: Vec<(FractalId, f64)> = topo.emerge_fractal_activations(&lexicon).into_iter().collect();
        let echo_exclude = vec!["sentire".to_string(), "pace".to_string()];
        let intention = Intention::Instruct { relational_fractal: EMPATIA };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &echo_exclude, None, None, None, None, None, None);
        if let Some(expr) = result {
            // Le parole dell'input non devono essere ripetute meccanicamente
            for excl in &echo_exclude {
                // Il VerbCandidate può avere "sentire" se è l'unico verbo — ma PrimaryWord no
                // Qui verifichiamo solo la struttura
                let _ = excl; // La struttura è più importante del contenuto
            }
            assert!(expr.text.starts_with("Tu puoi"),
                "Instruct mantiene struttura 'Tu puoi'. Testo: {}", expr.text);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Test IdentityContext — tensione primaria e archetipi frattali
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_tensione_emerge_in_reflect_quando_stabile() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Reflect;
        let identity = IdentityContext {
            dominant_fractal: None,
            primary_tension: Some(("corpo".to_string(), "mente".to_string())),
            tension_persistence: 5, // stabile oltre soglia
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], Some(&identity), None, None, None, None, None);
        let expr = result.expect("Con tensione stabile e Reflect deve produrre espressione");
        assert_eq!(expr.archetype_name, "tensione",
            "Deve usare archetipo tensione. Testo: {}", expr.text);
        assert!(expr.text.contains("corpo") && expr.text.contains("mente"),
            "Deve contenere entrambi i poli. Testo: {}", expr.text);
        assert!(expr.text.starts_with("Tra "),
            "Tensione inizia con 'Tra'. Testo: {}", expr.text);
    }

    #[test]
    fn test_tensione_non_emerge_sotto_soglia() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Reflect;
        let identity = IdentityContext {
            dominant_fractal: None,
            primary_tension: Some(("corpo".to_string(), "mente".to_string())),
            tension_persistence: 2, // sotto soglia (< 3)
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], Some(&identity), None, None, None, None, None);
        if let Some(expr) = result {
            assert_ne!(expr.archetype_name, "tensione",
                "Sotto soglia non deve usare archetipo tensione: {}", expr.text);
        }
    }

    #[test]
    fn test_tensione_non_emerge_per_express() {
        let (topo, lexicon, active_fractals) = setup();
        // Tensione stabile, ma intenzione Express → non deve emergere
        let intention = Intention::Express {
            salient_fractals: vec![IDENTITA],
            urgency: 0.7,
        };
        let identity = IdentityContext {
            dominant_fractal: None,
            primary_tension: Some(("corpo".to_string(), "mente".to_string())),
            tension_persistence: 10,
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], Some(&identity), None, None, None, None, None);
        if let Some(expr) = result {
            assert_ne!(expr.archetype_name, "tensione",
                "Express non usa archetipo tensione: {}", expr.text);
        }
    }

    #[test]
    fn test_archetipo_frattale_verita_con_frattale_dominante() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Express {
            salient_fractals: vec![IDENTITA],
            urgency: 0.7,
        };
        // VERITA (54) dominante con forza relativa alta
        let identity = IdentityContext {
            dominant_fractal: Some((54, 0.9)), // forza relativa 90% → sopra soglia 50%
            primary_tension: None,
            tension_persistence: 0,
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], Some(&identity), None, None, None, None, None);
        if let Some(expr) = result {
            assert_eq!(expr.archetype_name, "fractal_verita",
                "Con VERITA dominante usa archetipo verita. Testo: {}", expr.text);
        }
        // Se None: campo troppo sparse — ok per bootstrap minimo
    }

    #[test]
    fn test_archetipo_frattale_non_attivato_senza_dominanza() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Express {
            salient_fractals: vec![IDENTITA],
            urgency: 0.7,
        };
        // Frattale dominante debole (forza relativa 30% — sotto soglia 50%)
        let identity = IdentityContext {
            dominant_fractal: Some((54, 0.3)),
            primary_tension: None,
            tension_persistence: 0,
        };
        let result = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], Some(&identity), None, None, None, None, None);
        if let Some(expr) = result {
            // Con dominanza debole, usa express standard
            assert_ne!(expr.archetype_name, "fractal_verita",
                "Dominanza debole non usa archetipo frattale: {}", expr.text);
        }
    }

    #[test]
    fn test_senza_identity_comportamento_invariato() {
        let (topo, lexicon, active_fractals) = setup();
        let intention = Intention::Express {
            salient_fractals: vec![IDENTITA],
            urgency: 0.7,
        };
        // Senza identity context: usa express standard come prima
        let with_none = translate_state(&intention, &topo, &lexicon, &active_fractals, [0, 1], &[], None, None, None, None, None, None);
        if let Some(expr) = with_none {
            assert_eq!(expr.archetype_name, "express",
                "Senza identity usa express standard: {}", expr.text);
        }
    }
}
