# Latch Instruction Set Architecture (ISA)

**Version:** 1.0  
**Bytecode format version:** 1  
**ISA version:** 1  
**Status:** Frozen — numeric values and stack contracts are stable ABI.

---

## 1. Scope

This document is the canonical contract between:

- Compiler
- Bytecode emitter
- Verifier
- Virtual machine
- Debugger / disassembler
- LSP / optimizer

Once published, opcode numeric values, operand layouts, stack transitions, and the CALL ABI may only change with a new ISA version. Consumers MUST reject bytecode whose `isa_version` does not match.

---

## 2. Binary `.lbc` Header

Every serialized Latch bytecode file begins with:

| Field | Offset | Size | Value / Notes |
|-------|--------|------|---------------|
| Magic | 0 | 6 bytes | `LATCHB` (`0x4C 0x41 0x54 0x43 0x48 0x42`) |
| Format version | 6 | `u16` | Big-endian, currently `1` |
| ISA version | 8 | `u16` | Big-endian, currently `1` |
| Flags | 10 | `u16` | Reserved, MUST be `0` |
| Arity | 12 | `u16` | Function arity |
| Name length | 14 | `u16` | Length of function name in bytes |
| Name | 16 | variable | UTF-8 function name |
| Constant count | — | `u16` | Number of constants |
| Constant pool | — | variable | Typed constant payloads |
| Code length | — | `u32` | Bytecode length in bytes |
| Bytecode | — | variable | Instruction stream |
| Line length | — | `u32` | Line table length |
| Line table | — | variable | One `u32` per emitted byte |

`isa_version` is part of the stable ABI. A compiler that introduces a new opcode or changes an existing stack contract MUST bump `isa_version`.

---

## 3. Opcode Table

All multi-byte operands are big-endian. `slot` / `id` operands are `u16`.

| Hex | Name | Operands | Stack In | Stack Out | Description |
|-----|------|----------|----------|-----------|-------------|
| `0x00` | *(reserved)* | — | — | — | Invalid — must not appear in valid bytecode |
| `0x01` | `OP_CONSTANT` | `u16 const_id` | 0 | 1 | Push `constants[const_id]` onto the stack |
| `0x02` | `OP_ADD` | — | 2 | 1 | Pop `b`, pop `a`, push `a + b` (int/float/string) |
| `0x03` | `OP_SUB` | — | 2 | 1 | Pop `b`, pop `a`, push `a - b` |
| `0x04` | `OP_MUL` | — | 2 | 1 | Pop `b`, pop `a`, push `a * b` |
| `0x05` | `OP_DIV` | — | 2 | 1 | Pop `b`, pop `a`, push `a / b` |
| `0x06` | `OP_MOD` | — | 2 | 1 | Pop `b`, pop `a`, push `a % b` |
| `0x07` | `OP_NEG` | — | 1 | 1 | Pop `a`, push `-a` |
| `0x08` | `OP_NOT` | — | 1 | 1 | Pop `a`, push logical `!a` |
| `0x09` | `OP_EQUAL` | — | 2 | 1 | Pop `b`, pop `a`, push `a == b` |
| `0x0A` | `OP_LESS` | — | 2 | 1 | Pop `b`, pop `a`, push `a < b` |
| `0x0B` | `OP_GREATER` | — | 2 | 1 | Pop `b`, pop `a`, push `a > b` |
| `0x0C` | `OP_GET_LOCAL` | `u16 slot` | 0 | 1 | Push `stack[frame.slots + slot]` |
| `0x0D` | `OP_SET_LOCAL` | `u16 slot` | 1 | 1 | Write top of stack to `stack[frame.slots + slot]`; leave value on stack |
| `0x0E` | `OP_GET_GLOBAL` | `u16 global_id` | 0 | 1 | Push `globals[global_id].value` |
| `0x0F` | `OP_DEF_GLOBAL` | `u16 global_id` | 1 | 0 | Pop value, define `globals[global_id]` |
| `0x10` | `OP_SET_GLOBAL` | `u16 global_id` | 1 | 1 | Write top of stack to `globals[global_id]`; leave value on stack |
| `0x11` | `OP_JUMP` | `u16 offset` | 0 | 0 | Unconditional jump to absolute byte `offset` |
| `0x12` | `OP_JUMP_FALSE` | `u16 offset` | 1 | 0 | Pop condition; if falsy, jump to absolute byte `offset` |
| `0x13` | `OP_LOOP` | `u16 offset` | 0 | 0 | Backward jump to absolute byte `offset` |
| `0x14` | `OP_CALL` | `u16 argc` | `1 + argc` | 1 | Invoke callable with `argc` arguments; push result |
| `0x15` | `OP_RETURN` | — | 1 | 0 | Pop result, return to caller, push result in caller frame |
| `0x16` | `OP_POP` | — | 1 | 0 | Discard top of stack |
| `0x17` | `OP_LIST` | `u16 count` | `count` | 1 | Pop `count` values, build list, push list |
| `0x18` | `OP_MAP` | `u16 count` | `count * 2` | 1 | Pop `count` key/value pairs, build map, push map |
| `0x19` | `OP_INDEX` | — | 2 | 1 | Pop index, pop container, push `container[index]` |
| `0x1A` | `OP_INDEX_ASSIGN` | — | 3 | 1 | Pop value, pop index, pop container, assign, push value |
| `0x1B` | `OP_PRINT` | — | 1 | 1 | Print top of stack, leave value on stack |
| `0x1C` | `OP_IN` | — | 2 | 1 | Pop container, pop item, push membership result |
| `0x1D` | `OP_GET_UPVAL` | `u16 upval_id` | 0 | 1 | Push closure upvalue `upval_id` |
| `0x1E` | `OP_SET_UPVAL` | `u16 upval_id` | 1 | 1 | Write top of stack to closure upvalue `upval_id`; leave value |
| `0x1F` | `OP_CLOSURE` | `u16 func_id` | 0 | 1 | Wrap `constants[func_id]` function in a closure, push it |

### Stack effect notation

- `Stack In` is the number of values the instruction pops/consumes.
- `Stack Out` is the number of values the instruction pushes.
- Net stack delta = `Stack Out - Stack In`.

---

## 4. CALL ABI

### 4.1 Caller responsibility

Before `OP_CALL <argc>` the stack MUST contain:

```text
[ ... | callable | arg0 | arg1 | ... | arg<argc-1> ] <- top
```

Arguments are pushed left-to-right after the callable. `argc` does **not** include the callable.

### 4.2 Callee frame layout

The VM creates a new `CallFrame` whose base slot (`frame.slots`) points at `arg0`. The `return_slot` records the stack index that held the callable before the call. Locals are addressed relative to `frame.slots`:

```text
frame.slots + 0      -> arg0
frame.slots + 1      -> arg1
...
frame.slots + argc   -> local 0
frame.slots + argc+1 -> local 1
```

### 4.3 Return responsibility

On `OP_RETURN`:

1. Pop the result value.
2. Pop the callee frame.
3. Truncate the stack to `return_slot` (removing callable and all arguments).
4. If no caller frame remains, return the result as the script result.
5. Otherwise, push the result onto the caller's stack at `return_slot`.

This is a **caller-cleans** ABI: the callee only produces the result; the VM removes the call setup.

### 4.4 Native calls

Native functions receive their arguments as a slice. The VM is responsible for popping the callable and arguments and pushing the native result exactly as for a normal call.

---

## 5. Jump Encoding

All jump operands are **absolute byte offsets** into the code vector, encoded as big-endian `u16`. The maximum code size for a single function is therefore 64 KiB.

If a future compiler needs larger functions, a new ISA version with `u32` jump operands MUST be introduced; mixed-width jumps within one ISA version are forbidden.

---

## 6. Constant Pool ABI

Constants are referenced by `u16` index. Index `0` is reserved for the empty string singleton in optimized implementations. A small range of low indices (implementation-defined, typically `[-128, 127]` integers) may be reserved for cached small integers. These reservations are transparent to bytecode consumers: any valid `u16` index may be read from the pool.

---

## 7. Versioning Policy

- **Format version** bump: serialization layout change (e.g., new header fields).
- **ISA version** bump: any opcode numeric value, operand layout, stack contract, or CALL ABI change.
- Old `.lbc` files with a recognized format version but older ISA version MAY be accepted by a newer VM if the VM maintains backward compatibility; otherwise they MUST be rejected with a clear error.

---

*Latch ISA v1.0 — frozen ABI contract.*
