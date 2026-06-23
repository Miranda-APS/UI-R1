#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
bench_determinism.py — l'asserzione scientifica resa eseguibile:

  STESSO INPUT (stesso stato) → STESSA LETTURA, sempre.

Esegue la catena di comprensione PURA (probe_extract: tokenizzazione →
detect_speaker_claim → extract_propositions → confront_with_kg) DUE volte
sull'intero corpus e verifica che gli output siano byte-identici. Qualunque
divergenza (iterazione HashMap non ordinata, dipendenza da stato nascosto,
randomicità) fa fallire il gate.

NB: copre lo strato comprensione (stateless per costruzione). Lo strato atto
vive nell'engine con stato — il suo determinismo è "stesso input + stesso
stato → stesso output" e si verifica col bench dialogico, non qui.

Uso:
  python bench_determinism.py            # corpus default
  python bench_determinism.py file.txt   # corpus alternativo
Exit 0 = deterministico; 1 = divergenza (mostra il diff).
"""

import subprocess
import sys
from pathlib import Path

CORPUS = Path(sys.argv[1] if len(sys.argv) > 1 else "bench_corpus_extended.txt")
PROBE = Path("target/release/probe_extract.exe")

if not PROBE.exists():
    raise SystemExit("probe_extract non compilato: cargo build --release --bin probe_extract")

sentences = [
    l.strip() for l in CORPUS.read_text(encoding="utf-8").splitlines()
    if l.strip() and not l.strip().startswith("#")
]
stdin_data = "\n".join(sentences).encode("utf-8")


def run():
    p = subprocess.run([str(PROBE)], input=stdin_data, capture_output=True, timeout=600)
    if p.returncode != 0:
        raise SystemExit(f"probe_extract exit {p.returncode}:\n{p.stderr.decode('utf-8', 'replace')[:2000]}")
    return p.stdout.decode("utf-8", "replace")


print(f"corpus: {CORPUS} ({len(sentences)} frasi) — due esecuzioni…")
a, b = run(), run()

if a == b:
    print(f"DETERMINISTICO ✓  ({len(a.splitlines())} righe identiche su due run)")
    sys.exit(0)

print("DIVERGENZA ✗ — la comprensione non è riproducibile:")
for la, lb in zip(a.splitlines(), b.splitlines()):
    if la != lb:
        print(f"  run1: {la}")
        print(f"  run2: {lb}")
sys.exit(1)
