use std::collections::HashMap;

use crate::ast::*;
use crate::env::Value;
use crate::error::Result;
use crate::hir::*;

struct ResolverLocal {
    name: String,
    id: LocalId,
    depth: usize,
}

pub struct Resolver {
    locals: Vec<ResolverLocal>,
    scope_depth: usize,
    globals_map: HashMap<String, GlobalId>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
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

    fn get_or_create_global(&mut self, name: &str) -> GlobalId {
        if let Some(&id) = self.globals_map.get(name) {
            id
        } else {
            let id = GlobalId(self.globals_map.len() as u32);
            self.globals_map.insert(name.to_string(), id);
            id
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<HirStmt> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = self.resolve_expr(value)?;
                if self.scope_depth > 0 {
                    let id = LocalId(self.locals.len() as u32);
                    self.locals.push(ResolverLocal {
                        name: name.clone(),
                        id,
                        depth: self.scope_depth,
                    });
                    Ok(HirStmt::LetLocal { id, value: val })
                } else {
                    let id = self.get_or_create_global(name);
                    Ok(HirStmt::LetGlobal { id, value: val })
                }
            }

            Stmt::Assign { name, value } => {
                let val = self.resolve_expr(value)?;
                if let Some(id) = self.resolve_local(name) {
                    Ok(HirStmt::AssignLocal { id, value: val })
                } else {
                    let id = self.get_or_create_global(name);
                    Ok(HirStmt::AssignGlobal { id, value: val })
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

            Stmt::Const { name, value, .. } => {
                let val = self.resolve_expr(value)?;
                let id = self.get_or_create_global(name);
                Ok(HirStmt::LetGlobal { id, value: val })
            }

            _ => Ok(HirStmt::Expr(HirExpr::Constant(Value::Null))),
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<HirExpr> {
        match expr {
            Expr::Int(n)   => Ok(HirExpr::Constant(Value::Int(*n))),
            Expr::Float(f) => Ok(HirExpr::Constant(Value::Float(*f))),
            Expr::Bool(b)  => Ok(HirExpr::Constant(Value::Bool(*b))),
            Expr::Str(s)   => Ok(HirExpr::Constant(Value::Str(s.clone()))),
            Expr::Null     => Ok(HirExpr::Constant(Value::Null)),

            Expr::Ident(name) => {
                if let Some(id) = self.resolve_local(name) {
                    Ok(HirExpr::Local(id))
                } else {
                    let id = self.get_or_create_global(name);
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
                    _ => HirOp::Equal,
                };
                Ok(HirExpr::BinOp {
                    op: hir_op,
                    left: Box::new(l),
                    right: Box::new(r),
                })
            }

            Expr::Call { name: _, args, .. } => {
                let mut resolved_args = Vec::with_capacity(args.len());
                for arg in args {
                    resolved_args.push(self.resolve_expr(arg)?);
                }
                let func_id = FunctionId(0);
                Ok(HirExpr::Call {
                    func_id,
                    args: resolved_args,
                })
            }

            _ => Ok(HirExpr::Constant(Value::Null)),
        }
    }

    fn resolve_local(&self, name: &str) -> Option<LocalId> {
        for local in self.locals.iter().rev() {
            if local.name == name {
                return Some(local.id);
            }
        }
        None
    }
}
