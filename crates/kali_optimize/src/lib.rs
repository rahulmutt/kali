//! Optimization passes for the Kali compiler.
//!
//! The current implementation focuses on the deterministic, tree-shaped LIR
//! that the rest of the repository already produces. That gives us a safe place
//! to land constant folding, branch elimination, and a handful of algebraic
//! simplifications without needing a full SSA pipeline yet.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

mod profile;

pub use profile::{ProfileData, ProfileSample, ProfileSampleKind, PROFILE_DATA_VERSION};

mod driver;
pub use driver::{OptimizationLevel, OptimizationReport, Optimizer};

mod constant_fold;
pub(crate) use constant_fold::*;

mod specialize;
pub(crate) use specialize::*;

mod inline;
pub(crate) use inline::*;

mod object_fold;
pub(crate) use object_fold::*;

mod layout;
mod helpers;
pub(crate) use helpers::*;

use kali_lir::{LirNode, LirNodeId, LirNodeKind, LirProgram};
use kali_mir::{LayoutDescriptor, MirBindingKind, MirProgram as MirAnalysisProgram};

/// Minimum recorded weight for a function sample to count as hot in the PGO report.
const HOT_FUNCTION_MINIMUM_WEIGHT: u64 = 8;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
