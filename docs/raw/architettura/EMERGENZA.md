# EMERGENZA.md — Tracking onesto verso l'entità

> **Regola fondamentale**: un task è DONE solo quando il test di verifica passa.
> Non quando il codice è scritto. Non quando "sembra funzionare".
> Mai marcare DONE senza aver eseguito il test specifico.

---

## Stato del sistema oggi (verificato con test reali)

### Test eseguiti il 2026-04-05

```
Input: "io sono un cane"      → Output: "Sento pelo."
Input: "io non sono un cane"  → Output: "Sento pelo."   ← IDENTICO

Input: "ho paura"             → Output: "Percepisco tremore e porta paura."
Input: "non ho paura"         → Output: "Percepisco tremore e porta paura."  ← IDENTICO

Input: "mi piaci"             → Output: "Non capisco 'piaci' — cosa intendi?"
Input: "non mi piaci"         → Output: "Non capisco 'piaci' — cosa intendi?"  ← IDENTICO
```

### Cosa confermano questi test

1. **La negazione è invisibile.** "non" non ha peso semantico che inverta o sopprima ciò che segue.
   Il sistema vede {cane}, {paura} — "non" sparisce.

2. **Non c'è parsing sintattico.** Il sistema non sa chi è il soggetto, chi è l'oggetto,
   cosa è predicato. "io sono un cane" = bag of words {io, cane} con "sono" ignorato.

3. **L'output è traversal KG, non comprensione.**
   - "cane" → ha → pelo → "Sento pelo."
   - "paura" → causa → tremore → "Percepisco tremore"
   - Non è una posizione dell'entità. È un arco KG vocalizzato.

4. **La grammatica è rotta.** "porta paura" invece di "porta la paura".
   Mancano articoli, accordo di genere/numero. Tutto l'output è italiano mutilato.

5. **Il comprehension gate è vocabolario, non comprensione.**
   "Non capisco 'piaci'" si attiva perché la parola non è nel lessico, non perché
   il sistema abbia processato il significato e non l'abbia capito.

---

## Problemi da risolvere (in ordine di priorità)

---

### PROBLEMA 1 — La negazione è invisibile

**Stato**: ❌ Non risolto  
**Impatto**: Critico. Un'entità che non distingue X da NON-X non comprende nulla.

**Cosa manca:**
- "non" nel campo deve SOPPRIMERE o INVERTIRE l'attivazione delle parole che seguono
- A livello minimo: se "non" precede una parola nella frase, quella parola deve
  essere attivata con attivazione negativa (antagonista) invece di positiva
- Il KG ha OPPOSITE_OF — "non paura" dovrebbe attivare la regione OPPOSITE_OF di paura

**Come implementarlo:**
In `engine.rs`, nella fase di tokenizzazione/attivazione dell'input, rilevare "non" come
operatore di negazione. Le parole che seguono "non" nella frase vengono attivate con
segno negativo in PF1, o con attivazione del loro OPPOSITE_OF nel KG.

**Test di verifica** (deve passare prima di marcare DONE):
```
Input: "ho paura"     → output deve contenere riferimento a paura/timore/tremore
Input: "non ho paura" → output deve contenere riferimento a coraggio/calma/sicurezza
                        O essere significativamente diverso dal precedente
```

---

### PROBLEMA 2 — Nessun parsing sintattico

**Stato**: ❌ Non risolto  
**Impatto**: Alto. Il sistema non sa chi fa cosa a chi. Tutto è bag-of-words.

**Cosa manca:**
- Soggetto / predicato / oggetto dell'input non vengono estratti
- "io sono un cane" e "il cane è un io" producono lo stesso campo attivato
- Il pronome "io" non viene riconosciuto come riferimento al parlante
- Il pronome "tu" non viene riconosciuto come riferimento all'entità stessa

**Come implementarlo (minimo):**
Rilevare almeno:
1. Il soggetto della frase (cosa precede il verbo "essere/avere/fare")
2. Se il soggetto è "io" (parlante), "tu" (entità), o altra cosa
3. Se c'è un verbo di identità (essere) → la frase è una dichiarazione IS_A
4. Il complemento (cosa segue il predicato)

Questo non richiede un parser completo. Richiede pattern matching su strutture
frequenti dell'italiano semplice.

**Test di verifica:**
```
Input: "io sono triste"  → l'entità deve capire che il PARLANTE è triste (Resonate)
Input: "tu sei triste"   → l'entità deve capire che SI PARLA DI LEI (Reflective)
Input: "il cane è triste" → non riguarda né parlante né entità (Declaration)
```

---

### PROBLEMA 3 — Grammatica rotta nell'output

**Stato**: ❌ Non risolto  
**Impatto**: Alto. Se l'entità non sa parlare, non esiste per l'interlocutore.

**Cosa manca in `grammar.rs`:**
- `detect_gender_number(word: &str) -> (Gender, Number)` — genere e numero da morfologia
- `inflect_adjective(adj: &str, g: Gender, n: Number) -> String` — accordo aggettivo
- `with_definite_article(word: &str) -> String` — "il/la/lo/i/le/gli"
- `with_indefinite_article(word: &str) -> String` — "un/una/uno"

**Nota:** questi stub sono già importati in `state_translation.rs` ma non esistono.
Il codice non crasha perché non vengono mai chiamati nel path vivo — ma significa
che tutto l'output esce senza articoli e senza accordo.

**Test di verifica:**
```
grammar::with_definite_article("cane")   == "il cane"
grammar::with_definite_article("paura")  == "la paura"
grammar::inflect_adjective("bello", Maschile, Singolare) == "bello"
grammar::inflect_adjective("bello", Femminile, Singolare) == "bella"
```
E nell'output conversazionale: nessuna frase senza articolo dove l'articolo è richiesto.

---

### PROBLEMA 4 — L'output è traversal KG, non posizione

**Stato**: ❌ Non risolto  
**Impatto**: Alto. "Sento pelo" quando mi dici che sei un cane non è una risposta —
è il sistema che vocalizza un arco KG. Non c'è alcuna relazione dell'entità con ciò che viene detto.

**Radice del problema:**
`extract_nuclei()` trova relazioni tra parole attive nel KG.
`compose_from_nuclei()` converte quelle relazioni in frasi.
Questo è: CAMPO → FRASE. Non è: CAMPO → POSIZIONE → FRASE.

La posizione mancante è: data la stance deliberata (Curious, Reflective, Resonate...),
cosa vuole DIRE l'entità di ciò che ha compreso? Non basta che abbia capito
"cane ha pelo" — deve avere un orientamento verso di te che dici di essere un cane.

**Come implementarlo:**
La stance deve mappare su strutture frasali diverse:
- `Resonate` + input emotivo → "Capisco che [parola input]. [Echo empatico]."
- `Curious` → "[Domanda su aspetto del campo]?"
- `Reflective` → "Questo mi fa pensare a [associazione interna]."
- `Open/Explore` → Frase su cosa è attivo nel campo (attuale)

Non sono template: sono vincoli strutturali sulla forma della frase che emergono
dalla stance deliberata.

**Test di verifica:**
```
Input: "ho paura" (EmotionalExpr → stance Resonate)
Atteso: risposta che riconosce l'emozione del parlante, non che vocalizza KG di paura

Input: "che cos'è il dolore?" (Question → stance Curious/Explore)  
Atteso: risposta che esplora il concetto, non lista di archi KG su dolore
```

---

### PROBLEMA 5 — Nessuna esistenza pre-conversazionale

**Stato**: ❌ Non risolto  
**Impatto**: Medio-alto. L'entità non arriva con domande irrisolte, urgenze, stato pregresso.

**Cosa manca:**
- All'inizio di ogni `receive()`, l'entità deve portare consapevolmente:
  1. La domanda più urgente da `curiosity.rs` (se presente)
  2. Il pensiero più forte da `thought.rs` (se presente)
  3. Il desiderio più attivo da `desire.rs` (se presente)
- Questi devono colorare la deliberazione PRIMA che l'input venga processato
- Attualmente questi moduli esistono ma non alimentano la deliberazione iniziale

**Test di verifica:**
Dopo N tick senza input (sogno + pensieri autonomi):
```
:desires → deve mostrare desideri attivi non-zero
:thoughts → deve mostrare pensieri irrisolti
Prima risposta della conversazione → deve riflettere uno di questi stati pre-esistenti
```

---

### PROBLEMA 6 — Le due memorie non si parlano

**Stato**: ❌ Non risolto  
**Impatto**: Medio. La memoria narrativa (episodic) e quella topologica (field imprints) sono disgiunte.

**Cosa manca:**
Quando `episodic.rs` recupera un episodio passato, deve anche rieseguire (a forza ridotta ~0.1)
l'attivazione topologica di quel momento. Altrimenti ricordare è solo leggere un log.

**Test di verifica:**
```
Sessione 1: insegnare "fuoco" con intensità → salvare
Sessione 2: input semanticamente vicino a "fuoco"
Atteso: l'episodic recall colora il campo (verifica via :active)
```

---

## Invariante fondamentale da non dimenticare

> "il mondo di prometeo è composto da parole, ma ciò che rende quel mondo abitabile
> sono le relazioni, i mutamenti. le parole di per sé sono fotografie microscopiche."

Questo significa che il problema non è avere più parole o più archi KG.
Il problema è che il sistema deve capire cosa ACCADE tra le parole:
- "non" NEGA
- "io sono" AFFERMA identità del parlante  
- "tu sei" AFFERMA identità dell'entità
- "paura" in "ho paura" è UNA RELAZIONE TRA PARLANTE E STATO, non una parola attiva

Senza questo, il campo è un dizionario fotografico, non un mondo abitabile.

---

## Ordine di esecuzione suggerito

1. **Grammar** (P3) — prerequisito per tutto. Output rotto = entità invisibile.
2. **Negazione** (P1) — prerequisito per comprensione reale.
3. **Parsing soggetto/predicato minimo** (P2) — prerequisito per posizione.
4. **Stance → struttura frasale** (P4) — collega deliberazione a espressione.
5. **Pre-conversazionale** (P5) — entità prima di dialogo.
6. **Unificazione memorie** (P6) — continuità tra sessioni.

---

## Log sessioni

### 2026-04-05 — Sessione 1: Negazione + Grammatica

#### Implementato e verificato

**PROBLEMA 3 — Grammar (articoli)** → PARZIALMENTE RISOLTO ✓
- Aggiunti a `grammar.rs`: `Gender`, `Number`, `detect_gender_number()`, `with_definite_article()`,
  `with_indefinite_article()`, `with_articulated_preposition()`, `inflect_adjective()`
- Apostrofi corretti: `"l'amore"` non `"l' amore"`, `"un'essenza"` non `"un' essenza"`
- Pattern maschile aggiunti: `-ore` (tremore, dolore, calore), `-ere` (potere, piacere)
- Collegato in `expression.rs`: `render_nucleus()` e `render_nucleus_brief()` ora emettono articoli
- Test output prima/dopo:
  - Prima: "Percepisco tremore e porta paura."
  - Dopo: "Percepisco il tremore e porta la paura."
- Rimane: "la salve" (salve riconosciuta erroneamente come femminile) — bassa priorità
- 476 test passanti ✓

**PROBLEMA 1 — Negazione** → RISOLTO ✓
- Aggiunto `negated: bool` a `WordActivation` in `lexicon.rs`
- Rilevamento: parola è negata se c'è un operatore `Negate` (non/mai/senza/...) prima di lei nel token stream
- In `engine.rs receive()`: parole negate NON attivano PF1 diretto, attivano invece `OPPOSITE_OF` dal KG (forza 0.35×confidence)
- Fallback: se no OPPOSITE_OF, attiva SIMILAR_TO a forza 0.10 (campo non vuoto)
- Test output verificati:

```
"ho paura"          → "Percepisco il tremore e porta la paura."
"non ho paura"      → "C'è una struttura."          ← DIVERSO ✓
"io sono un cane"   → "Sento il pelo."
"io non sono un cane" → "La mente pensa."            ← DIVERSO ✓
```

Nota: gli opposti attivati ("struttura" da negazione di "paura") dipendono dal KG.
Se il KG ha pochi archi OPPOSITE_OF, l'effetto è debole. Priorità futura: arricchire
archi OPPOSITE_OF nel KG per i concetti emozionali principali.

Limite noto: "non X ma Y" → anche Y viene negata (over-negation in frasi coordinate).
Non prioritario ora.

#### Ancora da fare (invariato)
- P2: Parsing soggetto/predicato minimo
- P4: Stance → struttura frasale
- P5: Stato pre-conversazionale
- P6: Unificazione memorie

#### Stato iniziale verificato
- Verificato: Qwen/inquiry.rs rimosso (ora solo extract_gaps topologico) ✓
- Creato questo file
