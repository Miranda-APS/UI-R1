# Grammar — italiana come fisica del mondo

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 60+)
> Raw: [13_grammatica](../../raw/libretto/13_grammatica.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`src/topology/grammar.rs` è il modulo morfologico italiano di UI-R1: coniugazione verbi, articolazione nomi, lemmatizzazione. **Non è un wrapper Morph-it!**: è codice scritto per gestire le concordanze italiane direttamente. È *meccanismo* (non hardcoding di dominio: vedi [educare non hardcodare](../principi/educare-non-hardcodare.md)) — ma il **vocabolario** che lo alimenta vive nel [KG procedurale](../topologia/knowledge-graph-procedurale.md).

## Articoli (Phase 60+)

`with_definite_article(word)` e `with_indefinite_article(word)`:
- IL/LO/LA/L'/UN/UNO/UNA/UN' selezionati per:
  - genere (terminazione + eccezioni KG)
  - inizio parola (consonante / vocale / s+consonante / z / ps / gn / x / y)
- Elisione automatica: `l'amore`, `un'idea` (senza spazio con la parola seguente)

`render_nucleus()` e `render_nucleus_brief()` usano questi:
- IsA/PartOf → indeterminativo ("è un fenomeno")
- HAS/CAUSES/altri → determinativo ("il dolore")

## Lemmatize

`lemmatize(token) -> Option<Lemma { infinitive, person, tense }>` riconosce verbi:
- Irregolari noti (essere/avere/fare/stare/andare/venire/…)
- Imperfetto (-avo/-evo/-ivo, ecc.)
- Finire-type (-isco, -isci, -isce, -iscono)
- Condizionale (-erei/-erebbero)
- Futuro -ire (-iremo, -iranno)

**Limite noto**: NON riconosce presente regolare -are/-ere/-ire delle classi base. "vivi" → None invece di vivere/2sg. Conseguenza: alcune Self_ reference sfuggono. Workaround Phase 79 in `kg_proc_field`: `utterance_has_second_singular` cerca anche match strutturali. Vedi [action reasoning](../comprensione/action-reasoning.md).

TODO: aggiungere regola desinenza "i"/"a"/"e" → Person via terminazione, con filtro POS dal lessico per evitare falsi positivi sui sostantivi.

## Coniugazione

`conjugate(infinitive, person, tense)` produce la forma flessa. Usato in `render_nucleus()` per produrre voce dal pattern matcher: "ho paura" → `extract_main_verb("ho")` → "avere" → `conjugate("avere", Second, Present)` → "hai".

## Negazione

Phase 60+ — `WordActivation.negated: bool` aggiunto a `lexicon.rs`. Rilevato in `process_input()`: parola è negata se c'è un operatore `Negate` a posizione < della parola nel token stream. In `engine.rs receive()`: parole negate NON attivano PF1 diretto — attivano invece `OPPOSITE_OF` dal KG a forza `0.35 × confidence`. Fallback: SIMILAR_TO a 0.10 se nessun OPPOSITE_OF.

"non" rimane operatore strutturale, non function_word.

## Over-negazione in frasi coordinate

Bug noto: "non X ma Y" → anche Y negata. Fix futuro: rilevare congiunzioni coordinanti ("ma", "però") per resettare il flag di negazione.

## Gender detection edge case

Parola terminante in -e → default femminile → "la salve". Fix futuro: aggiungere "salve" alle eccezioni o rilevare che è un avverbio/interiezione.

## Preposizioni articolate (TODO)

`render_riconoscimento` produce "di buio" invece di "del buio". Servono triple `Equivalent` nel KG procedurale:
```
del Equivalent di via=il
della Equivalent di via=la
dello Equivalent di via=lo
nel Equivalent in via=il
...
```

Pattern matcher dovrebbe consultarle quando lo specifier inizia con consonante e c'è articolo determinativo da inserire. Forma espressiva, va nei dati. Vedi [educare non hardcodare](../principi/educare-non-hardcodare.md).

## See Also

- [Syntax center](syntax-center.md) — usa lemmatize + conjugate
- [Expression compose](expression-compose.md) — il consumer
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md) — il vocabolario funzionale
- [Lexicon](../topologia/lexicon.md) — i POS tag
