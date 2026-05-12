# Bisogni, desideri, volontà

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 53+64); Maslow, Chou
> Raw: [09_bisogni_desideri](../../raw/libretto/09_bisogni_desideri.md); [10_volonta](../../raw/libretto/10_volonta.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

Tre canali motivazionali distinti contribuiscono alla deliberazione di UI-R1: **bisogni** (Maslow gerarchico), **desideri** (orientamenti dinamici verso firme target), **volontà** (intenzione corrente filtrata). Tutti modulano la stance e l'intenzione, ma sono **fonti di pressione**, non decisori — la decisione finale è di [NarrativeSelf.deliberate](narrative-self.md).

## Bisogni (Phase 53)

`NeedsHierarchy` (`src/topology/needs.rs`) — 7 livelli alla Maslow:

| L | Bisogno | Esempi |
|---|---------|--------|
| 1 | Sopravvivenza | tensione vitale, energia |
| 2 | Sicurezza | confine, stabilità |
| 3 | Connessione | risonanza con l'Altro |
| 4 | Espressione | freschezza del campo |
| 5 | Comprensione | gap aperti, curiosità epistemica |
| 6 | Significato | coerenza identitaria |
| 7 | Trascendenza | apertura beyond-self |

`sense(vital, identity, self_model, field_metrics)` produce `NeedsState`. `compute_pressure()` → `NeedsPressure` con moltiplicatori will (es. L5 distress → Question ×0.8 + Reflect ×0.3, riduce Instruct).

**Prepotency gate**: L1-L2 insoddisfatti sopprimono livelli alti. Senza sopravvivenza, niente trascendenza.

## Desideri (Phase 53+64)

`DesireCore` (`src/topology/desire.rs`) — max 5 desideri attivi, decay 0.995/tick.

Phase 64 ha introdotto `DesireSource::OctalysisDriven(cd, val)`: il desiderio nasce dall'incrocio tra `last_comprehension` (cosa il KG ha capito) e il drive Octalysis dominante (|drives[cd]| > 0.28). Non "voglio esprimere" (circolare), ma "data comprensione X e drive CD5 attivo, voglio connettere in quella direzione".

Firma bersaglio = `field_sig + 0.35 nella dimensione del drive + 0.12 dal peso semantico della comprensione`.

`will_biases() → Vec<(usize, f64)>` produce compound_bias. Soglia bias > 0.001 (non 0.01). `check_satisfaction()`: cosine_distance < 0.2 per 3 tick → soddisfatto.

## Volontà (Phase 67)

`FieldPressures` struct in `will.rs` — pressioni grezze del campo senza selezione dominante. 7 campi f64 (express, explore, question, remember, withdraw, reflect, instruct) + codon + is_dreaming.

`compute_pressures()` calcola, `to_will_result()` converte per backward compat. `sense()` è wrapper. **NarrativeSelf è l'unico decisore** — il will fornisce pressioni, non decisioni.

`Intention` (in `will.rs`): Withdraw / Explore / Instruct / Express / Question / Remember / Reflect. Emerge da pressioni campo + needs/desire/coherence.

## Soglia espressione spontanea dinamica (Phase 54)

In `autonomous_tick()`, la soglia per `will.drive` parte da 0.6 e scende fino a 0.35 in base a `needs.dominant_pressure` (>0.5) e `desire.intensity` (>0.6). Bisogni e desideri forti rendono UI-R1 più espressiva — ma non oltrepassano la decisione narrative.

## Integrazione in deliberate()

`deliberate()` ha 12 parametri (Phase 67). Ultimi rilevanti:
- `inner: Option<&InnerState>` — needs, desires, interlocutor pattern/presence/resonance, humor, attributed_intent, coherence_integrity
- `field_pressures: Option<&FieldPressures>` — pressioni campo

Vedi [narrative self](narrative-self.md) per il flusso completo.

## L'input è sovrano (Phase 55)

In `deliberate()`, i bisogni/desideri/humor colorano l'intenzione SOLO quando `input_is_ambiguous` (Acknowledge). Soglia Need alzata a 0.95. La stance non viene forzata dai bisogni — unico override: Withdrawn → Open per connessione/espressione forte.

L'input concreto del parlante non viene sopraffatto da pressioni interne.

## See Also

- [Valenza Octalysis](valenza-octalysis.md) — i 8 drive che alimentano i desideri
- [Narrative self](narrative-self.md) — il decisore
- [Interlocutor model](interlocutor-model.md) — l'Altro che modula
- [Pipeline di comprensione](../comprensione/pipeline-comprensione.md)
