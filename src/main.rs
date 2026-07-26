mod ast;
mod env;
mod error;
mod hir;
mod hir_verifier;
mod interpreter;
mod lexer;
mod lowering;
mod lsp;
mod parser;
mod resolver;
mod runtime;
mod semantic;
mod symbol;
mod typechecker;
mod vm;

use std::io::{self, BufRead, Write};

use clap::{Parser, Subcommand};

use crate::error::{format_error, get_source_line, ErrorContext, LatchError};
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::semantic::SemanticAnalyzer;

#[derive(Parser)]
#[command(name = "latch", version = "0.4.3", about = "Latch — local automation scripting language")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a Latch script with Tree-walk Interpreter
    Run {
        /// Path to the .lt file
        file: String,
    },
    /// Run a Latch script with Bytecode Virtual Machine (VM)
    Vm {
        /// Path to the .lt file
        file: String,
    },
    /// Check a script for errors without running it
    Check {
        /// Path to the .lt file
        file: String,
    },
    /// Start Language Server Protocol (LSP) server
    Lsp,
    /// Interactive REPL
    Repl,
    /// Show version
    Version,
}

/// Print a LatchError with full context (file, line, source, reason, hint).
fn print_error(err: &LatchError, file: &str, source: &str) {
    let mut ctx = ErrorContext::new().with_file(file);

    // Try to resolve source line from the error's embedded line number
    if let Some(line) = err.line_number() {
        if let Some(src) = get_source_line(source, line) {
            ctx = ctx.with_source(&src);
        }
    }

    eprintln!("{}", format_error(err, &ctx));
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Run { file } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[latch] IO Error\n  file: {file}\n  reason: {e}");
                    std::process::exit(1);
                }
            };

            // Lex
            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize() {
                Ok(t) => t,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };

            // Parse
            let mut parser = crate::parser::Parser::new(tokens);
            let ast = match parser.parse_program() {
                Ok(a) => a,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };

            // Semantic analysis
            let mut analyzer = SemanticAnalyzer::new();
            let errors = analyzer.analyze(&ast);
            if !errors.is_empty() {
                for e in &errors {
                    print_error(e, &file, &source);
                }
                std::process::exit(1);
            }

            // Interpret
            let mut interp = Interpreter::new();
            if let Some(parent) = std::path::Path::new(&file).parent() {
                interp.set_script_dir(&parent.to_string_lossy());
            }
            if let Err(e) = interp.run(ast) {
                print_error(&e, &file, &source);
                std::process::exit(1);
            }
        }

        Command::Vm { file } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[latch] IO Error\n  file: {file}\n  reason: {e}");
                    std::process::exit(1);
                }
            };

            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize() {
                Ok(t) => t,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };

            let mut parser = crate::parser::Parser::new(tokens);
            let ast = match parser.parse_program() {
                Ok(a) => a,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };

            let mut analyzer = SemanticAnalyzer::new();
            let errors = analyzer.analyze(&ast);
            if !errors.is_empty() {
                for e in &errors {
                    print_error(e, &file, &source);
                }
                std::process::exit(1);
            }

            let mut resolver = crate::resolver::Resolver::new();
            let hir_module = match resolver.resolve_module("<script>", &ast) {
                Ok(m) => m,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };

            let mut typechecker = crate::typechecker::TypeChecker::new();
            let type_errors = typechecker.check_module(&hir_module);
            if !type_errors.is_empty() {
                for e in &type_errors {
                    print_error(e, &file, &source);
                }
                std::process::exit(1);
            }

            let optimizer = crate::vm::Optimizer::new();
            let opt_hir_module = optimizer.optimize_module(&hir_module);

            let compiler = crate::vm::Compiler::new();
            let script_fn = match compiler.compile_module(&opt_hir_module) {
                Ok(f) => f,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };

            let mut vm = match crate::vm::VM::new(script_fn) {
                Ok(v) => v,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };
            if let Err(e) = vm.run() {
                print_error(&e, &file, &source);
                std::process::exit(1);
            }
        }

        Command::Check { file } => {
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[latch] IO Error\n  file: {file}\n  reason: {e}");
                    std::process::exit(1);
                }
            };

            let mut lexer = Lexer::new(&source);
            let tokens = match lexer.tokenize() {
                Ok(t) => t,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };

            let mut parser = crate::parser::Parser::new(tokens);
            let ast = match parser.parse_program() {
                Ok(a) => a,
                Err(e) => {
                    print_error(&e, &file, &source);
                    std::process::exit(1);
                }
            };

            let mut analyzer = SemanticAnalyzer::new();
            let errors = analyzer.analyze(&ast);

            let mut typechecker = crate::typechecker::TypeChecker::new();
            let type_errors = typechecker.check_program(&ast);

            if errors.is_empty() && type_errors.is_empty() {
                println!("[latch] OK — no errors found.");
            } else {
                for e in &errors {
                    print_error(e, &file, &source);
                }
                for e in &type_errors {
                    print_error(e, &file, &source);
                }
                std::process::exit(1);
            }
        }

        Command::Lsp => {
            lsp::start_lsp_server();
        }

        Command::Repl => {
            run_repl();
        }

        Command::Version => {
            println!("latch v0.4.3");
        }
    }
}

// ── REPL ─────────────────────────────────────────────────────

fn run_repl() {
    println!("latch v0.4.3 — interactive REPL");
    println!("Type expressions or statements. Use Ctrl+D to exit.\n");

    let stdin = io::stdin();
    let mut interp = Interpreter::new();

    loop {
        print!("> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!("\n[latch] Bye!");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[latch] Read error: {e}");
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if trimmed == "exit" || trimmed == "quit" {
            println!("[latch] Bye!");
            break;
        }

        // Lex
        let mut lexer = Lexer::new(trimmed);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        // Parse
        let mut parser = crate::parser::Parser::new(tokens);
        let ast = match parser.parse_program() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        // Skip semantic pass in REPL — allow incremental definitions
        // Execute and print the result of the last expression
        for stmt in ast {
            match &stmt {
                crate::ast::Stmt::Expr(_) => {
                    match interp.eval_stmt_for_repl(stmt) {
                        Ok(Some(val)) => println!("{val}"),
                        Ok(None) => {}
                        Err(e) => eprintln!("{e}"),
                    }
                }
                _ => {
                    if let Err(e) = interp.exec_stmt_public(stmt) {
                        eprintln!("{e}");
                    }
                }
            }
        }
    }
}
