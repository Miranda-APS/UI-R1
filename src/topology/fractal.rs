/// Frattali — Attrattori nel campo 8D che generano dimensioni proprie.
///
/// Un frattale e una regione stabile nello spazio 8D. Ha:
/// - Una firma: alcune dimensioni sono fisse, altre sono libere
/// - Dimensioni generate: nuove coordinate che emergono dalle co-variazioni
///   delle dimensioni libere (come HSL emerge da RGB)
/// - Sotto-frattali: specializzazioni ricorsive

use std::collections::HashMap;
use crate::topology::primitive::{PrimitiveCore, Dim};

/// Identificatore univoco di un frattale.
pub type FractalId = u32;

/// Una dimensione nella firma 8D puo essere fissa o libera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DimConstraint {
    /// Valore fissato: questa dimensione e vincolata nel frattale.
    Fixed(f64),
    /// Libera: puo variare. Le co-variazioni delle dimensioni libere
    /// generano le dimensioni emergenti.
    Free,
}

/// Una dimensione emergente — generata dal frattale, non presente nelle 8D.
///
/// Ogni frattale ha dimensioni libere (non vincolate). Le parole che abitano
/// quel frattale si distribuiscono lungo queste dimensioni libere.
/// Le dimensioni emergenti catturano gli ASSI DI VARIAZIONE principali
/// tra le sorgenti specificate — sono il sistema di coordinate LOCALE
/// del frattale, calibrato dalla popolazione di parole.
#[derive(Debug, Clone)]
pub struct EmergentDimension {
    /// Nome della dimensione (es. "hue", "durata", "reciprocita")
    pub name: String,
    /// Da quali dimensioni libere 8D emerge (le sue "sorgenti")
    pub source_dims: Vec<Dim>,
    /// Direzione dell'asse nello spazio delle source_dims.
    /// Vettore unitario — i coefficienti indicano quanto ogni source_dim
    /// contribuisce a questa dimensione emergente.
    /// Calibrato dalla distribuzione delle parole nel frattale.
    pub direction: Vec<f64>,
    /// Media della proiezione (centro dell'asse nella popolazione)
    pub mean: f64,
    /// Deviazione standard (ampiezza della distribuzione)
    pub std_dev: f64,
    /// Range osservato [min, max] nella popolazione
    pub range: (f64, f64),
    /// Varianza spiegata: quanta dell'informazione totale cattura [0, 1]
    pub explained_variance: f64,
    /// Quante parole hanno contribuito all'ultima calibrazione
    pub calibration_population: usize,
}

impl EmergentDimension {
    pub fn new(name: &str, sources: Vec<Dim>) -> Self {
        let n = sources.len();
        // Direzione iniziale: uniforme (ogni sorgente contribuisce uguale)
        let uniform = 1.0 / (n as f64).sqrt();
        Self {
            name: name.to_string(),
            direction: vec![uniform; n],
            source_dims: sources,
            mean: 0.5,
            std_dev: 0.0,
            range: (0.0, 1.0),
            explained_variance: 0.0,
            calibration_population: 0,
        }
    }

    /// Proietta un punto 8D su questa dimensione emergente.
    /// Restituisce un valore normalizzato: 0 = media, ±1 = una deviazione standard.
    /// Senza calibrazione (std_dev == 0), restituisce la proiezione grezza centrata.
    pub fn project(&self, point: &PrimitiveCore) -> f64 {
        let raw: f64 = self.source_dims.iter()
            .zip(self.direction.iter())
            .map(|(dim, &coeff)| point.get(*dim) * coeff)
            .sum();

        if self.std_dev > 0.01 {
            (raw - self.mean) / self.std_dev
        } else {
            raw - self.mean
        }
    }

    /// Proiezione grezza (non normalizzata) — utile per confronti diretti.
    pub fn project_raw(&self, point: &PrimitiveCore) -> f64 {
        self.source_dims.iter()
            .zip(self.direction.iter())
            .map(|(dim, &coeff)| point.get(*dim) * coeff)
            .sum()
    }

    /// Calibra la dimensione emergente dalla popolazione di parole.
    /// Riceve le firme 8D di tutte le parole che abitano il frattale.
    /// Calcola la direzione di massima varianza nello spazio delle source_dims.
    pub fn calibrate(&mut self, signatures: &[&PrimitiveCore]) {
        let n = signatures.len();
        if n < 3 || self.source_dims.is_empty() {
            return;
        }
        self.calibration_population = n;

        let dim_count = self.source_dims.len();

        // Estrai valori delle source_dims per ogni parola
        let values: Vec<Vec<f64>> = signatures.iter()
            .map(|sig| {
                self.source_dims.iter()
                    .map(|d| sig.get(*d))
                    .collect()
            })
            .collect();

        // Calcola media per ogni source_dim
        let mut means = vec![0.0_f64; dim_count];
        for row in &values {
            for (j, val) in row.iter().enumerate() {
                means[j] += val;
            }
        }
        for m in &mut means {
            *m /= n as f64;
        }

        // Calcola matrice di covarianza (dim_count x dim_count)
        let mut cov = vec![vec![0.0_f64; dim_count]; dim_count];
        for row in &values {
            for j in 0..dim_count {
                for k in 0..dim_count {
                    cov[j][k] += (row[j] - means[j]) * (row[k] - means[k]);
                }
            }
        }
        for j in 0..dim_count {
            for k in 0..dim_count {
                cov[j][k] /= (n - 1) as f64;
            }
        }

        // Trova autovettore principale con power iteration
        // (sufficiente per matrici 2x2 o 3x3 delle source_dims)
        let mut eigenvec = vec![1.0 / (dim_count as f64).sqrt(); dim_count];
        let mut eigenval = 0.0_f64;

        for _ in 0..50 {
            // Moltiplica cov * eigenvec
            let mut new_vec = vec![0.0; dim_count];
            for j in 0..dim_count {
                for k in 0..dim_count {
                    new_vec[j] += cov[j][k] * eigenvec[k];
                }
            }

            // Norma
            let norm: f64 = new_vec.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < 1e-10 {
                break;
            }
            eigenval = norm;

            // Normalizza
            for v in &mut new_vec {
                *v /= norm;
            }

            // Convergenza?
            let diff: f64 = eigenvec.iter()
                .zip(new_vec.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();

            eigenvec = new_vec;
            if diff < 1e-8 {
                break;
            }
        }

        self.direction = eigenvec;

        // Calcola proiezioni di tutte le parole per mean/std/range
        let projections: Vec<f64> = values.iter()
            .map(|row| {
                row.iter()
                    .zip(self.direction.iter())
                    .map(|(v, d)| v * d)
                    .sum::<f64>()
            })
            .collect();

        let proj_mean = projections.iter().sum::<f64>() / n as f64;
        let proj_var = projections.iter()
            .map(|p| (p - proj_mean).powi(2))
            .sum::<f64>() / (n - 1) as f64;
        let proj_std = proj_var.sqrt();

        self.mean = proj_mean;
        self.std_dev = proj_std;

        let min = projections.iter().cloned().fold(f64::MAX, f64::min);
        let max = projections.iter().cloned().fold(f64::MIN, f64::max);
        self.range = (min, max);

        // Varianza spiegata: eigenvalue / varianza totale
        let total_var: f64 = (0..dim_count).map(|j| cov[j][j]).sum();
        self.explained_variance = if total_var > 1e-10 {
            (eigenval / total_var).min(1.0)
        } else {
            0.0
        };
    }

    /// La dimensione e stata calibrata con dati reali?
    pub fn is_calibrated(&self) -> bool {
        self.calibration_population >= 3 && self.std_dev > 0.001
    }

    // Backward compatibility
    pub fn with_value(mut self, _value: f64) -> Self {
        // Il valore singolo non ha piu senso — la dimensione ha una distribuzione.
        // Mantenuto per non rompere codice esistente.
        self
    }
}

/// Un frattale: attrattore nel campo 8D che genera il proprio mondo dimensionale.
#[derive(Debug, Clone)]
pub struct Fractal {
    /// Identificatore univoco
    pub id: FractalId,
    /// Nome leggibile
    pub name: String,
    /// Firma 8D: quali dimensioni sono fisse (definiscono il frattale)
    /// e quali sono libere (generano sotto-frattali)
    pub signature: [DimConstraint; 8],
    /// Dimensioni emergenti — generate dal frattale, nuove
    pub emergent_dimensions: Vec<EmergentDimension>,
    /// Sotto-frattali (figli)
    pub children: Vec<FractalId>,
    /// Frattale padre (None per i frattali di primo livello)
    pub parent: Option<FractalId>,
    /// Persistenza: quanto e stabile questo frattale [0.0, 1.0]
    pub persistence: f64,
    /// Plasticita: quanto puo essere modificato [0.0, 1.0]
    pub plasticity: f64,
    /// Contatore attivazioni: quante volte e stato attraversato
    pub activation_count: u64,
}

impl Fractal {
    /// Crea un nuovo frattale con firma data.
    pub fn new(id: FractalId, name: &str, signature: [DimConstraint; 8]) -> Self {
        Self {
            id,
            name: name.to_string(),
            signature,
            emergent_dimensions: Vec::new(),
            children: Vec::new(),
            parent: None,
            persistence: 0.5,
            plasticity: 0.8,
            activation_count: 0,
        }
    }

    /// Aggiunge una dimensione emergente.
    pub fn add_dimension(&mut self, dim: EmergentDimension) {
        self.emergent_dimensions.push(dim);
    }

    /// Restituisce le dimensioni fisse della firma.
    pub fn fixed_dims(&self) -> Vec<(Dim, f64)> {
        Dim::ALL.iter()
            .filter_map(|d| {
                if let DimConstraint::Fixed(v) = self.signature[d.index()] {
                    Some((*d, v))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Restituisce le dimensioni libere della firma.
    pub fn free_dims(&self) -> Vec<Dim> {
        Dim::ALL.iter()
            .filter(|d| matches!(self.signature[d.index()], DimConstraint::Free))
            .copied()
            .collect()
    }

    /// Calcola il core 8D "centro" di questo frattale.
    /// Le dimensioni fisse prendono il valore vincolato.
    /// Le dimensioni libere restano a 0.5 (neutro).
    pub fn center(&self) -> PrimitiveCore {
        let mut values = [0.5; 8];
        for d in &Dim::ALL {
            if let DimConstraint::Fixed(v) = self.signature[d.index()] {
                values[d.index()] = v;
            }
        }
        PrimitiveCore::new(values)
    }

    /// Quanto un punto 8D e vicino a questo frattale?
    /// Misura solo la distanza sulle dimensioni FISSE
    /// (le libere non contano — il frattale le accetta tutte).
    pub fn affinity(&self, point: &PrimitiveCore) -> f64 {
        let fixed = self.fixed_dims();
        if fixed.is_empty() {
            return 1.0; // Nessun vincolo → affinita massima
        }
        let sum_sq: f64 = fixed.iter()
            .map(|(dim, val)| {
                let diff = point.get(*dim) - val;
                diff * diff
            })
            .sum();
        let max_dist = (fixed.len() as f64).sqrt(); // distanza massima possibile
        1.0 - (sum_sq.sqrt() / max_dist).min(1.0)
    }

    /// Registra un'attivazione (il frattale e stato attraversato).
    pub fn activate(&mut self) {
        self.activation_count += 1;
        // Ogni attivazione riduce leggermente la plasticita
        // e aumenta la persistenza (cristallizzazione)
        self.plasticity = (self.plasticity * 0.998).max(0.05);
        self.persistence = (self.persistence + 0.002).min(1.0);
    }

    /// Numero totale di dimensioni (8D fisse usate + emergenti).
    pub fn total_dimensions(&self) -> usize {
        self.fixed_dims().len() + self.emergent_dimensions.len()
    }
}

/// Registro di tutti i frattali nel sistema.
#[derive(Debug, Clone)]
pub struct FractalRegistry {
    fractals: HashMap<FractalId, Fractal>,
    next_id: FractalId,
}

impl FractalRegistry {
    pub fn new() -> Self {
        Self {
            fractals: HashMap::new(),
            next_id: 0,
        }
    }

    /// Registra un frattale fondamentale (primo livello).
    pub fn register(&mut self, name: &str, signature: [DimConstraint; 8]) -> FractalId {
        let id = self.next_id;
        self.next_id += 1;
        let fractal = Fractal::new(id, name, signature);
        self.fractals.insert(id, fractal);
        id
    }

    /// Registra un sotto-frattale (figlio di un frattale esistente).
    /// Il sotto-frattale eredita le dimensioni fisse del padre
    /// e puo vincolare ulteriori dimensioni libere.
    pub fn register_child(
        &mut self,
        parent_id: FractalId,
        name: &str,
        additional_constraints: &[(Dim, f64)],
    ) -> Option<FractalId> {
        // Copia la firma del padre
        let parent_sig = self.fractals.get(&parent_id)?.signature;

        let mut child_sig = parent_sig;
        // Applica vincoli aggiuntivi (vincola dimensioni che nel padre erano libere)
        for (dim, val) in additional_constraints {
            child_sig[dim.index()] = DimConstraint::Fixed(*val);
        }

        let id = self.next_id;
        self.next_id += 1;

        let mut child = Fractal::new(id, name, child_sig);
        child.parent = Some(parent_id);

        // Aggiungi come figlio del padre
        if let Some(parent) = self.fractals.get_mut(&parent_id) {
            parent.children.push(id);
        }

        self.fractals.insert(id, child);
        Some(id)
    }

    /// Accesso a un frattale.
    pub fn get(&self, id: FractalId) -> Option<&Fractal> {
        self.fractals.get(&id)
    }

    /// Accesso mutabile a un frattale.
    pub fn get_mut(&mut self, id: FractalId) -> Option<&mut Fractal> {
        self.fractals.get_mut(&id)
    }

    /// Tutti i frattali di primo livello (senza padre).
    pub fn roots(&self) -> Vec<FractalId> {
        self.fractals.values()
            .filter(|f| f.parent.is_none())
            .map(|f| f.id)
            .collect()
    }

    /// Tutti gli id registrati.
    pub fn all_ids(&self) -> Vec<FractalId> {
        self.fractals.keys().copied().collect()
    }

    /// Numero totale di frattali.
    pub fn count(&self) -> usize {
        self.fractals.len()
    }

    /// Iteratore su tutti i frattali.
    pub fn iter(&self) -> impl Iterator<Item = (&FractalId, &Fractal)> {
        self.fractals.iter()
    }

    /// Trova il frattale piu affine a un punto 8D.
    pub fn nearest(&self, point: &PrimitiveCore) -> Option<FractalId> {
        self.fractals.values()
            .max_by(|a, b| {
                a.affinity(point)
                    .partial_cmp(&b.affinity(point))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|f| f.id)
    }

    /// Calcola le affinita di un punto 8D con TUTTI i frattali simultaneamente.
    /// Ogni frattale ha un'affinita geometrica — niente medie, niente assegnazioni.
    /// Una parola "vive" in tutti i frattali con intensita diversa,
    /// come un punto nello spazio colora tutti i campi gravitazionali intorno a se.
    pub fn all_affinities(&self, point: &PrimitiveCore) -> Vec<(FractalId, f64)> {
        self.fractals.values()
            .map(|f| (f.id, f.affinity(point)))
            .collect()
    }

    /// Calibra le dimensioni emergenti di TUTTI i frattali dalla popolazione
    /// di parole nel lessico. Ogni parola contribuisce al frattale con cui
    /// ha la massima affinita (o affinita > soglia).
    ///
    /// Questa funzione rende le dimensioni emergenti VIVE:
    /// da etichette statiche diventano assi calibrati dalla distribuzione reale.
    pub fn calibrate_all_emergent_dimensions(&mut self, word_signatures: &[(FractalId, PrimitiveCore)]) {
        // Raggruppa firme per frattale
        let mut per_fractal: HashMap<FractalId, Vec<PrimitiveCore>> = HashMap::new();
        for (fid, sig) in word_signatures {
            per_fractal.entry(*fid).or_default().push(*sig);
        }

        // Calibra ogni frattale
        for (fid, sigs) in &per_fractal {
            if let Some(fractal) = self.fractals.get_mut(fid) {
                let sig_refs: Vec<&PrimitiveCore> = sigs.iter().collect();
                for dim in &mut fractal.emergent_dimensions {
                    dim.calibrate(&sig_refs);
                }
            }
        }
    }

    /// Proietta un punto 8D sulle dimensioni emergenti di un frattale.
    /// Restituisce le coordinate emergenti normalizzate.
    pub fn project_emergent(&self, fractal_id: FractalId, point: &PrimitiveCore) -> Vec<(String, f64)> {
        if let Some(fractal) = self.fractals.get(&fractal_id) {
            fractal.emergent_dimensions.iter()
                .filter(|d| d.is_calibrated())
                .map(|d| (d.name.clone(), d.project(point)))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Distanza emergente tra due punti all'interno di un frattale.
    /// Usa solo le dimensioni emergenti calibrate.
    /// Restituisce 0.0 se il frattale non ha emergenti calibrate.
    pub fn emergent_distance(&self, fractal_id: FractalId, a: &PrimitiveCore, b: &PrimitiveCore) -> f64 {
        if let Some(fractal) = self.fractals.get(&fractal_id) {
            let calibrated: Vec<&EmergentDimension> = fractal.emergent_dimensions.iter()
                .filter(|d| d.is_calibrated())
                .collect();

            if calibrated.is_empty() {
                return 0.0;
            }

            let sum_sq: f64 = calibrated.iter()
                .map(|d| {
                    let pa = d.project(a);
                    let pb = d.project(b);
                    (pa - pb).powi(2)
                })
                .sum();

            (sum_sq / calibrated.len() as f64).sqrt()
        } else {
            0.0
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// I 64 Esagrammi — Operatori Cognitivi Universali
// ═══════════════════════════════════════════════════════════════

use DimConstraint::{Fixed, Free};

/// Gli 8 trigrammi dell'I Ching — operatori primitivi.
/// Ogni trigramma controlla una dimensione primaria con un valore fisso.
///
/// Codifica Yang: ☰=0.90 (tutto Yang), ⅔Yang=0.70, ⅓Yang=0.30, ☷=0.10 (tutto Yin)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigram {
    Cielo,      // ☰ 111 → Agency = 0.90     (forza che inizia)
    Terra,      // ☷ 000 → Permanenza = 0.10  (campo che sostiene)
    Tuono,      // ☳ 001 → Intensita = 0.30   (impulso che scuote)
    Acqua,      // ☵ 010 → Tempo = 0.30       (flusso che scava)
    Montagna,   // ☶ 100 → Confine = 0.30     (forma che arresta)
    Vento,      // ☴ 110 → Complessita = 0.70 (penetrazione diffusa)
    Fuoco,      // ☲ 101 → Definizione = 0.70 (luce che distingue)
    Lago,       // ☱ 011 → Valenza = 0.70     (scambio che apre)
}

impl Trigram {
    /// I 8 trigrammi in ordine canonico (indice 0-7).
    pub const ALL: [Trigram; 8] = [
        Trigram::Cielo, Trigram::Terra, Trigram::Tuono, Trigram::Acqua,
        Trigram::Montagna, Trigram::Vento, Trigram::Fuoco, Trigram::Lago,
    ];

    /// La dimensione primaria controllata da questo trigramma.
    pub fn dim(self) -> Dim {
        match self {
            Trigram::Cielo    => Dim::Agency,
            Trigram::Terra    => Dim::Permanenza,
            Trigram::Tuono    => Dim::Intensita,
            Trigram::Acqua    => Dim::Tempo,
            Trigram::Montagna => Dim::Confine,
            Trigram::Vento    => Dim::Complessita,
            Trigram::Fuoco    => Dim::Definizione,
            Trigram::Lago     => Dim::Valenza,
        }
    }

    /// Il valore fisso nella dimensione primaria.
    /// Codifica il contenuto Yang del trigramma: ☰=0.90, ⅔=0.70, ⅓=0.30, ☷=0.10
    pub fn value(self) -> f64 {
        match self {
            Trigram::Cielo    => 0.90, // 111 tutto Yang
            Trigram::Vento    => 0.70, // 110 ⅔ Yang
            Trigram::Fuoco    => 0.70, // 101 ⅔ Yang
            Trigram::Lago     => 0.70, // 011 ⅔ Yang
            Trigram::Tuono    => 0.30, // 001 ⅓ Yang
            Trigram::Acqua    => 0.30, // 010 ⅓ Yang
            Trigram::Montagna => 0.30, // 100 ⅓ Yang
            Trigram::Terra    => 0.10, // 000 tutto Yin
        }
    }

    /// Indice 0-7. ID esagramma = lower.index()*8 + upper.index()
    pub fn index(self) -> usize {
        match self {
            Trigram::Cielo    => 0,
            Trigram::Terra    => 1,
            Trigram::Tuono    => 2,
            Trigram::Acqua    => 3,
            Trigram::Montagna => 4,
            Trigram::Vento    => 5,
            Trigram::Fuoco    => 6,
            Trigram::Lago     => 7,
        }
    }

    /// Simbolo Unicode del trigramma.
    pub fn symbol(self) -> &'static str {
        match self {
            Trigram::Cielo    => "☰",
            Trigram::Terra    => "☷",
            Trigram::Tuono    => "☳",
            Trigram::Acqua    => "☵",
            Trigram::Montagna => "☶",
            Trigram::Vento    => "☴",
            Trigram::Fuoco    => "☲",
            Trigram::Lago     => "☱",
        }
    }
}

/// I 64 esagrammi: (trigramma_inferiore, trigramma_superiore, nome).
/// ID = lower.index()*8 + upper.index() → 0..63
/// Trigramma inferiore = processo interno; superiore = contesto esterno.
/// Il nome è la generalizzazione più assoluta di quella combinazione cognitiva.
pub static HEXAGRAMS: [(Trigram, Trigram, &str); 64] = {
    use Trigram::*;
    [
        // ☰ Cielo (Agency) come interno
        (Cielo,    Cielo,    "POTERE"),
        (Cielo,    Terra,    "CREAZIONE"),
        (Cielo,    Tuono,    "ENERGIA"),
        (Cielo,    Acqua,    "INTENZIONE"),
        (Cielo,    Montagna, "DETERMINAZIONE"),
        (Cielo,    Vento,    "INFLUENZA"),
        (Cielo,    Fuoco,    "VISIONE"),
        (Cielo,    Lago,     "DONO"),
        // ☷ Terra (Permanenza) come interno
        (Terra,    Cielo,    "VITA"),
        (Terra,    Terra,    "MATERIA"),
        (Terra,    Tuono,    "SENSAZIONE"),
        (Terra,    Acqua,    "MUTAMENTO"),
        (Terra,    Montagna, "STRUTTURA"),
        (Terra,    Vento,    "MONDO"),
        (Terra,    Fuoco,    "REALTÀ"),
        (Terra,    Lago,     "NUTRIMENTO"),
        // ☳ Tuono (Intensita) come interno
        (Tuono,    Cielo,    "INIZIATIVA"),
        (Tuono,    Terra,    "RADICAMENTO"),
        (Tuono,    Tuono,    "ARDORE"),
        (Tuono,    Acqua,    "RITMO"),
        (Tuono,    Montagna, "IMPATTO"),
        (Tuono,    Vento,    "RISONANZA"),
        (Tuono,    Fuoco,    "EVIDENZA"),
        (Tuono,    Lago,     "PASSIONE"),
        // ☵ Acqua (Tempo) come interno
        (Acqua,    Cielo,    "DESTINO"),
        (Acqua,    Terra,    "MEMORIA"),
        (Acqua,    Tuono,    "CRISI"),
        (Acqua,    Acqua,    "DIVENIRE"),
        (Acqua,    Montagna, "DURATA"),
        (Acqua,    Vento,    "STORIA"),
        (Acqua,    Fuoco,    "COMPRENSIONE"),
        (Acqua,    Lago,     "ESPERIENZA"),
        // ☶ Montagna (Confine) come interno
        (Montagna, Cielo,    "IDENTITÀ"),
        (Montagna, Terra,    "CORPO"),
        (Montagna, Tuono,    "RESISTENZA"),
        (Montagna, Acqua,    "EVOLUZIONE"),
        (Montagna, Montagna, "SPAZIO"),
        (Montagna, Vento,    "ECOSISTEMA"),
        (Montagna, Fuoco,    "SIMBOLO"),
        (Montagna, Lago,     "SOGLIA"),
        // ☴ Vento (Complessita) come interno
        (Vento,    Cielo,    "STRATEGIA"),
        (Vento,    Terra,    "CULTURA"),
        (Vento,    Tuono,    "CAOS"),
        (Vento,    Acqua,    "PROCESSO"),
        (Vento,    Montagna, "SISTEMA"),
        (Vento,    Vento,    "INTRECCIO"),
        (Vento,    Fuoco,    "LINGUAGGIO"),
        (Vento,    Lago,     "COMUNICAZIONE"),
        // ☲ Fuoco (Definizione) come interno
        (Fuoco,    Cielo,    "COSCIENZA"),
        (Fuoco,    Terra,    "CONOSCENZA"),
        (Fuoco,    Tuono,    "PERCEZIONE"),
        (Fuoco,    Acqua,    "INTUIZIONE"),
        (Fuoco,    Montagna, "IDEA"),
        (Fuoco,    Vento,    "PENSIERO"),
        (Fuoco,    Fuoco,    "VERITÀ"),
        (Fuoco,    Lago,     "ESPRESSIONE"),
        // ☱ Lago (Valenza) come interno
        (Lago,     Cielo,    "DESIDERIO"),
        (Lago,     Terra,    "AMORE"),
        (Lago,     Tuono,    "EMOZIONE"),
        (Lago,     Acqua,    "EMPATIA"),
        (Lago,     Montagna, "ACCORDO"),
        (Lago,     Vento,    "SOCIETÀ"),
        (Lago,     Fuoco,    "ETICA"),
        (Lago,     Lago,     "ARMONIA"),
    ]
};

/// Costruisce la firma 8D per un esagramma dato i suoi due trigrammi.
/// Dimensioni fisse: quella del trigramma inferiore e quella del superiore.
/// Se i due trigrammi controllano la stessa dimensione (esagrammi puri),
/// la dimensione viene fissata una volta sola.
/// Le restanti 6 (o 7) dimensioni sono libere.
fn hexagram_signature(lower: Trigram, upper: Trigram) -> [DimConstraint; 8] {
    let mut sig = [Free; 8];
    sig[lower.dim().index()] = Fixed(lower.value());
    sig[upper.dim().index()] = Fixed(upper.value());
    sig
}

/// Crea il registro con i 64 frattali-esagramma.
///
/// Ogni esagramma è un operatore cognitivo con:
/// - 2 dimensioni fisse (trigramma inferiore + superiore)
/// - 6 dimensioni libere (si calibrano con l'esperienza)
///
/// ID 0..63: lower.index()*8 + upper.index()
pub fn bootstrap_fractals() -> FractalRegistry {
    let mut reg = FractalRegistry::new();
    for (lower, upper, name) in &HEXAGRAMS {
        reg.register(name, hexagram_signature(*lower, *upper));
    }
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    // Costanti ID per i test (lower.index()*8 + upper.index())
    const POTERE: FractalId = 0;      // ☰☰ Agency=0.90
    const MATERIA: FractalId = 9;     // ☷☷ Permanenza=0.10
    const SPAZIO: FractalId = 36;     // ☶☶ Confine=0.30
    const IDENTITA: FractalId = 32;   // ☶☰ Confine=0.30, Agency=0.90
    const PENSIERO: FractalId = 53;   // ☲☴ Definizione=0.70, Complessita=0.70
    const ARMONIA: FractalId = 63;    // ☱☱ Valenza=0.70
    const VERITÀ: FractalId = 54;     // ☲☲ Definizione=0.70

    #[test]
    fn test_bootstrap_creates_64_hexagrams() {
        let reg = bootstrap_fractals();
        assert_eq!(reg.count(), 64, "Devono esserci esattamente 64 esagrammi");
    }

    #[test]
    fn test_hexagram_ids_sono_sequenziali() {
        let reg = bootstrap_fractals();
        // Ogni ID da 0 a 63 deve esistere
        for id in 0u32..64 {
            assert!(reg.get(id).is_some(), "ID {} non trovato", id);
        }
    }

    #[test]
    fn test_nomi_esagrammi() {
        let reg = bootstrap_fractals();
        assert_eq!(reg.get(POTERE).unwrap().name, "POTERE");
        assert_eq!(reg.get(SPAZIO).unwrap().name, "SPAZIO");
        assert_eq!(reg.get(IDENTITA).unwrap().name, "IDENTITÀ");
        assert_eq!(reg.get(PENSIERO).unwrap().name, "PENSIERO");
        assert_eq!(reg.get(ARMONIA).unwrap().name, "ARMONIA");
    }

    #[test]
    fn test_firme_esagrammi() {
        let reg = bootstrap_fractals();

        // POTERE (☰☰): Agency=0.90 fissa (esagramma puro: 1 sola dim fissa)
        let potere = reg.get(POTERE).unwrap();
        assert!(matches!(potere.signature[Dim::Agency.index()], DimConstraint::Fixed(v) if (v - 0.90).abs() < 1e-10));

        // SPAZIO (☶☶): Confine=0.30 fissa (esagramma puro)
        let spazio = reg.get(SPAZIO).unwrap();
        assert!(matches!(spazio.signature[Dim::Confine.index()], DimConstraint::Fixed(v) if (v - 0.30).abs() < 1e-10));

        // IDENTITÀ (☶☰): Confine=0.30 + Agency=0.90 fisse
        let identita = reg.get(IDENTITA).unwrap();
        assert!(matches!(identita.signature[Dim::Confine.index()], DimConstraint::Fixed(v) if (v - 0.30).abs() < 1e-10));
        assert!(matches!(identita.signature[Dim::Agency.index()], DimConstraint::Fixed(v) if (v - 0.90).abs() < 1e-10));

        // PENSIERO (☲☴): Definizione=0.70 + Complessita=0.70 fisse
        let pensiero = reg.get(PENSIERO).unwrap();
        assert!(matches!(pensiero.signature[Dim::Definizione.index()], DimConstraint::Fixed(v) if (v - 0.70).abs() < 1e-10));
        assert!(matches!(pensiero.signature[Dim::Complessita.index()], DimConstraint::Fixed(v) if (v - 0.70).abs() < 1e-10));
    }

    #[test]
    fn test_fractal_affinity_potere() {
        let reg = bootstrap_fractals();
        // Ordine I Ching: Agency=0. Punto con Agency=0.90 → affinita 1.0 con POTERE.
        let agency_point = PrimitiveCore::new([0.90, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        let potere = reg.get(POTERE).unwrap();
        let aff = potere.affinity(&agency_point);
        assert!((aff - 1.0).abs() < 1e-9, "Agency=0.90 → affinita POTERE = 1.0, got {}", aff);
    }

    #[test]
    fn test_fractal_affinity_spazio() {
        let reg = bootstrap_fractals();
        // Ordine I Ching: Confine=4. Punto con Confine=0.30 → affinita 1.0 con SPAZIO (☶☶).
        let point = PrimitiveCore::new([0.5, 0.5, 0.5, 0.5, 0.30, 0.5, 0.5, 0.5]);
        let spazio = reg.get(SPAZIO).unwrap();
        let aff = spazio.affinity(&point);
        assert!((aff - 1.0).abs() < 1e-9, "Confine=0.30 → affinita SPAZIO = 1.0, got {}", aff);
    }

    #[test]
    fn test_nearest_fractal_agency() {
        let reg = bootstrap_fractals();
        // Ordine I Ching: Agency=0. Punto con Agency=0.90 → POTERE.
        let point = PrimitiveCore::new([0.90, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        let nearest = reg.nearest(&point).unwrap();
        let name = &reg.get(nearest).unwrap().name;
        assert_eq!(name, "POTERE", "Punto Agency=0.90 vicino a POTERE, trovato: {}", name);
    }

    #[test]
    fn test_nearest_fractal_valenza() {
        let reg = bootstrap_fractals();
        // Ordine I Ching: Valenza=7. Punto con Valenza=0.70 → ARMONIA (☱☱).
        let point = PrimitiveCore::new([0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.70]);
        let nearest = reg.nearest(&point).unwrap();
        let name = &reg.get(nearest).unwrap().name;
        assert_eq!(name, "ARMONIA", "Punto Valenza=0.70 vicino a ARMONIA, trovato: {}", name);
    }

    #[test]
    fn test_tutti_radice() {
        let reg = bootstrap_fractals();
        // Tutti i 64 esagrammi sono frattali radice (nessun padre)
        let roots = reg.roots();
        assert_eq!(roots.len(), 64, "Tutti i 64 esagrammi devono essere radice");
    }

    #[test]
    fn test_emergent_calibration() {
        // Crea una dimensione emergente con source_dims [Valenza, Intensita]
        let mut dim = EmergentDimension::new("test_asse", vec![Dim::Valenza, Dim::Intensita]);
        assert!(!dim.is_calibrated(), "Non calibrata inizialmente");

        // Popolazione sintetica: 50 parole con Valenza e Intensita variabili
        let sigs: Vec<PrimitiveCore> = (0..50).map(|i| {
            let v = i as f64 / 50.0;
            let intensity = 0.3 + v * 0.4; // da 0.3 a 0.7
            let valence = 0.2 + v * 0.6;   // da 0.2 a 0.8
            PrimitiveCore::new([0.5, valence, intensity, 0.5, 0.5, 0.5, 0.5, 0.5])
        }).collect();

        let sig_refs: Vec<&PrimitiveCore> = sigs.iter().collect();
        dim.calibrate(&sig_refs);

        assert!(dim.is_calibrated(), "Deve essere calibrata dopo 50 parole");
        assert!(dim.std_dev > 0.0, "Deve avere varianza: std_dev={}", dim.std_dev);
        assert!(dim.explained_variance > 0.0, "Deve spiegare varianza: {}", dim.explained_variance);
        assert_eq!(dim.calibration_population, 50);

        // Proiezione: parola con alta Valenza e Intensita → alto sulla dimensione
        let high = PrimitiveCore::new([0.5, 0.9, 0.9, 0.5, 0.5, 0.5, 0.5, 0.5]);
        let low = PrimitiveCore::new([0.5, 0.1, 0.1, 0.5, 0.5, 0.5, 0.5, 0.5]);

        let proj_high = dim.project(&high);
        let proj_low = dim.project(&low);

        assert!(proj_high > proj_low,
            "Punto alto ({}) deve proiettare sopra punto basso ({})",
            proj_high, proj_low);
    }

    #[test]
    fn test_emergent_distance_same_fractal() {
        let mut reg = bootstrap_fractals();

        // Aggiungi una dimensione emergente a PENSIERO (id=53, Definizione+Complessita fissi)
        // Le dim libere: Confine, Valenza, Intensita, Permanenza, Agency, Tempo
        if let Some(f) = reg.get_mut(PENSIERO) {
            f.add_dimension(EmergentDimension::new("astrazione", vec![Dim::Valenza, Dim::Intensita]));
        }

        // Due punti con Definizione=0.70, Complessita=0.70 (fissi di PENSIERO) ma Valenza/Intensita diverse
        let sig_a = PrimitiveCore::new([0.5, 0.9, 0.9, 0.70, 0.70, 0.5, 0.5, 0.5]);
        let sig_b = PrimitiveCore::new([0.5, 0.1, 0.1, 0.70, 0.70, 0.5, 0.5, 0.5]);

        let population: Vec<PrimitiveCore> = (0..30).map(|i| {
            let t = i as f64 / 30.0;
            PrimitiveCore::new([0.5, t, t * 0.8, 0.70, 0.70, 0.5, 0.5, 0.3 + t * 0.4])
        }).collect();

        let word_sigs: Vec<(FractalId, PrimitiveCore)> = population.iter()
            .map(|s| (PENSIERO, *s))
            .collect();

        reg.calibrate_all_emergent_dimensions(&word_sigs);

        let dist = reg.emergent_distance(PENSIERO, &sig_a, &sig_b);
        assert!(dist > 0.1,
            "Distanza emergente tra punti opposti deve essere > 0.1: got {}", dist);
    }

    #[test]
    fn test_calibrate_emergent_aggiunge_dimensioni() {
        let mut reg = bootstrap_fractals();

        // Aggiungi una dimensione emergente a SPAZIO (id=36, Confine=0.30 fisso)
        // Dim libere: Valenza, Intensita, Definizione, Complessita, Permanenza, Agency, Tempo
        if let Some(f) = reg.get_mut(SPAZIO) {
            f.add_dimension(EmergentDimension::new("estensione", vec![Dim::Valenza, Dim::Complessita]));
        }

        let mut word_sigs = Vec::new();
        // 20 parole per SPAZIO (id=36, Confine=0.30 fisso)
        for i in 0..20 {
            let t = i as f64 / 20.0;
            word_sigs.push((SPAZIO, PrimitiveCore::new([
                0.30,          // Confine: fisso
                t * 0.8,       // Valenza: libera, varia
                0.5 + t * 0.3, // Intensita: libera
                0.5,           // Definizione: libera
                0.3 + t * 0.4, // Complessita: libera, varia
                0.5,           // Permanenza: libera
                0.5,           // Agency: libera
                t * 0.5,       // Tempo: libero
            ])));
        }
        // 20 parole per IDENTITÀ (id=32, Confine=0.30 + Agency=0.90 fissi)
        for i in 0..20 {
            let t = i as f64 / 20.0;
            word_sigs.push((IDENTITA, PrimitiveCore::new([
                0.30,          // Confine: fisso
                0.3 + t * 0.4, // Valenza: libera
                t * 0.8,       // Intensita: libera
                0.3 + t * 0.4, // Definizione: libera
                t,             // Complessita: libera
                0.3 + t * 0.3, // Permanenza: libera
                0.90,          // Agency: fisso
                0.3 + t * 0.4, // Tempo: libero
            ])));
        }

        reg.calibrate_all_emergent_dimensions(&word_sigs);

        // SPAZIO deve avere la dimensione emergente calibrata
        let spazio = reg.get(SPAZIO).unwrap();
        let calibrated: Vec<&EmergentDimension> = spazio.emergent_dimensions.iter()
            .filter(|d| d.is_calibrated())
            .collect();
        assert!(!calibrated.is_empty(),
            "SPAZIO deve avere dimensione emergente calibrata");
    }
}
