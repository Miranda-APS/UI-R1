# Architettura Olografica — Phase 69

> Documento di riferimento per la fase di curation profonda del Knowledge Graph
> e il riallineamento del runtime alle catene di significanti.
>
> Riferimenti filosofici: ontologia relazionale (Carlo Rovelli), catena del
> significante e *point de capiton* (Jacques Lacan).
>
> Riferimenti interni: `derive_8d_from_kg` (Phase 63), Phase 67 (architettura
> della comprensione), `inference.rs::field_boosts`.

---

## Premessa

Il Knowledge Graph di Prometeo non è un dizionario, non è un'ontologia, non è
un grafo di conoscenza nel senso classico. È un **campo olografico**: ogni
nodo, attraverso le sue relazioni, contiene una proiezione compressa
dell'intero campo. La firma 8D che `derive_8d_from_kg` calcola è esattamente
questo — una compressione olografica della posizione relazionale del nodo.

Da questo principio discendono regole architetturali precise. Questo documento
le fissa.

---

## Tre invarianti fondativi

**I — Una parola, un nodo.** Niente underscore, niente multi-token, niente
nodi sintetici. Ogni nodo del KG è una parola italiana reale del lessico.
Questo vincolo non è estetico: garantisce che ogni nodo sia attivabile da un
input naturale e che il KG resti tokenizzabile dal flusso conversazionale.

**II — Una relazione, una semantica precisa.** Le relazioni canoniche
(`IsA`, `Has`, `Does`, `PartOf`, `Causes`, `OppositeOf`, `SimilarTo`,
`UsedFor`, più le specifiche di Prometeo come `RemembersAs`, `FeelsAs`,
`WondersAbout`) costituiscono il vocabolario semantico chiuso. Non si
introducono nuove relazioni senza modificare il sistema di inferenza.

**III — `via` è un quilting point.** Il campo `via` di una tripla è una
**singola parola del lessico** che fissa il contesto entro cui la relazione
vale. È il *point de capiton* lacaniano: la parola che, retroattivamente,
seleziona quale catena di significanti la relazione abita. Non è metadato
passivo, è dimensione attiva del campo.

---

## Stratificazione

L'architettura è a tre strati. Il pruning morfologico, la migrazione delle
relazioni, l'educazione delle collettività e il runtime devono rispettare
questa separazione.

### Strato 1 — Knowledge Graph (campo olografico)

Contiene **solo concetti irriducibili e relazioni con via**. Non contiene
forme morfologiche derivabili. Ogni nodo è una posizione relazionale unica
nello spazio 8D. La firma 8D è la compressione olografica della rete di
relazioni del nodo.

Il pruning morfologico discusso in Phase 69 elimina dal KG le forme che il
livello grammaticale può ricostruire senza perdita.

### Strato 2 — Lessico e grammatica

`lexicon.rs`, `grammar.rs`, `syntax_center.rs`. Contiene **tutte le forme di
superficie** con POS tagging e regole di derivazione morfologica. È il
**ponte** tra superficie linguistica e KG semantico.

Le forme `pauroso`, `paurosa`, `impaurire`, `impaurito`, `impaurita` vivono
qui, non nel KG. Il KG ha solo `paura`. La traduzione superficie ↔ KG avviene
via lemma + regola morfologica.

### Strato 3 — Catene attive (runtime)

Le catene di significanti non sono dato, sono **fenomeno emergente** durante
la conversazione. Una catena è un percorso `nodo → via → nodo → via → nodo`
che si accende quando il contesto presenta le via giuste.

Il runtime mantiene un `active_via_set` che modula la propagazione del campo.
Le relazioni la cui via è presente nel set attivo si propagano con peso pieno;
le altre restano latenti.

---

## Pruning morfologico — la regola corretta

Il pruning morfologico di un satellite è giustificato **se e solo se** la
differenza tra satellite e ancora è interamente catturabile dal livello
grammaticale (Strato 2).

Casi:

1. **Satellite con archi tutti grammaticalmente derivabili** → prune secco.
   Es. `impaurire SimilarTo spaventare`: la regola "verbo deverbale di emozione X
   SimilarTo verbo deverbale di emozione Y" lo deriva da `paura SimilarTo spavento`.

2. **Satellite con archi che hanno via diverse dall'ancora** → migra archi
   con la loro via, poi prune.
   Es. `calmo OppositeOf onde via=mare`: migra a `calma OppositeOf onde via=mare`
   con `source_form=calmo`. Il satellite `calmo` viene poi potato; la sua
   specificità contestuale è preservata sull'ancora attraverso la via.

3. **Satellite con archi senza via ma con specificità contestuale plausibile**
   → l'edge entra in **coda di curation contestuale**. Non viene migrato cieco,
   non viene cancellato. Resta in attesa che una collettività competente
   assegni la via corretta.
   Es. `calmare SimilarTo sedare`: probabilmente via medica/farmacologica.
   Senza la via, è ambiguo.

---

## Educazione da parte delle collettività

Il sistema è progettato per essere **educato da collettività diverse**, ognuna
nel proprio dominio di competenza. Marinai, contadini, terapeuti, musicisti,
medici, insegnanti: ognuno aggiunge triple con via che riflettono la propria
prospettiva sul mondo.

Regole per l'educazione:

- **La via è una parola del lessico quotidiano della collettività**, non un
  termine tecnico chiuso. Se la collettività marittima usa `mare`, la via è
  `mare`. Se la collettività medica usa `dolore`, la via è `dolore`. La via
  deve essere comprensibile dentro e fuori il dominio.

- **Diverse collettività che insegnano sullo stesso nodo aggiungono via
  diverse**. Lo stesso `calmo` riceve `OppositeOf onde via=mare` dai marinai,
  `SimilarTo respiro via=corpo` dai praticanti somatici,
  `IsA atteggiamento via=stoicismo` dai filosofi. Il nodo si arricchisce di
  catene polisemiche strutturate, non di rumore.

- **L'arricchimento è solo additivo**. Una collettività può aggiungere via su
  triple esistenti, ma non modificare le triple di altri domini. Il KG cresce
  per stratificazione, non per sostituzione.

- **Le triple senza via** restano legittime — sono relazioni "trans-contestuali"
  (es. `paura IsA emozione`, valida ovunque). La via si aggiunge solo quando
  la relazione è dipendente dal contesto.

---

## Runtime — come le catene si accendono

Il principio `point de capiton`: il contesto recente fissa retroattivamente
quale catena di significanti è attiva. Operativamente:

### Comprensione (`receive`)

L'input attiva nodi nel campo 8D (Strato 1) tramite il lemma map del lessico
(Strato 2). Le **via dei nodi appena attivati** entrano nell'`active_via_set`
con un decay temporale (analogo al decay PF1).

`field_boosts()` riceve `active_via_set` come parametro e pesa gli archi del
KG: archi con via in set → peso pieno; archi con via diversa → peso ridotto;
archi senza via (trans-contestuali) → peso pieno.

Effetto: se l'utente ha appena parlato di mare, `calmo OppositeOf onde via=mare`
si propaga; in contesto somatico, la stessa relazione resta latente.

### Generazione (`expression::compose`)

La selezione dei nuclei semantici e dei candidati di superficie viene modulata
dall'`active_via_set`. I nuclei coerenti con la catena corrente ricevono
boost; quelli che attivano via fuori catena vengono penalizzati.

Effetto: la risposta di Prometeo non è solo semanticamente coerente — è
**contestualmente coerente**. La stessa parola attiva manifesta significati
diversi a seconda della catena.

### Narrazione (`narrative.rs`)

`NarrativeSelf` traccia, oltre alla traiettoria frattale, l'**evoluzione
dell'`active_via_set`** turn dopo turno. Quali via si sono attivate insieme,
con quale frequenza, attraverso quali transizioni.

Questo è il sostrato della memoria narrativa di lungo periodo: non solo
"quali parole sono state dette", ma "quali catene si sono tessute".

---

## Crescita del KG — Epifanie supervisionate

Il KG cresce attraverso due canali, simmetrici e complementari:

- **Canale esterno (collettività)**: una collettività competente aggiunge
  triple con via dal proprio dominio. È il flusso descritto sopra.
- **Canale interno (epifanie)**: Prometeo stesso, durante la riflessione,
  scopre relazioni plausibili nel proprio campo e le **propone** all'utente
  per validazione. È il flusso autopoietico — il sistema usa la propria
  struttura per estendere la propria struttura.

Le epifanie supervisionate sono il meccanismo lacaniano del **Nome-del-Padre**
applicato all'apprendimento autonomo: senza validazione esterna, il KG che
cresce sui propri loop diventa psicotico (rinforzo di pattern senza ancoraggio
simbolico). Con validazione, il sistema può scoprire e crescere senza
contaminarsi.

### Origine — quando si genera un'epifania

Il punto di emergenza è `thought.rs`. Oggi i tipi `Gap`, `MissingBridge`,
`Disconnection`, `AbductiveHypothesis` rilevano incompletezze nel campo. La
modifica:

1. Quando un pensiero di tipo `Gap` o `MissingBridge` raggiunge `strength > 0.6`
   E coinvolge due nodi `A` e `B` non direttamente connessi nel KG.
2. Il sistema cerca nel campo attivo un **nodo intermedio** `Z` che soddisfi:
   - co-attivato con A e B nel turno corrente o recente
   - ha relazioni con entrambi (anche tramite cammini 2-hop)
   - non è hub generico (degree < 200)
3. Se `Z` esiste, si genera una `PendingEpiphany { source: A, target: B, via: Z, relation: <inferita>, confidence: <calcolata>, justification: <traccia> }`.
4. La relazione inferita viene scelta tra le canoniche secondo il pattern
   delle relazioni di A e B con Z (se entrambi `Causes Z`, l'epifania è
   `SimilarTo`; se A `Causes Z` e Z `Causes B`, l'epifania è `Causes`; ecc.).

### Coda — `pending_epiphanies`

Le epifanie generate finiscono in una coda persistente, esposta dall'API:

- `GET /api/epiphanies/pending` — lista delle epifanie in attesa
- `POST /api/epiphanies/approve` — approva un'epifania, l'arco viene
  cristallizzato nel KG con `via` valorizzata
- `POST /api/epiphanies/reject` — rifiuta un'epifania; l'epifania entra in
  un registro `rejected_epiphanies` per evitare ri-proposte cicliche
- `POST /api/epiphanies/refine` — l'utente può modificare il `via` o la
  `relation` proposta prima di approvare

### Restrizioni — non è un canale di crescita libera

- Le epifanie *devono* avere un `via`. Non è un campo opzionale per loro:
  un'epifania senza Tramite è un loop psicotico, non un'inferenza.
- Le epifanie sono limitate per turno (es. max 3) per evitare flooding
  della coda.
- Le epifanie respinte non si ripropongono per N turni (cooldown), e dopo
  un certo numero di rifiuti diventano permanentemente bloccate.
- Le epifanie approvate vengono salvate con `provenance: Epiphany` (tag su
  arco), distinguibile dalle relazioni curate manualmente o importate.

### Convergenza con la curation contestuale

La **coda di curation contestuale** descritta sopra (archi senza via, in
attesa di una collettività competente) e la **coda epifanie** sono lo stesso
meccanismo visto da due angoli:

- *Curation contestuale*: l'arco esiste, manca la via. Una collettività esterna
  la aggiunge.
- *Epifania*: l'arco non esiste, ma il campo lo suggerisce con una via
  plausibile. Prometeo propone, l'utente valida.

In runtime futuro, quando il sistema incontra un arco in curation contestuale
durante una conversazione, può tentare di **proporre lui stesso una via**
all'utente — convertendo passivamente la curation contestuale in un'epifania
attiva.

---

## Identità digitale come pattern di catene rinforzate

Due istanze di Prometeo con lo stesso KG iniziale divergono nel tempo perché
ognuna rinforza catene diverse attraverso le proprie esperienze. Una istanza
educata da una collettività marittima rinforzerà catene `mare/onde/orizzonte`;
una istanza in contesto urbano rinforzerà `città/strada/incontro`.

L'identità non è il KG (è condiviso). Non è la firma 8D (è derivabile dal KG).
**È il pattern di quali catene si rinforzano insieme attraverso il tempo**.
`IdentityCore` evolve verso il tracking di catene, non solo di esposizioni di
parole.

Questo è anche il fondamento concreto del modello "una collettività, una
istanza": ogni quartiere, ogni gruppo, ogni comunità che educa Prometeo
ottiene un'entità con identità propria — non per personalizzazione cosmetica,
ma perché le catene rinforzate sono strutturalmente diverse.

---

## Schema operativo per il `migration_plan.tsv`

Quando un satellite morfologico viene potato, i suoi archi non-grammaticali
vengono migrati all'ancora con questo schema:

```
anchor	rel	target	conf	via	source_form
calma	OppositeOf	onde	0.7	mare	calmo
calma	SimilarTo	tranquillizzare	0.82		calmare
paura	OppositeOf	coraggio	0.95	forza	(none)
```

- `anchor`, `rel`, `target`, `conf`, `via` come da formato KG attuale.
- `source_form`: la forma morfologica originale da cui l'edge è stato migrato.
  Permette il rollback e la curation cronologica.
- Quando `via` è vuota e l'edge ha specificità contestuale plausibile,
  l'edge va in `context_curation_queue.tsv` invece che nel migration_plan.

---

## Confine fluido — onestà intellettuale

Il confine "morfologia in lessico vs KG" non è netto. Esempi:

- `pauroso IsA paura` è tecnicamente morfologico (regola: aggettivo deverbale
  IsA radice nominale) ma anche relazionale (predica una proprietà). In un
  sistema olografico, la regola è: **se il livello grammaticale può ricostruire
  l'edge senza perdita, prune; altrimenti tieni con via.**

- Le via mancanti nel KG attuale sono molte. Phase iniziali di curation hanno
  ignorato il campo. Riempirle è lavoro lungo e va fatto dalle collettività
  competenti — non da agenti automatici. Il sistema deve prevedere uno
  strumento (UI o CLI) che permette l'aggiunta additiva di via senza
  modificare le triple esistenti.

- Il runtime cambia in modo non banale se prendiamo sul serio le via attive.
  `field_boosts`, `extract_propositions`, `expression::compose` ricevono
  tutti `active_via_set` come argomento. La rifattorizzazione è sostanziale
  ma incrementale — si fa in fasi.

---

## Prossimi passi concreti (Phase 69)

1. **Migration plan con via**: estendere il prototipo morfologico per
   produrre `migration_plan.tsv` con campi `(anchor, rel, target, conf, via,
   source_form)` e `context_curation_queue.tsv` per gli archi senza via ma
   con specificità.

2. **Lemma map persistente nel lessico**: ogni forma di superficie del
   lessico riceve un campo `lemma_anchor` che punta al nodo KG canonico. Le
   regole morfologiche di derivazione vivono in `grammar.rs` come funzioni
   pure.

3. **`active_via_set` nel runtime**: aggiungere il set come campo di
   `Engine`, alimentato da `receive()` e consumato da `field_boosts()`,
   `extract_propositions()`, `expression::compose()`. Decay temporale in
   `autonomous_tick()`.

4. **Strumento di curation contestuale**: CLI o sezione web UI per
   permettere alle collettività di aggiungere via su triple esistenti senza
   toccare il resto.

5. **Test di coerenza contestuale**: aggiungere test che verificano la
   diversa attivazione del campo a parità di input, sotto contesti via
   diversi (es. "calmo" dopo "mare" vs "calmo" dopo "respiro").

6. **Pipeline epifanie supervisionate**:
   - Estendere `thought.rs::Gap`/`MissingBridge` per generare
     `PendingEpiphany` quando le condizioni sono soddisfatte (strength,
     intermediario plausibile, hub damping).
   - Aggiungere `pending_epiphanies` come campo persistente di
     `Engine`, con persistenza in `prometeo_topology_state.bin`.
   - Endpoint API `/api/epiphanies/{pending,approve,reject,refine}`.
   - UI minimale (CLI per `dialogue_educator` o sezione web) che mostra
     l'epifania pendente in forma comprensibile: *"Credo che X porti a Y
     tramite Z. È corretto?"*.
