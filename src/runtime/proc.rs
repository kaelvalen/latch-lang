use std::process::Command;

use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "exec" => {
            let cmd_str = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "proc.exec".into(), expected: 1, found: 0 })?
                .as_str()?
                .to_string();

            let output = if cfg!(target_os = "windows") {
                Command::new("cmd").args(["/C", &cmd_str]).output()
            } else {
                Command::new("sh").args(["-c", &cmd_str]).output()
            };

            let output = output.map_err(|e| LatchError::IoError(format!("proc.exec: {e}")))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let code = output.status.code().unwrap_or(-1);

            Ok(Value::ProcessResult { stdout, stderr, code })
        }

        "pipe" => {
            let cmds = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "proc.pipe".into(), expected: 1, found: 0 })?
                .as_list()?
                .clone();

            let mut input = String::new();

            for cmd_val in &cmds {
                let cmd_str = cmd_val.as_str()?.to_string();

                let mut child = if cfg!(target_os = "windows") {
                    Command::new("cmd").args(["/C", &cmd_str])
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .map_err(|e| LatchError::IoError(format!("proc.pipe: {e}")))?
                } else {
                    Command::new("sh").args(["-c", &cmd_str])
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::piped())
                        .spawn()
                        .map_err(|e| LatchError::IoError(format!("proc.pipe: {e}")))?
                };

                if !input.is_empty() {
                    use std::io::Write;
                    if let Some(ref mut stdin) = child.stdin {
                        stdin.write_all(input.as_bytes())
                            .map_err(|e| LatchError::IoError(format!("proc.pipe stdin: {e}")))?;
                    }
                }
                // Close stdin so the process can proceed
                drop(child.stdin.take());

                let output = child.wait_with_output()
                    .map_err(|e| LatchError::IoError(format!("proc.pipe: {e}")))?;

                input = String::from_utf8_lossy(&output.stdout).to_string();

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    let code = output.status.code().unwrap_or(-1);
                    return Ok(Value::ProcessResult { stdout: input, stderr, code });
                }
            }

            Ok(Value::ProcessResult {
                stdout: input,
                stderr: String::new(),
                code: 0,
            })
        }

        _ => Err(LatchError::UnknownMethod { module: "proc".into(), method: method.into() }),
    }
}
