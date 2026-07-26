use std::collections::HashMap;

use crate::ast::Type;
use crate::error::{LatchError, Result};
use crate::hir::*;

/// Pure HIR Level Type Checker — verifies types on resolved HirModule and HIR Nodes.
/// Employs lexically scoped local symbol stacks to correctly resolve variable shadowing.
pub struct TypeChecker {
    locals: Vec<HashMap<LocalId, Type>>,
    globals: HashMap<GlobalId, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            locals: vec![HashMap::new()],
            globals: HashMap::new(),
        }
    }

    /// Check resolved HirModule at HIR level
    pub fn check_module(&mut self, module: &HirModule) -> Vec<LatchError> {
        let mut errors = Vec::new();
        for stmt in &module.stmts {
            if let Err(e) = self.check_stmt(stmt) {
                errors.push(e);
            }
        }
        errors
    }

    pub fn check_program(&mut self, stmts: &[crate::ast::Stmt]) -> Vec<LatchError> {
        // Legacy entry point compatibility facade — resolves to HirModule
        let mut resolver = crate::resolver::Resolver::new();
        if let Ok(module) = resolver.resolve_module("<check>", stmts) {
            self.check_module(&module)
        } else {
            Vec::new()
        }
    }

    fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.locals.pop();
    }

    fn check_stmt(&mut self, stmt: &HirStmt) -> Result<()> {
        match stmt {
            HirStmt::LetLocal { id, value } => {
                let inferred = self.check_expr(value)?;
                if let Some(scope) = self.locals.last_mut() {
                    scope.insert(*id, inferred);
                }
            }

            HirStmt::LetGlobal { id, value } => {
                let inferred = self.check_expr(value)?;
                self.globals.insert(*id, inferred);
            }

            HirStmt::AssignLocal { id, value } => {
                let inferred = self.check_expr(value)?;
                if let Some(expected) = self.lookup_local(*id) {
                    if !types_compatible(&expected, &inferred) {
                        return Err(LatchError::TypeMismatch {
                            expected: format!("{expected:?}"),
                            found: format!("{inferred:?}"),
                        });
                    }
                }
                if let Some(scope) = self.locals.last_mut() {
                    scope.insert(*id, inferred);
                }
            }

            HirStmt::AssignGlobal { id, value } => {
                let inferred = self.check_expr(value)?;
                if let Some(expected) = self.globals.get(id) {
                    if !types_compatible(expected, &inferred) {
                        return Err(LatchError::TypeMismatch {
                            expected: format!("{expected:?}"),
                            found: format!("{inferred:?}"),
                        });
                    }
                }
                self.globals.insert(*id, inferred);
            }

            HirStmt::Expr(expr) => {
                self.check_expr(expr)?;
            }

            HirStmt::If { cond, then, else_ } => {
                let cond_type = self.check_expr(cond)?;
                if cond_type != Type::Bool && cond_type != Type::Any {
                    return Err(LatchError::TypeMismatch {
                        expected: "Bool".into(),
                        found: format!("{cond_type:?}"),
                    });
                }
                self.push_scope();
                for s in then {
                    self.check_stmt(s)?;
                }
                self.pop_scope();

                if let Some(else_stmt) = else_ {
                    self.push_scope();
                    self.check_stmt(else_stmt)?;
                    self.pop_scope();
                }
            }

            HirStmt::While { cond, body } => {
                let cond_type = self.check_expr(cond)?;
                if cond_type != Type::Bool && cond_type != Type::Any {
                    return Err(LatchError::TypeMismatch {
                        expected: "Bool".into(),
                        found: format!("{cond_type:?}"),
                    });
                }
                self.push_scope();
                for s in body {
                    self.check_stmt(s)?;
                }
                self.pop_scope();
            }

            HirStmt::Return(expr) => {
                self.check_expr(expr)?;
            }
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &HirExpr) -> Result<Type> {
        match expr {
            HirExpr::Constant(lit) => match lit {
                HirLiteral::Int(_) => Ok(Type::Int),
                HirLiteral::Float(_) => Ok(Type::Float),
                HirLiteral::Bool(_) => Ok(Type::Bool),
                HirLiteral::Str(_) => Ok(Type::Str),
                HirLiteral::Null => Ok(Type::Any),
            },

            HirExpr::Local(id) => {
                if let Some(t) = self.lookup_local(*id) {
                    Ok(t)
                } else {
                    Ok(Type::Any)
                }
            }

            HirExpr::Global(id) => {
                if let Some(t) = self.globals.get(id) {
                    Ok(t.clone())
                } else {
                    Ok(Type::Any)
                }
            }

            HirExpr::BinOp { op, left, right } => {
                let l_t = self.check_expr(left)?;
                let r_t = self.check_expr(right)?;

                match op {
                    HirOp::Add | HirOp::Sub | HirOp::Mul | HirOp::Div | HirOp::Mod => {
                        if l_t == Type::Float || r_t == Type::Float {
                            Ok(Type::Float)
                        } else {
                            Ok(Type::Int)
                        }
                    }
                    HirOp::Equal | HirOp::NotEqual | HirOp::Less | HirOp::LessEqual | HirOp::Greater | HirOp::GreaterEqual => {
                        Ok(Type::Bool)
                    }
                }
            }

            HirExpr::List(items) => {
                for i in items {
                    self.check_expr(i)?;
                }
                Ok(Type::List)
            }

            HirExpr::Map(pairs) => {
                for (k, v) in pairs {
                    self.check_expr(k)?;
                    self.check_expr(v)?;
                }
                Ok(Type::Dict)
            }

            HirExpr::Print(expr) => {
                self.check_expr(expr)?;
                Ok(Type::Any)
            }

            _ => Ok(Type::Any),
        }
    }

    fn lookup_local(&self, id: LocalId) -> Option<Type> {
        for scope in self.locals.iter().rev() {
            if let Some(t) = scope.get(&id) {
                return Some(t.clone());
            }
        }
        None
    }
}

fn types_compatible(expected: &Type, found: &Type) -> bool {
    if expected == &Type::Any || found == &Type::Any {
        return true;
    }
    expected == found
}
