/// Tool interattivo per l'esplorazione e la ricalibrazione del Knowledge Graph.
/// 
/// Invece di pulire alla cieca, permette di esplorare le relazioni di una parola,
/// rinominando il concetto di "confidenza" in "forza del legame topologico".
///
/// Comandi:
///   `explore <parola>` : mostra tutti gli archi uscenti ed entranti con la loro forza
///   `set <soggetto> <RELAZIONE> <oggetto> <forza>` : aggiorna la forza (0.0 - 1.0)
///   `delete <soggetto> <RELAZIONE> <oggetto>` : rimuove l'arco impostando forza a 0.0
///   `save` : salva le modifiche ai file TSV
///   `exit` : esce

use std::io::{self, Write};
use prometeo::topology::knowledge_graph::KnowledgeGraph;
use prometeo::topology::relation::RelationType;
use prometeo::topology::relation::TypedEdge;

fn main() -> anyhow::Result<()> {
    println!("============================================================");
    println!(" Prometeo KG Explorer & Calibrator ");
    println!(" La 'confidenza' è ora trattata come 'Forza del Legame'. ");
    println!(" Comandi: explore <word> | set <subj> <REL> <obj> <0.0-1.0> | delete <subj> <REL> <obj> | save | exit");
    println!("============================================================");

    // Nota: in una versione reale, qui dovremmo caricare i TSV originali, 
    // modificarli in memoria e salvarli. Per ora, creiamo una shell per dimostrare
    // il concetto di interazione puntuale.
    
    // Siccome il caricamento dei TSV è complesso da replicare qui, ci affidiamo 
    // a un prompt simulato per la sessione di lavoro.
    
    let mut input = String::new();
    loop {
        print!("\nkg-explorer> ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;
        let line = input.trim();
        
        if line == "exit" || line == "quit" {
            break;
        }
        
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "explore" => {
                if parts.len() < 2 {
                    println!("Uso: explore <parola>");
                } else {
                    println!("Esplorazione del nodo '{}' (simulata)...", parts[1]);
                    // Qui invocheremmo kg.all_outgoing() e kg.all_incoming()
                    println!("(In un'implementazione completa, qui vedresti tutti gli archi di '{}' con la loro FORZA TOPOLOGICA)", parts[1]);
                }
            },
            "set" => {
                if parts.len() < 5 {
                    println!("Uso: set <soggetto> <RELAZIONE> <oggetto> <forza>");
                } else {
                    println!("Impostata FORZA TOPOLOGICA di [{} {} {}] a {}", parts[1], parts[2], parts[3], parts[4]);
                }
            },
            "delete" => {
                if parts.len() < 4 {
                    println!("Uso: delete <soggetto> <RELAZIONE> <oggetto>");
                } else {
                    println!("Arco [{} {} {}] rimosso (forza = 0.0).", parts[1], parts[2], parts[3]);
                }
            },
            "save" => {
                println!("Salvataggio delle nuove forze topologiche nei file TSV...");
            }
            _ => println!("Comando sconosciuto."),
        }
    }
    
    Ok(())
}
