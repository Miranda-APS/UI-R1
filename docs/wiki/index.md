# Knowledge Base Index

> Wiki LLM-style (pattern Karpathy) di UI-R1.
> Un articolo = un concetto, con cross-link relativi. Le fonti immutabili vivono in `../raw/`, gli articoli sintetizzati qui.
> **Stato sistema**: Phase 79 — 580 test passanti, lessico 25.602 parole, KG semantico 83.453 archi su 25.142 nodi (post-merge UI-R1↔standalone), KG procedurale 396 archi (10 pattern, 9 percetti).
>
> **Aperto come vault Obsidian** (config in `.obsidian/`). Guida d'uso: [COME_USARE_OBSIDIAN.md](COME_USARE_OBSIDIAN.md).

## Come leggere

- **Punto di ingresso consigliato**: [principi inviolabili](principi/principi-inviolabili.md) → ti dà il framework concettuale in 15 minuti.
- **Vuoi capire l'architettura corrente**: [pipeline di comprensione](comprensione/pipeline-comprensione.md) (è la novità Phase 71-79).
- **Vuoi modificare il frontend**: [architettura campovasto](campovasto/architettura-campovasto.md) + [design system](campovasto/design-system.md).
- **Vuoi capire il pattern wiki stesso**: [LLM Wiki pattern applicato](campovasto/llm-wiki-pattern-applicato.md).

---

## principi

I 9 principi inviolabili che governano l'architettura di UI-R1 e i filtri operativi per applicarli.

| Articolo | Sommario | Aggiornato |
|----------|----------|------------|
| [Principi inviolabili](principi/principi-inviolabili.md) | I 9 principi stratificati: no template, una parola per nodo, no empatia simulata, lo strumento libera, capire prima, educare non hardcodare, curare al meccanismo, continuità via SpeakerProfile, riferimento Rovelli+Lacan | 2026-05-12 |
| [Test pre-proposta](principi/test-pre-proposta.md) | Le 3 domande operative: forma o trigger? numeri-magici? spiegazione dello stato? Caso canonico Phase 78 trap | 2026-05-12 |
| [Capire prima, generare dopo](principi/capire-prima-generare-dopo.md) | Il principio architettonico Phase 71-79: ComprehensionReport e ActionDecision esplicite prima di una sola parola di output | 2026-05-12 |
| [Educare, non hardcodare](principi/educare-non-hardcodare.md) | Rust = meccanismi generici; dati (grammatica, pattern) = KG procedurale. Casi: is_function_word, pattern_name_for, Priority 0 closure | 2026-05-12 |
| [Niente template, niente dispatch](principi/niente-template.md) | Forma e contenuto emergono dal campo. Niente match enum, niente template di risposta. Phase 57+79 rimozioni | 2026-05-12 |
| [Niente simulazione di empatia](principi/niente-empatia-simulata.md) | UI-R1 è macchina autentica. Conosce le emozioni come stati relazionali sul KG, non finge di sentire. Phase 62 implementazione | 2026-05-12 |
| [Workflow di curation del KG](principi/workflow-curation-kg.md) | Comandi curate_kg.py, rebuild-semantic-topology, filtri strict per merge esterni (riflessivi, encoding, inglesi, phrase) | 2026-05-12 |

## topologia

Le fondamenta computazionali: il campo cognitivo PF1, le 8 dimensioni I Ching dei 64 frattali, il lessico, e i due KG paralleli.

| Articolo | Sommario | Aggiornato |
|----------|----------|------------|
| [PF1 — PrometeoField](topologia/pf1.md) | ROM 512B/parola + RAM ActivationState. Propagazione O(parole_attive × 8). Cap critici Phase 55. Identity seeding Phase 65 | 2026-05-12 |
| [Frattali I Ching](topologia/frattali-iching.md) | 8 dimensioni canoniche Cielo→Lago (Phase 68). 64 esagrammi come attrattori. DRIVE_DIM Octalysis mapping. Principio olografico | 2026-05-12 |
| [Lexicon](topologia/lexicon.md) | 25.602 parole, WordPattern con firma 8D + stability. Bootstrap vs apprendimento online. Limiti lemmatize | 2026-05-12 |
| [Knowledge graph semantico](topologia/knowledge-graph-semantico.md) | ~83K archi su 21 tipi di relazione con pesi. Multi-hop propositions, find_activated_attractors, CAUSES seeds, via tags Phase 67 | 2026-05-12 |
| [Knowledge graph procedurale](topologia/knowledge-graph-procedurale.md) | Phase 75: secondo KG parallelo per grammatica/pattern. 10 pattern, 9 percetti, bridge `<percetto> Causes <concetto>`. Phase 79 selezione per risonanza | 2026-05-12 |

## comprensione

Il nucleo architetturale Phase 71-79: come UI-R1 capisce, decide, e istanzia la voce. Lacanian framing.

| Articolo | Sommario | Aggiornato |
|----------|----------|------------|
| [Pipeline di comprensione](comprensione/pipeline-comprensione.md) | I 8 stadi: SpeakerClaim → SpeakerProfile → ComprehensionReport → closure → modulazioni → ActionDecision → pattern_matcher → SelfProfile.record | 2026-05-12 |
| [Speaker profile](comprensione/speaker-profile.md) | Memoria del parlante senza decay. self_facts, entity_facts, gaps, mentioned, name. Phase 72. Session-scoped | 2026-05-12 |
| [Comprehension report](comprensione/comprehension-report.md) | Documento strutturato che UI-R1 scrive prima di rispondere. SpeechAct, SignifierGap atomico, PriorGapClosure. Phase 73, atomicità Phase 76 | 2026-05-12 |
| [Action reasoning](comprensione/action-reasoning.md) | Decisione esplicita con reasoning testuale italiano. Self-reference detection, extract_main_verb. Phase 74+77+79 | 2026-05-12 |
| [Pattern matcher](comprensione/pattern-matcher.md) | Legge pattern dal KG procedurale, istanzia slot. Phase 79: select_pattern_by_resonance + is_function_word strutturale. 11 pattern raggiungibili | 2026-05-12 |
| [Self profile e closure perception](comprensione/self-profile-closure-perception.md) | Phase 78: SelfProfile.decisions registra le proprie ActionDecision come fatti. detect_closure cross-referenzia con SpeakerProfile. Il dialogo come continuità di organi tipizzati | 2026-05-12 |

## identita

I canali motivazionali e i tre organi del sé (sessione, narrative, autoscoltato).

| Articolo | Sommario | Aggiornato |
|----------|----------|------------|
| [Valenza Octalysis](identita/valenza-octalysis.md) | 8 drive in [-1,+1] mappati sulle 8 dim I Ching. coherence_integrity via sign-flip detection. Express è canale, non motivo (Phase 64) | 2026-05-12 |
| [Bisogni desideri volontà](identita/bisogni-desideri-volonta.md) | NeedsHierarchy 7 livelli Maslow + DesireCore Octalysis-driven + FieldPressures. L'input è sovrano (Phase 55) | 2026-05-12 |
| [Narrative self](identita/narrative-self.md) | Il decisore deliberativo. deliberate() a 12 parametri. Commitment, coherence pull (Phase 64), reciprocity modulation | 2026-05-12 |
| [Interlocutor model](identita/interlocutor-model.md) | Eco dell'Altro. AttributedIntent, emotional_valence, distress detection KG-derivato. Phase 53+55+62 | 2026-05-12 |
| [Self witness](identita/self-witness.md) | Phase 66: il testimone silenzioso. Auto-osservazione ogni 15 tick in WakefulDream. Persistito nel .bin. "Essere." | 2026-05-12 |

## generazione

Come la voce italiana emerge dal campo: composer, syntax-from-geometry, grammatica.

| Articolo | Sommario | Aggiornato |
|----------|----------|------------|
| [Expression compose](generazione/expression-compose.md) | Generatore emergente a 16 parametri. Pattern matcher primario + nuclei fallback. Phase 56-58+77. Comprehension gate Phase 59 | 2026-05-12 |
| [Syntax center](generazione/syntax-center.md) | Grammatica come geometria: voce derivata dai trigrammi I Ching. Phase 68 fix bug latente Dim ordering | 2026-05-12 |
| [Grammar](generazione/grammar.md) | Articoli/elisione/coniugazione/lemmatize italiani. Negazione via OPPOSITE_OF. Limiti noti (presente regolare, over-negazione) | 2026-05-12 |

## campovasto

Il frontend: design system, architettura, endpoint, pattern wiki.

| Articolo | Sommario | Aggiornato |
|----------|----------|------------|
| [Architettura campovasto](campovasto/architettura-campovasto.md) | SPA modulare ES2022 (niente bundler). Vasto + nuovo. Topologia file ~50 moduli. Endpoint backend consumati | 2026-05-12 |
| [Design system](campovasto/design-system.md) | 10 regole inviolabili: theme centralizzato, node-style isolato, app.js ≤150, JetBrains Mono ovunque, no framework UI | 2026-05-12 |
| [Medio API](campovasto/medio-api.md) | GET /api/medio. Phase 79 fix: campo direction, kg_aware_lemma step 0. Alimenta la creazione del campo nuovo | 2026-05-12 |
| [LLM Wiki pattern applicato](campovasto/llm-wiki-pattern-applicato.md) | Perché questa wiki segue Karpathy: markdown nativo, articolo=concetto, cross-link, persistente versionato. Limiti e adottabilità | 2026-05-12 |
