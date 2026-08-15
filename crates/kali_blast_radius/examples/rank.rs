//! Prints sections 2-5 of `docs/superpowers/followups/blast-radius-ranking.md`,
//! preceded by the provenance table that goes in its §1.4.
//!
//! The generation itself is `kali_blast_radius::ranking::render`, in the library
//! rather than here, so that a test can hold the committed document to it. This
//! binary is the way a human re-runs it:
//!
//! ```text
//! cargo run -p kali_blast_radius --example rank
//! ```
//!
//! The output is spliced into the document between its two HTML-comment
//! markers. `ranking_tests::spliced_document_matches_the_generator` fails if the
//! two ever drift.

use kali_blast_radius::ranking::{render, repo_root};

fn main() {
    print!("{}", render(&repo_root()));
}
