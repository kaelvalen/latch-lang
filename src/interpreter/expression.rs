use std::collections::HashMap;

use crate::ast::*;
use crate::env::Value;
use crate::error::{LatchError, Result};
use crate::parser::Parser;
use crate::runtime;

use super::statement::values_equal;
use super::Interpreter;

impl Interpreter {
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

            Expr::Call { name, args, kwargs: _ } => {
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
                    "fs"     => runtime::fs::call(&method, evaluated),
                    "proc"   => runtime::proc::call(&method, evaluated),
                    "http"   => runtime::http::call(&method, evaluated),
                    "time"   => runtime::time::call(&method, evaluated),
                    "ai"     => runtime::ai::call(&method, evaluated),
                    "json"   => runtime::json::call(&method, evaluated),
                    "env"    => runtime::env::call(&method, evaluated),
                    "path"   => runtime::path::call(&method, evaluated),
                    "math"   => runtime::math::call(&method, evaluated),
                    "regex"  => runtime::regex::call(&method, evaluated),
                    "hash"   => runtime::hash::call(&method, evaluated),
                    "set"    => runtime::set::call(&method, evaluated),
                    "csv"    => runtime::csv::call(&method, evaluated),
                    "base64" => runtime::base64::call(&method, evaluated),
                    _ => Err(LatchError::UnknownModule(module)),
                }
            }

            Expr::MethodCall { receiver, method, args } => {
                let recv = self.eval_expr(*receiver)?;
                let evaluated: Vec<Value> = args.into_iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_>>()?;
                self.call_method(recv, &method, evaluated)
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
                    Value::Instance { fields, .. } => {
                        let guard = fields.lock().unwrap();
                        guard.get(&field)
                            .cloned()
                            .ok_or(LatchError::KeyNotFound(field))
                    }
                    Value::Map(map) => {
                        let guard = map.lock().unwrap();
                        guard.get(&field)
                            .cloned()
                            .ok_or(LatchError::KeyNotFound(field))
                    }
                    _ => Err(LatchError::TypeMismatch {
                        expected: "dict, instance, response, or process result".into(),
                        found: val.type_name().into(),
                    }),
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
                match *func {
                    Expr::Call { name, mut args, kwargs: _ } => {
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
                            "fs"     => runtime::fs::call(&method, evaluated),
                            "proc"   => runtime::proc::call(&method, evaluated),
                            "http"   => runtime::http::call(&method, evaluated),
                            "time"   => runtime::time::call(&method, evaluated),
                            "ai"     => runtime::ai::call(&method, evaluated),
                            "json"   => runtime::json::call(&method, evaluated),
                            "env"    => runtime::env::call(&method, evaluated),
                            "path"   => runtime::path::call(&method, evaluated),
                            "math"   => runtime::math::call(&method, evaluated),
                            "regex"  => runtime::regex::call(&method, evaluated),
                            "hash"   => runtime::hash::call(&method, evaluated),
                            "set"    => runtime::set::call(&method, evaluated),
                            "csv"    => runtime::csv::call(&method, evaluated),
                            "base64" => runtime::base64::call(&method, evaluated),
                            _ => Err(LatchError::UnknownModule(module)),
                        }
                    }
                    Expr::Fn { params, body } => {
                        self.call_closure(&params, &body, vec![val], None)
                    }
                    other => {
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
                    Value::Instance { fields, .. } => {
                        let guard = fields.lock().unwrap();
                        Ok(guard.get(&field).cloned().unwrap_or(Value::Null))
                    }
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

            Expr::Ternary { cond, true_branch, false_branch } => {
                let condition = self.eval_expr(*cond)?;
                if condition.is_truthy() {
                    self.eval_expr(*true_branch)
                } else {
                    self.eval_expr(*false_branch)
                }
            }

            Expr::ListComp { body, var, iter, cond } => {
                let iterable = self.eval_expr(*iter)?;
                let items = iterable.into_list()?;
                let mut result = Vec::new();
                
                for item in items {
                    let parent = std::mem::replace(&mut self.env, crate::env::Env::new());
                    self.env = parent.child();
                    self.env.set(&var, item);
                    
                    let include = if let Some(ref c) = cond {
                        self.eval_expr(*c.clone())?.is_truthy()
                    } else {
                        true
                    };
                    
                    if include {
                        let val = self.eval_expr(*body.clone())?;
                        result.push(val);
                    }
                    
                    let child = std::mem::replace(&mut self.env, crate::env::Env::new());
                    self.env = child.into_parent().unwrap();
                }
                
                Ok(Value::new_list(result))
            }

            Expr::Slice { expr, start, end } => {
                let list_val = self.eval_expr(*expr)?;
                match list_val {
                    Value::List(list) => {
                        let guard = list.lock().unwrap();
                        let len = guard.len() as i64;
                        
                        let start_idx = match start {
                            Some(s) => {
                                let s_val = self.eval_expr(*s)?;
                                let s_int = s_val.as_int()?;
                                if s_int < 0 { len + s_int } else { s_int }
                            }
                            None => 0,
                        };
                        
                        let end_idx = match end {
                            Some(e) => {
                                let e_val = self.eval_expr(*e)?;
                                let e_int = e_val.as_int()?;
                                if e_int < 0 { len + e_int } else { e_int }
                            }
                            None => len,
                        };
                        
                        let start_idx = start_idx.max(0).min(len) as usize;
                        let end_idx = end_idx.max(0).min(len) as usize;
                        
                        let sliced: Vec<Value> = guard[start_idx..end_idx].to_vec();
                        Ok(Value::new_list(sliced))
                    }
                    _ => Err(LatchError::TypeMismatch {
                        expected: "list".into(),
                        found: list_val.type_name().into(),
                    }),
                }
            }
        }
    }

    pub(crate) fn eval_binop(&self, op: BinOp, l: Value, r: Value) -> Result<Value> {
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

        if matches!(op, BinOp::Add) {
            if let (Value::Str(a), Value::Str(b)) = (&l, &r) {
                return Ok(Value::Str(format!("{a}{b}")));
            }
        }

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

        match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => self.int_binop(op, *a, *b),
            (Value::Float(a), Value::Float(b)) => self.float_binop(op, *a, *b),
            (Value::Int(a), Value::Float(b)) => self.float_binop(op, *a as f64, *b),
            (Value::Float(a), Value::Int(b)) => self.float_binop(op, *a, *b as f64),

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

            (Value::Str(a), Value::Str(b)) => match op {
                BinOp::Eq    => Ok(Value::Bool(a == b)),
                BinOp::NotEq => Ok(Value::Bool(a != b)),
                _ => Err(LatchError::TypeMismatch {
                    expected: "numeric".into(),
                    found: "string".into(),
                }),
            },

            (Value::List(_), Value::List(_)) => match op {
                BinOp::Eq    => Ok(Value::Bool(values_equal(&l, &r))),
                BinOp::NotEq => Ok(Value::Bool(!values_equal(&l, &r))),
                _ => Err(LatchError::TypeMismatch {
                    expected: "numeric".into(),
                    found: "list".into(),
                }),
            },

            (Value::List(list), Value::Int(n)) | (Value::Int(n), Value::List(list)) => {
                if op == BinOp::Mul {
                    if *n < 0 {
                        return Err(LatchError::GenericError("cannot multiply list by negative number".into()));
                    }
                    let guard = list.lock().unwrap();
                    let mut result = Vec::new();
                    for _ in 0..*n {
                        result.extend(guard.clone());
                    }
                    return Ok(Value::new_list(result));
                }
                Err(LatchError::TypeMismatch {
                    expected: "numeric".into(),
                    found: "list and int".into(),
                })
            },

            (Value::Map(_), Value::Map(_)) => match op {
                BinOp::Eq    => Ok(Value::Bool(values_equal(&l, &r))),
                BinOp::NotEq => Ok(Value::Bool(!values_equal(&l, &r))),
                _ => Err(LatchError::TypeMismatch {
                    expected: "numeric".into(),
                    found: "dict".into(),
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
            BinOp::Add   => a.checked_add(b)
                .map(Value::Int)
                .ok_or(LatchError::GenericError("integer overflow".into())),
            BinOp::Sub   => a.checked_sub(b)
                .map(Value::Int)
                .ok_or(LatchError::GenericError("integer overflow".into())),
            BinOp::Mul   => a.checked_mul(b)
                .map(Value::Int)
                .ok_or(LatchError::GenericError("integer overflow".into())),
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
}
