use crate::env::{ObjFunction, ObjFunctionBuilder, ObjRef};
use crate::error::{LatchError, Result};
use super::chunk::Chunk;

pub const LBC_MAGIC: &[u8; 6] = b"LATCHB";
pub const LBC_VERSION: u16 = 1;
pub const LBC_ISA_VERSION: u16 = 1;
pub const LBC_FLAGS: u16 = 0;

pub struct LbcSerializer;

impl LbcSerializer {
    /// Serialize an ObjFunction into binary .lbc byte stream.
    pub fn serialize(func: &ObjFunction) -> Vec<u8> {
        let mut buf = Vec::new();
        // 1. Magic bytes
        buf.extend_from_slice(LBC_MAGIC);
        // 2. Version
        buf.extend_from_slice(&LBC_VERSION.to_be_bytes());
        // 3. ISA version & flags
        buf.extend_from_slice(&LBC_ISA_VERSION.to_be_bytes());
        buf.extend_from_slice(&LBC_FLAGS.to_be_bytes());

        // 4. Arity & Name
        buf.extend_from_slice(&(func.arity as u16).to_be_bytes());
        let name_bytes = func.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
        buf.extend_from_slice(name_bytes);

        // 4. Constant Pool
        let const_count = func.chunk.constants().len() as u16;
        buf.extend_from_slice(&const_count.to_be_bytes());
        for c in func.chunk.constants() {
            Self::serialize_value(c, &mut buf);
        }

        // 5. Bytecode Stream
        let code_len = func.chunk.code().len() as u32;
        buf.extend_from_slice(&code_len.to_be_bytes());
        buf.extend_from_slice(func.chunk.code());

        // 6. Line table
        let lines_len = func.chunk.lines().len() as u32;
        buf.extend_from_slice(&lines_len.to_be_bytes());
        for line in func.chunk.lines() {
            buf.extend_from_slice(&line.to_be_bytes());
        }

        buf
    }

    /// Deserialize binary .lbc byte stream back into ObjRef<ObjFunction>.
    pub fn deserialize(bytes: &[u8]) -> Result<ObjRef<ObjFunction>> {
        if bytes.len() < 12 || &bytes[0..6] != LBC_MAGIC {
            return Err(LatchError::GenericError("Invalid .lbc magic binary header".into()));
        }

        let mut cursor = 6;
        let version = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;
        let isa_version = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;
        let flags = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;

        if version != LBC_VERSION || isa_version != LBC_ISA_VERSION {
            return Err(LatchError::GenericError(format!(
                "Unsupported .lbc version {version} (isa {isa_version})"
            )));
        }
        if flags != 0 {
            return Err(LatchError::GenericError(format!("Unsupported .lbc flags {flags}")));
        }

        let arity = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;

        let name_len = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        let name = String::from_utf8_lossy(&bytes[cursor..cursor + name_len]).to_string();
        cursor += name_len;

        // Constants
        let const_count = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        let mut constants = Vec::with_capacity(const_count);
        for _ in 0..const_count {
            let (val, n) = Self::deserialize_value(&bytes[cursor..])?;
            constants.push(val);
            cursor += n;
        }

        // Code
        let code_len = u32::from_be_bytes([
            bytes[cursor], bytes[cursor + 1], bytes[cursor + 2], bytes[cursor + 3],
        ]) as usize;
        cursor += 4;
        let code = bytes[cursor..cursor + code_len].to_vec();
        cursor += code_len;

        // Lines
        let lines_len = u32::from_be_bytes([
            bytes[cursor], bytes[cursor + 1], bytes[cursor + 2], bytes[cursor + 3],
        ]) as usize;
        cursor += 4;
        let mut lines = Vec::with_capacity(lines_len);
        for _ in 0..lines_len {
            let line = u32::from_be_bytes([
                bytes[cursor], bytes[cursor + 1], bytes[cursor + 2], bytes[cursor + 3],
            ]);
            lines.push(line);
            cursor += 4;
        }

        let chunk = Chunk::from_parts(code, constants, lines);
        let func = ObjFunctionBuilder::new(name, arity)
            .with_chunk(chunk)
            .build();

        Ok(ObjRef::new(func))
    }

    fn serialize_value(val: &super::chunk::Constant, buf: &mut Vec<u8>) {
        match val {
            super::chunk::Constant::Int(n) => {
                buf.push(1);
                buf.extend_from_slice(&n.to_be_bytes());
            }
            super::chunk::Constant::Float(f) => {
                buf.push(2);
                buf.extend_from_slice(&f.to_be_bytes());
            }
            super::chunk::Constant::Bool(b) => {
                buf.push(3);
                buf.push(if *b { 1 } else { 0 });
            }
            super::chunk::Constant::Str(s) => {
                buf.push(4);
                let bytes = s.as_bytes();
                buf.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
                buf.extend_from_slice(bytes);
            }
            _ => {
                buf.push(0); // Null
            }
        }
    }

    fn deserialize_value(bytes: &[u8]) -> Result<(super::chunk::Constant, usize)> {
        let tag = bytes[0];
        match tag {
            1 => {
                let n = i64::from_be_bytes(bytes[1..9].try_into().unwrap());
                Ok((super::chunk::Constant::Int(n), 9))
            }
            2 => {
                let f = f64::from_be_bytes(bytes[1..9].try_into().unwrap());
                Ok((super::chunk::Constant::Float(f), 9))
            }
            3 => {
                let b = bytes[1] != 0;
                Ok((super::chunk::Constant::Bool(b), 2))
            }
            4 => {
                let len = u16::from_be_bytes(bytes[1..3].try_into().unwrap()) as usize;
                let s = String::from_utf8_lossy(&bytes[3..3 + len]).to_string();
                Ok((super::chunk::Constant::Str(s), 3 + len))
            }
            _ => Ok((super::chunk::Constant::Null, 1)),
        }
    }
}
