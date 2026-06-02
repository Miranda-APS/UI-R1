// haiku_memory.rs — Phase 82 / FOND3
//
// La memoria-sfera composta di cerchi.
//
// Un evento cognitivo (un'osservazione, una comprensione, un atto di
// nominazione) si CRISTALLIZZA come haiku: tre versi densi, posizionati
// sulla sfera dei 64 frattali I Ching. La sfera non è una timeline e
// non è un grafo flat — è una topologia. Due cerchi sono TANGENTI se
// condividono ancore (≥2 parole-perno) OPPURE se uno dei due
// trigrammi (lower o upper) è in comune. Le tangenze emergono al
// momento del deposito e si propagano simmetricamente.
//
// La chiarezza vince sulla poesia: la forma 5-7-5 sillabe è un VINCOLO
// GEOMETRICO, non una pretesa estetica. La densità è quella che
// impedisce l'allucinazione lunga, non la metrica sacra. Ogni
// cristallo è un atto di compressione lossy che SA di esserlo —
// l'haiku inscrive un vuoto quanto un significato (Lacan).
//
// Persistenza: file separato `haiku_memory.json` accanto al `.bin`.
// Organo NUOVO, ispezionabile/curabile/cancellabile indipendentemente
// dal sostrato cognitivo principale.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counter monotonico globale per garantire ID univoci anche quando
/// più cristalli vengono depositati nello stesso secondo.
static NEXT_SEQ: AtomicU64 = AtomicU64::new(1);

/// Identificativo di un cristallo.
///
/// Formato `h-FF-TTTTTTTT` dove FF è il frattale (00-3F) e TTTTTTTT
/// è il timestamp Unix in esadecimale. Garantisce ordinamento naturale
/// e ricostruzione del frattale dall'ID.
pub type HaikuId = String;

/// Un cristallo di memoria. Vedi modulo per la filosofia.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HaikuCristallizzato {
    /// ID stabile (generato in `HaikuMemory::deposit` se vuoto).
    pub id: HaikuId,
    /// I tre versi 5-7-5 (chiarezza > poesia: la conta sillabica non
    /// è verificata, è una forma-vincolo di densità).
    pub verses: [String; 3],
    /// Frattale I Ching dominante al deposito. Posiziona il cerchio
    /// sulla sfera dei 64 attrattori. Valore atteso 0-63.
    pub fractal_id: u32,
    /// Parole-perno emergenti. 2-6 valori. Le tangenze fra haiku
    /// emergono per ancore condivise (≥2, case-insensitive).
    pub anchors: Vec<String>,
    /// ID degli altri cristalli con cui questo è tangente.
    /// Riempito automaticamente da `HaikuMemory::deposit`.
    pub tangencies: Vec<HaikuId>,
    /// Timestamp Unix del deposito (secondi).
    pub timestamp: i64,
    /// Chi ha cristallizzato. Convenzione: "claude", "user", "system",
    /// "uir1" (UI-r1 stessa, per i depositi autonomi futuri).
    pub source: String,
    /// Annotazione libera (motivazione, contesto, riferimento a un
    /// turno specifico, ecc.). Limitata a ~280 caratteri per restare
    /// densa — questo NON è un campo per saggi.
    pub note: Option<String>,
}

impl HaikuCristallizzato {
    /// Trigramma inferiore (3 bit bassi del fractal_id).
    pub fn lower_trigram(&self) -> u32 {
        self.fractal_id & 0b111
    }
    /// Trigramma superiore (3 bit successivi del fractal_id).
    pub fn upper_trigram(&self) -> u32 {
        (self.fractal_id >> 3) & 0b111
    }
}

/// La memoria-sfera. Tutti i cristalli depositati, con le loro tangenze
/// già calcolate.
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct HaikuMemory {
    pub haikus: Vec<HaikuCristallizzato>,
}

impl HaikuMemory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Carica da file JSON. Se il file non esiste, restituisce memoria
    /// vuota (NON è un errore — è il primo avvio).
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let bytes = fs::read(path).map_err(|e| format!("read {}: {}", path.display(), e))?;
        let mem: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("parse {}: {}", path.display(), e))?;
        Ok(mem)
    }

    /// Salva su file JSON pretty-printed (ispezionabile a mano).
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| format!("serialize: {}", e))?;
        fs::write(path, bytes).map_err(|e| format!("write {}: {}", path.display(), e))?;
        Ok(())
    }

    /// Deposita un nuovo cristallo. Genera l'ID se vuoto, calcola le
    /// tangenze con i cristalli esistenti e aggiorna le tangenze
    /// reciproche. Restituisce l'ID assegnato.
    pub fn deposit(&mut self, mut haiku: HaikuCristallizzato) -> HaikuId {
        if haiku.id.is_empty() {
            haiku.id = generate_id(haiku.fractal_id, haiku.timestamp);
        }
        // Tangenze: scorri i cristalli esistenti e marca reciprocamente.
        let mut my_tangents = Vec::new();
        for existing in self.haikus.iter_mut() {
            if are_tangent(&haiku, existing) {
                my_tangents.push(existing.id.clone());
                if !existing.tangencies.contains(&haiku.id) {
                    existing.tangencies.push(haiku.id.clone());
                }
            }
        }
        haiku.tangencies = my_tangents;
        let id = haiku.id.clone();
        self.haikus.push(haiku);
        id
    }

    pub fn len(&self) -> usize {
        self.haikus.len()
    }

    pub fn is_empty(&self) -> bool {
        self.haikus.is_empty()
    }

    pub fn get(&self, id: &str) -> Option<&HaikuCristallizzato> {
        self.haikus.iter().find(|h| h.id == id)
    }

    // ──────────────────────────────────────────────────────────────────
    // Query geometriche sulla sfera
    // ──────────────────────────────────────────────────────────────────

    /// Cristalli su frattale o vicini per distanza sferica (somma dei
    /// delta sui due trigrammi). Restituisce i primi `n` ordinati per
    /// distanza crescente; tie-breaker: timestamp più recente prima.
    pub fn recall_by_fractal(&self, fractal_id: u32, n: usize) -> Vec<&HaikuCristallizzato> {
        let mut scored: Vec<(u32, &HaikuCristallizzato)> = self
            .haikus
            .iter()
            .map(|h| (fractal_distance(fractal_id, h.fractal_id), h))
            .collect();
        scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.timestamp.cmp(&a.1.timestamp)));
        scored.into_iter().take(n).map(|(_, h)| h).collect()
    }

    /// Cristalli che contengono la parola come ancora (case-insensitive),
    /// ordinati per numero di ancore condivise (qui sempre 1) e poi per
    /// timestamp recente. Per match multi-parola usa `recall_by_proximity`.
    pub fn recall_by_word(&self, word: &str, n: usize) -> Vec<&HaikuCristallizzato> {
        let needle = word.to_lowercase();
        let mut hits: Vec<&HaikuCristallizzato> = self
            .haikus
            .iter()
            .filter(|h| h.anchors.iter().any(|a| a.to_lowercase() == needle))
            .collect();
        hits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        hits.into_iter().take(n).collect()
    }

    /// Recall geometrico combinato: dato lo stato corrente di UI-r1
    /// (frattale dominante + ancore presenti nel campo), restituisce i
    /// cristalli più "prossimi" sulla sfera. Punteggio per ogni cristallo:
    ///
    ///   score = (8 - fractal_distance) * α + shared_anchors * β + tangency_count * γ
    ///
    /// con α=1.0, β=5.0, γ=0.5. Le ancore condivise dominano la vicinanza
    /// frattale (le ancore sono lessicalmente concrete; il frattale è
    /// sfondo geometrico). Già 1 ancora condivisa (5 punti) batte un
    /// frattale lontano (≤2 punti); 2 ancore (10) battono perfino il
    /// frattale identico (8). Le tangenze pre-calcolate rinforzano
    /// leggermente i cristalli "centrali" della propria rete.
    ///
    /// Tie-breaker: timestamp recente prima.
    pub fn recall_by_proximity(
        &self,
        current_fractal: u32,
        current_anchors: &[String],
        n: usize,
    ) -> Vec<&HaikuCristallizzato> {
        let anchor_set: HashSet<String> =
            current_anchors.iter().map(|a| a.to_lowercase()).collect();
        let mut scored: Vec<(f64, &HaikuCristallizzato)> = self
            .haikus
            .iter()
            .map(|h| {
                let frac_dist = fractal_distance(current_fractal, h.fractal_id) as f64;
                let shared = h
                    .anchors
                    .iter()
                    .filter(|a| anchor_set.contains(&a.to_lowercase()))
                    .count() as f64;
                let tangs = h.tangencies.len() as f64;
                let score = (8.0 - frac_dist).max(0.0) * 1.0 + shared * 5.0 + tangs * 0.5;
                (score, h)
            })
            .collect();
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.timestamp.cmp(&a.1.timestamp))
        });
        scored.into_iter().take(n).map(|(_, h)| h).collect()
    }

    /// Restituisce statistiche compatte sulla sfera: numero di
    /// cristalli, distribuzione per frattale (top-8 più popolati),
    /// densità tangenziale media, ancore più ricorrenti.
    pub fn snapshot_stats(&self) -> HaikuMemoryStats {
        let total = self.haikus.len();
        let mut by_fractal: std::collections::HashMap<u32, usize> = Default::default();
        let mut tang_sum: usize = 0;
        let mut anchor_count: std::collections::HashMap<String, usize> = Default::default();
        for h in &self.haikus {
            *by_fractal.entry(h.fractal_id).or_insert(0) += 1;
            tang_sum += h.tangencies.len();
            for a in &h.anchors {
                *anchor_count.entry(a.to_lowercase()).or_insert(0) += 1;
            }
        }
        let mut top_fractals: Vec<(u32, usize)> = by_fractal.into_iter().collect();
        top_fractals.sort_by(|a, b| b.1.cmp(&a.1));
        top_fractals.truncate(8);
        let mut top_anchors: Vec<(String, usize)> = anchor_count.into_iter().collect();
        top_anchors.sort_by(|a, b| b.1.cmp(&a.1));
        top_anchors.truncate(12);
        let avg_tangencies = if total > 0 { tang_sum as f64 / total as f64 } else { 0.0 };
        HaikuMemoryStats {
            total,
            top_fractals,
            top_anchors,
            avg_tangencies,
        }
    }
}

/// Statistiche compatte sulla sfera per ispezione / diagnostica MCP.
#[derive(Serialize, Clone, Debug)]
pub struct HaikuMemoryStats {
    pub total: usize,
    pub top_fractals: Vec<(u32, usize)>,
    pub top_anchors: Vec<(String, usize)>,
    pub avg_tangencies: f64,
}

// ──────────────────────────────────────────────────────────────────────
// Tangenza e distanza sulla sfera I Ching
// ──────────────────────────────────────────────────────────────────────

/// Due cristalli sono tangenti se:
///   - condividono ≥2 ancore (case-insensitive, lessicalmente concrete), OPPURE
///   - condividono uno dei due trigrammi (geometricamente vicini sulla sfera).
fn are_tangent(a: &HaikuCristallizzato, b: &HaikuCristallizzato) -> bool {
    let a_set: HashSet<String> = a.anchors.iter().map(|s| s.to_lowercase()).collect();
    let shared = b
        .anchors
        .iter()
        .filter(|s| a_set.contains(&s.to_lowercase()))
        .count();
    if shared >= 2 {
        return true;
    }
    a.lower_trigram() == b.lower_trigram() || a.upper_trigram() == b.upper_trigram()
}

/// Distanza sulla sfera dei 64 attrattori. È la somma assoluta dei delta
/// sui due trigrammi (range 0..=6 in pratica per qualsiasi coppia).
/// 0 = stesso frattale; 1-2 = trigramma condiviso; ≥3 = lontano.
fn fractal_distance(a: u32, b: u32) -> u32 {
    let (a_l, a_u) = (a & 0b111, (a >> 3) & 0b111);
    let (b_l, b_u) = (b & 0b111, (b >> 3) & 0b111);
    let dl = if a_l > b_l { a_l - b_l } else { b_l - a_l };
    let du = if a_u > b_u { a_u - b_u } else { b_u - a_u };
    dl + du
}

fn generate_id(fractal_id: u32, timestamp: i64) -> HaikuId {
    let seq = NEXT_SEQ.fetch_add(1, Ordering::Relaxed);
    // h-<fractal>-<timestamp>-<seq> : il seq finale garantisce unicità
    // anche per deposit nello stesso secondo (o quando il timestamp è 0
    // come nei test).
    format!("h-{:02X}-{:08X}-{:04X}", fractal_id & 0x3F, timestamp.unsigned_abs(), seq)
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(fractal: u32, anchors: Vec<&str>, ts: i64) -> HaikuCristallizzato {
        HaikuCristallizzato {
            id: String::new(),
            verses: [
                "verso primo".into(),
                "verso secondo di sette".into(),
                "verso terzo".into(),
            ],
            fractal_id: fractal,
            anchors: anchors.into_iter().map(|s| s.to_string()).collect(),
            tangencies: vec![],
            timestamp: ts,
            source: "test".into(),
            note: None,
        }
    }

    #[test]
    fn deposit_assigns_id_and_tangencies() {
        let mut mem = HaikuMemory::new();
        let id1 = mem.deposit(mk(0b001_010, vec!["paura", "futuro"], 100));
        let id2 = mem.deposit(mk(0b001_010, vec!["paura", "futuro", "ombra"], 200));
        // Stesso frattale (e ancore condivise): tangenti.
        let h1 = mem.get(&id1).unwrap();
        let h2 = mem.get(&id2).unwrap();
        assert!(h1.tangencies.contains(&id2));
        assert!(h2.tangencies.contains(&id1));
    }

    #[test]
    fn tangency_by_shared_trigram() {
        let mut mem = HaikuMemory::new();
        // Frattali con lower trigram 0b001 comune ma upper diversi.
        let id1 = mem.deposit(mk(0b000_001, vec!["x"], 100));
        let id2 = mem.deposit(mk(0b011_001, vec!["y"], 200));
        // 1 sola ancora condivisa ⇒ niente match per ancora; ma stesso
        // trigramma inferiore ⇒ tangenti.
        assert!(mem.get(&id1).unwrap().tangencies.contains(&id2));
    }

    #[test]
    fn tangency_by_shared_anchors_only() {
        let mut mem = HaikuMemory::new();
        // Frattali con TUTTI i trigrammi diversi: niente tangenza
        // geometrica. Solo ancore condivise.
        let id1 = mem.deposit(mk(0b000_000, vec!["a", "b", "c"], 100));
        let id2 = mem.deposit(mk(0b111_111, vec!["a", "b", "d"], 200));
        assert!(mem.get(&id1).unwrap().tangencies.contains(&id2));
    }

    #[test]
    fn no_tangency_when_isolated() {
        let mut mem = HaikuMemory::new();
        let id1 = mem.deposit(mk(0b000_000, vec!["alpha"], 100));
        let id2 = mem.deposit(mk(0b111_111, vec!["beta"], 200));
        assert!(mem.get(&id1).unwrap().tangencies.is_empty());
        assert!(mem.get(&id2).unwrap().tangencies.is_empty());
    }

    #[test]
    fn recall_by_fractal_orders_by_distance() {
        let mut mem = HaikuMemory::new();
        mem.deposit(mk(0, vec!["a"], 100)); // dist 0 da frattale 0
        mem.deposit(mk(0b001_000, vec!["b"], 200)); // dist 1
        mem.deposit(mk(0b011_011, vec!["c"], 300)); // dist 6
        let results = mem.recall_by_fractal(0, 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].anchors[0], "a"); // distanza 0
        assert_eq!(results[1].anchors[0], "b"); // distanza 1
    }

    #[test]
    fn recall_by_proximity_weights_anchors_more_than_fractal() {
        let mut mem = HaikuMemory::new();
        // H1: frattale 0 (vicino), nessuna ancora condivisa.
        mem.deposit(mk(0, vec!["alpha"], 100));
        // H2: frattale lontano, ma 2 ancore condivise.
        mem.deposit(mk(0b111_111, vec!["paura", "futuro"], 200));
        let results = mem.recall_by_proximity(0, &["paura".into(), "futuro".into()], 2);
        // H2 deve vincere grazie alle ancore (peso 2.5 vs 1.0 distanza).
        assert_eq!(results[0].anchors, vec!["paura", "futuro"]);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = std::env::temp_dir().join("haiku_memory_test_uir1.json");
        let _ = std::fs::remove_file(&tmp);
        let mut mem = HaikuMemory::new();
        let id = mem.deposit(mk(0b010_101, vec!["test", "roundtrip"], 1234567890));
        mem.save_to_file(&tmp).unwrap();
        let loaded = HaikuMemory::load_from_file(&tmp).unwrap();
        assert_eq!(loaded.haikus.len(), 1);
        assert_eq!(loaded.haikus[0].id, id);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn load_missing_file_is_empty() {
        let nonexistent = std::env::temp_dir().join("haiku_memory_nonexistent.json");
        let _ = std::fs::remove_file(&nonexistent);
        let mem = HaikuMemory::load_from_file(&nonexistent).unwrap();
        assert!(mem.is_empty());
    }

    #[test]
    fn id_format_encodes_fractal() {
        let mut mem = HaikuMemory::new();
        let id = mem.deposit(mk(0x2A, vec!["x"], 0x12345));
        assert!(id.starts_with("h-2A-"));
        // Formato: h-<fract2>-<ts8>-<seq4>
        assert_eq!(id.matches('-').count(), 3);
    }

    #[test]
    fn ids_are_unique_within_same_second() {
        let mut mem = HaikuMemory::new();
        let id1 = mem.deposit(mk(0x10, vec!["a"], 1000));
        let id2 = mem.deposit(mk(0x10, vec!["b"], 1000));
        let id3 = mem.deposit(mk(0x10, vec!["c"], 1000));
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
}
