use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "md5" => {
            let data = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "hash.md5".into(), expected: 1, found: 0 })?
                .as_str()?;
            let result = format!("{:x}", md5::compute(data.as_bytes()));
            Ok(Value::Str(result))
        }

        "sha256" => {
            use sha2::{Sha256, Digest};
            let data = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "hash.sha256".into(), expected: 1, found: 0 })?
                .as_str()?;
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            let result = format!("{:x}", hasher.finalize());
            Ok(Value::Str(result))
        }

        "sha512" => {
            use sha2::{Sha512, Digest};
            let data = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "hash.sha512".into(), expected: 1, found: 0 })?
                .as_str()?;
            let mut hasher = Sha512::new();
            hasher.update(data.as_bytes());
            let result = format!("{:x}", hasher.finalize());
            Ok(Value::Str(result))
        }

        _ => Err(LatchError::UnknownMethod { module: "hash".into(), method: method.into() }),
    }
}
