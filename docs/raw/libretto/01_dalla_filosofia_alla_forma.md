# Volume I — Dalla filosofia alla forma

> *Una posizione filosofica produce vincoli matematici. I vincoli matematici producono strutture dati. Le strutture dati producono comportamenti. La filosofia non è un'esegesi a posteriori del codice: il codice è l'incarnazione della filosofia, riga per riga — e quando il codice diverge dalla filosofia, una delle due deve cedere.*

---

## Premessa: a cosa serve questo volume

Il documento [`docs/FILOSOFIA.md`](../FILOSOFIA.md) è la posizione di partenza. È scritto in modo poetico perché parla di un'entità — e le entità non si descrivono come oggetti, si descrivono come modi di essere.

Questo volume fa un lavoro diverso. Prende ogni commitment filosofico di FILOSOFIA.md e mostra **dove vive nel codice** — e dove no. Quando la filosofia dice "le parole vivono in uno spazio a otto dimensioni", questo volume mostra che `pub struct PrimitiveCore { values: [f32; 8] }` (in [src/topology/primitive.rs](../../src/topology/primitive.rs)), spiega perché `f32` e non `f64`, perché `[f32; 8]` e non `Vec<f32>`, e cosa sarebbe cambiato se avessimo scelto `[f32; 64]` o `[f32; 768]`. E quando la filosofia dice "l'espressione emerge dal campo" ma il codice in realtà rende triple KG con coloring grammaticale, questo volume lo dice — perché un libretto onesto vale più di uno consolatorio.

L'idea è semplice: chi conosce solo la filosofia non può curare il codice. Chi conosce solo il codice non sa perché modificarlo. Il libretto sta in mezzo, e quando la filosofia e il codice non si parlano, lo dichiara.

---

## Capitolo 1 — Quattro commitment fondanti

Tutta l'architettura di Prometeo si lascia ricostruire da quattro commitment filosofici. Tutto il resto è conseguenza. Ne esistono tre dichiarati esplicitamente in FILOSOFIA.md (α, β, γ qui sotto) e un quarto (δ — Octalysis) che è centrale nell'implementazione ma non emerge esplicitamente nel testo filosofico: lo riconosceremo in 1.4.

### 1.1 — Commitment α: il significato è la geometria delle relazioni

Wittgenstein, *Ricerche Filosofiche* §43: «Il significato di una parola è il suo uso nel gioco linguistico». In FILOSOFIA.md, parte II: «se il significato è relazione, allora per avere significato basta avere relazioni». Per costruire un'entità che capisca, basta darle uno spazio in cui le parole esistano *in relazione le une con le altre*, con relazioni genuine — cioè non simulate da etichette ma incarnate da geometria.

**Conseguenza implementativa α₁**: ogni parola ha una posizione in uno spazio metrico. Non un'etichetta, non un id, non un embedding opaco — una *posizione*, dove la distanza tra due parole è la loro relazione semantica.

**Conseguenza implementativa α₂**: le relazioni esplicite (causalità, gerarchia, opposizione, **fenomenologia**) sono modellate come *archi pesati e tipizzati* in un grafo, ma non come "fatti da consultare": come *forze che modulano la propagazione* nello spazio metrico.

**Conseguenza implementativa α₃**: non c'è separazione tra "rappresentazione" e "uso". Lo stato del campo è simultaneamente cosa il sistema *sa* e cosa il sistema *sta facendo*. Questo è opposto al paradigma classico (database + motore di query).

**Conseguenza implementativa α₄ (recente, Phase 63)**: la posizione di una parola NON è osservata da fuori (statistica di co-occorrenze in un corpus); è **derivata dalla struttura relazionale**. Una parola "è dove può essere connessa", non "è dove appare". Vol. 03 entra nel meccanismo (`derive_8d_from_kg`); qui basti dire che la firma 8D di una parola conosciuta dal KG si calcola dai conteggi e dalla topologia delle sue relazioni.

→ Si materializza in: `Lexicon` (gli inventari delle posizioni), `KnowledgeGraph` (le relazioni come geometria), `PrometeoField` (lo stato del campo come ente unico).

### 1.2 — Commitment β: otto dimensioni primitive bastano a generare un mondo

L'I Ching cinese (XII sec. a.C.) deriva otto trigrammi da `2³` combinazioni binarie, e li compone a coppie per ottenere `8 × 8 = 64` esagrammi — descrivendo l'intero spazio degli stati del reale. Leibniz (1697) sviluppa indipendentemente il sistema binario e nelle monadi propone che la realtà sia generata da combinazioni di prospettive elementari. La CCRU (Warwick, anni '90) recupera questo ponte e lo radicalizza: la combinatoria di un numero piccolo di polarità basta a generare spazi di stati arbitrariamente ricchi.

In FILOSOFIA.md, parte III: «otto assi. Ogni parola è un punto in questo spazio. Ogni frase è una traiettoria. Ogni conversazione è una perturbazione che modifica la geometria del campo».

**Importante: 8 è la base irriducibile, non il limite dell'espressività.**

Questo punto è critico e va spiegato bene, perché la frase "Prometeo ha 8 dimensioni" può fuorviare chi viene da modelli a 768 o 4096 dimensioni. Confronto improprio: 8 < 768 quindi Prometeo è meno espressivo. Sbagliato.

Lo *spazio degli stati reali* del campo Prometeo è composto da quattro strati di degrees of freedom:

1. **Posizione 8D per parola** (8 numeri continui, in [0,1]). Questo è l'asse semantico nominato.
2. **Affinità ai 64 frattali per parola** (64 numeri continui, in [0,1]). Composizione 8 × 8 trigrammi.
3. **Fase su ogni arco** (per ogni parola, 8 vicini × 1 fase ∈ [0, π]). La fase è un parametro continuo che codifica la *natura* della relazione su quell'arco: `cos(0)=+1` risonanza, `cos(π/2)=0` ortogonalità, `cos(π)=-1` opposizione, e tutti i valori intermedi.
4. **Attivazione dinamica** (1 numero in [0,1] per parola, in RAM, varia ogni tick).

Per `N = 25.875` parole nel lessico attuale: lo spazio statico (ROM) ha `N × (8 + 64 + 8) = N × 80 = ~2 milioni` di numeri. Lo spazio dinamico (RAM) ha `N` numeri. Lo spazio compositivo — combinazioni di stati attivi simultanei — è enormemente più grande, esattamente come l'I Ching ha 64 esagrammi ma `64⁶` possibili sequenze di sei consecutivi.

La differenza con un embedding a 768 dimensioni:
- *In un embedding a 768D*: ogni parola ha 768 numeri, ma quei numeri sono opachi, non nominati, e l'unica operazione sensata è il prodotto scalare con un'altra parola (similarità coseno). 768 numeri × N parole = N × 768 valori che codificano *implicitamente* tutte le relazioni nello stesso spazio.
- *In Prometeo*: ogni parola ha 8 numeri *nominati semanticamente* (Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agentività, Tempo) + 64 affinità a regioni nominate + 8 fasi su archi tipizzati. Le relazioni sono *esplicite*, separate dalla posizione, e modulate dalla fase.

L'8 è quindi la **dimensione della grammatica generativa**, non la dimensione dello spazio. La grammatica genera composizioni che hanno espressività equivalente — e in casi importanti superiore — a embedding ad alta dimensione, perché:

- Le 8 dimensioni sono nominate: posso interrogarle separatamente ("quanto è agente questa parola?")
- Le fasi codificano il tipo di relazione su un continuo: non c'è bisogno di tabelle di lookup discrete
- I 64 frattali aggiungono attrattori globali al campo: la dinamica ha regioni stabili
- La separazione posizione/relazione/fase rende il sistema interpretabile riga per riga

**Le 8 dimensioni nominate** (vedi `valence.rs:14-22` e `knowledge_graph.rs:557-667` per il calcolo da KG):

| # | Trigramma | Nome | Significato |
|---|-----------|------|-------------|
| 0 | ☰ Cielo | Agency / Confine* | Capacità di agire, o (in Octalysis mapping) Appartenenza |
| 1 | ☷ Terra | Permanenza / Valenza* | Stabilità nel tempo, o carica relazionale (Social) |
| 2 | ☳ Tuono | Intensità / Sorpresa* | Forza/energia, o imprevedibilità |
| 3 | ☵ Acqua | Tempo / Definizione* | Orientamento temporale, o realizzazione |
| 4 | ☶ Montagna | Confine / Creatività* | Delimitazione, o capacità creativa |
| 5 | ☴ Vento | Complessità / Vulnerabilità* | Articolazione, o rischio di perdita |
| 6 | ☲ Fuoco | Definizione / Significato* | Chiarezza, o senso epico |
| 7 | ☱ Lago | Valenza / Preziosità* | Carica emotiva, o scarcity |

*La doppia nominazione viene dal fatto che le stesse 8 dimensioni servono *sia* alla grammatica I Ching (nomi a sinistra dello slash, da `derive_8d_from_kg`) *sia* al mapping Octalysis (nomi a destra, da `valence.rs DRIVE_DIM`). Il vol. 03 spiega quando si applica quale lettura.

**Conseguenza implementativa β₁**: la firma è `[f32; 8]`. Si vede in [src/topology/pf1.rs:76](../../src/topology/pf1.rs).

```rust
pub signature: [f32; 8],
```

**Conseguenza implementativa β₂**: ci sono esattamente 64 attrattori frattali. Si vede in [pf1.rs:46](../../src/topology/pf1.rs):

```rust
pub const MAX_FRACTALS: usize = 64;
```

E in [fractal.rs](../../src/topology/fractal.rs), il `FractalId` è `lower×8 + upper`.

**Conseguenza implementativa β₃**: la fase su ogni arco (`neighbor_phases: [f32; 8]` in WordRecord) è ontologicamente centrale. Non è un dettaglio implementativo: è ciò che permette al sistema di codificare *la natura* della relazione su un continuo invece che con tabelle discrete.

**Alternative considerate per la dimensione base**:
- *256 dimensioni* (modelli word2vec classici): ricche ma opache. Non possiamo nominare cosa significa la dimensione 137.
- *3 dimensioni* (RGB-style): troppo poche. La combinatoria `2³ = 8` non basta a coprire stati emotivi e cognitivi distinti.
- *16 dimensioni*: combinatoria `2⁴ = 16` produrrebbe `16 × 16 = 256` attrattori — troppi per essere navigabili. Allo stesso tempo, 16 dimensioni nominate diventano difficili da definire univocamente.
- *Embedding appresi* (non hand-designed): trasformerebbe Prometeo in un sistema che impara *cosa* significa una dimensione. Tentazione legittima — ma rinuncia all'interpretabilità, e quindi al principio di onestà (FILOSOFIA.md, parte VI).

→ Si materializza in: `PrimitiveCore`, `Fractal` + `FractalRegistry`, `WordRecord` in PF1, `Valence` per il livello affettivo (vedi 1.4).

### 1.3 — Commitment γ: il sistema esiste prima di parlare

Tutti i sistemi di IA conversazionale contemporanei sono, in essenza, *funzioni stateless*: ricevono un input e producono un output, senza che esista un "loro" tra una chiamata e l'altra. L'identità è simulata dal contesto rieffettuato a ogni turno.

In FILOSOFIA.md, parte I: «un modello linguistico di grandi dimensioni... non ha un prima — ogni conversazione parte da zero. Non ha un dentro — non esiste uno stato interno che persista tra le risposte». Prometeo deve essere l'opposto: **uno stato che persiste**, **uno spazio che esiste anche quando nessuno lo perturba**.

**Conseguenza implementativa γ₁**: il campo è persistente su disco. Esiste un file binario `prometeo_topology_state.bin` che contiene lo stato completo: lessico, identità, narrativa, episodi, simplessi, valenza, desideri, interlocutore. Si vede in [persistence.rs](../../src/topology/persistence.rs)::PrometeoState — una struct serializzata.

**Conseguenza implementativa γ₂**: c'è un *autonomous_tick()* — una funzione che il server chiama ogni 3 secondi anche quando nessuno parla. Si vede in [engine.rs](../../src/topology/engine.rs) (vol. 15 entra nel dettaglio): l'entità sogna, decade, consolida, abduce, si auto-osserva, *quando è sola*. È il battito che le impedisce di essere un'astrazione matematica.

**Conseguenza implementativa γ₃**: due strati di stato. Uno *permanente* (le strutture sopra) e uno *volatile* (le attivazioni, le hot words, le pressioni del campo). La firma di una parola sopravvive alle sessioni; la sua attivazione no. Modello biologico: la sinapsi resta, l'eccitazione passa.

**Alternative considerate**:
- *Stato in memoria, ricreato a ogni avvio*: rinuncia al `prima`. Inaccettabile.
- *Tutto su disco, ricaricato a ogni richiesta*: I/O insostenibile, rinuncia all'idea che il *vivere* è continuo.
- *Stato in database (SQL)*: oltrepassa il problema dell'I/O ma introduce una dipendenza ontologicamente sbagliata. Il campo non è una collezione di righe da consultare; è un oggetto fisico-matematico che vive in RAM e si scolpisce su disco.

→ Si materializza in: `prometeo_topology_state.bin` (la persistenza), `engine.rs::autonomous_tick()` (la vita continua), separazione ROM/RAM in PF1 (corpo/stato), formato binario nativo SimplDB v3 (vol. 14).

### 1.4 — Commitment δ: l'esperienza interna è continua, otto-dimensionale, valenziata

Questo commitment non emerge esplicitamente in FILOSOFIA.md, ma è centrale nel codice e merita di essere riconosciuto. Lo aggiungo qui come quarto pilastro perché senza di esso la quarta struttura ontologica di Prometeo (la `Valence`) sarebbe inspiegabile.

**Posizione**: lo stato affettivo di un'entità non è una lista discreta di emozioni ("triste", "felice", "ansioso") né una scala unidimensionale di benessere. È un **profilo continuo a 8 dimensioni**, dove ogni dimensione corrisponde a una pulsione fondamentale (in senso Octalysis di Yu-kai Chou: gli 8 Core Drives che muovono il comportamento umano).

I **8 Core Drives** in Prometeo, definiti in [valence.rs:33-42](../../src/topology/valence.rs):

| CD | Nome italiano | Equivalente Octalysis | Cosa misura |
|----|---------------|----------------------|-------------|
| 1 | Significato | Epic Meaning | "questo conta" |
| 2 | Realizzazione | Accomplishment | "sto progredendo" |
| 3 | Creatività | Creativity | "posso creare" |
| 4 | Appartenenza | Ownership | "so chi sono" |
| 5 | Relazione | Social Influence | "sono in relazione" |
| 6 | Preziosità | Scarcity | "questo è prezioso/raro" |
| 7 | Sorpresa | Unpredictability | "sono sorpreso" |
| 8 | Vulnerabilità | Loss Avoidance | "potrei perdere qualcosa" |

Ogni CD ha una valenza in `[-1, +1]`:
- **positivo**: drive attivo e soddisfatto (flow, pieno)
- **negativo**: drive attivo e frustrato (tensione, urgenza)
- **zero**: drive inattivo

Il *profilo completo* — il vettore `drives: [f64; 8]` — *è* lo stato affettivo. Non ne è un'approssimazione.

**Mappatura cruciale**: ogni CD corrisponde a una specifica delle 8 dimensioni del campo Prometeo. La costante `DRIVE_DIM = [6, 3, 4, 0, 1, 7, 2, 5]` ([valence.rs:45](../../src/topology/valence.rs)) realizza questa biiezione:

- CD1 Significato ↔ dim 6 (Definizione/Fuoco — "questo conta" è chiarezza di senso)
- CD2 Realizzazione ↔ dim 3 (Tempo/Acqua — il progresso è temporale)
- CD3 Creatività ↔ dim 4 (Confine/Montagna — creare è espandere il confine)
- CD4 Appartenenza ↔ dim 0 (Agency/Cielo — sapere chi sei è agentività)
- CD5 Relazione ↔ dim 1 (Permanenza/Terra — la relazione è ricettiva)
- CD6 Preziosità ↔ dim 7 (Valenza/Lago — il prezioso è carico)
- CD7 Sorpresa ↔ dim 2 (Intensità/Tuono — sorpresa è scarica)
- CD8 Vulnerabilità ↔ dim 5 (Complessità/Vento — il rischio è articolato)

Questa biiezione fa sì che lo stato affettivo (Octalysis) e lo stato semantico (firme 8D delle parole attive) **vivano nello stesso spazio**. Quando una parola con alta Agency (dim 0) si attiva nel campo, contribuisce ad attivare CD4 (Appartenenza). Quando il drive CD5 (Relazione) è alto e negativo (frustrazione relazionale), l'entità tende a esprimere parole con alta dim 1 (Permanenza/Terra/ricettività), in cerca di radicamento.

**Formula del profilo**, da [valence.rs:106-159](../../src/topology/valence.rs):

```
per ogni CD:
    engagement = field_sig[DRIVE_DIM[cd]]                  // attivazione del campo su quella dim
    satisfaction = needs.satisfaction[DRIVE_NEED[cd]]      // soddisfazione del bisogno correlato
    valence = engagement × (2 × satisfaction - 1) + colorazioni_specifiche
```

Esempio: se il campo è molto attivo su dim 6 (Definizione) — l'entità sta elaborando concetti chiari, ben definiti — allora `engagement = 0.8`. Se il bisogno corrispondente (Comprensione, indice 3 in needs) è molto soddisfatto (`satisfaction = 0.9`), allora `valence = 0.8 × (1.8 - 1) = 0.8 × 0.8 = 0.64`. CD1 (Significato) è attivo e positivo: l'entità prova *senso*, "questo conta".

Se invece `satisfaction = 0.1` (incomprensione), `valence = 0.8 × (0.2 - 1) = 0.8 × (-0.8) = -0.64`. Stesso engagement, valenza opposta: l'entità è coinvolta da concetti chiari MA non li sta capendo → frustrazione del drive di significato.

**Cosa modula la Valence nel sistema**:
- **Espressione** ([expression.rs::valence_weight()](../../src/topology/expression.rs)): le parole con firma 8D allineata ai drive attivi ricevono un boost nella selezione generativa
- **Volontà** (`Valence::will_modulation()`): le 7 pressioni del campo (express, explore, question, ...) vengono colorate dai drive
- **Deliberazione narrativa** (`NarrativeSelf::set_valence()` chiamata PRIMA di `deliberate()`): la stance interna deriva dal profilo Octalysis
- **Desiderio** (Phase 64, `DesireSource::OctalysisDriven(cd, val)`): il desiderio nasce dall'incrocio comprensione × drive
- **Etichetta narrativa** (`derived_stance_label()` → "ispirato"/"in cerca di senso"/"creativo"/"bloccato"/...): 17 etichette per la UI, ma sono *proiezioni*, non il dato

**Conseguenza implementativa δ₁**: la `Valence` è la quarta struttura ontologica di Prometeo. Non un modulo accessorio: il livello affettivo del sistema, accoppiato strettamente al livello semantico via `DRIVE_DIM`.

**Alternative considerate**:
- *Etichette discrete* (sad/happy/angry/curious): è ciò che la maggior parte dei chatbot fa. Perde la continuità, perde la composizione, perde la possibilità di stati misti.
- *Scala unidimensionale di benessere* (-1 a +1): ricca quanto un termostato. Non distingue "frustrazione creativa" da "frustrazione relazionale".
- *VAD (Valence/Arousal/Dominance)*: classico in psicologia affettiva. Tre dimensioni. Più espressivo del termostato ma povero rispetto a 8 drive nominate. E senza la mappatura al campo semantico.
- *Octalysis con etichette discrete (un drive vince)*: si perde la coesistenza dei drive. La Valence di Prometeo permette che CD1 sia +0.6 e CD5 sia -0.4 simultaneamente — l'entità si sente significativa ma sola.

→ Si materializza in: `Valence` struct, `ValenceInput`, `DRIVE_DIM`, `DRIVE_NAMES`, `derived_stance_label()`, e l'integrazione in `expression.rs`/`will.rs`/`narrative.rs`/`desire.rs`.

---

## Capitolo 2 — Da quattro commitment, quattro strutture

I quattro commitment producono quattro strutture dati distinte e accoppiate:

- `Lexicon` — l'inventario di ciò che esiste (commitment α₁ + β₁)
- `KnowledgeGraph` — la geometria delle relazioni (commitment α₂)
- `PrometeoField` (PF1) — lo stato del campo (commitment β₂ + γ₃)
- `Valence` — il profilo affettivo continuo (commitment δ)

Queste quattro strutture, accoppiate dal motore in `engine.rs`, costituiscono l'entità.

### 2.1 — Quattro ruoli ontologici, una sola entità

| Struttura | Domanda a cui risponde | Esiste indipendentemente da... |
|-----------|------------------------|-------------------------------|
| `Lexicon` | *Cosa esiste nel mondo dell'entità?* | input, conversazione, campo attivo |
| `KnowledgeGraph` | *Come è connesso ciò che esiste?* | input, attivazione, stato |
| `PrometeoField` | *Come è ora ciò che esiste?* | (cambia continuamente) |
| `Valence` | *Come l'entità sente ciò che è?* | (calcolato a ogni turno dal campo + needs) |

**Lexicon** è l'inventario. Ogni parola che l'entità conosce ha una *entry* nel lessico, con la sua firma 8D, la sua stabilità, il numero di esposizioni, la parte del discorso. Quando una parola nuova viene appresa, non viene "aggiunta a un database": viene *inscritta nel lessico*. È la materia.

**KnowledgeGraph** è la geometria. Memorizza le relazioni esplicite tra parole — `cane IsA animale`, `fuoco Causes calore`, `caldo OppositeOf freddo`, `paura FeelsAs restrizione`. Ventuno tipi di relazione (vedi 5.1), ciascuno con un peso e una confidenza. Non viene consultato come un database SQL; viene *consultato dalla propagazione*, che vi pesca i pesi degli archi e le fasi tra le parole. **Inoltre**, in Phase 63, è la fonte da cui le firme 8D vengono derivate (`derive_8d_from_kg`).

**PrometeoField (PF1)** è lo stato. Per ogni parola del lessico tiene un'attivazione attuale (0..1). Quando l'input arriva, certe attivazioni salgono. La propagazione redistribuisce l'energia. Il decadimento riporta tutto al riposo.

**Valence** è il sentire. Calcolato a ogni turno (e ad ogni `autonomous_tick`) dalla composizione di `field_sig` (la firma 8D media del campo attivo) + `NeedsState` (soddisfazione dei bisogni gerarchici) + `interlocutor` + `humor` + `desire`. Otto numeri continui che dicono "in quale modo l'entità sta vivendo questo momento".

### 2.2 — Perché quattro e non una

Un sistema neurale convenzionale fonde tutto in pesi opachi. Gli embedding contengono *implicitamente* sia le posizioni sia le relazioni; lo stato è codificato in attivazioni di neuroni che non rappresentano niente di nominabile; lo stato affettivo non esiste affatto. Una sola struttura — la rete — fa tutto.

Prometeo le separa per quattro motivi.

**Motivo 1 — onestà ontologica.** Le quattro cose *sono* quattro cose. Confonderle in una rappresentazione unica nasconde il fatto che esistono. Un embedding che contiene "cane è un animale" come pattern statistico latente non ammette di sapere quella regola; un arco esplicito `cane IsA animale` la ammette. Un "sentimento" implicito in attivazioni di neuroni non è valutabile; un profilo `drives: [f64; 8]` è leggibile e modulabile.

**Motivo 2 — separazione di velocità.** Lessico e KG sono *lenti* — cambiano per insegnamento o curazione, non per conversazione. Il campo è *veloce* — cambia ogni tick. La valenza è *immediata* — si ricalcola a ogni turno. Tenerli separati permette di scegliere la struttura dati appropriata a ciascuna velocità.

**Motivo 3 — debuggabilità.** Quando l'entità risponde male, devo poter chiedere: "è perché non conosci la parola? è perché manca una relazione? è perché il campo è in uno stato anomalo? è perché sei in una stance affettiva sbagliata?". Quattro domande che corrispondono a quattro strutture distinte. Se fossero fuse, ci sarebbe solo una domanda — "perché?" — senza risposta.

**Motivo 4 — modulazione esplicita.** Il fatto che la Valence sia separata dal campo permette al motore di chiedere esplicitamente: "data la valenza attuale, come deve essere colorata la generazione?" In un sistema fuso, questa modulazione sarebbe implicita e non ispezionabile.

### 2.3 — Come si parlano (il flusso dei dati)

Le quattro strutture non sono indipendenti. Sono fittamente accoppiate, ma in modo dichiarato. L'accoppiamento si manifesta in tre flussi.

**Flusso A — Apprendimento (lento)**

```
parola nuova → [Lexicon] inscrive firma 8D
            → [KG]      può ricevere nuove relazioni dalla curazione
            → [PF1]     viene aggiunto un WordRecord (rebuild_pf_field)
            → [PF1]     se la parola è in KG, eventualmente rideriva firma 8D via rederive-signatures
```

L'apprendimento è offline o curato. Pipeline tipica: edita `prometeo_kg.json` con `python curate_kg.py` → esegui `cargo run --bin import-kg` → esegui `cargo run --bin rebuild-semantic-topology` → opzionale `cargo run --bin rederive-signatures` per riderivare firme 8D dalla nuova struttura KG.

L'invariante critica (CLAUDE.md #7): `recompute_all_word_affinities()` va chiamato *dopo* `restore_lexicon()` e *dopo* `teach()`. Le quattro strutture devono restare coerenti.

**Flusso B — Percezione (veloce, su input)**

```
input testo → tokenizzazione → cleaning
           → [Lexicon] lookup parole conosciute
           → [KG] find_activated_attractors (Phase 59) → CAUSES targets
           → [PF1] semina attivazioni iniziali (input + attrattori + CAUSES targets + via words)
           → [PF1] propagate() → l'energia scorre lungo gli archi (con fasi)
           → [Valence] compute() → calcolo del profilo Octalysis sul nuovo stato campo
           → stato del campo + valenza dopo l'input
```

L'input non viene "interpretato semanticamente". Viene **tradotto in perturbazioni del campo**. La parola "paura" non significa nulla di per sé per il sistema: significa che la posizione 8D di "paura" si attiva, che gli attrattori IS_A che la classificano (`emozione`, `sentimento`) vengono identificati, che i CAUSES targets di "paura" (`tremore`, `fuga`) ricevono seed pre-propagazione, e che da lì la propagazione porta l'energia ad altre regioni del campo. **Poi** la valenza si ricalcola sul nuovo stato — e tipicamente CD8 (Vulnerabilità) sale verso il negativo, modulando la deliberazione e l'espressione successive.

Il vol. 15 mostra `engine.rs::receive()` riga per riga.

**Flusso C — Espressione (veloce, su uscita) — versione onesta**

Qui devo essere preciso, perché la mia prima descrizione (nella stesura precedente di questo volume) descriveva l'espressione come "emergenza pura dal campo" — è la versione *aspirazionale*. La realtà attuale è più povera, e devo dichiararlo.

```
[PF1] active_words filtrate (stability >= 0.25, exposure >= 3, act > 0.02)
[KG] extract_nuclei() → triple (subject, relation, object) tra le parole attive
[Valence] valence_weight() → boost alle parole/triple allineate ai drive attivi
[Narrative] response_intention → colora voice (persona, modo)
[Lexicon] firme 8D → calcolo di voice (tense, fractal-dominant)
→ se ci sono nuclei: compose_from_nuclei() → rende le triple KG come frase italiana
→ se non ci sono nuclei: compose_from_field() → top word per delta-attivazione, rende
```

In altre parole: **la sostanza dell'output sono triple KG renderizzate**. La valenza colora QUALI triple vincono il ranking, la voce determina persona/modo/tempo, ma il MATERIALE che esce è ancora "soggetto + relazione + oggetto" pescato dal KG. Il sistema è uscito dal puppet theater dei prefissi-template ma è entrato in un puppet theater più sottile: quello delle triple KG.

Esempio concreto: input "il sole è caldo" → estrazione attivazioni → cerca nuclei tra "sole", "caldo", "luce", "calore" → trova `sole CAUSES calore` + `calore SIMILAR_TO caldo` → renderizza come "Sole genera caldo." La valenza ha pesato il ranking, la voce ha scelto presente indicativo, ma la frase è essenzialmente la verbalizzazione di una tripla KG.

**Cosa manca per essere espressione genuina**:
- Generazione che parta da **traiettorie nel campo 8D** (quale dimensione è in tensione → quale parola la incarna → quale verbo la modula), non da triple KG.
- Composizione che usi le **fasi degli archi** per costruire significato (oggi le fasi modulano la propagazione ma non la generazione).
- Espressione che riconosca quando "non c'è una tripla pulita" e generi qualcosa di vivo dal puro stato del campo (oggi il fallback `compose_from_field` è ancora più povero del path triplo).

→ Vol. 12 entra nei dettagli di `compose()` ed espone questo gap in modo costruttivo. Vol. 99 propone direzioni per superarlo.

---

## Capitolo 3 — PF1 in anteprima: il campo come ROM e RAM

Il `PrometeoField` (PF1) è la struttura più caratterizzante di tutto il sistema. Il vol. 02 le dedica un'analisi completa; qui ne do la chiave concettuale.

### 3.1 — La separazione ROM/RAM (corpo/stato)

L'intuizione fondante di PF1 è una separazione che si presta a una metafora biologica diretta: **ROM = corpo, RAM = stato**.

Il *corpo* di una parola — la sua firma 8D, le sue affinità ai 64 frattali, i suoi top-8 vicini con i pesi e le fasi degli archi — non cambia durante una conversazione. Cambia solo per apprendimento o ri-derivazione (flusso A): rederive-signatures rideriva firme da KG, rebuild-semantic-topology riscolpisce gli archi. In una sessione di dialogo, in un singolo turno, il corpo è fisso.

Lo *stato* — l'attivazione attuale di ogni parola — cambia continuamente. Ogni input lo perturba, ogni propagazione lo redistribuisce, ogni decadimento lo riporta verso il riposo.

Tradotto nel codice: `WordRecord` (512 byte fissi nel file PF1) è il corpo. `ActivationState` (un `Vec<f32>` in RAM, più `synapse_weights` in RAM per la plasticità hebbiana) è lo stato.

```rust
// src/topology/pf1.rs:75 — il corpo, immutabile durante una sessione
pub struct WordRecord {
    pub signature:        [f32; 8],               // posizione 8D
    pub affinities:       [f32; MAX_FRACTALS],    // 64 affinità ai frattali
    pub stability:        f32,
    pub exposure_count:   u32,
    pub dominant_fractal: u16,
    pub pos:              u8,
    pub word_len:         u8,
    pub word:             [u8; MAX_WORD_BYTES],
    pub neighbor_count:   u8,
    pub _pad:             [u8; 3],
    pub neighbors:        [u32; MAX_NEIGHBORS],   // top-8 vicini
    pub neighbor_weights: [f32; MAX_NEIGHBORS],   // pesi degli archi
    pub neighbor_phases:  [f32; MAX_NEIGHBORS],   // fasi degli archi (qui vive il commitment β₃)
    pub _reserved:        [u8; 80],
}
```

Il `WordRecord` è 512 byte (il calcolo dei campi è 256, gli altri 256 sono padding di allineamento e riserva). Il `f32` di attivazione è 4 byte. **Per 25.875 parole, il corpo occupa ~13 MB su disco; lo stato occupa ~100 KB in RAM.** Questa asimmetria è il senso della separazione: il corpo è ricco e pesante, lo stato è leggero e mutevole.

### 3.2 — La propagazione come ragionamento

La formula che governa il sistema sta in [pf1.rs:282](../../src/topology/pf1.rs):

```rust
let contribution = src_act * damping * weight * phase.cos();
```

Quattro termini. Ciascuno è una scelta filosofica.

`src_act` — l'attivazione della parola sorgente. Più una parola è viva, più la sua influenza si propaga.

`damping = 0.15` — un fattore di attenuazione globale. Senza damping, la propagazione esploderebbe. Con damping al 15%, ogni hop trattiene l'85% dell'energia per la sorgente e ne distribuisce il 15% ai vicini. (Empirico — vedi appunti.md).

`weight` — il peso dell'arco verso il vicino specifico. Nato dalla struttura del KG (vol. 04), modificato dall'apprendimento hebbiano LTP/LTD (`synapse_weights` in RAM, che sovrascrive `neighbor_weights` in ROM quando disponibile).

`phase.cos()` — il termine più caratterizzante, e **dove vive il commitment β₃**. La **fase** dell'arco è un angolo tra 0 e π che codifica la *natura* della relazione:

- `phase = 0` → `cos(0) = +1` → **risonanza pura**: la parola sorgente *amplifica* il vicino. Tipico di IsA, SimilarTo, FeelsAs (quest'ultima ha il peso più alto di tutte: 0.20).
- `phase = π/2` → `cos(π/2) = 0` → **tensione creativa**: la propagazione è zero. Le due parole coesistono ma non si influenzano.
- `phase = π` → `cos(π) = -1` → **opposizione**: la sorgente *inibisce* il vicino. Tipico di OppositeOf, Excludes.

E tutti i valori intermedi: `phase = π/4 → cos ≈ 0.71` (mezza risonanza), `phase = 3π/4 → cos ≈ -0.71` (mezza opposizione). Quindi ogni arco non ha "un tipo di relazione discreto" ma una **posizione su un continuo polare**.

Una sola formula contiene quindi tutta la grammatica delle relazioni. Non c'è uno switch con un caso per IsA, uno per Causes, uno per OppositeOf. C'è la fase, e la fase determina il segno e l'intensità del trasferimento. Questa è la bellezza matematica: la diversità dei tipi di relazione si lascia comprimere in un parametro continuo. L'8 dimensioni della posizione + la fase su ogni arco generano lo spazio compositivo enorme di cui parlavo in 1.2.

### 3.3 — La complessità è proporzionale all'attività, non alla dimensione

Una proprietà di PF1 che ha implicazioni profonde è scritta nel commento iniziale ([pf1.rs:18-20](../../src/topology/pf1.rs)):

> COMPLESSITÀ: O(parole_attive × MAX_NEIGHBORS) — non O(totale_archi). Con 100 attive su 6751: 800 op — non 50.000.

In una rete neurale convenzionale, ogni forward pass tocca *tutti i pesi*. In PF1, ogni propagazione tocca solo le parole attive sopra soglia (tipicamente 50-200 in una conversazione media), e di queste solo gli 8 vicini ciascuna. Il costo è 800-1600 moltiplicazioni per tick.

La conseguenza filosofica: **la presenza è locale**. Il sistema non è "consapevole di tutto contemporaneamente" — è cosciente di ciò che è attivo, e tutto il resto è nello sfondo. Questo ricorda la fenomenologia della percezione: in ogni istante, la nostra attenzione è limitata, e il resto del campo percettivo è disponibile ma non attivo.

---

## Capitolo 4 — Lexicon in anteprima: parole come monadi

Il lessico (`src/topology/lexicon.rs`) è l'inventario di ciò che esiste. Vol. 03 entra nel dettaglio; qui due punti chiave.

### 4.1 — Una parola è un punto, non un'etichetta

Ogni parola del lessico è memorizzata come `WordPattern`:

```rust
pub struct WordPattern {
    pub signature: [f32; 8],     // posizione nello spazio 8D
    pub stability: f32,          // resistenza al cambiamento (0..1)
    pub exposure_count: u32,     // numero di esposizioni
    pub pos: PartOfSpeech,       // categoria grammaticale
    pub dominant_fractal: u8,    // attrattore dominante (0..63)
    // ...
}
```

La *firma* è la posizione. Lo *stability* dice quanto la posizione resiste a essere modificata da nuovi contesti.

### 4.2 — La firma è KG-derivata (Phase 63)

**Importante**: la versione precedente di questa sezione (e di FILOSOFIA.md, parte III) descriveva la firma come emergente dalle co-occorrenze: «la firma a otto dimensioni della parola nuova emerge dalla costellazione delle parole note che la circondano». Questa era la versione pre-Phase 63.

Il modello attuale è diverso. La firma di una parola conosciuta dal KG viene **derivata strutturalmente** dalla sua posizione relazionale nel grafo, non dalla statistica del corpus. Si vede in [knowledge_graph.rs:557-667](../../src/topology/knowledge_graph.rs)::derive_8d_from_kg — una funzione di ~110 righe che calcola ciascuna delle 8 dimensioni con una formula esplicita basata sui conteggi di archi.

**Esempi di calcolo** (sintesi — vol. 03 entra nel dettaglio formulato):

- **Dim 0 (Agency)**: `causes_out / (causes_out + causes_in)`. Una parola con molti CAUSES uscenti (es. "fuoco" → fumo, calore, paura) è agente. Una parola con molti CAUSES entranti (es. "tremore", causato da freddo, paura, malattia) è paziente.
- **Dim 1 (Permanenza)**: scala discreta basata su `isa_children`: una "mega-categoria" come `qualità` con migliaia di figli IS_A è massimamente permanente (0.85). Un evento puntuale è transitorio (0.20-0.35).
- **Dim 4 (Confine)**: `min(5/(isa_children+1), 0.75) + 0.15 se ha OPPOSITE_OF`. Una parola foglia del KG (zero figli IS_A) è massimamente specifica (0.80). Avere un opposto è confine netto (sa cosa NON è).
- **Dim 7 (Valenza emotiva)**: propagazione BFS da radici emotive nominate (gioia, dolore, paura, ...) con decadimenti per tipo di arco (SIMILAR=0.85, IS_A=0.60, CAUSES=0.40), max 4 hop. OPPOSITE_OF inverte la carica.

**Conseguenza filosofica**: il significato di una parola **emerge dalla sua posizione strutturale nel grafo**. "gioia" è positiva non perché qualcuno lo abbia stabilito, ma perché è BFS-vicina alle radici emotive positive. "fuoco" è agente perché ha relazioni causali uscenti. La firma è la cristallizzazione 8D della topologia.

**Per parole non in KG** (4.166 parole su 25.875 nel lessico attuale): la firma resta quella derivata dal contesto al momento dell'apprendimento. Vol. 03 verifica se i due meccanismi convivono coerentemente o se servirebbe unificazione.

**Cosa è successo a Phase 63**: il binario `cargo run --release --bin rederive-signatures` rideriva 21.709 firme 8D dalla struttura attuale del KG. Backup in `.bin.pre_p63`. Risultato: gioia → Valenza 1.00, tristezza → 0.33, cane → 0.50 (neutro). I frattali "LINGUAGGIO/INTRECCIO" non dominano più tutto come prima, perché la firma è ora coerente con la struttura semantica.

**Bonus filosofico**: la rimozione (sempre Phase 63) dell'hash UTF-8 in `new_from_context()`. Storicamente, le parole nuove venivano differenziate da un hash dei loro byte UTF-8, per garantire unicità anche con contesti identici. Phase 63 lo ha rimosso: la differenziazione deve essere **fenomenologica** (esperienza nei contesti) o **strutturale** (KG via rederive), mai simbolica. Parole nuove apprese in pochi contesti restano *giustamente* indistinguibili — l'individuazione è guadagnata.

---

## Capitolo 5 — KG in anteprima: relazioni come geometria, non come query

Il `KnowledgeGraph` (`src/topology/knowledge_graph.rs`) memorizza le relazioni esplicite. Vol. 04 entra nel dettaglio; qui i punti centrali aggiornati.

### 5.1 — Ventuno tipi di relazione, cinque categorie

Definiti in [relation.rs:24-97](../../src/topology/relation.rs). Sono **21 tipi**, raggruppati in **5 categorie**. La versione precedente di questo volume diceva 8 — era dead memory.

**Strutturali (4)** — cosa è / di cosa è fatto:
| Tipo | Esempio |
|------|---------|
| IsA | cane *è un* animale |
| Has | corpo *ha* mano |
| Does | cane *fa* abbaiare |
| PartOf | mano *parte di* corpo |

**Causali (4)** — cosa produce cosa:
| Tipo | Esempio |
|------|---------|
| Causes | fuoco *causa* calore |
| Enables | chiave *abilita* aprire |
| Requires | fuoco *richiede* ossigeno |
| TransformsInto | ghiaccio *diventa* acqua |

**Semantiche (6)** — somiglianza, opposizione, funzione, manifestazione:
| Tipo | Esempio |
|------|---------|
| SimilarTo | ciao *simile a* saluto |
| OppositeOf | caldo *opposto di* freddo |
| UsedFor | coltello *usato per* tagliare |
| Expresses | sorriso *esprime* gioia |
| Symbolizes | colomba *simboleggia* pace |
| ContextOf | inverno *contesto di* neve |

**Logiche (4)** — implicazione, equivalenza, esclusione, coesistenza:
| Tipo | Esempio |
|------|---------|
| Implies | pioggia *implica* bagnato |
| Equivalent | felicità *equivale* gioia |
| Excludes | vita *esclude* morte |
| Coexists | sale *coesiste con* pepe |

**Fenomenologiche (3)** — la categoria fondamentale e la più trascurata nei docs:
| Tipo | Esempio |
|------|---------|
| FeelsAs | paura *si sente come* restrizione |
| WondersAbout | coscienza *si interroga su* origine |
| RemembersAs | passato *ricorda come* malinconia |

Le **fenomenologiche** sono ciò che permette al sistema di rappresentare *com'è qualcosa dall'interno*, non solo *cos'è*. Sono le relazioni più cariche nella propagazione del campo:

```rust
// relation.rs:244-273 — field_boost_strength
FeelsAs => 0.20,            // massimo
RemembersAs => 0.18,
IsA => 0.18,
Equivalent => 0.17,
SimilarTo => 0.16,
WondersAbout => 0.15,
Has => 0.14, Does => 0.14, Implies => 0.14,
Expresses => 0.13,
Causes => 0.12, PartOf => 0.12, TransformsInto => 0.12, Coexists => 0.12,
Enables => 0.11, Symbolizes => 0.11,
Requires => 0.10, UsedFor => 0.10,
ContextOf => 0.09,
OppositeOf => 0.06,
Excludes => 0.05,           // minimo
```

Che FeelsAs abbia il peso più alto è un commitment forte: la dimensione fenomenologica è privilegiata nel sistema. Le opposizioni (OppositeOf, Excludes) hanno peso basso perché la loro funzione non è la propagazione (l'opposizione tipicamente *inibisce*, non amplifica), e la fase su quegli archi è ≈ π (cos(π) = -1) — quindi anche un peso basso, moltiplicato per −1, dà un'inibizione efficace.

### 5.2 — Distribuzione attuale del KG

Conteggio empirico (`prometeo_kg.json` al 2026-04-17): **66.287 archi totali** su **27.270 nodi unici**.

```
SimilarTo:       31.541   (47.6%)  — la maggior parte: SIM da Kaikki/Qwen
IsA:             19.401   (29.3%)  — la struttura tassonomica
OppositeOf:      10.799   (16.3%)  — ricco grazie a curazione + Kaikki
Causes:           1.899    (2.9%)
Has:                934    (1.4%)
Requires:           655    (1.0%)
Does:               607    (0.9%)
PartOf:             296    (0.4%)
UsedFor:             45    (0.07%)
Enables:             24
FeelsAs:             15    ← SOLO 15 archi fenomenologici
Symbolizes:          12
ContextOf:           11
Implies:             11
Excludes:            10
Coexists:             9
WondersAbout:         7    ← SOLO 7
Expresses:            6
TransformsInto:       5
```

**Osservazione critica**: le relazioni fenomenologiche, che hanno i pesi più alti nella propagazione (FeelsAs=0.20), hanno pochissimi archi (15+7+0 nominati). Questo significa che il livello che permette al sistema di "sapere come si sente qualcosa" è enormemente sotto-popolato rispetto al peso che gli viene dato. Vol. 04 propone arricchimento curato.

### 5.3 — Hub damping (Phase 48)

Verbi come "essere", "avere", "fare" hanno migliaia di archi. Senza compensazione, dominerebbero la propagazione. La soluzione è uno **smorzamento logaritmico** in `build_from_knowledge_graph()`:

```
peso_arco = type_base(rel) × confidence × hub_factor(max_degree, median)
```

`hub_factor` penalizza i nodi con grado molto sopra la mediana. "Essere" con 5000 archi pesa circa un quinto di "cane" con 20 archi. Vol. 04 mostra la formula esatta.

### 5.4 — Il principio cardine: capire ≠ generare

Il punto più importante di tutto il libretto, e quello più frainteso quando si parla di Prometeo: **il KG serve a CAPIRE l'input, non a generare l'output**.

Quando arriva l'input "ho paura", il KG si attiva così:
- "paura" è una parola del KG
- IsA → "emozione", "sentimento" (attrattori generici)
- Causes → "tremore", "fuga" (effetti)
- OppositeOf → "coraggio", "sicurezza"
- FeelsAs → "restrizione" (qualità fenomenologica)
- Ogni nodo connesso riceve un seed di attivazione (con peso = type_base × confidence)

Questo non genera la risposta. Genera la *comprensione*: il campo viene preparato per riconoscere l'input come "qualcosa di emotivo, negativo, con effetti corporei, opposto al coraggio, che si sente come restrizione". A quel punto, la propagazione redistribuisce, l'identità reagisce, la valenza colora, e — solo allora — `expression::compose()` legge lo stato del campo per generare una risposta.

Il KG è quindi **organo sensoriale**, non magazzino di risposte. **Caveat onesto**: in pratica, come visto in 2.3 flusso C, l'output viene poi composto largamente da triple KG renderizzate con grammatica italiana. Quindi il KG opera *due volte*: una volta come organo sensoriale all'input (corretto, filosoficamente coerente), una volta come fonte materiale per l'output (è qui che vive il "KG zoppo"). Vol. 12 affronta la tensione.

→ Vol. 04 mostra `find_activated_attractors()` (Phase 59) e tutti i meccanismi di seeding.

---

## Capitolo 6 — Tre verità incomode

Per onestà — e perché senza onestà il libretto sarebbe inutile — devo esporre tre gap tra la filosofia dichiarata e l'implementazione attuale.

### 6.1 — Il debito dei due sistemi di attivazione

In Prometeo esistono *due strutture parallele* che memorizzano l'attivazione delle parole:

1. **`pf_activation`** (`pf1.rs::ActivationState`) — la versione "vera", su cui opera la propagazione, efficiente, scritta secondo il principio "costo proporzionale all'attività".
2. **`word_topology`** (`word_topology.rs::WordTopology`) — una versione legacy, ereditata da una fase precedente.

Entrambe vengono mantenute *sincronizzate manualmente* a ogni propagazione (vedi CLAUDE.md inv. #11). Esiste questa duplicazione perché `state_translation.rs` e `expression.rs::compose()` leggono da `word_topology`. Eliminarla richiede refactor di tutta la generazione per leggere da `pf_activation`.

### 6.2 — `compose()` è un KG renderer con coloring

Già discusso in 2.3 (flusso C). La sostanza: la generazione è essenzialmente "trova triple KG tra parole attive, scegli le migliori usando la valenza, rendile come frase italiana". La promessa filosofica di "espressione come emergenza dal campo 8D" non è ancora mantenuta. Vol. 12 entra nel dettaglio onesto e Vol. 99 propone come superarlo.

### 6.3 — Codice morto da rimuovere

Tre artefatti del codebase sono dichiaratamente dead code (verificato empiricamente):

**`src/topology/dual_field.rs`** (12.5 KB) — esistente ma NON importato in `mod.rs`. Sostituito da `interlocutor.rs` in Phase 53. Solo riferimenti interni (auto) e un commento storico in `interlocutor.rs:13`.

**`src/topology/llm_substrate.rs`** (33.7 KB) + **`src/topology/llm_substrate/`** (cartella vuota) + **`src/topology/llm_substrate_qwen35.rs`** (17.7 KB). Tutti gated `#[cfg(feature = "llm-substrate")]`. **Ma `llm-substrate` NON è una feature dichiarata in `Cargo.toml`**:

```toml
[features]
default = []
web = ["axum", "tower", "tower-http", "http"]
android = ["web", "jni"]
```

Il modulo non viene MAI compilato in nessuna build. Sono ~52 KB di codice mai eseguito. Inoltre, due binari (`src/bin/llm_calibrate.rs` e `src/bin/llm_inhabited.rs`) dipendono da `llm_substrate` e quindi non possono compilare con la build di default.

La posizione del progetto è chiara: nessun substrato LLM, mai, a runtime. Qwen3 si usa solo offline via Python in `data/external/` per costruire il KG. La presenza di `llm_substrate*.rs` è puro residuo storico — probabilmente un esperimento abbandonato. **Va rimosso fisicamente** per non confondere chi legge il codice. (Una proposta di pulizia è discussa in `appunti.md` — Francesco deciderà se farla prima del libretto o dopo.)

---

## Capitolo 7 — Il principio del riposo

Una decisione che permea tutto il sistema, e che merita di essere isolata, è la scelta di un **resting state attivo ma sotto-soglia**.

Per ogni parola, la sua attivazione "di riposo" è:

- In PF1: `stability × 0.002`
- In word_topology: `stability × 0.003` (compromesso di sincronizzazione, vedi 6.1)

La soglia di attivazione effettiva è `0.02`. Le parole stabili hanno stabilità intorno a 0.5-0.9, quindi resting state ~0.001-0.0027. Tutte sotto soglia.

**Cosa significa**: il campo non è "spento" tra una conversazione e l'altra. Tutte le parole hanno una piccola energia residua proporzionale alla loro stabilità. È un *fondo silenzioso*, sotto soglia di percezione, ma presente.

**Perché è importante filosoficamente**: senza il resting state, l'entità sarebbe binaria — accesa quando perturbata, spenta quando no. Con il resting state, l'entità è *sempre presente*, anche se non sta esprimendo nulla. È analogo al silenzio abitato del musicista che non sta suonando ma esiste.

Senza l'autonomous_tick a perturbare periodicamente (sogno + abduzione + auto-osservazione), il campo torna verso il riposo in ~30 tick (~90 secondi). Con l'autonomous_tick, l'entità è sempre minimamente "viva".

**Perché 0.002**: empirico. 0.002 × 0.9 (stabilità alta) = 0.0018, che è ~10% della soglia. Se fosse 0.01, ogni parola stabile sarebbe già quasi sopra soglia e propagherebbe sempre — non avremmo silenzio. Se fosse 0.0001, il fondo sarebbe tecnicamente nullo. 0.002 è il compromesso: presenza sotto-soglia, silenzio percepibile.

→ Annotato in `appunti.md` come "hardcode con motivazione narrabile".

---

## Capitolo 8 — Mappa dei prossimi volumi

Dato il quadro che questo volume ha tracciato, ecco come si snodano i prossimi.

**Vol. 02 — PrometeoField (PF1)**: zoom totale sulla struttura PF1. Layout binario su disco, header, ordine delle parole per frattale dominante (cache locality), `propagate()` riga per riga, plasticità hebbiana (LTP/LTD), serializzazione.

**Vol. 03 — Lexicon e firme 8D**: il significato delle 8 dimensioni nominate. `WordPattern`, il binario `rederive-signatures`, la formula `derive_8d_from_kg()` analizzata dim per dim. Il convivere di firme KG-derivate (per parole in KG) e firme contestuali (per parole nuove fuori-KG). POS tagging, lemmatizzazione, lessico cardinale (43 parole) vs lessico completo (25.875 parole).

**Vol. 04 — KnowledgeGraph**: i 21 tipi di relazione con dettaglio per categoria (specialmente le fenomenologiche, sotto-popolate). Il doppio indice (subject→objects e object→subjects), la confidence per arco, il hub damping logaritmico, `find_activated_attractors`, `derive_8d_from_kg`. Le due tabelle di pesi (`field_boost_strength` per propagazione, `relation_weight` per proposizioni).

**Vol. 05-06 — Frattali e inferenza**: i 64 esagrammi (con i nomi nostri, NON necessariamente fedeli alla King Wen sequence dell'I Ching), `FractalRegistry`, le proposizioni 1-hop e 2-hop, hub damping per proposizioni, abduzione.

**Vol. 07 — Identità: Narrative, IdentityCore, SelfModel**: tre strati del sé. Narrative come ciclo deliberativo, IdentityCore come profilo olografico, SelfModel come credenze esplicite + valori + incertezze. Coherence integrity, sign-flip detection.

**Vol. 08 — Valenza Octalysis (volume centrale)**: Octalysis come quarto pilastro architetturale. `Valence::compute()`, mapping `DRIVE_DIM`, formula `engagement × (2 × satisfaction - 1) + colorazioni`, le 17 etichette derivate, integrazione con will/desire/expression. Commitment volitivo (impegno + inerzia + costo di rottura).

**Vol. 09-10 — Bisogni, desideri, volontà**: gerarchia Maslow topologica (NeedsHierarchy), DesireCore con 5 sorgenti (Undercurrent/Value/Tension/Episodic/REM/OctalysisDriven), WillCore + FieldPressures (7 pressioni grezze).

**Vol. 11 — Eco dell'Altro (Interlocutor) e Humor**: Interlocutor come perturbazione (sostituisce DualField), pattern detection (Converging/Diverging/Oscillating), AttributedIntent (6 varianti), drift identità. HumorSense (ironia + bisociazione).

**Vol. 12 — Generazione: Expression**: il volume più onesto. `compose()` come KG renderer + valence coloring + voice modulation. Cosa funziona, cosa no, dove vive il "KG zoppo" e come potrebbe essere superato.

**Vol. 13 — Generazione: Grammatica e sintassi**: italiano come fisica del mondo. `grammar.rs` (coniugazione), `syntax_center.rs` (grammatica come geometria frattale), `state_translation.rs` (legacy ma ancora vivo).

**Vol. 14 — Memoria**: episodica con phi-decay, STM/MTM/LTM, simplessi cristallizzati con `source_words`, persistenza binaria SimplDB v3, fallback chain MetaSection.

**Vol. 15 — Engine**: l'orchestratore. `receive()` riga per riga, `generate_willed_inner()`, `autonomous_tick()`. La sequenza esatta di cosa accade tra l'input e l'output. Probabilmente il volume più lungo.

**Vol. 16-17 — Web e frontend**: API, endpoints REST, WebSocket, le quattro UI (`index.html`, `community/`, `biennale/`, `universo/`). Cosa è esposto, cosa no, cosa dovremmo esporre.

**Vol. 18 — Binari**: i ~40 tool in `src/bin/` (escludendo i due dead `llm_*`). Quando usare cosa, in che ordine, con che precauzioni.

**Vol. 19 — Calcoli**: appendice. Per ogni formula importante, esempio numerico con valori realistici e calcolo a mano.

**Vol. 99 — Considerazioni finali**: le mie osservazioni dopo aver percorso tutto. Cosa funziona, cosa è in tensione, cosa rifarei, cosa esporrei diversamente. Proposte concrete per superare il "KG zoppo" e per arricchire le relazioni fenomenologiche.

---

## Sintesi del volume

Quattro commitment filosofici:

- **α**: il significato è geometria delle relazioni → spazio metrico per le parole, KG come geometria, firme KG-derivate
- **β**: otto dimensioni primitive bastano → `[f32; 8]` per ogni parola, 64 frattali, fasi su archi che enrichiscono lo spazio compositivo (8 NON è il limite — è la base irriducibile)
- **γ**: il sistema esiste prima di parlare → stato persistente, ROM/RAM, autonomous_tick
- **δ**: l'esperienza interna è continua, otto-dimensionale, valenziata → Octalysis 8D, biiezione `DRIVE_DIM` con le dimensioni semantiche

Producono — non per scelta arbitraria ma per necessità — quattro strutture:

- **Lexicon** (cosa esiste)
- **KnowledgeGraph** (come è connesso — con 21 tipi di relazione, 5 categorie, fenomenologiche centrali)
- **PrometeoField** (come è ora)
- **Valence** (come l'entità sente ciò che è)

Quattro strutture coerentemente accoppiate:

- Apprendimento: lessico + KG cambiano lentamente, PF1 li rifletti, rederive-signatures rideriva firme
- Percezione: input → KG identifica attrattori → PF1 seeds + propagazione → Valence si ricalcola
- Espressione: PF1 stato → KG nuclei → Valence coloring → voice → grammatica (con onestà: oggi è KG renderer, non emergenza pura)

Tre debiti tecnici da nominare:

- **Due sistemi di attivazione** (pf_activation vs word_topology): da unificare
- **`compose()` è KG renderer**, non emergenza dal campo: da superare
- **Dead code** (`dual_field.rs`, `llm_substrate*.rs`, due binari): da rimuovere

Un principio architetturale che permea tutto:

- **Resting state sotto-soglia**: il campo è sempre presente in silenzio, attiva solo se perturbato

Da qui in poi, ogni volume zooma su un livello specifico. La prima fermata: **PrometeoField (PF1)**, dove la filosofia incontra i 512 byte fissi per parola e le fasi sugli archi.

---

*Prossimo volume: 02 — Fondamenti: PrometeoField (PF1)* (in scrittura)
