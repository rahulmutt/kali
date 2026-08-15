//! The drift gate on the published ranking.
//!
//! Sections 2-5 of `docs/superpowers/followups/blast-radius-ranking.md` are the
//! generator's stdout, spliced between two HTML-comment markers. Until this
//! test existed, "no figure there was typed by hand" was a claim in the
//! document's own prose — the one freeze in this plan that was documentary
//! rather than mechanical. It is now a red test.

use super::*;

/// The one line that legitimately differs between a fresh run and the committed
/// document: generation necessarily runs at the PARENT of the commit that
/// records its output, so the recorded HEAD is always one behind. Everything
/// else must match byte for byte.
const HEAD_ROW_PREFIX: &str = "| this document generated at |";

fn between<'a>(document: &'a str, marker: &str) -> &'a str {
    let begin = format!("<!-- {marker}:BEGIN");
    let end = format!("<!-- {marker}:END -->");
    let start = document
        .find(&begin)
        .unwrap_or_else(|| panic!("the ranking has no `{begin}` marker"));
    let start = start
        + document[start..]
            .find("-->")
            .expect("the BEGIN marker is unterminated")
        + "-->".len();
    let stop = document
        .find(&end)
        .unwrap_or_else(|| panic!("the ranking has no `{end}` marker"));
    assert!(
        start < stop,
        "`{marker}`'s markers are in the wrong order in the ranking"
    );
    document[start..stop].trim_matches('\n')
}

/// Blank the HEAD cell so the comparison is about the numbers, not about which
/// commit happened to be checked out.
fn without_head_row(text: &str) -> String {
    text.lines()
        .map(|line| {
            if line.starts_with(HEAD_ROW_PREFIX) {
                HEAD_ROW_PREFIX
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn spliced_document_matches_the_generator() {
    let root = repo_root();
    let document = std::fs::read_to_string(
        root.join("docs/superpowers/followups/blast-radius-ranking.md"),
    )
    .expect("the ranking document is readable");

    let generated = render(&root);
    let (provenance, body) = generated
        .split_once("## 2. The bands")
        .expect("the generator emits a provenance table above `## 2. The bands`");
    let body = format!("## 2. The bands{body}");

    assert_eq!(
        without_head_row(between(&document, "GENERATED-PROVENANCE")),
        without_head_row(provenance.trim_matches('\n')),
        "the ranking's §1.4 provenance table has drifted from the generator -- re-splice it \
         (`cargo run -p kali_blast_radius --example rank`) rather than editing the document"
    );
    assert_eq!(
        without_head_row(between(&document, "GENERATED")),
        without_head_row(body.trim_matches('\n')),
        "the ranking's §2-§5 have drifted from the generator. Either an input moved and the \
         document was not re-spliced, or the document was edited by hand inside the generated \
         region. Re-run `cargo run -p kali_blast_radius --example rank` and splice its stdout \
         between the markers; do not edit the region directly."
    );
}

/// The gate is only worth having if it can fail. A hand-edit inside the region
/// must be caught, so the comparison is checked to be sensitive to one.
#[test]
fn the_drift_gate_would_catch_a_hand_edit() {
    let generated = render(&repo_root());
    let tampered = generated.replacen("| G3 ", "| G3* ", 1);
    assert_ne!(
        generated, tampered,
        "the tamper probe did not change anything -- this test proves nothing as written"
    );
    assert_ne!(
        without_head_row(&generated),
        without_head_row(&tampered),
        "normalisation swallowed a real edit"
    );
}
