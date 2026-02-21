use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "read" => {
            let path = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "csv.read".into(), expected: 1, found: 0 })?
                .as_str()?;
            let content = std::fs::read_to_string(path)
                .map_err(|e| LatchError::IoError(format!("csv.read(\"{}\"): {}", path, e)))?;
            
            let mut rows: Vec<Value> = Vec::new();
            for line in content.lines() {
                let cells: Vec<Value> = line.split(',')
                    .map(|s| Value::Str(s.trim().to_string()))
                    .collect();
                rows.push(Value::new_list(cells));
            }
            Ok(Value::new_list(rows))
        }

        "write" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "csv.write".into(), expected: 2, found: args.len() });
            }
            let path = args[0].as_str()?;
            let rows = args[1].clone().into_list()?;
            
            let mut lines: Vec<String> = Vec::new();
            for row in rows {
                let cells = row.into_list()?;
                let line: Vec<String> = cells.iter()
                    .map(|v| format!("{}", v))
                    .collect();
                lines.push(line.join(","));
            }
            
            std::fs::write(path, lines.join("\n"))
                .map_err(|e| LatchError::IoError(format!("csv.write(\"{}\"): {}", path, e)))?;
            Ok(Value::Bool(true))
        }

        "parse" => {
            let text = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "csv.parse".into(), expected: 1, found: 0 })?
                .as_str()?;
            
            let mut rows: Vec<Value> = Vec::new();
            for line in text.lines() {
                let cells: Vec<Value> = line.split(',')
                    .map(|s| Value::Str(s.trim().to_string()))
                    .collect();
                rows.push(Value::new_list(cells));
            }
            Ok(Value::new_list(rows))
        }

        "stringify" => {
            let rows = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "csv.stringify".into(), expected: 1, found: 0 })?
                .clone()
                .into_list()?;
            
            let mut lines: Vec<String> = Vec::new();
            for row in rows {
                let cells = row.into_list()?;
                let line: Vec<String> = cells.iter()
                    .map(|v| format!("{}", v))
                    .collect();
                lines.push(line.join(","));
            }
            
            Ok(Value::Str(lines.join("\n")))
        }

        _ => Err(LatchError::UnknownMethod { module: "csv".into(), method: method.into() }),
    }
}
