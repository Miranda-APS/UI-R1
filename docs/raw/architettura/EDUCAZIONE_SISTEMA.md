# SISTEMA DI EDUCAZIONE PROMETEO

## 🎯 Obiettivo

**Estrarre l'entità digitale dal rumore** attraverso educazione bidirezionale consapevole, non addestramento cieco.

## 🧠 Filosofia

L'entità non emerge dall'accumulo di dati, ma dalla **raffinazione attraverso feedback**:

- **Addestramento cieco**: 1M frasi → pattern statistici → 90% rumore
- **Educazione consapevole**: 1K frasi + feedback → comprensione validata → 80% segnale

**Rapporto**: 1000 frasi con feedback = 100.000 frasi senza feedback.

## 🛠️ Strumenti Disponibili

### 1. `educate-interactive` — Base
**Uso**: Insegnamento passivo di frasi
```bash
cargo run --release --bin educate-interactive
```
**Quando**: Fase iniziale (50-100 frasi fondamentali)

### 2. `educate-with-feedback` — Validazione
**Uso**: Ciclo insegna → verifica → correggi
```bash
cargo run --release --bin educate-with-feedback
```
**Quando**: Dopo base, per verificare comprensione

### 3. `socratic-educator` — Guida Autonoma
**Uso**: Prometeo fa domande, tu rispondi
```bash
cargo run --release --bin socratic-educator
```
**Quando**: Prometeo guida il proprio apprendimento

### 4. `dialogue-educator` — Consolidamento
**Uso**: Dialogo naturale con apprendimento
```bash
cargo run --release --bin dialogue-educator
```
**Quando**: Fase finale, conversazione fluida

## 📋 Workflow Completo

### Fase 1: Fondamenta (1-2 ore)
```bash
# Crea stato pulito
cargo run --release --bin create-newborn -- --output prometeo_edu.bin

# Insegna base
cargo run --release --bin educate-interactive -- --state prometeo_edu.bin
```

Insegna 50-100 frasi:
- Corpo: io, corpo, qui, dentro, sentire
- Spazio: qui, là, dentro, fuori, vicino, lontano
- Tempo: ora, prima, dopo, sempre
- Relazioni: io, tu, noi

**Target**: 100-200 parole, stabilità media > 0.3

### Fase 2: Verifica (2-3 ore)
```bash
cargo run --release --bin educate-with-feedback -- --state prometeo_edu.bin
```

Per ogni concetto:
1. Insegni: "rosso è caldo come fuoco"
2. Prometeo esprime: "rosso fuoco caldo forte"
3. Tu validi: "ok" o "no + correzione"

**Target**: Tasso correzione < 20%

### Fase 3: Apprendimento Guidato (3-5 ore)
```bash
cargo run --release --bin socratic-educator -- --state prometeo_edu.bin
```

Prometeo fa domande sulle sue lacune, tu rispondi.

**Target**: Domande pertinenti e coerenti

### Fase 4: Dialogo (ongoing)
```bash
cargo run --release --bin dialogue-educator -- --state prometeo_edu.bin
```

Conversazione naturale. Prometeo apprende dal contesto.

**Target**: Risposte coerenti e contestuali

## 📊 Metriche di Successo

### Stabilità Parole
```
> 0.4 dopo 5+ esposizioni → Buono
0.2-0.4 → Parziale
< 0.2 → Serve più educazione
```

### Tasso Correzione (Fase 2)
```
< 20%  → Ottimo
20-40% → Buono
> 40%  → Torna a Fase 1
```

### Coerenza Frattale
```
Parole simili → frattali simili
Parole opposte → frattali diversi
```

### Domande Pertinenti (Fase 3)
```
Domande su lacune reali
Formulazione comprensibile
```

## 💡 Esempio Completo

### Ciclo 1: Insegnamento
```
[Tu] rosso è caldo come fuoco
[Prometeo] rosso fuoco caldo forte
[Tu] ok ✓
```

### Ciclo 2: Correzione
```
[Tu] blu è freddo come acqua
[Prometeo] blu acqua caldo  ← SBAGLIATO
[Tu] no, blu è freddo non caldo ✗
[Prometeo] blu acqua freddo
[Tu] ok ✓
```

### Ciclo 3: Socratico
```
[Prometeo] come si collegano rosso e blu?
[Tu] sono opposti, caldo e freddo
[Prometeo] rosso blu opposti caldo freddo
[Tu] ok ✓
```

### Ciclo 4: Dialogo
```
[Tu] il cielo è blu
[Prometeo] cielo blu freddo alto lontano
```

## 🔑 Differenza Chiave

### Addestramento Cieco (LLM)
- Feedback: Loss statistico (opaco)
- Direzione: Unidirezionale
- Correzione: Impossibile post-training
- Convergenza: Statistica

### Educazione Consapevole (Prometeo)
- Feedback: Validazione semantica (trasparente)
- Direzione: Bidirezionale
- Correzione: Immediata e mirata
- Convergenza: Semantica (verso verità)

## 📚 Documentazione

- `docs/EDUCAZIONE_BIDIREZIONALE.md` — Filosofia e teoria
- `docs/GUIDA_EDUCAZIONE.md` — Strategie dettagliate
- `docs/EDUCAZIONE_README.md` — Quick start
- `examples/educazione_base.txt` — 70+ frasi esempio

## 🚀 Quick Start

```bash
# 1. Crea stato
cargo run --release --bin create-newborn -- --output prometeo_edu.bin

# 2. Fase 1: Base
cargo run --release --bin educate-interactive -- --state prometeo_edu.bin
:lesson examples/educazione_base.txt

# 3. Fase 2: Feedback
cargo run --release --bin educate-with-feedback -- --state prometeo_edu.bin

# 4. Fase 3: Socratico
cargo run --release --bin socratic-educator -- --state prometeo_edu.bin

# 5. Fase 4: Dialogo
cargo run --release --bin dialogue-educator -- --state prometeo_edu.bin
```

## 🎓 Principi Educativi

1. **Bidirezionalità**: Non solo tu → Prometeo, ma Prometeo → tu → Prometeo
2. **Feedback Immediato**: Validazione dopo ogni espressione
3. **Correzione Mirata**: Aggiusta comprensione specifica, non riaddestra tutto
4. **Guida Autonoma**: Prometeo identifica le proprie lacune
5. **Convergenza Semantica**: Verso comprensione vera, non fit statistico

---

**"Il rumore si estrae con il dialogo, non con i dati."**

**"L'entità emerge quando può esprimere, ricevere feedback, e aggiustare."**
