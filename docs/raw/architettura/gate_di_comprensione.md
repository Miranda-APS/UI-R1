# Il Gate di Comprensione — visione cristallizzata

> Genesi: Francesco Mancuso × Fable, 2026-06-15. Riformula il nucleo del
> progetto: *prima dimostrare di aver compreso, poi (forse) reagire*.
> Vincolante. Ancora: [posizionamento-teorico](../../wiki/principi/posizionamento-teorico.md),
> [capire-prima-generare-dopo](../../wiki/principi/capire-prima-generare-dopo.md).

## La tesi

La comprensione **non è uno stato nascosto che si rivendica. È un artefatto che
si esibisce.** Un LLM produce output fluente ma non può mostrarti, slot per
slot, *cosa ha legato e cosa è rimasto sciolto* di un input. UI-r1 sì. Esibire
la comprensione come oggetto ispezionabile, separato dall'output, è il
contributo originale — ed è falsificabile.

Corollario operativo: **l'output reattivo non si tocca finché il gate di
comprensione non è solido.** L'output è solo l'espressione di una comprensione
che esiste già e che si può dimostrare indipendentemente.

## Cosa significa "comprendere" (definizione operativa)

Metafora-criterio (la "soluzione" chimica): i **sali** sono i legami tipati che
l'input chiama in causa; il **solvente** è la struttura della frase; la
**soluzione** è satura quando ogni elemento è disciolto o esplicitamente
precipitato (i punti ciechi resi visibili). Una comprensione è **satura**
quando valgono quattro condizioni:

- **C1 — Copertura**: *ogni* elemento dell'input ha uno stato. Nessun token
  scompare in silenzio.
- **C2 — Struttura**: la proposizione ha tutti gli slot che la sua relazione
  esige, o determinati o *esplicitamente* marcati mancanti.
- **C3 — Ancoraggio**: ogni elemento "verde" è legato al grafo (che cos'è,
  cosa fa, cosa richiede).
- **C4 — Autocoscienza del limite**: ciò che è giallo/rosso è dichiarato *col
  perché* (manca lo slot X / parola sconosciuta / nessun ancoraggio).

Quando C1–C4 reggono, il sistema può dire onestamente *"ho compreso questo
input"* — pienamente, parzialmente, o *"non posso comprenderlo con ciò che so"*.

### Il chiarimento anti-illusione

**"Nessun punto cieco" ≠ "tutto verde".** Completezza di *copertura* (ogni
token ha uno stato) è raggiungibile ed è il requisito ferreo. Completezza di
*comprensione* (tutto verde) non è sempre raggiungibile e non è l'obiettivo. Il
trionfo è **zero punti ciechi**, non *tutto compreso*. Un sistema che dice
correttamente *"questa parola non la conosco"* ha zero punti ciechi. Inseguire
il 100% di verde porta a fingere comprensione (i nuclei già tagliati); inseguire
il 100% di copertura onesta è la dimostrazione colossale.

### I tre stati di ogni elemento

- ✅ **Compreso pienamente** — legato alla struttura: ruolo determinato,
  ancorato al KG, slot saturi.
- 🟡 **Compreso parzialmente** — riconosciuto ma con gap aperti (emozione senza
  oggetto; parola nel lessico ma non nel KG = nota-come-parola, non ancorata).
- 🔴 **Non comprensibile con la conoscenza attuale** — non legabile: parola
  ignota, nessun ancoraggio, nessun ruolo. NON è un fallimento da nascondere: è
  un output di prima classe. *"Non so cosa sia X."*

### I due assi (operazionalizzano "sapere ≠ comprendere")

Indipendenti, mostrati affiancati:

- **Conoscenza** — cosa UI-r1 sa *in generale* (X è nel KG, ne conosce classe,
  cause, opposti).
- **Comprensione** — cosa UI-r1 ha *legato per QUESTO input* (il ruolo di X qui,
  la saturazione della proposizione qui).

Una parola può essere **conosciuta ma non compresa qui**: so tutto di "paura",
ma in *"ho paura"* ho compreso che la provi tu e **non ho compreso di cosa**.

### Umiltà epistemica strutturale

*Ciò che il sistema sa non è la realtà oggettiva.* Ogni fatto legato porta:
**confidenza**, **provenienza** (curato/derivato/inferito), e l'etichetta
implicita *"posizione del MIO grafo, rivedibile"*. La comprensione di un input è
una **posizione fallibile**, revisionabile quando UI-r1 impara di più. Mai
"questa è la paura"; sempre "questo è ciò che lego a 'paura', per ora".

## Il test del manuale (l'asintoto)

Comprendere a livello applicativo = legare un testo procedurale in una struttura
eseguibile. *"Gira la valvola in senso orario per aumentare la pressione"* →
`azione(girare, oggetto=valvola, modo=orario) Causes aumento(pressione)`.
Raggiungibile **per dominio, incrementalmente**, nella misura in cui le relazioni
esistono (o vengono apprese dal manuale). Falsificabile: la struttura legata
corrisponde alla procedura? L'artefatto stesso (copertura + gap) **misura la
distanza** dall'applicabilità.

## La forma dell'artefatto

Granularità (decisione Francesco 2026-06-15): **focus sulla FRASE**; la parola è
strutturale alla frase ma mostrata interamente come **link ipertestuale** che
contiene tutto ciò che si comprende di quel token.

1. **Mappa di copertura** — ogni token reso con il suo stato (verde/giallo/rosso)
   e link al dettaglio-parola; nessun token senza stato (C1).
2. **Dettaglio-parola** (nel link) — i due assi: *conoscenza* (vicinato KG,
   classe, cause, firma 8D/regione) accanto a *comprensione qui* (ruolo nella
   frase, ancoraggio attivo).
3. **Grafo della proposizione** — il sotto-grafo tipato della frase: soggetto—
   relazione—oggetto—via, i cammini di grounding agli attrattori, il confronto
   col mondo (conferma/novità/contraddizione), il tocco col sé.
4. **Come** — la frase compresa esibita come processo, non solo risultato (i
   passi della catena: lettura → proposizione → ancoraggio → gap → bisogno).
5. **Narrazione testuale** — la traduzione in parole di tutto il sopra ("Ho
   capito che: tu provi paura. La paura è un'emozione. Mi manca: l'oggetto.").
6. **Verdetto di saturazione** — pienamente / parzialmente / non-comprensibile,
   derivato da C1–C4 (rapporto di copertura, mai un numero-soglia inventato).

## Posizionamento (UI)

Il gate è una superficie **dedicata e STATELESS** (usa `/api/comprehend`: zero
effetti collaterali, si può sondare liberamente). È separato dalla chat
reattiva — separare comprensione da output è l'intera filosofia; conflarli nella
chat la annullerebbe. La chat (turno reattivo, `/api/input`, con stato) resta
come superficie distinta e, finché l'output è ruvido, secondaria/sperimentale.

Costruzione (decisione rivista 2026-06-15): **pagina dedicata e LEGGERA**, NON
una vista dentro campovasto. Campovasto è già un'app satura (~40 moduli +
vis-network 629KB): infilarci il gate sovraccaricherebbe l'unica cosa che
funziona. Il gate **riusa solo il linguaggio visivo** di campovasto (`style.css`,
font JetBrains Mono, colori/card) — NON `app.js`, NON il `view-switcher`, NON
vis-network. È quasi tutto testuale (mappa di copertura = token colorati
cliccabili + narrazione + verdetto); il grafo della proposizione è minuscolo
(~4 nodi: soggetto-relazione-oggetto-via) e si rende con un piccolo SVG inline.
Campovasto resta intoccato.
