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
        kwargs: Vec<(String, Expr)>, // Keyword arguments
    },

    ModuleCall {
        module: String,
        method: String,
        args: Vec<Expr>,
    },

    /// Method call on an arbitrary expression: `expr.method(args)`
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
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

    /// List comprehension: `[x*2 for x in list]` or `[x for x in list if x > 0]`
    ListComp {
        body: Box<Expr>,         // The expression to generate (e.g., x*2)
        var: String,             // Loop variable (e.g., x)
        iter: Box<Expr>,         // Iterable (e.g., list)
        cond: Option<Box<Expr>>, // Optional condition (e.g., x > 0)
    },

    /// Safe field access: `expr?.field`
    SafeAccess {
        expr: Box<Expr>,
        field: String,
    },

    /// Ternary operator: `cond ? true_expr : false_expr`
    Ternary {
        cond: Box<Expr>,
        true_branch: Box<Expr>,
        false_branch: Box<Expr>,
    },

    /// Slice: `list[1:5]`, `list[2:]`, `list[:-1]`
    Slice {
        expr: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
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
        else_: Option<Box<Stmt>>, // Box<Stmt::If> for elif, Box<Stmt::Block> for else
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
        finally_body: Option<Block>,
    },

    /// Constant declaration: `const PI = 3.14`
    Const {
        name: String,
        type_ann: Option<Type>,
        value: Expr,
    },

    /// `while condition { body }`
    While {
        cond: Expr,
        body: Block,
    },

    /// `break` — exit the innermost loop
    Break,

    /// `continue` — skip to the next iteration of the innermost loop
    Continue,

    /// Compound assignment: `x += 1`, `x -= 2`, etc.
    CompoundAssign {
        name: String,
        op: BinOp,
        value: Expr,
    },

    /// A bare expression used as a statement: `print("hi")`
    Expr(Expr),

    /// Class declaration: `class Point { x: int, y: int, fn move() { ... } }`
    Class {
        name: String,
        fields: Vec<(String, Option<Type>, Option<Expr>)>, // name, type, default
        methods: Vec<(String, Vec<Param>, Block)>,         // name, params, body
    },

    /// Export statement: `export { foo, bar }` or `export foo`
    Export(Vec<String>),

    /// Import statement: `import { foo, bar } from "module"`
    Import {
        items: Vec<String>,
        module: String,
    },

    /// Field assignment: `obj.field = value` (used for `self.x = val` in methods)
    FieldAssign {
        object: Expr,
        field: String,
        value: Expr,
    },

    /// Match statement: `match expr { case val: { stmts } default: { stmts } }`
    Match {
        expr: Expr,
        cases: Vec<(Expr, Block)>, // (pattern expr, body)
        default: Option<Block>,
    },
}

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<Type>,
    pub default: Option<Expr>, // Default value for optional parameter
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

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Str => write!(f, "string"),
            Type::List => write!(f, "list"),
            Type::Dict => write!(f, "dict"),
            Type::Process => write!(f, "process"),
            Type::File => write!(f, "file"),
            Type::Any => write!(f, "any"),
        }
    }
}
