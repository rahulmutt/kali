//! Embedding interfaces for Kali.
//!
//! This crate provides a stable embedding-oriented API for consumers that want
//! to compile Kali source in-process without going through the CLI.

use kali_cli::build;
pub use kali_sandbox::{
    HostOperation, HostPredicate, PolicyPredicateContext, PolicyPredicateRegistry, SandboxPolicy,
};

mod error;
mod artifact;
mod compiler;
mod context;

pub use artifact::{CompiledArtifact, LibArtifact};
pub use compiler::{CompilerConfig, KaliCompiler};
pub use context::{EmbeddingCtx, OperationContext, PredicateDecision};
pub use error::CompileError;

pub use build::LibraryExport;
pub use kali_cli::build::ArtifactMetadata;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
