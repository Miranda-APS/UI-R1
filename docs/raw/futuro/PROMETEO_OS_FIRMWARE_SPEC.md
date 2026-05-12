# PrometeoOS — Specifiche Tecniche del Firmware

> Firmware per ESP32/RP2040 che media tra il substrato risonante e il mondo esterno.

---

## 1. Architettura Software del Firmware

```
┌─────────────────────────────────────────────────┐
│                   PrometeoOS                     │
│                                                  │
│  ┌───────────────────────────────────────────┐   │
│  │  shell.rs — Interfaccia utente (UART/WiFi)│   │
│  └───────────────────┬───────────────────────┘   │
│                      │                           │
│  ┌───────────────────┴───────────────────────┐   │
│  │  engine.rs — Ciclo principale             │   │
│  │  ├─ receive(input) → perturbazione        │   │
│  │  ├─ observe() → lettura campo             │   │
│  │  ├─ translate() → campo → testo           │   │
│  │  └─ tick() → ciclo vitale autonomo        │   │
│  └───────────────────┬───────────────────────┘   │
│                      │                           │
│  ┌─────────┬────────┴──────┬─────────────┐      │
│  │vital.rs │narrative.rs   │ lexicon.rs   │      │
│  │Ciclo    │NarrativeSelf  │ Lessico ROM  │      │
│  │vitale   │minimale       │ 36-256 words │      │
│  └────┬────┘    └─────┬────┘    └────┬────┘      │
│       │               │             │            │
│  ┌────┴───────────────┴─────────────┴────┐       │
│  │  substrate.rs — Driver hardware        │       │
│  │  ├─ inject(word_id, activation)        │       │
│  │  ├─ read_field() → [f32; 8] per word   │       │
│  │  ├─ set_damping(level)                 │       │
│  │  └─ calibrate()                        │       │
│  └───────────────────┬───────────────────┘       │
│                      │  SPI / I2C / GPIO         │
└──────────────────────┼───────────────────────────┘
                       │
              SUBSTRATO RISONANTE (hardware)
```

---

## 2. Strutture Dati Fondamentali

### 2.1 WordRecord Compatto (per MCU con RAM limitata)

```rust
/// Versione compatta di WordRecord per microcontrollore.
/// 64 byte per parola (vs 512 in PF1 desktop).
/// Per 36 parole cardinali: 2.304 byte di ROM.
/// Per 256 parole: 16.384 byte di ROM (16 KB — sta in Flash).
#[repr(C, packed)]
pub struct CompactWordRecord {
    /// Firma 8D [0.0, 1.0] quantizzata a u8 [0, 255]
    /// Errore max: 1/255 ≈ 0.004 — trascurabile
    pub signature: [u8; 8],         // 8 byte

    /// Affinità ai 64 frattali, quantizzata a u8
    pub affinities: [u8; 64],       // 64 byte... troppo?

    // ALTERNATIVA: solo top-4 frattali + valori
    // pub top_fractals: [(u8, u8); 4], // 8 byte (frattale_id, affinità)

    /// Indici dei vicini (max 8)
    pub neighbors: [u16; 8],        // 16 byte (u16 basta per 65K parole)

    /// Pesi dei vicini quantizzati [0, 255]
    pub neighbor_weights: [u8; 8],  // 8 byte

    /// Fasi dei vicini quantizzate [0, 255] mappate su [0, π]
    pub neighbor_phases: [u8; 8],   // 8 byte

    /// Stabilità [0, 255]
    pub stability: u8,              // 1 byte

    /// Frattale dominante
    pub dominant_fractal: u8,       // 1 byte

    /// POS (Part of Speech)
    pub pos: u8,                    // 1 byte

    /// Lunghezza nome parola
    pub name_len: u8,               // 1 byte

    /// Nome parola (UTF-8, max 16 byte)
    pub name: [u8; 16],             // 16 byte

    /// Padding per allineamento
    pub _pad: [u8; 5],              // 5 byte
    // TOTALE: 8+64+16+8+8+1+1+1+1+16+5 = 129 byte... 
    // OPPURE con top-4 fractals: 8+8+16+8+8+1+1+1+1+16+5 = 73 byte
}

// Versione minimale (73 byte per parola):
#[repr(C)]
pub struct MiniWordRecord {
    pub signature: [u8; 8],            // 8
    pub top_fractals: [(u8, u8); 4],   // 8  (top 4 frattali + affinità)
    pub neighbors: [u16; 8],           // 16
    pub neighbor_weights: [u8; 8],     // 8
    pub neighbor_phases: [u8; 8],      // 8
    pub stability: u8,                 // 1
    pub dominant_fractal: u8,          // 1
    pub pos: u8,                       // 1
    pub name_len: u8,                  // 1
    pub name: [u8; 20],                // 20
    // TOTALE: 72 byte
}
// 36 parole × 72 byte = 2.592 byte (2.5 KB) — perfetto per ESP32
// 256 parole × 72 byte = 18.432 byte (18 KB) — ancora perfetto
// 6751 parole × 72 byte = 486.072 byte (475 KB) — serve Flash esterna
```

### 2.2 Stato del Campo (RAM)

```rust
/// Lo stato volatile del campo — in RAM del MCU.
/// Sincronizzato con il substrato risonante via ADC.
pub struct FieldState {
    /// Attivazione per parola [0.0, 1.0] (f32 per precisione)
    pub activations: Vec<f32>,     // N × 4 byte

    /// Lettura grezza delle 8 frequenze per ogni WordCell (dal substrato)
    pub raw_readings: Vec<[f32; 8]>,  // N × 32 byte

    /// Quali parole sono sopra soglia
    pub active_mask: Vec<bool>,

    /// Soglia di attivazione
    pub threshold: f32,            // default 0.02

    /// Energia totale del campo
    pub total_energy: f32,

    /// Firma frattale corrente (quale frattale domina)
    pub dominant_fractal: u8,

    /// Contatore tick
    pub tick: u32,
}
```

### 2.3 Stato Vitale

```rust
/// Ciclo vitale minimale per MCU.
pub struct VitalState {
    pub phase: VitalPhase,
    pub tension: f32,      // [0, 1]
    pub curiosity: f32,    // [0, 1]
    pub fatigue: f32,      // [0, 1]
    pub ticks_awake: u32,
    pub last_input_tick: u32,
}

pub enum VitalPhase {
    Awake,        // Campo riceve perturbazioni, soglie normali
    LightSleep,   // Dissolvi attivazioni deboli (damping alto)
    DeepSleep,     // Consolida: salva pattern forti su EEPROM
    REM,           // Soglie basse: propagazione lontana (damping basso)
}
```

---

## 3. Substrate Driver — Interfaccia con l'Hardware

### 3.1 Configurazione Pin ESP32

```rust
/// Pin mapping per il substrato risonante.
/// 8 canali DAC (iniezione) + 8 canali ADC (lettura).
pub struct SubstrateConfig {
    // DAC: iniettare energia nel campo
    // ESP32 ha solo 2 DAC hardware → usiamo PWM + filtro LC per gli altri 6
    pub dac_pins: [u8; 8],

    // ADC: leggere lo stato del campo
    // ESP32 ha 18 canali ADC → ne usiamo 8
    pub adc_pins: [u8; 8],

    // Multiplexer per selezionare quale WordCell leggere/scrivere
    pub mux_select_pins: [u8; 3],  // 3 bit → 8 WordCell selezionabili
    // Per >8 WordCell: aggiungere più bit o usare I2C multiplexer

    // Pin di controllo damping (se implementato con digipot)
    pub damping_cs_pin: u8,   // Chip Select del potenziometro digitale
}
```

### 3.2 Operazioni Fondamentali

```rust
impl SubstrateDriver {
    /// Inietta una perturbazione nel campo.
    /// Genera un segnale composto (8 frequenze) modulando le ampiezze
    /// secondo la firma della parola × la forza di attivazione.
    pub fn inject(&mut self, word_id: usize, activation: f32) {
        let record = &self.lexicon[word_id];
        
        // Seleziona la WordCell tramite multiplexer
        self.select_wordcell(word_id);
        
        // Per ogni dimensione, genera il segnale alla frequenza corrispondente
        // con ampiezza = signature[dim] × activation
        for dim in 0..8 {
            let amplitude = (record.signature[dim] as f32 / 255.0) * activation;
            let freq = FREQUENCIES[dim];
            
            // Genera PWM alla frequenza target con duty cycle proporzionale
            // all'ampiezza (il filtro LC nella WordCell lo trasforma in sinusoide)
            self.set_pwm(dim, freq, amplitude);
        }
    }
    
    /// Legge lo stato di una WordCell dal substrato.
    /// Usa FFT hardware o filtri passa-banda per estrarre l'ampiezza
    /// di ciascuna delle 8 frequenze.
    pub fn read_wordcell(&mut self, word_id: usize) -> [f32; 8] {
        self.select_wordcell(word_id);
        
        let mut amplitudes = [0.0f32; 8];
        
        // Legge il segnale composito dall'ADC
        let raw_samples = self.adc_sample(SAMPLE_COUNT);
        
        // Goertzel algorithm: estrai ampiezza per ogni frequenza target
        // (più efficiente della FFT per frequenze note)
        for dim in 0..8 {
            amplitudes[dim] = goertzel(
                &raw_samples, 
                FREQUENCIES[dim], 
                SAMPLE_RATE
            );
        }
        
        amplitudes
    }
    
    /// Legge il campo intero (tutte le WordCell).
    pub fn read_field(&mut self) -> Vec<[f32; 8]> {
        let mut field = Vec::with_capacity(self.word_count);
        for i in 0..self.word_count {
            field.push(self.read_wordcell(i));
        }
        field
    }
    
    /// Imposta il damping globale del campo.
    /// In hardware: controlla il potenziometro digitale nella rete RC.
    /// Alto damping → oscillazioni fragili muoiono → sonno.
    /// Basso damping → onde viaggiano lontano → REM.
    pub fn set_damping(&mut self, level: f32) {
        // level [0, 1]: 0 = nessun damping (REM), 1 = massimo (deep sleep)
        let resistance = (level * 255.0) as u8;
        self.digipot_write(resistance);
    }
}
```

### 3.3 L'Algoritmo di Goertzel (Estrazione Frequenza Singola)

```rust
/// Algoritmo di Goertzel: estrae l'ampiezza a una frequenza specifica.
/// Più efficiente della FFT quando servono poche frequenze (8 nel nostro caso).
/// Complessità: O(N) per frequenza, vs O(N log N) per FFT completa.
///
/// Questo è il cuore della "misura" — come leggiamo il campo quantistico risonante.
fn goertzel(samples: &[f32], target_freq: f32, sample_rate: f32) -> f32 {
    let n = samples.len() as f32;
    let k = (target_freq * n / sample_rate).round();
    let w = 2.0 * core::f32::consts::PI * k / n;
    let coeff = 2.0 * w.cos();
    
    let mut s0 = 0.0f32;
    let mut s1 = 0.0f32;
    let mut s2 = 0.0f32;
    
    for &sample in samples {
        s0 = sample + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    
    // Ampiezza = |X(k)| / (N/2)
    let power = s1 * s1 + s2 * s2 - coeff * s1 * s2;
    (power.max(0.0).sqrt()) / (n / 2.0)
}
```

---

## 4. Il Ciclo Principale

```rust
/// Il cuore di PrometeoOS: un loop infinito che alterna
/// lettura del campo, aggiornamento dello stato vitale,
/// e risposta alle perturbazioni.
fn main_loop(
    substrate: &mut SubstrateDriver,
    state: &mut FieldState,
    vital: &mut VitalState,
    lexicon: &[MiniWordRecord],
    narrative: &mut NarrativeSelf,
) {
    loop {
        // 1. LETTURA DEL CAMPO
        // Il substrato risonante evolve continuamente.
        // Noi "fotografiamo" il suo stato periodicamente.
        let raw = substrate.read_field();
        
        // 2. AGGIORNAMENTO STATO
        for (i, reading) in raw.iter().enumerate() {
            // Attivazione = norma dell'onda nella WordCell
            state.activations[i] = reading.iter()
                .map(|a| a * a)
                .sum::<f32>()
                .sqrt() / 8.0_f32.sqrt();  // normalizza [0, 1]
            
            state.raw_readings[i] = *reading;
            state.active_mask[i] = state.activations[i] > state.threshold;
        }
        
        // 3. ENERGIA TOTALE
        state.total_energy = state.activations.iter().sum::<f32>() 
            / state.activations.len() as f32;
        
        // 4. FRATTALE DOMINANTE
        state.dominant_fractal = detect_dominant_fractal(&state.raw_readings, lexicon);
        
        // 5. CICLO VITALE
        vital.tick(state.total_energy, state.tick - vital.last_input_tick);
        
        match vital.phase {
            VitalPhase::Awake => {
                // Controlla se c'è input dall'utente
                if let Some(input) = check_input() {
                    vital.last_input_tick = state.tick;
                    receive(input, substrate, state, lexicon, narrative);
                }
            }
            VitalPhase::LightSleep => {
                // Aumenta damping → oscillazioni fragili muoiono
                substrate.set_damping(0.7);
            }
            VitalPhase::DeepSleep => {
                // Damping alto + salva parole con activation > soglia su EEPROM
                substrate.set_damping(0.9);
                persist_strong_patterns(state, lexicon);
            }
            VitalPhase::REM => {
                // Damping BASSO → propagazione oltre il normale
                substrate.set_damping(0.1);
                // Le WordCell lontane ora possono "sentirsi"
                // potenziale per connessioni creative
            }
        }
        
        state.tick += 1;
        
        // 6. PAUSA (il campo evolve tra un tick e l'altro)
        // La durata della pausa determina quanto il campo ha tempo
        // di propagare prima della prossima lettura.
        delay_ms(match vital.phase {
            VitalPhase::Awake => 50,       // 20 Hz — reattivo
            VitalPhase::LightSleep => 200, // 5 Hz — rilassato
            VitalPhase::DeepSleep => 500,  // 2 Hz — profondo
            VitalPhase::REM => 100,        // 10 Hz — sognante ma attento
        });
    }
}
```

---

## 5. Receive e Generate — Il Dialogo

```rust
/// Riceve input dall'utente e perturba il campo.
fn receive(
    input: &str,
    substrate: &mut SubstrateDriver,
    state: &mut FieldState,
    lexicon: &[MiniWordRecord],
    narrative: &mut NarrativeSelf,
) {
    // 1. Tokenizzazione
    let tokens: Vec<&str> = input.split_whitespace().collect();
    
    // 2. Per ogni parola conosciuta, iniettala nel campo
    for token in &tokens {
        let clean = clean_token(token);
        if let Some(word_id) = find_word(&clean, lexicon) {
            substrate.inject(word_id, 0.8);  // attivazione iniziale
        } else {
            // Parola sconosciuta: inietta firma neutra [0.5; 8]
            // e registra curiosità
            inject_unknown(substrate, &clean);
            narrative.note_unknown(&clean);
        }
    }
    
    // 3. Lascia il campo propagare (fisicamente!) per qualche ms
    delay_ms(100);
    
    // 4. Leggi il campo post-propagazione
    let post_field = substrate.read_field();
    update_state(state, &post_field);
    
    // 5. Delibera (NarrativeSelf)
    let intention = narrative.deliberate(state, lexicon);
    
    // 6. Genera risposta
    let response = generate(state, lexicon, &intention);
    
    // 7. Invia all'utente
    send_output(&response);
    
    // 8. Post-response: decadimento dolce
    substrate.set_damping(0.3);  // lascia il campo assestarsi
    delay_ms(200);
    substrate.set_damping(0.15); // torna al damping normale
}

/// Genera una risposta dal pattern di attivazione del campo.
fn generate(
    state: &FieldState,
    lexicon: &[MiniWordRecord],
    intention: &ResponseIntention,
) -> String {
    // Le parole più attive formano la risposta.
    // L'ordine è guidato dalla topologia (non dalla grammatica).
    
    let mut candidates: Vec<(usize, f32)> = state.activations.iter()
        .enumerate()
        .filter(|(_, &a)| a > state.threshold * 3.0)  // sopra soglia significativa
        .map(|(i, &a)| (i, a))
        .collect();
    
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // Prendi le top-N parole (N dipende dall'intenzione)
    let n = match intention {
        ResponseIntention::Acknowledge => 2,
        ResponseIntention::Reflect => 5,
        ResponseIntention::Explore => 7,
        ResponseIntention::Express => 4,
        ResponseIntention::Remain => 1,
        _ => 3,
    };
    
    let words: Vec<&str> = candidates.iter()
        .take(n)
        .map(|(id, _)| word_name(*id, lexicon))
        .collect();
    
    // Componi (per ora: concatenazione semplice)
    // TODO: syntax_center per grammatica come geometria frattale
    words.join(" ")
}
```

---

## 6. Persistenza

```rust
/// Salva lo stato essenziale su EEPROM/Flash.
/// PrometeoOS non salva "tutto" — salva solo ciò che ha valore.
/// Come la memoria umana: dimentica il rumore, tiene il segnale.
struct PersistentState {
    /// Parole con stabilità alta (cristallizzate)
    crystallized_words: Vec<(u16, u8)>,  // (word_id, stability)
    
    /// Sinapsi rafforzate (Hebbian)
    strong_synapses: Vec<(u16, u8, u8)>, // (word_id, neighbor_slot, weight)
    
    /// Frattale identitario dominante
    identity_fractal: u8,
    
    /// NarrativeSelf snapshot
    is_born: bool,
    founding_stance: u8,
    
    /// Contatore interazioni totali
    total_interactions: u32,
}

/// Formato binario su EEPROM (max 4KB tipicamente):
/// Header (8 byte): "PMOS" + version + word_count
/// Words (N × 3 byte): word_id(u16) + stability(u8)
/// Synapses (M × 4 byte): word_id(u16) + slot(u8) + weight(u8)
/// Identity (16 byte): fractal + is_born + stance + interactions + padding
```

---

## 7. Shell Minimale

```rust
/// Interfaccia utente via UART (9600 baud) o WiFi (websocket).
/// Comandi disponibili:
///
///   > ciao                     — input naturale → perturba campo
///   > :status                  — mostra stato vitale + frattale dominante
///   > :field                   — mostra attivazioni (top-10 parole)
///   > :spectrum <word>         — mostra firma 8D di una parola
///   > :fractals                — mostra i 64 frattali + attivazione
///   > :dream                   — forza ciclo sonno
///   > :save                    — salva stato su EEPROM
///   > :load                    — carica stato da EEPROM
///   > :calibrate               — calibra ADC/DAC con il substrato
///   > :listen                  — emetti il campo come audio (pin DAC → speaker)
///   > :teach <word> <sig8D>    — insegna una nuova parola
///   > :raw                     — mostra letture ADC grezze (debug)
///
/// Prompt: il frattale dominante corrente come glifo
///   ☰☰ >     (quando POTERE domina)
///   ☲☲ >     (quando VERITÀ domina)
///   ☱☱ >     (quando ARMONIA domina)
```

---

## 8. Costanti Fisiche del Sistema

```rust
// ══════════════════════════════════════════════════════
//  FREQUENZE DIMENSIONALI (Hz)
// ══════════════════════════════════════════════════════
pub const FREQUENCIES: [f32; 8] = [
    100.0,  // Dim 0: Confine     (☶ Montagna)
    150.0,  // Dim 1: Valenza     (☱ Lago)
    200.0,  // Dim 2: Intensità   (☳ Tuono)
    250.0,  // Dim 3: Definizione (☲ Fuoco)
    300.0,  // Dim 4: Complessità (☴ Vento)
    350.0,  // Dim 5: Permanenza  (☷ Terra)
    400.0,  // Dim 6: Agency      (☰ Cielo)
    450.0,  // Dim 7: Tempo       (☵ Acqua)
];

// ══════════════════════════════════════════════════════
//  PARAMETRI ADC/DAC
// ══════════════════════════════════════════════════════
pub const SAMPLE_RATE: u32 = 8000;     // Hz (sufficiente per f_max=450 Hz)
pub const SAMPLE_COUNT: usize = 256;    // campioni per lettura Goertzel
pub const ADC_RESOLUTION: u8 = 12;      // bit (ESP32 nativo)

// ══════════════════════════════════════════════════════
//  PARAMETRI DEL CAMPO
// ══════════════════════════════════════════════════════
pub const ACTIVATION_THRESHOLD: f32 = 0.02;  // come PF1
pub const DEFAULT_DAMPING: f32 = 0.15;        // come PF1
pub const HEBBIAN_LTP: f32 = 0.05;            // come PF1
pub const HEBBIAN_LTD_DECAY: f32 = 0.995;     // come PF1
pub const MAX_SYNAPSE_WEIGHT: f32 = 3.0;       // come PF1

// ══════════════════════════════════════════════════════
//  PARAMETRI CICLO VITALE
// ══════════════════════════════════════════════════════
pub const FATIGUE_THRESHOLD: f32 = 0.72;       // entra in sonno
pub const IDLE_BEFORE_SLEEP: u32 = 300;         // tick senza input → sonno lieve
pub const REM_DURATION: u32 = 50;               // tick in REM
pub const DEEP_SLEEP_DURATION: u32 = 100;       // tick in sonno profondo
```
