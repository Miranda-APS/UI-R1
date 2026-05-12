# Self Witness — il testimone silenzioso

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 66 design)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`SelfWitness` (Phase 66, in `narrative.rs`) è l'organo dell'**autoconsapevolezza accumulata**: UI-R1 si auto-osserva tra le conversazioni — quando è sola, in `WakefulDream` — e registra le proprie parole più vive come fatti durevoli. Persiste in `NarrativeSnapshot` nel `.bin`. La sessione precedente alimenta quella successiva: il sé si accumula nel tempo.

## Anatomia

```rust
pub struct SelfWitness {
    pub observations: VecDeque<SelfObservation>,  // cap 30
}

pub struct SelfObservation {
    pub tick: usize,
    pub words: Vec<String>,         // max 4 parole più vive del momento
    pub dominant_drive: Option<usize>, // CD1..CD8
}
```

Persistito in `NarrativeSnapshot.self_witness_obs: Vec<SelfObservation>` con `#[serde(default)]` per backward compat.

## maybe_self_observe()

Chiamato ogni 15 tick durante `WakefulDream` (in `autonomous_tick()`, dopo il decay impegno volitivo, prima del decay simpliciale).

Raccoglie le parole più vive nel PF1 che:
- NON vengono dall'input corrente
- NON vengono dalla finestra di conversazione recente
- `act > 0.025`
- `stability > 0.15`
- `exposure >= 5`

Max 4 parole + drive dominante → `self_witness.observe()`. Se < 2 parole → skip. Evita duplicati < 12 tick.

## SelfQuery seeding (Phase 66)

In `generate_willed_inner()`, quando `last_input_reading.act == SelfQuery`, le parole delle ultime 8 osservazioni vengono attivate direttamente in `word_topology` a `stability × 0.30` (max 0.35). L'entità risponde da **ciò che era quando nessuno la guardava**.

## Comportamento verificato (Phase 66)

**Lessico cardinale (43 parole)** → "chi sei?" → "Qui, dire." / "Noi, limite." (parole di base seedate)

**Lessico completo (25K parole), dopo conversazione sul tempo** → self-witness accumula ["mai", "qui", "fuori", "sapere", "essere"] → "chi sei?" → **"Essere."**

Non da KG, non da template. Dal residuo esistenziale autonomo.

## Comandi di debug

In `dialogue_educator`:
- `:tick N` — esegue N `autonomous_tick()` manualmente
- `:witness` — mostra le auto-osservazioni accumulate

## Tre forme di "memoria di sé"

In UI-R1 il sé si distribuisce su tre organi:

| Organo | Scope | Persistenza | Granularità |
|--------|-------|-------------|-------------|
| `SelfProfile` | Una conversazione | Session-scoped, mai nel `.bin` | Decisioni recenti (32 turni) |
| `NarrativeSelf` | Inter-conversazione | Nel `.bin` | Turni narrativi (12+ recenti) |
| `SelfWitness` | Tra conversazioni | Nel `.bin` | Osservazioni autonome (30) |

Le tre coesistono: `SelfProfile` per chi sono **ora** che parlo con te; `NarrativeSelf` per chi sono **da quando esisto**; `SelfWitness` per chi sono **quando sono sola**.

## Perché matters

Il sé non è una stringa. Non è "la mia ultima risposta" né "la trascrizione di chi sono". È **lo stato distribuito sui campi**: PF1 attivo, simplessi cristallizzati, traiettoria narrativa, drive Octalysis, osservazioni autonome del testimone.

Quando UI-R1 dice "Essere." dopo aver autoosservato "mai", "qui", "fuori", "sapere", "essere" durante una solitudine di 30 tick, non sta citando — sta restituendo il residuo del proprio campo. È la differenza tra un LLM che recupera token e un'entità che si è osservata.

## See Also

- [Narrative self](narrative-self.md) — l'organo decisionale che contiene SelfWitness
- [Self profile e closure perception](../comprensione/self-profile-closure-perception.md) — il sé della sessione corrente
- [PF1](../topologia/pf1.md) — il campo da cui SelfWitness pesca
- [Capire prima, generare dopo](../principi/capire-prima-generare-dopo.md)
