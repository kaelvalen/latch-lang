# Object Model, Memory Layout and GC Preparation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (inline execution) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Freeze the VM-side object model ABI (`ObjHeader`, `ObjRef`, allocation API, closure/upvalue layout, GC hooks) and document the memory layout so future GC, JIT and serializer work can plug in without rewriting the runtime.

**Architecture:** Keep the existing `Value` enum surface unchanged in this plan (a full `Value` freeze is a follow-up project). Harden `ObjHeader` with an explicit GC color, standardize heap references behind `ObjRef<T>`, make `ObjFunction`/`ObjClosure` immutable via builders, move allocation tracking into a single `GcState` API, add GC stub hooks (`mark`, `trace`, `sweep`, `write_barrier`), introduce a `NativeObject` trait, and write `docs/MEMORY_LAYOUT.md`.

**Tech Stack:** Rust stable, `std::sync::Arc`, trait objects for `HeapObject`/`NativeObject`.

## Global Constraints

- Do **not** change the `Value` enum ABI in this plan; that is a separate follow-up plan.
- All existing `cargo test` tests and `./target/debug/latch vm examples/vm_test.lt` must pass after every task.
- No git mutations (commit, push, reset, etc.).
- Preserve runtime semantics; this plan is ABI scaffolding, not behavior changes.
- Match the existing code style: snake_case, 4-space indentation, `// ── section ──` dividers.

---

### Task 1: Harden `ObjHeader` with a real GC color field

**Files:**
- Modify: `src/env.rs:32-66`
- Test: `tests/spec_tests.rs`

**Interfaces:**
- Consumes: existing `ObjKind` enum.
- Produces: `pub enum GcColor { White, Gray, Black }`, `ObjHeader::gc_color: GcColor`, `ObjHeader::with_kind(kind: ObjKind) -> Self` builder, `ObjHeader::set_color(&mut self, color: GcColor)`, `ObjHeader::color(&self) -> GcColor`. All existing `ObjHeader::new(kind)` call sites continue to compile.

- [ ] **Step 1: Write the failing test**

Add to `tests/spec_tests.rs`:

```rust
#[test]
fn spec_obj_header_has_gc_color() {
    use latch_lang::env::{ObjHeader, ObjKind, GcColor};
    let mut header = ObjHeader::new(ObjKind::Function);
    assert_eq!(header.color(), GcColor::White);
    header.set_color(GcColor::Gray);
    assert_eq!(header.color(), GcColor::Gray);
    header.set_color(GcColor::Black);
    assert_eq!(header.color(), GcColor::Black);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test spec_obj_header_has_gc_color -- --nocapture`
Expected: FAIL — `GcColor` not found.

- [ ] **Step 3: Implement `GcColor` and update `ObjHeader`**

In `src/env.rs`, replace the `ObjHeader` definition with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcColor {
    White,
    Gray,
    Black,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjHeader {
    pub kind: ObjKind,
    pub flags: u8,
    pub mark: bool,
    pub gc_color: GcColor,
    pub generation: u8,
    pub size: usize,
    pub type_id: u32,
}

impl ObjHeader {
    pub fn new(kind: ObjKind) -> Self {
        ObjHeader {
            kind,
            flags: 0,
            mark: false,
            gc_color: GcColor::White,
            generation: 0,
            size: std::mem::size_of::<Self>(),
            type_id: 0,
        }
    }

    pub fn with_kind(kind: ObjKind) -> Self {
        Self::new(kind)
    }

    pub fn color(&self) -> GcColor {
        self.gc_color
    }

    pub fn set_color(&mut self, color: GcColor) {
        self.gc_color = color;
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test spec_obj_header_has_gc_color -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Run full test suite for regressions**

Run: `cargo test`
Expected: all tests pass.

---

### Task 2: Standardize heap references behind `ObjRef<T>`

**Files:**
- Modify: `src/env.rs:11-29`
- Modify: `src/vm/frame.rs:1-14`
- Modify: `src/vm/vm.rs:20, 27, 33, 47, 82, 121, 127, 143, 153, 169, 174, 354, 398`
- Modify: `src/vm/compiler.rs:23-43`
- Test: `tests/integration_tests.rs::test_execution_abi_contract`

**Interfaces:**
- Consumes: `ObjRef<T>(pub Arc<T>)`.
- Produces: `ObjRef::clone(&self) -> Self`, `ObjRef::into_arc(self) -> Arc<T>` (convenience), `CallFrame::closure: ObjRef<ObjClosure>`, `VM::new(script_fn: ObjRef<ObjFunction>)`, `VerifiedProgram::script_fn: ObjRef<ObjFunction>`, `VmBuilder::new(script_fn: ObjRef<ObjFunction>)`, `Compiler::compile_module(...) -> Result<ObjRef<ObjFunction>>`.

- [ ] **Step 1: Write the failing test**

Add to `tests/integration_tests.rs`:

```rust
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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test test_obj_ref_is_used_for_function_and_closure -- --nocapture`
Expected: FAIL — type mismatch between `Arc<ObjFunction>` and `ObjRef<ObjFunction>`.

- [ ] **Step 3: Add `ObjRef` helpers**

In `src/env.rs`:

```rust
impl<T> ObjRef<T> {
    pub fn new(val: T) -> Self {
        ObjRef(Arc::new(val))
    }

    pub fn clone(&self) -> Self {
        ObjRef(Arc::clone(&self.0))
    }

    pub fn into_arc(self) -> Arc<T> {
        self.0
    }

    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Arc::ptr_eq(&a.0, &b.0)
    }
}
```

- [ ] **Step 4: Update `CallFrame` to hold `ObjRef<ObjClosure>`**

In `src/vm/frame.rs`:

```rust
use crate::env::{ObjClosure, ObjRef};

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub closure: ObjRef<ObjClosure>,
    pub ip: usize,
    pub slots: usize,
    pub return_slot: usize,
    pub stack_limit: usize,
    pub flags: u32,
}

impl CallFrame {
    pub fn new(closure: ObjRef<ObjClosure>, slots: usize, return_slot: usize) -> Self {
        CallFrame {
            closure,
            ip: 0,
            slots,
            return_slot,
            stack_limit: 256,
            flags: 0,
        }
    }
}
```

- [ ] **Step 5: Update `vm.rs` to use `ObjRef` everywhere**

Replace all `Arc<ObjFunction>` and `Arc<ObjClosure>` usage with `ObjRef`. Key snippets:

```rust
pub struct VerifiedProgram {
    pub(crate) script_fn: ObjRef<ObjFunction>,
}

pub struct VmBuilder {
    script_fn: ObjRef<ObjFunction>,
}

impl VmBuilder {
    pub fn new(script_fn: ObjRef<ObjFunction>) -> Self { ... }
}

pub fn new(script_fn: ObjRef<ObjFunction>) -> Result<Self> { ... }

pub fn load(&mut self, script_fn: ObjRef<ObjFunction>) -> Result<()> { ... }
```

In `from_chunk` and `new_with_chunk` use `ObjRef::new(ObjFunction { ... })` instead of `Arc::new(...)`.

In `OpClosure` branch use `ObjRef(func)` instead of `crate::env::ObjRef(func)` where `func` is `Arc<ObjFunction>`; adjust by keeping `func` as `ObjRef<ObjFunction>` from the start.

- [ ] **Step 6: Update `compiler.rs` return type**

Change `compile_module` to return `Result<ObjRef<ObjFunction>>` and use `ObjRef::new(script_fn)` at the end.

- [ ] **Step 7: Update `main.rs` call site**

Change `let script_fn = match compiler.compile_module(&opt_hir_module) { ... }` so the value is passed to `VM::new(script_fn)` directly; `VM::new` now accepts `ObjRef<ObjFunction>`.

- [ ] **Step 8: Run the test to verify it passes**

Run: `cargo test test_obj_ref_is_used_for_function_and_closure -- --nocapture`
Expected: PASS.

- [ ] **Step 9: Run full test suite for regressions**

Run: `cargo test`
Expected: all tests pass.

---

### Task 3: Make `ObjFunction` construction immutable via `ObjFunctionBuilder`

**Files:**
- Modify: `src/env.rs:68-97`
- Modify: `src/vm/compiler.rs:31-43`
- Modify: `src/vm/vm.rs:47-58, 127-139`
- Test: `tests/integration_tests.rs`

**Interfaces:**
- Consumes: `ObjFunction::new`.
- Produces: `ObjFunctionBuilder`, `ObjFunctionBuilder::new(name, arity) -> Self`, `ObjFunctionBuilder::with_chunk(mut self, chunk: Chunk) -> Self`, `ObjFunctionBuilder::with_max_stack(mut self, max_stack: usize) -> Self`, `ObjFunctionBuilder::with_upvalue_count(mut self, count: usize) -> Self`, `ObjFunctionBuilder::build(self) -> ObjFunction`. `ObjFunction` fields remain `pub` for now (freeze happens in Task 7), but production code must only construct via the builder.

- [ ] **Step 1: Write the failing test**

Add to `tests/integration_tests.rs`:

```rust
#[test]
fn test_obj_function_builder_produces_valid_function() {
    use latch_lang::env::{ObjFunctionBuilder, Chunk};
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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test test_obj_function_builder_produces_valid_function -- --nocapture`
Expected: FAIL — `ObjFunctionBuilder` not found.

- [ ] **Step 3: Implement `ObjFunctionBuilder`**

In `src/env.rs`, after `ObjFunction`:

```rust
#[derive(Debug, Clone)]
pub struct ObjFunctionBuilder {
    arity: usize,
    chunk: Chunk,
    name: String,
    upvalue_count: usize,
    max_stack: usize,
    local_count: usize,
    module_id: u32,
    debug_id: u32,
    flags: u32,
}

impl ObjFunctionBuilder {
    pub fn new(name: impl Into<String>, arity: usize) -> Self {
        Self {
            arity,
            chunk: Chunk::new(),
            name: name.into(),
            upvalue_count: 0,
            max_stack: 256,
            local_count: 0,
            module_id: 0,
            debug_id: 0,
            flags: 0,
        }
    }

    pub fn with_chunk(mut self, chunk: Chunk) -> Self {
        self.chunk = chunk;
        self
    }

    pub fn with_max_stack(mut self, max_stack: usize) -> Self {
        self.max_stack = max_stack;
        self
    }

    pub fn with_upvalue_count(mut self, count: usize) -> Self {
        self.upvalue_count = count;
        self
    }

    pub fn with_local_count(mut self, count: usize) -> Self {
        self.local_count = count;
        self
    }

    pub fn build(self) -> ObjFunction {
        ObjFunction {
            header: ObjHeader::new(ObjKind::Function),
            arity: self.arity,
            chunk: self.chunk,
            name: self.name,
            upvalue_count: self.upvalue_count,
            max_stack: self.max_stack,
            local_count: self.local_count,
            module_id: self.module_id,
            debug_id: self.debug_id,
            flags: self.flags,
        }
    }
}
```

- [ ] **Step 4: Replace manual `ObjFunction` construction in `compiler.rs`**

In `src/vm/compiler.rs`:

```rust
let script_fn = ObjFunctionBuilder::new(module.name.clone(), 0)
    .with_chunk(self.chunk.build())
    .build();
Ok(ObjRef::new(script_fn))
```

- [ ] **Step 5: Replace manual construction in `vm.rs`**

In `from_chunk` and `new_with_chunk`:

```rust
let script_fn = ObjFunctionBuilder::new("<script>", 0)
    .with_chunk(chunk)
    .build();
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test test_obj_function_builder_produces_valid_function -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Run full test suite for regressions**

Run: `cargo test`
Expected: all tests pass.

---

### Task 4: Centralize allocation API in `GcState`

**Files:**
- Modify: `src/vm/gc.rs`
- Modify: `src/vm/vm.rs:153-178`
- Test: `tests/integration_tests.rs`

**Interfaces:**
- Consumes: `GcState::track_alloc`, `ObjRef<T>`, `ObjFunctionBuilder`.
- Produces: `GcState::allocate<T>(&self, obj: T) -> ObjRef<T>`, `GcState::allocate_function(&self, builder: ObjFunctionBuilder) -> ObjRef<ObjFunction>`, `GcState::allocate_closure(&self, function: ObjRef<ObjFunction>, upvalues: Vec<Arc<Mutex<Value>>>) -> ObjRef<ObjClosure>`, `GcState::allocate_class(&self, name: impl Into<String>) -> ObjRef<ObjClass>`. `VM::alloc_*` become thin wrappers that call `self.gc.allocate_*`.

- [ ] **Step 1: Write the failing test**

Add to `tests/integration_tests.rs`:

```rust
#[test]
fn test_gc_state_allocation_api() {
    use latch_lang::env::{ObjFunctionBuilder, ObjClass};
    use latch_lang::vm::gc::GcState;

    let gc = GcState::new();
    let func = gc.allocate_function(
        ObjFunctionBuilder::new("api_test", 0).build()
    );
    assert_eq!(func.name, "api_test");

    let closure = gc.allocate_closure(func.clone(), Vec::new());
    assert_eq!(closure.function.name, "api_test");

    let class = gc.allocate_class("ApiClass");
    assert_eq!(class.name, "ApiClass");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test test_gc_state_allocation_api -- --nocapture`
Expected: FAIL — `allocate_function` not found.

- [ ] **Step 3: Add allocation API to `GcState`**

In `src/vm/gc.rs`:

```rust
use crate::env::{ObjClass, ObjClosure, ObjFunction, ObjFunctionBuilder, ObjRef};
use std::sync::{Arc, Mutex};

impl GcState {
    pub fn allocate<T>(&self, obj: T) -> ObjRef<T> {
        self.track_alloc(std::mem::size_of::<T>());
        ObjRef::new(obj)
    }

    pub fn allocate_function(&self, builder: ObjFunctionBuilder) -> ObjRef<ObjFunction> {
        let func = builder.build();
        self.track_alloc(std::mem::size_of::<ObjFunction>() + func.chunk.code().len());
        ObjRef::new(func)
    }

    pub fn allocate_closure(
        &self,
        function: ObjRef<ObjFunction>,
        upvalues: Vec<Arc<Mutex<Value>>>,
    ) -> ObjRef<ObjClosure> {
        self.track_alloc(std::mem::size_of::<ObjClosure>());
        ObjRef::new(ObjClosure::new(function, upvalues))
    }

    pub fn allocate_class(&self, name: impl Into<String>) -> ObjRef<ObjClass> {
        let class = ObjClass::new(name);
        self.track_alloc(std::mem::size_of::<ObjClass>());
        ObjRef::new(class)
    }
}
```

- [ ] **Step 4: Make `VM::alloc_*` thin wrappers**

In `src/vm/vm.rs`:

```rust
pub fn alloc_function(&self, arity: usize, chunk: Chunk, name: String) -> ObjRef<ObjFunction> {
    self.gc.allocate_function(
        ObjFunctionBuilder::new(name, arity).with_chunk(chunk)
    )
}

pub fn alloc_closure(&self, function: ObjRef<ObjFunction>, upvalues: Vec<Arc<Mutex<Value>>>) -> ObjRef<ObjClosure> {
    self.gc.allocate_closure(function, upvalues)
}

pub fn alloc_class(&self, name: impl Into<String>) -> ObjRef<ObjClass> {
    self.gc.allocate_class(name)
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test test_gc_state_allocation_api -- --nocapture`
Expected: PASS.

- [ ] **Step 6: Run full test suite for regressions**

Run: `cargo test`
Expected: all tests pass.

---

### Task 5: Document and freeze the closure/upvalue ABI

**Files:**
- Modify: `src/env.rs:99-121`
- Modify: `src/vm/vm.rs:323-348, 350-357`
- Create: `docs/MEMORY_LAYOUT.md` (initial partial)
- Test: `tests/spec_tests.rs`

**Interfaces:**
- Consumes: `ObjClosure::new`.
- Produces: `ObjClosure::function(&self) -> &ObjRef<ObjFunction>`, `ObjClosure::upvalues(&self) -> &[Arc<Mutex<Value>>]`, documented upvalue layout in `docs/MEMORY_LAYOUT.md`. Fields of `ObjClosure` become `pub(crate)` to enforce external use of accessors.

- [ ] **Step 1: Write the failing test**

Add to `tests/spec_tests.rs`:

```rust
#[test]
fn spec_closure_abi_accessors_exist() {
    use latch_lang::env::{ObjClosure, ObjFunctionBuilder, ObjRef};
    let func = ObjRef::new(ObjFunctionBuilder::new("closure_abi", 0).build());
    let closure = ObjClosure::new(func.clone(), Vec::new());
    assert_eq!(closure.function().name, "closure_abi");
    assert!(closure.upvalues().is_empty());
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test spec_closure_abi_accessors_exist -- --nocapture`
Expected: FAIL — accessors do not exist.

- [ ] **Step 3: Add accessors and tighten visibility**

In `src/env.rs`:

```rust
#[derive(Debug, Clone)]
pub struct ObjClosure {
    pub header: ObjHeader,
    pub(crate) function: ObjRef<ObjFunction>,
    pub(crate) upvalues: Vec<Arc<Mutex<Value>>>,
}

impl ObjClosure {
    pub fn new(function: ObjRef<ObjFunction>, upvalues: Vec<Arc<Mutex<Value>>>) -> Self {
        ObjClosure {
            header: ObjHeader::new(ObjKind::Closure),
            function,
            upvalues,
        }
    }

    pub fn function(&self) -> &ObjRef<ObjFunction> {
        &self.function
    }

    pub fn upvalues(&self) -> &[Arc<Mutex<Value>>] {
        &self.upvalues
    }
}
```

- [ ] **Step 4: Update `vm.rs` to use accessors**

Replace all direct `closure.function.*` accesses with `closure.function().*` and direct `closure.upvalues` with `closure.upvalues()`. In `frame.rs` no direct field access beyond `closure` itself is needed.

- [ ] **Step 5: Seed `docs/MEMORY_LAYOUT.md` closure section**

Create `docs/MEMORY_LAYOUT.md` with:

```markdown
# Latch Memory Layout Specification

## Closure ABI

An `ObjClosure` is an immutable heap object with the following layout:

```text
ObjClosure
├── header: ObjHeader        (kind = Closure)
├── function: ObjRef<ObjFunction>
└── upvalues: [Arc<Mutex<Value>>]
```

- `function` is the shared compiled function template.
- `upvalues` is a fixed-size vector indexed by the upvalue slot in bytecode.
- Each upvalue currently holds a `Mutex<Value>`; future work will split this into open (stack pointer) and closed (boxed heap) representations.
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test spec_closure_abi_accessors_exist -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Run full test suite for regressions**

Run: `cargo test`
Expected: all tests pass.

---

### Task 6: Add GC API stubs (`mark`, `trace`, `sweep`, `write_barrier`)

**Files:**
- Modify: `src/vm/gc.rs`
- Modify: `src/env.rs:195-203`
- Test: `tests/spec_tests.rs`

**Interfaces:**
- Consumes: `HeapObject`, `GcTrace`.
- Produces: `GcState::mark_root(&self, obj: &dyn HeapObject)`, `GcState::trace_object(&self, obj: &dyn GcTrace)`, `GcState::sweep(&self)`, `GcState::write_barrier(&self, parent: &dyn HeapObject, child: &dyn HeapObject)`, `GcState::collect_if_needed(&self)`. All stubs log via `#[cfg(test)]` eprintln or no-op in release; they must not break existing behavior.

- [ ] **Step 1: Write the failing test**

Add to `tests/spec_tests.rs`:

```rust
#[test]
fn spec_gc_api_stubs_exist() {
    use latch_lang::env::{ObjFunctionBuilder, ObjRef, HeapObject};
    use latch_lang::vm::gc::GcState;

    let gc = GcState::new();
    let func = ObjRef::new(ObjFunctionBuilder::new("gc_stub", 0).build());
    gc.mark_root(&*func);
    gc.trace_object(&*func);
    gc.write_barrier(&*func, &*func);
    gc.sweep();
    gc.collect_if_needed();
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test spec_gc_api_stubs_exist -- --nocapture`
Expected: FAIL — methods not found.

- [ ] **Step 3: Implement `HeapObject` for `ObjFunction` and `ObjClosure`**

In `src/env.rs`, after `impl ObjClosure` and `impl ObjFunction` add:

```rust
impl HeapObject for ObjFunction {
    fn header(&self) -> &ObjHeader {
        &self.header
    }
}

impl HeapObject for ObjClosure {
    fn header(&self) -> &ObjHeader {
        &self.header
    }
}
```

- [ ] **Step 4: Add GC stub API to `GcState`**

In `src/vm/gc.rs`:

```rust
use crate::env::{HeapObject, GcTrace};

impl GcState {
    pub fn mark_root(&self, _obj: &dyn HeapObject) {
        // Stub: future GC will gray the object and enqueue it.
    }

    pub fn trace_object(&self, _obj: &dyn GcTrace) {
        // Stub: future GC will call trace() on the object.
    }

    pub fn sweep(&self) {
        // Stub: future GC will reclaim white objects.
    }

    pub fn write_barrier(&self, _parent: &dyn HeapObject, _child: &dyn HeapObject) {
        // Stub: generational / incremental GC barrier hook.
    }

    pub fn collect_if_needed(&self) {
        if self.should_collect() {
            self.sweep();
            self.update_threshold_after_sweep(0);
        }
    }
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test spec_gc_api_stubs_exist -- --nocapture`
Expected: PASS.

- [ ] **Step 6: Run full test suite for regressions**

Run: `cargo test`
Expected: all tests pass.

---

### Task 7: Introduce `NativeObject` trait

**Files:**
- Modify: `src/env.rs`
- Test: `tests/spec_tests.rs`

**Interfaces:**
- Consumes: `HeapObject`, `GcTrace`.
- Produces: `pub trait NativeObject: HeapObject + NativeCallable { fn type_name(&self) -> &'static str; fn finalize(&mut self) {} }` and a blanket `impl<T: NativeObject> GcTrace for T` if feasible; otherwise leave it for implementors.

- [ ] **Step 1: Write the failing test**

Add to `tests/spec_tests.rs`:

```rust
#[test]
fn spec_native_object_trait_exists() {
    use latch_lang::env::{NativeObject, NativeCallable, Value, Result};

    struct TestNative;
    impl NativeCallable for TestNative {
        fn call(&self, _args: &[Value]) -> Result<Value> {
            Ok(Value::Null)
        }
    }
    impl NativeObject for TestNative {
        fn type_name(&self) -> &'static str { "TestNative" }
    }

    let native = TestNative;
    assert_eq!(native.type_name(), "TestNative");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test spec_native_object_trait_exists -- --nocapture`
Expected: FAIL — `NativeObject` not found.

- [ ] **Step 3: Define the `NativeObject` trait**

In `src/env.rs`, after `NativeCallable`:

```rust
/// Native Dynamic Object Trait Contract for plugin / FFI objects.
pub trait NativeObject: HeapObject + NativeCallable {
    fn type_name(&self) -> &'static str;
    fn finalize(&mut self) {}
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test spec_native_object_trait_exists -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Run full test suite for regressions**

Run: `cargo test`
Expected: all tests pass.

---

### Task 8: Add allocation profiler and heap snapshot stubs

**Files:**
- Modify: `src/vm/profiler.rs`
- Modify: `src/vm/gc.rs`
- Test: `tests/spec_tests.rs`

**Interfaces:**
- Consumes: `GcState`.
- Produces: `VmProfiler::record_allocation(&mut self, kind: ObjKind, size: usize)`, `VmProfiler::allocation_summary(&self) -> Vec<(ObjKind, usize, usize)>` (count and total bytes), `GcState::heap_snapshot(&self) -> HeapSnapshot` where `HeapSnapshot` is a newtype over a Vec of `(ObjKind, usize)` stubs.

- [ ] **Step 1: Write the failing test**

Add to `tests/spec_tests.rs`:

```rust
#[test]
fn spec_allocation_profiler_records_allocations() {
    use latch_lang::env::ObjKind;
    use latch_lang::vm::profiler::VmProfiler;

    let mut profiler = VmProfiler::new();
    profiler.record_allocation(ObjKind::Function, 64);
    profiler.record_allocation(ObjKind::Function, 64);
    profiler.record_allocation(ObjKind::Class, 32);

    let summary = profiler.allocation_summary();
    let func_entry = summary.iter().find(|(k, _, _)| *k == ObjKind::Function).unwrap();
    assert_eq!(func_entry.1, 2); // count
    assert_eq!(func_entry.2, 128); // bytes
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test spec_allocation_profiler_records_allocations -- --nocapture`
Expected: FAIL — `record_allocation` not found.

- [ ] **Step 3: Add allocation tracking to `VmProfiler`**

In `src/vm/profiler.rs` (current content must be read first; if the struct is empty, add fields):

```rust
use std::collections::HashMap;
use crate::env::ObjKind;

pub struct VmProfiler {
    instruction_counts: HashMap<u8, u64>,
    allocations: HashMap<ObjKind, (usize, usize)>, // (count, total_bytes)
}

impl VmProfiler {
    pub fn new() -> Self {
        VmProfiler {
            instruction_counts: HashMap::new(),
            allocations: HashMap::new(),
        }
    }

    pub fn record_instruction(&mut self, opcode: u8) {
        *self.instruction_counts.entry(opcode).or_insert(0) += 1;
    }

    pub fn record_allocation(&mut self, kind: ObjKind, size: usize) {
        let entry = self.allocations.entry(kind).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += size;
    }

    pub fn allocation_summary(&self) -> Vec<(ObjKind, usize, usize)> {
        self.allocations
            .iter()
            .map(|(kind, (count, bytes))| (*kind, *count, *bytes))
            .collect()
    }
}
```

- [ ] **Step 4: Wire allocation recording into `GcState`**

In `src/vm/gc.rs`, inside `allocate`, `allocate_function`, `allocate_closure`, `allocate_class`, call `// profiler recording intentionally left out because GcState does not own a profiler`. Instead, keep the API clean; profiling is caller-side. (Alternatively, pass an optional profiler callback in a future plan.)

- [ ] **Step 5: Add heap snapshot stub to `GcState`**

In `src/vm/gc.rs`:

```rust
#[derive(Debug, Clone)]
pub struct HeapSnapshot {
    pub objects: Vec<(ObjKind, usize)>,
}

impl GcState {
    pub fn heap_snapshot(&self) -> HeapSnapshot {
        HeapSnapshot { objects: Vec::new() }
    }
}
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test spec_allocation_profiler_records_allocations -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Run full test suite for regressions**

Run: `cargo test`
Expected: all tests pass.

---

### Task 9: Finalize `docs/MEMORY_LAYOUT.md`

**Files:**
- Create/Modify: `docs/MEMORY_LAYOUT.md`

**Interfaces:**
- Produces: a complete memory layout document covering `Value`, `ObjHeader`, `ObjRef`, heap object layout, stack/frame layout, closure/upvalue ABI, allocation API, GC hooks, and native object interface.

- [ ] **Step 1: Write the full document**

Replace/extend `docs/MEMORY_LAYOUT.md` with:

```markdown
# Latch Memory Layout Specification

## 1. Value

`Value` is the runtime tagged union. This plan does **not** freeze its ABI; a future plan will split it into immediate values (`Int`, `Float`, `Bool`, `Null`) and a single heap object pointer variant.

## 2. ObjHeader

Every heap object starts with:

```text
ObjHeader
├── kind: ObjKind           (string, list, map, function, closure, class, instance, module, native)
├── flags: u8               (immutable / frozen / etc.)
├── mark: bool              (legacy mark bit, retained for compatibility)
├── gc_color: GcColor       (White / Gray / Black)
├── generation: u8           (generational GC generation)
├── size: usize              (object size in bytes, including header)
└── type_id: u32             (runtime type id, reserved for reflection)
```

## 3. ObjRef<T>

All heap references are `ObjRef<T>`, which currently wraps `Arc<T>`. The wrapper lets us swap the underlying pointer scheme (arena, moving GC, etc.) without touching call sites.

## 4. Heap Object Layouts

### ObjFunction
```text
ObjFunction
├── header: ObjHeader
├── arity: usize
├── chunk: Chunk
├── name: String
├── upvalue_count: usize
├── max_stack: usize
├── local_count: usize
├── module_id: u32
├── debug_id: u32
└── flags: u32
```

### ObjClosure
```text
ObjClosure
├── header: ObjHeader
├── function: ObjRef<ObjFunction>
└── upvalues: [Arc<Mutex<Value>>]
```

### ObjClass / ObjInstance
```text
ObjClass
├── header: ObjHeader
├── name: String
└── methods: HashMap<String, Value>

ObjInstance
├── header: ObjHeader
├── class: Arc<ObjClass>
└── fields: Arc<Mutex<HashMap<String, Value>>>
```

## 5. Stack and Frame

```text
ValueStack: [Value; N]
CallFrame
├── closure: ObjRef<ObjClosure>
├── ip: usize
├── slots: usize
├── return_slot: usize
├── stack_limit: usize
└── flags: u32
```

## 6. Allocation API

All object allocation goes through `GcState`:

```rust
let func = gc.allocate_function(ObjFunctionBuilder::new("f", 0).build());
let closure = gc.allocate_closure(func, Vec::new());
let class = gc.allocate_class("C");
```

## 7. GC Hooks

`GcState` exposes the following hooks for future collectors:
- `mark_root(&self, obj: &dyn HeapObject)`
- `trace_object(&self, obj: &dyn GcTrace)`
- `write_barrier(&self, parent: &dyn HeapObject, child: &dyn HeapObject)`
- `sweep(&self)`
- `collect_if_needed(&self)`

## 8. Native Object Interface

Native objects implement `NativeObject`:

```rust
trait NativeObject: HeapObject + NativeCallable {
    fn type_name(&self) -> &'static str;
    fn finalize(&mut self) {}
}
```
```

- [ ] **Step 2: Verify the file renders**

Run: `cat docs/MEMORY_LAYOUT.md | head -n 20`
Expected: the header and first sections are visible.

---

### Task 10: Final verification

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: all tests pass.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: no warnings (if the project was previously clean; new `#[allow(dead_code)]` may be needed for stub methods).

- [ ] **Step 3: Run the VM smoke test**

Run: `./target/debug/latch vm examples/vm_test.lt`
Expected: output contains `VM test passed!`.

- [ ] **Step 4: Report completion**

Confirm all checks passed and summarize the ABI freeze points: `ObjHeader` GC color, `ObjRef<T>` standardization, `ObjFunctionBuilder`, centralized `GcState` allocation API, closure/upvalue accessors, GC hooks, `NativeObject` trait, allocation profiler, and `MEMORY_LAYOUT.md`.

---

## Self-Review

**Spec coverage:**
- #051 (`Value` freeze) — deferred to a follow-up plan; all other tasks avoid touching `Value` ABI.
- #052 (`ObjHeader` hardening) — Task 1.
- #053 (`ObjKind` planning) — documented in `MEMORY_LAYOUT.md`, no code change needed.
- #054 (string interning) — deferred; requires runtime interner.
- #055 (List/Map trait) — partially addressed via `HeapObject`; full `ObjList`/`ObjMap` structs deferred to avoid `Value` ABI churn.
- #056 (Native Function ABI) — `NativeObject` trait in Task 7 and documented.
- #057/#058 (Closure/Upvalue ABI) — Task 5.
- #059 (`ObjFunction` immutable) — Task 3 builder.
- #060 (allocation API) — Task 4.
- #061 (`ObjRef<T>`) — Task 2.
- #062/#063 (GC API / write barrier) — Task 6.
- #064 (WeakRef) — deferred; no use sites exist yet.
- #065 (VM-GC decoupling) — Task 4 moves allocation into `GcState`.
- #066/#067 (heap snapshot / allocation profiler) — Task 8.
- #068 (`TypeId`) — already in `ObjHeader`; documented.
- #069 (`NativeObject`) — Task 7.
- #070 (`MEMORY_LAYOUT.md`) — Task 9.

**Placeholder scan:** No TBD/TODO placeholders in code steps; docs intentionally list future work as "deferred".

**Type consistency:** `ObjRef<T>` wraps `Arc<T>`; `VM::new` and `VmBuilder::new` accept `ObjRef<ObjFunction>`; `CallFrame::closure` is `ObjRef<ObjClosure>`; `GcState` allocation methods return `ObjRef<T>` throughout.
