use crate::ast::*;

#[derive(Debug)]
pub enum SemanticError {
    DuplicateFunction(String),
    DuplicateVariable(String),
    UndeclaredVariable(String),
    WrongArgCount { name: String, expected: usize, found: usize },
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
            return Err(SemanticError::DuplicateFunction(fn_sig.name));
        }
        self.functions.push(fn_sig);
        Ok(())
    }

    pub fn find_function(&self, name: &str) -> Option<FunctionSig> {
        self.functions.iter().find(|f| f.name == name).cloned()
    }
}

// Perform a basic semantic analysis pass.
// - Collect functions and ensure no duplicates
// - For each function, check variable declarations and uses
// - Check call argument counts against declared params (for known functions)
pub fn analyze(program: &Program) -> SemResult<()> {
    let mut errors: Vec<SemanticError> = Vec::new();
    let mut symbols = SymbolTable::new();

    // collect function signatures
    for func in &program.functions {
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
            analyze_stmt(stmt, &mut locals, &symbols, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn analyze_stmt(stmt: &Stmt, locals: &mut Vec<String>, symbols: &SymbolTable, errors: &mut Vec<SemanticError>) {
    match stmt {
        Stmt::VarDecl { name, value } => {
            if locals.iter().any(|n| n == name) {
                errors.push(SemanticError::DuplicateVariable(name.clone()));
            } else {
                analyze_expr(value, locals, symbols, errors);
                locals.push(name.clone());
            }
        }
        Stmt::ExprStmt(expr) => analyze_expr(expr, locals, symbols, errors),
        Stmt::Return(expr) => analyze_expr(expr, locals, symbols, errors),
    }
}

fn analyze_expr(expr: &Expr, locals: &Vec<String>, symbols: &SymbolTable, errors: &mut Vec<SemanticError>) {
    match expr {
        Expr::Number(_) => {}
        Expr::StringLiteral(_) => {}
        Expr::Ident(name) => {
            if !locals.iter().any(|n| n == name) {
                errors.push(SemanticError::UndeclaredVariable(name.clone()));
            }
        }
        Expr::Unary { op: _, expr } => analyze_expr(expr, locals, symbols, errors),
        Expr::Binary { op: _, left, right } => {
            analyze_expr(left, locals, symbols, errors);
            analyze_expr(right, locals, symbols, errors);
        }
        Expr::Assign { name, value } => {
            if !locals.iter().any(|n| n == name) {
                errors.push(SemanticError::UndeclaredVariable(name.clone()));
            }
            analyze_expr(value, locals, symbols, errors);
        }
        Expr::Call { name, args } => {
            // analyze args
            for a in args {
                analyze_expr(a, locals, symbols, errors);
            }
            // check arity if function known
            if let Some(sig) = symbols.find_function(name) {
                if sig.params.len() != args.len() {
                    errors.push(SemanticError::WrongArgCount { name: name.clone(), expected: sig.params.len(), found: args.len() });
                }
            }
            // else: calling external function (like printf) is allowed
        }
    }
}
