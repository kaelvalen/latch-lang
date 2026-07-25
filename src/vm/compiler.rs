use crate::ast::*;
use crate::env::Value;
use crate::error::Result;
use super::chunk::{Chunk, OpCode};

struct Local {
    name: String,
    depth: usize,
}

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn compile(mut self, stmts: &[Stmt]) -> Result<Chunk> {
        for stmt in stmts {
            self.compile_stmt(stmt)?;
        }
        let null_idx = self.chunk.add_constant(Value::Null);
        self.chunk.write_opcode(OpCode::OpConstant, 0);
        self.chunk.write_u16(null_idx as u16, 0);
        self.chunk.write_opcode(OpCode::OpReturn, 0);
        Ok(self.chunk)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                self.compile_expr(value)?;
                if self.scope_depth > 0 {
                    self.locals.push(Local { name: name.clone(), depth: self.scope_depth });
                } else {
                    let idx = self.chunk.add_constant(Value::Str(name.clone()));
                    self.chunk.write_opcode(OpCode::OpDefineGlobal, 0);
                    self.chunk.write_u16(idx as u16, 0);
                }
            }

            Stmt::Assign { name, value } => {
                self.compile_expr(value)?;
                if let Some(slot) = self.resolve_local(name) {
                    self.chunk.write_opcode(OpCode::OpSetLocal, 0);
                    self.chunk.write_u16(slot as u16, 0);
                } else {
                    let idx = self.chunk.add_constant(Value::Str(name.clone()));
                    self.chunk.write_opcode(OpCode::OpDefineGlobal, 0);
                    self.chunk.write_u16(idx as u16, 0);
                }
            }

            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.chunk.write_opcode(OpCode::OpPop, 0);
            }

            Stmt::If { cond, then, else_ } => {
                self.compile_expr(cond)?;
                let jump_if_false_offset = self.emit_jump(OpCode::OpJumpIfFalse, 0);
                self.chunk.write_opcode(OpCode::OpPop, 0);

                self.begin_scope();
                for s in then {
                    self.compile_stmt(s)?;
                }
                self.end_scope();

                let jump_offset = self.emit_jump(OpCode::OpJump, 0);
                self.patch_jump(jump_if_false_offset);

                self.chunk.write_opcode(OpCode::OpPop, 0);
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
                self.chunk.write_opcode(OpCode::OpPop, 0);

                self.begin_scope();
                for s in body {
                    self.compile_stmt(s)?;
                }
                self.end_scope();

                self.emit_loop(loop_start, 0);
                self.patch_jump(exit_jump);
                self.chunk.write_opcode(OpCode::OpPop, 0);
            }

            Stmt::Return(expr) => {
                self.compile_expr(expr)?;
                self.chunk.write_opcode(OpCode::OpReturn, 0);
            }

            Stmt::Const { name, value, .. } => {
                self.compile_expr(value)?;
                let idx = self.chunk.add_constant(Value::Str(name.clone()));
                self.chunk.write_opcode(OpCode::OpDefineGlobal, 0);
                self.chunk.write_u16(idx as u16, 0);
            }

            Stmt::IndexAssign { target, index, value } => {
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                self.compile_expr(value)?;
                self.chunk.write_opcode(OpCode::OpIndexAssign, 0);
            }

            _ => {}
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Int(n) => {
                let idx = self.chunk.add_constant(Value::Int(*n));
                self.chunk.write_opcode(OpCode::OpConstant, 0);
                self.chunk.write_u16(idx as u16, 0);
            }
            Expr::Float(f) => {
                let idx = self.chunk.add_constant(Value::Float(*f));
                self.chunk.write_opcode(OpCode::OpConstant, 0);
                self.chunk.write_u16(idx as u16, 0);
            }
            Expr::Bool(b) => {
                let idx = self.chunk.add_constant(Value::Bool(*b));
                self.chunk.write_opcode(OpCode::OpConstant, 0);
                self.chunk.write_u16(idx as u16, 0);
            }
            Expr::Str(s) => {
                let idx = self.chunk.add_constant(Value::Str(s.clone()));
                self.chunk.write_opcode(OpCode::OpConstant, 0);
                self.chunk.write_u16(idx as u16, 0);
            }
            Expr::Null => {
                let idx = self.chunk.add_constant(Value::Null);
                self.chunk.write_opcode(OpCode::OpConstant, 0);
                self.chunk.write_u16(idx as u16, 0);
            }
            Expr::Ident(name) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.chunk.write_opcode(OpCode::OpGetLocal, 0);
                    self.chunk.write_u16(slot as u16, 0);
                } else {
                    let idx = self.chunk.add_constant(Value::Str(name.clone()));
                    self.chunk.write_opcode(OpCode::OpGetGlobal, 0);
                    self.chunk.write_u16(idx as u16, 0);
                }
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                match op {
                    BinOp::Add => { self.chunk.write_opcode(OpCode::OpAdd, 0); }
                    BinOp::Sub => { self.chunk.write_opcode(OpCode::OpSub, 0); }
                    BinOp::Mul => { self.chunk.write_opcode(OpCode::OpMul, 0); }
                    BinOp::Div => { self.chunk.write_opcode(OpCode::OpDiv, 0); }
                    BinOp::Mod => { self.chunk.write_opcode(OpCode::OpMod, 0); }
                    BinOp::Eq => { self.chunk.write_opcode(OpCode::OpEqual, 0); }
                    BinOp::NotEq => {
                        self.chunk.write_opcode(OpCode::OpEqual, 0);
                        self.chunk.write_opcode(OpCode::OpNot, 0);
                    }
                    BinOp::Lt => { self.chunk.write_opcode(OpCode::OpLess, 0); }
                    BinOp::Gt => { self.chunk.write_opcode(OpCode::OpGreater, 0); }
                    BinOp::LtEq => {
                        self.chunk.write_opcode(OpCode::OpGreater, 0);
                        self.chunk.write_opcode(OpCode::OpNot, 0);
                    }
                    BinOp::GtEq => {
                        self.chunk.write_opcode(OpCode::OpLess, 0);
                        self.chunk.write_opcode(OpCode::OpNot, 0);
                    }
                    BinOp::In => { self.chunk.write_opcode(OpCode::OpIn, 0); }
                    _ => {}
                };
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(expr)?;
                match op {
                    UnaryOp::Neg => { self.chunk.write_opcode(OpCode::OpNeg, 0); }
                    UnaryOp::Not => { self.chunk.write_opcode(OpCode::OpNot, 0); }
                };
            }
            Expr::List(items) => {
                for item in items {
                    self.compile_expr(item)?;
                }
                self.chunk.write_opcode(OpCode::OpList, 0);
                self.chunk.write_u16(items.len() as u16, 0);
            }
            Expr::Map(entries) => {
                for (k, v) in entries {
                    let idx = self.chunk.add_constant(Value::Str(k.clone()));
                    self.chunk.write_opcode(OpCode::OpConstant, 0);
                    self.chunk.write_u16(idx as u16, 0);
                    self.compile_expr(v)?;
                }
                self.chunk.write_opcode(OpCode::OpMap, 0);
                self.chunk.write_u16(entries.len() as u16, 0);
            }
            Expr::Index { expr, index } => {
                self.compile_expr(expr)?;
                self.compile_expr(index)?;
                self.chunk.write_opcode(OpCode::OpIndex, 0);
            }
            Expr::Call { name, args, .. } => {
                if name == "print" {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    self.chunk.write_opcode(OpCode::OpPrint, 0);
                } else {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    let idx = self.chunk.add_constant(Value::Str(name.clone()));
                    self.chunk.write_opcode(OpCode::OpGetGlobal, 0);
                    self.chunk.write_u16(idx as u16, 0);
                    self.chunk.write_opcode(OpCode::OpCall, 0);
                    self.chunk.write_u16(args.len() as u16, 0);
                }
            }
            _ => {
                let null_idx = self.chunk.add_constant(Value::Null);
                self.chunk.write_opcode(OpCode::OpConstant, 0);
                self.chunk.write_u16(null_idx as u16, 0);
            }
        }
        Ok(())
    }

    fn emit_jump(&mut self, instruction: OpCode, line: u32) -> usize {
        self.chunk.write_opcode(instruction, line);
        self.chunk.write_u16(0xffff, line);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len();
        let bytes = (jump as u16).to_be_bytes();
        self.chunk.code[offset] = bytes[0];
        self.chunk.code[offset + 1] = bytes[1];
    }

    fn emit_loop(&mut self, loop_start: usize, line: u32) {
        self.chunk.write_opcode(OpCode::OpLoop, line);
        self.chunk.write_u16(loop_start as u16, line);
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
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
                self.chunk.write_opcode(OpCode::OpPop, 0);
            } else {
                break;
            }
        }
    }
}
