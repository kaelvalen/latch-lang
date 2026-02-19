use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::ast::{Block, Param};
use crate::error::{LatchError, Result};

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
    Null,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_)            => "int",
            Value::Float(_)          => "float",
            Value::Bool(_)           => "bool",
            Value::Str(_)            => "string",
            Value::List(_)           => "list",
            Value::Map(_)            => "dict",
            Value::Fn { .. }         => "fn",
            Value::ProcessResult { .. } => "process",
            Value::HttpResponse { .. }  => "response",
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
    parent: Option<Box<Env>>,
}

impl Env {
    pub fn new() -> Self {
        Env { vars: HashMap::new(), parent: None }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
            .or_else(|| self.parent.as_ref()?.get(name))
    }

    pub fn set(&mut self, name: &str, val: Value) {
        self.vars.insert(name.to_string(), val);
    }

    /// Reassign an already-declared variable (walks up the chain).
    pub fn assign(&mut self, name: &str, val: Value) -> Result<()> {
        if self.vars.contains_key(name) {
            self.vars.insert(name.to_string(), val);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, val)
        } else {
            Err(LatchError::UndefinedVariable(name.to_string()))
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
            parent: Some(Box::new(self)),
        }
    }

    /// Flatten into parent (for returning from a child scope).
    pub fn into_parent(self) -> Option<Env> {
        self.parent.map(|p| *p)
    }
}
