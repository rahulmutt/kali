//! The frozen corpus manifest.
//!
//! The freeze is what makes the ranking falsifiable. Without it, any desired
//! answer can be produced by adding or removing corpus programs after seeing
//! the scores, and no reader could detect it from the result. The published
//! ranking carries the corpus hash, so it pins exactly what was measured.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestFile {
    /// Corpus-root-relative, e.g. `anchor/nbody.js`.
    pub path: String,
    /// `anchor` or `extension`. Accept rates are reported per stratum and
    /// never pooled: the anchor is passing tests, so a pooled rate would
    /// inherit its ~100% and mean nothing.
    pub stratum: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub corpus_hash: String,
    pub files: Vec<ManifestFile>,
}

pub fn sha256_of(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    // Hex by hand: this `sha2` returns a `hybrid-array::Array`, which does not
    // implement `LowerHex`, so `format!("{:x}", ..)` will not compile. Same
    // idiom as `crates/kali_cli/tests/clbg_binary_trees_runtime.rs`.
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// A hash over the whole file list, independent of listing order so a manifest
/// reordering is not mistaken for a corpus change.
pub fn corpus_hash(files: &[ManifestFile]) -> String {
    let mut lines: Vec<String> = files
        .iter()
        .map(|file| format!("{} {} {}", file.stratum, file.path, file.sha256))
        .collect();
    lines.sort();
    sha256_of(lines.join("\n").as_bytes())
}

/// Structural checks that must hold of any manifest before its `corpus_hash`
/// means anything.
///
/// `corpus_hash` joins `"{stratum} {path} {sha256}"` records with `\n`, with no
/// escaping. That encoding is only injective if no field can contain a space or
/// a newline -- otherwise two different corpora hash alike. A demonstrated
/// collision: `[{anchor, x.js, <h1>}, {anchor, y.js, <h2>}]` digests the same as
/// the single file `{anchor, "x.js <h1>\nanchor y.js", <h2>}`, whose path is a
/// legal filename ending in `.js`, so `collect_js` would find it on disk and
/// `verify_manifest` would pass in both directions. Reaching it takes a
/// deliberately hostile filename -- but the freeze exists so a reader need not
/// trust that someone would have noticed, so the ambiguity is rejected here
/// rather than argued to be unreachable.
fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    // A corpus of nothing must never read as frozen: every rate over it would
    // be 0/0, and every predicate would score identically.
    if manifest.files.is_empty() {
        return Err("manifest lists no files -- an empty corpus is not a freeze".to_string());
    }

    let mut seen: Vec<&str> = Vec::with_capacity(manifest.files.len());
    for file in &manifest.files {
        check_token("path", &file.path)?;
        check_token("stratum", &file.stratum)?;

        if file.sha256.len() != 64
            || !file
                .sha256
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(format!(
                "`{}` has sha256 `{}`, which is not 64 lowercase hex digits",
                file.path, file.sha256
            ));
        }

        // The stratum is not free-form annotation: §4.1 requires accept rates
        // and counts be reported per stratum and never pooled, so a file under
        // `anchor/` labelled `extension` would silently move a program between
        // the two reported populations.
        let segment = file.path.split('/').next().unwrap_or_default();
        if segment != file.stratum || segment == file.path {
            return Err(format!(
                "`{}` is labelled stratum `{}`, which is not its leading path segment",
                file.path, file.stratum
            ));
        }

        if seen.contains(&file.path.as_str()) {
            return Err(format!(
                "`{}` is listed twice -- the counter would count it twice",
                file.path
            ));
        }
        seen.push(&file.path);
    }

    // The freeze token itself. Without this the manifest can carry any
    // `corpus_hash` at all and still verify clean, and the published ranking
    // would stamp that value as the provenance of what it measured.
    let recomputed = corpus_hash(&manifest.files);
    if manifest.corpus_hash != recomputed {
        return Err(format!(
            "recorded corpus_hash {} does not match its own file list ({recomputed})",
            manifest.corpus_hash
        ));
    }
    Ok(())
}

/// No whitespace, no control characters, not empty -- see `validate_manifest`.
fn check_token(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("a manifest entry has an empty `{field}`"));
    }
    if let Some(bad) = value
        .chars()
        .find(|c| c.is_whitespace() || c.is_control() || *c == '\u{fffd}')
    {
        return Err(format!(
            "`{field}` `{value}` contains {bad:?}, which the corpus_hash encoding cannot separate"
        ));
    }
    Ok(())
}

pub fn parse_manifest(json: &str) -> Result<Manifest, String> {
    let manifest: Manifest = serde_json::from_str(json)
        .map_err(|error| format!("manifest is not valid json: {error}"))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

/// Every manifest file exists with the recorded hash, and no `.js` file under
/// `root` is missing from the manifest.
///
/// Both directions are required. Checking only the recorded files would let an
/// untracked program be added to the corpus and counted while the frozen hash
/// still verified.
///
/// `validate_manifest` runs first, so a manifest reaching the on-disk checks is
/// already non-empty, unambiguously encoded, and self-consistent with its own
/// `corpus_hash`.
pub fn verify_manifest(root: &Path, manifest: &Manifest) -> Result<(), String> {
    validate_manifest(manifest)?;

    for file in &manifest.files {
        let full = root.join(&file.path);
        let bytes = std::fs::read(&full).map_err(|error| {
            format!(
                "manifest lists `{}`, which cannot be read: {error}",
                file.path
            )
        })?;
        let actual = sha256_of(&bytes);
        if actual != file.sha256 {
            return Err(format!(
                "`{}` has changed since the freeze: manifest {}, on disk {}",
                file.path, file.sha256, actual
            ));
        }
    }

    let mut found = Vec::new();
    collect_js(root, root, &mut found)?;
    for path in found {
        if !manifest.files.iter().any(|file| file.path == path) {
            return Err(format!(
                "`{path}` is in the corpus directory but not in the manifest -- the corpus was \
                 edited after the freeze"
            ));
        }
    }
    Ok(())
}

fn collect_js(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|error| format!("cannot read {}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_js(root, &path, out)?;
        } else if path.extension().is_some_and(|extension| extension == "js") {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| format!("path escapes corpus root: {error}"))?;
            out.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod manifest_tests;
