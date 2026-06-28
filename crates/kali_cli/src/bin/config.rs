//! CLI+manifest config resolution + runtime availability validation (crate-internal).

use kali_cli::build;
use kali_cli::output::CliOutputOptions;
use kali_error::{
    _error_codes::e5, Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use kali_npm::{discover_project_root, load_manifest, ProjectManifest};
use kali_optimize::ProfileData;
use kali_runtime::{
    browser_harness_command_parts_checked, browser_runtime_unavailable_diagnostic,
    normalize_runtime_profiles, BROWSER_HARNESS_COMMAND_ENV,
};
use std::{collections::BTreeSet, convert::TryFrom, path::{Path, PathBuf}};

pub(crate) fn resolve_effective_api_surface(
    explicit_api: Option<kali_cli::ApiSurface>,
) -> Result<kali_cli::ApiSurface, Vec<Diagnostic>> {
    if let Some(api) = explicit_api {
        return Ok(api);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let Some(manifest) = load_manifest(&project_root).map_err(|diagnostic| vec![diagnostic])?
    else {
        return Ok(kali_cli::ApiSurface::Deno);
    };

    manifest_api_surface(&manifest).map(|surface| surface.unwrap_or(kali_cli::ApiSurface::Deno))
}


pub(crate) fn config_diagnostic_context(config_path: &str) -> DiagnosticContext {
    DiagnosticContext::new(DiagnosticContextOrigin::Config).with_config_path(config_path)
}


pub(crate) fn config_diagnostic_context_with_value(
    config_path: &str,
    value: impl Into<String>,
) -> DiagnosticContext {
    config_diagnostic_context(config_path).with_effective_value(value)
}


pub(crate) fn manifest_api_surface(
    manifest: &ProjectManifest,
) -> Result<Option<kali_cli::ApiSurface>, Vec<Diagnostic>> {
    let Some(options) = manifest.compiler_options.as_ref() else {
        return Ok(None);
    };

    let Some(options) = options.as_object() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions` must be a JSON object",
        )
        .with_context(config_diagnostic_context("compilerOptions"))]);
    };

    let Some(value) = options.get("apiSurface") else {
        return Ok(None);
    };

    let Some(api_surface) = value.as_str() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.apiSurface` must be a string",
        )
        .with_context(config_diagnostic_context("compilerOptions.apiSurface"))]);
    };

    match api_surface {
        "deno" => Ok(Some(kali_cli::ApiSurface::Deno)),
        "node" => Ok(Some(kali_cli::ApiSurface::Node)),
        "browser" => Ok(Some(kali_cli::ApiSurface::Browser)),
        _ => Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            format!("unsupported apiSurface '{}' in kali.json", api_surface),
        )
        .with_context(config_diagnostic_context_with_value(
            "compilerOptions.apiSurface",
            api_surface,
        ))]),
    }
}


pub(crate) fn resolve_effective_compat_features(
    explicit_compat: Vec<String>,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let Some(manifest) = load_manifest(&project_root).map_err(|diagnostic| vec![diagnostic])?
    else {
        return Ok(normalize_compat_features(explicit_compat));
    };

    let mut features = normalize_compat_features(explicit_compat);
    features.extend(manifest_compat_features(&manifest)?);
    Ok(normalize_compat_features(features))
}


pub(crate) fn resolve_effective_runtime_profiles(
    explicit_wasm_threads: bool,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let Some(manifest) = load_manifest(&project_root).map_err(|diagnostic| vec![diagnostic])?
    else {
        return Ok(normalize_runtime_profiles(if explicit_wasm_threads {
            vec!["wasm-threads".to_string()]
        } else {
            Vec::new()
        }));
    };

    let mut profiles = normalize_runtime_profiles(if explicit_wasm_threads {
        vec!["wasm-threads".to_string()]
    } else {
        Vec::new()
    });
    profiles.extend(manifest_runtime_profiles(&manifest)?);
    Ok(normalize_runtime_profiles(profiles))
}


pub(crate) fn resolve_effective_max_specializations(
    explicit_max_specializations: Option<usize>,
) -> Result<usize, Vec<Diagnostic>> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let Some(manifest) = load_manifest(&project_root).map_err(|diagnostic| vec![diagnostic])?
    else {
        return Ok(explicit_max_specializations.unwrap_or(16));
    };

    let manifest_max_specializations = manifest_max_specializations(&manifest)?;
    Ok(explicit_max_specializations
        .or(manifest_max_specializations)
        .unwrap_or(16))
}


pub(crate) fn resolve_profile_data(profile: Option<PathBuf>) -> Result<Option<ProfileData>, Vec<Diagnostic>> {
    match profile {
        Some(profile_path) => build::load_profile_data_file(profile_path).map(Some),
        None => Ok(None),
    }
}


pub(crate) fn manifest_compat_features(manifest: &ProjectManifest) -> Result<Vec<String>, Vec<Diagnostic>> {
    let Some(compat) = manifest.compat.as_ref() else {
        return Ok(Vec::new());
    };

    let Some(compat) = compat.as_object() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compat` must be a JSON object",
        )
        .with_context(config_diagnostic_context("compat"))]);
    };

    let Some(features) = compat.get("features") else {
        return Ok(Vec::new());
    };

    let Some(features) = features.as_array() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compat.features` must be an array of strings",
        )
        .with_context(config_diagnostic_context("compat.features"))]);
    };

    let mut normalized = BTreeSet::new();
    for feature in features {
        let Some(feature) = feature.as_str() else {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                "`compat.features` entries must be strings",
            )
            .with_context(config_diagnostic_context("compat.features"))]);
        };

        let feature = feature.trim();
        if feature.is_empty() {
            continue;
        }

        if !matches!(feature, "eval") {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                format!("unsupported compat.feature '{}' in kali.json", feature),
            )
            .with_context(config_diagnostic_context_with_value(
                "compat.features",
                feature,
            ))]);
        }

        if !normalized.insert(feature.to_string()) {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                format!("duplicate compat.feature '{}' in kali.json", feature),
            )
            .with_context(config_diagnostic_context_with_value(
                "compat.features",
                feature,
            ))]);
        }
    }

    Ok(normalized.into_iter().collect())
}


pub(crate) fn manifest_runtime_profiles(manifest: &ProjectManifest) -> Result<Vec<String>, Vec<Diagnostic>> {
    let Some(options) = manifest.compiler_options.as_ref() else {
        return Ok(Vec::new());
    };

    let Some(options) = options.as_object() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions` must be a JSON object",
        )
        .with_context(config_diagnostic_context("compilerOptions"))]);
    };

    let Some(profiles) = options.get("runtimeProfiles") else {
        return Ok(Vec::new());
    };

    let Some(profiles) = profiles.as_array() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.runtimeProfiles` must be an array of strings",
        )
        .with_context(config_diagnostic_context(
            "compilerOptions.runtimeProfiles",
        ))]);
    };

    let mut raw_profiles = Vec::new();
    for profile in profiles {
        let Some(profile) = profile.as_str() else {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CONFIG as u32,
                "`compilerOptions.runtimeProfiles` entries must be strings",
            )
            .with_context(config_diagnostic_context(
                "compilerOptions.runtimeProfiles",
            ))]);
        };
        raw_profiles.push(profile.to_string());
    }

    build::validate_runtime_profiles(&raw_profiles, "kali.json").map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| {
                diagnostic
                    .with_context(config_diagnostic_context("compilerOptions.runtimeProfiles"))
            })
            .collect()
    })
}


pub(crate) fn manifest_max_specializations(
    manifest: &ProjectManifest,
) -> Result<Option<usize>, Vec<Diagnostic>> {
    let Some(options) = manifest.compiler_options.as_ref() else {
        return Ok(None);
    };

    let Some(options) = options.as_object() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions` must be a JSON object",
        )]);
    };

    let Some(value) = options.get("maxSpecializations") else {
        return Ok(None);
    };

    let Some(max_specializations) = value.as_u64() else {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.maxSpecializations` must be a non-negative integer",
        )]);
    };

    let max_specializations = usize::try_from(max_specializations).map_err(|_| {
        vec![Diagnostic::error(
            e5::INVALID_CONFIG as u32,
            "`compilerOptions.maxSpecializations` is too large for this host",
        )]
    })?;

    Ok(Some(max_specializations))
}


pub(crate) fn normalize_compat_features(features: Vec<String>) -> Vec<String> {
    let mut normalized = BTreeSet::new();
    for feature in features {
        let feature = feature.trim();
        if !feature.is_empty() {
            normalized.insert(feature.to_string());
        }
    }
    normalized.into_iter().collect()
}


pub(crate) fn reject_unavailable_compat_features(
    command: &str,
    compat_features: &[String],
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    let unavailable: Vec<String> = compat_features
        .iter()
        .filter(|feature| feature.as_str() != "eval")
        .cloned()
        .collect();

    if unavailable.is_empty() {
        return Ok(());
    }

    let diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "selected compatibility feature(s) {:?} are unavailable in this phase",
            unavailable
        ),
    )
    .note("canonical CLI flag: --compat")
    .note("canonical config path: compat.features");
    super::shared::emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        5,
        output,
        source_path,
        source_contents,
    )
}


pub(crate) fn reject_unavailable_runtime_profiles(
    command: &str,
    runtime_profiles: &[String],
    allow_threaded_profile: bool,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    let unavailable: Vec<String> = runtime_profiles
        .iter()
        .filter(|profile| profile.as_str() == "wasm-threads" && !allow_threaded_profile)
        .cloned()
        .collect();

    if unavailable.is_empty() {
        return Ok(());
    }

    let diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "selected runtime profile(s) {:?} are unavailable in this phase",
            unavailable
        ),
    )
    .note("canonical CLI flag: --wasm-threads")
    .note("canonical config path: compilerOptions.runtimeProfiles");
    super::shared::emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        5,
        output,
        source_path,
        source_contents,
    )
}


pub(crate) fn browser_runtime_harness_command_available() -> bool {
    std::env::var(BROWSER_HARNESS_COMMAND_ENV)
        .ok()
        .as_deref()
        .is_some_and(|command| browser_harness_command_parts_checked(Some(command)).is_ok())
}


pub(crate) fn reject_unavailable_browser_runtime(
    command: &str,
    api_surface: kali_cli::ApiSurface,
    browser_runtime_available: bool,
    browser_context: Option<DiagnosticContext>,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    if !matches!(api_surface, kali_cli::ApiSurface::Browser) || browser_runtime_available {
        return Ok(());
    }

    let diagnostic = browser_runtime_unavailable_diagnostic(Some(command), browser_context);
    super::shared::emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        1,
        output,
        source_path,
        source_contents,
    )
}


pub(crate) fn reject_unavailable_spawned_process_budget(
    command: &str,
    max_spawned_processes: Option<u64>,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    if max_spawned_processes.is_none_or(|count| count == 0) {
        return Ok(());
    }

    let mut diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        "selected resource budget(s) [\"resources.maxSpawnedProcesses\"] are unavailable in this phase",
    )
    .note("canonical CLI flag: --max-spawned-processes")
    .note("canonical config path: resources.maxSpawnedProcesses");

    if let Some(count) = max_spawned_processes {
        diagnostic = diagnostic.with_context(
            DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                .with_flag("--max-spawned-processes")
                .with_requested_value(count.to_string())
                .with_effective_value(count.to_string()),
        );
    }

    super::shared::emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        5,
        output,
        source_path,
        source_contents,
    )
}


pub(crate) fn reject_unavailable_zero_capable_budgets(
    command: &str,
    runtime_profiles: &[String],
    max_threads: Option<u64>,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_contents: Option<&str>,
) -> Result<(), i32> {
    let mut unavailable = Vec::new();
    let has_threaded_profile = runtime_profiles
        .iter()
        .any(|profile| profile.as_str() == "wasm-threads");
    if max_threads.is_some_and(|count| count > 0) && !has_threaded_profile {
        unavailable.push("resources.maxThreads");
    }

    if unavailable.is_empty() {
        return Ok(());
    }

    let mut diagnostic = Diagnostic::error(
        e5::FEATURE_UNAVAILABLE as u32,
        format!(
            "selected resource budget(s) {:?} are unavailable in this phase",
            unavailable
        ),
    )
    .note("canonical CLI flag: --max-threads")
    .note("canonical config path: resources.maxThreads")
    .note("threaded runtime profile config path: compilerOptions.runtimeProfiles");

    if let Some(count) = max_threads {
        diagnostic = diagnostic.with_context(
            DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                .with_flag("--max-threads")
                .with_requested_value(count.to_string())
                .with_effective_value(count.to_string()),
        );
    }
    super::shared::emit_diagnostics_and_exit(
        command,
        vec![diagnostic],
        5,
        output,
        source_path,
        source_contents,
    )
}


