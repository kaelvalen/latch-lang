use rayon::prelude::*;

use crate::ast::*;
use crate::env::{Env, Value};
use crate::error::{LatchError, Result};
use crate::lexer::Lexer;
use crate::parser::Parser;
use super::Interpreter;

impl Interpreter {
    pub(crate) fn exec_stmt(&mut self, stmt: Stmt) -> Result<()> {
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
                if let Expr::Ident(name) = &target {
                    self.env.index_assign(name, &idx, val)?;
                } else {
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
                    .ok_or_else(|| LatchError::UndefinedVariable(name.clone()))?;
                let rhs = self.eval_expr(value)?;
                let result = self.eval_binop(op, current, rhs)?;
                self.env.assign(&name, result)?;
            }

            Stmt::If { cond, then, else_ } => {
                let val = self.eval_expr(cond)?;
                if val.is_truthy() {
                    self.exec_block(then)?;
                } else if let Some(else_stmt) = else_ {
                    match *else_stmt {
                        Stmt::If { .. } => {
                            self.exec_stmt(*else_stmt)?;
                        }
                        Stmt::Expr(Expr::Fn { body, .. }) => {
                            self.exec_block(body)?;
                        }
                        _ => {
                            self.exec_stmt(*else_stmt)?;
                        }
                    }
                }
            }

            Stmt::For { var, iter, body } => {
                let list = self.eval_expr(iter)?.into_list()?;
                for item in list {
                    let parent = std::mem::replace(&mut self.env, Env::new());
                    self.env = parent.child();
                    self.env.set(&var, item);
                    
                    let mut should_continue = false;
                    for s in &body {
                        match self.exec_stmt(s.clone()) {
                            Ok(()) => {}
                            Err(LatchError::BreakSignal) => {
                                let child = std::mem::replace(&mut self.env, Env::new());
                                self.env = child.into_parent().unwrap();
                                return Ok(());
                            }
                            Err(LatchError::ContinueSignal) => {
                                should_continue = true;
                                break;
                            }
                            Err(e) => {
                                let child = std::mem::replace(&mut self.env, Env::new());
                                self.env = child.into_parent().unwrap();
                                return Err(e);
                            }
                        }
                    }
                    
                    let child = std::mem::replace(&mut self.env, Env::new());
                    self.env = child.into_parent().unwrap();
                    
                    if should_continue {
                        continue;
                    }
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

                let results: Vec<std::result::Result<(), LatchError>> = pool.install(|| {
                    list.into_par_iter()
                        .map(|item| {
                            let child_env = env_snapshot.clone().child();
                            child_env.set(&var, item);
                            let mut interp = Interpreter::with_env(child_env);
                            interp.run(body_clone.clone())
                        })
                        .collect()
                });

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

            Stmt::Try { body, catch_var, catch_body, finally_body } => {
                let parent = std::mem::replace(&mut self.env, Env::new());
                self.env = parent.child();

                let result = self.exec_block_inner(body);

                let child = std::mem::replace(&mut self.env, Env::new());
                self.env = child.into_parent().unwrap();

                let catch_result = if let Err(e) = result {
                    if matches!(e, LatchError::ReturnSignal(_)) {
                        if let Some(finally_block) = finally_body {
                            let _ = self.exec_block_inner(finally_block);
                        }
                        return Err(e);
                    }
                    let parent = std::mem::replace(&mut self.env, Env::new());
                    self.env = parent.child();
                    self.env.set(&catch_var, Value::Str(format!("{e}")));
                    let res = self.exec_block_inner(catch_body);
                    let child = std::mem::replace(&mut self.env, Env::new());
                    self.env = child.into_parent().unwrap();
                    res
                } else {
                    Ok(())
                };

                if let Some(finally_block) = finally_body {
                    let parent = std::mem::replace(&mut self.env, Env::new());
                    self.env = parent.child();
                    let finally_result = self.exec_block_inner(finally_block);
                    let child = std::mem::replace(&mut self.env, Env::new());
                    self.env = child.into_parent().unwrap();
                    
                    if finally_result.is_err() {
                        return finally_result;
                    }
                }

                catch_result?;
            }

            Stmt::Const { name, type_ann: _, value } => {
                let val = self.eval_expr(value)?;
                self.env.set_const(&name, val);
            }

            Stmt::While { cond, body } => {
                loop {
                    let val = self.eval_expr(cond.clone())?;
                    if !val.is_truthy() {
                        break;
                    }
                    let parent = std::mem::replace(&mut self.env, Env::new());
                    self.env = parent.child();
                    for s in &body {
                        match self.exec_stmt(s.clone()) {
                            Ok(()) => {}
                            Err(LatchError::BreakSignal) => {
                                let child = std::mem::replace(&mut self.env, Env::new());
                                self.env = child.into_parent().unwrap();
                                return Ok(());
                            }
                            Err(LatchError::ContinueSignal) => {
                                break;
                            }
                            Err(e) => {
                                let child = std::mem::replace(&mut self.env, Env::new());
                                self.env = child.into_parent().unwrap();
                                return Err(e);
                            }
                        }
                    }
                    let child = std::mem::replace(&mut self.env, Env::new());
                    self.env = child.into_parent().unwrap();
                }
            }

            Stmt::Break => {
                return Err(LatchError::BreakSignal);
            }

            Stmt::Continue => {
                return Err(LatchError::ContinueSignal);
            }

            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
            }

            Stmt::Class { name, fields, methods } => {
                let field_defs: Vec<(String, Option<crate::ast::Type>, Option<Block>)> = fields.into_iter()
                    .map(|(fname, ftype, fdefault)| {
                        let block = fdefault.map(|expr| vec![Stmt::Return(expr)]);
                        (fname, ftype, block)
                    })
                    .collect();
                let class_val = Value::Class { name: name.clone(), fields: field_defs, methods };
                self.env.set(&name, class_val);
            }

            Stmt::Export(names) => {
                for name in names {
                    if let Some(val) = self.env.get(&name) {
                        self.env.set(&format!("__export_{}", name), val.clone());
                    }
                }
            }

            Stmt::Import { items, module } => {
                let base = self.script_dir.clone().unwrap_or_else(|| ".".to_string());
                let filename = if module.ends_with(".lt") { module.clone() } else { format!("{}.lt", module) };
                let path = format!("{base}/{filename}");
                let canon = std::fs::canonicalize(&path)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or(path.clone());

                if self.loading.contains(&canon) {
                    return Err(LatchError::GenericError(
                        format!("Circular import detected: '{module}'")
                    ));
                }

                let source = std::fs::read_to_string(&path)
                    .map_err(|e| LatchError::IoError(format!("{path}: {e}")))?;
                let mut lexer = Lexer::new(&source);
                let tokens = lexer.tokenize()?;
                let mut parser = Parser::new(tokens);
                let ast = parser.parse_program()?;

                let mut mod_interp = Interpreter::new();
                mod_interp.loading = self.loading.clone();
                mod_interp.loading.insert(canon);
                if let Some(parent) = std::path::Path::new(&path).parent() {
                    mod_interp.script_dir = Some(parent.to_string_lossy().to_string());
                }
                mod_interp.run(ast)?;

                for item in items {
                    let export_key = format!("__export_{}", item);
                    if let Some(val) = mod_interp.env.get(&export_key).or_else(|| mod_interp.env.get(&item)) {
                        self.env.set(&item, val);
                    } else {
                        return Err(LatchError::ImportNotFound(format!("{item} from {module}")));
                    }
                }
            }

            Stmt::FieldAssign { object, field, value } => {
                let val = self.eval_expr(value)?;
                let obj = self.eval_expr(object)?;
                obj.set_field(&field, val)?;
            }

            Stmt::Match { expr, cases, default } => {
                let subject = self.eval_expr(expr)?;
                let mut matched = false;
                for (pattern, body) in cases {
                    let pat_val = self.eval_expr(pattern)?;
                    if values_equal(&subject, &pat_val) {
                        self.exec_block(body)?;
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    if let Some(default_body) = default {
                        self.exec_block(default_body)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn exec_block(&mut self, block: Block) -> Result<()> {
        let parent = std::mem::replace(&mut self.env, Env::new());
        self.env = parent.child();
        let result = self.exec_block_inner(block);
        let child = std::mem::replace(&mut self.env, Env::new());
        self.env = child.into_parent().unwrap();
        result
    }

    pub(crate) fn exec_block_inner(&mut self, block: Block) -> Result<()> {
        for stmt in block {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    pub(crate) fn exec_block_returning(&mut self, block: Block) -> Result<Value> {
        for stmt in block {
            match stmt {
                Stmt::Return(expr) => return self.eval_expr(expr),
                other => self.exec_stmt(other)?,
            }
        }
        Ok(Value::Null)
    }
}

pub(crate) fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Int(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Int(y)) => *x == (*y as f64),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Null, Value::Null) => true,
        (Value::List(x), Value::List(y)) => {
            let x_guard = x.lock().unwrap();
            let y_guard = y.lock().unwrap();
            if x_guard.len() != y_guard.len() {
                return false;
            }
            x_guard.iter().zip(y_guard.iter()).all(|(a, b)| values_equal(a, b))
        }
        (Value::Map(x), Value::Map(y)) => {
            let x_guard = x.lock().unwrap();
            let y_guard = y.lock().unwrap();
            if x_guard.len() != y_guard.len() {
                return false;
            }
            x_guard.iter().all(|(k, v)| {
                y_guard.get(k).map(|yv| values_equal(v, yv)).unwrap_or(false)
            })
        }
        (Value::Instance { fields: fx, .. }, Value::Instance { fields: fy, .. }) => {
            let gx = fx.lock().unwrap();
            let gy = fy.lock().unwrap();
            if gx.len() != gy.len() { return false; }
            gx.iter().all(|(k, v)| gy.get(k).map(|yv| values_equal(v, yv)).unwrap_or(false))
        }
        _ => false,
    }
}
