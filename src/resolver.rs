use std::collections::HashMap;

use crate::ast::*;
use crate::error::Result;
use crate::hir::*;
use crate::symbol::{SymbolId, SymbolTable};

struct ResolverLocal {
    symbol_id: SymbolId,
    id: LocalId,
    depth: usize,
}

pub struct Resolver {
    pub symbols: SymbolTable,
    locals: Vec<ResolverLocal>,
    scope_depth: usize,
    globals_map: HashMap<SymbolId, GlobalId>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            symbols: SymbolTable::new(),
            locals: Vec::new(),
            scope_depth: 0,
            globals_map: HashMap::new(),
        }
    }

    pub fn resolve_program(&mut self, stmts: &[Stmt]) -> Result<Vec<HirStmt>> {
        let mut resolved = Vec::with_capacity(stmts.len());
        for stmt in stmts {
            resolved.push(self.resolve_stmt(stmt)?);
        }
        Ok(resolved)
    }

    pub fn resolve_module(&mut self, name: impl Into<String>, stmts: &[Stmt]) -> Result<HirModule> {
        let resolved_stmts = self.resolve_program(stmts)?;

        // Populate exported symbols from top-level declarations
        let mut exports = Vec::new();
        for stmt in stmts {
            match stmt {
                Stmt::Assign { name, .. } | Stmt::Let { name, .. } => {
                    exports.push(name.clone());
                }
                Stmt::Fn { name, .. } => {
                    exports.push(name.clone());
                }
                _ => {}
            }
        }

        Ok(HirModule {
            name: name.into(),
            stmts: resolved_stmts,
            exports,
        })
    }

    fn get_or_create_global(&mut self, symbol_id: SymbolId) -> GlobalId {
        if let Some(&id) = self.globals_map.get(&symbol_id) {
            id
        } else {
            let id = GlobalId(self.globals_map.len() as u32);
            self.globals_map.insert(symbol_id, id);
            id
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let sym_id = self.symbols.intern(name);
                let val = self.resolve_expr(value)?;
                if self.scope_depth > 0 {
                    let id = LocalId(self.locals.len() as u32);
                    self.locals.push(ResolverLocal {
                        symbol_id: sym_id,
                        id,
                        depth: self.scope_depth,
                    });
                    Ok(HirStmt::LetLocal { id, value: val })
                } else {
                    let id = self.get_or_create_global(sym_id);
                    Ok(HirStmt::LetGlobal { id, value: val })
                }
            }

            Stmt::Assign { name, value } => {
                let sym_id = self.symbols.intern(name);
                let val = self.resolve_expr(value)?;
                if let Some(id) = self.resolve_local(sym_id) {
                    Ok(HirStmt::AssignLocal { id, value: val })
                } else {
                    let id = self.get_or_create_global(sym_id);
                    Ok(HirStmt::LetGlobal { id, value: val })
                }
            }

            Stmt::Expr(expr) => {
                let val = self.resolve_expr(expr)?;
                Ok(HirStmt::Expr(val))
            }

            Stmt::If { cond, then, else_ } => {
                let opt_cond = self.resolve_expr(cond)?;
                self.scope_depth += 1;
                let mut opt_then = Vec::new();
                for s in then {
                    opt_then.push(self.resolve_stmt(s)?);
                }
                self.scope_depth -= 1;

                let opt_else = if let Some(else_s) = else_ {
                    self.scope_depth += 1;
                    let res = self.resolve_stmt(else_s)?;
                    self.scope_depth -= 1;
                    Some(Box::new(res))
                } else {
                    None
                };

                Ok(HirStmt::If {
                    cond: opt_cond,
                    then: opt_then,
                    else_: opt_else,
                })
            }

            Stmt::While { cond, body } => {
                let opt_cond = self.resolve_expr(cond)?;
                self.scope_depth += 1;
                let mut opt_body = Vec::new();
                for s in body {
                    opt_body.push(self.resolve_stmt(s)?);
                }
                self.scope_depth -= 1;

                Ok(HirStmt::While {
                    cond: opt_cond,
                    body: opt_body,
                })
            }

            Stmt::Return(expr) => {
                let val = self.resolve_expr(expr)?;
                Ok(HirStmt::Return(val))
            }

            _ => Ok(HirStmt::Expr(HirExpr::Constant(HirLiteral::Null))),
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<HirExpr> {
        match expr {
            Expr::Int(n) => Ok(HirExpr::Constant(HirLiteral::Int(*n))),
            Expr::Float(f) => Ok(HirExpr::Constant(HirLiteral::Float(*f))),
            Expr::Bool(b) => Ok(HirExpr::Constant(HirLiteral::Bool(*b))),
            Expr::Str(s) => Ok(HirExpr::Constant(HirLiteral::Str(s.clone()))),
            Expr::Ident(name) => {
                let sym_id = self.symbols.intern(name);
                if let Some(id) = self.resolve_local(sym_id) {
                    Ok(HirExpr::Local(id))
                } else {
                    let id = self.get_or_create_global(sym_id);
                    Ok(HirExpr::Global(id))
                }
            }
            Expr::BinOp { op, left, right } => {
                let l = self.resolve_expr(left)?;
                let r = self.resolve_expr(right)?;
                let hir_op = match op {
                    BinOp::Add => HirOp::Add,
                    BinOp::Sub => HirOp::Sub,
                    BinOp::Mul => HirOp::Mul,
                    BinOp::Div => HirOp::Div,
                    BinOp::Mod => HirOp::Mod,
                    BinOp::Eq => HirOp::Equal,
                    BinOp::NotEq => HirOp::NotEqual,
                    BinOp::Lt => HirOp::Less,
                    BinOp::LtEq => HirOp::LessEqual,
                    BinOp::Gt => HirOp::Greater,
                    BinOp::GtEq => HirOp::GreaterEqual,
                    _ => HirOp::Add,
                };
                Ok(HirExpr::BinOp {
                    op: hir_op,
                    left: Box::new(l),
                    right: Box::new(r),
                })
            }
            Expr::Call { name, args, kwargs: _ } => {
                let mut opt_args = Vec::with_capacity(args.len());
                for a in args {
                    opt_args.push(self.resolve_expr(a)?);
                }
                if name == "print" && !opt_args.is_empty() {
                    return Ok(HirExpr::Print(Box::new(opt_args.remove(0))));
                }
                let sym_id = self.symbols.intern(name);
                let global_id = self.get_or_create_global(sym_id);
                let func_id = FunctionId(global_id.0);
                Ok(HirExpr::Call {
                    func_id,
                    args: opt_args,
                })
            }
            Expr::List(items) => {
                let mut opt_items = Vec::with_capacity(items.len());
                for item in items {
                    opt_items.push(self.resolve_expr(item)?);
                }
                Ok(HirExpr::List(opt_items))
            }
            Expr::Map(pairs) => {
                let mut opt_pairs = Vec::with_capacity(pairs.len());
                for (k, v) in pairs {
                    opt_pairs.push((HirExpr::Constant(HirLiteral::Str(k.clone())), self.resolve_expr(v)?));
                }
                Ok(HirExpr::Map(opt_pairs))
            }
            _ => Ok(HirExpr::Constant(HirLiteral::Null)),
        }
    }

    fn resolve_local(&self, sym_id: SymbolId) -> Option<LocalId> {
        for local in self.locals.iter().rev() {
            if local.symbol_id == sym_id {
                return Some(local.id);
            }
        }
        None
    }
}
