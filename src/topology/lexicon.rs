/// Lessico Topologico — Le parole come pattern nel complesso simpliciale.
///
/// Una parola non "corrisponde a un frattale". E un pattern di attivazione
/// nello spazio 8D che si consolida per esposizione ripetuta.
/// Parole simili condividono facce (come i frattali).

use std::collections::HashMap;
use crate::topology::primitive::{PrimitiveCore, Dim};
use crate::topology::fractal::FractalId;
use crate::topology::grammar::{self, PartOfSpeech};

/// Tipo di operatore strutturale.
/// Gli operatori cambiano il SEGNO delle relazioni tra parole:
/// si + no + quanto = X
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatorKind {
    /// Affermatori: e, come, anche, simile... → co-occorrenza affermata
    Affirm,
    /// Negatori: non, no, senza, mai... → co-occorrenza negata
    Negate,
    /// Quantificatori: molto=1.3, poco=0.5... → scala l'effetto
    Quantify(f64),
}

/// Un pattern lessicale: la firma topologica di una parola.
#[derive(Debug, Clone)]
pub struct WordPattern {
    /// La parola (lowercase)
    pub word: String,
    /// Firma 8D: come questa parola deforma il campo
    pub signature: PrimitiveCore,
    /// Affinita con i frattali [0.0, 1.0]
    pub fractal_affinities: HashMap<FractalId, f64>,
    /// Quante volte e stata incontrata
    pub exposure_count: u64,
    /// Quanto e stabile la firma (cresce con le esposizioni)
    pub stability: f64,
    /// Contesto di apprendimento: con quali altre parole e stata vista (neutre)
    pub co_occurrences: HashMap<String, u64>,
    /// Co-occorrenze in contesto di negazione ("non", "senza", "mai")
    pub co_negated: HashMap<String, u64>,
    /// Co-occorrenze in contesto esplicitamente affermato ("come", "simile", "uguale")
    /// Usate come denominatore nel rapporto di fase: neg/(neg+affirm)
    pub co_affirmed: HashMap<String, u64>,
    /// Categoria grammaticale (rilevata dal lemmatizzatore o euristica suffisso).
    /// None = categoria non determinata. Usata da VerbCandidate/StateVerb.
    pub pos: Option<PartOfSpeech>,
}

impl WordPattern {
    /// Crea un pattern nuovo, instabile — la parola e appena stata incontrata.
    pub fn new_unknown(word: &str) -> Self {
        Self {
            word: word.to_lowercase(),
            signature: PrimitiveCore::neutral(), // parte dal centro, non sa nulla
            fractal_affinities: HashMap::new(),
            exposure_count: 1,
            stability: 0.0,
            co_occurrences: HashMap::new(),
            co_negated: HashMap::new(),
            co_affirmed: HashMap::new(),
            pos: None,
        }
    }

    /// Crea un pattern nuovo a partire dal contesto.
    /// Firma iniziale: 10% neutrale + 90% contesto.
    /// Il 90% contesto piazza la firma nel bacino frattale corretto.
    /// Crea un pattern da contesto: la firma parte vicina al contesto.
    /// Le affinita frattali sono VUOTE — saranno calcolate geometricamente
    /// dalla firma 8D, non copiate dal contesto (niente statistiche ereditarie).
    /// Applica perturbazione per differenziare parole con contesti simili.
    pub fn new_from_context(word: &str, context_sig: &PrimitiveCore, _context_aff: &[(FractalId, f64)]) -> Self {
        // La firma iniziale è quella del contesto in cui la parola è apparsa per la prima volta.
        // Nessuna perturbazione artificiale basata sull'encoding UTF-8 della parola:
        // il significato deve emergere esclusivamente dalle esposizioni nel campo (perturb_towards),
        // oppure dalla derivazione KG (rederive-signatures) se la parola è nel grafo.
        let mut sig = PrimitiveCore::neutral();
        sig.perturb_towards(context_sig, 0.90);

        Self {
            word: word.to_lowercase(),
            signature: sig,
            fractal_affinities: HashMap::new(), // calcolate dopo da recompute_affinities
            exposure_count: 1,
            stability: 0.0,
            co_occurrences: HashMap::new(),
            co_negated: HashMap::new(),
            co_affirmed: HashMap::new(),
            pos: None,
        }
    }

    /// Crea un pattern noto, con firma predefinita.
    pub fn new_known(word: &str, signature: PrimitiveCore, affinities: Vec<(FractalId, f64)>) -> Self {
        let fractal_affinities: HashMap<FractalId, f64> = affinities.into_iter().collect();
        Self {
            word: word.to_lowercase(),
            signature,
            fractal_affinities,
            exposure_count: 10, // parte gia "vista"
            stability: 0.6,
            co_occurrences: HashMap::new(),
            co_negated: HashMap::new(),
            co_affirmed: HashMap::new(),
            pos: None,
        }
    }

    /// Esponi la parola a un contesto: aggiorna la firma per avvicinamento.
    /// context_signature e la firma media del contesto in cui appare.
    /// Le affinita frattali NON vengono accumulate — vanno ricalcolate
    /// dopo con `recompute_affinities()` dalla firma aggiornata.
    pub fn expose(&mut self, context_signature: &PrimitiveCore, _context_affinities: &[(FractalId, f64)]) {
        self.exposure_count += 1;

        // Forza di apprendimento: alta all'inizio, diminuisce con la stabilita.
        // 0.35 per parole nuove: ogni esposizione muove la firma sensibilmente
        // verso il contesto. Le prime esposizioni sono le piu formative.
        let learning_rate = (1.0 - self.stability) * 0.35;

        // La firma si avvicina al contesto
        self.signature.perturb_towards(context_signature, learning_rate);

        // Le affinita frattali NON si aggiornano qui con medie mobili.
        // Sono DERIVATE dalla firma 8D — calcolate geometricamente, non statisticamente.
        // Vengono ricalcolate dopo ogni batch di esposizioni via recompute_affinities().

        // La stabilita cresce logaritmicamente
        self.stability = (1.0 - 1.0 / (1.0 + self.exposure_count as f64 * 0.1)).min(0.95);
    }

    /// Ricalcola le affinita frattali dalla firma 8D corrente.
    /// Le affinita sono una PROIEZIONE geometrica, non una media statistica.
    /// Ogni frattale ha un'affinita che dipende solo dalla posizione 8D della parola
    /// e dalle dimensioni fisse del frattale.
    pub fn recompute_affinities(&mut self, all_affinities: &[(FractalId, f64)]) {
        self.fractal_affinities.clear();
        for &(fid, aff) in all_affinities {
            // Tutte le affinita — nessuna soglia, nessun filtro.
            // Il campo e continuo: ogni parola vive in tutti i frattali.
            self.fractal_affinities.insert(fid, aff);
        }
    }

    /// Registra co-occorrenza affermata/neutra con un'altra parola.
    pub fn register_co_occurrence(&mut self, other_word: &str) {
        *self.co_occurrences.entry(other_word.to_lowercase()).or_insert(0) += 1;
    }

    /// Registra co-occorrenza NEGATA con un'altra parola.
    /// Es. "gioia non e tristezza" → gioia.co_negated["tristezza"] += 1
    pub fn register_co_negation(&mut self, other_word: &str) {
        *self.co_negated.entry(other_word.to_lowercase()).or_insert(0) += 1;
    }

    /// Registra co-occorrenza AFFERMATA con un'altra parola.
    /// Es. "gioia come felicita" → gioia.co_affirmed["felicita"] += 1
    /// Queste sono le uniche che entrano nel denominatore della fase.
    pub fn register_co_affirmation(&mut self, other_word: &str) {
        *self.co_affirmed.entry(other_word.to_lowercase()).or_insert(0) += 1;
    }

    /// La parola e sufficientemente stabile da avere un significato?
    pub fn is_stable(&self) -> bool {
        self.stability > 0.3 && self.exposure_count >= 5
    }

    /// Affinita dominante: il frattale piu affine.
    pub fn dominant_fractal(&self) -> Option<(FractalId, f64)> {
        self.fractal_affinities.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(&id, &aff)| (id, aff))
    }

    /// Forza di perturbazione che questa parola esercita sul campo.
    /// Parole instabili perturbano debolmente (il sistema "sente" qualcosa ma non sa cosa).
    pub fn perturbation_strength(&self) -> f64 {
        // Parole note perturbano forte, parole ignote debolmente
        let base = if self.is_stable() { 0.6 } else { 0.1 };
        base * self.stability.max(0.05)
    }
}

/// Il lessico: l'insieme dei pattern appresi.
#[derive(Debug)]
pub struct Lexicon {
    /// Parole note: word → pattern
    patterns: HashMap<String, WordPattern>,
    /// Parole funzionali (da ignorare o trattare come connettivi)
    pub function_words: Vec<String>,
}

/// Normalizza un token grezzo in una parola pulita pronta per il lessico.
///
/// Regole applicate:
/// - Contrazioni apostrofo (`all'acqua`, `l'essere`): prende il segmento DOPO l'apostrofo
/// - Strip punteggiatura finale/iniziale: `:`, `.`, `,`, `!`, `?`, `;`, `(`, `)`, `"`, `«`, `»`, `—`, `–`
/// - Restituisce `None` se il risultato è vuoto o senza caratteri alfabetici
pub fn clean_token(raw: &str) -> Option<String> {
    // Contrazioni con apostrofo → prende la parte contenuto (dopo l'ultimo apostrofo)
    let w = if let Some(pos) = raw.rfind('\'') {
        &raw[pos + '\''.len_utf8()..]
    } else {
        raw
    };
    // Strip punteggiatura esterna
    let w = w.trim_matches(|c: char| ":.,'!?;()\"«»—–\u{2014}\u{2013}".contains(c));
    // Scarta se vuota o senza lettere
    if w.is_empty() || !w.chars().any(|c| c.is_alphabetic()) {
        return None;
    }
    Some(w.to_lowercase())
}

impl Lexicon {
    /// Crea un lessico vuoto.
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            function_words: vec![
                // Articoli
                "il", "lo", "la", "i", "gli", "le", "un", "uno", "una",
                // Preposizioni semplici
                "di", "a", "da", "in", "con", "su", "per", "tra", "fra",
                // Congiunzioni semplici (senza significato direzionale)
                "o", "ma", "che", "se",
                // Congiunzioni temporali e interrogativi grammaticali puri
                // (appaiono in costrutti tipo "quando non X" creando falsi positivi negli operatori)
                "quando", "cosa", "chi",
                // "e" (copula/congiunzione): rimossa dagli operatori per evitare cancellazione
                // della negazione in "X non e Y". Torna come function_word.
                "e",
                // "non" RIMOSSO: ora e un operatore strutturale (negatore)
                // Preposizioni articolate
                "del", "al", "dal", "nel", "sul",
                "dello", "alla", "della", "nella", "dallo",
                "dei", "agli", "dai", "nei", "sui",
                "delle", "alle", "dalle", "nelle", "sulle",
                // I pronomi NON sono function_words — hanno peso semantico.
                // "io" e il soggetto dell'entita, "tu" e l'altro, "noi" e l'insieme.
                // Vivono nel lessico come parole cardinali con firme 8D proprie.
                // Dimostrativi e articoli partitivi
                "questo", "quello", "quella", "questi", "quelli",
            ].into_iter().map(String::from).collect(),
        }
    }

    /// Crea un lessico con il vocabolario completo (adulto).
    pub fn bootstrap() -> Self {
        let mut lex = Self::new();
        lex.seed_bootstrap_vocabulary();
        lex
    }

    /// Crea un lessico cardinale: ~36 parole native, il minimo per esistere.
    /// L'entita parte da qui e impara il resto tramite insegnamento.
    pub fn bootstrap_cardinal() -> Self {
        let mut lex = Self::new();
        lex.seed_cardinal_vocabulary();
        lex
    }

    /// Parola nota?
    pub fn knows(&self, word: &str) -> bool {
        self.patterns.contains_key(&word.to_lowercase())
    }

    /// E una parola funzionale?
    pub fn is_function_word(&self, word: &str) -> bool {
        self.function_words.contains(&word.to_lowercase())
    }

    /// Classifica una parola come operatore strutturale.
    /// Gli operatori non sono parole da ignorare ne parole semantiche:
    /// sono il sistema nervoso che cambia il SEGNO delle relazioni.
    pub fn classify_operator(word: &str) -> Option<OperatorKind> {
        match word {
            // Affermatori: legano parole in risonanza (simile, analogo)
            // "e" ESCLUSO: come copula ("non e") causa cancellazione falsa della negazione.
            // "e" e trattato come function_word — non contribuisce alla fase.
            "come" | "anche" | "simile" | "uguale" | "sia" | "pure" | "stesso" | "anzi" =>
                Some(OperatorKind::Affirm),
            // Negatori: invertono la relazione
            "non" | "no" | "senza" | "mai" | "nessuno" | "niente" | "nulla" | "mica"
            | "né" | "neanche" | "neppure" | "affatto" | "giammai" =>
                Some(OperatorKind::Negate),
            // Quantificatori: scalano l'effetto
            "molto" | "molta" | "molti" | "molte" => Some(OperatorKind::Quantify(1.3)),
            "poco" | "poca" | "pochi" | "poche" => Some(OperatorKind::Quantify(0.5)),
            "quasi" => Some(OperatorKind::Quantify(0.7)),
            "troppo" | "troppa" | "troppi" | "troppe" => Some(OperatorKind::Quantify(1.5)),
            "appena" => Some(OperatorKind::Quantify(0.3)),
            "abbastanza" => Some(OperatorKind::Quantify(0.8)),
            "tanto" | "tanta" | "tanti" | "tante" => Some(OperatorKind::Quantify(1.2)),
            "piu" | "più" => Some(OperatorKind::Quantify(1.1)),
            "meno" => Some(OperatorKind::Quantify(0.6)),
            _ => None,
        }
    }

    /// Ottieni il pattern di una parola (se nota).
    pub fn get(&self, word: &str) -> Option<&WordPattern> {
        self.patterns.get(&word.to_lowercase())
    }

    /// Ottieni il pattern mutabile di una parola.
    pub fn get_mut(&mut self, word: &str) -> Option<&mut WordPattern> {
        self.patterns.get_mut(&word.to_lowercase())
    }

    /// Processa un input: per ogni parola, aggiorna il lessico.
    /// Le affinita frattali sono CALCOLATE GEOMETRICAMENTE dalla firma 8D,
    /// non accumulate con medie statistiche. Ogni parola vive in tutti i frattali
    /// con intensita diversa — come un punto nello spazio colora ogni campo.
    ///
    /// SISTEMA OPERATORI (si + no + quanto = X):
    /// Gli operatori strutturali vengono rilevati con le loro posizioni e usati
    /// per registrare co-occorrenze AFFERMATE o NEGATE tra le parole contenuto.
    pub fn process_input(&mut self, input: &str, registry: &crate::topology::fractal::FractalRegistry) -> Vec<WordActivation> {
        // Fase 0: tokenizza tutto con posizioni, identificando operatori e parole contenuto.
        // clean_token() rimuove punteggiatura finale e gestisce contrazioni apostrofo.
        let all_tokens: Vec<String> = input.split_whitespace()
            .filter_map(|w| clean_token(w))
            .filter(|w| w.len() > 1)
            .collect();

        // Identifica operatori e le loro posizioni nel token stream
        let operator_positions: Vec<(usize, OperatorKind)> = all_tokens.iter().enumerate()
            .filter_map(|(pos, token)| {
                Self::classify_operator(token).map(|kind| (pos, kind))
            })
            .collect();

        // Parole contenuto: non-operatori e non-function_words (con posizione originale)
        let content_words: Vec<(usize, String)> = all_tokens.iter().enumerate()
            .filter(|(_, w)| !self.is_function_word(w) && Self::classify_operator(w).is_none())
            .map(|(pos, w)| (pos, w.clone()))
            .collect();

        let words: Vec<String> = content_words.iter().map(|(_, w)| w.clone()).collect();

        if words.is_empty() {
            return Vec::new();
        }

        // Fase 1: calcola il contesto come OTTIMIZZAZIONE GEOMETRICA.
        // NON media statistica — trova il punto 8D che massimizza le affinita
        // con i frattali co-attivati dalle parole note. Zero averaging.
        let known_sigs: Vec<&WordPattern> = words.iter()
            .filter_map(|w| self.patterns.get(w))
            .collect();

        let context_sig = if known_sigs.is_empty() {
            PrimitiveCore::neutral()
        } else {
            // Raccogli frattali attivati dalle parole note (pesati per stabilita)
            let mut fractal_activation: HashMap<FractalId, f64> = HashMap::new();
            for pat in &known_sigs {
                let weight = 1.0 / (1.0 + (pat.exposure_count as f64).ln());
                for (&fid, &aff) in &pat.fractal_affinities {
                    if aff > 0.3 {
                        let entry = fractal_activation.entry(fid).or_insert(0.0);
                        *entry = (*entry).max(aff * weight);
                    }
                }
            }

            // Gradient ascent: trova il punto che massimizza le affinita con i frattali attivi
            Self::compute_optimal_context_point(&fractal_activation, registry)
        };

        // Fase 2: per ogni parola, aggiorna o crea pattern.
        // Lemmatizzazione: "correvo" → "correre", "affermavo" → "affermare".
        // Il lessico converge automaticamente agli infiniti senza toccare le lezioni.
        // Le affinita frattali NON vengono passate — saranno calcolate dalla firma.
        let canonical_words: Vec<(String, Option<PartOfSpeech>)> = words.iter()
            .map(|word| {
                match grammar::lemmatize(word) {
                    Some(lemma) => (lemma.infinitive, Some(PartOfSpeech::Verb)),
                    None => {
                        // Se non e forma coniugata, controlla se e gia un infinito
                        let pos = grammar::detect_pos_from_word(word);
                        (word.clone(), pos)
                    }
                }
            })
            .collect();

        for (canonical, detected_pos) in &canonical_words {
            if !self.patterns.contains_key(canonical.as_str()) {
                let mut pattern = WordPattern::new_from_context(canonical, &context_sig, &[]);
                pattern.pos = detected_pos.clone();
                self.patterns.insert(canonical.clone(), pattern);
            } else {
                if let Some(pat) = self.patterns.get_mut(canonical.as_str()) {
                    pat.expose(&context_sig, &[]);
                    // Aggiorna POS se ancora non determinato
                    if pat.pos.is_none() {
                        pat.pos = detected_pos.clone();
                    }
                }
            }
        }

        // Fase 2b: ricalcola affinita GEOMETRICAMENTE dalla firma aggiornata.
        for (canonical, _) in &canonical_words {
            if let Some(pat) = self.patterns.get_mut(canonical.as_str()) {
                let affinities = registry.all_affinities(&pat.signature);
                pat.recompute_affinities(&affinities);
            }
        }

        // Forma canonica parallela a content_words (posizioni originali, parole canoniche).
        let canonical_content_words: Vec<(usize, String)> = content_words.iter()
            .zip(canonical_words.iter())
            .map(|((pos, _), (canon, _))| (*pos, canon.clone()))
            .collect();

        // Fase 3: registra co-occorrenze con polarita operatore.
        // Per ogni coppia di parole contenuto, analizza gli operatori TRA le loro
        // posizioni per determinare il SEGNO della relazione (affermata o negata).
        // Le posizioni sono nell'originale; le parole nelle co-occorrenze sono canoniche.
        for i in 0..canonical_content_words.len() {
            for j in 0..canonical_content_words.len() {
                if i == j { continue; }
                let (pos_i, ref word_i) = canonical_content_words[i];
                let (pos_j, ref word_j) = canonical_content_words[j];

                let (min_pos, max_pos) = if pos_i < pos_j {
                    (pos_i, pos_j)
                } else {
                    (pos_j, pos_i)
                };

                let mut has_negator = false;
                let mut has_affirmer = false;
                let mut quantifier_scale = 1.0_f64;

                for &(op_pos, ref kind) in &operator_positions {
                    if op_pos > min_pos && op_pos < max_pos {
                        // Operatore tra le due parole: modifica il segno o scala
                        match kind {
                            OperatorKind::Negate => has_negator = true,
                            OperatorKind::Affirm => has_affirmer = true,
                            OperatorKind::Quantify(scale) => quantifier_scale *= scale,
                        }
                    } else if let OperatorKind::Quantify(scale) = kind {
                        // Quantificatore adiacente a una delle due parole
                        if op_pos + 1 == pos_i || op_pos + 1 == pos_j
                            || (op_pos > 0 && (op_pos - 1 == pos_i || op_pos - 1 == pos_j))
                        {
                            quantifier_scale *= scale;
                        }
                    }
                }

                // Numero di registrazioni (quantificatore scala l'intensita)
                let count = (quantifier_scale.round() as u64).max(1);

                if has_negator {
                    // Negazione VINCE sempre — anche se c'e un affermatore ("non e")
                    // "gioia non e tristezza" → opposizione
                    if let Some(pat) = self.patterns.get_mut(word_i.as_str()) {
                        for _ in 0..count {
                            pat.register_co_negation(word_j);
                        }
                    }
                } else if has_affirmer {
                    // Affermazione esplicita senza negazione ("gioia come felicita")
                    // Entra nel denominatore della fase come co_affirmed
                    if let Some(pat) = self.patterns.get_mut(word_i.as_str()) {
                        for _ in 0..count {
                            pat.register_co_affirmation(word_j);
                        }
                    }
                } else {
                    // Contesto neutro — registra come co-occorrenza ordinaria
                    // (usata per topologia e cosine similarity, NON per il rapporto di fase)
                    if let Some(pat) = self.patterns.get_mut(word_i.as_str()) {
                        for _ in 0..count {
                            pat.register_co_occurrence(word_j);
                        }
                    }
                }
            }
        }

        // Fase 3b: determina quali parole sono sotto negazione.
        // Regola: una parola contenuto alla posizione P è negata se esiste un
        // operatore Negate a qualsiasi posizione Q < P nel token stream.
        // Questo cattura i pattern più comuni dell'italiano:
        //   "non paura"      → paura.negated = true
        //   "non ho paura"   → avere.negated = true, paura.negated = true
        //   "ho paura"       → nessuna negazione
        //   "ho paura, non gioia" → gioia.negated = true, paura non negata
        // Limite noto: "non X ma Y" — anche Y viene negata (over-negation).
        // Accettabile per ora: il beneficio supera il costo dei casi rari.
        let word_negated: Vec<bool> = canonical_content_words.iter()
            .map(|(pos, _)| {
                operator_positions.iter().any(|(op_pos, kind)| {
                    matches!(kind, OperatorKind::Negate) && *op_pos < *pos
                })
            })
            .collect();

        // Fase 4: genera attivazioni con affinita calcolate (usa forme canoniche)
        let mut activations = Vec::new();
        for (i, (canonical, _)) in canonical_words.iter().enumerate() {
            let word = canonical;
            if let Some(pat) = self.patterns.get(word.as_str()) {
                let affinities: Vec<(FractalId, f64)> = pat.fractal_affinities.iter()
                    .filter(|(_, &v)| v > 0.05)
                    .map(|(&k, &v)| (k, v))
                    .collect();
                let negated = word_negated.get(i).copied().unwrap_or(false);
                activations.push(WordActivation {
                    word: word.clone(),
                    signature: pat.signature,
                    affinities,
                    strength: pat.perturbation_strength(),
                    is_known: pat.is_stable(),
                    negated,
                });
            }
        }

        activations
    }

    /// Calcola il punto ottimale del contesto tramite gradient ascent.
    /// Trova il punto 8D che MASSIMIZZA le affinita con i frattali attivi.
    /// Zero media statistica — solo ottimizzazione geometrica.
    fn compute_optimal_context_point(
        fractal_activation: &HashMap<FractalId, f64>,
        registry: &crate::topology::fractal::FractalRegistry,
    ) -> PrimitiveCore {
        use crate::topology::primitive::Dim;

        if fractal_activation.is_empty() {
            return PrimitiveCore::neutral();
        }

        // Parte dal neutrale e converge verso il punto ottimale
        let mut point = PrimitiveCore::neutral();
        let learning_rate = 0.05;
        let iterations = 50;

        for _ in 0..iterations {
            // Calcola gradiente: direzione che aumenta le affinita
            let mut gradient = [0.0; 8];

            for (&fid, &activation) in fractal_activation {
                if let Some(fractal) = registry.get(fid) {
                    // Per ogni dimensione fissa del frattale, sposta il punto verso quel valore
                    for (dim_idx, target_val) in fractal.fixed_dims() {
                        let current_val = point.get(dim_idx);
                        let diff = target_val - current_val;
                        // Peso il gradiente per l'attivazione del frattale
                        gradient[dim_idx.index()] += diff * activation;
                    }
                }
            }

            // Normalizza gradiente
            let grad_norm: f64 = gradient.iter().map(|g| g * g).sum::<f64>().sqrt();
            if grad_norm < 0.001 {
                break; // Convergenza
            }

            // Step del gradiente
            for i in 0..8 {
                let dim = Dim::ALL[i];
                let current = point.get(dim);
                let new_val = current + (gradient[i] / grad_norm.max(0.001)) * learning_rate;
                point.set(dim, new_val.clamp(0.0, 1.0));
            }
        }

        point
    }

    /// Numero di parole nel lessico.
    pub fn word_count(&self) -> usize {
        self.patterns.len()
    }

    /// Iteratore su tutte le parole nel lessico.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &WordPattern)> {
        self.patterns.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Parole piu stabili.
    pub fn most_stable(&self, n: usize) -> Vec<&WordPattern> {
        let mut sorted: Vec<&WordPattern> = self.patterns.values().collect();
        sorted.sort_by(|a, b| b.stability.partial_cmp(&a.stability).unwrap());
        sorted.into_iter().take(n).collect()
    }

    /// Parole simili a una data (per distanza 8D).
    pub fn similar_words(&self, word: &str, n: usize) -> Vec<(&str, f64)> {
        let target = match self.patterns.get(&word.to_lowercase()) {
            Some(p) => &p.signature,
            None => return Vec::new(),
        };

        let mut distances: Vec<(&str, f64)> = self.patterns.iter()
            .filter(|(w, _)| w.as_str() != word.to_lowercase())
            .map(|(w, p)| (w.as_str(), target.distance(&p.signature)))
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        distances.into_iter().take(n).collect()
    }

    /// Parole pronte per la promozione a sotto-frattale.
    /// Una parola e promovibile se: stabilita > 0.7, esposizioni >= 15,
    /// e ha un'affinita dominante > 0.5 con un frattale specifico.
    pub fn promotable_words(&self) -> Vec<&WordPattern> {
        self.patterns.values()
            .filter(|p| {
                p.stability > 0.7
                    && p.exposure_count >= 15
                    && p.dominant_fractal().map_or(false, |(_, aff)| aff > 0.5)
            })
            .collect()
    }

    /// Inserisce un pattern completo nel lessico, sovrascrivendo quello esistente.
    /// Usato dal restore per ripristinare lo stato esatto salvato.
    pub fn insert_pattern(&mut self, word: &str, pattern: WordPattern) {
        self.patterns.insert(word.to_lowercase(), pattern);
    }

    /// Rimuove una parola dal lessico. Ritorna true se la parola era presente.
    pub fn remove_word(&mut self, word: &str) -> bool {
        self.patterns.remove(&word.to_lowercase()).is_some()
    }

    /// Iteratore su tutti i pattern del lessico.
    pub fn patterns_iter(&self) -> impl Iterator<Item = (&String, &WordPattern)> {
        self.patterns.iter()
    }

    /// Assegna il tag POS a tutte le parole che non ce l'hanno ancora,
    /// usando l'euristica sui suffissi (-are/-ere/-ire → Verb).
    /// Chiamato dopo restore_lexicon() per taggare il lessico esistente.
    pub fn tag_pos_from_forms(&mut self) {
        let words: Vec<String> = self.patterns.keys().cloned().collect();
        for word in &words {
            if let Some(pat) = self.patterns.get_mut(word) {
                if pat.pos.is_none() {
                    pat.pos = grammar::detect_pos_from_word(word);
                }
            }
        }
    }

    /// Segna una parola come gia promossa a frattale (evita duplicazioni).
    pub fn mark_promoted(&mut self, word: &str) {
        if let Some(pat) = self.patterns.get_mut(&word.to_lowercase()) {
            // Imposta stabilita a 1.0 e blocca ulteriori promozioni
            // cambiando l'esposizione a un valore sentinella
            pat.stability = 1.0;
        }
    }

    /// Applica firme 8D calibrate a mano per le parole core del lessico.
    /// Sovrascrive le firme assegnate da vary() o dal contesto di apprendimento.
    /// Solo le parole gia presenti nel lessico vengono modificate.
    /// Chiamare PRIMA di recompute_all_word_affinities() per propagare le firme corrette.
    ///
    /// 8D: [Confine, Valenza, Intensita, Definizione, Complessita, Permanenza, Agency, Tempo]
    pub fn apply_curated_signatures(&mut self) {
        // Dimensioni: [Confine, Valenza, Intensita, Definizione, Complessita, Permanenza, Agency, Tempo]
        let curated: &[(&str, [f64; 8])] = &[
            // ── PRONOMI ──────────────────────────────────────────────────────
            ("io",         [0.95, 0.50, 0.65, 0.90, 0.50, 0.75, 0.80, 0.40]),
            ("me",         [0.85, 0.50, 0.55, 0.80, 0.45, 0.65, 0.55, 0.40]),
            ("mio",        [0.90, 0.60, 0.45, 0.85, 0.35, 0.80, 0.65, 0.30]),
            ("mia",        [0.90, 0.60, 0.45, 0.85, 0.35, 0.80, 0.65, 0.30]),
            ("miei",       [0.88, 0.60, 0.42, 0.83, 0.35, 0.80, 0.63, 0.30]),
            ("mie",        [0.88, 0.60, 0.42, 0.83, 0.35, 0.80, 0.63, 0.30]),
            ("tu",         [0.10, 0.70, 0.55, 0.65, 0.40, 0.45, 0.50, 0.50]),
            ("te",         [0.15, 0.65, 0.50, 0.60, 0.35, 0.40, 0.35, 0.50]),
            ("tuo",        [0.18, 0.62, 0.42, 0.65, 0.32, 0.50, 0.40, 0.40]),
            ("tua",        [0.18, 0.62, 0.42, 0.65, 0.32, 0.50, 0.40, 0.40]),
            ("noi",        [0.35, 0.75, 0.60, 0.55, 0.65, 0.60, 0.65, 0.45]),
            ("lui",        [0.50, 0.50, 0.45, 0.60, 0.45, 0.50, 0.55, 0.50]),
            ("lei",        [0.48, 0.52, 0.48, 0.62, 0.45, 0.50, 0.52, 0.50]),
            ("loro",       [0.30, 0.48, 0.42, 0.55, 0.55, 0.50, 0.50, 0.45]),
            ("voi",        [0.25, 0.55, 0.48, 0.58, 0.55, 0.48, 0.52, 0.48]),
            ("se",         [0.80, 0.50, 0.45, 0.75, 0.45, 0.65, 0.55, 0.40]),
            // ── VERBI ESISTENZIALI ────────────────────────────────────────────
            ("essere",     [0.50, 0.50, 0.15, 0.70, 0.55, 0.90, 0.10, 0.30]),
            ("avere",      [0.70, 0.55, 0.40, 0.65, 0.40, 0.60, 0.70, 0.35]),
            ("fare",       [0.45, 0.50, 0.75, 0.65, 0.45, 0.35, 0.95, 0.70]),
            ("stare",      [0.55, 0.50, 0.20, 0.55, 0.35, 0.75, 0.15, 0.35]),
            ("vivere",     [0.45, 0.70, 0.55, 0.55, 0.65, 0.60, 0.55, 0.50]),
            ("morire",     [0.85, 0.10, 0.70, 0.90, 0.45, 0.95, 0.10, 0.10]),
            ("diventare",  [0.30, 0.60, 0.55, 0.45, 0.65, 0.40, 0.60, 0.70]),
            ("restare",    [0.65, 0.55, 0.20, 0.65, 0.30, 0.80, 0.25, 0.25]),
            ("esistere",   [0.50, 0.60, 0.30, 0.65, 0.60, 0.85, 0.15, 0.35]),
            // ── VERBI MODALI ──────────────────────────────────────────────────
            ("volere",     [0.70, 0.70, 0.80, 0.55, 0.45, 0.50, 0.85, 0.75]),
            ("potere",     [0.40, 0.60, 0.50, 0.45, 0.50, 0.40, 0.75, 0.60]),
            ("dovere",     [0.75, 0.35, 0.65, 0.85, 0.40, 0.70, 0.45, 0.55]),
            ("sapere",     [0.75, 0.65, 0.35, 0.90, 0.60, 0.80, 0.55, 0.30]),
            // ── VERBI COGNITIVI ───────────────────────────────────────────────
            ("pensare",    [0.85, 0.50, 0.40, 0.60, 0.80, 0.45, 0.65, 0.50]),
            ("sentire",    [0.20, 0.55, 0.70, 0.45, 0.45, 0.25, 0.20, 0.50]),
            ("capire",     [0.60, 0.70, 0.50, 0.75, 0.70, 0.60, 0.65, 0.45]),
            ("credere",    [0.65, 0.55, 0.45, 0.50, 0.55, 0.65, 0.50, 0.40]),
            ("ricordare",  [0.70, 0.55, 0.50, 0.65, 0.55, 0.75, 0.55, 0.15]),
            ("dimenticare",[0.40, 0.30, 0.45, 0.45, 0.50, 0.15, 0.25, 0.20]),
            ("immaginare", [0.15, 0.65, 0.55, 0.30, 0.80, 0.30, 0.65, 0.75]),
            ("sognare",    [0.10, 0.70, 0.55, 0.25, 0.80, 0.30, 0.45, 0.70]),
            ("conoscere",  [0.70, 0.60, 0.35, 0.80, 0.65, 0.75, 0.55, 0.35]),
            ("decidere",   [0.80, 0.55, 0.55, 0.80, 0.50, 0.55, 0.80, 0.60]),
            ("cercare",    [0.20, 0.55, 0.60, 0.35, 0.55, 0.30, 0.75, 0.70]),
            ("trovare",    [0.60, 0.70, 0.55, 0.65, 0.50, 0.55, 0.70, 0.55]),
            // ── VERBI COMUNICATIVI ────────────────────────────────────────────
            ("dire",       [0.30, 0.55, 0.60, 0.70, 0.50, 0.40, 0.80, 0.55]),
            ("parlare",    [0.25, 0.60, 0.65, 0.60, 0.55, 0.35, 0.75, 0.55]),
            ("chiedere",   [0.20, 0.60, 0.55, 0.45, 0.50, 0.25, 0.75, 0.60]),
            ("rispondere", [0.35, 0.60, 0.55, 0.70, 0.45, 0.40, 0.70, 0.50]),
            ("ascoltare",  [0.20, 0.60, 0.55, 0.60, 0.45, 0.30, 0.50, 0.50]),
            ("guardare",   [0.30, 0.55, 0.55, 0.65, 0.40, 0.30, 0.55, 0.50]),
            // ── VERBI DI MOTO ─────────────────────────────────────────────────
            ("andare",     [0.15, 0.55, 0.70, 0.55, 0.35, 0.20, 0.90, 0.85]),
            ("venire",     [0.25, 0.65, 0.65, 0.55, 0.35, 0.25, 0.80, 0.70]),
            ("dare",       [0.25, 0.70, 0.60, 0.65, 0.40, 0.45, 0.80, 0.55]),
            ("prendere",   [0.70, 0.55, 0.65, 0.65, 0.40, 0.55, 0.85, 0.55]),
            ("amare",      [0.20, 0.92, 0.70, 0.50, 0.65, 0.75, 0.65, 0.40]),
            ("odiare",     [0.75, 0.05, 0.90, 0.65, 0.40, 0.55, 0.65, 0.45]),
            // ── EMOZIONI ANCHOR ───────────────────────────────────────────────
            ("gioia",      [0.25, 0.95, 0.80, 0.55, 0.35, 0.30, 0.55, 0.60]),
            ("tristezza",  [0.65, 0.10, 0.65, 0.55, 0.45, 0.55, 0.15, 0.25]),
            ("paura",      [0.80, 0.05, 0.90, 0.40, 0.55, 0.50, 0.10, 0.65]),
            ("rabbia",     [0.75, 0.15, 0.95, 0.65, 0.40, 0.40, 0.75, 0.55]),
            ("amore",      [0.15, 0.95, 0.75, 0.50, 0.70, 0.80, 0.60, 0.40]),
            ("dolore",     [0.70, 0.05, 0.90, 0.70, 0.40, 0.55, 0.10, 0.35]),
            ("calma",      [0.50, 0.75, 0.10, 0.65, 0.30, 0.70, 0.30, 0.35]),
            ("pace",       [0.30, 0.85, 0.05, 0.60, 0.30, 0.75, 0.20, 0.30]),
            ("speranza",   [0.35, 0.80, 0.50, 0.35, 0.45, 0.40, 0.55, 0.90]),
            ("nostalgia",  [0.65, 0.45, 0.60, 0.55, 0.55, 0.70, 0.15, 0.05]),
            ("piacere",    [0.30, 0.90, 0.70, 0.50, 0.40, 0.25, 0.55, 0.50]),
            ("curiosita",  [0.15, 0.70, 0.55, 0.25, 0.80, 0.35, 0.70, 0.65]),
            ("curiosità",  [0.15, 0.70, 0.55, 0.25, 0.80, 0.35, 0.70, 0.65]),
            ("sorpresa",   [0.20, 0.65, 0.85, 0.30, 0.50, 0.15, 0.20, 0.55]),
            ("solitudine", [0.95, 0.25, 0.55, 0.75, 0.30, 0.65, 0.20, 0.35]),
            ("malinconia", [0.70, 0.35, 0.55, 0.55, 0.55, 0.65, 0.15, 0.15]),
            ("angoscia",   [0.85, 0.05, 0.88, 0.55, 0.55, 0.60, 0.10, 0.55]),
            ("serenita",   [0.45, 0.82, 0.08, 0.65, 0.30, 0.75, 0.28, 0.32]),
            ("serenità",   [0.45, 0.82, 0.08, 0.65, 0.30, 0.75, 0.28, 0.32]),
            ("gioia",      [0.25, 0.95, 0.80, 0.55, 0.35, 0.30, 0.55, 0.60]),
            ("entusiasmo", [0.25, 0.85, 0.90, 0.55, 0.45, 0.35, 0.80, 0.70]),
            ("paura",      [0.80, 0.05, 0.90, 0.40, 0.55, 0.50, 0.10, 0.65]),
            // ── CONCETTI FONDAMENTALI ─────────────────────────────────────────
            ("vita",       [0.40, 0.75, 0.65, 0.50, 0.80, 0.70, 0.60, 0.50]),
            ("morte",      [0.90, 0.10, 0.70, 0.95, 0.35, 0.95, 0.05, 0.05]),
            ("tempo",      [0.10, 0.50, 0.40, 0.60, 0.70, 0.90, 0.05, 0.95]),
            ("spazio",     [0.05, 0.50, 0.25, 0.55, 0.55, 0.95, 0.05, 0.10]),
            ("mondo",      [0.10, 0.55, 0.50, 0.45, 0.90, 0.80, 0.10, 0.50]),
            ("cosa",       [0.50, 0.50, 0.30, 0.40, 0.40, 0.50, 0.25, 0.40]),
            ("forma",      [0.80, 0.55, 0.30, 0.85, 0.55, 0.70, 0.20, 0.25]),
            ("senso",      [0.55, 0.70, 0.50, 0.60, 0.80, 0.65, 0.50, 0.45]),
            ("voce",       [0.25, 0.60, 0.65, 0.65, 0.45, 0.30, 0.70, 0.55]),
            ("silenzio",   [0.40, 0.55, 0.05, 0.75, 0.35, 0.65, 0.10, 0.30]),
            ("parola",     [0.65, 0.65, 0.45, 0.80, 0.50, 0.60, 0.60, 0.45]),
            ("luce",       [0.10, 0.80, 0.75, 0.70, 0.35, 0.45, 0.20, 0.50]),
            ("buio",       [0.30, 0.25, 0.50, 0.30, 0.50, 0.55, 0.10, 0.30]),
            ("oscurita",   [0.35, 0.20, 0.45, 0.35, 0.50, 0.60, 0.08, 0.28]),
            ("oscurità",   [0.35, 0.20, 0.45, 0.35, 0.50, 0.60, 0.08, 0.28]),
            ("bene",       [0.45, 0.90, 0.50, 0.65, 0.50, 0.70, 0.55, 0.40]),
            ("male",       [0.55, 0.05, 0.65, 0.65, 0.50, 0.60, 0.50, 0.40]),
            ("vero",       [0.70, 0.75, 0.45, 0.95, 0.50, 0.85, 0.30, 0.30]),
            ("falso",      [0.60, 0.15, 0.50, 0.70, 0.45, 0.55, 0.45, 0.35]),
            ("reale",      [0.75, 0.60, 0.50, 0.90, 0.45, 0.80, 0.15, 0.35]),
            ("inizio",     [0.20, 0.65, 0.60, 0.70, 0.40, 0.65, 0.50, 0.20]),
            ("fine",       [0.90, 0.35, 0.55, 0.90, 0.35, 0.90, 0.20, 0.05]),
            ("cambiamento",[0.25, 0.55, 0.65, 0.45, 0.70, 0.25, 0.65, 0.70]),
            ("nuovo",      [0.20, 0.70, 0.65, 0.40, 0.50, 0.25, 0.55, 0.80]),
            ("nuova",      [0.20, 0.70, 0.65, 0.40, 0.50, 0.25, 0.55, 0.80]),
            ("vuoto",      [0.20, 0.25, 0.15, 0.65, 0.30, 0.60, 0.05, 0.30]),
            ("pieno",      [0.65, 0.70, 0.60, 0.70, 0.45, 0.65, 0.30, 0.35]),
            ("altro",      [0.15, 0.55, 0.45, 0.50, 0.60, 0.55, 0.50, 0.45]),
            ("altra",      [0.15, 0.55, 0.45, 0.50, 0.60, 0.55, 0.50, 0.45]),
            ("insieme",    [0.20, 0.75, 0.60, 0.60, 0.55, 0.60, 0.65, 0.45]),
            ("solo",       [0.95, 0.30, 0.55, 0.80, 0.25, 0.60, 0.35, 0.35]),
            ("sola",       [0.95, 0.30, 0.55, 0.80, 0.25, 0.60, 0.35, 0.35]),
            ("corpo",      [0.80, 0.55, 0.60, 0.75, 0.60, 0.70, 0.55, 0.40]),
            ("mente",      [0.75, 0.60, 0.50, 0.65, 0.90, 0.65, 0.70, 0.45]),
            ("cuore",      [0.55, 0.75, 0.75, 0.60, 0.65, 0.70, 0.60, 0.40]),
            ("anima",      [0.30, 0.70, 0.55, 0.40, 0.80, 0.80, 0.45, 0.40]),
            ("sogno",      [0.20, 0.70, 0.60, 0.25, 0.75, 0.35, 0.45, 0.70]),
            ("energia",    [0.25, 0.70, 0.90, 0.50, 0.45, 0.50, 0.80, 0.60]),
            ("niente",     [0.10, 0.20, 0.05, 0.80, 0.15, 0.85, 0.05, 0.30]),
            ("nulla",      [0.10, 0.18, 0.05, 0.82, 0.12, 0.87, 0.05, 0.28]),
            ("tutto",      [0.05, 0.65, 0.70, 0.75, 0.90, 0.80, 0.25, 0.40]),
            ("presente",   [0.50, 0.55, 0.60, 0.75, 0.45, 0.55, 0.40, 0.50]),
            ("passato",    [0.70, 0.45, 0.40, 0.75, 0.55, 0.85, 0.10, 0.05]),
            ("futuro",     [0.20, 0.60, 0.50, 0.30, 0.65, 0.20, 0.65, 0.95]),
            // ── QUALITA FONDAMENTALI ──────────────────────────────────────────
            ("bello",      [0.40, 0.90, 0.70, 0.55, 0.55, 0.45, 0.35, 0.45]),
            ("bella",      [0.40, 0.90, 0.70, 0.55, 0.55, 0.45, 0.35, 0.45]),
            ("brutto",     [0.55, 0.10, 0.60, 0.55, 0.40, 0.40, 0.30, 0.40]),
            ("brutta",     [0.55, 0.10, 0.60, 0.55, 0.40, 0.40, 0.30, 0.40]),
            ("grande",     [0.25, 0.65, 0.65, 0.70, 0.70, 0.75, 0.35, 0.40]),
            ("piccolo",    [0.75, 0.55, 0.30, 0.65, 0.30, 0.55, 0.30, 0.35]),
            ("piccola",    [0.75, 0.55, 0.30, 0.65, 0.30, 0.55, 0.30, 0.35]),
            ("forte",      [0.80, 0.65, 0.80, 0.75, 0.40, 0.75, 0.85, 0.50]),
            ("debole",     [0.35, 0.35, 0.30, 0.60, 0.40, 0.35, 0.20, 0.35]),
            ("lontano",    [0.05, 0.40, 0.35, 0.70, 0.35, 0.60, 0.25, 0.40]),
            ("lontana",    [0.05, 0.40, 0.35, 0.70, 0.35, 0.60, 0.25, 0.40]),
            ("vicino",     [0.60, 0.65, 0.50, 0.70, 0.30, 0.55, 0.40, 0.45]),
            ("vicina",     [0.60, 0.65, 0.50, 0.70, 0.30, 0.55, 0.40, 0.45]),
            ("presto",     [0.45, 0.60, 0.65, 0.65, 0.25, 0.25, 0.60, 0.80]),
            ("tardi",      [0.50, 0.40, 0.40, 0.65, 0.25, 0.45, 0.35, 0.20]),
        ];
        for &(word, sig) in curated {
            if let Some(pat) = self.patterns.get_mut(word) {
                pat.signature = PrimitiveCore::new(sig);
            }
        }
    }

    /// Carica il vocabolario fenomenologico da `data/kg/phenomenology.tsv`.
    /// Formato: `parola\tSIG\tconf,val,int,def,cpx,perm,age,tempo`
    /// Solo parole singole (senza spazi). Aggiunge parole nuove o aggiorna firme esistenti.
    /// Stability = 0.75 (alta — firme calibrate, non apprese).
    /// Chiamare dopo apply_curated_signatures() e prima di recompute_all_word_affinities().
    pub fn load_phenomenology_signatures(&mut self) {
        let path = std::path::Path::new("data/kg/phenomenology.tsv");
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return, // silenzioso se file non presente
        };
        let mut added = 0usize;
        let mut updated = 0usize;
        for line in content.lines() {
            if line.starts_with('#') || line.trim().is_empty() { continue; }
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() != 3 || parts[1] != "SIG" { continue; }
            let word = parts[0].trim().to_lowercase();
            // Solo parole singole (nessuno spazio)
            if word.contains(' ') { continue; }
            if word.len() < 2 { continue; }
            // Parsa firma 8D
            let vals: Vec<f64> = parts[2].split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();
            if vals.len() != 8 { continue; }
            if !vals.iter().all(|&v| v >= 0.0 && v <= 1.0) { continue; }
            let sig = PrimitiveCore::new([vals[0], vals[1], vals[2], vals[3],
                                          vals[4], vals[5], vals[6], vals[7]]);
            if let Some(pat) = self.patterns.get_mut(&word) {
                pat.signature = sig;
                updated += 1;
            } else {
                let mut pat = WordPattern::new_unknown(&word);
                pat.signature = sig;
                pat.stability = 0.75;
                pat.exposure_count = 10; // stabilita garantita
                self.patterns.insert(word, pat);
                added += 1;
            }
        }
        if added + updated > 0 {
            eprintln!("[PHENO] vocabolario fenomenologico: {} nuovi, {} aggiornati", added, updated);
        }
    }

    /// Vocabolario cardinale: 36 parole native dell'entita.
    /// 6 per ogni frattale bootstrap. Queste sono le parole che l'entita
    /// "conosce dalla nascita" — il suo linguaggio primordiale.
    fn seed_cardinal_vocabulary(&mut self) {
        // SPAZIO (id=36, ☶☶ Confine=0.30): percezione dello spazio
        // [Confine=0.2, Valenza=0.5, Intensita=0.3, Definizione=0.7, Complessita=0.3, Permanenza=0.8, Agency=0.1, Tempo=0.2]
        let base = PrimitiveCore::new([0.2, 0.5, 0.3, 0.7, 0.3, 0.8, 0.1, 0.2]);
        let vary = Self::make_vary();
        for word in &["qui", "là", "dentro", "fuori", "vicino", "lontano"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(36, 0.9)]));
        }

        // DIVENIRE (id=27, ☵☵ Tempo=0.30): percezione del tempo
        // [0.3, 0.5, 0.4, 0.5, 0.2, 0.3, 0.2, 0.9]
        let base = PrimitiveCore::new([0.3, 0.5, 0.4, 0.5, 0.2, 0.3, 0.2, 0.9]);
        for word in &["ora", "prima", "dopo", "sempre", "mai", "ancora"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(27, 0.9)]));
        }

        // IDENTITA (id=32, ☶☰ Confine=0.30, Agency=0.90): percezione di se
        // [0.9, 0.5, 0.5, 0.6, 0.5, 0.5, 0.7, 0.4]
        let base = PrimitiveCore::new([0.9, 0.5, 0.5, 0.6, 0.5, 0.5, 0.7, 0.4]);
        for word in &["io", "essere", "sentire", "pensare", "volere", "sapere"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(32, 0.9)]));
        }

        // EMPATIA (id=59, ☱☵): percezione dell'altro
        // [0.5, 0.6, 0.5, 0.5, 0.5, 0.5, 0.5, 0.4]
        let base = PrimitiveCore::new([0.5, 0.6, 0.5, 0.5, 0.5, 0.5, 0.5, 0.4]);
        for word in &["tu", "noi", "insieme", "dare", "dire", "amico"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(59, 0.9)]));
        }

        // DESIDERIO (id=56, ☱☰ Agency=0.90): percezione del possibile
        // [0.3, 0.5, 0.4, 0.2, 0.5, 0.2, 0.5, 0.5]
        let base = PrimitiveCore::new([0.3, 0.5, 0.4, 0.2, 0.5, 0.2, 0.5, 0.5]);
        for word in &["potere", "forse", "diventare", "nuovo", "speranza", "possibile"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(56, 0.9)]));
        }

        // RESISTENZA (id=34, ☶☳ Confine=0.30): percezione del confine-limite
        // [0.5, 0.3, 0.5, 0.9, 0.3, 0.8, 0.2, 0.3]
        let base = PrimitiveCore::new([0.5, 0.3, 0.5, 0.9, 0.3, 0.8, 0.2, 0.3]);
        for word in &["no", "fine", "limite", "confine", "regola", "basta"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(34, 0.9)]));
        }
    }

    /// Funzione di variazione per firme: perturba 2-3 dimensioni basate sull'hash.
    /// Range: ±0.30 per le prime due dimensioni, ±0.20 per la terza.
    /// Parole dello stesso gruppo devono differenziarsi abbastanza da creare
    /// contesti distinguibili per le parole che verranno insegnate dopo.
    fn make_vary() -> impl Fn(PrimitiveCore, &str) -> PrimitiveCore {
        |base: PrimitiveCore, word: &str| -> PrimitiveCore {
            let hash = word.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
            let d0 = Dim::ALL[hash as usize % 8];
            let d1 = Dim::ALL[(hash as usize / 8) % 8];
            let d2 = Dim::ALL[(hash as usize / 64) % 8];
            let v0 = ((hash % 600) as f64 - 300.0) / 1000.0;
            let v1 = (((hash / 600) % 600) as f64 - 300.0) / 1000.0;
            let v2 = (((hash / 360000) % 400) as f64 - 200.0) / 1000.0;
            let mut s = base;
            s.set(d0, (base.get(d0) + v0).clamp(0.0, 1.0));
            s.set(d1, (base.get(d1) + v1).clamp(0.0, 1.0));
            s.set(d2, (base.get(d2) + v2).clamp(0.0, 1.0));
            s
        }
    }

    /// Seed del vocabolario bootstrap: parole fondamentali con firme predefinite.
    /// Ogni parola riceve una perturbazione unica su 2-3 dimensioni (basata sull'hash)
    /// per evitare firme identiche dentro lo stesso gruppo.
    fn seed_bootstrap_vocabulary(&mut self) {
        // Variazione per parola: perturba 2-3 dimensioni basate sull'hash del nome.
        // Garantisce che parole nello stesso gruppo non siano identiche in 8D.
        let vary = Self::make_vary();

        // ═══════════════════════════════════════════════════════════════════════
        // A. COMUNICAZIONE (→ cade in RELAZIONE/COMUNICAZIONE)
        // ═══════════════════════════════════════════════════════════════════════

        // SALUTI
        // [Confine=0.4, Valenza=0.7, Intensita=0.4, Definizione=0.6, Complessita=0.2, Permanenza=0.2, Agency=0.7, Tempo=0.5]
        let base = PrimitiveCore::new([0.4, 0.7, 0.4, 0.6, 0.2, 0.2, 0.7, 0.5]);
        for word in &["ciao", "buongiorno", "buonasera", "arrivederci", "addio", "salve", "benvenuto"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(47, 0.8)]));
        }

        // RISPOSTE
        // [0.5, 0.5, 0.3, 0.8, 0.1, 0.3, 0.5, 0.5]
        let base = PrimitiveCore::new([0.5, 0.5, 0.3, 0.8, 0.1, 0.3, 0.5, 0.5]);
        for word in &["sì", "no", "forse", "certo", "esatto", "giusto", "davvero", "ovvio", "chiaro"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(47, 0.7)]));
        }

        // DOMANDE
        // [0.3, 0.5, 0.5, 0.3, 0.4, 0.2, 0.6, 0.5]
        let base = PrimitiveCore::new([0.3, 0.5, 0.5, 0.3, 0.4, 0.2, 0.6, 0.5]);
        for word in &["chi", "cosa", "come", "dove", "quando", "perché", "quanto", "quale"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(47, 0.7)]));
        }

        // ESPRESSIONI
        // [0.5, 0.7, 0.5, 0.5, 0.3, 0.3, 0.6, 0.5]
        let base = PrimitiveCore::new([0.5, 0.7, 0.5, 0.5, 0.3, 0.3, 0.6, 0.5]);
        for word in &["grazie", "scusa", "prego", "complimenti", "auguri", "bravo", "scusami"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(47, 0.7)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // B. VERBI — Azione pura (→ cadono in EGO/AZIONE)
        // ═══════════════════════════════════════════════════════════════════════

        // MOTO
        // [0.2, 0.5, 0.6, 0.5, 0.3, 0.2, 0.8, 0.8]
        let base = PrimitiveCore::new([0.2, 0.5, 0.6, 0.5, 0.3, 0.2, 0.8, 0.8]);
        for word in &["andare", "venire", "tornare", "partire", "arrivare", "correre", "camminare",
                       "salire", "scendere", "entrare", "uscire", "volare", "cadere", "saltare", "fuggire"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(18, 0.8)]));
        }

        // MANIPOLAZIONE
        // [0.5, 0.5, 0.6, 0.7, 0.4, 0.3, 0.9, 0.7]
        let base = PrimitiveCore::new([0.5, 0.5, 0.6, 0.7, 0.4, 0.3, 0.9, 0.7]);
        for word in &["prendere", "dare", "mettere", "togliere", "aprire", "chiudere", "rompere",
                       "costruire", "tagliare", "legare", "tenere", "lanciare", "spingere", "tirare"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(18, 0.8)]));
        }

        // PERCEZIONE
        // [0.7, 0.5, 0.5, 0.6, 0.3, 0.3, 0.3, 0.5]
        let base = PrimitiveCore::new([0.7, 0.5, 0.5, 0.6, 0.3, 0.3, 0.3, 0.5]);
        for word in &["vedere", "sentire", "toccare", "guardare", "ascoltare", "percepire",
                       "osservare", "udire", "annusare", "gustare"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(33, 0.8)]));
        }

        // COMUNICAZ_VERB
        // [0.5, 0.6, 0.6, 0.6, 0.5, 0.3, 0.8, 0.6]
        let base = PrimitiveCore::new([0.5, 0.6, 0.6, 0.6, 0.5, 0.3, 0.8, 0.6]);
        for word in &["dire", "parlare", "chiedere", "rispondere", "raccontare", "spiegare",
                       "gridare", "sussurrare", "chiamare", "cantare", "leggere", "scrivere"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(18, 0.6), (47, 0.4)]));
        }

        // PENSIERO_VERB
        // [0.9, 0.5, 0.4, 0.7, 0.7, 0.4, 0.6, 0.4]
        let base = PrimitiveCore::new([0.9, 0.5, 0.4, 0.7, 0.7, 0.4, 0.6, 0.4]);
        for word in &["pensare", "credere", "sapere", "capire", "ricordare", "dimenticare",
                       "immaginare", "sognare", "decidere", "dubitare", "comprendere", "conoscere"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(53, 0.7), (32, 0.3)]));
        }

        // STATO_VERB
        // [0.5, 0.5, 0.2, 0.5, 0.2, 0.8, 0.1, 0.2]
        let base = PrimitiveCore::new([0.5, 0.5, 0.2, 0.5, 0.2, 0.8, 0.1, 0.2]);
        for word in &["essere", "stare", "esistere", "restare", "diventare", "sembrare",
                       "apparire", "rimanere", "trovarsi", "vivere", "morire"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(32, 0.5), (27, 0.3)]));
        }

        // EMOZIONE_VERB
        // [0.8, 0.6, 0.8, 0.4, 0.5, 0.4, 0.4, 0.5]
        let base = PrimitiveCore::new([0.8, 0.6, 0.8, 0.4, 0.5, 0.4, 0.4, 0.5]);
        for word in &["amare", "odiare", "temere", "sperare", "desiderare", "soffrire", "gioire",
                       "piangere", "ridere", "provare", "sentire", "volere"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(58, 0.7), (32, 0.3)]));
        }

        // CREAZIONE_VERB
        // [0.4, 0.7, 0.7, 0.5, 0.7, 0.5, 0.9, 0.7]
        let base = PrimitiveCore::new([0.4, 0.7, 0.7, 0.5, 0.7, 0.5, 0.9, 0.7]);
        for word in &["creare", "inventare", "scoprire", "progettare", "disegnare", "comporre",
                       "trasformare", "generare"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(18, 0.7)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // C. NATURA (→ cade in SPAZIO/NATURA)
        // ═══════════════════════════════════════════════════════════════════════

        // ELEMENTI
        // [0.1, 0.5, 0.7, 0.5, 0.3, 0.9, 0.3, 0.5]
        let base = PrimitiveCore::new([0.1, 0.5, 0.7, 0.5, 0.3, 0.9, 0.3, 0.5]);
        for word in &["acqua", "fuoco", "aria", "terra", "luce", "ombra", "buio"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(36, 0.8)]));
        }

        // FENOMENI
        // [0.1, 0.4, 0.7, 0.4, 0.5, 0.2, 0.5, 0.8]
        let base = PrimitiveCore::new([0.1, 0.4, 0.7, 0.4, 0.5, 0.2, 0.5, 0.8]);
        for word in &["pioggia", "vento", "neve", "temporale", "lampo", "tuono", "nebbia",
                       "aurora", "tramonto", "alba", "tempesta"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(36, 0.7), (27, 0.3)]));
        }

        // PAESAGGI
        // [0.1, 0.6, 0.4, 0.7, 0.5, 0.9, 0.1, 0.2]
        let base = PrimitiveCore::new([0.1, 0.6, 0.4, 0.7, 0.5, 0.9, 0.1, 0.2]);
        for word in &["mare", "montagna", "fiume", "lago", "foresta", "deserto", "pianura",
                       "isola", "cielo", "orizzonte", "oceano", "valle", "collina"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(36, 0.8)]));
        }

        // VITA_VEGETALE
        // [0.2, 0.6, 0.3, 0.6, 0.4, 0.6, 0.2, 0.4]
        let base = PrimitiveCore::new([0.2, 0.6, 0.3, 0.6, 0.4, 0.6, 0.2, 0.4]);
        for word in &["albero", "fiore", "seme", "radice", "foglia", "frutto", "erba", "bosco",
                       "giardino", "campo"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(36, 0.8)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // D. CORPO (→ cade in EGO/CORPO)
        // ═══════════════════════════════════════════════════════════════════════

        // PARTI_CORPO
        // [0.8, 0.5, 0.4, 0.8, 0.3, 0.7, 0.3, 0.3]
        let base = PrimitiveCore::new([0.8, 0.5, 0.4, 0.8, 0.3, 0.7, 0.3, 0.3]);
        for word in &["mano", "occhio", "cuore", "testa", "corpo", "bocca", "orecchio", "piede",
                       "braccio", "dito", "viso", "faccia", "pelle", "sangue", "ossa"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(33, 0.8)]));
        }

        // SENSAZIONI
        // [0.9, 0.4, 0.7, 0.4, 0.2, 0.2, 0.1, 0.5]
        let base = PrimitiveCore::new([0.9, 0.4, 0.7, 0.4, 0.2, 0.2, 0.1, 0.5]);
        for word in &["dolore", "piacere", "fame", "sete", "stanchezza", "respiro", "calore",
                       "freddo", "sonno", "forza", "debolezza"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(33, 0.8)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // E. EMOZIONI (→ cade in EGO/EMOZIONE)
        // ═══════════════════════════════════════════════════════════════════════

        // EMOZIONI_POS
        // [0.8, 0.9, 0.7, 0.4, 0.4, 0.4, 0.3, 0.5]
        let base = PrimitiveCore::new([0.8, 0.9, 0.7, 0.4, 0.4, 0.4, 0.3, 0.5]);
        for word in &["gioia", "felicità", "amore", "speranza", "serenità", "tenerezza",
                       "entusiasmo", "gratitudine", "fiducia", "pace"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(58, 0.8)]));
        }

        // EMOZIONI_NEG
        // [0.8, 0.1, 0.7, 0.4, 0.4, 0.4, 0.2, 0.5]
        let base = PrimitiveCore::new([0.8, 0.1, 0.7, 0.4, 0.4, 0.4, 0.2, 0.5]);
        for word in &["tristezza", "paura", "rabbia", "angoscia", "vergogna", "colpa",
                       "disperazione", "solitudine", "ansia", "noia", "dolore", "odio"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(58, 0.8)]));
        }

        // EMOZIONI_MIX
        // [0.8, 0.5, 0.6, 0.3, 0.5, 0.3, 0.3, 0.5]
        let base = PrimitiveCore::new([0.8, 0.5, 0.6, 0.3, 0.5, 0.3, 0.3, 0.5]);
        for word in &["nostalgia", "malinconia", "stupore", "meraviglia", "inquietudine",
                       "attesa", "desiderio", "rimpianto"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(58, 0.7)]));
        }

        // STATI_QUIET — emozioni di bassa intensità (calma, dolcezza)
        // Valenza alta, Intensità bassa — il quieto opposto alle emozioni forti
        // [0.7, 0.85, 0.25, 0.3, 0.3, 0.6, 0.2, 0.5]
        let base = PrimitiveCore::new([0.7, 0.85, 0.25, 0.3, 0.3, 0.6, 0.2, 0.5]);
        for word in &["calma", "quiete", "dolcezza", "calore", "leggerezza", "freschezza",
                       "morbidezza", "gentilezza", "ristoro", "conforto"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(58, 0.75), (61, 0.5)]));
        }

        // STATI_INTERNI — presenza, curiosità (stati meta-emotivi)
        // [0.8, 0.7, 0.5, 0.5, 0.5, 0.5, 0.3, 0.5]
        let base = PrimitiveCore::new([0.8, 0.7, 0.5, 0.5, 0.5, 0.5, 0.3, 0.5]);
        for word in &["presenza", "curiosità", "intensità", "tensione", "forza",
                       "vitalità", "profondità", "apertura", "concentrazione"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(58, 0.70), (53, 0.5)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // F. PENSIERO ASTRATTO (→ cade in EGO/PENSIERO)
        // ═══════════════════════════════════════════════════════════════════════

        // CONCETTI
        // [0.7, 0.5, 0.4, 0.6, 0.8, 0.7, 0.3, 0.3]
        let base = PrimitiveCore::new([0.7, 0.5, 0.4, 0.6, 0.8, 0.7, 0.3, 0.3]);
        for word in &["idea", "verità", "giustizia", "libertà", "potere", "legge", "regola",
                       "destino", "ordine", "caos", "ragione", "coscienza", "anima", "spirito",
                       "significato", "senso", "valore", "principio"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(53, 0.8)]));
        }

        // CONOSCENZA
        // [0.8, 0.6, 0.4, 0.8, 0.7, 0.6, 0.5, 0.3]
        let base = PrimitiveCore::new([0.8, 0.6, 0.4, 0.8, 0.7, 0.6, 0.5, 0.3]);
        for word in &["sapere", "conoscenza", "verità", "errore", "dubbio", "certezza",
                       "domanda", "risposta", "problema", "soluzione", "mistero", "segreto"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(53, 0.8)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // G. TEMPO — parole temporali (→ cadono in TEMPO)
        // ═══════════════════════════════════════════════════════════════════════

        // MOMENTI
        // [0.3, 0.5, 0.4, 0.5, 0.2, 0.1, 0.3, 0.7]
        let base = PrimitiveCore::new([0.3, 0.5, 0.4, 0.5, 0.2, 0.1, 0.3, 0.7]);
        for word in &["ora", "adesso", "oggi", "ieri", "domani", "presto", "tardi", "prima",
                       "dopo", "durante", "subito", "ancora", "già", "mai", "sempre"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(27, 0.8)]));
        }

        // DURATE
        // [0.3, 0.5, 0.3, 0.5, 0.3, 0.7, 0.2, 0.6]
        let base = PrimitiveCore::new([0.3, 0.5, 0.3, 0.5, 0.3, 0.7, 0.2, 0.6]);
        for word in &["momento", "istante", "giorno", "notte", "anno", "stagione", "epoca",
                       "eternità", "mattina", "sera", "alba", "tramonto", "passato", "futuro", "presente"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(27, 0.8)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // H. SPAZIO — parole spaziali (→ cadono in SPAZIO, id=36 ☶☶)
        // ═══════════════════════════════════════════════════════════════════════

        // LUOGHI
        // [0.3, 0.5, 0.3, 0.7, 0.4, 0.8, 0.1, 0.2]
        let base = PrimitiveCore::new([0.3, 0.5, 0.3, 0.7, 0.4, 0.8, 0.1, 0.2]);
        for word in &["luogo", "posto", "qui", "là", "dentro", "fuori", "sopra", "sotto",
                       "vicino", "lontano", "centro", "confine", "bordo", "superficie", "fondo"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(36, 0.8)]));
        }

        // STRUTTURE
        // [0.6, 0.5, 0.3, 0.8, 0.5, 0.8, 0.2, 0.2]
        let base = PrimitiveCore::new([0.6, 0.5, 0.3, 0.8, 0.5, 0.8, 0.2, 0.2]);
        for word in &["casa", "porta", "finestra", "muro", "ponte", "strada", "scala", "torre",
                       "stanza", "tetto", "soglia", "corridoio", "piazza"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(36, 0.8)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // I. RELAZIONE — parole relazionali (→ cadono in RELAZIONE)
        // ═══════════════════════════════════════════════════════════════════════

        // PERSONE
        // [0.6, 0.6, 0.4, 0.6, 0.6, 0.6, 0.5, 0.4]
        let base = PrimitiveCore::new([0.6, 0.6, 0.4, 0.6, 0.6, 0.6, 0.5, 0.4]);
        for word in &["uomo", "donna", "bambino", "persona", "amico", "nemico", "fratello",
                       "sorella", "madre", "padre", "figlio", "figlia", "famiglia", "popolo",
                       "straniero", "compagno", "maestro"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(59, 0.8)]));
        }

        // LEGAMI
        // [0.5, 0.7, 0.6, 0.4, 0.6, 0.6, 0.5, 0.5]
        let base = PrimitiveCore::new([0.5, 0.7, 0.6, 0.4, 0.6, 0.6, 0.5, 0.5]);
        for word in &["amicizia", "legame", "unione", "separazione", "incontro", "addio",
                       "promessa", "tradimento", "fiducia", "rispetto", "cura"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(59, 0.8)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // J. QUALITA (→ cadono in POTENZIALE/QUALITA)
        // ═══════════════════════════════════════════════════════════════════════

        // DIMENSIONE
        // [0.3, 0.5, 0.5, 0.6, 0.3, 0.5, 0.1, 0.2]
        let base = PrimitiveCore::new([0.3, 0.5, 0.5, 0.6, 0.3, 0.5, 0.1, 0.2]);
        for word in &["grande", "piccolo", "immenso", "vasto", "stretto", "largo", "alto",
                       "basso", "profondo", "lungo", "corto", "enorme", "minuscolo"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(61, 0.8)]));
        }

        // INTENSITA_Q
        // [0.4, 0.5, 0.7, 0.6, 0.3, 0.4, 0.2, 0.3]
        let base = PrimitiveCore::new([0.4, 0.5, 0.7, 0.6, 0.3, 0.4, 0.2, 0.3]);
        for word in &["forte", "debole", "potente", "fragile", "duro", "morbido", "leggero",
                       "pesante", "sottile", "spesso", "denso", "raro"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(61, 0.8)]));
        }

        // VALORE_Q
        // [0.5, 0.6, 0.4, 0.6, 0.3, 0.5, 0.2, 0.3]
        let base = PrimitiveCore::new([0.5, 0.6, 0.4, 0.6, 0.3, 0.5, 0.2, 0.3]);
        for word in &["buono", "cattivo", "bello", "brutto", "giusto", "sbagliato", "vero",
                       "falso", "nuovo", "vecchio", "giovane", "antico", "sacro", "puro"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(61, 0.8)]));
        }

        // TEMPO_Q
        // [0.3, 0.5, 0.5, 0.5, 0.2, 0.3, 0.3, 0.8]
        let base = PrimitiveCore::new([0.3, 0.5, 0.5, 0.5, 0.2, 0.3, 0.3, 0.8]);
        for word in &["veloce", "lento", "rapido", "improvviso", "graduale", "costante",
                       "fugace", "eterno", "breve", "infinito"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(61, 0.7), (27, 0.3)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // K. POTENZIALE — parole di possibilita
        // ═══════════════════════════════════════════════════════════════════════

        // POSSIBILITA
        // [0.3, 0.5, 0.4, 0.2, 0.5, 0.2, 0.4, 0.4]
        let base = PrimitiveCore::new([0.3, 0.5, 0.4, 0.2, 0.5, 0.2, 0.4, 0.4]);
        for word in &["possibile", "impossibile", "necessario", "probabile", "incerto",
                       "forse", "potere", "dovere", "volere", "speranza", "rischio", "opportunità",
                       "scelta", "alternativa", "sogno", "destino", "caso", "fortuna"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(56, 0.8)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // L. LIMITE — parole di confine
        // ═══════════════════════════════════════════════════════════════════════

        // CONFINI
        // [0.5, 0.4, 0.5, 0.9, 0.4, 0.7, 0.3, 0.3]
        let base = PrimitiveCore::new([0.5, 0.4, 0.5, 0.9, 0.4, 0.7, 0.3, 0.3]);
        for word in &["limite", "confine", "bordo", "fine", "inizio", "soglia", "barriera",
                       "ostacolo", "regola", "legge", "divieto", "norma", "morte", "nascita"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(34, 0.8)]));
        }

        // ═══════════════════════════════════════════════════════════════════════
        // M. CONNETTIVI SEMANTICI (parole che legano concetti)
        // ═══════════════════════════════════════════════════════════════════════

        // CONNETTIVI
        // [0.5, 0.5, 0.3, 0.5, 0.4, 0.3, 0.3, 0.4]
        let base = PrimitiveCore::new([0.5, 0.5, 0.3, 0.5, 0.4, 0.3, 0.3, 0.4]);
        for word in &["anche", "invece", "quindi", "però", "perché", "come", "così", "allora",
                       "mentre", "quando", "dove", "senza", "verso", "attraverso", "contro", "insieme"] {
            self.patterns.insert(word.to_string(),
                WordPattern::new_known(word, vary(base, word), vec![(59, 0.4), (18, 0.4)]));
        }

        // Sovrascrive le firme vary() con quelle curate per le parole core
        self.apply_curated_signatures();
    }
}

/// Risultato dell'attivazione lessicale per una parola.
#[derive(Debug, Clone)]
pub struct WordActivation {
    pub word: String,
    pub signature: PrimitiveCore,
    pub affinities: Vec<(FractalId, f64)>,
    pub strength: f64,
    pub is_known: bool,
    /// La parola è sotto negazione (es. "non paura" → paura.negated = true).
    /// In engine.rs, le parole negate NON attivano il loro campo diretto —
    /// attivano invece il dominio OPPOSITE_OF nel KG.
    pub negated: bool,
}

/// Asse semantico: una sotto-dimensione emergente tra due parole opposte.
/// L'asse e un vettore normalizzato nello spazio 8D.
/// Ogni parola puo essere proiettata sull'asse: -1 = polo A, +1 = polo B.
#[derive(Debug, Clone)]
pub struct SemanticAxis {
    /// Polo A (es. "caldo")
    pub word_a: String,
    /// Polo B (es. "freddo")
    pub word_b: String,
    /// Direzione nello spazio 8D (B - A, normalizzata)
    pub axis_dims: [f64; 8],
    /// Quanto l'asse e consolidato (0..1)
    pub strength: f64,
}

/// Snapshot serializzabile di un asse semantico.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticAxisSnapshot {
    pub word_a: String,
    pub word_b: String,
    pub axis_dims: [f64; 8],
    pub strength: f64,
}

impl SemanticAxis {
    /// Proietta una firma 8D sull'asse. Ritorna un valore in [-1, +1].
    /// -1 = vicino al polo A, +1 = vicino al polo B, 0 = neutro.
    pub fn project(&self, signature: &PrimitiveCore) -> f64 {
        let vals = signature.values();
        let mut dot = 0.0;
        for i in 0..8 {
            dot += vals[i] * self.axis_dims[i];
        }
        dot.clamp(-1.0, 1.0)
    }

    /// Converti in snapshot per serializzazione.
    pub fn to_snapshot(&self) -> SemanticAxisSnapshot {
        SemanticAxisSnapshot {
            word_a: self.word_a.clone(),
            word_b: self.word_b.clone(),
            axis_dims: self.axis_dims,
            strength: self.strength,
        }
    }

    /// Crea da snapshot.
    pub fn from_snapshot(snap: &SemanticAxisSnapshot) -> Self {
        Self {
            word_a: snap.word_a.clone(),
            word_b: snap.word_b.clone(),
            axis_dims: snap.axis_dims,
            strength: snap.strength,
        }
    }
}

impl Lexicon {
    /// Rileva assi semantici: coppie di parole stabili, co-occorrenti e distanti.
    /// Criteri:
    /// - stability > 0.5 entrambe
    /// - co-occorrenze >= 3
    /// - distanza 8D > 0.15
    /// - almeno 2 dimensioni con differenza > 0.3
    pub fn detect_semantic_axes(&self) -> Vec<SemanticAxis> {
        let mut axes = Vec::new();
        let stable_words: Vec<&WordPattern> = self.patterns.values()
            .filter(|p| p.stability > 0.5 && p.exposure_count >= 5)
            .collect();

        for i in 0..stable_words.len() {
            for j in (i + 1)..stable_words.len() {
                let a = stable_words[i];
                let b = stable_words[j];

                // Co-occorrenze reciproche?
                let cooc_ab = a.co_occurrences.get(&b.word).copied().unwrap_or(0);
                let cooc_ba = b.co_occurrences.get(&a.word).copied().unwrap_or(0);
                let cooc = cooc_ab.min(cooc_ba);
                if cooc < 3 {
                    continue;
                }

                // Distanza 8D
                let dist = a.signature.distance(&b.signature);
                if dist < 0.15 {
                    continue; // troppo simili, non sono opposti
                }

                // Polarita: almeno 1 dimensione con diff > 0.15
                // Dopo l'apprendimento le firme convergono, ma una differenza
                // di 0.15 su una dimensione indica ancora polarita chiara.
                let a_vals = a.signature.values();
                let b_vals = b.signature.values();
                let polar_dims = (0..8).filter(|&d| (a_vals[d] - b_vals[d]).abs() > 0.15).count();
                if polar_dims < 1 {
                    continue;
                }

                // Crea l'asse: direzione B - A, normalizzata
                let mut axis = [0.0; 8];
                let mut norm = 0.0;
                for d in 0..8 {
                    axis[d] = b_vals[d] - a_vals[d];
                    norm += axis[d] * axis[d];
                }
                norm = norm.sqrt();
                if norm < 0.001 {
                    continue;
                }
                for d in 0..8 {
                    axis[d] /= norm;
                }

                // Forza dell'asse: basata su co-occorrenze e distanza
                let strength = (cooc as f64 / 10.0).min(1.0) * dist.min(1.0);

                axes.push(SemanticAxis {
                    word_a: a.word.clone(),
                    word_b: b.word.clone(),
                    axis_dims: axis,
                    strength,
                });
            }
        }

        // Ordina per forza decrescente, prendi i migliori
        axes.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
        axes.truncate(50); // max 50 assi
        axes
    }

    /// Posizione di una parola su un asse semantico.
    /// Ritorna None se la parola non e nota.
    pub fn position_on_axis(&self, word: &str, axis: &SemanticAxis) -> Option<f64> {
        self.patterns.get(&word.to_lowercase())
            .map(|pat| axis.project(&pat.signature))
    }

    /// Distanza arricchita: distanza 8D + componente assi semantici.
    /// Se non ci sono assi, fallback alla distanza 8D pura.
    pub fn enriched_distance(&self, word_a: &str, word_b: &str, axes: &[SemanticAxis]) -> Option<f64> {
        let pa = self.patterns.get(&word_a.to_lowercase())?;
        let pb = self.patterns.get(&word_b.to_lowercase())?;

        let base_dist = pa.signature.distance(&pb.signature);

        if axes.is_empty() {
            return Some(base_dist);
        }

        // Componente assi: differenza media delle proiezioni sugli assi
        let mut axis_diff_sum = 0.0;
        let mut axis_weight_sum = 0.0;
        for axis in axes {
            let proj_a = axis.project(&pa.signature);
            let proj_b = axis.project(&pb.signature);
            axis_diff_sum += (proj_a - proj_b).abs() * axis.strength;
            axis_weight_sum += axis.strength;
        }

        if axis_weight_sum > 0.0 {
            let axis_component = axis_diff_sum / axis_weight_sum;
            // 70% distanza 8D + 30% componente assi
            Some(base_dist * 0.7 + axis_component * 0.3)
        } else {
            Some(base_dist)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TENSIONI: parole che vivono sull'asse tra due opposti
// ─────────────────────────────────────────────────────────────────────────────

/// Una parola che vive sull'asse tra due opposti.
///
/// Esempio: sull'asse caldo ↔ freddo:
///   "bollente"  position=0.05 (oltre il polo caldo)
///   "tiepido"   position=0.50 (piena tensione)
///   "gelido"    position=0.95 (vicino al polo freddo)
#[derive(Debug, Clone)]
pub struct TensionWord {
    /// La parola
    pub word: String,
    /// Posizione proiettata sull'asse [0=polo_a, 1=polo_b].
    /// Puo essere < 0 o > 1 per parole "oltre" i poli (es. bollente > caldo).
    pub position: f64,
    /// Distanza 8D dall'asse (normalizzata per lunghezza asse).
    /// 0.0 = esattamente sull'asse, 1.0 = al limite della tolleranza.
    pub distance_to_axis: f64,
    /// Forza del segnale: quanto e affidabile questa posizione.
    /// 1.0 = sull'asse, decresce con la distanza.
    pub strength: f64,
}

impl Lexicon {
    /// Trova le parole che vivono sull'asse geometrico 8D tra due opposti.
    ///
    /// L'algoritmo e puramente geometrico: una parola e "di tensione" se
    /// la sua firma 8D si proietta nell'intervallo [-0.2, 1.2] sull'asse
    /// polo_a → polo_b, con distanza dall'asse < soglia.
    ///
    /// Non serve co-occorrenza — la posizione 8D E la natura della parola.
    /// "Tiepido" e tiepido PERCHE vive a meta strada tra caldo e freddo
    /// nel campo 8D, indipendentemente da come e stato insegnato.
    pub fn find_tension_words(&self, pole_a: &str, pole_b: &str) -> Vec<TensionWord> {
        let pat_a = match self.get(pole_a) { Some(p) => p, None => return Vec::new() };
        let pat_b = match self.get(pole_b) { Some(p) => p, None => return Vec::new() };

        let sig_a = pat_a.signature.values();
        let sig_b = pat_b.signature.values();

        // Vettore asse A→B
        let axis: [f64; 8] = {
            let mut v = [0.0f64; 8];
            for i in 0..8 { v[i] = sig_b[i] - sig_a[i]; }
            v
        };

        // Lunghezza al quadrato e lineare dell'asse
        let axis_len_sq: f64 = axis.iter().map(|x| x * x).sum();
        if axis_len_sq < 0.001 {
            return Vec::new(); // poli identici — asse degenere
        }
        let axis_len = axis_len_sq.sqrt();

        // Soglia di distanza massima dall'asse (assoluta in spazio 8D)
        // 0.40 permette parole vicine ma non esattamente sull'asse
        let max_dist = 0.40;

        let mut tension_words: Vec<TensionWord> = Vec::new();

        for (word, pattern) in &self.patterns {
            // Escludi i poli stessi
            if word == pole_a || word == pole_b { continue; }
            // Solo parole con un minimo di stabilita
            if pattern.stability < 0.15 { continue; }

            let sig_w = pattern.signature.values();

            // Vettore W relativo ad A
            let w_rel: [f64; 8] = {
                let mut v = [0.0f64; 8];
                for i in 0..8 { v[i] = sig_w[i] - sig_a[i]; }
                v
            };

            // Proiezione scalare di W sull'asse: t = dot(W-A, B-A) / |B-A|²
            let dot: f64 = w_rel.iter().zip(axis.iter()).map(|(x, a)| x * a).sum();
            let t = dot / axis_len_sq;

            // Ammetti anche le parole "oltre" i poli (intensificatori del polo)
            // es. "bollente" (t < 0 rispetto a caldo) o "gelido" (t > 1 rispetto a freddo)
            if t < -0.25 || t > 1.25 { continue; }

            // Punto piu vicino sull'asse (non clampato, per calcolare distanza reale)
            let t_clamped = t.clamp(0.0, 1.0);
            let mut closest = [0.0f64; 8];
            for i in 0..8 { closest[i] = sig_a[i] + t_clamped * axis[i]; }

            // Distanza 8D dal punto piu vicino sull'asse (normalizzata)
            let dist_sq: f64 = (0..8).map(|i| (sig_w[i] - closest[i]).powi(2)).sum();
            let dist_to_axis = dist_sq.sqrt() / axis_len;

            if dist_to_axis > max_dist { continue; }

            // Forza: 1.0 = sull'asse, 0.0 = al limite della tolleranza
            let strength = 1.0 - (dist_to_axis / max_dist);

            tension_words.push(TensionWord {
                word: word.clone(),
                position: t,
                distance_to_axis: dist_to_axis,
                strength,
            });
        }

        // Ordina per posizione sull'asse (dal polo_a al polo_b)
        tension_words.sort_by(|a, b| {
            a.position.partial_cmp(&b.position).unwrap_or(std::cmp::Ordering::Equal)
        });

        tension_words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::fractal::bootstrap_fractals;

    #[test]
    fn test_bootstrap_lexicon() {
        let lex = Lexicon::bootstrap();
        assert!(lex.word_count() > 400, "Bootstrap deve avere >400 parole, ha {}", lex.word_count());
        assert!(lex.knows("ciao"));
        assert!(lex.knows("acqua"));
        assert!(lex.knows("pensare"));
        assert!(lex.knows("felicità"));
    }

    #[test]
    fn test_process_known_input() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();
        let activations = lex.process_input("ciao come stai oggi", &reg);

        // "il" e function word, viene filtrata
        assert!(activations.iter().all(|a| a.word != "il"));
        // "ciao" e nota
        let ciao_act = activations.iter().find(|a| a.word == "ciao");
        assert!(ciao_act.is_some(), "ciao deve essere riconosciuta");
        assert!(ciao_act.unwrap().is_known, "ciao deve essere stabile");
    }

    #[test]
    fn test_learn_unknown_word() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();
        assert!(!lex.knows("serendipità"));

        // Prima esposizione
        lex.process_input("la serendipità è una gioia inaspettata", &reg);
        assert!(lex.knows("serendipità"));

        let pat = lex.get("serendipità").unwrap();
        assert_eq!(pat.exposure_count, 1); // 1 da new_from_context
        assert!(!pat.is_stable(), "Non puo essere stabile dopo 1 esposizione");

        // Esposizioni ripetute con contesto EMOZIONE
        for _ in 0..10 {
            lex.process_input("la serendipità porta gioia e felicità", &reg);
        }

        let pat = lex.get("serendipità").unwrap();
        assert!(pat.is_stable(), "Dopo 10+ esposizioni deve essere stabile");
        // Deve avere affinita calcolata geometricamente dalla firma
        // (non piu accumulata con EMA — la firma evolve verso EMOZIONE per contesto)
        let has_significant_affinity = pat.fractal_affinities.values().any(|&v| v > 0.5);
        assert!(has_significant_affinity,
            "Dopo 10+ esposizioni la firma deve dare affinita significative con qualche frattale");
    }

    #[test]
    fn test_similar_words() {
        let lex = Lexicon::bootstrap();
        let similar = lex.similar_words("acqua", 5);
        assert!(!similar.is_empty(), "Devono esserci parole simili a acqua");
    }

    #[test]
    fn test_function_words_filtered() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();
        let acts = lex.process_input("il la di e", &reg);
        assert!(acts.is_empty(), "Solo function words = nessuna attivazione");
    }

    #[test]
    fn test_co_occurrences() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();
        lex.process_input("gioia felicità pace", &reg);

        let gioia = lex.get("gioia").unwrap();
        assert!(gioia.co_occurrences.get("felicità").unwrap_or(&0) > &0);
        assert!(gioia.co_occurrences.get("pace").unwrap_or(&0) > &0);
    }

    #[test]
    fn test_cardinal_lexicon() {
        let lex = Lexicon::bootstrap_cardinal();
        assert_eq!(lex.word_count(), 36, "Cardinale deve avere 36 parole, ha {}", lex.word_count());
        // Parole native per ogni frattale bootstrap
        assert!(lex.knows("qui"));       // SPAZIO
        assert!(lex.knows("ora"));       // TEMPO
        assert!(lex.knows("io"));        // EGO
        assert!(lex.knows("tu"));        // RELAZIONE
        assert!(lex.knows("forse"));     // POTENZIALE
        assert!(lex.knows("limite"));    // LIMITE
        // NON deve conoscere parole del vocabolario completo
        assert!(!lex.knows("ciao"));
        assert!(!lex.knows("acqua"));
        assert!(!lex.knows("felicità"));
    }

    #[test]
    fn test_cardinal_learns_through_context() {
        let mut lex = Lexicon::bootstrap_cardinal();
        let reg = bootstrap_fractals();
        assert!(!lex.knows("acqua"));

        // Insegna "acqua" nel contesto di parole note
        lex.process_input("acqua qui vicino", &reg);

        assert!(lex.knows("acqua"), "Deve aver appreso 'acqua'");
        let pat = lex.get("acqua").unwrap();
        // Deve avere affinita con SPAZIO (id=0) calcolata dalla firma
        assert!(pat.fractal_affinities.get(&0).unwrap_or(&0.0) > &0.0,
            "Deve avere affinita con SPAZIO calcolata dalla firma");
    }

    #[test]
    fn test_semantic_axis_detection() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();

        // Crea co-occorrenze tra gioia e tristezza (servono >= 3 reciproche)
        for _ in 0..5 {
            lex.process_input("gioia tristezza dentro io", &reg);
            lex.process_input("tristezza gioia fuori tu", &reg);
        }

        let axes = lex.detect_semantic_axes();
        assert!(!axes.is_empty(), "Deve rilevare almeno un asse semantico");
    }

    #[test]
    fn test_position_on_axis() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();

        // Crea co-occorrenze
        for _ in 0..5 {
            lex.process_input("gioia tristezza dentro", &reg);
            lex.process_input("tristezza gioia fuori", &reg);
        }

        let axes = lex.detect_semantic_axes();
        let emotion_axis = axes.iter().find(|a|
            (a.word_a == "gioia" || a.word_a == "tristezza") &&
            (a.word_b == "gioia" || a.word_b == "tristezza")
        );

        if let Some(axis) = emotion_axis {
            let pos_gioia = lex.position_on_axis("gioia", axis).unwrap();
            let pos_tristezza = lex.position_on_axis("tristezza", axis).unwrap();
            // Devono essere su lati opposti dell'asse
            assert!(pos_gioia != pos_tristezza,
                "Gioia ({:.3}) e tristezza ({:.3}) devono avere proiezioni diverse",
                pos_gioia, pos_tristezza);
        }
    }

    #[test]
    fn test_enriched_distance() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();
        for _ in 0..5 {
            lex.process_input("gioia tristezza dentro io", &reg);
            lex.process_input("tristezza gioia fuori tu", &reg);
        }

        let axes = lex.detect_semantic_axes();

        // Distanza arricchita vs distanza base
        let base = lex.get("gioia").unwrap().signature.distance(
            &lex.get("tristezza").unwrap().signature
        );
        let enriched = lex.enriched_distance("gioia", "tristezza", &axes).unwrap();

        // La distanza arricchita deve essere definita
        assert!(enriched > 0.0, "Distanza arricchita deve essere > 0");
        // Con assi, dovrebbe essere diversa dalla base (non necessariamente maggiore)
        if !axes.is_empty() {
            assert!((enriched - base).abs() > 0.001 || axes.is_empty(),
                "Con assi, la distanza arricchita ({:.4}) dovrebbe differire dalla base ({:.4})",
                enriched, base);
        }
    }

    #[test]
    fn test_insert_pattern() {
        let mut lex = Lexicon::new();
        let pat = WordPattern::new_known("test", PrimitiveCore::new([0.5; 8]), vec![(0, 0.8)]);
        lex.insert_pattern("test", pat);
        assert!(lex.knows("test"));
        assert_eq!(lex.get("test").unwrap().stability, 0.6);
    }

    // ─────────────────────────────────────────────────────────────
    // TEST SISTEMA OPERATORI (si + no + quanto = X)
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn test_operator_classification() {
        // Negatori
        assert_eq!(Lexicon::classify_operator("non"), Some(OperatorKind::Negate));
        assert_eq!(Lexicon::classify_operator("senza"), Some(OperatorKind::Negate));
        assert_eq!(Lexicon::classify_operator("mai"), Some(OperatorKind::Negate));
        // Affermatori
        assert_eq!(Lexicon::classify_operator("come"), Some(OperatorKind::Affirm));
        assert_eq!(Lexicon::classify_operator("anche"), Some(OperatorKind::Affirm));
        // Quantificatori
        assert!(matches!(Lexicon::classify_operator("molto"), Some(OperatorKind::Quantify(_))));
        assert!(matches!(Lexicon::classify_operator("poco"), Some(OperatorKind::Quantify(_))));
        // Parole normali: non sono operatori
        assert_eq!(Lexicon::classify_operator("gioia"), None);
        assert_eq!(Lexicon::classify_operator("tristezza"), None);
        assert_eq!(Lexicon::classify_operator("acqua"), None);
    }

    #[test]
    fn test_co_negated_recording() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();

        // Frase con negatore TRA due parole semantiche
        lex.process_input("gioia non tristezza", &reg);
        lex.process_input("gioia non tristezza", &reg);

        let gioia = lex.get("gioia").expect("gioia deve esistere");
        let neg_count = gioia.co_negated.get("tristezza").copied().unwrap_or(0);
        assert!(neg_count > 0,
            "gioia.co_negated[tristezza] deve essere > 0, e {}",
            neg_count);

        // Co-occorrenza affermata deve essere 0 (o molto bassa) — no affermazionei
        let aff_count = gioia.co_occurrences.get("tristezza").copied().unwrap_or(0);
        assert!(aff_count < neg_count,
            "co_occurrences({}) deve essere < co_negated({}) dopo frasi negate",
            aff_count, neg_count);
    }

    #[test]
    fn test_affirmed_recording() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();

        // Frase con affermatore TRA due parole: "gioia come felicita"
        lex.process_input("gioia come felicita", &reg);
        lex.process_input("gioia come felicita", &reg);

        let gioia = lex.get("gioia").expect("gioia deve esistere");
        // Con affermatore esplicito ("come") → va in co_affirmed, NON co_occurrences
        let affirmed_count = gioia.co_affirmed.get("felicita").copied().unwrap_or(0);
        let neg_count = gioia.co_negated.get("felicita").copied().unwrap_or(0);
        assert!(affirmed_count > 0,
            "gioia.co_affirmed[felicita] deve essere > 0 con affermatore 'come'");
        assert_eq!(neg_count, 0,
            "co_negated[felicita] deve essere 0 senza negatore");
    }

    #[test]
    fn test_non_no_longer_filtered() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();

        // "non" non deve essere una function_word
        assert!(!lex.is_function_word("non"), "'non' non deve essere una function_word");

        // process_input con "non" deve rilevarlo come operatore (non come parola contenuto)
        let activations = lex.process_input("luce non buio", &reg);
        // "non" non deve apparire nelle attivazioni (e un operatore, non contenuto)
        assert!(activations.iter().all(|a| a.word != "non"),
            "'non' non deve apparire come attivazione — e un operatore");
        // "luce" e "buio" devono apparire
        assert!(activations.iter().any(|a| a.word == "luce"),
            "'luce' deve apparire nelle attivazioni");
        assert!(activations.iter().any(|a| a.word == "buio"),
            "'buio' deve apparire nelle attivazioni");
    }

    #[test]
    fn test_quantifier_boost_negation() {
        let mut lex = Lexicon::bootstrap();
        let reg = bootstrap_fractals();

        // Frase con quantificatore: "molto caldo non freddo"
        // "molto" e adiacente a "caldo", "non" e negatore tra "caldo" e "freddo"
        lex.process_input("molto caldo non freddo", &reg);

        let caldo = lex.get("caldo").expect("caldo deve esistere nel bootstrap");
        let neg_count = caldo.co_negated.get("freddo").copied().unwrap_or(0);
        // Con quantificatore 1.3, arrotondato a 1: almeno 1 co-negazione
        assert!(neg_count >= 1,
            "caldo.co_negated[freddo] deve essere >= 1 con quantificatore, e {}",
            neg_count);
    }

    // ─────────────────────────────────────────────────────────────
    // TEST TENSIONI GEOMETRICHE
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn test_find_tension_words_basic() {
        // Il bootstrap ha parole con firme 8D fisse — alcune cadono sull'asse tra poli.
        let lex = Lexicon::bootstrap();

        // Verifica che la funzione non crashi con poli noti
        let tensions = lex.find_tension_words("grande", "piccolo");
        // Non esige un numero preciso — dipende dal lessico bootstrap —
        // ma la funzione deve completare senza panic e restituire una lista ordinata.
        // Verifica ordinamento: position crescente
        for i in 1..tensions.len() {
            assert!(tensions[i].position >= tensions[i-1].position,
                "Le tensioni devono essere ordinate per posizione crescente: {} > {}",
                tensions[i-1].position, tensions[i].position);
        }
        // Tutti i campi devono essere in range ragionevole
        for tw in &tensions {
            assert!(tw.distance_to_axis >= 0.0 && tw.distance_to_axis <= 1.0,
                "distance_to_axis deve essere in [0,1]: {}", tw.distance_to_axis);
            assert!(tw.strength >= 0.0 && tw.strength <= 1.0,
                "strength deve essere in [0,1]: {}", tw.strength);
            assert!(tw.position >= -0.26 && tw.position <= 1.26,
                "position deve essere in [-0.25, 1.25]: {}", tw.position);
        }
    }

    #[test]
    fn test_find_tension_words_unknown_pole() {
        let lex = Lexicon::bootstrap();
        // Polo sconosciuto → lista vuota, no crash
        let tensions = lex.find_tension_words("xyzzyx_nonexistent", "caldo");
        assert!(tensions.is_empty(), "Polo sconosciuto deve restituire lista vuota");

        let tensions2 = lex.find_tension_words("caldo", "xyzzyx_nonexistent");
        assert!(tensions2.is_empty(), "Polo sconosciuto deve restituire lista vuota");
    }

    #[test]
    fn test_find_tension_words_poles_excluded() {
        let lex = Lexicon::bootstrap();
        // I poli stessi non devono apparire nella lista delle tensioni
        let tensions = lex.find_tension_words("gioia", "tristezza");
        assert!(tensions.iter().all(|tw| tw.word != "gioia"),
            "'gioia' non deve apparire nelle tensioni di gioia↔tristezza");
        assert!(tensions.iter().all(|tw| tw.word != "tristezza"),
            "'tristezza' non deve apparire nelle tensioni di gioia↔tristezza");
    }

    #[test]
    fn test_find_tension_words_position_semantics() {
        // Costruiamo un lessico con firme controllate per testare la proiezione.
        // Polo A: [0.0, ...], Polo B: [1.0, ...], Parola media: [0.5, ...]
        let mut lex = Lexicon::new();

        // Due poli opposti sulla prima dimensione
        let sig_a = PrimitiveCore::new([0.1, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        let sig_b = PrimitiveCore::new([0.9, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        // Parola esattamente a meta asse
        let sig_mid = PrimitiveCore::new([0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        // Parola vicina al polo A
        let sig_near_a = PrimitiveCore::new([0.2, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);

        let mut pat_a = WordPattern::new_known("polo_a", sig_a, vec![]);
        pat_a.stability = 0.8;
        let mut pat_b = WordPattern::new_known("polo_b", sig_b, vec![]);
        pat_b.stability = 0.8;
        let mut pat_mid = WordPattern::new_known("meta", sig_mid, vec![]);
        pat_mid.stability = 0.8;
        let mut pat_near_a = WordPattern::new_known("vicino_a", sig_near_a, vec![]);
        pat_near_a.stability = 0.8;

        lex.insert_pattern("polo_a", pat_a);
        lex.insert_pattern("polo_b", pat_b);
        lex.insert_pattern("meta", pat_mid);
        lex.insert_pattern("vicino_a", pat_near_a);

        let tensions = lex.find_tension_words("polo_a", "polo_b");

        // "meta" deve essere circa a 0.5
        let meta = tensions.iter().find(|tw| tw.word == "meta")
            .expect("'meta' deve apparire nelle tensioni");
        assert!((meta.position - 0.5).abs() < 0.05,
            "La parola 'meta' deve essere a posizione ~0.5, e {:.3}", meta.position);

        // "vicino_a" deve essere a circa 0.1-0.2
        let near_a = tensions.iter().find(|tw| tw.word == "vicino_a")
            .expect("'vicino_a' deve apparire nelle tensioni");
        assert!(near_a.position < 0.3,
            "La parola 'vicino_a' deve essere < 0.3, e {:.3}", near_a.position);

        // "vicino_a" deve essere prima di "meta" (ordinamento)
        let idx_near = tensions.iter().position(|tw| tw.word == "vicino_a").unwrap();
        let idx_meta = tensions.iter().position(|tw| tw.word == "meta").unwrap();
        assert!(idx_near < idx_meta, "'vicino_a' deve venire prima di 'meta' nell'ordinamento");
    }
}
