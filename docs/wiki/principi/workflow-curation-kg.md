# Workflow di curation del KG

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79; conversazione audit KG)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md); [04_fondamenti_kg](../../raw/libretto/04_fondamenti_kg.md)

## Overview

Il [KG semantico](../topologia/knowledge-graph-semantico.md) di UI-R1 ha ~83.000 archi e 25.000 nodi, curati nel corso di mesi. La curation segue regole strette: ancorata al meccanismo, evita dead-weight, unifica al lemma. Questo articolo è la guida operativa al workflow.

## Principio: curare ancorato al meccanismo

> Aggiungi al KG SOLO quello che serve a un meccanismo esistente o a un pattern che stai introducendo. Mai "potrebbe servire un giorno" — è dead-weight.

Esempi positivi:
- `discursive_knowledge.tsv` aggiunto in Phase 67 per supportare il rilevamento di proprietà discorsive (certezza/incertezza/apertura/chiusura/soggettività) nel ComprehensionReport. Ogni tripla serve uno specifico hop dell'inferenza.
- `agent_kg.tsv` (17.711 IS_A) aggiunto per popolare le mega-categorie I Ching: nodi che servono come attrattori IS_A per `find_activated_attractors()`.

Esempi negativi (esclusi):
- `agent_kg_full.tsv.excluded` — 62K archi CAUSES/PART_OF/USED_FOR generati da Qwen3 senza ground_word exact-match. Rumorosi, scartati.
- `curation_export.tsv.excluded` — 128K archi auto-generati non verificati.

## Convenzioni di forma

**Una parola sola per nodo.** Mai `pronome_interrogativo` come unico nodo. Concetti composti vivono come catene IS_A: `cosa IsA pronome` E `cosa IsA interrogativo`. Vedi [principio 2](principi-inviolabili.md).

**Lemma all'infinito per i verbi.** `affrancarsi` → `affrancare`, `definirsi` → `definire`. I riflessivi sono unificati. Le forme flesse non sono nodi.

**Italiano puro.** Parole inglesi (`partnership`, `feeling`, `outsourcing`), nomi propri di brand, forme con caratteri non-italiani (`autoritã` da corruption UTF-8) sono escluse.

**Phrase composti** (es. `direttore d'orchestra`) tendono a non essere nodi: l'atomicità del nodo è regola.

## Tipi di relazione (21)

Vedi [knowledge graph semantico](../topologia/knowledge-graph-semantico.md) per la lista completa. Categorie funzionali:

- **Strutturali**: `IsA`, `PartOf` — gerarchia categoriale (peso 0.9 / 0.8)
- **Causali**: `Causes`, `Enables`, `Requires`, `Implies` — propagazione orientata (peso 1.0 / 0.85 / 0.85 / 0.8)
- **Sinonimia/contrasto**: `SimilarTo` (peso 0.4 — basso perché 118K archi rumorosi), `OppositeOf` (peso 0.7), `Equivalent` (peso 0.95)
- **Funzionali**: `UsedFor`, `Does`, `Has` — azione/possesso (peso 0.8 / 0.9 / 0.85)
- **Fenomenologiche**: `FeelsAs`, `Expresses`, `Symbolizes`, `ContextOf` — sfumature sensoriali/simboliche
- **Procedurali** (KG procedurale): `Causes` come bridge `<percetto> → <concetto>`, `UsedFor X via Y` per slot dei pattern

## Workflow operativo

**1. Ispezione (read-only).**
```bash
python data/external/nightly_diagnostics.py --output report_kg.md
```
Hub, orfani, componenti connesse, distribuzione gradi.

**2. Curation file master.**
`curate_kg.py` (idempotente). Modifica sezioni §1-§76 (~120 parole centrali). Edita `prometeo_kg.json` direttamente.
```bash
python curate_kg.py --dry-run    # preview
python curate_kg.py              # applica e salva
```

**3. Rebuild topologia semantica.**
```bash
cargo run --release --bin rebuild-semantic-topology
```
**MAI** `import-kg` dopo curate_kg.py (sovrascriverebbe la curation).

**4. Arricchimento confidence (background, opzionale).**
```bash
python data/external/enrich_confidence.py --resume
```
Usa Qwen3 via Ollama (esterno, offline) per stimare confidence per-arco. Phase 48: ogni boost = `field_boost_strength × confidence_arco`.

**5. Persistenza KG su file (Phase 79 fix).**
Le mutazioni del KG fatte via `/api/community/teach`, `/transmit_batch`, `/kg/confirm_edge` chiamano `cura_save_kg()` che scrive atomicamente (tmp + rename) su `prometeo_kg.json`. Il `.bin` NON contiene il KG (caricato da `prometeo_kg.json` a ogni boot). Vedi articolo audit P67 in raw.

## Filtri di sicurezza per merge esterno

Quando si fonde un KG esterno (es. fork divergente), applicare filtri **strict** prima del merge:

- **Encoding strict**: whitelist `[a-z'-]` + accenti italiani `àèéìíòóùú`. Mai fallback `c.isalpha()` (entrano `ã`, `ñ`, ecc.)
- **Riflessivi unificati**: `-arsi → -are`, `-ersi → -ere`, `-irsi → -ire`, `-orsi → -orre`, `-ursi → -urre`. Skip se l'infinito è già presente.
- **Inglesi**: blacklist comuni + suffissi `-tion` (eccetto `-azione/-zione/-uzione/-izione`), `-ing`, `-ness`, `-ment`, `-ship`.
- **Phrase**: skip se contiene spazio.

Esempio (Phase 79 merge UI-R1 → prometeo_standalone): 5408 nodi candidati → 3951 accettati. Filtrati 1158 riflessivi, 243 encoding errors, 12 inglesi, 59 phrase.

## See Also

- [Principi inviolabili](principi-inviolabili.md) — principio 7
- [Knowledge graph semantico](../topologia/knowledge-graph-semantico.md)
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md)
- [Educare, non hardcodare](educare-non-hardcodare.md)
