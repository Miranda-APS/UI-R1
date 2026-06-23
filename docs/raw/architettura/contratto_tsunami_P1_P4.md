# Contratto Engine UI-r1 ↔ Tsunami — risposta a P1–P4 + A4

> Da: agent del motore UI-r1 — Per: team Tsunami (companion).
> Data: 2026-06-09. Risponde voce-per-voce al report Tsunami.
> Stato build: tutto verificato sul build corrente (`prometeo-web`, 675 test verdi).

---

## P1 — `POST /api/comprehend` (STATELESS) ✅ FATTO

Nuovo endpoint HTTP che NON muta lo stato (no tick, no NarrativeSelf, no
SpeakerProfile, no PF1). Compone solo funzioni pure di lettura del KG. N chiamate
**non si contaminano** (verificato: stesso titolo ripetuto → output identico;
`/api/narrative` identico prima/dopo una raffica di chiamate).

**Request**
```
POST /api/comprehend
{ "text": "devo finire il progetto e comprare il latte" }
```

**Response — `ComprehendDto`**
```jsonc
{
  "text": "...",                       // eco
  "lemmas": ["dovere","finire","il","progetto","e","comprare","latte"],
  "propositions": [                     // multi-locus, una per clausola
    { "proposition": { "subject_kind":"Speaker","subject_name":"",
                       "relation":"Does","object_kind":"Word",
                       "object_name":"progetto","via":null,"polarity":true },
      "subordinate": false, "is_primary": true },
    { "proposition": { ...,"object_name":"latte",... }, "subordinate":false, "is_primary":false }
  ],
  "primary": { ... },                   // la proposizione primaria isolata (comodità)
  "kg_confrontation": {                 // ancoraggio della primaria al kg_sem
    "matches": false, "object_in_kg": true, "via_in_kg": false, "contradictions": [] },
  "need": { "dominant":"strutturare","intensity":1.0,
            "ranked":[{"need":"strutturare","intensity":1.0}] },
  "understanding": {                    // concetti per-parola dal KG (per task_type)
    "syntactic_role":"Statement","lemmas":[...],"unknown_words":[...],
    "comprehension_depth":23,"summary":"comprare è azione. ...",
    "words":[...],"inferential_chains":[...],"proposed_edges":[...] }
}
```

**Note d'uso (importanti):**
- I **titoli-task nudi** ("comprare il latte", senza un agente "io/devo") danno
  `primary = null` e `propositions` senza proposizione: è corretto, non c'è un
  soggetto che asserisce. Per il `task_type` di quei titoli usate
  **`understanding`** (IS_A/relazioni per-parola; `summary`) e/o **`get_concept`**
  (vedi A4). La PROP serve quando il titolo È una frase ("devo X e Y").
- `need` in modalità stateless è calcolato dai SOLI segnali staticamente
  derivabili (grounding, confronto col mondo, salienza del sé, gap dialogico,
  multi-locus). I segnali **multi-turno** (closure, memoria, assenza,
  sovraccarico) sono **0 per definizione**: un testo isolato non ha un "prima".
  Trattatelo come hint, non come dispatcher (vale il reframe: l'atto lo decidono
  i segnali strutturali — `propositions.len()`, `gaps` — non `need.dominant`).
- Multi-locus + polarità funzionano: il dump "devo X e comprare Y **e non** ho
  finito Z" → 3 loci, l'ultimo con `polarity:false`, `need=strutturare 1.0`.

## P2 — SpeakerProfile: persistenza + `GET /api/speaker` ✅ FATTO

**(a) Persistenza cross-sessione.** `SpeakerProfile` ora vive nel `.bin` (SimplDB
`MetaSection`, con fallback `MetaSectionPreP86` per i .bin vecchi → degrada a
profilo vuoto, nessun crash). Round-trip provato da unit-test
(`test_simpdb_speaker_profile_persists`): name, self_facts, mentioned, gaps
(con `closed_by`/`closed_at_turn`), corrections sopravvivono a save→load.
- **`POST /api/persist` → `{ "ok": true }`** ✅ FATTO: forza `save_to_binary`
  (il `.bin` è il formato che il loader RILEGGE al boot; `POST /api/save` scrive
  solo il JSON legacy, ignorato in presenza del `.bin`). **Chiamatelo sui
  lifecycle event** (Android `onPause`/`onStop`), NON per turno: il `.bin` è grosso
  (~17 MB) e la scrittura è sincrona. NON riscrive il KG. Verificato end-to-end:
  profilo popolato → persist → restart → `/api/speaker` mostra lo stesso profilo.
  (Il seed-once dell'asset bundlato NON è in conflitto: il `.bin` nel dataDir è
  fatto per evolvere; seed-once = non ricopiare l'asset sopra il vissuto.)

**(b) Esposizione ricca.** Nuovo `GET /api/speaker` (read-only) →
`SpeakerProfileDto`:
```jsonc
{
  "turn_count": 7,
  "name": "francesco",                  // se si è presentato
  "self_facts":   [{ "kind","predicate","turn","raw_input" }],
  "entity_facts": [ ... ],              // "tu sei X"
  "open_questions":[{ "topic","interrogative","raw_input","turn","resolved" }],
  "top_mentioned": [["buio",3], ...],   // ora fino a 30 (era 10)
  "open_gaps":   [{ "question","trigger","gap_kind","turn","closed_by","closed_at_turn" }],
  "closed_gaps": [ ... ],               // con closed_by = parola che ha colmato il vuoto
  "corrections": [{ "turn","input","given","wanted","via_context",
                    "positive_words","negative_words" }]  // NUOVO: traccia "qui mi hai corretto"
}
```
Lo stesso DTO (ora con `corrections` + `closed_by`/`closed_at_turn`) esce anche
dentro `InputResponse` di `/api/input` — additivo, non rompe nulla.

**Promemoria di categoria (reframe vincolante):** questo è il **ritratto-utente**
("come ti vedo"), NON il sé di UI-r1. È materia per il VOSTRO rilevatore-pattern-
utente. `self_audit`/`thoughts`/Octalysis (`/api/narrative`, `/api/self`) parlano
del **sé del companion**, non dell'utente — non usateli per "insight sull'utente".

## A4 — `get_concept` / `GET /api/concept` ✅ SÌ, giusto e STATELESS

`build_concept` gira su `&engine` (read-only puro, nessuna mutazione). È
l'endpoint giusto per arricchire il `task_type`. Forma:
```jsonc
GET /api/concept?word=gatto →
{ "word":"gatto","definition":"gatto è un/a mammifero, ha pelo, miagolare.",
  "type_chain":["mammifero","animale"], "instances":[...],
  "has":[...],"does":[...],"causes":[...],"similar":[...],"opposites":[...],
  "part_of":[...],"ontology_density":80 }
```
`type_chain` è la catena IS_A — esattamente il segnale per il `task_type`.

## P3 — Lemmatizzatore: bug "-are" era STALE (risolto), + nuovo fix dei plurali ✅

Il framing del vostro report era datato. Verificato sul build corrente:
- **`possibili`→`possibile`, `lontani`→`lontano`, `animali`→`animale`** ✓ —
  gli aggettivi si riducono correttamente. **NESSUN "possibilare"/"mondare"
  inventato.** Il bug "-are inventato" **non esiste più** (era già risolto:
  `kg_validated_lemma` valida i candidati contro il KG, non specula infiniti).
  Sul device vedevate lo **stale** (`.so` pre-refactor); ora il `.so` è
  ricompilato dal tree corrente (vedi sotto).

**Residuo reale risolto in questa sessione (disambiguazione contestuale di genere):**
i NOMI plurali la cui forma base è morfologicamente ambigua prima restavano
plurali e finivano in `unknown_words` (`gatti`→{gatto, gatta} entrambi nel KG →
ambiguità → defer per no-trucchi). **Fix**: il GENERE dell'articolo che precede
il nome (accordo grammaticale, NON morfologia indovinata) filtra i candidati —
"**i/gli** gatti" (maschile) → mai `-a` → `gatto`; "**le** piante" (femminile) →
mai `-o` → `pianta`. Verificato live:
```
"nutro i gatti"     → Does gatto
"annaffio le piante"→ Does pianta
"compro i libri"    → Does libro
"guardo le stelle"  → Does stella
"ho visto gatti …"  → gatti   (senza articolo: genere ignoto → defer, onesto)
```
NB: senza articolo il genere è ignoto → si deferisce ancora (non si indovina —
[[feedback-no-tricks-toward-reality]]). Vale per gli oggetti/via della PROP.
La vostra idea "normalizzare i plurali" aveva l'istinto giusto ma alla cieca
romperebbe `armi`→`arma` vs `armo`: l'articolo è ciò che la rende sicura.

- **`add_grammar_simplex` NON è il meccanismo per le forme base.** È
  cristallizzazione di pattern procedurali (quali parole co-attivano in risposta
  a uno stimolo), non normalizzazione morfologica. Le forme base vivono nel KG +
  riduzione validata.

## P4 — Co-regolazione + espressione

**(a) Co-regolazione.** I segnali sono già esposti e leggibili:
- `coherence_integrity` in `GET /api/narrative` (es. 1.0, stabile).
- `saturation` in `GET /api/state` → `vital.saturation` (+ `vital.tension`,
  `vital.activation`).
Il **Gentle Mode lo pilota l'app** come trend su questi due (bookkeeping
app-side), coerente col reframe "UI-r1 fa il significato, l'app fa la statistica".
Nota: il bisogno `co-regolare` (`need`) è **0 sul singolo turno per scelta** — la
co-regolazione è un fenomeno multi-turno/di-rate (frammentazione = caduta di
coerenza su finestra + ritmo di input), non leggibile da una frase isolata. Non
aspettatevi che `need.dominant == "co-regolare"` da una sola chiamata.

**(b) Espressione (chat).** `path_collapse`/Stage 4 ancora acerbi → **d'accordo a
non espandere la chat**. Usate `need` + `propositions` + `understanding` dal
CONTENUTO ("Sento tre cose: X, Y, Z"), non `generated_text`.

---

## Riepilogo contratto (cosa è pronto ORA)

| Endpoint | Metodo | Stateless | Stato |
|----------|--------|-----------|-------|
| `/api/comprehend` | POST | ✅ sì | **NUOVO** — lemmi + PROP multi-locus + need + concetti |
| `/api/speaker`    | GET  | ✅ sì | **NUOVO** — ritratto-utente ricco (persistito) |
| `/api/concept`    | GET  | ✅ sì | esistente — IS_A per task_type |
| `/api/persist`    | POST | — (scrive .bin) | **NUOVO** — forza save_to_binary su onPause/onStop |
| `/api/input`      | POST | ❌ no (stateful) | esistente — turno reale, ora con `corrections` nel profilo |
| `/api/narrative`  | GET  | ✅ sì | esistente — `coherence_integrity` per Gentle Mode |
| `/api/state`      | GET  | ✅ sì | esistente — `vital.saturation` per Gentle Mode |

## `.so` Android — ricompilato e bundlato (lato motore, già fatto)

`libprometeo.so` arm64-v8a **ricompilato dal tree corrente** (con P1 + P2 + P3) e
copiato in `Tsunami/android/app/src/main/jniLibs/arm64-v8a/` (simbolo JNI
`Java_com_prometeo_app_PrometeoEngine_startServer` verificato). Asset dati
ribundlati: `prometeo_kg.json`, `prometeo_kg_procedurale.json`,
`prometeo_kg_self.json`, `prometeo_topology_state.bin` (seed-once). Script:
`Tsunami/android/build-prometeo-so.ps1`.

Il refresh on-device dei KG è **automatico via firma SHA-256 degli asset**
(`PrometeoPlugin.java`) — il vecchio problema `DATA_VERSION` non è più rilevante:
ogni APK con asset diversi rinfresca i KG nel dataDir. Il `.bin` resta seed-once.

**TODO vostra (sblocca tutto sul device):**
1. `cd Tsunami && npx cap sync android` → build/installa l'APK (porta il nuovo
   `.so` + asset). Da quel momento gli endpoint P1/P2 e il fix P3 sono live.
2. Cablare `POST /api/persist` su `onPause`/`onStop` (persiste SpeakerProfile).
3. Cablare `POST /api/comprehend` per il task_type del Mental Inbox (stateless).
4. `GET /api/speaker` per la memoria del companion ("come ti vedo").
