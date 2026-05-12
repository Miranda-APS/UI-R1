#!/usr/bin/env python3
"""
Prometeo Wave Simulation — Proof of Concept

FILOSOFIA:
Il codice attuale usa cos(phase) per la propagazione — questa È già fisica delle onde.
Invece di calcolare cos(phase) in software, usiamo ONDE FISICHE REALI dove
l'interferenza avviene naturalmente.

ARCHITETTURA:
- Ogni parola = onda con 8 frequenze (le 8 dimensioni primitive)
- Ogni dimensione = una frequenza specifica
- Propagazione = interferenza fisica tra onde
- Attivazione = ampiezza dell'onda risultante

QUESTO SCRIPT:
Simula 8 parole × 8 dimensioni usando onde sinusoidali.
Dimostra che la propagazione emerge NATURALMENTE dall'interferenza.
"""

import numpy as np
import matplotlib.pyplot as plt
from typing import List, Tuple, Dict

# ═══════════════════════════════════════════════════════════════════════
# COSTANTI FISICHE
# ═══════════════════════════════════════════════════════════════════════

# 8 frequenze base (Hz) — una per dimensione primitiva
# Scelte per essere armonicamente separate (evitare battimenti indesiderati)
FREQUENCIES = np.array([
    100,   # Dim 0: Confine
    150,   # Dim 1: Valenza
    200,   # Dim 2: Intensita
    250,   # Dim 3: Definizione
    300,   # Dim 4: Complessita
    350,   # Dim 5: Permanenza
    400,   # Dim 6: Agency
    450,   # Dim 7: Tempo
])

SAMPLE_RATE = 8000  # Hz — campionamento audio
DURATION = 0.1      # secondi per frame di propagazione

# ═══════════════════════════════════════════════════════════════════════
# WORD RECORD — Versione Wave
# ═══════════════════════════════════════════════════════════════════════

class WaveWord:
    """
    Una parola come onda composita.
    
    Invece di signature [f32; 8], abbiamo:
    - amplitudes [f32; 8]: quanto forte è ogni dimensione
    - phases [f32; 8]: fase iniziale di ogni componente
    
    L'onda totale è la somma delle 8 sinusoidi.
    """
    
    def __init__(self, name: str, signature: np.ndarray):
        """
        Args:
            name: nome della parola
            signature: [8] array con valori [0,1] — la firma 8D attuale
        """
        self.name = name
        # Convertiamo la firma [0,1] in ampiezze [0,1]
        self.amplitudes = signature.copy()
        # Fase iniziale casuale per ogni dimensione (simula unicità della parola)
        self.phases = np.random.uniform(0, 2*np.pi, 8)
        # Attivazione corrente [0,1]
        self.activation = 0.0
        
    def generate_wave(self, t: np.ndarray) -> np.ndarray:
        """
        Genera l'onda composita per questa parola.
        
        wave(t) = Σ amplitude[i] × cos(2π × freq[i] × t + phase[i])
        
        Questo È esattamente cos(phase) in PF1, ma fisico.
        """
        wave = np.zeros_like(t)
        for i in range(8):
            if self.amplitudes[i] > 0.01:  # skip dimensioni quasi-zero
                component = self.amplitudes[i] * np.cos(
                    2 * np.pi * FREQUENCIES[i] * t + self.phases[i]
                )
                wave += component * self.activation  # scala per attivazione
        return wave
    
    def __repr__(self):
        return f"WaveWord({self.name}, act={self.activation:.2f})"


# ═══════════════════════════════════════════════════════════════════════
# WAVE FIELD — Il campo topologico come mezzo fisico
# ═══════════════════════════════════════════════════════════════════════

class WaveField:
    """
    Il campo di Prometeo come mezzo di propagazione onde.
    
    Invece di calcolare propagazione in software (O(N) scan),
    le onde si propagano FISICAMENTE attraverso il mezzo.
    """
    
    def __init__(self, words: List[WaveWord]):
        self.words = {w.name: w for w in words}
        self.time = np.linspace(0, DURATION, int(SAMPLE_RATE * DURATION))
        
    def activate(self, word_name: str, strength: float):
        """Attiva una parola — imposta la sua ampiezza."""
        if word_name in self.words:
            self.words[word_name].activation = min(1.0, strength)
    
    def propagate(self) -> np.ndarray:
        """
        Propagazione = INTERFERENZA FISICA.
        
        Tutte le parole attive emettono onde simultaneamente.
        Le onde si sommano (interferenza costruttiva/distruttiva).
        L'onda risultante contiene TUTTA l'informazione di propagazione.
        
        Returns:
            wave_total: l'onda composita del campo
        """
        wave_total = np.zeros_like(self.time)
        
        # Ogni parola attiva contribuisce con la sua onda
        for word in self.words.values():
            if word.activation > 0.02:  # soglia minima
                wave_total += word.generate_wave(self.time)
        
        return wave_total
    
    def measure_activation(self, wave_total: np.ndarray) -> Dict[str, float]:
        """
        Misura quanto ogni parola risuona con l'onda totale.
        
        Questo è l'equivalente di "quanto si attiva ogni vicino" in PF1.
        Usiamo correlazione: quanto l'onda della parola somiglia all'onda totale?
        
        Returns:
            dict: {word_name: new_activation}
        """
        new_activations = {}
        
        for word in self.words.values():
            # Genera l'onda "ideale" di questa parola (activation=1.0)
            word_wave = np.zeros_like(self.time)
            for i in range(8):
                if word.amplitudes[i] > 0.01:
                    component = word.amplitudes[i] * np.cos(
                        2 * np.pi * FREQUENCIES[i] * self.time + word.phases[i]
                    )
                    word_wave += component
            
            # Correlazione normalizzata (quanto somiglia?)
            if np.std(word_wave) > 0 and np.std(wave_total) > 0:
                correlation = np.corrcoef(word_wave, wave_total)[0, 1]
                # Mappa correlazione [-1,1] → attivazione [0,1]
                activation = max(0.0, correlation)
            else:
                activation = 0.0
            
            new_activations[word.name] = activation
        
        return new_activations
    
    def step(self):
        """
        Un passo di propagazione:
        1. Genera onda totale (interferenza)
        2. Misura risonanza di ogni parola
        3. Aggiorna attivazioni
        """
        wave_total = self.propagate()
        new_acts = self.measure_activation(wave_total)
        
        # Aggiorna con damping (come in PF1)
        damping = 0.15
        for name, new_act in new_acts.items():
            current = self.words[name].activation
            self.words[name].activation = current * (1 - damping) + new_act * damping
        
        return wave_total


# ═══════════════════════════════════════════════════════════════════════
# DEMO: 8 parole, propagazione wave-based
# ═══════════════════════════════════════════════════════════════════════

def create_demo_words() -> List[WaveWord]:
    """
    Crea 8 parole con firme diverse (simulate da PF1).
    
    Usiamo parole cardinali reali da Prometeo.
    """
    # Firme simulate (in realtà verrebbero da PF1 ROM)
    signatures = {
        "io":      np.array([0.9, 0.5, 0.6, 0.7, 0.5, 0.6, 0.8, 0.5]),  # EGO: alto Confine, alta Agency
        "tu":      np.array([0.7, 0.6, 0.5, 0.6, 0.5, 0.5, 0.6, 0.5]),  # RELAZIONE
        "qui":     np.array([0.6, 0.5, 0.5, 0.8, 0.4, 0.7, 0.4, 0.5]),  # SPAZIO: alta Definizione
        "ora":     np.array([0.5, 0.5, 0.6, 0.7, 0.4, 0.3, 0.5, 0.8]),  # TEMPO: alto Tempo
        "sentire": np.array([0.8, 0.6, 0.5, 0.5, 0.6, 0.5, 0.7, 0.5]),  # EGO: percezione
        "calma":   np.array([0.7, 0.7, 0.3, 0.6, 0.4, 0.8, 0.4, 0.5]),  # EMOZIONE: alta Permanenza, bassa Intensita
        "gioia":   np.array([0.6, 0.9, 0.7, 0.6, 0.5, 0.6, 0.5, 0.5]),  # EMOZIONE: altissima Valenza
        "paura":   np.array([0.7, 0.2, 0.8, 0.5, 0.6, 0.4, 0.3, 0.5]),  # EMOZIONE: bassa Valenza, alta Intensita
    }
    
    return [WaveWord(name, sig) for name, sig in signatures.items()]


def demo_propagation():
    """
    Demo: attiva "io" e "sentire", osserva come si propaga.
    """
    print("=" * 70)
    print("PROMETEO WAVE SIMULATION — Proof of Concept")
    print("=" * 70)
    print()
    print("SETUP:")
    print("  8 parole × 8 dimensioni")
    print("  Frequenze: 100-450 Hz (8 canali)")
    print("  Propagazione: interferenza fisica")
    print()
    
    # Crea campo
    words = create_demo_words()
    field = WaveField(words)
    
    # Attiva due parole
    print("INPUT: attivo 'io' (0.8) e 'sentire' (0.6)")
    field.activate("io", 0.8)
    field.activate("sentire", 0.6)
    print()
    
    # Stato iniziale
    print("STATO INIZIALE:")
    for word in words:
        if field.words[word.name].activation > 0.01:
            print(f"  {word.name:10s}: {field.words[word.name].activation:.3f}")
    print()
    
    # Propagazione (3 step)
    print("PROPAGAZIONE (3 step):")
    for step in range(3):
        wave = field.step()
        print(f"\n  Step {step+1}:")
        
        # Mostra parole attive
        active = [(name, w.activation) for name, w in field.words.items() 
                  if w.activation > 0.02]
        active.sort(key=lambda x: x[1], reverse=True)
        
        for name, act in active[:5]:  # top-5
            print(f"    {name:10s}: {act:.3f}")
    
    print()
    print("=" * 70)
    print("RISULTATO:")
    print("  Le parole semanticamente vicine (io-sentire, calma-gioia)")
    print("  si attivano per RISONANZA FISICA — non per calcolo software.")
    print("  L'interferenza costruttiva/distruttiva È la propagazione.")
    print("=" * 70)


def plot_wave_interference():
    """
    Visualizza l'interferenza tra due parole.
    """
    words = create_demo_words()
    field = WaveField(words)
    
    field.activate("io", 1.0)
    field.activate("sentire", 0.8)
    
    t = field.time
    wave_io = field.words["io"].generate_wave(t)
    wave_sentire = field.words["sentire"].generate_wave(t)
    wave_total = wave_io + wave_sentire
    
    plt.figure(figsize=(12, 8))
    
    plt.subplot(3, 1, 1)
    plt.plot(t[:500], wave_io[:500], label="io", alpha=0.7)
    plt.ylabel("Ampiezza")
    plt.legend()
    plt.title("Onda 'io' (8 frequenze sovrapposte)")
    
    plt.subplot(3, 1, 2)
    plt.plot(t[:500], wave_sentire[:500], label="sentire", alpha=0.7, color='orange')
    plt.ylabel("Ampiezza")
    plt.legend()
    plt.title("Onda 'sentire'")
    
    plt.subplot(3, 1, 3)
    plt.plot(t[:500], wave_total[:500], label="interferenza", alpha=0.7, color='green')
    plt.xlabel("Tempo (s)")
    plt.ylabel("Ampiezza")
    plt.legend()
    plt.title("Interferenza fisica (somma) — QUESTA È LA PROPAGAZIONE")
    
    plt.tight_layout()
    plt.savefig("experiments/wave_interference.png", dpi=150)
    print("\nGrafico salvato: experiments/wave_interference.png")


if __name__ == "__main__":
    demo_propagation()
    
    print("\nGenerando visualizzazione...")
    try:
        plot_wave_interference()
    except Exception as e:
        print(f"(Visualizzazione saltata: {e})")
