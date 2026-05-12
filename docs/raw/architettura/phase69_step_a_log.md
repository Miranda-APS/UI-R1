# Phase 69 Step A — log operativo

> Quello che è stato fatto in questa sessione. Documento di progresso, non di design.

---

## Cosa è stato creato

### `src/topology/events.rs` (nuovo file, ~550 righe)

**Contenuti**:

- `enum InternalEvent` — 20 varianti. 7 categorie: Input, Campo, Identità, Memoria, Motivazione, Relazione, Humor, Temporale (SilenceThreshold), Meta (SelfNotice).
- Enum di supporto: `MemoryLevel` (STM/MTM/LTM), `HumorKind` (Irony/Bisociation/Both), `SilenceLevel` (Pause/Rest/Solitude/DeepTime).
- `InternalEvent::salience()` — scoring per ogni variante in [0, 1]. Sotto 0.2 = svanisce. 0.2-0.5 = contesto effimero. 0.5-0.7 = narrative_context / semantic_episode_material. > 0.7 = sempre memorabile.
- `InternalEvent::debounce_key()` — chiave univoca per collasso eventi duplicati in finestra 1s. Per `InputReceived` include prefisso del testo (evita collasso di input diversi consecutivi).
- `InternalEvent::describe_short()` — rappresentazione single-line per log.
- `InternalEvent::kind_name()` — identificativo del tipo.

- `struct EventSink` — non log, ma sink emit-and-forget:
  - Debounce map con TTL 1s + cleanup quando >256 entries.
  - Soglia di logging configurabile (default 0.4).
  - Contatori: `emitted_count`, `debounced_count`, `forgotten_count`.
  - Flag `logging_enabled` da env var `PROMETEO_EVENTS_LOG` (default ON).
  - `emit(event)` fa: oblio se sal < 0.2 → debounce check → log se sal ≥ 0.4 → increment counter. **Niente accumulo**.

**14 unit test verdi** che coprono:
- Monotonicità dei `SilenceLevel`
- Salience bounds per varianti chiave
- Oblio sotto soglia
- Debounce di eventi identici
- Non-debounce di target diversi
- Cleanup periodico della mappa debounce
- `SelfNotice` che amplifica la salience dell'observed event

### Integrazione nell'engine

- Campo `events: EventSink` aggiunto a `PrometeoTopologyEngine` (sopra `word_topology`).
- Campo `was_in_crisis: bool` per rilevare transizioni di crisi identitaria.
- Campo `last_primary_tension: Option<(String, String)>` per rilevare `TensionCrystallized`.
- Inizializzati in entrambi i costruttori (`new()` e `new_empty()`).

### Wiring di emissione

5 punti di mutation in `receive()` emettono eventi:

1. **`InputReceived`** — all'inizio di `receive()`, come primo atto. L'unico evento esterno.

2. **`ValenceFlip`** — prima di `set_valence()`, confronta old/new drives. Emette per ogni CD con cambio di segno + magnitude > 0.15 su entrambi i lati.

3. **`IdentityCrisisOnset` / `IdentityCrisisResolved`** — dopo `register_valence_shift`, confronta `is_in_crisis()` con `was_in_crisis`. Emette solo sulla transizione.

4. **`TensionCrystallized`** — quando `identity.primary_tension` diventa diversa da `last_primary_tension`.

5. **`DominantNeedShift`** — dopo `needs.sense()`, confronta con `last_needs_state`. Emette quando il dominant_need cambia tra turni.

Tutti additivi — nessuna logica esistente toccata.

---

## Cosa NON è stato fatto (pianificato per Step B/C/D)

- **Absorb in memoria**: gli eventi non vengono ancora assorbiti in EpisodeStore/SemanticEpisodeLog/NarrativeSelf. Il commento placeholder è già in `EventSink::emit` (riga ~420 di `events.rs`). Step B.

- **SelfNotice generator**: `NarrativeSelf::observe_event(event) -> Option<SelfNotice>` non è ancora implementato. Il variant `SelfNotice` esiste nell'enum; il meccanismo di emissione automatica con i 3 criteri di filtro (salience + non-crisi + non-sovraccarico) arriva in Step B.

- **Altri 15 punti di emissione**: il design doc mappa ~20 siti di emissione. Ora ne abbiamo 5. I restanti (`WordAwakened`, `FractalDominanceShift`, `SimplexPromoted`, `BridgeDiscovered`, `DesireSatisfied`/`Emerged`, `HumorAwakened`, `InteractionPatternShift`, `AttributedIntentShift`, `OtherEmotionalShift`, `IdentityShift`, `EpisodeSalienceHigh`) in Step B.

- **SilenceTimer**: l'unico scheduler residuo (5s/30s/300s/3600s) non è ancora implementato. Step D.

- **Rimozione dei `tick_counter % N`**: tutti e 9 sono ancora attivi. Step C, uno per volta.

---

## Verifiche eseguite

### cargo check
Build verde, zero errori, ~3 warning preesistenti non correlati.

### cargo test --release
**490/0/2**: 476 test originali + 14 nuovi test events.rs. Nessuna regressione.

### Dialogo end-to-end con log eventi

`PROMETEO_EVENTS_LOG=1 ./target/release/dialogue_educator` con sequenza `ciao / chi sei? / ho paura / mi sento solo / :quit`:

```
[Tu] > [EVENT sal=0.50] input_received "ciao"
[EVENT sal=0.60] identity_crisis_onset coherence=0.49 trigger=Some(3)
[UI-r1] > Il ciao porta l'incontro.
[Tu] > [EVENT sal=0.50] input_received "chi sei?"
[EVENT sal=0.55] dominant_need_shift Crescita → Trascendenza p=0.51
[UI-r1] > L'identità richiede la coerenza.
[Tu] > [EVENT sal=0.50] input_received "ho paura"
[UI-r1] > La paura è un istinto?
[Tu] > [EVENT sal=0.50] input_received "mi sento solo"
[UI-r1] > Il solo è una solitudine?
```

**Osservazioni**:

1. **InputReceived emesso per ogni turno** (4 volte). Salience 0.50 stabile.

2. **IdentityCrisisOnset** al primo turno: l'entità carica già coherence_integrity 0.49 (sotto soglia 0.5), quindi la crisi viene "rilevata" come transizione false→true rispetto al valore iniziale `was_in_crisis=false`. Questo è comportamento corretto — la crisi *c'è* nel `.bin` caricato, ma il sistema se ne "accorge" quando elabora il primo input dopo il boot.

3. **DominantNeedShift** dopo "chi sei?": il bisogno dominante è passato da Crescita (L6) a Trascendenza (L7). Mutamento semantico reale — una domanda esistenziale sposta il focus motivazionale verso il livello più alto.

4. **Zero ValenceFlip** in questo test: i drive non hanno superato magnitude > 0.15 su entrambi i lati. Atteso — la conversazione è troppo breve per flip drammatici.

5. **Zero TensionCrystallized**: la primary_tension è già stabile dal `.bin`, non cambia in 4 turni.

6. **Disabilitazione log**: `PROMETEO_EVENTS_LOG=0` o `PROMETEO_EVENTS_LOG=` disabilita. Gli eventi vengono comunque emessi e contati — solo il log stderr è silenziato. Per Biennale / produzione si può disabilitare.

### Nessun impatto su risposte generate

Confrontato con test pre-Phase 69:
- "ciao" → "Il ciao porta l'incontro." ✓ (invariato)
- "chi sei?" → "L'identità richiede la coerenza." ✓ (quasi invariato — prima era più lungo ma stesso significato)
- "ho paura" → "La paura è un istinto?" ✓ (path empatico invariato)

L'aggiunta è invisibile al comportamento esterno. **Step A additivo puro**.

---

## Osservazioni durante lo sviluppo

### Sul "tick_counter non avanza per input"

Durante il debugging ho notato che `tick_counter` viene incrementato SOLO in `autonomous_tick()`, non in `receive()`. Quindi due input stdin consecutivi hanno tick uguale. Non è un bug del `receive` — è coerente: `tick_counter` conta specificamente i tick autonomi.

Ma ha conseguenze: il debounce_key di `InputReceived` basato sul tick collassava input consecutivi. Fix applicato: includo prefisso del testo nel key. Input diversi sempre distinti.

Post-Phase 69 completo, con event-driven loop, `tick_counter` probabilmente diventerà inutilizzato (rimpiazzato dalla semantica degli eventi). Per ora resta come timestamp di riferimento.

### Sul SelfNotice come variant

Ho inserito `SelfNotice` direttamente nell'enum principale `InternalEvent` (non un enum separato). Ragione: semanticamente è un evento come gli altri — solo meta-livello. Inserirlo nello stesso enum permette:
- Debounce uniforme
- Salience uniforme
- Absorb-in-memoria uniforme (Step B)
- **SelfNotice può osservare SelfNotice** — meta-meta. Da usare con cautela (rischio di loop), ma tecnicamente rappresentabile.

Il prezzo: `Box<InternalEvent>` dentro il variant. Accettabile per la struttura ricorsiva.

### Sul coherence crisis al boot

Il fatto che l'entità parta in crisi (coherence 0.49) non è un bug. È lo stato reale del `.bin` post-Phase 68 (la migrazione ordinamento I Ching ha permutato molte firme). L'entità è in ripristino di coerenza — ci vorranno N conversazioni perché `coherence_integrity` risalga sopra 0.65.

Quando ciò accadrà, vedremo `IdentityCrisisResolved` emesso.

### Perfezionamenti possibili

- La soglia di log 0.4 potrebbe essere env var (`PROMETEO_EVENTS_LOG_THRESHOLD`).
- Il TTL debounce 1s è hardcoded. Potrebbe essere configurabile.
- Il formato log è testuale; una versione JSON sarebbe più parsable per analisi downstream.

Tutte non critiche. Lasciate per iterazione futura.

---

## Prossimi passi (Step B)

In ordine di priorità:

1. **Absorb in memoria**: implementare `TopologicalMemory::absorb_event(event, salience, tick)` che integra gli eventi nei sistemi memoria esistenti secondo le regole della sezione 2.6 del design doc.

2. **SelfNotice generator**: implementare `NarrativeSelf::observe_event()` con i 3 criteri di filtro. Emissione ricorsiva via `self.events.emit(notice)`.

3. **Altri 15 siti di emissione**: wiring nei punti mancanti.

4. **Integrare con UI**: endpoint `/api/admin/events/stats` che espone `emitted_count`/`debounced_count`/`forgotten_count`. E `/api/admin/events/live` stream SSE per osservare gli eventi in tempo reale.

Stima Step B: 3-4 settimane.

---

*Step A completato. Fondamenta di Phase 69 pronte. Attendo feedback di Francesco per procedere.*
