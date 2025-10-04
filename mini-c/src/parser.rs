// brings definition and functions from other types
use crate::token::Token;
use crate::ast::{Program, Function, Block, Stmt, Expr, Type};

// holds all tokens and pointer access
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

// new -> creates new parse
// look at the currennt token
// movee to the next token
// parse functions to programs
// parse one function
// parse the statements into one function
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current_token(&self) -> &Token {
        if self.position < self.tokens.len(){
            &self.tokens[self.position]
        }
        
        else{
            &Token::EOF
        }
    }


    fn advance(&mut self){
        if self.position<self.tokens.len(){
            self.position += 1;
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut functions = Vec::new();

        while *self.current_token() != Token::EOF {
            if let Some(func) = self.parse_function() {
                functions.push(func);
            }
            
            else{
                break;
            }
        }

        Program { functions }
    }

    fn parse_function(&mut self) -> Option<Function> {
        // Expect: <type> <ident>() { <body> }
        let return_type = match self.current_token() {
            Token::Int => { self.advance(); Type::Int }
            Token::Float => { self.advance(); Type::Float }
            Token::Char => { self.advance(); Type::Char }
            Token::Void => { self.advance(); Type::Void }
            _ => return None,
        };

        let name = if let Token::Ident(name) = self.current_token().clone() {
            self.advance();
            name
        } else {
            return None;
        };

        if *self.current_token() == Token::LParen {
            self.advance();
        }
        if *self.current_token() == Token::RParen {
            self.advance();
        }
        if *self.current_token() == Token::LBrace {
            self.advance();
        }

        let mut stmts = Vec::new();
        while *self.current_token() != Token::RBrace && *self.current_token() != Token::EOF {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                break;
            }
        }

        if *self.current_token() == Token::RBrace {
            self.advance();
        }

        Some(Function { 
            name, 
            return_type,
            params: vec![], 
            body: Block { stmts } 
        })
    }

    fn parse_statement(&mut self) -> Option<Stmt> {
        match self.current_token() {
            Token::Return => {
                self.advance();
                match self.current_token().clone() {
                    Token::Number(n) => {
                        self.advance();
                        if *self.current_token() == Token::Semicolon { self.advance(); }
                        return Some(Stmt::Return(Expr::Number(n)));
                    }
                    Token::FloatNumber(f) => {
                        self.advance();
                        if *self.current_token() == Token::Semicolon { self.advance(); }
                        return Some(Stmt::Return(Expr::FloatNumber(f)));
                    }
                    Token::CharLiteral(c) => {
                        self.advance();
                        if *self.current_token() == Token::Semicolon { self.advance(); }
                        return Some(Stmt::Return(Expr::CharLiteral(c)));
                    }
                    _ => {}
                }
            }
            Token::Int | Token::Float | Token::Char => {
                // Variable declaration: <type> name = value;
                let ty = match self.current_token() {
                    Token::Int => Type::Int,
                    Token::Float => Type::Float,
                    Token::Char => Type::Char,
                    _ => Type::Int,
                };
                self.advance();
                if let Token::Ident(name) = self.current_token().clone() {
                    self.advance();
                    if *self.current_token() == Token::Assign {
                        self.advance();
                        match self.current_token().clone() {
                            Token::Number(n) => {
                                self.advance();
                                if *self.current_token() == Token::Semicolon { self.advance(); }
                                return Some(Stmt::VarDecl { ty, name, value: Expr::Number(n) });
                            }
                            Token::FloatNumber(f) => {
                                self.advance();
                                if *self.current_token() == Token::Semicolon { self.advance(); }
                                return Some(Stmt::VarDecl { ty, name, value: Expr::FloatNumber(f) });
                            }
                            Token::CharLiteral(c) => {
                                self.advance();
                                if *self.current_token() == Token::Semicolon { self.advance(); }
                                return Some(Stmt::VarDecl { ty, name, value: Expr::CharLiteral(c) });
                            }
                            Token::Ident(var_name) => {
                                self.advance();
                                if *self.current_token() == Token::Semicolon { self.advance(); }
                                return Some(Stmt::VarDecl { ty, name, value: Expr::Ident(var_name.clone()) });
                            }
                            _ => {}
                        }
                    }
                }
            }
            Token::Ident(name) => {
                // Function call: name(...);
                let func_name = name.clone();
                self.advance();
                if *self.current_token() == Token::LParen {
                    self.advance();
                    let mut args = Vec::new();
                    
                    // Parse arguments
                    while *self.current_token() != Token::RParen && *self.current_token() != Token::EOF {
                        match self.current_token() {
                            Token::String(s) => {
                                args.push(Expr::StringLiteral(s.clone()));
                                self.advance();
                            }
                            Token::Ident(var_name) => {
                                args.push(Expr::Ident(var_name.clone()));
                                self.advance();
                            }
                            Token::Number(n) => {
                                args.push(Expr::Number(*n));
                                self.advance();
                            }
                            Token::FloatNumber(f) => {
                                args.push(Expr::FloatNumber(*f));
                                self.advance();
                            }
                            Token::CharLiteral(c) => {
                                args.push(Expr::CharLiteral(*c));
                                self.advance();
                            }
                            Token::Comma => {
                                self.advance(); // skip comma
                            }
                            _ => {
                                self.advance(); // skip unknown tokens
                            }
                        }
                    }
                    
                    if *self.current_token() == Token::RParen {
                        self.advance();
                    }
                    if *self.current_token() == Token::Semicolon {
                        self.advance();
                    }
                    return Some(Stmt::ExprStmt(Expr::Call { 
                        name: func_name, 
                        args 
                    }));
                }
            }
            _ => {}
        }
        None
    }
}
