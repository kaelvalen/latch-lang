use latch_lang::env::{ObjFunction, ObjHeader, ObjKind, Value};
use latch_lang::vm::{BytecodeVerifier, Chunk, VM};

#[test]
fn test_bytecode_verifier_rejects_invalid_opcodes() {
    let mut chunk = Chunk::new();
    chunk.write_u8(0xFF, 1); // Invalid opcode

    let func = ObjFunction {
        header: ObjHeader::new(ObjKind::Function),
        arity: 0,
        chunk,
        name: "invalid_fn".into(),
        upvalue_count: 0,
    };

    let verify_res = BytecodeVerifier::verify(&func);
    assert!(verify_res.is_err(), "Verifier should reject invalid opcode 0xFF");
}

#[test]
fn test_bytecode_fuzzer_resilience() {
    // Generate 50 pseudo-random valid bytecode streams
    for seed in 0..50u8 {
        let mut chunk = Chunk::new();
        let const_val = Value::Int(seed as i64);
        let const_idx = chunk.add_constant(const_val);

        // OP_CONSTANT <const_idx>
        chunk.write_opcode(latch_lang::vm::OpCode::OpConstant, 1);
        chunk.write_u16(const_idx as u16, 1);

        // OP_RETURN
        chunk.write_opcode(latch_lang::vm::OpCode::OpReturn, 1);

        let func = ObjFunction {
            header: ObjHeader::new(ObjKind::Function),
            arity: 0,
            chunk,
            name: format!("fuzz_{seed}"),
            upvalue_count: 0,
        };

        let verify_res = BytecodeVerifier::verify(&func);
        assert!(verify_res.is_ok(), "Generated fuzzing bytecode failed verifier!");

        let mut vm = VM::new(std::sync::Arc::new(func));
        let run_res = vm.run();
        assert!(run_res.is_ok(), "VM crashed executing verified fuzzing bytecode!");
        assert_eq!(run_res.unwrap().as_int().unwrap(), seed as i64);
    }
}
