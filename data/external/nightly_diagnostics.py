#!/usr/bin/env python3
"""
nightly_diagnostics.py — Diagnostica notturna del KG di Prometeo.

Analizza la struttura del KG e produce un report con:
1. Distribuzione gradi (top hub nodes da smorzare)
2. Nodi orfani (senza archi) — candidati per arricchimento
3. Componenti disconnesse
4. Statistiche confidence per tipo relazione
5. Candidati per nuovi archi (parole nel lessico senza KG entry)

Uso:
  python nightly_diagnostics.py                    # report completo
  python nightly_diagnostics.py --output report.md # salva su file
"""

import json
import argparse
import sys
from pathlib import Path
from collections import defaultdict, Counter

BASE_DIR = Path(__file__).parent.parent.parent
KG_JSON  = BASE_DIR / "prometeo_kg.json"


def load_kg(path: Path):
    with open(path) as f:
        return json.load(f)


def analyze_degrees(data):
    """Analizza distribuzione gradi."""
    out_deg = Counter()
    in_deg = Counter()
    for e in data["edges"]:
        out_deg[e["subject"]] += 1
        in_deg[e["object"]] += 1

    total_deg = Counter()
    all_nodes = set(out_deg.keys()) | set(in_deg.keys())
    for n in all_nodes:
        total_deg[n] = out_deg[n] + in_deg[n]

    return out_deg, in_deg, total_deg


def analyze_confidence(data):
    """Statistiche confidence per tipo relazione."""
    by_type = defaultdict(list)
    for e in data["edges"]:
        by_type[e["relation"]].append(e.get("confidence", 1.0))
    return by_type


def find_orphans(data):
    """Nodi connessi solo con SIMILAR_TO ma senza IS_A."""
    has_isa = set()
    all_nodes = set()
    for e in data["edges"]:
        all_nodes.add(e["subject"])
        all_nodes.add(e["object"])
        if e["relation"] == "IsA":
            has_isa.add(e["subject"])
    return all_nodes - has_isa


def connected_components(data):
    """Componenti connesse (union-find semplice)."""
    parent = {}
    def find(x):
        while parent.get(x, x) != x:
            parent[x] = parent.get(parent[x], parent[x])
            x = parent[x]
        return x
    def union(a, b):
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[ra] = rb

    for e in data["edges"]:
        union(e["subject"], e["object"])

    components = defaultdict(set)
    all_nodes = set()
    for e in data["edges"]:
        all_nodes.add(e["subject"])
        all_nodes.add(e["object"])
    for n in all_nodes:
        components[find(n)].add(n)

    return list(components.values())


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=str, default=None)
    args = parser.parse_args()

    lines = []
    def p(s=""):
        lines.append(s)
        print(s)

    p("# Prometeo KG — Diagnostica Notturna")
    p()

    data = load_kg(KG_JSON)
    n_edges = len(data["edges"])
    n_nodes = len(set(e["subject"] for e in data["edges"]) |
                   set(e["object"] for e in data["edges"]))

    p(f"## Panoramica")
    p(f"- Archi: {n_edges:,}")
    p(f"- Nodi: {n_nodes:,}")
    p()

    # Gradi
    out_deg, in_deg, total_deg = analyze_degrees(data)
    p("## Top 20 Hub (grado totale)")
    p("| Nodo | Grado | Out | In |")
    p("|------|-------|-----|-----|")
    for node, deg in total_deg.most_common(20):
        p(f"| {node} | {deg} | {out_deg[node]} | {in_deg[node]} |")
    p()

    # Distribuzione gradi
    degrees = sorted(total_deg.values())
    if degrees:
        median = degrees[len(degrees) // 2]
        p90 = degrees[int(len(degrees) * 0.9)]
        p99 = degrees[int(len(degrees) * 0.99)]
        p(f"## Distribuzione Gradi")
        p(f"- Mediana: {median}")
        p(f"- P90: {p90}")
        p(f"- P99: {p99}")
        p(f"- Max: {degrees[-1]}")
        p(f"- Nodi con grado > 100: {sum(1 for d in degrees if d > 100)}")
        p()

    # Confidence
    conf_by_type = analyze_confidence(data)
    p("## Confidence per Tipo Relazione")
    p("| Tipo | N archi | Media | Min | % default (1.0) |")
    p("|------|---------|-------|-----|-----------------|")
    for rel, confs in sorted(conf_by_type.items()):
        avg = sum(confs) / len(confs)
        mn = min(confs)
        pct_default = sum(1 for c in confs if c >= 0.99) / len(confs) * 100
        p(f"| {rel} | {len(confs):,} | {avg:.3f} | {mn:.2f} | {pct_default:.0f}% |")
    p()

    # Orfani IS_A
    orphans = find_orphans(data)
    p(f"## Nodi senza IS_A: {len(orphans):,}")
    if orphans:
        sample = sorted(orphans)[:30]
        p(f"Campione: {', '.join(sample)}")
    p()

    # Componenti connesse
    components = connected_components(data)
    p(f"## Componenti Connesse: {len(components)}")
    if len(components) > 1:
        sizes = sorted([len(c) for c in components], reverse=True)
        p(f"  Dimensioni: {sizes[:10]}{'...' if len(sizes) > 10 else ''}")
    p()

    if args.output:
        Path(args.output).write_text("\n".join(lines), encoding="utf-8")
        print(f"\nReport salvato in: {args.output}")


if __name__ == "__main__":
    main()
