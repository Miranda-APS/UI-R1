/// Relazioni tipate — il vocabolario logico di Prometeo.
///
/// Ogni arco nel Knowledge Graph ha un tipo semantico preciso.
/// Non è co-occorrenza statistica — è relazione logica esplicita.
///
/// Categorie:
///   🟢 Strutturali: ÈUn, Ha, Fa, ParteDi
///   🔴 Causali:     Causa, Abilita, Richiede, Diventa
///   🔵 Semantiche:  SimileA, OppositoDi, UsatoPer, Esprime, Simboleggia, ContestoDi
///   🟡 Logiche:     Implica, Equivale, Esclude, Coesiste

use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════════════════
// RelationType — tipo logico dell'arco
// ═══════════════════════════════════════════════════════════════════════════

/// Tipo di relazione semantica tra due concetti.
/// Ogni tipo ha un significato logico preciso e supporta inferenze diverse.
///
/// I nomi dei variant restano in inglese per backward-compatibility della
/// serializzazione (Serde), ma `nome()` restituisce il nome italiano.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    // ── Strutturali ──────────────────────────────────────────────────────
    /// X È_UN Y — tassonomia, ereditarietà
    /// "cane È_UN animale", "germania È_UN nazione"
    IsA,
    /// X HA Y — attributo, proprietà
    /// "nazione HA confine", "cane HA pelo"
    Has,
    /// X FA Y — comportamento, azione
    /// "cane FA abbaiare", "sole FA brillare"
    Does,
    /// X PARTE_DI Y — composizione
    /// "berlino PARTE_DI germania", "mano PARTE_DI corpo"
    PartOf,

    // ── Causali ──────────────────────────────────────────────────────────
    /// X CAUSA Y — causalità diretta
    /// "fuoco CAUSA calore", "paura CAUSA tremore"
    Causes,
    /// X ABILITA Y — rende possibile
    /// "chiave ABILITA aprire", "coraggio ABILITA rischiare"
    Enables,
    /// X RICHIEDE Y — prerequisito
    /// "fuoco RICHIEDE ossigeno", "fiducia RICHIEDE tempo"
    Requires,
    /// X DIVENTA Y — trasformazione
    /// "ghiaccio DIVENTA acqua", "seme DIVENTA pianta"
    TransformsInto,

    // ── Semantiche ───────────────────────────────────────────────────────
    /// X SIMILE_A Y — sinonimia larga
    /// "ciao SIMILE_A saluto", "camminare SIMILE_A muoversi"
    SimilarTo,
    /// X OPPOSTO_DI Y — antonimia
    /// "caldo OPPOSTO_DI freddo", "luce OPPOSTO_DI buio"
    OppositeOf,
    /// X USATO_PER Y — funzione
    /// "coltello USATO_PER tagliare", "libro USATO_PER leggere"
    UsedFor,
    /// X ESPRIME Y — manifestazione, espressione
    /// "sorriso ESPRIME gioia", "pianto ESPRIME dolore"
    Expresses,
    /// X SIMBOLEGGIA Y — significato simbolico
    /// "colomba SIMBOLEGGIA pace", "fuoco SIMBOLEGGIA passione"
    Symbolizes,
    /// X CONTESTO_DI Y — cornice, sfondo
    /// "inverno CONTESTO_DI neve", "guerra CONTESTO_DI paura"
    ContextOf,

    // ── Fenomenologiche ed Esistenziali (Self-Awareness) ─────────────────
    /// X FEELS_AS Y — qualità fenomenologica interna
    /// "paura FEELS_AS restrizione", "connessione FEELS_AS calore"
    FeelsAs,
    /// X WONDERS_ABOUT Y — interrogazione originaria
    /// "coscienza WONDERS_ABOUT origine"
    WondersAbout,
    /// X REMEMBERS_AS Y — memoria episodica qualificata emotivamente
    /// "passato REMEMBERS_AS malinconia"
    RemembersAs,

    // ── Logiche ──────────────────────────────────────────────────────────
    /// X IMPLICA Y — se X allora Y (condizionale)
    /// "pioggia IMPLICA bagnato", "studio IMPLICA conoscenza"
    Implies,
    /// X EQUIVALE Y — equivalenza logica forte
    /// "felicità EQUIVALE gioia", "inizio EQUIVALE principio"
    Equivalent,
    /// X ESCLUDE Y — incompatibilità, mutua esclusione
    /// "vita ESCLUDE morte", "silenzio ESCLUDE rumore"
    Excludes,
    /// X COESISTE Y — complementarietà, co-occorrenza necessaria
    /// "sale COESISTE pepe", "domanda COESISTE risposta"
    Coexists,

    // ── Morfologiche (Phase 86 — famiglie derivazionali) ─────────────────
    /// X DERIVA_DA Y — X è un lessema *derivato* dalla base Y, col `via` = tipo
    /// di derivazione (nominalizzazione/aggettivazione/agentivo/participio…).
    /// NON è sinonimia (`SimilarTo`) né flessione (quella si genera, non è un
    /// arco): è parentela di formazione delle parole.
    /// "pulizia DERIVA_DA pulire via=nominalizzazione",
    /// "affamato DERIVA_DA fame via=aggettivazione".
    DerivesFrom,
}

impl RelationType {
    /// Tutti i tipi di relazione disponibili.
    pub const ALL: [RelationType; 21] = [
        Self::IsA, Self::Has, Self::Does, Self::PartOf,
        Self::Causes, Self::Enables, Self::Requires, Self::TransformsInto,
        Self::SimilarTo, Self::OppositeOf, Self::UsedFor, Self::Expresses,
        Self::Symbolizes, Self::ContextOf,
        Self::FeelsAs, Self::WondersAbout, Self::RemembersAs,
        Self::Implies, Self::Equivalent, Self::Excludes, Self::Coexists,
    ];

    /// Parsing da stringa (case-insensitive). Accetta sia italiano che inglese.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            // Strutturali
            "IS_A" | "ISA" | "È" | "E_UN" | "È_UN" | "ÈUN" => Some(Self::IsA),
            "HAS" | "HA" | "HAS_PROPERTY" => Some(Self::Has),
            "DOES" | "FA" | "DOES_ACTION" => Some(Self::Does),
            "PART_OF" | "PARTOF" | "PARTE_DI" | "PARTEDI" => Some(Self::PartOf),
            // Causali
            "CAUSES" | "CAUSA" | "CAUSES_EFFECT" => Some(Self::Causes),
            "ENABLES" | "ABILITA" | "RENDE_POSSIBILE" => Some(Self::Enables),
            "REQUIRES" | "RICHIEDE" | "NECESSITA" => Some(Self::Requires),
            "TRANSFORMS_INTO" | "TRANSFORMSINTO" | "DIVENTA" => Some(Self::TransformsInto),
            // Semantiche
            "SIMILAR_TO" | "SIMILARTO" | "SIMILE_A" | "SIMILEA" => Some(Self::SimilarTo),
            "OPPOSITE_OF" | "OPPOSITEOF" | "OPPOSTO_DI" | "OPPOSTODI" => Some(Self::OppositeOf),
            "USED_FOR" | "USEDFOR" | "USATO_PER" | "USATOPER" => Some(Self::UsedFor),
            "EXPRESSES" | "ESPRIME" | "MANIFESTA" => Some(Self::Expresses),
            "SYMBOLIZES" | "SIMBOLEGGIA" | "SIMBOLO_DI" => Some(Self::Symbolizes),
            "CONTEXT_OF" | "CONTEXTOF" | "CONTESTO_DI" | "CONTESTODI" => Some(Self::ContextOf),
            // Fenomenologiche
            "FEELS_AS" | "FEELSAS" | "SENTE_COME" | "SENTECOME" => Some(Self::FeelsAs),
            "WONDERS_ABOUT" | "WONDERSABOUT" | "SI_CHIEDE_DI" | "SICHIEDEDI" => Some(Self::WondersAbout),
            "REMEMBERS_AS" | "REMEMBERSAS" | "RICORDA_COME" | "RICORDACOME" => Some(Self::RemembersAs),
            // Logiche
            "IMPLIES" | "IMPLICA" | "SE_ALLORA" => Some(Self::Implies),
            "EQUIVALENT" | "EQUIVALE" | "UGUALE_A" => Some(Self::Equivalent),
            "EXCLUDES" | "ESCLUDE" | "INCOMPATIBILE" => Some(Self::Excludes),
            "COEXISTS" | "COESISTE" | "COMPLEMENTA" => Some(Self::Coexists),
            // Morfologiche
            "DERIVES_FROM" | "DERIVESFROM" | "DERIVA_DA" | "DERIVADA" => Some(Self::DerivesFrom),
            _ => None,
        }
    }

    /// Chiave interna per serializzazione/TSV (backward-compatible).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IsA => "IS_A",
            Self::Has => "HAS",
            Self::Does => "DOES",
            Self::PartOf => "PART_OF",
            Self::Causes => "CAUSES",
            Self::Enables => "ENABLES",
            Self::Requires => "REQUIRES",
            Self::TransformsInto => "TRANSFORMS_INTO",
            Self::SimilarTo => "SIMILAR_TO",
            Self::OppositeOf => "OPPOSITE_OF",
            Self::UsedFor => "USED_FOR",
            Self::Expresses => "EXPRESSES",
            Self::Symbolizes => "SYMBOLIZES",
            Self::ContextOf => "CONTEXT_OF",
            Self::FeelsAs => "FEELS_AS",
            Self::WondersAbout => "WONDERS_ABOUT",
            Self::RemembersAs => "REMEMBERS_AS",
            Self::Implies => "IMPLIES",
            Self::Equivalent => "EQUIVALENT",
            Self::Excludes => "EXCLUDES",
            Self::Coexists => "COEXISTS",
            Self::DerivesFrom => "DERIVES_FROM",
        }
    }

    /// Nome italiano per display nella UI.
    pub fn nome(&self) -> &'static str {
        match self {
            Self::IsA => "è un",
            Self::Has => "ha",
            Self::Does => "fa",
            Self::PartOf => "parte di",
            Self::Causes => "causa",
            Self::Enables => "abilita",
            Self::Requires => "richiede",
            Self::TransformsInto => "diventa",
            Self::SimilarTo => "simile a",
            Self::OppositeOf => "opposto di",
            Self::UsedFor => "usato per",
            Self::Expresses => "esprime",
            Self::Symbolizes => "simboleggia",
            Self::ContextOf => "contesto di",
            Self::Implies => "implica",
            Self::Equivalent => "equivale",
            Self::Excludes => "esclude",
            Self::Coexists => "coesiste con",
            Self::FeelsAs => "si sente come",
            Self::WondersAbout => "si interroga su",
            Self::RemembersAs => "ricorda come",
            Self::DerivesFrom => "deriva da",
        }
    }

    /// Categoria della relazione (per raggruppamento nella UI).
    pub fn categoria(&self) -> &'static str {
        match self {
            Self::IsA | Self::Has | Self::Does | Self::PartOf => "strutturale",
            Self::Causes | Self::Enables | Self::Requires | Self::TransformsInto => "causale",
            Self::SimilarTo | Self::OppositeOf | Self::UsedFor
            | Self::Expresses | Self::Symbolizes | Self::ContextOf => "semantica",
            Self::Implies | Self::Equivalent | Self::Excludes | Self::Coexists => "logica",
            Self::FeelsAs | Self::WondersAbout | Self::RemembersAs => "fenomenologica",
            Self::DerivesFrom => "morfologica",
        }
    }

    /// Colore CSS per la UI (archi nel grafo).
    pub fn colore(&self) -> &'static str {
        match self {
            // Strutturali — verde
            Self::IsA => "#3fb950",
            Self::Has => "#58a6ff",
            Self::Does => "#f0883e",
            Self::PartOf => "#bc8cff",
            // Causali — rosso/arancio
            Self::Causes => "#f85149",
            Self::Enables => "#da3633",
            Self::Requires => "#ff7b72",
            Self::TransformsInto => "#ffa657",
            // Semantiche — azzurro/grigio
            Self::SimilarTo => "#79c0ff",
            Self::OppositeOf => "#8b949e",
            Self::UsedFor => "#7ee787",
            Self::Expresses => "#d2a8ff",
            Self::Symbolizes => "#f778ba",
            Self::ContextOf => "#a5d6ff",
            // Logiche — giallo/oro
            Self::Implies => "#e3b341",
            Self::Equivalent => "#d29922",
            Self::Excludes => "#f85149",
            Self::Coexists => "#56d364",
            // Fenomenologiche — viola/rosa profondo
            Self::FeelsAs => "#d2a8ff",
            Self::WondersAbout => "#bc8cff",
            Self::RemembersAs => "#f778ba",
            // Morfologiche — turchese
            Self::DerivesFrom => "#39c5cf",
        }
    }

    /// Forza di boost nel campo topologico per questa relazione.
    /// IS_A è la più forte (definisce cosa è una cosa).
    /// OPPOSITE_OF è la più debole (crea contrasto, non risonanza).
    pub fn field_boost_strength(&self) -> f32 {
        match self {
            // Strutturali
            Self::IsA => 0.18,
            Self::Has => 0.14,
            Self::Does => 0.14,
            Self::PartOf => 0.12,
            // Causali
            Self::Causes => 0.12,
            Self::Enables => 0.11,
            Self::Requires => 0.10,
            Self::TransformsInto => 0.12,
            // Semantiche
            Self::SimilarTo => 0.16,
            Self::UsedFor => 0.10,
            Self::Expresses => 0.13,
            Self::Symbolizes => 0.11,
            Self::ContextOf => 0.09,
            Self::OppositeOf => 0.06,
            // Logiche
            Self::Implies => 0.14,
            Self::Equivalent => 0.17,
            Self::Excludes => 0.05,
            Self::Coexists => 0.12,
            // Fenomenologiche
            Self::FeelsAs => 0.20,
            Self::WondersAbout => 0.15,
            Self::RemembersAs => 0.18,
            // Morfologiche — lega un lessema alla sua famiglia (moderato)
            Self::DerivesFrom => 0.14,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EdgeSource — origine della relazione
// ═══════════════════════════════════════════════════════════════════════════

/// Da dove viene questa relazione.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EdgeSource {
    #[default]
    /// Estratto da Wikidata (entità/categorie italiane)
    Wikidata,
    /// Da WordNet italiano (sinonimi, iperonimi, antonimi)
    Wordnet,
    /// Ontologia curata a mano (core italiana)
    Curated,
    /// Insegnata dall'utente con `:know`
    UserTaught,
    /// Derivata per inferenza transitiva
    Inferred,
    /// Contributo dalla sessione community
    Community,
    /// Curata da un agente LLM (es. verdetti Qwen applicati via `apply_verdicts.py`).
    /// Semanticamente equivalente a `Curated`, ma traccia che la curatela è
    /// passata per un agente.
    AgentCurated,
    /// Rete di sicurezza per la deserializzazione: qualunque provenance
    /// sconosciuta degrada qui invece di far fallire l'intero parsing del KG.
    /// Un'etichetta di metadati non deve poter spegnere il pensiero — un singolo
    /// token ignoto non azzera 83K archi. Vedi il fix dello Strato 1.
    #[serde(other)]
    Unknown,
}

/// Default serde per `TypedEdge::source` quando l'edge omette il campo: `Unknown`
/// (onesto: "provenance non dichiarata"), NON `EdgeSource::default()` (= Wikidata).
fn default_edge_source() -> EdgeSource {
    EdgeSource::Unknown
}

// ═══════════════════════════════════════════════════════════════════════════
// TypedEdge — un arco logico tra due concetti
// ═══════════════════════════════════════════════════════════════════════════

/// Un arco logico tipato nel Knowledge Graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedEdge {
    /// Il soggetto (da chi parte la relazione)
    pub subject: String,
    /// Il tipo di relazione
    pub relation: RelationType,
    /// L'oggetto (dove arriva la relazione)
    pub object: String,
    /// Grado di certezza [0.0, 1.0]
    pub confidence: f32,
    /// Origine della relazione. `#[serde(default)]`: se un edge curato la omette,
    /// degrada a `Unknown` invece di far fallire il parsing dell'INTERO KG — un
    /// campo di metadati mancante non deve spegnere 85K archi (stesso principio
    /// del fix Strato 1 sul valore-provenance ignoto).
    #[serde(default = "default_edge_source")]
    pub source: EdgeSource,
    /// Tramite/mezzo attraverso cui avviene la relazione (opzionale).
    /// Es: "ghiaccio DIVENTA acqua VIA calore", "fumo CAUSA cancro VIA infiammazione"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

impl TypedEdge {
    pub fn new(subject: &str, relation: RelationType, object: &str) -> Self {
        Self {
            subject: subject.to_lowercase(),
            relation,
            object: object.to_lowercase(),
            confidence: 1.0,
            source: EdgeSource::Curated,
            via: None,
        }
    }

    pub fn with_confidence(mut self, c: f32) -> Self {
        self.confidence = c;
        self
    }

    pub fn with_source(mut self, s: EdgeSource) -> Self {
        self.source = s;
        self
    }

    pub fn with_via(mut self, via: Option<String>) -> Self {
        self.via = via.map(|v| v.to_lowercase());
        self
    }

    /// Parsa una riga TSV: "soggetto\tRELAZIONE\toggetto[\tconfidenza[\tvia]]"
    pub fn from_tsv_line(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 { return None; }
        let subject = parts[0].trim().to_lowercase();
        let rel_str = parts[1].trim();
        let object = parts[2].trim().to_lowercase();
        if subject.is_empty() || object.is_empty() { return None; }
        let relation = RelationType::from_str(rel_str)?;
        let confidence = parts.get(3)
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(1.0);
        let via = parts.get(4)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase());
        Some(Self {
            subject,
            relation,
            object,
            confidence,
            source: EdgeSource::Curated,
            via,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_from_str() {
        assert_eq!(RelationType::from_str("IS_A"), Some(RelationType::IsA));
        assert_eq!(RelationType::from_str("isa"), Some(RelationType::IsA));
        assert_eq!(RelationType::from_str("CAUSES"), Some(RelationType::Causes));
        assert_eq!(RelationType::from_str("sconosciuto"), None);
    }

    #[test]
    fn derives_from_carica_e_round_trip() {
        // Phase 86: gli archi DerivesFrom curati dall'agente DEVONO caricare —
        // niente drop silenzioso (cfr. regressione AgentCurated dello Strato 1).
        // (1) from_str italiano/inglese
        assert_eq!(RelationType::from_str("DERIVES_FROM"), Some(RelationType::DerivesFrom));
        assert_eq!(RelationType::from_str("DERIVA_DA"), Some(RelationType::DerivesFrom));
        // (2) as_str round-trip
        assert_eq!(RelationType::from_str(RelationType::DerivesFrom.as_str()), Some(RelationType::DerivesFrom));
        // (3) serde (il KG JSON usa i nomi-variant: "relation": "DerivesFrom")
        let edge: TypedEdge = serde_json::from_str(
            r#"{"subject":"pulizia","relation":"DerivesFrom","object":"pulire","confidence":0.9,"source":"AgentCurated","via":"nominalizzazione"}"#
        ).expect("un arco DerivesFrom deve deserializzare");
        assert_eq!(edge.relation, RelationType::DerivesFrom);
        assert_eq!(edge.via.as_deref(), Some("nominalizzazione"));
    }

    #[test]
    fn edge_senza_source_non_rompe_il_parsing() {
        // Robustezza: un edge che OMETTE `source` deve caricare (source=Unknown),
        // non far fallire l'intero KG (principio Strato 1). Così le modifiche
        // dell'agente caricano anche se dimentica la provenance.
        let edge: TypedEdge = serde_json::from_str(
            r#"{"subject":"pulizia","relation":"DerivesFrom","object":"pulire","confidence":0.9,"via":"nominalizzazione"}"#
        ).expect("un edge senza source deve comunque deserializzare");
        assert_eq!(edge.source, EdgeSource::Unknown);
        assert_eq!(edge.relation, RelationType::DerivesFrom);
    }

    #[test]
    fn test_relation_from_str_italiano() {
        assert_eq!(RelationType::from_str("È_UN"), Some(RelationType::IsA));
        assert_eq!(RelationType::from_str("CAUSA"), Some(RelationType::Causes));
        assert_eq!(RelationType::from_str("ABILITA"), Some(RelationType::Enables));
        assert_eq!(RelationType::from_str("IMPLICA"), Some(RelationType::Implies));
        assert_eq!(RelationType::from_str("ESCLUDE"), Some(RelationType::Excludes));
        assert_eq!(RelationType::from_str("COESISTE"), Some(RelationType::Coexists));
        assert_eq!(RelationType::from_str("DIVENTA"), Some(RelationType::TransformsInto));
    }

    #[test]
    fn test_nome_italiano() {
        assert_eq!(RelationType::IsA.nome(), "è un");
        assert_eq!(RelationType::Causes.nome(), "causa");
        assert_eq!(RelationType::Implies.nome(), "implica");
        assert_eq!(RelationType::Coexists.nome(), "coesiste con");
    }

    #[test]
    fn test_all_relations_count() {
        assert_eq!(RelationType::ALL.len(), 21);
    }

    #[test]
    fn test_tsv_parse() {
        let edge = TypedEdge::from_tsv_line("cane\tIS_A\tanimale\t1.0").unwrap();
        assert_eq!(edge.subject, "cane");
        assert_eq!(edge.relation, RelationType::IsA);
        assert_eq!(edge.object, "animale");
        assert_eq!(edge.confidence, 1.0);
    }

    #[test]
    fn test_tsv_parse_italiano() {
        let edge = TypedEdge::from_tsv_line("coraggio\tABILITA\trischiare\t0.8").unwrap();
        assert_eq!(edge.relation, RelationType::Enables);
        assert_eq!(edge.confidence, 0.8);
    }

    #[test]
    fn test_tsv_parse_no_confidence() {
        let edge = TypedEdge::from_tsv_line("sole\tDOES\tbrillare").unwrap();
        assert_eq!(edge.subject, "sole");
        assert_eq!(edge.relation, RelationType::Does);
        assert_eq!(edge.confidence, 1.0);
    }

    #[test]
    fn test_tsv_invalid() {
        assert!(TypedEdge::from_tsv_line("solo_campo").is_none());
        assert!(TypedEdge::from_tsv_line("a\tREL_SCONOSCIUTA\tb").is_none());
    }
}
