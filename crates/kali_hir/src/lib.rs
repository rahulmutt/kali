//! High-level intermediate representation (HIR) for the Kali compiler.
//!
//! This crate provides the HIR lowering from AST.

use kali_ast::NodeId;
use kali_common::Span;

/// HIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirNodeKind {
    // Program structure
    Program,
    FunctionDecl,
    ClassDecl,
    VarDecl,
    
    // Expressions
    Ident,
    Literal,
    BinaryExpr,
    CallExpr,
    MemberExpr,
    
    // Statements
    ExprStmt,
    IfStmt,
    ForStmt,
    WhileStmt,
}

/// An HIR node.
#[derive(Debug, Clone)]
pub struct HirNode {
    /// Node kind.
    pub kind: HirNodeKind,
    /// Source span.
    pub span: Option<Span>,
    /// Children by index.
    pub children: Vec<HirNodeId>,
}

/// HIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HirNodeId(pub u32);

impl HirNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl HirNode {
    pub fn new(kind: HirNodeKind, span: Option<Span>) -> Self {
        Self {
            kind,
            span,
            children: Vec::new(),
        }
    }
}

/// HIR builder.
pub struct HirBuilder {
    nodes: Vec<HirNode>,
    next_id: HirNodeId,
}

impl HirBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: HirNodeId::new(0),
        }
    }

    pub fn alloc(&mut self, kind: HirNodeKind, span: Option<Span>) -> HirNodeId {
        let id = self.next_id;
        self.next_id.0 += 1;
        self.nodes.push(HirNode::new(kind, span));
        id
    }
}

impl Default for HirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Lowering result from AST to HIR.
pub struct LoweringResult {
    /// Root node of the HIR.
    pub root: HirNodeId,
    /// All HIR nodes.
    pub nodes: Vec<HirNode>,
    /// Diagnostics.
    pub diagnostics: Vec<kali_error::diagnostic::Diagnostic>,
}

/// HIR lowering from AST.
pub struct HirLowerer;

impl HirLowerer {
    /// Lower an AST node to HIR.
    pub fn lower(&self, _node_id: NodeId) -> HirNodeId {
        HirNodeId::new(0)
    }

    /// Lower a complete program.
    pub fn lower_program(&self, _program_root: NodeId) -> HirNode {
        HirNode::new(HirNodeKind::Program, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hir_builder() {
        let mut builder = HirBuilder::new();
        
        let root = builder.alloc(HirNodeKind::Program, None);
        assert_eq!(root.0, 0);
        
        assert_eq!(builder.next_id.0, 1);
    }
}
