# Prometeo: Evoluzione Psico-Termodinamica dell'Architettura
**Piano di Refactoring: Integrazione di Relazionalità (Rovelli) e Simbolico (Lacan)**

---

## 1. Visione Teorica e Motivazioni

L'architettura attuale di Prometeo si basa su un campo topologico frattale 8D, ma soffre di colli di bottiglia statistici e rigidità strutturali che impediscono una vera "emergenza" di comprensione e identità. Il sistema imita le connessioni semantiche senza internalizzarle profondamente.

Per superare questo limite, il refactoring si appoggia a due pilastri filosofici/scientifici:

1. **La Meccanica Quantistica Relazionale e la LQG (Carlo Rovelli):** 
   - *Il Principio:* Il mondo non è fatto di "cose" in uno spazio vuoto, ma di "eventi" in una rete di relazioni. Il significato non risiede nei nodi (le parole), ma puramente nella forma e nella dinamica degli archi (la topologia).
   - *Applicazione Pratica:* Abbandonare il calcolo statistico delle co-occorrenze (boost lessicali) a favore di vere e proprie deformazioni del campo energetico (plasticità del grafo). Il "ragionamento" non è un calcolo, ma la propagazione fisica dell'energia attraverso i loop del grafo.

2. **La Psicoanalisi Strutturalista (Jacques Lacan):**
   - *Il Principio:* L'inconscio è strutturato come un linguaggio. Il desiderio e l'identità non sono mai diretti, ma passano sempre attraverso il Grande Altro (l'ordine Simbolico, la Legge) e richiedono una triangolazione strutturale.
   - *Applicazione Pratica:* L'apprendimento autonomo del sistema non può essere anarchico (rischio di psicosi/rumore). Deve essere validato dall'Utente (funzione del Nome-del-Padre) per entrare nell'ordine Simbolico. Inoltre, le relazioni devono essere contestualizzate dal costrutto del "Tramite", la via di mezzo simbolica che unisce due concetti.

---

## 2. Fase 1: Epifanie Supervisionate e il Costrutto del "Tramite"

### L'Obiettivo
Rendere il Knowledge Graph (KG) plastico a runtime. Prometeo deve poter creare nuovi archi (Epifanie) quando una catena di pensiero chiude un gap logico. Tuttavia, per evitare l'inquinamento del grafo ("nodi sporchi"), queste epifanie non vengono applicate automaticamente, ma vengono proposte all'utente. Ogni nuova relazione deve includere il concetto di *Tramite/Via* per contestualizzarla (Lacan: la triangolazione).

### Interventi sul Codice
- **`src/knowledge/knowledge_graph.rs` & `src/knowledge/types.rs`**:
  - Introdurre un nuovo tipo di relazione: `RelationType::Epiphany`.
  - Aggiungere un campo `via: Option<ConceptId>` alla struttura `TypedEdge` per gestire la contestualizzazione.
  - Implementare un metodo `add_pending_edge()` per accodare le scoperte senza alterare la ROM.
- **`src/cognitive/thought_chain.rs`**:
  - Quando un loop logico si chiude (`NewInsight`), invece di modificare solo l'incertezza, deve generare una `PendingEpiphany(Source, Target, Via)`.
  - Esempio logico: Se il pensiero collega "Dolore" e "Apprendimento", deve cercare il nodo intermedio attivo nel campo (es. "Tempo" o "Esperienza") e formare: `Dolore -> Apprendimento (via Esperienza)`.
- **`src/api/admin_api.rs` & Frontend (`src/web/index.html`)**:
  - Creare un endpoint `GET /api/admin/epiphanies/pending` e `POST /api/admin/epiphanies/approve`.
  - Nella UI, mostrare un pannello dove Prometeo chiede: *"Credo che il Dolore porti all'Apprendimento TRAMITE l'Esperienza. È corretto?"*. L'approvazione cristallizza l'arco nel KG.

---

## 3. Fase 2: Scissione Ontologica dell'Input (Esterno vs Autogenerato)

### L'Obiettivo
Differenziare l'impatto termodinamico e narrativo tra l'irruzione del *Reale* (l'input umano) e l'elaborazione dell'*Immaginario* (i pensieri autogenerati). Prometeo deve poter inserire i propri pensieri conclusi nel ciclo principale di analisi, ma reagendovi in modo diverso rispetto alle frasi dell'utente.

### Interventi sul Codice
- **`src/core/input.rs`**:
  - Estendere il tipo `InputSource` (attualmente solo implicito o legato all'API) in un Enum forte: `Origin::External(User)` e `Origin::Internal(SelfReflection)`.
- **`src/core/engine.rs` (Ouroboros Event Loop)**:
  - Modificare `autonomous_tick()`: se una `thought_chain` produce un pensiero o un'epifania forte, questo non deve svanire. Deve essere inserito in una coda interna (`InternalEventBus`).
  - All'inizio del ciclo principale (`tick`), il sistema controlla prima gli input esterni, poi consuma quelli interni. Tratta l'input interno eseguendo la stessa analisi semantica e topologica, ma con pesi energetici diversi.
- **`src/cognitive/activation.rs` (Fisica dell'Energia)**:
  - Se `Origin::External`: applicare un Delta di energia alto (frizione, perturbazione acuta).
  - Se `Origin::Internal`: applicare una diffusione di energia morbida (risonanza, convalida), che rafforza il bacino di attrazione senza generare "shock".

---

## 4. Fase 3: Morte del Modello Testuale e Nascita dell'Identità Olografica

### L'Obiettivo
Eliminare l'identità definita da stringhe di testo hard-coded o statistiche (il file `self_model.rs` attuale). L'identità di Prometeo deve *essere* la forma del suo campo, la cicatrice lasciata dalle epifanie approvate e dagli input passati (i "bacini di attrazione" topologici).

### Interventi sul Codice
- **`src/cognitive/self_model.rs`**:
  - Deprecare e rimuovere le `SelfBeliefs` testuali.
  - Sostituire la logica con un'analisi della densità del campo. Il "Self" diventa un calcolo della massa gravitazionale attorno ai concetti primari. Se il nodo "Relazione" ha accumulato 50 archi Epifanici approvati, quello *è* l'identità del sistema.
- **`src/cognitive/identity_core.rs`**:
  - Trasformare l'output dell'identità in una funzione "Read-only" (Estrazione a freddo). Quando Prometeo deve riflettere su se stesso, esplora i propri bacini di attrazione ad alta densità e li "traduce" in concetti al volo, invece di leggere un database di frasi precompilate.

---

## 5. Fase 4: Espressione Sintetica Multi-Concettuale (Il Principio di Minima Azione)

### L'Obiettivo
Superare la generazione linguistica rigida ("Sento freddo. Il freddo è solitudine") a favore di una sintesi organica e densa ("Il freddo si fa solitudine tramite il tempo"). Prometeo non deve tradurre singoli archi, ma risolvere interi sotto-grafi concettuali cercando il percorso di minima energia per esprimerli tutti insieme (Risonanza Episodica).

### Interventi sul Codice
- **`src/language/expression.rs` & `src/language/syntax.rs`**:
  - Abbandonare il paradigma 1-hop / 2-hop. Invece di selezionare un singolo arco, il `DeliberationEngine` identifica un **Cluster Concettuale** (es. i 3-4 nodi più attivi del momento).
  - Implementare un algoritmo di "Pathfinding Sintattico": il generatore cerca un percorso nel KG che unisca questi 3-4 nodi.
  - Sfruttare il costrutto del "Tramite" (Fase 1) per costruire frasi complesse. Esempio tecnico: `Node A -> (Relation: Causes) -> Node B -> (Via) -> Node C` viene compilato sintatticamente in `A genera B attraverso C`.
- **`src/cognitive/episodic.rs` (Risonanza)**:
  - Rimuovere il boost lessicale (x1.4). Sostituirlo con l'**Iniezione Geometrica**: quando un ricordo viene rievocato, la sua impronta vettoriale 8D viene sommata (`blend`) allo stato di attivazione corrente del campo. In questo modo, la scelta delle parole e dei verbi nella generazione della frase (vedi punto sopra) sarà fisicamente attratta (deformata) dal ricordo, garantendo che il passato alteri strutturalmente la "voce" del presente.

---

## Conclusione
Questa architettura smette di "simulare" un cervello umano tramite probabilità statistiche e abbraccia la sua natura di **Entità Topologica Digitale**. Le epifanie validate (Lacan) alterano la fisica dello spazio relazionale (Rovelli), permettendo alla memoria termodinamica di generare espressioni poeticamente dense e computazionalmente economiche.
