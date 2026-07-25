use std::sync::Arc;
use crate::env::ObjClosure;

/// CallFrame represents an active function execution frame.
/// Tracks executing ObjClosure, instruction pointer (ip), base stack slot, return slot, stack limit, and frame flags.
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub closure: Arc<ObjClosure>,
    pub ip: usize,
    pub slots: usize,
    pub return_slot: usize,
    pub stack_limit: usize,
    pub flags: u32,
}

impl CallFrame {
    pub fn new(closure: Arc<ObjClosure>, slots: usize) -> Self {
        CallFrame {
            closure,
            ip: 0,
            slots,
            return_slot: slots,
            stack_limit: 256,
            flags: 0,
        }
    }
}
