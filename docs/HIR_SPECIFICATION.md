# Latch High-Level Intermediate Representation (HIR) Specification
**Formal Frontend-Backend Boundary Contract & Lowering Transformation Specification**

---

## 1. Executive Summary & Design Principles

The **Latch High-Level Intermediate Representation (HIR)** serves as the canonical boundary between the language frontend (Lexer, Parser, Symbol Resolver, Typechecker) and the runtime backend (HIR Optimizer, Bytecode Emitter, Verifier, VM Engine).

### Core Architectural Invariants:
1. **Unidirectional Layer Discipline**: HIR data structures MUST NEVER import or depend on runtime execution types (`Value`, `Env`, `VM`).
2. **Ident-Free Resolution**: All variable references, function calls, and global slots are lowered to strongly-typed index identifiers (`LocalId`, `GlobalId`, `FunctionId`, `ConstantId`, `UpvalueId`, `ModuleId`).
3. **Pure Frontend Literals**: Primitive literal values are represented as `HirLiteral` (`Int`, `Float`, `Bool`, `Str`, `Null`) to prevent runtime type leaking into lowering.

---

## 2. Compilation Pipeline Architecture

```text
Source Code
    │
 Lexer (Tokens & Spans)
    │
 Parser (AST)
    │
 SymbolTable (String Interning & SymbolId Lookup)
    │
 Resolver (Name Resolution & Shadow Checking)
    │
 HirLowering (AST ──> HirModule Transformation)
    │
 HirVerifier (Static HIR Structural & Index Bound Verification)
    │
 HIR Optimizer (HIR Constant Folding & Branch Pruning)
    │
 Bytecode Compiler (Dumb Bytecode Emitter)
    │
 Bytecode Verifier (CFG & Stack Depth Simulation)
    │
 Virtual Machine (Bytecode Execution)
```

---

## 3. Formal Type Specifications

### 3.1 Strongly-Typed Slot Identifiers
```rust
pub struct LocalId(pub u32);
pub struct GlobalId(pub u32);
pub struct FunctionId(pub u32);
pub struct ConstantId(pub u32);
pub struct UpvalueId(pub u32);
pub struct ModuleId(pub u32);
```

### 3.2 HIR Literal Representations
```rust
pub enum HirLiteral {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Null,
}
```

### 3.3 HIR Expression Graph (`HirExpr`)
```rust
pub enum HirExpr {
    Constant(HirLiteral),
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
    List(Vec<HirExpr>),
    Map(Vec<(HirExpr, HirExpr)>),
    Print(Box<HirExpr>),
}
```

### 3.4 HIR Control-Flow Statement Graph (`HirStmt`)
```rust
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
```

### 3.5 Top-Level Compilation Unit (`HirModule`)
```rust
pub struct HirModule {
    pub name: String,
    pub stmts: Vec<HirStmt>,
    pub exports: Vec<String>,
}
```

---

## 4. AST ──> HIR Lowering Transformation Rules

### 4.1 Variable Binding & Scoping Transformation
- **Local Scope (`scope_depth > 0`)**:
  $$\text{Stmt::Assign}(x, v) \Longrightarrow \text{HirStmt::LetLocal}(\text{LocalId}(n), \text{lower}(v))$$
- **Global Scope (`scope_depth == 0`)**:
  $$\text{Stmt::Assign}(x, v) \Longrightarrow \text{HirStmt::LetGlobal}(\text{GlobalId}(g), \text{lower}(v))$$

### 4.2 Conditional Branching Transformation
$$\text{Stmt::If}(c, t, e) \Longrightarrow \text{HirStmt::If} \{ \text{cond}: \text{lower}(c), \text{then}: \text{lower}(t), \text{else\_}: \text{lower}(e) \}$$

---

## 5. Static Verification & Safety Rules (`HirVerifier`)
1. **Index Upper-Bound Check**: All `LocalId`, `GlobalId`, `FunctionId`, and `UpvalueId` values must not exceed $65,535$ ($2^{16}-1$).
2. **Non-Null Expressions**: All expression sub-trees must evaluate to valid non-dangling HIR nodes.
3. **Empty Branch Protection**: Control-flow blocks must contain valid verified statements.
