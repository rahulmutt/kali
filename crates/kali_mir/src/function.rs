//! MIR function records and per-function summaries.

use std::collections::BTreeSet;

use kali_hir::FunctionFlavor;

use crate::{BorrowedLifetime, MirBinding, ThreadBoundaryProfile};

/// MIR function/module scope metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirFunctionKind {
    Module,
    Function,
    Closure,
}

/// MIR function/module scope analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirFunction {
    pub name: Option<String>,
    pub kind: MirFunctionKind,
    pub function_flavor: Option<FunctionFlavor>,
    pub bindings: Vec<MirBinding>,
}

impl MirFunction {
    pub fn binding(&self, name: &str) -> Option<&MirBinding> {
        self.bindings.iter().find(|binding| binding.name == name)
    }

    /// Return borrowed-lifetime summaries for the borrowed bindings in this scope.
    pub fn borrowed_lifetimes(&self, scope: impl Into<String>) -> Vec<BorrowedLifetime> {
        let scope = scope.into();
        let borrowed = self
            .bindings
            .iter()
            .filter_map(|binding| binding.borrowed_lifetime(scope.clone()))
            .collect::<BTreeSet<_>>();
        borrowed.into_iter().collect()
    }

    /// Return the thread-boundary profile for this function scope.
    pub fn thread_boundary_profile(&self, scope: impl Into<String>) -> ThreadBoundaryProfile {
        let scope = scope.into();
        let mut profile = ThreadBoundaryProfile::default();
        for binding in &self.bindings {
            profile.push_binding(scope.clone(), binding);
        }
        profile.finalize()
    }
}
