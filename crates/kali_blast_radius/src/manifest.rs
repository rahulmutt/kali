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

pub fn parse_manifest(json: &str) -> Result<Manifest, String> {
    serde_json::from_str(json).map_err(|error| format!("manifest is not valid json: {error}"))
}

/// Every manifest file exists with the recorded hash, and no `.js` file under
/// `root` is missing from the manifest.
///
/// Both directions are required. Checking only the recorded files would let an
/// untracked program be added to the corpus and counted while the frozen hash
/// still verified.
pub fn verify_manifest(root: &Path, manifest: &Manifest) -> Result<(), String> {
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
