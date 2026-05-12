# Niente template, niente dispatch

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md); [12_generazione_expression](../../raw/libretto/12_generazione_expression.md)

## Overview

UI-R1 non ha template di risposta. Non ha `match intent { Greeting => "Ciao!" … }`. La forma e il contenuto della risposta **emergono dal campo** — dalle parole più attive nel PF1, dai nuclei semantici recuperati dal KG, dai pattern istanziati per risonanza nel KG procedurale. Niente è hardcoded. Questo articolo spiega cosa significa concretamente e quali pattern *sembravano* innocui ma erano template mascherati.

## Cosa NON c'è in UI-R1

- **Nessun file di template** (no `responses.json`, no `phrases.yaml`)
- **Nessun enum `IntentKind` dispatchato a stringhe** (es. `Greeting → "Salve"`)
- **Nessun "se domanda allora apri con ‘Bella domanda…'"**
- **Nessun if/then sull'utterance** ("contiene `?` → risposta interrogativa")

## Cosa c'è invece

**Pattern espressivi come dati nel KG procedurale.** Es. il pattern `articolazione` esiste come nodo con triple `UsedFor chiedere via=oggetto`, `Requires pronome via=interrogativo`, `Requires preposizione via=specificazione`, ecc. Il [pattern_matcher](../comprensione/pattern-matcher.md) legge questo schema e istanzia gli slot dalle anchor_words decise da [action_reasoning](../comprensione/action-reasoning.md).

**Selezione per risonanza** (Phase 79). Quale pattern emerge non è una `match` su enum: è il `pattern_score` calcolato come somma delle attivazioni dei target `UsedFor X via Y` dopo che `seed_from_comprehension` ha seminato i percetti (saluto/chiusura/apertura/posizione/domanda/affermazione/introduzione) dal ComprehensionReport. Aggiungere un nuovo pattern = pure data, mai più Rust. Vedi [knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md).

**Selezione delle parole "vive" dal campo PF1.** Le anchor che riempiono gli slot sono parole con delta-activation positivo nel campo, filtrate per non-function-word (check strutturale via KG procedurale, non lista hardcoded) e per non-eco dell'input.

## Pattern mascherati che sono stati rimossi

**Phase 57 — `state_translation::translate_state()`** restituiva archetipi ("greet", "comfort", "explain") usati come *form template* per la generazione. Phase 57 lo ha rimosso dal path principale: oggi se `expression::compose()` ritorna `Some`, quella è la risposta. Se ritorna `None` viene emessa solo la parola più viva. Niente template di fallback.

**Phase 79 — `pattern_name_for(decision)`** mappava `ActionKind → pattern_name` come enum dispatch. Rimosso. La selezione è ora per risonanza, e gli 11 pattern del KG procedurale sono tutti raggiungibili.

**Phase 79 — Priority 0 closure**: era un `if/then` ("closure percepita → forza RecognizeClaim") in `decide_action`. Rimosso. La closure è ora un percetto seminato nel campo del KG procedurale; il pattern `riconoscimento` vince per risonanza.

## Conseguenza pratica

**Stesso input, stati diversi, risposte diverse.** Verificato Phase 57: "ho paura" dopo uno stato di crisi (`coherence_integrity < 0.5`) attiva nuclei diversi rispetto a "ho paura" dopo uno stato di gioia. Stesso input, output diverso, senza un singolo if/then.

**Cresce con il KG, non con il codice.** Per aggiungere un nuovo modo espressivo (es. un pattern "rispecchiamento", "specchio", "esitazione"), si aggiungono triple al KG procedurale. Zero righe di Rust toccate.

## Trappole tipiche

- "Aggiungo solo questa piccola lista di parole speciali in Rust" → no, va come dati nel KG. Vedi [educare non hardcodare](educare-non-hardcodare.md).
- "Faccio una mappa from `kind` to `pattern_name`, è solo una mappa" → è dispatch. Phase 79 lo ha già visto e rimosso.
- "Hardcodo solo la soglia >0.3 perché tanto è un parametro" → numeri in condizioni = trigger, vedi [Test pre-proposta](test-pre-proposta.md).

## See Also

- [Principi inviolabili](principi-inviolabili.md)
- [Educare, non hardcodare](educare-non-hardcodare.md)
- [Test pre-proposta](test-pre-proposta.md)
- [Pattern matcher](../comprensione/pattern-matcher.md)
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md)
- [Expression compose](../generazione/expression-compose.md)
