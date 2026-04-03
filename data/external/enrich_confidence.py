#!/usr/bin/env python3
"""
enrich_confidence.py — Arricchisce la confidence per-arco nel KG di Prometeo.

Phase 48: ogni relazione deve avere peso individuale, non costante per tipo.
Questo script legge gli archi con confidence=1.0 (default) e chiede a Qwen3
di stimare quanto forte/specifica sia ogni relazione su scala 0.0-1.0.

Esempio:
  "cane IS_A animale" → 0.95 (molto specifico, diretto)
  "cosa IS_A essere" → 0.30 (troppo generico, poco informativo)
  "fuoco CAUSES calore" → 0.90 (causale forte)
  "pioggia CAUSES crescita" → 0.55 (causale indiretta)

Output: data/kg/enriched_confidence.tsv (soggetto\tREL\toggetto\tconfidence)
Può essere ri-importato con import-kg (il parser accetta la 4a colonna confidence).

Uso:
  python enrich_confidence.py                  # processa tutto (ore)
  python enrich_confidence.py --test 50        # test su 50 archi
  python enrich_confidence.py --type IS_A      # solo un tipo di relazione
  python enrich_confidence.py --resume         # riprendi da dove si era fermato
"""

import json
import requests
import time
import argparse
import re
import sys
from pathlib import Path
from collections import defaultdict

# -- Configurazione ----------------------------------------------------------
OLLAMA_URL    = "http://localhost:11434/api/chat"
MODEL         = "qwen3"
BATCH_SIZE    = 10       # archi per prompt
DELAY_S       = 0.3
TIMEOUT_S     = 60

BASE_DIR    = Path(__file__).parent.parent.parent
KG_JSON     = BASE_DIR / "prometeo_kg.json"
OUTPUT_TSV  = BASE_DIR / "data" / "kg" / "enriched_confidence.tsv"
PROGRESS_F  = BASE_DIR / "data" / "kg" / "enrich_confidence_progress.txt"


def load_kg(path: Path):
    """Carica il KG e restituisce gli archi con confidence=1.0 (default)."""
    with open(path) as f:
        data = json.load(f)

    edges_to_enrich = []
    for e in data["edges"]:
        conf = e.get("confidence", 1.0)
        if conf >= 0.99:  # default o quasi
            edges_to_enrich.append((
                e["subject"].lower().strip(),
                e["relation"],
                e["object"].lower().strip(),
            ))
    return edges_to_enrich


def load_progress(path: Path) -> set:
    """Carica archi gia processati."""
    done = set()
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            parts = line.strip().split("\t")
            if len(parts) >= 3:
                done.add((parts[0], parts[1], parts[2]))
    return done


PROMPT_TEMPLATE = """/no_think
Sei un linguista computazionale italiano. Per ogni relazione semantica sotto,
stima quanto e' FORTE e SPECIFICA su scala 0.0-1.0.

Criteri:
- 0.90-1.00: relazione diretta, specifica, quasi definitoria ("cane IS_A animale")
- 0.70-0.89: relazione chiara ma non primaria ("cane IS_A essere_vivente")
- 0.50-0.69: relazione valida ma indiretta o generica ("cosa IS_A entita")
- 0.30-0.49: relazione debole o molto indiretta ("pioggia CAUSES felicita")
- 0.10-0.29: relazione marginale ("acqua SIMILAR_TO vino")

Rispondi SOLO con le righe nel formato: soggetto|relazione|oggetto|confidence
Nessuna spiegazione, nessun commento.

Relazioni da valutare:
{edges}"""


def build_prompt(batch):
    """Costruisce il prompt per un batch di archi."""
    lines = []
    for subj, rel, obj in batch:
        # Converti RelationType a formato leggibile
        rel_str = rel.replace("IsA", "IS_A").replace("SimilarTo", "SIMILAR_TO") \
                     .replace("PartOf", "PART_OF").replace("OppositeOf", "OPPOSITE_OF") \
                     .replace("UsedFor", "USED_FOR")
        lines.append(f"{subj} {rel_str} {obj}")
    return PROMPT_TEMPLATE.format(edges="\n".join(lines))


def query_ollama(prompt: str) -> str:
    """Invia prompt a Ollama via chat API (think: false per Qwen3)."""
    try:
        resp = requests.post(OLLAMA_URL, json={
            "model": MODEL,
            "messages": [{"role": "user", "content": prompt}],
            "stream": False,
            "think": False,
            "options": {"temperature": 0.1, "num_predict": 512},
        }, timeout=TIMEOUT_S)
        resp.raise_for_status()
        return resp.json().get("message", {}).get("content", "")
    except Exception as e:
        print(f"  [ERRORE] Ollama: {e}", file=sys.stderr)
        return ""


def parse_response(text: str, batch):
    """Parsa la risposta e restituisce (subj, rel, obj, confidence)."""
    results = []
    for line in text.strip().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("|")
        if len(parts) < 4:
            continue
        try:
            conf = float(parts[3].strip())
            conf = max(0.05, min(1.0, conf))  # clamp
            subj = parts[0].strip().lower()
            rel = parts[1].strip()
            obj = parts[2].strip().lower()
            results.append((subj, rel, obj, conf))
        except ValueError:
            continue
    return results


def main():
    parser = argparse.ArgumentParser(description="Arricchisci confidence KG")
    parser.add_argument("--test", type=int, default=0, help="Limita a N archi")
    parser.add_argument("--type", type=str, default=None,
                        help="Filtra per tipo relazione (IS_A, SIMILAR_TO, ...)")
    parser.add_argument("--resume", action="store_true", help="Riprendi da progress")
    args = parser.parse_args()

    print(f"[enrich_confidence] Caricamento KG da {KG_JSON}...")
    edges = load_kg(KG_JSON)
    print(f"  {len(edges)} archi con confidence=1.0 da valutare")

    # Filtro per tipo se richiesto
    if args.type:
        type_map = {
            "IS_A": "IsA", "ISA": "IsA",
            "SIMILAR_TO": "SimilarTo", "SIMILARTO": "SimilarTo",
            "CAUSES": "Causes", "PART_OF": "PartOf", "PARTOF": "PartOf",
            "HAS": "Has", "DOES": "Does",
            "OPPOSITE_OF": "OppositeOf", "OPPOSITEOF": "OppositeOf",
            "USED_FOR": "UsedFor", "USEDFOR": "UsedFor",
        }
        target_rel = type_map.get(args.type.upper(), args.type)
        edges = [(s, r, o) for s, r, o in edges if r == target_rel]
        print(f"  Filtrato a {len(edges)} archi di tipo {args.type}")

    # Resume
    done = set()
    if args.resume:
        done = load_progress(PROGRESS_F)
        edges = [(s, r, o) for s, r, o in edges if (s, r, o) not in done]
        print(f"  Resume: {len(done)} gia' processati, {len(edges)} rimasti")

    if args.test > 0:
        edges = edges[:args.test]
        print(f"  Test mode: {len(edges)} archi")

    if not edges:
        print("  Nessun arco da processare. Fine.")
        return

    # Apri file output in append
    output_mode = "a" if args.resume and OUTPUT_TSV.exists() else "w"
    total_enriched = 0

    with open(OUTPUT_TSV, output_mode, encoding="utf-8") as f_out, \
         open(PROGRESS_F, "a", encoding="utf-8") as f_prog:

        for i in range(0, len(edges), BATCH_SIZE):
            batch = edges[i:i + BATCH_SIZE]
            prompt = build_prompt(batch)

            print(f"  Batch {i // BATCH_SIZE + 1}/{(len(edges) + BATCH_SIZE - 1) // BATCH_SIZE} "
                  f"({len(batch)} archi)...", end=" ", flush=True)

            response = query_ollama(prompt)
            results = parse_response(response, batch)

            for subj, rel, obj, conf in results:
                # Converti rel in formato TSV
                rel_tsv = rel.upper().replace(" ", "_")
                f_out.write(f"{subj}\t{rel_tsv}\t{obj}\t{conf:.2f}\n")
                total_enriched += 1

            # Segna come processati
            for s, r, o in batch:
                f_prog.write(f"{s}\t{r}\t{o}\n")

            print(f"ok ({len(results)} risultati)")
            time.sleep(DELAY_S)

    print(f"\n[enrich_confidence] Completato: {total_enriched} archi arricchiti")
    print(f"  Output: {OUTPUT_TSV}")
    print(f"  Per applicare: cargo run --release --bin import-kg && "
          f"cargo run --release --bin rebuild-semantic-topology")


if __name__ == "__main__":
    main()
