use crate::error::{LatchError, Result};
use super::chunk::OpCode;

/// Decoded Single Bytecode Instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedInstruction {
    pub opcode: OpCode,
    pub operand: Option<u16>,
    pub offset: usize,
}

/// Zero-Cost Bytecode Decoder Cursor — isolates instruction reading from the main VM loop.
pub struct InstructionCursor<'a> {
    code: &'a [u8],
    pub ip: usize,
}

impl<'a> InstructionCursor<'a> {
    pub fn new(code: &'a [u8], ip: usize) -> Self {
        InstructionCursor { code, ip }
    }

    #[inline(always)]
    pub fn is_at_end(&self) -> bool {
        self.ip >= self.code.len()
    }

    #[inline(always)]
    pub fn decode_next(&mut self) -> Result<DecodedInstruction> {
        if self.ip >= self.code.len() {
            return Err(LatchError::GenericError("Unexpected EOF in bytecode stream".into()));
        }

        let offset = self.ip;
        let byte = self.code[self.ip];
        let op = OpCode::from_u8(byte).ok_or_else(|| {
            LatchError::GenericError(format!("Invalid opcode 0x{byte:02x} at offset={offset}"))
        })?;

        self.ip += 1;
        let desc = op.descriptor();

        let operand = if desc.operand_width == 2 {
            if self.ip + 1 > self.code.len() {
                return Err(LatchError::GenericError(format!("Truncated operand for {} at offset={offset}", desc.name)));
            }
            let val = u16::from_be_bytes([self.code[self.ip], self.code[self.ip + 1]]);
            self.ip += 2;
            Some(val)
        } else {
            None
        };

        Ok(DecodedInstruction {
            opcode: op,
            operand,
            offset,
        })
    }
}
