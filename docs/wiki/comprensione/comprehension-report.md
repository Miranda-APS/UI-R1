# Comprehension Report — il documento di "cosa ho capito"

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 73 design); Lacan
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`ComprehensionReport` (`src/topology/comprehension_report.rs`, Phase 73) è il **documento strutturato che UI-R1 "scrive" prima di rispondere**. Non un vettore di intent, non uno softmax — una struttura tipizzata, in italiano leggibile, che cattura cosa è stato detto, da quale posizione, con quali vuoti, e con quale rilevanza per l'entità. Il framing è esplicitamente **Lacaniano**: signifier_positions, signifier_gaps, l'Altro.

## Anatomia

```rust
pub struct ComprehensionReport {
    pub speech_act: SpeechAct,                       // saluto, interrogazione,
                                                     // posizionamento, denominazione, …
    pub signifier_positions: Vec<SignifierPosition>, // chi occupa quale posizione
    pub signifier_gaps: Vec<SignifierGap>,           // i vuoti aperti dall'enunciato
    pub inferences: Vec<Inference>,                  // cosa il KG aggiunge a questo input
    pub self_relevance: f64,                         // [0,1] quanto è "su di me"
    pub closes_prior_gap: Option<PriorGapClosure>,   // Phase 78
}
```

## SpeechAct

Enum non-dispatch (è etichetta, non if/then):
- `saluto` — atto fatico di apertura
- `interrogazione` — domanda
- `posizionamento` — affermazione di posizione (incluso "del buio" come continuazione di un'articolazione)
- `denominazione` — "mi chiamo X"
- `affermazione` — asserzione generica
- `pensiero` — meta-cognizione del parlante

Il `kind` di SpeechAct è una delle proprietà che `seed_from_comprehension` (Phase 79) consulta per seminare percetti nel KG procedurale.

## SignifierGap (Phase 76)

Il vuoto è una **parola atomica** + un context opzionale:

```rust
pub struct SignifierGap {
    pub missing: String,           // "oggetto" — parola singola, Phase 76
    pub from: String,              // "paura" — il trigger
    pub relation: String,          // "Requires"
    pub context: Option<String>,   // "emozione" — concetto composto
}
```

Es. da `derive_gaps()` su "ho paura":
```
{ missing: "oggetto", from: "paura", relation: "Requires", context: Some("emozione") }
```

Phase 76 ha imposto l'**atomicità**: la `missing` è sempre una parola singola (non `"oggetto-dell'emozione"`). Permette il join diretto con `cosa UsedFor chiedere via=oggetto` nel KG procedurale.

## PriorGapClosure (Phase 78)

Quando l'utterance corrente chiude un gap precedente:
```rust
pub struct PriorGapClosure {
    pub trigger: String,         // "paura"
    pub closing_word: String,    // "buio"
    pub opened_at_turn: usize,
    pub role: String,            // "emotion_object"
}
```

Se `closes_prior_gap.is_some()`:
- `speech_act.kind = "posizionamento"` (continuazione, non asserzione)
- `signifier_gaps = []` (vuoto colmato)
- `signifier_positions` con trigger PRIMA
- `self_relevance` esplicita "il parlante ha colmato il vuoto che avevo aperto al turno N"

Il report **stesso** riflette che questo enunciato è continuazione — l'azione che ne deriva è meccanica.

## derive_gaps()

`derive_gaps(claim, kg)` produce i gap labelati come parole singole. Oggi controlla solo `Requires` nel KG semantico. TODO architetturale: dovrebbe anche consultare KG procedurale per gap discorsivi (es. "Question senza pronome interrogativo").

## Inferences

Dopo aver costruito il report, l'attivazione si propaga nel campo. `comprehension_graph.rs` (Phase 73) prende il report e:
1. Attiva le `signifier_positions` come hub PF1
2. Semina gap come ricerche IS_A 1-hop
3. Le `inferences` sono i nodi più vivi della propagazione

## Self relevance

Score [0,1] che indica quanto l'enunciato parla dell'entità. Es:
- "tu sai X" → high
- "come stai?" → high (self-referenced via 2sg)
- "il sole è caldo" → low

`facts.self_referenced` in `engine.rs` è oggi limitato a "tu"/"ti" o claim Entity. Phase 79 ha aggirato in `kg_proc_field::utterance_has_second_singular` per catturare anche "come stai?" / "cosa pensi?" senza che `facts.self_referenced` fosse stato attivato a monte. TODO: propagare il segnale 2sg a monte in `input_reading`.

## See Also

- [Pipeline di comprensione](pipeline-comprensione.md)
- [Speaker profile](speaker-profile.md) — fornisce SpeakerClaim e gap context
- [Action reasoning](action-reasoning.md) — il consumer del report
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md) — riceve i percetti dal report
- [Capire prima, generare dopo](../principi/capire-prima-generare-dopo.md)
