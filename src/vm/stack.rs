use crate::env::Value;
use crate::error::{LatchError, Result};

pub struct ValueStack {
    stack: Vec<Value>,
}

impl ValueStack {
    pub fn new() -> Self {
        ValueStack {
            stack: Vec::with_capacity(256),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or_else(|| LatchError::GenericError("Stack underflow".into()))
    }

    #[inline(always)]
    pub fn peek(&self, distance: usize) -> Result<&Value> {
        if distance >= self.stack.len() {
            Err(LatchError::GenericError("Stack underflow".into()))
        } else {
            Ok(&self.stack[self.stack.len() - 1 - distance])
        }
    }

    #[inline(always)]
    pub fn get(&self, index: usize) -> &Value {
        &self.stack[index]
    }

    #[inline(always)]
    pub fn set(&mut self, index: usize, val: Value) {
        self.stack[index] = val;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    #[inline(always)]
    pub fn truncate(&mut self, len: usize) {
        self.stack.truncate(len);
    }
}
