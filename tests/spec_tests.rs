use latch_lang::hir::{HirLiteral, HirOp};
use latch_lang::vm::{OpCode, LBC_FLAGS, LBC_ISA_VERSION, LBC_MAGIC, LBC_VERSION};

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
fn spec_closure_abi_accessors_exist() {
    use latch_lang::env::{ObjClosure, ObjFunctionBuilder, ObjRef};
    let func = ObjRef::new(ObjFunctionBuilder::new("closure_abi", 0).build());
    let closure = ObjClosure::new(func.clone(), Vec::new());
    assert_eq!(closure.function().name, "closure_abi");
    assert!(closure.upvalues().is_empty());
}

#[test]
fn spec_gc_api_stubs_exist() {
    use latch_lang::env::{ObjFunctionBuilder, ObjRef};
    use latch_lang::vm::gc::GcState;

    let gc = GcState::new();
    let func = ObjRef::new(ObjFunctionBuilder::new("gc_stub", 0).build());
    gc.mark_root(&*func);
    gc.trace_object(&*func);
    gc.write_barrier(&*func, &*func);
    gc.sweep();
    gc.collect_if_needed();
}

#[test]
fn spec_native_object_trait_exists() {
    use latch_lang::env::{
        GcTrace, HeapObject, NativeCallable, NativeObject, ObjHeader, ObjKind, Value,
    };
    use latch_lang::error::Result;

    struct TestNative;
    impl NativeCallable for TestNative {
        fn call(&self, _args: &[Value]) -> Result<Value> {
            Ok(Value::Null)
        }
    }
    impl GcTrace for TestNative {
        fn trace(&self, _visitor: &mut dyn FnMut(&Value)) {}
    }
    impl HeapObject for TestNative {
        fn header(&self) -> &ObjHeader {
            static HEADER: std::sync::OnceLock<ObjHeader> = std::sync::OnceLock::new();
            HEADER.get_or_init(|| ObjHeader::new(ObjKind::Native))
        }
    }
    impl NativeObject for TestNative {
        fn type_name(&self) -> &'static str {
            "TestNative"
        }
    }

    let native = TestNative;
    assert_eq!(native.type_name(), "TestNative");
}

#[test]
fn spec_allocation_profiler_records_allocations() {
    use latch_lang::env::ObjKind;
    use latch_lang::vm::profiler::VmProfiler;

    let profiler = VmProfiler::new();
    profiler.record_allocation(ObjKind::Function, 64);
    profiler.record_allocation(ObjKind::Function, 64);
    profiler.record_allocation(ObjKind::Class, 32);

    let summary = profiler.allocation_summary();
    let func_entry = summary
        .iter()
        .find(|(k, _, _)| *k == ObjKind::Function)
        .unwrap();
    assert_eq!(func_entry.1, 2); // count
    assert_eq!(func_entry.2, 128); // bytes
}

#[test]
fn spec_hir_literals_and_ops_match_specification() {
    let lit_int = HirLiteral::Int(42);
    assert_eq!(lit_int, HirLiteral::Int(42));

    let op_add = HirOp::Add;
    assert_eq!(op_add, HirOp::Add);
}

#[test]
fn spec_obj_header_has_gc_color() {
    use latch_lang::env::{GcColor, ObjHeader, ObjKind};
    let mut header = ObjHeader::new(ObjKind::Function);
    assert_eq!(header.color(), GcColor::White);
    header.set_color(GcColor::Gray);
    assert_eq!(header.color(), GcColor::Gray);
    header.set_color(GcColor::Black);
    assert_eq!(header.color(), GcColor::Black);
}
