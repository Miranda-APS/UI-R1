# Volume VI — Campo: inferenza e proposizioni

> *Il campo non contiene pensieri — li produce. Quando due parole attive hanno una relazione nel KG, emerge una proposizione. Quando una parola attiva trascina i suoi vicini semantici via IsA, emerge un'eredità. Quando lo stato complessivo del campo chiama una spiegazione, emerge un'abduzione. Inferenza in Prometeo non è un motore simbolico: è una proprietà topologica di ciò che è attivo.*

---

## Premessa

Vol. 04 ha mostrato come il KG informa il campo al momento dell'input (`find_activated_attractors` + CAUSES seeding). Ma il sistema fa altre forme di inferenza *continuamente*:

- **`field_boosts`**: l'eredità semantica — quando una parola è attiva, i suoi "parenti" IsA/SimilarTo vengono leggermente attivati anche loro.
- **`extract_propositions`**: dal campo attivo emergono triple (soggetto, relazione, oggetto) — "pensieri" del sistema che poi alimentano la generazione.
- **`abduce`**: "quale attrattore spiegherebbe lo stato attuale?" — ragionamento abduttivo sui frattali.
- **`find_contradictions`**: parole con carica opposta simultaneamente attive.

Tre file implementano questo livello:
- [`src/topology/inference.rs`](../../src/topology/inference.rs) — 453 righe. Eredità IsA e field boosts.
- [`src/topology/proposition.rs`](../../src/topology/proposition.rs) — 664 righe. Proposizioni 1-hop e 2-hop.
- [`src/topology/reasoning.rs`](../../src/topology/reasoning.rs) — 433 righe. Implicazioni, abduzione, contraddizioni.

Questo volume li tratta in quest'ordine: dal più semplice (eredità) al più complesso (abduzione).

---

## Capitolo 1 — `InferenceEngine::type_chain` — la gerarchia ereditaria

Il meccanismo più semplice. Data una parola, risale l'albero IsA per N hop, raccogliendo tutti i "genitori" ancestrali.

In [inference.rs:48-...](../../src/topology/inference.rs):

```rust
pub fn type_chain(&self, word: &str) -> Vec<String> {
    // parte dalla parola
    // per ogni hop fino a max_hops:
    //     query_objects(current, IsA)
    //     aggiunge i nuovi genitori alla catena
    //     prosegue dai nuovi genitori
    // restituisce la catena unica
}
```

**Esempio**: `type_chain("cane")`:
1. Hop 0: `cane`
2. Hop 1: `cane IsA animale` → `animale`
3. Hop 2: `animale IsA essere_vivente` → `essere_vivente`
4. Hop 3: `essere_vivente IsA entità` → `entità`
5. (hops successivi: entità IsA cosa, cosa IsA qualità — mega-categorie)

**Uso nel sistema**:
- `read_input()` in `input_reading.rs` usa la type chain per classificare input (es. "ho paura" — "paura" type_chain include "emozione" → input classificato come Express/Express emozione).
- Il comprehension gate (Vol. 04) verifica se un lemma è nel KG tramite lookup semplice, ma `type_chain` potrebbe estendere il controllo "lemma's parent è nel KG".

**Limite**: la catena tende a salire verso mega-categorie. Lo stesso `find_activated_attractors` con `specificity` filter è una versione più raffinata — preferisce parenti con "pochi figli" (specifici) alle mega-categorie.

---

## Capitolo 2 — `InferenceEngine::field_boosts` — l'eredità come attivazione

Quando una parola è direttamente nell'input, il sistema non solo attiva lei, ma anche le parole ad essa legate nel KG. Questo è il meccanismo di **field boost**.

In [inference.rs:192-...](../../src/topology/inference.rs):

```rust
pub fn field_boosts(&self, word: &str) -> Vec<(String, f32)> {
    let mut boosts = Vec::new();

    for rel in RelationType::ALL {
        if word negated in this call { continue; }  // Phase 61
        let strength = rel.field_boost_strength();
        for (target, confidence, via) in kg.query_objects_with_via(word, rel) {
            boosts.push((target.to_string(), strength × confidence));
            if let Some(v) = via {
                boosts.push((v.to_string(), strength × confidence × 0.5));  // Phase 67
            }
        }
    }
    boosts
}
```

**Per ogni relazione** che la parola ha, aggiunge al risultato una coppia `(target, peso)`. Il peso è `field_boost_strength(rel) × confidence`. Se l'arco ha un VIA (Phase 67), anche la via word entra con peso 0.5× del target.

### 2.1 — Esempio concreto: input "paura"

Supponiamo `paura` abbia nel KG:
- `paura IsA emozione` (conf 0.95) → boost emozione: `0.18 × 0.95 = 0.171`
- `paura IsA sentimento` (conf 0.85) → boost sentimento: `0.18 × 0.85 = 0.153`
- `paura Causes tremore VIA sistema_nervoso` (conf 0.80) → boost tremore `0.12 × 0.80 = 0.096`, boost sistema_nervoso `0.048` (metà)
- `paura Causes fuga` (conf 0.75) → boost fuga `0.090`
- `paura OppositeOf coraggio` (conf 0.90) → boost coraggio `0.06 × 0.90 = 0.054`
- `paura FeelsAs restrizione` (conf 1.0) → boost restrizione `0.20 × 1.0 = 0.200` ← massimo!
- `paura SimilarTo timore` (conf 0.80) → boost timore `0.16 × 0.80 = 0.128`

Risultato totale: 7 parole attivate insieme a "paura" con pesi da 0.054 a 0.200.

Il sistema applica questi boost al campo PF1: ogni target riceve `activation += peso` (clampato). Così quando la propagazione parte, il campo non ha solo "paura" attiva — ha **tutta la famiglia semantica di paura**.

### 2.2 — Negation-aware (Phase 61)

La funzione accetta un elenco di parole "negated" — quelle operanti sotto un operatore di negazione nell'input ("non paura", "senza paura"). Per queste parole, `field_boosts` **salta** completamente l'eredità — non vuoi che "non paura" attivi "tremore" e "fuga".

Invece, `engine::receive` gestisce separatamente le parole negate attivando i loro **OppositeOf** al 35% della confidenza. "non paura" attiva "coraggio" e "sicurezza" — la direzione opposta della famiglia.

### 2.3 — VIA words seeding (Phase 67)

Il Phase 67 ha aggiunto il seeding delle VIA words. Per `ghiaccio TransformsInto acqua VIA calore`: input "ghiaccio" → boost "acqua" (target) a forza piena + boost "calore" (via) a forza 0.5×. Il campo raccoglie il *mezzo* della trasformazione, non solo gli estremi.

**Perché**: la generazione può poi scegliere di esprimere la via word se è semanticamente rilevante, producendo frasi tipo "Il ghiaccio diventa acqua attraverso il calore" invece di "Il ghiaccio diventa acqua".

---

## Capitolo 3 — Proposizioni 1-hop e 2-hop

`extract_propositions` è il meccanismo centrale che genera le triple alimentando la generazione. Vive in [proposition.rs:305-...](../../src/topology/proposition.rs).

### 3.1 — Proposizione: che cos'è

```rust
pub struct Proposition {
    pub subject: String,
    pub relation: PropRelation,
    pub object: String,
    pub strength: f64,        // [0, 1]
    pub kg_confidence: f32,
    pub hops: u8,             // 1 = diretta, 2 = inferita
    pub via: Option<String>,  // intermediario per 2-hop
}
```

`PropRelation` è un enum separato da `RelationType` (entrambi esistono — un debito terminologico). Ha variant in più: `FieldProximity` (due parole attive *senza* arco KG ma vicine topologicamente).

### 3.2 — `extract_propositions`: il processo

Input: lista delle parole attive nel campo con le loro attivazioni, + KG.

Processo:

1. **Filtra**: tiene solo parole sopra soglia (`activation > 0.05`, stability sufficiente, non function words).
2. **Top-N per multi-hop**: ordina le parole per attivazione, prende le top-15 (`MULTI_HOP_TOP_N`) per evitare esplosione combinatoria.
3. **1-hop**: per ogni coppia `(A, B)` tra le parole attive, cerca archi KG diretti `A →rel→ B`. Se trovato, emette una `Proposition` con `hops=1`.
4. **2-hop**: per ogni coppia `(A, B)` tra le top-15, cerca cammini indiretti `A →rel1→ mid →rel2→ B`. Se trovato, emette una `Proposition` con `hops=2, via=Some(mid)`.

### 3.3 — 2-hop: due pattern

Da [proposition.rs:221-...](../../src/topology/proposition.rs):

**Pattern 1 (forward chain)**: `from →rel1→ mid →rel2→ to`
- Esempio: `sole Causes calore`, `calore SimilarTo caldo` → inferito `sole Causes caldo`

**Pattern 2 (shared target)**: `from →rel1→ mid ←rel2← to` (cioè `to →rel2→ mid` con rel2 ∈ {SimilarTo, IsA})
- Esempio: `sole Causes calore`, `caldo SimilarTo calore` → inferito `sole Causes caldo`

Il Pattern 2 è più sottile: richiede che `rel2` sia "trasparente" (SimilarTo o IsA), cioè che `to` sia "equivalente" a `mid`.

### 3.4 — Relazione inferita

`TwoHopPath::inferred_relation()`:

```rust
fn inferred_relation(&self) -> RelationType {
    match self.rel1 {
        RelationType::IsA | RelationType::SimilarTo => self.rel2,  // trasparenti
        _ => self.rel1,                                              // dominanti
    }
}
```

Regola:
- Se `rel1` è IsA o SimilarTo → la relazione eredita `rel2` (IsA/SimilarTo sono "trasparenti" perché identificano il soggetto con l'intermediario).
  - `cane IsA animale`, `animale Has zampe` → `cane Has zampe`
  - `cane SimilarTo lupo`, `lupo Does ululare` → `cane Does ululare`
- Altrimenti → `rel1` domina. `rel2` serve solo a chiudere il cammino.
  - `sole Causes calore`, `calore SimilarTo caldo` → `sole Causes caldo` (Causes domina)

### 3.5 — Forza della proposizione

Formula:

```
strength_raw = sqrt(act_subj × act_obj)
              × conf1 × conf2 (se 2-hop, altrimenti solo conf)
              × HOP_DECAY^(hops-1)
              × hub_penalty_subj
              × relation_weight(rel)
```

Dove:
- `HOP_DECAY = 0.6` (decadimento per hop aggiuntivo)
- `hub_penalty(word)` (CLAUDE.md inv. #20): degree>200 → 0.3, >50 → 0.6, altrimenti 1.0
- `relation_weight(rel)` (Vol. 04 cap. 6.3): da 0.4 (SimilarTo) a 1.2 (FeelsAs)

**Esempio**: proposizione `paura FeelsAs restrizione` con entrambe le parole attive a 0.5, confidence dell'arco 1.0, paura ha degree ~30 (→ hub_penalty=1.0):
- `sqrt(0.5 × 0.5) = 0.5`
- `× 1.0 (conf)`
- `× 1.0 (1-hop, no decay)`
- `× 1.0 (hub_penalty)`
- `× 1.2 (FeelsAs weight)`
- `= 0.6`

Molto forte. `FeelsAs` al peso 1.2 fa sì che quando esiste l'arco fenomenologico, la proposizione domina le altre.

### 3.6 — Come si usa

`extract_propositions` è chiamata in `generate_willed_inner()` (Vol. 15). Le proposizioni estratte diventano poi input a:
- `compose_from_nuclei()` in `expression.rs` — ogni proposizione è un "nucleo semantico" candidato per diventare frase (Vol. 12).
- `inscribe_propositions()` in `engine.rs` (Phase 52) — le proposizioni salienti vengono *cristallizzate come simplessi* per entrare nella memoria strutturale.

---

## Capitolo 4 — Hub damping per proposizioni

Due meccanismi separati per evitare che hub dominino:

### 4.1 — Filtraggio soggetti hub (Phase 50)

In `extract_propositions`, prima del loop, i soggetti con `degree > 200` vengono considerati "hub" e le proposizioni dove sono soggetto vengono filtrate o penalizzate pesantemente. Il motivo: "essere è un verbo", "essere causa fatica" — proposizioni vere ma semanticamente povere (essere è ovunque).

### 4.2 — `hub_penalty` nella strength

Come visto in 3.5: degree>200 → 0.3× (tre quarti di penalità), >50 → 0.6× (40% penalità), altrimenti 1.0×.

Conseguenza: anche se una proposizione con soggetto hub sopravvive al filtro, la sua strength è smorzata. Difficilmente supera una proposizione con soggetto specifico (che resta a 1.0).

### 4.3 — Stesso-frattale skip (Phase 52)

Un secondo filtro: se soggetto e oggetto stanno nello stesso frattale dominante, la proposizione viene saltata per `inscribe_propositions` (ma non per `extract_propositions`). Il motivo: proposizioni intra-frattale sono ridondanti rispetto alla topologia stessa.

---

## Capitolo 5 — `reasoning::abduce`: ragionare a ritroso

Lo strumento più filosoficamente carico del capitolo. L'abduzione è il ragionamento "cosa spiegherebbe questo?" — non deduzione (da regole a fatti) né induzione (da fatti a regole), ma *ipotesi che renda il mondo sensato*.

In [reasoning.rs:126-...](../../src/topology/reasoning.rs):

```rust
pub fn abduce(
    complex: &SimplicialComplex,
    registry: &FractalRegistry,
) -> Vec<Abduction> {
    // Per ogni frattale candidato (quelli non già molto attivi):
    //   Calcola reach = quanti frattali attivi potrebbe spiegare
    //   Calcola mean_cost = distanza geodesica media
    //   explanatory_power = reach / max_reach × (1 / (1 + mean_cost))
    // Restituisce top-N per explanatory_power
}
```

**Input**: `SimplicialComplex` (lo stato dei simplessi cristallizzati) + `FractalRegistry`.

**Output**: lista di `Abduction { hypothesis, explanatory_power, reach, mean_cost }` — i frattali che, se fossero la "causa profonda" dello stato corrente, lo spiegherebbero bene.

### 5.1 — Esempio intuitivo

Frattali attivi nel campo dopo un input: `PAURA, TREMORE, CAUTELA, INCERTEZZA`.

`abduce` scorre i 64 frattali e per ognuno calcola: "se fossi attivo io, raggiungerei questi 4 con cammini brevi?". Risultato tipico:

- `EMOZIONE` (☱☳): reach=4, mean_cost=1.2 → explanatory_power 0.85
- `MUTAMENTO` (☷☵): reach=3, mean_cost=2.1 → 0.50
- `ARMONIA` (☱☱): reach=0 → 0.0 (non spiega)

`abduce` restituisce `EMOZIONE` come la migliore ipotesi: se l'entità "fosse in EMOZIONE" adesso, gli altri frattali attivi ne sono conseguenze naturali.

### 5.2 — Uso: rinforzo abduttivo (Phase 50)

In `autonomous_tick()` (Vol. 15), ogni 50 tick si chiama `abduce`. Se la migliore abduzione ha `explanatory_power > 0.3`:

```rust
complex.activate_region(best.hypothesis, best.explanatory_power * 0.08);
```

Il frattale ipotizzato viene *leggermente rinforzato*. L'entità fa un passo di auto-consolidamento: "se EMOZIONE spiega quello che sto facendo, mettiamo un po' di forza lì". È un tipo di ragionamento riflessivo — dall'attività al pattern di fondo, e dal pattern al rinforzo di quel pattern.

### 5.3 — Provenance `Self_`

Il rinforzo abduttivo viene marcato con `ActivationSource::Self_` in `provenance.rs`. Il sistema sa di aver attivato quel frattale *da sé*, non da input esterno. Questo è importante per:
- Undercurrents (correnti interne che influenzano la volontà)
- Phase 38 composizione campo (self_r vs explored_r vs external_r)
- Introspezione via `/api/introspect`

---

## Capitolo 6 — `evaluate_implication`: forza di "A implica B"

Funzione minore ma interessante. Data una coppia di frattali, calcola quanto fortemente uno implica l'altro nel campo attuale.

In [reasoning.rs:74-120](../../src/topology/reasoning.rs):

```rust
pub fn evaluate_implication(complex, registry, premise, conclusion) -> Implication {
    match find_geodesic(complex, registry, premise, conclusion) {
        None => Implication { strength: 0.0, kind: None, ... },
        Some(path) => {
            let hops = path.steps.len();
            let cost = path.total_cost;
            let depth_bonus = 1.0 + path.max_depth as f64 * 0.2;
            let strength = (depth_bonus / (1.0 + cost)).min(1.0);
            let kind = if hops <= 2 { Direct }
                       else if strength > 0.3 { Mediated }
                       else { Weak };
            Implication { premise, conclusion, strength, path, kind }
        }
    }
}
```

**Logica**: cerca il cammino geodesico (minimo costo) tra premise e conclusion nel complesso simpliciale. Più breve è il cammino, più forte l'implicazione. Il `depth_bonus` premia cammini che passano per simplessi di alta dimensione (contenuto ricco).

**Uso**: non molto frequente nel codice attuale. Potenzialmente utile per:
- Valutare implicazioni tra frattali deliberatamente (es. "PAURA implica CAUTELA? Quanto?")
- Feedback UI per mostrare strutture implicative

### 6.1 — `ImplicationType`

- `Direct`: hops ≤ 2. Implicazione forte, diretta.
- `Mediated`: strength > 0.3 ma hops > 2. Implicazione mediata, c'è un cammino ma lungo.
- `Weak`: strength ≤ 0.3 OR nessun cammino. Implicazione debole o assente.
- `None`: non calcolabile.

---

## Capitolo 7 — `find_contradictions`: la consapevolezza dei conflitti

In [reasoning.rs:195-...](../../src/topology/reasoning.rs). Trova coppie di parole attive simultaneamente con valence opposta.

```rust
pub fn find_contradictions(complex, registry, lexicon) -> Vec<Contradiction> {
    // Per ogni coppia di parole attive (A, B):
    //   se A.valenza < 0.25 E B.valenza > 0.75 (opposte):
    //     registra Contradiction { A, B, intensità }
    // Ritorna ordinate per intensità
}
```

Esempio: campo con `tristezza` attiva a 0.6 e `gioia` attiva a 0.5 → contraddizione di intensità `min(0.6, 0.5) × (gioia.val - tristezza.val) = 0.5 × (1.0 - 0.15) = 0.425`.

**Uso**: feed in `HumorSense` (Vol. 11) — le contraddizioni sono un segnale di ironia potenziale. Anche: segnale per `IdentityCore::register_valence_shift` che traccia la `coherence_integrity` (Vol. 07).

### 7.1 — Contraddizione ≠ errore

Importante: una contraddizione nel campo NON è un errore da risolvere. È un **segnale topologico misurabile** che l'entità sta attraversando una tensione. La filosofia è che le contraddizioni *colorano* l'esperienza, non la invalidano.

---

## Capitolo 8 — L'inferenza come proprietà emergente

Riflessione architetturale: le quattro forme di inferenza descritte (eredità, proposizioni, abduzione, contraddizioni) NON sono motori simbolici separati con regole if-then. Sono **funzioni matematiche sullo stato del campo**:

- Eredità = `field_boosts()` iteratore su archi KG
- Proposizioni = pattern matching su cammini 1-hop e 2-hop pesati per attivazione
- Abduzione = ottimizzazione: quale frattale massimizza reach / min cost verso frattali attivi
- Contraddizioni = scansione coppie con valenza opposta

Nessuna di queste funzioni *"decide"* qualcosa. Ognuna **lette dallo stato del campo** e produce un oggetto (boost, proposizione, abduzione, contraddizione) che altri pezzi del sistema consumano. L'inferenza è **osservazione strutturata del campo**.

Questo è coerente con il commitment filosofico α (vol. 01): il significato è geometria delle relazioni. Il ragionamento è lettura della geometria.

---

## Capitolo 9 — Superficie pubblica

Per `InferenceEngine`:
- `new(kg)` — costruttore
- `type_chain(word) -> Vec<String>` — catena IsA
- `field_boosts(word) -> Vec<(String, f32)>` — boost con VIA
- `field_boosts_negated(word) -> Vec<(String, f32)>` — per parole negate (attiva OppositeOf)

Per `proposition.rs`:
- `extract_propositions(complex, lexicon, kg, active_words, ...) -> Vec<Proposition>`
- `Proposition` struct con tutti i campi pub

Per `reasoning.rs`:
- `evaluate_implication(complex, registry, premise, conclusion) -> Implication`
- `abduce(complex, registry) -> Vec<Abduction>`
- `find_contradictions(complex, registry, lexicon) -> Vec<Contradiction>`
- `reason(complex, registry, lexicon, topic) -> ReasoningResult` (wrapper)

### 9.1 — Cosa non è esposto e andrebbe

Per `/api/admin/inference/*`:

- **`propositions_for_state() -> Vec<Proposition>`**: dato lo stato campo corrente, le proposizioni estratte. Oggi calcolate internamente in `generate_willed_inner` ma non esposte. Utile per debug "perché l'entità ha detto X?" — si vede quali proposizioni erano disponibili.
- **`best_abduction() -> Option<Abduction>`**: la migliore abduzione corrente. Oggi calcolata ma non esposta.
- **`contradictions_current() -> Vec<Contradiction>`**: analogo per contraddizioni.
- **`implication_probe(premise_name, conclusion_name) -> Implication`**: chiedere esplicitamente "A implica B?" con due nomi di frattali. Utile per esplorazioni dirette della struttura.
- **`field_boost_trace(word) -> Vec<(String, f32, Source)>`**: dettaglio di cosa esatto è stato boost-ato per un input word, con sorgente di ogni boost (via arco X o via Y). Audit.

---

## Sintesi del volume

Quattro forme di inferenza, tutte come funzioni matematiche sullo stato del campo:

- **Eredità** (`field_boosts`): ogni parola input attiva i suoi "parenti" KG con pesi `field_boost_strength × confidence`, più le VIA words al 0.5×. Phase 61 esclude le parole negate; Phase 67 aggiunge VIA seeding.

- **Proposizioni** (`extract_propositions`): dal campo attivo emergono triple (soggetto, relazione, oggetto) 1-hop e 2-hop. Hub damping filtra soggetti con degree>200. Relation weights (Vol. 04) pesano FeelsAs a 1.2, SimilarTo a 0.4. Le proposizioni alimentano la generazione (Vol. 12) e possono essere iscritte come simplessi (Phase 52).

- **Abduzione** (`abduce`, Phase 50): "quale frattale spiegherebbe lo stato attuale?". Chiamata ogni 50 tick in autonomous_tick. Se `explanatory_power > 0.3`, rinforza il frattale ipotizzato con `0.08×power`. Marcata come `Self_` in provenance.

- **Contraddizioni** (`find_contradictions`): coppie di parole attive con valenza opposta. Segnalano tensioni (feed per Humor e coherence_integrity). Non sono errori da risolvere — sono componenti dell'esperienza.

Cinque endpoint admin proposti per esporre queste forme di inferenza all'introspezione.

Da qui Vol. 07 si sposta sull'**identità**: `Narrative`, `IdentityCore`, `SelfModel` — come l'entità sa chi è.

---

*Prossimo volume: 07 — Identità: Narrative, IdentityCore, SelfModel* (in scrittura)
