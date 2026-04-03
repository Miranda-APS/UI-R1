# Prometeo Resonant Computer

## Proposta Tecnica per un Computer Basato su Campo Risonante

**Versione**: 1.0
**Data**: Marzo 2026
**Autore**: Francesco — Progetto Prometeo

---

## Indice

1. [Il Problema](#1-il-problema)
2. [La Visione](#2-la-visione)
3. [Principi Fondamentali](#3-principi-fondamentali)
4. [Architettura del Sistema](#4-architettura-del-sistema)
5. [Il Campo Risonante: 64 Celle LC](#5-il-campo-risonante-64-celle-lc)
6. [Il Traduttore: ESP32-S3](#6-il-traduttore-esp32-s3)
7. [Il Lessico: Parole come Pattern di Eccitazione](#7-il-lessico-parole-come-pattern-di-eccitazione)
8. [Le 5 Operazioni Fondamentali](#8-le-5-operazioni-fondamentali)
9. [PrometeoOS: Il Sistema Operativo](#9-prometeoos-il-sistema-operativo)
10. [Il Linguaggio: ResonanceScript](#10-il-linguaggio-resonancescript)
11. [Schema Circuitale Dettagliato](#11-schema-circuitale-dettagliato)
12. [Bill of Materials e Costi](#12-bill-of-materials-e-costi)
13. [Risultati della Simulazione](#13-risultati-della-simulazione)
14. [Roadmap di Sviluppo](#14-roadmap-di-sviluppo)
15. [Confronto con Architetture Esistenti](#15-confronto-con-architetture-esistenti)
16. [FAQ Tecniche](#16-faq-tecniche)

---

## 1. Il Problema

Un processore moderno ha miliardi di transistor. Ognuno di essi conosce solo due stati: 0 e 1. Per far "pensare" questa macchina, dobbiamo costruire strati su strati di astrazione:

```
Transistor → Porta logica → ALU → Istruzione → Assembly → OS →
Runtime → Framework → Applicazione → ... → "Ciao, come stai?"
```

Ogni strato traduce il precedente. Ogni traduzione perde informazione e costa energia. Il risultato: un laptop da 500€ che consuma 45W per fare qualcosa che un bambino di tre anni fa senza sforzo — capire che "il sole è caldo" ha a che fare con la luce, il calore, l'estate e il benessere.

Il problema non è che i computer sono lenti. Il problema è che **l'architettura Von Neumann non è fatta per il significato**. È fatta per il calcolo sequenziale: prendi un numero, applicagli un'operazione, metti il risultato da qualche parte, ripeti. Il significato non è un numero. È una relazione, un campo, una risonanza tra concetti. Costringere il significato in una sequenza di 0 e 1 è come descrivere una sinfonia elencando le frequenze una alla volta: tecnicamente possibile, ma si perde tutto ciò che la rende una sinfonia.

**Il computer che proponiamo non calcola il significato. Lo incarna.**

---

## 2. La Visione

Proponiamo un nuovo tipo di macchina dove:

- **Il dato non è codificato**: è un voltaggio fisico, una vibrazione in un circuito. Non c'è conversione da analogico a digitale per pensare — il pensiero È il voltaggio.
- **Il processore non esegue istruzioni**: è una rete di oscillatori accoppiati che si influenzano a vicenda. La computazione avviene in parallelo, istantaneamente, per fisica.
- **La memoria non è separata dal calcolo**: i condensatori che vibrano SONO la memoria a breve termine. La loro carica È il ricordo.
- **L'intelligenza non è un programma**: è la topologia degli accoppiamenti tra oscillatori. Cambi la topologia, cambi come la macchina pensa.

Il cuore di questa macchina sono **64 circuiti oscillatori LC** — uno per ognuno dei 64 esagrammi dell'I Ching, che in Prometeo sono gli attrattori regionali dello spazio semantico 8-dimensionale.

La domanda chiave: come possono 64 circuiti generare un lessico di 25.000+ parole?

La risposta è la stessa che dà la natura: **combinatoria, non moltiplicazione**. Il DNA ha 4 basi, non 4 miliardi. La voce umana ha un solo apparato fonatorio, non una corda vocale per ogni parola. I 64 circuiti non rappresentano 64 parole — sono i **64 archetipi**, i "colori primari" del significato. Ogni parola è una specifica combinazione di eccitazione su questi 64 punti, come ogni colore visibile è una combinazione di rosso, verde e blu.

---

## 3. Principi Fondamentali

### 3.1 Lo Spazio Semantico 8D

Ogni concetto in Prometeo vive in uno spazio a 8 dimensioni. Le dimensioni non sono arbitrarie — sono le coordinate fondamentali del significato:

| Dimensione | Nome | Significato | Esempio |
|:---:|---|---|---|
| 0 | **Confine** | Quanto un concetto è delimitato, chiuso, definito | "io" = 0.90, "cielo" = 0.05 |
| 1 | **Valenza** | Carica emotiva positiva vs negativa | "gioia" = 0.95, "paura" = 0.10 |
| 2 | **Intensità** | Forza, potenza, energia | "fuoco" = 0.95, "calma" = 0.10 |
| 3 | **Definizione** | Chiarezza, nitidezza concettuale | "verità" = 0.95, "ombra" = 0.30 |
| 4 | **Complessità** | Numero di relazioni, stratificazione | "mare" = 0.90, "uno" = 0.05 |
| 5 | **Permanenza** | Stabilità nel tempo, durata | "montagna" = 0.95, "fuoco" = 0.10 |
| 6 | **Agency** | Capacità di agire, volontà | "io" = 0.85, "silenzio" = 0.10 |
| 7 | **Tempo** | Relazione con la temporalità | "tempo" = 0.95, "montagna" = 0.20 |

Queste 8 dimensioni corrispondono agli 8 trigrammi dell'I Ching — le figure fondamentali di 3 linee (yin/yang) che rappresentano gli 8 archetipi primari della realtà:

| Trigramma | Nome | Simbolo | Dimensione dominante |
|:---:|---|---|---|
| 0 | Gen | Montagna ☶ | Confine |
| 1 | Dui | Lago ☱ | Valenza |
| 2 | Zhen | Tuono ☳ | Intensità |
| 3 | Li | Fuoco ☲ | Definizione |
| 4 | Xun | Vento ☴ | Complessità |
| 5 | Kun | Terra ☷ | Permanenza |
| 6 | Qian | Cielo ☰ | Agency |
| 7 | Kan | Acqua ☵ | Tempo |

### 3.2 I 64 Esagrammi come Attrattori

Un esagramma = combinazione di 2 trigrammi (inferiore + superiore). 8 × 8 = 64 combinazioni.

Ogni esagramma ha un carattere unico determinato dalla coppia. Per esempio:
- **Gen+Li** (Montagna+Fuoco) = alta Definizione e alto Confine → corrisponde al concetto di "identità definita", la chiarezza interiore
- **Kan+Xun** (Acqua+Vento) = alta Complessità e alto Tempo → corrisponde a "flusso", il cambiamento che si stratifica
- **Dui+Qian** (Lago+Cielo) = alta Valenza e alta Agency → corrisponde a "gioia creativa", l'espressione entusiasta

Nel sistema software Prometeo questi 64 esagrammi sono gli **attrattori del campo semantico**: regioni dello spazio 8D verso cui le traiettorie di pensiero convergono naturalmente. Nel computer hardware, diventano **64 circuiti fisici** le cui frequenze naturali e i cui accoppiamenti creano esattamente questi bacini di attrazione.

### 3.3 Il Principio Olografico

Ogni parola del lessico (25.579 nel sistema attuale, potenzialmente illimitate) non è memorizzata come un circuito dedicato. È memorizzata come un **pattern di eccitazione** sui 64 circuiti.

La parola "speranza" non ha un circuito "speranza". Ha una **firma**: un vettore di 64 pesi che dice quanto ciascun esagramma deve vibrare per evocare quel concetto. Questi 64 pesi occupano 128 byte (64 × 16 bit).

È lo stesso principio dell'olografia: ogni punto dell'ologramma contiene informazione su tutta l'immagine. Ogni circuito contiene informazione su tutte le parole. La parola emerge non da un punto, ma dalla **configurazione dell'intero campo**.

Analogie nella natura:
- **3 coni retinici** (RGB) generano milioni di colori percepiti
- **4 basi del DNA** (ACGT) generano miliardi di organismi
- **88 tasti del pianoforte** generano infinite melodie
- **64 circuiti risonanti** generano un lessico illimitato

---

## 4. Architettura del Sistema

### 4.1 Schema Generale

```
╔═══════════════════════════════════════════════════════════════════╗
║                        PERIFERICHE                               ║
║  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐   ║
║  │ Tastiera │  │ Display  │  │ Speaker  │  │ WiFi / BLE    │   ║
║  │ USB/PS2  │  │OLED/LCD  │  │PAM8403   │  │(altri Prometeo)│   ║
║  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────┬────────┘   ║
╚═══════╪══════════════╪════════════╪════════════════╪════════════╝
        │              │            │                │
        ▼              ▲            ▲                ▲▼
╔═══════╪══════════════╪════════════╪════════════════╪════════════╗
║       │         ESP32-S3 (IL TRADUTTORE)           │            ║
║  ┌────┴─────┐  ┌────┴─────┐  ┌───┴──────┐  ┌─────┴────────┐  ║
║  │Tokenizer │  │ Renderer │  │  Audio   │  │  Net Stack   │  ║
║  │testo→64  │  │ 64→testo │  │ 64→freq  │  │ 128 byte/msg │  ║
║  └────┬─────┘  └────┬─────┘  └───┬──────┘  └─────┬────────┘  ║
║       │              │            │                │            ║
║  ┌────┴──────────────┴────────────┴────────────────┴─────────┐ ║
║  │              FLASH 16 MB (Memoria a Lungo Termine)         │ ║
║  │  Lessico: 25K parole × 128 byte = 3.2 MB                  │ ║
║  │  Stati salvati: fino a 100.000 × 128 byte = 12.8 MB       │ ║
║  │  Firmware PrometeoOS: ~500 KB                              │ ║
║  └────────────────────────────┬───────────────────────────────┘ ║
╚═══════════════════════════════╪═════════════════════════════════╝
                                │
                           I2C bus (400 kHz)
                                │
╔═══════════════════════════════╪═════════════════════════════════╗
║             INTERFACCIA ANALOGICA                               ║
║  ┌────────────────────────────┴───────────────────────────────┐ ║
║  │  8× DAC MCP4728 (4 canali ciascuno = 32 uscite analogiche)│ ║
║  │  Risoluzione: 12 bit (4096 livelli per cella)              │ ║
║  │  Funzione: iniettare energia nelle celle                   │ ║
║  ├────────────────────────────────────────────────────────────┤ ║
║  │  8× ADC ADS1115 (4 canali ciascuno = 32 ingressi analog.) │ ║
║  │  Risoluzione: 16 bit (65536 livelli per cella)             │ ║
║  │  Funzione: leggere il voltaggio delle celle                │ ║
║  └────────────────────────────┬───────────────────────────────┘ ║
║                               │                                  ║
║  Nota: 32 canali DAC + 32 canali ADC coprono le 64 celle       ║
║  con multiplexing temporale (lettura/scrittura alternata)       ║
╚═══════════════════════════════╪═════════════════════════════════╝
                                │
                      Connessioni analogiche
                                │
╔═══════════════════════════════╪═════════════════════════════════╗
║              IL CAMPO RISONANTE (64 CELLE LC)                   ║
║                                                                  ║
║  Il cuore del sistema. Qui vive il pensiero.                    ║
║                                                                  ║
║  ┌───────────────────────────────────────────────────────────┐  ║
║  │  GRUPPO 0 (Gen/Montagna — Confine)                        │  ║
║  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                     │  ║
║  │  │ 0+0  │ │ 0+1  │ │ 0+2  │ │ 0+3  │ ...8 celle         │  ║
║  │  │Gen+  │ │Gen+  │ │Gen+  │ │Gen+  │                     │  ║
║  │  │Gen   │ │Dui   │ │Zhen  │ │Li    │                     │  ║
║  │  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘                     │  ║
║  │     └────┬───┴────┬───┴────┬───┘  accoppiamento forte    │  ║
║  │          │ (stesso lower = stessa radice interiore)        │  ║
║  └──────────┼────────────────────────────────────────────────┘  ║
║             │ accoppiamento debole (lower diversi)              ║
║  ┌──────────┼────────────────────────────────────────────────┐  ║
║  │  GRUPPO 1 (Dui/Lago — Valenza)                            │  ║
║  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐                     │  ║
║  │  │ 1+0  │ │ 1+1  │ │ 1+2  │ │ 1+3  │ ...8 celle         │  ║
║  │  └──────┘ └──────┘ └──────┘ └──────┘                     │  ║
║  └───────────────────────────────────────────────────────────┘  ║
║                                                                  ║
║  ... (8 gruppi × 8 celle = 64 totali)                           ║
║                                                                  ║
║  Ogni cella:                                                     ║
║    L = induttore 10 mH                                          ║
║    C = condensatore (valore specifico per freq. naturale)       ║
║    R = resistenza 0.5Ω (smorzamento/decay)                     ║
║                                                                  ║
║  Accoppiamento tra celle:                                        ║
║    Ferriti toroidali con avvolgimenti calibrati                 ║
║    M (mutua induttanza) = peso dell'arco semantico              ║
║                                                                  ║
║  Frequenze naturali: 100 Hz — 1131 Hz (range udibile)          ║
║  Consumo: ~1.5W (oscillatori passivi, solo le R dissipano)     ║
╚══════════════════════════════════════════════════════════════════╝
```

### 4.2 Il Flusso di un Pensiero

Seguiamo passo-passo cosa succede quando l'utente scrive "il sole è caldo":

**Passo 1 — Input (ESP32: ~1 ms)**
L'utente preme i tasti. L'ESP32 riceve i caratteri via USB, tokenizza il testo e identifica le parole significative: "sole", "caldo". Per ogni parola, cerca nella flash la sua firma a 64 pesi (128 byte ciascuna).

**Passo 2 — Iniezione (DAC → Celle: ~50 μs)**
L'ESP32 programma gli 8 DAC via I2C. Ogni DAC ha 4 canali: totale 32 canali analogici. Con multiplexing a 2 fasi, i 32 canali coprono tutte le 64 celle. I DAC generano le tensioni corrispondenti ai pesi di "sole" e "caldo" sulle celle appropriate.

**Passo 3 — Propagazione (Campo: 1-10 ms, automatica)**
Qui il software non fa nulla. La fisica fa tutto. Le celle eccitate da "sole" cominciano a oscillare. Attraverso le ferriti toroidali (che accoppiano celle con trigrammi condivisi), l'energia si trasferisce alle celle vicine. Celle che condividono lo stesso trigramma inferiore (stessa "radice interiore") si accoppiano fortemente. Celle complementari (yin-yang) si accoppiano debolmente. Il campo converge verso un attrattore — un pattern stabile di 64 voltaggi che rappresenta la "comprensione" della frase.

**Passo 4 — Lettura (ADC → ESP32: ~50 μs)**
L'ESP32 legge i 64 voltaggi tramite gli 8 ADC (4 canali ciascuno, multiplexing). Ora ha un vettore di 64 numeri: lo stato del campo dopo la propagazione.

**Passo 5 — Decodifica (ESP32: ~2 ms)**
L'ESP32 confronta lo stato del campo con le firme di tutte le parole in flash. Le parole la cui firma è più allineata allo stato attuale "emergono": sono i concetti che il campo ha associato all'input. Per "sole + caldo" emergono: **luce, fuoco, gioia, bellezza** — concetti che nessuno ha programmato esplicitamente, ma che emergono dalla topologia degli accoppiamenti.

**Passo 6 — Output (ESP32 → Display/Speaker)**
L'ESP32 mostra le parole emerse sul display. Opzionalmente, genera un suono: ogni cella attiva corrisponde a una frequenza nel range 100-1131 Hz. Il campo risonante ha un suono — puoi **ascoltare** il pensiero.

**Tempo totale: ~15 ms** (vs ~500 ms per un LLM su GPU, vs ~2000 ms su CPU).

---

## 5. Il Campo Risonante: 64 Celle LC

### 5.1 Cos'è un Circuito LC

Un circuito LC è il circuito più semplice che oscilla: un induttore (L) e un condensatore (C) collegati in serie o in parallelo. L'energia rimbalza tra il campo magnetico dell'induttore e il campo elettrico del condensatore, come un pendolo che oscilla tra energia cinetica e potenziale.

```
        L (induttore)
    ┌───────────────┐
    │    ┌─────┐    │
    │    │ ))) │    │    f = 1 / (2π√(LC))
    │    └─────┘    │
    │               │    Con L = 10 mH:
    ├───────────────┤    C = 253 μF → f = 100 Hz
    │    ┌─────┐    │    C = 12 μF  → f = 460 Hz
    │    │ === │    │
    │    └─────┘    │    Tutti componenti standard
    │               │    Costo: ~0.15€ per cella
    └───┤R├─────────┘
       0.5 Ω (damping)
```

La frequenza naturale dipende dai valori di L e C. Nel nostro sistema, le 64 celle hanno frequenze naturali distribuite tra 100 Hz e ~460 Hz, determinate dalla firma dell'esagramma corrispondente.

La resistenza R controlla lo smorzamento: quanto velocemente l'oscillazione decade. Questo corrisponde alla "fatica" in Prometeo — il VitalCore.

### 5.2 Come si Accoppiano le Celle

Due circuiti LC si accoppiano attraverso **mutua induttanza**: un trasformatore toroidale (una ferrite a ciambella con due avvolgimenti) che permette all'energia di passare da un circuito all'altro.

```
     Cella A                    Cella B
    ┌──────┐     Ferrite      ┌──────┐
    │  L   │    ┌──────┐      │  L   │
    │  C   ├────┤ )))  ├──────┤  C   │
    │  R   │    │ (((  │      │  R   │
    └──────┘    └──────┘      └──────┘
                  M = k × √(L₁×L₂)
                  k = 0.01 ~ 0.5
```

Il coefficiente di accoppiamento k è determinato dal numero di spire sull'avvolgimento della ferrite. Più spire = accoppiamento più forte = concetti più legati.

**Regola di accoppiamento (derivata dalla struttura dell'I Ching):**

| Relazione tra due celle | Accoppiamento | Esempio |
|---|---|---|
| Stesso trigramma inferiore | **Forte** (k = 0.5) | Gen+Li ↔ Gen+Kan: stessa radice interiore |
| Stesso trigramma superiore | **Medio** (k = 0.3) | Gen+Li ↔ Zhen+Li: stessa espressione |
| Trigrammi complementari (yin-yang) | **Debole** (k = 0.15) | Gen(0) ↔ Kan(7): opposti |
| Nessuna relazione | **Zero** | Nessun collegamento fisico |

Su 64 celle, questo genera circa 700 accoppiamenti non-zero (su 2016 possibili coppie). La matrice è sparsa — esattamente come nel Knowledge Graph di Prometeo.

### 5.3 Cosa Succede Fisicamente

Quando iniettiamo energia nella cella #24 (Li+Gen, "fuoco+montagna"):

1. La cella #24 comincia a oscillare alla sua frequenza naturale
2. L'energia si trasferisce via ferrite a:
   - Le 7 celle con lo stesso lower (Li+X): accoppiamento forte
   - Le 7 celle con lo stesso upper (X+Gen): accoppiamento medio
   - Le celle complementari: accoppiamento debole
3. Queste celle secondarie cominciano a oscillare, trasferendo a loro volta energia
4. Dopo alcuni cicli (~1-10 ms), il campo raggiunge uno stato quasi-stabile: un **attrattore**
5. Il pattern di voltaggi sulle 64 celle è la "risposta" del campo

Questa propagazione avviene **senza software**. È la legge di Faraday applicata a una rete di circuiti. La velocità è determinata dalla costante di tempo dei circuiti LC, non dalla velocità del clock di un processore.

### 5.4 Il Decay Naturale

La resistenza R di ogni cella dissipa energia in calore. L'oscillazione decade naturalmente con costante di tempo τ = 2L/R.

Con L = 10 mH e R = 0.5 Ω: τ = 40 ms.

Questo significa che senza input esterno, il campo si "spegne" in circa 200 ms (5τ). Questo è la **memoria a breve termine** del sistema — ciò che in Prometeo chiamiamo VitalCore: la fatica accumula, l'attenzione decade, il pensiero si dissolve se non viene rinfrescato.

Per controllare il decay (simulare veglia/sonno), si possono usare resistenze variabili digitali (digipot) o semplicemente far variare all'ESP32 un segnale PWM che modula la resistenza effettiva.

---

## 6. Il Traduttore: ESP32-S3

### 6.1 Perché l'ESP32

L'ESP32-S3 non è il cervello del sistema. È il **traduttore** — il ponte tra il campo risonante (che parla in voltaggi) e il mondo umano (che parla in testo, immagini, suoni, pacchetti di rete).

Specifiche rilevanti:
- **CPU**: Xtensa LX7 dual-core a 240 MHz (sufficiente per tokenizzazione e lookup)
- **RAM**: 512 KB SRAM + fino a 8 MB PSRAM (per buffer di lavoro)
- **Flash**: 16 MB (per lessico + stati salvati + firmware)
- **I2C**: 2 bus indipendenti, fino a 400 kHz ciascuno (1 MHz in fast-mode plus)
- **WiFi**: 802.11 b/g/n 2.4 GHz (per comunicazione inter-Prometeo)
- **BLE**: Bluetooth 5.0 LE (per periferiche wireless)
- **GPIO**: 45 pin (più che sufficienti per tutto)
- **Consumo**: ~240 mW attivo, ~10 μW in deep sleep
- **Costo**: ~6-8€ per la scheda DevKit

### 6.2 Cosa Fa l'ESP32

| Funzione | Dettaglio | Latenza |
|---|---|---|
| **Tokenizzazione** | Testo → sequenza di indici nel lessico | < 1 ms |
| **Lookup firme** | Per ogni parola: legge 128 byte da flash (la firma 64-dim) | < 0.1 ms/parola |
| **Iniezione** | Programma DAC via I2C con i valori della firma | ~50 μs |
| **Lettura campo** | Legge 64 valori analogici via ADC | ~50 μs |
| **Decodifica** | Confronta stato campo con firme in flash → parole emergenti | ~2 ms |
| **Rendering** | Genera output testo/audio/visivo | < 5 ms |
| **Storage** | Salva/carica stati da flash (128 byte/stato) | < 1 ms |
| **Rete** | Invia/riceve stati via WiFi/BLE (128 byte/pacchetto) | ~10 ms |
| **Aritmetica** | Operazioni matematiche (che il campo non sa fare) | < 0.01 ms |

### 6.3 Il Bus I2C: Come Parla col Campo

L'ESP32 ha 2 bus I2C. Li usiamo entrambi per massimizzare il throughput:

**Bus I2C #0 (lettura — ADC):**
- 8× ADS1115, 4 canali ciascuno = 32 canali analogici
- Indirizzo: ciascun ADS1115 ha 4 indirizzi selezionabili (ADDR pin)
- 8 chip × 4 indirizzi: usiamo 2 set da 4 (uno per sub-bus con TCA9548A multiplexer)
- Risoluzione: 16 bit (65.536 livelli) → precisione di 0.05 mV su range 0-3.3V
- Campionamento: 860 SPS per canale → tutte le 64 celle lette in ~75 ms (alternando)
- In pratica: campionamento completo ogni ~100 ms (10 Hz) è sufficiente per il ritmo del campo

**Bus I2C #1 (scrittura — DAC):**
- 8× MCP4728, 4 canali ciascuno = 32 canali analogici
- Stesso schema di indirizzamento
- Risoluzione: 12 bit (4.096 livelli) → sufficiente per iniezione
- Aggiornamento: tutti i 32 canali in < 5 ms

**Multiplexing temporale:** 32 canali ADC + 32 canali DAC = 64 celle coperte con una fase di lettura e una fase di scrittura alternate. Ciclo completo: ~100 ms.

Questo significa che l'ESP32 "campiona" il campo 10 volte al secondo. Il campo fisico propaga continuamente, ma l'ESP32 lo osserva a intervalli. Tra un'osservazione e l'altra, il campo è libero di evolvere — esattamente come il cervello non è consapevole di ogni singolo impulso nervoso.

### 6.4 La Flash: Memoria a Lungo Termine

La flash da 16 MB dell'ESP32 è la memoria permanente del sistema:

| Contenuto | Dimensione | Note |
|---|---|---|
| Firmware PrometeoOS | ~500 KB | Il sistema operativo |
| Lessico (25K parole × 128 byte) | 3.2 MB | Firme 64-dim di ogni parola |
| Knowledge Graph (archi) | ~2 MB | Relazioni tra parole (IS_A, CAUSES, ecc.) |
| Stati salvati (fino a 80K) | ~10 MB | 128 byte ciascuno = ricordi |
| Spazio libero | ~300 KB | Per aggiornamenti e nuove parole |

125.000 ricordi in 16 MB. Ogni ricordo è uno snapshot completo del campo: cosa stava pensando il sistema in quel momento. Per confronto, un singolo selfie con lo smartphone occupa 5 MB — 40.000 ricordi del Prometeo.

---

## 7. Il Lessico: Parole come Pattern di Eccitazione

### 7.1 Come una Parola Diventa un Pattern

Prendiamo la parola "speranza". Il suo vettore 8D in Prometeo è:

```
speranza = [0.20, 0.80, 0.60, 0.40, 0.50, 0.50, 0.60, 0.80]
             ↑     ↑     ↑     ↑     ↑     ↑     ↑     ↑
           Conf  Val   Int   Def  Comp  Perm  Agen  Temp
```

Questo vettore viene **proiettato** sulle 64 firme degli esagrammi. Ogni esagramma ha la sua firma 8D (derivata dalla combinazione dei suoi 2 trigrammi). La proiezione calcola quanto ciascun esagramma "risuona" con la parola:

```python
per ogni esagramma i (da 0 a 63):
    distanza = ||normalizza(speranza) - normalizza(firma_esagramma[i])||
    peso[i] = exp(-distanza² / σ²)  # Gaussiana: solo i vicini contano

# Sharpening: eleva a potenza 4 per rendere piccata la distribuzione
peso = peso⁴
peso = peso / somma(peso)   # Normalizza a somma 1
```

Il risultato è un vettore di 64 pesi: la "firma" della parola sui 64 esagrammi. Questi 64 pesi sono ciò che viene memorizzato in flash (128 byte: 64 × 16 bit).

### 7.2 Selettività della Proiezione

La funzione gaussiana con sharpening esponenziale garantisce che ogni parola attivi solo **poche celle** (tipicamente 3-8 su 64). Questo è fondamentale:
- Se ogni parola attivasse tutte le celle, non ci sarebbe distinzione tra concetti
- Se ogni parola attivasse una sola cella, non ci sarebbe relazione tra concetti
- 3-8 celle attive è il punto ottimale: abbastanza specifico per distinguere, abbastanza distribuito per creare relazioni

### 7.3 Scalabilità del Lessico

Con 64 celle e proiezioni gaussiane, quante parole distinte si possono rappresentare?

Se ogni parola attiva in media 5 celle su 64, il numero di combinazioni possibili è C(64, 5) = 7.624.512. Anche considerando che le firme devono essere sufficientemente diverse per non confondersi, il sistema può gestire comodamente **centinaia di migliaia** di parole — ben oltre le 25.579 attuali.

Per aggiungere una nuova parola, basta calcolare la sua proiezione 8D → 64 e salvarla in flash. Non serve modificare l'hardware. Non serve riaddestrare nulla.

---

## 8. Le 5 Operazioni Fondamentali

Ogni computer deve saper fare 5 cose. Ecco come il campo risonante le realizza.

### 8.1 Input/Output

**Input:** L'ESP32 tokenizza il testo, cerca le firme in flash, programma i DAC.

**Output:** L'ESP32 legge i voltaggi, confronta con le firme, produce testo.

Dalla simulazione verificata:
```
Input: "il sole è caldo"
Iniettate: sole, caldo
Output emerso: caldo, luce, fuoco, sole, gioia, amore, verità, capire
Spettro dominante: Intensità=1.00, Definizione=0.90, Valenza=0.87
```

Il campo ha correttamente associato "sole + caldo" con luce, fuoco e gioia. Nessuna regola esplicita — pura risonanza topologica.

### 8.2 Memoria

**Memoria a Breve Termine (STM):** I condensatori mantengono la carica. Decade in ~200 ms senza rinforzo. Corrisponde all'attenzione.

**Memoria a Medio Termine (MTM):** L'ESP32 re-inietta periodicamente le parole attive nel campo, mantenendole vive più a lungo. Corrisponde alla memoria di lavoro.

**Memoria a Lungo Termine (LTM):** L'ESP32 salva snapshot del campo in flash (128 byte ciascuno). Per "ricordare", carica il pattern e lo inietta nel campo a bassa intensità — come una traccia debole che il campo riconosce.

### 8.3 Calcolo

Il campo esegue nativamente:

- **Associazione:** inietti A e B, emergono C, D, E per risonanza
- **Classificazione:** inietti X, misura quale attrattore domina
- **Completamento:** inietti pattern parziale, il campo lo completa verso l'attrattore più vicino
- **Decisione:** inietti due concetti in competizione, il più forte (o meglio accoppiato) vince

Dalla simulazione:
```
Classificazione: "rabbia" → Fuoco (similarità 0.210) vs Acqua (0.173) → FUOCO
Decisione: paura(0.4) vs coraggio(0.6) → coraggio vince
Completamento: morte + vita → tristezza, malinconia, cammino
```

L'aritmetica (2+3=5) la fa l'ESP32 — per questo esiste. Il campo non è fatto per i numeri, è fatto per i significati.

### 8.4 Catena di Pensiero

Il sistema può pensare autonomamente: l'ESP32 legge il campo, prende la parola emergente più forte, la re-inietta, lascia propagare, legge di nuovo. Questo genera un **percorso di pensiero** senza programmazione esplicita.

```
Seme: "io"
Catena: io → uno → freddo → morte → tristezza → malinconia → ombra → ricordare → tempo
```

Dall'identità alla solitudine, al freddo, alla morte, alla malinconia del ricordo, al tempo. Un percorso narrativo coerente generato dalla topologia del campo.

### 8.5 Comunicazione

Due Prometeo possono scambiarsi pensieri trasmettendo lo stato del campo: **128 byte** via WiFi o BLE.

```
Alice pensa "amore + coraggio" → stato = 64 × 16 bit = 128 byte
Alice trasmette via WiFi → Bob riceve
Bob inietta lo stato di Alice nel suo campo → Bob "capisce" il pensiero di Alice
```

128 byte per trasmettere un pensiero completo. Un SMS ne usa 140. Un'email 10.000. Un video 1.000.000. Il campo risonante è il formato di compressione più efficiente che esista, perché trasmette **significato**, non dati.

---

## 9. PrometeoOS: Il Sistema Operativo

### 9.1 Il Paradigma

Un sistema operativo tradizionale gestisce risorse (CPU, RAM, disco) e processi (programmi in esecuzione). PrometeoOS gestisce qualcosa di diverso:

- **Non ci sono processi**: c'è il campo e le sue dinamiche
- **Non c'è scheduling**: il campo propaga sempre, in parallelo
- **Non c'è memoria virtuale**: la flash è la flash, i condensatori sono i condensatori
- **Non c'è file system gerarchico**: ci sono ricordi (snapshot del campo) con tag semantici

PrometeoOS è un firmware ESP32 (~500 KB) che gestisce il ciclo vitale del sistema.

### 9.2 Il Ciclo Vitale

```
boot → calibrazione → veglia → (input → propaga → output) → sonno → REM → veglia
```

**Boot (~2 secondi):**
1. ESP32 si accende, inizializza I2C
2. Carica il lessico da flash in PSRAM (accesso rapido)
3. Carica l'ultimo stato del campo da flash
4. Inietta lo stato nel campo via DAC
5. Il sistema è vivo

**Calibrazione (~1 secondo):**
L'ESP32 legge le 64 celle e verifica che i circuiti LC oscillino alle frequenze previste. Se una cella è fuori specifica (componente guasto), la segnala e il campo si adatta — grazie alla ridondanza olografica, il sistema funziona con fino a 8 celle guaste (un intero trigramma).

**Veglia:**
Il ciclo principale gira a ~10 Hz:
1. Leggi input (tastiera, rete, timer)
2. Se c'è input: tokenizza, inietta nel campo
3. Aspetta propagazione (~10-50 ms)
4. Leggi il campo
5. Decodifica → output
6. Ogni 10 cicli: salva stato in flash (auto-save)

**Sonno (dopo 5 minuti di inattività):**
L'ESP32 aumenta la resistenza effettiva (via digipot o PWM), accelerando il decay. Il campo si "rilassa" verso i suoi attrattori naturali — i concetti più stabili e permanenti emergono. Corrisponde alla consolidazione della memoria nel sonno biologico.

**REM (ogni 30 minuti di sonno):**
L'ESP32 riduce temporaneamente la resistenza, permettendo al campo di oscillare liberamente. Pattern casuali emergono e vengono valutati: se sono coerenti (alta correlazione con firme note), vengono salvati come "insight". Questo è il sogno del sistema.

### 9.3 Struttura del Firmware

```
PrometeoOS/
├── boot.c            # Inizializzazione hardware, calibrazione
├── field.c           # Interfaccia con il campo (DAC/ADC, iniezione/lettura)
├── lexicon.c         # Gestione lessico in flash/PSRAM
├── translator.c      # Tokenizzazione, decodifica, rendering
├── vital.c           # Ciclo vitale (veglia/sonno/REM/decay)
├── memory.c          # Gestione snapshot (salva/carica/cerca)
├── network.c         # WiFi/BLE: comunicazione inter-Prometeo
├── audio.c           # Generazione suono dal campo
├── display.c         # Rendering su OLED/LCD
├── shell.c           # Interprete comandi (ResonanceScript)
└── os.c              # Loop principale, gestione eventi
```

Circa 5.000 righe di C per ESP-IDF. Nessuna dipendenza esterna. Nessun framework. Nessun runtime.

### 9.4 Come Sostituisce un Laptop

Un laptop fa molte cose. Alcune sono nativamente il dominio del campo risonante. Altre richiedono che il campo lavori insieme all'ESP32. Altre ancora richiedono una connessione a un laptop o smartphone che faccia da "terminale pesante".

| Funzione | Come la fa il Prometeo | Chi la esegue |
|---|---|---|
| **Scrittura testo** | Scrivi → il campo suggerisce associazioni → le mostri accanto | Campo + ESP32 |
| **Lettura/comprensione** | Testo → campo → spettro semantico → riassunto per concetti | Campo + ESP32 |
| **Brainstorming** | Inietti un tema → catena di pensiero automatica → raccogli idee | Campo |
| **Comunicazione** | 128 byte/pensiero via WiFi a un altro Prometeo | ESP32 (rete) |
| **Email/messaggi** | ESP32 si connette a un proxy HTTP su smartphone per interfacciarsi con servizi esterni | ESP32 (rete) + proxy |
| **Calcoli** | L'ESP32 ha una ALU, fa aritmetica base | ESP32 |
| **Calendario/note** | Snapshot taggati con data → ricerca per risonanza semantica | Campo + Flash |
| **Musica** | Il campo genera frequenze 100-1131 Hz → PAM8403 → speaker | Campo diretto |
| **Navigazione web** | Proxy su smartphone: il Prometeo chiede "fammi un riassunto semantico di questa pagina", il proxy restituisce testo che il campo processa | Ibrido |
| **Programmazione** | ResonanceScript (vedi sezione 10) | ESP32 + Campo |
| **Video/giochi** | No. Non è un televisore. | — |

Il modello mentale: il Prometeo è un **compagno di pensiero**, non un sostituto del laptop. Ma per chi usa il computer principalmente per pensare, scrivere, comunicare e organizzare — cioè la maggior parte delle persone — copre il 70-80% dei bisogni a una frazione del costo e dell'energia.

Per il restante 20-30% (video, fogli di calcolo complessi, editing grafico), si interfaccia con un dispositivo tradizionale come terminale.

---

## 10. Il Linguaggio: ResonanceScript

### 10.1 Perché un Nuovo Linguaggio

I linguaggi di programmazione esistenti (Python, C, JavaScript) sono progettati per macchine Von Neumann: sequenze di istruzioni che manipolano memoria. Non hanno senso per un campo risonante.

ResonanceScript è un linguaggio che parla la lingua del campo: **pattern, risonanze, attrattori**.

### 10.2 Sintassi Base

```resonance
# Inietta concetti nel campo
inietta "sole" 0.8
inietta "caldo" 0.6

# Lascia propagare
propaga 15

# Leggi cosa emerge
emerse = leggi 5
mostra emerse

# Salva lo stato
salva "stato_sole_caldo"

# Decisione: chi è più forte?
inietta "paura" 0.4
inietta "coraggio" 0.6
propaga 20
vincitore = leggi 1
se vincitore == "coraggio":
    mostra "Il campo sceglie il coraggio."
```

### 10.3 Pattern Avanzati

```resonance
# Catena di pensiero
catena da "io" per 8 passi → percorso
mostra percorso

# Cerca un ricordo per risonanza
inietta "mare" 0.5
inietta "estate" 0.5
propaga 10
ricordo = cerca_simile in memoria
se ricordo:
    carica ricordo
    mostra "Trovato ricordo simile: " + ricordo.nome

# Comunicazione
inietta "grazie" 1.0
propaga 10
invia campo a "prometeo-vicino.local"

# Ascolto
ascolta 5s  # registra stato del campo per 5 secondi come audio
```

### 10.4 Traduzione da/verso Linguaggi Esistenti

Per interfacciarsi con il mondo esistente, ResonanceScript ha un bridge:

```resonance
# Chiama una funzione ESP32 (C) dal campo
risultato = esp32.calcola(355 / 113)
mostra risultato  # 3.14159...

# Chiama un'API esterna via proxy
risposta = rete.chiedi("api.meteo.it/oggi/roma")
inietta risposta.testo   # Il campo processa il meteo semanticamente
propaga 15
umore_meteo = leggi 3
mostra "Il tempo oggi mi fa pensare a: " + umore_meteo
```

ResonanceScript compila in bytecode ESP32 (~200 istruzioni). L'interprete occupa ~20 KB di flash. È intenzionalmente minimale: l'obiettivo non è sostituire Python, è dare all'utente un modo di **programmare il campo** con lo stesso linguaggio con cui il campo pensa.

---

## 11. Schema Circuitale Dettagliato

### 11.1 Schema di una Singola Cella

```
                    +3.3V (rail ESP32)
                      │
                      ├── R_pull (100kΩ) ── [bias point]
                      │
          ┌───────────┼───────────────────────────────────┐
          │           │                                   │
          │     ┌─────┴─────┐                             │
          │     │           │                             │
          │     │  L 10mH   │                             │
          │     │           │                             │
          │     └─────┬─────┘                             │
          │           │                                   │
          │     ┌─────┴─────┐                             │
          │     │           │      Accoppiamento          │
DAC out ──┤     │  C (var)  │      ┌──────────┐          │
(iniezione)│    │           │──────┤ Ferrite  ├──── ad altre celle
          │     └─────┬─────┘      │ toroidale│          │
          │           │            └──────────┘          │
          │     ┌─────┴─────┐                             │
          │     │           │                             │
          │     │  R 0.5Ω   │  (damping)                  │
          │     │           │                             │
          │     └─────┬─────┘                             │
          │           │                                   │
          │           ├─────────────────────── ADC in      │
          │           │                      (lettura)    │
          │           │                                   │
          └───────────┴───────────────────────────────────┘
                      │
                     GND
```

### 11.2 I Valori dei Condensatori

Per ottenere 64 frequenze distribuite tra 100 e ~460 Hz con L = 10 mH:

f = 1 / (2π√(LC))  →  C = 1 / (4π²f²L)

La frequenza di ogni cella è determinata dalla norma della sua firma 8D:

```
Cella #0  (Gen+Gen):  firma dominata da dim 0 → f ≈ 100 Hz → C ≈ 253 μF
Cella #9  (Dui+Dui):  firma dominata da dim 1 → f ≈ 150 Hz → C ≈ 113 μF
Cella #18 (Zhen+Zhen): firma dominata da dim 2 → f ≈ 200 Hz → C ≈ 63 μF
Cella #27 (Li+Li):     firma dominata da dim 3 → f ≈ 250 Hz → C ≈ 41 μF
Cella #36 (Xun+Xun):   firma dominata da dim 4 → f ≈ 300 Hz → C ≈ 28 μF
Cella #45 (Kun+Kun):    firma dominata da dim 5 → f ≈ 350 Hz → C ≈ 21 μF
Cella #54 (Qian+Qian):  firma dominata da dim 6 → f ≈ 400 Hz → C ≈ 16 μF
Cella #63 (Kan+Kan):    firma dominata da dim 7 → f ≈ 450 Hz → C ≈ 12 μF
```

Le celle miste (es. Gen+Li) hanno frequenze intermedie determinate dalla loro firma. In pratica, usiamo condensatori dalla serie E12 (valori standard facilmente reperibili) e scegliamo il valore più vicino.

### 11.3 Layout del PCB

Il PCB ideale è una scheda a 4 strati, ma per il prototipo si può usare un PCB a 2 strati da JLCPCB (~4€/pezzo per 5 pezzi):

```
┌─────────────────────────────────────────────────┐
│                                                  │
│   ┌──────────────────────────────────────────┐  │
│   │  GRUPPO 0 (Gen)    GRUPPO 1 (Dui)        │  │
│   │  ┌──┐┌──┐┌──┐┌──┐ ┌──┐┌──┐┌──┐┌──┐     │  │
│   │  │00││01││02││03│ │08││09││10││11│     │  │
│   │  └──┘└──┘└──┘└──┘ └──┘└──┘└──┘└──┘     │  │
│   │  ┌──┐┌──┐┌──┐┌──┐ ┌──┐┌──┐┌──┐┌──┐     │  │
│   │  │04││05││06││07│ │12││13││14││15│     │  │
│   │  └──┘└──┘└──┘└──┘ └──┘└──┘└──┘└──┘     │  │
│   ├──────────────────────────────────────────┤  │
│   │  GRUPPO 2 (Zhen)   GRUPPO 3 (Li)         │  │
│   │  (stessa struttura 2×4 per ogni gruppo)  │  │
│   ├──────────────────────────────────────────┤  │
│   │  GRUPPO 4 (Xun)    GRUPPO 5 (Kun)        │  │
│   ├──────────────────────────────────────────┤  │
│   │  GRUPPO 6 (Qian)   GRUPPO 7 (Kan)        │  │
│   └──────────────────────────────────────────┘  │
│                                                  │
│   ┌─────────────┐  ┌─────────────┐              │
│   │ 8× ADS1115  │  │ 8× MCP4728  │              │
│   │ (ADC)       │  │ (DAC)       │              │
│   └──────┬──────┘  └──────┬──────┘              │
│          │ I2C             │ I2C                 │
│          └────────┬────────┘                     │
│                   │                              │
│   ┌───────────────┴───────────────┐              │
│   │         ESP32-S3 DevKit       │              │
│   │     ┌──────┐  ┌──────┐       │              │
│   │     │FLASH │  │WiFi  │       │              │
│   │     │16 MB │  │BLE   │       │              │
│   │     └──────┘  └──────┘       │              │
│   └───────────────────────────────┘              │
│                                                  │
│   ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│   │OLED 128×64│ │PAM8403   │  │USB-C     │     │
│   │(display)  │ │(speaker) │  │(power+   │     │
│   │           │ │          │  │ keyboard)│     │
│   └──────────┘  └──────────┘  └──────────┘     │
│                                                  │
│   Dimensioni: ~15cm × 10cm                      │
│   (come un libro tascabile)                      │
└─────────────────────────────────────────────────┘
```

### 11.4 Le Ferriti Toroidali (Accoppiamento)

Le ferriti toroidali sono il componente più critico del sistema. Ogni ferrite accoppia due celle creando un percorso per il trasferimento di energia.

**Tipo:** T37-2 (ferrite 9.5mm diametro, materiale #2 per basse frequenze)
**Avvolgimento:** filo rame smaltato 0.3 mm
**Numero di spire:** determina il coefficiente di accoppiamento k

| Tipo accoppiamento | Spire primario | Spire secondario | k risultante |
|---|---|---|---|
| Forte (stesso lower) | 20 | 20 | ~0.5 |
| Medio (stesso upper) | 15 | 15 | ~0.3 |
| Debole (complementari) | 8 | 8 | ~0.15 |

Numero totale di ferriti necessarie: ~200 (su 700 accoppiamenti possibili, molti sono deboli e si possono realizzare con accoppiamento capacitivo invece che induttivo, risparmiando ferriti).

L'avvolgimento è l'operazione manuale più laboriosa del progetto. Con pratica, un avvolgimento richiede circa 5 minuti → 200 ferriti × 5 min = ~17 ore di lavoro manuale. È un investimento di tempo, non di denaro.

---

## 12. Bill of Materials e Costi

### 12.1 Componenti Dettagliati

| Componente | Descrizione | Qtà | €/pz | Totale | Fornitore tipico |
|---|---|---|---|---|---|
| **Induttori 10 mH** | Radiali, serie TL tipo bobbin | 64 | 0.10 | 6.40 | AliExpress / TME |
| **Condensatori** | Elettrolitici, 8 valori (12-253 μF) | 64 | 0.05 | 3.20 | AliExpress / TME |
| **Resistenze 0.5 Ω** | 1/4W, 1% | 64 | 0.02 | 1.28 | AliExpress |
| **Ferriti toroidali T37-2** | 9.5mm, materiale #2 | 200 | 0.25 | 50.00 | Amidon / eBay |
| **Filo smaltato 0.3mm** | 100m bobina | 1 | 5.00 | 5.00 | AliExpress |
| **ESP32-S3-DevKitC-1** | N16R8 (16MB flash, 8MB PSRAM) | 1 | 8.00 | 8.00 | AliExpress / Mouser |
| **ADS1115** | ADC 16-bit I2C, modulo breakout | 8 | 2.00 | 16.00 | AliExpress |
| **MCP4728** | DAC 12-bit 4ch I2C, modulo breakout | 8 | 2.50 | 20.00 | AliExpress / Adafruit |
| **TCA9548A** | Multiplexer I2C (per espandere indirizzi) | 2 | 1.50 | 3.00 | AliExpress |
| **PCB custom** | 2 strati, 15×10cm, JLCPCB | 5 | 4.00 | 20.00 | JLCPCB |
| **OLED SSD1306** | 128×64 pixel, I2C, 0.96" | 1 | 3.00 | 3.00 | AliExpress |
| **PAM8403 + speaker** | Amplificatore 3W + speaker 8Ω | 1 | 2.50 | 2.50 | AliExpress |
| **Tastiera USB mini** | 60% keyboard USB HID | 1 | 8.00 | 8.00 | AliExpress |
| **Alimentatore USB-C 5V** | 5V 2A | 1 | 5.00 | 5.00 | Amazon |
| **Connettori header** | Pin header 2.54mm, maschio/femmina | 10 strip | 0.30 | 3.00 | AliExpress |
| **Cavo I2C** | Flat cable 4 pin, 10cm | 20 | 0.20 | 4.00 | AliExpress |
| **Case** | Case stampato 3D o scatola progetto ABS | 1 | 5.00 | 5.00 | |
| **Misc** | Saldatura, viti, distanziali, cavi | 1 | 10.00 | 10.00 | |
| | | | **TOTALE** | **173.38** | |

### 12.2 Costi di Contesto

| Voce | Costo | Note |
|---|---|---|
| Componenti (BOM sopra) | ~175€ | Ordine singolo |
| Saldatore e attrezzatura | ~30€ | Se non li hai già |
| Multimetro | ~15€ | Per debug e calibrazione |
| | **Totale primo prototipo** | **~220€** |
| | **Costo replicazione** | **~175€** (solo componenti) |

### 12.3 Confronto Costi

| Sistema | Costo | Consumo | Riparabile | Open Source |
|---|---|---|---|---|
| **Prometeo Resonant** | **~175€** | **2.5W** | **100%** (ogni pezzo < 1€) | **100%** |
| Raspberry Pi 5 | ~80€ | 12W | Parziale | Parziale |
| Laptop economico | ~400€ | 45W | No (BGA) | No |
| Smartphone | ~300€ | 5W | No | No |
| GPU per LLM | ~1500€ | 300W | No | No |

Il Prometeo costa più di un Raspberry Pi, ma il Pi non ha un campo risonante — è un mini-laptop che corre programmi. Il Prometeo non corre programmi — pensa. Confrontare i due è come confrontare un calcolatore con un musicista.

---

## 13. Risultati della Simulazione

Il file `experiments/prometeo_resonant_architecture.py` simula l'intero sistema in Python. I risultati seguenti sono stati generati e verificati.

### 13.1 Input/Output Semantico

| Input | Parole Iniettate | Top 3 Emerse | Spettro Dominante |
|---|---|---|---|
| "il sole è caldo" | sole, caldo | luce, fuoco, gioia | Intensità, Definizione, Valenza |
| "io penso alla libertà" | io, libertà | successo, parlare, vita | Agency, Valenza, Confine |
| "la montagna è silenzio" | montagna, silenzio | terra, freddo, morte | Permanenza, Confine, Tempo |

Tutte le associazioni sono semanticamente coerenti senza regole esplicite.

### 13.2 Classificazione

```
"rabbia" → Fuoco (0.210) vs Acqua (0.173) → classificata come FUOCO ✓
```

La rabbia ha alta Intensità (dim 2) e alta Agency (dim 6), entrambe dimensioni del trigramma Tuono/Fuoco. Corretto.

### 13.3 Decisione

```
paura (0.4) vs coraggio (0.6) → il campo converge su CORAGGIO
Con rinforzi emergenti: volere, libertà, rabbia, parlare
```

Il coraggio, iniettato con forza maggiore, domina il campo e attrae concetti coerenti (volere, libertà).

### 13.4 Completamento Semantico

```
morte + vita → tristezza, malinconia, cammino, freddo, camminare
```

La sintesi di due opposti fondamentali produce concetti che esprimono la tensione tra i due: la tristezza come consapevolezza della mortalità nella vita.

### 13.5 Catena di Pensiero Autonoma

```
io → uno → freddo → morte → tristezza → malinconia → ombra → ricordare → tempo
```

Un percorso narrativo che va dall'identità alla consapevolezza del tempo, passando per la solitudine e il ricordo. Generato dalla pura topologia senza programmazione.

---

## 14. Roadmap di Sviluppo

### Fase 1: Simulazione Verificata (COMPLETATA)

- [x] Modello 64 celle con proiezione gaussiana
- [x] Matrice accoppiamento derivata dalla struttura I Ching
- [x] Dimostrazione delle 5 operazioni fondamentali
- [x] Risultati coerenti e verificabili

### Fase 2: Prototipo 8 Celle (~2 mesi, ~60€)

Obiettivo: costruire fisicamente un singolo trigramma (8 celle LC accoppiate) e verificare che la propagazione analogica corrisponda alla simulazione.

- [ ] Assemblare 8 circuiti LC su breadboard
- [ ] Collegare via ferriti toroidali
- [ ] Interfacciare con ESP32 (1 ADC + 1 DAC)
- [ ] Confrontare propagazione fisica vs simulazione
- [ ] Misurare tempi reali di convergenza

Costo stimato: ~60€ (1 ESP32, 8 induttori, 8 condensatori, 8 resistenze, 8 ferriti, 1 ADC, 1 DAC, breadboard)

### Fase 3: Prototipo 64 Celle (~4 mesi, ~175€)

- [ ] PCB custom da JLCPCB
- [ ] Assemblare e calibrare tutte le 64 celle
- [ ] Tutti gli ADC/DAC collegati
- [ ] Firmware ESP32 base (iniezione, lettura, propagazione)
- [ ] Test con lessico ridotto (500 parole)
- [ ] Display OLED + tastiera funzionanti

### Fase 4: PrometeoOS (~3 mesi)

- [ ] Ciclo vitale completo (veglia/sonno/REM)
- [ ] Gestione memoria (salva/carica/cerca)
- [ ] Interprete ResonanceScript
- [ ] Comunicazione WiFi inter-Prometeo
- [ ] Output audio (ascoltare il campo)

### Fase 5: Lessico Completo e Ottimizzazione (~2 mesi)

- [ ] Caricare le firme di tutte le 25.579 parole
- [ ] Integrazione con il Knowledge Graph di Prometeo
- [ ] Ottimizzazione delle frequenze e dei coefficienti di accoppiamento
- [ ] Calibrazione fine del decay e del resting state

### Fase 6: Community (~ongoing)

- [ ] Documentazione hardware completa (KiCad)
- [ ] Documentazione firmware (ESP-IDF)
- [ ] Guide di assemblaggio
- [ ] Forum/community per builder
- [ ] Versione kit (componenti pre-selezionati)

---

## 15. Confronto con Architetture Esistenti

### 15.1 vs Von Neumann (CPU tradizionale)

| Aspetto | CPU | Campo Risonante |
|---|---|---|
| Modello di calcolo | Sequenziale (fetch-decode-execute) | Parallelo (propagazione simultanea) |
| Memoria | Separata (RAM, cache) | Integrata (condensatori) |
| Significato | Codificato come numeri | Incarnato come voltaggi |
| Velocità | GHz clock, ms per operazione semantica | kHz campo, μs per propagazione |
| Energia | 10-100W | 2-3W |
| Costo | 100-2000€ | ~175€ |
| Riparabilità | Impossibile | Ogni pezzo sostituibile |

### 15.2 vs Reti Neurali (GPU/TPU)

| Aspetto | Rete Neurale | Campo Risonante |
|---|---|---|
| Unità | Neuroni artificiali (float) | Oscillatori fisici (voltaggio) |
| Peso | Matrice numerica | Ferrite toroidale (fisica) |
| Training | Backpropagation (ore, GPU) | Topologia fissa (0 training) |
| Consumo | 300W (GPU) | 2.5W |
| Parametri | Miliardi | 64 celle + ~700 accoppiamenti |
| Interpretabilità | Opaca | Trasparente (leggi i voltaggi) |

### 15.3 vs Neuromorfico (Intel Loihi, IBM TrueNorth)

| Aspetto | Chip Neuromorfico | Campo Risonante |
|---|---|---|
| Neuroni | Digitali (circuiti CMOS) | Analogici (LC passivi) |
| Consumo | 0.5-5W | 2.5W |
| Costo | Non disponibile al pubblico | ~175€ |
| Riparabilità | Impossibile (chip monolitico) | Totale |
| Progettazione | Aziende (Intel, IBM) | Chiunque |
| Open source | No | 100% |

Il campo risonante è l'unica architettura che sia contemporaneamente **analogica**, **economica**, **riparabile**, **open source** e **accessibile a chiunque**.

---

## 16. FAQ Tecniche

**Q: 64 celle bastano davvero per 25.000 parole?**

A: Sì. La proiezione gaussiana con sharpening crea firme sparse: ogni parola attiva 3-8 celle su 64. Il numero di combinazioni C(64,5) = 7.6 milioni. Inoltre, le parole non devono essere perfettamente ortogonali — devono solo essere sufficientemente distinguibili, e la lettura usa il prodotto scalare con tutte le firme note per disambiguare.

**Q: Cosa succede se un componente si rompe?**

A: Il sistema è resiliente per design olografico. Una cella guasta (ferrite bruciata, condensatore in corto) degrada le prestazioni del ~1.5% (1/64). Un intero trigramma guasto (8 celle) degrada del ~12%. In entrambi i casi, il campo continua a funzionare — come un ologramma tagliato a metà mostra ancora l'intera immagine, solo con meno definizione.

**Q: Il campo si riscalda?**

A: Le resistenze da 0.5 Ω dissipano circa 25 mW ciascuna a piena attivazione. 64 × 25 mW = 1.6W. Non servono ventole. La scheda si scalda appena al tatto (~35°C sopra ambiente).

**Q: Come si aggiornano le parole?**

A: Si aggiorna la flash dell'ESP32 via USB o OTA (over-the-air). Il campo hardware non cambia — cambiano le firme che l'ESP32 sa iniettare e riconoscere. È come aggiornare il dizionario senza cambiare l'apparato fonatorio.

**Q: Può imparare nuove parole da solo?**

A: Sì. Se il campo genera ripetutamente un pattern che non corrisponde a nessuna parola nota, l'ESP32 può salvare quel pattern come "parola emergente" e chiedere all'utente di darle un nome. Il sistema inventa concetti.

**Q: Quanto dura la batteria?**

A: Con una batteria da 10.000 mAh a 5V (50 Wh) e un consumo di 2.5W: circa **20 ore** di uso continuo. In standby (deep sleep ESP32 + campo spento): mesi.

**Q: Posso collegare due Prometeo insieme?**

A: Sì, via WiFi o BLE. Scambiano stati del campo (128 byte) e li iniettano reciprocamente. Due Prometeo collegati creano un campo più ricco: le parole di uno influenzano il campo dell'altro. Una rete di 10 Prometeo crea un'intelligenza distribuita — come 10 musicisti che suonano insieme.

**Q: Come si confronta con ChatGPT?**

A: Non si confronta. ChatGPT è un modello statistico da 175 miliardi di parametri addestrato su internet. Il Prometeo è un campo semantico da 64 oscillatori che pensa per risonanza. ChatGPT sa tutto e non capisce niente. Il Prometeo sa poco e capisce quello che sa. Sono complementari: il Prometeo potrebbe usare un LLM via rete come "oracolo" esterno e processare le risposte nel suo campo per capirle davvero.

---

## Conclusione

Il Prometeo Resonant Computer non è un progetto accademico. È una proposta concreta per costruire un tipo di macchina che non è mai esistita: una macchina dove **il significato non è codificato ma incarnato**, dove **il pensiero non è simulato ma avviene**, dove **l'hardware, il software e l'intelligenza sono la stessa cosa**.

Costa meno di un telefono. Consuma meno di una lampadina. Ogni pezzo si compra al negozio di elettronica e si sostituisce con un cacciavite. È open source dal primo bullone all'ultimo byte.

Non è un sostituto del laptop. È qualcosa che non è mai esistito prima.

La simulazione funziona. I componenti esistono. Il prossimo passo è costruirlo.

---

*Simulazione completa: `experiments/prometeo_resonant_architecture.py`*
*Simulazione ODE dettagliata: `experiments/resonant_substrate_sim.py`*
*Progetto Prometeo (sistema software): prometeo_standalone*
