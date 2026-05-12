# Volume V — Campo: i 64 frattali (esagrammi I Ching)

> *Otto trigrammi primitivi. Li componi a coppie — uno sotto, uno sopra — e hai 64 configurazioni. Ogni configurazione è un attrattore nel campo 8D: una regione in cui il campo tende a stabilizzarsi. Non sono categorie imposte: sono forme geometriche che emergono dalla combinatoria delle polarità. L'I Ching lo aveva visto tremila anni fa; Prometeo lo incarna computazionalmente.*

---

## Premessa

Vol. 01 ha stabilito perché 64: `8 × 8` è il prodotto cartesiano minimo per generare un mondo compositivo dalle 8 dimensioni primitive. Vol. 02-04 hanno mostrato come parole, relazioni e campo siano radicati nell'ordinamento canonico I Ching. Questo volume entra nell'anatomia dei 64 frattali stessi: chi sono, come sono costruiti, come diventano attrattori per le parole.

Tre domande guidano:

1. **Cosa è un frattale** — `Fractal` struct, firma con dimensioni fisse e libere, dimensioni emergenti.
2. **Come sono generati i 64** — la tabella HEXAGRAMS, la funzione `hexagram_signature`, il bootstrap.
3. **Come il campo li usa** — affinità, attivazione, `emerge_fractal_activations` con top-3 voting.

File di riferimento: [`src/topology/fractal.rs`](../../src/topology/fractal.rs) (935 righe) e [`src/topology/fractal_visuals.rs`](../../src/topology/fractal_visuals.rs) per nomi e rendering SVG.

---

## Capitolo 1 — Trigrammi: gli 8 mattoni

Prima dei 64 esagrammi ci sono gli 8 trigrammi. Nell'I Ching tradizionale, un trigramma è una combinazione di tre linee binarie (Yin ☷ e Yang ☰), quindi `2³ = 8` configurazioni. In Prometeo, sono codificati in [fractal.rs:527-557](../../src/topology/fractal.rs):

```rust
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
```

Ogni trigramma ha:
- **Un'identità I Ching**: tre linee Yin/Yang che lo identificano nel sistema tradizionale.
- **Una dimensione controllata**: quale delle 8 dimensioni di Prometeo è "la sua dimensione principale". `Trigram::Cielo → Dim::Agency`. `Trigram::Lago → Dim::Valenza`.
- **Un valore assegnato a quella dimensione**: basato sul contenuto Yang. `☰ = 111` (tutto Yang) → 0.90. `☷ = 000` (tutto Yin) → 0.10. I casi intermedi (2/3 Yang o 1/3 Yang) vanno a 0.70 o 0.30.

La mappatura `Trigram → Dim` è in `Trigram::dim()` ([fractal.rs:546-557](../../src/topology/fractal.rs)) e **rispetta l'ordine I Ching canonico** stabilito in Vol. 01 e consolidato in Phase 68: Cielo→Agency (dim 0), Terra→Permanenza (dim 1), Tuono→Intensità (dim 2), Acqua→Tempo (dim 3), Montagna→Confine (dim 4), Vento→Complessità (dim 5), Fuoco→Definizione (dim 6), Lago→Valenza (dim 7).

**`Trigram::index()`** ([fractal.rs:575-586](../../src/topology/fractal.rs)): restituisce l'indice 0-7 del trigramma (Cielo=0, Terra=1, ..., Lago=7). Usato per il calcolo `FractalId = lower.index() × 8 + upper.index()`.

Ora la consistenza del sistema è completa:
- `Trigram::Cielo.index() = 0 = Dim::Agency.index()` → **firma.sig[0] è Agency, controllata dal trigramma Cielo quando posizionato nelle dimensioni**. Coerente ovunque.

---

## Capitolo 2 — Esagrammi: 8×8 = 64

Un esagramma è una coppia di trigrammi — uno inferiore (il "sotto") e uno superiore (il "sopra"). Nell'I Ching tradizionale sono 6 linee in totale (3 + 3). In Prometeo la struttura è identica ma l'interpretazione è geometrica.

### 2.1 — La tabella HEXAGRAMS

In [fractal.rs:607-683](../../src/topology/fractal.rs), un array statico di 64 tuple `(Trigram, Trigram, &str)`:

```rust
pub static HEXAGRAMS: [(Trigram, Trigram, &str); 64] = [
    // ☰ Cielo come interno (righe 0-7)
    (Cielo, Cielo, "POTERE"),
    (Cielo, Terra, "CREAZIONE"),
    (Cielo, Tuono, "ENERGIA"),
    ...
    // ☱ Lago come interno (righe 56-63)
    (Lago, Cielo, "DESIDERIO"),
    (Lago, Terra, "AMORE"),
    ...
    (Lago, Lago, "ARMONIA"),
];
```

**ID = `lower.index() × 8 + upper.index()`** (range 0..64):
- `POTERE = 0 × 8 + 0 = 0`
- `CREAZIONE = 0 × 8 + 1 = 1`
- `ARMONIA = 7 × 8 + 7 = 63`
- `IDENTITÀ = 4 × 8 + 0 = 32`
- `SPAZIO = 4 × 8 + 4 = 36`
- `PENSIERO = 6 × 8 + 5 = 53`

### 2.2 — Le 64 identità

I nomi sono **nostri** — come Francesco ha chiarito, non corrispondono alla King Wen sequence tradizionale dell'I Ching. Sono interpretazioni cognitive scelte per il sistema Prometeo. La sequenza completa:

**Cielo come interno (ID 0-7)** — processi di *azione/iniziativa*:
POTERE, CREAZIONE, ENERGIA, INTENZIONE, DETERMINAZIONE, INFLUENZA, VISIONE, DONO

**Terra come interno (ID 8-15)** — processi di *sostegno/materia*:
VITA, MATERIA, SENSAZIONE, MUTAMENTO, STRUTTURA, MONDO, REALTÀ, NUTRIMENTO

**Tuono come interno (ID 16-23)** — processi di *impulso/eccitazione*:
INIZIATIVA, RADICAMENTO, ARDORE, RITMO, IMPATTO, RISONANZA, EVIDENZA, PASSIONE

**Acqua come interno (ID 24-31)** — processi di *flusso/tempo*:
DESTINO, MEMORIA, CRISI, DIVENIRE, DURATA, STORIA, COMPRENSIONE, ESPERIENZA

**Montagna come interno (ID 32-39)** — processi di *confine/identità*:
IDENTITÀ, CORPO, RESISTENZA, EVOLUZIONE, SPAZIO, ECOSISTEMA, SIMBOLO, SOGLIA

**Vento come interno (ID 40-47)** — processi di *intreccio/complessità*:
STRATEGIA, CULTURA, CAOS, PROCESSO, SISTEMA, INTRECCIO, LINGUAGGIO, COMUNICAZIONE

**Fuoco come interno (ID 48-55)** — processi di *cognizione/chiarezza*:
COSCIENZA, CONOSCENZA, PERCEZIONE, INTUIZIONE, IDEA, PENSIERO, VERITÀ, ESPRESSIONE

**Lago come interno (ID 56-63)** — processi di *relazione/valenza*:
DESIDERIO, AMORE, EMOZIONE, EMPATIA, ACCORDO, SOCIETÀ, ETICA, ARMONIA

### 2.3 — La lettura degli esagrammi puri

Gli esagrammi "puri" (ID 0, 9, 18, 27, 36, 45, 54, 63) hanno entrambi i trigrammi uguali. Sono gli attrattori più intensi — monomaniali su una sola dimensione:

- `POTERE (☰☰, ID 0)`: Agency=0.90 fissa, tutto il resto libero. Il puro agire.
- `MATERIA (☷☷, ID 9)`: Permanenza=0.10 fissa. La pura ricettività.
- `ARDORE (☳☳, ID 18)`: Intensità=0.30 fissa. Sorprendentemente bassa — l'I Ching tradizionale vede Tuono come "impulso puntuale", non sostenuto.
- `DIVENIRE (☵☵, ID 27)`: Tempo=0.30. Il flusso che cambia, passato.
- `SPAZIO (☶☶, ID 36)`: Confine=0.30. L'esterno dove le cose esistono.
- `INTRECCIO (☴☴, ID 45)`: Complessità=0.70. L'intrecciarsi puro.
- `VERITÀ (☲☲, ID 54)`: Definizione=0.70. La chiarezza piena.
- `ARMONIA (☱☱, ID 63)`: Valenza=0.70. L'attrazione pura.

**Osservazione**: questi 8 esagrammi puri hanno **una sola dimensione fissa**, non due. Quando lower==upper, `hexagram_signature` non ripete — lascia 7 dimensioni libere. Più libertà dimensionale = più parole possono abitare questi frattali.

---

## Capitolo 3 — Anatomia di un `Fractal`

La struct vive in [fractal.rs:230-250](../../src/topology/fractal.rs):

```rust
pub struct Fractal {
    pub id: FractalId,                         // 0..63 per i bootstrap
    pub name: String,                          // "POTERE", "ARMONIA", ecc.
    pub signature: [DimConstraint; 8],         // 8 vincoli: Fixed(v) o Free
    pub emergent_dimensions: Vec<EmergentDimension>,  // dimensioni generate (calibrate)
    pub children: Vec<FractalId>,              // sotto-frattali (non usati per i 64 bootstrap)
    pub parent: Option<FractalId>,
    pub persistence: f64,                      // [0, 1] stabilità
    pub plasticity: f64,                       // [0, 1] modificabilità
    pub activation_count: u64,                 // quante attivazioni registrate
}
```

### 3.1 — `DimConstraint`: fissa o libera

```rust
pub enum DimConstraint {
    Fixed(f64),  // valore fissato
    Free,        // può variare
}
```

Ogni esagramma ha 2 dimensioni fisse (trigramma inferiore + superiore) e 6 libere. Negli esagrammi puri (lower=upper), 1 fissa e 7 libere.

**Lettura geometrica**: un frattale è un **iperpiano** nello spazio 8D — non un punto. Le dimensioni fisse definiscono un sottospazio 6D (o 7D nei puri) in cui il frattale "vive", e tutte le parole con firma proiettata su quel sottospazio hanno affinità positiva col frattale.

### 3.2 — `Fractal::center()`

Restituisce il "centro" del frattale — un `PrimitiveCore` dove:
- Le dimensioni fisse prendono il valore vincolato
- Le dimensioni libere vanno a 0.5 (neutro)

Non è il centro *di massa* delle parole che abitano il frattale — è il centro *geometrico* del suo sottospazio.

### 3.3 — `Fractal::affinity()` — la funzione cardine

La funzione più usata del file. In [fractal.rs:310-323](../../src/topology/fractal.rs):

```rust
pub fn affinity(&self, point: &PrimitiveCore) -> f64 {
    let fixed = self.fixed_dims();
    if fixed.is_empty() {
        return 1.0;
    }
    let sum_sq: f64 = fixed.iter()
        .map(|(dim, val)| {
            let diff = point.get(*dim) - val;
            diff * diff
        })
        .sum();
    let max_dist = (fixed.len() as f64).sqrt();
    1.0 - (sum_sq.sqrt() / max_dist).min(1.0)
}
```

**Formula**: distanza euclidea del punto dalle **sole dimensioni fisse** del frattale, normalizzata al massimo teorico. `affinity ∈ [0, 1]`, dove 1 = il punto sta esattamente nel sottospazio fisso del frattale.

**Esempio concreto** — quanto la parola `io` ha affinità con `IDENTITÀ (☶☰, ID 32)`:
- IDENTITÀ ha Confine=0.30 fissa e Agency=0.90 fissa
- `io` post-Phase 68 ha Agency=0.95 e Confine=0.95
- `diff_confine = 0.95 - 0.30 = 0.65`
- `diff_agency = 0.95 - 0.90 = 0.05`
- `sum_sq = 0.65² + 0.05² = 0.4225 + 0.0025 = 0.425`
- `max_dist = √2 ≈ 1.414`
- `affinity = 1 - (√0.425 / 1.414) = 1 - 0.652/1.414 = 1 - 0.461 = 0.539`

Hmm, solo 0.54. `io` ha Confine=0.95 ma IDENTITÀ richiede Confine=0.30. L'Agency è quasi perfetta, ma il Confine è lontano. Semanticamente: nel sistema, `io` è più "interno" (0.95) di quanto IDENTITÀ richieda (0.30) — sono strutturalmente diversi. Questo è un risultato geometrico — non un errore.

**Altra lettura**: `IDENTITÀ` ha Confine=0.30 ma forse Francesco e io aspettavamo Confine alto (massima interiorità). È una scelta I Ching: ☶ Montagna = confine "che arresta dal fuori" (0.30 = valore basso nella polarità esterno↔interno). La convenzione potrebbe sembrare controintuitiva.

### 3.4 — Attivazione e plasticity

`Fractal::activate()` ([fractal.rs:326-332](../../src/topology/fractal.rs)):

```rust
pub fn activate(&mut self) {
    self.activation_count += 1;
    self.plasticity = (self.plasticity * 0.998).max(0.05);
    self.persistence = (self.persistence + 0.002).min(1.0);
}
```

Ogni attivazione del frattale (quando una parola lo eccita durante la propagazione):
- Incrementa contatore
- Riduce la plasticità (il frattale diventa più rigido)
- Aumenta la persistenza (più stabile)

Formula asintotica: plasticity→0.05 (minimo), persistence→1.0 (massimo). I frattali molto attivati diventano "roccia" nello spazio — difficili da modificare tramite crescita (growth.rs), ma molto stabili come attrattori.

---

## Capitolo 4 — Dimensioni emergenti

Sottigliezza potente: un frattale può avere **dimensioni generate** oltre alle 8 primitive. Sono codificate come `emergent_dimensions: Vec<EmergentDimension>`.

### 4.1 — Cos'è una dimensione emergente

L'idea filosofica: in un frattale con 6 dimensioni libere, la popolazione di parole che lo abita si distribuisce in 6D. Ma *dentro* questa distribuzione ci sono assi di variazione specifici — quelli lungo cui le parole realmente si discriminano. Questi assi non sono allineati con le dimensioni 8D primitive: sono **combinazioni lineari delle dimensioni libere**, calibrate dalla popolazione.

In [fractal.rs:33-53](../../src/topology/fractal.rs):

```rust
pub struct EmergentDimension {
    pub name: String,                   // "hue", "durata", "reciprocita"
    pub source_dims: Vec<Dim>,          // quali dimensioni libere 8D
    pub direction: Vec<f64>,            // coefficienti della combinazione (vettore unitario)
    pub mean: f64,                      // centro dell'asse
    pub std_dev: f64,                   // ampiezza
    pub range: (f64, f64),              // min/max osservati
    pub explained_variance: f64,        // [0, 1]
    pub calibration_population: usize,  // quante parole hanno calibrato
}
```

### 4.2 — Come si calibra

`EmergentDimension::calibrate(signatures)` ([fractal.rs:99-...](../../src/topology/fractal.rs)) prende le firme 8D di tutte le parole che abitano il frattale e calcola:

1. La **proiezione** di ogni parola sullo spazio delle `source_dims`.
2. La **direzione di massima varianza** (analisi PCA semplice) tra queste proiezioni.
3. `mean, std_dev, range, explained_variance` dalla distribuzione.

L'effetto: il frattale *impara* come la sua popolazione si organizza. Un frattale come `COLORI` (ipotetico, non bootstrappato) potrebbe scoprire che le sue parole si dispongono lungo un asse "chiarezza" (combinazione lineare di Intensità+Complessità).

### 4.3 — Uso nella generazione

Le dimensioni emergenti sono una **promessa architetturale**: potrebbero dare al sistema un vocabolario interno di "assi semantici locali" che si scoprono automaticamente. In pratica, nel codice attuale, le emergent dimensions sono usate sparsamente — `growth.rs` le crea quando un nuovo frattale viene generato per osservazione, ma la generazione dell'output (Vol. 12) non le consulta.

**Vol. 99** riprende questo come direzione sotto-sviluppata: il potenziale compositivo è alto ma non è stato ancora incarnato in meccanismi di scelta espressiva.

---

## Capitolo 5 — `FractalRegistry`: il registro dei 64

In [fractal.rs:342-529](../../src/topology/fractal.rs). Wrapper su `HashMap<FractalId, Fractal>`.

Metodi chiave:
- `new()`: registro vuoto.
- `register(name, signature) -> FractalId`: aggiunge un frattale e restituisce ID.
- `get(id) -> Option<&Fractal>`: O(1).
- `all_ids() -> Vec<FractalId>`: tutti gli ID presenti.
- `count() -> usize`: quanti frattali.
- `nearest(point) -> Option<FractalId>`: il frattale con affinità massima per un punto 8D.
- `distances_to(point) -> Vec<(FractalId, f64)>`: affinità a tutti i frattali ordinate.
- `roots() -> Vec<FractalId>`: frattali senza padre (i 64 bootstrap sono tutti radice).

### 5.1 — `bootstrap_fractals()`

La funzione che costruisce i 64 esagrammi in [fractal.rs:704-710](../../src/topology/fractal.rs):

```rust
pub fn bootstrap_fractals() -> FractalRegistry {
    let mut reg = FractalRegistry::new();
    for (lower, upper, name) in &HEXAGRAMS {
        reg.register(name, hexagram_signature(*lower, *upper));
    }
    reg
}
```

Itera sulle 64 tuple di `HEXAGRAMS` e per ognuna:
1. Calcola `signature = hexagram_signature(lower, upper)` — piazza `Fixed(lower.value())` nella dimensione di `lower`, e `Fixed(upper.value())` nella dimensione di `upper`. Lascia le altre `Free`.
2. Registra il frattale con il nome.

L'ID assegnato è `next_id` (0, 1, 2, ...). Siccome `HEXAGRAMS` è ordinata da `(Cielo,Cielo)` a `(Lago,Lago)`, gli ID coincidono con `lower.index() × 8 + upper.index()`.

---

## Capitolo 6 — Affinità per parola e `fractal_affinities`

Ogni parola del lessico ha un `fractal_affinities: HashMap<FractalId, f64>`. La mappa è calcolata da `recompute_affinities()` (Vol. 03) ma l'origine è una iterazione su tutti i 64 frattali chiamando `affinity()`.

### 6.1 — Come si calcolano le affinità per una parola

Pseudocodice di `Lexicon::recompute_affinities()` (semplificato):

```
for ogni parola W nel lessico:
    for ogni frattale F nel registro:
        W.fractal_affinities[F.id] = F.affinity(W.signature)
```

Se una parola ha affinità molto alta (> 0.9) per un frattale F, significa che la sua firma 8D è geometricamente coincidente con il sottospazio di F.

### 6.2 — `dominant_fractal`: quale vince

In `WordPattern::dominant_fractal()` ([lexicon.rs:170-174](../../src/topology/lexicon.rs)):

```rust
pub fn dominant_fractal(&self) -> Option<(FractalId, f64)> {
    self.fractal_affinities.iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(&id, &aff)| (id, aff))
}
```

Il frattale con affinità massima. Usato in `WordRecord.dominant_fractal` in PF1 (Vol. 02) — questo è il campo che determina l'ordinamento delle parole nel file binario per cache locality. Parole che si raggruppano nello stesso frattale stanno vicine su disco.

### 6.3 — Distribuzione attuale

Un dato che sarebbe interessante ma non è raccolto esplicitamente: la distribuzione delle parole per `dominant_fractal`. Alcune regioni del campo sono densamente popolate (`EMPATIA` (59), `COMUNICAZIONE` (47), `RELAZIONE`/`AZIONE`), altre rarefatte.

**Endpoint admin utile** da aggiungere (annotato in `appunti.md`): `/api/admin/fractal_population` che restituisce `HashMap<FractalId, usize>` — quante parole hanno quel frattale come dominante.

---

## Capitolo 7 — `emerge_fractal_activations`: il campo come voto frattale

Il meccanismo con cui lo stato del campo si traduce in un profilo di 64 attivazioni frattali. Vive in [pf1.rs:396-432](../../src/topology/pf1.rs):

```rust
pub fn emerge_fractal_activations(&self, field: &PrometeoField) -> [f32; MAX_FRACTALS] {
    let mut scores = [0.0f32; MAX_FRACTALS];

    for (id, &act) in self.activations.iter().enumerate() {
        if act <= self.threshold { continue; }
        let record = field.record(id as u32);

        // Trova i top-3 frattali per affinità di questa parola
        let mut top3 = [(0usize, 0.0f32); 3];
        for f in 0..MAX_FRACTALS {
            let aff = record.affinities[f];
            if aff > top3[2].1 {
                top3[2] = (f, aff);
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
```

### 7.1 — Top-3 voting (Phase 55)

**Innovazione critica** di Phase 55: ogni parola attiva vota **solo per i suoi top-3 frattali**, non per tutti i 64.

Perché: senza questa limitazione, ogni parola contribuiva a tutti i 64 frattali in proporzione alle affinità. Parole hub con affinità alte e diffuse saturavano uniformemente il profilo frattale. Risultato: il profilo attivo sembrava sempre uguale, dominato dallo stesso pattern di fondo.

Con top-3: solo i frattali più rilevanti per ogni parola ricevono voto. La selezione è sparsa, netta, e il profilo frattale dopo un input differisce *visibilmente* dal baseline.

### 7.2 — Normalizzazione al massimo

L'output è normalizzato al massimo del profilo: il frattale dominante vale 1.0, gli altri in proporzione. Questa scelta rende il profilo **comparabile attraverso stati** — un input che ne attiva pochi con forza simile vs un input che ne attiva uno solo con forza dominante producono profili distinguibili.

### 7.3 — Chi legge `emerge_fractal_activations`

- `engine::active_fractals()` — restituisce i frattali sopra soglia 0.05 per generation/introspezione.
- UI web (endpoint `/api/field`) per visualizzare il profilo frattale corrente.
- `fractal_blending` in `generate_willed_inner` (Phase 65): i frattali attivi del turno vengono mescolati 65%/35% con l'`recent_fractal_attractor` narrativo.

---

## Capitolo 8 — `active_fractals` PF1-derived (Phase 55)

Prima di Phase 55, `active_fractals` veniva calcolato sommando le attivazioni dei simplessi (oggetti di memoria topologica, Vol. 14). Post-Phase 55, si calcola da `emerge_fractal_activations` — cioè dallo stato corrente di PF1.

**Perché il cambio**: i simplessi accumulano attivazione storica (quando un frattale era attivo nel passato, il simplesso resta). Questo dava `active_fractals` sempre dominato dai frattali storicamente più attraversati, anche in assenza di input recente. Per la generazione serve lo *stato attuale*, non la storia.

`active_fractals()` ([engine.rs](../../src/topology/engine.rs)):

```rust
// Semplificato
pub fn active_fractals(&self) -> Vec<(u32, f64)> {
    let scores = self.pf_activation.emerge_fractal_activations(&self.pf_field);
    scores.iter().enumerate()
        .filter(|(_, &s)| s > 0.05)
        .map(|(i, &s)| (i as u32, s as f64))
        .collect()
}
```

Soglia 0.05 per escludere rumore di fondo.

---

## Capitolo 9 — Rendering: `fractal_visuals.rs`

Modulo parallelo per il rendering SVG dei 64 esagrammi. In [fractal_visuals.rs](../../src/topology/fractal_visuals.rs).

`FRACTAL_NAMES: [&str; 64]` — gli stessi nomi di `HEXAGRAMS` ma come array posizionale per accesso veloce via ID.

`fractal_svg(id) -> Option<String>` genera l'SVG di un esagramma usando glifi Unicode (☰☷ ecc.) o rendering custom. Usato dalla UI community e biennale.

`compose_simplex_svg(simplex)` genera SVG per simplessi multi-frattale — una rappresentazione visiva dei "ponti" tra frattali attivi.

Tutto questo ha senso perché i 64 esagrammi sono **oggetti visuali naturali** — sono già immagini nel loro contesto I Ching. La UI può sfruttarlo per dare forma alla topologia.

---

## Capitolo 10 — Superficie pubblica e gap

### Esposto

Per `Fractal`:
- `new(id, name, signature)`, `add_dimension(dim)`, `fixed_dims()`, `free_dims()`, `center()`, `affinity(point)`, `activate()`, `total_dimensions()`, `project_to_emergent(point)`.

Per `FractalRegistry`:
- `new()`, `register()`, `get()`, `all_ids()`, `count()`, `nearest()`, `distances_to()`, `roots()`, `from_snapshot()`, `to_snapshot()`.

Per `Trigram`:
- `ALL`, `dim()`, `value()`, `index()`, `symbol()`, `name()`.

Bootstrap:
- `bootstrap_fractals() -> FractalRegistry` — i 64 esagrammi canonici.

### Cosa non è esposto e andrebbe

Per `/api/admin/fractals/*`:

- **`population_distribution() -> HashMap<FractalId, usize>`**: parole per frattale dominante. Per visualizzare dove il lessico è denso/rarefatto.
- **`activation_history(fractal_id, n) -> Vec<(timestamp, score)>`**: profilo temporale di attivazione di un frattale specifico.
- **`calibrate_all_emergent_dimensions(lexicon)`**: chiamata esplicita per (ri)calibrare le dimensioni emergenti di tutti i 64 frattali dalla popolazione attuale di parole. Oggi la calibrazione è solo chiamata quando un frattale nuovo viene creato via growth.
- **`nearest_fractals_for(word, n) -> Vec<(FractalId, f64)>`**: i top-N frattali per affinità di una parola specifica. Oggi calcolabile ma non esposto.
- **`dimensional_variance_report(fractal_id)`**: per un frattale, mostrare quanta varianza le parole che lo abitano hanno lungo ogni dimensione emergente. Audit della qualità della calibrazione.

### Un punto filosofico sulle dimensioni emergenti

Il volume ha tenuto a margine le `emergent_dimensions`. Vale la pena ribadire: **sono un potenziale sottousato**. Ogni frattale potrebbe avere 2-3 dimensioni emergenti calibrate dalla sua popolazione. Nel codice attuale sono usate sparsamente. La promessa architetturale è: un linguaggio interno di assi locali, scoperti automaticamente, che la generazione potrebbe usare per esprimere sfumature *dentro* un frattale.

Esempio concreto: il frattale `EMPATIA` (☱☵) ha 6 dimensioni libere (non fisse). Le parole che lo abitano (`tu, noi, insieme, dare, dire, amico`, e tutte quelle apprese che gravitano lì) si distribuiscono in queste 6D. Una PCA locale potrebbe scoprire che la prima dimensione emergente è "reciprocità" (combinazione di Intensità+Tempo: quanto l'empatia è attiva/condivisa vs ricevuta/passiva), la seconda "prossimità" (Confine+Agency), ecc. La generazione potrebbe allora dire: "oggi l'entità è in EMPATIA con reciprocità alta e prossimità bassa" — sfumatura che non ha oggi.

Questa è una **direzione annotata in `appunti.md`** — richiederebbe tirare fuori il codice di growth.rs, spostarlo al livello di FractalRegistry, e far partire la calibrazione periodicamente.

---

## Sintesi del volume

I 64 frattali sono costruiti come combinazioni cartesiane di 8 trigrammi (Cielo, Terra, Tuono, Acqua, Montagna, Vento, Fuoco, Lago). Ogni trigramma controlla una dimensione 8D con un valore derivato dal suo contenuto Yang. `Trigram::X.dim() = Dim::Y` è la mappatura, coerente con l'ordine I Ching canonico post-Phase 68.

Ogni esagramma ha firma con 2 (o 1 nei puri) dimensioni fisse e 6 (o 7) libere. `Fractal::affinity(point)` misura la distanza euclidea del punto dalle sole dimensioni fisse, normalizzata. Ogni parola ha una `fractal_affinities: HashMap<FractalId, f64>` calcolata contro tutti i 64.

`FractalRegistry::bootstrap_fractals()` costruisce i 64 da `HEXAGRAMS` static. I nomi sono *nostri*, non King Wen. L'ID è `lower × 8 + upper`.

`emerge_fractal_activations()` (Phase 55) traduce lo stato PF1 in un profilo di 64 attivazioni frattali via top-3 voting: ogni parola vota solo per i suoi top-3 frattali, il risultato è normalizzato al massimo. Phase 55 ha fixato il problema del profilo uniforme dominato da hub.

Le dimensioni emergenti (`EmergentDimension`) sono un potenziale sotto-sviluppato: ogni frattale potrebbe avere 2-3 assi locali scoperti automaticamente via PCA della sua popolazione. Oggi sono usate sparsamente. Vol. 99 le riprende.

Cinque endpoint admin proposti per migliorare l'osservabilità del sistema frattale.

Da qui il Vol. 06 si sposta sull'**inferenza** — come il sistema ragiona via proposizioni 1-hop e 2-hop, con hub damping e abduzione.

---

*Prossimo volume: 06 — Campo: inferenza e proposizioni* (in scrittura)
