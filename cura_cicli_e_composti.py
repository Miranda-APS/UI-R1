#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
cura_cicli_e_composti.py — curation idempotente del kg_sem (prometeo_kg.json).

Risolve gli errori del gate `kg_lint.py`:
  P1 (E6) — spezza i 6 cicli IsA (la tassonomia deve essere un DAG)
  P2 (E1) — scompone/rimuove i nodi NON atomici (underscore)
  P3       — aggiunge supertipi mancanti a verbi comuni (additivo)
  P4 (E4/E3) — rimuove duplicati esatti e self-loop (generico)

NON tocca:
  - i W1 (classi grammaticali nel kg_sem) — passata a priorità più bassa
  - i nodi con apostrofo eliso (senz'altro, all'incirca, d'oro): il lint NON
    li flagga (E1 controlla solo spazi/underscore), quindi sono fuori scopo

Idempotente: rilanciarlo non rompe nulla. Il backup viene creato UNA sola volta
(non sovrascrive un backup esistente, per preservare l'originale tra i run).

Uso:
  python cura_cicli_e_composti.py            # applica e salva
  python cura_cicli_e_composti.py --dry-run  # mostra cosa farebbe, non salva
"""

import json
import sys
from collections import defaultdict
from pathlib import Path

KG = Path(__file__).with_name("prometeo_kg.json")
BACKUP = Path(__file__).with_name("prometeo_kg.json.bak_cura")

DRY = "--dry-run" in sys.argv


def norm(s):
    return str(s or "").strip().lower()


def key_of(e, with_via=True):
    """Chiave identità di un arco, coerente con kg_lint (case-insensitive)."""
    k = (norm(e.get("subject")), str(e.get("relation", "")).strip(),
         norm(e.get("object")))
    if with_via:
        k = k + (norm(e.get("via")),)
    return k


# ─────────────────────────────────────────────────────────────────────────
# P1 — CICLI IsA: archi IsA da rimuovere (subject, object), in minuscolo.
# Un arco si rimuove solo se relation == "IsA" e subject/object combaciano.
# Per ognuno: il PERCHÉ.
# ─────────────────────────────────────────────────────────────────────────
CYCLE_REMOVE_ISA = [
    # SCC grande (cicli 1 e 2): evento→storia→cultura→conoscenza→processo→
    # sviluppo→azione→evento + sotto-cicli. Tre tagli rendono il cluster un DAG.
    ("cultura", "conoscenza",
     "cultura NON è una specie di conoscenza (è più ampia: pratiche, valori, "
     "arte). Il legame resta come `cultura SimilarTo conoscenza` (già presente). "
     "Taglio indicato dal curatore come dubbio."),
    ("processo", "sviluppo",
     "back-edge: il processo è PIÙ GENERALE dello sviluppo (lo sviluppo è una "
     "specie di cambiamento, che è un processo). `processo SimilarTo sviluppo` "
     "resta. Rompe processo→sviluppo→cambiamento→processo."),
    ("evento", "storia",
     "invertito: la storia CONTIENE eventi, un evento non è una specie di "
     "storia. Rompe evento→storia→processo→evento. `storia IsA processo` resta."),

    # Ciclo 4: conflitto↔scontro. Tieni scontro→conflitto (corretto), togli l'altro.
    ("conflitto", "scontro",
     "lo scontro è una specie di conflitto, non viceversa. `scontro IsA "
     "conflitto` (corretto) resta; `conflitto SimilarTo scontro` resta."),

    # Ciclo 5: linguaggio→geometria→matematica→linguaggio.
    ("linguaggio", "geometria",
     "anello più debole: il linguaggio NON è una specie di geometria "
     "(metafora). `geometria IsA matematica` (corretto) e `matematica IsA "
     "linguaggio` (difendibile, linguaggio formale) restano. linguaggio resta "
     "IsA struttura/sistema."),

    # Ciclo 6: gruppo↔insieme. Tieni gruppo→insieme (un gruppo è un insieme).
    ("insieme", "gruppo",
     "un GRUPPO è una specie di insieme, non viceversa. `gruppo IsA insieme` "
     "resta; rimuovo la direzione inversa."),
]


# ─────────────────────────────────────────────────────────────────────────
# P2 — NODI COMPOSTI (underscore). Per ognuno: azione precisa.
#   ("redirect_obj", k_match, new_object, new_via)  → cambia object (+via)
#   ("redirect_subj", k_match, new_subject)         → cambia subject
#   ("remove", k_match)                             → rimuove l'arco
# k_match = (subject_lower, relation, object_lower, via_lower) come da key_of.
# La via_lower originale è "" se assente.
# ─────────────────────────────────────────────────────────────────────────
COMPOSITE_OPS = [
    # melos IsA elemento_musicale → melos IsA elemento (il senso musicale è
    # già in `melos SimilarTo melodia`).
    ("redirect_obj", ("melos", "IsA", "elemento_musicale", ""),
     "elemento", "musica",
     "elemento_musicale → object atomico `elemento`, qualificatore `musica` in via."),

    # intelligenza UsedFor risolvere_problemi → UsedFor risolvere via=problema
    ("redirect_obj", ("intelligenza", "UsedFor", "risolvere_problemi", ""),
     "risolvere", "problema",
     "risolvere_problemi → verbo atomico `risolvere`, qualificatore `problema` in via."),
    # intelligenza Enables risolvere_problemi → Enables risolvere via=problema
    ("redirect_obj", ("intelligenza", "Enables", "risolvere_problemi", ""),
     "risolvere", "problema",
     "risolvere_problemi → verbo atomico `risolvere`, qualificatore `problema` in via."),
    # ia UsedFor risolvere_problemi → UsedFor risolvere via=problema
    ("redirect_obj", ("ia", "UsedFor", "risolvere_problemi", ""),
     "risolvere", "problema",
     "risolvere_problemi → verbo atomico `risolvere`, qualificatore `problema` in via."),
    # intelligenza UsedFor capire_il_mondo → UsedFor capire via=mondo
    ("redirect_obj", ("intelligenza", "UsedFor", "capire_il_mondo", ""),
     "capire", "mondo",
     "capire_il_mondo → verbo atomico `capire`, qualificatore `mondo` in via."),

    # capire_l_altro IsA empatia → RIMUOVI (subject non atomizzabile;
    # ridondante: empatia Has/SimilarTo comprensione già presenti).
    ("remove", ("capire_l_altro", "IsA", "empatia", ""),
     "capire_l_altro non atomizzabile; ridondante con empatia↔comprensione."),

    # torrente IsA corso_d_acqua → torrente IsA corso via=acqua
    ("redirect_obj", ("torrente", "IsA", "corso_d_acqua", ""),
     "corso", "acqua",
     "corso_d_acqua → object atomico `corso`, qualificatore `acqua` in via."),

    # ia SimilarTo sistema_esperto → ia SimilarTo sistema
    ("redirect_obj", ("ia", "SimilarTo", "sistema_esperto", ""),
     "sistema", None,
     "sistema_esperto → `sistema` (la specificità 'esperto' è ridondante col soggetto `ia`)."),

    # ia Causes errore_sistematico → Causes errore via=sistematicità
    ("redirect_obj", ("ia", "Causes", "errore_sistematico", ""),
     "errore", "sistematicità",
     "errore_sistematico → object atomico `errore`, qualificatore in via."),

    # ia Expresses volontà_umana → Expresses volontà via=uomo
    ("redirect_obj", ("ia", "Expresses", "volontà_umana", ""),
     "volontà", "uomo",
     "volontà_umana → object atomico `volontà`, qualificatore `uomo` in via."),

    # emozione SimilarTo stato_emotivo → RIMUOVI (tautologia: 'stato emotivo'
    # ripete il soggetto `emozione`).
    ("remove", ("emozione", "SimilarTo", "stato_emotivo", ""),
     "stato_emotivo è tautologico rispetto a `emozione`; rimosso."),

    # sentire_con IsA empatia → RIMUOVI (subject non atomizzabile; 'sentire con'
    # = empatia, ridondante).
    ("remove", ("sentire_con", "IsA", "empatia", ""),
     "sentire_con non atomizzabile; ridondante con empatia."),

    # artificiale UsedFor imitare_la_natura → UsedFor imitare via=natura
    ("redirect_obj", ("artificiale", "UsedFor", "imitare_la_natura", ""),
     "imitare", "natura",
     "imitare_la_natura → verbo atomico `imitare`, qualificatore `natura` in via."),

    # prospettiva Has punto_di_vista → RIMUOVI (tautologico: 'punto di vista' =
    # prospettiva; gli atomi visione/angolazione sono già in SimilarTo).
    ("remove", ("prospettiva", "Has", "punto_di_vista", ""),
     "punto_di_vista è sinonimo di `prospettiva`; ridondante con SimilarTo visione/angolazione."),

    # innocenza Requires non_conoscenza → innocenza Excludes conoscenza
    ("redirect_obj_rel", ("innocenza", "Requires", "non_conoscenza", ""),
     "Excludes", "conoscenza", None,
     "non_conoscenza (negazione) → relazione atomica `Excludes conoscenza`."),

    # fatica_cognitiva (subject, 8 archi) → fatica  (mappa tutto su `fatica`)
    ("redirect_subj", ("fatica_cognitiva", "Does", "consumare", "risorsa"), "fatica",
     "fatica_cognitiva → `fatica` (fatica mentale = fatica; gli archi sopravvivono su fatica)."),
    ("redirect_subj", ("fatica_cognitiva", "Causes", "esaurimento", "energia"), "fatica",
     "fatica_cognitiva → `fatica`."),
    ("redirect_subj", ("fatica_cognitiva", "Causes", "blocco", "mente"), "fatica",
     "fatica_cognitiva → `fatica`."),
    ("redirect_subj", ("fatica_cognitiva", "OppositeOf", "lucidità", ""), "fatica",
     "fatica_cognitiva → `fatica`."),
    ("redirect_subj", ("fatica_cognitiva", "Requires", "silenzio", ""), "fatica",
     "fatica_cognitiva → `fatica`."),
    ("redirect_subj", ("fatica_cognitiva", "Requires", "riposo", ""), "fatica",
     "fatica_cognitiva → `fatica`."),
    ("redirect_subj", ("fatica_cognitiva", "Has", "sovraccarico", ""), "fatica",
     "fatica_cognitiva → `fatica`."),
    ("redirect_subj", ("fatica_cognitiva", "IsA", "limite", "corpo"), "fatica",
     "fatica_cognitiva → `fatica`."),
    # fatica_cognitiva come OBJECT
    ("redirect_obj", ("adhd", "Causes", "fatica_cognitiva", "sovraccarico"),
     "fatica", "__keep_via__",
     "fatica_cognitiva (object) → `fatica` (via originale conservata)."),
    ("redirect_obj", ("lucidità", "OppositeOf", "fatica_cognitiva", ""),
     "fatica", None,
     "fatica_cognitiva (object) → `fatica`."),
]


# ─────────────────────────────────────────────────────────────────────────
# P3 — supertipi mancanti (additivo, idempotente). Solo questi due: gli altri
# verbi nominati dal curatore (servire/bastare/nascere/litigare/mancare)
# HANNO GIÀ `IsA azione`; aggiungerne altri sarebbe dead-weight (Principio 7).
# ─────────────────────────────────────────────────────────────────────────
P3_ADD = [
    {"subject": "piovere", "relation": "IsA", "object": "evento",
     "confidence": 0.9, "source": "Curated"},
    {"subject": "succedere", "relation": "IsA", "object": "accadimento",
     "confidence": 0.9, "source": "Curated"},
]


def detect_isa_cycles(edges):
    """Stessa logica di kg_lint E6: DFS con colori, ritorna lista di cicli."""
    isa = defaultdict(set)
    for e in edges:
        if str(e.get("relation", "")).strip() == "IsA":
            isa[norm(e.get("subject"))].add(norm(e.get("object")))
    WHITE, GRAY, BLACK = 0, 1, 2
    color = defaultdict(int)
    cycles = []
    for start in list(isa.keys()):
        if color[start] != WHITE:
            continue
        stack = [(start, iter(sorted(isa.get(start, ()))))]
        color[start] = GRAY
        path_nodes = [start]
        while stack:
            node, it = stack[-1]
            advanced = False
            for nxt in it:
                if color[nxt] == GRAY:
                    cyc = (path_nodes[path_nodes.index(nxt):] + [nxt]
                           if nxt in path_nodes else [node, nxt])
                    cycles.append(" → ".join(cyc))
                elif color[nxt] == WHITE:
                    color[nxt] = GRAY
                    stack.append((nxt, iter(sorted(isa.get(nxt, ())))))
                    path_nodes.append(nxt)
                    advanced = True
                    break
            if not advanced:
                color[node] = BLACK
                stack.pop()
                if path_nodes and path_nodes[-1] == node:
                    path_nodes.pop()
    return cycles


def main():
    data = json.loads(KG.read_text(encoding="utf-8"))
    edges = data["edges"]
    n0 = len(edges)
    log = []

    # backup una sola volta
    if not BACKUP.exists():
        if not DRY:
            BACKUP.write_text(KG.read_text(encoding="utf-8"), encoding="utf-8")
        log.append(f"[backup] creato {BACKUP.name} ({n0} archi)")
    else:
        log.append(f"[backup] {BACKUP.name} già esistente — non sovrascritto")

    # indice per chiave-completa → lista indici (per match preciso)
    def find(km):
        out = []
        for i, e in enumerate(edges):
            if key_of(e) == km:
                out.append(i)
        return out

    to_remove = set()

    # ── P1: cicli IsA ──
    p1_done = 0
    for subj, obj, why in CYCLE_REMOVE_ISA:
        hit = False
        for i, e in enumerate(edges):
            if (str(e.get("relation", "")).strip() == "IsA"
                    and norm(e.get("subject")) == subj
                    and norm(e.get("object")) == obj
                    and i not in to_remove):
                to_remove.add(i)
                hit = True
                p1_done += 1
                log.append(f"[P1 ciclo] RIMOSSO  {subj} IsA {obj}  — {why}")
                break
        if not hit:
            log.append(f"[P1 ciclo] (già assente) {subj} IsA {obj}")

    # ── P2: nodi composti ──
    p2_remove = p2_redirect = 0
    for op in COMPOSITE_OPS:
        kind = op[0]
        if kind == "remove":
            _, km, why = op
            idxs = find(km)
            if idxs:
                for i in idxs:
                    to_remove.add(i)
                p2_remove += 1
                log.append(f"[P2 composto] RIMOSSO  {km[0]} {km[1]} {km[2]}  — {why}")
            else:
                log.append(f"[P2 composto] (già assente) {km[0]} {km[1]} {km[2]}")
        elif kind == "redirect_obj":
            _, km, new_obj, new_via, why = op
            idxs = find(km)
            if idxs:
                e = edges[idxs[0]]
                e["object"] = new_obj
                if new_via == "__keep_via__":
                    pass  # conserva la via esistente
                elif new_via is None:
                    e.pop("via", None)
                else:
                    e["via"] = new_via
                p2_redirect += 1
                via_s = e.get("via", "")
                log.append(f"[P2 composto] REDIRECT {km[0]} {km[1]} {km[2]} → "
                           f"{new_obj}" + (f" via={via_s}" if via_s else "") + f"  — {why}")
            else:
                log.append(f"[P2 composto] (già assente/curato) {km[0]} {km[1]} {km[2]}")
        elif kind == "redirect_obj_rel":
            _, km, new_rel, new_obj, new_via, why = op
            idxs = find(km)
            if idxs:
                e = edges[idxs[0]]
                e["relation"] = new_rel
                e["object"] = new_obj
                if new_via is None:
                    e.pop("via", None)
                else:
                    e["via"] = new_via
                p2_redirect += 1
                log.append(f"[P2 composto] REDIRECT {km[0]} {km[1]} {km[2]} → "
                           f"{new_rel} {new_obj}  — {why}")
            else:
                log.append(f"[P2 composto] (già assente/curato) {km[0]} {km[1]} {km[2]}")
        elif kind == "redirect_subj":
            _, km, new_subj, why = op
            idxs = find(km)
            if idxs:
                edges[idxs[0]]["subject"] = new_subj
                p2_redirect += 1
                log.append(f"[P2 composto] REDIRECT subj {km[0]} → {new_subj} "
                           f"({km[1]} {km[2]})  — {why}")
            else:
                log.append(f"[P2 composto] (già assente/curato) subj {km[0]} {km[1]} {km[2]}")

    # applica le rimozioni P1+P2
    edges = [e for i, e in enumerate(edges) if i not in to_remove]

    # ── P4: self-loop (E3) e duplicati esatti (E4), generico ──
    p4_self = p4_dup = 0
    seen = set()
    kept = []
    for e in edges:
        s, o = norm(e.get("subject")), norm(e.get("object"))
        if s == o:
            p4_self += 1
            log.append(f"[P4 self-loop] RIMOSSO {s} {e.get('relation')} {o}")
            continue
        k = key_of(e)
        if k in seen:
            p4_dup += 1
            via_s = e.get("via", "")
            log.append(f"[P4 duplicato] RIMOSSO {k[0]} {k[1]} {k[2]}"
                       + (f" via={via_s}" if via_s else ""))
            continue
        seen.add(k)
        kept.append(e)
    edges = kept

    # ── P3: supertipi mancanti (additivo, idempotente) ──
    p3_add = 0
    existing = {key_of(e) for e in edges}
    for new in P3_ADD:
        k = key_of(new)
        if k in existing:
            log.append(f"[P3 supertipo] (già presente) {new['subject']} IsA {new['object']}")
            continue
        edges.append(new)
        existing.add(k)
        p3_add += 1
        log.append(f"[P3 supertipo] AGGIUNTO {new['subject']} IsA {new['object']}")

    data["edges"] = edges
    n1 = len(edges)

    # ── verifica cicli residui (stessa logica di kg_lint E6) ──
    residual = detect_isa_cycles(edges)

    # ── salva ──
    if not DRY:
        KG.write_text(json.dumps(data, ensure_ascii=False, indent=0) + "\n",
                      encoding="utf-8")

    # ── report ──
    print("\n".join(log))
    print("\n" + "=" * 70)
    print(f"{'[DRY-RUN] ' if DRY else ''}CURATION COMPLETATA")
    print(f"  archi: {n0} → {n1}  (Δ {n1 - n0})")
    print(f"  P1 cicli IsA rotti        : {p1_done}")
    print(f"  P2 composti rimossi       : {p2_remove}")
    print(f"  P2 composti rediretti     : {p2_redirect}")
    print(f"  P3 supertipi aggiunti     : {p3_add}")
    print(f"  P4 self-loop rimossi      : {p4_self}")
    print(f"  P4 duplicati rimossi      : {p4_dup}")
    print(f"  CICLI IsA RESIDUI         : {len(residual)}")
    for c in residual:
        print(f"     ⚠ {c}")
    if DRY:
        print("  (nessuna scrittura — rimuovi --dry-run per applicare)")


if __name__ == "__main__":
    main()
