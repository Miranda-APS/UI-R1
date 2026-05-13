# LLM Wiki pattern applicato — perché questa wiki

> Sources: Francesco Mancuso, 2026-05-12 conv. + Andrej Karpathy, "llm-wiki" gist (2026)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

Questa wiki segue il **pattern Karpathy LLM-Wiki** ([gist originale](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f), aprile 2026): un'alternativa al RAG dove la knowledge base è una raccolta di markdown interlinkati che l'LLM **compila e mantiene** in modo evolutivo, e che noi **leggiamo e interroghiamo**. Su small-medium KB il token consumption scende fino al 95% rispetto a RAG, e l'accuratezza spesso aumenta perché il contesto è già curato.

Questo articolo spiega perché abbiamo adottato il pattern e come si applica a UI-R1.

## Il pattern in 5 punti

1. **Markdown è il formato nativo del contesto LLM**. Non vettori, non chunk, non embedding rovesciati: testo che l'LLM scrive bene e legge bene.
2. **Un articolo = un concetto**. Granularità semantica, non chunking arbitrario.
3. **Cross-link relativi**. Le `[Article Title](other.md)` sono "edges" navigabili dall'LLM senza embed.
4. **Persistente e versionato**. Git è la storia naturale; le modifiche sono diff leggibili, non re-indicizzazioni.
5. **L'LLM scrive, l'umano legge**. Inversione classica: l'LLM è il manutentore, l'umano il consumer.

## Perché si applica naturalmente a UI-R1

UI-R1 stesso è basato su un **KG ipertestuale**. La sua wiki di documentazione lo specchia:

| UI-R1 (sistema) | LLM-Wiki (docs) |
|-----------------|-----------------|
| Nodi parola con firme 8D | Articoli markdown con frontmatter |
| Archi IS_A / Causes / SimilarTo | `[link](relative.md)` |
| `find_activated_attractors` | Index gerarchico + cross-link |
| Curation manuale dei nodi | Wiki linting + ingest controllato |
| Phase = versione semantica | Git tag + diff |

L'utente non deve "credere" che la wiki sia accurata: può navigarla, fare diff, vedere il log delle operazioni.

## Struttura adottata

```
docs/
├─ wiki/
│  ├─ index.md             # mappa globale, una riga per articolo, raggruppato per topic
│  ├─ log.md               # log append-only: init, ingest, query archived, lint
│  ├─ principi/            # i 9 principi inviolabili + filtri operativi
│  ├─ topologia/           # PF1, frattali, lessico, KG semantico, KG procedurale
│  ├─ comprensione/        # Phase 71-79: pipeline, profile, report, action, pattern, closure
│  ├─ identita/            # valenza, bisogni, narrative, interlocutor, witness
│  ├─ generazione/         # expression, syntax, grammar
│  └─ campovasto/          # frontend: arch, design system, medio-api, llm-wiki (questo)
└─ raw/
   ├─ libretto/            # 23 capitoli storici (raw immutabili)
   ├─ contesto/CLAUDE_phase79.md
   ├─ architettura/        # ARCHITECTURE, FILOSOFIA, olografica, AUDIT P67, refactor logs
   └─ frontend/            # campovasto: CLAUDE, FRONTEND, regole di design, roadmap UX
```

> Nota: la cartella `docs/raw/futuro/` (8 documenti speculativi OS/hardware) è stata
> rimossa dal repo pubblico. Resta solo nel filesystem locale di Francesco (vedi
> `.gitignore`). La wiki documenta lo **stato attuale** del sistema, non roadmap
> speculative di lungo termine.

## Workflow d'uso

**Lettura (umano)**: parti da `wiki/index.md`, naviga al concetto. Ogni articolo ha "See Also" che porta altrove.

**Query (LLM)**: legge `wiki/index.md`, trova articoli rilevanti, cita con markdown link.

**Ingest (LLM)**: prende una fonte nuova, la salva in `raw/<topic>/YYYY-MM-DD-slug.md`, la compila in `wiki/<topic>/<concept>.md` (merge se concetto esiste, crea altrimenti). Cascade update agli articoli correlati. Append a log.

**Lint (LLM, mensile)**: deterministico (link rotti, raw references, orphan pages) + euristico (contraddizioni, claim outdated). Auto-fix sicuri, report umani per il resto.

## Cosa si guadagna rispetto a docs tradizionali

**Velocità di onboarding**: chi atterra qui legge `principi/` e ha il framework mentale in 15 minuti. Tradizionalmente serve scorrere 22 capitoli del libretto.

**Token efficiency**: un LLM coding agent può leggere 3-4 articoli targeted per rispondere a una domanda specifica su UI-R1, invece di ingerire l'intero CLAUDE.md da 2000+ righe.

**Drift detection**: il lint trova contraddizioni tra articoli quando il sistema evolve. Senza wiki, le docs decadono silenziosamente.

**Cascade updates**: una Phase nuova → si tocca un articolo, il cascade automatico aggiorna gli articoli correlati (e li segna come bisognosi di review umana se l'LLM è incerto).

## Limiti del pattern

Karpathy stesso nota: RAG vince quando il KB è troppo grande per il context (centinaia di K tokens). UI-R1 wiki è oggi ~60KB, ampiamente dentro il context anche dei modelli piccoli. Se cresce oltre i 200K token (~50 articoli da 4KB ciascuno), si valuterà chunking + RAG.

## See Also

- [Architettura campovasto](architettura-campovasto.md) — il frontend che ha ispirato la riflessione
- [Design system](design-system.md) — le regole di design sono articoli wiki
- [Principi inviolabili](../principi/principi-inviolabili.md) — il punto di entrata della wiki
- Gist originale: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
- OmegaWiki (implementazione completa del pattern): https://github.com/skyllwt/OmegaWiki
- Skill astro-han/karpathy-llm-wiki: https://github.com/astro-han/karpathy-llm-wiki
