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
fn test_closure_mutation_upvalue() {
    use latch_lang::lexer::Lexer;
    use latch_lang::parser::Parser;
    use latch_lang::interpreter::Interpreter;

    let src = r#"
        counter := fn() {
            n := 0
            inc := fn() { n = n + 1; return n }
            return inc
        }
        c := counter()
        r1 := c()
        r2 := c()
        r3 := c()
    "#;
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    let mut interp = Interpreter::new();
    interp.run(ast).unwrap();
    assert_eq!(interp.env.get("r1").unwrap().to_string(), "1");
    assert_eq!(interp.env.get("r2").unwrap().to_string(), "2");
    assert_eq!(interp.env.get("r3").unwrap().to_string(), "3");
}

#[test]
fn test_or_error_fallback() {
    use latch_lang::lexer::Lexer;
    use latch_lang::parser::Parser;
    use latch_lang::interpreter::Interpreter;

    let src = r#"
        data := json.parse("{invalid json") or {"fallback": 42}
        res := data["fallback"]
    "#;
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    let mut interp = Interpreter::new();
    interp.run(ast).unwrap();
    assert_eq!(interp.env.get("res").unwrap().to_string(), "42");
}

#[test]
fn test_standalone_trim_upper_and_exit_code() {
    use latch_lang::lexer::Lexer;
    use latch_lang::parser::Parser;
    use latch_lang::interpreter::Interpreter;

    let src = r#"
        t := trim("  hello  ")
        u := upper(t)
        res := proc.exec("echo ok")
        code := res.exit_code
    "#;
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    let mut interp = Interpreter::new();
    interp.run(ast).unwrap();
    assert_eq!(interp.env.get("t").unwrap().to_string(), "hello");
    assert_eq!(interp.env.get("u").unwrap().to_string(), "HELLO");
    assert_eq!(interp.env.get("code").unwrap().to_string(), "0");
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
    use latch_lang::resolver::Resolver;
    use latch_lang::vm::Compiler;

    let stmts = vec![Stmt::Break];
    let mut resolver = Resolver::new();
    if let Ok(module) = resolver.resolve_module("test", &stmts) {
        let compiler = Compiler::new();
        if let Ok(func) = compiler.compile_module(&module) {
            func.chunk.disassemble("test_chunk");
        }
    }
}

#[test]
fn test_optimizer_constant_folding() {
    use latch_lang::ast::{Expr, Stmt, BinOp};
    use latch_lang::hir::{HirExpr, HirLiteral, HirStmt};
    use latch_lang::resolver::Resolver;
    use latch_lang::vm::Optimizer;

    let stmt = Stmt::Assign {
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
    };

    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("test", &[stmt]).expect("Resolver error");

    let optimizer = Optimizer::new();
    let opt_module = optimizer.optimize_module(&module);
    if let HirStmt::LetGlobal { value: HirExpr::Constant(HirLiteral::Int(val)), .. } = &opt_module.stmts[0] {
        assert_eq!(*val, 50);
    } else {
        panic!("HIR Constant folding failed!");
    }
}

#[test]
fn test_execution_abi_contract() {
    use latch_lang::ast::{Expr, Stmt, BinOp};
    use latch_lang::resolver::Resolver;
    use latch_lang::vm::{Compiler, VM};

    let stmts = vec![
        Stmt::Assign {
            name: "a".into(),
            value: Expr::Int(15),
        },
        Stmt::Assign {
            name: "b".into(),
            value: Expr::Int(25),
        },
        Stmt::Assign {
            name: "c".into(),
            value: Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Ident("a".into())),
                right: Box::new(Expr::Ident("b".into())),
            },
        },
    ];

    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("test", &stmts).expect("Resolver error");

    let compiler = Compiler::new();
    let func = compiler.compile_module(&module).expect("Compile error");
    let mut vm = VM::new(func).expect("VM construction error");
    let result = vm.run().expect("VM run error");
    assert_eq!(result, latch_lang::env::Value::Null);
}

#[test]
fn test_obj_ref_is_used_for_function_and_closure() {
    use latch_lang::ast::{Expr, Stmt};
    use latch_lang::resolver::Resolver;
    use latch_lang::vm::{Compiler, VM};
    use latch_lang::env::ObjRef;

    let stmts = vec![Stmt::Assign { name: "a".into(), value: Expr::Int(1) }];
    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("test", &stmts).expect("resolve");
    let compiler = Compiler::new();
    let func = compiler.compile_module(&module).expect("compile");
    let _func_ref: ObjRef<_> = func.clone();
    let mut vm = VM::new(func).expect("VM construction error");
    let result = vm.run().expect("VM run error");
    assert_eq!(result, latch_lang::env::Value::Null);
}

#[test]
fn test_obj_function_builder_produces_valid_function() {
    use latch_lang::env::ObjFunctionBuilder;
    use latch_lang::vm::Chunk;
    let chunk = Chunk::new();
    let func = ObjFunctionBuilder::new("test", 2)
        .with_chunk(chunk)
        .with_max_stack(64)
        .with_upvalue_count(1)
        .build();
    assert_eq!(func.name, "test");
    assert_eq!(func.arity, 2);
    assert_eq!(func.max_stack, 64);
    assert_eq!(func.upvalue_count, 1);
}

#[test]
fn test_gc_state_allocation_api() {
    use latch_lang::env::ObjFunctionBuilder;
    use latch_lang::vm::gc::GcState;

    let gc = GcState::new();
    let func = gc.allocate_function(
        ObjFunctionBuilder::new("api_test", 0)
    );
    assert_eq!(func.name, "api_test");

    let closure = gc.allocate_closure(func.clone(), Vec::new());
    assert_eq!(closure.function().name, "api_test");

    let class = gc.allocate_class("ApiClass");
    assert_eq!(class.name, "ApiClass");
}
