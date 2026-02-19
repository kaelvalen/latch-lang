use std::collections::HashMap;
use std::fmt;

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
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Fn {
        params: Vec<Param>,
        body: Block,
    },
    ProcessResult {
        stdout: String,
        stderr: String,
        code: i32,
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
            Value::Map(_)            => "map",
            Value::Fn { .. }         => "fn",
            Value::ProcessResult { .. } => "process",
            Value::Null              => "null",
        }
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

    pub fn as_list(&self) -> Result<&Vec<Value>> {
        match self {
            Value::List(l) => Ok(l),
            _ => Err(LatchError::TypeMismatch {
                expected: "list".into(),
                found: self.type_name().into(),
            }),
        }
    }

    pub fn into_list(self) -> Result<Vec<Value>> {
        match self {
            Value::List(l) => Ok(l),
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
                write!(f, "[")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Value::Fn { .. } => write!(f, "<fn>"),
            Value::ProcessResult { stdout, stderr, code } => {
                write!(f, "ProcessResult(code={code}, stdout={stdout:?}, stderr={stderr:?})")
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
