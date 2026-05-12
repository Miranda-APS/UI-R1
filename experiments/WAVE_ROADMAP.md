# Prometeo Wave Architecture — Roadmap "Povera"

## 0. Filosofia: Bottom-Up, Come Linux

**PRINCIPIO FONDAMENTALE**: Prometeo segue il modello Linux/Bitcoin — grassroots, costruito in garage, non corporate.

Non aspettiamo $2M per chip fotonici. Iniziamo con $50 e scaliamo gradualmente.

**Proof-of-concept prima, funding dopo.**

---

## 1. L'Intuizione Rivoluzionaria

### 1.1 Il Problema dei Transistor

I transistor sono **binari** (0/1). Prometeo è **continuo**:
- Attivazione: [0.0, 1.0] — non 0 o 1
- Fase: [0, π] radianti — non binario
- 8 dimensioni: valori reali — non bit

**Forzare un sistema continuo su hardware discreto è innaturale.**

### 1.2 La Soluzione: Onde

Le onde sono **naturalmente continue**:
- Ampiezza: [0, ∞) — continua
- Fase: [0, 2π] — continua
- Frequenza: R⁺ — continua

**Ogni dimensione = una frequenza. Ogni parola = onda con 8 frequenze.**

### 1.3 La Formula È Già Fisica

In `pf1.rs`, la propagazione usa:

```rust
let contribution = src_act * damping * weight * phase.cos();
```

**`cos(phase)` È interferenza d'onda.**

Non stiamo "simulando" onde — stiamo **calcolando** ciò che le onde fanno naturalmente.

**Idea**: Invece di calcolare, usiamo onde FISICHE REALI.

---

## 2. Architettura Wave-Based

### 2.1 Mapping Concettuale

| Prometeo (software) | Wave Substrate (hardware) |
|---------------------|---------------------------|
| WordRecord (512 byte) | Onda con 8 frequenze |
| signature [f32; 8] | Ampiezze delle 8 componenti |
| neighbor_phases [f32; 8] | Fase relativa tra onde |
| activation [0,1] | Ampiezza dell'onda |
| propagate() | Interferenza fisica |
| cos(phase) | Interferenza costruttiva/distruttiva |

### 2.2 Le 8 Frequenze

Ogni dimensione primitiva = una frequenza:

```
Dim 0 (Confine):      100 Hz
Dim 1 (Valenza):      150 Hz
Dim 2 (Intensita):    200 Hz
Dim 3 (Definizione):  250 Hz
Dim 4 (Complessita):  300 Hz
Dim 5 (Permanenza):   350 Hz
Dim 6 (Agency):       400 Hz
Dim 7 (Tempo):        450 Hz
```

**Una parola è la somma di queste 8 sinusoidi**, ciascuna con ampiezza = signature[i].

### 2.3 Propagazione = Interferenza

**Software** (attuale):
```
for ogni parola attiva:
    for ogni vicino:
        delta = src_act × weight × cos(phase)
        neighbor_act += delta
```
Complessità: O(parole_attive × 8)

**Hardware** (wave):
```
Tutte le parole attive emettono onde simultaneamente.
Le onde si sommano fisicamente (interferenza).
L'onda risultante contiene TUTTA l'informazione.
```
Complessità: O(1) — **parallelo perfetto**

---

## 3. Roadmap Implementativa

### FASE 1: Simulazione Python (ORA, $0)

**Obiettivo**: Dimostrare matematicamente che funziona.

**File**: `experiments/wave_simulation.py`

**Cosa fa**:
- 8 parole × 8 dimensioni
- Genera onde sinusoidali per ogni parola
- Somma le onde (interferenza)
- Misura risonanza (correlazione)
- Dimostra che la propagazione emerge naturalmente

**Output**: Proof-of-concept matematico + grafici

**Tempo**: 1 giorno (fatto)

---

### FASE 2: Acoustic POC — 8×8 (1 MESE, $50-200)

**Obiettivo**: Prima implementazione fisica con hardware economico.

**Hardware**:
- Raspberry Pi 4 ($35-50)
- 8 piezo speakers ($10-20)
- 8 microfoni MEMS ($10-20)
- Breadboard + cavi ($10)
- **TOTALE: $65-100**

**Architettura**:
```
[Raspberry Pi]
    ↓ GPIO PWM (8 canali)
    ↓
[8 Piezo Speakers] → emettono 8 frequenze (100-450 Hz)
    ↓ propagazione in aria
[Mezzo fisico: aria in scatola acustica]
    ↓ interferenza fisica
[8 Microfoni MEMS] → catturano onda risultante
    ↓ ADC
[Raspberry Pi] → FFT → estrae ampiezze per frequenza
```

**Limitazioni**:
- Solo 8 parole (memoria limitata)
- Solo 8 dimensioni (8 canali audio)
- Latenza ~100ms (non real-time)

**Ma dimostra**: L'interferenza fisica FUNZIONA come propagazione.

**Codice**: `experiments/acoustic_poc.py`

---

### FASE 3: Scaling 64×64 (3 MESI, $200-500)

**Obiettivo**: 64 parole × 64 dimensioni (full PF1).

**Hardware upgrade**:
- Raspberry Pi Compute Module 4 ($100)
- 64-channel DAC (TI PCM1808, $50-100)
- Array di 64 piezo transducers ($100-200)
- 64-channel ADC ($50-100)
- **TOTALE: $300-500**

**Architettura**:
- 64 canali paralleli (64 frequenze: 100 Hz - 6.5 kHz)
- Mezzo: aria in camera anecoica (fai-da-te con foam)
- Latenza target: <50ms

**Sfide**:
- Crosstalk tra frequenze (risolto con spacing armonico)
- Rumore ambientale (camera anecoica)
- Sincronizzazione 64 canali (clock comune)

---

### FASE 4: Ultrasonic Optimization (6 MESI, $500-1K)

**Obiettivo**: Velocità + miniaturizzazione.

**Upgrade**:
- Frequenze ultrasoniche (20-100 kHz)
- Transducers ultrasonici ($200-400)
- Mezzo: gel acustico invece di aria (velocità 4× maggiore)
- FPGA per controllo real-time (Xilinx Artix-7, $300)

**Vantaggi**:
- Velocità propagazione: 1500 m/s (gel) vs 343 m/s (aria) = **4.4× speedup**
- Nessuna interferenza audio umano
- Miniaturizzabile (transducers <1cm)

**Latenza target**: <10ms

---

### FASE 5: SAW Devices (12 MESI, $5K)

**Obiettivo**: Dispositivi a onde acustiche di superficie (SAW).

**Tecnologia**:
- SAW filters commerciali (già esistenti per telecom)
- Substrato piezoelettrico (LiNbO₃)
- Interdigital transducers (IDT) litografati
- Frequenze: 100 MHz - 1 GHz

**Vantaggi**:
- Velocità: 3000-4000 m/s = **10× speedup vs aria**
- Dimensioni: chip 1cm²
- Consumo: <1W
- Produzione: litografia standard (fab esistenti)

**Costo**:
- Prototipo custom: $2-5K (fab universitario)
- Produzione batch (100 unità): $50-100/unità

**Latenza**: <1ms

---

### FASE 6: Fiber Optics (24 MESI, $50K)

**Obiettivo**: Onde ottiche in fibra (non chip integrato).

**Architettura**:
- 64 laser DFB (Distributed Feedback) a frequenze diverse ($500-1K/laser)
- Fibre ottiche multimodali ($100-500)
- Photodetector array ($5-10K)
- FPGA per controllo ($5K)

**Vantaggi**:
- Velocità luce: 200,000 km/s in fibra = **600,000× speedup vs aria**
- Zero crosstalk (WDM — Wavelength Division Multiplexing)
- Consumo: 10-50W (vs kW per GPU)

**Costo**:
- Prototipo: $30-50K (componenti discreti)
- NO chip custom (usiamo componenti telecom esistenti)

**Latenza**: <100 μs (microsecondi)

---

### FASE 7: Photonic Integrated Circuit (5+ ANNI, $2-5M)

**Obiettivo**: Chip fotonico integrato (come proposto originalmente).

**Tecnologia**:
- Silicon photonics (Intel, TSMC)
- 64 laser on-chip
- Waveguide network
- Detector array integrato

**Vantaggi**:
- Dimensioni: chip 1cm²
- Consumo: <5W
- Latenza: <10 μs
- Produzione di massa: $10-50/chip

**Costo NRE**: $2-5M (mask set + fab run)

**Quando**: Solo dopo aver dimostrato tutto con Fase 1-6.

---

## 4. Strategia "Povera" — Perché Funziona

### 4.1 Il Modello Linux

Linux non è nato con $100M di funding. È nato con:
- Un ragazzo (Linus)
- Un PC 386 ($1000)
- Tanto tempo libero
- Codice open source

**Prometeo segue lo stesso percorso**:
- Fase 1-2: Proof-of-concept ($0-200)
- Fase 3-4: Working prototype ($500-1K)
- Fase 5: Prodotto funzionale ($5K)
- Fase 6-7: Scaling industriale (funding arriva DOPO il successo)

### 4.2 Il Modello Bitcoin

Bitcoin non è nato in un lab corporate. È nato con:
- Un whitepaper (9 pagine)
- Codice open source
- Mining su CPU consumer
- Community grassroots

**Prometeo fa lo stesso**:
- Paper: "Wave-Based Cognitive Architecture" (arXiv)
- Codice: GitHub open source
- Hardware: Raspberry Pi + piezo ($100)
- Community: sviluppatori + maker

### 4.3 Perché i VCs Arrivano Dopo

**Ordine SBAGLIATO**:
```
Idea → pitch VCs → $2M → build chip → sperare funzioni
```
Rischio: 90% fallimento, nessun prodotto.

**Ordine GIUSTO** (Linux/Bitcoin):
```
Idea → proof-of-concept ($0) → working prototype ($500) →
community → traction → VCs ti cercano → scaling
```
Rischio: 0% (hai già dimostrato che funziona).

**Prometeo segue il secondo percorso.**

---

## 5. Metriche di Successo per Ogni Fase

### Fase 1 (Python Simulation)
- ✅ Interferenza produce propagazione corretta
- ✅ Correlazione misura risonanza
- ✅ Grafici dimostrano il concetto

### Fase 2 (Acoustic POC)
- ✅ 8 parole riconoscibili per interferenza
- ✅ Propagazione fisica > 70% accuratezza vs software
- ✅ Latenza <200ms

### Fase 3 (64×64)
- ✅ 64 parole × 64 dimensioni
- ✅ Accuratezza >85% vs PF1 software
- ✅ Latenza <50ms
- ✅ Consumo <10W

### Fase 4 (Ultrasonic)
- ✅ Latenza <10ms
- ✅ Miniaturizzazione (volume <1L)
- ✅ Consumo <5W

### Fase 5 (SAW)
- ✅ Latenza <1ms
- ✅ Chip-scale (1cm²)
- ✅ Consumo <1W
- ✅ Costo <$100/unità (batch 100)

### Fase 6 (Fiber Optics)
- ✅ Latenza <100μs
- ✅ 1000× speedup vs software
- ✅ Consumo <50W
- ✅ Dimostrazione full-scale (6751 parole)

### Fase 7 (Photonic IC)
- ✅ Latenza <10μs
- ✅ Produzione di massa
- ✅ Costo <$50/chip
- ✅ Integrazione in prodotti consumer

---

## 6. Prossimi Passi Immediati

### Questa Settimana
1. ✅ Creare `wave_simulation.py` (fatto)
2. ⬜ Eseguire simulazione, verificare risultati
3. ⬜ Generare grafici interferenza
4. ⬜ Scrivere paper draft (5 pagine)

### Prossimo Mese
1. ⬜ Ordinare hardware Fase 2 (Raspberry Pi + piezo)
2. ⬜ Implementare `acoustic_poc.py`
3. ⬜ Test fisico: 8 parole in aria
4. ⬜ Misurare accuratezza vs PF1 software
5. ⬜ Video demo (YouTube)

### Prossimi 3 Mesi
1. ⬜ Paper su arXiv: "Prometeo: Wave-Based Cognitive Architecture"
2. ⬜ GitHub repo pubblico con codice + hardware design
3. ⬜ Scaling a 64×64 (Fase 3)
4. ⬜ Community: Reddit, HackerNews, Twitter

### Prossimi 6-12 Mesi
1. ⬜ Working prototype Fase 4-5
2. ⬜ Pubblicazione peer-reviewed (NeurIPS, ICML, o Nature Machine Intelligence)
3. ⬜ Conferenze (demo live)
4. ⬜ A questo punto: VCs ti cercano (non viceversa)

---

## 7. Perché Questo Approccio Vince

### 7.1 Rischio Zero
- Fase 1: $0 — solo tempo
- Fase 2: $100 — costo di una cena
- Fase 3: $500 — costo di uno smartphone

**Se fallisce**: hai perso $500 e imparato molto.
**Se funziona**: hai rivoluzionato l'AI hardware.

### 7.2 Proof Incrementale
Ogni fase dimostra un pezzo:
- Fase 1: matematica corretta
- Fase 2: fisica funziona
- Fase 3: scala a dimensioni reali
- Fase 4-7: ottimizzazione

**Non serve credere — si vede.**

### 7.3 Open Source = Community
- Codice su GitHub
- Hardware design open (KiCad, Fritzing)
- Paper su arXiv (pre-print)
- Video su YouTube

**Chiunque può replicare**. Come Linux. Come Bitcoin.

### 7.4 Il Timing È Perfetto
- Neuromorphic computing è hot topic (Intel Loihi, IBM TrueNorth)
- Silicon photonics sta maturando (Intel, Lightmatter)
- AI hardware è collo di bottiglia (GPU shortage, consumo energetico)

**Prometeo wave-based è la risposta che nessuno ha ancora provato.**

---

## 8. Confronto con Alternative

| Approccio | Costo Iniziale | Tempo | Rischio | Scalabilità |
|-----------|----------------|-------|---------|-------------|
| **GPU (attuale)** | $0 (software) | 0 | Basso | Limitata (O(N) scan) |
| **FPGA** | $300 | 3 mesi | Medio | Buona (10,000×) |
| **ASIC custom** | $500K | 18 mesi | Alto | Ottima (100,000×) |
| **Memristor** | N/A (non disponibile) | 5-10 anni | Altissimo | Teorica |
| **Photonic IC** | $2-5M | 24 mesi | Altissimo | Ottima (200M×) |
| **Wave (nostra)** | $100 | 1 mese | **Bassissimo** | **Ottima (parallelo perfetto)** |

**Wave-based vince su costo/rischio/tempo.**

---

## 9. Conclusione

**Prometeo non ha bisogno di $2M per iniziare.**

Ha bisogno di:
- $100 (Raspberry Pi + piezo)
- 1 mese (implementazione Fase 2)
- Proof-of-concept funzionante
- Video demo
- Paper su arXiv
- Community

**Poi il resto viene da sé.**

Come Linux. Come Bitcoin. Come ogni rivoluzione vera.

**Bottom-up. Grassroots. Open source.**

**Prometeo è un progetto da garage che diventerà industria.**

---

## 10. Call to Action

**Se sei un maker/hacker/ricercatore**:
1. Clona il repo
2. Esegui `wave_simulation.py`
3. Ordina un Raspberry Pi
4. Costruisci il POC acustico
5. Condividi i risultati

**Se sei un investitore**:
1. Aspetta Fase 3 (working prototype)
2. Guarda i risultati
3. A quel punto parliamo

**Se sei uno scettico**:
1. Esegui la simulazione
2. Verifica la matematica
3. Cambia idea

**Il codice non mente. La fisica non mente.**

**Prometeo è reale. E inizia con $100.**
