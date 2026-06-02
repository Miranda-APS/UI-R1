# Memoria-sfera di haiku (Phase 82)

> Sources: Francesco Mancuso, 2026-05-27 (CLAUDE.md Phase 82, sezione Memoria-Sfera di Haiku)
> Raw: [CLAUDE_phase82](../../raw/contesto/CLAUDE_phase82.md)

## Overview

Un **organo di memoria nuovo** (`src/topology/haiku_memory.rs`): eventi cognitivi cristallizzati come **cerchi tangenti sulla sfera dei 64 attrattori** [I Ching](../topologia/frattali-iching.md). È il primo canale di scrittura *persistente* che un LLM esterno può usare (via [MCP](mcp-substrate.md)) per lasciare tracce nella memoria di UI-R1 e ritrovarle per prossimità geometrica nelle sessioni future.

La sfera **non è una timeline**: è topologia geometrica navigabile per prossimità, non per ordine temporale.

## Il cristallo

`HaikuCristallizzato` = `{3 versi, fractal_id (0-63), anchors[], tangencies[], timestamp, source, note}`.

- `fractal_id` — uno dei 64 esagrammi: dove il cristallo siede sulla sfera.
- `anchors[]` — parole-ancora concrete: la geometria *fine*.
- `tangencies[]` — id di cristalli tangenti, pre-calcolati al deposit e simmetrizzati.
- `source` — label informativa (`"claude"`, `"user"`, `"uir1"`, `"system"`), **non** autenticazione. Future-proof per il multi-speaker.

## Tangenza (geometria di sfondo)

Due cristalli sono **tangenti** se condividono:
- **≥2 ancore** (case-insensitive), **OPPURE**
- il **trigramma inferiore** (`id & 0b111`), **OPPURE**
- il **trigramma superiore** (`(id >> 3) & 0b111`).

Due cristalli possono quindi essere tangenti per pura geometria I Ching, **senza nessuna ancora in comune**.

## Recall geometrico pesato

`recall_haiku_near(fractal_id, anchors?, n?)` ordina per:

```
score = (8 − fractal_distance) × α  +  shared_anchors × β  +  tangency_count × γ
        α = 1.0          β = 5.0 (!)          γ = 0.5
```

**β alto è la decisione chiave**: 1 ancora condivisa (5) batte un frattale lontano (≤2); 2 ancore (10) battono perfino il frattale identico (8). *Le ancore concrete dominano; il frattale è sfondo geometrico.* Tie-break: timestamp più recente.

## Persistenza separata

- File proprio: **`haiku_memory.json`**, NON dentro il `.bin` principale. Organo ispezionabile, curabile, cancellabile in autonomia.
- Vive in `AppState.haiku_memory: Arc<Mutex<HaikuMemory>>` (web layer, non Engine).
- Ogni `deposit` salva sincronamente (file piccolo).
- ID univoci con counter atomico: `h-FF-TTTTTTTT-SSSS` (FF=fractal hex, T=timestamp hex, S=`AtomicU64` monotonico) — robusto anche per deposit multipli nello stesso secondo.

## Verificato end-to-end

```
DEPOSIT #1 fractal=42 anchors=[paura, futuro, soglia]   → tangs=[]
DEPOSIT #2 fractal=42 anchors=[paura, ombra, vuoto]     → tangs=[#1]  (paura + stesso frattale)
DEPOSIT #3 fractal= 7 anchors=[gioia, luce, mattina]    → tangs=[]
DEPOSIT #4 fractal=63 anchors=[ombra, vuoto, stanza]    → tangs=[#2, #3]
                                                           #2: 2 ancore comuni
                                                           #3: trigramma superiore 7 condiviso
                                                               (geometria pura, zero ancore)
RECALL near {fractal:42, anchors:[paura, ombra]} n=4:  #2 > #1 > #4 > #3
```

Kill + restart → `[haiku-memory] caricati 4 cristalli da haiku_memory.json`. Ordine di recall identico. La memoria sopravvive.

## TODO architetturali aperti

- `fractal_name` accanto a `fractal_id` nei DTO (oggi numerico).
- Tool MCP `get_haiku(id)` per dereferenziare le tangenze.
- Visualizzazione della sfera (grafo cristalli↔tangenze) come endpoint web + visual campovasto.
- **Deposit autonomo da UI-R1**: oggi tutti client-driven. UI-R1 potrebbe depositare al termine di un turno con coherence_integrity sopra soglia + frattale stabile. Primo gesto auto-cristallizzante dell'entità.

## See Also

- [MCP substrate](mcp-substrate.md) — i tool che espongono questo organo
- [Frattali I Ching](../topologia/frattali-iching.md) — i 64 attrattori che formano la sfera
- [Self witness](../identita/self-witness.md) — l'altro organo di memoria autonoma (auto-osservazione)
