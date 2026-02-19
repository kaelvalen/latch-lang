use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    let key = std::env::var("LATCH_AI_KEY")
        .map_err(|_| LatchError::AiError("LATCH_AI_KEY not set. Set it with: export LATCH_AI_KEY=your_key".into()))?;

    let prompt = match method {
        "ask" => {
            args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "ai.ask".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string()
        }
        "summarize" => {
            let text = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "ai.summarize".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();
            format!("Summarize the following:\n\n{text}")
        }
        _ => return Err(LatchError::UnknownMethod { module: "ai".into(), method: method.into() }),
    };

    let response = reqwest::blocking::Client::new()
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 1024,
            "messages": [{ "role": "user", "content": prompt }]
        }))
        .send()
        .map_err(|e| LatchError::AiError(format!("Request failed: {e}")))?
        .json::<serde_json::Value>()
        .map_err(|e| LatchError::AiError(format!("Invalid JSON response: {e}")))?;

    let text = response["content"][0]["text"]
        .as_str()
        .ok_or_else(|| LatchError::AiError(format!("Invalid response structure: {response}")))?
        .to_string();

    Ok(Value::Str(text))
}
