# Chunker clausa-aware — l'analisi logica che comprende dal contesto

**Un enunciato non è una frase: è una o più CLAUSOLE. In ogni clausola ogni
parola ha un ruolo (predicato / argomento / attributo / circostanza / …). Dal
ruolo discende tutto: la lemmatizzazione (verbo→infinito, argomento→singolare),
la proposizione, il bisogno. Nessuna lettura cieca — la comprensione è
conseguenza della struttura.**

> Design di nucleo, Phase 86+. Genesi: Francesco (2026-06-08). Documento
> AUTOSUFFICIENTE per la nuova chat: contiene il contesto necessario a partire.

## 0. Da dove veniamo (stato attuale — leggere prima)

Catena di comprensione esistente (file reali):
- `input_reading.rs::detect_speaker_claim` → trova **il** verbo principale (via
  `lemma_of_verb` = morfologia + `is_verb_concept`) + agente + predicato. Frame:
  dativo (`mi manca X`), tempi composti (`ho lavorato`), copula.
- `sentence_proposition.rs::extract_proposition` → `SentenceProposition`
  (subject + relation + object + via + polarity + `complements`). **Una sola
  proposizione per enunciato.** `extract_complements` (Stadio 2) disambigua i
  complementi preposizionali via `prepositions::disambiguate`.
- `analisi_logica.rs` (chunker, embrionale): `attributo_indices` (modificatore
  nel gruppo nominale) + `circostanza_indices` (avverbi). Per ora alimenta solo
  lo **strumento RESIDUO** (`sentence_proposition::unaccounted_tokens`), non la PROP.
- `kg_proc` §J: la mappa **classe→funzione come DATO** (`aggettivo UsedFor
  attributo`, `preposizione UsedFor complemento`, `avverbio UsedFor circostanza`,
  `nome UsedFor argomento`, …). Le classi chiuse sono curate; le aperte
  (nome/aggettivo/verbo) si riconoscono per morfologia + kg_sem.
- `grammar.rs::lemmatize` (solo-verbi: forma→infinito+persona+tempo);
  `lemma_candidates` (verbo+nome+aggettivo, validato dal chiamante);
  `kg_validated_lemma` (interim ONESTO: lemmatizza se il KG dà una sola risposta,
  altrimenti DEFERISCE — niente trucchi, cfr. [[feedback-no-tricks-toward-reality]]).

**Limiti che questo design risolve:**
1. **Una proposizione per enunciato** — ma i dialoghi sono multi-clausola
   ("ho litigato con mia sorella **e** non mi parla **più**").
2. **Lemmatizzazione non contestuale** — "mondi" oggetto di "visto" resta "mondi"
   (non→mondo) perché la PROP non lemmatizza l'argomento per ruolo.
3. **A4**: "perché ho paura?" = domanda **+** claim; oggi la domanda ingoia il
   claim ("paura" finisce nel residuo).
4. **Il caso difficile**: "ho visto mondi possibili e **idee nuove**" (argomenti
   coordinati) vs "ho visto mondi possibili e **fatto tante cose**" (secondo
   predicato sotto l'ausiliare gappato).

## 1. Principio

> Comprendere un enunciato = **segmentarlo in clausole**, e in ogni clausola
> **assegnare a ogni token un ruolo** leggendo la grammatica-come-dato del
> kg_proc con un meccanismo generico. La lemmatizzazione, la proposizione, il
> bisogno sono CONSEGUENZE dei ruoli — mai letture cieche.

Minimi denominatori: poche funzioni (ruoli) + frame di clausola finiti → enunciati
infiniti. Niente regole frase-specifiche (= parser hardcoded, vietato).

## 2. Il modello

### 2.1 Segmentazione in clausole

Una **clausola** = un predicato + i suoi argomenti/circostanze. I confini:
- un **verbo di modo finito** apre una clausola;
- una **congiunzione** (`e/o/ma/perché/se/quando/che`) può separare clausole
  (coordinate o subordinate) — MA solo se introduce un nuovo predicato;
- la **punteggiatura** forte (`.` `;` `?` `!`) chiude.

**Caso difficile (la coordinazione):** dopo "e", la testa è un **participio/verbo**
(→ nuovo predicato, eventualmente con ausiliare gappato: "ho [visto … e fatto …]")
o un **nome** (→ argomento coordinato della clausola precedente: "… e idee nuove")?
Il discriminatore è la **classe della testa post-"e"**, riconosciuta da morfologia
+ `is_verb_concept` (`IsA azione`): participio/verbo → predicato; nome/aggettivo →
argomento. *Qui `IsA azione` serve davvero: conferma se la testa è un verbo-concetto.*
Se ambiguo e il contesto non basta → **deferire** (una clausola, segnalare l'ambiguità),
mai indovinare.

### 2.2 Ruoli per clausola

In ogni clausola, leggendo `classe UsedFor funzione` + attacco per adiacenza:
- **predicato**: il gruppo verbale (ausiliare/modale* + verbo). Frame esistenti
  (dativo/composto/copula) restano, ora *per clausola*.
- **argomento**: i gruppi nominali (soggetto/oggetto — ruolo da posizione/preposizione).
- **attributo**: aggettivo adiacente al nome-testa (mattone già fatto).
- **circostanza**: avverbi + gruppi nominali temporali (mattone parziale).
- **complemento**: gruppo preposizionale, tipo da `prepositions::disambiguate`.
- **connettivo/marcatore**: congiunzioni, "secondo me", interiezioni.
- residuo: ciò che non riceve ruolo → misurato, da portare a zero.

### 2.3 Lemmatizzazione contestuale (la conseguenza)

Dal ruolo:
- token nel **predicato** → `lemmatize` (verbale → infinito + persona/tempo);
- token **argomento/attributo** → riduzione **nominale** (plurale→singolare). E
  ora è disambigua: sappiamo che è un nome → si generano **solo candidati
  nominali**, "mondare" non è nemmeno considerato. "mondi" (oggetto) → "mondo".

→ risolve il limite 2 *senza alcun trucco*: la classe viene dal contesto, non dalla
forma. `kg_validated_lemma` diventa la riduzione nominale chiamata **solo sui token
marcati argomento/attributo**.

### 2.4 Dalla clausola alla proposizione (multi-locus)

Ogni clausola → una `SentenceProposition` (la vista-tripla del suo predicato +
argomenti). Un enunciato → `Vec<SentenceProposition>`. A4 si scioglie: "perché ho
paura?" = una clausola con predicato `FeelsAs paura` (claim) **e** marcatore
interrogativo `perché` (l'atto domanda) — coesistono come ruoli distinti, "paura"
non è più residuo. Il bisogno (`need.rs`) e il grounding (`comprehension_path`)
leggono le proposizioni per clausola.

## 3. Il caso difficile, lavorato

```
"ho visto mondi possibili e idee nuove"
  clausola unica:
    predicato = ho visto            (composto: aux ho + part. visto → vedere)
    argomento = mondi  (→ mondo, nominale perché argomento)
      attributo = possibili (→ possibile, adiacente, concorda)
    [e: coordina argomenti — la testa post-e "idee" è NOME, non verbo]
    argomento = idee   (→ idea)
      attributo = nuove  (→ nuovo)
  → PROP: Speaker Does vedere ; oggetti {mondo[possibile], idea[nuovo]}

"ho visto mondi possibili e fatto tante cose"
  due predicati sotto l'ausiliare gappato:
    clausola A: predicato = ho visto ; argomento = mondi[possibili]
    clausola B: predicato = (ho) fatto ; argomento = cose ; circostanza/attr = tante
    [e: coordina PREDICATI — la testa post-e "fatto" è PARTICIPIO/verbo (is_verb_concept)]
  → due PROP: Speaker Does vedere {mondo} ; Speaker Does fare {cosa}
```

Il discriminatore unico: **classe della testa dopo "e"** (nome vs verbo), via
morfologia + `IsA azione`. Niente euristica cieca; se davvero ambiguo, deferire.

## 4. Test Pre-Proposta (anti parser-hardcoded)

1. **Forma o trigger?** Si codifica *come* la lingua si struttura (classe→funzione,
   frame di clausola), non *quando* comportarsi. ✓
2. **Numeri-magici?** Nessuno: segmentazione e ruoli per classe + adiacenza, non
   per soglia. ✓ (vigilanza: la coordinazione NON deve diventare "se ≥N parole…").
3. **Spiegazione?** Perché "fatto" è predicato? È participio (`IsA azione`) sotto
   l'ausiliare "ho". Perché "mondi" è "mondo"? È argomento → riduzione nominale.
   Tutto spiegabile dal dato + struttura. ✓

> Guardia: nessuna regola frase-specifica. Il **RESIDUO** resta la spia — se per
> coprire un caso serve una regola ad-hoc, è il trap.

## 5. Cosa si costruisce (incrementale, misurato col RESIDUO + bench-criteri)

1. **Struttura dati** `Analisi { clausole: Vec<Clausola> }`, `Clausola { ruoli:
   Vec<(token, Funzione)>, predicato, argomenti, … }`. Additiva, ispezionabile nel bench.
2. **Segmentatore di clausole** (verbi finiti + coordinazione participi + punteggiatura).
3. **Assegnatore di ruoli per clausola** (estende attributo/circostanza con
   predicato/argomento/complemento) — copertura totale o residuo.
4. **Lemmatizzazione per ruolo** (predicato→verbale, argomento→nominale validata).
5. **`SentenceProposition` ri-derivata per clausola** (multi-locus) → la PROP non è
   più calcolata a parte ma è la vista-tripla dell'analisi.
6. Aggancio a `need.rs` (bisogno per clausola) e `comprehension_path` (grounding per PROP).

## 6. Invarianti / vincoli

- **NON toccare** `verbo IsA {azione,atto,processo,…}` (è `is_verb_concept`, load-bearing).
- Grammatica = dato nel kg_proc + meccanismo generico in Rust; **mai** liste
  frase-specifiche.
- Nuove `RelationType` (se servissero): **Rust-first** (relation.rs prima del JSON,
  trauma Phase 84).
- Additivo e reversibile: il chunker prima OSSERVA (bench), poi alimenta la PROP.
- **Mai trucchi** ([[feedback-no-tricks-toward-reality]]): nell'ambiguità si deferisce.

## 7. Riferimenti
- [analisi_logica_grammatica_kg_proc.md](analisi_logica_grammatica_kg_proc.md) — la grammatica come dato (fondamenta)
- [comprensione_esplorativa_design.md](comprensione_esplorativa_design.md) — i cammini *dopo* il parse
- [comprensione_bisogno_atto.md](comprensione_bisogno_atto.md) — l'atto *dopo* la comprensione
- memoria: [[project-analisi-logica-grammatica]], [[project-kg-sem-due-strutture]], [[feedback-no-tricks-toward-reality]]
