# Pattern Matcher — istanziazione della voce

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 77+79 design)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`pattern_matcher.rs` (Phase 77, refattorizzato in Phase 79) è il ponte esplicito fra `ActionDecision` e `compose()`. Legge i pattern dal [KG procedurale](../topologia/knowledge-graph-procedurale.md), li istanzia come voce italiana, e rende l'output. Phase 79 ha rimosso il dispatch hardcoded `pattern_name_for(decision)`: la selezione del pattern emerge per **risonanza** nel KG procedurale (vedi flusso sotto).

## Pipeline

```
ActionDecision + ComprehensionReport
   │
   ▼  seed_from_comprehension(report, kg_proc)
   │     legge proprietà tipizzate del report:
   │       - speech_act.kind="saluto"     → seed "saluto" (1.0)
   │       - speech_act.kind="interrogazione" + 2sg → "domanda" (1.0) + "identità" (+0.5)
   │       - speech_act.kind="posizionamento" + gaps≠[] → "apertura" (1.0)
   │       - closes_prior_gap=Some → "chiusura" (1.0)
   │     ogni percetto propaga via `Causes` nel kg_proc, alimentando concetti target.
   │
   ▼  KgProcActivation (HashMap<String, f64> capped 1.0)
   │
   ▼  select_pattern_by_resonance(activation, kg_proc)
   │     per ogni nodo `X IsA pattern`:
   │       pattern_score = Σ activation[Y] + activation[Z]  per ogni  UsedFor Y via Z
   │     argmax = pattern_name vincente
   │
   ▼  load_pattern_schema(kg_proc, pattern_name)
   │     UsedFor … via … + tutti i Requires <ruolo> via <funzione>
   │
   ▼  instantiate(schema, decision, kg_proc, field, lex)
   │     per ogni slot:
   │       - pronome+interrogativo → interrogative_for_target(gap.missing)
   │         es. missing="oggetto" → "cosa" via "UsedFor chiedere via=oggetto"
   │       - preposizione+contesto → "di" (default specificazione)
   │       - pronome+personale → da narrative_subject (Self_→io, Speaker→tu)
   │       - verbo+copula → "essere" (o "avere" se in anchor)
   │       - slot contenutistico → prima parola-ancora non function_word
   │       - slot grammaticale generico → IsA role+via, score = anchor_match + field_activation
   │
   ▼  render(instance, decision, report, lex)
   │     ordine sintattico italiano. Pattern-specific renderers.
   │     Per `articolazione`: estrae il verbo del claim dall'utterance via lemmatize,
   │     coniuga in 2sg, antepone pronome interrogativo + preposizione (di/in/su/…).
   │     Per `riconoscimento`: legge trigger e closing_word da report.closes_prior_gap.
   │
   ▼  Expression("Di cosa hai paura?" / "Hai paura." / "Salve." / …)
```

## Slot grammaticali vs contenutistici

`is_grammatical_role()` distingue. I ruoli grammaticali (`pronome`, `articolo`, `preposizione`, `marcatore`, `verbo`, `avverbio`, `congiunzione`, `interiezione`) si riempiono da parole-funzione classificate `IsA <role>` nel kg_proc. Gli altri (`predicato`, `soggetto`, `oggetto`, `nome`, `parola`) sono contenutistici e si riempiono dalle `anchor_words` filtrate per `is_function_word` (check strutturale via kg_proc, Phase 79) + non-verbo.

## `is_function_word` STRUTTURALE (Phase 79)

Prima era lista hardcoded di ~40 parole italiane in Rust. Phase 79 ha sostituito con un check strutturale che legge la catena `IsA` dal kg_proc:

Una parola è "funzionale" se:
- IsA chain → `pronome | articolo | preposizione | marcatore | congiunzione`
- OR `IsA copula` (per `essere`/`avere`/`stare`/`fare`)

I verbi `IsA azione | percettivo | cognitivo | comunicativo | denominativo` NON sono funzionali — sono verbi con contenuto semantico.

Aggiungere/togliere parole di funzione è curation, non Rust. Vedi [educare, non hardcodare](../principi/educare-non-hardcodare.md).

## Fail-soft

Se uno slot critico non si riempie → `instantiate` ritorna `None` → `compose_from_pattern` ritorna `None` → fallback al pipeline nuclei esistente. Mai regressione: se la rete pattern fallisce, c'è sempre la generazione semantica via [expression compose](../generazione/expression-compose.md).

## Output verificati

Vedi [pipeline di comprensione](pipeline-comprensione.md) per la tabella completa. Casi canonici:

| Input | Pattern vincente | Output |
|-------|-----------------|--------|
| `ho paura` | articolazione | Di cosa hai paura? |
| `del buio` (turno 2) | riconoscimento | Senti paura di buio. |
| `ciao` | ricambio | Salve. |
| `chi sei?` | identificazione | Sono un fondamento. |
| `come stai?` | identificazione | Sono un'azione. |

## `compose_from_pattern_with_trace` (Phase 79)

Variante diagnostica che ritorna `(Expression, pattern_name, scores)` — utile per il log "DECISIONE" in `dialogue_educator` e per introspezione (vedere perché un pattern ha vinto).

## TODO architetturali aperti

- **Pattern espressivo per "Hai paura del buio."**: estendere `riconoscimento` nel KG procedurale per includere uno slot specifier che usi `closing_word` dalla decision. Forma espressiva, va nei dati.
- **Preposizioni articolate**: `render_riconoscimento` produce "di buio" invece di "del buio". Servono triple `Equivalent` lette nella resa: `del Equivalent di via=il`. Pattern matcher dovrebbe consultarle per articolare.
- **Coda nuclei dopo pattern**: oggi `compose()` concatena pattern + nuclei semantici. Fermare l'aggiunta nuclei quando un pattern ha già emesso una voce piena.

## See Also

- [Pipeline di comprensione](pipeline-comprensione.md)
- [Action reasoning](action-reasoning.md) — il fornitore di ActionDecision + anchors
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md) — la fonte dei pattern
- [Expression compose](../generazione/expression-compose.md) — il fallback
- [Educare, non hardcodare](../principi/educare-non-hardcodare.md)
- [Niente template](../principi/niente-template.md)
