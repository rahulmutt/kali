//! LIR node kinds, ids, and the arena builder.

use kali_hir::FunctionFlavor;

/// LIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LirNodeKind {
    Program,
    Block,
    Instruction,
    Value,
    Branch,
    Call,
    Literal,
    Unknown,
}

/// LIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LirNodeId(pub u32);

impl LirNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// LIR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirNode {
    pub kind: LirNodeKind,
    pub text: Option<String>,
    pub children: Vec<LirNodeId>,
    pub function_flavor: Option<FunctionFlavor>,
}

impl LirNode {
    pub fn new(kind: LirNodeKind) -> Self {
        Self {
            kind,
            text: None,
            children: Vec::new(),
            function_flavor: None,
        }
    }

    pub fn with_text(kind: LirNodeKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: Some(text.into()),
            children: Vec::new(),
            function_flavor: None,
        }
    }
}

/// LIR builder.
#[derive(Default)]
pub struct LirBuilder {
    pub(crate) nodes: Vec<LirNode>,
}

impl LirBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, kind: LirNodeKind) -> LirNodeId {
        let id = LirNodeId(self.nodes.len() as u32);
        self.nodes.push(LirNode::new(kind));
        id
    }

    pub fn alloc_text(&mut self, kind: LirNodeKind, text: impl Into<String>) -> LirNodeId {
        let id = LirNodeId(self.nodes.len() as u32);
        self.nodes.push(LirNode::with_text(kind, text));
        id
    }

    pub fn node_mut(&mut self, id: LirNodeId) -> Option<&mut LirNode> {
        self.nodes.get_mut(id.0 as usize)
    }

    pub fn into_nodes(self) -> Vec<LirNode> {
        self.nodes
    }
}
