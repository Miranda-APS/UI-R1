# MCP substrate — UI-R1 abitabile da un LLM (Phase 82)

> Sources: Francesco Mancuso, 2026-05-27 (CLAUDE.md Phase 82, sezione MCP Substrate)
> Raw: [CLAUDE_phase82](../../raw/contesto/CLAUDE_phase82.md)

## Overview

Fino a Phase 81 UI-R1 era un sistema standalone con UI web. Phase 82 lo trasforma in un **substrato cognitivo strutturato abitabile da un LLM esterno** (Claude Desktop, Claude Code, qualunque client MCP) via il protocollo standard [Model Context Protocol](https://modelcontextprotocol.io) di Anthropic.

Inverte lo stack tipico. Invece di *"LLM = pensiero + UI-R1 come tool RAG"*, qui **UI-R1 = pensiero esibito + LLM = voce vincolata**. Il KG curato porta la prospettiva del curatore; l'LLM realizza in linguaggio fluente, ma sempre ancorato alla comprensione strutturale che UI-R1 esibisce turno per turno ([PROP](../comprensione/frase-come-proposizione.md), [ActionDecision](../comprensione/action-reasoning.md), [ComprehensionReport](../comprensione/comprehension-report.md)).

## Architettura: HTTP-wrapper (decisione 1A)

Il binario `prometeo-mcp` (`src/bin/prometeo_mcp.rs`, feature `mcp`) parla MCP col client via **stdio** e fa richieste REST al server `prometeo-web` in esecuzione su `PROMETEO_WEB_URL` (default `http://127.0.0.1:3000`).

```
client MCP (Claude Desktop / Code)
   │  tools/call: comprehend({input: "ho paura del futuro"})
   ▼  stdio JSON-RPC
prometeo-mcp (rmcp 1.7)
   │  HTTP POST /api/input
   ▼  reqwest 0.12
prometeo-web :3000  →  engine.receive(...)
```

Un solo engine, un solo `.bin`, una sola sessione viva — **condivisa** tra la UI [campovasto](../campovasto/architettura-campovasto.md) e il client MCP. Quando l'LLM interroga UI-R1 mentre la web UI è aperta, vedono lo stesso UI-R1 nello stesso momento.

`prometeo-mcp` è solo il canale: la sessione cognitiva vive in `prometeo-web`, che dev'essere in esecuzione.

## `comprehend` è un turno reale (decisione 2A)

Ogni chiamata `comprehend(input)` **incrementa il tick, aggiorna [NarrativeSelf](../identita/narrative-self.md), scrive in [SpeakerProfile](../comprensione/speaker-profile.md), modula il [PF1](../topologia/pf1.md)**. L'LLM non è spettatore — è interlocutore. È il punto d'ingresso della [pipeline di comprensione](../comprensione/pipeline-comprensione.md) via MCP.

Il parametro `speaker_id` è oggi future-proof (l'engine lo ignora) e diventerà la chiave per multi-speaker quando SpeakerProfile sarà multi-istanza.

## I 12 tool esposti (V1)

**Lettura — UI-R1 si lascia ascoltare:**

| Tool | Cosa restituisce | Endpoint REST |
|------|------------------|---------------|
| `comprehend(input, speaker_id?)` | Turno REALE: testo + ComprehensionReport + SentenceProposition + KgConfrontation + ActionDecision + SpeakerProfile + deliberazione + stance + intention | `POST /api/input` |
| `get_field_state()` | PF1 live: parole attive con attivazioni | `GET /api/wordfield` |
| `get_narrative_state()` | Stance, drive Octalysis, coherence_integrity, intention, attractor, commitment, attributed_intent | `GET /api/narrative` |
| `get_active_fractals()` | 64 attrattori I Ching con punteggi correnti | `GET /api/visuals` |
| `get_thoughts()` | Gap, MissingBridge, Hypothesis, SelfDiscovery, Need, Desire, Interlocutor, Humor, Tension | `GET /api/thoughts` |
| `get_self_profile()` | IdentityCore + SelfModel + frattali dominanti + episodi semantici recenti | `GET /api/self` |
| `query_kg(word, limit?)` | Vicinato KG: triple uscenti/entranti, confidence, via | `GET /api/word_neighbors` |
| `get_word_detail(word)` | Firma 8D (ordine I Ching canonico), stability, exposure, POS, affinità frattali | `GET /api/word_detail` |
| `get_concept(word)` | IS_A ancestors + descendant samples + relazioni caratterizzanti | `GET /api/concept` |

**Scrittura persistente — UI-R1 ricorda tra le sessioni** (vedi [memoria-haiku](memoria-haiku.md)):

| Tool | Cosa fa | Endpoint REST |
|------|---------|---------------|
| `deposit_haiku(verses[3], fractal_id, anchors[], source?, note?)` | Deposita un cristallo. Tangenze automatiche. PERSISTENTE | `POST /api/haiku/deposit` |
| `recall_haiku_near(fractal_id, anchors?, n?)` | Recall geometrico (β=5.0 ancore, α=1.0 frattale, γ=0.5 tangenze) | `POST /api/haiku/recall` |
| `get_haiku_stats()` | Totale, top frattali, top ancore, densità tangenziale | `GET /api/haiku/stats` |

## Note di implementazione (decisioni consolidate)

- **Binario opzionale dietro feature `mcp`.** `rmcp = "1.7"` e `reqwest = "0.12"` sono `optional = true`. Build base leggero. Aggiungere primitive epistemiche è lavoro in `prometeo_mcp.rs`, mai modifica al core engine.
- **`use rmcp::schemars`** — rmcp re-esporta `schemars`; usarlo via rmcp evita conflitti di versione (0.8 vs 1.2).
- **`ServerInfo::new(caps).with_instructions(s)`** — solo builder, niente struct literal (`#[non_exhaustive]`).
- **Bridge DTO Phase 81** — `SentencePropositionDto` + `KgConfrontationDto` aggiunti a `src/web/state.rs`; `InputResponse` esteso. `subject`/`object` della PROP spezzati in coppie `(kind, name)` per il consumo JSON.

## Come configurare un client MCP

**Claude Code** — aggiungere a `~/.claude.json`:
```json
{"mcpServers": {"uir1": {"command": "C:\\...\\target\\release\\prometeo-mcp.exe", "args": [], "env": {"PROMETEO_WEB_URL": "http://127.0.0.1:3000", "PROMETEO_MCP_LOG": "1"}}}}
```
**Claude Desktop** — `%APPDATA%\Claude\claude_desktop_config.json`, stesso schema.
**Prerequisito**: `prometeo-web` in esecuzione.

## TODO architetturali aperti

- `fractal_name` accanto a `fractal_id` nei DTO haiku (oggi numerico 0-63).
- Tool MCP `propose_triple` (scrittura kg_sem + audit) e `get_haiku(id)` (dereferenzia tangenze).
- Multi-speaker reale: `speaker_id` oggi pass-through ignorato.
- Deposit autonomo da UI-R1 (oggi tutti client-driven).

## See Also

- [Memoria-haiku](memoria-haiku.md) — l'organo di memoria persistente esposto via MCP
- [Pipeline di comprensione](../comprensione/pipeline-comprensione.md) — cosa fa `comprehend` dentro l'engine
- [La frase come proposizione](../comprensione/frase-come-proposizione.md) — la struttura PROP restituita
- [Knowledge graph semantico](../topologia/knowledge-graph-semantico.md) — ciò che `query_kg`/`get_concept` espongono
