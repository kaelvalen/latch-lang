use crate::env::Value;
use crate::error::{LatchError, Result};
use regex::Regex;

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "match" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "regex.match".into(), expected: 2, found: args.len() });
            }
            let pattern = args[0].as_str()?;
            let text = args[1].as_str()?;
            let re = Regex::new(pattern)
                .map_err(|e| LatchError::GenericError(format!("Invalid regex pattern: {}", e)))?;
            Ok(Value::Bool(re.is_match(text)))
        }

        "search" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "regex.search".into(), expected: 2, found: args.len() });
            }
            let pattern = args[0].as_str()?;
            let text = args[1].as_str()?;
            let re = Regex::new(pattern)
                .map_err(|e| LatchError::GenericError(format!("Invalid regex pattern: {}", e)))?;
            if let Some(mat) = re.find(text) {
                let mut result = std::collections::HashMap::new();
                result.insert("match".to_string(), Value::Str(mat.as_str().to_string()));
                result.insert("start".to_string(), Value::Int(mat.start() as i64));
                result.insert("end".to_string(), Value::Int(mat.end() as i64));
                Ok(Value::new_map(result))
            } else {
                Ok(Value::Null)
            }
        }

        "findall" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "regex.findall".into(), expected: 2, found: args.len() });
            }
            let pattern = args[0].as_str()?;
            let text = args[1].as_str()?;
            let re = Regex::new(pattern)
                .map_err(|e| LatchError::GenericError(format!("Invalid regex pattern: {}", e)))?;
            let matches: Vec<Value> = re.find_iter(text)
                .map(|m| Value::Str(m.as_str().to_string()))
                .collect();
            Ok(Value::new_list(matches))
        }

        "split" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "regex.split".into(), expected: 2, found: args.len() });
            }
            let pattern = args[0].as_str()?;
            let text = args[1].as_str()?;
            let re = Regex::new(pattern)
                .map_err(|e| LatchError::GenericError(format!("Invalid regex pattern: {}", e)))?;
            let parts: Vec<Value> = re.split(text)
                .map(|s| Value::Str(s.to_string()))
                .collect();
            Ok(Value::new_list(parts))
        }

        "replace" => {
            if args.len() < 3 {
                return Err(LatchError::ArgCountMismatch { name: "regex.replace".into(), expected: 3, found: args.len() });
            }
            let pattern = args[0].as_str()?;
            let replacement = args[1].as_str()?;
            let text = args[2].as_str()?;
            let re = Regex::new(pattern)
                .map_err(|e| LatchError::GenericError(format!("Invalid regex pattern: {}", e)))?;
            Ok(Value::Str(re.replace_all(text, replacement).to_string()))
        }

        _ => Err(LatchError::UnknownMethod { module: "regex".into(), method: method.into() }),
    }
}
