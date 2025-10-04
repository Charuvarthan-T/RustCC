use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Symbol {
    Function(FunctionSig),
    Variable { name: String },
    Param { name: String },
}

#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub params: Vec<String>,
}

struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>,
}

pub struct SymbolTable {
    scopes: Vec<Scope>,
    current: usize,
}

impl Scope {
    fn new(parent: Option<usize>) -> Self {
        Scope { symbols: HashMap::new(), parent }
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(Scope::new(None)); // global scope index 0
        SymbolTable { scopes, current: 0 }
    }

    pub fn enter_scope(&mut self) {
        let parent = Some(self.current);
        let id = self.scopes.len();
        self.scopes.push(Scope::new(parent));
        self.current = id;
    }

    pub fn leave_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        }
    }

    pub fn declare_global_function(&mut self, sig: FunctionSig) -> Result<(), String> {
        let name = sig.name.clone();
        if self.scopes[0].symbols.contains_key(&name) {
            return Err(format!("duplicate function: {}", name));
        }
        self.scopes[0].symbols.insert(name.clone(), Symbol::Function(sig));
        Ok(())
    }

    pub fn declare_local_var(&mut self, name: &str) -> Result<(), String> {
        let scope = &mut self.scopes[self.current];
        if scope.symbols.contains_key(name) {
            return Err(format!("duplicate local: {}", name));
        }
        scope.symbols.insert(name.to_string(), Symbol::Variable { name: name.to_string() });
        Ok(())
    }

    pub fn declare_param(&mut self, name: &str) -> Result<(), String> {
        let scope = &mut self.scopes[self.current];
        if scope.symbols.contains_key(name) {
            return Err(format!("duplicate param: {}", name));
        }
        scope.symbols.insert(name.to_string(), Symbol::Param { name: name.to_string() });
        Ok(())
    }

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

    pub fn find_global_function(&self, name: &str) -> Option<FunctionSig> {
        if let Some(sym) = self.scopes[0].symbols.get(name) {
            if let Symbol::Function(sig) = sym {
                return Some(sig.clone());
            }
        }
        None
    }
}
