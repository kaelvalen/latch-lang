use std::process::Command;

use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "exec" => {
            let arg = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "proc.exec".into(), expected: 1, found: 0 })?;

            let output = match arg {
                // Array form: proc.exec(["git", "status"]) — no shell, direct exec
                Value::List(items) => {
                    let items = items.lock().unwrap();
                    if items.is_empty() {
                        return Err(LatchError::GenericError("proc.exec: empty command list".into()));
                    }
                    let cmd_parts: Vec<String> = items.iter()
                        .map(|v| match v {
                            Value::Str(s) => Ok(s.clone()),
                            other => Err(LatchError::TypeMismatch {
                                expected: "string".into(),
                                found: other.type_name().into(),
                            }),
                        })
                        .collect::<Result<_>>()?;
                    Command::new(&cmd_parts[0])
                        .args(&cmd_parts[1..])
                        .output()
                        .map_err(|e| LatchError::IoError(format!("proc.exec: {e}")))?
                }
                // String form: proc.exec("ls -la") — via shell
                Value::Str(cmd_str) => {
                    if cfg!(target_os = "windows") {
                        Command::new("cmd").args(["/C", cmd_str]).output()
                    } else {
                        Command::new("sh").args(["-c", cmd_str]).output()
                    }.map_err(|e| LatchError::IoError(format!("proc.exec: {e}")))?
                }
                _ => return Err(LatchError::TypeMismatch {
                    expected: "string or list".into(),
                    found: arg.type_name().into(),
                }),
            };

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let code = output.status.code().unwrap_or(-1);

            Ok(Value::ProcessResult { stdout, stderr, code })
        }

        "pipe" => {
            let cmds = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "proc.pipe".into(), expected: 1, found: 0 })?
                .as_list()?;

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
