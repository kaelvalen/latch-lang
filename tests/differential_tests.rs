use latch_lang::interpreter::Interpreter;
use latch_lang::lexer::Lexer;
use latch_lang::parser::Parser;
use latch_lang::resolver::Resolver;
use latch_lang::vm::{Compiler, VM};

fn run_tree_walk(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer error");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_program().expect("Parser error");
    let mut interpreter = Interpreter::new();

    match interpreter.run(stmts) {
        Err(latch_lang::error::LatchError::ReturnSignal(val)) => format!("{val}\n"),
        Ok(()) => "null\n".into(),
        Err(e) => format!("Error: {e}\n"),
    }
}

fn run_bytecode_vm(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer error");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_program().expect("Parser error");

    let mut resolver = Resolver::new();
    let module = resolver
        .resolve_module("diff_test", &stmts)
        .expect("Resolver error");

    let compiler = Compiler::new();
    let script_fn = compiler.compile_module(&module).expect("Compiler error");
    let mut vm = VM::new(script_fn).expect("VM construction error");

    match vm.run() {
        Ok(val) => format!("{val}\n"),
        Err(e) => format!("Error: {e}\n"),
    }
}

#[test]
fn test_differential_arithmetic_expressions() {
    let code = "a = 10; b = 20; c = a + b * 2; return c;";
    let tw = run_tree_walk(code);
    let vm = run_bytecode_vm(code);
    assert_eq!(
        tw.trim(),
        vm.trim(),
        "Differential test failed for arithmetic expressions!"
    );
}

#[test]
fn test_differential_conditionals() {
    let code = "x = 15; if x > 10 { return 100; } else { return 200; }";
    let tw = run_tree_walk(code);
    let vm = run_bytecode_vm(code);
    assert_eq!(
        tw.trim(),
        vm.trim(),
        "Differential test failed for conditionals!"
    );
}

#[test]
fn test_differential_named_function_call() {
    let code = "fn add(a, b) { return a + b; } return add(15, 27);";
    let tw = run_tree_walk(code);
    let vm = run_bytecode_vm(code);
    assert_eq!(
        tw.trim(),
        vm.trim(),
        "Differential test failed for named function call!"
    );
}

#[test]
fn test_differential_recursive_function() {
    let code =
        "fn fact(n) { if n <= 1 { return 1; } else { return n * fact(n - 1); } } return fact(5);";
    let tw = run_tree_walk(code);
    let vm = run_bytecode_vm(code);
    assert_eq!(
        tw.trim(),
        vm.trim(),
        "Differential test failed for recursive function!"
    );
}

#[test]
fn test_differential_closure_return() {
    let code = "make_adder := fn(x) { return fn(y) { return x + y; }; }; add5 := make_adder(5); return add5(10);";
    let tw = run_tree_walk(code);
    let vm = run_bytecode_vm(code);
    assert_eq!(
        tw.trim(),
        vm.trim(),
        "Differential test failed for closure return!"
    );
}

#[test]
fn test_differential_or_fallback() {
    let code = "x := null or 42; return x;";
    let tw = run_tree_walk(code);
    let vm = run_bytecode_vm(code);
    assert_eq!(
        tw.trim(),
        vm.trim(),
        "Differential test failed for OR fallback!"
    );
}
