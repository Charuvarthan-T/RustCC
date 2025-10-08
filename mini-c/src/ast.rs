// allows unused code during development
#![allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]


// basic types
pub enum Type {
    Int,
    Float,
    Char,
    Void,
}


// expressions
#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    FloatNumber(f64),
    CharLiteral(char),
    StringLiteral(String),
    Ident(String),
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr> },
    Assign { name: String, value: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
}


// unary operators
#[derive(Debug, Clone)]
pub enum UnaryOp { Neg, Not }


// binary operators
#[derive(Debug, Clone)]
pub enum BinaryOp { Add, Sub, Mul, Div }


// statements
#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl { ty: Type, name: String, value: Expr },
    ExprStmt(Expr),
    Return(Expr),
    // minimal subset for now; add If/While later
}


// a block of statements -> this is for inside functions
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}


// a function with name, return type, parameters, and body
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: Type,
    pub params: Vec<(Type, String)>,  // param type and name
    pub body: Block,
}



// the whole program with multiple functions
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}


