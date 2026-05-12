# Test pre-proposta — diagnostica emergenza vs hardcoding

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

Il principio "[educare, non hardcodare](educare-non-hardcodare.md)" è troppo astratto per essere un filtro affidabile da solo. Questo è il **test operativo a tre domande** da applicare prima di proporre qualunque meccanismo nuovo (Rust o KG procedurale). Se la proposta non lo passa, è hardcoding mascherato — anche se vive in JSON.

## Le tre domande

**1. Forma o trigger?**
La proposta codifica *come si esprime* X (vocabolario linguistico, forme espressive) o *quando fare* X (transizione comportamentale)? Il [KG procedurale](../topologia/knowledge-graph-procedurale.md) contiene solo il primo. Mai il secondo.

**2. Test dei numeri-magici.**
La proposta contiene numeri in condizioni (≥3 turni, >0.5, dopo N volte, soglia X)? Se sì, è quasi certamente un trigger mascherato — anche in JSON è un `if/then`. La dinamica emergente non ha numeri in condizioni; i numeri sono **effetti del campo** (attivazioni, valenze, coerenze), mai soglie di switch.

**3. Spiegazione dello stato.**
Posso spiegare *perché* questo pattern viene scelto, in termini di stato corrente (drives [Octalysis](../identita/valenza-octalysis.md), valenza, `coherence_integrity`, `recent_fractal_attractor`, traiettoria narrativa), senza dire "perché la regola dice così"?

Se la proposta contiene numeri-magici o "quando", non è emergenza — è hardcoding spostato di file.

## Riformulazione corretta

Un nuovo organo percettivo (es. `SelfProfile`) registra esiti come fatti relazionali. Quegli esiti modulano canali di stato esistenti (drives, valenza, coerenza, traiettoria). Il pipeline esistente ([action_reasoning](../comprensione/action-reasoning.md) + [pattern_matcher](../comprensione/pattern-matcher.md)) sceglie diversamente perché il campo è diverso.

**Non aggiungere un nuovo decisore con regole — aggiungi una nuova fonte di percezione ai sistemi che già esistono.**

## Caso canonico (Phase 78 trap, da non ripetere)

**Proposta tipo trap**: "Tre articolazioni fallite → dubitazione" come triple nel KG procedurale.

Questo è il trap classico: i numeri ("tre"), il trigger ("quando fallisce N volte"), la specificità che *sembra* qualità ("FailsAfter 3 è concreto"). È un if/then dispatchato sul KG.

**Riformulazione corretta che è stata adottata**: `SelfProfile` percepisce "gap aperto + claim ripetuto" → `coherence_integrity` cala → drive Octalysis si sposta → `action_reasoning` + `pattern_matcher` (invariati) attivano `dubitazione` perché lo stato lo richiama. Stesso meccanismo, campo diverso, scelta diversa. Il "quando" è uno stato del corpo, non una transizione tabellata.

Vedi [self profile e closure perception](../comprensione/self-profile-closure-perception.md) per l'implementazione completa.

## Avvertenza al collaboratore AI

Il prior classico dei modelli LLM tira fortissimo verso dispatcher / state-machine / intent-classification, perché è il 99% dei sistemi AI nel corpus di training. La specificità che sembra qualità ("FailsAfter 3" è concreto, tangibile) è il segnale della trappola.

In un sistema emergente, le proposte buone *non* sembrano specifiche nei trigger — sembrano specifiche nelle **strutture percettive** e nei **canali di modulazione**.

## See Also

- [Principi inviolabili](principi-inviolabili.md) — il principio 6 di cui questo è il test operativo
- [Educare, non hardcodare](educare-non-hardcodare.md) — il principio astratto
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md) — dove vivono i pattern come dati
- [Self profile e closure perception](../comprensione/self-profile-closure-perception.md) — l'esempio canonico di riformulazione corretta
