//! Common utilities shared across all Kali crates.
//!
//! This crate provides:
//! - String interning for identifiers and literals
//! - Source file registry with compact FileId
//! - Span type for source positions
//! - SourceMap for human-readable diagnostics

mod helpers;
pub mod interner;
pub mod source_map;
pub mod span;
pub mod template;

pub(crate) use helpers::*;
pub use interner::{InternedString, Interner};
pub use span::Span;
mod registry;
pub use registry::*;
mod messages;
pub use messages::*;
mod process_kill;
pub use process_kill::*;
mod object;
pub use object::*;
mod number;
pub use number::*;
mod repr;
pub use repr::*;
mod math;
pub use math::*;
mod promise;
pub use promise::*;
mod array;
pub use array::*;
mod template_literal;
pub use template_literal::*;
mod collections;
pub use collections::*;
mod late;
pub use late::*;
mod intl;
pub use intl::*;
