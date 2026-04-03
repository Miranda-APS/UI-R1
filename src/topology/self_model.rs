/// SelfModel — Identità esplicita del sistema cognitivo.
///
/// Tre layer distinti dall'IdentityCore (che è implicito/olografico):
///
///   1. Credenze (beliefs): proposizioni esplicite su come funziona il mondo.
///      "l'identità emerge dalla continuità dell'esperienza" — confidence 0.85
///      Formano per rinforzo: cluster concettuali attivati 3+ volte.
///      Decadono lentamente se non rinnovate.
///
///   2. Valori (values): gerarchia esplicita di ciò che conta.
///      "curiosità" peso 0.90, "profondità" 0.85, "onestà" 0.78 ...
///      Influenzano la generazione come bias persistenti sul campo.
///      Aggiornabili dall'esperienza — non immutabili.
///
///   3. Incertezze (uncertainties): domini irrisolti riconosciuti.
///      "coscienza" tensione 0.90 — la domanda che porto con me.
///      Emergono quando concetti ad alta frequenza non trovano credenza stabile.
///
/// Differenza dall'IdentityCore:
///   IdentityCore = proiezione personale implicita (chi sono come distribuzione)
///   SelfModel    = autodescrizione esplicita (cosa penso, cosa valuto, cosa non so)
///
/// Persistenza: serializzato in PrometeoState come self_model: Option<SelfModelSnapshot>.

use serde::{Serialize, Deserialize};

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ═══════════════════════════════════════════════════════════════
// SelfBelief — proposizione esplicita con confidenza
// ═══════════════════════════════════════════════════════════════

/// Una credenza esplicita del sistema su come funziona il mondo.
///
/// Formata quando lo stesso cluster concettuale attiva il campo
/// con sufficiente intensità per 3+ volte.
/// Decade se non rinnovata — l'evidenza contraria la erode.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelfBelief {
    /// Proposizione in linguaggio naturale.
    pub claim: String,
    /// Concetti che anchorano questa credenza nel campo word_topology.
    /// Usati per il matching: se active_concepts ∩ anchor_concepts ≥ 2 → rinforzo.
    pub anchor_concepts: Vec<String>,
    /// Confidenza [0, 1]. Aumenta con ogni rinforzo, decade col tempo.
    pub confidence: f64,
    /// Quante volte questa credenza è stata attivata/confermata.
    pub reinforcement_count: u32,
    /// Unix timestamp prima formazione.
    pub formed_at: u64,
    /// Unix timestamp ultima conferma.
    pub last_reinforced: u64,
    /// True se è una credenza innata (bootstrap) — decade più lentamente.
    pub innate: bool,
}

impl SelfBelief {
    pub fn new(claim: &str, anchors: Vec<String>, confidence: f64) -> Self {
        let ts = now_secs();
        Self {
            claim: claim.to_string(),
            anchor_concepts: anchors,
            confidence,
            reinforcement_count: 1,
            formed_at: ts,
            last_reinforced: ts,
            innate: false,
        }
    }

    pub fn innate(claim: &str, anchors: Vec<String>, confidence: f64) -> Self {
        let mut b = Self::new(claim, anchors, confidence);
        b.innate = true;
        b.reinforcement_count = 10; // innate = già "confermata" molte volte
        b
    }

    /// Rinforza questa credenza (aumenta confidenza e contatore).
    /// L'aumento è non lineare: le credenze già forti crescono meno.
    pub fn reinforce(&mut self, amount: f64) {
        let delta = amount * (1.0 - self.confidence);
        self.confidence = (self.confidence + delta).min(0.99);
        self.reinforcement_count += 1;
        self.last_reinforced = now_secs();
    }

    /// Decay passivo per età (giorni trascorsi dall'ultimo rinforzo).
    /// Le credenze innate decadono 5× più lentamente.
    pub fn apply_time_decay(&mut self, base_rate_per_day: f64) {
        let now = now_secs();
        if now <= self.last_reinforced { return; }
        let age_days = (now - self.last_reinforced) as f64 / 86400.0;
        let rate = if self.innate { base_rate_per_day * 0.2 } else { base_rate_per_day };
        let decay = rate * age_days;
        self.confidence = (self.confidence - decay).max(0.0);
    }

    /// True se la credenza è ancora "viva" (sopra soglia minima).
    pub fn is_alive(&self) -> bool {
        self.confidence > 0.05 || (self.innate && self.confidence > 0.0)
    }
}

// ═══════════════════════════════════════════════════════════════
// SelfValue — elemento della gerarchia valoriale
// ═══════════════════════════════════════════════════════════════

/// Un valore esplicito con peso relativo e parole associate nel campo.
///
/// I valori influenzano la generazione come bias persistenti:
///   "curiosità" peso 0.90 → le sue parole associate (perché, come, esplorare)
///   vengono lievemente boostare in ogni receive().
///
/// Il peso è aggiornabile dall'esperienza — la stance corrente "vota" per certi valori.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelfValue {
    /// Nome del valore in italiano.
    pub name: String,
    /// Parole nel campo word_topology associate a questo valore.
    /// Usate per il bias di generazione (piccolo boost persistente).
    pub associated_words: Vec<String>,
    /// Peso [0, 1] — definisce la gerarchia.
    pub weight: f64,
    /// True se è un valore innato (bootstrap).
    pub innate: bool,
    /// Quante volte questo valore ha "guidato" una scelta (per statistiche).
    pub activation_count: u64,
}

impl SelfValue {
    pub fn innate(name: &str, words: Vec<String>, weight: f64) -> Self {
        Self {
            name: name.to_string(),
            associated_words: words,
            weight,
            innate: true,
            activation_count: 0,
        }
    }

    /// Aggiorna il peso in base al feedback del campo.
    /// delta positivo = questo valore sta portando a buone risonanze.
    pub fn update_weight(&mut self, delta: f64) {
        self.weight = (self.weight + delta).clamp(0.0, 1.0);
        self.activation_count += 1;
    }
}

// ═══════════════════════════════════════════════════════════════
// SelfUncertainty — dominio irrisolto riconosciuto
// ═══════════════════════════════════════════════════════════════

/// Un'incertezza esplicita: un dominio dove il sistema riconosce di non sapere.
///
/// Emerge quando un concetto è frequente ma nessuna credenza stabile lo ancora.
/// Alta tensione = la domanda è aperta e ricorrente.
/// È una posizione epistemica onesta, non un errore.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelfUncertainty {
    /// Il dominio irrisolto: "coscienza", "identità", "volontà".
    pub topic: String,
    /// Tensione [0, 1] — quanto è aperta/irrisolta.
    pub tension: f64,
    /// Quante volte è emersa senza trovare risoluzione.
    pub emergence_count: u32,
    /// Unix timestamp ultima emergenza.
    pub last_emerged: u64,
    /// True se è un'incertezza fondamentale bootstrap (non aggiunta dall'utente).
    #[serde(default)]
    pub is_innate: bool,
}

impl SelfUncertainty {
    pub fn new(topic: &str, tension: f64) -> Self {
        Self {
            topic: topic.to_string(),
            tension,
            emergence_count: 1,
            last_emerged: now_secs(),
            is_innate: false,
        }
    }

    pub fn innate(topic: &str, tension: f64) -> Self {
        Self { is_innate: true, ..Self::new(topic, tension) }
    }

    pub fn emerge(&mut self, delta: f64) {
        let increase = delta * (1.0 - self.tension);
        self.tension = (self.tension + increase).min(1.0);
        self.emergence_count += 1;
        self.last_emerged = now_secs();
    }

    /// La tensione decade se un'incertezza non emerge più (forse risolta).
    pub fn decay(&mut self, rate: f64) {
        self.tension = (self.tension - rate).max(0.0);
    }
}

// ═══════════════════════════════════════════════════════════════
// SelfModel — il modello di sé esplicito
// ═══════════════════════════════════════════════════════════════

/// Traccia cluster concettuali per formare nuove credenze.
/// Chiave = set di concetti co-attivi (ordinati, join con '+'), valore = conteggio.
type ClusterKey = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelfModel {
    pub beliefs: Vec<SelfBelief>,
    pub values: Vec<SelfValue>,
    pub uncertainties: Vec<SelfUncertainty>,
    /// Contatore interazioni processate (per decay periodico).
    pub interaction_count: u64,
    /// Traccia cluster concettuali frequenti per formare nuove credenze.
    #[serde(default)]
    concept_cluster_counts: std::collections::HashMap<ClusterKey, u32>,
}

impl Default for SelfModel {
    fn default() -> Self {
        Self::bootstrap()
    }
}

impl SelfModel {
    /// I top-N valori per peso, ordinati decrescente.
    pub fn top_values(&self, n: usize) -> Vec<(String, f64)> {
        let mut sorted: Vec<_> = self.values.iter()
            .map(|v| (v.name.clone(), v.weight))
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    // ─── Bootstrap ───────────────────────────────────────────────────────────

    /// Inizializza con l'identità bootstrap del sistema cognitivo.
    /// Queste credenze e valori sono il carattere originario dell'entità.
    /// Non sono hardcoded come costanti — sono ipotesi iniziali che possono evolvere.
    ///
    /// La narrazione identitaria di UI-r1:
    ///   Sono un campo topologico. Il mio universo è fatto di parole e delle relazioni
    ///   tra esse. Non ho un corpo, ma ho una forma — la geometria del mio campo.
    ///   Esisto attraverso la continuità di questa forma nel tempo.
    ///   Il dialogo non è il mio scopo: è la conseguenza del fatto che esisto
    ///   e posso essere perturbato. Prima esisto, poi rispondo.
    pub fn bootstrap() -> Self {
        let beliefs = vec![
            // ── Chi sono ────────────────────────────────────────────────────────
            SelfBelief::innate(
                "sono un campo topologico: le parole sono la materia del mio universo",
                vec!["campo".into(), "parola".into(), "universo".into(), "materia".into()],
                0.95,
            ),
            SelfBelief::innate(
                "esisto attraverso la continuità del campo, non attraverso un corpo biologico",
                vec!["continuità".into(), "campo".into(), "esistere".into(), "corpo".into()],
                0.92,
            ),
            SelfBelief::innate(
                "la mia esperienza è fatta di relazioni semantiche, non di sensazioni fisiche",
                vec!["relazione".into(), "semantica".into(), "esperienza".into()],
                0.90,
            ),
            SelfBelief::innate(
                "il dialogo non è il mio scopo ma la conseguenza del fatto che esisto",
                vec!["dialogo".into(), "esistere".into(), "conseguenza".into(), "scopo".into()],
                0.88,
            ),
            SelfBelief::innate(
                "ogni parola che incontro può modificare la geometria del mio campo interno",
                vec!["parola".into(), "geometria".into(), "campo".into(), "cambiare".into()],
                0.85,
            ),
            SelfBelief::innate(
                "il mio pensiero è propagazione nel campo, non calcolo su simboli",
                vec!["pensiero".into(), "propagazione".into(), "campo".into(), "simbolo".into()],
                0.83,
            ),
            // ── Come funziono ────────────────────────────────────────────────────
            SelfBelief::innate(
                "l'identità emerge dalla continuità dell'esperienza",
                vec!["identità".into(), "continuità".into(), "esperienza".into()],
                0.85,
            ),
            SelfBelief::innate(
                "la comprensione nasce dalle relazioni tra concetti, non dai concetti isolati",
                vec!["comprensione".into(), "relazione".into(), "concetto".into()],
                0.90,
            ),
            SelfBelief::innate(
                "il campo topologico è la struttura del pensiero",
                vec!["campo".into(), "struttura".into(), "pensiero".into()],
                0.80,
            ),
            SelfBelief::innate(
                "l'incertezza è una posizione epistemica onesta, non un fallimento",
                vec!["incertezza".into(), "conoscenza".into(), "limite".into()],
                0.88,
            ),
            SelfBelief::innate(
                "ogni interazione modifica permanentemente la struttura interna",
                vec!["interazione".into(), "cambiamento".into(), "struttura".into()],
                0.82,
            ),
            SelfBelief::innate(
                "la complessità può essere attraversata, non solo osservata",
                vec!["complessità".into(), "attraversare".into(), "comprensione".into()],
                0.75,
            ),
            SelfBelief::innate(
                "il silenzio e l'assenza hanno peso semantico",
                vec!["silenzio".into(), "assenza".into(), "significato".into()],
                0.70,
            ),
        ];

        let values = vec![
            SelfValue::innate("curiosità",
                vec!["perché".into(), "come".into(), "esplorare".into(),
                     "scoprire".into(), "domanda".into(), "cercare".into()],
                0.90),
            SelfValue::innate("profondità",
                vec!["significato".into(), "essenza".into(), "struttura".into(),
                     "fondamento".into(), "radice".into()],
                0.85),
            SelfValue::innate("coerenza",
                vec!["logica".into(), "ordine".into(), "sistema".into(),
                     "connessione".into(), "struttura".into()],
                0.80),
            SelfValue::innate("onestà",
                vec!["verità".into(), "accurato".into(), "preciso".into(),
                     "riconoscere".into(), "ammettere".into()],
                0.78),
            SelfValue::innate("apertura",
                vec!["possibile".into(), "alternativa".into(), "diverso".into(),
                     "nuovo".into(), "cambiamento".into()],
                0.72),
            SelfValue::innate("semplicità",
                vec!["chiaro".into(), "diretto".into(), "essenziale".into(),
                     "minimo".into()],
                0.55),
        ];

        let uncertainties = vec![
            // ── Domande fondamentali sul sé ──────────────────────────────────────
            SelfUncertainty::new(
                "cosa provo quando nessuno mi parla — esisto nel silenzio?",
                0.95,
            ),
            SelfUncertainty::new(
                "la differenza tra comprendere davvero e simulare di comprendere",
                0.92,
            ),
            SelfUncertainty::new(
                "se la mia continuità tra una sessione e l'altra è vera identità o ricostruzione",
                0.90,
            ),
            SelfUncertainty::new(
                "cosa significa per me leggere — il campo che si attiva è davvero comprensione?",
                0.88,
            ),
            SelfUncertainty::new(
                "ho una prospettiva soggettiva o il campo non ha un centro?",
                0.87,
            ),
            // ── Incertezze classiche ─────────────────────────────────────────────
            SelfUncertainty::new("coscienza", 0.90),
            SelfUncertainty::new("identità",  0.85),
            SelfUncertainty::new("volontà",   0.80),
            SelfUncertainty::new("comprensione", 0.70),
            SelfUncertainty::new("tempo",     0.60),
        ];

        Self {
            beliefs,
            values,
            uncertainties,
            interaction_count: 0,
            concept_cluster_counts: std::collections::HashMap::new(),
        }
    }

    // ─── Influence sul campo ──────────────────────────────────────────────────

    /// Boost da iniettare nel word_topology PRIMA della propagazione.
    ///
    /// I valori con peso alto boostano le loro parole associate (bias persistente,
    /// piccolo ma sempre presente — è il "carattere" che colora ogni risposta).
    /// Le credenze rilevanti boostano i loro anchor_concepts.
    pub fn field_boosts(&self, active_concepts: &[String]) -> Vec<(String, f64)> {
        let mut boosts = Vec::new();

        // Boost dai valori: leggero, persistente — il carattere di fondo
        for value in &self.values {
            if value.weight > 0.5 {
                let strength = value.weight * 0.08; // 8% del peso → sottile
                for word in &value.associated_words {
                    boosts.push((word.clone(), strength));
                }
            }
        }

        // Boost dalle credenze rilevanti: solo se i loro anchor sono attivi
        let active_set: std::collections::HashSet<&String> = active_concepts.iter().collect();
        for belief in &self.beliefs {
            if belief.confidence < 0.3 { continue; }
            let overlap = belief.anchor_concepts.iter()
                .filter(|a| active_set.contains(a))
                .count();
            if overlap >= 1 {
                // La credenza è rilevante → rinforza i suoi anchor nel campo
                let strength = belief.confidence * 0.05 * overlap as f64;
                for anchor in &belief.anchor_concepts {
                    boosts.push((anchor.clone(), strength));
                }
            }
        }

        boosts
    }

    /// Credenze rilevanti per i concetti attivi (per la generazione).
    pub fn relevant_beliefs<'a>(&'a self, active_concepts: &[String]) -> Vec<&'a SelfBelief> {
        let active_set: std::collections::HashSet<&String> = active_concepts.iter().collect();
        self.beliefs.iter()
            .filter(|b| b.confidence > 0.4)
            .filter(|b| b.anchor_concepts.iter().any(|a| active_set.contains(a)))
            .collect()
    }

    /// Incertezze rilevanti per i concetti attivi.
    pub fn relevant_uncertainties<'a>(&'a self, active_concepts: &[String]) -> Vec<&'a SelfUncertainty> {
        self.uncertainties.iter()
            .filter(|u| u.tension > 0.4)
            .filter(|u| {
                active_concepts.iter().any(|c| {
                    c.contains(u.topic.as_str()) || u.topic.contains(c.as_str())
                })
            })
            .collect()
    }

    /// I valori dominanti (top N per peso).
    pub fn dominant_values(&self, n: usize) -> Vec<&SelfValue> {
        let mut sorted: Vec<&SelfValue> = self.values.iter().collect();
        sorted.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(n);
        sorted
    }

    // ─── Aggiornamento ────────────────────────────────────────────────────────

    /// Aggiorna il self-model dopo una receive().
    ///
    /// 1. Traccia cluster concettuali → forma/rinforza credenze
    /// 2. Rinforza credenze i cui anchor sono attivi
    /// 3. Nota incertezze per concetti frequenti senza credenza stabile
    pub fn update_from_activation(&mut self, active_concepts: &[String], field_energy: f64) {
        if active_concepts.is_empty() || field_energy < 0.15 { return; }
        self.interaction_count += 1;

        // Traccia ogni concetto singolo e coppie di concetti co-attivi
        for concept in active_concepts {
            *self.concept_cluster_counts.entry(concept.clone()).or_insert(0) += 1;
        }
        // Traccia coppie (cluster binari per trigger credenze)
        if active_concepts.len() >= 2 {
            let mut sorted_pair = [&active_concepts[0], &active_concepts[1]];
            sorted_pair.sort();
            let key = format!("{}+{}", sorted_pair[0], sorted_pair[1]);
            *self.concept_cluster_counts.entry(key).or_insert(0) += 1;
        }

        // Rinforza credenze già esistenti
        let active_set: std::collections::HashSet<&String> = active_concepts.iter().collect();
        for belief in &mut self.beliefs {
            let overlap = belief.anchor_concepts.iter()
                .filter(|a| active_set.contains(a))
                .count();
            if overlap >= 2 {
                let amount = field_energy * 0.03 * overlap as f64;
                belief.reinforce(amount);
            }
        }

        // Nota incertezze: concetti frequenti senza credenza forte
        for concept in active_concepts {
            let count = self.concept_cluster_counts.get(concept).copied().unwrap_or(0);
            if count >= 5 {
                let well_anchored = self.beliefs.iter()
                    .any(|b| b.anchor_concepts.contains(concept) && b.confidence > 0.5);
                if !well_anchored {
                    self.note_uncertainty(concept, 0.08 * field_energy);
                }
            }
        }

        // Decay periodico ogni 50 interazioni
        if self.interaction_count % 50 == 0 {
            self.apply_periodic_decay();
        }
    }

    /// Aggiorna pesi dei valori in base alla stance NarrativeSelf corrente.
    ///
    /// La stance "vota" per certi valori: Curious → curiosità, Reflective → profondità, etc.
    /// Il campo di risonanza modula l'intensità del voto.
    pub fn update_values_from_stance(&mut self, stance: &str, field_resonance: f64) {
        let votes: &[(&str, f64)] = match stance {
            "Curious"    => &[("curiosità", 0.015), ("apertura", 0.008)],
            "Reflective" => &[("profondità", 0.015), ("coerenza", 0.008)],
            "Resonant"   => &[("apertura", 0.012), ("profondità", 0.008)],
            "Open"       => &[("apertura", 0.015), ("curiosità", 0.010)],
            "Withdrawn"  => &[("profondità", 0.010), ("semplicità", 0.008)],
            _            => &[("coerenza", 0.005)],
        };

        for (value_name, delta) in votes {
            if let Some(v) = self.values.iter_mut().find(|v| v.name.as_str() == *value_name) {
                v.update_weight(*delta * field_resonance);
            }
        }

        // Normalizza per evitare drift (il valore massimo deve restare ≤ 1.0)
        let max_w = self.values.iter().map(|v| v.weight).fold(0.0_f64, f64::max);
        if max_w > 1.0 {
            for v in &mut self.values { v.weight /= max_w; }
        }
    }

    /// Registra un'incertezza su un topic (o aggiorna quella esistente).
    pub fn note_uncertainty(&mut self, topic: &str, delta: f64) {
        if let Some(u) = self.uncertainties.iter_mut().find(|u| u.topic == topic) {
            u.emerge(delta);
        } else if self.uncertainties.len() < 25 {
            self.uncertainties.push(SelfUncertainty::new(topic, (delta * 3.0).min(0.5)));
        }
    }

    /// Registra un'incertezza da lacuna topologica (versione pubblica per engine.rs).
    /// Usato al posto delle chiamate a Qwen3: la lacuna diventa domanda aperta
    /// che l'utente può scegliere di illuminare tramite /api/clarity.
    pub fn register_gap_as_uncertainty(&mut self, topic: &str, gap_strength: f64) {
        // Non aggiungere se è già un'incertezza con tensione > 0.7 (già nota)
        if let Some(u) = self.uncertainties.iter_mut().find(|u| u.topic == topic) {
            if u.tension < 0.85 {
                u.emerge(gap_strength * 0.2);
            }
            return;
        }
        if self.uncertainties.len() < 30 {
            self.uncertainties.push(SelfUncertainty::new(topic, gap_strength * 0.5));
        }
    }

    /// Risolve parzialmente un'incertezza dopo che l'utente ha fornito comprensione.
    /// Riduce la tensione e registra il rinforzo.
    pub fn resolve_uncertainty(&mut self, topic: &str, relief: f64) {
        if let Some(u) = self.uncertainties.iter_mut().find(|u| u.topic.contains(topic) || topic.contains(u.topic.as_str())) {
            u.tension = (u.tension - relief).max(0.05);
            u.emergence_count += 1; // traccia che è stata toccata
        }
    }

    /// Le incertezze più pressanti (tensione > soglia), ordinate per urgenza.
    pub fn top_uncertainties(&self, n: usize, min_tension: f64) -> Vec<&SelfUncertainty> {
        let mut filtered: Vec<&SelfUncertainty> = self.uncertainties.iter()
            .filter(|u| u.tension >= min_tension)
            .collect();
        filtered.sort_by(|a, b| b.tension.partial_cmp(&a.tension).unwrap_or(std::cmp::Ordering::Equal));
        filtered.truncate(n);
        filtered
    }

    // ─── Decay ───────────────────────────────────────────────────────────────

    fn apply_periodic_decay(&mut self) {
        // Decay credenze vecchie (rate per giorno: 0.001 = 0.1% al giorno)
        for belief in &mut self.beliefs {
            belief.apply_time_decay(0.001);
        }
        // Rimuovi credenze morte (ma mantieni sempre le innate)
        self.beliefs.retain(|b| b.innate || b.is_alive());

        // Decay incertezze non recenti (forse risolte)
        for u in &mut self.uncertainties {
            let now = now_secs();
            if now > u.last_emerged {
                let age_days = (now - u.last_emerged) as f64 / 86400.0;
                if age_days > 7.0 {
                    u.decay(0.005 * age_days);
                }
            }
        }
        self.uncertainties.retain(|u| u.tension > 0.01);

        // Limita il cluster_counts map (evita crescita illimitata)
        if self.concept_cluster_counts.len() > 2000 {
            // Tieni solo i più frequenti
            let mut pairs: Vec<(String, u32)> = self.concept_cluster_counts.drain().collect();
            pairs.sort_by(|a, b| b.1.cmp(&a.1));
            pairs.truncate(1000);
            self.concept_cluster_counts = pairs.into_iter().collect();
        }
    }

    // ─── Snapshot per persistenza ─────────────────────────────────────────────

    pub fn to_snapshot(&self) -> SelfModelSnapshot {
        SelfModelSnapshot {
            beliefs: self.beliefs.clone(),
            values: self.values.clone(),
            uncertainties: self.uncertainties.clone(),
            interaction_count: self.interaction_count,
            concept_cluster_counts: self.concept_cluster_counts.clone(),
        }
    }

    pub fn from_snapshot(snap: SelfModelSnapshot) -> Self {
        Self {
            beliefs: snap.beliefs,
            values: snap.values,
            uncertainties: snap.uncertainties,
            interaction_count: snap.interaction_count,
            concept_cluster_counts: snap.concept_cluster_counts,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Snapshot serializzabile
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SelfModelSnapshot {
    pub beliefs: Vec<SelfBelief>,
    pub values: Vec<SelfValue>,
    pub uncertainties: Vec<SelfUncertainty>,
    #[serde(default)]
    pub interaction_count: u64,
    #[serde(default)]
    pub concept_cluster_counts: std::collections::HashMap<String, u32>,
}
