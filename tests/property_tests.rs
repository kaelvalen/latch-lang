use latch_lang::lexer::Lexer;
use latch_lang::parser::Parser;
use latch_lang::resolver::Resolver;
use latch_lang::vm::{Compiler, LbcSerializer, VM};

fn test_roundtrip_program(source: &str) {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer error");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_program().expect("Parser error");

    let mut resolver = Resolver::new();
    let hir = resolver.resolve_program(&stmts).expect("Resolver error");

    // 1. Direct compilation to VM
    let compiler1 = Compiler::new();
    let script_fn1 = compiler1.compile_hir(&hir).expect("Compile error");
    let mut vm1 = VM::new(script_fn1).expect("VM1 construction error");
    let res1 = vm1.run().expect("VM1 run error");

    // 2. Compile -> Serialize -> Deserialize -> VM
    let compiler2 = Compiler::new();
    let script_fn2 = compiler2.compile_hir(&hir).expect("Compile error");
    let bytes = LbcSerializer::serialize(&script_fn2);
    let script_fn3 = LbcSerializer::deserialize(&bytes).expect("Deserialize error");
    let mut vm2 = VM::new(script_fn3).expect("VM2 construction error");
    let res2 = vm2.run().expect("VM2 run error");

    assert_eq!(res1, res2, "Roundtrip serialization test failed!");
}

#[test]
fn test_lbc_roundtrip_property() {
    test_roundtrip_program("x := 40; y := 20; return x + y;");
    test_roundtrip_program(
        "count := 0; i := 0; while i < 5 { count = count + i; i = i + 1; } return count;",
    );
}
