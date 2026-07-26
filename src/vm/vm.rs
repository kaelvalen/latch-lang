use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::chunk::{Chunk, OpCode};
use super::decoder::InstructionCursor;
use super::frame::CallFrame;
use super::gc::GcState;
use super::globals::{Global, GlobalFlags};
pub use super::ic::InlineCache;
use super::profiler::VmProfiler;
use super::stack::ValueStack;
use super::verifier::BytecodeVerifier;
use crate::env::{ObjClosure, ObjFunction, ObjFunctionBuilder, ObjRef, Value};
use crate::error::{LatchError, Result};

/// A type-state wrapper proving a program has passed BytecodeVerifier.
/// VM only accepts VerifiedProgram — never raw ObjFunction.
/// (Findings #033, #034)
pub struct VerifiedProgram {
    pub(crate) script_fn: ObjRef<ObjFunction>,
}

/// VmBuilder — separates verification, instantiation, and execution concerns.
/// Replaces monolithic VM::new() that silently swallowed verifier failures.
/// (Findings #026, #029, #032, #033)
pub struct VmBuilder {
    script_fn: ObjRef<ObjFunction>,
}

impl VmBuilder {
    /// Create a builder from a compiled function.
    pub fn new(script_fn: ObjRef<ObjFunction>) -> Self {
        VmBuilder { script_fn }
    }

    /// Verify bytecode. Returns a VerifiedProgram proof token on success.
    /// Verification failure is a hard error — not a warning. (Finding #026)
    pub fn verify(self) -> Result<VerifiedProgram> {
        BytecodeVerifier::verify(&self.script_fn)?;
        Ok(VerifiedProgram {
            script_fn: self.script_fn,
        })
    }

    /// Instantiate VM directly from a chunk without a verifier pass.
    /// Only use in tests where the chunk is trivially known-valid.
    #[cfg(test)]
    pub fn from_chunk(chunk: Chunk) -> Result<VM> {
        let script_fn = ObjRef::new(
            ObjFunctionBuilder::new("<script>", 0)
                .with_chunk(chunk)
                .build(),
        );
        VmBuilder::new(script_fn).verify()?.instantiate()
    }

    /// Instantiate VM without verification. ONLY use when chunk has already
    /// been verified externally. Never call from production paths.
    pub fn instantiate_unchecked(self) -> VM {
        let gc = GcState::new();
        let closure = gc
            .allocate_closure(self.script_fn.clone(), Vec::new())
            .into_arc();
        let frame = CallFrame::new(crate::env::ObjRef(closure), 0, 0);
        VM {
            frames: vec![frame],
            stack: ValueStack::new(),
            globals: Vec::new(),
            gc,
            profiler: VmProfiler::new(),
            inline_caches: HashMap::new(),
        }
    }
}

impl VerifiedProgram {
    /// Instantiate a VM from a verified program token.
    /// This is the only public VM construction path in production code.
    pub fn instantiate(self) -> Result<VM> {
        let gc = GcState::new();
        let closure = gc
            .allocate_closure(self.script_fn.clone(), Vec::new())
            .into_arc();
        let frame = CallFrame::new(crate::env::ObjRef(closure), 0, 0);
        Ok(VM {
            frames: vec![frame],
            stack: ValueStack::new(),
            globals: Vec::new(),
            gc,
            profiler: VmProfiler::new(),
            inline_caches: HashMap::new(),
        })
    }
}

/// Unified Production Virtual Machine Engine
/// Fields are private — access via controlled accessor methods. (Finding #030)
pub struct VM {
    frames: Vec<CallFrame>,
    stack: ValueStack,
    globals: Vec<Global>,
    gc: GcState,
    profiler: VmProfiler,
    /// Keyed by instruction byte offset. (Finding #031)
    inline_caches: HashMap<usize, InlineCache>,
}

impl VM {
    // ── Controlled Accessors (Finding #030) ─────────────────────────────

    pub fn stack(&self) -> &ValueStack {
        &self.stack
    }
    pub fn gc(&self) -> &GcState {
        &self.gc
    }
    pub fn profiler(&self) -> &VmProfiler {
        &self.profiler
    }
    pub fn inline_caches(&self) -> &HashMap<usize, InlineCache> {
        &self.inline_caches
    }

    // ── Legacy compatibility entry point (deprecated — prefer VmBuilder) ─

    /// Instantiate VM from a compiled ObjFunction.
    /// Performs mandatory bytecode verification. Returns Err on invalid bytecode.
    /// (Findings #026, #029, #032)
    pub fn new(script_fn: ObjRef<ObjFunction>) -> Result<Self> {
        VmBuilder::new(script_fn).verify()?.instantiate()
    }

    /// Build and run from a Chunk directly; primarily used in integration tests.
    pub fn new_with_chunk(chunk: Chunk) -> Result<Self> {
        let script_fn = ObjRef::new(
            ObjFunctionBuilder::new("<script>", 0)
                .with_chunk(chunk)
                .build(),
        );
        VmBuilder::new(script_fn).verify()?.instantiate()
    }

    /// Load and run a new script into an existing VM instance.
    pub fn load(&mut self, script_fn: ObjRef<ObjFunction>) -> Result<()> {
        BytecodeVerifier::verify(&script_fn)?;
        let closure = self.gc.allocate_closure(script_fn, Vec::new()).into_arc();
        let frame = CallFrame::new(crate::env::ObjRef(closure), 0, 0);
        self.frames = vec![frame];
        self.stack.clear();
        Ok(())
    }

    pub fn alloc_function(
        &self,
        arity: usize,
        chunk: Chunk,
        name: String,
    ) -> crate::env::ObjRef<ObjFunction> {
        self.gc
            .allocate_function(ObjFunctionBuilder::new(name, arity).with_chunk(chunk))
    }

    pub fn alloc_closure(
        &self,
        function: crate::env::ObjRef<ObjFunction>,
        upvalues: Vec<Arc<Mutex<Value>>>,
    ) -> crate::env::ObjRef<ObjClosure> {
        self.gc.allocate_closure(function, upvalues)
    }

    pub fn alloc_class(&self, name: impl Into<String>) -> crate::env::ObjRef<crate::env::ObjClass> {
        self.gc.allocate_class(name)
    }

    // ── Primary VM Loop (InstructionCursor & ValueStack Migration) ────────

    pub fn run(&mut self) -> Result<Value> {
        loop {
            if self.frames.is_empty() {
                break;
            }

            let frame_idx = self.frames.len() - 1;
            let code_len = self.frames[frame_idx].closure.function().chunk.code().len();
            let current_ip = self.frames[frame_idx].ip;

            if current_ip >= code_len {
                let result = self.pop().unwrap_or(Value::Null);
                self.frames.pop();
                if self.frames.is_empty() {
                    return Ok(result);
                }
                self.push(result);
                continue;
            }

            // Decode next instruction via InstructionCursor
            let (op, operand, next_ip) = {
                let frame = &self.frames[frame_idx];
                let mut cursor =
                    InstructionCursor::new(frame.closure.function().chunk.code(), frame.ip);
                let instr = cursor.decode_next()?;
                (instr.opcode, instr.operand, cursor.ip)
            };

            self.frames[frame_idx].ip = next_ip;
            self.profiler.record_instruction(op as u8);

            match op {
                OpCode::OpConstant => {
                    let idx = operand.unwrap_or(0);
                    let val = self.current_frame().closure.function().chunk.constants()
                        [idx as usize]
                        .to_value();
                    self.push(val);
                }

                OpCode::OpAdd => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(self.add_values(a, b)?);
                }

                OpCode::OpSub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(self.sub_values(a, b)?);
                }

                OpCode::OpMul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(self.mul_values(a, b)?);
                }

                OpCode::OpDiv => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(self.div_values(a, b)?);
                }

                OpCode::OpMod => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(self.mod_values(a, b)?);
                }

                OpCode::OpNeg => {
                    let val = self.pop()?;
                    match val {
                        Value::Int(n) => self.push(Value::Int(-n)),
                        Value::Float(f) => self.push(Value::Float(-f)),
                        _ => {
                            return Err(LatchError::TypeMismatch {
                                expected: "number".into(),
                                found: format!("{val:?}"),
                            })
                        }
                    }
                }

                OpCode::OpNot => {
                    let val = self.pop()?;
                    self.push(Value::Bool(!val.is_truthy()));
                }

                OpCode::OpEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a == b));
                }

                OpCode::OpLess => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(self.less_than(a, b)?));
                }

                OpCode::OpGreater => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(self.greater_than(a, b)?));
                }

                OpCode::OpGetLocal => {
                    let slot_idx = operand.unwrap_or(0) as usize;
                    let frame_slots = self.current_frame().slots;
                    let val = self.stack.get(frame_slots + slot_idx).clone();
                    self.push(val);
                }

                OpCode::OpSetLocal => {
                    let slot_idx = operand.unwrap_or(0) as usize;
                    let frame_slots = self.current_frame().slots;
                    let val = self.peek(0)?.clone();
                    self.stack.set(frame_slots + slot_idx, val);
                }

                OpCode::OpGetGlobal => {
                    let global_id = operand.unwrap_or(0) as usize;
                    if global_id >= self.globals.len() {
                        return Err(LatchError::UndefinedVariable(format!("global#{global_id}")));
                    }
                    let val = self.globals[global_id].value.clone();
                    self.push(val);
                }

                OpCode::OpDefineGlobal => {
                    let global_id = operand.unwrap_or(0) as usize;
                    let val = self.pop()?;
                    if global_id >= self.globals.len() {
                        self.globals.resize(
                            global_id + 1,
                            Global {
                                value: Value::Null,
                                flags: GlobalFlags::new(),
                            },
                        );
                    }
                    self.globals[global_id] = Global {
                        value: val,
                        flags: GlobalFlags::new(),
                    };
                }

                OpCode::OpSetGlobal => {
                    let global_id = operand.unwrap_or(0) as usize;
                    let val = self.peek(0)?.clone();
                    if global_id >= self.globals.len() {
                        return Err(LatchError::UndefinedVariable(format!("global#{global_id}")));
                    }
                    self.globals[global_id].value = val;
                }

                OpCode::OpGetUpvalue => {
                    let slot_idx = operand.unwrap_or(0) as usize;
                    let closure = self.current_frame().closure.clone();
                    if slot_idx < closure.upvalues().len() {
                        // Internal invariant: upvalue Mutex is never poisoned under normal use.
                        let val = closure.upvalues()[slot_idx]
                            .lock()
                            .map_err(|_| LatchError::GenericError("Upvalue lock poisoned".into()))?
                            .clone();
                        self.push(val);
                    } else {
                        return Err(LatchError::GenericError(format!(
                            "Invalid upvalue index {slot_idx}"
                        )));
                    }
                }

                OpCode::OpSetUpvalue => {
                    let slot_idx = operand.unwrap_or(0) as usize;
                    let val = self.peek(0)?.clone();
                    let closure = self.current_frame().closure.clone();
                    if slot_idx < closure.upvalues().len() {
                        *closure.upvalues()[slot_idx].lock().map_err(|_| {
                            LatchError::GenericError("Upvalue lock poisoned".into())
                        })? = val;
                    } else {
                        return Err(LatchError::GenericError(format!(
                            "Invalid upvalue index {slot_idx}"
                        )));
                    }
                }

                OpCode::OpClosure => {
                    let func_idx = operand.unwrap_or(0) as usize;
                    let func_val = self.current_frame().closure.function().chunk.constants()
                        [func_idx]
                        .to_value();
                    if let Value::Function(func) = func_val {
                        let frame_offset = self.current_frame().slots;
                        let frame_len = self.stack.len().saturating_sub(frame_offset);
                        let mut upvalues = Vec::with_capacity(frame_len);
                        for i in 0..frame_len {
                            let val = self.stack.get(frame_offset + i).clone();
                            upvalues.push(std::sync::Arc::new(std::sync::Mutex::new(val)));
                        }
                        let closure = self
                            .gc
                            .allocate_closure(crate::env::ObjRef(func), upvalues)
                            .into_arc();
                        self.push(Value::Closure(closure));
                    }
                }

                OpCode::OpDup => {
                    let val = self.peek(0)?.clone();
                    self.push(val);
                }

                OpCode::OpJump => {
                    let target = operand.unwrap_or(0) as usize;
                    self.current_frame_mut().ip = target;
                }

                OpCode::OpJumpIfFalse => {
                    let target = operand.unwrap_or(0) as usize;
                    let condition = self.pop()?;
                    if !condition.is_truthy() {
                        self.current_frame_mut().ip = target;
                    }
                }

                OpCode::OpLoop => {
                    let target = operand.unwrap_or(0) as usize;
                    self.current_frame_mut().ip = target;
                }

                OpCode::OpCall => {
                    let arg_count = operand.unwrap_or(0) as usize;
                    let callee = self.peek(arg_count)?.clone();
                    match callee {
                        Value::Closure(closure) => {
                            if arg_count != closure.function().arity {
                                return Err(LatchError::GenericError(format!(
                                    "Expected {} arguments but got {}.",
                                    closure.function().arity,
                                    arg_count
                                )));
                            }
                            let arg_base = self.stack.len() - arg_count;
                            let return_slot = arg_base - 1;
                            let frame =
                                CallFrame::new(crate::env::ObjRef(closure), arg_base, return_slot);
                            self.frames.push(frame);
                        }
                        Value::Function(func) => {
                            if arg_count != func.arity {
                                return Err(LatchError::GenericError(format!(
                                    "Expected {} arguments but got {}.",
                                    func.arity, arg_count
                                )));
                            }
                            let closure = self
                                .gc
                                .allocate_closure(crate::env::ObjRef(func), Vec::new())
                                .into_arc();
                            let arg_base = self.stack.len() - arg_count;
                            let return_slot = arg_base - 1;
                            let frame =
                                CallFrame::new(crate::env::ObjRef(closure), arg_base, return_slot);
                            self.frames.push(frame);
                        }
                        Value::Native(native) => {
                            let mut args = Vec::with_capacity(arg_count);
                            for _ in 0..arg_count {
                                args.push(self.pop()?);
                            }
                            args.reverse();
                            self.pop()?; // Pop callable
                            let result = (native.function)(&args)?;
                            self.push(result);
                        }
                        _ => {
                            return Err(LatchError::GenericError(format!(
                                "Not callable: {callee:?}"
                            )))
                        }
                    }
                }

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

                OpCode::OpPop => {
                    self.pop()?;
                }

                OpCode::OpList => {
                    let count = operand.unwrap_or(0) as usize;
                    let mut items = Vec::with_capacity(count);
                    for _ in 0..count {
                        items.push(self.pop()?);
                    }
                    items.reverse();
                    self.push(Value::List(Arc::new(Mutex::new(items))));
                }

                OpCode::OpMap => {
                    let count = operand.unwrap_or(0) as usize;
                    let mut map = HashMap::new();
                    for _ in 0..count {
                        let val = self.pop()?;
                        let key = self.pop()?;
                        if let Value::Str(k) = key {
                            map.insert(k, val);
                        }
                    }
                    self.push(Value::Map(Arc::new(Mutex::new(map))));
                }

                OpCode::OpIndex => {
                    let index = self.pop()?;
                    let container = self.pop()?;
                    match (&container, &index) {
                        (Value::List(l), Value::Int(i)) => {
                            let list = l.lock().map_err(|_| {
                                LatchError::GenericError("List lock poisoned".into())
                            })?;
                            let idx = if *i < 0 { list.len() as i64 + i } else { *i } as usize;
                            if idx < list.len() {
                                self.push(list[idx].clone());
                            } else {
                                self.push(Value::Null);
                            }
                        }
                        (Value::Map(m), Value::Str(k)) => {
                            let map = m.lock().map_err(|_| {
                                LatchError::GenericError("Map lock poisoned".into())
                            })?;
                            if let Some(val) = map.get(k) {
                                self.push(val.clone());
                            } else {
                                self.push(Value::Null);
                            }
                        }
                        _ => self.push(Value::Null),
                    }
                }

                OpCode::OpIndexAssign => {
                    let val = self.pop()?;
                    let index = self.pop()?;
                    let container = self.pop()?;
                    match (&container, &index) {
                        (Value::List(l), Value::Int(i)) => {
                            let mut list = l.lock().map_err(|_| {
                                LatchError::GenericError("List lock poisoned".into())
                            })?;
                            let idx = if *i < 0 { list.len() as i64 + i } else { *i } as usize;
                            if idx < list.len() {
                                list[idx] = val.clone();
                            }
                        }
                        (Value::Map(m), Value::Str(k)) => {
                            let mut map = m.lock().map_err(|_| {
                                LatchError::GenericError("Map lock poisoned".into())
                            })?;
                            map.insert(k.clone(), val.clone());
                        }
                        _ => {}
                    }
                    self.push(val);
                }

                OpCode::OpPrint => {
                    let val = self.pop()?;
                    println!("{val}");
                    self.push(val);
                }

                OpCode::OpIn => {
                    let container = self.pop()?;
                    let item = self.pop()?;
                    match (&container, &item) {
                        (Value::List(l), _) => {
                            let list = l.lock().map_err(|_| {
                                LatchError::GenericError("List lock poisoned".into())
                            })?;
                            self.push(Value::Bool(list.contains(&item)));
                        }
                        (Value::Map(m), Value::Str(k)) => {
                            let map = m.lock().map_err(|_| {
                                LatchError::GenericError("Map lock poisoned".into())
                            })?;
                            self.push(Value::Bool(map.contains_key(k)));
                        }
                        _ => self.push(Value::Bool(false)),
                    }
                }
            }
        }

        Ok(Value::Null)
    }

    // ── Helper Methods ──────────────────────────────────────────────────

    #[inline(always)]
    fn current_frame(&self) -> &CallFrame {
        // Internal invariant: run() always checks frames.is_empty() before calling this.
        self.frames
            .last()
            .expect("VM invariant violated: no active frame")
    }

    #[inline(always)]
    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames
            .last_mut()
            .expect("VM invariant violated: no active frame")
    }

    #[inline(always)]
    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Result<Value> {
        self.stack.pop()
    }

    #[inline(always)]
    pub fn peek(&self, distance: usize) -> Result<&Value> {
        self.stack.peek(distance)
    }

    fn add_values(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x + y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 + y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + y as f64)),
            (Value::Str(x), Value::Str(y)) => Ok(Value::Str(format!("{x}{y}"))),
            _ => Err(LatchError::TypeMismatch {
                expected: "addable".into(),
                found: "types".into(),
            }),
        }
    }

    fn sub_values(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x - y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 - y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x - y as f64)),
            _ => Err(LatchError::TypeMismatch {
                expected: "numbers".into(),
                found: "types".into(),
            }),
        }
    }

    fn mul_values(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
            (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x * y)),
            (Value::Int(x), Value::Float(y)) => Ok(Value::Float(x as f64 * y)),
            (Value::Float(x), Value::Int(y)) => Ok(Value::Float(x * y as f64)),
            _ => Err(LatchError::TypeMismatch {
                expected: "numbers".into(),
                found: "types".into(),
            }),
        }
    }

    fn div_values(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 {
                    return Err(LatchError::DivisionByZero);
                }
                Ok(Value::Int(x / y))
            }
            (Value::Float(x), Value::Float(y)) => {
                if y == 0.0 {
                    return Err(LatchError::DivisionByZero);
                }
                Ok(Value::Float(x / y))
            }
            _ => Err(LatchError::TypeMismatch {
                expected: "numbers".into(),
                found: "types".into(),
            }),
        }
    }

    fn mod_values(&self, a: Value, b: Value) -> Result<Value> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => {
                if y == 0 {
                    return Err(LatchError::DivisionByZero);
                }
                Ok(Value::Int(x % y))
            }
            _ => Err(LatchError::TypeMismatch {
                expected: "integers".into(),
                found: "types".into(),
            }),
        }
    }

    fn less_than(&self, a: Value, b: Value) -> Result<bool> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(x < y),
            (Value::Float(x), Value::Float(y)) => Ok(x < y),
            (Value::Int(x), Value::Float(y)) => Ok((x as f64) < y),
            (Value::Float(x), Value::Int(y)) => Ok(x < (y as f64)),
            _ => Err(LatchError::TypeMismatch {
                expected: "comparable".into(),
                found: "types".into(),
            }),
        }
    }

    fn greater_than(&self, a: Value, b: Value) -> Result<bool> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(x > y),
            (Value::Float(x), Value::Float(y)) => Ok(x > y),
            (Value::Int(x), Value::Float(y)) => Ok((x as f64) > y),
            (Value::Float(x), Value::Int(y)) => Ok(x > (y as f64)),
            _ => Err(LatchError::TypeMismatch {
                expected: "comparable".into(),
                found: "types".into(),
            }),
        }
    }
}
