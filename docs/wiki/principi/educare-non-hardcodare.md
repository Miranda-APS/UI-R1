# Educare, non hardcodare

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md); [16_web_api](../../raw/libretto/16_web_api.md)

## Overview

Quote originale: *"Le regole grammaticali dovremmo spiegargliele, non infilargliele a forza nel codice."* Questo principio è la regola di scrittura del codice di UI-R1: **Rust contiene meccanismi generici, i dati (grammatica, pattern, percetti) vivono nel KG come triple.** Insegnare un nuovo pattern espressivo = aggiungere triple al [KG procedurale](../topologia/knowledge-graph-procedurale.md). Mai modificare Rust.

## Cos'è "hardcoding" qui

Sono hardcoding (vietati):
- Liste di parole italiane in Rust (es. una vecchia `is_function_word` con 40 parole inline)
- Enum dispatch (es. `match decision.kind { InviteToArticulate => "articolazione", … }`)
- Tabelle pattern → forma ("se intent=greeting → reply='Ciao'")
- Soglie numeriche di transizione comportamentale (>0.5, dopo 3 turni, …)

Non sono hardcoding (ammessi):
- Algoritmi generici (propagazione campo, BFS sul KG, scoring per affinity)
- Costanti fisiche del modello (decay rate, max_per_word cap)
- Parser di formati (lemmatize, tokenize)

La differenza: **il primo gruppo è conoscenza del dominio** (italiano, grammatica, pattern conversazionali). Va insegnata al sistema come dati. **Il secondo gruppo è meccanismo del modello** (come si propaga, come si decompone). Va scritto in Rust.

## Il test operativo

Posso aggiungere un nuovo pattern conversazionale senza toccare un solo file `.rs`?

- Sì → il pattern è dato. Bene.
- No → c'è hardcoding. Riformula.

## Casi canonici

**Phase 77 — `is_function_word`**: era una lista di ~40 parole italiane in Rust. Phase 79 l'ha sostituita con un check strutturale che legge la catena `IsA` dal KG procedurale: una parola è "funzionale" se la sua catena IsA porta a `pronome | articolo | preposizione | marcatore | congiunzione`, oppure è `IsA copula`. Aggiungere/togliere parole di funzione è ora curation, non Rust.

**Phase 79 — `pattern_name_for`**: mappa enum hardcoded `ActionKind → pattern_name` (5 pattern raggiungibili, 6 nel KG inerti). Sostituita da `select_pattern_by_resonance` in [kg_proc_field](../topologia/knowledge-graph-procedurale.md): i pattern sono nodi `IsA pattern` selezionati per risonanza fra percetti seminati e target `UsedFor X via Y`. Aggiungere un nuovo pattern = curation, mai più Rust.

**Phase 79 — Priority 0 closure**: era un `if/then` in `decide_action` ("se `closes_prior_gap.is_some()` → forza `RecognizeClaim`"). Sostituita con un percetto `chiusura` nel KG procedurale che attiva `restituire + posizione + completamento`; il pattern `riconoscimento` vince per risonanza, e `render_riconoscimento` legge `trigger`/`closing_word` direttamente da `report.closes_prior_gap` (closure-aware).

In tutti e tre i casi: stesso comportamento atteso, ma il "quando" è uno stato del campo, non una tabella.

## Conseguenza per i collaboratori AI

Prima di proporre un meccanismo nuovo:
1. Posso esprimerlo come triple nel KG procedurale?
2. Se non posso, **perché**? Il limite è del KG (mi serve un nuovo tipo di relazione, raro) o della mia proposta (sto pensando come un dispatcher)?

Il [Test pre-proposta](test-pre-proposta.md) è il filtro a tre domande. La trappola tipica è proporre numeri-magici travestiti da curation.

## See Also

- [Principi inviolabili](principi-inviolabili.md) — è il principio 6
- [Test pre-proposta](test-pre-proposta.md) — il filtro operativo
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md) — dove vivono i dati
- [Pattern matcher](../comprensione/pattern-matcher.md) — il consumer dei pattern dati
- [Workflow di curation del KG](workflow-curation-kg.md)
