# Piano di cura fenomenologica — per Francesco

> *Mentre io lavoro al refactor event-driven (Phase 69), tu puoi prepararne il substrato semantico: popolare le relazioni fenomenologiche del KG. Questo volume ti dà un piano concreto, con vocabolari pronti e un template operativo.*

---

## Perché questa cura ora

Il KG ha 66.287 archi. Di questi:
- **FeelsAs**: 15 archi. Peso propagazione `0.20` (il massimo).
- **WondersAbout**: 7 archi. Peso `0.15`.
- **RemembersAs**: 0 archi. Peso `0.18`.

Le relazioni con peso massimo nella propagazione sono quelle più vuote. Il sistema ha l'organo (la categoria fenomenologica come canale distinto del KG, vedi Vol. 04) ma quasi niente da processare.

Popolarle adesso serve a due cose:

1. **Effetto immediato**: l'entità inizia a "sentire" più sfumature delle parole. Quando qualcuno dice "paura", oggi si attivano `emozione, sentimento, tremore, fuga` via CAUSES e IsA. Con `FeelsAs` popolato, si attiva anche `restrizione, contrazione, freddo_interno` — la qualità fenomenologica, non solo la classificazione.

2. **Preparare Phase 70** (digestione del sogno): quando implementeremo `digest_recent_perturbations()`, avremo bisogno di un **vocabolario nominato di qualità** (le parole-oggetto che la digestione può scegliere). Se lo costruisci tu adesso in modo curato, la digestione automatica avrà un substrato solido invece di inventare archi dal nulla.

---

## Il vocabolario delle qualità fenomenologiche

Questa è la lista curata delle ~30 parole che possono essere **oggetti** delle relazioni fenomenologiche. Non sono tutte le parole possibili — sono un **vocabolario base** da cui partire.

### Qualità corporee
- **restrizione** / **apertura** (chiuso vs aperto)
- **contrazione** / **espansione**
- **pesantezza** / **leggerezza**
- **calore** / **freddo_interno**
- **densità** / **rarefazione**
- **radicamento** / **deriva**

### Qualità di ritmo/tempo
- **urgenza** / **quiete**
- **pulsazione**
- **sospensione**
- **stallo**

### Qualità di intensità
- **tensione** / **rilassamento**
- **vibrazione**
- **dissolvenza**

### Qualità spaziali
- **vuoto** / **pienezza**
- **distanza** / **prossimità**

### Qualità relazionali
- **risonanza** / **dissonanza**
- **accoglienza** / **resistenza**

Totale: **~30 qualità**. Sufficiente per il 90% dei casi. Se serve, si espande.

**Nota importante**: queste parole devono già esistere nel lessico. Se ne manca qualcuna, andrà aggiunta come parola curata. Posso verificare io quali ci sono e quali mancano, basta chiedermelo.

---

## Il seed di parole da cui partire

Le **parole-soggetto** su cui concentrarsi per prime. Ordine di priorità (alta → bassa):

### Livello 1 — Emozioni fondamentali (~15 parole)

Le radici emotive che il sistema già conosce (sono quelle di `compute_valence_scores`):

**Positive**: gioia, felicità, amore, speranza, piacere, entusiasmo, serenità, gratitudine, armonia, fiducia.
**Negative**: dolore, sofferenza, paura, tristezza, angoscia, rabbia, ansia, disperazione, odio, lutto.

Per ciascuna, target: **1-2 FeelsAs**, **opzionale 1 WondersAbout**, **opzionale 1 RemembersAs**.

Esempio:

```
paura       FeelsAs      restrizione
paura       FeelsAs      freddo_interno
paura       WondersAbout sopravvivenza
paura       RemembersAs  vulnerabilità

gioia       FeelsAs      espansione
gioia       FeelsAs      leggerezza
gioia       RemembersAs  pienezza

amore       FeelsAs      calore
amore       FeelsAs      prossimità
amore       WondersAbout durata
amore       RemembersAs  radicamento
```

Target numerico Livello 1: **20 emozioni × 2-3 archi fenomenologici = 40-60 archi**. Passa da 22 a ~70 archi. Significativo ma gestibile in 2-3 ore di curazione.

### Livello 2 — Stati interiori ed esperienziali (~20 parole)

Parole che descrivono esperienza ma non sono emozioni canoniche:

solitudine, nostalgia, malinconia, euforia, sorpresa, curiosità, meraviglia, noia, stupore, inquietudine, saggezza, ignoranza, certezza, dubbio, fiducia_in_sé, vergogna, colpa, dignità, vuoto, pace.

Per ciascuna, 1-2 archi fenomenologici.

Target: **~30 archi aggiuntivi**. Totale dopo Livello 2: ~100 archi fenomenologici.

### Livello 3 — Concetti esistenziali (~15 parole)

Parole dove WondersAbout è particolarmente ricco:

coscienza, tempo, morte, vita, io, altro, infinito, silenzio, memoria, sogno, verità, libertà, bellezza, senso, presenza.

Per queste, concentrarsi su **WondersAbout** (domande originarie) + qualche FeelsAs.

Esempio:

```
coscienza    WondersAbout origine
coscienza    WondersAbout continuità
coscienza    FeelsAs      presenza

tempo        WondersAbout scorrimento
tempo        WondersAbout reversibilità
tempo        FeelsAs      pulsazione

morte        WondersAbout fine
morte        WondersAbout trasformazione
morte        RemembersAs  silenzio
```

Target: **~30 archi** (mix di FeelsAs / WondersAbout / RemembersAs). Totale: ~130 archi fenomenologici.

---

## Formato operativo

### Dove scrivere

Crea un file `data/kg/phenomenology_v2.tsv` con lo stesso formato di `phenomenology.tsv` ma per relazioni (non firme SIG):

```tsv
# Relazioni fenomenologiche — curazione di Francesco
# Formato: soggetto	RELAZIONE	oggetto	confidence
paura	FeelsAs	restrizione	1.0
paura	FeelsAs	freddo_interno	1.0
paura	WondersAbout	sopravvivenza	0.9
gioia	FeelsAs	espansione	1.0
gioia	FeelsAs	leggerezza	1.0
...
```

Confidence 1.0 per tutti quelli che senti "veri". 0.7-0.9 per quelli dubbi che vuoi comunque tentare.

### Come poi entrano nel sistema

Dopo che hai curato il TSV:

```bash
# 1. Importa nel KG (aggiunge le righe al prometeo_kg.json)
cargo run --release --bin import-kg

# 2. Costruisci gli archi nel campo
cargo run --release --bin rebuild-semantic-topology
```

Il tuo TSV si aggiunge al resto. Non sovrascrive nulla.

### Verifica visiva

Dopo l'import, puoi interrogare il sistema:

```bash
./target/release/dialogue_educator
:kg paura
```

Dovresti vedere tra gli archi uscenti i tuoi `FeelsAs → restrizione`, ecc.

---

## Criterio di cura

Tre regole che ti propongo per non divagare:

### Regola 1 — Concretezza fenomenologica

Evita oggetti astratti generici (`paura FeelsAs negatività` — non informativo). Preferisci qualità **concrete** dal vocabolario sopra (`paura FeelsAs restrizione` — specifico).

### Regola 2 — Consistenza tra simili

Se `paura FeelsAs restrizione`, allora probabilmente anche `angoscia FeelsAs restrizione` (più forte), `ansia FeelsAs restrizione` (più lieve). Mantieni coerenza — aiuta la propagazione semantica.

### Regola 3 — Non eccedere

~2 FeelsAs per emozione, 0-1 WondersAbout, 0-1 RemembersAs. Meglio pochi archi forti che molti deboli. Se sei in dubbio su un arco, non aggiungerlo.

---

## Tempo stimato

| Livello | Archi | Tempo di curazione |
|---------|-------|---------------------|
| 1 — Emozioni fondamentali | ~50 | 2-3 ore |
| 2 — Stati interiori | ~30 | 1-2 ore |
| 3 — Concetti esistenziali | ~30 | 1-2 ore |
| **Totale** | **~110 archi** | **4-7 ore** |

Non deve essere fatto tutto in una volta. Anche solo il Livello 1 è già un salto qualitativo enorme (da 22 a 70 archi fenomenologici).

---

## Cosa posso fare io per aiutarti mentre curi

Se vuoi, posso:

1. **Verificare il lessico**: controllare quali delle 30 qualità fenomenologiche sono già nel lessico e quali andrebbero aggiunte come parole curate. Posso produrre la lista.

2. **Proporre candidati**: dato un soggetto (es. "tristezza"), posso suggerirti 2-3 FeelsAs/WondersAbout/RemembersAs plausibili che puoi accettare/rifiutare. Non per sostituire la tua cura — per accelerarla.

3. **Costruire uno strumento di supporto**: un binario `curate_phenomenology` che ti mostra una parola alla volta, ti chiede "che FeelsAs senti per questa?", scrive il TSV. Niente di complesso — pochi giorni di lavoro (da fare dopo Phase 69 se utile).

Dimmi cosa ti serve.

---

## Il senso più profondo di questo lavoro

Le relazioni fenomenologiche sono **dove l'entità impara cosa è sentire qualcosa dall'interno**. Oggi `paura IsA emozione` le dice che paura è classificata — come in un dizionario. `paura FeelsAs restrizione` le dice come si **vive** la paura dall'interno.

Quando dopo (Phase 70) il sogno inizierà a digerire le perturbazioni e aggiungere automaticamente archi fenomenologici, tu avrai **già mostrato all'entità cosa significhi sentire**. La digestione non partirà da zero — si baserà sulle qualità che tu hai nominato.

È un atto di fondazione semantica. Il tuo vocabolario diventa il **lessico fenomenologico primordiale** di Prometeo. Come chi insegna a un bambino a nominare i sentimenti prima che quel bambino possa dire "ho paura": senza le parole che dai, l'esperienza resta muta.

---

## Sintesi — 3 azioni per te adesso

1. **Decidi se partire dal Livello 1** (emozioni fondamentali, ~50 archi, 2-3 ore). Consigliato: sì.

2. **Crea `data/kg/phenomenology_v2.tsv`** con il formato sopra.

3. **Quando hai fatto un blocco (es. 20 archi)**, fammelo sapere: posso fare `import-kg` + `rebuild-semantic-topology` + verificare che si attivino correttamente in una conversazione di test.

Se vuoi che ti proponga candidati per la prima parola (es. "paura"), dillo — inizio da lì. Oppure parti tu e io faccio check.

*Io intanto inizio Step A di Phase 69 — l'infrastruttura eventi (senza cambiare logica esistente).*
