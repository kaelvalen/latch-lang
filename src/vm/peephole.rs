use super::chunk::{ChunkBuilder, Constant, OpCode};

/// Post-compilation peephole optimizer. Operates on a `ChunkBuilder`
/// before it is sealed into an immutable `Chunk`. All passes preserve the
/// ISA stack contract and are only applied when the transformation is
/// provably safe.
pub struct BytecodePeephole;

impl BytecodePeephole {
    /// Run all peephole passes on `builder` in place.
    pub fn optimize(builder: &mut ChunkBuilder) {
        Self::fold_constant_int_ops(builder);
        Self::thread_jumps(builder);
    }

    /// Fold sequences of the form:
    ///   OP_CONSTANT <a>
    ///   OP_CONSTANT <b>
    ///   OP_ADD / OP_SUB / OP_MUL
    /// into a single OP_CONSTANT <result>.
    fn fold_constant_int_ops(builder: &mut ChunkBuilder) {
        let (code, constants) = builder.parts_mut();

        let mut i = 0;
        while i < code.len() {
            if code[i] != OpCode::OpConstant as u8 {
                let width = OpCode::from_u8(code[i])
                    .map(|o| o.descriptor().operand_width)
                    .unwrap_or(0);
                i += 1 + width;
                continue;
            }

            let a_idx = u16::from_be_bytes([code[i + 1], code[i + 2]]) as usize;
            let next = i + 3;
            if next >= code.len() || code[next] != OpCode::OpConstant as u8 {
                i += 3;
                continue;
            }
            let b_idx = u16::from_be_bytes([code[next + 1], code[next + 2]]) as usize;
            let op_offset = next + 3;
            if op_offset >= code.len() {
                i += 3;
                continue;
            }
            let op_byte = code[op_offset];

            if a_idx >= constants.len() || b_idx >= constants.len() {
                i += 3;
                continue;
            }

            let (Constant::Int(a), Constant::Int(b)) = (&constants[a_idx], &constants[b_idx])
            else {
                i += 3;
                continue;
            };

            let result = match OpCode::from_u8(op_byte) {
                Some(OpCode::OpAdd) => a + b,
                Some(OpCode::OpSub) => a - b,
                Some(OpCode::OpMul) => a * b,
                _ => {
                    i += 3;
                    continue;
                }
            };

            // Replace the window with OP_CONSTANT <new_const>.
            let new_idx = constants.len();
            constants.push(Constant::Int(result));
            code[i] = OpCode::OpConstant as u8;
            code[i + 1] = ((new_idx >> 8) & 0xff) as u8;
            code[i + 2] = (new_idx & 0xff) as u8;
            code.drain((i + 3)..(op_offset + 1));
            i += 3;
        }
    }

    /// Replace unreachable or chained jumps where safe.
    /// Currently handles JUMP -> JUMP (target is also an unconditional jump).
    fn thread_jumps(builder: &mut ChunkBuilder) {
        let code = builder.code_mut();
        let code_snapshot = code.clone();
        let mut replacements: Vec<(usize, u16)> = Vec::new();

        let mut offset = 0;
        while offset < code_snapshot.len() {
            let op = match OpCode::from_u8(code_snapshot[offset]) {
                Some(op) => op,
                None => {
                    offset += 1;
                    continue;
                }
            };

            if op == OpCode::OpJump {
                let target =
                    u16::from_be_bytes([code_snapshot[offset + 1], code_snapshot[offset + 2]])
                        as usize;
                if target < code_snapshot.len() && code_snapshot[target] == OpCode::OpJump as u8 {
                    let final_target =
                        u16::from_be_bytes([code_snapshot[target + 1], code_snapshot[target + 2]]);
                    replacements.push((offset + 1, final_target));
                }
            }

            offset += 1 + op.descriptor().operand_width;
        }

        for (offset, target) in replacements {
            code[offset] = ((target >> 8) & 0xff) as u8;
            code[offset + 1] = (target & 0xff) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::chunk::Constant;
    use super::*;

    #[test]
    fn peephole_folds_int_add() {
        let mut builder = ChunkBuilder::new();
        let a = builder.add_constant(Constant::Int(5));
        let b = builder.add_constant(Constant::Int(3));
        builder.write_opcode(OpCode::OpConstant, 1);
        builder.write_u16(a as u16, 1);
        builder.write_opcode(OpCode::OpConstant, 1);
        builder.write_u16(b as u16, 1);
        builder.write_opcode(OpCode::OpAdd, 1);

        BytecodePeephole::optimize(&mut builder);
        let chunk = builder.build();

        assert_eq!(chunk.code().len(), 3);
        assert_eq!(chunk.code()[0], OpCode::OpConstant as u8);
        let folded_idx = u16::from_be_bytes([chunk.code()[1], chunk.code()[2]]) as usize;
        assert_eq!(chunk.constants()[folded_idx], Constant::Int(8));
    }

    #[test]
    fn peephole_threads_unconditional_jumps() {
        let mut builder = ChunkBuilder::new();
        // Layout:
        // 0: OP_JUMP 3
        // 3: OP_RETURN
        // 4: OP_JUMP 7
        // 7: OP_RETURN
        let first_jump_offset = builder.write_opcode(OpCode::OpJump, 1);
        builder.write_u16(0, 1);
        builder.write_opcode(OpCode::OpReturn, 1);
        let second_jump_offset = builder.write_opcode(OpCode::OpJump, 1);
        builder.write_u16(0, 1);
        builder.write_opcode(OpCode::OpReturn, 1);

        builder.patch_u16(first_jump_offset + 1, 4);
        builder.patch_u16(second_jump_offset + 1, 7);

        BytecodePeephole::optimize(&mut builder);
        let chunk = builder.build();

        // First jump operand should now point to 7, not 4.
        let target = u16::from_be_bytes([
            chunk.code()[first_jump_offset + 1],
            chunk.code()[first_jump_offset + 2],
        ]);
        assert_eq!(target, 7);
    }
}
