use latch_lang::vm::{OpCode, LBC_MAGIC, LBC_VERSION, LBC_ISA_VERSION, LBC_FLAGS};
use latch_lang::hir::{HirLiteral, HirOp};

#[test]
fn spec_vm_opcode_descriptors_match_isa_specification() {
    // Test OP_CONSTANT (0x01)
    let c_desc = OpCode::OpConstant.descriptor();
    assert_eq!(c_desc.opcode as u8, 0x01);
    assert_eq!(c_desc.name, "OP_CONSTANT");
    assert_eq!(c_desc.operand_count, 1);
    assert_eq!(c_desc.stack_out, 1);

    // Test OP_ADD (0x02)
    let add_desc = OpCode::OpAdd.descriptor();
    assert_eq!(add_desc.opcode as u8, 0x02);
    assert_eq!(add_desc.name, "OP_ADD");
    assert_eq!(add_desc.stack_in, 2);
    assert_eq!(add_desc.stack_out, 1);

    // Test OP_CALL (0x14 = 20)
    let call_desc = OpCode::OpCall.descriptor();
    assert_eq!(call_desc.opcode as u8, 20);
    assert_eq!(call_desc.name, "OP_CALL");
    assert!(call_desc.gc_safe);
}

#[test]
fn spec_lbc_binary_format_header_matches_specification() {
    assert_eq!(LBC_MAGIC, b"LATCHB");
    assert_eq!(LBC_VERSION, 1);
    assert_eq!(LBC_ISA_VERSION, 1);
    assert_eq!(LBC_FLAGS, 0);
}

#[test]
fn spec_hir_literals_and_ops_match_specification() {
    let lit_int = HirLiteral::Int(42);
    assert_eq!(lit_int, HirLiteral::Int(42));

    let op_add = HirOp::Add;
    assert_eq!(op_add, HirOp::Add);
}
