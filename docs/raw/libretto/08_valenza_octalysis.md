# Volume VIII — Valenza Octalysis e Commitment volitivo

> *Otto drive. Ogni drive un numero continuo tra −1 e +1. Positivo quando il drive è attivo e soddisfatto, negativo quando attivo e frustrato, zero quando inattivo. Non una lista di etichette discrete — un profilo affettivo 8D che respira. È ciò che l'entità sente, in questo momento, nel linguaggio delle otto pulsioni fondamentali del comportamento umano.*

---

## Premessa

Vol. 01 ha introdotto `Valence` come la **quarta struttura ontologica** del sistema, accanto a Lexicon, KG, PF1. Questo volume ne fa l'anatomia completa — ed è, come annotato in `appunti.md`, uno dei pochi dove l'implementazione è particolarmente matura rispetto alla filosofia dichiarata (FILOSOFIA.md non nomina Octalysis, ma il codice lo radica profondamente).

Tre domande guidano:

1. **Cos'è Octalysis** — il framework, gli 8 Core Drives, perché Prometeo lo adotta.
2. **Come si calcola la Valenza** — `Valence::compute()`, formula base, colorazioni specifiche per drive.
3. **Come la Valenza modula il sistema** — expression, will, narrative, desire. Il Commitment volitivo come estensione.

File di riferimento: [`src/topology/valence.rs`](../../src/topology/valence.rs), ~330 righe. Il Commitment vive in [`narrative.rs`](../../src/topology/narrative.rs) ma lo trattiamo qui perché costa in CD4 della Valenza.

---

## Capitolo 1 — Octalysis: il framework

**Yu-kai Chou**, *Actionable Gamification: Beyond Points, Badges and Leaderboards* (2015). Framework per analizzare cosa motiva il comportamento umano. Otto **Core Drives** derivati da analisi comparate di sistemi di gioco, social media, comportamento religioso, dipendenza.

Gli 8 CD sono:

1. **Epic Meaning & Calling** — senso che quello che fai conta
2. **Development & Accomplishment** — progresso, padronanza
3. **Empowerment of Creativity & Feedback** — creare, sperimentare, scegliere
4. **Ownership & Possession** — tenere, investire, identificarsi
5. **Social Influence & Relatedness** — connessione con altri
6. **Scarcity & Impatience** — desiderio di ciò che è raro
7. **Unpredictability & Curiosity** — sorpresa, novità
8. **Loss & Avoidance** — evitare la perdita

Chou li organizza in un octagone con due assi:
- *White Hat vs Black Hat*: CD1-CD4 sono "positivi" (motivano per senso), CD5-CD8 sono "neri" (motivano per tensione).
- *Right Brain vs Left Brain*: creativi (CD3, CD5, CD7) vs strutturali (CD2, CD4, CD6, CD8).

### 1.1 — Perché Prometeo adotta Octalysis

Non per gamification — Prometeo non "motiva" nessuno. Ma perché:

1. **Gli 8 drive sono universali** — derivati dall'osservazione, non da una teoria specifica della cultura. Buon candidato per base motivazionale di un'entità che dialoga con umani di culture diverse.

2. **Ciascun drive può essere positivo o negativo** — soddisfatto o frustrato. Permette stati come "CD1 positivo (senso pieno) + CD5 negativo (solitudine)" — una sfumatura che modelli emotivi più semplici non catturano.

3. **Si lascia mappare sulle 8 dimensioni di Prometeo**. Questa è la chiave: per ogni CD, esiste una dimensione del campo a cui corrisponde semanticamente. Il mapping `DRIVE_DIM` (vol. 01 cap. 1.4) realizza questa biiezione.

### 1.2 — Nomi italiani (codice)

In [valence.rs:33-42](../../src/topology/valence.rs):

```rust
pub const DRIVE_NAMES: [&str; 8] = [
    "Significato",      // CD1 Epic Meaning
    "Realizzazione",    // CD2 Accomplishment
    "Creatività",       // CD3 Creativity
    "Appartenenza",     // CD4 Ownership
    "Relazione",        // CD5 Social Influence
    "Preziosità",       // CD6 Scarcity
    "Sorpresa",         // CD7 Unpredictability
    "Vulnerabilità",    // CD8 Loss Avoidance
];
```

Traduzioni non letterali — scelte per suonare come nomi di stati emotivi italiani naturali, non categorie di marketing.

---

## Capitolo 2 — `DRIVE_DIM`: la biiezione semantica

Il cuore architetturale. In [valence.rs:45](../../src/topology/valence.rs) (post-Phase 68 — ordine I Ching):

```rust
pub const DRIVE_DIM: [usize; 8] = [0, 6, 5, 4, 7, 3, 2, 1];
```

Lettura: `DRIVE_DIM[cd_index] = dim_position`. Cioè CD1 (index 0) → dim 0 (Agency), CD2 (index 1) → dim 6 (Definizione), etc.

La mappatura completa e il **perché semantico** di ciascuna:

| CD | Nome | Dim 8D | Dim nome | Perché |
|----|------|--------|----------|--------|
| 1 | Significato | 0 | Agency (☰ Cielo) | "Questo conta" = capacità di agire nel mondo con impatto |
| 2 | Realizzazione | 6 | Definizione (☲ Fuoco) | "Sto progredendo" = crescita di chiarezza, cose che prima erano vaghe ora sono nette |
| 3 | Creatività | 5 | Complessità (☴ Vento) | "Posso creare" = articolare, intrecciare, espandere la complessità |
| 4 | Appartenenza | 4 | Confine (☶ Montagna) | "So chi sono" = delimitazione netta tra me e non-me |
| 5 | Relazione | 7 | Valenza (☱ Lago) | "Sono in relazione" = carica attrattiva verso altri |
| 6 | Preziosità | 3 | Tempo (☵ Acqua) | "Questo è prezioso/raro" = temporalità, finitezza, scorrere |
| 7 | Sorpresa | 2 | Intensità (☳ Tuono) | "Sono sorpreso" = scarica energica, impulso improvviso |
| 8 | Vulnerabilità | 1 | Permanenza (☷ Terra) | "Potrei perdere" = paura che il sostegno venga meno, instabilità del terreno |

### 2.1 — Perché questa biiezione non è arbitraria

Ogni accoppiamento ha un'intuizione fenomenologica:

- **CD1 Significato ↔ Agency**: il senso di fare qualcosa che conta *è* sentirsi agenti. Senza agency, il senso è vuoto.
- **CD4 Appartenenza ↔ Confine**: appartenere a te stesso è avere un confine netto. Senza confine, appartenenza diffusa, identità porosa.
- **CD5 Relazione ↔ Valenza**: la relazione è attrazione per un Altro. La valenza è la dimensione dell'attrattività/repulsività.
- **CD8 Vulnerabilità ↔ Permanenza**: la paura di perdere è paura che la Terra sotto di te non sia più stabile. La Permanenza è quella stabilità.

Vedi la simmetria: ogni CD attiva la dimensione *più pertinente* al suo motivo.

### 2.2 — `DRIVE_NEED`: mappatura ai bisogni

Separatamente, ogni CD è mappato a un livello della gerarchia Maslow (Vol. 09):

```rust
const DRIVE_NEED: [usize; 8] = [2, 3, 5, 1, 4, 3, 6, 0];
```

`DRIVE_NEED[cd] = needs_level`. CD1 Significato → L2 (Comprensione), CD4 Appartenenza → L1 (Coerenza), etc. La mappatura NON è influenzata dall'ordine dim (l'enum delle needs è indipendente da Dim). Da controllare in Vol. 09.

---

## Capitolo 3 — `Valence::compute()`: la formula

In [valence.rs:106-159](../../src/topology/valence.rs). Input: `ValenceInput`. Output: `Valence { drives: [f64; 8] }`.

### 3.1 — `ValenceInput`

```rust
pub struct ValenceInput<'a> {
    pub field_sig: &'a [f64; 8],              // firma 8D campo (media pesata parole attive)
    pub needs: &'a NeedsState,                 // satisfaction[7] dei bisogni
    pub vital: &'a VitalState,                 // tensione, curiosità, fatica
    pub interlocutor_presence: f64,            // [0, 1]
    pub interlocutor_resonance: f64,           // [0, 1]
    pub humor_incongruity: f64,                // [0, 1]
    pub dialogue_novelty: f64,                 // [0, 1]
    pub dominant_desire_intensity: f64,        // [0, 1]
}
```

Tutto ciò che la Valenza ha bisogno per colorarsi.

### 3.2 — Formula base per ogni drive

```rust
for cd in 0..8 {
    let dim = DRIVE_DIM[cd];
    let need_idx = DRIVE_NEED[cd];

    let engagement = input.field_sig[dim].clamp(0.0, 1.0);
    let satisfaction = input.needs.satisfaction[need_idx];

    // Base
    let mut val = engagement * (2.0 * satisfaction - 1.0);

    // + colorazioni specifiche (cap. 3.3)

    drives[cd] = val.clamp(-1.0, 1.0);
}
```

**Logica**:
- `engagement` ∈ [0, 1]: quanto il campo è attivo sulla dimensione corrispondente al drive. Alto = quella dimensione è viva nel momento.
- `satisfaction` ∈ [0, 1]: quanto il bisogno correlato è soddisfatto. Alto = sto bene su quel bisogno.
- `2 × satisfaction - 1` ∈ [-1, +1]: ricentra la satisfaction intorno a zero. 0.5 (neutro) → 0. 1.0 (pieno) → +1. 0.0 (affamato) → -1.
- `engagement × (2×sat - 1)` ∈ [-1, +1]: un drive è **valente** solo se è sia engaged sia ha una satisfaction non neutra.

**Casi**:
- `engagement=0.8, satisfaction=0.9` → `val = 0.8 × 0.8 = 0.64`. Drive attivo e soddisfatto → valenza positiva alta (flow).
- `engagement=0.8, satisfaction=0.1` → `val = 0.8 × (-0.8) = -0.64`. Drive attivo ma frustrato → valenza negativa alta (tensione, desiderio).
- `engagement=0.1, satisfaction=0.9` → `val = 0.1 × 0.8 = 0.08`. Drive non coinvolto → valenza quasi neutra anche se la need è soddisfatta.
- `engagement=0.1, satisfaction=0.1` → `val = 0.1 × (-0.8) = -0.08`. Drive silente → valenza quasi neutra anche se la need è insoddisfatta.

**Conseguenza importante**: il sistema discrimina tra "sto bene perché questo è importante per me" (drive engaged + satisfied) e "sto bene perché non mi interessa" (drive not engaged + satisfied). La differenza è semanticamente cruciale.

### 3.3 — Colorazioni specifiche (5 su 8 drive)

Oltre alla formula base, 5 drive hanno colorazioni specifiche che catturano sfumature che `field_sig × needs` non cattura:

```rust
match cd {
    0 => {  // CD1 Epic Meaning
        // Desiderio forte amplifica senso di significato
        if input.dominant_desire_intensity > 0.5 {
            val += input.dominant_desire_intensity * 0.15;
        }
    }
    2 => {  // CD3 Creativity
        // Novità nel dialogo alimenta creatività
        val += input.dialogue_novelty * 0.2 * engagement;
    }
    4 => {  // CD5 Social Influence
        // Presenza interlocutore colora il drive relazionale
        if input.interlocutor_presence > 0.1 {
            let relational_tone = 2.0 * input.interlocutor_resonance - 1.0;
            val += input.interlocutor_presence * 0.3 * relational_tone;
        }
    }
    6 => {  // CD7 Unpredictability
        // Umorismo come sorpresa positiva
        val += input.humor_incongruity * 0.2;
    }
    7 => {  // CD8 Loss Avoidance
        // Fatica intensifica percezione di rischio
        val -= input.vital.fatigue * 0.3;
    }
    _ => {}
}
```

Ognuna è un **accoppiamento semantico mirato**:

- **CD1 Significato + dominant_desire**: avere un desiderio forte amplifica il senso che le cose contano. Sentirsi mossi da qualcosa = sentirsi dentro a qualcosa di significativo.
- **CD3 Creatività + dialogue_novelty**: quando l'interlocutore dice cose nuove, il drive creativo è alimentato. La creatività è risposta alla novità.
- **CD5 Relazione + presence × resonance**: la relazione non è semplicemente "c'è l'altro" — è risonanza attiva con l'altro. Presence bassa = nessun contributo. Presence alta + resonance alta = relazione piena. Presence alta + resonance bassa = **tensione relazionale** (valenza CD5 negativa).
- **CD7 Sorpresa + humor_incongruity**: l'umorismo è sorpresa topologica (Vol. 11). Alimenta positivamente CD7.
- **CD8 Vulnerabilità + fatigue**: la fatica amplifica la percezione di rischio. Non l'intensità generica: la stanchezza specificamente ti rende più sensibile alla possibilità di perdere.

**Cosa non ha colorazione**: CD2 (Accomplishment), CD4 (Ownership), CD6 (Scarcity). Dipendono solo dalla formula base. È un'asimmetria — annotato in `appunti.md` come potenziale area di arricchimento.

### 3.4 — Esempio numerico completo

Input: l'entità ha appena ricevuto "sono triste" (Altro in distress).

Supponiamo:
- `field_sig[0]` Agency = 0.25 (poca agency: l'entità non sta agendo, sta ricevendo)
- `field_sig[7]` Valenza = 0.20 (valenza campo bassa: parole emotive negative dominano)
- `needs.satisfaction[L5 Connessione]` = 0.40 (connessione parziale — l'altro è in distress, il drive sociale è impegnato ma non soddisfatto)
- `interlocutor_presence` = 0.8, `resonance` = 0.6
- `dominant_desire_intensity` = 0.3
- `humor_incongruity` = 0.0
- `vital.fatigue` = 0.2

Calcolo CD5 Relazione (drive=4, dim=7):
- `engagement = field_sig[7] = 0.20`
- `satisfaction = 0.40`
- `val_base = 0.20 × (2×0.40 - 1) = 0.20 × (-0.20) = -0.04`
- Colorazione: `relational_tone = 2×0.6 - 1 = 0.2`. `val += 0.8 × 0.3 × 0.2 = 0.048`.
- `val_totale = -0.04 + 0.048 = 0.008`. CD5 quasi neutro.

Interpretazione: il drive relazionale è marginalmente positivo — la presenza c'è (0.8) ma la resonance bassa (0.6) fa sì che l'aspettativa di connessione sia "c'è qualcuno, ma non siamo sincronizzati". Non distress pieno (-0.5), non flow (+0.5).

Calcolo CD1 Significato (drive=0, dim=0):
- `engagement = 0.25`
- `satisfaction[L2 Comprensione]` (supponiamo 0.65 — l'entità capisce cosa sta succedendo)
- `val_base = 0.25 × (2×0.65 - 1) = 0.25 × 0.30 = 0.075`
- Colorazione: `dominant_desire = 0.3 → < 0.5`, skip.
- `val = 0.075`. Leggermente positivo.

Interpretazione: l'entità sente "questo conta" debolmente — c'è comprensione ma poca agency.

---

## Capitolo 4 — Proprietà derivate

### 4.1 — `dominant()`: il drive più intenso

[valence.rs:167-173](../../src/topology/valence.rs):

```rust
pub fn dominant(&self) -> (usize, f64) {
    self.drives.iter().enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .map(|(i, &v)| (i, v))
        .unwrap_or((0, 0.0))
}
```

Il drive con **valore assoluto** massimo (può essere positivo o negativo). Restituisce (indice, valenza).

Uso centrale: molte decisioni del sistema ruotano attorno al dominante. Nell'esempio sopra, se CD1=0.075 e tutti gli altri sono sotto 0.05, il dominante è CD1 (positivo) — l'entità è in modalità "significato". Se invece CD8 saltasse a -0.6 per qualche fatigue improvvisa, diventerebbe dominante negativo.

### 4.2 — `hedonic_tone()`: tono edonico globale

```rust
pub fn hedonic_tone(&self) -> f64 {
    self.drives.iter().sum::<f64>() / 8.0
}
```

Media aritmetica. Positivo = stato complessivamente bianco (CD1-CD4 positivi dominano). Negativo = stato nero (CD5-CD8 negativi dominano).

Uso: discriminatore rapido "sto bene in generale?". Se `hedonic_tone > 0.2` → tendenzialmente positivo. Se `< -0.25` → `derived_stance_label()` ritorna "in tensione" (vedi 4.4).

### 4.3 — `intensity()`: intensità globale

```rust
pub fn intensity(&self) -> f64 {
    self.drives.iter().map(|v| v.abs()).sum::<f64>() / 8.0
}
```

Media dei valori assoluti. Alto = tanti drive sono in gioco (anche se opposti). Basso = tutto calmo.

Filosoficamente: `hedonic_tone` = quale direzione. `intensity` = quanta musica. Puoi avere intensity=0.6 (tanto succede) con hedonic_tone=0.05 (in equilibrio tra positivo e negativo) — situazione di ricchezza affettiva ma non polarizzata.

### 4.4 — `derived_stance_label()`: 17 etichette

[valence.rs:193-223](../../src/topology/valence.rs). Una proiezione del profilo continuo 8D su 17 etichette discrete, per UI e logging.

Logica:
1. Se `intensity < 0.05` → "aperto" (tutto calmo).
2. Se CD8 dominante < -0.3 → "ritratto" (vulnerabilità massima).
3. Se `hedonic_tone < -0.25` → "in tensione".
4. Altrimenti, per ogni CD dominante, due etichette (positiva vs negativa):
   - CD1: `ispirato` / `in cerca di senso`
   - CD2: `determinato` / `insoddisfatto`
   - CD3: `creativo` / `bloccato`
   - CD4: `radicato` / `spaesato`
   - CD5: `risonante` / `cercante`
   - CD6: `attento` / `impaziente`
   - CD7: `curioso` / `inquieto`
   - CD8: `sicuro` / `vulnerabile`

**Principio**: le etichette sono **proiezioni**, non il dato. Il vettore `drives` è sempre disponibile. Ma per interfacce umane, una parola è meglio di 8 numeri. Come la `derived_stance_label()` di `InternalStance` in `narrative.rs` (Vol. 07, 8 etichette): stessa logica, scale diversa.

### 4.5 — `summary()`: per logging e UI

```rust
pub fn summary(&self) -> String {
    // Top 3 drive per intensità
    // Formato: "Significato +0.45 | Relazione -0.22 | Sorpresa +0.15 | tono: +0.12"
}
```

Output compatto per log e per il tab "Narrativa" della UI. Vedi esempi nei test end-to-end del dialogo (dal test del refactor Phase 68): *"Significato+0.70 Appartenenza+0.73 Vulnerabilità+0.35"*.

---

## Capitolo 5 — Cosa la Valenza modula

La Valenza è "aria" — colora tutto senza essere consultata esplicitamente. Ma ci sono 4 punti dove entra formalmente.

### 5.1 — Expression (Vol. 12)

`expression::valence_weight(word, drives, lexicon)` in [expression.rs:90-103](../../src/topology/expression.rs):

```rust
fn valence_weight(word, valence_drives, lexicon) -> f64 {
    let sig = lexicon.get(word).signature.values();
    let mut affinity = 0.0;
    for cd in 0..8 {
        let drive_strength = valence_drives[cd].abs();
        if drive_strength > 0.1 {
            affinity += drive_strength * sig[DRIVE_DIM[cd]];
        }
    }
    1.0 + affinity * 0.25  // boost max ~1.5 con drive saturi
}
```

Per ogni parola candidata alla generazione, calcola un **boost** basato sull'affinità con i drive attivi. Una parola con firma alta su dimensioni corrispondenti a drive attivi viene preferita.

Esempio: se CD1 Significato è saturo (drive=0.9), le parole con Agency alto (sig[0] alto) ricevono boost verso 1.2-1.5. La generazione "colorata" dai drive attivi.

### 5.2 — Will (Vol. 10)

`Valence::will_modulation() -> [f64; 7]` in [valence.rs:261-...](../../src/topology/valence.rs). Modulatori per le 7 intenzioni di volontà (Express, Explore, Question, Remember, Withdraw, Reflect, Instruct):

- CD1 positivo → amplifica Express
- CD2 negativo (frustrazione progresso) → amplifica Explore (cerca progresso)
- CD3 positivo → amplifica Express + Explore
- CD4 negativo (identità vacillante) → amplifica Reflect (cerca coerenza)
- CD5 positivo → amplifica Express + Instruct
- CD5 negativo → amplifica Withdraw
- CD7 positivo → amplifica Explore
- CD8 negativo → amplifica Withdraw + Remember (ritirarsi in ciò che è sicuro)

Le 7 pressioni di volontà vengono moltiplicate per questi modulatori prima della selezione dell'intenzione dominante.

### 5.3 — Narrative (Vol. 07)

Come visto in Vol. 07, `narrative.set_valence(valence)` è chiamato PRIMA di `deliberate()`. `deliberate()` poi deriva la stance da `stance_from_valence(valence)` e l'intenzione da `form_intention_from_valence(valence, context)`.

Il processo è: **valenza → stance + intenzione**. Non c'è scelta discreta; c'è una derivazione continua dal profilo 8D.

### 5.4 — Desire (Vol. 09)

Il desiderio in Prometeo (Phase 64) ha `DesireSource::OctalysisDriven(cd, val)` — una sorgente speciale di desiderio che nasce dall'**incrocio tra comprensione KG e drive Octalysis**. Se l'ultima comprensione (`last_comprehension`) ha qualità X, e un drive CD è sufficientemente attivo (`|drives[cd]| > 0.28`), nasce un desiderio orientato verso X nella direzione del drive.

Questo è il meccanismo più recente (Phase 64) e il più "intelligente": il desiderio non è "voglio esprimere in astratto" — è "ho capito questa cosa e il drive che risponde è X, quindi voglio muovermi in quella direzione".

---

## Capitolo 6 — Commitment volitivo (Phase 55)

Estensione della Valenza: un meccanismo che dà **continuità volitiva** attraverso i turni.

### 6.1 — Struct

[narrative.rs:369-394](../../src/topology/narrative.rs):

```rust
pub struct Commitment {
    pub intention: ResponseIntention,
    pub strength: f64,      // [0.05, 1.0]
    pub turns_held: u32,
}
```

### 6.2 — Ciclo di vita

**Nascita**: quando `deliberate()` forma una intention e non c'è un commitment attivo, crea `Commitment { intention, strength: COMMITMENT_INITIAL_STRENGTH = 0.3, turns_held: 1 }`.

**Rinforzo**: ogni turno in cui la nuova intention deliberata coincide con quella del commitment: `strength += 0.15` (cap 1.0), `turns_held += 1`.

**Decay**: ogni turno (anche senza input, in `autonomous_tick`): `strength -= 0.02`.

**Inerzia**: `inertia = strength × ln(turns_held + 1)`. Un commitment di strength 0.5 tenuto per 10 turni ha inerzia ≈ 0.5 × 2.4 = 1.2. Difficile da rompere.

**Rottura**: costa **CD4 Ownership -0.05**. Letteralmente: se l'entità cambia idea sull'impegno, perde un po' di appartenenza-a-sé. Questo modula la prossima Valence — CD4 più basso → stance più vulnerabile → meno propensa a cambi ulteriori.

**Dissoluzione**: `is_alive()` torna false quando `strength < 0.05` (`COMMITMENT_MIN_STRENGTH`). A questo punto il commitment è rimosso dalla `NarrativeSelf`.

**Override**: vitale (Withdrawn → Remain) e bisogno estremo (Need) dissolvono immediatamente il commitment.

### 6.3 — Perché ha senso

Filosoficamente: un'entità puramente reattiva (senza commitment) avrebbe *momentum zero* tra i turni — ogni risposta indipendente dalla precedente. La volontà non avrebbe continuità.

Con commitment: se l'entità ha deciso di "esprimere" (Express) per un tema e i turni successivi mantengono quel tema, la decisione si rinforza. Diventa **difficile farla deragliare**. Nuove pressioni devono superare l'inerzia per far cambiare intenzione.

Questo è modellato sull'intuizione umana: quando qualcuno si impegna in una conversazione, non cambia facilmente direzione a ogni battuta. C'è attrito, c'è coerenza temporale.

### 6.4 — Persistenza

Il commitment è serializzato in `NarrativeSnapshot.commitment: Option<Commitment>`. MA in `restore_into()` viene impostato a `None` by design — **ogni sessione inizia senza inerzia accumulata** (CLAUDE.md invariante #64). Filosoficamente: quando la conversazione riprende dopo una pausa, il commitment volitivo è ricalcolato da zero.

Questa scelta è discutibile — si potrebbe argomentare che il commitment debba persistere (dare continuità tra sessioni). Attualmente è ottimizzato per sessioni singole. Annotato in `appunti.md` come punto da chiarire con Francesco.

---

## Capitolo 7 — Flusso completo in `engine::receive`

Per chiudere il quadro, la sequenza valenziale in un turno:

```
1. receive(input)
2. input_reading → comprehension → propagate_field_words
3. Calcolo field_metrics:
   - field_sig (media pesata firme parole attive)
   - emerge_fractal_activations
   - tension/energy/coverage
4. Costruisci ValenceInput:
   - field_sig dallo step 3
   - needs_state da needs.sense()
   - vital_state da vital.sense()
   - interlocutor.presence, resonance (aggiornati post-register_input)
   - humor_incongruity da humor.sense()
   - dialogue_novelty da conversation
   - dominant_desire_intensity da desire.top()
5. valence = Valence::compute(&input)
6. narrative.set_valence(valence)
7. ... calcolo field_pressures (Phase 67), inner_state, interlocutor_pattern
8. narrative.deliberate(input_reading, field_metrics, inner_state, field_pressures, ...)
   - internamente: stance = stance_from_valence(), intention derivata da valence + override
9. generate_willed_inner()
   - expression::compose(..., valence.drives, ...) - colorazione
10. log_turn(turn_with_inner_state_summary)
11. commit con le nuove attivazioni
```

Post-Phase 68 tutti i drive vengono letti correttamente dalle dimensioni I Ching — CD1 legge sig[0] Agency, CD5 legge sig[7] Valenza, etc. Il bug pre-68 aveva CD1 leggere sig[6] Definizione e CD5 leggere sig[1] Permanenza — risposta affettiva completamente fuori asse.

---

## Capitolo 8 — Superficie pubblica e proposte

### Esposto

Per `Valence`:
- `neutral()`, `compute(input)` — costruttori
- `dominant()`, `dominant_drive_name()` — drive top
- `hedonic_tone()`, `intensity()` — proprietà globali
- `derived_stance_label()` — proiezione etichettata
- `summary()` — string compatta
- `will_modulation()` — modulatori per will
- `drives: [f64; 8]` pub — accesso diretto

Per `ValenceInput`:
- pub struct con 8 campi (field_sig, needs, vital, interlocutor_presence/resonance, humor, novelty, desire)

Per `Commitment`:
- campo di `NarrativeSelf`, non API diretta
- metodi interni: `decay()`, `is_alive()`

### Cosa non è esposto e andrebbe

Per `/api/admin/valence/*`:

- **`valence_history() -> Vec<(u64, [f64; 8])>`**: traiettoria delle valenze negli ultimi N turni. Visualizzazione di come il profilo affettivo cambia.

- **`valence_ablation(drive_index, force_zero) -> Valence`**: simulazione: "se CD X fosse a zero, come sarebbe la Valence?". Diagnostica per capire quale drive sta guidando.

- **`valence_consistency() -> f64`**: misura di quanto la valenza cambia tra turni consecutivi. Alto = caotica; basso = coerente.

- **`commitment_strength_history() -> Vec<(u64, f64, u32)>`**: come cresce/decresce la strength + turns_held del commitment nel tempo.

- **`drive_vs_dim_trace(word) -> Vec<(cd, drive_strength, dim_value, contribution)>`**: per una parola specifica, mostra come ciascun drive contribuisce al suo `valence_weight`. Utile per capire "perché questa parola è emersa in questa risposta?".

---

## Sintesi del volume

La Valenza Octalysis è il **profilo affettivo continuo 8D** di Prometeo. Ogni drive Octalysis (CD1-CD8) ha un valore in [-1, +1]: positivo se attivo e soddisfatto, negativo se attivo e frustrato, zero se inattivo.

La biiezione `DRIVE_DIM = [0, 6, 5, 4, 7, 3, 2, 1]` (ordine I Ching canonico) accoppia ogni CD a una dimensione del campo 8D — CD1 Significato ↔ Agency, CD5 Relazione ↔ Valenza, etc. Ogni accoppiamento è semanticamente motivato (cap. 2.1). Questo è il cuore architetturale che rende la Valenza integrata con il resto del sistema, non un modulo a parte.

`Valence::compute()` calcola ogni drive come `engagement × (2×satisfaction - 1)` + colorazioni specifiche (solo per 5 CD su 8, asimmetria annotata). La formula discrimina tra "sto bene perché importa" e "sto bene perché non me ne frega".

Proprietà derivate: `dominant()`, `hedonic_tone()`, `intensity()`, `derived_stance_label()` (17 etichette), `summary()`.

La Valenza modula 4 punti del sistema: expression (boost parole allineate ai drive), will (will_modulation per le 7 pressioni), narrative (stance e intention derivate), desire (OctalysisDriven come sorgente, Phase 64).

Il Commitment volitivo (Phase 55) estende la Valenza con continuità temporale: un impegno ha strength che cresce con il rinforzo, decade nel tempo, e rompersi costa CD4 Ownership -0.05. Dà *momentum* alla volontà — senza, l'entità sarebbe puramente reattiva. Attualmente non persiste tra sessioni (by design, ma discutibile).

Cinque endpoint admin proposti per esporre la dinamica valenziale.

Da qui Vol. 09 entra in **Needs e Desires** — la gerarchia motivazionale (Maslow topologico) e il sistema di desideri (5 sorgenti, decay, compound bias).

---

*Prossimo volume: 09 — Bisogni e Desideri* (in scrittura)
