// A simple semantic analysis pass for Mini-C.
use crate::ast::*;
use crate::symbol::{SymbolTable, FunctionSig};
use std::fmt;



// An enumeration of possible semantic errors.
#[derive(Debug, Clone)]
pub enum SemanticError {
    DuplicateFunction { name: String },
    DuplicateParam { func: String, name: String },
    DuplicateVariable { func: String, name: String },
    UndeclaredVariable { func: String, name: String },
    WrongArgCount { func: String, name: String, expected: usize, found: usize },
    TypeMismatch { func: String, expected: Type, found: Type },
    ReturnTypeMismatch { func: String, expected: Type, found: Type },
    // future: add TypeMismatch, ReturnMissing, etc.
}


// A result type for semantic analysis, accumulating multiple errors.
pub type SemResult<T> = Result<T, Vec<SemanticError>>;


// reuse symbol::SymbolTable and FunctionSig
impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::DuplicateFunction { name } => write!(f, "Duplicate function '{}'.", name),
            SemanticError::DuplicateParam { func, name } => write!(f, "Duplicate parameter '{}' in function '{}'.", name, func),
            SemanticError::DuplicateVariable { func, name } => write!(f, "Duplicate variable '{}' in function '{}'.", name, func),
            SemanticError::UndeclaredVariable { func, name } => write!(f, "Undeclared variable '{}' in function '{}'.", name, func),
            SemanticError::WrongArgCount { func, name, expected, found } => write!(f, "Wrong argument count for call to '{}' in function '{}': expected {}, found {}.", name, func, expected, found),
            SemanticError::TypeMismatch { func, expected, found } => write!(f, "Type mismatch in function '{}': expected {:?}, found {:?}.", func, expected, found),
            SemanticError::ReturnTypeMismatch { func, expected, found } => write!(f, "Return type mismatch in function '{}': expected {:?}, found {:?}.", func, expected, found),
        }
    }
}



// Analyze the program for semantic errors
pub fn analyze(program: &Program) -> SemResult<()> {
    let mut errors: Vec<SemanticError> = Vec::new();
    let mut symbols = SymbolTable::new();

    // collect function signatures and check duplicate params
    for func in &program.functions {
        // check duplicate params within the function -> just compare parameter names
        for i in 0..func.params.len() {
            for j in (i + 1)..func.params.len() {
                if func.params[i].1 == func.params[j].1 {
                    errors.push(SemanticError::DuplicateParam { func: func.name.clone(), name: func.params[i].1.clone() });
                }
            }
        }


        // declare function in global scope
        let sig = FunctionSig {
            name: func.name.clone(),
            return_type: func.return_type.clone(),
            params_types: func.params.iter().map(|(t, _)| t.clone()).collect(),
        };


        // insert into symbol table, check duplicate function
        if let Err(_e) = symbols.declare_global_function(sig.clone()) {
            errors.push(SemanticError::DuplicateFunction { name: func.name.clone() });
        }
    }

    // analyze each function body using proper scopes
    for func in &program.functions {
        symbols.enter_scope();
        // declare params in the new function scope
        for (t, pname) in &func.params {
            if let Err(_) = symbols.declare_param(pname, t.clone()) {
                errors.push(SemanticError::DuplicateParam { func: func.name.clone(), name: pname.clone() });
            }
        }

        // walk statements and use symbol table for locals
        for stmt in &func.body.stmts {
            analyze_stmt(stmt, &mut symbols, &mut errors, &func.name);
        }

        symbols.leave_scope();
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn analyze_stmt(stmt: &Stmt, symbols: &mut SymbolTable, errors: &mut Vec<SemanticError>, func_name: &str) {
    match stmt {
        Stmt::VarDecl { ty, name, value } => {
            // check duplicate in current scope
            if let Err(_) = symbols.declare_local_var(name, ty.clone()) {
                errors.push(SemanticError::DuplicateVariable { func: func_name.to_string(), name: name.clone() });
            } else {
                analyze_expr(value, symbols, errors, func_name);
                // type check initializer
                if let Some(vt) = expr_type(value, symbols) {
                    if vt != *ty {
                        errors.push(SemanticError::TypeMismatch { func: func_name.to_string(), expected: ty.clone(), found: vt });
                    }
                }
            }
        }
        Stmt::ExprStmt(expr) => analyze_expr(expr, symbols, errors, func_name),
        Stmt::Return(expr) => {
            analyze_expr(expr, symbols, errors, func_name);
            // check return type against function signature
            if let Some(sig) = symbols.find_global_function(func_name) {
                if let Some(rt) = expr_type(expr, symbols) {
                    if rt != sig.return_type {
                        errors.push(SemanticError::ReturnTypeMismatch { func: func_name.to_string(), expected: sig.return_type.clone(), found: rt });
                    }
                }
            }
        }
    }
}



// Analyze an expression for semantic errors.
fn analyze_expr(expr: &Expr, symbols: &SymbolTable, errors: &mut Vec<SemanticError>, func_name: &str) {
    match expr {
    Expr::Number(_) => {}
    Expr::FloatNumber(_) => {}
    Expr::CharLiteral(_) => {}
    Expr::StringLiteral(_) => {}


        // identifier: check declared
        Expr::Ident(name) => {
            if symbols.lookup(name).is_none() {
                errors.push(SemanticError::UndeclaredVariable { func: func_name.to_string(), name: name.clone() });
            }
        }


        // unary operation: analyze sub-expression
        Expr::Unary { op: _, expr } => analyze_expr(expr, symbols, errors, func_name),
        Expr::Binary { op: _, left, right } => {
            analyze_expr(left, symbols, errors, func_name);
            analyze_expr(right, symbols, errors, func_name);
        }



        Expr::Assign { name, value } => {
            // check variable declared
            if symbols.lookup(name).is_none() {
                errors.push(SemanticError::UndeclaredVariable { func: func_name.to_string(), name: name.clone() });
            }
            analyze_expr(value, symbols, errors, func_name);
        }
    Expr::Call { name, args: _args } => {
            // analyze args
            for a in _args {
                analyze_expr(a, symbols, errors, func_name);
            }
            // check arity if function known
            if let Some(sig) = symbols.find_global_function(name) {
                if sig.params_types.len() != 0 && sig.params_types.len() != _args.len() {
                    errors.push(SemanticError::WrongArgCount { func: func_name.to_string(), name: name.clone(), expected: sig.params_types.len(), found: _args.len() });
                }
            }
        }
    }
}



// Determine the type of an expression where possible. Returns None for unknown
fn expr_type(expr: &Expr, symbols: &SymbolTable) -> Option<Type> {
    // Determine the type of an expression where possible. Returns None for unknown
    match expr {
        Expr::Number(_) => Some(Type::Int),
        Expr::FloatNumber(_) => Some(Type::Float),
        Expr::CharLiteral(_) => Some(Type::Char),
        Expr::StringLiteral(_) => None,
        Expr::Ident(name) => {
            if let Some(sym) = symbols.lookup(name) {
                match sym {
                    crate::symbol::Symbol::Variable { name: _, ty } => Some(ty.clone()),
                    crate::symbol::Symbol::Param { name: _, ty } => Some(ty.clone()),
                    crate::symbol::Symbol::Function(_) => None,
                }
            } else {
                None
            }
        }


        // type is type of sub-expression
        Expr::Unary { .. } => None,
        Expr::Binary { left, right, .. } => {
            let l = expr_type(left, symbols);
            let r = expr_type(right, symbols);
            if l == r { l } else { None }
        }


        // type is variable's type if known
        Expr::Assign { name, value } => {
            if let Some(sym) = symbols.lookup(name) {
                if let crate::symbol::Symbol::Variable { name: _, ty } = sym {
                    return Some(ty.clone());
                }
            }
            expr_type(value, symbols)
        }



        // type is return type of function if known
        Expr::Call { name, .. } => {
            if let Some(sig) = symbols.find_global_function(name) {
                Some(sig.return_type.clone())
            } else {
                None
            }
        }
    }
}
