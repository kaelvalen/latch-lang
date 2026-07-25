# Latch Virtual Machine Specification (VM Spec v1.0)
**Architectural Blueprint & ABI Contract for Latch Language System**

---

## Table of Contents
1. [Architecture Overview & Pipeline](#1-architecture-overview--pipeline)
2. [Instruction Set & Bytecode Encoding](#2-instruction-set--bytecode-encoding)
3. [Value Representation & ABI](#3-value-representation--abi)
4. [Object Model & Header Layout](#4-object-model--header-layout)
5. [Function, Closure & Upvalue Specification](#5-function-closure--upvalue-specification)
6. [Call Structure, Stack Windowing & Frames](#6-call-structure-stack-windowing--frames)
7. [Resolver & High-Level Intermediate Representation (HIR)](#7-resolver--high-level-intermediate-representation-hir)
8. [Module & Namespace Isolation](#8-module--namespace-isolation)
9. [Garbage Collection & Memory Model Invariants](#9-garbage-collection--memory-model-invariants)
10. [Optimizer Pipeline & Execution Contracts](#10-optimizer-pipeline--execution-contracts)
11. [Inline Caching & Polymorphic Call Site Protocol](#11-inline-caching--polymorphic-call-site-protocol)

---

## 1. Architecture Overview & Pipeline

Latch implements a decoupled 5-layer pipeline architecture. Syntactic representation is strictly separated from semantic resolution, intermediate representation, bytecode compilation, and virtual machine execution.

```text
Source Code
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
[ Optimizer ] ──> AST/HIR Constant Folding, DCE, Strength Reduction
    │
    ▼
[ Bytecode Compiler ] ──> Pure Bytecode Emitter (No Names, Only Slots & IDs)
    │
    ▼
[ Chunk ] ──> Immutable Bytecode Stream + Constants + Lines
    │
    ▼
[ Virtual Machine ] ──> Single Stack, Windowed Frames, O(1) Dispatch
```

### Key Architectural Invariants
1. **AST Invariant**: Pure syntax representation. Nodes contain no resolved slots, scope depths, or type annotations.
2. **Resolver Invariant**: Performs 100% of lexical scoping, shadowing, upvalue capture analysis, and slot assignments. Outputs resolved **HIR**.
3. **Compiler Invariant**: Pure code emitter. Operates exclusively on HIR; receives zero variable names, performing zero semantic validation.
4. **Chunk Invariant**: Immutable byte array (`code: Vec<u8>`), constant pool (`constants: Vec<Value>`), line table (`lines: Vec<u32>`).
5. **VM Invariant**: Execution loop operates over a single global `Vec<Value>` stack with windowed `CallFrame` offsets (`frame.slots`).

---

## 2. Instruction Set & Bytecode Encoding

### Instruction Format
Instructions are encoded as a 1-byte opcode (`u8`) followed by fixed-width Big-Endian immediate operands (`u16` / `u32`).

| Instruction | Opcode (`u8`) | Operand 1 (`u16`) | Operand 2 (`u16`) | Stack Transition | Description |
|---|---|---|---|---|---|
| `OP_CONSTANT` | `0x01` | `const_index` | — | `[] -> [value]` | Push constant from constant table |
| `OP_ADD` | `0x02` | — | — | `[a, b] -> [a + b]` | Add top two values |
| `OP_SUB` | `0x03` | — | — | `[a, b] -> [a - b]` | Subtract top two values |
| `OP_MUL` | `0x04` | — | — | `[a, b] -> [a * b]` | Multiply top two values |
| `OP_DIV` | `0x05` | — | — | `[a, b] -> [a / b]` | Divide top two values |
| `OP_MOD` | `0x06` | — | — | `[a, b] -> [a % b]` | Modulo top two values |
| `OP_NEG` | `0x07` | — | — | `[a] -> [-a]` | Negate top numeric value |
| `OP_NOT` | `0x08` | — | — | `[a] -> [!a]` | Logical NOT |
| `OP_EQUAL` | `0x09` | — | — | `[a, b] -> [bool]` | Structural equality check |
| `OP_LESS` | `0x0A` | — | — | `[a, b] -> [bool]` | Numeric less-than check |
| `OP_GREATER` | `0x0B` | — | — | `[a, b] -> [bool]` | Numeric greater-than check |
| `OP_GET_LOCAL` | `0x0C` | `slot_index` | — | `[] -> [val]` | Push `stack[frame.slots + slot]` |
| `OP_SET_LOCAL` | `0x0D` | `slot_index` | — | `[val] -> [val]` | Set `stack[frame.slots + slot]` |
| `OP_GET_GLOBAL`| `0x0E` | `global_id` | — | `[] -> [val]` | Push `module.globals[global_id]` |
| `OP_DEF_GLOBAL`| `0x0F` | `global_id` | — | `[val] -> []` | Define `module.globals[global_id]` |
| `OP_SET_GLOBAL`| `0x10` | `global_id` | — | `[val] -> [val]` | Set `module.globals[global_id]` |
| `OP_JUMP` | `0x11` | `target_offset`| — | `[] -> []` | Jump to absolute target offset |
| `OP_JUMP_FALSE`| `0x12` | `target_offset`| — | `[cond] -> []` | Jump if condition is falsy |
| `OP_LOOP` | `0x13` | `target_offset`| — | `[] -> []` | Loop back to target offset |
| `OP_CALL` | `0x14` | `arg_count` | — | `[fn, args...] -> [result]` | Invoke closure/function |
| `OP_RETURN` | `0x15` | — | — | `[val] -> []` | Pop frame & return result |
| `OP_POP` | `0x16` | — | — | `[val] -> []` | Discard top stack value |
| `OP_GET_UPVAL` | `0x1D` | `upval_index` | — | `[] -> [val]` | Read upvalue |
| `OP_SET_UPVAL` | `0x1E` | `upval_index` | — | `[val] -> [val]` | Write upvalue |
| `OP_CLOSURE` | `0x1F` | `func_const_id`| — | `[] -> [closure]` | Instantiate closure object |

---

## 3. Value Representation & ABI

### Value Discriminant Layout
Values are non-allocating immediate primitives or pointers to GC-managed heap objects (`Object*`).

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Object(Arc<ObjHeader>), // Heap pointer for GC-managed objects
}
```

### Immediate vs Heap Invariant
- `Int`, `Float`, `Bool`, `Null` are **immediates** stored directly on the stack or constant pool with zero heap allocations.
- All composite types (`String`, `List`, `Map`, `Function`, `Closure`, `Class`, `Instance`, `Module`) are heap-managed **Objects**.

---

## 4. Object Model & Header Layout

### Unified `ObjHeader` Contract
Every heap-allocated object begins with an identical `ObjHeader` memory block:

```rust
pub struct ObjHeader {
    pub kind: ObjKind,       // Object type discriminant
    pub is_marked: bool,     // Mark-and-Sweep GC mark bit
    pub generation: u8,      // Generational GC field (reserved)
    pub size: usize,         // Allocated byte size
}
```

### Core Heap Object Hierarchy
- `ObjString`: UTF-8 immutable interned string.
- `ObjList`: Dynamic array of `Value` (`Vec<Value>`).
- `ObjMap`: Hash map of `String` keys to `Value` (`HashBrown` / `HashMap`).
- `ObjFunction`: Immutable compiled bytecode block (`Chunk`).
- `ObjClosure`: Function reference + captured `Upvalue` array.
- `ObjClass`: Class definition (`name`, `methods`, field descriptors).
- `ObjInstance`: Class instance (`class`, field values).
- `ObjModule`: Isolated namespace (`globals`, `constants`, `exports`).
- `ObjNative`: C/Rust FFI native function binding.

---

## 5. Function, Closure & Upvalue Specification

### `ObjFunction` Contract
`ObjFunction` represents an immutable compiled function unit:
```rust
pub struct ObjFunction {
    pub header: ObjHeader,
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
    pub upvalue_count: usize,
    pub max_stack: usize,
}
```

### `ObjClosure` & Upvalue Capture Algorithm
When a nested function accesses a variable from its enclosing lexical scope, the Resolver marks the variable as an **Upvalue**.

```rust
pub enum Upvalue {
    Open(usize),        // Points to stack slot while enclosing frame is active
    Closed(Value),      // Moves to heap value when enclosing frame returns
}

pub struct ObjClosure {
    pub header: ObjHeader,
    pub function: Arc<ObjFunction>,
    pub upvalues: Vec<Arc<Mutex<Value>>>,
}
```

---

## 6. Call Structure, Stack Windowing & Frames

### Single Global Stack Architecture
The VM executes over a single continuous stack (`Vec<Value>`). Active function invocations do not allocate independent stack buffers.

### `CallFrame` Layout
```rust
pub struct CallFrame {
    pub closure: Arc<ObjClosure>,
    pub ip: usize,      // Instruction Pointer in closure.function.chunk
    pub slots: usize,   // Frame window offset in global stack
}
```

### Frame Execution Window Formula
Local variable slot access inside any frame:
$$\text{LocalAddress}(\text{slot}) = \text{stack}[\text{frame.slots} + \text{slot}]$$

### Call & Return ABI Contract
1. **Invocations (`OP_CALL <argc>`)**:
   - Target closure is located at `stack[top - argc - 1]`.
   - Frame window base is initialized at `slots = top - argc - 1`.
   - New `CallFrame` pushed to `VM.frames`.
2. **Returns (`OP_RETURN`)**:
   - Return value popped from stack.
   - Current frame popped from `VM.frames`.
   - Stack truncated: `stack.truncate(frame.slots)`.
   - Return value pushed to caller's stack slot.

---

## 7. Resolver & High-Level Intermediate Representation (HIR)

### Resolver Responsibilities
The `Resolver` performs a static semantic pass over the AST prior to compilation:
1. Lexical scope tracking, block nesting, and shadowing rules.
2. Identifiers resolved into exact index categories:
   - `Local(slot)`: Stack slot relative to frame.
   - `Global(id)`: Index in module globals array.
   - `Upvalue(slot)`: Index in closure upvalue array.
   - `Builtin(id)`: Index in native engine table.
3. Outputs resolved **HIR (High-Level Intermediate Representation)**.

---

## 8. Module & Namespace Isolation

Modules represent independent global namespaces:
```rust
pub struct ObjModule {
    pub header: ObjHeader,
    pub name: String,
    pub globals: Vec<Global>,
    pub exports: HashMap<String, usize>,
}
```
Global variables within a module are indexed O(1) via `globals: Vec<Global>`. There are zero global string hashmap lookups during bytecode execution.

---

## 9. Garbage Collection & Memory Model Invariants

- **Collector**: Mark-and-Sweep collector operating over registered heap objects (`ObjHeader`).
- **Roots**: VM global stack, active `CallFrame` closures, module global tables, and interned string pools.
- **Mark Phase**: Traverses reachable `ObjHeader` references setting `is_marked = true`.
- **Sweep Phase**: Reclaims unmarked heap object allocations.

---

## 10. Optimizer Pipeline & Execution Contracts

The HIR Optimizer performs pure transformations before bytecode emission:
1. **Constant Folding**: Evaluates static constant math and expressions.
2. **Dead Code Elimination (DCE)**: Removes unreachable `if false` branches and dead statements.
3. **Strength Reduction**: Replaces expensive operations (e.g., `x * 1` -> `x`, `x * 0` -> `0`).
4. **Jump Threading / Fusion**: Chains sequential jump instructions.

---

## 11. Inline Caching & Polymorphic Call Site Protocol

Dynamic field and method lookup sites reserve feedback slots for **Inline Caching**:
```rust
pub enum IcState {
    Uninitialized,
    Monomorphic { class_name: String, offset: usize },
    Polymorphic { slots: Vec<(String, usize)> },
    Megamorphic,
}
```
Sites transition from Uninitialized -> Monomorphic -> Polymorphic -> Megamorphic, ensuring O(1) method invocation performance for dynamic call sites.

---
*Latch VM Specification v1.0 — Approved & Frozen Architecture Contract*
