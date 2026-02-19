use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "sleep" => {
            let ms = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "time.sleep".into(), expected: 1, found: 0 })?
                .as_int()?;
            std::thread::sleep(std::time::Duration::from_millis(ms as u64));
            Ok(Value::Null)
        }

        "now" => {
            let now = chrono::Utc::now().to_rfc3339();
            Ok(Value::Str(now))
        }

        _ => Err(LatchError::UnknownMethod { module: "time".into(), method: method.into() }),
    }
}
