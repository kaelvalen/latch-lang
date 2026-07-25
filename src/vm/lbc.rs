use std::sync::Arc;

use crate::env::{ObjFunction, ObjHeader, ObjKind, Value};
use crate::error::{LatchError, Result};
use super::chunk::Chunk;

pub const LBC_MAGIC: &[u8; 6] = b"LATCHB";
pub const LBC_VERSION: u16 = 1;

pub struct LbcSerializer;

impl LbcSerializer {
    /// Serialize an ObjFunction into binary .lbc byte stream.
    pub fn serialize(func: &ObjFunction) -> Vec<u8> {
        let mut buf = Vec::new();
        // 1. Magic bytes
        buf.extend_from_slice(LBC_MAGIC);
        // 2. Version
        buf.extend_from_slice(&LBC_VERSION.to_be_bytes());

        // 3. Arity & Name
        buf.extend_from_slice(&(func.arity as u16).to_be_bytes());
        let name_bytes = func.name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
        buf.extend_from_slice(name_bytes);

        // 4. Constant Pool
        let const_count = func.chunk.constants.len() as u16;
        buf.extend_from_slice(&const_count.to_be_bytes());
        for c in &func.chunk.constants {
            Self::serialize_value(c, &mut buf);
        }

        // 5. Bytecode Stream
        let code_len = func.chunk.code.len() as u32;
        buf.extend_from_slice(&code_len.to_be_bytes());
        buf.extend_from_slice(&func.chunk.code);

        // 6. Line table
        let lines_len = func.chunk.lines.len() as u32;
        buf.extend_from_slice(&lines_len.to_be_bytes());
        for line in &func.chunk.lines {
            buf.extend_from_slice(&line.to_be_bytes());
        }

        buf
    }

    /// Deserialize binary .lbc byte stream back into Arc<ObjFunction>.
    pub fn deserialize(bytes: &[u8]) -> Result<Arc<ObjFunction>> {
        if bytes.len() < 8 || &bytes[0..6] != LBC_MAGIC {
            return Err(LatchError::GenericError("Invalid .lbc magic binary header".into()));
        }

        let mut cursor = 6;
        let version = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]);
        cursor += 2;
        if version != LBC_VERSION {
            return Err(LatchError::GenericError(format!("Unsupported .lbc version {version}")));
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

        let chunk = Chunk { code, constants, lines };
        let func = ObjFunction {
            header: ObjHeader::new(ObjKind::Function),
            arity,
            chunk,
            name,
            upvalue_count: 0,
            max_stack: 256,
            local_count: 0,
            module_id: 0,
            debug_id: 0,
            flags: 0,
        };

        Ok(Arc::new(func))
    }

    fn serialize_value(val: &Value, buf: &mut Vec<u8>) {
        match val {
            Value::Int(n) => {
                buf.push(1);
                buf.extend_from_slice(&n.to_be_bytes());
            }
            Value::Float(f) => {
                buf.push(2);
                buf.extend_from_slice(&f.to_be_bytes());
            }
            Value::Bool(b) => {
                buf.push(3);
                buf.push(if *b { 1 } else { 0 });
            }
            Value::Str(s) => {
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

    fn deserialize_value(bytes: &[u8]) -> Result<(Value, usize)> {
        let tag = bytes[0];
        match tag {
            1 => {
                let n = i64::from_be_bytes(bytes[1..9].try_into().unwrap());
                Ok((Value::Int(n), 9))
            }
            2 => {
                let f = f64::from_be_bytes(bytes[1..9].try_into().unwrap());
                Ok((Value::Float(f), 9))
            }
            3 => {
                let b = bytes[1] != 0;
                Ok((Value::Bool(b), 2))
            }
            4 => {
                let len = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
                let s = String::from_utf8_lossy(&bytes[3..3 + len]).to_string();
                Ok((Value::Str(s), 3 + len))
            }
            _ => Ok((Value::Null, 1)),
        }
    }
}
