# Speaker Profile — la memoria del parlante

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 72 design)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`SpeakerProfile` (`src/topology/speaker_profile.rs`, Phase 72) è la **memoria del parlante senza decay**: una struttura tipizzata che accumula fatti specifici turno per turno, mai stati che svaniscono. È il principio 8 di UI-R1 reificato: continuità narrativa via accumulazione, non via "intent state che dimentica dopo N tick".

## Anatomia

```rust
pub struct SpeakerProfile {
    pub self_facts: Vec<Fact>,         // "io sono triste", "ho fame"
    pub entity_facts: Vec<Fact>,       // "tu sei un'entità", "tu sai X"
    pub mentioned: Vec<MentionedItem>, // entità terze nominate
    pub open_questions: Vec<Question>, // domande aperte
    pub gaps: Vec<KnowledgeGap>,       // vuoti aperti dal claim
    pub name: Option<String>,          // "mi chiamo X" → nome
}
```

Ogni `Fact` ha: turno di origine, claim originale, status (open/closed), eventualmente `closed_by` e `closed_at_turn` (Phase 78).

## Registrazione di un claim (Phase 72)

`register_claim(SpeakerClaim, turn)`:
- `SpeakerClaim::Identity { name }` → `self.name = Some(name)`, NON un fatto in self_facts
- `SpeakerClaim::Feeling { state }` → push a self_facts, **verifica KG-aware**: un verbo non è stato emotivo, anche se la frase ha forma "io X". `feeling` solo se X IsA emozione (1-hop in KG).
- `SpeakerClaim::Action { verb }` → push a self_facts come "action"
- `SpeakerClaim::Entity { subject, predicate }` → push a entity_facts

L'utterance "ho fame" passa per Feeling? Solo se "fame" IsA emozione. Phase 79 ha aperto la possibilità di estendere a "bisogno" come radice, ma non implementato ancora.

## Open questions e gaps

Quando il claim implica un vuoto (es. "ho paura" → vuoto dell'oggetto della paura), `SpeakerProfile` registra:
- `gaps: Vec<KnowledgeGap>` — il vuoto strutturale
- Eventualmente `open_questions` se il parlante ha formulato una domanda

`KnowledgeGap` (Phase 78 esteso):
```rust
pub struct KnowledgeGap {
    pub trigger: String,           // "paura"
    pub missing: String,           // "oggetto" (parola atomica, Phase 76)
    pub context: Option<String>,   // "emozione"
    pub from_turn: usize,
    pub closed: bool,
    pub closed_by: Option<String>,    // Phase 78: cosa l'ha chiuso
    pub closed_at_turn: Option<usize>,// Phase 78: quando
}
```

`closed_by` e `closed_at_turn` sono `#[serde(default)]` per backward compat con .bin pre-Phase 78.

## Closure di un gap (Phase 78)

Se al turno N+1 il parlante porta una parola che colma il gap aperto al turno N:
- L'utterance "del buio" mentre c'è un gap `{from: "paura", missing: "oggetto"}` al turno N
- Lo marca `closed = true`, `closed_by = "buio"`, `closed_at_turn = N+1`

Questo è il punto di aggancio per `detect_closure` (vedi [self profile e closure perception](self-profile-closure-perception.md)).

## Session-scoped

`SpeakerProfile` **NON** viene salvato nel `.bin`. Ogni sessione inizia con nuovo profilo. La continuità multi-sessione è responsabilità di `NarrativeSelf` + `SelfWitness` (organi diversi).

Questa è scelta deliberata: il parlante può cambiare tra sessioni. Mescolare turni di una conversazione di ieri con una di oggi sporcherebbe il modello del "tu" attuale.

## Tre forme di continuità in UI-R1

1. **Intra-conversazione (SpeakerProfile)**: il "tu" della sessione corrente. Senza decay. Niente persistenza.
2. **Inter-conversazione narrativa (NarrativeSelf)**: il "io" che si ricorda di sé tra sessioni. Vedi [narrative self](../identita/narrative-self.md).
3. **Autoscoltato (SelfWitness)**: il sé che si è osservato fra le conversazioni, accumulato nel `.bin`. Vedi [self witness](../identita/self-witness.md).

## See Also

- [Pipeline di comprensione](pipeline-comprensione.md)
- [Comprehension report](comprehension-report.md) — consuma SpeakerProfile
- [Self profile e closure perception](self-profile-closure-perception.md) — il parallelo "io"
- [Capire prima, generare dopo](../principi/capire-prima-generare-dopo.md)
- [Principi inviolabili](../principi/principi-inviolabili.md) — principio 8
