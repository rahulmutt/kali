//! Embedding interfaces for Kali.
//!
//! This crate provides a stable embedding-oriented API for consumers that want
//! to compile Kali source in-process without going through the CLI.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use kali_cli::{
    build::{self, BuildMode},
    ApiSurface,
};
use kali_error::{
    _error_codes::{e4, e5, e8},
    Diagnostic,
};
pub use kali_sandbox::{
    HostOperation, HostPredicate, PolicyPredicateContext, PolicyPredicateRegistry, SandboxPolicy,
};

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

/// Decision returned by host-registered sandbox predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateDecision {
    /// Allow the guarded operation to proceed.
    Allow,
    /// Reject the guarded operation with a host-specific note.
    Deny(String),
}

impl PredicateDecision {
    /// Convenience constructor for an allowed operation.
    pub fn allow() -> Self {
        Self::Allow
    }

    /// Convenience constructor for a rejected operation.
    pub fn deny(message: impl Into<String>) -> Self {
        Self::Deny(message.into())
    }
}

impl From<bool> for PredicateDecision {
    fn from(value: bool) -> Self {
        if value {
            Self::Allow
        } else {
            Self::Deny(String::new())
        }
    }
}

/// Canonical operation context observed by host-registered narrowing predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationContext {
    /// Canonical capability name from the sandbox vocabulary.
    pub capability: String,
    /// Subject/resource string associated with the host operation.
    pub resource: String,
    /// Host operation being evaluated.
    pub operation: HostOperation,
    /// Deterministic extra details for host-specific predicate logic.
    pub details: BTreeMap<String, String>,
}

impl OperationContext {
    /// Create the canonical predicate context for one host operation.
    pub fn from_operation(operation: &HostOperation) -> Self {
        let policy_context = PolicyPredicateContext::from_operation(operation);
        Self {
            capability: policy_context.capability,
            resource: policy_context.subject,
            operation: policy_context.operation,
            details: policy_context.details,
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
        let exports = build::collect_library_exports(path).map_err(CompileError::from)?;
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

            let exports = build::collect_library_exports(&temp_path).map_err(CompileError::from)?;
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

#[derive(Clone)]
struct RegisteredPredicate {
    name: String,
    predicate: Arc<dyn Fn(&OperationContext) -> PredicateDecision + Send + Sync + 'static>,
}

/// Embedding context retained for compatibility with the original stub API.
pub struct EmbeddingCtx {
    compiler: KaliCompiler,
    predicates: BTreeMap<String, Vec<RegisteredPredicate>>,
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
            predicates: BTreeMap::new(),
        }
    }

    /// Register a deterministic narrowing predicate for one canonical capability name.
    pub fn register_sandbox_predicate(
        &mut self,
        capability: impl Into<String>,
        name: impl Into<String>,
        predicate: impl Fn(&OperationContext) -> PredicateDecision + Send + Sync + 'static,
    ) -> &mut Self {
        let capability = capability.into();
        let entry = self.predicates.entry(capability).or_default();
        entry.push(RegisteredPredicate {
            name: name.into(),
            predicate: Arc::new(predicate),
        });
        self
    }

    /// Evaluate a host operation against a declarative policy and the registered predicates.
    pub fn check_operation_with_policy(
        &self,
        policy: &SandboxPolicy,
        operation: HostOperation,
    ) -> Result<(), Diagnostic> {
        policy.check_operation(operation.clone())?;

        let context = OperationContext::from_operation(&operation);
        let Some(predicates) = self.predicates.get(&context.capability) else {
            return Ok(());
        };

        for predicate in predicates {
            match (predicate.predicate)(&context) {
                PredicateDecision::Allow => {}
                PredicateDecision::Deny(reason) => {
                    return Err(predicate_violation(&predicate.name, &context, &reason));
                }
            }
        }

        Ok(())
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

fn predicate_violation(name: &str, context: &OperationContext, reason: &str) -> Diagnostic {
    let reason_suffix = if reason.trim().is_empty() {
        String::new()
    } else {
        format!(": {}", reason)
    };

    Diagnostic::error(
        e4::EFFECT_NOT_PERMITTED as u32,
        format!(
            "host-registered predicate '{}' rejected {} for resource '{}'{}",
            name, context.capability, context.resource, reason_suffix
        ),
    )
}

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
#[path = "tests.rs"]
mod tests;
