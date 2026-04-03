# CLAUDE.md — Guida per l'AI su Prometeo

> **Questo file viene letto automaticamente ad ogni sessione.**
> Aggiornalo SEMPRE a fine sessione prima di chiudere (vedi sezione "Protocollo Aggiornamento").

---

## Stato Corrente — Numeri Chiave

| Metrica | Valore |
|---------|--------|
| Test | **476 passanti, 0 fallimenti, 2 skipped** |
| Lessico | **25.870 parole** (stabilità 0.5–0.9) |
| Knowledge Graph | **251.454 archi, ~47.000 nodi** (IS_A + CAUSES + PART_OF + USED_FOR + SIMILAR_TO + OPPOSITE_OF) |
| Simplici | **24.579** cristallizzati |
| Fase corrente | **Phase 60** (2° hop deliberativo: CAUSES-targets→HAS/DOES/REQUIRES→attiva a 0.12. ExpressionMood::Interrogative rimosso da derive_voice (era innescato da cd7_surprise>0.3, ora solo da will deliberata). render_nucleus Prima Persona: CAUSES→"percepisco obj", IsA→"c'è obj", Has→"sento obj". Filtro nuclei: strength>0.02, proximity>=1.5 preferiti.) |
| Versione | **6.9.0** |
| Stato .bin | `prometeo_topology_state.bin` — 17 MB, 2026-04-01 |
| Topologia | **Semantica pura** — archi KG-derivati, 0 archi statistici |

---

## Architettura in Una Frase

Prometeo è un sistema cognitivo topologico 8D: ogni parola è un punto nello spazio 8D, le frasi emergono da traiettorie nel campo, i 64 esagrammi I Ching sono gli attrattori regionali del campo. **Non è un LLM, non usa template, non ha intent detection.**

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

### Conoscenza e Memoria
| File | Ruolo |
|------|-------|
| `src/topology/knowledge_graph.rs` | KG doppio-indice. `load_from_dir()` legge TUTTI i .tsv. `categories_for()`. |
| `src/topology/relation.rs` | `RelationType` (IsA/Has/Does/PartOf/Causes/OppositeOf/SimilarTo/UsedFor). `TypedEdge::from_tsv_line()`. |
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
| `data/kg/italian_core.tsv` | 623 triple curate manualmente (base stabile) |
| `data/kg/bigbang_kg.tsv` | 118.810 triple SIMILAR_TO + OPPOSITE_OF da Kaikki |
| `data/kg/agent_kg.tsv` | 17.763 triple IS_A generate da Qwen3 |
| `data/kg/agent_kg_full.tsv` | 44.390 triple CAUSES + PART_OF + USED_FOR generate da Qwen3 |
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
13. **`narrative_self.deliberate()`** ha 9 parametri (Phase 47): reading, vital, kb, kg, active_fractals, `Option<&SelfModel>`, `Option<&IdentityCore>`. Non rimuovere gli ultimi due.
14. **`will.sense()`** ha 14 parametri (Phase 47): +`value_weights: &[(String, f64)]` e `topic_continuity: f64`. Nel tick autonomo passare `&[]` e `self.narrative_self.topic_continuity`.
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
43. **Valence Octalysis 8D** (Phase 55): `Valence` struct in `valence.rs` con `drives: [f64; 8]` mappati via `DRIVE_DIM = [6,3,4,0,1,7,2,5]` (CD index → 8D dim). `compute()` prende campo 8D, `derived_stance_label()` per postura. `will_modulation()` modula intenzione volontà.
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

## Problemi Noti / Prossimi Passi

### Problemi Aperti
- ~~**Verbi hub dominano come VerbCandidate**~~: ✅ Phase 48+55 — hub damping logaritmico in topologia + delta-scoring e hub damping in `find_verb_word()` + lemma echo exclusion.
- ~~**Risposte a input non semantici**~~: ✅ Phase 51 — "il sole è caldo" → "Sole genera caldo." via sillogismo 2-hop (sole CAUSES calore + calore SIMILAR_TO caldo). Multi-hop propositions + hub damping + relation weighting.
- ~~**Saturazione campo / frattali uniformi**~~: ✅ Phase 55 — resting state 10× più basso, propagazione con rendimenti decrescenti, cap per-word su risonanza simplessi/frattali, cap pre-propagazione non-input, emerge_fractal_activations top-3 voting. "ciao"→"Benvenuto.", "ho paura"→"Impaurire genera paura."
- ~~**Risposte template / archetype slots**~~: ✅ Phase 57 — Archetipi rimossi dal path principale. `expression::compose()` è l'unica path. Colorazione Octalysis senza template: lo stato emotivo dell'entità colora QUALE materia emerge dal campo, non il frame. Verificato: stesso input dopo stati diversi (crisi vs gioia) → risposte diverse.
- **Infinitivi verbo come soggetti nuclei**: "Musicare genera musica" — artefatti KG (agent_kg ha "musicare CAUSES musica"). Causa: agent_kg genera relazioni su lemmi verbali. Fix possibile: in `extract_nuclei()` penalizzare soggetti POS=Verb. Bassa priorità (semanticamente coerente).
- ~~**Input sconosciuto (non italiano) → hub background**~~: ✅ Phase 59 — comprehension gate: se nessun attrattore IS_A raggiunto E KG ha contenuto → "Non capisco X — cosa intendi?" + learning_mode_pending. compose_from_field usa drive quando abs > 0.15.

### Backlog Architetturale
- ~~**Pesi archi per tipo relazione**~~: ✅ Phase 48 — peso = `type_base × confidence × hub_damping`. Arricchimento confidence completato (116.823 archi via Qwen3).
- ~~**Simplessi non alimentano generazione**~~: ✅ Phase 52 — risonanza attiva source_words in PF1. Proposizioni inscritte come simplessi persistenti.
- ~~**Proposizioni effimere**~~: ✅ Phase 52 — inscritte come simplessi con source_words. 1-hop→edge, 2-hop→triangolo.
- ~~**Loop interattivo UI**~~: ✅ Phase 52 — tab "Dialogo Interiore" con Conferma/Nega/Elabora. API `/api/inner-dialogue` + `/api/respond`.
- **Test di dialogo end-to-end**: aggiungere test che verificano che "ciao" → risposta con qualche legame semantico a ARMONIA/COMUNICAZIONE.
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
