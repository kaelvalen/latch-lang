use std::collections::HashMap;

use crate::ast::*;
use crate::error::{LatchError, Result};

pub struct TypeChecker {
    symbol_types: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            symbol_types: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, stmts: &[Stmt]) -> Vec<LatchError> {
        let mut errors = Vec::new();
        for stmt in stmts {
            if let Err(e) = self.check_stmt(stmt) {
                errors.push(e);
            }
        }
        errors
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, type_ann, value } => {
                let inferred = self.check_expr(value)?;
                if let Some(ann) = type_ann {
                    if !types_compatible(ann, &inferred) {
                        return Err(LatchError::TypeMismatch {
                            expected: format!("{ann}"),
                            found: format!("{inferred}"),
                        });
                    }
                    self.symbol_types.insert(name.clone(), ann.clone());
                } else {
                    self.symbol_types.insert(name.clone(), inferred);
                }
            }

            Stmt::Const { name, type_ann, value } => {
                let inferred = self.check_expr(value)?;
                if let Some(ann) = type_ann {
                    if !types_compatible(ann, &inferred) {
                        return Err(LatchError::TypeMismatch {
                            expected: format!("{ann}"),
                            found: format!("{inferred}"),
                        });
                    }
                    self.symbol_types.insert(name.clone(), ann.clone());
                } else {
                    self.symbol_types.insert(name.clone(), inferred);
                }
            }

            Stmt::Fn { name, return_type, .. } => {
                if let Some(ret) = return_type {
                    self.symbol_types.insert(name.clone(), ret.clone());
                }
            }

            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
            }

            _ => {}
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Int(_) => Ok(Type::Int),
            Expr::Float(_) => Ok(Type::Float),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::Str(_) => Ok(Type::Str),
            Expr::Null => Ok(Type::Any),

            Expr::List(_) => Ok(Type::List),
            Expr::Map(_) => Ok(Type::Dict),

            Expr::Ident(name) => {
                if let Some(t) = self.symbol_types.get(name) {
                    Ok(t.clone())
                } else {
                    Ok(Type::Any)
                }
            }

            Expr::BinOp { op, left, right } => {
                let l_t = self.check_expr(left)?;
                let r_t = self.check_expr(right)?;

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if l_t == Type::Float || r_t == Type::Float {
                            Ok(Type::Float)
                        } else {
                            Ok(Type::Int)
                        }
                    }
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                        Ok(Type::Bool)
                    }
                    _ => Ok(Type::Any),
                }
            }

            _ => Ok(Type::Any),
        }
    }
}

fn types_compatible(expected: &Type, found: &Type) -> bool {
    if expected == &Type::Any || found == &Type::Any {
        return true;
    }
    expected == found
}
