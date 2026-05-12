# PF1 — PrometeoField

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79); libretto cap. 02
> Raw: [02_fondamenti_pf1](../../raw/libretto/02_fondamenti_pf1.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

PF1 (`src/topology/pf1.rs`) è il **campo cognitivo primario** di UI-R1. È diviso in due parti:
- **ROM** (`PrometeoField`): 512 byte per parola con la firma 8D + connessioni precalcolate. Persiste sul `.bin`.
- **RAM** (`ActivationState`): stato di attivazione corrente delle parole, vive in memoria, decade.

La propagazione è O(parole_attive × 8) — non O(N totale). Ogni tick il campo evolve secondo le connessioni ROM, ma solo le attive partecipano.

## Struttura ROM (512 B/parola)

Per ogni parola:
- **Firma 8D** — 64 byte (8 float64): le 8 dimensioni I Ching che caratterizzano la parola. Vedi [frattali I Ching](frattali-iching.md).
- **Stability** — 8 byte: quanto è "stabile" la parola (cumulative exposure × consistency).
- **Connessioni** — fino a N neighbours indexati con pesi precomputati dal KG. Hub damping già applicato.
- **Metadata** — POS tag, fractal_id principale, exposure count.

Persiste nel `prometeo_topology_state.bin` (NON tracciato da git: 17MB, generato da bootstrap).

## Struttura RAM (ActivationState)

HashMap<word_id, f64> con le attivazioni correnti, plus:
- `resting_threshold` per ogni parola = `stability × 0.002`
- `propagation_cap` = `MAX_POSITIVE_DELTA = 0.15` (Phase 55 — evita convergenza hub)
- `decay_rate` = `0.92` per tick (Phase 55)

Una parola sotto la soglia (`act < 0.02`) si considera "non attiva" e non partecipa alla propagazione.

## Propagazione

Per ogni tick:
1. **Decay** delle attive: `act *= 0.92`. Sotto soglia → floor a `threshold × 0.5`.
2. **Propagazione**: per ogni parola attiva `w` con attivazione `a_w`, per ogni neighbour `v` con peso `p_wv`:
   - delta = `a_w × p_wv × diminishing_return(a_v)` dove `diminishing_return = 1/(1+4×a_v)` (Phase 55: parole già attive ricevono meno)
   - clamp a `MAX_POSITIVE_DELTA = 0.15`
3. **Resting state**: ogni parola tende verso il proprio resting (`stability × 0.002`).

Effetto: senza input, il campo torna a riposo in ~30 tick (~90s con autonomous_tick ogni 3s). Vedi [Architecture](../../raw/architettura/ARCHITECTURE.md) per i dettagli del loop.

## Cap critici (Phase 55)

Anti-saturazione del campo:
- `MAX_POSITIVE_DELTA = 0.15` in `propagate()` — propagazione cap globale.
- `MAX_RESONANCE_BOOST = 0.10` per-parola da simplex source_words in `receive()`.
- `MAX_PER_WORD = 0.06` in `apply_fractal_resonance()` — la risonanza frattale è sfondo, non segnale.
- Pre-propagazione: parole non-input con `act > 0.25` cappate a 0.25.
- Top-3 fractal voting in `emerge_fractal_activations()` (ogni parola vota per i suoi 3 frattali massimi).

## Sincronizzazione PF1 ↔ word_topology

UI-R1 ha **due sistemi di attivazione** che convivono:
- `pf_activation` — il PF1 (semantica pura, dimensione 8D).
- `word_topology` — sistema legacy ancora consultato da `state_translation.rs`.

Sync: `propagate_field_words()` DEVE sincronizzare PF1 → word_topology con `decay_all(1.0)` + copia hot_words. Resting state coefficienti diversi: PF1=`×0.002`, word_topology=`×0.003`. Vedi nota in CLAUDE.md invariante 11-12.

## Identity seeding e fractal blending (Phase 65)

Dalla seconda conversazione in poi, l'identità semina parole caratteristiche con `identity_seed_field_scaled(20.0)` ≈ 0.06 di forza, **prima** di `propagate_field_words()`. Le parole entità competono con le parole KG-seeded (0.15-0.30) nella selezione generativa.

In `generate_willed_inner`, i frattali attivi vengono blendati con `recent_fractal_attractor(4)` al rapporto 65/35% (dopo 2+ turni di storia). L'espressione riflette l'intersezione tra campo attivo (input) e traiettoria narrativa recente.

## See Also

- [Frattali I Ching](frattali-iching.md) — le 8 dimensioni della firma
- [Lexicon](lexicon.md) — il dizionario delle parole con firme
- [Knowledge graph semantico](knowledge-graph-semantico.md) — la fonte delle connessioni ROM
