// import other files as modules here
mod token;
mod lexer;
mod parser;
mod ast;
mod semantic;

// imports as in python
// 1. access CLI
// 2. access file system
// 3, 4. imports structs from respective files
use std::env;
use std::fs;
use lexer::Lexer;
use parser::Parser;


fn main() {

    // iterates over the CLI and stores it as a array of strings
    let args: Vec<String> = env::args().collect();

    // if the command does not have a File name
    if args.len() < 2 {
        eprintln!("Usage: mini-c <filename>");
        return;
    }

    
    let filename = &args[1];
    let input = fs::read_to_string(filename).expect("Couldnt read the file");

    // create instances of structures
    let mut lexer = Lexer::new(&input);
    let mut tokens = Vec::new();

    // extract tokens from the input until u get a EOF
    loop {
        let tok = lexer.next_token();
        if tok == token::Token::EOF {
            break;
        }
        tokens.push(tok);
    }

    
    // create a parse and call the AST
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program();

    // Run semantic analysis
    match semantic::analyze(&ast) {
        Ok(()) => {
            // no semantic errors
        }
        Err(errs) => {
            eprintln!("Semantic errors found:");
            for e in errs {
                eprintln!("{:?}", e);
            }
            // still print AST for debugging
        }
    }

    // debug formatter -> :?
    // pretty print -> #
    println!("{:#?}", ast);
}
