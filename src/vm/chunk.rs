use crate::env::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    OpConstant     = 1,
    OpNull         = 2,
    OpTrue         = 3,
    OpFalse        = 4,
    OpAdd          = 5,
    OpSub          = 6,
    OpMul          = 7,
    OpDiv          = 8,
    OpMod          = 9,
    OpNeg          = 10,
    OpNot          = 11,
    OpEq           = 12,
    OpNotEq        = 13,
    OpLt           = 14,
    OpGt           = 15,
    OpLtEq         = 16,
    OpGtEq         = 17,
    OpIn           = 18,
    OpDefineGlobal = 19,
    OpGetGlobal    = 20,
    OpSetGlobal    = 21,
    OpGetLocal     = 22,
    OpSetLocal     = 23,
    OpJump         = 24,
    OpJumpIfFalse  = 25,
    OpLoop         = 26,
    OpCall         = 27,
    OpReturn       = 28,
    OpPop          = 29,
    OpList         = 30,
    OpMap          = 31,
    OpIndex        = 32,
    OpIndexAssign  = 33,
    OpFieldAccess  = 34,
    OpFieldAssign  = 35,
    OpPrint        = 36,
}

impl OpCode {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            1  => Some(OpCode::OpConstant),
            2  => Some(OpCode::OpNull),
            3  => Some(OpCode::OpTrue),
            4  => Some(OpCode::OpFalse),
            5  => Some(OpCode::OpAdd),
            6  => Some(OpCode::OpSub),
            7  => Some(OpCode::OpMul),
            8  => Some(OpCode::OpDiv),
            9  => Some(OpCode::OpMod),
            10 => Some(OpCode::OpNeg),
            11 => Some(OpCode::OpNot),
            12 => Some(OpCode::OpEq),
            13 => Some(OpCode::OpNotEq),
            14 => Some(OpCode::OpLt),
            15 => Some(OpCode::OpGt),
            16 => Some(OpCode::OpLtEq),
            17 => Some(OpCode::OpGtEq),
            18 => Some(OpCode::OpIn),
            19 => Some(OpCode::OpDefineGlobal),
            20 => Some(OpCode::OpGetGlobal),
            21 => Some(OpCode::OpSetGlobal),
            22 => Some(OpCode::OpGetLocal),
            23 => Some(OpCode::OpSetLocal),
            24 => Some(OpCode::OpJump),
            25 => Some(OpCode::OpJumpIfFalse),
            26 => Some(OpCode::OpLoop),
            27 => Some(OpCode::OpCall),
            28 => Some(OpCode::OpReturn),
            29 => Some(OpCode::OpPop),
            30 => Some(OpCode::OpList),
            31 => Some(OpCode::OpMap),
            32 => Some(OpCode::OpIndex),
            33 => Some(OpCode::OpIndexAssign),
            34 => Some(OpCode::OpFieldAccess),
            35 => Some(OpCode::OpFieldAssign),
            36 => Some(OpCode::OpPrint),
            _  => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_u8(&mut self, byte: u8, line: usize) -> usize {
        self.code.push(byte);
        self.lines.push(line);
        self.code.len() - 1
    }

    pub fn write_opcode(&mut self, op: OpCode, line: usize) -> usize {
        self.write_u8(op as u8, line)
    }

    pub fn write_u16(&mut self, val: u16, line: usize) {
        let bytes = val.to_be_bytes();
        self.write_u8(bytes[0], line);
        self.write_u8(bytes[1], line);
    }

    pub fn write_u32(&mut self, val: u32, line: usize) {
        let bytes = val.to_be_bytes();
        self.write_u8(bytes[0], line);
        self.write_u8(bytes[1], line);
        self.write_u8(bytes[2], line);
        self.write_u8(bytes[3], line);
    }

    pub fn add_constant(&mut self, val: Value) -> usize {
        for (i, c) in self.constants.iter().enumerate() {
            if format!("{c}") == format!("{val}") {
                return i;
            }
        }
        self.constants.push(val);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:04} ");
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let byte = self.code[offset];
        let op = match OpCode::from_u8(byte) {
            Some(op) => op,
            None => {
                println!("Unknown opcode {byte}");
                return offset + 1;
            }
        };

        let op_name = format!("{op:?}");
        match op {
            OpCode::OpConstant | OpCode::OpDefineGlobal | OpCode::OpGetGlobal | OpCode::OpSetGlobal => {
                let idx = self.read_u16_at(offset + 1);
                let val = &self.constants[idx as usize];
                println!("{op_name:<16} {idx:4} '{val}'");
                offset + 3
            }
            OpCode::OpGetLocal | OpCode::OpSetLocal | OpCode::OpCall | OpCode::OpList | OpCode::OpMap => {
                let idx = self.read_u16_at(offset + 1);
                println!("{op_name:<16} {idx:4}");
                offset + 3
            }
            OpCode::OpJump | OpCode::OpJumpIfFalse | OpCode::OpLoop => {
                let jump = self.read_u16_at(offset + 1);
                println!("{op_name:<16} {offset:4} -> {jump:4}");
                offset + 3
            }
            _ => {
                println!("{op_name}");
                offset + 1
            }
        }
    }

    fn read_u16_at(&self, offset: usize) -> u16 {
        u16::from_be_bytes([self.code[offset], self.code[offset + 1]])
    }
}
