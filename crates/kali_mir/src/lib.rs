//! Low-level intermediate representation (LIR) for the Kali compiler.
//! This is the placeholder for Phase 2 MIR implementation.
//!
//! Mid-level IR (MIR) will provide ownership analysis and escape analysis.

use kali_hir::HirNodeId;

/// MIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirNodeKind {
    Program,
    Function,
    Block,
    Decl,
}

/// MIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MirNodeId(pub u32);

impl MirNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// MIR node.
#[derive(Debug, Clone)]
pub struct MirNode {
    pub kind: MirNodeKind,
    pub children: Vec<MirNodeId>,
}

impl MirNode {
    pub fn new(kind: MirNodeKind) -> Self {
        Self {
            kind,
            children: Vec::new(),
        }
    }
}

/// MIR lowering from HIR.
#[derive(Default)]
pub struct MirLowerer;

/// Lower HIR to MIR.
impl MirLowerer {
    pub fn lower_hir(&self, _hir: HirNodeId) -> MirNodeId {
        MirNodeId::new(0)
    }
}
