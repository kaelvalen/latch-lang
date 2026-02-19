#![allow(dead_code)]
/// AST node types for the Latch language.

// ── String interpolation parts ───────────────────────────────
#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(String),
    Expr(Vec<crate::lexer::Spanned<crate::lexer::Token>>), // tokens inside ${}
}

// ── Expressions — anything that produces a value ─────────────
#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Null,
    List(Vec<Expr>),
    Map(Vec<(String, Expr)>),

    Ident(String),

    Interpolated(Vec<StringPart>),

    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },

    Call {
        name: String,
        args: Vec<Expr>,
    },

    ModuleCall {
        module: String,
        method: String,
        args: Vec<Expr>,
    },

    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
    },

    /// `expr or default`
    OrDefault {
        expr: Box<Expr>,
        default: Box<Expr>,
    },

    /// Field access like `result.stdout`
    FieldAccess {
        expr: Box<Expr>,
        field: String,
    },

    /// Anonymous function: `fn(x) { return x + 1 }`
    Fn {
        params: Vec<Param>,
        body: Block,
    },

    /// Null coalesce: `expr ?? default`
    NullCoalesce {
        expr: Box<Expr>,
        default: Box<Expr>,
    },

    /// Range: `1..10`
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },

    /// Pipe: `expr |> func()`
    Pipe {
        expr: Box<Expr>,
        func: Box<Expr>,
    },

    /// Safe field access: `expr?.field`
    SafeAccess {
        expr: Box<Expr>,
        field: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
    In,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

// ── Statements — side-effect producing constructs ────────────
#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        type_ann: Option<Type>,
        value: Expr,
    },

    Assign {
        name: String,
        value: Expr,
    },

    /// Index assignment: `list[0] = 5`, `map["key"] = val`, or `cfg["db"]["port"] = 4000`
    IndexAssign {
        target: Expr,
        index: Expr,
        value: Expr,
    },

    If {
        cond: Expr,
        then: Block,
        else_: Option<Block>,
    },

    For {
        var: String,
        iter: Expr,
        body: Block,
    },

    Parallel {
        var: String,
        iter: Expr,
        workers: Option<Expr>,
        body: Block,
    },

    Fn {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Block,
    },

    Return(Expr),

    Try {
        body: Block,
        catch_var: String,
        catch_body: Block,
    },

    Use(String),

    /// `stop 1` — exit the script with a code
    Stop(Expr),

    /// Compound assignment: `x += 1`, `x -= 2`, etc.
    CompoundAssign {
        name: String,
        op: BinOp,
        value: Expr,
    },

    /// A bare expression used as a statement: `print("hi")`
    Expr(Expr),
}

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    List,
    Dict,
    Process,
    File,
    Any,
}
