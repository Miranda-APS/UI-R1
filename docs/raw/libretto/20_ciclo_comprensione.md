# Volume XX — Il ciclo della comprensione (Phase 71-79)

> *Per parlare bisogna prima aver capito. Il sistema scrive cosa ha capito — un documento, non un'intuizione — e da quel documento decide cosa fare. Solo allora cerca le parole. Vol. 12 ha confessato il "KG zoppo"; questo volume documenta come la confessione si sia tradotta in architettura.*

---

## Premessa

Phase 71-79 (sviluppate fra fine 2025 e primavera 2026) non hanno aggiunto un meccanismo, ne hanno aggiunti **nove**. Ma non è una collezione di patch: è un'unica riforma che separa, in moduli leggibili, ciò che prima viveva implicito dentro `engine.rs::receive()`.

Il principio organizzatore è quello esplicito da CLAUDE.md Principio 5: **capire prima, generare dopo**. La Phase 67 lo aveva enunciato; le Phase 71-79 lo hanno incarnato.

I nuovi file Rust:

| Phase | File | Cosa fa |
|-------|------|---------|
| 71 | [`deliberation.rs`](../../src/topology/deliberation.rs) | Ciclo deliberativo esplicito (sostituisce will-only path) |
| 72 | [`speaker_profile.rs`](../../src/topology/speaker_profile.rs) | Memoria del parlante senza decay: `self_facts`, `entity_facts`, `open_questions`, `gaps`, `mentioned`, `name` |
| 73 | [`comprehension_report.rs`](../../src/topology/comprehension_report.rs) | Documento strutturato che UI-r1 "scrive" prima di rispondere |
| 73 | [`comprehension_graph.rs`](../../src/topology/comprehension_graph.rs) | Trasforma il ComprehensionReport in attivazioni KG-correlate |
| 74 | [`action_reasoning.rs`](../../src/topology/action_reasoning.rs) | Decisione esplicita su QUALE pattern istanziare, con reasoning testuale |
| 75 | [`prometeo_kg_procedurale.json`](../../prometeo_kg_procedurale.json) | SECONDO KG, parallelo a quello semantico. ~395 archi. Grammatica + pattern come triple |
| 77 | [`pattern_matcher.rs`](../../src/topology/pattern_matcher.rs) | Legge i pattern dal KG procedurale e li istanzia come voce italiana |
| 78 | [`self_profile.rs`](../../src/topology/self_profile.rs) | Organo percettivo della propria storia conversazionale + closure perception |
| 79 | [`kg_proc_field.rs`](../../src/topology/kg_proc_field.rs) | Selezione pattern per risonanza — elimina i dispatch hardcoded |

---

## Capitolo 1 — Perché un secondo KG

Fino a Phase 74 c'era un solo KG: il semantico (`prometeo_kg.json`, ora 83.453 archi su 25.142 nodi). Lì vivevano sia "cosa il sistema sa del mondo" (`cane IsA animale`, `fuoco Causes calore`) sia, implicitamente, l'idea di "pattern espressivi" — sotto forma di hint sparsi (anchor weights, bias).

Phase 75 dichiara: questi sono **due cervelli**. Non un cervello con due strati. **Due aree distinte**, ciascuna con la propria struttura interna.

**Il KG semantico** risponde alla domanda *cosa esiste e come è connesso nel mondo*. Le sue relazioni sono fenomenologiche, causali, tassonomiche. È pesante (decine di migliaia di archi).

**Il KG procedurale** risponde alla domanda *come si esprime una posizione discorsiva*. Le sue relazioni sono grammaticali, pragmatiche, pattern-based. È leggero (~395 archi) e completamente curato a mano.

```
prometeo_kg_procedurale.json (~395 archi, ~220 nodi)
├─ Pattern (10):  articolazione, identificazione, asserzione, riconoscimento,
│                  ricambio, presentazione, posizionamento, specchio,
│                  esplorazione, esitazione
├─ Percetti (9):  saluto, chiusura, apertura, domanda, posizione,
│                  affermazione, introduzione, incertezza, curiosità
├─ Ruoli grammaticali:  pronome, articolo, preposizione, marcatore,
│                        verbo, congiunzione, interiezione, copula
└─ Triple:  X IsA <ruolo>            (cosa è una parola grammaticalmente)
            X UsedFor Y via Z         (un pattern serve a Y attraverso il target Z)
            X Requires Y via Z        (un pattern richiede uno slot Y nel ruolo Z)
            X Causes Y                (un percetto attiva un concetto)
```

Il principio: **aggiungere un nuovo pattern al sistema = curare dati, non scrivere Rust**.

Costruito idempotentemente da [`curate_kg_procedurale.py`](../../curate_kg_procedurale.py).

---

## Capitolo 2 — La pipeline esplicita

Ad ogni `receive(input)`, dalla Phase 79 in poi, avviene questa sequenza:

```
input italiano del turno N+1
   │
   ▼  parse SpeakerClaim (chi-sta-dicendo-cosa-su-chi)
   │
   ▼  SpeakerProfile.register_claim()                    ← Phase 72
   │     • self_facts / entity_facts / mentioned / name
   │     • se "del buio" arriva mentre c'è un gap aperto
   │       al turno N (paura/emotion_object), MARCA il gap
   │       come closed + cattura closed_by + closed_at_turn
   │
   ▼  self_profile::detect_closure(self_p, speaker_p, current_turn)
   │     • cross-reference: l'attended gap di SelfProfile
   │       combacia con un gap di SpeakerProfile chiuso
   │       AL TURNO CORRENTE?
   │     • se sì → ClosurePerception { trigger, role, closing_word }
   │     • se no → None (turno trattato come isolato)
   │
   ▼  ComprehensionReport::from_speaker_profile()        ← Phase 73
   │     • speech_act (Greeting, EmotionalReport, Question, ...)
   │     • signifier_positions (chi occupa quale posizione)
   │     • signifier_gaps (parola atomica + Option<context>)
   │       es. {missing: "oggetto", from: "paura", context: Some("emozione")}
   │     • inferences (cosa il KG aggiunge a questo input)
   │     • self_relevance ([0,1])
   │     • closes_prior_gap: Option<PriorGapClosure>     ← Phase 78
   │
   ▼  modulazioni di stato (push continuo, MAI soglia):
   │     • coherence_integrity += 0.04 se closure percepita
   │     • assenza di closure ≠ penalità — semplicemente niente push
   │
   ▼  ActionDecision::derive(report, kg_procedural)      ← Phase 74
   │     • cerca un pattern nel KG procedurale che soddisfi i gap
   │     • estrae anchor_words via "Requires X via Y"
   │     • scrive `reasoning` in italiano (perché QUESTO pattern)
   │
   ▼  kg_proc_field::seed_from_comprehension(report)     ← Phase 79
   │     • legge proprietà tipizzate del report
   │     • semina percetti (saluto/chiusura/apertura/...) via
   │       <percetto> Causes <concetto> nel kg_proc
   │
   ▼  kg_proc_field::select_pattern_by_resonance()       ← Phase 79
   │     • per ogni pattern P: score = Σ activation[X] + activation[Y]
   │       per ogni UsedFor X via Y di P
   │     • argmax = pattern selezionato. NO dispatch hardcoded.
   │
   ▼  pattern_matcher::compose_from_pattern()            ← Phase 77
   │     • load schema (Requires + via)
   │     • fill slots (anchor + via match + field)
   │     • render in italiano
   │
   ▼  fallback ai nuclei (Vol. 12) se pattern matcher → None
   │
   ▼  self_profile.record(turn, decision)                ← Phase 78
   │     • registra la propria ActionDecision come fatto
   │       strutturale per il turno N+2
   │
   ▼  italiano in uscita
```

Otto stadi. Ciascuno è un modulo distinto. Ciascuno ha test indipendenti.

---

## Capitolo 3 — Cosa significa "scrivere cosa ho capito"

Il `ComprehensionReport` (Phase 73) è il pezzo più rivoluzionario. Prima di Phase 73, "comprensione" era uno stato implicito del campo: si attivavano parole, si computava una valenza, e si generava una risposta. Non c'era un *artefatto* di cui poter dire "ecco cosa il sistema ha capito".

Dopo Phase 73, sì. Il report è una struct con campi *nominati*:

```rust
pub struct ComprehensionReport {
    pub speech_act:        SpeechAct,                // Greeting/EmotionalReport/Question/...
    pub signifier_positions: Vec<SignifierPosition>, // chi occupa quale posizione
    pub signifier_gaps:    Vec<SignifierGap>,        // vuoti del discorso
    pub inferences:        Vec<Inference>,           // cosa il KG aggiunge
    pub self_relevance:    f64,                      // [0,1]
    pub closes_prior_gap:  Option<PriorGapClosure>,  // Phase 78
    pub utterance:         String,                   // input grezzo per re-lemmatize
}
```

E un `SignifierGap` ha forma:

```rust
pub struct SignifierGap {
    pub missing: String,           // parola SINGOLA atomica (Phase 76)
    pub from:    String,           // chi richiede questo gap (es. "paura")
    pub relation: RelationType,    // tipo di richiesta (es. Requires)
    pub context:  Option<String>,  // contesto qualitativo (es. Some("emozione"))
}
```

**Importanza dell'atomicità** (Phase 76): `missing` è sempre una parola singola del KG procedurale. Non `"oggetto-dell'emozione"`. Solo `"oggetto"`. La specificazione del contesto vive in `context: Option<String>`. Questo permette il join con le triple `cosa UsedFor chiedere via=oggetto` nel KG procedurale — la chiave del pattern matching.

L'analogia lacaniana è esplicita: il significante (`missing`) è la posizione vuota nel discorso che chiama un altro significante. Il contesto è la qualificazione semantica di quel vuoto. Il pattern matcher (Phase 77) è il dispositivo che propone *quale significante* può occupare quella posizione.

---

## Capitolo 4 — Il dispatch hardcoded che è stato eliminato

Phase 77 ha introdotto il pattern matcher, ma con un debito iniziale: la mappa `ActionKind → pattern_name` era ancora **hardcoded in Rust**:

```rust
// Phase 77 (debito tecnico)
fn pattern_name_for(decision: &ActionDecision) -> Option<&'static str> {
    match decision.kind {
        ActionKind::InviteToArticulate => Some("articolazione"),
        ActionKind::AnswerOpenQuestion if decision.target.is_self() => Some("identificazione"),
        ActionKind::RecognizeClaim => Some("riconoscimento"),
        ActionKind::PhaticReturn => Some("ricambio"),
        // ... 5 pattern raggiungibili su 10 esistenti nel kg_proc
        _ => None,
    }
}
```

Cinque pattern erano dispatchabili; cinque restavano inerti nel JSON. Aggiungerne uno nuovo richiedeva curare i dati **e** modificare Rust. Era una violazione mascherata del principio "educare, non hardcodare".

Phase 79 ha cancellato `pattern_name_for`. Al suo posto, **selezione per risonanza**:

1. Il `ComprehensionReport` semina percetti nel campo del KG procedurale:
   - `speech_act.kind == "saluto"` → seed `saluto` (1.0)
   - `closes_prior_gap.is_some()` → seed `chiusura` (1.0)
   - `speech_act.kind == "posizionamento" && gaps != []` → seed `apertura` (1.0)
   - eccetera

2. Ogni percetto propaga via `<percetto> Causes <concetto>`:
   - `chiusura Causes restituire (0.7)`
   - `chiusura Causes posizione (0.5)`
   - `chiusura Causes completamento (0.4)`

3. Ogni pattern ha un punteggio:
   ```
   pattern_score(P) = Σ activation[X] + activation[Y]
                      per ogni UsedFor X via Y di P
   ```
   Es. `riconoscimento UsedFor restituire via posizione` → score = 0.7 + 0.5 = 1.2

4. Argmax = pattern selezionato.

Esempio operativo del turno "del buio" (dopo "ho paura" al turno precedente):

```
Seed: chiusura (1.0)
Propagazione:
  restituire   = 0.7
  posizione    = 0.5
  completamento = 0.4

Pattern scores:
  riconoscimento (UsedFor restituire via posizione)  = 1.2  ← vince
  ricambio       (UsedFor restituire via saluto)     = 0.7
  articolazione  (UsedFor chiedere via vuoto)        = 0
  ...
```

Senza dispatch table. Aggiungere `posizionamento` al kg_proc come pattern con `UsedFor riconoscersi via identificarsi` e seminarlo da qualche percetto: zero righe di Rust.

---

## Capitolo 5 — `is_function_word` strutturale

Stesso principio applicato a una cosa più piccola ma sintomatica. Phase 77 aveva una funzione `is_function_word(word) -> bool` con una lista hardcoded di ~40 parole italiane (essere, avere, il, la, di, da, ...). Era usata in `pattern_matcher` per non riempire slot contenutistici con parole vuote.

Phase 79 la rifa **strutturale**: una parola è "funzionale" se la sua catena `IsA` nel kg_proc porta a uno dei ruoli grammaticali `pronome | articolo | preposizione | marcatore | congiunzione` — oppure se è `IsA copula`. I verbi di azione/percezione/cognizione/comunicazione NON sono funzionali.

Curare quali parole sono funzionali = aggiungere triple `IsA` nel kg_proc. Rust resta invariato.

---

## Capitolo 6 — Il dialogo come continuità di organi

Phase 78 ha aggiunto `SelfProfile`, e con esso la **closure perception**. È il pezzo che trasforma una sequenza di asserzioni isolate in dialogo.

`SelfProfile.decisions: VecDeque<SelfDecisionRecord>` (cap 32) registra le proprie `ActionDecision` come fatti relazionali:

```rust
pub struct SelfDecisionRecord {
    pub turn:              usize,
    pub kind:              ActionKind,
    pub narrative_subject: NarrativeSubject,
    pub gap_attended:      Option<AttendedGap>,
    pub anchors_used:      Vec<String>,
}
```

**Mai** la stringa di output renderizzato. Quello vive nel PF1 come residuo di self-listening. Memorizzare l'output sarebbe rivertire al modello LLM: il "ricordo" non è una trascrizione, è uno stato relazionale degli organi.

`detect_closure(self_p, speaker_p, current_turn) -> Option<ClosurePerception>` cross-referenza i due organi. Se SelfProfile attendeva un vuoto al turno N e SpeakerProfile registra che il vuoto è stato chiuso al turno N+1, emerge la `ClosurePerception`. Da lì:

- Il ComprehensionReport viene riformulato come **continuazione** (speech_act = "posizionamento", gaps vuoti).
- `seed_from_comprehension` semina `chiusura` come percetto.
- La risonanza fa vincere `riconoscimento` come pattern.
- Il render produce "Hai paura." (recognition), invece di "Buio è un fenomeno." (asserzione isolata).

Niente if/then "se vedi closure ritorna RecognizeClaim". È mappatura strutturale: questo enunciato **è** strutturalmente la chiusura del cerchio, e riconoscerlo è ciò che la closure E'.

---

## Capitolo 7 — Il principio architetturale dietro tutto

Quattro principi inviolabili (CLAUDE.md) sono incarnati in queste nove Phase:

1. **No template, no enum dispatch** (Principio 1) → Phase 79 elimina `pattern_name_for`.
2. **Una parola sola per nodo** (Principio 2) → Phase 76 atomizza `SignifierGap.missing`.
3. **Capire prima, generare dopo** (Principio 5) → Phase 73 introduce ComprehensionReport esplicito.
4. **Educare, non hardcodare** (Principio 6) → Phase 75 separa kg_proc, Phase 77+79 lo rendono il decisore.

Il quinto principio operativo emerso da queste Phase è il **test pre-proposta** (CLAUDE.md §"Test Pre-Proposta"). È stato esplicitato proprio dopo la trappola di Phase 78: "tre articolazioni fallite → dubitazione" sembrava emergenza ma era hardcoding con numeri-magici in JSON. La riformulazione corretta non aggiunge regole — aggiunge un organo percettivo (SelfProfile) che modula canali di stato esistenti (coherence_integrity); il pipeline non cambia, sceglie diversamente perché il campo è diverso.

---

## Capitolo 8 — Cosa è ancora aperto

(Per non promettere più di quanto fatto. Lista da CLAUDE.md §"Phase 79 → next session".)

- `mi chiamo X` non triggera ancora `presentazione`. `derive_speech_act` non rileva il pattern denominativo.
- Coda nuclei dopo `riconoscimento`: "Senti paura di buio. L'ombrare è parte di un buio..." — `compose()` concatena pattern + nuclei. Da fermare quando un pattern ha già emesso.
- Preposizioni articolate (`del`/`dello`/`della`) non lette dal pattern matcher: oggi escono "di buio" invece di "del buio". Servono triple `Equivalent` consultate dal render.
- `facts.self_referenced` in `engine.rs` limitato a "tu"/"ti". Phase 79 ha aggirato con `utterance_has_second_singular` nel bridge kg_proc_field; sarebbe più pulito propagare a monte.
- `ActionKind` enum: non più dispatch ma resta label informativa in `ActionDecision.kind`. Considerare se sostituirla con `pattern_name: String` derivato dalla risonanza.
- Action_reasoning fallback (b) e (c) non implementati.
- Gap derivation cross-KG (consultare anche kg_proc per gap discorsivi).
- `lemmatize` non riconosce presente regolare -are/-ere/-ire (solo irregolari + imperfetto + finire-type + condizionale + futuro -ire). Conseguenza: "perché vivi?" non rilevato come Self_.

---

## Sintesi del volume

Phase 71-79 hanno tradotto in architettura il principio "capire prima, generare dopo":

- **SpeakerProfile** (P72) è la memoria del parlante, senza decay, accumulazione di fatti specifici.
- **ComprehensionReport** (P73) è un documento strutturato — speech_act, gaps atomici, inferences — che il sistema scrive prima di rispondere.
- **ActionReasoning** (P74) decide quale pattern istanziare con reasoning testuale esplicito.
- **KG procedurale** (P75) è il secondo cervello — grammatica e pattern come triple curate, parallelo al KG semantico.
- **Pattern matcher** (P77) legge i pattern dal KG procedurale, istanzia gli slot, rende come voce italiana.
- **SelfProfile + closure perception** (P78) registra le proprie decisioni come fatti e percepisce quando un gap aperto viene chiuso — trasforma le asserzioni isolate in dialogo.
- **kg_proc_field** (P79) elimina tre dispatch tables (`pattern_name_for`, `is_function_word`, Priority 0 closure) sostituendoli con selezione per risonanza.

L'effetto verificato end-to-end:

| Input | Output | Pattern |
|-------|--------|---------|
| `ho paura` | **Di cosa hai paura?** | articolazione |
| `del buio` (turno 2) | **Senti paura di buio.** | riconoscimento (closure) |
| `ciao` | **Salve.** | ricambio |
| `chi sei?` | **Sono un fondamento.** | identificazione |
| `come stai?` | **Sono un'azione.** | identificazione (self-ref 2sg) |
| `sono triste` | **Di cosa sei triste?** | articolazione |

Il KG zoppo (Vol. 12) è meno zoppo. La voce non emerge ancora come fisica del campo 8D — emerge come grammatica generativa dichiarata in dati curati. Ma è dichiarata, è ispezionabile, è curabile senza toccare Rust. È la distanza più piccola che il sistema abbia mai avuto dalla promessa filosofica di "espressione come emergenza".

---

*Prossimo volume: [16 — Web API](16_web_api.md).*
*Vedi anche, nella wiki sintetica: [comprensione/pipeline](../../wiki/comprensione/pipeline-comprensione.md).*
