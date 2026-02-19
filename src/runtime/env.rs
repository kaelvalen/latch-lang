use std::collections::HashMap;

use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "get" => {
            let key = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch {
                    name: "env.get".into(), expected: 1, found: 0,
                })?
                .as_str()?
                .to_string();
            match std::env::var(&key) {
                Ok(val) => Ok(Value::Str(val)),
                Err(_) => Err(LatchError::KeyNotFound(key)),
            }
        }

        "set" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch {
                    name: "env.set".into(), expected: 2, found: args.len(),
                });
            }
            let key = args[0].as_str()?.to_string();
            let val = args[1].as_str()?.to_string();
            // SAFETY: env.set() only affects the current Latch process and
            // child processes spawned via proc.exec(). It does NOT propagate
            // to the parent shell.
            unsafe { std::env::set_var(&key, &val); }
            Ok(Value::Bool(true))
        }

        "list" => {
            let map: HashMap<String, Value> = std::env::vars()
                .map(|(k, v)| (k, Value::Str(v)))
                .collect();
            Ok(Value::new_map(map))
        }

        _ => Err(LatchError::UnknownMethod {
            module: "env".into(), method: method.into(),
        }),
    }
}
