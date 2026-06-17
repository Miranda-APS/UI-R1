# Legenda del KG procedurale — per chi cura `prometeo_kg_procedurale.json`

> Il KG procedurale è la **grammatica** di UI-r1: come si analizza una frase e
> come si passa da *comprensione* a *posizione* a *output*. È separato dal KG
> semantico (`prometeo_kg.json`, i fatti del mondo). Si cura **solo** via
> `curate_kg_procedurale.py` (idempotente), mai a mano sul JSON. Ogni nodo è
> **una sola parola** italiana (niente trattini/underscore/composti).

## Le relazioni (come leggerle)

| relazione | si legge | esempio |
|---|---|---|
| `IsA` | «è un / appartiene a» | `cosa IsA pronome` |
| `UsedFor` | «serve a» (+ `via` = su cosa/come) | `cosa UsedFor chiedere via=oggetto` |
| `Causes` | «produce / attiva» | `chiusura Causes restituire` |
| `Expresses` | «esprime» | `interrogativo Expresses richiesta` |
| `Has` | «ha» | `tu Has seconda` (persona) |
| `Equivalent` | «equivale a» (+ `via`) | `del Equivalent di via=il` |
| `via` | il *qualificatore*: su cosa/come/tramite cosa | — |

## I tipi di nodo (le "classi")

- **categoria / qualificatore / funzione** → metalinguaggio della classificazione
  grammaticale (pronome, articolo, preposizione, verbo, marcatore…).
- **pattern** → una forma espressiva istanziabile (articolazione, riconoscimento…).
- **percetto** → uno stato percettivo seminato dalla comprensione (saluto,
  chiusura, apertura, dissonanza, conferma…), che fa vincere un pattern per risonanza.
- **atto** → un modo di rispondere (chiedere, esplorare, confermare, elencare).

## §L — La trasformazione comprensione → posizione → output (la parte viva)

Qui sta la **grammatica generale dell'atto**: *non* frasi pronte per ogni intento
(quello richiederebbe cura infinita), ma due regole che la macchina di collasso
(Rust, `path_collapse::collapse_speaker`) applica a qualunque frase.

### Regola 1 — bisogno → atto, sul *punto* (locus)

> «Quando il bisogno di UI-r1 è X, l'atto è Y, e punta sul punto Z.»

```
capire        UsedFor chiedere    via=oggetto       # manca un pezzo → lo chiedo
posizionarsi  UsedFor esplorare   via=causa         # claim nuovo → ne chiedo il perché
riconoscere   UsedFor confermare  via=affermazione  # il mondo conferma → confermo
strutturare   UsedFor elencare    via=cose          # più cose insieme → le elenco
```

I **bisogni** sono i nomi di `need.rs` (il sistema sceglie quale è dominante dalla
forma della comprensione: un vuoto→capire, un claim nuovo→posizionarsi, una
triple già nota→riconoscere, un elenco→strutturare).

### Regola 2 — punto (locus) → parola interrogativa

> «Per chiedere del punto Z, usa questa parola.»

```
oggetto  UsedFor chiedere via=cosa     # "Di cosa hai paura?"
causa    UsedFor chiedere via=perché   # "Perché non hai voglia di lavorare?"
modo     UsedFor chiedere via=come
```

### Come aggiungere un atto nuovo

Una sola riga nella Regola 1 (e, se serve un'interrogazione nuova, una riga nella
Regola 2). **Mai** scrivere la frase: la frase la costruisce il collasso, coniugando
il verbo della frase dell'utente, applicando la deissi (l'«io» dell'utente diventa
«tu»), e usando l'interrogativo del punto. Esempio di ciò che ne esce, senza curare
nulla di specifico:

- `non ho voglia di lavorare` → posizionarsi → esplorare/causa → **"Perché non hai voglia di lavorare?"**
- `ho paura` (manca l'oggetto) → capire → chiedere/oggetto → **"Di cosa hai paura?"**

### ⚠ Attenzione al riuso di parole

Alcune parole sono già nel kg_proc con un altro senso. Es. `capire` è anche un
**verbo** (`capire IsA verbo`, `capire UsedFor esprimere via=comprensione`). Per
questo la macchina, fra i `UsedFor` di un bisogno, sceglie l'edge il cui oggetto
**`IsA atto`** (chiedere/esplorare/confermare/elencare sono taggati così). Se aggiungi
un atto nuovo, **taggalo `IsA atto`** o non verrà scelto.
