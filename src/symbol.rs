use std::collections::HashMap;

/// Strongly-Typed Identifier for Interned Strings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Variable,
    Function,
    Class,
    Field,
    Module,
    Keyword,
}

/// Global/Scoped Symbol Metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub is_global: bool,
}

/// Centralized String Interner & Symbol Table
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    strings: Vec<String>,
    map: HashMap<String, SymbolId>,
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            strings: Vec::new(),
            map: HashMap::new(),
            symbols: Vec::new(),
        }
    }

    pub fn intern(&mut self, s: impl Into<String>) -> SymbolId {
        let name = s.into();
        if let Some(&id) = self.map.get(&name) {
            return id;
        }

        let id = SymbolId(self.strings.len() as u32);
        self.map.insert(name.clone(), id);
        self.strings.push(name.clone());
        self.symbols.push(Symbol {
            id,
            name,
            kind: SymbolKind::Variable,
            is_global: true,
        });
        id
    }

    pub fn lookup_name(&self, id: SymbolId) -> Option<&str> {
        self.strings.get(id.0 as usize).map(|s| s.as_str())
    }

    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.0 as usize)
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}
