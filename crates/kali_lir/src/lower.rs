//! Structural MIR→LIR lowering.

use kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram};

use crate::{LirBuilder, LirNodeId, LirNodeKind, LirProgram};

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
