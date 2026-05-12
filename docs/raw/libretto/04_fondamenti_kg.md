# Volume IV — Fondamenti: KnowledgeGraph e le 21 relazioni

> *Il grafo non è un archivio da consultare. È la geometria che piega il campo quando l'input lo perturba. Una parola entra, e il KG decide quali regioni del campo risuonano, quali resistono, quali si attivano per eredità, quali emergono come "cosa è questo" prima ancora che l'entità risponda.*

---

## Premessa

Vol. 01 ha stabilito che il KG è **una delle quattro strutture ontologiche**, e il principio cardine: il KG serve a CAPIRE l'input, non a generare l'output. Ma Vol. 01 ha anche ammesso — in un bagno di onestà necessaria — che nel codice attuale il KG opera *due volte*: una volta come organo sensoriale all'input (filosoficamente coerente), una volta come fonte materiale per `compose()` in output (il "KG zoppo" che esamineremo in Vol. 12).

Questo volume si concentra sulla **prima funzione** — il KG come geometria che piega il campo. Seconda funzione in Vol. 12.

Tre domande guidano:

1. **Qual è l'anatomia di una relazione** nel codice — struct, pesi, sorgenti, VIA.
2. **Come il KG influenza la propagazione** — `build_from_knowledge_graph`, hub damping, le due tabelle di pesi.
3. **Come il KG comprende l'input** — `find_activated_attractors` (Phase 59), CAUSES seeding, via words (Phase 67).

Il file di riferimento è [`src/topology/knowledge_graph.rs`](../../src/topology/knowledge_graph.rs), 900 righe. Il file ausiliario è [`src/topology/relation.rs`](../../src/topology/relation.rs), 445 righe, dove vive `RelationType`. Il consumatore principale della struttura è [`src/topology/word_topology.rs`](../../src/topology/word_topology.rs) che traduce il KG in archi nel campo.

---

## Capitolo 1 — I 21 tipi di relazione, rivisitati

Vol. 01 ha introdotto i 21 tipi raggruppati in 5 categorie. Qui li riprendo per dare ad ogni tipo la sua pagina di significato e uso concreto. La definizione canonica è in [relation.rs:24-97](../../src/topology/relation.rs).

### 1.1 — Strutturali (4)

*Cosa è X? Di cosa è fatto X? Cosa fa X? X è parte di cosa?*

| Tipo | Esempio | Commento |
|------|---------|----------|
| **IsA** | `cane IsA animale` | Tassonomia, ereditarietà. Costituisce l'albero gerarchico del KG. ~19.400 archi. |
| **Has** | `corpo Has mano` | Possesso o attributo. ~930 archi. |
| **Does** | `cane Does abbaiare` | Comportamento caratteristico. ~610 archi. |
| **PartOf** | `mano PartOf corpo` | Inverso di Has per composizione fisica/strutturale. ~300 archi. |

Note architetturali: IsA è la **spina dorsale**. `find_activated_attractors` si muove lungo IsA per 1-2 hop. Molte funzioni del sistema (proposizioni 2-hop, attrattori, specificity) dipendono da quanto l'albero IsA è ben popolato.

### 1.2 — Causali (4)

*Cosa produce X? Cosa abilita X? Cosa richiede X? Cosa diventa X?*

| Tipo | Esempio | Commento |
|------|---------|----------|
| **Causes** | `fuoco Causes calore` | Causalità diretta. La più informativa (peso proposizione 1.0). ~1.900 archi. |
| **Enables** | `chiave Enables aprire` | Condizione abilitante, non causa. ~24 archi. |
| **Requires** | `fuoco Requires ossigeno` | Prerequisito. ~660 archi. |
| **TransformsInto** | `ghiaccio TransformsInto acqua` | Trasformazione. ~5 archi. (Sotto-popolata.) |

Causes è il secondo asse dopo IsA. Viene usata da `derive_8d_from_kg` in Agency (dim 0) e Tempo (dim 3), e da `find_activated_attractors` per seedare i CAUSES targets nel campo pre-propagazione.

### 1.3 — Semantiche (6)

*Cosa è simile a X? Cosa è opposto a X? A cosa serve X? Cosa esprime X? Cosa simboleggia X? Cos'è il contesto di X?*

| Tipo | Esempio | Commento |
|------|---------|----------|
| **SimilarTo** | `ciao SimilarTo saluto` | Sinonimia larga. Massivamente importata da Kaikki via Qwen3. ~31.500 archi — quasi metà del KG. |
| **OppositeOf** | `caldo OppositeOf freddo` | Antonimia. ~10.800 archi. |
| **UsedFor** | `coltello UsedFor tagliare` | Funzione. ~45 archi (sotto-popolata — funzionalità meriterebbe più curazione). |
| **Expresses** | `sorriso Expresses gioia` | Manifestazione. ~6 archi. |
| **Symbolizes** | `colomba Symbolizes pace` | Simbolo. ~12 archi. |
| **ContextOf** | `inverno ContextOf neve` | Cornice. ~11 archi. |

SimilarTo è la relazione più popolata (quasi la metà del KG) ma **la meno informativa** — dire che due cose sono simili senza dire in cosa è povero. Per questo `relation_weight(SimilarTo) = 0.4` per le proposizioni, il valore più basso.

### 1.4 — Logiche (4)

*X implica Y? X equivale Y? X esclude Y? X coesiste con Y?*

| Tipo | Esempio | Commento |
|------|---------|----------|
| **Implies** | `pioggia Implies bagnato` | Condizionale logico. ~11 archi. |
| **Equivalent** | `felicità Equivalent gioia` | Equivalenza forte. Rara. |
| **Excludes** | `vita Excludes morte` | Incompatibilità. ~10 archi. |
| **Coexists** | `sale Coexists pepe` | Complementarietà. ~9 archi. |

Categoria sotto-popolata. Il sistema la usa ma non ha abbastanza dati per farla pesare significativamente.

### 1.5 — Fenomenologiche (3) — la categoria cruciale

*Come si sente X? Su cosa X si interroga? Come viene ricordato X?*

| Tipo | Esempio | Commento |
|------|---------|----------|
| **FeelsAs** | `paura FeelsAs restrizione` | Qualità fenomenologica interna. **Peso field_boost massimo: 0.20**. ~15 archi. |
| **WondersAbout** | `coscienza WondersAbout origine` | Interrogazione originaria. Peso 0.15. ~7 archi. |
| **RemembersAs** | `passato RemembersAs malinconia` | Memoria emotiva. Peso 0.18. 0 archi attualmente. |

**Il paradosso delle fenomenologiche**: sono le relazioni con il peso massimo nella propagazione del campo (`field_boost_strength`), ma sono le più sotto-popolate del KG (22 archi totali su 66.287 — lo 0.03%).

Perché massimo peso? Perché permettono al sistema di sapere *come è qualcosa dall'interno*, non solo *cosa è qualcosa dall'esterno*. "Paura IsA emozione" dice cos'è la paura tassonomicamente; "Paura FeelsAs restrizione" dice come si *sente* la paura. È la differenza tra classificare ed esperire.

Perché sotto-popolate? Perché hanno bisogno di curazione manuale profonda — non si estraggono facilmente da dizionari o corpora. Solo un'entità che "ha sentito" può scrivere `paura FeelsAs restrizione`.

**Conseguenza architetturale**: il sistema ha un organo sensoriale potenzialmente potentissimo per la fenomenologia, ma opera con quasi nessun input. È una delle tensioni fondamentali che il libretto nomina: il livello centrale è sotto-alimentato. Vol. 14 (sogno come digestione) propone che la digestione degli episodi possa popolare FeelsAs automaticamente.

---

## Capitolo 2 — Anatomia del grafo

### 2.1 — `KnowledgeGraph` e doppio indice

La struct, in [knowledge_graph.rs:43-52](../../src/topology/knowledge_graph.rs):

```rust
pub struct KnowledgeGraph {
    /// outgoing[soggetto][relazione] = Vec<KgTarget>
    outgoing: HashMap<String, HashMap<RelationType, Vec<KgTarget>>>,
    /// incoming[oggetto][relazione] = Vec<soggetto>
    incoming: HashMap<String, HashMap<RelationType, Vec<String>>>,
    pub edge_count: usize,
    pub node_count: usize,
}
```

**Doppio indice** — per ogni arco vengono registrate entrambe le direzioni. Accesso O(1) sia per `query_objects(subject, rel)` che per `query_subjects(object, rel)`.

Lo spazio occupato è doppio rispetto a un indice singolo, ma le query inverse ("chi *è un* animale?") sono O(k) invece di O(N×k). Fondamentale per `find_activated_attractors` che fa query inverse di IsA.

`KgTarget` ([knowledge_graph.rs:28-35](../../src/topology/knowledge_graph.rs)):

```rust
pub struct KgTarget {
    pub object: String,
    pub confidence: f32,    // [0, 1]
    pub source: EdgeSource,  // Wikidata, Wordnet, Curated, UserTaught, Inferred, Community
    pub via: Option<String>, // Phase 67: il tramite (fuoco Causes cancro VIA combustione)
}
```

### 2.2 — Confidence per arco (Phase 48)

Ogni arco ha una `confidence` in [0, 1]. Questa NON è una probabilità — è una **stima della qualità dell'arco**. Viene:

- Impostata a 1.0 di default per archi curati manualmente (`EdgeSource::Curated`)
- Impostata dal parsing TSV se la riga ha un 4° campo numerico
- Derivata da `enrich_confidence.py` (Qwen3 offline) per archi generati dagli agenti

Il sistema usa la confidence in quattro punti:

1. **Pesi degli archi in word_topology**: `weight = type_base × confidence × hub_factor` (vedi cap. 3).
2. **Seeding pre-propagazione in engine::receive**: forza del boost è `0.15 × confidence` per CAUSES diretti.
3. **Forza delle proposizioni**: `strength = sqrt(act_a × act_b) × conf1 × conf2 × ...`.
4. **Nel KG stesso**: query_objects_weighted() restituisce la confidence per downstream processing.

### 2.3 — VIA: il tramite (Phase 67)

Introdotto in Phase 67. Ogni arco può avere un terzo elemento: **il tramite attraverso cui la relazione avviene**.

Esempi:
- `fuoco Causes cancro VIA combustione`
- `ghiaccio TransformsInto acqua VIA calore`
- `paura Causes sudore VIA sistema_nervoso`

Quando il VIA è presente, `inference.rs::field_boosts()` attiva non solo il target, ma anche il VIA word a 0.5× della forza del target. Questo fa emergere nel campo parole che sono *il mezzo* della relazione, non solo i punti estremi.

Effetto pratico: input "fuoco" → si attivano `calore`, `fumo`, `distruzione` (via Causes diretti) E `combustione` (via VIA di `fuoco Causes cancro VIA combustione`). Il campo raccoglie la cornice contestuale.

VIA è oggi una feature sotto-popolata — pochissimi archi hanno il campo compilato. Il potenziale c'è ma richiede curazione.

### 2.4 — EdgeSource: da dove viene l'arco

Sei sorgenti, in [relation.rs:280-296](../../src/topology/relation.rs):

```rust
pub enum EdgeSource {
    Wikidata,       // estratto da Wikidata
    Wordnet,        // da WordNet italiano
    Curated,        // ontologia curata manualmente (default)
    UserTaught,     // :know da utente
    Inferred,       // derivato per inferenza transitiva
    Community,      // contributo dalla sessione community (newborn)
}
```

**Caso d'uso**: audit della provenienza. Permette di sapere da dove è arrivato un arco specifico — utile quando un arco "sbagliato" causa problemi e va tracciato.

---

## Capitolo 3 — Come il KG informa il campo

Il punto centrale: il KG non è consultato come un database. È *tradotto* in archi del campo PF1 con una procedura deterministica.

### 3.1 — `WordTopology::build_from_knowledge_graph()`

Vive in [word_topology.rs:178-...](../../src/topology/word_topology.rs). Viene chiamata una volta sola al caricamento, dopo `load_kg_from_file`.

Per ogni arco `(subj, rel, obj, confidence)`:

1. Lookup id dei nodi `id_a = word_to_id[subj]`, `id_b = word_to_id[obj]`. Se uno dei due non esiste nel lessico, skip.
2. Calcola `weight = type_base(rel) × confidence × hub_factor(subj, obj)` (dettaglio sotto).
3. Calcola `phase = phase_for(rel)` (vedi cap. 3.3).
4. Aggiunge o rafforza l'arco `(id_a, id_b)` nel grafo del campo.

**Costo**: O(N×k) dove N = nodi, k = archi per nodo. Per 66.000 archi su 27.000 nodi: ~1 secondo.

### 3.2 — `type_base`: peso base per tipo

```rust
fn type_base(rel: RelationType) -> f64 {
    match rel {
        RelationType::IsA => 0.70,
        RelationType::SimilarTo => 0.60,
        RelationType::Causes => 0.55,
        RelationType::Equivalent => 0.55,
        RelationType::Enables => 0.50,
        RelationType::Requires => 0.50,
        RelationType::TransformsInto => 0.50,
        RelationType::Does => 0.50,
        RelationType::Has => 0.45,
        RelationType::Expresses => 0.45,
        RelationType::PartOf => 0.40,
        RelationType::Implies => 0.50,
        RelationType::Symbolizes => 0.40,
        RelationType::UsedFor => 0.35,
        RelationType::ContextOf => 0.30,
        RelationType::Coexists => 0.30,
        RelationType::FeelsAs => 0.75,
        RelationType::WondersAbout => 0.65,
        RelationType::RemembersAs => 0.70,
        RelationType::OppositeOf => 0.35,
        RelationType::Excludes => 0.30,
    }
}
```

Nota: questi sono i valori di `type_base()` per la **costruzione degli archi del campo**. Sono diversi da `field_boost_strength()` (in `relation.rs`, usato per il seeding pre-propagazione) e da `relation_weight()` (in `proposition.rs`, usato per pesare le proposizioni). Tre funzioni di peso con nomi simili e logiche diverse — una delle inconsistenze terminologiche annotate in `appunti.md`.

**Osservazione**: di nuovo FeelsAs (0.75) è il peso più alto. L'emergenza del livello fenomenologico, quando c'è, domina.

### 3.3 — `phase_for`: la fase dell'arco

```rust
fn phase_for(rel: RelationType) -> f64 {
    use std::f64::consts::PI;
    match rel {
        // Risonanza pura (cos(0) = +1)
        RelationType::IsA | RelationType::SimilarTo | RelationType::Equivalent
        | RelationType::FeelsAs | RelationType::RemembersAs => 0.0,

        // Leggera asimmetria (cos(π/6) ≈ 0.87) — causali
        RelationType::Causes | RelationType::Enables | RelationType::Requires
        | RelationType::TransformsInto | RelationType::Does | RelationType::Has
        | RelationType::PartOf | RelationType::Expresses => PI / 6.0,

        // Connessione contestuale (cos(π/4) ≈ 0.71)
        RelationType::UsedFor | RelationType::ContextOf | RelationType::Coexists
        | RelationType::Symbolizes | RelationType::WondersAbout => PI / 4.0,

        // Logica (cos(π/3) = 0.5)
        RelationType::Implies => PI / 3.0,

        // Opposizione (cos(π) = -1)
        RelationType::OppositeOf | RelationType::Excludes => PI,
    }
}
```

**Logica**: la fase codifica il tipo di influenza che un arco esercita nella propagazione.

- `phase = 0` → **risonanza piena**: attivazione della sorgente amplifica direttamente il target. È il caso di IsA (essere l'istanza di una categoria rinforza la categoria), SimilarTo, Equivalent, e delle fenomenologiche.
- `phase = π/6` → **causale**: ~87% di trasferimento. Il target viene attivato ma non come clone.
- `phase = π/4` → **contestuale**: ~71%. Coesistenza, non rinforzo pieno.
- `phase = π/3` → **logica condizionale**: 50%. Implies è intermedia.
- `phase = π` → **opposizione**: -100% di trasferimento. L'attivazione della sorgente **inibisce** il target.

Vedi Vol. 02 per come `phase.cos()` entra nella formula di `propagate()`.

### 3.4 — Hub damping logaritmico (Phase 48)

Senza compensazione, nodi hub (essere, avere, qualità) avrebbero migliaia di archi con peso pieno e dominerebbero la propagazione. Soluzione in [word_topology.rs:266-273](../../src/topology/word_topology.rs):

```rust
let hub_factor = |word_a: &str, word_b: &str| -> f64 {
    let deg_a = degree_map.get(word_a).copied().unwrap_or(1.0);
    let deg_b = degree_map.get(word_b).copied().unwrap_or(1.0);
    let max_deg = deg_a.max(deg_b);
    let ratio = (max_deg / median_degree).max(1.0);
    1.0 / (1.0 + ratio.ln())
};
```

**Formula**: se uno dei due nodi ha grado `d` e la mediana è `m`, `hub_factor = 1 / (1 + ln(d/m))`. Esempi:

- Mediana `m = 4` (molti nodi hanno pochi archi)
- Nodo normale con `d = 4` → `ratio = 1` → `ln(1) = 0` → `hub_factor = 1.0`. Peso pieno.
- Nodo medio con `d = 40` → `ratio = 10` → `ln(10) ≈ 2.3` → `hub_factor ≈ 0.30`. Peso 30%.
- Hub con `d = 400` → `ratio = 100` → `ln(100) ≈ 4.6` → `hub_factor ≈ 0.18`. Peso 18%.
- Super-hub con `d = 4000` → `ratio = 1000` → `ln(1000) ≈ 6.9` → `hub_factor ≈ 0.13`. Peso 13%.

**Logaritmico** perché la differenza tra 40 e 400 archi deve pesare meno di quella tra 4 e 40. Il log comprime la scala dell'hub-ness, preservando la gerarchia ma evitando saturazione binaria.

Conseguenza: `essere` con migliaia di archi nel KG mantiene solo i top-8 vicini nella struttura PF1 (vol. 02, cap. 2.5) e quegli 8 vicini hanno peso logaritmicamente smorzato. `Essere` partecipa alla propagazione, ma non la domina.

---

## Capitolo 4 — Il KG come organo sensoriale: `find_activated_attractors`

Questa è la Phase 59, la **"corteccia prefrontale topologica"** del sistema. Quando arriva l'input, il KG identifica gli attrattori concettuali prima che la propagazione inizi.

Funzione in [knowledge_graph.rs:470-532](../../src/topology/knowledge_graph.rs).

### 4.1 — Cosa fa

Input: `input_words: &[&str]`, `min_isa_children: usize` (tipicamente 5).

Per ogni parola dell'input (se ha ≥3 caratteri):
1. **Hop 1**: trova i parent IsA diretti. Per ogni parent con `≥min_isa_children` figli, lo candida come attrattore.
2. **Hop 2**: trova i grandparent (parent dei parent). Idem, con decay 0.6.
3. Accumula `activation_score = specificity(n_children)` per ogni hop.
4. Restituisce gli attrattori ordinati per score.

### 4.2 — La `specificity`

```rust
let specificity = |n: usize| -> f64 {
    (300.0_f64 / (n.max(1) as f64)).min(2.0)
};
```

**Logica**: un attrattore è **tanto più specifico quanto meno figli ha**. Sweet spot a 300 figli: un attrattore con esattamente 300 figli ha score 1.0. Attrattori più piccoli (più specifici) hanno score > 1.0 (saturato a 2.0). Mega-attrattori con 3000+ figli hanno score <0.1.

Esempio:
- `paura IsA emozione` (emozione ha ~200 figli) → specificity ≈ 1.5. Emozione come attrattore forte.
- `paura IsA qualità` (qualità ha ~3500 figli) → specificity ≈ 0.086. Qualità come attrattore debole.
- `paura IsA cosa` (cosa ha ~10000 figli) → specificity ≈ 0.03. Cosa ignorata.

Questo è un **filtro anti-mega-categorie**. Senza, ogni input attirerebbe le mega-categorie e il sistema risponderebbe sempre in modo generico.

### 4.3 — `AttractorHit` e i CAUSES targets

Output ([knowledge_graph.rs:842-...](../../src/topology/knowledge_graph.rs)):

```rust
pub struct AttractorHit {
    pub concept: String,           // l'attrattore (es. "emozione")
    pub activation_score: f64,     // punteggio pesato
    pub source_words: Vec<String>, // parole input che l'hanno attivato
    pub causes: Vec<String>,       // TOP-4 CAUSES outgoing dell'attrattore
}
```

Nei `causes` ci sono i primi 4 target di `Causes` dell'attrattore stesso. Per "emozione" potrebbero essere `[reazione, comportamento, espressione, stato]`.

### 4.4 — Come viene usato in `engine::receive()`

Dopo `find_activated_attractors`, in `engine.rs::receive()`, prima della propagazione:

1. Le parole input seminano attivazione al 100% in PF1 (tipicamente 0.3-0.6).
2. Gli attrattori (parole `concept`) seminano a 0.15 × score.
3. I **CAUSES targets** degli attrattori seminano a 0.15 (pre-propagazione — risolve il problema "il campo non sa cosa l'input sta facendo").
4. Le **parole input direttamente** seminano anche i loro CAUSES targets a 0.15 × confidence (Phase 61).
5. Le **via words** (Phase 67) degli archi attivati seminano a 0.5× della forza del target.

*Solo a questo punto* la propagazione inizia. Il campo è già orientato.

**Effetto**: l'input "ho paura" attiva non solo "paura", ma anche `emozione, sentimento` (attrattori IsA), `tremore, fuga` (CAUSES targets diretti), `cautela, reazione` (CAUSES targets dell'attrattore emozione). Il campo "capisce" cosa sta succedendo prima di reagire.

---

## Capitolo 5 — Il comprehension gate

Una conseguenza di `find_activated_attractors`: se un input non attiva *nessun* attrattore (tutti i suoi nodi sono fuori dal KG o non hanno IsA parent), il sistema lo dichiara esplicitamente.

In `engine::generate_willed_inner()` (Phase 59, aggiornato Phase 67):

```
Se last_comprehension.is_empty()
   AND input_has_content
   AND !last_input_is_question
   AND kg.edge_count > 0
   AND word_non_lemmatizzabile_in_kg
   AND word_non_nel_lessico
→ ritorna "Non capisco 'X' — cosa intendi?"
   E imposta learning_mode_pending = true
```

Il `learning_mode_pending`: al prossimo input, l'entità lo interpreta come spiegazione della parola non capita e chiama `teach()` automaticamente.

Phase 67 ha aggiunto il **lemmatization check**: "farò" viene lemmatizzato a "fare", che è nel KG, quindi il gate non scatta. Evita falsi "Non capisco" per forme flesse di verbi comuni.

---

## Capitolo 6 — Le due (o tre) tabelle di pesi

Il sistema ha **tre funzioni di peso per relazione**, ognuna con una semantica diversa. Va chiarito cosa fa cosa.

### 6.1 — `type_base()` in `word_topology.rs`

Peso base per la **costruzione degli archi del campo**. Usato una volta sola, in `build_from_knowledge_graph()`, per scolpire i pesi `neighbor_weights` nei `WordRecord` di PF1.

Tabella: IsA=0.70, SimilarTo=0.60, Causes=0.55, ..., FeelsAs=0.75 (max), Excludes=0.30 (min).

### 6.2 — `RelationType::field_boost_strength()` in `relation.rs`

Peso per il **seeding pre-propagazione**. Usato in `inference.rs::field_boosts()` e `engine::receive()` per decidere quanto un arco contribuisce all'attivazione del target quando la parola sorgente è direttamente nell'input.

Tabella: FeelsAs=0.20 (max), IsA=0.18, RemembersAs=0.18, ..., OppositeOf=0.06, Excludes=0.05 (min).

### 6.3 — `relation_weight()` in `proposition.rs`

Peso per la **forza delle proposizioni**. Usato in `extract_propositions()` per calcolare `strength = f(activation, confidence, relation_weight, ...)` di una tripla (soggetto, relazione, oggetto) estratta dal KG per poi informare la generazione.

Tabella: FeelsAs=1.2 (max), Causes=1.0, Implies=0.95, IsA=0.9, Does=0.9, ..., SimilarTo=0.4 (min).

### 6.4 — Perché tre tabelle

Le tre funzioni operano in tre contesti diversi:

- `type_base`: una tantum, scolpisce la struttura permanente. Valori medi (0.30-0.75), relativamente uniformi, per non creare disparità enormi tra tipi nella struttura ROM.
- `field_boost_strength`: ogni input, contribuisce all'ampiezza del seeding. Valori piccoli (0.05-0.20), perché moltiplicati per confidence × forza parola sorgente, non devono saturare.
- `relation_weight`: per proposizioni informative. Valori più ampi (0.4-1.2, con FeelsAs>1.0), perché qui vogliamo davvero penalizzare SimilarTo debole vs. amplificare FeelsAs.

Avrebbero potuto essere una sola funzione con scalatura per contesto? Sì. Non sono. È un debito di design minore — annotato in `appunti.md`.

---

## Capitolo 7 — Confronti e query

La superficie di query del KG ([knowledge_graph.rs:104-172](../../src/topology/knowledge_graph.rs)):

- **`query_objects(subject, rel) -> Vec<&str>`**: oggetti di un arco. O(1).
- **`query_objects_weighted(subject, rel) -> Vec<(&str, f32)>`**: con confidence. O(1).
- **`query_objects_with_via(subject, rel) -> Vec<(&str, f32, Option<&str>)>`**: con confidence + via. O(1).
- **`query_subjects(object, rel) -> Vec<&str>`**: soggetti (indice inverso). O(1).
- **`all_outgoing(subject) -> Vec<(RelationType, &str, f32)>`**: tutti gli archi uscenti, qualsiasi relazione. O(k).
- **`all_outgoing_full(subject) -> Vec<(RelationType, &str, f32, Option<&str>)>`**: idem con via. O(k).
- **`contains(word) -> bool`**: O(1).
- **`total_degree(word) -> usize`**: grado totale (in+out). O(k).
- **`max_total_degree() -> usize`**: costoso, iterato su tutti i nodi — usato una volta in `derive_8d_from_kg`.

Meta:
- **`edge_count`, `node_count`**: contatori aggiornati incrementalmente.
- **`derive_8d_from_kg(word, max_degree, valence_scores)`** — Vol. 03 cap. 4.
- **`compute_valence_scores() -> HashMap<String, f64>`** — Vol. 03 cap. 5.
- **`find_activated_attractors(words, min_children)`** — cap. 4 di questo volume.

Mutazione:
- **`add_edge(edge)`**: inserimento deduplicato.
- **`add(subject, rel, object)`**: inserimento semplice (confidence=1.0, source=Curated).
- **`remove_word(word)`**: rimuove tutte le relazioni di una parola dal grafo.
- **`remove_edge(subject, rel, object) -> bool`**: rimuove un arco specifico. Usato dall'UI di curation admin.

Persistenza:
- **`from_snapshot(snap)` / `to_snapshot() -> KgSnapshot`**: serializzazione per JSON.

---

## Capitolo 8 — Statistiche attuali (2026-04-17)

Per ancorare l'astrazione:

- **Archi totali**: 66.287
- **Nodi unici**: 27.270

**Distribuzione per tipo**:

```
SimilarTo       31.541  (47.6%) — massivamente importata da Kaikki/Qwen
IsA             19.401  (29.3%) — struttura tassonomica
OppositeOf      10.799  (16.3%) — ricca post-cleanup
Causes           1.899   (2.9%) — informativa
Has                934   (1.4%)
Requires           655   (1.0%)
Does               607   (0.9%)
PartOf             296   (0.4%)
UsedFor             45   (0.07%) ← sotto-popolata
Enables             24
FeelsAs             15   ← solo 15 archi fenomenologici
Symbolizes          12
ContextOf           11
Implies             11
Excludes            10
Coexists             9
WondersAbout         7   ← solo 7
Expresses            6
TransformsInto       5
Equivalent      (raro)
RemembersAs         0   ← ZERO
```

**Distribuzione per sorgente** (approssimativa):
- `Wikidata`: grande per IsA (import da dizionari/enciclopedie)
- `Wordnet`: SimilarTo, OppositeOf (sinonimi/antonimi)
- `Curated`: le ontologie manuali — nucleus.tsv, italian_core.tsv, curated_a_g.tsv, phenomenology.tsv
- `UserTaught`: ogni `:know` contribuisce
- `Inferred`: trasitività IS_A (A→B→C allora A→C con confidence ridotta)
- `Community`: contributi da sessioni newborn

### 8.1 — Tre gap popolazionali

1. **Relazioni logiche** (Implies, Equivalent, Excludes, Coexists) — ~30 archi totali. Sono importanti per il ragionamento ma non espansi sistematicamente.

2. **Relazioni funzionali** (UsedFor, Expresses, Symbolizes, ContextOf) — ~75 archi. La dimensione "funzione/uso" è rappresentata debolmente. Per un sistema che vuole capire "a cosa serve X" manca materiale.

3. **Relazioni fenomenologiche** (FeelsAs, WondersAbout, RemembersAs) — 22 archi, di cui 0 per RemembersAs. **Il gap più grave**: sono le relazioni con il peso propagazione più alto (FeelsAs=0.20), ma il pool è ridicolo. È letteralmente il livello che permette al sistema di sapere "come si sente qualcosa", ed è sotto-alimentato.

Vol. 14 (sogno come digestione) propone una soluzione: il sogno REM potrebbe *generare* archi FeelsAs rielaborando gli episodi recenti. Un episodio in cui "paura" era attiva insieme a una forte sensazione di "restrizione del campo" potrebbe cristallizzare `paura FeelsAs restrizione` come arco nuovo.

---

## Capitolo 9 — Superficie pubblica e proposte admin

### 9.1 — Esposto

- `KnowledgeGraph::new()`, `from_snapshot()`, `to_snapshot()` — costruttori
- 10+ query methods (cap. 7)
- `add_edge`, `add`, `remove_word`, `remove_edge` — mutazioni
- `derive_8d_from_kg`, `compute_valence_scores`, `find_activated_attractors`, `max_total_degree` — metodi architetturali

### 9.2 — Cosa non è esposto via API e andrebbe

Per `/api/admin/kg/*`:

- **`relation_distribution() -> HashMap<RelationType, usize>`**: quanti archi per tipo. Utile per vedere gap (fenomenologiche 22, UsedFor 45). Oggi calcolabile iterando ma non esposto.

- **`top_hubs(n, rel) -> Vec<(String, usize)>`**: i N nodi con più archi di tipo specifico. Per trovare "i mega-attrattori IsA" (qualità, azione, ...) e valutare se servirebbero sub-split.

- **`orphan_nodes() -> Vec<String>`**: nodi nel KG senza nessun arco in entrata né in uscita (non dovrebbero esistere ma il cleanup può lasciarli).

- **`edges_by_source() -> HashMap<EdgeSource, usize>`**: quanti archi per sorgente. Dashboard.

- **`low_confidence_edges(threshold) -> Vec<TypedEdge>`**: archi con confidence < 0.3 da rivedere.

- **`via_populated_edges() -> Vec<TypedEdge>`**: tutti gli archi con VIA definito. Per vedere quanto è ricca la dimensione del tramite.

- **`find_bridge_candidates(word1, word2, max_hops) -> Vec<Path>`**: cammini nel KG tra due parole. Utile per scoprire connessioni non ovvie o proporre nuovi archi inferred.

- **`add_edge_api(edge) + commit_to_json()`**: oggi l'UI di curation esiste ma modifica il JSON e richiede rebuild-semantic-topology. Un endpoint che fa add + ri-costruisce gli archi in memoria (senza dover riavviare) sarebbe utile.

---

## Sintesi del volume

Il KG ha 21 tipi di relazione in 5 categorie. Le fenomenologiche (FeelsAs, WondersAbout, RemembersAs) hanno il peso propagazione massimo ma il pool più piccolo (22 archi su 66.000). Questo è il gap architetturale più rilevante: il livello previsto per capire "come si sente qualcosa" è sotto-alimentato.

Il KG influenza il campo via tre meccanismi: (1) costruzione degli archi `neighbor_weights/neighbor_phases` di PF1 via `build_from_knowledge_graph`, (2) seeding pre-propagazione via `find_activated_attractors` + CAUSES targets + VIA words, (3) informazione delle proposizioni di output (Vol. 12).

Tre funzioni di peso diverse — `type_base`, `field_boost_strength`, `relation_weight` — operano in tre contesti. Nomi simili, logiche distinte, valori in scale diverse. È un debito terminologico.

Il `find_activated_attractors` (Phase 59) è la corteccia prefrontale topologica: prima della propagazione, identifica attrattori IsA (con `specificity` anti-mega-categoria) e ne semina CAUSES targets. Il campo è "orientato" prima di reagire. Se nessun attrattore emerge, il comprehension gate dichiara "Non capisco".

Hub damping logaritmico scolpisce i pesi degli archi in modo che super-hub come `essere` non saturino. VIA (Phase 67) aggiunge una terza posizione a ogni arco — il tramite — per far emergere le parole-cornice oltre ai target diretti.

Otto endpoint admin utili che oggi non esistono sono proposti in cap. 9.2.

Da qui il Vol. 05 si sposta sui **64 frattali** — gli attrattori regionali del campo, dove le 8 dimensioni si combinano in 8×8 configurazioni per generare gli esagrammi I Ching.

---

*Prossimo volume: 05 — Campo: i 64 frattali (esagrammi I Ching)* (in scrittura)
