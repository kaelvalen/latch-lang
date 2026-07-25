use crate::env::Value;

/// Strongly-Typed Index Identifiers for Compile-Time Type Safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpvalueId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub u32);

/// Independent HIR Binary Operators (Zero AST Dependencies)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// Standalone High-Level Intermediate Representation (HIR)
/// Pure resolved instructions operating on ID slots — zero strings or AST nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum HirExpr {
    Constant(Value),
    Local(LocalId),
    Global(GlobalId),
    Upvalue(UpvalueId),
    BinOp {
        op: HirOp,
        left: Box<HirExpr>,
        right: Box<HirExpr>,
    },
    Call {
        func_id: FunctionId,
        args: Vec<HirExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    LetLocal { id: LocalId, value: HirExpr },
    LetGlobal { id: GlobalId, value: HirExpr },
    AssignLocal { id: LocalId, value: HirExpr },
    AssignGlobal { id: GlobalId, value: HirExpr },
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
