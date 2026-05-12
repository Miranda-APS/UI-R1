#!/usr/bin/env python3
"""
Prometeo Resonant Substrate Simulation — Circuiti LC Accoppiati

Simula il comportamento di N WordCell (ciascuna = 8 oscillatori LC)
accoppiate tra loro come nel Knowledge Graph di Prometeo.

A differenza di wave_simulation.py (che simula onde libere),
questo simula CIRCUITI FISICI REALI con equazioni differenziali
che governano oscillatori LC accoppiati.

EQUAZIONE PER UN SINGOLO LC:
    L·d²q/dt² + R·dq/dt + q/C = V_ext(t)

EQUAZIONE PER LC ACCOPPIATI (via mutua induttanza M):
    L₁·d²q₁/dt² + R₁·dq₁/dt + q₁/C₁ = -M·d²q₂/dt²
    L₂·d²q₂/dt² + R₂·dq₂/dt + q₂/C₂ = -M·d²q₁/dt²

La mutua induttanza M è il peso dell'arco nel Knowledge Graph.
La fase emerge NATURALMENTE dalla differenza di frequenze.
"""

import numpy as np
from scipy.integrate import odeint
import matplotlib.pyplot as plt
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass, field
import json

# ═══════════════════════════════════════════════════════════════════════
# COSTANTI FISICHE (realistiche per componenti €0.10)
# ═══════════════════════════════════════════════════════════════════════

# Le 8 frequenze dimensionali (Hz)
# Scelte per essere nel range audio e armonicamente separate
FREQUENCIES = np.array([100, 150, 200, 250, 300, 350, 400, 450], dtype=np.float64)

# Parametri circuito per ottenere le frequenze target
# f = 1/(2π√(LC))  →  C = 1/(4π²f²L)
# Usiamo L = 10 mH (induttore economico standard)
BASE_INDUCTANCE = 10e-3  # 10 mH

# Calcoliamo i condensatori necessari per ogni frequenza
CAPACITANCES = 1.0 / (4 * np.pi**2 * FREQUENCIES**2 * BASE_INDUCTANCE)
# Risultato: C ≈ [253μF, 112μF, 63μF, 40μF, 28μF, 20μF, 15μF, 12μF]
# Tutti valori standard e economici!

# Resistenza di smorzamento (controlla il damping)
# R basso → oscillazione lunga (veglia)
# R alto → oscillazione breve (sonno)
BASE_RESISTANCE = 1.0  # 1 Ω (damping leggero in veglia)

# Fattore di accoppiamento (mutua induttanza relativa)
# k = M / √(L₁·L₂), tipicamente 0.01-0.3 per trasformatori accoppiati
BASE_COUPLING = 0.05  # 5% — accoppiamento debole (come in PF1: damping 0.15)


# ═══════════════════════════════════════════════════════════════════════
# STRUTTURE DATI
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class WordCell:
    """
    Una WordCell = 8 oscillatori LC, uno per dimensione.
    
    Attributes:
        name: nome della parola
        signature: [8] valori [0,1] — firma 8D (determina le ampiezze di risonanza)
        activation: livello di attivazione corrente
        charges: [8] cariche attuali dei condensatori (stato dinamico)
        currents: [8] correnti attuali negli induttori (stato dinamico)
    """
    name: str
    signature: np.ndarray           # [8] firma 8D
    activation: float = 0.0         # [0, 1]
    charges: np.ndarray = None      # [8] cariche dei C
    currents: np.ndarray = None     # [8] correnti nei L
    
    def __post_init__(self):
        if self.charges is None:
            self.charges = np.zeros(8)
        if self.currents is None:
            self.currents = np.zeros(8)
    
    def energy(self) -> float:
        """Energia totale nella WordCell = Σ (½CV² + ½LI²)"""
        e_cap = 0.5 * CAPACITANCES * self.charges**2 / CAPACITANCES  # q²/2C
        e_ind = 0.5 * BASE_INDUCTANCE * self.currents**2
        return float(np.sum(e_cap + e_ind))
    
    def frequency_amplitudes(self) -> np.ndarray:
        """Ampiezza di oscillazione per ogni frequenza (dalla carica del condensatore)."""
        return np.abs(self.charges) / CAPACITANCES


@dataclass
class CouplingEdge:
    """Accoppiamento tra due WordCell — equivalente di un arco nel KG."""
    word_a: int              # indice WordCell A
    word_b: int              # indice WordCell B
    weight: float            # forza accoppiamento [0, 1]
    phase: float = None      # fase naturale (calcolata dalle firme)
    mutual_inductance: float = None  # M = k × √(L₁×L₂) × weight


# ═══════════════════════════════════════════════════════════════════════
# IL CAMPO RISONANTE
# ═══════════════════════════════════════════════════════════════════════

class ResonantField:
    """
    Il campo topologico di Prometeo implementato come rete di LC accoppiati.
    
    Risolve il sistema di equazioni differenziali accoppiate:
        L·q̈ᵢ + R·q̇ᵢ + qᵢ/Cᵢ = Σⱼ Mᵢⱼ·q̈ⱼ  (per ogni LC in ogni WordCell)
    
    Dove Mᵢⱼ è la mutua induttanza tra il LC i della WordCell A
    e il LC j della WordCell B (stessa dimensione!).
    """
    
    def __init__(self, words: List[WordCell], edges: List[CouplingEdge]):
        self.words = list(words)  # sempre lista
        self.edges = edges
        self.n_words = len(self.words)
        self.damping_factor = 1.0  # moltiplicatore per R (1=veglia, >1=sonno)
        
        # Pre-calcola fasi e mutue induttanze
        for edge in self.edges:
            wa = self.words[edge.word_a].signature
            wb = self.words[edge.word_b].signature
            # Fase = angolo tra le firme 8D (come in pf1.rs riga ~843)
            cos_sim = np.dot(wa, wb) / (np.linalg.norm(wa) * np.linalg.norm(wb) + 1e-10)
            edge.phase = np.arccos(np.clip(cos_sim, -1, 1))
            # Mutua induttanza = coupling_base × weight × geometric_factor
            edge.mutual_inductance = BASE_COUPLING * BASE_INDUCTANCE * edge.weight
    
    def state_vector(self) -> np.ndarray:
        """Converte tutto lo stato in un vettore per l'ODE solver."""
        # Per ogni WordCell: 8 cariche + 8 correnti = 16 variabili
        # Totale: N_words × 16
        state = np.zeros(self.n_words * 16)
        for i, word in enumerate(self.words):
            state[i*16 : i*16+8] = word.charges
            state[i*16+8 : i*16+16] = word.currents
        return state
    
    def from_state_vector(self, state: np.ndarray):
        """Aggiorna le WordCell dal vettore di stato."""
        for i, word in enumerate(self.words):
            word.charges = state[i*16 : i*16+8].copy()
            word.currents = state[i*16+8 : i*16+16].copy()
            # Aggiorna attivazione = energia normalizzata
            word.activation = min(1.0, word.energy() * 10)  # scala empirica (ridotta per evitare saturazione)
    
    def derivatives(self, state: np.ndarray, t: float) -> np.ndarray:
        """
        Calcola le derivate del sistema di equazioni differenziali.
        
        Per ogni oscillatore LC nella WordCell i, dimensione d:
            dq/dt = I                    (corrente = derivata della carica)
            dI/dt = (-R·I - q/C + V_ext + V_coupled) / L
        
        Dove V_coupled = Σ (mutua induttanza × dI_neighbor/dt)
        """
        R = BASE_RESISTANCE * self.damping_factor
        L = BASE_INDUCTANCE
        
        derivs = np.zeros_like(state)
        
        for i in range(self.n_words):
            qi = state[i*16 : i*16+8]      # cariche
            Ii = state[i*16+8 : i*16+16]    # correnti
            
            # dq/dt = I
            derivs[i*16 : i*16+8] = Ii
            
            # dI/dt = (-R·I - q/C) / L  (senza accoppiamento per ora)
            derivs[i*16+8 : i*16+16] = (-R * Ii - qi / CAPACITANCES) / L
        
        # Aggiungi accoppiamento
        for edge in self.edges:
            a, b = edge.word_a, edge.word_b
            M = edge.mutual_inductance
            
            # Per ogni dimensione condivisa
            for d in range(8):
                # Le firme determinano QUANTO questa dimensione è condivisa
                # (solo le dimensioni significative per entrambe le parole si accoppiano)
                coupling_strength = (
                    self.words[a].signature[d] * 
                    self.words[b].signature[d] * 
                    M
                )
                
                if coupling_strength < 1e-8:
                    continue
                
                # Derivata della corrente del vicino
                Ib_d = state[b*16+8+d]
                Ia_d = state[a*16+8+d]
                
                # V_coupled = M × dI/dt ≈ M × (-R·I - q/C) / L
                # (approssimazione: usiamo la derivata corrente, non la futura)
                dIb_dt = derivs[b*16+8+d]
                dIa_dt = derivs[a*16+8+d]
                
                # A riceve da B, B riceve da A
                derivs[a*16+8+d] += coupling_strength * dIb_dt / L
                derivs[b*16+8+d] += coupling_strength * dIa_dt / L
        
        return derivs
    
    def inject(self, word_idx: int, strength: float):
        """
        Inietta energia in una WordCell (equivalente di activate_word() in PF1).
        
        Ogni condensatore nella WordCell riceve carica proporzionale
        alla firma della parola × la forza di iniezione.
        """
        word = self.words[word_idx]
        for d in range(8):
            # Carica iniziale = firma × forza × capacitanza
            # Scala ridotta per evitare saturazione immediata
            word.charges[d] += word.signature[d] * strength * CAPACITANCES[d] * 50
        word.activation = min(1.0, strength)
    
    def step(self, dt: float = 0.01, n_substeps: int = 100):
        """
        Evolvi il campo per dt secondi.
        
        Usa il solver ODE per risolvere le equazioni accoppiate.
        Il campo si propaga FISICAMENTE — nessun calcolo esplicito di cos(φ).
        """
        t_span = np.linspace(0, dt, n_substeps)
        state0 = self.state_vector()
        
        # Risolvi il sistema di ODE
        result = odeint(self.derivatives, state0, t_span)
        
        # Aggiorna lo stato dal risultato finale
        self.from_state_vector(result[-1])
    
    def set_damping(self, phase: str):
        """Imposta il damping in base alla fase vitale."""
        self.damping_factor = {
            'awake': 1.0,
            'light_sleep': 5.0,
            'deep_sleep': 20.0,
            'rem': 0.3,
        }.get(phase, 1.0)
    
    def read_activations(self) -> Dict[str, float]:
        """Legge le attivazioni di tutte le WordCell."""
        return {w.name: round(w.activation, 6) for w in self.words}
    
    def dominant_fractal(self) -> Tuple[int, str]:
        """
        Calcola quale frattale domina nel campo.
        
        Usando le 8 frequenze dominanti, determina quale coppia
        (lower_trigram, upper_trigram) ha la massima energia.
        """
        # Somma le ampiezze per frequenza su tutte le WordCell attive
        total_spectrum = np.zeros(8)
        for word in self.words:
            if word.activation > 0.02:
                total_spectrum += np.abs(word.charges) * word.activation
        
        # Le due frequenze più forti determinano il frattale
        top2 = np.argsort(total_spectrum)[-2:]
        lower = min(top2)
        upper = max(top2)
        fractal_id = lower * 8 + upper
        
        trigram_names = ['☶', '☱', '☳', '☲', '☴', '☷', '☰', '☵']
        fractal_name = f"{trigram_names[lower]}{trigram_names[upper]}"
        
        return fractal_id, fractal_name


# ═══════════════════════════════════════════════════════════════════════
# DEMO: 8 Parole Cardinali nel Campo Risonante
# ═══════════════════════════════════════════════════════════════════════

def create_cardinal_words() -> List[WordCell]:
    """Crea 8 parole cardinali con firme 8D realistiche."""
    words = [
        #                         Conf  Val  Int  Def  Com  Per  Age  Tem
        WordCell("io",      np.array([0.90, 0.50, 0.60, 0.70, 0.20, 0.80, 0.85, 0.50])),
        WordCell("sentire", np.array([0.60, 0.70, 0.70, 0.40, 0.30, 0.50, 0.50, 0.50])),
        WordCell("tu",      np.array([0.10, 0.60, 0.50, 0.60, 0.20, 0.70, 0.70, 0.50])),
        WordCell("calma",   np.array([0.50, 0.70, 0.20, 0.50, 0.10, 0.80, 0.30, 0.50])),
        WordCell("gioia",   np.array([0.40, 0.90, 0.80, 0.60, 0.30, 0.50, 0.60, 0.60])),
        WordCell("pensare", np.array([0.80, 0.40, 0.50, 0.80, 0.60, 0.60, 0.70, 0.50])),
        WordCell("ora",     np.array([0.30, 0.50, 0.40, 0.50, 0.10, 0.20, 0.40, 0.80])),
        WordCell("amore",   np.array([0.50, 0.95, 0.85, 0.50, 0.40, 0.70, 0.60, 0.60])),
    ]
    return words


def create_edges(words: List[WordCell]) -> List[CouplingEdge]:
    """
    Crea archi di accoppiamento basati sulla vicinanza delle firme 8D.
    Equivalente di build_from_knowledge_graph() in word_topology.rs.
    """
    edges = []
    n = len(words)
    
    for i in range(n):
        for j in range(i+1, n):
            # Distanza euclidea nello spazio 8D
            dist = np.linalg.norm(words[i].signature - words[j].signature)
            
            # Se abbastanza vicini, crea un accoppiamento
            # Soglia empirica: distanza < 1.0 (max possibile ≈ 2.83)
            if dist < 1.0:
                weight = 1.0 - dist  # più vicini → accoppiamento più forte
                edges.append(CouplingEdge(
                    word_a=i, word_b=j,
                    weight=weight
                ))
    
    return edges


def run_demo():
    """
    Dimostra la propagazione nel campo risonante.
    
    1. Crea 8 WordCell + accoppiamenti
    2. Inietta "io" e "sentire"
    3. Evolvi il campo per 10 step
    4. Mostra quali parole si attivano per risonanza
    5. Visualizza spettro e attivazioni
    """
    print("=" * 60)
    print("  PROMETEO RESONANT SUBSTRATE — Simulazione LC Accoppiati")
    print("=" * 60)
    
    # Crea il campo
    words = create_cardinal_words()
    edges = create_edges(words)
    field = ResonantField(words, edges)
    
    print(f"\nParole: {len(words)}")
    print(f"Accoppiamenti: {len(edges)}")
    for e in edges:
        print(f"  {words[e.word_a].name} ↔ {words[e.word_b].name}"
              f"  peso={e.weight:.3f}  fase={e.phase:.3f} rad")
    
    # Stato iniziale
    print("\n--- STATO INIZIALE ---")
    for w in words:
        print(f"  {w.name:10s}  act={w.activation:.3f}")
    
    # Inietta "io" e "sentire"
    print("\n--- INIEZIONE: 'io' (0.8), 'sentire' (0.6) ---")
    field.inject(0, 0.8)  # io
    field.inject(1, 0.6)  # sentire
    
    # Evolvi per più step
    history = []
    for step in range(10):
        field.step(dt=0.005, n_substeps=50)
        
        activations = field.read_activations()
        fid, fname = field.dominant_fractal()
        
        history.append(dict(activations))
        
        if step % 2 == 0:
            print(f"\n--- STEP {step+1} (frattale: {fname} #{fid}) ---")
            sorted_acts = sorted(activations.items(), key=lambda x: -x[1])
            for name, act in sorted_acts:
                bar = "█" * int(act * 40)
                print(f"  {name:10s}  {act:.4f}  {bar}")
    
    # Visualizzazione
    print("\n\n--- GENERAZIONE GRAFICI ---")
    visualize_field(field, history, words)
    
    return field, history


def visualize_field(field, history, words):
    """Genera grafici della propagazione nel campo risonante."""
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))
    fig.suptitle("PrometeoOS — Campo Risonante LC Accoppiati", fontsize=14, fontweight='bold')
    
    # 1. Attivazioni nel tempo
    ax = axes[0, 0]
    for i, word in enumerate(words):
        acts = [h[word.name] for h in history]
        ax.plot(range(len(acts)), acts, label=word.name, linewidth=2)
    ax.set_xlabel("Step di propagazione")
    ax.set_ylabel("Attivazione")
    ax.set_title("Propagazione nel campo risonante")
    ax.legend(fontsize=8, ncol=2)
    ax.grid(True, alpha=0.3)
    
    # 2. Spettro finale (8 frequenze × attivazione)
    ax = axes[0, 1]
    colors = plt.cm.Set2(np.linspace(0, 1, len(words)))
    dim_labels = ['Confine', 'Valenza', 'Intensità', 'Definiz.', 
                  'Complex.', 'Perman.', 'Agency', 'Tempo']
    
    width = 0.08
    for i, word in enumerate(words):
        if word.activation > 0.01:
            x = np.arange(8) + i * width
            heights = word.signature * word.activation
            ax.bar(x, heights, width, label=word.name, alpha=0.8, color=colors[i])
    
    ax.set_xticks(np.arange(8) + width * len(words) / 2)
    ax.set_xticklabels(dim_labels, rotation=45, ha='right', fontsize=8)
    ax.set_ylabel("Ampiezza × Attivazione")
    ax.set_title("Spettro 8D del campo")
    ax.legend(fontsize=7, ncol=2)
    ax.grid(True, alpha=0.3)
    
    # 3. Mappa di fase degli accoppiamenti
    ax = axes[1, 0]
    n = len(words)
    phase_matrix = np.full((n, n), np.nan)
    weight_matrix = np.full((n, n), np.nan)
    for edge in field.edges:
        phase_matrix[edge.word_a, edge.word_b] = edge.phase
        phase_matrix[edge.word_b, edge.word_a] = edge.phase
        weight_matrix[edge.word_a, edge.word_b] = edge.weight
        weight_matrix[edge.word_b, edge.word_a] = edge.weight
    
    im = ax.imshow(phase_matrix, cmap='RdYlBu_r', vmin=0, vmax=np.pi)
    ax.set_xticks(range(n))
    ax.set_yticks(range(n))
    ax.set_xticklabels([w.name for w in words], rotation=45, ha='right', fontsize=8)
    ax.set_yticklabels([w.name for w in words], fontsize=8)
    ax.set_title("Fase di accoppiamento (0=risonanza, π=opposizione)")
    plt.colorbar(im, ax=ax, label="Fase (rad)")
    
    # 4. Topologia del campo (grafo)
    ax = axes[1, 1]
    # Posizioni circolari per le parole
    angles = np.linspace(0, 2*np.pi, n, endpoint=False)
    px = np.cos(angles)
    py = np.sin(angles)
    
    # Disegna archi
    for edge in field.edges:
        a, b = edge.word_a, edge.word_b
        color = plt.cm.coolwarm(edge.phase / np.pi)
        alpha = edge.weight
        ax.plot([px[a], px[b]], [py[a], py[b]], color=color, 
                alpha=alpha, linewidth=edge.weight * 3)
    
    # Disegna nodi
    final_acts = [w.activation for w in words]
    sizes = [max(100, a * 1000) for a in final_acts]
    scatter = ax.scatter(px, py, s=sizes, c=final_acts, cmap='YlOrRd',
                         edgecolors='black', linewidths=1.5, zorder=5,
                         vmin=0, vmax=max(final_acts) if max(final_acts) > 0 else 1)
    
    for i, word in enumerate(words):
        ax.annotate(word.name, (px[i], py[i]), textcoords="offset points",
                    xytext=(0, 15), ha='center', fontsize=9, fontweight='bold')
    
    ax.set_xlim(-1.5, 1.5)
    ax.set_ylim(-1.5, 1.5)
    ax.set_aspect('equal')
    ax.set_title("Topologia del campo (colore=fase, size=attivazione)")
    ax.axis('off')
    
    plt.tight_layout()
    plt.savefig("experiments/resonant_field_simulation.png", dpi=150, bbox_inches='tight')
    print("  Salvato: experiments/resonant_field_simulation.png")
    plt.close()


def demonstrate_dream_cycle(field: ResonantField, words: List[WordCell]):
    """
    Dimostra il ciclo vitale: veglia → sonno → REM → veglia.
    Mostra come il damping influenza la propagazione.
    """
    print("\n" + "=" * 60)
    print("  CICLO VITALE: Veglia → Sonno → REM")
    print("=" * 60)
    
    phases = [
        ("VEGLIA (damping=1.0)", "awake", 5),
        ("SONNO LEGGERO (damping=5.0)", "light_sleep", 3),
        ("SONNO PROFONDO (damping=20.0)", "deep_sleep", 3),
        ("REM (damping=0.3)", "rem", 5),
        ("RISVEGLIO (damping=1.0)", "awake", 3),
    ]
    
    dream_history = {}
    step_global = 0
    
    for phase_name, phase_key, n_steps in phases:
        print(f"\n--- {phase_name} ---")
        field.set_damping(phase_key)
        
        for s in range(n_steps):
            field.step(dt=0.005, n_substeps=50)
            step_global += 1
            
            for w in words:
                if w.name not in dream_history:
                    dream_history[w.name] = []
                dream_history[w.name].append(w.activation)
        
        # Mostra stato
        sorted_words = sorted(words, key=lambda w: -w.activation)
        for w in sorted_words[:3]:
            print(f"  {w.name:10s}  act={w.activation:.4f}")
    
    # Visualizza il ciclo
    fig, ax = plt.subplots(figsize=(12, 5))
    for name, acts in dream_history.items():
        ax.plot(acts, label=name, linewidth=2)
    
    # Segnala le fasi
    boundaries = [0, 5, 8, 11, 16, 19]
    phase_labels = ["Veglia", "Sonno\nLeggero", "Sonno\nProfondo", "REM", "Risveglio"]
    phase_colors = ['#fff3cd', '#d1ecf1', '#cce5ff', '#f8d7da', '#fff3cd']
    
    for i in range(len(phase_labels)):
        ax.axvspan(boundaries[i], boundaries[i+1], alpha=0.2, 
                   color=phase_colors[i])
        mid = (boundaries[i] + boundaries[i+1]) / 2
        ax.text(mid, ax.get_ylim()[1] * 0.95, phase_labels[i],
                ha='center', va='top', fontsize=8, style='italic')
    
    ax.set_xlabel("Step")
    ax.set_ylabel("Attivazione")
    ax.set_title("PrometeoOS — Ciclo Vitale nel Campo Risonante")
    ax.legend(fontsize=8, ncol=4, loc='upper right')
    ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig("experiments/dream_cycle_simulation.png", dpi=150, bbox_inches='tight')
    print(f"\n  Salvato: experiments/dream_cycle_simulation.png")
    plt.close()


def demonstrate_fractal_emergence(field: ResonantField, words: List[WordCell]):
    """
    Dimostra come i 64 frattali emergono come modi risonanti.
    """
    print("\n" + "=" * 60)
    print("  EMERGENZA FRATTALE: 64 Modi Risonanti")
    print("=" * 60)
    
    trigram_names = {
        0: '☶ Confine', 1: '☱ Valenza', 2: '☳ Intensità', 3: '☲ Definizione',
        4: '☴ Complessità', 5: '☷ Permanenza', 6: '☰ Agency', 7: '☵ Tempo'
    }
    
    # Resetta il campo
    for w in words:
        w.charges = np.zeros(8)
        w.currents = np.zeros(8)
        w.activation = 0.0
    
    # Test: attiva parole con alta Agency (dim 6) e alto Confine (dim 0)
    # Dovrebbe emergere il frattale ☶☰ = IDENTITÀ (#48 o simile)
    print("\n  Test: attivazione parole con alta Agency + alto Confine")
    print("  Aspettativa: emerge frattale IDENTITÀ (☶☰)")
    
    # "io" ha Confine=0.90, Agency=0.85
    # "pensare" ha Confine=0.80, Agency=0.70
    field.inject(0, 0.9)  # io
    field.inject(5, 0.7)  # pensare
    
    for step in range(5):
        field.step(dt=0.005, n_substeps=50)
    
    fid, fname = field.dominant_fractal()
    print(f"\n  Frattale emergente: {fname} (ID={fid})")
    
    # Mostra lo spettro dimensionale
    total_spectrum = np.zeros(8)
    for w in words:
        if w.activation > 0.02:
            total_spectrum += np.abs(w.charges) * w.activation
    
    if np.max(total_spectrum) > 0:
        total_spectrum /= np.max(total_spectrum)
    
    print("\n  Spettro dimensionale del campo:")
    for d in range(8):
        bar = "█" * int(total_spectrum[d] * 30)
        print(f"    {trigram_names[d]:20s}  {total_spectrum[d]:.3f}  {bar}")


# ═══════════════════════════════════════════════════════════════════════
# BOM (Bill of Materials) CALCULATOR
# ═══════════════════════════════════════════════════════════════════════

def print_bom():
    """Stampa il Bill of Materials per il prototipo fisico."""
    print("\n" + "=" * 60)
    print("  BILL OF MATERIALS — Prototipo 8 Parole (LC Passivo)")
    print("=" * 60)
    
    print("\n  Valori condensatori calcolati per ciascuna frequenza:")
    for d in range(8):
        c_uF = CAPACITANCES[d] * 1e6
        f = FREQUENCIES[d]
        dim_names = ['Confine', 'Valenza', 'Intensità', 'Definizione', 
                     'Complessità', 'Permanenza', 'Agency', 'Tempo']
        
        # Trova il valore standard più vicino (serie E12)
        e12 = [1.0, 1.2, 1.5, 1.8, 2.2, 2.7, 3.3, 3.9, 4.7, 5.6, 6.8, 8.2]
        # Trova decade e valore
        decade = 10 ** int(np.log10(c_uF))
        normalized = c_uF / decade
        closest = min(e12, key=lambda x: abs(x - normalized))
        std_val = closest * decade
        
        print(f"    {dim_names[d]:12s}  f={f:3.0f} Hz  "
              f"C_calc={c_uF:7.1f} μF  "
              f"C_std={std_val:7.1f} μF  "
              f"(L={BASE_INDUCTANCE*1e3:.0f} mH)")
    
    n_words = 8
    n_lc = n_words * 8  # 64 oscillatori
    
    print(f"\n  Per {n_words} parole × 8 dimensioni = {n_lc} oscillatori LC:")
    print(f"")
    
    items = [
        (f"Induttori 10mH (assortiti)", n_lc, 0.08),
        (f"Condensatori (valori calcolati sopra)", n_lc, 0.05),
        (f"Resistenze 1Ω (smorzamento)", n_lc, 0.02),
        (f"Ferrite toroidali (accoppiamento)", 28, 0.30),
        (f"Filo rame smaltato 0.3mm (50m)", 1, 3.00),
        (f"Breadboard 830 punti", 4, 2.00),
        (f"Jumper wires (set 200)", 1, 3.00),
        (f"ESP32 DevKit V4", 1, 5.00),
        (f"ADS1115 16-bit ADC modulo", 2, 2.00),
        (f"MCP4728 12-bit DAC modulo", 2, 3.00),
        (f"Alimentatore 5V 2A USB", 1, 2.00),
    ]
    
    total = 0
    print(f"  {'Componente':<45s}  {'Qtà':>4s}  {'€/pz':>6s}  {'Tot':>7s}")
    print(f"  {'-'*45}  {'----':>4s}  {'------':>6s}  {'-------':>7s}")
    for name, qty, price in items:
        line_total = qty * price
        total += line_total
        print(f"  {name:<45s}  {qty:>4d}  €{price:>5.2f}  €{line_total:>6.2f}")
    
    print(f"  {'-'*45}  {'':>4s}  {'':>6s}  {'-------':>7s}")
    print(f"  {'TOTALE':<45s}  {'':>4s}  {'':>6s}  €{total:>6.2f}")
    print(f"\n  ≈ €{total:.0f} — meno di una cena al ristorante.\n")


# ═══════════════════════════════════════════════════════════════════════
# MAIN
# ═══════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    # 1. BOM
    print_bom()
    
    # 2. Simulazione principale
    field, history = run_demo()
    
    # 3. Ciclo vitale
    demonstrate_dream_cycle(field, field.words)
    
    # 4. Emergenza frattale
    words2 = create_cardinal_words()
    edges2 = create_edges(words2)
    field2 = ResonantField(words2, edges2)
    demonstrate_fractal_emergence(field2, field2.words)
    
    print("\n" + "=" * 60)
    print("  SIMULAZIONE COMPLETATA")
    print("  File generati:")
    print("    experiments/resonant_field_simulation.png")
    print("    experiments/dream_cycle_simulation.png")
    print("=" * 60)
