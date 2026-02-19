use std::path::Path;

use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "join" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch {
                    name: "path.join".into(), expected: 2, found: args.len(),
                });
            }
            let base = args[0].as_str()?.to_string();
            let rest = args[1].as_str()?.to_string();
            let joined = Path::new(&base).join(&rest);
            Ok(Value::Str(joined.display().to_string()))
        }

        "basename" => {
            let p = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch {
                    name: "path.basename".into(), expected: 1, found: 0,
                })?
                .as_str()?
                .to_string();
            let name = Path::new(&p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            Ok(Value::Str(name))
        }

        "dirname" => {
            let p = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch {
                    name: "path.dirname".into(), expected: 1, found: 0,
                })?
                .as_str()?
                .to_string();
            let dir = Path::new(&p)
                .parent()
                .map(|d| d.display().to_string())
                .unwrap_or_default();
            Ok(Value::Str(dir))
        }

        "ext" => {
            let p = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch {
                    name: "path.ext".into(), expected: 1, found: 0,
                })?
                .as_str()?
                .to_string();
            let ext = Path::new(&p)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();
            Ok(Value::Str(ext))
        }

        "abs" => {
            let p = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch {
                    name: "path.abs".into(), expected: 1, found: 0,
                })?
                .as_str()?
                .to_string();
            let abs = std::fs::canonicalize(&p)
                .map_err(|e| LatchError::IoError(format!("path.abs(\"{p}\"): {e}")))?;
            Ok(Value::Str(abs.display().to_string()))
        }

        _ => Err(LatchError::UnknownMethod {
            module: "path".into(), method: method.into(),
        }),
    }
}
