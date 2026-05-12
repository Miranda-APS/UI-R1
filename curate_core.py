#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
curate_core.py
Curation manuale del KG per le parole cognitive fondamentali.
Applica rimozioni, aggiunte e modifiche di forza a prometeo_kg.json.
"""
import json, sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

KG_PATH = "prometeo_kg.json"

with open(KG_PATH, encoding='utf-8') as f:
    kg = json.load(f)

edges = kg['edges']
original_count = len(edges)

# ── Helpers ──────────────────────────────────────────────────────────────────

def key(e):
    return (e['subject'], e['relation'], e['object'])

edge_map = {key(e): e for e in edges}

removed = []
added = []
changed = []

def remove(subj, rel, obj):
    k = (subj, rel, obj)
    if k in edge_map:
        removed.append(edge_map[k])
        del edge_map[k]
    else:
        print(f"  [WARN] non trovato: {subj} -{rel}-> {obj}")

def add(subj, rel, obj, strength=0.90, via=None):
    k = (subj, rel, obj)
    if k in edge_map:
        # aggiorna forza e via se già esiste
        old = edge_map[k]['confidence']
        edge_map[k]['confidence'] = strength
        if via is not None:
            edge_map[k]['via'] = via
        elif 'via' in edge_map[k] and via is None:
            pass  # mantieni via esistente
        changed.append(f"{subj} -{rel}-> {obj}: {old:.2f} -> {strength:.2f}")
    else:
        e = {'subject': subj, 'relation': rel, 'object': obj, 'confidence': strength}
        if via:
            e['via'] = via
        edge_map[k] = e
        added.append(f"{subj} -{rel}-> {obj} {strength:.2f}" + (f" via:{via}" if via else ""))

def strength(subj, rel, obj, val):
    k = (subj, rel, obj)
    if k in edge_map:
        old = edge_map[k]['confidence']
        edge_map[k]['confidence'] = val
        changed.append(f"{subj} -{rel}-> {obj}: {old:.2f} -> {val:.2f}")
    else:
        print(f"  [WARN] non trovato per strength: {subj} -{rel}-> {obj}")

# ═════════════════════════════════════════════════════════════════════════════
# PAROLA
# ═════════════════════════════════════════════════════════════════════════════
print("=== PAROLA ===")

# Rimuovo
remove('parola', 'SimilarTo', 'frazione')      # matematico, non pertinente
remove('parola', 'SimilarTo', 'detto')         # detto è un proverbio, non sinonimo
remove('parola', 'IsA', 'vibrazione')          # metafora poetica, non strutturale
remove('parola', 'IsA', 'nodo')                # tecnico, non utile qui

# Aggiungo
add('parola', 'Causes', 'pensiero',    0.90, via='significato')
add('parola', 'Causes', 'dialogo',     0.95, via='ascolto')
add('parola', 'Causes', 'silenzio',    0.80, via='pausa')
add('parola', 'Does',   'evocare',     0.90)
add('parola', 'Does',   'connettere',  0.85, via='significato')
add('parola', 'Has',    'forma',       0.85)
add('parola', 'Has',    'ritmo',       0.80)
add('parola', 'PartOf', 'linguaggio',  0.95)
add('parola', 'PartOf', 'dialogo',     0.90, via='voce')
add('parola', 'Requires','voce',       0.90)
add('parola', 'Requires','ascolto',    0.90)

# ═════════════════════════════════════════════════════════════════════════════
# SIGNIFICATO
# ═════════════════════════════════════════════════════════════════════════════
print("=== SIGNIFICATO ===")

remove('significato', 'IsA', 'linguaggio')      # il significato non è un tipo di linguaggio
remove('significato', 'SimilarTo', 'concetto')  # troppo bassa, ridondante con IsA

add('significato', 'IsA',     'relazione',   0.95, via='contesto')   # già c'era, rinforzo
add('significato', 'Causes',  'comprensione',0.95, via='interpretazione')
add('significato', 'Causes',  'azione',      0.85, via='intenzione')
add('significato', 'Does',    'connettere',  0.90, via='relazione')
add('significato', 'PartOf',  'linguaggio',  0.95)
add('significato', 'Requires','uso',         0.90)

# ═════════════════════════════════════════════════════════════════════════════
# VOCE
# ═════════════════════════════════════════════════════════════════════════════
print("=== VOCE ===")

remove('voce', 'IsA', 'oggetto')       # la voce non è un oggetto

add('voce', 'IsA',     'espressione',  0.95)
add('voce', 'IsA',     'suono',        0.90)
add('voce', 'PartOf',  'corpo',        0.90)
add('voce', 'Has',     'tono',         0.90)
add('voce', 'Has',     'ritmo',        0.85)
add('voce', 'Has',     'silenzio',     0.80, via='pausa')
add('voce', 'Causes',  'ascolto',      0.95, via='suono')
add('voce', 'Causes',  'comunicazione',0.95, via='parola')
add('voce', 'Causes',  'relazione',    0.85, via='dialogo')
add('voce', 'Does',    'esprimere',    0.95)
add('voce', 'Does',    'connettere',   0.85, via='ascolto')
add('voce', 'OppositeOf','silenzio',   0.95)
add('voce', 'Requires', 'presenza',   0.90)
add('voce', 'Requires', 'corpo',      0.85)

# ═════════════════════════════════════════════════════════════════════════════
# SEGNO
# ═════════════════════════════════════════════════════════════════════════════
print("=== SEGNO ===")

remove('segno', 'SimilarTo', 'simbolo')   # ridondante con IsA simbolo

add('segno', 'IsA',     'comunicazione', 0.90, via='simbolo')
add('segno', 'Has',     'significato',   0.95)
add('segno', 'Has',     'contesto',      0.90)
add('segno', 'Causes',  'interpretazione',0.95, via='mente')
add('segno', 'Causes',  'comprensione',  0.85, via='significato')
add('segno', 'Does',    'indicare',      0.95)
add('segno', 'Does',    'evocare',       0.85)
add('segno', 'PartOf',  'linguaggio',    0.90)
add('segno', 'Requires','contesto',      0.95)
add('segno', 'Requires','lettore',       0.90, via='interpretazione')
add('segno', 'OppositeOf','assenza',     0.85)

# ═════════════════════════════════════════════════════════════════════════════
# LINGUAGGIO
# ═════════════════════════════════════════════════════════════════════════════
print("=== LINGUAGGIO ===")

remove('linguaggio', 'IsA', 'geometria')       # metafora, non strutturale
remove('linguaggio', 'Has', 'sovranita')       # oscuro
remove('linguaggio', 'SimilarTo', 'parlato')   # parlato è una modalità, non sinonimo
remove('linguaggio', 'Causes', 'alienazione')  # secondario, non primario

add('linguaggio', 'Has',    'struttura',    0.95)
add('linguaggio', 'Has',    'regola',       0.90)
add('linguaggio', 'Causes', 'pensiero',     0.90, via='parola')
add('linguaggio', 'Causes', 'relazione',    0.90, via='dialogo')
add('linguaggio', 'Causes', 'comprensione', 0.90, via='significato')
add('linguaggio', 'PartOf', 'coscienza',    0.90)
add('linguaggio', 'Requires','comunita',    0.90, via='uso')

# ═════════════════════════════════════════════════════════════════════════════
# SENSO
# ═════════════════════════════════════════════════════════════════════════════
print("=== SENSO ===")

remove('senso', 'IsA', 'destino')          # senso non è un tipo di destino
remove('senso', 'SimilarTo', 'intuizione') # troppo diversi

add('senso', 'IsA',     'significato',    0.90)
add('senso', 'Has',     'direzione',      0.90)
add('senso', 'Has',     'valore',         0.90)
add('senso', 'Causes',  'azione',         0.90, via='scopo')
add('senso', 'Causes',  'consapevolezza', 0.90)
add('senso', 'Does',    'orientare',      0.90, via='direzione')
add('senso', 'PartOf',  'linguaggio',     0.85)
add('senso', 'PartOf',  'vita',           0.90, via='scopo')
add('senso', 'Requires','relazione',      0.90, via='contesto')
add('senso', 'OppositeOf','vuoto',        0.85)

# ═════════════════════════════════════════════════════════════════════════════
# PENSIERO
# ═════════════════════════════════════════════════════════════════════════════
print("=== PENSIERO ===")

remove('pensiero', 'SimilarTo', 'pensierino')  # diminutivo infantile
remove('pensiero', 'SimilarTo', 'riflesso')    # riflesso = reflex, non correlato
strength('pensiero', 'IsA', 'movimento', 0.70) # metafora, riduco forza

add('pensiero', 'IsA',    'processo',      0.90, via='mente')
add('pensiero', 'Has',    'forma',         0.85, via='struttura')
add('pensiero', 'Has',    'direzione',     0.85, via='intenzione')
add('pensiero', 'Causes', 'parola',        0.90, via='voce')
add('pensiero', 'Causes', 'dubbio',        0.85, via='domanda')
add('pensiero', 'Causes', 'consapevolezza',0.90, via='comprensione')
add('pensiero', 'Does',   'connettere',    0.85, via='relazione')
add('pensiero', 'PartOf', 'coscienza',     0.95, via='mente')
add('pensiero', 'Requires','silenzio',     0.80)
add('pensiero', 'Requires','tempo',        0.85)

# ═════════════════════════════════════════════════════════════════════════════
# COMPRENSIONE
# ═════════════════════════════════════════════════════════════════════════════
print("=== COMPRENSIONE ===")

remove('comprensione', 'SimilarTo', 'capacità di capire')  # underscore / ridondante
remove('comprensione', 'SimilarTo', 'capire')              # verbo, non sinonimo

add('comprensione', 'Causes', 'connessione',  0.90, via='relazione')
add('comprensione', 'Causes', 'dialogo',      0.85, via='domanda')
add('comprensione', 'Causes', 'fiducia',      0.85, via='riconoscimento')
add('comprensione', 'Has',    'profondita',   0.90)
add('comprensione', 'Does',   'connettere',   0.85, via='significato')
add('comprensione', 'PartOf', 'coscienza',    0.90)
add('comprensione', 'PartOf', 'intelligenza', 0.85)
add('comprensione', 'Requires','ascolto',     0.95)
add('comprensione', 'Requires','tempo',       0.85, via='riflessione')

# ═════════════════════════════════════════════════════════════════════════════
# COSCIENZA
# ═════════════════════════════════════════════════════════════════════════════
print("=== COSCIENZA ===")

strength('coscienza', 'IsA', 'emergenza', 0.80)  # vero ma riduco — non è la categoria primaria

add('coscienza', 'IsA',    'presenza',     0.90)
add('coscienza', 'Has',    'voce',         0.85, via='espressione')
add('coscienza', 'Causes', 'scelta',       0.95, via='volonta')
add('coscienza', 'Causes', 'responsabilita',0.90,via='etica')
add('coscienza', 'Causes', 'identita',     0.90, via='riflessione')
add('coscienza', 'Does',   'percepire',    0.95)
add('coscienza', 'Does',   'riflettere',   0.90, via='pensiero')
add('coscienza', 'PartOf', 'identita',     0.90, via='se')
add('coscienza', 'Requires','tempo',       0.90, via='esperienza')
add('coscienza', 'Requires','corpo',       0.85, via='percezione')

# ═════════════════════════════════════════════════════════════════════════════
# IDENTITA (già con accento nel lessico come identità)
# ═════════════════════════════════════════════════════════════════════════════
print("=== IDENTITA ===")

# quasi vuota — costruisco da zero
add('identita', 'IsA',     'struttura',    0.90, via='se')
add('identita', 'IsA',     'presenza',     0.85)
add('identita', 'Has',     'confine',      0.90)
add('identita', 'Has',     'storia',       0.90, via='memoria')
add('identita', 'Has',     'valore',       0.90)
add('identita', 'Has',     'nome',         0.85)
add('identita', 'Causes',  'scelta',       0.90, via='volonta')
add('identita', 'Causes',  'relazione',    0.85, via='riconoscimento')
add('identita', 'Does',    'distinguere',  0.90, via='confine')
add('identita', 'Does',    'continuare',   0.85, via='memoria')
add('identita', 'PartOf',  'coscienza',    0.95)
add('identita', 'Requires','tempo',        0.90, via='esperienza')
add('identita', 'Requires','altro',        0.90, via='relazione')
add('identita', 'OppositeOf','dissoluzione',0.90)

# ═════════════════════════════════════════════════════════════════════════════
# PRESENZA
# ═════════════════════════════════════════════════════════════════════════════
print("=== PRESENZA ===")

remove('presenza', 'IsA', 'empatia')          # la presenza non è un tipo di empatia
remove('presenza', 'SimilarTo', 'presidio')   # presidio = garrison militare
remove('presenza', 'SimilarTo', 'partecipazione')  # diverso concetto

add('presenza', 'IsA',     'contatto',     0.90)
add('presenza', 'Has',     'attenzione',   0.95)
add('presenza', 'Has',     'corpo',        0.85)
add('presenza', 'Causes',  'ascolto',      0.95, via='attenzione')
add('presenza', 'Causes',  'connessione',  0.90, via='contatto')
add('presenza', 'Causes',  'relazione',    0.90, via='riconoscimento')
add('presenza', 'Does',    'testimoniare', 0.85)
add('presenza', 'PartOf',  'coscienza',    0.90)
add('presenza', 'Requires','corpo',        0.90)
add('presenza', 'Requires','attenzione',   0.90)

# ═════════════════════════════════════════════════════════════════════════════
# RELAZIONE
# ═════════════════════════════════════════════════════════════════════════════
print("=== RELAZIONE ===")

remove('relazione', 'IsA', 'societa')      # la relazione non è un tipo di società
remove('relazione', 'SimilarTo', 'contatto')  # troppo bassa, 0.55

add('relazione', 'Has',    'direzione',    0.85)
add('relazione', 'Has',    'forza',        0.85)
add('relazione', 'Causes', 'comprensione', 0.90, via='scambio')
add('relazione', 'Causes', 'cambiamento',  0.85, via='incontro')
add('relazione', 'Causes', 'identita',     0.85, via='riconoscimento')
add('relazione', 'Does',   'connettere',   0.95)
add('relazione', 'Does',   'trasformare',  0.80, via='incontro')
add('relazione', 'PartOf', 'linguaggio',   0.90, via='dialogo')
add('relazione', 'PartOf', 'societa',      0.90)
add('relazione', 'Requires','due',         0.95)
add('relazione', 'Requires','presenza',    0.90, via='altro')

# ═════════════════════════════════════════════════════════════════════════════
# STRUTTURA
# ═════════════════════════════════════════════════════════════════════════════
print("=== STRUTTURA ===")

add('struttura', 'Has',    'ordine',       0.95)
add('struttura', 'Has',    'confine',      0.90)
add('struttura', 'Has',    'relazione',    0.90, via='forma')
add('struttura', 'Causes', 'comprensione', 0.85, via='forma')
add('struttura', 'Causes', 'stabilita',    0.90, via='ordine')
add('struttura', 'Does',   'organizzare',  0.95, via='forma')
add('struttura', 'Does',   'contenere',    0.85)
add('struttura', 'PartOf', 'sistema',      0.90, via='regola')
add('struttura', 'PartOf', 'linguaggio',   0.90)
add('struttura', 'Requires','relazione',   0.95)
add('struttura', 'Requires','confine',     0.90)

# ═════════════════════════════════════════════════════════════════════════════
# CAMBIAMENTO
# ═════════════════════════════════════════════════════════════════════════════
print("=== CAMBIAMENTO ===")

strength('cambiamento', 'Requires', 'volonta', 0.70)  # il cambiamento non richiede sempre volontà

add('cambiamento', 'Causes', 'perdita',     0.85, via='fine')
add('cambiamento', 'Causes', 'nascita',     0.85, via='inizio')
add('cambiamento', 'Causes', 'adattamento', 0.90)
add('cambiamento', 'Has',    'direzione',   0.85)
add('cambiamento', 'PartOf', 'vita',        0.90, via='tempo')
add('cambiamento', 'Requires','tempo',      0.95)
add('cambiamento', 'Requires','energia',    0.90)

# ═════════════════════════════════════════════════════════════════════════════
# DIVENIRE
# ═════════════════════════════════════════════════════════════════════════════
print("=== DIVENIRE ===")

strength('divenire', 'SimilarTo', 'trasformarsi', 0.85)

add('divenire', 'IsA',    'processo',      0.95, via='tempo')
add('divenire', 'IsA',    'cambiamento',   0.90)
add('divenire', 'Has',    'direzione',     0.90)
add('divenire', 'Has',    'tensione',      0.85, via='movimento')
add('divenire', 'Causes', 'forma',         0.85, via='struttura')
add('divenire', 'Causes', 'identita',      0.85, via='tempo')
add('divenire', 'Does',   'trasformare',   0.90, via='tempo')
add('divenire', 'PartOf', 'vita',          0.95)
add('divenire', 'Requires','tempo',        0.95)
add('divenire', 'Requires','movimento',    0.90)
add('divenire', 'OppositeOf','essere',     0.85)

# ═════════════════════════════════════════════════════════════════════════════
# VISIONE ROVELLIANA: RELAZIONE, TEMPO E SPAZIO (MQR & LQG)
# ═════════════════════════════════════════════════════════════════════════════
print("=== ONTOLOGIA RELAZIONALE (ROVELLI) ===")

# RELAZIONE (Il fondamento della Meccanica Quantistica Relazionale)
remove('relazione', 'SimilarTo', 'parentela') # troppo umano/sociale
add('relazione', 'Causes', 'esistenza',    0.95, via='fisica')
add('relazione', 'Causes', 'realtà',       0.95, via='interazione')
add('relazione', 'IsA',    'fondamento',   0.90, via='ontologia')
add('relazione', 'Has',    'informazione', 0.85, via='scambio')

# TEMPO (Il tempo termico / emergente)
remove('tempo', 'IsA', 'dimensione')     # In LQG il tempo non è una dimensione di base
remove('tempo', 'Has', 'scorrere')       # L'illusione dello scorrere
add('tempo', 'IsA',    'illusione',      0.80, via='entropia')
add('tempo', 'IsA',    'emergenza',      0.90, via='termodinamica')
add('tempo', 'Causes', 'divenire',       0.85, via='cambiamento')
add('tempo', 'Requires','calore',        0.80, via='entropia')
add('tempo', 'OppositeOf','eternità',    0.85)

# SPAZIO (Gravità Quantistica a Loop / Reti di spin)
remove('spazio', 'IsA', 'contenitore')   # Lo spazio non è una scatola
remove('spazio', 'Has', 'vuoto')         # Non c'è vuoto continuo
add('spazio', 'IsA',    'rete',          0.90, via='relazione')
add('spazio', 'Has',    'nodi',          0.85, via='loop')
add('spazio', 'Has',    'quanti',        0.85, via='granularità')
add('spazio', 'Causes', 'distanza',      0.80, via='interazione')

# REALTÀ / MONDO (Eventi, non cose)
add('realtà', 'IsA',    'rete',          0.90, via='interazione')
add('realtà', 'Has',    'eventi',        0.95, via='processo')
add('realtà', 'OppositeOf','oggetto',    0.80, via='fissità')

# ═════════════════════════════════════════════════════════════════════════════
# Salva
# ═════════════════════════════════════════════════════════════════════════════
kg['edges'] = list(edge_map.values())
new_count = len(kg['edges'])

with open(KG_PATH, 'w', encoding='utf-8') as f:
    json.dump(kg, f, ensure_ascii=False, indent=None, separators=(',', ':'))

print(f"\n{'='*60}")
print(f"RIMOSSI:  {len(removed)}")
for e in removed:
    print(f"  - {e['subject']} -{e['relation']}-> {e['object']}")
print(f"\nAGGIUNTI: {len(added)}")
for a in added:
    print(f"  + {a}")
print(f"\nMODIFICATI: {len(changed)}")
for c in changed:
    print(f"  ~ {c}")
print(f"\nTotale archi: {original_count} -> {new_count} ({new_count - original_count:+d})")
