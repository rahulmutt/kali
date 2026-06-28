//! package-effects + package-audit command handlers.

use kali_cli::output::{self, CliOutputOptions};
use kali_error::{
    _error_codes::{e5, e6},
    Diagnostic, DiagnosticContext, DiagnosticContextOrigin,
};
use kali_npm::{audit_registry_package, discover_project_root, resolve_materialized_import};
use kali_sandbox::{
    compare_effects_to_policy, effect_report_from_inference, infer_effects_from_roots,
    package_effects_report, EffectAnalysisContext, PackageCoordinate, SandboxPolicy,
};
use serde_json::Value;
use std::{fs, path::{Path, PathBuf}};

use super::shared;
use super::config;

pub(crate) fn package_analysis_specific_flag_context(
    api: Option<kali_cli::ApiSurface>,
    compat: &[String],
    wasm_threads: bool,
    sandbox: Option<&Path>,
) -> Option<DiagnosticContext> {
    if let Some(api) = api {
        let api_value = api.to_string();
        return Some(
            DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                .with_flag("--api")
                .with_requested_value(api_value.clone())
                .with_effective_value(api_value),
        );
    }

    if let Some(compat_value) = compat.first() {
        return Some(
            DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                .with_flag("--compat")
                .with_requested_value(compat_value.clone())
                .with_effective_value(compat_value.clone()),
        );
    }

    if wasm_threads {
        return Some(
            DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                .with_flag("--wasm-threads")
                .with_requested_value("true")
                .with_effective_value("true"),
        );
    }

    if let Some(sandbox) = sandbox {
        let sandbox_value = sandbox.display().to_string();
        return Some(
            DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                .with_flag("--sandbox")
                .with_requested_value(sandbox_value.clone())
                .with_effective_value(sandbox_value),
        );
    }

    None
}

fn reject_package_analysis_specific_flags(
    command: &str,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if api.is_some() || !compat.is_empty() || wasm_threads || sandbox.is_some() {
        let mut diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "`{}` does not accept package-analysis-specific flags like `--api`, `--compat`, `--wasm-threads`, or `--sandbox`; use inherited project config instead",
                command
            ),
        );
        if let Some(context) =
            package_analysis_specific_flag_context(api, &compat, wasm_threads, sandbox.as_deref())
        {
            diagnostic = diagnostic.with_context(context);
        }
        return shared::emit_diagnostics_and_exit(command, vec![diagnostic], 5, output, None, None);
    }

    Ok(())
}

fn require_single_registry_package_target(
    command: &str,
    targets: Vec<String>,
    output: &CliOutputOptions,
) -> Result<String, i32> {
    let (message, exit_code, context) = match targets.as_slice() {
        [target] if target.trim().is_empty() => (
            format!("`{}` requires a non-empty package argument", command),
            5,
            Some(
                DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                    .with_requested_value(target.clone())
                    .with_effective_value(target.trim().to_string()),
            ),
        ),
        [target] if target.trim() != target => (
            format!(
                "`{}` requires a package argument without leading or trailing whitespace",
                command
            ),
            5,
            Some(
                DiagnosticContext::new(DiagnosticContextOrigin::Cli)
                    .with_requested_value(target.clone())
                    .with_effective_value(target.trim().to_string()),
            ),
        ),
        [target] => return Ok(target.clone()),
        [] => (
            format!("`{}` requires exactly one package argument", command),
            5,
            None,
        ),
        _ => (
            format!("`{}` accepts exactly one package argument", command),
            5,
            None,
        ),
    };

    let mut diagnostic = Diagnostic::error(e5::INVALID_CLI_USAGE as u32, message);
    if let Some(context) = context {
        diagnostic = diagnostic.with_context(context);
    }
    if output.is_json() {
        shared::print_envelope(
            command,
            false,
            vec![output::diagnostic_to_json(&diagnostic, None, None, "error")],
            vec![],
            Value::Null,
            None,
            None,
            exit_code,
            output,
        );
    } else {
        eprintln!("{}", diagnostic);
    }
    Err(exit_code)
}

pub(crate) fn package_effects_command(
    target: Vec<String>,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    reject_package_analysis_specific_flags(
        "package-effects",
        api,
        compat,
        wasm_threads,
        sandbox,
        output,
    )?;
    let target = require_single_registry_package_target("package-effects", target, output)?;

    let parsed = match parse_registry_package_target("package-effects", &target) {
        Ok(parsed) => parsed,
        Err(diagnostic) => {
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                5,
                output,
                None,
                None,
            );
        }
    };

    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!("failed to read current directory: {}", error),
            );
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let entry_path = match resolve_materialized_import(&project_root, &parsed.install_name) {
        Some(path) => path,
        None => {
            let diagnostic = Diagnostic::error(
                e6::DEPENDENCY_STATE_MISSING as u32,
                format!(
                    "package '{}' is not materialized in the current project",
                    target
                ),
            );
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };

    let package_root = match find_package_root(&entry_path) {
        Some(root) => root,
        None => {
            let diagnostic = Diagnostic::error(
                e6::DEPENDENCY_STATE_MISSING as u32,
                format!("unable to locate package root for '{}'", target),
            );
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };

    let package_version = match read_package_version(&package_root) {
        Ok(version) => version,
        Err(diagnostic) => {
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };

    let effective_api = match config::resolve_effective_api_surface(None) {
        Ok(api) => api,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                diagnostics,
                5,
                output,
                None,
                None,
            );
        }
    };
    let effective_runtime_profiles = match config::resolve_effective_runtime_profiles(false) {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                diagnostics,
                5,
                output,
                None,
                None,
            );
        }
    };
    // Registry-analysis inherits semantic analysis context, including the threaded profile.
    // The command still rejects explicit package-analysis flags, but inherited runtime profile
    // state is part of the stable report contract.
    if let Err(exit_code) = config::reject_unavailable_runtime_profiles(
        "package-effects",
        &effective_runtime_profiles,
        true,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let effective_compat = match config::resolve_effective_compat_features(Vec::new()) {
        Ok(features) => features,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                diagnostics,
                5,
                output,
                None,
                None,
            );
        }
    };
    if let Err(exit_code) =
        config::reject_unavailable_compat_features("package-effects", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let context =
        analysis_context_for_api(effective_api, effective_runtime_profiles, effective_compat);
    let inference = match infer_effects_from_roots(&[entry_path.clone()], context.clone()) {
        Ok(inference) => inference,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "package-effects",
                diagnostics,
                1,
                output,
                None,
                None,
            );
        }
    };

    let report = effect_report_from_inference(vec![parsed.report_label], context, inference);
    let payload = package_effects_report(
        PackageCoordinate {
            name: parsed.package_name,
            version: package_version,
            registry: parsed.registry,
        },
        report,
    );
    shared::emit_native_json_payload("package-effects", &payload, output)
}

pub(crate) fn sort_package_audit_findings(findings: &mut [Diagnostic]) {
    findings.sort_by(|left, right| {
        let left_rank = match left.severity {
            kali_error::Severity::Error => 0u8,
            kali_error::Severity::Warning => 1,
            kali_error::Severity::Info => 2,
        };
        let right_rank = match right.severity {
            kali_error::Severity::Error => 0u8,
            kali_error::Severity::Warning => 1,
            kali_error::Severity::Info => 2,
        };

        left_rank
            .cmp(&right_rank)
            .then_with(|| {
                left.code
                    .unwrap_or(u32::MAX)
                    .cmp(&right.code.unwrap_or(u32::MAX))
            })
            .then_with(|| left.message.cmp(&right.message))
            .then_with(|| left.notes.cmp(&right.notes))
            .then_with(|| left.suggestion.cmp(&right.suggestion))
            .then_with(|| diagnostic_span_sort_key(left).cmp(&diagnostic_span_sort_key(right)))
    });
}

fn diagnostic_span_sort_key(diagnostic: &Diagnostic) -> (u32, u32, u32, bool) {
    let Some(span) = diagnostic.span else {
        return (u32::MAX, u32::MAX, u32::MAX, true);
    };

    (span.file_id.as_u32(), span.start, span.end, false)
}

pub(crate) const PACKAGE_AUDIT_PREVIEW_MESSAGE: &str =
    "legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape";

pub(crate) fn package_audit_preview_diagnostic() -> Diagnostic {
    Diagnostic::error(e5::INVALID_CLI_USAGE as u32, PACKAGE_AUDIT_PREVIEW_MESSAGE).with_context(
        DiagnosticContext::new(DiagnosticContextOrigin::Cli)
            .with_flag("--preview")
            .with_requested_value("true")
            .with_effective_value("true"),
    )
}

pub(crate) fn package_audit_command(
    target: Vec<String>,
    preview: bool,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if preview {
        let diagnostic = package_audit_preview_diagnostic();
        return shared::emit_diagnostics_and_exit("package-audit", vec![diagnostic], 5, output, None, None);
    }

    reject_package_analysis_specific_flags(
        "package-audit",
        api,
        compat,
        wasm_threads,
        sandbox,
        output,
    )?;
    let target = require_single_registry_package_target("package-audit", target, output)?;

    let parsed = match parse_registry_package_target("package-audit", &target) {
        Ok(parsed) => parsed,
        Err(diagnostic) => {
            return shared::emit_diagnostics_and_exit(
                "package-audit",
                vec![diagnostic],
                5,
                output,
                None,
                None,
            );
        }
    };

    let audit = match audit_registry_package(&parsed.registry, &parsed.package_name) {
        Ok(audit) => audit,
        Err(diagnostic) => {
            return shared::emit_diagnostics_and_exit(
                "package-audit",
                vec![diagnostic],
                1,
                output,
                None,
                None,
            );
        }
    };

    let kali_npm::RegistryPackageAudit {
        registry,
        name,
        version,
        mut findings,
    } = audit;
    sort_package_audit_findings(&mut findings);
    let has_errors = findings.iter().any(|diagnostic| diagnostic.is_error());
    let summary = if findings.is_empty() {
        format!(
            "Package audit completed for {} package '{}@{}'; no security findings were computed.",
            registry, name, version
        )
    } else {
        let error_count = findings
            .iter()
            .filter(|diagnostic| diagnostic.is_error())
            .count();
        let warning_count = findings.len() - error_count;
        format!(
            "Package audit completed for {} package '{}@{}'; {} error(s), {} warning(s).",
            registry, name, version, error_count, warning_count
        )
    };

    if output.is_json() {
        let (errors, warnings) = shared::split_and_convert_diagnostics(&findings, None, None);
        shared::print_envelope(
            "package-audit",
            errors.is_empty(),
            errors,
            warnings,
            Value::Null,
            Some(summary),
            None,
            if has_errors { 1 } else { 0 },
            output,
        );
    } else if !output.quiet {
        println!("{summary}");
        for diagnostic in &findings {
            eprintln!("{}", diagnostic);
        }
    }

    if has_errors {
        Err(1)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PackageBinEntrypoint {
    package_name: String,
    bin_name: String,
}

pub(crate) fn validate_package_bin_runtime_entrypoint(
    source: &Path,
    api_surface: kali_cli::ApiSurface,
) -> Option<Diagnostic> {
    if api_surface == kali_cli::ApiSurface::Node {
        return None;
    }

    let package_bin = detect_package_bin_entrypoint(source)?;
    let source_contents = fs::read_to_string(source).ok()?;
    let markers = collect_unsupported_node_bin_markers(&source_contents);
    if markers.is_empty() {
        return None;
    }

    Some(
        Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "npm package bin '{}' from package '{}' assumes Node.js CLI features ({}) that are unavailable on the '{}' API surface in this phase",
                package_bin.bin_name,
                package_bin.package_name,
                markers.join(", "),
                api_surface,
            ),
        )
        .note("package bins that depend on CommonJS `require()` or the Node `process` global require the later Node compatibility path")
        .with_suggestion("run the package through an imported library entrypoint, or use the documented later-phase Node compatibility target"),
    )
}

fn detect_package_bin_entrypoint(source: &Path) -> Option<PackageBinEntrypoint> {
    let package_root = package_root_for_node_modules_source(source)?;
    let package_json_path = package_root.join("package.json");
    let package_json_contents = fs::read_to_string(&package_json_path).ok()?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_contents).ok()?;
    let package_name = package_json
        .get("name")
        .and_then(|value| value.as_str())?
        .to_string();
    let bin = package_json.get("bin")?;
    let relative_path = source.strip_prefix(&package_root).ok()?.to_string_lossy();

    match bin {
        serde_json::Value::String(path) => {
            if relative_path == path.as_str() {
                Some(PackageBinEntrypoint {
                    package_name: package_name.clone(),
                    bin_name: package_name,
                })
            } else {
                None
            }
        }
        serde_json::Value::Object(entries) => entries.iter().find_map(|(name, value)| {
            let path = value.as_str()?;
            (relative_path == path).then(|| PackageBinEntrypoint {
                package_name: package_name.clone(),
                bin_name: name.clone(),
            })
        }),
        _ => None,
    }
}

fn package_root_for_node_modules_source(source: &Path) -> Option<PathBuf> {
    for ancestor in source.ancestors() {
        if !ancestor.join("package.json").exists() {
            continue;
        }

        let parent = ancestor.parent()?;
        if parent.file_name() == Some(std::ffi::OsStr::new("node_modules")) {
            return Some(ancestor.to_path_buf());
        }

        let grandparent = parent.parent()?;
        if parent
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('@'))
            && grandparent.file_name() == Some(std::ffi::OsStr::new("node_modules"))
        {
            return Some(ancestor.to_path_buf());
        }
    }

    None
}

fn collect_unsupported_node_bin_markers(source: &str) -> Vec<&'static str> {
    let mut markers = Vec::new();
    if source.contains("require(") {
        markers.push("CommonJS require()")
    }
    if source_mentions_identifier(source, "process") {
        markers.push("Node process global")
    }
    markers
}

fn source_mentions_identifier(source: &str, ident: &str) -> bool {
    let bytes = source.as_bytes();
    let ident_bytes = ident.as_bytes();
    if ident_bytes.is_empty() || bytes.len() < ident_bytes.len() {
        return false;
    }

    let is_ident = |byte: u8| byte == b'_' || byte.is_ascii_alphanumeric();
    for start in 0..=bytes.len() - ident_bytes.len() {
        if &bytes[start..start + ident_bytes.len()] != ident_bytes {
            continue;
        }

        let before_ok = start == 0 || !is_ident(bytes[start - 1]);
        let after_index = start + ident_bytes.len();
        let after_ok = after_index == bytes.len() || !is_ident(bytes[after_index]);
        if before_ok && after_ok {
            return true;
        }
    }

    false
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedRegistryPackageTarget {
    registry: String,
    package_name: String,
    install_name: String,
    report_label: String,
}

pub(crate) fn parse_registry_package_target(
    command: &str,
    target: &str,
) -> Result<ParsedRegistryPackageTarget, Diagnostic> {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("./")
        || target.starts_with("../")
        || target.starts_with('/')
    {
        return Err(Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "`kali {command}` accepts only registry package identifiers, not '{}'",
                target
            ),
        ));
    }

    if target.chars().any(|ch| ch.is_whitespace()) {
        return Err(Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            format!(
                "`kali {command}` accepts only registry package identifiers without whitespace, not '{}'",
                target
            ),
        ));
    }

    if let Some((scheme, _)) = target.split_once(':') {
        if scheme != "jsr" {
            return Err(Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!(
                    "`kali {command}` accepts only bare npm package names or `jsr:` identifiers, not '{}'",
                    target
                ),
            ));
        }
    }

    let (registry, package_name, install_name, report_label) =
        if let Some(spec) = target.strip_prefix("jsr:") {
            if spec.is_empty() {
                return Err(Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    format!("`kali {command}` requires a package name after `jsr:`"),
                ));
            }
            if spec.contains(':')
                || spec.starts_with("http://")
                || spec.starts_with("https://")
                || spec.starts_with("./")
                || spec.starts_with("../")
                || spec.starts_with('/')
            {
                return Err(Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    format!(
                        "`kali {command}` accepts only registry package identifiers, not '{}'",
                        target
                    ),
                ));
            }
            if is_version_suffixed_package_spec(spec) {
                return Err(Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    format!("`kali {command}` does not accept explicit package versions yet"),
                ));
            }
            (
                "jsr".to_string(),
                spec.to_string(),
                spec.to_string(),
                target.to_string(),
            )
        } else {
            if is_version_suffixed_package_spec(target) {
                return Err(Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    format!("`kali {command}` does not accept explicit package versions yet"),
                ));
            }
            (
                "npm".to_string(),
                target.to_string(),
                target.to_string(),
                target.to_string(),
            )
        };

    Ok(ParsedRegistryPackageTarget {
        registry,
        package_name,
        install_name,
        report_label,
    })
}

fn is_version_suffixed_package_spec(spec: &str) -> bool {
    match spec.rsplit_once('@') {
        Some((name, version)) if !name.is_empty() && !version.is_empty() => true,
        _ => false,
    }
}

fn find_package_root(entry_path: &Path) -> Option<PathBuf> {
    let mut current = entry_path.parent()?.to_path_buf();
    loop {
        if current.join("package.json").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn read_package_version(package_root: &Path) -> Result<String, Diagnostic> {
    let path = package_root.join("package.json");
    let raw = fs::read_to_string(&path).map_err(|error| {
        Diagnostic::error(
            e6::DEPENDENCY_STATE_MISSING as u32,
            format!(
                "failed to read package metadata '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    let package_json: Value = serde_json::from_str(&raw).map_err(|error| {
        Diagnostic::error(
            e6::DEPENDENCY_STATE_MISSING as u32,
            format!(
                "failed to parse package metadata '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    package_json
        .get("version")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::DEPENDENCY_STATE_MISSING as u32,
                format!("package metadata '{}' is missing a version", path.display()),
            )
        })
}

pub(crate) fn analysis_context_for_api(
    api: kali_cli::ApiSurface,
    runtime_profiles: Vec<String>,
    compat_features: Vec<String>,
) -> EffectAnalysisContext {
    let mut context = EffectAnalysisContext::new(api.to_string());
    context.runtime_profiles = runtime_profiles;
    context.compat_features = compat_features;
    context.normalized()
}

pub(crate) fn validate_source_effects_against_policy(
    source: &Path,
    policy: &SandboxPolicy,
    api: kali_cli::ApiSurface,
) -> Result<(), Vec<Diagnostic>> {
    validate_source_effects_against_policy_for_roots(&[source.to_path_buf()], policy, api)
}

pub(crate) fn validate_source_effects_against_policy_for_roots(
    roots: &[PathBuf],
    policy: &SandboxPolicy,
    api: kali_cli::ApiSurface,
) -> Result<(), Vec<Diagnostic>> {
    let context = analysis_context_for_api(api, Vec::new(), Vec::new());
    let inference = infer_effects_from_roots(roots, context)?;
    let diagnostics = compare_effects_to_policy(&inference.effects, policy);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}
