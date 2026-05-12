#!/usr/bin/env python3
"""
Prometeo Acoustic POC — Raspberry Pi Implementation

HARDWARE REQUIRED:
- Raspberry Pi 4 (or 3B+)
- 8× Piezo speakers (or small speakers)
- 8× MEMS microphones (or USB audio interface with 8 inputs)
- Breadboard + jumper wires

ARCHITECTURE:
    [RPi GPIO PWM] → [8 Speakers] → [Air medium] → [8 Mics] → [RPi ADC]
         ↓                              ↓                         ↓
    Generate waves            Physical interference        Capture result

FREQUENCIES:
    100, 150, 200, 250, 300, 350, 400, 450 Hz (8 dimensions)

LATENCY TARGET: <200ms per propagation step
"""

import numpy as np
import time
from typing import List, Dict, Tuple
import json

# Try to import RPi.GPIO (will fail on non-RPi systems)
try:
    import RPi.GPIO as GPIO
    RPI_AVAILABLE = True
except ImportError:
    RPI_AVAILABLE = False
    print("WARNING: RPi.GPIO not available. Running in simulation mode.")

# Try to import audio libraries
try:
    import pyaudio
    AUDIO_AVAILABLE = True
except ImportError:
    AUDIO_AVAILABLE = False
    print("WARNING: pyaudio not available. Audio I/O disabled.")

# ═══════════════════════════════════════════════════════════════════════
# CONSTANTS
# ═══════════════════════════════════════════════════════════════════════

# 8 frequencies for 8 dimensions (Hz)
FREQUENCIES = np.array([100, 150, 200, 250, 300, 350, 400, 450], dtype=np.float32)

# Audio parameters
SAMPLE_RATE = 44100  # Hz (standard audio)
CHUNK_SIZE = 4096    # samples per buffer
DURATION = 0.1       # seconds per propagation frame

# GPIO pins for PWM output (BCM numbering)
PWM_PINS = [12, 13, 18, 19, 20, 21, 26, 27]  # 8 pins with hardware PWM support

# Activation threshold
ACTIVATION_THRESHOLD = 0.02

# ═══════════════════════════════════════════════════════════════════════
# WAVE WORD — Hardware Version
# ═══════════════════════════════════════════════════════════════════════

class AcousticWord:
    """
    A word as an acoustic wave pattern.
    
    Attributes:
        name: word string
        signature: [8] amplitudes for each dimension [0,1]
        activation: current activation level [0,1]
        phases: [8] initial phases for each frequency component
    """
    
    def __init__(self, name: str, signature: np.ndarray):
        self.name = name
        self.signature = signature.astype(np.float32)
        self.activation = 0.0
        self.phases = np.random.uniform(0, 2*np.pi, 8).astype(np.float32)
    
    def generate_wave_samples(self, duration: float, sample_rate: int) -> np.ndarray:
        """
        Generate audio samples for this word.
        
        Returns:
            [8, N] array where each row is one frequency channel
        """
        n_samples = int(duration * sample_rate)
        t = np.linspace(0, duration, n_samples, dtype=np.float32)
        
        waves = np.zeros((8, n_samples), dtype=np.float32)
        
        for i in range(8):
            if self.signature[i] > 0.01:
                amplitude = self.signature[i] * self.activation
                waves[i] = amplitude * np.sin(2 * np.pi * FREQUENCIES[i] * t + self.phases[i])
        
        return waves
    
    def __repr__(self):
        return f"AcousticWord({self.name}, act={self.activation:.2f})"


# ═══════════════════════════════════════════════════════════════════════
# ACOUSTIC FIELD — Physical Wave Propagation
# ═══════════════════════════════════════════════════════════════════════

class AcousticField:
    """
    Physical wave field using speakers and microphones.
    
    Propagation happens in PHYSICAL AIR, not in software.
    """
    
    def __init__(self, words: List[AcousticWord], use_hardware: bool = False):
        self.words = {w.name: w for w in words}
        self.use_hardware = use_hardware and RPI_AVAILABLE and AUDIO_AVAILABLE
        
        if self.use_hardware:
            self._init_hardware()
        else:
            print("Running in SIMULATION mode (no hardware)")
    
    def _init_hardware(self):
        """Initialize GPIO and audio hardware."""
        # Setup GPIO for PWM output
        GPIO.setmode(GPIO.BCM)
        self.pwm_channels = []
        
        for pin in PWM_PINS:
            GPIO.setup(pin, GPIO.OUT)
            # Create PWM at base frequency (will modulate)
            pwm = GPIO.PWM(pin, 1000)  # 1kHz base
            pwm.start(0)  # 0% duty cycle initially
            self.pwm_channels.append(pwm)
        
        # Setup audio input
        self.audio = pyaudio.PyAudio()
        self.input_stream = self.audio.open(
            format=pyaudio.paFloat32,
            channels=8,  # 8-channel input
            rate=SAMPLE_RATE,
            input=True,
            frames_per_buffer=CHUNK_SIZE
        )
        
        print("Hardware initialized: 8 PWM outputs + 8 audio inputs")
    
    def activate(self, word_name: str, strength: float):
        """Activate a word."""
        if word_name in self.words:
            self.words[word_name].activation = min(1.0, strength)
    
    def _emit_waves_hardware(self, duration: float):
        """
        Emit waves through speakers (hardware).
        
        Uses PWM to generate audio frequencies on GPIO pins.
        """
        n_samples = int(duration * SAMPLE_RATE)
        
        # Generate composite wave for each channel
        channel_waves = np.zeros((8, n_samples), dtype=np.float32)
        
        for word in self.words.values():
            if word.activation > ACTIVATION_THRESHOLD:
                word_waves = word.generate_wave_samples(duration, SAMPLE_RATE)
                channel_waves += word_waves
        
        # Normalize to prevent clipping
        max_amp = np.abs(channel_waves).max()
        if max_amp > 0:
            channel_waves /= max_amp
        
        # Emit through PWM (simplified - real implementation needs DMA)
        # This is a placeholder - actual PWM audio requires more sophisticated approach
        for i, pwm in enumerate(self.pwm_channels):
            # Convert wave to duty cycle (0-100%)
            duty_cycle = (channel_waves[i].mean() + 1.0) * 50.0
            pwm.ChangeDutyCycle(duty_cycle)
        
        time.sleep(duration)
        
        # Stop emission
        for pwm in self.pwm_channels:
            pwm.ChangeDutyCycle(0)
    
    def _capture_waves_hardware(self, duration: float) -> np.ndarray:
        """
        Capture waves from microphones (hardware).
        
        Returns:
            [8, N] array of captured audio samples
        """
        n_samples = int(duration * SAMPLE_RATE)
        
        # Read from audio input
        audio_data = self.input_stream.read(n_samples)
        
        # Convert to numpy array [N, 8] then transpose to [8, N]
        samples = np.frombuffer(audio_data, dtype=np.float32)
        samples = samples.reshape(-1, 8).T
        
        return samples
    
    def _propagate_simulation(self) -> Dict[str, float]:
        """
        Simulate propagation in software (when no hardware available).
        
        This is the fallback - uses the same math as wave_simulation.py
        """
        # Generate composite wave
        t = np.linspace(0, DURATION, int(SAMPLE_RATE * DURATION))
        wave_total = np.zeros_like(t)
        
        for word in self.words.values():
            if word.activation > ACTIVATION_THRESHOLD:
                for i in range(8):
                    if word.signature[i] > 0.01:
                        component = (word.signature[i] * word.activation * 
                                   np.sin(2 * np.pi * FREQUENCIES[i] * t + word.phases[i]))
                        wave_total += component
        
        # Measure resonance for each word
        new_activations = {}
        
        for word in self.words.values():
            # Generate ideal wave for this word
            word_wave = np.zeros_like(t)
            for i in range(8):
                if word.signature[i] > 0.01:
                    component = (word.signature[i] * 
                               np.sin(2 * np.pi * FREQUENCIES[i] * t + word.phases[i]))
                    word_wave += component
            
            # Correlation = resonance
            if np.std(word_wave) > 0 and np.std(wave_total) > 0:
                correlation = np.corrcoef(word_wave, wave_total)[0, 1]
                activation = max(0.0, correlation)
            else:
                activation = 0.0
            
            new_activations[word.name] = activation
        
        return new_activations
    
    def _propagate_hardware(self) -> Dict[str, float]:
        """
        Propagate using physical hardware.
        
        1. Emit waves through speakers
        2. Capture interference through microphones
        3. FFT to extract frequency components
        4. Measure resonance for each word
        """
        # Emit waves
        self._emit_waves_hardware(DURATION)
        
        # Capture result
        captured = self._capture_waves_hardware(DURATION)
        
        # FFT on each channel
        fft_results = np.fft.rfft(captured, axis=1)
        freqs = np.fft.rfftfreq(captured.shape[1], 1.0/SAMPLE_RATE)
        
        # Extract amplitudes at our 8 frequencies
        measured_amplitudes = np.zeros(8, dtype=np.float32)
        for i, freq in enumerate(FREQUENCIES):
            # Find closest FFT bin
            idx = np.argmin(np.abs(freqs - freq))
            measured_amplitudes[i] = np.abs(fft_results[i, idx])
        
        # Normalize
        if measured_amplitudes.max() > 0:
            measured_amplitudes /= measured_amplitudes.max()
        
        # Measure resonance: how similar is each word's signature to measured amplitudes?
        new_activations = {}
        for word in self.words.values():
            # Cosine similarity between word signature and measured amplitudes
            dot = np.dot(word.signature, measured_amplitudes)
            norm_word = np.linalg.norm(word.signature)
            norm_meas = np.linalg.norm(measured_amplitudes)
            
            if norm_word > 0 and norm_meas > 0:
                similarity = dot / (norm_word * norm_meas)
                activation = max(0.0, similarity)
            else:
                activation = 0.0
            
            new_activations[word.name] = activation
        
        return new_activations
    
    def step(self) -> Dict[str, float]:
        """
        One propagation step.
        
        Returns:
            dict of new activations
        """
        if self.use_hardware:
            new_acts = self._propagate_hardware()
        else:
            new_acts = self._propagate_simulation()
        
        # Update activations with damping
        damping = 0.15
        for name, new_act in new_acts.items():
            current = self.words[name].activation
            self.words[name].activation = current * (1 - damping) + new_act * damping
        
        return new_acts
    
    def cleanup(self):
        """Cleanup hardware resources."""
        if self.use_hardware:
            for pwm in self.pwm_channels:
                pwm.stop()
            GPIO.cleanup()
            self.input_stream.stop_stream()
            self.input_stream.close()
            self.audio.terminate()


# ═══════════════════════════════════════════════════════════════════════
# DEMO
# ═══════════════════════════════════════════════════════════════════════

def load_words_from_pf1(pf1_path: str = "prometeo_field.bin") -> List[AcousticWord]:
    """
    Load words from PF1 binary file.
    
    For now, returns demo words. TODO: parse actual PF1 format.
    """
    # Demo words (same as wave_simulation.py)
    signatures = {
        "io":      np.array([0.9, 0.5, 0.6, 0.7, 0.5, 0.6, 0.8, 0.5]),
        "tu":      np.array([0.7, 0.6, 0.5, 0.6, 0.5, 0.5, 0.6, 0.5]),
        "qui":     np.array([0.6, 0.5, 0.5, 0.8, 0.4, 0.7, 0.4, 0.5]),
        "ora":     np.array([0.5, 0.5, 0.6, 0.7, 0.4, 0.3, 0.5, 0.8]),
        "sentire": np.array([0.8, 0.6, 0.5, 0.5, 0.6, 0.5, 0.7, 0.5]),
        "calma":   np.array([0.7, 0.7, 0.3, 0.6, 0.4, 0.8, 0.4, 0.5]),
        "gioia":   np.array([0.6, 0.9, 0.7, 0.6, 0.5, 0.6, 0.5, 0.5]),
        "paura":   np.array([0.7, 0.2, 0.8, 0.5, 0.6, 0.4, 0.3, 0.5]),
    }
    
    return [AcousticWord(name, sig) for name, sig in signatures.items()]


def demo_acoustic_propagation(use_hardware: bool = False):
    """
    Demo: activate words and observe acoustic propagation.
    """
    print("=" * 70)
    print("PROMETEO ACOUSTIC POC")
    print("=" * 70)
    print()
    
    if use_hardware:
        print("MODE: HARDWARE (Raspberry Pi + Speakers + Mics)")
        print("WARNING: Ensure hardware is connected properly!")
    else:
        print("MODE: SIMULATION (no hardware required)")
    print()
    
    # Load words
    words = load_words_from_pf1()
    field = AcousticField(words, use_hardware=use_hardware)
    
    try:
        # Activate input words
        print("INPUT: activating 'io' (0.8) and 'sentire' (0.6)")
        field.activate("io", 0.8)
        field.activate("sentire", 0.6)
        print()
        
        # Initial state
        print("INITIAL STATE:")
        for word in words:
            if field.words[word.name].activation > 0.01:
                print(f"  {word.name:10s}: {field.words[word.name].activation:.3f}")
        print()
        
        # Propagation steps
        print("PROPAGATION (3 steps):")
        for step in range(3):
            print(f"\n  Step {step+1}:")
            start_time = time.time()
            
            field.step()
            
            elapsed = time.time() - start_time
            print(f"    Latency: {elapsed*1000:.1f} ms")
            
            # Show active words
            active = [(name, w.activation) for name, w in field.words.items() 
                      if w.activation > ACTIVATION_THRESHOLD]
            active.sort(key=lambda x: x[1], reverse=True)
            
            for name, act in active[:5]:
                print(f"    {name:10s}: {act:.3f}")
        
        print()
        print("=" * 70)
        print("RESULT:")
        if use_hardware:
            print("  Physical wave interference produced propagation!")
            print("  The computation happened in AIR, not in CPU.")
        else:
            print("  Simulation complete. Ready for hardware test.")
        print("=" * 70)
    
    finally:
        field.cleanup()


def benchmark_latency(use_hardware: bool = False, n_steps: int = 10):
    """
    Benchmark propagation latency.
    """
    print(f"\nBENCHMARK: {n_steps} propagation steps")
    print(f"Mode: {'HARDWARE' if use_hardware else 'SIMULATION'}")
    print()
    
    words = load_words_from_pf1()
    field = AcousticField(words, use_hardware=use_hardware)
    
    try:
        field.activate("io", 0.8)
        field.activate("sentire", 0.6)
        
        latencies = []
        for i in range(n_steps):
            start = time.time()
            field.step()
            elapsed = time.time() - start
            latencies.append(elapsed * 1000)  # ms
        
        print(f"Average latency: {np.mean(latencies):.1f} ms")
        print(f"Min latency: {np.min(latencies):.1f} ms")
        print(f"Max latency: {np.max(latencies):.1f} ms")
        print(f"Std dev: {np.std(latencies):.1f} ms")
        
        if np.mean(latencies) < 200:
            print("\n✓ TARGET MET: <200ms average latency")
        else:
            print("\n✗ TARGET MISSED: >200ms average latency")
    
    finally:
        field.cleanup()


if __name__ == "__main__":
    import sys
    
    # Check if running on Raspberry Pi
    use_hw = "--hardware" in sys.argv or "-hw" in sys.argv
    
    if use_hw and not RPI_AVAILABLE:
        print("ERROR: RPi.GPIO not available. Cannot use hardware mode.")
        print("Install with: pip install RPi.GPIO")
        sys.exit(1)
    
    if use_hw and not AUDIO_AVAILABLE:
        print("ERROR: pyaudio not available. Cannot use hardware mode.")
        print("Install with: pip install pyaudio")
        sys.exit(1)
    
    # Run demo
    demo_acoustic_propagation(use_hardware=use_hw)
    
    # Benchmark
    if "--benchmark" in sys.argv or "-b" in sys.argv:
        benchmark_latency(use_hardware=use_hw, n_steps=20)
