pub mod expression;
pub mod function;
pub mod statement;

use std::collections::HashSet;

use crate::ast::*;
use crate::env::{Env, Value};
use crate::error::Result;

/// Tree-walk interpreter — executes a checked AST.
pub struct Interpreter {
    pub env: Env,
    /// Tracks modules currently being loaded (for circular import detection).
    pub(crate) loading: HashSet<String>,
    /// Script file directory (for relative imports).
    pub(crate) script_dir: Option<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { env: Env::new(), loading: HashSet::new(), script_dir: None }
    }

    pub fn with_env(env: Env) -> Self {
        Interpreter { env, loading: HashSet::new(), script_dir: None }
    }

    pub fn set_script_dir(&mut self, dir: &str) {
        self.script_dir = Some(dir.to_string());
    }

    pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    /// Public wrapper for REPL: execute a single statement.
    pub fn exec_stmt_public(&mut self, stmt: Stmt) -> Result<()> {
        self.exec_stmt(stmt)
    }

    /// REPL helper: evaluate an expression statement and return its value.
    pub fn eval_stmt_for_repl(&mut self, stmt: Stmt) -> Result<Option<Value>> {
        match stmt {
            Stmt::Expr(expr) => {
                let val = self.eval_expr(expr)?;
                match &val {
                    Value::Null => Ok(None),
                    _ => Ok(Some(val)),
                }
            }
            other => {
                self.exec_stmt(other)?;
                Ok(None)
            }
        }
    }
}
