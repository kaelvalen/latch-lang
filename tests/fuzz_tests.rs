use latch_lang::env::{ObjFunction, ObjHeader, ObjKind};
use latch_lang::vm::{BytecodeVerifier, ChunkBuilder, Constant, VM};

#[test]
fn test_bytecode_verifier_rejects_invalid_opcodes() {
    let mut builder = ChunkBuilder::new();
    builder.write_u8(0xFF, 1); // Invalid opcode
    let chunk = builder.build();

    let func = ObjFunction {
        header: ObjHeader::new(ObjKind::Function),
        arity: 0,
        chunk,
        name: "invalid_fn".into(),
        upvalue_count: 0,
        max_stack: 256,
        local_count: 0,
        module_id: 0,
        debug_id: 0,
        flags: 0,
    };

    let verify_res = BytecodeVerifier::verify(&func);
    assert!(verify_res.is_err(), "Verifier should reject invalid opcode 0xFF");
}

#[test]
fn test_bytecode_fuzzer_resilience() {
    // Generate 50 pseudo-random valid bytecode streams
    for seed in 0..50u8 {
        let mut builder = ChunkBuilder::new();
        let const_val = Constant::Int(seed as i64);
        let const_idx = builder.add_constant(const_val);

        // OP_CONSTANT <const_idx>
        builder.write_opcode(latch_lang::vm::OpCode::OpConstant, 1);
        builder.write_u16(const_idx as u16, 1);

        // OP_RETURN
        builder.write_opcode(latch_lang::vm::OpCode::OpReturn, 1);

        let chunk = builder.build();
        let func = ObjFunction {
            header: ObjHeader::new(ObjKind::Function),
            arity: 0,
            chunk,
            name: format!("fuzz_{seed}"),
            upvalue_count: 0,
            max_stack: 256,
            local_count: 0,
            module_id: 0,
            debug_id: 0,
            flags: 0,
        };

        let verify_res = BytecodeVerifier::verify(&func);
        assert!(verify_res.is_ok(), "Generated fuzzing bytecode failed verifier!");

        let mut vm = VM::new(std::sync::Arc::new(func)).expect("VM construction error");
        let run_res = vm.run();
        assert!(run_res.is_ok(), "VM crashed executing verified fuzzing bytecode!");
        assert_eq!(run_res.unwrap().as_int().unwrap(), seed as i64);
    }
}
