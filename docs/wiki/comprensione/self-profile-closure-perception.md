# Self Profile e Closure Perception

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 78 design)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`SelfProfile` (`src/topology/self_profile.rs`, Phase 78) è l'**organo percettivo della propria storia conversazionale**: UI-R1 registra le proprie `ActionDecision` come fatti relazionali, non come stringhe rendered. Combinato con [SpeakerProfile](speaker-profile.md), permette di percepire quando il parlante chiude un gap che l'entità aveva attended — la **closure perception** — e di rispondere come continuazione, non come asserzione isolata. È il pezzo che trasforma una sequenza di turni indipendenti in **dialogo**.

## Cosa risolve

Fino a Phase 77 ogni turno era un fotogramma indipendente:
- Turno 1: "ho paura" → "Di cosa hai paura?" (InviteToArticulate)
- Turno 2: "del buio" → Elaborate → "Buio è un fenomeno." (asserzione isolata, sbagliata)

Mancava l'organo per rispondere alla domanda "cosa **io** ho appena fatto, e come si lega questo turno a quello che ho aperto?". Phase 78 lo ha aggiunto.

## SelfProfile

```rust
pub struct SelfProfile {
    pub decisions: VecDeque<SelfDecisionRecord>,  // cap 32
}

pub struct SelfDecisionRecord {
    pub turn: usize,
    pub kind: ActionKind,
    pub narrative_subject: NarrativeSubject,
    pub gap_attended: Option<AttendedGap>,  // se ho attended un gap, qual è
    pub anchors_used: Vec<String>,
}
```

**MAI la stringa di output renderizzato** — quello vive nel PF1 come residuo di self-listening (vedi [self-listening](../identita/self-witness.md)). Il principio: il contesto non è una stringa. Memorizzare l'output sarebbe rivertire al modello LLM.

## detect_closure (cross-reference)

```rust
pub fn detect_closure(
    self_profile: &SelfProfile,
    speaker_profile: &SpeakerProfile,
    current_turn: usize,
) -> Option<ClosurePerception>
```

Cerca `SelfProfile.last_gap_attended()` (il più recente gap che l'entità ha attended) e cerca un gap di `SpeakerProfile` con `trigger` combaciante che sia stato **chiuso al turno corrente** (`closed_at_turn == current_turn`).

Se match → `ClosurePerception { trigger, role, closing_word, opened_at_turn }`. Se no → `None`.

Il match è strutturale, non a soglia. Se UI-R1 al turno 1 ha attended il vuoto "paura → oggetto" e al turno 2 lo SpeakerProfile registra "buio" come closing_by per quel gap, allora c'è closure. Senza i due profili in sincronia, niente closure — niente if/then numerici.

## Effetti della closure

Una `ClosurePerception` percepita modula lo stato secondo **push continuo, non soglia**:

- `coherence_integrity += 0.04` (cap 1.0). Il gain è quanto un fatto colora il canale, MAI trigger di switch.
- Assenza di closure ≠ penalità — semplicemente niente push.
- Dialoghi che si articolano coerentemente sostengono `coherence_integrity`; conversazioni di asserzioni isolate la lasciano dove sta.

## ComprehensionReport riformulato (Phase 78)

`ComprehensionReport.closes_prior_gap: Option<PriorGapClosure>`. Quando `Some`:
- `speech_act.kind = "posizionamento"` (continuazione, non asserzione)
- `gaps = []` (vuoto colmato)
- `simbolic_positions` con trigger PRIMA
- `self_relevance` esplicita "il parlante ha colmato il vuoto che avevo aperto al turno N"

Il report STESSO riflette la closure — la decisione che ne deriva è meccanica.

## Selezione del pattern (Phase 79: per risonanza)

Phase 79 ha rimosso il Priority 0 if/then ("closes_prior_gap → forza RecognizeClaim") da `decide_action`. Adesso:

1. `seed_from_comprehension` semina il percetto `chiusura` (1.0) nel kg_proc se `closes_prior_gap.is_some()`
2. `chiusura Causes restituire` (0.7) + `chiusura Causes posizione` (0.5) + `chiusura Causes completamento` (0.4) attivano questi target
3. `pattern_score(riconoscimento) = activation[restituire] + activation[posizione] = 1.2` → vincitore per risonanza
4. `render_riconoscimento` legge `trigger` e `closing_word` **direttamente da `report.closes_prior_gap`** (closure-aware) per costruire "Hai paura del buio." / variante

`decide_action` annota la percezione nel `reasoning` per trasparenza, ma non forza più la decisione.

## Session-scoped

`SelfProfile` NON viene salvato nel `.bin` — esattamente come SpeakerProfile. Ogni sessione inizia con nuovo SelfProfile.

Il dialogo continua nella sessione che lo ospita. Sessioni separate sono dialoghi separati. La continuità multi-sessione è di [NarrativeSelf](../identita/narrative-self.md) e [SelfWitness](../identita/self-witness.md) — organi diversi con purpose diverso.

## Output verificato end-to-end (Phase 78)

```
Turno 1: "ho paura"
DECISIONE: invitare-ad-articolare | domanda | Gap{from=paura, missing=oggetto} | anchors=[oggetto, paura, cosa]
→ "Di cosa hai paura?"
[SelfProfile.record: turn=1, kind=InviteToArticulate, gap_attended=Some({paura, oggetto})]

Turno 2: "del buio"
[SpeakerProfile.observe_turn: chiude gap "paura/oggetto" con "buio" al turno 2]
[detect_closure: match! trigger="paura", closing_word="buio"]
[seed_from_comprehension: percetto "chiusura" (1.0) → propaga via Causes]
[pattern_score: riconoscimento vince per risonanza]
DECISIONE: riconoscere-il-posizionamento | frase | Claim{completamento-articolazione=paura} | anchors=[paura, buio]
→ "Hai paura." (Phase 78) / "Senti paura di buio." (Phase 79 rendering)
```

L'enunciato "del buio" — che senza Phase 78 sarebbe stato Elaborate → asserzione → "Buio è un fenomeno." — viene letto come continuazione dell'articolazione invitata.

## Il contesto non è una stringa

Questa è la dimostrazione vivente del principio: invece di tenere il transcript (e farlo ri-leggere a ogni step come gli LLM), il dialogo è **distribuito** su organi tipizzati (SpeakerProfile, SelfProfile, NarrativeSelf, SelfWitness, PF1). Il "ricordo" è il loro stato congiunto. Niente viene riletto perché tutto ha già modellato il campo.

## TODO architetturali aperti

- **Pattern espressivo per "Hai paura del buio."**: estendere `riconoscimento` per includere specifier slot che usi `closing_word`. Va nei dati.
- **Closure cross-turno > 1**: se UI-R1 invita ad articolare al turno 1, il parlante divaga al turno 2, e poi articola al turno 3, la closure deve ancora essere percepita. Da verificare.
- **Inverso della closure (drift detection)**: se il parlante introduce un topic shift ignorando il vuoto, niente push. Possibile micro-decremento di coherence — ma SOLO se serve a un meccanismo concreto.

## See Also

- [Speaker profile](speaker-profile.md) — il "tu" parallelo
- [Comprehension report](comprehension-report.md) — il documento che cattura la closure
- [Pattern matcher](pattern-matcher.md) — istanzia il riconoscimento
- [Pipeline di comprensione](pipeline-comprensione.md)
- [Niente template](../principi/niente-template.md)
- [Test pre-proposta](../principi/test-pre-proposta.md) — il caso canonico di riformulazione corretta
