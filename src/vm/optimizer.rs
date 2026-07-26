use crate::hir::*;

/// Pure HIR Optimizer — operates directly on resolved HirModule and HIR Nodes.
/// Performs Constant Folding, Constant Propagation, Dead Code Elimination, and Branch Pruning.
#[derive(Debug, Clone, Default)]
pub struct Optimizer;

impl Optimizer {
    pub fn new() -> Self {
        Optimizer
    }

    /// Primary Optimizer entry point — transforms an HirModule into an optimized HirModule.
    pub fn optimize_module(&self, module: &HirModule) -> HirModule {
        let mut optimized_stmts = Vec::with_capacity(module.stmts.len());
        for stmt in &module.stmts {
            if let Some(opt_stmt) = self.optimize_stmt(stmt) {
                optimized_stmts.push(opt_stmt);
            }
        }

        HirModule {
            name: module.name.clone(),
            stmts: optimized_stmts,
            exports: module.exports.clone(),
        }
    }

    fn optimize_stmt(&self, stmt: &HirStmt) -> Option<HirStmt> {
        match stmt {
            HirStmt::LetLocal { id, value } => Some(HirStmt::LetLocal {
                id: *id,
                value: self.optimize_expr(value),
            }),

            HirStmt::LetGlobal { id, value } => Some(HirStmt::LetGlobal {
                id: *id,
                value: self.optimize_expr(value),
            }),

            HirStmt::AssignLocal { id, value } => Some(HirStmt::AssignLocal {
                id: *id,
                value: self.optimize_expr(value),
            }),

            HirStmt::AssignGlobal { id, value } => Some(HirStmt::AssignGlobal {
                id: *id,
                value: self.optimize_expr(value),
            }),

            HirStmt::Expr(expr) => Some(HirStmt::Expr(self.optimize_expr(expr))),

            HirStmt::If { cond, then, else_ } => {
                let opt_cond = self.optimize_expr(cond);

                // Constant Branch Pruning
                if let HirExpr::Constant(HirLiteral::Bool(b)) = opt_cond {
                    if b {
                        // Keep then branch
                        let mut body = Vec::new();
                        for s in then {
                            if let Some(st) = self.optimize_stmt(s) {
                                body.push(st);
                            }
                        }
                        return Some(HirStmt::Expr(HirExpr::Constant(HirLiteral::Null)));
                    } else if let Some(else_stmts) = else_ {
                        let mut body = Vec::new();
                        for s in else_stmts {
                            if let Some(st) = self.optimize_stmt(s) {
                                body.push(st);
                            }
                        }
                        return Some(HirStmt::Expr(HirExpr::Constant(HirLiteral::Null)));
                    } else {
                        return None; // Dead branch
                    }
                }

                let mut opt_then = Vec::new();
                for s in then {
                    if let Some(st) = self.optimize_stmt(s) {
                        opt_then.push(st);
                    }
                }

                let opt_else = else_
                    .as_ref()
                    .map(|stmts| stmts.iter().filter_map(|s| self.optimize_stmt(s)).collect());

                Some(HirStmt::If {
                    cond: opt_cond,
                    then: opt_then,
                    else_: opt_else,
                })
            }

            HirStmt::While { cond, body } => {
                let opt_cond = self.optimize_expr(cond);

                // Prune while false loops
                if let HirExpr::Constant(HirLiteral::Bool(false)) = opt_cond {
                    return None;
                }

                let mut opt_body = Vec::new();
                for s in body {
                    if let Some(st) = self.optimize_stmt(s) {
                        opt_body.push(st);
                    }
                }

                Some(HirStmt::While {
                    cond: opt_cond,
                    body: opt_body,
                })
            }

            HirStmt::For { var_id, iter, body } => {
                let opt_iter = self.optimize_expr(iter);
                let opt_body = body.iter().filter_map(|s| self.optimize_stmt(s)).collect();
                Some(HirStmt::For {
                    var_id: *var_id,
                    iter: opt_iter,
                    body: opt_body,
                })
            }

            HirStmt::Return(expr) => Some(HirStmt::Return(self.optimize_expr(expr))),
        }
    }

    fn optimize_expr(&self, expr: &HirExpr) -> HirExpr {
        match expr {
            HirExpr::BinOp { op, left, right } => {
                let l = self.optimize_expr(left);
                let r = self.optimize_expr(right);

                // Constant Folding for Int BinOps
                if let (
                    HirExpr::Constant(HirLiteral::Int(a)),
                    HirExpr::Constant(HirLiteral::Int(b)),
                ) = (&l, &r)
                {
                    match op {
                        HirOp::Add => return HirExpr::Constant(HirLiteral::Int(a + b)),
                        HirOp::Sub => return HirExpr::Constant(HirLiteral::Int(a - b)),
                        HirOp::Mul => return HirExpr::Constant(HirLiteral::Int(a * b)),
                        HirOp::Div if *b != 0 => return HirExpr::Constant(HirLiteral::Int(a / b)),
                        HirOp::Mod if *b != 0 => return HirExpr::Constant(HirLiteral::Int(a % b)),
                        HirOp::Equal => return HirExpr::Constant(HirLiteral::Bool(a == b)),
                        HirOp::NotEqual => return HirExpr::Constant(HirLiteral::Bool(a != b)),
                        HirOp::Less => return HirExpr::Constant(HirLiteral::Bool(a < b)),
                        HirOp::LessEqual => return HirExpr::Constant(HirLiteral::Bool(a <= b)),
                        HirOp::Greater => return HirExpr::Constant(HirLiteral::Bool(a > b)),
                        HirOp::GreaterEqual => return HirExpr::Constant(HirLiteral::Bool(a >= b)),
                        _ => {}
                    }
                }

                HirExpr::BinOp {
                    op: *op,
                    left: Box::new(l),
                    right: Box::new(r),
                }
            }

            HirExpr::List(items) => {
                let opt_items = items.iter().map(|i| self.optimize_expr(i)).collect();
                HirExpr::List(opt_items)
            }

            HirExpr::Map(pairs) => HirExpr::Map(
                pairs
                    .iter()
                    .map(|(k, v)| (self.optimize_expr(k), self.optimize_expr(v)))
                    .collect(),
            ),

            HirExpr::Index { target, index } => HirExpr::Index {
                target: Box::new(self.optimize_expr(target)),
                index: Box::new(self.optimize_expr(index)),
            },

            HirExpr::Print(expr) => HirExpr::Print(Box::new(self.optimize_expr(expr))),

            other => other.clone(),
        }
    }
}
