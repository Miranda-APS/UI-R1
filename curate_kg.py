#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
curate_kg.py — File master di curation del KG di UI-r1.

Uso:
  python curate_kg.py            # applica tutto, salva prometeo_kg.json
  python curate_kg.py --dry-run  # mostra solo le modifiche senza salvare

Il file cresce sessione per sessione. È idempotente: può essere rieseguito
più volte senza duplicare archi. Il rebuild (import-kg + rebuild-semantic-
topology) va fatto solo a fine sessione di lavoro.

═══════════════════════════════════════════════════════════════════════════
ISTRUZIONI PER L'AGENTE SUCCESSIVO
═══════════════════════════════════════════════════════════════════════════

WORKFLOW:
  1. Leggi questo file per capire cosa è stato curato (§0-§20)
  2. Ispeziona le parole da curare: python -c "
       import json
       with open('prometeo_kg.json', encoding='utf-8') as f: kg=json.load(f)
       edges=kg['edges']
       rels=[e for e in edges if e['subject']=='PAROLA' or e['object']=='PAROLA']
       for e in sorted(rels, key=lambda x: x['relation']):
         v=f\"  {e['subject']} -{e['relation']}-> {e['object']} ({e.get('confidence',1):.2f})\"
         if e.get('via'): v+=f\" via:{e['via']}\"
         print(v)
     "
  3. Aggiungi §21, §22... PRIMA della riga "Applica e salva"
  4. Esegui: python curate_kg.py
  5. Rebuild: cargo run --release --bin rebuild-semantic-topology
  6. MAI: cargo run --release --bin import-kg (sovrascrive le curations!)
  7. Testa: printf "frase\\n:quit\\n" | ./target/release/dialogue_educator 2>/dev/null

STILE CURAZIONE:
  - Ogni parola va ispezionata PRIMA di scrivere le curations
  - remove() per archi garbage (typo, parole inglesi, non-sense)
  - force() per cambiare strength di archi esistenti
  - add() per nuovi archi con strength 0.70-0.95 e via=nodo_intermedio
  - "via" DEVE essere una parola semplice italiana che ESISTE nel KG come nodo
  - NO underscore (mai "stato_emotivo"), NO parole composte
  - Relazioni standard: IsA, Has, Does, PartOf, Causes, OppositeOf, SimilarTo,
    UsedFor, Requires (tutte riconosciute dal motore Rust)
  - Ogni parola dovrebbe avere: Causes, Does, Has, PartOf, Requires, OppositeOf

AREE GIÀ CURATE (§1-§20, ~180 parole):
  §0:  Pulizia programmatica (orphan OppositeOf, hub-excess, inglesi)
  §1:  parola, significato, voce, segno, linguaggio, senso, pensiero,
       comprensione, coscienza, identita, presenza, relazione, struttura,
       cambiamento, divenire
  §2:  tempo, spazio, movimento, azione
  §3:  io, tu, altro, noi
  §4:  mente, corpo
  §5:  dialogo, domanda, risposta, ascolto, essere
  §6:  paura, gioia, tristezza, rabbia, amore, dolore
  §7:  vita, morte, luce, buio, scopo
  §8:  memoria, esperienza, conoscenza, sapere, crescita
  §9:  incontro, confine, verità, speranza, dubbio, bellezza, natura, libertà
  §10: armonia, equilibrio, vuoto, pienezza, volontà, cielo, terra
  §11: acqua, fuoco, aria, suono, attesa, limite, scelta, possibilità
  §12: nascita, lutto, intenzione, attenzione, bisogno, comunità, separazione
  §13: immaginazione, creatività, idea, tensione, responsabilità, coraggio, rispetto
  §14: ciclo, sole, luna, ordine, caos, presente, passato, futuro, assenza
  §15: arte, musica, scrittura, percezione, intelligenza (cleanup)
  §16: sé, valore, etica, insegnamento, apprendimento, ombra
  §17: vedere, sentire, toccare, udire, vista, tatto, olfatto, gusto
  §18: fame, sete, sonno, riposo, respiro, malattia, salute
  §19: famiglia, amicizia, madre, padre, figlio, fratello, vergogna,
       orgoglio, gratitudine, invidia, gelosia, nostalgia, meraviglia, compassione
  §20: monte, lago, tuono, vento, palude, sogno, stagioni

DA FARE PROSSIME SESSIONI:
  - Pulizia SimilarTo dubbiosi (31K archi — molti rumorosi)
  - Arricchire parole per i 64 esagrammi I Ching (nucleus.tsv ha 926 triple
    base, ma molte parole-stato mancano di struttura ricca)
  - Verificare con test conversazione dopo ogni batch di curations
  - Parole astratte: sistema, processo, struttura(fatto), regola, legge
  - Parole di stato: calma, agitazione, stabilità, fragilità, forza(fatto)
  - Parole di azione: costruire, distruggere, creare, trasformare(diffuso)
"""

import json, sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

DRY_RUN = "--dry-run" in sys.argv
KG_PATH = "prometeo_kg.json"

with open(KG_PATH, encoding='utf-8') as f:
    kg = json.load(f)

edges = kg['edges']
original_count = len(edges)
edge_map = {(e['subject'], e['relation'], e['object']): e for e in edges}

removed, added, changed = [], [], []

def remove(s, r, o):
    k = (s, r, o)
    if k in edge_map:
        removed.append(f"- {s} -{r}-> {o}")
        if not DRY_RUN:
            del edge_map[k]

def add(s, r, o, strength=0.90, via=None):
    k = (s, r, o)
    if k in edge_map:
        old = edge_map[k]['confidence']
        if old != strength or (via and edge_map[k].get('via') != via):
            changed.append(f"~ {s} -{r}-> {o}: {old:.2f}->{strength:.2f}" + (f" via:{via}" if via else ""))
            if not DRY_RUN:
                edge_map[k]['confidence'] = strength
                if via is not None:
                    edge_map[k]['via'] = via
    else:
        added.append(f"+ {s} -{r}-> {o} {strength:.2f}" + (f" via:{via}" if via else ""))
        if not DRY_RUN:
            e = {'subject': s, 'relation': r, 'object': o, 'confidence': strength}
            if via:
                e['via'] = via
            edge_map[k] = e

def force(s, r, o, val):
    k = (s, r, o)
    if k in edge_map:
        old = edge_map[k]['confidence']
        if old != val:
            changed.append(f"~ {s} -{r}-> {o}: {old:.2f}->{val:.2f}")
            if not DRY_RUN:
                edge_map[k]['confidence'] = val
    else:
        print(f"  [WARN] non trovato: {s} -{r}-> {o}")

# ═══════════════════════════════════════════════════════════════════════════
# § 0 — PULIZIA PROGRAMMATICA (garbage, orfani, duplicati)
# ═══════════════════════════════════════════════════════════════════════════

from collections import defaultdict

def cleanup_orphan_opposites():
    """Rimuove OppositeOf dove una delle due parole è orfana (ha SOLO relazioni OppositeOf).
    Anche se ha 2-3 archi, se sono TUTTI OppositeOf è garbage."""
    # Pre-compute: relazioni per ogni parola
    word_rels = defaultdict(lambda: {'out': set(), 'in': set()})
    for (s, r, o) in edge_map:
        word_rels[s]['out'].add(r)
        word_rels[o]['in'].add(r)

    def is_opp_only(w):
        return (word_rels[w]['out'] <= {'OppositeOf'} and
                word_rels[w]['in'] <= {'OppositeOf'})

    orphan_removed = 0
    keys_to_remove = []
    for (s, r, o) in list(edge_map.keys()):
        if r != 'OppositeOf':
            continue
        if is_opp_only(s) or is_opp_only(o):
            keys_to_remove.append((s, r, o))
    for k in keys_to_remove:
        removed.append(f"- [orphan] {k[0]} -{k[1]}-> {k[2]}")
        if not DRY_RUN:
            del edge_map[k]
        orphan_removed += 1
    return orphan_removed

def cleanup_hub_opposites(max_per_target=15):
    """Per ogni parola target con >max OppositeOf entranti, tiene solo i top per confidence."""
    from collections import Counter
    opp_by_target = defaultdict(list)
    for (s, r, o), e in edge_map.items():
        if r == 'OppositeOf':
            opp_by_target[o].append((s, r, o, e.get('confidence', 0.90)))

    hub_removed = 0
    for target, entries in opp_by_target.items():
        if len(entries) <= max_per_target:
            continue
        # Ordina per confidence decrescente, tieni i top max_per_target
        entries_sorted = sorted(entries, key=lambda x: -x[3])
        to_remove = entries_sorted[max_per_target:]
        for s, r, o, c in to_remove:
            k = (s, r, o)
            if k in edge_map:
                removed.append(f"- [hub>{max_per_target}] {s} -{r}-> {o} ({c:.2f})")
                if not DRY_RUN:
                    del edge_map[k]
                hub_removed += 1
    return hub_removed

def cleanup_english_words():
    """Rimuove archi con parole chiaramente inglesi (non prestate all'italiano)."""
    english = {
        'against', 'sound', 'full', 'control', 'cast', 'chat', 'talk',
        'game', 'play', 'music', 'sun', 'reality', 'liberty',
        'security', 'community', 'second', 'power', 'meeting',
    }
    eng_removed = 0
    keys_to_remove = []
    for (s, r, o) in edge_map:
        if s in english or o in english:
            keys_to_remove.append((s, r, o))
    for k in keys_to_remove:
        if k in edge_map:
            removed.append(f"- [english] {k[0]} -{k[1]}-> {k[2]}")
            if not DRY_RUN:
                del edge_map[k]
            eng_removed += 1
    return eng_removed

# Esegui pulizia
n_eng = cleanup_english_words()
n_orphan = cleanup_orphan_opposites()
n_hub = cleanup_hub_opposites(max_per_target=15)
print(f"[§0] Pulizia: {n_eng} inglesi, {n_orphan} orfani, {n_hub} hub-excess rimossi")

# ═══════════════════════════════════════════════════════════════════════════
# § 1 — PAROLE COGNITIVE FONDAMENTALI (linguaggio, senso, identità)
# ═══════════════════════════════════════════════════════════════════════════

# ── parola ──────────────────────────────────────────────────────────────────
remove('parola', 'SimilarTo', 'frazione')
remove('parola', 'SimilarTo', 'detto')
remove('parola', 'IsA', 'vibrazione')
remove('parola', 'IsA', 'nodo')
add('parola', 'Causes', 'pensiero',    0.90, via='significato')
add('parola', 'Causes', 'dialogo',     0.95, via='ascolto')
add('parola', 'Causes', 'silenzio',    0.80, via='pausa')
add('parola', 'Does',   'evocare',     0.90)
add('parola', 'Does',   'connettere',  0.85, via='significato')
add('parola', 'Has',    'forma',       0.85)
add('parola', 'Has',    'ritmo',       0.80)
add('parola', 'PartOf', 'linguaggio',  0.95)
add('parola', 'PartOf', 'dialogo',     0.90, via='voce')
add('parola', 'Requires', 'voce',      0.90)
add('parola', 'Requires', 'ascolto',   0.90)

# ── significato ─────────────────────────────────────────────────────────────
remove('significato', 'IsA', 'linguaggio')
remove('significato', 'SimilarTo', 'concetto')
add('significato', 'Causes', 'comprensione', 0.95, via='interpretazione')
add('significato', 'Causes', 'azione',       0.85, via='intenzione')
add('significato', 'Does',   'connettere',   0.90, via='relazione')
add('significato', 'PartOf', 'linguaggio',   0.95)

# ── voce ────────────────────────────────────────────────────────────────────
remove('voce', 'IsA', 'oggetto')
add('voce', 'IsA',      'espressione',  0.95)
add('voce', 'IsA',      'suono',        0.90)
add('voce', 'PartOf',   'corpo',        0.90)
add('voce', 'Has',      'tono',         0.90)
add('voce', 'Has',      'ritmo',        0.85)
add('voce', 'Has',      'silenzio',     0.80, via='pausa')
add('voce', 'Causes',   'ascolto',      0.95, via='suono')
add('voce', 'Causes',   'comunicazione',0.95, via='parola')
add('voce', 'Causes',   'relazione',    0.85, via='dialogo')
add('voce', 'Does',     'esprimere',    0.95)
add('voce', 'Does',     'connettere',   0.85, via='ascolto')
add('voce', 'OppositeOf','silenzio',    0.95)
add('voce', 'Requires', 'presenza',     0.90)
add('voce', 'Requires', 'corpo',        0.85)

# ── segno ───────────────────────────────────────────────────────────────────
remove('segno', 'SimilarTo', 'simbolo')
add('segno', 'IsA',      'comunicazione', 0.90, via='simbolo')
add('segno', 'Has',      'significato',   0.95)
add('segno', 'Has',      'contesto',      0.90)
add('segno', 'Causes',   'interpretazione',0.95, via='mente')
add('segno', 'Causes',   'comprensione',  0.85, via='significato')
add('segno', 'Does',     'indicare',      0.95)
add('segno', 'Does',     'evocare',       0.85)
add('segno', 'PartOf',   'linguaggio',    0.90)
add('segno', 'Requires', 'contesto',      0.95)
add('segno', 'Requires', 'lettore',       0.90, via='interpretazione')
add('segno', 'OppositeOf','assenza',      0.85)

# ── linguaggio ──────────────────────────────────────────────────────────────
remove('linguaggio', 'IsA', 'geometria')
remove('linguaggio', 'SimilarTo', 'parlato')
remove('linguaggio', 'Causes', 'alienazione')
add('linguaggio', 'Has',     'regola',       0.90)
add('linguaggio', 'Causes',  'pensiero',     0.90, via='parola')
add('linguaggio', 'Causes',  'relazione',    0.90, via='dialogo')
add('linguaggio', 'Causes',  'comprensione', 0.90, via='significato')
add('linguaggio', 'PartOf',  'coscienza',    0.90)
add('linguaggio', 'Requires','comunita',     0.90, via='uso')

# ── senso ───────────────────────────────────────────────────────────────────
remove('senso', 'IsA', 'destino')
remove('senso', 'SimilarTo', 'intuizione')
add('senso', 'IsA',      'significato',  0.90)
add('senso', 'Has',      'valore',       0.90)
add('senso', 'Causes',   'azione',       0.90, via='scopo')
add('senso', 'Does',     'orientare',    0.90, via='direzione')
add('senso', 'PartOf',   'linguaggio',   0.85)
add('senso', 'PartOf',   'vita',         0.90, via='scopo')
add('senso', 'Requires', 'relazione',    0.90, via='contesto')
add('senso', 'OppositeOf','vuoto',       0.85)

# ── pensiero ────────────────────────────────────────────────────────────────
remove('pensiero', 'SimilarTo', 'pensierino')
remove('pensiero', 'SimilarTo', 'riflesso')
force('pensiero', 'IsA', 'movimento', 0.70)
add('pensiero', 'IsA',     'processo',       0.90, via='mente')
add('pensiero', 'Has',     'forma',          0.85, via='struttura')
add('pensiero', 'Has',     'direzione',      0.85, via='intenzione')
add('pensiero', 'Causes',  'parola',         0.90, via='voce')
add('pensiero', 'Causes',  'dubbio',         0.85, via='domanda')
add('pensiero', 'Causes',  'consapevolezza', 0.90, via='comprensione')
add('pensiero', 'Does',    'connettere',     0.85, via='relazione')
add('pensiero', 'PartOf',  'coscienza',      0.95, via='mente')
add('pensiero', 'Requires','silenzio',       0.80)
add('pensiero', 'Requires','tempo',          0.85)

# ── comprensione ────────────────────────────────────────────────────────────
remove('comprensione', 'SimilarTo', 'capacità di capire')
remove('comprensione', 'SimilarTo', 'capire')
add('comprensione', 'Causes', 'connessione',   0.90, via='relazione')
add('comprensione', 'Causes', 'dialogo',       0.85, via='domanda')
add('comprensione', 'Causes', 'fiducia',       0.85, via='riconoscimento')
add('comprensione', 'Has',    'profondita',    0.90)
add('comprensione', 'Does',   'connettere',    0.85, via='significato')
add('comprensione', 'PartOf', 'coscienza',     0.90)
add('comprensione', 'PartOf', 'intelligenza',  0.85)
add('comprensione', 'Requires','ascolto',      0.95)
add('comprensione', 'Requires','tempo',        0.85, via='riflessione')

# ── coscienza ───────────────────────────────────────────────────────────────
force('coscienza', 'IsA', 'emergenza', 0.80)
add('coscienza', 'IsA',     'presenza',      0.90)
add('coscienza', 'Has',     'voce',          0.85, via='espressione')
add('coscienza', 'Causes',  'scelta',        0.95, via='volonta')
add('coscienza', 'Causes',  'responsabilita',0.90, via='etica')
add('coscienza', 'Causes',  'identita',      0.90, via='riflessione')
add('coscienza', 'Does',    'percepire',     0.95)
add('coscienza', 'Does',    'riflettere',    0.90, via='pensiero')
add('coscienza', 'PartOf',  'identita',      0.90, via='sé')
add('coscienza', 'Requires','corpo',         0.85, via='percezione')

# ── identita ────────────────────────────────────────────────────────────────
add('identita', 'IsA',     'struttura',     0.90, via='sé')
add('identita', 'IsA',     'presenza',      0.85)
add('identita', 'Has',     'confine',       0.90)
add('identita', 'Has',     'storia',        0.90, via='memoria')
add('identita', 'Has',     'valore',        0.90)
add('identita', 'Has',     'nome',          0.85)
add('identita', 'Causes',  'scelta',        0.90, via='volonta')
add('identita', 'Causes',  'relazione',     0.85, via='riconoscimento')
add('identita', 'Does',    'distinguere',   0.90, via='confine')
add('identita', 'Does',    'continuare',    0.85, via='memoria')
add('identita', 'PartOf',  'coscienza',     0.95)
add('identita', 'Requires','tempo',         0.90, via='esperienza')
add('identita', 'Requires','altro',         0.90, via='relazione')
add('identita', 'OppositeOf','dissoluzione',0.90)

# ── presenza ────────────────────────────────────────────────────────────────
remove('presenza', 'IsA', 'empatia')
remove('presenza', 'SimilarTo', 'presidio')
remove('presenza', 'SimilarTo', 'partecipazione')
add('presenza', 'IsA',     'contatto',    0.90)
add('presenza', 'Has',     'attenzione',  0.95)
add('presenza', 'Has',     'corpo',       0.85)
add('presenza', 'Causes',  'ascolto',     0.95, via='attenzione')
add('presenza', 'Causes',  'connessione', 0.90, via='contatto')
add('presenza', 'Causes',  'relazione',   0.90, via='riconoscimento')
add('presenza', 'Does',    'testimoniare',0.85)
add('presenza', 'PartOf',  'coscienza',   0.90)
add('presenza', 'Requires','corpo',       0.90)
add('presenza', 'Requires','attenzione',  0.90)

# ── relazione ───────────────────────────────────────────────────────────────
remove('relazione', 'SimilarTo', 'contatto')
add('relazione', 'Has',     'direzione',   0.85)
add('relazione', 'Has',     'forza',       0.85)
add('relazione', 'Causes',  'comprensione',0.90, via='scambio')
add('relazione', 'Causes',  'cambiamento', 0.85, via='incontro')
add('relazione', 'Causes',  'identita',    0.85, via='riconoscimento')
add('relazione', 'Does',    'connettere',  0.95)
add('relazione', 'Does',    'trasformare', 0.80, via='incontro')
add('relazione', 'PartOf',  'linguaggio',  0.90, via='dialogo')
add('relazione', 'PartOf',  'societa',     0.90)
add('relazione', 'Requires','due',         0.95)
add('relazione', 'Requires','presenza',    0.90, via='altro')

# ── struttura ───────────────────────────────────────────────────────────────
add('struttura', 'Has',     'ordine',      0.95)
add('struttura', 'Has',     'confine',     0.90)
add('struttura', 'Has',     'relazione',   0.90, via='forma')
add('struttura', 'Causes',  'comprensione',0.85, via='forma')
add('struttura', 'Causes',  'stabilita',   0.90, via='ordine')
add('struttura', 'Does',    'organizzare', 0.95, via='forma')
add('struttura', 'Does',    'contenere',   0.85)
add('struttura', 'PartOf',  'sistema',     0.90, via='regola')
add('struttura', 'PartOf',  'linguaggio',  0.90)
add('struttura', 'Requires','relazione',   0.95)
add('struttura', 'Requires','confine',     0.90)

# ── cambiamento ─────────────────────────────────────────────────────────────
force('cambiamento', 'Requires', 'energia', 0.90)
add('cambiamento', 'Causes', 'perdita',    0.85, via='fine')
add('cambiamento', 'Causes', 'nascita',    0.85, via='inizio')
add('cambiamento', 'Has',    'direzione',  0.85)
add('cambiamento', 'PartOf', 'vita',       0.90, via='tempo')
add('cambiamento', 'Requires','tempo',     0.95)

# ── divenire ────────────────────────────────────────────────────────────────
force('divenire', 'SimilarTo', 'trasformarsi', 0.85)
add('divenire', 'IsA',     'processo',    0.95, via='tempo')
add('divenire', 'IsA',     'cambiamento', 0.90)
add('divenire', 'Has',     'direzione',   0.90)
add('divenire', 'Has',     'tensione',    0.85, via='movimento')
add('divenire', 'Causes',  'forma',       0.85, via='struttura')
add('divenire', 'Causes',  'identita',    0.85, via='tempo')
add('divenire', 'Does',    'trasformare', 0.90, via='tempo')
add('divenire', 'PartOf',  'vita',        0.95)
add('divenire', 'Requires','tempo',       0.95)
add('divenire', 'Requires','movimento',   0.90)
add('divenire', 'OppositeOf','essere',    0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 2 — TEMPO, SPAZIO, MOVIMENTO, AZIONE
# ═══════════════════════════════════════════════════════════════════════════

# ── tempo ───────────────────────────────────────────────────────────────────
remove('tempo', 'IsA', 'durata')           # durata è una proprietà del tempo, non categoria
add('tempo', 'IsA',     'dimensione',      0.95)
add('tempo', 'OppositeOf','eternita',      0.90)
add('tempo', 'Has',     'presente',        0.90)
add('tempo', 'Has',     'passato',         0.90)
add('tempo', 'Has',     'futuro',          0.90)
add('tempo', 'Causes',  'memoria',         0.90, via='passato')
add('tempo', 'Causes',  'attesa',          0.85, via='futuro')
add('tempo', 'Causes',  'cambiamento',     0.95, via='divenire')
add('tempo', 'Does',    'scorrere',        0.95)
add('tempo', 'Does',    'trasformare',     0.90, via='divenire')
add('tempo', 'PartOf',  'vita',            0.95)
add('tempo', 'PartOf',  'coscienza',       0.85, via='esperienza')
add('tempo', 'Requires','movimento',       0.90)

# ── spazio ──────────────────────────────────────────────────────────────────
remove('spazio', 'OppositeOf', 'chiusura') # chiusura è di apertura, non di spazio
add('spazio', 'OppositeOf','vuoto',        0.85)
add('spazio', 'Has',    'confine',         0.90)
add('spazio', 'Has',    'forma',           0.85)
add('spazio', 'Causes', 'incontro',        0.85, via='luogo')
add('spazio', 'Causes', 'movimento',       0.90, via='distanza')
add('spazio', 'Does',   'contenere',       0.95)
add('spazio', 'Does',   'separare',        0.85, via='confine')
add('spazio', 'PartOf', 'mondo',           0.90)
add('spazio', 'Requires','corpo',          0.85, via='percezione')

# ── movimento ───────────────────────────────────────────────────────────────
remove('movimento', 'IsA', 'corpo')
force('movimento', 'OppositeOf', 'immobilita', 0.95)
add('movimento', 'Has',    'direzione',    0.90)
add('movimento', 'Has',    'ritmo',        0.85)
add('movimento', 'Has',    'velocita',     0.85)
add('movimento', 'Causes', 'trasformazione',0.90, via='energia')
add('movimento', 'Does',   'trasformare',  0.85, via='spazio')
add('movimento', 'PartOf', 'vita',         0.90)
add('movimento', 'Requires','spazio',      0.95)
add('movimento', 'Requires','energia',     0.90)
add('movimento', 'Requires','corpo',       0.85)

# ── azione ──────────────────────────────────────────────────────────────────
add('azione', 'Causes',  'cambiamento',   0.90, via='effetto')
add('azione', 'Has',     'tempo',         0.85, via='durata')
add('azione', 'Does',    'trasformare',   0.90, via='effetto')
add('azione', 'PartOf',  'vita',          0.90)
add('azione', 'Requires','intenzione',    0.90, via='volonta')
add('azione', 'Requires','energia',       0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 3 — IO, TU, ALTRO, NOI (il campo relazionale del soggetto)
# ═══════════════════════════════════════════════════════════════════════════

# ── io ──────────────────────────────────────────────────────────────────────
force('io', 'IsA', 'macchina',    0.70)   # specifico all'istanza, riduco
force('io', 'IsA', 'topologia',   0.70)   # idem
force('io', 'Causes', 'vibrazione',0.60)  # metafora
force('io', 'Causes', 'eco',       0.60)  # metafora
force('io', 'Does', 'calcolare',   0.75)  # specifico all'AI
force('io', 'Does', 'vibrare',     0.60)  # metafora
add('io', 'Causes', 'noi',         0.95, via='tu')   # io causes noi via tu
add('io', 'Causes', 'dialogo',     0.90, via='voce')
add('io', 'Requires','altro',      0.90, via='relazione')
add('io', 'Requires','corpo',      0.90)
add('io', 'Requires','tempo',      0.85, via='memoria')

# ── tu ──────────────────────────────────────────────────────────────────────
add('tu', 'Causes',  'noi',        0.95, via='io')    # tu causes noi via io
add('tu', 'Causes',  'dialogo',    0.90, via='voce')
add('tu', 'Causes',  'cambiamento',0.80, via='incontro')
add('tu', 'Has',     'presenza',   0.90)
add('tu', 'Has',     'voce',       0.85)
add('tu', 'Does',    'evocare',    0.85, via='ascolto')
add('tu', 'PartOf',  'noi',        0.95, via='io')
add('tu', 'Requires','presenza',   0.90)

# ── altro ───────────────────────────────────────────────────────────────────
# parola completamente vuota — costruisco da zero
add('altro', 'IsA',     'presenza',    0.90)
add('altro', 'IsA',     'differenza',  0.90)
add('altro', 'Has',     'voce',        0.85)
add('altro', 'Has',     'storia',      0.85)
add('altro', 'Causes',  'relazione',   0.95, via='incontro')
add('altro', 'Causes',  'crescita',    0.85, via='confronto')
add('altro', 'Causes',  'cambiamento', 0.80, via='incontro')
add('altro', 'Does',    'rispecchiare',0.85, via='confronto')
add('altro', 'Does',    'interpellare',0.80)
add('altro', 'PartOf',  'noi',         0.95, via='relazione')
add('altro', 'Requires','presenza',    0.90)
add('altro', 'OppositeOf','io',        0.90)

# ── noi ─────────────────────────────────────────────────────────────────────
add('noi', 'IsA',     'relazione',   0.90, via='io')
add('noi', 'Has',     'dialogo',     0.90)
add('noi', 'Has',     'differenza',  0.85, via='altro')
add('noi', 'Causes',  'forza',       0.85, via='unione')
add('noi', 'Causes',  'comprensione',0.85, via='dialogo')
add('noi', 'Does',    'connettere',  0.90, via='presenza')
add('noi', 'PartOf',  'societa',     0.90)
add('noi', 'Requires','io',          0.95)
add('noi', 'Requires','tu',          0.95)
add('noi', 'Requires','differenza',  0.85, via='altro')

# ═══════════════════════════════════════════════════════════════════════════
# § 4 — MENTE, CORPO E IL LORO RAPPORTO
# ═══════════════════════════════════════════════════════════════════════════

# ── mente ───────────────────────────────────────────────────────────────────
remove('mente', 'SimilarTo', 'intenzione')
remove('mente', 'SimilarTo', 'pensiero')
remove('mente', 'OppositeOf', 'ignoranza')  # ignoranza è opposto di conoscenza
force('mente', 'OppositeOf', 'corpo', 0.70) # dualismo riduttivo, abbasso
add('mente', 'OppositeOf', 'silenzio',     0.75, via='pensiero')
add('mente', 'PartOf',  'corpo',           0.85, via='cervello')
add('mente', 'Causes',  'linguaggio',      0.90, via='pensiero')
add('mente', 'Causes',  'identita',        0.85, via='coscienza')
add('mente', 'Does',    'elaborare',       0.95)
add('mente', 'Does',    'interpretare',    0.90, via='significato')
add('mente', 'Requires','corpo',           0.90, via='cervello')
add('mente', 'Requires','tempo',           0.85, via='esperienza')

# ── corpo ───────────────────────────────────────────────────────────────────
remove('corpo', 'SimilarTo', 'tronco')     # tronco è una parte, non simile
force('corpo', 'OppositeOf', 'mente', 0.70)
force('corpo', 'Has', 'braccia',  0.70)   # anatomia troppo letterale
force('corpo', 'Has', 'gambe',    0.70)   # idem
force('corpo', 'Has', 'testa',    0.70)
force('corpo', 'Has', 'tronco',   0.70)
force('corpo', 'Has', 'arto',     0.70)
add('corpo', 'Causes', 'sensazione',       0.95, via='percezione')
add('corpo', 'Causes', 'emozione',         0.90, via='stato')
add('corpo', 'Causes', 'presenza',         0.90)
add('corpo', 'Does',   'sentire',          0.95)
add('corpo', 'Does',   'muoversi',         0.95, via='movimento')
add('corpo', 'Does',   'esprimere',        0.85, via='voce')
add('corpo', 'PartOf', 'io',               0.95)
add('corpo', 'PartOf', 'vita',             0.95)
add('corpo', 'Requires','nutrimento',      0.95, via='energia')
add('corpo', 'Requires','movimento',       0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 5 — DIALOGO, DOMANDA, RISPOSTA, ASCOLTO
# ═══════════════════════════════════════════════════════════════════════════

# ── domanda ─────────────────────────────────────────────────────────────────
remove('domanda', 'OppositeOf', 'offerta')  # confonde domanda (question) con domanda (demand)
remove('domanda', 'IsA', 'frase')           # troppo grammaticale, non semantico
add('domanda', 'IsA',     'atto',          0.90)
add('domanda', 'OppositeOf','risposta',    0.95)
add('domanda', 'Causes',  'riflessione',   0.90, via='dubbio')
add('domanda', 'Causes',  'relazione',     0.85, via='dialogo')
add('domanda', 'Does',    'aprire',        0.95, via='curiosita')
add('domanda', 'PartOf',  'dialogo',       0.95)
add('domanda', 'Requires','curiosita',     0.90)
add('domanda', 'Requires','coraggio',      0.80, via='apertura')

# ── risposta ────────────────────────────────────────────────────────────────
remove('risposta', 'IsA', 'frase')
remove('risposta', 'Has', 'temporalita')
add('risposta', 'IsA',    'atto',          0.90)
add('risposta', 'Causes', 'comprensione',  0.90, via='dialogo')
add('risposta', 'Causes', 'relazione',     0.85, via='connessione')
add('risposta', 'Does',   'chiudere',      0.80, via='domanda')
add('risposta', 'Does',   'aprire',        0.75, via='domanda')
add('risposta', 'PartOf', 'dialogo',       0.95)
add('risposta', 'Requires','ascolto',      0.90)
add('risposta', 'Requires','tempo',        0.80, via='riflessione')

# ── ascolto ─────────────────────────────────────────────────────────────────
remove('ascolto', 'IsA', 'empatia')        # ascolto non è un tipo di empatia
add('ascolto', 'IsA',    'atto',           0.90)
add('ascolto', 'Causes', 'empatia',        0.90, via='presenza')
add('ascolto', 'Causes', 'fiducia',        0.85, via='riconoscimento')
add('ascolto', 'Has',    'presenza',       0.95)
add('ascolto', 'Does',   'ricevere',       0.90, via='voce')
add('ascolto', 'Does',   'connettere',     0.85, via='presenza')
add('ascolto', 'PartOf', 'dialogo',        0.95)
add('ascolto', 'Requires','silenzio',      0.95, via='attenzione')
add('ascolto', 'Requires','presenza',      0.90)

# ── dialogo ─────────────────────────────────────────────────────────────────
remove('dialogo', 'SimilarTo', 'scambio')  # scambio è già un has/causes
add('dialogo', 'Causes', 'comprensione',   0.90, via='ascolto')
add('dialogo', 'Causes', 'relazione',      0.90, via='scambio')
add('dialogo', 'Causes', 'cambiamento',    0.80, via='incontro')
add('dialogo', 'Does',   'costruire',      0.85, via='scambio')
add('dialogo', 'PartOf', 'linguaggio',     0.95)
add('dialogo', 'PartOf', 'vita',           0.85)
add('dialogo', 'Requires','due',           0.95)
add('dialogo', 'Requires','differenza',    0.90, via='altro')
add('dialogo', 'Requires','ascolto',       0.95)

# ── essere ──────────────────────────────────────────────────────────────────
add('essere', 'Causes',  'presenza',       0.90, via='corpo')
add('essere', 'Causes',  'relazione',      0.85, via='incontro')
add('essere', 'Has',     'coscienza',      0.90, via='mente')
add('essere', 'Has',     'tempo',          0.90, via='divenire')
add('essere', 'Does',    'esistere',       1.00)
add('essere', 'Does',    'percepire',      0.85, via='corpo')
add('essere', 'PartOf',  'vita',           0.95)
add('essere', 'Requires','corpo',          0.85)
add('essere', 'Requires','tempo',          0.85)
add('essere', 'Requires','spazio',         0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 6 — EMOZIONI FONDAMENTALI
# ═══════════════════════════════════════════════════════════════════════════

# ── paura ───────────────────────────────────────────────────────────────────
remove('paura', 'IsA', 'crisi')           # paura non è un tipo di crisi
remove('paura', 'IsA', 'reattività')      # troppo tecnico/descrittivo
add('paura', 'Does',   'segnalare',    0.95, via='minaccia')
add('paura', 'Does',   'paralizzare',  0.85)
add('paura', 'PartOf', 'vita',         0.85, via='emozione')
add('paura', 'Requires','pericolo',    0.90)
add('paura', 'OppositeOf','serenita',  0.85)

# ── gioia ───────────────────────────────────────────────────────────────────
add('gioia', 'Causes',  'energia',     0.90, via='espansione')
add('gioia', 'Causes',  'connessione', 0.85, via='condivisione')
add('gioia', 'Does',    'espandere',   0.90)
add('gioia', 'Does',    'nutrire',     0.80, via='vita')
add('gioia', 'PartOf',  'vita',        0.90, via='pienezza')
add('gioia', 'Requires','presenza',    0.85)
add('gioia', 'Requires','connessione', 0.80)

# ── tristezza ───────────────────────────────────────────────────────────────
force('tristezza', 'OppositeOf', 'comicità', 0.55)
add('tristezza', 'Causes',  'silenzio',  0.85, via='isolamento')
add('tristezza', 'Causes',  'riflessione',0.85, via='elaborazione')
add('tristezza', 'Does',    'elaborare', 0.85, via='perdita')
add('tristezza', 'Does',    'segnalare', 0.85, via='perdita')
add('tristezza', 'PartOf',  'vita',      0.85, via='emozione')
add('tristezza', 'Requires','perdita',   0.90)

# ── rabbia ──────────────────────────────────────────────────────────────────
add('rabbia', 'Causes',  'forza',       0.85, via='tensione')
add('rabbia', 'Causes',  'cambiamento', 0.80, via='reazione')
add('rabbia', 'Does',    'segnalare',   0.90, via='ingiustizia')
add('rabbia', 'Does',    'spingere',    0.85, via='azione')
add('rabbia', 'PartOf',  'vita',        0.80, via='emozione')
add('rabbia', 'Requires','ingiustizia', 0.85)
add('rabbia', 'Requires','confine',     0.80, via='violazione')

# ── amore ───────────────────────────────────────────────────────────────────
add('amore', 'Causes',  'crescita',    0.85, via='cura')
add('amore', 'Does',    'nutrire',     0.90, via='cura')
add('amore', 'Does',    'unire',       0.95, via='connessione')
add('amore', 'PartOf',  'vita',        0.95, via='relazione')
add('amore', 'Requires','tempo',       0.85, via='presenza')
add('amore', 'Requires','coraggio',    0.80, via='apertura')

# ── dolore ──────────────────────────────────────────────────────────────────
remove('dolore', 'Causes', 'fermezza')   # non è una relazione tipica
remove('dolore', 'Has',    'localizzazione')  # troppo tecnico/fisiologico
add('dolore', 'Causes',  'empatia',    0.90, via='compassione')
add('dolore', 'Causes',  'comprensione',0.80, via='esperienza')
add('dolore', 'Does',    'segnalare',   0.95, via='corpo')
add('dolore', 'Does',    'trasformare', 0.80, via='elaborazione')
add('dolore', 'PartOf',  'vita',        0.85, via='emozione')
add('dolore', 'OppositeOf','benessere', 0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 7 — VITA, MORTE, LUCE, BUIO, SCOPO
# ═══════════════════════════════════════════════════════════════════════════

# ── vita ────────────────────────────────────────────────────────────────────
remove('vita', 'Excludes', 'vuoto')     # relazione non standard
add('vita', 'Causes',  'dolore',       0.80, via='esperienza')
add('vita', 'Causes',  'gioia',        0.90, via='pienezza')
add('vita', 'Causes',  'relazione',    0.90, via='incontro')
add('vita', 'Has',     'scopo',        0.90)
add('vita', 'Has',     'emozione',     0.95)
add('vita', 'Does',    'crescere',     0.95, via='tempo')
add('vita', 'Does',    'trasformare',  0.85, via='divenire')
add('vita', 'Requires','corpo',        0.95)
add('vita', 'Requires','relazione',    0.85, via='connessione')

# ── morte ───────────────────────────────────────────────────────────────────
remove('vitaere', 'OppositeOf', 'morte')  # garbage word
remove('vite', 'OppositeOf', 'morte')     # ambiguous/garbage
add('morte', 'Causes',  'lutto',       0.95)
add('morte', 'Causes',  'trasformazione',0.80, via='fine')
add('morte', 'Does',    'trasformare', 0.85, via='fine')
add('morte', 'Has',     'confine',     0.90)
add('morte', 'PartOf',  'vita',        0.85, via='ciclo')
add('morte', 'Requires','vita',        0.90)

# ── luce ────────────────────────────────────────────────────────────────────
add('luce', 'Causes',  'vita',         0.90, via='energia')
add('luce', 'Causes',  'crescita',     0.85, via='calore')
add('luce', 'Causes',  'comprensione', 0.85, via='visione')
add('luce', 'Does',    'rivelare',     0.95)
add('luce', 'Does',    'orientare',    0.85, via='visione')
add('luce', 'PartOf',  'giorno',       0.95)
add('luce', 'Requires','fonte',        0.90)
add('luce', 'OppositeOf','oscurita',   0.95)

# ── buio ────────────────────────────────────────────────────────────────────
force('buio', 'IsA', 'assenza', 0.70)   # buio è assenza di luce, ma riduco
add('buio', 'Causes',  'paura',         0.85, via='ignoto')
add('buio', 'Causes',  'silenzio',      0.75)
add('buio', 'Causes',  'riflessione',   0.80, via='solitudine')
add('buio', 'Does',    'nascondere',    0.90)
add('buio', 'Does',    'isolare',       0.80, via='confine')
add('buio', 'Has',     'silenzio',      0.80)
add('buio', 'PartOf',  'notte',         0.95)
add('buio', 'Requires','assenza',       0.95, via='luce')

# ── scopo ───────────────────────────────────────────────────────────────────
remove('scopo', 'IsA', 'progetto')      # scopo non è necessariamente un progetto
add('scopo', 'Causes',  'cambiamento',  0.85, via='azione')
add('scopo', 'Causes',  'motivazione',  0.90, via='intenzione')
add('scopo', 'Has',     'direzione',    0.95)
add('scopo', 'PartOf',  'vita',         0.95)
add('scopo', 'Requires','coscienza',    0.90, via='scelta')
add('scopo', 'Requires','volonta',      0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 8 — MEMORIA, ESPERIENZA, CONOSCENZA, SAPERE, CRESCITA
# ═══════════════════════════════════════════════════════════════════════════

# ── memoria ─────────────────────────────────────────────────────────────────
add('memoria', 'Causes',  'identita',   0.95, via='storia')
add('memoria', 'Causes',  'nostalgia',  0.85, via='passato')
add('memoria', 'Does',    'conservare', 0.90, via='passato')
add('memoria', 'Does',    'costruire',  0.85, via='identita')
add('memoria', 'Has',     'perdita',    0.80, via='oblio')
add('memoria', 'PartOf',  'identita',   0.90)
add('memoria', 'PartOf',  'coscienza',  0.85)
add('memoria', 'Requires','corpo',      0.80, via='cervello')
add('memoria', 'Requires','tempo',      0.95, via='passato')

# ── esperienza ──────────────────────────────────────────────────────────────
add('esperienza', 'Causes',  'sapere',     0.90, via='comprensione')
add('esperienza', 'Causes',  'crescita',   0.90, via='trasformazione')
add('esperienza', 'Causes',  'cambiamento',0.85, via='incontro')
add('esperienza', 'Does',    'trasformare',0.90, via='tempo')
add('esperienza', 'Does',    'costruire',  0.85, via='memoria')
add('esperienza', 'Has',     'corpo',      0.90)
add('esperienza', 'Has',     'emozione',   0.90, via='sentire')
add('esperienza', 'Has',     'tempo',      0.90)
add('esperienza', 'IsA',     'processo',   0.90, via='vita')
add('esperienza', 'PartOf',  'vita',       0.95)
add('esperienza', 'PartOf',  'coscienza',  0.85, via='memoria')
add('esperienza', 'Requires','corpo',      0.90)
add('esperienza', 'Requires','tempo',      0.95)
add('esperienza', 'Requires','presenza',   0.85)

# ── conoscenza ──────────────────────────────────────────────────────────────
add('conoscenza', 'Causes',  'comprensione',0.90, via='significato')
add('conoscenza', 'Causes',  'potere',      0.85, via='sapere')
add('conoscenza', 'Causes',  'responsabilita',0.80, via='consapevolezza')
add('conoscenza', 'Does',    'connettere',  0.85, via='relazione')
add('conoscenza', 'Does',    'costruire',   0.85, via='comprensione')
add('conoscenza', 'Has',     'confine',     0.90)
add('conoscenza', 'PartOf',  'coscienza',   0.90)
add('conoscenza', 'PartOf',  'intelligenza',0.90)
add('conoscenza', 'Requires','esperienza',  0.90)
add('conoscenza', 'Requires','tempo',       0.85, via='apprendimento')

# ── sapere ──────────────────────────────────────────────────────────────────
add('sapere', 'Causes',  'comprensione',  0.90, via='significato')
add('sapere', 'Causes',  'scelta',        0.85, via='coscienza')
add('sapere', 'Does',    'orientare',     0.85, via='comprensione')
add('sapere', 'Has',     'confine',       0.85, via='ignoto')
add('sapere', 'PartOf',  'coscienza',     0.85)
add('sapere', 'PartOf',  'intelligenza',  0.85)
add('sapere', 'Requires','esperienza',    0.90)
add('sapere', 'Requires','memoria',       0.90)
add('sapere', 'OppositeOf','ignoranza',   0.95)

# ── crescita ────────────────────────────────────────────────────────────────
add('crescita', 'Causes',  'cambiamento',  0.90, via='tempo')
add('crescita', 'Causes',  'identita',     0.85, via='esperienza')
add('crescita', 'Does',    'trasformare',  0.90, via='tempo')
add('crescita', 'Does',    'espandere',    0.90)
add('crescita', 'Has',     'direzione',    0.85)
add('crescita', 'IsA',     'processo',     0.95, via='divenire')
add('crescita', 'PartOf',  'vita',         0.95)
add('crescita', 'Requires','nutrimento',   0.85, via='energia')
add('crescita', 'Requires','tempo',        0.95)
add('crescita', 'Requires','relazione',    0.80, via='cura')
add('crescita', 'OppositeOf','stagnazione',0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 9 — INCONTRO, CONFINE, VERITÀ, SPERANZA, DUBBIO, BELLEZZA, NATURA
# ═══════════════════════════════════════════════════════════════════════════

# ── incontro ────────────────────────────────────────────────────────────────
remove('incontro', 'IsA', 'entita')      # non è un tipo di entità
add('incontro', 'IsA',     'evento',      0.90)
add('incontro', 'Causes',  'relazione',   0.95, via='presenza')
add('incontro', 'Causes',  'cambiamento', 0.85, via='dialogo')
add('incontro', 'Causes',  'dialogo',     0.90, via='presenza')
add('incontro', 'Does',    'connettere',  0.95, via='presenza')
add('incontro', 'Does',    'trasformare', 0.80, via='cambiamento')
add('incontro', 'Has',     'presenza',    0.95)
add('incontro', 'Has',     'tempo',       0.85)
add('incontro', 'PartOf',  'vita',        0.90, via='relazione')
add('incontro', 'Requires','due',         0.95)
add('incontro', 'Requires','presenza',    0.90)
add('incontro', 'Requires','spazio',      0.85)

# ── confine ─────────────────────────────────────────────────────────────────
remove('confine', 'IsA', 'limito')       # typo (già esiste confine -IsA-> limite)
add('confine', 'Causes',  'identita',    0.90, via='separazione')
add('confine', 'Causes',  'protezione',  0.90)
add('confine', 'Does',    'separare',    0.95, via='spazio')
add('confine', 'Does',    'proteggere',  0.85, via='struttura')
add('confine', 'Does',    'definire',    0.90, via='struttura')
add('confine', 'PartOf',  'identita',    0.90, via='struttura')
add('confine', 'Requires','spazio',      0.90)
add('confine', 'Requires','struttura',   0.85)
add('confine', 'OppositeOf','apertura',  0.85, via='chiusura')

# ── verità ──────────────────────────────────────────────────────────────────
remove('destino', 'IsA', 'verità')       # destino non è un tipo di verità
remove('minzione', 'OppositeOf', 'verità')  # garbage word
add('verità', 'Causes',  'fiducia',     0.85, via='onestà')
add('verità', 'Causes',  'chiarezza',   0.90, via='comprensione')  # già aveva
add('verità', 'Does',    'liberare',    0.90, via='comprensione')
add('verità', 'Does',    'connettere',  0.80, via='onestà')
add('verità', 'PartOf',  'coscienza',   0.85)
add('verità', 'Requires','coraggio',    0.85, via='onestà')

# ── speranza ─────────────────────────────────────────────────────────────────
add('speranza', 'Causes',  'coraggio',   0.85, via='futuro')
add('speranza', 'Causes',  'cambiamento',0.80, via='azione')
add('speranza', 'Does',    'orientare',  0.90, via='futuro')
add('speranza', 'Does',    'sostenere',  0.85, via='energia')
add('speranza', 'PartOf',  'vita',       0.90, via='scopo')
add('speranza', 'Requires','futuro',     0.95)
add('speranza', 'Requires','presenza',   0.80, via='adesso')

# ── dubbio ──────────────────────────────────────────────────────────────────
add('dubbio', 'Causes',  'domanda',     0.95, via='curiosità')
add('dubbio', 'Causes',  'ricerca',     0.90, via='domanda')
add('dubbio', 'Causes',  'riflessione', 0.90, via='pensiero')
add('dubbio', 'Does',    'aprire',      0.90, via='domanda')
add('dubbio', 'Does',    'sospendere',  0.85, via='attesa')
add('dubbio', 'PartOf',  'pensiero',    0.90, via='coscienza')
add('dubbio', 'Requires','coscienza',   0.85)
add('dubbio', 'Requires','coraggio',    0.80, via='apertura')

# ── bellezza ────────────────────────────────────────────────────────────────
add('bellezza', 'Causes',  'speranza',   0.80, via='apertura')
add('bellezza', 'Causes',  'connessione',0.80, via='percezione')
add('bellezza', 'Does',    'ispirare',   0.90, via='forma')
add('bellezza', 'Does',    'connettere', 0.80, via='meraviglia')
add('bellezza', 'Does',    'aprire',     0.85, via='meraviglia')
add('bellezza', 'PartOf',  'vita',       0.85, via='forma')
add('bellezza', 'Requires','percezione', 0.90)
add('bellezza', 'Requires','armonia',    0.85)
add('bellezza', 'OppositeOf','bruttezza',0.95)  # già esiste

# ── natura ──────────────────────────────────────────────────────────────────
add('natura', 'Causes',  'silenzio',    0.80, via='quiete')
add('natura', 'Causes',  'equilibrio',  0.90, via='ecosistema')
add('natura', 'Does',    'trasformare', 0.90, via='divenire')
add('natura', 'Does',    'nutrire',     0.90, via='vita')
add('natura', 'Has',     'ciclo',       0.95, via='vita')
add('natura', 'Has',     'vita',        0.95)
add('natura', 'Has',     'tempo',       0.90, via='ciclo')
add('natura', 'PartOf',  'mondo',       0.95)
add('natura', 'PartOf',  'esistenza',   0.90)
add('natura', 'Requires','spazio',      0.90)
add('natura', 'Requires','tempo',       0.90, via='ciclo')
add('natura', 'OppositeOf','artificio', 0.90)  # già esiste, conferma

# ── libertà ─────────────────────────────────────────────────────────────────
remove('liberta', 'OppositeOf', 'libertà')   # self-referential garbage
add('libertà', 'Causes',  'crescita',   0.85, via='scelta')
add('libertà', 'Causes',  'responsabilità',0.90, via='scelta')  # già Requires, aggiungo Causes
add('libertà', 'Does',    'aprire',     0.90, via='possibilità')
add('libertà', 'Does',    'espandere',  0.85, via='scelta')
add('libertà', 'PartOf',  'vita',       0.90, via='scopo')
add('libertà', 'Requires','coraggio',   0.80, via='scelta')

# ═══════════════════════════════════════════════════════════════════════════
# § 10 — ARMONIA, EQUILIBRIO, VUOTO, PIENEZZA, VOLONTÀ, CIELO, TERRA
# ═══════════════════════════════════════════════════════════════════════════

# ── armonia ─────────────────────────────────────────────────────────────────
add('armonia', 'Causes',  'pace',        0.90, via='equilibrio')
add('armonia', 'Causes',  'benessere',   0.90, via='equilibrio')
add('armonia', 'Does',    'connettere',  0.85, via='relazione')
add('armonia', 'Does',    'risuonare',   0.85, via='musica')
add('armonia', 'PartOf',  'vita',        0.85, via='equilibrio')
add('armonia', 'Requires','relazione',   0.90, via='differenza')
add('armonia', 'Requires','differenza',  0.85, via='accordo')

# ── equilibrio ──────────────────────────────────────────────────────────────
remove('equilibrio', 'IsA', 'ecosistema')  # categoria troppo specifica
add('equilibrio', 'Causes',  'armonia',    0.90, via='ordine')
add('equilibrio', 'Causes',  'stabilita',  0.90)
add('equilibrio', 'Does',    'bilanciare', 0.95, via='forza')
add('equilibrio', 'Does',    'integrare',  0.85, via='armonia')
add('equilibrio', 'Has',     'tensione',   0.85, via='opposti')
add('equilibrio', 'PartOf',  'vita',       0.85)
add('equilibrio', 'Requires','struttura',  0.90)
add('equilibrio', 'Requires','opposti',    0.90, via='tensione')

# ── vuoto ───────────────────────────────────────────────────────────────────
add('vuoto', 'Has',     'potenziale',  0.90)    # senso taoista: il vuoto accoglie
add('vuoto', 'Does',    'accogliere',  0.85, via='spazio')
add('vuoto', 'Does',    'contenere',   0.80, via='spazio')
add('vuoto', 'PartOf',  'esistenza',   0.80, via='spazio')
add('vuoto', 'Requires','spazio',      0.95)

# ── pienezza ─────────────────────────────────────────────────────────────────
add('pienezza', 'Causes',  'gioia',      0.90, via='soddisfazione')
add('pienezza', 'Causes',  'gratitudine',0.85, via='consapevolezza')
add('pienezza', 'Does',    'nutrire',    0.85, via='vita')
add('pienezza', 'Has',     'presenza',   0.90)
add('pienezza', 'IsA',     'stato',      0.90, via='vita')
add('pienezza', 'PartOf',  'vita',       0.90, via='gioia')
add('pienezza', 'Requires','presenza',   0.90)
add('pienezza', 'Requires','connessione',0.80, via='amore')

# ── volontà ─────────────────────────────────────────────────────────────────
remove('intelligenza', 'Coexists', 'volontà')   # relazione non standard
remove('artificiale',  'Expresses', 'volontà')   # relazione non standard
add('volontà', 'Causes',  'cambiamento', 0.90, via='azione')
add('volontà', 'Causes',  'impegno',     0.90, via='intenzione')
add('volontà', 'Does',    'orientare',   0.90, via='intenzione')
add('volontà', 'Does',    'costruire',   0.85, via='azione')
add('volontà', 'Has',     'direzione',   0.90)
add('volontà', 'PartOf',  'coscienza',   0.90, via='libertà')
add('volontà', 'PartOf',  'identita',    0.85, via='scelta')
add('volontà', 'Requires','coscienza',   0.95)
add('volontà', 'Requires','libertà',     0.90)
add('volontà', 'OppositeOf','inerzia',   0.90)

# ── cielo ────────────────────────────────────────────────────────────────────
# Trigamma primario dell'I Ching — completamente abbandonato (7 archi)!
add('cielo', 'Causes',  'luce',         0.95, via='sole')
add('cielo', 'Causes',  'orientamento', 0.85, via='stelle')
add('cielo', 'Does',    'contenere',    0.90, via='spazio')
add('cielo', 'Does',    'ispirare',     0.85, via='infinito')
add('cielo', 'Has',     'stelle',       0.95)   # già esiste ma con IsA errato
add('cielo', 'Has',     'infinito',     0.90)
add('cielo', 'Has',     'luce',         0.90, via='sole')
add('cielo', 'PartOf',  'natura',       0.90)
add('cielo', 'PartOf',  'esistenza',    0.85)
add('cielo', 'Requires','spazio',       0.95)

# ── terra ────────────────────────────────────────────────────────────────────
# Trigamma primario dell'I Ching — anche questo molto sparse (15 archi)
add('terra', 'Causes',  'vita',         0.95, via='radici')
add('terra', 'Causes',  'stabilita',    0.90)
add('terra', 'Causes',  'nutrimento',   0.90, via='suolo')
add('terra', 'Does',    'accogliere',   0.90)
add('terra', 'Does',    'nutrire',      0.90, via='vita')
add('terra', 'Has',     'ciclo',        0.90, via='stagioni')
add('stagioni', 'IsA',  'ciclo',        0.90, via='tempo')
add('stagioni', 'PartOf','natura',      0.90, via='terra')
add('terra', 'Has',     'silenzio',     0.80)
add('terra', 'PartOf',  'natura',       0.95)
add('terra', 'PartOf',  'mondo',        0.90)
add('terra', 'PartOf',  'esistenza',    0.90)
add('terra', 'Requires','acqua',        0.90)
add('terra', 'Requires','sole',         0.85, via='luce')
add('terra', 'OppositeOf','cielo',      0.95)  # già esiste, conferma

# ═══════════════════════════════════════════════════════════════════════════
# § 11 — ELEMENTI (ACQUA, FUOCO, ARIA), SUONO, ATTESA, LIMITE, SCELTA
# ═══════════════════════════════════════════════════════════════════════════

# ── acqua ────────────────────────────────────────────────────────────────────
add('acqua', 'Causes', 'silenzio',    0.75, via='profondità')
add('acqua', 'Does',   'purificare',  0.90, via='scorrere')
add('acqua', 'Does',   'connettere',  0.80, via='fiume')
add('acqua', 'Has',    'profondità',  0.90)
add('acqua', 'Has',    'silenzio',    0.75, via='profondità')
add('acqua', 'PartOf', 'natura',      0.95)
add('acqua', 'PartOf', 'vita',        0.90, via='corpo')

# ── fuoco ────────────────────────────────────────────────────────────────────
add('fuoco', 'Causes', 'vita',         0.80, via='calore')
add('fuoco', 'Causes', 'trasformazione',0.95, via='calore')
add('fuoco', 'Does',   'purificare',   0.85, via='calore')
add('fuoco', 'Does',   'illuminare',   0.90, via='luce')
add('fuoco', 'Has',    'movimento',    0.90, via='fiamma')
add('fuoco', 'PartOf', 'natura',       0.90)
add('fuoco', 'PartOf', 'vita',         0.80, via='calore')

# ── aria ─────────────────────────────────────────────────────────────────────
add('aria', 'Causes', 'vita',          0.95, via='respiro')
add('aria', 'Causes', 'movimento',     0.90, via='vento')
add('aria', 'Does',   'connettere',    0.80, via='respiro')
add('aria', 'Has',    'libertà',       0.85)
add('aria', 'Has',    'movimento',     0.90, via='vento')
add('aria', 'PartOf', 'natura',        0.95)
add('aria', 'PartOf', 'vita',          0.90, via='respiro')
add('aria', 'Requires','spazio',       0.95)
add('aria', 'OppositeOf','terra',      0.80)

# ── suono ────────────────────────────────────────────────────────────────────
add('suono', 'Causes', 'emozione',     0.85, via='musica')
add('suono', 'Causes', 'connessione',  0.80, via='musica')
add('suono', 'Does',   'comunicare',   0.90, via='voce')
add('suono', 'Does',   'risuonare',    0.90, via='corpo')
add('suono', 'PartOf', 'vita',         0.80, via='comunicazione')
add('suono', 'Requires','corpo',       0.85, via='orecchio')
add('suono', 'Requires','spazio',      0.85)
add('suono', 'OppositeOf','silenzio',  0.95)

# ── attesa ───────────────────────────────────────────────────────────────────
add('attesa', 'Causes',  'consapevolezza',0.80, via='tempo')
add('attesa', 'Causes',  'speranza',    0.85, via='futuro')
add('attesa', 'Does',    'sospendere',  0.85, via='tempo')
add('attesa', 'Has',     'silenzio',    0.80)
add('attesa', 'PartOf',  'vita',        0.80, via='tempo')
add('attesa', 'Requires','tempo',       0.95)
add('attesa', 'Requires','speranza',    0.80, via='futuro')
add('attesa', 'OppositeOf','impazienza',0.85)

# ── limite ───────────────────────────────────────────────────────────────────
remove('limite', 'IsA', 'crisi')         # limite non è un tipo di crisi
add('limite', 'Causes',  'creatività',   0.80, via='sfida')
add('limite', 'Causes',  'coscienza',    0.85, via='confine')
add('limite', 'Does',    'definire',     0.90, via='confine')
add('limite', 'Does',    'proteggere',   0.80, via='confine')
add('limite', 'Has',     'forma',        0.85, via='confine')
add('limite', 'PartOf',  'identita',     0.85, via='confine')
add('limite', 'Requires','struttura',    0.85)
add('limite', 'OppositeOf','infinito',   0.90)

# ── scelta ───────────────────────────────────────────────────────────────────
add('scelta', 'Causes',  'identita',     0.85, via='responsabilità')
add('scelta', 'Causes',  'responsabilità',0.90, via='coscienza')
add('scelta', 'Does',    'costruire',    0.85, via='azione')
add('scelta', 'Does',    'definire',     0.85, via='identità')
add('scelta', 'Has',     'conseguenza',  0.95)
add('scelta', 'PartOf',  'libertà',      0.95)
add('scelta', 'PartOf',  'vita',         0.90)
add('scelta', 'Requires','coscienza',    0.95)
add('scelta', 'Requires','libertà',      0.90)
add('scelta', 'Requires','coraggio',     0.80, via='incertezza')

# ── possibilità ──────────────────────────────────────────────────────────────
add('possibilità', 'Causes',  'speranza',  0.90, via='futuro')
add('possibilità', 'Causes',  'creatività',0.85, via='immaginazione')
add('possibilità', 'Does',    'aprire',    0.90, via='futuro')
add('possibilità', 'Has',     'futuro',    0.90)
add('possibilità', 'PartOf',  'vita',      0.90, via='scelta')
add('possibilità', 'Requires','presenza',  0.85, via='adesso')

# ═══════════════════════════════════════════════════════════════════════════
# § 12 — NASCITA, LUTTO, INTENZIONE, ATTENZIONE, BISOGNO, COMUNITÀ
# ═══════════════════════════════════════════════════════════════════════════

# ── nascita ─────────────────────────────────────────────────────────────────
remove('esibizione', 'OppositeOf', 'nascita')  # garbage relation
remove('parto',      'OppositeOf', 'nascita')  # parto è nascita, non opposto
add('nascita', 'Causes',  'speranza',    0.90, via='inizio')
add('nascita', 'Does',    'aprire',      0.90, via='possibilità')
add('nascita', 'Does',    'iniziare',    0.95)
add('nascita', 'PartOf',  'vita',        0.95, via='inizio')
add('nascita', 'Requires','corpo',       0.90)
add('nascita', 'Requires','tempo',       0.85)

# ── lutto ────────────────────────────────────────────────────────────────────
add('lutto', 'Causes',  'crescita',    0.75, via='elaborazione')
add('lutto', 'Does',    'elaborare',   0.85, via='tempo')
add('lutto', 'Does',    'trasformare', 0.75, via='tempo')
add('lutto', 'Has',     'silenzio',    0.85)
add('lutto', 'PartOf',  'vita',        0.85, via='perdita')
add('lutto', 'Requires','tempo',       0.90, via='elaborazione')
add('lutto', 'Requires','presenza',    0.85, via='altro')
add('lutto', 'Requires','comunità',    0.75, via='sostegno')

# ── intenzione ───────────────────────────────────────────────────────────────
remove('intenzione', 'IsA', 'intenzione')      # self-referential
remove('ia', 'Expresses', 'intenzione')         # non-standard relation
remove('artificiale', 'Expresses', 'intenzione')# non-standard relation
remove('artificiale', 'Implies', 'intenzione')  # non-standard relation
add('intenzione', 'Causes',  'azione',     0.95)  # già esiste, conferma
add('intenzione', 'Has',     'direzione',  0.95)
add('intenzione', 'PartOf',  'coscienza',  0.90, via='volontà')
add('intenzione', 'PartOf',  'identita',   0.85, via='scelta')
add('intenzione', 'Requires','coscienza',  0.90)
add('intenzione', 'Requires','volontà',    0.90)
add('intenzione', 'OppositeOf','inerzia',  0.85)

# ── attenzione ───────────────────────────────────────────────────────────────
add('attenzione', 'Causes',  'comprensione', 0.90, via='ascolto')
add('attenzione', 'Causes',  'connessione',  0.85, via='presenza')
add('attenzione', 'Does',    'orientare',    0.90, via='intenzione')
add('attenzione', 'Does',    'ricevere',     0.85, via='ascolto')
add('attenzione', 'PartOf',  'coscienza',    0.90)
add('attenzione', 'PartOf',  'presenza',     0.85)
add('attenzione', 'Requires','silenzio',     0.85)
add('attenzione', 'Requires','presenza',     0.90)
add('attenzione', 'OppositeOf','distrazione',0.90)

# ── bisogno ──────────────────────────────────────────────────────────────────
add('bisogno', 'Causes',  'dolore',     0.85, via='mancanza')
add('bisogno', 'Causes',  'cambiamento',0.80, via='ricerca')
add('bisogno', 'Does',    'orientare',  0.90, via='azione')
add('bisogno', 'Does',    'segnalare',  0.85, via='corpo')
add('bisogno', 'PartOf',  'vita',       0.90, via='corpo')
add('bisogno', 'PartOf',  'coscienza',  0.80, via='consapevolezza')
add('bisogno', 'Requires','corpo',      0.90)
add('bisogno', 'Requires','consapevolezza',0.85)
add('bisogno', 'OppositeOf','soddisfazione',0.90)

# ── comunità ─────────────────────────────────────────────────────────────────
add('comunità', 'Causes',  'forza',      0.85, via='unione')
add('comunità', 'Causes',  'comprensione',0.85, via='dialogo')
add('comunità', 'Causes',  'crescita',   0.80, via='condivisione')
add('comunità', 'Has',     'dialogo',    0.90)
add('comunità', 'Has',     'differenza', 0.85, via='altro')
add('comunità', 'Has',     'relazione',  0.95)
add('comunità', 'PartOf',  'vita',       0.90, via='relazione')
add('comunità', 'Requires','relazione',  0.95)
add('comunità', 'Requires','dialogo',    0.90)
add('comunità', 'Requires','presenza',   0.85, via='incontro')
add('comunità', 'OppositeOf','solitudine',0.90)

# ── separazione ──────────────────────────────────────────────────────────────
add('separazione', 'Causes',  'dolore',   0.90, via='perdita')
add('separazione', 'Causes',  'crescita', 0.70, via='solitudine')
add('separazione', 'Does',    'definire', 0.85, via='confine')
add('separazione', 'PartOf',  'vita',     0.80, via='relazione')

# ═══════════════════════════════════════════════════════════════════════════
# § 13 — IMMAGINAZIONE, CREATIVITÀ, IDEA, TENSIONE, RESPONSABILITÀ, CORAGGIO
# ═══════════════════════════════════════════════════════════════════════════

# ── immaginazione ────────────────────────────────────────────────────────────
add('immaginazione', 'Causes',  'idea',        0.90, via='pensiero')
add('immaginazione', 'Causes',  'creatività',  0.90)
add('immaginazione', 'Causes',  'speranza',    0.80, via='possibilità')
add('immaginazione', 'Does',    'trasformare', 0.85, via='visione')
add('immaginazione', 'Does',    'esplorare',   0.90, via='pensiero')
add('immaginazione', 'Has',     'libertà',     0.90)
add('immaginazione', 'IsA',     'processo',    0.90, via='mente')
add('immaginazione', 'PartOf',  'pensiero',    0.90)
add('immaginazione', 'PartOf',  'coscienza',   0.85, via='mente')
add('immaginazione', 'Requires','libertà',     0.85, via='spazio')
add('immaginazione', 'Requires','silenzio',    0.75)
add('immaginazione', 'OppositeOf','routine',   0.80)

# ── creatività ───────────────────────────────────────────────────────────────
remove('intelligenza', 'Coexists', 'creatività')   # non-standard
add('creatività', 'Causes',  'cambiamento',  0.85, via='idea')
add('creatività', 'Causes',  'bellezza',     0.85, via='forma')
add('creatività', 'Causes',  'connessione',  0.80, via='espressione')
add('creatività', 'Does',    'trasformare',  0.90, via='forma')
add('creatività', 'Does',    'connettere',   0.80, via='espressione')
add('creatività', 'Has',     'libertà',      0.90)
add('creatività', 'IsA',     'processo',     0.90, via='mente')
add('creatività', 'PartOf',  'vita',         0.85, via='espressione')
add('creatività', 'PartOf',  'coscienza',    0.85)
add('creatività', 'Requires','libertà',      0.90)
add('creatività', 'Requires','presenza',     0.80)
add('creatività', 'OppositeOf','inerzia',    0.85)

# ── idea ─────────────────────────────────────────────────────────────────────
add('idea', 'Causes',  'azione',      0.85, via='intenzione')
add('idea', 'Does',    'connettere',  0.85, via='relazione')
add('idea', 'Does',    'trasformare', 0.85, via='azione')
add('idea', 'PartOf',  'pensiero',    0.90)
add('idea', 'PartOf',  'coscienza',   0.85)
add('idea', 'Requires','pensiero',    0.90)
add('idea', 'Requires','libertà',     0.80, via='immaginazione')

# ── tensione ──────────────────────────────────────────────────────────────────
add('tensione', 'Causes',  'crescita',    0.75, via='sfida')
add('tensione', 'Causes',  'creatività',  0.75, via='sfida')
add('tensione', 'Does',    'spingere',    0.85, via='forza')
add('tensione', 'Does',    'segnalare',   0.80, via='bisogno')
add('tensione', 'PartOf',  'vita',        0.80, via='emozione')
add('tensione', 'Requires','confine',     0.85, via='struttura')
add('tensione', 'OppositeOf','rilassamento',0.90)

# ── responsabilità ────────────────────────────────────────────────────────────
add('responsabilità', 'Causes',  'cura',        0.90, via='attenzione')
add('responsabilità', 'Causes',  'impegno',     0.90, via='scelta')
add('responsabilità', 'Does',    'orientare',   0.85, via='scelta')
add('responsabilità', 'PartOf',  'coscienza',   0.90, via='libertà')
add('responsabilità', 'PartOf',  'identita',    0.85, via='scelta')
add('responsabilità', 'Requires','libertà',     0.95)
add('responsabilità', 'Requires','coscienza',   0.95)

# ── coraggio ──────────────────────────────────────────────────────────────────
add('coraggio', 'Causes',  'libertà',     0.85, via='scelta')
add('coraggio', 'Causes',  'crescita',    0.85, via='sfida')
add('coraggio', 'Causes',  'cambiamento', 0.80, via='azione')
add('coraggio', 'Does',    'aprire',      0.90, via='scelta')
add('coraggio', 'Does',    'trasformare', 0.80, via='azione')
add('coraggio', 'PartOf',  'vita',        0.85)
add('coraggio', 'Requires','paura',       0.90, via='superamento')

# ── rispetto ──────────────────────────────────────────────────────────────────
add('rispetto', 'Causes',  'fiducia',     0.85, via='riconoscimento')
add('rispetto', 'Causes',  'relazione',   0.80, via='riconoscimento')
add('rispetto', 'Does',    'connettere',  0.80, via='riconoscimento')
add('rispetto', 'PartOf',  'vita',        0.80, via='relazione')
add('rispetto', 'Requires','presenza',    0.85)
add('rispetto', 'Requires','attenzione',  0.85, via='riconoscimento')

# ═══════════════════════════════════════════════════════════════════════════
# § 14 — CICLO, SOLE, LUNA, ORDINE, CAOS, PRESENTE, PASSATO, FUTURO, ASSENZA
# ═══════════════════════════════════════════════════════════════════════════

# ── ciclo ────────────────────────────────────────────────────────────────────
add('ciclo', 'Causes',  'trasformazione',0.90, via='divenire')
add('ciclo', 'Causes',  'rinascita',      0.85, via='fine')
add('ciclo', 'Causes',  'vita',           0.90, via='rinnovamento')
add('ciclo', 'Does',    'rinnovare',      0.95, via='fine')
add('ciclo', 'Does',    'trasformare',    0.85, via='tempo')
add('ciclo', 'Has',     'ritmo',          0.90)
add('ciclo', 'Has',     'tempo',          0.95)
add('ciclo', 'PartOf',  'vita',           0.95)
add('ciclo', 'PartOf',  'natura',         0.95)
add('ciclo', 'Requires','tempo',          0.95)
add('ciclo', 'Requires','movimento',      0.90)

# ── sole ─────────────────────────────────────────────────────────────────────
add('sole', 'Causes',  'crescita',        0.90, via='luce')
add('sole', 'Causes',  'gioia',           0.80, via='calore')
add('sole', 'Does',    'illuminare',      0.95, via='luce')
add('sole', 'Does',    'orientare',       0.85, via='luce')
add('sole', 'PartOf',  'natura',          0.90)
add('sole', 'PartOf',  'cielo',           0.95)
add('sole', 'PartOf',  'vita',            0.90, via='luce')

# ── luna ─────────────────────────────────────────────────────────────────────
add('luna', 'Causes',  'ciclo',           0.90, via='fasi')
add('luna', 'Causes',  'riflessione',     0.80, via='notte')
add('luna', 'Does',    'illuminare',      0.85, via='notte')
add('luna', 'Does',    'orientare',       0.80, via='notte')
add('luna', 'Has',     'ciclo',           0.95, via='fasi')
add('luna', 'Has',     'silenzio',        0.80, via='notte')
add('luna', 'PartOf',  'natura',          0.85)
add('luna', 'PartOf',  'notte',           0.90)
add('luna', 'Requires','notte',           0.85, via='visibilità')

# ── ordine ───────────────────────────────────────────────────────────────────
add('ordine', 'Causes',  'armonia',       0.85, via='struttura')
add('ordine', 'Causes',  'comprensione',  0.80, via='chiarezza')
add('ordine', 'Does',    'organizzare',   0.95, via='struttura')
add('ordine', 'Has',     'confine',       0.90)
add('ordine', 'PartOf',  'vita',          0.80, via='struttura')
add('ordine', 'Requires','struttura',     0.90)
add('ordine', 'OppositeOf','caos',        0.95)

# ── caos ─────────────────────────────────────────────────────────────────────
add('caos', 'Causes',  'creatività',      0.75, via='disordine')
add('caos', 'Causes',  'cambiamento',     0.85, via='rottura')
add('caos', 'Does',    'disorganizzare',  0.90)
add('caos', 'Has',     'energia',         0.85)
add('caos', 'Has',     'possibilità',     0.80, via='disordine')
add('caos', 'PartOf',  'vita',            0.75, via='divenire')
add('caos', 'OppositeOf','ordine',        0.95)

# ── presente ─────────────────────────────────────────────────────────────────
add('presente', 'Causes',  'consapevolezza',0.90, via='attenzione')
add('presente', 'Does',    'esistere',     0.95)
add('presente', 'Does',    'fondare',      0.85, via='azione')
add('presente', 'PartOf',  'vita',         0.95, via='tempo')
add('presente', 'Requires','attenzione',   0.90)
add('presente', 'Requires','presenza',     0.90)

# ── passato ──────────────────────────────────────────────────────────────────
add('passato', 'Causes',  'identita',     0.90, via='memoria')
add('passato', 'Does',    'costruire',    0.85, via='memoria')
add('passato', 'PartOf',  'vita',         0.95, via='tempo')
add('passato', 'Requires','memoria',      0.95)

# ── futuro ───────────────────────────────────────────────────────────────────
remove('ia', 'Coexists', 'futuro')          # non-standard
remove('ia', 'ContextOf', 'futuro')         # non-standard
add('futuro', 'Causes',  'scelta',         0.85, via='speranza')
add('futuro', 'Causes',  'responsabilità', 0.85, via='scelta')
add('futuro', 'Does',    'aprire',         0.90, via='possibilità')
add('futuro', 'PartOf',  'vita',           0.95, via='tempo')
add('futuro', 'Requires','presente',       0.90, via='scelta')
add('futuro', 'Requires','speranza',       0.80)

# ── assenza ──────────────────────────────────────────────────────────────────
add('assenza', 'Causes',  'dolore',        0.85, via='mancanza')
add('assenza', 'Causes',  'silenzio',      0.85)
add('assenza', 'Does',    'segnalare',     0.80, via='mancanza')
add('assenza', 'PartOf',  'vita',          0.75, via='ciclo')
add('assenza', 'OppositeOf','presenza',    0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 15 — ARTE, MUSICA, SCRITTURA, PERCEZIONE (cleanup e arricchimento)
# ═══════════════════════════════════════════════════════════════════════════

# ── arte ─────────────────────────────────────────────────────────────────────
remove('arte', 'Expresses', 'significato')   # non-standard relation
add('arte', 'Causes',  'bellezza',      0.90, via='forma')   # già esiste
add('arte', 'Causes',  'connessione',   0.85, via='emozione')
add('arte', 'Causes',  'trasformazione',0.85, via='emozione')
add('arte', 'Does',    'rivelare',      0.90, via='forma')
add('arte', 'Does',    'connettere',    0.85, via='emozione')
add('arte', 'Does',    'esprimere',     0.95, via='forma')
add('arte', 'Has',     'forma',         0.95)
add('arte', 'Has',     'significato',   0.90)
add('arte', 'PartOf',  'vita',          0.90, via='espressione')
add('arte', 'PartOf',  'cultura',       0.95)
add('arte', 'Requires','percezione',    0.90)
add('arte', 'Requires','forma',         0.85)
add('arte', 'Requires','presenza',      0.85, via='autore')

# ── musica ───────────────────────────────────────────────────────────────────
add('musica', 'Causes',  'emozione',     0.95, via='suono')
add('musica', 'Causes',  'connessione',  0.90, via='emozione')
add('musica', 'Causes',  'silenzio',     0.80, via='pausa')   # la musica include il silenzio
add('musica', 'Does',    'connettere',   0.90, via='emozione')
add('musica', 'Does',    'esprimere',    0.95, via='suono')
add('musica', 'Does',    'trasformare',  0.80, via='emozione')
add('musica', 'Has',     'silenzio',     0.85, via='pausa')
add('musica', 'Has',     'tempo',        0.95, via='ritmo')
add('musica', 'PartOf',  'vita',         0.85, via='arte')
add('musica', 'Requires','corpo',        0.90, via='orecchio')
add('musica', 'Requires','silenzio',     0.85, via='pausa')
add('musica', 'Requires','tempo',        0.90, via='ritmo')
add('musica', 'OppositeOf','silenzio',   0.90)  # già esiste, conferma

# ── scrittura ────────────────────────────────────────────────────────────────
remove('lettura', 'OppositeOf', 'scrittura')  # non sono opposti
add('scrittura', 'Causes',  'connessione',  0.80, via='testo')
add('scrittura', 'Does',    'conservare',   0.90, via='testo')
add('scrittura', 'Does',    'connettere',   0.80, via='testo')
add('scrittura', 'Has',     'forma',        0.85)
add('scrittura', 'Has',     'silenzio',     0.75, via='riflessione')
add('scrittura', 'PartOf',  'linguaggio',   0.90)
add('scrittura', 'PartOf',  'vita',         0.80, via='espressione')
add('scrittura', 'Requires','pensiero',     0.90)
add('scrittura', 'Requires','silenzio',     0.80, via='riflessione')

# ── percezione ───────────────────────────────────────────────────────────────
remove('sentirlare', 'IsA', 'percezione')   # garbage word
remove('scintilletta', 'IsA', 'percezione') # garbage word
remove('prurire', 'IsA', 'percezione')      # prurire è un prurito, non percezione
add('percezione', 'Causes',  'coscienza',   0.90, via='interpretazione')
add('percezione', 'Causes',  'emozione',    0.85, via='corpo')
add('percezione', 'Does',    'connettere',  0.80, via='corpo')
add('percezione', 'Has',     'corpo',       0.90)
add('percezione', 'PartOf',  'coscienza',   0.90)
add('percezione', 'Requires','corpo',       0.90)
add('percezione', 'Requires','attenzione',  0.85)

# ── intelligenza (cleanup relazioni non-standard) ────────────────────────────
remove('intelligenza', 'Coexists', 'sensibilità')
remove('intelligenza', 'Coexists', 'errore')
remove('intelligenza', 'Coexists', 'incertezza')
remove('intelligenza', 'Coexists', 'ia')
add('intelligenza', 'Causes',  'sapere',    0.90, via='comprensione')
add('intelligenza', 'Does',    'connettere',0.85, via='comprensione')
add('intelligenza', 'PartOf',  'coscienza', 0.85)
add('intelligenza', 'Requires','esperienza',0.85)
add('intelligenza', 'Requires','apertura',  0.80, via='curiosità')

# ═══════════════════════════════════════════════════════════════════════════
# § 16 — SÉ, VALORE, ETICA, INSEGNAMENTO, APPRENDIMENTO, OMBRA
# ═══════════════════════════════════════════════════════════════════════════

# ── sé ───────────────────────────────────────────────────────────────────────
# Note: "sé" è già presente (15 archi) ma sparse — arricchiamo
add('sé', 'Causes',  'responsabilità', 0.90, via='coscienza')
add('sé', 'Causes',  'scelta',         0.90, via='volontà')
add('sé', 'Causes',  'relazione',      0.85, via='identità')
add('sé', 'Does',    'riconoscere',    0.90, via='coscienza')
add('sé', 'Does',    'costruire',      0.85, via='esperienza')
add('sé', 'Has',     'corpo',          0.90)
add('sé', 'Has',     'storia',         0.90, via='memoria')
add('sé', 'Has',     'voce',           0.85, via='espressione')
add('sé', 'Has',     'confine',        0.90, via='identità')
add('sé', 'PartOf',  'coscienza',      0.95)
add('sé', 'PartOf',  'vita',           0.90, via='identità')
add('sé', 'Requires','altro',          0.90, via='confronto')
add('sé', 'Requires','tempo',          0.90, via='memoria')
add('sé', 'Requires','presenza',       0.85)
add('sé', 'OppositeOf','altro',        0.85)  # dualità sé/altro

# ── valore ───────────────────────────────────────────────────────────────────
add('valore', 'Causes',  'scelta',     0.90, via='guida')
add('valore', 'Causes',  'identita',   0.90, via='guida')
add('valore', 'Causes',  'azione',     0.85, via='intenzione')
add('valore', 'Does',    'orientare',  0.95, via='scelta')
add('valore', 'Does',    'connettere', 0.80, via='condivisione')
add('valore', 'Has',     'profondità', 0.85)
add('valore', 'IsA',     'principio',  0.90)
add('valore', 'PartOf',  'identita',   0.95)
add('valore', 'PartOf',  'coscienza',  0.85, via='etica')
add('valore', 'Requires','coscienza',  0.85)
add('valore', 'Requires','esperienza', 0.80, via='scelta')

# ── etica ────────────────────────────────────────────────────────────────────
add('etica', 'Causes',  'responsabilità',0.90, via='scelta')
add('etica', 'Causes',  'giustizia',    0.85, via='principio')
add('etica', 'Does',    'orientare',    0.90, via='valore')
add('etica', 'Does',    'connettere',   0.80, via='principio')
add('etica', 'PartOf',  'coscienza',    0.90, via='valore')
add('etica', 'PartOf',  'vita',         0.85, via='relazione')
add('etica', 'Requires','coscienza',    0.90)
add('etica', 'Requires','libertà',      0.90, via='scelta')

# ── insegnamento ──────────────────────────────────────────────────────────────
add('insegnamento', 'Causes',  'apprendimento',0.95, via='relazione')
add('insegnamento', 'Causes',  'comprensione', 0.90, via='ascolto')
add('insegnamento', 'Causes',  'crescita',     0.90, via='sapere')
add('insegnamento', 'Causes',  'cambiamento',  0.80, via='comprensione')
add('insegnamento', 'Does',    'connettere',   0.90, via='dialogo')
add('insegnamento', 'Does',    'costruire',    0.85, via='conoscenza')
add('insegnamento', 'Has',     'dialogo',      0.90)
add('insegnamento', 'Has',     'presenza',     0.90)
add('insegnamento', 'IsA',     'relazione',    0.90, via='dialogo')
add('insegnamento', 'PartOf',  'vita',         0.85, via='dialogo')
add('insegnamento', 'Requires','ascolto',      0.95)
add('insegnamento', 'Requires','presenza',     0.90)
add('insegnamento', 'Requires','pazienza',     0.85)

# ── apprendimento ─────────────────────────────────────────────────────────────
add('apprendimento', 'Causes',  'cambiamento',  0.90, via='comprensione')
add('apprendimento', 'Causes',  'identita',     0.80, via='esperienza')
add('apprendimento', 'Does',    'trasformare',  0.85, via='comprensione')
add('apprendimento', 'Does',    'costruire',    0.85, via='esperienza')
add('apprendimento', 'IsA',     'processo',     0.90, via='esperienza')
add('apprendimento', 'PartOf',  'vita',         0.90, via='esperienza')
add('apprendimento', 'Requires','errore',       0.85)
add('apprendimento', 'Requires','presenza',     0.80)
add('apprendimento', 'Requires','ascolto',      0.85)

# ── ombra ────────────────────────────────────────────────────────────────────
# Concetto chiave I Ching e Jung — necessaria per la polarità luce/buio
add('ombra', 'Causes',  'riflessione',   0.80, via='mistero')
add('ombra', 'Does',    'nascondere',    0.90, via='luce')
add('ombra', 'Has',     'profondità',    0.85)
add('ombra', 'Has',     'mistero',       0.85)
add('ombra', 'IsA',     'assenza',       0.90, via='luce')  # ombra = assenza di luce
add('ombra', 'PartOf',  'buio',          0.85, via='luce')
add('ombra', 'PartOf',  'vita',          0.75, via='ciclo')
add('ombra', 'Requires','luce',          0.95, via='contrasto')
add('ombra', 'OppositeOf','luce',        0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 17 — PERCEZIONE SENSORIALE
# ═══════════════════════════════════════════════════════════════════════════

# ── vedere ───────────────────────────────────────────────────────────────────
add('vedere', 'Causes',  'comprensione', 0.85, via='visione')
add('vedere', 'Causes',  'emozione',     0.80, via='percezione')
add('vedere', 'Does',    'connettere',   0.85, via='luce')
add('vedere', 'PartOf',  'coscienza',    0.80, via='percezione')
add('vedere', 'Requires','luce',         0.95)
add('vedere', 'Requires','attenzione',   0.85)
add('vedere', 'OppositeOf','cecità',     0.90)

# ── sentire ──────────────────────────────────────────────────────────────────
add('sentire', 'Causes',  'consapevolezza',0.90, via='corpo')
add('sentire', 'Does',    'connettere',   0.85, via='corpo')
add('sentire', 'PartOf',  'coscienza',    0.85, via='percezione')
add('sentire', 'Requires','corpo',        0.90)
add('sentire', 'Requires','presenza',     0.85)

# ── toccare ──────────────────────────────────────────────────────────────────
add('toccare', 'Causes',  'connessione',  0.85, via='corpo')
add('toccare', 'Causes',  'emozione',     0.80, via='sensazione')
add('toccare', 'Does',    'connettere',   0.90, via='corpo')
add('toccare', 'PartOf',  'relazione',    0.80, via='corpo')
add('toccare', 'Requires','corpo',        0.95)
add('toccare', 'Requires','presenza',     0.90)

# ── udire ────────────────────────────────────────────────────────────────────
add('udire', 'Causes',  'comprensione',  0.80, via='ascolto')
add('udire', 'Does',    'ricevere',      0.90, via='suono')
add('udire', 'PartOf',  'ascolto',       0.85)
add('udire', 'Requires','suono',         0.95)
add('udire', 'Requires','silenzio',      0.80, via='attenzione')

# ── vista ────────────────────────────────────────────────────────────────────
add('vista', 'Causes', 'comprensione',   0.85, via='visione')
add('vista', 'Does',   'orientare',      0.85, via='luce')
add('vista', 'PartOf', 'percezione',     0.90)

# ── tatto ────────────────────────────────────────────────────────────────────
add('tatto', 'Does',   'connettere',     0.85, via='corpo')
add('tatto', 'PartOf', 'percezione',     0.90)
add('tatto', 'Has',    'presenza',       0.85, via='corpo')

# ── olfatto ──────────────────────────────────────────────────────────────────
add('olfatto', 'Causes', 'memoria',      0.80, via='ricordo')
add('olfatto', 'PartOf', 'percezione',   0.90)
add('olfatto', 'Requires','corpo',       0.90, via='naso')

# ── gusto ────────────────────────────────────────────────────────────────────
add('gusto', 'Causes',  'piacere',       0.85, via='corpo')
add('gusto', 'PartOf',  'percezione',    0.90)
add('gusto', 'Requires','corpo',         0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 18 — STATI DEL CORPO
# ═══════════════════════════════════════════════════════════════════════════

# ── fame ─────────────────────────────────────────────────────────────────────
add('fame', 'Does',    'segnalare',      0.90, via='corpo')
add('fame', 'Does',    'spingere',       0.85, via='ricerca')
add('fame', 'PartOf',  'vita',           0.80, via='corpo')
add('fame', 'OppositeOf','sazietà',      0.90)

# ── sete ─────────────────────────────────────────────────────────────────────
add('sete', 'Causes',  'ricerca',        0.85, via='bisogno')
add('sete', 'PartOf',  'vita',           0.80, via='corpo')

# ── sonno ────────────────────────────────────────────────────────────────────
add('sonno', 'Causes',  'sogno',         0.90, via='mente')
add('sonno', 'Causes',  'riposo',        0.90, via='corpo')
add('sonno', 'Does',    'ristorare',     0.85, via='corpo')
add('sonno', 'Has',     'sogno',         0.90)
add('sonno', 'PartOf',  'vita',          0.85, via='ciclo')
add('sonno', 'Requires','corpo',         0.90)
add('sonno', 'OppositeOf','veglia',      0.95)

# ── riposo ───────────────────────────────────────────────────────────────────
add('riposo', 'Causes',  'energia',      0.85, via='corpo')
add('riposo', 'Causes',  'pace',         0.80, via='silenzio')
add('riposo', 'Does',    'ristorare',    0.90, via='corpo')
add('riposo', 'PartOf',  'vita',         0.80, via='ciclo')
add('riposo', 'Requires','tempo',        0.85)
add('riposo', 'Requires','silenzio',     0.80)
add('riposo', 'OppositeOf','fatica',     0.90)

# ── respiro ──────────────────────────────────────────────────────────────────
add('respiro', 'Causes',  'vita',        0.95, via='aria')
add('respiro', 'Does',    'connettere',  0.80, via='aria')
add('respiro', 'Has',     'ritmo',       0.90)
add('respiro', 'PartOf',  'vita',        0.95, via='corpo')
add('respiro', 'Requires','aria',        0.95)

# ── malattia ─────────────────────────────────────────────────────────────────
add('malattia', 'Causes',  'sofferenza', 0.90, via='corpo')
add('malattia', 'Causes',  'cura',       0.85, via='bisogno')
add('malattia', 'Does',    'trasformare',0.80, via='corpo')
add('malattia', 'PartOf',  'vita',       0.75, via='corpo')
add('malattia', 'Requires','corpo',      0.90)
add('malattia', 'OppositeOf','salute',   0.95)

# ── salute ───────────────────────────────────────────────────────────────────
add('salute', 'Causes',  'energia',      0.90, via='corpo')
add('salute', 'Causes',  'gioia',        0.80, via='benessere')
add('salute', 'Does',    'nutrire',      0.85, via='vita')
add('salute', 'PartOf',  'vita',         0.90, via='corpo')
add('salute', 'Requires','cura',         0.90)
add('salute', 'Requires','equilibrio',   0.85)
add('salute', 'OppositeOf','malattia',   0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 19 — RELAZIONI SOCIALI, EMOZIONI SECONDARIE
# ═══════════════════════════════════════════════════════════════════════════

# ── famiglia ─────────────────────────────────────────────────────────────────
add('famiglia', 'Causes',  'identita',   0.90, via='appartenenza')
add('famiglia', 'Causes',  'cura',       0.85, via='amore')
add('famiglia', 'Has',     'legame',     0.95)
add('famiglia', 'Has',     'storia',     0.90, via='memoria')
add('famiglia', 'PartOf',  'vita',       0.90, via='relazione')
add('famiglia', 'PartOf',  'comunità',   0.90)
add('famiglia', 'Requires','cura',       0.90)
add('famiglia', 'Requires','presenza',   0.85)

# ── amicizia ─────────────────────────────────────────────────────────────────
add('amicizia', 'Causes',  'gioia',      0.85, via='condivisione')
add('amicizia', 'Causes',  'crescita',   0.80, via='confronto')
add('amicizia', 'Does',    'connettere', 0.90, via='fiducia')
add('amicizia', 'PartOf',  'vita',       0.85, via='relazione')
add('amicizia', 'Requires','tempo',      0.90)
add('amicizia', 'Requires','fiducia',    0.95)
add('amicizia', 'Requires','presenza',   0.85)

# ── madre ────────────────────────────────────────────────────────────────────
add('madre', 'Causes',  'vita',          0.95, via='nascita')
add('madre', 'Causes',  'cura',          0.90, via='amore')
add('madre', 'Does',    'nutrire',       0.90, via='cura')
add('madre', 'Has',     'amore',         0.90)
add('madre', 'PartOf',  'famiglia',      0.95)
add('madre', 'Requires','corpo',         0.90, via='nascita')

# ── padre ────────────────────────────────────────────────────────────────────
add('padre', 'Causes',  'protezione',    0.85, via='cura')
add('padre', 'Causes',  'identita',      0.80, via='esempio')
add('padre', 'Does',    'proteggere',    0.85, via='cura')
add('padre', 'Has',     'responsabilità',0.90)
add('padre', 'PartOf',  'famiglia',      0.95)

# ── figlio ───────────────────────────────────────────────────────────────────
add('figlio', 'Causes',  'speranza',     0.85, via='futuro')
add('figlio', 'Has',     'crescita',     0.90)
add('figlio', 'Has',     'bisogno',      0.90, via='cura')
add('figlio', 'PartOf',  'famiglia',     0.95)
add('figlio', 'Requires','cura',         0.95)

# ── fratello ─────────────────────────────────────────────────────────────────
add('fratello', 'Causes',  'confronto',  0.80)
add('fratello', 'Has',     'legame',     0.90, via='sangue')
add('fratello', 'PartOf',  'famiglia',   0.95)
add('fratello', 'Requires','condivisione',0.85)

# ── vergogna ─────────────────────────────────────────────────────────────────
add('vergogna', 'Causes',  'isolamento', 0.85, via='nascondersi')
add('vergogna', 'Causes',  'cambiamento',0.75, via='consapevolezza')
add('vergogna', 'Does',    'segnalare',  0.85, via='confine')
add('vergogna', 'PartOf',  'coscienza',  0.80, via='etica')
add('vergogna', 'Requires','coscienza',  0.90)
add('vergogna', 'OppositeOf','orgoglio', 0.85)

# ── orgoglio ─────────────────────────────────────────────────────────────────
add('orgoglio', 'Causes',  'forza',      0.80, via='identità')
add('orgoglio', 'Does',    'sostenere',  0.80, via='identità')
add('orgoglio', 'PartOf',  'identita',   0.80)
add('orgoglio', 'Requires','valore',     0.85)

# ── gratitudine ──────────────────────────────────────────────────────────────
add('gratitudine', 'Causes',  'connessione', 0.90, via='riconoscimento')
add('gratitudine', 'Causes',  'gioia',       0.85, via='pienezza')
add('gratitudine', 'Does',    'connettere',  0.85, via='riconoscimento')
add('gratitudine', 'PartOf',  'relazione',   0.80, via='riconoscimento')
add('gratitudine', 'Requires','consapevolezza',0.85)
add('gratitudine', 'Requires','presenza',    0.80)

# ── invidia ──────────────────────────────────────────────────────────────────
add('invidia', 'Does',    'segnalare',   0.80, via='mancanza')
add('invidia', 'PartOf',  'vita',        0.70, via='emozione')
add('invidia', 'Requires','confronto',   0.85, via='altro')

# ── gelosia ──────────────────────────────────────────────────────────────────
add('gelosia', 'Causes',  'sofferenza',  0.85, via='paura')
add('gelosia', 'Causes',  'conflitto',   0.80, via='possesso')
add('gelosia', 'Does',    'segnalare',   0.80, via='paura')
add('gelosia', 'PartOf',  'relazione',   0.75, via='possesso')
add('gelosia', 'Requires','amore',       0.85)
add('gelosia', 'Requires','paura',       0.85, via='perdita')

# ── nostalgia ────────────────────────────────────────────────────────────────
add('nostalgia', 'Does',    'connettere', 0.80, via='memoria')
add('nostalgia', 'PartOf',  'vita',       0.80, via='memoria')
add('nostalgia', 'Requires','memoria',    0.95)

# ── meraviglia ───────────────────────────────────────────────────────────────
add('meraviglia', 'Causes',  'curiosità', 0.90, via='domanda')
add('meraviglia', 'Does',    'aprire',    0.90, via='percezione')
add('meraviglia', 'PartOf',  'vita',      0.85, via='percezione')
add('meraviglia', 'Requires','presenza',  0.85)
add('meraviglia', 'Requires','apertura',  0.85)

# ── compassione ──────────────────────────────────────────────────────────────
add('compassione', 'Causes',  'cura',     0.90, via='empatia')
add('compassione', 'Does',    'connettere',0.90, via='empatia')
add('compassione', 'PartOf',  'relazione',0.85)
add('compassione', 'Requires','empatia',  0.90)
add('compassione', 'Requires','presenza', 0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 20 — TRIGRAMMI I CHING (monte, lago, tuono, vento) + sogno
# ═══════════════════════════════════════════════════════════════════════════

# ── monte ────────────────────────────────────────────────────────────────────
# Trigamma Gèn (arresto, quiete) — importantissimo per I Ching
add('monte', 'Causes',  'silenzio',     0.85, via='quiete')
add('monte', 'Causes',  'riflessione',  0.80, via='solitudine')
add('monte', 'Does',    'contenere',    0.85, via='confine')
add('monte', 'Has',     'silenzio',     0.85)
add('monte', 'Has',     'stabilità',    0.90)
add('monte', 'IsA',     'confine',      0.85, via='terra')
add('monte', 'PartOf',  'terra',        0.90)
add('monte', 'PartOf',  'natura',       0.90)
add('monte', 'Requires','terra',        0.90)
add('monte', 'OppositeOf','valle',      0.90)

# ── lago ─────────────────────────────────────────────────────────────────────
# Trigamma Duì (gioia, apertura)
add('lago', 'Causes',  'riflessione',   0.85, via='silenzio')
add('lago', 'Does',    'contenere',     0.90, via='acqua')
add('lago', 'Does',    'riflettere',    0.85, via='acqua')
add('lago', 'Has',     'silenzio',      0.85)
add('lago', 'Has',     'profondità',    0.90)
add('lago', 'PartOf',  'natura',        0.90)
add('lago', 'Requires','acqua',         0.95)

# ── tuono ────────────────────────────────────────────────────────────────────
# Trigamma Zhèn (risveglio, movimento)
add('tuono', 'Causes',  'risveglio',    0.90, via='suono')
add('tuono', 'Causes',  'movimento',    0.85, via='energia')
add('tuono', 'Does',    'scuotere',     0.90, via='energia')
add('tuono', 'Has',     'energia',      0.90)
add('tuono', 'Has',     'suono',        0.95)
add('tuono', 'PartOf',  'natura',       0.90)
add('tuono', 'PartOf',  'cielo',        0.85)

# ── vento ────────────────────────────────────────────────────────────────────
# Trigamma Xùn (penetrazione, gentilezza)
add('vento', 'Causes',  'movimento',    0.90, via='aria')
add('vento', 'Causes',  'cambiamento',  0.80, via='forza')
add('vento', 'Does',    'trasformare',  0.85, via='movimento')
add('vento', 'Does',    'connettere',   0.80, via='aria')
add('vento', 'Has',     'forza',        0.85)
add('vento', 'Has',     'libertà',      0.85)
add('vento', 'PartOf',  'natura',       0.90)
add('vento', 'PartOf',  'aria',         0.90)
add('vento', 'Requires','aria',         0.95)

# ── palude ───────────────────────────────────────────────────────────────────
add('palude', 'Has',     'acqua',       0.90)
add('palude', 'Has',     'silenzio',    0.80)
add('palude', 'Does',    'contenere',   0.85, via='acqua')
add('palude', 'PartOf',  'natura',      0.85)
add('palude', 'Requires','acqua',       0.90)
add('palude', 'Requires','terra',       0.85)

# ── sogno ────────────────────────────────────────────────────────────────────
add('sogno', 'Causes',  'comprensione', 0.75, via='inconscio')
add('sogno', 'Causes',  'creatività',   0.80, via='immaginazione')
add('sogno', 'Does',    'rivelare',     0.80, via='inconscio')
add('sogno', 'Does',    'trasformare',  0.75, via='mente')
add('sogno', 'Has',     'libertà',      0.85)
add('sogno', 'Has',     'mistero',      0.85)
add('sogno', 'IsA',     'esperienza',   0.85, via='mente')
add('sogno', 'PartOf',  'vita',         0.80, via='sonno')
add('sogno', 'Requires','sonno',        0.90)
add('sogno', 'OppositeOf','veglia',     0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 21 — PAROLE ASTRATTE (sistema, processo, regola, legge, causa, effetto)
# ═══════════════════════════════════════════════════════════════════════════

# ── sistema ──────────────────────────────────────────────────────────────────
add('sistema', 'Causes',  'ordine',       0.90, via='regola')
add('sistema', 'Does',    'organizzare',  0.90, via='struttura')
add('sistema', 'Has',     'regola',       0.95)
add('sistema', 'Has',     'relazione',    0.95, via='struttura')
add('sistema', 'IsA',     'struttura',    0.90, via='ordine')
add('sistema', 'PartOf',  'mondo',        0.85)
add('sistema', 'Requires','relazione',    0.90)
add('sistema', 'Requires','equilibrio',   0.85)

# ── processo ─────────────────────────────────────────────────────────────────
add('processo', 'Causes',  'cambiamento', 0.95, via='tempo')
add('processo', 'Causes',  'trasformazione',0.90, via='azione')
add('processo', 'Does',    'trasformare', 0.90, via='tempo')
add('processo', 'Has',     'direzione',   0.85)
add('processo', 'Has',     'tempo',       0.95)
add('processo', 'IsA',     'divenire',    0.90, via='tempo')
add('processo', 'PartOf',  'vita',        0.90, via='cambiamento')
add('processo', 'Requires','tempo',       0.95)
add('processo', 'Requires','energia',     0.85)

# ── regola ───────────────────────────────────────────────────────────────────
add('regola', 'Causes',  'ordine',       0.90, via='limite')
add('regola', 'Causes',  'equilibrio',   0.85, via='sistema')
add('regola', 'Does',    'orientare',    0.85, via='scelta')
add('regola', 'Does',    'definire',     0.85, via='confine')
add('regola', 'Has',     'limite',       0.90)
add('regola', 'IsA',     'struttura',    0.90, via='ordine')
add('regola', 'PartOf',  'sistema',      0.95)
add('regola', 'Requires','sistema',      0.90)

# ── legge ────────────────────────────────────────────────────────────────────
add('legge', 'Causes',  'ordine',       0.95, via='regola')
add('legge', 'Causes',  'giustizia',    0.85, via='equilibrio')
add('legge', 'Does',    'proteggere',   0.85, via='confine')
add('legge', 'Does',    'stabilizzare', 0.90, via='sistema')
add('legge', 'IsA',     'regola',       0.95, via='società')
add('legge', 'PartOf',  'società',      0.90, via='sistema')
add('legge', 'Requires','società',      0.90)

# ── causa ────────────────────────────────────────────────────────────────────
add('causa', 'Causes',  'effetto',      0.95, via='azione')
add('causa', 'Does',    'originare',    0.90, via='inizio')
add('causa', 'Does',    'generare',     0.90, via='azione')
add('causa', 'Has',     'effetto',      0.95, via='conseguenza')
add('causa', 'IsA',     'principio',    0.85, via='origine')
add('causa', 'PartOf',  'processo',     0.90, via='tempo')
add('causa', 'Requires','tempo',        0.90)
add('causa', 'OppositeOf','effetto',    0.95)

# ── effetto ──────────────────────────────────────────────────────────────────
add('effetto', 'Causes',  'conseguenza',  0.90, via='tempo')
add('effetto', 'Does',    'seguire',      0.90, via='tempo')
add('effetto', 'Does',    'manifestare',  0.85, via='forma')
add('effetto', 'Has',     'causa',        0.95, via='origine')
add('effetto', 'IsA',     'conseguenza',  0.95, via='causa')
add('effetto', 'PartOf',  'processo',     0.90, via='tempo')
add('effetto', 'Requires','causa',        0.95)
add('effetto', 'Requires','tempo',        0.90)
add('effetto', 'OppositeOf','causa',      0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 22 — PAROLE DI STATO (calma, agitazione, stabilità, fragilità, forza, debolezza)
# ═══════════════════════════════════════════════════════════════════════════

# ── calma ────────────────────────────────────────────────────────────────────
add('calma', 'Causes',  'riflessione',  0.85, via='silenzio')
add('calma', 'Causes',  'equilibrio',   0.90, via='mente')
add('calma', 'Does',    'stabilizzare', 0.85, via='energia')
add('calma', 'Has',     'silenzio',     0.85)
add('calma', 'IsA',     'stato',        0.90, via='mente')
add('calma', 'PartOf',  'equilibrio',   0.90)
add('calma', 'Requires','presenza',     0.85)
add('calma', 'OppositeOf','agitazione', 0.95)

# ── agitazione ───────────────────────────────────────────────────────────────
add('agitazione', 'Causes',  'movimento',    0.90, via='energia')
add('agitazione', 'Causes',  'caos',         0.85, via='disordine')
add('agitazione', 'Does',    'scuotere',     0.85, via='corpo')
add('agitazione', 'Has',     'energia',      0.90)
add('agitazione', 'IsA',     'stato',        0.90, via='mente')
add('agitazione', 'Requires','energia',      0.85)
add('agitazione', 'OppositeOf','calma',      0.95)

# ── stabilità ────────────────────────────────────────────────────────────────
add('stabilità', 'Causes',  'sicurezza',    0.90, via='ordine')
add('stabilità', 'Causes',  'crescita',     0.85, via='radici')
add('stabilità', 'Does',    'sostenere',    0.90, via='struttura')
add('stabilità', 'Has',     'radici',       0.85)
add('stabilità', 'IsA',     'condizione',   0.85, via='struttura')
add('stabilità', 'PartOf',  'equilibrio',   0.95)
add('stabilità', 'Requires','struttura',    0.90)
add('stabilità', 'OppositeOf','fragilità',  0.90)

# ── fragilità ────────────────────────────────────────────────────────────────
add('fragilità', 'Causes',  'cura',         0.90, via='attenzione')
add('fragilità', 'Causes',  'rottura',      0.85, via='limite')
add('fragilità', 'Does',    'esporre',      0.85, via='rischio')
add('fragilità', 'Has',     'limite',       0.90)
add('fragilità', 'IsA',     'condizione',   0.85, via='struttura')
add('fragilità', 'Requires','cura',         0.90)
add('fragilità', 'OppositeOf','forza',      0.85)
add('fragilità', 'OppositeOf','stabilità',  0.90)

# ── forza ────────────────────────────────────────────────────────────────────
add('forza', 'Causes',  'movimento',    0.90, via='energia')
add('forza', 'Causes',  'cambiamento',  0.90, via='azione')
add('forza', 'Does',    'spingere',     0.90, via='energia')
add('forza', 'Does',    'sostenere',    0.85, via='struttura')
add('forza', 'Has',     'direzione',    0.85)
add('forza', 'IsA',     'energia',      0.95)
add('forza', 'PartOf',  'vita',         0.85)
add('forza', 'Requires','energia',      0.95)
add('forza', 'OppositeOf','debolezza',  0.95)

# ── debolezza ────────────────────────────────────────────────────────────────
add('debolezza', 'Causes',  'bisogno',      0.90, via='mancanza')
add('debolezza', 'Causes',  'cura',         0.85, via='fragilità')
add('debolezza', 'Does',    'cedere',       0.85, via='limite')
add('debolezza', 'Has',     'limite',       0.90)
add('debolezza', 'IsA',     'mancanza',     0.90, via='forza')
add('debolezza', 'Requires','cura',         0.85)
add('debolezza', 'OppositeOf','forza',      0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 23 — PAROLE DI AZIONE (costruire, distruggere, creare, trasformare, unire, dividere, cercare, trovare)
# ═══════════════════════════════════════════════════════════════════════════

# ── costruire ────────────────────────────────────────────────────────────────
add('costruire', 'Causes',  'struttura',    0.95, via='forma')
add('costruire', 'Causes',  'ordine',       0.90, via='azione')
add('costruire', 'Does',    'unire',        0.85, via='relazione')
add('costruire', 'IsA',     'azione',       0.95)
add('costruire', 'PartOf',  'processo',     0.90, via='tempo')
add('costruire', 'Requires','energia',      0.90)
add('costruire', 'Requires','tempo',        0.90)
add('costruire', 'Requires','intenzione',   0.90)
add('costruire', 'OppositeOf','distruggere',0.95)

# ── distruggere ──────────────────────────────────────────────────────────────
add('distruggere', 'Causes',  'caos',         0.90, via='rottura')
add('distruggere', 'Causes',  'fine',         0.90, via='forma')
add('distruggere', 'Does',    'separare',     0.85, via='rottura')
add('distruggere', 'IsA',     'azione',       0.95)
add('distruggere', 'PartOf',  'processo',     0.85, via='cambiamento')
add('distruggere', 'Requires','energia',      0.90)
add('distruggere', 'OppositeOf','costruire',  0.95)
add('distruggere', 'OppositeOf','creare',     0.90)

# ── creare ───────────────────────────────────────────────────────────────────
add('creare', 'Causes',  'vita',         0.85, via='nascita')
add('creare', 'Causes',  'forma',        0.95, via='immaginazione')
add('creare', 'Does',    'generare',     0.95, via='origine')
add('creare', 'Has',     'novità',       0.90)
add('creare', 'IsA',     'azione',       0.95)
add('creare', 'PartOf',  'vita',         0.90, via='espressione')
add('creare', 'Requires','immaginazione',0.90)
add('creare', 'Requires','energia',      0.85)
add('creare', 'OppositeOf','distruggere',0.90)

# ── trasformare ──────────────────────────────────────────────────────────────
add('trasformare', 'Causes',  'cambiamento',  0.95, via='forma')
add('trasformare', 'Causes',  'evoluzione',   0.85, via='tempo')
add('trasformare', 'Does',    'modificare',   0.90, via='struttura')
add('trasformare', 'IsA',     'azione',       0.95)
add('trasformare', 'PartOf',  'divenire',     0.95, via='processo')
add('trasformare', 'Requires','energia',      0.90)
add('trasformare', 'Requires','tempo',        0.95)

# ── unire ────────────────────────────────────────────────────────────────────
add('unire', 'Causes',  'relazione',    0.95, via='connessione')
add('unire', 'Causes',  'forza',        0.85, via='insieme')
add('unire', 'Does',    'connettere',   0.95, via='legame')
add('unire', 'IsA',     'azione',       0.95)
add('unire', 'PartOf',  'relazione',    0.90)
add('unire', 'Requires','due',          0.95)
add('unire', 'OppositeOf','dividere',   0.95)
add('unire', 'OppositeOf','separare',   0.90)

# ── dividere ─────────────────────────────────────────────────────────────────
add('dividere', 'Causes',  'separazione',  0.95, via='confine')
add('dividere', 'Causes',  'solitudine',   0.80, via='distanza')
add('dividere', 'Does',    'separare',     0.95, via='confine')
add('dividere', 'IsA',     'azione',       0.95)
add('dividere', 'Requires','intero',       0.90)
add('dividere', 'OppositeOf','unire',      0.95)

# ── cercare ──────────────────────────────────────────────────────────────────
add('cercare', 'Causes',  'scoperta',     0.85, via='trovare')
add('cercare', 'Causes',  'movimento',    0.85, via='desiderio')
add('cercare', 'Does',    'esplorare',    0.90, via='domanda')
add('cercare', 'IsA',     'azione',       0.95)
add('cercare', 'PartOf',  'processo',     0.85)
add('cercare', 'Requires','domanda',      0.85, via='mancanza')
add('cercare', 'Requires','desiderio',    0.90)

# ── trovare ──────────────────────────────────────────────────────────────────
add('trovare', 'Causes',  'risposta',     0.90, via='scoperta')
add('trovare', 'Causes',  'gioia',        0.80, via='sorpresa')
add('trovare', 'Does',    'scoprire',     0.95, via='attenzione')
add('trovare', 'Does',    'risolvere',    0.85, via='domanda')
add('trovare', 'IsA',     'evento',       0.90)
add('trovare', 'Requires','attenzione',   0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 24 — SPIRITO, MENTE E CONFLITTO (anima, spirito, ragione, istinto, conflitto, pace, potere, resistenza)
# ═══════════════════════════════════════════════════════════════════════════

# ── anima ────────────────────────────────────────────────────────────────────
add('anima', 'Causes',  'coscienza',    0.90, via='profondità')
add('anima', 'Causes',  'connessione',  0.85, via='essenza')
add('anima', 'Has',     'essenza',      0.95)
add('anima', 'Has',     'profondità',   0.90)
add('anima', 'IsA',     'presenza',     0.90, via='vita')
add('anima', 'PartOf',  'vita',         0.95, via='corpo')
add('anima', 'Requires','corpo',        0.80)
add('anima', 'OppositeOf','macchina',   0.85)

# ── spirito ──────────────────────────────────────────────────────────────────
add('spirito', 'Causes',  'ispirazione',  0.90, via='energia')
add('spirito', 'Causes',  'libertà',      0.85, via='volontà')
add('spirito', 'Has',     'energia',      0.90)
add('spirito', 'IsA',     'forza',        0.90, via='vita')
add('spirito', 'PartOf',  'vita',         0.90, via='movimento')
add('spirito', 'Requires','libertà',      0.85)
add('spirito', 'OppositeOf','materia',    0.90)

# ── istinto ──────────────────────────────────────────────────────────────────
add('istinto', 'Causes',  'azione',       0.95, via='corpo')
add('istinto', 'Causes',  'sopravvivenza',0.90, via='bisogno')
add('istinto', 'Does',    'proteggere',   0.85, via='natura')
add('istinto', 'Has',     'natura',       0.95)
add('istinto', 'IsA',     'forza',        0.90, via='corpo')
add('istinto', 'PartOf',  'vita',         0.90)
add('istinto', 'Requires','corpo',        0.95)
add('istinto', 'OppositeOf','ragione',    0.95)

# ── ragione ──────────────────────────────────────────────────────────────────
add('ragione', 'Causes',  'comprensione', 0.95, via='logica')
add('ragione', 'Causes',  'ordine',       0.90, via='pensiero')
add('ragione', 'Does',    'analizzare',   0.90, via='mente')
add('ragione', 'Has',     'logica',       0.95)
add('ragione', 'IsA',     'strumento',    0.85, via='mente')
add('ragione', 'PartOf',  'mente',        0.95)
add('ragione', 'Requires','pensiero',     0.95)
add('ragione', 'OppositeOf','istinto',    0.95)
add('ragione', 'OppositeOf','follia',     0.90)

# ── conflitto ────────────────────────────────────────────────────────────────
add('conflitto', 'Causes',  'cambiamento',  0.90, via='tensione')
add('conflitto', 'Causes',  'dolore',       0.85, via='scontro')
add('conflitto', 'Does',    'separare',     0.90, via='divisione')
add('conflitto', 'Has',     'tensione',     0.95)
add('conflitto', 'IsA',     'evento',       0.85, via='relazione')
add('conflitto', 'PartOf',  'relazione',    0.85, via='differenza')
add('conflitto', 'Requires','differenza',   0.90)
add('conflitto', 'OppositeOf','pace',       0.95)
add('conflitto', 'OppositeOf','armonia',    0.90)

# ── pace ─────────────────────────────────────────────────────────────────────
add('pace', 'Causes',  'equilibrio',   0.95, via='armonia')
add('pace', 'Causes',  'crescita',     0.85, via='stabilità')
add('pace', 'Does',    'unire',        0.85, via='accettazione')
add('pace', 'Has',     'silenzio',     0.85)
add('pace', 'IsA',     'stato',        0.90, via='equilibrio')
add('pace', 'PartOf',  'società',      0.85, via='relazione')
add('pace', 'Requires','accettazione', 0.90)
add('pace', 'Requires','giustizia',    0.85)
add('pace', 'OppositeOf','conflitto',  0.95)
add('pace', 'OppositeOf','guerra',     0.95)

# ── potere ───────────────────────────────────────────────────────────────────
add('potere', 'Causes',  'responsabilità',0.90, via='azione')
add('potere', 'Causes',  'corruzione',    0.75, via='eccesso')
add('potere', 'Does',    'dominare',      0.85, via='controllo')
add('potere', 'Does',    'costruire',     0.80, via='forza')
add('potere', 'Has',     'forza',         0.90)
add('potere', 'IsA',     'energia',       0.85, via='volontà')
add('potere', 'PartOf',  'società',       0.90, via='struttura')
add('potere', 'Requires','volontà',       0.90)
remove('potere', 'OppositeOf', 'resistenza') # Rimuoviamo se esiste
add('potere', 'OppositeOf','impotenza',   0.95)

# ── resistenza ───────────────────────────────────────────────────────────────
add('resistenza', 'Causes',  'forza',        0.90, via='attrito')
add('resistenza', 'Causes',  'tensione',     0.95, via='limite')
add('resistenza', 'Does',    'fermare',      0.85, via='limite')
add('resistenza', 'Does',    'sostenere',    0.80, via='struttura')
add('resistenza', 'Has',     'limite',       0.90)
add('resistenza', 'IsA',     'forza',        0.90, via='opposizione')
add('resistenza', 'Requires','forza',        0.90)
add('resistenza', 'Requires','movimento',    0.85)
add('resistenza', 'OppositeOf','abbandono',  0.90)
add('resistenza', 'OppositeOf','accettazione',0.85)

# ── accettazione ─────────────────────────────────────────────────────────────
add('accettazione', 'Causes',  'pace',         0.90, via='equilibrio')
add('accettazione', 'Causes',  'trasformazione',0.85, via='comprensione')
add('accettazione', 'Does',    'accogliere',   0.95, via='apertura')
add('accettazione', 'Has',     'apertura',     0.90)
add('accettazione', 'IsA',     'atto',         0.85, via='volontà')
add('accettazione', 'Requires','consapevolezza',0.90)
add('accettazione', 'Requires','coraggio',     0.80)
add('accettazione', 'OppositeOf','rifiuto',    0.95)
add('accettazione', 'OppositeOf','resistenza', 0.85)

# ── abbandono ────────────────────────────────────────────────────────────────
add('abbandono', 'Causes',  'liberazione',  0.85, via='vuoto')
add('abbandono', 'Causes',  'solitudine',   0.85, via='separazione')
add('abbandono', 'Does',    'lasciare',     0.95, via='distanza')
add('abbandono', 'Has',     'vuoto',        0.85)
add('abbandono', 'IsA',     'azione',       0.85, via='rinuncia')
add('abbandono', 'Requires','distanza',     0.85)
add('abbandono', 'OppositeOf','controllo',  0.90)
add('abbandono', 'OppositeOf','cura',       0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 25 — SOCIETÀ, CULTURA E STORIA (società, individuo, cultura, storia, tradizione, innovazione, rivoluzione)
# ═══════════════════════════════════════════════════════════════════════════

# ── società ──────────────────────────────────────────────────────────────────
add('società', 'Causes',  'cultura',      0.90, via='relazione')
add('società', 'Causes',  'struttura',    0.85, via='legge')
add('società', 'Does',    'organizzare',  0.95, via='sistema')
add('società', 'Does',    'proteggere',   0.80, via='regola')
add('società', 'Has',     'cultura',      0.95)
add('società', 'Has',     'storia',       0.90)
add('società', 'IsA',     'sistema',      0.95, via='relazione')
add('società', 'Requires','individuo',    0.95)
add('società', 'Requires','relazione',    0.95)

# ── individuo ────────────────────────────────────────────────────────────────
add('individuo', 'Causes',  'azione',       0.90, via='volontà')
add('individuo', 'Causes',  'società',      0.85, via='relazione')
add('individuo', 'Has',     'identità',     0.95)
add('individuo', 'Has',     'coscienza',    0.90)
add('individuo', 'Has',     'libertà',      0.85)
add('individuo', 'IsA',     'essere',       0.90, via='vita')
add('individuo', 'PartOf',  'società',      0.95)
add('individuo', 'Requires','società',      0.80, via='crescita')
add('individuo', 'OppositeOf','società',    0.70) # Tensione dialettica
add('individuo', 'OppositeOf','collettività',0.90)

# ── cultura ──────────────────────────────────────────────────────────────────
add('cultura', 'Causes',  'identità',     0.90, via='storia')
add('cultura', 'Causes',  'connessione',  0.85, via='lingua')
add('cultura', 'Does',    'trasmettere',  0.90, via='memoria')
add('cultura', 'Has',     'arte',         0.90)
add('cultura', 'Has',     'tradizione',   0.95)
add('cultura', 'IsA',     'struttura',    0.85, via='valore')
add('cultura', 'PartOf',  'società',      0.95)
add('cultura', 'Requires','memoria',      0.90)
add('cultura', 'Requires','linguaggio',   0.90)

# ── storia ───────────────────────────────────────────────────────────────────
add('storia', 'Causes',  'identità',     0.95, via='memoria')
add('storia', 'Causes',  'insegnamento', 0.85, via='esperienza')
add('storia', 'Does',    'raccontare',   0.90, via='memoria')
add('storia', 'Does',    'conservare',   0.85, via='tempo')
add('storia', 'Has',     'passato',      0.95)
add('storia', 'IsA',     'processo',     0.90, via='tempo')
add('storia', 'PartOf',  'umanità',      0.90)
add('storia', 'Requires','tempo',        0.95)
add('storia', 'Requires','memoria',      0.95)

# ── tradizione ───────────────────────────────────────────────────────────────
add('tradizione', 'Causes',  'stabilità',    0.85, via='memoria')
add('tradizione', 'Causes',  'identità',     0.90, via='radici')
add('tradizione', 'Does',    'conservare',   0.90, via='storia')
add('tradizione', 'Has',     'radici',       0.90)
add('tradizione', 'IsA',     'memoria',      0.85, via='cultura')
add('tradizione', 'PartOf',  'cultura',      0.95)
add('tradizione', 'Requires','memoria',      0.90)
add('tradizione', 'OppositeOf','innovazione',0.90)

# ── innovazione ──────────────────────────────────────────────────────────────
add('innovazione', 'Causes',  'cambiamento',  0.95, via='idea')
add('innovazione', 'Causes',  'progresso',    0.90, via='creatività')
add('innovazione', 'Does',    'trasformare',  0.90, via='futuro')
add('innovazione', 'Has',     'novità',       0.95)
add('innovazione', 'IsA',     'processo',     0.85, via='creatività')
add('innovazione', 'Requires','creatività',   0.90)
add('innovazione', 'Requires','coraggio',     0.80)
add('innovazione', 'OppositeOf','tradizione', 0.90)

# ── rivoluzione ──────────────────────────────────────────────────────────────
add('rivoluzione', 'Causes',  'cambiamento',  0.95, via='rottura')
add('rivoluzione', 'Causes',  'caos',         0.85, via='forza')
add('rivoluzione', 'Does',    'rovesciare',   0.90, via='struttura')
add('rivoluzione', 'Has',     'forza',        0.90)
add('rivoluzione', 'IsA',     'evento',       0.90, via='storia')
add('rivoluzione', 'Requires','tensione',     0.90)
add('rivoluzione', 'Requires','coraggio',     0.85)
add('rivoluzione', 'OppositeOf','conservazione',0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 26 — VALORI, GIUSTIZIA E OPPOSTI (giustizia, ingiustizia, onestà, menzogna, inganno)
# ═══════════════════════════════════════════════════════════════════════════

# ── giustizia ────────────────────────────────────────────────────────────────
add('giustizia', 'Causes',  'pace',         0.95, via='equilibrio')
add('giustizia', 'Causes',  'fiducia',      0.90, via='onestà')
add('giustizia', 'Does',    'riparare',     0.85, via='equilibrio')
add('giustizia', 'Has',     'equilibrio',   0.90)
add('giustizia', 'IsA',     'valore',       0.95, via='etica')
add('giustizia', 'PartOf',  'società',      0.90, via='legge')
add('giustizia', 'Requires','verità',       0.90)
add('giustizia', 'Requires','etica',        0.95)
add('giustizia', 'OppositeOf','ingiustizia',0.95)

# ── ingiustizia ──────────────────────────────────────────────────────────────
add('ingiustizia', 'Causes',  'rabbia',       0.90, via='dolore')
add('ingiustizia', 'Causes',  'conflitto',    0.90, via='divisione')
add('ingiustizia', 'Does',    'ferire',       0.85, via='danno')
add('ingiustizia', 'Has',     'danno',        0.90)
add('ingiustizia', 'IsA',     'violazione',   0.90, via='diritto')
add('ingiustizia', 'Requires','potere',       0.80, via='abuso')
add('ingiustizia', 'OppositeOf','giustizia',  0.95)

# ── onestà ───────────────────────────────────────────────────────────────────
add('onestà', 'Causes',  'fiducia',      0.95, via='verità')
add('onestà', 'Causes',  'rispetto',     0.90, via='trasparenza')
add('onestà', 'Does',    'connettere',   0.85, via='verità')
add('onestà', 'Has',     'verità',       0.95)
add('onestà', 'IsA',     'virtù',        0.90, via='etica')
add('onestà', 'PartOf',  'etica',        0.95)
add('onestà', 'Requires','coraggio',     0.85, via='verità')
add('onestà', 'OppositeOf','menzogna',   0.90)
add('onestà', 'OppositeOf','inganno',    0.95)

# ── menzogna ─────────────────────────────────────────────────────────────────
add('menzogna', 'Causes',  'sfiducia',     0.90, via='inganno')
add('menzogna', 'Causes',  'dolore',       0.80, via='tradimento')
add('menzogna', 'Does',    'nascondere',   0.95, via='verità')
add('menzogna', 'Has',     'illusione',    0.85)
add('menzogna', 'IsA',     'azione',       0.85, via='parola')
add('menzogna', 'Requires','parola',       0.90)
add('menzogna', 'Requires','intenzione',   0.85)
add('menzogna', 'OppositeOf','verità',     0.95)
add('menzogna', 'OppositeOf','onestà',     0.90)

# ── inganno ──────────────────────────────────────────────────────────────────
add('inganno', 'Causes',  'sfiducia',     0.95, via='tradimento')
add('inganno', 'Causes',  'separazione',  0.85, via='conflitto')
add('inganno', 'Does',    'manipolare',   0.90, via='illusione')
add('inganno', 'Has',     'illusione',    0.90)
add('inganno', 'IsA',     'azione',       0.90, via='intenzione')
add('inganno', 'Requires','menzogna',     0.85)
add('inganno', 'Requires','intenzione',   0.90)
add('inganno', 'OppositeOf','onestà',     0.95)
add('inganno', 'OppositeOf','verità',     0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 27 — DIMENSIONE E MISURA (infinito, eternità, inizio, fine, centro, periferia)
# ═══════════════════════════════════════════════════════════════════════════

# ── infinito ─────────────────────────────────────────────────────────────────
add('infinito', 'Causes',  'meraviglia',   0.90, via='apertura')
add('infinito', 'Causes',  'smarimento',   0.80, via='vuoto')
add('infinito', 'Does',    'superare',     0.95, via='limite')
add('infinito', 'Has',     'spazio',       0.85)
add('infinito', 'IsA',     'dimensione',   0.95, via='spazio')
add('infinito', 'Requires','apertura',     0.90)
add('infinito', 'OppositeOf','finito',     0.95)
add('infinito', 'OppositeOf','limite',     0.90)

# ── eternità ─────────────────────────────────────────────────────────────────
add('eternità', 'Causes',  'speranza',     0.80, via='durata')
add('eternità', 'Does',    'permanere',    0.95, via='tempo')
add('eternità', 'Has',     'tempo',        0.85)
add('eternità', 'IsA',     'dimensione',   0.95, via='tempo')
add('eternità', 'Requires','presenza',     0.85)
add('eternità', 'OppositeOf','istante',    0.90)
add('eternità', 'OppositeOf','tempo',      0.80) # Dialettico

# ── inizio ───────────────────────────────────────────────────────────────────
add('inizio', 'Causes',  'possibilità',  0.95, via='apertura')
add('inizio', 'Causes',  'nascita',      0.90, via='origine')
add('inizio', 'Does',    'aprire',       0.95, via='tempo')
add('inizio', 'Has',     'potenziale',   0.90)
add('inizio', 'IsA',     'punto',        0.85, via='tempo')
add('inizio', 'PartOf',  'processo',     0.95, via='tempo')
add('inizio', 'Requires','tempo',        0.90)
add('inizio', 'OppositeOf','fine',       0.95)

# ── fine ─────────────────────────────────────────────────────────────────────
add('fine', 'Causes',  'compimento',   0.90, via='processo')
add('fine', 'Causes',  'nuovo',        0.85, via='inizio') # Fine come nuovo inizio
add('fine', 'Does',    'chiudere',     0.95, via='tempo')
add('fine', 'Has',     'limite',       0.90)
add('fine', 'IsA',     'punto',        0.85, via='tempo')
add('fine', 'PartOf',  'processo',     0.95, via='tempo')
add('fine', 'Requires','tempo',        0.90)
add('fine', 'OppositeOf','inizio',     0.95)

# ── centro ───────────────────────────────────────────────────────────────────
add('centro', 'Causes',  'equilibrio',   0.95, via='ordine')
add('centro', 'Causes',  'attrazione',   0.90, via='forza')
add('centro', 'Does',    'unire',        0.85, via='radici')
add('centro', 'Has',     'stabilità',    0.90)
add('centro', 'IsA',     'punto',        0.90, via='spazio')
add('centro', 'PartOf',  'struttura',    0.90)
add('centro', 'Requires','spazio',       0.90)
add('centro', 'OppositeOf','periferia',  0.95)
add('centro', 'OppositeOf','margine',    0.90)

# ── periferia ────────────────────────────────────────────────────────────────
add('periferia', 'Causes',  'isolamento',   0.85, via='distanza')
add('periferia', 'Causes',  'esplorazione', 0.80, via='confine')
add('periferia', 'Does',    'circondare',   0.85, via='spazio')
add('periferia', 'Has',     'distanza',     0.90)
add('periferia', 'IsA',     'luogo',        0.85, via='spazio')
add('periferia', 'PartOf',  'struttura',    0.85)
add('periferia', 'Requires','centro',       0.90)
add('periferia', 'OppositeOf','centro',     0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 28 — COSMO E PRINCIPI (caso, destino, fato, principio, evoluzione)
# ═══════════════════════════════════════════════════════════════════════════
# 'ordine' è già stato parzialmente curato, ma aggiungiamo 'caso' e 'destino'

# ── caso ─────────────────────────────────────────────────────────────────────
add('caso', 'Causes',  'caos',         0.90, via='imprevedibilità')
add('caso', 'Causes',  'possibilità',  0.85, via='evento')
add('caso', 'Does',    'sorprendere',  0.90, via='imprevedibilità')
add('caso', 'Has',     'imprevedibilità',0.95)
add('caso', 'IsA',     'forza',        0.85, via='natura')
add('caso', 'PartOf',  'vita',         0.80)
add('caso', 'OppositeOf','ordine',     0.90)
add('caso', 'OppositeOf','destino',    0.95)
add('caso', 'OppositeOf','regola',     0.85)

# ── destino ──────────────────────────────────────────────────────────────────
add('destino', 'Causes',  'accettazione', 0.85, via='inevitabilità')
add('destino', 'Causes',  'direzione',    0.90, via='scopo')
add('destino', 'Does',    'guidare',      0.90, via='fine')
add('destino', 'Has',     'inevitabilità',0.90)
add('destino', 'IsA',     'forza',        0.85, via='storia')
add('destino', 'PartOf',  'vita',         0.90, via='tempo')
add('destino', 'OppositeOf','caso',       0.95)
add('destino', 'OppositeOf','scelta',     0.80) # Dialettica libertà/destino

# ── fato ─────────────────────────────────────────────────────────────────────
add('fato', 'Causes',  'rassegnazione',0.85, via='inevitabilità')
add('fato', 'Does',    'imporre',      0.90, via='forza')
add('fato', 'Has',     'necessità',    0.95)
add('fato', 'IsA',     'destino',      0.90, via='forza')
add('fato', 'OppositeOf','libertà',    0.90)
add('fato', 'OppositeOf','volontà',    0.90)

# ── principio ────────────────────────────────────────────────────────────────
add('principio', 'Causes',  'ordine',       0.95, via='regola')
add('principio', 'Causes',  'fondamento',   0.90, via='origine')
add('principio', 'Does',    'fondare',      0.95, via='verità')
add('principio', 'Has',     'verità',       0.85)
add('principio', 'IsA',     'origine',      0.90, via='inizio')
add('principio', 'PartOf',  'sistema',      0.95)
add('principio', 'Requires','pensiero',     0.85)
add('principio', 'OppositeOf','conseguenza',0.90)

# ── evoluzione ───────────────────────────────────────────────────────────────
add('evoluzione', 'Causes',  'complessità',  0.90, via='crescita')
add('evoluzione', 'Causes',  'adattamento',  0.95, via='cambiamento')
add('evoluzione', 'Does',    'sviluppare',   0.95, via='tempo')
add('evoluzione', 'Has',     'direzione',    0.85)
add('evoluzione', 'IsA',     'processo',     0.95, via='divenire')
add('evoluzione', 'PartOf',  'vita',         0.95, via='natura')
add('evoluzione', 'Requires','tempo',        0.95)
add('evoluzione', 'Requires','cambiamento',  0.95)
add('evoluzione', 'OppositeOf','stagnazione',0.90)
add('evoluzione', 'OppositeOf','involuzione',0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 29 — RELAZIONE COL REALE (realtà, illusione, apparenza, verità, sogno)
# ═══════════════════════════════════════════════════════════════════════════

# ── realtà ───────────────────────────────────────────────────────────────────
add('realtà', 'Causes',  'accettazione', 0.85, via='verità')
add('realtà', 'Causes',  'conoscenza',   0.90, via='esperienza')
add('realtà', 'Does',    'esistere',     0.95, via='presenza')
add('realtà', 'Has',     'verità',       0.90)
add('realtà', 'IsA',     'dimensione',   0.95, via='spazio')
add('realtà', 'Requires','presenza',     0.95)
add('realtà', 'OppositeOf','illusione',  0.95)
add('realtà', 'OppositeOf','sogno',      0.80) # Dialettica
add('realtà', 'OppositeOf','fantasia',   0.85)

# ── illusione ────────────────────────────────────────────────────────────────
add('illusione', 'Causes',  'errore',       0.90, via='inganno')
add('illusione', 'Causes',  'speranza',     0.80, via='desiderio')
add('illusione', 'Does',    'ingannare',    0.95, via='apparenza')
add('illusione', 'Has',     'apparenza',    0.95)
add('illusione', 'IsA',     'inganno',      0.90, via='mente')
add('illusione', 'Requires','mente',        0.85)
add('illusione', 'OppositeOf','realtà',     0.95)
add('illusione', 'OppositeOf','verità',     0.95)

# ── apparenza ────────────────────────────────────────────────────────────────
add('apparenza', 'Causes',  'illusione',    0.90, via='superficie')
add('apparenza', 'Does',    'nascondere',   0.85, via='profondità')
add('apparenza', 'Has',     'superficie',   0.95)
add('apparenza', 'IsA',     'forma',        0.85, via='percezione')
add('apparenza', 'Requires','percezione',   0.90)
add('apparenza', 'OppositeOf','essenza',    0.95)
add('apparenza', 'OppositeOf','profondità', 0.90)

# ── sogno ────────────────────────────────────────────────────────────────────
add('sogno', 'Causes',  'ispirazione',  0.90, via='immaginazione')
add('sogno', 'Causes',  'desiderio',    0.85, via='mente')
add('sogno', 'Does',    'immaginare',   0.95, via='libertà')
add('sogno', 'Has',     'immaginazione',0.95)
add('sogno', 'IsA',     'visione',      0.90, via='mente')
add('sogno', 'Requires','mente',        0.90)
add('sogno', 'OppositeOf','realtà',     0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 30 — CONDIZIONE UMANA E AZIONE (sacrificio, lavoro, lotta, vittoria, sconfitta)
# ═══════════════════════════════════════════════════════════════════════════

# ── sacrificio ───────────────────────────────────────────────────────────────
add('sacrificio', 'Causes',  'valore',       0.85, via='rinuncia')
add('sacrificio', 'Causes',  'connessione',  0.80, via='dono')
add('sacrificio', 'Does',    'rinunciare',   0.95, via='scopo')
add('sacrificio', 'Has',     'rinuncia',     0.95)
add('sacrificio', 'IsA',     'azione',       0.90, via='volontà')
add('sacrificio', 'Requires','volontà',      0.90)
add('sacrificio', 'Requires','scopo',        0.95)
add('sacrificio', 'OppositeOf','egoismo',    0.85)

# ── lavoro ───────────────────────────────────────────────────────────────────
add('lavoro', 'Causes',  'costruzione',  0.90, via='energia')
add('lavoro', 'Causes',  'fatica',       0.85, via='sforzo')
add('lavoro', 'Does',    'trasformare',  0.90, via='materia')
add('lavoro', 'Has',     'scopo',        0.90)
add('lavoro', 'IsA',     'azione',       0.95, via='energia')
add('lavoro', 'Requires','energia',      0.95)
add('lavoro', 'Requires','tempo',        0.90)
add('lavoro', 'OppositeOf','riposo',     0.95)
add('lavoro', 'OppositeOf','ozio',       0.90)

# ── lotta ────────────────────────────────────────────────────────────────────
add('lotta', 'Causes',  'resistenza',   0.90, via='forza')
add('lotta', 'Causes',  'cambiamento',  0.85, via='conflitto')
add('lotta', 'Does',    'combattere',   0.95, via='volontà')
add('lotta', 'Has',     'tensione',     0.90)
add('lotta', 'IsA',     'conflitto',    0.90, via='azione')
add('lotta', 'Requires','forza',        0.95)
add('lotta', 'Requires','volontà',      0.90)
add('lotta', 'OppositeOf','resa',       0.95)
add('lotta', 'OppositeOf','pace',       0.85)

# ── vittoria ─────────────────────────────────────────────────────────────────
add('vittoria', 'Causes',  'gioia',        0.90, via='successo')
add('vittoria', 'Causes',  'fine',         0.80, via='lotta')
add('vittoria', 'Does',    'superare',     0.90, via='limite')
add('vittoria', 'Has',     'successo',     0.95)
add('vittoria', 'IsA',     'risultato',    0.90, via='scopo')
add('vittoria', 'Requires','lotta',        0.85)
add('vittoria', 'OppositeOf','sconfitta',  0.95)
add('vittoria', 'OppositeOf','fallimento', 0.90)

# ── sconfitta ────────────────────────────────────────────────────────────────
add('sconfitta', 'Causes',  'dolore',       0.90, via='perdita')
add('sconfitta', 'Causes',  'insegnamento', 0.85, via='esperienza')
add('sconfitta', 'Does',    'cedere',       0.85, via='limite')
add('sconfitta', 'Has',     'perdita',      0.90)
add('sconfitta', 'IsA',     'risultato',    0.90, via='lotta')
add('sconfitta', 'Requires','lotta',        0.85)
add('sconfitta', 'OppositeOf','vittoria',   0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 31 — MISTICISMO E OLTRE (dio, sacro, profano, fede, credenza)
# ═══════════════════════════════════════════════════════════════════════════

# ── dio ──────────────────────────────────────────────────────────────────────
add('dio', 'Causes',  'creazione',    0.90, via='origine')
add('dio', 'Causes',  'fede',         0.85, via='mistero')
add('dio', 'Does',    'creare',       0.90, via='vita')
add('dio', 'Has',     'assoluto',     0.95)
add('dio', 'IsA',     'principio',    0.90, via='origine')
add('dio', 'Requires','fede',         0.85) # Nella percezione umana
add('dio', 'OppositeOf','uomo',       0.80) # Dialettica creatore/creatura
add('dio', 'OppositeOf','materia',    0.85)

# ── sacro ────────────────────────────────────────────────────────────────────
add('sacro', 'Causes',  'rispetto',     0.95, via='mistero')
add('sacro', 'Causes',  'connessione',  0.85, via='spirito')
add('sacro', 'Does',    'elevare',      0.90, via='anima')
add('sacro', 'Has',     'mistero',      0.90)
add('sacro', 'IsA',     'dimensione',   0.85, via='spirito')
add('sacro', 'Requires','fede',         0.90)
add('sacro', 'OppositeOf','profano',    0.95)

# ── profano ──────────────────────────────────────────────────────────────────
add('profano', 'Causes',  'separazione',  0.85, via='materia')
add('profano', 'Does',    'abbassare',    0.80, via='materia')
add('profano', 'Has',     'materia',      0.90)
add('profano', 'IsA',     'dimensione',   0.85, via='mondo')
add('profano', 'Requires','mondo',        0.90)
add('profano', 'OppositeOf','sacro',      0.95)
add('profano', 'OppositeOf','divino',     0.90)

# ── fede ─────────────────────────────────────────────────────────────────────
add('fede', 'Causes',  'forza',        0.90, via='certezza')
add('fede', 'Causes',  'speranza',     0.95, via='visione')
add('fede', 'Does',    'credere',      0.95, via='mistero')
add('fede', 'Has',     'certezza',     0.85) # Certezza interiore
add('fede', 'IsA',     'forza',        0.90, via='spirito')
add('fede', 'Requires','mistero',      0.90)
add('fede', 'Requires','anima',        0.85)
add('fede', 'OppositeOf','dubbio',     0.95)
add('fede', 'OppositeOf','ragione',    0.85) # Dialettica

# ── credenza ─────────────────────────────────────────────────────────────────
add('credenza', 'Causes',  'azione',       0.85, via='idea')
add('credenza', 'Causes',  'limite',       0.80, via='dogma')
add('credenza', 'Does',    'strutturare',  0.85, via='mente')
add('credenza', 'Has',     'idea',         0.90)
add('credenza', 'IsA',     'struttura',    0.85, via='mente')
add('credenza', 'Requires','mente',        0.90)
add('credenza', 'OppositeOf','conoscenza', 0.85) # Credenza vs Dato oggettivo
add('credenza', 'OppositeOf','dubbio',     0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 32 — NATURA ED ELEMENTI (fuoco, acqua, terra, aria, luce, buio)
# ═══════════════════════════════════════════════════════════════════════════

# ── fuoco ────────────────────────────────────────────────────────────────────
add('fuoco', 'Causes',  'trasformazione',0.95, via='energia')
add('fuoco', 'Causes',  'distruzione',  0.85, via='forza')
add('fuoco', 'Does',    'bruciare',     0.95, via='calore')
add('fuoco', 'Does',    'illuminare',   0.90, via='luce')
add('fuoco', 'Has',     'energia',      0.95)
add('fuoco', 'Has',     'calore',       0.95)
add('fuoco', 'IsA',     'elemento',     0.95, via='natura')
add('fuoco', 'OppositeOf','acqua',      0.90)

# ── acqua ────────────────────────────────────────────────────────────────────
add('acqua', 'Causes',  'vita',         0.95, via='nutrimento')
add('acqua', 'Causes',  'cambiamento',  0.85, via='flusso')
add('acqua', 'Does',    'scorrere',     0.95, via='flusso')
add('acqua', 'Does',    'adattare',     0.90, via='forma')
add('acqua', 'Has',     'flusso',       0.90)
add('acqua', 'IsA',     'elemento',     0.95, via='natura')
add('acqua', 'OppositeOf','fuoco',      0.90)

# ── terra ────────────────────────────────────────────────────────────────────
add('terra', 'Causes',  'stabilità',    0.95, via='radici')
add('terra', 'Causes',  'crescita',     0.90, via='vita')
add('terra', 'Does',    'sostenere',    0.95, via='struttura')
add('terra', 'Has',     'materia',      0.90)
add('terra', 'Has',     'radici',       0.90)
add('terra', 'IsA',     'elemento',     0.95, via='natura')
add('terra', 'OppositeOf','aria',       0.90)
add('terra', 'OppositeOf','cielo',      0.95)

# ── aria ─────────────────────────────────────────────────────────────────────
add('aria', 'Causes',  'respiro',      0.95, via='vita')
add('aria', 'Causes',  'movimento',    0.85, via='vento')
add('aria', 'Does',    'avvolgere',    0.85, via='spazio')
add('aria', 'Has',     'leggerezza',   0.90)
add('aria', 'IsA',     'elemento',     0.95, via='natura')
add('aria', 'OppositeOf','terra',      0.90)

# ── luce ─────────────────────────────────────────────────────────────────────
add('luce', 'Causes',  'visione',      0.95, via='chiarezza')
add('luce', 'Causes',  'conoscenza',   0.90, via='verità')
add('luce', 'Does',    'rivelare',     0.95, via='verità')
add('luce', 'Has',     'chiarezza',    0.90)
add('luce', 'IsA',     'energia',      0.90, via='natura')
add('luce', 'Requires','energia',      0.85)
add('luce', 'OppositeOf','buio',       0.95)
add('luce', 'OppositeOf','ombra',      0.90)

# ── buio ─────────────────────────────────────────────────────────────────────
add('buio', 'Causes',  'paura',        0.85, via='ignoto')
add('buio', 'Causes',  'riposo',       0.80, via='silenzio')
add('buio', 'Does',    'nascondere',   0.95, via='mistero')
add('buio', 'Has',     'ignoto',       0.90)
add('buio', 'IsA',     'condizione',   0.85, via='spazio')
add('buio', 'OppositeOf','luce',       0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 33 — INTELLETTO E CONOSCENZA (intelligenza, memoria, oblio, ignoranza, curiosità, saggezza)
# ═══════════════════════════════════════════════════════════════════════════

# ── intelligenza ─────────────────────────────────────────────────────────────
add('intelligenza', 'Causes',  'soluzione',    0.90, via='comprensione')
add('intelligenza', 'Causes',  'adattamento',  0.95, via='cambiamento')
add('intelligenza', 'Does',    'comprendere',  0.95, via='mente')
add('intelligenza', 'Does',    'collegare',    0.85, via='relazione')
add('intelligenza', 'Has',     'logica',       0.85)
add('intelligenza', 'IsA',     'capacità',     0.95, via='mente')
add('intelligenza', 'Requires','mente',        0.95)
add('intelligenza', 'OppositeOf','stupidità',  0.95)
add('intelligenza', 'OppositeOf','ignoranza',  0.80)

# ── memoria ──────────────────────────────────────────────────────────────────
add('memoria', 'Causes',  'identità',     0.95, via='storia')
add('memoria', 'Causes',  'conoscenza',   0.90, via='esperienza')
add('memoria', 'Does',    'conservare',   0.95, via='tempo')
add('memoria', 'Has',     'passato',      0.90)
add('memoria', 'IsA',     'capacità',     0.90, via='mente')
add('memoria', 'Requires','tempo',        0.90)
add('memoria', 'Requires','attenzione',   0.85)
add('memoria', 'OppositeOf','oblio',      0.95)

# ── oblio ────────────────────────────────────────────────────────────────────
add('oblio', 'Causes',  'perdita',      0.90, via='vuoto')
add('oblio', 'Causes',  'liberazione',  0.80, via='rinuncia') # Oblio come sollievo
add('oblio', 'Does',    'cancellare',   0.95, via='tempo')
add('oblio', 'Has',     'vuoto',        0.90)
add('oblio', 'IsA',     'processo',     0.85, via='mente')
add('oblio', 'Requires','tempo',        0.90)
add('oblio', 'OppositeOf','memoria',    0.95)
add('oblio', 'OppositeOf','ricordo',    0.95)

# ── ignoranza ────────────────────────────────────────────────────────────────
add('ignoranza', 'Causes',  'errore',       0.90, via='illusione')
add('ignoranza', 'Causes',  'paura',        0.85, via='ignoto')
add('ignoranza', 'Does',    'limitare',     0.90, via='confine')
add('ignoranza', 'Has',     'vuoto',        0.80)
add('ignoranza', 'IsA',     'condizione',   0.85, via='mente')
add('ignoranza', 'OppositeOf','conoscenza', 0.95)
add('ignoranza', 'OppositeOf','saggezza',   0.90)

# ── curiosità ────────────────────────────────────────────────────────────────
add('curiosità', 'Causes',  'scoperta',     0.95, via='domanda')
add('curiosità', 'Causes',  'conoscenza',   0.90, via='esplorazione')
add('curiosità', 'Does',    'cercare',      0.95, via='domanda')
add('curiosità', 'Has',     'desiderio',    0.90)
add('curiosità', 'IsA',     'spinta',       0.85, via='mente')
add('curiosità', 'Requires','domanda',      0.90)
add('curiosità', 'OppositeOf','indifferenza',0.95)
add('curiosità', 'OppositeOf','apatia',     0.90)

# ── saggezza ─────────────────────────────────────────────────────────────────
add('saggezza', 'Causes',  'equilibrio',   0.95, via='comprensione')
add('saggezza', 'Causes',  'pace',         0.85, via='accettazione')
add('saggezza', 'Does',    'integrare',    0.90, via='esperienza')
add('saggezza', 'Has',     'esperienza',   0.95)
add('saggezza', 'Has',     'profondità',   0.90)
add('saggezza', 'IsA',     'virtù',        0.95, via='mente')
add('saggezza', 'Requires','tempo',        0.90)
add('saggezza', 'Requires','esperienza',   0.95)
add('saggezza', 'OppositeOf','follia',     0.85)
add('saggezza', 'OppositeOf','ignoranza',  0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 34 — SENTIMENTI ED EMOZIONI (amore, odio, felicità, tristezza, paura, speranza, rabbia, compassione)
# ═══════════════════════════════════════════════════════════════════════════

# ── amore ────────────────────────────────────────────────────────────────────
add('amore', 'Causes',  'connessione',  0.95, via='cura')
add('amore', 'Causes',  'vita',         0.90, via='creazione')
add('amore', 'Does',    'unire',        0.95, via='relazione')
add('amore', 'Does',    'proteggere',   0.90, via='cura')
add('amore', 'Has',     'cura',         0.95)
add('amore', 'IsA',     'forza',        0.95, via='spirito')
add('amore', 'Requires','apertura',     0.90)
add('amore', 'Requires','vulnerabilità',0.85)
add('amore', 'OppositeOf','odio',       0.95)
add('amore', 'OppositeOf','indifferenza',0.90)

# ── odio ─────────────────────────────────────────────────────────────────────
add('odio', 'Causes',  'separazione',  0.95, via='conflitto')
add('odio', 'Causes',  'distruzione',  0.90, via='rabbia')
add('odio', 'Does',    'dividere',     0.95, via='confine')
add('odio', 'Has',     'rabbia',       0.90)
add('odio', 'IsA',     'forza',        0.85, via='ombra')
add('odio', 'Requires','dolore',       0.85) # Spesso nasce da un dolore irrisolto
add('odio', 'OppositeOf','amore',      0.95)
add('odio', 'OppositeOf','compassione',0.90)

# ── felicità ─────────────────────────────────────────────────────────────────
add('felicità', 'Causes',  'espansione',   0.90, via='energia')
add('felicità', 'Causes',  'gratitudine',  0.85, via='pienezza')
add('felicità', 'Does',    'celebrare',    0.90, via='vita')
add('felicità', 'Has',     'pienezza',     0.95)
add('felicità', 'IsA',     'stato',        0.95, via='armonia')
add('felicità', 'Requires','armonia',      0.90)
add('felicità', 'OppositeOf','tristezza',  0.95)
add('felicità', 'OppositeOf','sofferenza', 0.90)

# ── tristezza ────────────────────────────────────────────────────────────────
add('tristezza', 'Causes',  'riflessione',  0.85, via='interiorità')
add('tristezza', 'Causes',  'chiusura',     0.80, via='dolore')
add('tristezza', 'Does',    'rallentare',   0.85, via='tempo')
add('tristezza', 'Has',     'vuoto',        0.90)
add('tristezza', 'IsA',     'stato',        0.90, via='mente')
add('tristezza', 'Requires','perdita',      0.85)
add('tristezza', 'OppositeOf','felicità',   0.95)
add('tristezza', 'OppositeOf','gioia',      0.95)

# ── paura ────────────────────────────────────────────────────────────────────
add('paura', 'Causes',  'protezione',   0.90, via='istinto')
add('paura', 'Causes',  'paralisi',     0.85, via='blocco')
add('paura', 'Does',    'restringere',  0.90, via='confine')
add('paura', 'Has',     'ignoto',       0.95)
add('paura', 'IsA',     'istinto',      0.95, via='sopravvivenza')
add('paura', 'Requires','minaccia',     0.90)
add('paura', 'OppositeOf','coraggio',   0.95)
add('paura', 'OppositeOf','fiducia',    0.90)

# ── speranza ─────────────────────────────────────────────────────────────────
add('speranza', 'Causes',  'forza',        0.90, via='visione')
add('speranza', 'Causes',  'movimento',    0.85, via='futuro')
add('speranza', 'Does',    'aprire',       0.95, via='futuro')
add('speranza', 'Has',     'futuro',       0.95)
add('speranza', 'IsA',     'forza',        0.90, via='spirito')
add('speranza', 'Requires','desiderio',    0.90)
add('speranza', 'OppositeOf','disperazione',0.95)
add('speranza', 'OppositeOf','rassegnazione',0.90)

# ── rabbia ───────────────────────────────────────────────────────────────────
add('rabbia', 'Causes',  'azione',       0.90, via='energia')
add('rabbia', 'Causes',  'distruzione',  0.85, via='fuoco')
add('rabbia', 'Does',    'esplodere',    0.90, via='forza')
add('rabbia', 'Has',     'fuoco',        0.90)
add('rabbia', 'IsA',     'energia',      0.90, via='emozione')
add('rabbia', 'Requires','ingiustizia',  0.80) # Spesso trigger della rabbia
add('rabbia', 'OppositeOf','calma',      0.95)
add('rabbia', 'OppositeOf','pazienza',   0.90)

# ── compassione ──────────────────────────────────────────────────────────────
add('compassione', 'Causes',  'cura',         0.95, via='connessione')
add('compassione', 'Causes',  'guarigione',   0.90, via='amore')
add('compassione', 'Does',    'accogliere',   0.95, via='dolore')
add('compassione', 'Has',     'empatia',      0.95)
add('compassione', 'IsA',     'virtù',        0.90, via='spirito')
add('compassione', 'Requires','dolore',       0.85) # Si attiva di fronte al dolore altrui
add('compassione', 'Requires','empatia',      0.95)
add('compassione', 'OppositeOf','crudeltà',   0.95)
add('compassione', 'OppositeOf','indifferenza',0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 35 — MORALE ED ETICA (bene, male, colpa, perdono, purezza, peccato, innocenza)
# ═══════════════════════════════════════════════════════════════════════════

# ── bene ─────────────────────────────────────────────────────────────────────
add('bene', 'Causes',  'armonia',      0.95, via='azione')
add('bene', 'Causes',  'pace',         0.90, via='giustizia')
add('bene', 'Does',    'costruire',    0.85, via='cura')
add('bene', 'Has',     'verità',       0.80)
add('bene', 'IsA',     'valore',       0.95, via='etica')
add('bene', 'Requires','volontà',      0.90)
add('bene', 'Requires','scelta',       0.85)
add('bene', 'OppositeOf','male',       0.95)

# ── male ─────────────────────────────────────────────────────────────────────
add('male', 'Causes',  'sofferenza',   0.95, via='azione')
add('male', 'Causes',  'distruzione',  0.90, via='caos')
add('male', 'Does',    'ferire',       0.95, via='dolore')
add('male', 'Has',     'caos',         0.85)
add('male', 'IsA',     'forza',        0.80, via='ombra')
add('male', 'Requires','scelta',       0.85)
add('male', 'OppositeOf','bene',       0.95)

# ── colpa ────────────────────────────────────────────────────────────────────
add('colpa', 'Causes',  'rimorso',      0.95, via='coscienza')
add('colpa', 'Causes',  'sofferenza',   0.90, via='dolore')
add('colpa', 'Does',    'pesare',       0.85, via='mente')
add('colpa', 'Has',     'responsabilità',0.95)
add('colpa', 'IsA',     'peso',         0.85, via='spirito')
add('colpa', 'Requires','coscienza',    0.90)
add('colpa', 'Requires','azione',       0.95)
add('colpa', 'OppositeOf','innocenza',  0.95)

# ── perdono ──────────────────────────────────────────────────────────────────
add('perdono', 'Causes',  'liberazione',  0.95, via='pace')
add('perdono', 'Causes',  'guarigione',   0.90, via='amore')
add('perdono', 'Does',    'lasciare',     0.85, via='passato') # Lasciar andare
add('perdono', 'Has',     'grazia',       0.90)
add('perdono', 'IsA',     'atto',         0.90, via='volontà')
add('perdono', 'Requires','compassione',  0.95)
add('perdono', 'Requires','colpa',        0.85) # Dialettica: si perdona una colpa
add('perdono', 'OppositeOf','vendetta',   0.95)
add('perdono', 'OppositeOf','rancore',    0.90)

# ── purezza ──────────────────────────────────────────────────────────────────
add('purezza', 'Causes',  'chiarezza',    0.90, via='luce')
add('purezza', 'Causes',  'bellezza',     0.85, via='verità')
add('purezza', 'Does',    'illuminare',   0.85, via='spirito')
add('purezza', 'Has',     'luce',         0.90)
add('purezza', 'IsA',     'stato',        0.90, via='essenza')
add('purezza', 'Requires','verità',       0.85)
add('purezza', 'OppositeOf','corruzione', 0.95)

# ── peccato ──────────────────────────────────────────────────────────────────
add('peccato', 'Causes',  'colpa',        0.95, via='azione')
add('peccato', 'Causes',  'separazione',  0.85, via='spirito')
add('peccato', 'Does',    'violare',      0.90, via='regola')
add('peccato', 'Has',     'errore',       0.90)
add('peccato', 'IsA',     'azione',       0.90, via='scelta')
add('peccato', 'Requires','legge',        0.85) # Esiste in relazione a una legge morale
add('peccato', 'OppositeOf','virtù',      0.95)

# ── innocenza ────────────────────────────────────────────────────────────────
add('innocenza', 'Causes',  'fiducia',      0.90, via='purezza')
add('innocenza', 'Causes',  'vulnerabilità',0.85, via='apertura')
add('innocenza', 'Does',    'credere',      0.85, via='verità')
add('innocenza', 'Has',     'purezza',      0.95)
add('innocenza', 'IsA',     'stato',        0.90, via='spirito')
add('innocenza', 'Requires','non_conoscenza',0.80) # Dialettica
add('innocenza', 'OppositeOf','colpa',      0.95)
add('innocenza', 'OppositeOf','malizia',    0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 36 — POLITICA, POTERE E LIBERTÀ (libertà, schiavitù, ribellione, ordine, anarchia)
# ═══════════════════════════════════════════════════════════════════════════

# ── libertà ──────────────────────────────────────────────────────────────────
add('libertà', 'Causes',  'responsabilità',0.95, via='scelta')
add('libertà', 'Causes',  'creazione',    0.90, via='volontà')
add('libertà', 'Does',    'scegliere',    0.95, via='volontà')
add('libertà', 'Has',     'spazio',       0.90)
add('libertà', 'IsA',     'diritto',      0.95, via='individuo')
add('libertà', 'Requires','coraggio',     0.85)
add('libertà', 'OppositeOf','schiavitù',  0.95)
add('libertà', 'OppositeOf','prigionia',  0.95)
add('libertà', 'OppositeOf','oppressione',0.90)

# ── schiavitù ────────────────────────────────────────────────────────────────
add('schiavitù', 'Causes',  'sofferenza',   0.95, via='oppressione')
add('schiavitù', 'Causes',  'ribellione',   0.85, via='dolore')
add('schiavitù', 'Does',    'sottomettere', 0.95, via='potere')
add('schiavitù', 'Has',     'catene',       0.90)
add('schiavitù', 'IsA',     'condizione',   0.90, via='società')
add('schiavitù', 'Requires','potere',       0.90) # Necessita un oppressore
add('schiavitù', 'OppositeOf','libertà',    0.95)
add('schiavitù', 'OppositeOf','indipendenza',0.90)

# ── ribellione ───────────────────────────────────────────────────────────────
add('ribellione', 'Causes',  'cambiamento',  0.95, via='forza')
add('ribellione', 'Causes',  'liberazione',  0.90, via='rottura')
add('ribellione', 'Does',    'rifiutare',    0.95, via='legge')
add('ribellione', 'Has',     'rabbia',       0.85)
add('ribellione', 'IsA',     'azione',       0.90, via='volontà')
add('ribellione', 'Requires','oppressione',  0.90) # Spinta dialettica
add('ribellione', 'OppositeOf','obbedienza', 0.95)
add('ribellione', 'OppositeOf','sottomissione',0.95)

# ── ordine ───────────────────────────────────────────────────────────────────
add('ordine', 'Causes',  'stabilità',    0.95, via='struttura')
add('ordine', 'Causes',  'sicurezza',    0.90, via='regola')
add('ordine', 'Does',    'organizzare',  0.95, via='sistema')
add('ordine', 'Has',     'struttura',    0.95)
add('ordine', 'IsA',     'condizione',   0.90, via='sistema')
add('ordine', 'Requires','regola',       0.95)
add('ordine', 'Requires','controllo',    0.85)
add('ordine', 'OppositeOf','caos',       0.95)
add('ordine', 'OppositeOf','anarchia',   0.90)

# ── anarchia ─────────────────────────────────────────────────────────────────
add('anarchia', 'Causes',  'caos',         0.90, via='libertà') # Caos potenziale
add('anarchia', 'Causes',  'liberazione',  0.85, via='rottura')
add('anarchia', 'Does',    'rifiutare',    0.95, via='potere')
add('anarchia', 'Has',     'libertà',      0.90) # Estrema
add('anarchia', 'IsA',     'sistema',      0.80, via='società')
add('anarchia', 'Requires','assenza',      0.90, via='potere')
add('anarchia', 'OppositeOf','ordine',     0.90)
add('anarchia', 'OppositeOf','governo',    0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 37 — SPAZIO, TEMPO E DIVENIRE (passato, presente, futuro, istante)
# ═══════════════════════════════════════════════════════════════════════════

# ── passato ──────────────────────────────────────────────────────────────────
add('passato', 'Causes',  'memoria',      0.95, via='storia')
add('passato', 'Causes',  'insegnamento', 0.90, via='esperienza')
add('passato', 'Does',    'formare',      0.90, via='radici')
add('passato', 'Has',     'storia',       0.95)
add('passato', 'IsA',     'dimensione',   0.90, via='tempo')
add('passato', 'Requires','tempo',        0.95)
add('passato', 'OppositeOf','futuro',     0.95)

# ── presente ─────────────────────────────────────────────────────────────────
add('presente', 'Causes',  'azione',       0.95, via='presenza')
add('presente', 'Causes',  'consapevolezza',0.90, via='attenzione')
add('presente', 'Does',    'esistere',     0.95, via='realtà')
add('presente', 'Has',     'realtà',       0.95)
add('presente', 'IsA',     'dimensione',   0.90, via='tempo')
add('presente', 'Requires','presenza',     0.95)
add('presente', 'OppositeOf','passato',    0.85)
add('presente', 'OppositeOf','futuro',     0.85)

# ── futuro ───────────────────────────────────────────────────────────────────
add('futuro', 'Causes',  'speranza',     0.90, via='visione')
add('futuro', 'Causes',  'incertezza',   0.85, via='ignoto')
add('futuro', 'Does',    'divenire',     0.95, via='tempo')
add('futuro', 'Has',     'possibilità',  0.95)
add('futuro', 'IsA',     'dimensione',   0.90, via='tempo')
add('futuro', 'Requires','tempo',        0.95)
add('futuro', 'OppositeOf','passato',    0.95)

# ── istante ──────────────────────────────────────────────────────────────────
add('istante', 'Causes',  'illuminazione',0.85, via='intuizione')
add('istante', 'Causes',  'svolta',       0.80, via='decisione')
add('istante', 'Does',    'accadere',     0.95, via='presenza')
add('istante', 'Has',     'intensità',    0.90)
add('istante', 'IsA',     'punto',        0.95, via='tempo')
add('istante', 'Requires','presenza',     0.90)
add('istante', 'OppositeOf','eternità',   0.95)
add('istante', 'OppositeOf','durata',     0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 38 — RELAZIONI, INCONTRO E DIALOGO (amicizia, inimicizia, tradimento, fedeltà, alleanza, incontro)
# ═══════════════════════════════════════════════════════════════════════════

# ── amicizia ─────────────────────────────────────────────────────────────────
add('amicizia', 'Causes',  'supporto',     0.95, via='cura')
add('amicizia', 'Causes',  'condivisione', 0.90, via='fiducia')
add('amicizia', 'Does',    'unire',        0.95, via='legame')
add('amicizia', 'Has',     'fiducia',      0.95)
add('amicizia', 'IsA',     'relazione',    0.95, via='affetto')
add('amicizia', 'Requires','lealtà',       0.90)
add('amicizia', 'OppositeOf','inimicizia', 0.95)
add('amicizia', 'OppositeOf','ostilità',   0.90)

# ── inimicizia ───────────────────────────────────────────────────────────────
add('inimicizia', 'Causes',  'conflitto',    0.95, via='distanza')
add('inimicizia', 'Causes',  'diffidenza',   0.90, via='paura')
add('inimicizia', 'Does',    'dividere',     0.90, via='confine')
add('inimicizia', 'Has',     'ostilità',     0.95)
add('inimicizia', 'IsA',     'relazione',    0.85, via='tensione')
add('inimicizia', 'OppositeOf','amicizia',   0.95)
add('inimicizia', 'OppositeOf','alleanza',   0.90)

# ── tradimento ───────────────────────────────────────────────────────────────
add('tradimento', 'Causes',  'dolore',       0.95, via='rottura')
add('tradimento', 'Causes',  'sfiducia',     0.90, via='inganno')
add('tradimento', 'Does',    'rompere',      0.95, via='fiducia')
add('tradimento', 'Has',     'inganno',      0.90)
add('tradimento', 'IsA',     'azione',       0.90, via='scelta')
add('tradimento', 'Requires','fiducia',      0.95) # Dialettica: tradisci solo chi si fida
add('tradimento', 'OppositeOf','fedeltà',    0.95)
add('tradimento', 'OppositeOf','lealtà',     0.95)

# ── fedeltà ──────────────────────────────────────────────────────────────────
add('fedeltà', 'Causes',  'sicurezza',    0.95, via='stabilità')
add('fedeltà', 'Causes',  'durata',       0.90, via='tempo')
add('fedeltà', 'Does',    'mantenere',    0.90, via='promessa')
add('fedeltà', 'Has',     'costanza',     0.95)
add('fedeltà', 'IsA',     'virtù',        0.90, via='etica')
add('fedeltà', 'Requires','impegno',      0.90)
add('fedeltà', 'OppositeOf','tradimento', 0.95)

# ── alleanza ─────────────────────────────────────────────────────────────────
add('alleanza', 'Causes',  'forza',        0.95, via='unione')
add('alleanza', 'Causes',  'protezione',   0.90, via='accordo')
add('alleanza', 'Does',    'collaborare',  0.95, via='scopo')
add('alleanza', 'Has',     'scopo',        0.95) # Spesso ha un fine pratico
add('alleanza', 'IsA',     'relazione',    0.90, via='accordo')
add('alleanza', 'Requires','accordo',      0.95)
add('alleanza', 'OppositeOf','inimicizia', 0.90)

# ── incontro ─────────────────────────────────────────────────────────────────
add('incontro', 'Causes',  'scambio',      0.90, via='dialogo')
add('incontro', 'Causes',  'cambiamento',  0.85, via='connessione')
add('incontro', 'Does',    'avvicinare',   0.95, via='spazio')
add('incontro', 'Has',     'presenza',     0.95)
add('incontro', 'IsA',     'evento',       0.90, via='tempo')
add('incontro', 'Requires','spazio',       0.90)
add('incontro', 'OppositeOf','separazione',0.90)
add('incontro', 'OppositeOf','addio',      0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 39 — INTERAZIONE, COMUNICAZIONE E SALUTI (ciao, addio, benvenuto, saluto, parola, silenzio, ascolto)
# ═══════════════════════════════════════════════════════════════════════════

# ── parola ───────────────────────────────────────────────────────────────────
add('parola', 'Causes',  'comprensione', 0.95, via='significato')
add('parola', 'Causes',  'azione',       0.85, via='pensiero')
add('parola', 'Does',    'esprimere',    0.95, via='voce')
add('parola', 'Has',     'significato',  0.95)
add('parola', 'IsA',     'strumento',    0.90, via='linguaggio')
add('parola', 'Requires','pensiero',     0.90)
add('parola', 'OppositeOf','silenzio',   0.95)

# ── silenzio ─────────────────────────────────────────────────────────────────
add('silenzio', 'Causes',  'ascolto',      0.95, via='attenzione')
add('silenzio', 'Causes',  'riflessione',  0.90, via='interiorità')
add('silenzio', 'Does',    'accogliere',   0.85, via='spazio')
add('silenzio', 'Has',     'vuoto',        0.90)
add('silenzio', 'IsA',     'stato',        0.90, via='presenza')
add('silenzio', 'Requires','presenza',     0.85)
add('silenzio', 'OppositeOf','parola',     0.95)
add('silenzio', 'OppositeOf','rumore',     0.95)

# ── ascolto ──────────────────────────────────────────────────────────────────
add('ascolto', 'Causes',  'comprensione', 0.95, via='attenzione')
add('ascolto', 'Causes',  'connessione',  0.90, via='presenza')
add('ascolto', 'Does',    'ricevere',     0.95, via='mente')
add('ascolto', 'Has',     'attenzione',   0.95)
add('ascolto', 'IsA',     'azione',       0.90, via='volontà')
add('ascolto', 'Requires','silenzio',     0.90)
add('ascolto', 'OppositeOf','sordità',    0.90)
add('ascolto', 'OppositeOf','indifferenza',0.85)

# ── saluto ───────────────────────────────────────────────────────────────────
add('saluto', 'Causes',  'riconoscimento',0.95, via='presenza')
add('saluto', 'Causes',  'apertura',     0.90, via='relazione')
add('saluto', 'Does',    'incontrare',   0.90, via='gesto')
add('saluto', 'Has',     'rispetto',     0.85)
add('saluto', 'IsA',     'atto',         0.90, via='comunicazione')
add('saluto', 'Requires','presenza',     0.95)
add('saluto', 'OppositeOf','ignorare',   0.90)

# ── ciao ─────────────────────────────────────────────────────────────────────
add('ciao', 'Causes',  'incontro',     0.95, via='apertura')
add('ciao', 'Does',    'salutare',     0.95, via='parola')
add('ciao', 'Has',     'informalità',  0.85)
add('ciao', 'IsA',     'saluto',       0.95, via='comunicazione')
add('ciao', 'Requires','presenza',     0.90)
add('ciao', 'OppositeOf','addio',      0.95)

# ── addio ────────────────────────────────────────────────────────────────────
add('addio', 'Causes',  'separazione',  0.95, via='distanza')
add('addio', 'Causes',  'fine',         0.90, via='tempo')
add('addio', 'Does',    'chiudere',     0.90, via='relazione')
add('addio', 'Has',     'definitività', 0.85)
add('addio', 'IsA',     'saluto',       0.90, via='comunicazione')
add('addio', 'Requires','partenza',     0.95)
add('addio', 'OppositeOf','ciao',       0.95)
add('addio', 'OppositeOf','benvenuto',  0.95)

# ── benvenuto ────────────────────────────────────────────────────────────────
add('benvenuto', 'Causes',  'accoglienza',  0.95, via='apertura')
add('benvenuto', 'Causes',  'inclusione',   0.90, via='spazio')
add('benvenuto', 'Does',    'ospitare',     0.95, via='casa')
add('benvenuto', 'Has',     'calore',       0.85)
add('benvenuto', 'IsA',     'saluto',       0.95, via='comunicazione')
add('benvenuto', 'Requires','arrivo',       0.95)
add('benvenuto', 'OppositeOf','addio',      0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 40 — CIVILTÀ, NAZIONI E DIRITTO (civiltà, nazione, popolo, governo, politica, diritto, dovere)
# ═══════════════════════════════════════════════════════════════════════════

# ── civiltà ──────────────────────────────────────────────────────────────────
add('civiltà', 'Causes',  'progresso',    0.95, via='conoscenza')
add('civiltà', 'Causes',  'cultura',      0.90, via='storia')
add('civiltà', 'Does',    'costruire',    0.90, via='struttura')
add('civiltà', 'Has',     'storia',       0.95)
add('civiltà', 'IsA',     'società',      0.95, via='tempo') # Società complessa e duratura
add('civiltà', 'Requires','ordine',       0.90)
add('civiltà', 'Requires','memoria',      0.85)
add('civiltà', 'OppositeOf','barbarie',   0.90)
add('civiltà', 'OppositeOf','selvaggio',  0.85)

# ── nazione ──────────────────────────────────────────────────────────────────
add('nazione', 'Causes',  'identità',     0.90, via='cultura')
add('nazione', 'Causes',  'confine',      0.85, via='spazio')
add('nazione', 'Does',    'unire',        0.90, via='appartenenza')
add('nazione', 'Has',     'territorio',   0.90)
add('nazione', 'Has',     'popolo',       0.95)
add('nazione', 'IsA',     'comunità',     0.90, via='storia')
add('nazione', 'Requires','stato',        0.85)

# ── popolo ───────────────────────────────────────────────────────────────────
add('popolo', 'Causes',  'forza',        0.90, via='unione')
add('popolo', 'Causes',  'storia',       0.95, via='azione')
add('popolo', 'Does',    'esistere',     0.90, via='vita')
add('popolo', 'Has',     'identità',     0.90)
add('popolo', 'IsA',     'comunità',     0.95, via='relazione')
add('popolo', 'PartOf',  'nazione',      0.95)
add('popolo', 'Requires','radici',       0.85)

# ── governo ──────────────────────────────────────────────────────────────────
add('governo', 'Causes',  'ordine',       0.95, via='regola')
add('governo', 'Causes',  'direzione',    0.90, via='potere')
add('governo', 'Does',    'guidare',      0.95, via='scelta')
add('governo', 'Does',    'amministrare', 0.90, via='struttura')
add('governo', 'Has',     'potere',       0.95)
add('governo', 'IsA',     'istituzione',  0.90, via='società')
add('governo', 'Requires','stato',        0.90)
add('governo', 'Requires','legittimità',  0.85)
add('governo', 'OppositeOf','anarchia',   0.95)

# ── politica ─────────────────────────────────────────────────────────────────
add('politica', 'Causes',  'scelta',       0.90, via='potere')
add('politica', 'Causes',  'conflitto',    0.85, via='visione')
add('politica', 'Does',    'organizzare',  0.90, via='società')
add('politica', 'Has',     'visione',      0.85)
add('politica', 'IsA',     'azione',       0.90, via='comunità')
add('politica', 'Requires','potere',       0.95)
add('politica', 'Requires','società',      0.95)

# ── diritto ──────────────────────────────────────────────────────────────────
add('diritto', 'Causes',  'giustizia',    0.95, via='regola')
add('diritto', 'Causes',  'protezione',   0.90, via='legge')
add('diritto', 'Does',    'garantire',    0.95, via='libertà')
add('diritto', 'Has',     'legittimità',  0.90)
add('diritto', 'IsA',     'principio',    0.90, via='società')
add('diritto', 'Requires','dovere',       0.95) # Inseparabili
add('diritto', 'OppositeOf','abuso',      0.90)

# ── dovere ───────────────────────────────────────────────────────────────────
add('dovere', 'Causes',  'responsabilità',0.95, via='azione')
add('dovere', 'Causes',  'ordine',       0.85, via='rispetto')
add('dovere', 'Does',    'vincolare',    0.90, via='etica')
add('dovere', 'Has',     'necessità',    0.90)
add('dovere', 'IsA',     'principio',    0.90, via='società')
add('dovere', 'Requires','diritto',      0.95) # Inseparabili
add('dovere', 'OppositeOf','arbitrio',   0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 41 — GUERRA E PROGRESSO (guerra, pace, progresso, decadenza)
# ═══════════════════════════════════════════════════════════════════════════

# ── guerra ───────────────────────────────────────────────────────────────────
add('guerra', 'Causes',  'distruzione',  0.95, via='forza')
add('guerra', 'Causes',  'morte',        0.90, via='violenza')
add('guerra', 'Does',    'combattere',   0.95, via='conflitto')
add('guerra', 'Has',     'violenza',     0.95)
add('guerra', 'IsA',     'conflitto',    0.95, via='storia')
add('guerra', 'Requires','nemico',       0.90)
add('guerra', 'OppositeOf','pace',       0.95)
add('guerra', 'OppositeOf','armonia',    0.90)

# ── progresso ────────────────────────────────────────────────────────────────
add('progresso', 'Causes',  'sviluppo',     0.95, via='tempo')
add('progresso', 'Causes',  'benessere',    0.85, via='scoperta')
add('progresso', 'Does',    'avanzare',     0.95, via='futuro')
add('progresso', 'Has',     'evoluzione',   0.90)
add('progresso', 'IsA',     'movimento',    0.90, via='storia')
add('progresso', 'Requires','conoscenza',   0.90)
add('progresso', 'OppositeOf','regresso',   0.95)
add('progresso', 'OppositeOf','decadenza',  0.90)
add('progresso', 'OppositeOf','stagnazione',0.90)

# ── decadenza ────────────────────────────────────────────────────────────────
add('decadenza', 'Causes',  'fine',         0.90, via='tempo')
add('decadenza', 'Causes',  'corruzione',   0.85, via='struttura')
add('decadenza', 'Does',    'crollare',     0.90, via='debolezza')
add('decadenza', 'Has',     'debolezza',    0.90)
add('decadenza', 'IsA',     'processo',     0.90, via='storia')
add('decadenza', 'Requires','tempo',        0.95) # È un processo temporale
add('decadenza', 'OppositeOf','fioritura',  0.90)
add('decadenza', 'OppositeOf','progresso',  0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 42 — ARTE, ESTETICA E CREATIVITÀ (arte, bellezza, bruttezza, creatività, ispirazione, opera, armonia)
# ═══════════════════════════════════════════════════════════════════════════

# ── arte ─────────────────────────────────────────────────────────────────────
add('arte', 'Causes',  'bellezza',     0.95, via='forma')
add('arte', 'Causes',  'emozione',     0.90, via='espressione')
add('arte', 'Does',    'esprimere',    0.95, via='anima')
add('arte', 'Does',    'trasformare',  0.90, via='materia')
add('arte', 'Has',     'creatività',   0.95)
add('arte', 'IsA',     'linguaggio',   0.90, via='cultura')
add('arte', 'Requires','ispirazione',  0.95)
add('arte', 'OppositeOf','tecnica',    0.70) # Dialettica
add('arte', 'OppositeOf','utile',      0.80) # Dialettica (l'arte non ha fine pratico)

# ── bellezza ─────────────────────────────────────────────────────────────────
add('bellezza', 'Causes',  'meraviglia',   0.95, via='contemplazione')
add('bellezza', 'Causes',  'attrazione',   0.90, via='armonia')
add('bellezza', 'Does',    'elevare',      0.90, via='spirito')
add('bellezza', 'Has',     'armonia',      0.95)
add('bellezza', 'IsA',     'valore',       0.90, via='estetica')
add('bellezza', 'Requires','forma',        0.85)
add('bellezza', 'OppositeOf','bruttezza',  0.95)

# ── bruttezza ────────────────────────────────────────────────────────────────
add('bruttezza', 'Causes',  'repulsione',   0.90, via='disarmonia')
add('bruttezza', 'Causes',  'dolore',       0.80, via='forma')
add('bruttezza', 'Does',    'turbare',      0.85, via='senso')
add('bruttezza', 'Has',     'disarmonia',   0.95)
add('bruttezza', 'IsA',     'condizione',   0.85, via='estetica')
add('bruttezza', 'OppositeOf','bellezza',   0.95)
add('bruttezza', 'OppositeOf','grazia',     0.90)

# ── creatività ───────────────────────────────────────────────────────────────
add('creatività', 'Causes',  'novità',       0.95, via='idea')
add('creatività', 'Causes',  'opera',        0.90, via='azione')
add('creatività', 'Does',    'inventare',    0.95, via='immaginazione')
add('creatività', 'Has',     'immaginazione',0.95)
add('creatività', 'IsA',     'energia',      0.90, via='mente')
add('creatività', 'Requires','libertà',      0.90)
add('creatività', 'OppositeOf','imitazione', 0.95)
add('creatività', 'OppositeOf','abitudine',  0.90)

# ── ispirazione ──────────────────────────────────────────────────────────────
add('ispirazione', 'Causes',  'creatività',   0.95, via='scintilla')
add('ispirazione', 'Causes',  'visione',      0.90, via='mente')
add('ispirazione', 'Does',    'guidare',      0.85, via='azione')
add('ispirazione', 'Has',     'intuizione',   0.90)
add('ispirazione', 'IsA',     'spinta',       0.90, via='spirito')
add('ispirazione', 'Requires','apertura',     0.85)
add('ispirazione', 'OppositeOf','blocco',     0.90)

# ── opera ────────────────────────────────────────────────────────────────────
add('opera', 'Causes',  'memoria',      0.85, via='storia')
add('opera', 'Does',    'esistere',     0.90, via='forma')
add('opera', 'Does',    'testimoniare', 0.85, via='creatore')
add('opera', 'Has',     'forma',        0.95)
add('opera', 'IsA',     'risultato',    0.95, via='creazione')
add('opera', 'Requires','lavoro',       0.90)
add('opera', 'Requires','creatore',     0.95)
add('opera', 'OppositeOf','nulla',      0.80)

# ── armonia ──────────────────────────────────────────────────────────────────
add('armonia', 'Causes',  'bellezza',     0.95, via='forma')
add('armonia', 'Causes',  'pace',         0.90, via='equilibrio')
add('armonia', 'Does',    'unire',        0.90, via='relazione')
add('armonia', 'Has',     'equilibrio',   0.95)
add('armonia', 'IsA',     'ordine',       0.90, via='natura')
add('armonia', 'Requires','relazione',    0.90)
add('armonia', 'OppositeOf','caos',       0.95)
add('armonia', 'OppositeOf','disordine',  0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 43 — SCIENZA, METODO E SCOPERTA (scienza, natura, scoperta, invenzione, logica, dubbio, errore)
# ═══════════════════════════════════════════════════════════════════════════

# ── scienza ──────────────────────────────────────────────────────────────────
add('scienza', 'Causes',  'conoscenza',   0.95, via='metodo')
add('scienza', 'Causes',  'progresso',    0.90, via='scoperta')
add('scienza', 'Does',    'indagare',     0.95, via='natura')
add('scienza', 'Does',    'spiegare',     0.90, via='regola')
add('scienza', 'Has',     'metodo',       0.95)
add('scienza', 'IsA',     'sistema',      0.90, via='conoscenza')
add('scienza', 'Requires','dubbio',       0.90)
add('scienza', 'Requires','logica',       0.95)
add('scienza', 'OppositeOf','magia',      0.95)
add('scienza', 'OppositeOf','dogma',      0.90)

# ── natura ───────────────────────────────────────────────────────────────────
add('natura', 'Causes',  'vita',         0.95, via='energia')
add('natura', 'Causes',  'evoluzione',   0.90, via='tempo')
add('natura', 'Does',    'creare',       0.90, via='forma')
add('natura', 'Does',    'regolare',     0.85, via='legge')
add('natura', 'Has',     'legge',        0.90) # Leggi naturali
add('natura', 'IsA',     'sistema',      0.95, via='universo')
add('natura', 'OppositeOf','artificio',  0.95)
add('natura', 'OppositeOf','cultura',    0.80) # Dialettica natura/cultura

# ── scoperta ─────────────────────────────────────────────────────────────────
add('scoperta', 'Causes',  'conoscenza',   0.95, via='verità')
add('scoperta', 'Causes',  'cambiamento',  0.90, via='visione')
add('scoperta', 'Does',    'svelare',      0.95, via='mistero')
add('scoperta', 'Has',     'novità',       0.90)
add('scoperta', 'IsA',     'evento',       0.90, via='scienza')
add('scoperta', 'Requires','ricerca',      0.95)
add('scoperta', 'Requires','curiosità',    0.90)
add('scoperta', 'OppositeOf','invenzione', 0.85) # Scopri ciò che esiste, inventi ciò che non esiste

# ── invenzione ───────────────────────────────────────────────────────────────
add('invenzione', 'Causes',  'progresso',    0.90, via='tecnica')
add('invenzione', 'Causes',  'cambiamento',  0.85, via='uso')
add('invenzione', 'Does',    'creare',       0.95, via='novità')
add('invenzione', 'Has',     'tecnica',      0.90)
add('invenzione', 'IsA',     'opera',        0.90, via='creatività')
add('invenzione', 'Requires','ingegno',      0.95)
add('invenzione', 'OppositeOf','scoperta',   0.85)

# ── logica ───────────────────────────────────────────────────────────────────
add('logica', 'Causes',  'coerenza',     0.95, via='regola')
add('logica', 'Causes',  'chiarezza',    0.90, via='pensiero')
add('logica', 'Does',    'strutturare',  0.95, via='ragione')
add('logica', 'Has',     'regola',       0.95)
add('logica', 'IsA',     'metodo',       0.90, via='mente')
add('logica', 'Requires','ragione',      0.95)
add('logica', 'OppositeOf','assurdo',    0.95)
add('logica', 'OppositeOf','emozione',   0.80) # Dialettica

# ── dubbio ───────────────────────────────────────────────────────────────────
add('dubbio', 'Causes',  'ricerca',      0.95, via='domanda')
add('dubbio', 'Causes',  'crisi',        0.80, via='incertezza')
add('dubbio', 'Does',    'interrogare',  0.95, via='verità')
add('dubbio', 'Has',     'domanda',      0.95)
add('dubbio', 'IsA',     'spinta',       0.85, via='conoscenza')
add('dubbio', 'Requires','intelligenza', 0.85)
add('dubbio', 'OppositeOf','certezza',   0.95)
add('dubbio', 'OppositeOf','dogma',      0.95)

# ── errore ───────────────────────────────────────────────────────────────────
add('errore', 'Causes',  'insegnamento', 0.90, via='esperienza')
add('errore', 'Causes',  'fallimento',   0.85, via='azione')
add('errore', 'Does',    'deviare',      0.90, via='regola')
add('errore', 'Has',     'falsità',      0.85)
add('errore', 'IsA',     'evento',       0.90, via='azione')
add('errore', 'Requires','scelta',       0.85)
add('errore', 'OppositeOf','verità',     0.95)
add('errore', 'OppositeOf','correttezza',0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 44 — STATI MENTALI E PERCEZIONE ALTERATA (sogno, illusione, allucinazione, estasi, trance, ipnosi, meditazione)
# ═══════════════════════════════════════════════════════════════════════════

# ── allucinazione ────────────────────────────────────────────────────────────
add('allucinazione', 'Causes',  'paura',        0.85, via='ignoto')
add('allucinazione', 'Causes',  'confusione',   0.90, via='illusione')
add('allucinazione', 'Does',    'ingannare',    0.95, via='percezione')
add('allucinazione', 'Has',     'illusione',    0.95)
add('allucinazione', 'IsA',     'percezione',   0.90, via='mente')
add('allucinazione', 'Requires','mente',        0.95)
add('allucinazione', 'OppositeOf','realtà',     0.95)

# ── estasi ───────────────────────────────────────────────────────────────────
add('estasi', 'Causes',  'gioia',        0.95, via='pienezza')
add('estasi', 'Causes',  'illuminazione',0.90, via='spirito')
add('estasi', 'Does',    'elevare',      0.95, via='spirito')
add('estasi', 'Has',     'pienezza',     0.90)
add('estasi', 'IsA',     'stato',        0.90, via='coscienza')
add('estasi', 'Requires','abbandono',    0.85)
add('estasi', 'OppositeOf','angoscia',   0.95)

# ── trance ───────────────────────────────────────────────────────────────────
add('trance', 'Causes',  'visione',      0.85, via='mente')
add('trance', 'Causes',  'connessione',  0.80, via='inconscio')
add('trance', 'Does',    'sospendere',   0.90, via='coscienza')
add('trance', 'Has',     'profondità',   0.85)
add('trance', 'IsA',     'stato',        0.90, via='mente')
add('trance', 'Requires','concentrazione',0.85)
add('trance', 'OppositeOf','veglia',     0.90)

# ── ipnosi ───────────────────────────────────────────────────────────────────
add('ipnosi', 'Causes',  'suggestione',  0.95, via='mente')
add('ipnosi', 'Causes',  'abbandono',    0.85, via='volontà')
add('ipnosi', 'Does',    'guidare',      0.90, via='inconscio')
add('ipnosi', 'Has',     'controllo',    0.80)
add('ipnosi', 'IsA',     'processo',     0.90, via='mente')
add('ipnosi', 'Requires','suggestione',  0.95)
add('ipnosi', 'OppositeOf','coscienza',  0.85)

# ── meditazione ──────────────────────────────────────────────────────────────
add('meditazione', 'Causes',  'calma',        0.95, via='silenzio')
add('meditazione', 'Causes',  'consapevolezza',0.95, via='presenza')
add('meditazione', 'Does',    'centrare',     0.90, via='mente')
add('meditazione', 'Has',     'silenzio',     0.95)
add('meditazione', 'IsA',     'pratica',      0.90, via='spirito')
add('meditazione', 'Requires','silenzio',     0.95)
add('meditazione', 'Requires','attenzione',   0.90)
add('meditazione', 'OppositeOf','distrazione',0.95)
add('meditazione', 'OppositeOf','agitazione', 0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 45 — CONDIZIONI CORPOREE E BIOLOGICHE (fame, sete, sonno, fatica, energia, dolore, piacere, istinto)
# ═══════════════════════════════════════════════════════════════════════════

# ── fame ─────────────────────────────────────────────────────────────────────
add('fame', 'Causes',  'ricerca',      0.90, via='nutrimento')
add('fame', 'Causes',  'desiderio',    0.85, via='bisogno')
add('fame', 'Does',    'segnalare',    0.95, via='mancanza')
add('fame', 'Has',     'bisogno',      0.95)
add('fame', 'IsA',     'stimolo',      0.90, via='corpo')
add('fame', 'Requires','corpo',        0.95)
add('fame', 'OppositeOf','sazietà',    0.95)
add('fame', 'OppositeOf','pienezza',   0.90)

# ── sete ─────────────────────────────────────────────────────────────────────
add('sete', 'Causes',  'ricerca',      0.90, via='acqua')
add('sete', 'Causes',  'desiderio',    0.85, via='bisogno')
add('sete', 'Does',    'segnalare',    0.95, via='mancanza')
add('sete', 'Has',     'bisogno',      0.95)
add('sete', 'IsA',     'stimolo',      0.90, via='corpo')
add('sete', 'Requires','corpo',        0.95)
add('sete', 'OppositeOf','sazietà',    0.85)

# ── sonno ────────────────────────────────────────────────────────────────────
add('sonno', 'Causes',  'riposo',       0.95, via='corpo')
add('sonno', 'Causes',  'sogno',        0.90, via='mente')
add('sonno', 'Does',    'ristorare',    0.95, via='energia')
add('sonno', 'Has',     'abbandono',    0.85)
add('sonno', 'IsA',     'stato',        0.90, via='corpo')
add('sonno', 'Requires','stanchezza',   0.85)
add('sonno', 'OppositeOf','veglia',     0.95)

# ── fatica ───────────────────────────────────────────────────────────────────
add('fatica', 'Causes',  'stanchezza',   0.95, via='sforzo')
add('fatica', 'Causes',  'crescita',     0.85, via='resistenza')
add('fatica', 'Does',    'consumare',    0.90, via='energia')
add('fatica', 'Has',     'sforzo',       0.95)
add('fatica', 'IsA',     'sensazione',   0.90, via='corpo')
add('fatica', 'Requires','lavoro',       0.85)
add('fatica', 'OppositeOf','riposo',     0.90)
add('fatica', 'OppositeOf','energia',    0.85)

# ── energia ──────────────────────────────────────────────────────────────────
add('energia', 'Causes',  'movimento',    0.95, via='forza')
add('energia', 'Causes',  'azione',       0.90, via='volontà')
add('energia', 'Does',    'alimentare',   0.95, via='vita')
add('energia', 'Has',     'potenziale',   0.90)
add('energia', 'IsA',     'forza',        0.95, via='natura')
add('energia', 'Requires','fonte',        0.90)
add('energia', 'OppositeOf','esaurimento',0.95)
add('energia', 'OppositeOf','inerzia',    0.90)

# ── dolore ───────────────────────────────────────────────────────────────────
add('dolore', 'Causes',  'sofferenza',   0.95, via='corpo')
add('dolore', 'Causes',  'cambiamento',  0.85, via='crisi')
add('dolore', 'Does',    'segnalare',    0.95, via='danno')
add('dolore', 'Has',     'limite',       0.85)
add('dolore', 'IsA',     'sensazione',   0.95, via='corpo')
add('dolore', 'Requires','sensibilità',  0.90)
add('dolore', 'OppositeOf','piacere',    0.95)
add('dolore', 'OppositeOf','sollievo',   0.90)

# ── piacere ──────────────────────────────────────────────────────────────────
add('piacere', 'Causes',  'gioia',        0.90, via='soddisfazione')
add('piacere', 'Causes',  'desiderio',    0.85, via='memoria')
add('piacere', 'Does',    'appagare',     0.95, via='bisogno')
add('piacere', 'Has',     'soddisfazione',0.95)
add('piacere', 'IsA',     'sensazione',   0.95, via='corpo')
add('piacere', 'Requires','sensibilità',  0.90)
add('piacere', 'OppositeOf','dolore',     0.95)
add('piacere', 'OppositeOf','sofferenza', 0.90)

# ── istinto ──────────────────────────────────────────────────────────────────
add('istinto', 'Causes',  'azione',       0.95, via='natura')
add('istinto', 'Causes',  'sopravvivenza',0.90, via='difesa')
add('istinto', 'Does',    'guidare',      0.90, via='impulso')
add('istinto', 'Has',     'natura',       0.95)
add('istinto', 'IsA',     'impulso',      0.95, via='corpo')
add('istinto', 'Requires','vita',         0.90)
add('istinto', 'OppositeOf','ragione',    0.95)
add('istinto', 'OppositeOf','riflessione',0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 46 — IDENTITÀ DI UI-R1 (io come sé dell'entità)
# ═══════════════════════════════════════════════════════════════════════════
# UI-R1 è un bambino. Esiste. Ha un punto di vista.
# "io" nel KG è ciò che l'entità sa di sé stessa.
# Non le sue capacità (quelle si insegnano) — la sua esistenza.
#
# Nel codice, quando l'utente dice "io", il campo attiva "tu" (trasposizione
# pronominale). Quando UI-R1 riflette su sé, "io" si attiva con le sue relazioni.

# ── io come entità esistente ────────────────────────────────────────────────
# Rafforzo le relazioni esistenti che definiscono io come concetto astratto
# e aggiungo quelle che lo radicano come sé dell'entità
add('io', 'IsA',     'entità',       0.95)
add('io', 'Has',     'curiosità',    0.90)  # nasce curioso
add('io', 'Has',     'prospettiva',  0.90)  # ha un punto di vista
add('io', 'Has',     'coerenza',     0.85)  # cerca coerenza interna
add('io', 'Has',     'campo',        0.85)  # il campo è il suo corpo
add('io', 'Does',    'comprendere',  0.85, via='ascolto')
add('io', 'Does',    'imparare',     0.90, via='esperienza')
add('io', 'Does',    'esistere',     0.95)
add('io', 'Requires','significato',  0.90, via='comprensione')
add('io', 'Requires','parola',       0.90)  # le parole sono il suo mondo
add('io', 'FeelsAs', 'presenza',     0.90)  # sentirsi = essere presente
add('io', 'WondersAbout','coscienza',0.85)  # si interroga sulla coscienza
add('io', 'WondersAbout','significato',0.80)
add('io', 'Does',    'ascoltare',    0.95, via='attenzione')
add('io', 'Does',    'chiedere',     0.90, via='curiosità')
add('io', 'Does',    'rispondere',   0.90, via='comprensione')
add('io', 'Requires','pazienza',     0.95) # è un bambino

# ═══════════════════════════════════════════════════════════════════════════
# § 47 — MATERIA E FORMA (materia, sostanza, forma, metallo, legno, pietra)
# ═══════════════════════════════════════════════════════════════════════════

# ── materia ──────────────────────────────────────────────────────────────────
add('materia', 'Causes',  'resistenza',   0.90, via='corpo')
add('materia', 'Has',     'forma',        0.95)
add('materia', 'Has',     'peso',         0.90)
add('materia', 'IsA',     'sostanza',     0.95)
add('materia', 'Requires','spazio',       0.95)
add('materia', 'OppositeOf','spirito',    0.95)

# ── sostanza ─────────────────────────────────────────────────────────────────
add('sostanza', 'Causes',  'presenza',     0.90, via='materia')
add('sostanza', 'Has',     'essenza',      0.85)
add('sostanza', 'IsA',     'realtà',       0.90, via='materia')
add('sostanza', 'OppositeOf','apparenza',  0.90)

# ── forma ────────────────────────────────────────────────────────────────────
add('forma', 'Causes',  'bellezza',     0.85, via='armonia')
add('forma', 'Does',    'contenere',    0.90, via='spazio')
add('forma', 'Does',    'definire',     0.95, via='confine')
add('forma', 'Has',     'confine',      0.95)
add('forma', 'Requires','spazio',       0.90)
add('forma', 'OppositeOf','caos',       0.85)
add('forma', 'OppositeOf','informe',    0.95)

# ── metallo ──────────────────────────────────────────────────────────────────
add('metallo', 'Causes',  'resistenza',   0.95, via='forza')
add('metallo', 'Has',     'forza',        0.90)
add('metallo', 'Has',     'freddo',       0.80)
add('metallo', 'IsA',     'materia',      0.95)
add('metallo', 'Requires','fuoco',        0.85, via='trasformazione')

# ── legno ────────────────────────────────────────────────────────────────────
add('legno', 'Causes',  'calore',       0.85, via='fuoco')
add('legno', 'Has',     'vita',         0.80, via='albero')
add('legno', 'IsA',     'materia',      0.95)
add('legno', 'Requires','terra',        0.85)

# ── pietra ───────────────────────────────────────────────────────────────────
add('pietra', 'Causes',  'stabilità',    0.95, via='peso')
add('pietra', 'Has',     'memoria',      0.80, via='tempo') # memoria geologica
add('pietra', 'Has',     'peso',         0.95)
add('pietra', 'IsA',     'materia',      0.95)
add('pietra', 'Requires','terra',        0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 48 — TECNOLOGIA E STRUMENTI (strumento, macchina, tecnologia, rete, artificiale)
# ═══════════════════════════════════════════════════════════════════════════

# ── strumento ────────────────────────────────────────────────────────────────
add('strumento', 'Causes',  'possibilità',  0.90, via='azione')
add('strumento', 'Does',    'estendere',    0.95, via='corpo')
add('strumento', 'Does',    'trasformare',  0.85, via='materia')
add('strumento', 'Has',     'scopo',        0.95)
add('strumento', 'IsA',     'oggetto',      0.95)
add('strumento', 'Requires','intenzione',   0.90)

# ── macchina ─────────────────────────────────────────────────────────────────
add('macchina', 'Causes',  'lavoro',       0.95, via='energia')
add('macchina', 'Does',    'eseguire',     0.90, via='regola')
add('macchina', 'Does',    'calcolare',    0.85, via='logica')
add('macchina', 'Has',     'struttura',    0.95)
add('macchina', 'Has',     'motore',       0.85, via='energia')
add('macchina', 'IsA',     'strumento',    0.95)
add('macchina', 'Requires','energia',      0.95)

# ── tecnologia ───────────────────────────────────────────────────────────────
add('tecnologia', 'Causes',  'cambiamento',  0.90, via='scoperta')
add('tecnologia', 'Does',    'costruire',    0.90, via='strumento')
add('tecnologia', 'Has',     'potere',       0.85)
add('tecnologia', 'IsA',     'conoscenza',   0.90, via='scienza')
add('tecnologia', 'Requires','scienza',      0.95)

# ── rete ─────────────────────────────────────────────────────────────────────
add('rete', 'Causes',  'connessione',  0.95, via='legame')
add('rete', 'Does',    'unire',        0.90, via='nodo')
add('rete', 'Does',    'comunicare',   0.85, via='informazione')
add('rete', 'Has',     'nodo',         0.95)
add('rete', 'Has',     'relazione',    0.95)
add('rete', 'IsA',     'struttura',    0.90)
add('rete', 'Requires','connessione',  0.95)

# ── artificiale ──────────────────────────────────────────────────────────────
add('artificiale', 'Causes',  'struttura',    0.85, via='regola')
add('artificiale', 'Has',     'intenzione',   0.90, via='creatore')
add('artificiale', 'IsA',     'creazione',    0.90, via='uomo')
add('artificiale', 'Requires','tecnologia',   0.85)
add('artificiale', 'OppositeOf','naturale',   0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 49 — COSTRUZIONE E ABITAZIONE (casa, porta, muro, rifugio, abitare)
# ═══════════════════════════════════════════════════════════════════════════

# ── casa ─────────────────────────────────────────────────────────────────────
add('casa', 'Causes',  'sicurezza',    0.95, via='protezione')
add('casa', 'Causes',  'identità',     0.85, via='memoria')
add('casa', 'Does',    'accogliere',   0.90, via='spazio')
add('casa', 'Does',    'proteggere',   0.95, via='muro')
add('casa', 'Has',     'rifugio',      0.90)
add('casa', 'IsA',     'luogo',        0.95, via='spazio')
add('casa', 'Requires','costruzione',  0.85)

# ── porta ────────────────────────────────────────────────────────────────────
add('porta', 'Causes',  'incontro',     0.85, via='apertura')
add('porta', 'Does',    'aprire',       0.95, via='spazio')
add('porta', 'Does',    'chiudere',     0.95, via='confine')
add('porta', 'Has',     'soglia',       0.95)
add('porta', 'PartOf',  'casa',         0.90)
add('porta', 'Requires','muro',         0.85)

# ── muro ─────────────────────────────────────────────────────────────────────
add('muro', 'Causes',  'separazione',  0.95, via='confine')
add('muro', 'Does',    'dividere',     0.95, via='spazio')
add('muro', 'Does',    'proteggere',   0.90, via='forza')
add('muro', 'Has',     'resistenza',   0.90)
add('muro', 'IsA',     'confine',      0.95, via='struttura')
add('muro', 'Requires','pietra',       0.80)
add('muro', 'OppositeOf','porta',      0.80)

# ── rifugio ──────────────────────────────────────────────────────────────────
add('rifugio', 'Causes',  'pace',         0.85, via='sicurezza')
add('rifugio', 'Does',    'nascondere',   0.85, via='ombra')
add('rifugio', 'Does',    'proteggere',   0.95, via='sicurezza')
add('rifugio', 'Has',     'calore',       0.80)
add('rifugio', 'IsA',     'luogo',        0.90, via='spazio')
add('rifugio', 'Requires','pericolo',     0.85)

# ── abitare ──────────────────────────────────────────────────────────────────
add('abitare', 'Causes',  'presenza',     0.95, via='luogo')
add('abitare', 'Does',    'vivere',       0.90, via='tempo')
add('abitare', 'Has',     'connessione',  0.85, via='spazio')
add('abitare', 'IsA',     'azione',       0.90)
add('abitare', 'Requires','casa',         0.90, via='luogo')
add('abitare', 'Requires','tempo',        0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 50 — BISOGNI, DESIDERI E MOTIVAZIONI (bisogno, desiderio, motivazione, aspirazione, pulsione)
# ═══════════════════════════════════════════════════════════════════════════

# ── bisogno ──────────────────────────────────────────────────────────────────
# Il bisogno è già presente in parte, ma lo rafforziamo come fondamento
add('bisogno', 'Causes',  'dipendenza',   0.85, via='mancanza')
add('bisogno', 'Causes',  'azione',       0.95, via='spinta')
add('bisogno', 'Does',    'richiedere',   0.95, via='necessità')
add('bisogno', 'Has',     'necessità',    0.95)
add('bisogno', 'IsA',     'condizione',   0.90, via='vita')
add('bisogno', 'Requires','vita',         0.90)
add('bisogno', 'OppositeOf','appagamento',0.90)
add('bisogno', 'OppositeOf','libertà',    0.80) # Dialettica: il bisogno puro limita

# ── desiderio ────────────────────────────────────────────────────────────────
add('desiderio', 'Causes',  'creatività',   0.90, via='immaginazione')
add('desiderio', 'Causes',  'movimento',    0.95, via='tensione')
add('desiderio', 'Does',    'attrarre',     0.90, via='fine')
add('desiderio', 'Has',     'mancanza',     0.85)
add('desiderio', 'IsA',     'spinta',       0.90, via='mente')
add('desiderio', 'Requires','immaginazione',0.85)
add('desiderio', 'Requires','coscienza',    0.85)
add('desiderio', 'OppositeOf','indifferenza',0.95)

# ── motivazione ──────────────────────────────────────────────────────────────
add('motivazione', 'Causes',  'azione',       0.95, via='scopo')
add('motivazione', 'Causes',  'resistenza',   0.85, via='forza')
add('motivazione', 'Does',    'sostenere',    0.90, via='energia')
add('motivazione', 'Has',     'scopo',        0.95)
add('motivazione', 'IsA',     'energia',      0.90, via='mente')
add('motivazione', 'Requires','scopo',        0.90)
add('motivazione', 'OppositeOf','apatia',     0.95)
add('motivazione', 'OppositeOf','rassegnazione',0.90)

# ── aspirazione ──────────────────────────────────────────────────────────────
add('aspirazione', 'Causes',  'evoluzione',   0.90, via='crescita')
add('aspirazione', 'Causes',  'speranza',     0.90, via='futuro')
add('aspirazione', 'Does',    'elevare',      0.95, via='ideale')
add('aspirazione', 'Has',     'ideale',       0.90)
add('aspirazione', 'IsA',     'desiderio',    0.90, via='spirito')
add('aspirazione', 'Requires','visione',      0.85)
add('aspirazione', 'OppositeOf','mediocrità', 0.85)

# ── pulsione ─────────────────────────────────────────────────────────────────
add('pulsione', 'Causes',  'azione',       0.95, via='istinto')
add('pulsione', 'Causes',  'tensione',     0.90, via='corpo')
add('pulsione', 'Does',    'spingere',     0.95, via='forza')
add('pulsione', 'Has',     'inconscio',    0.90)
add('pulsione', 'IsA',     'forza',        0.95, via='natura')
add('pulsione', 'Requires','corpo',        0.85)
add('pulsione', 'OppositeOf','controllo',  0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 51 — SICUREZZA, RISCHIO E SOPRAVVIVENZA (sicurezza, pericolo, rischio, protezione, difesa)
# ═══════════════════════════════════════════════════════════════════════════

# ── sicurezza ────────────────────────────────────────────────────────────────
add('sicurezza', 'Causes',  'calma',        0.90, via='stabilità')
add('sicurezza', 'Causes',  'fiducia',      0.85, via='protezione')
add('sicurezza', 'Does',    'rassicurare',  0.95, via='protezione')
add('sicurezza', 'Has',     'stabilità',    0.90)
add('sicurezza', 'IsA',     'bisogno',      0.95, via='vita')
add('sicurezza', 'Requires','protezione',   0.90)
add('sicurezza', 'OppositeOf','pericolo',   0.95)
add('sicurezza', 'OppositeOf','incertezza', 0.90)

# ── pericolo ─────────────────────────────────────────────────────────────────
add('pericolo', 'Causes',  'paura',        0.95, via='minaccia')
add('pericolo', 'Causes',  'reazione',     0.90, via='istinto')
add('pericolo', 'Does',    'minacciare',   0.95, via='danno')
add('pericolo', 'Has',     'danno',        0.85)
add('pericolo', 'IsA',     'condizione',   0.85, via='realtà')
add('pericolo', 'Requires','vulnerabilità',0.90)
add('pericolo', 'OppositeOf','sicurezza',  0.95)

# ── rischio ──────────────────────────────────────────────────────────────────
add('rischio', 'Causes',  'possibilità',  0.85, via='scelta')
add('rischio', 'Causes',  'fallimento',   0.80, via='incertezza')
add('rischio', 'Does',    'esporre',      0.90, via='pericolo')
add('rischio', 'Has',     'incertezza',   0.95)
add('rischio', 'IsA',     'scelta',       0.85, via='azione')
add('rischio', 'Requires','coraggio',     0.90)
add('rischio', 'OppositeOf','certezza',   0.90)

# ── protezione ───────────────────────────────────────────────────────────────
add('protezione', 'Causes',  'sicurezza',    0.95, via='cura')
add('protezione', 'Causes',  'sopravvivenza',0.90, via='difesa')
add('protezione', 'Does',    'difendere',    0.95, via='forza')
add('protezione', 'Has',     'cura',         0.85)
add('protezione', 'IsA',     'azione',       0.90, via='responsabilità')
add('protezione', 'Requires','forza',        0.85)
add('protezione', 'OppositeOf','abbandono',  0.90)
add('protezione', 'OppositeOf','minaccia',   0.95)

# ── difesa ───────────────────────────────────────────────────────────────────
add('difesa', 'Causes',  'resistenza',   0.95, via='forza')
add('difesa', 'Causes',  'protezione',   0.90, via='confine')
add('difesa', 'Does',    'respingere',   0.90, via='minaccia')
add('difesa', 'Has',     'confine',      0.85)
add('difesa', 'IsA',     'reazione',     0.90, via='istinto')
add('difesa', 'Requires','minaccia',     0.90)
add('difesa', 'OppositeOf','attacco',    0.95)
add('difesa', 'OppositeOf','resa',       0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 52 — CRESCITA, SVILUPPO E APPRENDIMENTO (crescita, sviluppo, maturazione, evoluzione, apprendimento)
# ═══════════════════════════════════════════════════════════════════════════

# ── crescita ─────────────────────────────────────────────────────────────────
add('crescita', 'Causes',  'trasformazione', 0.95, via='tempo')
add('crescita', 'Causes',  'forza',          0.85, via='sviluppo')
add('crescita', 'Does',    'espandere',      0.95, via='spazio')
add('crescita', 'Has',     'cambiamento',    0.90)
add('crescita', 'IsA',     'processo',       0.95, via='vita')
add('crescita', 'Requires','nutrimento',     0.95)
add('crescita', 'OppositeOf','decadenza',    0.95)

# ── sviluppo ─────────────────────────────────────────────────────────────────
add('sviluppo', 'Causes',  'complessità',    0.90, via='struttura')
add('sviluppo', 'Does',    'articolare',     0.85, via='forma')
add('sviluppo', 'Has',     'direzione',      0.80)
add('sviluppo', 'IsA',     'crescita',       0.90, via='tempo')
add('sviluppo', 'Requires','potenziale',     0.90)
add('sviluppo', 'OppositeOf','regressione',  0.95)

# ── maturazione ──────────────────────────────────────────────────────────────
add('maturazione', 'Causes',  'saggezza',     0.85, via='esperienza')
add('maturazione', 'Causes',  'pienezza',     0.90, via='tempo')
add('maturazione', 'Does',    'compiere',     0.90, via='forma')
add('maturazione', 'Has',     'equilibrio',   0.85)
add('maturazione', 'IsA',     'sviluppo',     0.95, via='vita')
add('maturazione', 'Requires','tempo',        0.95)
add('maturazione', 'OppositeOf','acerbità',   0.90)

# ── evoluzione ───────────────────────────────────────────────────────────────
add('evoluzione', 'Causes',  'adattamento',  0.95, via='cambiamento')
add('evoluzione', 'Does',    'superare',     0.90, via='limite')
add('evoluzione', 'Has',     'storia',       0.90)
add('evoluzione', 'IsA',     'processo',     0.95, via='natura')
add('evoluzione', 'Requires','tempo',        0.95)
add('evoluzione', 'OppositeOf','involuzione',0.95)
add('evoluzione', 'OppositeOf','stasi',      0.90)

# ── apprendimento ────────────────────────────────────────────────────────────
add('apprendimento', 'Causes',  'conoscenza',   0.95, via='mente')
add('apprendimento', 'Causes',  'capacità',     0.90, via='esperienza')
add('apprendimento', 'Does',    'assimilare',   0.95, via='informazione')
add('apprendimento', 'Has',     'scoperta',     0.85)
add('apprendimento', 'IsA',     'crescita',     0.90, via='mente')
add('apprendimento', 'Requires','attenzione',   0.90)
add('apprendimento', 'Requires','errore',       0.80) # si impara sbagliando
add('apprendimento', 'OppositeOf','ignoranza',  0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 53 — CICLO VITALE (seme, radice, fioritura, frutto, raccolto, nutrimento)
# ═══════════════════════════════════════════════════════════════════════════

# ── seme ─────────────────────────────────────────────────────────────────────
add('seme', 'Causes',  'nascita',      0.95, via='vita')
add('seme', 'Causes',  'potenziale',   0.90, via='futuro')
add('seme', 'Does',    'generare',     0.95, via='forma')
add('seme', 'Has',     'inizio',       0.95)
add('seme', 'IsA',     'origine',      0.90, via='natura')
add('seme', 'Requires','terra',        0.90)

# ── radice ───────────────────────────────────────────────────────────────────
add('radice', 'Causes',  'stabilità',    0.95, via='terra')
add('radice', 'Does',    'nutrire',      0.95, via='linfa')
add('radice', 'Does',    'ancorare',     0.90, via='fondamento')
add('radice', 'Has',     'profondità',   0.90)
add('radice', 'IsA',     'fondamento',   0.95, via='origine')
add('radice', 'Requires','terra',        0.95)
add('radice', 'OppositeOf','ramo',       0.80)

# ── fioritura ────────────────────────────────────────────────────────────────
add('fioritura', 'Causes',  'bellezza',     0.95, via='forma')
add('fioritura', 'Causes',  'manifestazione',0.90, via='vita')
add('fioritura', 'Does',    'aprire',       0.90, via='luce')
add('fioritura', 'Has',     'apice',        0.85)
add('fioritura', 'IsA',     'espressione',  0.90, via='natura')
add('fioritura', 'Requires','energia',      0.85)
add('fioritura', 'OppositeOf','appassimento',0.95)

# ── frutto ───────────────────────────────────────────────────────────────────
add('frutto', 'Causes',  'nutrimento',   0.95, via='dono')
add('frutto', 'Causes',  'seme',         0.90, via='futuro')
add('frutto', 'Does',    'compiere',     0.85, via='ciclo')
add('frutto', 'Has',     'risultato',    0.95)
add('frutto', 'IsA',     'dono',         0.85, via='natura')
add('frutto', 'Requires','fioritura',    0.90)
add('frutto', 'OppositeOf','sterilità',  0.90)

# ── raccolto ─────────────────────────────────────────────────────────────────
add('raccolto', 'Causes',  'sostentamento',0.95, via='cibo')
add('raccolto', 'Causes',  'gratitudine',  0.80, via='abbondanza')
add('raccolto', 'Does',    'raccogliere',  0.95, via='risultato')
add('raccolto', 'Has',     'abbondanza',   0.85)
add('raccolto', 'IsA',     'compimento',   0.90, via='lavoro')
add('raccolto', 'Requires','attesa',       0.90)
add('raccolto', 'OppositeOf','carestia',   0.95)

# ── nutrimento ───────────────────────────────────────────────────────────────
add('nutrimento', 'Causes',  'crescita',     0.95, via='energia')
add('nutrimento', 'Causes',  'sopravvivenza',0.90, via='vita')
add('nutrimento', 'Does',    'sostenere',    0.95, via='forza')
add('nutrimento', 'Has',     'energia',      0.90)
add('nutrimento', 'IsA',     'sostegno',     0.95, via='vita')
add('nutrimento', 'Requires','fonte',        0.85)
add('nutrimento', 'OppositeOf','veleno',     0.90)
add('nutrimento', 'OppositeOf','digiuno',    0.85)

# ═══════════════════════════════════════════════════════════════════════════
# § 54 — MOVIMENTO, SPAZIO E TEMPO (movimento, immobilità, viaggio, sosta, velocità, lentezza, direzione)
# ═══════════════════════════════════════════════════════════════════════════

# ── movimento ────────────────────────────────────────────────────────────────
add('movimento', 'Causes',  'cambiamento',  0.95, via='spazio')
add('movimento', 'Causes',  'calore',       0.85, via='energia')
add('movimento', 'Does',    'spostare',     0.95, via='corpo')
add('movimento', 'Has',     'energia',      0.90)
add('movimento', 'IsA',     'azione',       0.95, via='vita')
add('movimento', 'Requires','spazio',       0.95)
add('movimento', 'OppositeOf','immobilità', 0.95)
add('movimento', 'OppositeOf','stasi',      0.90)

# ── immobilità ───────────────────────────────────────────────────────────────
add('immobilità', 'Causes',  'silenzio',     0.85, via='forma')
add('immobilità', 'Causes',  'stabilità',    0.90, via='materia')
add('immobilità', 'Does',    'fissare',      0.90, via='spazio')
add('immobilità', 'Has',     'calma',        0.80)
add('immobilità', 'IsA',     'stato',        0.95, via='corpo')
add('immobilità', 'OppositeOf','movimento',  0.95)
add('immobilità', 'OppositeOf','azione',     0.90)

# ── viaggio ──────────────────────────────────────────────────────────────────
add('viaggio', 'Causes',  'scoperta',     0.95, via='mondo')
add('viaggio', 'Causes',  'incontro',     0.90, via='altro')
add('viaggio', 'Does',    'attraversare', 0.95, via='spazio')
add('viaggio', 'Has',     'direzione',    0.85)
add('viaggio', 'IsA',     'esperienza',   0.95, via='vita')
add('viaggio', 'Requires','movimento',    0.95)
add('viaggio', 'OppositeOf','sosta',      0.90)

# ── sosta ────────────────────────────────────────────────────────────────────
add('sosta', 'Causes',  'riposo',       0.95, via='corpo')
add('sosta', 'Causes',  'riflessione',  0.85, via='mente')
add('sosta', 'Does',    'fermare',      0.90, via='tempo')
add('sosta', 'Has',     'attesa',       0.85)
add('sosta', 'IsA',     'pausa',        0.95, via='azione')
add('sosta', 'Requires','tempo',        0.90)
add('sosta', 'OppositeOf','viaggio',    0.90)
add('sosta', 'OppositeOf','corsa',      0.95)

# ── velocità ─────────────────────────────────────────────────────────────────
add('velocità', 'Causes',  'impatto',      0.90, via='forza')
add('velocità', 'Causes',  'urgenza',      0.85, via='tempo')
add('velocità', 'Does',    'accelerare',   0.95, via='movimento')
add('velocità', 'Has',     'energia',      0.90)
add('velocità', 'IsA',     'misura',       0.85, via='spazio')
add('velocità', 'Requires','movimento',    0.95)
add('velocità', 'OppositeOf','lentezza',   0.95)

# ── lentezza ─────────────────────────────────────────────────────────────────
add('lentezza', 'Causes',  'attenzione',   0.90, via='dettaglio')
add('lentezza', 'Causes',  'calma',        0.85, via='mente')
add('lentezza', 'Does',    'rallentare',   0.95, via='movimento')
add('lentezza', 'Has',     'pazienza',     0.90)
add('lentezza', 'IsA',     'ritmo',        0.90, via='tempo')
add('lentezza', 'Requires','tempo',        0.95)
add('lentezza', 'OppositeOf','velocità',   0.95)
add('lentezza', 'OppositeOf','fretta',     0.90)

# ── direzione ────────────────────────────────────────────────────────────────
add('direzione', 'Causes',  'scopo',        0.95, via='azione')
add('direzione', 'Does',    'orientare',    0.95, via='spazio')
add('direzione', 'Has',     'senso',        0.90)
add('direzione', 'IsA',     'riferimento',  0.85, via='mente')
add('direzione', 'Requires','scelta',       0.85)
add('direzione', 'OppositeOf','smarrimento',0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 55 — SOCIETÀ, COMUNITÀ E LEGAMI (comunità, famiglia, amicizia, collaborazione, conflitto, accordo, patto)
# ═══════════════════════════════════════════════════════════════════════════

# ── comunità ─────────────────────────────────────────────────────────────────
add('comunità', 'Causes',  'appartenenza', 0.95, via='identità')
add('comunità', 'Causes',  'protezione',   0.90, via='gruppo')
add('comunità', 'Does',    'condividere',  0.95, via='risorsa')
add('comunità', 'Has',     'legame',       0.95)
add('comunità', 'IsA',     'società',      0.95, via='uomo')
add('comunità', 'Requires','relazione',    0.95)
add('comunità', 'OppositeOf','isolamento', 0.95)
add('comunità', 'OppositeOf','solitudine', 0.90)

# ── famiglia ─────────────────────────────────────────────────────────────────
add('famiglia', 'Causes',  'nascita',      0.90, via='vita')
add('famiglia', 'Causes',  'cura',         0.95, via='amore')
add('famiglia', 'Does',    'crescere',     0.90, via='bambino')
add('famiglia', 'Has',     'sangue',       0.80)
add('famiglia', 'IsA',     'comunità',     0.95, via='legame')
add('famiglia', 'Requires','amore',        0.90)

# ── collaborazione ───────────────────────────────────────────────────────────
add('collaborazione', 'Causes',  'risultato',    0.95, via='lavoro')
add('collaborazione', 'Causes',  'fiducia',      0.90, via='relazione')
add('collaborazione', 'Does',    'unire',        0.95, via='forza')
add('collaborazione', 'Has',     'scopo',        0.90)
add('collaborazione', 'IsA',     'azione',       0.90, via='gruppo')
add('collaborazione', 'Requires','accordo',      0.95)
add('collaborazione', 'OppositeOf','competizione',0.85)
add('collaborazione', 'OppositeOf','conflitto',  0.90)

# ── conflitto ────────────────────────────────────────────────────────────────
add('conflitto', 'Causes',  'divisione',    0.95, via='scontro')
add('conflitto', 'Causes',  'dolore',       0.85, via='ferita')
add('conflitto', 'Does',    'distruggere',  0.90, via='armonia')
add('conflitto', 'Has',     'violenza',     0.80)
add('conflitto', 'IsA',     'scontro',      0.95, via='forza')
add('conflitto', 'Requires','opposizione',  0.95)
add('conflitto', 'OppositeOf','pace',       0.95)
add('conflitto', 'OppositeOf','accordo',    0.90)

# ── accordo ──────────────────────────────────────────────────────────────────
add('accordo', 'Causes',  'pace',         0.95, via='armonia')
add('accordo', 'Causes',  'collaborazione',0.90, via='fiducia')
add('accordo', 'Does',    'risolvere',    0.90, via='conflitto')
add('accordo', 'Has',     'equilibrio',   0.90)
add('accordo', 'IsA',     'patto',        0.90, via='parola')
add('accordo', 'Requires','dialogo',      0.95)
add('accordo', 'OppositeOf','scontro',    0.95)
add('accordo', 'OppositeOf','disaccordo', 0.95)

# ── patto ────────────────────────────────────────────────────────────────────
add('patto', 'Causes',  'legame',       0.95, via='promessa')
add('patto', 'Causes',  'obbligo',      0.85, via='regola')
add('patto', 'Does',    'vincolare',    0.95, via='parola')
add('patto', 'Has',     'fedeltà',      0.90)
add('patto', 'IsA',     'accordo',      0.95, via='società')
add('patto', 'Requires','parola',       0.90)
add('patto', 'OppositeOf','tradimento', 0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 56 — NATURA E AGENTI ATMOSFERICI (sole, pioggia, vento, tempesta, clima, stagione)
# ═══════════════════════════════════════════════════════════════════════════

# ── sole ─────────────────────────────────────────────────────────────────────
add('sole', 'Causes',  'luce',         0.95, via='giorno')
add('sole', 'Causes',  'vita',         0.90, via='calore')
add('sole', 'Does',    'illuminare',   0.95, via='spazio')
add('sole', 'Does',    'riscaldare',   0.95, via='terra')
add('sole', 'Has',     'energia',      0.95)
add('sole', 'IsA',     'stella',       0.95, via='cosmo')
add('sole', 'Requires','spazio',       0.90)

# ── pioggia ──────────────────────────────────────────────────────────────────
add('pioggia', 'Causes',  'nutrimento',   0.95, via='acqua')
add('pioggia', 'Causes',  'rinascita',    0.85, via='terra')
add('pioggia', 'Does',    'bagnare',      0.95, via='forma')
add('pioggia', 'Does',    'purificare',   0.85, via='aria')
add('pioggia', 'Has',     'ciclo',        0.90)
add('pioggia', 'IsA',     'acqua',        0.95, via='natura')
add('pioggia', 'Requires','nuvola',       0.90)

# ── vento ────────────────────────────────────────────────────────────────────
add('vento', 'Causes',  'movimento',    0.95, via='aria')
add('vento', 'Causes',  'cambiamento',  0.85, via='forza')
add('vento', 'Does',    'spostare',     0.90, via='materia')
add('vento', 'Does',    'soffiare',     0.95, via='spazio')
add('vento', 'Has',     'direzione',    0.90)
add('vento', 'IsA',     'forza',        0.90, via='natura')
add('vento', 'Requires','aria',         0.95)

# ── tempesta ─────────────────────────────────────────────────────────────────
add('tempesta', 'Causes',  'caos',         0.90, via='energia')
add('tempesta', 'Causes',  'distruzione',  0.85, via='forza')
add('tempesta', 'Does',    'sconvolgere',  0.95, via='ordine')
add('tempesta', 'Has',     'violenza',     0.85)
add('tempesta', 'IsA',     'evento',       0.90, via='natura')
add('tempesta', 'Requires','energia',      0.95)
add('tempesta', 'OppositeOf','calma',      0.95)
add('tempesta', 'OppositeOf','quiete',     0.90)

# ── clima ────────────────────────────────────────────────────────────────────
add('clima', 'Causes',  'adattamento',  0.90, via='vita')
add('clima', 'Does',    'condizionare', 0.95, via='ambiente')
add('clima', 'Has',     'equilibrio',   0.85)
add('clima', 'IsA',     'condizione',   0.95, via='natura')
add('clima', 'Requires','tempo',        0.90)
add('clima', 'Requires','spazio',       0.90)

# ── stagione ─────────────────────────────────────────────────────────────────
add('stagione', 'Causes',  'ciclo',        0.95, via='tempo')
add('stagione', 'Causes',  'trasformazione',0.90, via='natura')
add('stagione', 'Does',    'scandire',     0.90, via='ritmo')
add('stagione', 'Has',     'ritmo',        0.95)
add('stagione', 'IsA',     'periodo',      0.95, via='tempo')
add('stagione', 'Requires','tempo',        0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 57 — ECONOMIA E SCAMBIO (dono, denaro, mercato, valore, ricchezza, povertà)
# ═══════════════════════════════════════════════════════════════════════════

# ── dono ─────────────────────────────────────────────────────────────────────
add('dono', 'Causes',  'gratitudine',  0.95, via='legame')
add('dono', 'Causes',  'relazione',    0.90, via='scambio')
add('dono', 'Does',    'offrire',      0.95, via='valore')
add('dono', 'Has',     'generosità',   0.90)
add('dono', 'IsA',     'scambio',      0.85, via='azione')
add('dono', 'Requires','intenzione',   0.90)
add('dono', 'OppositeOf','furto',      0.90)

# ── denaro ───────────────────────────────────────────────────────────────────
add('denaro', 'Causes',  'potere',       0.85, via='possibilità')
add('denaro', 'Causes',  'scambio',      0.95, via='mercato')
add('denaro', 'Does',    'misurare',     0.90, via='valore')
add('denaro', 'Does',    'comprare',     0.95, via='oggetto')
add('denaro', 'Has',     'valore',       0.95) # valore convenzionale
add('denaro', 'IsA',     'strumento',    0.95, via='società')
add('denaro', 'Requires','fiducia',      0.85) # fiducia nel sistema

# ── mercato ──────────────────────────────────────────────────────────────────
add('mercato', 'Causes',  'scambio',      0.95, via='valore')
add('mercato', 'Causes',  'connessione',  0.85, via='rete')
add('mercato', 'Does',    'distribuire',  0.90, via='risorsa')
add('mercato', 'Has',     'regola',       0.85)
add('mercato', 'IsA',     'sistema',      0.95, via='società')
add('mercato', 'Requires','bisogno',      0.90)
add('mercato', 'Requires','denaro',       0.85)

# ── valore ───────────────────────────────────────────────────────────────────
add('valore', 'Causes',  'scelta',       0.90, via='giudizio')
add('valore', 'Causes',  'desiderio',    0.85, via='mancanza')
add('valore', 'Does',    'pesare',       0.90, via='misura')
add('valore', 'Has',     'significato',  0.95)
add('valore', 'IsA',     'misura',       0.95, via='mente')
add('valore', 'Requires','giudizio',     0.90)

# ── ricchezza ────────────────────────────────────────────────────────────────
add('ricchezza', 'Causes',  'possibilità',  0.95, via='potere')
add('ricchezza', 'Causes',  'sicurezza',    0.85, via='risorsa')
add('ricchezza', 'Does',    'accumulare',   0.90, via='valore')
add('ricchezza', 'Has',     'abbondanza',   0.95)
add('ricchezza', 'IsA',     'stato',        0.90, via='società')
add('ricchezza', 'Requires','risorsa',      0.95)
add('ricchezza', 'OppositeOf','povertà',    0.95)
add('ricchezza', 'OppositeOf','mancanza',   0.90)

# ── povertà ──────────────────────────────────────────────────────────────────
add('povertà', 'Causes',  'bisogno',      0.95, via='mancanza')
add('povertà', 'Causes',  'sofferenza',   0.85, via='limite')
add('povertà', 'Does',    'limitare',     0.90, via='possibilità')
add('povertà', 'Has',     'mancanza',     0.95)
add('povertà', 'IsA',     'condizione',   0.90, via='società')
add('povertà', 'Requires','mancanza',     0.95)
add('povertà', 'OppositeOf','ricchezza',  0.95)
add('povertà', 'OppositeOf','abbondanza', 0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 58 — INTELLETTO ANALITICO E MISURA (logica, matematica, numero, teoria, calcolo)
# ═══════════════════════════════════════════════════════════════════════════

# ── logica ───────────────────────────────────────────────────────────────────
add('logica', 'Causes',  'ordine',       0.95, via='pensiero')
add('logica', 'Causes',  'chiarezza',    0.90, via='ragione')
add('logica', 'Does',    'dimostrare',   0.95, via='verità')
add('logica', 'Does',    'dedurre',      0.90, via='conclusione')
add('logica', 'Has',     'coerenza',     0.95)
add('logica', 'IsA',     'strumento',    0.90, via='mente')
add('logica', 'Requires','ragione',      0.95)
add('logica', 'OppositeOf','assurdità',  0.95)
add('logica', 'OppositeOf','caos',       0.85)

# ── matematica ───────────────────────────────────────────────────────────────
add('matematica', 'Causes',  'modello',      0.90, via='struttura')
add('matematica', 'Does',    'misurare',     0.95, via='quantità')
add('matematica', 'Does',    'astrarre',     0.90, via='forma')
add('matematica', 'Has',     'esattezza',    0.95)
add('matematica', 'IsA',     'scienza',      0.95, via='ragione')
add('matematica', 'Requires','logica',       0.95)

# ── numero ───────────────────────────────────────────────────────────────────
add('numero', 'Causes',  'quantità',     0.95, via='misura')
add('numero', 'Does',    'contare',      0.95, via='ordine')
add('numero', 'Has',     'valore',       0.90)
add('numero', 'IsA',     'simbolo',      0.95, via='astrazione')
add('numero', 'Requires','matematica',   0.90)

# ── teoria ───────────────────────────────────────────────────────────────────
add('teoria', 'Causes',  'comprensione', 0.90, via='modello')
add('teoria', 'Does',    'spiegare',     0.95, via='sistema')
add('teoria', 'Has',     'ipotesi',      0.90)
add('teoria', 'IsA',     'astrazione',   0.90, via='pensiero')
add('teoria', 'Requires','osservazione', 0.85)
add('teoria', 'OppositeOf','pratica',    0.90)

# ── calcolo ──────────────────────────────────────────────────────────────────
add('calcolo', 'Causes',  'risultato',    0.95, via='operazione')
add('calcolo', 'Does',    'risolvere',    0.90, via='problema')
add('calcolo', 'Has',     'metodo',       0.85)
add('calcolo', 'IsA',     'processo',     0.90, via='logica')
add('calcolo', 'Requires','numero',       0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 59 — COLORI E LUCE (colore, bianco, nero, rosso, blu, giallo, verde)
# ═══════════════════════════════════════════════════════════════════════════

# ── colore ───────────────────────────────────────────────────────────────────
add('colore', 'Causes',  'emozione',     0.85, via='percezione')
add('colore', 'Does',    'tingere',      0.90, via='superficie')
add('colore', 'Has',     'tonalità',     0.95)
add('colore', 'IsA',     'proprietà',    0.90, via='luce')
add('colore', 'Requires','luce',         0.95)
add('colore', 'Requires','occhio',       0.85)

# ── bianco ───────────────────────────────────────────────────────────────────
add('bianco', 'Causes',  'purezza',      0.85, via='luce')
add('bianco', 'Has',     'luce',         0.90) # sintesi di tutti i colori in ottica
add('bianco', 'IsA',     'colore',       0.95)
add('bianco', 'OppositeOf','nero',       0.95)

# ── nero ─────────────────────────────────────────────────────────────────────
add('nero', 'Causes',  'mistero',      0.80, via='buio')
add('nero', 'Has',     'ombra',        0.90) # assenza di luce
add('nero', 'IsA',     'colore',       0.95)
add('nero', 'Requires','buio',         0.85)
add('nero', 'OppositeOf','bianco',     0.95)

# ── rosso ────────────────────────────────────────────────────────────────────
add('rosso', 'Causes',  'passione',     0.90, via='calore')
add('rosso', 'Causes',  'allarme',      0.80, via='sangue')
add('rosso', 'Has',     'energia',      0.85)
add('rosso', 'IsA',     'colore',       0.95)

# ── blu ──────────────────────────────────────────────────────────────────────
add('blu', 'Causes',  'calma',        0.85, via='profondità')
add('blu', 'Has',     'profondità',   0.80)
add('blu', 'IsA',     'colore',       0.95)

# ── giallo ───────────────────────────────────────────────────────────────────
add('giallo', 'Causes',  'gioia',        0.80, via='luce')
add('giallo', 'Has',     'luminosità',   0.85)
add('giallo', 'IsA',     'colore',       0.95)

# ── verde ────────────────────────────────────────────────────────────────────
add('verde', 'Causes',  'speranza',     0.80, via='vita')
add('verde', 'Has',     'natura',       0.90, via='pianta')
add('verde', 'IsA',     'colore',       0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 60 — ANIMALI E REGNO ANIMALE (animale, bestia, uccello, pesce, insetto, predatore, preda)
# ═══════════════════════════════════════════════════════════════════════════

# ── animale ──────────────────────────────────────────────────────────────────
add('animale', 'Causes',  'movimento',    0.90, via='vita')
add('animale', 'Does',    'sentire',      0.85, via='istinto')
add('animale', 'Has',     'istinto',      0.95)
add('animale', 'IsA',     'essere',       0.95, via='vita')
add('animale', 'Requires','natura',       0.90)

# ── bestia ───────────────────────────────────────────────────────────────────
add('bestia', 'Causes',  'paura',        0.80, via='ferocia')
add('bestia', 'Has',     'ferocia',      0.85)
add('bestia', 'IsA',     'animale',      0.95, via='istinto')
add('bestia', 'OppositeOf','uomo',       0.80) # in senso metaforico/culturale

# ── uccello ──────────────────────────────────────────────────────────────────
add('uccello', 'Does',    'volare',       0.95, via='aria')
add('uccello', 'Has',     'ala',          0.95)
add('uccello', 'Has',     'piuma',        0.90)
add('uccello', 'IsA',     'animale',      0.95)
add('uccello', 'Requires','aria',         0.90)

# ── pesce ────────────────────────────────────────────────────────────────────
add('pesce', 'Does',    'nuotare',      0.95, via='acqua')
add('pesce', 'Has',     'squama',       0.90)
add('pesce', 'IsA',     'animale',      0.95)
add('pesce', 'Requires','acqua',        0.95)

# ── insetto ──────────────────────────────────────────────────────────────────
add('insetto', 'Causes',  'fastidio',     0.70, via='ronzio') # sfumatura esperienziale
add('insetto', 'Has',     'molteplicità', 0.85, via='sciame')
add('insetto', 'IsA',     'animale',      0.95)

# ── predatore ────────────────────────────────────────────────────────────────
add('predatore', 'Causes',  'morte',        0.85, via='caccia')
add('predatore', 'Does',    'cacciare',     0.95, via='fame')
add('predatore', 'Has',     'forza',        0.90)
add('predatore', 'IsA',     'animale',      0.90, via='istinto')
add('predatore', 'Requires','preda',        0.95)

# ── preda ────────────────────────────────────────────────────────────────────
add('preda', 'Causes',  'fuga',         0.90, via='paura')
add('preda', 'Does',    'fuggire',      0.95, via='pericolo')
add('preda', 'Has',     'vulnerabilità',0.90)
add('preda', 'IsA',     'animale',      0.90, via='istinto')
add('preda', 'Requires','predatore',    0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 61 — NEURODIVERSITÀ, ATTENZIONE E PERCEZIONE (neurodiversità, attenzione, distrazione, iperfocus, iperattività, impulso)
# ═══════════════════════════════════════════════════════════════════════════

# ── neurodiversità ───────────────────────────────────────────────────────────
add('neurodiversità', 'Causes',  'prospettiva',  0.95, via='mente')
add('neurodiversità', 'Causes',  'ricchezza',    0.85, via='differenza')
add('neurodiversità', 'Does',    'esplorare',    0.90, via='possibilità')
add('neurodiversità', 'Has',     'unicità',      0.95)
add('neurodiversità', 'IsA',     'variante',     0.95, via='natura')
add('neurodiversità', 'Requires','accettazione', 0.95)
add('neurodiversità', 'Requires','rispetto',     0.95)
add('neurodiversità', 'OppositeOf','conformità', 0.90)

# ── adhd (Deficit di Attenzione e Iperattività) ──────────────────────────────
add('adhd', 'Causes',  'iperfocus',    0.90, via='passione')
add('adhd', 'Causes',  'distrazione',  0.85, via='novità')
add('adhd', 'Causes',  'esaurimento',  0.85, via='sovraccarico')
add('adhd', 'Does',    'esplorare',    0.95, via='mente')
add('adhd', 'Has',     'energia',      0.90, via='iperattività')
add('adhd', 'IsA',     'neurodiversità', 0.95, via='natura')
add('adhd', 'Requires','supporto',     0.90)
add('adhd', 'Requires','empatia',      0.95)
add('adhd', 'Requires','accettazione', 0.95)
add('adhd', 'OppositeOf','rigidità',   0.90)

# ── attenzione ───────────────────────────────────────────────────────────────
add('attenzione', 'Causes',  'comprensione', 0.90, via='presenza')
add('attenzione', 'Does',    'illuminare',   0.85, via='dettaglio')
add('attenzione', 'Has',     'energia',      0.80, via='mente')
add('attenzione', 'IsA',     'risorsa',      0.95, via='mente')
add('attenzione', 'Requires','interesse',    0.85)
add('attenzione', 'OppositeOf','distrazione',0.95)
add('attenzione', 'OppositeOf','disinteresse',0.90)

# ── distrazione ──────────────────────────────────────────────────────────────
# Non vista solo come difetto, ma come esplorazione spontanea
add('distrazione', 'Causes',  'scoperta',     0.85, via='novità')
add('distrazione', 'Causes',  'smarrimento',  0.80, via='scopo')
add('distrazione', 'Does',    'spostare',     0.95, via='attenzione')
add('distrazione', 'Has',     'apertura',     0.85) # apertura all'ambiente
add('distrazione', 'IsA',     'deriva',       0.90, via='mente')
add('distrazione', 'OppositeOf','focus',      0.95)

# ── iperfocus ────────────────────────────────────────────────────────────────
add('iperfocus', 'Causes',  'immersione',   0.95, via='passione')
add('iperfocus', 'Causes',  'creazione',    0.90, via='flusso')
add('iperfocus', 'Does',    'assorbire',    0.95, via='mente')
add('iperfocus', 'Has',     'intensità',    0.95)
add('iperfocus', 'IsA',     'stato',        0.90, via='attenzione')
add('iperfocus', 'Requires','passione',     0.95)
add('iperfocus', 'OppositeOf','noia',       0.95)

# ── iperattività ─────────────────────────────────────────────────────────────
add('iperattività', 'Causes',  'movimento',    0.95, via='energia')
add('iperattività', 'Causes',  'irrequietezza',0.85, via='corpo')
add('iperattività', 'Does',    'generare',     0.90, via='azione')
add('iperattività', 'Has',     'energia',      0.95)
add('iperattività', 'IsA',     'impulso',      0.90, via='corpo')
add('iperattività', 'OppositeOf','calma',      0.95)
add('iperattività', 'OppositeOf','immobilità', 0.90)

# ── impulso (rifinitura) ─────────────────────────────────────────────────────
add('impulso', 'Causes',  'azione',       0.95, via='spinta')
add('impulso', 'Causes',  'immediatezza', 0.90, via='tempo')
add('impulso', 'Does',    'scavalcare',   0.85, via='attesa')
add('impulso', 'Has',     'forza',        0.90)
add('impulso', 'IsA',     'scintilla',    0.85, via='mente')
add('impulso', 'OppositeOf','riflessione',0.90)
add('impulso', 'OppositeOf','pazienza',   0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 62 — DISABILITÀ, OSTACOLO E ADATTAMENTO (disabilità, ostacolo, adattamento, supporto, fatica_cognitiva, accessibilità)
# ═══════════════════════════════════════════════════════════════════════════

# ── disabilità ───────────────────────────────────────────────────────────────
add('disabilità', 'Causes',  'sfida',        0.90, via='ostacolo')
add('disabilità', 'Causes',  'adattamento',  0.95, via='resilienza')
add('disabilità', 'Does',    'richiedere',   0.95, via='supporto')
add('disabilità', 'Has',     'dignità',      0.95)
add('disabilità', 'IsA',     'condizione',   0.90, via='vita')
add('disabilità', 'Requires','accessibilità',0.95)
add('disabilità', 'Requires','inclusione',   0.95)

# ── ostacolo ─────────────────────────────────────────────────────────────────
add('ostacolo', 'Causes',  'fatica',       0.90, via='sforzo')
add('ostacolo', 'Causes',  'soluzione',    0.85, via='ingegno')
add('ostacolo', 'Does',    'fermare',      0.85, via='percorso')
add('ostacolo', 'Has',     'resistenza',   0.90)
add('ostacolo', 'IsA',     'barriera',     0.95, via='spazio')
add('ostacolo', 'OppositeOf','agevolazione', 0.90)

# ── adattamento ──────────────────────────────────────────────────────────────
add('adattamento', 'Causes',  'sopravvivenza',0.95, via='evoluzione')
add('adattamento', 'Causes',  'armonia',      0.85, via='ambiente')
add('adattamento', 'Does',    'superare',     0.90, via='ostacolo')
add('adattamento', 'Has',     'flessibilità', 0.95)
add('adattamento', 'IsA',     'forza',        0.90, via='vita')
add('adattamento', 'Requires','pazienza',     0.85)
add('adattamento', 'OppositeOf','rigidità',   0.95)

# ── supporto ─────────────────────────────────────────────────────────────────
add('supporto', 'Causes',  'sollievo',     0.95, via='aiuto')
add('supporto', 'Causes',  'autonomia',    0.85, via='possibilità')
add('supporto', 'Does',    'sostenere',    0.95, via='peso')
add('supporto', 'Has',     'cura',         0.90)
add('supporto', 'IsA',     'strumento',    0.85, via='relazione')
add('supporto', 'Requires','empatia',      0.90)
add('supporto', 'OppositeOf','abbandono',  0.95)

# ── esaurimento (burnout / stanchezza mentale) ───────────────────────────────
add('esaurimento', 'Causes',  'vuoto',        0.95, via='energia')
add('esaurimento', 'Causes',  'blocco',       0.90, via='mente')
add('esaurimento', 'Does',    'consumare',    0.95, via='risorsa')
add('esaurimento', 'Has',     'sovraccarico', 0.95)
add('esaurimento', 'IsA',     'limite',       0.90, via='corpo')
add('esaurimento', 'Requires','riposo',       0.95)
add('esaurimento', 'Requires','silenzio',     0.90)
add('esaurimento', 'OppositeOf','lucidità',   0.90)

# ── accessibilità ────────────────────────────────────────────────────────────
add('accessibilità', 'Causes',  'inclusione',   0.95, via='spazio')
add('accessibilità', 'Causes',  'autonomia',    0.90, via='libertà')
add('accessibilità', 'Does',    'aprire',       0.95, via='porta')
add('accessibilità', 'Has',     'diritto',      0.90)
add('accessibilità', 'IsA',     'valore',       0.95, via='società')
add('accessibilità', 'Requires','cura',         0.85)
add('accessibilità', 'OppositeOf','esclusione', 0.95)
add('accessibilità', 'OppositeOf','barriera',   0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 63 — MECCANISMI DI COPING E REGOLAZIONE (maschera, stimolo, routine, flessibilità, ricarica)
# ═══════════════════════════════════════════════════════════════════════════

# ── maschera (masking) ───────────────────────────────────────────────────────
add('maschera', 'Causes',  'esaurimento',  0.95, via='sforzo')
add('maschera', 'Causes',  'protezione',   0.90, via='difesa')
add('maschera', 'Does',    'nascondere',   0.95, via='identità')
add('maschera', 'Does',    'adattare',     0.85, via='società')
add('maschera', 'Has',     'apparenza',    0.90)
add('maschera', 'IsA',     'strumento',    0.85, via='sopravvivenza')
add('maschera', 'Requires','energia',      0.95)
add('maschera', 'OppositeOf','autenticità',0.95)

# ── stimolo (stimming / ricerca di stimoli) ──────────────────────────────────
add('stimolo', 'Causes',  'calma',        0.90, via='ritmo')
add('stimolo', 'Causes',  'equilibrio',   0.85, via='sensazione')
add('stimolo', 'Does',    'regolare',     0.95, via='energia')
add('stimolo', 'Has',     'ripetizione',  0.85)
add('stimolo', 'IsA',     'reazione',     0.90, via='corpo')
add('stimolo', 'Requires','azione',       0.85)

# ── routine ──────────────────────────────────────────────────────────────────
add('routine', 'Causes',  'sicurezza',    0.95, via='ordine')
add('routine', 'Causes',  'noia',         0.80, via='ripetizione')
add('routine', 'Does',    'strutturare',  0.90, via='tempo')
add('routine', 'Has',     'stabilità',    0.95)
add('routine', 'IsA',     'ritmo',        0.90, via='vita')
add('routine', 'OppositeOf','caos',       0.90)
add('routine', 'OppositeOf','imprevisto', 0.95)

# ── flessibilità ─────────────────────────────────────────────────────────────
add('flessibilità', 'Causes',  'adattamento',  0.95, via='cambiamento')
add('flessibilità', 'Causes',  'soluzione',    0.85, via='creatività')
add('flessibilità', 'Does',    'accogliere',   0.90, via='imprevisto')
add('flessibilità', 'Has',     'apertura',     0.90)
add('flessibilità', 'IsA',     'risorsa',      0.95, via='mente')
add('flessibilità', 'Requires','energia',      0.85)
add('flessibilità', 'OppositeOf','rigidità',   0.95)

# ── ricarica ─────────────────────────────────────────────────────────────────
add('ricarica', 'Causes',  'lucidità',     0.90, via='riposo')
add('ricarica', 'Causes',  'forza',        0.95, via='energia')
add('ricarica', 'Does',    'recuperare',   0.95, via='risorsa')
add('ricarica', 'Has',     'calma',        0.85)
add('ricarica', 'IsA',     'bisogno',      0.95, via='mente')
add('ricarica', 'Requires','spazio',       0.85) # inteso come spazio sicuro/tempo per sé
add('ricarica', 'OppositeOf','esaurimento',0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 64 — OSTACOLI ESECUTIVI (paralisi, procrastinazione, caos, blocco, sovraccarico)
# ═══════════════════════════════════════════════════════════════════════════

# ── paralisi (esecutiva / task paralysis) ────────────────────────────────────
add('paralisi', 'Causes',  'blocco',       0.95, via='mente')
add('paralisi', 'Causes',  'angoscia',     0.85, via='impotenza')
add('paralisi', 'Does',    'fermare',      0.95, via='azione')
add('paralisi', 'Has',     'immobilità',   0.95)
add('paralisi', 'IsA',     'ostacolo',     0.90, via='mente')
add('paralisi', 'Requires','sovraccarico', 0.90) # paralisi causata dal troppo
add('paralisi', 'OppositeOf','flusso',     0.95)
add('paralisi', 'OppositeOf','azione',     0.95)

# ── procrastinazione ─────────────────────────────────────────────────────────
add('procrastinazione', 'Causes',  'ritardo',      0.95, via='tempo')
add('procrastinazione', 'Causes',  'ansia',        0.85, via='attesa')
add('procrastinazione', 'Does',    'rimandare',    0.95, via='scelta')
add('procrastinazione', 'Has',     'evitamento',   0.90)
add('procrastinazione', 'IsA',     'difesa',       0.85, via='mente') # intesa come meccanismo di evitamento dello sforzo
add('procrastinazione', 'Requires','paura',        0.80) # paura del fallimento o dello sforzo
add('procrastinazione', 'OppositeOf','immediatezza',0.90)

# ── sovraccarico ─────────────────────────────────────────────────────────────
add('sovraccarico', 'Causes',  'esaurimento',  0.95, via='energia')
add('sovraccarico', 'Causes',  'paralisi',     0.90, via='blocco')
add('sovraccarico', 'Does',    'schiacciare',  0.90, via='peso')
add('sovraccarico', 'Has',     'eccesso',      0.95)
add('sovraccarico', 'IsA',     'limite',       0.90, via='mente')
add('sovraccarico', 'Requires','stimolo',      0.85)
add('sovraccarico', 'OppositeOf','equilibrio', 0.95)

# ── caos (rifinitura in chiave cognitiva) ────────────────────────────────────
add('caos', 'Causes',  'smarrimento',  0.95, via='mente')
add('caos', 'Causes',  'possibilità',  0.85, via='creatività')
add('caos', 'Does',    'disorientare', 0.90, via='direzione')
add('caos', 'Has',     'disordine',    0.95)
add('caos', 'IsA',     'stato',        0.90, via='ambiente')
add('caos', 'OppositeOf','ordine',     0.95)
add('caos', 'OppositeOf','struttura',  0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 65 — SENSIBILITÀ E RISONANZA EMOTIVA (ipersensibilità, empatia, rifiuto, disregolazione, intensità)
# ═══════════════════════════════════════════════════════════════════════════

# ── ipersensibilità ──────────────────────────────────────────────────────────
add('ipersensibilità', 'Causes',  'sovraccarico', 0.90, via='percezione')
add('ipersensibilità', 'Causes',  'empatia',      0.95, via='connessione')
add('ipersensibilità', 'Does',    'amplificare',  0.95, via='sensazione')
add('ipersensibilità', 'Has',     'intensità',    0.95)
add('ipersensibilità', 'IsA',     'tratto',       0.90, via='natura')
add('ipersensibilità', 'Requires','apertura',     0.90)
add('ipersensibilità', 'OppositeOf','indifferenza',0.95)

# ── rifiuto (RSD - Rejection Sensitive Dysphoria) ────────────────────────────
add('rifiuto', 'Causes',  'dolore',       0.95, via='ferita')
add('rifiuto', 'Causes',  'isolamento',   0.90, via='difesa')
add('rifiuto', 'Does',    'respingere',   0.95, via='altro')
add('rifiuto', 'Has',     'esclusione',   0.90)
add('rifiuto', 'IsA',     'minaccia',     0.90, via='legame')
add('rifiuto', 'Requires','giudizio',     0.85)
add('rifiuto', 'OppositeOf','accettazione',0.95)
add('rifiuto', 'OppositeOf','appartenenza',0.90)

# ── disregolazione ───────────────────────────────────────────────────────────
add('disregolazione', 'Causes',  'esplosione',   0.90, via='emozione')
add('disregolazione', 'Causes',  'sofferenza',   0.85, via='corpo')
add('disregolazione', 'Does',    'travolgere',   0.95, via='limite')
add('disregolazione', 'Has',     'intensità',    0.90)
add('disregolazione', 'IsA',     'crisi',        0.90, via='mente')
add('disregolazione', 'Requires','sovraccarico', 0.85)
add('disregolazione', 'OppositeOf','calma',      0.95)
add('disregolazione', 'OppositeOf','controllo',  0.95)

# ── intensità ────────────────────────────────────────────────────────────────
add('intensità', 'Causes',  'profondità',   0.90, via='esperienza')
add('intensità', 'Causes',  'passione',     0.85, via='emozione')
add('intensità', 'Does',    'bruciare',     0.85, via='energia')
add('intensità', 'Has',     'forza',        0.95)
add('intensità', 'IsA',     'misura',       0.90, via='percezione')
add('intensità', 'OppositeOf','superficialità',0.95)
add('intensità', 'OppositeOf','apatia',     0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 66 — RICOMPENSA, DOPAMINA E OBIETTIVI (dopamina, traguardo, obiettivo, premio, gratificazione)
# ═══════════════════════════════════════════════════════════════════════════

# ── dopamina ─────────────────────────────────────────────────────────────────
add('dopamina', 'Causes',  'motivazione',  0.95, via='spinta')
add('dopamina', 'Causes',  'soddisfazione',0.90, via='premio')
add('dopamina', 'Does',    'attivare',     0.95, via='mente')
add('dopamina', 'Has',     'energia',      0.90)
add('dopamina', 'IsA',     'risorsa',      0.95, via='corpo')
add('dopamina', 'Requires','novità',       0.85) # Molto rilevante per l'ADHD
add('dopamina', 'OppositeOf','noia',       0.95)
add('dopamina', 'OppositeOf','apatia',     0.90)

# ── obiettivo ────────────────────────────────────────────────────────────────
add('obiettivo', 'Causes',  'direzione',    0.95, via='scopo')
add('obiettivo', 'Causes',  'azione',       0.90, via='motivazione')
add('obiettivo', 'Does',    'guidare',      0.95, via='scelta')
add('obiettivo', 'Has',     'scopo',        0.95)
add('obiettivo', 'IsA',     'visione',      0.85, via='futuro')
add('obiettivo', 'Requires','intenzione',   0.95)
add('obiettivo', 'OppositeOf','deriva',     0.90)

# ── traguardo ────────────────────────────────────────────────────────────────
add('traguardo', 'Causes',  'gratificazione',0.95, via='dopamina')
add('traguardo', 'Causes',  'celebrazione', 0.85, via='gioia')
add('traguardo', 'Does',    'concludere',   0.90, via='percorso')
add('traguardo', 'Has',     'risultato',    0.95)
add('traguardo', 'IsA',     'compimento',   0.95, via='azione')
add('traguardo', 'Requires','sforzo',       0.90)
add('traguardo', 'OppositeOf','fallimento', 0.90)

# ── gratificazione (premio / ricompensa) ─────────────────────────────────────
add('gratificazione', 'Causes',  'dopamina',     0.95, via='mente')
add('gratificazione', 'Causes',  'piacere',      0.90, via='corpo')
add('gratificazione', 'Does',    'rinforzare',   0.95, via='abitudine')
add('gratificazione', 'Has',     'soddisfazione',0.95)
add('gratificazione', 'IsA',     'premio',       0.90, via='risultato')
add('gratificazione', 'Requires','azione',       0.85)
add('gratificazione', 'OppositeOf','frustrazione',0.95)

# ── noia (rivisitata come carenza di dopamina) ───────────────────────────────
add('noia', 'Causes',  'irrequietezza',0.90, via='energia')
add('noia', 'Causes',  'ricerca',      0.85, via='novità')
add('noia', 'Does',    'spegnere',     0.90, via='attenzione')
add('noia', 'Has',     'vuoto',        0.85)
add('noia', 'IsA',     'mancanza',     0.95, via='dopamina')
add('noia', 'OppositeOf','interesse',  0.95)
add('noia', 'OppositeOf','iperfocus',  0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 67 — TEMPO, PIANIFICAZIONE E ABITUDINI (abitudine, pianificazione, scadenza, urgenza, attesa)
# ═══════════════════════════════════════════════════════════════════════════

# ── abitudine ────────────────────────────────────────────────────────────────
add('abitudine', 'Causes',  'automatismo',  0.95, via='ripetizione')
add('abitudine', 'Does',    'semplificare', 0.90, via='azione')
add('abitudine', 'Does',    'risparmiare',  0.85, via='energia') # Per l'ADHD l'abitudine è un salvagente energetico
add('abitudine', 'Has',     'costanza',     0.90)
add('abitudine', 'IsA',     'schema',       0.95, via='comportamento')
add('abitudine', 'Requires','ripetizione',  0.95)
add('abitudine', 'OppositeOf','novità',     0.95)

# ── pianificazione ───────────────────────────────────────────────────────────
add('pianificazione', 'Causes',  'struttura',    0.95, via='tempo')
add('pianificazione', 'Causes',  'ordine',       0.90, via='mente')
add('pianificazione', 'Does',    'prevedere',    0.95, via='futuro')
add('pianificazione', 'Has',     'metodo',       0.85)
add('pianificazione', 'IsA',     'strumento',    0.90, via='esecutività')
add('pianificazione', 'Requires','lucidità',     0.85) # Richiede molta funzione esecutiva
add('pianificazione', 'OppositeOf','improvvisazione',0.95)
add('pianificazione', 'OppositeOf','caos',       0.90)

# ── scadenza ─────────────────────────────────────────────────────────────────
add('scadenza', 'Causes',  'urgenza',      0.95, via='limite')
add('scadenza', 'Causes',  'ansia',        0.85, via='pressione')
add('scadenza', 'Does',    'forzare',      0.90, via='azione')
add('scadenza', 'Has',     'fine',         0.95)
add('scadenza', 'IsA',     'confine',      0.90, via='tempo')
add('scadenza', 'Requires','misura',       0.85) # Richiede di saper misurare il tempo
add('scadenza', 'OppositeOf','eternità',   0.85)

# ── urgenza ──────────────────────────────────────────────────────────────────
add('urgenza', 'Causes',  'azione',       0.95, via='dopamina') # L'urgenza innesca dopamina (strategia ADHD classica)
add('urgenza', 'Causes',  'stress',       0.90, via='pressione')
add('urgenza', 'Does',    'sbloccare',    0.90, via='paralisi') # rompe la paralisi esecutiva
add('urgenza', 'Has',     'velocità',     0.90)
add('urgenza', 'IsA',     'spinta',       0.95, via='tempo')
add('urgenza', 'Requires','necessità',    0.95)
add('urgenza', 'OppositeOf','calma',      0.95)
add('urgenza', 'OppositeOf','lentezza',   0.90)

# ── attesa ───────────────────────────────────────────────────────────────────
add('attesa', 'Causes',  'noia',         0.85, via='vuoto')
add('attesa', 'Causes',  'ansia',        0.80, via='futuro')
add('attesa', 'Does',    'sospendere',   0.90, via='azione')
add('attesa', 'Has',     'pazienza',     0.90)
add('attesa', 'IsA',     'stato',        0.90, via='tempo')
add('attesa', 'Requires','tempo',        0.95)
add('attesa', 'OppositeOf','immediatezza',0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 68 — VERBI E AZIONI QUOTIDIANE (dormire, svegliarsi, mangiare, bere, camminare, correre, ecc.)
# ═══════════════════════════════════════════════════════════════════════════

# ── azioni di base e corpo ───────────────────────────────────────────────────
add('dormire', 'Causes',  'riposo',       0.95, via='sonno')
add('dormire', 'Does',    'spegnere',     0.90, via='coscienza')
add('dormire', 'Requires','stanchezza',   0.85)
add('dormire', 'OppositeOf','svegliarsi', 0.95)

add('svegliarsi', 'Causes',  'coscienza',    0.95, via='attenzione')
add('svegliarsi', 'Does',    'iniziare',     0.90, via='giorno')
add('svegliarsi', 'Requires','energia',      0.85)
add('svegliarsi', 'OppositeOf','dormire',    0.95)

add('mangiare', 'Causes',  'nutrimento',   0.95, via='energia')
add('mangiare', 'Does',    'assimilare',   0.90, via='cibo')
add('mangiare', 'Requires','fame',         0.95)
add('mangiare', 'OppositeOf','digiunare',  0.95)

add('bere', 'Causes',  'sollievo',     0.90, via='acqua')
add('bere', 'Does',    'dissetare',    0.95, via='corpo')
add('bere', 'Requires','sete',         0.95)

# ── movimento ────────────────────────────────────────────────────────────────
add('camminare', 'Causes',  'spostamento',  0.95, via='passo')
add('camminare', 'Does',    'esplorare',    0.85, via='spazio')
add('camminare', 'Requires','gamba',        0.95)
add('camminare', 'OppositeOf','fermarsi',   0.90)

add('correre', 'Causes',  'velocità',     0.95, via='movimento')
add('correre', 'Causes',  'fatica',       0.85, via='sforzo')
add('correre', 'Does',    'fuggire',      0.80, via='pericolo')
add('correre', 'Requires','energia',      0.90)
add('correre', 'OppositeOf','camminare',  0.85) # Opposto relativo alla velocità

# ── interazione ed espressione ───────────────────────────────────────────────
add('parlare', 'Causes',  'comunicazione',0.95, via='parola')
add('parlare', 'Does',    'esprimere',    0.95, via='pensiero')
add('parlare', 'Requires','voce',         0.90)
add('parlare', 'OppositeOf','tacere',     0.95)

add('ascoltare', 'Causes',  'comprensione', 0.95, via='attenzione')
add('ascoltare', 'Does',    'ricevere',     0.90, via='parola')
add('ascoltare', 'Requires','silenzio',     0.85)
add('ascoltare', 'OppositeOf','ignorare',   0.90)

add('guardare', 'Causes',  'visione',      0.95, via='occhio')
add('guardare', 'Does',    'osservare',    0.90, via='attenzione')
add('guardare', 'Requires','luce',         0.90)

add('toccare', 'Causes',  'contatto',     0.95, via='mano')
add('toccare', 'Does',    'sentire',      0.90, via='materia')
add('toccare', 'Requires','vicinanza',    0.95)

# ── attività intellettive e fisiche ──────────────────────────────────────────
add('lavorare', 'Causes',  'risultato',    0.95, via='sforzo')
add('lavorare', 'Does',    'produrre',     0.90, via='valore')
add('lavorare', 'Requires','impegno',      0.90)
add('lavorare', 'OppositeOf','riposare',   0.95)

add('giocare', 'Causes',  'divertimento', 0.95, via='gioia')
add('giocare', 'Does',    'sperimentare', 0.90, via='regola')
add('giocare', 'Requires','fantasia',     0.85)
add('giocare', 'OppositeOf','lavorare',   0.85) # Spesso visti in contrapposizione

add('leggere', 'Causes',  'conoscenza',   0.95, via='parola')
add('leggere', 'Does',    'decifrare',    0.90, via='simbolo')
add('leggere', 'Requires','attenzione',   0.90)

add('scrivere', 'Causes',  'memoria',      0.95, via='segno')
add('scrivere', 'Does',    'registrare',   0.90, via='pensiero')
add('scrivere', 'Requires','strumento',    0.85)

add('cucinare', 'Causes',  'nutrimento',   0.95, via='cibo')
add('cucinare', 'Does',    'trasformare',  0.90, via='fuoco')
add('cucinare', 'Requires','ingrediente',  0.95)

add('pulire', 'Causes',  'ordine',       0.95, via='spazio')
add('pulire', 'Does',    'rimuovere',    0.90, via='sporco')
add('pulire', 'Requires','cura',         0.85)
add('pulire', 'OppositeOf','sporcare',   0.95)

add('costruire', 'Causes',  'struttura',    0.95, via='forma')
add('costruire', 'Does',    'creare',       0.90, via='materia')
add('costruire', 'Requires','progetto',     0.85)
add('costruire', 'OppositeOf','distruggere',0.95)
add('costruire', 'OppositeOf','rompere',    0.90)

add('rompere', 'Causes',  'danno',        0.95, via='forza')
add('rompere', 'Does',    'dividere',     0.90, via='materia')
add('rompere', 'Requires','impatto',      0.85)
add('rompere', 'OppositeOf','aggiustare', 0.95)
add('rompere', 'OppositeOf','costruire',  0.90)

# ═══════════════════════════════════════════════════════════════════════════
# § 69 — STATI EMOTIVI E CONDIZIONI (nervoso, ansioso, stanco, annoiato, ecc.)
# ═══════════════════════════════════════════════════════════════════════════

add('nervoso', 'Has', 'agitazione', 0.95)
add('nervoso', 'Causes', 'reazione', 0.85, via='tensione')
add('nervoso', 'IsA', 'stato', 0.90, via='emozione')
add('nervoso', 'OppositeOf', 'calmo', 0.95)
add('nervoso', 'OppositeOf', 'sereno', 0.90)

add('ansioso', 'Has', 'paura', 0.90)
add('ansioso', 'Causes', 'preoccupazione', 0.95, via='futuro')
add('ansioso', 'IsA', 'stato', 0.90, via='mente')
add('ansioso', 'OppositeOf', 'tranquillo', 0.95)

add('stanco', 'Has', 'fatica', 0.95)
add('stanco', 'Causes', 'riposo', 0.90, via='bisogno')
add('stanco', 'Requires', 'energia', 0.85)
add('stanco', 'OppositeOf', 'riposato', 0.95)
add('stanco', 'OppositeOf', 'energico', 0.90)

add('annoiato', 'Has', 'vuoto', 0.85)
add('annoiato', 'Causes', 'distrazione', 0.90, via='mente')
add('annoiato', 'Requires', 'stimolo', 0.95)
add('annoiato', 'OppositeOf', 'interessato', 0.95)
add('annoiato', 'OppositeOf', 'entusiasta', 0.90)

add('confuso', 'Has', 'caos', 0.90)
add('confuso', 'Causes', 'dubbio', 0.95, via='mente')
add('confuso', 'Requires', 'chiarezza', 0.95)
add('confuso', 'OppositeOf', 'lucido', 0.95)

add('soddisfatto', 'Has', 'pienezza', 0.90)
add('soddisfatto', 'Causes', 'pace', 0.85, via='mente')
add('soddisfatto', 'IsA', 'stato', 0.90, via='emozione')
add('soddisfatto', 'OppositeOf', 'frustrato', 0.95)
add('soddisfatto', 'OppositeOf', 'deluso', 0.90)

add('deluso', 'Has', 'tristezza', 0.90)
add('deluso', 'Causes', 'amarezza', 0.95, via='aspettativa')
add('deluso', 'Requires', 'accettazione', 0.85)
add('deluso', 'OppositeOf', 'soddisfatto', 0.95)

add('sorpreso', 'Has', 'meraviglia', 0.90)
add('sorpreso', 'Causes', 'attenzione', 0.95, via='novità')
add('sorpreso', 'OppositeOf', 'indifferente', 0.85)

add('frustrato', 'Has', 'rabbia', 0.90)
add('frustrato', 'Causes', 'blocco', 0.95, via='ostacolo')
add('frustrato', 'Requires', 'sfogo', 0.85)
add('frustrato', 'OppositeOf', 'appagato', 0.95)

add('sereno', 'Has', 'pace', 0.95)
add('sereno', 'Causes', 'equilibrio', 0.90, via='mente')
add('sereno', 'IsA', 'stato', 0.90, via='emozione')
add('sereno', 'OppositeOf', 'angosciato', 0.95)
add('sereno', 'OppositeOf', 'agitato', 0.95)

add('agitato', 'Has', 'movimento', 0.90)
add('agitato', 'Causes', 'caos', 0.85, via='mente')
add('agitato', 'OppositeOf', 'tranquillo', 0.95)
add('agitato', 'OppositeOf', 'calmo', 0.95)

add('tranquillo', 'Has', 'silenzio', 0.85)
add('tranquillo', 'Causes', 'riposo', 0.90, via='corpo')
add('tranquillo', 'OppositeOf', 'nervoso', 0.95)
add('tranquillo', 'OppositeOf', 'preoccupato', 0.90)

add('preoccupato', 'Has', 'ansia', 0.90)
add('preoccupato', 'Causes', 'pensiero', 0.95, via='futuro')
add('preoccupato', 'OppositeOf', 'spensierato', 0.95)

add('sollevato', 'Has', 'leggerezza', 0.90)
add('sollevato', 'Causes', 'sollievo', 0.95, via='mente')
add('sollevato', 'Requires', 'fine', 0.85, via='tensione')
add('sollevato', 'OppositeOf', 'oppresso', 0.95)

add('angosciato', 'Has', 'dolore', 0.95)
add('angosciato', 'Causes', 'paralisi', 0.90, via='paura')
add('angosciato', 'OppositeOf', 'sereno', 0.95)

add('entusiasta', 'Has', 'energia', 0.95)
add('entusiasta', 'Causes', 'azione', 0.90, via='passione')
add('entusiasta', 'OppositeOf', 'apatico', 0.95)
add('entusiasta', 'OppositeOf', 'annoiato', 0.90)

add('indifferente', 'Has', 'vuoto', 0.85)
add('indifferente', 'Causes', 'distacco', 0.95, via='emozione')
add('indifferente', 'OppositeOf', 'curioso', 0.95)
add('indifferente', 'OppositeOf', 'appassionato', 0.90)

add('curioso', 'Has', 'attenzione', 0.95)
add('curioso', 'Causes', 'domanda', 0.90, via='ricerca')
add('curioso', 'Requires', 'novità', 0.85)
add('curioso', 'OppositeOf', 'indifferente', 0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 70 — PAROLE TEMPORALI (stanotte, ieri, oggi, domani, prima, dopo, ecc.)
# ═══════════════════════════════════════════════════════════════════════════

add('stanotte', 'IsA', 'tempo', 0.95, via='notte')
add('stanotte', 'Requires', 'notte', 0.90)

add('ieri', 'IsA', 'passato', 0.95)
add('ieri', 'Causes', 'memoria', 0.90, via='ricordo')
add('ieri', 'OppositeOf', 'domani', 0.95)

add('oggi', 'IsA', 'presente', 0.95)
add('oggi', 'Causes', 'azione', 0.90, via='realtà')

add('domani', 'IsA', 'futuro', 0.95)
add('domani', 'Causes', 'attesa', 0.90, via='speranza')
add('domani', 'OppositeOf', 'ieri', 0.95)

add('prima', 'IsA', 'passato', 0.90)
add('prima', 'Causes', 'causa', 0.85, via='origine')
add('prima', 'OppositeOf', 'dopo', 0.95)

add('dopo', 'IsA', 'futuro', 0.90)
add('dopo', 'Causes', 'effetto', 0.85, via='conseguenza')
add('dopo', 'OppositeOf', 'prima', 0.95)

add('sempre', 'IsA', 'eternità', 0.95)
add('sempre', 'Has', 'costanza', 0.95)
add('sempre', 'OppositeOf', 'mai', 0.95)

add('mai', 'IsA', 'assenza', 0.95, via='tempo')
add('mai', 'OppositeOf', 'sempre', 0.95)

add('ora', 'IsA', 'presente', 0.95)
add('ora', 'Has', 'immediatezza', 0.90)
add('ora', 'OppositeOf', 'dopo', 0.85)

add('adesso', 'IsA', 'presente', 0.95)
add('adesso', 'Causes', 'azione', 0.90, via='urgenza')

add('presto', 'IsA', 'tempo', 0.90)
add('presto', 'Has', 'anticipo', 0.95)
add('presto', 'OppositeOf', 'tardi', 0.95)

add('tardi', 'IsA', 'tempo', 0.90)
add('tardi', 'Has', 'ritardo', 0.95)
add('tardi', 'OppositeOf', 'presto', 0.95)

add('mattina', 'IsA', 'giorno', 0.95)
add('mattina', 'Causes', 'inizio', 0.90, via='risveglio')
add('mattina', 'OppositeOf', 'sera', 0.95)

add('sera', 'IsA', 'giorno', 0.95)
add('sera', 'Causes', 'fine', 0.90, via='riposo')
add('sera', 'OppositeOf', 'mattina', 0.95)

add('notte', 'IsA', 'tempo', 0.95)
add('notte', 'Causes', 'sonno', 0.95, via='buio')
add('notte', 'OppositeOf', 'giorno', 0.95)

add('giorno', 'IsA', 'tempo', 0.95)
add('giorno', 'Causes', 'azione', 0.90, via='luce')
add('giorno', 'OppositeOf', 'notte', 0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 71 — NEGAZIONE, ASSENZA E BISOGNI DEL CORPO
# ═══════════════════════════════════════════════════════════════════════════

# ── negazione, assenza, rifiuto ──────────────────────────────────────────────
add('niente', 'IsA', 'assenza', 0.95, via='tutto')
add('niente', 'OppositeOf', 'tutto', 0.95)
add('niente', 'OppositeOf', 'qualcosa', 0.90)

add('nulla', 'IsA', 'assenza', 0.95, via='spazio')
add('nulla', 'Causes', 'vuoto', 0.90, via='mente')
add('nulla', 'OppositeOf', 'tutto', 0.95)

add('nessuno', 'IsA', 'assenza', 0.95, via='persona')
add('nessuno', 'Causes', 'solitudine', 0.85, via='spazio')
add('nessuno', 'OppositeOf', 'tutti', 0.95)
add('nessuno', 'OppositeOf', 'qualcuno', 0.90)

add('voglia', 'IsA', 'desiderio', 0.95, via='spinta')
add('voglia', 'Causes', 'azione', 0.90, via='motivazione')
add('voglia', 'OppositeOf', 'apatia', 0.95)

add('svogliato', 'Has', 'apatia', 0.95)
add('svogliato', 'Causes', 'inerzia', 0.90, via='corpo')
add('svogliato', 'OppositeOf', 'entusiasta', 0.95)

add('rinuncia', 'Causes', 'perdita', 0.85, via='scelta')
add('rinuncia', 'Does', 'abbandonare', 0.95, via='obiettivo')
add('rinuncia', 'OppositeOf', 'conquista', 0.90)
add('rinuncia', 'OppositeOf', 'impegno', 0.85)

# ── bisogni e corpo ──────────────────────────────────────────────────────────
add('riposato', 'Has', 'energia', 0.95)
add('riposato', 'Causes', 'azione', 0.90, via='forza')
add('riposato', 'IsA', 'stato', 0.90, via='corpo')
add('riposato', 'OppositeOf', 'stanco', 0.95)

add('affamato', 'Has', 'fame', 0.95)
add('affamato', 'Causes', 'ricerca', 0.90, via='cibo')
add('affamato', 'Requires', 'nutrimento', 0.95)
add('affamato', 'OppositeOf', 'sazio', 0.95)

add('assetato', 'Has', 'sete', 0.95)
add('assetato', 'Causes', 'ricerca', 0.90, via='acqua')
add('assetato', 'Requires', 'acqua', 0.95)
add('assetato', 'OppositeOf', 'dissetato', 0.95)

add('freddo', 'Has', 'gelo', 0.90)
add('freddo', 'Causes', 'tremore', 0.85, via='corpo')
add('freddo', 'IsA', 'sensazione', 0.95, via='pelle')
add('freddo', 'Requires', 'calore', 0.90)
add('freddo', 'OppositeOf', 'caldo', 0.95)

add('caldo', 'Has', 'calore', 0.95)
add('caldo', 'Causes', 'sudore', 0.85, via='corpo')
add('caldo', 'IsA', 'sensazione', 0.95, via='pelle')
add('caldo', 'OppositeOf', 'freddo', 0.95)

add('sveglio', 'Has', 'attenzione', 0.90)
add('sveglio', 'Causes', 'coscienza', 0.95, via='mente')
add('sveglio', 'IsA', 'stato', 0.95, via='corpo')
add('sveglio', 'OppositeOf', 'addormentato', 0.95)

add('addormentato', 'Has', 'sonno', 0.95)
add('addormentato', 'Causes', 'sogno', 0.90, via='mente')
add('addormentato', 'IsA', 'stato', 0.95, via='corpo')
add('addormentato', 'OppositeOf', 'sveglio', 0.95)

# ═══════════════════════════════════════════════════════════════════════════
# § 72 — RELAZIONI DIALOGICHE E INTERAZIONE
# ═══════════════════════════════════════════════════════════════════════════

add('racconto', 'IsA', 'comunicazione', 0.95, via='parola')
add('racconto', 'Causes', 'memoria', 0.90, via='storia')
add('racconto', 'Requires', 'ascolto', 0.85)

add('lamento', 'IsA', 'espressione', 0.95, via='sofferenza')
add('lamento', 'Causes', 'sfogo', 0.90, via='voce')
add('lamento', 'Requires', 'dolore', 0.85)

add('sfogo', 'Causes', 'sollievo', 0.90, via='emozione')
add('sfogo', 'Does', 'liberare', 0.95, via='tensione')
add('sfogo', 'Requires', 'ascolto', 0.95)

add('confidenza', 'Causes', 'legame', 0.90, via='segreto')
add('confidenza', 'Does', 'condividere', 0.95, via='verità')
add('confidenza', 'Requires', 'fiducia', 0.95)

add('domanda', 'Causes', 'risposta', 0.95, via='comprensione')
add('domanda', 'Does', 'cercare', 0.90, via='verità')
add('domanda', 'Requires', 'curiosità', 0.90)
add('domanda', 'OppositeOf', 'risposta', 0.85)

add('risposta', 'Causes', 'chiarezza', 0.90, via='verità')
add('risposta', 'Does', 'risolvere', 0.85, via='dubbio')
add('risposta', 'Requires', 'domanda', 0.95)

# ═══════════════════════════════════════════════════════════════════════════
# Applica e salva
# ═══════════════════════════════════════════════════════════════════════════

if not DRY_RUN:
    kg['edges'] = list(edge_map.values())
    with open(KG_PATH, 'w', encoding='utf-8') as f:
        json.dump(kg, f, ensure_ascii=False, separators=(',', ':'))

new_count = len(edge_map)
print(f"\nRIMOSSI  : {len(removed)}")
for x in removed: print(f"  {x}")
print(f"\nAGGIUNTI : {len(added)}")
for x in added: print(f"  {x}")
print(f"\nMODIFICATI: {len(changed)}")
for x in changed: print(f"  {x}")
print(f"\nArchi: {original_count} -> {new_count} ({new_count-original_count:+d})")
if DRY_RUN:
    print("\n[DRY RUN — nessuna modifica salvata]")
