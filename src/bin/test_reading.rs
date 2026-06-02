use prometeo::topology::input_reading::{read_input, InputAct};
use prometeo::topology::lexicon::Lexicon;
use prometeo::topology::fractal::FractalId;
use prometeo::topology::knowledge::{KnowledgeBase, KnowledgeDomain};

fn kb_with_anchors() -> KnowledgeBase {
    let mut kb = KnowledgeBase::new();
    kb.teach_concept(
        KnowledgeDomain::Social,
        "un saluto è un atto di avvicinamento sociale: chi saluta vuole connessione",
        "ciao",
        vec![63, 47],
    );
    kb.teach_concept(
        KnowledgeDomain::Emotional,
        "un'emozione è uno stato interno vissuto: chi esprime un'emozione cerca risonanza",
        "paura",
        vec![58, 33],
    );
    kb.teach_concept(
        KnowledgeDomain::Self_,
        "un'indagine sull'identità è la domanda su chi è Prometeo, cosa sente, cosa vuole",
        "sei",
        vec![32],
    );
    kb
}

fn main() {
    let lex = Lexicon::bootstrap();
    let kb = kb_with_anchors();

    // Delta frattali simulati (come se l'input avesse causato questi cambiamenti)
    let emozione_delta: Vec<(FractalId, f64)> = vec![(58, 0.50), (33, 0.30)];
    let identita_delta: Vec<(FractalId, f64)> = vec![(32, 0.40), (47, 0.20)];
    let armonia_delta: Vec<(FractalId, f64)>  = vec![(63, 0.60), (59, 0.30)];
    let empty: Vec<(FractalId, f64)> = vec![];

    let cases: Vec<(&str, &Vec<(FractalId, f64)>, &str)> = vec![
        ("ciao",                &empty,          "Greeting"),      // word_match "ciao"
        ("ciao come stai",      &armonia_delta,  "Greeting"),      // word_match + delta
        ("salve",               &armonia_delta,  "Greeting"),      // delta ARMONIA
        ("ho paura",            &empty,          "EmotionalExpr"), // word_match "paura"
        ("mi sento triste",     &emozione_delta, "EmotionalExpr"), // delta EMOZIONE
        ("chi sei?",            &empty,          "SelfQuery"),     // word_match "sei" + ?
        ("cosa sei tu?",        &empty,          "SelfQuery"),     // word_match "sei" + ?
        ("cosa pensi?",         &identita_delta, "SelfQuery"),     // delta IDENTITA + ?
        ("come stai?",          &empty,          "Question"),      // solo ? senza Self_
        ("cosa succede?",       &empty,          "Question"),
        ("sei felice?",         &empty,          "Question"),      // "sei" = trigger Self_ + ?
        ("la luce è bella",     &empty,          "Declaration"),
        ("ti voglio bene",      &empty,          "Declaration"),
        ("dimmi qualcosa",      &empty,          "Declaration"),
    ];

    println!("{:<28} {:<25} {:<18} {:}", "INPUT", "DELTA DOMINANTE", "ACT RILEVATO", "CORRETTO?");
    println!("{}", "─".repeat(90));

    let mut ok = 0;
    let mut tot = 0;
    for (text, delta, expected) in &cases {
        let words: Vec<String> = text.split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();

        let r = read_input(&words, text, delta, &kb, &lex, None, None);

        let campo = if delta.is_empty() { "vuoto".to_string() }
            else { format!("frac[{}]={:.2}", delta[0].0, delta[0].1) };

        let act_str = format!("{:?}", r.act);
        let check = if act_str == *expected {
            ok += 1; "✓".to_string()
        } else {
            format!("✗ (atteso: {})", expected)
        };
        tot += 1;

        println!("{:<28} {:<25} {:<18} {}",
            format!("\"{}\"", text), campo, act_str, check);
    }

    println!("{}", "─".repeat(90));
    println!("Risultato: {}/{} corretti", ok, tot);
}
