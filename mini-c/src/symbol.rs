use std::collections::HashMap;
use crate::ast::Type;

#[allow(dead_code)]
#[derive(Debug, Clone)]


// a symbol in the symbol table
pub enum Symbol {
    Function(FunctionSig),
    Variable { name: String, ty: Type },
    Param { name: String, ty: Type },
}

#[derive(Debug, Clone)]


// a function signature
pub struct FunctionSig {
    pub name: String,
    pub return_type: Type,
    pub params_types: Vec<Type>,
}



//  a symbol table with nested scopes
struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>,
}


// a symbol table with nested scopes
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current: usize,
}


// helper methods for Scope and SymbolTable
impl Scope {
    fn new(parent: Option<usize>) -> Self {
        Scope { symbols: HashMap::new(), parent }
    }
}



// helper methods for SymbolTable
impl SymbolTable {

    // create a new symbol table with global scope
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(Scope::new(None)); // global scope index 0
        SymbolTable { scopes, current: 0 }
    }


    // enter a new nested scope
    pub fn enter_scope(&mut self) {
        let parent = Some(self.current);
        let id = self.scopes.len();
        self.scopes.push(Scope::new(parent));
        self.current = id;
    }

    // leave the current scope and return to parent
    pub fn leave_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        }
    }


    // declare a global function in the global scope
    pub fn declare_global_function(&mut self, sig: FunctionSig) -> Result<(), String> {
        let name = sig.name.clone();
        if self.scopes[0].symbols.contains_key(&name) {
            return Err(format!("duplicate function: {}", name));
        }
        self.scopes[0].symbols.insert(name.clone(), Symbol::Function(sig));
        Ok(())
    }


    // declare a local variable in the current scope
    pub fn declare_local_var(&mut self, name: &str, ty: Type) -> Result<(), String> {
        let scope = &mut self.scopes[self.current];
        if scope.symbols.contains_key(name) {
            return Err(format!("duplicate local: {}", name));
        }
        scope.symbols.insert(name.to_string(), Symbol::Variable { name: name.to_string(), ty });
        Ok(())
    }


    // declare a parameter in the current scope
    pub fn declare_param(&mut self, name: &str, ty: Type) -> Result<(), String> {
        let scope = &mut self.scopes[self.current];
        if scope.symbols.contains_key(name) {
            return Err(format!("duplicate param: {}", name));
        }
        scope.symbols.insert(name.to_string(), Symbol::Param { name: name.to_string(), ty });
        Ok(())
    }

    // lookup a symbol by name, searching from current scope up to global
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut scope_idx = self.current;
        loop {
            if let Some(sym) = self.scopes[scope_idx].symbols.get(name) {
                return Some(sym);
            }
            if let Some(parent) = self.scopes[scope_idx].parent {
                scope_idx = parent;
            } else {
                break;
            }
        }
        None
    }


    // lookup a global function by name
    pub fn find_global_function(&self, name: &str) -> Option<FunctionSig> {
        if let Some(sym) = self.scopes[0].symbols.get(name) {
            if let Symbol::Function(sig) = sym {
                return Some(sig.clone());
            }
        }
        None
    }
}
