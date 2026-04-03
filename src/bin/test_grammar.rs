use prometeo::topology::grammar::{detect_gender_number, inflect_adjective, with_definite_article, with_articulated_preposition};
use prometeo::topology::grammar::{Gender, Number};

fn main() {
    println!("=== TEST GRAMMATICA ===");
    
    let words = vec![
        "tavolo", "sedia", "amici", "idee",
        "cane", "gatto", "problema", "soluzione"
    ];
    
    for w in &words {
        let (g, n) = detect_gender_number(w);
        println!("Parola: {:<10} -> Genere: {:?}, Numero: {:?}", w, g, n);
        
        let art = with_definite_article(w);
        println!("  Articolo det: {}", art);
        
        let prep_di = with_articulated_preposition("di", w);
        let prep_a = with_articulated_preposition("a", w);
        println!("  Prep 'di': {}", prep_di);
        println!("  Prep 'a':  {}", prep_a);
        
        let adj_bello = inflect_adjective("bello", g, n);
        let adj_forte = inflect_adjective("forte", g, n);
        println!("  Aggettivi: {} {}", w, adj_bello);
        println!("  Aggettivi: {} {}", w, adj_forte);
        println!();
    }
}
