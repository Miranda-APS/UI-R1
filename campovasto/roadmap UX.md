# Roadmap UX — campovasto

> Visione di flusso utente e direzioni di sviluppo. È una bussola, non un
> contratto come `regole di design.md` — si aggiorna man mano che la UX
> matura. Le direzioni qui scritte non vincolano l'ordine di esecuzione,
> solo la coerenza generale.

---

## 1. Flusso utente in tre fasi

### Fase A — campo vasto come prima vista

- All'apertura l'utente vede il **campo vasto**: la galassia di ~3.000 parole.
- Strumenti contestuali nella sidebar:
  - **`#filtri`** — filtri per *dimensione* del campo vasto (range slider sulle 8 dimensioni).
  - **`#relazioni`** — la legenda dei tipi di arco diventa **interattiva**: cliccare un tipo di relazione filtra gli archi visibili nel campo. Stessa logica dei filtri dimensione, applicata agli archi anziché ai nodi.
- L'utente esplora visivamente. Non è ancora "in dialogo" con una parola specifica.

### Fase B — selezione di una parola

- Trigger: click su un nodo nel grafo, o ricerca via `#ricerca`.
- La sidebar mostra **solo i tool legati alla parola**:
  - `#info-parola` (titolo + segnale campo + azioni)
  - `#dimensioni` ("valori della parola", 8 barre)
  - `#spinte` (radar Octalysis)
- I tool del campo (filtri, legenda relazioni) **si attenuano o scompaiono** — da decidere quale dei due (vedi §4).

### Fase C — creazione del proprio campo

- Punto di ingresso **visibile sul canvas** (non solo nella sidebar): un bottone iniziale "crea il tuo campo" che invita all'azione.
- L'utente inserisce una parola singola o una frase.
- Durante la creazione, la sidebar mostra solo i tool della parola.
- Il campo vasto si ritira, lo schermo diventa "il proprio campo" in costruzione.

---

## 2. Animazione del campo medio (frase → significati)

### Critica all'attuale

La sequenza step-by-step di apparizione delle parole **non preserva l'ordine sintattico** della frase. La comprensione del senso si perde.

### Direzione proposta

1. **Prima** la frase appare scritta in linea sul campo, ordine naturale di lettura.
2. **Poi**, per ciascuna parola, gli archi verso i significati circostanti si espandono in modo fluido ed elegante.
3. La posizione spaziale 8D delle parole è **superflua finché l'utente non clicca**: in modalità "comprensione della frase" la geometria 8D distrae; serve solo quando si esplora una parola specifica.

### Tecnologia

- **vis-network non è adatto** in questa fase: overhead fisico/layout-driven, antieconomico per 5-15 nodi animati.
- Sostituirlo con **animazione custom** stile *community* (apprezzata) — limando l'eccesso di elasticità che la rendeva "molleggiata".
- Possibile dual-stack:
  - vis-network → campo vasto (27K nodi, performance massiva)
  - animazione custom → campo medio (frase, 5-15 nodi, qualità di movimento)
- Componente da creare: `js/components/expand-animation.js` (o simile) che orchestra:
  - rendering della frase in linea
  - espansione degli archi per parola
  - eventuale handover a vis-network quando l'utente clicca per esplorare semanticamente

---

## 3. Cosa è già allineato

- Sidebar a tag semantici e h3 uniformi: visivo coerente con la "vista per fasi".
- `#filtri` come sezione permanente in vasto: pronta per Fase A.
- `body.parola-selezionata` gating: meccanismo già pronto per Fase B (basta estendere quale UI si attenua/sparisce).

---

## 4. Decisioni aperte

1. **Tool del campo durante selezione parola** (Fase B): nascondere `#filtri` + `#relazioni` o solo dimmare?
2. **Bottone "crea il tuo campo" sul canvas** (Fase C): posizione (overlay alto-sinistra? centrale invitante? footer del canvas?), stile (primario emergente o discreto)?
3. **Tecnologia animazione frase** (§2): canvas 2D, SVG, o WebGL? La community usava ____ (verificare).
4. **Curva di assestamento** dell'animazione: quanto smorzare l'elasticità rispetto a community?

---

## 5. Riferimenti

- Animazione community: `community.html` + relativi script — punto di partenza qualitativo per la nuova animazione del medio.
- `regole di design.md` §3 (cursore) e §4 (stati): vocabolario coerente per i nuovi controlli.
