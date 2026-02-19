use std::collections::HashMap;

use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "parse" => {
            let s = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch {
                    name: "json.parse".into(), expected: 1, found: 0,
                })?
                .as_str()?
                .to_string();
            let json_val: serde_json::Value = serde_json::from_str(&s)
                .map_err(|e| LatchError::GenericError(format!("json.parse: {e}")))?;
            Ok(json_to_latch(json_val))
        }

        "stringify" => {
            let val = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch {
                    name: "json.stringify".into(), expected: 1, found: 0,
                })?;
            let json_val = latch_to_json(val);
            let s = serde_json::to_string_pretty(&json_val)
                .map_err(|e| LatchError::GenericError(format!("json.stringify: {e}")))?;
            Ok(Value::Str(s))
        }

        _ => Err(LatchError::UnknownMethod {
            module: "json".into(), method: method.into(),
        }),
    }
}

/// Convert a serde_json::Value into a Latch Value.
fn json_to_latch(val: serde_json::Value) -> Value {
    match val {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::Str(s),
        serde_json::Value::Array(arr) => {
            Value::List(arr.into_iter().map(json_to_latch).collect())
        }
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, Value> = obj.into_iter()
                .map(|(k, v)| (k, json_to_latch(v)))
                .collect();
            Value::Map(map)
        }
    }
}

/// Convert a Latch Value into a serde_json::Value.
fn latch_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => serde_json::json!(*n),
        Value::Float(n) => serde_json::json!(*n),
        Value::Str(s) => serde_json::Value::String(s.clone()),
        Value::List(items) => {
            serde_json::Value::Array(items.iter().map(latch_to_json).collect())
        }
        Value::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map.iter()
                .map(|(k, v)| (k.clone(), latch_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Fn { .. } => serde_json::Value::String("<fn>".into()),
        Value::ProcessResult { stdout, stderr, code } => {
            serde_json::json!({
                "stdout": stdout,
                "stderr": stderr,
                "code": code,
            })
        }
        Value::HttpResponse { status, body, headers } => {
            serde_json::json!({
                "status": status,
                "body": body,
                "headers": headers,
            })
        }
    }
}
