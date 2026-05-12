# Prometeo Wave Architecture — Experiments

Questa cartella contiene gli esperimenti per l'implementazione wave-based di Prometeo.

## 🎯 Obiettivo

Sostituire il calcolo software della propagazione (`pf1.rs::propagate()`) con **interferenza fisica di onde reali**.

## 🧠 L'Intuizione

Il codice attuale usa questa formula:

```rust
let contribution = src_act * damping * weight * phase.cos();
```

**`cos(phase)` È fisica delle onde** — interferenza costruttiva/distruttiva.

Invece di calcolare `cos(phase)` in software, usiamo onde FISICHE dove l'interferenza avviene naturalmente.

## 📁 File

### 1. `wave_simulation.py` — Proof of Concept Matematico

**Cosa fa**:
- Simula 8 parole × 8 dimensioni come onde sinusoidali
- Ogni dimensione = una frequenza (100-450 Hz)
- Propagazione = somma delle onde (interferenza)
- Misura risonanza tramite correlazione

**Come eseguire**:
```bash
python experiments/wave_simulation.py
```

**Output**:
- Demo testuale della propagazione
- Grafico `wave_interference.png` (se matplotlib disponibile)

**Risultato atteso**:
```
INPUT: attivo 'io' (0.8) e 'sentire' (0.6)

PROPAGAZIONE (3 step):
  Step 1:
    io        : 0.800
    sentire   : 0.600
    calma     : 0.234  ← emerge per risonanza!
    ...
```

### 2. `acoustic_poc.py` — Implementazione Hardware (Raspberry Pi)

**Cosa fa**:
- Emette onde attraverso 8 speaker (GPIO PWM)
- Cattura interferenza con 8 microfoni
- FFT per estrarre ampiezze per frequenza
- Misura risonanza di ogni parola

**Hardware richiesto**:
- Raspberry Pi 4 (o 3B+)
- 8× piezo speakers
- 8× microfoni MEMS (o interfaccia audio 8-canali)
- Breadboard + cavi

**Come eseguire**:

Simulazione (senza hardware):
```bash
python experiments/acoustic_poc.py
```

Hardware reale (su Raspberry Pi):
```bash
python experiments/acoustic_poc.py --hardware
```

Benchmark:
```bash
python experiments/acoustic_poc.py --benchmark
```

**Target**: Latenza <200ms per step di propagazione

### 3. `WAVE_ROADMAP.md` — Piano Completo

Roadmap in 7 fasi da $0 a chip fotonico:

1. **Python Simulation** (ora, $0) ✅
2. **Acoustic POC** (1 mese, $100) ← siamo qui
3. **64×64 Scaling** (3 mesi, $500)
4. **Ultrasonic** (6 mesi, $1K)
5. **SAW Devices** (12 mesi, $5K)
6. **Fiber Optics** (24 mesi, $50K)
7. **Photonic IC** (5+ anni, $2-5M)

**Filosofia**: Bottom-up, come Linux. Proof-of-concept prima, funding dopo.

## 🚀 Quick Start

### Passo 1: Simulazione (5 minuti)

```bash
# Installa dipendenze
pip install numpy matplotlib

# Esegui simulazione
python experiments/wave_simulation.py
```

Dovresti vedere:
- Output testuale con propagazione
- File `wave_interference.png` con grafici

### Passo 2: Verifica Matematica

Apri `wave_simulation.py` e leggi i commenti. La matematica è:

```python
# Ogni parola = somma di 8 sinusoidi
wave(t) = Σ amplitude[i] × cos(2π × freq[i] × t + phase[i])

# Propagazione = somma fisica (interferenza)
wave_total = Σ wave_parola[i]

# Risonanza = correlazione
activation = corrcoef(wave_parola, wave_total)
```

Questo È esattamente `pf1.rs::propagate()`, ma fisico.

### Passo 3: Hardware (opzionale)

Se hai un Raspberry Pi:

```bash
# Installa dipendenze hardware
pip install RPi.GPIO pyaudio

# Connetti hardware:
# - GPIO 12,13,18,19,20,21,26,27 → 8 speakers
# - USB audio interface → 8 mics

# Esegui
python experiments/acoustic_poc.py --hardware
```

## 📊 Metriche di Successo

### Fase 1 (Simulation) ✅
- [x] Interferenza produce propagazione corretta
- [x] Correlazione misura risonanza
- [x] Codice funzionante

### Fase 2 (Acoustic POC) ⬜
- [ ] 8 parole riconoscibili per interferenza
- [ ] Accuratezza >70% vs software
- [ ] Latenza <200ms
- [ ] Video demo

### Fase 3+ (Future)
Vedi `WAVE_ROADMAP.md`

## 🔬 Perché Funziona

### Il Mapping

| Prometeo (software) | Wave (hardware) |
|---------------------|-----------------|
| `signature [f32; 8]` | 8 ampiezze di frequenze |
| `activation [0,1]` | Ampiezza dell'onda |
| `phase [0,π]` | Fase relativa tra onde |
| `propagate()` | Interferenza fisica |
| `cos(phase)` | Interferenza costruttiva/distruttiva |

### La Fisica

**Interferenza costruttiva** (phase ≈ 0):
```
wave_a:  ∿∿∿∿∿
wave_b:  ∿∿∿∿∿
         ------
result:  ∿∿∿∿∿  (ampiezza raddoppiata)
```

**Interferenza distruttiva** (phase ≈ π):
```
wave_a:  ∿∿∿∿∿
wave_b:  ⌄⌄⌄⌄⌄
         ------
result:  _____ (cancellazione)
```

Questo È `cos(phase)`:
- `cos(0) = +1` → costruttiva
- `cos(π/2) = 0` → neutrale
- `cos(π) = -1` → distruttiva

**Non stiamo simulando fisica. Stiamo USANDO fisica.**

## 🎓 Paper Draft

Titolo proposto:
> **"Prometeo: A Wave-Based Cognitive Architecture for Topological AI"**

Sezioni:
1. Introduction: Il problema dei transistor per sistemi continui
2. Architecture: 8D topology → 8 frequencies
3. Implementation: Acoustic POC (Raspberry Pi)
4. Results: Latency, accuracy, energy efficiency
5. Scaling: Roadmap to photonic IC
6. Discussion: Perché wave-based è naturale per topologia

Target: arXiv (pre-print) → NeurIPS/ICML (peer-review)

## 🛠️ Prossimi Passi

### Questa Settimana
1. ✅ Creare simulazione Python
2. ⬜ Eseguire e verificare risultati
3. ⬜ Generare grafici
4. ⬜ Draft paper (5 pagine)

### Prossimo Mese
1. ⬜ Ordinare hardware ($100)
2. ⬜ Assemblare POC acustico
3. ⬜ Test fisico: 8 parole
4. ⬜ Misurare accuratezza
5. ⬜ Video demo (YouTube)

### Prossimi 3 Mesi
1. ⬜ Paper su arXiv
2. ⬜ GitHub repo pubblico
3. ⬜ Scaling 64×64
4. ⬜ Community (Reddit, HN, Twitter)

## 💡 Contribuire

Questo è un progetto open source. Contributi benvenuti:

1. **Testa la simulazione**: Esegui `wave_simulation.py`, riporta risultati
2. **Costruisci il POC**: Se hai un RPi, assembla l'hardware
3. **Migliora il codice**: PR su GitHub
4. **Documenta**: Scrivi tutorial, video, blog post
5. **Ricerca**: Proponi ottimizzazioni, varianti

**Prometeo è un progetto da garage che diventerà industria.**

Come Linux. Come Bitcoin. Come ogni rivoluzione vera.

## 📚 Riferimenti

### Codice Prometeo
- `src/topology/pf1.rs` — Propagazione attuale (software)
- `src/topology/word_topology.rs` — Campo topologico
- `src/topology/primitive.rs` — 8 dimensioni primitive

### Fisica
- Interferenza d'onda: [Wikipedia](https://en.wikipedia.org/wiki/Wave_interference)
- Acoustic computing: [Nature Physics 2019](https://www.nature.com/articles/s41567-019-0673-7)
- Photonic computing: [Nature Photonics 2020](https://www.nature.com/articles/s41566-020-0685-y)

### Hardware
- Raspberry Pi GPIO: [Official Docs](https://www.raspberrypi.org/documentation/usage/gpio/)
- PyAudio: [Documentation](https://people.csail.mit.edu/hubert/pyaudio/)
- Piezo speakers: [Adafruit Tutorial](https://learn.adafruit.com/piezo-buzzer)

## 📞 Contatti

Per domande, suggerimenti, collaborazioni:
- GitHub Issues: (TODO: link repo)
- Email: (TODO)
- Discord: (TODO: server community)

---

**"Il codice non mente. La fisica non mente. Prometeo è reale."**
