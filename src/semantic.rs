use std::collections::HashMap;

use crate::ast::*;
use crate::error::LatchError;

/// Static analysis pass — catches errors before runtime.
pub struct SemanticAnalyzer {
    scopes: Vec<HashMap<String, SymbolInfo>>,
    current_fn: Option<String>,
    errors: Vec<LatchError>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub type_ann: Option<Type>,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable,
    Function { param_count: usize },
}

impl SymbolInfo {
    fn variable() -> Self {
        SymbolInfo { kind: SymbolKind::Variable, type_ann: None }
    }

    fn constant() -> Self {
        SymbolInfo { kind: SymbolKind::Variable, type_ann: None }
    }

    fn function(param_count: usize) -> Self {
        SymbolInfo { kind: SymbolKind::Function { param_count }, type_ann: None }
    }
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            scopes: Vec::new(),
            current_fn: None,
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self, stmts: &[Stmt]) -> Vec<LatchError> {
        self.push_scope();
        self.register_builtins();

        for stmt in stmts {
            self.check_stmt(stmt);
        }

        self.pop_scope();
        self.errors.clone()
    }

    // ── Scope management ─────────────────────────────────────

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, info: SymbolInfo) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), info);
    }

    fn resolve(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn register_builtins(&mut self) {
        // Built-in functions
        self.declare("print", SymbolInfo::function(1));
        self.declare("len", SymbolInfo::function(1));
        self.declare("str", SymbolInfo::function(1));
        self.declare("int", SymbolInfo::function(1));
        self.declare("float", SymbolInfo::function(1));
        self.declare("typeof", SymbolInfo::function(1));
        self.declare("push", SymbolInfo::function(2));
        self.declare("keys", SymbolInfo::function(1));
        self.declare("values", SymbolInfo::function(1));
        self.declare("range", SymbolInfo::function(2));
        self.declare("split", SymbolInfo::function(2));
        self.declare("trim", SymbolInfo::function(1));
        self.declare("lower", SymbolInfo::function(1));
        self.declare("upper", SymbolInfo::function(1));
        self.declare("starts_with", SymbolInfo::function(2));
        self.declare("ends_with", SymbolInfo::function(2));
        self.declare("contains", SymbolInfo::function(2));
        self.declare("replace", SymbolInfo::function(3));
        self.declare("repeat", SymbolInfo::function(2));
        self.declare("assert", SymbolInfo::function(2)); // assert(condition, message)
        self.declare("sum", SymbolInfo::function(1));
        self.declare("max", SymbolInfo::function(1));
        self.declare("min", SymbolInfo::function(1));
        self.declare("sort", SymbolInfo::function(1));
        self.declare("filter", SymbolInfo::function(2));
        self.declare("map", SymbolInfo::function(2));
        self.declare("each", SymbolInfo::function(2));

        // Modules are not functions — they're resolved via ModuleCall,
        // but we register them as variables so `fs` doesn't trigger "undefined".
        self.declare("fs", SymbolInfo::variable());
        self.declare("proc", SymbolInfo::variable());
        self.declare("http", SymbolInfo::variable());
        self.declare("time", SymbolInfo::variable());
        self.declare("ai", SymbolInfo::variable());
        self.declare("json", SymbolInfo::variable());
        self.declare("env", SymbolInfo::variable());
        self.declare("path", SymbolInfo::variable());
    }

    // ── Statement checking ───────────────────────────────────

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value, type_ann } => {
                self.check_expr(value);
                if let Some(ann) = type_ann {
                    self.check_literal_type(name, ann, value);
                }
                self.declare(name, SymbolInfo::variable());
            }

            Stmt::Assign { name, value } => {
                if self.resolve(name).is_none() {
                    self.errors.push(LatchError::UndeclaredAssign(name.clone()));
                }
                self.check_expr(value);
            }

            Stmt::IndexAssign { target, index, value } => {
                self.check_expr(target);
                self.check_expr(index);
                self.check_expr(value);
            }

            Stmt::Fn { name, params, body, .. } => {
                if let Some(info) = self.resolve(name) {
                    if matches!(info.kind, SymbolKind::Function { .. }) {
                        self.errors.push(LatchError::DuplicateFn(name.clone()));
                    }
                }
                self.declare(name, SymbolInfo::function(params.len()));

                self.push_scope();
                let prev = self.current_fn.take();
                self.current_fn = Some(name.clone());
                for p in params {
                    self.declare(&p.name, SymbolInfo::variable());
                }
                for s in body {
                    self.check_stmt(s);
                }
                self.current_fn = prev;
                self.pop_scope();
            }

            Stmt::Return(_) if self.current_fn.is_none() => {
                self.errors.push(LatchError::ReturnOutsideFn);
            }

            Stmt::Return(expr) => {
                self.check_expr(expr);
            }

            Stmt::If { cond, then, else_ } => {
                self.check_expr(cond);
                self.push_scope();
                for s in then { self.check_stmt(s); }
                self.pop_scope();
                if let Some(e) = else_ {
                    self.push_scope();
                    // Handle both elif (If) and else block
                    match &**e {
                        Stmt::If { .. } => self.check_stmt(e),
                        Stmt::Expr(Expr::Fn { body, .. }) => {
                            for s in body { self.check_stmt(s); }
                        }
                        _ => self.check_stmt(e),
                    }
                    self.pop_scope();
                }
            }

            Stmt::For { var, iter, body } => {
                self.check_expr(iter);
                self.push_scope();
                self.declare(var, SymbolInfo::variable());
                for s in body { self.check_stmt(s); }
                self.pop_scope();
            }

            Stmt::Parallel { var, iter, workers, body } => {
                self.check_expr(iter);
                if let Some(w) = workers { self.check_expr(w); }
                self.push_scope();
                self.declare(var, SymbolInfo::variable());
                for s in body { self.check_stmt(s); }
                self.pop_scope();
            }

            Stmt::Try { body, catch_var, catch_body, finally_body } => {
                self.push_scope();
                for s in body { self.check_stmt(s); }
                self.pop_scope();

                self.push_scope();
                self.declare(catch_var, SymbolInfo::variable());
                for s in catch_body { self.check_stmt(s); }
                self.pop_scope();

                if let Some(finally_block) = finally_body {
                    self.push_scope();
                    for s in finally_block { self.check_stmt(s); }
                    self.pop_scope();
                }
            }

            Stmt::Use(path) => {
                // Check if file exists
                if !std::path::Path::new(path).exists() {
                    self.errors.push(LatchError::ImportNotFound(path.clone()));
                }
            }

            Stmt::Stop(expr) => {
                self.check_expr(expr);
            }

            Stmt::Const { name, type_ann, value } => {
                self.check_expr(value);
                if let Some(ann) = type_ann {
                    self.check_literal_type(name, ann, value);
                }
                self.declare(name, SymbolInfo::constant());
            }

            Stmt::Yield(expr) => {
                self.check_expr(expr);
            }

            Stmt::While { cond, body } => {
                self.check_expr(cond);
                self.push_scope();
                for s in body { self.check_stmt(s); }
                self.pop_scope();
            }

            Stmt::Break => {}

            Stmt::Continue => {}

            Stmt::CompoundAssign { name, value, .. } => {
                if self.resolve(name).is_none() {
                    self.errors.push(LatchError::UndeclaredAssign(name.clone()));
                }
                self.check_expr(value);
            }

            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }

            Stmt::Class { name, fields, methods } => {
                self.declare(name, SymbolInfo::variable());
                for (_field_name, type_ann, default) in fields {
                    if let Some(_ann) = type_ann {
                        // Type checking would go here
                    }
                    if let Some(val) = default {
                        self.check_expr(val);
                    }
                }
                for (_method_name, params, body) in methods {
                    self.push_scope();
                    for param in params {
                        self.declare(&param.name, SymbolInfo::variable());
                    }
                    for s in body { self.check_stmt(s); }
                    self.pop_scope();
                }
            }

            Stmt::Export(names) => {
                for name in names {
                    if self.resolve(name).is_none() {
                        self.errors.push(LatchError::UndefinedVariable(name.clone()));
                    }
                }
            }

            Stmt::Import { items, module: _ } => {
                // For now, declare all imported items as variables
                for item in items {
                    self.declare(item, SymbolInfo::variable());
                }
            }
        }
    }

    // ── Expression checking ──────────────────────────────────

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name) => {
                if self.resolve(name).is_none() {
                    self.errors.push(LatchError::UndefinedVariable(name.clone()));
                }
            }

            Expr::Call { name, args, kwargs: _ } => {
                match self.resolve(name) {
                    None => {
                        self.errors.push(LatchError::UndefinedFunction(name.clone()));
                    }
                    Some(SymbolInfo { kind: SymbolKind::Function { param_count }, .. }) => {
                        let pc = *param_count;
                        if args.len() != pc {
                            self.errors.push(LatchError::ArgCountMismatch {
                                name: name.clone(),
                                expected: pc,
                                found: args.len(),
                            });
                        }
                    }
                    _ => {}
                }
                for arg in args { self.check_expr(arg); }
            }

            Expr::ModuleCall { args, .. } => {
                for arg in args { self.check_expr(arg); }
            }

            Expr::BinOp { left, right, .. } => {
                self.check_expr(left);
                self.check_expr(right);
            }

            Expr::UnaryOp { expr, .. } => {
                self.check_expr(expr);
            }

            Expr::OrDefault { expr, default } => {
                self.check_expr(expr);
                self.check_expr(default);
            }

            Expr::Index { expr, index } => {
                self.check_expr(expr);
                self.check_expr(index);
            }

            Expr::FieldAccess { expr, .. } => {
                self.check_expr(expr);
            }

            Expr::SafeAccess { expr, .. } => {
                self.check_expr(expr);
            }

            Expr::NullCoalesce { expr, default } => {
                self.check_expr(expr);
                self.check_expr(default);
            }

            Expr::Range { start, end } => {
                self.check_expr(start);
                self.check_expr(end);
            }

            Expr::Pipe { expr, func } => {
                self.check_expr(expr);
                // Don't check func with normal check_expr because pipe injects
                // an implicit first argument. Check sub-expressions manually.
                self.check_pipe_func(func);
            }

            Expr::List(items) => {
                for item in items { self.check_expr(item); }
            }

            Expr::Map(entries) => {
                for (_, v) in entries { self.check_expr(v); }
            }

            Expr::Fn { params, body } => {
                self.push_scope();
                let prev = self.current_fn.take();
                self.current_fn = Some("<anonymous>".to_string());
                for p in params {
                    self.declare(&p.name, SymbolInfo::variable());
                }
                for s in body {
                    self.check_stmt(s);
                }
                self.current_fn = prev;
                self.pop_scope();
            }

            Expr::Interpolated(parts) => {
                // We don't deep-check interpolation sub-expressions in semantic
                // because they're re-parsed at runtime. Could be improved.
                let _ = parts;
            }

            // Ternary operator: cond ? true_expr : false_expr
            Expr::Ternary { cond, true_branch, false_branch } => {
                self.check_expr(cond);
                self.check_expr(true_branch);
                self.check_expr(false_branch);
            }

            // List comprehension: [body for var in iter if cond]
            Expr::ListComp { body, var, iter, cond } => {
                self.check_expr(iter);
                self.push_scope();
                self.declare(var, SymbolInfo::variable());
                self.check_expr(body);
                if let Some(c) = cond {
                    self.check_expr(c);
                }
                self.pop_scope();
            }

            // Slice: list[1:5], list[2:], list[:-1]
            Expr::Slice { expr, start, end } => {
                self.check_expr(expr);
                if let Some(s) = start { self.check_expr(s); }
                if let Some(e) = end { self.check_expr(e); }
            }

            // Literals — no checks needed
            Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Str(_) | Expr::Null => {}
        }
    }

    fn check_literal_type(&mut self, name: &str, ann: &Type, value: &Expr) {
        let found = match value {
            Expr::Int(_) => Some(Type::Int),
            Expr::Float(_) => Some(Type::Float),
            Expr::Bool(_) => Some(Type::Bool),
            Expr::Str(_) | Expr::Interpolated(_) => Some(Type::Str),
            Expr::List(_) => Some(Type::List),
            Expr::Map(_) => Some(Type::Dict),
            _ => None, // can't determine at compile time
        };
        if let Some(found) = found {
            if ann != &found && ann != &Type::Any {
                self.errors.push(LatchError::TypeAnnotationMismatch {
                    name: name.to_string(),
                    expected: ann.clone(),
                    found,
                });
            }
        }
    }

    /// Check a pipe‐target expression, accounting for the implicit first argument.
    fn check_pipe_func(&mut self, func: &Expr) {
        match func {
            Expr::Call { name, args, kwargs: _ } => {
                // Pipe adds one implicit arg, so check arity with +1
                if let Some(SymbolInfo { kind: SymbolKind::Function { param_count }, .. }) = self.resolve(name) {
                    let pc = *param_count;
                    if args.len() + 1 != pc {
                        self.errors.push(LatchError::ArgCountMismatch {
                            name: name.clone(),
                            expected: pc,
                            found: args.len() + 1,
                        });
                    }
                }
                for arg in args { self.check_expr(arg); }
            }
            Expr::ModuleCall { args, .. } => {
                for arg in args { self.check_expr(arg); }
            }
            // `expr |> func() or default` — the OrDefault wraps the call
            Expr::OrDefault { expr: inner, default } => {
                self.check_pipe_func(inner);
                self.check_expr(default);
            }
            _ => self.check_expr(func),
        }
    }
}
