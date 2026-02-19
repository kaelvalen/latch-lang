use std::collections::HashMap;

use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "get" => {
            let url = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "http.get".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();

            let response = reqwest::blocking::get(&url)
                .map_err(|e| LatchError::HttpError(format!("http.get(\"{url}\"): {e}")))?;

            let status = response.status().as_u16() as i64;
            let body = response.text()
                .map_err(|e| LatchError::HttpError(format!("http.get response: {e}")))?;

            let mut map = HashMap::new();
            map.insert("body".to_string(), Value::Str(body));
            map.insert("status".to_string(), Value::Int(status));

            Ok(Value::Map(map))
        }

        "post" => {
            if args.len() < 2 {
                return Err(LatchError::ArgCountMismatch { name: "http.post".into(), expected: 2, found: args.len() });
            }
            let url = args[0].as_str()?.to_string();
            let data = args[1].as_str()?.to_string();

            let client = reqwest::blocking::Client::new();
            let response = client.post(&url)
                .header("Content-Type", "application/json")
                .body(data)
                .send()
                .map_err(|e| LatchError::HttpError(format!("http.post(\"{url}\"): {e}")))?;

            let status = response.status().as_u16() as i64;
            let body = response.text()
                .map_err(|e| LatchError::HttpError(format!("http.post response: {e}")))?;

            let mut map = HashMap::new();
            map.insert("body".to_string(), Value::Str(body));
            map.insert("status".to_string(), Value::Int(status));

            Ok(Value::Map(map))
        }

        _ => Err(LatchError::UnknownMethod { module: "http".into(), method: method.into() }),
    }
}
