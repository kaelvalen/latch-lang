use latch_lang::ast::{Expr, Stmt};
use latch_lang::env::Value;
use latch_lang::resolver::Resolver;
use latch_lang::vm::{Compiler, VM};

#[test]
fn pipeline_compiles_verifies_and_runs_arithmetic() {
    let stmts = vec![
        Stmt::Assign {
            name: "a".into(),
            value: Expr::Int(15),
        },
        Stmt::Assign {
            name: "b".into(),
            value: Expr::Int(25),
        },
        Stmt::Return(Expr::BinOp {
            op: latch_lang::ast::BinOp::Add,
            left: Box::new(Expr::Ident("a".into())),
            right: Box::new(Expr::Ident("b".into())),
        }),
    ];

    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("pipe", &stmts).unwrap();
    let compiler = Compiler::new();
    let func = compiler.compile_module(&module).unwrap();
    let mut vm = VM::new(func).expect("VM construction error");
    let result = vm.run().expect("VM run error");
    assert_eq!(result, Value::Int(40));
}

#[test]
fn pipeline_compiles_verifies_and_runs_conditionals() {
    let stmts = vec![
        Stmt::Assign {
            name: "x".into(),
            value: Expr::Int(10),
        },
        Stmt::If {
            cond: Expr::BinOp {
                op: latch_lang::ast::BinOp::Gt,
                left: Box::new(Expr::Ident("x".into())),
                right: Box::new(Expr::Int(5)),
            },
            then: vec![Stmt::Return(Expr::Int(1))],
            else_: None,
        },
        Stmt::Return(Expr::Int(0)),
    ];

    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("pipe", &stmts).unwrap();
    let compiler = Compiler::new();
    let func = compiler.compile_module(&module).unwrap();
    let mut vm = VM::new(func).expect("VM construction error");
    let result = vm.run().expect("VM run error");
    assert_eq!(result, Value::Int(1));
}
