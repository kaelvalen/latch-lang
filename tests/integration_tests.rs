use std::process::Command;

#[test]
fn test_tree_walk_interpreter() {
    let output = Command::new("./target/debug/latch")
        .args(["run", "examples/vm_test.lt"])
        .output()
        .expect("Failed to run latch");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("50"));
    assert!(stdout.contains("VM test passed!"));
    assert!(stdout.contains("[1, 2, 3, 4, 5]"));
}

#[test]
fn test_bytecode_vm() {
    let output = Command::new("./target/debug/latch")
        .args(["vm", "examples/vm_test.lt"])
        .output()
        .expect("Failed to run latch vm");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("50"));
    assert!(stdout.contains("VM test passed!"));
    assert!(stdout.contains("[1, 2, 3, 4, 5]"));
}

#[test]
fn test_typechecker_and_check() {
    let output = Command::new("./target/debug/latch")
        .args(["check", "examples/vm_test.lt"])
        .output()
        .expect("Failed to run latch check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OK — no errors found."));
}

#[test]
fn test_vm_disassembler() {
    use latch_lang::ast::Stmt;
    use latch_lang::vm::Compiler;

    let stmts = vec![Stmt::Break];
    let compiler = Compiler::new();
    if let Ok(chunk) = compiler.compile(&stmts) {
        chunk.disassemble("test_chunk");
    }
}
