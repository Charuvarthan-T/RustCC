use std::fs;
use mini_c::lexer::Lexer;
use mini_c::token::Token;

fn main() {
    let source = fs::read_to_string("examples/example1.c").unwrap();
    let mut lexer = Lexer::new(&source);

    println!("Lexer Output for examples/example1.c:");
    println!("------------------------------------");
    
    // Collect tokens in a more compact format
    let mut tokens = Vec::new();
    
    loop {
        let token = lexer.next_token();
        
        // Format the token in a more compact way
        match &token {
            Token::EOF => {
                tokens.push("EOF".to_string());
                break;
            },
            Token::Ident(name) => tokens.push(format!("IDENT({})", name)),
            Token::Number(val) => tokens.push(format!("NUM({})", val)),
            Token::String(s) => tokens.push(format!("STR(\"{}\")", s)),
            _ => tokens.push(format!("{:?}", token)),
        }
    }
    
    // Print tokens in a more compact format, multiple per line
    const TOKENS_PER_LINE: usize = 4;
    for chunk in tokens.chunks(TOKENS_PER_LINE) {
        println!("{}", chunk.join(" | "));
    }
}