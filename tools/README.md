# Prometeo Semantic Agent

Agente autonomo basato su **Qwen2.5** (via Ollama) per esplorare, analizzare e migliorare il campo topologico di Prometeo.

> **Nota**: Configurato per Qwen2.5:7b-instruct (già disponibile in locale). 
> Quando Qwen3 sarà rilasciato su Ollama, basterà cambiare `model="qwen3:9b"` in `agent/loop.py`.

## Architettura

```
┌─────────────┐     HTTP      ┌─────────────┐     API       ┌─────────────────┐
│  Ollama     │◄──────────────►│ Agent Loop  │◄─────────────►│ PrometeoBridge  │
│  Qwen3:9b   │                │  (Python)   │               │   (KG/Topology) │
└─────────────┘                └──────┬──────┘               └─────────────────┘
                                      │
                                      ▼
                               ┌─────────────┐
                               │ ToolRegistry│
                               │  - analyze  │
                               │  - query    │
                               │  - propose  │
                               │  - commit   │
                               └─────────────┘
```

## Quick Start

### 1. Prerequisiti

```bash
# Installa Ollama
# Windows: https://ollama.com/download/windows

# Scarica Qwen3 9B
ollama pull qwen3:9b

# Verifica
ollama run qwen3:9b
```

### 2. Setup Python

```bash
cd tools
pip install -r requirements.txt
```

### 3. Avvia l'Agente

```bash
cd tools

# Esplorazione libera
python agent/loop.py

# Task specifico (analisi e riparazione)
python agent/loop.py --task steering/tasks/analyze_and_repair.json -i 30

# Task creativo (nuovi concetti)
python agent/loop.py --task steering/tasks/create_concepts.json -i 20

# Simulazione (non salva)
python agent/loop.py --dry-run -i 10
```

## Struttura

```
tools/
├── steering/
│   ├── system_prompt.md          # Personalità dell'agente
│   ├── schemas/
│   │   └── tool_schemas.json     # Definizioni tool per Qwen
│   └── tasks/
│       ├── analyze_and_repair.json
│       └── create_concepts.json
├── agent/
│   ├── loop.py                   # Loop supervisor
│   ├── ollama_client.py          # Client Ollama
│   ├── prometeo_bridge.py        # Bridge KG/Topology
│   ├── tools.py                  # Implementazione tool
│   └── logger.py                 # Logging opinioni
└── logs/                         # Output sessioni
    ├── opinions_YYYYMMDD_HHMMSS.jsonl
    ├── session_YYYYMMDD_HHMMSS.json
    └── agent_YYYYMMDD_HHMMSS.log
```

## Tool Disponibili

| Tool | Descrizione |
|------|-------------|
| `analyze_field` | Analizza stato campo (nodi isolati, cluster, gap) |
| `query_concept` | Interroga un concetto nel KG |
| `propose_connection` | Propone nuova relazione semantica |
| `check_fractal_consistency` | Verifica coerenza con i 64 frattali |
| `find_analogies` | Trova pattern analogici (A:B::C:?) |
| `log_opinion` | Registra osservazione/insight |
| `commit_changes` | Applica batch modifiche |
| `pause_loop` | Pausa controllata |

## Configurazione

### Token Limiting
Modifica in `loop.py`:
```python
LoopConfig(
    token_budget_per_turn=2048,  # Max token per risposta
    max_iterations=50,           # Max cicli
)
```

### Temperature
```bash
# Più creativo
python agent/loop.py --temp 0.7

# Più conservativo (default)
python agent/loop.py --temp 0.3
```

## Ciclo di Lavoro

1. **Exploration** → `analyze_field` mappa lo stato
2. **Reflection** → `log_opinion` registra insights
3. **Deep Dive** → `query_concept` esplora nodi critici
4. **Proposal** → `propose_connection` suggerisce archi
5. **Validation** → `check_fractal_consistency` verifica
6. **Commit** → `commit_changes` applica (dry_run prima)

## Log e Opinioni

Ogni sessione genera:

- **opinions_*.jsonl**: Una riga per opinione con timestamp, categoria, contenuto
- **session_*.json**: Statistiche aggregate
- **agent_*.log**: Log completo con timestamp

### Analisi Opinioni

```python
import json

# Leggi opinioni
with open('logs/opinions_20260307_111430.jsonl') as f:
    opinions = [json.loads(line) for line in f]

# Filtra insights
insights = [o for o in opinions if o['category'] == 'insight']

# Concetti più discussi
from collections import Counter
concepts = []
for o in opinions:
    concepts.extend(o.get('related_concepts', []))
print(Counter(concepts).most_common(10))
```

## Circuit Breaker

Il loop si ferma automaticamente se:
- ≥3 errori consecutivi
- ≥5 iterazioni senza miglioramenti
- Token budget esaurito

## Estensione

### Aggiungere un nuovo tool

1. Definisci schema in `steering/schemas/tool_schemas.json`
2. Implementa in `agent/tools.py`
3. Registra in `ToolRegistry._tools`

### Creare task personalizzato

```json
{
  "name": "my_task",
  "max_iterations": 25,
  "strategy": {
    "type": "sequential",
    "phases": [...]
  }
}
```

## Troubleshooting

| Problema | Soluzione |
|----------|-----------|
| "Ollama non raggiungibile" | Verifica `ollama serve` e `http://localhost:11434` |
| Timeout | Aumenta `num_predict` o riduci complessità task |
| Out of memory | Usa `qwen3:4b` invece di 9b, riduci `num_ctx` |
| Modifiche non salvate | Controlla permessi su `prometeo_kg.json` |

## Note

- L'agente modifica **solo** `prometeo_kg.json`, non il file binario della topologia
- Per applicare cambiamenti al campo: ricarica Prometeo
- Il bridge legge il lessico da `prometeo_topology_state.bin` ma in sola lettura
