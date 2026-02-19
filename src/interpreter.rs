use std::collections::HashMap;

use rayon::prelude::*;

use crate::ast::*;
use crate::env::{Env, Value};
use crate::error::{LatchError, Result};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::runtime;

/// Tree-walk interpreter — executes a checked AST.
pub struct Interpreter {
    pub env: Env,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { env: Env::new() }
    }

    pub fn with_env(env: Env) -> Self {
        Interpreter { env }
    }

    pub fn run(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    // ── Statements ───────────────────────────────────────────

    fn exec_stmt(&mut self, stmt: Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = self.eval_expr(value)?;
                self.env.set(&name, val);
            }

            Stmt::Assign { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.assign(&name, val)?;
            }

            Stmt::IndexAssign { target, index, value } => {
                let idx = self.eval_expr(index)?;
                let val = self.eval_expr(value)?;
                // Simple case: target is Ident(name) → use env.index_assign
                if let Expr::Ident(name) = &target {
                    self.env.index_assign(name, &idx, val)?;
                } else {
                    // Nested case: evaluate target to get the container, then assign
                    let container = self.eval_expr(target)?;
                    match (&container, &idx) {
                        (Value::List(list), Value::Int(i)) => {
                            let i = *i as usize;
                            let mut guard = list.lock().unwrap();
                            if i >= guard.len() {
                                return Err(LatchError::IndexOutOfBounds { index: i as i64, len: guard.len() });
                            }
                            guard[i] = val;
                        }
                        (Value::Map(map), Value::Str(key)) => {
                            map.lock().unwrap().insert(key.clone(), val);
                        }
                        _ => return Err(LatchError::TypeMismatch {
                            expected: "list[int] or dict[string]".into(),
                            found: "incompatible types".into(),
                        }),
                    }
                }
            }

            Stmt::CompoundAssign { name, op, value } => {
                let current = self.env.get(&name)
                    .cloned()
                    .ok_or_else(|| LatchError::UndefinedVariable(name.clone()))?;
                let rhs = self.eval_expr(value)?;
                let result = self.eval_binop(op, current, rhs)?;
                self.env.assign(&name, result)?;
            }

            Stmt::If { cond, then, else_ } => {
                let val = self.eval_expr(cond)?;
                if val.is_truthy() {
                    self.exec_block(then)?;
                } else if let Some(else_block) = else_ {
                    self.exec_block(else_block)?;
                }
            }

            Stmt::For { var, iter, body } => {
                let list = self.eval_expr(iter)?.into_list()?;
                for item in list {
                    let parent = std::mem::replace(&mut self.env, Env::new());
                    self.env = parent.child();
                    self.env.set(&var, item);
                    for s in &body {
                        self.exec_stmt(s.clone())?;
                    }
                    let child = std::mem::replace(&mut self.env, Env::new());
                    self.env = child.into_parent().unwrap();
                }
            }

            Stmt::Parallel { var, iter, workers, body } => {
                let list = self.eval_expr(iter)?.into_list()?;
                let worker_count = match workers {
                    Some(w) => Some(self.eval_expr(w)?.as_int()? as usize),
                    None => None,
                };

                let pool = match worker_count {
                    Some(n) => rayon::ThreadPoolBuilder::new()
                        .num_threads(n)
                        .build()
                        .map_err(|e| LatchError::GenericError(e.to_string()))?,
                    None => rayon::ThreadPoolBuilder::new()
                        .build()
                        .map_err(|e| LatchError::GenericError(e.to_string()))?,
                };

                let env_snapshot = self.env.clone();
                let body_clone = body.clone();

                // Deterministic parallel: ALL workers run to completion.
                // Errors are collected; the first error is propagated after
                // every worker has finished. No early cancellation.
                let results: Vec<std::result::Result<(), LatchError>> = pool.install(|| {
                    list.into_par_iter()
                        .map(|item| {
                            let mut child_env = env_snapshot.clone().child();
                            child_env.set(&var, item);
                            let mut interp = Interpreter::with_env(child_env);
                            interp.run(body_clone.clone())
                        })
                        .collect()
                });

                // Propagate the first error (if any) after all workers finished
                for result in results {
                    if let Err(e) = result {
                        return Err(e);
                    }
                }
            }

            Stmt::Fn { name, params, body, .. } => {
                let val = Value::Fn { params, body, captured_env: None };
                self.env.set(&name, val);
            }

            Stmt::Return(expr) => {
                let val = self.eval_expr(expr)?;
                return Err(LatchError::ReturnSignal(val));
            }

            Stmt::Try { body, catch_var, catch_body } => {
                // Execute body in its own scope
                let parent = std::mem::replace(&mut self.env, Env::new());
                self.env = parent.child();

                let result = self.exec_block_inner(body);

                let child = std::mem::replace(&mut self.env, Env::new());
                self.env = child.into_parent().unwrap();

                if let Err(e) = result {
                    // Don't catch return signals
                    if matches!(e, LatchError::ReturnSignal(_)) {
                        return Err(e);
                    }
                    let parent = std::mem::replace(&mut self.env, Env::new());
                    self.env = parent.child();
                    self.env.set(&catch_var, Value::Str(format!("{e}")));
                    self.exec_block_inner(catch_body)?;
                    let child = std::mem::replace(&mut self.env, Env::new());
                    self.env = child.into_parent().unwrap();
                }
            }

            Stmt::Use(path) => {
                let source = std::fs::read_to_string(&path)
                    .map_err(|e| LatchError::IoError(format!("{path}: {e}")))?;
                let mut lexer = Lexer::new(&source);
                let tokens = lexer.tokenize()?;
                let mut parser = Parser::new(tokens);
                let ast = parser.parse_program()?;
                // Run imported file in the current environment
                self.run(ast)?;
            }

            Stmt::Stop(expr) => {
                let val = self.eval_expr(expr)?;
                let code = val.as_int().unwrap_or(1) as i32;
                return Err(LatchError::StopSignal(code));
            }

            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
            }
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

    fn exec_block(&mut self, block: Block) -> Result<()> {
        let parent = std::mem::replace(&mut self.env, Env::new());
        self.env = parent.child();
        let result = self.exec_block_inner(block);
        let child = std::mem::replace(&mut self.env, Env::new());
        self.env = child.into_parent().unwrap();
        result
    }

    fn exec_block_inner(&mut self, block: Block) -> Result<()> {
        for stmt in block {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    // ── Expressions ──────────────────────────────────────────

    pub fn eval_expr(&mut self, expr: Expr) -> Result<Value> {
        match expr {
            Expr::Int(n)   => Ok(Value::Int(n)),
            Expr::Float(n) => Ok(Value::Float(n)),
            Expr::Bool(b)  => Ok(Value::Bool(b)),
            Expr::Str(s)   => Ok(Value::Str(s)),
            Expr::Null     => Ok(Value::Null),

            Expr::List(items) => {
                let vals: Vec<Value> = items.into_iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_>>()?;
                Ok(Value::new_list(vals))
            }

            Expr::Map(entries) => {
                let mut map = HashMap::new();
                for (key, val_expr) in entries {
                    map.insert(key, self.eval_expr(val_expr)?);
                }
                Ok(Value::new_map(map))
            }

            Expr::Fn { params, body } => {
                // Capture the current environment for closure semantics
                let captured = self.env.clone();
                Ok(Value::Fn { params, body, captured_env: Some(Box::new(captured)) })
            }

            Expr::Ident(name) => {
                self.env.get(&name)
                    .cloned()
                    .ok_or(LatchError::UndefinedVariable(name))
            }

            Expr::Interpolated(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(&s),
                        StringPart::Expr(tokens) => {
                            let mut parser = Parser::new(tokens);
                            let expr = parser.parse_program()?;
                            // Evaluate the first (and only) expression statement
                            if let Some(Stmt::Expr(e)) = expr.into_iter().next() {
                                let val = self.eval_expr(e)?;
                                result.push_str(&format!("{val}"));
                            }
                        }
                    }
                }
                Ok(Value::Str(result))
            }

            Expr::BinOp { op, left, right } => {
                let l = self.eval_expr(*left)?;
                let r = self.eval_expr(*right)?;
                self.eval_binop(op, l, r)
            }

            Expr::UnaryOp { op, expr } => {
                let val = self.eval_expr(*expr)?;
                match op {
                    UnaryOp::Neg => match val {
                        Value::Int(n)   => Ok(Value::Int(-n)),
                        Value::Float(n) => Ok(Value::Float(-n)),
                        _ => Err(LatchError::TypeMismatch {
                            expected: "number".into(),
                            found: val.type_name().into(),
                        }),
                    },
                    UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
                }
            }

            Expr::Call { name, args } => {
                let evaluated: Vec<Value> = args.into_iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_>>()?;
                self.call_function(&name, evaluated)
            }

            Expr::ModuleCall { module, method, args } => {
                let evaluated: Vec<Value> = args.into_iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_>>()?;

                match module.as_str() {
                    "fs"   => runtime::fs::call(&method, evaluated),
                    "proc" => runtime::proc::call(&method, evaluated),
                    "http" => runtime::http::call(&method, evaluated),
                    "time" => runtime::time::call(&method, evaluated),
                    "ai"   => runtime::ai::call(&method, evaluated),
                    "json" => runtime::json::call(&method, evaluated),
                    "env"  => runtime::env::call(&method, evaluated),
                    "path" => runtime::path::call(&method, evaluated),
                    _ => Err(LatchError::UnknownModule(module)),
                }
            }

            Expr::Index { expr, index } => {
                let container = self.eval_expr(*expr)?;
                let idx = self.eval_expr(*index)?;

                match (&container, &idx) {
                    (Value::List(list), Value::Int(i)) => {
                        let i = *i;
                        let guard = list.lock().unwrap();
                        if i < 0 || i as usize >= guard.len() {
                            Err(LatchError::IndexOutOfBounds { index: i, len: guard.len() })
                        } else {
                            Ok(guard[i as usize].clone())
                        }
                    }
                    (Value::Map(map), Value::Str(key)) => {
                        let guard = map.lock().unwrap();
                        guard.get(key)
                            .cloned()
                            .ok_or(LatchError::KeyNotFound(key.clone()))
                    }
                    _ => Err(LatchError::TypeMismatch {
                        expected: "list[int] or dict[string]".into(),
                        found: format!("{}[{}]", container.type_name(), idx.type_name()),
                    }),
                }
            }

            Expr::FieldAccess { expr, field } => {
                let val = self.eval_expr(*expr)?;
                match val {
                    Value::ProcessResult { stdout, stderr, code } => {
                        match field.as_str() {
                            "stdout" => Ok(Value::Str(stdout)),
                            "stderr" => Ok(Value::Str(stderr)),
                            "code"   => Ok(Value::Int(code as i64)),
                            _ => Err(LatchError::KeyNotFound(field)),
                        }
                    }
                    Value::HttpResponse { status, body, headers } => {
                        match field.as_str() {
                            "status"  => Ok(Value::Int(status)),
                            "body"    => Ok(Value::Str(body)),
                            "headers" => {
                                let map: HashMap<String, Value> = headers.into_iter()
                                    .map(|(k, v)| (k, Value::Str(v)))
                                    .collect();
                                Ok(Value::new_map(map))
                            }
                            _ => Err(LatchError::KeyNotFound(field)),
                        }
                    }
                    Value::Map(map) => {
                        let guard = map.lock().unwrap();
                        guard.get(&field)
                            .cloned()
                            .ok_or(LatchError::KeyNotFound(field))
                    }
                    _ => Err(LatchError::TypeMismatch {
                        expected: "dict, response, or process result".into(),
                        found: val.type_name().into(),
                    }),
                }
            }

            Expr::OrDefault { expr, default } => {
                match self.eval_expr(*expr) {
                    Ok(val) => Ok(val),
                    Err(_) => self.eval_expr(*default),
                }
            }

            Expr::NullCoalesce { expr, default } => {
                let val = self.eval_expr(*expr)?;
                if matches!(val, Value::Null) {
                    self.eval_expr(*default)
                } else {
                    Ok(val)
                }
            }

            Expr::Range { start, end } => {
                let s = self.eval_expr(*start)?.as_int()?;
                let e = self.eval_expr(*end)?.as_int()?;
                let list: Vec<Value> = (s..e).map(Value::Int).collect();
                Ok(Value::new_list(list))
            }

            Expr::Pipe { expr, func } => {
                let val = self.eval_expr(*expr)?;
                // func is a Call expression — inject val as first argument
                match *func {
                    Expr::Call { name, mut args } => {
                        // Evaluate existing args, then prepend the piped value
                        let mut evaluated = vec![val];
                        for a in args.drain(..) {
                            evaluated.push(self.eval_expr(a)?);
                        }
                        self.call_function(&name, evaluated)
                    }
                    Expr::ModuleCall { module, method, mut args } => {
                        let mut evaluated = vec![val];
                        for a in args.drain(..) {
                            evaluated.push(self.eval_expr(a)?);
                        }
                        match module.as_str() {
                            "fs"   => runtime::fs::call(&method, evaluated),
                            "proc" => runtime::proc::call(&method, evaluated),
                            "http" => runtime::http::call(&method, evaluated),
                            "time" => runtime::time::call(&method, evaluated),
                            "ai"   => runtime::ai::call(&method, evaluated),
                            "json" => runtime::json::call(&method, evaluated),
                            "env"  => runtime::env::call(&method, evaluated),
                            "path" => runtime::path::call(&method, evaluated),
                            _ => Err(LatchError::UnknownModule(module)),
                        }
                    }
                    Expr::Fn { params, body } => {
                        // Pipe into anonymous function — call inline, no capture
                        self.call_closure(&params, &body, vec![val], None)
                    }
                    other => {
                        // Try evaluating as a function value
                        let func_val = self.eval_expr(other)?;
                        if let Value::Fn { params, body, captured_env } = func_val {
                            self.call_closure(&params, &body, vec![val], captured_env.map(|e| *e))
                        } else {
                            Err(LatchError::TypeMismatch {
                                expected: "function".into(),
                                found: func_val.type_name().into(),
                            })
                        }
                    }
                }
            }

            Expr::SafeAccess { expr, field } => {
                let val = self.eval_expr(*expr)?;
                match val {
                    Value::Null => Ok(Value::Null),
                    Value::Map(map) => {
                        let guard = map.lock().unwrap();
                        Ok(guard.get(&field).cloned().unwrap_or(Value::Null))
                    }
                    Value::HttpResponse { status, body, headers } => {
                        match field.as_str() {
                            "status"  => Ok(Value::Int(status)),
                            "body"    => Ok(Value::Str(body)),
                            "headers" => {
                                let map: HashMap<String, Value> = headers.into_iter()
                                    .map(|(k, v)| (k, Value::Str(v)))
                                    .collect();
                                Ok(Value::new_map(map))
                            }
                            _ => Ok(Value::Null),
                        }
                    }
                    Value::ProcessResult { stdout, stderr, code } => {
                        match field.as_str() {
                            "stdout" => Ok(Value::Str(stdout)),
                            "stderr" => Ok(Value::Str(stderr)),
                            "code"   => Ok(Value::Int(code as i64)),
                            _ => Ok(Value::Null),
                        }
                    }
                    _ => Ok(Value::Null),
                }
            }
        }
    }

    // ── Binary operations ────────────────────────────────────

    fn eval_binop(&self, op: BinOp, l: Value, r: Value) -> Result<Value> {
        // Null equality — handle before anything else
        if matches!(op, BinOp::Eq | BinOp::NotEq) {
            let is_eq = matches!((&l, &r), (Value::Null, Value::Null));
            let either_null = matches!(&l, Value::Null) || matches!(&r, Value::Null);
            if either_null {
                return match op {
                    BinOp::Eq => Ok(Value::Bool(is_eq)),
                    BinOp::NotEq => Ok(Value::Bool(!is_eq)),
                    _ => unreachable!(),
                };
            }
        }

        // String concatenation
        if matches!(op, BinOp::Add) {
            if let (Value::Str(a), Value::Str(b)) = (&l, &r) {
                return Ok(Value::Str(format!("{a}{b}")));
            }
        }

        // `in` operator: value in container
        if matches!(op, BinOp::In) {
            return match &r {
                Value::List(list) => {
                    let guard = list.lock().unwrap();
                    let found = guard.iter().any(|item| values_equal(item, &l));
                    Ok(Value::Bool(found))
                }
                Value::Str(haystack) => {
                    let needle = l.as_str()?;
                    Ok(Value::Bool(haystack.contains(needle)))
                }
                Value::Map(map) => {
                    let guard = map.lock().unwrap();
                    let key = l.as_str()?;
                    Ok(Value::Bool(guard.contains_key(key)))
                }
                _ => Err(LatchError::TypeMismatch {
                    expected: "list, string, or dict".into(),
                    found: r.type_name().into(),
                }),
            };
        }

        // Numeric operations
        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => self.int_binop(op, *a, *b),
            (Value::Float(a), Value::Float(b)) => self.float_binop(op, *a, *b),
            (Value::Int(a), Value::Float(b)) => self.float_binop(op, *a as f64, *b),
            (Value::Float(a), Value::Int(b)) => self.float_binop(op, *a, *b as f64),

            // Boolean logical operations
            (Value::Bool(a), Value::Bool(b)) => match op {
                BinOp::And => Ok(Value::Bool(*a && *b)),
                BinOp::Or  => Ok(Value::Bool(*a || *b)),
                BinOp::Eq  => Ok(Value::Bool(a == b)),
                BinOp::NotEq => Ok(Value::Bool(a != b)),
                _ => Err(LatchError::TypeMismatch {
                    expected: "numeric".into(),
                    found: "bool".into(),
                }),
            },

            // Equality for strings
            (Value::Str(a), Value::Str(b)) => match op {
                BinOp::Eq    => Ok(Value::Bool(a == b)),
                BinOp::NotEq => Ok(Value::Bool(a != b)),
                _ => Err(LatchError::TypeMismatch {
                    expected: "numeric".into(),
                    found: "string".into(),
                }),
            },

            _ => Err(LatchError::TypeMismatch {
                expected: "compatible types".into(),
                found: format!("{} and {}", l.type_name(), r.type_name()),
            }),
        }
    }

    fn int_binop(&self, op: BinOp, a: i64, b: i64) -> Result<Value> {
        match op {
            BinOp::Add   => Ok(Value::Int(a + b)),
            BinOp::Sub   => Ok(Value::Int(a - b)),
            BinOp::Mul   => Ok(Value::Int(a * b)),
            BinOp::Div   => {
                if b == 0 { return Err(LatchError::DivisionByZero); }
                Ok(Value::Int(a / b))
            }
            BinOp::Mod   => {
                if b == 0 { return Err(LatchError::DivisionByZero); }
                Ok(Value::Int(a % b))
            }
            BinOp::Eq    => Ok(Value::Bool(a == b)),
            BinOp::NotEq => Ok(Value::Bool(a != b)),
            BinOp::Lt    => Ok(Value::Bool(a < b)),
            BinOp::Gt    => Ok(Value::Bool(a > b)),
            BinOp::LtEq  => Ok(Value::Bool(a <= b)),
            BinOp::GtEq  => Ok(Value::Bool(a >= b)),
            BinOp::And | BinOp::Or | BinOp::In => Err(LatchError::TypeMismatch {
                expected: "bool".into(), found: "int".into(),
            }),
        }
    }

    fn float_binop(&self, op: BinOp, a: f64, b: f64) -> Result<Value> {
        match op {
            BinOp::Add   => Ok(Value::Float(a + b)),
            BinOp::Sub   => Ok(Value::Float(a - b)),
            BinOp::Mul   => Ok(Value::Float(a * b)),
            BinOp::Div   => {
                if b == 0.0 { return Err(LatchError::DivisionByZero); }
                Ok(Value::Float(a / b))
            }
            BinOp::Mod   => {
                if b == 0.0 { return Err(LatchError::DivisionByZero); }
                Ok(Value::Float(a % b))
            }
            BinOp::Eq    => Ok(Value::Bool(a == b)),
            BinOp::NotEq => Ok(Value::Bool(a != b)),
            BinOp::Lt    => Ok(Value::Bool(a < b)),
            BinOp::Gt    => Ok(Value::Bool(a > b)),
            BinOp::LtEq  => Ok(Value::Bool(a <= b)),
            BinOp::GtEq  => Ok(Value::Bool(a >= b)),
            BinOp::And | BinOp::Or | BinOp::In => Err(LatchError::TypeMismatch {
                expected: "bool".into(), found: "float".into(),
            }),
        }
    }

    // ── Function calls ───────────────────────────────────────

    fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Built-in functions
        match name {
            "print" => {
                if let Some(val) = args.first() {
                    println!("{val}");
                }
                return Ok(Value::Null);
            }
            "len" => {
                return match args.first() {
                    Some(Value::List(l)) => Ok(Value::Int(l.lock().unwrap().len() as i64)),
                    Some(Value::Str(s))  => Ok(Value::Int(s.len() as i64)),
                    Some(Value::Map(m))  => Ok(Value::Int(m.lock().unwrap().len() as i64)),
                    _ => Err(LatchError::TypeMismatch {
                        expected: "list, string, or dict".into(),
                        found: args.first().map(|v| v.type_name()).unwrap_or("none").into(),
                    }),
                };
            }
            "str" => {
                return match args.first() {
                    Some(val) => Ok(Value::Str(format!("{val}"))),
                    None => Ok(Value::Str(String::new())),
                };
            }
            "int" => {
                return match args.first() {
                    Some(Value::Str(s)) => s.trim().parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| LatchError::TypeMismatch {
                            expected: "parseable int".into(), found: format!("\"{s}\""),
                        }),
                    Some(Value::Float(f)) => Ok(Value::Int(*f as i64)),
                    Some(Value::Int(n)) => Ok(Value::Int(*n)),
                    _ => Err(LatchError::TypeMismatch {
                        expected: "string or number".into(),
                        found: args.first().map(|v| v.type_name()).unwrap_or("none").into(),
                    }),
                };
            }
            "float" => {
                return match args.first() {
                    Some(Value::Str(s)) => s.trim().parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| LatchError::TypeMismatch {
                            expected: "parseable float".into(), found: format!("\"{s}\""),
                        }),
                    Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
                    Some(Value::Float(f)) => Ok(Value::Float(*f)),
                    _ => Err(LatchError::TypeMismatch {
                        expected: "string or number".into(),
                        found: args.first().map(|v| v.type_name()).unwrap_or("none").into(),
                    }),
                };
            }
            "typeof" => {
                return match args.first() {
                    Some(val) => Ok(Value::Str(val.type_name().to_string())),
                    None => Ok(Value::Str("none".into())),
                };
            }
            "push" => {
                if args.len() == 2 {
                    if let Value::List(ref list) = args[0] {
                        list.lock().unwrap().push(args[1].clone());
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, value".into(),
                    found: "invalid args".into(),
                });
            }
            "keys" => {
                return match args.first() {
                    Some(Value::Map(m)) => {
                        let guard = m.lock().unwrap();
                        let mut keys: Vec<String> = guard.keys().cloned().collect();
                        keys.sort();
                        let keys: Vec<Value> = keys.into_iter().map(Value::Str).collect();
                        Ok(Value::new_list(keys))
                    }
                    _ => Err(LatchError::TypeMismatch {
                        expected: "dict".into(),
                        found: args.first().map(|v| v.type_name()).unwrap_or("none").into(),
                    }),
                };
            }
            "values" => {
                return match args.first() {
                    Some(Value::Map(m)) => {
                        let guard = m.lock().unwrap();
                        let mut sorted_keys: Vec<String> = guard.keys().cloned().collect();
                        sorted_keys.sort();
                        let vals: Vec<Value> = sorted_keys.iter().map(|k| guard[k].clone()).collect();
                        Ok(Value::new_list(vals))
                    }
                    _ => Err(LatchError::TypeMismatch {
                        expected: "dict".into(),
                        found: args.first().map(|v| v.type_name()).unwrap_or("none").into(),
                    }),
                };
            }
            "range" => {
                if args.len() == 2 {
                    let start = args[0].as_int()?;
                    let end = args[1].as_int()?;
                    let list: Vec<Value> = (start..end).map(Value::Int).collect();
                    return Ok(Value::new_list(list));
                }
                return Err(LatchError::ArgCountMismatch {
                    name: "range".into(), expected: 2, found: args.len(),
                });
            }
            "split" => {
                if args.len() == 2 {
                    let s = args[0].as_str()?.to_string();
                    let delim = args[1].as_str()?.to_string();
                    let parts: Vec<Value> = s.split(&delim)
                        .map(|p| Value::Str(p.to_string()))
                        .collect();
                    return Ok(Value::new_list(parts));
                }
                return Err(LatchError::ArgCountMismatch {
                    name: "split".into(), expected: 2, found: args.len(),
                });
            }
            "trim" => {
                return match args.first() {
                    Some(Value::Str(s)) => Ok(Value::Str(s.trim().to_string())),
                    _ => Err(LatchError::TypeMismatch {
                        expected: "string".into(),
                        found: args.first().map(|v| v.type_name()).unwrap_or("none").into(),
                    }),
                };
            }
            "lower" => {
                return match args.first() {
                    Some(Value::Str(s)) => Ok(Value::Str(s.to_lowercase())),
                    _ => Err(LatchError::TypeMismatch {
                        expected: "string".into(),
                        found: args.first().map(|v| v.type_name()).unwrap_or("none").into(),
                    }),
                };
            }
            "upper" => {
                return match args.first() {
                    Some(Value::Str(s)) => Ok(Value::Str(s.to_uppercase())),
                    _ => Err(LatchError::TypeMismatch {
                        expected: "string".into(),
                        found: args.first().map(|v| v.type_name()).unwrap_or("none").into(),
                    }),
                };
            }
            "starts_with" => {
                if args.len() == 2 {
                    let s = args[0].as_str()?;
                    let prefix = args[1].as_str()?;
                    return Ok(Value::Bool(s.starts_with(prefix)));
                }
                return Err(LatchError::ArgCountMismatch {
                    name: "starts_with".into(), expected: 2, found: args.len(),
                });
            }
            "ends_with" => {
                if args.len() == 2 {
                    let s = args[0].as_str()?;
                    let suffix = args[1].as_str()?;
                    return Ok(Value::Bool(s.ends_with(suffix)));
                }
                return Err(LatchError::ArgCountMismatch {
                    name: "ends_with".into(), expected: 2, found: args.len(),
                });
            }
            "contains" => {
                if args.len() == 2 {
                    return match (&args[0], &args[1]) {
                        (Value::Str(haystack), Value::Str(needle)) => {
                            Ok(Value::Bool(haystack.contains(needle.as_str())))
                        }
                        (Value::List(list), val) => {
                            let guard = list.lock().unwrap();
                            let found = guard.iter().any(|item| values_equal(item, val));
                            Ok(Value::Bool(found))
                        }
                        _ => Err(LatchError::TypeMismatch {
                            expected: "string or list".into(),
                            found: args[0].type_name().into(),
                        }),
                    };
                }
                return Err(LatchError::ArgCountMismatch {
                    name: "contains".into(), expected: 2, found: args.len(),
                });
            }
            "replace" => {
                if args.len() == 3 {
                    let s = args[0].as_str()?.to_string();
                    let from = args[1].as_str()?.to_string();
                    let to = args[2].as_str()?.to_string();
                    return Ok(Value::Str(s.replace(&from, &to)));
                }
                return Err(LatchError::ArgCountMismatch {
                    name: "replace".into(), expected: 3, found: args.len(),
                });
            }

            "sort" => {
                return match args.into_iter().next() {
                    Some(Value::List(list)) => {
                        let mut vec = list.lock().unwrap().clone();
                        vec.sort_by(|a, b| {
                            match (a, b) {
                                (Value::Int(x), Value::Int(y)) => x.cmp(y),
                                (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                (Value::Str(x), Value::Str(y)) => x.cmp(y),
                                _ => std::cmp::Ordering::Equal,
                            }
                        });
                        Ok(Value::new_list(vec))
                    }
                    _ => Err(LatchError::TypeMismatch {
                        expected: "list".into(),
                        found: "invalid args".into(),
                    }),
                };
            }

            // filter(list, fn) — returns items where fn(item) is truthy
            "filter" => {
                if args.len() != 2 {
                    return Err(LatchError::ArgCountMismatch {
                        name: "filter".into(), expected: 2, found: args.len(),
                    });
                }
                let list = args[0].clone().into_list()?;
                let func = args[1].clone();
                if let Value::Fn { params, body, captured_env } = func {
                    let mut result = Vec::new();
                    for item in list {
                        let val = self.call_closure(&params, &body, vec![item.clone()], captured_env.as_deref().cloned())?;
                        if val.is_truthy() {
                            result.push(item);
                        }
                    }
                    return Ok(Value::new_list(result));
                }
                return Err(LatchError::TypeMismatch {
                    expected: "fn".into(), found: args[1].type_name().into(),
                });
            }

            // map(list, fn) — returns [fn(item) for each item]
            "map" => {
                if args.len() != 2 {
                    return Err(LatchError::ArgCountMismatch {
                        name: "map".into(), expected: 2, found: args.len(),
                    });
                }
                let list = args[0].clone().into_list()?;
                let func = args[1].clone();
                if let Value::Fn { params, body, captured_env } = func {
                    let mut result = Vec::new();
                    for item in list {
                        let val = self.call_closure(&params, &body, vec![item], captured_env.as_deref().cloned())?;
                        result.push(val);
                    }
                    return Ok(Value::new_list(result));
                }
                return Err(LatchError::TypeMismatch {
                    expected: "fn".into(), found: args[1].type_name().into(),
                });
            }

            // each(list, fn) — runs fn(item) for each item, returns null
            "each" => {
                if args.len() != 2 {
                    return Err(LatchError::ArgCountMismatch {
                        name: "each".into(), expected: 2, found: args.len(),
                    });
                }
                let list = args[0].clone().into_list()?;
                let func = args[1].clone();
                if let Value::Fn { params, body, captured_env } = func {
                    for item in list {
                        self.call_closure(&params, &body, vec![item], captured_env.as_deref().cloned())?;
                    }
                    return Ok(Value::Null);
                }
                return Err(LatchError::TypeMismatch {
                    expected: "fn".into(), found: args[1].type_name().into(),
                });
            }

            _ => {}
        }

        // User-defined functions
        let func = self.env.get(name).cloned();
        match func {
            Some(Value::Fn { params, body, captured_env }) => {
                self.call_closure(&params, &body, args, captured_env.map(|e| *e))
            }
            _ => Err(LatchError::UndefinedFunction(name.to_string())),
        }
    }

    /// Call a closure (Fn value) with the given arguments.
    /// If `captured_env` is provided, use it as the parent scope (closure semantics).
    /// Otherwise, use the current env as the parent (regular function call).
    fn call_closure(&mut self, params: &[Param], body: &Block, args: Vec<Value>, captured_env: Option<Env>) -> Result<Value> {
        // Save the caller's environment
        let caller_env = std::mem::replace(&mut self.env, Env::new());

        // Set up the function scope
        self.env = match captured_env {
            Some(cap) => cap.child(),            // closure: parent = captured env
            None => caller_env.clone().child(),   // regular fn: parent = caller env
        };

        for (param, arg) in params.iter().zip(args.into_iter()) {
            self.env.set(&param.name, arg);
        }

        let result = self.exec_block_inner(body.clone());

        // Restore the caller's environment
        self.env = caller_env;

        match result {
            Ok(()) => Ok(Value::Null),
            Err(LatchError::ReturnSignal(val)) => Ok(val),
            Err(e) => Err(e),
        }
    }
}

/// Structural equality for Latch values (used by `in`, `contains`, `==`).
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Int(y)) => *x == (*y as f64),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Null, Value::Null) => true,
        _ => false,
    }
}
