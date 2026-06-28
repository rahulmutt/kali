//! Low-level IR (LIR) for the Kali compiler.
//!
//! LIR is a linearized, codegen-oriented view of MIR. The current Phase-1
//! implementation keeps the lowering deterministic and structurally faithful so
//! later WASM emission can build on a stable node order.

pub use kali_hir::FunctionFlavor;
use kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram};

mod node;
pub use node::{LirBuilder, LirNode, LirNodeKind, LirNodeId};

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
