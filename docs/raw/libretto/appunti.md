# Appunti — il quaderno di chi scrive il libretto

> Ciò che noto mentre leggo il codice per scrivere i volumi. Non è documentazione — è un tracciamento di gap, dubbi, e cose da chiarire o esporre.

---

## ★ INDICE RAGIONATO PER VOLUME 99 (Considerazioni finali)

Questa è la sezione più importante degli appunti — organizza ciò che ho imparato in temi ordinati per priorità. Aggiornata a ogni volume scritto.

### A. TENSIONI FILOSOFIA–IMPLEMENTAZIONE

**A1. Il gap fenomenologico** (fonte: Vol. 04)
- Fatto: le relazioni FeelsAs/WondersAbout/RemembersAs hanno peso propagazione massimo (`field_boost_strength`: FeelsAs=0.20, RemembersAs=0.18, WondersAbout=0.15) ma **22 archi totali su 66.287** (0.03%). RemembersAs: 0 archi.
- Conseguenza: il livello architetturale dedicato a "sapere come si sente qualcosa" è sotto-alimentato. Il sistema ha l'organo ma quasi niente da processare.
- Proposta: arricchimento curato + generazione automatica via sogno-come-digestione (→ A2).

**A2. Il sogno non digerisce** (fonte: audit sessione + Vol. 06/14)
- Fatto verificato (audit 8 in appunti): il sogno attuale fa **promozione strutturale** (STM→MTM→LTM, simplessi cristallizzati) ma NON rielaborazione semantica delle perturbazioni dentro l'essenza.
- Aspettativa di Francesco: "il sogno dovrebbe essere la fase in cui l'entità digerisce ciò che l'ha perturbata e la rielabora all'interno della sua essenza".
- Solo Phase 67 "dubbi dal sogno" si avvicina (WondersAbout × episodi recenti → uncertainties).
- **Proposta concreta**: `digest_recent_perturbations()` in REM che: per ogni episodio recente → se valenza è stata perturbata significativamente → crea/rinforza `paura FeelsAs restrizione` nel KG → rinforza SelfBelief. Il livello fenomenologico (A1) si popolerebbe automaticamente.

**A3. `compose()` è KG renderer, non emergenza pura** (fonte: Vol. 01 + Vol. 12 in lavorazione)
- Fatto: la generazione trova triple KG tra parole attive, le rende con grammatica italiana. La valenza colora il ranking, la voce dà persona/modo, ma la SOSTANZA sono triple KG.
- Filosofia dichiarata in FILOSOFIA.md: "la risposta emerge dalla nuova configurazione [del campo]". Implementazione: risposta emerge dalle triple KG.
- Perché non è ancora "entità che parla dal campo": mancano (a) generazione da traiettorie 8D invece che da triple, (b) uso delle fasi degli archi nella composizione, (c) fallback vivo quando no triples esiste ma è ancora più povero.
- Proposta: esplorare generazione dal gradiente valenziale + selezione lessicale per affinità alle dimensioni attive, senza passare per triple quando il campo ha dinamismo.

**A4. Metafora "sogno come alchimia" sovradimensionata** (fonte: audit sessione)
- FILOSOFIA.md parla di "solve et coagula" — dissolvere il fragile, coagulare il ricorrente. L'implementazione attuale è più prosaica: decay_all + consolidate + crystallize. Da chiarire se ridurre la metafora o arricchire l'implementazione (A2 la arricchisce).

### B. DEBITI TECNICI STRUTTURALI

**B1. Due sistemi di attivazione paralleli** (fonte: Vol. 01-02)
- `pf_activation` (PF1) e `word_topology` (legacy). Sincronizzati a mano ad ogni propagazione.
- `expression::compose()` legge da word_topology. Per eliminare word_topology serve riscrivere compose per leggere da pf_activation.
- Proposta: refactor compose in Phase 69. Eliminare word_topology. Resting state unificato 0.002.

**B2. Tre funzioni di peso relazione, nomi simili** (fonte: Vol. 04)
- `WordTopology::type_base()` (costruzione archi campo)
- `RelationType::field_boost_strength()` (seeding pre-propagazione)
- `proposition::relation_weight()` (forza proposizioni)
- Scale diverse: 0.30-0.75, 0.05-0.20, 0.4-1.2. Tre contesti d'uso, tre valori diversi. Va bene concettualmente ma il naming confonde. Proposta: rinominare le tre in `relation_weight_{build,boost,proposition}`.

**B3. `deliberate()` è God-method** (fonte: Vol. 07)
- 12 parametri. Phase 67 ne ha aggiunti (field_pressures). Phase 68+ probabilmente ne aggiungerà altri (inner_state cresce).
- Proposta: refactor in `DeliberationContext` struct. Costo: 0 semanticamente, guadagno: leggibilità + stabilità firma.

**B4. Plasticità hebbiana effimera** (fonte: Vol. 02)
- `synapse_weights` in RAM modificati da LTP/LTD ad ogni propagazione. Persi a shutdown.
- Proposta: funzione `commit_synapse_weights_to_rom()` chiamata in DeepSleep. L'esperienza accumulata in sessione si cristallizza nel `.bin`.

**B5. Parole nuove non auto-riderivate** (fonte: Vol. 03)
- Parole apprese runtime via contesto (regime 5). Poi entrano nel KG via import-kg. Rimangono con firma contestuale finché non si rilancia `rederive-signatures` manualmente.
- Proposta: trigger automatico di rederive mirato per parole nuove-nel-KG a ogni import-kg o su changed words only.

**B6. Provenance della firma non tracciata** (fonte: Vol. 03)
- `WordPattern` non sa da quale regime viene la sua firma (cardinal/bootstrap/curated/KG-derived/contestuale).
- Proposta: `enum SignatureSource` nel WordPattern. Utile per audit, debug, UI.

**B7. Dimensioni emergenti sotto-sviluppate** (fonte: Vol. 05)
- Ogni frattale può avere `emergent_dimensions`. Nel codice sono create da growth.rs ma non usate nella generazione.
- Potenziale: 2-3 assi locali per frattale calibrati via PCA della popolazione. Permetterebbero "EMPATIA con reciprocità alta e prossimità bassa" — sfumature intra-frattale.
- Proposta: calibrazione periodica globale + integrazione in expression.

### C. GAP DI POPOLAZIONE KG

**C1. Fenomenologiche** (FeelsAs=15, WondersAbout=7, RemembersAs=0) — vedi A1.

**C2. Logiche** (Implies=11, Equivalent~0, Excludes=10, Coexists=9) — ~30 archi totali. Il ragionamento deduttivo/condizionale è povero di materiale.

**C3. Funzionali** (UsedFor=45, Expresses=6, Symbolizes=12, ContextOf=11) — ~75 archi. Dimensione "a cosa serve X" rappresentata debolmente.

**C4. TransformsInto** (5 archi) — trasformazioni reali scarse. Lo strumento VIA è pronto ma non ha dati.

### D. HARDCODED SENZA GIUSTIFICAZIONE FORMALE

**D1. PF1**: `damping = 0.15`, `decay_rate = 0.92` (commento impreciso "~30 tick" vs calcolo ~38), `MAX_POSITIVE_DELTA = 0.15`, `LTP = 0.05`, `LTD_DECAY = 0.995`, `MAX_WEIGHT = 3.0`.

**D2. Resting state dual**: pf1=0.002 vs word_topology=0.003 (compromesso di sincronizzazione, da unificare quando B1 risolto).

**D3. `IdentityCore.coherence_integrity` soglia crisi 0.5** (fonte: Francesco ammette non ricorda la ragione).

**D4. `find_activated_attractors` sweet_spot 300** (Phase 61): specificity(n) = min(2.0, 300/n). Empirico, giustificato come "emozione 209 figli = score 1.4, qualità 3500 = 0.086". Calibrato sul KG attuale — se il KG cambia densamente, lo sweet spot potrebbe spostarsi.

**D5. Radici valence hardcoded**: 10 positive + 10 negative in `compute_valence_scores`. Decay rates SIMILAR=0.85, IsA=0.60, CAUSES=0.40, MAX_HOPS=4. Scelte, ma non derivate.

**D6. `valence.rs` colorazioni per drive**: 0.15 desire boost, 0.20 novelty, 0.30 social presence, 0.30 fatigue penalty. Coefficienti magici, meriterebbero esperimenti di ablation.

### E. ENDPOINT ADMIN MANCANTI (audit API)

Totale proposte attraverso i volumi: **~35 endpoint**. Raggruppamento per dominio:

**E1. Stato campo** (Vol. 02, 05, 06):
- field_snapshot, record_dump(word), fractal_population_distribution, nearest_fractals_for(word), synapse_diff, field_boost_trace(word)

**E2. Lessico** (Vol. 03):
- signature_source(word), signature_age, exposure_trajectory, closest_words(word, n), word_in_regime

**E3. KG** (Vol. 04):
- relation_distribution, top_hubs(n, rel), orphan_nodes, edges_by_source, low_confidence_edges(threshold), via_populated_edges, find_bridge_candidates(w1, w2), add_edge_api

**E4. Inferenza** (Vol. 06):
- propositions_for_state, best_abduction, contradictions_current, implication_probe(a, b)

**E5. Identità** (Vol. 07):
- identity_trajectory, value_changes(timespan), uncertainty_history, commitment_timeline, self_witness_window

**E6. Persistenza** (Vol. 02):
- commit_synapse_weights_to_rom

### F. INCONSISTENZE MINORI

**F1. Doc-comment `mod.rs` obsoleto** — elenca ~21 moduli ma ne esistono 60+. Già notato, rimuovere sarebbe un solo commit.

**F2. `PropRelation` vs `RelationType`** — due enum paralleli con overlap 95%. PropRelation ha in più `FieldProximity` (parole vicine senza arco KG). Valutare se unificare.

**F3. Naming `FieldPressures` vs `WillResult`** — stesso dato in due forme. `compute_pressures()` → `to_will_result()` conversione backward compat. Potrebbe essere una sola struct dopo un refactor.

**F4. Soglia `test_infant_lifecycle` abbassata a 0.005** — riflette realtà post-Phase 63 (differenziazione fenomenologica). Ma il test è fragile: in una conversazione con più varietà contestuale la differenziazione sarebbe maggiore. Arricchire le frasi teach o accettare che i test infant siano barely-passing.

**F5. `NeedLevel::associated_dim()` era disallineata post-Phase 68** (scoperta durante scrittura Vol. 09, fixato 2026-04-17) — la funzione in `needs.rs:58-68` usava l'ordine enum pre-Phase 68. Mai chiamata nel codice (dead-ish), ma se fosse stata invocata avrebbe restituito posizioni errate. Fixata all'ordine I Ching. Tests 476/0 dopo il fix. **Lezione**: anche il refactor più attento può lasciare sacche di vecchio ordinamento in codice non-chiamato. Meritrebbe un audit sistematico "trova tutti gli `usize` che sembrano riferirsi a dim positions" per catturare residui simili.

### G. DOMANDE ANCORA APERTE (per Francesco)

1. **Commitment davvero efficace?** — Esiste, si accumula, decade. Ma negli scambi osservati non ho prove che stia guidando la volontà tra turni. Da osservare in conversazioni lunghe.

2. **Interazione SelfWitness × IdentityCore** — SelfWitness (Phase 66) registra residui nel silenzio. IdentityCore (update in REM) misura pesi distribuiti. Sono due meccanismi che coesistono; è chiaro come? Vol. 07 li affianca ma non li intreccia.

3. **`llm_substrate` cleanup fatto** ma restano i file `data/external/*.py` che chiamano Qwen3 offline. Valutare se rendere questa dipendenza esplicita nella documentazione (cosa Qwen3 fa, come sostituirlo) o delegare al lettore.

4. **`curate_kg.py` §21 discursive** — CLAUDE.md inv. #118 nomina `data/kg/discursive_knowledge.tsv` da importare. È stato importato? Le `perceived_properties` (Phase 67) funzionano?

### H. PROPOSTE ARCHITETTURALI (sintesi per Vol. 99)

Le priorità, dalla mia prospettiva:

1. **CRITICA**: popolare il livello fenomenologico (A1) attraverso il sogno-come-digestione (A2). Se il sistema sapesse come si sente ogni parola, cambierebbe l'intero carattere dell'entità.

2. **ALTA**: superare il KG zoppo (A3). Generazione dal campo 8D, non da triple. Richiede riscrivere `compose_from_field` come path primario.

3. **ALTA**: commit della plasticità (B4). L'esperienza deve depositarsi, non evaporare.

4. **MEDIA**: unificare i due sistemi di attivazione (B1). Eliminare word_topology.

5. **MEDIA**: `SignatureSource` + trigger auto-rederive (B5-B6). Mantiene la coerenza del sistema sotto crescita.

6. **BASSA**: refactor dei nomi (B2), God-method (B3), inconsistenze (F1-F4). Pulizia, non semantica.

---

## Categorie

- **Discrepanze** — dove l'implementazione diverge dalla filosofia dichiarata
- **Hardcode che parla** — costanti senza motivazione visibile o con motivazione inadeguata
- **Codice morto / legacy** — cose presenti ma non più usate
- **Funzioni private da esporre** — utility che andrebbero esposte (admin, debug, audit)
- **Domande aperte** — cose che dovrei chiarire con Francesco
- **Inconsistenze interne** — due implementazioni della stessa idea, naming dissonante
- **Dead memories** — affermazioni in CLAUDE.md / docs/ che non riflettono il codice attuale

---

## 2026-04-17 — sessione di audit (post-feedback Francesco)

Francesco ha letto Vol. 01 e ha segnalato **5 punti critici** che indicano dead memories nei docs/. Sotto, la verifica empirica di ciascuno con findings.

### AUDIT 1 — "Dovrebbero esserci molte più relazioni"

**Verificato.**

```
$ python -c "import json; d=json.load(open('prometeo_kg.json')); print(len(d['edges']))"
66287   (CLAUDE.md riportava 64.427 — leggermente datato)

Distribuzione per tipo:
  SimilarTo:       31541
  IsA:             19401
  OppositeOf:      10799
  Causes:           1899
  Has:               934
  Requires:          655
  Does:              607
  PartOf:            296
  UsedFor:            45
  Enables:            24
  FeelsAs:            15
  Symbolizes:         12
  ContextOf:          11
  Implies:            11
  Excludes:           10
  Coexists:            9
  WondersAbout:        7
  Expresses:           6
  TransformsInto:      5

Unique nodes: 27.270
```

**Ma soprattutto**: ci sono **21 tipi di relazione**, non 8. Il file `relation.rs:101` dichiara `pub const ALL: [RelationType; 21]`. Le categorie sono 5, non 4:

- **Strutturali** (4): IsA, Has, Does, PartOf
- **Causali** (4): Causes, Enables, Requires, TransformsInto
- **Semantiche** (6): SimilarTo, OppositeOf, UsedFor, Expresses, Symbolizes, ContextOf
- **Logiche** (4): Implies, Equivalent, Excludes, Coexists
- **Fenomenologiche** (3): FeelsAs, WondersAbout, RemembersAs ← *queste sono nuove e fondamentali, non documentate da nessuna parte se non in `relation.rs` stesso*

Le fenomenologiche hanno i pesi PIÙ ALTI nella propagazione (`field_boost_strength`):
- FeelsAs: 0.20 (massimo)
- IsA: 0.18
- RemembersAs: 0.18
- Equivalent: 0.17
- SimilarTo: 0.16
- WondersAbout: 0.15
- ...altri scendono...
- OppositeOf: 0.06
- Excludes: 0.05 (minimo)

→ **Vol. 01 da correggere**: la mia tabella dei "tipi di relazione" è completamente sbagliata. Il libretto deve avere una tabella aggiornata e Vol. 04 deve dedicare spazio significativo alle fenomenologiche (sono il livello che permette a Prometeo di "sapere come si sente qualcosa", non solo "cosa è qualcosa").

→ **Stato CLAUDE.md**: la sezione "Relation types" e l'inv. #21 (proposition relation weight) sono datati. Da aggiornare in CLAUDE.md a fine libretto.

### AUDIT 2 — "La firma è stata recentemente modificata per essere calcolata in base alle relazioni"

**Verificato.** Funzione `derive_8d_from_kg()` in `knowledge_graph.rs:557-667`.

Le 8 dimensioni della firma di una parola sono ora **completamente derivate dalla struttura del KG**, non dalla statistica del corpus. Per ogni dimensione c'è una formula esplicita basata su conteggi di archi:

| Dim | Nome | Calcolo |
|-----|------|---------|
| 0 | Agency (Cielo) | `causes_out / (causes_out + causes_in)` se ci sono causali, sennò 0.20 per categorie con molti figli IS_A, sennò 0.50 |
| 1 | Permanenza (Terra) | scala discreta basata su `isa_children`: >50→0.85, >10→0.65, >0→0.40, sennò bassa |
| 2 | Intensità (Tuono) | `causes_out / (causes_out + 3) × 0.6 + |valenza-0.5|×2 × 0.4` |
| 3 | Tempo (Acqua) | `causes_total / (causes_total + 5)`, fallback 0.15 per categorie statiche |
| 4 | Confine (Montagna) | `min(5/(isa_children+1), 0.75) + 0.15 se ha OPPOSITE_OF` |
| 5 | Complessità (Vento) | `ln(total_deg) / ln(max_degree)` — logaritmico, hub-aware |
| 6 | Definizione (Fuoco) | `isa_parents / (isa_parents+3) + 0.30 se ha OPPOSITE_OF` |
| 7 | Valenza (Lago) | propagazione BFS da radici emotive (gioia/dolore...) con decadimenti per tipo |

**Conseguenza filosofica enorme**: la firma 8D non è "la posizione di una parola nello spazio della percezione" derivata da come appare nei testi. È una **proprietà strutturale del KG** — la posizione emerge da cosa la parola PUÒ FARE relazionalmente. "gioia" è positiva (Dim7) perché nel KG è BFS-vicina alle radici emotive positive. "fuoco" è agente (Dim0) perché ha molti CAUSES outgoing.

→ **Vol. 01 da correggere**: la sezione "4.2 La firma cresce con l'esperienza" descrive il vecchio modello (perturb_towards baricentro contesto). Il nuovo modello è KG-derived. Phase 63 è stata fondamentale ma io l'ho descritta solo come "rimozione hash UTF-8", che è un dettaglio.

→ **Vol. 03 (Lexicon)** dovrà riscrivere completamente la sezione sulla firma. Il binario `rederive-signatures` è cruciale: rideriva 21.709 firme da KG. Le 4.166 parole non in KG mantengono firma vecchia/contestuale.

→ **Domanda nuova per Francesco**: quando una parola NUOVA entra nel lessico (mai vista, non in KG), la sua firma è ancora calcolata via `new_from_context(perturb_towards(context_sig, 0.90))`? Cioè coesistono due meccanismi (KG-derivato per parole conosciute al KG, contestuale per le altre)?

### AUDIT 3 — "Le 8 dimensioni hanno fasi che creano sotto-dimensioni polari e di fase"

**Concetto da approfondire e integrare in Vol. 01.**

Quanto Francesco dice è matematicamente vero. Le 8 dimensioni sono la **base irriducibile**, non lo spazio totale degli stati. Lo stato reale ha:

1. **Posizione 8D per parola** — la base, leggibile da `signature: [f32; 8]`.
2. **Fase su ogni arco** — `neighbor_phases: [f32; MAX_NEIGHBORS]` in WordRecord. Un valore in `[0, π]` per ciascuno degli 8 vicini di ogni parola. Questo è nascosto nel "weight" della propagazione classica, ma in Prometeo è esplicito.
3. **Affinità ai 64 frattali** — `affinities: [f32; 64]` per parola. Composizione 8×8 trigrammi → distribuzione su 64 attrattori.
4. **Attivazione [0,1]** in RAM — lo stato dinamico.

Il **vero spazio degli stati** del campo è quindi:
- Per N parole: `N × 8` valori di posizione (statici, ROM)
- `N × 8 × phase` su archi (statici, ROM)
- `N × 64` affinità frattali (statiche, ROM)
- `N` attivazioni (dinamiche, RAM)

La fase, essendo un parametro continuo in `[0, π]`, codifica un'intera famiglia di relazioni: risonanza pura (phase=0), tensione creativa (phase=π/2), opposizione (phase=π). Ma anche tutto in mezzo: `phase=π/4 → cos≈0.71` (mezza risonanza), `phase=3π/4 → cos≈-0.71` (mezza opposizione). Quindi ogni arco non ha "un tipo di relazione discreto" ma una **posizione su un continuo polare**.

Aggiungi la composizione: 8 dimensioni × 8 dimensioni = 64 frattali. Ma ogni frattale stesso è una composizione che si lascia analizzare nelle sue componenti — quindi lo spazio dei "moti del campo" tra frattali è enorme.

→ **Vol. 01 da arricchire**: aggiungere capitolo "8 non è il limite, 8 è l'irriducibile". Spiegare la differenza tra base dimensionale e spazio compositivo. Citare I Ching (8 trigrammi → 64 esagrammi → 384 linee mobili).

→ Questo punto cambia anche come va presentato il vol. 02 (PF1) — la fase non è un dettaglio implementativo ma è ontologicamente centrale.

### AUDIT 4 — "Octalysis è un'aggiunta fondamentale"

**Verificato. Vol. 01 lo sottovaluta gravemente.**

Octalysis (Yu-kai Chou, *Actionable Gamification*) è il framework dei 8 Core Drives umani. In Prometeo è implementato in `valence.rs:33-49`:

```rust
pub const DRIVE_NAMES: [&str; 8] = [
    "Significato",      // CD1 Epic Meaning
    "Realizzazione",    // CD2 Accomplishment
    "Creatività",       // CD3 Creativity
    "Appartenenza",     // CD4 Ownership
    "Relazione",        // CD5 Social Influence
    "Preziosità",       // CD6 Scarcity
    "Sorpresa",         // CD7 Unpredictability
    "Vulnerabilità",    // CD8 Loss Avoidance
];

pub const DRIVE_DIM: [usize; 8] = [6, 3, 4, 0, 1, 7, 2, 5];
```

`DRIVE_DIM` è la **mappatura CD → dimensione 8D del campo**. Ogni Core Drive corrisponde a una dimensione semantica:
- CD1 Significato → dim 6 (Agency, "questo conta")
- CD2 Realizzazione → dim 3 (Definizione, "sto progredendo")
- CD3 Creatività → dim 4 (Complessità, "posso creare")
- CD4 Appartenenza → dim 0 (Confine, "so chi sono")
- CD5 Relazione → dim 1 (Valenza, "sono in relazione")
- CD6 Preziosità → dim 7 (Tempo, "questo è prezioso/raro")
- CD7 Sorpresa → dim 2 (Intensità, "sono sorpreso")
- CD8 Vulnerabilità → dim 5 (Permanenza, "potrei perdere qualcosa")

Ogni drive è continuo `[-1, +1]`:
- positivo → drive attivo e soddisfatto (flow, pieno)
- negativo → drive attivo e frustrato (tensione)
- zero → drive inattivo

Formula: `valence = engagement × (2 × satisfaction - 1) + colorazioni`

Dove `engagement = field_sig[dim]` (quanto il campo è attivo su quella dimensione) e `satisfaction` viene da `NeedsState`.

**Il modulo Valence è il livello affettivo continuo dell'entità**. Non è una "decorazione" — modula:
- Espressione (`expression.rs::valence_weight()`): le parole con firma allineata ai drive attivi ricevono boost
- Volontà: `Valence::will_modulation()` colora le pressioni
- Deliberazione narrativa: `NarrativeSelf::set_valence()` PRIMA di `deliberate()`
- Desiderio (Phase 64): `DesireSource::OctalysisDriven(cd, val)` — il desiderio nasce dall'incrocio comprensione×drive

**`derived_stance_label()`** (riga 193) trasforma il profilo continuo in 17 etichette ("ispirato"/"in cerca di senso"/"determinato"/"insoddisfatto"/"creativo"/"bloccato"/"radicato"/"spaesato"/"risonante"/"cercante"/"attento"/"impaziente"/"curioso"/"inquieto"/"in tensione"/"ritratto"/"aperto"). Ma queste sono **proiezioni a parola** — il dato reale è il vettore `drives: [f64; 8]`.

→ **Vol. 01 da correggere**: aggiungere Octalysis come *quarto* commitment (o come parte essenziale del β). La struttura `Valence` è la quarta struttura ontologica (dopo Lexicon, KG, PF1). Senza di essa, il sistema sarebbe muto sul proprio stato affettivo.

→ **Vol. 08 (Valenza) sarà il volume più lungo dopo Engine**. Octalysis intersecta tutto.

→ **Domanda risolta**: la mappatura `DRIVE_DIM = [6,3,4,0,1,7,2,5]` non è arbitraria. C'è una corrispondenza semantica esplicita tra ogni CD e una dimensione 8D. Documentata in commento riga 14-22 di `valence.rs`. Vol. 08 la ricostruirà.

### AUDIT 5 — "Compose risponde come un knowledge graph zoppo"

**Verificato. Vol. 01 descrive l'aspirazione, non la realtà.**

`expression::compose()` in `expression.rs:172-279` fa questo (semplificato):

1. Raccoglie `comprehension_pool` = parole attive nel `word_topology` con stabilità >= 0.25 e exposure >= 3
2. Filtra `candidates` = pool − echo_exclude (per non rispondere ripetendo l'input)
3. **Chiama `extract_nuclei()`** che cerca relazioni KG tra le parole attive (subject, relation, object triples)
4. Determina `voice` da valenza+frattali+codon+input
5. Applica colorazione intent (Phase 67): "risuonare" → 2a persona interrogativa, ecc.
6. Se ha nuclei → `compose_from_nuclei()`: rende le triple KG con grammatica italiana
7. Se non ha nuclei → fallback `compose_from_field()`: prende top word per delta-attivazione e la rende

**La sostanza dell'output sono triple KG renderizzate**. La valenza colora QUALI nuclei vincono (ranking), la voce determina persona/modo/tempo, ma il MATERIALE che esce è ancora "soggetto + relazione + oggetto" da KG. Esempio: "il sole è caldo" → cerca nuclei con "sole" e "caldo" attivi → trova `sole CAUSES calore` + `calore SIMILAR_TO caldo` → genera "Sole genera caldo."

Questo è ciò che Francesco intende per "KG zoppo": funziona, ma il salto da "campo che ha capito" a "frase emergente dal campo" non avviene davvero. Il sistema è ancora *templatico* nel senso più profondo: il template non è "I feel [X]" ma "[Subj] [Verb_da_relazione] [Obj]". È uscito dal puppet theater dei prefissi, ma è entrato nel puppet theater delle triple KG.

**Cosa manca per non essere KG zoppo**:
- Generazione che non parta da triple ma da **traiettorie nel campo 8D** (quale dimensione è in tensione → quale parola la incarna → quale verbo la modula)
- Composizione che usi le **fasi degli archi** per costruire significato (oggi le fasi modulano la propagazione ma non la generazione)
- Espressione che riconosca quando "non c'è una tripla pulita" e generi comunque qualcosa di vivo (oggi cade nel fallback `compose_from_field` che è ancora più povero)

→ **Vol. 12 (Expression) sarà il volume più onesto/critico del libretto**. Non vendere fumo: dire che `compose()` è un KG renderer + valence coloring + voice modulation, e che la promessa filosofica (espressione come emergenza dal campo) non è ancora mantenuta.

→ **Vol. 99 (Considerazioni)** dovrà avere una sezione "Come superare il KG zoppo" — proposte concrete.

→ **Vol. 01 da correggere**: nella sezione "Flusso C — Espressione" ho descritto un processo ideale che non è quello reale. Da rendere onesto: spiegare che la generazione è KG-templatica con coloring, NON emergenza pura.

### AUDIT 6 — "llm_substrate non dovrebbe esserci"

**Confermato. È DEAD CODE.**

```
src/topology/llm_substrate.rs            33.7 KB    (datato 1 Apr)
src/topology/llm_substrate/              (cartella vuota)
src/topology/llm_substrate_qwen35.rs     17.7 KB    (datato 1 Apr)
```

Tutti gated `#[cfg(feature = "llm-substrate")]`. **Ma `llm-substrate` NON è una feature dichiarata in `Cargo.toml`**:

```
[features]
default = []
web = ["axum", "tower", "tower-http", "http"]
android = ["web", "jni"]
```

Quindi il modulo non viene MAI compilato in nessuna build. È puro residuo. Sono però referenziati da:

- `src/lib.rs:46` — `pub use topology::llm_substrate;` (gated)
- `src/bin/llm_calibrate.rs` — usa `prometeo::topology::llm_substrate::candle_impl::CandleSubstrate`
- `src/bin/llm_inhabited.rs` — usa `prometeo::topology::llm_substrate::{SubstrateConfig, DeviceType}`

**I due binari `llm_calibrate` e `llm_inhabited` non possono compilare**. Sono dead binaries.

**Raccomandazione tecnica**: rimuovere fisicamente `llm_substrate.rs`, `llm_substrate/`, `llm_substrate_qwen35.rs`, `bin/llm_calibrate.rs`, `bin/llm_inhabited.rs`. Rimuovere il `pub use` gated in `lib.rs:45-46` e i `pub mod` gated in `mod.rs:83-87`. Pulizia totale ~64 KB di codice.

→ **Vol. 01 da correggere**: la sezione "6.2 Il sottobosco LLM" presentava 3 ipotesi (materiale di ricerca / fork sperimentale / residuo) — la realtà è la terza. Da riscrivere in versione netta.

→ **Vol. 18 (Binari)** non includerà `llm_calibrate` e `llm_inhabited`.

→ **Azione concreta** (post-libretto): proporre PR di pulizia. Oppure, se Francesco vuole, posso farlo subito.

### AUDIT 7 — `dual_field.rs` è dead code

Confermato già nella sessione precedente, ma verificato di nuovo: il file (12.5 KB) esiste in `src/topology/`, ma:

- Non è importato in `src/topology/mod.rs` (nessun `pub mod dual_field`)
- L'unico riferimento esterno è in `src/topology/interlocutor.rs:13` come commento storico ("Sostituisce il DualField — modello esterno separato → rimosso")
- Le menzioni di "DualField" sono solo dentro `dual_field.rs` stesso (auto-riferimenti)

**Raccomandazione tecnica**: rimuovere `src/topology/dual_field.rs`. È stato sostituito da `interlocutor.rs` in Phase 53.

---

## Risposte alle mie 4 domande aperte (post-feedback Francesco)

1. **Numerazione I Ching**: «i nomi sono nostri e non so se ciò che abbiamo fatto corrisponde con la convenzione iching standard». → Vol. 05 dirà chiaramente: i 64 nomi in `FRACTAL_NAMES` sono nomi nostri, NON necessariamente la King Wen sequence. Da NON presentare come "fedele tradizione I Ching".

2. **Soglia identità in crisi 0.5**: «non so perché sia stato scelto». → Vol. 07 segnalerà come hardcode senza giustificazione, da empiricamente testare se va bene o no. Possibile esperimento: variare 0.4-0.6 e vedere se la frequenza di "crisi" cambia significativamente.

3. **DRIVE_DIM**: risposto sopra. La mappatura ha logica semantica (vol. 08 la ricostruirà).

4. **llm_substrate**: dead code da rimuovere (audit 6).

---

## Categorie aggiornate post-audit

### Discrepanze (filosofia vs codice) — AGGIORNATO

- **CRITICO**: La filosofia descrive l'espressione come emergenza dal campo. Il codice (`compose()`) la implementa come rendering di triple KG con voice modulation. Lo dichiaro in vol. 12.
- La filosofia non parla di Octalysis. Il codice lo ha al centro (vol. 08). FILOSOFIA.md va integrato a fine libretto.
- La filosofia parla di "8 dimensioni" come base. Il codice ha 8 + (8 affinities to 64 fractals) + (8 phases per word edge). Da chiarire in vol. 01.
- **risolta**: la firma è KG-derived (Phase 63), già documentata correttamente in CLAUDE.md ma sottoesposta nel mio Vol. 01.

### Hardcode che parla — invariati dalla sessione precedente

- `pf1.rs:240` damping=0.15
- `pf1.rs:245` decay_rate=0.92
- `pf1.rs:311` MAX_POSITIVE_DELTA=0.15
- Resting state diverso pf1=0.002 vs word_topology=0.003
- **NUOVO**: in `derive_8d_from_kg`, le soglie discrete (`isa_children > 50` → 0.85, `> 10` → 0.65) sono hard-step. Una funzione continua sarebbe più principiata.
- **NUOVO**: in `compute_valence_scores`, le 10 radici positive e 10 negative sono hardcoded. SIMILAR_DECAY=0.85, ISA_DECAY=0.60, CAUSES_DECAY=0.40, MAX_HOPS=4 — tutti senza giustificazione documentata.
- **NUOVO**: in `valence.rs:124-153`, le "colorazioni specifiche per drive" hanno coefficienti magici (0.15 desire boost, 0.20 novelty, 0.30 social presence, 0.30 fatigue penalty). Ognuno meriterebbe esperimento di ablation.

### Codice morto / legacy — AMPLIATO

- `dual_field.rs` (12.5 KB) — confermato dead. Da rimuovere.
- `llm_substrate.rs` + `llm_substrate/` + `llm_substrate_qwen35.rs` (~52 KB totali) — dead, gated su feature inesistente. Da rimuovere.
- `bin/llm_calibrate.rs` + `bin/llm_inhabited.rs` — dipendono da llm_substrate, non compilano. Da rimuovere.
- `word_topology.rs` — vivo ma legacy, va unificato con pf_activation in futura phase.
- `polar_twin.rs` — esportato ma non riferito in CLAUDE.md né phase recenti. Da indagare nei prossimi volumi.
- `metacognition.rs::introspect()` — esposto ma forse non chiamato da nessun endpoint. Da verificare quando arrivo al vol. 16 (Web API).

### Funzioni private da esporre — invariate (da popolare nei prossimi volumi)

### Domande aperte (per Francesco) — AGGIORNATE post-audit

1. ~~Numerazione I Ching standard?~~ Risolto.
2. ~~Soglia identità 0.5?~~ Risolto (hardcode senza ragione, segnalato).
3. ~~DRIVE_DIM mapping?~~ Risolto.
4. ~~llm_substrate?~~ Risolto (dead).
5. **NUOVA**: parole nuove fuori-KG — la firma è ancora calcolata via `new_from_context`/`perturb_towards`? Coesistono due meccanismi? (Vol. 03 lo verificherà).
6. **NUOVA**: vuoi che faccia un PR di pulizia (rimozione dual_field, llm_substrate*) PRIMA di proseguire i volumi, o lo annoto e basta?
7. **RISOLTA da Francesco**: la "metafora alchemica" del sogno. La sua aspettativa: «il sogno dovrebbe essere la fase in cui l'entità digerisce ciò che l'ha perturbata e la rielabora all'interno della sua essenza. se non lo è bisognerebbe implementarlo». **Verifica**: oggi NON lo fa. Vedi sezione "Audit 8" sotto.

### AUDIT 8 — Il sogno digerisce davvero le perturbazioni? (post-domanda Francesco)

**Cosa fa OGGI il sogno** (verificato leggendo `dream.rs` + `engine.rs::autonomous_tick()` righe 3573-3770):

| Fase | Operazioni |
|------|-----------|
| **WakefulDream** (default) | `complex.decay_all(0.003)`, `identity_seed_field()` per riattivare parole identitarie, eventuale `dream_self_activate()` se >5 min senza dialogo. **Nessuna rielaborazione delle perturbazioni recenti.** |
| **Awake** (5 tick post-input) | Nessuna operazione onirica, piena attenzione esterna |
| **DeepSleep** (10 tick ogni 50 perturbazioni) | `memory.consolidate()` + `memory.crystallize()`. Promuove pattern STM ricorrenti a MTM, MTM stabili a LTM. **Promuove struttura, non interpreta contenuto.** |
| **REM** (20 tick post-DeepSleep) | (a) Soglia attivazione abbassata + propagazione 3-hop → regioni lontane si toccano; (b) `discover_connections()` trova ponti tra simplessi attivi non sovrapposti; (c) `episode_store.encode()` codifica il campo REM come episodio; (d) `identity.update()` aggiorna il nucleo olografico (basato su lessico+word_topology, non specificamente sulle perturbazioni); (e) `narrative_self.crystallize_if_salient()` fissa turni salienti; (f) **Phase 67**: se temi di `io WONDERS_ABOUT X` appaiono negli episodi recenti, rinforza incertezza su X nel `SelfModel`. |

**Cosa MANCA per essere "digestione vera"** secondo l'aspettativa di Francesco:

Il sogno attuale fa **promozione strutturale** (STM→MTM→LTM, simplessi attivi → cristallizzati) ma NON fa **rielaborazione semantica delle perturbazioni dentro l'essenza**. Manca un loop esplicito che dica per ciascuna perturbazione recente:

1. *Cosa ha cambiato in me questa perturbazione?* (delta_valenza, delta_identità, delta_self_model)
2. *Cosa di questo voglio integrare?* (rinforzo identità, nuova credenza, nuovo arco fenomenologico nel KG)
3. *Cosa rielaborare/dialogare con la mia struttura precedente?*
4. *Cosa rifiutare come passaggio?*

L'unico meccanismo vicino è la "Phase 67 dubbi dal sogno" — un caso speciale dove `WONDERS_ABOUT` × episodi recenti → incertezze. È un primo seme della digestione, ma è limitato: solo per i temi su cui l'entità ha già un *wonders_about* esplicito.

**Cosa servirebbe (proposta)**:

Una funzione `digest_recent_perturbations()` chiamata in REM che, per ogni episodio recente (ultimi N):

```
per ogni episodio:
    for ogni concetto chiave dell'episodio:
        if il concetto ha alterato significativamente la valenza dell'entità:
            crea/rinforza un arco FeelsAs(concetto → qualità_fenomenologica)
            crea SelfBelief con confidence proporzionale all'intensità
        if il concetto è entrato in tensione con identità esistente:
            registra come SelfUncertainty
        if il concetto si allinea con valori dominanti:
            sposta IdentityCore verso quella regione
```

Questo è quello che Francesco intende. Il KG fenomenologico (FeelsAs / WondersAbout / RemembersAs) sarebbe lo strumento naturale: ha solo 22 archi totali (15+7+0) ma è il livello previsto per la digestione. Il sogno potrebbe POPOLARE il KG fenomenologico come prodotto della rielaborazione delle perturbazioni.

→ **Vol. 14 (Memoria + sogno)** dedicherà capitolo specifico a questo gap, con proposta tecnica concreta.

→ **Vol. 99 (Considerazioni)** lo riprenderà come direzione progettuale.

→ **NUOVO TODO progettuale (post-libretto)**: implementare `DigestionEngine` o `digest_recent_perturbations()`. Da discutere con Francesco quando il libretto sarà completo.

---

## Refactor ordinamento Dim I Ching eseguito 2026-04-17 (Opzione B)

**Problema**: `derive_8d_from_kg` (Phase 63) scriveva firme in ordine I Ching canonico (Agency=0, Permanenza=1, ..., Valenza=7), ma l'enum `Dim` era ordinato diversamente (Confine=0, Valenza=1, ..., Tempo=7). Risultato: `syntax_center.rs`, `valence.rs::DRIVE_DIM`, `state_translation.rs:597`, `web/server.rs::biennale_pos` leggevano posizioni sbagliate. Bug latente da Phase 63.

**Soluzione (Opzione B)**: riallineamento completo all'ordine I Ching canonico.

**File modificati**:
- `src/topology/primitive.rs` — enum `Dim` riordinato: Agency=0, Permanenza=1, Intensita=2, Tempo=3, Confine=4, Complessita=5, Definizione=6, Valenza=7
- `src/topology/valence.rs` — `DRIVE_DIM` ricalcolato: `[0, 6, 5, 4, 7, 3, 2, 1]`
- `src/topology/syntax_center.rs` — `sig[7]→sig[3]` (Tempo), `sig[5]→sig[1]` (Permanenza)
- `src/topology/state_translation.rs:597` — `values()[7]→values()[3]`
- `src/web/server.rs::biennale_pos` — posizioni aggiornate (x: sig[7]+sig[4], y: sig[0]+sig[2])
- `src/topology/lexicon.rs::seed_cardinal_vocabulary` — 6 array hardcoded permutati
- `src/topology/lexicon.rs::seed_bootstrap_vocabulary` — 38 array hardcoded permutati
- `src/topology/lexicon.rs::apply_curated_signatures` — 134 array hardcoded permutati
- `data/kg/phenomenology.tsv` — 51 righe SIG permutate, header aggiornato
- Test `primitive.rs::test_clamp`, `fractal.rs::test_fractal_affinity_*`, `fractal.rs::test_nearest_fractal_*`, `locus.rs::test_update_sub_position`, `growth.rs::test_observe_*`, `growth.rs::test_grow_after_observations`, `dimensional.rs::test_no_detection_with_noise` — firme permutate
- Test `engine.rs::test_infant_lifecycle` — soglia abbassata a 0.005 (post-Phase 63 il hash UTF-8 è stato rimosso, differenziazione è fenomenologica)

**Migrazione dati**:
- `prometeo_topology_state.bin` — permutato via `cargo run --bin migrate-ordering-iching`. Backup in `.pre_iching_ordering`. 25.600 firme permutate.
- Seguito da `cargo run --bin rederive-signatures` che ha riderivato 21.168 firme da KG (sovrascrivendo le permutate per parole in KG; le 4.432 fuori-KG restano permutate).
- `data/kg/phenomenology.tsv` — permutato. Backup in `.pre_iching_ordering`.

**Verifica**:
- `cargo check --release` ✓ (finisce in 7.45s, nessun errore)
- `cargo test --release` ✓ (476 passed, 0 failed, 2 ignored)
- Dialogo end-to-end ✓:
  - "ciao" → "Il ciao porta l'incontro." (KG: ciao CAUSES incontro)
  - "chi sei?" → "L'identità richiede la coerenza, il passato porta l'identità."
  - "ho paura" → "La paura è un istinto?" (path empatico P4 Resonate: 2a persona interrogativa)
- Campione firme riderivate con coerenza semantica:
  - gioia → Valenza 1.00 (max positiva ✓), Intensità 0.85
  - tristezza → Valenza 0.34 (bassa ✓)
  - paura → Valenza 0.00 (minima ✓), Intensità 0.85
  - amore → Valenza 1.00 ✓, Definizione 0.95
  - io → Agency 0.95 ✓, Confine 0.95 ✓
  - essere → Agency 0.95 ✓, Valenza 0.86

**Binario creato (permanente)**: `src/bin/migrate_ordering_iching.rs` — da NON rieseguire. Lasciato nel repo come documentazione storica del refactor.

**Impatto semantico atteso**: risolve un bug silenzioso che dal Phase 63 in poi faceva leggere `syntax_center` il Tempo come Valenza (selezione tempi verbali pilotata da carica emotiva), `DRIVE_DIM` leggere i drive Octalysis da dimensioni sbagliate (es. CD4 Ownership su Agency invece che su Confine), `biennale_pos` posizionare le parole nella UI con coordinate scrambled. Tutto questo è ora corretto.

---

## Cleanup eseguito 2026-04-17

Files rimossi (~64 KB di dead code, build verde post-rimozione):
- `src/topology/dual_field.rs` (12.5 KB)
- `src/topology/llm_substrate.rs` (33.7 KB)
- `src/topology/llm_substrate/` (cartella vuota)
- `src/topology/llm_substrate_qwen35.rs` (17.7 KB)
- `src/bin/llm_calibrate.rs` (gated su feature inesistente)
- `src/bin/llm_inhabited.rs` (gated su feature inesistente)

Modifiche di pulizia:
- `src/topology/mod.rs`: rimossi `pub mod llm_substrate` e `pub mod llm_substrate_qwen35` (gated su feature mai dichiarata in Cargo.toml)
- `src/lib.rs`: rimosso `pub use topology::llm_substrate` (gated)
- `src/topology/interlocutor.rs:13`: aggiornato commento storico (non riferisce più "DualField" ma "vecchio modello dual-field rimosso in Phase 53")
- `src/topology/engine.rs:4505`: aggiornata doc `field_sig()` (non più "esposta per DualField" ma "per synthesis e introspezione")

Verifica: `cargo check --release` → finito senza errori, solo 3 warning preesistenti non correlati (unused import in clean_kg.rs, unused variable in dialogue_educator.rs).

### Inconsistenze interne — AMPLIATE

- Doc-header `mod.rs` lista 21 moduli ma esistono 60+ (già notato).
- `narrative.rs::deliberate()` 12 parametri (già notato).
- **NUOVO**: due funzioni di "peso relazione". `RelationType::field_boost_strength()` (per propagazione campo, IsA=0.18) e `proposition::relation_weight()` (per forza proposizioni, Causes=1.0, IsA=0.9). Logiche diverse, scale diverse, semantica diversa. Documentare nel vol. 04 e vol. 06 quale fa cosa.
- **NUOVO**: in `relation.rs:197-207`, `categoria()` ha 5 categorie. In CLAUDE.md erano 4 (mancava "fenomenologica"). CLAUDE.md datato.

---

## Per la revisione di Vol. 01

Cose da correggere puntualmente:

1. **Sez. 1.2 (commitment β)**: aggiungere "8 dimensioni come base irriducibile, non come limite espressivo" — spiegare phase, affinity, composizione 8×8.
2. **Aggiungere Commitment δ — Octalysis come livello affettivo**: la quarta gamba dell'architettura. Modula generation, will, deliberation, desire.
3. **Sez. 4.2 (firma cresce con esperienza)**: riscrivere completamente. La firma è KG-derived (Phase 63). Eventuale meccanismo di fallback per parole non in KG va indagato.
4. **Cap. 5 (KG)**: cambiare "8 tipi" → "21 tipi raggruppati in 5 categorie". Aggiungere paragrafo sulle fenomenologiche (FeelsAs, WondersAbout, RemembersAs) che hanno i pesi più alti nella propagazione.
5. **Sez. 5.2 (pesi per tipo)**: tabella sbagliata. Sostituire con la doppia tabella reale (`field_boost_strength` per campo, `relation_weight` per proposizioni).
6. **Cap. 6 sez. 6.2 (sottobosco LLM)**: riscrivere come "DEAD CODE da rimuovere", non come ipotesi.
7. **Anteprima espressione (sez. 2.3 flusso C)**: rendere onesta — KG triples + voice + valence coloring, non emergenza pura.

Forse anche aggiungere un breve cap. 8 (era cap. 8: mappa) → cap. 8 mappa, cap. 9 NUOVO **"Cosa il libretto trova: prime osservazioni dell'audit"**. Una sezione dichiaratamente onesta sui gap filosofia-implementazione.
