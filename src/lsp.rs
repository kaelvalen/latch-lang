use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;

pub fn start_lsp_server() {
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut stdout = io::stdout();

    loop {
        let mut length_line = String::new();
        if stdin_lock.read_line(&mut length_line).unwrap_or(0) == 0 {
            break;
        }

        if !length_line.starts_with("Content-Length: ") {
            continue;
        }

        let len: usize = length_line
            .trim_start_matches("Content-Length: ")
            .trim()
            .parse()
            .unwrap_or(0);

        // Read empty header separator line
        let mut blank = String::new();
        stdin_lock.read_line(&mut blank).ok();

        // Read JSON payload
        let mut body = vec![0u8; len];
        if io::Read::read_exact(&mut stdin_lock, &mut body).is_err() {
            break;
        }

        let req: Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = req.get("id");

        match method {
            "initialize" => {
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": 1,
                            "completionProvider": { "triggerCharacters": [".", ":"] },
                            "hoverProvider": true
                        }
                    }
                });
                send_response(&mut stdout, &resp);
            }

            "textDocument/didOpen" | "textDocument/didChange" => {
                if let Some(params) = req.get("params") {
                    if let Some(doc) = params.get("textDocument") {
                        let uri = doc.get("uri").and_then(|u| u.as_str()).unwrap_or("");
                        let text = doc.get("text").and_then(|t| t.as_str()).unwrap_or("");

                        let diagnostics = analyze_document(text);
                        let notif = json!({
                            "jsonrpc": "2.0",
                            "method": "textDocument/publishDiagnostics",
                            "params": {
                                "uri": uri,
                                "diagnostics": diagnostics
                            }
                        });
                        send_response(&mut stdout, &notif);
                    }
                }
            }

            "textDocument/hover" => {
                if let Some(id_val) = id {
                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id_val,
                        "result": {
                            "contents": {
                                "kind": "markdown",
                                "value": "**Latch Language Server**\nLocal automation & scripting engine."
                            }
                        }
                    });
                    send_response(&mut stdout, &resp);
                }
            }

            "textDocument/completion" => {
                if let Some(id_val) = id {
                    let keywords = vec![
                        "let", "const", "fn", "if", "else", "for", "while", "break", "continue",
                        "return", "try", "catch", "finally", "parallel", "class", "import",
                        "export", "match",
                    ];
                    let items: Vec<Value> = keywords
                        .into_iter()
                        .map(|kw| {
                            json!({
                                "label": kw,
                                "kind": 14 // Keyword
                            })
                        })
                        .collect();

                    let resp = json!({
                        "jsonrpc": "2.0",
                        "id": id_val,
                        "result": items
                    });
                    send_response(&mut stdout, &resp);
                }
            }

            "shutdown" => {
                if let Some(id_val) = id {
                    let resp = json!({ "jsonrpc": "2.0", "id": id_val, "result": null });
                    send_response(&mut stdout, &resp);
                }
            }

            "exit" => break,

            _ => {}
        }
    }
}

fn analyze_document(text: &str) -> Vec<Value> {
    let mut diagnostics = Vec::new();

    let mut lexer = Lexer::new(text);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            if let Some(line) = e.line_number() {
                diagnostics.push(json!({
                    "range": {
                        "start": { "line": line.saturating_sub(1), "character": 0 },
                        "end": { "line": line.saturating_sub(1), "character": 100 }
                    },
                    "severity": 1,
                    "message": format!("{e}")
                }));
            }
            return diagnostics;
        }
    };

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse_program() {
        Ok(a) => a,
        Err(e) => {
            if let Some(line) = e.line_number() {
                diagnostics.push(json!({
                    "range": {
                        "start": { "line": line.saturating_sub(1), "character": 0 },
                        "end": { "line": line.saturating_sub(1), "character": 100 }
                    },
                    "severity": 1,
                    "message": format!("{e}")
                }));
            }
            return diagnostics;
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    for err in analyzer.analyze(&ast) {
        if let Some(line) = err.line_number() {
            diagnostics.push(json!({
                "range": {
                    "start": { "line": line.saturating_sub(1), "character": 0 },
                    "end": { "line": line.saturating_sub(1), "character": 100 }
                },
                "severity": 1,
                "message": format!("{err}")
            }));
        }
    }

    diagnostics
}

fn send_response(stdout: &mut io::Stdout, val: &Value) {
    let payload = serde_json::to_string(val).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", payload.len());
    stdout.write_all(header.as_bytes()).ok();
    stdout.write_all(payload.as_bytes()).ok();
    stdout.flush().ok();
}
