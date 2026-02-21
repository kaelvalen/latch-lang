use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "encode" => {
            let data = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "base64.encode".into(), expected: 1, found: 0 })?
                .as_str()?;
            let encoded = base64::encode(data.as_bytes());
            Ok(Value::Str(encoded))
        }

        "decode" => {
            let data = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "base64.decode".into(), expected: 1, found: 0 })?
                .as_str()?;
            let decoded = base64::decode(data)
                .map_err(|e| LatchError::GenericError(format!("Base64 decode error: {}", e)))?;
            Ok(Value::Str(String::from_utf8_lossy(&decoded).to_string()))
        }

        _ => Err(LatchError::UnknownMethod { module: "base64".into(), method: method.into() }),
    }
}
