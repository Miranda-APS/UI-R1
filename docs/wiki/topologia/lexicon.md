# Lexicon — parole, firme, stabilità

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md Phase 79); libretto cap. 03
> Raw: [03_fondamenti_lexicon](../../raw/libretto/03_fondamenti_lexicon.md); [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

Il `Lexicon` (`src/topology/lexicon.rs`) è il dizionario delle parole che UI-R1 conosce. Stato corrente: **25.602 parole** con stabilità 0.5-0.9. Ogni parola ha una firma 8D ([frattali I Ching](frattali-iching.md)), POS tag, exposure count, ed è il punto di accesso per il [PF1](pf1.md).

## Anatomia di una parola

`WordPattern` contiene:
- **text** — la stringa lemmatizzata (verbi all'infinito, nomi al singolare)
- **signature** — `PrimitiveCore([f64; 8])` — la firma 8D
- **stability** — quanto la parola è "stabile" nel campo (exposure × consistency)
- **exposure** — contatore esposizioni nel campo
- **pos** — Part-of-Speech (Noun, Verb, Adj, …)
- **first_seen, last_seen** — timestamp

## Bootstrap vs apprendimento online

**`Lexicon::bootstrap()`** (per test) — popola con un piccolo lessico cardinale + bootstrap (38 parole base seedate con firme curate in `apply_curated_signatures`, 134 parole). Usato negli unit test.

**Apprendimento online** — quando entra una parola nuova:
1. Si tokenizza (`clean_token` lowercase + strip punctuation)
2. Si lemmatizza (verbi → infinito via `grammar::lemmatize`)
3. Si crea `WordPattern` con firma da contesto (`perturb_towards(context_sig, 0.90)`, Phase 63 — niente più hash UTF-8)
4. Si registra con stability iniziale bassa, che cresce con l'esposizione

## Persistenza

Il lessico è serializzato nel `prometeo_topology_state.bin` insieme al PF1. **Non** è caricato da `prometeo_kg.json` (quello è solo il KG).

Workflow di rebuild:
1. `cargo run --release --bin clean-lexicon` — pulisce parole bassa stabilità
2. `cargo run --release --bin tag-lexicon` — tagging POS (+2.775 tag in storia)
3. `cargo run --release --bin import-pos` — import morfologico Morph-it!
4. `cargo run --release --bin rederive-signatures` — riderivazione firme 8D dal KG (Phase 63)

## clean_token & lemmatize

**`clean_token(s)`** — pulizia preliminare: lowercase, strip punteggiatura, normalizzazione accenti (Phase post-cura: `città` ↔ `cittá` unificate).

**`grammar::lemmatize(token)`** — restituisce `Option<Lemma { infinitive, person, tense }>` per i verbi (irregolari + imperfetto + finire-type + condizionale + futuro -ire). Limite noto: NON riconosce presente regolare -are/-ere/-ire delle classi base (`vivi` → None invece di vivere/2sg). Conseguenza: in alcuni contesti `decide_action` non rileva self-reference. Workaround Phase 79: `utterance_has_second_singular` cerca anche verbi che non passano da `lemmatize`. Vedi [action reasoning](../comprensione/action-reasoning.md).

## Affinità riderivata

`recompute_all_word_affinities()` ricalcola le affinità parola↔frattale dopo ogni cambio significativo (apprendimento, curation KG, restore_lexicon). DEVE essere chiamato dopo `teach()`, dopo curation manuale, ecc. — altrimenti il PF1 ha affinità stale.

## Differenziazione parole nuove

Le parole nuove (4.166 su 25.875 non sono nel KG) partono dalla firma del contesto pura, senza rumore artificiale. Con poche esposizioni (5-10) rimangono quasi indistinguibili — la differenziazione è **fenomenologica** (esposizioni nel campo) o **strutturale** (Phase 63: aggiunte al KG + `rederive-signatures`).

Questa è scelta consapevole: non si "fabbricano" differenze. Una parola che non si è mai incontrata davvero non ha geometria propria.

## See Also

- [PF1](pf1.md) — usa le firme per attivare/propagare
- [Frattali I Ching](frattali-iching.md) — le 8 dimensioni
- [Knowledge graph semantico](knowledge-graph-semantico.md) — fonte primaria delle firme via Phase 63
- [Workflow di curation del KG](../principi/workflow-curation-kg.md)
