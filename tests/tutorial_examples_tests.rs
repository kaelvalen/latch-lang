use std::process::Command;

#[test]
fn tutorial_examples_harness() {
    let output = Command::new("bash")
        .args(["scripts/run_examples.sh"])
        .output()
        .expect("failed to run example harness");

    if !output.status.success() {
        eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
}

#[test]
fn test_pop_arity_regression() {
    use latch_lang::interpreter::Interpreter;
    use latch_lang::lexer::Lexer;
    use latch_lang::parser::Parser;

    // `pop` must accept both one argument (pop last) and two arguments (pop at index).
    let src = r#"
        items := [1, 2, 3, 4]
        last := pop(items)
        at_index := pop(items, 0)
        result := last + at_index
    "#;
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();
    let mut interp = Interpreter::new();
    interp.run(ast).unwrap();
    assert_eq!(interp.env.get("last").unwrap().to_string(), "4");
    assert_eq!(interp.env.get("at_index").unwrap().to_string(), "1");
    assert_eq!(interp.env.get("result").unwrap().to_string(), "5");
}

#[test]
fn test_string_concat_assignment_regression() {
    use latch_lang::interpreter::Interpreter;
    use latch_lang::lexer::Lexer;
    use latch_lang::parser::Parser;

    // Assigning a string concatenation to a string variable must be allowed by the typechecker.
    let src = r#"
        msg := "Hello"
        msg = msg + " World"
        msg = msg + "!"
    "#;
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program().unwrap();

    let mut typechecker = latch_lang::typechecker::TypeChecker::new();
    let errors = typechecker.check_program(&ast);
    assert!(errors.is_empty(), "typechecker errors: {errors:?}");

    let mut interp = Interpreter::new();
    interp.run(ast).unwrap();
    assert_eq!(interp.env.get("msg").unwrap().to_string(), "Hello World!");
}
