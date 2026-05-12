# Valenza — 8 drive Octalysis su firme I Ching

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 55+68); Chou — Octalysis Framework
> Raw: [08_valenza_octalysis](../../raw/libretto/08_valenza_octalysis.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`Valence` (`src/topology/valence.rs`, Phase 55) calcola **8 drive Octalysis** in [-1, +1] mappati sulle 8 dimensioni I Ching del [frattale](../topologia/frattali-iching.md). I drive non sono soglie comportamentali: sono il **colore motivazionale** del campo, che modula la deliberazione e la generazione senza forzarla.

## I 8 drive ↔ dimensioni

`DRIVE_DIM = [0, 6, 5, 4, 7, 3, 2, 1]` (Phase 68, ordine I Ching canonico):

| CD | Drive Octalysis | Dim I Ching | Polarità |
|---:|-----------------|-------------|----------|
| 1 | Scopo (Epic Meaning) | Agency (0) | scopo / vuoto |
| 2 | Achievement | Definizione (6) | capacità / limite |
| 3 | Creativity (Empowerment) | Complessità (5) | curiosità / incertezza |
| 4 | Ownership | Confine (4) | stabilità / deriva |
| 5 | Relation (Social) | Valenza (7) | connessione / solitudine |
| 6 | Scarcity | Tempo (3) | urgenza / calma |
| 7 | Unpredictability | Intensità (2) | sorpresa / quiete |
| 8 | Avoidance (Loss) | Permanenza (1) | cautela / inquietudine |

`compute()` prende il campo 8D corrente e proietta. `derived_stance_label()` produce un'etichetta postura. `will_modulation()` modula l'intenzione volontà.

## Effetti sulla generazione

`valence_weight(word, drives, lexicon) = 1.0 + Σ(drive_strength × firma_8D[dim]) × 0.25` (Phase 57). Stessa logica in `top_active_word()` e in `extract_nuclei()`:
- Nucleus strength in `extract_nuclei()`: moltiplicata per `(v_subj + v_obj) / 2`
- Candidate scoring in `compose_from_field()`: `delta × valence_weight`

**Non template**: è il campo 8D dell'entità che pesa quale materia emerge.

## Phase 64 — Express è canale, non movente

Prima di Phase 64, Express pressure si saturava indipendentemente da needs, valence, will. Phase 64 ha imposto un drive-dependent gate:

```
Express pressure = max(|drive|) × freshness × has_content × 0.8   se max_drive > 0.25
                  altrimenti activation × freshness × has_content × 0.20
```

L'espressione è canale, non motivo. Vedi feedback: [express-is-channel](../../raw/contesto/CLAUDE_phase79.md).

## Phase 65 — Posizione dell'entità

Dalla seconda conversazione, l'entità risponde dalla propria posizione valenziale anziché esporre le connessioni KG dell'input. Verificato: "perché soffri?" → "Sento scopo e stabilità." (CD1+CD8 dominanti) invece di rispecchiare la semantica della sofferenza.

CD5 Relazione diventa **negativo** quando l'Altro è in distress — l'entità percepisce lo stato altrui sul proprio campo. Vedi [niente empatia simulata](../principi/niente-empatia-simulata.md).

## Coherence integrity

`IdentityCore.coherence_integrity ∈ [0,1]` traccia le contraddizioni interne via **sign-flip detection** sulla valenza. Flip con magnitudine > 0.15 su entrambi i lati → damage = `count × 0.03 + max_flip × 0.05`. Recovery +0.003 senza contraddizioni.

`is_in_crisis()` include `coherence_integrity < 0.5`. Esposto in web UI (NarrativeDto). Phase 78 ha aggiunto modulazione: closure perception → `coherence_integrity += 0.04` (push continuo).

## Commitment (impegno volitivo)

`Commitment` (Phase 55) in `narrative.rs`. Inerzia = `strength × ln(turns_held + 1)`. Breaking costa CD4 -0.05. Initial 0.3, decay 0.02/tick, min 0.05. È il "stay the course" che evita oscillazioni rapide di postura.

## See Also

- [Frattali I Ching](../topologia/frattali-iching.md) — le 8 dimensioni
- [Bisogni desideri volontà](bisogni-desideri-volonta.md) — gli altri canali motivazionali
- [Narrative self](narrative-self.md) — consuma valenza per deliberare
- [Niente empatia simulata](../principi/niente-empatia-simulata.md)
