# Latch Virtual Machine Specification (VM Spec v1.0)
**Formal Instruction Set Architecture (ISA) Specification & System Contracts**

---

## 1. Architecture Overview & Pipeline

Latch implements a decoupled 5-layer pipeline architecture. Syntactic representation is strictly separated from semantic resolution, intermediate representation, bytecode compilation, and virtual machine execution.

```text
Source Code (.lt)
    │
    ▼
[ Lexer ] ──> Token Stream
    │
    ▼
[ Parser ] ──> Pure AST (Syntax Only — Unresolved)
    │
    ▼
[ Resolver ] ──> Lexical Scoping, Upvalue Resolution, Slot Allocation
    │
    ▼
[ HIR ] ──> High-Level Intermediate Representation
    │
    ▼
[ Optimizer ] ──> HIR Constant Folding, DCE, Strength Reduction
    │
    ▼
[ Bytecode Compiler ] ──> Pure Bytecode Emitter (No Names, Only Slots & IDs)
    │
    ▼
[ Chunk ] ──> Immutable Bytecode Stream + Constants + Lines (.lbc)
    │
    ▼
[ Virtual Machine ] ──> Single Stack, Windowed Frames, O(1) Dispatch
```

---

## 2. Formal ISA Specification & Opcode Table

Every instruction in Latch VM has a strict binary layout, stack transition contract, failure condition specification, GC safepoint invariant, and optimizer rewrite contract.

| Opcode Name | Hex | Operands | Stack Transition | GC Safepoint | Failure Conditions | Optimizer Transformation Rules | Description |
|---|---|---|---|---|---|---|---|
| `OP_CONSTANT` | `0x01` | `u16 const_id` | `[] -> [val]` | No | `const_id >= constants.len()` | Constant Propagation | Loads constant from constant pool onto stack |
| `OP_ADD` | `0x02` | — | `[a, b] -> [a + b]` | Yes (str concat) | Type mismatch | Constant Folding if operands are literals | Numerical addition or string concatenation |
| `OP_SUB` | `0x03` | — | `[a, b] -> [a - b]` | No | Non-numeric operands | Constant Folding | Numerical subtraction |
| `OP_MUL` | `0x04` | — | `[a, b] -> [a * b]` | No | Non-numeric operands | Strength reduction (`x * 0 -> 0`, `x * 1 -> x`) | Numerical multiplication |
| `OP_DIV` | `0x05` | — | `[a, b] -> [a / b]` | No | Division by zero, non-numeric | Constant Folding | Numerical division |
| `OP_MOD` | `0x06` | — | `[a, b] -> [a % b]` | No | Division by zero, non-numeric | Constant Folding | Modulo division |
| `OP_NEG` | `0x07` | — | `[a] -> [-a]` | No | Non-numeric operand | Fold `Double Negation (-(-x) -> x)` | Unary negation |
| `OP_NOT` | `0x08` | — | `[a] -> [!a]` | No | None | Fold `Double Not (!(!x) -> x)` | Logical negation |
| `OP_EQUAL` | `0x09` | — | `[a, b] -> [bool]` | No | None | Fold constant comparisons | Structural equality check |
| `OP_LESS` | `0x0A` | — | `[a, b] -> [bool]` | No | Non-numeric comparison | Fold constant comparisons | Less-than comparison |
| `OP_GREATER` | `0x0B` | — | `[a, b] -> [bool]` | No | Non-numeric comparison | Fold constant comparisons | Greater-than comparison |
| `OP_GET_LOCAL`| `0x0C` | `u16 slot` | `[] -> [val]` | No | `slot >= stack.len()` | Copy propagation | Pushes local variable at `stack[frame.slots + slot]` |
| `OP_SET_LOCAL`| `0x0D` | `u16 slot` | `[val] -> [val]` | No | `slot >= stack.len()` | Dead store elimination | Writes to local slot `stack[frame.slots + slot]` |
| `OP_GET_GLOBAL`| `0x0E` | `u16 global_id`| `[] -> [val]` | No | Undefined variable error | Global ID caching | Pushes `module.globals[global_id]` |
| `OP_DEF_GLOBAL`| `0x0F` | `u16 global_id`| `[val] -> []` | Yes | Allocation failure | Dead store elimination | Defines `module.globals[global_id]` |
| `OP_SET_GLOBAL`| `0x10` | `u16 global_id`| `[val] -> [val]` | No | Undefined variable error | Dead store elimination | Sets `module.globals[global_id]` |
| `OP_JUMP` | `0x11` | `u16 offset` | `[] -> []` | No | Target out of bounds | Jump threading & NOP fusion | Unconditional jump to `offset` |
| `OP_JUMP_FALSE`| `0x12` | `u16 offset` | `[cond] -> []` | No | Target out of bounds | DCE on static condition (`if false`) | Conditional jump if condition is falsy |
| `OP_LOOP` | `0x13` | `u16 offset` | `[] -> []` | Yes | Target out of bounds | Unroll constant loops | Backward loop jump to `offset` |
| `OP_CALL` | `0x14` | `u16 argc` | `[fn, args...] -> [res]` | Yes | Arity mismatch, non-callable | Devirtualization & Inlining | Invokes closure or native function |
| `OP_RETURN` | `0x15` | — | `[val] -> []` | No | Empty call frame stack | Tail-call optimization | Truncates frame stack & returns value |
| `OP_POP` | `0x16` | — | `[val] -> []` | No | Stack underflow | Remove useless pop after side-effect-free expr | Discards top stack value |
| `OP_GET_UPVAL`| `0x1D` | `u16 upval_id` | `[] -> [val]` | No | `upval_id >= upvalues.len()` | Copy propagation | Reads upvalue from closure upvalue table |
| `OP_SET_UPVAL`| `0x1E` | `u16 upval_id` | `[val] -> [val]` | No | `upval_id >= upvalues.len()` | Dead store elimination | Writes upvalue to closure upvalue table |
| `OP_CLOSURE` | `0x1F` | `u16 func_id` | `[] -> [closure]`| Yes | Allocation failure | Hoist top-level closures | Instantiates closure wrapping function `constants[func_id]` |

---

## 3. Value Representation & Memory Model

### Discriminant Layout
```rust
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Object(Arc<ObjHeader>),
}
```

- **Immediates**: `Int`, `Float`, `Bool`, `Null` are 100% stack-contained with zero dynamic heap allocation.
- **Heap Objects**: All dynamic structures (`String`, `List`, `Map`, `Function`, `Closure`, `Class`, `Instance`, `Module`, `Native`) implement a unified `ObjHeader`.

---

## 4. Object Model & Header Layout

```rust
pub struct ObjHeader {
    pub kind: ObjKind,       // Object type discriminant
    pub is_marked: bool,     // GC Mark bit
    pub generation: u8,      // Generational GC field
    pub size: usize,         // Allocated byte payload
}
```

---

## 5. High-Level Intermediate Representation (HIR) & Resolver Invariant

Syntactic AST nodes are strictly converted into **HIR** by `src/resolver.rs` before optimizer or bytecode compilation:

- **Local Identifier**: `HirExpr::Local { slot }`
- **Global Identifier**: `HirExpr::Global { id }`
- **Upvalue Capture**: `HirExpr::Upvalue { slot }`

The compiler is a pure emitter operating on resolved HIR.

---

## 6. Binary Bytecode Format (`.lbc`) Specification

Latch bytecode binaries (`.lbc`) adhere to the following binary format:

```text
+-------------------------------------------------------+
| Magic Bytes: "LATCHB" (0x4C 0x41 0x54 0x43 0x48 0x42)  |
+-------------------------------------------------------+
| Version: u16 (e.g., 0x0001)                           |
+-------------------------------------------------------+
| Constant Count: u16                                   |
+-------------------------------------------------------+
| Constant Pool Payload (Encoded Values)                |
+-------------------------------------------------------+
| Code Length: u32                                      |
+-------------------------------------------------------+
| Bytecode Payload (Vec<u8>)                            |
+-------------------------------------------------------+
```

---

## 7. Native API & Foreign Function Interface (FFI)

Native C/Rust functions are bound as first-class `ObjNative` objects:
```rust
pub struct ObjNative {
    pub header: ObjHeader,
    pub name: String,
    pub function: fn(&[Value]) -> Result<Value>,
}
```

---

## 8. Profiling & Performance Metrics

Latch VM embeds a zero-cost profiling hook layer (`VmProfiler`):
- `instruction_count`: Total executed opcodes.
- `call_count`: Total function/closure invocations.
- `opcode_histogram`: Array tracking execution frequency per `OpCode`.
- `alloc_count`: Total heap allocations.
- `gc_time_ns`: Total duration spent in GC safepoint collection.

---

*Latch VM Specification v1.0 — Approved Formal ISA Contract*
