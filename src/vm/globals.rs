use crate::env::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalFlags {
    pub is_mutable: bool,
    pub is_exported: bool,
}

impl GlobalFlags {
    pub fn new() -> Self {
        GlobalFlags {
            is_mutable: true,
            is_exported: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Global {
    pub value: Value,
    pub flags: GlobalFlags,
}
