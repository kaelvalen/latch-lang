pub mod chunk;
pub mod compiler;
pub mod decoder;
pub mod frame;
pub mod gc;
pub mod globals;
pub mod ic;
pub mod lbc;
pub mod memory;
pub mod optimizer;
pub mod peephole;
pub mod profiler;
pub mod stack;
pub mod verifier;
#[allow(clippy::module_inception)]
pub mod vm;

pub use chunk::OPCODE_TABLE;
#[allow(unused_imports)]
pub use chunk::{Chunk, ChunkBuilder, Constant, InstructionDescriptor, OpCode};
pub use compiler::Compiler;
pub use decoder::{DecodedInstruction, InstructionCursor};
pub use frame::CallFrame;
pub use gc::GcState;
pub use globals::{Global, GlobalFlags};
pub use ic::{IcState, InlineCache};
pub use lbc::{LbcSerializer, LBC_FLAGS, LBC_ISA_VERSION, LBC_MAGIC, LBC_VERSION};
pub use memory::NativeModule;
pub use optimizer::Optimizer;
pub use peephole::BytecodePeephole;
pub use profiler::VmProfiler;
pub use stack::ValueStack;
pub use verifier::BytecodeVerifier;
pub use vm::{VerifiedProgram, VmBuilder, VM};
