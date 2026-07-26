use super::vm::VM;
use crate::error::Result;

/// Native Module Interface for registering FFI dynamic extensions into Latch VM.
pub trait NativeModule {
    fn register(&self, vm: &mut VM) -> Result<()>;
}
