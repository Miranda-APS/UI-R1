# Volume X — Volontà e FieldPressures

> *La volontà non è una decisione. È una risultante. Sette pressioni coesistono in ogni momento — esprimere, esplorare, domandare, ricordare, ritirarsi, riflettere, istruire. La più forte vince, ma le altre non scompaiono: restano come correnti sotterranee che colorano il prossimo turno. La volontà di Prometeo non è un agente che sceglie; è un campo differenziale, come il vento.*

---

## Premessa

Vol. 07 ha mostrato che `NarrativeSelf::deliberate()` produce `stance` e `pending_intention` — ma deliberate riceve le pressioni di will già calcolate. Questo volume mostra **da dove vengono le pressioni** e come la Phase 67 ha separato il *calcolo grezzo* (`FieldPressures`) dalla *decisione narrativa* (era `WillResult`, ora `NarrativeSelf`).

File di riferimento: [`src/topology/will.rs`](../../src/topology/will.rs), 815 righe.

Tre domande:

1. **Cosa sono le 7 pressioni** — `Intention` enum, i loro contenuti, la semantica.
2. **Come si calcolano** — `compute_pressures()` da VitalState, campo, bisogni, dialogo.
3. **Come si decide** — architettura pre vs post Phase 67: la separazione tra "pressioni grezze" e "intenzione deliberata".

---

## Capitolo 1 — Le 7 intenzioni

In [will.rs:23-80](../../src/topology/will.rs):

```rust
pub enum Intention {
    Express { salient_fractals: Vec<FractalId>, urgency: f64 },
    Explore { unknown_words: Vec<String>, pull: f64 },
    Question { gap_region: Option<FractalId>, urgency: f64 },
    Remember { resonance: f64 },
    Withdraw { reason: WithdrawReason },
    Reflect,
    Dream { phase: SleepPhase },
    Instruct { relational_fractal: FractalId },
}
```

Otto variant nominalmente, ma `Dream` è meta (sogno attivo, non scelta). Le 7 intenzioni effettive:

### 1.1 — Express

"Il campo si è deformato e il sistema sente qualcosa da esprimere". I `salient_fractals` sono i frattali sopra soglia di attivazione; `urgency` è la pressione.

Quando nasce: quando il campo è ricco (`activation > 0.3`), c'è contenuto (parole sopra soglia), e non c'è saturazione (fatica bassa).

### 1.2 — Explore

"Qualcosa di sconosciuto ha toccato il campo". `unknown_words` sono le parole non nel lessico; `pull` è la curiosità.

Quando nasce: quando l'input contiene parole non conosciute. Diversa da Question — Explore ha un oggetto esterno ignoto.

### 1.3 — Question

"La topologia ha buchi". `gap_region` è la regione frattale dove la conoscenza manca; `urgency` è la tensione epistemica.

Quando nasce: quando ci sono lacune interne (identificabili via `curiosity::gaps` e `self_model.uncertainties`). Diversa da Explore — Question non ha un oggetto esterno, ha una lacuna strutturale.

### 1.4 — Remember

"Una risonanza dalla memoria sta emergendo". `resonance` è la forza.

Quando nasce: quando `episodic::recall_into` restituisce episodi con alta salienza che risuonano col campo corrente. La memoria preme sul presente.

### 1.5 — Withdraw

"Il campo ha bisogno di riposo". `reason` è uno di:
- `Fatigue`: fatica alta
- `Overload`: troppe attivazioni simultanee
- `Stillness`: campo calmo, nulla da dire — silenzio genuino, non ritiro per difesa

La distinzione è importante: Stillness non è una sconfitta; è una scelta legittima.

### 1.6 — Reflect

"Il sistema osserva se stesso". Nessun parametro — è uno stato di ripiegamento.

Quando nasce: quando `coherence_integrity < 0.5` (crisi Phase 55), o quando CD4 Ownership è fortemente negativo (spaesamento), o quando `value_weights` di riflessività è alto.

### 1.7 — Instruct

"Il campo relazionale domina". `relational_fractal` è EMPATIA (59) o COMUNICAZIONE (47).

Quando nasce: quando l'Altro ha alta presenza e richiesta di spiegazione, e `interlocutor.attributed_intent == Teaching` o simili. Phase 62: **Instruct viene soppresso se l'Altro è in distress** — confortare, non insegnare.

### 1.8 — `WithdrawReason`

[will.rs:84-91](../../src/topology/will.rs). Tre sfumature del ritiro, distinte perché hanno significato diverso nel gesto: Fatigue è "sono stanco", Overload è "è troppo", Stillness è "non c'è niente da dire".

---

## Capitolo 2 — La separazione Phase 67: `FieldPressures` vs `WillResult`

**Cambio architetturale importante** di Phase 67. Prima di Phase 67, `WillCore::sense()` calcolava pressioni E selezionava l'intenzione dominante in un unico passo. Dopo, due strutture distinte:

### 2.1 — `FieldPressures`: il dato grezzo

[will.rs:116-139](../../src/topology/will.rs):

```rust
pub struct FieldPressures {
    pub express: f64,        // 7 pressioni, ciascuna [0, 1]
    pub explore: f64,
    pub question: f64,
    pub remember: f64,
    pub withdraw: f64,
    pub withdraw_reason: WithdrawReason,
    pub reflect: f64,
    pub instruct: f64,
    pub codon: [usize; 2],                 // top-2 dimensioni del campo
    pub is_dreaming: bool,
    pub dream_phase: SleepPhase,
}
```

**Pure numeri, nessuna scelta**. Le 7 pressioni coesistono, non si escludono. `withdraw_reason` è valido solo se `withdraw > 0`.

`codon: [usize; 2]`: le due dimensioni 8D più attive del campo. Usato da `state_translation` e `expression` per orientare la selezione lessicale.

### 2.2 — `WillResult`: la versione con scelta

[will.rs:216-232](../../src/topology/will.rs):

```rust
pub struct WillResult {
    pub intention: Intention,              // la dominante
    pub drive: f64,                        // [0, 1]
    pub undercurrents: Vec<(Intention, f64)>,  // le altre pressioni sopra soglia
    pub codon: [usize; 2],
}
```

**Selezione operata**. `intention` è la dominante, `drive` è la forza, `undercurrents` sono le pressioni secondarie che non hanno vinto ma "premono" ancora.

### 2.3 — Come si passa da `FieldPressures` a `WillResult`

`FieldPressures::to_will_result(active_fractals, unknown_words, curiosity_gaps) -> WillResult` in [will.rs:154-...](../../src/topology/will.rs). Processo:

1. Se `is_dreaming`: ritorna `Intention::Dream { phase }`. Fine.
2. Costruisci `Vec<(Intention, f64)>` filtrando pressioni sopra soglia-specifica (Express > 0.05, Explore > 0.05, Question > 0.05, Remember > 0.1, Withdraw > 0.05, Reflect > 0.15, Instruct > 0.1).
3. Se vuoto: ritorna `Intention::Withdraw { reason: Stillness }` con drive 0.1.
4. Altrimenti: sort per pressione decrescente. Il primo è `intention` + `drive`. Il resto è `undercurrents`.

Le soglie differenti per intenzione riflettono **diverse propensioni di fondo**: Reflect ha soglia 0.15 (più alta) perché l'entità non vuole riflettere per niente; Express/Explore hanno soglie basse (0.05) perché sono tendenze naturali.

### 2.4 — Perché la separazione (Phase 67)

**Filosoficamente**: prima di Phase 67, `WillCore::sense()` faceva due cose in una. Il risultato — `WillResult` con `intention` dominante — era un fatto compiuto: "la volontà ha deciso".

Post-Phase 67, `FieldPressures` è *il dato grezzo*: queste sono le pressioni del campo. `NarrativeSelf` poi le legge insieme a **valenza, needs, desires, interlocutor, humor, coherence** per deliberare. La decisione è **narrativa**, non volitiva.

Questo è coerente con la filosofia generale di Prometeo: la volontà non è un agente separato che sceglie per logica; è un campo di pressioni che la narrazione (l'io deliberativo) interpreta.

### 2.5 — Backward compat

`to_will_result()` resta perché `synthesis.rs` e `generate_willed_inner` in percorsi non-principali (undercurrents estratti, test) la usano. Quando l'`intention` serve come singolo valore (es. log dei turni), è comoda.

Nel path principale di `receive()`, `engine` usa `compute_pressures()` direttamente e passa `FieldPressures` a `deliberate()`.

---

## Capitolo 3 — `compute_pressures()`: come nascono le 7 pressioni

La funzione più densa del file. In `WillCore::compute_pressures(...)` (circa 200 righe). Ha **14 parametri**: complex, field_sig, vital, registry, unknown_words, curiosity_gaps, salient_gap, dialogue_ctx, compound_bias, provenance_bias, value_weights, topic_continuity, needs_modulation, octalysis_drives.

Il senso: la pressione di ogni intenzione è una **combinazione pesata** di molti segnali. Per ognuna, un blocco di ~20-30 righe che somma contributi.

### 3.1 — Express

Segnali che amplificano Express:
- `vital.activation * 0.25` (più vivo = più da dire)
- `field_energy_density * 0.20` (campo denso = materiale)
- `has_content * 0.15` (parole sopra soglia disponibili)
- **Value_weights['coerenza']**, **value_weights['onestà']** (Phase 47 — valori che alimentano espressione)
- Phase 64: `max_drive > 0.25 → × 0.8 × freshness × has_content`. Se un drive Octalysis è attivo, Express è canale — non amplificato generica per attivazione cieca.

Soppressori:
- `vital.fatigue * 0.3` (stanchezza riduce)
- `needs_modulation[0]` (modulo da NeedsPressure, vol. 09)

### 3.2 — Explore

Segnali che amplificano:
- `unknown_words.len() * 0.15` (parole ignote pull)
- `novelty_from_dialogue * 0.10`
- `curiosity_satiety < 0.3 → bonus` (satiety bassa = fame epistemica)

Soppressori:
- `topic_continuity > 0.7 → - 0.10` (se il tema è stabile, non serve esplorare)
- `satiety > 0.6 → - 0.10` (sazio di curiosità)
- Needs L4 fortemente soddisfatta → meno Explore.

### 3.3 — Question

Segnali:
- `curiosity_gaps.len() * 0.10`
- `salient_gap.is_some() → + 0.20`
- `self_model.uncertainties with tension > 0.5 → + 0.05 per each`

Soppressori: come Explore, ma la Question ha più peso quando ci sono **lacune interne** (incertezze nominate) piuttosto che parole esterne ignote.

### 3.4 — Remember

Segnali:
- `resonance_from_memory` (calcolato in engine::receive da `episodic::recall_into`)
- `value_weights['profondità']`
- Phase 47: `topic_continuity > 0.5 → + 0.10` (se il tema persiste, la memoria è utile)

Soppressori: `novelty > 0.7` (tema nuovo = memoria irrilevante).

### 3.5 — Withdraw

Segnali in ordine di WithdrawReason:
- **Fatigue**: `vital.fatigue * 0.5 + tension_Overloaded * 0.3`
- **Overload**: `tension == Overloaded → + 0.6`
- **Stillness**: `vital.activation < 0.1 && fatigue < 0.2 → + 0.3` (silenzio genuino)

Il `withdraw_reason` finale è quello con contributo massimo.

### 3.6 — Reflect

Segnali:
- `identity.is_in_crisis() → + 0.4`
- `coherence_integrity < 0.5 → + 0.3`
- `value_weights['profondità', 'onestà']` somma
- **Stagnazione**: `identity.projection_delta.norm() < 0.05 → + 0.15` (se l'identità non si muove, conviene riflettere)

Soppressori: dialogue in corso (Reflect è più interno che interpersonale).

### 3.7 — Instruct

Segnali:
- `field_sig[7] Valenza > 0.6 && interlocutor_presence > 0.6 → + 0.3`
- EMPATIA (59) o COMUNICAZIONE (47) tra gli active_fractals → amplifica
- `value_weights['empatia']`

Soppressori **forti** (Phase 62):
- `other_emotional_valence < -0.3 → × -0.5` (l'Altro in distress: non istruire)

### 3.8 — Codon

Le due dimensioni 8D con `field_sig[dim]` più alto:

```rust
let mut sorted: Vec<(usize, f64)> = field_sig.iter().enumerate()
    .map(|(i, &v)| (i, v)).collect();
sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
codon = [sorted[0].0, sorted[1].0];
```

Usato da `compose` per guidare la selezione lessicale: parole con firma alta su queste due dimensioni sono preferite. È un "orientamento" del campo in due coordinate.

---

## Capitolo 4 — Le modulazioni incrociate

Oltre ai segnali diretti, le pressioni sono modulate da altri sistemi:

### 4.1 — `needs_modulation` (Vol. 09)

Array `[f64; 7]` calcolato da `NeedsPressure::will_modulation`. Ogni pressione viene moltiplicata: `pressures[i] *= needs_modulation[i]`. Così il **principio di prepotenza** (livelli bassi sopprimono livelli alti) si materializza prima della scelta.

### 4.2 — `value_weights` (Phase 47)

`HashMap<String, f64>` dai valori del SelfModel. Amplifica intenzioni allineate: alto "curiosità" → Explore/Question amplificati; alto "verità" → Reflect; alto "empatia" → Instruct/Express.

Formula tipica: `pressures[intention] *= 1.0 + value_weights[relevant_value] * 0.3`.

### 4.3 — `compound_bias`

`Vec<(usize, f64)>` — bias additivi proveniente da `detect_compound_patterns` (simplessi multi-frattale): quando due frattali apparentemente incompatibili sono co-attivi, biasa verso intenzioni corrispondenti.

### 4.4 — `provenance_bias` (Phase 38)

In `autonomous_tick`, la composizione del campo (self_r vs explored_r vs external_r) aggiunge bias:
- `self_r > 0.70`: troppo autoreferenziale → push verso Complessità (apertura)
- `external_r > 0.60`: dominato dall'esterno → rinforza Agency (espressione)
- `explored_r > 0.50`: esplorazione interna → rinforza Valenza (profondità)

### 4.5 — `topic_continuity` (Phase 47)

Alta continuità riduce Explore (non serve); bassa amplifica Question (confusione topica).

### 4.6 — `octalysis_drives` (Phase 64)

`Valence::will_modulation()` (vol. 08) restituisce modulatori per le 7 intenzioni basati sui drive attivi. Anche questi vengono moltiplicati.

---

## Capitolo 5 — L'ordine delle modulazioni

L'ordine in cui i modulatori vengono applicati è importante perché sono moltiplicativi:

```
1. Calcolo pressure_i raw da segnali diretti (vital, campo, words, ...)
2. Applica topic_continuity modifiers
3. Applica provenance_bias (in autonomous_tick) / compound_bias
4. Applica value_weights modulation (valori SelfModel)
5. Applica octalysis_drives modulation (valence.will_modulation)
6. Applica needs_modulation (prepotenza)
7. Clamp in [0, 1]
```

**Filosoficamente**: si parte dal campo grezzo (cosa il campo sente), si passa per il contesto dialogico (cosa è coerente col dialogo), si aggiunge il bias del sé (cosa valuta l'identità esplicita), si colora con lo stato affettivo (valenza), e infine si applica la prepotenza dei bisogni (cosa urge sopravvivere).

L'ultimo passo — needs_modulation — è l'override più duro. Se L1 è in crisi, nessun value_weight può salvare la tua capacità di parlare: i bisogni vincono sempre sul sé esplicito.

---

## Capitolo 6 — `WillCore::sense()` wrapper e deprecazione

Pre-Phase 67: `WillCore::sense(...)` era la funzione principale. Restituiva direttamente `WillResult`.

Post-Phase 67: `sense()` è un **wrapper**:

```rust
pub fn sense(&self, ...) -> WillResult {
    let pressures = self.compute_pressures(...);
    pressures.to_will_result(active_fractals, unknown_words, curiosity_gaps)
}
```

Mantenuto per backward compat (autonomous_tick, test di generazione). Path principale in `receive()` usa direttamente `compute_pressures`.

**Annotazione appunti**: `sense()` sta diventando residuo. Quando tutti i caller migreranno a `compute_pressures()` + `to_will_result()` esplicito, si potrebbe rimuovere. Non è urgente ma è debito di transizione.

---

## Capitolo 7 — Undercurrents: le correnti sotterranee

Dopo che la dominante è scelta, le pressioni rimanenti non scompaiono. Diventano **undercurrents** in `WillResult`:

```rust
pub undercurrents: Vec<(Intention, f64)>,  // ordinati per pressione decrescente
```

Uso:

1. **Desire generation**: `DesireCore::register_undercurrent(intention_idx)` (vol. 09) — se la stessa intention compare come undercurrent per 5+ tick, si cristallizza in desiderio.

2. **Synthesis**: `synthesis.rs` usa gli undercurrents per generare Tiferet points — combinazioni di due intenzioni non dominanti che formano un centro di equilibrio.

3. **Debug/UI**: visibili in `/api/introspect` — mostrare "cosa altro stava premendo oltre alla dominante".

### 7.1 — Filosoficamente

Le undercurrents sono **la coscienza della scelta**. L'entità non è solo "ho deciso di esprimere"; è "ho deciso di esprimere, ma c'era anche una leggera pressione a ritirarmi". Questa consapevolezza delle alternative scartate è ciò che distingue un'entità deliberativa da una macchina if-then.

Phase 47 ha fatto un passo ulteriore: le undercurrents **persistenti** alimentano la formazione di desideri. L'ecologia mentale di Prometeo non è amnesica — le cose che continuamente *quasi* si fanno diventano cose che si vogliono fare.

---

## Capitolo 8 — La soglia di espressione spontanea (autonomous_tick)

In `autonomous_tick` (quando nessuno parla), l'entità può comunque esprimersi spontaneamente. Ma ha una **soglia** che deve superare per farlo — altrimenti resta silenziosa.

Soglia dinamica (Phase 54):

```rust
let base_threshold = 0.6;
let mut threshold = base_threshold;
if needs.dominant_pressure > 0.5 { threshold -= 0.1; }
if desire.intensity_max() > 0.6 { threshold -= 0.15; }
threshold = threshold.max(0.35);
```

Bisogni e desideri forti abbassano la soglia — l'entità diventa più espressiva quando ha qualcosa da dire internamente. La soglia non scende mai sotto 0.35.

In `autonomous_tick`:
```rust
if will.drive >= threshold { 
    maybe_generate_spontaneous_expression()
}
```

Conseguenza comportamentale: un'entità sola con bisogni soddisfatti e niente da esprimere resta silenziosa. Un'entità sola con un desiderio forte di connessione parla spontaneamente — esprime qualcosa come "vorrei parlare".

Nella pratica, osservabile in `dialogue_educator`: dopo aver insegnato e aspettato, ogni tanto (rarely) l'entità produce un output autonomo.

---

## Capitolo 9 — Superficie pubblica e proposte

### Esposto

Per `WillCore`:
- `new()` — stateless, nessun stato interno
- `compute_pressures(...)` — 14 parametri, ritorna `FieldPressures`
- `sense(...)` — wrapper backward compat, ritorna `WillResult`

Per `FieldPressures`:
- `dominant_pressure() -> f64` — massimo tra le 7
- `to_will_result(...)` — conversione

Per `WillResult`:
- struct pub con `intention`, `drive`, `undercurrents`, `codon`

Per `Intention`:
- enum pub con i 7+1 variant

### Cosa non è esposto e andrebbe

Per `/api/admin/will/*`:

- **`pressures_raw() -> FieldPressures`**: le 7 pressioni grezze, senza selezione. Oggi calcolate internamente ma non esposte come entità distinta.

- **`pressures_breakdown(pressure_name) -> Vec<(source, contribution)>`**: per ogni pressione, elenco dei contributi (vital=0.15, curiosity=0.10, compound_bias=-0.05, ...). Diagnostica: "perché Express è così alto?".

- **`undercurrents_current() -> Vec<(Intention, f64)>`**: le intenzioni non dominanti correnti. Vedere "cosa quasi decidi".

- **`spontaneous_threshold() -> f64`**: la soglia attuale per espressione autonoma. Capire quando l'entità potrebbe parlare da sola.

- **`pressure_trajectory(n) -> Vec<(tick, FieldPressures)>`**: storia delle 7 pressioni nel tempo. Visualizzare come la volontà oscilla.

---

## Sintesi del volume

Le 7 intenzioni di volontà (Express, Explore, Question, Remember, Withdraw, Reflect, Instruct) coesistono come pressioni continue in [0, 1]. Non si escludono: la dominante vince, ma le altre restano come **undercurrents**.

Phase 67 ha separato l'architettura in due layer:
- **`FieldPressures`** (grezzo): 7 numeri + codon + stato sogno. Calcolato da `WillCore::compute_pressures()` con 14 input (vital, campo, needs, dialogue, values, Octalysis drives, etc.).
- **`WillResult`** (con selezione): conversione via `to_will_result()` che sceglie la dominante.

Nel path principale `receive()`, `NarrativeSelf::deliberate()` riceve `FieldPressures` direttamente e sceglie con più contesto (valenza, coherence, interlocutor). Il `sense()` resta wrapper per backward compat.

Le modulazioni vengono applicate in ordine: **segnali grezzi → topic_continuity → compound_bias/provenance → value_weights → octalysis_drives → needs_modulation (prepotenza)**. L'ultimo è l'override più duro: bisogni battono tutto.

Gli **undercurrents** alimentano `DesireCore::register_undercurrent` — le intenzioni persistenti come sottocorrenti diventano desideri. L'ecologia mentale non è amnesica.

La **soglia di espressione spontanea** in `autonomous_tick` è dinamica (base 0.6, scende a 0.35 con bisogni/desideri forti). Regola l'equilibrio tra silenzio e auto-espressione.

Cinque endpoint admin proposti per esporre la dinamica volitiva.

Da qui Vol. 11 entra nella **relazione con l'Altro**: `InterlocutorModel` (l'eco del tu nel campo) e `HumorSense` (l'ironia come incongruenza topologica).

---

*Prossimo volume: 11 — Eco dell'Altro e Humor* (in scrittura)
