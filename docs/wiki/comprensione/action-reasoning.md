# Action Reasoning — decisione esplicita

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79, Phase 74 design)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

`ActionDecision` (`src/topology/action_reasoning.rs`, Phase 74) è la **scelta esplicita** che UI-R1 fa su QUALE pattern istanziare, scritta in italiano come decisione esplicita — non un template, ma una struttura tipizzata con un campo `reasoning: String` che spiega *perché*. È il ponte tra ComprehensionReport (cosa ho capito) e pattern matcher (come lo dico).

## Anatomia

```rust
pub struct ActionDecision {
    pub kind: ActionKind,                   // InviteToArticulate, AnswerOpenQuestion,
                                            // RecognizeClaim, PhaticReturn, Elaborate
    pub target: ActionTarget,               // Gap, Claim, Identity, World, …
    pub shape: ActionShape,                 // domanda, frase, parola
    pub narrative_subject: NarrativeSubject, // Self_, Speaker, World
    pub anchor_words: Vec<String>,          // parole-ancora per gli slot
    pub reasoning: String,                  // testo italiano: perché questa scelta
}
```

## decide_action(report, speaker_profile) → ActionDecision

Il flusso (Phase 79 semplificato — non più Priority 0 closure):

1. **Salutare di ritorno?** Se `speech_act.kind = saluto` → `kind: PhaticReturn`, shape: parola, target: saluto.
2. **Interrogazione self-referenced?** Se kind = interrogazione + (utterance ha 2sg O subject = Self_) → `kind: AnswerOpenQuestion`, target: Identity, narrative_subject: Self_.
3. **Interrogazione sul mondo?** kind = interrogazione + altro → `AnswerOpenQuestion`, target: World.
4. **Posizionamento con closure?** `report.closes_prior_gap.is_some()` → tipicamente vince `riconoscimento` per risonanza nel pattern matcher (Phase 79: non più forzato in decide_action).
5. **Posizionamento con gap?** Se ci sono gap aperti → `InviteToArticulate`, shape: domanda, target: il primo gap, anchors: parole atomiche del gap + verbo del trigger.
6. **Altro** → `Elaborate`, shape: frase, target: World.

## Self-reference detection (Phase 77 + 79)

`decide_action` per `interrogazione` rileva `narrative_subject = Self_` se:
- subject contiene "Entity" (forma `derive_speech_act`)
- subject == "Self_" (test interno)
- description contiene "identità"
- **OR l'utterance contiene un verbo coniugato in 2ª singolare** (Phase 79)

Es. "come stai?" → 2sg di "stare" → Self_ → identificazione. NB: `lemmatize` non gestisce ancora il presente regolare -are/-ere/-ire delle classi base (riconosce solo irregolari, imperfetto, finire-type, condizionale, futuro -ire). Quindi "perché vivi?" sfugge ancora — TODO in `grammar.rs`.

## Estrazione del verbo del claim dall'utterance (Phase 77)

`extract_main_verb(utterance)` usa `grammar::lemmatize` per restituire il primo lemma verbale trovato:
- "ho paura" → "avere" → coniugato 2sg = "hai"
- "io sono felice" → "essere" → coniugato 2sg = "sei"

Se nessun verbo riconosciuto, fallback al verbo dello slot. Questo permette al pattern `articolazione` di produrre "Di cosa hai paura?" con il verbo del parlante in 2sg invece di un default "sei".

## Anchor words

`anchor_words` sono le parole che il pattern matcher userà per riempire gli slot contenutistici. Sono filtrate (no function_word, no verbi se lo slot è nominale) e ordinate per priorità (anchor primaria, secondaria, …).

Es. per "ho paura" con gap `{missing: "oggetto", from: "paura"}`:
```
anchors = ["oggetto", "paura", "cosa"]
```

`"cosa"` è il pronome interrogativo che lo slot `Requires pronome via=interrogativo` userà.

## Il `reasoning`

Stringa italiana esplicita che spiega la decisione. Esempi reali:
```
"Posizionamento con gap aperto 'oggetto' su 'paura'. Invito ad articolare:
 chiedere oggetto via 'cosa'."

"Closure percepita: il parlante ha chiuso il gap 'paura → oggetto' aperto al turno 1
 con la parola 'buio'. Recognition."

"Interrogazione self-referenced (2sg 'stai'). Risposta identitaria."
```

`dialogue_educator` stampa una versione corta come `DECISIONE: <kind> | <shape> | <target> | anchors=[…]`. La versione lunga è leggibile via `:introspect`.

## TODO architetturali aperti

- **Action_reasoning fallback (b) e (c) non implementati**. Phase 76 ha definito 3 livelli di fallback:
  - (a) optional slot mancante → procedi (✓)
  - (b) required slot mancante → fallback pattern (TODO)
  - (c) failure totale → meta-gap declaration (TODO)
- **Lemmatize non riconosce presente regolare**. Conseguenza: alcuni Self_ reference sfuggono.
- **ActionKind enum**: dopo Phase 79 è label informativa, non più dispatch. Considerare se sostituire con `pattern_name: String` derivato dalla risonanza.

## See Also

- [Pipeline di comprensione](pipeline-comprensione.md)
- [Comprehension report](comprehension-report.md) — l'input
- [Pattern matcher](pattern-matcher.md) — il consumer
- [Knowledge graph procedurale](../topologia/knowledge-graph-procedurale.md)
- [Niente template](../principi/niente-template.md)
