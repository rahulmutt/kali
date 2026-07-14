//! Compile pipeline, source analysis, incremental cache, profile data.

use super::entrypoint::validate_unique_export_names_from_statements;
use super::eval::{rewrite_eval_compat_source, source_uses_eval_compat};
use super::helpers::*;
use super::metadata::{append_metadata_section, build_artifact_metadata};
use super::paths::executable_output_path_for;

use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use kali_codegen::{lower_lir_to_wasm, CodegenCtx, TargetConfig};
use kali_common::FileId;
use kali_error::{_error_codes::e5, _error_codes::e8, Diagnostic};
use kali_hir::HirLowerer;
use kali_lexer::Lexer;
use kali_lir::LirLowerer;
use kali_mir::MirLowerer;
use kali_optimize::{OptimizationLevel, Optimizer, ProfileData, PROFILE_DATA_VERSION};
use kali_parser::Parser;
use kali_runtime::normalize_runtime_profiles;
use kali_sandbox::SandboxPolicy;
use kali_types::TypeContext;
use wasm_encoder::{CustomSection, Section};

use crate::{is_declaration_only_source_file, ApiSurface};

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

pub fn check_source_file(
    source_path: impl AsRef<Path>,
    api_surface: ApiSurface,
    runtime_profiles: &[String],
    compat_eval: bool,
    sandbox_policy_attached: bool,
) -> Result<(), Vec<Diagnostic>> {
    let _analysis = analyze_source_file(
        source_path.as_ref(),
        api_surface,
        runtime_profiles,
        compat_eval,
        sandbox_policy_attached,
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

    profile_data.validate().map_err(|error| {
        vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            format!(
                "failed to validate PGO profile data '{}': {}",
                profile_path.display(),
                error
            ),
        )]
    })?;

    Ok(profile_data.normalized())
}

pub(crate) fn profile_data_hash(profile_data: Option<&ProfileData>) -> Option<String> {
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
    sandbox_policy_attached: bool,
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
            sandbox_policy_attached,
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
            sandbox_policy_attached,
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
        false,
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
    sandbox_policy_attached: bool,
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
        sandbox_policy_attached,
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
    sandbox_policy_attached: bool,
    validate_ir: bool,
    coverage: bool,
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let analyzed = analyze_source_file(
        source_path.as_ref(),
        api_surface,
        runtime_profiles,
        compat_eval,
        sandbox_policy_attached,
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
    let optimizer = Optimizer::with_max_specializations(optimization_level, max_specializations)
        // Growable (push-accumulated) array bindings mutate through `.push`
        // (throw-fallout Stage 4): the optimizer must treat them as mutated,
        // else its constant-binding envs inline/fold the stale declarator
        // literal (`o.length` → 0, `o[0]` → undefined) over the runtime
        // contents.
        .with_growable_array_bindings(analyzed.repr_table.growable_array_binding_names());
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
    ctx.repr_table = analyzed.repr_table.clone();
    ctx.arena_table = kali_mir::analysis::arena_gate::compute_arena_table(&mir);
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
pub(crate) fn incremental_cache_path(
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
    sandbox_policy_attached: bool,
) -> Result<AnalyzedSource, Vec<Diagnostic>> {
    let source = read_compiler_source_file(source_path)?;

    let source = if compat_eval {
        rewrite_eval_compat_source(&source)
    } else {
        source
    };

    if api_surface != ApiSurface::Node && source_uses_process_env_mutation(&source) {
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

    if source_uses_optional_chain_math_pow(&source) {
        return Err(vec![Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "Math.pow is unavailable through optional-chain wrappers in the current phase; use a direct call or the later compatibility path",
        )]);
    }

    let lexer = Lexer::new(FileId::new(0), source.clone());
    let lexed = lexer.lex_all();
    let mut diagnostics = lexed.diagnostics;
    let mut parser = Parser::new(FileId::new(0), lexed.tokens);
    let mut parsed = parser.parse(Some(source_path.to_string_lossy().to_string()));
    diagnostics.extend(parsed.diagnostics);

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

    // AST module-linking (throw-fallout Stage 5). Runs BEFORE monomorphize so
    // linked functions participate in specialization, and AFTER export-name
    // validation (mangled `__link` names must not collide — the pass
    // re-checks). No provenance found (the overwhelming majority of sources)
    // is a guaranteed no-op.
    crate::build::module_link::link_provable_module_namespaces(
        source_path,
        &source,
        &mut parsed.statements,
        &mut diagnostics,
    );
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    // Object-shape monomorphization (fasta Spec 5). Runs AFTER the export-name
    // uniqueness check (so the fresh clone names are never treated as public
    // exports / duplicate names) and BEFORE the resolver → repr_infer, which
    // then sees each specialized clone as a separate monomorphic function. An
    // empty plan (no function reached by ≥2 distinct object-param shapes — true
    // for every current fixture/test) is a hard no-op: the statements pass
    // through byte-identical.
    kali_types::monomorphize::monomorphize_statements(&mut parsed.statements);

    // Name every anonymous function-shaped node BEFORE the resolver, so kali_types
    // (binding_repr_function_key) and kali_hir (synthetic function names) key on the
    // SAME name. Re-deriving HIR's counter inside kali_types would be a
    // hand-mirrored oracle — this repo has shipped two of those, and both failed OPEN.
    // Runs AFTER monomorphize (not before): monomorphize clones whole function ASTs
    // to specialize them, and naming first would mean every clone's inner anonymous
    // nodes shared the SAME `__kali_fn_N` id — a collision this pass exists to
    // prevent, not create. Running after means each specialized clone gets its own,
    // independently-numbered names.
    crate::build::name_anon_functions::name_anonymous_functions(&mut parsed.statements);

    let mut repr_table = kali_common::ReprTable::default();
    if !is_declaration_only_source_file(source_path) {
        let mut resolver = TypeContext::with_base_path_and_api_surface_and_runtime_profiles(
            source_path,
            api_surface.to_string(),
            runtime_profiles.to_vec(),
        );
        resolver.set_sandbox_policy_attached(sandbox_policy_attached);
        let resolved = resolver.resolve_statements_in_file(source_path, &parsed.statements);
        diagnostics.extend(resolved.diagnostics);
        if has_errors(&diagnostics) {
            return Err(diagnostics);
        }
        repr_table = resolved.repr_table;

        for message in repr_table.shape_conflicts() {
            diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                message.clone(),
            ));
        }
        if has_errors(&diagnostics) {
            return Err(diagnostics);
        }
    }

    Ok(AnalyzedSource {
        statements: parsed.statements,
        diagnostics,
        repr_table,
    })
}

fn source_uses_optional_chain_math_pow(source: &str) -> bool {
    let patterns = [
        "?.Math.pow",
        "?.Math[\"pow\"]",
        "?.Math['pow']",
        "?.Math?.pow",
        "?.Math?.[\"pow\"]",
        "?.Math?.['pow']",
        "?.[\"Math\"].pow",
        "?.[\"Math\"][\"pow\"]",
        "?.[\"Math\"]['pow']",
        "?.[\"Math\"]?.pow",
        "?.[\"Math\"]?.[\"pow\"]",
        "?.[\"Math\"]?.['pow']",
        "?.['Math'].pow",
        "?.['Math']['pow']",
        "?.['Math']?.pow",
        "?.['Math']?.[\"pow\"]",
        "?.['Math']?.['pow']",
        "Math?.pow",
        "Math?.[\"pow\"]",
        "Math?.['pow']",
        "globalThis.Math?.pow",
        "globalThis.Math?.[\"pow\"]",
        "globalThis.Math?.['pow']",
        "globalThis[\"Math\"]?.pow",
        "globalThis[\"Math\"]?.[\"pow\"]",
        "globalThis[\"Math\"]?.['pow']",
        "globalThis['Math']?.pow",
        "globalThis['Math']?.[\"pow\"]",
        "globalThis['Math']?.['pow']",
        "globalThis?.Math?.pow",
        "globalThis?.Math?.[\"pow\"]",
        "globalThis?.Math?.['pow']",
        "globalThis?.Math[\"pow\"]",
        "globalThis?.Math['pow']",
        "globalThis?.[\"Math\"].pow",
        "globalThis?.[\"Math\"][\"pow\"]",
        "globalThis?.[\"Math\"]['pow']",
        "globalThis?.['Math'].pow",
        "globalThis?.['Math'][\"pow\"]",
        "globalThis?.['Math']['pow']",
        "globalThis?.[\"Math\"]?.pow",
        "globalThis?.[\"Math\"]?.[\"pow\"]",
        "globalThis?.[\"Math\"]?.['pow']",
        "globalThis?.['Math']?.pow",
        "globalThis?.['Math']?.[\"pow\"]",
        "globalThis?.['Math']?.['pow']",
    ];

    patterns.iter().any(|pattern| source.contains(pattern))
}

fn source_uses_process_env_mutation(source: &str) -> bool {
    let patterns = [
        "process.env =",
        "process.env.",
        "process.env[",
        "globalThis.process.env =",
        "globalThis.process.env.",
        "globalThis.process.env[",
        "process[\"env\"] =",
        "process[\"env\"].",
        "process[\"env\"][",
        "process['env'] =",
        "process['env'].",
        "process['env'][",
        "globalThis.process[\"env\"] =",
        "globalThis.process[\"env\"].",
        "globalThis.process[\"env\"][",
        "globalThis.process['env'] =",
        "globalThis.process['env'].",
        "globalThis.process['env'][",
        "globalThis[\"process\"].env =",
        "globalThis[\"process\"].env.",
        "globalThis[\"process\"].env[",
        "globalThis[\"process\"][\"env\"] =",
        "globalThis[\"process\"][\"env\"].",
        "globalThis[\"process\"][\"env\"][",
        "globalThis['process'].env =",
        "globalThis['process'].env.",
        "globalThis['process'].env[",
        "globalThis['process']['env'] =",
        "globalThis['process']['env'].",
        "globalThis['process']['env'][",
        "delete process.env",
        "delete globalThis.process.env",
        "delete process[\"env\"]",
        "delete process['env']",
        "delete globalThis.process[\"env\"]",
        "delete globalThis.process['env']",
        "delete globalThis[\"process\"].env",
        "delete globalThis['process'].env",
        "delete globalThis[\"process\"][\"env\"]",
        "delete globalThis['process']['env']",
    ];

    patterns.iter().any(|pattern| source.contains(pattern))
}

struct AnalyzedSource {
    statements: Vec<kali_ast::Statement>,
    diagnostics: Vec<Diagnostic>,
    repr_table: kali_common::ReprTable,
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

pub(crate) fn source_hash_for_file(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("sha256-{:x}", hasher.finalize()))
}

pub(crate) fn build_mode_name(mode: BuildMode) -> &'static str {
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

    for (index, profile) in runtime_profiles.iter().enumerate() {
        let profile = profile.trim();
        if profile.is_empty() {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                format!(
                    "empty or whitespace-only runtimeProfile at index {index} in {}",
                    source_label
                ),
            )]);
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

#[cfg(test)]
mod module_link_wiring_tests {
    // Exercises `analyze_source_file`'s ACTUAL wiring of
    // `module_link::link_provable_module_namespaces` (throw-fallout Stage 5
    // Task 7) — the module_link.rs unit tests call the pass function
    // directly and never touch this file's insertion point, so they cannot
    // catch a wiring mistake here (call omitted, wrong argument, wrong
    // ordering relative to `monomorphize_statements`/export-name
    // validation). These two tests compile through the REAL
    // `analyze_source_file` entry point instead.
    use super::*;
    use kali_ast::{Expression, Statement};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn analyze_source_file_links_namespace_import_end_to_end() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("util.js"),
            "export function greet() { return 'hi'; }\n",
        )
        .expect("write util.js");
        let main_path = dir.path().join("main.js");
        fs::write(
            &main_path,
            r#"import * as ns from "./util.js";
console.log(ns.greet());
"#,
        )
        .expect("write main.js");

        let analyzed = analyze_source_file(&main_path, ApiSurface::Node, &[], false, false)
            .expect("compilation must succeed: the namespace import is fully provable");

        // Proves the wiring actually ran (not just that the source happened
        // to compile for an unrelated reason): the mangled linked clone
        // must be present in the analyzed statements, and the call site
        // must reference it — exactly what `link_provable_module_namespaces`
        // is responsible for producing before `monomorphize_statements` runs.
        let has_linked_clone = analyzed.statements.iter().any(|statement| {
            matches!(
                statement,
                Statement::FunctionDeclaration(function) if function.name == "__link0_greet"
            )
        });
        assert!(
            has_linked_clone,
            "expected the wiring to append `__link0_greet`, got {:#?}",
            analyzed.statements
        );
        let call_rewritten = analyzed.statements.iter().any(|statement| match statement {
            Statement::ExpressionStatement(stmt) => matches!(
                stmt.expression.as_ref(),
                Expression::CallExpression(call)
                    if matches!(
                        &call.args.first(),
                        Some(Expression::CallExpression(inner))
                            if inner.callee == Expression::Identifier("__link0_greet".to_string())
                    )
            ),
            _ => false,
        });
        assert!(
            call_rewritten,
            "expected `ns.greet()` to be rewritten to `__link0_greet()`, got {:#?}",
            analyzed.statements
        );
    }

    #[test]
    fn analyze_source_file_denies_leftover_namespace_binding_reference() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("lazy.js"),
            "export function lazyValue() { return 7; }\n",
        )
        .expect("write lazy.js");
        let main_path = dir.path().join("main.js");
        fs::write(
            &main_path,
            r#"async function main() {
    const chunk = await import("./lazy.js");
    console.log(chunk);
}
main();
"#,
        )
        .expect("write main.js");

        let diagnostics = match analyze_source_file(&main_path, ApiSurface::Node, &[], false, false)
        {
            Ok(_) => panic!("a leftover bare `chunk` value use must fail the build"),
            Err(diagnostics) => diagnostics,
        };

        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && d.message.contains("chunk")),
            "expected an E5506 naming `chunk`, got {diagnostics:?}"
        );
    }
}
