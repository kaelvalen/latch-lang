pub mod chunk;
pub mod compiler;
pub mod frame;
pub mod gc;
pub mod globals;
pub mod ic;
pub mod lbc;
pub mod memory;
pub mod optimizer;
pub mod profiler;
pub mod stack;
pub mod verifier;
pub mod vm;

#[allow(unused_imports)]
pub use chunk::{Chunk, InstructionDescriptor, OpCode};
pub use compiler::Compiler;
pub use frame::CallFrame;
pub use gc::GcState;
pub use globals::{Global, GlobalFlags};
pub use ic::{IcState, InlineCache};
pub use lbc::LbcSerializer;
pub use memory::NativeModule;
pub use optimizer::Optimizer;
pub use profiler::VmProfiler;
pub use stack::ValueStack;
pub use verifier::BytecodeVerifier;
pub use vm::VM;
