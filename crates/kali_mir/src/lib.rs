//! Mid-level IR (MIR) for the Kali compiler.
//!
//! MIR is a conservative structural lowering of HIR that preserves the source
//! shape while providing a stable bridge for later memory/ownership analysis.

mod analysis;
mod binding;
mod function;
mod layout;
mod lower;
mod node;
mod ownership;
mod program;

pub use binding::{BorrowedLifetime, MirBinding, MirBindingKind};
pub use function::{MirFunction, MirFunctionKind};
pub use layout::LayoutDescriptor;
pub use lower::MirLowerer;
pub use node::{MirBuilder, MirNode, MirNodeId, MirNodeKind, PlaceRef, PlaceValue};
pub use ownership::{
    OwnershipClass, ThreadBoundaryBinding, ThreadBoundaryDisposition, ThreadBoundaryProfile,
};
pub use program::MirProgram;

pub(crate) use analysis::{parameter_escape_flags, OwnershipAnalyzer, ScopeState, UseContext};

#[cfg(test)]
mod test_support;
