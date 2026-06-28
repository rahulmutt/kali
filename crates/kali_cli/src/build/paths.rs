//! Output path computation for build artifacts.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use crate::BundleFormat;
use super::source_stem;

pub fn executable_output_path_for(source_path: &Path, out_dir: Option<&Path>) -> PathBuf {
    let stem = source_stem(source_path);
    let file_name = format!("{}.wasm", stem);
    match out_dir {
        Some(dir) => dir.join(file_name),
        None => source_path.with_file_name(file_name),
    }
}

pub fn library_output_paths_for(
    source_path: &Path,
    out_dir: Option<&Path>,
) -> (PathBuf, PathBuf, PathBuf) {
    let stem = source_stem(source_path);
    let wasm_name = format!("{}.lib.wasm", stem);
    let wit_name = format!("{}.lib.wit", stem);
    let meta_name = format!("{}.lib.meta.json", stem);
    match out_dir {
        Some(dir) => (
            dir.join(&wasm_name),
            dir.join(&wit_name),
            dir.join(&meta_name),
        ),
        None => (
            source_path.with_file_name(wasm_name),
            source_path.with_file_name(wit_name),
            source_path.with_file_name(meta_name),
        ),
    }
}

pub fn bundle_output_paths_for(
    source_path: &Path,
    out_dir: Option<&Path>,
    format: BundleFormat,
) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let stem = source_stem(source_path);
    let root = match out_dir {
        Some(dir) => dir.join(&stem),
        None => source_path.with_file_name(&stem),
    };
    let js_extension = match format {
        BundleFormat::Esm => "js",
        BundleFormat::Cjs => "cjs",
    };
    (
        root.join(format!("{}.wasm", stem)),
        root.join(format!("{}.{}", stem, js_extension)),
        root.join(format!("{}.{}.map", stem, js_extension)),
        root.join(format!("{}.meta.json", stem)),
    )
}

pub fn bundle_chunk_output_dir_for(source_path: &Path, out_dir: Option<&Path>) -> PathBuf {
    let stem = source_stem(source_path);
    let mut hasher = Sha256::new();
    hasher.update(source_path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let suffix = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);
    let chunk_label = format!("{}-{:08x}", stem, suffix);
    match out_dir {
        Some(dir) => dir.join("chunks").join(chunk_label),
        None => source_path
            .with_file_name(stem)
            .join("chunks")
            .join(chunk_label),
    }
}

pub fn capi_output_paths_for(
    source_path: &Path,
    out_dir: Option<&Path>,
) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let stem = source_stem(source_path);
    let wasm_name = format!("{}.capi.wasm", stem);
    let wit_name = format!("{}.wit", stem);
    let header_name = format!("{}.h", stem);
    let meta_name = format!("{}.capi.meta.json", stem);
    match out_dir {
        Some(dir) => (
            dir.join(&wasm_name),
            dir.join(&wit_name),
            dir.join(&header_name),
            dir.join(&meta_name),
        ),
        None => (
            source_path.with_file_name(wasm_name),
            source_path.with_file_name(wit_name),
            source_path.with_file_name(header_name),
            source_path.with_file_name(meta_name),
        ),
    }
}

pub fn binding_package_manifest_output_path_for(
    source_path: &Path,
    out_dir: Option<&Path>,
) -> PathBuf {
    let stem = source_stem(source_path);
    let manifest_name = format!("{}.binding-package.json", stem);
    match out_dir {
        Some(dir) => dir.join(manifest_name),
        None => source_path.with_file_name(manifest_name),
    }
}

pub fn component_output_paths_for(
    source_path: &Path,
    out_dir: Option<&Path>,
) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let stem = source_stem(source_path);
    let wasm_name = format!("{}.component.wasm", stem);
    let wit_name = format!("{}.wit", stem);
    let meta_name = format!("{}.component.meta.json", stem);
    let binding_package_name = format!("{}.binding-package.json", stem);
    match out_dir {
        Some(dir) => (
            dir.join(&wasm_name),
            dir.join(&wit_name),
            dir.join(&meta_name),
            dir.join(&binding_package_name),
        ),
        None => (
            source_path.with_file_name(wasm_name),
            source_path.with_file_name(wit_name),
            source_path.with_file_name(meta_name),
            source_path.with_file_name(binding_package_name),
        ),
    }
}
