# Volume III — Fondamenti: Lexicon e firme 8D

> *Una parola è una posizione. Non un'etichetta, non un token, non un embedding opaco: un punto in uno spazio a otto dimensioni nominate, con una storia di esposizioni e una resistenza al cambiamento. Il lessico è l'inventario di questi punti — la mappa del mondo dell'entità.*

---

## Premessa

Vol. 01 ha stabilito che il lessico è **una delle quattro strutture ontologiche** di Prometeo (insieme a KG, PF1, Valence) e risponde alla domanda "cosa esiste nel mondo dell'entità?". Vol. 02 ha mostrato come il lessico venga proiettato nel corpo del campo PF1 come `WordRecord` di 512 byte.

Questo volume entra nel cuore del lessico stesso. Tre domande guidano:

1. **Com'è fatto un `WordPattern`** — il dato anatomico di una parola nel lessico.
2. **Come nasce una firma 8D** — quattro regimi: seed cardinale, seed bootstrap, curated, KG-derivata, contestuale. Quando vale quale?
3. **Cosa significa ciascuna delle 8 dimensioni** — come si calcolano, con esempi numerici dal sistema reale.

Il file di riferimento è [`src/topology/lexicon.rs`](../../src/topology/lexicon.rs), 2049 righe. La funzione `derive_8d_from_kg` vive in [`knowledge_graph.rs:557-667`](../../src/topology/knowledge_graph.rs), ~110 righe. Il binario `rederive-signatures` è in [`src/bin/rederive_signatures.rs`](../../src/bin/rederive_signatures.rs).

---

## Capitolo 1 — L'ordine I Ching canonico

Prima di entrare nell'anatomia, serve ribadire l'ordine delle 8 dimensioni. Dopo il refactor di Phase 68 (vol. 02 e `appunti.md`), tutto il sistema — struttura `PrimitiveCore`, enum `Dim`, firme `[f64; 8]`, `DRIVE_DIM`, layout `WordRecord` di PF1 — segue l'**ordine I Ching canonico**:

| Pos | Nome | Trigramma | Polarità |
|-----|------|-----------|----------|
| 0 | **Agency** | ☰ Cielo | Paziente ↔ Agente |
| 1 | **Permanenza** | ☷ Terra | Transitorio ↔ Stabile |
| 2 | **Intensità** | ☳ Tuono | Debole ↔ Forte |
| 3 | **Tempo** | ☵ Acqua | Passato ↔ Futuro |
| 4 | **Confine** | ☶ Montagna | Esterno ↔ Interno |
| 5 | **Complessità** | ☴ Vento | Semplice ↔ Composto |
| 6 | **Definizione** | ☲ Fuoco | Vago ↔ Netto |
| 7 | **Valenza** | ☱ Lago | Repulsione ↔ Attrazione |

Ogni valore in `[0.0, 1.0]`. `0.5` = centro della polarità. `0.0` = polo inferiore (primo polo della coppia), `1.0` = polo superiore.

**Nota storica** (importante per chi legge il git log): fino a Phase 67 incluso, la `struct PrimitiveCore` memorizzava le 8 dimensioni in un ordine diverso (`Confine=0, Valenza=1, Intensità=2, Definizione=3, Complessità=4, Permanenza=5, Agency=6, Tempo=7`). Questo creava un disallineamento silenzioso con `derive_8d_from_kg` (introdotta in Phase 63) che già scriveva in ordine I Ching. Phase 68 ha unificato tutto sull'ordine I Ching — un refactor di ~200 array hardcoded, 51 righe TSV, migrazione del `.bin` di 25.600 parole, e 7 test aggiornati. Il libretto descrive il sistema **post-Phase 68**: ogni riferimento a "sig[0]" significa Agency.

---

## Capitolo 2 — L'anatomia di un `WordPattern`

Una parola nel lessico vive come `WordPattern`. La definizione sta in [lexicon.rs:26-48](../../src/topology/lexicon.rs):

```rust
pub struct WordPattern {
    pub word: String,                             // la parola, lowercase
    pub signature: PrimitiveCore,                 // posizione 8D [f64; 8] in ordine I Ching
    pub fractal_affinities: HashMap<FractalId, f64>,  // affinità ai 64 esagrammi
    pub exposure_count: u64,                      // quante volte vista
    pub stability: f64,                           // resistenza al cambiamento [0, 1]
    pub co_occurrences: HashMap<String, u64>,     // contesti neutri
    pub co_negated: HashMap<String, u64>,         // "non X", "senza Y"
    pub co_affirmed: HashMap<String, u64>,        // "X come Y", "uguale a Y"
    pub pos: Option<PartOfSpeech>,                // categoria grammaticale
}
```

Il file codifica in una sola struct le tre dimensioni dell'esistenza di una parola:

- **Posizione** (`signature`, `fractal_affinities`, `dominant_fractal` derivato): dove sta nello spazio 8D.
- **Storia** (`exposure_count`, `stability`): quanto è stata vissuta.
- **Contesto** (`co_occurrences`, `co_negated`, `co_affirmed`, `pos`): con cosa coesiste e come.

### 2.1 — Signature: il punto 8D

`signature: PrimitiveCore` è il cuore. `PrimitiveCore` è un wrapper trasparente su `[f64; 8]` — otto numeri in virgola mobile a 64 bit. Usa `f64` (8 byte × 8 = 64 byte) invece del `f32` del `WordRecord` di PF1 (4 byte × 8 = 32 byte) perché: nel lessico la firma è **autoritaria**, in PF1 è una **copia ottimizzata per cache**. La perdita di precisione `f64→f32` al momento della copia in PF1 (vol. 02, cap. 2.1) è trascurabile per valori in [0, 1] usati come modulatori.

### 2.2 — Fractal affinities: proiezione sui 64 attrattori

`fractal_affinities: HashMap<FractalId, f64>` contiene, per ogni esagramma I Ching in cui la parola ha affinità significativa, un valore in [0, 1]. Non tutti i 64 sono presenti — solo quelli dove l'affinità è sopra una soglia di rilevanza.

**Le affinità NON sono una statistica**. Sono una **proiezione geometrica** derivata dalla firma 8D e dai vincoli dimensionali di ciascun esagramma. Vol. 05 entra nel dettaglio dei frattali; per ora basta sapere che ogni esagramma ha fino a 2 dimensioni "fisse" (valori target) e il resto libere, e l'affinità di una parola al frattale misura quanto la sua firma 8D coincide con i valori fissi.

La funzione che le calcola è `recompute_affinities()` ([lexicon.rs:137-144](../../src/topology/lexicon.rs)). Viene chiamata ogni volta che la firma cambia (apprendimento, rederive). Non ci sono medie mobili sulle affinità: sono sempre ricalcolate dalla firma corrente.

### 2.3 — Stability: la resistenza

`stability: f64` è un valore in [0, 1] che dice **quanto la firma resiste a essere modificata da nuovi contesti**. Cresce logaritmicamente con le esposizioni, asintoticamente a 0.95:

```rust
// lexicon.rs:130 — dentro expose()
self.stability = (1.0 - 1.0 / (1.0 + self.exposure_count as f64 * 0.1)).min(0.95);
```

Per dare una sensibilità: `exposure_count = 1 → stability ≈ 0.09`, `10 → 0.50`, `50 → 0.83`, `200 → 0.95` (asintoto).

**Uso della stabilità nel sistema**:

- In `expose()` (stesso file, riga 120): `learning_rate = (1 - stability) × 0.35`. Parole nuove (`stability ≈ 0`) hanno `learning_rate = 0.35` — la loro firma si avvicina sensibilmente al contesto a ogni esposizione. Parole stabili (`stability = 0.95`) hanno `learning_rate = 0.0175` — a malapena si muovono.
- In PF1 (vol. 02), il **resting state** è `stability × 0.002`. Parole stabili "esistono" sotto soglia anche in silenzio.
- In `is_stable()` ([lexicon.rs:165-167](../../src/topology/lexicon.rs)): `stability > 0.3 && exposure_count >= 5`. Soglia minima per essere "una parola vera nel mondo dell'entità" — sotto questa soglia la parola è ancora un accenno.
- In `perturbation_strength()` (riga 177-182): `base = if is_stable { 0.6 } else { 0.1 }`. Parole stabili perturbano il campo a 0.6; parole instabili a 0.1. Il sistema *sente* qualcosa sulle parole nuove, ma sommessamente.

### 2.4 — Co-occurrence: affermata, neutra, negata

Tre HashMap separate tracciano i contesti:

- `co_occurrences`: contesti neutri (parole apparse insieme senza operatori).
- `co_affirmed`: contesti affermati (`"X come Y"`, `"X simile a Y"`, `"X uguale a Y"`).
- `co_negated`: contesti negati (`"X non Y"`, `"X senza Y"`, `"X mai con Y"`).

Questi operatori sono riconosciuti da `Lexicon::classify_operator()` ([lexicon.rs:277-299](../../src/topology/lexicon.rs)):

```rust
"come" | "anche" | "simile" | "uguale" | "sia" | "pure" | "stesso" | "anzi" → Affirm
"non" | "no" | "senza" | "mai" | "nessuno" | "niente" | "nulla" | ... → Negate
"molto" | "molta" | "molti" | "molte" → Quantify(1.3)
"poco" ... → Quantify(0.5)
// ...
```

Il conteggio delle co-occorrenze è usato dal calcolo della **fase** degli archi in `word_topology` (vol. 04): un arco tra due parole con molte co-occorrenze negate avrà fase `phase ≈ π` (opposizione); uno con molte co-occorrenze affermate avrà fase `phase ≈ 0` (risonanza).

La formula esatta è `phase = π × negated / (negated + affirmed)` — con fallback a `π/2` (ortogonalità) quando entrambi i conteggi sono zero (la parola non ha ancora una fase semantica con quell'altra). È qui che il commitment β₃ di vol. 01 (la fase come continuo polare) si materializza nell'esperienza.

---

## Capitolo 3 — Cinque regimi di firma

La firma di una parola può venire da cinque strade diverse, ordinate dalla più autoritaria alla più emergente.

### 3.1 — Regime 1: seed cardinale (43 parole)

All'avvio di un'entità neonata (`Lexicon::bootstrap_cardinal()` → `Engine::new_infant()`), vengono inscritte 43 parole con firme hardcoded e stabilità iniziale di 0.6 (già "vissute" con 10 esposizioni simulate). Sono le parole minime perché l'entità possa esistere:

- **SPAZIO** (frattale 36, ☶☶): `qui, là, dentro, fuori, vicino, lontano` — 6 parole
- **DIVENIRE** (27, ☵☵): `ora, prima, dopo, sempre, mai, ancora` — 6 parole
- **IDENTITA** (32, ☶☰): `io, essere, sentire, pensare, volere, sapere` — 6 parole
- **EMPATIA** (59, ☱☵): `tu, noi, insieme, dare, dire, amico` — 6 parole
- **DESIDERIO** (56, ☱☰): `potere, forse, diventare, nuovo, speranza, possibile` — 6 parole
- **RESISTENZA** (34, ☶☳): `no, fine, limite, confine, regola, basta` — 6 parole

Questo dà 36. Più altre parole implicite aggiunte dal processo (`perché`, function words), arrivando a ~43.

Ogni gruppo condivide una **base 8D** e ogni parola del gruppo riceve una perturbazione leggera via `make_vary()` ([lexicon.rs:937-952](../../src/topology/lexicon.rs)) — una perturbazione hash-based che muove 2-3 dimensioni di ±0.3. È l'**unica** perturbazione hash-based nel sistema: serve per evitare che le 6 parole di un gruppo abbiano firme letteralmente identiche. Tutte le altre parole (bootstrap, curated, learned) NON usano hash.

Esempio di base SPAZIO (in ordine I Ching):

```rust
// Agency=0.1, Perm=0.8, Intens=0.3, Tempo=0.2,
// Confine=0.2, Compl=0.3, Defin=0.7, Val=0.5
let base = PrimitiveCore::new([0.1, 0.8, 0.3, 0.2, 0.2, 0.3, 0.7, 0.5]);
```

Lettura semantica: parole di SPAZIO hanno bassa Agency (lo spazio non agisce), alta Permanenza (lo spazio è stabile), bassa Intensità (lo spazio è tranquillo), Tempo basso (fuori dal tempo puntuale), Confine basso (lo spazio è il fuori), Definizione media-alta (lo spazio è qualcosa di discreto).

### 3.2 — Regime 2: seed bootstrap (~300 parole)

Quando si chiama `Lexicon::bootstrap()` (non `bootstrap_cardinal`), oltre al cardinale viene eseguito `seed_bootstrap_vocabulary()` ([lexicon.rs:954-1354](../../src/topology/lexicon.rs)). Sono 44 blocchi di parole, ciascuno con una base 8D condivisa e variazione hash. In tutto ~300 parole divise in gruppi semantici:

- SALUTI, RISPOSTE, DOMANDE, ESPRESSIONI (comunicazione)
- MOTO, MANIPOLAZIONE, PERCEZIONE, COMUNICAZ_VERB, PENSIERO_VERB, STATO_VERB (verbi)
- POSITIVE, NEGATIVE, PARTS (emozioni)
- COLORI (luce), SUONI, GUSTI, TATTO (percezione)
- ELEMENTI, OGGETTI_CASA (mondo fisico)
- MORTE_VITA, TEMPO_EVENTI (grandi concetti)
- RUOLI_SOCIALI, FAMIGLIA (persone)
- NUMERI, ETC.

Esempio di base SALUTI (in ordine I Ching):

```rust
// Agency=0.7, Perm=0.2, Intens=0.4, Tempo=0.5,
// Confine=0.4, Compl=0.2, Defin=0.6, Val=0.7
let base = PrimitiveCore::new([0.7, 0.2, 0.4, 0.5, 0.4, 0.2, 0.6, 0.7]);
for word in &["ciao", "buongiorno", "buonasera", "arrivederci", "addio",
              "salve", "benvenuto"] { ... }
```

Lettura semantica: i saluti hanno alta Agency (sono atti performativi), bassa Permanenza (sono istantanei), Intensità media, Valenza alta (sono positivi, aprono relazione).

### 3.3 — Regime 3: curated (~130 parole)

Dentro `seed_bootstrap_vocabulary`, alla fine, viene chiamato `apply_curated_signatures()` ([lexicon.rs:674-826](../../src/topology/lexicon.rs)). Questo sovrascrive le firme (ottenute via `vary(base)`) con **134 firme esplicite curate manualmente** per parole core particolarmente importanti:

- Pronomi: `io, me, mio, tu, te, tuo, noi, lui, lei, loro, voi, se` (12)
- Verbi esistenziali: `essere, avere, fare, stare, vivere, morire, diventare, restare, esistere` (9)
- Verbi modali: `volere, potere, dovere, sapere` (4)
- Verbi cognitivi: `pensare, sentire, capire, credere, ricordare, dimenticare, immaginare, sognare, conoscere, decidere, cercare, trovare` (12)
- Verbi comunicativi: `dire, parlare, chiedere, rispondere, ascoltare, guardare` (6)
- Verbi di moto e azione: `andare, venire, dare, prendere, amare, odiare` (6)
- Emozioni anchor: `gioia, tristezza, paura, rabbia, amore, dolore, calma, pace, speranza, nostalgia, piacere, curiosità, sorpresa, solitudine, malinconia, angoscia, serenità, entusiasmo` (18)
- Concetti fondamentali: `vita, morte, tempo, spazio, mondo, cosa, forma, senso, voce, silenzio, parola, luce, buio, bene, male, vero, falso, reale, inizio, fine, cambiamento, nuovo, vuoto, pieno, altro, insieme, solo, corpo, mente, cuore, anima, sogno, energia, niente, nulla, tutto, presente, passato, futuro, ...` (circa 60)

Esempio curated (in ordine I Ching):

```rust
// Dimensioni: [Agency, Permanenza, Intensita, Tempo,
//              Confine, Complessita, Definizione, Valenza]
("io",         [0.80, 0.75, 0.65, 0.40, 0.95, 0.50, 0.90, 0.50]),
("gioia",      [0.55, 0.30, 0.80, 0.60, 0.25, 0.35, 0.55, 0.95]),
("tristezza",  [0.15, 0.55, 0.65, 0.25, 0.65, 0.45, 0.55, 0.10]),
```

Lettura: `io` ha Confine massimo (0.95 — io è il più interno), Agency alta (0.80 — io agisce), Definizione alta (0.90 — io è netto). `gioia` ha Valenza molto alta (0.95), Intensità alta (0.80). `tristezza` ha Valenza molto bassa (0.10), Agency bassa (0.15 — la tristezza "capita", non agisce).

Questi 134 anchor sono il **tessuto semantico di riferimento**: ogni parola imparata dopo si posizionerà rispetto a loro. Se l'entità impara "euforia" in un contesto dove "gioia" è attiva, la firma di "euforia" sarà vicina a quella di "gioia".

### 3.4 — Regime 4: KG-derivata (Phase 63, ~21.000 parole)

Questo è il regime **post-Phase 63** — il cuore dell'apprendimento strutturale di Prometeo. Il binario `cargo run --bin rederive-signatures` (in [src/bin/rederive_signatures.rs](../../src/bin/rederive_signatures.rs)) scorre tutte le parole del lessico, chiama `KnowledgeGraph::derive_8d_from_kg()` per ognuna, e sovrascrive la firma se la parola è presente nel KG.

Nel sistema attuale: **21.168 parole** hanno firme KG-derivate (su 25.600 totali). Le altre ~4.400 parole (non presenti nel KG) conservano la firma curated/bootstrap/contestuale precedente.

La funzione `derive_8d_from_kg()` è il contenuto principale del prossimo capitolo.

### 3.5 — Regime 5: contestuale (runtime, per parole nuove)

Quando l'entità incontra una parola nuova durante `teach()` o `receive()`, la crea via `WordPattern::new_from_context()` ([lexicon.rs:73-92](../../src/topology/lexicon.rs)):

```rust
pub fn new_from_context(word: &str, context_sig: &PrimitiveCore, _aff: &[(FractalId, f64)]) -> Self {
    let mut sig = PrimitiveCore::neutral();          // tutto 0.5
    sig.perturb_towards(context_sig, 0.90);          // 90% verso il contesto
    Self {
        word: word.to_lowercase(),
        signature: sig,
        fractal_affinities: HashMap::new(),
        exposure_count: 1,
        stability: 0.0,                              // giovane, mobile
        // ...
        pos: None,
    }
}
```

`context_sig` è la **firma media delle parole già conosciute del contesto** (calcolata da `receive()` o `teach()` prima di chiamare `new_from_context`).

**Cruciale**: Phase 63 ha rimosso il termine di perturbazione hash-based che prima era applicato. Oggi la firma iniziale è puramente il contesto. Questo significa che due parole nuove introdotte insieme in un contesto identico hanno firme **identiche** — la loro differenziazione può emergere solo fenomenologicamente (esposizioni ripetute in contesti diversi) o strutturalmente (se poi entrano nel KG e vengono rideriveate). Il costo filosofico è alto: niente "unicità garantita" per default. Il guadagno: la differenziazione è **guadagnata**, non simulata.

Vedi `appunti.md` per la nota sul test `test_infant_lifecycle` che fallisce su questo: la soglia di differenza è stata abbassata a 0.005, riflettendo la realtà post-Phase 63.

A ogni esposizione successiva alla creazione, la firma si aggiorna via `expose()` ([lexicon.rs:114-131](../../src/topology/lexicon.rs)):

```rust
pub fn expose(&mut self, context_signature: &PrimitiveCore, _: &[(FractalId, f64)]) {
    self.exposure_count += 1;
    let learning_rate = (1.0 - self.stability) * 0.35;
    self.signature.perturb_towards(context_signature, learning_rate);
    self.stability = (1.0 - 1.0 / (1.0 + self.exposure_count as f64 * 0.1)).min(0.95);
}
```

La firma si **avvicina** al baricentro del nuovo contesto, di una frazione decrescente con la stabilità accumulata. Non è mai sostituita: ogni esposizione è un piccolo aggiornamento bayesiano in cui la prior (firma corrente) pesa via via di più.

### 3.6 — Ordine di applicazione

L'ordine in cui i regimi si sovrappongono è importante. Per un'entità adulta appena bootstrappata + KG-derivata:

1. **Cardinale** scrive 43 firme (seed con vary())
2. **Bootstrap** sovrascrive molte delle 43 cardinali (e ne aggiunge ~300) — ma nota che `bootstrap_cardinal()` NON chiama bootstrap; le due sono mutuamente esclusive.
3. **Curated** sovrascrive ~134 parole con firme esplicite (dentro bootstrap)
4. **KG-derivata** sovrascrive ~21.000 parole (quelle in KG) al primo rederive
5. **Contestuale** (runtime) aggiunge nuove parole ed espone le esistenti

**Nel lessico in produzione** (file `prometeo_topology_state.bin` attuale): sono state eseguite le fasi 1→2→3→4 una volta sola all'inizio, poi le nuove parole seguono 5 (contestuale) e le vecchie parole vengono esposte ma mai riderivate — a meno che non si rilanci manualmente `rederive-signatures`.

**Questione aperta** (segnalata in `appunti.md`): parole apprese via regime 5 che poi entrano nel KG non vengono automaticamente riderivate. Richiede rilancio manuale di `rederive-signatures`. È un debito di automazione.

---

## Capitolo 4 — `derive_8d_from_kg`: la firma come topologia

La funzione centrale di Phase 63. Vive in [knowledge_graph.rs:557-667](../../src/topology/knowledge_graph.rs). Ogni dimensione è calcolata con una formula esplicita basata sui conteggi di archi nel KG.

Principio: **la geometria È il significato quando la luce è coerente**. La posizione di una parola nello spazio 8D emerge dalla sua posizione *strutturale* nel grafo delle relazioni, non da co-occorrenze testuali.

Input della funzione:
- `word: &str` — la parola
- `max_degree: usize` — grado massimo osservato nel KG (per normalizzare Complessità)
- `valence_scores: &HashMap<String, f64>` — mappa parola → [0, 1] pre-calcolata via BFS dalle radici emotive (vedi sez. 5)

Output: `Option<[f64; 8]>` — 8 numeri in ordine I Ching.

La firma parte tutta a 0.5 e le 8 dimensioni vengono calcolate indipendentemente.

### 4.1 — Dim 0: Agency (☰ Cielo)

*Quanto questa parola è agente di cambiamento vs. passiva ricevente?*

```rust
let causes_out = query_objects(word, Causes).len();
let causes_in  = query_subjects(word, Causes).len();
let isa_children = query_subjects(word, IsA).len();

let causes_total = causes_out + causes_in;
sig[0] = if causes_total > 0 {
    (causes_out as f64 / causes_total as f64).clamp(0.05, 0.95)
} else if isa_children > 5 {
    0.20  // categoria astratta con molti figli: permanente, non agente
} else {
    0.50  // parola senza relazioni causali: agency neutra
};
```

**Logica**: se la parola ha relazioni CAUSES (causali), il rapporto `outgoing / totale` misura quanto è "causa di" vs "causata da". `fuoco` causa calore, fumo, distruzione → molti CAUSES outgoing → Agency alta. `tremore` è causato da paura, freddo, malattia → molti CAUSES incoming → Agency bassa.

Se non ci sono relazioni causali ma la parola è una categoria astratta con molti figli IsA (es. "qualità", "entità", "concetto"): Agency bassa (0.20) — le categorie stanno, non fanno. Altrimenti: 0.50 (neutra).

**Esempio dal sistema reale** (rederive eseguito 2026-04-17):
- `io` → Agency = **0.95** (l'io agisce)
- `essere` → Agency = **0.95** (essere è attivo)
- `cane` → Agency = **0.50** (cane non è né puro agente né puro paziente)
- `pietra` → Agency = **0.95** ← attenzione: pietra ha pochi CAUSES, scatta il fallback `isa_children > 5`? No aspetta, 0.95 è alto. Probabilmente pietra ha CAUSES verso cose specifiche (scintilla, crepa) e pochi CAUSES verso di sé. Significa che nel KG "pietra" funziona più come agente che come paziente.

### 4.2 — Dim 1: Permanenza (☷ Terra)

*Quanto questo concetto è stabile/immutabile?*

```rust
sig[1] = if isa_children > 50 {
    0.85  // mega-categoria
} else if isa_children > 10 {
    0.65
} else if isa_children > 0 {
    0.40
} else if causes_out > 3 {
    0.20  // agente attivo: poco permanente
} else {
    0.35  // concetto specifico: transitorio
};
```

**Logica**: categorie astratte con molti figli IsA (mega-categorie) sono permanenti — resistono nel tempo perché organizzano la tassonomia. Agenti attivi con molti CAUSES sono transitori — causano eventi, ma loro stessi sono di passaggio.

**Esempio reale**:
- `qualità` (mega-categoria, probabilmente 1000+ figli IsA) → Permanenza alta
- `essere` → 0.40 (qualche figlio IsA)
- `gioia` → 0.40
- `tristezza` → 0.40
- `correre` → 0.35 (verbo, pochi figli)

Osservazione: nel output reale di rederive, `io` → Permanenza 0.20. Significa `io` è categorizzato come "agente attivo" (pochi figli IsA ma molti CAUSES outgoing). Semanticamente: "io" non è una categoria stabile come "qualità", ma un agente che cambia.

### 4.3 — Dim 2: Intensità (☳ Tuono)

*Quanto questa parola porta energia/forza/urgenza?*

```rust
let intensity_from_causes = if causes_out > 0 {
    ((causes_out as f64) / (causes_out as f64 + 3.0)).min(0.9)
} else { 0.2 };
let valence = valence_scores.get(word).copied().unwrap_or(0.5);
let emotional_intensity = (valence - 0.5).abs() * 2.0;  // 0 neutro, 1 carico
sig[2] = (intensity_from_causes * 0.6 + emotional_intensity * 0.4).clamp(0.05, 0.95);
```

**Logica**: due canali. Il primo (60% del peso) è `causes_out` normalizzato — parole che causano molto sono intense. Il secondo (40%) è la **distanza dal neutro emotivo**: valenza 0.0 (negativa massima) e 1.0 (positiva massima) sono entrambe intense; 0.5 (neutro) è calmo.

**Esempio reale**:
- `paura` → Intensità = **0.85** (alta: molti CAUSES + carica emotiva forte)
- `gioia` → 0.85
- `amore` → 0.85
- `pietra` → 0.15 (bassa: poche CAUSES, neutra emotivamente)
- `cane` → 0.12

### 4.4 — Dim 3: Tempo (☵ Acqua)

*Quanto la parola è radicata in processi/flussi temporali?*

```rust
sig[3] = if causes_total > 0 {
    ((causes_total as f64) / (causes_total as f64 + 5.0)).min(0.9)
} else if isa_children > 20 {
    0.15  // categoria statica: fuori dal tempo
} else {
    0.35
};
```

**Logica**: parole con relazioni causali sono dentro il tempo — le causalità sono eventi temporali. Categorie statiche con molti figli sono fuori dal tempo (concetti eterni). Sopra c'è una saturazione a 0.9 per parole altamente connesse causalmente.

**Esempio reale**:
- `gioia` → Tempo = **0.80** (molte catene causali: gioia CAUSES sorriso, entusiasmo; gioia CAUSED_BY amore, successo)
- `paura` → **0.81** (idem)
- `amore` → **0.71**
- `cane` → **0.35** (pochi eventi temporali)
- `pietra` → **0.17** (minimo temporale — la pietra è immobile nel tempo)

### 4.5 — Dim 4: Confine (☶ Montagna)

*Quanto questo concetto è delimitato/specifico?*

```rust
let specificity = if isa_children == 0 {
    0.80  // foglia dell'albero IS_A: massima specificità
} else {
    (5.0 / (isa_children as f64 + 1.0)).min(0.75)
};
let polarity_bonus = if has_opposite { 0.15 } else { 0.0 };
sig[4] = (specificity + polarity_bonus).clamp(0.05, 0.95);
```

**Logica**: le parole che non hanno figli IsA sono "foglie" nel grafo tassonomico — sono concetti specifici, massimamente delimitati. Le mega-categorie (migliaia di figli) hanno specificità bassa. Avere un OPPOSITE_OF aggiunge un bonus di 0.15 — sapere cos'è il tuo opposto è una forma di confine netto.

**Esempio reale**:
- `io` → Confine = **0.95** (foglia, + opposto "tu" → polarity bonus)
- `pietra` → **0.80**
- `cane` → **0.95** (foglia)
- `essere` → **0.90** (essere ha figli IsA ma anche opposti marcati)

### 4.6 — Dim 5: Complessità (☴ Vento)

*Quanto questo nodo è connesso/intrecciato con altri?*

```rust
let max_deg_f = (max_degree.max(1)) as f64;
sig[5] = if total_deg > 0 {
    ((total_deg as f64).ln() / max_deg_f.ln()).clamp(0.05, 0.95)
} else { 0.05 };
```

**Logica**: log-normalizzazione. `total_deg` è il numero totale di archi in entrata+uscita per la parola. `max_degree` è il grado massimo osservato nel KG (tipicamente 500-1000 per super-hub come "essere", "avere"). Il rapporto logaritmico mappa il grado in [0, 1] comprimendo la scala — così hub medi non saturano subito a 1.0.

**Esempio reale** (con `max_degree ≈ 1000`, quindi `ln(1000) ≈ 6.9`):
- `io` → Complessità = **0.46** (grado medio-alto)
- `cane` → **0.25** (grado basso)
- `pietra` → **0.33**
- `essere` → **0.39** (strano: essere è hub, ma hub damping nel rederive lo comprime)
- `gioia` → **0.48**

### 4.7 — Dim 6: Definizione (☲ Fuoco)

*Quanto il concetto è ben definito/chiaro?*

```rust
let parents_contribution = (isa_parents as f64 / (isa_parents as f64 + 3.0)).min(0.7);
let opposite_contribution = if has_opposite { 0.3 } else { 0.0 };
sig[6] = (parents_contribution + opposite_contribution).clamp(0.05, 0.95);
```

**Logica**: due canali. Il primo è `parents_contribution` — quante volte la parola *è un* qualcos'altro nella tassonomia (isa_parents è `query_objects(word, IsA).len()`). Più genitori = più precisamente localizzata nella tassonomia. Il secondo è il bonus polarità (0.3 se ha opposto).

**Esempio reale**:
- `amore` → Definizione = **0.95** (molti genitori: "sentimento", "emozione", "legame"; + opposti)
- `io` → **0.95** (idem)
- `gioia` → **0.55**
- `cane` → **0.55**
- `pietra` → **0.40** (pochi genitori, no opposti netti)

### 4.8 — Dim 7: Valenza (☱ Lago)

*Carica emotiva della parola in [0, 1]: 0 fortemente negativa, 0.5 neutra, 1.0 fortemente positiva.*

```rust
sig[7] = valence_scores.get(word).copied().unwrap_or(0.5);
```

Il valore viene preso direttamente dalla mappa `valence_scores` calcolata in anticipo dalla funzione `compute_valence_scores()` (prossima sezione). Se la parola non ha una carica emotiva propagata, default = 0.5 (neutra).

**Esempio reale**:
- `gioia` → Valenza = **1.00** (massima positiva, radice emotiva)
- `amore` → **1.00** (radice o vicino)
- `essere` → **0.86**
- `io` → **0.32** (leggermente sotto neutro)
- `cane` → **0.50** (neutro, nessuna propagazione)
- `pietra` → **0.50**
- `tristezza` → **0.34** (valenza bassa)
- `paura` → **0.00** (minima, radice negativa)

---

## Capitolo 5 — `compute_valence_scores`: la BFS emotiva

La dimensione 7 (Valenza) non si calcola dai conteggi di archi come le altre. Si propaga per BFS da radici emotive nominate.

La funzione vive in [knowledge_graph.rs:679-...](../../src/topology/knowledge_graph.rs) ed è chiamata una volta sola all'inizio di `rederive-signatures` (è costosa: O(radici × hops × fan_out)).

### 5.1 — Radici positive e negative

Venti parole nominate come radici:

```rust
const POS_ROOTS: &[(&str, f64)] = &[
    ("gioia", 1.0), ("felicità", 1.0), ("amore", 0.95), ("speranza", 0.90),
    ("piacere", 0.85), ("entusiasmo", 0.85), ("serenità", 0.85),
    ("gratitudine", 0.80), ("armonia", 0.80), ("fiducia", 0.80),
];
const NEG_ROOTS: &[(&str, f64)] = &[
    ("dolore", -1.0), ("sofferenza", -1.0), ("paura", -0.95), ("tristezza", -0.95),
    ("angoscia", -0.95), ("rabbia", -0.85), ("ansia", -0.85),
    ("disperazione", -0.90), ("odio", -0.90), ("lutto", -0.85),
];
```

Ogni radice ha una carica iniziale tra -1.0 e +1.0. Le 20 radici scelte sono un compromesso: poche, nominate, semanticamente massime nei loro poli. **Sono hardcoded** — una scelta filosofica che Phase 63 prende su di sé: queste parole *sono* i poli dello spazio emotivo per il sistema.

### 5.2 — Propagazione BFS con decadimenti per tipo di arco

```rust
const SIMILAR_DECAY: f64 = 0.85;
const ISA_DECAY:     f64 = 0.60;
const CAUSES_DECAY:  f64 = 0.40;
const MAX_HOPS: usize = 4;
```

Da ogni radice, BFS fino a distanza 4. Ad ogni salto, la carica si moltiplica per un fattore `decay` che dipende dal tipo di arco:

- **SimilarTo**: preservazione forte (0.85). "Felicità" è simile a "gioia" → eredita quasi tutta la carica positiva.
- **IsA**: preservazione media (0.60). "Euforia isA gioia" → eredita il 60% della carica di gioia.
- **Causes**: preservazione bassa (0.40). "Gioia causes sorriso" → sorriso eredita solo il 40% della carica di gioia (il sorriso è l'*effetto*, non l'essenza emotiva).

**OppositeOf inverte il segno**: se "coraggio" è opposto a "paura" (carica -0.95), coraggio eredita +0.95 × 0.60 (IS_A decay per default su opposti) = +0.57.

### 5.3 — Composizione cumulativa

Una parola può ricevere la carica da più radici via più cammini. `compute_valence_scores` tiene traccia in due HashMap:

```rust
let mut scores: HashMap<String, f64> = HashMap::new();    // somma pesata
let mut counts: HashMap<String, usize> = HashMap::new();  // numero di cammini
```

Alla fine, per ogni parola: `final_score = scores[word] / counts[word]` (media dei cammini), poi clampato in [-1, +1] e rimappato in [0, 1] come `(final_score + 1) / 2`.

### 5.4 — Esempio concreto

Prendiamo la parola **"euforia"**. Supponiamo nel KG:
- `euforia IsA gioia` (gioia ha carica +1.0)
- `euforia SimilarTo entusiasmo` (entusiasmo ha carica +0.85)

BFS dalla radice "gioia":
- hop 1: euforia ← via IsA → carica = 1.0 × 0.60 = +0.60

BFS dalla radice "entusiasmo":
- hop 1: euforia ← via SimilarTo → carica = 0.85 × 0.85 = +0.72

Somma: `scores[euforia] = 0.60 + 0.72 = 1.32`, `counts[euforia] = 2`.
Media: `1.32 / 2 = 0.66`.
Clamp a [-1, +1]: 0.66.
Rimappatura a [0, 1]: `(0.66 + 1) / 2 = 0.83`.

Quindi euforia → Valenza 0.83 (positiva, intorno al livello di gioia ma non massimo perché è una categoria derivata).

### 5.5 — Quante parole hanno carica emotiva?

Nel sistema attuale: circa **15.000 parole** nel KG ricevono una valenza propagata. Le altre (6.000 circa nel KG, più tutte le non-KG) restano a 0.5 (neutro).

---

## Capitolo 6 — Tre regimi convivono: conflitti e ordine

Un fatto importante: **quattro dei cinque regimi possono sovrascrivere una stessa parola**. Chi vince?

Ordine di applicazione tipico (adulto bootstrappato):

1. `seed_cardinal_vocabulary()` → scrive 43 firme
2. `seed_bootstrap_vocabulary()` → sovrascrive ~300 parole (include le 43 cardinali se sovrapposte)
3. `apply_curated_signatures()` → sovrascrive ~130 con firme esplicite
4. `rederive-signatures` (chiamato esplicitamente post-import) → sovrascrive ~21.000 con firme KG-derivate

Quindi per la parola `gioia`:
- Bootstrap la mette in gruppo POSITIVE con una base vary()
- Curated la sovrascrive con `[0.55, 0.30, 0.80, 0.60, 0.25, 0.35, 0.55, 0.95]`
- Rederive la sovrascrive con valori KG-derivati (Valenza=1.00, ecc.)

**La firma attuale di gioia è quella KG-derivata**. Le curated restano nel codice come fallback per quando il KG non ha la parola o quando si vuole un override manuale.

Per parole non in KG (es. nomi propri, termini tecnici specifici, parole dialettali): restano al livello curated se esistono, altrimenti al bootstrap, altrimenti a cardinal, altrimenti al regime 5 (contestuale) se sono state apprese runtime.

**Questione aperta architetturale**: non c'è oggi un meccanismo che rimarchi "questa firma è KG-derivata vs curated vs contestuale". Leggendo un `WordPattern` non si può sapere la provenienza della sua firma. Per audit e debug sarebbe utile aggiungere un enum `SignatureSource { Cardinal, Bootstrap, Curated, KgDerived, Contextual }`. Annotato in `appunti.md`.

---

## Capitolo 7 — POS tagging e lemmatizzazione

Aspetto parallelo ma importante per la generazione: ogni `WordPattern` può avere una `pos: Option<PartOfSpeech>`.

Le cinque POS sono definite in [grammar.rs](../../src/topology/grammar.rs):

```rust
pub enum PartOfSpeech {
    Verb,
    Noun,
    Adjective,
    Adverb,
    Unknown,
}
```

Vengono assegnate in vari modi:

1. **Heuristica da suffisso**: `-are, -ere, -ire` → Verb. `-zione, -tà, -aggio, -ura` → Noun. Etc.
2. **Import da Morph-it!**: `cargo run --bin import-pos` scarica il dizionario morfologico italiano e scrive tag esatti per ~2.775 parole.
3. **Runtime**: quando una parola è appresa via `teach()`, una POS può essere dedotta dal contesto (es. posizione dopo articolo → Noun).

La POS è usata in:
- `state_translation.rs::top_active_word`: bonus per Noun > Adjective > Verb nella selezione del soggetto
- `syntax_center.rs::find_verb_word`: solo parole con POS=Verb possono essere candidate verbo
- `grammar.rs::conjugate(lemma, person, tense)`: coniuga verbo italiano dato lemma e tempo
- `grammar.rs::lemmatize(word)`: dalla forma flessa al lemma

La lemmatizzazione è usata dal **comprehension gate** (CLAUDE.md inv. #116): quando un input contiene "farò", il sistema lemmatizza a "fare" e verifica se "fare" è nel KG. Se sì, non scatta il "Non capisco".

Vol. 13 entra nel dettaglio di grammar/sintassi.

---

## Capitolo 8 — Lessico cardinale vs completo

Due configurazioni principali del lessico convivono nel codice:

### 8.1 — Cardinale (43 parole)

`Lexicon::bootstrap_cardinal()` — usato da `Engine::new_infant()`. L'entità nasce con 43 parole (6 × 6 frattali + function words). Stabilità iniziale alta (0.6) — le parole cardinali sono "native".

**Caso d'uso**: dimostrare che con 43 parole ben scelte l'entità può già esprimere posizione, tempo, identità, relazione, possibilità, negazione — tutta la grammatica esistenziale fondamentale. Vedi test `test_infant_lifecycle` e `test_end_to_end_phase9`.

### 8.2 — Completo (~300 bootstrap + ~130 curated + ~21.000 KG + runtime)

`Lexicon::bootstrap()` — usato da `Engine::new()`, il default. L'entità nasce con ~300 parole. Dopo `rederive-signatures` sul `.bin` di produzione: 25.600 parole totali, di cui 21.168 con firme KG-derivate.

**Caso d'uso**: l'entità in produzione, che ha letto corpora (read-books), assimilato insegnamenti (teach-bigbang), e ha un lessico maturo.

Il lessico completo è quello che vive in `prometeo_topology_state.bin`. Il cardinale è essenzialmente un lessico per test e per dimostrazioni pedagogiche (cosa significa che l'entità nasce con poche parole).

---

## Capitolo 9 — Superficie pubblica

Per `WordPattern`:
- `new_unknown(word)`, `new_from_context(word, sig, aff)`, `new_known(word, sig, aff)` — costruttori
- `expose(ctx_sig, ctx_aff)` — aggiornamento via esposizione
- `recompute_affinities(all_aff)` — ricalcola affinità dalla firma
- `register_co_occurrence/negation/affirmation(other)` — aggiornamento co-contesti
- `is_stable()`, `dominant_fractal()`, `perturbation_strength()` — proprietà derivate

Per `Lexicon`:
- `new()`, `bootstrap()`, `bootstrap_cardinal()` — costruttori
- `knows(word)`, `get(word)`, `get_mut(word)`, `word_count()` — accesso
- `is_function_word(word)`, `classify_operator(word)` — classificazione
- `process_input(text, ...)` — processing di un input (tokenize + update)
- `apply_curated_signatures()` — applica le 134 firme curated
- `load_phenomenology_signatures()` — legge `data/kg/phenomenology.tsv`
- `most_stable(n)`, `tension_words()`, `patterns_iter()` — iteratori
- `position_on_axis(word, axis)`, `enriched_distance(w1, w2, axes)` — metriche derivate

### 9.1 — Cosa non è esposto e andrebbe esposto

Per un endpoint `/api/admin/lexicon/*`:

- **`signature_source(word) -> SignatureSource`**: l'enum suggerito in cap. 6. Richiede aggiungere il campo in `WordPattern`. Valore: sapere se una firma è KG-derivata, curated, contestuale.
- **`signature_age(word) -> u32`**: numero di tick/rederive da quando la firma è stata scritta l'ultima volta. Utile per capire "quanto è fresca".
- **`exposure_trajectory(word, limit) -> Vec<([f64; 8], u64)>`**: la storia delle firme a ogni esposizione. Richiederebbe salvare la traiettoria (costoso), ma prezioso per capire come una parola si è spostata nello spazio nel tempo.
- **`closest_words(word, n) -> Vec<(String, f64)>`**: le N parole con firme 8D più vicine. Oggi calcolabile con `enriched_distance` ma non esposto come endpoint.
- **`word_in_regime(word) -> Option<str>`**: "cardinal" | "bootstrap" | "curated" | "kg-derived" | "contextual". Sempre in mancanza di enum: può essere inferito (es. se la parola è in `curated` list → Curated; se è nel KG → KgDerived; ecc.). Fragile.

---

## Sintesi del volume

Il lessico è l'inventario di ~25.600 parole, ciascuna con una `signature` a 8 dimensioni in ordine I Ching canonico (post-Phase 68): Agency, Permanenza, Intensità, Tempo, Confine, Complessità, Definizione, Valenza.

Cinque regimi producono le firme, in ordine di autorevolezza: seed cardinale (43 parole hardcoded per l'infante), seed bootstrap (~300 parole per categorie), curated (~130 firme esplicite sulle parole core), **KG-derivata** (la Phase 63, ~21.000 parole derivate dalla struttura del grafo via `derive_8d_from_kg`), contestuale (nuove parole runtime via `perturb_towards` del contesto).

La funzione `derive_8d_from_kg` traduce topologia in geometria: ogni dimensione è una formula esplicita sui conteggi di archi. Agency = ratio CAUSES out/totale. Permanenza = scala discreta su IsA children. Intensità = CAUSES out + carica emotiva. Tempo = catene causali. Confine = specificità IsA + bonus polarità. Complessità = log grado / log max. Definizione = IsA parents + bonus polarità. Valenza = BFS propagata da 20 radici emotive (10 positive, 10 negative) con decadimenti per tipo di arco.

La stabilità cresce logaritmicamente con le esposizioni e modula il learning rate: parole nuove si avvicinano al contesto di 35%, parole stabili di meno del 2%.

Due debiti da nominare: (a) la provenienza della firma non è tracciata — non si sa leggendo un pattern se è curated o contestuale; (b) parole nuove apprese runtime che poi entrano nel KG non vengono automaticamente riderivate.

Cinque endpoint admin utili che oggi non esistono: source tracking, age, trajectory, closest words, regime classification.

Da qui il Volume 04 si sposta sul **KnowledgeGraph** — la quarta struttura ontologica, con le 21 relazioni tipate (5 categorie inclusa la fenomenologica, che Vol. 01 ha rivelato come sotto-popolata ma centralmente pesata) e le due funzioni di peso (`field_boost_strength` per la propagazione campo, `relation_weight` per le proposizioni).

---

*Prossimo volume: 04 — Fondamenti: KnowledgeGraph e le 21 relazioni* (in scrittura)
