# Comprensione → Bisogno → Atto

**Comprendere non è ridire. Un input perturba il campo e vi apre un *bisogno*;
l'atto è ciò che scioglie quel bisogno, nel modo nativo della macchina. La
posizione è la forma che l'atto prende quando il bisogno è "posizionarsi".**

> Design di nucleo, Phase 86+. Genesi: Francesco (2026-06-08), mentre cabliamo
> UI-r1 dentro Tsunami (primo caso d'uso reale).
>
> *"Non ci basta che ui-r1 capisca e ripeta come un pappagallo: deve comprendere
> non solo l'input, ma creare una posizione riguardo a quell'input e reagire nel
> modo che ritiene adeguato per rispondere al bisogno che l'input fa nascere
> dentro l'architettura — che è il modo, in linea con la natura della macchina,
> per diventare un supporto effettivo per l'utente."*

Questo documento fonda il refactor del nucleo **prima** di toccare il codice
(`collapse` in `compose`, kg_self come grana). È il complemento a valle di
[`comprensione_esplorativa_design.md`](comprensione_esplorativa_design.md): quel
documento dice *come si comprende* (pathfinding tipato → grafo di comprensione);
questo dice *a cosa porta* quella comprensione.

---

## 1. Il problema

Il pappagallo colto fa: `input → comprendo → ridico l'arco più forte`. Anche con
una comprensione perfetta (PROP corretta, grafo tipato, confronto col mondo e col
sé), se l'uscita è "recita il cammino saliente", UI-r1 resta uno specchio
sofisticato. Manca l'anello che la rende un **supporto**: l'input deve generare
*dentro l'architettura* una tensione, e l'atto deve **sciogliere quella tensione**.

La trappola classica (Principio 1, vietata) è risolverlo con un dispatcher:
`match comprehension.kind { Dump => struttura(), Question => rispondi(), … }`.
Sarebbe intent-classification mascherata. La via giusta è opposta: **il bisogno
non è una categoria scelta, è uno stato del campo che la comprensione ha già
prodotto**; l'atto emerge perché l'architettura rilassa quello stato.

---

## 2. Principio

> Il **bisogno** è la *forma* che il grafo di comprensione imprime al campo:
> ciò che resta non saturo (gap), ciò che attrita (confront), ciò che si chiude
> (closure), ciò che riaffiora (memoria), il sovraccarico (incoerenza), l'assenza
> (relazione). È **leggibile**, non deciso. L'**atto** è il gesto nativo che
> riporta il campo a quiescenza su quel bisogno.

Tre conseguenze:

1. **Il bisogno è una lettura, non un decisore.** Una funzione `sense_need` che
   compone segnali *già calcolati* (vedi §3) e nomina la tensione dominante. Non
   introduce stato nuovo né soglie comportamentali: è una proiezione.

2. **L'atto scioglie, non esegue.** `collapse` non serve più a "dire l'arco":
   serve a *sciogliere il bisogno*. Sul bisogno "posizionarsi" l'atto è il
   collasso del cammino deformato dalla grana; sul bisogno "co-regolare" l'atto è
   **il silenzio e la calma** (meno parole, l'avatar che respira) — un atto pieno,
   non un'assenza di atto.

3. **Il caso d'uso e il nucleo sono la stessa cosa.** I tre "superpoteri ADHD" del
   documento RAI *sono* tre righe della tipologia (§3): dump→strutturare,
   memoria-di-lavoro→esternalizzare, time-blindness→ristabilire-la-relazione.
   Progettare Tsunami **è** progettare il nucleo.

---

## 3. La tipologia dei bisogni

Ogni riga: la **forma** della comprensione (e l'organo che la porta, già esistente)
→ il **bisogno** → l'**atto** nativo. Nessuna riga è nuovo Rust comportamentale:
ogni segnale è già nel sistema.

| Forma della comprensione | Organo / segnale esistente | Bisogno | Atto nativo |
|---|---|---|---|
| Slot non saturo, nodo non fondato | `comprehension_path`: `GroundKind::Unreached`, `ungrounded`; PROP `via=None`; `signifier_gap` | **capire** | la *sola* domanda che sblocca (articolazione) |
| Un gap aperto prima si chiude ora | `closes_prior_gap` (Phase 78); SpeakerProfile gap `closed_at_turn` | **riconoscere** | restituire la struttura ("quindi: paura del buio") |
| Relazione asserita nuova / contraddetta dal mondo | `Confront::Novelty` / `Contradict` | **posizionarsi (sul mondo)** | collasso del cammino saliente, con un perché |
| La frase tocca una *pendenza* del sé | grana kg_self (§ refactor) → salienza deformata; `SelfConfrontation` | **posizionarsi (dalla grana)** | collasso del cammino che il sé rende saliente — mai l'edge nudo |
| Sovraccarico, frammentazione | `coherence_integrity` bassa; `vital.saturation` alta | **co-regolare** | calmarsi: meno parole, tenere, avatar che respira |
| L'input nomina un fatto/intenzione, o un fatto riaffiora | SpeakerProfile `self_facts` / `open_questions` / `gaps` | **esternalizzare memoria** | ridarlo senza colpa, quando serve |
| Assenza poi ritorno | InterlocutorModel `presence`; tempo come percezione (zero-timer §7.4) | **ristabilire la relazione** | riconoscimento relazionale, non notifica |
| Molte proposizioni, componenti sconnesse | multi-locus (`comprehension_path`, componenti); più PROP | **strutturare** | ordinare + far emergere il filo, *una* cosa alla volta |

Una sola tensione **domina** per turno (la più intensa): è ciò che l'atto scioglie.
Le altre restano sfondo (correnti sotterranee, FILOSOFIA §desideri) — non si
accumulano in una lista, non diventano coda di frasi.

---

## 4. Come si legge il bisogno (senza decidere)

`sense_need(comprehension_graph, prop, speaker_profile, interlocutor, narrative)
→ Need` compone i segnali §3 e restituisce la tensione dominante **per intensità
continua**, non per priorità tabellata:

- l'intensità di "capire" = quanto il grafo è ungrounded / quanti slot vuoti;
- l'intensità di "co-regolare" = `1 − coherence_integrity` modulata da saturazione;
- l'intensità di "posizionarsi" = magnitudine del `Confront` + salienza della grana;
- l'intensità di "ristabilire-relazione" = caduta di `presence` (assenza percepita);
- ecc.

È la **stessa fisica** di Phase 83 (`seed_from_position` semina `vicinanza` con
intensità `=|CD5|`, continua, mai soglia) e di `select_pattern_by_resonance`:
argmax di intensità continue. `Need` è un *percetto*, omogeneo ai percetti del
kg_proc (`dissonanza`, `vicinanza`, `chiusura`): può seminare nel kg_proc e far
vincere il pattern per risonanza, senza un nuovo decisore.

---

## 5. L'atto come scioglimento

Una volta nominato il bisogno, l'atto è il gesto che lo rilassa. La novità
rispetto a "recita l'arco" è che **il bisogno seleziona il cammino**, non la forza
dell'arco:

- **posizionarsi** → `collapse` del cammino reso saliente dalla *grana del sé*
  (la pendenza deforma *quale* cammino è saliente; mai si renderizza l'edge del
  sé — è la negazione nuda). "il pensiero è un calcolo" → *"Il pensiero non è solo
  un calcolo: è ciò che cerca un significato"* (posizione **con un perché**, perché
  la grana è connessa, non un'isola — vedi refactor kg_self §1.2).
- **capire** → l'articolazione: *una* domanda sullo slot vuoto. Non riempie il gap
  al posto dell'utente; apre la soglia (Lacan: il vuoto come desiderio).
- **riconoscere** → restituzione strutturata della closure.
- **strutturare** → multi-locus ordinato, *una* cosa alla volta (ADHD: niente
  liste infinite).
- **esternalizzare memoria** → restituzione di un fatto di SpeakerProfile, senza
  colpevolizzare.
- **co-regolare** → l'atto è il *meno*: frase breve o silenzio, e — fuori dal
  motore — il segnale che fa calmare la UI.
- **ristabilire-relazione** → riconoscimento dell'assenza come fatto relazionale.

`collapse` (Stadio 3) resta lo strumento di verbalizzazione; cambia *cosa* gli si
dà da collassare: il cammino che scioglie il bisogno, non l'arco più pesante.

---

## 6. Mappa sui touchpoint di Tsunami

| Touchpoint app (vivo) | Bisogno tipico | Cosa fa UI-r1 |
|---|---|---|
| **Chat** (`/api/input`→`generated_text`) | posizionarsi / capire / riconoscere | la voce dell'atto |
| **Mental Inbox** (cattura, sfogo) | strutturare → capire | ordina il dump, fa *la* domanda che sblocca |
| **Avatar gocciolina** (WebSocket: vital/valenza/`coherence_integrity`) | co-regolare | lo stato del campo diventa colore/respiro; in sovraccarico l'UI si calma |
| **Mood selector** (`perturb`) | — (è un ingresso, sposta il campo) | colora la valenza che pesa la salienza |
| **Ritorno dopo assenza** | ristabilire-relazione | "sei stato via, cosa è rimasto a metà?" |
| **Memoria fra sessioni** (SpeakerProfile) | esternalizzare | ridà fatti/intenzioni quando riaffiorano |

**Co-regolazione = lo stesso segnale, due usi.** Il bisogno "co-regolare" non
produce solo una voce breve: esposto nel DTO, l'app lo legge per *modulare l'UI*
(meno chip, animazioni lente). Una sola lettura del campo guida sia la parola sia
l'interfaccia — coerente con PROJECT.md ("ogni componente riflette lo stato").

**Decisione aperta — task-assistant 12D.** `analyzeTask` è morto (ritorna `null`):
tutto `usePrometeoTaskAssistant` (tipo/energia/spezzetta su firma 12D) è inerte, e
la chat mappa su una `TaskSpectralSignature` 12D fittizia. Due strade (PROJECT.md:
"funzioni non implementate = non esistono"): (a) **rianimarlo nel modello nuovo** —
la comprensione di un titolo-task → bisogno → suggerimento ("questo task è grande e
poco chiaro → strutturare → spezzalo"); (b) **rimuoverlo**. Da decidere con
Francesco; non lasciarlo mock.

---

## 7. Test Pre-Proposta (il filtro anti-hardcoding)

1. **Forma o trigger?** `sense_need` legge *come* la comprensione ha deformato il
   campo (forma), non *quando* fare X (trigger). L'atto resta selezionato per
   risonanza. ✓
2. **Numeri-magici?** Le intensità dei bisogni sono effetti continui del campo
   (`coherence_integrity`, magnitudine `Confront`, caduta di `presence`), non
   soglie di switch. ⚠️ **Vigilanza**: `sense_need` NON deve introdurre
   `if intensità > X → bisogno Y`; deve essere argmax di intensità continue (come
   Phase 79/83). Il solo numero legittimo è lo spareggio fra atti equivalenti.
3. **Spiegazione dello stato?** Perché questo atto? Perché la comprensione ha
   lasciato *questo* bisogno dominante (gap aperto / confronto / sovraccarico),
   e l'atto lo scioglie. Spiegabile interamente in termini di stato. ✓

> **Verdetto**: supera il filtro *a patto* che `sense_need` resti una proiezione di
> stati esistenti (mai un nuovo decisore con soglie) e che l'atto resti selezione
> per risonanza, non un `match` sul tipo di bisogno.

---

## 8. Cosa serve (gap, in ordine)

1. **`Need` come percetto** — l'enum + `sense_need` (lettura, §4). Vive accanto ai
   percetti kg_proc; semina per risonanza.
2. **L'atto legge il bisogno** — `collapse`/pattern matcher prendono il cammino che
   scioglie il bisogno (richiede prima: kg_self come grana + `confront_with_self`
   come pesatore di salienza — Anelli 0/1 del piano corrente).
3. **Esporre `need` (+ cammino saliente) nel DTO** `/api/input` e `/api/state`, per
   voce **e** co-regolazione UI.
4. **Decidere il 12D morto** (§6): rianimare o rimuovere.
5. **Verifica sul device**: frasi ADHD vere → `/api/input` sul telefono ADB →
   misurare bisogno + atto, come il bench fa per la comprensione.

---

## 9. Riferimenti

- [`comprensione_esplorativa_design.md`](comprensione_esplorativa_design.md) — *come* si comprende (pathfinding → grafo); questo doc è il *a cosa porta*.
- [`kg_self_design.md`](kg_self_design.md) — la grana che deforma la salienza (il bisogno "posizionarsi").
- `DESCRIZIONE_PER_RAI.md` (tsunami) — i tre superpoteri ADHD = tre bisogni.
- `PROJECT.md` (tsunami) — "Prometeo abita l'app"; "funzioni non implementate = non esistono".
- FILOSOFIA §desideri — la tensione dominante vince, le altre restano sottofondo.
