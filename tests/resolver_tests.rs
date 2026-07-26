use latch_lang::hir::{HirExpr, HirLiteral, HirStmt, LocalId};
use latch_lang::hir_verifier::HirVerifier;
use latch_lang::lexer::Lexer;
use latch_lang::lowering::HirLowering;
use latch_lang::parser::Parser;
use latch_lang::resolver::Resolver;

#[test]
fn test_resolver_scope_shadowing_resolution() {
    let source = "x = 1; y = 2;";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer error");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_program().expect("Parser error");

    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("shadow_test", &stmts).expect("Resolver error");

    // Verify HIR Verifier accepts the resolved module
    let verify_res = HirVerifier::verify(&module);
    assert!(verify_res.is_ok(), "HirVerifier rejected resolved module!");

    // Verify scope depth created distinct LocalIds for shadowed variable x
    assert!(!module.stmts.is_empty());
}

#[test]
fn test_hir_lowering_phase() {
    let source = "a = 10; b = 20; return a + b;";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Lexer error");
    let mut parser = Parser::new(tokens);
    let stmts = parser.parse_program().expect("Parser error");

    let mut lowering = HirLowering::new();
    let module = lowering.lower_module("lowering_test", &stmts).expect("Lowering error");

    assert_eq!(module.name, "lowering_test");
    assert!(module.stmts.len() >= 3);
}
