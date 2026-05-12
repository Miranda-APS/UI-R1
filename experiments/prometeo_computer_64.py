#!/usr/bin/env python3
"""
Prometeo Resonant Computer — 64 Esagrammi Hardware

Il campo non SIMULA il pensiero — È il pensiero.
L'ESP32 non CALCOLA — traduce tra linguaggi (voltaggio <-> testo).

Architettura:
    64 celle LC (analogiche) <-- I2C --> ESP32 (traduttore) <-- periferiche

    Costo: ~214 EUR. Tutti componenti standard, tutti sostituibili.
"""

import numpy as np
import matplotlib.pyplot as plt
from dataclasses import dataclass
from typing import List, Dict, Tuple

# ═══════════════════════════════════════════════════════════════════════
# I. GLI 8 TRIGRAMMI — I COLORI PRIMARI DEL SIGNIFICATO
# ═══════════════════════════════════════════════════════════════════════

DIM_NAMES = ['Confine', 'Valenza', 'Intensita', 'Definizione',
             'Complessita', 'Permanenza', 'Agency', 'Tempo']

TRIGRAM_NAMES = ['Gen', 'Dui', 'Zhen', 'Li', 'Xun', 'Kun', 'Qian', 'Kan']
TRIGRAM_SYMBOLS = ['Mountain', 'Lake', 'Thunder', 'Fire', 'Wind', 'Earth', 'Heaven', 'Water']

# Ogni trigramma ha un profilo di risonanza sulle 8 dimensioni
# Questi sono i "colori primari" — tutto il resto ne deriva
TRIGRAM_PROFILES = np.array([
    #  Conf  Val   Int   Def   Comp  Perm  Agen  Temp
    [0.95, 0.20, 0.10, 0.80, 0.30, 0.90, 0.40, 0.15],  # Gen/Mountain
    [0.30, 0.90, 0.50, 0.40, 0.35, 0.50, 0.30, 0.60],  # Dui/Lake
    [0.20, 0.40, 0.95, 0.30, 0.20, 0.15, 0.70, 0.80],  # Zhen/Thunder
    [0.40, 0.60, 0.70, 0.95, 0.50, 0.20, 0.60, 0.40],  # Li/Fire
    [0.15, 0.50, 0.30, 0.50, 0.95, 0.40, 0.35, 0.70],  # Xun/Wind
    [0.70, 0.60, 0.05, 0.30, 0.40, 0.95, 0.10, 0.50],  # Kun/Earth
    [0.30, 0.50, 0.80, 0.60, 0.60, 0.50, 0.95, 0.40],  # Qian/Heaven
    [0.50, 0.30, 0.40, 0.20, 0.80, 0.60, 0.50, 0.95],  # Kan/Water
])


# ═══════════════════════════════════════════════════════════════════════
# II. I 64 ESAGRAMMI — LE CELLE HARDWARE
# ═══════════════════════════════════════════════════════════════════════

def build_signatures() -> np.ndarray:
    """
    Costruisce le 64 firme 8D degli esagrammi.

    Ogni esagramma = coppia (lower, upper) di trigrammi.
    Firma = media pesata dei profili, con rinforzo sulle dimensioni
    dominanti di ciascun trigramma.

    Returns: matrice [64, 8] di firme normalizzate
    """
    sigs = np.zeros((64, 8))
    for lower in range(8):
        for upper in range(8):
            idx = lower * 8 + upper
            low = TRIGRAM_PROFILES[lower]
            up = TRIGRAM_PROFILES[upper]

            # Media dei profili
            sig = (low + up) / 2

            # Rinforzo: le dimensioni dove un trigramma e' dominante (>0.8)
            # vengono amplificate — sono il "carattere" dell'esagramma
            for d in range(8):
                if low[d] > 0.8 or up[d] > 0.8:
                    sig[d] = max(sig[d], max(low[d], up[d]) * 0.9)

            sigs[idx] = sig

    return sigs


# Pre-calcola firme e matrice di accoppiamento
SIGNATURES = build_signatures()  # [64, 8]

def build_coupling() -> np.ndarray:
    """
    Matrice di accoppiamento 64x64.

    Celle che condividono un trigramma si accoppiano.
    Nel hardware: ferriti toroidali con avvolgimenti calibrati.
    """
    coupling = np.zeros((64, 64))
    for i in range(64):
        li, ui = i // 8, i % 8
        for j in range(i+1, 64):
            lj, uj = j // 8, j % 8

            k = 0.0
            if li == lj:  # Stessa radice interiore
                k += 0.5
            if ui == uj:  # Stessa espressione
                k += 0.3
            if li + lj == 7:  # Complementari yin-yang
                k += 0.15
            if ui + uj == 7:
                k += 0.1

            if k > 0:
                coupling[i, j] = k
                coupling[j, i] = k

    return coupling

COUPLING = build_coupling()  # [64, 64]


# ═══════════════════════════════════════════════════════════════════════
# III. IL LESSICO — PAROLE COME PATTERN DI ECCITAZIONE
# ═══════════════════════════════════════════════════════════════════════

# Parole con firme 8D: [Confine, Valenza, Intensita, Definizione, Complessita, Permanenza, Agency, Tempo]
WORD_SIGNATURES = {
    # Identita
    "io":          [0.90, 0.50, 0.60, 0.70, 0.20, 0.80, 0.85, 0.50],
    "tu":          [0.10, 0.60, 0.50, 0.60, 0.20, 0.70, 0.70, 0.50],
    "noi":         [0.20, 0.70, 0.50, 0.50, 0.40, 0.70, 0.75, 0.50],
    "essere":      [0.50, 0.50, 0.30, 0.40, 0.60, 0.90, 0.30, 0.70],
    # Emozioni
    "gioia":       [0.20, 0.95, 0.80, 0.50, 0.30, 0.40, 0.50, 0.60],
    "paura":       [0.80, 0.10, 0.85, 0.30, 0.50, 0.30, 0.15, 0.70],
    "amore":       [0.30, 0.95, 0.85, 0.50, 0.40, 0.70, 0.60, 0.60],
    "rabbia":      [0.70, 0.10, 0.95, 0.60, 0.20, 0.20, 0.80, 0.80],
    "calma":       [0.50, 0.70, 0.10, 0.50, 0.10, 0.85, 0.20, 0.30],
    "tristezza":   [0.60, 0.15, 0.40, 0.40, 0.50, 0.60, 0.10, 0.70],
    "speranza":    [0.20, 0.80, 0.60, 0.40, 0.50, 0.50, 0.60, 0.80],
    "malinconia":  [0.60, 0.30, 0.30, 0.50, 0.70, 0.60, 0.15, 0.80],
    # Natura
    "sole":        [0.10, 0.80, 0.80, 0.90, 0.20, 0.70, 0.70, 0.50],
    "luna":        [0.30, 0.60, 0.30, 0.70, 0.60, 0.70, 0.20, 0.80],
    "acqua":       [0.20, 0.50, 0.40, 0.20, 0.80, 0.80, 0.10, 0.90],
    "fuoco":       [0.40, 0.50, 0.95, 0.90, 0.30, 0.10, 0.70, 0.70],
    "terra":       [0.70, 0.50, 0.10, 0.50, 0.40, 0.95, 0.10, 0.30],
    "vento":       [0.05, 0.40, 0.50, 0.20, 0.90, 0.20, 0.30, 0.70],
    "montagna":    [0.95, 0.30, 0.10, 0.80, 0.40, 0.95, 0.30, 0.20],
    "mare":        [0.10, 0.50, 0.50, 0.20, 0.90, 0.80, 0.10, 0.90],
    "cielo":       [0.05, 0.60, 0.50, 0.40, 0.70, 0.80, 0.80, 0.50],
    "stella":      [0.10, 0.70, 0.60, 0.90, 0.50, 0.90, 0.20, 0.90],
    # Azioni mentali
    "pensare":     [0.60, 0.40, 0.50, 0.80, 0.80, 0.50, 0.70, 0.50],
    "sentire":     [0.30, 0.70, 0.60, 0.30, 0.50, 0.40, 0.40, 0.60],
    "capire":      [0.50, 0.60, 0.40, 0.90, 0.70, 0.60, 0.60, 0.50],
    "ricordare":   [0.50, 0.50, 0.30, 0.60, 0.60, 0.80, 0.30, 0.90],
    "sognare":     [0.10, 0.70, 0.50, 0.20, 0.90, 0.30, 0.40, 0.80],
    "volere":      [0.40, 0.50, 0.80, 0.50, 0.30, 0.30, 0.90, 0.60],
    "creare":      [0.20, 0.70, 0.70, 0.60, 0.80, 0.30, 0.90, 0.50],
    # Concetti
    "tempo":       [0.20, 0.30, 0.30, 0.30, 0.70, 0.70, 0.10, 0.95],
    "spazio":      [0.30, 0.30, 0.20, 0.50, 0.80, 0.80, 0.20, 0.40],
    "liberta":     [0.05, 0.80, 0.70, 0.40, 0.60, 0.30, 0.90, 0.50],
    "giustizia":   [0.70, 0.60, 0.50, 0.90, 0.60, 0.80, 0.60, 0.40],
    "verita":      [0.50, 0.60, 0.40, 0.95, 0.50, 0.90, 0.50, 0.40],
    "bellezza":    [0.30, 0.90, 0.60, 0.80, 0.70, 0.50, 0.30, 0.50],
    "morte":       [0.90, 0.10, 0.50, 0.60, 0.50, 0.90, 0.05, 0.90],
    "vita":        [0.20, 0.80, 0.70, 0.40, 0.70, 0.60, 0.80, 0.70],
    "casa":        [0.80, 0.70, 0.10, 0.70, 0.30, 0.90, 0.40, 0.30],
    "cammino":     [0.10, 0.50, 0.50, 0.30, 0.50, 0.40, 0.70, 0.80],
    "luce":        [0.10, 0.80, 0.70, 0.95, 0.30, 0.40, 0.50, 0.50],
    "ombra":       [0.60, 0.20, 0.20, 0.30, 0.70, 0.60, 0.10, 0.70],
    "silenzio":    [0.70, 0.40, 0.05, 0.50, 0.60, 0.80, 0.10, 0.60],
    "parola":      [0.30, 0.60, 0.50, 0.70, 0.60, 0.40, 0.60, 0.50],
    "caldo":       [0.20, 0.60, 0.70, 0.50, 0.20, 0.30, 0.30, 0.50],
    "freddo":      [0.60, 0.20, 0.30, 0.50, 0.30, 0.50, 0.10, 0.50],
}


def project_word(word_8d: np.ndarray) -> np.ndarray:
    """
    Proietta un vettore 8D sui 64 esagrammi.

    Ogni esagramma "risponde" in proporzione a quanto la sua firma
    e' vicina al vettore della parola. Gaussiana stretta = selettivita'.
    """
    word_norm = word_8d / (np.linalg.norm(word_8d) + 1e-10)

    weights = np.zeros(64)
    for i in range(64):
        sig_norm = SIGNATURES[i] / (np.linalg.norm(SIGNATURES[i]) + 1e-10)
        # Cosine similarity
        cos_sim = np.dot(word_norm, sig_norm)
        # Solo le celle sufficientemente allineate rispondono
        if cos_sim > 0.7:
            # Potenza 8 per sharpening: solo i match migliori contano
            weights[i] = (cos_sim - 0.7) ** 2

    total = np.sum(weights)
    if total > 0:
        weights /= total

    return weights


# Pre-calcola proiezioni per tutto il lessico
LEXICON_PROJECTIONS: Dict[str, np.ndarray] = {}
for _word, _sig in WORD_SIGNATURES.items():
    LEXICON_PROJECTIONS[_word] = project_word(np.array(_sig))


# ═══════════════════════════════════════════════════════════════════════
# IV. IL COMPUTER RISONANTE
# ═══════════════════════════════════════════════════════════════════════

class ResonantComputer:
    """
    Il computer a 64 celle risonanti.

    Stato = vettore di 64 attivazioni [0, 1].
    Nel hardware: 64 voltaggi sui condensatori.
    """

    def __init__(self):
        self.activation = np.zeros(64)  # Stato del campo
        self.tick = 0

    def inject(self, word: str, strength: float = 1.0):
        """Inietta una parola nel campo."""
        proj = LEXICON_PROJECTIONS.get(word)
        if proj is None:
            print(f"  [!] '{word}' non nel lessico")
            return
        self.activation += proj * strength
        self.activation = np.clip(self.activation, 0, 1)

    def propagate(self, steps: int = 10):
        """
        Propaga il campo per N step.

        Nel hardware: succede da solo (corrente nei circuiti LC accoppiati).
        Qui simuliamo: ogni step, le celle attive trasferiscono energia
        alle celle accoppiate, poi tutto decade.
        """
        DECAY = 0.92
        TRANSFER = 0.08  # Quanto energia passa per ogni arco per step

        for _ in range(steps):
            # Energia trasferita: matrice accoppiamento * attivazioni
            incoming = COUPLING @ self.activation * TRANSFER
            # Nuova attivazione = vecchia decaduta + incoming
            self.activation = self.activation * DECAY + incoming
            self.activation = np.clip(self.activation, 0, 1)

        self.tick += 1

    def read(self, top_n: int = 10) -> List[Tuple[str, float]]:
        """
        Legge il campo e restituisce le parole piu' rilevanti.

        Per ogni parola: score = dot(field_state, word_projection).
        Le parole il cui pattern di eccitazione e' piu' allineato
        con lo stato attuale del campo emergono con score piu' alto.
        """
        results = []
        for word, proj in LEXICON_PROJECTIONS.items():
            score = np.dot(self.activation, proj)
            results.append((word, score))
        results.sort(key=lambda x: -x[1])
        return results[:top_n]

    def read_fractals(self, top_n: int = 5) -> List[Tuple[int, str, float]]:
        """Legge quali esagrammi dominano."""
        results = []
        for i in range(64):
            if self.activation[i] > 0.001:
                lower, upper = i // 8, i % 8
                name = f"{TRIGRAM_NAMES[lower]}+{TRIGRAM_NAMES[upper]}"
                results.append((i, name, self.activation[i]))
        results.sort(key=lambda x: -x[2])
        return results[:top_n]

    def reset(self):
        self.activation = np.zeros(64)

    def total_energy(self) -> float:
        return float(np.sum(self.activation))

    def active_cells(self) -> int:
        return int(np.sum(self.activation > 0.01))


# ═══════════════════════════════════════════════════════════════════════
# V. DEMO
# ═══════════════════════════════════════════════════════════════════════

def demo_thinking():
    """Dimostra che il campo pensa: inietti parole, ne emergono altre."""
    print("=" * 70)
    print("  PROMETEO RESONANT COMPUTER — 64 Celle Esagramma")
    print("=" * 70)

    c = ResonantComputer()

    n_couplings = np.count_nonzero(COUPLING) // 2
    n_words = len(LEXICON_PROJECTIONS)
    flash_kb = n_words * (64 * 2 + 32) / 1024

    print(f"\n  Celle: 64 | Accoppiamenti: {n_couplings} | Lessico: {n_words}")
    print(f"  Flash per 25K parole: {25000 * 160 / 1024 / 1024:.1f} MB (ESP32 ne ha 16)")

    # --- Diagnostica proiezioni ---
    print(f"\n  Diagnostica proiezioni (celle attive per parola):")
    for word in ["sole", "caldo", "io", "amore", "montagna", "vento"]:
        proj = LEXICON_PROJECTIONS[word]
        n_active = np.sum(proj > 0.001)
        top_cell = np.argmax(proj)
        print(f"    {word:12s}  celle attive: {n_active:2.0f}  "
              f"top: #{top_cell} ({TRIGRAM_NAMES[top_cell//8]}+{TRIGRAM_NAMES[top_cell%8]}) = {proj[top_cell]:.3f}")

    # --- Test 1: "il sole e' caldo" ---
    print("\n" + "-" * 70)
    print("  TEST 1: Inietto 'sole' + 'caldo'")
    print("-" * 70)

    c.inject("sole", 0.8)
    c.inject("caldo", 0.6)

    print(f"  Dopo iniezione: energia={c.total_energy():.3f}  celle attive={c.active_cells()}")

    c.propagate(steps=15)

    print(f"  Dopo propagazione: energia={c.total_energy():.3f}  celle attive={c.active_cells()}")

    emerged = c.read(15)
    injected = {"sole", "caldo"}
    print(f"\n  Parole emerse:")
    for word, score in emerged:
        if score < 0.0001:
            continue
        marker = " << INIETTATA" if word in injected else ""
        bar = "#" * int(score * 500)
        print(f"    {word:15s}  {score:.4f}  {bar}{marker}")

    fractals = c.read_fractals()
    if fractals:
        print(f"\n  Frattali dominanti:")
        for fid, fname, act in fractals:
            print(f"    #{fid:2d} {fname:20s}  {act:.4f}")

    # --- Test 2: "io penso" ---
    c.reset()
    print("\n" + "-" * 70)
    print("  TEST 2: Inietto 'io' + 'pensare'")
    print("-" * 70)

    c.inject("io", 0.9)
    c.inject("pensare", 0.7)
    c.propagate(steps=15)

    emerged = c.read(10)
    injected = {"io", "pensare"}
    print(f"\n  Parole emerse:")
    for word, score in emerged:
        if score < 0.0001:
            continue
        marker = " << INIETTATA" if word in injected else ""
        bar = "#" * int(score * 500)
        print(f"    {word:15s}  {score:.4f}  {bar}{marker}")

    # --- Test 3: "amore" da solo ---
    c.reset()
    print("\n" + "-" * 70)
    print("  TEST 3: Inietto solo 'amore' — cosa attrae?")
    print("-" * 70)

    c.inject("amore", 1.0)
    c.propagate(steps=15)

    emerged = c.read(10)
    print(f"\n  Parole emerse:")
    for word, score in emerged:
        if score < 0.0001:
            continue
        marker = " << INIETTATA" if word == "amore" else ""
        bar = "#" * int(score * 500)
        print(f"    {word:15s}  {score:.4f}  {bar}{marker}")

    return c


def demo_memory():
    """Il campo ricorda: i condensatori mantengono la carica."""
    print("\n" + "=" * 70)
    print("  MEMORIA: Il campo ricorda")
    print("=" * 70)

    c = ResonantComputer()

    # Fase 1: impara "sole" + "gioia"
    print("\n  Fase 1: Inietto 'sole' + 'gioia'")
    c.inject("sole", 0.8)
    c.inject("gioia", 0.7)
    c.propagate(steps=10)

    top3 = c.read(3)
    print(f"  Top 3: {', '.join(f'{w}({s:.3f})' for w, s in top3)}")
    print(f"  Energia: {c.total_energy():.3f}")

    # Fase 2: decade
    print("\n  Fase 2: 30 step di decay (nessun input)...")
    for _ in range(30):
        c.propagate(steps=1)

    top3 = c.read(3)
    print(f"  Top 3 (tracce): {', '.join(f'{w}({s:.3f})' for w, s in top3)}")
    print(f"  Energia residua: {c.total_energy():.3f}")

    # Fase 3: ri-inietta solo "sole" — "gioia" riemerge?
    print("\n  Fase 3: Inietto solo 'sole' — riemerge 'gioia'?")
    c.inject("sole", 0.5)
    c.propagate(steps=10)

    emerged = c.read(8)
    print(f"  Parole emerse:")
    for w, s in emerged:
        if s < 0.0001:
            continue
        note = " << ricordo!" if w == "gioia" else ""
        print(f"    {w:15s}  {s:.4f}{note}")


def demo_as_computer():
    """Il campo come computer generico."""
    print("\n" + "=" * 70)
    print("  IL CAMPO COME COMPUTER")
    print("=" * 70)

    c = ResonantComputer()

    # 1. CLASSIFICAZIONE
    print("\n  --- CLASSIFICAZIONE ---")
    print("  'rabbia' + 'fuoco' -> quale attrattore domina?")
    c.inject("rabbia", 0.8)
    c.inject("fuoco", 0.6)
    c.propagate(steps=10)

    fractals = c.read_fractals()
    if fractals:
        print(f"  Attrattore dominante: {fractals[0][1]} (score: {fractals[0][2]:.4f})")
    top = c.read(3)
    print(f"  Parole: {', '.join(f'{w}' for w, s in top)}")

    # 2. DECISIONE
    c.reset()
    print("\n  --- DECISIONE ---")
    print("  'paura' (0.5) vs 'speranza' (0.7) — chi vince?")
    c.inject("paura", 0.5)
    c.inject("speranza", 0.7)
    c.propagate(steps=20)

    top = c.read(5)
    print(f"  Il campo decide: {top[0][0]} (vince)")
    for w, s in top[:5]:
        if s > 0.0001:
            print(f"    {w:15s}  {s:.4f}")

    # 3. CATENA DI PENSIERO
    c.reset()
    print("\n  --- CATENA DI PENSIERO ---")
    print("  Parto da 'io', leggo l'output, lo re-inietto...")

    current = "io"
    chain = [current]
    for _ in range(6):
        c.reset()
        c.inject(current, 0.8)
        c.propagate(steps=12)
        emerged = c.read(8)
        for w, s in emerged:
            if w != current and w not in chain:
                current = w
                chain.append(current)
                break

    print(f"  Catena: {' -> '.join(chain)}")


def visualize(computer: ResonantComputer):
    """Visualizza il campo a 64 celle."""
    fig, axes = plt.subplots(2, 2, figsize=(15, 12))
    fig.suptitle("Prometeo Resonant Computer — 64 Celle", fontsize=14, fontweight='bold')

    # 1. Griglia 8x8
    ax = axes[0, 0]
    grid = computer.activation.reshape(8, 8)
    im = ax.imshow(grid, cmap='YlOrRd', vmin=0, vmax=max(0.01, np.max(grid)),
                   aspect='equal', interpolation='nearest')
    ax.set_xticks(range(8))
    ax.set_xticklabels(TRIGRAM_NAMES, rotation=45, ha='right', fontsize=8)
    ax.set_yticks(range(8))
    ax.set_yticklabels(TRIGRAM_NAMES, fontsize=8)
    ax.set_xlabel("Upper trigram")
    ax.set_ylabel("Lower trigram")
    ax.set_title("Attivazione 64 celle (8x8)")
    plt.colorbar(im, ax=ax, shrink=0.8)
    for i in range(8):
        for j in range(8):
            v = grid[i, j]
            color = 'white' if v > np.max(grid) * 0.5 else 'black'
            ax.text(j, i, f"{v:.2f}", ha='center', va='center', fontsize=6, color=color)

    # 2. Proiezione parole
    ax = axes[0, 1]
    words_show = ["sole", "amore", "pensare", "montagna"]
    colors_map = ['#e74c3c', '#e67e22', '#2ecc71', '#3498db']
    x = np.arange(64)
    w = 0.2
    for i, word in enumerate(words_show):
        proj = LEXICON_PROJECTIONS.get(word, np.zeros(64))
        ax.bar(x + i * w, proj, w, label=word, alpha=0.7, color=colors_map[i])
    ax.set_xlabel("Cella (0-63)")
    ax.set_ylabel("Peso")
    ax.set_title("Proiezione parole -> 64 celle")
    ax.legend(fontsize=8)

    # 3. Sparsita' accoppiamento
    ax = axes[1, 0]
    ax.spy(COUPLING, markersize=1, color='navy')
    ax.set_title(f"Matrice accoppiamento ({np.count_nonzero(COUPLING)//2} archi)")
    ax.set_xlabel("Cella j")
    ax.set_ylabel("Cella i")

    # 4. Spettro 8D
    ax = axes[1, 1]
    spectrum = np.zeros(8)
    for i in range(64):
        if computer.activation[i] > 0.001:
            spectrum += SIGNATURES[i] * computer.activation[i]
    if np.max(spectrum) > 0:
        spectrum /= np.max(spectrum)

    bars = ax.bar(range(8), spectrum, color=plt.cm.Set2(np.linspace(0, 1, 8)))
    ax.set_xticks(range(8))
    ax.set_xticklabels(DIM_NAMES, rotation=45, ha='right', fontsize=9)
    ax.set_ylabel("Intensita")
    ax.set_title("Spettro 8D del campo")
    ax.set_ylim(0, 1.1)

    plt.tight_layout()
    out = "experiments/prometeo_computer_64.png"
    plt.savefig(out, dpi=150, bbox_inches='tight')
    print(f"\n  Salvato: {out}")
    plt.close()


# ═══════════════════════════════════════════════════════════════════════
# VI. SPECS & BOM
# ═══════════════════════════════════════════════════════════════════════

def print_specs():
    print("\n" + "=" * 70)
    print("  SPECIFICHE HARDWARE")
    print("=" * 70)

    print("""
  ARCHITETTURA:

  [Tastiera/Display/WiFi] <-> [ESP32 Traduttore] <-> [64 Celle LC]

  Il campo (64 celle) pensa. L'ESP32 traduce. Le periferiche comunicano.

  BOM:""")

    items = [
        ("Induttori 10mH", 512, 0.08),
        ("Condensatori ceramici (8 valori)", 512, 0.04),
        ("Resistenze 0.5 ohm", 512, 0.02),
        ("Ferriti toroidali T37-2", 200, 0.25),
        ("Filo smaltato 0.3mm (100m)", 1, 5.00),
        ("ESP32-S3 DevKit (16MB flash)", 1, 8.00),
        ("ADS1115 ADC 16-bit (x8)", 8, 2.00),
        ("MCP4728 DAC 12-bit (x8)", 8, 2.50),
        ("PCB custom JLCPCB (x5)", 5, 4.00),
        ("OLED SSD1306 128x64", 1, 3.00),
        ("Amplificatore PAM8403 + speaker", 1, 2.50),
        ("Alimentatore USB-C 5V", 1, 5.00),
        ("Connettori/misc", 1, 10.00),
    ]

    total = 0
    for name, qty, price in items:
        t = qty * price
        total += t
        print(f"    {name:<40s} {qty:>4d} x {price:>5.2f} = {t:>6.2f}")
    print(f"    {'':40s} {'TOTALE':>14s} = {total:>6.2f} EUR")

    print(f"""
  vs Laptop: ~500 EUR, 45W, non riparabile, closed source.
  Questo:    ~{total:.0f} EUR,  2W, ogni pezzo da 0.05 EUR, 100% open.

  Cosa fa meglio: associazione semantica, pensiero analogico, 0.5W
  Cosa fa peggio: aritmetica (la fa l'ESP32), storage (flash ESP32)
  Cosa fa diversamente: non separa HW/SW/IA. Il campo E' l'intelligenza.
    """)


def print_how_computer():
    """Come diventa un computer completo."""
    print("=" * 70)
    print("  COME DIVENTA UN COMPUTER COMPLETO")
    print("=" * 70)
    print("""
  5 operazioni fondamentali di un computer:

  1. INPUT:   tastiera -> ESP32 -> pattern 64 pesi -> iniezione LC
  2. OUTPUT:  ADC legge 64 ampiezze -> lessico -> testo/suono
  3. MEMORIA: STM = condensatori (secondi), LTM = flash ESP32 (permanente)
  4. CALCOLO: associazione, completamento, classificazione (campo)
              aritmetica (ESP32)
  5. DECISIONE: competizione tra attrattori nel campo

  BONUS: le frequenze sono 100-1131 Hz = UDIBILI.
  Puoi ASCOLTARE il campo pensare collegando uno speaker.

  Il paradigma:
  Von Neumann = impiegato (segue istruzioni, una alla volta)
  Campo risonante = organismo (risponde al mondo, tutto insieme)
  ESP32 = interprete (traduce tra i due linguaggi)
    """)


# ═══════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print_specs()
    print_how_computer()
    computer = demo_thinking()
    demo_memory()
    demo_as_computer()
    visualize(computer)

    print("\n" + "=" * 70)
    print("  COMPLETATO. Grafico: experiments/prometeo_computer_64.png")
    print("=" * 70)
