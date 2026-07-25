use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::env::{ObjClosure, ObjFunction, Value};
use crate::error::{LatchError, Result};
use super::chunk::{Chunk, OpCode};
use super::frame::CallFrame;
use super::gc::GcState;
use super::globals::{Global, GlobalFlags};
use super::profiler::VmProfiler;

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: Vec<Global>,
    pub gc: GcState,
    pub profiler: VmProfiler,
}

impl VM {
    pub fn new(script_fn: Arc<ObjFunction>) -> Self {
        let func_ref = crate::env::ObjRef(script_fn);
        let closure = Arc::new(ObjClosure::new(func_ref, Vec::new()));
        let frame = CallFrame {
            closure,
            ip: 0,
            slots: 0,
        };
        VM {
            frames: vec![frame],
            stack: Vec::with_capacity(256),
            globals: Vec::new(),
            gc: GcState::new(),
            profiler: VmProfiler::new(),
        }
    }

    pub fn alloc_function(&self, arity: usize, chunk: Chunk, name: String) -> crate::env::ObjRef<ObjFunction> {
        self.gc.track_alloc(std::mem::size_of::<ObjFunction>());
        crate::env::ObjRef::new(ObjFunction {
            header: crate::env::ObjHeader::new(crate::env::ObjKind::Function),
            arity,
            chunk,
            name,
            upvalue_count: 0,
            max_stack: 256,
            local_count: 0,
            module_id: 0,
            debug_id: 0,
            flags: 0,
        })
    }

    pub fn alloc_closure(&self, function: crate::env::ObjRef<ObjFunction>, upvalues: Vec<Arc<Mutex<Value>>>) -> crate::env::ObjRef<ObjClosure> {
        self.gc.track_alloc(std::mem::size_of::<ObjClosure>());
        crate::env::ObjRef::new(ObjClosure::new(function, upvalues))
    }

    pub fn alloc_class(&self, name: impl Into<String>) -> crate::env::ObjRef<crate::env::ObjClass> {
        self.gc.track_alloc(std::mem::size_of::<crate::env::ObjClass>());
        crate::env::ObjRef::new(crate::env::ObjClass::new(name))
    }

    pub fn new_with_chunk(chunk: Chunk) -> Self {
        let script_fn = crate::env::ObjRef::new(ObjFunction {
            header: crate::env::ObjHeader::new(crate::env::ObjKind::Function),
            arity: 0,
            chunk,
            name: "<script>".into(),
            upvalue_count: 0,
            max_stack: 256,
            local_count: 0,
            module_id: 0,
            debug_id: 0,
            flags: 0,
        });
        Self::new(script_fn.0)
    }

    pub fn run(&mut self) -> Result<Value> {
        loop {
            if self.frames.is_empty() {
                break;
            }

            let frame_idx = self.frames.len() - 1;
            if self.frames[frame_idx].ip >= self.frames[frame_idx].closure.function.chunk.code.len() {
                let result = self.pop().unwrap_or(Value::Null);
                self.frames.pop();
                if self.frames.is_empty() {
                    return Ok(result);
                }
                self.push(result);
                continue;
            }

            let byte = self.read_byte();
            self.profiler.record_instruction(byte);
            let op = match OpCode::from_u8(byte) {
                Some(o) => o,
                None => return Err(LatchError::GenericError(format!("Invalid opcode 0x{byte:02x}"))),
            };

            match op {
                OpCode::OpConstant => {
                    let idx = self.read_u16();
                    let val = self.current_frame().closure.function.chunk.constants[idx as usize].clone();
                    self.push(val);
                }

                OpCode::OpAdd => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a.add(&b)?);
                }

                OpCode::OpSub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a.sub(&b)?);
                }

                OpCode::OpMul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a.mul(&b)?);
                }

                OpCode::OpDiv => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(a.div(&b)?);
                }

                OpCode::OpMod => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            if y == 0 { return Err(LatchError::DivisionByZero); }
                            self.push(Value::Int(x % y));
                        }
                        _ => return Err(LatchError::GenericError("Incompatible types for Mod".into())),
                    }
                }

                OpCode::OpNeg => {
                    let a = self.pop()?;
                    self.push(a.negate()?);
                }

                OpCode::OpNot => {
                    let a = self.pop()?;
                    self.push(Value::Bool(!a.is_truthy()));
                }

                OpCode::OpEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a == b));
                }

                OpCode::OpLess => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Bool(x < y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Bool(x < y)),
                        _ => return Err(LatchError::GenericError("Incompatible comparison".into())),
                    }
                }

                OpCode::OpGreater => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Bool(x > y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Bool(x > y)),
                        _ => return Err(LatchError::GenericError("Incompatible comparison".into())),
                    }
                }

                OpCode::OpIn => {
                    let container = self.pop()?;
                    let item = self.pop()?;
                    match container {
                        Value::List(list) => {
                            let guard = list.lock().unwrap();
                            let found = guard.iter().any(|v| v == &item);
                            self.push(Value::Bool(found));
                        }
                        Value::Str(s) => {
                            self.push(Value::Bool(s.contains(item.as_str()?)));
                        }
                        _ => return Err(LatchError::GenericError("Invalid container for 'in'".into())),
                    }
                }

                OpCode::OpDefineGlobal => {
                    let idx = self.read_u16() as usize;
                    let val = self.pop()?;
                    if idx >= self.globals.len() {
                        self.globals.resize(idx + 1, Global { value: Value::Null, flags: GlobalFlags::new() });
                    }
                    self.globals[idx] = Global { value: val, flags: GlobalFlags::new() };
                }

                OpCode::OpGetGlobal => {
                    let idx = self.read_u16() as usize;
                    if idx < self.globals.len() {
                        let val = self.globals[idx].value.clone();
                        self.push(val);
                    } else {
                        return Err(LatchError::UndefinedVariable(format!("global#{idx}")));
                    }
                }

                OpCode::OpSetGlobal => {
                    let idx = self.read_u16() as usize;
                    let val = self.peek(0)?.clone();
                    if idx < self.globals.len() {
                        self.globals[idx].value = val;
                    } else {
                        return Err(LatchError::UndefinedVariable(format!("global#{idx}")));
                    }
                }

                OpCode::OpGetLocal => {
                    let slot = self.read_u16() as usize;
                    let base = self.current_frame().slots;
                    let val = self.stack[base + slot].clone();
                    self.push(val);
                }

                OpCode::OpSetLocal => {
                    let slot = self.read_u16() as usize;
                    let base = self.current_frame().slots;
                    let val = self.peek(0)?.clone();
                    self.stack[base + slot] = val;
                }

                OpCode::OpGetUpvalue => {
                    let slot = self.read_u16() as usize;
                    let upval = &self.current_frame().closure.upvalues[slot];
                    let val = upval.lock().unwrap().clone();
                    self.push(val);
                }

                OpCode::OpSetUpvalue => {
                    let slot = self.read_u16() as usize;
                    let val = self.peek(0)?.clone();
                    let upval = &self.current_frame().closure.upvalues[slot];
                    *upval.lock().unwrap() = val;
                }

                OpCode::OpClosure => {
                    let func_idx = self.read_u16();
                    let func_val = self.current_frame().closure.function.chunk.constants[func_idx as usize].clone();
                    if let Value::Function(func) = func_val {
                        let closure = Arc::new(ObjClosure::new(crate::env::ObjRef(func), Vec::new()));
                        self.push(Value::Closure(closure));
                    }
                }

                OpCode::OpJump => {
                    let target = self.read_u16() as usize;
                    self.current_frame_mut().ip = target;
                }

                OpCode::OpJumpIfFalse => {
                    let target = self.read_u16() as usize;
                    let condition = self.peek(0)?;
                    if !condition.is_truthy() {
                        self.current_frame_mut().ip = target;
                    }
                }

                OpCode::OpLoop => {
                    let target = self.read_u16() as usize;
                    self.current_frame_mut().ip = target;
                }

                OpCode::OpCall => {
                    let arg_count = self.read_u16() as usize;
                    let callee = self.peek(arg_count)?.clone();
                    match callee {
                        Value::Closure(closure) => {
                            if arg_count != closure.function.arity {
                                return Err(LatchError::GenericError(format!(
                                    "Expected {} arguments but got {}.", closure.function.arity, arg_count
                                )));
                            }
                            let frame = CallFrame {
                                closure,
                                ip: 0,
                                slots: self.stack.len() - arg_count,
                            };
                            self.frames.push(frame);
                        }
                        Value::Function(func) => {
                            if arg_count != func.arity {
                                return Err(LatchError::GenericError(format!(
                                    "Expected {} arguments but got {}.", func.arity, arg_count
                                )));
                            }
                            let closure = Arc::new(ObjClosure::new(crate::env::ObjRef(func), Vec::new()));
                            let frame = CallFrame {
                                closure,
                                ip: 0,
                                slots: self.stack.len() - arg_count,
                            };
                            self.frames.push(frame);
                        }
                        Value::Native(native) => {
                            let mut args = Vec::with_capacity(arg_count);
                            for _ in 0..arg_count {
                                args.push(self.pop()?);
                            }
                            args.reverse();
                            self.pop()?; // Pop native callable
                            let res = (native.function)(&args)?;
                            self.push(res);
                        }
                        _ => return Err(LatchError::GenericError("Can only call functions".into())),
                    }
                }

                OpCode::OpPop => {
                    self.pop()?;
                }

                OpCode::OpList => {
                    let count = self.read_u16() as usize;
                    let mut items = Vec::with_capacity(count);
                    for _ in 0..count {
                        items.push(self.pop()?);
                    }
                    items.reverse();
                    self.push(Value::List(Arc::new(Mutex::new(items))));
                }

                OpCode::OpMap => {
                    let count = self.read_u16() as usize;
                    let mut map = HashMap::new();
                    for _ in 0..count {
                        let val = self.pop()?;
                        let key = self.pop()?.as_str()?.to_string();
                        map.insert(key, val);
                    }
                    self.push(Value::Map(Arc::new(Mutex::new(map))));
                }

                OpCode::OpIndex => {
                    let idx = self.pop()?;
                    let container = self.pop()?;
                    match (container, idx) {
                        (Value::List(l), Value::Int(i)) => {
                            let guard = l.lock().unwrap();
                            if i < 0 || i as usize >= guard.len() {
                                return Err(LatchError::IndexOutOfBounds { index: i, len: guard.len() });
                            }
                            self.push(guard[i as usize].clone());
                        }
                        (Value::Map(m), Value::Str(k)) => {
                            let guard = m.lock().unwrap();
                            if let Some(val) = guard.get(&k) {
                                self.push(val.clone());
                            } else {
                                return Err(LatchError::KeyNotFound(k));
                            }
                        }
                        _ => return Err(LatchError::GenericError("Invalid index access".into())),
                    }
                }

                OpCode::OpPrint => {
                    let val = self.pop()?;
                    println!("{val}");
                    self.push(Value::Null);
                }

                OpCode::OpReturn => {
                    let result = self.pop().unwrap_or(Value::Null);
                    let frame = self.frames.pop().unwrap();
                    self.stack.truncate(frame.slots);
                    if self.frames.is_empty() {
                        return Ok(result);
                    }
                    self.push(result);
                }

                _ => {}
            }
        }

        Ok(Value::Null)
    }

    #[inline(always)]
    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    #[inline(always)]
    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().unwrap();
        let b = frame.closure.function.chunk.code[frame.ip];
        frame.ip += 1;
        b
    }

    #[inline(always)]
    fn read_u16(&mut self) -> u16 {
        let b1 = self.read_byte();
        let b2 = self.read_byte();
        u16::from_be_bytes([b1, b2])
    }

    #[inline(always)]
    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    #[inline(always)]
    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or_else(|| LatchError::GenericError("Stack underflow".into()))
    }

    #[inline(always)]
    fn peek(&self, distance: usize) -> Result<&Value> {
        if distance >= self.stack.len() {
            Err(LatchError::GenericError("Stack underflow".into()))
        } else {
            Ok(&self.stack[self.stack.len() - 1 - distance])
        }
    }
}
