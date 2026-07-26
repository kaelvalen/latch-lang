use super::chunk::{ChunkBuilder, Constant, OpCode};
use super::peephole::BytecodePeephole;
use crate::env::{ObjFunction, ObjFunctionBuilder, ObjRef};
use crate::error::Result;
use crate::hir::*;

/// Dumb Bytecode Emitter — transforms resolved HirModule directly into a compiled Chunk.
/// Contains zero AST imports, Value runtime dependencies, scope maps, or semantic checking logic.
pub struct Compiler {
    chunk: ChunkBuilder,
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: ChunkBuilder::new(),
        }
    }

    /// Pure Compiler entry point — accepts a resolved HirModule.
    pub fn compile_module(mut self, module: &HirModule) -> Result<ObjRef<ObjFunction>> {
        for stmt in &module.stmts {
            self.compile_stmt(stmt)?;
        }
        self.emit_constant(Constant::Null, 0);
        self.emit_return(0);
        BytecodePeephole::optimize(&mut self.chunk);

        let script_fn = ObjFunctionBuilder::new(module.name.clone(), 0)
            .with_chunk(self.chunk.build())
            .build();
        Ok(ObjRef::new(script_fn))
    }

    pub fn compile_hir(self, stmts: &[HirStmt]) -> Result<ObjRef<ObjFunction>> {
        let module = HirModule {
            name: "<script>".into(),
            stmts: stmts.to_vec(),
            exports: Vec::new(),
        };
        self.compile_module(&module)
    }

    // ── Low-Level Emitter Methods ─────────────────────────────

    #[inline]
    fn emit_opcode(&mut self, op: OpCode, line: u32) -> usize {
        self.chunk.write_opcode(op, line)
    }

    #[inline]
    fn emit_u16(&mut self, val: u16, line: u32) {
        self.chunk.write_u16(val, line);
    }

    fn emit_constant(&mut self, val: Constant, line: u32) {
        let idx = self.chunk.add_constant(val);
        self.emit_opcode(OpCode::OpConstant, line);
        self.emit_u16(idx as u16, line);
    }

    fn emit_jump(&mut self, instruction: OpCode, line: u32) -> usize {
        self.emit_opcode(instruction, line);
        self.emit_u16(0xffff, line);
        self.chunk.code_len() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code_len();
        self.chunk.patch_u16(offset, jump as u16);
    }

    fn emit_loop(&mut self, loop_start: usize, line: u32) {
        self.emit_opcode(OpCode::OpLoop, line);
        self.emit_u16(loop_start as u16, line);
    }

    fn emit_call(&mut self, argc: u16, line: u32) {
        self.emit_opcode(OpCode::OpCall, line);
        self.emit_u16(argc, line);
    }

    fn emit_return(&mut self, line: u32) {
        self.emit_opcode(OpCode::OpReturn, line);
    }

    fn emit_pop(&mut self, line: u32) {
        self.emit_opcode(OpCode::OpPop, line);
    }

    // ── HIR Emitter ──────────────────────────────────────────

    fn compile_stmt(&mut self, stmt: &HirStmt) -> Result<()> {
        match stmt {
            HirStmt::LetLocal { id, value } => {
                self.compile_expr(value)?;
                self.emit_opcode(OpCode::OpSetLocal, 0);
                self.emit_u16(id.0 as u16, 0);
                self.emit_pop(0);
            }

            HirStmt::LetGlobal { id, value } => {
                self.compile_expr(value)?;
                self.emit_opcode(OpCode::OpDefineGlobal, 0);
                self.emit_u16(id.0 as u16, 0);
            }

            HirStmt::AssignLocal { id, value } => {
                self.compile_expr(value)?;
                self.emit_opcode(OpCode::OpSetLocal, 0);
                self.emit_u16(id.0 as u16, 0);
                self.emit_pop(0);
            }

            HirStmt::AssignGlobal { id, value } => {
                self.compile_expr(value)?;
                self.emit_opcode(OpCode::OpSetGlobal, 0);
                self.emit_u16(id.0 as u16, 0);
                self.emit_pop(0);
            }

            HirStmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.emit_pop(0);
            }

            HirStmt::If { cond, then, else_ } => {
                self.compile_expr(cond)?;
                let jump_false = self.emit_jump(OpCode::OpJumpIfFalse, 0);

                for s in then {
                    self.compile_stmt(s)?;
                }

                let jump_then = self.emit_jump(OpCode::OpJump, 0);
                self.patch_jump(jump_false);

                if let Some(else_stmts) = else_ {
                    for s in else_stmts {
                        self.compile_stmt(s)?;
                    }
                }

                self.patch_jump(jump_then);
            }

            HirStmt::While { cond, body } => {
                let loop_start = self.chunk.code_len();
                self.compile_expr(cond)?;

                let exit_jump = self.emit_jump(OpCode::OpJumpIfFalse, 0);

                for s in body {
                    self.compile_stmt(s)?;
                }

                self.emit_loop(loop_start, 0);
                self.patch_jump(exit_jump);
            }

            HirStmt::Return(expr) => {
                self.compile_expr(expr)?;
                self.emit_return(0);
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &HirExpr) -> Result<()> {
        match expr {
            HirExpr::Constant(lit) => {
                let c = match lit {
                    HirLiteral::Int(n) => Constant::Int(*n),
                    HirLiteral::Float(f) => Constant::Float(*f),
                    HirLiteral::Bool(b) => Constant::Bool(*b),
                    HirLiteral::Str(s) => Constant::Str(s.clone()),
                    HirLiteral::Null => Constant::Null,
                };
                self.emit_constant(c, 0);
            }

            HirExpr::Local(id) => {
                self.emit_opcode(OpCode::OpGetLocal, 0);
                self.emit_u16(id.0 as u16, 0);
            }

            HirExpr::Global(id) => {
                self.emit_opcode(OpCode::OpGetGlobal, 0);
                self.emit_u16(id.0 as u16, 0);
            }

            HirExpr::Upvalue(id) => {
                self.emit_opcode(OpCode::OpGetUpvalue, 0);
                self.emit_u16(id.0 as u16, 0);
            }

            HirExpr::Function { name, params, body } => {
                let mut sub_compiler = Compiler::new();
                for stmt in body {
                    sub_compiler.compile_stmt(stmt)?;
                }
                sub_compiler.emit_constant(Constant::Null, 0);
                sub_compiler.emit_return(0);
                BytecodePeephole::optimize(&mut sub_compiler.chunk);

                let sub_fn = ObjFunctionBuilder::new(name.clone(), params.len())
                    .with_chunk(sub_compiler.chunk.build())
                    .build();
                let func_arc = std::sync::Arc::new(sub_fn);
                let const_idx = self.chunk.add_constant(Constant::Function(func_arc));
                self.emit_opcode(OpCode::OpClosure, 0);
                self.emit_u16(const_idx as u16, 0);
            }

            HirExpr::BinOp { op, left, right } => match op {
                HirOp::Or => {
                    self.compile_expr(left)?;
                    self.emit_opcode(OpCode::OpDup, 0);
                    let jump_false = self.emit_jump(OpCode::OpJumpIfFalse, 0);
                    let jump_end = self.emit_jump(OpCode::OpJump, 0);
                    self.patch_jump(jump_false);
                    self.emit_pop(0);
                    self.compile_expr(right)?;
                    self.patch_jump(jump_end);
                }
                HirOp::And => {
                    self.compile_expr(left)?;
                    self.emit_opcode(OpCode::OpDup, 0);
                    let jump_false = self.emit_jump(OpCode::OpJumpIfFalse, 0);
                    self.emit_pop(0);
                    self.compile_expr(right)?;
                    let jump_end = self.emit_jump(OpCode::OpJump, 0);
                    self.patch_jump(jump_false);
                    self.patch_jump(jump_end);
                }
                _ => {
                    self.compile_expr(left)?;
                    self.compile_expr(right)?;
                    match op {
                        HirOp::Add => {
                            self.emit_opcode(OpCode::OpAdd, 0);
                        }
                        HirOp::Sub => {
                            self.emit_opcode(OpCode::OpSub, 0);
                        }
                        HirOp::Mul => {
                            self.emit_opcode(OpCode::OpMul, 0);
                        }
                        HirOp::Div => {
                            self.emit_opcode(OpCode::OpDiv, 0);
                        }
                        HirOp::Mod => {
                            self.emit_opcode(OpCode::OpMod, 0);
                        }
                        HirOp::Equal => {
                            self.emit_opcode(OpCode::OpEqual, 0);
                        }
                        HirOp::NotEqual => {
                            self.emit_opcode(OpCode::OpEqual, 0);
                            self.emit_opcode(OpCode::OpNot, 0);
                        }
                        HirOp::Less => {
                            self.emit_opcode(OpCode::OpLess, 0);
                        }
                        HirOp::LessEqual => {
                            self.emit_opcode(OpCode::OpGreater, 0);
                            self.emit_opcode(OpCode::OpNot, 0);
                        }
                        HirOp::Greater => {
                            self.emit_opcode(OpCode::OpGreater, 0);
                        }
                        HirOp::GreaterEqual => {
                            self.emit_opcode(OpCode::OpLess, 0);
                            self.emit_opcode(OpCode::OpNot, 0);
                        }
                        _ => {}
                    }
                }
            },

            HirExpr::Call { func_id, args } => {
                self.emit_opcode(OpCode::OpGetGlobal, 0);
                self.emit_u16(func_id.0 as u16, 0);
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_call(args.len() as u16, 0);
            }

            HirExpr::List(items) => {
                let count = items.len();
                for item in items {
                    self.compile_expr(item)?;
                }
                self.emit_opcode(OpCode::OpList, 0);
                self.emit_u16(count as u16, 0);
            }

            HirExpr::Map(pairs) => {
                let count = pairs.len();
                for (k, v) in pairs {
                    self.compile_expr(k)?;
                    self.compile_expr(v)?;
                }
                self.emit_opcode(OpCode::OpMap, 0);
                self.emit_u16(count as u16, 0);
            }

            HirExpr::Print(expr) => {
                self.compile_expr(expr)?;
                self.emit_opcode(OpCode::OpPrint, 0);
            }
        }
        Ok(())
    }
}
