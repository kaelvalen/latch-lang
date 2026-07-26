use std::collections::HashMap;

use crate::env::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    OpConstant = 1,
    OpAdd = 2,
    OpSub = 3,
    OpMul = 4,
    OpDiv = 5,
    OpMod = 6,
    OpNeg = 7,
    OpNot = 8,
    OpEqual = 9,
    OpLess = 10,
    OpGreater = 11,
    OpGetLocal = 12,
    OpSetLocal = 13,
    OpGetGlobal = 14,
    OpDefineGlobal = 15,
    OpSetGlobal = 16,
    OpJump = 17,
    OpJumpIfFalse = 18,
    OpLoop = 19,
    OpCall = 20,
    OpReturn = 21,
    OpPop = 22,
    OpList = 23,
    OpMap = 24,
    OpIndex = 25,
    OpIndexAssign = 26,
    OpPrint = 27,
    OpIn = 28,
    OpGetUpvalue = 29,
    OpSetUpvalue = 30,
    OpClosure = 31,
    OpDup = 32,
}

impl OpCode {
    #[inline(always)]
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            1 => Some(OpCode::OpConstant),
            2 => Some(OpCode::OpAdd),
            3 => Some(OpCode::OpSub),
            4 => Some(OpCode::OpMul),
            5 => Some(OpCode::OpDiv),
            6 => Some(OpCode::OpMod),
            7 => Some(OpCode::OpNeg),
            8 => Some(OpCode::OpNot),
            9 => Some(OpCode::OpEqual),
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
            29 => Some(OpCode::OpGetUpvalue),
            30 => Some(OpCode::OpSetUpvalue),
            31 => Some(OpCode::OpClosure),
            32 => Some(OpCode::OpDup),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn descriptor(self) -> &'static InstructionDescriptor {
        &OPCODE_TABLE[self as usize]
    }
}

/// Compile-time opcode metadata table. Indexed by `OpCode as usize`.
/// Index 0 is a placeholder for the reserved `0x00` opcode.
pub const OPCODE_TABLE: [InstructionDescriptor; 33] = [
    InstructionDescriptor {
        opcode: OpCode::OpConstant,
        name: "<reserved>",
        operand_count: 0,
        operand_width: 0,
        stack_in: 0,
        stack_out: 0,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    }, // 0x00 placeholder
    InstructionDescriptor {
        opcode: OpCode::OpConstant,
        name: "OP_CONSTANT",
        operand_count: 1,
        operand_width: 2,
        stack_in: 0,
        stack_out: 1,
        is_jump: false,
        may_allocate: true,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpAdd,
        name: "OP_ADD",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: true,
    },
    InstructionDescriptor {
        opcode: OpCode::OpSub,
        name: "OP_SUB",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpMul,
        name: "OP_MUL",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpDiv,
        name: "OP_DIV",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpMod,
        name: "OP_MOD",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpNeg,
        name: "OP_NEG",
        operand_count: 0,
        operand_width: 0,
        stack_in: 1,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpNot,
        name: "OP_NOT",
        operand_count: 0,
        operand_width: 0,
        stack_in: 1,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpEqual,
        name: "OP_EQUAL",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpLess,
        name: "OP_LESS",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpGreater,
        name: "OP_GREATER",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpGetLocal,
        name: "OP_GET_LOCAL",
        operand_count: 1,
        operand_width: 2,
        stack_in: 0,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpSetLocal,
        name: "OP_SET_LOCAL",
        operand_count: 1,
        operand_width: 2,
        stack_in: 1,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpGetGlobal,
        name: "OP_GET_GLOBAL",
        operand_count: 1,
        operand_width: 2,
        stack_in: 0,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpDefineGlobal,
        name: "OP_DEF_GLOBAL",
        operand_count: 1,
        operand_width: 2,
        stack_in: 1,
        stack_out: 0,
        is_jump: false,
        may_allocate: true,
        gc_safe: true,
    },
    InstructionDescriptor {
        opcode: OpCode::OpSetGlobal,
        name: "OP_SET_GLOBAL",
        operand_count: 1,
        operand_width: 2,
        stack_in: 1,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpJump,
        name: "OP_JUMP",
        operand_count: 1,
        operand_width: 2,
        stack_in: 0,
        stack_out: 0,
        is_jump: true,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpJumpIfFalse,
        name: "OP_JUMP_FALSE",
        operand_count: 1,
        operand_width: 2,
        stack_in: 1,
        stack_out: 0,
        is_jump: true,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpLoop,
        name: "OP_LOOP",
        operand_count: 1,
        operand_width: 2,
        stack_in: 0,
        stack_out: 0,
        is_jump: true,
        may_allocate: false,
        gc_safe: true,
    },
    InstructionDescriptor {
        opcode: OpCode::OpCall,
        name: "OP_CALL",
        operand_count: 1,
        operand_width: 2,
        stack_in: 1,
        stack_out: 1,
        is_jump: false,
        may_allocate: true,
        gc_safe: true,
    },
    InstructionDescriptor {
        opcode: OpCode::OpReturn,
        name: "OP_RETURN",
        operand_count: 0,
        operand_width: 0,
        stack_in: 1,
        stack_out: 0,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpPop,
        name: "OP_POP",
        operand_count: 0,
        operand_width: 0,
        stack_in: 1,
        stack_out: 0,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpList,
        name: "OP_LIST",
        operand_count: 1,
        operand_width: 2,
        stack_in: 1,
        stack_out: 1,
        is_jump: false,
        may_allocate: true,
        gc_safe: true,
    },
    InstructionDescriptor {
        opcode: OpCode::OpMap,
        name: "OP_MAP",
        operand_count: 1,
        operand_width: 2,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: true,
        gc_safe: true,
    },
    InstructionDescriptor {
        opcode: OpCode::OpIndex,
        name: "OP_INDEX",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpIndexAssign,
        name: "OP_INDEX_ASSIGN",
        operand_count: 0,
        operand_width: 0,
        stack_in: 3,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpPrint,
        name: "OP_PRINT",
        operand_count: 0,
        operand_width: 0,
        stack_in: 1,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: true,
    },
    InstructionDescriptor {
        opcode: OpCode::OpIn,
        name: "OP_IN",
        operand_count: 0,
        operand_width: 0,
        stack_in: 2,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpGetUpvalue,
        name: "OP_GET_UPVAL",
        operand_count: 1,
        operand_width: 2,
        stack_in: 0,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpSetUpvalue,
        name: "OP_SET_UPVAL",
        operand_count: 1,
        operand_width: 2,
        stack_in: 1,
        stack_out: 1,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
    InstructionDescriptor {
        opcode: OpCode::OpClosure,
        name: "OP_CLOSURE",
        operand_count: 1,
        operand_width: 2,
        stack_in: 0,
        stack_out: 1,
        is_jump: false,
        may_allocate: true,
        gc_safe: true,
    },
    InstructionDescriptor {
        opcode: OpCode::OpDup,
        name: "OP_DUP",
        operand_count: 0,
        operand_width: 0,
        stack_in: 1,
        stack_out: 2,
        is_jump: false,
        may_allocate: false,
        gc_safe: false,
    },
];

/// Centralized Instruction Descriptor metadata table for Disassembler, Verifier, Debugger, & Optimizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionDescriptor {
    pub opcode: OpCode,
    pub name: &'static str,
    pub operand_count: usize,
    pub operand_width: usize,
    pub stack_in: usize,
    pub stack_out: usize,
    pub is_jump: bool,
    pub may_allocate: bool,
    pub gc_safe: bool,
}

/// Independent Compile-Time Constant Representation (Zero Runtime ABI dependencies)
#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Symbol(u32),
    Function(std::sync::Arc<crate::env::ObjFunction>),
    Null,
}

impl PartialEq for Constant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Constant::Int(a), Constant::Int(b)) => a == b,
            (Constant::Float(a), Constant::Float(b)) => {
                // Canonicalize NaN: all NaN bit patterns compare equal.
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (Constant::Bool(a), Constant::Bool(b)) => a == b,
            (Constant::Str(a), Constant::Str(b)) => a == b,
            (Constant::Symbol(a), Constant::Symbol(b)) => a == b,
            (Constant::Function(a), Constant::Function(b)) => std::sync::Arc::ptr_eq(a, b),
            (Constant::Null, Constant::Null) => true,
            _ => false,
        }
    }
}

impl Constant {
    pub fn to_value(&self) -> Value {
        match self {
            Constant::Int(n) => Value::Int(*n),
            Constant::Float(f) => Value::Float(*f),
            Constant::Bool(b) => Value::Bool(*b),
            Constant::Str(s) => Value::Str(s.clone()),
            Constant::Symbol(id) => Value::Int(*id as i64),
            Constant::Function(func) => Value::Function(func.clone()),
            Constant::Null => Value::Null,
        }
    }
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Int(n) => write!(f, "{n}"),
            Constant::Float(fl) => write!(f, "{fl}"),
            Constant::Bool(b) => write!(f, "{b}"),
            Constant::Str(s) => write!(f, "{s}"),
            Constant::Symbol(id) => write!(f, "symbol#{id}"),
            Constant::Function(func) => write!(f, "<fn {}>", func.name),
            Constant::Null => write!(f, "null"),
        }
    }
}

/// Immutable compiled bytecode container. Created by `ChunkBuilder::build`.
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Constant>,
    lines: Vec<u32>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub(crate) fn from_parts(code: Vec<u8>, constants: Vec<Constant>, lines: Vec<u32>) -> Self {
        Self {
            code,
            constants,
            lines,
        }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
    pub fn constants(&self) -> &[Constant] {
        &self.constants
    }
    pub fn lines(&self) -> &[u32] {
        &self.lines
    }

    pub fn disassemble(&self, name: &str) {
        let mut buf = String::new();
        self.disassemble_to(name, &mut buf);
        print!("{buf}");
    }

    pub fn disassemble_to<W: std::fmt::Write>(&self, name: &str, out: &mut W) {
        writeln!(out, "== {name} ==").unwrap();
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction_to(out, offset);
        }
    }

    fn disassemble_instruction_to<W: std::fmt::Write>(&self, out: &mut W, offset: usize) -> usize {
        write!(out, "{offset:04} ").unwrap();
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(out, "   | ").unwrap();
        } else {
            write!(out, "{:4} ", self.lines[offset]).unwrap();
        }

        let byte = self.code[offset];
        let op = match OpCode::from_u8(byte) {
            Some(op) => op,
            None => {
                writeln!(out, "Unknown opcode {byte}").unwrap();
                return offset + 1;
            }
        };

        let op_name = op.descriptor().name;
        match op {
            OpCode::OpConstant => {
                let idx = self.read_u16_at(offset + 1);
                let val = &self.constants[idx as usize];
                writeln!(out, "{op_name:<16} {idx:4} '{val}'").unwrap();
                offset + 3
            }
            OpCode::OpGetLocal
            | OpCode::OpSetLocal
            | OpCode::OpDefineGlobal
            | OpCode::OpGetGlobal
            | OpCode::OpSetGlobal
            | OpCode::OpCall
            | OpCode::OpList
            | OpCode::OpMap => {
                let idx = self.read_u16_at(offset + 1);
                writeln!(out, "{op_name:<16} {idx:4}").unwrap();
                offset + 3
            }
            OpCode::OpJump | OpCode::OpJumpIfFalse | OpCode::OpLoop => {
                let jump = self.read_u16_at(offset + 1);
                writeln!(out, "{op_name:<16} {offset:4} -> {jump:4}").unwrap();
                offset + 3
            }
            _ => {
                writeln!(out, "{op_name}").unwrap();
                offset + 1
            }
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

        let op_name = op.descriptor().name;
        match op {
            OpCode::OpConstant => {
                let idx = self.read_u16_at(offset + 1);
                let val = &self.constants[idx as usize];
                println!("{op_name:<16} {idx:4} '{val}'");
                offset + 3
            }
            OpCode::OpGetLocal
            | OpCode::OpSetLocal
            | OpCode::OpDefineGlobal
            | OpCode::OpGetGlobal
            | OpCode::OpSetGlobal
            | OpCode::OpCall
            | OpCode::OpList
            | OpCode::OpMap => {
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

const SMALL_INT_MIN: i64 = -128;
const SMALL_INT_MAX: i64 = 127;

/// Mutable bytecode emitter buffer. Use `build()` to produce an immutable `Chunk`.
#[derive(Debug, Clone)]
pub struct ChunkBuilder {
    code: Vec<u8>,
    constants: Vec<Constant>,
    lines: Vec<u32>,
    string_table: HashMap<String, usize>,
}

impl Default for ChunkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
            string_table: HashMap::new(),
        };
        // Empty string singleton at index 0.
        builder.constants.push(Constant::Str(String::new()));
        builder.string_table.insert(String::new(), 0);
        // Small integer cache.
        for n in SMALL_INT_MIN..=SMALL_INT_MAX {
            builder.constants.push(Constant::Int(n));
        }
        builder
    }

    pub fn add_constant(&mut self, constant: Constant) -> usize {
        match constant {
            Constant::Int(n) if (SMALL_INT_MIN..=SMALL_INT_MAX).contains(&n) => {
                ((n - SMALL_INT_MIN) + 1) as usize
            }
            Constant::Str(ref s) if s.is_empty() => 0,
            Constant::Str(s) => {
                if let Some(&idx) = self.string_table.get(&s) {
                    return idx;
                }
                let idx = self.constants.len();
                self.string_table.insert(s.clone(), idx);
                self.constants.push(Constant::Str(s));
                idx
            }
            _ => {
                for (i, existing) in self.constants.iter().enumerate() {
                    if existing == &constant {
                        return i;
                    }
                }
                let idx = self.constants.len();
                self.constants.push(constant);
                idx
            }
        }
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

    #[allow(dead_code)]
    pub fn write_u32(&mut self, val: u32, line: u32) {
        let bytes = val.to_be_bytes();
        self.write_u8(bytes[0], line);
        self.write_u8(bytes[1], line);
        self.write_u8(bytes[2], line);
        self.write_u8(bytes[3], line);
    }

    pub fn code_len(&self) -> usize {
        self.code.len()
    }

    pub fn patch_u16(&mut self, offset: usize, val: u16) {
        let bytes = val.to_be_bytes();
        self.code[offset] = bytes[0];
        self.code[offset + 1] = bytes[1];
    }

    pub(crate) fn code_mut(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }
    pub(crate) fn parts_mut(&mut self) -> (&mut Vec<u8>, &mut Vec<Constant>) {
        (&mut self.code, &mut self.constants)
    }

    pub fn build(self) -> Chunk {
        Chunk {
            code: self.code,
            constants: self.constants,
            lines: self.lines,
        }
    }
}
