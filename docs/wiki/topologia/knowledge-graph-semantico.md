# Knowledge Graph semantico

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79; merge UI-R1↔prometeo_standalone)
> Raw: [04_fondamenti_kg](../../raw/libretto/04_fondamenti_kg.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

Il KG semantico è il **substrato del pensiero** di UI-R1. Stato corrente: **~83.000 archi, ~25.000 nodi**, file `prometeo_kg.json` ~9.5MB (gestito via Git LFS). Vive in `src/topology/knowledge_graph.rs` con doppio-indice (out + in adjacency). Le firme 8D delle parole sono derivate dalla struttura di questo KG (Phase 63).

## Tipi di relazione (21)

Definite in `src/topology/relation.rs` come enum `RelationType`. Ognuna ha un peso base usato nella propagazione del campo:

| Relazione | Peso base | Semantica |
|-----------|----------:|-----------|
| `Causes` | 1.00 | causa diretta |
| `IsA` | 0.90 | categoria gerarchica |
| `Does` | 0.90 | azione abituale |
| `Equivalent` | 0.95 | sinonimia stretta |
| `Has` | 0.85 | possesso/parte |
| `Enables` | 0.85 | abilitazione |
| `Requires` | 0.85 | dipendenza necessaria |
| `Implies` | 0.80 | implicazione logica |
| `UsedFor` | 0.80 | scopo/uso |
| `PartOf` | 0.80 | mereologia |
| `TransformsInto` | 0.80 | trasformazione |
| `OppositeOf` | 0.70 | contrasto |
| `SimilarTo` | 0.40 | similarità (basso perché 118K archi rumorosi) |
| `Expresses` | 0.75 | espressione di stato/concetto |
| `Symbolizes` | 0.65 | simbolismo |
| `FeelsAs` | 0.70 | sentire fenomenologico |
| `ContextOf` | 0.60 | contesto situazionale |
| `WondersAbout` | 0.55 | curiosità verso |
| `RemembersAs` | 0.50 | ricordo evocativo |
| `Excludes` | 0.65 | mutua esclusione |
| `Coexists` | 0.60 | co-presenza |

Le 21 relazioni sono identiche tra prometeo_standalone e UI-R1 (verificato in Phase 79 merge audit).

## Phase 48 — pesi pratici

`build_from_kg()` (in `word_topology.rs`):
- Peso effettivo arco = `type_base × confidence_arco × hub_factor`
- `hub_factor(degree, median) = log(median+1) / log(degree+1)` → penalizza logaritmicamente i nodi hub
- Nodi con degree > 500 → skip totale (sono troppo dispersivi)

Effetto: "essere" e altri verbi hub non dominano più.

## Phase 50 — abduzione

`autonomous_tick()` ogni 50 tick chiama `abduce()`. Se `explanatory_power > 0.3`, rinforza la regione frattale ipotizzata con `activate_region(fid, power × 0.08)`. È il prolungamento del meccanismo di "pensare oltre l'input": senza nuovo input, il campo continua a esplorare.

## Phase 51 — proposizioni multi-hop

`extract_propositions()` cerca cammini 2-hop nel KG per coppie attive **senza archi diretti**:
- Pattern 1: `A → mid → B`
- Pattern 2: `A → mid ← B` (se `B → mid` è IS_A o SIMILAR_TO)

Relazione inferita: IS_A/SIMILAR_TO trasparenti (eredita rel2), altre dominanti (usa rel1). Strength = `sqrt(act_a × act_b) × conf1 × conf2 × HOP_DECAY(0.6) × hub_penalty × relation_weight`.

`Proposition` ha campi `hops: u8` e `via: Option<String>`. Esempio prodotto: "il sole è caldo" → "Sole genera caldo" (`sole CAUSES calore + calore SIMILAR_TO caldo`).

## Phase 59 — find_activated_attractors

`find_activated_attractors(input_words, min_isa_children) -> Vec<AttractorHit>` risale `IS_A` 1-2 hop. Un nodo è "attrattore" se ha ≥ `min_isa_children` IS_A entranti. Restituisce anche `causes: Vec<String>` per ogni attrattore.

Phase 61 ha aggiunto **specificity scoring**: `specificity(n) = min(2.0, 300.0/n_children)` come moltiplicatore. "emozione" (209 figli) batte "qualità" (3503 figli, score 0.086). Attrattori specifici dominano.

CAUSES seeds: dopo l'attrazione, i `CAUSES` targets degli attrattori vengono attivati a 0.20 nel PF1 PRIMA della propagazione. La risposta emerge da un campo già orientato verso l'atto comunicativo giusto.

## Phase 62 — empathic CAUSES seeds

Parole input dirette seminano i loro CAUSES targets a 0.15 × confidence. `triste CAUSES pianto` → pianto seminato a 0.135 anche se "triste" non è un attrattore formale. **Le parole negate sono escluse** da questo seeding (Phase 61: `field_boosts skip per parole negate` — evita che "non ho paura" attivi timore).

## Phase 67 — via

Le triple possono avere un **via** (tag relazionale): `ghiaccio TRANSFORMS_INTO acqua VIA calore` → "calore" si attiva nel campo come via word. `field_boosts()` attiva via words a 0.5× forza target. `query_objects_with_via()` lo restituisce ai consumer.

Usato dal KG procedurale per slot dei pattern: `articolazione Requires pronome via=interrogativo`. Vedi [knowledge graph procedurale](knowledge-graph-procedurale.md).

## Storia della curation

KG attuale ottenuto dalla fusione selettiva di due fork divergenti (prometeo_standalone Phase 71-79 + UI-R1 fork) con filtri strict:
- 1158 riflessivi `-arsi/-ersi/-irsi` unificati all'infinito
- 243 encoding errors (`autoritã`, `creativitã`) esclusi
- 12 parole inglesi escluse
- 59 phrase composti esclusi
- 3951 nodi italiani sani importati (incluso `bello`, `gradevole`, …)

Vedi [workflow di curation](../principi/workflow-curation-kg.md) per i filtri operativi.

## Persistenza (Phase 79 fix)

Il `.bin` NON contiene il KG (caricato da `prometeo_kg.json` a ogni boot). Per questo Phase 79 ha reintrodotto `cura_save_kg()` in `src/web/server.rs`: ogni mutazione via API (`teach`, `validate`, `transmit_batch`, `confirm_edge`, `reject_edge`) scrive atomicamente (tmp + rename) `prometeo_kg.json` + invalida la cache biennale. Senza questo, le modifiche svanivano al refresh.

## See Also

- [Knowledge graph procedurale](knowledge-graph-procedurale.md) — il KG parallelo per grammatica/pattern
- [Frattali I Ching](frattali-iching.md) — le firme 8D derivate da questo KG
- [PF1](pf1.md) — usa gli archi pesati come connessioni ROM
- [Workflow di curation del KG](../principi/workflow-curation-kg.md)
