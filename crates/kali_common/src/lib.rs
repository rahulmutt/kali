//! Common utilities shared across all Kali crates.
//!
//! This crate provides:
//! - String interning for identifiers and literals
//! - Source file registry with compact FileId
//! - Span type for source positions
//! - SourceMap for human-readable diagnostics

mod helpers;
pub mod interner;
pub mod js_number;
pub mod numeric_literal;
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
mod display_name;
pub use display_name::{captured_param_spelling, display_names_in};
mod registration;
pub use registration::is_deferred_registration_callee;
mod process_kill;
pub use process_kill::*;
mod object;
pub use object::*;
mod number;
pub use number::*;
mod repr;
pub use repr::*;
mod arena_table;
pub use arena_table::*;
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

/// Growable-runtime-arrays residual round 3 (spec A-44): how many alias
/// hops inference (`repr_infer::is_static_numeric`) and codegen
/// (`static_numeric_chain`) follow when deciding that a value is a
/// compile-time number. One value, so the two cannot disagree on a long
/// chain.
pub const STATIC_NUMERIC_CHAIN_DEPTH: usize = 1024;
