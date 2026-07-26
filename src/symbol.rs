use std::collections::HashMap;

/// Strongly-Typed Identifier for Interned Raw Strings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedStringId(pub u32);

/// Strongly-Typed Identifier for Resolved Compiler Symbols
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Private,
    Public,
    Exported,
}

/// Rich Compiler Symbol Metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub string_id: InternedStringId,
    pub name: String,
    pub kind: SymbolKind,
    pub is_global: bool,
    pub module_id: u32,
    pub visibility: Visibility,
    pub type_ann: Option<String>,
    pub flags: u32,
}

/// Centralized String Interner & Production Symbol Table
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    strings: Vec<String>,
    string_map: HashMap<String, InternedStringId>,
    symbols: Vec<Symbol>,
    symbol_map: HashMap<String, SymbolId>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            strings: Vec::new(),
            string_map: HashMap::new(),
            symbols: Vec::new(),
            symbol_map: HashMap::new(),
        }
    }

    pub fn intern_string(&mut self, s: impl Into<String>) -> InternedStringId {
        let name = s.into();
        if let Some(&id) = self.string_map.get(&name) {
            return id;
        }

        let id = InternedStringId(self.strings.len() as u32);
        self.string_map.insert(name.clone(), id);
        self.strings.push(name);
        id
    }

    pub fn intern(&mut self, s: impl Into<String>) -> SymbolId {
        let name = s.into();
        if let Some(&id) = self.symbol_map.get(&name) {
            return id;
        }

        let string_id = self.intern_string(&name);
        let id = SymbolId(self.symbols.len() as u32);
        self.symbol_map.insert(name.clone(), id);
        self.symbols.push(Symbol {
            id,
            string_id,
            name,
            kind: SymbolKind::Variable,
            is_global: true,
            module_id: 0,
            visibility: Visibility::Public,
            type_ann: None,
            flags: 0,
        });
        id
    }

    pub fn lookup_name(&self, id: SymbolId) -> Option<&str> {
        self.symbols.get(id.0 as usize).map(|sym| sym.name.as_str())
    }

    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.0 as usize)
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }
}
