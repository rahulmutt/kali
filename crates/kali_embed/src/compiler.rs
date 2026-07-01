use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use kali_cli::{
    build::{self, BuildMode},
    ApiSurface,
};
use kali_error::{
    _error_codes::{e5, e8},
    Diagnostic,
};

use crate::artifact::{CompiledArtifact, LibArtifact};
use crate::error::CompileError;

/// Compiler configuration for the embedding API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerConfig {
    /// Selected build mode.
    pub build_mode: BuildMode,
    /// Effective API surface used for analysis and artifact metadata.
    pub api_surface: ApiSurface,
    /// Requested runtime profiles.
    pub runtime_profiles: Vec<String>,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            build_mode: BuildMode::Fast,
            api_surface: ApiSurface::Deno,
            runtime_profiles: Vec::new(),
        }
    }
}

/// Stable embedding compiler entry point.
#[derive(Debug, Clone)]
pub struct KaliCompiler {
    config: CompilerConfig,
}

impl Default for KaliCompiler {
    fn default() -> Self {
        Self::new(CompilerConfig::default())
    }
}

impl KaliCompiler {
    /// Construct a compiler with the provided configuration.
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile a source file into a standalone executable artifact.
    pub fn compile_file(&self, path: &Path) -> Result<CompiledArtifact, CompileError> {
        let runtime_profiles = self.normalized_runtime_profiles()?;
        let mut wasm_bytes = build::compile_source_file(
            path,
            self.config.build_mode,
            self.config.api_surface,
            &runtime_profiles,
            false,
            false,
        )
        .map_err(CompileError::from)?;
        let metadata = build::build_artifact_metadata(
            path,
            "executable",
            self.config.build_mode,
            &self.config.api_surface.to_string(),
            &runtime_profiles,
            16,
            None,
            None,
        )
        .map_err(CompileError::from)?;
        build::append_metadata_section(&mut wasm_bytes, &metadata).map_err(CompileError::from)?;

        Ok(CompiledArtifact {
            wasm_bytes,
            metadata,
        })
    }

    /// Compile a source file into a library artifact plus a deterministic WIT sidecar.
    pub fn compile_lib(&self, path: &Path) -> Result<LibArtifact, CompileError> {
        let runtime_profiles = self.normalized_runtime_profiles()?;
        let exports =
            build::collect_library_exports(path, self.config.api_surface, &runtime_profiles)
                .map_err(CompileError::from)?;
        let mut wasm_bytes = build::compile_source_file(
            path,
            self.config.build_mode,
            self.config.api_surface,
            &runtime_profiles,
            false,
            false,
        )
        .map_err(CompileError::from)?;
        let metadata = build::build_artifact_metadata(
            path,
            "lib",
            self.config.build_mode,
            &self.config.api_surface.to_string(),
            &runtime_profiles,
            16,
            None,
            Some(exports.clone()),
        )
        .map_err(CompileError::from)?;
        build::append_metadata_section(&mut wasm_bytes, &metadata).map_err(CompileError::from)?;
        let wit = build::library_wit_for(&path.display().to_string(), &exports);

        Ok(LibArtifact {
            wasm_bytes,
            wit,
            metadata,
        })
    }

    /// Compile a raw source string into a library artifact plus a deterministic WIT sidecar.
    pub fn compile_lib_source(
        &self,
        module_name: &str,
        source: &str,
    ) -> Result<LibArtifact, CompileError> {
        let runtime_profiles = self.normalized_runtime_profiles()?;
        let temp_path = temporary_source_path(module_name);
        if let Some(parent) = temp_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let result = (|| {
            fs::write(&temp_path, source).map_err(|error| {
                CompileError::from(vec![Diagnostic::error(
                    e8::INTERNAL_ERROR as u32,
                    format!(
                        "failed to materialize embedded library source '{}': {}",
                        temp_path.display(),
                        error
                    ),
                )])
            })?;

            let exports = build::collect_library_exports(
                &temp_path,
                self.config.api_surface,
                &runtime_profiles,
            )
            .map_err(CompileError::from)?;
            let mut wasm_bytes = build::compile_source_file(
                &temp_path,
                self.config.build_mode,
                self.config.api_surface,
                &runtime_profiles,
                false,
                false,
            )
            .map_err(CompileError::from)?;
            let mut metadata = build::build_artifact_metadata(
                &temp_path,
                "lib",
                self.config.build_mode,
                &self.config.api_surface.to_string(),
                &runtime_profiles,
                16,
                None,
                Some(exports.clone()),
            )
            .map_err(CompileError::from)?;
            metadata.entrypoint = module_name.to_string();
            build::append_metadata_section(&mut wasm_bytes, &metadata)
                .map_err(CompileError::from)?;
            let wit = build::library_wit_for(module_name, &exports);

            Ok(LibArtifact {
                wasm_bytes,
                wit,
                metadata,
            })
        })();

        let _ = fs::remove_file(&temp_path);
        result
    }

    fn normalized_runtime_profiles(&self) -> Result<Vec<String>, CompileError> {
        let runtime_profiles =
            build::validate_runtime_profiles(&self.config.runtime_profiles, "embedding config")
                .map_err(CompileError::from)?;

        if runtime_profiles
            .iter()
            .any(|profile| profile == "wasm-threads")
        {
            return Err(CompileError::from(vec![Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "selected runtime profile is unavailable in this phase",
            )]));
        }

        Ok(runtime_profiles)
    }
}

pub(crate) fn temporary_source_path(module_name: &str) -> PathBuf {
    static TEMP_SOURCE_COUNTER: AtomicU64 = AtomicU64::new(0);

    let pid = std::process::id();
    let nonce = TEMP_SOURCE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let module_name = sanitize_module_name(module_name);
    std::env::temp_dir().join(format!("kali-embed-{pid}-{nonce}-{module_name}.ts"))
}

fn sanitize_module_name(module_name: &str) -> String {
    let mut sanitized = String::with_capacity(module_name.len());
    for ch in module_name.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }

    if sanitized.is_empty() {
        String::from("embedded")
    } else {
        sanitized
    }
}
