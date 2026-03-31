//! AST definitions for TypeScript/JavaScript.
//!
//! This crate defines the Abstract Syntax Tree node types.

use kali_common::Span;

/// A node identifier for AST nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "n{}", self.0)
    }
}

/// An AST node.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// The node's kind/type.
    pub kind: NodeKind,
    /// Source span for this node.
    pub span: Option<Span>,
    /// Child nodes (by NodeId).
    pub children: Vec<NodeId>,
}

impl Node {
    /// Create a new AST node.
    pub fn new(id: NodeId, kind: NodeKind, span: Option<Span>) -> Self {
        Self {
            id,
            kind,
            span,
            children: Vec::new(),
        }
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: NodeId) {
        self.children.push(child);
    }
}

/// Types of AST nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    // Program structure
    Program,
    Block,
    ExpressionStatement,
    EmptyStatement,

    // Declarations
    VariableDeclaration,
    FunctionDeclaration,
    FunctionExpression,
    ClassDeclaration,
    ClassExpression,
    ImportDeclaration,
    ExportDeclaration,

    // Expressions
    Identifier,
    Literal,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ThisExpression,

    // Statements
    IfStatement,
    ForStatement,
    WhileStatement,
    DoWhileStatement,
    SwitchStatement,
    WithStatement,
    BreakStatement,
    ContinueStatement,

    // TypeScript-specific
    TSTypeAnnotation,
    TSInterfaceDeclaration,
    TSTypeAliasDeclaration,
    TSInterfaceHeritage,
    TSPropertySignature,
}

impl NodeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeKind::Program => "Program",
            NodeKind::Block => "Block",
            NodeKind::ExpressionStatement => "ExpressionStatement",
            NodeKind::EmptyStatement => "EmptyStatement",
            NodeKind::VariableDeclaration => "VariableDeclaration",
            NodeKind::FunctionDeclaration => "FunctionDeclaration",
            NodeKind::FunctionExpression => "FunctionExpression",
            NodeKind::ClassDeclaration => "ClassDeclaration",
            NodeKind::ClassExpression => "ClassExpression",
            NodeKind::ImportDeclaration => "ImportDeclaration",
            NodeKind::ExportDeclaration => "ExportDeclaration",
            NodeKind::Identifier => "Identifier",
            NodeKind::Literal => "Literal",
            NodeKind::BinaryExpression => "BinaryExpression",
            NodeKind::UnaryExpression => "UnaryExpression",
            NodeKind::CallExpression => "CallExpression",
            NodeKind::MemberExpression => "MemberExpression",
            NodeKind::ThisExpression => "ThisExpression",
            NodeKind::IfStatement => "IfStatement",
            NodeKind::ForStatement => "ForStatement",
            NodeKind::WhileStatement => "WhileStatement",
            NodeKind::DoWhileStatement => "DoWhileStatement",
            NodeKind::SwitchStatement => "SwitchStatement",
            NodeKind::WithStatement => "WithStatement",
            NodeKind::BreakStatement => "BreakStatement",
            NodeKind::ContinueStatement => "ContinueStatement",
            NodeKind::TSTypeAnnotation => "TSTypeAnnotation",
            NodeKind::TSInterfaceDeclaration => "TSInterfaceDeclaration",
            NodeKind::TSTypeAliasDeclaration => "TSTypeAliasDeclaration",
            NodeKind::TSInterfaceHeritage => "TSInterfaceHeritage",
            NodeKind::TSPropertySignature => "TSPropertySignature",
        }
    }
}

/// An AST builder.
pub struct AST {
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

impl AST {
    /// Create a new AST.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }

    /// Create a new node with an auto-assigned ID.
    pub fn new_node(&mut self, kind: NodeKind, span: Option<Span>) -> NodeId {
        let id = NodeId::new(self.nodes.len() as u32);
        let mut node = Node::new(id, kind, span);
        self.nodes.push(node);
        id
    }

    /// Get a reference to a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.as_u32() as usize)
    }

    /// Get a mutable reference to a node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.as_u32() as usize)
    }

    /// Set the root of the AST.
    pub fn set_root(&mut self, root: NodeId) {
        self.root = Some(root);
    }

    /// Get the root of the AST.
    pub fn root(&self) -> Option<&NodeId> {
        self.root.as_ref()
    }

    /// Get all nodes in the AST.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Get mutable access to all nodes.
    pub fn nodes_mut(&mut self) -> &mut [Node] {
        &mut self.nodes
    }
}

impl Default for AST {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(42);
        assert_eq!(id.as_u32(), 42);
        assert_eq!(id.to_string(), "n42");
    }

    #[test]
    fn test_ast_creation() {
        let mut ast = AST::new();
        let root_id = ast.new_node(NodeKind::Program, None);
        ast.set_root(root_id);

        let root = ast.get_node(root_id).unwrap();
        assert_eq!(root.kind, NodeKind::Program);
        
        assert!(ast.root().is_some());
    }
}
