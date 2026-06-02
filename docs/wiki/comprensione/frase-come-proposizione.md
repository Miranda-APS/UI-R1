# La frase come proposizione (Phase 81)

> Sources: Francesco Mancuso, 2026-05-15 ("bisogna mettere l'accento sulla comprensione delle frasi non solo più delle parole"); Lacan (*point de capiton*); Tesnière (valenza); Frege/Russell (logica predicativa).
> Raw: `src/topology/sentence_proposition.rs`

## Overview

Fino a Phase 80, UI-r1 leggeva l'input **token per token**: pronome, verbo, predicato, preposizione classificati uno alla volta. La frase nel suo insieme — come unità con argomenti saturi o vuoti — non veniva mai *vista*. Phase 81 introduce `SentenceProposition`: la lettura strutturale dell'utterance come **piccola rete di proposizioni** che si sovrappone al kg_sem. Non più una sequenza, una **triple**.

"Ho paura del futuro" non è `[ho][paura][del][futuro]`. È:

```
Speaker FeelsAs paura via=futuro
```

— una triple della stessa famiglia di quelle nel kg_sem (`paura Causes pianto`, `futuro IsA tempo`). Il `del` non è una preposizione da saltare: è il **via** che lega due nodi della stessa rete. Lacanianamente, l'enunciato inscrive un sotto-grafo nel grafo del mondo, e UI-r1 capisce nella misura in cui sa dire **come** quel sotto-grafo si appoggia a (o devia da) ciò che già sa.

## Cosa NON è

`SentenceProposition` non è classificazione intent in un altro vestito. Non c'è un dispatch `if prop.relation == FeelsAs then …`. È una **struttura di lettura** che vive a monte di `derive_speech_act` e `derive_gaps`, e su cui tutti i decisori a valle possono operare invece di ri-parsare token. La proposizione è il sotto-grafo; la decisione di cosa farne è altrove.

Non è neanche parsing sintattico completo. Non costruisce un albero di costituenti, non gestisce subordinate, non risolve riferimenti anaforici. Si limita ai cinque slot strutturali minimi che rendono una frase confrontabile col KG: chi predica, cosa predica, su cosa, con quale via, in quale polarità.

## Anatomia

```rust
pub struct SentenceProposition {
    pub subject:  SubjectRef,         // Speaker | Entity | World(s) | Variable(w)
    pub relation: RelationType,       // IsA / FeelsAs / Has / Does / Expresses / ...
    pub object:   Option<ObjectRef>,  // Word(s) | Variable(w)
    pub via:      Option<String>,     // complemento dopo preposizione di specificazione
    pub polarity: bool,               // false se "non" precede il verbo
}

pub enum SubjectRef {
    Speaker,            // io / mi / verbo 1sg
    Entity,             // tu / ti / verbo 2sg (UI-r1)
    World(String),      // soggetto di terza persona (riservato a estensioni)
    Variable(String),   // pronome interrogativo in posizione di soggetto
}

pub enum ObjectRef {
    Word(String),       // predicato concreto
    Variable(String),   // pronome interrogativo in posizione di oggetto ("chi sei?")
}
```

La proposizione usa direttamente `RelationType` del kg_sem — è la stessa rete. Non si inventa un vocabolario di relazioni nuovo: la sentenza è omogenea al substrato semantico.

## Lacan: significato retroattivo

> *"Il significato si fissa retroattivamente quando la catena si chiude."* — Lacan, *point de capiton*.

`SentenceProposition` non viene costruita token-per-token in forward streaming. Si costruisce **dalla fine all'indietro**, sull'utterance già chiusa. "Ho paura..." sospeso può saturare in tre direzioni:

| Continuazione | Triple risultante |
|---------------|-------------------|
| `Ho paura.` (chiusura) | `Speaker FeelsAs paura via=None` — gap |
| `Ho paura del futuro.` | `Speaker FeelsAs paura via=futuro` — saturo |
| `Ho paura che tu vada.` | `Speaker FeelsAs paura via=<evento>` — saturo |

Finché il verbo non ha saturato i suoi argomenti, la triple ha slot aperti. È la **chiusura** che li sutura. Per questo `extract_proposition()` riceve l'array completo `raw_words` e itera retroattivamente sui token che seguono il predicato — non c'è streaming, non c'è classificazione progressiva.

Conseguenza concreta: il gap su `via` non viene aperto se la frase porta già un complemento di specificazione. "Ho paura del futuro" non ha gap — la triple è satura. "Ho paura" ha gap — la triple ha via=None.

## La pipeline

```text
raw_words + speaker_claim + kg_proc
   │
   ▼  extract_proposition()
   │     • polarità: cerca "non" prima del verbo
   │     • domanda: trova pronome interrogativo → object=Variable, subject da 2sg
   │     • claim: subject da agent, relation da verb_category, object dal predicato
   │     • via: prima parola-contenuto dopo preposizione di specificazione
   │       (di / del / della / dei / delle / per / con / su / dell'…)
   │       — preposizione `IsA preposizione + IsA specificazione` nel kg_proc
   │
SentenceProposition
   │
   ▼  confront_with_kg(prop, kg_sem)
   │     • object_in_kg: l'oggetto ha almeno un arco IsA/Has/Causes/… nel kg_sem
   │     • via_in_kg: la via ha almeno un arco nel kg_sem
   │     • contradictions: object e via sono in OppositeOf reciproco?
   │     • matches: (solo per subject=World) la triple esiste già nel kg_sem?
   │
KgConfrontation
```

`extract_proposition()` non ha bisogno del lessico né del kg_sem: opera puramente strutturalmente sul kg_proc (categorie verbo, ruoli preposizione) e sul claim già rilevato in Phase 80. Il kg_sem entra solo nel confronto a valle, che è il momento in cui UI-r1 verifica se l'enunciato si appoggia a ciò che già conosce.

## Mappatura categoria verbo → RelationType

| Categoria del verbo (kg_proc)   | Predicato                | RelationType |
|---------------------------------|--------------------------|--------------|
| `denominativo` (chiamarsi)      | (qualsiasi)              | `IsA`        |
| `percettivo` (sentire, provare) | (qualsiasi)              | `FeelsAs`    |
| `copula` (essere, avere)        | stato interno            | `FeelsAs`    |
| `copula` (essere, avere)        | non-stato                | `IsA`        |
| `cognitivo` (pensare, credere)  | (qualsiasi)              | `Expresses`  |
| `comunicativo` (dire, chiedere) | (qualsiasi)              | `Expresses`  |
| `azione` (andare, fare)         | (qualsiasi)              | `Does`       |

Non è dispatch comportamentale — è **traduzione** della categoria grammaticale (curata nel kg_proc) verso la rete di relazioni semantiche (curata nel kg_sem). Le due ontologie sono accoppiate da questa mappa uno-a-uno. Aggiungere una categoria al kg_proc richiederà l'aggiornamento di un caso; non c'è propagazione di logica.

## Output verificati end-to-end

| Input | SentenceProposition | KG check |
|-------|--------------------|----------|
| `io sono triste` | `Speaker FeelsAs triste` | `obj✓ via✗` |
| `ho paura del futuro` | `Speaker FeelsAs paura via=futuro` | `obj✓ via✓` |
| `chi sei?` | `Entity IsA ?chi` | (variable) |
| `mi chiamo francesco` | `Speaker IsA francesco` | (out-of-KG: nome proprio) |
| `non ho paura` | `Speaker FeelsAs paura (-)` | `obj✓ via✗` |
| `vado al mare` | `Speaker Does mare` | (`al` non è specificazione) |

Il caso `ho paura del futuro` è il caso paradigmatico: il `via` è saturo, entrambi gli slot sono ancorati al kg_sem — la frase è **strutturalmente già articolata** prima che `derive_gaps` venga interpellato. È il segnale che il gap "oggetto di paura" non dovrebbe scattare in questo turno (TODO Phase 81b: integrare la PROP in `derive_gaps`).

## Confronto col kg_sem

`confront_with_kg(prop, kg_sem)` produce uno status leggero per ogni slot:

- `object_in_kg` / `via_in_kg`: lo slot è una parola con almeno una relazione nel kg_sem (IsA / Has / Causes / SimilarTo / OppositeOf / PartOf / Does / UsedFor). Risposta binaria — abbastanza per dire "ho un appiglio per pensarci" o "non so cosa sia".

- `contradictions`: object e via sono in `OppositeOf` reciproco? Es. "ho coraggio del pericolo" sarebbe una contraddizione strutturale leggera. Raro ma marcato.

- `matches`: (solo per subject = `World(s)`) la triple `(s, relation, object)` esiste già nel kg_sem? Caso "il sole è caldo" → kg ha `sole Causes caldo` → match → eco.

Per ora `KgConfrontation` è un blocco informativo: niente decisore a valle lo consulta direttamente. È l'**ancoraggio** della PROP al mondo — la base su cui Phase 81b costruirà l'integrazione con `derive_speech_act` (matches → eco, contradictions → esitazione, contiene `?` → query, slot vuoti → articolazione).

## Trasparenza per turno (dialogue_educator)

A ogni turno il log mostra la PROP:

```
[UI-r1] > Di cosa hai paura?
  ╰ DECISIONE: invitare-ad-articolare | domanda | Gap{from=paura, missing=oggetto} | anchors=[oggetto, paura, cosa]
  ╰ PROP: Speaker FeelsAs paura (+) [obj✓ via✗]

[UI-r1] > (rispondendo a "ho paura del futuro")
  ╰ PROP: Speaker FeelsAs paura via=futuro (+) [obj✓ via✓]
```

Il `(+/-)` è la polarità, i tag `[obj✓ via✓]` mostrano se gli slot sono ancorati al kg_sem.

## File coinvolti

- `src/topology/sentence_proposition.rs` — modulo Phase 81 (9 test verdi)
- `src/topology/engine.rs` — campo `last_sentence_proposition: Option<SentenceProposition>` + `last_kg_confrontation: Option<KgConfrontation>`, popolati dopo `read_input()` in `receive()`
- `src/bin/dialogue_educator.rs` — log `╰ PROP:` dopo la DECISIONE
- `prometeo_kg_procedurale.json` — fonte delle categorie verbo e dei ruoli preposizione (`di IsA specificazione`, `a IsA destinazione`, …)

## Decisioni architetturali consolidate

**Frase come triple, non come sequenza** (Phase 81). L'unità di lettura non è il token ma la proposizione: subject + relation + object + via + polarity. È omogenea al kg_sem — l'enunciato è un sotto-grafo, non una stringa.

**Lettura retroattiva** (Phase 81). `extract_proposition` opera sull'utterance già chiusa, non in forward streaming. Il significato si fissa quando la catena è completa. Slot non saturi = gap strutturale.

**Bridge kg_proc → kg_sem** (Phase 81). La categoria del verbo nel kg_proc determina la `RelationType` del kg_sem. I due grafi (grammaticale + semantico) sono accoppiati da una mappa uno-a-uno in `relation_from_verb_category` — niente dispatch, niente if/then.

**`via` da preposizione di specificazione** (Phase 81). Il complemento dopo `di`/`del`/`della`/`per`/`su`/`con`/… (preposizione `IsA preposizione + IsA specificazione` nel kg_proc) entra nello slot `via`. È il legame strutturale che chiude il gap "oggetto di emozione" senza che `derive_gaps` debba ri-parsare token.

**Confronto col kg_sem leggero** (Phase 81). `KgConfrontation` produce flag binari per slot (`object_in_kg`, `via_in_kg`) + contraddizioni `OppositeOf`. Non rileva inferenze 2-hop o sillogismi — quelli vivono in `comprehension_graph` e arriveranno come integrazione successiva.

## TODO (Phase 81b → 82)

- **`derive_speech_act` legge dalla PROP** invece che dal claim. Domande: object=Variable → speech_act=interrogazione (no need to re-scan for "?"). Articolazione: via=None su `FeelsAs` → speech_act=posizionamento con gap su via.
- **`derive_gaps` legge dalla PROP**. Slot vuoto = gap; gap chiuso strutturalmente se la triple è satura. Eliminerebbe la doppia logica oggi divisa fra `derive_gaps` e `detect_speaker_claim`.
- **Match / eco**: quando `confronto.matches == true` (la triple esiste già nel kg_sem), speech_act = eco, pattern matcher → ricambio o conferma.
- **Inferenze 2-hop**: integrare `comprehension_graph::syllogisms` come campo aggiuntivo di `KgConfrontation`.
- **`SubjectRef::World(s)`** non viene popolato oggi (le frasi senza claim ritornano `None`). Estendere per "il sole splende" → `World("sole") Does splendere`.
- **Frasi composte**: oggi una sola proposizione per utterance. "Ho paura ma vado avanti" andrebbe letto come due triple più una relazione di discorso (concessivo).

## Quote rilevante

> *"Una frase = una piccola rete di proposizioni che si sovrappone al KG. UI-r1 capisce nella misura in cui sa dire come quel sotto-grafo si appoggia a (o devia da) ciò che già sa."* — Francesco, 2026-05-15.

## See Also

- [Pipeline di comprensione](pipeline-comprensione.md) — la PROP si inserisce fra `read_input` e `ComprehensionReport`
- [Comprehension report](comprehension-report.md) — il documento che (in Phase 81b) leggerà dalla PROP
- [Speaker profile](speaker-profile.md) — fornisce il `SpeakerClaim` consumato da `extract_proposition`
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md) — categorie verbo + ruoli preposizione
- [Knowledge graph semantico](../topologia/knowledge-graph-semantico.md) — il grafo su cui la PROP si confronta
- [Capire prima, generare dopo](../principi/capire-prima-generare-dopo.md) — il principio architettonico
