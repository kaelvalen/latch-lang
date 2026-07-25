use crate::env::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    OpConstant     = 1,
    OpAdd          = 2,
    OpSub          = 3,
    OpMul          = 4,
    OpDiv          = 5,
    OpMod          = 6,
    OpNeg          = 7,
    OpNot          = 8,
    OpEqual        = 9,
    OpLess         = 10,
    OpGreater      = 11,
    OpGetLocal     = 12,
    OpSetLocal     = 13,
    OpGetGlobal    = 14,
    OpDefineGlobal = 15,
    OpSetGlobal    = 16,
    OpJump         = 17,
    OpJumpIfFalse  = 18,
    OpLoop         = 19,
    OpCall         = 20,
    OpReturn       = 21,
    OpPop          = 22,
    OpList         = 23,
    OpMap          = 24,
    OpIndex        = 25,
    OpIndexAssign  = 26,
    OpPrint        = 27,
    OpIn           = 28,
}

impl OpCode {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            1  => Some(OpCode::OpConstant),
            2  => Some(OpCode::OpAdd),
            3  => Some(OpCode::OpSub),
            4  => Some(OpCode::OpMul),
            5  => Some(OpCode::OpDiv),
            6  => Some(OpCode::OpMod),
            7  => Some(OpCode::OpNeg),
            8  => Some(OpCode::OpNot),
            9  => Some(OpCode::OpEqual),
            10 => Some(OpCode::OpLess),
            11 => Some(OpCode::OpGreater),
            12 => Some(OpCode::OpGetLocal),
            13 => Some(OpCode::OpSetLocal),
            14 => Some(OpCode::OpGetGlobal),
            15 => Some(OpCode::OpDefineGlobal),
            16 => Some(OpCode::OpSetGlobal),
            17 => Some(OpCode::OpJump),
            18 => Some(OpCode::OpJumpIfFalse),
            19 => Some(OpCode::OpLoop),
            20 => Some(OpCode::OpCall),
            21 => Some(OpCode::OpReturn),
            22 => Some(OpCode::OpPop),
            23 => Some(OpCode::OpList),
            24 => Some(OpCode::OpMap),
            25 => Some(OpCode::OpIndex),
            26 => Some(OpCode::OpIndexAssign),
            27 => Some(OpCode::OpPrint),
            28 => Some(OpCode::OpIn),
            _  => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_u8(&mut self, byte: u8, line: u32) -> usize {
        self.code.push(byte);
        self.lines.push(line);
        self.code.len() - 1
    }

    pub fn write_opcode(&mut self, op: OpCode, line: u32) -> usize {
        self.write_u8(op as u8, line)
    }

    pub fn write_u16(&mut self, val: u16, line: u32) {
        let bytes = val.to_be_bytes();
        self.write_u8(bytes[0], line);
        self.write_u8(bytes[1], line);
    }

    pub fn write_u32(&mut self, val: u32, line: u32) {
        let bytes = val.to_be_bytes();
        self.write_u8(bytes[0], line);
        self.write_u8(bytes[1], line);
        self.write_u8(bytes[2], line);
        self.write_u8(bytes[3], line);
    }

    pub fn add_constant(&mut self, val: Value) -> usize {
        for (i, c) in self.constants.iter().enumerate() {
            if c == &val {
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
            OpCode::OpConstant => {
                let idx = self.read_u16_at(offset + 1);
                let val = &self.constants[idx as usize];
                println!("{op_name:<16} {idx:4} '{val}'");
                offset + 3
            }
            OpCode::OpGetLocal | OpCode::OpSetLocal | OpCode::OpDefineGlobal | OpCode::OpGetGlobal | OpCode::OpSetGlobal | OpCode::OpCall | OpCode::OpList | OpCode::OpMap => {
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
