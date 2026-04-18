//! Embedding interfaces for Kali.
//!
//! This crate provides a stable embedding-oriented API for consumers that want
//! to compile Kali source in-process without going through the CLI.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use kali_cli::{
    build::{self, BuildMode},
    ApiSurface,
};
use kali_error::{_error_codes::e8, Diagnostic};

/// Compiler configuration for the embedding API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerConfig {
    /// Selected build mode.
    pub build_mode: BuildMode,
    /// Effective API surface used for analysis and artifact metadata.
    pub api_surface: ApiSurface,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            build_mode: BuildMode::Fast,
            api_surface: ApiSurface::Deno,
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
        let mut wasm_bytes = build::compile_source_file(
            path,
            self.config.build_mode,
            self.config.api_surface,
            false,
        )
        .map_err(CompileError::from)?;
        let metadata = build::build_artifact_metadata(
            path,
            "executable",
            self.config.build_mode,
            &self.config.api_surface.to_string(),
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
        let exports = build::collect_library_exports(path).map_err(CompileError::from)?;
        let mut wasm_bytes = build::compile_source_file(
            path,
            self.config.build_mode,
            self.config.api_surface,
            false,
        )
        .map_err(CompileError::from)?;
        let metadata = build::build_artifact_metadata(
            path,
            "lib",
            self.config.build_mode,
            &self.config.api_surface.to_string(),
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

            let exports = build::collect_library_exports(&temp_path).map_err(CompileError::from)?;
            let mut wasm_bytes = build::compile_source_file(
                &temp_path,
                self.config.build_mode,
                self.config.api_surface,
                false,
            )
            .map_err(CompileError::from)?;
            let mut metadata = build::build_artifact_metadata(
                &temp_path,
                "lib",
                self.config.build_mode,
                &self.config.api_surface.to_string(),
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
}

/// Compiled standalone artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledArtifact {
    wasm_bytes: Vec<u8>,
    metadata: ArtifactMetadata,
}

impl CompiledArtifact {
    /// Get the compiled WASM bytes.
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }

    /// Get the associated artifact metadata.
    pub fn metadata(&self) -> &ArtifactMetadata {
        &self.metadata
    }
}

/// Compiled library artifact with a WIT sidecar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibArtifact {
    wasm_bytes: Vec<u8>,
    wit: String,
    metadata: ArtifactMetadata,
}

impl LibArtifact {
    /// Get the compiled WASM bytes.
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }

    /// Get the generated WIT interface description.
    pub fn wit(&self) -> &str {
        &self.wit
    }

    /// Get the associated artifact metadata.
    pub fn metadata(&self) -> &ArtifactMetadata {
        &self.metadata
    }
}

/// Compile error wrapper for embedding callers.
#[derive(Debug, Clone)]
pub struct CompileError {
    diagnostics: Vec<Diagnostic>,
}

impl CompileError {
    /// Access the underlying diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl From<Vec<Diagnostic>> for CompileError {
    fn from(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.diagnostics.first() {
            Some(diagnostic) => write!(f, "{diagnostic}"),
            None => f.write_str("embedding compile error"),
        }
    }
}

impl std::error::Error for CompileError {}

/// Embedding context retained for compatibility with the original stub API.
pub struct EmbeddingCtx {
    compiler: KaliCompiler,
}

impl Default for EmbeddingCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingCtx {
    pub fn new() -> Self {
        Self {
            compiler: KaliCompiler::new(CompilerConfig::default()),
        }
    }

    /// Build a library artifact from raw source text by reusing the stable compiler API.
    pub fn build_library(&self, source: &str) -> Vec<u8> {
        self.compiler
            .compile_lib_source("embedded", source)
            .map(|artifact| artifact.wasm_bytes().to_vec())
            .unwrap_or_default()
    }
}

pub use build::LibraryExport;
pub use kali_cli::build::ArtifactMetadata;

fn temporary_source_path(module_name: &str) -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn compiles_standalone_artifacts_in_memory() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.ts");
        fs::write(&source_path, "export function add(a, b) { return a + b; }")
            .expect("write source");

        let compiler = KaliCompiler::new(CompilerConfig::default());
        let artifact = compiler.compile_file(&source_path).expect("compile file");

        assert!(!artifact.wasm_bytes().is_empty());
        assert_eq!(artifact.metadata().artifact_kind, "executable");
        assert_eq!(
            artifact.metadata().entrypoint,
            source_path.display().to_string()
        );
    }

    #[test]
    fn compiles_library_artifacts_with_wit_sidecars() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("lib.ts");
        fs::write(&source_path, "export function add(a, b) { return a + b; }")
            .expect("write source");

        let compiler = KaliCompiler::new(CompilerConfig::default());
        let artifact = compiler.compile_lib(&source_path).expect("compile lib");

        assert!(!artifact.wasm_bytes().is_empty());
        assert!(artifact.wit().contains("package kali:embed;"));
        assert!(artifact.wit().contains("export add: func();"));
        assert_eq!(artifact.metadata().artifact_kind, "lib");
        assert_eq!(artifact.metadata().exports.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn compile_lib_reports_missing_export_surfaces() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("lib.ts");
        fs::write(&source_path, "const value = 1;").expect("write source");

        let compiler = KaliCompiler::new(CompilerConfig::default());
        let error = compiler
            .compile_lib(&source_path)
            .expect_err("compile should fail");

        assert!(
            error.diagnostics().iter().any(|diagnostic| diagnostic.code
                == Some(kali_error::_error_codes::e5::INVALID_EXPORT_SURFACE as u32)),
            "expected E5011 diagnostic: {error}"
        );
    }

    #[test]
    fn temporary_source_paths_are_unique_across_calls() {
        let first = temporary_source_path("first-module");
        let second = temporary_source_path("second/module");

        assert_ne!(first, second);
        assert!(first.display().to_string().contains("kali-embed-"));
        assert!(second.display().to_string().contains("kali-embed-"));
        assert!(first.display().to_string().contains("first-module"));
        assert!(second.display().to_string().contains("second_module"));
    }

    #[test]
    fn compile_lib_from_raw_source_uses_a_stable_module_name() {
        let compiler = KaliCompiler::new(CompilerConfig::default());
        let artifact = compiler
            .compile_lib_source(
                "math/embedded",
                "export function add(a, b) { return a + b; }",
            )
            .expect("compile lib source");

        assert!(!artifact.wasm_bytes().is_empty());
        assert_eq!(artifact.metadata().artifact_kind, "lib");
        assert_eq!(artifact.metadata().entrypoint, "math/embedded");
        assert!(artifact.wit().contains("// module: math/embedded"));
        assert!(artifact.wit().contains("export add: func();"));
    }

    #[test]
    fn embedding_context_uses_the_stable_compiler_api() {
        let ctx = EmbeddingCtx::new();
        let wasm_bytes = ctx.build_library("export function add(a, b) { return a + b; }");

        assert!(!wasm_bytes.is_empty());
    }
}
