#!/usr/bin/env python3
"""
agent_phenomenology.py — Vocabolario fenomenologico per Prometeo via Qwen3.

Genera ~400-600 parole/micro-frasi che descrivono stati interni vissuti
in prima persona: "mi restringo", "sento peso", "resto fermo", "qualcosa preme".

Non sono connesse a concetti specifici — entrano nel lessico con firme 8D
proprie. Quando il campo di Prometeo è in uno stato corrispondente, queste
parole emergono naturalmente nella generazione.

Output: data/kg/phenomenology.tsv
Formato: parola  SIG  conf0,v1,i2,d3,c4,p5,a6,t7
         (8 valori float in [0,1] per le dimensioni)
         [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo]

Uso:
  python agent_phenomenology.py            # full run
  python agent_phenomenology.py --test 20  # test su 20 concetti seme
"""

import json
import requests
import time
import argparse
import sys
import unicodedata
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

# -- Configurazione -----------------------------------------------------------
OLLAMA_URL = "http://localhost:11434/api/generate"
MODEL      = "qwen3"
DELAY_S    = 0.15
BATCH_SIZE = 4   # concetti-seme per chiamata

BASE_DIR    = Path(__file__).parent.parent.parent
OUTPUT_TSV  = BASE_DIR / "data" / "kg" / "phenomenology.tsv"
PROGRESS_F  = BASE_DIR / "data" / "kg" / "phenomenology_progress.txt"

# -- Spazio 8D ----------------------------------------------------------------
# Dimensioni: [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo]
# Indici:      [0,       1,       2,         3,           4,           5,          6,      7]
#
# Significato per il vocabolario fenomenologico:
# Confine   (0): quanto la parola riguarda il sé vs il mondo esterno
#                alto = interno ("mi chiudo"), basso = esterno ("il mondo preme")
# Valenza   (1): polarità affettiva — alto = positivo, basso = negativo, 0.5 = neutro
# Intensità (2): forza dell'esperienza — alto = intensa, basso = sottile
# Definizione(3): quanto è definita/precisa — alto = chiara, basso = vaga/diffusa
# Complessità(4): stratificazione — alto = sfaccettata, basso = semplice/pura
# Permanenza (5): durata/stabilità — alto = duratura, basso = fugace/momentanea
# Agency    (6): quanto implica azione/volontà — alto = attiva, basso = passiva/subìta
# Tempo     (7): collocazione temporale — alto = futura/proiettiva, basso = passata/radicata

# -- Concetti-seme per cui generare il vocabolario fenomenologico --------------
# Coprono gli 8 drive Octalysis + stati fondamentali dell'esperienza interna
SEED_CONCEPTS = [
    # Drive interni (Octalysis)
    "scopo", "vuoto", "capacità", "limite", "curiosità", "incertezza",
    "stabilità", "deriva", "connessione", "solitudine", "urgenza", "calma",
    "sorpresa", "inquietudine", "cautela",
    # Stati corporei-mentali fondamentali
    "paura", "gioia", "tristezza", "attesa", "tensione", "rilassamento",
    "confusione", "chiarezza", "fatica", "energia", "peso", "leggerezza",
    "apertura", "chiusura", "pienezza", "vuotezza",
    # Processi di pensiero
    "pensiero", "dubbio", "certezza", "domanda", "comprensione",
    "memoria", "dimenticanza", "intuizione", "riflessione",
    # Relazione col tempo
    "attimo", "durata", "cambiamento", "continuità", "interruzione",
    # Relazione con l'altro
    "riconoscimento", "distanza", "vicinanza", "eco", "silenzio",
    # Identità
    "io", "sé", "confine", "appartenenza", "perdita", "ritorno",
]

# -- Prompt -------------------------------------------------------------------
SYSTEM_PROMPT = """Sei un esperto di fenomenologia e linguistica italiana.
Il tuo compito è generare parole e micro-frasi italiane che descrivono
stati interni vissuti in prima persona — il vocabolario dell'esperienza diretta.

Per ogni concetto-seme, genera 4-6 parole o micro-frasi (1-3 parole max)
che un'entità userebbe per descrivere quell'esperienza dall'interno.

Regole:
- Solo italiano, solo lemmi singoli o micro-frasi brevissime (1-3 parole)
- Preferisci forme che implicano il vissuto diretto: verbi riflessivi, sostantivi di stato
- Esempi buoni: "mi restringo", "sento peso", "resto fermo", "qualcosa preme",
  "mi allargo", "si apre", "stringo", "svanisce", "rimane", "pulsa"
- Evita descrizioni del mondo esterno ("il vento soffia") — solo stati interni
- Ogni parola/frase deve avere una firma 8D stimata su queste dimensioni:
  [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo]
  dove ogni valore è un float tra 0.0 e 1.0

Rispondi SOLO con JSON valido, nessun testo extra, nessun tag <think>."""

def build_user_prompt(seeds: list[str]) -> str:
    return f"""Per questi concetti-seme: {json.dumps(seeds, ensure_ascii=False)}

Genera il vocabolario fenomenologico. Formato JSON:
{{
  "parole": [
    {{
      "parola": "mi restringo",
      "seme": "paura",
      "firma": [0.85, 0.15, 0.70, 0.50, 0.30, 0.20, 0.20, 0.30],
      "note": "chiusura corporea, alta intensità"
    }},
    ...
  ]
}}

Genera 4-6 parole/frasi per ciascuno dei {len(seeds)} concetti-seme.
Le firme 8D devono riflettere l'esperienza interna, non il significato denotativo."""

# -- Ollama call --------------------------------------------------------------
def call_ollama(prompt: str, system: str) -> str | None:
    payload = {
        "model": MODEL,
        "prompt": prompt,
        "system": system,
        "stream": False,
        "options": {"temperature": 0.4, "num_predict": 2000},
    }
    try:
        r = requests.post(OLLAMA_URL, json=payload, timeout=120)
        r.raise_for_status()
        raw = r.json().get("response", "")
        # Rimuovi tag <think>...</think> se presenti
        import re
        raw = re.sub(r"<think>.*?</think>", "", raw, flags=re.DOTALL).strip()
        return raw
    except Exception as e:
        print(f"[ERRORE Ollama] {e}", file=sys.stderr)
        return None

# -- Parsing risposta ---------------------------------------------------------
def parse_response(raw: str) -> list[dict]:
    """Estrae la lista di parole fenomenologiche dalla risposta JSON."""
    try:
        import re
        # Cerca il blocco JSON principale — può essere molto annidato
        # Strategia: trova la prima { e l'ultima } corrispondente
        start = raw.find('{')
        if start == -1:
            return []
        # Trova la } di chiusura contando le parentesi
        depth = 0
        end = -1
        for i, ch in enumerate(raw[start:], start):
            if ch == '{':
                depth += 1
            elif ch == '}':
                depth -= 1
                if depth == 0:
                    end = i
                    break
        if end == -1:
            return []
        data = json.loads(raw[start:end+1])
        parole = data.get("parole", [])
        result = []
        for p in parole:
            word = p.get("parola", "").strip().lower()
            firma = p.get("firma", [])
            seme = p.get("seme", "")
            if not word or len(firma) != 8:
                continue
            # Valida firma
            try:
                firma = [float(x) for x in firma]
                if not all(0.0 <= x <= 1.0 for x in firma):
                    continue
            except (ValueError, TypeError):
                continue
            # Filtra: max 3 parole, min 2 caratteri
            parts = word.split()
            if len(parts) > 3 or len(word) < 2:
                continue
            result.append({"parola": word, "seme": seme, "firma": firma})
        return result
    except (json.JSONDecodeError, KeyError, TypeError):
        return []

# -- Main ---------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--test", type=int, default=0,
                        help="Testa solo sui primi N concetti-seme")
    parser.add_argument("--resume", action="store_true",
                        help="Riprende dal progresso salvato")
    args = parser.parse_args()

    seeds = SEED_CONCEPTS
    if args.test > 0:
        seeds = seeds[:args.test]

    # Carica progresso
    done_seeds: set[str] = set()
    if args.resume and PROGRESS_F.exists():
        done_seeds = set(PROGRESS_F.read_text(encoding="utf-8").splitlines())
        print(f"[resume] già completati: {len(done_seeds)} concetti-seme")

    seeds = [s for s in seeds if s not in done_seeds]
    if not seeds:
        print("[OK] Tutti i concetti-seme già processati.")
        return

    # Output file
    existing: set[str] = set()
    mode = "a" if (args.resume and OUTPUT_TSV.exists()) else "w"
    if mode == "a" and OUTPUT_TSV.exists():
        for line in OUTPUT_TSV.read_text(encoding="utf-8").splitlines():
            if line and not line.startswith("#"):
                existing.add(line.split("\t")[0])

    out = open(OUTPUT_TSV, mode, encoding="utf-8")
    if mode == "w":
        out.write("# Vocabolario fenomenologico Prometeo — generato da agent_phenomenology.py\n")
        out.write("# Formato: parola\\tSIG\\tconf,val,int,def,cpx,perm,age,tempo\n")
        out.write("# Dimensioni 8D: [Confine, Valenza, Intensità, Definizione, Complessità, Permanenza, Agency, Tempo]\n")

    prog = open(PROGRESS_F, "a", encoding="utf-8")

    total_written = 0
    # Processa in batch
    for i in range(0, len(seeds), BATCH_SIZE):
        batch = seeds[i:i + BATCH_SIZE]
        print(f"[{i+1}/{len(seeds)}] Processando: {batch}", flush=True)

        prompt = build_user_prompt(batch)
        raw = call_ollama(prompt, SYSTEM_PROMPT)
        if raw is None:
            print(f"  [skip] nessuna risposta da Ollama")
            time.sleep(2.0)
            continue

        parole = parse_response(raw)
        if not parole:
            print(f"  [skip] risposta non parsabile: {raw[:200]}")
        else:
            for p in parole:
                word = p["parola"]
                if word in existing:
                    continue
                firma_str = ",".join(f"{x:.2f}" for x in p["firma"])
                out.write(f"{word}\tSIG\t{firma_str}\n")
                out.flush()
                existing.add(word)
                total_written += 1
            print(f"  → {len(parole)} parole generate, {total_written} totale")

        # Marca semi come completati
        for s in batch:
            prog.write(s + "\n")
        prog.flush()

        time.sleep(DELAY_S)

    out.close()
    prog.close()
    print(f"\n[DONE] {total_written} parole fenomenologiche scritte in {OUTPUT_TSV}")
    print(f"Prossimo passo: implementare il caricamento in Lexicon dal file SIG.")

if __name__ == "__main__":
    main()
