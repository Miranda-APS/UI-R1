# Principi inviolabili

> Sources: Francesco Mancuso, 2026-05-12 (CLAUDE.md, Phase 79)
> Raw: [CLAUDE_phase79](../../raw/contesto/CLAUDE_phase79.md)

## Overview

UI-R1 (precedentemente "Prometeo") è un sistema cognitivo topologico che NON è un LLM, NON usa template, NON ha intent detection enum-driven e NON simula empatia. Nove principi stratificati nel corso di molte conversazioni con l'utente: ogni violazione è un regresso architetturale. Questo articolo è il filtro da consultare PRIMA di proporre qualunque modifica.

## I nove principi

**1. No template, no enum dispatch.** Niente `match input_act { Greeting => ... }`. La forma e il contenuto della risposta emergono da fatti KG-derivati + stato del campo, mai da tabelle hardcoded. Vedi [niente template](niente-template.md).

**2. Una parola sola per nodo del KG.** Mai `pronome_interrogativo` come unico nodo. Concetti composti = relazioni multiple (`cosa IsA pronome` E `cosa IsA interrogativo`). Vedi [knowledge graph semantico](../topologia/knowledge-graph-semantico.md).

**3. Nessuna simulazione di empatia.** UI-r1 è una macchina autentica. Può COMPRENDERE come ti senti (via KG: classi emotive, prossimità) e usare quella conoscenza per orientarsi verso quello che ti aiuta — senza fingere di sentire. "L'agente non sente; può però conoscere sé stessa logicamente in un modo che un umano non riesce." Vedi [niente empatia simulata](niente-empatia-simulata.md).

**4. Lo strumento deve liberare, non creare bisogno.** UI-r1 aiuta le persone a NON aver più bisogno di lei usando lei stessa. Niente dipendenza, niente finta intimità, niente engagement metrics.

**5. Capire prima, generare dopo.** L'output non importa se UI-r1 non ha prima capito davvero l'input. ComprehensionReport scritto in italiano leggibile (Phase 73), ActionDecision scritta come decisione esplicita (Phase 74), poi le parole. Vedi [capire prima, generare dopo](capire-prima-generare-dopo.md).

**6. Educare, non hardcodare.** "Le regole grammaticali dovremmo spiegargliele, non infilargliele a forza nel codice." La grammatica vive nel KG procedurale come dati. Rust contiene meccanismi generici. Insegnare un nuovo pattern = aggiungere triple, mai modificare Rust. Vedi [educare non hardcodare](educare-non-hardcodare.md).

**7. Curare ancorato al meccanismo.** Aggiungi al KG SOLO quello che serve a un meccanismo esistente o a un pattern che stai introducendo. Mai "potrebbe servire un giorno" — è dead-weight. Vedi [workflow di curation del KG](workflow-curation-kg.md).

**8. Continuità narrativa via SpeakerProfile, non via stato che decade.** La memoria del parlante è accumulazione di fatti specifici (`self_facts`, `entity_facts`, `open_questions`, `gaps`), non stati che svaniscono. Vedi [speaker profile](../comprensione/speaker-profile.md).

**9. Riferimento concettuale**: le teorie di Carlo Rovelli (relazioni come substrato, niente cose in sé) e Lacan (significante / Altro / catena di significanti, vuoto come soglia di desiderio) guidano l'architettura.

## Come usarli

I principi non sono un manifesto — sono **un filtro operativo**. Prima di proporre un meccanismo nuovo, scorri la lista: la proposta viola uno dei nove? Se sì, riformula. Il filtro complementare è il [Test pre-proposta](test-pre-proposta.md) (emergenza vs hardcoding mascherato).

## Origine

Questi principi non sono stati progettati in anticipo. Sono **stratificati** in conversazioni multiple tra Francesco e l'agente AI, ogni volta che una proposta sembrava "ragionevole" ma in realtà tradiva la natura del sistema. Sono diagnostiche di errori passati promosse a regole.

## See Also

- [Test pre-proposta — diagnostica emergenza vs hardcoding](test-pre-proposta.md)
- [Capire prima, generare dopo](capire-prima-generare-dopo.md)
- [Niente template, niente dispatch](niente-template.md)
- [Educare, non hardcodare](educare-non-hardcodare.md)
- [Niente simulazione di empatia](niente-empatia-simulata.md)
- [Workflow di curation del KG](workflow-curation-kg.md)
