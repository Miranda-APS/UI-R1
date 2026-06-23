# Disambiguazione per via-registro (omonimi/polisemia)

> Contratto condiviso kg_sem (curation) ↔ grounding (agente). 2026-06-09.
> Metodo: ogni parola con più sensi → **un arco-per-senso** nel kg_sem, ciascuno
> con un `via` che nomina il **registro** sotto cui quel senso vale. Il campo
> seleziona; nessuna lista di parole hardcoded in Rust.

## La regola (lato grounding, generica)
Quando un nodo ha archi concorrenti con `via`, l'esplorazione preferisce:
1. l'arco la cui **relazione combacia con la costruzione** della frase (via path (b)),
2. e/o l'arco il cui **`via` è co-attivo nel campo / tra le parole-contenuto** (via path (a)),
3. poi confidenza/distanza (fallback = BFS attuale).
Mai soglie, mai liste di parole: è forma (questo senso vale in questo registro),
la scelta emerge dal campo.

## Due percorsi di attivazione del registro
- **(b) costruzione → registro** — attivo ORA. La relazione/costruzione *è* il
  disambiguatore. Copre i sensi legati alla relazione (tipicamente vissuti).
- **(a) parola-contenuto → registro** — si attiva quando il campo è cablato nel
  grounding. Copre i sensi dove a disambiguare è il complemento/contesto, non la
  costruzione (entrambi i sensi spesso hanno la stessa relazione, es. IsA).

## Mappa relazione/costruzione → registro (per (b))
| costruzione / relazione        | registro   |
|--------------------------------|------------|
| `FeelsAs` / `sentirsi + agg`   | emozione   |
| `Expresses` / verbo cognitivo  | pensiero   |
| `Does` / verbo d'azione        | azione     |
| (content-word, vedi (a))       | —          |

## Lessico dei registri (i `via` usati finora)
| registro    | senso che seleziona                         | esempi di parole-contesto |
|-------------|---------------------------------------------|---------------------------|
| emozione    | vissuto/stato interiore                     | sentirsi, provare, cuore  |
| musica      | musicale                                    | suono, chitarra, nota     |
| restrizione | quantificatore/avverbio "soltanto"          | solo, appena              |
| progetto    | piano/intenzione/scopo                      | progetto, piano, scopo    |
| arredo      | mobile/oggetto domestico                    | mobile, cucina, casa      |
| fede        | credenza/convinzione                        | credere, fede, dio        |
| pianta      | botanico                                    | albero, terra, radici     |
| origine     | base astratta/fondamento/matematica         | inizio, fondamento, numero|
| luogo       | spazio fisico                               | terreno, prato, posto     |
| dominio     | campo-di-studio/insieme/sistema             | scienza, teoria, ambito   |

(Si estende quando serve — mai una lista chiusa.)

## Archi già taggati nel kg_sem
| parola    | arco                          | via         | percorso |
|-----------|-------------------------------|-------------|----------|
| solo      | FeelsAs solitudine            | emozione    | (b) ✓ live |
| solo      | SimilarTo assolo              | musica      | (a)      |
| solo      | SimilarTo soltanto            | restrizione | (a)      |
| piano     | IsA oggetto (pianoforte)      | musica      | (a)      |
| piano     | IsA intenzione                | progetto    | (a)      |
| credenza  | IsA struttura (mobile)        | arredo      | (a)      |
| credenza  | IsA concetto (fede)           | fede        | (a)      |
| radice    | IsA oggetto (botanica)        | pianta      | (a)      |
| radice    | IsA fondamento                | origine     | (a)      |
| campo     | IsA spazio                    | luogo       | (a)      |
| campo     | IsA sistema                   | dominio     | (a)      |

## Nota di disciplina
- NON si aggiungono sensi che non compaiono nell'input reale (principio 7: niente
  dead-weight). Si taggano solo gli archi-senso **già presenti** e conflati.
- Il KG attuale è in gran parte mono-senso: gli omonimi conflati sono pochi. Questo
  non è un lavoro di massa — è curation mirata sui nodi davvero ambigui.
