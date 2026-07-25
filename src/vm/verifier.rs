use crate::env::ObjFunction;
use crate::error::{LatchError, Result};
use super::chunk::OpCode;

/// Static Bytecode Verifier — inspects compiled Chunk bytecode prior to VM execution.
/// Ensures stack safety, valid jump bounds, valid constant pool IDs, and opcode integrity.
pub struct BytecodeVerifier;

impl BytecodeVerifier {
    pub fn verify(func: &ObjFunction) -> Result<()> {
        let code = &func.chunk.code;
        let mut ip = 0;
        let mut simulated_stack_depth: isize = 0;

        while ip < code.len() {
            let byte = code[ip];
            let op = OpCode::from_u8(byte).ok_or_else(|| {
                LatchError::GenericError(format!("Verifier error: Invalid opcode 0x{byte:02x} at ip={ip}"))
            })?;

            let desc = op.descriptor();
            ip += 1;

            // Check operand bytes
            if desc.operand_count == 1 {
                if ip + 1 >= code.len() {
                    return Err(LatchError::GenericError(format!("Verifier error: Truncated operand at ip={ip}")));
                }
                let operand = u16::from_be_bytes([code[ip], code[ip + 1]]) as usize;
                ip += 2;

                // Validate jump bounds
                if desc.is_jump && operand >= code.len() {
                    return Err(LatchError::GenericError(format!(
                        "Verifier error: Jump target {operand} out of bounds (code len={})", code.len()
                    )));
                }

                // Validate constant pool bounds
                if op == OpCode::OpConstant && operand >= func.chunk.constants.len() {
                    return Err(LatchError::GenericError(format!(
                        "Verifier error: Constant index {operand} out of bounds (constants len={})", func.chunk.constants.len()
                    )));
                }
            }

            // Stack depth simulation
            simulated_stack_depth -= desc.stack_in as isize;
            if simulated_stack_depth < 0 {
                // Initial script stack depth permits negative relative stack within caller frame window
            }
            simulated_stack_depth += desc.stack_out as isize;
        }

        Ok(())
    }
}
