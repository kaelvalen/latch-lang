use crate::ast::*;

/// AST & Expression Optimizer Pipeline.
/// Implements Constant Folding, Dead Code Elimination, and Strength Reduction.
pub struct Optimizer;

impl Optimizer {
    pub fn new() -> Self {
        Optimizer
    }

    /// Optimize a list of AST statements.
    pub fn optimize_stmts(&self, stmts: &[Stmt]) -> Vec<Stmt> {
        let mut optimized = Vec::with_capacity(stmts.len());
        for stmt in stmts {
            if let Some(opt_stmt) = self.optimize_stmt(stmt) {
                optimized.push(opt_stmt);
            }
        }
        optimized
    }

    fn optimize_stmt(&self, stmt: &Stmt) -> Option<Stmt> {
        match stmt {
            Stmt::Let { name, value, type_ann } => {
                let opt_val = self.optimize_expr(value);
                Some(Stmt::Let {
                    name: name.clone(),
                    value: opt_val,
                    type_ann: type_ann.clone(),
                })
            }

            Stmt::Assign { name, value } => {
                let opt_val = self.optimize_expr(value);
                Some(Stmt::Assign {
                    name: name.clone(),
                    value: opt_val,
                })
            }

            Stmt::Expr(expr) => {
                let opt_expr = self.optimize_expr(expr);
                Some(Stmt::Expr(opt_expr))
            }

            Stmt::If { cond, then, else_ } => {
                let opt_cond = self.optimize_expr(cond);

                // Constant Folding & Dead Code Elimination for 'if'
                if let Expr::Bool(b) = opt_cond {
                    if b {
                        // Condition is statically true: emit only 'then' branch statements wrapped in block/if
                        let opt_then = self.optimize_stmts(then);
                        return Some(Stmt::If {
                            cond: Expr::Bool(true),
                            then: opt_then,
                            else_: None,
                        });
                    } else {
                        // Condition is statically false: emit only 'else' branch if present
                        if let Some(else_stmt) = else_ {
                            return self.optimize_stmt(else_stmt);
                        } else {
                            return None; // Entire if statement eliminated as dead code
                        }
                    }
                }

                let opt_then = self.optimize_stmts(then);
                let opt_else = else_.as_ref().and_then(|e| self.optimize_stmt(e)).map(Box::new);
                Some(Stmt::If {
                    cond: opt_cond,
                    then: opt_then,
                    else_: opt_else,
                })
            }

            Stmt::While { cond, body } => {
                let opt_cond = self.optimize_expr(cond);
                if let Expr::Bool(false) = opt_cond {
                    return None; // While loop with statically false condition is dead code
                }
                let opt_body = self.optimize_stmts(body);
                Some(Stmt::While {
                    cond: opt_cond,
                    body: opt_body,
                })
            }

            Stmt::Return(expr) => {
                let opt_expr = self.optimize_expr(expr);
                Some(Stmt::Return(opt_expr))
            }

            _ => Some(stmt.clone()),
        }
    }

    fn optimize_expr(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::BinOp { op, left, right } => {
                let opt_left = self.optimize_expr(left);
                let opt_right = self.optimize_expr(right);

                // 1. Constant Folding for Int arithmetic
                if let (Expr::Int(l), Expr::Int(r)) = (&opt_left, &opt_right) {
                    match op {
                        BinOp::Add => return Expr::Int(l + r),
                        BinOp::Sub => return Expr::Int(l - r),
                        BinOp::Mul => return Expr::Int(l * r),
                        BinOp::Div if *r != 0 => return Expr::Int(l / r),
                        BinOp::Mod if *r != 0 => return Expr::Int(l % r),
                        BinOp::Eq => return Expr::Bool(l == r),
                        BinOp::NotEq => return Expr::Bool(l != r),
                        BinOp::Lt => return Expr::Bool(l < r),
                        BinOp::Gt => return Expr::Bool(l > r),
                        BinOp::LtEq => return Expr::Bool(l <= r),
                        BinOp::GtEq => return Expr::Bool(l >= r),
                        _ => {}
                    }
                }

                // Constant Folding for Float arithmetic
                if let (Expr::Float(l), Expr::Float(r)) = (&opt_left, &opt_right) {
                    match op {
                        BinOp::Add => return Expr::Float(l + r),
                        BinOp::Sub => return Expr::Float(l - r),
                        BinOp::Mul => return Expr::Float(l * r),
                        BinOp::Div if *r != 0.0 => return Expr::Float(l / r),
                        BinOp::Eq => return Expr::Bool(l == r),
                        BinOp::NotEq => return Expr::Bool(l != r),
                        BinOp::Lt => return Expr::Bool(l < r),
                        BinOp::Gt => return Expr::Bool(l > r),
                        BinOp::LtEq => return Expr::Bool(l <= r),
                        BinOp::GtEq => return Expr::Bool(l >= r),
                        _ => {}
                    }
                }

                // Constant Folding for String concatenation
                if let (Expr::Str(l), Expr::Str(r)) = (&opt_left, &opt_right) {
                    if let BinOp::Add = op {
                        return Expr::Str(format!("{l}{r}"));
                    }
                }

                // 2. Strength Reduction (e.g. x * 0 -> 0, x * 1 -> x, x + 0 -> x)
                match (op, &opt_left, &opt_right) {
                    (BinOp::Mul, Expr::Int(0), _) | (BinOp::Mul, _, Expr::Int(0)) => return Expr::Int(0),
                    (BinOp::Mul, Expr::Int(1), x) | (BinOp::Mul, x, Expr::Int(1)) => return x.clone(),
                    (BinOp::Add, Expr::Int(0), x) | (BinOp::Add, x, Expr::Int(0)) => return x.clone(),
                    (BinOp::Sub, x, Expr::Int(0)) => return x.clone(),
                    _ => {}
                }

                Expr::BinOp {
                    op: *op,
                    left: Box::new(opt_left),
                    right: Box::new(opt_right),
                }
            }

            Expr::UnaryOp { op, expr } => {
                let opt_sub = self.optimize_expr(expr);

                // Constant Folding for Unary operators
                match (op, &opt_sub) {
                    (UnaryOp::Neg, Expr::Int(n)) => return Expr::Int(-n),
                    (UnaryOp::Neg, Expr::Float(f)) => return Expr::Float(-f),
                    (UnaryOp::Not, Expr::Bool(b)) => return Expr::Bool(!b),
                    // Double negation cancellation: -(-x) -> x, !!x -> x (for bool)
                    (UnaryOp::Neg, Expr::UnaryOp { op: UnaryOp::Neg, expr: inner }) => return *inner.clone(),
                    (UnaryOp::Not, Expr::UnaryOp { op: UnaryOp::Not, expr: inner }) => return *inner.clone(),
                    _ => {}
                }

                Expr::UnaryOp {
                    op: *op,
                    expr: Box::new(opt_sub),
                }
            }

            _ => expr.clone(),
        }
    }
}
