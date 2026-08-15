//! Pure logic for the blast-radius measurement: register parsing, the
//! predicate catalogue, verdict classification, and Pareto banding.
//!
//! Deliberately a leaf crate with no kali dependencies. Everything here is
//! unit-testable without running a compiler or a process, which is what lets
//! the instruments be validated before they are trusted -- see
//! `docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md` §10.

mod catalogue;
pub use catalogue::{check_completeness, parse_catalogue, CatalogueEntry, Predicate};

mod manifest;
pub use manifest::{
    corpus_hash, parse_manifest, sha256_of, verify_manifest, Manifest, ManifestFile,
};

mod register;
pub use register::{parse_register, RegisterEntry};

mod verdict;
pub use verdict::{
    classify, classify_observing, is_documented_code, runs_agree, ObservedStream, Run, Verdict,
};
