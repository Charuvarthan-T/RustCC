use crate::ast::*;
use std::fmt;

#[derive(Debug, Clone)]
pub enum SemanticError {
    DuplicateFunction { name: String },
    DuplicateParam { func: String, name: String },
    DuplicateVariable { func: String, name: String },
    UndeclaredVariable { func: String, name: String },
    WrongArgCount { func: String, name: String, expected: usize, found: usize },
    // future: add TypeMismatch, ReturnMissing, etc.
}

pub type SemResult<T> = Result<T, Vec<SemanticError>>;

pub struct SymbolTable {
    pub functions: Vec<FunctionSig>,
}

#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub params: Vec<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable { functions: Vec::new() }
    }

    pub fn add_function(&mut self, fn_sig: FunctionSig) -> Result<(), SemanticError> {
        if self.functions.iter().any(|f| f.name == fn_sig.name) {
            return Err(SemanticError::DuplicateFunction { name: fn_sig.name });
        }
        self.functions.push(fn_sig);
        Ok(())
    }

    pub fn find_function(&self, name: &str) -> Option<FunctionSig> {
        self.functions.iter().find(|f| f.name == name).cloned()
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::DuplicateFunction { name } => write!(f, "Duplicate function '{}'.", name),
            SemanticError::DuplicateParam { func, name } => write!(f, "Duplicate parameter '{}' in function '{}'.", name, func),
            SemanticError::DuplicateVariable { func, name } => write!(f, "Duplicate variable '{}' in function '{}'.", name, func),
            SemanticError::UndeclaredVariable { func, name } => write!(f, "Undeclared variable '{}' in function '{}'.", name, func),
            SemanticError::WrongArgCount { func, name, expected, found } => write!(f, "Wrong argument count for call to '{}' in function '{}': expected {}, found {}.", name, func, expected, found),
        }
    }
}

// Perform a basic semantic analysis pass.
// - Collect functions and ensure no duplicates
// - For each function, check variable declarations and uses
// - Check call argument counts against declared params (for known functions)
pub fn analyze(program: &Program) -> SemResult<()> {
    let mut errors: Vec<SemanticError> = Vec::new();
    let mut symbols = SymbolTable::new();

    // collect function signatures and check duplicate params
    for func in &program.functions {
        // check duplicate params within the function
        for i in 0..func.params.len() {
            for j in (i + 1)..func.params.len() {
                if func.params[i] == func.params[j] {
                    errors.push(SemanticError::DuplicateParam { func: func.name.clone(), name: func.params[i].clone() });
                }
            }
        }

        let sig = FunctionSig {
            name: func.name.clone(),
            params: func.params.clone(),
        };
        if let Err(e) = symbols.add_function(sig) {
            errors.push(e);
        }
    }

    // analyze each function body
    for func in &program.functions {
        let mut locals: Vec<String> = Vec::new();
        // params are considered declared
        for p in &func.params {
            locals.push(p.clone());
        }

        for stmt in &func.body.stmts {
            analyze_stmt(stmt, &mut locals, &symbols, &mut errors, &func.name);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn analyze_stmt(stmt: &Stmt, locals: &mut Vec<String>, symbols: &SymbolTable, errors: &mut Vec<SemanticError>, func_name: &str) {
    match stmt {
        Stmt::VarDecl { name, value } => {
            if locals.iter().any(|n| n == name) {
                errors.push(SemanticError::DuplicateVariable { func: func_name.to_string(), name: name.clone() });
            } else {
                analyze_expr(value, locals, symbols, errors, func_name);
                locals.push(name.clone());
            }
        }
        Stmt::ExprStmt(expr) => analyze_expr(expr, locals, symbols, errors, func_name),
        Stmt::Return(expr) => analyze_expr(expr, locals, symbols, errors, func_name),
    }
}

fn analyze_expr(expr: &Expr, locals: &Vec<String>, symbols: &SymbolTable, errors: &mut Vec<SemanticError>, func_name: &str) {
    match expr {
        Expr::Number(_) => {}
        Expr::StringLiteral(_) => {}
        Expr::Ident(name) => {
            if !locals.iter().any(|n| n == name) {
                errors.push(SemanticError::UndeclaredVariable { func: func_name.to_string(), name: name.clone() });
            }
        }
        Expr::Unary { op: _, expr } => analyze_expr(expr, locals, symbols, errors, func_name),
        Expr::Binary { op: _, left, right } => {
            analyze_expr(left, locals, symbols, errors, func_name);
            analyze_expr(right, locals, symbols, errors, func_name);
        }
        Expr::Assign { name, value } => {
            if !locals.iter().any(|n| n == name) {
                errors.push(SemanticError::UndeclaredVariable { func: func_name.to_string(), name: name.clone() });
            }
            analyze_expr(value, locals, symbols, errors, func_name);
        }
        Expr::Call { name, args } => {
            // analyze args
            for a in args {
                analyze_expr(a, locals, symbols, errors, func_name);
            }
            // check arity if function known
            if let Some(sig) = symbols.find_function(name) {
                if sig.params.len() != args.len() {
                    errors.push(SemanticError::WrongArgCount { func: func_name.to_string(), name: name.clone(), expected: sig.params.len(), found: args.len() });
                }
            }
            // else: calling external function (like printf) is allowed
        }
    }
}
