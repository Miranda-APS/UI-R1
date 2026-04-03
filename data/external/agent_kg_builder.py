#!/usr/bin/env python3
"""
agent_kg_builder.py — Arricchisce il KG di Prometeo con relazioni IS_A
validate da Qwen 3 via Ollama.

Strategia:
  1. Inheritance: word senza IS_A ma con vicino SIMILAR_TO che ha IS_A
     -> Qwen conferma se l'eredita' e' corretta
  2. Direct: word senza IS_A e senza vicini IS_A
     -> Qwen assegna categoria semantica diretta

Output: data/kg/agent_kg.tsv (stesso formato di italian_core.tsv)

Uso:
  python agent_kg_builder.py               # processa tutto (ore)
  python agent_kg_builder.py --test 20     # test su 20 parole
  python agent_kg_builder.py --test 20 --mode direct
  python agent_kg_builder.py --test 20 --mode inheritance
"""

import json
import requests
import time
import argparse
import sys
import unicodedata
from collections import defaultdict
from pathlib import Path

# -- Configurazione ----------------------------------------------------------
OLLAMA_URL    = "http://localhost:11434/api/generate"
MODEL         = "qwen3"
CONFIDENCE_TH = 0.72
BATCH_SIZE    = 8
DELAY_S       = 0.2

BASE_DIR    = Path(__file__).parent.parent.parent  # data/external -> data -> prometeo_standalone
KG_JSON     = BASE_DIR / "prometeo_kg.json"
OUTPUT_TSV  = BASE_DIR / "data" / "kg" / "agent_kg.tsv"
PROGRESS_F  = BASE_DIR / "data" / "kg" / "agent_kg_progress.txt"


# -- Caricamento KG ----------------------------------------------------------
def load_kg(path: Path):
    with open(path) as f:
        data = json.load(f)

    is_a_out    = defaultdict(list)  # word -> [categorie]
    similar_map = defaultdict(set)   # word -> {simili}
    existing    = set()              # (subj, REL, obj) gia' presenti

    for e in data["edges"]:
        subj = e["subject"].lower().strip()
        obj  = e["object"].lower().strip()
        rel  = e["relation"]
        existing.add((subj, rel.upper(), obj))
        if rel == "IsA":
            is_a_out[subj].append(obj)
        if rel == "SimilarTo":
            similar_map[subj].add(obj)
            similar_map[obj].add(subj)

    return is_a_out, similar_map, existing


# -- Generazione candidati ---------------------------------------------------
def candidates_inheritance(is_a_out, similar_map, existing, max_candidates=None):
    """
    Parole senza IS_A che hanno un vicino SIMILAR_TO con IS_A.
    Propone di ereditare la categoria del vicino.
    """
    candidates = []
    for word, neighbors in similar_map.items():
        if is_a_out[word]:
            continue
        inherited = {}  # categoria -> vicino donatore
        for nbr in neighbors:
            for cat in is_a_out[nbr]:
                if cat not in inherited:
                    inherited[cat] = nbr
        for cat, donor in inherited.items():
            triple = (word, "IS_A", cat)
            if triple not in existing:
                candidates.append({
                    "mode":     "inheritance",
                    "word":     word,
                    "relation": "IS_A",
                    "target":   cat,
                    "hint":     f"simile a '{donor}' che IS_A '{cat}'"
                })
        if max_candidates and len(candidates) >= max_candidates:
            break
    return candidates[:max_candidates] if max_candidates else candidates


ONTOLOGY_CATS = [
    "persona", "animale", "oggetto", "luogo", "azione", "stato",
    "emozione", "concetto", "qualita", "evento", "processo", "relazione",
    "sostanza", "strumento", "comunicazione", "percezione", "movimento",
    "struttura", "sistema", "parte", "tempo", "spazio", "forma"
]

def normalize_str(s: str) -> str:
    """Rimuove diacritici e normalizza a ASCII-safe lowercase."""
    nfkd = unicodedata.normalize("NFKD", s)
    return "".join(c for c in nfkd if not unicodedata.combining(c)).lower().strip()

def candidates_direct(is_a_out, similar_map, existing, max_candidates=None):
    """
    Parole senza IS_A e senza vicini con IS_A.
    Qwen le classifica direttamente.
    """
    all_words = set(similar_map.keys()) | set(is_a_out.keys())
    candidates = []
    for word in all_words:
        if is_a_out[word]:
            continue
        if any(is_a_out[n] for n in similar_map[word]):
            continue  # gestita da inheritance
        candidates.append({
            "mode":     "direct",
            "word":     word,
            "relation": "IS_A",
            "target":   None,
            "cats":     ONTOLOGY_CATS
        })
        if max_candidates and len(candidates) >= max_candidates:
            break
    return candidates


# -- Prompt e query LLM ------------------------------------------------------
def build_prompt_inheritance(batch):
    lines = []
    for i, c in enumerate(batch, 1):
        lines.append(f'{i}. "{c["word"]}" IS_A "{c["target"]}"? ({c["hint"]})')
    items_str = "\n".join(lines)
    n = len(batch)
    return f"""Sei un lessicografo italiano esperto. Per ogni affermazione, valuta se e' semanticamente corretta in italiano.
Rispondi SOLO con un array JSON. Nessun testo extra, nessun ragionamento.

Affermazioni:
{items_str}

Schema (array di {n} oggetti):
[{{"id": 1, "correct": true, "confidence": 0.95, "note": ""}}, ...]

JSON:"""


def build_prompt_direct(batch):
    cats_str = ", ".join(ONTOLOGY_CATS)
    lines = [f'{i}. "{c["word"]}"' for i, c in enumerate(batch, 1)]
    items_str = "\n".join(lines)
    n = len(batch)
    return f"""Sei un lessicografo italiano esperto. Per ogni parola italiana, indica la categoria semantica IS_A piu' appropriata.
Categorie disponibili: {cats_str}
Rispondi SOLO con un array JSON. Nessun testo extra, nessun ragionamento.

Parole:
{items_str}

Schema (array di {n} oggetti):
[{{"id": 1, "word": "parola", "is_a": "categoria", "confidence": 0.95}}, ...]

JSON:"""


def query_llm(prompt: str, retries: int = 2):
    payload = {
        "model":   MODEL,
        "stream":  False,
        "think":   False,
        "prompt":  prompt,
        "options": {"temperature": 0.1}
    }
    for attempt in range(retries + 1):
        try:
            r = requests.post(OLLAMA_URL, json=payload, timeout=90)
            r.raise_for_status()
            r.encoding = "utf-8"
            return r.json().get("response", "").strip()
        except Exception as e:
            if attempt < retries:
                time.sleep(2)
            else:
                print(f"  [ERRORE LLM] {e}", file=sys.stderr)
    return None


def parse_json_response(text):
    if not text:
        return None
    for start_ch, end_ch in [('[', ']'), ('{', '}')]:
        s = text.find(start_ch)
        e = text.rfind(end_ch)
        if s != -1 and e != -1 and e > s:
            try:
                return json.loads(text[s:e+1])
            except json.JSONDecodeError:
                pass
    return None


# -- Processamento batch -----------------------------------------------------
def process_batch_inheritance(batch, existing):
    prompt   = build_prompt_inheritance(batch)
    response = query_llm(prompt)
    results  = parse_json_response(response)
    accepted = []

    if not isinstance(results, list):
        print(f"  [WARN] risposta non parsabile: {str(response)[:120]}")
        return accepted

    for item in results:
        try:
            idx  = int(item["id"]) - 1
            if not (0 <= idx < len(batch)):
                continue
            c    = batch[idx]
            conf = float(item.get("confidence", 0))
            ok   = bool(item.get("correct", False))
            if ok and conf >= CONFIDENCE_TH:
                target = normalize_str(c["target"])
                triple = (c["word"], "IS_A", target)
                if triple not in existing:
                    existing.add(triple)
                    accepted.append((c["word"], "IS_A", target, conf))
        except (KeyError, IndexError, ValueError, TypeError):
            continue
    return accepted


def process_batch_direct(batch, existing):
    prompt   = build_prompt_direct(batch)
    response = query_llm(prompt)
    results  = parse_json_response(response)
    accepted = []

    if not isinstance(results, list):
        print(f"  [WARN] risposta non parsabile: {str(response)[:120]}")
        return accepted

    for item in results:
        try:
            word = normalize_str(str(item.get("word", "")))
            cat  = normalize_str(str(item.get("is_a", "")))
            conf = float(item.get("confidence", 0))
            if word and cat and conf >= CONFIDENCE_TH:
                triple = (word, "IS_A", cat)
                if triple not in existing:
                    existing.add(triple)
                    accepted.append((word, "IS_A", cat, conf))
        except (KeyError, ValueError, TypeError):
            continue
    return accepted


# -- Output TSV --------------------------------------------------------------
def append_to_tsv(rows, path: Path):
    new_file = not path.exists()
    with open(path, "a", encoding="utf-8") as f:
        if new_file:
            f.write("# KG arricchito da Qwen 3 agent\n")
            f.write("# soggetto\tRELAZIONE\toggetto\tconfidenza\n")
        for (subj, rel, obj, conf) in rows:
            f.write(f"{subj}\t{rel}\t{obj}\t{conf:.2f}\n")


# -- Progresso ---------------------------------------------------------------
def load_progress():
    if PROGRESS_F.exists():
        return set(PROGRESS_F.read_text(encoding="utf-8").splitlines())
    return set()


def save_progress(done):
    PROGRESS_F.write_text("\n".join(done), encoding="utf-8")


# -- Main --------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description="Agent KG Builder per Prometeo")
    parser.add_argument("--test", type=int, default=0,
                        help="Test: processa solo N parole")
    parser.add_argument("--mode", choices=["inheritance", "direct", "both"],
                        default="both", help="Strategia candidati (default: both)")
    parser.add_argument("--output", type=str, default=str(OUTPUT_TSV),
                        help="File TSV di output")
    args = parser.parse_args()
    out_path = Path(args.output)

    print("=== Agent KG Builder — Prometeo ===")
    print(f"Modello : {MODEL}  |  Soglia: {CONFIDENCE_TH}  |  Batch: {BATCH_SIZE}")
    print(f"Output  : {out_path}")
    if args.test:
        print(f"Modalita': TEST ({args.test} parole)")

    # 1. Carica KG
    print("\n[1] Caricamento KG...")
    is_a_out, similar_map, existing = load_kg(KG_JSON)
    n_words = len(set(similar_map.keys()) | set(is_a_out.keys()))
    print(f"    Parole: {n_words}  |  Con IS_A: {len(is_a_out)}  |  Senza: {n_words - len(is_a_out)}")

    # 2. Candidati
    print("\n[2] Generazione candidati...")
    max_c = args.test if args.test else None
    inh_cands, dir_cands = [], []

    if args.mode in ("inheritance", "both"):
        inh_cands = candidates_inheritance(is_a_out, similar_map, existing, max_c)
        print(f"    Inheritance: {len(inh_cands)}")

    if args.mode in ("direct", "both"):
        rem = (max_c - len(inh_cands)) if max_c else None
        if rem is None or rem > 0:
            dir_cands = candidates_direct(is_a_out, similar_map, existing, rem)
            print(f"    Direct     : {len(dir_cands)}")

    candidates = (inh_cands + dir_cands)[:max_c] if max_c else inh_cands + dir_cands
    print(f"    Totale     : {len(candidates)}")

    if not candidates:
        print("Nessun candidato. Uscita.")
        return

    # Filtra gia' processati
    done = load_progress()
    candidates = [c for c in candidates if c["word"] not in done]
    print(f"    Da fare    : {len(candidates)}  (gia' fatti: {len(done)})")

    # 3. Query LLM
    print(f"\n[3] Query Qwen 3...")
    total_acc = 0
    total_proc = 0
    n_tot = len(candidates)

    def run_batches(cands, process_fn):
        nonlocal total_acc, total_proc
        for i in range(0, len(cands), BATCH_SIZE):
            batch   = cands[i:i+BATCH_SIZE]
            rows    = process_fn(batch, existing)
            if rows:
                append_to_tsv(rows, out_path)
                total_acc += len(rows)
                for row in rows:
                    print(f"  + {row[0]:25s} IS_A  {row[2]:20s}  (conf={row[3]:.2f})")
            total_proc += len(batch)
            done.update(c["word"] for c in batch)
            pct = total_proc / n_tot * 100
            print(f"  [{pct:5.1f}%] proc={total_proc}/{n_tot}  acc={total_acc}")
            if not args.test:
                save_progress(done)
            time.sleep(DELAY_S)

    run_batches([c for c in candidates if c["mode"] == "inheritance"],
                process_batch_inheritance)
    run_batches([c for c in candidates if c["mode"] == "direct"],
                process_batch_direct)

    # Report
    print(f"\n=== COMPLETATO ===")
    print(f"Processati : {total_proc}")
    print(f"Accettati  : {total_acc}")
    if total_proc:
        print(f"Tasso      : {total_acc/total_proc*100:.1f}%")
    print(f"Output     : {out_path}")


if __name__ == "__main__":
    main()
