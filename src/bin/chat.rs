use prometeo::topology::engine::PrometeoTopologyEngine;
use prometeo::topology::persistence::PrometeoState;
use std::path::Path;

fn main() {
    println!("=== PROMETEO SIMULATED CHAT ===");
    let state_path = Path::new("prometeo_topology_state.bin");
    
    let mut engine = if state_path.exists() {
        println!("Caricamento stato...");
        let state = PrometeoState::load_from_binary(state_path).unwrap();
        let mut eng = PrometeoTopologyEngine::new();
        state.restore_lexicon(&mut eng);
        println!("Stato caricato: {} parole.", eng.lexicon.word_count());
        eng
    } else {
        println!("Stato non trovato, avvio a vuoto.");
        PrometeoTopologyEngine::new()
    };

    print!("Equilibrazione del campo (15 tick)... ");
    for _ in 0..15 {
        engine.autonomous_tick();
    }
    println!("Fatto.");

    let inputs = vec![
        "ciao prometeo",
        "cosa pensi dell'intelligenza?",
        "che cos'è la verità?",
        "parlami di un'idea complessa",
        "come ti senti oggi?"
    ];

    for input in inputs {
        println!("\nTu: {}", input);
        
        engine.receive(input);
        
        // Fai qualche tick per elaborare
        for _ in 0..5 {
            engine.autonomous_tick();
        }
        
        let result = engine.generate_willed();
        println!("Prometeo: {}", result.text);
    }
    println!("\nTest completato.");
}
