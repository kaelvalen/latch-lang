use crate::ast::*;
use crate::error::Result;
use crate::hir::*;
use crate::resolver::Resolver;

/// Pure HIR Lowering Transformation Phase
/// Transforms Resolved AST constructs into standalone HirModule representation.
pub struct HirLowering {
    resolver: Resolver,
}

impl Default for HirLowering {
    fn default() -> Self {
        Self::new()
    }
}

impl HirLowering {
    pub fn new() -> Self {
        HirLowering {
            resolver: Resolver::new(),
        }
    }

    pub fn lower_module(&mut self, name: impl Into<String>, stmts: &[Stmt]) -> Result<HirModule> {
        self.resolver.resolve_module(name, stmts)
    }
}
