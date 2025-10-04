use std::fs;
use mini_c::lexer::Lexer;
use mini_c::parser::Parser;
use mini_c::semantic;

fn parse_file(path: &str) -> mini_c::ast::Program {
    let input = fs::read_to_string(path).expect("Could not read file");
    let mut lexer = Lexer::new(&input);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        if tok == mini_c::token::Token::EOF {
            break;
        }
        tokens.push(tok);
    }
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[test]
fn example_should_pass_semantic() {
    let prog = parse_file("examples/example1.c");
    let res = semantic::analyze(&prog);
    assert!(res.is_ok());
}

#[test]
fn bad_should_fail_semantic() {
    let prog = parse_file("examples/bad.c");
    let res = semantic::analyze(&prog);
    assert!(res.is_err());
}
