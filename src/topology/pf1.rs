/// PrometeoField (PF1) — Il substrato topologico di Prometeo.
///
/// PRINCIPIO FONDAMENTALE:
///   Il costo di ogni operazione è proporzionale all'ATTIVITÀ, non alla dimensione.
///
/// DUE LAYER SEPARATI:
///
///   CAMPO (disco, mmap) — struttura permanente, non cambia durante la conversazione:
///     • firme 8D di ogni parola
///     • archi verso i vicini (peso + fase)
///     • affinità frattali precalcolate
///
///   ATTIVAZIONI (RAM) — stato volatile, cambia ogni tick:
///     • [f32; N] — 27KB per 6751 parole
///     • proporzionale alla dimensione del lessico, mai di più
///
/// PROPAGAZIONE:
///   O(parole_attive × 8) — non O(totale_archi)
///   Con 100 parole attive su 6751: 800 operazioni, non 50.000.
///   Come le sinapsi: più connessioni = routing più preciso, non più lento.
///
/// FORMATO FILE (PF1):
///   [HEADER 128 byte] [RECORD × N  (256 byte/parola)]
///   Le parole sono ordinate per frattale dominante — ogni frattale è un range
///   contiguo nel file. La cache del processore lavora CON la struttura.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Write, Read, BufWriter};
use std::path::Path;

use crate::topology::lexicon::Lexicon;
use crate::topology::grammar::PartOfSpeech;
use crate::topology::word_topology::WordTopology;
use crate::topology::simplex::SimplicialComplex;

// ═══════════════════════════════════════════════════════════════════════
// COSTANTI
// ═══════════════════════════════════════════════════════════════════════

pub const RECORD_SIZE: usize = 512;
pub const HEADER_SIZE: usize = 128;
pub const MAGIC: &[u8; 8] = b"PMTF0002";
pub const MAX_WORD_BYTES: usize = 32;
pub const MAX_NEIGHBORS: usize = 8;
pub const MAX_FRACTALS: usize = 64;
pub const PF1_VERSION: u16 = 2;

// ═══════════════════════════════════════════════════════════════════════
// WORD RECORD — 256 byte per parola (layout fisso, accesso O(1))
// ═══════════════════════════════════════════════════════════════════════

/// Un record di parola nel campo topologico.
///
/// 512 byte fissi. Offset = HEADER_SIZE + word_id * 512.
/// Accesso O(1) senza indici, senza allocazioni.
///
/// Layout (repr C, allineato):
///   [0..32]    firma 8D (8 × f32)
///   [32..288]  affinità frattali (64 × f32) — tutti e 64 gli esagrammi
///   [288..292] stabilità (f32)
///   [292..296] conteggio esposizioni (u32)
///   [296..298] frattale dominante (u16)
///   [298]      part-of-speech (u8: 0=Sconosciuto, 1=Verbo, 2=Nome, 3=Agg, 4=Avv)
///   [299]      lunghezza parola in byte (u8)
///   [300..332] parola UTF-8, max 32 byte ([u8; 32])
///   [332]      numero vicini (u8, max 8)
///   [333..336] padding allineamento ([u8; 3])
///   [336..368] word_id dei top-8 vicini ([u32; 8])
///   [368..400] pesi degli archi [0,1] ([f32; 8])
///   [400..432] fasi degli archi [0,π] ([f32; 8])
///   [432..512] padding riservato ([u8; 80])
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WordRecord {
    pub signature:        [f32; 8],
    pub affinities:       [f32; MAX_FRACTALS],
    pub stability:        f32,
    pub exposure_count:   u32,
    pub dominant_fractal: u16,
    pub pos:              u8,
    pub word_len:         u8,
    pub word:             [u8; MAX_WORD_BYTES],
    pub neighbor_count:   u8,
    pub _pad:             [u8; 3],
    pub neighbors:        [u32; MAX_NEIGHBORS],
    pub neighbor_weights: [f32; MAX_NEIGHBORS],
    pub neighbor_phases:  [f32; MAX_NEIGHBORS],
    pub _reserved:        [u8; 80],
}

impl WordRecord {
    /// Record vuoto — parola sconosciuta.
    pub fn empty() -> Self {
        Self {
            signature:        [0.5; 8],
            affinities:       [0.0; MAX_FRACTALS],
            stability:        0.0,
            exposure_count:   0,
            dominant_fractal: 0,
            pos:              0,
            word_len:         0,
            word:             [0u8; MAX_WORD_BYTES],
            neighbor_count:   0,
            _pad:             [0u8; 3],
            neighbors:        [0u32; MAX_NEIGHBORS],
            neighbor_weights: [0.0; MAX_NEIGHBORS],
            neighbor_phases:  [std::f32::consts::FRAC_PI_2; MAX_NEIGHBORS],
            _reserved:        [0u8; 80],
        }
    }

    /// Restituisce la parola come stringa.
    pub fn word_str(&self) -> &str {
        let len = self.word_len as usize;
        std::str::from_utf8(&self.word[..len]).unwrap_or("")
    }

    /// Restituisce la firma come slice.
    pub fn signature_slice(&self) -> &[f32] {
        &self.signature
    }
}

// Verifica a compile-time (in un test) che il layout sia esattamente 512 byte.
const _: () = assert!(std::mem::size_of::<WordRecord>() == RECORD_SIZE,
    "WordRecord deve essere esattamente 512 byte");

// ═══════════════════════════════════════════════════════════════════════
// ACTIVATION STATE — il layer volatile in RAM
// ═══════════════════════════════════════════════════════════════════════

/// Lo stato di attivazione del campo — l'unica cosa in RAM durante la conversazione.
///
/// Dimensione: word_count * 4 byte ≈ 27KB per 6751 parole.
/// Risiede interamente in RAM. La struttura (WordRecord) è sul disco.
pub struct ActivationState {
    /// Attivazione corrente per ogni parola [0.0, 1.0]
    pub activations: Vec<f32>,
    /// Quante volte ogni parola è stata attivata in questa sessione
    pub counts: Vec<u64>,
    /// Soglia minima per considerare una parola "attiva"
    pub threshold: f32,
    /// Pesi sinaptici vivi (RAM) — [word_id * MAX_NEIGHBORS + slot].
    /// Inizializzati da WordRecord.neighbor_weights (ROM basale),
    /// poi aggiornati per LTP/LTD hebbiano durante la conversazione.
    /// La topologia (chi connette chi) rimane ROM; solo la forza cambia.
    pub synapse_weights: Vec<f32>,
}

impl ActivationState {
    pub fn new(word_count: usize) -> Self {
        Self {
            activations: vec![0.0; word_count],
            counts: vec![0; word_count],
            threshold: 0.02,
            // Peso sinaptico neutro (1.0) — verrà inizializzato dai pesi ROM
            // tramite init_synapse_weights_from_field() subito dopo la costruzione.
            synapse_weights: vec![1.0; word_count * MAX_NEIGHBORS],
        }
    }

    /// Inizializza i pesi sinaptici dai valori basali del campo ROM.
    ///
    /// Chiamare DOPO rebuild_pf_field() — copia i pesi iniziali (storia
    /// di apprendimento cristallizzata) dalla ROM nella RAM sinaptica.
    /// Da quel momento i pesi evolvono per Hebbian LTP/LTD.
    pub fn init_synapse_weights_from_field(&mut self, field: &PrometeoField) {
        let n = field.word_count as usize;
        self.synapse_weights = vec![0.0; n * MAX_NEIGHBORS];
        for id in 0..n {
            let record = field.record(id as u32);
            for j in 0..record.neighbor_count as usize {
                self.synapse_weights[id * MAX_NEIGHBORS + j] = record.neighbor_weights[j];
            }
        }
    }

    /// Attiva una parola per ID.
    #[inline]
    pub fn activate(&mut self, id: u32, strength: f32) {
        let i = id as usize;
        if i < self.activations.len() {
            self.activations[i] = (self.activations[i] + strength).min(1.0);
            self.counts[i] += 1;
        }
    }

    /// Attiva una parola per nome — richiede lookup nel campo.
    pub fn activate_by_name(&mut self, field: &PrometeoField, name: &str, strength: f32) {
        if let Some(id) = field.word_id(name) {
            self.activate(id, strength);
        }
    }

    /// Imposta l'attivazione di una parola per nome (SET, non ADD).
    pub fn set_by_name(&mut self, field: &PrometeoField, name: &str, val: f32) {
        if let Some(id) = field.word_id(name) {
            self.activations[id as usize] = val.clamp(0.0, 1.0);
        }
    }

    /// Decadimento globale. rate = 0.15 → mantiene 85% dell'attivazione.
    pub fn decay_all(&mut self, rate: f32) {
        let keep = 1.0 - rate;
        for a in self.activations.iter_mut() {
            *a *= keep;
            if *a < 0.001 {
                *a = 0.0;
            }
        }
    }

    /// Decadimento moltiplicativo delle attivazioni.
    ///
    /// `rate` = fattore di mantenimento (0.85 → mantiene 85%, decade del 15%).
    /// A differenza di decay_all (che usa rate come percentuale di decadimento),
    /// questo è un moltiplicatore diretto: decay(0.85) ≡ decay_all(0.15).
    /// Usato in propagate_field_words() invece del reset completo,
    /// per permettere all'attivazione di persistere tra i frame (memoria di campo).
    pub fn decay(&mut self, rate: f32) {
        for a in self.activations.iter_mut() {
            *a *= rate;
            if *a < 0.001 {
                *a = 0.0;
            }
        }
    }

    /// Propagazione dell'attivazione nel campo.
    ///
    /// COMPLESSITÀ: O(parole_attive × MAX_NEIGHBORS)
    ///   Con 100 attive su 6751: 800 op — non 50.000.
    ///
    /// La fase dell'arco determina il tipo di propagazione:
    ///   cos(0)   = +1 → risonanza (amplifica)
    ///   cos(π/2) =  0 → tensione creativa (nessuna propagazione)
    ///   cos(π)   = -1 → opposizione (inibisce)
    pub fn propagate(&mut self, field: &PrometeoField) {
        let damping = 0.15_f32;

        // Decadimento: le attivazioni sopra soglia scendono gradualmente.
        // Senza input esterno, il campo torna a riposo in ~30 tick (~90s).
        // Il resting state (sotto soglia) non viene toccato.
        let decay_rate = 0.92_f32;
        for act in self.activations.iter_mut() {
            if *act > self.threshold {
                *act *= decay_rate;
                // Se scende sotto soglia, lasciagli il floor di riposo
                if *act < self.threshold { *act = self.threshold * 0.5; }
            }
        }

        // Raccogli solo le parole attive — il fronte di attivazione
        let hot: Vec<(u32, f32)> = self.activations.iter().enumerate()
            .filter(|(_, &a)| a > self.threshold)
            .map(|(i, &a)| (i as u32, a))
            .collect();

        // Accumula i delta senza modificare activations durante il loop
        let mut deltas = vec![0.0f32; self.activations.len()];

        for (src_id, src_act) in &hot {
            let record = field.record(*src_id);
            let n = record.neighbor_count as usize;
            let base = *src_id as usize * MAX_NEIGHBORS;

            for i in 0..n {
                let nid = record.neighbors[i] as usize;
                if nid >= self.activations.len() { continue; }

                // Sinapsi viva dalla RAM (aggiornata per LTP/LTD hebbiano).
                // Fallback ai pesi ROM se synapse_weights non ancora inizializzati.
                let weight = if base + i < self.synapse_weights.len() {
                    self.synapse_weights[base + i]
                } else {
                    record.neighbor_weights[i]
                };
                let phase = record.neighbor_phases[i];  // geometria fisica — ROM

                // Formula unica: cos(fase) determina segno e intensità
                let contribution = src_act * damping * weight * phase.cos();

                if contribution.abs() < 0.001 { continue; }

                if contribution > 0.0 {
                    // Phase 55: rinforzo positivo con rendimenti decrescenti.
                    // Parole già attive ricevono meno boost (evita retroazione)
                    // ma NON sono bloccate completamente — altrimenti i vicini
                    // semantici dell'input non vengono amplificati dalla propagazione.
                    let current = self.activations[nid];
                    let diminish = if current <= self.threshold {
                        1.0  // sotto soglia: pieno effetto
                    } else {
                        // rendimento decrescente: 1/(1 + 4*current)
                        // a 0.15 → 0.63, a 0.30 → 0.45, a 0.50 → 0.33
                        1.0 / (1.0 + 4.0 * current)
                    };
                    deltas[nid] += contribution * diminish;
                } else {
                    // Opposizione: inibisce a qualsiasi livello
                    deltas[nid] += contribution;
                }
            }
        }

        // Cap positivo: previene convergenza hub.
        // Parole con molti archi entranti accumulano delta enormi;
        // il cap assicura che nessuna parola venga "eletta" dalla propagazione
        // più forte del segnale diretto dell'input (tipicamente 0.3-0.5).
        const MAX_POSITIVE_DELTA: f32 = 0.15;
        for delta in deltas.iter_mut() {
            if *delta > MAX_POSITIVE_DELTA { *delta = MAX_POSITIVE_DELTA; }
        }

        // Applica tutti i delta
        for (i, delta) in deltas.iter().enumerate() {
            if delta.abs() > 0.001 {
                self.activations[i] = (self.activations[i] + delta).clamp(0.0, 1.0);
            }
        }
    }

    /// Aggiornamento hebbiano delle sinapsi dopo propagazione.
    ///
    /// "Neurons that fire together, wire together."
    ///
    /// Per ogni coppia (sorgente attiva, vicino attivo): LTP — rafforza la sinapsi.
    /// Per ogni coppia (sorgente attiva, vicino inattivo): LTD — indebolisce la sinapsi.
    ///
    /// Costanti:
    ///   LTP  = 0.05  — guadagno per co-attivazione
    ///   LTD  = 0.995 — decadimento per vicino inattivo (0.5%/step)
    ///   MAX  = 3.0   — peso massimo (3× il basale ROM)
    pub fn hebbian_update(&mut self, field: &PrometeoField) {
        const LTP: f32 = 0.05;
        const LTD_DECAY: f32 = 0.995;
        const MAX_WEIGHT: f32 = 3.0;

        if self.synapse_weights.is_empty() { return; }

        let hot: Vec<(u32, f32)> = self.activations.iter().enumerate()
            .filter(|(_, &a)| a > self.threshold)
            .map(|(i, &a)| (i as u32, a))
            .collect();

        for (src_id, src_act) in &hot {
            let record = field.record(*src_id);
            let base = *src_id as usize * MAX_NEIGHBORS;
            if base >= self.synapse_weights.len() { continue; }

            for i in 0..record.neighbor_count as usize {
                let nid = record.neighbors[i] as usize;
                if nid >= self.activations.len() { continue; }
                let sw_idx = base + i;
                if sw_idx >= self.synapse_weights.len() { continue; }

                let neighbor_act = self.activations[nid];
                if neighbor_act > self.threshold {
                    // LTP: entrambi co-attivi — rinforza la sinapsi
                    self.synapse_weights[sw_idx] =
                        (self.synapse_weights[sw_idx] + LTP * src_act * neighbor_act)
                        .min(MAX_WEIGHT);
                } else {
                    // LTD: sorgente attiva ma vicino silenzioso — indebolisce
                    self.synapse_weights[sw_idx] *= LTD_DECAY;
                }
            }
        }
    }

    /// Seme dello stato di riposo — l'entità "esiste" anche senza input.
    ///
    /// Le parole più stabili hanno una presenza minima nel campo.
    /// Formula: activation = stability × 0.02
    /// Il baseline è volutamente basso: il segnale semantico dell'input (0.5-0.8)
    /// deve dominare sul rumore di fondo. Un baseline di 0.02 × 0.9 = 0.018
    /// è 30× più debole → il campo risponde a ciò che viene detto, non a ciò
    /// che è stato letto più spesso nei testi didattici.
    pub fn seed_resting_state(&mut self, field: &PrometeoField) {
        for id in 0..field.word_count {
            let record = field.record(id);
            if record.stability > 0.20 {
                let initial = record.stability * 0.002;
                self.activate(id, initial);
            }
        }
    }

    /// Attivazioni frattali emergenti dallo stato corrente.
    ///
    /// Ogni parola attiva vota per i suoi top-3 frattali (affinità più alta).
    /// Questo produce differenziazione netta: solo i frattali nella regione
    /// semantica dell'input emergono come dominanti.
    /// Il risultato è normalizzato al massimo → [0, 1].
    pub fn emerge_fractal_activations(&self, field: &PrometeoField) -> [f32; MAX_FRACTALS] {
        let mut scores = [0.0f32; MAX_FRACTALS];

        for (id, &act) in self.activations.iter().enumerate() {
            if act <= self.threshold { continue; }
            let record = field.record(id as u32);

            // Trova i top-3 frattali per affinità di questa parola
            let mut top3 = [(0usize, 0.0f32); 3];
            for f in 0..MAX_FRACTALS {
                let aff = record.affinities[f];
                // Inserisci nel top-3 se meglio del minimo corrente
                if aff > top3[2].1 {
                    top3[2] = (f, aff);
                    // Bubble sort di 3 elementi
                    if top3[2].1 > top3[1].1 { top3.swap(1, 2); }
                    if top3[1].1 > top3[0].1 { top3.swap(0, 1); }
                }
            }

            // Vota solo per i top-3
            for &(f, aff) in &top3 {
                if aff > 0.0 {
                    scores[f] += act * aff;
                }
            }
        }

        // Normalizza al massimo → il frattale dominante è 1.0
        let max_score = scores.iter().cloned().fold(0.0f32, f32::max);
        if max_score > 0.0 {
            for s in scores.iter_mut() {
                *s /= max_score;
            }
        }
        scores
    }

    /// Parole attive con nome e attivazione, ordinate per attivazione decrescente.
    pub fn hot_words(&self, field: &PrometeoField, limit: usize) -> Vec<(String, f32)> {
        let mut result: Vec<(String, f32)> = self.activations.iter().enumerate()
            .filter(|(_, &a)| a > self.threshold)
            .map(|(id, &a)| (field.record(id as u32).word_str().to_string(), a))
            .collect();
        result.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result.truncate(limit);
        result
    }

    /// Energia totale del campo.
    pub fn field_energy(&self) -> f32 {
        self.activations.iter().sum()
    }

    /// Numero di parole attive sopra soglia.
    pub fn active_count(&self) -> usize {
        self.activations.iter().filter(|&&a| a > self.threshold).count()
    }

    /// Reset completo delle attivazioni.
    pub fn reset(&mut self) {
        self.activations.iter_mut().for_each(|a| *a = 0.0);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PROMETEO FIELD — il campo topologico sul disco
// ═══════════════════════════════════════════════════════════════════════

/// Il campo topologico di Prometeo — struttura permanente su disco.
///
/// In RAM: solo l'indice nome→id (HashMap<String, u32>, ~100KB).
/// Sul disco: tutti i record (256 byte × N parole).
/// Durante la conversazione: la struttura non cambia.
/// I cambiamenti (nuove parole apprese) vengono scritti alla fine della sessione.
pub struct PrometeoField {
    /// Tutti i record in memoria — caricati dal file o costruiti da lessico.
    /// In futuro: mmap diretto dal file (zero-copy).
    data: Vec<WordRecord>,
    /// Indice nome → word_id per lookup O(log N) → O(1)
    word_to_id: HashMap<String, u32>,
    /// Numero totale di parole
    pub word_count: u32,
}

impl PrometeoField {
    /// Costruisce il campo dal lessico e dalla topologia esistente.
    ///
    /// Processo:
    ///   1. Raccoglie tutti i WordPattern dal lessico
    ///   2. Ordina per frattale dominante (località cache per operazioni frattali)
    ///   3. Per ogni parola, trova i top-8 vicini per peso dalla word_topology
    ///   4. Scrive i record in ordine
    pub fn build_from_lexicon(
        lexicon: &Lexicon,
        topology: &WordTopology,
        complex: Option<&SimplicialComplex>,
    ) -> Self {
        use std::f32::consts::FRAC_PI_2;

        // Raccoglie tutte le parole con i loro dati
        let mut entries: Vec<(String, WordRecord)> = Vec::new();

        for (word, pattern) in lexicon.patterns_iter() {
            let mut record = WordRecord::empty();

            // Firma 8D (f64 → f32, precisione sufficiente)
            let sig = pattern.signature.values();
            for i in 0..8.min(sig.len()) {
                record.signature[i] = sig[i] as f32;
            }

            // Affinità frattali
            for (&fid, &aff) in &pattern.fractal_affinities {
                let idx = fid as usize;
                if idx < MAX_FRACTALS {
                    record.affinities[idx] = aff as f32;
                }
            }

            // Stabilità e conteggio
            record.stability = pattern.stability as f32;
            record.exposure_count = pattern.exposure_count.min(u32::MAX as u64) as u32;

            // Frattale dominante
            record.dominant_fractal = pattern.fractal_affinities.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(&fid, _)| fid as u16)
                .unwrap_or(0);

            // Part-of-speech
            record.pos = match pattern.pos {
                Some(PartOfSpeech::Verb)      => 1,
                Some(PartOfSpeech::Noun)      => 2,
                Some(PartOfSpeech::Adjective) => 3,
                Some(PartOfSpeech::Adverb)    => 4,
                _                             => 0,
            };

            // Parola come bytes
            let word_bytes = word.as_bytes();
            let len = word_bytes.len().min(MAX_WORD_BYTES);
            record.word[..len].copy_from_slice(&word_bytes[..len]);
            record.word_len = len as u8;

            entries.push((word.clone(), record));
        }

        // Ordina per frattale dominante, poi per nome (località cache + ricerca binaria)
        entries.sort_unstable_by(|a, b| {
            a.1.dominant_fractal.cmp(&b.1.dominant_fractal)
                .then(a.0.cmp(&b.0))
        });

        // Costruisce l'indice nome → id (gli id sono gli indici nell'array ordinato)
        let mut word_to_id: HashMap<String, u32> = HashMap::new();
        let mut topo_id_to_pf1_id: HashMap<u32, u32> = HashMap::new();

        for (pf1_id, (word, _)) in entries.iter().enumerate() {
            word_to_id.insert(word.clone(), pf1_id as u32);
            // Mappa topology WordId → PF1 id (per copiare gli archi)
            if let Some(topo_id) = topology.word_id(word) {
                topo_id_to_pf1_id.insert(topo_id, pf1_id as u32);
            }
        }

        // Copia i vicini dalla topology in ogni record
        let mut data: Vec<WordRecord> = entries.into_iter().map(|(_, r)| r).collect();

        // Precomputa frattale → [(pf1_id, affinità)] per selezione vicini da simplicial complex.
        // Mappa invertita: dato un frattale, quali parole vi risuonano di più?
        let fractal_word_map: HashMap<u32, Vec<(u32, f32)>> = if complex.is_some() {
            let mut fwm: HashMap<u32, Vec<(u32, f32)>> = HashMap::new();
            for (word, pattern) in lexicon.patterns_iter() {
                let pf1_id = match word_to_id.get(word) {
                    Some(&id) => id,
                    None => continue,
                };
                for (&fid, &aff) in &pattern.fractal_affinities {
                    if aff > 0.15 {
                        fwm.entry(fid).or_default().push((pf1_id, aff as f32));
                    }
                }
            }
            // Ordina per affinità decrescente, limita a top-10 per frattale
            for v in fwm.values_mut() {
                v.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                v.truncate(10);
            }
            fwm
        } else {
            HashMap::new()
        };

        // Pre-computazione cache: frattale → top-5 simplessi ordinati per persistenza.
        // Costo: O(N_frattali × sort) — una sola volta invece di O(N_parole × 3 × sort).
        // Elimina ~700M HashMap lookup durante il loop parole (il vero bottleneck del secondo build).
        let fractal_simplex_cache: Option<HashMap<u32, Vec<u32>>> =
            complex.map(|cx| cx.build_fractal_simplex_cache(5));

        // Per ogni parola, trova i top-8 vicini per peso
        for pf1_id in 0..data.len() {
            let word = {
                let r = &data[pf1_id];
                std::str::from_utf8(&r.word[..r.word_len as usize])
                    .unwrap_or("").to_string()
            };

            let topo_id = match topology.word_id(&word) {
                Some(id) => id,
                None => continue,
            };

            // I vicini vengono dalla word_topology (KG semantico) come PRIMA fonte.
            // Il simplicial complex (pensieri cristallizzati) rinforza i pesi
            // ma NON sostituisce le relazioni logiche.
            //
            // Principio: un bambino impara prima CHE "cane IS_A animale" (KG),
            // poi conferma con l'esperienza (co-attivazione nei simplessi).
            // La comprensione logica guida, la statistica rinforza.
            let mut neighbor_scores: HashMap<u32, (f32, f32)> = HashMap::new(); // pf1_id → (weight, phase)

            // 1. Vicini semantici dalla word_topology (KG: IS_A, CAUSES, SIMILAR_TO...)
            for neighbor_word in neighbor_words(topology, topo_id) {
                let pf1_neighbor = match word_to_id.get(&neighbor_word) {
                    Some(&id) => id,
                    None => continue,
                };
                if pf1_neighbor == pf1_id as u32 { continue; }
                let weight = topology.edge_weight_between(&word, &neighbor_word).unwrap_or(0.0) as f32;
                let phase  = topology.edge_phase(&word, &neighbor_word)
                    .unwrap_or(std::f64::consts::FRAC_PI_2) as f32;
                if weight > 0.01 {
                    neighbor_scores.insert(pf1_neighbor, (weight, phase));
                }
            }

            // 2. Simplicial complex rinforza (bonus +30% se confermato da esperienza)
            if let Some(cx) = complex {
                let cache = fractal_simplex_cache.as_ref().unwrap();
                let simplex_neighbors = simplex_based_neighbors(
                    pf1_id as u32,
                    &data[pf1_id].affinities,
                    &data[pf1_id].signature,
                    cx,
                    cache,
                    &fractal_word_map,
                    &data,
                );
                for (nid, sx_weight, sx_phase) in simplex_neighbors {
                    let entry = neighbor_scores.entry(nid).or_insert((0.0, sx_phase));
                    // Se già presente nel KG: rinforza. Se solo nel complex: aggiungilo con peso ridotto.
                    if entry.0 > 0.0 {
                        entry.0 = (entry.0 + sx_weight * 0.3).min(1.0); // KG + rinforzo esperienza
                    } else {
                        entry.0 = sx_weight * 0.5; // Solo esperienza, peso dimezzato
                    }
                }
            }

            // Ordina per peso decrescente, prendi top-8
            let mut neighbors_raw: Vec<(u32, f32, f32)> = neighbor_scores.into_iter()
                .map(|(id, (w, p))| (id, w, p))
                .collect();
            neighbors_raw.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            neighbors_raw.truncate(MAX_NEIGHBORS);

            let record = &mut data[pf1_id];
            record.neighbor_count = neighbors_raw.len() as u8;
            for (i, (nid, w, ph)) in neighbors_raw.into_iter().enumerate() {
                record.neighbors[i]        = nid;
                record.neighbor_weights[i] = w;
                record.neighbor_phases[i]  = ph;
            }
        }

        let word_count = data.len() as u32;

        Self { data, word_to_id, word_count }
    }

    /// Accede a un record per ID — O(1), nessuna allocazione.
    #[inline]
    pub fn record(&self, id: u32) -> &WordRecord {
        &self.data[id as usize]
    }

    /// Restituisce il WordId di una parola per nome — O(1).
    #[inline]
    pub fn word_id(&self, name: &str) -> Option<u32> {
        self.word_to_id.get(&name.to_lowercase()).copied()
    }

    /// Restituisce il nome di una parola per ID.
    pub fn word_name(&self, id: u32) -> &str {
        if id < self.word_count {
            self.data[id as usize].word_str()
        } else {
            ""
        }
    }

    /// Campo vuoto — usato per inizializzazione dell'engine prima del primo rebuild.
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            word_to_id: HashMap::new(),
            word_count: 0,
        }
    }

    /// Aggiunge una nuova parola al campo (apprendimento).
    /// Ritorna il nuovo word_id assegnato.
    pub fn add_word(&mut self, word: &str, record: WordRecord) -> u32 {
        let id = self.word_count;
        self.word_to_id.insert(word.to_lowercase(), id);
        self.data.push(record);
        self.word_count += 1;
        id
    }

    /// Salva il campo su file in formato PF1.
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Header 128 byte
        let mut header = [0u8; HEADER_SIZE];
        header[0..8].copy_from_slice(MAGIC);
        header[8..10].copy_from_slice(&PF1_VERSION.to_le_bytes());
        header[10..14].copy_from_slice(&self.word_count.to_le_bytes());
        header[14..16].copy_from_slice(&(MAX_FRACTALS as u16).to_le_bytes());
        writer.write_all(&header)?;

        // Record
        for record in &self.data {
            // Converti il record in bytes (repr(C) garantisce layout fisso)
            let bytes: &[u8] = unsafe {
                std::slice::from_raw_parts(
                    record as *const WordRecord as *const u8,
                    RECORD_SIZE,
                )
            };
            writer.write_all(bytes)?;
        }

        Ok(())
    }

    /// Carica il campo da file PF1.
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut raw = Vec::new();
        file.read_to_end(&mut raw)?;

        // Verifica magic
        if raw.len() < HEADER_SIZE || &raw[0..8] != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File PF1 non valido o magic errato",
            ));
        }

        let word_count = u32::from_le_bytes(raw[10..14].try_into().unwrap());

        // Leggi i record
        let mut data: Vec<WordRecord> = Vec::with_capacity(word_count as usize);
        let mut word_to_id = HashMap::new();

        for i in 0..word_count as usize {
            let offset = HEADER_SIZE + i * RECORD_SIZE;
            if offset + RECORD_SIZE > raw.len() { break; }

            let record: WordRecord = unsafe {
                std::ptr::read_unaligned(raw[offset..].as_ptr() as *const WordRecord)
            };

            let word = record.word_str().to_string();
            if !word.is_empty() {
                word_to_id.insert(word, i as u32);
            }
            data.push(record);
        }

        Ok(Self { data, word_to_id, word_count })
    }

    /// Statistiche del campo.
    pub fn stats(&self) -> FieldStats {
        let total_edges: usize = self.data.iter()
            .map(|r| r.neighbor_count as usize)
            .sum();

        FieldStats {
            word_count: self.word_count,
            total_edges: total_edges as u32,
            avg_neighbors: if self.word_count > 0 {
                total_edges as f32 / self.word_count as f32
            } else {
                0.0
            },
        }
    }
}

/// Statistiche riassuntive del campo.
#[derive(Debug)]
pub struct FieldStats {
    pub word_count: u32,
    pub total_edges: u32,
    pub avg_neighbors: f32,
}

// ═══════════════════════════════════════════════════════════════════════
// Helper interno: lista nomi vicini da WordTopology
// ═══════════════════════════════════════════════════════════════════════

fn neighbor_words(topology: &WordTopology, id: u32) -> Vec<String> {
    topology.adjacency_list(id)
        .iter()
        .filter_map(|&nid| topology.word_name(nid).map(|s| s.to_string()))
        .collect()
}

/// Calcola i vicini topologici di una parola tramite il simplicial complex.
///
/// Invece di usare le co-occorrenze statistiche (WordTopology), usa la
/// struttura cristallizzata dei pensieri (simplici) per determinare chi
/// è connesso a chi. Due parole sono vicine se abitano gli stessi pensieri.
///
/// Algoritmo:
///   1. Trova i top-3 frattali della parola sorgente (max affinità)
///   2. Per ogni frattale, recupera i simplici che lo contengono
///   3. In ogni simplesso, trova gli altri frattali (co-presenti)
///   4. Trova le parole con alta affinità per quei frattali
///   5. Score = persistenza × aff_sorgente × aff_destinazione
///   6. Fase = distanza angolare 8D tra le due firme (geometria pura)
fn simplex_based_neighbors(
    pf1_id_self: u32,
    affinities_a: &[f32; MAX_FRACTALS],  // affinità frattali della parola sorgente
    sig_a: &[f32; 8],                     // firma 8D della parola sorgente
    complex: &SimplicialComplex,
    fractal_simplex_cache: &HashMap<u32, Vec<u32>>,  // frattale → top-5 simplessi pre-ordinati
    fractal_word_map: &HashMap<u32, Vec<(u32, f32)>>,
    data: &[WordRecord],
) -> Vec<(u32, f32, f32)> {
    // Top-3 frattali della parola sorgente — i suoi "dendriti" principali
    let mut top_frac: Vec<(u32, f32)> = affinities_a.iter().enumerate()
        .filter(|(_, &a)| a > 0.15)
        .map(|(idx, &a)| (idx as u32, a))
        .collect();
    top_frac.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    top_frac.truncate(3);

    if top_frac.is_empty() {
        return Vec::new();
    }

    let mut candidates: HashMap<u32, f32> = HashMap::new();

    for (src_fid, src_aff) in &top_frac {
        // Usa cache pre-computata: top-5 simplessi per persistenza (zero sort, zero HashMap lookup)
        let sids = match fractal_simplex_cache.get(src_fid) {
            Some(v) => v.as_slice(),
            None => continue,
        };

        for &sid in sids {
            let simplex = match complex.get(sid) {
                Some(s) => s,
                None => continue,
            };
            let persistence = simplex.persistence as f32;

            // Per ogni altro frattale nello stesso simplesso
            for &other_fid in &simplex.vertices {
                if other_fid == *src_fid { continue; }

                // Trova parole vicine tramite quel frattale
                if let Some(words) = fractal_word_map.get(&other_fid) {
                    for &(candidate_id, dst_aff) in words.iter().take(5) {
                        if candidate_id == pf1_id_self { continue; }
                        // Score = persistenza × affinità_sorgente × affinità_destinazione
                        let score = persistence * src_aff * dst_aff;
                        let entry = candidates.entry(candidate_id).or_insert(0.0);
                        *entry = entry.max(score);
                    }
                }
            }
        }
    }

    // Calcola la fase come distanza angolare 8D tra le firme (geometria pura, non statistica)
    let mut result: Vec<(u32, f32, f32)> = candidates.into_iter()
        .filter_map(|(cand_id, weight)| {
            if cand_id as usize >= data.len() { return None; }
            let sig_b = &data[cand_id as usize].signature;

            // Phase = arccos(dot(sig_a, sig_b) / (|sig_a| × |sig_b|))
            let dot: f32 = sig_a.iter().zip(sig_b.iter()).map(|(a, b)| a * b).sum();
            let ma: f32 = sig_a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let mb: f32 = sig_b.iter().map(|x| x * x).sum::<f32>().sqrt();
            let phase = if ma > 0.0 && mb > 0.0 {
                (dot / (ma * mb)).clamp(-1.0, 1.0).acos()
            } else {
                std::f32::consts::FRAC_PI_2
            };

            Some((cand_id, weight, phase))
        })
        .collect();

    result.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result.truncate(MAX_NEIGHBORS);
    result
}

// ═══════════════════════════════════════════════════════════════════════
// TEST
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_size() {
        assert_eq!(
            std::mem::size_of::<WordRecord>(),
            RECORD_SIZE,
            "WordRecord deve essere esattamente {} byte", RECORD_SIZE
        );
    }

    #[test]
    fn test_record_size_header() {
        // Header deve essere multiplo di 8 per allineamento
        assert_eq!(HEADER_SIZE % 8, 0);
    }

    #[test]
    fn test_activation_state_creation() {
        let state = ActivationState::new(100);
        assert_eq!(state.activations.len(), 100);
        assert_eq!(state.active_count(), 0);
        assert_eq!(state.field_energy(), 0.0);
    }

    #[test]
    fn test_activate_and_decay() {
        let mut state = ActivationState::new(10);
        state.activate(0, 0.8);
        assert!(state.activations[0] > 0.7);

        state.decay_all(0.15);
        assert!(state.activations[0] < 0.8);
        assert!(state.activations[0] > 0.0);
    }

    #[test]
    fn test_build_from_lexicon() {
        use crate::topology::lexicon::Lexicon;
        use crate::topology::word_topology::WordTopology;

        let lexicon = Lexicon::bootstrap();
        let topology = WordTopology::build_from_lexicon(&lexicon);
        let field = PrometeoField::build_from_lexicon(&lexicon, &topology, None);

        assert_eq!(field.word_count, lexicon.word_count() as u32);

        // Ogni parola deve essere recuperabile per nome
        let first_word = field.word_name(0).to_string();
        let found_id = field.word_id(&first_word);
        assert!(found_id.is_some(), "parola '{}' non trovata nell'indice", first_word);
        assert_eq!(found_id.unwrap(), 0);
    }

    #[test]
    fn test_propagation_efficiency() {
        use crate::topology::lexicon::Lexicon;
        use crate::topology::word_topology::WordTopology;

        let lexicon = Lexicon::bootstrap();
        let topology = WordTopology::build_from_lexicon(&lexicon);
        let field = PrometeoField::build_from_lexicon(&lexicon, &topology, None);

        let mut state = ActivationState::new(field.word_count as usize);

        // Attiva solo 3 parole su tutto il lessico
        state.activate_by_name(&field, "gioia", 0.8);
        state.activate_by_name(&field, "calma", 0.6);
        state.activate_by_name(&field, "tempo", 0.5);

        let active_before = state.active_count();

        // La propagazione deve operare SOLO sul fronte attivo
        state.propagate(&field);

        // Dopo propagazione: potenzialmente qualche parola in più attivata
        // ma la maggior parte del lessico rimane a 0
        let total = field.word_count as usize;
        let active_after = state.active_count();
        let inactive = total - active_after;

        assert!(
            inactive > total / 2,
            "La maggior parte delle parole ({}) deve restare inattiva dopo propagazione (attive: {})",
            inactive, active_after
        );

        println!("Lessico: {}, Attive prima: {}, dopo: {}",
            total, active_before, active_after);
    }

    #[test]
    fn test_emerge_fractals() {
        use crate::topology::lexicon::Lexicon;
        use crate::topology::word_topology::WordTopology;

        let lexicon = Lexicon::bootstrap();
        let topology = WordTopology::build_from_lexicon(&lexicon);
        let field = PrometeoField::build_from_lexicon(&lexicon, &topology, None);

        let mut state = ActivationState::new(field.word_count as usize);
        state.activate_by_name(&field, "qui", 0.8);
        state.activate_by_name(&field, "vicino", 0.6);
        state.activate_by_name(&field, "lontano", 0.5);

        let fractal_acts = state.emerge_fractal_activations(&field);

        // Deve emergere almeno un frattale attivo
        let active_fractals = fractal_acts.iter().filter(|&&a| a > 0.03).count();
        assert!(active_fractals > 0, "Almeno un frattale deve emergere");

        println!("Attivazioni frattali: {:?}", fractal_acts);
    }

    #[test]
    fn test_hebbian_ltp() {
        use crate::topology::lexicon::Lexicon;
        use crate::topology::word_topology::WordTopology;

        let lexicon = Lexicon::bootstrap();
        let topology = WordTopology::build_from_lexicon(&lexicon);
        let field = PrometeoField::build_from_lexicon(&lexicon, &topology, None);

        let n = field.word_count as usize;
        let mut state = ActivationState::new(n);
        state.init_synapse_weights_from_field(&field);

        // Struttura sempre corretta indipendentemente dagli archi
        assert_eq!(state.synapse_weights.len(), n * MAX_NEIGHBORS,
            "synapse_weights deve avere dimensione word_count × MAX_NEIGHBORS");

        // Il lessico bootstrap può non avere archi — test directo sotto
        let maybe_src = (0..field.word_count).find(|&id| {
            field.record(id).neighbor_count > 0
        });
        if let Some(src_id) = maybe_src {
            let neighbor_id = field.record(src_id).neighbors[0];
            let initial_weight = state.synapse_weights[src_id as usize * MAX_NEIGHBORS];
            for _ in 0..10 {
                state.activate(src_id, 0.8);
                state.activate(neighbor_id, 0.6);
                state.hebbian_update(&field);
            }
            let final_weight = state.synapse_weights[src_id as usize * MAX_NEIGHBORS];
            assert!(final_weight > initial_weight,
                "LTP: peso deve crescere ({} → {})", initial_weight, final_weight);
        }
    }

    /// LTP con campo sintetico: verifica il meccanismo hebbiano direttamente.
    #[test]
    fn test_hebbian_ltp_direct() {
        let mut field = PrometeoField::empty();

        // Parola 0 connessa a parola 1, peso 0.5, fase 0 (piena risonanza)
        let mut r0 = WordRecord::empty();
        r0.word[..3].copy_from_slice(b"aaa"); r0.word_len = 3;
        r0.neighbor_count = 1;
        r0.neighbors[0] = 1;
        r0.neighbor_weights[0] = 0.5;
        r0.neighbor_phases[0] = 0.0;

        let mut r1 = WordRecord::empty();
        r1.word[..3].copy_from_slice(b"bbb"); r1.word_len = 3;

        let mut r2 = WordRecord::empty();
        r2.word[..3].copy_from_slice(b"ccc"); r2.word_len = 3;

        field.add_word("aaa", r0);
        field.add_word("bbb", r1);
        field.add_word("ccc", r2);

        let mut state = ActivationState::new(3);
        state.init_synapse_weights_from_field(&field);

        // Il peso iniziale in RAM deve matchare il basale ROM
        assert!((state.synapse_weights[0] - 0.5).abs() < 0.001,
            "init: peso iniziale deve essere 0.5, trovato {}", state.synapse_weights[0]);

        // Co-attivazione ripetuta: LTP deve aumentare il peso sinaptico
        for _ in 0..5 {
            state.activate(0, 0.8);
            state.activate(1, 0.7);
            state.hebbian_update(&field);
        }

        assert!(state.synapse_weights[0] > 0.5,
            "LTP: peso deve crescere da 0.5, trovato {}", state.synapse_weights[0]);

        // Il record ROM non deve essere modificato (la ROM è immutabile)
        assert!((field.record(0).neighbor_weights[0] - 0.5).abs() < 0.001,
            "ROM immutabile: neighbor_weights[0] deve restare 0.5");
    }

    #[test]
    fn test_decay_preserves_activation() {
        let mut state = ActivationState::new(10);
        state.synapse_weights = vec![1.0; 10 * MAX_NEIGHBORS];
        state.activate(0, 1.0);
        state.activate(1, 0.5);

        state.decay(0.85);

        // Le attivazioni devono essere scalate del 15%
        assert!((state.activations[0] - 0.85).abs() < 0.01,
            "decay(0.85) su 1.0 deve dare ~0.85, trovato {}", state.activations[0]);
        assert!((state.activations[1] - 0.425).abs() < 0.01,
            "decay(0.85) su 0.5 deve dare ~0.425, trovato {}", state.activations[1]);
        assert_eq!(state.activations[2], 0.0, "parole inattive rimangono 0.0");
    }

    #[test]
    fn test_save_and_load() {
        use crate::topology::lexicon::Lexicon;
        use crate::topology::word_topology::WordTopology;
        use std::path::PathBuf;

        let lexicon = Lexicon::bootstrap();
        let topology = WordTopology::build_from_lexicon(&lexicon);
        let field = PrometeoField::build_from_lexicon(&lexicon, &topology, None);

        let path = PathBuf::from("test_pf1_temp.bin");

        // Salva
        field.save_to_file(&path).expect("salvataggio fallito");

        // Ricarica
        let loaded = PrometeoField::load_from_file(&path).expect("caricamento fallito");
        assert_eq!(loaded.word_count, field.word_count);

        // Pulizia
        let _ = std::fs::remove_file(&path);
    }
}
