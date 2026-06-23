# Design — Curation per famiglie: lessema-radice, flessione derivata, derivazione linkata

**Ogni parola è un mini-grafo potenziale. Una parola flessa non è un nodo. Una parola derivata è un nodo a sé, linkato. La morfologia vive in noi, non in Kaikki.**

> Documento di design + **policy operativa per l'agente di curation** di `prometeo_kg.json`.
> Genesi: conversazione Francesco Mancuso × Claude, 2026-06-07, a valle di
> [comprensione_esplorativa_design](comprensione_esplorativa_design.md) (le isole
> `SimilarTo` e il gap di verbità emersi dai probe) e del principio dei minimi termini.
> Stato: proposta — la **§3 (policy) è già azionabile**; il resto è da validare col
> [Test Pre-Proposta](../../wiki/principi/test-pre-proposta.md).

---

## 0. Il principio in una riga

> **Si immagazzina un nodo per *lessema* (con le sue relazioni e la sua firma). La flessione si *genera* (grammatica). La derivazione si *linka* (`DerivesFrom`). Nulla si importa da Kaikki: la morfologia la incarniamo.**

---

## 1. Genesi: tre problemi, una visione

- **Le isole `SimilarTo`.** I probe ([comprensione_esplorativa_design](comprensione_esplorativa_design.md), App. A) hanno mostrato che gran parte del rumore del kg_sem sono `SimilarTo` che mescolano cose diverse: varianti ortografiche (`caffe`/`caffè`), forme flesse, parenti derivazionali (`pensata`/`pensarsi`/`impensierire` attorno a `pensiero`), sinonimi veri (`danaro`/`denaro`) e spazzatura dialettale (`schei`, `conquibus`, `coto`). Tutto sotto un'unica relazione debole.
- **I minimi termini.** Francesco: *"ogni parola può essere un mini-grafo potenziale da cui derivano tutte le forme a lei collegate."* Immagazzinare decine di forme flesse per verbo è dead-weight: il lemma + le regole bastano.
- **Il dubbio decisivo (il guardrail).** Francesco: *"cambiando il tempo verbale, la coniugazione, oppure usando un aggettivo rispetto a un verbo, possono cambiare le relazioni associate e la firma della parola."* **Esatto** — ed è proprio questo che separa ciò che è la *stessa* parola (flessione: relazioni/firma invariate) da ciò che è un'*altra* parola (derivazione: relazioni/firma diverse).

E un quarto: **niente Kaikki**. Le liste `SimilarTo` rumorose *vengono* da import esterni (Kaikki/agent_similar). La direzione del progetto è opposta — relazioni curate, non co-occorrenze importate. La morfologia che serve va **incarnata** (regole nel nostro motore + irregolari/semantica come dato), non ri-pescata da un dump.

---

## 2. Principio architetturale: due assi ortogonali

| | Flessione | Derivazione |
|---|---|---|
| Esempi | `pulisco/pulivo/pulirò`; `bello/bella/belli` | `pulire→pulito→pulizia`; `fame→affamato`; `sociologia→sociologo` |
| Cosa cambia | solo tratti grammaticali (tempo, persona, genere, numero) | categoria e **significato** (verbo→nome→aggettivo) |
| Relazioni | **invariate** (è lo stesso lessema) | **diverse** (è un altro lessema) |
| Firma 8D | quella del lemma (+ overlay grammaticale alla generazione) | **diversa** (posizione propria) |
| Nel KG | **NON è un nodo** → si genera | **è un nodo a sé** → si linka |

Il dubbio di Francesco è il *test di appartenenza*: se cambiando forma cambiano relazioni e firma, **è derivazione → nodo separato**; se no, **è flessione → si deriva, non si immagazzina**.

Questi due assi convivono con un terzo già esistente — l'**albero semantico-tassonomico** (le catene `IsA`: che *tipo* di cosa è). Non si fondono: `IsA` dice *cos'è*, `DerivesFrom` dice *da quale base si forma*. Le reti di formazione delle parole (DeriNet, CELEX; e **Morph-it!**, già importato, per la flessione) sono il riferimento per il secondo asse.

---

## 3. POLICY DI CURATION (azionabile — da passare all'agente)

> Per **ogni** parola e **ogni** edge del kg_sem, applica queste regole. Usa i metodi del `KnowledgeGraph`: `merge_word_into(from, into)`, `remove_word`, `remove_edge`, `add_edge`, `update_edge`.

### 3.1 — Nodi: cosa è un nodo e cosa no
- **Un nodo per lessema** (forma di citazione: infinito per i verbi, singolare maschile per i nomi/aggettivi). `pulire`, `pulito`, `pulizia` = **tre nodi**. `pulisco`, `pulirò`, `pulita` = **nessun nodo** (forme flesse).
- **NON creare** nodi per forme flesse (coniugazioni, tempi, genere/numero, plurali). Le genera la grammatica.
- **CREARE** un nodo per ogni lessema derivato, **con le sue relazioni proprie** (non copiare quelle della base: `pulire OppositeOf sporcare`, ma `pulito OppositeOf sporco`, `pulizia IsA qualità`).

### 3.2 — Gestione delle forme GIÀ ESISTENTI (le migliaia) — procedura per ogni `A SimilarTo B`
Classifica la relazione tra A e B e agisci:

1. **Variante ortografica/accentuale** (`caffe`/`caffè`, `gia`/`già`): tieni la forma corretta, `merge_word_into(variante, corretta)`, elimina l'edge.
2. **Forma flessa** (B è coniugazione/genere/plurale di A): B non è un lessema → `merge_word_into(B, lemma)` e rimuovi; la flessione la fa la grammatica.
3. **Parente derivazionale** (B deriva da A o condividono una base: `pensata`/`pensare`, `affamato`/`fame`): **sostituisci `SimilarTo` con `DerivesFrom`** (§3.3), tieni entrambi i nodi con le loro relazioni.
4. **Sinonimo vero** (lessema diverso, stesso significato: `danaro`/`denaro`, `timore`/`paura`): **tieni `SimilarTo`** (è legittimo) — sarà depriorizzato in comprensione (è rumore *solo* lì). Se più registri, scegli un canonico.
5. **Dialettale / arcaico / gergo / spazzatura** (`schei`, `conquibus`, `coto`, `tradigione`): **pota** (`remove_word`/`remove_edge`), a meno che non si voglia tenerlo come sinonimo marcato di registro.

> Euristica per pre-classificare (semi-automatica, poi giudizio): se B = A + suffisso flessivo noto → caso 2; se B = base(A) + suffisso derivazionale noto (§4) → caso 3; se edit-distance minima e differenza solo di accenti/raddoppi → caso 1.

### 3.3 — Il nuovo arco `DerivesFrom`
- Direzione: `derivato DerivesFrom base`, con **`via` = tipo di derivazione**:
  - `pulizia DerivesFrom pulire via=nominalizzazione`
  - `pulito DerivesFrom pulire via=participio`
  - `affamato DerivesFrom fame via=aggettivazione`
  - `sociologo DerivesFrom sociologia via=agentivo`
  - `abbagliante DerivesFrom abbagliare via=participio-attivo`
- **Mai `SimilarTo`** per la parentela derivazionale.
- Il derivato tiene **le sue** relazioni semantiche (è la fonte della sua firma). Il `DerivesFrom` aggiunge la navigabilità della famiglia, non sostituisce le relazioni proprie.

---

### 3.4 — Salvaguardie operative (raffinamenti dall'agente, 2026-06-07, vincolanti)

Cinque regole che irrigidiscono la §3.2 e proteggono dai buchi:

1. **Contratto lessico↔KG.** La curation tocca **solo il KG**. Il lessico (forme di superficie + firme apprese per esposizione, nel `.bin`) resta **intatto**; `lemmatize` è il ponte. Rimuovere un nodo-forma dal KG **non** orfana la sua firma: quella viene dall'esposizione, non da `rederive-signatures` (che rigenera solo i nodi-KG). L'overlay firma-lemma→forma del §2 è un **raffinamento futuro, non un prerequisito** (oggi non esiste; la comprensione raggiunge il lemma via `lemmatize`, non via firma-per-forma).
2. **Gate `lemmatize` per-forma (regola d'oro).** Si fonde una forma flessa **solo se `lemmatize(forma)` la risolve già al lemma**. Altrimenti **quarantena** (si tiene il nodo). Mai un buco (né nodo né lemmatizzazione = parola incomprensibile). Questo subordina il passo-3 (merge) al passo-2 (riconoscitore).
3. **Invariante di merge ferreo (più stretto del §3.2 caso-2).** **Non fondere MAI un nodo che porti relazioni proprie diverse da `SimilarTo`** (IsA/Has/Causes/…). Solo i nodi "muti" (solo `SimilarTo`) sono fondibili; chi ha vita semantica propria è derivazione → si linka. Protegge i participi-aggettivo (pulito/stanco/aperto/-ante/-ente).
4. **`DerivesFrom` = navigazione, non propagazione.** Non semina la spreading-activation: `build_from_knowledge_graph` lo **salta** (deciso e cablato, Phase 86), `field_boosts` non lo interroga. Resta attraversabile dal pathfinding (`comprehension_path` legge il KG). E si **costruisce il consumatore minimo (§2: `lemmatize`-risali-base) PRIMA** del giro di massa — niente `DerivesFrom` a migliaia senza un lettore (Principio 7).
5. **Volume + rete di sicurezza.** Ogni fusione di massa passa per **pending + dry-run report** ("N forme → M lemmi, di cui K con relazioni proprie → flaggate") prima dell'apply, e **ogni merge/prune nel `cura_ledger`** (reversibile uno per uno, non solo backup JSON). Le riconnessioni-cerotto `SimilarTo`-al-lemma già fatte vanno **convertite in merge** (se passano il gate #2 e l'invariante #3).

## 4. Incarnare la morfologia (niente Kaikki)

La morfologia che ci serve vive **dentro il sistema**, divisa per natura:

- **Regole REGOLARI → motore (grammatica), come "fisica del mondo".** Già esistono `lemmatize` (entrata) e `conjugate`/articoli (uscita) per la flessione. Si aggiunge una **tabella derivazionale compatta** (suffisso → tipo + categoria risultante): `-mento/-zione/-aggio`→nominalizzazione; `-tore/-ore/-ista/-logo`→agentivo; `-ato/-uto/-ito`→participio; `-ante/-ente`→participio-attivo; `-oso/-ale/-are/-ico`→aggettivazione; `-mente`→avverbio. Sono regole di *forma* (come la coniugazione) — passano il Test Pre-Proposta (forma, non trigger).
- **IRREGOLARI + SEMANTICA → dato curato nel KG.** I derivati non predicibili (forma irregolare *o* significato slittato: `pulizia`=atto *e* "pulizia etnica") sono nodi con le loro relazioni + `DerivesFrom`. La regola genera/riconosce la forma; **il contenuto è sempre dato**.
- **Kaikki/agent_similar retrocedono a "candidati da rivedere"**, non fonte di verità. Non si re-importa: si *cura* ciò che c'è e si *incarna* il resto come regola.

Conseguenza: il KG diventa **piccolo come un lessico curato, non grande come un corpus** — un nodo per lessema, le forme generate, le famiglie linkate.

---

## 5. La firma 8D dei derivati (perché restano nodi separati)

`derive_8d_from_kg` (Phase 63) calcola la firma dalla **posizione relazionale** del nodo. Quindi:
- un derivato, avendo **relazioni proprie**, ottiene **la sua firma** dalla sua posizione — non da una trasformazione lossy della base;
- collassare i derivati su un nodo solo **medierebbe firme incompatibili** (`pulire` agentivo vs `pulito` stativo vs `pulizia` astratto) — il danno che il dubbio di Francesco prevede.

Quindi: **non ereditare la firma dalla base.** Tenere i nodi separati e lasciarla emergere dalle relazioni. Il `DerivesFrom` *contribuisce* alla posizione (lega il derivato alla famiglia) ma non la determina.

---

## 6. Conseguenze a valle (comprensione ed espressione)

- **Comprensione.** `lemmatize` collassa **solo la flessione** → al nodo-lessema (la cui POS combacia con la forma in frase). Una forma **derivata** resta il suo nodo; se sconosciuta, il `DerivesFrom` (o la regola §4) permette di risalire alla base e capirla *come* "aggettivo di fame" / "nome del pulire". Risolve anche il gap di verbità visto in [comprensione_esplorativa_design](comprensione_esplorativa_design.md): un verbo non confermato può esserlo via la sua forma/famiglia.
- **Espressione.** Per la frase giusta si **naviga la famiglia** (`DerivesFrom`) per prendere la forma della categoria che serve (verbo vs nome vs aggettivo), poi la si **flette** con la grammatica. È il pezzo che serve al collasso articolato (Stadio 3 di Phase 86).

---

## 7. Rischi e questioni aperte

- **Derivazione semi-produttiva e idiosincratica.** Non tutte le forme esistono; i significati slittano. → la forma è regola, ma **il significato si cura/impara** (mai generare relazioni automaticamente).
- **Merge distruttivo.** `merge_word_into` sposta gli archi: prima di fondere una "forma flessa", verificare che non porti relazioni proprie legittime (se le porta, forse è derivazione, non flessione). Backup del JSON prima delle fusioni di massa.
- **Confine flessione/derivazione sfumoso** (participi: `pulito` è participio *e* aggettivo). Regola pratica: se la forma ha **uso aggettivale con relazioni proprie** (`pulito OppositeOf sporco`) → nodo derivato; se compare **solo** come tempo composto (`ho pulito`) → flessione (la gestisce il frame ausiliare+participio, Phase 84/86).
- **Lessico vs KG.** Il lessico (25K parole, firme apprese) può contenere forme di superficie incontrate; il KG (substrato del significato) è lessema+derivazione. Allineare è lavoro separato — qui si cura il KG.
- **Volume.** Migliaia di `SimilarTo` da rivedere: serve un giro semi-automatico (euristica §3.2) + revisione. Loggare cosa viene potato/fuso (niente tagli silenziosi).

---

## 8. Test Pre-Proposta

- **(1) Forma o trigger?** Le regole derivazionali (§4) sono *forma* (come si forma/flette una parola), non transizioni comportamentali. `DerivesFrom` è una relazione descrittiva, non un dispatch. ✓
- **(2) Numeri-magici?** Nessuna soglia di switch. L'euristica di pre-classificazione (edit-distance, suffissi) è un *aiuto alla revisione*, non un gate automatico sul significato (la decisione finale è curata). ✓ (vigilanza: non far potare/fondere all'agente in autonomia sui soli numeri — confermare i casi dubbi.)
- **(3) Spiegazione dello stato?** Perché `affamato` è un nodo e `pulirò` no? Perché il primo ha relazioni/firma proprie (derivazione), il secondo è lo stesso lessema flesso. Spiegabile in termini strutturali. ✓

---

## 9. Staging

1. **`RelationType::DerivesFrom`** in `relation.rs` (+ parsing TSV/JSON, + `via`). Additivo.
2. **Tabella derivazionale** (suffisso→tipo+categoria) nel motore morfologico (grammatica), con `lemmatize` esteso a *riconoscere la base* di una forma derivata sconosciuta.
3. **Giro di curation** sui `SimilarTo` esistenti con la procedura §3.2 (semi-automatico + revisione), loggando merge/prune. **Backup del JSON prima.**
4. **Curation dei nuovi lessemi derivati** con relazioni proprie + `DerivesFrom`.
5. **Verifica end-to-end**: una forma derivata sconosciuta in input viene capita via base; in output si sceglie la forma di categoria giusta dalla famiglia. (Si misura con `:explore`, Phase 86.)

Ogni stadio è osservabile e reversibile. Il cuore curatoriale è 3 (pulire le isole) + 4 (linkare le famiglie); 1-2 sono l'infrastruttura minima che li rende possibili.

---

## Appendice — Gli esempi di Francesco, mappati

| Input | Asse | Nel KG |
|---|---|---|
| `pulisco`, `pulivo`, `pulirò` | flessione | nessun nodo → `pulire` + grammatica |
| `pulire` | lessema | nodo (verbo): OppositeOf sporcare, Causes pulizia |
| `pulito` (agg.) | derivazione | nodo (agg.): OppositeOf sporco; `DerivesFrom pulire via=participio` |
| `pulizia` | derivazione | nodo (nome): IsA qualità/azione; `DerivesFrom pulire via=nominalizzazione` |
| `fame` → `affamato` | derivazione | due nodi; `affamato DerivesFrom fame via=aggettivazione` |
| `abbagliare` → `abbagliato` / `abbagliante` | derivazione | tre nodi; participio passivo / attivo via `DerivesFrom` |
| `sociologia` → `sociologo` | derivazione | due nodi; `sociologo DerivesFrom sociologia via=agentivo` |
| `bici` / `bicicletta` | variante (abbreviazione) | `merge_word_into(bici, bicicletta)` |
| `danaro` / `denaro` | sinonimo vero | due nodi, `SimilarTo` (depriorizzato in comprensione) |
| `schei`, `conquibus` | dialettale/gergo | potare |

---

## See Also
- [comprensione_esplorativa_design](comprensione_esplorativa_design.md) — le isole `SimilarTo` e il gap di verbità che questo risolve a monte.
- [knowledge-graph-semantico](../../wiki/topologia/knowledge-graph-semantico.md) — il KG che si cura.
- [feedback "curare ancorato al meccanismo"] + [Principio 7] — aggiungere solo ciò che serve a un meccanismo.
- Phase 63 `derive_8d_from_kg` — la firma dalla posizione relazionale (perché i derivati restano nodi).
- Morph-it! (già importato) — la flessione come dato d'avvio, da incarnare nel motore, non da re-importare.
