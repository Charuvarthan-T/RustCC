// to define the vocabulary of language

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int,
    Float,
    Char,
    Void,
    Return,
    Ident(String),
    Number(i64),
    FloatNumber(f64),
    CharLiteral(char),
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
