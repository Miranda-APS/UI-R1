#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
curate_kg_procedurale.py — KG procedurale di UI-r1.

Phase 79 (refactor strutturale). Quest'area del KG è SEPARATA da
`prometeo_kg.json` (semantico). Stessa struttura dati (KnowledgeGraph), stessa
API di query, stesso formato JSON. Diversa funzione: qui vivono i pattern del
FARE — grammatica, ruoli sintattici, atti di parola, conoscenza del COME —
invece dei fatti del mondo.

═══════════════════════════════════════════════════════════════════════════
CONVENZIONI INVIOLABILI
═══════════════════════════════════════════════════════════════════════════

1. **Ogni nodo è UNA SOLA PAROLA** del dizionario italiano.
   - Niente underscore, niente trattini, niente composti
   - Concetti composti si esprimono con MULTIPLE relazioni:
     `cosa IsA pronome` E `cosa IsA interrogativo`

2. **Le 21 relazioni del KG semantico** (RelationType in Rust):
   IsA, Has, Does, PartOf, Causes, Enables, Requires, TransformsInto,
   SimilarTo, OppositeOf, UsedFor, Expresses, Symbolizes, ContextOf,
   FeelsAs, WondersAbout, RemembersAs, Implies, Equivalent,
   Excludes, Coexists.
   Niente nuove relazioni: il KG procedurale ne usa un sottoinsieme.

3. **`via` per scopare il ruolo della relazione**:
   `cosa UsedFor chiedere VIA oggetto` significa: "cosa è usato per
   chiedere quando ciò che manca è l'oggetto". Il via permette al
   pattern matcher di trovare la parola giusta per il ruolo specifico.

4. **Vocabolario coerente con kg_sem**: i concetti che compaiono come
   nodi (vuoto, posizione, saluto, identità, completamento, ecc.) sono
   parole italiane atomiche già presenti nel kg_sem. Il kg_proc estende
   il loro uso con relazioni grammaticali/pragmatiche, ma non inventa
   parole. I QUALIFICATORI puri (cognitivo, percettivo, modale) sono
   metalinguaggio della categorizzazione, vivono solo qui.

═══════════════════════════════════════════════════════════════════════════
ARCHITETTURA DELLA CONOSCENZA PROCEDURALE
═══════════════════════════════════════════════════════════════════════════

Quattro livelli:

A. **CATEGORIE GRAMMATICALI** (i tipi di parole):
   - pronome, articolo, preposizione, marcatore, verbo, avverbio,
     congiunzione, interiezione

B. **SOTTOCATEGORIE** (raffinamento via IsA multipli):
   - pronome × interrogativo / personale / dimostrativo / possessivo / riflessivo
   - articolo × determinativo / indeterminativo
   - marcatore × interrogativo / esclamativo / dichiarativo
   - verbo × copula / azione / stato / movimento / cognitivo / comunicativo / percettivo / denominativo
   - preposizione × semplice / articolata
   - avverbio × modale
   Ogni parola riceve TUTTI gli IsA che le si addicono.

C. **PATTERN** (gli atti compositivi — i nodi che la voce istanzia):
   - articolazione   — chiedere ciò che il parlante non ha articolato
   - identificazione — rispondere a "chi sei?"
   - ricambio        — atto fatico simmetrico (saluto, ecc.)
   - asserzione      — affermare qualcosa nel mondo
   - presentazione   — "mi chiamo X"
   - riconoscimento  — restituire il claim del parlante
   - posizionamento  — rispondere da una propria prospettiva
   - specchio        — verificare la comprensione
   - esplorazione    — domandare per curiosità (non rincorre vuoto)
   - esitazione      — esprimere incertezza epistemica
   Ogni pattern è un nodo con `IsA pattern` + `UsedFor X via Y` (la sua
   pertinenza) + `Requires <ruolo> via <funzione>` per ogni slot.

D. **PERCETTI** (gli stati del campo che attivano i concetti):
   Nodi-singolo-parola che il `ComprehensionReport` può seminare
   nel campo del kg_proc. Ognuno `Causes` i concetti che il pattern
   appropriato richiama. Il `confidence` della relazione Causes è il
   peso di attivazione (gain percettivo, esplicito perché curato).
   Esempi: `saluto Causes restituire` (0.7), `chiusura Causes posizione`
   (0.5). Il pattern `riconoscimento` (UsedFor restituire via=posizione)
   risuona naturalmente con questi due segnali.

═══════════════════════════════════════════════════════════════════════════
COME ESTENDERE
═══════════════════════════════════════════════════════════════════════════

- **Nuova parola di una categoria esistente**: aggiungi le triple
  IsA/UsedFor sotto la sezione della categoria.

- **Nuovo pattern grammaticale**: aggiungi il nodo nuovo con `IsA pattern`,
  `UsedFor X VIA Y` (la sua pertinenza), e un `Requires <ruolo> VIA <scopo>`
  per ogni slot. La voce lo userà automaticamente — niente Rust nuovo.

- **Nuovo percetto**: aggiungi `<parola> IsA percetto` + le triple
  `<parola> Causes <concetto> (peso)` per ogni concetto attivato.
  Il bridge Rust legge le proprietà del ComprehensionReport e mappa
  ognuna al percetto corrispondente.

═══════════════════════════════════════════════════════════════════════════
USO
═══════════════════════════════════════════════════════════════════════════

  python curate_kg_procedurale.py            # rigenera da zero
  python curate_kg_procedurale.py --dry-run  # vedi cosa farebbe

Idempotente: rigenera SEMPRE da zero (non c'è storico orfano).
"""

import json
import sys
from pathlib import Path

KG_PATH = Path("prometeo_kg_procedurale.json")
DRY_RUN = "--dry-run" in sys.argv

# ══════════════════════════════════════════════════════════════════════════
# Rigenera da zero — niente storico orfano
# ══════════════════════════════════════════════════════════════════════════

edge_map = {}

def add(subject, relation, obj, strength=0.95, via=None):
    key = (subject, relation, obj, via)
    edge = {
        "subject": subject,
        "relation": relation,
        "object": obj,
        "confidence": strength,
        "source": "Curated",
    }
    if via:
        edge["via"] = via
    edge_map[key] = edge

# ══════════════════════════════════════════════════════════════════════════
# § A — CATEGORIE GRAMMATICALI BASE
# ══════════════════════════════════════════════════════════════════════════

for cat in ["pronome", "articolo", "preposizione", "marcatore",
            "verbo", "avverbio", "congiunzione", "interiezione"]:
    add(cat, "IsA", "categoria")

# Sottocategorie come qualificatori (concetti astratti che le parole
# concrete acquisiscono via IsA multipli).
for sub in ["interrogativo", "personale", "dimostrativo", "possessivo", "riflessivo",
            "determinativo", "indeterminativo",
            "esclamativo", "dichiarativo",
            "copula", "azione", "stato", "movimento",
            "cognitivo", "comunicativo", "percettivo", "denominativo",
            "semplice", "articolata", "modale"]:
    add(sub, "IsA", "qualificatore")

# ══════════════════════════════════════════════════════════════════════════
# § B — PRONOMI INTERROGATIVI
# ══════════════════════════════════════════════════════════════════════════

interrogativi = [
    ("cosa",   "oggetto"),
    ("che",    "oggetto"),
    ("chi",    "persona"),
    ("dove",   "luogo"),
    ("quando", "tempo"),
    ("perché", "causa"),
    ("come",   "modo"),
    ("quale",  "scelta"),
    ("quanto", "misura"),
]
for pron, ruolo in interrogativi:
    add(pron, "IsA", "pronome")
    add(pron, "IsA", "interrogativo")
    add(pron, "UsedFor", "chiedere", via=ruolo)

# ══════════════════════════════════════════════════════════════════════════
# § C — PRONOMI PERSONALI E RIFLESSIVI
# ══════════════════════════════════════════════════════════════════════════

personali = [
    ("io",   "soggetto",     "prima"),
    ("tu",   "destinatario", "seconda"),
    ("lui",  "soggetto",     "terza"),
    ("lei",  "soggetto",     "terza"),
    ("noi",  "soggetto",     "prima"),
    ("voi",  "destinatario", "seconda"),
    ("loro", "soggetto",     "terza"),
]
for pron, ruolo, persona in personali:
    add(pron, "IsA", "pronome")
    add(pron, "IsA", "personale")
    add(pron, "UsedFor", "indicare", via=ruolo)
    add(pron, "Has", persona)

# Pronomi riflessivi (per il pattern presentazione: "mi chiamo X")
riflessivi = [
    ("mi", "prima"),
    ("ti", "seconda"),
    ("si", "terza"),
    ("ci", "prima"),
    ("vi", "seconda"),
]
for pron, persona in riflessivi:
    add(pron, "IsA", "pronome")
    add(pron, "IsA", "riflessivo")
    add(pron, "Has", persona)

# ══════════════════════════════════════════════════════════════════════════
# § D — ARTICOLI
# ══════════════════════════════════════════════════════════════════════════

for art in ["il", "lo", "la", "i", "gli", "le", "l"]:
    add(art, "IsA", "articolo")
    add(art, "IsA", "determinativo")

for art in ["un", "uno", "una"]:
    add(art, "IsA", "articolo")
    add(art, "IsA", "indeterminativo")

# ══════════════════════════════════════════════════════════════════════════
# § E — PREPOSIZIONI
# ══════════════════════════════════════════════════════════════════════════

prep_semplici = [
    ("di",  "specificazione"),
    ("a",   "destinazione"),
    ("da",  "origine"),
    ("in",  "luogo"),
    ("con", "compagnia"),
    ("su",  "argomento"),
    ("per", "scopo"),
    ("tra", "relazione"),
    ("fra", "relazione"),
]
for prep, ruolo in prep_semplici:
    add(prep, "IsA", "preposizione")
    add(prep, "IsA", "semplice")
    # Tag funzionale: la preposizione "di" è ANCHE classificata come
    # `IsA specificazione`, così il pattern matcher la trova quando uno
    # slot richiede `preposizione via=specificazione` (lookup uniforme con
    # i sub-tipi). Senza questo tag il pattern_matcher cadeva in fallback
    # e prendeva una preposizione qualunque (es. "su" per riconoscimento).
    add(prep, "IsA", ruolo)
    add(prep, "UsedFor", "introdurre", via=ruolo)
    # Anche il qualificatore meta-tag, se non è già stato aggiunto altrove
    add(ruolo, "IsA", "qualificatore")

prep_articolate = [
    ("del", "di", "il"), ("della", "di", "la"), ("dei", "di", "i"),
    ("degli", "di", "gli"), ("delle", "di", "le"),
    ("al", "a", "il"), ("alla", "a", "la"), ("ai", "a", "i"),
    ("agli", "a", "gli"), ("alle", "a", "le"),
    ("dal", "da", "il"), ("dalla", "da", "la"),
    ("nel", "in", "il"), ("nella", "in", "la"),
    ("sul", "su", "il"), ("sulla", "su", "la"),
]
for art, base, articolo in prep_articolate:
    add(art, "IsA", "preposizione")
    add(art, "IsA", "articolata")
    add(art, "Equivalent", base, via=articolo)

# ══════════════════════════════════════════════════════════════════════════
# § F — MARCATORI DISCORSIVI
# ══════════════════════════════════════════════════════════════════════════

add("interrogativo", "IsA", "marcatore")
add("interrogativo", "Causes", "domanda")
add("interrogativo", "Expresses", "richiesta")

add("esclamativo", "IsA", "marcatore")
add("esclamativo", "Causes", "esclamazione")
add("esclamativo", "Expresses", "intensità")

add("dichiarativo", "IsA", "marcatore")
add("dichiarativo", "Causes", "asserzione")

# ══════════════════════════════════════════════════════════════════════════
# § G — VERBI BASE (slot-filler dei pattern)
# ══════════════════════════════════════════════════════════════════════════

# Copule
add("essere", "IsA", "verbo")
add("essere", "IsA", "copula")
add("essere", "UsedFor", "predicare", via="identità")
add("essere", "UsedFor", "predicare", via="stato")

add("avere", "IsA", "verbo")
add("avere", "IsA", "copula")
add("avere", "UsedFor", "predicare", via="possesso")
add("avere", "UsedFor", "predicare", via="stato")

add("stare", "IsA", "verbo")
add("stare", "IsA", "copula")
add("stare", "UsedFor", "predicare", via="condizione")

add("fare", "IsA", "verbo")
add("fare", "IsA", "copula")  # ausiliare in molte locuzioni ("fare fatica", "fare male")
add("fare", "UsedFor", "predicare", via="azione")

# Verbi percettivi (per il pattern di riconoscimento)
add("sentire", "IsA", "verbo")
add("sentire", "IsA", "azione")
add("sentire", "IsA", "percettivo")
add("sentire", "UsedFor", "percepire")

add("vedere", "IsA", "verbo")
add("vedere", "IsA", "azione")
add("vedere", "IsA", "percettivo")
add("vedere", "UsedFor", "percepire")

add("ricevere", "IsA", "verbo")
add("ricevere", "IsA", "azione")
add("ricevere", "UsedFor", "accogliere")

# Verbi denominativi (per il pattern di presentazione)
add("chiamare", "IsA", "verbo")
add("chiamare", "IsA", "azione")
add("chiamare", "IsA", "denominativo")
add("chiamare", "UsedFor", "denominare")

add("nominare", "IsA", "verbo")
add("nominare", "IsA", "azione")
add("nominare", "IsA", "denominativo")
add("nominare", "UsedFor", "denominare")

# Verbi cognitivi (per esitazione + posizionamento)
add("pensare", "IsA", "verbo")
add("pensare", "IsA", "cognitivo")
add("pensare", "UsedFor", "esprimere", via="ipotesi")

add("credere", "IsA", "verbo")
add("credere", "IsA", "cognitivo")
add("credere", "UsedFor", "esprimere", via="ipotesi")

add("sembrare", "IsA", "verbo")
add("sembrare", "IsA", "cognitivo")
add("sembrare", "UsedFor", "esprimere", via="apparenza")

add("sapere", "IsA", "verbo")
add("sapere", "IsA", "cognitivo")
add("sapere", "UsedFor", "esprimere", via="conoscenza")

# Verbi comunicativi (per esplorazione + specchio)
add("chiedere", "IsA", "verbo")
add("chiedere", "IsA", "comunicativo")
add("chiedere", "UsedFor", "domandare", via="curiosità")

add("dire", "IsA", "verbo")
add("dire", "IsA", "comunicativo")
add("dire", "UsedFor", "esprimere", via="affermazione")

add("ascoltare", "IsA", "verbo")
add("ascoltare", "IsA", "comunicativo")
add("ascoltare", "UsedFor", "ricevere", via="parola")

add("intendere", "IsA", "verbo")
add("intendere", "IsA", "cognitivo")
add("intendere", "UsedFor", "verificare", via="comprensione")

# ══════════════════════════════════════════════════════════════════════════
# § H — AVVERBI MODALI (per esitazione)
# ══════════════════════════════════════════════════════════════════════════

add("forse", "IsA", "avverbio")
add("forse", "IsA", "modale")
add("forse", "Expresses", "incertezza")
add("forse", "UsedFor", "marcare", via="ipotesi")

add("probabilmente", "IsA", "avverbio")
add("probabilmente", "IsA", "modale")
add("probabilmente", "Expresses", "probabilità")
add("probabilmente", "UsedFor", "marcare", via="ipotesi")

add("certamente", "IsA", "avverbio")
add("certamente", "IsA", "modale")
add("certamente", "Expresses", "certezza")
add("certamente", "UsedFor", "marcare", via="affermazione")

add("forse", "OppositeOf", "certamente")

# ══════════════════════════════════════════════════════════════════════════
# § H.bis — CONGIUNZIONI (parole di legame, function-word per natura)
# ══════════════════════════════════════════════════════════════════════════

congiunzioni = [
    ("e",  "additiva"),
    ("o",  "disgiuntiva"),
    ("ma", "avversativa"),
    ("se", "ipotetica"),
    ("perché", "causale"),  # già pronome interrogativo, anche congiunzione
    ("che", "subordinante"),  # già pronome, anche congiunzione subordinante
]
for cong, ruolo in congiunzioni:
    add(cong, "IsA", "congiunzione")
    add(cong, "UsedFor", "legare", via=ruolo)

# ══════════════════════════════════════════════════════════════════════════
# § I — PATTERN GRAMMATICALI (gli atti compositivi)
# ══════════════════════════════════════════════════════════════════════════
# Ogni pattern è un nodo con:
#  - IsA pattern (così select_pattern li trova)
#  - IsA atto (metaconoscenza)
#  - UsedFor <azione> via <ruolo>  ← la sua pertinenza/posizione nel campo
#  - Requires <ruolo> via <funzione>  ← per ogni slot da riempire

# ── articolazione: "Di cosa hai paura?" ──
add("articolazione", "IsA", "pattern")
add("articolazione", "IsA", "atto")
add("articolazione", "UsedFor", "chiedere", via="vuoto")
add("articolazione", "Causes", "domanda")
add("articolazione", "Requires", "pronome",      via="interrogativo")
add("articolazione", "Requires", "preposizione", via="specificazione")
add("articolazione", "Requires", "verbo",        via="predicato")
add("articolazione", "Requires", "marcatore",    via="interrogativo")

# ── identificazione: "Sono un'entità." ──
add("identificazione", "IsA", "pattern")
add("identificazione", "IsA", "atto")
add("identificazione", "UsedFor", "rispondere", via="identità")
add("identificazione", "Requires", "pronome", via="personale")
add("identificazione", "Requires", "verbo",   via="copula")
add("identificazione", "Requires", "predicato", via="identità")

# ── ricambio: atto fatico (saluto/congedo) ──
add("ricambio", "IsA", "pattern")
add("ricambio", "IsA", "atto")
add("ricambio", "UsedFor", "restituire", via="saluto")
add("ricambio", "Requires", "parola", via="classe")

# ── asserzione: "Il fuoco causa calore." ──
add("asserzione", "IsA", "pattern")
add("asserzione", "IsA", "atto")
add("asserzione", "UsedFor", "affermare")
add("asserzione", "Requires", "soggetto")
add("asserzione", "Requires", "verbo",     via="predicato")
add("asserzione", "Requires", "predicato", via="oggetto")

# ── presentazione: "Mi chiamo X." ──
add("presentazione", "IsA", "pattern")
add("presentazione", "IsA", "atto")
add("presentazione", "UsedFor", "introdurre", via="nome")
add("presentazione", "Requires", "pronome", via="riflessivo")
add("presentazione", "Requires", "verbo",   via="denominativo")
add("presentazione", "Requires", "nome",    via="proprio")

# ── riconoscimento: "Hai paura." (eco lacaniano del posizionamento) ──
add("riconoscimento", "IsA", "pattern")
add("riconoscimento", "IsA", "atto")
add("riconoscimento", "UsedFor", "restituire", via="posizione")
add("riconoscimento", "Requires", "pronome",      via="personale")
add("riconoscimento", "Requires", "verbo",        via="percettivo")
add("riconoscimento", "Requires", "predicato",    via="stato")
add("riconoscimento", "Requires", "preposizione", via="specificazione")

# ── posizionamento: "Per me è X." (rispondere da una propria prospettiva) ──
add("posizionamento", "IsA", "pattern")
add("posizionamento", "IsA", "atto")
add("posizionamento", "UsedFor", "rispondere", via="prospettiva")
add("posizionamento", "Requires", "preposizione", via="prospettiva")
add("posizionamento", "Requires", "pronome",      via="personale")
add("posizionamento", "Requires", "verbo",        via="copula")
add("posizionamento", "Requires", "predicato",    via="oggetto")

# ── specchio: "Intendi X?" (verificare la comprensione) ──
add("specchio", "IsA", "pattern")
add("specchio", "IsA", "atto")
add("specchio", "UsedFor", "verificare", via="comprensione")
add("specchio", "Causes", "chiarimento")
add("specchio", "Requires", "pronome",   via="destinatario")
add("specchio", "Requires", "verbo",     via="cognitivo")
add("specchio", "Requires", "oggetto",   via="oggetto")
add("specchio", "Requires", "marcatore", via="interrogativo")

# ── esplorazione: "Come?" (curiosità genuina di UI-r1) ──
add("esplorazione", "IsA", "pattern")
add("esplorazione", "IsA", "atto")
add("esplorazione", "UsedFor", "domandare", via="curiosità")
add("esplorazione", "Causes", "domanda")
add("esplorazione", "Requires", "pronome",    via="interrogativo")
add("esplorazione", "Requires", "verbo",      via="comunicativo")
add("esplorazione", "Requires", "marcatore",  via="interrogativo")

# ── esitazione: "Forse [ipotesi]." (incertezza epistemica esplicita) ──
add("esitazione", "IsA", "pattern")
add("esitazione", "IsA", "atto")
add("esitazione", "UsedFor", "esprimere", via="incertezza")
add("esitazione", "Requires", "avverbio",  via="modale")
add("esitazione", "Requires", "verbo",     via="cognitivo")
add("esitazione", "Requires", "predicato", via="ipotesi")

# ══════════════════════════════════════════════════════════════════════════
# § J — PERCETTI (gli stati del campo che attivano i concetti)
# ══════════════════════════════════════════════════════════════════════════
# Ogni percetto è un nodo singolo-parola, italiano, già coerente con kg_sem.
# `Causes` con confidence=peso significa: la percezione di X attiva il
# concetto Y nel campo del kg_proc al peso indicato.
#
# Il bridge Rust legge proprietà tipizzate del ComprehensionReport (saluto?
# closure? gap aperto?) e semina il percetto corrispondente. Il pattern
# matcher poi sceglie il pattern che risuona meglio con il campo seminato.

# ── saluto (atto fatico in entrata) ──
add("saluto", "IsA", "percetto")
add("saluto", "Causes", "restituire", strength=0.7)
add("saluto", "Causes", "saluto",     strength=0.6)

# ── chiusura (closure di un vuoto aperto da UI-r1) ──
add("chiusura", "IsA", "percetto")
add("chiusura", "Causes", "restituire",    strength=0.7)
add("chiusura", "Causes", "posizione",     strength=0.5)
add("chiusura", "Causes", "completamento", strength=0.4)

# ── apertura (vuoto strutturale rilevato nel claim del parlante) ──
add("apertura", "IsA", "percetto")
add("apertura", "Causes", "chiedere", strength=0.7)
add("apertura", "Causes", "vuoto",    strength=0.5)

# ── domanda (interrogazione del parlante) ──
add("domanda", "IsA", "percetto")
add("domanda", "Causes", "rispondere", strength=0.7)
# subject=Self_ aggiunge identità via bridge Rust (informazione tipata)

# ── posizione (claim del parlante senza vuoto, da riconoscere) ──
add("posizione", "IsA", "percetto")
add("posizione", "Causes", "restituire", strength=0.4)
add("posizione", "Causes", "posizione",  strength=0.4)

# ── affermazione (asserzione sul mondo, senza claim attribuibile) ──
add("affermazione", "IsA", "percetto")
add("affermazione", "Causes", "affermare", strength=0.7)

# ── introduzione (mi chiamo X) ──
add("introduzione", "IsA", "percetto")
add("introduzione", "Causes", "introdurre", strength=0.7)
add("introduzione", "Causes", "nome",       strength=0.5)

# ── incertezza-mia (UI-r1 non è certa di quello che potrebbe dire) ──
# Soggetto = UI-r1; lo distingue il bridge Rust (osserva self_profile).
add("incertezza", "IsA", "percetto")
add("incertezza", "Causes", "esprimere",  strength=0.7)
add("incertezza", "Causes", "incertezza", strength=0.6)

# ── curiosità-mia (UI-r1 vuole sapere) ──
add("curiosità", "IsA", "percetto")
add("curiosità", "Causes", "domandare",  strength=0.7)
add("curiosità", "Causes", "curiosità",  strength=0.6)

# ══════════════════════════════════════════════════════════════════════════
# § K — META: ancore concettuali
# ══════════════════════════════════════════════════════════════════════════

add("pattern", "IsA", "struttura")
add("atto", "IsA", "azione")
add("categoria", "IsA", "classe")
add("qualificatore", "IsA", "classe")
add("percetto", "IsA", "stato")

# ══════════════════════════════════════════════════════════════════════════
# Salvataggio
# ══════════════════════════════════════════════════════════════════════════

edges = list(edge_map.values())
print(f"Triple generate: {len(edges)}")

# Statistica per relazione
from collections import Counter
rel_counts = Counter(e["relation"] for e in edges)
for rel, count in sorted(rel_counts.items(), key=lambda x: -x[1]):
    print(f"  {rel:15} {count:4}")

# Statistica per pattern
patterns = sorted([e["subject"] for e in edges
                   if e["relation"] == "IsA" and e["object"] == "pattern"])
print(f"\nPattern ({len(patterns)}): {', '.join(patterns)}")

# Statistica per percetto
percetti = sorted([e["subject"] for e in edges
                   if e["relation"] == "IsA" and e["object"] == "percetto"])
print(f"Percetti ({len(percetti)}): {', '.join(percetti)}")

if not DRY_RUN:
    kg = {"edges": edges}
    with open(KG_PATH, "w", encoding="utf-8") as f:
        json.dump(kg, f, ensure_ascii=False, indent=2)
    print(f"\nSalvato in {KG_PATH}")
else:
    print("\n[DRY RUN — nessuna modifica salvata]")
