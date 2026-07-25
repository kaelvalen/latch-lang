use crate::error::Result;
use super::vm::VM;

/// Native Module Interface for registering FFI dynamic extensions into Latch VM.
pub trait NativeModule {
    fn register(&self, vm: &mut VM) -> Result<()>;
}
