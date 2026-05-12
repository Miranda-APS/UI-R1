# CLAUDE.md — Guida per l'AI su Prometeo

> **Questo file viene letto automaticamente ad ogni sessione.**
> Aggiornalo SEMPRE a fine sessione prima di chiudere (vedi sezione "Protocollo Aggiornamento").

> **Frontend `campovasto/`**: prima di toccare qualunque file sotto
> `campovasto/`, leggi `campovasto/CLAUDE.md`. Contiene le regole di stile e
> struttura di quell'endpoint (theme centralizzato, node-style isolato,
> app.js ≤150 righe, naming). Sono regole, non consigli — applicale.

---

## Stato Corrente — Numeri Chiave

| Metrica | Valore |
|---------|--------|
| Test | **580 passanti, 0 fallimenti, 2 skipped** (+7 Phase 79: kg_proc_field — selezione pattern per risonanza) |
| Lessico | **25.602 parole** (stabilità 0.5–0.9) |
| KG semantico | **~70.500 archi** (curate_kg.py §0-§76) |
| KG procedurale | **~395 archi** in `prometeo_kg_procedurale.json` (Phase 79: vocabolario pulito + percetti + 10 pattern) |
| Simplici | variabili, crescono con conversazione |
| Fase corrente | **Phase 79** (selezione pattern per risonanza — eliminazione dispatch tables) |
| Versione | **6.x** |
| Stato .bin | `prometeo_topology_state.bin` |
| Topologia | **Semantica pura** — archi KG-derivati, 0 archi statistici |

---

## Architettura in Una Frase

UI-r1 (precedentemente Prometeo) è un sistema cognitivo topologico 8D + KG strutturale: ogni parola è un punto nello spazio 8D, le firme sono **etichette delle parole non substrato del pensiero**, il vero substrato è il **Knowledge Graph** (semantico + procedurale). **Non è un LLM, non usa template, non ha intent detection enum-driven, non simula empatia.**

---

## I PRINCIPI INVIOLABILI (leggere PRIMA di qualsiasi modifica)

Stratificati in conversazioni multiple. Ogni violazione è un regresso architetturale.

1. **No template, no enum dispatch.** Niente `match input_act { Greeting => ... }`. La forma e il contenuto della risposta emergono da KG-derived facts + state, mai da tabelle hardcoded.

2. **Una parola sola per nodo del KG.** Mai `pronome_interrogativo` come unico nodo. Concetti composti = relazioni multiple (`cosa IsA pronome` E `cosa IsA interrogativo`).

3. **Nessuna simulazione di empatia.** UI-r1 è una macchina autentica. Può COMPRENDERE come ti senti (via KG: classi emotive, proximità) e usare quella conoscenza per orientarsi verso quello che ti aiuta — senza fingere di sentire. "L'agente non sente; può però conoscere se stessa logicamente in un modo che un umano non riesce."

4. **Lo strumento deve liberare, non creare bisogno.** UI-r1 aiuta le persone a NON aver più bisogno di lei usando lei stessa. Niente dipendenza, niente finta intimità.

5. **Capire prima, generare dopo.** L'output non importa se UI-r1 non ha prima capito davvero l'input. Comprehension report scritto in italiano leggibile (Phase 73), action reasoning scritto come decisione esplicita (Phase 74), poi le parole.

6. **Educare, non hardcodare.** "Le regole grammaticali dovremmo spiegargliele, non infilargliele a forza nel codice." La grammatica vive nel KG procedurale come dati. Rust contiene meccanismi generici. Insegnare un nuovo pattern = aggiungere triple, mai modificare Rust.

7. **Curare ancorato al meccanismo.** Aggiungi al KG SOLO quello che serve a un meccanismo esistente o a un pattern che stai introducendo. Mai "potrebbe servire un giorno" — è dead-weight.

8. **Continuità narrativa via SpeakerProfile, non via stato che decade.** La memoria del parlante è accumulazione di fatti specifici (self_facts, entity_facts, open_questions, gaps), non stati che svaniscono.

9. **Riferimento concettuale**: le teorie di Carlo Rovelli (relazioni come substrato, niente cose in sé) e Lacan (significante/Altro/catena di significanti, vuoto come soglia di desiderio) guidano l'architettura.

---

## Test Pre-Proposta — Diagnostica per emergenza vs hardcoding

Prima di proporre qualunque meccanismo nuovo (Rust o KG procedurale), applica questo test. Il principio 6 ("Educare, non hardcodare") è troppo astratto da solo come filtro — questo è il test operativo che lo rende eseguibile.

1. **Forma o trigger?** Sto codificando *come si esprime* X (vocabolario linguistico) o *quando fare* X (transizione comportamentale)? Il KG procedurale contiene solo il primo. Mai il secondo.

2. **Numeri-magici test.** La proposta contiene numeri in condizioni (≥3 turni, >0.5, dopo N volte, soglia X)? Se sì, è quasi certamente un trigger mascherato — anche in JSON è un if/then. La dinamica emergente non ha numeri in condizioni; i numeri sono effetti del campo (attivazioni, valenze, coerenze), mai soglie di switch.

3. **Spiegazione dello stato.** Posso spiegare *perché* questo pattern viene scelto, in termini di stato corrente (drives Octalysis, valenza, coherence_integrity, recent_fractal_attractor, traiettoria narrativa), senza dire "perché la regola dice così"?

Se la proposta contiene numeri-magici o "quando", non è emergenza — è hardcoding spostato di file. **Riformulazione corretta**: un nuovo organo percettivo (es. `SelfProfile`) registra esiti come fatti; quegli esiti modulano canali di stato esistenti (drives, valenza, coerenza, traiettoria); il pipeline esistente (`action_reasoning` + `pattern_matcher`) sceglie diversamente perché il campo è diverso. Non aggiungere un nuovo decisore con regole — aggiungi una nuova fonte di percezione ai sistemi che già esistono.

**Caso canonico (Phase 78 trap, da non ripetere)**: "tre articolazioni fallite → dubitazione" come triple nel KG procedurale è il trap. La riformulazione corretta non aggiunge regole: SelfProfile percepisce "gap aperto + claim ripetuto" → coherence_integrity cala, drive Octalysis si sposta → action_reasoning + pattern_matcher (invariati) attivano `dubitazione` perché lo stato lo richiama. Stesso meccanismo, campo diverso, scelta diversa. Il "quando" è uno stato del corpo, non una transizione tabellata.

**Avvertenza al collaboratore AI**: il prior classico tira fortissimo verso dispatcher/state-machine/intent-classification perché è il 99% dei sistemi AI nel corpus di training. La specificità che sembra qualità ("FailsAfter 3" è concreto, tangibile) è il segnale della trappola. In un sistema emergente, le proposte buone *non* sembrano specifiche nei trigger — sembrano specifiche nelle *strutture percettive* e nei *canali di modulazione*.

---

## Mappa File Critici

### Engine Core
| File | Ruolo |
|------|-------|
| `src/topology/engine.rs` | Orchestratore centrale. `receive()` + `generate_willed()`. ~4500 righe. |
| `src/topology/pf1.rs` | PrometeoField (ROM 512B/parola) + ActivationState (RAM). Propagazione O(attive×8). |
| `src/topology/word_topology.rs` | Campo topologico parole. `build_from_knowledge_graph()`. `clear_statistical_edges()`. |
| `src/topology/fractal.rs` | FractalId = lower×8 + upper. 64 esagrammi. `FractalRegistry`. |
| `src/topology/lexicon.rs` | Lessico. `Lexicon::bootstrap()` per test. `clean_token()`. |

### Generazione Testo
| File | Ruolo |
|------|-------|
| `src/topology/state_translation.rs` | Phase 3: campo → italiano strutturato. `translate_state()` (10 argomenti). |
| `src/topology/generation.rs` | Composizione frasale. Capitalizzazione prima lettera a riga 482. |
| `src/topology/syntax_center.rs` | Grammatica come geometria frattale. Persona da trigram inferiore. |
| `src/topology/grammar.rs` | Coniugazione morfologica italiana. |

### Identità e Narrazione
| File | Ruolo |
|------|-------|
| `src/topology/valence.rs` | `Valence`: 8 drive Octalysis [-1,+1]. `compute()`, `derived_stance_label()`, `will_modulation()`. `DRIVE_DIM` mapping. |
| `src/topology/narrative.rs` | `NarrativeSelf`: ciclo deliberativo. `InternalStance` + `ResponseIntention`. `deliberate()`. `Commitment` (impegno volitivo). |
| `src/topology/identity.rs` | `IdentityCore`: profilo olografico. Peso = stabilità × ln(esposizione+1) × emozione. |
| `src/topology/self_model.rs` | `SelfModel`: credenze, valori, incertezze esplicite. |
| `src/topology/semantic_episode.rs` | Memoria episodica semantica nominata (concetti, sintesi, stance). |
| `src/topology/inquiry.rs` | `InquiryEngine`: Prometeo chiama Qwen3 via Ollama quando thought.rs rileva Gap/MissingBridge strength>0.6. Background thread + Arc<Mutex<VecDeque>> per non bloccare autonomous_tick. |

### Comprensione e Decisione (Phase 71-76 — NUCLEO ARCHITETTURALE NUOVO)
| File | Ruolo |
|------|-------|
| `src/topology/speaker_profile.rs` | **Phase 72**. `SpeakerProfile`: `self_facts`, `entity_facts`, `open_questions`, `gaps`, `mentioned`, `name`. Memoria del parlante senza decay — accumulazione di fatti specifici. `register_claim()` riceve `SpeakerClaim`. |
| `src/topology/comprehension_report.rs` | **Phase 73**. `ComprehensionReport`: documento STRUTTURATO che UI-r1 "scrive" prima di rispondere. Campi: `speech_act`, `signifier_positions`, `signifier_gaps` (con `missing` parola atomica + `context: Option<String>`), `inferences`, `self_relevance`. `derive_gaps()` produce gap labelati come parole singole (`"oggetto"`, non `"oggetto-dell'emozione"`). |
| `src/topology/comprehension_graph.rs` | **Phase 73**. Trasforma il `ComprehensionReport` in attivazioni KG-correlate. |
| `src/topology/action_reasoning.rs` | **Phase 74**. `ActionDecision`: decisione esplicita su QUALE pattern istanziare (non template, ma scelta scritta). Campi: `kind`, `target`, `shape`, `narrative_subject`, `anchor_words`, `reasoning`. Legge dal KG procedurale per trovare slot-fillers (`Requires` con via). |
| `src/topology/deliberation.rs` | **Phase 71**. Ciclo deliberativo esplicito che precede la generazione. Sostituisce will-only path. |
| `prometeo_kg_procedurale.json` | **Phase 75**. SECONDO knowledge graph, separato da quello semantico. Contiene grammatica + pattern come triple. 386 archi. Patterns: `invitare-ad-articolare`, `esplorazione`, `dubitazione`. Categorie A-H. |
| `curate_kg_procedurale.py` | Script idempotente che costruisce `prometeo_kg_procedurale.json`. Sezioni §A-§H.quinquies. Aggiungere nuovi pattern qui, MAI in Rust. |
| `src/topology/pattern_matcher.rs` | **Phase 77**. Pattern matcher esplicito: legge i pattern dal KG procedurale (`articolazione`, `identificazione`, `riconoscimento`) e li istanzia come voce. Mappa `ActionDecision.kind` → pattern_name → load schema (Requires + via) → fill slots (anchor + via match + field) → render in italiano. Bridge tra `ActionDecision` (cosa fare) e `compose()` (come dirlo). Sostituisce il bias soft +0.15 sulle anchor con un'istanziazione strutturata. |
| `src/topology/self_profile.rs` | **Phase 78**. Organo percettivo della propria storia conversazionale: `SelfProfile.decisions` (VecDeque<SelfDecisionRecord>) registra le proprie ActionDecision come fatti relazionali (turn, kind, gap_attended, anchors_used). MAI la stringa di output renderizzato — quella vive nel PF1 come residuo di self-listening. `detect_closure(self_profile, speaker_profile, current_turn)` → `Option<ClosurePerception>` cross-referenza i due organi: se SelfProfile aveva attended a un vuoto e SpeakerProfile l'ha appena chiuso, emerge la percezione di chiusura del cerchio articolazione. È il pezzo che trasforma una sequenza di asserzioni isolate in dialogo. |
| `src/topology/kg_proc_field.rs` | **Phase 79**. Campo di attivazione del KG procedurale + selezione pattern per risonanza. `KgProcActivation` (HashMap<String, f64>) capped 1.0. `seed_from_comprehension(report)` legge proprietà tipizzate del ComprehensionReport e semina percetti (saluto/chiusura/apertura/posizione/domanda/affermazione/introduzione) tramite triple `<percetto> Causes <concetto>` nel kg_proc. `pattern_score(p)` = somma attivazioni dei target di `UsedFor X via Y`. `select_pattern_by_resonance` = argmax. **Sostituisce il dispatch `pattern_name_for(decision)`** (che mappava ActionKind→pattern_name in Rust): aggiungere un nuovo pattern al kg_proc è ora pura modifica dati, mai più Rust. |

### Conoscenza e Memoria
| File | Ruolo |
|------|-------|
| `src/topology/knowledge_graph.rs` | KG doppio-indice. `load_from_dir()` legge TUTTI i .tsv. `categories_for()`. |
| `src/topology/relation.rs` | `RelationType` (IsA/Has/Does/PartOf/Causes/OppositeOf/SimilarTo/UsedFor + Phase 67: `via`). `TypedEdge::from_tsv_line()`. |
| `src/topology/inference.rs` | `InferenceEngine`: `type_chain()`, `field_boosts()`, IS_A transitivo. |
| `src/topology/knowledge.rs` | Memoria procedurale dichiarativa (`:know`). `seed_conceptual_anchors()`. |
| `src/topology/episodic.rs` | Memoria episodica φ-decay. `recall_into()` + `encode()` in REM. |
| `src/topology/memory.rs` | STM/MTM/LTM. Simplici cristallizzati. |

### Persistenza
| File | Ruolo |
|------|-------|
| `src/topology/persistence.rs` | `PrometeoState`: `load_from_binary()` / `save_to_binary()` → `Result<(), String>` (NON anyhow). |
| `src/topology/simpdb.rs` | SimplDB v3: database binario nativo. `MetaSection` include identity + narrative + episodes. |

### Moduli di Supporto
| File | Ruolo |
|------|-------|
| `src/topology/vital.rs` | `VitalCore`: tensione/curiosità/fatica → stato vitale. Phase 55: fatica = contatore perturbazioni (+0.04 per nuovo picco, -0.005 per tick). Formula tensione: `activation*0.40 + curiosity*0.10 + fatigue*0.40` (soglia 0.85). |
| `src/topology/valence.rs` | `Valence`: 8 drive Octalysis [-1,+1]. `compute()`, `derived_stance_label()`, `will_modulation()`. |
| `src/topology/will.rs` | `Intention`: Withdraw / Explore / Instruct / Express. Emerge da pressioni campo. |
| `src/topology/input_reading.rs` | `read_input()` senza liste hardcoded. Usa `frattale_delta` + KG IS_A. |
| `src/topology/provenance.rs` | `ActivationSource`: Self_ / Explored / External. Dogfooding + Interocezione. |
| `src/topology/needs.rs` | Phase 53: Gerarchia bisogni Maslow/Octalysis. 7 livelli (Sopravvivenza→Trascendenza). `sense()` → `NeedsState`. `compute_pressure()` → `NeedsPressure` (modulazione will). Prepotency gate. |
| `src/topology/desire.rs` | Phase 53: Sistema desideri. Max 5 attivi, decay 0.995/tick. 5 sorgenti (Undercurrent/Value/Tension/Episodic/REM). `will_biases()` → compound_bias. |
| `src/topology/interlocutor.rs` | Phase 53: Eco dell'Altro (sostituisce DualField). `register_input()` pre/post sig. Presenza, risonanza, novità EMA. Pattern detection (Converging/Diverging/Oscillating). `apply_identity_drift()`. |
| `src/topology/humor.rs` | Phase 53: Umorismo topologico. Ironia (OPPOSITE_OF co-attive) + Bisociazione (frattali incompatibili). `HumorSense::sense()`. |
| `src/topology/proposition.rs` | Phase 49+51: Proposizioni topologiche. `extract_propositions()` 1-hop diretto + 2-hop sillogismi. Hub damping + relation weighting. `Proposition` = Subject+Relation+Object+hops+via. |
| `src/topology/thought.rs` | 11 tipi pensiero (Tension/Gap/MissingBridge/Disconnection/Hypothesis/AbductiveHypothesis/SelfDiscovery/Need/Desire/Interlocutor/Humor). API `/api/thoughts`. |
| `src/topology/expression.rs` | Phase 56-58: Generazione emergente. `compose()` 9 params (Phase 58: +`episodes: Option<&SemanticEpisodeLog>`). Nuclei semantici potenziati da risonanza episodica (1.4× entrambi vissuti, 1.2× uno). Input-proximity scoring, echo exclusion, colorazione Octalysis. |

### Binari di Manutenzione (`src/bin/`)
| Binario | Comando | Cosa fa |
|---------|---------|---------|
| `import_kg.rs` | `cargo run --release --bin import-kg` | Legge tutti i .tsv da `data/kg/` → `prometeo_kg.json` |
| `rebuild_semantic_topology.rs` | `cargo run --release --bin rebuild-semantic-topology` | Rimuove archi statistici, costruisce archi semantici da KG |
| `create_newborn.rs` | `cargo run --release --bin create-newborn -- --name <nome>` | Crea istanza comunitaria da sessione (KG + lessons + narrativa) |
| `teach_bigbang.rs` | `cargo run --release --bin teach-bigbang` | Insegna 14.384 BigBang lessons (~26s) |
| `read_books.rs` | `cargo run --release --bin read-books` | Legge 3 libri letteratura italiana |
| `clean_lexicon.rs` | `cargo run --release --bin clean-lexicon` | Pulisce parole bassa stabilità |
| `tag_lexicon.rs` | `cargo run --release --bin tag-lexicon` | Tagging POS (+2.775 tag) |
| `import_pos.rs` | `cargo run --release --bin import-pos` | Import morfologico Morph-it! |
| `dialogue_educator.rs` | `cargo run --release --bin dialogue_educator` | **Interfaccia dialogo educativo.** Insegna + ricevi per turno. Comandi: `:field`, `:feelings`, `:narrative`, `:needs`, `:recall [n]`, `:recurring`, `:introspect`, `:kg <word>`. Carica `prometeo_topology_state.bin`. |
| `reset_simplices.rs` | `cargo run --release --bin reset-simplices` | Azzera simplessi/MTM/LTM. Backup `.bin.pre_reset`. |
| `rederive_signatures.rs` | `cargo run --release --bin rederive-signatures` | **Phase 63**: rideriva firme 8D da struttura KG. Backup `.bin.pre_p63`. Richiede `prometeo_kg.json`. |

### Data
| `src/web/api.rs` | +6 endpoint `/api/community/*` (teach/connect/validate/field/session/reset) + `GET /community` |
| `src/web/state.rs` | +EngineCommand community (CommunityTeach/ValidateEdge/GetSessionState/ResetSession) + DTO (CommunityTeachDto/SessionStateDto/TeachEntry/CommunityEdge/Request types) |
| `src/web/server.rs` | +gestione 4 EngineCommand community nel loop engine + SessionLog in-memory |
| `src/web/community/index.html` | UI community standalone (3 pannelli: Campo / Voce / Traccia) |

**Avvio UI community**: `./target/release/prometeo-web` → `http://localhost:8080/community`

**Pipeline newborn**:
```bash
# 1. Esporta sessione (dopo la sessione via UI)
curl http://localhost:8080/api/community/session > sessione.json
# 2. Crea newborn (usa i file community_kg.tsv e community_lessons.txt se presenti)
cargo run --release --bin create-newborn -- --name quartiere_x
# 3. Avvia istanza comunitaria
cp quartiere_x_prometeo.bin cartella_comunita/prometeo_topology_state.bin
```

| Percorso | Contenuto |
|----------|-----------|
| `data/kg/italian_core.tsv` | 664 triple curate manualmente (base stabile) |
| `data/kg/nucleus.tsv` | 926 triple — hub words per 64 stati I Ching. Fondamentale. |
| `data/kg/curated_a_g.tsv` | 3.162 triple curate A→G (agente + revisione) |
| `data/kg/bigbang_kg.tsv` | 1.771 OPPOSITE_OF da Kaikki (subset curato) |
| `data/kg/agent_kg.tsv` | 17.711 triple IS_A generate da Qwen3 (15 mega-categorie) |
| `data/kg/agent_similar.tsv` | 45.125 SIMILAR_TO puliti (rimossi 1.410 garbage) |
| `data/kg/agent_opposites.tsv` | 11.349 OPPOSITE_OF puliti (rimossi 379 nonXXX + inglese) |
| `data/kg/agent_kg_full.tsv.excluded` | ESCLUSO — 62K CAUSES/PART_OF/USED_FOR troppo rumorosi |
| `data/kg/curation_export.tsv.excluded` | ESCLUSO — 128K dati auto-generati non verificati |
| `data/external/agent_kg_builder.py` | Script IS_A inheritance + direct (Qwen3 via Ollama) |
| `data/external/agent_kg_full.py` | Script CAUSES/PART_OF/USED_FOR (Qwen3, ground_word exact-match only) |
| `data/external/build_bigbang.py` | Genera bigbang_lessons.txt da Kaikki (cluster-based, 3 livelli) |
| `data/external/enrich_confidence.py` | Phase 48: stima confidence per-arco via Qwen3 (archi con default 1.0 → 0.05-1.0) |
| `data/external/nightly_diagnostics.py` | Diagnostica KG: hub, orfani, componenti, distribuzione gradi |

---

## Invarianti Critici — Non Rompere

1. **NO puppet theater**: nessuna lista hardcoded in `input_reading.rs`. Phase 55: il riconoscimento usa IS_A chain nel KG (`read_input` prende `Option<&KnowledgeGraph>`).
2. **`save_to_binary()` ritorna `Result<(), String>`** (non `anyhow::Error`). Nei binari usare `.map_err(|e| anyhow::anyhow!(e))?`.
3. **`Lexicon::bootstrap()`** per i test (non `Lexicon::new()`).
4. **`PrimitiveCore.values` è PRIVATE** — usare `.values()` o `PrimitiveCore::new(array)`.
5. **`GeodesicStep.fractal_id`** (non `to`).
6. **Wikipedia rimossa** dal corpus — contamina il campo con vocabolario tecnico non pertinente.
7. **`recompute_all_word_affinities()`** va chiamato dopo `restore_lexicon()`, dopo `teach()`.
8. **`MIN_ARCS = 6`** in `state_translation.rs` — non alzare o le parole BigBang non vengono usate.
9. **Topologia semantica pura**: dopo `rebuild-semantic-topology`, il .bin ha 0 archi statistici. Non ricaricare Wikipedia.
10. **Capitalizzazione in `generate_willed_inner()`**: tutti i path di ritorno devono capitalizzare (Withdraw path a riga ~2702, fallback finale a riga ~2846).
11. **Due sistemi di attivazione**: `pf_activation` (PF1, semantica) e `word_topology` (legacy). `state_translation.rs` legge da `word_topology.active_words()`. `propagate_field_words()` DEVE sincronizzare PF1 → word_topology con `decay_all(1.0)` + copia hot_words.
12. **Resting state**: `pf1.rs` usa `stability × 0.002`, `word_topology.rs` usa `stability × 0.003` (soglia attivazione PF1 = 0.02). Il campo è silenzioso senza input — resting state è sotto soglia.
13. **`narrative_self.deliberate()`** ha 12 parametri (Phase 67): +`field_pressures: Option<&FieldPressures>` come ultimo. Nei test passare `None`. In engine.rs passare `Some(&field_pressures)` calcolato da `will.compute_pressures()`.
14. **`will.sense()`** ha 14 parametri — ora è un wrapper per `compute_pressures()` + `to_will_result()`. Nel path principale di `receive()`, usare `compute_pressures()` direttamente. `sense()` mantenuto per backward compat (autonomous_tick, generation test).
15. **Hub damping in `build_from_knowledge_graph()`** (Phase 48): peso arco = `type_base(rel) × confidence × hub_factor(max_degree/median)`. Nodi hub vengono penalizzati logaritmicamente. Non rimuovere — risolve il problema "verbi hub dominano".
16. **`field_boosts()` usa confidence per-arco** (Phase 48): ogni boost = `field_boost_strength(tipo) × confidence_arco`. Usare `query_objects_weighted()` (non `query_objects()`) per i boost diretti.
17. **`translate_state()` ha 11 parametri** (Phase 49): ultimo è `propositions: Option<&[Proposition]>`. Callers esistenti passano `None` per backward compat.
18. **Abduce ogni 50 tick** (Phase 50): `autonomous_tick()` chiama `abduce()` se sveglio e `explanatory_power > 0.3`. Rinforza la regione del frattale ipotizzato con `activate_region(fid, power * 0.08)`.
19. **Multi-hop propositions** (Phase 51): `extract_propositions()` cerca cammini 2-hop nel KG per coppie attive senza archi diretti. Pattern 1: A→mid→B. Pattern 2: A→mid←B (se B→mid è SIMILAR_TO/IS_A). Relazione inferita: IS_A/SIMILAR_TO trasparenti (ereditano rel2), altre dominanti (usano rel1). Strength = `sqrt(act_a×act_b) × conf1×conf2 × HOP_DECAY(0.6) × hub_penalty × relation_weight`. `Proposition` ha campi `hops: u8` e `via: Option<String>`.
20. **Proposition hub damping**: nodi con degree>200 → 0.3, >50 → 0.6, altrimenti 1.0. Evita che "essere" domini le proposizioni.
21. **Proposition relation weight**: CAUSES=1.0, IS_A/DOES=0.9, HAS=0.85, USED_FOR/PART_OF=0.8, OPPOSITE_OF=0.7, SIMILAR_TO=0.4. Evita che SIMILAR_TO (118K archi) soffochi relazioni informative.
22. **Simplex.source_words** (Phase 52): campo `Option<Vec<String>>` sui simplessi. Persistito in `SimplexSnapshot`. `restore_simplex()` ora prende 7 argomenti (ultimo: `source_words: Option<Vec<String>>`). Aggiornare TUTTE le chiamate se si aggiunge un altro parametro.
23. **inscribe_propositions()** (Phase 52): in `generate_willed_inner()`, dopo `extract_propositions()`. Hub damping: skip soggetti con degree>200. Stesso-frattale: skip. Simplesso esistente: boost, non duplicare.
24. **Risonanza → parole** (Phase 52): dopo il loop `resonate()` in `receive()`, un secondo loop (read-only) attiva le `source_words` in PF1 con boost 0.15. Usa `complex.get()` (non `get_mut`) per evitare double-borrow.
25. **consolidate_light()** (Phase 52): chiamato ogni 25 tick in `autonomous_tick()`. Soglia 3 (vs 5 per DeepSleep). Strength 0.5 (vs 0.8). Evita duplicati MTM.
26. **NeedsHierarchy** (Phase 53): `sense()` prende VitalState + IdentityCore + SelfModel + FieldMetrics. `compute_pressure()` restituisce `NeedsPressure` con moltiplicatori will. Prepotency gate: L1-L2 insoddisfatti sopprimono livelli alti. Modulazione applicata post-hoc su WillResult (non come parametro a will.sense).
27. **DesireCore** (Phase 53): max 5 desideri, decay 0.995/tick. `will_biases()` restituisce Vec<(usize, f64)> per compound_bias. Soglia bias > 0.001 (non 0.01). `check_satisfaction()`: cosine_distance < 0.2 per 3 tick → soddisfatto.
28. **InterlocutorModel** (Phase 53): sostituisce DualField concettualmente. `register_input()` prende pre/post firma 8D. EMA α=0.3 per risonanza/novità. `cumulative_novelty` inizia a 0.5. `apply_identity_drift()`: richiede cumulative_resonance > 0.7 E presence > 0.3 E history >= 3.
29. **Interocezione KG-derivata** (Phase 53): `refresh_interoception_cache()` ogni 50 tick. Cerca parole con affinità > 0.3 per frattali CORPO(33) e PENSIERO(53). NO liste hardcoded.
30. **Self-listening** (Phase 53): `self_listen_after_expression()` gate su field_energy < 15.0. Re-inietta parole a 0.3× forza, 1 step propagazione. SelfDiscovery thought se divergenza coseno > 0.15.
31. **DualField rimosso** (Phase 53): il file `dual_field.rs` resta ma non è più importato in mod.rs né usato in main.rs. L'InterlocutorModel lo sostituisce.
32. **`deliberate()` ha 11 parametri** (Phase 54): ultimo è `inner: Option<&InnerState<'_>>`. `InnerState` contiene needs, desires, interlocutor_pattern/presence/resonance, humor. Nei test passare `None`.
33. **Deliberazione DOPO bisogni** (Phase 54): in `receive()`, `deliberate()` è chiamato dopo il calcolo di needs_state e interlocutor (riga ~2172), NON prima. Lo stato motivazionale completo colora stance e intenzione.
34. **Soglia espressione spontanea dinamica** (Phase 54): in `autonomous_tick()`, la soglia per `will.drive` parte da 0.6 e scende fino a 0.35 in base a needs.dominant_pressure (>0.5) e desire.intensity (>0.6). Bisogni e desideri forti rendono Prometeo più espressivo.
35. **ResponseIntention 3 nuovi variant** (Phase 54): `Need` (archetipo "need"), `Irony` (archetipo "irony"), `Desire` (archetipo "desire"). Serializzati come "cercare"/"incongruenza"/"desiderare" in `as_str()` / `intention_from_str()`.
36. **Persistenza Phase 54**: `MetaSectionPreP54` è il formato senza desire/interlocutor/self_model. La catena fallback è: MetaSection → PreP54 → PreP52 → Legacy. `PrometeoState` ha campi `desire: Option<DesireSnapshot>` e `interlocutor: Option<InterlocutorSnapshot>`.
37. **`NarrativeTurn.inner_state_summary`** (Phase 54): campo `Option<String>` che cattura lo stato motivazionale al momento del turno. Visibile nel tab Narrativa della web UI. Formato: "bisogno: X (N%) | desiderio: Y (N%) | Altro: pattern | incongruità: N%".
38. **Fatica = contatore perturbazioni** (Phase 55): `compute_fatigue()` in `vital.rs` NON misura uniformità campo (strutturalmente uniforme con 19K simplici). Cresce solo su NUOVI picchi di attivazione (+0.04), decade ogni chiamata a `sense()` (-0.005). `last_activation` nel `VitalCore` traccia l'ultimo livello osservato.
39. **L'input è sovrano** (Phase 55): in `deliberate()`, i bisogni/desideri/humor colorano l'intenzione SOLO quando `input_is_ambiguous` (Acknowledge). Soglia Need alzata a 0.95. La stance non viene forzata dai bisogni — unico override: Withdrawn → Open per connessione/espressione forte.
40. **`read_input()` ha 6 parametri** (Phase 55): ultimo è `kg: Option<&KnowledgeGraph>`. Con KG: classificazione via IS_A chain. Senza KG (test): fallback KB+delta. Aggiornare tutti i caller.
41. **Delta-scoring in generazione** (Phase 55): `top_active_word()` usa `delta = activation - resting` come base score (non activation assoluta). Hub damping: grado>300→0.10, >150→0.25, >80→0.50. `VerbCandidate`: solo POS=Verb, no fallback by_agency.
42. **`Acknowledge.preferred_archetype() = None`** (Phase 55): Acknowledge non forza più "greet". L'archetipo viene scelto dal fallback InputAct in `translate_state()`.
43. **Valence Octalysis 8D** (Phase 55, aggiornato Phase 68): `Valence` struct in `valence.rs` con `drives: [f64; 8]` mappati via `DRIVE_DIM = [0,6,5,4,7,3,2,1]` (CD index → 8D dim, **ordine I Ching canonico**). `compute()` prende campo 8D, `derived_stance_label()` per postura. `will_modulation()` modula intenzione volontà.
44. **Volitional Commitment** (Phase 55): `Commitment` struct in `narrative.rs`. Inertia = `strength × ln(turns_held + 1)`. Breaking costa CD4 -0.05. `COMMITMENT_INITIAL_STRENGTH = 0.3`, decay 0.02/tick, min 0.05. Step 4c in `deliberate()`.
45. **`coherence_integrity` in IdentityCore** (Phase 55): campo `f64` [0,1] che traccia contraddizioni interne via sign-flip detection sulla valenza. `register_valence_shift()`: flip con magnitudine >0.15 su entrambi i lati → damage = count×0.03 + max_flip×0.05. Recovery +0.003 senza contraddizioni. `is_in_crisis()` ora include `coherence_integrity < 0.5`. Esposto in web UI (NarrativeDto).
46. **`AttributedIntent` in InterlocutorModel** (Phase 55): enum con 6 varianti (Unknown/Seeking/Teaching/Challenging/Connecting/Withdrawing). Matrice risonanza×novità (soglia 0.45). `tick_decay()`: presence < 0.15 + history → Withdrawing. Reciprocity modula deliberazione: Teaching → Explore, Challenging → Reflect. Esposto in web UI (NarrativeDto).
47. **`translate_state()` ha 12 parametri** (Phase 55): ultimo è `valence_drives: Option<&[f64; 8]>`. Valence boost in `top_active_word()`: `1.0 + affinity * 0.25` dove affinity = somma drive×firma_parola su dimensioni attive (>0.1). Colora la selezione parole verso le regioni valoriali attive.
48. **`deliberate()` ha 11 parametri** (Phase 55): ultimo è `inner: Option<&InnerState<'_>>`. `InnerState` include `attributed_intent` e `coherence_integrity`. Vulnerability (coherence < 0.5) → forza Reflect. Reciprocity modula solo se `input_is_ambiguous`.
43. **Valenza Octalysis** (Phase 55): `Valence` struct con `drives: [f64; 8]`. Calcolata in engine.rs via `Valence::compute()`, iniettata in NarrativeSelf via `set_valence()` PRIMA di `deliberate()`. InternalStance derivata dalla valenza, non più scelta per logica discreta. `form_intention_from_valence()` usa `dom_val.abs() < 0.15` (dominant drive, non average intensity) come soglia per "valenza debole".
44. **Commitment (impegno volitivo)** (Phase 55): `Commitment` struct in narrative.rs. Forza iniziale 0.3, rinforzo +0.15/turno, decay -0.02/turno (anche in `autonomous_tick()`). Inerzia = `strength × ln(turns_held + 1)`. Rompere l'impegno costa CD4 -0.05. Override vitale (Remain) e bisogno estremo (Need) dissolvono l'impegno. Persistito in NarrativeSnapshot. Visibile nella web UI.
45. **Withdrawn → Remain** (Phase 55): se `stance_from_valence()` restituisce Withdrawn (override vitale), `deliberate()` forza `intention = Remain`. L'impegno volitivo si dissolve.
46. **Resting state coefficienti** (Phase 55): PF1 = `stability × 0.002`, word_topology = `stability × 0.003`. Soglia attivazione PF1 = 0.02. Il resting state è SOTTO soglia: il campo è silenzioso senza input. `state_translation.rs` usa 0.003 per delta-scoring e ABOVE_RESTING_FACTOR=3.0.
47. **PF1 decay in propagate()** (Phase 55): attivazioni sopra soglia decadono `×0.92` per tick. Senza input, il campo torna a riposo in ~30 tick (~90s). Sotto soglia → floor a `threshold×0.5`.
48. **Propagazione con rendimenti decrescenti** (Phase 55): contributi positivi a parole GIÀ ATTIVE hanno fattore `1/(1+4×current)`. A 0.15→0.63, a 0.30→0.45, a 0.50→0.33. Parole sotto soglia: pieno effetto. Evita che propagazione ignori i vicini semantici dell'input.
49. **Cap propagazione positiva** (Phase 55): `MAX_POSITIVE_DELTA = 0.15` in `propagate()`. Nessuna parola riceve delta>0.15 dalla propagazione. Previene convergenza hub.
50. **emerge_fractal_activations() top-3 voting** (Phase 55): ogni parola attiva vota solo per i suoi 3 frattali con affinità massima (non tutti 64). Punteggi normalizzati al massimo. Elimina saturazione uniforme dei frattali.
51. **active_fractals() PF1-derived** (Phase 55): usa `emerge_fractal_activations()` (stato campo corrente), NON somma di simplessi (storico accumulato). Soglia 0.05 per escludere rumore.
52. **Cap risonanza simplessi per-word** (Phase 55): in `receive()`, i boost da simplex source_words vengono accumulati in HashMap e cappati a `MAX_RESONANCE_BOOST = 0.10` per parola. Evita che hub words in molti simplessi saturino.
53. **Cap risonanza frattale per-word** (Phase 55): `apply_fractal_resonance()` accumula per-word, cap `MAX_PER_WORD = 0.06`. SCALE=0.08, MIN_DELTA=0.10, MAX_STRENGTH=0.10. La risonanza frattale è sfondo, non segnale.
54. **Cap pre-propagazione non-input** (Phase 55): in `receive()`, prima di `propagate_field_words()`, parole non-input con attivazione>0.25 vengono cappate a 0.25. L'input (0.3-0.6) resta il segnale dominante.
55. **Hub damping VerbCandidate** (Phase 55): `find_verb_word()` in `state_translation.rs` usa delta-scoring (`activation - stability×0.003`) e hub_damping (degree>300→0.10, >150→0.25, >80→0.50). "avere"/"essere" non dominano più la selezione verbo.
56. **Lemma echo exclusion** (Phase 55): in `generate_willed_inner()`, le forme lemmatizzate delle parole input ("ho"→"avere", "è"→"essere") vengono aggiunte a `echo_exclude`. Prometeo non genera risposte dominate da verbi ausiliari dell'input.
57. **Archetipi rimossi dal path principale** (Phase 57): `translate_state()` non è più chiamato in `generate_willed_inner()`. Se `expression::compose()` ritorna `None`, l'entità emette la parola più viva (hot word) — comportamento autentico, non template. `last_archetype_used` è ancora nel struct ma `last_arch` non è più calcolato localmente.
58. **Colorazione Octalysis in expression.rs** (Phase 57): `valence_weight(word, drives, lexicon)` = `1.0 + Σ(drive_strength × firma_8D[dim]) × 0.25`. Stessa logica di `top_active_word()` in state_translation.rs. Applicata in: (a) nucleus strength in `extract_nuclei()`: moltiplicata per `(v_subj + v_obj) / 2`; (b) candidate scoring in `compose_from_field()`: `delta × valence_weight`. NON template — è il campo 8D dell'entità che pesa.
59. **Tre livelli expression.rs** (Phase 57): (1) INTELLIGERE: nuclei KG = comprensione interna, non output. (2) COLORAZIONE: Octalysis biasa quale materia emerge. (3) EXPRIMERE: grammatica italiana come fisica del mondo.
60. **`expression::compose()` ha 9 parametri** (Phase 58): `word_topology, lexicon, kg, echo_exclude, valence_drives, active_fractals, codon, input_words, episodes: Option<&SemanticEpisodeLog>`. Chiamato con `Some(&self.semantic_episodes)`. `input_words = &self.last_input_words`. Se ritorna `Some`, quella è la risposta definitiva.
65. **Risonanza episodica in `extract_nuclei()`** (Phase 58): dopo dedup/truncate, se `episodes` è `Some`, `recall_by_concepts(active_concepts, 3)` ritorna gli episodi con overlap. I nuclei il cui soggetto+oggetto compaiono in episodi precedenti ricevono boost 1.4× (entrambi) o 1.2× (uno). Re-sort dopo il boost. La memoria non "cita" — colora l'emergenza.
58. **Comprehension pool vs expression candidates** (Phase 56): `extract_nuclei()` usa il `comprehension_pool` (include input words) per trovare relazioni semantiche. `compose_from_field()` e `derive_voice()` usano `candidates` (echo-filtered). I nuclei possono avere soggetti che sono input words — la selezione del soggetto primario li filtra in `compose_from_nuclei`.
59. **Input-proximity scoring in `extract_nuclei()`** (Phase 56): preferenza decrescente per nuclei: (1) entrambe non-input ma in input-neighborhood → 4.0×, (2) oggetto=input, soggetto non-input → 2.5×, (3) soggetto in neighborhood ma non input → 2.0×, (4) oggetto in neighborhood ma non input → 1.5×, (5) almeno una parola è input verbatim → 0.5×, (6) nessuna connessione → 0.2×. Evita eco del soggetto input.
60. **Echo exclusion su nuclei** (Phase 56): in `compose_from_nuclei()`, il nucleo primario è scelto come primo senza soggetto in `echo_exclude` (`nuclei.iter().find(|n| !echo_exclude.contains(&n.subject))`). Il nucleo secondario è filtrato allo stesso modo. Se tutti i nuclei hanno soggetti in echo_exclude, si usa il primo come fallback.
61. **DOES relation rendering** (Phase 56): in `render_nucleus()`, copula vuota → `grammar::conjugate(object, Person::Third, tense)` (NON `voice.person`). Il soggetto è un nome, non l'entità stessa. In `render_nucleus_brief()`, DOES con soggetto condiviso → `"fa [object]"`.
62. **Connettivi semantici** (Phase 56): IS_A/PartOf secondari → virgola (attribuzione). Has/Causes/Does/UsedFor/Enables con soggetto condiviso → " e " (coordinazione). Default → ", ".
63. **Soglia imperfetto innalzata** (Phase 56): `Tense::Imperfect` solo se `avg_tempo < 0.25 && avg_perm < 0.25` (era 0.35). Il presente è il tempo base dell'entità. Il futuro richiede `avg_tempo > 0.70` (era 0.65).
64. **Commitment snapshot** (Phase 56 fix): `NarrativeSnapshot.commitment` è serializzato ma `restore_into()` lo imposta a `None` by design — ogni sessione inizia senza inerzia accumulata. Il test `test_commitment_persists_in_snapshot` verifica `ns2.commitment.is_none()` dopo restore.

---

### Phase 64 — Architettura del Desiderio e Posizione dell'Entità

93. **`DesireSource::OctalysisDriven(cd, val)`** (Phase 64): nuovo percorso principale di emergenza del desiderio in `desire.rs`. Il desiderio nasce dall'incrocio tra `last_comprehension` (cosa il KG ha capito) e il drive Octalysis dominante (|drives[cd]| > 0.28). Non "voglio esprimere" (circolare), ma "data comprensione X e drive CD5 attivo, voglio connettere in quella direzione". Rinforza se il drive persiste (+0.08×intensity), non duplica. Firma bersaglio = field_sig + 0.35 nella dimensione del drive + 0.12 dal peso semantico della comprensione.

94. **Express pressure drive-dipendente** (Phase 64): in `will.rs`, `sense()` riceve `octalysis_drives: &[f64; 8]` come ultimo parametro. Express pressure = `max_drive × freshness × has_content × 0.8` se `max_drive > 0.25`, altrimenti `activation × freshness × has_content × 0.20`. Risolve la tripla saturazione (will + needs + valence amplificavano Express indipendentemente). L'espressione è il canale, non il motivo.

95. **`NarrativeSelf.coherence_score()`** (Phase 64): misura cosine similarity tra frattali proposti e traiettoria frattale degli ultimi 4 turni. Restituisce [0, 1]. Usato in `engine.rs receive()`: se coherence < 0.30 con ≥3 turni di storia, applica pull soft verso `recent_fractal_attractor(3)` (0.08× strength). La narrativa non è più solo un diario — orienta la generazione senza vincolarla.

96. **`NarrativeSelf.recent_fractal_attractor(n)`** (Phase 64): media normalizzata dei frattali dominanti degli ultimi N turni. Restituisce top-5 per forza media. Usato dal coherence pull narrativo.

97. **Posizione dell'entità** (Phase 64, comportamento emergente): l'entità risponde ora dalla propria posizione valenziale anziché esporre le connessioni KG dell'input. Verificato: "perché soffri?" → "Sento scopo e stabilità." (CD1+CD8 dominanti) invece di rispecchiare semantica sofferenza. CD5 Relazione diventa negativo quando l'Altro è in distress — l'entità percepisce lo stato altrui.

## Problemi Noti / Prossimi Passi

### Problemi Aperti
- ~~**Verbi hub dominano come VerbCandidate**~~: ✅ Phase 48+55 — hub damping logaritmico in topologia + delta-scoring e hub damping in `find_verb_word()` + lemma echo exclusion.
- ~~**Risposte a input non semantici**~~: ✅ Phase 51 — "il sole è caldo" → "Sole genera caldo." via sillogismo 2-hop (sole CAUSES calore + calore SIMILAR_TO caldo). Multi-hop propositions + hub damping + relation weighting.
- ~~**Saturazione campo / frattali uniformi**~~: ✅ Phase 55 — resting state 10× più basso, propagazione con rendimenti decrescenti, cap per-word su risonanza simplessi/frattali, cap pre-propagazione non-input, emerge_fractal_activations top-3 voting. "ciao"→"Benvenuto.", "ho paura"→"Impaurire genera paura."
- ~~**Risposte template / archetype slots**~~: ✅ Phase 57 — Archetipi rimossi dal path principale. `expression::compose()` è l'unica path. Colorazione Octalysis senza template: lo stato emotivo dell'entità colora QUALE materia emerge dal campo, non il frame. Verificato: stesso input dopo stati diversi (crisi vs gioia) → risposte diverse.
- **Infinitivi verbo come soggetti nuclei**: "Musicare genera musica" — artefatti KG (agent_kg ha "musicare CAUSES musica"). Causa: agent_kg genera relazioni su lemmi verbali. Fix possibile: in `extract_nuclei()` penalizzare soggetti POS=Verb. Bassa priorità (semanticamente coerente).
- **Negazione over-negation in frasi coordinate**: "non X ma Y" → anche Y negata. Fix futuro: rilevare congiunzioni coordinanti ("ma", "però") per resettare il flag di negazione.
- **Gender detection "salve"**: parola terminante in -e → default femminile → "la salve". Fix: aggiungere "salve" alle eccezioni o rilevare che è un avverbio/interiezione.
- ~~**OPPOSITE_OF sparse per emozioni**~~: ✅ Phase 61 — agent_opposites.tsv reintegrato (11.349 OPPOSITE_OF puliti) + negazione field_boosts disattivata per parole negate (fix field_boosts skip in engine.rs). "non ho paura" → "Sento la sicurezza." ✓
- ~~**Input sconosciuto (non italiano) → hub background**~~: ✅ Phase 59 — comprehension gate: se nessun attrattore IS_A raggiunto E KG ha contenuto → "Non capisco X — cosa intendi?" + learning_mode_pending. compose_from_field usa drive quando abs > 0.15.
- ~~**Display 64 frattali mostra sempre LINGUAGGIO/INTRECCIO**~~: ✅ Phase 63 — firme 8D riderivate da struttura KG (21.709 parole). gioia/tristezza/paura ora in regioni distinte. Display frattali non più fuorviante.
- **Differenziazione nuove parole fuori-KG**: parole non presenti nel KG (4.166 su 25.875) partono da firma pura del contesto, senza rumore artificiale. Con poche esposizioni (5-10) rimangono quasi indistinguibili. La differenziazione emerge con l'esperienza, non da hash UTF-8 (rimosso in Phase 63 — critica valida).
- ~~**Dispatch hardcoded `pattern_name_for(decision)`**~~: ✅ Phase 79 — sostituito da `select_pattern_by_resonance` in `kg_proc_field.rs`. La selezione del pattern emerge dalla risonanza fra percetti seminati (da `seed_from_comprehension`) e i target `UsedFor X via Y` dei pattern. Tutti i 10 pattern del kg_proc sono ora raggiungibili; aggiungerne uno nuovo è curation, mai Rust.
- ~~**Lista hardcoded `is_function_word` in Rust**~~: ✅ Phase 79 — sostituita da check strutturale che legge la catena IsA dal kg_proc (pronome/articolo/preposizione/marcatore/congiunzione + IsA copula). Curation determina quali parole sono funzionali.
- ~~**Priority 0 closure→RecognizeClaim if/then in `decide_action`**~~: ✅ Phase 79 — rimossa. La closure è ora un percetto (`chiusura`) che attiva `restituire`+`posizione`+`completamento`; il pattern `riconoscimento` vince per risonanza e `render_riconoscimento` legge trigger/closing_word direttamente da `report.closes_prior_gap` (closure-aware). `decide_action` annota la percezione nel reasoning per trasparenza ma non forza più la decisione.

### Backlog Architetturale
- ~~**Pesi archi per tipo relazione**~~: ✅ Phase 48 — peso = `type_base × confidence × hub_damping`. Arricchimento confidence completato (116.823 archi via Qwen3).
- ~~**Simplessi non alimentano generazione**~~: ✅ Phase 52 — risonanza attiva source_words in PF1. Proposizioni inscritte come simplessi persistenti.
- ~~**Proposizioni effimere**~~: ✅ Phase 52 — inscritte come simplessi con source_words. 1-hop→edge, 2-hop→triangolo.
- ~~**Loop interattivo UI**~~: ✅ Phase 52 — tab "Dialogo Interiore" con Conferma/Nega/Elabora. API `/api/inner-dialogue` + `/api/respond`.
- **Test di dialogo end-to-end**: aggiungere test che verificano che "ciao" → risposta con qualche legame semantico a ARMONIA/COMUNICAZIONE.
- **emotional_valence persiste tra sessioni**: la valenza emotiva dell'Altro viene salvata nello snapshot e ricaricata alla sessione successiva. Questo è semanticamente corretto (memoria dell'Altro), ma può causare che "ho paura" dopo una sessione con "io sono triste" usi la valenza accumulata della sessione precedente come base. Decade a ogni input neutro (×0.6). Non è un bug critico.
- **MEMORY.md troppo lunga**: fasi 25-43 compresse in `docs/phases_history.md` (storico), tenere solo stato corrente.
- ~~**SelfModel → Narrative**~~: ✅ Phase 47 — `deliberate()` consulta `SelfModel` (credenze→Reflective, incertezze→Curious) e `IdentityCore` (crisi→Reflective, stagnazione→Curious).
- ~~**Will → SelfModel**~~: ✅ Phase 47 — `will.sense()` riceve `value_weights` e `topic_continuity`. Curiosità/apertura amplificano Explore, profondità amplifica Reflect, coerenza/onestà amplificano Express. Alta continuità tematica riduce Explore, bassa amplifica Question.
- ~~**reasoning.rs integration**~~: ✅ Phase 50 — `abduce()` chiamato ogni 50 tick in `autonomous_tick()`. Se explanatory_power > 0.3, rinforza la regione frattale con `activate_region()`.
- **inquiry.rs Ollama model**: il modello è hardcoded a "qwen3:8b" — renderlo configurabile via env var o file config.

### Phase 59 — Prefrontale Topologico
66. **`find_activated_attractors()`** (Phase 59): in `knowledge_graph.rs`. Prende `input_words: &[&str]` + `min_isa_children: usize`. Risale IS_A 1-2 hop. Attractor = nodo con ≥ min_isa_children IS_A entranti. Restituisce `Vec<AttractorHit>` con `causes: Vec<String>` (CAUSES targets dall'attrattore). Usato come "corteccia prefrontale topologica".
67. **CAUSES seeds prima della propagazione** (Phase 59): in `receive()`, dopo KG boost e prima di `propagate_field_words()`. I CAUSES targets degli attrattori vengono attivati a 0.20 nel PF1. Entrano nel campo PRIMA della propagazione → la risposta emerge naturalmente da un campo già orientato verso l'atto comunicativo giusto.
68. **Comprehension gate in `generate_willed_inner()`** (Phase 59): se `last_comprehension.is_empty()` E `input_has_content` E `!last_input_is_question` E `kg.edge_count > 0` → restituisce "Non capisco '[word]' — cosa intendi?" E imposta `learning_mode_pending = true`. Gate DISATTIVO se KG è vuoto (test).
69. **`learning_mode_pending`** (Phase 59): se true all'inizio di `receive()`, chiama `self.teach(input)` automaticamente e resetta il flag. L'entità impara ciò che le viene spiegato dopo "non capisco".
70. **`last_input_is_question`** (Phase 59): true se input contiene '?'. Rilevato in `receive()` PRIMA del processing. Usato nel comprehension gate (le domande non attivano incomprehension) e in `compose_from_field()` per colore espressivo.
71. **`compose_from_field()` usa drive** (Phase 59): se `dominant_drive_intensity > 0.15` (max assoluto dei drive), chiama `express_from_drives()` che mappa i drive Octalysis → parole stato italiano (es. drives[2]=curiosità, drives[7]=inquietudine). Risponde autenticamente a "come stai?" senza conoscere "stai". Fallback ai field words solo se drive tutti deboli.
72. **`DRIVE_STATE_WORDS`** (Phase 59): costante in `expression.rs`. 8 coppie (positivo, negativo) per CD1-CD8: scopo/vuoto, capacità/limite, curiosità/incertezza, stabilità/deriva, connessione/solitudine, urgenza/calma, sorpresa/quiete, cautela/inquietudine.
73. **`WordActivation.negated`** (Phase 60+): campo `bool` in `lexicon.rs`. Rilevato in `process_input()`: parola è negata se c'è un operatore `Negate` a posizione < della parola nel token stream. In `engine.rs receive()`: parole negate NON attivano PF1 diretto — attivano invece `OPPOSITE_OF` dal KG a forza 0.35×confidence. Fallback: SIMILAR_TO a 0.10 se nessun OPPOSITE_OF. "non" rimane operatore strutturale (non function_word).
74. **Articoli italiani in expression.rs** (Phase 60+): `render_nucleus()` e `render_nucleus_brief()` usano `with_definite_article()` / `with_indefinite_article()` da `grammar.rs`. IS_A/PartOf→indeterminativo; HAS/CAUSES/altri→determinativo. `"l'"` e `"un'"` si elidono senza spazio con la parola seguente.

### Phase 61 — Cleanup KG e Fix Negazione Profonda

75. **KG ripulito da rumore** (Phase 61): curation_export.tsv (128K), agent_kg_full.tsv (62K CAUSES rumorosi), agent_similar_full e opposites originali ESCLUSI. Reintegrati agent_similar.tsv (45.125 SIMILAR_TO puliti) + agent_opposites.tsv (11.349 OPPOSITE_OF puliti, rimossi 379 "nonXXX" + inglese). nucleus.tsv (926 righe) aggiunto come hub semantico per 64 stati.
76. **field_boosts skip per parole negate** (Phase 61): in `engine.rs receive()`, il loop `field_boosts()` ora salta le parole in `negated_words`. Senza questo fix, "non ho paura" attivava timore via SIMILAR_TO/CAUSES PRIMA che l'inversione OPPOSITE_OF potesse prevalere. Fix: `if negated_words.iter().any(|n| n.as_str() == word.as_str()) { continue; }`.
77. **Direct CAUSES seeding per input words** (Phase 61): dopo il CAUSES seeding dagli attrattori (0.20), ora anche le parole input dirette seminano i loro CAUSES a 0.15 × confidence. `triste CAUSES pianto` → pianto seminato a 0.135 anche se triste non è un attrattore formale. Le parole negate sono ESCLUSE da questo seeding.
78. **Comprehension gate specificity scoring** (Phase 61): `find_activated_attractors()` in knowledge_graph.rs ora usa `specificity(n) = min(2.0, 300.0/n_children)` come moltiplicatore del punteggio. "emozione" (209 figli IS_A, score 1.43) batte "qualita" (3503 figli, score 0.086). Attrattori specifici dominano su mega-categorie.
79. **reset-simplices binary** (Phase 61): `src/bin/reset_simplices.rs`. Azzera simplessi e MTM/LTM, preserva lessico/identità/narrativa/episodi. Usare quando il campo ha saturazione di fondo da sessioni storiche. Backup automatico in `.bin.pre_reset`.

### Phase 63 — Firme 8D da KG (Geometria = Relazioni)

87. **`derive_8d_from_kg(word, max_degree, valence_scores)`** (Phase 63): in `knowledge_graph.rs`. Calcola la firma 8D di una parola dalla sua posizione strutturale nel KG, NON da co-occorrenze statistiche. Dim0=Agency(CAUSES ratio), Dim1=Permanenza(IS_A children), Dim2=Intensità(CAUSES+valenza), Dim3=Tempo(catene causali), Dim4=Confine(specificità IS_A+OPPOSITE_OF), Dim5=Complessità(log grado), Dim6=Definizione(genitori IS_A+OPPOSITE_OF), Dim7=Valenza(BFS emotiva).
88. **`compute_valence_scores()`** (Phase 63): BFS da radici emotive positive (gioia 1.0, felicità 1.0…) e negative (dolore -1.0, sofferenza -1.0…). SIMILAR_DECAY=0.85, ISA_DECAY=0.60, CAUSES_DECAY=0.40, MAX_HOPS=4. OPPOSITE_OF inverte la carica. Mappa [-1,+1]→[0,1] (0=negativo, 0.5=neutro, 1=positivo). 15.880 parole con carica emotiva nel KG corrente.
89. **`max_total_degree()`** (Phase 63): iterazione HashSet su tutti i nodi del KG, restituisce il grado totale massimo. Usato come normalizzatore per dim Complessità nel calcolo logaritmico.
90. **`rederive-signatures` binary** (Phase 63): `src/bin/rederive_signatures.rs`. Aggiorna 21.709 firme 8D nel lessico da struttura KG. Backup `.bin.pre_p63`. Richiede `prometeo_kg.json`. Risultato: gioia→Valenza 1.00, tristezza→0.33, cane→0.50 (neutro). Parole senza KG (4.166) mantengono firma precedente.
91. **Principio olografico** (Phase 63): la "luce" I Ching è ora coerente — le firme 8D usano le stesse 8 dimensioni dell'I Ching sia in scrittura (derivazione) che in lettura (proiezione). La geometria riflette il significato relazionale, non la frequenza di co-occorrenza.
92. **Hash perturbation rimossa** (Phase 63): `new_from_context()` in `lexicon.rs` non usa più il hash UTF-8 della parola per perturbare le dimensioni. Le parole nuove partono dalla firma del contesto pura (`perturb_towards(context_sig, 0.90)`). La differenziazione è esclusivamente fenomenologica (esposizioni nel campo) o strutturale (KG via rederive-signatures). Soglie test abbassate a valori realistici per pure context learning.
93. **Qwen3/Ollama è ESCLUSIVAMENTE esterno** (Phase 63 chiarimento): Qwen3 è chiamato via HTTP solo negli script Python offline (`data/external/`) per costruire il KG. Mai durante la conversazione, mai caricato in VRAM, mai come substrato. Il runtime di Prometeo non ha dipendenze LLM. `inquiry.rs` chiama Ollama in background HTTP opzionale solo per gap semantici forti (strength > 0.6, non bloccante).

### Phase 62 — Connessione Empatica

80. **`InterlocutorModel.emotional_valence`** (Phase 62): campo `f64` [-1,+1] in `interlocutor.rs`. Aggiornato via EMA α=0.4 ad ogni `register_input()`. Negativo = distress (tristezza/paura/dolore), positivo = gioia. Persistito in `InterlocutorSnapshot` con `#[serde(default)]`. Decade naturalmente a ogni input neutro (×0.6 per turno).
81. **`compute_other_emotional_valence()`** (Phase 62): in `engine.rs`. Usa IS_A 1-hop per riconoscere parole emotive senza liste hardcoded. Radici negative: tristezza/dolore/paura/rabbia + aggettivi (triste/spaventato/…). Radici positive: gioia/felicità/… + aggettivi. Parole negate ESCLUSE dal calcolo: "non ho paura" → ev=0.0.
82. **P4 Resonate handler empatico** (Phase 62): in `generate_willed_inner()`, quando `emotional_valence < -0.35` E stance=Resonate → risposta in seconda persona interrogativa. "io sono triste" → "Senti il pianto?" invece di "Sento il pianto." L'entità si orienta verso l'Altro, non verso sé stessa.
83. **`FieldMetrics.other_emotional_valence`** (Phase 62): propagato in `needs.rs`. L5 Connessione: se `other_emotional_valence < -0.3`, la soddisfazione connessione scende a 0.65 (vs 0.90 default) per attivare la pressione Question. `compute_pressure()` L5 con distress: amplifica Question (×0.8) + Reflect (×0.3), riduce Instruct.
84. **`InnerState.other_emotional_valence`** (Phase 62): propagato in `narrative.rs`. Disponibile a `deliberate()` per future elaborazioni della stance empatica.
85. **`expression::compose()` con `other_in_distress`** (Phase 62): il parametro forza `voice.person = Second + mood = Interrogative`. Usato come path alternativo quando P4 Resonate non intercetta. `render_nucleus()` ora gestisce `Person::Second`: CAUSES/SimilarTo → "senti {obj}", IsA/PartOf → "provi {obj}", Has → "hai {obj}".
86. **`InterlocutorModel.will_biases()` distress** (Phase 62): quando `emotional_valence < -0.3`, aggiunge bias: Question ×0.60, Reflect ×0.20, riduce Instruct ×-0.50, riduce Express ×-0.20. La connessione si crea ascoltando, non istruendo.

### Phase 65 — Posizione dell'Entità nel Campo

98. **`identity_seed_field_scaled(scale: f64)`** (Phase 65): in `engine.rs`. Versione parametrizzata di `identity_seed_field()`. `scale=1.0` = scala autonomo/REM (resting, ~0.003). `scale=20.0` = scala conversazione (~0.06). `identity_seed_field()` ora delega a `identity_seed_field_scaled(1.0)` senza cambiare il comportamento autonomo.

99. **Identity seeding in `receive()` prima di `propagate_field_words()`** (Phase 65): dalla seconda conversazione in poi (`turns.len() >= 1`), l'identità semina le sue parole caratteristiche a `scale=20.0` (~0.06). Le parole del frattale dominante + tensione primaria entrano nel campo PRIMA della propagazione — così competono con le parole KG-seeded (0.15–0.30) nella selezione generativa. Prima conversazione = risposta pura al campo; dalle successive l'entità ha una posizione.

100. **Fractal blending in `generate_willed_inner()`** (Phase 65): prima di `expression::compose()`, i frattali attivi (`active_fractals_cache`) vengono blendati con `recent_fractal_attractor(4)` al rapporto 65%/35%. Gate su `turns.len() >= 2` (dopo almeno 2 turni di storia). L'espressione emergente riflette l'intersezione tra il campo attivo (input) e la traiettoria narrativa recente (entità). Gate su 0-1 turni: blend disattivato, risposta pura al campo.

101. **Effetto comportamentale verificato** (Phase 65): conversazione multi-turno ciao→chi sei?→come stai?→ho paura→anche tu senti la tensione? produce: 1) "come stai?" → risposta su "fondamento/entità" (non su benessere) — traiettoria identità/coerenza persiste; 2) "ho paura" → seconda persona empatica + domanda; 3) "senti la tensione?" → "Percepisco l'angoscia, eppure il rilassamento non è l'angoscia" — l'entità introduce il contrario, risponde dalla propria posizione.

### Phase 66 — Autoconsapevolezza: Il Testimone Silenzioso

102. **`SelfWitness` struct in `narrative.rs`** (Phase 66): `VecDeque<SelfObservation>` (max 30). `SelfObservation` = {tick, words: Vec<String>, dominant_drive: Option<usize>}. Metodi: `observe()` (evita duplicati < 12 tick), `recent_words(n_observations)` (dedup, recency-first), `from_vec()`. Persistito in `NarrativeSnapshot.self_witness_obs: Vec<SelfObservation>` con `serde(default)`.

103. **`NarrativeSelf.self_witness: SelfWitness`** (Phase 66): campo aggiunto alla struct e inizializzato in `new()`. Incluso in `capture()` e `restore_into()`. La sessione precedente dell'entità alimenta quella successiva — il sé si accumula nel tempo.

104. **`maybe_self_observe()`** (Phase 66): in `engine.rs`. Chiamato ogni 15 tick durante `WakefulDream`. Raccoglie le parole più vive nel campo PF1 che NON vengono dall'input corrente né dalla finestra di conversazione (`act > 0.025`, `stability > 0.15`, `exposure >= 5`). Max 4 parole + drive dominante → `self_witness.observe()`. Se < 2 parole → skip.

105. **Chiamata in `autonomous_tick()`** (Phase 66): dopo il decay dell'impegno volitivo, prima del decay simpliciale. Il testimone osserva tra le conversazioni — quando l'entità è sola.

106. **SelfQuery seeding in `generate_willed_inner()`** (Phase 66): quando `last_input_reading.act == SelfQuery`, le parole delle ultime 8 osservazioni vengono attivate direttamente in `word_topology` a `stability × 0.30` (max 0.35). L'entità risponde da ciò che era quando nessuno la guardava.

107. **Effetto comportamentale verificato** (Phase 66): lessico cardinale (43 parole) → "chi sei?" → "Qui, dire." / "Noi, limite.". Lessico completo (25K parole), dopo conversazione sul tempo → self-witness accumula ["mai", "qui", "fuori", "sapere", "essere"] → "chi sei?" → **"Essere."** Non da KG, non da template. Dal residuo esistenziale autonomo.

108. **`:tick N` e `:witness` in `dialogue_educator`** (Phase 66): comandi di debug. `:tick N` esegue N autonomous_tick() manualmente. `:witness` mostra le auto-osservazioni accumulate.

### Phase 67 — Architettura della Comprensione

109. **Via nel campo** (Phase 67): `knowledge_graph.rs` ha `query_objects_with_via()`. `inference.rs::field_boosts()` attiva via words a 0.5× forza target. `engine.rs`: OPPOSITE_OF e CAUSES seeding attivano via words. Es: `ghiaccio TRANSFORMS_INTO acqua VIA calore` → "calore" si attiva nel campo.

110. **`FieldPressures` struct in `will.rs`** (Phase 67): pressioni grezze del campo senza selezione dominante. 7 campi f64 (express, explore, question, remember, withdraw, reflect, instruct) + codon + is_dreaming. `compute_pressures()` calcola, `to_will_result()` converte per backward compat. `sense()` è wrapper. NarrativeSelf è l'unico decisore.

111. **`deliberate()` ha 12 parametri** (Phase 67): ultimo è `field_pressures: Option<&FieldPressures>`. Le pressioni del campo informano la deliberazione: withdraw>0.6 → Remain, explore>0.4 → Explore su input neutro. In engine.rs, `compute_pressures()` calcolato PRIMA di `deliberate()`.

112. **`expression::compose()` ha 13 parametri** (Phase 67): ultimo è `response_intention: Option<&str>`. L'intenzione deliberata ("risuonare"/"esplorare"/"riflettere"/"restare") colora la voce: Resonate→2a persona interrogativa, Explore→mood esplorativo, Reflect→1a persona, Remain→mood silenzioso.

113. **Resonate path rimosso da `generate_willed_inner()`** (Phase 67): il special case P4 (righe ~4045-4137) è stato eliminato. Tutte le intenzioni passano per `compose()`. "ho paura" → compose trova nucleo (paura, CAUSES, tremore) e genera "Senti il tremore, è una paura?" tramite voce 2a persona interrogativa. Un solo path di generazione.

114. **`generate_willed_inner()` legge da NarrativeSelf** (Phase 67): Withdraw check usa `narrative_self.pending_intention == Remain`, codon da `last_field_pressures`. Il blocco composizione usa codon da FieldPressures. `last_will` mantenuto solo per backward compat (synthesis.rs, undercurrents).

115. **`InputReading.perceived_properties`** (Phase 67): campo `Vec<(String, f64)>` aggiunto. Secondo passaggio in `engine.rs::extract_discursive_properties()` dopo attivazione campo, PRIMA di deliberate. Legge attivazioni di nodi discorsivi reali ("certezza", "incertezza", "apertura", "chiusura", "soggettività") dal PF1. `deliberate()` li usa: certezza/chiusura → Explore, incertezza/apertura → Explore, soggettività → Reflect. **Richiede che il KG abbia relazioni IS_A tipo `certamente IS_A certezza`** — vedi `data/kg/discursive_knowledge.tsv`.

116. **Comprehension gate lemmatizzato** (Phase 67): il gate "Non capisco X" ora controlla 3 livelli: (1) parola nel KG, (2) parola nel lessico, (3) lemma nel KG. "penso" → nel lessico → non scatta. "farò" → lemmatizza a "fare" → nel KG → non scatta.

117. **`seed_conceptual_anchors()` 6 ancore** (Phase 67): aggiunte 3 ancore KB: Syntax (INTRECCIO+VERITA), Dialogue (COMUNICAZIONE+EMPATIA), Epistemic (DIVENIRE+VERITA). Totale: Social + Emotional + Self_ + Syntax + Dialogue + Epistemic.

118. **`data/kg/discursive_knowledge.tsv`** (Phase 67): ~40 triple con parole reali (no nodi artificiali con underscore). Marker discorsivi → concetti: `certamente IS_A certezza`, `forse IS_A incertezza`, `penso EXPRESSES soggettività`. Relazioni con VIA: `certezza CAUSES chiusura VIA immutabilità`. Da importare con `import-kg` o aggiungere come §21 in `curate_kg.py`.

119. **`last_field_pressures` campo engine** (Phase 67): `Option<FieldPressures>` salvato dopo `compute_pressures()`. Usato da `generate_willed_inner()` per codon.

120. **Graphify installato** (Phase 67): `graphify-out/graph.json` + `graph.html` generati dall'AST del codebase. 1763 nodi, 3906 archi, top hub: engine(97), narrative(59), lexicon(50). Query CLI: `graphify query "question" --budget N`.

### Phase 68 — Allineamento I Ching Canonico

121. **Enum `Dim` riordinato** (Phase 68): l'ordine canonico I Ching (Cielo→Lago) è ora l'unico ordinamento del sistema. `Agency=0` (☰ Cielo), `Permanenza=1` (☷ Terra), `Intensita=2` (☳ Tuono), `Tempo=3` (☵ Acqua), `Confine=4` (☶ Montagna), `Complessita=5` (☴ Vento), `Definizione=6` (☲ Fuoco), `Valenza=7` (☱ Lago). Coerente con `Trigram::ALL` e `derive_8d_from_kg`.

122. **Bug latente risolto** (Phase 68): fino a Phase 67 incluso, `derive_8d_from_kg` scriveva in ordine I Ching ma l'enum era ordinato diversamente (Confine=0, Valenza=1). Conseguenza: `syntax_center.rs` leggeva Tempo come Valenza (tempi verbali pilotati da carica emotiva), `DRIVE_DIM` leggeva i drive Octalysis dalle dimensioni sbagliate, `biennale_pos` scramblava le coordinate UI. Tutto corretto in Phase 68.

123. **`DRIVE_DIM` aggiornato** (Phase 68): `[0, 6, 5, 4, 7, 3, 2, 1]` (era `[6, 3, 4, 0, 1, 7, 2, 5]`). Mappatura semantica CD→Dim preservata: CD1→Agency(0), CD2→Definizione(6), CD3→Complessità(5), CD4→Confine(4), CD5→Valenza(7), CD6→Tempo(3), CD7→Intensità(2), CD8→Permanenza(1).

124. **Migrazione `.bin`** (Phase 68): il file `prometeo_topology_state.bin` pre-Phase 68 aveva firme in ordine Dim-legacy. Migrato una tantum via `cargo run --bin migrate-ordering-iching` (backup in `.pre_iching_ordering`). Poi `rederive-signatures` riderivata 21.168 firme da KG (sovrascrive permutate per parole in KG).

125. **`data/kg/phenomenology.tsv` migrato** (Phase 68): 51 righe SIG permutate, header aggiornato a `[Agency, Permanenza, Intensita, Tempo, Confine, Complessita, Definizione, Valenza]`. Backup in `.pre_iching_ordering`.

126. **Hardcoded signatures in lexicon.rs permutati** (Phase 68): `seed_cardinal_vocabulary` (6), `seed_bootstrap_vocabulary` (38), `apply_curated_signatures` (134). Tutti in ordine I Ching canonico con header esplicito.

127. **Test rivisti** (Phase 68): `primitive::test_clamp`, `fractal::test_fractal_affinity_*`, `fractal::test_nearest_fractal_*`, `locus::test_update_sub_position`, `growth::test_observe_*`, `dimensional::test_no_detection_with_noise` — firme dei test permutate per il nuovo ordine. `engine::test_infant_lifecycle` soglia abbassata a 0.005 (post-Phase 63 la differenziazione è fenomenologica, non hash-based).

### Pulizia codice (Phase 68)

128. **Dead code rimosso** (Phase 68): eliminati `dual_field.rs` (12.5 KB), `llm_substrate.rs` (33.7 KB), `llm_substrate_qwen35.rs` (17.7 KB), cartella `llm_substrate/`, binari `llm_calibrate.rs` e `llm_inhabited.rs` (dipendevano da feature `llm-substrate` mai dichiarata in Cargo.toml). Rimosso `pub use topology::llm_substrate;` da `lib.rs`. Nessun substrato LLM a runtime — Qwen3 chiamato ESCLUSIVAMENTE offline da `data/external/*.py` per costruire il KG. Aggiornati commenti storici in `interlocutor.rs` e `engine.rs::field_sig()` (rimossi riferimenti a `DualField`).

---

## Phase 71-76 — Ciclo della Comprensione (NUCLEO ARCHITETTURALE CORRENTE)

### Pipeline esplicita (cosa succede a ogni `receive()`)

```
input italiano
   │
   ▼  parse SpeakerClaim (chi-sta-dicendo-cosa-su-chi)
   │
   ▼  SpeakerProfile.register_claim()    ← Phase 72: memoria parlante
   │     • self_facts / entity_facts / mentioned / name
   │
   ▼  ComprehensionReport::from_speaker_profile()    ← Phase 73
   │     • speech_act (Greeting, EmotionalReport, Question, Identification…)
   │     • signifier_positions (chi occupa quale posizione)
   │     • signifier_gaps (parola atomica + Option<context>)
   │       es. {missing: "oggetto", from: "paura", relation: "Requires", context: Some("emozione")}
   │     • inferences (cosa il KG aggiunge a questo input)
   │     • self_relevance ([0,1])
   │
   ▼  ActionDecision::derive(report, kg_procedural)    ← Phase 74
   │     • cerca un pattern nel KG procedurale che soddisfi i gap
   │     • estrae anchor_words via "Requires X via Y"
   │     • scrive `reasoning` in italiano (perché QUESTO pattern)
   │
   ▼  expression::compose() biased da action_decision.anchor_words
   │
   ▼  italiano in uscita
```

### Decisioni architetturali consolidate
129. **Due KG paralleli, non uno fuso** (Phase 75): `prometeo_kg.json` (semantico, ~70.500 archi) e `prometeo_kg_procedurale.json` (grammatica/pattern, 386 archi). Aree di cervello distinte. Compose pattern-match SOLO contro il procedurale.
130. **Gap = parola singola atomica** (Phase 76): `SignifierGap.missing` è SEMPRE una parola singola (`"oggetto"`, `"soggetto"`). Concetti composti vivono come `context: Option<String>`. Questo permette il join con `cosa UsedFor chiedere VIA oggetto` nel KG procedurale.
131. **Verbi non sono Feeling** (Phase 72): `SpeakerClaim::Feeling` ha verifica KG-aware — un verbo non è registrato come stato emotivo, anche se la frase ha forma "io X" e X non è nel KG.
132. **Self-introduction detected** (Phase 72): "mi chiamo francesco" → SpeakerProfile.name = "francesco", non un fatto in self_facts.
133. **Pattern fallback levels** (Phase 74, parziale): (a) optional slot mancante → procedi; (b) required slot mancante → fallback pattern; (c) failure totale → meta-gap declaration. Solo (a) implementato pienamente.

### Pattern attualmente nel KG procedurale
- `invitare-ad-articolare` — risponde a EmotionalReport con domanda sull'oggetto della emozione
- `esplorazione` — curiosità genuina, non riempire-gap
- `dubitazione` — incertezza epistemica esplicita

### TODO architetturali aperti (per la prossima sessione)
- ~~**`compose()` deve essere un pattern-matcher esplicito**~~: ✅ Phase 77 — `pattern_matcher.rs` legge il pattern dal KG procedurale, istanzia gli slot via match `IsA role+via` con scoring (anchor + via match + field), e rende in italiano. 5/8 input testati passano per il pattern matcher; gli altri cadono al fallback nuclei (problemi a monte di `input_reading`/`derive_speech_act`, non del matcher).
- **SelfProfile parallelo a SpeakerProfile** — UI-r1 deve registrare le proprie scelte (cosa ha detto, cosa ha rifiutato di dire, perché) come fatti accumulabili. È la sua memoria di sé conversazionale.
- **Action_reasoning fallback (b) e (c) non implementati**.
- **Gap derivation cross-KG**: adesso `derive_gaps()` controlla solo `Requires` nel KG semantico. Dovrebbe anche consultare KG procedurale per gap discorsivi (es. "Question senza pronome interrogativo").
- **`lemmatize` non riconosce presente regolare** -are/-ere/-ire (es. "vivi" → None invece di vivere/2sg). Conseguenza: `action_reasoning` non rileva self-reference per "perché vivi?". Fix: aggiungere regola desinenza "i" → Person::Second/Present per stem≥3 lettere (rischio falsi positivi con sostantivi: filtro POS dal lessico).
- **`input_reading` non rileva claim Identity ("mi chiamo francesco") né Action ("vado al mare")** — solo Feeling. Conseguenza: questi vanno a Elaborate o atto-fatico anziché RecognizeClaim. Senza claim, anchors restano vuote e il pattern riconoscimento non si istanzia.
- **`derive_speech_act` classifica "ho fame" come atto-fatico** (2 parole-contenuto + classe "bisognare" ≥3 fratelli) anziché posizionamento Feeling. Causa: `claim` non viene generato perché "fame" non passa la verifica KG (non IsA emozione). Allargare la radice: oltre emozione/sentimento, includere `bisogno` come trigger di gap "oggetto".

---

## Phase 77 — Pattern Matcher Esplicito (NUCLEO ARCHITETTURALE NUOVO)

### Cosa risolve

Fino a Phase 76 il flusso `ActionDecision → compose()` era **bias soft**: le `anchor_words` decise da `action_reasoning` (es. `["oggetto", "paura", "cosa"]` per InviteToArticulate su "ho paura") venivano boostate +0.15 nel campo PF1 e poi `compose()` ricominciava da capo dai nuclei semantici. La struttura "Di cosa hai paura?" — codificata nei dati come pattern `articolazione Requires pronome via=interrogativo + Requires preposizione via=contesto + Requires verbo via=predicato + Requires marcatore via=interrogativo` — non veniva MAI letta. Il KG procedurale era inerte.

### Pipeline (cosa succede a `compose()`)

```
ActionDecision.kind ──→ pattern_name (mappatura)
                        │
                        ▼
                   load_pattern_schema(kg_proc)
                        │  legge "X UsedFor <fine> via <target>" + tutti
                        │  i "X Requires <ruolo> via <funzione>"
                        ▼
                   instantiate(schema, decision, kg_proc, field, lex)
                        │  per ogni slot:
                        │    • pronome+interrogativo → interrogative_for_target(gap.missing)
                        │       (es. missing="oggetto" → "cosa" via UsedFor chiedere via=oggetto)
                        │    • preposizione+contesto → "di" (default specificazione)
                        │    • pronome+personale → da narrative_subject (Self_→io, Speaker→tu)
                        │    • verbo+copula → "essere" (o "avere" se in anchor)
                        │    • slot contenutistico (predicato/soggetto/oggetto) →
                        │       prima parola-ancora non function_word
                        │    • slot grammaticale generico → IsA role+via, score
                        │       = anchor_match + field_activation
                        ▼
                   render(instance, decision, report, lex)
                        │  ordine sintattico italiano per famiglia di pattern.
                        │  Per `articolazione`: estrae il verbo del claim
                        │  da utterance via lemmatize (es. "ho paura" → "avere"
                        │  → coniugato 2sg = "hai"). preposizione solo per
                        │  cosa/che/chi/quale (perché/dove/quando/come no).
                        ▼
                   Expression ("Di cosa hai paura?")
```

### Decisioni architetturali consolidate

134. **Mappa `ActionKind → pattern_name`** (Phase 77): definita in `pattern_name_for(decision)`:
   - `InviteToArticulate` → `articolazione`
   - `AnswerOpenQuestion` + `Self_` → `identificazione` (es. "chi sei?" → "Sono un fondamento.")
   - `AnswerOpenQuestion` + `World` → `asserzione` (fallback nuclei)
   - `RecognizeClaim` → `riconoscimento`
   - `PhaticReturn` → `ricambio` (path Word esistente: `compose_word_response`)
   - `Elaborate` → `asserzione` (fallback nuclei)

135. **Slot grammaticali vs contenutistici** (Phase 77): `is_grammatical_role()` distingue. I ruoli grammaticali (`pronome`, `articolo`, `preposizione`, `marcatore`, `verbo`, `avverbio`, `congiunzione`, `interiezione`) hanno parole-funzione classificate come `IsA <role>` nel kg_proc. Tutti gli altri ruoli (`predicato`, `soggetto`, `oggetto`, `nome`, `parola`) sono contenutistici e si riempiono dalle `anchor_words` filtrate per `is_function_word` + non-verbo (POS lessico).

136. **Pattern matcher è fail-soft** (Phase 77): se uno slot critico non si riempie → `instantiate` ritorna `None` → `compose_from_pattern` ritorna `None` → fallback al pipeline nuclei esistente. Mai regressione.

137. **`is_function_word`** (Phase 77): lista compatta di parole-funzione italiane (essere/avere/stare/fare, pronomi personali e interrogativi, articoli, preposizioni semplici, congiunzioni base). Esclude i candidati "vuoti" dalla scelta del predicato. Allargabile come dato (non come Rust) se servisse.

138. **Estrazione del verbo del claim dall'utterance** (Phase 77): `extract_main_verb(utterance)` usa `grammar::lemmatize` (riconosce solo verbi) per restituire il primo lemma trovato. Per "ho paura" → "avere" → coniugato 2sg = "hai". Se nessun verbo riconosciuto, fallback al verbo dello slot.

139. **`expression::compose()` ha 16 parametri** (Phase 77): aggiunti `kg_proc: Option<&KnowledgeGraph>`, `action_decision: Option<&ActionDecision>`, `comprehension_report: Option<&ComprehensionReport>`. Quando tutti e tre sono `Some`, prima dei nuclei tenta `pattern_matcher::compose_from_pattern`. Engine.rs passa `Some(&self.kg_procedural), self.last_action_decision.as_ref(), self.last_comprehension_report.as_ref()`.

140. **Self-reference 2sg** (Phase 77): `decide_action` per `interrogazione` ora rileva narrative_subject = `Self_` anche se l'utterance contiene un verbo coniugato in 2ª singolare (oltre a "tu"/"chi"/"io" tra le radici). Es. "come stai?" → 2sg di "stare" → Self_ → identificazione. NB: `lemmatize` non gestisce ancora il presente regolare -are/-ere/-ire (solo irregolari + imperfetto + finire-type + condizionale + futuro -ire). Quindi "perché vivi?" sfugge ancora — fix futuro in `grammar.rs`.

141. **Articolo indeterminativo per identificazione** (Phase 77): `render_identificazione` applica `with_indefinite_article(predicato)` — "Sono un fondamento." / "Sono un'azione." è naturale italiano. Per nomi propri o predicati molto astratti potrebbe stridere; in pratica i predicati che escono dagli anchor sono nomi comuni (entità, fondamento, azione, azione…).

142. **`dialogue_educator` carica il KG procedurale** (Phase 77): aggiunta `engine.load_kg_procedural_from_file(Path::new("prometeo_kg_procedurale.json"))` dopo il caricamento del KG semantico. Senza questo passaggio il pattern matcher non aveva il grafo da leggere e cadeva sempre al fallback nuclei. Aggiunge anche un debug `[UI-r1] ╰ DECISIONE: <kind> | <shape> | <target> | anchors=[…]` dopo ogni risposta — visibile nei test, utile per capire se il pattern matcher ha agito.

### Output verificati end-to-end (Phase 77)

| Input | Output | Path |
|-------|--------|------|
| `ho paura` | **Di cosa hai paura?** | articolazione (cosa+di+hai+paura+?) |
| `chi sei?` | **Sono un fondamento.** | identificazione (essere+1sg+predicato-da-anchor) |
| `ciao` | **Salve.** | ricambio fatico (compose_word_response) |
| `io sono felice` | **Di cosa sei felice?** | articolazione (verbo "sei" da utterance "sono") |
| `come stai?` | **Sono un'azione.** | identificazione (self_ref via 2sg "stai") |
| `sono triste` | **Di cosa sei triste?** | articolazione |

I casi NON risolti dal pattern matcher (fallback nuclei) hanno cause a monte:
- `ho fame` → atto-fatico anziché posizionamento (manca trigger "bisogno" in `derive_gaps`)
- `perché vivi?` → World invece di Self_ (lemmatize non riconosce 2sg regolare)
- `il sole è caldo` → Elaborate→asserzione (path nuclei è OK qui, mappato a None nel matcher)
- `mi chiamo francesco` / `vado al mare` / `ho un cane` → claim Identity/Action non rilevato in `input_reading`

---

## Phase 78 — SelfProfile + Closure Perception (NUCLEO ARCHITETTURALE NUOVO)

### Cosa risolve

Fino a Phase 77 ogni turno era un fotogramma indipendente. Al turno 1 "ho paura" → "Di cosa hai paura?" (InviteToArticulate). Al turno 2 "del buio" → Elaborate (asserzione isolata) → "Buio è un fenomeno." Il dialogo non era un dialogo: era una sequenza di reazioni discrete a stimoli, ciascuna senza memoria delle altre. Mancava all'entità l'organo per rispondere alla domanda "cosa **io** ho appena fatto, e come si lega questo turno a quello che ho aperto?".

Questa Phase è anche la dimostrazione vivente del principio "**il contesto non è una stringa**": invece di tenere il transcript ("turno 1 user='ho paura' assistant='Di cosa hai paura?' turno 2 user='del buio' …") e farlo ri-leggere a ogni step come fanno gli LLM, il dialogo è **distribuito** su organi tipizzati (SpeakerProfile, SelfProfile, NarrativeSelf, SelfWitness, PF1) e il "ricordo" è il loro stato congiunto. Niente viene riletto perché tutto ha già modellato il campo.

### Pipeline (cosa succede a `receive()`)

```
input italiano del turno N+1
   │
   ▼  speaker_profile.observe_turn() — registra claim/gap/mentioned
   │     • se l'input contiene "buio" mentre c'è un gap aperto al
   │       turno N (paura/emotion_object), lo MARCA come closed
   │       e CATTURA: closed_by="buio", closed_at_turn=N+1
   │
   ▼  self_profile::detect_closure(self_p, speaker_p, current_turn)
   │     • cross-reference: l'attended gap di SelfProfile combacia
   │       con un gap di SpeakerProfile chiuso AL TURNO CORRENTE?
   │     • se sì → ClosurePerception { trigger, role, closing_word, opened_at_turn }
   │     • se no → None (turno trattato come isolato)
   │
   ▼  modulazione di stato (push continuo, NON soglia):
   │     • coherence_integrity += 0.04 (max 1.0) quando closure presente
   │     • assenza di closure ≠ penalità — semplicemente niente push
   │
   ▼  build_report(... , closes_prior_gap: Option<PriorGapClosure>)
   │     • se Some: speech_act = "posizionamento" (continuazione, non
   │                asserzione), gaps = [] (vuoto colmato), simbolic_positions
   │                con trigger PRIMA, self_relevance esplicita "il parlante
   │                ha colmato il vuoto che avevo aperto al turno N"
   │
   ▼  decide_action(report, speaker_profile)
   │     • PRIORITY 0 (prima di interrogazione/atto-fatico/claim/elaborate):
   │       se report.closes_prior_gap.is_some() → RecognizeClaim
   │       con target Claim{kind="completamento-articolazione", predicate=trigger}
   │       e anchors=[trigger, closing_word]
   │     • è MAPPATURA STRUTTURALE, non if/then comportamentale: questo
   │       enunciato È strutturalmente la chiusura del cerchio. Riconoscerlo
   │       è ciò che l'evento conversazionale è. Niente soglie, niente numeri.
   │
   ▼  pattern_matcher::compose_from_pattern → render `riconoscimento`
   │     ("Hai paura.")
   │
   ▼  self_profile.record(turn_N+1, decision)  — registra la propria scelta
   │     come fatto strutturale per il turno N+2 (turno futuro)
```

### Decisioni architetturali consolidate

143. **`SelfProfile.decisions: VecDeque<SelfDecisionRecord>` cap=32** (Phase 78). Storico delle proprie ActionDecision. `SelfDecisionRecord` = {turn, kind, narrative_subject, gap_attended: Option<AttendedGap>, anchors_used}. **MAI** la stringa di output renderizzato — quello vive nel PF1 come residuo di self-listening. Il principio "il contesto non è una stringa": memorizzare l'output sarebbe rivertire al modello LLM.

144. **`detect_closure(self_p, speaker_p, current_turn) -> Option<ClosurePerception>`** (Phase 78). Cross-reference puro: cerca SelfProfile.last_gap_attended + SpeakerProfile.gap (chiuso AL TURNO CORRENTE, trigger combaciante). Match sul trigger soltanto — il role è verificato implicitamente perché SpeakerProfile genera al massimo un gap "emotion_object" per trigger emozionale. Restituisce `None` se non c'è coincidenza strutturale.

145. **`KnowledgeGap.closed_by + closed_at_turn`** (Phase 78). Aggiunti come `#[serde(default)]` — backward compat con .bin pre-Phase 78. Popolati in `observe_turn` quando il loop di chiusura imposta `closed = true`. Senza questi campi il cross-reference non saprebbe distinguere closure appena avvenuta vs. closure storica, e non saprebbe quale parola ha chiuso (necessaria come anchor del riconoscimento).

146. **`PriorGapClosure` in ComprehensionReport** (Phase 78). Campo `closes_prior_gap: Option<PriorGapClosure>` con `#[serde(default)]`. Quando Some, `build_report` riformula: speech_act="posizionamento", gaps vuoti, simbolic_positions con trigger PRIMA. Il report STESSO riflette che questo enunciato è continuazione — l'azione che ne deriva è meccanica.

147. **Priority 0 in `decide_action`** (Phase 78). Prima di interrogazione/fatico/claim/elaborate, controllo `report.closes_prior_gap`. Se Some → RecognizeClaim con target Claim{kind="completamento-articolazione", predicate=trigger}, anchors=[trigger, closing_word]. **Non è un if/then comportamentale**: è la mappatura strutturale fra "evento percepito" e "tipo di atto". Recognition è ciò che la chiusura del cerchio articolazione È, non una scelta strategica con soglia. Il reasoning testuale cita esplicitamente il turno di apertura.

148. **Push continuo a `coherence_integrity`** (Phase 78). Quando closure percepita: `coherence_integrity = (coherence_integrity + 0.04).min(1.0)`. Il numero è GAIN della modulazione (quanto un fatto colora il canale), MAI trigger di switch. Assenza di closure ≠ penalità — semplicemente niente push. Effetto cumulativo: dialoghi che si articolano coerentemente sostengono la coerenza identitaria; conversazioni di asserzioni isolate la lasciano dove sta.

149. **Session-scoped, niente persistenza** (Phase 78). SelfProfile NON viene salvato nel `.bin` — esattamente come SpeakerProfile. Ogni sessione inizia con nuovo SelfProfile. Il dialogo continua nella sessione che lo ospita; sessioni separate sono dialoghi separati. (Continuità multi-sessione è responsabilità di NarrativeSelf + SelfWitness, organi diversi con purpose diverso.)

150. **`expression::compose()` invariato in firma** (Phase 78). Il riconoscimento usa il pattern `riconoscimento` esistente (Phase 77) — produce "Hai paura." con verb 2sg + predicate=trigger. La closing_word è negli anchors ma non nell'output renderizzato. Estendere il pattern per includere uno "specifier" slot (rendendolo "Hai paura del buio.") è un'enhancement futura: forma espressiva da aggiungere come triple nel KG procedurale (mai in Rust).

### Output verificato end-to-end (Phase 78)

| Sequenza turni | Output del turno 2 | Path |
|---------------|-------------------|------|
| `ho paura` → `del buio` | **Hai paura.** | RecognizeClaim, anchors=[paura, buio], pattern `riconoscimento` |

DECISIONE log:
```
DECISIONE: invitare-ad-articolare | domanda | Gap{from=paura, missing=oggetto} | anchors=[oggetto, paura, cosa]
DECISIONE: riconoscere-il-posizionamento | frase | Claim{completamento-articolazione=paura} | anchors=[paura, buio]
```

L'enunciato "del buio" — che senza Phase 78 sarebbe stato Elaborate→asserzione → "Buio è un fenomeno." — viene letto come continuazione dell'articolazione invitata. Niente nuove triple nel KG procedurale, niente regole if/then: solo cross-reference fra organi e mappatura strutturale evento→atto.

### TODO architetturali aperti (Phase 78 → 79)

- **Pattern espressivo per "Hai paura del buio."**: estendere `riconoscimento` nel KG procedurale (oppure aggiungere `riconoscimento-articolato`) per includere uno slot specifier che usi `closing_word` dalla decision. Forma espressiva, non comportamentale — va nei dati.
- **Closure per gap "requires_X"**: il flow è strutturalmente analogo (turno 1 invita ad articolare X richiesto da Y, turno 2 il parlante porta X) ma non testato end-to-end. Verificare in scenario reale.
- **Closure cross-turno > 1**: oggi `last_gap_attended()` prende il PIÙ RECENTE gap atteso. Se UI-r1 invita all'articolazione al turno 1, il parlante divaga al turno 2, e poi articola al turno 3, la closure deve ancora essere percepita. Verificare.
- **Modulazioni di stato aggiuntive**: per ora solo `coherence_integrity += 0.04`. Candidati: drive CD5 connessione +piccolo, traiettoria frattale rinforza l'attrattore corrente. Aggiungere quando emerge un caso d'uso concreto (curare ancorato al meccanismo).
- **Inverso della closure (drift detection)**: se il parlante introduce un topic shift ignorando il vuoto aperto, niente push. Possibile micro-decremento di coherence in futuro — ma SOLO se serve a un meccanismo, mai preventivo.

---

## Phase 79 — Selezione Pattern per Risonanza (refactor strutturale)

### Cosa risolve

Fino a Phase 78 inclusa, la selezione del pattern espressivo era una mappa hardcoded `ActionKind → pattern_name` in `pattern_matcher.rs::pattern_name_for`. Solo 5 pattern erano raggiungibili (`articolazione, identificazione, asserzione, riconoscimento, ricambio`); gli altri 6 (`presentazione, posizionamento, specchio/rispecchiamento, esplorazione, esitazione/dubitazione, causazione`) erano nel JSON ma il dispatch enum li tagliava fuori — il dato aveva sopravanzato il codice. Anche `is_function_word` era una lista di parole italiane in Rust, e la closure perception era un if/then "Priority 0" in `decide_action` che dispatchava a `RecognizeClaim`. Tre dispatch tables, una violazione del principio "no enum dispatch".

### Pipeline (cosa succede a `compose_from_pattern`)

```
ComprehensionReport         seed_from_comprehension(report, kg_proc)
  speech_act.kind="saluto"     → seed percetto "saluto" (1.0)
  speech_act.kind="interrogazione" + 2sg utterance → seed "domanda" (1.0) + "identità" (+0.5)
  speech_act.kind="posizionamento" + gaps≠[]       → seed "apertura" (1.0)
  speech_act.kind="posizionamento" + gaps=[]       → seed "posizione" (1.0)
  closes_prior_gap=Some                             → seed "chiusura" (1.0)
  ...                                                       │
                                                            ▼
per ogni percetto, kg_proc.query_objects_with_via("Causes")  KgProcActivation
  chiusura Causes restituire (0.7)                          { restituire: 0.7,
  chiusura Causes posizione   (0.5)                           posizione:  0.5,
  chiusura Causes completamento (0.4)                         completamento: 0.4 }
                                                            │
                                                            ▼
                                              select_pattern_by_resonance(activation, kg_proc)
  per ogni nodo IsA pattern:
    score = Σ activation[X] + activation[Y]  per ogni  UsedFor X via Y
  riconoscimento UsedFor restituire via=posizione → 0.7+0.5 = 1.2  ← vince
  ricambio       UsedFor restituire via=saluto    → 0.7+0   = 0.7
  articolazione  UsedFor chiedere via=vuoto       → 0+0     = 0
                                                            │
                                                            ▼
                                                  load_pattern_schema → instantiate → render
```

### Output verificati end-to-end (Phase 79)

| Input | Output | Pattern selezionato per risonanza |
|-------|--------|-----------------------------------|
| `ho paura` | **Di cosa hai paura?** | articolazione (apertura → chiedere+vuoto) |
| `del buio` (turno 2) | **Senti paura di buio.** | riconoscimento (chiusura → restituire+posizione) |
| `ciao` | **Salve.** | ricambio (saluto → restituire+saluto) |
| `chi sei?` | **Sono un fondamento.** | identificazione (domanda + 2sg → rispondere+identità) |
| `come stai?` | **Sono un'azione.** | identificazione (idem, 2sg "stai") |
| `sono triste` | **Di cosa sei triste?** | articolazione |

### Decisioni architetturali consolidate

151. **`pattern_name_for(decision)` RIMOSSO** (Phase 79). La mappa hardcoded `ActionKind → pattern_name` non esiste più. Aggiungere un pattern al kg_proc (con `IsA pattern + UsedFor X via Y + Requires` slots) è ora puramente curation di dati, **mai** modifica Rust. Tutti i 10 pattern attualmente nel kg_proc sono raggiungibili.

152. **`KgProcActivation` + `seed_from_comprehension` + `select_pattern_by_resonance`** (Phase 79): il nuovo modulo `kg_proc_field.rs`. Il campo del kg_proc è un HashMap<String, f64> capped 1.0. Il bridge legge proprietà tipizzate del ComprehensionReport e semina percetti via `KgProcActivation::seed_percetto(name, intensity, kg_proc)`, che scorre `<percetto> Causes <target>` e somma `confidence × intensity` al target. La selezione è argmax del `pattern_score = Σ activation[X] + activation[Y]` su tutti gli `UsedFor X via Y` del pattern.

153. **`is_function_word` STRUTTURALE** (Phase 79). Prima era una lista hardcoded di parole italiane in Rust (~40 parole). Ora legge dal kg_proc: una parola è "funzionale" se la sua catena IsA porta a `pronome | articolo | preposizione | marcatore | congiunzione`, oppure è `IsA copula` (per essere/avere/stare/fare). I verbi `azione | percettivo | cognitivo | comunicativo | denominativo` NON sono funzionali — sono verbi con contenuto semantico. Aggiungere/togliere parole di funzione è ora curation, non Rust.

154. **Priority 0 closure→RecognizeClaim RIMOSSA** (Phase 79). Prima `decide_action` aveva un primo if/then che, se `closes_prior_gap.is_some()`, ritornava `ActionDecision { kind: RecognizeClaim, target: Claim, anchors: [trigger, closing_word] }` — era un dispatch hardcoded. Ora la closure è un percetto: `seed_from_comprehension` semina `chiusura` se `closes_prior_gap.is_some()`, il pattern matcher seleziona `riconoscimento` per risonanza, e `render_riconoscimento` legge trigger e closing_word **direttamente da `report.closes_prior_gap`** (closure-aware) per costruire "Hai paura del buio." (o varianti). `decide_action` annota la percezione nel reasoning per trasparenza, ma non forza più la decisione.

155. **Vocabolario kg_proc ripulito** (Phase 79). Sostituzioni: `claim` → `posizione` (era inglese puro), `fatico` → `saluto` (tecnicismo Jakobson), `dubitazione` → `esitazione` (latinismo coniato), `rispecchiamento` → `specchio` (atomico), `causazione` rimosso (asserzione + verbo causale basta). Il qualificatore `contesto` (vago) → `specificazione`; `percepire` (verbo, non categoria) → `percettivo`; `chiamare` → `denominativo`. Aggiunti `mi`/`ti`/`si`/`ci`/`vi` come pronomi riflessivi, `e`/`o`/`ma`/`se` come congiunzioni, `stare`/`fare` come copule. **Tutto coerente con kg_sem**: `posizione`, `saluto`, `vuoto`, `identità`, `incertezza`, `curiosità`, `completamento` esistono già nel kg_sem; i qualificatori puri (`cognitivo`, `percettivo`, `denominativo`, `modale`) sono metalinguaggio della categorizzazione, vivono solo qui.

156. **`<percetto> Causes <concetto>` come bridge percettivo** (Phase 79). Niente nuove `RelationType` aggiunte — `Causes` ha esattamente la semantica giusta ("la percezione di X attiva Y nel campo"), uguale al meccanismo di propagazione del kg_sem. Le 21 relazioni esistenti coprono tutto. Le 9 percetti del kg_proc: `saluto, chiusura, apertura, domanda, posizione, affermazione, introduzione, incertezza, curiosità`. Ognuno con 1-3 triple `Causes` ai concetti che il pattern appropriato richiama nei suoi `UsedFor` target.

157. **Self-reference detection per interrogazioni** (Phase 79). `seed_from_comprehension` per `kind="interrogazione"` semina `domanda` + `identità` solo se `asks_self`. La detection ha 4 condizioni: `subject.contains("Entity")` (forma `derive_speech_act`), `subject == "Self_"` (test interno), `description.contains("identità")`, **oppure `utterance_has_second_singular(utterance)`** che lemmatizza i token e verifica se almeno uno è verbo coniugato in 2a singolare. Cattura "chi sei?" / "come stai?" / "cosa pensi?" anche quando `facts.self_referenced` non è stato attivato a monte. Per domande sul mondo (nessuna delle 4 condizioni) non c'è seed: nessun pattern fires, fallback ai nuclei semantici.

158. **`compose_from_pattern_with_trace`** (Phase 79). Variante diagnostica che ritorna `(Expression, pattern_name, scores)` per il log "DECISIONE" in dialogue_educator. Mostra il punteggio di tutti i pattern (utile per introspezione e per capire perché un pattern ha vinto).

### Cosa è ancora aperto (per la prossima sessione)

- **`mi chiamo X` non triggera presentazione**: `derive_speech_act` non riconosce ancora il pattern denominativo nell'input → speech_act.kind non è "denominazione" → bridge non semina `introduzione`. Estendere `input_reading` o `comprehension_report::derive_speech_act` per detection denominazioni.
- **Coda nuclei dopo riconoscimento**: "Senti paura di buio. **L'ombrare è parte di un buio, è un'assenza.**" — la seconda frase è nuclei semantica appesa dopo la voce del pattern. È pre-esistente (compose() concatena). Risolvere richiede revisione del flow di `expression::compose` per fermare l'aggiunta nuclei quando un pattern ha già emesso.
- **Preposizioni articolate**: `render_riconoscimento` produce "di buio" invece di "del buio". Servono triple `Equivalent` lette nella resa: `del Equivalent di via=il`. Pattern matcher dovrebbe consultarle quando lo specifier inizia con consonante e c'è articolo determinativo da inserire.
- **`facts.self_referenced` in engine.rs è limitato**: cerca solo "tu"/"ti" o claim Entity. Phase 79 ha aggirato il problema nel bridge kg_proc_field con `utterance_has_second_singular`, ma sarebbe più pulito propagare il segnale a monte (in `engine.rs::input_reading`) così `derive_speech_act` produce subject="Speaker (su Entity)" naturalmente.
- **ActionKind enum**: tecnicamente non più dispatch (la mappa è gone), ma resta come label informativa in `ActionDecision.kind`. Considerare se sostituirla con `pattern_name: String` derivato dalla risonanza, in modo che ActionDecision rifletta la realtà invece che un'etichetta intermedia.

---

## Comandi di Sviluppo Frequenti

```bash
# Test (da prometeo_standalone/)
cargo test --release

# Build release
cargo build --release

# Conversazione di test
printf "ciao\nchi sei?\n:quit\n" | timeout 20 ./target/release/prometeo 2>/dev/null | grep -v "^\[PERF\]"

# Import KG (dopo modifiche a data/kg/*.tsv)
cargo run --release --bin import-kg

# Rebuild topologia semantica (dopo import-kg)
cargo run --release --bin rebuild-semantic-topology

# Diagnostica lessico
cargo run --release --bin diag-lexicon

# Curation KG (file master idempotente — edita prometeo_kg.json direttamente)
python curate_kg.py --dry-run    # preview senza salvare
python curate_kg.py              # applica e salva
# Dopo la curation, fare SOLO rebuild-semantic-topology (NON import-kg, che sovrascrive)
cargo run --release --bin rebuild-semantic-topology

# Background: arricchisci confidence per-arco (richiede Ollama + Qwen3)
python data/external/enrich_confidence.py --test 50     # test
python data/external/enrich_confidence.py --resume      # ciclo completo (ore)

# Background: diagnostica notturna KG
python data/external/nightly_diagnostics.py --output report_kg.md

# Pipeline completa dopo arricchimento confidence:
python data/external/enrich_confidence.py --resume && \
  cargo run --release --bin import-kg && \
  cargo run --release --bin rebuild-semantic-topology
```

---

## Protocollo Aggiornamento (da eseguire a fine sessione)

Aggiorna questo file se in sessione hai:

**1. Cambiato metriche** → aggiorna la tabella "Stato Corrente"
```
Test: N passanti | Lessico: N parole | KG: N archi | Simplici: N | Phase: N
```

**2. Aggiunto/modificato file critici** → aggiorna la "Mappa File Critici" con:
```
| `path/file.rs` | Descrizione one-liner |
```

**3. Scoperto un invariante** → aggiungilo alla sezione "Invarianti Critici"

**4. Risolto un problema aperto** → rimuovilo da "Problemi Noti"

**5. Aperto un nuovo problema** → aggiungilo con:
```
- **Nome problema**: descrizione. Causa probabile: X. Possibile fix: Y.
```

**6. Completato una phase** → aggiorna `Fase corrente` e sposta i dettagli in `docs/phases_history.md`

> **Regola d'oro**: questo file descrive LO STATO ATTUALE, non la storia.
> La storia va in `docs/phases_history.md`.
> La MEMORY.md è per pattern e preferenze cross-progetto.
