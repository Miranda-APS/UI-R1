# Interlocutor Model — eco dell'Altro

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 53+55+62)
> Raw: [11_interlocutor_humor](../../raw/libretto/11_interlocutor_humor.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`InterlocutorModel` (`src/topology/interlocutor.rs`, Phase 53) è l'**eco dell'Altro** — il modello che UI-R1 si costruisce del parlante turno per turno, senza simulare empatia ma percependo la valenza emotiva, l'intento attribuito, il pattern di interazione. Sostituisce concettualmente il vecchio `DualField` (rimosso in Phase 53).

## Anatomia

```rust
pub struct InterlocutorModel {
    pub presence: f64,                  // [0,1] con decay
    pub cumulative_resonance: f64,
    pub cumulative_novelty: f64,
    pub novelty_ema: f64,               // EMA α=0.3
    pub history: VecDeque<InterlocutorTurn>,
    pub pattern: InterlocutorPattern,   // Converging/Diverging/Oscillating
    pub attributed_intent: AttributedIntent, // Phase 55
    pub emotional_valence: f64,         // [-1,+1], Phase 62, EMA α=0.4
}
```

## register_input (pre/post 8D)

`register_input(pre_sig, post_sig)` prende firma 8D pre/post propagazione. Calcola:
- **Risonanza**: cosine_similarity(pre_sig, post_sig) — quanto il campo già conteneva l'input
- **Novità**: 1 − risonanza
- EMA α=0.3 per risonanza, α=0.3 per novità
- `cumulative_novelty` inizia a 0.5 (default neutro)

## AttributedIntent (Phase 55)

Enum: Unknown / Seeking / Teaching / Challenging / Connecting / Withdrawing.

Matrice risonanza × novità (soglia 0.45):
- alta novità + alta risonanza → Connecting
- alta novità + bassa risonanza → Teaching
- bassa novità + alta risonanza → Seeking
- bassa novità + bassa risonanza → Withdrawing

`tick_decay()`: presence < 0.15 + history → Withdrawing.

**Reciprocity modulazione**: in `NarrativeSelf.deliberate()`:
- Teaching → Explore
- Challenging → Reflect

Solo se `input_is_ambiguous` (Acknowledge). L'input concreto non viene sopraffatto.

## emotional_valence (Phase 62)

Campo `f64 ∈ [-1, +1]` aggiornato via EMA α=0.4 ad ogni `register_input()`. Negativo = distress (tristezza/paura/dolore), positivo = gioia.

`compute_other_emotional_valence()` in `engine.rs` usa IS_A 1-hop per riconoscere parole emotive **senza liste hardcoded**:
- Radici negative: tristezza/dolore/paura/rabbia + aggettivi (triste/spaventato/…)
- Radici positive: gioia/felicità/… + aggettivi

**Parole negate escluse**: "non ho paura" → ev = 0.0 (Phase 61: `field_boosts skip per parole negate`).

Decade naturalmente a ogni input neutro (×0.6 per turno).

## Effetti del distress sull'entità

Phase 62 — quando `other_emotional_valence < -0.35`:
- `will_biases()` aggiunge: Question ×0.60, Reflect ×0.20, riduce Instruct ×-0.50, Express ×-0.20
- L5 Connessione: pressione Question amplificata (×0.8) + Reflect (×0.3)
- P4 Resonate handler empatico: risposta in seconda persona interrogativa. "io sono triste" → "Senti il pianto?" invece di "Sento il pianto."
- `expression::compose()` con `other_in_distress`: forza `voice.person = Second + mood = Interrogative`

CD5 Relazione diventa negativo: l'entità percepisce lo stato altrui sul proprio campo, non finge di provarlo. Vedi [niente empatia simulata](../principi/niente-empatia-simulata.md).

## InterlocutorPattern

Stato qualitativo della conversazione:
- **Converging**: risonanza cresce, l'Altro entra in sintonia
- **Diverging**: risonanza cala, l'Altro si allontana
- **Oscillating**: alternanza
- **Stable**: equilibrio

Usato per coloring narrative, non per dispatch.

## apply_identity_drift

Phase 53: richiede `cumulative_resonance > 0.7` E `presence > 0.3` E `history.len() >= 3`. Modifica la firma identitaria nel tempo verso il pattern dell'Altro — ma con grande inerzia, mai sopra il commitment volitivo.

## Persistenza Phase 54

`InterlocutorSnapshot` ha `#[serde(default)]` per `emotional_valence` (backward compat). Il modello persiste sessione → sessione: quando l'utente torna, UI-R1 ricorda lo stato dell'Altro accumulato. Questo è semanticamente corretto (memoria dell'Altro), ma può causare carryover se la sessione precedente era pesante.

## See Also

- [Narrative self](narrative-self.md) — consuma attributed_intent e valence
- [Valenza Octalysis](valenza-octalysis.md) — CD5 Relation è alimentato qui
- [Niente empatia simulata](../principi/niente-empatia-simulata.md)
- [Pipeline di comprensione](../comprensione/pipeline-comprensione.md)
