# Knowledge Graph procedurale

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79; Phase 75-79 design)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

A partire da Phase 75, UI-R1 ha **due KG paralleli**, non uno fuso: il [KG semantico](knowledge-graph-semantico.md) per la conoscenza del mondo, e il **KG procedurale** (`prometeo_kg_procedurale.json`, 10 pattern, 9 percetti — conteggio archi corrente nell'[index](../index.md)) per grammatica e pattern espressivi. Aree distinte del "cervello" linguistico. La selezione del pattern emerge per risonanza tra percetti seminati dal ComprehensionReport e i target dei pattern (Phase 79).

## Perché separato

Il KG semantico contiene fatti del mondo (`cane IS_A animale`, `acqua Causes bagnato`). Il KG procedurale contiene meta-linguaggio: classi grammaticali, ruoli sintattici, pattern espressivi italiani. Mescolarli inquinava la propagazione semantica (es. `pronome IsA classe-grammaticale` non deve attivare nulla nel campo del discorso reale).

## Struttura

File: `prometeo_kg_procedurale.json` (~220 nodi; conteggio archi corrente nell'[index](../index.md)). Costruito idempotentemente da `curate_kg_procedurale.py` (sezioni §A-§H.quinquies). Aggiungere nuovi pattern qui, **MAI** in Rust.

Tre tipi di nodi:

1. **Classi grammaticali** (parole funzione tag): `pronome`, `articolo`, `preposizione`, `marcatore`, `congiunzione`, `copula`, `verbo`, `interiezione`, `avverbio`, …
2. **Percetti** (categorie semantiche del ComprehensionReport): `saluto`, `chiusura`, `apertura`, `domanda`, `posizione`, `affermazione`, `introduzione`, `incertezza`, `curiosità`
3. **Pattern espressivi** (nodi `IsA pattern`): `articolazione`, `identificazione`, `asserzione`, `riconoscimento`, `ricambio`, `presentazione`, `posizionamento`, `specchio`, `esplorazione`, `esitazione`

## Tre tipi di triple

**Classificazione parole**:
```
cosa IsA pronome
cosa IsA interrogativo
del IsA preposizione
del Equivalent di via=il
```

**Schema pattern**:
```
articolazione IsA pattern
articolazione UsedFor chiedere via=oggetto
articolazione Requires pronome via=interrogativo
articolazione Requires preposizione via=specificazione
articolazione Requires verbo via=predicato
articolazione Requires marcatore via=interrogativo
```

`UsedFor X via Y` = "questo pattern serve per X attraverso la dimensione Y". Lo `via` è il **bersaglio della risonanza** (Phase 79).

**Bridge percetto → concetto** (Phase 79):
```
chiusura Causes restituire
chiusura Causes posizione
chiusura Causes completamento
domanda Causes chiedere
domanda Causes vuoto
saluto Causes restituire
```

`<percetto> Causes <concetto>` è il bridge che lega le percezioni del ComprehensionReport ai target dei pattern. Nessuna nuova `RelationType` aggiunta — `Causes` ha esattamente la semantica giusta (propagazione del campo).

## Pattern matching per risonanza (Phase 79)

Il vecchio dispatch hardcoded `pattern_name_for(decision)` è stato eliminato. Sostituito da `select_pattern_by_resonance` in `src/topology/kg_proc_field.rs`:

```
1. seed_from_comprehension(report, kg_proc) → KgProcActivation (HashMap<String, f64> capped 1.0)
   Per ogni proprietà del report (speech_act.kind, closes_prior_gap, gaps, …),
   semina un percetto in kg_proc. Il percetto propaga via `Causes` ai concetti
   target (es. `chiusura → restituire` 0.7, `chiusura → posizione` 0.5).

2. pattern_score(p) = Σ activation[X] + activation[Y]  per ogni  UsedFor X via Y
   Es: riconoscimento UsedFor restituire via=posizione → 0.7+0.5 = 1.2

3. select_pattern_by_resonance = argmax(pattern_score)
```

Effetto: tutti i 10 pattern del kg_proc sono raggiungibili. Aggiungere un pattern nuovo = curation, mai Rust.

## I 10 pattern Phase 79

| Pattern | UsedFor … via … | Quando vince |
|---------|-----------------|--------------|
| `articolazione` | chiedere via=oggetto/vuoto | gap aperto + apertura |
| `identificazione` | rispondere via=identità | domanda + self-reference 2sg |
| `asserzione` | porre via=affermazione | Elaborate |
| `riconoscimento` | restituire via=posizione/completamento | closure perception |
| `ricambio` | restituire via=saluto | speech_act = saluto |
| `presentazione` | dichiarare via=introduzione | claim Identity ("mi chiamo X") |
| `posizionamento` | porre via=posizione | claim posizionante |
| `specchio` | restituire via=affermazione | rispecchiamento input |
| `esplorazione` | chiedere via=curiosità | curiosità genuina |
| `esitazione` | porre via=incertezza | incertezza epistemica |

## I 9 percetti

`saluto`, `chiusura`, `apertura`, `domanda`, `posizione`, `affermazione`, `introduzione`, `incertezza`, `curiosità` — nodi italiani atomici (no underscore, no jargon). Coerenti con il KG semantico dove esistono già come concetti (`posizione`, `saluto`, `vuoto`, `identità`, `incertezza`, `curiosità`, `completamento`).

Phase 79 ha ripulito il vocabolario kg_proc:
- `claim` → `posizione` (era inglese)
- `fatico` → `saluto` (tecnicismo Jakobson)
- `dubitazione` → `esitazione` (latinismo coniato)
- `rispecchiamento` → `specchio` (atomico)
- `causazione` rimosso (asserzione + verbo causale bastava)

## See Also

- [Knowledge graph semantico](knowledge-graph-semantico.md) — il KG parallelo per il mondo
- [Comprehension report](../comprensione/comprehension-report.md) — la fonte dei percetti
- [Pattern matcher](../comprensione/pattern-matcher.md) — il consumer
- [Educare, non hardcodare](../principi/educare-non-hardcodare.md) — il principio
- [Niente template](../principi/niente-template.md)
