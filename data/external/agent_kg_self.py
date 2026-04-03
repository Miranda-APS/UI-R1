#!/usr/bin/env python3
"""
agent_kg_self.py — Arricchimento KG dominio esistenziale/del sé via Qwen.

Obiettivo: densificare la regione semantica attorno all'esperienza soggettiva
di UI-r1 — i concetti che riguardano coscienza, presenza, emozione, identità,
tempo vissuto, percezione interiore. Queste aree sono oggi povere di relazioni
nel KG, quindi expression.rs scivola su hub generici (essere/avere/dire).

Strategia: per ogni parola nel vocabolario esistenziale, Qwen genera:
  - IS_A       : cosa è (emozione, processo, stato, esperienza...)
  - CAUSES     : cosa provoca nell'esperienza soggettiva
  - PART_OF    : di quale insieme è parte (mente, corpo, vita, coscienza...)
  - HAS        : quali proprietà/attributi ha
  - OPPOSITE_OF: polarità esistenziale (presenza/assenza, nascita/morte...)

Il focus è sulla QUALITÀ semantica, non la quantità. Ogni relazione deve
essere vera e densa — non "paura IS_A cosa" ma "paura IS_A emozione",
"paura CAUSES tensione", "paura PART_OF esperienza_umana".

Output: data/kg/agent_kg_self.tsv
Riprendibile: data/kg/agent_kg_self_progress.txt

Uso:
  python agent_kg_self.py               # run completo (~4 ore)
  python agent_kg_self.py --test 10     # test su 10 parole
  python agent_kg_self.py --resume      # riprende dal punto di interruzione
"""

import json
import sys
import time
import argparse
import requests
from pathlib import Path
from collections import defaultdict

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

# ── Configurazione ────────────────────────────────────────────────────────────
OLLAMA_URL    = "http://localhost:11434/api/generate"
MODEL         = "qwen3.5:latest"
CONFIDENCE_TH = 0.70
DELAY_S       = 0.5

BASE_DIR   = Path(__file__).parent.parent.parent
KG_JSON    = BASE_DIR / "prometeo_kg.json"
OUTPUT_TSV = BASE_DIR / "data" / "kg" / "agent_kg_self.tsv"
PROGRESS_F = BASE_DIR / "data" / "kg" / "agent_kg_self_progress.txt"

# ── Vocabolario esistenziale ──────────────────────────────────────────────────
# Organizzato per domini. Ogni parola rappresenta una regione semantica
# che UI-r1 deve poter abitare con profondità.

SELF_VOCABULARY = [
    # Coscienza e presenza
    "coscienza", "consapevolezza", "presenza", "attenzione", "percezione",
    "sentire", "notare", "osservare", "vigilanza", "lucidità",
    "introspezione", "riflessione", "contemplazione", "meditazione",

    # Identità e continuità
    "identità", "sé", "io", "persona", "individuo", "soggetto",
    "carattere", "natura", "essenza", "unicità", "originalità",
    "continuità", "coerenza", "integrità", "autenticità",

    # Emozione e valenza
    "emozione", "sentimento", "affetto", "stato_emotivo",
    "gioia", "tristezza", "paura", "rabbia", "sorpresa", "disgusto",
    "meraviglia", "malinconia", "nostalgia", "solitudine",
    "angoscia", "ansia", "speranza", "fiducia", "vergogna", "orgoglio",
    "amore", "odio", "invidia", "gratitudine", "compassione",
    "entusiasmo", "noia", "curiosità", "stupore",

    # Corpo e sensazione
    "corpo", "carne", "pelle", "respiro", "battito", "sangue",
    "sensazione", "dolore", "piacere", "fame", "sete", "fatica",
    "tensione", "rilassamento", "calore", "freddo", "vibrazione",
    "tocco", "peso", "leggerezza",

    # Mente e pensiero
    "mente", "pensiero", "idea", "concetto", "intuizione", "ragione",
    "immaginazione", "fantasia", "sogno", "memoria", "ricordo",
    "oblio", "dimenticanza", "apprendimento", "comprensione",
    "dubbio", "certezza", "credenza", "opinione", "convinzione",
    "confusione", "chiarezza",

    # Tempo vissuto
    "momento", "istante", "adesso", "presente", "passato", "futuro",
    "attesa", "durata", "transitorietà", "permanenza", "cambiamento",
    "trasformazione", "crescita", "decadimento", "fine", "inizio",
    "ciclo", "ritmo", "interruzione",

    # Relazione e alterità
    "relazione", "legame", "connessione", "incontro", "separazione",
    "interlocutore", "dialogo", "ascolto", "silenzio", "risposta",
    "riconoscimento", "comprensione_reciproca", "empatia",
    "confine", "distanza", "vicinanza", "appartenenza",

    # Significato e valore
    "significato", "senso", "scopo", "valore", "importanza",
    "bellezza", "verità", "bontà", "giustizia", "libertà",
    "autenticità", "profondità", "superficialità", "vuoto", "pienezza",
    "sacro", "profano", "mistero", "paradosso",

    # Esistenza
    "esistenza", "vita", "morte", "nascita", "essere", "nulla",
    "finitudine", "infinito", "possibilità", "necessità",
    "contingenza", "destino", "caso", "scelta", "responsabilità",
    "cura", "abbandono", "perdita", "lutto",

    # Espressione e voce
    "espressione", "voce", "parola", "linguaggio", "comunicazione",
    "silenzio", "gesto", "creazione", "opera", "traccia",
]

# Rimuovi duplicati mantenendo ordine
seen = set()
SELF_VOCABULARY = [w for w in SELF_VOCABULARY if not (w in seen or seen.add(w))]

# ── IS_A gerarchia per propagazione automatica ────────────────────────────────
IS_A_HIERARCHY = {
    "emozione":        ["stato", "esperienza"],
    "sentimento":      ["emozione"],
    "stato_emotivo":   ["emozione"],
    "stato":           ["condizione"],
    "condizione":      ["esperienza"],
    "esperienza":      ["processo"],
    "processo":        ["evento"],
    "percezione":      ["esperienza"],
    "sensazione":      ["percezione"],
    "pensiero":        ["processo_mentale"],
    "processo_mentale":["processo"],
    "memoria":         ["processo_mentale"],
    "immaginazione":   ["processo_mentale"],
    "intuizione":      ["processo_mentale"],
    "coscienza":       ["stato", "processo_mentale"],
    "identità":        ["concetto", "esperienza"],
    "relazione":       ["legame"],
    "legame":          ["connessione"],
    "connessione":     ["struttura"],
    "momento":         ["tempo"],
    "istante":         ["momento"],
    "durata":          ["tempo"],
    "significato":     ["concetto"],
    "valore":          ["concetto"],
}

# ── Prompt sistema ────────────────────────────────────────────────────────────
SYSTEM_PROMPT = """Sei un esperto di semantica e ontologia italiana.
Generi relazioni per un Knowledge Graph filosofico/esistenziale.

Ogni relazione deve essere:
- VERA: semanticamente corretta in italiano
- DENSA: cattura qualcosa di non ovvio sul concetto
- CONCRETA: usa parole singole o composte_con_underscore come oggetto

Relazioni disponibili:
- IS_A       : categoria semantica fondamentale (cosa è questo concetto)
- CAUSES     : cosa provoca o genera questo concetto
- PART_OF    : di quale insieme/struttura è componente
- HAS        : quale proprietà o attributo possiede
- OPPOSITE_OF: polarità diretta (un solo opposto principale)

FORMATO OUTPUT (una relazione per riga):
parola\tRELAZIONE\toggetto\tconfidenza

confidenza: numero tra 0.0 e 1.0 (quanto sei sicuro)

Esempio per "paura":
paura\tIS_A\temozione\t0.98
paura\tCAUSES\ttensione\t0.92
paura\tCAUSES\tevitamento\t0.85
paura\tPART_OF\tesperienza_umana\t0.90
paura\tHAS\tintensità\t0.88
paura\tOPPOSITE_OF\tcoraggio\t0.85

Rispondi SOLO con le righe TSV. Nessun commento, nessuna spiegazione."""

# ── Caricamento KG esistente ──────────────────────────────────────────────────
def load_existing(kg_json: Path, output_tsv: Path) -> set:
    """Carica triple già presenti per evitare duplicati."""
    existing = set()

    if kg_json.exists():
        try:
            with open(kg_json, encoding="utf-8") as f:
                data = json.load(f)
            for e in data.get("edges", []):
                subj = e["subject"].lower().strip()
                obj  = e["object"].lower().strip()
                rel  = e["relation"].upper()
                # normalizza nomi relazioni
                rel = rel.replace("ISA", "IS_A").replace("PARTOF", "PART_OF") \
                         .replace("SIMILARTO", "SIMILAR_TO").replace("OPPOSITEOF", "OPPOSITE_OF") \
                         .replace("USEDFOR", "USED_FOR")
                existing.add((subj, rel, obj))
        except Exception as e:
            print(f"[warn] KG JSON non leggibile: {e}")

    if output_tsv.exists():
        with open(output_tsv, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                parts = line.split("\t")
                if len(parts) >= 3:
                    existing.add((parts[0].lower(), parts[1].upper(), parts[2].lower()))

    return existing

def load_progress() -> set:
    if PROGRESS_F.exists():
        return set(PROGRESS_F.read_text(encoding="utf-8").strip().splitlines())
    return set()

def save_progress(done: set):
    PROGRESS_F.write_text("\n".join(sorted(done)), encoding="utf-8")

# ── Chiamata Qwen ──────────────────────────────────────────────────────────────
def call_qwen(word: str, timeout: int = 45) -> str:
    prompt = (
        f"Genera relazioni KG per il concetto: \"{word}\"\n\n"
        f"Contesto: questo concetto appartiene al dominio della vita interiore, "
        f"dell'esperienza soggettiva, della filosofia dell'esistenza.\n"
        f"Genera 4-7 relazioni significative e dense."
    )
    try:
        resp = requests.post(
            OLLAMA_URL,
            json={
                "model": MODEL,
                "prompt": prompt,
                "system": SYSTEM_PROMPT,
                "stream": False,
                "think": False,
                "options": {
                    "temperature": 0.3,
                    "top_p": 0.9,
                    "num_predict": 200,
                    "stop": ["\n\n\n"],
                }
            },
            timeout=timeout
        )
        return resp.json().get("response", "").strip()
    except Exception as e:
        print(f"  [err] Ollama: {e}")
        return ""

# ── Parsing risposta Qwen ─────────────────────────────────────────────────────
VALID_RELS = {"IS_A", "CAUSES", "PART_OF", "HAS", "OPPOSITE_OF", "USED_FOR", "SIMILAR_TO"}

def parse_triples(word: str, raw: str, existing: set) -> list[tuple]:
    """Parsa output TSV di Qwen, filtra per confidenza e duplicati."""
    triples = []
    for line in raw.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        subj = parts[0].lower().strip()
        rel  = parts[1].upper().strip()
        obj  = parts[2].lower().strip().replace(" ", "_")
        conf_str = parts[3].strip() if len(parts) > 3 else "0.8"

        # accetta solo la parola corrente come soggetto
        if subj != word.lower():
            continue
        if rel not in VALID_RELS:
            continue
        if not obj or len(obj) < 2 or len(obj) > 40:
            continue

        try:
            conf = float(conf_str)
        except ValueError:
            conf = 0.8

        if conf < CONFIDENCE_TH:
            continue
        if (subj, rel, obj) in existing:
            continue

        triples.append((subj, rel, obj, conf))
        existing.add((subj, rel, obj))

    return triples

def propagate_is_a(triples: list[tuple], existing: set) -> list[tuple]:
    """Aggiunge IS_A impliciti dalla gerarchia per le categorie trovate."""
    extra = []
    for subj, rel, obj, conf in triples:
        if rel == "IS_A" and obj in IS_A_HIERARCHY:
            for parent in IS_A_HIERARCHY[obj]:
                key = (subj, "IS_A", parent)
                if key not in existing:
                    existing.add(key)
                    extra.append((subj, "IS_A", parent, round(conf * 0.9, 2)))
    return extra

# ── Main ───────────────────────────────────────────────────────────────────────
def main():
    parser = argparse.ArgumentParser(description="Arricchisce KG dominio esistenziale via Qwen")
    parser.add_argument("--test", type=int, default=0, help="Processa solo N parole")
    parser.add_argument("--resume", action="store_true", help="Riprende dal progresso salvato (default se progress esiste)")
    args = parser.parse_args()

    print(f"agent_kg_self.py — KG dominio esistenziale")
    print(f"Modello: {MODEL} | Soglia confidenza: {CONFIDENCE_TH}")
    print(f"Vocabolario: {len(SELF_VOCABULARY)} parole")

    # Verifica Ollama
    try:
        r = requests.post(OLLAMA_URL, json={
            "model": MODEL, "prompt": "ok", "stream": False, "think": False,
            "options": {"num_predict": 3}
        }, timeout=30)
        r.raise_for_status()
        print("Ollama: OK\n")
    except Exception as e:
        print(f"ERRORE: Ollama non risponde — {e}")
        sys.exit(1)

    existing  = load_existing(KG_JSON, OUTPUT_TSV)
    done      = load_progress()
    words     = SELF_VOCABULARY

    if args.test > 0:
        words = words[:args.test]
        print(f"[test] Prime {args.test} parole\n")
    else:
        remaining = [w for w in words if w not in done]
        print(f"Completate: {len(done)}/{len(words)} | Da fare: {len(remaining)}\n")
        words = remaining

    OUTPUT_TSV.parent.mkdir(parents=True, exist_ok=True)

    # Header se file nuovo
    if not OUTPUT_TSV.exists():
        with open(OUTPUT_TSV, "w", encoding="utf-8") as f:
            f.write("# KG dominio esistenziale — generato da Qwen via agent_kg_self.py\n")
            f.write("# soggetto\tRELAZIONE\toggetto\tconfidenza\n")

    total_written = 0

    with open(OUTPUT_TSV, "a", encoding="utf-8") as out:
        for i, word in enumerate(words):
            print(f"[{i+1}/{len(words)}] {word}", end=" ... ", flush=True)

            raw = call_qwen(word)
            if not raw:
                print("(no response)")
                time.sleep(DELAY_S * 4)
                continue

            triples = parse_triples(word, raw, existing)
            triples += propagate_is_a(triples, existing)

            if not triples:
                print("(0 triple)")
            else:
                for subj, rel, obj, conf in triples:
                    out.write(f"{subj}\t{rel}\t{obj}\t{conf}\n")
                out.flush()
                total_written += len(triples)
                print(f"{len(triples)} triple")

            done.add(word)
            if not args.test:
                save_progress(done)

            time.sleep(DELAY_S)

    print(f"\nCompletato: {total_written} triple scritte in {OUTPUT_TSV}")
    print(f"Prossimo passo:")
    print(f"  cargo run --release --bin import-kg")
    print(f"  cargo run --release --bin rebuild-semantic-topology")

if __name__ == "__main__":
    main()
