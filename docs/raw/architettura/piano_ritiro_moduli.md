# Piano di ritiro — moduli, pagine ed endpoint del frontend

> Per: Francesco + agente del Gate. Da: Opus (su direttiva di Francesco,
> 2026-06-15). Complementare a [piano_design_admin.md](piano_design_admin.md) e
> [gate_di_comprensione.md](gate_di_comprensione.md).
> **Scopo**: non ridisegnare ciò che c'è — *togliere*. Decidere, modulo per
> modulo, cosa sopravvive a "ciò che il sistema è diventato ora" (Fase 86: la
> comprensione è il prodotto; l'output reattivo è secondario; lo stato interno
> conta solo se è onesto).

## La lente (criterio di sopravvivenza)

Un modulo sopravvive **solo** se serve una di queste tre superfici:
1. **Gate di comprensione** — l'artefatto esibito (pagina dedicata, altro agent).
2. **campovasto** — l'esploratore del KG (canonico, design language).
3. **Stato interno onesto** — ciò che il sistema *davvero* è ora, non i resti di fasi morte.

Tutto il resto è: duplicato, morto, o instrada esperimenti dismessi. Una route
raggiungibile *afferma* che il sistema fa ancora quella cosa — se non è vero, è
un danno, non un di-più.

## Superficie finale (il target)

- `/` — home pubblica (snellita, stat dinamiche).
- **Gate** — pagina dedicata (altro agent), linkata dall'hub.
- **campovasto** — `/campovasto`, lente principale sul KG.
- **Frattali** — pagina esterna linkata (lente 3D *alternativa* sul KG, da tenere
  per un punto di vista diverso domani). NON tab dell'admin.
- **Stato interno** — pagina dedicata che unifica le facce oneste del sé.
- **Chat** — *una sola* superficie, dichiaratamente sperimentale.
- **Cura** — *uno solo* strumento.
- `/admin` — hub sottile: vitali O(1) + griglia di link. Zero rendering pesante.

Tutto ciò che non è in questa lista → ritiro.

## Tab dell'admin monolite (`src/web/index.html`, 3480 righe, inline, 3 librerie grafiche)

| Tab | Verdetto | Motivo |
|---|---|---|
| **CHAT** | **Ritira la tab; tieni 1 chat altrove** | Output reattivo = secondario (gate doc). Il `comp-panel` interno è la vecchia animazione-comprensione D3, **soppiantata dal Gate** → rimuovere. La chat resta come superficie sperimentale unica (vedi `/dialogo`). |
| **CAMPO** | **RITIRA** (decisione Francesco) | Gauge energia + lista parole attive = telemetria da dev. Il polso lo danno già i vitali dell'header; il campo lo mostra campovasto/Frattali. |
| **FRATTALI** | **TIENI → pagina esterna** (decisione Francesco) | Lente 3D sul KG (Three.js, `/api/universe`). Valida come punto di vista alternativo. MA: oggi trascina `uAnimLoop_DEAD()` (~167 righe morte) + stub canvas 2D + `vis-network` caricato-e-mai-usato → estrarre PULITA in pagina dedicata, non portarsi dietro il monolite. Assorbe `/universo` (stesso endpoint). |
| **VOLONTÀ** | **Fondi in Stato interno** (sola lettura) | `/api/will` è reale (intenzione/drive/trigger). È stato interno. Il controllo `will/focus` (mutante) è un knob da dev → cade. |
| **SOGNO** | **Ritira la tab; controlli → angolo manutenzione** | Diagramma ciclo onirico + STM/MTM/LTM + bottoni *force-dream/grow/save*. Sono controlli di manutenzione, non "da mostrare". Per giunta il metronomo 3s è in discussione (Francesco: zero timer nell'engine). I bottoni dream/grow/save/persist vivono in un angolo tecnico dell'hub, non in vetrina. |
| **NARRATIVA** | **TIENI → Stato interno** | Valenza + sé narrativo + turni. Stato interno onesto, `/api/narrative` reale. Nucleo della pagina unificata. |
| **IDENTITÀ** | **Fondi in Stato interno — tieni SOLO le incertezze, butta le innate** (decisione Francesco) | La vista oggi mostra un sé DISMESSO: `build_self_dto` legge `engine.self_model.beliefs` (convinzioni innate ★), ma il sé reale è `engine.kg_self` = 37 pendenze, **0 opinioni** (le 22 innate furono *dissolte* il 2026-06-10). Le convinzioni/valori innati NON si mostrano. La parte salvabile sono le **incertezze fondamentali** (`uncertainties`: topic + tension) — che sono *domande che UI-r1 si pone*, allineate al cuore della pagina (vedi sotto). |
| **EPISODI** | **TIENI → Stato interno** | `/api/episodes` reale, memoria episodica onesta. |
| **FORMA** (nascosta) | **RITIRA** (formalizza) | Già `display:none!important`. 16 primitivi + simplici con SVG = I Ching *esibito* → vietato user-facing ([[feedback-iching-is-internal-not-aesthetic]]). |
| **DIALOGO** | **RITIRA** | `/api/inner-dialogue` + `/api/respond` = "Dialogo Interiore" Phase 52, **superato** dalla pipeline di comprensione Phase 73+. |

### Pagina "Stato interno" — il cuore: *cosa UI-r1 non ha ancora capito*

Unifica NARRATIVA + EPISODI + VOLONTÀ(sola lettura) + le incertezze di IDENTITÀ.
Una pagina, riusa il theme di campovasto, **chiara e facilmente leggibile**.

> **L'aspetto che conta di più (direttiva Francesco): le domande che UI-r1 fa
> sui propri gap di comprensione.** È il *titolo* della pagina, non una sezione
> in fondo. Tutto il resto (valenza, turni, episodi) è contorno.

Le domande-sui-gap esistono già come dati, in **due facce** da non confondere:
- **Per-frase (live)** → `/api/comprehend` `gaps[]` = territorio del **Gate** (altro agent). Lo Stato interno NON lo replica.
- **Permanenti (accumulate nel dialogo)** → territorio di QUESTA pagina:
  - `/api/speaker` → `open_questions[]` (cosa sta chiedendo) + `open_gaps[]`
    (vuoti aperti, con `gap_kind`) + **`closed_gaps[]`** (vuoto aperto e poi
    chiuso da una parola successiva, con `closed_by` → la narrativa che si compone).
  - `/api/thoughts` → pensieri `Gap` / `MissingBridge` / `Need` (stato cognitivo vivo).
  - `/api/self` `uncertainties` → incertezze fondamentali (topic + tension) — le
    domande di IDENTITÀ, salvate dal folding.

Onestà strutturale (gate doc, C4): ogni gap è dichiarato *col perché*. Mai
inventare una domanda — si mostra solo ciò che gli organi producono. Coordinare
col Gate per non duplicare la faccia per-frase.

## Pagine e route HTML

| Pagina / route | Verdetto | Motivo |
|---|---|---|
| `/` (`biennale/home.html`) | **TIENI, snellisci** | Landing pubblica. Stat hardcoded (25.870 / 251.454) → renderle dinamiche da `/api/state` o toglierle. |
| `/campovasto` | **TIENI** (canonico) | Design language + fondamenta. |
| `/biennale`, `/campo-vasto` (`biennale/index.html`) | **RITIRA** | Clone monolitico di campovasto (1506 righe, `vis-network` da `unpkg` → rotto offline). Soppiantato. |
| `campovasto-cy/` (PoC Cytoscape) | **✅ ELIMINATO** (2026-06-15) | Directory rimossa (era untracked → rimozione definitiva). Resta da togliere la route `/campovasto-cy` (server.rs:238) nel Tier 2. |
| `/dialogo` (`biennale/dialogo.html`) | **TIENI come chat sperimentale unica** | È la chat pubblica funzionante. Assorbe la tab CHAT dell'admin. Dichiararla sperimentale. |
| `/diffrazione` (`biennale/diffrazione.html`) | **RITIRA** | Dipende da `/api/diffraction` che fa shell-out a uno script Python esterno (fragile). Esperimento fuori mappa. |
| `/ui-r1` (`biennale/uir1.html`) | **RITIRA (verifica)** | Pezzo da esposizione (demo word-detail). Il Gate è il nuovo pezzo da esposizione → ridondante. |
| `/universo` (`universo/index.html`) | **RITIRA → assorbito da Frattali** | Vecchia viz di `/api/universe`, stessa cosa della tab FRATTALI. |
| `/community` + `campovasto/community.html` | **NON TOCCARE per ora** (decisione Francesco) | Doppia UI community, consolidamento rinviato. |
| `/curazione` (`biennale/curazione.html`) | **NON TOCCARE per ora** (decisione Francesco) | Strumento di cura, funziona. Rinviato. |
| `/cura-mobile/` (PWA) | **NON TOCCARE per ora** (decisione Francesco) | PWA offline. Rinviato. |

## Endpoint dismessi ancora instradati (Tier 2 — coordinare con backend)

Codice reale, ma fuori dalla mappa Fase 86. Ogni route raggiungibile lascia
intendere che il sistema faccia ancora quella cosa. Da ritirare *dopo* il
frontend, in coordinamento (richiede ricompilazione Rust; non pestare il Gate):

- `/api/diffraction` (+ `/diffrazione`) — shell-out Python, fragile. **Primo a cadere.**
- Sonde topologiche Phase-67: `/api/navigate`, `/api/phase`, `/api/tension`, `/api/projection`.
- Introspezione vecchia: `/api/why`, `/api/ask`, `/api/open-questions`, `/api/clarity`, `/api/thought-chain`.
- Dialogo interiore Phase 52: `/api/inner-dialogue`, `/api/respond`.
- Altre sonde: `/api/generate`, `/api/introspect`, `/api/compounds`, `/api/locus-simulate`, `/api/simpdb`, `/api/topology`.
- `/api/universe` — **TENERE** finché la pagina Frattali esterna lo consuma.

Resta vivo (Fase 86): `/api/comprehend`, `/api/state`, `/api/wordfield`,
`/api/visuals`, `/api/narrative`, `/api/self` (da ri-fondare), `/api/episodes`,
`/api/will`, `/api/thoughts`, `/api/concept`, `/api/speaker`, `/api/biennale/*`,
`/api/cura/*`, `/ws`.

## Ordine di esecuzione

**Tier 1 — frontend puro (sicuro, niente ricompilazione; campovasto è servito via `ServeDir`)**
1. Hub admin sottile: vitali + griglia di link (Gate, campovasto, Frattali, Stato interno, Chat, Cura).
2. Estrai **Frattali** pulita in pagina dedicata (taglia codice morto, una libreria sola); assorbi `/universo`.
3. Costruisci **Stato interno**: titolo = le domande sui gap (`/api/speaker`
   open_questions/open_gaps/closed_gaps + `/api/thoughts` Gap; incertezze da
   `/api/self`). Contorno: narrativa + volontà-ro + episodi. IDENTITÀ folde qui
   *senza* le convinzioni innate.
4. Ritira pagine: `/biennale`, `/diffrazione`, `/ui-r1`, `/universo`. (community
   e cura: NON toccare per ora.)
5. Home: stat dinamiche.

**Tier 2 — backend (coordinato, ricompilazione)**
6. Togli la route `/campovasto-cy` (dir già eliminata).
7. `/api/self`: smetti di esporre le convinzioni innate; tieni le `uncertainties`.
8. Ritira le route dismesse (lista sopra), `/api/diffraction` per primo.

**Nota di disciplina**: nessuna cancellazione "alla cieca". Ogni `RITIRA` qui è
motivato da evidenza (clone / endpoint fragile / fase superata / sé dismesso).
Prima di rimuovere file, conferma che nessuna pagina-tenuta vi linki.

---

## Critiche dello staff (2026-06-15) — triage

Lista di critiche sulla vecchia versione. Legenda: ✅ risolta · 🟡 parziale ·
⬜ aperta · 🔁 risolta-per-ritiro · 💬 da discutere.

### ADMIN (in gran parte indirizzato dal redesign)
- **Rename "prometeo" → UI-r1 anche negli artefatti** (clash col progetto di
  Saracco): 🟡 UI nuova già "UI-r1"; binari/`.bin`/`.json` ancora "prometeo" →
  refactor backend+path, **APERTO**.
- CAMPO "non si capisce come si attivano le parole": 🔁 CAMPO esce; il "come"
  della comprensione lo mostra il Gate.
- Una frase per tab: ✅ ogni card dell'hub ha la riga di spiegazione.
- FRATTALI barra stretta / "simile a" sovrapposto / click impreciso: 🟡
  pannello allargato (200–260px) + wrap righe relazione **fatto**; click su 25k
  punti resta impreciso (raycaster) → la ricerca per nome è l'accesso affidabile.
- Pesi "festa" diversi frattali/campovasto: ✅ **firma 8D identica** (verificato);
  era confusione affinità/stabilità vs firma. Da etichettare in UI.
- SOGNO spiegare cicli: 🔁 tab ritirata; tooltip sui controlli manutenzione **fatto**.
- NARRATIVA tooltip su tono/intensità: 🟡 tooltip su turno/postura/tono in
  Stato interno **fatti**; il resto delle metriche segue col folding.
- IDENTITÀ ↔ "chi sei?": 💬 allineato — il sé reale è `kg_self` (pendenze +
  opinioni guadagnate), non le innate dismesse. Stato interno mostrerà
  domande + opinioni guadagnate, stessa sorgente di "chi sei?".
- EPISODI "non capito": 🔁 demoto a contorno; valutare se spiegare o togliere.
- DIALOGO conferma/nega/elabora + "educare come staff": 💬 il modulo
  (inner-dialogue Phase 52) è ritirato; l'idea staff-education è buona e va sul
  path di teaching attuale (cura + community + validazione opinioni `kg_self`).

### CAMPOVASTO — DA FARE (prossimo cantiere, non ancora toccato)
1. ⬜ **Nome relazione a metà dell'arco** (`buildEdgeSpec` in node-style.js:
   oggi gli archi hanno `label:''`).
2. ⬜ **Frase di conferma alla creazione di un arco** ("bocca è parte di
   faccia") — il render esiste (`path_collapse`), agganciarlo al flusso di
   edge-creation (campovasto/cura; NON nel viewer frattali, lì l'edit è tolto).
3. ⬜ **Modale relazioni: direzione chiara** (freccia "è causa di / è causato
   da" poco visibile).
4. ⬜ **Modale relazioni: ordine leggibile** soggetto→relazione→oggetto→tramite
   (oggi "Curiosità→mistero→causa→tramite" → "Curiosità→causa→mistero→tramite").
5. 🟡 **Header-nav in campovasto con le sezioni dell'admin + spiegazioni**
   (onboarding contributori) — l'hub con le descrizioni esiste già, va linkato
   dall'header di campovasto.
6. 💬 **"relazione" → nodi-tipo / frasi**: separare (a) comporre "X è parte di
   Y" (già possibile) da (b) rendere "relazione" un hub di tipi-relazione
   (meta-linguistico — meglio come legenda/palette UI che come triple nel
   kg_sem, per non violare "una parola = un nodo").

**Disciplina viste**: hub/stato-interno/frattali importano SOLO i token di
design (`theme.js`/font) da `/campovasto` via path assoluto. Niente logica
condivisa. Per simmetria totale futura: estrarre `theme.js` in cartella neutra.

---

## Stato di esecuzione (2026-06-16) — Tier 1 + Tier 2 COMPLETI ✅

Tutto verificato dal vivo (server :3000, ricompilato `--features web`).

**Tier 1 (frontend)** — ✅ tutto fatto:
1. ✅ Hub admin sottile (`viste/hub.html`): vitali O(1) da `/api/state` + griglia
   di card-link + angolo manutenzione (dream/grow/persist). Servito da disco via
   `serve_vista()` (editabile senza rebuild). Tolto il badge "in arrivo" dal Gate.
2. ✅ Frattali pulita (`viste/frattali.html`): solo three.js da `/viste/vendor/`,
   consuma `/api/universe`. Assorbe l'ex `/universo`.
3. ✅ Stato interno (`viste/stato-interno.html`): titolo = "Cosa non ho ancora
   capito" (domande/vuoti aperti/chiusi da `/api/speaker` + pensieri-limite da
   `/api/thoughts` + incertezze da `/api/self`). **Fix**: le incertezze leggevano
   `/api/open-questions` (vuoto + dismesso) → ora `/api/self.uncertainties`.
4. ✅ Ritirate le pagine: `/biennale`, `/campo-vasto`, `/diffrazione`, `/ui-r1`,
   `/universo` → route 404, handler+`include_str!` rimossi, 4 file HTML eliminati
   (`biennale/{index,diffrazione,uir1}.html`, `universo/index.html`). Tenuti i loro
   *dati* (`/api/universe`→frattali, `/api/biennale/*`→campovasto).
5. ✅ Home: stat dinamiche da `/api/state` (`vocabulary_size` + nuovo campo
   `kg_edge_count` in `ReportDto`) → 25.612 parole / 95.527 connessioni (non più
   25.870 / 251.454 hardcoded stale). Tolto "(esagrammi I Ching)" dalla landing.

**Tier 2 (backend)** — ✅ tutto fatto:
6. ✅ `/campovasto-cy`: route già assente (dir già eliminata) → 404.
7. ✅ `/api/self`: `build_self_dto` non espone più convinzioni/valori INNATI
   (il sé dismesso); shape DTO invariato (campi → `Vec::new()`, nessun consumer
   rotto, incl. tool MCP `get_self_profile`); `uncertainties` tenute (24 live).
8. ✅ Ritirati 18 endpoint dismessi → 404: `/api/diffraction`, sonde Phase-67
   (`navigate`, `phase`, `tension`, `projection`), introspezione vecchia (`why`,
   `ask`, `open-questions`, `clarity`, `thought-chain`), dialogo Phase 52
   (`inner-dialogue`, `respond`), altre sonde (`generate`, `introspect`,
   `compounds`, `locus-simulate`, `simpdb`, `topology`). Verifica pre-rimozione:
   nessuna pagina-tenuta li chiamava. Handler `pub` lasciati in `api.rs` come
   dead-code (innocui; cleanup opzionale futuro).

**Vivo e verificato (200)**: `/api/comprehend` (Gate, POST — "ho paura del futuro"
→ verdetto *piena*), `/api/state`, `/api/narrative`, `/api/thoughts`,
`/api/visuals`, `/api/universe`, `/api/wordfield`, `/api/self`, `/api/speaker`,
`/api/will`, `/api/episodes`, `/api/concept`, `/api/biennale/*`, `/api/cura/*`.

## Cleanup profondo (2026-06-16, dopo Tier 1+2)

- **MOJIBAKE = FALSO ALLARME.** Il "bug" volonta->volontA NON esiste: i byte
  della risposta /api/self sono `76 6f 6c 6f 6e 74 c3 a0` = "volonta" UTF-8
  CORRETTO (identico a `printf`). Lo snapshot self_model ha topic puliti. La
  "corruzione" era un artefatto del diagnostico *python su Windows* che leggeva
  lo stream UTF-8 come cp1252. Il browser (UTF-8) rende bene in Stato interno.
  La migrazione difensiva tentata e' stata RIMOSSA (no codice speculativo).
  Lezione: su Windows diagnosticare sui byte (xxd), non via `python | json`.
- **Monolite src/web/index.html** (153KB, orfano) -> **old/web/index.html** (git mv).
- **4 pagine ritirate preservate** (non cancellate) in **old/web/**:
  biennale/{index,diffrazione,uir1}.html, universo/index.html.
- **18 handler orfani RIMOSSI da api.rs** (topology/navigate/projection/introspect/
  why/ask/open_questions/thought_chain/clarity/generate/compounds/phase/tension/
  locus_simulate/simpdb/inner_dialogue/respond/diffraction). Build verde, zero
  nuove warning (EngineCommand e' pub).
- **viste/** (hub/stato-interno/frattali + vendor three.js) ora TRACCIATA in git.
- **Root scratch NON toccata** (sicurezza): i ~50 file untracked in root sono
  materiale di ricerca vivo/critico (es. prometeo_kg_self.json caricato al boot;
  cura_*.py/*_pending.json/bench_*/kg_lint.py infrastruttura attiva). Spostarli
  romperebbe boot/workflow. Archiviare solo con conferma per-file.

**Dead-code residuo (opzionale)**: il backplane interno delle 18 route ritirate
(varianti EngineCommand + match arm server.rs + build-fn + DTO) resta peso morto
non raggiungibile da HTTP, nessuna warning (enum pub). Rimozione = refactor
cascade, rimandata.
