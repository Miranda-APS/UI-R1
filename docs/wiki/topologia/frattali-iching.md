# Frattali I Ching — 8 dimensioni canoniche

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, invarianti 121-127); libretto cap. 05
> Raw: [05_campo_frattali](../../raw/libretto/05_campo_frattali.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md); [architettura_olografica](../../raw/architettura/architettura_olografica.md)

## Overview

UI-R1 usa **8 dimensioni** come spazio cognitivo, allineate ai trigrammi I Ching in **ordine canonico Cielo→Lago** (Phase 68). Ogni parola ha una firma 8D che la posiziona in questo spazio. I 64 frattali (8×8 esagrammi) sono attrattori regionali emergenti. Questa architettura è il "DNA" del sistema — i 64 frattali sono la condizione dell'emergenza, non un problema.

## Le 8 dimensioni

Ordine canonico I Ching (enum `Dim` in `src/topology/primitive.rs`):

| idx | Nome | Trigramma | Simbolo | Significato |
|----:|------|-----------|---------|-------------|
| 0 | **Agency** | Cielo | ☰ | agisce o subisce — `CAUSES_out / CAUSES_in ratio` |
| 1 | **Permanenza** | Terra | ☷ | permanenza vs evanescenza — IS_A children (massa) |
| 2 | **Intensita** | Tuono | ☳ | movimento vs inerzia — CAUSES + valenza |
| 3 | **Tempo** | Acqua | ☵ | futuro vs passato — catene causali (profondità) |
| 4 | **Confine** | Montagna | ☶ | definito vs vago — specificità IS_A + OPPOSITE_OF |
| 5 | **Complessita** | Vento | ☴ | complesso vs semplice — log del grado totale |
| 6 | **Definizione** | Fuoco | ☲ | nitido vs sfumato — genitori IS_A + OPPOSITE_OF |
| 7 | **Valenza** | Lago | ☱ | attrae vs respinge — BFS emotiva (Phase 63) |

Le dimensioni sono **derivate da struttura KG** (Phase 63 `derive_8d_from_kg`), non da co-occorrenze. La geometria riflette il significato relazionale, non la frequenza.

## Phase 68 — allineamento canonico

Fino a Phase 67 incluso, `derive_8d_from_kg` scriveva in ordine I Ching ma l'enum era ordinato diversamente. Conseguenza: `syntax_center.rs` leggeva Tempo come Valenza (tempi verbali pilotati da carica emotiva), `DRIVE_DIM` mappava Octalysis sulle dimensioni sbagliate, `biennale_pos` mescolava le coordinate UI. Phase 68 ha ridato l'enum canonico e migrato tutto in un colpo (`.bin.pre_iching_ordering` come backup).

## 64 frattali

`FractalRegistry` (`src/topology/fractal.rs`) registra 64 attrattori: ogni frattale è un esagramma I Ching = combinazione di due trigrammi (lower + upper). `FractalId = lower × 8 + upper`.

Esempi (numerazione I Ching):
- 1 = Cielo + Cielo = `CREATIVITA`
- 2 = Terra + Terra = `RICETTIVITA`
- 11 = Cielo + Terra = `PACE`
- 33 = Montagna + Montagna = `CORPO`
- 53 = Vento + Montagna = `PENSIERO` (vedi interocezione KG-derivata Phase 53)
- 64 = Fuoco + Acqua = `NON ANCORA FINITO`

I 64 nomi sono in `data/kg/nucleus.tsv` (926 triple hub-words per stati).

## Affinità parola → frattale

`affinity(word_sig, fractal_sig) = dot_product / norms`. Affinità ∈ [-1, +1]. Una parola "appartiene" debolmente o fortemente a un frattale a seconda della similarità tra la sua firma 8D e la firma media del frattale.

In `emerge_fractal_activations()` (Phase 55), ogni parola attiva vota solo per i suoi **3 frattali con affinità massima** — non tutti 64. Punteggi normalizzati al massimo. Elimina saturazione uniforme.

## Drive Octalysis ↔ dimensioni

Vedi [valenza Octalysis](../identita/valenza-octalysis.md) per la mappatura completa. `DRIVE_DIM = [0, 6, 5, 4, 7, 3, 2, 1]` (Phase 68):
- CD1 Scopo → Agency(0)
- CD2 Achievement → Definizione(6)
- CD3 Creativity → Complessità(5)
- CD4 Ownership → Confine(4)
- CD5 Relation → Valenza(7)
- CD6 Scarcity → Tempo(3)
- CD7 Unpredictability → Intensità(2)
- CD8 Avoidance → Permanenza(1)

## Principio olografico (Phase 63)

Le firme 8D usano le stesse 8 dimensioni I Ching sia in **scrittura** (derivazione da KG) che in **lettura** (proiezione nel campo). La "luce" I Ching è coerente: lo stesso strumento misura lo stesso spazio.

Phase 63 ha rimosso la perturbazione hash UTF-8 dalle nuove parole (`new_from_context()` in lexicon.rs): le parole nuove partono dalla firma del contesto pura. La differenziazione è esclusivamente fenomenologica (esposizioni nel campo) o strutturale (KG via `rederive-signatures`).

## See Also

- [PF1](pf1.md) — usa le firme 8D per propagazione
- [Lexicon](lexicon.md) — dove vivono le firme
- [Knowledge graph semantico](knowledge-graph-semantico.md) — la fonte delle dimensioni (Phase 63)
- [Valenza Octalysis](../identita/valenza-octalysis.md) — i drive emergono dalle 8D
- [Memoria-sfera di haiku](../interfacce/memoria-haiku.md) — la sfera dei 64 attrattori usata come memoria (Phase 82)
