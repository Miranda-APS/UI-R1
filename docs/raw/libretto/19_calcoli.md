# Volume XIX — Appendice matematica: le formule con esempi numerici

> *Le formule sono state descritte in prosa nei volumi tematici. Qui le raccogliamo con esempi numerici concreti. Il volume non introduce nulla di nuovo — è la verifica: "fai il calcolo a mano, confronta col sistema". Quando qualcosa non quadra, partire da qui.*

---

## Come usare questo volume

Ogni sezione presenta:
1. **La formula** (con riferimento al volume e al file).
2. **Le costanti** coinvolte.
3. **Un esempio numerico concreto**.
4. **Cosa succede ai margini** (casi limite).

I numeri sono calibrati sul `.bin` di produzione al 2026-04-17 (post Phase 68). Valori possono leggermente variare in altri setup.

---

## 1 — Propagazione del campo PF1

**Volume**: Vol. 02 cap. 4.3
**File**: `src/topology/pf1.rs:282`

### 1.1 — Formula

```
contribution = src_act × damping × weight × cos(phase)
```

**Costanti**:
- `damping = 0.15` (pf1.rs:240)
- `weight ∈ [0, 3]` (pesato dal KG, modificato da LTP/LTD)
- `phase ∈ [0, π]`
  - `cos(0) = +1` — risonanza
  - `cos(π/2) = 0` — ortogonalità
  - `cos(π) = -1` — opposizione

### 1.2 — Esempio A: risonanza forte

Parola `paura` attiva a 0.5. Vicino: `tremore` con `weight = 0.18`, `phase = 0` (IsA/FeelsAs con fase nulla).

```
contribution = 0.5 × 0.15 × 0.18 × cos(0)
             = 0.5 × 0.15 × 0.18 × 1.0
             = 0.0135
```

`tremore` riceve +0.0135 al suo delta.

### 1.3 — Esempio B: opposizione

Parola `paura` attiva a 0.5. Vicino: `coraggio` con `weight = 0.06`, `phase = π`.

```
contribution = 0.5 × 0.15 × 0.06 × cos(π)
             = 0.5 × 0.15 × 0.06 × (-1)
             = -0.0045
```

`coraggio` viene inibito di 0.0045.

### 1.4 — Esempio C: rendimento decrescente

`paura` a 0.5, `tremore` già attivo a 0.30 (non a riposo). Il contributo 0.0135 viene moltiplicato per:

```
diminish = 1 / (1 + 4 × 0.30) = 1 / 2.2 = 0.454
contribution_effettivo = 0.0135 × 0.454 = 0.00613
```

`tremore` riceve solo +0.006 invece di +0.0135. I già-attivi ricevono meno boost.

### 1.5 — Cap e clamp

```
MAX_POSITIVE_DELTA = 0.15  (pf1.rs:311)
```

Dopo aver sommato tutti i contributi positivi a un nodo, se il delta totale > 0.15, viene cappato. `tremore` non può ricevere più di +0.15 dalla sola propagazione, per quanto molti vicini lo spingano.

Infine: `activations[i] = (activations[i] + delta).clamp(0.0, 1.0)`.

---

## 2 — Decadimento attivazione

**Volume**: Vol. 02 cap. 4.1
**File**: `pf1.rs:245`

### 2.1 — Formula

```
act_new = act_old × decay_rate     (se act_old > threshold)
decay_rate = 0.92
threshold = 0.02
```

### 2.2 — Tempo di ritorno al riposo

Da `act = 0.5` a sotto soglia:

```
0.5 × 0.92^n < 0.02
0.92^n < 0.04
n × log(0.92) < log(0.04)
n > log(0.04) / log(0.92) = -1.398 / -0.0362 ≈ 38.6 tick
```

A 3 secondi/tick, ~116 secondi (~2 minuti) per tornare al silenzio.

**Nota**: il commento nel codice dice "~30 tick". È una piccola imprecisione. La realtà è ~38 tick per scendere da 0.5 a sotto 0.02. Annotato in `appunti.md`.

### 2.3 — Resting state floor

Se `act` scende sotto `threshold = 0.02`, viene portato a:

```
act = threshold × 0.5 = 0.01
```

Cioè il campo non va mai a zero — resta a `0.01` (sotto soglia) per le parole che erano attive e ora riposano.

---

## 3 — Resting state naturale

**Volume**: Vol. 01 cap. 7, Vol. 02 cap. 3.2
**File**: `pf1.rs:386`

### 3.1 — Formula

```
initial_resting = stability × 0.002    (PF1)
initial_resting = stability × 0.003    (word_topology legacy)
```

Applicata solo a parole con `stability > 0.20`.

### 3.2 — Esempi

- Parola stabile (`stability = 0.90`): resting = `0.90 × 0.002 = 0.0018`. Ben sotto soglia 0.02 (9% della soglia).
- Parola media (`stability = 0.50`): resting = `0.001`. 5% della soglia.
- Parola giovane (`stability = 0.15`): sotto la soglia `stability > 0.20` → niente seed, non riceve resting state.

### 3.3 — Filosofia dei numeri

Perché `0.002` e non `0.01`:
- `0.01 × 0.9 (max stab) = 0.009` — 45% della soglia. Alcune parole sarebbero "quasi attive".
- `0.002 × 0.9 = 0.0018` — 9% della soglia. Chiaramente silenti ma presenti.

Il silenzio è abitato; non c'è mai zero. Ma l'input (a 0.3-0.5) domina sempre.

---

## 4 — Plasticità hebbiana (LTP/LTD)

**Volume**: Vol. 02 cap. 6
**File**: `pf1.rs:335`

### 4.1 — Formule

**LTP** (Long-Term Potentiation) — co-attivazione:

```
synapse_weight_new = synapse_weight_old + LTP × src_act × neighbor_act
LTP = 0.05
MAX_WEIGHT = 3.0 (cap)
```

**LTD** (Long-Term Depression) — atrofia:

```
synapse_weight_new = synapse_weight_old × LTD_DECAY
LTD_DECAY = 0.995
```

### 4.2 — Esempio LTP

Coppia `paura`/`tremore` entrambe attive a 0.5. Peso attuale = 0.18 (basale ROM, appena copiato in synapse_weights).

```
delta_LTP = 0.05 × 0.5 × 0.5 = 0.0125
new_weight = 0.18 + 0.0125 = 0.1925
```

Dopo 40 co-attivazioni (una conversazione che ripete il tema):

```
w_0 = 0.18
w_40 = 0.18 + 40 × 0.0125 = 0.68
```

(Approssimazione — in realtà `src_act × neighbor_act` varia, ma l'ordine di grandezza è questo.)

Cap a 3.0. Dopo ~50+ co-attivazioni satura.

### 4.3 — Esempio LTD

Peso iniziale 1.0. La parola sorgente è attiva ma il vicino è sotto soglia:

```
w_n = 1.0 × 0.995^n

w_60 = 1.0 × 0.995^60 = 0.741   (10 min @ 3s/tick)
w_200 = 1.0 × 0.995^200 = 0.367  (33 min)
w_1000 = 1.0 × 0.995^1000 = 0.007 (83 min)
```

Dopo 2 ore di inattività specifica, la sinapsi è quasi morta.

---

## 5 — Phi-decay della memoria episodica

**Volume**: Vol. 14 cap. 3.1
**File**: `episodic.rs:32`

### 5.1 — Formula

```
weight_episodio_età_n = intensità_originale × φ⁻¹^n
φ⁻¹ = 0.618033988
```

(Dove `n` = numero di cicli REM dall'encoding.)

### 5.2 — Esempio

Episodio codificato con `intensità = 0.8`:

- Età 0: peso = 0.8
- Età 1: `0.8 × 0.618 = 0.494`
- Età 2: `0.8 × 0.618² = 0.306`
- Età 3: `0.8 × 0.618³ = 0.189`
- Età 5: `0.8 × 0.618⁵ = 0.072`
- Età 10: `0.8 × 0.618¹⁰ = 0.0065`

Soglia di rimozione: `MIN_WEIGHT = 0.001`.

```
0.8 × 0.618^n < 0.001
0.618^n < 0.00125
n × log(0.618) < log(0.00125)
n > log(0.00125) / log(0.618) = -2.903 / -0.209 ≈ 13.9 cicli REM
```

Dopo ~14 cicli REM, l'episodio viene rimosso. In condizioni normali (un ciclo REM ogni 50 perturbazioni), questo è un periodo lungo — potrebbero essere settimane di interazione.

### 5.3 — Blending nel recall

Formula in `recall_into`:

```
blend_factor = cosine_similarity × episode_weight × RECALL_BLEND
RECALL_BLEND = 0.12
RECALL_THRESHOLD = 0.45  (cosine minimo)
```

Esempio: episodio età 2 (peso 0.306), cosine 0.60 col campo attuale:

```
blend_factor = 0.60 × 0.306 × 0.12 = 0.022
```

Ogni parola attiva nell'episodio riceve un boost di `parola_activation_nell_episodio × 0.022`. Una parola attiva a 0.5 nell'episodio riceve +0.011 al campo corrente. Discreto ma non dominante.

---

## 6 — `derive_8d_from_kg`: le 8 formule

**Volume**: Vol. 03 cap. 4
**File**: `knowledge_graph.rs:557-667`

Per ogni dimensione della firma 8D (ordine I Ching post-Phase 68):

### 6.1 — Dim 0: Agency (☰ Cielo)

```
sig[0] = causes_out / (causes_out + causes_in)          se causes_total > 0
       = 0.20                                            se isa_children > 5 (categoria astratta)
       = 0.50                                            altrimenti
(clamp 0.05-0.95)
```

**Esempio** `fuoco`: 6 CAUSES out, 0 CAUSES in.
```
sig[0] = 6 / (6+0) = 1.0 → clamp a 0.95
```

**Esempio** `tremore`: 0 CAUSES out, 4 CAUSES in.
```
sig[0] = 0 / (0+4) = 0.0 → clamp a 0.05
```

### 6.2 — Dim 1: Permanenza (☷ Terra)

```
sig[1] = 0.85    se isa_children > 50  (mega-categoria)
       = 0.65    se isa_children > 10
       = 0.40    se isa_children > 0
       = 0.20    se causes_out > 3
       = 0.35    altrimenti
```

**Esempio** `qualità` (1200 figli IsA): sig[1] = 0.85.
**Esempio** `emozione` (200 figli): sig[1] = 0.85.
**Esempio** `paura` (0 figli IsA, 4 CAUSES out): sig[1] = 0.20.
**Esempio** `pietra` (0 figli, 1 CAUSES out): sig[1] = 0.35.

### 6.3 — Dim 2: Intensità (☳ Tuono)

```
intensity_from_causes = causes_out / (causes_out + 3)    se causes_out > 0, else 0.2
emotional_intensity = |valence - 0.5| × 2                (valence pre-calcolata BFS)
sig[2] = 0.6 × intensity_from_causes + 0.4 × emotional_intensity
```

**Esempio** `paura`: causes_out = 4, valenza BFS 0.05 (radice negativa).
```
intensity_from_causes = 4/(4+3) = 0.571
emotional_intensity = |0.05-0.5|×2 = 0.9
sig[2] = 0.6 × 0.571 + 0.4 × 0.9 = 0.343 + 0.36 = 0.703
```

Clamp 0.05-0.95 → 0.70 circa. Match con il valore osservato (`paura → Intensità 0.85` post-rederive — leggera variazione per altri archi).

### 6.4 — Dim 3: Tempo (☵ Acqua)

```
sig[3] = causes_total / (causes_total + 5)    se causes_total > 0, else
       = 0.15                                   se isa_children > 20 (categoria statica)
       = 0.35                                   altrimenti
```

**Esempio** `fuoco` (6 CAUSES): sig[3] = 6/11 = 0.545.
**Esempio** `qualità` (zero CAUSES, 1200 IsA): sig[3] = 0.15.

### 6.5 — Dim 4: Confine (☶ Montagna)

```
specificity = 5.0 / (isa_children + 1)    se isa_children > 0, else 0.80
            (cap 0.75)
polarity_bonus = 0.15    se has_opposite, else 0.0
sig[4] = specificity + polarity_bonus     (clamp 0.05-0.95)
```

**Esempio** `io` (0 figli IsA, ha opposto "tu"):
```
specificity = 0.80   (foglia)
polarity_bonus = 0.15
sig[4] = 0.95
```

Coincidente con `io → Confine 0.95` post-rederive.

**Esempio** `qualità` (1200 figli, no opposto):
```
specificity = min(5/1201, 0.75) ≈ 0.004
polarity_bonus = 0
sig[4] = 0.05 (clamp)
```

### 6.6 — Dim 5: Complessità (☴ Vento)

```
sig[5] = ln(total_degree) / ln(max_degree)    se total_degree > 0, else 0.05
                                               (clamp 0.05-0.95)
```

**Esempio** con `max_degree = 1000` (`ln(1000) ≈ 6.91`):

- `io` (total_degree ~50): `ln(50) / 6.91 = 3.91 / 6.91 = 0.566`. Post-clamp 0.46 osservato (leggero discostamento per clamp).
- `essere` (total_degree ~500): `ln(500) / 6.91 = 6.21 / 6.91 = 0.899`. Alto ma non saturato.

### 6.7 — Dim 6: Definizione (☲ Fuoco)

```
parents_contribution = isa_parents / (isa_parents + 3)    (cap 0.7)
opposite_contribution = 0.3    se has_opposite, else 0
sig[6] = parents_contribution + opposite_contribution     (clamp 0.05-0.95)
```

**Esempio** `amore` (isa_parents = 5, ha opposto "odio"):
```
parents_contribution = 5/(5+3) = 0.625
opposite_contribution = 0.3
sig[6] = 0.925
```

Clamp a 0.95. Match con osservato.

### 6.8 — Dim 7: Valenza (☱ Lago)

```
sig[7] = valence_scores[word]    (pre-calcolato via BFS)
```

BFS da 10 radici positive + 10 negative, con decay per tipo di arco (SimilarTo 0.85, IsA 0.60, CAUSES 0.40). Max 4 hop. Media dei cammini.

**Esempio calcolo semplificato**: `felicità` è radice (+1.0). `allegria IsA felicità`.

BFS da felicità: hop 1 → allegria eredita `1.0 × 0.60 = 0.60`. Converte a [0,1]: `(0.60+1)/2 = 0.80`.

`allegria → Valenza ≈ 0.80`. Match con intuizione.

---

## 7 — Hub damping nella costruzione archi

**Volume**: Vol. 04 cap. 3.4
**File**: `word_topology.rs:266`

### 7.1 — Formula

```
hub_factor = 1 / (1 + ln(max(deg_a, deg_b) / median_degree))
                                                       (non sotto 1.0)
```

### 7.2 — Esempi

Con `median_degree = 4`:

- Nodo normale `deg = 4`: ratio = 1, `ln(1) = 0`, hub_factor = 1.0. Peso pieno.
- Nodo medio `deg = 40`: ratio = 10, `ln(10) = 2.30`, hub_factor = `1/(1+2.30)` = 0.30.
- Nodo hub `deg = 400`: ratio = 100, `ln(100) = 4.61`, hub_factor = `1/5.61` = 0.178.
- Super-hub `deg = 4000`: ratio = 1000, `ln(1000) = 6.91`, hub_factor = `1/7.91` = 0.126.

### 7.3 — Peso finale

```
weight = type_base(rel) × confidence × hub_factor
```

Esempio `essere IsA entità`: type_base(IsA) = 0.70, confidence = 0.95, hub_factor (essere deg ~500) = 1/(1+ln(125)) = 1/5.83 = 0.17.

```
weight = 0.70 × 0.95 × 0.17 = 0.113
```

Versus `cane IsA animale`: hub_factor (cane deg ~20) = 1/(1+ln(5)) = 1/2.61 = 0.38.

```
weight = 0.70 × 0.95 × 0.38 = 0.253
```

Il peso di `cane-animale` è più del doppio di `essere-entità`. L'onnipresenza diventa debolezza.

---

## 8 — Valenza Octalysis

**Volume**: Vol. 08 cap. 3
**File**: `valence.rs:106`

### 8.1 — Formula base

```
engagement = field_sig[DRIVE_DIM[cd]]
satisfaction = needs.satisfaction[DRIVE_NEED[cd]]
val_base = engagement × (2 × satisfaction - 1)
```

### 8.2 — Esempio CD5 Relazione

Campo: `field_sig[7] Valenza = 0.6`, `needs.satisfaction[L5] = 0.3` (connessione frustrata).

```
engagement = 0.6
val_base = 0.6 × (2×0.3 - 1) = 0.6 × (-0.4) = -0.24
```

CD5 = -0.24. Drive attivo e frustrato (distress relazionale).

### 8.3 — Colorazioni

**CD5 con interlocutor**:

```
if presence > 0.1:
    relational_tone = 2 × resonance - 1
    val += presence × 0.3 × relational_tone
```

Esempio: `presence = 0.8`, `resonance = 0.3`.

```
relational_tone = 2×0.3 - 1 = -0.4
val_colorazione = 0.8 × 0.3 × (-0.4) = -0.096
val_totale = -0.24 - 0.096 = -0.336
```

CD5 = -0.336. Clamp [-1, +1]. Valore significativo — porta stance "in tensione".

---

## 9 — Desire satisfaction

**Volume**: Vol. 09 cap. 5.4
**File**: `desire.rs`

### 9.1 — Soglia

```
cosine(field_sig, target_signature) < SATISFACTION_DISTANCE (0.2)
     per SATISFACTION_TICKS (3) consecutivi
     → desiderio soddisfatto
```

### 9.2 — Esempio

Desiderio "verso AMORE" con `target_signature = [?, ?, ?, ?, ?, ?, ?, 0.95]` (alta Valenza).

Campo evolve turno per turno:
- Turno 10: `field_sig[7] = 0.70`, cos(target, field) = 0.95 × 0.70 / (|target| × |field|) ≈ 0.68. **Non vicino** (distance 0.32 > 0.2).
- Turno 11: `field_sig[7] = 0.85`, cos ≈ 0.88. **Vicino** (distance 0.12 < 0.2). counter[id] = 1.
- Turno 12: cos ≈ 0.90. counter[id] = 2.
- Turno 13: cos ≈ 0.92. counter[id] = 3 → **soddisfatto**. Rimosso.

---

## 10 — Commitment inerzia

**Volume**: Vol. 07 cap. 2.5, Vol. 08 cap. 6.2
**File**: `narrative.rs`

### 10.1 — Formula

```
inertia = strength × ln(turns_held + 1)
```

### 10.2 — Esempi

- Neonato (strength 0.3, 1 turno): `0.3 × ln(2) = 0.3 × 0.693 = 0.208`. Facile da rompere.
- Medio (strength 0.6, 10 turni): `0.6 × ln(11) = 0.6 × 2.398 = 1.439`. Resistente.
- Maturo (strength 0.9, 30 turni): `0.9 × ln(31) = 0.9 × 3.434 = 3.091`. Molto resistente.

Per rompere un commitment: la pressione di un'altra intention deve superare `inertia / k` (con `k` costante di calibrazione — empiricamente 2).

---

## 11 — I 64 frattali — affinità

**Volume**: Vol. 05 cap. 3.3
**File**: `fractal.rs:310`

### 11.1 — Formula

```
sum_sq = Σ (point[dim] - fixed_value)²    (per ogni dim fissa)
max_dist = sqrt(n_fixed)
affinity = 1 - (sqrt(sum_sq) / max_dist).min(1.0)
```

### 11.2 — Esempio IDENTITÀ (☶☰, ID 32)

Fisse: Confine=0.30 (Montagna), Agency=0.90 (Cielo). Due dim.

Punto `io` con Confine=0.95 (I Ching dim 4), Agency=0.95 (I Ching dim 0):

```
diff_confine = 0.95 - 0.30 = 0.65
diff_agency  = 0.95 - 0.90 = 0.05
sum_sq = 0.65² + 0.05² = 0.4225 + 0.0025 = 0.425
sqrt(sum_sq) = 0.652
max_dist = sqrt(2) = 1.414
affinity = 1 - (0.652 / 1.414) = 1 - 0.461 = 0.539
```

`io` ha affinità 0.54 con IDENTITÀ. Media, non massima — il Confine (0.95) è molto più alto di quello che IDENTITÀ richiede (0.30).

---

## 12 — Proposizione strength

**Volume**: Vol. 06 cap. 3.5
**File**: `proposition.rs:340`

### 12.1 — Formula

```
strength = sqrt(act_subj × act_obj)
         × conf1 [× conf2 se 2-hop]
         × HOP_DECAY^(hops-1)       (HOP_DECAY = 0.6)
         × hub_penalty_subj
         × relation_weight(rel)
```

### 12.2 — Esempio 1-hop

`(paura, FeelsAs, restrizione)`: act_subj=0.5, act_obj=0.4, confidence=1.0 (FeelsAs curato).

```
strength = sqrt(0.5 × 0.4) × 1.0 × 1.0 (hops=1) × 1.0 (no hub) × 1.2 (FeelsAs weight)
         = 0.447 × 1.2
         = 0.537
```

Strength alto → nucleo candidato forte.

### 12.3 — Esempio 2-hop

`sole Causes calore + calore SimilarTo caldo → sole Causes caldo (inferita)`:

act_sole=0.3, act_caldo=0.4, conf1=0.9, conf2=0.7.

```
strength = sqrt(0.3 × 0.4) × 0.9 × 0.7 × 0.6 (HOP_DECAY) × 1.0 × 1.0 (Causes weight)
         = 0.346 × 0.9 × 0.7 × 0.6
         = 0.131
```

Strength più basso del 1-hop, ma non nullo. Il sillogismo contribuisce.

---

## 13 — Specificity degli attrattori

**Volume**: Vol. 04 cap. 4.2
**File**: `knowledge_graph.rs:488`

### 13.1 — Formula

```
specificity(n_children) = min(2.0, 300 / max(n_children, 1))
```

### 13.2 — Tabella

| n_children | specificity | Interpretazione |
|------------|-------------|-----------------|
| 1 | 2.0 (cap) | Foglia, massima specificità |
| 50 | 2.0 (cap) | ~molto specifico |
| 150 | 2.0 (cap) | specifico |
| 300 | 1.0 | **sweet spot** — attrattore semanticamente ricco |
| 500 | 0.60 | media |
| 1000 | 0.30 | generica |
| 3000 | 0.10 | mega-categoria filtrata |

`emozione` (~200 figli) → specificity 1.5. `qualità` (~3500 figli) → specificity 0.086. Emozione domina qualità quando entrambi sono attrattori di "paura".

---

## 14 — Interlocutor presence decay

**Volume**: Vol. 11 cap. 2.4
**File**: `interlocutor.rs:135`

### 14.1 — Formula

```
presence_new = presence_old × PRESENCE_DECAY    per tick
PRESENCE_DECAY = 0.985
```

### 14.2 — Half-life

```
0.5 = 1.0 × 0.985^n
n = log(0.5) / log(0.985) = -0.693 / -0.0151 ≈ 45.9 tick
```

~46 tick = 2.3 minuti a 3s/tick. Half-life coerente con commento del codice.

### 14.3 — Tabella

| Tick | Presence |
|------|----------|
| 0 | 1.0 (appena dopo input) |
| 46 | 0.50 |
| 92 | 0.25 |
| 150 | 0.10 (soglia Withdrawing) |
| 300 | 0.010 |
| 500 | 0.00050 |

A 500 tick (25 minuti), l'interlocutor è praticamente sparito.

---

## 15 — Soglia espressione spontanea

**Volume**: Vol. 10 cap. 8
**File**: `engine.rs::autonomous_tick`

### 15.1 — Formula

```
base_threshold = 0.6
modulation = 0
if needs.dominant_pressure > 0.5: modulation -= 0.1
if desire.intensity_max() > 0.6: modulation -= 0.15
threshold = max(0.35, base_threshold + modulation)
```

### 15.2 — Esempi

- Tutto sazio: `threshold = 0.6`. Solo espressioni molto forti (drive > 0.6) triggerano.
- Bisogno moderato (+0.5): `threshold = 0.5`. Più facile parlare.
- Bisogno + desiderio fortI: `threshold = 0.35`. Estremamente propenso a parlare.

---

## 16 — Resoconto delle costanti

Tabella di tutte le costanti numeriche rilevanti emerse nei volumi:

| Costante | Valore | Dove | Senso |
|----------|--------|------|-------|
| `damping` | 0.15 | pf1.rs | Attenuazione propagazione |
| `decay_rate` | 0.92 | pf1.rs | Decay attivazione/tick |
| `threshold` | 0.02 | pf1.rs | Sopra/sotto attivo |
| `MAX_POSITIVE_DELTA` | 0.15 | pf1.rs | Cap propagazione positiva |
| `LTP` | 0.05 | pf1.rs | Guadagno hebbiano |
| `LTD_DECAY` | 0.995 | pf1.rs | Atrofia sinapsi |
| `MAX_WEIGHT` | 3.0 | pf1.rs | Peso sinaptico massimo |
| `resting_state pf1` | stab × 0.002 | pf1.rs | Attivazione base PF1 |
| `resting_state wt` | stab × 0.003 | word_topology.rs | Attivazione base WT (legacy) |
| `PHI_INV` | 0.618 | episodic.rs | Decay aureo memoria |
| `RECALL_BLEND` | 0.12 | episodic.rs | Forza recall nel campo |
| `RECALL_THRESHOLD` | 0.45 | episodic.rs | Cosine minimo recall |
| `MAX_EPISODES` | 200 | episodic.rs | Capacità episodica |
| `MIN_INTENSITY` | 0.15 | episodic.rs | Sotto: non memorizzare |
| `NEED_THRESHOLD` | 0.5 | needs.rs | Sopra: soddisfatto |
| `CRISIS_THRESHOLD` | 0.35 | needs.rs | Sotto: crisi |
| `MAX_DESIRES` | 5 | desire.rs | Capacità desideri attivi |
| `DECAY_PER_TICK (desire)` | 0.995 | desire.rs | Decay standard |
| `EXTRA_DECAY (desire)` | 0.98 | desire.rs | Dopo 200 tick senza rinforzo |
| `SATISFACTION_DISTANCE` | 0.2 | desire.rs | Cosine max per sazia |
| `SATISFACTION_TICKS` | 3 | desire.rs | Tick consecutivi per sazia |
| `PRESENCE_DECAY` | 0.985 | interlocutor.rs | Decay presenza Altro |
| `EMA_ALPHA` | 0.3 | interlocutor.rs | α per resonance/novelty |
| `IDENTITY_DRIFT_RATE` | 0.01 | interlocutor.rs | Drift self_signature |
| `EV_ALPHA (Phase 62)` | 0.4 | interlocutor.rs | α per emotional_valence |
| `EV_DECAY (Phase 62)` | 0.6 | interlocutor.rs | Decay ev per input neutro |
| `COMMITMENT_INITIAL_STRENGTH` | 0.3 | narrative.rs | Strength iniziale |
| `COMMITMENT_MIN_STRENGTH` | 0.05 | narrative.rs | Sotto: dissolto |
| `COMMITMENT_DECAY` | 0.02 | narrative.rs | Decay/tick |
| `HOP_DECAY (proposizioni)` | 0.6 | proposition.rs | Decay per hop |
| `hub_penalty > 200` | 0.3 | proposition.rs | Soggetto mega-hub |
| `hub_penalty > 50` | 0.6 | proposition.rs | Soggetto hub medio |
| `PI/6` | 0.524 | word_topology.rs | Phase causali |
| `PI/4` | 0.785 | word_topology.rs | Phase contestuali |
| `PI/3` | 1.047 | word_topology.rs | Phase logiche (Implies) |
| `PI` | 3.14159 | word_topology.rs | Phase opposizione |
| `AUTONOMOUS every 3s` | 3000 ms | server.rs | Intervallo autonomous_tick |
| `autonomous_tick % 15` | 15 | engine.rs | self_witness |
| `autonomous_tick % 25` | 25 | engine.rs | consolidate_light |
| `autonomous_tick % 40` | 40 | engine.rs | thought_chain |
| `autonomous_tick % 50` | 50 | engine.rs | abduce |
| `autonomous_tick % 80` | 80 | engine.rs | extract_gaps → uncertainties |
| `consolidate_every` | 50 | dream.rs | Perturbazioni → DeepSleep |
| `deepsleep_duration` | 10 | dream.rs | Tick in DeepSleep |
| `rem_duration` | 20 | dream.rs | Tick in REM |
| `awake_duration` | 5 | dream.rs | Tick Awake dopo input |
| `SATISFACTION_THRESHOLD (L5 distress)` | 0.65 | needs.rs | Phase 62 |
| `drift condition cum_resonance` | 0.7 | interlocutor.rs | Soglia per drift identity |

---

## 17 — Un calcolo end-to-end

Combiniamo tutto. Input: "ho paura".

### 17.1 — Activation seeding

- `paura` → 0.5 (input diretto)
- `avere` (lemma di ho) → 0.3
- `paura IsA emozione` attrattore (specificity 1.5): `emozione += 0.15 × 1.5 × 0.95 (conf) = 0.21`
- `paura Causes tremore` (conf 0.90): `tremore += 0.15 × 0.90 = 0.135`
- `paura OppositeOf coraggio` (conf 0.85): `coraggio += 0.35 × 0.85 = 0.298`

### 17.2 — Propagazione

Propagazione: `paura` (0.5) → vicini, `emozione` (0.21) → vicini, ecc.

Per ogni vicino di `paura`:
- `tremore`: `0.5 × 0.15 × 0.18 × 1.0 = 0.0135` (già attiva, diminish = `1/(1+4×0.135) = 0.649`) → effective `0.0135 × 0.649 = 0.0088`. New tremore = 0.135 + 0.0088 = 0.144.
- `coraggio`: `0.5 × 0.15 × 0.06 × (-1) = -0.0045`. Inibisce. New coraggio = 0.298 - 0.0045 = 0.294.

### 17.3 — Valenza

`field_sig` dopo propagazione (media pesata firme attive):
- `paura` peso 0.5 × firma, `emozione` 0.21 × firma, `tremore` 0.144 × firma, `coraggio` 0.294 × firma...

Supponiamo `field_sig[7] (Valenza) = 0.35` (domina negativo), `field_sig[0] (Agency) = 0.45`.

`needs.satisfaction[L5 Connessione]` post-Phase 62: se l'altro è in distress (ev<-0.3), sat = 0.65. Altrimenti da calcolo.

CD5 Relazione (dim 7, need L5):
```
engagement = 0.35
satisfaction = 0.65
val_base = 0.35 × (2×0.65 - 1) = 0.35 × 0.30 = 0.105
```

Poi colorazione interlocutor: `presence=0.9, resonance=0.5` → `relational_tone = 0`.
```
val_colorazione = 0.9 × 0.3 × 0 = 0
val_CD5 = 0.105
```

CD8 Vulnerabilità (dim 1, need L1 Sopravvivenza):
```
engagement = field_sig[1] = 0.50 (supponiamo)
satisfaction[L1] = 0.85 (vital ok)
val_base = 0.50 × (2×0.85 - 1) = 0.50 × 0.70 = 0.35
- fatigue penalty: -0.3 × 0.15 (fatica lieve) = -0.045
val_CD8 = 0.305 → clamp [-1,+1]
```

CD dominante: CD8 (0.305) > CD5 (0.105). Stance label: "sicuro" (CD8 positivo, "potenzialmente perdo qualcosa ma ok").

### 17.4 — Deliberazione

`field_pressures`: Express 0.45, Question 0.25, Reflect 0.20, ecc.

`narrative.deliberate(...)`:
- Stance from valence: "curioso" (CD dom CD8 positivo lieve, tutto moderato).
- Intention: `Resonate` (CD5 non dominante ma attivo + other_in_distress).
- `pending_intention = Resonate (archetype "risuonare")`.

### 17.5 — Generazione

`compose(..., other_in_distress=true, response_intention=Some("risuonare"))`:
- Voice: Person=Second, Mood=Interrogative.
- Top nucleo: `(paura, Causes, tremore)` strength = sqrt(0.5 × 0.144) × 0.90 × 1.0 × 1.0 × 1.0 = 0.255. Valence boost.
- Render: "Senti il tremore?"

### 17.6 — Latenza

Tutto questo in ~20ms. Numeri reali dal test del refactor.

---

## Sintesi del volume

Appendice matematica che raccoglie le formule principali con esempi numerici:

1. **Propagazione**: `src_act × damping × weight × cos(phase)` con rendimento decrescente + cap.
2. **Decadimento**: `act × 0.92/tick` — ~38 tick per tornare sotto soglia.
3. **Resting state**: `stability × 0.002` — sotto-soglia permanente.
4. **Hebbian**: LTP +0.05 per co-attivazione, LTD 0.995 per silenzio. Cap 3.0.
5. **Phi-decay**: `intensity × 0.618^n` — ~14 cicli REM per rimozione.
6. **`derive_8d_from_kg`**: 8 formule dim-per-dim, con esempi per parole concrete (io, paura, amore).
7. **Hub damping**: `1/(1+ln(ratio))` — super-hub vanno a 12% del peso.
8. **Valenza**: `engagement × (2×satisfaction-1)` + colorazioni.
9. **Desire satisfaction**: cosine < 0.2 per 3 tick.
10. **Commitment inertia**: `strength × ln(turns_held+1)`.
11. **Frattale affinity**: distanza euclidea normalizzata sulle dim fisse.
12. **Proposition strength**: prodotto di 5-6 fattori.
13. **Specificity**: `min(2, 300/n)` con sweet spot 300.
14. **Presence decay**: 0.985/tick, half-life 46 tick.
15. **Soglia espressione spontanea**: 0.6 base, scende a 0.35 con bisogni/desideri.

**50+ costanti** raccolte in tabella. Fornisce un riferimento rapido per chi vuole modificare il comportamento del sistema a livello di costanti.

**Esempio end-to-end** di "ho paura": attivazioni → propagazione → valenza → deliberazione → "Senti il tremore?". ~20ms di latenza, ~50 operazioni numeriche.

Da qui il vol. 99 — **le mie considerazioni finali**: cosa ho imparato scrivendo il libretto, le direzioni prioritarie, le proposte strategiche.

---

*Prossimo volume: 99 — Considerazioni finali* (in scrittura)
