# Latch Memory Layout Specification

This document describes the stable ABI of Latch's object model, heap references, allocation API, and GC hooks. Decisions captured here are intended to outlive the current interpreter implementation and to support future work such as a generational GC, JIT compiler, and bytecode serializer.

## 1. `Value`

`Value` is the runtime tagged union used by the interpreter and the VM. This iteration of the object-model refactor intentionally **does not freeze the `Value` ABI**; a follow-up plan will split it into immediate values (`Int`, `Float`, `Bool`, `Null`) and a single heap-object pointer variant.

## 2. `ObjHeader`

Every heap object starts with a uniform header:

```text
ObjHeader
├── kind: ObjKind           (string, list, map, function, closure, class, instance, module, native)
├── flags: u8               (immutable / frozen / etc., reserved for future use)
├── mark: bool              (legacy mark bit, retained for compatibility)
├── gc_color: GcColor       (White / Gray / Black)
├── generation: u8           (generational GC generation)
├── size: usize              (object size in bytes, including header)
└── type_id: u32             (runtime type id, reserved for reflection)
```

`GcColor` is part of the stable ABI and must not be reordered or removed without a bytecode / object-format bump.

## 3. `ObjRef<T>`

All heap references are `ObjRef<T>`, which currently wraps `Arc<T>`. The wrapper exists so that the underlying pointer scheme can be swapped (arena, moving GC, tagged pointers, etc.) without changing call sites.

```rust
pub struct ObjRef<T>(pub Arc<T>);
```

`ObjRef<T>` implements `Clone`, `Deref`, and pointer equality via `ObjRef::ptr_eq`.

## 4. Heap Object Layouts

### `ObjFunction`

Immutable compiled function template:

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

Production code must construct `ObjFunction` only through `ObjFunctionBuilder`.

### `ObjClosure`

```text
ObjClosure
├── header: ObjHeader        (kind = Closure)
├── function: ObjRef<ObjFunction>
└── upvalues: [Arc<Mutex<Value>>]
```

- `function` is the shared compiled function template.
- `upvalues` is a fixed-size vector indexed by the upvalue slot in bytecode.
- The current representation stores a `Mutex<Value>` per upvalue. Future work will split this into open (stack pointer) and closed (boxed heap) upvalue representations while keeping the external ABI unchanged.

### `ObjClass` / `ObjInstance`

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

All object allocation must go through `GcState`:

```rust
let func = gc.allocate_function(ObjFunctionBuilder::new("f", 0));
let closure = gc.allocate_closure(func, Vec::new());
let class = gc.allocate_class("C");
```

`VM::alloc_*` methods are thin wrappers around the `GcState` API.

## 7. GC Hooks

`GcState` exposes the following hooks for future collectors:

- `mark_root(&self, obj: &dyn HeapObject)` — register a root object.
- `trace_object(&self, obj: &dyn GcTrace)` — trace an object's children.
- `write_barrier(&self, parent: &dyn HeapObject, child: &dyn HeapObject)` — generational / incremental barrier.
- `sweep(&self)` — reclaim unreachable objects.
- `collect_if_needed(&self)` — trigger collection when the threshold is exceeded.

All hooks are currently stubs; they validate the API surface without performing collection.

## 8. Native Object Interface

Native / plugin objects implement `NativeObject`:

```rust
pub trait NativeObject: HeapObject + NativeCallable {
    fn type_name(&self) -> &'static str;
    fn finalize(&mut self) {}
}
```

`NativeCallable` defines the call interface:

```rust
pub trait NativeCallable: Send + Sync {
    fn call(&self, args: &[Value]) -> Result<Value>;
}
```

## 9. Allocation Profiler

`VmProfiler` records per-object-kind allocation counts and bytes:

```rust
profiler.record_allocation(ObjKind::Function, 64);
let summary = profiler.allocation_summary(); // Vec<(ObjKind, count, total_bytes)>
```

## 10. Heap Snapshot

`GcState::heap_snapshot()` returns a `HeapSnapshot` for debugging. It is currently a stub and will be populated once the collector is implemented.
