// to define the vocabulary of language

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int,
    Return,
    Ident(String),
    Number(i64),
    String(String),
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Assign,
    Comma,
    EOF,
}
