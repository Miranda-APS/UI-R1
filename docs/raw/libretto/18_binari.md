# Volume XVIII — Binari di manutenzione

> *Quarantadue binari in `src/bin/`. Non sono parte del runtime dell'entità — sono gli attrezzi per costruirla, curarla, testarla, misurarla. Ognuno risponde a una domanda operativa: "come si carica il KG?", "come si riderivano le firme?", "come si prepara un newborn?", "come si insegna un libro?". Questo volume è la scatola degli attrezzi annotata.*

---

## Premessa

Ogni binario è un file in `src/bin/`. Si compila e si esegue con:

```bash
cargo run --release --bin <nome> -- [argomenti]
```

Oppure (dopo build):

```bash
./target/release/<nome> [argomenti]
```

Alcuni binari modificano il `.bin` di produzione (`prometeo_topology_state.bin`) — pericolosi se non si sa cosa fanno. Altri sono sola lettura / diagnostica.

Classifico i 42 binari per **categoria di rischio e scopo**:

1. **Import / Build** (4): preparano i dati.
2. **Curation KG** (8): puliscono e modificano il grafo.
3. **Lessico management** (5): manutenzione parole.
4. **Firme 8D** (4): ricalcolo/verifica firme.
5. **Insegnamento** (6): educazione educazione + corpus.
6. **Test operativi** (6): test end-to-end.
7. **Diagnostica** (5): audit sola lettura.
8. **Server** (2): web + ai_education_server.
9. **Newborn / Migration** (2): generazione istanze + one-shot.

Totale: 42.

---

## Capitolo 1 — Import / Build (4 binari)

### 1.1 — `import-kg`

**Uso**: `cargo run --release --bin import-kg`

**Cosa fa**: legge tutti i file `.tsv` in `data/kg/*.tsv` (escludendo quelli con `.excluded`), applica lemmatization + normalizzazione accenti, costruisce un `KnowledgeGraph` completo, salva come JSON in `prometeo_kg.json`.

**Quando**:
- Dopo aver aggiunto nuovi `.tsv` in `data/kg/`
- Dopo `curate_kg.py` che modifica il JSON master? **NO** — `curate_kg.py` edita direttamente `prometeo_kg.json`. Se riesegui `import-kg`, sovrascrivi la curazione.
- In generale, `import-kg` è la *sorgente di verità* quando i TSV sono la sorgente. `curate_kg.py` è la sorgente quando il JSON è la sorgente.

**Rischio**: bassa — non tocca il `.bin` di produzione. Modifica solo `prometeo_kg.json`.

**Invariante (CLAUDE.md)**: dopo `import-kg`, serve eseguire `rebuild-semantic-topology` per applicare il KG al campo PF1.

### 1.2 — `rebuild-semantic-topology`

**Uso**: `cargo run --release --bin rebuild-semantic-topology`

**Cosa fa**: legge `prometeo_kg.json` + `prometeo_topology_state.bin`, rimuove gli archi statistici del vecchio `word_topology`, costruisce archi semantici da KG via `WordTopology::build_from_knowledge_graph` (hub damping + type_base + confidence, vol. 04). Salva il `.bin` aggiornato.

**Quando**:
- Dopo `import-kg`
- Dopo curazione del JSON
- Quando vuoi ricostruire la struttura del campo da scratch

**Rischio**: medio — modifica il `.bin` di produzione. Backup consigliato in `.bin.pre_rebuild`.

**Invariante (CLAUDE.md #9)**: topologia semantica pura — dopo questo binario, il `.bin` ha 0 archi statistici. Non ricaricare Wikipedia (contaminava il campo con vocabolario tecnico non pertinente).

### 1.3 — `import-pos`

**Uso**: `cargo run --release --bin import-pos`

**Cosa fa**: scarica e parsa il dizionario morfologico **Morph-it!** (italiano). Per ogni parola già nel lessico, assegna `pos: Some(PartOfSpeech::X)` esatta. Aggiunge ~2.775 tag POS precisi (vs heuristica da suffisso).

**Quando**: una tantum dopo aver popolato il lessico con corpora / teach-bigbang. Non va rieseguito spesso.

**Rischio**: bassa — modifica solo il campo `pos` dei `WordPattern`. Backup consigliato.

### 1.4 — `read-books`

**Uso**: `cargo run --release --bin read-books`

**Cosa fa**: legge 3 libri di letteratura italiana (hardcoded paths — da verificare nel codice) e li espone al lessico come esperienze. Ogni frase diventa una perturbazione del campo che **arricchisce il lessico esistente** (espone parole ripetutamente, aggiorna firme, non crea archi KG).

**Quando**: per arricchire il lessico con vocabolario letterario. Dopo `teach-bigbang` di solito.

**Rischio**: medio — molte perturbazioni, modifica il `.bin` significativamente. Lungo (secondi di elaborazione per libro).

**Nota (CLAUDE.md inv. #6)**: **Wikipedia rimossa** dal corpus — contaminava il campo con vocabolario tecnico. Usare solo letteratura curata.

---

## Capitolo 2 — Curation KG (8 binari)

### 2.1 — `curate-kg`

**Uso**: `cargo run --release --bin curate-kg`

**Cosa fa**: wrapper Rust di `curate_kg.py` (il file Python master di curation). Alternativa CLI quando non si vuole usare Python.

**Nota**: lo script Python `curate_kg.py` è probabilmente più usato perché più rapido da editare.

### 2.2 — `clean-kg`

**Uso**: `cargo run --release --bin clean-kg`

**Cosa fa**: rimuove archi con confidence < soglia, archi duplicati, archi con parole non valide (caratteri strani, troppo corte, inglesi).

**Rischio**: medio — modifica `prometeo_kg.json`. Backup consigliato.

### 2.3 — `clean-kg-from-invalid-words`

**Uso**: `cargo run --release --bin clean-kg-from-invalid-words`

**Cosa fa**: rimuove archi che coinvolgono parole non presenti nel lessico attuale (quindi inutili — non potranno mai attivarsi).

**Quando**: dopo `filter-lexicon` che ha rimosso parole a bassa stabilità.

### 2.4 — `nuke-kg-noise`

**Uso**: `cargo run --release --bin nuke-kg-noise`

**Cosa fa**: rimozione aggressiva di archi. Usa criteri empirici (es. archi SimilarTo tra parole con frattali completamente incompatibili).

**Rischio**: alto — nome eloquente. Backup essenziale prima di eseguire.

### 2.5 — `clean-cooccurrences`

**Uso**: `cargo run --release --bin clean-cooccurrences`

**Cosa fa**: legge il lessico, pulisce i `co_occurrences` dei `WordPattern` da entries rumorose (conteggi molto bassi, parole function, ecc.).

**Rischio**: basso — modifica solo i contatori di co-occorrenza, non le firme.

### 2.6 — `enrich-missing-relations`

**Uso**: `cargo run --release --bin enrich-missing-relations`

**Cosa fa**: scanner automatico di relazioni mancanti. Per parole nel KG senza IsA parent, suggerisce candidati via pattern matching.

**Rischio**: medio — aggiunge archi al JSON. Di solito supervisionato.

### 2.7 — `verify-kg`

**Uso**: `cargo run --release --bin verify-kg`

**Cosa fa**: audit sola lettura. Controlla consistenze (archi ciclici IsA, duplicati, orfani).

**Rischio**: nessuno — sola lettura.

### 2.8 — `analyze-edges` / `analyze-kg-completeness`

Due binari di analisi con output stampato. `analyze-edges` statistica per tipo. `analyze-kg-completeness` trova gap (parole con <N archi).

**Rischio**: nessuno — sola lettura.

---

## Capitolo 3 — Lessico management (5 binari)

### 3.1 — `clean-lexicon`

**Uso**: `cargo run --release --bin clean-lexicon`

**Cosa fa**: rimuove parole con `stability < 0.05 && exposure_count < 3`. Pulizia di parole mai veramente apprese.

**Rischio**: medio — modifica il lessico. Backup consigliato. CLAUDE.md lo elenca come comando di manutenzione standard.

### 3.2 — `filter-lexicon`

**Uso**: `cargo run --release --bin filter-lexicon`

**Cosa fa**: filtra parole per criteri configurabili (POS, stabilità minima, ecc.). Rimuove quelle fuori criteri.

**Rischio**: medio — backup.

### 3.3 — `tag-lexicon`

**Uso**: `cargo run --release --bin tag-lexicon`

**Cosa fa**: assegna POS via heuristica da suffisso (`detect_pos_from_word` di grammar.rs, vol. 13). Complementare a `import-pos` (Morph-it!) — `tag-lexicon` è euristico, `import-pos` è preciso.

### 3.4 — `fix-post-corpus`

**Uso**: `cargo run --release --bin fix-post-corpus`

**Cosa fa**: cleanup specifico dopo caricamento di corpus (Wikipedia, libri). Rimuove artefatti di tokenizzazione.

**Quando**: una tantum dopo `read-books` o corpus massivi.

### 3.5 — `diag-lexicon`

**Uso**: `cargo run --release --bin diag-lexicon`

**Cosa fa**: diagnostica — stampa statistiche (parole totali, distribuzione stabilità, distribuzione frattali dominanti, parole con POS=None).

**Rischio**: nessuno — sola lettura.

---

## Capitolo 4 — Firme 8D (4 binari)

### 4.1 — `rederive-signatures`

**Uso**: `cargo run --release --bin rederive-signatures`

**Cosa fa** (Phase 63, vol. 03): rideriva le firme 8D di tutte le parole nel KG via `derive_8d_from_kg`. Backup in `.bin.pre_p63`. Mostra campione dopo (gioia/tristezza/paura/... con le 8 dimensioni).

**Quando**:
- Dopo `import-kg` se il KG è cambiato significativamente.
- Dopo Phase 68 (ordinamento I Ching) per consolidare le firme nell'ordine corretto.

**Rischio**: medio-alto — sovrascrive 21.000+ firme. Backup automatico.

**Invariante**: richiede `prometeo_kg.json` presente. Non va eseguito prima di `import-kg`.

### 4.2 — `derive-signatures-iching`

**Uso**: `cargo run --release --bin derive-signatures-iching`

**Cosa fa**: variante di `rederive-signatures` che produce output in ordine I Ching esplicito (per audit, esporta CSV leggibile). Sola lettura del KG, non modifica il `.bin`.

**Rischio**: nessuno.

### 4.3 — `sartorial-signatures`

**Uso**: `cargo run --release --bin sartorial-signatures`

**Cosa fa**: applica le firme curate manualmente da `data/kg/phenomenology.tsv` al lessico. Complementare a `rederive-signatures` — per parole in `phenomenology.tsv`, la firma curata prevale.

**Rischio**: basso-medio.

### 4.4 — `calibrate-signatures` / `verify-signatures`

- **`calibrate-signatures`**: riaggiusta firme basandosi su criteri di calibrazione (pattern comuni nel lessico). Più specifico di `rederive-signatures`.

- **`verify-signatures`**: diagnostica sola lettura, stampa firme di parole chiave. Vol. 3 cap. 7 ha l'header aggiornato a I Ching.

### 4.5 — `export-master-signatures`

**Uso**: `cargo run --release --bin export-master-signatures`

**Cosa fa**: esporta le firme attuali in formato CSV I Ching (8 colonne nominate). Per backup, audit, condivisione.

**Rischio**: nessuno (sola lettura).

---

## Capitolo 5 — Insegnamento (6 binari)

### 5.1 — `dialogue-educator`

**Uso**: `./target/release/dialogue_educator`

**Cosa fa**: CLI interattiva con comandi speciali (`:field`, `:feelings`, `:narrative`, `:needs`, `:recall [n]`, `:recurring`, `:introspect`, `:kg <word>`, `:stats`, `:tick N`, `:witness`, `:quit`). Ogni riga non-comando viene **insegnata** (teach) e poi **ricevuta** (receive).

**Uso tipico**: sessione di addestramento + dialogo interattivo. È il binario che io (Claude) ho usato per testare nel Phase 68.

**Rischio**: medio (modifica `.bin`).

### 5.2 — `educate-interactive`

**Uso**: `cargo run --release --bin educate-interactive`

**Cosa fa**: variante di `dialogue-educator` con flusso più strutturato (passa attraverso lezioni predefinite).

### 5.3 — `educate-with-feedback`

**Uso**: `cargo run --release --bin educate-with-feedback`

**Cosa fa**: sessione educativa dove ogni lezione viene testata immediatamente (teach + receive + verifica che keywords emergano). Utile per workshop dimostrativi.

### 5.4 — `socratic-educator`

**Uso**: `cargo run --release --bin socratic-educator`

**Cosa fa**: variante di educator che *fa domande* all'entità invece di insegnargli. Testa la comprensione via Q&A.

### 5.5 — `teach-bigbang`

**Uso**: `cargo run --release --bin teach-bigbang`

**Cosa fa** (da CLAUDE.md): insegna 14.384 BigBang lessons in ~26s. Un corpus massivo di lezioni curate basate su Kaikki (cluster-based, 3 livelli).

**Quando**: una tantum su `.bin` vergine, per popolarlo del vocabolario di base.

**Rischio**: alto — enorme cambio al lessico. Backup essenziale.

### 5.6 — `teach-corpus`

**Uso**: `cargo run --release --bin teach-corpus [path]`

**Cosa fa**: teach batch da file di testo (un corpus arbitrario). Più generico di `teach-bigbang`.

---

## Capitolo 6 — Test operativi (6 binari)

### 6.1 — `chat`

**Uso**: `./target/release/prometeo`

**Cosa fa**: CLI di chat semplice. No comandi speciali. Input → receive → stampa risposta.

**Rischio**: medio (modifica `.bin` con le perturbazioni).

### 6.2 — `conversation-test`

**Uso**: `cargo run --release --bin conversation-test`

**Cosa fa**: esegue una conversazione preregistrata (hardcoded nel codice) e verifica che gli output soddisfino criteri (non vuoti, contengono keywords attese, ecc.).

**Uso**: test di regressione manuale.

### 6.3 — `sense-test` / `dream-test`

- **`sense-test`**: testa il ciclo completo sense → deliberate → compose.
- **`dream-test`**: testa il DreamEngine — esegue 100 tick autonomi, verifica le transizioni di fase.

Entrambi test end-to-end con assertion. Utili per CI (non automatizzato).

### 6.4 — `test-grammar` / `test-reading`

- **`test-grammar`**: testa coniugazioni, lemmatization, articoli. Assertion hardcoded.
- **`test-reading`**: testa l'input_reading (classificazione input via IsA chain).

Entrambi come unit test ma in binari separati (per isolamento).

---

## Capitolo 7 — Diagnostica (5 binari)

### 7.1 — `analyze`

**Uso**: `cargo run --release --bin analyze`

**Cosa fa**: dump di stato generale — lessico size, KG size, frattali popolati, top-parole per stabilità, ecc.

### 7.2 — `kg-explorer`

**Uso**: `cargo run --release --bin kg-explorer`

**Cosa fa**: CLI per esplorare il KG. Query come:
- `parent <word>`: trova parent IsA
- `children <word>`: figli IsA
- `causes <word>`: CAUSES outgoing
- `connected <w1> <w2>`: cammino tra due parole

Interattivo. Rischio: nessuno (sola lettura).

### 7.3 — `analyze-edges`, `analyze-kg-completeness`, `verify-kg`

Già citati in cap. 2.

### 7.4 — `diag-lexicon`, `verify-signatures`

Già citati in cap. 3, 4.

---

## Capitolo 8 — Server (2 binari)

### 8.1 — `prometeo-web`

**Uso**: `cargo run --release --bin prometeo-web --features web`

**Cosa fa**: server HTTP su porta 8080 (default) con tutti gli endpoint del vol. 16.

**Nota**: richiede `--features web` perché il web è gated.

### 8.2 — `ai-education-server`

**Uso**: `cargo run --release --bin ai-education-server`

**Cosa fa**: server HTTP specializzato per sessioni educative multi-utente. Espone endpoint diversi (non gli stessi di `prometeo-web`) ottimizzati per Q&A strutturato.

**Uso**: workshop, scuole. Storico — parzialmente superato da `community/` UI di `prometeo-web`.

---

## Capitolo 9 — Newborn / Migration (2 binari)

### 9.1 — `create-newborn`

**Uso**: `cargo run --release --bin create-newborn -- --name <nome>`

**Cosa fa** (vol. 17 cap. 2.4): crea un `.bin` derivato da una sessione community. Ingredienti:
- Lezioni della sessione (da `community_lessons.txt`)
- Archi KG proposti (da `community_kg.tsv`)
- Narrativa della sessione
- Identità iniziale customizzabile

Output: `<nome>_prometeo.bin`.

**Uso**: workshop educativi — al termine, ogni gruppo esce con la propria "newborn".

### 9.2 — `migrate-ordering-iching`

**Uso**: `cargo run --release --bin migrate-ordering-iching`

**Cosa fa** (Phase 68): permuta le firme 8D dall'ordine legacy `Dim`-enum all'ordine I Ching canonico. Backup in `.bin.pre_iching_ordering`. **ONE-SHOT** — non va rieseguito.

**Rischio**: alto per un `.bin` già migrato — scrambling delle firme. Il file include un warning esplicito.

**Quando**: una volta sola, al passaggio Phase 67 → 68. Io stesso l'ho eseguito nel turno del refactor.

---

## Capitolo 10 — Pipeline tipiche

### 10.1 — Bootstrap completo da zero

```bash
# Prepara corpus TSV (manualmente o via script Python esterni)
# ...

# 1. Importa KG
cargo run --release --bin import-kg

# 2. Costruisci topologia semantica
cargo run --release --bin rebuild-semantic-topology

# 3. Insegna BigBang
cargo run --release --bin teach-bigbang

# 4. Import POS (Morph-it)
cargo run --release --bin import-pos

# 5. Rideriva firme da KG
cargo run --release --bin rederive-signatures

# 6. Leggi libri
cargo run --release --bin read-books

# 7. Finale: diag + verify
cargo run --release --bin diag-lexicon
cargo run --release --bin verify-kg
cargo run --release --bin verify-signatures

# 8. Serve
cargo run --release --bin prometeo-web --features web
```

~30 minuti totali per il bootstrap.

### 10.2 — Cura KG dopo modifiche manuali

```bash
# Edit prometeo_kg.json (o via UI /curazione)
python curate_kg.py --dry-run   # preview
python curate_kg.py              # applica

# Ricostruisci archi campo
cargo run --release --bin rebuild-semantic-topology

# Rideriva firme con il nuovo KG
cargo run --release --bin rederive-signatures

# Verifica
cargo run --release --bin verify-signatures
```

### 10.3 — Workshop newborn

```bash
# Avvia server con community UI
cargo run --release --bin prometeo-web --features web

# Durante workshop: utenti usano /community
# ...

# A fine workshop, esporta sessione
curl http://localhost:8080/api/community/session > sessione.json

# Crea newborn
cargo run --release --bin create-newborn -- --name workshop_2026_04_17

# L'output è workshop_2026_04_17_prometeo.bin — copiare dove serve
cp workshop_2026_04_17_prometeo.bin cartella_workshop/prometeo_topology_state.bin
```

### 10.4 — Migration Phase 68 (una tantum)

Come eseguito nel refactor (vol. appunti.md):

```bash
# 1. Migrazione .bin firme → I Ching order
cargo run --release --bin migrate-ordering-iching

# 2. Refactor codice (enum Dim, DRIVE_DIM, syntax_center, ...)
# 3. cargo check + cargo test

# 4. Rederive per consolidare (sovrascrive per parole in KG)
cargo run --release --bin rederive-signatures

# 5. Dialogue test
printf "ciao\nchi sei?\nho paura\n:quit\n" | ./target/release/dialogue_educator
```

---

## Capitolo 11 — Rischio e backup

Regola d'oro: **prima di qualsiasi binario che modifica `.bin`, fare backup**.

```bash
cp prometeo_topology_state.bin prometeo_topology_state.bin.backup_$(date +%Y%m%d_%H%M%S)
```

I binari di maggior rischio hanno backup automatico (vedi `.pre_p63`, `.pre_iching_ordering`). Ma non tutti.

Binari **sola lettura** (rischio zero): `diag-lexicon`, `analyze`, `kg-explorer`, `verify-kg`, `verify-signatures`, `analyze-edges`, `analyze-kg-completeness`, `export-master-signatures`.

Binari **basso rischio** (modificano solo KG JSON, non `.bin`): `import-kg`, `clean-kg`, `nuke-kg-noise`, `enrich-missing-relations`.

Binari **medio rischio** (modificano `.bin`): `rebuild-semantic-topology`, `clean-lexicon`, `filter-lexicon`, `tag-lexicon`, `import-pos`, `fix-post-corpus`, `clean-cooccurrences`, `sartorial-signatures`.

Binari **alto rischio** (modifiche massive al `.bin`): `teach-bigbang`, `read-books`, `rederive-signatures`, `migrate-ordering-iching`, `calibrate-signatures`.

---

## Capitolo 12 — Cosa semplificare

Osservazione: 42 binari sono molti. Alcuni sono sovrapposti o obsoleti. Proposte di razionalizzazione:

### 12.1 — Consolidare test

`sense-test`, `dream-test`, `test-grammar`, `test-reading`, `conversation-test` → potrebbero essere unificati in un unico `diagnostic-tests` con flag `--what=sense|dream|grammar|reading|conversation`.

### 12.2 — Consolidare curation

`clean-kg`, `clean-kg-from-invalid-words`, `nuke-kg-noise`, `clean-cooccurrences` → un solo `clean-kg` con flag `--mode=gentle|aggressive|only-invalid-words|cooccurrences`.

### 12.3 — Consolidare education

`dialogue-educator`, `educate-interactive`, `educate-with-feedback`, `socratic-educator` → oggi hanno differenze sottili. Un `educator` unico con modalità selezionabili sarebbe più chiaro.

### 12.4 — Rimuovere dead/legacy

- `ai-education-server` — superato da `community/` UI di `prometeo-web`?
- `fix-post-corpus` — se Wikipedia non è più usata (inv. #6), probabilmente inutile.

Da verificare con Francesco prima di rimuovere.

---

## Capitolo 13 — Superficie pubblica (tutti binari)

Sono già tutti "pub" nel senso che sono eseguibili. La proposta di aggiunta riguarda meta-tooling:

### 13.1 — `cargo xtask`

Usare il pattern `cargo xtask` per orchestrare le pipeline tipiche:

```bash
cargo xtask bootstrap       # esegue pipeline 10.1
cargo xtask rebuild-kg      # esegue pipeline 10.2
cargo xtask workshop-setup  # avvia community mode + prepara ambienti
```

Un singolo comando invece di 6-8 invocazioni. Un file `xtask/src/main.rs` orchestra.

### 13.2 — `Makefile` o `justfile`

Alternativa più semplice: un Makefile con target named:

```makefile
bootstrap: import-kg rebuild-semantic-topology teach-bigbang import-pos rederive-signatures
rebuild-kg: import-kg rebuild-semantic-topology rederive-signatures
...
```

O `justfile` (più moderno). Riduce il rischio di dimenticare passi.

### 13.3 — CI

Non esiste CI documentato. Alcuni binari (test-*, verify-*) potrebbero correre in CI al push. Proposta: GitHub Actions che esegue:
- `cargo check --release`
- `cargo test --release`
- `./target/release/verify-signatures`

Dopo ogni PR/push.

---

## Sintesi del volume

**42 binari in `src/bin/`**, classificati per rischio e scopo:

1. **Import/Build (4)**: `import-kg`, `rebuild-semantic-topology`, `import-pos`, `read-books`.
2. **Curation KG (8)**: `curate-kg`, `clean-kg` + varianti, `nuke-kg-noise`, `clean-cooccurrences`, `enrich-missing-relations`, `verify-kg`, `analyze-edges`, `analyze-kg-completeness`.
3. **Lessico (5)**: `clean-lexicon`, `filter-lexicon`, `tag-lexicon`, `fix-post-corpus`, `diag-lexicon`.
4. **Firme 8D (5)**: `rederive-signatures`, `derive-signatures-iching`, `sartorial-signatures`, `calibrate-signatures`, `verify-signatures`, `export-master-signatures`.
5. **Insegnamento (6)**: `dialogue-educator`, `educate-interactive`, `educate-with-feedback`, `socratic-educator`, `teach-bigbang`, `teach-corpus`.
6. **Test operativi (5)**: `chat`, `conversation-test`, `sense-test`, `dream-test`, `test-grammar`, `test-reading`.
7. **Diagnostica (2 oltre ai già citati)**: `analyze`, `kg-explorer`.
8. **Server (2)**: `prometeo-web`, `ai-education-server`.
9. **Newborn/Migration (2)**: `create-newborn`, `migrate-ordering-iching`.

**Pipeline tipiche**: bootstrap da zero (8 passi, ~30 min), cura KG post-modifica (4 passi), workshop newborn, migration Phase 68 (una tantum).

**Rischi classificati**: sola lettura (0 rischio), modifica JSON (basso), modifica `.bin` singolo campo (medio), modifica `.bin` massiva (alto). Binari ad alto rischio hanno backup automatico (`.pre_p63`, `.pre_iching_ordering`).

**Proposte di razionalizzazione**:
- Consolidare test in `diagnostic-tests --what=X`.
- Consolidare curation in `clean-kg --mode=X`.
- Consolidare education in `educator --mode=X`.
- Rimuovere legacy (`ai-education-server`?, `fix-post-corpus`?) dopo verifica.

**Meta-tooling** proposto: `cargo xtask` per orchestrare pipeline; `justfile` per target standard; CI GitHub Actions per check/test/verify al push.

Da qui Vol. 19 entra nella **matematica**: appendice con esempi numerici concreti di ogni formula importante.

---

*Prossimo volume: 19 — Calcoli (appendice matematica)* (in scrittura)
