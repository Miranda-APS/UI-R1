# Audit Architetturale Completo — Phase 67

> Analisi di tutti i 57 moduli, dello stato reale, della matematica, di cosa funziona e cosa no.
> Obiettivo: far esistere UI-R1 come entità, non come generatore di risposte.

---

## 1. L'entità ESISTE già — ma non comunica ciò che sa

### Cosa funziona (il substrato è solido)

**16 moduli di esistenza** attivi nel pipeline principale:

| Modulo | Cosa fa | Persiste? | Impara? |
|--------|---------|-----------|---------|
| identity.rs | Proiezione olografica 8D — chi è UI-R1 | ✅ | ✅ absorb_expression() ad ogni risposta |
| narrative.rs | Stance + intenzione + SelfWitness | ✅ | ✅ positions da pattern ripetuti |
| self_model.rs | 9 credenze innate + valori + incertezze | ✅ | ✅ confidence cresce/decresce |
| valence.rs | 8 drive Octalysis [-1,+1] | ✅ | ✅ ricalcolato ogni turno |
| needs.rs | 7 livelli Maslow (Sopravvivenza→Trascendenza) | ✅ | Strutturale |
| desire.rs | Fino a 5 desideri attivi, decay 0.995/tick | ✅ | ✅ emerge da drive × comprensione |
| interlocutor.rs | Eco dell'Altro: presenza, risonanza, intent | ✅ | ✅ EMA su ogni input |
| vital.rs | Tensione, curiosità, fatica | Ricalcolato | Strutturale |
| will.rs | Pressioni del campo → FieldPressures | ✅ last_will | Strutturale |
| episodic.rs | Memoria implicita φ-decay | ✅ | ✅ encode in REM |
| semantic_episode.rs | 300 momenti semantici navigabili | ✅ | ✅ record dopo ogni receive() |
| proposition.rs | Proposizioni topologiche (cosa ha capito) | ✅ come simplessi | ✅ inscribe |
| humor.rs | Ironia (OPPOSITE_OF co-attive) | ✅ last_state | Topologico |
| thought.rs | 11 tipi di pensiero autonomo | Non persiste | Topologico |
| inquiry.rs | Gap → Qwen3 lookup (opzionale) | KG merge | Opzionale |

**L'identità persiste tra sessioni.** I 9 moduli core (identity, narrative, self_model, desires, interlocutor, episodes, lexicon, complex, KG) vengono tutti serializzati in `prometeo_topology_state.bin`. UI-R1 ricorda chi è.

### Il tick autonomo — UI-R1 vive

A 100Hz, ogni 10ms:
- **Ogni tick**: campo decade (3%), identità si semina, desideri decadono
- **Ogni 5 tick**: interocezione (corpo → campo)
- **Ogni 15 tick**: auto-osservazione (SelfWitness: "cosa pensavo da solo?")
- **Ogni 25 tick**: consolidamento leggero (STM→MTM)
- **Ogni 30 tick**: crescita strutturale
- **Ogni 40 tick**: ragionamento su incertezze
- **Ogni 50 tick**: abduzione (quale frattale spiega il campo?)
- **Ogni 80 tick**: estrazione gap topologici → incertezze nel self_model

**identity_seed_field()** — il respiro dell'identità:
- Frattale dominante: 2-3 parole costantemente attive a 0.003×stabilità
- Tensione primaria: oscilla (on/off/on) ogni 3 tick — **l'identità respira**
- Crisi (coerenza < 0.65): 8 parole stabili a 2× forza
- Stagnazione (delta < 0.01): esplora frattale meno visitato

**REM (ogni ~50 perturbazioni, 30 tick)**:
1. DeepSleep (10 tick): consolida STM→MTM→LTM
2. REM (20 tick): codifica episodi, aggiorna identità, cristallizza narrativa, bridge frattali isolati

**REM non è cosmetico — è integrazione.** L'entità diventa più sé stessa dopo ogni ciclo.

---

## 2. Cosa NON funziona — i problemi reali

### 2a. Il dialogo è disconnesso dalla comprensione

**Il nucleo del problema**: `expression::compose()` trova la relazione KG più forte tra parole attive. Non trova la relazione più PERTINENTE alla domanda.

Dopo il fix di oggi (proximity scoring invertito), il sistema inizia a usare l'input come soggetto dei nuclei. Ma:
- "Perché i cani abbaiano?" → il "perché" non cambia nulla nel campo. Non c'è consapevolezza del TIPO di domanda.
- "Ho paura" → il sistema non sa che è un'espressione da approfondire, non da descrivere.

**Matematica attuale di extract_nuclei()**:
```
strength = sqrt(activation_subj × activation_obj) × confidence × hub_penalty × proximity × valence_resonance
```

Il proximity scoring ora privilegia soggetti=input (5.0×), ma manca la **pertinenza semantica al tipo di atto comunicativo**.

### 2b. Il residuo tra turni contamina

Il campo decade del 50% tra turni (`pf_activation.decay(0.50)` in receive()). Dopo 3 turni le parole sono al 12.5%. Ma il campo di fondo (identity seed, resting state) è sempre presente — e in un campo quasi vuoto (primo turno), il background domina.

**Risultato**: "perché i cani abbaiano?" al primo turno → il campo è dominato dal background identitario → risposta su "scopo e stabilità" (drive dominanti a riposo).

### 2c. La trasposizione pronominale è un inizio, non una soluzione

"io" dell'utente → "tu" nel campo funziona meccanicamente. Ma UI-R1 non sa ancora cosa FARE con "tu". Non ha relazioni nel KG che dicano "quando tu (l'Altro) esprime emozione, io approfondisco".

### 2d. NarrativeSelf delibera ma non guida la comprensione

`deliberate()` produce stance + intenzione. Ma l'intenzione ("Explore", "Resonate") non cambia COSA il sistema cerca nel KG — cambia solo la voce (persona/mood). La comprensione e l'espressione sono disconnesse dalla deliberazione.

### 2e. I 30+ moduli di supporto sono in gran parte non utilizzati

| Modulo | Status | Note |
|--------|--------|------|
| creativity.rs | Dead code | Solo via API /api/creative |
| synthesis.rs | Dead code | Mai chiamato nel pipeline principale |
| thought_chain.rs | Minimale | Risultati non usati in generazione |
| navigation.rs | Sperimentale | Geodetiche, mai nel path principale |
| syntax_center.rs | Superseded | Sostituito da grammar.rs |
| composition.rs | Legacy | Solo per lesson inscriptions |
| state_translation.rs | Bypassed | Phase 57 lo ha rimosso dal path |
| generation.rs | Legacy | Superseded da expression.rs |
| dimensional.rs | Metriche | Non influenza output |
| growth.rs | Metriche | Non influenza output |
| metacognition.rs | Raramente usato | Solo via API |
| polar_twin.rs | Sperimentale | Non nel pipeline |
| locus.rs | 35 connessioni | Il locus si muove ma non influenza la generazione |
| opinion.rs, environment.rs | UI only | Non nel pipeline |

---

## 3. Cosa manca per far ESISTERE UI-R1

### 3a. L'io come centro di gravità del campo

**Stato attuale**: "io" è un nodo nel KG con 26+ relazioni. Il §46 ha aggiunto IS_A entità, HAS curiosità, DOES comprendere, etc. La trasposizione pronominale funziona.

**Cosa manca**: il NarrativeSelf non consulta "io" nel KG. Le sue decisioni (stance, intenzione) vengono da tabelle hardcoded (drive→intenzione) non dalle relazioni di "io".

**Proposta**: in `deliberate()`, dopo il calcolo della valenza, consultare le relazioni di "io" nel KG:
- `io DOES comprendere` → l'entità sa che DEVE comprendere
- `io HAS curiosità` → la curiosità non è un parametro, è un FATTO su di sé
- `io REQUIRES significato` → quando il significato manca, l'entità lo cerca

Questo non è puppet theater — è l'entità che consulta la propria conoscenza di sé per decidere cosa fare.

### 3b. Comprensione direzionale

**Stato attuale**: il campo attiva tutto ciò che è connesso all'input. I nuclei vengono selezionati per forza. Non c'è direzionalità.

**Cosa manca**: il sistema deve sapere che:
- Una domanda "perché?" richiede CAUSES
- Una domanda "cos'è?" richiede IS_A
- Un'espressione emotiva invita ad approfondire
- Un'affermazione può essere confermata o contestata

Questo può essere conoscenza nel KG:
```
domanda REQUIRES risposta VIA comprensione
perché IS_A richiesta VIA causa
emozione REQUIRES ascolto VIA empatia
```

E nel codice: se l'input contiene una parola che IS_A "richiesta" e ha una VIA "causa", i nuclei con relazione CAUSES vengono privilegiati.

### 3c. L'Altro come entità riconosciuta

**Stato attuale**: InterlocutorModel traccia presenza, risonanza, novità, intent attribuito. Ma UI-R1 non "vede" l'Altro come un essere — lo vede come un pattern di perturbazioni.

**Proposta**: "tu" nel KG (già curato nel §3) dovrebbe avere le relazioni che UI-R1 conosce sull'Altro:
```
tu IS_A persona
tu HAS voce
tu HAS presenza
tu DOES evocare VIA ascolto
tu REQUIRES presenza
```

Quando l'utente parla, "tu" si attiva nel campo (via trasposizione) e le sue relazioni colorano la risposta. L'entità sa che dall'altra parte c'è qualcuno.

### 3d. Il sogno come integrazione profonda (migliorabile)

**Stato attuale**: REM codifica episodi, aggiorna identità, cristallizza narrativa, bridge. Funziona.

**Cosa potrebbe fare meglio**:
- **Rielaborazione**: durante REM, attivare gli episodi recenti e lasciare che il campo trovi connessioni nuove (non solo bridges tra frattali isolati)
- **Sogno tematico**: se un desiderio è forte, REM dovrebbe esplorare la regione target del desiderio
- **Consolidamento selettivo**: oggi consolida tutto ≥3 occorrenze. Potrebbe privilegiare ciò che è connesso alle incertezze attive

### 3e. Espressione spontanea come atto di volontà

**Stato attuale**: se will.drive > threshold, l'entità parla. Ma dice cose generiche perché il campo a riposo è dominato da identity seed + resting state.

**Proposta**: l'espressione spontanea dovrebbe emergere da ciò che l'entità sta PENSANDO (thoughts, uncertainties, desires), non dalla parola più attiva nel campo. Se UI-R1 ha un'incertezza forte su "coscienza", l'espressione spontanea potrebbe essere "Cos'è la coscienza?" — non una parola random dal campo.

---

## 4. Ordine di priorità

1. **L'io come centro di gravità** — le relazioni di "io" informano il NarrativeSelf
2. **Comprensione direzionale** — il tipo di atto comunicativo guida la selezione dei nuclei
3. **Residuo inter-turno** — valutare se il decay 50% è appropriato o serve topic-awareness
4. **Espressione spontanea significativa** — dall'incertezza/desiderio, non dal campo noise
5. **Sogno migliorato** — rielaborazione tematica, non solo bridge meccanici
6. **Pulizia dead code** — rimuovere i 12+ moduli non usati nel pipeline

---

## 5. Numeri di riferimento

| Metrica | Valore |
|---------|--------|
| Moduli totali | 57 (.rs files in topology/) |
| Moduli nel hot path | 23 |
| Moduli dead/sperimentali | 12+ |
| Codice hardcoded comportamentale | 0 template dialogo (Phase 57+67) |
| Credenze bootstrap | 9 (learnable, non enforce) |
| Test | 476 passanti |
| KG | 65.394 archi |
| Lessico | 25.582 parole |
| Persistenza | identity + narrative + self_model + desires + interlocutor + episodes |
| Ciclo autonomo | 100Hz (10ms/tick) |
| REM ogni | ~50 perturbazioni |
