use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use kali_ast::{ExportDefaultDeclaration, Expression, Statement};
use kali_codegen::{lower_lir_to_wasm, CodegenCtx, TargetConfig};
use kali_common::{template::resolve_interpolated_template_literal, FileId};
use kali_error::{_error_codes::e5, _error_codes::e8, Diagnostic};
use kali_hir::HirLowerer;
use kali_lexer::{Lexer, Token, TokenType};
use kali_lir::LirLowerer;
use kali_mir::MirLowerer;
use kali_optimize::{OptimizationLevel, Optimizer, ProfileData, PROFILE_DATA_VERSION};
use kali_parser::Parser;
use kali_runtime::{normalize_runtime_profiles, RuntimeBackend, RuntimeHostContract};
use kali_sandbox::SandboxPolicy;
use kali_types::TypeContext;
use serde::Serialize;
use serde_json::{json, Value};
use wasm_encoder::{CustomSection, Section};

use crate::{
    is_declaration_only_source_file, output::validate_sorted_string_array_value, ApiSurface,
    BundleFormat,
};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicImportTarget {
    pub specifier: String,
    pub target: PathBuf,
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
    #[serde(rename = "runtimeProfiles")]
    pub runtime_profiles: Vec<String>,
    #[serde(rename = "maxSpecializations")]
    pub max_specializations: usize,
    #[serde(rename = "hostContract", skip_serializing_if = "Option::is_none")]
    pub host_contract: Option<String>,
    #[serde(rename = "runtimeBackend", skip_serializing_if = "Option::is_none")]
    pub runtime_backend: Option<String>,
    #[serde(rename = "kaliVersion")]
    pub kali_version: String,
    #[serde(rename = "sourceHash")]
    pub source_hash: String,
    #[serde(rename = "profileDataHash", skip_serializing_if = "Option::is_none")]
    pub profile_data_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<Vec<LibraryExport>>,
}

pub fn check_source_file(
    source_path: impl AsRef<Path>,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    compat_eval: bool,
) -> Result<(), Vec<Diagnostic>> {
    let _analysis = analyze_source_file(
        source_path.as_ref(),
        api_surface,
        runtime_profiles,
        compat_eval,
    )?;
    Ok(())
}

pub fn normalize_compiler_source(source: &str) -> Cow<'_, str> {
    if source.starts_with("#!") {
        let mut normalized = source.to_string();
        normalized.replace_range(..2, "//");
        Cow::Owned(normalized)
    } else {
        Cow::Borrowed(source)
    }
}

pub fn read_compiler_source_file(source_path: &Path) -> Result<String, Vec<Diagnostic>> {
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

    Ok(normalize_compiler_source(&source).into_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileOutput {
    pub wasm_bytes: Vec<u8>,
    pub cache_hit: bool,
    pub cache_path: Option<PathBuf>,
}

pub fn load_profile_data_file(
    profile_path: impl AsRef<Path>,
) -> Result<ProfileData, Vec<Diagnostic>> {
    let profile_path = profile_path.as_ref();
    let profile_json = fs::read_to_string(profile_path).map_err(|error| {
        vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            format!(
                "failed to read PGO profile data '{}': {}",
                profile_path.display(),
                error
            ),
        )]
    })?;

    let profile_data: ProfileData = serde_json::from_str(&profile_json).map_err(|error| {
        vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            format!(
                "failed to parse PGO profile data '{}': {}",
                profile_path.display(),
                error
            ),
        )]
    })?;

    if !profile_data.is_current_version() {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            format!(
                "unsupported PGO profile data version {} in '{}'; expected {}",
                profile_data.version,
                profile_path.display(),
                PROFILE_DATA_VERSION
            ),
        )]);
    }

    Ok(profile_data.normalized())
}

fn profile_data_hash(profile_data: Option<&ProfileData>) -> Option<String> {
    profile_data.map(|profile_data| {
        let normalized = profile_data.clone().normalized();
        let profile_json = serde_json::to_vec(&normalized).expect("serialize profile data");
        let profile_hash = Sha256::digest(profile_json);
        format!("sha256-{profile_hash:x}")
    })
}

#[allow(clippy::too_many_arguments)]
pub fn compile_source_file_with_cache_state(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    compat_eval: bool,
    coverage: bool,
) -> Result<CompileOutput, Vec<Diagnostic>> {
    compile_source_file_with_cache_state_and_profile_data_and_validation(
        source_path,
        mode,
        max_specializations,
        api_surface,
        None,
        runtime_profiles,
        compat_eval,
        false,
        coverage,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn compile_source_file_with_cache_state_and_profile_data(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
    compat_eval: bool,
    coverage: bool,
) -> Result<CompileOutput, Vec<Diagnostic>> {
    compile_source_file_with_cache_state_and_profile_data_and_validation(
        source_path,
        mode,
        max_specializations,
        api_surface,
        profile_data,
        runtime_profiles,
        compat_eval,
        false,
        coverage,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn compile_source_file_with_cache_state_and_profile_data_and_validation(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
    compat_eval: bool,
    validate_ir: bool,
    coverage: bool,
) -> Result<CompileOutput, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let runtime_profiles = validate_runtime_profiles(
        runtime_profiles,
        &format!("compile request for '{}'", source_path.display()),
    )?;
    let profile_data = profile_data.cloned().map(ProfileData::normalized);

    if let Some(cache_path) = if validate_ir {
        None
    } else {
        incremental_cache_path(
            source_path,
            mode,
            max_specializations,
            api_surface,
            &runtime_profiles,
            profile_data.as_ref(),
            compat_eval,
            coverage,
        )?
    } {
        match fs::read(&cache_path) {
            Ok(wasm_bytes) => {
                return Ok(CompileOutput {
                    wasm_bytes,
                    cache_hit: true,
                    cache_path: Some(cache_path),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(vec![Diagnostic::error(
                    e8::INTERNAL_ERROR as u32,
                    format!(
                        "failed to read incremental cache '{}': {}",
                        cache_path.display(),
                        error
                    ),
                )])
            }
        }

        let wasm_bytes = compile_source_file_uncached(
            source_path,
            mode,
            max_specializations,
            api_surface,
            profile_data.as_ref(),
            &runtime_profiles,
            compat_eval,
            validate_ir,
            coverage,
        )?;

        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
            let _ = fs::write(&cache_path, &wasm_bytes);
        }

        Ok(CompileOutput {
            wasm_bytes,
            cache_hit: false,
            cache_path: Some(cache_path),
        })
    } else {
        let wasm_bytes = compile_source_file_uncached(
            source_path,
            mode,
            max_specializations,
            api_surface,
            profile_data.as_ref(),
            &runtime_profiles,
            compat_eval,
            validate_ir,
            coverage,
        )?;
        Ok(CompileOutput {
            wasm_bytes,
            cache_hit: false,
            cache_path: None,
        })
    }
}

pub fn compile_source_file(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    compat_eval: bool,
    coverage: bool,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_source_file_with_specialization_cap(
        source_path,
        mode,
        16,
        api_surface,
        runtime_profiles,
        compat_eval,
        coverage,
    )
}

pub fn compile_source_file_with_specialization_cap(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    compat_eval: bool,
    coverage: bool,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_source_file_with_specialization_cap_and_profile_data_and_validation(
        source_path,
        mode,
        max_specializations,
        api_surface,
        None,
        runtime_profiles,
        compat_eval,
        false,
        coverage,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn compile_source_file_with_specialization_cap_and_validation(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    compat_eval: bool,
    validate_ir: bool,
    coverage: bool,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_source_file_with_specialization_cap_and_profile_data_and_validation(
        source_path,
        mode,
        max_specializations,
        api_surface,
        None,
        runtime_profiles,
        compat_eval,
        validate_ir,
        coverage,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn compile_source_file_with_specialization_cap_and_profile_data(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
    compat_eval: bool,
    coverage: bool,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_source_file_with_specialization_cap_and_profile_data_and_validation(
        source_path,
        mode,
        max_specializations,
        api_surface,
        profile_data,
        runtime_profiles,
        compat_eval,
        false,
        coverage,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn compile_source_file_with_specialization_cap_and_profile_data_and_validation(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
    compat_eval: bool,
    validate_ir: bool,
    coverage: bool,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_source_file_with_cache_state_and_profile_data_and_validation(
        source_path,
        mode,
        max_specializations,
        api_surface,
        profile_data,
        runtime_profiles,
        compat_eval,
        validate_ir,
        coverage,
    )
    .map(|output| output.wasm_bytes)
}

#[allow(clippy::too_many_arguments)]
fn compile_source_file_uncached(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    profile_data: Option<&ProfileData>,
    runtime_profiles: &[String],
    compat_eval: bool,
    validate_ir: bool,
    coverage: bool,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let analyzed = analyze_source_file(
        source_path.as_ref(),
        api_surface,
        runtime_profiles,
        compat_eval,
    )?;
    let mut hir_lowerer = HirLowerer::new();
    let hir = hir_lowerer.lower_statements(&analyzed.statements);
    let mut diagnostics = analyzed.diagnostics;
    diagnostics.extend(hir.diagnostics.clone());
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }
    if validate_ir {
        validate_hir_tree(&hir)?;
    }

    let mir = MirLowerer::new().lower_hir_result(&hir);
    if validate_ir {
        validate_mir_program(&mir)?;
    }
    let mut lir = LirLowerer::new().lower_program(&mir);
    if validate_ir {
        validate_lir_program(&lir)?;
    }

    let optimization_level = match mode {
        BuildMode::Fast => OptimizationLevel::Fast,
        BuildMode::Release => OptimizationLevel::Release,
        BuildMode::ReleaseAdvanced => OptimizationLevel::ReleaseAdvanced,
    };
    let optimizer = Optimizer::with_max_specializations(optimization_level, max_specializations);
    let optimizer = if let Some(profile_data) = profile_data {
        optimizer.with_profile_data(profile_data.clone())
    } else {
        optimizer
    };
    optimizer.optimize_program_with_mir(&mut lir, &mir);

    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations,
        compat_eval,
        coverage,
    });
    ctx.source_path = Some(source_path.as_ref().to_path_buf());
    let result = lower_lir_to_wasm(&mut ctx, &lir);
    diagnostics.extend(result.diagnostics);

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    Ok(result.wasm_bytes)
}

fn validation_diagnostic(stage: &str, error: String) -> Vec<Diagnostic> {
    vec![Diagnostic::error(
        e8::INTERNAL_ERROR as u32,
        format!("{stage} validation failed: {error}"),
    )]
}

fn validate_hir_tree(hir: &kali_hir::LoweringResult) -> Result<(), Vec<Diagnostic>> {
    hir.validate()
        .map_err(|error| validation_diagnostic("HIR", error))
}

fn validate_mir_program(mir: &kali_mir::MirProgram) -> Result<(), Vec<Diagnostic>> {
    mir.validate()
        .map_err(|error| validation_diagnostic("MIR", error))
}

fn validate_lir_program(lir: &kali_lir::LirProgram) -> Result<(), Vec<Diagnostic>> {
    lir.validate()
        .map_err(|error| validation_diagnostic("LIR", error))
}

#[allow(clippy::too_many_arguments)]
fn incremental_cache_path(
    source_path: &Path,
    mode: BuildMode,
    max_specializations: usize,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    profile_data: Option<&ProfileData>,
    compat_eval: bool,
    coverage: bool,
) -> Result<Option<PathBuf>, Vec<Diagnostic>> {
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
    let Some(project_root) = project_root_for_source(source_path) else {
        return Ok(None);
    };
    let normalized_runtime_profiles = normalize_runtime_profiles(runtime_profiles.to_vec());
    let profile_key = profile_data
        .map(|profile| {
            let profile = profile.clone().normalized();
            let profile_json = serde_json::to_string(&profile).expect("serialize profile data");
            let profile_hash = Sha256::digest(profile_json.as_bytes());
            format!("profile:{profile_hash:x}")
        })
        .unwrap_or_else(|| "profile:none".to_string());
    let cache_key = format!(
        "{}-{}-{}-{}-profiles:{}-{}-{}-{}-{}",
        source_hash,
        build_mode_name(mode),
        api_surface,
        max_specializations,
        normalized_runtime_profiles.join(","),
        profile_key,
        compat_eval,
        coverage,
        env!("CARGO_PKG_VERSION")
    );
    Ok(Some(
        project_root
            .join(".kali-cache")
            .join("incremental")
            .join(format!("{}.wasm", cache_key)),
    ))
}

fn project_root_for_source(source_path: &Path) -> Option<PathBuf> {
    let start = source_path.parent().unwrap_or(source_path);
    for ancestor in start.ancestors() {
        if ancestor.join("kali.json").exists() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn analyze_source_file(
    source_path: &Path,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    compat_eval: bool,
) -> Result<AnalyzedSource, Vec<Diagnostic>> {
    let source = read_compiler_source_file(source_path)?;

    let source = if compat_eval {
        rewrite_eval_compat_source(&source)
    } else {
        source
    };

    if source_uses_process_env_mutation(&source) {
        return Err(vec![Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "environment mutation API 'process.env' (aka process[\"env\"]) is unavailable until the later mutable env path is enabled",
        )]);
    }

    let lexer = Lexer::new(FileId::new(0), source.clone());
    let tokens = lexer.lex_all().tokens;
    if !compat_eval && source_uses_eval_compat(&tokens) {
        return Err(vec![Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "compatibility feature 'eval' (including `Function()`) is unavailable without `--compat eval`",
        )]);
    }

    let lexer = Lexer::new(FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let parsed = parser.parse(Some(source_path.to_string_lossy().to_string()));
    let mut diagnostics = parsed.diagnostics;

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    diagnostics.extend(validate_unique_export_names_from_statements(
        &parsed.statements,
        source_path,
    ));
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    if !is_declaration_only_source_file(source_path) {
        let mut resolver = TypeContext::with_base_path_and_api_surface_and_runtime_profiles(
            source_path,
            api_surface.to_string(),
            runtime_profiles.to_vec(),
        );
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

fn source_uses_process_env_mutation(source: &str) -> bool {
    let patterns = [
        "process.env =",
        "globalThis.process.env =",
        "process[\"env\"] =",
        "process['env'] =",
        "globalThis.process[\"env\"] =",
        "globalThis.process['env'] =",
        "globalThis[\"process\"][\"env\"] =",
        "globalThis['process']['env'] =",
    ];

    patterns.iter().any(|pattern| source.contains(pattern))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EvalConst {
    String(String),
    Number(i64),
    Boolean(bool),
    Null,
}

impl EvalConst {
    fn render(&self) -> String {
        match self {
            EvalConst::String(value) => {
                serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value))
            }
            EvalConst::Number(value) => value.to_string(),
            EvalConst::Boolean(value) => value.to_string(),
            EvalConst::Null => "null".to_string(),
        }
    }

    fn to_string_value(&self) -> String {
        match self {
            EvalConst::String(value) => value.clone(),
            EvalConst::Number(value) => value.to_string(),
            EvalConst::Boolean(value) => value.to_string(),
            EvalConst::Null => "null".to_string(),
        }
    }
}

fn rewrite_eval_compat_source(source: &str) -> String {
    let source = rewrite_static_eval_calls(source);
    rewrite_static_function_constructor_calls(&source)
}

fn rewrite_static_eval_calls(source: &str) -> String {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let bindings = collect_constant_bindings(&tokens, source);
    let mut rewritten = source.to_string();
    let mut search_start = 0usize;

    while let Some(relative) = rewritten[search_start..].find("eval(") {
        let call_start = search_start + relative;
        let arg_start = call_start + "eval(".len();
        let Some(call_end) = find_call_end(&rewritten, arg_start) else {
            break;
        };
        let arg_source = &rewritten[arg_start..call_end];
        let lexer = Lexer::new(FileId::new(1), arg_source.to_string());
        let mut arg_tokens = lexer.lex_all().tokens;
        while matches!(arg_tokens.last(), Some(token) if token.kind == TokenType::Eof) {
            arg_tokens.pop();
        }

        let mut replacement = None;
        if let Some((value, consumed)) = parse_constant_expression(&arg_tokens, 0, &bindings) {
            if consumed == arg_tokens.len() {
                if let EvalConst::String(source_snippet) = value {
                    if let Some(result) = parse_eval_source_snippet(&source_snippet) {
                        replacement = Some(result.render());
                    }
                }
            }
        }

        if let Some(replacement) = replacement {
            rewritten.replace_range(call_start..=call_end, &replacement);
            search_start = call_start + replacement.len();
        } else {
            search_start = call_end + 1;
        }
    }

    rewritten
}

fn rewrite_static_function_constructor_calls(source: &str) -> String {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let bindings = collect_constant_bindings(&tokens, source);
    let mut rewritten = source.to_string();
    let mut search_start = 0usize;

    while let Some(relative) = rewritten[search_start..].find("Function(") {
        let call_start = search_start + relative;
        if !is_bare_function_constructor_spelling(&rewritten, call_start) {
            search_start = call_start + "Function(".len();
            continue;
        }

        let prefix_start = if call_start >= 4 && &rewritten[call_start - 4..call_start] == "new " {
            call_start - 4
        } else {
            call_start
        };
        let arg_start = call_start + "Function(".len();
        let Some(call_end) = find_call_end(&rewritten, arg_start) else {
            break;
        };
        let immediate_call_end = match find_immediate_invocation_end(&rewritten, call_end) {
            Some(end) => end,
            None => {
                search_start = call_end + 1;
                continue;
            }
        };

        let arg_source = &rewritten[arg_start..call_end];
        let lexer = Lexer::new(FileId::new(1), arg_source.to_string());
        let mut arg_tokens = lexer.lex_all().tokens;
        while matches!(arg_tokens.last(), Some(token) if token.kind == TokenType::Eof) {
            arg_tokens.pop();
        }

        let mut replacement = None;
        if let Some((value, consumed)) = parse_constant_expression(&arg_tokens, 0, &bindings) {
            if consumed == arg_tokens.len() {
                if let EvalConst::String(body_source) = value {
                    if let Some(result) = parse_function_constructor_body_snippet(&body_source) {
                        replacement = Some(result.render());
                    }
                }
            }
        }

        if let Some(replacement) = replacement {
            rewritten.replace_range(prefix_start..=immediate_call_end, &replacement);
            search_start = prefix_start + replacement.len();
        } else {
            search_start = immediate_call_end + 1;
        }
    }

    rewritten
}

fn is_bare_function_constructor_spelling(source: &str, index: usize) -> bool {
    !matches!(
        source[..index].chars().next_back(),
        Some(ch) if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || ch == '.'
    )
}

fn find_immediate_invocation_end(source: &str, constructor_end: usize) -> Option<usize> {
    let remainder = source.get(constructor_end + 1..)?;
    let trimmed = remainder.trim_start();
    let open = constructor_end + 1 + (remainder.len().saturating_sub(trimmed.len()));
    if source.as_bytes().get(open).copied()? != b'(' {
        return None;
    }
    find_call_end(source, open + 1)
}

fn parse_function_constructor_body_snippet(source: &str) -> Option<EvalConst> {
    let trimmed = source.trim();
    let body = trimmed.strip_prefix("return")?.trim();
    let body = body.strip_suffix(';').unwrap_or(body).trim();
    if body.is_empty() {
        return None;
    }
    parse_eval_source_snippet(body)
}

fn find_call_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0i32;
    let mut index = start;
    let mut string_delimiter: Option<u8> = None;
    let mut escape = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(delimiter) = string_delimiter {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == delimiter {
                string_delimiter = None;
            }
            index += 1;
            continue;
        }

        match byte {
            b'\'' | b'\"' | b'`' => string_delimiter = Some(byte),
            b'(' => depth += 1,
            b')' => {
                if depth == 0 {
                    return Some(index);
                }
                depth -= 1;
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn collect_constant_bindings(tokens: &[Token], source: &str) -> BTreeMap<String, EvalConst> {
    let mut bindings = BTreeMap::new();
    let mut index = 0usize;

    while index + 3 < tokens.len() {
        let token = &tokens[index];
        if matches!(
            token.kind,
            TokenType::Const | TokenType::Let | TokenType::Var
        ) && matches!(tokens.get(index + 1), Some(name) if name.kind == TokenType::Identifier)
            && matches!(tokens.get(index + 2), Some(eq) if eq.kind == TokenType::Eq)
        {
            let name = tokens[index + 1].value.clone();
            let expr_start = index + 3;
            let expr_end = find_statement_end(tokens, expr_start);
            if expr_end > expr_start {
                if let Some((value, consumed)) =
                    parse_constant_expression(&tokens[expr_start..expr_end], 0, &bindings)
                {
                    if consumed == expr_end - expr_start {
                        bindings.insert(name, value);
                    }
                }
            }
        }
        index += 1;
    }

    let _ = source;
    bindings
}

pub fn discover_dynamic_import_targets(
    source: &Path,
    source_contents: &str,
) -> Result<Vec<DynamicImportTarget>, Vec<Diagnostic>> {
    let lexer = Lexer::new(FileId::new(0), source_contents.to_string());
    let tokens = lexer
        .lex_all()
        .tokens
        .into_iter()
        .filter(|token| !matches!(token.kind, TokenType::Comment | TokenType::Eof))
        .collect::<Vec<_>>();
    let bindings = collect_constant_bindings(&tokens, source_contents);
    let mut targets = Vec::new();
    let mut index = 0usize;

    while index + 1 < tokens.len() {
        if tokens[index].kind != TokenType::Import || tokens[index + 1].kind != TokenType::LeftParen
        {
            index += 1;
            continue;
        }

        if let Some((specifier, next_index)) =
            parse_static_dynamic_import_specifier(&tokens, index + 1, &bindings)
        {
            if let Some(target) = resolve_dynamic_import_target(source, &specifier) {
                targets.push(DynamicImportTarget { specifier, target });
            }
            index = next_index;
        } else {
            index += 1;
        }
    }

    Ok(targets)
}

fn parse_static_dynamic_import_specifier(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(String, usize)> {
    let call_end = find_token_call_end(tokens, index)?;
    let (value, consumed) = parse_constant_expression(&tokens[index + 1..call_end], 0, env)?;
    if consumed == call_end.saturating_sub(index + 1) {
        Some((value.to_string_value(), call_end + 1))
    } else {
        None
    }
}

fn find_token_call_end(tokens: &[Token], start: usize) -> Option<usize> {
    let mut depth = 1i32;
    for (index, token) in tokens.iter().enumerate().skip(start + 1) {
        match token.kind {
            TokenType::LeftParen => depth += 1,
            TokenType::RightParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

fn resolve_dynamic_import_target(source: &Path, specifier: &str) -> Option<PathBuf> {
    let specifier = specifier.trim();
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }

    let parent = source.parent()?;
    let candidate = parent.join(specifier);
    let try_paths = std::iter::once(candidate.clone()).chain([
        candidate.with_extension("ts"),
        candidate.with_extension("tsx"),
        candidate.with_extension("js"),
        candidate.with_extension("jsx"),
        candidate.with_extension("mts"),
        candidate.with_extension("mjs"),
        candidate.with_extension("cts"),
        candidate.with_extension("cjs"),
    ]);
    for path in try_paths {
        if let Some(canonical) = canonicalize_dynamic_import_candidate(&path) {
            return Some(canonical);
        }
    }
    None
}

fn canonicalize_dynamic_import_candidate(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_file() {
        return fs::canonicalize(candidate).ok();
    }

    if candidate.is_dir() {
        for index_name in [
            "index.ts",
            "index.tsx",
            "index.js",
            "index.jsx",
            "index.mts",
            "index.mjs",
            "index.cts",
            "index.cjs",
        ] {
            let index_candidate = candidate.join(index_name);
            if index_candidate.is_file() {
                return fs::canonicalize(index_candidate).ok();
            }
        }
    }

    None
}

fn find_statement_end(tokens: &[Token], start: usize) -> usize {
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    let mut depth_brace = 0i32;

    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind {
            TokenType::LeftParen => depth_paren += 1,
            TokenType::RightParen => {
                if depth_paren == 0 {
                    return index;
                }
                depth_paren -= 1;
            }
            TokenType::LeftBracket => depth_bracket += 1,
            TokenType::RightBracket => {
                if depth_bracket == 0 {
                    return index;
                }
                depth_bracket -= 1;
            }
            TokenType::LeftBrace => depth_brace += 1,
            TokenType::RightBrace => {
                if depth_brace == 0 {
                    return index;
                }
                depth_brace -= 1;
            }
            TokenType::Semicolon if depth_paren == 0 && depth_bracket == 0 && depth_brace == 0 => {
                return index;
            }
            _ => {}
        }
    }
    tokens.len()
}

fn source_uses_eval_compat(tokens: &[Token]) -> bool {
    for index in 0..tokens.len().saturating_sub(1) {
        let current = &tokens[index];
        let next = &tokens[index + 1];
        let previous_is_dot = index
            .checked_sub(1)
            .and_then(|previous| tokens.get(previous))
            .is_some_and(|token| token.kind == TokenType::Dot);

        if !previous_is_dot
            && current.kind == TokenType::Identifier
            && current.value == "eval"
            && next.kind == TokenType::LeftParen
        {
            return true;
        }

        if !previous_is_dot
            && current.kind == TokenType::Identifier
            && current.value == "Function"
            && next.kind == TokenType::LeftParen
        {
            return true;
        }

        if current.kind == TokenType::New
            && next.kind == TokenType::Identifier
            && next.value == "Function"
            && tokens
                .get(index + 2)
                .is_some_and(|token| token.kind == TokenType::LeftParen)
        {
            return true;
        }
    }
    false
}

fn parse_eval_source_snippet(source: &str) -> Option<EvalConst> {
    let lexer = Lexer::new(FileId::new(1), source.to_string());
    let mut tokens = lexer.lex_all().tokens;
    while matches!(tokens.last(), Some(token) if token.kind == TokenType::Eof) {
        tokens.pop();
    }
    let env = BTreeMap::new();
    let (value, consumed) = parse_constant_expression(&tokens, 0, &env)?;
    if consumed == tokens.len() {
        Some(value)
    } else {
        None
    }
}

fn parse_constant_expression(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(EvalConst, usize)> {
    let (mut left, mut index) = parse_constant_primary(tokens, index, env)?;
    while let Some(token) = tokens.get(index) {
        if token.kind != TokenType::Plus {
            break;
        }
        let (right, next_index) = parse_constant_primary(tokens, index + 1, env)?;
        left = eval_plus(left, right);
        index = next_index;
    }
    Some((left, index))
}

fn parse_constant_primary(
    tokens: &[Token],
    index: usize,
    env: &BTreeMap<String, EvalConst>,
) -> Option<(EvalConst, usize)> {
    let token = tokens.get(index)?;
    match token.kind {
        TokenType::StringLiteral => Some((
            EvalConst::String(unquote_string_literal(&token.value)),
            index + 1,
        )),
        TokenType::Template => parse_template_constant_value(&token.value, env)
            .map(|value| (EvalConst::String(value), index + 1)),
        TokenType::NumericLiteral => token
            .value
            .parse::<i64>()
            .ok()
            .map(|value| (EvalConst::Number(value), index + 1)),
        TokenType::True => Some((EvalConst::Boolean(true), index + 1)),
        TokenType::False => Some((EvalConst::Boolean(false), index + 1)),
        TokenType::Null | TokenType::Undefined => Some((EvalConst::Null, index + 1)),
        TokenType::Identifier => env
            .get(&token.value)
            .cloned()
            .map(|value| (value, index + 1)),
        TokenType::Minus => {
            let (value, next_index) = parse_constant_primary(tokens, index + 1, env)?;
            match value {
                EvalConst::Number(number) => Some((EvalConst::Number(-number), next_index)),
                _ => None,
            }
        }
        TokenType::LeftParen => {
            let (value, next_index) = parse_constant_expression(tokens, index + 1, env)?;
            match tokens.get(next_index) {
                Some(token) if token.kind == TokenType::RightParen => Some((value, next_index + 1)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn eval_plus(left: EvalConst, right: EvalConst) -> EvalConst {
    match (left, right) {
        (EvalConst::Number(a), EvalConst::Number(b)) => EvalConst::Number(a + b),
        (left, right) => EvalConst::String(format!(
            "{}{}",
            left.to_string_value(),
            right.to_string_value()
        )),
    }
}

fn parse_template_constant_value(value: &str, env: &BTreeMap<String, EvalConst>) -> Option<String> {
    let has_interpolation = value.contains("${");
    resolve_interpolated_template_literal(value, |segment| {
        let lexer = Lexer::new(FileId::new(1), segment.to_string());
        let mut tokens = lexer.lex_all().tokens;
        while matches!(tokens.last(), Some(token) if token.kind == TokenType::Eof) {
            tokens.pop();
        }
        let (parsed, consumed) = parse_constant_expression(&tokens, 0, env)?;
        if consumed == tokens.len() {
            Some(parsed.to_string_value())
        } else {
            None
        }
    })
    .or_else(|| {
        if has_interpolation {
            None
        } else {
            Some(unquote_string_literal(value))
        }
    })
}

fn unquote_string_literal(value: &str) -> String {
    let trimmed = value.trim();
    let Some(first) = trimmed.chars().next() else {
        return trimmed.to_string();
    };
    let Some(last) = trimmed.chars().last() else {
        return trimmed.to_string();
    };

    if matches!((first, last), ('"', '"') | ('\'', '\'') | ('`', '`')) {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

struct AnalyzedSource {
    statements: Vec<kali_ast::Statement>,
    diagnostics: Vec<Diagnostic>,
}

#[allow(clippy::too_many_arguments)]
pub fn build_source_file(
    source_path: impl AsRef<Path>,
    mode: BuildMode,
    api_surface: ApiSurface,
    compat_eval: bool,
    runtime_profiles: &[String],
    max_specializations: usize,
    out_dir: Option<&Path>,
    sandbox_policy: Option<&SandboxPolicy>,
) -> Result<BuildOutput, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let mut wasm_bytes = compile_source_file(
        source_path,
        mode,
        api_surface,
        runtime_profiles,
        compat_eval,
        false,
    )?;
    let metadata = build_artifact_metadata(
        source_path,
        "executable",
        mode,
        &api_surface.to_string(),
        runtime_profiles,
        max_specializations,
        None,
        None,
    )?;
    append_metadata_section(&mut wasm_bytes, &metadata)?;

    if let Some(policy) = sandbox_policy {
        let policy_bytes = policy
            .to_embedded_json_bytes()
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

pub fn build_mode_from_flags(fast: bool, release: bool, release_advanced: bool) -> BuildMode {
    if release_advanced {
        BuildMode::ReleaseAdvanced
    } else if release {
        BuildMode::Release
    } else {
        let _ = fast;
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

pub fn validate_runtime_profiles(
    runtime_profiles: &[String],
    source_label: &str,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let mut normalized = BTreeSet::new();

    for profile in runtime_profiles {
        let profile = profile.trim();
        if profile.is_empty() {
            continue;
        }

        if !matches!(profile, "wasm-threads") {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                format!(
                    "unsupported runtimeProfile '{}' in {}",
                    profile, source_label
                ),
            )]);
        }

        if !normalized.insert(profile.to_string()) {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                format!("duplicate runtimeProfile '{}' in {}", profile, source_label),
            )]);
        }
    }

    Ok(normalized.into_iter().collect())
}

#[allow(clippy::too_many_arguments)]
pub fn build_artifact_metadata(
    source_path: &Path,
    artifact_kind: &str,
    mode: BuildMode,
    api_surface: &str,
    runtime_profiles: &[String],
    max_specializations: usize,
    profile_data: Option<&ProfileData>,
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

    let runtime_profiles = validate_runtime_profiles(
        runtime_profiles,
        &format!("artifact metadata for '{}'", source_path.display()),
    )?;

    let metadata = ArtifactMetadata {
        schema_version: 1,
        artifact_kind: artifact_kind.to_string(),
        entrypoint: source_path.to_string_lossy().to_string(),
        build_mode: build_mode_name(mode).to_string(),
        api_surface: api_surface.to_string(),
        runtime_profiles,
        max_specializations,
        host_contract: Some(
            RuntimeHostContract::KaliHosted
                .canonical_label()
                .to_string(),
        ),
        runtime_backend: Some(RuntimeBackend::Wasmtime.canonical_label().to_string()),
        kali_version: env!("CARGO_PKG_VERSION").to_string(),
        source_hash,
        profile_data_hash: profile_data_hash(profile_data),
        exports,
    };
    validate_generated_artifact_metadata(&metadata).map_err(|error| {
        vec![Diagnostic::error(
            e8::INTERNAL_ERROR as u32,
            format!(
                "generated artifact metadata for '{}' failed validation: {}",
                source_path.display(),
                error
            ),
        )]
    })?;

    Ok(metadata)
}

pub(crate) fn validate_artifact_metadata_value(value: &Value) -> Result<(), String> {
    const REQUIRED_KEYS: [&str; 7] = [
        "schemaVersion",
        "artifactKind",
        "entrypoint",
        "buildMode",
        "apiSurface",
        "kaliVersion",
        "sourceHash",
    ];
    const VALID_ARTIFACT_KINDS: [&str; 5] = ["executable", "lib", "bundle", "capi", "component"];
    const VALID_BUILD_MODES: [&str; 3] = ["fast", "release", "release-advanced"];

    let Some(object) = value.as_object() else {
        return Err("artifact metadata must be a JSON object".to_string());
    };

    for key in REQUIRED_KEYS {
        if !object.contains_key(key) {
            return Err(format!("artifact metadata is missing required key `{key}`"));
        }
    }
    validate_no_unexpected_keys(
        object,
        "artifact metadata",
        &[
            "schemaVersion",
            "artifactKind",
            "entrypoint",
            "buildMode",
            "apiSurface",
            "runtimeProfiles",
            "maxSpecializations",
            "hostContract",
            "runtimeBackend",
            "kaliVersion",
            "sourceHash",
            "profileDataHash",
            "exports",
        ],
    )?;

    match object.get("schemaVersion") {
        Some(Value::Number(number)) if number.as_u64() == Some(1) => {}
        Some(other) => {
            return Err(format!(
                "artifact metadata schemaVersion must be the numeric value 1, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    match object.get("artifactKind") {
        Some(Value::String(kind)) if VALID_ARTIFACT_KINDS.contains(&kind.as_str()) => {}
        Some(Value::String(kind)) => {
            return Err(format!("unsupported artifact metadata kind '{kind}'"));
        }
        Some(other) => {
            return Err(format!(
                "artifact metadata artifactKind must be a string, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    match object.get("entrypoint") {
        Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "artifact metadata entrypoint must be a string, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    match object.get("buildMode") {
        Some(Value::String(mode)) if VALID_BUILD_MODES.contains(&mode.as_str()) => {}
        Some(Value::String(mode)) => {
            return Err(format!("unsupported artifact metadata buildMode '{mode}'"));
        }
        Some(other) => {
            return Err(format!(
                "artifact metadata buildMode must be a string, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    match object.get("apiSurface") {
        Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "artifact metadata apiSurface must be a string, got {other}"
            ));
        }
        None => unreachable!("validated above"),
    }

    validate_sorted_string_array_value(
        object.get("runtimeProfiles"),
        "artifact metadata runtimeProfiles",
        true,
    )?;

    match object.get("maxSpecializations") {
        Some(Value::Number(number)) if number.as_u64().is_some() => {}
        Some(other) => {
            return Err(format!(
                "artifact metadata maxSpecializations must be an integer, got {other}"
            ));
        }
        None => {}
    }

    for key in [
        "hostContract",
        "runtimeBackend",
        "kaliVersion",
        "sourceHash",
        "profileDataHash",
    ] {
        if let Some(value) = object.get(key) {
            if !value.is_string() {
                return Err(format!(
                    "artifact metadata {key} must be a string, got {value}"
                ));
            }
        }
    }

    match object.get("exports") {
        Some(Value::Array(items)) => {
            let mut seen_names = std::collections::BTreeSet::new();
            for (index, item) in items.iter().enumerate() {
                let Some(export) = item.as_object() else {
                    return Err(format!(
                        "artifact metadata exports[{index}] must be an object, got {item}"
                    ));
                };
                if export.len() != 2
                    || !export.contains_key("name")
                    || !export.contains_key("signature")
                {
                    return Err(format!(
                        "artifact metadata exports[{index}] must contain only 'name' and 'signature'"
                    ));
                }
                match export.get("name") {
                    Some(Value::String(name)) => {
                        if !seen_names.insert(name.clone()) {
                            return Err(format!(
                                "artifact metadata exports[{index}].name duplicates `{name}`"
                            ));
                        }
                    }
                    Some(other) => {
                        return Err(format!(
                            "artifact metadata exports[{index}].name must be a string, got {other}"
                        ));
                    }
                    None => unreachable!("validated above"),
                }
                match export.get("signature") {
                    Some(Value::String(_)) => {}
                    Some(other) => {
                        return Err(format!(
                            "artifact metadata exports[{index}].signature must be a string, got {other}"
                        ));
                    }
                    None => unreachable!("validated above"),
                }
            }
        }
        Some(other) => {
            return Err(format!(
                "artifact metadata exports must be an array, got {other}"
            ));
        }
        None => {}
    }

    Ok(())
}

fn validate_generated_artifact_metadata(metadata: &ArtifactMetadata) -> Result<(), String> {
    let value = serde_json::to_value(metadata)
        .map_err(|error| format!("artifact metadata could not be serialized: {error}"))?;
    validate_artifact_metadata_value(&value)
}

pub fn validate_build_result_value(value: &Value) -> Result<(), String> {
    let Some(object) = value.as_object() else {
        return Err("build result must be a JSON object".to_string());
    };

    let artifact_kind = object
        .get("artifactKind")
        .and_then(Value::as_str)
        .ok_or_else(|| "build result field 'artifactKind' must be a string".to_string())?;

    for key in [
        "artifactKind",
        "outputPath",
        "sizeBytes",
        "buildMode",
        "sourceHash",
    ] {
        if !object.contains_key(key) {
            return Err(format!("build result is missing required key `{key}`"));
        }
    }

    match object.get("outputPath") {
        Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "build result outputPath must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("sizeBytes") {
        Some(Value::Number(number)) if number.as_u64().is_some() => {}
        Some(other) => {
            return Err(format!(
                "build result sizeBytes must be an integer, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("buildMode") {
        Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "build result buildMode must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    match object.get("sourceHash") {
        Some(Value::String(_)) => {}
        Some(other) => {
            return Err(format!(
                "build result sourceHash must be a string, got {other}"
            ))
        }
        None => unreachable!("validated above"),
    }

    if let Some(profile_data_hash) = object.get("profileDataHash") {
        if !profile_data_hash.is_string() {
            return Err(format!(
                "build result profileDataHash must be a string, got {profile_data_hash}"
            ));
        }
    }

    let allowed_keys: &[&str] = match artifact_kind {
        "executable" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "profileDataHash",
        ],
        "lib" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "profileDataHash",
            "metadataPath",
            "witPath",
            "artifacts",
            "exports",
        ],
        "bundle" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "profileDataHash",
            "artifacts",
            "exports",
            "bundleFormat",
        ],
        "capi" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "profileDataHash",
            "metadataPath",
            "witPath",
            "headerPath",
            "artifacts",
            "exports",
        ],
        "component" => &[
            "artifactKind",
            "outputPath",
            "sizeBytes",
            "buildMode",
            "sourceHash",
            "profileDataHash",
            "metadataPath",
            "witPath",
            "bindingPackagePath",
            "artifacts",
            "exports",
        ],
        other => return Err(format!("unsupported build result artifactKind '{other}'")),
    };
    validate_no_unexpected_keys(object, "build result", allowed_keys)?;

    match artifact_kind {
        "executable" => {}
        "lib" => {
            for key in ["metadataPath", "witPath", "artifacts", "exports"] {
                if !object.contains_key(key) {
                    return Err(format!("build result is missing required key `{key}`"));
                }
            }
            match object.get("metadataPath") {
                Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "build result metadataPath must be a string, got {other}"
                    ))
                }
                None => unreachable!("validated above"),
            }
            if let Some(wit_path) = object.get("witPath") {
                if !wit_path.is_string() {
                    return Err(format!(
                        "build result witPath must be a string, got {wit_path}"
                    ));
                }
            }
            validate_build_result_artifacts_array(
                object.get("artifacts"),
                "build result artifacts",
            )?;
            validate_build_result_exports_array(object.get("exports"), "build result exports")?;
        }
        "bundle" => {
            for key in ["artifacts", "exports", "bundleFormat"] {
                if !object.contains_key(key) {
                    return Err(format!("build result is missing required key `{key}`"));
                }
            }
            validate_build_result_artifacts_array(
                object.get("artifacts"),
                "build result artifacts",
            )?;
            validate_build_result_exports_array(object.get("exports"), "build result exports")?;
            match object.get("bundleFormat") {
                Some(Value::String(format)) if matches!(format.as_str(), "esm" | "cjs") => {}
                Some(Value::String(format)) => {
                    return Err(format!("unsupported build result bundleFormat '{format}'"));
                }
                Some(other) => {
                    return Err(format!(
                        "build result bundleFormat must be a string, got {other}"
                    ));
                }
                None => unreachable!("validated above"),
            }
        }
        "capi" => {
            for key in [
                "metadataPath",
                "witPath",
                "headerPath",
                "artifacts",
                "exports",
            ] {
                if !object.contains_key(key) {
                    return Err(format!("build result is missing required key `{key}`"));
                }
            }
            match object.get("metadataPath") {
                Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "build result metadataPath must be a string, got {other}"
                    ))
                }
                None => unreachable!("validated above"),
            }
            match object.get("headerPath") {
                Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "build result headerPath must be a string, got {other}"
                    ))
                }
                None => unreachable!("validated above"),
            }
            match object.get("witPath") {
                Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "build result witPath must be a string, got {other}"
                    ))
                }
                None => unreachable!("validated above"),
            }
            validate_build_result_artifacts_array(
                object.get("artifacts"),
                "build result artifacts",
            )?;
            validate_build_result_exports_array(object.get("exports"), "build result exports")?;
        }
        "component" => {
            for key in [
                "metadataPath",
                "witPath",
                "bindingPackagePath",
                "artifacts",
                "exports",
            ] {
                if !object.contains_key(key) {
                    return Err(format!("build result is missing required key `{key}`"));
                }
            }
            match object.get("metadataPath") {
                Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "build result metadataPath must be a string, got {other}"
                    ))
                }
                None => unreachable!("validated above"),
            }
            match object.get("witPath") {
                Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "build result witPath must be a string, got {other}"
                    ))
                }
                None => unreachable!("validated above"),
            }
            match object.get("bindingPackagePath") {
                Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(format!(
                        "build result bindingPackagePath must be a string, got {other}"
                    ))
                }
                None => unreachable!("validated above"),
            }
            validate_build_result_artifacts_array(
                object.get("artifacts"),
                "build result artifacts",
            )?;
            validate_build_result_exports_array(object.get("exports"), "build result exports")?;
        }
        other => return Err(format!("unsupported build result artifactKind '{other}'")),
    }

    Ok(())
}

fn validate_build_result_artifacts_array(
    value: Option<&Value>,
    context: &str,
) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    let mut seen_primary_executable = false;
    let mut seen_primary_library = false;
    let mut seen_primary_component = false;

    let mut seen_kind_path_pairs = std::collections::BTreeSet::new();

    for (index, item) in items.iter().enumerate() {
        let Some(object) = item.as_object() else {
            return Err(format!("{context}[{index}] must be an object, got {item}"));
        };

        validate_no_unexpected_keys(
            object,
            &format!("{context}[{index}]"),
            &["kind", "path", "role"],
        )?;

        match object.get("kind") {
            Some(Value::String(_)) => {}
            Some(other) => {
                return Err(format!(
                    "{context}[{index}].kind must be a string, got {other}"
                ))
            }
            None => return Err(format!("{context}[{index}] is missing required key `kind`")),
        }
        match object.get("path") {
            Some(Value::String(_)) => {}
            Some(other) => {
                return Err(format!(
                    "{context}[{index}].path must be a string, got {other}"
                ))
            }
            None => return Err(format!("{context}[{index}] is missing required key `path`")),
        }

        let kind = object
            .get("kind")
            .and_then(Value::as_str)
            .expect("validated above");
        let path = object
            .get("path")
            .and_then(Value::as_str)
            .expect("validated above");
        if !seen_kind_path_pairs.insert((kind.to_string(), path.to_string())) {
            return Err(format!(
                "{context}[{index}] duplicates artifact `{kind}` at `{path}`"
            ));
        }

        if let Some(role) = object.get("role") {
            if !role.is_string() {
                return Err(format!(
                    "{context}[{index}].role must be a string, got {role}"
                ));
            }
            match role.as_str().unwrap() {
                "primary-executable" => {
                    if seen_primary_executable {
                        return Err(format!(
                            "{context}[{index}].role duplicates primary-executable"
                        ));
                    }
                    seen_primary_executable = true;
                }
                "primary-library" => {
                    if seen_primary_library {
                        return Err(format!(
                            "{context}[{index}].role duplicates primary-library"
                        ));
                    }
                    seen_primary_library = true;
                }
                "primary-component" => {
                    if seen_primary_component {
                        return Err(format!(
                            "{context}[{index}].role duplicates primary-component"
                        ));
                    }
                    seen_primary_component = true;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn validate_build_result_exports_array(value: Option<&Value>, context: &str) -> Result<(), String> {
    let Some(Value::Array(items)) = value else {
        return Err(format!("{context} must be an array"));
    };

    let mut seen_names = std::collections::BTreeSet::new();

    for (index, item) in items.iter().enumerate() {
        let Some(object) = item.as_object() else {
            return Err(format!("{context}[{index}] must be an object, got {item}"));
        };
        if object.len() != 2 || !object.contains_key("name") || !object.contains_key("signature") {
            return Err(format!(
                "{context}[{index}] must contain only 'name' and 'signature'"
            ));
        }
        match object.get("name") {
            Some(Value::String(name)) => {
                if !seen_names.insert(name.clone()) {
                    return Err(format!("{context}[{index}].name duplicates `{name}`"));
                }
            }
            Some(other) => {
                return Err(format!(
                    "{context}[{index}].name must be a string, got {other}"
                ))
            }
            None => return Err(format!("{context}[{index}] is missing required key `name`")),
        }
        match object.get("signature") {
            Some(Value::String(_)) => {}
            Some(other) => {
                return Err(format!(
                    "{context}[{index}].signature must be a string, got {other}"
                ))
            }
            None => {
                return Err(format!(
                    "{context}[{index}] is missing required key `signature`"
                ))
            }
        }
    }

    Ok(())
}

fn validate_no_unexpected_keys(
    object: &serde_json::Map<String, Value>,
    context: &str,
    allowed_keys: &[&str],
) -> Result<(), String> {
    for key in object.keys() {
        if !allowed_keys.contains(&key.as_str()) {
            return Err(format!("{context} has unexpected key `{key}`"));
        }
    }

    Ok(())
}

pub fn serialize_artifact_metadata(metadata: &ArtifactMetadata) -> Vec<u8> {
    validate_generated_artifact_metadata(metadata)
        .expect("serialized artifact metadata must satisfy schema-v1 shape");
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

pub fn browser_bundle_source_map(
    source_path: &Path,
    js_path: &Path,
    source_contents: &str,
    exports: &[LibraryExport],
) -> String {
    let source_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("input.ts")
        .to_string();
    let js_name = js_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("bundle.js")
        .to_string();
    let names: Vec<String> = exports.iter().map(|export| export.name.clone()).collect();
    json!({
        "version": 3,
        "file": js_name,
        "sourceRoot": "",
        "sources": [source_name],
        "sourcesContent": [source_contents],
        "names": names,
        "mappings": "",
    })
    .to_string()
}

pub fn library_wit_for(module_name: &str, exports: &[LibraryExport]) -> String {
    let mut wit = String::from("package kali:embed;\n\nworld library {\n");
    wit.push_str(&format!("  // module: {}\n", module_name));
    for export in exports {
        wit.push_str(&format!(
            "  // signature: {}\n  export {}: func();\n",
            export.signature,
            sanitize_wit_identifier(&export.name)
        ));
    }
    wit.push_str("}\n");
    wit
}

pub fn collect_library_exports(
    source_path: impl AsRef<Path>,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let parsed = parse_source_file(source_path)?;

    let mut resolver = TypeContext::with_base_path_and_api_surface_and_runtime_profiles(
        source_path,
        api_surface.to_string(),
        runtime_profiles.to_vec(),
    );
    let resolved = resolver.resolve_statements_in_file(source_path, &parsed);
    if has_errors(&resolved.diagnostics) {
        return Err(resolved.diagnostics);
    }

    collect_library_exports_from_statements(&parsed, source_path)
}

pub fn collect_browser_bundle_exports(
    source_path: impl AsRef<Path>,
    tree_shake: bool,
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let source_path = source_path.as_ref();
    let statements = parse_source_file(source_path)?;
    let exports = collect_library_exports_from_statements(&statements, source_path)?;

    if !tree_shake {
        return Ok(exports);
    }

    let candidate_names = exports
        .iter()
        .map(|export| export.name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let comment_roots = collect_tree_shake_roots(source_path)?;
    let reachable = collect_reachable_bundle_exports(&statements, &candidate_names, &comment_roots);

    if reachable.is_empty() {
        return Ok(Vec::new());
    }

    let filtered = exports
        .into_iter()
        .filter(|export| reachable.contains(&export.name))
        .collect::<Vec<_>>();

    Ok(filtered)
}

fn parse_source_file(source_path: &Path) -> Result<Vec<Statement>, Vec<Diagnostic>> {
    let source = read_compiler_source_file(source_path)?;

    let lexer = Lexer::new(FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let parsed = parser.parse(Some(source_path.to_string_lossy().to_string()));
    let diagnostics = parsed.diagnostics;

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    Ok(parsed.statements)
}

fn validate_unique_export_names_from_statements(
    statements: &[Statement],
    source_path: &Path,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen_names = BTreeSet::<String>::new();

    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(func) => {
                if !seen_names.insert(func.name.clone()) {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        &format!("duplicate export name `{}`", func.name),
                    ));
                }
            }
            Statement::ExportNamed(declaration) => {
                for specifier in &declaration.specifiers {
                    if !seen_names.insert(specifier.exported.clone()) {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{}`", specifier.exported),
                        ));
                    }
                }
            }
            Statement::ExportDefault(default_decl) => match default_decl {
                ExportDefaultDeclaration::FunctionDeclaration(func) => {
                    let export_name = if func.name.is_empty() {
                        "default".to_string()
                    } else {
                        func.name.clone()
                    };
                    if !seen_names.insert(export_name.clone()) {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{export_name}`"),
                        ));
                    }
                }
                ExportDefaultDeclaration::Expression(_)
                | ExportDefaultDeclaration::ClassDeclaration(_) => {
                    if !seen_names.insert("default".to_string()) {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            "duplicate export name `default`",
                        ));
                    }
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

    diagnostics
}

fn collect_library_exports_from_statements(
    statements: &[Statement],
    source_path: &Path,
) -> Result<Vec<LibraryExport>, Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut exports = BTreeMap::<String, String>::new();
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(func) => {
                let signature = function_signature(&func.params);
                if exports.insert(func.name.clone(), signature).is_some() {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        &format!("duplicate export name `{}`", func.name),
                    ));
                }
            }
            Statement::ExportNamed(declaration) => {
                if declaration.source.is_some() {
                    diagnostics.push(invalid_export_surface(
                        source_path,
                        "re-exported surfaces are not statically known yet",
                    ));
                    continue;
                }

                for specifier in &declaration.specifiers {
                    let signature = signature_from_export_specifier(&specifier.local);
                    if exports
                        .insert(specifier.exported.clone(), signature)
                        .is_some()
                    {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{}`", specifier.exported),
                        ));
                    }
                }
            }
            Statement::ExportDefault(default_decl) => match default_decl {
                ExportDefaultDeclaration::FunctionDeclaration(func) => {
                    let export_name = if func.name.is_empty() {
                        "default".to_string()
                    } else {
                        func.name.clone()
                    };
                    if exports
                        .insert(export_name.clone(), function_signature(&func.params))
                        .is_some()
                    {
                        diagnostics.push(invalid_export_surface(
                            source_path,
                            &format!("duplicate export name `{export_name}`"),
                        ));
                    }
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

fn collect_tree_shake_roots(
    source_path: &Path,
) -> Result<std::collections::BTreeSet<String>, Vec<Diagnostic>> {
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

    let mut roots = std::collections::BTreeSet::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("// kali-tree-shake:") {
            for name in rest.split(|ch: char| ch == ',' || ch.is_whitespace()) {
                let name = name.trim();
                if !name.is_empty() {
                    roots.insert(name.to_string());
                }
            }
        }
    }

    Ok(roots)
}

fn collect_reachable_bundle_exports(
    statements: &[Statement],
    candidate_names: &std::collections::BTreeSet<String>,
    initial_roots: &std::collections::BTreeSet<String>,
) -> std::collections::BTreeSet<String> {
    let function_map = statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::FunctionDeclaration(function)
                if candidate_names.contains(&function.name) =>
            {
                Some((function.name.clone(), function))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();

    let mut reachable = std::collections::BTreeSet::new();
    let mut worklist = collect_direct_bundle_calls_from_statements(statements, candidate_names);
    worklist.extend(initial_roots.iter().cloned());
    if candidate_names.contains("main") {
        worklist.insert("main".to_string());
    }
    while let Some(name) = worklist.pop_first() {
        if !reachable.insert(name.clone()) {
            continue;
        }

        if let Some(function) = function_map.get(&name) {
            collect_direct_bundle_calls_from_statements(&function.body.body, candidate_names)
                .into_iter()
                .filter(|dep| !reachable.contains(dep))
                .for_each(|dep| {
                    worklist.insert(dep);
                });
        }
    }

    reachable
}

fn collect_direct_bundle_calls_from_statements(
    statements: &[Statement],
    candidate_names: &std::collections::BTreeSet<String>,
) -> std::collections::BTreeSet<String> {
    let mut calls = std::collections::BTreeSet::new();
    for statement in statements {
        collect_direct_bundle_calls_from_statement(statement, candidate_names, &mut calls);
    }
    calls
}

fn collect_direct_bundle_calls_from_statement(
    statement: &Statement,
    candidate_names: &std::collections::BTreeSet<String>,
    calls: &mut std::collections::BTreeSet<String>,
) {
    match statement {
        Statement::ExpressionStatement(expr) => collect_direct_bundle_calls_from_expression(
            expr.expression.as_ref(),
            candidate_names,
            calls,
        ),
        Statement::ReturnStatement(statement) => {
            if let Some(argument) = &statement.argument {
                collect_direct_bundle_calls_from_expression(argument, candidate_names, calls);
            }
        }
        Statement::ThrowStatement(statement) => {
            collect_direct_bundle_calls_from_expression(
                &statement.argument,
                candidate_names,
                calls,
            );
        }
        Statement::WithStatement(statement) => {
            collect_direct_bundle_calls_from_expression(&statement.object, candidate_names, calls);
            collect_direct_bundle_calls_from_statement(
                statement.body.as_ref(),
                candidate_names,
                calls,
            );
        }
        Statement::LabeledStatement(statement) => {
            collect_direct_bundle_calls_from_statement(
                statement.body.as_ref(),
                candidate_names,
                calls,
            );
        }
        Statement::IfStatement(statement) => {
            collect_direct_bundle_calls_from_expression(&statement.test, candidate_names, calls);
            collect_direct_bundle_calls_from_block(
                statement.consequent.as_ref(),
                candidate_names,
                calls,
            );
            if let Some(alternate) = &statement.alternate {
                collect_direct_bundle_calls_from_block(alternate.as_ref(), candidate_names, calls);
            }
        }
        Statement::SwitchStatement(statement) => {
            collect_direct_bundle_calls_from_expression(
                &statement.discriminant,
                candidate_names,
                calls,
            );
            for case in &statement.cases {
                if let Some(test) = &case.test {
                    collect_direct_bundle_calls_from_expression(test, candidate_names, calls);
                }
                for inner in &case.consequent {
                    collect_direct_bundle_calls_from_statement(inner, candidate_names, calls);
                }
            }
        }
        Statement::TryStatement(statement) => {
            collect_direct_bundle_calls_from_block(
                statement.block.as_ref(),
                candidate_names,
                calls,
            );
            if let Some(handler) = &statement.handler {
                collect_direct_bundle_calls_from_block(
                    handler.body.as_ref(),
                    candidate_names,
                    calls,
                );
            }
            if let Some(finalizer) = &statement.finalizer {
                collect_direct_bundle_calls_from_block(finalizer, candidate_names, calls);
            }
        }
        Statement::BlockStatement(block) => {
            collect_direct_bundle_calls_from_block(block, candidate_names, calls)
        }
        Statement::ForStatement(statement) => {
            if let Some(init) = &statement.init {
                match init {
                    kali_ast::ForInit::Expression(expression) => {
                        collect_direct_bundle_calls_from_expression(
                            expression,
                            candidate_names,
                            calls,
                        )
                    }
                    kali_ast::ForInit::VariableDeclaration(declaration) => {
                        collect_direct_bundle_calls_from_variable_declaration(
                            declaration,
                            candidate_names,
                            calls,
                        )
                    }
                }
            }
            if let Some(test) = &statement.test {
                collect_direct_bundle_calls_from_expression(test, candidate_names, calls);
            }
            if let Some(update) = &statement.update {
                collect_direct_bundle_calls_from_expression(update, candidate_names, calls);
            }
            collect_direct_bundle_calls_from_block(statement.body.as_ref(), candidate_names, calls);
        }
        Statement::ForInStatement(statement) => {
            match &statement.left {
                kali_ast::ForInLefthand::Expression(expression) => {
                    collect_direct_bundle_calls_from_expression(expression, candidate_names, calls)
                }
                kali_ast::ForInLefthand::VariableDeclaration(declaration) => {
                    collect_direct_bundle_calls_from_variable_declaration(
                        declaration,
                        candidate_names,
                        calls,
                    )
                }
            }
            collect_direct_bundle_calls_from_expression(&statement.right, candidate_names, calls);
            collect_direct_bundle_calls_from_statement(
                statement.body.as_ref(),
                candidate_names,
                calls,
            );
        }
        Statement::ForOfStatement(statement) => {
            match &statement.left {
                kali_ast::ForOfLefthand::Expression(expression) => {
                    collect_direct_bundle_calls_from_expression(expression, candidate_names, calls)
                }
                kali_ast::ForOfLefthand::VariableDeclaration(declaration) => {
                    collect_direct_bundle_calls_from_variable_declaration(
                        declaration,
                        candidate_names,
                        calls,
                    )
                }
            }
            collect_direct_bundle_calls_from_expression(&statement.right, candidate_names, calls);
            collect_direct_bundle_calls_from_statement(
                statement.body.as_ref(),
                candidate_names,
                calls,
            );
        }
        Statement::WhileStatement(statement) => {
            collect_direct_bundle_calls_from_expression(&statement.test, candidate_names, calls);
            collect_direct_bundle_calls_from_block(statement.body.as_ref(), candidate_names, calls);
        }
        Statement::DoWhileStatement(statement) => {
            collect_direct_bundle_calls_from_block(statement.body.as_ref(), candidate_names, calls);
            collect_direct_bundle_calls_from_expression(&statement.test, candidate_names, calls);
        }
        Statement::VariableDeclaration(statement) => {
            collect_direct_bundle_calls_from_variable_declaration(statement, candidate_names, calls)
        }
        Statement::EnumDeclaration(statement) => {
            for member in &statement.members {
                if let Some(value) = &member.value {
                    collect_direct_bundle_calls_from_expression(value, candidate_names, calls);
                }
            }
        }
        Statement::FunctionDeclaration(_)
        | Statement::ClassDeclaration(_)
        | Statement::ImportDeclaration(_)
        | Statement::BreakStatement(_)
        | Statement::ContinueStatement(_)
        | Statement::DebuggerStatement(_)
        | Statement::TypeAliasDeclaration(_)
        | Statement::InterfaceDeclaration(_)
        | Statement::ExportNamed(_)
        | Statement::ExportDefault(_) => {}
    }
}

fn collect_direct_bundle_calls_from_block(
    block: &kali_ast::BlockStatement,
    candidate_names: &std::collections::BTreeSet<String>,
    calls: &mut std::collections::BTreeSet<String>,
) {
    for statement in &block.body {
        collect_direct_bundle_calls_from_statement(statement, candidate_names, calls);
    }
}

fn collect_direct_bundle_calls_from_variable_declaration(
    declaration: &kali_ast::VariableDeclaration,
    candidate_names: &std::collections::BTreeSet<String>,
    calls: &mut std::collections::BTreeSet<String>,
) {
    for declarator in &declaration.declarations {
        if let Some(init) = &declarator.init {
            collect_direct_bundle_calls_from_expression(init, candidate_names, calls);
        }
    }
}

fn collect_direct_bundle_calls_from_expression(
    expression: &Expression,
    candidate_names: &std::collections::BTreeSet<String>,
    calls: &mut std::collections::BTreeSet<String>,
) {
    match expression {
        Expression::Identifier(name) if candidate_names.contains(name) => {
            calls.insert(name.clone());
        }
        Expression::Identifier(_) => {}
        Expression::Literal(_)
        | Expression::ThisExpression
        | Expression::SuperExpression
        | Expression::PrivateIdentifier(_)
        | Expression::BigIntLiteral(_)
        | Expression::JsxEmptyExpression => {}
        Expression::BinaryExpression(expr) => {
            collect_direct_bundle_calls_from_expression(&expr.left, candidate_names, calls);
            collect_direct_bundle_calls_from_expression(&expr.right, candidate_names, calls);
        }
        Expression::UnaryExpression(expr) => {
            collect_direct_bundle_calls_from_expression(&expr.argument, candidate_names, calls);
        }
        Expression::CallExpression(expr) => {
            if let Expression::Identifier(name) = expr.callee.as_ref() {
                if candidate_names.contains(name) {
                    calls.insert(name.clone());
                }
            }
            collect_direct_bundle_calls_from_expression(
                expr.callee.as_ref(),
                candidate_names,
                calls,
            );
            for arg in &expr.args {
                collect_direct_bundle_calls_from_expression(arg, candidate_names, calls);
            }
        }
        Expression::MemberExpression(expr) => {
            collect_direct_bundle_calls_from_expression(&expr.object, candidate_names, calls);
        }
        Expression::ArrayExpression(expr) => {
            for element in &expr.elements {
                match element {
                    Some(kali_ast::ExpressionOrSpread::Expression(value)) => {
                        collect_direct_bundle_calls_from_expression(value, candidate_names, calls)
                    }
                    Some(kali_ast::ExpressionOrSpread::Spread(spread)) => {
                        collect_direct_bundle_calls_from_expression(
                            &spread.argument,
                            candidate_names,
                            calls,
                        )
                    }
                    Some(kali_ast::ExpressionOrSpread::Empty) | None => {}
                }
            }
        }
        Expression::ObjectExpression(expr) => {
            for property in &expr.properties {
                collect_direct_bundle_calls_from_expression(
                    &property.value,
                    candidate_names,
                    calls,
                );
            }
        }
        Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::NewExpression(_)
        | Expression::MetaProperty(_)
        | Expression::TemplateLiteral(_)
        | Expression::TaggedTemplateExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::AssignmentExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::ConditionalExpression(_)
        | Expression::SequenceExpression(_)
        | Expression::ParenthesizedExpression(_)
        | Expression::YieldExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::OptionalChainExpression(_)
        | Expression::ChainExpression(_)
        | Expression::SpreadElement(_)
        | Expression::RestElement(_)
        | Expression::ImportExpression(_)
        | Expression::DecoratedExpression(_)
        | Expression::JsxElement(_)
        | Expression::JsxFragment(_)
        | Expression::TypeAssertion(_)
        | Expression::SatisfiesExpression(_) => {}
    }
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

fn sanitize_wit_identifier(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        let keep = ch.is_ascii_alphanumeric() || ch == '_';
        if index == 0 && ch.is_ascii_digit() {
            out.push('_');
            out.push(ch);
        } else if keep {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.is_error())
}

#[cfg(test)]
#[path = "build_tests.rs"]
mod tests;
