//! High-level intermediate representation (HIR) for the Kali compiler.
//!
//! This crate provides a deterministic AST-to-HIR lowering layer used by the
//! later MIR/LIR stages. The implementation is intentionally conservative and
//! source-shaped so the phase-1 pipeline can round-trip representative programs
//! without inventing extra semantics.

mod builder;
mod helpers;
mod lowering;
mod node;
mod result;

pub use builder::HirBuilder;
pub use lowering::HirLowerer;
pub use node::{HirNode, HirNodeId, HirNodeKind};
pub use result::{FunctionFlavor, LoweringResult};

#[cfg(test)]
mod test_support;
