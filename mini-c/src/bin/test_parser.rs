use std::fs;
use mini_c::lexer::Lexer;
use mini_c::parser::Parser;
use mini_c::token::Token;

fn main() {
    let source = fs::read_to_string("examples/example1.c").unwrap();
    
    // First, tokenize the input
    let mut lexer = Lexer::new(&source);
    let mut tokens = Vec::new();
    
    // Collect all tokens
    loop {
        let token = lexer.next_token();
        if matches!(token, Token::EOF) {
            tokens.push(token);
            break;
        }
        tokens.push(token);
    }
    
    // Parse the tokens into an AST
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();
    
    // Print the resulting AST with nice formatting
    println!("Parser Output (AST) for examples/example1.c:");
    println!("-------------------------------------------");
    println!("{:#?}", program);
    
    // Print a simplified summary of the parsed program
    println!("\nProgram Summary:");
    println!("--------------");
    
    for function in &program.functions {
        println!("Function: {}", function.name);
    let param_names: Vec<String> = function.params.iter().map(|(_, name)| name.clone()).collect();
    println!("  Parameters: [{}]", param_names.join(", "));
        println!("  Body contains {} statements", function.body.stmts.len());
        
        // Print a condensed view of each statement
        for (i, stmt) in function.body.stmts.iter().enumerate() {
            match stmt {
                mini_c::ast::Stmt::VarDecl { name, .. } => {
                    println!("    Stmt {}: Variable Declaration: {} = ...", i+1, name);
                },
                mini_c::ast::Stmt::ExprStmt(expr) => {
                    match expr {
                        mini_c::ast::Expr::Call { name, .. } => {
                            println!("    Stmt {}: Function Call: {}(...)", i+1, name);
                        },
                        _ => println!("    Stmt {}: Expression Statement", i+1),
                    }
                },
                mini_c::ast::Stmt::Return(..) => {
                    println!("    Stmt {}: Return Statement", i+1);
                },
            }
        }
        println!("");
    }
}
