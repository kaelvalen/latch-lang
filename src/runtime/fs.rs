use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "read" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.read".into(), expected: 1, found: 0 })?
                .as_str()?;
            let content = std::fs::read_to_string(path)
                .map_err(|e| LatchError::IoError(format!("fs.read(\"{}\"): {}", path, e)))?;
            Ok(Value::Str(content))
        }

        "write" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "fs.write".into(), expected: 2, found: args.len() });
            }
            let path = args[0].as_str()?;
            let data = args[1].as_str()?;
            std::fs::write(path, data)
                .map_err(|e| LatchError::IoError(format!("fs.write(\"{}\"): {}", path, e)))?;
            Ok(Value::Bool(true))
        }

        "exists" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.exists".into(), expected: 1, found: 0 })?
                .as_str()?;
            Ok(Value::Bool(std::path::Path::new(path).exists()))
        }

        "glob" => {
            let pattern = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.glob".into(), expected: 1, found: 0 })?
                .as_str()?;
            let entries = glob::glob(pattern)
                .map_err(|e| LatchError::IoError(format!("fs.glob(\"{}\"): {}", pattern, e)))?;
            let mut list = Vec::new();
            for entry in entries {
                match entry {
                    Ok(path) => list.push(Value::Str(path.display().to_string())),
                    Err(e) => list.push(Value::Str(format!("error: {}", e))),
                }
            }
            Ok(Value::new_list(list))
        }

        "append" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "fs.append".into(), expected: 2, found: args.len() });
            }
            let path = args[0].as_str()?;
            let data = args[1].as_str()?;
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|e| LatchError::IoError(format!("fs.append(\"{}\"): {}", path, e)))?;
            file.write_all(data.as_bytes())
                .map_err(|e| LatchError::IoError(format!("fs.append(\"{}\"): {}", path, e)))?;
            Ok(Value::Bool(true))
        }

        "readlines" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.readlines".into(), expected: 1, found: 0 })?
                .as_str()?;
            let content = std::fs::read_to_string(path)
                .map_err(|e| LatchError::IoError(format!("fs.readlines(\"{}\"): {}", path, e)))?;
            let lines: Vec<Value> = content.lines()
                .map(|l| Value::Str(l.to_string()))
                .collect();
            Ok(Value::new_list(lines))
        }

        "mkdir" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.mkdir".into(), expected: 1, found: 0 })?
                .as_str()?;
            let recursive = args.get(1).map(|v| v.is_truthy()).unwrap_or(false);
            if recursive {
                std::fs::create_dir_all(path)
                    .map_err(|e| LatchError::IoError(format!("fs.mkdir(\"{}\"): {}", path, e)))?;
            } else {
                std::fs::create_dir(path)
                    .map_err(|e| LatchError::IoError(format!("fs.mkdir(\"{}\"): {}", path, e)))?;
            }
            Ok(Value::Bool(true))
        }

        "remove" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.remove".into(), expected: 1, found: 0 })?
                .as_str()?;
            let p = std::path::Path::new(path);
            if p.is_dir() {
                std::fs::remove_dir_all(path)
                    .map_err(|e| LatchError::IoError(format!("fs.remove(\"{}\"): {}", path, e)))?;
            } else {
                std::fs::remove_file(path)
                    .map_err(|e| LatchError::IoError(format!("fs.remove(\"{}\"): {}", path, e)))?;
            }
            Ok(Value::Bool(true))
        }

        "rmdir" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.rmdir".into(), expected: 1, found: 0 })?
                .as_str()?;
            let recursive = args.get(1).map(|v| v.is_truthy()).unwrap_or(false);
            if recursive {
                std::fs::remove_dir_all(path)
                    .map_err(|e| LatchError::IoError(format!("fs.rmdir(\"{}\"): {}", path, e)))?;
            } else {
                std::fs::remove_dir(path)
                    .map_err(|e| LatchError::IoError(format!("fs.rmdir(\"{}\"): {}", path, e)))?;
            }
            Ok(Value::Bool(true))
        }

        "rename" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "fs.rename".into(), expected: 2, found: args.len() });
            }
            let from = args[0].as_str()?;
            let to = args[1].as_str()?;
            std::fs::rename(from, to)
                .map_err(|e| LatchError::IoError(format!("fs.rename(\"{}\", \"{}\"): {}", from, to, e)))?;
            Ok(Value::Bool(true))
        }

        "stat" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.stat".into(), expected: 1, found: 0 })?
                .as_str()?;
            let meta = std::fs::metadata(path)
                .map_err(|e| LatchError::IoError(format!("fs.stat(\"{}\"): {}", path, e)))?;
            let mut map = std::collections::HashMap::new();
            map.insert("size".to_string(), Value::Int(meta.len() as i64));
            map.insert("is_file".to_string(), Value::Bool(meta.is_file()));
            map.insert("is_dir".to_string(), Value::Bool(meta.is_dir()));
            map.insert("readonly".to_string(), Value::Bool(meta.permissions().readonly()));
            Ok(Value::new_map(map))
        }

        "copy" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "fs.copy".into(), expected: 2, found: args.len() });
            }
            let src = args[0].as_str()?;
            let dst = args[1].as_str()?;
            std::fs::copy(src, dst)
                .map_err(|e| LatchError::IoError(format!("fs.copy(\"{}\", \"{}\"): {}", src, dst, e)))?;
            Ok(Value::Bool(true))
        }

        "move" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "fs.move".into(), expected: 2, found: args.len() });
            }
            let src = args[0].as_str()?;
            let dst = args[1].as_str()?;
            std::fs::rename(src, dst)
                .map_err(|e| LatchError::IoError(format!("fs.move(\"{}\", \"{}\"): {}", src, dst, e)))?;
            Ok(Value::Bool(true))
        }

        "isfile" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.isfile".into(), expected: 1, found: 0 })?
                .as_str()?;
            Ok(Value::Bool(std::path::Path::new(path).is_file()))
        }

        "isdir" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.isdir".into(), expected: 1, found: 0 })?
                .as_str()?;
            Ok(Value::Bool(std::path::Path::new(path).is_dir()))
        }

        "listdir" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.listdir".into(), expected: 1, found: 0 })?
                .as_str()?;
            let entries = std::fs::read_dir(path)
                .map_err(|e| LatchError::IoError(format!("fs.listdir(\"{}\"): {}", path, e)))?;
            let mut list = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    list.push(Value::Str(entry.path().display().to_string()));
                }
            }
            Ok(Value::new_list(list))
        }

        "walk" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "fs.walk".into(), expected: 1, found: 0 })?
                .as_str()?;
            let mut result = Vec::new();
            fn walk_dir(dir: &str, result: &mut Vec<Value>) -> Result<()> {
                for entry in std::fs::read_dir(dir)
                    .map_err(|e| LatchError::IoError(format!("fs.walk(\"{}\"): {}", dir, e)))? {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        result.push(Value::Str(path.display().to_string()));
                        if path.is_dir() {
                            walk_dir(&path.display().to_string(), result)?;
                        }
                    }
                }
                Ok(())
            }
            walk_dir(path, &mut result)?;
            Ok(Value::new_list(result))
        }

        _ => Err(LatchError::UnknownMethod { module: "fs".into(), method: method.into() }),
    }
}
