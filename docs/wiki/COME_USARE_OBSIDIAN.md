# Come usare la wiki UI-R1 come vault Obsidian

> Guida d'uso. La wiki è prima di tutto markdown semplice (pattern Karpathy):
> funziona perfettamente leggendola in VSCode, GitHub, o un editor qualsiasi.
> Obsidian è una **lente d'uso opzionale** che aggiunge graph view, backlinks,
> ricerca full-text, e link `[[wikilink]]`. Non sostituisce nulla: arricchisce.

## 1. Aprire il vault

1. Installa [Obsidian](https://obsidian.md) (gratuito, ~150 MB, multipiattaforma).
2. Al primo avvio, clicca **"Open folder as vault"**.
3. Naviga a `c:\Users\Fra\Desktop\Prometeo\prometeo_standalone\docs\wiki\` (questa cartella).
4. Conferma. Obsidian leggerà `.obsidian/` già configurato.
5. Al boot, Obsidian apre [index.md](index.md) in preview. Da lì puoi navigare.

> **Non** aprire `UI-r!/` come vault — è il vault Obsidian vuoto di default che avevi
> dal 10 aprile. Resta lì come scratchpad personale; ignoralo.

## 2. Cosa vedi

Layout configurato in `.obsidian/workspace.json`:

```
┌────────────┬──────────────────────────────────┬──────────────┐
│ File       │                                  │ Backlinks    │
│ Explorer   │     Markdown attivo              │ Outgoing     │
│            │     (es. index.md)               │ Tags         │
│ Search     │                                  │ Outline      │
└────────────┴──────────────────────────────────┴──────────────┘
```

- **Sinistra**: file explorer + ricerca full-text.
- **Centro**: il documento aperto in modalità preview/source.
- **Destra**: chi linka questo doc (backlinks), cosa linka questo (outgoing), tag, e
  l'outline gerarchico delle sezioni `##`.

## 3. Le tre azioni più utili

### A — Graph view (la cosa più sorprendente)

`Ctrl + G` apre il grafo dei link interni — un nodo per articolo (conteggio corrente nell'[index](index.md)), colorati per topic:

| Colore | Topic |
|--------|-------|
| viola chiaro | principi |
| verde acqua | topologia |
| arancione | comprensione |
| arancione caldo | identità |
| blu | generazione |
| viola | campovasto |
| viola lavanda | interfacce (Phase 82) |

Trascina i nodi per riordinarli. Cluster naturali emergono: *principi* tira tutto verso
di sé, *comprensione* è densamente intra-collegata (Phase 71-79), *campovasto* è
relativamente isolato.

**Per cosa serve**: vedere quali articoli sono "hub" (es. `principi-inviolabili.md`,
`pipeline-comprensione.md`, `expression-compose.md`) e quali sono orfani (zero link
entranti/uscenti — candidati a essere collegati meglio o rimossi).

### B — `Ctrl + O` — quick switch

Apre un picker con fuzzy-search. Digita 3-4 lettere del nome di un concetto
(`patt`, `selfp`, `kgproc`) e ti porta direttamente all'articolo. Più veloce di
navigare nel file explorer.

### C — `Ctrl + Shift + F` — ricerca full-text

Cerca in tutto il vault. Esempi utili:

- `"pattern_matcher"` — tutti gli articoli che lo menzionano (utile per audit cross-doc)
- `path:comprensione/` — restringe alla sotto-cartella
- `tag:#stale` — se in futuro marchi articoli come "da rivedere" via tag

## 4. Wikilinks `[[...]]` vs link markdown `[](path)`

La wiki è scritta con link markdown standard (`[testo](path.md)`) perché devono
funzionare anche su GitHub. Obsidian li capisce nativamente — ci clicchi sopra
con `Ctrl + click` e ti porta al file.

**Quando aggiungi note tue**, puoi usare la sintassi `[[nome-articolo]]` (più
veloce: Obsidian autocomplete). Funzionano solo dentro Obsidian, però — su GitHub
appaiono come testo grezzo. Suggerimento: per le note "scratch" personali usa
wikilinks; per gli articoli pubblicati, link markdown.

## 5. Cosa NON salvare nel repo

`.gitignore` del progetto esclude già la parte volatile del vault:

```
docs/wiki/.obsidian/workspace.json   ← lo stato dei tab aperti
docs/wiki/.obsidian/workspaces.json
docs/wiki/.obsidian/cache
docs/wiki/.obsidian/plugins/*/data.json
```

Resta versionata la **configurazione strutturale**: `app.json`, `appearance.json`,
`core-plugins.json`, `graph.json`. Così chiunque cloni il repo apre il vault già
configurato.

## 6. Plugin opzionali consigliati (non installati di default)

Se vuoi spingerti oltre, dal panel "Settings → Community plugins":

- **Dataview**: crea tabelle automatiche da query sui file. Es.: "tutti gli articoli
  comprensione modificati negli ultimi 30 giorni". Potente per audit.
- **Excalidraw**: disegni a mano dentro la wiki. Comodo per schemi architetturali
  che PDF/PNG diagrammi non sostituiscono.
- **Templater**: template per nuove voci log (data corrente, frontmatter).
- **Git**: commit/push direttamente da Obsidian senza uscire dall'editor.

Tutti questi salvano i loro file in `.obsidian/plugins/<nome>/` — già escluso
da git via `.gitignore`.

## 7. Workflow tipico

**Quando leggi**: apri Obsidian, parti da [index.md](index.md), naviga seguendo
i link, usa il pannello destro "Backlinks" per scoprire chi cita ciò che stai
leggendo.

**Quando scrivi**: aggiungi un nuovo articolo con `Ctrl + N`, scrivilo, linka da
e a articoli esistenti (usa `[[` per autocomplete dei nomi), aggiorna [index.md](index.md)
con una riga nella tabella appropriata, e [log.md](log.md) con una voce datata.

**Quando l'AI riscrive**: l'AI (in una sessione Claude Code) lavora su markdown
grezzo — vedi i suoi diff in git come per qualunque altro file. Non c'è "magia
Obsidian": Obsidian legge gli stessi file.

## 8. Differenza con il vault `UI-r!/` (quello vecchio)

Il vault Obsidian a `prometeo_standalone/UI-r!/` esiste dal 10 aprile, è vuoto
(solo `Benvenuto.md` di default + canvas vuoto), e **non è la wiki**. Resta lì
come scratchpad personale. Per la documentazione del sistema, usa
`prometeo_standalone/docs/wiki/`.

Suggerimento: rinomina `UI-r!/` in `_scratchpad/` o cancellalo se non ti serve —
crea confusione averli entrambi.

---

*File: [docs/wiki/COME_USARE_OBSIDIAN.md](COME_USARE_OBSIDIAN.md). Aggiornato 2026-05-13.*
