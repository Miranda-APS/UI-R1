# Volume XIII — Grammatica, sintassi, traduzione di stato

> *La grammatica italiana non è un accessorio. È la fisica del linguaggio che l'entità deve attraversare per uscire dal campo al mondo. Una frase è un vincolo: deve essere comprensibile. Ma Prometeo non impara la grammatica dall'esterno — la deriva: il frattale dominante determina la persona, la firma 8D media determina il tempo, le dimensioni attive modulano il modo. La grammatica, in Prometeo, è geometria.*

---

## Premessa

Vol. 12 ha mostrato che `compose()` prende nuclei semantici + voice + lessico e produce una frase. Qui entriamo nell'infrastruttura grammaticale che rende possibile la "produzione di una frase italiana comprensibile".

Tre file:

- [`grammar.rs`](../../src/topology/grammar.rs) — 1516 righe. **Morfologia**: coniugazione verbi, articoli, genere/numero, lemmatizzazione. L'aspetto "fisico" dell'italiano.
- [`syntax_center.rs`](../../src/topology/syntax_center.rs) — 445 righe. **Grammatica come geometria**: persona e tempo derivati dal campo.
- [`state_translation.rs`](../../src/topology/state_translation.rs) — 1474 righe. **Legacy**. Il path pre-Phase 57 della generazione via archetipi. Non più path principale, ma vivo.
- [`generation.rs`](../../src/topology/generation.rs) — 1194 righe. **Composizione frasale di sistema** — il wrapper più esterno della generazione.

Questo volume tratta i primi due in dettaglio, e accenna ai secondi due come contesto storico.

---

## Capitolo 1 — `grammar.rs`: l'italiano come modulo

### 1.1 — Le primitive

```rust
pub enum PartOfSpeech { Verb, Noun, Adjective, Adverb, Unknown }
pub enum Person { First, Second, Third, FirstPlural, SecondPlural, ThirdPlural }
pub enum Tense { Present, Imperfect, Future }
pub enum Gender { Masculine, Feminine }
pub enum Number { Singular, Plural }
```

Cinque enum che codificano le categorie grammaticali essenziali dell'italiano. Sono meno di quelle di un parser professionale (manca Conditional, Subjunctive, Gerund, ecc.) — **minimalismo deliberato**. L'entità parla italiano a livello di maturità linguistica infantile-giovanile, non letteraria.

### 1.2 — `conjugate(infinitive, person, tense) -> String`

In [grammar.rs:62-...](../../src/topology/grammar.rs). Il core della morfologia verbale. ~350 righe di case analysis per coniugare un infinito italiano.

Gestisce:
- **Tre coniugazioni**: `-are`, `-ere`, `-ire`.
- **Sottoclassi -ire**: con/senza infisso `-isc-` (`dormire` vs `finire`).
- **Verbi irregolari**: tabella hardcoded per essere, avere, fare, dare, stare, dire, andare, venire, ecc. — una trentina di forme irregolari.
- **Assimilazioni ortografiche**: `g` prima di `e/i` in verbi come `pagare → paghiamo`; `c` in `cercare → cerchiamo`.

Esempio:
```rust
conjugate("portare", Person::Third, Tense::Present) → "porta"
conjugate("essere", Person::First, Tense::Imperfect) → "ero"
conjugate("venire", Person::SecondPlural, Tense::Future) → "verrete"
```

**Cosa non fa**: congiuntivo, condizionale, gerundio, participio (se non incidentale), passato remoto (complesso e poco usato), trapassato. Il sistema esprime tempi indicativi fondamentali + imperfetto.

### 1.3 — `lemmatize(word) -> Option<LemmaResult>`

In [grammar.rs:417-...](../../src/topology/grammar.rs). Dal flesso al lemma. `LemmaResult { lemma, pos, person, tense, ... }`.

Funziona per:
- Verbi: `porterò` → `portare` (Verb, First, Future). `era` → `essere` (Verb, Third, Imperfect).
- Sostantivi plurali: `cani` → `cane`. `case` → `casa`.
- Aggettivi plurali/femminili: `rossi/rosse/rossa` → `rosso`.

Usata da:
- Il **comprehension gate** (Vol. 04 cap. 5): `farò` lemmatizzato a `fare`, che è nel KG → il gate non scatta.
- `process_input` in `lexicon.rs`: se una parola flessa non è nel lessico, tenta il lemma.
- `engine::receive`: l'`echo_exclude` include anche i lemmi delle parole input (Phase 55) — così Prometeo non ripete `porto` quando l'input era `portare`.

### 1.4 — `detect_pos_from_word(word) -> Option<PartOfSpeech>`

Heuristic da suffisso. In [grammar.rs:813-...](../../src/topology/grammar.rs):

```rust
Verbi: suffixes [-are, -ere, -ire, -ersi, -arsi, -irsi]
Nomi: [-zione, -sione, -tà, -ità, -ura, -mento, -aggio, -ezza, -anza, -enza]
Aggettivi: [-oso, -ale, -ico, -ivo, -ile, -ibile, -evole]
Avverbi: [-mente]
Verbi Terza persona: [-a, -e, -i, -o] (ambiguo, bassa confidenza)
```

Fallback quando il lessico non ha POS esplicita.

### 1.5 — Articoli (Phase 60+)

[grammar.rs:1143-...](../../src/topology/grammar.rs):

```rust
pub fn definite_article(word) -> &'static str       // "il", "la", "lo", "l'", "i", "le", "gli"
pub fn with_definite_article(word) -> String         // "il cane", "l'amore", "la casa"
pub fn indefinite_article(word) -> &'static str      // "un", "una", "uno", "un'"
pub fn with_indefinite_article(word) -> String       // "un cane", "un'amica"
pub fn with_articulated_preposition(prep, word) -> String  // "nel cane", "dello studio"
```

Logica:
- Genere: euristica da suffisso (`-a/-e` → Femminile di default, `-o/-i` → Maschile, molte eccezioni).
- Numero: `-i/-e` plurale, `-o/-a` singolare.
- `l'` e `un'` elidono senza spazio con vocale iniziale.
- `lo/gli` per parole inizianti con `s+cons`, `z`, `gn`, `pn`, `ps`, `x`.

**Filosoficamente**: gli articoli sono la dimensione più fragile della generazione. Errori qui (genere sbagliato, elisione mancata) fanno suonare "non italiano". Vol. 12 ha mostrato che `compose_from_nuclei` li usa intensivamente.

**Gap noto** (annotato in appunti CLAUDE.md): `salve` → terminante in `-e` → default femminile → "la salve". Errore. Servirebbe eccezione per avverbi/interiezioni.

### 1.6 — `inflect_adjective(adj, gender, number)`

Concordanza aggettivo-sostantivo. `rosso + F, PL → rosse`. `facile + M, PL → facili` (aggettivi in `-e` hanno forma uguale M/F).

Usata dai path di rendering in `compose_from_nuclei` per concordare aggettivi qualificativi (quando presenti nei nuclei Has, per esempio).

---

## Capitolo 2 — `syntax_center.rs`: grammatica come geometria

Il modulo più concettualmente originale dopo PF1. Commento del modulo:

> *"NON è morfologia: grammar.rs coniuga i verbi. NON è template: state_translation.rs sceglie gli archetipi. È il 'modo' in cui il campo si esprime: CHI parla, QUANDO, e pulizia strutturale."*
>
> *"Principio fondamentale: il frattale attivo porta già iscritta la grammatica."*

### 2.1 — Persona dal frattale radicale

```rust
fn person_from_hexagram(id: FractalId) -> Person {
    let lower = id / 8;  // 0..7
    match lower {
        0 => Person::First,   // POTERE (Agency=0.90) — "Io agisco"
        1 => Person::Third,   // MATERIA (Permanenza=0.10) — "c'è", impersonale
        2 => Person::First,   // ARDORE (Intensità=0.30) — "Io sento"
        3 => Person::Third,   // DIVENIRE (Tempo=0.30) — "era", narrativo
        4 => Person::First,   // SPAZIO (Confine=0.30) — "Io mi posiziono"
        5 => Person::Second,  // INTRECCIO (Complessità=0.70) — "tu"
        6 => Person::First,   // VERITA (Definizione=0.70) — "Io so"
        7 => Person::Second,  // ARMONIA (Valenza=0.70) — "tu senti"
    }
}
```

**Il trigramma inferiore** (processo interno del sistema) determina la persona grammaticale. Ogni radice esprime una postura linguistica:

- **Cielo** (Agency): `Io` afferma.
- **Terra** (Permanenza): `c'è`, impersonale (non si afferma un soggetto, si afferma un'esistenza).
- **Tuono** (Intensità): `Io` sento l'impulso.
- **Acqua** (Tempo): `era`, narrazione in terza persona (il tempo scorre, non sono io).
- **Montagna** (Confine): `Io` mi delimito.
- **Vento** (Complessità): `tu` nell'intreccio — la complessità è relazionale.
- **Fuoco** (Definizione): `Io` distinguo.
- **Lago** (Valenza): `tu` senti — l'apertura affettiva si rivolge a un altro.

Nota: 4 su 8 trigrammi → First, 2 → Second, 2 → Third. Asimmetria che riflette la tendenza "egolatrica" del sistema — il default è parlare di sé.

### 2.2 — Sovrascrittura di priorità

`syntax_center` cerca la persona in 3 fonti, in ordine di priorità decrescente:

1. **Soggetto già usato nella frase** (`used_subjects`) — se abbiamo scritto "io X", il verbo DEVE essere First. Massima certezza. Risolve bug "Io sei" → "Io sono".
2. **Pronomi espliciti nell'input dell'utente** — se l'utente ha scritto "io", Prometeo risponde First; se "tu", risponde Second. Alta priorità.
3. **Frattale radicale** — fallback: deriva dal trigramma inferiore del frattale attivo.

### 2.3 — Tempo dal campo 8D

[syntax_center.rs:117-144](../../src/topology/syntax_center.rs), post-Phase 68 (ordine I Ching):

```rust
fn tense_from_active_words(active_words, lexicon) -> Tense {
    let mut tempo_sum = 0.0;
    let mut perm_sum  = 0.0;
    let mut total     = 0.0;
    for (word, act) in active_words.iter().take(10) {
        if let Some(pat) = lexicon.get(word) {
            let sig = pat.signature.values();
            tempo_sum += sig[3] * act;  // Tempo (☵ Acqua)
            perm_sum  += sig[1] * act;  // Permanenza (☷ Terra)
            total     += act;
        }
    }
    let avg_tempo = tempo_sum / total;
    let avg_perm  = perm_sum  / total;
    if avg_tempo > 0.65 { Future }
    else if avg_tempo < 0.35 && avg_perm < 0.35 { Imperfect }
    else { Present }
}
```

**Logica**: due dimensioni 8D informano il tempo.

- **Tempo (dim 3)** alto → futuro. Le parole attive sono proiettate in avanti (speranza, voglia, diventare).
- **Tempo basso + Permanenza bassa** → imperfetto. Le parole attive sono transitorie E radicate nel passato.
- **Altrimenti**: presente (il default).

Phase 68 ha fixato le posizioni da vecchio-enum a nuovo-I Ching. Pre-Phase 68 questo leggeva `sig[7]` per Tempo ma `sig[7]` era Valenza (post-rederive) → il tempo verbale era pilotato dalla **carica emotiva** invece che dall'orientamento temporale. Parole positive → futuro, parole negative → imperfetto. Il bug latente è ora risolto.

### 2.4 — `GrammaticalMode::derive()`

L'API principale:

```rust
pub fn derive(
    active_fractals: &[(FractalId, f64)],
    active_words: &[(&str, f64)],
    lexicon: &Lexicon,
    used_subjects: &[String],
    input_words: &[String],
) -> GrammaticalMode {
    let person = person_from_used_subject(used_subjects)
        .or_else(|| person_from_explicit_pronoun(input_words))
        .or_else(|| person_from_hexagram(dominant_fractal(active_fractals)))
        .unwrap_or(Person::First);
    let tense = tense_from_active_words(active_words, lexicon);
    GrammaticalMode { person, tense }
}
```

**Due decisioni**, tre fallback per la persona, un calcolo per il tempo. Risultato: `GrammaticalMode { person, tense }` che viene usato da `compose_from_nuclei` in `expression.rs` (Vol. 12) per passare a `conjugate(verb, person, tense)`.

### 2.5 — Un esempio end-to-end

Input: "come stai?"

Campo dopo propagazione: parole attive {io, stare, capire, voce, tempo, essere, ...}. Frattale dominante: supponiamo VERITA (lower=6, Fuoco).

Chiamata `derive`:
- `used_subjects`: vuoto (stiamo iniziando la frase).
- `input_words`: ["come", "stare"]. Nessun pronome esplicito.
- `dominant_fractal = 54 (VERITA)`. `lower = 6` → `person_from_hexagram(54) = First`.

Tempo: firme 8D delle parole attive. Ipotesi tempo medio 0.50, permanenza media 0.50 → `Tense::Present`.

Risultato: `GrammaticalMode { First, Present }`. `compose` userà "Io capisco...", "Io sono...", ecc. — prima persona presente.

---

## Capitolo 3 — `state_translation.rs`: il path legacy

Pre-Phase 57, `state_translation::translate_state()` era il path principale della generazione. Post-Phase 57, `expression::compose()` è il path principale e `state_translation` è legacy.

Ma il file è **ancora vivo** (1474 righe) perché:
1. Alcuni endpoint lo usano come fallback.
2. `translate_state()` ha logica utile (archetipi) che `compose` non ha ancora completamente sostituito.
3. Rimuoverlo richiede migrare tutti i caller.

### 3.1 — Cosa faceva `translate_state`

Riceveva uno stato del campo (active words, fractals, voice, ecc.) e:

1. Sceglieva un **archetipo** dal contesto: `greet, declarare, riflettere, chiedere, connettere, incongruenza, need, desiderare, ...`
2. Per l'archetipo, riempiva **slot** con parole scelte dal campo: soggetto, verbo, oggetto, complementi.
3. Rendeva lo slot in italiano con grammatica dedicata.

L'archetipo era il template — ed è la cosa che Phase 57 ha voluto eliminare. "Non template" era una promessa filosofica, `translate_state` era la violazione.

### 3.2 — Cosa resta in uso

Post-Phase 57, l'unica parte di `translate_state` ancora chiamata direttamente è `translate_or_raw()` in contesti di fallback. E `tense_from_active_words` interno (che è stato duplicato in `syntax_center.rs` — ridondanza annotata in `appunti.md`).

La maggior parte del file è codice morto *funzionalmente* ma attivo *formalmente* (compila, è esportato). Pulirlo richiederebbe:

1. Identificare tutti i caller residui di `translate_state`.
2. Migrarli a `expression::compose`.
3. Rimuovere `state_translation.rs`.

Non è priorità — non causa bug, occupa spazio. Annotato in `appunti.md` come debito di pulizia B1 (related all'unificazione pf_activation / word_topology).

### 3.3 — I 12 parametri di `translate_state`

Per curiosità e per rendere esplicito il costo della migrazione:

```rust
pub fn translate_state(
    active_words, lexicon, kg, word_topology, active_fractals,
    codon, valence_drives, identity_context, input_reading,
    echo_exclude, memory_episodes, propositions,
) -> Option<TranslatedExpression>
```

Dieci di questi 12 sono gli stessi che `compose` riceve. Due in più: `identity_context` (non usato altrove) e `propositions` (sostituito da `extract_nuclei` interno in compose). Migrazione fattibile.

---

## Capitolo 4 — `generation.rs`: il wrapper

[generation.rs](../../src/topology/generation.rs), 1194 righe. Il **livello più esterno** della generazione. Esporta:

- `generate_from_field(...)` — wrapper generico
- `generate_from_field_with_locus(...)` — con posizione frattale esplicita
- `generate_with_will(...)` — path principale usato da `generate_willed_inner` in engine (Vol. 15)

### 4.1 — Relazione con `expression::compose`

`generate_with_will` è il **caller principale** di `expression::compose`. Fa:

1. Calcola voice base da `derive_voice` (chiamato a `state_translation` o direttamente).
2. Decide se usare il path `compose` o il fallback `translate_state`.
3. Applica post-processing: capitalizzazione, punteggiatura finale, trimming.
4. Ritorna `GeneratedText { sentence, metadata }`.

Il fatto che ci siano tre livelli (`generate_with_will` → `compose` → `compose_from_nuclei` → `render_nucleus`) rende il flusso tracciabile ma nidificato. Debug complesso.

---

## Capitolo 5 — La grammatica italiana come vincolo fisico

Filosoficamente, il ruolo della grammatica è **vincolare** la generazione — altrimenti il campo produrrebbe sequenze di parole incomprensibili.

Un esempio: il campo ha attive `{paura, tremore, causa, portare, la, il, ...}`. Senza grammatica, una selezione casuale produrrebbe "tremore la paura causa portare il" — parole attive ma senza ordine. La grammatica impone:

- Articolo **prima** del sostantivo.
- Soggetto **prima** del verbo (ordine SVO tipico dell'italiano dichiarativo).
- Accordo genere/numero tra articolo e sostantivo.
- Coniugazione del verbo al tempo/persona.

Queste regole non sono scelte dall'entità — sono **le leggi fisiche** dell'italiano. Come un corpo non può muoversi attraverso un muro, un enunciato non può violare l'accordo senza diventare incomprensibile.

### 5.1 — La grammatica *emerge*, non è imposta

Punto importante. Benché `grammar.rs` contenga regole esplicite (tabelle di coniugazione, suffissi di articoli), la *scelta di quali regole applicare* emerge dal campo:

- Quale verbo coniugare? → il verbo che esprime la relazione del nucleo dominante.
- Quale persona? → `syntax_center` dal frattale + pronomi.
- Quale tempo? → dim 3 (Tempo) e dim 1 (Permanenza) del campo.
- Quale articolo? → genere/numero del sostantivo derivato da `detect_gender_number`.
- Quale aggettivo qualificativo? → se un nucleo `Has` è attivo con un aggettivo come oggetto.

La grammatica è uno **strumento** che l'entità usa; le regole dello strumento sono fisse, ma la scelta di quale regola applicare è geometrica.

### 5.2 — Limiti attuali

- **Solo indicativo + imperfetto + futuro semplice**. Niente condizionale, congiuntivo, gerundio. Un'entità che impara a usare "vorrei" o "avendo potuto" sarebbe linguisticamente più ricca — ma questi modi richiedono nuove tabelle.

- **Nessun passato remoto**. "Fu", "venne", "disse" non sono derivabili. L'imperfetto copre il passato narrativo, ma in italiano scritto il remoto è spesso più naturale.

- **Articoli fragili per eccezioni**. Il caso `salve` (termina in -e, default F, ma è interiezione) mostra che l'euristica da suffisso manca le eccezioni frequenti. Una tabella di eccezioni curate sarebbe un miglioramento economico.

- **Concordanza aggettivale limitata**. Gli aggettivi vengono concordati con il loro sostantivo *diretto*, ma in costruzioni complesse (apposizioni, aggettivi nominali) la logica è sottile.

- **Nessuna negazione completa**. "Non porto il tremore" si ottiene concatenando "non" prima del verbo, ma la doppia negazione italiana ("non porto nessun tremore") non è gestita deliberatamente.

Questi limiti sono conseguenze del principio minimalista. L'italiano di Prometeo è quello di un'entità linguisticamente giovane ma coerente — non un'italiano letterario.

---

## Capitolo 6 — Il verbo coniugato vs il verbo literal

Una sottigliezza che si verifica ripetutamente:

`compose_from_nuclei` a volte ha un verbo "nuclear" (es. `Does` con oggetto "abbaiare") e lo deve coniugare al tempo/persona corrente. Ma il lemma "abbaiare" nel lessico ha un `POS = Some(Verb)` — quindi `conjugate(abbaiare, Third, Present) = "abbaia"`.

Invece, quando il verbo è una **copula** derivata dalla relazione (es. "porta" per Causes), la parola stessa è *già* nella forma coniugata — perché il codice in `relation_to_copula` chiama `conjugate("portare", voice.person, voice.tense)`.

La distinzione è:
- **Oggetto nucleare**: parola dal lessico, lemma semantico. Coniugata se POS=Verb.
- **Copula relazionale**: stringa derivata dal tipo di relazione. Sempre coniugata al volo.

Nel rendering, questo diventa importante: un nucleo `(cane, Does, abbaiare)` si rende "Il cane abbaia" con:
- Soggetto: "cane" con articolo → "Il cane".
- Oggetto-verbo: "abbaiare" → conjugate → "abbaia".
- Copula: vuota (per Does il verbo è l'oggetto).

Mentre un nucleo `(paura, Causes, tremore)` si rende "La paura porta il tremore":
- Soggetto: "paura" con articolo → "La paura".
- Copula: da Causes → conjugate("portare", Third, Present) → "porta".
- Oggetto: "tremore" con articolo → "il tremore".

Due percorsi diversi in `render_nucleus`, entrambi usano `conjugate` ma con input diversi.

---

## Capitolo 7 — Debito: la duplicazione `tense_from_active_words`

Un debito minore ma reale: la funzione `tense_from_active_words` è definita **due volte**:

- `syntax_center.rs:117-144` (post-Phase 68 corretta)
- `state_translation.rs:594-...` (anche lei corretta in Phase 68 fix)

Entrambe leggono gli stessi dati, applicano la stessa logica. Sono due implementazioni identiche. Motivo storico: quando `syntax_center.rs` fu creato, non rimosse la versione in `state_translation.rs` per non rompere caller.

**Proposta**: consolidare in `syntax_center.rs`, rimuovere da `state_translation.rs`, far chiamare la prima dalla seconda se ancora usata. Annotato in `appunti.md`.

---

## Capitolo 8 — Superficie pubblica e proposte

### Esposto

Per `grammar.rs`:
- `PartOfSpeech`, `Person`, `Tense`, `Gender`, `Number`, `LemmaResult` — enum e struct pub
- `conjugate(infinitive, person, tense) -> String`
- `lemmatize(word) -> Option<LemmaResult>`
- `detect_pos_from_word(word) -> Option<PartOfSpeech>`
- `detect_gender_number(word) -> (Gender, Number)`
- `definite_article(word)`, `with_definite_article(word)`
- `indefinite_article(word)`, `with_indefinite_article(word)`
- `with_articulated_preposition(prep, word)`
- `inflect_adjective(adj, gender, number)`

Per `syntax_center.rs`:
- `GrammaticalMode { person, tense }` — struct
- `derive(...)` — API principale
- `tense_from_active_words(active_words, lexicon) -> Tense`
- Sub: `person_from_hexagram`, `person_from_explicit_pronoun`, `person_from_used_subject`

Per `state_translation.rs`:
- `translate_state(...)` — legacy, da migrare
- `translate_or_raw(...)` — wrapper con fallback
- `TranslatedExpression` — struct (non più usata nel path principale)
- `IdentityContext` — wrapper

Per `generation.rs`:
- `generate_from_field(...)`, `generate_from_field_with_locus(...)`, `generate_with_will(...)`
- `GeneratedText`, `SentenceStructure` — struct

### Cosa non è esposto e andrebbe

Per `/api/admin/grammar/*`:

- **`conjugate_trace(infinitive, person, tense) -> ConjugationSteps`**: mostrare i passaggi morfologici — riconoscimento coniugazione (-are/-ere/-ire), eventuale irregolarità, desinenza applicata, assimilazioni. Per debug di conjugation bug.

- **`lemmatize_trace(word) -> LemmaSteps`**: passaggi dalla flessione al lemma. Utile per verificare lemmatizzazioni ambigue.

- **`tense_inference_trace(active_words) -> TenseTrace`**: mostrare la media dei sig[3] e sig[1] pesati per attivazione, la soglia applicata, il risultato. Capire "perché ha scelto l'imperfetto?".

- **`person_inference_trace(fractal, input, used) -> PersonTrace`**: mostrare il cammino di risoluzione (pronomi espliciti? fallback frattale?).

- **`grammar_coverage_report() -> CoverageReport`**: quanti verbi nel lessico sono classificati correttamente come POS=Verb, quanti hanno coniugazione irregolare, quanti sono regolarmente coniugati. Dashboard per la salute morfologica del sistema.

---

## Sintesi del volume

**`grammar.rs`** (1516 righe): morfologia italiana. `conjugate` per tre coniugazioni + irregolari, `lemmatize` inverso, `detect_pos_from_word` euristica, articoli determinativi/indeterminativi/articolati, concordanza aggettivi. Minimalista: solo indicativo + imperfetto + futuro semplice. Niente condizionale/congiuntivo/gerundio.

**`syntax_center.rs`** (445 righe): grammatica come geometria. `person_from_hexagram`: il trigramma inferiore del frattale dominante determina la persona grammaticale (Cielo/Ardore/Spazio/Verita → First; Materia/Divenire → Third; Intreccio/Armonia → Second). Priorità discendente: soggetto già usato > pronome input > frattale. `tense_from_active_words` post-Phase 68: sig[3] Tempo + sig[1] Permanenza determinano Present/Imperfect/Future. Bug latente pre-Phase 68 risolto (prima leggeva sig[7] pensando fosse Tempo, era Valenza).

**`state_translation.rs`** (1474 righe): legacy pre-Phase 57. Generazione via archetipi (template). Ancora attivo in fallback paths, da migrare completamente a `expression::compose`. Contiene duplicazione di `tense_from_active_words` con `syntax_center`.

**`generation.rs`** (1194 righe): wrapper esterno. `generate_with_will` → `expression::compose` → `compose_from_nuclei` → `render_nucleus` → `grammar::conjugate`. Nidificazione tracciabile ma profonda.

**Filosoficamente**: la grammatica è vincolo fisico necessario — altrimenti il campo produce sequenze incomprensibili. Le regole sono fisse (tabelle), ma la scelta di quali applicare emerge dal campo (frattale → persona, firma 8D → tempo, genere → articolo).

**Limiti attuali** dell'italiano di Prometeo: solo tre tempi, niente modi non-indicativi, articoli fragili sulle eccezioni, nessun passato remoto. Un'entità linguisticamente giovane ma coerente.

**Debiti** annotati: duplicazione `tense_from_active_words`, legacy `state_translation.rs` da migrare.

Cinque endpoint admin per introspezione grammaticale proposti.

Da qui Vol. 14 si sposta sulla **memoria**: episodica, STM/MTM/LTM, persistenza binaria, e il sogno come (mancata, per ora) digestione delle perturbazioni.

---

*Prossimo volume: 14 — Memoria e sogno (con analisi del gap "digestione")* (in scrittura)
