use crate::env::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    OpConstant(usize),
    OpNull,
    OpTrue,
    OpFalse,
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpMod,
    OpNeg,
    OpNot,
    OpEq,
    OpNotEq,
    OpLt,
    OpGt,
    OpLtEq,
    OpGtEq,
    OpIn,
    OpDefineGlobal(usize),
    OpGetGlobal(usize),
    OpSetGlobal(usize),
    OpGetLocal(usize),
    OpSetLocal(usize),
    OpJump(usize),
    OpJumpIfFalse(usize),
    OpLoop(usize),
    OpCall(usize), // num args
    OpReturn,
    OpPop,
    OpList(usize),
    OpMap(usize),
    OpIndex,
    OpIndexAssign,
    OpFieldAccess(usize),
    OpFieldAssign(usize),
    OpPrint,
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, op: OpCode, line: usize) -> usize {
        self.code.push(op);
        self.lines.push(line);
        self.code.len() - 1
    }

    pub fn add_constant(&mut self, val: Value) -> usize {
        // Check if constant already exists
        for (i, c) in self.constants.iter().enumerate() {
            if format!("{c}") == format!("{val}") {
                return i;
            }
        }
        self.constants.push(val);
        self.constants.len() - 1
    }
}
