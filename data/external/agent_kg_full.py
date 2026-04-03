#!/usr/bin/env python3
"""
agent_kg_full.py — Arricchimento KG completo di Prometeo via Qwen 3.

Estrae in un unico passaggio per parola:
  - IS_A      : categoria semantica + profondita' automatica via gerarchia
  - CAUSES    : cosa provoca (emozioni, stati, azioni, fenomeni)
  - PART_OF   : di cosa e' parte (corpo, natura, oggetti composti)
  - USED_FOR  : a cosa serve (strumenti, sostanze, concetti applicati)

Gerarchia IS_A (propagazione automatica senza LLM):
  emozione -> stato -> condizione
  persona/animale -> essere_vivente -> entita
  strumento -> oggetto -> entita
  azione -> evento -> accadimento
  luogo -> spazio -> dimensione
  ... (vedi IS_A_HIERARCHY sotto)

Uso:
  python agent_kg_full.py               # full run
  python agent_kg_full.py --test 25     # test su 25 parole
  python agent_kg_full.py --skip-is-a   # solo CAUSES/PART_OF/USED_FOR
"""

import json
import requests
import time
import argparse
import sys
import unicodedata
from collections import defaultdict
from pathlib import Path

# Fix encoding su Windows (cp1252 non supporta caratteri UTF-8)
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

# -- Configurazione ----------------------------------------------------------
OLLAMA_URL    = "http://localhost:11434/api/generate"
MODEL         = "qwen3"
CONFIDENCE_TH = 0.72       # soglia minima accettazione
BATCH_SIZE    = 5          # parole per chiamata LLM (output piu' ricco -> batch piu' piccolo)
DELAY_S       = 0.1

BASE_DIR   = Path(__file__).parent.parent.parent
KG_JSON    = BASE_DIR / "prometeo_kg.json"
AGENT_TSV  = BASE_DIR / "data" / "kg" / "agent_kg.tsv"       # output run precedente (IS_A)
OUTPUT_TSV = BASE_DIR / "data" / "kg" / "agent_kg_full.tsv"
PROGRESS_F = BASE_DIR / "data" / "kg" / "agent_kg_full_progress.txt"

# -- Gerarchia IS_A per propagazione automatica (senza LLM) -----------------
# Quando una parola riceve IS_A X, ottiene automaticamente anche IS_A Y
# per ogni Y nella catena. Costruisce profondita' senza chiamate extra.
IS_A_HIERARCHY: dict[str, list[str]] = {
    # stati mentali
    "emozione":       ["stato", "percezione"],
    "percezione":     ["processo"],
    "stato":          ["condizione"],
    "condizione":     ["concetto"],
    # esseri
    "persona":        ["essere_vivente"],
    "animale":        ["essere_vivente"],
    "essere_vivente": ["entita"],
    # oggetti
    "strumento":      ["oggetto"],
    "oggetto":        ["entita"],
    "struttura":      ["oggetto"],
    "sistema":        ["struttura"],
    # materia
    "sostanza":       ["materia"],
    "materia":        ["entita"],
    # azioni
    "movimento":      ["azione"],
    "comunicazione":  ["azione"],
    "azione":         ["evento"],
    "processo":       ["evento"],
    "evento":         ["accadimento"],
    "accadimento":    ["entita"],
    # spazio/tempo
    "luogo":          ["spazio"],
    "spazio":         ["dimensione"],
    "tempo":          ["dimensione"],
    "dimensione":     ["concetto"],
    # astratti
    "qualita":        ["attributo"],
    "attributo":      ["concetto"],
    "relazione":      ["concetto"],
    "forma":          ["concetto"],
    "parte":          ["entita"],
    "sistema":        ["struttura"],
    "entita":         ["concetto"],
}

# Categorie per cui CAUSES ha senso
CAUSES_CATS = {"emozione", "stato", "condizione", "azione", "evento",
               "processo", "movimento", "percezione", "fenomeno"}

# Categorie per cui PART_OF ha senso
PART_OF_CATS = {"parte", "struttura", "oggetto", "essere_vivente",
                "persona", "animale", "sostanza", "forma"}

# Categorie per cui USED_FOR ha senso
USED_FOR_CATS = {"strumento", "oggetto", "sostanza", "sistema",
                 "comunicazione", "processo", "concetto"}

# Categorie IS_A disponibili (presentate al LLM)
IS_A_CATS = [
    "persona", "animale", "oggetto", "luogo", "azione", "stato",
    "emozione", "concetto", "qualita", "evento", "processo", "relazione",
    "sostanza", "strumento", "comunicazione", "percezione", "movimento",
    "struttura", "sistema", "parte", "tempo", "spazio", "forma", "fenomeno"
]


# -- Utilita' ----------------------------------------------------------------
def normalize(s: str) -> str:
    nfkd = unicodedata.normalize("NFKD", str(s))
    return "".join(c for c in nfkd if not unicodedata.combining(c)).lower().strip()


def ground_word(word: str, all_words: set) -> str | None:
    """
    Verifica che 'word' sia nel lessico di Prometeo (exact match).
    Se non e' presente, ritorna None — la parola viene scartata.
    Non si usa fuzzy matching: la correttezza semantica e' responsabilita' di Qwen,
    non di euristiche ortografiche.
    """
    if not word or len(word) < 2 or " " in word or "'" in word:
        return None
    return word if word in all_words else None


def expand_is_a(category: str, depth: int = 0) -> list[str]:
    """Propaga IS_A lungo la gerarchia. Ritorna categoria + tutti gli antenati."""
    if depth > 6:
        return []
    results = [category]
    for parent in IS_A_HIERARCHY.get(category, []):
        results += expand_is_a(parent, depth + 1)
    return list(dict.fromkeys(results))  # deduplicato, ordine preservato


# -- Caricamento KG ----------------------------------------------------------
def load_all_words_and_existing(skip_is_a: bool) -> tuple[set, set, dict]:
    """
    Ritorna:
      all_words   : tutte le parole note (da KG json + bigbang + agent_kg)
      existing    : triple (subj, REL, obj) gia' presenti
      word_is_a   : {word: categoria} per parole che hanno gia' IS_A
    """
    existing  = set()
    word_is_a = {}  # word -> categoria IS_A gia' assegnata
    all_words = set()

    # 1. prometeo_kg.json
    with open(KG_JSON, encoding="utf-8") as f:
        data = json.load(f)
    for e in data["edges"]:
        s = normalize(e["subject"])
        o = normalize(e["object"])
        r = e["relation"].upper()
        existing.add((s, r, o))
        all_words.add(s)
        all_words.add(o)
        if r == "ISA":
            word_is_a[s] = o

    # 2. bigbang_kg.tsv
    bb_path = BASE_DIR / "data" / "kg" / "bigbang_kg.tsv"
    if bb_path.exists():
        with open(bb_path, encoding="utf-8") as f:
            for line in f:
                if line.startswith("#") or not line.strip():
                    continue
                parts = line.strip().split("\t")
                if len(parts) >= 3:
                    s, r, o = normalize(parts[0]), parts[1].upper(), normalize(parts[2])
                    existing.add((s, r, o))
                    all_words.add(s)
                    all_words.add(o)

    # 3. agent_kg.tsv (run IS_A precedente)
    if AGENT_TSV.exists():
        with open(AGENT_TSV, encoding="utf-8") as f:
            for line in f:
                if line.startswith("#") or not line.strip():
                    continue
                parts = line.strip().split("\t")
                if len(parts) >= 3:
                    s, r, o = normalize(parts[0]), parts[1].upper(), normalize(parts[2])
                    existing.add((s, r, o))
                    all_words.add(s)
                    if r == "IS_A":
                        word_is_a[s] = o
                        # Propaga la gerarchia gia' nelle existing
                        for anc in expand_is_a(o)[1:]:
                            existing.add((s, "IS_A", anc))

    # Indice (word, relation) -> bool per lookup O(1)
    word_rel_index: set[tuple[str, str]] = set()
    for (s, r, _o) in existing:
        word_rel_index.add((s, r))

    return all_words, existing, word_is_a, word_rel_index


# -- Costruzione candidati ---------------------------------------------------
def build_candidates(all_words: set, existing: set, word_is_a: dict,
                     word_rel_index: set, skip_is_a: bool) -> list[dict]:
    """
    Per ogni parola costruisce un record con le relazioni DA CHIEDERE.
    Usa word_rel_index per lookup O(1) invece di scansione O(N).
    """
    candidates = []
    for word in sorted(all_words):
        if len(word) < 2 or not word.isalpha():
            continue

        needs_is_a     = not skip_is_a and word not in word_is_a
        known_cat      = word_is_a.get(word)
        needs_causes   = known_cat is None or known_cat in CAUSES_CATS
        needs_part_of  = known_cat is None or known_cat in PART_OF_CATS
        needs_used_for = known_cat is None or known_cat in USED_FOR_CATS

        has_causes   = (word, "CAUSES")  in word_rel_index
        has_part_of  = (word, "PART_OF") in word_rel_index
        has_used_for = (word, "USED_FOR") in word_rel_index

        if not needs_is_a and has_causes and has_part_of and has_used_for:
            continue

        candidates.append({
            "word":        word,
            "known_cat":   known_cat,
            "ask_is_a":    needs_is_a,
            "ask_causes":  needs_causes and not has_causes,
            "ask_part_of": needs_part_of and not has_part_of,
            "ask_used_for": needs_used_for and not has_used_for,
        })

    return candidates


# -- Prompt LLM --------------------------------------------------------------
def build_prompt(batch: list[dict]) -> str:
    cats_str = ", ".join(IS_A_CATS)
    words_str = ", ".join(f'"{c["word"]}"' for c in batch)

    # Costruisce la lista con indicazioni per word
    details = []
    for i, c in enumerate(batch, 1):
        hints = []
        if c["ask_is_a"]:
            hints.append(f"is_a (scegli da: {cats_str})")
        else:
            hints.append(f'is_a: "{c["known_cat"]}" (gia\' noto, conferma)')
        if c["ask_causes"]:
            hints.append("causes (lista 1-3 parole italiane che questa parola provoca, null se non applicabile)")
        if c["ask_part_of"]:
            hints.append("part_of (di cosa e' parte, null se non applicabile)")
        if c["ask_used_for"]:
            hints.append("used_for (lista 1-2 scopi, null se non applicabile)")
        details.append(f'{i}. "{c["word"]}": {" | ".join(hints)}')

    details_str = "\n".join(details)
    n = len(batch)

    return f"""Sei un lessicografo italiano esperto. Per ogni parola italiana fornisci le relazioni semantiche richieste.
Rispondi SOLO con un array JSON di {n} oggetti. Nessun testo extra, nessun ragionamento.

Parole e relazioni richieste:
{details_str}

Schema (adatta i campi a quelli richiesti per ogni parola):
[
  {{
    "id": 1,
    "word": "...",
    "is_a": "categoria",
    "is_a_confidence": 0.95,
    "causes": ["parola1", "parola2"],
    "causes_confidence": 0.85,
    "part_of": "parola",
    "part_of_confidence": 0.90,
    "used_for": ["scopo1"],
    "used_for_confidence": 0.80
  }}
]

Usa null per i campi non applicabili. Solo parole italiane singole (no frasi).

JSON:"""


# -- Query LLM ---------------------------------------------------------------
def query_llm(prompt: str, retries: int = 2) -> str | None:
    payload = {
        "model":   MODEL,
        "stream":  False,
        "think":   False,
        "prompt":  prompt,
        "options": {"temperature": 0.1}
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


def parse_json(text: str):
    if not text:
        return None
    for sc, ec in [('[', ']'), ('{', '}')]:
        s = text.find(sc)
        e = text.rfind(ec)
        if s != -1 and e != -1 and e > s:
            try:
                return json.loads(text[s:e+1])
            except json.JSONDecodeError:
                pass
    return None


# -- Processamento batch -----------------------------------------------------
def process_batch(batch: list[dict], existing: set, word_is_a: dict, existing_words: set) -> list[tuple]:
    """Ritorna lista di (subj, REL, obj, conf) da aggiungere."""
    prompt   = build_prompt(batch)
    response = query_llm(prompt)
    results  = parse_json(response)
    accepted = []

    if not isinstance(results, list):
        print(f"  [WARN] JSON non parsabile: {str(response)[:100]}")
        return accepted

    for item in results:
        if not isinstance(item, dict):
            continue
        try:
            idx = int(item.get("id", 0)) - 1
            if not (0 <= idx < len(batch)):
                continue
            c = batch[idx]
            word = c["word"]

            # --- IS_A ---
            if c["ask_is_a"]:
                cat  = normalize(item.get("is_a") or "")
                conf = float(item.get("is_a_confidence", 0))
                if cat and cat in IS_A_CATS and conf >= CONFIDENCE_TH:
                    # Categoria diretta
                    for anc in expand_is_a(cat):
                        triple = (word, "IS_A", anc)
                        if triple not in existing:
                            existing.add(triple)
                            # Confidenza decresce di 0.05 per ogni livello
                            c_adj = max(conf - 0.05 * expand_is_a(cat).index(anc), 0.70)
                            accepted.append((word, "IS_A", anc, round(c_adj, 2)))
                    word_is_a[word] = cat  # aggiorna mappa locale

            # --- CAUSES ---
            if c["ask_causes"]:
                causes = item.get("causes")
                conf   = float(item.get("causes_confidence", 0))
                if isinstance(causes, list) and conf >= CONFIDENCE_TH:
                    for obj in causes:
                        obj = ground_word(normalize(obj), existing_words)
                        if obj:
                            triple = (word, "CAUSES", obj)
                            if triple not in existing:
                                existing.add(triple)
                                accepted.append((word, "CAUSES", obj, conf))

            # --- PART_OF ---
            if c["ask_part_of"]:
                part = item.get("part_of")
                conf = float(item.get("part_of_confidence", 0))
                if part and isinstance(part, str) and conf >= CONFIDENCE_TH:
                    part = ground_word(normalize(part), existing_words)
                    if part:
                        triple = (word, "PART_OF", part)
                        if triple not in existing:
                            existing.add(triple)
                            accepted.append((word, "PART_OF", part, conf))

            # --- USED_FOR ---
            if c["ask_used_for"]:
                usages = item.get("used_for")
                conf   = float(item.get("used_for_confidence", 0))
                if isinstance(usages, list) and conf >= CONFIDENCE_TH:
                    for obj in usages:
                        obj = ground_word(normalize(obj), existing_words)
                        if obj:
                            triple = (word, "USED_FOR", obj)
                            if triple not in existing:
                                existing.add(triple)
                                accepted.append((word, "USED_FOR", obj, conf))

        except (KeyError, ValueError, TypeError, AttributeError):
            continue

    return accepted


# -- Output e progresso ------------------------------------------------------
def append_tsv(rows: list, path: Path):
    new_file = not path.exists()
    with open(path, "a", encoding="utf-8") as f:
        if new_file:
            f.write("# KG arricchimento completo (IS_A + CAUSES + PART_OF + USED_FOR) — Qwen 3\n")
            f.write("# soggetto\tRELAZIONE\toggetto\tconfidenza\n")
        for (s, r, o, c) in rows:
            f.write(f"{s}\t{r}\t{o}\t{c:.2f}\n")


def load_progress() -> set:
    if PROGRESS_F.exists():
        return set(PROGRESS_F.read_text(encoding="utf-8").splitlines())
    return set()


def save_progress(done: set):
    PROGRESS_F.write_text("\n".join(done), encoding="utf-8")


# -- Main --------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description="Agent KG Full — Prometeo")
    parser.add_argument("--test", type=int, default=0,
                        help="Test: processa solo N parole")
    parser.add_argument("--skip-is-a", action="store_true",
                        help="Salta IS_A (gia' fatto da agent_kg_builder), solo CAUSES/PART_OF/USED_FOR")
    args = parser.parse_args()

    print("=== Agent KG Full — Prometeo ===")
    print(f"Modello : {MODEL}  |  Soglia: {CONFIDENCE_TH}  |  Batch: {BATCH_SIZE}")
    print(f"Skip IS_A: {args.skip_is_a}")
    print(f"Output  : {OUTPUT_TSV}")
    if args.test:
        print(f"Modalita': TEST ({args.test} parole)")

    # 1. Carica tutto
    print("\n[1] Caricamento KG e parole note...")
    all_words, existing, word_is_a, word_rel_index = load_all_words_and_existing(args.skip_is_a)
    print(f"    Parole totali    : {len(all_words)}")
    print(f"    Triple esistenti : {len(existing)}")
    print(f"    Parole con IS_A  : {len(word_is_a)}")

    # 2. Candidati
    print("\n[2] Costruzione candidati...")
    candidates = build_candidates(all_words, existing, word_is_a, word_rel_index, args.skip_is_a)
    if args.test:
        candidates = candidates[:args.test]
    print(f"    Candidati: {len(candidates)}")

    # Filtra gia' processati
    done = load_progress()
    candidates = [c for c in candidates if c["word"] not in done]
    print(f"    Da fare  : {len(candidates)}  (gia' fatti: {len(done)})")

    if not candidates:
        print("Nessun candidato. Uscita.")
        return

    # 3. Query LLM
    print(f"\n[3] Query Qwen 3 (batch={BATCH_SIZE})...")
    total_acc  = 0
    total_proc = 0
    n_tot      = len(candidates)
    rel_counts: dict[str, int] = defaultdict(int)

    for i in range(0, n_tot, BATCH_SIZE):
        batch = candidates[i:i+BATCH_SIZE]
        rows  = process_batch(batch, existing, word_is_a, all_words)

        if rows:
            append_tsv(rows, OUTPUT_TSV)
            total_acc += len(rows)
            for row in rows:
                rel_counts[row[1]] += 1
                print(f"  + {row[0]:25s} {row[1]:10s} {row[2]:20s}  (conf={row[3]:.2f})")

        total_proc += len(batch)
        done.update(c["word"] for c in batch)
        pct = total_proc / n_tot * 100
        print(f"  [{pct:5.1f}%] proc={total_proc}/{n_tot}  acc={total_acc}  "
              f"IS_A={rel_counts['IS_A']} CAUSES={rel_counts['CAUSES']} "
              f"PART_OF={rel_counts['PART_OF']} USED_FOR={rel_counts['USED_FOR']}")

        if not args.test:
            save_progress(done)
        time.sleep(DELAY_S)

    # Report
    print(f"\n=== COMPLETATO ===")
    print(f"Processati  : {total_proc}")
    print(f"Accettati   : {total_acc}")
    for rel, cnt in sorted(rel_counts.items()):
        print(f"  {rel:12s}: {cnt}")
    print(f"Output      : {OUTPUT_TSV}")


if __name__ == "__main__":
    main()
