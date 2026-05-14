//! Low-level IR (LIR) for the Kali compiler.
//!
//! LIR is a linearized, codegen-oriented view of MIR. The current Phase-1
//! implementation keeps the lowering deterministic and structurally faithful so
//! later WASM emission can build on a stable node order.

pub use kali_hir::FunctionFlavor;
use kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram};

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
    nodes: Vec<LirNode>,
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

/// LIR lowering result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirProgram {
    pub root: LirNodeId,
    pub nodes: Vec<LirNode>,
}

impl LirProgram {
    /// Validate the structural consistency of the lowered LIR tree.
    pub fn validate(&self) -> Result<(), String> {
        validate_tree(
            "LIR",
            self.root,
            &self.nodes,
            |node| &node.children,
            |id| id.0 as usize,
        )
    }
}

/// LIR lowering.
#[derive(Default)]
pub struct LirLowerer;

impl LirLowerer {
    pub fn new() -> Self {
        Self
    }

    pub fn lower_mir(&self, _mir: MirNodeId) -> LirNodeId {
        LirNodeId::new(0)
    }

    pub fn lower_program(&self, mir: &MirProgram) -> LirProgram {
        let mut builder = LirBuilder::new();
        let root = self.lower_mir_node(&mut builder, &mir.nodes, mir.root);
        LirProgram {
            root,
            nodes: builder.nodes,
        }
    }

    fn lower_mir_node(
        &self,
        builder: &mut LirBuilder,
        nodes: &[MirNode],
        id: MirNodeId,
    ) -> LirNodeId {
        let node = &nodes[id.0 as usize];
        let kind = map_kind(&node.kind);
        let lir_id = match node.text.as_ref() {
            Some(text) => builder.alloc_text(kind, text.clone()),
            None => builder.alloc(kind),
        };
        if let Some(lir_node) = builder.node_mut(lir_id) {
            lir_node.function_flavor = node.function_flavor;
        }
        let mut children = Vec::with_capacity(node.children.len());
        for child in &node.children {
            children.push(self.lower_mir_node(builder, nodes, *child));
        }
        if let Some(lir_node) = builder.node_mut(lir_id) {
            lir_node.children = children;
        }
        lir_id
    }
}

fn map_kind(kind: &MirNodeKind) -> LirNodeKind {
    match kind {
        MirNodeKind::Program => LirNodeKind::Program,
        MirNodeKind::Block => LirNodeKind::Block,
        MirNodeKind::Function => LirNodeKind::Instruction,
        MirNodeKind::Decl => LirNodeKind::Instruction,
        MirNodeKind::Expr => LirNodeKind::Value,
        MirNodeKind::Call => LirNodeKind::Call,
        MirNodeKind::Literal => LirNodeKind::Literal,
        MirNodeKind::ControlFlow => LirNodeKind::Branch,
        MirNodeKind::Unknown => LirNodeKind::Unknown,
    }
}

fn validate_tree<Node, Id>(
    label: &str,
    root: Id,
    nodes: &[Node],
    children: impl Fn(&Node) -> &[Id],
    to_index: impl Fn(Id) -> usize,
) -> Result<(), String>
where
    Id: Copy,
{
    if nodes.is_empty() {
        return Err(format!("{label} tree contains no nodes"));
    }

    let root_index = to_index(root);
    if root_index >= nodes.len() {
        return Err(format!(
            "{label} root node id {root_index} is out of bounds for {} nodes",
            nodes.len()
        ));
    }

    for (index, node) in nodes.iter().enumerate() {
        for child in children(node) {
            let child_index = to_index(*child);
            if child_index >= nodes.len() {
                return Err(format!(
                    "{label} node {index} references child node id {child_index} outside the node table of {} nodes",
                    nodes.len()
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
