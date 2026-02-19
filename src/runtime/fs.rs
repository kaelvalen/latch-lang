use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "read" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.read".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();
            let content = std::fs::read_to_string(&path)
                .map_err(|e| LatchError::IoError(format!("fs.read(\"{path}\"): {e}")))?;
            Ok(Value::Str(content))
        }

        "write" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "fs.write".into(), expected: 2, found: args.len() });
            }
            let path = args[0].as_str()?.to_string();
            let data = args[1].as_str()?.to_string();
            std::fs::write(&path, &data)
                .map_err(|e| LatchError::IoError(format!("fs.write(\"{path}\"): {e}")))?;
            Ok(Value::Bool(true))
        }

        "exists" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.exists".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();
            Ok(Value::Bool(std::path::Path::new(&path).exists()))
        }

        "glob" => {
            let pattern = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.glob".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();
            let entries = glob::glob(&pattern)
                .map_err(|e| LatchError::IoError(format!("fs.glob(\"{pattern}\"): {e}")))?;
            let mut list = Vec::new();
            for entry in entries {
                match entry {
                    Ok(path) => list.push(Value::Str(path.display().to_string())),
                    Err(e) => list.push(Value::Str(format!("error: {e}"))),
                }
            }
            Ok(Value::List(list))
        }

        _ => Err(LatchError::UnknownMethod { module: "fs".into(), method: method.into() }),
    }
}
