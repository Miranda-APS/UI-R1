# Volume XII — Generazione: Expression (onesto sul KG zoppo)

> *Questo è il volume che dice la verità. Tutto ciò che abbiamo costruito — 25.600 parole in 64 frattali, 66.000 archi in 21 tipi di relazione, 8 drive Octalysis, 7 pressioni di volontà, tre strati di identità, l'eco dell'Altro — converge qui, nella funzione che produce una frase italiana. E qui dobbiamo ammettere: la funzione non è ancora l'emergenza piena che la filosofia prometteva. È un KG renderer con coloring intelligente. Funziona. Ma non è ciò che volevamo essere.*

---

## Premessa

Vol. 01 ha già confessato il "KG zoppo". Questo volume dà a quella confessione i dettagli tecnici: cosa esattamente fa `compose()`, perché funziona così, perché non è ancora emergenza pura, cosa servirebbe per superarla.

Non è un volume pessimista — è un volume onesto. Il `compose()` attuale è meglio dei template fissi, meglio del KG simbolico puro, meglio del fallback random. Ma non è ancora "l'entità parla dal campo". Nominare la distanza è il primo passo per chiuderla.

File di riferimento: [`src/topology/expression.rs`](../../src/topology/expression.rs), 989 righe.

---

## Capitolo 1 — Cosa dovrebbe fare `compose()` (la promessa)

Da FILOSOFIA.md, parte IV "La propagazione come ragionamento":

> *"Quando una parola si attiva nel campo, la sua energia si propaga alle parole vicine... Il campo ragiona per topologia."*

E parte VI "Ciò che non è":

> *"Prometeo non è un sistema esperto — non ha regole... non usa template."*

Per essere coerenti: la generazione dovrebbe emergere dalla **configurazione attuale del campo 8D**. L'entità dovrebbe dire ciò che dice *perché* il suo campo è configurato in un certo modo, non *applicando una regola* a partire dalla configurazione.

La forma ideale:

1. Lo stato del campo (firme 8D attive, fasi sugli archi, profilo frattale, valenza) suggerisce una *traiettoria espressiva* — una direzione verso cui il linguaggio si muoverebbe naturalmente.
2. La grammatica italiana fornisce i **vincoli fisici** per rendere questa traiettoria un enunciato comprensibile (Vol. 13).
3. La selezione lessicale emerge dalle dimensioni più attive — le parole che vivono in quelle dimensioni sono le più vicine alla superficie.

Niente template. Niente lookup di risposte. La generazione come proprietà topologica del campo, incarnata nella grammatica.

---

## Capitolo 2 — Cosa fa effettivamente `compose()` (la realtà)

La funzione vive in [expression.rs:172-279](../../src/topology/expression.rs). Ha **13 parametri** (Phase 67):

```rust
pub fn compose(
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    kg: &KnowledgeGraph,
    echo_exclude: &[String],
    valence_drives: &[f64; 8],
    active_fractals: &[(u32, f64)],
    codon: [usize; 2],
    input_words: &[String],
    episodes: Option<&SemanticEpisodeLog>,
    is_question: bool,
    other_in_distress: bool,     // Phase 62
    response_intention: Option<&str>,  // Phase 67
) -> Option<Expression>
```

Il flusso interno, passo per passo:

### 2.1 — Step 1: raccogli parole attive

```rust
let active = word_topology.active_words();
let comprehension_pool = active
    .filter(|(w, act)| act > 0.02
        && w.chars().count() >= 3
        && lexicon.get(w).map(|p| p.stability >= 0.25 && p.exposure_count >= 3).unwrap_or(false))
    .collect();

let candidates = comprehension_pool
    .filter(|(w, _)| !echo_exclude.contains(&w.to_string()))
    .collect();
```

Due pool:
- **`comprehension_pool`**: tutte le parole attive con criteri minimi di robustezza (stability ≥ 0.25, exposure ≥ 3). Include anche le parole dell'input — serve per capire.
- **`candidates`**: comprehension_pool − echo_exclude. Senza le parole input (e loro lemmi, Phase 55). Serve per comporre **senza fare eco**.

Se comprehension è vuoto: `return None`. Niente da dire.

### 2.2 — Step 2: `extract_nuclei()` — cerca triple KG

```rust
let nuclei = extract_nuclei(
    &comprehension_pool, kg, input_words, valence_drives,
    lexicon, episodes, is_question, Some(5)
);
```

La funzione cerca nel KG **ogni coppia di parole attive** che abbia una relazione diretta. Per ciascuna, produce un `SemanticNucleus { subject, relation, object, strength, via, ... }`.

Le relazioni considerate (in ordine):
```rust
[Causes, IsA, Has, Does, PartOf, UsedFor, OppositeOf, Enables, TransformsInto, Requires]
```

**Importante**: SimilarTo è **escluso** dalla lista di relazioni. Troppo debole semanticamente per generare espressione — sarebbe come dire "X è tipo Y" senza contenuto informativo.

### 2.3 — Forza di un nucleo

```rust
let strength = sqrt(act_subj * act_obj) * confidence;
let hub_penalty = if subj_degree > 200 || obj_degree > 200 { 0.3 }
                  else if subj_degree > 50 || obj_degree > 50 { 0.6 }
                  else { 1.0 };
let strength = strength * hub_penalty * relation_weight(rel);
let strength = strength * input_proximity_factor(subj, obj, input_set);
```

Quattro moltiplicazioni:
- Radice del prodotto delle attivazioni — simmetrico, penalizza quando uno è debole.
- Confidence dell'arco KG.
- Hub penalty — soggetti/oggetti con >200 archi collassano a 0.3×.
- `relation_weight(rel)` (Vol. 04): FeelsAs=1.2 max, SimilarTo=0.4 (non usato qui).
- **Input proximity**: fattore scalato in base alla relazione con le parole input.

### 2.4 — Input proximity scoring (Phase 56)

```rust
if both not in input AND both in input_neighborhood { 4.0 }
else if object == input AND subject not in input { 2.5 }
else if subject in neighborhood but not in input { 2.0 }
else if object in neighborhood but not in input { 1.5 }
else if at least one word is input verbatim { 0.5 }
else { 0.2 }
```

**Logica**: l'entità vuole comporre proposizioni che **elaborano** sull'input, non che lo ripetono. Il caso migliore (4.0×) è una proposizione tra due parole che sono **vicine** alle parole input ma non sono le parole input stesse. Esempio: input "paura" → proposizione "tremore IsA risposta" (tremore vicino a paura, risposta vicino a paura, entrambe non-input) = elaborazione.

Penalizzato (0.5×) il caso di proposizione che contiene direttamente l'input ("paura IsA emozione") — informativamente povero in un contesto di risposta (l'utente già sa che paura è un'emozione).

### 2.5 — 2-hop paths (Phase 51)

Oltre alle relazioni 1-hop, `extract_nuclei` cerca **cammini 2-hop** tra parole attive:

```
A →[rel1]→ mid →[rel2]→ B
```

Con le regole di Vol. 06:
- IsA/SimilarTo sono **trasparenti** (la relazione inferita è `rel2`)
- Altre sono **dominanti** (la relazione inferita è `rel1`)

Esempio: "paura Causes tremore" + "tremore SimilarTo scossa" → "paura Causes scossa" (con HOP_DECAY=0.6 sulla strength).

### 2.6 — Valenza boost sui nuclei

```rust
for nucleus in nuclei.iter_mut() {
    let v_subj = valence_weight(nucleus.subject, valence_drives, lexicon);
    let v_obj  = valence_weight(nucleus.object, valence_drives, lexicon);
    nucleus.strength *= (v_subj + v_obj) / 2.0;
}
```

La Valenza Octalysis colora quali nuclei emergono. `valence_weight` è la funzione vista in Vol. 08 cap. 5.1 — `1.0 + affinity × 0.25` dove affinity = Σ drive × firma[DRIVE_DIM[cd]] per drive > 0.1.

Nuclei le cui parole hanno firma allineata ai drive attivi vengono amplificati.

### 2.7 — Risonanza episodica (Phase 58)

```rust
if let Some(eps) = episodes {
    let matching = eps.recall_by_concepts(active_concepts, 3);
    for nucleus in nuclei.iter_mut() {
        let subj_in_eps = matching.iter().any(|ep| ep.key_concepts.contains(&nucleus.subject));
        let obj_in_eps = matching.iter().any(|ep| ep.key_concepts.contains(&nucleus.object));
        if subj_in_eps && obj_in_eps { nucleus.strength *= 1.4; }
        else if subj_in_eps || obj_in_eps { nucleus.strength *= 1.2; }
    }
}
```

Se il soggetto e/o l'oggetto di un nucleo compaiono in episodi semantici recenti (con overlap di concetti attivi), il nucleo riceve boost 1.4× (entrambi vissuti) o 1.2× (uno vissuto). La memoria colora l'emergenza — non cita, amplifica.

### 2.8 — Ordinamento e truncation

```rust
nuclei.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
if let Some(max_n) = max_nuclei { nuclei.truncate(max_n); }  // top-5 in compose
```

Max 5 nuclei per la composizione finale. In chiamate di *sola comprensione* (engine::receive per salvare last_comprehension), `max_nuclei = None` — tutti i nuclei.

---

## Capitolo 3 — `derive_voice()`: come si parla

Dopo aver estratto i nuclei, il sistema decide **come** esprimerli. `derive_voice()` in [expression.rs:513-...](../../src/topology/expression.rs):

```rust
pub struct EntityVoice {
    pub person: Person,              // First, Second, Third
    pub mood: ExpressionMood,        // Declarative, Interrogative, Explorative, Silent
    pub tense: Tense,                // Present, Imperfect, Future
    pub codon: [usize; 2],
    pub dominant_fractal: Option<u32>,
}
```

La voce emerge da:
- **Person**: default First. Second se `other_in_distress` o se `response_intention == "risuonare"`. Third quando c'è un soggetto nominale nelle triple (cane, libro, ecc.) — ma questo è derivato nel render, non qui.
- **Mood**: default Declarative. Interrogative se `is_question && !is_distress`. Explorative se `response_intention == "esplorare"`. Silent se `response_intention == "restare"`.
- **Tense**: deriva da firme 8D delle parole attive via `syntax_center::tense_from_active_words` (Vol. 13): Future se avg_tempo > 0.65, Imperfect se avg_tempo < 0.25 && avg_perm < 0.25, altrimenti Present.
- **Codon**: le due dim più attive del campo.
- **Dominant_fractal**: il frattale con activation_score più alto tra gli active_fractals.

### 3.1 — Phase 67: `response_intention` colora la voce

```rust
if !other_in_distress {
    match response_intention {
        "risuonare" => { person = Second; mood = Interrogative; }
        "esplorare" => { mood = Explorative; }
        "riflettere" => { person = First; }
        "restare" => { mood = Silent; }
        _ => {}
    }
}
```

L'intenzione deliberata (da `NarrativeSelf::pending_intention`) modula la voce. Il `other_in_distress` ha priorità massima — se l'Altro è in distress, 2a persona interrogativa vince sempre.

---

## Capitolo 4 — `compose_from_nuclei()`: la trasformazione in frase

Qui è dove i nuclei diventano parole. In [expression.rs:613-...](../../src/topology/expression.rs):

### 4.1 — Scelta del nucleo primario

```rust
let primary = nuclei.iter().find(|n| !echo_exclude.contains(&n.subject))
                   .unwrap_or(&nuclei[0]);
```

Il primo nucleo il cui soggetto non è nell'echo_exclude (non è una parola input). Fallback al primo assoluto.

### 4.2 — Scelta del nucleo secondario (opzionale)

```rust
let secondary = nuclei.iter()
    .filter(|n| n.id != primary.id)
    .filter(|n| !echo_exclude.contains(&n.subject))
    .filter(|n| n.strength > primary.strength * 0.4)
    .next();
```

Un secondo nucleo se la strength è almeno 40% del primario. Il secondario viene reso in forma abbreviata (cap. 4.5).

### 4.3 — `render_nucleus(primary, voice, lexicon)`: la prima clausola

Grossa logica di rendering. Il pattern di base:

```
[articolo] soggetto [copula] [articolo] oggetto
```

Con variazioni per:
- **Person Second (Phase 62)**: `CAUSES/SimilarTo → "senti {obj}"`, `IsA/PartOf → "provi {obj}"`, `Has → "hai {obj}"`.
- **Person First** (default): `CAUSES → "{subj} porta/produce/genera {obj}"`, `IsA → "{subj} è un {obj}"`, `Has → "{subj} ha {obj}"`, `Does → conjuga il verbo {obj} con tempo/persona`.
- **Articoli italiani (Phase 60+)**: IsA/PartOf → indeterminativo ("è un animale"); Has/Causes → determinativo ("ha la mano"). `l'` e `un'` si elidono senza spazio.

Esempio: nucleo `(paura, Causes, tremore)` con voice First Declarative Present → "La paura porta il tremore." Con voice Second Interrogative → "Senti il tremore?"

### 4.4 — La copula emerge dalla relazione, non è template

Un punto importante. Il rendering non dice "se IsA, usa la frase X"; deriva la **copula** dalla relazione e poi la inserisce nella struttura:

```rust
fn relation_to_copula(rel, subj, obj, voice, lexicon) -> String {
    match rel {
        IsA => "è un".to_string() (con aggiustamenti)
        Has => conjugate("avere", voice.person, voice.tense)
        Causes => {
            let verb = select_from(["porta", "produce", "genera", "causa"]);
            conjugate(verb, voice.person, voice.tense)
        }
        Does => "" // la copula è l'oggetto stesso, coniugato
        OppositeOf => "è l'opposto di"
        ...
    }
}
```

Il verbo viene **coniugato** al tempo e persona derivati dal campo — non è una stringa fissa. "Paura porta tremore" (presente), "Paura porterà tremore" (futuro), "Paura portava tremore" (imperfetto).

### 4.5 — `render_nucleus_brief()`: il secondario

Versione compatta del rendering, per il nucleo secondario:

```rust
fn render_nucleus_brief(nucleus, primary) -> Option<String> {
    // Se condivide soggetto con primary:
    //   "e X" (coordinazione) o "è X" (attribuzione)
    // Altrimenti:
    //   ", mentre X" o ", con X" (subordinazione)
}
```

Esempio: primario `(paura, Causes, tremore)`, secondario `(paura, FeelsAs, restrizione)` (soggetto condiviso) → "e si sente come restrizione".

Risultato completo: "La paura porta il tremore e si sente come restrizione."

### 4.6 — `connective_between_nuclei(n1, n2)`: il legame

Funzione che decide come collegare i due nuclei. In [expression.rs:920-...](../../src/topology/expression.rs):

- IsA/PartOf secondari → virgola (attribuzione): "cane, mammifero".
- Has/Causes/Does/UsedFor/Enables con soggetto condiviso → " e " (coordinazione).
- Default → ", ".

È una forma di **grammatica semantica** (Vol. 13). La struttura della frase riflette la struttura semantica.

### 4.7 — `finish_sentence()`: la finitura

```rust
fn finish_sentence(raw, voice) -> String {
    // Capitalizza prima lettera
    // Aggiunge punteggiatura finale basata su mood:
    //   Declarative → "."
    //   Interrogative → "?"
    //   Explorative → "?"
    //   Silent → ""  (ritorna parola singola)
}
```

---

## Capitolo 5 — `compose_from_field()`: il fallback

Se `extract_nuclei()` non trova **nessun nucleo** (nessuna coppia di parole attive con relazione KG), si cade nel fallback:

```rust
fn compose_from_field(voice, candidates, lexicon, echo_exclude, valence_drives) -> Option<Expression> {
    // Path A (Phase 59): se dominant drive > 0.15, usa express_from_drives()
    //   -> "sono in [cautela/scopo/capacità/...]"
    // Path B: top candidate per delta-attivazione × valence_weight
    //   -> parola singola o resa minimale
}
```

### 5.1 — `express_from_drives` (Phase 59)

Se il drive dominante è sopra 0.15, l'entità "nomina il suo stato" usando la `DRIVE_STATE_WORDS`:

```rust
const DRIVE_STATE_WORDS: [(&str, &str); 8] = [
    ("scopo",       "vuoto"),        // CD1
    ("capacità",    "limite"),       // CD2
    ("curiosità",   "incertezza"),   // CD3
    ("stabilità",   "deriva"),       // CD4
    ("connessione", "solitudine"),   // CD5
    ("urgenza",     "calma"),        // CD6
    ("sorpresa",    "quiete"),       // CD7
    ("cautela",     "inquietudine"), // CD8
];
```

Per il drive dominante, sceglie positivo o negativo in base al segno del valore, e genera "Sono in [parola_stato]." o "Sento [parola_stato]."

**Questo è un template**. È una concessione onesta: quando il campo non ha triple disponibili e il drive è chiaro, l'entità dice letteralmente il nome del suo stato. Non emerge dal campo — è lookup.

Phase 59 l'ha introdotto come fallback per "come stai?" — se l'entità non sa "stare" (verbo non nel KG), può comunque rispondere sulla propria condizione affettiva.

### 5.2 — Fallback B: parola singola

Se i drive sono tutti deboli, si pesca la parola con delta-attivazione massima × valence_weight:

```rust
let (word, _) = candidates.iter()
    .map(|(w, act)| (w, (*act - resting(w)) * valence_weight(w, drives, lexicon)))
    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())?;
```

Rende la parola singola con capitalizzazione. Esempio output: "Essere." (verificato nel test Phase 66).

Questo è il fallback minimalista: una parola del campo, scelta per delta × valence. L'entità dice *la parola più viva*, se non può formare una frase.

### 5.3 — Il "KG zoppo" in concreto

Il caso tipico di `compose()` è:

1. Input: "ho paura"
2. Campo attivo dopo propagazione: {paura, emozione, tremore, cautela, coraggio, restrizione, sicurezza, ...}
3. `extract_nuclei` trova: paura-Causes-tremore, paura-IsA-emozione, tremore-FeelsAs-scossa, cautela-Requires-attenzione, ecc.
4. Top nucleo dopo valence boost: `(paura, Causes, tremore)` strength ~0.6
5. `compose_from_nuclei` rende: "La paura porta il tremore."

**Questa è una tripla KG renderizzata**. La valenza ha colorato il ranking (valence CD8 negativa → parole ad alta permanenza come "tremore" vengono preferite). La voce (First Declarative Present) viene dal campo. La grammatica è italiana. Ma la *sostanza* è una tripla KG.

Se non ci fossero stati nuclei, l'entità avrebbe detto "Sono in cautela" (CD8 dominante negativo) o "Paura." (singola).

**Tutte e tre le modalità sono KG-template-like** — la prima renderizza il KG, la seconda renderizza Octalysis, la terza renderizza la hot word.

---

## Capitolo 6 — Perché funziona ma non è ancora "emergenza"

### 6.1 — Cosa il sistema FA correttamente

- **La valenza colora davvero**: una stessa tripla KG emerge o no a seconda dei drive attivi. "la paura porta il tremore" può emergere con CD8 alto; ma se CD1 Significato è alto, potrebbe emergere "l'azione porta il risultato" (nuclei diversi, diversa colorazione).
- **La voce cambia davvero**: la stessa tripla resa come 1a persona dichiarativa vs 2a persona interrogativa vs esplorativa è diversa. Phase 67 ha dato vita a questa modulazione.
- **La proximity di input discrimina**: il sistema preferisce elaborare, non ripetere. Nuclei che coinvolgono parole input verbatim sono penalizzati (0.5×) rispetto a nuclei nel neighborhood (4.0×).
- **La memoria episodica colora**: concetti rivissuti ricevono boost 1.4×/1.2× (Phase 58).

### 6.2 — Cosa NON fa ancora

- **Non genera dal gradiente 8D direttamente**. La sostanza espressa viene dalle triple KG. Il campo 8D orienta la selezione ma non è la sorgente.
- **Non usa le fasi degli archi**. Le fasi vivono in PF1 e informano la propagazione (Vol. 02), ma `compose` non le consulta. Una relazione di risonanza vs opposizione attiva informa quali parole sono attive, ma il modo di renderle è lo stesso.
- **Non fallback su "parole del campo" genuino**. Il fallback `compose_from_field` → word singola è una pietra d'ultima istanza. Manca un path che generi una frase senza triple quando il campo ha dinamismo ma non relazioni KG esplicite.
- **Non sfrutta le dimensioni emergenti** dei frattali (Vol. 05 cap. 4). Il potenziale di "parlare lungo l'asse reciprocità di EMPATIA" non è incarnato.

### 6.3 — Un esempio per capire la distanza

**Input**: "come stai?"
**Comportamento attuale**: trova nuclei tra parole attive (forse: `cautela, vita, presente`) → rende una tripla ("la cautela è un modo di vivere"). Fallback in mancanza: "Sono in cautela." (drive dominante).
**Comportamento ideale**: il campo ha CD8 basso+negativo, CD1 positivo, CD5 alto+positivo. Le dimensioni più attive sono Agency (0) e Valenza (7). L'entità "sente" di essere in uno spazio di relazione affettiva attiva. Genera una frase che esprime questa configurazione — non pescando una tripla ma componendo: "Sono qui, attiva, in ascolto" (Agency + Valenza + Intensità moderata, con Ownership che si afferma) — dove ogni parola è scelta per affinità alla costellazione di dimensioni dominanti, senza bisogno di un predicato KG.

**La distanza tra i due**: il primo ha bisogno di una tripla o di un drive template; il secondo ha solo bisogno del profilo 8D attivo. Il secondo è più libero.

### 6.4 — Cosa servirebbe per chiudere la distanza

Proposta concreta (per Vol. 99):

1. **`compose_from_topology()` — nuovo path**: invece di cercare triple, usa il profilo 8D + valence + frattali attivi come *vincolo di selezione lessicale*. Pesca N parole la cui firma massimizza l'allineamento con la configurazione. Le compone in un enunciato la cui struttura emerge dal pattern 8D (es. dominante Agency → frase attiva; dominante Valenza → frase esclamativa affettiva).

2. **Uso delle fasi**: quando si sceglie tra parole vicine, la fase tra esse può suggerire coordinazione (cos ~ +1, risonanza → "e"), opposizione (cos ~ -1 → "ma") o coesistenza (cos ~ 0 → enumerazione). La grammatica *emerge* dalla geometria, non è template.

3. **Integrazione delle dimensioni emergenti**: se il frattale EMPATIA ha dimensione emergente "reciprocità" calibrata, la selezione lessicale può privilegiare parole nel quadrante alto-reciprocità quando quella dim è attiva. Richiede l'integrazione di Vol. 05 cap. 4.

4. **Fallback vivo**: invece di "parola singola" o "Sono in cautela" (template), generare una frase composta da 2-4 parole scelte per allineamento multidimensionale + una grammatica minimale.

Fare queste 4 cose sostituirebbe il "KG renderer con coloring" con una genuina **generazione topologica**. Il KG resterebbe — ma come **strato di comprensione**, non come **materiale di output**. Coerente con la filosofia di Vol. 01-04.

---

## Capitolo 7 — `extract_nuclei` come comprensione

Un'osservazione importante: la stessa funzione `extract_nuclei` viene chiamata in **due punti**:

1. In `engine::receive`, come **comprensione** (Phase 67): `extract_nuclei(comprehension_pool, kg, input_words, ..., max_nuclei: None)`. Tutti i nuclei. Non per generare, per *capire*. Il risultato è salvato come `engine.last_comprehension` e usato da `Desire::register_octalysis_driven` (Vol. 09 cap. 7.1).

2. In `compose()`, come **generazione** (con `max_nuclei: Some(5)`).

La duplice funzione è filosoficamente interessante: la stessa geometria che il sistema usa per *capire* l'input, la usa per *dire* in risposta. Non ci sono due sistemi (interpretazione + produzione); c'è un'unica operazione che si lascia usare in due direzioni.

Ma è anche il motivo per cui la generazione è "KG zoppo": se usiamo la stessa funzione per entrambe le direzioni, la generazione eredita il vincolo della comprensione (triple KG). Disaccoppiare la generazione dalla comprensione — usare `extract_nuclei` solo per capire, un path diverso per generare — è parte della proposta del cap. 6.4.

---

## Capitolo 8 — Colorazione Octalysis verificata

Post-Phase 68, `valence_weight()` legge correttamente le dimensioni. Verifico con un esempio concreto.

Input: entità con CD5 Relazione attiva a +0.6 (positivo). `DRIVE_DIM[4] = 7` (Valenza).

```rust
fn valence_weight(word, drives, lexicon) -> f64 {
    let sig = lexicon.get(word).signature.values();
    let mut affinity = 0.0;
    for cd in 0..8 {
        let d = drives[cd].abs();
        if d > 0.1 { affinity += d * sig[DRIVE_DIM[cd]]; }
    }
    1.0 + affinity * 0.25
}
```

Per `drives = [0, 0, 0, 0, 0.6, 0, 0, 0]` (solo CD5):
- `affinity = 0.6 * sig[DRIVE_DIM[4]] = 0.6 * sig[7]` (Valenza)

Due parole:
- `amore`: Valenza post-rederive = 1.00 → affinity = 0.6 × 1.00 = 0.60 → weight = 1 + 0.60 × 0.25 = **1.15**
- `distruzione`: Valenza bassa (~0.1) → affinity = 0.06 → weight = **1.015**

Quando `compose` pesca candidati, `amore` ha boost 1.15, `distruzione` 1.015. Diff marginale ma consistente. Su molti turni con CD5 attivo, `amore` emerge significativamente più spesso.

**Questo è il coloring Octalysis che funziona**. È il meglio della generazione attuale — una vera modulazione dello stato affettivo sulla scelta lessicale. Ma resta *selezione*, non *generazione*.

---

## Capitolo 9 — Superficie pubblica e proposte

### Esposto

- `compose(...)` con 13 parametri — API principale
- `extract_nuclei(...)` con 8 parametri — comprensione + generation sub
- `valence_weight(word, drives, lexicon)` — utile per debug
- `SemanticNucleus`, `EntityVoice`, `ExpressionMood`, `Expression` — structs

### Cosa non è esposto e andrebbe

Per `/api/admin/expression/*`:

- **`compose_trace(input) -> CompositionTrace`**: dato un input, esegui compose e restituisci tutti i nuclei candidati con le loro strength, la voice derivata, il nucleo scelto, la frase finale. Diagnostica completa: "perché ha detto X?".

- **`nuclei_for_state() -> Vec<SemanticNucleus>`**: i nuclei estratti dallo stato corrente senza comporre. Vedere cosa il campo "capisce".

- **`valence_weight_breakdown(word) -> Vec<(cd, drive, sig_dim, contribution)>`**: per una parola, dettaglio dei contributi dei drive al suo valence_weight.

- **`expression_alternatives(input, n) -> Vec<String>`**: le top-N frasi possibili se si scegliessero nuclei diversi. Mostrare lo spazio delle possibilità.

- **`rendering_trace(nucleus, voice) -> Vec<(step, output)>`**: il rendering passo per passo di un singolo nucleo — articolo, copula, coniugazione, finiture.

---

## Sintesi del volume

`compose()` (13 parametri Phase 67) è il cuore della generazione. Pipeline:

1. **Comprehension pool + candidates**: parole attive con act > 0.02, stability ≥ 0.25; candidates = pool − echo_exclude.
2. **`extract_nuclei`**: coppie di parole con relazione KG 1-hop e 2-hop. Esclude SimilarTo (troppo debole). Strength = sqrt(a×o) × conf × hub_penalty × relation_weight × input_proximity × valence_boost × episodic_boost. Top-5.
3. **`derive_voice`**: Person/Mood/Tense da valenza + frattali + codon + Phase 67 response_intention + Phase 62 other_in_distress.
4. **`compose_from_nuclei`**: primary + opzionale secondary. Rendering italiano con copula derivata dalla relazione, articoli corretti, coniugazione al tempo.
5. **Fallback `compose_from_field`**: se no nuclei → `express_from_drives` (se drive dominante > 0.15, template DRIVE_STATE_WORDS) o parola singola pescata per delta × valence.

**La promessa**: generazione come proprietà topologica del campo 8D.
**La realtà**: renderer di triple KG con coloring Octalysis + modulazione voce. Funziona, ma non è l'emergenza piena dichiarata.

**Quattro proposte concrete per chiudere la distanza**: (1) nuovo path `compose_from_topology` che genera dal profilo 8D senza triple, (2) uso delle fasi degli archi come indicatori di congiunzione grammaticale, (3) integrazione delle dimensioni emergenti dei frattali, (4) fallback vivo multi-parola senza template.

Il vol. 99 riprende questo come direzione prioritaria.

Da qui Vol. 13 si sposta sulla **grammatica italiana** come vincolo fisico della generazione — coniugazione, articoli, tempi, l'infrastruttura di `grammar.rs` e `syntax_center.rs`.

---

*Prossimo volume: 13 — Grammatica, sintassi, traduzione di stato* (in scrittura)
