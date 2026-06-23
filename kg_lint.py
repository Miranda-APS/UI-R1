#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
kg_lint.py — il CONTRATTO DI CONTRIBUZIONE del kg_sem, reso macchina-verificabile.

È il gate che rende il KG semantico affidabile a una comunità: una contribuzione
che passa il lint (+ non regredisce il bench) è accettabile senza revisione
dell'architetto. Codifica gli invarianti dei "Principi Inviolabili" (CLAUDE.md)
e del contratto di strato (2026-06-10):

  IL kg_sem CONTIENE SOLO TRIPLE DI CONOSCENZA-MONDO FRA PAROLE-CONTENUTO.
  OGNI CONVENZIONE COMPORTAMENTALE VIVE NEL kg_proc.

ERRORI (exit 1 — la contribuzione non entra):
  E1  nodo non atomico (spazi/underscore: mai `pronome_interrogativo`)
  E2  relazione sconosciuta (fuori dalle 22 di RelationType)
  E3  self-loop (X rel X)
  E4  duplicato esatto (stesso subject+relation+object+via)
  E5  confidence fuori [0,1]
  E6  ciclo IsA (la tassonomia è un DAG: `a IsA b IsA a` rompe l'ereditarietà)

AVVISI (exit 0 — informativi, per la curation):
  W1  nodo di classe grammaticale nel kg_sem (pronome/articolo/copula/…):
      violazione di strato — appartiene al kg_proc
  W2  emozione (IsA emozione, diretto) senza marcatura di valenza
      (`Requires oggetto` = transitiva, assente = stato-completo):
      non è un errore, ma la scelta dev'essere CONSAPEVOLE — questo avviso
      è la discoverability che evita la cura ossessiva
  W3  nodo maiuscolo/non-normalizzato

Uso:
  python kg_lint.py                      # linta prometeo_kg.json
  python kg_lint.py path/al/kg.json      # linta un file specifico
  python kg_lint.py --quiet              # solo conteggi + exit code
"""

import json
import sys
from collections import defaultdict
from pathlib import Path

# Le 22 relazioni reali di src/topology/relation.rs — se ne aggiungi una in
# Rust, aggiungila qui (il lint deve fallire su relazioni inventate).
VALID_RELATIONS = {
    "IsA", "Has", "Does", "PartOf", "Causes", "Enables", "Requires",
    "TransformsInto", "SimilarTo", "OppositeOf", "UsedFor", "Expresses",
    "Symbolizes", "ContextOf", "FeelsAs", "WondersAbout", "RemembersAs",
    "Implies", "Equivalent", "Excludes", "Coexists", "DerivesFrom",
}

# Classi grammaticali: nodi del kg_proc che NON devono comparire nel kg_sem
# (violazione di strato). Lista = vocabolario delle classi del kg_proc.
GRAMMAR_CLASS_NODES = {
    "pronome", "articolo", "preposizione", "congiunzione", "marcatore",
    "copula", "ausiliare", "determinante", "interrogativo", "specificazione",
    "percettivo", "cognitivo", "comunicativo", "denominativo", "modale",
    "pronominale", "dativo", "avverbio", "interiezione", "pattern",
    "percetto", "subordinante",
}


def load_edges(path: Path):
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, dict) and "edges" in data:
        return data["edges"]
    raise SystemExit(f"{path}: formato inatteso (manca 'edges')")


def lint(path: Path, quiet: bool):
    edges = load_edges(path)
    errors, warnings = [], []

    seen = set()
    isa = defaultdict(set)  # per il check dei cicli
    emotion_nodes = set()
    valence_marked = set()

    for i, e in enumerate(edges):
        s = str(e.get("subject", "")).strip()
        r = str(e.get("relation", "")).strip()
        o = str(e.get("object", "")).strip()
        via = e.get("via")
        conf = e.get("confidence", 1.0)
        where = f"#{i} {s} {r} {o}" + (f" via={via}" if via else "")

        if not s or not o:
            errors.append(("E1", f"{where}: subject/object vuoto"))
            continue
        for node in (s, o):
            if " " in node or "_" in node:
                errors.append(("E1", f"{where}: nodo non atomico '{node}'"))
            if node != node.lower():
                warnings.append(("W3", f"{where}: nodo non normalizzato '{node}'"))
        if r not in VALID_RELATIONS:
            errors.append(("E2", f"{where}: relazione sconosciuta '{r}'"))
            continue
        if s.lower() == o.lower():
            errors.append(("E3", f"{where}: self-loop"))
        key = (s.lower(), r, o.lower(), (via or "").lower())
        if key in seen:
            errors.append(("E4", f"{where}: duplicato"))
        seen.add(key)
        try:
            c = float(conf)
            if not (0.0 <= c <= 1.0):
                errors.append(("E5", f"{where}: confidence {c} fuori [0,1]"))
        except (TypeError, ValueError):
            errors.append(("E5", f"{where}: confidence non numerica"))

        if r == "IsA":
            isa[s.lower()].add(o.lower())
            if o.lower() == "emozione":
                emotion_nodes.add(s.lower())
        if r == "Requires" and o.lower() == "oggetto":
            valence_marked.add(s.lower())

        # W1 — strato: classi grammaticali nel grafo del mondo.
        for node in (s.lower(), o.lower()):
            if node in GRAMMAR_CLASS_NODES:
                warnings.append(("W1", f"{where}: '{node}' è una classe grammaticale — appartiene al kg_proc"))

    # E6 — cicli IsA (DFS iterativo con colori).
    WHITE, GRAY, BLACK = 0, 1, 2
    color = defaultdict(int)
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
                    cycle = path_nodes[path_nodes.index(nxt):] + [nxt] if nxt in path_nodes else [node, nxt]
                    errors.append(("E6", f"ciclo IsA: {' → '.join(cycle)}"))
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

    # W2 — discoverability della valenza emotiva (eredità inclusa: marcato se
    # lui o un antenato IsA porta Requires oggetto).
    def inherited(n, depth=0):
        if n in valence_marked:
            return True
        if depth >= 2:
            return False
        return any(inherited(p, depth + 1) for p in isa.get(n, ()))

    unmarked = sorted(n for n in emotion_nodes if not inherited(n))
    if unmarked:
        warnings.append(("W2",
            f"{len(unmarked)} emozioni senza marcatura di valenza (né ereditata): "
            + ", ".join(unmarked[:20]) + (" …" if len(unmarked) > 20 else "")
            + "  [transitiva? aggiungi `X Requires oggetto`; stato-completo? ok così]"))

    # ── Report ──
    print(f"kg_lint — {path}: {len(edges)} archi")
    if not quiet:
        for code, msg in errors[:60]:
            print(f"  ✗ {code}  {msg}")
        if len(errors) > 60:
            print(f"  … e altri {len(errors) - 60} errori")
        for code, msg in warnings[:40]:
            print(f"  ⚠ {code}  {msg}")
        if len(warnings) > 40:
            print(f"  … e altri {len(warnings) - 40} avvisi")
    print(f"ERRORI: {len(errors)}   AVVISI: {len(warnings)}")
    return 1 if errors else 0


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    quiet = "--quiet" in sys.argv
    target = Path(args[0]) if args else Path("prometeo_kg.json")
    sys.exit(lint(target, quiet))
