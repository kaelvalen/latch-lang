use super::chunk::OpCode;
use super::decoder::InstructionCursor;
use crate::env::ObjFunction;
use crate::error::{LatchError, Result};

/// Enhanced Static Bytecode Verifier (CFG, Stack Depth Simulation, Bound Checks)
pub struct BytecodeVerifier;

impl BytecodeVerifier {
    pub fn verify(func: &ObjFunction) -> Result<()> {
        let code = func.chunk.code();
        let mut cursor = InstructionCursor::new(code, 0);

        let mut simulated_stack_depth: isize = 0;

        while !cursor.is_at_end() {
            let offset = cursor.ip;
            let instr = cursor.decode_next()?;
            let desc = instr.opcode.descriptor();

            // Validate jump boundaries
            if desc.is_jump {
                if let Some(target) = instr.operand {
                    if (target as usize) >= code.len() {
                        return Err(LatchError::GenericError(format!(
                            "Verifier error: Jump target {} out of code bounds (code len={}) at offset={offset}",
                            target, code.len()
                        )));
                    }
                }
            }

            // Validate constant pool bounds
            if instr.opcode == OpCode::OpConstant {
                if let Some(const_idx) = instr.operand {
                    if (const_idx as usize) >= func.chunk.constants().len() {
                        return Err(LatchError::GenericError(format!(
                            "Verifier error: Constant index {} out of constants bounds (len={}) at offset={offset}",
                            const_idx, func.chunk.constants().len()
                        )));
                    }
                }
            }

            // Stack depth simulation
            match instr.opcode {
                OpCode::OpList | OpCode::OpMap => {
                    let count = instr.operand.unwrap_or(0) as isize;
                    let consumed = if instr.opcode == OpCode::OpList {
                        count
                    } else {
                        count * 2
                    };
                    simulated_stack_depth -= consumed;
                    if simulated_stack_depth < 0 {
                        return Err(LatchError::GenericError(format!(
                            "Verifier error: Stack underflow for {} at offset={offset}",
                            desc.name
                        )));
                    }
                    simulated_stack_depth += 1;
                }
                _ => {
                    simulated_stack_depth -= desc.stack_in as isize;
                    if simulated_stack_depth < 0 {
                        return Err(LatchError::GenericError(format!(
                            "Verifier error: Stack underflow for {} at offset={offset}",
                            desc.name
                        )));
                    }
                    simulated_stack_depth += desc.stack_out as isize;
                }
            }
        }

        Ok(())
    }
}
