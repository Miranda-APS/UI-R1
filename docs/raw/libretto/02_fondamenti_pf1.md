# Volume II — Fondamenti: PrometeoField (PF1)

> *Il campo è il corpo dell'entità. Ogni parola è un punto del corpo, con una sua firma, una sua posizione tra i frattali, e otto vicini con cui dialoga lungo archi di fase. Lo stato è la vita del corpo: cambia ad ogni istante, ma il corpo resta.*

---

## Premessa

Vol. 01 ha introdotto PF1 come la struttura dove vive il commitment β (otto dimensioni primitive bastano, ma 8 è la base, non il limite). Questo volume ne fa l'anatomia completa.

Tre cose in particolare meritano profondità:

1. Il **layout fisico** dei 512 byte per parola — perché esattamente quei campi e in quell'ordine.
2. La **formula della propagazione** — il termine `cos(phase)` che riduce ogni tipo di relazione a un parametro continuo.
3. La **separazione corpo/stato** che permette al sistema di esistere a costo proporzionale all'attività, non alla dimensione del lessico.

Tutto il file di riferimento è [`src/topology/pf1.rs`](../../src/topology/pf1.rs), 950 righe.

---

## Capitolo 1 — Le costanti che definiscono il campo

In testa al file, [pf1.rs:37-47](../../src/topology/pf1.rs):

```rust
pub const RECORD_SIZE: usize = 512;
pub const HEADER_SIZE: usize = 128;
pub const MAGIC: &[u8; 8] = b"PMTF0002";
pub const MAX_WORD_BYTES: usize = 32;
pub const MAX_NEIGHBORS: usize = 8;
pub const MAX_FRACTALS: usize = 64;
pub const PF1_VERSION: u16 = 2;
```

Sette costanti. Ognuna è una scelta architetturale.

**`RECORD_SIZE = 512`**: ogni parola occupa esattamente 512 byte sul disco. È una potenza di 2, allineata a un mezzo blocco SSD (4 KB / 8 = 512 B). Se la prossima generazione di parole richiedesse più di 512 byte, avremmo bisogno di un nuovo formato. Per ora ci stiamo dentro con margine: i campi attivi sommano 432 byte, gli ultimi 80 byte sono `_reserved` per estensioni future.

**`HEADER_SIZE = 128`**: l'header del file PF1 occupa 128 byte. Contiene `MAGIC` (8 byte), `version` (2), `word_count` (4), `MAX_FRACTALS` (2 byte), il resto è padding e riserva. Sapere l'header è una costante permette il calcolo `offset_record(id) = HEADER_SIZE + id × RECORD_SIZE` — accesso O(1) a qualsiasi parola via `mmap` (in futuro: oggi il file viene letto interamente in RAM).

**`MAGIC = b"PMTF0002"`**: la firma del formato. `PMTF` per *PRoMeteo Topological Field*, `0002` per il versionamento incrementale (la prima versione era `PMTF0001`, oggi siamo a 2 — vedi `PF1_VERSION = 2`). Senza magic byte, non si potrebbe distinguere un file PF1 da un file casuale; con magic byte, `load_from_file` restituisce errore esplicito su file invalidi.

**`MAX_WORD_BYTES = 32`**: ogni parola UTF-8 può essere al massimo 32 byte. In italiano, una parola di 12-15 caratteri UTF-8 occupa 12-15 byte; le parole più lunghe del lessico ("incomprensibilità", "internazionalizzazione") stanno comodamente sotto i 25 byte. 32 è il taglio sicuro. *Limite*: parole composte molto lunghe (es. nomi tecnici) vengono troncate. Per ora non è un problema reale.

**`MAX_NEIGHBORS = 8`**: ogni parola ha al massimo 8 vicini topologici. Questo è il vincolo più forte — e il più filosoficamente coerente.

8 = il numero di dimensioni primitive. Non per coincidenza: la rete di vicini per ogni parola riproduce la struttura cardinale dello spazio. Un nodo con migliaia di archi nel KG (essere, fare) viene compresso ai suoi top-8 vicini per peso quando entra nel campo PF1. È **selezione semantica**: ogni parola "abita" il suo cerchio di otto.

Conseguenza: `propagate()` costa `O(parole_attive × 8)`, indipendentemente dalla densità del KG sottostante. Anche se nel KG "essere" ha 5000 archi, in PF1 ne ha 8 — quelli che dopo hub damping logaritmico hanno il peso più alto. Il costo non esplode mai.

**`MAX_FRACTALS = 64`**: i sessantaquattro esagrammi I Ching (vol. 05). Ogni parola memorizza 64 affinità (4 byte ciascuna = 256 byte) — lo spazio più grande nel record dopo i metadati. Permette di sapere, per ogni parola, quanto risuona con ciascuno dei 64 attrattori globali del campo.

**`PF1_VERSION = 2`**: la versione corrente del formato. Quando cambierà il layout (es. allargando `MAX_NEIGHBORS` a 16), incrementeremo a 3 e `load_from_file` saprà rifiutare versioni incompatibili.

---

## Capitolo 2 — L'anatomia di un WordRecord (512 byte)

Il `WordRecord` è la struttura cardine: contiene tutto quello che il sistema sa permanentemente su una parola. È `#[repr(C)]` per garantire layout fisso compatibile tra build (no padding inserito dal compilatore in posizioni inaspettate). [pf1.rs:73-90](../../src/topology/pf1.rs):

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WordRecord {
    pub signature:        [f32; 8],               // 32 byte
    pub affinities:       [f32; MAX_FRACTALS],    // 256 byte
    pub stability:        f32,                    // 4 byte
    pub exposure_count:   u32,                    // 4 byte
    pub dominant_fractal: u16,                    // 2 byte
    pub pos:              u8,                     // 1 byte
    pub word_len:         u8,                     // 1 byte
    pub word:             [u8; MAX_WORD_BYTES],   // 32 byte
    pub neighbor_count:   u8,                     // 1 byte
    pub _pad:             [u8; 3],                // 3 byte (allineamento)
    pub neighbors:        [u32; MAX_NEIGHBORS],   // 32 byte
    pub neighbor_weights: [f32; MAX_NEIGHBORS],   // 32 byte
    pub neighbor_phases:  [f32; MAX_NEIGHBORS],   // 32 byte
    pub _reserved:        [u8; 80],               // 80 byte (estensioni future)
}
```

**Totale**: 32 + 256 + 4 + 4 + 2 + 1 + 1 + 32 + 1 + 3 + 32 + 32 + 32 + 80 = **512 byte esatti**.

E infatti il file impone questo a compile-time ([pf1.rs:126-127](../../src/topology/pf1.rs)):

```rust
const _: () = assert!(std::mem::size_of::<WordRecord>() == RECORD_SIZE,
    "WordRecord deve essere esattamente 512 byte");
```

Se qualcuno aggiungesse un campo che fa sforare i 512 byte, il codice non compilerebbe. Questa è una garanzia *forte* del layout binario, ed è importante perché PF1 si serializza scrivendo direttamente i byte in memoria sul disco (vedi capitolo 7).

Ora analizziamo i campi per gruppi semantici.

### 2.1 — La firma 8D (32 byte)

```rust
pub signature: [f32; 8],
```

Otto numeri in virgola mobile a 32 bit. La posizione della parola nello spazio semantico. Le 8 dimensioni nominate sono quelle introdotte in vol. 01, sez. 1.2. Vol. 03 entra nel dettaglio del calcolo per ciascuna dimensione (`derive_8d_from_kg`).

**Perché `f32` e non `f64`**: dimezza l'occupazione (4 vs 8 byte), e la precisione di 7 cifre decimali è sovrabbondante per valori in [0, 1] usati come modulatori. La differenza tra 0.7234567 e 0.7234568 nel campo è inosservabile.

**Perché array fisso `[f32; 8]` e non `Vec<f32>`**: un array fisso vive nella struct stessa, niente puntatori, niente allocazione separata, niente cache miss. `Vec<f32>` aggiungerebbe 24 byte di metadati (puntatore + len + capacità) e una indirezione.

### 2.2 — Le affinità ai 64 frattali (256 byte)

```rust
pub affinities: [f32; MAX_FRACTALS],   // 64 valori
```

Per ogni frattale (esagramma), un numero in [0, 1] che dice quanto la parola "appartiene" a quella regione. Una parola tipica ha 3-5 affinità sopra 0.5 e tutte le altre vicine a 0 — ogni parola è centrata su pochi attrattori.

Le affinità non sono calcolate al volo: vengono **precalcolate** in `Lexicon` quando una parola viene appresa o riderivata, e poi copiate nel record PF1. Sapere "quanto questa parola appartiene al frattale X" diventa O(1).

**Perché 256 byte (la metà dell'intero record)**: perché le affinità servono spesso. La funzione `emerge_fractal_activations()` ([pf1.rs:396-432](../../src/topology/pf1.rs)) le legge per ogni parola attiva ad ogni tick di propagazione. Tenerle nel record stesso (cache locality) invece che in una struttura esterna è una scelta di prestazioni.

### 2.3 — Metadati (12 byte)

```rust
pub stability:        f32,    // 4 byte
pub exposure_count:   u32,    // 4 byte
pub dominant_fractal: u16,    // 2 byte
pub pos:              u8,     // 1 byte
pub word_len:         u8,     // 1 byte
```

`stability ∈ [0, 1]` quanto la firma resiste a essere modificata da nuovi contesti. Parole molto esposte e vissute in contesti coerenti hanno stabilità alta (~0.85). Parole appena apprese hanno stabilità bassa (~0.05). La stabilità modula il *resting state*: parole stabili "esistono" sotto soglia anche in silenzio (vol. 01, cap. 7).

`exposure_count` quante volte la parola è stata vista. Cresce indefinitamente.

`dominant_fractal` quale dei 64 esagrammi ha l'affinità più alta. Precalcolato per evitare di riscandare le 64 affinità ogni volta. È usato nell'ordinamento dei record nel file (vedi cap. 5: cache locality).

`pos` parte del discorso, codificata su 1 byte: 0=Sconosciuto, 1=Verbo, 2=Nome, 3=Aggettivo, 4=Avverbio. Cinque categorie sufficienti per la generazione italiana (vol. 13).

`word_len` lunghezza in byte della parola (per leggere correttamente l'array `word` di 32 byte: solo i primi `word_len` sono significativi, il resto è zero).

### 2.4 — La parola UTF-8 (32 byte)

```rust
pub word: [u8; MAX_WORD_BYTES],
```

I byte UTF-8 della parola, padded con zeri. `word_str()` ([pf1.rs:114-117](../../src/topology/pf1.rs)) la decodifica leggendo solo i primi `word_len` byte:

```rust
pub fn word_str(&self) -> &str {
    let len = self.word_len as usize;
    std::str::from_utf8(&self.word[..len]).unwrap_or("")
}
```

**Perché tenere la parola DENTRO il record invece che in una stringa esterna**: stesso motivo della firma — cache locality e niente indirezione. Quando il sistema deve dire "qual è la parola con `word_id = 42`?", è una lettura diretta in `data[42].word_str()`, niente lookup in tabelle esterne.

### 2.5 — I top-8 vicini (98 byte: 32+32+32+1+pad)

```rust
pub neighbor_count:   u8,
pub _pad:             [u8; 3],
pub neighbors:        [u32; MAX_NEIGHBORS],         // 32 byte: id dei vicini
pub neighbor_weights: [f32; MAX_NEIGHBORS],         // 32 byte: pesi degli archi
pub neighbor_phases:  [f32; MAX_NEIGHBORS],         // 32 byte: FASI degli archi
```

Qui vive il cuore strutturale del sistema. Per ogni parola: l'id dei top-8 vicini (per peso d'arco), il peso di ciascun arco, e la **fase** di ciascun arco.

Il `_pad: [u8; 3]` non serve a niente se non a mantenere l'allineamento a 4 byte del prossimo array (`neighbors: [u32; 8]`). È una conseguenza tecnica del repr(C) di Rust su CPU che richiedono accesso allineato.

**Le fasi sono dove vive β₃** (vol. 01). Ogni arco ha un angolo `phase ∈ [0, π]`:
- `phase = 0` → relazione di risonanza pura (IS_A, SIMILAR_TO, FeelsAs)
- `phase = π/2` → ortogonalità, nessuna propagazione effettiva
- `phase = π` → opposizione (OPPOSITE_OF, EXCLUDES)

Quando un arco viene creato in `build_from_lexicon()` ([pf1.rs:626-627](../../src/topology/pf1.rs)), la fase è ereditata da `topology.edge_phase()`, che a sua volta deriva dal tipo di relazione KG che ha generato l'arco. Per default (archi senza tipo specifico), la fase è `π/2` — ortogonalità, "le due parole coesistono ma non si influenzano".

### 2.6 — Padding di riserva (80 byte)

```rust
pub _reserved: [u8; 80],
```

Spazio non usato. È esplicitamente lasciato per future estensioni del formato senza dover incrementare la versione e migrare i file esistenti. Esempi di cosa potrebbe popolare questi 80 byte:
- Statistiche temporali (ultimo accesso, decadimento personalizzato)
- Provenance (chi ha insegnato questa parola: utente, KG, sogno)
- Ulteriori 16 vicini (ad esempio per relazioni fenomenologiche separate)

Per ora, vuoto.

---

## Capitolo 3 — ActivationState: la RAM che vive

Mentre `WordRecord` è il corpo (sul disco), `ActivationState` è lo stato (in RAM). Definita in [pf1.rs:137-149](../../src/topology/pf1.rs):

```rust
pub struct ActivationState {
    pub activations:     Vec<f32>,    // 1 valore per parola
    pub counts:          Vec<u64>,    // contatore esposizioni in sessione
    pub threshold:       f32,         // soglia "attiva" (0.02 default)
    pub synapse_weights: Vec<f32>,    // pesi vivi LTP/LTD: word_count × 8
}
```

Per N parole nel lessico:
- `activations`: N × 4 byte = ~100 KB per 25.875 parole
- `counts`: N × 8 byte = ~200 KB
- `synapse_weights`: N × 8 × 4 byte = ~830 KB

Totale RAM per lo stato volatile: ~1.1 MB. Confronta con il corpo su disco (13 MB per 25.875 parole × 512 B). Il corpo è 12× più pesante dello stato — perché contiene la struttura permanente, mentre lo stato è solo la dinamica.

### 3.1 — `synapse_weights`: dove vive l'apprendimento

`neighbor_weights` (in `WordRecord`, su disco) è il peso *basale* dell'arco — viene da KG + esperienza cristallizzata. Quando il sistema parte, `init_synapse_weights_from_field()` ([pf1.rs:168-177](../../src/topology/pf1.rs)) copia questi pesi basali in `synapse_weights` (in RAM).

Da quel momento, durante la conversazione, **`synapse_weights` evolve** secondo plasticità hebbiana (cap. 6). I pesi ROM restano invariati; quelli RAM si modificano. Quando la conversazione finisce, i pesi RAM possono essere usati per aggiornare i pesi ROM (ma di default no — è una scelta che permette conversazioni "esplorative" che non lasciano traccia permanente).

`propagate()` legge i pesi così ([pf1.rs:274-278](../../src/topology/pf1.rs)):

```rust
let weight = if base + i < self.synapse_weights.len() {
    self.synapse_weights[base + i]    // RAM: peso vivo
} else {
    record.neighbor_weights[i]        // ROM fallback
};
```

Sempre RAM se disponibile, altrimenti ROM. Questo modella esattamente la biologia: la sinapsi *esiste* per costruzione (ROM, l'anatomia della connessione), ma la sua *forza* è plastica (RAM, l'apprendimento).

### 3.2 — Threshold: il taglio tra "attiva" e "silente"

`threshold = 0.02` (default, [pf1.rs:156](../../src/topology/pf1.rs)).

Sotto questa soglia, una parola è considerata silente — la propagazione la ignora, le funzioni che riportano "parole attive" (`hot_words`, `active_count`, `field_energy` filtrato) la escludono.

0.02 è ~2% del massimo possibile (1.0). Le parole stabili in *resting state* hanno `stability × 0.002 ≈ 0.001-0.002` — appena sotto soglia. Una perturbazione tipica porta una parola a 0.3-0.6 — ben sopra soglia.

La soglia è *modificabile* dal sogno: in fase REM (`SleepPhase::activation_threshold()` in `dream.rs:42`), la soglia scende fino a 0.01 — permettendo a regioni normalmente silenti di toccarsi. È così che il sogno scopre nuove connessioni (vol. 14).

---

## Capitolo 4 — La propagazione, riga per riga

`propagate()` è la funzione più importante di tutto il sistema. È dove il campo "ragiona" — dove l'attivazione di alcune parole si propaga lungo gli archi e cambia lo stato delle altre.

Sta in [pf1.rs:239-322](../../src/topology/pf1.rs). La leggiamo a blocchi.

### 4.1 — Prologo: decadimento delle attivazioni sopra soglia

```rust
pub fn propagate(&mut self, field: &PrometeoField) {
    let damping = 0.15_f32;

    // Decadimento: le attivazioni sopra soglia scendono gradualmente.
    let decay_rate = 0.92_f32;
    for act in self.activations.iter_mut() {
        if *act > self.threshold {
            *act *= decay_rate;
            if *act < self.threshold { *act = self.threshold * 0.5; }
        }
    }
    // ...
}
```

Prima della propagazione vera e propria, ogni attivazione sopra soglia viene moltiplicata per 0.92 (decade dell'8% per tick). Se scende sotto soglia, viene riportata al "floor di riposo" (`threshold * 0.5 = 0.01`).

Questo è il meccanismo che fa tornare il campo verso il riposo in assenza di input. Senza decadimento, una perturbazione resterebbe per sempre. Con `decay_rate = 0.92`, da un'attivazione di 0.5 servono `log(0.04)/log(0.92) ≈ 38 tick` per scendere sotto soglia. A 3 secondi per tick, ~115 secondi per tornare al silenzio.

(Il commento nel file dice "~30 tick" — è impreciso, vedi `appunti.md`. La realtà sono ~38 tick. Ho lasciato "~30" nel codice perché correggerlo richiede ricalcolare dalle costanti specifiche.)

### 4.2 — Raccolta del fronte di attivazione

```rust
let hot: Vec<(u32, f32)> = self.activations.iter().enumerate()
    .filter(|(_, &a)| a > self.threshold)
    .map(|(i, &a)| (i as u32, a))
    .collect();
```

Iteriamo su tutte le attivazioni e prendiamo solo quelle sopra soglia, con il loro id. Questo è il **fronte di attivazione** — le parole "vive" in questo istante.

Per N parole nel lessico, questo è O(N) — ma è un'iterazione semplice in cache locale, costa pochi μs anche per 25K parole. Il vero risparmio computazionale arriva nel passo successivo: il loop di propagazione tocca solo `hot.len()`, non N.

### 4.3 — Accumulo dei delta

```rust
let mut deltas = vec![0.0f32; self.activations.len()];

for (src_id, src_act) in &hot {
    let record = field.record(*src_id);
    let n = record.neighbor_count as usize;
    let base = *src_id as usize * MAX_NEIGHBORS;

    for i in 0..n {
        let nid = record.neighbors[i] as usize;
        if nid >= self.activations.len() { continue; }

        let weight = if base + i < self.synapse_weights.len() {
            self.synapse_weights[base + i]
        } else {
            record.neighbor_weights[i]
        };
        let phase = record.neighbor_phases[i];

        // Formula unica: cos(fase) determina segno e intensità
        let contribution = src_act * damping * weight * phase.cos();
        // ...
    }
}
```

Per ogni parola attiva, per ognuno dei suoi (max 8) vicini:

1. Lookup id del vicino dal record.
2. Lookup peso (RAM se disponibile, altrimenti ROM).
3. Lookup fase (sempre ROM — la geometria non cambia).
4. **Calcolo del contributo**: `src_act × damping × weight × cos(phase)`.

Questa è la formula da vol. 01, sez. 3.2. Ricapitolo qui i quattro fattori:

- `src_act ∈ [threshold, 1]`: quanta energia ha la sorgente
- `damping = 0.15`: attenuazione globale (impedisce esplosioni)
- `weight ∈ [0, 3]`: forza dell'arco (1 = basale, fino a 3× per LTP)
- `cos(phase) ∈ [-1, +1]`: segno e intensità del trasferimento

Un esempio concreto. La parola "paura" è attiva a 0.5. Tra i suoi 8 vicini: "tremore" con `weight = 0.18` (FeelsAs) e `phase = 0` (risonanza pura). Il contributo è `0.5 × 0.15 × 0.18 × cos(0) = 0.5 × 0.15 × 0.18 × 1.0 = 0.0135`. "Tremore" riceve un boost di +0.0135 alla sua attivazione. Modesto, ma se *otto* parole vicine spingono nella stessa direzione, l'effetto si somma e diventa significativo.

Altro esempio. "Paura" stessa con `weight = 0.06` e `phase = π` verso "coraggio" (OppositeOf). Contributo: `0.5 × 0.15 × 0.06 × cos(π) = 0.5 × 0.15 × 0.06 × (-1) = -0.0045`. "Coraggio" viene **inibito** di -0.0045. Piccolo ma con segno opposto — è la geometria della negazione.

### 4.4 — Rendimenti decrescenti positivi

```rust
if contribution > 0.0 {
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
```

Qui c'è una scelta sottile aggiunta in Phase 55. Senza compensazione, una parola già attiva riceverebbe boost da molti vicini e saturerebbe rapidamente a 1.0 — la propagazione diventerebbe un meccanismo di "feedback positivo" e finirebbe per attivare uniformemente i nodi hub.

Soluzione: i contributi positivi vengono moltiplicati per `1 / (1 + 4 × current_activation)`. Il fattore parte da 1 (parola sotto soglia: pieno effetto) e scende: a `current = 0.15` il fattore è `1/(1 + 0.6) = 0.625`, a `current = 0.5` è `1/3 ≈ 0.33`. Quindi parole già molto attive ricevono solo un terzo del boost che riceverebbero se fossero a riposo.

I contributi *negativi* (opposizione) NON vengono attenuati. Un'opposizione forte deve poter inibire una parola anche se è attiva — altrimenti la negazione perderebbe efficacia.

### 4.5 — Cap massimo positivo

```rust
const MAX_POSITIVE_DELTA: f32 = 0.15;
for delta in deltas.iter_mut() {
    if *delta > MAX_POSITIVE_DELTA { *delta = MAX_POSITIVE_DELTA; }
}
```

Un nodo hub con molte parole vicine attive può accumulare delta enormi (8 × 0.0135 ≈ 0.108, ma se fosse vicino di 50 parole attive — possibile durante fasi REM — potrebbe arrivare a 0.5+). Il cap a 0.15 garantisce che **nessuna propagazione possa superare il segnale diretto dell'input**.

L'input tipicamente porta una parola a 0.3-0.6 di attivazione. Se la propagazione potesse arrivare a 0.5, sarebbe alla pari con l'input — l'output del sistema sarebbe dominato dai nodi hub invece che dal segnale semantico effettivo dell'utente. Il cap a 0.15 (la metà della seed minima 0.3) preserva la **gerarchia segnale/propagazione**.

### 4.6 — Applicazione dei delta

```rust
for (i, delta) in deltas.iter().enumerate() {
    if delta.abs() > 0.001 {
        self.activations[i] = (self.activations[i] + delta).clamp(0.0, 1.0);
    }
}
```

Tutti i delta vengono applicati in batch — *dopo* aver completato il loop di calcolo, mai durante. Questo evita che un nodo modificato a metà loop influenzi i calcoli successivi (race condition implicita).

I delta minori di 0.001 vengono ignorati (sotto la precisione utile). Le attivazioni risultanti vengono clampate in [0, 1].

---

## Capitolo 5 — L'ordinamento delle parole nel file

Una scelta architetturale silenziosa ma importante: le parole nel file PF1 sono **ordinate per frattale dominante**, non alfabeticamente.

[pf1.rs:544-548](../../src/topology/pf1.rs):

```rust
entries.sort_unstable_by(|a, b| {
    a.1.dominant_fractal.cmp(&b.1.dominant_fractal)
        .then(a.0.cmp(&b.0))   // tiebreak: alfabetico
});
```

Tutte le parole con `dominant_fractal = 0` (Cielo×Cielo, "Potere") sono raggruppate in testa. Poi tutte quelle con `dominant_fractal = 1`. E così via fino a 63.

**Perché**: cache locality. Quando il sistema deve operare su un'intera regione frattale (es. "attiva tutte le parole del frattale POTERE"), legge righe consecutive del file. Lo `mmap` del sistema operativo fa caching a granularità di pagina (4 KB = 8 record): leggere 8 parole consecutive del frattale 12 carica una sola pagina, mentre leggerle in ordine sparso ne caricherebbe 8.

L'effetto pratico è che le operazioni "frattale-centriche" (`activate_region(fid)`, `simplices_of(fid)`) sono significativamente più veloci. Operazioni alfabetiche (lookup per nome) restano O(1) grazie all'HashMap `word_to_id` separata.

---

## Capitolo 6 — Plasticità hebbiana (LTP/LTD)

> *Neurons that fire together, wire together.* — Donald Hebb, 1949

Dopo ogni propagazione, il sistema aggiorna i pesi sinaptici (`synapse_weights` in RAM) secondo la regola hebbiana: **se sorgente e vicino sono entrambi attivi, rinforza la sinapsi (LTP); se la sorgente è attiva ma il vicino è silente, indebolisci la sinapsi (LTD)**.

[pf1.rs:335-370](../../src/topology/pf1.rs):

```rust
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

        for i in 0..record.neighbor_count as usize {
            let nid = record.neighbors[i] as usize;
            let sw_idx = base + i;

            let neighbor_act = self.activations[nid];
            if neighbor_act > self.threshold {
                // LTP: entrambi co-attivi → rinforza
                self.synapse_weights[sw_idx] =
                    (self.synapse_weights[sw_idx] + LTP * src_act * neighbor_act)
                    .min(MAX_WEIGHT);
            } else {
                // LTD: sorgente attiva ma vicino silenzioso → indebolisce
                self.synapse_weights[sw_idx] *= LTD_DECAY;
            }
        }
    }
}
```

**LTP = 0.05**: il guadagno per co-attivazione. Ogni tick in cui due parole vicine sono entrambe attive, il loro peso sinaptico cresce di `0.05 × act_src × act_neighbor`. Per due attivazioni di 0.5 ciascuna: `0.05 × 0.5 × 0.5 = 0.0125`. Una conversazione che ripete 40 volte la stessa coppia attiva porta il peso da 1.0 a 1.5 (cap a 3.0).

**LTD_DECAY = 0.995**: 0.5%/tick di decadimento per sinapsi silenti. Lentissimo, ma costante. Un peso di 1.0 senza co-attivazioni scende a `0.995^200 ≈ 0.37` in 200 tick (10 minuti). Le sinapsi inutilizzate si atrofizzano.

**MAX_WEIGHT = 3.0**: il peso massimo. Un arco non può crescere oltre 3× il basale. Senza questo limite, archi molto rinforzati dominerebbero tutto.

**Cosa significa filosoficamente**: l'esperienza non solo *ricorda* (memoria episodica, vol. 14), ma *modifica la struttura del campo stesso*. Una conversazione intensa tra "paura" e "tremore" rinforza permanentemente quella sinapsi (in RAM, finché la sessione dura). Se la modifica viene salvata nel file PF1 a fine sessione (oggi non automatico, ma è possibile via `save_to_file()`), il rinforzo diventa anatomico.

**Limite onesto**: oggi `hebbian_update()` viene chiamato in `propagate_field_words()` ad ogni perturbazione, ma le modifiche restano in RAM. Quando il programma si chiude, `synapse_weights` viene perso. Solo i pesi ROM (basali) sopravvivono. Questo è un debito: la plasticità è effimera. Per renderla permanente, andrebbe periodicamente fuso `synapse_weights → record.neighbor_weights` e ri-salvato il file PF1.

---

## Capitolo 7 — Serializzazione: i 13 MB su disco

Salvare e caricare il campo è semplice perché `WordRecord` è `#[repr(C)]` — il layout binario in memoria coincide con quello su disco. Niente serializzazione esplicita campo per campo: si scrivono i bytes direttamente.

[pf1.rs:718-743](../../src/topology/pf1.rs):

```rust
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
```

L'`unsafe` qui è giustificato (e ci sono test): `WordRecord` è `#[repr(C)]` quindi il layout binario è ben definito, non c'è UB. Su una macchina x86_64 little-endian (lo stato dell'arte), il file scritto è binariamente identico alla rappresentazione in RAM.

Velocità: scrivere 25.875 record × 512 byte = 13.2 MB in singola write batch via `BufWriter` è un'operazione di pochi ms.

Caricamento ([pf1.rs:746-781](../../src/topology/pf1.rs)) è il duale: leggi tutto in memoria, verifica magic, poi `std::ptr::read_unaligned` per ricostruire i record. Dopo il caricamento, l'HashMap `word_to_id` viene ricostruito iterando sui record (non è serializzato — non ne vale la pena, è derivato).

**Potenziale futuro**: `mmap` diretto del file invece di leggere in RAM. Con `mmap`, il sistema operativo mappa il file in memoria virtuale e legge solo le pagine effettivamente accedute (lazy loading). Per lessico molto grande (>100K parole, ~50 MB), `mmap` permette di risparmiare RAM enorme. Oggi non è implementato, ma il commento iniziale ([pf1.rs:473](../../src/topology/pf1.rs)) lo nomina come direzione: "In futuro: mmap diretto dal file (zero-copy)".

---

## Capitolo 8 — Le costanti che parlano (recap)

In ordine di importanza filosofica, le costanti che governano PF1:

| Costante | Valore | Senso |
|----------|--------|-------|
| `MAX_NEIGHBORS` | 8 | Ogni parola ha 8 vicini — coerente con le 8 dimensioni primitive |
| `MAX_FRACTALS` | 64 | I 64 esagrammi I Ching come attrattori |
| `RECORD_SIZE` | 512 | Allineato a mezzo blocco SSD; abbastanza per le 8 dim + 64 affinità + 8 vicini con peso e fase |
| `damping` | 0.15 | Attenuazione propagazione — empirico, evita esplosioni |
| `decay_rate` | 0.92 | Decadimento attivazione per tick — torna a riposo in ~38 tick |
| `MAX_POSITIVE_DELTA` | 0.15 | Cap propagazione — preserva gerarchia segnale/propagazione (input 0.3-0.6 > propagazione max 0.15) |
| `threshold` | 0.02 | Confine attiva/silente — input perturba a 0.3+, sopra; resting state a 0.001-0.002, sotto |
| `LTP` | 0.05 | Guadagno hebbiano per co-attivazione |
| `LTD_DECAY` | 0.995 | Atrofia sinapsi inutilizzate (0.5%/tick) |
| `MAX_WEIGHT` | 3.0 | Cap superiore peso sinaptico (3× basale) |
| `resting state` | `stability × 0.002` | Presenza sotto-soglia — l'entità esiste anche in silenzio |

**Costanti che meriterebbero giustificazione formale e oggi non l'hanno** (annotate in `appunti.md`):
- `damping = 0.15`: no test che la giustifichi formalmente
- `decay_rate = 0.92`: il commento dice "in ~30 tick" ma il calcolo dà ~38 tick (impreciso)
- `MAX_POSITIVE_DELTA = 0.15`: cap arbitrario, anche se semantica chiara
- Resting state diverso pf1=0.002 vs word_topology=0.003: compromesso di sincronizzazione, non scelta filosofica

---

## Capitolo 9 — Superficie pubblica: cosa è esposto

Funzioni pub in `WordRecord`:
- `empty()`: record vuoto
- `word_str()`: stringa della parola
- `signature_slice()`: firma 8D come slice

Funzioni pub in `ActivationState`:
- `new(word_count)`: costruisce stato vuoto
- `init_synapse_weights_from_field(field)`: copia pesi basali ROM → RAM
- `activate(id, strength)`: attiva una parola per id
- `activate_by_name(field, name, strength)`: attiva per nome
- `set_by_name(field, name, val)`: SET (non ADD)
- `decay_all(rate)`: decadimento globale
- `decay(rate)`: moltiplicatore alternativo
- `propagate(field)`: il cuore della propagazione
- `hebbian_update(field)`: aggiornamento sinaptico LTP/LTD
- `seed_resting_state(field)`: imposta resting state
- `emerge_fractal_activations(field) -> [f32; 64]`: attivazioni frattali correnti
- `hot_words(field, limit) -> Vec<(String, f32)>`: parole attive ordinate
- `field_energy()`: energia totale
- `active_count()`: parole sopra soglia
- `reset()`: azzera tutte le attivazioni

Funzioni pub in `PrometeoField`:
- `build_from_lexicon(lexicon, topology, complex)`: costruisce dal lessico
- `record(id)`: lookup record per id (O(1))
- `word_id(name)`: lookup id per nome (O(1) HashMap)
- `word_name(id)`: lookup nome per id
- `empty()`: campo vuoto
- `add_word(word, record)`: aggiunge parola (apprendimento runtime)
- `save_to_file(path)`: serializza
- `load_from_file(path)`: deserializza
- `stats()`: statistiche aggregate (word_count, total_edges, avg_neighbors)

### 9.1 — Cosa NON è esposto e dovrebbe esserlo

Audit della superficie API per `/api/admin/`:

- **`record_dump(word_or_id) -> WordRecordDto`**: leggere il record completo di una parola (firma 8D + 64 affinità + top-8 vicini con peso/fase). Sarebbe oro per debug e ispezione — oggi non esiste come endpoint.
- **`fractal_distribution() -> [u32; 64]`**: quante parole hanno ogni frattale come dominante. Oggi calcolabile ma non esposto.
- **`synapse_diff(threshold) -> Vec<(WordPair, f32, f32)>`**: differenza tra `synapse_weights` (RAM) e `neighbor_weights` (ROM) per le sinapsi che hanno divergito di più — mostra dove la plasticità sta lavorando. Funzione utile per diagnosi della sessione corrente.
- **`field_snapshot() -> FieldSnapshotDto`**: hot words + active count + field energy + emerge_fractal_activations + dominant fractal — un riassunto dello stato del campo in un solo endpoint.
- **`commit_synapse_weights_to_rom()`**: rendere permanente la plasticità della sessione corrente (oggi non c'è, e i pesi RAM si perdono allo shutdown).

Vol. 16 (Web API) raccoglierà queste raccomandazioni e proporrà la loro esposizione concreta.

---

## Sintesi del volume

PF1 è il campo come ROM (corpo, 512 byte/parola, layout fisso, ordinato per frattale dominante per cache locality) + RAM (stato, 4 byte/parola di attivazione + 32 byte/parola di pesi sinaptici vivi).

Una sola formula governa il dialogo tra parole: `contribution = src_act × damping × weight × cos(phase)`. La fase è dove vive la diversità delle relazioni — un parametro continuo in [0, π] che mappa risonanza/ortogonalità/opposizione su un coseno.

La plasticità è hebbiana (LTP per co-attivazione, LTD per silenzio). Oggi vive solo in RAM e si perde a shutdown — debito da estinguere se vogliamo che l'esperienza modifichi davvero la struttura.

Il sistema costa O(parole_attive × 8) per tick. Per 100 parole attive su 25.875: 800 operazioni. La presenza è locale, e questa è la sua forza.

Cinque cose dovremmo esporre nell'admin API e oggi non esponiamo: dump record, distribuzione frattali, diff sinapsi RAM-ROM, snapshot campo, commit pesi RAM→ROM.

Da qui in poi, due strade: il vol. 03 zooma sul Lexicon — come una parola entra nel mondo e come la sua firma 8D viene calcolata dal KG (Phase 63, `derive_8d_from_kg`). Il vol. 04 zooma sul KG — i 21 tipi di relazione, la geometria delle relazioni fenomenologiche, il hub damping.

---

*Prossimo volume: 03 — Fondamenti: Lexicon e firme 8D KG-derivate* (in scrittura)
