// ast.rs
// AST definitions for mini-c

#![allow(dead_code)]
// Abstract Syntax Tree (AST) definitions

// works only on the assumption that all the datatypes are "int"

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Char,
    Void,
}


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

// defining separate separate items make things go smoothl
#[derive(Debug, Clone)]
pub enum UnaryOp { Neg, Not }

#[derive(Debug, Clone)]
pub enum BinaryOp { Add, Sub, Mul, Div }

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl { ty: Type, name: String, value: Expr },
    ExprStmt(Expr),
    Return(Expr),
    // minimal subset for now; add If/While later
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: Type,
    pub params: Vec<(Type, String)>,  // param type and name
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}


