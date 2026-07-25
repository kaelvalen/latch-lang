use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::ast::{Block, Param, Type};
use crate::error::{LatchError, Result};

use crate::vm::Chunk;

/// Generic Object Pointer Reference Wrapper (Fully Encapsulated for GC swapping)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjRef<T>(pub Arc<T>);

impl<T> ObjRef<T> {
    pub fn new(val: T) -> Self {
        ObjRef(Arc::new(val))
    }

    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Arc::ptr_eq(&a.0, &b.0)
    }
}

impl<T> std::ops::Deref for ObjRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Unified Object Header for Wren / Lua style heap object representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjKind {
    String,
    List,
    Map,
    Function,
    Closure,
    Class,
    Instance,
    Module,
    Native,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjHeader {
    pub kind: ObjKind,
    pub flags: u8,
    pub mark: bool,
    pub generation: u8,
    pub size: usize,
    pub type_id: u32,
}

impl ObjHeader {
    pub fn new(kind: ObjKind) -> Self {
        ObjHeader {
            kind,
            flags: 0,
            mark: false,
            generation: 0,
            size: std::mem::size_of::<Self>(),
            type_id: 0,
        }
    }
}

/// First-class Compiled Function Object in the VM.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjFunction {
    pub header: ObjHeader,
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
    pub upvalue_count: usize,
}

impl ObjFunction {
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        ObjFunction {
            header: ObjHeader::new(ObjKind::Function),
            arity,
            chunk: Chunk::new(),
            name: name.into(),
            upvalue_count: 0,
        }
    }
}

/// First-class Compiled Closure Object in the VM.
#[derive(Debug, Clone)]
pub struct ObjClosure {
    pub header: ObjHeader,
    pub function: ObjRef<ObjFunction>,
    pub upvalues: Vec<Arc<Mutex<Value>>>,
}

impl PartialEq for ObjClosure {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
    }
}

impl ObjClosure {
    pub fn new(function: ObjRef<ObjFunction>, upvalues: Vec<Arc<Mutex<Value>>>) -> Self {
        ObjClosure {
            header: ObjHeader::new(ObjKind::Closure),
            function,
            upvalues,
        }
    }
}

/// Compiled Class Object in the VM.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjClass {
    pub header: ObjHeader,
    pub name: String,
    pub methods: HashMap<String, Value>,
}

impl ObjClass {
    pub fn new(name: impl Into<String>) -> Self {
        ObjClass {
            header: ObjHeader::new(ObjKind::Class),
            name: name.into(),
            methods: HashMap::new(),
        }
    }
}

/// Compiled Instance Object in the VM.
#[derive(Debug, Clone)]
pub struct ObjInstance {
    pub header: ObjHeader,
    pub class: Arc<ObjClass>,
    pub fields: Arc<Mutex<HashMap<String, Value>>>,
}

impl PartialEq for ObjInstance {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.class, &other.class)
    }
}

/// Isolated Module Object in the VM.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjModule {
    pub header: ObjHeader,
    pub name: String,
    pub globals: Vec<Value>,
    pub exports: HashMap<String, usize>,
}

impl ObjModule {
    pub fn new(name: impl Into<String>) -> Self {
        ObjModule {
            header: ObjHeader::new(ObjKind::Module),
            name: name.into(),
            globals: Vec::new(),
            exports: HashMap::new(),
        }
    }
}

/// First-class Native C/Rust Function Object in the VM.
#[derive(Clone)]
pub struct ObjNative {
    pub header: ObjHeader,
    pub name: String,
    pub function: fn(&[Value]) -> Result<Value>,
}

impl PartialEq for ObjNative {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl fmt::Debug for ObjNative {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn {}>", self.name)
    }
}

/// Runtime value – the result of evaluating any expression.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Arc<Mutex<Vec<Value>>>),
    Map(Arc<Mutex<HashMap<String, Value>>>),
    Function(Arc<ObjFunction>),
    Closure(Arc<ObjClosure>),
    Native(Arc<ObjNative>),
    Fn {
        params: Vec<Param>,
        body: Block,
        captured_env: Option<Box<Env>>,
    },
    ProcessResult {
        stdout: String,
        stderr: String,
        code: i32,
    },
    HttpResponse {
        status: i64,
        body: String,
        headers: HashMap<String, String>,
    },
    /// A class definition (not an instance).
    Class {
        name: String,
        fields: Vec<(String, Option<Type>, Option<Block>)>, // name, type, default_stmts
        methods: Vec<(String, Vec<Param>, Block)>,
    },
    /// A class instance.
    Instance {
        class_name: String,
        fields: Arc<Mutex<HashMap<String, Value>>>,
        methods: Arc<Vec<(String, Vec<Param>, Block)>>,
    },
    Null,
}

impl Value {
    // ── Constructors ─────────────────────────────────────────
    pub fn int(n: i64) -> Value { Value::Int(n) }
    pub fn float(n: f64) -> Value { Value::Float(n) }
    pub fn bool(b: bool) -> Value { Value::Bool(b) }
    pub fn str(s: impl Into<String>) -> Value { Value::Str(s.into()) }
    pub fn null() -> Value { Value::Null }

    // ── Type Predicates ──────────────────────────────────────
    pub fn is_int(&self) -> bool { matches!(self, Value::Int(_)) }
    pub fn is_float(&self) -> bool { matches!(self, Value::Float(_)) }
    pub fn is_number(&self) -> bool { matches!(self, Value::Int(_) | Value::Float(_)) }
    pub fn is_bool(&self) -> bool { matches!(self, Value::Bool(_)) }
    pub fn is_str(&self) -> bool { matches!(self, Value::Str(_)) }
    pub fn is_null(&self) -> bool { matches!(self, Value::Null) }
    pub fn is_list(&self) -> bool { matches!(self, Value::List(_)) }
    pub fn is_map(&self) -> bool { matches!(self, Value::Map(_)) }
    pub fn is_fn(&self) -> bool { matches!(self, Value::Fn { .. }) }

    // ── Encapsulated Operations ──────────────────────────────
    pub fn add(&self, rhs: &Value) -> Result<Value> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
            _ => Err(LatchError::TypeMismatch {
                expected: "numeric or string".into(),
                found: format!("{} and {}", self.type_name(), rhs.type_name()),
            }),
        }
    }

    pub fn sub(&self, rhs: &Value) -> Result<Value> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(LatchError::TypeMismatch {
                expected: "numeric".into(),
                found: format!("{} and {}", self.type_name(), rhs.type_name()),
            }),
        }
    }

    pub fn mul(&self, rhs: &Value) -> Result<Value> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(LatchError::TypeMismatch {
                expected: "numeric".into(),
                found: format!("{} and {}", self.type_name(), rhs.type_name()),
            }),
        }
    }

    pub fn div(&self, rhs: &Value) -> Result<Value> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 { return Err(LatchError::DivisionByZero); }
                Ok(Value::Int(a / b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 { return Err(LatchError::DivisionByZero); }
                Ok(Value::Float(a / b))
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 { return Err(LatchError::DivisionByZero); }
                Ok(Value::Float(*a as f64 / b))
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 { return Err(LatchError::DivisionByZero); }
                Ok(Value::Float(a / *b as f64))
            }
            _ => Err(LatchError::TypeMismatch {
                expected: "numeric".into(),
                found: format!("{} and {}", self.type_name(), rhs.type_name()),
            }),
        }
    }

    pub fn negate(&self) -> Result<Value> {
        match self {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(LatchError::TypeMismatch {
                expected: "number".into(),
                found: self.type_name().into(),
            }),
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_)            => "int",
            Value::Float(_)          => "float",
            Value::Bool(_)           => "bool",
            Value::Str(_)            => "string",
            Value::List(_)           => "list",
            Value::Map(_)            => "dict",
            Value::Fn { .. }         => "fn",
            Value::Function(_)       => "fn",
            Value::Closure(_)        => "fn",
            Value::Native(_)         => "native_fn",
            Value::ProcessResult { .. } => "process",
            Value::HttpResponse { .. }  => "response",
            Value::Class { .. }      => "class",
            Value::Instance { class_name, .. } => class_name.as_str(),
            Value::Null              => "null",
        }
    }

    /// Construct a new reference-counted list.
    pub fn new_list(items: Vec<Value>) -> Value {
        Value::List(Arc::new(Mutex::new(items)))
    }

    /// Construct a new reference-counted dict.
    pub fn new_map(map: HashMap<String, Value>) -> Value {
        Value::Map(Arc::new(Mutex::new(map)))
    }

    pub fn as_int(&self) -> Result<i64> {
        match self {
            Value::Int(n) => Ok(*n),
            _ => Err(LatchError::TypeMismatch {
                expected: "int".into(),
                found: self.type_name().into(),
            }),
        }
    }

    #[allow(dead_code)]
    pub fn as_float(&self) -> Result<f64> {
        match self {
            Value::Float(n) => Ok(*n),
            Value::Int(n)   => Ok(*n as f64),
            _ => Err(LatchError::TypeMismatch {
                expected: "float".into(),
                found: self.type_name().into(),
            }),
        }
    }

    #[allow(dead_code)]
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(LatchError::TypeMismatch {
                expected: "bool".into(),
                found: self.type_name().into(),
            }),
        }
    }

    pub fn as_str(&self) -> Result<&str> {
        match self {
            Value::Str(s) => Ok(s),
            _ => Err(LatchError::TypeMismatch {
                expected: "string".into(),
                found: self.type_name().into(),
            }),
        }
    }

    pub fn as_list(&self) -> Result<Vec<Value>> {
        match self {
            Value::List(l) => Ok(l.lock().unwrap().clone()),
            _ => Err(LatchError::TypeMismatch {
                expected: "list".into(),
                found: self.type_name().into(),
            }),
        }
    }

    pub fn into_list(self) -> Result<Vec<Value>> {
        match self {
            Value::List(l) => Ok(l.lock().unwrap().clone()),
            _ => Err(LatchError::TypeMismatch {
                expected: "list".into(),
                found: self.type_name().into(),
            }),
        }
    }


    /// Truthiness: false and null are falsy, everything else is truthy.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Null => false,
            Value::Int(0) => false,
            Value::Str(s) if s.is_empty() => false,
            _ => true,
        }
    }

    /// Get a field from an instance (for `instance.field`).
    pub fn get_field(&self, field: &str) -> Option<Value> {
        match self {
            Value::Instance { fields, .. } => {
                fields.lock().unwrap().get(field).cloned()
            }
            Value::ProcessResult { stdout, stderr, code } => match field {
                "stdout" => Some(Value::Str(stdout.clone())),
                "stderr" => Some(Value::Str(stderr.clone())),
                "code"   => Some(Value::Int(*code as i64)),
                _ => None,
            },
            Value::Map(map) => {
                map.lock().unwrap().get(field).cloned()
            }
            _ => None,
        }
    }

    /// Set a field on an instance in-place (for `self.field = val`).
    pub fn set_field(&self, field: &str, val: Value) -> Result<()> {
        match self {
            Value::Instance { fields, .. } => {
                fields.lock().unwrap().insert(field.to_string(), val);
                Ok(())
            }
            _ => Err(LatchError::TypeMismatch {
                expected: "instance".into(),
                found: self.type_name().into(),
            }),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Int(a), Value::Float(b)) => (*a as f64) == *b,
            (Value::Float(a), Value::Int(b)) => *a == (*b as f64),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::List(a), Value::List(b)) => {
                let ga = a.lock().unwrap();
                let gb = b.lock().unwrap();
                *ga == *gb
            }
            (Value::Map(a), Value::Map(b)) => {
                let ga = a.lock().unwrap();
                let gb = b.lock().unwrap();
                *ga == *gb
            }
            (Value::Function(a), Value::Function(b)) => Arc::ptr_eq(a, b) || a == b,
            (Value::Closure(a), Value::Closure(b)) => Arc::ptr_eq(a, b) || a == b,
            (Value::Native(a), Value::Native(b)) => Arc::ptr_eq(a, b) || a == b,
            (Value::Class { name: na, .. }, Value::Class { name: nb, .. }) => na == nb,
            (Value::Instance { class_name: ca, fields: fa, .. }, Value::Instance { class_name: cb, fields: fb, .. }) => {
                if ca != cb { return false; }
                let ga = fa.lock().unwrap();
                let gb = fb.lock().unwrap();
                *ga == *gb
            }
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Null => write!(f, "null"),
            Value::List(items) => {
                let items = items.lock().unwrap();
                write!(f, "[")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::Map(map) => {
                let map = map.lock().unwrap();
                let mut sorted_entries: Vec<_> = map.iter().collect();
                sorted_entries.sort_by_key(|(k, _)| (*k).clone());
                write!(f, "{{")?;
                for (i, (k, v)) in sorted_entries.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Value::Fn { .. } => write!(f, "<fn>"),
            Value::Function(func) => write!(f, "<fn {}>", func.name),
            Value::Closure(c) => write!(f, "<fn {}>", c.function.name),
            Value::Native(n) => write!(f, "<native fn {}>", n.name),
            Value::Class { name, .. } => write!(f, "<class {name}>"),
            Value::Instance { class_name, fields, .. } => {
                let guard = fields.lock().unwrap();
                let mut sorted: Vec<_> = guard.iter().collect();
                sorted.sort_by_key(|(k, _)| (*k).clone());
                write!(f, "{class_name} {{")?;
                for (i, (k, v)) in sorted.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Value::ProcessResult { stdout, stderr, code } => {
                write!(f, "ProcessResult(code={code}, stdout={stdout:?}, stderr={stderr:?})")
            }
            Value::HttpResponse { status, body, .. } => {
                let preview = if body.len() > 80 { &body[..80] } else { body.as_str() };
                write!(f, "HttpResponse(status={status}, body={preview:?}...)")
            }
        }
    }
}

// ── Environment (scope chain) ────────────────────────────────

#[derive(Debug, Clone)]
pub struct Env {
    vars: HashMap<String, Value>,
    consts: HashSet<String>,
    parent: Option<Box<Env>>,
}

impl Env {
    pub fn new() -> Self {
        Env { vars: HashMap::new(), consts: HashSet::new(), parent: None }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
            .or_else(|| self.parent.as_ref()?.get(name))
    }

    /// Declare or overwrite a mutable variable in the current scope.
    pub fn set(&mut self, name: &str, val: Value) {
        self.vars.insert(name.to_string(), val);
    }

    /// Declare an immutable constant in the current scope.
    pub fn set_const(&mut self, name: &str, val: Value) {
        self.vars.insert(name.to_string(), val);
        self.consts.insert(name.to_string());
    }

    fn is_const(&self, name: &str) -> bool {
        if self.consts.contains(name) { return true; }
        self.parent.as_ref().map(|p| p.is_const(name)).unwrap_or(false)
    }

    fn has(&self, name: &str) -> bool {
        self.vars.contains_key(name)
            || self.parent.as_ref().map(|p| p.has(name)).unwrap_or(false)
    }

    /// Assign to a variable — walks up the chain if it already exists there,
    /// otherwise declares in the current scope (Python-style).
    pub fn assign(&mut self, name: &str, val: Value) -> Result<()> {
        if self.is_const(name) {
            return Err(LatchError::GenericError(
                format!("Cannot reassign constant '{name}'")
            ));
        }
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), val);
            Ok(())
        } else if let Some(ref mut parent) = self.parent {
            if parent.has(name) {
                parent.assign(name, val)
            } else {
                // Not found anywhere — declare in current scope
                self.vars.insert(name.to_string(), val);
                Ok(())
            }
        } else {
            // Top-level scope — just declare
            self.vars.insert(name.to_string(), val);
            Ok(())
        }
    }

    /// Mutate a list or map element in-place: `name[index] = val`.
    pub fn index_assign(&mut self, name: &str, index: &Value, val: Value) -> Result<()> {
        // Find the variable in the scope chain and mutate it.
        // With Arc<Mutex> values, mutation goes through the lock,
        // so aliased lists/maps see the change.
        if let Some(container) = self.vars.get(name) {
            match (container, index) {
                (Value::List(list), Value::Int(i)) => {
                    let i = *i as usize;
                    let mut guard = list.lock().unwrap();
                    if i >= guard.len() {
                        return Err(LatchError::IndexOutOfBounds { index: i as i64, len: guard.len() });
                    }
                    guard[i] = val;
                    Ok(())
                }
                (Value::Map(map), Value::Str(key)) => {
                    map.lock().unwrap().insert(key.clone(), val);
                    Ok(())
                }
                _ => Err(LatchError::TypeMismatch {
                    expected: "list[int] or dict[string]".into(),
                    found: "incompatible types".into(),
                }),
            }
        } else if let Some(parent) = &mut self.parent {
            parent.index_assign(name, index, val)
        } else {
            Err(LatchError::UndefinedVariable(name.to_string()))
        }
    }

    /// Create a child scope.
    pub fn child(self) -> Env {
        Env {
            vars: HashMap::new(),
            consts: HashSet::new(),
            parent: Some(Box::new(self)),
        }
    }

    /// Flatten into parent (for returning from a child scope).
    pub fn into_parent(self) -> Option<Env> {
        self.parent.map(|p| *p)
    }
}
