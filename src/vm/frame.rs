use std::sync::Arc;
use crate::env::ObjClosure;

/// A CallFrame represents an active function execution frame.
/// It tracks the executing ObjClosure, instruction pointer (ip), and base stack slot.
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub closure: Arc<ObjClosure>,
    pub ip: usize,
    pub slots: usize,
}
