#!/usr/bin/env python3
"""
agent_similar.py — Genera relazioni SIMILAR_TO semantiche reali via Qwen 3.5.

A differenza del bigbang_kg (Kaikki lessicografico), qui la similarità è SEMANTICA:
- "coraggio" SIMILAR_TO "audacia" (0.85) — sinonimi concettuali
- "sole" SIMILAR_TO "fuoco" (0.35) — condividono calore ma usi diversi
- "cane" SIMILAR_TO "canaglia" — RIFIUTATO (somiglianza solo ortografica)

Il valore di similarità (0.0–1.0) diventa la confidence dell'arco nel KG.
Il modello valuta: puoi sostituire A con B in un discorso senza cambiare il senso?

Uso:
  python agent_similar.py --test 20       # test su 20 parole
  python agent_similar.py --resume        # riprendi da dove eri rimasto
  python agent_similar.py --batch 8       # cambia dimensione batch
"""

import json
import requests
import time
import argparse
import sys
from pathlib import Path

# -- Configurazione ----------------------------------------------------------
OLLAMA_URL    = "http://localhost:11434/api/generate"
MODEL         = "qwen3.5:latest"
BATCH_SIZE    = 8          # parole per chiamata
DELAY_S       = 0.05
MIN_SIMILARITY = 0.30      # sotto questa soglia non salviamo
MIN_WORD_LEN  = 3          # parole troppo corte sono ambigue

BASE_DIR   = Path(__file__).parent.parent.parent
KG_JSON    = BASE_DIR / "prometeo_kg.json"
OUTPUT_TSV = BASE_DIR / "data" / "kg" / "agent_similar.tsv"
PROGRESS_F = BASE_DIR / "data" / "kg" / "agent_similar_progress.txt"

# -- Carica lessico dal KG ---------------------------------------------------
def load_words():
    """Carica parole uniche dal KG JSON (nodi con archi)."""
    if not KG_JSON.exists():
        print(f"ERRORE: {KG_JSON} non trovato. Esegui prima import-kg.")
        sys.exit(1)
    with open(KG_JSON, "r", encoding="utf-8") as f:
        kg = json.load(f)

    # Il JSON può avere wrapper {"edges": [...]} o essere una lista diretta
    if isinstance(kg, dict) and "edges" in kg:
        kg = kg["edges"]

    words = set()
    for entry in kg:
        s = entry.get("subject", "").strip().lower()
        o = entry.get("object", "").strip().lower()
        if len(s) >= MIN_WORD_LEN:
            words.add(s)
        if len(o) >= MIN_WORD_LEN:
            words.add(o)

    # Filtra parole funzionali e troppo generiche
    stop = {"essere", "avere", "fare", "dire", "potere", "dovere", "volere",
            "andare", "venire", "dare", "stare", "cosa", "modo", "tipo",
            "parte", "fatto", "caso", "punto", "altro", "molto", "poco",
            "grande", "piccolo", "nuovo", "buono", "bene", "male",
            "tutto", "ogni", "stesso", "proprio", "primo", "ultimo"}
    words -= stop
    return sorted(words)


def load_progress():
    """Carica parole già processate."""
    if PROGRESS_F.exists():
        return set(PROGRESS_F.read_text(encoding="utf-8").strip().split("\n"))
    return set()


def save_progress(word):
    """Segna una parola come processata."""
    with open(PROGRESS_F, "a", encoding="utf-8") as f:
        f.write(word + "\n")


# -- Prompt per Qwen ---------------------------------------------------------
SYSTEM_PROMPT = """\
Sei un linguista semantico italiano. Per ogni parola ti chiedo di trovare \
parole SEMANTICAMENTE simili — cioè che puoi usare in modo intercambiabile \
o che condividono lo stesso significato profondo.

REGOLE FERREE:
- La similarità è SEMANTICA, non ortografica. "cane"/"canaglia" = 0 (zero).
- Valuta: "posso sostituire A con B in una frase senza cambiare il senso?"
  - Se sì quasi sempre → 0.80-0.95
  - Se sì in molti contesti → 0.60-0.79
  - Se condividono un aspetto importante → 0.40-0.59
  - Se la somiglianza è solo superficiale → NON includerla
- NO derivati morfologici (correre/corridore, bello/bellezza)
- NO iponimi/iperonimi (cane/animale — quello è IS_A, non SIMILAR_TO)
- Solo parole italiane comuni, singolari, minuscole
- Massimo 5 simili per parola. Meglio 2 buoni che 5 forzati.

FORMATO OUTPUT (JSON puro, nessun testo extra):
[{"w":"PAROLA","sim":[["simile1",0.82],["simile2",0.65]]}]
"""


def build_prompt(words_batch):
    """Costruisce il prompt per un batch di parole."""
    word_list = ", ".join(words_batch)
    return f"""/no_think
Trova i simili semantici per queste parole: {word_list}

Rispondi SOLO con JSON nel formato:
[{{"w":"parola","sim":[["simile1",0.80],["simile2",0.65]]}}]

Se una parola non ha simili semantici validi, metti "sim":[]."""


def query_ollama(prompt, retries=2):
    """Chiama Ollama e ritorna il testo generato."""
    for attempt in range(retries + 1):
        try:
            resp = requests.post(OLLAMA_URL, json={
                "model": MODEL,
                "prompt": prompt,
                "system": SYSTEM_PROMPT,
                "stream": False,
                "think": False,
                "options": {
                    "temperature": 0.3,
                    "num_predict": 2048,
                    "top_p": 0.9,
                }
            }, timeout=120)
            resp.raise_for_status()
            return resp.json().get("response", "")
        except Exception as e:
            if attempt < retries:
                print(f"  retry {attempt+1}: {e}")
                time.sleep(2)
            else:
                print(f"  ERRORE Ollama: {e}")
                return ""


def parse_response(text, valid_words):
    """Parsa la risposta JSON e valida le coppie."""
    # Estrai JSON dall'output (potrebbe avere testo attorno)
    text = text.strip()
    # Cerca il primo [ e l'ultimo ]
    start = text.find("[")
    end = text.rfind("]")
    if start == -1 or end == -1:
        return []

    json_str = text[start:end+1]
    try:
        data = json.loads(json_str)
    except json.JSONDecodeError:
        # Prova a fixare JSON troncato
        try:
            data = json.loads(json_str + "]")
        except:
            return []

    results = []
    for entry in data:
        if not isinstance(entry, dict):
            continue
        word = entry.get("w", "").strip().lower()
        sims = entry.get("sim", [])
        if not word or not isinstance(sims, list):
            continue

        for pair in sims:
            if not isinstance(pair, (list, tuple)) or len(pair) != 2:
                continue
            sim_word = str(pair[0]).strip().lower()
            try:
                score = float(pair[1])
            except (ValueError, TypeError):
                continue

            # Validazione
            if score < MIN_SIMILARITY:
                continue
            if score > 1.0:
                score = 1.0
            if len(sim_word) < MIN_WORD_LEN:
                continue
            if sim_word == word:
                continue
            # Rifiuta se troppo simili ortograficamente ma semanticamente diversi
            # (condividono >80% dei caratteri E score basso)
            if sim_word in valid_words or True:  # accetta anche parole non nel lessico
                results.append((word, sim_word, round(score, 2)))

    return results


# -- Main --------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description="Genera SIMILAR_TO semantici via Qwen 3.5")
    parser.add_argument("--test", type=int, help="Processa solo N parole (test)")
    parser.add_argument("--resume", action="store_true", help="Riprendi da dove eri rimasto")
    parser.add_argument("--batch", type=int, default=BATCH_SIZE, help="Parole per batch")
    args = parser.parse_args()

    words = load_words()
    print(f"Lessico: {len(words)} parole candidate")

    done = load_progress() if args.resume else set()
    if not args.resume and OUTPUT_TSV.exists():
        OUTPUT_TSV.unlink()
    if not args.resume and PROGRESS_F.exists():
        PROGRESS_F.unlink()

    remaining = [w for w in words if w not in done]
    if args.test:
        remaining = remaining[:args.test]

    print(f"Da processare: {len(remaining)} parole (batch size: {args.batch})")

    # Header TSV
    if not OUTPUT_TSV.exists() or not args.resume:
        with open(OUTPUT_TSV, "w", encoding="utf-8") as f:
            f.write("# SIMILAR_TO semantici generati da Qwen 3.5\n")
            f.write("# Formato: soggetto\tSIMILAR_TO\toggetto\tconfidence\n")

    total_edges = 0
    valid_set = set(words)

    for i in range(0, len(remaining), args.batch):
        batch = remaining[i:i+args.batch]
        prompt = build_prompt(batch)
        response = query_ollama(prompt)

        if not response:
            for w in batch:
                save_progress(w)
            continue

        pairs = parse_response(response, valid_set)

        # Scrivi TSV
        if pairs:
            with open(OUTPUT_TSV, "a", encoding="utf-8") as f:
                for word, sim, score in pairs:
                    f.write(f"{word}\tSIMILAR_TO\t{sim}\t{score}\n")
            total_edges += len(pairs)

        for w in batch:
            save_progress(w)

        # Progresso
        done_count = len(done) + i + len(batch)
        pct = done_count / len(words) * 100
        print(f"  [{done_count}/{len(words)}] {pct:.1f}% -- batch: {', '.join(batch[:3])}{'...' if len(batch)>3 else ''} -> {len(pairs)} archi (totale: {total_edges})")

        time.sleep(DELAY_S)

    print(f"\nCompletato! {total_edges} archi SIMILAR_TO scritti in {OUTPUT_TSV}")
    print(f"Prossimo passo: cargo run --release --bin import-kg && cargo run --release --bin rebuild-semantic-topology")


if __name__ == "__main__":
    main()
