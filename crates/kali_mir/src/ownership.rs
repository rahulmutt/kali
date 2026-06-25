//! Ownership classes and thread-boundary types for MIR analysis.

use std::collections::BTreeMap;

use crate::MirBinding;

/// Canonical ownership classes used by MIR analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwnershipClass {
    Stack,
    OwnedHeap,
    SharedHeap,
    Borrowed,
}

/// Canonical thread-boundary disposition for a value in the later threaded profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ThreadBoundaryDisposition {
    /// The value must remain local to one runtime instance / thread.
    LocalOnly,
    /// The value is shareable across thread boundaries via shared-heap ownership.
    SharedOnly,
}

/// Canonical thread-boundary profile entry for a MIR binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadBoundaryBinding {
    pub scope: String,
    pub name: String,
    pub disposition: ThreadBoundaryDisposition,
}

/// Canonical thread-boundary profile for a MIR function or whole program.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThreadBoundaryProfile {
    pub bindings: Vec<ThreadBoundaryBinding>,
}

impl ThreadBoundaryProfile {
    pub(crate) fn push_binding(&mut self, scope: impl Into<String>, binding: &MirBinding) {
        self.bindings.push(ThreadBoundaryBinding {
            scope: scope.into(),
            name: binding.name.clone(),
            disposition: binding.thread_boundary_disposition(),
        });
    }

    pub(crate) fn finalize(self) -> Self {
        let mut merged: BTreeMap<(String, String), ThreadBoundaryDisposition> = BTreeMap::new();
        for binding in self.bindings {
            let key = (binding.scope, binding.name);
            merged
                .entry(key)
                .and_modify(|existing| {
                    if matches!(binding.disposition, ThreadBoundaryDisposition::SharedOnly) {
                        *existing = ThreadBoundaryDisposition::SharedOnly;
                    }
                })
                .or_insert(binding.disposition);
        }

        Self {
            bindings: merged
                .into_iter()
                .map(|((scope, name), disposition)| ThreadBoundaryBinding {
                    scope,
                    name,
                    disposition,
                })
                .collect(),
        }
    }

    /// Return a scope-filtered copy of this profile.
    pub fn in_scope(&self, scope: impl AsRef<str>) -> Self {
        let scope = scope.as_ref();
        let bindings = self
            .bindings
            .iter()
            .filter(|binding| binding.scope == scope)
            .cloned()
            .collect();
        Self { bindings }
    }
}

impl OwnershipClass {
    /// Return the canonical thread-boundary disposition for this ownership class.
    pub fn thread_boundary_disposition(self) -> ThreadBoundaryDisposition {
        match self {
            OwnershipClass::SharedHeap => ThreadBoundaryDisposition::SharedOnly,
            OwnershipClass::Stack | OwnershipClass::OwnedHeap | OwnershipClass::Borrowed => {
                ThreadBoundaryDisposition::LocalOnly
            }
        }
    }

    /// Return the canonical fingerprint tag for this ownership class.
    pub fn fingerprint_tag(self) -> &'static str {
        match self {
            OwnershipClass::Stack => "stack",
            OwnershipClass::OwnedHeap => "owned-heap",
            OwnershipClass::SharedHeap => "shared-heap",
            OwnershipClass::Borrowed => "borrowed",
        }
    }

    /// Whether this ownership class may cross thread boundaries in the later threaded profile.
    pub fn is_thread_shareable(self) -> bool {
        matches!(
            self.thread_boundary_disposition(),
            ThreadBoundaryDisposition::SharedOnly
        )
    }

    /// Whether this ownership class must remain thread-local.
    pub fn is_thread_local(self) -> bool {
        !self.is_thread_shareable()
    }
}

#[cfg(test)]
#[path = "ownership_tests.rs"]
mod ownership_tests;
