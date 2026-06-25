//! Arena-style AST storage: the `ASTBuilder` and finalized `AST`.

use crate::{Node, NodeId, NodeKind};
use kali_common::Span;

#[cfg(test)]
#[path = "builder_tests.rs"]
mod builder_tests;

/// An AST builder. After parsing, nodes are allocated in arena-style storage.
pub struct ASTBuilder {
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

impl ASTBuilder {
    /// Create a new AST builder.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }

    /// Create a new node with an auto-assigned ID.
    pub fn new_node(&mut self, kind: NodeKind, span: Option<Span>) -> NodeId {
        let id = NodeId::new(self.nodes.len() as u32);
        self.nodes.push(Node::new(id, kind, span));
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

    /// Finalize the AST and consume the builder.
    pub fn into_ast(self) -> AST {
        AST {
            nodes: self.nodes,
            root: self.root,
        }
    }
}

impl Default for ASTBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed AST with source-level tree structure.
pub struct AST {
    nodes: Vec<Node>,
    root: Option<NodeId>,
}

impl AST {
    /// Create a new AST from nodes and root.
    pub fn new(nodes: Vec<Node>, root: Option<NodeId>) -> Self {
        Self { nodes, root }
    }

    /// Get a reference to a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.as_u32() as usize)
    }

    /// Get a mutable reference to a node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.as_u32() as usize)
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

    /// Create an empty AST.
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
        }
    }
}

impl Default for AST {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::convert::From<ASTBuilder> for AST {
    fn from(builder: ASTBuilder) -> Self {
        builder.into_ast()
    }
}
