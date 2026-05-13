# Wiki Log

## [2026-05-12] init | bootstrap wiki UI-R1 Phase 79
Wiki inizializzata sotto `docs/` con pattern Karpathy LLM-Wiki. Skill `karpathy-llm-wiki`
installata localmente (astro-han, 802 stars). Layout: `docs/raw/` per fonti immutabili,
`docs/wiki/` per articoli compilati. Index + log root.

## [2026-05-12] ingest | libretto storico (22 capitoli)
Spostato il libretto 22 file (`00_come_leggere.md` → `19_calcoli.md`, `99_considerazioni.md`,
`100_cosa_potrebbe_essere.md`, `appunti.md`) da `docs/libretto/` a `docs/raw/libretto/`.
Sorgente storica per la wiki nuova; preservato verbatim.

## [2026-05-12] ingest | CLAUDE.md (Phase 79)
Copiato `CLAUDE.md` corrente a `docs/raw/contesto/CLAUDE_phase79.md`. È la fonte primaria
per la wiki: contiene principi inviolabili, invariati critici, mappa file critici, descrizione
dello stato Phase 79.

## [2026-05-12] ingest | docs/architettura
Spostati a `docs/raw/architettura/`: `ARCHITECTURE.md`, `FILOSOFIA.md`, `architettura_olografica.md`,
`ARCHITECTURAL_AUDIT_P67.md`, `POSIZIONAMENTO.md`, e i 4 doc Phase 69 in `refactor/`.

## [2026-05-12] ingest | docs/Futuro/ (8 doc OS)
Spostati a `docs/raw/futuro/`: `PROMETEO_OS_CIRCUIT_DESIGN`, `PROMETEO_OS_FIRMWARE_SPEC`,
`PROMETEO_OS_HARDWARE`, `PROMETEO_RESONANT_COMPUTER_PROPOSAL`, `TECHNICAL_DEEP_ANALYSIS` (parts 1-3),
`PRESENTAZIONE_CIRCOSCRIZIONI_TORINO`.

## [2026-05-12] ingest | campovasto/*.md (best practice frontend)
Copiati a `docs/raw/frontend/`: `CLAUDE_campovasto.md`, `FRONTEND.md`, `regole_di_design.md`,
`roadmap_UX.md`. Sorgenti delle regole inviolabili campovasto.

## [2026-05-12] ingest | compile | principi (6 articoli)
Compilati: `principi-inviolabili`, `test-pre-proposta`, `capire-prima-generare-dopo`,
`educare-non-hardcodare`, `niente-template`, `niente-empatia-simulata`, `workflow-curation-kg`.
Fonte primaria: CLAUDE_phase79.

## [2026-05-12] ingest | compile | topologia (5 articoli)
Compilati: `pf1`, `frattali-iching`, `lexicon`, `knowledge-graph-semantico`,
`knowledge-graph-procedurale`. Fonte primaria: CLAUDE_phase79 + libretto cap. 02-05.
- Updated: `educare-non-hardcodare` (link a kg-procedurale)
- Updated: `niente-template` (link a kg-procedurale + pattern-matcher)

## [2026-05-12] ingest | compile | comprensione (6 articoli)
Compilati: `pipeline-comprensione`, `speaker-profile`, `comprehension-report`,
`action-reasoning`, `pattern-matcher`, `self-profile-closure-perception`.
È il topic più denso: copre Phase 71-79.
- Updated: `capire-prima-generare-dopo` (cross-link a tutti i nuovi articoli)
- Updated: `principi-inviolabili` (link a speaker-profile)

## [2026-05-12] ingest | compile | identita (5 articoli)
Compilati: `valenza-octalysis`, `bisogni-desideri-volonta`, `narrative-self`,
`interlocutor-model`, `self-witness`. Coprono Phase 47-67.

## [2026-05-12] ingest | compile | generazione (3 articoli)
Compilati: `expression-compose`, `syntax-center`, `grammar`. Coprono Phase 56-77.

## [2026-05-12] ingest | compile | campovasto (4 articoli)
Compilati: `architettura-campovasto`, `design-system`, `medio-api`,
`llm-wiki-pattern-applicato`. Fonte: campovasto/CLAUDE.md + FRONTEND.md + regole di design.md.

## [2026-05-12] update | index.md + log.md
Index globale con 29 articoli su 6 topic. Log con cronologia degli ingest.

## [2026-05-13] structure | rimosso futuro/ dalla wiki
Spostato `docs/raw/futuro/` (8 doc speculativi OS/hardware) a `roadmap_futuro/` al livello del
progetto. Motivazione: la wiki documenta **lo stato attuale**, non le roadmap speculative.

## [2026-05-13] cleanup | rimosso dal repo pubblico
- `roadmap_futuro/`: untracked + gitignored. Resta solo locale.
- `campovasto-mobile/` (variante mobile non mantenuta): untracked + gitignored.
- `src/web/server.rs`: rimosso endpoint `/campovasto-mobile` orfano.

Restano nel repo: `books/` (3 .txt usati da `read-books` binary, dominio pubblico) e
`fractals/` (96 SVG I Ching usati da `fractal_visuals.rs` + endpoint web).

## [2026-05-13] update | metriche allineate al post-merge KG
- `index.md`: header "Stato sistema" riallineato (lessico 25.602, KG 83.453 archi su 25.142 nodi,
  KG procedurale 396 archi).
- `topologia/lexicon.md`: corretta differenza lessico/nodi KG (era citato "4.166 su 25.875",
  ora "~460 su 25.602: 25.602 lessico − 25.142 nodi KG post-merge").
- `campovasto/llm-wiki-pattern-applicato.md`: rimossa voce `futuro/` dalla descrizione struttura.

## [2026-05-13] note | libretto in `docs/raw/libretto/` allineato fino a Phase 68
Audit del libretto (23 capitoli, ~12K righe): Vol. 00 e 01 ancorati a Phase 63-68. Phase 71-79
(ciclo della comprensione: speaker_profile / comprehension_report / action_reasoning / KG
procedurale / pattern_matcher / self_profile / kg_proc_field) **non documentata nel libretto**.
Fix mirati applicati ai 5 punti più obsoleti + aggiunto capitolo nuovo
`20_ciclo_comprensione.md` per Phase 71-79. Il libretto resta sorgente primaria;
gli articoli wiki "comprensione/" sono già allineati alla Phase 79.

## [2026-05-13] vault | aggiunta configurazione Obsidian
Creato `docs/wiki/.obsidian/` con config minimale per aprire la wiki come vault Obsidian.
Graph view + backlinks + tag pane attivi. Guida d'uso in `docs/wiki/COME_USARE_OBSIDIAN.md`.
