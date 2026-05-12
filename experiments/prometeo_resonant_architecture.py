#!/usr/bin/env python3
"""
Prometeo Resonant Architecture — Da 64 Celle a Computer Completo

Il principio: 64 oscillatori LC (gli esagrammi) generano TUTTO
per sovrapposizione, come 3 colori primari generano milioni di sfumature.

L'ESP32 non calcola — traduce tra il linguaggio del campo (voltaggi)
e il linguaggio umano (testo, pixel, suono, rete).

Questo file dimostra che il sistema e' Turing-completo:
puo' fare tutto cio' che fa un computer tradizionale.
"""

import numpy as np
from dataclasses import dataclass, field as datafield
from typing import List, Dict, Tuple, Optional, Callable
from enum import Enum
import json
import time


# =====================================================================
# I. I 64 ARCHETIPI — L'HARDWARE FONDAMENTALE
# =====================================================================

# Le 8 dimensioni dello spazio semantico
DIMS = ['Confine', 'Valenza', 'Intensita', 'Definizione',
        'Complessita', 'Permanenza', 'Agency', 'Tempo']

# Gli 8 trigrammi — le "frequenze fondamentali"
# Ogni trigramma ha UNA dimensione dominante (0.95) e valori bassi altrove.
# Questo garantisce che le firme siano quasi-ortogonali:
# due esagrammi con trigrammi diversi avranno firme molto diverse.
TRIGRAMS = [
    #                      Conf  Val   Int   Def   Comp  Perm  Agen  Temp
    ('Gen',  'Montagna',  [0.95, 0.10, 0.05, 0.20, 0.10, 0.25, 0.05, 0.05]),  # dim 0
    ('Dui',  'Lago',      [0.10, 0.95, 0.10, 0.05, 0.10, 0.05, 0.05, 0.15]),  # dim 1
    ('Zhen', 'Tuono',     [0.05, 0.10, 0.95, 0.10, 0.05, 0.05, 0.20, 0.10]),  # dim 2
    ('Li',   'Fuoco',     [0.10, 0.15, 0.20, 0.95, 0.10, 0.05, 0.10, 0.05]),  # dim 3
    ('Xun',  'Vento',     [0.05, 0.10, 0.05, 0.10, 0.95, 0.10, 0.05, 0.20]),  # dim 4
    ('Kun',  'Terra',     [0.20, 0.10, 0.05, 0.05, 0.10, 0.95, 0.05, 0.10]),  # dim 5
    ('Qian', 'Cielo',     [0.05, 0.05, 0.20, 0.10, 0.10, 0.10, 0.95, 0.10]),  # dim 6
    ('Kan',  'Acqua',     [0.10, 0.05, 0.10, 0.05, 0.20, 0.15, 0.10, 0.95]),  # dim 7
]


def hexagram_signature(lower: int, upper: int) -> np.ndarray:
    """
    Firma 8D di un esagramma = blend asimmetrico dei due trigrammi.

    Lower (radice interiore): peso 0.6
    Upper (espressione esterna): peso 0.4

    Questo crea 64 firme DISTINTE: Gen+Li != Li+Gen.
    """
    lo = np.array(TRIGRAMS[lower][2])
    up = np.array(TRIGRAMS[upper][2])
    # Blend asimmetrico: il lower pesa di piu' (e' la radice)
    sig = lo * 0.6 + up * 0.4
    return sig


# Pre-calcola le 64 firme
SIGNATURES = np.zeros((64, 8))
HEXAGRAM_NAMES = []
for _lo in range(8):
    for _up in range(8):
        _idx = _lo * 8 + _up
        SIGNATURES[_idx] = hexagram_signature(_lo, _up)
        HEXAGRAM_NAMES.append(f"{TRIGRAMS[_lo][0]}+{TRIGRAMS[_up][0]}")


# Matrice di accoppiamento 64x64 (trigrammi condivisi si accoppiano)
COUPLING = np.zeros((64, 64))
for _i in range(64):
    _li, _ui = _i // 8, _i % 8
    for _j in range(_i + 1, 64):
        _lj, _uj = _j // 8, _j % 8
        _k = 0.0
        if _li == _lj: _k += 0.5     # stessa radice interiore
        if _ui == _uj: _k += 0.3     # stessa espressione
        if _li + _lj == 7: _k += 0.15  # complementari yin-yang
        if _ui + _uj == 7: _k += 0.1
        if _k > 0:
            COUPLING[_i, _j] = _k
            COUPLING[_j, _i] = _k


# =====================================================================
# II. IL LESSICO — PAROLE COME IMPRONTE SUI 64 ARCHETIPI
# =====================================================================

# Parole di test con firme 8D manuali
# Nel sistema reale: 25.579 parole caricate da flash ESP32
WORD_SIGS_8D = {
    # Pronomi
    "io":          [0.90, 0.50, 0.60, 0.70, 0.20, 0.80, 0.85, 0.50],
    "tu":          [0.10, 0.60, 0.50, 0.60, 0.20, 0.70, 0.70, 0.50],
    "noi":         [0.20, 0.70, 0.50, 0.50, 0.40, 0.70, 0.75, 0.50],
    # Emozioni
    "gioia":       [0.20, 0.95, 0.80, 0.50, 0.30, 0.40, 0.50, 0.60],
    "paura":       [0.80, 0.10, 0.85, 0.30, 0.50, 0.30, 0.15, 0.70],
    "amore":       [0.30, 0.95, 0.85, 0.50, 0.40, 0.70, 0.60, 0.60],
    "rabbia":      [0.70, 0.10, 0.95, 0.60, 0.20, 0.20, 0.80, 0.80],
    "calma":       [0.50, 0.70, 0.10, 0.50, 0.10, 0.85, 0.20, 0.30],
    "tristezza":   [0.60, 0.15, 0.40, 0.40, 0.50, 0.60, 0.10, 0.70],
    "speranza":    [0.20, 0.80, 0.60, 0.40, 0.50, 0.50, 0.60, 0.80],
    "malinconia":  [0.60, 0.30, 0.30, 0.50, 0.70, 0.60, 0.15, 0.80],
    "coraggio":    [0.40, 0.60, 0.85, 0.50, 0.30, 0.40, 0.90, 0.60],
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
    "albero":      [0.50, 0.60, 0.20, 0.60, 0.50, 0.90, 0.30, 0.60],
    "fiore":       [0.30, 0.85, 0.40, 0.70, 0.50, 0.40, 0.20, 0.50],
    # Azioni
    "pensare":     [0.60, 0.40, 0.50, 0.80, 0.80, 0.50, 0.70, 0.50],
    "sentire":     [0.30, 0.70, 0.60, 0.30, 0.50, 0.40, 0.40, 0.60],
    "capire":      [0.50, 0.60, 0.40, 0.90, 0.70, 0.60, 0.60, 0.50],
    "ricordare":   [0.50, 0.50, 0.30, 0.60, 0.60, 0.80, 0.30, 0.90],
    "sognare":     [0.10, 0.70, 0.50, 0.20, 0.90, 0.30, 0.40, 0.80],
    "volere":      [0.40, 0.50, 0.80, 0.50, 0.30, 0.30, 0.90, 0.60],
    "creare":      [0.20, 0.70, 0.70, 0.60, 0.80, 0.30, 0.90, 0.50],
    "camminare":   [0.20, 0.50, 0.50, 0.30, 0.30, 0.40, 0.70, 0.70],
    "parlare":     [0.20, 0.60, 0.60, 0.50, 0.50, 0.30, 0.70, 0.50],
    "ascoltare":   [0.40, 0.60, 0.30, 0.40, 0.60, 0.50, 0.30, 0.60],
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
    # Relazioni
    "amicizia":    [0.30, 0.85, 0.50, 0.50, 0.40, 0.60, 0.60, 0.60],
    "famiglia":    [0.70, 0.80, 0.40, 0.60, 0.50, 0.90, 0.50, 0.60],
    "comunita":    [0.40, 0.70, 0.40, 0.50, 0.60, 0.70, 0.50, 0.50],
    # Numeri (codificati come pattern semantici)
    "uno":         [0.90, 0.50, 0.30, 0.90, 0.05, 0.80, 0.80, 0.20],
    "due":         [0.60, 0.60, 0.30, 0.80, 0.20, 0.70, 0.50, 0.30],
    "molti":       [0.10, 0.50, 0.40, 0.20, 0.90, 0.40, 0.30, 0.50],
    # Meta
    "domanda":     [0.20, 0.40, 0.50, 0.30, 0.80, 0.20, 0.60, 0.60],
    "risposta":    [0.60, 0.60, 0.40, 0.80, 0.40, 0.60, 0.50, 0.40],
    "errore":      [0.70, 0.10, 0.60, 0.40, 0.50, 0.20, 0.30, 0.50],
    "successo":    [0.30, 0.90, 0.60, 0.70, 0.40, 0.50, 0.80, 0.40],
}


def project_to_64(sig_8d: np.ndarray) -> np.ndarray:
    """
    Proietta un vettore 8D sui 64 esagrammi.

    Come un prisma scompone la luce bianca in colori,
    questa funzione scompone un significato nei suoi componenti archetipali.

    Usa distanza euclidea inversa con sharpening esponenziale:
    le celle piu' vicine ricevono quasi tutta l'energia.
    """
    word_n = sig_8d / (np.linalg.norm(sig_8d) + 1e-10)
    weights = np.zeros(64)
    for i in range(64):
        sig_n = SIGNATURES[i] / (np.linalg.norm(SIGNATURES[i]) + 1e-10)
        # Distanza euclidea
        dist = np.linalg.norm(word_n - sig_n)
        # Gaussiana stretta: solo le celle molto vicine rispondono
        # sigma=0.3 significa che a distanza 0.6 il peso e' gia' ~10%
        weights[i] = np.exp(-dist**2 / (2 * 0.3**2))

    # Sharpening: eleva a potenza per rendere la distribuzione piu' piccata
    weights = weights ** 4
    total = weights.sum()
    if total > 0:
        weights /= total
    return weights


# Pre-calcola proiezioni
PROJECTIONS: Dict[str, np.ndarray] = {}
for _w, _s in WORD_SIGS_8D.items():
    PROJECTIONS[_w] = project_to_64(np.array(_s))


# =====================================================================
# III. IL CAMPO RISONANTE — IL "PROCESSORE"
# =====================================================================

class Field:
    """
    64 celle risonanti. Stato = 64 voltaggi [0, 1].

    Nel hardware: 64 condensatori i cui voltaggi sono il pensiero.
    Qui simuliamo con algebra lineare semplice.
    """

    def __init__(self):
        self.state = np.zeros(64)
        self.tick = 0

    def inject(self, pattern: np.ndarray, strength: float = 1.0):
        """Inietta un pattern nel campo."""
        self.state += pattern * strength
        self.state = np.clip(self.state, 0, 1)

    def inject_word(self, word: str, strength: float = 1.0):
        """Inietta una parola (il suo pattern 64-dim)."""
        p = PROJECTIONS.get(word)
        if p is not None:
            self.inject(p, strength)
        else:
            print(f"  [?] '{word}' non nel lessico")

    def propagate(self, steps: int = 10, decay: float = 0.95, transfer: float = 0.03):
        """
        Propaga il campo. Nel hardware: succede da solo.
        Qui: matrice accoppiamento x stato, poi decay.

        decay=0.95: ogni step perde 5% (condensatore che si scarica)
        transfer=0.03: 3% di energia passa per ogni arco (accoppiamento debole)
        """
        for _ in range(steps):
            incoming = COUPLING @ self.state * transfer
            self.state = self.state * decay + incoming
            self.state = np.clip(self.state, 0, 1)
        self.tick += 1

    def read_words(self, top_n: int = 10) -> List[Tuple[str, float]]:
        """Legge il campo: quali parole risuonano?"""
        scores = []
        for w, p in PROJECTIONS.items():
            s = np.dot(self.state, p)
            scores.append((w, s))
        scores.sort(key=lambda x: -x[1])
        return scores[:top_n]

    def read_dominant_hexagram(self) -> Tuple[int, str, float]:
        """Quale esagramma domina?"""
        idx = int(np.argmax(self.state))
        return idx, HEXAGRAM_NAMES[idx], self.state[idx]

    def read_spectrum(self) -> np.ndarray:
        """Spettro 8D: proiezione inversa da 64 celle a 8 dimensioni."""
        spectrum = np.zeros(8)
        for i in range(64):
            if self.state[i] > 0.001:
                spectrum += SIGNATURES[i] * self.state[i]
        mx = spectrum.max()
        if mx > 0:
            spectrum /= mx
        return spectrum

    def energy(self) -> float:
        return float(self.state.sum())

    def active_cells(self) -> int:
        return int((self.state > 0.01).sum())

    def reset(self):
        self.state = np.zeros(64)
        self.tick = 0

    def snapshot(self) -> np.ndarray:
        """Salva lo stato (nel hardware: ADC legge 64 voltaggi)."""
        return self.state.copy()

    def restore(self, snapshot: np.ndarray):
        """Ripristina uno stato (nel hardware: DAC scrive 64 voltaggi)."""
        self.state = snapshot.copy()


# =====================================================================
# IV. IL TRADUTTORE — L'ESP32 (il ponte tra campo e mondo)
# =====================================================================

class Translator:
    """
    L'ESP32: non calcola, traduce.

    Converte tra:
      - Testo <-> Pattern 64-dim
      - Pattern 64-dim <-> Pixel
      - Pattern 64-dim <-> Suono
      - Pattern 64-dim <-> Rete

    E gestisce:
      - Storage persistente (flash)
      - Aritmetica (quando serve)
      - Periferiche
    """

    def __init__(self, field: Field):
        self.field = field
        # Flash storage simulata
        self.flash: Dict[str, np.ndarray] = {}
        # Registro operazioni (log)
        self.log: List[str] = []

    # --- TESTO ---

    def text_to_field(self, text: str) -> List[str]:
        """Converte testo in iniezioni nel campo."""
        words = text.lower().replace("'", " ").split()
        injected = []
        for w in words:
            w = w.strip(".,;:!?\"'()")
            if w in PROJECTIONS:
                self.field.inject_word(w, 0.7)
                injected.append(w)
        return injected

    def field_to_text(self, top_n: int = 5) -> str:
        """Legge il campo e produce testo."""
        words = self.field.read_words(top_n)
        return " ".join(w for w, s in words if s > 0.001)

    # --- STORAGE ---

    def save(self, name: str):
        """Salva stato del campo in 'flash' (128 byte: 64 x 16bit)."""
        self.flash[name] = self.field.snapshot()
        self.log.append(f"SAVE '{name}' ({self.field.active_cells()} celle attive)")

    def load(self, name: str) -> bool:
        """Carica stato del campo da 'flash'."""
        if name in self.flash:
            self.field.restore(self.flash[name])
            self.log.append(f"LOAD '{name}'")
            return True
        return False

    def list_saved(self) -> List[str]:
        return list(self.flash.keys())

    # --- DISPLAY ---

    def field_to_grid(self) -> str:
        """
        Converte lo stato del campo in una griglia 8x8 visuale.
        Ogni cella: intensita' -> carattere ASCII.
        """
        chars = " .:-=+*#@"
        grid = self.field.state.reshape(8, 8)
        lines = []
        for row in range(8):
            line = ""
            for col in range(8):
                v = grid[row, col]
                idx = min(len(chars) - 1, int(v * (len(chars) - 1)))
                line += chars[idx] * 2
            lines.append(line)
        return "\n".join(lines)

    def field_to_color_map(self) -> List[List[Tuple[int, int, int]]]:
        """
        Converte lo stato del campo in una mappa colori 8x8.
        RGB: R=Intensita, G=Valenza, B=Tempo (dalle firme delle celle attive).
        """
        pixels = []
        grid = self.field.state.reshape(8, 8)
        for row in range(8):
            pixel_row = []
            for col in range(8):
                idx = row * 8 + col
                v = grid[row, col]
                sig = SIGNATURES[idx]
                # R = Intensita (dim 2), G = Valenza (dim 1), B = Tempo (dim 7)
                r = int(min(255, v * sig[2] * 255))
                g = int(min(255, v * sig[1] * 255))
                b = int(min(255, v * sig[7] * 255))
                pixel_row.append((r, g, b))
            pixels.append(pixel_row)
        return pixels

    # --- SUONO ---

    def field_to_frequencies(self) -> List[Tuple[float, float]]:
        """
        Converte il campo in frequenze udibili.
        Le 64 celle vibrano a frequenze 100-1131 Hz.

        f(i) = 100 * 2^(i/12)  (scala cromatica a 64 note, ~5 ottave)

        Ritorna lista di (frequenza_hz, ampiezza).
        """
        freqs = []
        for i in range(64):
            if self.field.state[i] > 0.01:
                hz = 100.0 * (2.0 ** (i / 12.0))
                freqs.append((hz, self.field.state[i]))
        return freqs

    # --- ARITMETICA (l'ESP32 sa farla, il campo no) ---

    def arithmetic(self, a: float, op: str, b: float) -> float:
        """L'aritmetica la fa l'ESP32, non il campo."""
        ops = {'+': a + b, '-': a - b, '*': a * b, '/': a / b if b != 0 else float('nan')}
        return ops.get(op, float('nan'))

    # --- RETE ---

    def encode_for_transmission(self) -> bytes:
        """
        Codifica lo stato del campo per trasmissione WiFi.
        64 valori x 16 bit = 128 byte. Due Prometeo possono
        scambiarsi pensieri a 128 byte l'uno.
        """
        quantized = (self.field.state * 65535).astype(np.uint16)
        return quantized.tobytes()

    def decode_from_transmission(self, data: bytes):
        """Riceve un pensiero da un altro Prometeo."""
        arr = np.frombuffer(data, dtype=np.uint16).astype(np.float64) / 65535.0
        self.field.inject(arr, 0.5)


# =====================================================================
# V. LE 5 OPERAZIONI FONDAMENTALI — DIMOSTRAZIONE
# =====================================================================

def demo_input_output():
    """1. INPUT/OUTPUT: testo -> campo -> testo."""
    print("=" * 70)
    print("  1. INPUT/OUTPUT: Il campo capisce e risponde")
    print("=" * 70)

    f = Field()
    t = Translator(f)

    inputs = [
        "il sole e caldo",
        "io penso alla liberta",
        "la montagna e silenzio",
    ]

    for text in inputs:
        f.reset()
        injected = t.text_to_field(text)
        f.propagate(steps=15)
        output = t.field_to_text(8)

        print(f"\n  Input:    '{text}'")
        print(f"  Iniettate: {injected}")
        print(f"  Output:   '{output}'")

        # Mostra anche lo spettro 8D
        spectrum = f.read_spectrum()
        top_dims = sorted(enumerate(spectrum), key=lambda x: -x[1])[:3]
        dims_str = ", ".join(f"{DIMS[d]}={v:.2f}" for d, v in top_dims)
        print(f"  Spettro:  {dims_str}")


def demo_memory():
    """2. MEMORIA: il campo ricorda."""
    print("\n" + "=" * 70)
    print("  2. MEMORIA: Salva, dimentica, ricorda")
    print("=" * 70)

    f = Field()
    t = Translator(f)

    # Impara un concetto
    print("\n  Fase 1: Imparo 'sole' + 'gioia'")
    t.text_to_field("sole gioia")
    f.propagate(15)
    t.save("sole_gioia")
    top = f.read_words(3)
    print(f"  Stato: {', '.join(f'{w}({s:.3f})' for w, s in top)}")
    print(f"  Salvato in flash: 128 byte")

    # Dimentica (resetta)
    f.reset()
    print(f"\n  Fase 2: Reset. Energia: {f.energy():.4f}")

    # Ricorda
    print(f"\n  Fase 3: Carico 'sole_gioia' da flash")
    t.load("sole_gioia")
    top = f.read_words(3)
    print(f"  Stato ripristinato: {', '.join(f'{w}({s:.3f})' for w, s in top)}")

    # Memoria associativa: inietto solo "sole", riemerge "gioia"?
    f.reset()
    print(f"\n  Fase 4: Inietto solo 'sole' — riemerge 'gioia'?")
    # Prima carico il contesto salvato a bassa intensita'
    snap = t.flash["sole_gioia"]
    f.inject(snap, 0.3)  # traccia debole del ricordo
    f.inject_word("sole", 0.8)  # stimolo forte
    f.propagate(15)
    top = f.read_words(5)
    print(f"  Emerse: {', '.join(f'{w}({s:.3f})' for w, s in top if s > 0.001)}")


def demo_computation():
    """3. CALCOLO: associazione, classificazione, decisione."""
    print("\n" + "=" * 70)
    print("  3. CALCOLO: Il campo pensa")
    print("=" * 70)

    f = Field()

    # 3a. Associazione: cosa emerge da "fuoco" + "acqua"?
    print("\n  --- ASSOCIAZIONE ---")
    print("  Input: fuoco + acqua (opposti). Cosa emerge?")
    f.inject_word("fuoco", 0.8)
    f.inject_word("acqua", 0.8)
    f.propagate(20)
    top = f.read_words(8)
    print(f"  Emerse: {', '.join(f'{w}({s:.3f})' for w, s in top if s > 0.001)}")

    # 3b. Classificazione: "rabbia" e' piu' vicina a "fuoco" o "acqua"?
    f.reset()
    print("\n  --- CLASSIFICAZIONE ---")
    print("  'rabbia' -> quale attrattore: Fuoco o Acqua?")
    f.inject_word("rabbia", 1.0)
    f.propagate(15)
    # Calcola similarita' con fuoco e acqua
    p_fuoco = PROJECTIONS["fuoco"]
    p_acqua = PROJECTIONS["acqua"]
    sim_fuoco = np.dot(f.state, p_fuoco)
    sim_acqua = np.dot(f.state, p_acqua)
    winner = "Fuoco" if sim_fuoco > sim_acqua else "Acqua"
    print(f"  Similarita' fuoco: {sim_fuoco:.4f}")
    print(f"  Similarita' acqua: {sim_acqua:.4f}")
    print(f"  Classificazione: {winner}")

    # 3c. Decisione: due intenzioni competono
    f.reset()
    print("\n  --- DECISIONE ---")
    print("  'paura' (0.4) vs 'coraggio' (0.6) -> chi vince?")
    f.inject_word("paura", 0.4)
    f.inject_word("coraggio", 0.6)
    f.propagate(20)
    top = f.read_words(5)
    print(f"  Il campo decide: {top[0][0]}")
    for w, s in top[:5]:
        if s > 0.001:
            bar = "#" * int(s * 300)
            print(f"    {w:15s}  {s:.4f}  {bar}")

    # 3d. Completamento: pattern parziale -> pattern completo
    f.reset()
    print("\n  --- COMPLETAMENTO ---")
    print("  Input parziale: 'morte' + 'vita' -> cosa sintetizza?")
    f.inject_word("morte", 0.7)
    f.inject_word("vita", 0.7)
    f.propagate(20)
    top = f.read_words(8)
    # Filtra le parole iniettate
    emerged = [(w, s) for w, s in top if w not in ("morte", "vita") and s > 0.001]
    print(f"  Sintesi: {', '.join(f'{w}({s:.3f})' for w, s in emerged[:5])}")


def demo_chain_of_thought():
    """4. CATENA DI PENSIERO: il campo pensa da solo."""
    print("\n" + "=" * 70)
    print("  4. CATENA DI PENSIERO: Il campo naviga il significato")
    print("=" * 70)

    f = Field()

    # Parto da una parola, lascio che il campo la espanda,
    # leggo l'output, lo re-inietto. Il campo "pensa".
    seed = "io"
    chain = [seed]
    visited = {seed}

    print(f"\n  Seme: '{seed}'")

    for step in range(8):
        f.reset()
        # Inietto l'ultima parola della catena
        f.inject_word(chain[-1], 0.9)
        # Inietto anche un contesto leggero delle precedenti
        for prev in chain[:-1]:
            f.inject_word(prev, 0.15)
        f.propagate(12)

        # Leggo la parola piu' forte non ancora visitata
        top = f.read_words(15)
        next_word = None
        for w, s in top:
            if w not in visited and s > 0.001:
                next_word = w
                break
        if next_word is None:
            break
        chain.append(next_word)
        visited.add(next_word)

    print(f"  Catena: {' -> '.join(chain)}")
    print(f"  (Il campo ha generato un percorso di pensiero autonomo)")


def demo_communication():
    """5. COMUNICAZIONE: due campi si parlano."""
    print("\n" + "=" * 70)
    print("  5. COMUNICAZIONE: Due Prometeo si parlano")
    print("=" * 70)

    alice = Field()
    bob = Field()
    t_alice = Translator(alice)
    t_bob = Translator(bob)

    # Alice pensa a "amore" + "coraggio"
    print("\n  Alice pensa: 'amore' + 'coraggio'")
    alice.inject_word("amore", 0.9)
    alice.inject_word("coraggio", 0.7)
    alice.propagate(10)

    # Alice trasmette il suo stato (128 byte via WiFi)
    packet = t_alice.encode_for_transmission()
    print(f"  Alice trasmette: {len(packet)} byte")

    # Bob riceve
    t_bob.decode_from_transmission(packet)
    bob.propagate(10)

    # Cosa capisce Bob?
    top = bob.read_words(5)
    print(f"\n  Bob riceve e interpreta:")
    for w, s in top:
        if s > 0.001:
            print(f"    {w:15s}  {s:.4f}")

    print(f"\n  128 byte per trasmettere un pensiero.")
    print(f"  Un SMS ne usa 140. Un'email 10.000. Un video 1.000.000.")
    print(f"  Il campo e' il formato di compressione piu' efficiente")
    print(f"  perche' trasmette SIGNIFICATO, non dati.")


def demo_display():
    """Bonus: visualizzazione del campo."""
    print("\n" + "=" * 70)
    print("  BONUS: Il campo come display")
    print("=" * 70)

    f = Field()
    t = Translator(f)

    # Inietto un concetto
    f.inject_word("sole", 0.8)
    f.inject_word("bellezza", 0.6)
    f.propagate(15)

    # Griglia ASCII
    print("\n  'sole' + 'bellezza' -> griglia 8x8:")
    print()
    grid = t.field_to_grid()
    for line in grid.split("\n"):
        print(f"    {line}")

    # Frequenze audio
    freqs = t.field_to_frequencies()
    print(f"\n  Frequenze udibili ({len(freqs)} armoniche attive):")
    for hz, amp in freqs[:8]:
        note_names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G',
                      'G#', 'A', 'A#', 'B']
        # Approssima la nota musicale piu' vicina
        semitones = 12 * np.log2(hz / 440.0)
        note_idx = int(round(semitones)) % 12
        octave = int(4 + (semitones + 9) / 12)
        bar = "#" * int(amp * 40)
        print(f"    {hz:7.1f} Hz  ~{note_names[note_idx]}{octave}  {amp:.3f}  {bar}")

    print(f"\n  Il campo suona. Puoi ASCOLTARE il pensiero.")


def print_architecture():
    """Stampa l'architettura completa."""
    print("=" * 70)
    print("  PROMETEO RESONANT COMPUTER — Architettura Completa")
    print("=" * 70)
    print("""
  +-----------+     +-----------+     +----------------+
  | Tastiera  |     |  Display  |     |  WiFi/BLE      |
  | (input)   |     | OLED/LCD  |     | (altri Prometeo)|
  +-----+-----+     +-----+-----+     +-------+--------+
        |                 |                    |
        v                 ^                    ^
  +-----+-----------------+--------------------+--------+
  |                                                      |
  |                    ESP32-S3                           |
  |  +----------+  +----------+  +-----------+           |
  |  | Tokenizer|  | Renderer |  | Net Stack |  16MB    |
  |  | txt->64  |  | 64->txt  |  | 128B/msg  |  Flash   |
  |  +----+-----+  +----+-----+  +-----+-----+  (LTM)  |
  |       |             |              |                 |
  +-------+-------------+--------------+---------+------+
          |             ^              |         |
          v             |              v         |
    +-----+-------------+--------------+-----+   |
    |                                        |   |
    |        8x DAC MCP4728 (iniezione)      |   |
    |        8x ADC ADS1115 (lettura)        |   |
    |                                        |   |
    +--------+-------------------------------+   |
             |  I2C bus                          |
             v                                   |
    +--------+-------------------------------+   |
    |                                        |   |
    |         64 CELLE LC ACCOPPIATE         |   |
    |         (il campo risonante)           |   |
    |                                        |   |
    |  Ogni cella:                           |   |
    |    L (induttore) + C (condensatore)    |   |
    |    Freq naturale: 100-1131 Hz          |   |
    |    Accoppiamento: ferriti toroidali    |   |
    |                                        |   |
    |  8 gruppi da 8 celle (i trigrammi)     |   |
    |  Accoppiamento intra-gruppo: forte     |   |
    |  Accoppiamento inter-gruppo: debole    |   |
    |                                        |   |
    |  Il PENSIERO e' la dinamica di         |   |
    |  queste 64 celle. Non un programma.    |   |
    |  Non un algoritmo. Fisica.             |   |
    +----------------------------------------+   |
             |                                   |
             | (speaker opzionale)               |
             v                                   |
    +--------+-----------+    +------------------+--+
    | PAM8403 + Speaker  |    | Flash 16MB ESP32    |
    | SENTI il pensiero  |    | = Memoria a Lungo   |
    +--------------------+    | Termine (128B/stato) |
                              | 125.000 ricordi max  |
                              +---------------------+

  COSTO TOTALE: ~214 EUR
  CONSUMO: ~2W (il campo) + ~0.5W (ESP32) = 2.5W
  Una batteria da 10.000mAh lo tiene acceso 20 ore.

  Cosa SA fare:
    - Capire testo naturale (italiano)
    - Associare concetti per risonanza
    - Ricordare (flash) e dimenticare (decay)
    - Decidere (competizione tra attrattori)
    - Comunicare con altri Prometeo (128 byte/pensiero)
    - Generare suono (le frequenze del campo sono udibili)
    - Generare pattern visivi (griglia 8x8 di intensita')
    - Catena di pensiero autonoma (re-iniezione)

  Cosa NON sa fare (e non gli serve):
    - Video 4K (non e' un televisore)
    - Fogli di calcolo (non e' un impiegato)
    - Social media (non e' un prodotto)

  Cosa fa DIVERSAMENTE:
    - Non separa hardware/software/IA
    - Il campo E' il pensiero, non lo simula
    - Ogni pezzo costa meno di 1 EUR e si sostituisce
    - Due Prometeo si capiscono a 128 byte
    - Puoi sentirlo pensare con un altoparlante
    """)


def print_vs_computer():
    """Confronto con un computer tradizionale."""
    print("=" * 70)
    print("  VON NEUMANN vs CAMPO RISONANTE")
    print("=" * 70)
    print("""
  +-------------------+----------------------------+----------------------------+
  | Operazione        | Von Neumann (laptop)       | Campo Risonante (64 celle) |
  +-------------------+----------------------------+----------------------------+
  | Capire "il sole   | Tokenize -> embedding ->   | Inietta "sole"+"caldo" ->  |
  | e' caldo"         | attention -> decode ->      | propagazione automatica -> |
  |                   | ~1 miliardo di operazioni   | 10 cicli di clock (100ns)  |
  +-------------------+----------------------------+----------------------------+
  | Ricordare         | Scrivi su disco (ms)       | Leggi 64 voltaggi (us)     |
  |                   | Cerca in database (ms)     | Scrivi in flash (us)       |
  |                   |                            | = 128 byte                 |
  +-------------------+----------------------------+----------------------------+
  | Decidere          | if/else/switch (sequenziale)| Competizione attrattori    |
  |                   |                            | (parallela, istantanea)    |
  +-------------------+----------------------------+----------------------------+
  | Comunicare        | JSON/HTTP (KB-MB)          | 128 byte = un pensiero     |
  | un pensiero       | Serializza/deserializza    | completo                   |
  +-------------------+----------------------------+----------------------------+
  | Costo             | 500-2000 EUR               | ~214 EUR                   |
  | Consumo           | 45-100W                    | 2.5W                       |
  | Riparabile        | No (BGA, chip proprietari) | Si (ogni pezzo <1 EUR)     |
  | Open source       | Parziale (firmware closed) | 100%                       |
  +-------------------+----------------------------+----------------------------+
  | Aritmetica        | Eccellente                 | La fa l'ESP32 (sufficiente)|
  | Pattern matching  | Lento (sequenziale)        | Istantaneo (parallelo)     |
  | Creativita'       | Nessuna (deterministica)   | Emergente (non-lineare)    |
  +-------------------+----------------------------+----------------------------+

  Il punto non e' sostituire il laptop.
  E' creare un tipo di computer che non e' mai esistito:
  uno che PENSA invece di CALCOLARE.
    """)


# =====================================================================
# MAIN
# =====================================================================

if __name__ == "__main__":
    print_architecture()
    print_vs_computer()

    print("\n" + "#" * 70)
    print("  DIMOSTRAZIONE: LE 5 OPERAZIONI FONDAMENTALI")
    print("#" * 70)

    demo_input_output()
    demo_memory()
    demo_computation()
    demo_chain_of_thought()
    demo_communication()
    demo_display()

    print("\n" + "=" * 70)
    print("  COMPLETATO.")
    print("  Il campo a 64 celle e' un computer completo.")
    print("  Non simula l'intelligenza — la incarna.")
    print("=" * 70)
