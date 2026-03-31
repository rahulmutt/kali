//! Low-level intermediate representation (LIR) for the Kali compiler.
//!
//! LIR is a linearized form of MIR ready for code generation.

use kali_mir::MirNodeId;

/// LIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LirNodeKind {
    Program,
    Instruction,
    Block,
    Phi,
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
#[derive(Debug, Clone)]
pub struct LirNode {
    pub kind: LirNodeKind,
    pub children: Vec<LirNodeId>,
}

/// LIR lowering.
#[derive(Default)]
pub struct LirLowerer;

impl LirLowerer {
    pub fn lower_mir(&self, _mir: MirNodeId) -> LirNodeId {
        LirNodeId::new(0)
    }
}
