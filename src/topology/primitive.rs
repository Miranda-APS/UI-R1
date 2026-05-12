/// Le 8 Dimensioni Primitive — L'RGB della semantica di Prometeo.
///
/// Non descrivono il mondo. Lo GENERANO.
/// Ogni dimensione e un asse continuo [0.0, 1.0] con poli opposti.

use std::fmt;

/// Indici delle 8 dimensioni primitive.
/// Usati per accesso posizionale e per identificare quali dimensioni
/// sono fisse vs libere in un frattale.
///
/// Ordine I Ching canonico (Cielo→Lago). Consistente con:
///   - `Trigram::ALL` (Cielo=0, Terra=1, ..., Lago=7)
///   - `KnowledgeGraph::derive_8d_from_kg` (Phase 63)
///   - `phenomenology.tsv` (header)
///   - `valence.rs::DRIVE_DIM` (mappatura Octalysis→dim)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Dim {
    Agency = 0,       // ☰ Cielo     — Paziente ↔ Agente
    Permanenza = 1,   // ☷ Terra     — Transitorio ↔ Stabile
    Intensita = 2,    // ☳ Tuono     — Debole ↔ Forte
    Tempo = 3,        // ☵ Acqua     — Passato ↔ Futuro
    Confine = 4,      // ☶ Montagna  — Esterno ↔ Interno/Io
    Complessita = 5,  // ☴ Vento     — Semplice ↔ Composto
    Definizione = 6,  // ☲ Fuoco     — Vago ↔ Netto
    Valenza = 7,      // ☱ Lago      — Repulsione ↔ Attrazione
}

impl Dim {
    pub const ALL: [Dim; 8] = [
        Dim::Agency, Dim::Permanenza, Dim::Intensita, Dim::Tempo,
        Dim::Confine, Dim::Complessita, Dim::Definizione, Dim::Valenza,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    /// Ricostruisce un Dim dal suo indice [0..7].
    pub fn from_index(idx: usize) -> Dim {
        match idx {
            0 => Dim::Agency,
            1 => Dim::Permanenza,
            2 => Dim::Intensita,
            3 => Dim::Tempo,
            4 => Dim::Confine,
            5 => Dim::Complessita,
            6 => Dim::Definizione,
            7 => Dim::Valenza,
            _ => Dim::Agency, // fallback
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Dim::Confine => "Confine",
            Dim::Valenza => "Valenza",
            Dim::Intensita => "Intensita",
            Dim::Definizione => "Definizione",
            Dim::Complessita => "Complessita",
            Dim::Permanenza => "Permanenza",
            Dim::Agency => "Agency",
            Dim::Tempo => "Tempo",
        }
    }

    /// Ricostruisce un Dim dal suo nome (Debug format).
    pub fn from_name(name: &str) -> Option<Dim> {
        match name {
            "Confine" => Some(Dim::Confine),
            "Valenza" => Some(Dim::Valenza),
            "Intensita" => Some(Dim::Intensita),
            "Definizione" => Some(Dim::Definizione),
            "Complessita" => Some(Dim::Complessita),
            "Permanenza" => Some(Dim::Permanenza),
            "Agency" => Some(Dim::Agency),
            "Tempo" => Some(Dim::Tempo),
            _ => None,
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Dim::Confine => "CON",
            Dim::Valenza => "VAL",
            Dim::Intensita => "INT",
            Dim::Definizione => "DEF",
            Dim::Complessita => "CMP",
            Dim::Permanenza => "PER",
            Dim::Agency => "AGE",
            Dim::Tempo => "TMP",
        }
    }

    pub fn poles(self) -> (&'static str, &'static str) {
        match self {
            Dim::Confine => ("Esterno", "Interno"),
            Dim::Valenza => ("Repulsione", "Attrazione"),
            Dim::Intensita => ("Debole", "Forte"),
            Dim::Definizione => ("Vago", "Netto"),
            Dim::Complessita => ("Semplice", "Composto"),
            Dim::Permanenza => ("Transitorio", "Stabile"),
            Dim::Agency => ("Paziente", "Agente"),
            Dim::Tempo => ("Passato", "Futuro"),
        }
    }
}

/// Il core primitivo a 8 dimensioni.
/// Ogni valore e nell'intervallo [0.0, 1.0].
#[derive(Clone, Copy, PartialEq)]
pub struct PrimitiveCore {
    values: [f64; 8],
}

impl PrimitiveCore {
    /// Core neutro: tutto a 0.5 (centro di ogni asse).
    pub fn neutral() -> Self {
        Self { values: [0.5; 8] }
    }

    /// Core da valori espliciti. Clampa automaticamente a [0.0, 1.0].
    pub fn new(values: [f64; 8]) -> Self {
        let mut v = values;
        for x in &mut v {
            *x = x.clamp(0.0, 1.0);
        }
        Self { values: v }
    }

    /// Accesso per dimensione.
    pub fn get(&self, dim: Dim) -> f64 {
        self.values[dim.index()]
    }

    /// Modifica una dimensione (clampata).
    pub fn set(&mut self, dim: Dim, value: f64) {
        self.values[dim.index()] = value.clamp(0.0, 1.0);
    }

    /// Accesso diretto all'array.
    pub fn values(&self) -> &[f64; 8] {
        &self.values
    }

    /// Distanza euclidea nello spazio 8D.
    pub fn distance(&self, other: &Self) -> f64 {
        self.values.iter()
            .zip(other.values.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt()
    }

    /// Distanza pesata: alcune dimensioni contano piu di altre nel contesto dato.
    pub fn weighted_distance(&self, other: &Self, weights: &[f64; 8]) -> f64 {
        self.values.iter()
            .zip(other.values.iter())
            .zip(weights.iter())
            .map(|((a, b), w)| w * (a - b) * (a - b))
            .sum::<f64>()
            .sqrt()
    }

    /// Interferenza elastica: sposta self verso other con una certa forza.
    /// Non sovrascrive — deforma.
    pub fn perturb_towards(&mut self, other: &Self, strength: f64) {
        let s = strength.clamp(0.0, 1.0);
        for i in 0..8 {
            self.values[i] = self.values[i] * (1.0 - s) + other.values[i] * s;
        }
    }

    /// Energia del core: distanza dal centro (neutro).
    pub fn energy(&self) -> f64 {
        self.values.iter()
            .map(|v| (v - 0.5) * (v - 0.5))
            .sum::<f64>()
            .sqrt()
    }

    /// Media pesata di due core.
    pub fn blend(a: &Self, b: &Self, weight_a: f64) -> Self {
        let w = weight_a.clamp(0.0, 1.0);
        let mut values = [0.0; 8];
        for i in 0..8 {
            values[i] = a.values[i] * w + b.values[i] * (1.0 - w);
        }
        Self { values }
    }

    /// Proietta il core lungo una dimensione specifica (estrae il valore, azzera il resto).
    pub fn project(&self, dim: Dim) -> f64 {
        self.values[dim.index()]
    }
}

impl fmt::Debug for PrimitiveCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Core8D[")?;
        for (i, dim) in Dim::ALL.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}:{:.2}", dim.name(), self.values[i])?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for PrimitiveCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neutral_is_centered() {
        let core = PrimitiveCore::neutral();
        for dim in &Dim::ALL {
            assert!((core.get(*dim) - 0.5).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_distance_to_self_is_zero() {
        let core = PrimitiveCore::new([0.2, 0.8, 0.1, 0.9, 0.5, 0.3, 0.7, 0.4]);
        assert!(core.distance(&core) < f64::EPSILON);
    }

    #[test]
    fn test_perturb_towards() {
        let mut a = PrimitiveCore::neutral();
        let b = PrimitiveCore::new([1.0; 8]);
        a.perturb_towards(&b, 0.5);
        for dim in &Dim::ALL {
            assert!((a.get(*dim) - 0.75).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_clamp() {
        // Ordine I Ching: [Agency, Permanenza, Intensita, Tempo, Confine, Complessita, Definizione, Valenza]
        let core = PrimitiveCore::new([1.5, -0.3, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);
        assert!((core.get(Dim::Agency) - 1.0).abs() < f64::EPSILON);       // clamp superiore: 1.5 → 1.0
        assert!((core.get(Dim::Permanenza) - 0.0).abs() < f64::EPSILON);  // clamp inferiore: -0.3 → 0.0
    }

    #[test]
    fn test_energy_of_neutral_is_zero() {
        let core = PrimitiveCore::neutral();
        assert!(core.energy() < f64::EPSILON);
    }

    #[test]
    fn test_blend() {
        let a = PrimitiveCore::new([0.0; 8]);
        let b = PrimitiveCore::new([1.0; 8]);
        let c = PrimitiveCore::blend(&a, &b, 0.5);
        for dim in &Dim::ALL {
            assert!((c.get(*dim) - 0.5).abs() < f64::EPSILON);
        }
    }
}
