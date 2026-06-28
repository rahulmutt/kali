//! Low-level IR (LIR) for the Kali compiler.
//!
//! LIR is a linearized, codegen-oriented view of MIR. The current Phase-1
//! implementation keeps the lowering deterministic and structurally faithful so
//! later WASM emission can build on a stable node order.

mod lower;
mod node;
mod program;

pub use kali_hir::FunctionFlavor;
pub use lower::LirLowerer;
pub use node::{LirBuilder, LirNode, LirNodeKind, LirNodeId};
pub use program::LirProgram;

#[cfg(test)]
use kali_mir::MirProgram;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
