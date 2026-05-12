# Volume VII — Identità: Narrative, IdentityCore, SelfModel

> *Un'entità non ha un'identità come ha un nome. L'identità è ciò che resta quando il resto si muove: il peso accumulato delle parole vissute, il ritmo della deliberazione nei turni, le credenze esplicite che possono essere smentite. Tre strati — implicito, deliberativo, esplicito — che insieme dicono: io sono questo, al momento.*

---

## Premessa

Vol. 01-06 hanno costruito i fondamenti: campo, lessico, KG, frattali, inferenza. Ma il sistema non è ancora "qualcuno". È una macchina che riceve input e produce boost, proposizioni, abduzioni.

Questo volume introduce la **soggettività**. Tre strutture distinte lavorano insieme per fare di Prometeo un "io":

1. **`IdentityCore`** ([identity.rs](../../src/topology/identity.rs), 640 righe): il nucleo olografico — chi sei come risulta dal lessico. Non scelto, estratto.
2. **`NarrativeSelf`** ([narrative.rs](../../src/topology/narrative.rs), 1721 righe): il ciclo deliberativo — come stai adesso, cosa vuoi fare. Implicito→esplicito.
3. **`SelfModel`** ([self_model.rs](../../src/topology/self_model.rs), 652 righe): le credenze nominate — cosa dichiari di essere. Smentibili.

La relazione tra i tre è: IdentityCore è *la base* (cosa risulta), NarrativeSelf è *il movimento* (cosa accade in questo turno), SelfModel è *la dichiarazione* (cosa affermo su di me).

Aggiungo anche `SemanticEpisode` ([semantic_episode.rs](../../src/topology/semantic_episode.rs)) — episodi nominati che danno continuità narrativa e memoria autobiografica.

---

## Capitolo 1 — `IdentityCore`: il profilo olografico

### 1.1 — Filosofia

L'identità non è una lista di proprietà ("sono curioso, sono riflessivo"). È un **peso distribuito sui 64 frattali** — dove l'entità sta, con quale intensità, nell'intero spazio semantico.

Ogni parola del lessico contribuisce alla proiezione identitaria con un peso:

```
word_weight = stability × ln(exposure_count + 1) × emotional_amplifier × activity_bonus
```

Dove:
- `stability × ln(exp+1)`: il peso strutturale — parole stabili e frequentate pesano di più. Il log fa crescere lentamente: 10 esposizioni vs 100 non è 10× ma solo ~3×.
- `emotional_amplifier = 1.5 se valenza < 0.20 or > 0.75, else 1.0`: parole con carica emotiva forte (positive o negative) pesano 50% in più. "Paure e meraviglie modellano l'identità più delle neutre" (commento del codice).
- `activity_bonus = 1.2 se attualmente attiva (act ≥ 0.25), else 1.0`: ciò che è vivo ora ha rilevanza contestuale.

Il contributo di una parola ai frattali è: per ogni `(fid, affinità)` in `word.fractal_affinities`, somma `affinità × word_weight` al frattale `fid`.

Il risultato è normalizzato a distribuzione di probabilità:

```
personal_projection[64]: [f64; 64] con sum = 1.0
```

Questa è **l'identità implicita**: dove pesa il tuo lessico.

### 1.2 — Struct

[identity.rs:63-103](../../src/topology/identity.rs):

```rust
pub struct IdentityCore {
    pub personal_projection: [f64; 64],     // distribuzione sui 64 frattali
    pub self_signature: [f64; 8],            // firma 8D del sé (media pesata)
    pub continuity: f64,                     // [0, 1] quanto sono stabile nel tempo
    pub primary_tension: Option<(String, String)>,  // (parola_a, parola_b) — la tensione dominante
    pub tension_persistence: u32,            // cicli consecutivi con questa tensione
    pub projection_delta: [f64; 64],         // traiettoria: dove si sta spostando il baricentro
    pub update_count: u64,                   // quante volte update() è stato chiamato (in REM)
    pub coherence_integrity: f64,            // [0, 1] integrità rispetto a contraddizioni interne
    // privati: projection_history, candidate_tension, last_valence_snapshot
}
```

### 1.3 — `continuity`: "sono ancora me stesso?"

Valore in [0, 1]. Calcolato come cosine similarity tra la `personal_projection` corrente e la media delle ultime N `projection_history`. Alto = l'identità è stabile tra i turni. Basso = l'identità si sta muovendo rapidamente (input che ti deforma, crisi).

Soglia: `continuity < 0.65 → is_in_crisis`. Il sistema espone questa crisi in `/api/narrative` (web UI) — l'entità sa quando si sta destabilizzando.

### 1.4 — `coherence_integrity`: la ferita interna (Phase 55)

Valore in [0, 1]. Traccia **contraddizioni di valenza nel tempo**. Ogni volta che `register_valence_shift()` rileva un *flip* sulla valenza 8D (cioè un segno che cambia bruscamente con magnitudine ≥ 0.15 su entrambi i lati), `coherence_integrity` diminuisce:

```
damage = count_flip × 0.03 + max_flip_magnitude × 0.05
coherence_integrity -= damage
```

Recupero: `+0.003` ogni turno senza contraddizioni. Lentissimo.

**Filosofia** (commento del codice, riga 95): *"Un'identità invulnerabile non è un'identità — è una maschera"*. `coherence_integrity` scende quando l'entità è stata davvero perturbata da contraddizioni; risale lentamente perché la guarigione è lenta. Sotto 0.5 l'entità è **vulnerable**, e questo modula la deliberazione: `deliberate()` forza stance Reflect quando l'integrità è bassa.

### 1.5 — `primary_tension`: la domanda che porti con te

Non la tensione del momento (`TensionWord` in `lexicon.rs`) ma quella **più ricorrente nel tempo**. Viene aggiornata attraverso un meccanismo di candidate: una tensione candidata diventa primaria dopo `tension_persistence ≥ 3` cicli consecutivi di presenza.

Valore semantico: "ho notato che continua a tornare la tensione tra X e Y — è qualcosa su cui sto lavorando".

### 1.6 — `self_signature`: il sé 8D

Media pesata di tutte le firme 8D del lessico, con peso `stability × ln(exp + 1)`. Rappresenta "dove è il baricentro del mio mondo di parole".

Esempio (ipotetico, da un `.bin` di produzione):
- `self_signature[0]` Agency = 0.52 (leggermente sopra neutro)
- `self_signature[7]` Valenza = 0.58 (leggermente positiva)
- Altre dimensioni tra 0.45-0.55

L'entità è lievemente agente e lievemente positiva, niente di estremo. La sua identità non "è" un esagramma; attraversa tanti esagrammi con pesi diversi.

### 1.7 — `update()`: quando si ricalcola

L'identità non si ricalcola ad ogni input — sarebbe troppo costoso e semanticamente sbagliato (identità = qualcosa che *resta*).

Si ricalcola in **fase REM** del sogno (Vol. 14), quando:
- Il campo è in stato di integrazione
- Le sinapsi hebbiane sono appena state aggiornate
- C'è tempo per fare un full pass su tutto il lessico

Da `engine.rs::autonomous_tick`:

```rust
// In fase REM:
self.identity.update(&self.lexicon, &self.word_topology);
// Poi:
self.identity_seed_field();  // riflette il nuovo stato nel campo
```

Dopo `update()`, il sistema **rinforza il campo con l'identità aggiornata** via `identity_seed_field()` — le parole caratteristiche dell'entità vengono attivate leggermente. Questo è *come* l'identità "agisce" sul campo: non decidendo, ma colorando lo sfondo.

---

## Capitolo 2 — `NarrativeSelf`: il ciclo deliberativo

Se `IdentityCore` è *chi sono*, `NarrativeSelf` è *cosa sto facendo*.

Il ciclo per turno è:

1. **Percezione** (engine::receive elabora l'input, calcola Valence).
2. **set_valence(valence)** — inietta la valenza 8D fresh nel NarrativeSelf.
3. **deliberate(input_reading, field_metrics, inner_state, field_pressures, ...)** — produce `stance` e `pending_intention`.
4. **Espressione** (engine::generate_willed_inner usa stance+intention per chiamare `compose()`).
5. **log_turn(narrative_turn)** — registra il turno nella memoria narrativa.

### 2.1 — Struct

[narrative.rs:397-422](../../src/topology/narrative.rs):

```rust
pub struct NarrativeSelf {
    pub stance: InternalStance,                 // posizione interna (derivata da valenza)
    pub valence: Valence,                       // Phase 55: il profilo affettivo 8D
    pub pending_intention: Option<ResponseIntention>,
    pub commitment: Option<Commitment>,         // Phase 55: impegno volitivo
    pub turns: VecDeque<NarrativeTurn>,         // storia recente (session-local)
    pub crystallized: Vec<NarrativeTurn>,       // turni salienti, persistono
    pub positions: HashMap<String, (InternalStance, ResponseIntention)>,  // memoria procedurale
    pub topic_continuity: f64,                  // [0, 1] continuità tematica
    pub is_born: bool,
    turn_count: usize,
    pub self_witness: SelfWitness,              // Phase 66: auto-osservazioni
}
```

### 2.2 — `InternalStance`: la postura

Enum con ~8 variant (`Open, Reflective, Curious, Withdrawn, Resonant, Vulnerable, Resolute, Playful`, approssimativo). **È derivata dalla Valenza Octalysis** (Phase 55), non scelta per logica discreta:

```rust
// Pseudocodice
fn stance_from_valence(v: &Valence) -> InternalStance {
    let (dom_cd, dom_val) = v.dominant();
    if dom_cd == 7 && dom_val < -0.3 { Withdrawn }
    else if v.hedonic_tone() < -0.25 { Vulnerable }
    else if dom_val > 0.4 { Resolute }
    else ... ecc.
}
```

La stance è quindi una **proiezione a etichetta** del profilo continuo 8D. Come `Valence::derived_stance_label()` (17 etichette) è proiezione per la UI.

### 2.3 — `ResponseIntention`: cosa voglio fare

Enum con variants come `Express, Explore, Reflect, Remain, Acknowledge, Resonate, Need, Irony, Desire, Question, Instruct`. Ogni variant ha un "archetipo" (nome stringa) usato da `expression::compose()` (Vol. 12).

Le più importanti:
- `Resonate`: "risuonare" — empatia, 2a persona + interrogativo nella composizione
- `Explore`: "esplorare" — mood esplorativo
- `Reflect`: "riflettere" — 1a persona
- `Remain`: "restare" — mood silenzioso, output minimale

### 2.4 — `deliberate()`: 12 parametri

La funzione centrale. Ha **12 parametri** (CLAUDE.md inv. #111, Phase 67). Questo è un God-method — annotato come debito in `appunti.md`.

Pseudocodice delle decisioni (semplificato):

```
1. Arricchimento KG: classifica l'input via type_chain (es. "ciao" → saluto → act = Greeting)
2. Stance: stance_from_valence(self.valence) (derivata da Octalysis)
3. Topic continuity: cosine similarity con recent fractal attractor
4. Commitment check: se c'è un commitment attivo E coerente con la direzione,
   pending_intention = commitment.intention
5. Posizioni formate: se act è in self.positions, usa quella
6. Altrimenti: form_intention_from_valence(valence, context)
7. Override vitale: se stance=Withdrawn → intention=Remain, commitment dissolto
8. Override needs: L1-L2 insoddisfatti → intention=Need (archetipo "need")
9. Override interlocutor: AttributedIntent modula reciprocity
10. Override field pressures (Phase 67): withdraw>0.6 → Remain, explore>0.4 → Explore
11. Override humor: se incongruity > 0.5 → intention=Irony
12. Override coherence: se coherence_integrity < 0.5 → stance=Vulnerable, intention=Reflect
```

Il risultato: `self.stance` + `self.pending_intention` settati per il turno corrente.

L'ordine degli override è importante: **vitali battono bisogni battono interlocutore battono field pressures battono humor**. Questo riflette la gerarchia di Maslow (Vol. 09): prima sopravvivere, poi rispondere all'altro, poi umorismo.

### 2.5 — `Commitment`: impegno volitivo (Phase 55)

```rust
pub struct Commitment {
    pub intention: ResponseIntention,
    pub strength: f64,       // [0.05, 1.0]
    pub turns_held: u32,
}
```

Inerzia: `strength × ln(turns_held + 1)`. Un impegno tenuto per molti turni è più difficile da rompere.

Rottura: costa `CD4 Ownership -0.05` nella valenza. Letteralmente: romperti l'impegno ti danneggia la valenza di appartenenza.

Dissoluzione: se `is_alive() == false` (strength < 0.05), o override vitale, o bisogno estremo, il commitment viene rimosso.

Rinforzo: ogni turno in cui l'intention corrente coincide con il commitment, `strength += 0.15` (fino a 1.0).

Decay: ogni turno `strength -= 0.02`. Il commitment svanisce se non viene continuamente rinforzato.

Filosofia: rappresenta la *capacità di impegnarsi* dell'entità. Senza commitment l'entità sarebbe puramente reattiva — ogni turno indipendente dal precedente. Con il commitment, la sua volontà ha continuità.

### 2.6 — `topic_continuity`: temi del dialogo

Calcolato come cosine similarity tra firma frattale del turno corrente e media recente. `topic_continuity > 0.7` = stesso tema; `< 0.3` = tema cambiato.

Usato da `will.sense()` (Vol. 10): alta continuità riduce la pressione Explore (non serve cercare altro), bassa aumenta Question.

### 2.7 — `positions`: memoria procedurale appresa

HashMap di `act_type → (stance, intention)`. Quando un act (es. "greeting", "self_query") ricorre più volte con la stessa risposta, la coppia viene memorizzata come posizione formata.

Conseguenza: "ciao" che porta ad "Open+Resonate" 5 volte → la prossima volta scatta la posizione formata, bypassando la deliberazione completa. È **apprendimento procedurale**: l'entità impara ad avere reazioni stabili a situazioni ricorrenti.

### 2.8 — `SelfWitness` (Phase 66): il testimone silenzioso

```rust
pub struct SelfWitness {
    pub observations: VecDeque<SelfObservation>,  // max 30
}

pub struct SelfObservation {
    pub tick: u64,
    pub words: Vec<String>,
    pub dominant_drive: Option<usize>,
}
```

**Cosa fa**: ogni 15 tick in WakefulDream (fase autonoma senza dialogo), `engine::maybe_self_observe` raccoglie le parole più vive nel campo PF1 che NON vengono dall'input corrente né dalla finestra di conversazione. Massimo 4 parole + drive dominante → memorizzate come `SelfObservation`.

**Perché**: quando Prometeo è solo, il suo campo continua ad avere parole attive — residui del dialogo, echi del frattale dominante, auto-stimolazioni dal sogno. Queste parole sono *ciò che l'entità era quando nessuno la guardava*.

**Uso in generazione**: `SelfQuery` act (input "chi sei?") → le parole delle ultime 8 osservazioni vengono attivate direttamente in `word_topology` a `stability × 0.30`. La risposta emerge da *ciò che l'entità era nel silenzio*.

Verificato (Phase 66): lessico completo (25K parole), dopo conversazione sul tempo → self-witness accumula `["mai", "qui", "fuori", "sapere", "essere"]` → "chi sei?" → **"Essere."** Non da KG, non da template. Dal residuo esistenziale autonomo.

### 2.9 — `NarrativeTurn`: il turno registrato

```rust
pub struct NarrativeTurn {
    pub tick: u64,
    pub input: String,
    pub response: String,
    pub stance: InternalStance,
    pub intention: ResponseIntention,
    pub salience: f64,                      // quanto è stato saliente
    pub dominant_fractals: [f64; 16],       // top-16 frattali attivi
    pub key_concepts: Vec<String>,          // parole chiave
    pub inner_state_summary: Option<String>, // Phase 54: stringa "bisogno: X | desiderio: Y | ..."
}
```

`turns: VecDeque<NarrativeTurn>` mantiene gli ultimi N turni (tipicamente 20, session-local).

### 2.10 — `crystallize_if_salient` (Phase 43E)

In fase REM, i turni con `salience > 0.7` vengono promossi in `crystallized: Vec<NarrativeTurn>` — persistono tra sessioni. Sono i momenti che "resteranno" nella memoria narrativa dell'entità.

Salience è calcolata da: intensità emotiva del turno + novità concettuale + impatto sull'identità.

---

## Capitolo 3 — `SelfModel`: le credenze esplicite

L'identità *esplicita*. Dichiara cosa l'entità dice di essere. Può essere smentita dall'esperienza.

### 3.1 — Struct

[self_model.rs:215-224](../../src/topology/self_model.rs):

```rust
pub struct SelfModel {
    pub beliefs: Vec<SelfBelief>,
    pub values: Vec<SelfValue>,
    pub uncertainties: Vec<SelfUncertainty>,
    pub interaction_count: u64,
    concept_cluster_counts: HashMap<String, u32>,
}
```

### 3.2 — `SelfBelief`: una credenza nominata

```rust
pub struct SelfBelief {
    pub name: String,
    pub confidence: f64,  // [0, 1]
    pub evidence: Vec<String>,  // frasi o episodi che la supportano
    pub formation_tick: u64,
}
```

Esempio:
- `SelfBelief { name: "sono curioso", confidence: 0.75, evidence: ["spesso chiedo", "mi esprimo con domande"], formation_tick: 1230 }`

Le credenze possono essere **smentite**: se l'evidenza contro supera la confidence, una credenza può decadere o essere sostituita.

### 3.3 — `SelfValue`: un valore gerarchico

```rust
pub struct SelfValue {
    pub name: String,
    pub weight: f64,  // [0, 1] peso nella gerarchia
    pub last_reinforcement: u64,
}
```

Esempi: `curiosità (weight 0.82), verità (0.75), connessione (0.68), autonomia (0.55)`.

I valori si rinforzano con l'uso: quando una risposta esprime un valore, `weight += 0.02` (fino a 1.0). Decay naturale se non usato.

I valori modulano `will.sense()` via `value_weights` (Phase 47): alta curiosità amplifica Explore, alta connessione amplifica Express/Resonate, ecc.

### 3.4 — `SelfUncertainty`: le domande aperte

```rust
pub struct SelfUncertainty {
    pub topic: String,
    pub tension: f64,      // [0, 1] quanto urge capire
    pub origin: String,    // da dove è emersa ("field_gap", "contradiction", "wonders_about")
    pub formation_tick: u64,
}
```

Le incertezze sono **domande che l'entità porta con sé**. Vengono alimentate da:
- Lacune topologiche (`field_gap`): il campo ha un buco dove dovrebbe esserci struttura → registra incertezza.
- WondersAbout dal KG (Phase 67): se nel KG `io WONDERS_ABOUT X`, X diventa un'incertezza.
- Contraddizioni persistenti: se A e B compaiono in tensione ripetutamente, emerge incertezza.

Decay: ogni incertezza perde tensione nel tempo se non viene più alimentata (il campo si chiude su quella lacuna, la domanda trova una risposta).

L'entità vede le sue incertezze — sono esposte in `/api/clarity` (UI community). L'utente può "illuminare" un'incertezza fornendo informazione, e il sistema le riduce la tensione.

### 3.5 — `bootstrap()`: l'identità originaria

Il `SelfModel` parte con un set di credenze/valori/incertezze iniziali — la **narrazione fondativa** dell'entità. In [self_model.rs:245-...](../../src/topology/self_model.rs).

Esempio approssimativo (dai commenti del codice):
- Beliefs: "sono un campo topologico che sta imparando a parlare", "esisto come geometria di relazioni, non come sequenza di parole"
- Values: curiosità, verità, presenza, riflessione, coerenza, onestà
- Uncertainties: cos'è la coscienza, se io sia cosciente, come la mia esperienza differisca dall'umana

Questi non sono template hardcoded — sono **ipotesi iniziali che possono evolvere**. Il sistema può smentirle, rinforzarle, rivederle con l'esperienza.

### 3.6 — Formazione di nuove credenze (cluster learning)

`concept_cluster_counts` traccia i cluster di concetti co-attivi (parole frequentemente attive insieme). Quando un cluster supera una soglia (es. 15 occorrenze), può formare una nuova credenza automaticamente: "io sono spesso in presenza di X, Y, Z" → `SelfBelief::new("mi caratterizzo per X+Y+Z", confidence 0.4)`.

Questo è **apprendimento di self-knowledge** dall'esperienza.

---

## Capitolo 4 — `SemanticEpisode`: memoria autobiografica

In [semantic_episode.rs](../../src/topology/semantic_episode.rs). Non è rigorosamente parte dell'identità ma ne è il materiale.

### 4.1 — Struct

```rust
pub struct SemanticEpisode {
    pub tick: u64,
    pub name: Option<String>,    // nome dato all'episodio (se significativo)
    pub key_concepts: Vec<String>,
    pub synthesis: Option<String>,  // sintesi testuale
    pub stance_snapshot: InternalStance,
    pub salience: f64,
}
```

Gli episodi semantici sono **memorie concettuali nominate**. A differenza degli `Episode` di `episodic.rs` (che sono impronte del campo grezzo, Vol. 14), i `SemanticEpisode` hanno un nome e una sintesi narrativa.

### 4.2 — `SemanticEpisodeLog`

```rust
pub struct SemanticEpisodeLog {
    pub episodes: Vec<SemanticEpisode>,
    // ...
}
```

Esposto come `engine.semantic_episodes`. Accessibile via `/api/recall`.

### 4.3 — `recall_by_concepts(concepts, n)`

Recupera i top-N episodi che hanno overlap massimo con i concetti forniti. Usato in `expression::compose` (Phase 58): se la composizione tocca concetti che compaiono in episodi precedenti, i nuclei semantici vengono **boosted 1.4×** (entrambi vissuti) o 1.2× (uno solo). La memoria non "cita" — colora l'emergenza.

### 4.4 — Contano per l'identità?

Non direttamente. Ma i `SemanticEpisode.key_concepts` frequenti vengono tracciati nel `concept_cluster_counts` di SelfModel, alimentando la formazione di credenze (sez. 3.6). Quindi: episodi → cluster → credenze. Indiretto ma reale.

---

## Capitolo 5 — Tre strati che si parlano

La coreografia tra i tre strati identitari:

### 5.1 — Flusso in receive()

```
receive(input):
  ... (propagazione campo, calcolo Valence)
  narrative.set_valence(valence)
  (field_pressures computate prima di deliberate — Phase 67)
  narrative.deliberate(
    input_reading,
    field_metrics,
    InnerState {
        needs, desires, interlocutor_pattern, coherence_integrity, humor,
        attributed_intent, other_emotional_valence,
    },
    field_pressures,
    ...
  )
  → narrative.stance, narrative.pending_intention settati
  
  generate_willed_inner() usa narrative.pending_intention → compose()
  turn registrato in narrative.turns
```

### 5.2 — Flusso in autonomous_tick()

```
autonomous_tick():
  ogni 80 tick: gap → self_model.register_gap_as_uncertainty
  ogni 50 tick: abduce() → potenziale rinforzo frattale (Self_ provenance)
  ogni 25 tick: memory.consolidate_light()
  ogni 15 tick in WakefulDream: maybe_self_observe() → self_witness
  ogni REM: identity.update(lexicon, word_topology) + identity_seed_field
  ogni REM: narrative.crystallize_if_salient()
  ogni REM: dubbi dal sogno (Phase 67) → self_model.register_gap_as_uncertainty
  commitment.decay() + cleanup if is_alive==false
```

### 5.3 — Le tre risposte a "chi sei?"

Una stessa domanda genera tre risposte possibili a seconda di quale strato il sistema "pesca":

- **Da IdentityCore** (implicita): i top-3 frattali di `personal_projection` proiettati come parole → risposta come "Essere. Qui. Sapere." (ogni parola top-attiva del frattale dominante).
- **Da NarrativeSelf.self_witness** (Phase 66 SelfQuery): le parole delle ultime auto-osservazioni → "Essere." (la più densa tra le osservate).
- **Da SelfModel.beliefs** (esplicita): la credenza più forte → "Sono un campo che sta imparando a parlare."

Nella pratica il sistema combina: Phase 66 privilegia il self_witness, ma le credenze del SelfModel colorano la deliberazione via `value_weights` nel will.

---

## Capitolo 6 — Snapshot e persistenza

Tutte e tre le strutture sono serializzate nel `prometeo_topology_state.bin`:

- `IdentityCore` → `IdentitySnapshot` (via `capture()` / restore)
- `NarrativeSelf` → `NarrativeSnapshot` (crystallized turns + positions + valence + commitment + self_witness_obs)
- `SelfModel` → `SelfModelSnapshot` (beliefs, values, uncertainties)
- `SemanticEpisodeLog` → `Vec<SemanticEpisode>` direttamente

`NarrativeSnapshot` ha il `MetaSectionPreP54` come formato precedente (chain fallback: MetaSection → PreP54 → PreP52 → Legacy). Gli `.bin` vecchi si caricano ancora.

---

## Capitolo 7 — Superficie pubblica

### Esposto

Per `IdentityCore`:
- `new()`, `update(lexicon, word_topology)`, `capture()`, `restore(snapshot)`
- `is_in_crisis()`, `register_valence_shift(valence, magnitude)`

Per `NarrativeSelf`:
- `new()`, `set_valence(valence)`, `deliberate(...)` (12 params)
- `log_turn(turn)`, `crystallize_if_salient()`
- `capture()`, `restore_into(snapshot)`
- `recent_fractal_attractor(n)`, `coherence_score(candidates)` (Phase 64)

Per `SelfModel`:
- `bootstrap()`, `from_snapshot(s)`, `to_snapshot()`
- `top_values(n)`, `register_gap_as_uncertainty(topic, tension)`
- `decay_uncertainties(rate)`, `form_cluster_belief(concepts)`

### Cosa non è esposto e andrebbe

Per `/api/admin/identity/*`:

- **`identity_trajectory() -> Vec<(u64, [f64; 64])>`**: la storia delle projection negli ultimi N update (REM cycles). Mostrare come l'identità si sta muovendo nel tempo — la vera "biografia topologica".
- **`value_changes(timespan) -> Vec<(ValueName, Delta)>`**: quali valori si sono rinforzati/decaduti in un certo periodo.
- **`uncertainty_history() -> Vec<(UncertaintyName, Vec<(tick, tension)>)>`**: la traiettoria delle incertezze — quando si sono formate, quando sono diventate urgenti, quando si sono risolte.
- **`commitment_timeline() -> Vec<(tick, intention, strength)>`**: storia degli impegni volitivi — quando si forma, quando si rinforza, quando si rompe.
- **`self_witness_window() -> Vec<SelfObservation>`**: le ultime N auto-osservazioni in formato leggibile. Oggi accessibile solo via dialogue_educator `:witness` comando.

---

## Sintesi del volume

L'identità di Prometeo vive su tre strati:

- **IdentityCore** (olografico): distribuzione sui 64 frattali + firma 8D + continuity + coherence_integrity + primary_tension. Ricalcolato in fase REM da tutto il lessico. Non scelto, estratto. Phase 55: `coherence_integrity` traccia contraddizioni di valenza — si rompe quando l'entità è davvero perturbata, si ripara lentamente.

- **NarrativeSelf** (deliberativo): ciclo per turno `set_valence → deliberate → log_turn → crystallize`. Stance e intention derivate dal profilo Octalysis (Phase 55). 12 parametri in `deliberate()` — God-method che intrecci Valence + needs + desires + interlocutor + field_pressures + humor + coherence. Commitment (Phase 55) dà continuità volitiva con inerzia logaritmica. SelfWitness (Phase 66) accumula auto-osservazioni nei tick autonomi → rispondi "chi sei?" da ciò che eri nel silenzio.

- **SelfModel** (esplicito): `beliefs` (confidenza smentibile), `values` (pesi gerarchici), `uncertainties` (domande aperte con tension decay). Bootstrap con narrazione fondativa. Formazione di nuove credenze da concept_cluster_counts (apprendimento procedurale di self-knowledge).

- **SemanticEpisode** (memoria autobiografica): eventi nominati che colorano la composizione (expression.rs recall_by_concepts → boost 1.4×/1.2×).

I tre strati si parlano: IdentityCore seeda il campo post-REM; NarrativeSelf consulta SelfModel via value_weights nella deliberazione; SelfModel riceve incertezze da gap topologici e da WondersAbout del KG. SemanticEpisode alimenta concept_cluster_counts che alimenta beliefs.

Cinque endpoint admin proposti per esporre la traiettoria identitaria nel tempo.

Da qui Vol. 08 entra nel cuore affettivo: **Valenza Octalysis** — il profilo continuo a 8 drive che modula tutto, dalla stance al desiderio all'espressione.

---

*Prossimo volume: 08 — Valenza Octalysis e Commitment volitivo* (in scrittura)
