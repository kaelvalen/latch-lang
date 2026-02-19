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

        "append" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "fs.append".into(), expected: 2, found: args.len() });
            }
            let path = args[0].as_str()?.to_string();
            let data = args[1].as_str()?.to_string();
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| LatchError::IoError(format!("fs.append(\"{path}\"): {e}")))?;
            file.write_all(data.as_bytes())
                .map_err(|e| LatchError::IoError(format!("fs.append(\"{path}\"): {e}")))?;
            Ok(Value::Bool(true))
        }

        "readlines" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.readlines".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();
            let content = std::fs::read_to_string(&path)
                .map_err(|e| LatchError::IoError(format!("fs.readlines(\"{path}\"): {e}")))?;
            let lines: Vec<Value> = content.lines()
                .map(|l| Value::Str(l.to_string()))
                .collect();
            Ok(Value::List(lines))
        }

        "mkdir" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.mkdir".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();
            std::fs::create_dir_all(&path)
                .map_err(|e| LatchError::IoError(format!("fs.mkdir(\"{path}\"): {e}")))?;
            Ok(Value::Bool(true))
        }

        "remove" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.remove".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();
            let p = std::path::Path::new(&path);
            if p.is_dir() {
                std::fs::remove_dir_all(&path)
                    .map_err(|e| LatchError::IoError(format!("fs.remove(\"{path}\"): {e}")))?;
            } else {
                std::fs::remove_file(&path)
                    .map_err(|e| LatchError::IoError(format!("fs.remove(\"{path}\"): {e}")))?;
            }
            Ok(Value::Bool(true))
        }

        "stat" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.stat".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();
            let meta = std::fs::metadata(&path)
                .map_err(|e| LatchError::IoError(format!("fs.stat(\"{path}\"): {e}")))?;
            let mut map = std::collections::HashMap::new();
            map.insert("size".to_string(), Value::Int(meta.len() as i64));
            map.insert("is_file".to_string(), Value::Bool(meta.is_file()));
            map.insert("is_dir".to_string(), Value::Bool(meta.is_dir()));
            map.insert("readonly".to_string(), Value::Bool(meta.permissions().readonly()));
            Ok(Value::Map(map))
        }

        _ => Err(LatchError::UnknownMethod { module: "fs".into(), method: method.into() }),
    }
}
