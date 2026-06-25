//! Arena builder that allocates HIR nodes by id.

use crate::node::{HirNode, HirNodeId, HirNodeKind};
use kali_common::Span;

/// HIR builder.
pub struct HirBuilder {
    pub(crate) nodes: Vec<HirNode>,
    pub(crate) next_id: HirNodeId,
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

    pub fn alloc_text(
        &mut self,
        kind: HirNodeKind,
        span: Option<Span>,
        text: impl Into<String>,
    ) -> HirNodeId {
        let id = self.next_id;
        self.next_id.0 += 1;
        self.nodes.push(HirNode::with_text(kind, span, text));
        id
    }

    pub fn node_mut(&mut self, id: HirNodeId) -> Option<&mut HirNode> {
        self.nodes.get_mut(id.0 as usize)
    }
}

impl Default for HirBuilder {
    fn default() -> Self {
        Self::new()
    }
}
