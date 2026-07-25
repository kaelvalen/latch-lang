use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::env::Value;
use crate::error::{LatchError, Result};
use super::chunk::{Chunk, OpCode};

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        VM {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<Value> {
        loop {
            if self.ip >= self.chunk.code.len() {
                break;
            }

            let byte = self.read_byte();
            let op = match OpCode::from_u8(byte) {
                Some(o) => o,
                None => return Err(LatchError::GenericError(format!("Invalid opcode 0x{byte:02x} at ip={}", self.ip - 1))),
            };

            match op {
                OpCode::OpConstant => {
                    let idx = self.read_u16();
                    let val = self.chunk.constants[idx as usize].clone();
                    self.push(val);
                }

                OpCode::OpAdd => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x + y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x + y)),
                        (Value::Str(x), Value::Str(y)) => self.push(Value::Str(format!("{x}{y}"))),
                        _ => return Err(LatchError::GenericError("Incompatible types for Add".into())),
                    }
                }

                OpCode::OpSub => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x - y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x - y)),
                        _ => return Err(LatchError::GenericError("Incompatible types for Sub".into())),
                    }
                }

                OpCode::OpMul => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.push(Value::Int(x * y)),
                        (Value::Float(x), Value::Float(y)) => self.push(Value::Float(x * y)),
                        _ => return Err(LatchError::GenericError("Incompatible types for Mul".into())),
                    }
                }

                OpCode::OpDiv => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => {
                            if y == 0 { return Err(LatchError::DivisionByZero); }
                            self.push(Value::Int(x / y));
                        }
                        (Value::Float(x), Value::Float(y)) => {
                            if y == 0.0 { return Err(LatchError::DivisionByZero); }
                            self.push(Value::Float(x / y));
                        }
                        _ => return Err(LatchError::GenericError("Incompatible types for Div".into())),
                    }
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
                    match a {
                        Value::Int(x) => self.push(Value::Int(-x)),
                        Value::Float(x) => self.push(Value::Float(-x)),
                        _ => return Err(LatchError::GenericError("Invalid type for negation".into())),
                    }
                }

                OpCode::OpNot => {
                    let a = self.pop()?;
                    self.push(Value::Bool(!a.is_truthy()));
                }

                OpCode::OpEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(format!("{a}") == format!("{b}")));
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
                            let found = guard.iter().any(|v| format!("{v}") == format!("{item}"));
                            self.push(Value::Bool(found));
                        }
                        Value::Str(s) => {
                            self.push(Value::Bool(s.contains(item.as_str()?)));
                        }
                        _ => return Err(LatchError::GenericError("Invalid container for 'in'".into())),
                    }
                }

                OpCode::OpDefineGlobal => {
                    let idx = self.read_u16();
                    let name = self.chunk.constants[idx as usize].as_str()?.to_string();
                    let val = self.pop()?;
                    self.globals.insert(name, val);
                }

                OpCode::OpGetGlobal => {
                    let idx = self.read_u16();
                    let name = self.chunk.constants[idx as usize].as_str()?;
                    if let Some(val) = self.globals.get(name) {
                        self.push(val.clone());
                    } else {
                        return Err(LatchError::UndefinedVariable(name.to_string()));
                    }
                }

                OpCode::OpSetGlobal => {
                    let idx = self.read_u16();
                    let name = self.chunk.constants[idx as usize].as_str()?.to_string();
                    let val = self.peek(0)?.clone();
                    if self.globals.contains_key(&name) {
                        self.globals.insert(name, val);
                    } else {
                        return Err(LatchError::UndefinedVariable(name));
                    }
                }

                OpCode::OpGetLocal => {
                    let slot = self.read_u16() as usize;
                    let val = self.stack[slot].clone();
                    self.push(val);
                }

                OpCode::OpSetLocal => {
                    let slot = self.read_u16() as usize;
                    let val = self.peek(0)?.clone();
                    self.stack[slot] = val;
                }

                OpCode::OpJump => {
                    let target = self.read_u16() as usize;
                    self.ip = target;
                }

                OpCode::OpJumpIfFalse => {
                    let target = self.read_u16() as usize;
                    let condition = self.peek(0)?;
                    if !condition.is_truthy() {
                        self.ip = target;
                    }
                }

                OpCode::OpLoop => {
                    let target = self.read_u16() as usize;
                    self.ip = target;
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
                    let val = self.pop().unwrap_or(Value::Null);
                    return Ok(val);
                }

                _ => {}
            }
        }

        Ok(Value::Null)
    }

    fn read_byte(&mut self) -> u8 {
        let b = self.chunk.code[self.ip];
        self.ip += 1;
        b
    }

    fn read_u16(&mut self) -> u16 {
        let b1 = self.read_byte();
        let b2 = self.read_byte();
        u16::from_be_bytes([b1, b2])
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or_else(|| LatchError::GenericError("Stack underflow".into()))
    }

    fn peek(&self, distance: usize) -> Result<&Value> {
        if distance >= self.stack.len() {
            Err(LatchError::GenericError("Stack underflow".into()))
        } else {
            Ok(&self.stack[self.stack.len() - 1 - distance])
        }
    }
}
