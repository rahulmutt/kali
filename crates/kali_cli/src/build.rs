use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use kali_ast::{ExportDefaultDeclaration, Statement};
use kali_codegen::{lower_lir_to_wasm, CodegenCtx, TargetConfig};
use kali_common::FileId;
use kali_error::{Diagnostic, _error_codes::e5, _error_codes::e8};
use kali_hir::HirLowerer;
use kali_lexer::Lexer;
use kali_lir::LirLowerer;
use kali_mir::MirLowerer;
use kali_parser::Parser;
use kali_sandbox::SandboxPolicy;
use kali_types::TypeContext;
use serde::Serialize;
use wasm_encoder::{CustomSection, Section};

use crate::is_declaration_only_source_file;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildMode {
    Fast,
    Release,
    ReleaseAdvanced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOutput {
    pub output_path: PathBuf,
    pub wasm_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LibraryExport {
    pub name: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactMetadata {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub entrypoint: String,
    #[serde(rename = "buildMode")]
    pub build_mode: String,
    #[serde(rename = "apiSurface")]
    pub api_surface: String,
    #[serde(rename = "kaliVersion")]
    pub kali_version: String,
    #[serde(rename = "sourceHash")]
    pub source_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<Vec<LibraryExport>>,
}

pub fn check_source_file(source_path: impl AsRef<Path>) -> Result<(), Vec<Diagnostic>> {
    let _analysis = analyze_source_file(source_path.as_ref())?;
    Ok(())
}

pub fn compile_source_file(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let analyzed = analyze_source_file(source_path.as_ref())?;

    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_statements(&analyzed.statements);
    let mut diagnostics = analyzed.diagnostics;
    diagnostics.extend(hir.diagnostics.clone());
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let mir = MirLowerer::new().lower_hir_result(&hir);
    let lir = LirLowerer::new().lower_program(&mir);

    let mut ctx = CodegenCtx::new(TargetConfig {
        optimize: !matches!(mode, BuildMode::Fast),
    });
    let result = lower_lir_to_wasm(&mut ctx, &lir);
    diagnostics.extend(result.diagnostics);

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    Ok(result.wasm_bytes)
}

fn analyze_source_file(source_path: &Path) -> Result<AnalyzedSource, Vec<Diagnostic>> {
    let source = fs::read_to_string(source_path).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "failed to read source file '{}': {}",
                source_path.display(),
                error
            ),
        )]
    })?;

    let lexer = Lexer::new(FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let parsed = parser.parse(Some(source_path.to_string_lossy().to_string()));
    let mut diagnostics = parsed.diagnostics;

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    if !is_declaration_only_source_file(source_path) {
        let mut resolver = TypeContext::with_base_path(source_path);
        let resolved = resolver.resolve_statements_in_file(source_path, &parsed.statements);
        diagnostics.extend(resolved.diagnostics);
        if has_errors(&diagnostics) {
            return Err(diagnostics);
        }
    }

    Ok(AnalyzedSource {
        statements: parsed.statements,
        diagnostics,
    })
}

struct AnalyzedSource {
    statements: Vec<kali_ast::Statement>,
    diagnostics: Vec<Diagnostic>,
}

pub fn build_source_file(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    out_dir: Option<&Path>,
    sandbox_policy: Option<&SandboxPolicy>,
) -> Result<BuildOutput, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let mut wasm_bytes = compile_source_file(source_path, mode)?;
    let metadata = build_artifact_metadata(source_path, "executable", mode, "deno", None)?;
    append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = sandbox_policy {
        let policy_bytes = policy
            .to_canonical_json_bytes()
            .map_err(|diagnostic| vec![diagnostic])?;
        CustomSection {
            name: Cow::Borrowed("kali:policy"),
            data: Cow::Owned(policy_bytes),
        }
        .append_to(&mut wasm_bytes);
    }

    let output_path = executable_output_path_for(source_path, out_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e8::INTERNAL_ERROR as u32,
                format!(
                    "failed to create output directory '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }

    fs::write(&output_path, &wasm_bytes).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "failed to write WASM artifact '{}': {}",
                output_path.display(),
                error
            ),
        )]
    })?;

    Ok(BuildOutput {
        output_path,
        wasm_bytes,
    })
}

pub fn executable_output_path_for(source_path: &Path, out_dir: Option<&Path>) -> PathBuf {
    let stem = source_stem(source_path);
    let file_name = format!("{}.wasm", stem);
    match out_dir {
        Some(dir) => dir.join(file_name),
        None => source_path.with_file_name(file_name),
    }
}

pub fn library_output_paths_for(source_path: &Path, out_dir: Option<&Path>) -> (PathBuf, PathBuf) {
    let stem = source_stem(source_path);
    let wasm_name = format!("{}.lib.wasm", stem);
    let meta_name = format!("{}.lib.meta.json", stem);
    match out_dir {
        Some(dir) => (dir.join(&wasm_name), dir.join(&meta_name)),
        None => (
            source_path.with_file_name(wasm_name),
            source_path.with_file_name(meta_name),
        ),
    }
}

pub fn bundle_output_paths_for(
    source_path: &Path,
    out_dir: Option<&Path>,
) -> (PathBuf, PathBuf, PathBuf) {
    let stem = source_stem(source_path);
    let root = match out_dir {
        Some(dir) => dir.join(&stem),
        None => source_path.with_file_name(&stem),
    };
    (
        root.join(format!("{}.wasm", stem)),
        root.join(format!("{}.js", stem)),
        root.join(format!("{}.meta.json", stem)),
    )
}

pub fn build_mode_from_flags(fast: bool, release: bool, release_advanced: bool) -> BuildMode {
    if release_advanced {
        BuildMode::ReleaseAdvanced
    } else if release {
        BuildMode::Release
    } else if fast {
        BuildMode::Fast
    } else {
        BuildMode::Fast
    }
}

fn source_hash_for_file(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("sha256-{:x}", hasher.finalize()))
}

fn build_mode_name(mode: BuildMode) -> &'static str {
    match mode {
        BuildMode::Fast => "fast",
        BuildMode::Release => "release",
        BuildMode::ReleaseAdvanced => "release-advanced",
    }
}

pub fn build_artifact_metadata(
    source_path: &Path,
    artifact_kind: &str,
    mode: BuildMode,
    api_surface: &str,
    exports: Option<Vec<LibraryExport>>,
) -> Result<ArtifactMetadata, Vec<Diagnostic>> {
    let source_hash = source_hash_for_file(source_path).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "failed to hash source file '{}': {}",
                source_path.display(),
                error
            ),
        )]
    })?;

    Ok(ArtifactMetadata {
        schema_version: 1,
        artifact_kind: artifact_kind.to_string(),
        entrypoint: source_path.to_string_lossy().to_string(),
        build_mode: build_mode_name(mode).to_string(),
        api_surface: api_surface.to_string(),
        kali_version: env!("CARGO_PKG_VERSION").to_string(),
        source_hash,
        exports,
    })
}

pub fn serialize_artifact_metadata(metadata: &ArtifactMetadata) -> Vec<u8> {
    serde_json::to_vec(metadata).expect("serialize artifact metadata")
}

pub fn append_metadata_section(
    wasm_bytes: &mut Vec<u8>,
    metadata: &ArtifactMetadata,
) -> Result<(), Vec<Diagnostic>> {
    let metadata_bytes = serialize_artifact_metadata(metadata);
    CustomSection {
        name: Cow::Borrowed("kali:metadata"),
        data: Cow::Owned(metadata_bytes),
    }
    .append_to(wasm_bytes);
    Ok(())
}

pub fn collect_library_exports(
    source_path: impl AsRef<Path>,
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let source = fs::read_to_string(source_path).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "failed to read source file '{}': {}",
                source_path.display(),
                error
            ),
        )]
    })?;

    let lexer = Lexer::new(FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let parsed = parser.parse(Some(source_path.to_string_lossy().to_string()));
    let mut diagnostics = parsed.diagnostics;

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let mut exports = BTreeMap::<String, String>::new();
    for statement in parsed.statements {
        match statement {
            Statement::FunctionDeclaration(func) => {
                exports
                    .entry(func.name.clone())
                    .or_insert_with(|| function_signature(&func.params));
            }
            Statement::ExportNamed(declaration) => {
                if declaration.source.is_some() {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        "re-exported surfaces are not statically known yet",
                    ));
                    continue;
                }

                for specifier in declaration.specifiers {
                    exports
                        .entry(specifier.exported.clone())
                        .or_insert_with(|| signature_from_export_specifier(&specifier.local));
                }
            }
            Statement::ExportDefault(default_decl) => match default_decl {
                ExportDefaultDeclaration::FunctionDeclaration(func) => {
                    exports
                        .entry(if func.name.is_empty() {
                            "default".to_string()
                        } else {
                            func.name.clone()
                        })
                        .or_insert_with(|| function_signature(&func.params));
                }
                ExportDefaultDeclaration::Expression(_)
                | ExportDefaultDeclaration::ClassDeclaration(_) => {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        "default export expressions and classes are not part of the Phase-1 base library artifact",
                    ));
                }
            },
            Statement::ImportDeclaration(_)
            | Statement::BreakStatement(_)
            | Statement::ContinueStatement(_)
            | Statement::WithStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::LabeledStatement(_)
            | Statement::IfStatement(_)
            | Statement::SwitchStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::TryStatement(_)
            | Statement::DebuggerStatement(_)
            | Statement::BlockStatement(_)
            | Statement::ForStatement(_)
            | Statement::ForInStatement(_)
            | Statement::ForOfStatement(_)
            | Statement::WhileStatement(_)
            | Statement::DoWhileStatement(_)
            | Statement::ClassDeclaration(_)
            | Statement::VariableDeclaration(_)
            | Statement::EnumDeclaration(_)
            | Statement::TypeAliasDeclaration(_)
            | Statement::InterfaceDeclaration(_)
            | Statement::ExpressionStatement(_) => {}
        }
    }

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let exports = exports
        .into_iter()
        .map(|(name, signature)| LibraryExport { name, signature })
        .collect::<Vec<_>>();

    if exports.is_empty() {
        return Err(vec![invalid_export_surface(
            source_path,
            "no statically known export surface was found",
        )]);
    }

    Ok(exports)
}

fn function_signature(params: &[String]) -> String {
    format!("({}) => unknown", params.join(", "))
}

fn signature_from_export_specifier(local: &str) -> String {
    format!("({}) => unknown", local)
}

fn invalid_export_surface(source_path: &Path, message: &str) -> Diagnostic {
    Diagnostic::error(
        e5::INVALID_EXPORT_SURFACE as u32,
        format!(
            "cannot build library artifact from '{}': {}",
            source_path.display(),
            message
        ),
    )
}

fn source_stem(source_path: &Path) -> String {
    source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("main")
        .to_string()
}

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.is_error())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use wasmparser::Validator;

    #[test]
    fn build_source_file_writes_valid_wasm_artifact() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("main.ts");
        fs::write(
            &source_path,
            "function add(a, b) { return a + b; } add(1, 2);",
        )
        .expect("write source");

        let output = build_source_file(&source_path, BuildMode::Fast, None, None)
            .expect("build should succeed");

        assert!(output.output_path.exists());
        Validator::new()
            .validate_all(&output.wasm_bytes)
            .expect("artifact should validate");
    }

    #[test]
    fn output_path_uses_source_stem() {
        let source = PathBuf::from("/tmp/demo/main.ts");
        let output = executable_output_path_for(&source, Some(Path::new("dist")));
        assert_eq!(output, PathBuf::from("dist/main.wasm"));
    }
}
