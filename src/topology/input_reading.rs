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
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Phase 80: categoria del verbo letta strutturalmente dal kg_proc.
    /// Valori: "copula" / "percettivo" / "cognitivo" / "comunicativo" /
    /// "denominativo" / "azione". Permette a `derive_speech_act` di
    /// distinguere casi che il ClaimKind da solo non separa:
    /// Identity+denominativo ("mi chiamo X") vs Identity+copula ("ho un cane").
    /// `None` quando il verbo non è classificato (lessico esterno al kg_proc).
    pub verb_category: Option<String>,
    /// Phase 83: complemento/tema della costruzione, quando il frame del verbo
    /// lo separa dal predicato. Per i verbi dativi ("mi manca mia madre")
    /// `predicate` porta l'emozione lessicalizzata nel verbo ("mancanza") e
    /// `complement` porta il tema ("madre"). `None` per il frame nominativo
    /// di default. Fluisce nello slot `via` della proposizione.
    pub complement: Option<String>,
    /// Lemma del verbo di superficie del claim (es. "uccidere", "lavorare",
    /// "studiare"). Fluisce in `SentenceProposition.verb_lemma`: la relazione
    /// porta il tipo, questo il verbo concreto. `None` per le copule pure.
    pub verb_lemma: Option<String>,
    /// Il SOGGETTO di superficie, recuperato anche quando è CELATO (sottinteso):
    /// "vogliamo"→"noi", "devo"→"io", "sei"→"tu". In italiano il soggetto è
    /// pro-drop (omesso, ricostruito dalla desinenza); questo lo rende esplicito
    /// per la comprensione. `None` per la 3ª persona (il soggetto è un nome del
    /// mondo, non un pronome) o se non deducibile.
    pub subject_surface: Option<String>,
}

/// Recupera il pronome-soggetto dalla persona del verbo (pro-drop italiano):
/// la desinenza verbale codifica il soggetto anche quando è omesso. 3ª persona
/// → `None` (il soggetto è un nome, non un pronome). Meccanismo generale.
pub fn surface_subject_for(person: crate::topology::grammar::Person) -> Option<String> {
    use crate::topology::grammar::Person;
    Some(match person {
        Person::First       => "io",
        Person::FirstPlural => "noi",
        Person::Second      => "tu",
        Person::SecondPlural => "voi",
        Person::Third | Person::ThirdPlural => return None,
    }.to_string())
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
    /// Phase 67: proprietà discorsive percepite dal campo post-attivazione.
    pub perceived_properties: Vec<(String, f64)>,
    /// Phase 67: profondità della comprensione — quanti nuclei semantici
    /// l'entità ha estratto dall'input. 0 = non ha capito nulla.
    /// 5+ = comprensione profonda. Usato da deliberate() per decidere
    /// se esplorare (poco capito) o esprimere (molto capito).
    pub comprehension_depth: usize,
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
    kg_proc: Option<&KnowledgeGraph>,
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
    let speaker_claim = detect_speaker_claim(raw_words, lexicon, kg, kg_proc);

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

    InputReading { act, intensity, salient_word, speaker_claim, perceived_properties: vec![], comprehension_depth: 0 }
}

/// Phase 80: rileva un claim strutturale soggetto/predicato nell'input.
///
/// L'identificazione del verbo, della sua categoria semantica e del kind del
/// claim è completamente STRUTTURALE — legge dal kg_proc le triple
/// `<verbo> IsA verbo`, `<verbo> IsA copula|percettivo|cognitivo|comunicativo|
/// denominativo|azione`. Nessuna lista hardcoded di verbi italiani in Rust.
/// Aggiungere/rimuovere verbi è curation, mai modifica di codice.
///
/// Pipeline:
///   1. Pronome esplicito (io/mi → Speaker; tu/ti → Entity), se presente.
///   2. Per ogni token, prova a lemmatizzarlo come verbo (irregolari coperti
///      da `grammar::lemmatize`, regolari coperti da fallback morfologico
///      validato sul kg_proc).
///   3. Il primo verbo lemmatizzato che ha categoria nota nel kg_proc è il
///      verbo del claim. La sua categoria determina il ClaimKind:
///        denominativo → Identity (presentazione)
///        percettivo   → Feeling  (validato: predicato deve essere stato interno)
///        cognitivo    → Action
///        comunicativo → Action
///        azione       → Action
///        copula       → Identity se predicato non è stato interno,
///                       Feeling   se predicato è stato interno
///   4. L'agente è determinato dal pronome esplicito (se presente) o dalla
///      persona del verbo (1sg → Speaker, 2sg → Entity).
///   5. Il predicato è la prima parola-contenuto dopo il verbo, saltando
///      le function-word identificate strutturalmente dal kg_proc.
///
/// Restituisce None se nessun verbo classificato emerge dall'input.
pub fn detect_speaker_claim(
    raw_words: &[String],
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<SpeakerClaim> {
    if raw_words.is_empty() { return None; }

    // ── 1. Pronome esplicito ─────────────────────────────────────────────
    // "io"/"mi"/"noi"/"ci" → Speaker. "tu"/"ti"/"voi"/"vi" → Entity.
    let explicit_agent: Option<ClaimAgent> = raw_words.iter()
        .find_map(|w| match w.as_str() {
            "io" | "mi" | "noi" | "ci" => Some(ClaimAgent::Speaker),
            "tu" | "ti" | "voi" | "vi" => Some(ClaimAgent::Entity),
            _ => None,
        });

    // ── 2. Trova il primo verbo classificato ─────────────────────────────
    //   Le parole-funzione (articolo/preposizione/avverbio/…) NON sono mai verbi
    //   finiti: si saltano PRIMA di lemma_of_verb, così "una"→unire (falso
    //   positivo morfologico) non viene scambiato per un verbo. La categoria
    //   viene dal kg_proc (verbi curati: percettivo/cognitivo/…) oppure, per i
    //   verbi-concetto del kg_sem (`IsA azione/atto/…`) non ancora curati nel
    //   kg_proc, di default "azione" — così riuscire/dormire/concentrarsi sono
    //   riconosciuti come verbi del claim senza enumerarli a mano.
    let mut verb_match: Option<(usize, String, crate::topology::grammar::Person, &'static str)> = None;
    'find_verb: for (pos, w) in raw_words.iter().enumerate() {
        if is_kg_proc_function_word(w, kg_proc) {
            continue;
        }
        // Analisi logica (Phase 86+, no-trucchi): una parola preceduta da
        // articolo o determinante è la TESTA DI UN SINTAGMA NOMINALE — mai un
        // verbo coniugato ("il silenzio" ≠ 1sg di silenziare; "io silenzio i
        // critici" resta verbo perché "io" è pronome). Una parola preceduta da
        // PREPOSIZIONE non è mai un verbo FINITO (la preposizione regge nome o
        // infinito): "di accordo"/"con anna" → accordo/anna sono nomi, non
        // accordare/annare. Il contesto decide, come DATO del kg_proc.
        if pos > 0 {
            let prev = raw_words[pos - 1].to_lowercase();
            if is_nominal_intro(&prev, kg_proc)
                || crate::topology::prepositions::is_preposition(&prev) {
                continue;
            }
        }
        if let Some((infinitive, person)) = lemma_of_verb(w, kg_proc, kg) {
            let curated = verb_category(&infinitive, kg_proc);
            let category = curated
                .or_else(|| if is_verb_concept(&infinitive, kg) { Some("azione") } else { None });
            if let Some(category) = category {
                // Disambiguazione nome-proprio/verbo, STRUTTURALE (no maiuscole, no
                // liste di nomi): un candidato DEBOLMENTE attestato (verbità solo
                // dalla catena semantica del kg_sem, non curato nel kg_proc) e
                // immediatamente seguito da un verbo CURATO è il SOGGETTO, non il
                // verbo. "Marco preferisce" → marcare (debole) precede preferire
                // (curato, valutativo) → marco è soggetto. La congiunzione spezza
                // l'adiacenza ("nuoto e amo": "e" non è verbo → nuoto resta verbo).
                if curated.is_none() {
                    for next in raw_words.iter().skip(pos + 1) {
                        let nl = next.to_lowercase();
                        let ninf = lemma_of_verb(next, kg_proc, kg).map(|(i, _)| i);
                        // copula o verbo CURATO più avanti ⇒ il candidato debole era
                        // il SOGGETTO ("Marco preferisce", "Marco non è d'accordo").
                        let stronger_verb = kg_proc.map_or(false, |kp| {
                            is_kg_proc_isa(kp, &nl, "copula")
                                || ninf.as_deref().map_or(false, |x|
                                    verb_category(x, kg_proc).is_some()
                                        || is_kg_proc_isa(kp, x, "copula"))
                        });
                        if stronger_verb { continue 'find_verb; }
                        // salta SOLO negazione/avverbio interposti; una CONGIUNZIONE o
                        // una parola di contenuto non-verbo chiude la scansione (la
                        // congiunzione coordina due verbi: "nuoto e amo" → nuoto verbo).
                        let skippable = kg_proc.map_or(false, |kp|
                            is_kg_proc_isa(kp, &nl, "marcatore")
                                || is_kg_proc_isa(kp, &nl, "avverbio"));
                        if skippable { continue; }
                        break;
                    }
                }
                verb_match = Some((pos, infinitive, person, category));
                break;
            }
        }
    }
    let (verb_pos, infinitive, person, category) = verb_match?;

    // ── 2.bis Frame di costruzione: i verbi dativi rimappano i ruoli ──────
    //   "mi manca mia madre" → esperiente=clitico (mi→Speaker), emozione
    //   lessicalizzata nel verbo (Expresses→"mancanza"), tema=nome dopo il
    //   verbo ("madre"). Nessun caso Rust verbo-specifico: il frame e
    //   l'emozione vivono nel kg_proc come dato.
    if verb_frame(&infinitive, kg_proc).as_deref() == Some("dativo") {
        return build_dative_claim(raw_words, verb_pos, &infinitive, explicit_agent, kg_proc);
    }

    // ── 2.bis-pron Frame PRONOMINALE-riflessivo: "mi sento solo" ─────────
    //   Un verbo pronominale (`IsA pronominale` nel kg_proc) con un clitico
    //   riflessivo presente (`IsA riflessivo`) è una predicazione di STATO:
    //   l'aggettivo che segue è un FeelsAs, anche se non è un inner-state curato.
    //   È la COSTRUZIONE a marcare il sentire (capacità grammaticale come dato),
    //   non il lessico. Senza clitico ("sento la voce") il frame non scatta →
    //   il ramo percettivo applica il gate inner-state (correttamente rifiuta).
    if is_pronominal_reflexive(&infinitive, raw_words, kg_proc) {
        return build_reflexive_claim(raw_words, verb_pos, &infinitive, explicit_agent, person, kg_proc);
    }

    // ── 2.bis-mod Frame MODALE + infinito: "devo studiare", "voglio cambiare" ─
    //   Un modale (`IsA modale` nel kg_proc: dovere/volere/potere) modula un
    //   altro verbo: il CONTENUTO è l'infinito che segue (saltando "a"/"di"),
    //   non il modale. "voglio cambiare vita" → l'azione è `cambiare` (oggetto
    //   `vita`), non `Expresses cambiare`. Senza infinito ("voglio un caffè") il
    //   frame non scatta → il modale resta il verbo (cognitivo). L'agente viene
    //   dal modale (la persona di "devo"/"voglio").
    if kg_proc.map(|kp| is_kg_proc_isa(kp, &infinitive, "modale")).unwrap_or(false) {
        if let Some((inf_pos, inf_lemma)) = infinitive_after(raw_words, verb_pos + 1, kg_proc, kg) {
            if let Some(c) = build_modal_claim(raw_words, inf_pos, &inf_lemma, explicit_agent, person, lexicon, kg, kg_proc) {
                return Some(c);
            }
        }
    }

    // ── 2.ter Frame tempi composti: ausiliare + participio passato ───────
    //   "ho lavorato", "sono andato" → l'ausiliare NON è il verbo di
    //   contenuto: lo è il participio. Frame come DATO (avere/essere IsA
    //   ausiliare nel kg_proc) + participio per morfologia (-ato/-uto/-ito).
    //   Un composto è un atto/evento compiuto → categoria azione. Evita il
    //   misread "ho lavorato" → "Speaker IsA lavorato" (identità falsa).
    //   "sono triste"/"sono Marco" non hanno participio → cadono al ramo copula.
    let is_auxiliary = kg_proc
        .map(|kp| is_kg_proc_isa(kp, &infinitive, "ausiliare"))
        .unwrap_or(false);
    if is_auxiliary {
        if let Some((part_pos, part_lemma)) = find_past_participle(raw_words, verb_pos + 1, kg_proc) {
            use crate::topology::grammar::Person as P;
            let agent = match explicit_agent {
                Some(a) => a,
                None => match person {
                    P::First | P::FirstPlural   => ClaimAgent::Speaker,
                    P::Second | P::SecondPlural => ClaimAgent::Entity,
                    P::Third | P::ThirdPlural   => return None,
                },
            };
            // Oggetto dell'azione = oggetto DIRETTO dopo il participio; se
            // intransitivo ("ho lavorato") o seguito solo da complementi
            // preposizionali ("ho litigato con sorella"), il verbo stesso è il
            // contenuto e il nome resta complemento.
            let predicate = direct_object_after(raw_words, part_pos + 1, kg_proc)
                .unwrap_or_else(|| part_lemma.clone());
            return Some(SpeakerClaim {
                agent,
                kind: ClaimKind::Action,
                predicate,
                verb_category: Some("azione".to_string()),
                complement: None,
                verb_lemma: Some(part_lemma),
                subject_surface: explicit_subject_pronoun(raw_words).or_else(|| surface_subject_for(person)),
            });
        }
    }

    // ── 3. Agente: pronome vince; altrimenti deduzione dalla persona ─────
    //   MA: un clitico (mi/ti/ci/vi) davanti a un verbo di 3ª persona è
    //   l'OGGETTO, non il soggetto — "mia moglie mi capisce" = la moglie capisce
    //   ME. L'agente è allora il Mondo (3ª persona), non lo Speaker. Solo un
    //   pronome-SOGGETTO (io/tu/noi/voi) vale come agente di un verbo di 3ª.
    //   I frame riflessivo/dativo (dove il clitico È l'esperiente) sono già
    //   intercettati sopra: qui resta la costruzione transitiva col clitico-oggetto.
    use crate::topology::grammar::Person;
    let has_subject_pron = raw_words.iter()
        .any(|w| matches!(w.as_str(), "io" | "tu" | "noi" | "voi"));
    let clitic_object_third =
        matches!(person, Person::Third | Person::ThirdPlural) && !has_subject_pron;
    let agent = match explicit_agent {
        Some(a) if !clitic_object_third => a,
        _ => match person {
            Person::First  | Person::FirstPlural  => ClaimAgent::Speaker,
            Person::Second | Person::SecondPlural => ClaimAgent::Entity,
            // 3sg/pl non sono claim diretti — TRANNE l'imperativo. Per gli -are
            // la 2sg imperativa coincide col 3sg presente ("gira"/"mescola"): la
            // disambiguazione è STRUTTURALE (niente lista di verbi) — verbo a
            // INIZIO frase, senza soggetto esplicito, con un oggetto-contenuto
            // dopo = istruzione rivolta all'interlocutore (Entity). "gira la
            // valvola" → Entity Does valvola. "piove" (nessun oggetto) → None.
            Person::Third | Person::ThirdPlural => {
                let has_object = raw_words.iter()
                    .skip(verb_pos + 1)
                    .any(|w| !is_kg_proc_function_word(w, kg_proc));
                if verb_pos == 0 && has_object {
                    ClaimAgent::Entity
                } else {
                    return None;
                }
            }
        },
    };

    // ── 4. Estrai il predicato — la TESTA del sintagma dopo il verbo ─────
    //   Non la prima parola-contenuto (sarebbe l'aggettivo attributivo:
    //   "una grande nostalgia" → "grande"), ma la TESTA del sintagma nominale:
    //   il nome. Scansiono il sintagma (parole-contenuto consecutive, saltati
    //   gli articoli/determinanti iniziali, fermandomi alla prima preposizione/
    //   congiunzione che lo chiude) e scelgo la testa: uno stato interno
    //   (Feeling) o un nome (IsA non vuoto nel kg_sem); tra pari, l'ultimo del
    //   sintagma (in italiano il nome segue gli aggettivi attributivi).
    let np: Vec<String> = raw_words.iter()
        .skip(verb_pos + 1)
        .skip_while(|w| is_kg_proc_function_word(w, kg_proc))
        .take_while(|w| !is_kg_proc_function_word(w, kg_proc))
        .cloned()
        .collect();
    let is_head = |w: &str| -> bool {
        is_inner_state(w, lexicon, kg)
            || kg.map(|k| !k.query_objects(&w.to_lowercase(), RelationType::IsA).is_empty())
                .unwrap_or(false)
    };
    // Una parola la cui IsA DIRETTA è `qualità`/`attributo` è un aggettivo
    // (bello/grande/profondo/rosso), non la testa nominale. Segnale strutturale
    // dal kg_sem, non POS/morfologia: "futuro bello" → testa=futuro (IsA tempo),
    // bello demoto (IsA qualità). Vale per l'aggettivo POSPOSTO, che l'euristica
    // "ultimo del sintagma" da sola sbagliava.
    let is_quality = |w: &str| -> bool {
        kg.map(|k| k.query_objects(&w.to_lowercase(), RelationType::IsA)
            .iter().any(|t| t.eq_ignore_ascii_case("qualità") || t.eq_ignore_ascii_case("attributo")))
            .unwrap_or(false)
    };
    let predicate = if np.len() >= 2 {
        // Testa = ultimo SOSTANTIVO (head non-qualità). Fallback onesto: ultimo
        // head qualunque, poi ultimo del sintagma.
        np.iter().rev().find(|w| is_head(w) && !is_quality(w))
            .or_else(|| np.iter().rev().find(|w| is_head(w)))
            .or_else(|| np.last())
            .cloned()?
    } else {
        np.first().cloned()?
    };

    // ── 5. ClaimKind dalla categoria del verbo ───────────────────────────
    //   La mappa categoria→kind è strutturale (lettura della IsA chain),
    //   non un dispatch: ogni categoria denota un modo di posizionarsi che
    //   ClaimKind classifica con le sue 3 varianti.
    let kind = match category {
        "denominativo" => ClaimKind::Identity,    // "mi chiamo X"
        "percettivo"   => {
            // "sento/provo X" è Feeling solo se X è stato interno.
            // "sento la voce" → percezione esterna, non claim: rifiuta.
            if is_inner_state(&predicate, lexicon, kg) {
                ClaimKind::Feeling
            } else {
                return None;
            }
        }
        // "amo/preferisco/desidero X" — attitudine valutativa: un posizionamento
        // affettivo verso l'oggetto (FeelsAs, non Expresses). È un Feeling.
        "valutativo"   => ClaimKind::Feeling,
        "cognitivo" | "comunicativo" | "azione" => ClaimKind::Action,
        "copula"       => {
            // "io sono triste" → Feeling; "io ho un cane" → Identity.
            // Il predicato decide se il posizionamento è di stato interno.
            if is_inner_state(&predicate, lexicon, kg) {
                ClaimKind::Feeling
            } else {
                ClaimKind::Identity
            }
        }
        _ => return None, // categoria non riconosciuta
    };

    Some(SpeakerClaim {
        agent,
        kind,
        predicate,
        verb_category: Some(category.to_string()),
        complement: None,
        verb_lemma: Some(infinitive.clone()),
        subject_surface: explicit_subject_pronoun(raw_words).or_else(|| surface_subject_for(person)),
    })
}

/// Il pronome-soggetto ESPLICITO presente nell'input, se c'è (io/noi/tu/voi/
/// lei/lui/loro/egli/ella). Quando assente, il soggetto è celato e si recupera
/// dalla persona (`surface_subject_for`).
fn explicit_subject_pronoun(raw_words: &[String]) -> Option<String> {
    for w in raw_words {
        match w.to_lowercase().as_str() {
            p @ ("io" | "noi" | "tu" | "voi" | "lei" | "lui" | "loro" | "egli" | "ella") => {
                return Some(p.to_string());
            }
            _ => {}
        }
    }
    None
}

/// Phase 83: costruisce un claim per un verbo a costruzione dativa.
/// L'esperiente è il clitico (mi/ti/ci/vi → già in `explicit_agent`);
/// l'emozione è lessicalizzata nel verbo (`Expresses` nel kg_proc); il tema è
/// la prima parola-contenuto dopo il verbo (i determinanti si saltano).
/// `None` se manca il clitico o l'emozione insegnata — niente fallback
/// rumoroso. Nessuna logica verbo-specifica: tutto è dato del kg_proc.
fn build_dative_claim(
    raw_words: &[String],
    verb_pos: usize,
    infinitive: &str,
    explicit_agent: Option<ClaimAgent>,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<SpeakerClaim> {
    let agent = explicit_agent?;
    let kp = kg_proc?;
    let emotion = kp.query_objects(infinitive, RelationType::Expresses)
        .into_iter().next()?.to_string();
    let theme = raw_words.iter()
        .skip(verb_pos + 1)
        .find(|w| !is_kg_proc_function_word(w, kg_proc))
        .cloned();
    let subject_surface = explicit_subject_pronoun(raw_words)
        .or(Some(match agent { ClaimAgent::Speaker => "io", ClaimAgent::Entity => "tu" }.to_string()));
    Some(SpeakerClaim {
        agent,
        kind: ClaimKind::Feeling,
        predicate: emotion,
        verb_category: Some("percettivo".to_string()),
        complement: theme,
        // Nel frame dativo il verbo ("piace"/"manca") è ASSORBITO nella relazione
        // (FeelsAs) + nell'emozione lessicalizzata (predicate): non c'è un verbo
        // di contenuto da realizzare. Inoltre concorderebbe con lo STIMOLO, non
        // con l'esperiente → coniugarlo alla persona del soggetto darebbe forme
        // sbagliate ("piaci"). `None` → la voce usa il verbo-relazione ("provi").
        verb_lemma: None,
        subject_surface,
    })
}

/// Phase 86+: la costruzione pronominale-riflessiva ("mi sento solo"). Il verbo
/// è dichiarato pronominale nel kg_proc (`IsA pronominale`) E nell'enunciato c'è
/// un clitico riflessivo (`IsA riflessivo`: mi/ti/si/ci/vi). Allora l'aggettivo
/// che segue è uno STATO sentito → FeelsAs, anche senza essere un inner-state
/// curato. Strutturale: il clitico è il discriminatore (legge dal kg_proc, niente
/// liste). "sento la voce" (nessun clitico) → false → resta percezione esterna.
fn is_pronominal_reflexive(
    infinitive: &str,
    raw_words: &[String],
    kg_proc: Option<&KnowledgeGraph>,
) -> bool {
    let Some(kp) = kg_proc else { return false; };
    if !is_kg_proc_isa(kp, infinitive, "pronominale") {
        return false;
    }
    raw_words.iter().any(|w| is_kg_proc_isa(kp, &w.to_lowercase(), "riflessivo"))
}

/// Costruisce il claim per un verbo pronominale-riflessivo: "mi sento solo" →
/// `Speaker FeelsAs solo`. Lo stato sentito è la prima parola-contenuto dopo il
/// verbo; la relazione FeelsAs deriva dalla categoria percettiva (mappa
/// esistente). Bypassa il gate inner-state: è la costruzione a marcare il sentire.
fn build_reflexive_claim(
    raw_words: &[String],
    verb_pos: usize,
    infinitive: &str,
    explicit_agent: Option<ClaimAgent>,
    person: crate::topology::grammar::Person,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<SpeakerClaim> {
    use crate::topology::grammar::Person;
    let agent = explicit_agent.or(match person {
        Person::First | Person::FirstPlural   => Some(ClaimAgent::Speaker),
        Person::Second | Person::SecondPlural => Some(ClaimAgent::Entity),
        Person::Third | Person::ThirdPlural   => None,
    })?;
    let predicate = raw_words.iter()
        .skip(verb_pos + 1)
        .find(|w| !is_kg_proc_function_word(w, kg_proc))
        .cloned()?;
    Some(SpeakerClaim {
        agent,
        kind: ClaimKind::Feeling,
        predicate,
        verb_category: Some("percettivo".to_string()),
        complement: None,
        verb_lemma: Some(infinitive.to_string()),
        subject_surface: explicit_subject_pronoun(raw_words).or_else(|| surface_subject_for(person)),
    })
}

/// Phase 86+: l'infinito-contenuto dopo un modale (o un catenativo), saltando
/// la preposizione "a"/"di" e le funzionali. "devo [—] studiare", "voglio
/// cambiare", "riesco A dormire". Finestra breve; ritorna (posizione, lemma).
pub(crate) fn infinitive_after(
    raw_words: &[String],
    from: usize,
    kg_proc: Option<&KnowledgeGraph>,
    kg: Option<&KnowledgeGraph>,
) -> Option<(usize, String)> {
    let n = raw_words.len();
    let mut j = from;
    while j < n && j <= from + 2 {
        let w = raw_words[j].to_lowercase();
        // Infinito nudo ("studiare") O infinito con enclitico ("svegliarmi",
        // "alzarsi", "farlo"): il secondo è la stessa forma con un pronome
        // clitico agganciato (regola grammaticale, non lista di verbi).
        if let Some(inf) = infinitive_surface(&w) {
            if lemma_of_verb(&inf, kg_proc, kg).is_some() {
                return Some((j, inf));
            }
        }
        if is_kg_proc_function_word(&w, kg_proc) {
            j += 1;
            continue;
        }
        break;
    }
    None
}

/// La parola è un infinito di superficie? Ritorna l'infinito nudo. Riconosce
/// l'infinito puro (-are/-ere/-ire) e l'infinito con un PRONOME ENCLITICO
/// agganciato ("svegliarmi"→svegliare, "alzarsi"→alzare, "farlo"→fare,
/// "dirgli"→dire): l'italiano cliticizza posponendo il pronome e cadendo la -e
/// finale dell'infinito. È morfologia (regola finita di posizione clitica), non
/// una lista di verbi — il chiamante valida poi il lemma contro il KG.
fn infinitive_surface(w: &str) -> Option<String> {
    if w.ends_with("are") || w.ends_with("ere") || w.ends_with("ire") {
        return Some(w.to_string());
    }
    // Pronomi enclitici (singoli + combinati comuni), dal più lungo al più corto
    // per non troncare "glielo" in "lo".
    const CLITICS: &[&str] = &[
        "glielo", "gliela", "glieli", "gliele", "gliene",
        "melo", "mela", "meli", "mele", "mene",
        "telo", "tela", "teli", "tele", "tene",
        "celo", "cela", "celi", "cele", "cene",
        "velo", "vela", "veli", "vele", "vene",
        "selo", "sela", "seli", "sele", "sene",
        "mi", "ti", "si", "ci", "vi", "ne", "lo", "la", "li", "le", "gli",
    ];
    for clitic in CLITICS {
        if let Some(stem) = w.strip_suffix(clitic) {
            // Dopo aver tolto il clitico, lo stem deve finire in -ar/-er/-ir
            // (l'infinito senza la -e caduta) e avere corpo sufficiente.
            if stem.len() >= 3
                && (stem.ends_with("ar") || stem.ends_with("er") || stem.ends_with("ir"))
            {
                return Some(format!("{stem}e"));
            }
        }
    }
    None
}

/// Phase 86+: costruisce il claim quando un modale modula un infinito. Il
/// CONTENUTO è l'infinito: la sua categoria dà la relazione, il suo oggetto
/// diretto (se transitivo) è il predicato, altrimenti l'infinito stesso.
/// L'agente viene dal modale ("devo"→Speaker). "devo finire il progetto" →
/// Speaker Does progetto; "devo studiare per l'esame" → Speaker Does studiare.
fn build_modal_claim(
    raw_words: &[String],
    inf_pos: usize,
    inf_lemma: &str,
    explicit_agent: Option<ClaimAgent>,
    person: crate::topology::grammar::Person,
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<SpeakerClaim> {
    use crate::topology::grammar::Person;
    let agent = explicit_agent.or(match person {
        Person::First | Person::FirstPlural   => Some(ClaimAgent::Speaker),
        Person::Second | Person::SecondPlural => Some(ClaimAgent::Entity),
        Person::Third | Person::ThirdPlural   => None,
    })?;
    let category = verb_category(inf_lemma, kg_proc).unwrap_or("azione");
    let predicate = direct_object_after(raw_words, inf_pos + 1, kg_proc)
        .unwrap_or_else(|| inf_lemma.to_string());
    let kind = match category {
        // Sotto un modale, un verbo denominativo è un'AZIONE ("devo chiamare il
        // dottore" = chiamare-azione, non auto-denominarsi). Il denominativo vale
        // solo nel riflessivo "mi chiamo X" (gestito altrove).
        "denominativo" => ClaimKind::Action,
        "percettivo" => {
            if is_inner_state(&predicate, lexicon, kg) { ClaimKind::Feeling } else { ClaimKind::Action }
        }
        "copula" => ClaimKind::Identity,
        _ => ClaimKind::Action, // cognitivo | comunicativo | azione
    };
    // denominativo→azione: la relazione dev'essere Does, non IsA → forziamo la
    // categoria a "azione" così relation_from_verb_category dà Does.
    let category = if category == "denominativo" { "azione" } else { category };
    Some(SpeakerClaim {
        agent,
        kind,
        predicate,
        verb_category: Some(category.to_string()),
        complement: None,
        verb_lemma: Some(inf_lemma.to_string()),
        subject_surface: explicit_subject_pronoun(raw_words).or_else(|| surface_subject_for(person)),
    })
}

/// Phase 83: legge il frame di costruzione del verbo dal kg_proc (oggi solo
/// `dativo`; l'assenza = frame nominativo di default). I frame sono una
/// tabella finita — il minimo denominatore della struttura argomentale —
/// mentre i verbi sono infiniti e quasi tutti a default.
/// Phase 84 (2c-C): cerca il primo participio passato a partire da `from`,
/// saltando avverbi/determinanti ("ho **molto** lavorato"). Ritorna
/// `(posizione, lemma-infinito)`. Morfologia regolare: -ato/-uto/-ito (con
/// concordanza -a/-i/-e) → -are/-ere/-ire. Nel contesto post-ausiliare la
/// morfologia è un segnale forte; non serve validare oltre.
/// Prima parola-contenuto dopo `start` che sia un OGGETTO DIRETTO del verbo.
/// Restituisce `None` se la prima parola-contenuto è introdotta da una
/// preposizione di contenuto (con/di/da/per/su/in/contro/…): in quel caso il
/// nome è un COMPLEMENTO (catturato da `extract_complements`), non l'oggetto —
/// il verbo è usato intransitivamente ("ho litigato CON sorella" → litigare,
/// non "sorella"). La preposizione di moto/dativo "a" (hypotheses vuote) NON
/// interrompe. Strutturale: nessuna lista di verbi transitivi/intransitivi.
fn direct_object_after(
    raw_words: &[String],
    start: usize,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<String> {
    for w in raw_words.iter().skip(start) {
        let lw = w.to_lowercase();
        if !crate::topology::prepositions::hypotheses(&lw).is_empty() {
            // Preposizione di contenuto → ciò che segue è complemento, non oggetto.
            return None;
        }
        if !is_kg_proc_function_word(w, kg_proc) {
            return Some(w.clone());
        }
    }
    None
}

/// Cerca il participio passato di un tempo composto a partire da `from`. Il
/// participio dev'essere ADIACENTE all'ausiliare, modulo parole-funzione
/// (clitici/avverbi/articoli): "ho **sempre** fatto" ok. Una parola di CONTENUTO
/// interposta interrompe — NON è un composto: "ho **voglia** di gelato" è
/// avere+nome, non "ho gelato" (Phase 86+, bug stanato su input vari: "gelato"
/// è morfologicamente un participio, ma il chunker lo vede complemento). Mirror
/// di `analisi_logica::participio_dopo`.
pub(crate) fn find_past_participle(
    raw_words: &[String],
    from: usize,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<(usize, String)> {
    for j in from..raw_words.len() {
        let w = raw_words[j].to_lowercase();
        // Un articolo/determinante introduce un sintagma NOMINALE: ciò che
        // segue è un NOME, non il participio di un tempo composto — "ho UN
        // significato" è avere+nome, non il passato prossimo di significare.
        // Tra ausiliare e participio stanno solo clitici/avverbi ("ho sempre fatto").
        if is_nominal_intro(&w, kg_proc) {
            break;
        }
        if let Some(lemma) = participle_lemma(&w) {
            return Some((j, lemma));
        }
        // salta le altre parole-funzione (clitici/avverbi/prep); una parola
        // di contenuto non-participio chiude: non c'è composto.
        if is_kg_proc_function_word(&w, kg_proc) {
            continue;
        }
        break;
    }
    None
}

/// La parola introduce un sintagma nominale (articolo o determinante, letti
/// dal kg_proc come DATO — classe→funzione)? Ciò che la segue è un nome.
fn is_nominal_intro(word: &str, kg_proc: Option<&KnowledgeGraph>) -> bool {
    let Some(kp) = kg_proc else { return false };
    let w = word.to_lowercase();
    is_kg_proc_isa(kp, &w, "articolo") || is_kg_proc_isa(kp, &w, "determinante")
}

/// Lemma infinito da un participio passato regolare, o `None`.
fn participle_lemma(word: &str) -> Option<String> {
    let w = word.to_lowercase();
    const GROUPS: &[(&[&str], &str)] = &[
        (&["ato", "ata", "ati", "ate"], "are"),
        (&["uto", "uta", "uti", "ute"], "ere"),
        (&["ito", "ita", "iti", "ite"], "ire"),
    ];
    for (suffixes, inf) in GROUPS {
        for suf in *suffixes {
            if let Some(stem) = w.strip_suffix(suf) {
                // stem ≥3 esclude "stato"(st)/"dato"(d) e simili monosillabi.
                if stem.len() >= 3 {
                    return Some(format!("{}{}", stem, inf));
                }
            }
        }
    }
    // Phase 86 (#3): participi IRREGOLARI ("ho preso"→prendere, "ho scritto"→scrivere).
    crate::topology::grammar::irregular_participle(&w)
}

fn verb_frame(infinitive: &str, kg_proc: Option<&KnowledgeGraph>) -> Option<String> {
    let kp = kg_proc?;
    let parents = kp.query_objects(infinitive, RelationType::IsA);
    const FRAMES: &[&str] = &["dativo"];
    FRAMES.iter()
        .find(|f| parents.iter().any(|p| p.eq_ignore_ascii_case(f)))
        .map(|f| f.to_string())
}

/// Espande un token grezzo trattando l'ELISIONE come GRAMMATICA, non come taglio
/// cieco dell'apostrofo. Un token con apostrofo è [parola-funzione elisa | resto];
/// la regola è unica e chiusa-classe (nessun dato per-caso):
///  - preposizione elisa (dall'/dell'/d'/nell'/sull'/all'/coll') → PRESERVATA come
///    token: porta l'ipotesi di relazione. "dall'ignoto" → ["da","ignoto"].
///  - clitico eliso (m'/t'/c'/s'/v'/n') → PRESERVATO: porta un ruolo (oggetto/
///    riflessivo). "m'ascolti" → ["mi","ascolti"].
///  - "l'" AMBIGUO (articolo vs clitico-oggetto): lo decide la categoria della
///    parola SEGUENTE — verbo → clitico "lo" ("l'ascolto"→["lo","ascolto"]); nome
///    → articolo, scartato ("l'amore"→["amore"]). Ponderato, non cieco: serve la
///    grammatica (kg_proc); se assente, default articolo (comportamento storico).
///  - articolo eliso non ambiguo (un'/gl') → scartato (gli articoli si saltano).
///  - apocope (po'=poco, be'=bene): niente dopo l'apostrofo → forma piena.
/// I pezzi passano per `clean_token` (pulizia/lowercase). Per un token SENZA
/// apostrofo è esattamente `clean_token` — comportamento invariato.
pub fn expand_elision(
    raw: &str,
    kg_proc: Option<&crate::topology::knowledge_graph::KnowledgeGraph>,
    kg: Option<&crate::topology::knowledge_graph::KnowledgeGraph>,
) -> Vec<String> {
    use crate::topology::lexicon::clean_token;
    let clean1 = |s: &str| clean_token(s).into_iter().collect::<Vec<String>>();

    let apos = raw.char_indices().find(|(_, c)| *c == '\'' || *c == '\u{2019}');
    let (i, ch) = match apos {
        Some(x) => x,
        None => return clean1(raw),
    };
    let pre = raw[..i].to_lowercase();
    let post = &raw[i + ch.len_utf8()..];

    // Apocope: nulla di alfabetico dopo l'apostrofo (po' / be' / mo').
    if !post.chars().any(|c| c.is_alphabetic()) {
        let full = match pre.as_str() { "po" => "poco", "be" => "bene", o => o };
        return clean1(full);
    }

    // Preposizione elisa → preservata (porta l'ipotesi di relazione).
    if let Some(p) = match pre.as_str() {
        "dall" => Some("da"), "dell" => Some("di"), "d" => Some("di"),
        "nell" => Some("in"), "sull" => Some("su"), "all" => Some("a"),
        "coll" => Some("con"), _ => None,
    } {
        let mut out = vec![p.to_string()];
        out.extend(clean1(post));
        return out;
    }

    // Clitico eliso → preservato (porta un ruolo: oggetto/riflessivo).
    if let Some(cl) = match pre.as_str() {
        "m" => Some("mi"), "t" => Some("ti"), "c" => Some("ci"),
        "s" => Some("si"), "v" => Some("vi"), "n" => Some("ne"), _ => None,
    } {
        let mut out = vec![cl.to_string()];
        out.extend(clean1(post));
        return out;
    }

    // "l'" ambiguo: verbo dopo → clitico-oggetto "lo"; nome dopo → articolo
    // (scartato). La categoria della parola seguente decide — ponderato, non cieco.
    if pre == "l" {
        let rest = clean1(post);
        let follows_verb = kg_proc.is_some()
            && rest.first().map(|w| lemma_of_verb(w, kg_proc, kg).is_some()).unwrap_or(false);
        if follows_verb {
            let mut out = vec!["lo".to_string()];
            out.extend(rest);
            return out;
        }
        return rest;
    }

    // un'/gl' (articolo) e forme sconosciute → comportamento storico (tieni il resto).
    clean1(post)
}

/// Phase 80: lemmatizza una forma verbale italiana usando, in ordine:
///   1. `grammar::lemmatize` (irregolari + suffissi noti)
///   2. Match diretto del token come infinito nel kg_proc
///   3. Fallback morfologico per il presente regolare (-are/-ere/-ire),
///      validato sul kg_proc (il candidato deve essere `IsA verbo`).
///
/// Restituisce `(infinito, persona)` se il verbo è riconosciuto, `None`
/// altrimenti. Il fallback è autocorrettivo: se il candidato infinitivo
/// non è nel kg_proc, il match fallisce — niente falsi positivi su
/// sostantivi che terminano in `-o`/`-i`.
pub fn lemma_of_verb(
    word: &str,
    kg_proc: Option<&KnowledgeGraph>,
    kg: Option<&KnowledgeGraph>,
) -> Option<(String, crate::topology::grammar::Person)> {
    use crate::topology::grammar::{lemmatize, Person};

    let w = word.to_lowercase();

    // (0) PONDERAZIONE omografi nome/verbo (es. "deriva"): la grammatica CURATA
    //     batte l'inferenza semantica rumorosa. `lemmatize` può leggere una forma
    //     ambigua come imperfetto ("der-iva"→"derire", che il kg_sem attesta per
    //     rumore) quando il presente regolare dà un verbo CURATO ("derivare", IsA
    //     verbo nel kg_proc). Preferiamo il verbo attestato nel kg_proc, da
    //     qualunque interpretazione provenga — candidati + selezione, non congettura.
    if kg_proc.is_some() {
        let is_proc_verb = |c: &str| {
            kg_proc.map_or(false, |kp| is_kg_proc_isa(kp, c, "verbo"))
        };
        if let Some(r) = lemmatize(&w) {
            if is_proc_verb(&r.infinitive) {
                return Some((r.infinitive, r.person));
            }
        }
        const PRESENT0: &[(&str, Person)] = &[
            ("iamo", Person::FirstPlural), ("ate", Person::SecondPlural),
            ("ete", Person::SecondPlural), ("ite", Person::SecondPlural),
            ("ano", Person::ThirdPlural), ("ono", Person::ThirdPlural),
            ("o", Person::First), ("i", Person::Second),
            ("a", Person::Third), ("e", Person::Third),
        ];
        for (suf, person) in PRESENT0 {
            if let Some(stem) = w.strip_suffix(suf) {
                if stem.len() < 2 { continue; }
                for inf_suf in &["are", "ere", "ire"] {
                    let cand = format!("{}{}", stem, inf_suf);
                    if is_proc_verb(&cand) { return Some((cand, *person)); }
                }
            }
        }
    }

    // (1) Irregolari + suffissi noti — ma validati strutturalmente.
    //     `grammar::lemmatize` ha falsi positivi su forme regolari: es.
    //     "chiamo" (= chiamare 1sg) viene letto come `chare` 1pl perché
    //     termina in "iamo". Validiamo l'infinito proposto come VERBO
    //     (`is_verb_form`): se non lo è, scartiamo e proviamo il fallback
    //     morfologico (che produrrà "chiamare", valido).
    if let Some(r) = lemmatize(&w) {
        if kg_proc.is_none() && kg.is_none() {
            // Path test: nessun grafo per validare → fidati della morfologia.
            return Some((r.infinitive, r.person));
        }
        if is_verb_form(&r.infinitive, kg_proc, kg) {
            return Some((r.infinitive, r.person));
        }
        // falso positivo morfologico: continua al fallback
    }

    // (2) Il token stesso è già un INFINITO? (es. "essere", "andare" in
    //     un'enumerazione). Richiede la MORFOLOGIA da infinito (-are/-ere/-ire):
    //     senza questo gate, le NOMINALIZZAZIONI (tradimento/movimento/
    //     sentimento, tutte `IsA azione` nel kg_sem) venivano scambiate per
    //     verbi da `is_verb_concept`, rubando il ruolo di pivot al verbo vero.
    if (w.ends_with("are") || w.ends_with("ere") || w.ends_with("ire"))
        && is_verb_form(&w, kg_proc, kg)
    {
        return Some((w.clone(), Person::Third)); // infinito non porta persona
    }

    // (3) Fallback morfologico: prova suffissi del presente regolare.
    //     Lo stem deve avere lunghezza ≥3 per evitare match su monosillabi;
    //     il candidato infinito deve essere validato come verbo (`is_verb_form`).
    const PRESENT_SUFFIXES: &[(&str, Person)] = &[
        ("iamo", Person::FirstPlural),
        ("ate",  Person::SecondPlural),
        ("ete",  Person::SecondPlural),
        ("ite",  Person::SecondPlural),
        ("ano",  Person::ThirdPlural),
        ("ono",  Person::ThirdPlural),
        ("o",    Person::First),
        ("i",    Person::Second),
        ("a",    Person::Third),
        ("e",    Person::Third),
    ];

    for (suf, person) in PRESENT_SUFFIXES {
        if let Some(stem) = w.strip_suffix(suf) {
            // stem ≥2 cattura verbi brevi ("amo"→am→amare); la validazione
            // `is_verb_form` (kg) scarta i candidati-spazzatura, quindi
            // possiamo permetterci stem corti senza falsi positivi sui nomi.
            if stem.len() < 2 { continue; }
            for inf_suf in &["are", "ere", "ire"] {
                let candidate = format!("{}{}", stem, inf_suf);
                if is_verb_form(&candidate, kg_proc, kg) {
                    return Some((candidate, *person));
                }
            }
        }
    }

    // (3-bis) Participio passato REGOLARE come forma verbale (-ato/-uto/-ito +
    //     concordanza). "lavorato"→lavorare, "andata"→andare. KG-validato come
    //     gli altri; nel contesto post-articolo il chiamante (coverage) applica
    //     la guardia nominale ("il gelato" ≠ verbo). Gli irregolari passano da
    //     `lemmatize` (1b) sopra.
    const PARTICIPLE: &[(&[&str], &str, Person)] = &[
        (&["ato", "ata", "ati", "ate"], "are", Person::Third),
        (&["uto", "uta", "uti", "ute"], "ere", Person::Third),
        (&["ito", "ita", "iti", "ite"], "ire", Person::Third),
    ];
    for (sufs, inf_suf, person) in PARTICIPLE {
        for suf in *sufs {
            if let Some(stem) = w.strip_suffix(suf) {
                if stem.len() < 2 { continue; }
                let candidate = format!("{}{}", stem, inf_suf);
                if is_verb_form(&candidate, kg_proc, kg) {
                    return Some((candidate, *person));
                }
            }
        }
    }

    // (3-ter) Gerundio (-ando→are, -endo→ere/ire): "cercando"→cercare,
    //     "scrivendo"→scrivere. La progressiva "sto cercando" è gemella del
    //     composto "ho cercato". KG-validato.
    if let Some(stem) = w.strip_suffix("ando") {
        if stem.len() >= 2 {
            let candidate = format!("{}are", stem);
            if is_verb_form(&candidate, kg_proc, kg) {
                return Some((candidate, Person::Third));
            }
        }
    }
    if let Some(stem) = w.strip_suffix("endo") {
        if stem.len() >= 2 {
            for inf_suf in &["ere", "ire"] {
                let candidate = format!("{}{}", stem, inf_suf);
                if is_verb_form(&candidate, kg_proc, kg) {
                    return Some((candidate, Person::Third));
                }
            }
        }
    }

    // (4) Fallback FUTURO regolare. Il futuro di -are e -ere collassa sullo
    //     stesso tema (-er-): "amerà"/"temerà" → ambigui dalla sola superficie,
    //     così come "-ir-" per -ire. Generiamo i candidati e lasciamo che il KG
    //     disambigui (`is_verb_form`) — stesso principio del presente, nessuna
    //     lista. Ordine suffissi: prima i lunghi. La radice del futuro porta
    //     già l'infisso, quindi i candidati sono `stem(+a/e/i)+re`:
    //       "-erò/erai/erà/eremo/erete/eranno" → {are, ere}
    //       "-irò/irai/irà/iremo/irete/iranno" → {ire}
    use crate::topology::grammar::Person as P;
    const FUTURE_ER: &[(&str, P)] = &[
        ("eranno", P::ThirdPlural), ("eremo", P::FirstPlural), ("erete", P::SecondPlural),
        ("erò", P::First), ("erai", P::Second), ("erà", P::Third),
    ];
    const FUTURE_IR: &[(&str, P)] = &[
        ("iranno", P::ThirdPlural), ("iremo", P::FirstPlural), ("irete", P::SecondPlural),
        ("irò", P::First), ("irai", P::Second), ("irà", P::Third),
    ];
    for (suf, person) in FUTURE_ER {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() < 2 { continue; }
            for inf_suf in &["are", "ere"] {
                let candidate = format!("{}{}", stem, inf_suf);
                if is_verb_form(&candidate, kg_proc, kg) {
                    return Some((candidate, *person));
                }
            }
        }
    }
    for (suf, person) in FUTURE_IR {
        if let Some(stem) = w.strip_suffix(suf) {
            if stem.len() < 2 { continue; }
            let candidate = format!("{}ire", stem);
            if is_verb_form(&candidate, kg_proc, kg) {
                return Some((candidate, *person));
            }
        }
    }

    None
}

/// Phase 84 (2c-A): un infinito è un verbo se UN segnale lo conferma —
/// (i) kg_proc lo marca `IsA verbo` (verbi curati), oppure (ii) la sua catena
/// IsA nel kg_sem raggiunge un concetto-verbo. Il lessico NON porta POS
/// affidabili sui verbi (verificato dal vivo: POS=None su amare/pensare/…),
/// quindi la verbità è morfologia (`lemmatize`, a monte) + conferma semantica
/// (qui) — mai una lista curata di tutti i verbi italiani.
fn is_verb_form(
    infinitive: &str,
    kg_proc: Option<&KnowledgeGraph>,
    kg: Option<&KnowledgeGraph>,
) -> bool {
    if let Some(kp) = kg_proc {
        if is_kg_proc_isa(kp, infinitive, "verbo") {
            return true;
        }
    }
    is_verb_concept(infinitive, kg)
}

/// Catena IsA (1-2 hop) nel kg_sem verso un concetto-verbo. I root sono
/// concetti (non parole) — insieme piccolo e stabile, non cresce col lessico.
fn is_verb_concept(word: &str, kg: Option<&KnowledgeGraph>) -> bool {
    const VERB_ROOTS: &[&str] = &[
        "azione", "atto", "processo", "evento",
        "movimento", "accadimento", "attività", "fare",
    ];
    let Some(kg) = kg else { return false; };
    let w = word.to_lowercase();
    for p in kg.query_objects(&w, RelationType::IsA) {
        if VERB_ROOTS.contains(&p.to_lowercase().as_str()) {
            return true;
        }
        for gp in kg.query_objects(p, RelationType::IsA) {
            if VERB_ROOTS.contains(&gp.to_lowercase().as_str()) {
                return true;
            }
        }
    }
    false
}

/// Phase 80: legge la categoria del verbo (copula/percettivo/cognitivo/
/// comunicativo/denominativo/azione) dal kg_proc tramite la sua IsA chain.
///
/// Quando un verbo è in più categorie (es. "sentire IsA azione + IsA
/// percettivo"), prevale quella più specifica nel posizionamento:
/// denominativo > percettivo > cognitivo > comunicativo > copula > azione.
/// Questa priorità riflette la specificità semantica del verbo (non un
/// dispatch arbitrario): "chiamare" come denominativo è informativo,
/// "chiamare" come azione generica è una banalità.
pub fn verb_category(
    infinitive: &str,
    kg_proc: Option<&KnowledgeGraph>,
) -> Option<&'static str> {
    let kp = kg_proc?;
    let parents = kp.query_objects(infinitive, RelationType::IsA);
    const PRIORITY: &[&str] = &[
        "denominativo", "percettivo", "valutativo", "cognitivo",
        "comunicativo", "copula", "azione",
    ];
    PRIORITY.iter().find(|cat| parents.contains(cat)).copied()
}

/// Phase 80: una parola è "funzione" se il kg_proc la classifica come
/// pronome, articolo, preposizione, marcatore, congiunzione, oppure
/// `IsA copula`. La distinzione vive nei dati: aggiungere/togliere
/// parole-funzione è curation.
pub fn is_kg_proc_function_word(word: &str, kg_proc: Option<&KnowledgeGraph>) -> bool {
    let Some(kp) = kg_proc else { return false; };
    let w = word.to_lowercase();
    let parents = kp.query_objects(&w, RelationType::IsA);
    for p in &parents {
        let p_lc = p.to_lowercase();
        if matches!(p_lc.as_str(),
            "pronome" | "articolo" | "preposizione" |
            "marcatore" | "congiunzione" | "copula" |
            "determinante" |
            // Phase 86+ (circostanza): un avverbio modifica, non è mai
            // soggetto/oggetto/predicato → si salta nella selezione del predicato
            // ("sono sempre felice" → predicato=felice, non "sempre"). Allinea a
            // is_function_word_simple, che già lo include.
            "avverbio" |
            // Un'interiezione è un atto espressivo a sé, mai un argomento.
            "interiezione") {
            return true;
        }
    }
    false
}

/// Helper: verifica `subject IsA target` 1-hop nel kg_proc.
pub(crate) fn is_kg_proc_isa(kg_proc: &KnowledgeGraph, subject: &str, target: &str) -> bool {
    kg_proc.query_objects(subject, RelationType::IsA)
        .iter().any(|p| p.to_lowercase() == target.to_lowercase())
}

/// Phase 73: rileva una presentazione di nome del parlante.
/// Pattern riconosciuti:
///   "mi chiamo X"       → Some("X")
///   "io mi chiamo X"    → Some("X")
///   "il mio nome è X"   → Some("X")
///   "sono X" (X è una parola sconosciuta al KG e al lessico)  → Some("X")
///
/// X deve essere una parola "non strutturale" — non in lessico stabile,
/// non in KG, lunghezza ≥ 3. Questo riduce falsi positivi tipo
/// "sono triste" → "triste" (che è in lessico ed è un aggettivo emotivo).
pub fn detect_name_introduction(
    raw_words: &[String],
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
) -> Option<String> {
    if raw_words.len() < 2 { return None; }
    let words: Vec<&str> = raw_words.iter().map(|w| w.as_str()).collect();

    // Pattern "mi chiamo X" / "io mi chiamo X" / "chiamo X" (chiamarsi 1a sg)
    if let Some(p) = words.iter().position(|&w| w == "chiamo") {
        // Verifica che sia preceduto da "mi" o sia in posizione iniziale dopo "io"
        let preceded_by_mi = p > 0 && (words[p - 1] == "mi" ||
            (p >= 2 && words[p - 2] == "io" && words[p - 1] == "mi"));
        let initial_or_after_io = p == 0 || (p == 1 && words[0] == "io");
        if preceded_by_mi || initial_or_after_io {
            // Il nome è la prima parola-contenuto dopo "chiamo"
            for &w in words.iter().skip(p + 1) {
                if is_likely_proper_name(w, lexicon, kg) {
                    return Some(w.to_string());
                }
            }
        }
    }

    // Pattern "il mio nome è X"
    if let Some(p) = words.iter().position(|&w| w == "nome") {
        if p + 1 < words.len() && (words[p + 1] == "è" || words[p + 1] == "e") {
            for &w in words.iter().skip(p + 2) {
                if is_likely_proper_name(w, lexicon, kg) {
                    return Some(w.to_string());
                }
            }
        }
    }

    None
}

/// Euristica: una parola è probabilmente un nome proprio se NON è nel
/// lessico stabile e NON è nel KG e ha lunghezza ≥ 3 e non è una
/// function word.
fn is_likely_proper_name(
    word: &str,
    lexicon: &Lexicon,
    kg: Option<&KnowledgeGraph>,
) -> bool {
    if word.len() < 3 { return false; }
    if lexicon.is_function_word(word) { return false; }
    // Salta articoli e preposizioni
    if matches!(word, "un" | "una" | "uno" | "il" | "la" | "lo" | "i" | "gli" | "le") {
        return false;
    }
    // Se è già un sostantivo/aggettivo conosciuto al KG, probabilmente non è un nome proprio
    if let Some(kg) = kg {
        if !kg.query_objects(word, RelationType::IsA).is_empty() {
            return false;
        }
    }
    // Se è nel lessico con stabilità alta, probabilmente non è un nome
    if let Some(pat) = lexicon.get(word) {
        if pat.stability > 0.5 && pat.exposure_count > 5 {
            return false;
        }
    }
    true
}

/// Phase 80: verifica se una parola denota uno stato interno del soggetto
/// (emozione, sentimento, sensazione, bisogno) leggendo strutturalmente la
/// rete di relazioni IsA / Has nel KG semantico.
///
/// Una parola è "stato interno" se:
///   (a) la sua catena IsA (1-2 hop) raggiunge uno dei super-tipi semantici
///       {emozione, sentimento, stato_d_animo, sensazione, affetto, umore,
///        bisogno, dolore, sofferenza}, oppure
///   (b) ha una relazione `Has bisogno` / `Has sofferenza` / `Has mancanza`
///       — segnale strutturale che il significante porta un bisogno (es. il
///       KG codifica "fame Has bisogno" anziché "fame IsA bisogno": gli
///       stati corporei *hanno* bisogni, non li *sono*).
///
/// I super-tipi sono concetti (non parole italiane) — l'insieme è piccolo
/// e stabile, non cresce con il vocabolario. Qualunque significante che
/// risale a uno di essi (es. "fame" → "bisogno", "buio" → niente di emotivo)
/// emerge come stato interno o no.
pub(crate) fn is_inner_state(word: &str, lexicon: &Lexicon, kg: Option<&KnowledgeGraph>) -> bool {
    const INNER_STATE_ROOTS: &[&str] = &[
        "emozione", "sentimento", "stato_d_animo", "sensazione",
        "affetto", "umore", "bisogno", "dolore", "sofferenza",
    ];
    const HAS_SIGNALS: &[&str] = &[
        "bisogno", "sofferenza", "mancanza", "dolore",
    ];

    let Some(kg) = kg else {
        // Senza KG: fallback al lessico — POS=Adjective con alta Valenza è
        // segnale debole di parola valutativa (es. "triste" senza KG).
        // Usato solo dai test che girano senza KG.
        if let Some(pat) = lexicon.get(word) {
            if matches!(pat.pos, Some(crate::topology::grammar::PartOfSpeech::Adjective))
                && pat.stability > 0.3
                && pat.signature.get(crate::topology::primitive::Dim::Valenza) > 0.6
            {
                return true;
            }
        }
        return false;
    };

    let w = word.to_lowercase();

    // (a) IsA chain 1-2 hop verso i super-tipi.
    let direct_parents: Vec<&str> = kg.query_objects(&w, RelationType::IsA);
    for p in &direct_parents {
        if INNER_STATE_ROOTS.contains(&p.to_lowercase().as_str()) {
            return true;
        }
        // 2-hop: il genitore IsA un super-tipo? (es. fame → bisogno-fisiologico → bisogno)
        for gp in kg.query_objects(p, RelationType::IsA) {
            if INNER_STATE_ROOTS.contains(&gp.to_lowercase().as_str()) {
                return true;
            }
        }
    }

    // (b) Has signals: parola che porta un bisogno/sofferenza è stato interno.
    let has_targets: Vec<&str> = kg.query_objects(&w, RelationType::Has);
    for t in &has_targets {
        if HAS_SIGNALS.contains(&t.to_lowercase().as_str()) {
            return true;
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
            None,
        );
        assert_eq!(r.act, InputAct::Declaration);
    }

    #[test]
    fn test_intensity_from_delta() {
        let lex = lex();
        let kb = kb_with_anchors();
        let delta = vec![(58u32, 0.6f64), (32u32, 0.4f64), (33u32, 0.2f64)];
        let r = read_input(&[], "", &delta, &kb, &lex, None, None);
        // avg top-3 assoluti = (0.6 + 0.4 + 0.2) / 3 ≈ 0.4
        assert!((r.intensity - 0.4).abs() < 0.01,
            "intensity attesa ~0.4, ottenuta {}", r.intensity);
    }

    #[test]
    fn test_no_anchors_fallback() {
        // Senza ancore concettuali, solo `?` e Declaration funzionano
        let lex = lex();
        let kb = KnowledgeBase::new(); // vuota
        let r = read_input(&["ciao".to_string()], "ciao", &empty_delta(), &kb, &lex, None, None);
        // Senza ancora Social, "ciao" → Declaration (non riconosciuto)
        assert_eq!(r.act, InputAct::Declaration,
            "senza KnowledgeBase, ciao non è riconoscibile come saluto");
        let r2 = read_input(&["cosa".to_string()], "cosa succede?", &empty_delta(), &kb, &lex, None, None);
        assert_eq!(r2.act, InputAct::Question,
            "senza KB, `?` mantiene Question come fallback sintattico");
    }
}
