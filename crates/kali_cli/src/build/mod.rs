//! Build pipeline: compile, eval-compat, metadata, entrypoint validation, and export/signature/type collection.

mod compile;
mod entrypoint;
mod eval;
mod exports;
mod helpers;
mod metadata;
pub mod module_link;
pub mod name_anon_functions;
mod paths;
mod wit;

pub use compile::{
    build_mode_from_flags, build_source_file, check_source_file, compile_source_file,
    compile_source_file_with_cache_state, compile_source_file_with_cache_state_and_profile_data,
    compile_source_file_with_cache_state_and_profile_data_and_validation,
    compile_source_file_with_specialization_cap,
    compile_source_file_with_specialization_cap_and_profile_data,
    compile_source_file_with_specialization_cap_and_profile_data_and_validation,
    compile_source_file_with_specialization_cap_and_validation, load_profile_data_file,
    normalize_compiler_source, read_compiler_source_file, validate_runtime_profiles, BuildMode,
    BuildOutput, CompileOutput,
};
pub use entrypoint::reject_async_and_generator_class_methods_in_runtime_entrypoint;
pub use eval::{discover_dynamic_import_targets, DynamicImportTarget};
pub use exports::{collect_browser_bundle_exports, collect_library_exports};
pub use metadata::{
    append_metadata_section, build_artifact_metadata, serialize_artifact_metadata,
    validate_build_result_value, ArtifactMetadata,
};
pub use paths::{
    binding_package_manifest_output_path_for, bundle_chunk_output_dir_for, bundle_output_paths_for,
    capi_output_paths_for, component_output_paths_for, executable_output_path_for,
    library_output_paths_for,
};
pub use wit::{browser_bundle_source_map, library_wit_for, LibraryExport};

// Cutoff re-exports for build_tests.rs (uses `use super::*`); test-only → cfg(test)-gated
// per crates/kali_runtime/src/lib.rs:41-42 precedent (avoids unused-import warnings in lib build).
#[cfg(test)]
pub(crate) use compile::incremental_cache_path;
#[cfg(test)]
pub(crate) use exports::{
    collect_direct_bundle_calls_from_statements, collect_library_exports_from_statements,
};
pub(crate) use helpers::*;
#[cfg(test)]
pub(crate) use metadata::validate_artifact_metadata_value; // parse_source_file, source_stem, has_errors, etc. for sibling modules

// Test-only names re-exported into mod.rs scope for build_tests.rs (`use super::*`); cfg(test)-gated
// to avoid unused-import warnings in the lib build (no production code in this facade uses them).
#[cfg(test)]
use crate::ApiSurface;
#[cfg(test)]
use kali_ast::{Expression, Statement};
#[cfg(test)]
use std::path::{Path, PathBuf};

#[cfg(test)]
#[path = "../build_tests.rs"]
mod tests;
