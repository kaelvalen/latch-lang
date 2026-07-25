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

#[test]
fn test_optimizer_constant_folding() {
    use latch_lang::ast::{Expr, Stmt, BinOp};
    use latch_lang::vm::Optimizer;

    // stmt: let x = 10 + 20 * 2;
    let stmt = Stmt::Let {
        name: "x".into(),
        value: Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Int(10)),
            right: Box::new(Expr::BinOp {
                op: BinOp::Mul,
                left: Box::new(Expr::Int(20)),
                right: Box::new(Expr::Int(2)),
            }),
        },
        type_ann: None,
    };

    let optimizer = Optimizer::new();
    let opt_stmts = optimizer.optimize_stmts(&[stmt]);
    if let Stmt::Let { value: Expr::Int(val), .. } = &opt_stmts[0] {
        assert_eq!(*val, 50);
    } else {
        panic!("Constant folding failed!");
    }
}

#[test]
fn test_execution_abi_contract() {
    use latch_lang::ast::{Expr, Stmt, BinOp};
    use latch_lang::vm::{Compiler, VM};

    // Test ABI execution: let a = 15; let b = 25; let c = a + b;
    let stmts = vec![
        Stmt::Let {
            name: "a".into(),
            value: Expr::Int(15),
            type_ann: None,
        },
        Stmt::Let {
            name: "b".into(),
            value: Expr::Int(25),
            type_ann: None,
        },
        Stmt::Let {
            name: "c".into(),
            value: Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Ident("a".into())),
                right: Box::new(Expr::Ident("b".into())),
            },
            type_ann: None,
        },
        Stmt::Return(Expr::Ident("c".into())),
    ];

    let compiler = Compiler::new();
    let chunk = compiler.compile(&stmts).expect("Compilation failed");
    let mut vm = VM::new(chunk);
    let result = vm.run().expect("VM execution failed");
    assert_eq!(result.as_int().unwrap(), 40);
}
