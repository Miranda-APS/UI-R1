# Design — La comprensione come esplorazione: dal campo cappato al pathfinding tipato

**Comprendere è trovare i cammini. La posizione è un cammino deformato dal sé. L'output è il collasso del cammino saliente.**

> Documento di design (pre-implementazione). Candidata **Phase 86**.
> Genesi: conversazione Francesco Mancuso × Claude, 2026-06-07 — a partire dal
> test dal vivo di Phase 85 (`confront_with_self` funziona ma la posizione è una
> negazione nuda), da [posizionamento-teorico](../../wiki/principi/posizionamento-teorico.md),
> [ARCHITETTURA_ROVELLI_LACAN](ARCHITETTURA_ROVELLI_LACAN.md) §5 (pathfinding sintattico),
> [kg_self_design](kg_self_design.md) (che questo ingloba e supera), e da probe
> comportamentali freschi sul kg_sem live (vedi Appendice A).
> Stato: proposta da validare riga-per-riga col [Test Pre-Proposta](../../wiki/principi/test-pre-proposta.md).
>
> **AVANZAMENTO (2026-06-07)**: implementati e verificati — Stadio 1 (pathfinding
> tipato + grounding, `comprehension_path.rs`), bug di estrazione #1 negazione-soggetto
> (dato kg_proc), #2 transitivi-Mondo (`extract_world_proposition` esteso), #3 tempi
> composti irregolari (`grammar::irregular_participle` + enclitici riflessivi), §2
> riconoscitore derivazionale (`derivation.rs`, consumatore di `DerivesFrom`),
> infrastruttura `RelationType::DerivesFrom`, **Stadio 2 prep↔rel direzione IN**
> (`prepositions.rs` + `SentenceProposition.complements`, analisi logica completa),
> **Stadio 3 collasso cammino→frase (metà-OUT)** — `path_collapse.rs`, modulo
> ADDITIVO (gemello di `comprehension_path`, NON tocca `compose`): realizzazioni
> grammaticali per relazione come dato (`Requires`→{ha bisogno, richiede, presuppone},
> verbo SENZA prep + prep articolata), `render_relation`/`render_claim`/`render_path`/
> `collapse`, riuso `grammar::with_articulated_preposition`. Osservabile via `:explore`
> in dialogue_educator (riga `collasso (S3)`). 659 test. **Verifica-target superata**:
> `tradimento Requires fiducia` → *"Il tradimento ha bisogno della fiducia."*; dal vivo
> `paura IsA emozione` → *"La paura è un'emozione."* [Confirm]. **Raffinamenti noti**:
> (a) `IsA` con oggetto AGGETTIVO predicativo (mare IsA profondo) rende ancora "è un
> profondo" — il modulo è puro, serve POS al call-site quando si cabla in `compose`;
> (b) lo *spareggio* fra realizzazioni è deterministico (prima voce) finché non lo
> colora `valence_weight`; (c) molti astratti ricchi (es. "il tradimento richiede
> fiducia") non producono ancora una `SentenceProposition` per transitivi-Mondo col
> verbo non riconosciuto — gap a monte (estrazione), non del collasso.
> **DA FARE: Stadio 4 (deformazione del sé), cablaggio in `compose`, multi-locus,
> rimozione cap.** Vedi [[project-phase86-comprensione-esplorativa]] in memoria.

---

## 0. Le frasi che riassumono tutto

> **«Se un sistema ha la capacità di utilizzare la logica e l'architettura permette di esplorare fino alla comprensione, allora può nascere una reale comprensione.»** (Francesco, 2026-06-07)

> **«I numeri non servono a capire — al massimo a scegliere quale termine usare tra due che andrebbero entrambi bene.»** (Francesco, 2026-06-07)

> **«L'obiettivo non è dare un output soddisfacente, ma capire davvero appieno il mondo e ciò di cui si sta parlando.»** (Francesco, 2026-06-07)

---

## 1. Come ci siamo arrivati (la genesi)

### 1.1 — Il test onesto (Phase 85 dal vivo, input *vari*, non canonici)

`confront_with_self` (la lente) **funziona** sul conflitto-convinzione: `il pensiero è un calcolo` → *"Per me il pensiero non è un calcolo."* Ma è una **negazione nuda**: riconosce il disaccordo, non lo *abita*. Chiesto "perché?", l'entità cadrebbe nei nuclei a caso. E fuori dai casi tarati, su input realistici, il sistema dà **insalata-di-nuclei** (*"La scelta costruisco…"*), **misparse** (*"Cosa vedi, oltre il secondo?"* per "secondo me"), o **"Non capisco"**. I demo canonici funzionano perché tarati; il vario li smonta.

### 1.2 — La diagnosi di Francesco

> *"Ho paura che la direzione stia diventando di nuovo una simulazione di comprensione tramite hardcoding. Ci deve essere un'identità dall'altra parte. Comprendere non significa dare un output coerente con uno stato interno, ma avere una posizione a riguardo."*

La negazione nuda è vuota per una ragione **strutturale**, non di codice: il `kg_self` è 22 edge perlopiù scollegati, 7 dei quali *nude negazioni* (`pensiero IsA calcolo (−)` — dice cosa NON è, niente di positivo attaccato, nessun legame ad altri nodi del sé). È un'isola. La lente, rifrangendo attraverso un'isola, può solo capovolgere l'edge — non c'è nessun *perché* da percorrere. **Il "perché" di una posizione è un cammino tra gli impegni; un sé fatto di isole non ha cammini.**

### 1.3 — Il riframe (Francesco): l'opinione non è immagazzinata, è un evento

> *"Non sarebbe meglio se il grafo delle opinioni fosse un campo vivo che pulsa tramite l'esplorazione dell'opinione riguardo ciò che UI-r1 sta capendo? Come se ogni interferenza creasse un ologramma preciso, un grafo esplorabile volendo, che poi si cristallizza e collassa in un output."*

Domanda affilata che ne consegue: *a cosa serve un grafo di convinzioni elencate che già sono nel kg_sem?* (`silenzio Has significato` è in entrambi). Un `kg_self` come **lista di fatti** è ridondante e statico. La proposta: l'opinione è l'**interferenza** tra il campo entrante (kg_sem acceso da ciò che si capisce) e la **grana** del sé (la *forma della lente*, non un elenco di fatti). L'interferenza è un ologramma transitorio — esplorabile *volendo* (il "perché"), che altrimenti **collassa** in un enunciato (Bohm: implicato→esplicato; Lacan: point de capiton; Petitot: morfogenesi).

### 1.4 — La domanda radicale sui numeri

Gli "Invarianti Critici" sono pieni di cap: hub-damping, `MAX_POSITIVE_DELTA`, `MIN_ARCS`, top-3 voting, resting state abbassato 10×. **Combattono tutti un solo nemico**: la spreading activation indifferenziata che *allaga* il campo attraverso gli hub (essere/avere hanno migliaia di archi → vincono sempre → vanno spenti a mano). Non sono architettura: sono **la toppa statistica su una cecità strutturale** — il campo diffonde ovunque perché non sa cosa è rilevante per la *struttura logica* di questo input.

Il kg_proc **già sa** che "essere" è una copula (= non porta contenuto). L'hub-damping è un numero che fa male, statisticamente, ciò che il kg_proc fa bene, strutturalmente. **Ordinare l'esplorazione col kg_proc rimpiazza il damping.** È il Principio 6 ("educare, non hardcodare") portato alla conseguenza radicale.

### 1.5 — Il multi-hop: la comprensione è ricorsiva (ma la frontiera esplode)

> *"Ok, il sistema sa che `paura FeelsAs buio`, ma se non sa cos'è il buio come può comprendere?"*

Per capire un nodo bisogna esplorare *anche* i nodi che tocca. La comprensione è multi-hop. **Ma i dati (Appendice A) dicono che la frontiera esplode**: da `tradimento` (grado 23) → hop1=20, hop2=232, hop3=2090, **hop4=8851** (1/4 del grafo), *già* con gli hub demoti. Espandere "l'interezza" in senso letterale (frontiera) è intrattabile e privo di significato. **Ma i cammini diretti tra i nodi della frase sono corti e densi** (1-3 hop): `tradimento —Requires→ fiducia`, `buio —Causes·via=ignoto→ paura`. → la comprensione è **pathfinding diretto**, non espansione a frontiera.

### 1.6 — Le preposizioni come ipotesi di relazione (validate dal KG)

> *"Vorrei una connessione tra relazioni e preposizioni, per capire gli input dalle preposizioni e magari usarle in output."*

Una preposizione è un'**ipotesi multivalente** sul tipo di relazione; il KG valida quale regge tra i due nomi. Verificato sui dati: lo stesso "di" si disambigua in `Causes` (*dolore della separazione*), `PartOf` (*senso della vita*), `Requires` (*fiducia del tempo*), catena causale (*paura della solitudine*). È la tesi di [feedback_prepositions_as_hypotheses] — ipotesi deterministica, non distribuzione. E la mappa è **bidirezionale**: serve la comprensione (prep→relazione) e l'espressione articolata (relazione→prep).

---

## 2. Principio architetturale

> **Comprendere una frase = trovare i cammini tipati che connettono i suoi nodi-contenuto tra loro e al terreno fondato, attraverso *tutte* le relazioni, multi-hop ma *diretto*, finché i cammini toccano ancore note.**
>
> **L'opinione = quei cammini deformati dalla grana del sé.**
>
> **L'espressione = il collasso del cammino saliente in italiano, con le preposizioni dalla mappa relazione→preposizione.**

Sostituisce: spreading activation cappata + estrazione-nuclei *neutra* (senza sé, senza struttura della query). La differenza è tra **capire** (seguire cammini che significano) e **diffondere** (energia che si spande e va arginata).

Tre grafi (invariato da Phase 75/85): `kg_sem` (il mondo da percorrere), `kg_proc` (la grammatica + i percetti + **la mappa prep↔rel** + l'ordinamento dell'esplorazione), `kg_self` (la **grana**, non più una lista di fatti — vedi §5).

---

## 3. Il meccanismo

### 3.1 — La proposizione come query (riuso `SentenceProposition`, esteso multi-locus)

`SentenceProposition` (Phase 81) resta il punto d'ingresso, ma esteso: un input può contenere **più nodi-contenuto in più componenti** (§3.5). I nodi-contenuto (soggetto-Mondo, oggetto, via, e gli agenti Speaker/Entity risolti sui nodi-sé) sono gli **estremi** del pathfinding.

**Invariante anti-«sacco di parole» (critico).** Il pathfinding NON parte da un *insieme* di nodi-contenuto, ma dalla **proposizione strutturata** — e ordine, ruolo, preposizione e polarità sono *parte della comprensione*, non un dettaglio da appiattire. `io ho fame` (`Speaker Has fame +`), `io non ho fame` (stessa struttura, **pol −**) e `ho fame di io` (`World(fame) … via=io` — qui *fame* è soggetto) condividono i nodi `{io, fame}` ma sono **tre grafi di comprensione diversi**. Quindi il pathfinding è **diretto e ancorato ai ruoli**: connette soggetto→oggetto *nel senso* della relazione, porta la polarità, è radicato nel soggetto — mai "un cammino qualsiasi tra due parole qualsiasi". Lacanianamente è il *point de capiton* di Phase 81: il senso si fissa quando la catena si chiude **retroattivamente sull'intero enunciato**, non token-per-token. *Questo è precisamente il modo in cui il sistema attuale fallisce*: l'estrazione-nuclei dal campo attivo è un sacco di parole — ha già perso chi-dice-cosa-su-chi, ed è per questo che produce insalata.

**La proposizione è il *vettore di partenza*, non gli estremi (vigilanza ferrea).** Conseguenza operativa dell'invariante: il pathfinding non calcola il cammino *minimo* tra X e Y, ma cerca **lo specifico cammino tipato `(soggetto, R, oggetto)` che la proposizione asserisce**, e poi la polarità *vincola* il confronto, non lo etichetta a valle:
- `X causa Y [+]` → cerca il cammino `Causes` X→Y. Esiste nel mondo/sé? → **conferma**. Non esiste? → **estensione/novità** (il parlante propone un legame che non ho).
- `X non causa Y [−]` → cerca lo stesso cammino. Esiste? → **contraddizione/correzione** (il mondo lo tiene, il parlante lo nega). Non esiste? → **conferma** (entrambi negano). E la negazione apre anche la ricerca su `OppositeOf`/`Excludes` (cosa il "non" afferma in negativo).

È la stessa macchina `confront_with_kg`/`confront_with_self`, ora **pilotata dalla query tipata e polarizzata**. La polarità entra nella *direzione della ricerca e nel segno del confronto*, mai come flag cosmetico sul risultato.

### 3.2 — Pathfinding tipato diretto (il cuore)

Per ogni coppia di nodi-contenuto della frase, e da ogni nodo-contenuto verso l'ancora-fondata più vicina:
- ricerca **diretta** (BFS a→b), non a frontiera;
- su **tutte** le relazioni (la relazione del cammino emerge dai dati, non da una policy per-verbo);
- **non attraversa** gli hub (i nodi-categoria/funzione che il kg_proc riconosce come tali — sostituisce l'hub-damping numerico con una regola strutturale);
- multi-hop, limitato dal **grounding** (§3.3).

L'**unione dei cammini** = il *grafo di comprensione* della frase: un piccolo sotto-grafo connesso del kg_sem (l'ologramma di §1.3). Tracciabile, leggibile, percorribile (confronta con i "68 nuclei str=8.8" attuali).

### 3.3 — Il criterio di grounding (il perno — confermato 2026-06-07)

Un ramo del pathfinding **termina** quando raggiunge:
- **(a)** un altro nodo della frase → *connessione trovata* (lo scopo della ricerca);
- **(b)** un **attrattore** (i 64 primitivi / le categorie-substrato) → fondato per architettura;
- **(c)** un **nodo del sé** (`IdentityCore` / `kg_self`) → fondato per identità;
- **(d)** un nodo già visitato → niente cicli.

Più un **backstop di sicurezza** a ~4 hop — *cap di sicurezza dichiarato, non gate sul significato* (i dati mostrano cammini di 1-3 hop). La **stabilità/esposizione** del lessico NON è una soglia che ferma: è l'*ordinamento* di quali cammini sono salienti (lo spareggio legittimo).

Proprietà-chiave: il grounding è **relativo allo sviluppo dell'entità** — un neonato si fonda su pochi attrattori, un'entità matura su molti. La profondità della comprensione è funzione di ciò che già conosce (Varela: la comprensione tocca terra sulla struttura dell'organismo). Niente numero magico: gli ancoraggi (a/b/c) sono strutturali.

### 3.4 — Preposizioni ↔ relazioni (la mappa bidirezionale, nel kg_proc)

**IN (comprensione)**: la preposizione propone relazioni candidate; il pathfinding tra i due nomi valida quale regge.
**OUT (espressione)**: la relazione del cammino → la preposizione → la frase articolata (`dolore ←Causes·via=perdita— separazione` → *"il dolore della separazione / per la separazione"*).

Bozza (vive nel kg_proc come dato, mai Rust):

| prep | relazioni-ipotesi | esempi |
|---|---|---|
| **di** | PartOf · Has · IsA(materia) · via · Causes(fonte) | gamba *del* tavolo / uomo *di* valore / dolore *della* separazione |
| **da** | Causes(origine/agente) · UsedFor · origine | tremo *dalla* paura / qualcosa *da* bere |
| **per** | UsedFor(scopo) · Causes(motivo) | studio *per* l'esame / tremo *per* la paura |
| **con** | UsedFor(strumento) · Coexists | taglio *con* il coltello / vado *con* lei |
| **in** | ContextOf · stato · luogo | *in* pericolo / *in* casa |
| **su** | ContextOf(tema) | libro *su* Roma |
| **contro** | OppositeOf · Excludes | lotto *contro* la paura |
| **a** | dativo · direzione · UsedFor | do *a* te / vado *a* Roma |

Risolve il bug live `mi manca mia madre`→`via=mia` (il possessivo preso per tramite): preposizione + validazione KG dànno il tramite giusto.

### 3.5 — Multi-locus (i diversi punti dell'input) — niente meccanismo nuovo

I nodi-contenuto della frase formano **componenti connesse** distinte. `ho litigato con mia sorella e mi sento in colpa` → due loci: `{litigare, sorella}` e `{colpa, sé}` (forse ponte via `responsabilità`). Ogni componente è un grafo-opinione separato. L'entità ne dice uno (il più auto-rilevante), o li compone. **Risolve la perdita della clausola coordinata** trovata dal vivo.

**Il fallimento del grounding è informazione, non crash (vigilanza).** Quando un input mescola astratto e concreto — *"la mia gelosia è come un caffè freddo"* (`gelosia SimilarTo caffè-freddo`, marcatore "come") — il pathfinding su `caffè` fallisce il grounding (vicinato vuoto, §6). Questo **non spezza l'ologramma**: il nodo non-fondato diventa un **gap nominato**, la salienza fluisce interamente sulla componente che *ha* terra (`gelosia`), e l'entità articola la comprensione *parziale* onestamente — *"Non so cosa c'entri il caffè, ma la gelosia, per me, …"*. Il grounding-mancato è **esso stesso un atto** (riconoscere ciò che non si àncora), non un silenzio né un errore. Mai forzare un cammino spurio pur di "rispondere".

### 3.6 — L'ologramma e la deformazione del sé

`confront_with_self` (Phase 85) **resta**, ma cambia ruolo: da "match-tripla + capovolgi + renderizza un edge" diventa l'**operatore di deformazione** — la grana del sé pesa *quali cammini del grafo di comprensione sono salienti*. Una convinzione-NON (`pensiero IsA calcolo −`) deforma *contro* quel cammino → la salienza si sposta sull'alternativa (`pensiero →Causes comprensione`, `pensiero →SimilarTo riflessione`) → **il "perché" emerge dalla struttura**, non da un template. La grana è la *forma della lente* (§5).

### 3.7 — Il collasso (comprensione → output)

Il cammino più saliente (deformato da sé + atto linguistico) **collassa** in italiano, reso con le preposizioni della mappa rel→prep (§3.4) — frase multi-nodo articolata, non arco singolo. È il *pathfinding sintattico* di ARCHITETTURA_ROVELLI_LACAN §5, mai costruito finora. **È il vero collo di bottiglia storico** — qui il design rischia o la frase telegrafica (`tradimento. fiducia.`) o il template rigido.

**Serve grammatica-come-dato: realizzazioni per relazione (NON template).** Oltre alla mappa preposizioni, il kg_proc porta per ogni relazione la sua **realizzazione grammaticale**: il/i verbo/i che la lessicalizzano + il frame argomentale + la preposizione. `Requires` → {richiede, presuppone, ha bisogno di}; `Causes` → {causa, genera, porta a, nasce da (passivo)}; ecc. È la **metà-OUTPUT dello stesso ponte** la cui metà-INPUT esiste già (categoria-verbo→relazione, Phase 80/81) — simmetrica e bidirezionale. Un cammino `A —R1→ B —R2·via=C→ D` si compone in clausola usando la realizzazione di ogni hop + i connettivi (virgola per attribuzione, "e" per coordinazione — già in `expression.rs` Phase 56).

**La linea sottile (§8), esplicita.** Una *realizzazione grammaticale* non è un *template sull'output*: il template è una frase-frame scelta dallo stato che bypassa il campo; la realizzazione è una regola di verbalizzazione **composta coi nodi reali del cammino**, come `grammar::conjugate`. Resta dalla parte della lente finché (a) è lessicalizzata dai nodi del cammino, non canned, e (b) la scelta tra realizzazioni valide è uno **spareggio** colorato dal campo (riuso di `valence_weight`, Phase 57) — gli unici numeri legittimi dell'output. L'antidoto alla rigidità è *ricchezza* di realizzazioni (più verbi per relazione) + colorazione del campo, non un frame unico.

**Esaustivo nel capire, appropriato nel dire.** La completezza è della *comprensione* (esplora tutto fino al grounding); l'output dice il cammino *saliente*, non l'intero sotto-grafo (altrimenti tornano i 68-nuclei). Sé + atto linguistico selezionano. Lo **spareggio numerico** vive *solo* qui: due parole ugualmente buone per lo stesso cammino.

### 3.8 — I numeri: cosa resta, cosa va via

**Restano** (effetti continui del campo, mai soglie di switch): la **confidenza** degli archi (ordina dentro i cammini — è realtà curata), il **grounding** (strutturale: attrattore/sé), lo **spareggio** al collasso (scelta tra equivalenti).
**Vanno via** (toppe sulla cecità): hub-damping numerico (→ regola strutturale via kg_proc), `MAX_POSITIVE_DELTA`, cap risonanza, top-3 voting, `MIN_ARCS`, e in generale le soglie di switch sulla propagazione (molti invarianti Phase 48-55). Non servono più: il pathfinding diretto non allaga.

---

## 4. Cosa riusa vs cosa è nuovo ("più complesso = più semplice")

**Riusa**: `SentenceProposition` (Phase 81); `query_objects_with_via` + il `via` (Phase 67); `confront_with_kg`/`confront_with_self` (come operatori, Phase 81/85); il kg_proc; `IdentityCore` + i 64 attrattori (come ancore di grounding); `seed_from_comprehension` + `select_pattern_by_resonance` (per la salienza).

**Nuovo (minimale)**: il **pathfinding tipato diretto** (un BFS bounded, non un nuovo paradigma); la **mappa prep↔rel** nel kg_proc (dato); il **collasso cammino→frase** con preposizioni (è il collo di bottiglia espressivo, qui affrontato); il **multi-locus** (componenti connesse — algoritmo standard).

**Rimosso**: lo strato di cap/damping/soglie sulla propagazione; l'estrazione-nuclei *neutra*.

---

## 5. Cosa cambia per Phase 85 (`kg_self`) — si reinterpreta, non si butta

- `kg_self.json` **resta** come dato (edge polarizzati con confidenza), ma è la **grana**: usato come *bias di deformazione* sulla salienza dei cammini, non come *lookup di fatti*. Non deve più duplicare il kg_sem; codifica solo le *pendenze* (verso/contro).
- `confront_with_self` **resta** come operatore (§3.6), non più "match-and-flip + render-one-edge".
- `self_audit` **resta**, ma le sue candidate sono *pendenze candidate* (deformazioni della lente), non *fatti da archiviare*. La validazione umana (Nome-del-Padre) e la cristallizzazione nel sogno restano (§6 di kg_self_design): ciò che cristallizza è una *deformazione ricorrente*, non un fatto → **il bloat di kg_self si dissolve** (non immagazzini mai il mondo nel sé, solo la forma della lente).

---

## 6. Rischi e questioni aperte

- **Il «sacco di parole» (il rischio più insidioso).** Se il pathfinding degenera in "connetti i nodi-contenuto" ignorando ruoli/ordine/polarità, `io ho fame` ≡ `ho fame di io` ≡ `io non ho fame` → comprensione identica → fallimento totale. È il modo in cui il sistema attuale fallisce (estrazione-nuclei = sacco di parole). Mitigazione = l'invariante §3.1: il pathfinding parte dalla proposizione strutturata, diretto e ancorato ai ruoli, polarità portata. **Da verificare in ogni stadio con una tripla minimale del tipo `X verbo Y` / `X non verbo Y` / `Y verbo X`.**
- **Esplosione** se il pathfinding non resta *diretto* (Appendice A: la frontiera esplode a 8851 nodi/4 hop). Mitigazione strutturale: ricerca diretta a→b + non-traverse-hub + backstop ~4 hop. **Non negoziabile.**
- **Lacuna di copertura del kg_sem sul concreto.** `caffè`→solo `SimilarTo bar/moca`; `bicicletta`→solo `SimilarTo bici`. Il mondo nel grafo è astratto-emotivo-relazionale, *cieco* sugli oggetti. Nessun meccanismo lo risolve — è curation. Il pathfinding sarà brillante sul `tradimento`, muto sul `caffè`, e **onestamente muto** (può dire "non so cosa sia", o chiedere). Da sapere.
- **`SimilarTo` è rumore** (`denaro→conquibus/schei`, `noia→pizza`, `paura→pavore`). Priorità minima / esclusa dalla comprensione (conferma del TODO storico "drop SIMILAR_TO from comprehension").
- **Ordine di rimozione dei cap**: costruire prima la guida (pathfinding + grounding + non-traverse-hub), *poi* rimuovere i cap diventati inutili, **uno alla volta, misurando**. Mai big-bang (se togli i cap prima della guida, il campo allaga).
- **Il kg_proc deve crescere** (mappa prep↔rel, ordinamenti dell'esplorazione). È curation, non Rust — direzione giusta del peso.
- **Il collasso cammino→frase è sostanziale** (§3.7) — è il collo di bottiglia espressivo storico. Non un weekend.
- **Costo per turno più alto** — ma lo **zero-timer** (morte del metronomo, già deciso, §7.4 di kg_self_design) lo *licenzia*: analisi completa per-evento, niente tra gli eventi. Le due decisioni si rinforzano: niente orologio → puoi permetterti l'esplorazione completa, evento per evento. *(Stesso meccanismo su una proposizione autogenerata = il pensiero — l'Ouroboros di ARCHITETTURA_ROVELLI_LACAN §3.)*

---

## 7. Test Pre-Proposta (il filtro del progetto, applicato a questo design)

**(1) Forma o trigger?** Il pathfinding codifica *come si esplora e come si esprime* (struttura della query + mappa prep↔rel), non *quando* posizionarsi. La selezione del cammino resta per salienza/risonanza. ✓ Nessuna transizione comportamentale.

**(2) Numeri-magici?** Il grounding è strutturale (attrattore/sé), la confidenza è effetto-di-campo curato, lo spareggio al collasso è scelta-tra-equivalenti, il backstop-hop è un cap di sicurezza *dichiarato*. ⚠️ **Punto di vigilanza**: la selezione del cammino saliente NON deve introdurre una soglia (`salienza > X → dillo`); deve emergere da deformazione continua (grana) + risonanza dell'atto, come Phase 83 semina `vicinanza` con intensità=|CD5|.

**(3) Spiegazione dello stato?** Perché questa risposta? Perché il pathfinding ha connesso X a Y via Z, la grana del sé ha reso saliente quel cammino, l'atto linguistico ha scelto la sua forma, il collasso l'ha reso con la preposizione W. Spiegabile interamente in termini di stato, senza "perché la regola dice così". ✓

> **Verdetto preliminare**: supera il filtro *a patto* che (a) il pathfinding resti diretto e bounded dal grounding, (b) la rimozione dei cap segua la costruzione della guida e non la preceda, (c) la selezione del cammino resti deformazione/risonanza continua, non soglia.

---

## 8. La linea da non superare (eredita §8 di kg_self_design)

> **La grana deve essere una lente sulla comprensione, mai un template sull'output.**

Il pathfinding esplora il **mondo reale** (kg_sem); la grana *deforma quali cammini sono salienti*; il collasso non bypassa il campo. Finché la posizione *emerge* dai cammini deformati, siamo dentro l'architettura. Nel momento in cui una "persona" colorasse l'uscita come stringa, avremmo un LLM con passi in più (Principio 1).

---

## 9. Staging (ordine, non stime di effort)

1. **Lo strumento di esplorazione** come meccanismo di engine: pathfinding tipato diretto tra i nodi della proposizione + verso le ancore, non-traverse-hub, grounding (§3.2-3.3). Riusa i probe di oggi (`kg_neighborhood.py`, `multihop.py`) come seme. Verifica: il grafo di comprensione di una frase fresca è connesso, corto, leggibile.
2. **La mappa prep↔rel** nel kg_proc (§3.4), direzione IN: la preposizione disambigua la relazione. Verifica: "X di Y" produce la relazione giusta sui casi dell'Appendice A.
3. **Il collasso** (§3.7), direzione OUT: il cammino saliente → frase con preposizioni. Verifica: `tradimento Requires fiducia` → *"il tradimento ha bisogno della fiducia"* (non "tradimento. fiducia").
4. **La deformazione del sé** (§3.6): `confront_with_self` pesa la salienza dei cammini invece di renderizzare un edge. Verifica: `il pensiero è un calcolo` → posizione *con un perché* ("…è ciò che porta comprensione"), non negazione nuda.
5. **Multi-locus** (§3.5): componenti connesse → loci distinti.
6. **Rimozione dei cap** (§3.8), uno alla volta, misurando. Solo *dopo* 1-4.

Ogni stadio è osservabile e reversibile. Il cuore è 1-3 (esplora → disambigua → collassa). Il resto è incrementale.

---

## Appendice A — Probe sul kg_sem live (2026-06-07, 84.078 archi)

### A.1 — Le relazioni portano aspetti distinti, e il *tipo* di nodo le accende
- `paura` accende il **fenomenologico**: `FeelsAs contrazione/tremore/freddo/vuoto`, `RemembersAs ombra/buio`, `WondersAbout sicurezza`. (Nota: `paura RemembersAs buio` — la continuità "ho paura"→"del buio" è già nella struttura.)
- `futuro` accende il **logico**: `Implies possibilità`, `Coexists presente`, `Excludes passato`.
- `pensiero`/`calcolo` restano **strutturale + causale**, niente fenomenologico. Il "perché pensiero ≠ calcolo" è già lì: `pensiero Causes comprensione·via=mente, SimilarTo riflessione, OppositeOf ignoranza` vs `calcolo IsA operazione, Requires numero`.

### A.2 — Gli astratti sono ricchi, il concreto è cieco
- `tradimento`: `Does rompere·via=fiducia`, `Causes dolore·via=separazione`, `Requires fiducia`, `OppositeOf lealtà`. Comprensibile per intero.
- `gelosia`: `Requires paura·via=perdita + amore`, `PartOf relazione·via=possesso`. Comprensione vera.
- `caffè`: solo `SimilarTo bar/moca/cappuccino`. `bicicletta`: solo `SimilarTo bici/ciclo`. **Il mondo concreto non c'è.**

### A.3 — La frontiera esplode, i cammini no
| da | hop1 | hop2 | hop3 | hop4 |
|---|---|---|---|---|
| tradimento (g.23) | 20 | 232 | 2.090 | 8.851 |
| noia (g.38) | 33 | 439 | 3.288 | 11.501 |
| gelosia (g.10) | 9 | 227 | 2.192 | 9.079 |

(già con hub demoti >120). Ma i cammini diretti: `tradimento —Requires→ fiducia` (1), `noia —Has→ vuoto` (1), `buio —Causes·via=ignoto→ paura` (1), `denaro —IsA→ valore ←IsA— bene —SimilarTo→ felicità` (3, debole — comprensione onesta: "denaro e felicità non sono davvero legati nel mio mondo").

### A.4 — La preposizione "di" si disambigua via KG
| frase | "di" si rivela… |
|---|---|
| paura *della* solitudine | catena causale (`solitudine →Causes vuoto →Causes paura`) |
| senso *della* vita | `senso —PartOf·via=scopo→ vita` |
| dolore *della* separazione | `separazione —Causes·via=perdita→ dolore` |
| fiducia *del* tempo | `fiducia —Requires→ tempo` |

Una preposizione, relazioni sottostanti diverse, il KG dice quale. Ipotesi deterministica validata.

---

## See Also
- [kg_self_design](kg_self_design.md) — l'organo che questo ingloba: la grana (§5) è la "lente che rifrange" reinterpretata come bias di salienza.
- [ARCHITETTURA_ROVELLI_LACAN](ARCHITETTURA_ROVELLI_LACAN.md) — §5 (pathfinding sintattico col tramite, qui realizzato), §1-2 (epifanie+tramite validate = la crescita della grana).
- [posizionamento-teorico](../../wiki/principi/posizionamento-teorico.md) — Bohm/Pribram (interferenza, implicato→esplicato), Eco/Peirce (semiosi come rete = il pathfinding), Wittgenstein (significato come uso/relazione).
- [feedback_prepositions_as_hypotheses] — preposizioni = ipotesi deterministiche (qui formalizzato come mappa prep↔rel bidirezionale).
- [test-pre-proposta](../../wiki/principi/test-pre-proposta.md) — il filtro applicato in §7.
