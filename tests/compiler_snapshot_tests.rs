use latch_lang::ast::{Expr, Stmt};
use latch_lang::resolver::Resolver;
use latch_lang::vm::Compiler;

#[test]
fn snapshot_print_one_plus_two() {
    let stmts = vec![Stmt::Expr(Expr::Call {
        name: "print".into(),
        args: vec![Expr::BinOp {
            op: latch_lang::ast::BinOp::Add,
            left: Box::new(Expr::Int(1)),
            right: Box::new(Expr::Int(2)),
        }],
        kwargs: Vec::new(),
    })];

    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("snap", &stmts).unwrap();
    let compiler = Compiler::new();
    let func = compiler.compile_module(&module).unwrap();

    let mut output = String::new();
    func.chunk.disassemble_to("print(1+2)", &mut output);

    assert!(output.contains("OP_CONSTANT"));
    assert!(output.contains("'3'"));
    assert!(output.contains("OP_PRINT"));
}

#[test]
fn snapshot_while_loop_has_jump_and_loop() {
    let stmts = vec![
        Stmt::Assign {
            name: "i".into(),
            value: Expr::Int(0),
        },
        Stmt::While {
            cond: Expr::BinOp {
                op: latch_lang::ast::BinOp::Lt,
                left: Box::new(Expr::Ident("i".into())),
                right: Box::new(Expr::Int(3)),
            },
            body: vec![Stmt::Expr(Expr::Call {
                name: "print".into(),
                args: vec![Expr::Ident("i".into())],
                kwargs: Vec::new(),
            })],
        },
    ];

    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("snap", &stmts).unwrap();
    let compiler = Compiler::new();
    let func = compiler.compile_module(&module).unwrap();

    let mut output = String::new();
    func.chunk.disassemble_to("while_loop", &mut output);

    assert!(output.contains("OP_JUMP_FALSE"));
    assert!(output.contains("OP_LOOP"));
}
