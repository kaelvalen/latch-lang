use std::process::Command;

use crate::env::Value;
use crate::error::{LatchError, Result};

pub fn call(method: &str, args: Vec<Value>) -> Result<Value> {
    match method {
        "exec" => {
            let arg = args.first()
                .ok_or_else(|| LatchError::ArgCountMismatch { name: "proc.exec".into(), expected: 1, found: 0 })?;

            // Parse options from second argument if provided
            let opts = if args.len() > 1 {
                match &args[1] {
                    Value::Map(m) => m.lock().unwrap().clone(),
                    _ => std::collections::HashMap::new(),
                }
            } else {
                std::collections::HashMap::new()
            };

            let _cwd = opts.get("cwd").and_then(|v| v.as_str().ok());
            let _timeout_secs = opts.get("timeout").and_then(|v| v.as_int().ok());

            let mut cmd = match arg {
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
                    let mut c = Command::new(&cmd_parts[0]);
                    c.args(&cmd_parts[1..]);
                    c
                }
                // String form: proc.exec("ls -la") — via shell
                Value::Str(cmd_str) => {
                    if cfg!(target_os = "windows") {
                        let mut c = Command::new("cmd");
                        c.args(["/C", cmd_str]);
                        c
                    } else {
                        let mut c = Command::new("sh");
                        c.args(["-c", cmd_str]);
                        c
                    }
                }
                _ => return Err(LatchError::TypeMismatch {
                    expected: "string or list".into(),
                    found: arg.type_name().into(),
                }),
            };

            // Apply cwd if specified
            if let Some(ref cwd) = _cwd {
                cmd.current_dir(cwd);
            }

            // Apply env vars if specified
            if let Some(Value::Map(env_map)) = opts.get("env") {
                let env_vars = env_map.lock().unwrap();
                for (k, v) in env_vars.iter() {
                    if let Ok(val) = v.as_str() {
                        cmd.env(k, val);
                    }
                }
            }

            // Execute (timeout requires additional crate, skipping for now)
            let output = cmd.output().map_err(|e| LatchError::IoError(format!("proc.exec: {e}")))?;

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
