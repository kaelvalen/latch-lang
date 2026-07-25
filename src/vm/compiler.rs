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
        self.chunk.write(OpCode::OpNull, 0);
        self.chunk.write(OpCode::OpReturn, 0);
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
                    self.chunk.write(OpCode::OpDefineGlobal(idx), 0);
                }
            }

            Stmt::Assign { name, value } => {
                self.compile_expr(value)?;
                if let Some(slot) = self.resolve_local(name) {
                    self.chunk.write(OpCode::OpSetLocal(slot), 0);
                } else {
                    let idx = self.chunk.add_constant(Value::Str(name.clone()));
                    self.chunk.write(OpCode::OpDefineGlobal(idx), 0);
                }
            }

            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.chunk.write(OpCode::OpPop, 0);
            }

            Stmt::If { cond, then, else_ } => {
                self.compile_expr(cond)?;
                let jump_if_false_ip = self.chunk.write(OpCode::OpJumpIfFalse(0), 0);
                self.chunk.write(OpCode::OpPop, 0);

                self.begin_scope();
                for s in then {
                    self.compile_stmt(s)?;
                }
                self.end_scope();

                let jump_ip = self.chunk.write(OpCode::OpJump(0), 0);
                let else_target = self.chunk.code.len();
                self.chunk.code[jump_if_false_ip] = OpCode::OpJumpIfFalse(else_target);

                self.chunk.write(OpCode::OpPop, 0);
                if let Some(else_stmt) = else_ {
                    self.begin_scope();
                    self.compile_stmt(else_stmt)?;
                    self.end_scope();
                }

                let end_target = self.chunk.code.len();
                self.chunk.code[jump_ip] = OpCode::OpJump(end_target);
            }

            Stmt::While { cond, body } => {
                let loop_start = self.chunk.code.len();
                self.compile_expr(cond)?;
                let exit_jump = self.chunk.write(OpCode::OpJumpIfFalse(0), 0);
                self.chunk.write(OpCode::OpPop, 0);

                self.begin_scope();
                for s in body {
                    self.compile_stmt(s)?;
                }
                self.end_scope();

                self.chunk.write(OpCode::OpLoop(loop_start), 0);
                let exit_target = self.chunk.code.len();
                self.chunk.code[exit_jump] = OpCode::OpJumpIfFalse(exit_target);
                self.chunk.write(OpCode::OpPop, 0);
            }

            Stmt::Return(expr) => {
                self.compile_expr(expr)?;
                self.chunk.write(OpCode::OpReturn, 0);
            }

            Stmt::Const { name, value, .. } => {
                self.compile_expr(value)?;
                let idx = self.chunk.add_constant(Value::Str(name.clone()));
                self.chunk.write(OpCode::OpDefineGlobal(idx), 0);
            }

            Stmt::IndexAssign { target, index, value } => {
                self.compile_expr(target)?;
                self.compile_expr(index)?;
                self.compile_expr(value)?;
                self.chunk.write(OpCode::OpIndexAssign, 0);
            }

            _ => {
                // Fallback for complex statement types
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Int(n) => {
                let idx = self.chunk.add_constant(Value::Int(*n));
                self.chunk.write(OpCode::OpConstant(idx), 0);
            }
            Expr::Float(f) => {
                let idx = self.chunk.add_constant(Value::Float(*f));
                self.chunk.write(OpCode::OpConstant(idx), 0);
            }
            Expr::Bool(b) => {
                if *b {
                    self.chunk.write(OpCode::OpTrue, 0);
                } else {
                    self.chunk.write(OpCode::OpFalse, 0);
                }
            }
            Expr::Str(s) => {
                let idx = self.chunk.add_constant(Value::Str(s.clone()));
                self.chunk.write(OpCode::OpConstant(idx), 0);
            }
            Expr::Null => {
                self.chunk.write(OpCode::OpNull, 0);
            }
            Expr::Ident(name) => {
                if let Some(slot) = self.resolve_local(name) {
                    self.chunk.write(OpCode::OpGetLocal(slot), 0);
                } else {
                    let idx = self.chunk.add_constant(Value::Str(name.clone()));
                    self.chunk.write(OpCode::OpGetGlobal(idx), 0);
                }
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                match op {
                    BinOp::Add => self.chunk.write(OpCode::OpAdd, 0),
                    BinOp::Sub => self.chunk.write(OpCode::OpSub, 0),
                    BinOp::Mul => self.chunk.write(OpCode::OpMul, 0),
                    BinOp::Div => self.chunk.write(OpCode::OpDiv, 0),
                    BinOp::Mod => self.chunk.write(OpCode::OpMod, 0),
                    BinOp::Eq => self.chunk.write(OpCode::OpEq, 0),
                    BinOp::NotEq => self.chunk.write(OpCode::OpNotEq, 0),
                    BinOp::Lt => self.chunk.write(OpCode::OpLt, 0),
                    BinOp::Gt => self.chunk.write(OpCode::OpGt, 0),
                    BinOp::LtEq => self.chunk.write(OpCode::OpLtEq, 0),
                    BinOp::GtEq => self.chunk.write(OpCode::OpGtEq, 0),
                    BinOp::In => self.chunk.write(OpCode::OpIn, 0),
                    _ => 0,
                };
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(expr)?;
                match op {
                    UnaryOp::Neg => self.chunk.write(OpCode::OpNeg, 0),
                    UnaryOp::Not => self.chunk.write(OpCode::OpNot, 0),
                };
            }
            Expr::List(items) => {
                for item in items {
                    self.compile_expr(item)?;
                }
                self.chunk.write(OpCode::OpList(items.len()), 0);
            }
            Expr::Map(entries) => {
                for (k, v) in entries {
                    let idx = self.chunk.add_constant(Value::Str(k.clone()));
                    self.chunk.write(OpCode::OpConstant(idx), 0);
                    self.compile_expr(v)?;
                }
                self.chunk.write(OpCode::OpMap(entries.len()), 0);
            }
            Expr::Index { expr, index } => {
                self.compile_expr(expr)?;
                self.compile_expr(index)?;
                self.chunk.write(OpCode::OpIndex, 0);
            }
            Expr::Call { name, args, .. } => {
                if name == "print" {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    self.chunk.write(OpCode::OpPrint, 0);
                } else {
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    let idx = self.chunk.add_constant(Value::Str(name.clone()));
                    self.chunk.write(OpCode::OpGetGlobal(idx), 0);
                    self.chunk.write(OpCode::OpCall(args.len()), 0);
                }
            }
            _ => {
                self.chunk.write(OpCode::OpNull, 0);
            }
        }
        Ok(())
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
                self.chunk.write(OpCode::OpPop, 0);
            } else {
                break;
            }
        }
    }
}
