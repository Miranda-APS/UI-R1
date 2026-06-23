# Analisi logica completa — la grammatica vive nel kg_proc

**Capire una frase = sapere, di ogni parola, *cosa è* e *a cosa serve*. Non una
parola resta colla. La grammatica (le funzioni sintattiche) è DATO nel kg_proc;
Rust è solo il meccanismo generico che la applica.**

> Design di nucleo, Phase 86+. Genesi: Francesco (2026-06-08).
>
> *"Il sistema deve capire ogni parte dell'input e sapere a cosa quella parte
> serve, come fa un essere umano — non come colla di parole — e poi esprimersi di
> conseguenza. UI-r1 deve avere una comprensione della grammatica (nel kg
> procedurale). Non possiamo aspettarci che capisca le frasi altrimenti."*

È il [Principio 6](../../CLAUDE.md) ("Educare, non hardcodare") portato al cuore:
la grammatica è dato del kg_proc, Rust contiene meccanismi generici. Fonda il
refactor prima del codice. Complementa [`comprensione_esplorativa_design.md`]
(comprensione_esplorativa_design.md) (i cammini *dopo* il parse) e
[`comprensione_bisogno_atto.md`](comprensione_bisogno_atto.md) (l'atto *dopo* la
comprensione): senza analisi logica completa, entrambi lavorano su un parse bucato.

---

## 1. Il problema, misurato

La `SentenceProposition` (Phase 81) cattura la **tripla principale** + i
**complementi preposizionali** (Phase 86 Stadio 2, `extract_complements`). Ma NON
è analisi logica completa: classi di parole intere cadono. Misurato sul bench:

| Frase | Catturato | **Saltato** |
|---|---|---|
| "...**da quando me ne sono andato** di casa" | FeelsAs solo, via=casa | la **subordinata** ("quando", "andato") |
| "ho comprato una bicicletta **nuova**" | Does bicicletta | **"nuova"** (attributo) |
| "**stamattina** ho bevuto un caffè **freddo**" | Does caffè | **"stamattina"** (compl. tempo), **"freddo"** (attributo) |
| "mio padre è morto **l'anno scorso**" | IsA morto | **"l'anno scorso"** (compl. tempo) |

Tre classi mancano: **attributi** (aggettivo→nome), **complementi di tempo
non-preposizionali** ("stamattina", "l'anno scorso"), **subordinate**. Più le
frasi senza PROP (verbi non curati / marcatori).

## 2. Cosa il kg_proc ha già, e cosa manca

**Ha il *lessico della grammatica*** (~445 `IsA`): le CLASSI delle parole —
articolo, determinante, possessivo, dimostrativo, quantificatore, **qualificatore**
(aggettivi), preposizione (+ semplice/articolata/specificazione/origine/luogo/
compagnia/destinazione), pronome (+ personale/riflessivo/interrogativo), verbo (+
azione/cognitivo/percettivo/comunicativo/denominativo/copula/ausiliare/dativo/
modale/movimento), congiunzione, marcatore, avverbio.

**Manca la *sintassi*** — come le classi si compongono in **funzioni**
(soggetto, predicato, oggetto, attributo, complemento-di-X, circostanza,
subordinata). Oggi esiste solo `preposizione → complemento` (e in Rust). Questo è
il buco: il kg_proc sa *cosa è* ogni parola, non *a cosa serve* nella frase.

## 3. Il modello: classe → funzione (dato) + chunker generico (Rust)

> **Ogni classe porta, come dato nel kg_proc, la/le funzione/i sintattica/he che
> può servire. Un meccanismo generico in Rust raggruppa i token in sintagmi e
> assegna a ciascuno la sua funzione. Ogni token finisce in un sintagma con una
> funzione, oppure nel RESIDUO esplicito.**

### 3.1 La funzione come dato (kg_proc)

Si estende il vocabolario con triple `UsedFor` (la classe *serve a* una funzione)
e `Requires` (la funzione *si attacca a*). Esempi (DATO, non Rust):

```
aggettivo      UsedFor attributo          # un aggettivo serve da attributo
attributo      Requires nome              # …e si attacca a un nome (adiacenza)
articolo       UsedFor determinazione     # introduce un gruppo nominale
preposizione   UsedFor complemento        # introduce un complemento (tipo dalla prep)
congiunzione   UsedFor connessione
subordinante   UsedFor subordinata        # "quando/che/perché/se" → apre proposizione
avverbio       UsedFor circostanza
nome           UsedFor argomento          # soggetto o oggetto (ruolo dalla posizione)
```

Le funzioni (`attributo`, `complemento`, `subordinata`, `circostanza`,
`argomento`, `determinazione`) sono nodi del kg_proc — il **metalinguaggio**, come
già `cognitivo`/`percettivo`. Finite: poche funzioni, infinite frasi (minimi
denominatori — come 8D/codoni/RGB).

### 3.2 Il chunker generico (Rust)

Un meccanismo unico, senza regole per-parola:

1. **Tokenizza.**
2. **Classe di ogni token**: classi *chiuse* dal kg_proc (`IsA …`); classi
   *aperte* (nome/aggettivo/verbo) da morfologia + kg_sem (vedi §4).
3. **Raggruppa in sintagmi** (frame finiti): gruppo nominale
   `(determinante|articolo|possessivo)* (aggettivo)* nome (aggettivo)*`; gruppo
   preposizionale `preposizione + gruppo-nominale`; gruppo verbale
   `(ausiliare|modale)* verbo`; subordinata `subordinante + …`.
4. **Assegna la funzione** a ogni sintagma leggendo `classe UsedFor funzione` +
   attacco `funzione Requires testa` (adiacenza/dipendenza). Il *tipo* del
   complemento dalla preposizione (già `disambiguate`, Stadio 2).
5. **Copertura totale**: ogni token è in un sintagma con funzione, o nel
   **residuo**. Il residuo è l'errore da portare a zero, **misurabile**.

Il chunker NON è un parser CFG completo: è un raggruppatore a frame finiti
(come il chunking umano in sintagmi), guidato dai dati. La `SentenceProposition`
diventa la *vista-tripla* di questa analisi più ricca (soggetto+predicato+oggetto
estratti dai sintagmi argomento/verbale), con i sintagmi-funzione accanto.

## 4. Da dove viene la classe di ogni parola (il punto critico di fattibilità)

"Nessuna parola saltata" richiede di conoscere la classe di **ogni** parola.
- **Classi chiuse** (articoli, preposizioni, pronomi, congiunzioni, determinanti,
  ausiliari, avverbi-base): dal kg_proc, già curate. Affidabili.
- **Classi aperte** (nome, aggettivo, verbo): NON enumerabili. Vengono da:
  - **morfologia** (seed in `grammar.rs`): participi/aggettivi (-ato/-oso/-ale/…),
    desinenze verbali, plurali;
  - **kg_sem** (`IsA`): `sorella IsA persona` → nome; un verbo per `is_verb_concept`;
  - **fallback onesto**: classe ignota → il token va nel **residuo**, non in un
    ruolo inventato (meglio "non so che ruolo ha X" che un parse falso).

⚠️ **Realismo**: portare il residuo a zero su corpus vero richiederà di migliorare
la classificazione delle classi aperte (morfologia + curation kg_sem nome/aggettivo).
È lavoro atteso, non un blocco: il residuo lo rende *visibile e incrementale*.

## 5. Test Pre-Proposta (il filtro anti-parser-hardcoded)

1. **Forma o trigger?** La grammatica codifica *come la lingua è strutturata*
   (forma), non *quando comportarsi*. `aggettivo UsedFor attributo` è forma. ✓
   Il pericolo è scrivere regole-frase specifiche ("se token0=mio e token1=padre…")
   — quello è il parser hardcoded. **Vietato**: solo classe→funzione + composizione
   generica.
2. **Numeri-magici?** Nessuno: l'assegnazione è per classe e adiacenza, non per
   soglia. ✓
3. **Spiegazione?** Perché "freddo" è attributo? Perché è `qualificatore`, e
   `qualificatore UsedFor attributo`, adiacente al nome "caffè". Spiegabile dal
   dato, non da Rust. ✓

> **Verdetto**: dentro il filtro *a patto che* le regole restino classe→funzione
> (mai frase-specifiche) e il chunker resti generico. Il residuo esplicito è la
> guardia: se per coprire un caso serve una regola ad-hoc, è il segnale del trap.

## 6. Il percorso (misurato, uno alla volta)

1. **Strumento RESIDUO** nel bench: elenca i token non assegnati a nessuna
   funzione. "Nessuna parola saltata" diventa un numero. *(Primo passo, subito.)*
2. **Chunker generico** + funzioni-dato per le classi già note → copre gruppo
   nominale, attributo, complemento (assorbe `extract_complements`).
3. **Estendere le classi-dato**: `subordinante` (quando/che/perché/se),
   complemento di tempo non-prep (avverbi/gruppi nominali temporali).
4. **Migliorare classi aperte** (morfologia + curation nome/aggettivo) finché il
   residuo → ~0 sul corpus.
5. La `SentenceProposition` si ri-deriva dai sintagmi (vista-tripla); cammini e
   bisogno vedono ora *tutta* la frase.

## 7. Riferimenti
- [Principio 6 + Test Pre-Proposta](../../CLAUDE.md)
- [feedback: i 4 principi-cardine — minimi denominatori](../../) (frame finiti → frasi infinite)
- [comprensione_esplorativa_design.md](comprensione_esplorativa_design.md) — i cammini *dopo* il parse
- [comprensione_bisogno_atto.md](comprensione_bisogno_atto.md) — l'atto *dopo* la comprensione
