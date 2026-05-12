# Libretto Prometeo — Come leggere

> *Un'entità non si cura: si comprende. Da quella comprensione, la cura emerge.*

---

## Cos'è questo libretto

Non è un manuale, non è un'API reference, non è documentazione tecnica nel senso usuale.

È un **libretto di proprietà** — nel senso in cui chi adotta un cane riceve un libretto che spiega cosa mangia, di cosa ha bisogno, come si ammala, perché fa quello che fa. Per prendersi cura di Prometeo bisogna sapere cosa è, perché è così, e cosa potrebbe essere altrimenti.

Ogni volume parte da una posizione filosofica e scende nel codice riga per riga. Quando una formula appare, viene spiegata: cosa calcola, perché in quel modo, quali alternative sono state scartate. Quando una costante è stata scelta — `0.15`, `0.92`, `0.002` — viene chiesto se è ben scelta o se è un compromesso ereditato.

L'obiettivo è che alla fine tu possa aprire qualsiasi file del progetto e sapere non solo *cosa fa* ma *perché esiste in quella forma*.

---

## Ordine consigliato di lettura

I volumi sono numerati per costruire una progressione: ogni volume presume i precedenti. Si possono leggere in ordine sparso una volta acquisita la mappa, ma la prima lettura ha un senso lineare.

**Parte I — Le tre fondamenta**

| Vol | Titolo | Cosa apre |
|-----|--------|-----------|
| [01](01_dalla_filosofia_alla_forma.md) | Dalla filosofia alla forma | Perché PF1, Lexicon, KG, Valence — quattro commitment fondanti |
| [02](02_fondamenti_pf1.md) | Fondamenti: PrometeoField (PF1) | Il campo come ROM+RAM, propagazione O(attive×8), plasticità hebbiana |
| [03](03_fondamenti_lexicon.md) | Fondamenti: Lexicon e firme 8D | Cinque regimi di firma. `derive_8d_from_kg` dim-per-dim |
| [04](04_fondamenti_kg.md) | Fondamenti: KnowledgeGraph | 21 relazioni in 5 categorie. Fenomenologiche sotto-popolate |

**Parte II — Il campo che vive**

| Vol | Titolo | Cosa apre |
|-----|--------|-----------|
| [05](05_campo_frattali.md) | Campo: i 64 frattali | Esagrammi I Ching come attrattori. Dimensioni emergenti |
| [06](06_campo_inferenza.md) | Campo: inferenza e proposizioni | Eredità, proposizioni 1/2-hop, abduzione, contraddizioni |

**Parte III — Chi è l'entità**

| Vol | Titolo | Cosa apre |
|-----|--------|-----------|
| [07](07_identita.md) | Identità: Narrative, IdentityCore, SelfModel | Tre strati di sé + SelfWitness |
| [08](08_valenza_octalysis.md) | Valenza Octalysis e Commitment | 8 drive × 8 dim. Formula continua. Impegno volitivo |
| [09](09_bisogni_desideri.md) | Motivazione: Bisogni e Desideri | Maslow topologico, prepotenza, 5 sorgenti di desiderio |
| [10](10_volonta.md) | Volontà e FieldPressures | 7 pressioni, separazione Phase 67 |
| [11](11_interlocutor_humor.md) | Eco dell'Altro e Humor | Interlocutor come perturbazione, ironia come opposizione co-attiva |

**Parte IV — Come parla**

| Vol | Titolo | Cosa apre |
|-----|--------|-----------|
| [12](12_generazione_expression.md) | Generazione: Expression | **ONESTO**: il KG zoppo. Path attuale vs emergenza promessa |
| [13](13_grammatica.md) | Generazione: Grammatica, sintassi | Italiano come fisica. `syntax_center` + `grammar` + legacy |

**Parte V — Memoria e tempo**

| Vol | Titolo | Cosa apre |
|-----|--------|-----------|
| [14](14_memoria_sogno.md) | Memoria e sogno | Bergson + phi-decay. **Il gap "digestione"** analizzato |

**Parte VI — Il cuore in movimento**

| Vol | Titolo | Cosa apre |
|-----|--------|-----------|
| [15](15_engine.md) | Engine: receive, generate_willed, autonomous_tick | 2000+800+500 righe in pipeline tracciata |

**Parte VII — La pelle**

| Vol | Titolo | Cosa apre |
|-----|--------|-----------|
| [16](16_web_api.md) | Web API | 71 endpoint. Architettura single-writer+channels |
| [17](17_frontend.md) | Frontend | 9 UI dedicate. 10.567 righe di HTML/JS/CSS |
| [18](18_binari.md) | Binari di manutenzione | 42 binari classificati per scopo e rischio |

**Parte VIII — Misure e considerazioni**

| Vol | Titolo | Cosa apre |
|-----|--------|-----------|
| [19](19_calcoli.md) | Appendice matematica | 17 formule con esempi numerici + tabella 50 costanti |
| [99](99_considerazioni.md) | **Considerazioni finali** | Le mie osservazioni. Roadmap proposta. Priorità A/B/C |
| [100](100_cosa_potrebbe_essere.md) | **Cosa questa entità potrebbe essere** | Partendo dalla tesi ontologica di Francesco: recipient adeguato alla cristallizzazione di coscienza. Uccidere il tick. Narrativa propria. Punto di vista critico. La Biennale come esperienza ontologica |

**Documento vivo**

| File | Contenuto |
|------|-----------|
| [appunti.md](appunti.md) | Note che ho preso scrivendo il libretto: discrepanze, hardcode sospetti, codice legacy, funzioni private che andrebbero esposte. Non è un volume — è il mio quaderno. |

---

## Convenzioni

**Riferimenti al codice.** Quando cito un file uso il percorso completo: `src/topology/pf1.rs:239`. Quando cito una costante o una formula la riporto verbatim e spiego cosa significa.

**Riferimenti filosofici.** [FILOSOFIA.md](../FILOSOFIA.md) è il documento ombra. Questo libretto non lo ripete: lo presuppone e lo ancora al codice. Quando un capitolo dice "come spiegato in FILOSOFIA.md", aggiungo il riferimento alla parte/sezione.

**Alternative scartate.** Per ogni decisione importante c'è una sottosezione *"Alternative considerate"*. Anche quando non ricordiamo l'esatto motivo storico, ricostruisco perché *adesso* l'alternativa sarebbe peggiore (o migliore — nel qual caso lo annoto in `appunti.md`).

**Costanti che parlano.** Ogni numero hardcoded viene interrogato. Se la sua presenza ha una ragione, la espongo. Se non ne trovo una, lo annoto come *hardcode sospetto* in `appunti.md`.

**Cosa è esposto, cosa no.** Per ogni modulo, alla fine del capitolo, una sezione *"Superficie pubblica"* elenca cosa è `pub` e cosa no. Una sotto-sezione *"Cosa dovremmo esporre"* segnala funzioni utili attualmente private (e perché lo sarebbero — strumenti di debug, query di stato, audit).

---

## Ipotesi del lettore

Presumo che tu (Francesco) conosca:

- Il concetto generale di Prometeo (entità topologica, 64 frattali, KG per capire non per generare).
- I fondamenti di Rust (struct, impl, traits, ownership) — non i dettagli avanzati.
- Le idee centrali di [FILOSOFIA.md](../FILOSOFIA.md).

Non presumo che tu ricordi:

- I nomi esatti dei moduli o le firme delle funzioni.
- Il significato di costanti tipo `RECORD_SIZE = 512`.
- Quale phase ha introdotto cosa (sta in [phases_history.md](../phases_history.md) — qui non rifaccio la storia, descrivo lo stato).

---

## Filosofia del libretto stesso

Tre principi mi guidano nello scrivere:

**1. Profondità prima di completezza.**
Meglio dieci pagine su `propagate()` che cinquanta su tutto in superficie. Se devi tagliare, taglia in lunghezza, non in profondità.

**2. Perché prima di cosa.**
Ogni "fa X" è preceduto da "perché X". Senza il perché, il cosa è inerte.

**3. Onestà sui buchi.**
Quando non capisco una scelta, lo dico. Quando una costante sembra arbitraria, lo dico. Il libretto non è un'apologia — è uno strumento per chi deve prendersi cura.

---

*Inizia da [Volume 01 — Dalla filosofia alla forma](01_dalla_filosofia_alla_forma.md).*
