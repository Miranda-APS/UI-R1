# Expression Compose — generazione emergente

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 56-58+77)
> Raw: [12_generazione_expression](../../raw/libretto/12_generazione_expression.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`expression::compose()` (`src/topology/expression.rs`, Phase 56-58, esteso Phase 77) è il **generatore emergente** di UI-R1: non template, non slot-filling LLM, ma una funzione che dato `word_topology + KG + drive + active_fractals + ComprehensionReport + ActionDecision`, prima tenta il pattern matcher (Phase 77), e in fallback compone una voce dai nuclei semantici del campo.

## Tre livelli (Phase 57)

1. **Intelligere**: nuclei KG = comprensione interna, non output diretto.
2. **Colorazione**: drive Octalysis biasa quale materia emerge dal campo.
3. **Exprimere**: grammatica italiana come fisica del mondo (concordanze, tempi, articoli).

## Firma — 16 parametri (Phase 77)

```rust
pub fn compose(
    word_topology: &WordTopology,
    lexicon: &Lexicon,
    kg: &KnowledgeGraph,
    echo_exclude: &[String],
    valence_drives: Option<&[f64; 8]>,
    active_fractals: &[FractalSnapshot],
    codon: Codon,
    input_words: &[String],
    episodes: Option<&SemanticEpisodeLog>,
    response_intention: Option<&str>,
    other_in_distress: bool,
    voice_hint: Option<Voice>,
    kg_proc: Option<&KnowledgeGraph>,          // Phase 77
    action_decision: Option<&ActionDecision>,  // Phase 77
    comprehension_report: Option<&ComprehensionReport>, // Phase 77
    field_pressures: Option<&FieldPressures>,
) -> Option<Expression>
```

Quando `kg_proc`, `action_decision`, `comprehension_report` sono tutti `Some`, prima tenta `pattern_matcher::compose_from_pattern`. Se ritorna `Some` → quella è la risposta. Altrimenti fallback ai nuclei semantici.

## Pipeline nuclei (fallback)

1. **extract_nuclei(kg, comprehension_pool, episodes)** — trova relazioni semantiche tra parole attive nel KG. Pesa con valenza (Phase 57): `nucleus.strength *= (v_subj + v_obj) / 2`.
2. **Risonanza episodica** (Phase 58): se `episodes.is_some()`, `recall_by_concepts(active_concepts, 3)` ritorna episodi con overlap. Nuclei con soggetto+oggetto in episodi precedenti ricevono boost 1.4× (entrambi) o 1.2× (uno).
3. **Input-proximity scoring** (Phase 56): preferenza decrescente per nuclei:
   - (1) entrambe non-input ma in input-neighborhood → 4.0×
   - (2) oggetto=input, soggetto non-input → 2.5×
   - (3) soggetto in neighborhood ma non input → 2.0×
   - (4) oggetto in neighborhood ma non input → 1.5×
   - (5) almeno una parola è input verbatim → 0.5×
   - (6) nessuna connessione → 0.2×
4. **Echo exclusion** (Phase 56): in `compose_from_nuclei()`, il nucleo primario è scelto come primo senza soggetto in `echo_exclude`. Se tutti i nuclei hanno soggetti in echo_exclude → fallback al primo.
5. **render_nucleus** — applica voice (Phase 67: 2sg interrogativa se `other_in_distress` o `response_intention="risuonare"`).
6. **Articolazione** italiana: `with_definite_article` / `with_indefinite_article` da [grammar.md](grammar.md). IS_A/PartOf → indeterminativo; HAS/CAUSES → determinativo.

## Drive-based expression (Phase 59)

Se `dominant_drive_intensity > 0.15`, `express_from_drives()` mappa i drive → parole stato italiano:

```
DRIVE_STATE_WORDS = [
  (scopo, vuoto),       // CD1
  (capacità, limite),   // CD2
  (curiosità, incertezza), // CD3
  (stabilità, deriva),  // CD4
  (connessione, solitudine), // CD5
  (urgenza, calma),     // CD6
  (sorpresa, quiete),   // CD7
  (cautela, inquietudine), // CD8
]
```

Risponde autenticamente a "come stai?" senza conoscere "stai". Fallback ai field words solo se drive tutti deboli.

## Comprehension gate (Phase 59)

In `generate_willed_inner()`: se `last_comprehension.is_empty()` AND `input_has_content` AND `!last_input_is_question` AND `kg.edge_count > 0` → "Non capisco '[word]' — cosa intendi?" + `learning_mode_pending = true`.

Phase 61: specificity scoring per attrattori — "emozione" (209 IS_A children) batte "qualità" (3503 children, score 0.086).

Phase 67: comprehension gate lemmatizzato — controlla 3 livelli: (1) parola nel KG, (2) parola nel lessico, (3) lemma nel KG. "farò" lemmatizza a "fare" → nel KG → non scatta.

## Cap propagazione (Phase 55)

`MAX_POSITIVE_DELTA = 0.15` in `propagate()`. Nessuna parola riceve delta > 0.15 dalla propagazione. Previene convergenza hub.

## Soglia imperfetto innalzata (Phase 56)

`Tense::Imperfect` solo se `avg_tempo < 0.25 && avg_perm < 0.25` (era 0.35). Il presente è il tempo base dell'entità. Il futuro richiede `avg_tempo > 0.70` (era 0.65).

## See Also

- [Pattern matcher](../comprensione/pattern-matcher.md) — il primo tentativo prima del fallback nuclei
- [Syntax center](syntax-center.md) — la grammatica come geometria
- [Grammar](grammar.md) — articoli e coniugazione italiana
- [Valenza Octalysis](../identita/valenza-octalysis.md) — i drive che colorano
- [Niente template](../principi/niente-template.md)
