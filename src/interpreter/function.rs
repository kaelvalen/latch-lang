#![allow(clippy::mem_replace_with_default)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ast::*;
use crate::env::{Env, Value};
use crate::error::{LatchError, Result};

use super::statement::values_equal;
use super::Interpreter;

impl Interpreter {
    pub(crate) fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
        // Built-in functions
        match name {
            "print" => {
                if let Some(val) = args.first() {
                    if let Value::Instance { .. } = val {
                        match self.call_method(val.clone(), "to_str", vec![]) {
                            Ok(Value::Str(s)) => {
                                println!("{s}");
                            }
                            _ => {
                                println!("{val}");
                            }
                        }
                    } else {
                        println!("{val}");
                    }
                } else {
                    println!();
                }
                return Ok(Value::Null);
            }

            "exit" => {
                let code = args.first().and_then(|v| v.as_int().ok()).unwrap_or(0) as i32;
                std::process::exit(code);
            }
            "len" => {
                return match args.first() {
                    Some(Value::List(l)) => Ok(Value::Int(l.lock().unwrap().len() as i64)),
                    Some(Value::Str(s)) => Ok(Value::Int(s.len() as i64)),
                    Some(Value::Map(m)) => Ok(Value::Int(m.lock().unwrap().len() as i64)),
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
                    Some(Value::Str(s)) => s.trim().parse::<i64>().map(Value::Int).map_err(|_| {
                        LatchError::TypeMismatch {
                            expected: "parseable int".into(),
                            found: format!("\"{s}\""),
                        }
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
                    Some(Value::Str(s)) => {
                        s.trim().parse::<f64>().map(Value::Float).map_err(|_| {
                            LatchError::TypeMismatch {
                                expected: "parseable float".into(),
                                found: format!("\"{s}\""),
                            }
                        })
                    }
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

            "extend" => {
                if args.len() == 2 {
                    if let (Value::List(ref list), Value::List(ref items)) = (&args[0], &args[1]) {
                        let mut guard = list.lock().unwrap();
                        let items_guard = items.lock().unwrap();
                        for item in items_guard.iter() {
                            guard.push(item.clone());
                        }
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, list".into(),
                    found: "invalid args".into(),
                });
            }

            "insert" => {
                if args.len() == 3 {
                    if let Value::List(ref list) = args[0] {
                        let index = args[1].as_int()?;
                        let mut guard = list.lock().unwrap();
                        let idx = if index < 0 {
                            (guard.len() as i64 + index).max(0) as usize
                        } else {
                            index.min(guard.len() as i64) as usize
                        };
                        guard.insert(idx, args[2].clone());
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, index, value".into(),
                    found: "invalid args".into(),
                });
            }

            "remove" => {
                if args.len() == 2 {
                    if let Value::List(ref list) = args[0] {
                        let mut guard = list.lock().unwrap();
                        let val = &args[1];
                        if let Some(pos) = guard.iter().position(|x| values_equal(x, val)) {
                            guard.remove(pos);
                            return Ok(Value::Null);
                        }
                        return Err(LatchError::GenericError("value not found in list".into()));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, value".into(),
                    found: "invalid args".into(),
                });
            }

            "pop" => {
                if !args.is_empty() {
                    if let Value::List(ref list) = args[0] {
                        let mut guard = list.lock().unwrap();
                        if guard.is_empty() {
                            return Err(LatchError::GenericError("pop from empty list".into()));
                        }
                        let index = if args.len() >= 2 {
                            let idx = args[1].as_int()?;
                            if idx < 0 {
                                (guard.len() as i64 + idx).max(0) as usize
                            } else {
                                idx as usize
                            }
                        } else {
                            guard.len() - 1
                        };
                        if index >= guard.len() {
                            return Err(LatchError::GenericError("pop index out of range".into()));
                        }
                        return Ok(guard.remove(index));
                    }

                    if let Value::Map(ref m) = args[0] {
                        if args.len() >= 2 {
                            let mut guard = m.lock().unwrap();
                            let key = args[1].as_str()?;
                            if let Some(val) = guard.remove(key) {
                                return Ok(val);
                            }
                            if args.len() >= 3 {
                                return Ok(args[2].clone());
                            }
                            return Err(LatchError::GenericError(format!(
                                "key not found: {}",
                                key
                            )));
                        }
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, [index] or dict, key, [default]".into(),
                    found: "invalid args".into(),
                });
            }

            "list_clear" => {
                if args.len() == 1 {
                    if let Value::List(ref list) = args[0] {
                        list.lock().unwrap().clear();
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list".into(),
                    found: "invalid args".into(),
                });
            }

            "index" => {
                if args.len() == 2 {
                    if let Value::List(ref list) = args[0] {
                        let guard = list.lock().unwrap();
                        let val = &args[1];
                        if let Some(pos) = guard.iter().position(|x| values_equal(x, val)) {
                            return Ok(Value::Int(pos as i64));
                        }
                        return Err(LatchError::GenericError("value not found in list".into()));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, value".into(),
                    found: "invalid args".into(),
                });
            }

            "count" => {
                if args.len() == 2 {
                    if let Value::List(ref list) = args[0] {
                        let guard = list.lock().unwrap();
                        let val = &args[1];
                        let cnt = guard.iter().filter(|x| values_equal(x, val)).count();
                        return Ok(Value::Int(cnt as i64));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, value".into(),
                    found: "invalid args".into(),
                });
            }

            "reverse" => {
                if args.len() == 1 {
                    if let Value::List(ref list) = args[0] {
                        list.lock().unwrap().reverse();
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list".into(),
                    found: "invalid args".into(),
                });
            }

            "sum" => {
                if let Some(Value::List(ref list)) = args.first() {
                    let guard = list.lock().unwrap();
                    let mut is_float = false;
                    let mut int_sum = 0i64;
                    let mut float_sum = 0.0f64;
                    for item in guard.iter() {
                        match item {
                            Value::Int(n) => {
                                int_sum += n;
                                float_sum += *n as f64;
                            }
                            Value::Float(f) => {
                                is_float = true;
                                float_sum += f;
                            }
                            _ => {}
                        }
                    }
                    if is_float {
                        return Ok(Value::Float(float_sum));
                    } else {
                        return Ok(Value::Int(int_sum));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list".into(),
                    found: "invalid args".into(),
                });
            }

            "max" => {
                if let Some(Value::List(ref list)) = args.first() {
                    let guard = list.lock().unwrap();
                    if let Some(m) = guard.iter().max_by(|a, b| match (a, b) {
                        (Value::Int(x), Value::Int(y)) => x.cmp(y),
                        (Value::Float(x), Value::Float(y)) => {
                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Value::Str(x), Value::Str(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    }) {
                        return Ok(m.clone());
                    }
                    return Ok(Value::Null);
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list".into(),
                    found: "invalid args".into(),
                });
            }

            "min" => {
                if let Some(Value::List(ref list)) = args.first() {
                    let guard = list.lock().unwrap();
                    if let Some(m) = guard.iter().min_by(|a, b| match (a, b) {
                        (Value::Int(x), Value::Int(y)) => x.cmp(y),
                        (Value::Float(x), Value::Float(y)) => {
                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Value::Str(x), Value::Str(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    }) {
                        return Ok(m.clone());
                    }
                    return Ok(Value::Null);
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list".into(),
                    found: "invalid args".into(),
                });
            }

            "filter" => {
                if args.len() == 2 {
                    if let (
                        Value::List(ref list),
                        Value::Fn {
                            params,
                            body,
                            captured_env,
                        },
                    ) = (&args[0], &args[1])
                    {
                        let guard = list.lock().unwrap();
                        let mut filtered = Vec::new();
                        let cap = captured_env.as_ref().map(|b| (**b).clone());
                        for item in guard.iter() {
                            let res =
                                self.call_closure(params, body, vec![item.clone()], cap.clone())?;
                            if res.is_truthy() {
                                filtered.push(item.clone());
                            }
                        }
                        return Ok(Value::new_list(filtered));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, fn".into(),
                    found: "invalid args".into(),
                });
            }

            "map" => {
                if args.len() == 2 {
                    if let (
                        Value::List(ref list),
                        Value::Fn {
                            params,
                            body,
                            captured_env,
                        },
                    ) = (&args[0], &args[1])
                    {
                        let guard = list.lock().unwrap();
                        let mut mapped = Vec::new();
                        let cap = captured_env.as_ref().map(|b| (**b).clone());
                        for item in guard.iter() {
                            let res =
                                self.call_closure(params, body, vec![item.clone()], cap.clone())?;
                            mapped.push(res);
                        }
                        return Ok(Value::new_list(mapped));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, fn".into(),
                    found: "invalid args".into(),
                });
            }

            "each" => {
                if args.len() == 2 {
                    if let (
                        Value::List(ref list),
                        Value::Fn {
                            params,
                            body,
                            captured_env,
                        },
                    ) = (&args[0], &args[1])
                    {
                        let guard = list.lock().unwrap();
                        let cap = captured_env.as_ref().map(|b| (**b).clone());
                        for item in guard.iter() {
                            self.call_closure(params, body, vec![item.clone()], cap.clone())?;
                        }
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list, fn".into(),
                    found: "invalid args".into(),
                });
            }

            "sort" => {
                if args.len() == 1 {
                    if let Value::List(ref list) = args[0] {
                        let mut guard = list.lock().unwrap();
                        guard.sort_by(|a, b| match (a, b) {
                            (Value::Int(x), Value::Int(y)) => x.cmp(y),
                            (Value::Float(x), Value::Float(y)) => {
                                x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                            }
                            (Value::Str(x), Value::Str(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        });
                        return Ok(Value::new_list(guard.clone()));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list".into(),
                    found: "invalid args".into(),
                });
            }

            "list_copy" => {
                if args.len() == 1 {
                    if let Value::List(ref list) = args[0] {
                        let guard = list.lock().unwrap();
                        return Ok(Value::new_list(guard.clone()));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "list".into(),
                    found: "invalid args".into(),
                });
            }
            "split" => {
                if args.len() >= 2 {
                    let s = args[0].as_str()?;
                    let delim = args[1].as_str()?;
                    let parts: Vec<Value> =
                        s.split(delim).map(|p| Value::Str(p.to_string())).collect();
                    return Ok(Value::new_list(parts));
                }
                return Err(LatchError::TypeMismatch {
                    expected: "string, string".into(),
                    found: "invalid args".into(),
                });
            }

            "starts_with" => {
                if args.len() >= 2 {
                    let s = args[0].as_str()?;
                    let pat = args[1].as_str()?;
                    return Ok(Value::Bool(s.starts_with(pat)));
                }
                return Err(LatchError::TypeMismatch {
                    expected: "string, string".into(),
                    found: "invalid args".into(),
                });
            }
            "ends_with" => {
                if args.len() >= 2 {
                    let s = args[0].as_str()?;
                    let pat = args[1].as_str()?;
                    return Ok(Value::Bool(s.ends_with(pat)));
                }
                return Err(LatchError::TypeMismatch {
                    expected: "string, string".into(),
                    found: "invalid args".into(),
                });
            }
            "contains" => {
                if args.len() >= 2 {
                    let s = args[0].as_str()?;
                    let pat = args[1].as_str()?;
                    return Ok(Value::Bool(s.contains(pat)));
                }
                return Err(LatchError::TypeMismatch {
                    expected: "string, string".into(),
                    found: "invalid args".into(),
                });
            }
            "replace" => {
                if args.len() >= 3 {
                    let s = args[0].as_str()?;
                    let from = args[1].as_str()?;
                    let to = args[2].as_str()?;
                    return Ok(Value::Str(s.replace(from, to)));
                }
                return Err(LatchError::TypeMismatch {
                    expected: "string, string, string".into(),
                    found: "invalid args".into(),
                });
            }

            "trim" => {
                let s = args
                    .first()
                    .ok_or_else(|| LatchError::ArgCountMismatch {
                        name: "trim".into(),
                        expected: 1,
                        found: 0,
                    })?
                    .as_str()?;
                return Ok(Value::Str(s.trim().to_string()));
            }
            "upper" => {
                let s = args
                    .first()
                    .ok_or_else(|| LatchError::ArgCountMismatch {
                        name: "upper".into(),
                        expected: 1,
                        found: 0,
                    })?
                    .as_str()?;
                return Ok(Value::Str(s.to_uppercase()));
            }
            "lower" => {
                let s = args
                    .first()
                    .ok_or_else(|| LatchError::ArgCountMismatch {
                        name: "lower".into(),
                        expected: 1,
                        found: 0,
                    })?
                    .as_str()?;
                return Ok(Value::Str(s.to_lowercase()));
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

            "get" => {
                if args.len() >= 2 {
                    if let Value::Map(ref m) = args[0] {
                        let guard = m.lock().unwrap();
                        let key = args[1].as_str()?;
                        if let Some(val) = guard.get(key) {
                            return Ok(val.clone());
                        }
                        if args.len() >= 3 {
                            return Ok(args[2].clone());
                        }
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "dict, key, [default]".into(),
                    found: "invalid args".into(),
                });
            }

            "popitem" => {
                if args.len() == 1 {
                    if let Value::Map(ref m) = args[0] {
                        let mut guard = m.lock().unwrap();
                        if let Some(key) = guard.keys().next().cloned() {
                            if let Some(val) = guard.remove(&key) {
                                return Ok(Value::new_list(vec![Value::Str(key), val]));
                            }
                        }
                        return Err(LatchError::GenericError("popitem from empty dict".into()));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "dict".into(),
                    found: "invalid args".into(),
                });
            }

            "update" => {
                if args.len() == 2 {
                    if let (Value::Map(ref m), Value::Map(ref other)) = (&args[0], &args[1]) {
                        let mut guard = m.lock().unwrap();
                        let other_guard = other.lock().unwrap();
                        for (k, v) in other_guard.iter() {
                            guard.insert(k.clone(), v.clone());
                        }
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "dict, dict".into(),
                    found: "invalid args".into(),
                });
            }

            "setdefault" => {
                if args.len() == 3 {
                    if let Value::Map(ref m) = args[0] {
                        let mut guard = m.lock().unwrap();
                        let key = args[1].as_str()?;
                        if let Some(val) = guard.get(key) {
                            return Ok(val.clone());
                        }
                        guard.insert(key.to_string(), args[2].clone());
                        return Ok(args[2].clone());
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "dict, key, default".into(),
                    found: "invalid args".into(),
                });
            }

            "dict_clear" => {
                if args.len() == 1 {
                    if let Value::Map(ref m) = args[0] {
                        m.lock().unwrap().clear();
                        return Ok(Value::Null);
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "dict".into(),
                    found: "invalid args".into(),
                });
            }

            "dict_copy" => {
                if args.len() == 1 {
                    if let Value::Map(ref m) = args[0] {
                        let guard = m.lock().unwrap();
                        let copy: HashMap<String, Value> = guard.clone();
                        return Ok(Value::Map(Arc::new(Mutex::new(copy))));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "dict".into(),
                    found: "invalid args".into(),
                });
            }

            "items" => {
                if args.len() == 1 {
                    if let Value::Map(ref m) = args[0] {
                        let guard = m.lock().unwrap();
                        let mut items: Vec<Value> = Vec::new();
                        let mut sorted_keys: Vec<String> = guard.keys().cloned().collect();
                        sorted_keys.sort();
                        for key in sorted_keys {
                            if let Some(val) = guard.get(&key) {
                                items.push(Value::new_list(vec![Value::Str(key), val.clone()]));
                            }
                        }
                        return Ok(Value::new_list(items));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "dict".into(),
                    found: "invalid args".into(),
                });
            }

            "values" => {
                if args.len() == 1 {
                    if let Value::Map(ref m) = args[0] {
                        let guard = m.lock().unwrap();
                        let mut sorted_keys: Vec<String> = guard.keys().cloned().collect();
                        sorted_keys.sort();
                        let vals: Vec<Value> = sorted_keys
                            .iter()
                            .filter_map(|k| guard.get(k).cloned())
                            .collect();
                        return Ok(Value::new_list(vals));
                    }
                }
                return Err(LatchError::TypeMismatch {
                    expected: "dict".into(),
                    found: "invalid args".into(),
                });
            }

            _ => {}
        }

        // Look up in environment
        let val = self
            .env
            .get(name)
            .ok_or_else(|| LatchError::UndefinedFunction(name.to_string()))?;

        match val {
            Value::Fn {
                params,
                body,
                captured_env,
            } => self.call_closure(&params, &body, args, captured_env.map(|e| *e)),
            Value::Class {
                name: class_name,
                fields,
                methods,
            } => self.instantiate_class(&class_name, &fields, &methods, args),
            _ => Err(LatchError::TypeMismatch {
                expected: "function or class".into(),
                found: val.type_name().into(),
            }),
        }
    }

    pub(crate) fn instantiate_class(
        &mut self,
        class_name: &str,
        fields: &[(String, Option<crate::ast::Type>, Option<Block>)],
        methods: &[(String, Vec<Param>, Block)],
        args: Vec<Value>,
    ) -> Result<Value> {
        let mut instance_fields = HashMap::new();
        for (fname, _ftype, fdefault) in fields {
            let val = if let Some(block) = fdefault {
                match self.exec_block_returning(block.clone()) {
                    Ok(v) => v,
                    Err(_) => Value::Null,
                }
            } else {
                Value::Null
            };
            instance_fields.insert(fname.clone(), val);
        }
        let instance = Value::Instance {
            class_name: class_name.to_string(),
            fields: Arc::new(Mutex::new(instance_fields)),
            methods: Arc::new(methods.to_vec()),
        };

        if let Some((_, params, body)) = methods.iter().find(|(n, _, _)| n == "init") {
            let mut call_args = vec![instance.clone()];
            call_args.extend(args);
            self.call_closure(params, body, call_args, None)?;
        }
        Ok(instance)
    }

    pub(crate) fn call_method(
        &mut self,
        recv: Value,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value> {
        match &recv {
            Value::Instance {
                class_name,
                methods,
                ..
            } => {
                let methods_snap = methods.clone();
                if let Some((_, params, body)) = methods_snap.iter().find(|(n, _, _)| n == method) {
                    let mut call_args = vec![recv.clone()];
                    call_args.extend(args);
                    self.call_closure(params, body, call_args, None)
                } else {
                    Err(LatchError::UnknownMethod {
                        module: class_name.clone(),
                        method: method.to_string(),
                    })
                }
            }
            Value::Str(s) => {
                let s = s.clone();
                match method {
                    "upper" => Ok(Value::Str(s.to_uppercase())),
                    "lower" => Ok(Value::Str(s.to_lowercase())),
                    "trim" => Ok(Value::Str(s.trim().to_string())),
                    "len" => Ok(Value::Int(s.len() as i64)),
                    "split" => {
                        let delim = args
                            .first()
                            .map(|v| v.as_str().unwrap_or(""))
                            .unwrap_or("")
                            .to_string();
                        Ok(Value::new_list(
                            s.split(&delim).map(|p| Value::Str(p.to_string())).collect(),
                        ))
                    }
                    "starts_with" => Ok(Value::Bool(
                        s.starts_with(args.first().and_then(|v| v.as_str().ok()).unwrap_or("")),
                    )),
                    "ends_with" => Ok(Value::Bool(
                        s.ends_with(args.first().and_then(|v| v.as_str().ok()).unwrap_or("")),
                    )),
                    "contains" => Ok(Value::Bool(
                        s.contains(args.first().and_then(|v| v.as_str().ok()).unwrap_or("")),
                    )),
                    "replace" => {
                        if args.len() >= 2 {
                            let from = args[0].as_str()?;
                            let to = args[1].as_str()?;
                            Ok(Value::Str(s.replace(from, to)))
                        } else {
                            Err(LatchError::ArgCountMismatch {
                                name: "str.replace".into(),
                                expected: 2,
                                found: args.len(),
                            })
                        }
                    }
                    _ => Err(LatchError::UnknownMethod {
                        module: "string".into(),
                        method: method.to_string(),
                    }),
                }
            }
            Value::List(list) => {
                let list = list.clone();
                match method {
                    "len" => Ok(Value::Int(list.lock().unwrap().len() as i64)),
                    "push" => {
                        if let Some(v) = args.into_iter().next() {
                            list.lock().unwrap().push(v);
                        }
                        Ok(Value::Null)
                    }
                    "pop" => {
                        let mut guard = list.lock().unwrap();
                        guard
                            .pop()
                            .ok_or_else(|| LatchError::GenericError("pop from empty list".into()))
                    }
                    "reverse" => {
                        list.lock().unwrap().reverse();
                        Ok(Value::Null)
                    }
                    "sort" => {
                        let mut vec = list.lock().unwrap().clone();
                        vec.sort_by(|a, b| match (a, b) {
                            (Value::Int(x), Value::Int(y)) => x.cmp(y),
                            (Value::Float(x), Value::Float(y)) => {
                                x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                            }
                            (Value::Str(x), Value::Str(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        });
                        *list.lock().unwrap() = vec;
                        Ok(Value::Null)
                    }
                    "contains" => {
                        let needle = args.into_iter().next().unwrap_or(Value::Null);
                        let guard = list.lock().unwrap();
                        Ok(Value::Bool(guard.iter().any(|x| values_equal(x, &needle))))
                    }
                    _ => Err(LatchError::UnknownMethod {
                        module: "list".into(),
                        method: method.to_string(),
                    }),
                }
            }
            Value::Map(map) => {
                let map = map.clone();
                match method {
                    "keys" => {
                        let guard = map.lock().unwrap();
                        let mut keys: Vec<String> = guard.keys().cloned().collect();
                        keys.sort();
                        Ok(Value::new_list(keys.into_iter().map(Value::Str).collect()))
                    }
                    "values" => {
                        let guard = map.lock().unwrap();
                        let mut sorted_keys: Vec<String> = guard.keys().cloned().collect();
                        sorted_keys.sort();
                        Ok(Value::new_list(
                            sorted_keys.iter().map(|k| guard[k].clone()).collect(),
                        ))
                    }
                    "get" => {
                        let key = args
                            .first()
                            .and_then(|v| v.as_str().ok().map(|s| s.to_string()))
                            .unwrap_or_default();
                        let guard = map.lock().unwrap();
                        Ok(guard.get(&key).cloned().unwrap_or(Value::Null))
                    }
                    "has" | "contains" => {
                        let key = args
                            .first()
                            .and_then(|v| v.as_str().ok().map(|s| s.to_string()))
                            .unwrap_or_default();
                        Ok(Value::Bool(map.lock().unwrap().contains_key(&key)))
                    }
                    _ => Err(LatchError::UnknownMethod {
                        module: "dict".into(),
                        method: method.to_string(),
                    }),
                }
            }
            other => Err(LatchError::TypeMismatch {
                expected: "instance, string, list, or dict".into(),
                found: other.type_name().into(),
            }),
        }
    }

    pub(crate) fn call_closure(
        &mut self,
        params: &[Param],
        body: &Block,
        args: Vec<Value>,
        captured_env: Option<Env>,
    ) -> Result<Value> {
        let caller_env = std::mem::replace(&mut self.env, Env::new());

        self.env = match captured_env {
            Some(cap) => cap.child(),
            None => caller_env.clone().child(),
        };

        for (i, param) in params.iter().enumerate() {
            if i < args.len() {
                self.env.set(&param.name, args[i].clone());
            } else if let Some(ref default_expr) = param.default {
                let default_val = self.eval_expr(default_expr.clone())?;
                self.env.set(&param.name, default_val);
            } else {
                return Err(LatchError::ArgCountMismatch {
                    name: param.name.clone(),
                    expected: params.len(),
                    found: args.len(),
                });
            }
        }

        let result = self.exec_block_inner(body.clone());

        self.env = caller_env;

        match result {
            Ok(()) => Ok(Value::Null),
            Err(LatchError::ReturnSignal(val)) => Ok(val),
            Err(e) => Err(e),
        }
    }
}
