# Syntax Center — grammatica come geometria

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 68)
> Raw: [13_grammatica](../../raw/libretto/13_grammatica.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`syntax_center.rs` non è un parser BNF. È la **fisica grammaticale** di UI-R1: deriva tempi verbali, persona, modo, voce direttamente dalla **geometria 8D** della voce corrente (trigrammi I Ching). La grammatica emerge dal campo, non da una grammar table.

## Voce dalla firma 8D

`derive_voice(field_sig, drives, response_intention, …)`:
- **Persona** dal trigramma inferiore (lower trigram dell'esagramma dominante)
  - Cielo (☰) → 3sg impersonale o 1sg con CD1 alto
  - Terra (☷) → 1pl o impersonale
  - Tuono (☳) → 2sg attivo
  - Acqua (☵) → 3sg riflessivo
  - ecc.
- **Tempo** dalla Dim Tempo (3) e Permanenza (1):
  - `avg_tempo < 0.25 && avg_perm < 0.25` → Imperfect (Phase 56 — soglia alzata)
  - `avg_tempo > 0.70` → Futuro
  - altrimenti Presente (default)
- **Modo** da Dim Confine (4):
  - basso confine + alta interrogazione → Interrogative
  - alto confine + bassa intensità → Imperative
  - altrimenti Indicative
- **Numero** da Permanenza (1) — alta = plurale ricorsivo, bassa = singolare individuante

## Override da response_intention (Phase 67)

`compose()` ha 13° parametro `response_intention: Option<&str>` che colora la voce:
- "risuonare" → 2sg + interrogative (empatico verso l'Altro in distress)
- "esplorare" → 1sg + interrogative (curiosità self-driven)
- "riflettere" → 1sg + indicative (asserzione personale)
- "restare" → silenzio (mood Withdraw)

## Override da other_in_distress (Phase 62)

`other_in_distress: bool` forza `voice.person = Second + mood = Interrogative`. Usato come path alternativo quando P4 Resonate non intercetta.

`render_nucleus()` ora gestisce `Person::Second`:
- CAUSES/SimilarTo → "senti {obj}"
- IsA/PartOf → "provi {obj}"
- Has → "hai {obj}"

## Phase 68 — allineamento I Ching canonico

Prima di Phase 68 c'era un **bug latente**: `derive_8d_from_kg` scriveva in ordine I Ching ma l'enum `Dim` era ordinato diversamente. Conseguenza: `syntax_center.rs` leggeva Tempo come Valenza (tempi verbali pilotati da carica emotiva). Phase 68 ha ridato l'enum canonico Cielo→Lago (Agency=0…Valenza=7).

Effetto: i tempi verbali ora derivano dalla Dim Tempo come previsto. Le coniugazioni sono coerenti con l'intuizione I Ching.

## Capitalizzazione

Tutti i path di ritorno in `generate_willed_inner()` devono capitalizzare (Withdraw path a riga ~2702, fallback finale a riga ~2846). Non è cosmesi — è una convenzione di "fine frase" italiana, e se manca rompe l'output.

## Connettivi semantici (Phase 56)

In `render_nucleus`:
- IS_A/PartOf secondari → virgola (attribuzione)
- Has/Causes/Does/UsedFor/Enables con soggetto condiviso → " e " (coordinazione)
- Default → ", "

DOES relation rendering: copula vuota → `grammar::conjugate(object, Person::Third, tense)` (NON `voice.person`). Il soggetto è un nome, non l'entità stessa.

## See Also

- [Frattali I Ching](../topologia/frattali-iching.md) — le 8 dimensioni sorgente
- [Grammar](grammar.md) — coniugazioni e articoli italiani
- [Expression compose](expression-compose.md) — il consumer
- [Valenza Octalysis](../identita/valenza-octalysis.md)
