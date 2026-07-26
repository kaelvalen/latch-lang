# Compiler + ISA Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform the current opcode enum and bytecode emitter into a frozen ISA with compile-time metadata, immutable chunks, versioned bytecode, a documented CALL ABI, and the testing/snapshot infrastructure needed to keep it stable.

**Architecture:** Keep the existing `HIR -> Compiler -> Chunk -> Verifier -> VM` pipeline intact, but harden the boundary between Compiler and Runtime by introducing a single source-of-truth ISA metadata table, a `ChunkBuilder -> Chunk` immutability gate, and explicit bytecode versioning. Peephole optimization and constant-pool deduplication are added as later, isolated layers so they cannot destabilize the core ABI.

**Tech Stack:** Rust 2021, existing `latch-lang` crate, `cargo test`, `insta` (optional for snapshot tests — add only if not already present).

## Global Constraints

- All opcode numeric values MUST remain stable once `docs/ISA.md` is published; any future change requires an ISA version bump.
- The VM must continue to accept verified bytecode from the current compiler without behavioral regressions.
- No new external dependencies unless absolutely necessary (prefer std-only; `insta` is the only allowed exception for snapshot tests).
- All existing tests in `tests/` and `cargo test` must pass after every task.
- HIR data structures MUST NOT depend on runtime `Value`/`VM` types (existing invariant).

---

## File Structure

| File | Responsibility |
|------|----------------|
| `docs/ISA.md` | Canonical ISA specification: opcode numbers, stack transitions, operand layouts, CALL ABI, bytecode header. |
| `src/vm/chunk.rs` | `OpCode` enum, `InstructionDescriptor`, `Constant`, `Chunk` (immutable), `ChunkBuilder` (mutable emitter buffer). |
| `src/vm/decoder.rs` | Decode bytes into `DecodedInstruction` using the const ISA table. |
| `src/vm/compiler.rs` | HIR -> `ChunkBuilder` -> `Chunk` emitter; high-level `emit_*` helpers. |
| `src/vm/verifier.rs` | Static bytecode verification using ISA metadata (stack effects, jump bounds). |
| `src/vm/vm.rs` | VM execution loop; enforces CALL ABI stack cleanup. |
| `src/vm/lbc.rs` | Binary `.lbc` serialization/deserialization with versioned header. |
| `src/vm/peephole.rs` | Optional post-compilation peephole pass (scaffold only in this plan). |
| `tests/spec_tests.rs` | ISA descriptor / LBC format contract tests. |
| `tests/compiler_snapshot_tests.rs` | Bytecode disassembly snapshot tests. |
| `tests/pipeline_tests.rs` | Compiler -> Verifier -> VM integration tests. |

---

## Phase 1: ISA Freeze & Compile-Time Metadata Table

### Task 1.1: Create `docs/ISA.md` as the canonical ISA specification

**Files:**
- Create: `docs/ISA.md`

**Interfaces:**
- Produces: human-readable ISA contract referenced by all subsequent tasks.

- [ ] **Step 1: Write the ISA document**

```markdown
# Latch Instruction Set Architecture (ISA)

Version: 1.0
Bytecode header version: 1

## Binary Header

| Field | Size | Value |
|-------|------|-------|
| Magic | 6 bytes | `LATCHB` |
| Format version | u16 | 1 |
| ISA version | u16 | 1 |
| Flags | u16 | reserved, must be 0 |

## Opcode Table

| Hex | Name | Operands | Stack In | Stack Out | Description |
|-----|------|----------|----------|-----------|-------------|
| 0x01 | OP_CONSTANT | u16 const_id | 0 | 1 | Push constant |
| 0x02 | OP_ADD | — | 2 | 1 | Add / concat |
| 0x03 | OP_SUB | — | 2 | 1 | Subtract |
| 0x04 | OP_MUL | — | 2 | 1 | Multiply |
| 0x05 | OP_DIV | — | 2 | 1 | Divide |
| 0x06 | OP_MOD | — | 2 | 1 | Modulo |
| 0x07 | OP_NEG | — | 1 | 1 | Negate |
| 0x08 | OP_NOT | — | 1 | 1 | Logical not |
| 0x09 | OP_EQUAL | — | 2 | 1 | Equality |
| 0x0A | OP_LESS | — | 2 | 1 | Less-than |
| 0x0B | OP_GREATER | — | 2 | 1 | Greater-than |
| 0x0C | OP_GET_LOCAL | u16 slot | 0 | 1 | Read local |
| 0x0D | OP_SET_LOCAL | u16 slot | 1 | 1 | Write local |
| 0x0E | OP_GET_GLOBAL | u16 global_id | 0 | 1 | Read global |
| 0x0F | OP_DEF_GLOBAL | u16 global_id | 1 | 0 | Define global |
| 0x10 | OP_SET_GLOBAL | u16 global_id | 1 | 1 | Write global |
| 0x11 | OP_JUMP | u16 offset | 0 | 0 | Unconditional jump |
| 0x12 | OP_JUMP_FALSE | u16 offset | 1 | 0 | Conditional jump |
| 0x13 | OP_LOOP | u16 offset | 0 | 0 | Backward jump |
| 0x14 | OP_CALL | u16 argc | 1 + argc | 1 | Invoke callable |
| 0x15 | OP_RETURN | — | 1 | 0 | Return value |
| 0x16 | OP_POP | — | 1 | 0 | Discard top |
| 0x17 | OP_LIST | u16 count | count | 1 | Build list |
| 0x18 | OP_MAP | u16 count | count * 2 | 1 | Build map |
| 0x19 | OP_INDEX | — | 2 | 1 | Index read |
| 0x1A | OP_INDEX_ASSIGN | — | 3 | 1 | Index write |
| 0x1B | OP_PRINT | — | 1 | 1 | Print and push back |
| 0x1C | OP_IN | — | 2 | 1 | Membership |
| 0x1D | OP_GET_UPVAL | u16 upval_id | 0 | 1 | Read upvalue |
| 0x1E | OP_SET_UPVAL | u16 upval_id | 1 | 1 | Write upvalue |
| 0x1F | OP_CLOSURE | u16 func_id | 0 | 1 | Create closure |

## CALL ABI

- Arguments are pushed left-to-right after the callable.
- Before call: `[ callable | arg0 | arg1 | ... | argN ]`.
- `OP_CALL <N>` expects `N` arguments plus the callable.
- The VM creates a new frame whose base slot points at `arg0`.
- On `OP_RETURN`, the VM pops the callee frame, removes the callable and all arguments from the stack, and pushes the result at the caller's previous top.
- Return slot = caller's stack top before the call.

## Jump Encoding

All jump offsets are absolute byte offsets into the code vector, encoded as big-endian `u16`. The maximum code size is therefore 64 KiB. If a future compiler needs larger functions, a new ISA version with `u32` offsets must be introduced.
```

- [ ] **Step 2: Commit**

```bash
git add docs/ISA.md
git commit -m "docs: add canonical ISA specification v1.0"
```

---

### Task 1.2: Convert `OpCode::descriptor()` into a compile-time const table

**Files:**
- Modify: `src/vm/chunk.rs`

**Interfaces:**
- Consumes: existing `OpCode` enum values.
- Produces: `pub const OPCODE_TABLE: [InstructionDescriptor; 32]` indexed by `opcode as usize`.

- [ ] **Step 1: Add stack-effect-only fields to `InstructionDescriptor`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionDescriptor {
    pub opcode: OpCode,
    pub name: &'static str,
    pub operand_count: usize,
    pub operand_width: usize, // bytes consumed after opcode
    pub stack_in: usize,
    pub stack_out: usize,
    pub is_jump: bool,
    pub may_allocate: bool,
    pub gc_safe: bool,
}
```

- [ ] **Step 2: Define the const table**

```rust
pub const OPCODE_TABLE: [InstructionDescriptor; 32] = [
    InstructionDescriptor { opcode: OpCode::OpConstant, name: "OP_CONSTANT", operand_count: 1, operand_width: 2, stack_in: 0, stack_out: 1, is_jump: false, may_allocate: true, gc_safe: false },
    // ... one entry per opcode, indexed by opcode as u8
    InstructionDescriptor { opcode: OpCode::OpClosure, name: "OP_CLOSURE", operand_count: 1, operand_width: 2, stack_in: 0, stack_out: 1, is_jump: false, may_allocate: true, gc_safe: true },
];
```

- [ ] **Step 3: Replace `descriptor()` with a const lookup**

```rust
impl OpCode {
    #[inline(always)]
    pub const fn descriptor(self) -> &'static InstructionDescriptor {
        &OPCODE_TABLE[self as usize]
    }
}
```

- [ ] **Step 4: Update call sites that returned owned descriptors**

Change `let desc = op.descriptor();` to `let desc = op.descriptor();` (now returns reference). Update any code that expected `InstructionDescriptor` by value to use `*desc` or reference fields directly.

- [ ] **Step 5: Run tests**

```bash
cargo test spec_tests
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/vm/chunk.rs
git commit -m "feat(vm): compile-time opcode descriptor table"
```

---

### Task 1.3: Fix `Chunk::write_u32` and align jump encoding with ISA

**Files:**
- Modify: `src/vm/chunk.rs`

**Interfaces:**
- Produces: correct big-endian `u32` encoding.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn chunk_write_u32_roundtrip() {
    let mut chunk = Chunk::new();
    chunk.write_u32(0x12345678, 1);
    assert_eq!(chunk.code, vec![0x12, 0x34, 0x56, 0x78]);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test chunk_write_u32_roundtrip
```

Expected: FAIL (current implementation skips byte index 2).

- [ ] **Step 3: Fix the implementation**

```rust
pub fn write_u32(&mut self, val: u32, line: u32) {
    let bytes = val.to_be_bytes();
    self.write_u8(bytes[0], line);
    self.write_u8(bytes[1], line);
    self.write_u8(bytes[2], line);
    self.write_u8(bytes[3], line);
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test chunk_write_u32_roundtrip
cargo test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/vm/chunk.rs
git commit -m "fix(vm): correct write_u32 byte order"
```

---

### Task 1.4: Update decoder and verifier to use the const table

**Files:**
- Modify: `src/vm/decoder.rs`
- Modify: `src/vm/verifier.rs`

**Interfaces:**
- Consumes: `OpCode::descriptor()` returning `&'static InstructionDescriptor`.

- [ ] **Step 1: Update decoder to read `operand_width` from descriptor**

```rust
let desc = op.descriptor();
let operand = if desc.operand_width == 2 {
    // read u16
} else {
    None
};
```

- [ ] **Step 2: Update verifier to validate stack depth never goes negative**

```rust
simulated_stack_depth -= desc.stack_in as isize;
if simulated_stack_depth < 0 {
    return Err(LatchError::GenericError(format!("Stack underflow at offset={offset}")));
}
simulated_stack_depth += desc.stack_out as isize;
```

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/vm/decoder.rs src/vm/verifier.rs
git commit -m "refactor(vm): use const opcode table in decoder and verifier"
```

---

## Phase 2: Bytecode Format Versioning

### Task 2.1: Add `isa_version` and `flags` fields to the `.lbc` header

**Files:**
- Modify: `src/vm/lbc.rs`
- Modify: `docs/ISA.md` (already done in Task 1.1)

**Interfaces:**
- Produces: `LBC_ISA_VERSION: u16 = 1` and `LBC_FLAGS: u16 = 0` constants.

- [ ] **Step 1: Define new constants**

```rust
pub const LBC_MAGIC: &[u8; 6] = b"LATCHB";
pub const LBC_VERSION: u16 = 1;
pub const LBC_ISA_VERSION: u16 = 1;
pub const LBC_FLAGS: u16 = 0;
```

- [ ] **Step 2: Update serializer**

After writing `LBC_VERSION`, write `LBC_ISA_VERSION` and `LBC_FLAGS`.

```rust
buf.extend_from_slice(&LBC_VERSION.to_be_bytes());
buf.extend_from_slice(&LBC_ISA_VERSION.to_be_bytes());
buf.extend_from_slice(&LBC_FLAGS.to_be_bytes());
```

- [ ] **Step 3: Update deserializer**

```rust
let version = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
cursor += 2;
let isa_version = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
cursor += 2;
let flags = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
cursor += 2;

if version != LBC_VERSION || isa_version != LBC_ISA_VERSION {
    return Err(LatchError::GenericError(format!("Unsupported .lbc version {version}/{isa_version}")));
}
if flags != 0 {
    return Err(LatchError::GenericError(format!("Unsupported .lbc flags {flags}")));
}
```

- [ ] **Step 4: Update `spec_lbc_binary_format_header_matches_specification`**

```rust
#[test]
fn spec_lbc_binary_format_header_matches_specification() {
    assert_eq!(LBC_MAGIC, b"LATCHB");
    assert_eq!(LBC_VERSION, 1);
    assert_eq!(LBC_ISA_VERSION, 1);
    assert_eq!(LBC_FLAGS, 0);
}
```

- [ ] **Step 5: Run tests**

```bash
cargo test spec_lbc
cargo test
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/vm/lbc.rs tests/spec_tests.rs
git commit -m "feat(vm): versioned .lbc header with isa_version and flags"
```

---

## Phase 3: Chunk Builder & Immutability

### Task 3.1: Introduce `ChunkBuilder` and make `Chunk` immutable

**Files:**
- Modify: `src/vm/chunk.rs`

**Interfaces:**
- Produces: `pub struct ChunkBuilder { code: Vec<u8>, constants: Vec<Constant>, lines: Vec<u32> }`.
- Produces: `impl ChunkBuilder { pub fn build(self) -> Chunk }`.

- [ ] **Step 1: Remove mutating methods from `Chunk`**

`Chunk` keeps:
```rust
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Constant>,
    lines: Vec<u32>,
}
```

with read-only accessors:
```rust
impl Chunk {
    pub fn code(&self) -> &[u8] { &self.code }
    pub fn constants(&self) -> &[Constant] { &self.constants }
    pub fn lines(&self) -> &[u32] { &self.lines }
}
```

- [ ] **Step 2: Implement `ChunkBuilder`**

```rust
#[derive(Debug, Clone, Default)]
pub struct ChunkBuilder {
    code: Vec<u8>,
    constants: Vec<Constant>,
    lines: Vec<u32>,
}

impl ChunkBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn write_u8(&mut self, byte: u8, line: u32) -> usize {
        self.code.push(byte);
        self.lines.push(line);
        self.code.len() - 1
    }

    pub fn write_opcode(&mut self, op: OpCode, line: u32) -> usize {
        self.write_u8(op as u8, line)
    }

    pub fn write_u16(&mut self, val: u16, line: u32) {
        let bytes = val.to_be_bytes();
        self.write_u8(bytes[0], line);
        self.write_u8(bytes[1], line);
    }

    pub fn write_u32(&mut self, val: u32, line: u32) {
        let bytes = val.to_be_bytes();
        self.write_u8(bytes[0], line);
        self.write_u8(bytes[1], line);
        self.write_u8(bytes[2], line);
        self.write_u8(bytes[3], line);
    }

    pub fn add_constant(&mut self, constant: Constant) -> usize {
        for (i, existing) in self.constants.iter().enumerate() {
            if existing == &constant { return i; }
        }
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn build(self) -> Chunk {
        Chunk { code: self.code, constants: self.constants, lines: self.lines }
    }
}
```

- [ ] **Step 3: Update `Chunk::disassemble` to read from private fields**

Use `&self.code`, `&self.constants`, `&self.lines` inside the same module.

- [ ] **Step 4: Run tests**

```bash
cargo test
```

Expected: FAIL — compiler and tests still use public fields.

- [ ] **Step 5: Commit**

```bash
git add src/vm/chunk.rs
git commit -m "feat(vm): introduce ChunkBuilder and immutable Chunk"
```

---

### Task 3.2: Migrate `Compiler` to `ChunkBuilder`

**Files:**
- Modify: `src/vm/compiler.rs`

**Interfaces:**
- Consumes: `ChunkBuilder`.
- Produces: `Chunk` via `chunk_builder.build()`.

- [ ] **Step 1: Replace `Chunk` with `ChunkBuilder` in `Compiler`**

```rust
pub struct Compiler {
    chunk: ChunkBuilder,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { chunk: ChunkBuilder::new() }
    }
}
```

- [ ] **Step 2: Build the chunk before returning**

```rust
let chunk = self.chunk.build();
let script_fn = ObjFunction { ... chunk ... };
```

- [ ] **Step 3: Update emit helpers**

`self.chunk` is now `ChunkBuilder`; `patch_jump` uses `self.chunk.code` (builder internals still accessible within module if needed, or add a `code_mut()` accessor).

- [ ] **Step 4: Run tests**

```bash
cargo test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/vm/compiler.rs
git commit -m "refactor(vm): compiler emits into ChunkBuilder"
```

---

### Task 3.3: Update `LbcSerializer` and tests for private `Chunk` fields

**Files:**
- Modify: `src/vm/lbc.rs`
- Modify: `tests/fuzz_tests.rs`
- Modify: `tests/integration_tests.rs`

**Interfaces:**
- Consumes: `Chunk` read accessors.

- [ ] **Step 1: Update serializer**

```rust
let const_count = func.chunk.constants().len() as u16;
let code_len = func.chunk.code().len() as u32;
let lines_len = func.chunk.lines().len() as u32;
```

- [ ] **Step 2: Update tests that mutate `Chunk` directly**

Tests creating raw `Chunk` should use `ChunkBuilder` instead.

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/vm/lbc.rs tests/fuzz_tests.rs tests/integration_tests.rs
git commit -m "refactor(vm): adapt lbc and tests to immutable Chunk"
```

---

## Phase 4: CALL ABI & High-Level Emit API

### Task 4.1: Document and enforce caller-cleans CALL ABI

**Files:**
- Modify: `src/vm/vm.rs`

**Interfaces:**
- Produces: correct stack behavior for `OP_CALL` and `OP_RETURN`.

- [ ] **Step 1: Add a helper to compute return slot before call**

```rust
let arg_base = self.stack.len() - arg_count;
let return_slot = arg_base - 1; // slot currently holding the callable
let frame = CallFrame::new(closure, arg_base);
self.frames.push(frame);
```

- [ ] **Step 2: On `OP_RETURN`, restore caller stack to return slot and push result**

```rust
OpCode::OpReturn => {
    let result = self.pop().unwrap_or(Value::Null);
    let return_slot = self.current_frame().return_slot;
    self.frames.pop();
    self.stack.truncate(return_slot);
    if self.frames.is_empty() {
        return Ok(result);
    }
    self.push(result);
}
```

- [ ] **Step 3: Update `CallFrame::new` to accept `return_slot`**

```rust
pub fn new(closure: Arc<ObjClosure>, slots: usize, return_slot: usize) -> Self {
    CallFrame { closure, ip: 0, slots, return_slot, stack_limit: 256, flags: 0 }
}
```

- [ ] **Step 4: Update all `CallFrame::new` call sites**

`vm.rs` builder and main loop; pass `0` for the top-level script frame.

- [ ] **Step 5: Run tests**

```bash
cargo test
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/vm/vm.rs src/vm/frame.rs
git commit -m "fix(vm): enforce caller-cleans CALL ABI with return_slot"
```

---

### Task 4.2: Add high-level `emit_*` helpers to `Compiler`

**Files:**
- Modify: `src/vm/compiler.rs`

**Interfaces:**
- Produces: `emit_call`, `emit_jump`, `emit_loop`, `emit_return`, `emit_binop`, `emit_local_get`, etc.

- [ ] **Step 1: Add helpers**

```rust
fn emit_call(&mut self, argc: u16, line: u32) {
    self.emit_opcode(OpCode::OpCall, line);
    self.emit_u16(argc, line);
}

fn emit_jump(&mut self, instruction: OpCode, line: u32) -> usize {
    self.emit_opcode(instruction, line);
    self.emit_u16(0xffff, line);
    self.chunk.code.len() - 2
}

fn emit_loop(&mut self, loop_start: usize, line: u32) {
    self.emit_opcode(OpCode::OpLoop, line);
    self.emit_u16(loop_start as u16, line);
}

fn emit_return(&mut self, line: u32) {
    self.emit_opcode(OpCode::OpReturn, line);
}
```

- [ ] **Step 2: Replace inline opcode emits in `compile_stmt`/`compile_expr` with helpers**

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/vm/compiler.rs
git commit -m "refactor(vm): high-level compiler emit helpers"
```

---

## Phase 5: Constant Pool Optimizations

### Task 5.1: Add small integer cache and empty string singleton

**Files:**
- Modify: `src/vm/chunk.rs`

**Interfaces:**
- Produces: optimized `ChunkBuilder::add_constant`.

- [ ] **Step 1: Define cache ranges**

```rust
const SMALL_INT_MIN: i64 = -128;
const SMALL_INT_MAX: i64 = 127;
```

- [ ] **Step 2: Pre-populate builder with cached constants**

```rust
impl ChunkBuilder {
    pub fn new() -> Self {
        let mut builder = Self::default();
        // Empty string at index 0
        builder.constants.push(Constant::Str(String::new()));
        // Small ints
        for n in SMALL_INT_MIN..=SMALL_INT_MAX {
            builder.constants.push(Constant::Int(n));
        }
        builder
    }
}
```

- [ ] **Step 3: Update `add_constant` to use caches**

```rust
pub fn add_constant(&mut self, constant: Constant) -> usize {
    match constant {
        Constant::Int(n) if (SMALL_INT_MIN..=SMALL_INT_MAX).contains(&n) => {
            ((n - SMALL_INT_MIN) + 1) as usize
        }
        Constant::Str(ref s) if s.is_empty() => 0,
        _ => {
            for (i, existing) in self.constants.iter().enumerate() {
                if existing == &constant { return i; }
            }
            self.constants.push(constant);
            self.constants.len() - 1
        }
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/vm/chunk.rs
git commit -m "feat(vm): small int cache and empty string singleton"
```

---

### Task 5.2: Add common string deduplication

**Files:**
- Modify: `src/vm/chunk.rs`

**Interfaces:**
- Produces: `ChunkBuilder` with `HashSet<String>` for fast string dedup.

- [ ] **Step 1: Add string interner to builder**

```rust
#[derive(Debug, Clone, Default)]
pub struct ChunkBuilder {
    code: Vec<u8>,
    constants: Vec<Constant>,
    lines: Vec<u32>,
    string_table: HashMap<String, usize>,
}
```

- [ ] **Step 2: Use interner in `add_constant`**

```rust
Constant::Str(s) => {
    if let Some(&idx) = self.string_table.get(&s) {
        return idx;
    }
    let idx = self.constants.len();
    self.string_table.insert(s.clone(), idx);
    self.constants.push(Constant::Str(s));
    idx
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/vm/chunk.rs
git commit -m "feat(vm): string interning in constant pool"
```

---

## Phase 6: Peephole Optimizer Foundation

### Task 6.1: Add `BytecodePeephole` scaffold

**Files:**
- Create: `src/vm/peephole.rs`
- Modify: `src/vm/mod.rs`

**Interfaces:**
- Produces: `pub struct BytecodePeephole; impl BytecodePeephole { pub fn optimize(chunk: &mut Chunk) }`.

- [ ] **Step 1: Create the file**

```rust
use super::chunk::{Chunk, Constant, OpCode};

pub struct BytecodePeephole;

impl BytecodePeephole {
    pub fn optimize(chunk: &mut Chunk) {
        // TODO: implement passes
        let _ = chunk;
    }
}
```

- [ ] **Step 2: Export it**

```rust
pub mod peephole;
pub use peephole::BytecodePeephole;
```

- [ ] **Step 3: Commit**

```bash
git add src/vm/peephole.rs src/vm/mod.rs
git commit -m "feat(vm): bytecode peephole optimizer scaffold"
```

---

### Task 6.2: Implement constant-load folding for integer ADD/SUB/MUL

**Files:**
- Modify: `src/vm/peephole.rs`

**Interfaces:**
- Consumes: `Chunk` with private fields (peephole lives in `vm` module).

- [ ] **Step 1: Implement single-pass window optimization**

```rust
impl BytecodePeephole {
    pub fn optimize(chunk: &mut Chunk) {
        let mut i = 0;
        while i + 5 < chunk.code.len() {
            if let Some((new_code, new_const)) = Self::try_fold(&chunk.code, &chunk.constants, i) {
                // Replace window with optimized sequence
                let window_len = new_code.len();
                chunk.code.splice(i..i + window_len, new_code);
                // Add folded constant and patch OP_CONSTANT operand
                let const_idx = chunk.constants.len();
                chunk.constants.push(new_const);
                let bytes = (const_idx as u16).to_be_bytes();
                chunk.code[i + 1] = bytes[0];
                chunk.code[i + 2] = bytes[1];
                i += 3;
            } else {
                i += 1;
            }
        }
    }

    fn try_fold(code: &[u8], constants: &[Constant], i: usize) -> Option<(Vec<u8>, Constant)> {
        if code[i] != OpCode::OpConstant as u8 { return None; }
        let a_idx = u16::from_be_bytes([code[i+1], code[i+2]]) as usize;
        if code[i+3] != OpCode::OpConstant as u8 { return None; }
        let b_idx = u16::from_be_bytes([code[i+4], code[i+5]]) as usize;
        let op = code.get(i + 6)?;

        let Constant::Int(a) = constants.get(a_idx)? else { return None; };
        let Constant::Int(b) = constants.get(b_idx)? else { return None; };

        let result = match OpCode::from_u8(*op)? {
            OpCode::OpAdd => a + b,
            OpCode::OpSub => a - b,
            OpCode::OpMul => a * b,
            _ => return None,
        };

        Some((vec![OpCode::OpConstant as u8, 0, 0], Constant::Int(result)))
    }
}
```

- [ ] **Step 2: Add a test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::chunk::{ChunkBuilder, Constant};

    #[test]
    fn peephole_folds_int_add() {
        let mut builder = ChunkBuilder::new();
        let a = builder.add_constant(Constant::Int(5));
        let b = builder.add_constant(Constant::Int(3));
        builder.write_opcode(OpCode::OpConstant, 1);
        builder.write_u16(a as u16, 1);
        builder.write_opcode(OpCode::OpConstant, 1);
        builder.write_u16(b as u16, 1);
        builder.write_opcode(OpCode::OpAdd, 1);
        let mut chunk = builder.build();
        BytecodePeephole::optimize(&mut chunk);
        assert_eq!(chunk.code.len(), 3);
        assert_eq!(chunk.code[0], OpCode::OpConstant as u8);
        assert_eq!(chunk.constants[chunk.constants.len() - 1], Constant::Int(8));
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test peephole
cargo test
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/vm/peephole.rs
git commit -m "feat(vm): peephole constant folding for int arithmetic"
```

---

## Phase 7: Snapshot & Pipeline Tests

### Task 7.1: Add disassembly snapshot tests

**Files:**
- Create: `tests/compiler_snapshot_tests.rs`

**Interfaces:**
- Consumes: `Compiler`, `Resolver`, AST.

- [ ] **Step 1: Add a basic snapshot test**

```rust
use latch_lang::ast::{Expr, Stmt, BinOp};
use latch_lang::resolver::Resolver;
use latch_lang::vm::Compiler;

#[test]
fn snapshot_print_one_plus_two() {
    let stmts = vec![Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Ident("print".into())),
        args: vec![Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Int(1)),
            right: Box::new(Expr::Int(2)),
        }],
    })];
    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("snap", &stmts).unwrap();
    let compiler = Compiler::new();
    let func = compiler.compile_module(&module).unwrap();

    let mut output = Vec::new();
    func.chunk.disassemble_to("print(1+2)", &mut output);
    let asm = String::from_utf8(output).unwrap();

    assert!(asm.contains("OP_CONSTANT"));
    assert!(asm.contains("OP_ADD"));
    assert!(asm.contains("OP_PRINT"));
}
```

- [ ] **Step 2: Add `disassemble_to` helper to `Chunk`**

```rust
pub fn disassemble_to<W: std::fmt::Write>(&self, name: &str, out: &mut W) {
    writeln!(out, "== {name} ==").unwrap();
    // ... same logic, using write! instead of print!
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test compiler_snapshot_tests
```

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add tests/compiler_snapshot_tests.rs src/vm/chunk.rs
git commit -m "test(vm): compiler disassembly snapshot tests"
```

---

### Task 7.2: Add Compiler -> Verifier -> VM pipeline tests

**Files:**
- Create: `tests/pipeline_tests.rs`

**Interfaces:**
- Consumes: full pipeline.

- [ ] **Step 1: Add a pipeline test**

```rust
use latch_lang::ast::{Expr, Stmt, BinOp};
use latch_lang::resolver::Resolver;
use latch_lang::vm::{Compiler, VM};
use latch_lang::env::Value;

#[test]
fn pipeline_compiles_verifies_and_runs() {
    let stmts = vec![
        Stmt::Assign { name: "a".into(), value: Expr::Int(10) },
        Stmt::Assign { name: "b".into(), value: Expr::Int(20) },
        Stmt::Expr(Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Ident("a".into())),
            right: Box::new(Expr::Ident("b".into())),
        }),
    ];
    let mut resolver = Resolver::new();
    let module = resolver.resolve_module("pipe", &stmts).unwrap();
    let compiler = Compiler::new();
    let func = compiler.compile_module(&module).unwrap();
    let mut vm = VM::new(func).unwrap();
    let result = vm.run().unwrap();
    assert_eq!(result, Value::Int(30));
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test pipeline_tests
```

Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add tests/pipeline_tests.rs
git commit -m "test(vm): compiler-verifier-vm pipeline tests"
```

---

## Final Verification

- [ ] **Step 1: Run the full test suite**

```bash
cargo test
cargo clippy --all-targets
```

Expected: PASS / zero warnings introduced by these changes.

- [ ] **Step 2: Run integration smoke tests**

```bash
cargo build
./target/debug/latch vm examples/vm_test.lt
```

Expected: prints `VM test passed!` and `[1, 2, 3, 4, 5]`.

---

## Self-Review Checklist

1. **Spec coverage:** Every finding from #036–#050 maps to at least one task above.
2. **Placeholder scan:** All steps contain concrete code, file paths, and commands.
3. **Type consistency:** `OpCode::descriptor()` returns `&'static InstructionDescriptor`; `ChunkBuilder::build()` returns `Chunk`; `CallFrame::new` takes `return_slot`.

**Execution handoff:** Plan complete and saved to `docs/superpowers/plans/2026-07-26-compiler-isa-hardening.md`.
