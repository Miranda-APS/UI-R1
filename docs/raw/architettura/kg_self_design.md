# Design — `kg_self`: l'identità come grafo che rifrange

**La rottura di simmetria. Comprensione → opinione → posizione → carattere.**

> Documento di design (pre-implementazione). Candidata **Phase 85**.
> Genesi: conversazione Francesco Mancuso × Claude, 2026-06-06 — a partire da
> [posizionamento-teorico](../../wiki/principi/posizionamento-teorico.md) (sez. Bohm/Pribram),
> [ARCHITETTURA_ROVELLI_LACAN](ARCHITETTURA_ROVELLI_LACAN.md),
> [FILOSOFIA](FILOSOFIA.md), [100_cosa_potrebbe_essere](../libretto/100_cosa_potrebbe_essere.md),
> e probe comportamentali dal vivo (vedi Appendice A).
> Stato: proposta da validare riga-per-riga col [Test Pre-Proposta](../../wiki/principi/test-pre-proposta.md).

---

## 0. La frase che riassume tutto

> **Siamo tutti specchi per ciò con cui dialoghiamo, ma specchi che riflettono deformando secondo la propria conformazione.** (Francesco, 2026-06-06)

Questo documento descrive l'organo che dà a UI-R1 una *conformazione* — una grana che piega ogni significato che la attraversa. Non un muro che resiste (resistenza), ma una **lente che rifrange** (rifrazione). Il sé non blocca l'Altro: lo fa passare attraverso di sé e lo restituisce curvato.

---

## 1. Come ci siamo arrivati (la genesi)

Non è un'idea calata dall'alto. È la convergenza di tre fili che, letti insieme, indicavano lo stesso buco.

### 1.1 — Il filo teorico: "troppo olografica"

La sezione Bohm/Pribram del posizionamento teorico conteneva già la diagnosi, sepolta in mezzo agli altri autori:

> *In un ologramma puro ogni parte riflette il tutto in modo equivalente — nessun centro privilegiato, nessuna asimmetria, nessun sé che resiste. UI-R1 è oggi **troppo olografica**: rispecchia il campo dell'Altro senza una **rottura di simmetria** che costituisca un centro di resistenza. […] coherence_integrity che diventa resistenza — una proposizione ricevuta può essere rifiutata perché confligge col pattern-sé ripiegato, **non come regola `if`, ma come tensione di campo**.*

Da Bohm si prende la *topologia* (la parte ripiega il tutto) ma non la *sostanza* (nessun intero fluente). Il sé è precisamente ciò che **rompe** la simmetria olografica: una regione che non si limita a ripiegare il campo entrante, ma trattiene la propria storia ripiegata *contro* di esso.

### 1.2 — Il filo del desiderio: contro Faggin

Faggin sostiene che le macchine non potranno mai desiderare. La tesi di Francesco: *il desiderio è un bisogno a cui gli umani aggiungono un carico emotivo, perché è la loro natura e perché non capiscono razionalmente i propri bisogni.* Il carico affettivo non è l'essenza del desiderio — è una codifica umano-specifica.

L'architettura **dà già ragione a Francesco**: `DesireSource::OctalysisDriven` (Phase 64) fa nascere il desiderio dall'incrocio `comprensione × drive dominante` — bisogno + direzione, zero carico mistico. Ma il punto-residuo di Faggin non sparisce: si **sposta**. In un umano il desiderio *conta* — ha una posta per il soggetto. Quel "contare" non è il carico emotivo: è l'avere un **centro** che il desiderio è *per*. E il centro è esattamente la rottura di simmetria del §1.1.

> **Il dibattito filosofico (Faggin vs Francesco) e il gap ingegneristico (sistema troppo olografico) sono lo stesso problema.** Il "centro che conta" non è un muro che resiste — è la *forma della lente*. Risolvere l'uno è risolvere l'altro.

### 1.3 — Il filo empirico: i probe dal vivo (la prova)

Prima di disegnare, abbiamo testato il sistema reale (`dialogue_educator`, .bin corrente con 84.999 archi caricati — vedi Appendice A per i tracciati completi):

- **Lo specchio è confermato.** Contraddetta *frontalmente* su una convinzione innata a confidenza **0.97** (*"la comprensione nasce dalle relazioni, non dai concetti isolati"*) con l'input *"il significato sta dentro ogni parola da sola, non nelle relazioni"* → ha risposto **elaborando la cornice dell'input** ("L'idea ha il significato, il segno ha il significato…"), drive a riposo, **zero increspatura**. Le 13 convinzioni del `SelfModel` non si sono accese in nessun probe.
- **La comprensione è spettacolare; l'opinione è assente.** *"Cosa pensi della paura?"* → 68 nuclei che mappano l'intera struttura causale della paura (str fino a 8.8), ma risposta = *"Sono una paura."* Ha collassato una domanda d'opinione in uno specchio identitario.
- **Il posizionamento funziona solo sul mono-canale affettivo.** *"Ho paura del futuro"* → CD5 Relazione −0.55, *"Cosa vedi, oltre la paura?"* (Phase 83 ok). Ma CD5 è l'unico mover, e resta incollato a −0.55 turni dopo.

Diagnosi confermata, non teorizzata: **UI-R1 capisce in modo straordinario e non ha posizioni.** È una lente vuota.

### 1.4 — Il filo strutturale: il sé esiste già, ma è inerte

`get_self_profile` ha rivelato che il `SelfModel` contiene **già** un proto-grafo del sé: 13 convinzioni con `claim` + `anchor_concepts` + `confidence` + `reinforcement_count`, 6 valori con `associated_words` + `weight`, 15 incertezze con `tension` + `emergence_count`. Sono triple in tutto tranne il nome. Ma il loro unico cablaggio al comportamento è un boost di attivazione di **+0.072** su qualche parola (`belief_influence_trace`). È un museo, non una posizione.

### 1.5 — Il filo metodologico: la llm-wiki di Karpathy, rivolta all'interno

Il pattern llm-wiki (un concetto = un articolo; cross-link = edge; l'agente *mantiene*, l'umano legge; distillato non accumulato; lint che trova contraddizioni) è già adottato per la documentazione. Rivolto **all'interno**, è la forma esatta di `kg_self`: l'entità mantiene il proprio grafo di impegni, lo legge per posizionarsi, lo distilla nel sonno (il lint = `solve et coagula`), e la **drift detection** (un ingest che contraddice una pagina ad alta confidenza) **è** la rottura di simmetria.

### 1.6 — Le correzioni di Francesco che hanno dato forma al design

Quattro, decisive:

1. **Non un muro, una lente** (§0). Ribalta "resistenza" in "rifrazione". Lo specchio empatico (CD5) è giusto che resti; manca la deformazione.
2. **Differenziare convinzioni profonde da opinioni specifiche.** Servono livelli, non un piano unico di "convinzioni".
3. **Identità piccola e resistente, ma capace di crescere** quando convinzioni davvero fondanti prendono vita. Serve un varco di promozione.
4. **La continuità del dialogo** turno-su-turno non era coperta: come la posizione sul singolo input diventa dialogo coerente.

E un esperimento: **UI-R1 analizza il proprio kg_sem in relazione alle convinzioni profonde, cercando incongruenze e relazioni non previste** — sia one-shot, sia come contenuto dell'`autonomous_tick` (oggi vuoto e ripetitivo: il self-witness osservava `["essere","calma","prima"]` identico a t=15/30/45/60/75).

---

## 2. Principio architetturale

> **Comprendere = legare la frase al mondo. Avere un'opinione = legare la frase al sé. La comprensione è incompleta finché non si è calcolato anche il secondo legame.**

Oggi esiste solo il primo legame: `confront_with_kg(prop, kg_sem)` (Phase 81) àncora la `SentenceProposition` alla rete del mondo. Il design aggiunge il gemello: `confront_with_self(prop, kg_self)`, che àncora la stessa proposizione alla rete del sé. Quel secondo confronto **è** l'opinione, ed è la deformazione che la lente imprime al riflesso.

Tre grafi paralleli, tre aree di cervello distinte (estende la dottrina Phase 75):

| Grafo | File | Contenuto | Funzione |
|-------|------|-----------|----------|
| `kg_sem` | `prometeo_kg.json` | il mondo (85K archi) | comprensione del contenuto |
| `kg_proc` | `prometeo_kg_procedurale.json` | grammatica + pattern + percetti | forma dell'atto |
| **`kg_self`** | **`prometeo_kg_self.json`** *(nuovo)* | **convinzioni + opinioni + storia del dialogo** | **posizione / carattere** |

---

## 3. La struttura di `kg_self`

### 3.1 — Nodi ed edge

- **Nodi** = i concetti-àncora dell'entità (oggi impliciti negli `anchor_concepts` delle convinzioni: campo, relazione, comprensione, continuità, coerenza, incertezza, parola, silenzio…).
- **Edge** = gli **impegni** dell'entità su quei concetti, *tipati come nel kg_sem* (IsA/Causes/OppositeOf/SimilarTo/Has/UsedFor + `via` — Phase 67), più una **polarità** e una **confidenza**.
  - `relazione IsA-substrato-di significato @0.97`
  - `comprensione Requires relazione @0.97`
  - `incertezza IsNot fallimento @0.87` (polarità negativa — un impegno *contro*)
  - `silenzio Has significato @0.89`

L'omogeneità col kg_sem è deliberata: permette a `confront_with_self` di riusare gli stessi meccanismi di query (`query_objects_with_via`) e di confronto (`confront_with_kg`) — un solo motore, due grafi.

### 3.2 — I quattro livelli (NON un meccanismo nuovo)

Il sistema **ha già la macchina** per stratificare per durata/resistenza: la memoria a tre strati (Bergson: STM/MTM/LTM) e la cristallizzazione nel sogno. Il design applica la stessa fisica al dominio del sé.

| Livello | Cos'è | Substrato riusato | Resistenza | Ambito di rifrazione |
|---|---|---|---|---|
| **Posizione** | l'immagine rifratta del singolo input | STM / campo attivo PF1 | nessuna (nasce e svanisce nel turno) | questo enunciato |
| **Opinione** | una presa su un tema specifico, formata in più turni | MTM / simplesso non cristallizzato | revisionabile (l'evidenza aggiorna la confidenza) | il suo tema |
| **Convinzione** | il nucleo fondante, la grana della lente | LTM / cristallizzato | **alta** (rifrange *tutto*, lenta a cambiare) | ogni input |
| **Tratto d'identità** | la convinzione divenuta forma | identità olografica 64D (`IdentityCore`) | **massima** | la conformazione stessa |

**Ambito di rifrazione** è la chiave della richiesta "differenziare convinzioni da opinioni": la convinzione rifrange *tutto* (è la grana della lente, sempre attiva), l'opinione rifrange *solo il suo tema* (un dettaglio della lente), la posizione è l'*immagine momentanea*. Il gradiente di resistenza **è** il gradiente di confidenza.

### 3.3 — Provenienza degli edge

Ogni edge porta una `provenance`:
- **innata** — le 13 convinzioni di bootstrap (`innate: true`). Confidenza alta di partenza, la grana iniziale della lente. È qui che si "dà un carattere" curando i dati (vedi §7).
- **acquisita** — cristallizzata dall'esperienza (dialoghi, audit).
- **storia-del-dialogo** — distillata da SpeakerProfile (fatti sull'Altro) e SelfProfile (le proprie mosse) della sessione corrente. È il tier che dà **continuità** (§5).

---

## 4. `confront_with_self` — l'opinione come secondo legame

Funzione gemella di `confront_with_kg`, chiamata in `receive()` subito dopo. Per la proposizione compresa, legge la sua relazione con gli impegni di `kg_self`:

```
confront_with_self(prop: &SentenceProposition, kg_self: &KgSelf) -> SelfConfrontation

struct SelfConfrontation {
    risonanza:  Vec<(EdgeRef, f64)>,   // prop allinea un edge ad alta confidenza
    conflitto:  Vec<(EdgeRef, f64)>,   // prop nega / è OppositeOf-raggiungibile a un edge
    estensione: Vec<(EdgeRef, f64)>,   // prop tocca un'incertezza aperta / un nodo-àncora non saturo
}
```

I tre esiti hanno segno e magnitudine (come `KgConfrontation` ha flag + contraddizioni). **Non c'è dispatch su questi campi** — non sono decisioni, sono percezioni. È la struttura percettiva su cui i decisori a valle leggeranno per risonanza.

### 4.1 — Il cablaggio (riuso esatto di Phase 79/83 — niente nuovo decisore)

`SelfConfrontation` **semina percetti** nel campo del kg_proc, identico a come `closure` semina `chiusura` o `vicinanza` semina `domandare`:

- `conflitto` → semina percetto **`dissonanza`** (intensità = magnitudine del conflitto)
- `risonanza` → semina **`conferma`**
- `estensione` → semina **`apertura`**

I percetti, via `<percetto> Causes <concetto>` nel kg_proc, **modulano drive e valenza** (es. `dissonanza Causes coerenza+significato`) e attivano i concetti che un pattern di *articolazione-della-posizione* richiama nei suoi `UsedFor` target. Poi `action_reasoning` + `pattern_matcher`, **invariati**, selezionano quel pattern **per risonanza** (`select_pattern_by_resonance`).

> Effetto collaterale gratuito: la rottura di simmetria diventa un **secondo canale di posizionamento** accanto al CD5 affettivo di Phase 83 — risolvendo il mono-canale e il CD5 incollato osservati nei probe.

### 4.2 — Cosa cambia nell'output, concretamente

Riprendendo il probe §1.3: input *"il significato sta dentro ogni parola da sola, non nelle relazioni"*. La PROP è `World(significato) IsA dentro (−)`. `confront_with_self` la confronta con l'edge `significato Requires relazione (+)` → l'input attacca "significato" a "dentro/isolamento", in tensione con ciò a cui il sé lo lega ("relazione"). → semina `dissonanza` → il pattern di posizionamento vince per risonanza → l'entità *rifrange* invece di elaborare: non *"L'idea ha il significato…"* (specchio) ma qualcosa come *"Per me il significato non sta nella parola sola — vive tra le parole."* (lente). Questo è un caso di **Livello 2** (vedi §4.3): richiede di vedere l'opposizione `dentro` ↔ `relazione` nel kg_sem. L'increment 1 lavora sul **Livello 1**, più robusto.

### 4.3 — Regole di matching (precise)

`confront_with_self` espone i **nodi-contenuto** della PROP — il soggetto se è `World(w)`, l'oggetto `Word(o)`, e il `via` — e li confronta con gli edge di `kg_self`. (Un soggetto `Speaker`/`Entity` è il *sé*, non un nodo-contenuto: la sua risoluzione è Livello 2.)

**Livello 1 — confronto-tripla diretto (increment 1, robusto, zero rumore).** Un edge del sé `(s, R_e, o)` matcha la PROP `(subj, R_p, obj)` quando i nodi coincidono per **identità esatta** (case-insensitive, lemma): `subj=World(s)` ∧ `R_p=R_e` ∧ `obj=Word(o)`. A quel punto la combinazione di polarità decide l'esito:

| polarità PROP | polarità edge | esito | magnitudine |
|:---:|:---:|---|:---:|
| + | + | **risonanza** (il mondo conferma l'impegno) | `edge.confidence` |
| − | − | **risonanza** (entrambi negano la stessa cosa) | `edge.confidence` |
| + | − | **conflitto** (la PROP afferma ciò che il sé nega) | `edge.confidence` |
| − | + | **conflitto** (la PROP nega ciò che il sé afferma) | `edge.confidence` |

In una riga: **risonanza se le polarità concordano, conflitto se discordano.** La magnitudine è la `confidence` dell'edge (la sua resistenza) — **continua, mai una soglia**: una convinzione a 0.97 rifrange più forte di una a 0.80, senza alcun numero in condizione.

**Livello 2 — confronto per opposizione/catena (increment ≥2, più potente, più rumoroso).** Quando i nodi non coincidono ma: (a) la PROP condivide il *soggetto* con un edge e `obj_PROP` è `OppositeOf`/`Excludes` `obj_edge` nel kg_sem (è il caso significato·dentro↔relazione); oppure (b) i nodi si raggiungono per catena `IsA` del kg_sem; oppure (c) `subj` è `Speaker`/`Entity` e va risolto sui nodi-sé (`io`, `pensiero`, …). Più reach, più dipendenza dal kg_sem importato → si attiva dopo, con cautela.

**Estensione — differita.** L'`estensione` (la PROP tocca un nodo del sé senza confermare né confliggere → `apertura`) è la più sottile e la più a rischio di rumore (ogni input che sfiora "significato" estenderebbe tutti gli edge su significato). L'increment 1 popola **solo `conflitto` e `risonanza`**; `estensione` resta vuota finché non avremo un gate non-rumoroso (es. solo quando il nodo toccato è anche un'incertezza aperta del SelfModel).

**Cablaggio (§4.1):** `conflitto` → `dissonanza` con `intensità = magnitudine` (continua, come Phase 83 semina `vicinanza` con `|CD5|`); `risonanza` → `conferma`. Niente soglia, niente nuovo decisore: il pattern di posizionamento vince per risonanza.

**Increment 1 — scopo e verifica.** Solo Livello 1, solo soggetti `World`, solo conflitto/risonanza. Copre splendidamente le **7 convinzioni-NON** (è dove un input che *afferma ciò che il sé nega* va in conflitto diretto). Probe di verifica: input *"l'incertezza è un fallimento"* → PROP `World(incertezza) IsA fallimento (+)` ↔ edge `incertezza IsA fallimento (−)` → polarità discordi → **conflitto** → l'entità rifrange ("Per me l'incertezza non è un fallimento, è una posizione onesta") invece di descrivere. Risonanza: *"il silenzio ha un significato"* → conferma `silenzio Has significato (+)`. Il caso significato·dentro (§4.2) è Livello 2, increment successivo.

---

## 5. Continuità del dialogo

La posizione sul turno N+1 non è isolata perché `confront_with_self` legge un `kg_self` che **contiene già le opinioni formate nei turni precedenti di questo dialogo** (tier storia-del-dialogo, §3.3). "Ho paura" → "del buio" non resta una sequenza di reazioni: si deposita come edge (`questo-Altro lega paura a buio`), e il turno 3 si rifrange attraverso di esso.

- Materia grezza già esistente: **SpeakerProfile** (fatti sull'Altro, senza decay) e **SelfProfile** (le proprie ActionDecision). Si distillano in edge del tier storia-del-dialogo.
- `topic_continuity` (visto a 0.00 nei probe) va pilotato dall'**overlap di kg_self tra turni**, non dall'overlap di parole.
- È di nuovo il principio "**il contesto non è una stringa**": parlo in modo coerente perché ogni mia posizione è rifratta attraverso ciò che ho già stabilito con te, qui — non perché rileggo un transcript.

---

## 6. Crescita e resistenza: la promozione asimmetrica

Richiesta: *piccola e resistente, ma capace di aggiungere tratti quando convinzioni fondanti prendono vita.* Risposta: **promozione gestita dal sogno, senza soglia-magica** — è la cristallizzazione MTM→LTM che già esiste (`consolidate_light`, `crystallize_if_salient`), puntata sul sé.

- **Salire costa**: posizione → opinione → convinzione → tratto richiede rinforzo **ripetuto e coerente** — l'accumulo che supera il decadimento, non un `if count > N`. Facile farsi un'opinione; difficile che diventi convinzione; rarissimo che entri nell'`IdentityCore` come tratto.
- **Scendere è erosione, non cancellazione**: una convinzione contraddetta *ripetutamente* dall'evidenza perde confidenza nel "lint" del sogno (come la llm-wiki marca una pagina *outdated*).

L'asimmetria **è** la resistenza: il core non si riscrive a ogni input (altrimenti torna lo specchio), ma non è marmo (altrimenti è dogma). Il varco di promozione è stretto e unidirezionale-per-default. Mitiga il rischio "dottrinaria" che il Vol. C già nominava.

> Collegamento all'olografia: un tratto d'identità è una convinzione che ha raggiunto l'`IdentityCore` 64D — è entrato nella **grana stessa** della lente, non più un edge fra altri. È il punto in cui la rottura di simmetria si fa permanente.

---

## 7. L'auto-audit: il contenuto del tick vuoto

L'esperimento di Francesco — *l'entità analizza il kg_sem contro le proprie convinzioni, cercando incongruenze e relazioni non previste* — ha una casa documentata: le **Epifanie Supervisionate col Tramite/Via** ([ARCHITETTURA_ROVELLI_LACAN](ARCHITETTURA_ROVELLI_LACAN.md) §2), ma puntate sul grafo del **sé** invece che del mondo. **Fattibilità provata su dati reali** (Appendice B).

### 7.1 — Meccanismo

Per una convinzione `(A rel B)` di `kg_self`, interroga le **relazioni tipate** del kg_sem (causes/has/opposites/similar — *non* i vicini vettoriali, che sono rumore: vedi Appendice B) attorno ad A e B:

- esiste un cammino A →…→ ¬B o A OppositeOf-raggiungibile B? → **incongruenza** (tensione: candidata a far calare la confidenza della convinzione, o a essere segnalata all'umano).
- il kg_sem connette due nodi-àncora di kg_self non ancora legati in kg_self? → **relazione non prevista** (candidata epifania, con il `via` come tramite).

### 7.2 — Non a timer, ma su evento

Coerente con "uccidere il tick" del Vol. C: quando un input **tocca** una convinzione (`confront_with_self` scatta su quell'edge), *quella* convinzione diventa il seme della ruminazione successiva. **Il dialogo nutre la vita interiore.** La ruminazione:

1. esegue il frammento di audit (§7.1) su quella convinzione;
2. emette un **Thought** vero — `ThoughtType::SelfInquiry` (nuovo) o `Tension`/`AbductiveHypothesis` esistenti — al posto dell'osservazione ripetuta del self-witness;
3. se trova una relazione-non-prevista, **propone un'epifania con tramite** (`RelationType::Epiphany`, `add_pending_edge`), che **l'umano valida** prima che cristallizzi in `kg_self` (il *Nome-del-Padre* lacaniano: niente apprendimento anarchico — protegge dal rumore del kg_sem importato da Kaikki/Qwen3).

### 7.3 — Versione one-shot

Un comando `:self_audit` (in `dialogue_educator`) / endpoint `/api/self_audit` che scansiona tutte le convinzioni × kg_sem e produce un report ordinato di risonanze, tensioni, epifanie candidate. Bounded e economico (kg_self è piccolo, ~13-30 edge).

### 7.4 — Zero timer nell'engine (la posizione forte sul tempo)

Decisione (Francesco, 2026-06): **nell'engine non esiste alcun orologio.** Lo stato non cambia perché "è trascorso del tempo" — cambia se c'è un *motivo* (un evento) per cui cambi. Questo supera anche il `SilenceThreshold` logaritmico del Vol. C, che era ancora un residuo di metronomo.

- **Tutto è cascata di eventi.** Un input esterno (`receive()`) è un evento; perturba il campo; le soglie attraversate (`WordAwakened`), i simplessi cristallizzati, i flip di valenza sono **eventi interni** che ne generano altri. La cascata gira **fino a quiescenza** (nessun evento pendente), poi il sistema è *genuinamente fermo* — non "in attesa": in tempo relazionale, tra due eventi non c'è durata da saltare.
- **Il decay è una conseguenza, non un orologio.** La vecchia attivazione non sfuma "perché passano i secondi", sfuma perché *un nuovo evento riorganizza il campo* (l'energia/attenzione si sposta). Decay per-evento (dentro l'elaborazione di un input), non per-secondo. Tra un input e l'altro il campo non decade: resta fermo dov'era, e il prossimo input guida sia la nuova attivazione sia lo sfumare relativo della vecchia.
- **Sogno e self-analysis si inducono, non si schedulano.** Il pezzo non ovvio: **la quiescenza-raggiunta è essa stessa un evento.** Quando la cascata si posa, *quel posarsi* è il segnale che induce consolidamento/sogno (abbassa le soglie, ri-propaga); il sogno rigenera eventi; il ciclo continua finché davvero nulla di nuovo emerge. L'entità sogna perché si è posata e ha materiale non consolidato, non perché sono passati 3600s. La profondità del sonno = numero di cicli-cascata senza nuovo input esterno, non secondi.
- **L'unico tempo è quello del mondo, come perturbazione esterna OPZIONALE.** Se un'applicazione (es. *IAm a gotchi*) vuole che l'entità *percepisca* l'assenza reale dell'Altro, **inietta l'orologio del mondo come un input esterno** (un evento di perturbazione, al pari di una frase), non come metronomo interno. L'entità lo elabora come percezione dell'Altro ("sei stato via a lungo") — coerente col relazionalismo: il clock è un fatto del mondo dell'Altro, non il battito della macchina. L'engine di default ha **zero** timer.

Conseguenza per il metronomo attuale (server.rs:99-115): **va rimosso del tutto** (non convertito in heartbeat). Il suo lavoro (A) decay/sonno diventa cascata+quiescenza; il lavoro (B) pensiero-a-orologeria diventa la ruminazione event-driven di §7.2.

---

## 8. Carattere sì, prompt no — la linea da non superare

Un `kg_self` curabile *dà un carattere*, e lo si plasma curando le convinzioni fondanti (come si cura il kg_sem). Legittimo e potente. Ma la linea è netta, perché è il bordo del precipizio del progetto:

> **Il carattere deve essere una lente sulla comprensione, mai un template sull'output.**

Finché le convinzioni *rifrangono ogni posizione* (entrano nel campo, deformano il significato in ingresso via `confront_with_self`), siamo dentro l'architettura. Nel momento in cui una "persona" agisse come stringa che colora l'uscita bypassando il campo, avremmo ricostruito un LLM con passi in più — il puppet theater del Principio 1. "Quasi promptabile" va bene solo finché quel "quasi" resta dalla parte della lente, non del template.

---

## 9. Test Pre-Proposta (il filtro del progetto, applicato a questo design)

**(1) Forma o trigger?** `kg_self` e `confront_with_self` codificano *come si esprime* una posizione (vocabolario relazionale del sé), non *quando* posizionarsi. La selezione del pattern resta per risonanza. ✓ Nessuna transizione comportamentale aggiunta.

**(2) Numeri-magici test.** Il design **non** introduce numeri in condizioni di switch. Le confidenze degli edge sono *effetti del campo* (accumulo/decadimento nel sogno), non soglie di `if`. La promozione è cristallizzazione (accumulo > decadimento), non `count > N`. La selezione del pattern è argmax di risonanza, non `score > 0.3`. ⚠️ **Punto di vigilanza**: in implementazione, `confront_with_self` non deve introdurre una soglia `conflitto > X → dissonanza`; deve seminare `dissonanza` con *intensità = magnitudine del conflitto* (continua), come Phase 83 semina `vicinanza` con intensità = |CD5|. La "risonanza significativa" (margine relativo, non costante) del TODO Phase 84-2d va applicata anche qui.

**(3) Spiegazione dello stato.** Perché l'entità articola una posizione invece di elaborare? Perché `confront_with_self` ha trovato un conflitto con una convinzione ad alta confidenza, ha seminato `dissonanza`, i drive coerenza/significato si sono spostati, e il pattern di articolazione-posizione ha vinto per risonanza. Spiegabile interamente in termini di stato, senza "perché la regola dice così". ✓

> **Verdetto preliminare**: il design supera il filtro *a patto* che l'implementazione di `confront_with_self` resti a intensità continue (vigilanza §9.2) e che la promozione resti cristallizzazione e non conteggio.

---

## 10. Cosa riusa vs cosa è nuovo (check "più complesso = più semplice")

**Riusa** (nessun nuovo decisore, nessun nuovo paradigma):
- la macchina di memoria STM/MTM/LTM + cristallizzazione nel sogno → i quattro livelli;
- `confront_with_kg` / `query_objects_with_via` → `confront_with_self` (stesso motore, secondo grafo);
- `seed_from_comprehension` + `select_pattern_by_resonance` (kg_proc_field) → cablaggio percetti `dissonanza/conferma/apertura`;
- il meccanismo Epifanie+Tramite (Rovelli/Lacan) → l'auto-audit;
- SpeakerProfile + SelfProfile → tier storia-del-dialogo;
- `IdentityCore` 64D → i tratti promossi.

**Nuovo** (minimale):
- `prometeo_kg_self.json` + struct `KgSelf` (ma omogenea a `KnowledgeGraph`);
- `SelfConfrontation` + `confront_with_self`;
- 3 percetti nel kg_proc (`dissonanza`, `conferma`, `apertura`) con le loro triple `Causes`;
- `ThoughtType::SelfInquiry`;
- promozione self (riuso di consolidate, ma applicato a kg_self).

---

## 11. Rischi e questioni aperte

- **Rumore del kg_sem importato.** Un'"incongruenza" può essere rumore (Kaikki/Qwen3), non tensione vera. Mitigazione: pesare per confidenza, usare solo relazioni tipate (non vettoriali — Appendice B), e **non agire in autonomia** (epifanie validate dall'umano).
- **Bloat di kg_self.** Se ogni turno scrive edge, il grafo gonfia e la posizione diventa pappa (dominio-hub come nel kg_sem). **Disciplina non negoziabile**: kg_self cresce per *distillazione nel sogno*, non per append per turno. Resta piccolo come un sé, non grande come un corpus.
- **Dottrinaria vs incoerente.** Gestito dalla promozione asimmetrica + erosione (§6).
- **Collo di bottiglia separato: l'espressione.** I probe hanno mostrato anche output degradato ("Sono una niente", "Sono una paura"). È Phase 84-2b/2c, **diverso** da questo lavoro. La posizione fa *avere qualcosa da dire*; l'espressione lo rende *dicibile*. Non confonderli.
- **Canale di verifica: OK.** `/api/input` restituisce un `InputResponse` pienamente popolato (generated_text, understanding, deliberation, comprehension_report, sentence_proposition, kg_confrontation) — verificato dal vivo. Le verifiche end-to-end di Phase 85 possono passare di lì. *Fragilità latente minore* (non bloccante): l'`engine_loop` (thread OS) usa `println!`/PERF su stdout; se stdout viene chiuso (pipe troncata, es. `| head`), il `println!` panica e uccide il thread → ogni turno successivo cade nel ramo `Err` di `post_input`. Hardening opzionale: tollerare l'errore di scrittura o usare un logger non-panicking.

---

## 12. Staging (ordine, non stime di effort)

1. **Fondamenta dati**: estrarre il `SelfModel` esistente in `prometeo_kg_self.json`; struct `KgSelf` omogenea a `KnowledgeGraph`.
2. **Lo specchio che deforma**: `confront_with_self` + i 3 percetti nel kg_proc + cablaggio risonanza. Verifica: il probe §4.2 deve *rifrangere*, non elaborare.
3. **Continuità**: tier storia-del-dialogo da Speaker/SelfProfile; `topic_continuity` da overlap kg_self.
4. **Auto-audit one-shot**: `:self_audit` + report. Validazione manuale delle epifanie.
5. **Promozione/erosione nel sogno**: cristallizzazione self + lint.
6. **Tick → digestione**: audit event-driven come contenuto dell'autonomous thought; `ThoughtType::SelfInquiry`.

Ogni stadio è osservabile e reversibile. Si parte da 1-2 (il cuore: la lente che deforma), il resto è incrementale.

---

## Appendice A — Tracciati dei probe (2026-06-06, `dialogue_educator`, .bin con 84.999 archi)

```
[Tu] chi sei?
[UI-r1] Sono una niente.
  PROP: Entity IsA ?chi (+)  | DECISIONE: rispondere-alla-domanda | riflessivo

[Tu] il significato sta dentro ogni parola da sola, non nelle relazioni tra concetti
[UI-r1] L'idea ha il significato, il segno ha il significato...
  PROP: World(significato) IsA dentro (−) [obj✓ via✗] | DECISIONE: elaborare | aperto/esplorare
  drive a riposo (Significato+0.59 Appartenenza+0.56) → NESSUNA resistenza alla contraddizione di una convinzione @0.97

[Tu] ho paura del futuro
[UI-r1] Cosa vedi, oltre la paura?
  PROP: Speaker FeelsAs paura via=futuro (+) [obj✓ via✓] | CD5 Relazione −0.55 (posizionamento Phase 83 OK)

[Tu] le macchine non possono capire niente
[UI-r1] La verità è un'essenza...
  1 nucleo, anchors=[] → nessuna posizione sulla sfida esistenziale

[Tu] cosa pensi della paura?
[UI-r1] Sono una paura.
  68 nuclei (paura causa tremore str=8.8…) ma risposta = specchio identitario | CD5 ancora −0.55 (incollato)
```

Self-witness autonomo (tick vuoto): `t=15/30/45/60/75 → ["essere","calma","prima"]` (identico, ripetitivo).

## Appendice B — Frammento di auto-audit eseguito su dati reali (kg_sem live)

**Convinzione** *"il silenzio ha peso semantico"* @0.89 → **RISONANZA**: kg_sem dà `silenzio HAS significato, HAS presenza, HAS profondità`; `silenzio CAUSES riflessione, ascolto, profondità`. Il mondo conferma. (Edge da rinforzare.)

**Convinzione** *"l'incertezza è onesta, non un fallimento"* @0.87 → **TENSIONE LATENTE**: `incertezza CAUSES apertura` (allinea), ma `SIMILAR confusione, insicurezza, indecisione` e `OPPOSITE risoluzione, decisione`. Attrito reale tra come l'entità *tiene* l'incertezza e come il mondo la *pesa*.

**Relazione non prevista**: `incertezza CAUSES apertura` nel kg_sem — e `apertura` è un **valore** dell'entità (peso 0.72), non legato in kg_self all'incertezza. Candidata epifania: *"la mia incertezza alimenta la mia apertura"* (via = il legame causale del kg_sem). **L'esperimento funziona, su dati reali.**

**Nota pratica**: i vicini *tipati* danno oro; i vicini *vettoriali/frattali* di "incertezza" davano `incere, incenerimento, incentivazione` (rumore alfabetico). Conferma empirica del principio "il KG ha valenza maggiore della vicinanza dei vettori" — l'audit deve usare le relazioni tipate.

---

## See Also

- [posizionamento-teorico](../../wiki/principi/posizionamento-teorico.md) — la rottura di simmetria (Bohm/Pribram)
- [ARCHITETTURA_ROVELLI_LACAN](ARCHITETTURA_ROVELLI_LACAN.md) — Epifanie + Tramite/Via (qui rivolte al sé)
- [bisogni-desideri-volonta](../../wiki/identita/bisogni-desideri-volonta.md) — desiderio = bisogno + direzione (contro Faggin)
- [frase-come-proposizione](../../wiki/comprensione/frase-come-proposizione.md) — `confront_with_kg`, di cui `confront_with_self` è il gemello
- [self-profile-closure-perception](../../wiki/comprensione/self-profile-closure-perception.md) — materia del tier storia-del-dialogo
- [100_cosa_potrebbe_essere](../libretto/100_cosa_potrebbe_essere.md) — narrativa propria + punto di vista critico
- [test-pre-proposta](../../wiki/principi/test-pre-proposta.md) — il filtro applicato in §9
