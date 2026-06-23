# UI-r1 in Tsunami — la comprensione come prodotto (non il dialogo)

**UI-r1 vale come motore di COMPRENSIONE che alimenta le feature dell'app, non
come chatbot. Il dialogo è una funzione minore (in sviluppo). Il valore è la
comprensione strutturata che l'app legge e trasforma in AZIONE.**

> Genesi: Francesco (2026-06-08). "Applica solo la comprensione a Tsunami,
> immaginando cosa può fare tramite la comprensione degli input, valutando
> l'utilizzo di tool. Il dialogo è solo una delle funzioni e nemmeno la più
> importante." Complemento a valle di `comprensione_bisogno_atto.md`.

## 1. Cosa UI-r1 espone (DTO `/api/input`, oggi)

Ogni turno (`POST /api/input {text}`) ritorna — oltre a `generated_text` (la voce,
in sviluppo) — la comprensione strutturata. I campi che l'app consuma:

| Campo DTO | Cos'è | Stato |
|---|---|---|
| `need` | il BISOGNO dominante che l'input apre + classifica (`{dominant, intensity, ranked}`) | ✅ esposto |
| `propositions` | **multi-locus**: una proposizione per clausola, `is_primary`/`subordinate` | ✅ esposto |
| `sentence_proposition` | la PROP primaria: `subject · relation · object · via · polarity` | ✅ |
| `kg_confrontation` | la frase confrontata col mondo (matches / contraddizioni) | ✅ |
| `comprehension_report` | atto di parola, posizioni, vuoti, pertinenza | ✅ |
| `state.coherence_integrity` | coerenza/saturazione del campo [0,1] | ✅ (in `state`) |
| `speaker_profile` | fatti accumulati sul parlante (self_facts, gaps, open_questions) | ✅ |

L'app **non deve** rendere `generated_text` come risposta finale: deve leggere
`need` + `propositions` + `sentence_proposition` e DECIDERE una feature.

## 2. Cosa Tsunami FA con ciascun segnale (comprensione → azione)

| Comprensione | Bisogno | Feature Tsunami / TOOL |
|---|---|---|
| Più proposizioni INDIPENDENTI (`propositions` con ≥2 `!subordinate`) | `strutturare` | **Mental Inbox**: esplode il dump in N item (uno per `proposition`), li mostra come task ordinabili. Tool: `structure_dump(text) → [item]`. *Una cosa alla volta* (ADHD). |
| Emozione/stato senza oggetto (`sentence_proposition.relation=FeelsAs`, `via=null`, e `comprehension_report.gaps` non vuoto) | `capire` | **La UNA domanda che sblocca** sullo slot vuoto (non chat libera). |
| `coherence_integrity` basso / saturazione | `co-regolare` | **Avatar gocciolina**: rallenta, respira; la UI si semplifica (meno chip). È un segnale → animazione, NON parola. |
| Un fatto del parlante riaffiora (`speaker_profile.self_facts` rilevante) | `esternalizzare-memoria` | **Riemergi un fatto/intenzione** ("avevi detto che…"). Tool: `recall_fact`. Senza colpa. |
| Caduta di presenza (ritorno dopo assenza) | `ristabilire-relazione` | Al rientro: riconoscimento ("cosa è rimasto a metà?"), non notifica. |
| `sentence_proposition` con `FeelsAs`/valenza | (lettura di stato) | Tag dello stato emotivo → mood, colore dell'avatar, routing. |
| `kg_confrontation.matches` / contraddizioni | `posizionarsi` | "Questo si lega a / contraddice ciò che sai" — spunto, non verdetto. |

## 3. Il framing a TOOL (due consumatori)

La comprensione esposta serve **due** attuatori, entrambi vincolati da ciò che
UI-r1 ha capito:

1. **L'app nativa** legge il DTO e guida le feature (tabella §2) — zero LLM, zero
   chat: è logica deterministica sopra una comprensione strutturata.
2. **Una voce LLM** (via MCP `comprehend`, che inoltra lo stesso DTO) legge la
   comprensione e **chiama tool** (`structure_dump`, `recall_fact`, …). UI-r1 =
   la comprensione ancorata al grafo; l'LLM = l'attuatore fluente, ma **non può
   inventare**: agisce su `need` + `propositions` che UI-r1 ha prodotto.

> Inversione rispetto al RAG classico: non "LLM che pensa + UI-r1 come database",
> ma **UI-r1 che comprende + LLM/app come attuatore vincolato**.

## 4. Il task-assistant 12D (`analyzeTask`) — da rianimare

Oggi `analyzeTask` ritorna `null` (12D firma spettrale fittizia, inerte). Va
**rianimato nel modello nuovo**: il titolo di un task è un input da COMPRENDERE.

```
analyzeTask(titolo) = comprehend(titolo) → { need, propositions, sentence_proposition }
  → suggerimento:
     • propositions ≥2 indipendenti  → "task composto: spezzalo" (strutturare)
     • FeelsAs/ostacolo emotivo       → "c'è una resistenza qui"
     • via=null su un'azione          → "manca un pezzo: cosa/quando?"
```

Niente più 12D fittizia: il "tipo/energia/spezzetta" del task emerge dalla
comprensione reale del titolo, non da una firma inventata.

## 5. Cosa NON fare (per ora)

- **Non** lucidare `generated_text` come se fosse il prodotto: è la voce, in
  sviluppo, presentabile come funzione-in-sviluppo.
- **Non** far decidere la feature all'LLM da solo: la feature segue `need`
  (lettura del campo), non l'estro del modello.

## 6. Stato di copertura della comprensione (onesto)

Solida: emozioni dirette+via (`ho bisogno di aiuto`→FeelsAs bisogno via=aiuto),
riflessivo (`mi sento inutile`), dump multi-clausola (segmentazione su "e"),
domande, posizionamento col grounding deterministico. Crepe note (iterabili, non
d'impianto): testa=nome nei gruppi "aggettivo+nome" (`grande nostalgia`),
soggetto-Mondo + verbo d'azione (`mia moglie non mi capisce` → niente PROP),
emozione-aggettivo (`geloso`→IsA invece di FeelsAs). L'app dovrebbe degradare con
grazia quando `sentence_proposition` è `null` (comprensione parziale) — meglio
nessuna azione che un'azione su comprensione sbagliata.

## 7. Riferimenti
- `comprensione_bisogno_atto.md` — la tipologia dei bisogni (la sorgente di §2).
- `chunker_clausa_aware_design.md` — da dove vengono le `propositions` multi-locus.
- memoria: [[project-comprensione-bisogno-atto]], [[project-tsunami-adhd-app]].
