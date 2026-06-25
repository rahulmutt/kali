//! MIR node kinds, ids, place references, and the arena builder.

use kali_hir::FunctionFlavor;

/// MIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirNodeKind {
    Program,
    Block,
    Function,
    Decl,
    Expr,
    Call,
    Literal,
    ControlFlow,
    Unknown,
}

/// MIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MirNodeId(pub u32);

impl MirNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// MIR place reference (an addressable location).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlaceRef(pub MirNodeId);

impl PlaceRef {
    pub fn new(id: MirNodeId) -> Self {
        Self(id)
    }
}

/// MIR place value (the loaded value from a place).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlaceValue(pub MirNodeId);

impl PlaceValue {
    pub fn new(id: MirNodeId) -> Self {
        Self(id)
    }
}

/// MIR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirNode {
    pub kind: MirNodeKind,
    pub text: Option<String>,
    pub children: Vec<MirNodeId>,
    pub function_flavor: Option<FunctionFlavor>,
}

impl MirNode {
    pub fn new(kind: MirNodeKind) -> Self {
        Self {
            kind,
            text: None,
            children: Vec::new(),
            function_flavor: None,
        }
    }

    pub fn with_text(kind: MirNodeKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: Some(text.into()),
            children: Vec::new(),
            function_flavor: None,
        }
    }
}

/// MIR builder.
#[derive(Default)]
pub struct MirBuilder {
    pub(crate) nodes: Vec<MirNode>,
}

impl MirBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, kind: MirNodeKind) -> MirNodeId {
        let id = MirNodeId(self.nodes.len() as u32);
        self.nodes.push(MirNode::new(kind));
        id
    }

    pub fn alloc_text(&mut self, kind: MirNodeKind, text: impl Into<String>) -> MirNodeId {
        let id = MirNodeId(self.nodes.len() as u32);
        self.nodes.push(MirNode::with_text(kind, text));
        id
    }

    pub fn node_mut(&mut self, id: MirNodeId) -> Option<&mut MirNode> {
        self.nodes.get_mut(id.0 as usize)
    }
}
