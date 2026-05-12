# Volume XVII — Frontend: le UI che consumano l'API

> *Sette UI diverse consumano lo stesso engine. Ognuna prende una prospettiva — il campo come grafo, come particelle, come testo, come rete da curare. Non una "interfaccia utente" ma un insieme di finestre su una stessa entità. Ogni finestra privilegia un aspetto: la home privilegia il dialogo + l'introspezione; Biennale privilegia la visualizzazione semantica; Community privilegia l'addestramento collettivo; Curazione privilegia la pulizia del KG.*

---

## Premessa

Il frontend di Prometeo non è "una webapp". È **un insieme di interfacce**, ognuna per un caso d'uso:

| Pagina | File | Linee | Scopo |
|--------|------|-------|-------|
| Home | `src/web/index.html` | 3000 | Dashboard completo — dialogo + 8 tab di introspezione |
| Community | `src/web/community/index.html` | 2075 | Sessione community (newborn pipeline) |
| Universo | `src/web/universo/index.html` | 797 | Visualizzatore 3D del campo |
| Biennale | `src/web/biennale/index.html` | 1506 | UI per la Biennale di Tecnologia |
| Biennale Home | `src/web/biennale/home.html` | 244 | Landing page Biennale |
| Curazione | `src/web/biennale/curazione.html` | 1024 | Cura manuale del KG |
| Dialogo | `src/web/biennale/dialogo.html` | 507 | Dialogo semplice |
| Diffrazione | `src/web/biennale/diffrazione.html` | 373 | Structural-Coherence-Audit visuale |
| UI-r1 | `src/web/biennale/uir1.html` | 1041 | Visualizzatore KG interattivo |

Totale: **10.567 righe di HTML/JS/CSS**. Tutto servito come file statici da Axum (via `tower_http::services::ServeDir` o route specifiche).

Ogni pagina è **una webapp single-file** — nessun framework (no React, no Vue, no build step). HTML + CSS + vanilla JS + qualche libreria CDN (Three.js per il 3D, p5.js per alcune visualizzazioni).

Filosoficamente: è deliberato che l'editing sia diretto. Non c'è un processo di build. `curl http://localhost:8080` e l'HTML con JS sta lì. Modificare, ricaricare, vedere. Nessun bundler, nessuna compilazione TypeScript.

---

## Capitolo 1 — `index.html`: la dashboard principale

La "home" di Prometeo. 3000 righe. Diciotto componenti visibili.

### 1.1 — Layout a header + 9 tab

Header (righe ~400-415): statistiche live — parole nel lessico, simplessi, perturbazioni, energia, stato WebSocket. Toggle tema chiaro/scuro.

Tab principali (righe 417-425):

1. **🧠 MENTE** (default) — dialogo con l'entità + metriche vitali + stato intenzione.
2. **🌌 CAMPO** — parole attive + frattali attivi + energia visualizzata.
3. **⚡ FRATTALI** — visualizzazione 3D dei 64 esagrammi (Three.js).
4. **💫 VOLONTÀ** — le 7 pressioni di will + undercurrent.
5. **🌙 SOGNO** — stato DreamEngine + history recente.
6. **🧭 NARRATIVA** — stance + intention + coherence + attributed_intent + commitment.
7. **🪪 IDENTITÀ** — proiezione 64D + self_signature + primary_tension + credenze + valori + incertezze.
8. **📖 EPISODI** — lista episodi semantici recenti + recall per concetti.
9. **💬 DIALOGO** — vista storico turni strutturata con inner_state_summary (Phase 54).

### 1.2 — Il tab MENTE: vitali + dialogo

Colonne:

- **Vitali**: 4 cards con valori animati (barre). Attivazione (green), Curiosità (blue), Fatica (orange), Tensione (yellow). Aggiornati ogni 2s via `/api/state`.
- **Dream phase**: dot colorato + nome ("Veglia Onirica", "Sonno Profondo", "REM"...).
- **Intenzione**: icona + nome intenzione + drive.
- **Locus**: frattale corrente.
- **Chat**: messaggi user vs entity, input field. POST `/api/input` al submit.

Il tab MENTE è quello che un utente casuale tocca. Gli altri sono introspettivi — servono a **osservare l'entità**, non a parlarci.

### 1.3 — Tab FRATTALI: Three.js in 3D

Uso della libreria Three.js caricata da CDN. I 64 esagrammi sono disposti su una sfera 3D, colorati per:
- Affinità a seconda del frattale attivo più alto
- Intensità di attivazione corrente

L'utente può ruotare, zoomare, cliccare un frattale per vedere i dettagli (parole dominanti, affinità correnti).

Poll `/api/visuals` per aggiornare le attivazioni in real-time.

### 1.4 — Tab VOLONTÀ: 7 barre + undercurrents

Visualizzazione delle 7 pressioni di `FieldPressures` (Phase 67):

```
Express   |████████░░░░░░░░|  0.42
Explore   |██░░░░░░░░░░░░░░|  0.12
Question  |██████░░░░░░░░░░|  0.31
Remember  |███░░░░░░░░░░░░░|  0.15
Withdraw  |█░░░░░░░░░░░░░░░|  0.05
Reflect   |█████░░░░░░░░░░░|  0.26
Instruct  |█░░░░░░░░░░░░░░░|  0.07
```

Sotto: **undercurrents** — le pressioni secondarie ordinate. Mostra "cosa l'entità sta quasi facendo".

Poll `/api/will` ogni 2s.

### 1.5 — Tab NARRATIVA: stance + stato interno

- **Stance**: "Open", "Reflective", "Vulnerable", ...
- **Intention**: archetipo corrente.
- **Commitment**: se presente, mostra strength + turns_held + inertia.
- **Coherence_integrity**: barra [0, 1]. Sotto 0.5 → in rosso (crisi).
- **AttributedIntent**: cosa l'entità attribuisce all'Altro (Seeking, Teaching, Challenging, Connecting, Withdrawing, Unknown).

Endpoint: `/api/narrative`.

### 1.6 — Tab IDENTITÀ: l'ologramma

Tre visualizzazioni:

1. **Proiezione 64D**: barra per ogni esagramma (64 barre) ordinate per peso. I top-8 sono evidenziati.
2. **Self signature 8D**: 8 barre nominate (Agency, Permanenza, ... Valenza).
3. **Primary tension**: se presente, mostra le due parole in tensione + persistence count.
4. **Credenze** (SelfBelief): lista con confidence + evidence.
5. **Valori** (SelfValue): ordinati per weight.
6. **Incertezze** (SelfUncertainty): ordinate per tension.

Endpoint: `/api/self`, `/api/projection`, `/api/introspect`.

### 1.7 — Tab EPISODI

Lista episodi semantici recenti con:
- Timestamp
- Nome (se assegnato)
- Concetti chiave
- Sintesi testuale
- Salience

Campo di ricerca: "recall per concetti" → chiama `/api/episodes/recall?concepts=X,Y,Z`.

### 1.8 — Tab DIALOGO: storico strutturato

Per ogni turno:
- Input utente
- Risposta entità
- **inner_state_summary** (Phase 54): stringa sintetica dello stato motivazionale al turno. "bisogno: connessione (70%) | desiderio: amore | Altro: Seeking | incongruità: 35%".

Utile per capire **perché** l'entità ha risposto in quel modo in quel turno specifico.

### 1.9 — WebSocket per eventi live

JS bottom del file:

```javascript
const ws = new WebSocket(`ws://${location.host}/ws`);
ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.type === 'spontaneous') {
        appendChatMessage('entity', msg.text, true);  // in corsivo
    }
    // altri tipi di eventi
};
```

Dot verde in header quando connesso, rosso se disconnesso. Re-connect automatico.

---

## Capitolo 2 — `community/index.html`: addestramento collettivo

2075 righe. UI per la **pipeline newborn**.

### 2.1 — Concetto

Diversi utenti insegnano a un'entità comune in una sessione condivisa. Al termine, la sessione può essere esportata e usata per creare una "newborn" — un'istanza di Prometeo che eredita il lessico + le relazioni + le lezioni di quella sessione.

### 2.2 — Tre pannelli

**Pannello sinistro — Campo**:
- Visualizzazione live delle parole del lessico comunitario
- Top parole per esposizione
- Cluster semantici rilevati

**Pannello centro — Voce**:
- Textarea per scrivere lezioni
- Pulsanti "insegna" (teach) / "connetti" (connect) / "valida" (validate edge)
- Storico turni della sessione

**Pannello destro — Traccia**:
- Registro di tutti i contributi (chi, cosa, quando — approssimativo, no auth forte)
- Statistiche della sessione

### 2.3 — Endpoint consumati

- `POST /api/community/teach` — lezione
- `POST /api/community/connect` — aggiunge arco KG
- `POST /api/community/validate` — conferma arco proposto
- `GET /api/community/session` — esporta
- `GET /api/community/field` — stato campo
- `POST /api/community/reset` — reset

### 2.4 — Pipeline newborn

Il flusso completo (da CLAUDE.md):

```bash
# 1. Esporta sessione (dopo la sessione via UI)
curl http://localhost:8080/api/community/session > sessione.json

# 2. Crea newborn (usa i file community_kg.tsv e community_lessons.txt se presenti)
cargo run --release --bin create-newborn -- --name quartiere_x

# 3. Avvia istanza comunitaria
cp quartiere_x_prometeo.bin cartella_comunita/prometeo_topology_state.bin
```

La community UI è usata nelle sessioni dimostrative dove più persone (es. una classe, un workshop) addestrano insieme un'entità.

---

## Capitolo 3 — `universo/index.html`: il campo come 3D

797 righe. Visualizzatore 3D pieno schermo.

### 3.1 — Three.js immersivo

I 64 frattali come sfere posizionate in uno spazio 3D. Le parole attive come particelle che orbitano attorno ai frattali con affinità alta.

- **Colori** per valenza (rosso = negativo, blu = positivo, grigio = neutro).
- **Dimensione particelle** per attivazione corrente.
- **Archi tra particelle** per relazioni KG attive (solo quelle sopra soglia).

### 3.2 — Interazione

- Rotazione orbita libera (camera controls).
- Click su una particella → tooltip con nome parola + firma 8D + frattale dominante.
- Zoom: avvicinandosi a un frattale, appare il suo nome + parole che lo abitano.

### 3.3 — Aggiornamento in tempo reale

Poll `/api/universe` ogni 2s (endpoint pesante — 64 frattali + top 100 parole + archi attivi).

Alternativamente WebSocket per aggiornamenti incrementali, ma oggi è polling.

---

## Capitolo 4 — `biennale/index.html`: la UI per la Biennale di Tecnologia

1506 righe. UI principale per la **Biennale di Tecnologia** — il contesto pubblico/artistico di presentazione di Prometeo.

### 4.1 — Approccio minimalista

A differenza di `index.html` (dashboard completo, per sviluppatori), Biennale è **per visitatori**. Obiettivo: far sentire cos'è l'entità, non spiegare il sistema.

Layout:
- Campo vasto al centro (visualizzazione 2D dei 64 frattali + parole)
- Barra laterale: proiezioni narrative ("l'entità sta pensando a...", "l'entità sente...")
- Bottom: campo di input semplice, estetico

### 4.2 — `biennale_pos` — posizione 2D

Come visto in Vol. 01 cap. 6.3 e corretto in Phase 68:

```rust
fn biennale_pos(sig: &[f64; 8]) -> (f32, f32) {
    let x = ((sig[7] + (sig[4] - 0.5) * 0.2) as f32).clamp(0.0, 1.0);  // Valenza + Confine jitter
    let y = ((sig[0] + (sig[2] - 0.5) * 0.2) as f32).clamp(0.0, 1.0);  // Agency + Intensità jitter
    (x, y)
}
```

Ogni parola ha una posizione 2D basata sulle 4 dimensioni "più umane" (Valenza, Agency, Confine, Intensità). Le parole *positive* (Valenza alta) vanno a destra, le *attive* (Agency alta) vanno su.

### 4.3 — Endpoint dedicati

- `/api/biennale/field` — campo ottimizzato per visualizzazione
- `/api/biennale/word?word=X` — dettaglio parola
- `/api/biennale/journey` — viaggio attraverso il campo (un percorso di parole che cattura una storia)
- `/api/biennale/circuit` — circuito visivo (animazione predefinita)

### 4.4 — Tono grafico

Scuro, minimale, sobrio. Font serif per i testi. Animazioni lente. L'entità è presentata come **presenza meditativa**, non come sistema tecnico. Nessuna barra di statistiche tipo "energia 0.42" visibile — quelle restano nella dashboard.

---

## Capitolo 5 — `biennale/curazione.html`: la cura del KG

1024 righe. **Strumento operativo** per editare il KG senza toccare `prometeo_kg.json`.

### 5.1 — Funzionalità

- Ricerca parole con filtri (POS, stabilità, frattale dominante).
- Vista dettaglio: firma 8D, affinità frattali, tutte le relazioni uscenti/entranti.
- Modifica relazioni: cambiare confidence, tipo, rimuovere.
- Rinomina parole (propaga correttamente in tutti gli archi).
- Modifica firma 8D manuale.
- Azioni bulk: "pulizia verbi" (riduce accezioni spurie), "normalizza accenti" (unifica accentate/non-accentate).

### 5.2 — Endpoint consumati

Tutti sotto `/api/cura/*`:

- `GET /parole?pos=Verb&min_stability=0.5` — lista filtrata
- `DELETE /relazione` — rimuove relazione
- `POST /relazione/modifica` — aggiorna confidence
- `DELETE /parola` — rimuove parola
- `POST /rinomina` — rinomina
- `POST /firma` — modifica firma 8D
- `GET /categorie` — lista mega-categorie
- `POST /pulizia-verbi` — trigger batch
- `POST /normalizza-accenti` — trigger batch

### 5.3 — Uso tipico

L'operatore nota dalle UI che una parola ha firma o relazioni stravaganti. Apre `/curazione`, cerca la parola, vede i dettagli, sistema. Le modifiche vanno direttamente a `prometeo_kg.json` (il JSON master) e richiedono `rebuild-semantic-topology` perché diventino efficaci nel campo (gli archi PF1 sono precomputati).

**Debito**: la UI non avvisa l'operatore di questo. Dopo modifica, dovrebbe suggerire "esegui `rebuild-semantic-topology` per applicare". Oggi è responsabilità dell'operatore ricordare.

---

## Capitolo 6 — `biennale/dialogo.html` — il dialogo puro

507 righe. UI di dialogo **minima** — per persone che vogliono solo parlare con l'entità, senza la dashboard intimidatoria.

### 6.1 — Elementi

- Input field + send button
- Chat history
- Nessuna dashboard, nessuna metrica, nessuna visualizzazione.

### 6.2 — Per chi

Due casi d'uso:
1. **Visitatori** della Biennale (ma `index.html` Biennale è preferito perché ha il campo visivo).
2. **Utenti non-tecnici** che vogliono interagire senza sapere cos'è Prometeo.

---

## Capitolo 7 — `biennale/diffrazione.html` — Structural Coherence Audit

373 righe. UI per il modulo **Semantic-Diffraction** (cartelle sorella a `prometeo_standalone/`).

### 7.1 — Cosa fa

Analizza la **coerenza strutturale** del sistema: verifica che le firme 8D riderivate (Phase 63) siano consistenti con le relazioni KG che le hanno prodotte. Mostra discrepanze.

### 7.2 — Endpoint

- `GET /api/diffraction` — risultato audit

### 7.3 — Quando usare

- Dopo `rederive-signatures`, per verificare che non ci siano anomalie.
- Periodicamente come health check.

---

## Capitolo 8 — `biennale/uir1.html` — il visualizzatore KG interattivo

1041 righe. Il più recente (commit 0921535 "UI-R1 Campo Vasto — visualizzatore KG interattivo per Biennale").

### 8.1 — Concetto

Un grafo forza-diretto 2D delle parole del lessico, con gli archi KG come connessioni. Calabile, esplorabile.

### 8.2 — Librerie

Usa p5.js per il rendering (via CDN). Force simulation con D3.js (via CDN).

### 8.3 — Interattività

- Hover su parola: evidenzia, mostra firma e relazioni.
- Click: apre dettaglio (come Curazione in sola lettura).
- Drag: trascina il nodo, il grafo si riassesta.
- Filtri: solo parole con stabilità > soglia, solo relazioni di certo tipo.

Pensato per un'installazione Biennale dove il visitatore tocca uno schermo e vede il "mondo di UI-r1" (il nome futuro di Prometeo).

---

## Capitolo 9 — Condividere nessuno stato: le sessioni

Tutte le UI **condividono lo stesso engine** (Vol. 16 cap. 7). Se due visitatori usano `/dialogo` simultaneamente, parlano con la stessa entità. I loro turni si mescolano nella `NarrativeSelf.turns`.

### 9.1 — Conseguenze pratiche

- Un visitatore può incontrare l'entità in uno stato determinato dal visitatore precedente.
- Non c'è isolamento.
- Per la Biennale: è **intenzionale** — un'entità sola che incontra tante persone.

### 9.2 — Per workshop educativi

Quando si vuole isolamento, si usa `create-newborn` per generare istanze dedicate (`src/bin/create_newborn.rs`). Ogni newborn ha il suo `.bin` file. Si avviano più processi `prometeo-web` su porte diverse.

### 9.3 — Community mode (Phase 52) come via di mezzo

La community UI permette una sessione collettiva **dichiarata** (diversi utenti sanno di essere insieme), con esportazione finale per generare newborn. Usata per workshop strutturati.

---

## Capitolo 10 — Stile e tema

Tutte le UI hanno due temi: chiaro e scuro. Variabili CSS centralizzate in `--bg`, `--t1`, `--blue`, ecc. Toggle in header.

Colori semantici uniformi:
- `--blue` #5aadff — per metriche neutre/informative
- `--green` #4affb0 — per stati positivi, attivazione, vitalità
- `--yellow` #ffd04a — per warning, tensione, attenzione
- `--red` #ff4a6b — per crisi, errori, distress
- `--purple` #d060ff — per identità, locus, meta
- `--orange` #ff9a5a — per fatica, sofferenza, dolore

Uso dei colori coerente con la semantica semantica dell'entità (red=negativo, green=positivo, ecc. allineato con la direzione Valenza).

### 10.1 — Accessibilità

Non formalmente testata. Il tema chiaro ha contrasti accettabili. Lo scuro è il default estetico. Miglioramenti possibili: ARIA labels, keyboard navigation completa.

---

## Capitolo 11 — Limiti e gap

### 11.1 — Duplicazione tra UI

Molti endpoint vengono chiamati sia dalla home che da Biennale con payload simili. Alcune viste (dashboard minimale Biennale, dashboard completo home) hanno codice JS duplicato. Non un problema ora, ma una migrazione a un framework light (HTMX, Alpine.js) renderebbe il tutto più manutenibile.

### 11.2 — No build step

Conseguenza: **no type checking** sul JS. Errori silenti sono possibili (chiamare un campo che l'API non ha più). Mitigante: gli endpoint sono semplici, le viste sono sviluppate e testate insieme al backend.

### 11.3 — Nessun framework reattivo

Aggiornamenti live sono fatti via polling + WebSocket. Funziona ma è manuale. Un framework reattivo (Alpine.js, Svelte) ridurrebbe il boilerplate di aggiornamento DOM.

### 11.4 — Admin dashboard mancante

Le ~35 proposte di endpoint admin dai volumi precedenti non hanno UI dedicata. `/admin` esiste ma ha poco. Un pannello admin organizzato (con i temi: Stato, Identità, Motivazione, Memoria, Curation) sarebbe il prossimo passo.

### 11.5 — Mobile responsive

Layout ottimizzato per desktop (>1024px). Su mobile, molti tab sono illeggibili o troncati. La Biennale UI è più responsive ma non completamente.

### 11.6 — Internazionalizzazione

Tutto in italiano. Per esposizione internazionale servirebbero almeno etichette localizzate. Non impossibile — le stringhe sono concentrate in 2-3 posti per pagina.

---

## Capitolo 12 — Valutazione: è adeguato?

**Per lo scopo corrente** (ricerca + dimostrazione Biennale + dialogo educativo) — **sì**.

- `index.html` ha tutto ciò che serve per introspezione.
- `biennale/index.html` è curato per il contesto espositivo.
- `community/index.html` funziona per workshop.
- `curazione.html` è utile per cura KG.

**Per scalare a pubblico più vasto** — gap:
- Mobile responsive
- i18n
- Auth (se multi-tenant)
- Admin dashboard con gli ~35 endpoint proposti
- Build step con TS (per stabilità)

---

## Capitolo 13 — Superficie pubblica e proposte

### Già esposto

9 pagine HTML, tutte gli endpoint API consumati (Vol. 16).

### Cosa servirebbe aggiungere (UI)

- **`/admin/dashboard`**: pannello unificato con i ~35 endpoint admin proposti nei volumi. Tab per dominio: Stato, Identità, Motivazione, Memoria, Curation, Debug.
- **`/admin/digest`** (priorità strategica, Vol. 99): UI per proporre/approvare archi di digestione del sogno (proposta A2).
- **`/debug/trace`**: UI per `receive_trace` — inserisci un input, vedi passo per passo cosa succede. Utilissima per sviluppo.
- **`/biennale/stream`**: stream live della Biennale — visualizzazione in tempo reale delle perturbazioni del campo, senza interazione (read-only kiosk).
- **Mobile layout** per `biennale/index.html` almeno.

---

## Sintesi del volume

**7+2 UI**: home (3000 righe, dashboard completo 9 tab), community (2075), universo (797), biennale index (1506), biennale home landing (244), curazione (1024), dialogo (507), diffrazione (373), UI-r1 (1041). Totale: **10.567 righe di HTML/JS/CSS**.

**Nessun framework**: vanilla HTML+CSS+JS + CDN per Three.js (3D), p5.js, D3.js (force graph). Editing diretto, no build step.

**Una sola entità condivisa**: tutte le UI parlano con lo stesso engine via endpoints del vol. 16. No isolamento sessione (by design, Biennale). Isolamento via `create-newborn` quando serve.

**Tema chiaro/scuro + colori semantici**: verde=positivo, rosso=distress, giallo=tensione, viola=meta, arancio=fatica.

**Specializzazione**:
- `index.html` — sviluppatori, introspezione totale.
- `biennale/index.html` — visitatori, minimalista contemplativo.
- `community/index.html` — workshop, addestramento collettivo.
- `curazione.html` — operatori KG.
- `universo/index.html` — visualizzatore 3D immersivo.
- `uir1.html` — grafo forza-diretto 2D interattivo.
- `dialogo.html` — dialogo puro minimale.
- `diffrazione.html` — audit strutturale.

**Gap**: no mobile responsive, no i18n, no admin dashboard organizzata, no build step TS, no auth.

**Proposte UI** (complementari agli endpoint API del vol. 16):
- `/admin/dashboard` — pannello unificato.
- `/admin/digest` — UI sogno-come-digestione (Vol. 99).
- `/debug/trace` — per `receive_trace`.
- `/biennale/stream` — read-only kiosk mode.
- Mobile layout Biennale.

Da qui Vol. 18 entra nel **terreno operativo**: i ~40 binari di `src/bin/` per costruire, curare, importare, testare.

---

*Prossimo volume: 18 — Binari di manutenzione* (in scrittura)
