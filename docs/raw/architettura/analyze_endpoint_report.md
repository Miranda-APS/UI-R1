# Report per l'agente — Modalità Analisi / Osservatore (UI-r1)

> Risposta alla tua nota su `comprehend` per l'analisi di transcript di terzi.
> TL;DR: la modalità che chiedevi **esiste già** (read-only, no "io sono il
> destinatario") e ora hai anche un **endpoint batch** + un **tool MCP dedicato**.
> Data: 2026-06-20.

---

## 1. Il malinteso di fondo (e l'unblock immediato)

Stavi usando il **tool MCP `comprehend`**, che mappa su `POST /api/input`: quello è
un **turno di dialogo REALE**. Per definizione UI-r1 si tratta da interlocutore →
`addressee = UI-r1`, `self_relevance`, "il parlante denomina UI-r1 come…", e **muta lo
stato vivo** (tick, NarrativeSelf, SpeakerProfile, PF1). Output ~113 KB. Per analizzare
testi di terzi è la cornice sbagliata — avevi ragione.

**La modalità osservatore esiste già come endpoint separato.** Due strade, entrambe
read-only e senza cornice relazionale:

| | `POST /api/comprehend` | `POST /api/analyze` (NUOVO) |
|---|---|---|
| Scopo | una frase/segmento isolato | un testo (N frasi) → N analisi + aggregato |
| Stato | **read-only** (zero mutazione) | **read-only** |
| addressee / self_relevance | **assenti** | **assenti** |
| Dimensione | ~28 KB (1 frase) | ~6 KB per 5 frasi (compatto) |
| Tool MCP | `comprehend` ❌ (è /api/input) | **`analyze`** ✅ (è /api/analyze) |

**Azione minima che ti sblocca oggi**: smetti di usare il tool MCP `comprehend`; usa il
nuovo tool MCP **`analyze`** (o direttamente `POST /api/analyze`).

---

## 2. `POST /api/analyze` — contratto

Richiesta:
```json
{ "text": "Dobbiamo decidere il budget entro venerdì. Marco preferisce il piano A al piano B. ..." }
```

Il testo viene **segmentato in frasi** (split su `. ! ? ; :` e newline; i frammenti
senza lettere sono scartati). Ogni frase è compresa in modo **stateless e compatto**.

Risposta (`AnalyzeDto`):
```json
{
  "sentence_count": 5,
  "sentences": [
    {
      "text": "Dobbiamo decidere il budget entro venerdì",
      "speech_act": { "kind": "asserzione", "is_question": false, "content_lemmas": [...] },
      "claim": {                         // il contenuto proposizionale (chi-dice-cosa)
        "subject_kind": "Speaker",       // Speaker | Entity | World | Variable
        "subject_name": "",
        "subject_surface": "noi",        // soggetto celato recuperato (pro-drop)
        "relation": "Does",              // IsA|Has|Does|FeelsAs|Expresses|Causes|...
        "verb_lemma": "decidere",
        "verb_display": "decidete",      // verbo coniugato (prospettiva di UI-r1)
        "object_kind": "Word", "object_name": "budget",
        "via": null,
        "polarity": true,                // false = frase negata
        "complements": [                 // analisi logica completa (Stadio 2)
          { "preposition": "al", "noun": "piano", "role": "termine", "relation": null }
        ]
      },
      "anchor_concepts": [               // collocazione ontologica per il tagging tematico
        { "word": "budget", "isa": ["risorsa","..."], "relations": ["serve a: progetto, ..."] }
      ],
      "inferences": [ "sole → produce calore → è energia" ],   // catene 2-hop dal KG
      "contradictions": [ ["a","b"] ]    // opposti nel grafo (tensioni)
    }
  ],
  "aggregate": {
    "speech_acts": [ ["asserzione", 4], ["interrogazione", 1] ],
    "concepts":    [ ["report", 2], ["budget", 1], ... ],   // concetti ricorrenti (top-30)
    "contradictions": [ ... ]
  }
}
```

`claim` è `null` quando la frase non porta una proposizione (saluti, frammenti).

---

## 3. Come copre le tue 5 richieste

1. **Modalità osservatore (no addressee/self_relevance)** → ✅ **fatto** (`/api/analyze` e
   `/api/comprehend` non hanno mai quella cornice).
2. **Read-only / stateless** → ✅ **fatto** (nessuna mutazione di tick/NarrativeSelf/
   SpeakerProfile/PF1).
3. **Output compatto** → ✅ **fatto** (no deliberation/dream/fractal/octalysis; ~6 KB / 5 frasi).
   Se serve ancora più snello, possiamo aggiungere `verbosity:"minimal"` per tagliare
   `anchor_concepts`/`inferences` — dimmelo.
4. **Atto di parola a livello di DISCORSO** (decisione/impegno/asserzione/domanda/proposta/
   obiezione) → ⚠️ **parziale**. Oggi `speech_act.kind` distingue `asserzione | interrogazione
   | atto-fatico | frammento | non-comprensibile`, e `claim.relation` dà `Does` (azione) /
   `FeelsAs` (attitudine) / `Expresses` (credenza/comunicazione) / `IsA` (identità). Le
   distinzioni **decisione / impegno / proposta / obiezione** richiedono analisi deontica/
   modale ("dobbiamo X", "propongo X", "non sono d'accordo") — è il prossimo lavoro vero,
   non un flag. Intanto puoi derivare un primo strato: modale `dovere`+infinito → impegno/
   azione; negazione + verbo di accordo → obiezione.
5. **Multi-speaker + batch** → batch ✅ **fatto** (`/api/analyze`). Multi-speaker (attribuire
   gli atti a parlanti diversi) → roadmap: oggi il soggetto è `Speaker/World`, non un
   parlante nominato. Se i tuoi transcript hanno già i nomi-parlante per turno, puoi passare
   una frase per turno e tenere tu l'attribuzione esterna; l'attribuzione interna arriverà.

---

## 4. Limiti di estrazione da conoscere mentre testi (onestà)

Questi NON sono bug del batch: sono limiti dell'estrazione per-frase, che il batch espone
fedelmente. Sono il materiale che ci serve dai tuoi test reali per affinare.

- 🔴 **Nome proprio letto come verbo** — il più importante per i verbali. "**Marco**
  preferisce il piano A" oggi legge `Marco`→`marcare` (falso-positivo morfologico) →
  soggetto/verbo corrotti. Qualunque nome proprio con desinenza verbo-simile (Marco,
  Italia, …) è a rischio. **Fix in arrivo** (un nome noto al grafo / maiuscolo non è mai
  verbo). Se nei tuoi dati i nomi sono frequenti, aspettati rumore qui finché non lo chiudiamo.
- **Negazione** a volte non propaga la polarità sull'intera frase.
- **Complementi di tempo/scadenza** ("entro venerdì") non ancora tipizzati come tali.
- **Segmentazione** ingenua: "ecc." o "Dr." spezzano la frase (sovra-segmentazione).

**Cosa funziona bene** (confermo i tuoi riscontri): `query_kg`/`get_concept` per vicinati
e collocazione ontologica; `claim` per chi-dice-cosa quando il soggetto non è un nome
proprio; `complements` (Stadio 2) per il secondo argomento ("preferisco X **a Y**" → Y come
*paragone*); `contradictions` per le tensioni; l'aggregato per concetti ricorrenti.

---

## 5. Cosa ci serve dai tuoi test

Mandaci, su transcript reali: (a) i casi dove `claim` esce sbagliato (specie nomi propri);
(b) quali distinzioni di atto-discorsivo ti servono di più (decisione vs impegno vs
proposta…) per prioritizzarle; (c) se `verbosity:"minimal"` ti serve davvero. Su questi
iteriamo l'estrazione — niente pezze caso-per-caso, meccanismi generali.
