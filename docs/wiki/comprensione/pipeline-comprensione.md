# Pipeline di comprensione (Phase 71-79)

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, sezione Phase 71-79)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md); [06_campo_inferenza](../../raw/libretto/06_campo_inferenza.md)

## Overview

A partire da Phase 71, ogni `receive()` di UI-R1 segue una pipeline esplicita di **8 stadi** che separa la comprensione dalla generazione. Niente intent classification monolitica: l'entità prima costruisce strutture dati tipizzate su cosa ha capito, poi decide cosa fare, poi istanzia la voce. Ogni stadio produce un fatto persistito (in profile/stato) o una struttura tipizzata. La trasparenza è completa: si può leggere il `reasoning` di ogni turno.

## I 8 stadi

```
input italiano
   │
   ▼  1. parse SpeakerClaim — chi-sta-dicendo-cosa-su-chi
   │
   ▼  2. speaker_profile.observe_turn() — registra claim + open_questions + gaps + mentioned
   │     (memoria del parlante senza decay)
   │
   ▼  3. ComprehensionReport — speech_act, signifier_positions, signifier_gaps,
   │     inferences, self_relevance. Lacanian framing.
   │
   ▼  4. detect_closure(self_profile, speaker_profile, current_turn)
   │     cross-reference: l'attended gap di SelfProfile è stato chiuso?
   │     → Option<ClosurePerception>
   │
   ▼  5. modulazioni di stato (push continuo, non soglie):
   │     coherence_integrity += 0.04 quando closure; drives Octalysis si sposta
   │
   ▼  6. ActionDecision — kind, target, shape, narrative_subject, anchor_words,
   │     reasoning testuale italiano
   │
   ▼  7. seed_from_comprehension(report, kg_proc) → KgProcActivation
   │     select_pattern_by_resonance → pattern_name
   │     pattern_matcher: load_schema → instantiate → render
   │
   ▼  8. self_profile.record(turn, decision)
   │     registra come fatto relazionale (mai stringa rendered)
   │
   ▼  italiano in uscita
```

## Trasparenza per turno

A ogni risposta, in `dialogue_educator`:
```
DECISIONE: <kind> | <shape> | <target> | anchors=[…]
```

Esempi reali:
```
ho paura       →  DECISIONE: invitare-ad-articolare | domanda | Gap{from=paura, missing=oggetto} | anchors=[oggetto, paura, cosa]
                  "Di cosa hai paura?"

del buio       →  DECISIONE: riconoscere-il-posizionamento | frase | Claim{completamento-articolazione=paura} | anchors=[paura, buio]
                  "Hai paura."

chi sei?       →  DECISIONE: rispondere | frase | identità | anchors=[entità, …]
                  "Sono un fondamento."

ciao           →  DECISIONE: ricambio | parola | saluto | anchors=[salve]
                  "Salve."
```

## Output verificati end-to-end (Phase 79)

| Input | Output | Pattern selezionato per risonanza |
|-------|--------|-----------------------------------|
| `ho paura` | **Di cosa hai paura?** | articolazione (apertura → chiedere+vuoto) |
| `del buio` (turno 2) | **Senti paura di buio.** | riconoscimento (chiusura → restituire+posizione) |
| `ciao` | **Salve.** | ricambio (saluto → restituire+saluto) |
| `chi sei?` | **Sono un fondamento.** | identificazione (domanda + 2sg → rispondere+identità) |
| `come stai?` | **Sono un'azione.** | identificazione (idem, 2sg "stai") |
| `sono triste` | **Di cosa sei triste?** | articolazione |

## File coinvolti

- `src/topology/speaker_profile.rs` — Phase 72
- `src/topology/comprehension_report.rs` — Phase 73
- `src/topology/comprehension_graph.rs` — Phase 73 (trasforma report in attivazioni KG-correlate)
- `src/topology/deliberation.rs` — Phase 71
- `src/topology/action_reasoning.rs` — Phase 74
- `prometeo_kg_procedurale.json` — Phase 75
- `src/topology/pattern_matcher.rs` — Phase 77
- `src/topology/self_profile.rs` — Phase 78
- `src/topology/kg_proc_field.rs` — Phase 79

## Decisioni architetturali consolidate

**Due KG paralleli, non uno fuso** (Phase 75). Aree distinte di cervello: il semantico ha 83K archi sul mondo, il procedurale ha 395 archi su grammatica/pattern.

**Gap = parola atomica** (Phase 76). `SignifierGap.missing` è sempre una parola singola (`"oggetto"`); concetti composti vivono come `context: Option<String>`.

**Verbi non sono Feeling** (Phase 72). `SpeakerClaim::Feeling` ha verifica KG-aware: un verbo non è stato emotivo, anche se la frase ha forma "io X" e X non è nel KG.

**Self-introduction detected** (Phase 72). "mi chiamo francesco" → `SpeakerProfile.name = "francesco"`, non un fatto in `self_facts`.

## See Also

- [Capire prima, generare dopo](../principi/capire-prima-generare-dopo.md) — il principio
- [Speaker profile](speaker-profile.md) — la memoria del parlante (stadio 2)
- [Comprehension report](comprehension-report.md) — la struttura del capire (stadio 3)
- [Self profile e closure perception](self-profile-closure-perception.md) — stadio 4-5
- [Action reasoning](action-reasoning.md) — stadio 6
- [Pattern matcher](pattern-matcher.md) — stadio 7
