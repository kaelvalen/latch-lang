use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "sqrt" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.sqrt".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.sqrt()))
        }

        "abs" => {
            match args.first() {
                Some(Value::Int(n)) => Ok(Value::Int(n.abs())),
                Some(Value::Float(n)) => Ok(Value::Float(n.abs())),
                _ => Err(LatchError::TypeMismatch { expected: "number".into(), found: args.first().map(|v| v.type_name()).unwrap_or("none").into() }),
            }
        }

        "floor" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.floor".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.floor()))
        }

        "ceil" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.ceil".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.ceil()))
        }

        "round" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.round".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.round()))
        }

        "pow" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "math.pow".into(), expected: 2, found: args.len() });
            }
            let base = args[0].as_float()?;
            let exp = args[1].as_float()?;
            Ok(Value::Float(base.powf(exp)))
        }

        "sin" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.sin".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.sin()))
        }

        "cos" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.cos".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.cos()))
        }

        "tan" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.tan".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.tan()))
        }

        "log" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.log".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.ln()))
        }

        "log10" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.log10".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.log10()))
        }

        "exp" => {
            let n = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "math.exp".into(), expected: 1, found: 0 })?
                .as_float()?;
            Ok(Value::Float(n.exp()))
        }

        "pi" => {
            Ok(Value::Float(std::f64::consts::PI))
        }

        "e" => {
            Ok(Value::Float(std::f64::consts::E))
        }

        "random" => {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            use std::time::{SystemTime, UNIX_EPOCH};
            
            let mut hasher = DefaultHasher::new();
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
            let random_val = (hasher.finish() as f64) / (u64::MAX as f64);
            Ok(Value::Float(random_val))
        }

        _ => Err(LatchError::UnknownMethod { module: "math".into(), method: method.into() }),
    }
}
