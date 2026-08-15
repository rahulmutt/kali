//! Pure logic for the blast-radius measurement: register parsing, the
//! predicate catalogue, verdict classification, and Pareto banding.
//!
//! Deliberately a leaf crate with no kali dependencies. Everything here is
//! unit-testable without running a compiler or a process, which is what lets
//! the instruments be validated before they are trusted -- see
//! `docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md` §10.

mod catalogue;
pub use catalogue::{check_completeness, parse_catalogue, CatalogueEntry, Predicate};

mod register;
pub use register::{parse_register, RegisterEntry};
