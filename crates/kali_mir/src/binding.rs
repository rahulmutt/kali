//! MIR binding types and borrowed-lifetime summaries.

use crate::{LayoutDescriptor, OwnershipClass, ThreadBoundaryBinding, ThreadBoundaryDisposition};

/// MIR binding classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirBindingKind {
    Parameter,
    Local,
    Function,
    Import,
}

/// MIR binding metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirBinding {
    pub name: String,
    pub kind: MirBindingKind,
    pub ownership: OwnershipClass,
    pub layout: LayoutDescriptor,
    pub escapes: bool,
    pub captured_by: Vec<String>,
}

/// Deterministic borrowed-lifetime summary for a MIR binding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BorrowedLifetime {
    pub scope: String,
    pub name: String,
    pub captured_by: Vec<String>,
}

impl MirBinding {
    /// Return the canonical thread-boundary disposition for this binding.
    pub fn thread_boundary_disposition(&self) -> ThreadBoundaryDisposition {
        self.ownership.thread_boundary_disposition()
    }

    /// Convert this binding into a canonical thread-boundary profile entry.
    pub fn thread_boundary_binding(&self, scope: impl Into<String>) -> ThreadBoundaryBinding {
        ThreadBoundaryBinding {
            scope: scope.into(),
            name: self.name.clone(),
            disposition: self.thread_boundary_disposition(),
        }
    }

    /// Return the borrowed-lifetime summary for this binding when it is borrowed.
    pub fn borrowed_lifetime(&self, scope: impl Into<String>) -> Option<BorrowedLifetime> {
        matches!(self.ownership, OwnershipClass::Borrowed).then(|| BorrowedLifetime {
            scope: scope.into(),
            name: self.name.clone(),
            captured_by: self.captured_by.clone(),
        })
    }

    /// Whether this binding may cross thread boundaries in the later threaded profile.
    pub fn is_thread_shareable(&self) -> bool {
        self.ownership.is_thread_shareable()
    }

    /// Whether this binding must remain thread-local.
    pub fn is_thread_local(&self) -> bool {
        self.ownership.is_thread_local()
    }

    /// Return the canonical layout/representation fingerprint for this binding.
    pub fn layout_fingerprint(&self) -> String {
        self.layout.fingerprint()
    }

    /// Return the canonical ownership-sensitive representation fingerprint for this binding.
    pub fn representation_fingerprint(&self) -> String {
        format!(
            "ownership={};layout={}",
            self.ownership.fingerprint_tag(),
            self.layout.fingerprint()
        )
    }
}
