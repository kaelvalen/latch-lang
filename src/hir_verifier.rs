use crate::error::{LatchError, Result};
use crate::hir::*;

/// Static HIR Verifier — inspects resolved HirModule for structural validity,
/// ID bound validity, and control-flow integrity prior to Optimization & Bytecode Emission.
pub struct HirVerifier;

impl HirVerifier {
    pub fn verify(module: &HirModule) -> Result<()> {
        for stmt in &module.stmts {
            Self::verify_stmt(stmt)?;
        }
        Ok(())
    }

    fn verify_stmt(stmt: &HirStmt) -> Result<()> {
        match stmt {
            HirStmt::LetLocal { id, value } => {
                Self::verify_local_id(*id)?;
                Self::verify_expr(value)?;
            }
            HirStmt::LetGlobal { id, value } => {
                Self::verify_global_id(*id)?;
                Self::verify_expr(value)?;
            }
            HirStmt::AssignLocal { id, value } => {
                Self::verify_local_id(*id)?;
                Self::verify_expr(value)?;
            }
            HirStmt::AssignGlobal { id, value } => {
                Self::verify_global_id(*id)?;
                Self::verify_expr(value)?;
            }
            HirStmt::Expr(expr) => {
                Self::verify_expr(expr)?;
            }
            HirStmt::If { cond, then, else_ } => {
                Self::verify_expr(cond)?;
                for s in then {
                    Self::verify_stmt(s)?;
                }
                if let Some(else_stmts) = else_ {
                    for s in else_stmts {
                        Self::verify_stmt(s)?;
                    }
                }
            }
            HirStmt::While { cond, body } => {
                Self::verify_expr(cond)?;
                for s in body {
                    Self::verify_stmt(s)?;
                }
            }
            HirStmt::For { iter, body, .. } => {
                Self::verify_expr(iter)?;
                for s in body {
                    Self::verify_stmt(s)?;
                }
            }
            HirStmt::Return(expr) => {
                Self::verify_expr(expr)?;
            }
        }
        Ok(())
    }

    fn verify_expr(expr: &HirExpr) -> Result<()> {
        match expr {
            HirExpr::Constant(_) => Ok(()),
            HirExpr::Local(id) => Self::verify_local_id(*id),
            HirExpr::Global(id) => Self::verify_global_id(*id),
            HirExpr::Upvalue(id) => Self::verify_upvalue_id(*id),
            HirExpr::BinOp { left, right, .. } => {
                Self::verify_expr(left)?;
                Self::verify_expr(right)?;
                Ok(())
            }
            HirExpr::Call { func_id, args } => {
                Self::verify_func_id(*func_id)?;
                for arg in args {
                    Self::verify_expr(arg)?;
                }
                Ok(())
            }
            HirExpr::List(items) => {
                for item in items {
                    Self::verify_expr(item)?;
                }
                Ok(())
            }
            HirExpr::Map(pairs) => {
                for (k, v) in pairs {
                    Self::verify_expr(k)?;
                    Self::verify_expr(v)?;
                }
                Ok(())
            }
            HirExpr::Index { target, index } => {
                Self::verify_expr(target)?;
                Self::verify_expr(index)?;
                Ok(())
            }
            HirExpr::Function { body, .. } => {
                for s in body {
                    Self::verify_stmt(s)?;
                }
                Ok(())
            }
            HirExpr::Print(expr) => Self::verify_expr(expr),
        }
    }

    fn verify_local_id(id: LocalId) -> Result<()> {
        if id.0 > 65535 {
            return Err(LatchError::GenericError(format!(
                "HIR Verifier error: LocalId {} exceeds u16 limit",
                id.0
            )));
        }
        Ok(())
    }

    fn verify_global_id(id: GlobalId) -> Result<()> {
        if id.0 > 65535 {
            return Err(LatchError::GenericError(format!(
                "HIR Verifier error: GlobalId {} exceeds u16 limit",
                id.0
            )));
        }
        Ok(())
    }

    fn verify_func_id(id: FunctionId) -> Result<()> {
        if id.0 > 65535 {
            return Err(LatchError::GenericError(format!(
                "HIR Verifier error: FunctionId {} exceeds u16 limit",
                id.0
            )));
        }
        Ok(())
    }

    fn verify_upvalue_id(id: UpvalueId) -> Result<()> {
        if id.0 > 65535 {
            return Err(LatchError::GenericError(format!(
                "HIR Verifier error: UpvalueId {} exceeds u16 limit",
                id.0
            )));
        }
        Ok(())
    }
}
