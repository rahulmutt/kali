//! Diagnostic system for the Kali compiler.
//!
//! This crate provides:
//! - Error code namespaces for different compiler stages
//! - Diagnostic types and severity levels
//! - Non-aborting diagnostic collection

pub mod diagnostic;
pub mod severity;

pub use diagnostic::{
    set_verbose_diagnostics, Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
pub use severity::Severity;

#[doc(hidden)]
pub mod _error_codes;


#[cfg(test)]
#[path = "tests.rs"]
mod tests;
