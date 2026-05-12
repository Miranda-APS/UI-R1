# Narrative Self — il decisore deliberativo

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79; Phase 47-67 design)
> Raw: [07_identita](../../raw/libretto/07_identita.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`NarrativeSelf` (`src/topology/narrative.rs`) è il **decisore deliberativo** centrale di UI-R1. Riceve stato vitale, valenza, needs, desires, pressure di volontà, coherence_integrity, attributed_intent dell'Altro, e produce `InternalStance` + `ResponseIntention`. È l'unico componente che decide *cosa fare*; tutti gli altri forniscono pressioni o percezioni.

## Anatomia

```rust
pub struct NarrativeSelf {
    pub identity_core: IdentityCore,
    pub turns: VecDeque<NarrativeTurn>,
    pub pending_intention: ResponseIntention,
    pub pending_stance: InternalStance,
    pub commitment: Option<Commitment>,         // Phase 55
    pub self_witness: SelfWitness,              // Phase 66
    pub valence_history: VecDeque<Valence>,
    pub coherence_integrity: f64,               // [0,1] (Phase 55, 78)
}
```

## deliberate() — 12 parametri

Phase 67 firma:
```rust
fn deliberate(
    &mut self,
    valence: &Valence,
    will_pressure: &FieldPressures,
    fractal_attractor: Option<&FractalSnapshot>,
    needs: &NeedsState,
    desire: &DesireCore,
    self_model: &SelfModel,
    field_metrics: &FieldMetrics,
    inner: Option<&InnerState>,                 // attributed_intent, humor, ecc.
    input_reading: &InputReading,
    input_is_ambiguous: bool,
    field_pressures: Option<&FieldPressures>,
)
```

## Stance derivation (Phase 55)

`InternalStance` deriva dalla valenza tramite `form_stance_from_valence(drives)`:
- Withdrawn (`max_intensity < 0.1`)
- Curious (CD3 Creativity dominante)
- Open (CD5 Relation positivo)
- Reflective (CD8 Avoidance dominante)
- Resonate (CD5 positivo + L5 connessione attiva)
- Bold (CD7 Unpredictability dominante)
- Anchored (CD4 Ownership dominante)

`stance_from_valence` usa `dom_val.abs() < 0.15` (dominant drive, non average intensity) come soglia per "valenza debole" → Withdrawn.

**Override vulnerability** (Phase 55): se `coherence_integrity < 0.5` → forza Reflective. L'entità in crisi si fa rumore.

**Reciprocity modulazione** (Phase 55): `attributed_intent` dell'Altro modula la deliberazione SE `input_is_ambiguous`:
- Teaching → Explore
- Challenging → Reflect

## ResponseIntention

Enum: Resonate / Express / Explore / Question / Reflect / Remain / Need / Irony / Desire (Phase 54 ha aggiunto gli ultimi 3).

Serializzati come "risuonare" / "esprimere" / "esplorare" / "domandare" / "riflettere" / "restare" / "cercare" / "incongruenza" / "desiderare" (`as_str()` / `intention_from_str()`).

## Commitment (Phase 55)

`Commitment { strength: 0.3, decay: 0.02/tick, min: 0.05 }`. Inerzia = `strength × ln(turns_held + 1)`. Rompere il commitment costa CD4 -0.05.

- Override vitale (Remain) e bisogno estremo (Need) dissolvono l'impegno.
- Withdrawn → Remain forza dissoluzione (`stance_from_valence = Withdrawn` → `intention = Remain` → `commitment = None`).

Persistito in `NarrativeSnapshot` ma restore_into() lo imposta a None (Phase 56 fix: ogni sessione inizia senza inerzia accumulata).

## Coherence pull (Phase 64)

`coherence_score()` misura cosine similarity tra frattali proposti e traiettoria frattale degli ultimi 4 turni → [0, 1]. In `engine.rs receive()`: se coherence < 0.30 con ≥3 turni di storia → pull soft verso `recent_fractal_attractor(3)` (0.08× strength). La narrativa orienta senza vincolare.

`recent_fractal_attractor(n)` — media normalizzata dei frattali dominanti degli ultimi N turni. Top-5 per forza media.

## NarrativeTurn.inner_state_summary (Phase 54)

Ogni turno cattura lo stato motivazionale al momento:
```
"bisogno: connessione (78%) | desiderio: comprendere-paura (62%) |
 Altro: pattern=Converging (resonance 0.65) | incongruità: 23%"
```

Visibile nel tab Narrativa della web UI.

## Pipeline post-deliberazione

Dopo `deliberate()`:
1. `generate_willed_inner()` legge `pending_intention` e `commitment`
2. Codon da `last_field_pressures`
3. Withdraw check usa `narrative_self.pending_intention == Remain`
4. Pattern matcher istanzia la voce
5. Self-listening: re-iniettare parole a 0.3× per riconoscere il proprio sé

## See Also

- [Valenza Octalysis](valenza-octalysis.md)
- [Bisogni desideri volontà](bisogni-desideri-volonta.md)
- [Interlocutor model](interlocutor-model.md)
- [Self witness](self-witness.md) — la memoria autonoma dell'entità
- [Action reasoning](../comprensione/action-reasoning.md) — il decisore Phase 71-79 specifico per la voce
