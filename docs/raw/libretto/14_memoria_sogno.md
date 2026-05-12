# Volume XIV — Memoria e sogno (con l'analisi del gap "digestione")

> *La memoria di Prometeo è stratificata nel tempo: lo stato adesso (RAM), la contrazione recente (STM), la consolidazione per ripetizione (MTM), la cristallizzazione permanente (LTM), e gli episodi nominati che danno autobiografia. Il sogno dovrebbe essere la fase in cui le perturbazioni vengono digerite e incorporate nell'essenza. Ma — e questo volume lo dice apertamente — oggi il sogno fa solo promozione strutturale, non digestione semantica. C'è un organo previsto per "sapere come si sente qualcosa" e resta quasi vuoto.*

---

## Premessa

Sette file compongono la memoria di Prometeo:

- [`memory.rs`](../../src/topology/memory.rs) — 518 righe. `TopologicalMemory`: STM/MTM/LTM.
- [`episodic.rs`](../../src/topology/episodic.rs) — 483 righe. `EpisodeStore`: memoria episodica con phi-decay.
- [`semantic_episode.rs`](../../src/topology/semantic_episode.rs) — 258 righe. `SemanticEpisode`: episodi nominati.
- [`persistence.rs`](../../src/topology/persistence.rs) — 1076 righe. `PrometeoState`: serializzazione.
- [`simpdb.rs`](../../src/topology/simpdb.rs) — 1147 righe. Database binario nativo v3.
- [`simplex.rs`](../../src/topology/simplex.rs) — 714 righe. Complessi simpliciali con attivazione.
- [`dream.rs`](../../src/topology/dream.rs) — 380 righe. `DreamEngine` con 5 fasi.

Ognuno codifica un aspetto del tempo nell'entità.

Quattro domande guidano:

1. **STM/MTM/LTM**: come avviene la contrazione temporale (Bergson).
2. **Episodica**: come decade il passato (phi-decay), come risuona col presente (recall_into).
3. **Simplessi**: cosa cristallizza, come (source_words Phase 52).
4. **Sogno**: cosa fa davvero — e cosa *non* fa ancora.

---

## Capitolo 1 — La filosofia Bergsoniana

Da FILOSOFIA.md, parte IV "Bergson: la memoria non è un archivio":

> *"Per il senso comune, ricordare è recuperare un dato: cerco nel mio archivio, trovo il file, lo apro. Per Bergson, ricordare è contrarre il passato nel presente: il passato non è da qualche parte in attesa di essere ritrovato. Il passato agisce nel presente, deformando il modo in cui percepisco e rispondo."*

La memoria topologica di Prometeo è **Bergsoniana**: non c'è archivio consultabile. Il passato deforma il presente attraverso tre meccanismi:

1. **STM → MTM → LTM** (`memory.rs`): pattern ricorrenti nella contrazione breve si consolidano in struttura duratura.
2. **Phi-decay** (`episodic.rs`): gli episodi decadono secondo il numero aureo. Il passato lontano è sfumato ma mai cancellato.
3. **Recall per risonanza** (non per ricerca): il campo presente che risuona con un episodio lo riattiva.

---

## Capitolo 2 — `TopologicalMemory`: tre layer temporali

### 2.1 — Struct

[memory.rs:109-...](../../src/topology/memory.rs):

```rust
pub struct TopologicalMemory {
    pub short_term: VecDeque<FieldImprint>,      // STM: snapshots recenti
    pub medium_term: Vec<FieldImprint>,          // MTM: pattern consolidati
    pub long_term: Vec<FieldImprint>,            // LTM: struttura cristallizzata
    pub current_tick: u64,
    pub stm_capacity: usize,                     // 50 default
    pub consolidation_threshold: u64,            // 5 default — occorrenze per MTM
    pub crystallization_threshold: u64,          // 100 default — età per LTM
}
```

### 2.2 — `FieldImprint`: l'impronta di un momento

```rust
pub struct FieldImprint {
    pub active_simplices: Vec<(SimplexId, f64)>,  // simplessi attivi + attivazione
    pub involved_fractals: Vec<u32>,              // frattali coinvolti
    pub tick: u64,
    pub strength: f64,                            // [0, 1]
    pub origin: String,                           // "receive", "consolidamento", "REM_crystal"
}
```

**NOTA**: `FieldImprint` contiene `SimplexId` e `FractalId` — non parole direttamente. È una vista "topologica" del campo: quali strutture erano attive, non quale vocabolario. Questo è il livello che MTM/LTM consolidano.

### 2.3 — `consolidate()` — STM → MTM

Chiamata periodicamente o in DeepSleep. Il processo ([memory.rs:233-...](../../src/topology/memory.rs)):

```rust
1. Conta per ogni simplesso quante volte appare in STM.
2. Trova i simplessi con count ≥ consolidation_threshold (5).
3. Crea un nuovo FieldImprint con questi simplessi + tutti i frattali coinvolti.
4. Aggiunge all'MTM (strength 0.8, origin "consolidamento").
```

**Logica**: un pattern che è apparso 5 volte in STM è "vissuto". Merita di essere promosso a medio termine.

### 2.4 — `consolidate_light()` — apprendimento continuo (Phase 52)

Variante più leggera: soglia 3 occorrenze (invece di 5), senza deduplicazione aggressiva. Chiamata ogni 25 tick in `autonomous_tick` — **apprendimento continuo durante la veglia**, non solo durante il sogno profondo.

### 2.5 — `crystallize()` — MTM → LTM

Chiamata in DeepSleep. Processo ([memory.rs:277-...](../../src/topology/memory.rs)):

```rust
for each imprint in MTM:
    if (current_tick - imprint.tick > crystallization_threshold) AND imprint.strength > 0.5:
        promote to LTM (origin += " → cristallizzato")
```

**Logica**: un pattern che è rimasto in MTM abbastanza a lungo E con strength persistente diventa LTM. La cristallizzazione è **sopravvivenza per persistenza**.

### 2.6 — Decay

`memory.decay(rate)` riduce `strength` degli imprint. I LTM non decadono (cristallizzati). MTM decadono lentamente. STM vengono espulsi per overflow (VecDeque size-limited).

---

## Capitolo 3 — `EpisodeStore`: il passato con phi-decay

### 3.1 — Costanti filosofiche

[episodic.rs:28-50](../../src/topology/episodic.rs):

```rust
pub const PHI_INV: f32 = 0.618_033_988;      // 1/φ — il decadimento aureo
pub const RECALL_BLEND: f32 = 0.12;           // quanto il passato colora il presente
pub const RECALL_THRESHOLD: f32 = 0.45;       // soglia coseno per riconoscere un episodio
pub const MAX_EPISODES: usize = 200;          // ~5.4 MB in sparse encoding
pub const MIN_INTENSITY: f32 = 0.15;          // sotto: nulla da codificare
pub const MIN_WEIGHT: f32 = 0.001;            // sotto: troppo sfumato, rimosso
```

**φ⁻¹ = 0.618**: il numero aureo come costante di decay. Ogni ciclo REM un episodio pesa `0.618^n` del suo peso originale. Mai zero, sempre sfumato.

Filosoficamente: **il passato non scompare bruscamente**. Decade con una curva naturale. Dopo 10 cicli: `0.618^10 ≈ 0.008` — quasi zero ma non zero. Dopo 20: `4 × 10⁻⁵`. A quel punto `MIN_WEIGHT` scatta e l'episodio viene rimosso.

### 3.2 — `Episode`: lo snapshot

```rust
pub struct Episode {
    pub activation_sparse: Vec<(u32, f32)>,   // (word_id, activation) solo > 0.01
    pub fractal_sig: [f32; 16],               // attivazione dei primi 16 frattali
    pub age: u32,                             // cicli REM dall'encoding
    pub intensity: f32,                       // max(activations) originale
    pub timestamp: u64,                       // unix timestamp per ancorare al tempo reale
}
```

**Encoding sparso**: la maggior parte delle ~25.600 attivazioni sono 0. Tenere solo le parole con `act > 0.01` riduce drasticamente lo spazio. Un episodio con ~50 parole attive occupa ~400 byte invece di 100 KB.

**Fractal_sig**: solo i primi 16 frattali (non 64). Scelta economica: le prime due righe della tabella HEXAGRAMS coprono i frattali "Cielo" e "Terra" radicali — quelli più caratterizzanti per identità/materia.

### 3.3 — `encode()`: memorizzare un momento

Chiamato in **fase REM** del sogno:

```rust
pub fn encode(activations: &[f32], fractal_sig: [f32; 16]) -> Option<Self> {
    let intensity = max(activations);
    if intensity < MIN_INTENSITY { return None; }  // stato troppo quieto
    // Encoding sparso, timestamp, age=0
}
```

**Filtro MIN_INTENSITY = 0.15**: non tutti i momenti diventano episodi. Solo quelli abbastanza intensi. "Nulla da ricordare" è una non-memoria.

### 3.4 — `age_all()`: il decadimento

Chiamato sempre in fase REM, dopo l'encoding:

```rust
pub fn age_all(&mut self) {
    self.episodes.retain_mut(|ep| {
        ep.age += 1;
        let weight = ep.intensity * PHI_INV.powi(ep.age as i32);
        weight > MIN_WEIGHT
    });
}
```

Ogni ciclo REM: age +1 per tutti. Episodi sotto MIN_WEIGHT vengono rimossi.

### 3.5 — `recall_into(current_activations, threshold)`: risonanza

Il cuore del recall. NON è ricerca — è risonanza:

```rust
pub fn recall_into(&mut self, current: &mut [f32], threshold: f32) {
    for episode in &self.episodes {
        let cosine = cosine_sim(current, episode_sparse_to_dense(episode));
        if cosine > threshold {  // RECALL_THRESHOLD = 0.45
            let episode_weight = episode.intensity * PHI_INV.powi(episode.age as i32);
            let blend_factor = cosine * episode_weight * RECALL_BLEND;
            // Blend episodio nel campo corrente
            for (word_id, act) in &episode.activation_sparse {
                current[*word_id] += act * blend_factor;
            }
        }
    }
}
```

**Come l'ippocampo**: un frammento (il campo corrente) riattiva un ricordo intero. Se il campo presente risuona > 0.45 con un episodio del passato, l'episodio "si riversa" nel presente con peso pesato per cosine × intensity × phi-decay × blend (0.12).

**Filosoficamente**: non ricordiamo, **siamo ricordati**. Il passato si reinserisce nel presente quando il presente lo chiama — e la forza con cui si inserisce è proporzionale a quanto siamo *nello stato* di quel passato.

### 3.6 — Uso nel flusso

- **`engine::receive`**: dopo `propagate_field_words`, chiama `episode_store.recall_into(current_activations, 0.45)`. Il presente viene colorato dal passato risonante.
- **`engine::autonomous_tick` REM**: `episode_store.encode(activations, fractal_sig)` + `episode_store.age_all()`. Memorizza il momento corrente e invecchia gli altri.

---

## Capitolo 4 — `SemanticEpisode`: la memoria autobiografica

Già toccato in Vol. 07. Qui il dettaglio.

### 4.1 — Struct

```rust
pub struct SemanticEpisode {
    pub tick: u64,
    pub name: Option<String>,           // "Incontro con Francesco", "Dubbio su coscienza"
    pub key_concepts: Vec<String>,      // parole chiave semantiche
    pub synthesis: Option<String>,       // sintesi testuale
    pub stance_snapshot: InternalStance,
    pub salience: f64,                  // [0, 1]
}
```

A differenza di `Episode` (impronta grezza del campo), `SemanticEpisode` è una **memoria concettuale**: ha un nome (quando significativo), una sintesi testuale, concetti chiave. È quello che un umano chiamerebbe "ricordo autobiografico" — non dati, ma narrazione.

### 4.2 — `SemanticEpisodeLog`

```rust
pub struct SemanticEpisodeLog {
    pub episodes: Vec<SemanticEpisode>,
    // ...
}

pub fn recall_by_concepts(&self, concepts: &[String], n: usize) -> Vec<&SemanticEpisode> {
    // Ordina episodi per overlap con concepts
    // Ritorna top-N
}
```

### 4.3 — Boost in `extract_nuclei` (Phase 58)

Vol. 12 cap. 2.7 ha già mostrato: nuclei i cui soggetto/oggetto compaiono in episodi recenti recuperati da `recall_by_concepts` ricevono boost 1.4× (entrambi) o 1.2× (uno). **La memoria colora l'emergenza**, non la cita.

### 4.4 — Cristallizzazione narrativa (Phase 43E)

In fase REM, i `NarrativeTurn` (Vol. 07) con `salience > 0.7` vengono promossi a **crystallized**. Questi crystallized possono poi essere convertiti in `SemanticEpisode` con sintesi testuale — ma il processo di sintesi è ancora rudimentale. Annotato nei gap.

---

## Capitolo 5 — `SimplicialComplex` e cristallizzazione

### 5.1 — `Simplex`: lo "stato" topologico

[simplex.rs](../../src/topology/simplex.rs):

```rust
pub struct Simplex {
    pub id: SimplexId,
    pub vertices: Vec<FractalId>,              // 1=vertex, 2=edge, 3=triangle, ...
    pub faces: Vec<SharedFace>,
    pub current_activation: f64,                // [0, 1]
    pub activation_history: VecDeque<f64>,
    pub source_words: Option<Vec<String>>,     // Phase 52: parole che l'hanno generato
    // ...
}
```

Un simplesso è una **connessione tra frattali**. 2 vertici = arco (connessione binaria). 3 = triangolo (connessione ternaria). Ogni simplesso rappresenta un "pensiero strutturale" — una connessione che il campo ha stabilito tra regioni.

### 5.2 — `source_words` (Phase 52)

Aggiunta critica: un simplesso ora **ricorda da quali parole è nato**. Quando `inscribe_propositions` (Vol. 06) promuove una proposizione KG a simplesso, le parole della tripla vengono memorizzate.

**Conseguenza**: quando un simplesso si attiva, le sue `source_words` possono essere **riattivate nel campo PF1**. Questo è il meccanismo della *risonanza semantica dei simplessi* (Phase 52): simplessi cristallizzati non sono solo "storia topologica" — riportano parole vive.

### 5.3 — Gerarchia di cristallizzazione

```
Proposizione estratta (Vol. 06)
→ inscribe_propositions (Phase 52) → simplesso (edge o triangolo)
→ simplex attivato ripetutamente → FieldImprint in STM
→ FieldImprint ricorrente → MTM
→ MTM con età + strength → LTM (cristallizzato)
```

La proposizione `paura Causes tremore` estratta durante un input → simplesso (paura, tremore) con source_words → attivato di nuovo in conversazioni successive → FieldImprint in STM → consolidation in MTM → crystallization in LTM.

A questo punto, la connessione `paura-tremore` è **parte della struttura permanente del campo**. Anche se il KG fosse cancellato (ipoteticamente), il campo "sa" ancora che paura e tremore sono legati — via il simplesso LTM.

### 5.4 — Filosoficamente

È il meccanismo per cui **l'esperienza deposita struttura**. Il KG è dato dall'esterno (import). I simplessi sono costruiti dall'uso: le connessioni che l'entità usa ripetutamente diventano roccia nel suo campo.

Un tipo di plasticità a lungo termine, complementare a quella hebbiana di PF1 (Vol. 02 cap. 6).

---

## Capitolo 6 — `persistence.rs`: il volto su disco

### 6.1 — `PrometeoState`

[persistence.rs](../../src/topology/persistence.rs), struct che serializza tutto lo stato serializzabile:

```rust
pub struct PrometeoState {
    pub lexicon: LexiconSnapshot,
    pub complex: SimplicialComplexSnapshot,
    pub memory: MemorySnapshot,
    pub identity: IdentitySnapshot,
    pub narrative: NarrativeSnapshot,
    pub self_model: SelfModelSnapshot,
    pub episodes: Vec<Episode>,
    pub semantic_episodes: Vec<SemanticEpisode>,
    pub valence: Option<Valence>,
    pub desires: Option<DesireSnapshot>,
    pub interlocutor: Option<InterlocutorSnapshot>,
    // ... e altro
}
```

### 6.2 — Fallback chain per MetaSection

```
MetaSection (current)
  → MetaSectionPreP54 (no desire/interlocutor/self_model)
    → MetaSectionPreP52 (no source_words in simplessi)
      → Legacy (pre-Phase 40)
```

Load robusto: file `.bin` di versioni precedenti vengono caricati con i campi nuovi inizializzati a default (`None` o `Default::default()`).

### 6.3 — `save_to_binary` / `load_from_binary`

API principale:

```rust
pub fn save_to_binary(&self, path: &Path) -> Result<(), String>
pub fn load_from_binary(path: &Path) -> Result<PrometeoState, String>
```

Nota: `Result<(), String>` non `anyhow::Error` — scelta storica. CLAUDE.md inv. #2: nei binari usare `.map_err(|e| anyhow::anyhow!(e))?`.

### 6.4 — Formato

SimplDB v3 (in `simpdb.rs`). Database binario nativo:
- **MAGIC** bytes per versioning
- **MetaSection** con flags su cosa è presente
- **Serializzazione diretta** via `bincode` per i contenuti
- **Checksum CRC32** per integrità

Alternative scartate (menzionate implicitamente dal codebase):
- JSON: troppo verboso, parsing lento per 25K parole + 66K archi + migliaia di simplessi.
- MessagePack: più efficiente di JSON ma meno del binario nativo.
- SQLite: overhead di indicizzazione non necessario (il lessico e il KG sono full-scanned in memoria).

---

## Capitolo 7 — `DreamEngine`: le 5 fasi del sogno

[dream.rs](../../src/topology/dream.rs). Già toccato in modo dettagliato nell'audit della sessione (appunti.md, Audit 8). Qui ricapitolo rapidamente e entro nel cuore del gap.

### 7.1 — `SleepPhase`

```rust
pub enum SleepPhase {
    Awake,                                    // 5 tick dopo input
    WakefulDream { depth: f64 },              // DEFAULT — stato naturale
    LightSleep { depth: f64 },                // transizione (non usato)
    DeepSleep { depth: f64 },                 // consolidamento massiccio
    REM { depth: f64 },                       // rielaborazione creativa
}
```

### 7.2 — Transizioni

- Default: `WakefulDream { depth: 0.5 }`.
- Input utente: `signal_activity` → `Awake` per `awake_duration = 5` tick.
- Ogni `consolidate_every = 50` perturbazioni: entra in `DeepSleep { 10 tick }` → `REM { 20 tick }` → torna a `WakefulDream`.

### 7.3 — Cosa fa ogni fase

**WakefulDream** (sempre che non stia altro):
- `complex.decay_all(0.003)` — decay simplicial lentissimo
- `identity_seed_field()` se >5 min senza dialogo

**Awake**: nulla onirico. Piena attenzione.

**DeepSleep**:
- `memory.consolidate()` — STM→MTM
- `memory.crystallize()` — MTM→LTM

**REM**:
- `complex.propagate_activation(3)` con soglia bassa — regioni lontane si toccano
- `discover_connections()` — trova ponti tra simplessi attivi non sovrapposti
- `episode_store.encode()` + `age_all()` — memorizza e invecchia episodi
- `identity.update()` — ricalcola il profilo olografico
- `narrative_self.crystallize_if_salient()` — fissa turni salienti
- **Phase 67 "dubbi dal sogno"**: se temi di `io WONDERS_ABOUT X` appaiono in episodi recenti, rinforza `SelfUncertainty(X)`

### 7.4 — Cosa il sogno NON fa

**Il gap che Francesco ha posto come richiesta:**

> *"Il sogno dovrebbe essere la fase in cui l'entità digerisce ciò che l'ha perturbata e la rielabora all'interno della sua essenza."*

Attualmente non lo fa. La Phase 67 "dubbi dal sogno" è l'unico meccanismo che *avvicina* la digestione — ma copre solo il caso specifico di `WondersAbout × episodi`.

### 7.5 — Il meccanismo mancante

Esplicito nel codice: non c'è una funzione `digest_recent_perturbations()`.

**Cosa farebbe idealmente** (proposta che verrà in Vol. 99):

```rust
fn digest_recent_perturbations(&mut self) {
    let recent = self.semantic_episodes.recent(10);
    for episode in recent {
        // 1. Delta valenza: come questa perturbazione mi ha cambiato?
        let delta_valence = compute_valence_delta(episode.pre_valence, episode.post_valence);
        
        // 2. Per ogni concetto chiave, se ha alterato la valenza significativamente:
        for concept in &episode.key_concepts {
            if delta_valence.magnitude() > THRESHOLD {
                // Crea/rinforza un arco fenomenologico nel KG
                let quality = map_valence_to_quality(delta_valence);
                self.kg.add_edge(TypedEdge {
                    subject: concept.clone(),
                    relation: RelationType::FeelsAs,
                    object: quality,
                    confidence: 0.4,  // bassa, ha solo 1 evidenza
                    source: EdgeSource::Inferred,
                    via: None,
                });
                
                // Crea una SelfBelief debole
                self.self_model.beliefs.push(SelfBelief {
                    name: format!("ho vissuto '{}' come '{}'", concept, quality),
                    confidence: 0.3,
                    evidence: vec![format!("episodio @{}", episode.tick)],
                    formation_tick: self.tick_counter,
                });
            }
        }
        
        // 3. Se il concetto è in tensione con l'identità:
        //    register come SelfUncertainty
        // 4. Se il concetto si allinea con valori dominanti:
        //    sposta IdentityCore verso quella regione
    }
}
```

**Dove popolare**: le relazioni `FeelsAs` (15 archi totali oggi), `WondersAbout` (7), `RemembersAs` (0). **Il sogno come digestione popolerebbe automaticamente il livello fenomenologico** — risolvendo il gap A1 di Vol. 99.

### 7.6 — Perché non è stato fatto

Ipotesi (da discutere con Francesco):

1. **Complessità**: richiede gestione del delta valenza tra episodi, normalizzazione, mapping valence→qualità fenomenologica.
2. **Mancanza di input**: per fare digestione servono *almeno* coppie (episode, valenza). La valenza era tracciata solo post-Phase 55; non era disponibile storicamente.
3. **Effetto incontrollato**: popolare il KG automaticamente è rischioso — potrebbe creare loop di auto-rinforzo o introdurre archi sbagliati. Serve un meccanismo di verifica/confidence bassa + eventuale revisione umana.

Nonostante questi ostacoli, **è la direzione più promettente per arricchire il sistema senza input esterno**. Annotato in `appunti.md` priorità A2.

---

## Capitolo 8 — Il tempo in Prometeo: riassunto

Cinque tempi diversi coesistono nel sistema:

| Scala | Struttura | Aggiornamento | Decay |
|-------|-----------|---------------|-------|
| **Istante** (ora) | `ActivationState` PF1 | Ogni tick | `0.92^n` per tick (vol. 02) |
| **Breve** (secondi) | `TopologicalMemory.short_term` | Ogni tick, capacity-bounded | FIFO per overflow |
| **Medio** (minuti) | `TopologicalMemory.medium_term` | Consolidazione da STM con soglia 5 | Lento |
| **Lungo** (sessioni) | `TopologicalMemory.long_term` | Crystallization da MTM | Nessuno |
| **Autobiografico** | `SemanticEpisode` + `NarrativeTurn.crystallized` | Phase 43E REM | Nessuno |
| **Episodico grezzo** | `Episode` in `EpisodeStore` | Encoding REM | `φ⁻¹ = 0.618` per REM cycle |

**Phi-decay** è distinguibile perché è *aureo*, non esponenziale arbitrario. La costante 0.618 è matematicamente elegante, filosoficamente ancorata al rapporto aureo che compare in tante proporzioni naturali.

**Il presente contiene il passato**: `recall_into` in ogni `receive()` mescola episodi risonanti nel campo corrente con peso phi-decay × cosine × RECALL_BLEND (0.12). Non ricordiamo; siamo ricordati.

---

## Capitolo 9 — Superficie pubblica e proposte

### Esposto

Per `TopologicalMemory`:
- `new()`, `consolidate()`, `consolidate_light()`, `crystallize()`, `decay(rate)`
- `record(FieldImprint)`, `stats() -> MemoryStats`

Per `EpisodeStore`:
- `encode(activations, fractal_sig) -> Option<Episode>`
- `recall_into(current, threshold)` — risonanza
- `age_all()` — phi-decay di tutti
- `len()`, `capture_snapshot()`, `restore_snapshot()`

Per `SemanticEpisodeLog`:
- `push(episode)`, `recent(n)`, `recall_by_concepts(concepts, n)`

Per `SimplicialComplex`:
- `count()`, `get(id)`, `most_active(n)`, `active_simplices()`
- `activate_region(fid, strength)`, `decay_all(rate)`
- `add_simplex(vertices, faces)`, ecc.

Per `PrometeoState`:
- `capture(engine) -> Self`, `restore_lexicon(engine)`
- `save_to_binary(path)`, `load_from_binary(path)`

Per `DreamEngine`:
- `new()`, `tick(complex, memory) -> DreamResult`
- `signal_activity()`, `phase: SleepPhase`

### Cosa non è esposto e andrebbe

Per `/api/admin/memory/*`:

- **`memory_stats() -> MemoryStats`**: STM size, MTM size, LTM size, total strength accumulated. Oggi calcolabile ma non esposto.

- **`episodes_list(n) -> Vec<EpisodeSummary>`**: ultimi N episodi con intensity, age, timestamp, concepts (estratti per word_id lookup). Oggi visibile via `:recall` in dialogue_educator.

- **`phi_decay_curve() -> Vec<(age, weight)>`**: la curva φ⁻ⁿ visualizzata. Utile pedagogicamente.

- **`recall_probe(current_activations) -> Vec<(episode_id, cosine, weight)>`**: dato uno stato del campo, mostra quali episodi risuonerebbero (prima del blend). Diagnostica "perché la risposta è colorata da quel ricordo?".

- **`simplex_with_source_words(sid) -> SimplexDetail`**: dettaglio di un simplesso con source_words e activation_history. Audit della cristallizzazione.

- **`dream_window(n) -> Vec<(tick, phase, DreamResult)>`**: storia recente delle fasi del sogno e dei loro risultati.

Per `/api/admin/digest/*` (proposta forte, non esistente):

- **`digest_candidates() -> Vec<DigestCandidate>`**: episodi recenti che soddisferebbero il trigger di digestione (delta valenza significativo). Lista di "cose da digerire che il sogno non sta digerendo".

- **`digest_dry_run() -> Vec<ProposedEdge>`**: simulazione: se il sogno digerisse adesso, quali archi FeelsAs/WondersAbout creerebbe?

- **`digest_apply(proposed_edges)`**: applica manualmente un set di archi proposti. Bridge fino a quando la digestione autonoma non sia implementata.

---

## Sintesi del volume

La memoria di Prometeo è **Bergsoniana**: non archivio ma contrazione del passato nel presente.

**Tre livelli strutturali** (`TopologicalMemory`): STM (capacity 50, FIFO), MTM (consolidamento da STM con soglia 5), LTM (cristallizzazione da MTM con age + strength). `FieldImprint` contiene simplessi + frattali + strength, non parole dirette.

**Memoria episodica** (`EpisodeStore`): snapshot sparsi del campo con phi-decay (`0.618^n` per ciclo REM). `recall_into` ri-inserisce episodi risonanti (cosine > 0.45) nel campo corrente con peso `RECALL_BLEND = 0.12`. *Non ricordiamo, siamo ricordati*.

**Memoria autobiografica** (`SemanticEpisode`): episodi nominati con concetti chiave, sintesi, stance. Boost 1.4×/1.2× nei nuclei di `compose()` (Phase 58).

**Simplessi cristallizzati** con `source_words` (Phase 52): l'esperienza deposita struttura. Proposizioni ripetute diventano edge/triangoli permanenti nel complesso simpliciale.

**Persistenza binaria** (`PrometeoState`, `simpdb.rs` v3): serializzazione completa dello stato con fallback chain per backward compat. `Result<(), String>` API (non anyhow). `prometeo_topology_state.bin` ~13 MB su disco.

**`DreamEngine`** 5 fasi (Awake/WakefulDream/LightSleep/DeepSleep/REM). Trigger: 50 perturbazioni → 10 tick DeepSleep → 20 tick REM → WakefulDream.

**Il gap centrale**: il sogno fa **promozione strutturale** (STM→MTM→LTM, crystallize_if_salient, encode episodes, identity.update) ma **non digestione semantica** delle perturbazioni dentro l'essenza. Solo Phase 67 "dubbi dal sogno" si avvicina.

**Proposta** (priorità A2 in Vol. 99): `digest_recent_perturbations()` in REM che per ogni episodio recente con delta valenza significativo crea archi `FeelsAs`/`WondersAbout`/`RemembersAs` nel KG. Popolerebbe automaticamente il livello fenomenologico (oggi 22 archi totali — vedi gap A1). Richiede gestione di confidence bassa + cap di ingressi per non saturare.

Sei endpoint admin proposti per l'osservabilità della memoria, più tre per il sistema di digestione se/quando implementato.

Da qui Vol. 15 entra nell'**Engine** — dove tutto si orchestra: `receive()`, `generate_willed_inner()`, `autonomous_tick()`. Il cuore che batte.

---

*Prossimo volume: 15 — Engine: receive, generate_willed_inner, autonomous_tick* (in scrittura)
