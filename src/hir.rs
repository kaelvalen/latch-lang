use crate::ast::BinOp;
use crate::env::Value;

/// High-Level Intermediate Representation (HIR)
/// Resolved, semantically-checked AST representation where identifiers
/// are converted to explicit Local(slot), Global(id), or Upvalue(slot).
#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    Constant(Value),
    Local { slot: usize },
    Global { id: usize },
    Upvalue { slot: usize },
    BinOp {
        op: BinOp,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },
    Call {
        name: String,
        global_id: usize,
        args: Vec<HirExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    LetLocal { slot: usize, value: HirExpr },
    LetGlobal { id: usize, value: HirExpr },
    AssignLocal { slot: usize, value: HirExpr },
    AssignGlobal { id: usize, value: HirExpr },
    Expr(HirExpr),
    If {
        cond: HirExpr,
        then: Vec<HirStmt>,
        else_: Option<Box<HirStmt>>,
    },
    While {
        cond: HirExpr,
        body: Vec<HirStmt>,
    },
    Return(HirExpr),
}
