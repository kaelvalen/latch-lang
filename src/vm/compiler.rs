use std::collections::HashMap;

use crate::ast::*;
use crate::env::Value;
use crate::error::Result;
use super::chunk::{Chunk, OpCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalFlags {
    pub is_captured: bool,
    pub is_mutable: bool,
    pub is_initialized: bool,
}

impl LocalFlags {
    pub fn new(is_mutable: bool) -> Self {
        LocalFlags {
            is_captured: false,
            is_mutable,
            is_initialized: true,
        }
    }
}

pub struct Local {
    pub name: String,
    pub slot: usize,
    pub depth: usize,
    pub flags: LocalFlags,
}

/// Pure Code Generator (Emitter) — transforms a semantically validated AST into bytecode.
pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    globals_map: HashMap<String, usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            globals_map: HashMap::new(),
        }
    }

    pub fn compile(mut self, stmts: &[Stmt]) -> Result<Chunk> {
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        self.emit_constant(Value::Null, 0);
        self.emit_opcode(OpCode::OpReturn, 0);
        Ok(self.chunk)
    }

    // ── Low-Level Emitter Methods ─────────────────────────────

    #[inline]
    fn emit_byte(&mut self, byte: u8, line: u32) -> usize {
        self.chunk.write_u8(byte, line)
    }

    #[inline]
    fn emit_opcode(&mut self, op: OpCode, line: u32) -> usize {
        self.chunk.write_opcode(op, line)
    }

    #[inline]
    fn emit_u16(&mut self, val: u16, line: u32) {
        self.chunk.write_u16(val, line);
    }

    fn emit_constant(&mut self, val: Value, line: u32) {
        let idx = self.chunk.add_constant(val);
        self.emit_opcode(OpCode::OpConstant, line);
        self.emit_u16(idx as u16, line);
    }

    fn emit_jump(&mut self, instruction: OpCode, line: u32) -> usize {
        self.emit_opcode(instruction, line);
        self.emit_u16(0xffff, line);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len();
        let bytes = (jump as u16).to_be_bytes();
        self.chunk.code[offset] = bytes[0];
        self.chunk.code[offset + 1] = bytes[1];
    }

    fn emit_loop(&mut self, loop_start: usize, line: u32) {
        self.emit_opcode(OpCode::OpLoop, line);
        self.emit_u16(loop_start as u16, line);
    }

    fn get_or_create_global(&mut self, name: &str) -> usize {
        if let Some(&id) = self.globals_map.get(name) {
            id
        } else {
            let id = self.globals_map.len();
            self.globals_map.insert(name.to_string(), id);
            id
        }
    }

    // ── Statement & Expression Generation ────────────────────

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                self.compile_expr(value)?;
                if self.scope_depth > 0 {
                    let slot = self.locals.len();
                    self.locals.push(Local {
                        name: name.clone(),
                        slot,
                        depth: self.scope_depth,
                        flags: LocalFlags::new(true),
                    });
                } else {
                    let global_id = self.get_or_create_global(name);
                    self.emit_opcode(OpCode::OpDefineGlobal, 0);
                    self.emit_u16(global_id as u16, 0);
                }
            }

            Stmt::Assign { name, value } => {
                self.compile_expr(value)?;
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_opcode(OpCode::OpSetLocal, 0);
                    self.emit_u16(slot as u16, 0);
                } else {
                    let global_id = self.get_or_create_global(name);
                    self.emit_opcode(OpCode::OpDefineGlobal, 0);
                    self.emit_u16(global_id as u16, 0);
                }
            }

            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.emit_opcode(OpCode::OpPop, 0);
            }

            Stmt::If { cond, then, else_ } => {
                self.compile_expr(cond)?;
                let jump_if_false_offset = self.emit_jump(OpCode::OpJumpIfFalse, 0);
                self.emit_opcode(OpCode::OpPop, 0);

                self.begin_scope();
                for s in then {
                    self.compile_stmt(s)?;
                }
                self.end_scope();

                let jump_offset = self.emit_jump(OpCode::OpJump, 0);
                self.patch_jump(jump_if_false_offset);

                self.emit_opcode(OpCode::OpPop, 0);
                if let Some(else_stmt) = else_ {
                    self.begin_scope();
                    self.compile_stmt(else_stmt)?;
                    self.end_scope();
                }

                self.patch_jump(jump_offset);
            }

            Stmt::While { cond, body } => {
                let loop_start = self.chunk.code.len();
                self.compile_expr(cond)?;
                let exit_jump = self.emit_jump(OpCode::OpJumpIfFalse, 0);
                self.emit_opcode(OpCode::OpPop, 0);

                self.begin_scope();
                for s in body {
                    self.compile_stmt(s)?;
                }
                self.end_scope();

                self.emit_loop(loop_start, 0);
                self.patch_jump(exit_jump);
                self.emit_opcode(OpCode::OpPop, 0);
            }

            Stmt::Return(expr) => {
                self.compile_expr(expr)?;
                self.emit_opcode(OpCode::OpReturn, 0);
            }

            Stmt::Const { name, value, .. } => {
                self.compile_expr(value)?;
                let global_id = self.get_or_create_global(name);
                self.emit_opcode(OpCode::OpDefineGlobal, 0);
                self.emit_u16(global_id as u16, 0);
            }

            Stmt::IndexAssign { target, index, value } => {
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                self.compile_expr(value)?;
                self.emit_opcode(OpCode::OpIndexAssign, 0);
            }

            _ => {}
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Int(n)   => self.emit_constant(Value::Int(*n), 0),
            Expr::Float(f) => self.emit_constant(Value::Float(*f), 0),
            Expr::Bool(b)  => self.emit_constant(Value::Bool(*b), 0),
            Expr::Str(s)   => self.emit_constant(Value::Str(s.clone()), 0),
            Expr::Null     => self.emit_constant(Value::Null, 0),

            Expr::Ident(name) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_opcode(OpCode::OpGetLocal, 0);
                    self.emit_u16(slot as u16, 0);
                } else {
                    let global_id = self.get_or_create_global(name);
                    self.emit_opcode(OpCode::OpGetGlobal, 0);
                    self.emit_u16(global_id as u16, 0);
                }
            }

            Expr::BinOp { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                match op {
                    BinOp::Add => { self.emit_opcode(OpCode::OpAdd, 0); }
                    BinOp::Sub => { self.emit_opcode(OpCode::OpSub, 0); }
                    BinOp::Mul => { self.emit_opcode(OpCode::OpMul, 0); }
                    BinOp::Div => { self.emit_opcode(OpCode::OpDiv, 0); }
                    BinOp::Mod => { self.emit_opcode(OpCode::OpMod, 0); }
                    BinOp::Eq  => { self.emit_opcode(OpCode::OpEqual, 0); }
                    BinOp::NotEq => {
                        self.emit_opcode(OpCode::OpEqual, 0);
                        self.emit_opcode(OpCode::OpNot, 0);
                    }
                    BinOp::Lt => { self.emit_opcode(OpCode::OpLess, 0); }
                    BinOp::Gt => { self.emit_opcode(OpCode::OpGreater, 0); }
                    BinOp::LtEq => {
                        self.emit_opcode(OpCode::OpGreater, 0);
                        self.emit_opcode(OpCode::OpNot, 0);
                    }
                    BinOp::GtEq => {
                        self.emit_opcode(OpCode::OpLess, 0);
                        self.emit_opcode(OpCode::OpNot, 0);
                    }
                    BinOp::In => { self.emit_opcode(OpCode::OpIn, 0); }
                    _ => {}
                };
            }

            Expr::UnaryOp { op, expr } => {
                self.compile_expr(expr)?;
                match op {
                    UnaryOp::Neg => { self.emit_opcode(OpCode::OpNeg, 0); }
                    UnaryOp::Not => { self.emit_opcode(OpCode::OpNot, 0); }
                };
            }

            Expr::List(items) => {
                for item in items {
                    self.compile_expr(item)?;
                }
                self.emit_opcode(OpCode::OpList, 0);
                self.emit_u16(items.len() as u16, 0);
            }

            Expr::Map(entries) => {
                for (k, v) in entries {
                    let idx = self.chunk.add_constant(Value::Str(k.clone()));
                    self.emit_opcode(OpCode::OpConstant, 0);
                    self.emit_u16(idx as u16, 0);
                    self.compile_expr(v)?;
                }
                self.emit_opcode(OpCode::OpMap, 0);
                self.emit_u16(entries.len() as u16, 0);
            }

            Expr::Index { expr, index } => {
                self.compile_expr(expr)?;
                self.compile_expr(index)?;
                self.emit_opcode(OpCode::OpIndex, 0);
            }

            Expr::Call { name, args, .. } => {
                if name == "print" {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    self.emit_opcode(OpCode::OpPrint, 0);
                } else {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    let global_id = self.get_or_create_global(name);
                    self.emit_opcode(OpCode::OpGetGlobal, 0);
                    self.emit_u16(global_id as u16, 0);
                    self.emit_opcode(OpCode::OpCall, 0);
                    self.emit_u16(args.len() as u16, 0);
                }
            }

            _ => {
                self.emit_constant(Value::Null, 0);
            }
        }
        Ok(())
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for local in self.locals.iter().rev() {
            if local.name == name {
                return Some(local.slot);
            }
        }
        None
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.locals.pop();
                self.emit_opcode(OpCode::OpPop, 0);
            } else {
                break;
            }
        }
    }
}
