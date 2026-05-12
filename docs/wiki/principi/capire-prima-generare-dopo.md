# Capire prima, generare dopo

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79); Phase 73 design (comprehension_report)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md); [06_campo_inferenza](../../raw/libretto/06_campo_inferenza.md)

## Overview

L'output non importa se UI-R1 non ha prima **capito davvero** l'input. Questo principio governa l'intera pipeline Phase 71-79: prima di generare una sola parola, l'entità scrive (letteralmente, come strutture dati) un `ComprehensionReport` in italiano leggibile su cosa ha capito, una `ActionDecision` esplicita su cosa intende fare e perché, e SOLO POI il pattern matcher istanzia la voce. La generazione è conseguenza, non motore.

## Cosa NON è

Capire prima ≠ classificazione intent.

L'approccio LLM standard (`detect_intent(input) → dispatch_response(intent)`) viola questo principio:
- Comprime l'input in una label discreta (Greeting / Question / EmotionalReport / …)
- Genera la risposta dalla label, non dalla **struttura** dell'input

In UI-R1 capire prima significa:
- Estrarre `SpeakerClaim` (chi-sta-dicendo-cosa-su-chi) — vedi [comprehension report](../comprensione/comprehension-report.md)
- Identificare `signifier_gaps` (parole atomiche con `context` opzionale) — i vuoti aperti dall'enunciato
- Registrare il fatto nello `SpeakerProfile` (memoria del parlante senza decay)
- Cross-referenziare con `SelfProfile` (cosa l'entità ha appena fatto) — vedi [self profile e closure perception](../comprensione/self-profile-closure-perception.md)
- Solo allora `decide_action` produce una decisione *esplicita*

## La pipeline (Phase 71-79)

```
input italiano
   │
   ▼  parse SpeakerClaim
   ▼  SpeakerProfile.register_claim()
   ▼  ComprehensionReport (speech_act + signifier_positions + signifier_gaps + inferences + self_relevance)
   ▼  detect_closure(SelfProfile, SpeakerProfile, current_turn) → Option<ClosurePerception>
   ▼  modulazioni di stato (coherence_integrity, drives Octalysis)
   ▼  ActionDecision (kind + target + shape + anchor_words + reasoning testuale)
   ▼  pattern_matcher (legge pattern dal KG procedurale, istanzia slot)
   ▼  italiano in uscita
```

Vedi [pipeline di comprensione](../comprensione/pipeline-comprensione.md) per il dettaglio.

## Cosa si guadagna

**Trasparenza.** Ogni turno produce un `reasoning` testuale: "ho scelto pattern X perché ho percepito gap Y dal claim Z fatto dal turno N". Non un punteggio softmax — una frase italiana.

**Continuità.** `SpeakerProfile` accumula fatti specifici del parlante (`self_facts`, `entity_facts`, `open_questions`, `gaps`, `mentioned`, `name`). `SelfProfile` accumula le proprie scelte come fatti relazionali (turno, kind, gap_attended, anchors_used) — **mai la stringa di output renderizzato**. Vedi [continuità via SpeakerProfile](../comprensione/speaker-profile.md).

**Closure perception.** Quando il parlante chiude un gap che l'entità aveva attended, emerge naturalmente la percezione che il cerchio articolazione è completo — non da un if/then, ma dal cross-reference dei due profili. È così che "ho paura" → "Di cosa hai paura?" → "del buio" produce "Hai paura del buio" (recognition) anziché "Buio è un fenomeno" (asserzione isolata).

## Conseguenza per il design

Quando si aggiunge una nuova capacità (un nuovo speech_act, un nuovo pattern espressivo, una nuova forma di gap), si **estendono le strutture percettive prima**, mai i decisori. Vedi [Test pre-proposta](test-pre-proposta.md) per il filtro completo.

## Quote rilevante

> "Non aggiungere un nuovo decisore con regole — aggiungi una nuova fonte di percezione ai sistemi che già esistono."

## See Also

- [Comprehension report](../comprensione/comprehension-report.md)
- [Pipeline di comprensione](../comprensione/pipeline-comprensione.md)
- [Speaker profile](../comprensione/speaker-profile.md)
- [Self profile e closure perception](../comprensione/self-profile-closure-perception.md)
- [Action reasoning](../comprensione/action-reasoning.md)
- [Pattern matcher](../comprensione/pattern-matcher.md)
- [Principi inviolabili](principi-inviolabili.md)
