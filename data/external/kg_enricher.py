#!/usr/bin/env python3
"""
kg_enricher.py — Arricchisce il KG con le relazioni mancanti per ogni parola.

Relazioni target (in ordine di priorità):
  1. OppositeOf  — 93.6% mancante
  2. PartOf      — 82.6% mancante
  3. UsedFor     — 80.4% mancante
  4. Causes      — 49.6% mancante
  5. IsA         — 24.8% mancante (già parzialmente gestito da agent_kg_builder.py)

Chiave filosofica:
  Il valore numerico è la FORZA della relazione, non la confidenza
  che esista. Se Qwen la genera, esiste — il numero dice quanto è forte.
  0.3 = relazione debole ma reale. 1.0 = relazione definitoria.

Uso:
  python kg_enricher.py --rel OppositeOf --test 20
  python kg_enricher.py --rel OppositeOf --resume
  python kg_enricher.py --rel PartOf --resume
  python kg_enricher.py --rel all --resume      # tutti in sequenza
"""

import json
import requests
import time
import argparse
import sys
import re
from collections import defaultdict
from pathlib import Path

# ── Configurazione ──────────────────────────────────────────────────────────
OLLAMA_URL = "http://localhost:11434/api/generate"
MODEL      = "qwen3"
BATCH_SIZE = 12
DELAY_S    = 0.15

BASE_DIR   = Path(__file__).parent.parent.parent  # data/external -> data -> prometeo_standalone
KG_JSON    = BASE_DIR / "prometeo_kg.json"
KG_DIR     = BASE_DIR / "data" / "kg"

# File di output per tipo di relazione
OUTPUT_FILES = {
    "OppositeOf": KG_DIR / "agent_opposites.tsv",
    "PartOf":     KG_DIR / "agent_partof.tsv",
    "UsedFor":    KG_DIR / "agent_usedfor.tsv",
    "Causes":     KG_DIR / "agent_causes.tsv",
    "IsA":        KG_DIR / "agent_isa_enriched.tsv",
}

PROGRESS_FILES = {
    rel: KG_DIR / f"agent_{rel.lower()}_progress.txt"
    for rel in OUTPUT_FILES
}

# Priorità di processamento
REL_PRIORITY = ["OppositeOf", "PartOf", "UsedFor", "Causes", "IsA"]


# ── Caricamento KG ───────────────────────────────────────────────────────────
def load_kg(path: Path):
    """Restituisce: word_rels (word -> set di relazioni), existing (set di triple)."""
    with open(path, encoding="utf-8") as f:
        data = json.load(f)

    word_rels = defaultdict(set)   # word -> {IsA, Causes, ...}
    existing  = set()              # (subj, rel, obj) già presenti

    for e in data["edges"]:
        subj = e["subject"].lower().strip()
        obj  = e["object"].lower().strip()
        rel  = e["relation"]
        word_rels[subj].add(rel)
        existing.add((subj, rel, obj))

    print(f"KG caricato: {len(data['edges'])} archi, {len(word_rels)} parole")
    return word_rels, existing


def load_done(progress_file: Path) -> set:
    """Parole già processate (per riprendibilità)."""
    if not progress_file.exists():
        return set()
    return set(progress_file.read_text(encoding="utf-8").splitlines())


def mark_done(progress_file: Path, word: str):
    with open(progress_file, "a", encoding="utf-8") as f:
        f.write(word + "\n")


# ── Prompt per tipo di relazione ────────────────────────────────────────────

PROMPTS = {
    "OppositeOf": """Sei un lessicografo italiano. Per ogni parola fornisci il contrario principale in italiano.
REGOLE CRITICHE:
- L'oggetto deve essere UNA SOLA PAROLA italiana (no spazi, no trattini)
- Il valore "strength": 1.0=opposto definitorio (caldo/freddo), 0.7=opposto comune, 0.4=opposto contestuale
- Se non esiste un contrario naturale a una sola parola, usa null
- Minuscolo, niente accenti (usa vocale semplice o apostrofo)
- Rispondi SOLO con array JSON valido, zero testo extra

Parole:
{words_list}

Schema ({n} oggetti):
[{{"id": 1, "word": "parola", "opposite": "contrario_o_null", "strength": 0.9}}, ...]

JSON:""",

    "PartOf": """Sei un lessicografo italiano. Per ogni parola indica di cosa è parte (PART_OF).
REGOLE CRITICHE:
- L'oggetto deve essere UNA SOLA PAROLA italiana (no spazi, no trattini)
- Solo relazioni parti-di concrete e prototipiche (foglia→albero, dito→mano, capitolo→libro)
- Verbi e aggettivi non sono parti di nulla: usa null
- Il valore "strength": 1.0=parte definitoria, 0.7=parte prototipica, 0.4=parte possibile
- Se non c'è contenitore naturale a una sola parola, usa null
- Minuscolo, rispondi SOLO con array JSON valido

Parole:
{words_list}

Schema ({n} oggetti):
[{{"id": 1, "word": "parola", "whole": "contenitore_o_null", "strength": 0.8}}, ...]

JSON:""",

    "UsedFor": """Sei un lessicografo italiano. Per ogni parola indica il suo scopo/uso principale (USED_FOR).
REGOLE CRITICHE:
- L'oggetto deve essere UNA SOLA PAROLA italiana: un sostantivo o infinito verbale (es: tagliare, protezione, lettura)
- Il valore "strength": 1.0=funzione primaria, 0.7=uso comune, 0.4=uso possibile
- Articoli, preposizioni, congiunzioni → usa null
- Minuscolo, rispondi SOLO con array JSON valido

Parole:
{words_list}

Schema ({n} oggetti):
[{{"id": 1, "word": "parola", "purpose": "scopo_o_null", "strength": 0.85}}, ...]

JSON:""",

    "Causes": """Sei un lessicografo italiano. Per ogni parola indica cosa provoca/causa tipicamente (CAUSES).
REGOLE CRITICHE:
- L'oggetto deve essere UNA SOLA PAROLA italiana (no spazi, no trattini)
- Scegli l'effetto più diretto e comune: pioggia→umidità, sole→calore, paura→fuga
- Articoli, preposizioni, congiunzioni → usa null
- Il valore "strength": 1.0=causa diretta e certa, 0.7=causa comune, 0.4=causa possibile
- Minuscolo, rispondi SOLO con array JSON valido

Parole:
{words_list}

Schema ({n} oggetti):
[{{"id": 1, "word": "parola", "effect": "effetto_o_null", "strength": 0.8}}, ...]

JSON:""",

    "IsA": """Sei un lessicografo italiano. Per ogni parola indica la categoria semantica IS_A più appropriata.
Categorie disponibili: persona, animale, oggetto, luogo, azione, stato, emozione, concetto, qualita,
evento, processo, relazione, sostanza, strumento, comunicazione, percezione, movimento, struttura,
sistema, parte, tempo, spazio, forma, suono, colore, numero, simbolo

REGOLE CRITICHE:
- Scegli UNA categoria, la più specifica e corretta
- Il valore "strength": 1.0=categorizzazione definitoria, 0.7=prototipica, 0.4=marginale
- Minuscolo, rispondi SOLO con array JSON valido

Parole:
{words_list}

Schema ({n} oggetti):
[{{"id": 1, "word": "parola", "category": "categoria", "strength": 0.9}}, ...]

JSON:""",
}

# Chiavi del campo target per ogni relazione
TARGET_KEYS = {
    "OppositeOf": "opposite",
    "PartOf":     "whole",
    "UsedFor":    "purpose",
    "Causes":     "effect",
    "IsA":        "category",
}


# ── LLM ─────────────────────────────────────────────────────────────────────
def query_llm(prompt: str, retries: int = 2) -> str | None:
    payload = {
        "model":   MODEL,
        "stream":  False,
        "think":   False,
        "prompt":  prompt,
        "options": {"temperature": 0.1},
    }
    for attempt in range(retries + 1):
        try:
            r = requests.post(OLLAMA_URL, json=payload, timeout=120)
            r.raise_for_status()
            r.encoding = "utf-8"
            return r.json().get("response", "").strip()
        except Exception as e:
            if attempt < retries:
                time.sleep(2)
            else:
                print(f"  [ERRORE LLM] {e}", file=sys.stderr)
    return None


def parse_json_response(text: str):
    if not text:
        return None
    for start_ch, end_ch in [("[", "]"), ("{", "}")]:
        s = text.find(start_ch)
        e = text.rfind(end_ch)
        if s != -1 and e != -1 and e > s:
            try:
                return json.loads(text[s:e+1])
            except json.JSONDecodeError:
                pass
    return None


# ── Processamento batch ──────────────────────────────────────────────────────
def process_batch(batch: list[str], rel_type: str, existing: set) -> list[tuple]:
    """
    Processa un batch di parole per un tipo di relazione.
    Restituisce lista di (soggetto, relazione, oggetto, strength).
    """
    words_list = "\n".join(f'{i+1}. "{w}"' for i, w in enumerate(batch))
    prompt = PROMPTS[rel_type].format(words_list=words_list, n=len(batch))

    raw = query_llm(prompt)
    parsed = parse_json_response(raw)
    if not parsed:
        print(f"  [PARSE ERR] batch {batch[:3]}...", file=sys.stderr)
        return []

    if isinstance(parsed, dict):
        parsed = [parsed]

    results = []
    target_key = TARGET_KEYS[rel_type]

    for item in parsed:
        if not isinstance(item, dict):
            continue
        idx = item.get("id", 0)
        if isinstance(idx, int) and 1 <= idx <= len(batch):
            word = batch[idx - 1]
        else:
            word = item.get("word", "").lower().strip()
            if not word or word not in batch:
                continue

        target = item.get(target_key)
        if not target or target == "null":
            continue

        target = str(target).lower().strip()
        # Pulizia: rimuovi caratteri non ammessi
        target = re.sub(r"[^\w']", "", target).strip()
        # Rifiuta oggetti multi-parola (il KG usa solo token singoli)
        if not target or len(target) < 2 or " " in target:
            continue

        strength = float(item.get("strength", 0.7))
        strength = max(0.1, min(1.0, strength))

        # Evita tautologie (X rel X)
        if target == word:
            continue

        triple = (word, rel_type, target)
        if triple in existing:
            continue

        results.append((word, rel_type, target, strength))
        existing.add(triple)  # aggiorna per evitare duplicati nel batch

    return results


def write_results(results: list[tuple], out_file: Path, rel_type: str, first_write: bool):
    """Scrive le triple nel TSV di output."""
    mode = "w" if first_write else "a"
    with open(out_file, mode, encoding="utf-8") as f:
        if first_write:
            f.write(f"# KG arricchimento {rel_type} — generato da kg_enricher.py\n")
            f.write(f"# soggetto\tRELAZIONE\toggetto\tforza\n")
        for subj, rel, obj, strength in results:
            f.write(f"{subj}\t{rel}\t{obj}\t{strength:.2f}\n")


# ── Logica principale ────────────────────────────────────────────────────────
def enrich_relation(rel_type: str, word_rels: dict, existing: set,
                    max_words: int | None = None):
    """Arricchisce il KG per un tipo di relazione specifico."""

    out_file   = OUTPUT_FILES[rel_type]
    prog_file  = PROGRESS_FILES[rel_type]
    done_words = load_done(prog_file)

    # Parole che mancano questa relazione
    missing = [w for w, rels in word_rels.items() if rel_type not in rels]
    # Rimuovi già processate
    todo = [w for w in missing if w not in done_words]

    if max_words:
        todo = todo[:max_words]

    total    = len(todo)
    processed = 0
    added     = 0
    first_write = not out_file.exists()

    print(f"\n{'='*60}")
    print(f"  Relazione: {rel_type}")
    print(f"  Parole mancanti: {len(missing)}")
    print(f"  Già processate:  {len(done_words)}")
    print(f"  Da processare:   {total}")
    print(f"{'='*60}")

    if total == 0:
        print("  Niente da fare!")
        return

    # Processa in batch
    for i in range(0, total, BATCH_SIZE):
        batch = todo[i:i + BATCH_SIZE]
        results = process_batch(batch, rel_type, existing)

        if results:
            write_results(results, out_file, rel_type, first_write)
            first_write = False
            added += len(results)

        # Marca tutte le parole del batch come processate (anche quelle senza output)
        for w in batch:
            mark_done(prog_file, w)

        processed += len(batch)
        pct = processed / total * 100
        print(f"  [{processed:6d}/{total}] {pct:5.1f}%  +{len(results)} triple  (totale: {added})",
              end="\r", flush=True)

        time.sleep(DELAY_S)

    print(f"\n  Completato! {added} triple aggiunte per {rel_type}")


# ── Entry point ──────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(description="Arricchisce il KG con relazioni mancanti")
    parser.add_argument("--rel", default="OppositeOf",
                        choices=REL_PRIORITY + ["all"],
                        help="Tipo di relazione da arricchire (default: OppositeOf)")
    parser.add_argument("--test", type=int, default=None,
                        help="Processa solo N parole (test)")
    parser.add_argument("--resume", action="store_true",
                        help="Riprendi dal punto precedente (usa il progress file)")
    args = parser.parse_args()

    word_rels, existing = load_kg(KG_JSON)

    rels_to_process = REL_PRIORITY if args.rel == "all" else [args.rel]

    for rel in rels_to_process:
        enrich_relation(
            rel_type   = rel,
            word_rels  = word_rels,
            existing   = existing,
            max_words  = args.test,
        )
        # Piccola pausa tra relazioni diverse
        if args.rel == "all" and rel != rels_to_process[-1]:
            print("  Pausa 2s tra relazioni...")
            time.sleep(2)

    print("\nDone. Ricordati di:")
    print("  1. cargo run --release --bin import-kg")
    print("  2. cargo run --release --bin rebuild-semantic-topology")


if __name__ == "__main__":
    main()
