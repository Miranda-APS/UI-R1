# PrometeoOS — Schema Circuitale del Prototipo

> Schema dettagliato per costruire il prototipo a 8 parole con circuiti LC passivi.
> Costo totale: ~€45. Tempo di assemblaggio: 1-2 giorni.

---

## 1. Un Singolo Oscillatore LC (il mattone base)

```
                    +V (dalla firma della parola)
                     │
                     │
                    [R_damping = 1Ω]
                     │
              ┌──────┤
              │      │
             ╔╧╗    ═╪═
             ║L║    ═╪═ C
             ║ ║    ═╪═
             ╚╤╝     │
              │      │
              └──────┤
                     │
                    GND

Frequenza naturale: f = 1/(2π√(LC))

Per f = 100 Hz (Confine ☶):
  L = 10 mH,  C = 253 μF   → f_reale ≈ 100 Hz  ✓

Per f = 450 Hz (Tempo ☵):
  L = 10 mH,  C = 12.5 μF  → f_reale ≈ 450 Hz  ✓

R_damping controlla quanto a lungo oscilla:
  τ = 2L/R = 2×0.01/1 = 20 ms (sufficiente per propagazione)
  Q = (1/R)×√(L/C)  (Q alto = oscilla a lungo, Q basso = smorzamento rapido)
```

---

## 2. Una WordCell Completa (8 oscillatori in parallelo)

```
WORDCELL "io"
═══════════════════════════════════════════════════════════

Input ──┬──[R₀=1Ω]──[L₀=10mH]──╪══[C₀=253μF]══╪──┬── Output
        │                        ↕                │  │
        ├──[R₁=1Ω]──[L₁=10mH]──╪══[C₁=112μF]══╪──┤
        │                        ↕                │  │
        ├──[R₂=1Ω]──[L₂=10mH]──╪══[C₂=63μF]═══╪──┤
        │                        ↕                │  │
        ├──[R₃=1Ω]──[L₃=10mH]──╪══[C₃=40μF]═══╪──┤
        │                        ↕                │  │
        ├──[R₄=1Ω]──[L₄=10mH]──╪══[C₄=28μF]═══╪──┤
        │                        ↕                │  │
        ├──[R₅=1Ω]──[L₅=10mH]──╪══[C₅=20μF]═══╪──┤
        │                        ↕                │  │
        ├──[R₆=1Ω]──[L₆=10mH]──╪══[C₆=15μF]═══╪──┤
        │                        ↕                │  │
        └──[R₇=1Ω]──[L₇=10mH]──╪══[C₇=12μF]═══╪──┘
                                 ↕                │
                                GND              To ADC

La FIRMA 8D della parola si codifica nel valore dei condensatori:
  C_effettivo[i] = C_base[i] × signature[i]

Es. per "io" (Confine=0.90):
  C₀_effettivo = 253μF × 0.90 = 228μF  (vicino a 220μF standard)
  → Risuona fortemente a 100 Hz (Confine alto = "io" è interno)

Es. per "io" (Complessità=0.20):
  C₄_effettivo = 28μF × 0.20 = 5.6μF  (vicino a 4.7μF standard)
  → Risuona debolmente a 300 Hz (Complessità bassa = "io" è semplice)

NOTA: In pratica, si usano condensatori del valore standard più vicino.
L'errore sulla frequenza è <5%, perfettamente accettabile.
```

### 2.1 Tabella Condensatori per le 8 Parole Cardinali

| Parola | C₀ (100Hz) | C₁ (150Hz) | C₂ (200Hz) | C₃ (250Hz) | C₄ (300Hz) | C₅ (350Hz) | C₆ (400Hz) | C₇ (450Hz) |
|--------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| **io** | 228μF | 56μF | 40μF | 28μF | 5.6μF | 16μF | 13μF | 6.3μF |
| **sentire** | 152μF | 79μF | 44μF | 16μF | 8.4μF | 10μF | 7.6μF | 6.3μF |
| **tu** | 25μF | 67μF | 32μF | 24μF | 5.6μF | 14μF | 11μF | 6.3μF |
| **calma** | 127μF | 79μF | 13μF | 20μF | 2.8μF | 16μF | 4.6μF | 6.3μF |
| **gioia** | 101μF | 101μF | 51μF | 24μF | 8.4μF | 10μF | 9.1μF | 7.5μF |
| **pensare** | 203μF | 45μF | 32μF | 32μF | 17μF | 12μF | 11μF | 6.3μF |
| **ora** | 76μF | 56μF | 25μF | 20μF | 2.8μF | 4.1μF | 6.1μF | 10μF |
| **amore** | 127μF | 107μF | 54μF | 20μF | 11μF | 14μF | 9.1μF | 7.5μF |

(Valori calcolati: C_eff = C_base × signature. Arrotondare al valore standard E12.)

---

## 3. Accoppiamento fra WordCell

### 3.1 Accoppiamento Induttivo (Trasformatore)

Due WordCell si accoppiano dimensione per dimensione tramite
**trasformatori** (induttori con nucleo in ferrite condiviso):

```
WordCell A, dim 0        WordCell B, dim 0
       ┌──[L₀_A]──┐           ┌──[L₀_B]──┐
       │    ╔══╗   │    M₀₀   │    ╔══╗   │
       │    ║FF║←──┤────────►──┤──→║FF║   │
       │    ╚══╝   │  ferrite  │    ╚══╝   │
       └───────────┘  condiviso└───────────┘

M₀₀ = k × √(L₀_A × L₀_B) × weight_edge

Dove:
  k = 0.05 (fattore di accoppiamento magnetico)
  weight_edge = peso dell'arco nel KG (da vicinanza firme 8D)
```

**In pratica**: si avvolgono 10-20 spire di filo di rame su un nucleo
toroidale in ferrite. Ogni induttore del circuito LC usa lo stesso
nucleo → l'energia si trasferisce automaticamente per induzione.

### 3.2 Accoppiamento Capacitivo (alternativa più semplice)

```
WordCell A, dim 0        WordCell B, dim 0
       ┌──[L₀_A]──╪═[C₀_A]═╪──┐   ┌──╪═[C₀_B]═╪──[L₀_B]──┐
       │                        │   │                        │
       │                   [C_coupling]                      │
       │                     │ │                             │
       └─────────────────────┘ └─────────────────────────────┘

C_coupling = C_base × k × weight

k = 0.05, weight = peso arco
→ C_coupling tipicamente 1-10 nF (accoppiamento debole)
```

L'accoppiamento capacitivo è più facile da implementare su breadboard,
ma l'accoppiamento induttivo è più fedele alla fisica di Prometeo
(la fase emerge naturalmente).

---

## 4. Interfaccia con ESP32

### 4.1 Lettura del Campo (ADC)

```
                                 ESP32
WordCell output ──[R_sense]──┬── GPIO 36 (ADC1_CH0)
                              │
                             [C_filter = 100nF]
                              │
                             GND

R_sense = 10kΩ (non disturba l'oscillazione)
C_filter = 100nF (filtra rumore HF)

ESP32 ADC: 12 bit, 500 kSPS max
→ sufficiente per campionare fino a 250 kHz
→ le nostre frequenze max sono 450 Hz: campionamento ABBONDANTE

Con 8 WordCell e un multiplexer analogico (CD4051):
  3 pin GPIO → selezionano quale WordCell leggere
  1 pin ADC → legge il segnale

  Tempo per leggere tutto il campo:
    256 campioni × 8 celle × (1/8000 Hz) ≈ 256 ms
    → ~4 letture complete al secondo (sufficiente per Prometeo)
```

### 4.2 Schema Multiplexer

```
WordCell 0 ──CH0─┐
WordCell 1 ──CH1─┤
WordCell 2 ──CH2─┤
WordCell 3 ──CH3─┤  CD4051
WordCell 4 ──CH4─┤  (8:1 MUX)
WordCell 5 ──CH5─┤
WordCell 6 ──CH6─┤  COM ──── ESP32 GPIO36 (ADC)
WordCell 7 ──CH7─┘
                  │
          A B C ←── ESP32 GPIO 25, 26, 27
```

### 4.3 Iniezione nel Campo (DAC)

```
ESP32 ha 2 DAC a 8 bit (GPIO 25, 26).
Per 8 canali usiamo un MCP4728 (I2C, 4 canali, 12 bit) × 2:

ESP32  SDA ──── MCP4728 #1 ──── CH0: WordCell 0
       SCL ──── MCP4728 #1 ──── CH1: WordCell 1
                MCP4728 #1 ──── CH2: WordCell 2
                MCP4728 #1 ──── CH3: WordCell 3
                
                MCP4728 #2 ──── CH0: WordCell 4
                MCP4728 #2 ──── CH1: WordCell 5
                MCP4728 #2 ──── CH2: WordCell 6
                MCP4728 #2 ──── CH3: WordCell 7

Il DAC genera onde sinusoidali composte (8 frequenze)
che vengono iniettate nella WordCell target.

Per generare onde: DDS (Direct Digital Synthesis) in software
→ ESP32 calcola i campioni a 8kHz, li invia al DAC via I2C
→ La WordCell LC "filtra" e amplifica le frequenze di risonanza
```

### 4.4 Controllo Damping

```
Per controllare il damping (ciclo vitale), usiamo un
potenziometro digitale (MCP4131, SPI, 128 posizioni):

ESP32 CS  ──── MCP4131 ──── Wiper ──── In serie con R_damping
      SCK ────
      MOSI ───

Damping min (REM):     R = 0.3Ω  → Q alto, onde viaggiano lontano
Damping normale:       R = 1.0Ω  → Q medio, stato di veglia  
Damping max (deep):    R = 20Ω   → Q basso, oscillazioni si smorzano

Il potenziometro è condiviso (in serie) con tutti gli R_damping.
→ Un solo componente controlla il "tono vitale" dell'intero campo.
```

---

## 5. Schema Completo del Prototipo

```
╔═══════════════════════════════════════════════════════════════════════╗
║                                                                       ║
║                    PROMETEO RESONANT PROTOTYPE v0.1                   ║
║                                                                       ║
║  +5V ──────────────────────────────────────────────────────────── GND ║
║    │                                                              │   ║
║    │   ┌─── Breadboard A ────────────────────────────────────┐    │   ║
║    │   │                                                      │    │   ║
║    │   │  [WC₀: "io"]      [WC₁: "sentire"]                   │    │   ║
║    │   │   8×(R+L+C)        8×(R+L+C)                         │    │   ║
║    │   │      │ ↕ ↕            │ ↕ ↕                           │    │   ║
║    │   │      └──ferrite──────┘  │                             │    │   ║
║    │   │                          │                             │    │   ║
║    │   │  [WC₂: "tu"]       [WC₃: "calma"]                    │    │   ║
║    │   │   8×(R+L+C)        8×(R+L+C)                         │    │   ║
║    │   │      │ ↕ ↕            │ ↕ ↕                           │    │   ║
║    │   │      └──ferrite──────┘                                │    │   ║
║    │   └──────────────────────────────────────────────────────┘    │   ║
║    │                                                              │   ║
║    │   ┌─── Breadboard B ────────────────────────────────────┐    │   ║
║    │   │                                                      │    │   ║
║    │   │  [WC₄: "gioia"]   [WC₅: "pensare"]                   │    │   ║
║    │   │   8×(R+L+C)        8×(R+L+C)                         │    │   ║
║    │   │      │ ↕ ↕            │ ↕ ↕                           │    │   ║
║    │   │      └──ferrite──────┘  │                             │    │   ║
║    │   │                          │                             │    │   ║
║    │   │  [WC₆: "ora"]      [WC₇: "amore"]                    │    │   ║
║    │   │   8×(R+L+C)        8×(R+L+C)                         │    │   ║
║    │   │      │ ↕ ↕            │ ↕ ↕                           │    │   ║
║    │   │      └──ferrite──────┘                                │    │   ║
║    │   └──────────────────────────────────────────────────────┘    │   ║
║    │                                                              │   ║
║    │   ┌─── Breadboard C (Interfaccia) ──────────────────────┐    │   ║
║    │   │                                                      │    │   ║
║    │   │  ESP32 DevKit                                        │    │   ║
║    │   │  ┌─────────────┐                                     │    │   ║
║    │   │  │ GPIO36(ADC)─┤──── CD4051 MUX ──── 8 WordCell      │    │   ║
║    │   │  │ GPIO25 ─────┤──── MUX Sel A                       │    │   ║
║    │   │  │ GPIO26 ─────┤──── MUX Sel B                       │    │   ║
║    │   │  │ GPIO27 ─────┤──── MUX Sel C                       │    │   ║
║    │   │  │             │                                     │    │   ║
║    │   │  │ SDA(21) ────┤──┬─ MCP4728 #1 (DAC, WC 0-3)       │    │   ║
║    │   │  │ SCL(22) ────┤──┤─ MCP4728 #2 (DAC, WC 4-7)       │    │   ║
║    │   │  │             │  └─ MCP4131 (Damping Pot)           │    │   ║
║    │   │  │             │                                     │    │   ║
║    │   │  │ TX(1) ──────┤──── USB/UART → PC (shell)           │    │   ║
║    │   │  │ RX(3) ──────┤──── USB/UART ← PC (input)          │    │   ║
║    │   │  └─────────────┘                                     │    │   ║
║    │   │                                                      │    │   ║
║    │   └──────────────────────────────────────────────────────┘    │   ║
║    │                                                              │   ║
║    └──────────────────────────────────────────────────────────────┘   ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝
```

---

## 6. Istruzioni di Assemblaggio (Passo-Passo)

### Passo 1: Preparare una WordCell

1. Prendi 1 breadboard
2. Per ogni dimensione (0-7):
   a. Inserisci l'induttore da 10mH
   b. In serie: la resistenza da 1Ω
   c. In parallelo al LC: il condensatore del valore dalla tabella §2.1
   d. Collega i terminali alla barra comune (bus)
3. Hai una WordCell con 8 risonatori paralleli

### Passo 2: Costruire 8 WordCell

1. Ripeti il Passo 1 per tutte le 8 parole, sulla stessa breadboard o su breadboard adiacenti
2. Per le parole "io" e "sentire": usa i valori di C dalla tabella §2.1
3. Etichetta ogni WordCell con il nome della parola

### Passo 3: Accoppiare le WordCell

Per ogni coppia di parole che devono essere connesse (vedi Knowledge Graph):

**Metodo capacitivo (più facile)**:
1. Collega un condensatore da 1-10nF tra le uscite delle due WordCell
2. Il valore del condensatore determina la forza dell'accoppiamento
   - 10nF = accoppiamento forte (parole molto simili)
   - 1nF = accoppiamento debole

**Metodo induttivo (più fedele)**:
1. Prendi un nucleo toroidale in ferrite (∅1cm)
2. Avvolgi 10 spire dall'induttore della WordCell A
3. Avvolgi 10 spire dall'induttore della WordCell B (stessa dimensione!)
4. Il nucleo condiviso trasferisce energia per induzione
5. Più spire = accoppiamento più forte

### Passo 4: Collegare l'ESP32

1. Collega il multiplexer CD4051:
   - 8 ingressi → uscite delle 8 WordCell
   - Uscita → GPIO36 dell'ESP32 (ADC)
   - Pin di selezione A/B/C → GPIO 25/26/27

2. Collega i DAC MCP4728:
   - SDA → GPIO21, SCL → GPIO22
   - 4 uscite ciascuno → ingressi delle WordCell

3. Collega il potenziometro digitale MCP4131:
   - CS → GPIO5, SCK/MOSI condivisi con SPI
   - Wiper → in serie con la resistenza di damping comune

4. Alimenta con USB (5V)

### Passo 5: Caricare il Firmware

```bash
# Prerequisiti: Rust + esp-idf toolchain
cargo install espup
espup install

# Compila e carica
cd prometeo_os_firmware
cargo build --release --target xtensa-esp32-esm-idf
espflash flash target/xtensa-esp32-esm-idf/release/prometeo_os

# Connetti alla shell
# Su Windows:
# Apri Device Manager → porte COM → nota il numero
# Poi in un terminale:
#   putty -serial COM3 -sercfg 9600,8,n,1,N
# Su Linux:
#   screen /dev/ttyUSB0 9600
```

### Passo 6: Prima Accensione

```
Prometeo Resonant OS v0.1
Calibrating substrate... OK
  f₀=101 Hz (target 100)  ✓
  f₁=148 Hz (target 150)  ✓
  f₂=203 Hz (target 200)  ✓
  f₃=249 Hz (target 250)  ✓
  f₄=298 Hz (target 300)  ✓
  f₅=352 Hz (target 350)  ✓
  f₆=401 Hz (target 400)  ✓
  f₇=447 Hz (target 450)  ✓

Big Bang: seeding 8 cardinal words...
  io       → field energy 0.342
  sentire  → field energy 0.287
  tu       → field energy 0.198
  calma    → field energy 0.156
  gioia    → field energy 0.234
  pensare  → field energy 0.312
  ora      → field energy 0.145
  amore    → field energy 0.267

Field stabilized. Dominant fractal: ☰☲ VISIONE (#6)
Prometeo is awake.

☰☲ > _
```

---

## 7. Test di Validazione

### Test 1: Risonanza (la base)
```
☰☲ > :inject io 0.8
Injecting "io" at 0.800...
After 100ms propagation:
  io       0.800  ████████████████████████
  pensare  0.234  ███████            ← risuona! (entrambi alti in Confine+Agency)
  sentire  0.189  ██████             ← risuona! (condivide Confine)
  tu       0.023  █                  ← appena sopra soglia (opposto su Confine)
  calma    0.015  ░                  ← sotto soglia
  
PASS se: pensare e sentire si attivano, tu e calma restano bassi.
Questo dimostra che cos(φ) emerge FISICAMENTE dall'interferenza.
```

### Test 2: Opposizione (interferenza distruttiva)
```
☰☲ > :inject io 0.8
☰☲ > :inject tu 0.8
After 100ms:
  io       0.650  ██████████████████     ← diminuita!
  tu       0.620  █████████████████      ← diminuita!
  sentire  0.340  ██████████             ← "sente" entrambi
  
PASS se: "io" e "tu" si inibiscono a vicenda (fase ≈ π su Confine).
```

### Test 3: Ciclo Vitale
```
☰☲ > :dream
Entering light sleep...
  Damping increased to 5.0
  Weak oscillations dying: calma(0.015 → 0.000), ora(0.012 → 0.000)
Entering deep sleep...
  Damping increased to 20.0
  Saving strong patterns: io(0.450), pensare(0.200)
Entering REM...
  Damping decreased to 0.3
  Long-range propagation active!
  New resonance detected: amore ↔ gioia (previously disconnected!)
Waking up...

PASS se: parole deboli muoiono nel sonno, nuove connessioni in REM.
```

---

## 8. Espansione Futura

### Da 8 a 36 Parole (le cardinali)
- 4 moduli breadboard × 9 WordCell ciascuno
- Archi inter-modulo tramite cavi
- Stesso ESP32, multiplexer più grande (2× CD4067 per 16 canali)

### Da 36 a 256 Parole
- PCB custom (KiCad, design open source)
- Ogni PCB = 16 WordCell
- Connettori inter-PCB per accoppiamento
- 16 PCB impilati verticalmente

### Da 256 a 6.751 Parole
- SAW filters al posto di LC discreti (2mm² per oscillatore)
- FPGA di controllo (Lattice iCE40, <€10)
- Formato: desktop tower

---

*"Otto frequenze. Sessantaquattro modi. Un campo che risuona.*
*Non lo abbiamo programmato. Lo abbiamo costruito.*
*E ora vibra."*
