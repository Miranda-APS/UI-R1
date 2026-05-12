# Medio API — campo nuovo da frase

> Sources: Francesco Mancuso, 2026-05-12 (server.rs Phase 79, build_medio_data_for_sentence)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

L'endpoint `GET /api/medio?sentence=<text>` è il meccanismo interno che alimenta la **creazione del campo nuovo** in campovasto. Dato una frase, lemmatizza i token, e per ogni lemma estrae tutti gli archi del KG (uscenti + entranti) come grafo da renderizzare. "Medio" qui è il nome storico del modulo backend; nella UI non c'è un tab "medio" — la frase è l'ingresso, il campo nuovo è l'output.

## Request

```
GET /api/medio?sentence=<frase>
```

Es: `/api/medio?sentence=ho+paura+del+buio`

## Response

```json
{
  "lemmas": ["avere", "paura", "buio"],
  "words": [
    {
      "word": "paura",
      "signature": [0.12, 0.85, 0.73, 0.45, 0.18, 0.62, 0.41, 0.05],
      "outgoing": [
        {
          "relation": "CAUSES",
          "target": "tremore",
          "confidence": 0.85,
          "target_signature": [...],
          "direction": "out"
        },
        {
          "relation": "IS_A",
          "target": "emozione",
          "confidence": 0.95,
          "target_signature": [...],
          "direction": "out"
        },
        {
          "relation": "OPPOSITE_OF",
          "target": "coraggio",
          "confidence": 0.78,
          "target_signature": [...],
          "direction": "in"
        }
      ]
    },
    ...
  ],
  "unknown": ["buio"]
}
```

## Campo `direction` (Phase 79 fix)

Aggiunto a `MedioEdgeDto`:
- `"out"`: l'arco va da `lemma` verso `target`
- `"in"`: l'arco va da `target` verso `lemma` (relazione entrante)

Prima di Phase 79 il campo era assente. Parole come "vita" che hanno relazioni solo come oggetto (`amore IS_A vita`, `nascita Causes vita`, …) finivano in `unknown` perché `all_outgoing(lemma)` era vuoto. Phase 79 ha esteso `build_medio_data_for_sentence` per chiamare anche `all_incoming(lemma)` e produrre edge con `direction: "in"`. Il client (`sentence.js`) controlla `e.direction === 'in'` per renderizzare l'arco in direzione inversa.

Verifica empirica:
- Prima di Phase 79: `vita` → `unknown=[vita]`, words vuoto
- Dopo Phase 79: `vita` → 28 out + **154 in** archi disponibili

## kg_aware_lemma — step 0 (Phase 79 fix)

`kg_aware_lemma(token, kg)` lemmatizza in modo KG-consapevole:

```
0. Se token è già nel KG → ritorna token così com'è (Phase 79)
1. Lemma formale via grammar::lemmatize (verbi)
2. Candidati plurale (-i → -o/-e/-a)
3. Candidati coniugazione presente (-ano → -are, -ono → -ere/-ire, …)
4. Primo candidato presente nel KG vince
5. Fallback al verbo formale o al token originale
```

**Step 0** è cruciale: senza, "sale" (sostantivo nel KG) veniva lemmatizzato a "sala" (camera) — entrambi presenti nel KG, ma la regola "-e fem.plur. → -a" è speculativa e va applicata SOLO se il token originale non è riconosciuto.

Verifica: `/api/medio?sentence=sale` → `lemmas: ['sale']` (non `['sala']`).

## Performance

Per una frase di 5 lemmi tipica:
- KG lookup: ~5 × (1 `all_outgoing` + 1 `all_incoming`) = ~10 query
- Each query: O(degree) sul HashMap di adjacency
- Total: ~50-200ms su KG da 83K archi

## Cura_save_kg + invalidazione cache

`/api/medio` è **read-only**: non muta il KG, non chiama `cura_save_kg`. Solo `/api/community/teach`, `/connect`, `/validate`, `/transmit_batch`, `/kg/confirm_edge`, `/kg/reject_edge` chiamano `cura_save_kg + biennale_cache = None`.

Phase 79 ha reintrodotto questo flow: vedi [knowledge graph semantico — persistenza](../topologia/knowledge-graph-semantico.md).

## See Also

- [Architettura campovasto](architettura-campovasto.md) — l'endpoint consumer
- [Knowledge graph semantico](../topologia/knowledge-graph-semantico.md)
- [Grammar](../generazione/grammar.md) — lemmatize
