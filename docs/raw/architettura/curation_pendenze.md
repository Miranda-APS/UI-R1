# Registro delle pendenze di CURATION (per sessione dedicata)

> Tutto ciò che richiede CURATION del KG (contenuto), non modifiche di codice.
> Tenuto aggiornato man mano che emergono buchi durante lo sviluppo. Quando
> facciamo la sessione di cura, questo è l'ordine del giorno. Ogni voce dice:
> cosa manca, come si manifesta, come si cura.
>
> Confine: qui SOLO buchi di *contenuto* (parole/relazioni del kg_sem/kg_proc).
> I buchi di *meccanismo* (estrazione, codice) NON stanno qui — stanno nei TODO
> di CLAUDE.md. Strumento di verifica: `python kg_lint.py`.

---

> **✅ P1 e P2 FATTE (2026-06-16b)** via `cura_cicli_e_composti.py` (idempotente,
> backup `prometeo_kg.json.bak_cura`): 6 cicli IsA spezzati + 25 nodi composti
> risolti + verbi-supertipo piovere/succedere (P3 parziale). `kg_lint.py` → **0
> errori** (E1/E3/E4/E6). Poi `rebuild-semantic-topology` dalla fonte ufficiale.
> NB su P3: il 🟡 di `pioverà`/`succederà` NON era curation (il supertipo c'era già)
> — è estrazione (verbo impersonale senza soggetto, non *legato*). Restano P4/P5/P6.

## ✅ P1 — Cicli IsA (FATTA 2026-06-16b — qui sotto la documentazione storica)

Trovati da `kg_lint.py` (E6). Un ciclo `a IsA b IsA … IsA a` rende l'ereditarietà
e il grounding tassonomico instabili. **Priorità massima.** I 6 cicli:

- `evento → storia → cultura → conoscenza → processo → evento`
- `azione → evento → storia → cultura → conoscenza → processo → sviluppo → azione`
- `processo → sviluppo → cambiamento → processo`
- `conflitto → scontro → conflitto`
- `linguaggio → geometria → matematica → linguaggio`
- `gruppo → insieme → gruppo`

**Cura**: spezzare ogni ciclo rimuovendo/ridirezionando l'arco IsA meno
difendibile (es. `cultura IsA conoscenza` è dubbio; `scontro IsA conflitto` ok ma
`conflitto IsA scontro` no — tenere una direzione sola).

## 🔴 P2 — Nodi non atomici (25, violano "una parola per nodo")

Trovati da `kg_lint.py` (E1). Es: `risolvere_problemi`, `capire_il_mondo`,
`corso_d_acqua`, `capire_l_altro`, `sentire_con`, `imitare_la_natura`,
`stato_emotivo`, `errore_sistematico`, `volontà_umana`, `sistema_esperto`…
**Cura**: scomporre in relazioni multiple (`risolvere_problemi` →
`intelligenza UsedFor risolvere` + `risolvere ContextOf problema`), o rimuovere
se ridondanti.

## 🟡 P3 — Verbi comuni senza supertipo-verbo (IsA azione/evento/…)

Si manifesta: un verbo viene riconosciuto morfologicamente ma resta 🟡 nel gate
("succederà·verbo" giallo, "pioverà·verbo" giallo) perché `is_verb_concept`
(catena IsA → {azione,atto,processo,evento,movimento,accadimento,attività,fare})
non lo raggiunge. Verbi noti mancanti: **piovere, succedere** (+ verificare:
servire, bastare, nascere, litigare, mancare già curati?).
**Cura**: aggiungere `<verbo> IsA {azione|evento|accadimento|…}` al kg_sem per i
verbi comuni. Sblocca il loro riconoscimento pieno (🟡→✅) senza toccare codice.

## 🟡 P4 — Emozioni senza scelta-di-valenza esplicita (W2 del lint)

Si manifesta: il gap "oggetto" si apre solo per le emozioni transitive
(`<em> Requires oggetto`, eredità inclusa). Emozioni di stato (tristezza,
solitudine) correttamente NON aprono gap. Ma alcune emozioni dovrebbero essere
transitive e non sono marcate → discoverability. `kg_lint.py` W2 le elenca.
**Cura**: per ogni emozione segnalata, DECIDERE consapevolmente: transitiva
(`Requires oggetto`) o stato-completo (niente). È una scelta, non un default.

## 🟡 P5 — Meta-edge di valenza che leccano nel grounding (layering)

Si manifesta(va): `<emozione> Requires oggetto` è dato PROCEDURALE (marca la
valenza per `derive_gaps`) ma vive nel kg_sem → poteva leccare nella voce
("La paura ha bisogno dell'oggetto"). Il grounding multi-candidato lo ha
mitigato (preferenza tassonomica), MA il dato resta ambiguo di strato.
**Decisione da prendere (Francesco)**: marcare i meta-edge (es. `via=meta` che
il collasso salta) oppure spostarli in un namespace separato. Non urgente
(non leccà più), ma è debito di layering.

## 🟡 P6 — "nascere da" = provenienza (relazione mancante/sbagliata)

Si manifesta: "la rabbia nasce dalla paura" → `World(rabbia) Does paura`
(sbagliato). "nascere da X" è provenienza/causa, dovrebbe dare
`rabbia Causes-da paura` o un `via=paura`. **Cura/decisione**: serve una
relazione di provenienza, o mappare "nascere da" → Causes inverso. (Confine:
metà codice metà dato — la costruzione "nascere da" va insegnata al kg_proc.)

---

## NON sono curation (buchi di meccanismo — stanno in CLAUDE.md, li chiudo io)

- Binding dell'oggetto nei frame progressivo/infinito ("sto cercando un
  **senso**", "per avviare il **motore**") — estensione dell'estrazione.
- "mia moglie non mi capisce" → PROP None (Mondo+3ª+clitico) — estrazione.
- Punteggiatura dell'articolazione ("?" perso) — render.
- "fa" (fare 3sg irregolare) non lemmatizzato in contesto aritmetico — minore.
