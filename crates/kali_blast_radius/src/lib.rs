//! Logic for the blast-radius measurement: register parsing, the predicate
//! catalogue, verdict classification, Pareto banding, and the ranking's own
//! document generator.
//!
//! Deliberately a leaf crate with **no kali dependencies**: nothing here
//! compiles or runs a JavaScript program, so an instrument can be validated
//! before it is trusted -- see
//! `docs/superpowers/specs/2026-08-15-blast-radius-ranking-design.md` §10.
//!
//! **The analysis modules -- `catalogue`, `register`, `score`, `verdict` -- are
//! pure**, and are unit-testable against in-memory inputs with no filesystem
//! and no subprocess. Two modules are not, and the claim is scoped rather than
//! quietly broken:
//!
//! - `manifest` reads corpus files to hash and verify them, which is the point
//!   of a freeze check.
//! - `ranking` reads seven repository files -- the register, `counts.json`,
//!   `clusters.json`, `predicates.json`, `accepts.json`,
//!   `anchor-provenance.json` and the corpus README -- **and spawns
//!   `git rev-parse` for the provenance table**. That process spawn is the
//!   genuinely new thing in this crate: nothing else here starts one, and the
//!   sentence this doc comment used to carry ("without running a compiler or a
//!   process") stopped being true of the crate as a whole when it landed. Its
//!   test reads an eighth file, the published ranking, to hold it to the
//!   generator. `ranking` lives in the library rather than in
//!   `examples/rank.rs` so that `ranking_tests` can hold the published document
//!   to the generator; a test binary cannot invoke an example, and a freeze
//!   nothing re-runs is the failure mode this whole project exists to remove.

mod catalogue;
pub use catalogue::{check_completeness, parse_catalogue, CatalogueEntry, Predicate};

mod manifest;
pub use manifest::{
    corpus_hash, parse_manifest, sha256_of, verify_manifest, Manifest, ManifestFile,
};

pub mod ranking;

mod register;
pub use register::{parse_register, RegisterEntry};

mod score;
pub use score::{aggregate, band, dominates, Cluster, ScoredEntry};

mod verdict;
pub use verdict::{
    classify, classify_observing, is_documented_code, runs_agree, ObservedStream, Run, Verdict,
};
