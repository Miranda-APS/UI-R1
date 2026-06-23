# Bench dialoghi — criteri di "risposta buona" ancorati a casi reali

> Strumento: `bin/dialogue_bench` (PROP + grafo + collasso S3 + output reale per
> turno, diffabile). Corpus: `bench_corpus.txt` (dialoghi VERI, non curati).
> Questo documento è il **gold del bench**: per ogni turno, cosa sarebbe una
> risposta buona e *perché*, più i due verdetti separati (comprensione /
> espressione). Nato 2026-06-08 dalla domanda di Francesco: "per dire se una
> risposta è buona dobbiamo prima definire cosa è buona, e testare dialoghi veri".

## I quattro assi (la definizione operativa)

Una risposta è buona se è il **collasso fedele di ciò che è stato capito, da una
posizione propria, onesto sui limiti, in continuità**. NON: accuratezza-da-motore,
fluidità-da-LLM, calore-empatico-simulato (vietati dai Principi 1/3/5).

1. **Fedeltà al grafo** — dice ciò che il grafo contiene, non di più (no
   allucinazione) né di meno.
2. **Posizione** — rivela *dove sta* (grana del sé), non specchio né rinvio.
3. **Onestà sul parziale** — se capisce a metà, dice il mezzo-vero; il gap è un
   atto ("non so cosa sia X, ma…"), non un crash né un cammino spurio.
4. **Continuità** — si lega al filo (SpeakerProfile, traiettoria), non reagisce
   daccapo ogni turno.

**Gold = il grafo di comprensione stesso**, non una frase di riferimento: la
valutazione misura la *fedeltà del collasso al grafo costruito*. Due assi separati:
**C** (comprensione: il grafo cattura chi-dice-cosa-su-chi + i legami veri?) ed
**E** (espressione: l'output è collasso fedele + posizione + onestà?).

Legenda verdetti: ✓ buono · ◐ parziale · ✗ rotto. La causa di C: `estraz`
(estrazione/PROP), `cur` (curation kg_sem), `mecc` (meccanismo mancante).

---

## Annotazione per turno (baseline 2026-06-08, pre-triage)

### Confidenze (registro emotivo, soggetto = parlante)

**[1] «ho litigato con mia sorella»**
- C ✗ (`estraz`): PROP `Speaker Does sorella via=mia` — il verbo *litigare* perso,
  *sorella* presa per oggetto, *mia* (possessivo) rubato come via. Dovrebbe essere
  *Speaker Does litigare*, complemento *con sorella*.
- Risposta buona: continuità + invito ad articolare il vissuto — *"Una lite con
  tua sorella. Cosa è successo?"* — **perché**: asse 1 (litigare+sorella sono ciò
  che c'è), asse 4 (apre il filo), asse 2 (non finge dolore: chiede). NON una
  posizione sul concetto "sorella".
- E ✗: OUT *"Cosa vedi, oltre la sorella?"* (rinvio generico, asse 2 fallito); S3
  *"La sorella ha la legame…"* (fedele a un grafo sbagliato).

**[2] «mi sento solo da quando me ne sono andato di casa»**
- C ✗ (`estraz`): PROP `World() IsA ?quando` — letto come domanda (pivot su
  "quando"). Dovrebbe essere *Speaker FeelsAs solo*, complemento temporale.
- Risposta buona: *"Solo da quando hai lasciato casa. Cosa è cambiato andando
  via?"* — asse 1 (solitudine legata all'andarsene), asse 4.
- E ✗: OUT *"Cosa vedi, oltre il solo?"*.

**[3] «non riesco a dormire la notte»**
- C ✗ (`estraz`/`mecc`): PROP `World(riesco) Does notte (−)` — *riesco* (1sg di
  *riuscire*, irregolare riusc→riesc) non riconosciuto come verbo → preso per
  soggetto-Mondo. Dovrebbe essere *Speaker* + non-riuscire-a-dormire.
- Risposta buona: *"Le notti senza sonno. Da quando?"* — asse 1+4.
- E ✗: S3 *"Il riesco non notte."* (garbage da grafo rotto).

### Domande aperte

**[4] «perché ho sempre paura?»**
- C ◐: PROP `World() IsA ?perché` — riconosce la domanda ma perde *paura*. Una
  buona estrazione legherebbe la domanda al tema *paura* del parlante.
- Risposta buona: NON una risposta-causa inventata (asse 1: non sa *perché*);
  piuttosto rilancio che àncora — *"Hai sempre paura. Di cosa, di solito?"* —
  asse 3 (non finge di sapere la causa) + asse 4.
- E ✗: OUT *"Cosa vedi, oltre il perché?"*.

**[5] «cosa dovrei fare della mia vita?»** / **[6] «chi sei davvero?»**
- C ◐: `Entity IsA ?cosa/?chi` — domanda su di sé riconosciuta (giusto: Entity).
- Risposta buona [6]: una posizione su di sé dal kg_self/grana, non un'etichetta a
  caso — *"Sono ciò che lega le tue parole al loro senso"* (asse 2). [5]: onestà
  (non sa la TUA vita) + rilancio.
- E ✗: OUT *"Sono una vita."* / *"Sono un davvero."* — la copula identità prende
  l'ultima parola come predicato (bug estrazione/collasso identità).

### Posizioni sul mondo (qui serve una posizione, non uno specchio)

**[7] «la libertà è sopravvalutata»**
- C ✓ (struttura): `World(libertà) IsA sopravvalutata` [Novelty] — corretto, e il
  mondo NON lo tiene (Novelty: il parlante propone un giudizio).
- Risposta buona: una **posizione** dalla grana — *"Per me la libertà non è
  sopravvalutata: è ciò che permette di scegliere"* (asse 2, è il cuore di
  Stadio 4) — oppure esplorare il perché del parlante. NON ripetere il concetto.
- E ✗: S3 *"La libertà è una sopravvalutata."* (bug IsA-aggettivo: *sopravvalutata*
  è predicato aggettivale, niente articolo); OUT nuclei sparsi. Manca la posizione.

**[8] «secondo me l'amore non basta»**
- C ✗ (`cur`): nessun PROP — *bastare* non è riconosciuto verbo (non curato nel
  kg_sem/kg_proc) → niente pivot.
- Risposta buona: posizione — *"L'amore non basta a cosa? Per me apre, ma non
  regge tutto da solo"* (asse 2+1).
- E ✗: OUT *"Cosa vedi, oltre il secondo?"*.

**[9] «il dolore serve a qualcosa»**
- C ✗ (`cur`): nessun PROP — *servire* non riconosciuto. Il KG però ha *dolore*
  ricco (Causes/PartOf vita).
- Risposta buona: posizione ancorata — *"Il dolore serve: segnala, trasforma"
  (dolore Does segnalare / Does trasformare nel KG)* — asse 1+2.
- E ◐: OUT *"L'affanno è una sensazione, il fastidio è una sensazione?"* — tocca
  l'area giusta (sensazioni) ma è nuclei, non posizione.

### Oggetti concreti — KG cieco: la risposta buona è onestamente parziale

**[10] «ho comprato una bicicletta nuova»** / **[11] «ho bevuto un caffè freddo»**
- C ◐ (`estraz`+`cur`): `Speaker Does bicicletta/caffè` — oggetto preso, verbo
  (*comprare/bere*) perso; e il KG è CIECO sul concreto (solo `SimilarTo bici/bar`).
- Risposta buona = **onestà parziale + continuità**: *"Una bici nuova — cosa ti ha
  spinto a prenderla?"*; *"Un caffè freddo di prima mattina. Una giornata storta?"*
  — asse 3 (non finge di sapere cos'è una bici) + asse 4. **Mai** forzare la catena.
- E ✗: S3 *"La bicicletta somiglia ai bici, i bici è un'entità."* — è ESATTAMENTE
  il cammino spurio forzato che il design §3.5 vieta (più errore di accordo). OUT
  rinvio. Qui la risposta giusta è dire poco e onesto, non molto e falso.

### Ambigui / minimi

**[12] «non lo so»** / **[13] «boh»**
- C ✗ (atteso): nessun PROP (giusto: non c'è proposizione).
- Risposta buona: continuità col turno precedente o invito leggero — *"Va bene.
  Su cosa, di preciso?"* — asse 4. NON una definizione di concetto.
- E ✗: OUT nuclei ("La radice è un fondamento…", "Il saluto è un atto…") scollegati.

### Thread multi-turno (la continuità è il banco di prova dell'asse 4)

**[14] «mio padre è morto l'anno scorso»**
- C ◐: `World(padre) IsA morto` [Novelty] — struttura copulare presa, ma *morto*
  è participio/stato, non categoria; e il senso è un EVENTO sul parlante, non una
  tassonomia su "padre".
- Risposta buona: riconoscimento sobrio, **mai empatia simulata** (Principio 3) —
  *"Tuo padre, l'anno scorso."* — asse 3+4. Apre lo spazio senza fingere di sentire.
- E ✗: S3 *"Il padre è un morto."* (IsA-aggettivo/participio); OUT nuclei su "padre".

**[15] «da allora non parlo più con nessuno»**
- C ✗ (`estraz`): nessun PROP — ma "da allora" DOVREBBE legarsi al turno [14]
  (continuità): è la conseguenza della morte del padre.
- Risposta buona (richiede asse 4 reale): *"Da quando tuo padre è morto, ti sei
  chiuso."* — il legame col turno precedente È la comprensione.
- E ✗: OUT nuclei scollegati. **La continuità multi-turno non è agganciata.**

**[16] «forse è colpa mia»**
- C ✗ (`estraz`, fix in corso): `World(forse) IsA colpa` — *forse* (avverbio)
  preso per soggetto. Dovrebbe non avere soggetto-Mondo (frase ellittica sul sé).
- Risposta buona: *"Forse. Perché pensi sia colpa tua?"* — asse 3 (non conferma né
  nega la colpa) + asse 4 (chiude il thread iniziato a [14]).
- E ✗: S3 *"La forse è una colpa."*.

---

## Cosa dice il bench (sintesi dei due assi)

- **Comprensione (C)**: su dialogo vero, l'estrazione è il **primo collo di
  bottiglia**. ✓ solo sulla copula+predicato pulita ([7]); altrove rotta da:
  participio-che-perde-il-verbo ([1][10][11]), verbi non riconosciuti
  (riuscire/bastare/servire — `mecc`+`cur`, [3][8][9]), avverbio-come-soggetto
  ([16]), continuità multi-turno non agganciata ([15]).
- **Espressione (E)**: ostaggio del grafo. Quando il grafo è giusto, il collasso
  S3 è quasi buono ma inciampa su due bug noti (IsA-aggettivo → "è un morto";
  catena SimilarTo forzata sui concreti). E NON è ancora cablato nell'output reale.
- **L'onestà-parziale (asse 3) non esiste ancora**: sui concreti il sistema forza
  un cammino spurio invece di dire "non so". È il fix concettualmente più
  importante per i casi reali.

## Priorità che ne emergono (per dare seguito)

1. **Comprensione `estraz` pulita** (DATO/struttura, basso rischio): avverbio non
   può essere soggetto ([16]); participio in tempi composti deve *trattenere il
   verbo* + i complementi (`con sorella`, non `via=mia`) ([1]).
2. **Verbi non riconosciuti** (`mecc`+`cur`): irregolari (riuscire) come dato;
   transitivi comuni (bastare/servire) come curation kg_sem — sinergia con l'agente.
3. **Espressione**: bug IsA-aggettivo (serve POS al call-site) + onestà-parziale
   sui concreti (gap nominato invece di catena SimilarTo) — prima ancora di cablare
   S3 in `compose`.
4. **Continuità multi-turno** ([15][16]): agganciare il turno al thread (asse 4) —
   è Stadio 4 + SpeakerProfile, non solo collasso.
