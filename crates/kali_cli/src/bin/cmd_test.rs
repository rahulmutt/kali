//! test command handler.

use kali_cli::{
    build, discover_test_files,
    output::{self, validate_test_payload_value, CliOutputOptions},
};
use kali_error::DiagnosticContextOrigin;
use kali_npm::discover_project_root;
use kali_runtime::{browser_runtime_request_context, RuntimeCtx};
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Instant,
};
use wasmparser::{Parser as WasmParser, Payload};

use super::cmd_package::validate_source_effects_against_policy_for_roots;
use super::cmd_run::browser_stdout_thread_topology_snapshot_value;
use super::config;
use super::shared;

pub(crate) fn test_command(
    files: Vec<String>,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    max_specializations: Option<usize>,
    max_spawned_processes: Option<u64>,
    max_threads: Option<u64>,
    filter: Option<String>,
    coverage: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let effective_api = match config::resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None)
        }
    };

    let browser_context = if matches!(effective_api, kali_cli::ApiSurface::Browser) {
        let origin = if api.is_some() {
            DiagnosticContextOrigin::Cli
        } else {
            DiagnosticContextOrigin::Config
        };
        let context = browser_runtime_request_context(origin);
        Some(if api.is_some() {
            context.with_flag("--api")
        } else {
            context.with_config_path("compilerOptions.apiSurface")
        })
    } else {
        None
    };

    // The opt-in browser harness can execute standalone browser-requested tests,
    // but it is not a Kali-hosted runtime sandbox. Keep `test --api browser --sandbox`
    // on the browser-runtime availability gate instead of silently dropping policy
    // enforcement when a harness override is present.
    let browser_runtime_available =
        config::browser_runtime_harness_command_available() && sandbox.is_none();
    if let Err(exit_code) = config::reject_unavailable_browser_runtime(
        "test",
        effective_api,
        browser_runtime_available,
        browser_context,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }

    shared::ensure_project_ready_or_exit(output)?;
    let effective_compat = match config::resolve_effective_compat_features(compat) {
        Ok(features) => features,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) =
        config::reject_unavailable_compat_features("test", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let effective_runtime_profiles = match config::resolve_effective_runtime_profiles(wasm_threads)
    {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) = config::reject_unavailable_runtime_profiles(
        "test",
        &effective_runtime_profiles,
        true,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let policy = shared::load_policy_or_exit(sandbox, &effective_runtime_profiles, output)?;
    if let Err(exit_code) = config::reject_unavailable_spawned_process_budget(
        "test",
        max_spawned_processes,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    if let Err(exit_code) = config::reject_unavailable_zero_capable_budgets(
        "test",
        &effective_runtime_profiles,
        max_threads,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    let max_specializations =
        match config::resolve_effective_max_specializations(max_specializations) {
            Ok(max_specializations) => max_specializations,
            Err(diagnostics) => {
                return shared::emit_diagnostics_and_exit(
                    "test",
                    diagnostics,
                    5,
                    output,
                    None,
                    None,
                )
            }
        };
    let compat_eval = effective_compat.iter().any(|feature| feature == "eval");

    let selected_files = if files.is_empty() {
        discover_test_files(".")
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
    } else {
        files
    };

    let mut valid_files = Vec::new();
    for file in selected_files {
        let source = PathBuf::from(&file);
        if let Err(diagnostic) = shared::validate_runtime_entrypoint(&source, effective_api) {
            return shared::emit_diagnostics_and_exit(
                "test",
                vec![diagnostic],
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
        if let Err(diagnostics) =
            build::reject_async_and_generator_class_methods_in_runtime_entrypoint(&source)
        {
            return shared::emit_diagnostics_and_exit(
                "test",
                diagnostics,
                1,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
        valid_files.push(file);
    }

    let filtered_files = if let Some(pattern) = filter.as_deref() {
        valid_files
            .into_iter()
            .filter(|file| shared::matches_test_filter(file, pattern))
            .collect::<Vec<_>>()
    } else {
        valid_files
    };

    if filtered_files.is_empty() {
        if output.is_json() {
            let payload = if coverage {
                json!({
                    "total": 0,
                    "passed": 0,
                    "failed": 0,
                    "skipped": 0,
                    "runtimeMs": 0,
                    "coverage": {
                        "mode": "function",
                        "files": [],
                        "summary": {
                            "functionsTotal": 0,
                            "functionsCovered": 0,
                            "functionsMissed": 0,
                            "coveragePercent": 100.0,
                        },
                    },
                })
            } else {
                json!({ "total": 0, "passed": 0, "failed": 0, "skipped": 0, "runtimeMs": 0 })
            };
            shared::print_envelope(
                "test",
                true,
                vec![],
                vec![],
                payload,
                Some(String::new()),
                Some(String::new()),
                0,
                output,
            );
        } else if !output.quiet {
            println!("ok 0");
        }
        return Ok(());
    }

    if let Some(policy) = policy.as_ref() {
        let roots = filtered_files.iter().map(PathBuf::from).collect::<Vec<_>>();
        if let Err(diagnostics) =
            validate_source_effects_against_policy_for_roots(&roots, policy, effective_api)
        {
            return shared::emit_diagnostics_and_exit("test", diagnostics, 5, output, None, None);
        }
    }

    let runtime = RuntimeCtx::with_api_surface(policy.clone(), effective_api.to_string())
        .with_runtime_profiles(effective_runtime_profiles.clone())
        .with_max_threads(max_threads)
        .with_max_spawned_processes(max_spawned_processes);
    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut captured_stdout = String::new();
    let mut captured_stderr = String::new();
    let mut thread_topology = json!({
        "totalInstances": 0,
        "terminatedInstances": 0,
        "liveInstances": [],
    });
    let mut diagnostics = Vec::new();
    let mut coverage_reports = Vec::new();
    let project_root =
        discover_project_root(&env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
    let start = Instant::now();

    for file in filtered_files {
        let source = PathBuf::from(&file);
        let wasm_bytes = match build::compile_source_file_with_specialization_cap_and_validation(
            &source,
            build::BuildMode::Fast,
            max_specializations,
            effective_api,
            &effective_runtime_profiles,
            compat_eval,
            false,
            coverage,
        ) {
            Ok(bytes) => bytes,
            Err(errs) => {
                diagnostics.extend(errs.clone());
                if !output.is_json() {
                    for diagnostic in errs {
                        eprintln!("{}", diagnostic);
                    }
                }
                failed += 1;
                continue;
            }
        };

        let coverage_total = if coverage {
            coverage_function_count_from_wasm(&wasm_bytes).unwrap_or(0)
        } else {
            0
        };

        match runtime.execute_tests(&wasm_bytes) {
            Ok(outcome) => {
                total += outcome.tests_run;
                passed += outcome.tests_run.saturating_sub(outcome.tests_failed);
                failed += outcome.tests_failed;
                captured_stdout.push_str(&outcome.stdout);
                captured_stderr.push_str(&outcome.stderr);
                let mut outcome_thread_topology =
                    outcome.thread_topology.thread_topology_snapshot_value();
                if matches!(effective_api, kali_cli::ApiSurface::Browser)
                    && outcome.thread_topology.total_instances == 0
                    && outcome.thread_topology.terminated_instances == 0
                    && outcome.thread_topology.live_instances.is_empty()
                {
                    if let Some(stdout_thread_topology) =
                        browser_stdout_thread_topology_snapshot_value(&outcome.stdout)
                    {
                        output::merge_thread_topology_snapshot_values(
                            &mut outcome_thread_topology,
                            &stdout_thread_topology,
                        );
                    }
                }
                output::merge_thread_topology_snapshot_values(
                    &mut thread_topology,
                    &outcome_thread_topology,
                );
                if coverage {
                    let covered = outcome.coverage_hits.len().min(coverage_total);
                    coverage_reports.push(json!({
                        "file": normalize_coverage_report_path(&file, &project_root),
                        "functionsTotal": coverage_total,
                        "functionsCovered": covered,
                        "functionsMissed": coverage_total.saturating_sub(covered),
                    }));
                }
            }
            Err(errs) => {
                diagnostics.extend(errs.clone());
                if !output.is_json() {
                    for diagnostic in errs {
                        eprintln!("{}", diagnostic);
                    }
                }
                failed += 1;
            }
        }
    }

    if coverage {
        sort_coverage_reports(&mut coverage_reports);
    }

    if output.is_json() {
        let payload = if coverage {
            let functions_total = coverage_reports
                .iter()
                .map(|report| report["functionsTotal"].as_u64().unwrap_or(0) as usize)
                .sum::<usize>();
            let functions_covered = coverage_reports
                .iter()
                .map(|report| report["functionsCovered"].as_u64().unwrap_or(0) as usize)
                .sum::<usize>();
            let functions_missed = functions_total.saturating_sub(functions_covered);
            json!({
                "total": total,
                "passed": passed,
                "failed": failed,
                "skipped": 0,
                "runtimeMs": start.elapsed().as_millis(),
                "hostContract": runtime.host_contract().canonical_label(),
                "runtimeBackend": runtime.runtime_backend().canonical_label(),
                "threadTopology": thread_topology,
                "coverage": {
                    "mode": "function",
                    "files": coverage_reports,
                    "summary": {
                        "functionsTotal": functions_total,
                        "functionsCovered": functions_covered,
                        "functionsMissed": functions_missed,
                        "coveragePercent": coverage_percent(functions_covered, functions_total),
                    },
                },
            })
        } else {
            json!({
                "total": total,
                "passed": passed,
                "failed": failed,
                "skipped": 0,
                "runtimeMs": start.elapsed().as_millis(),
                "hostContract": runtime.host_contract().canonical_label(),
                "runtimeBackend": runtime.runtime_backend().canonical_label(),
                "threadTopology": thread_topology,
            })
        };
        let success = diagnostics.is_empty();
        let (errors, warnings) = shared::split_and_convert_diagnostics(&diagnostics, None, None);
        validate_test_payload_value(&payload)
            .expect("constructed test payload must satisfy schema-v1 shape");
        shared::print_envelope(
            "test",
            success,
            errors,
            warnings,
            payload,
            Some(captured_stdout),
            Some(captured_stderr),
            if success { 0 } else { 1 },
            output,
        );
    } else if !output.quiet {
        if !captured_stdout.is_empty() {
            print!("{}", captured_stdout);
        }
        if !captured_stderr.is_empty() {
            eprint!("{}", captured_stderr);
        }
        if diagnostics.is_empty() {
            if coverage {
                let functions_total = coverage_reports
                    .iter()
                    .map(|report| report["functionsTotal"].as_u64().unwrap_or(0) as usize)
                    .sum::<usize>();
                let functions_covered = coverage_reports
                    .iter()
                    .map(|report| report["functionsCovered"].as_u64().unwrap_or(0) as usize)
                    .sum::<usize>();
                println!(
                    "ok {} (coverage: {}/{} functions, {:.1}%)",
                    passed,
                    functions_covered,
                    functions_total,
                    coverage_percent(functions_covered, functions_total)
                );
            } else {
                println!("ok {}", passed);
            }
        } else {
            println!("FAILED {}", failed);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(1)
    }
}

fn coverage_function_count_from_wasm(bytes: &[u8]) -> Option<usize> {
    for payload in WasmParser::new(0).parse_all(bytes) {
        if let Ok(Payload::CustomSection(section)) = payload {
            if section.name() == "kali:coverage" && section.data().len() >= 4 {
                let mut raw = [0u8; 4];
                raw.copy_from_slice(&section.data()[..4]);
                return Some(u32::from_le_bytes(raw) as usize);
            }
        }
    }
    None
}

fn normalize_coverage_report_path(file: &str, project_root: &Path) -> String {
    let source = PathBuf::from(file);
    let candidate = if source.is_absolute() {
        source
    } else {
        env::current_dir()
            .ok()
            .map(|cwd| cwd.join(&source))
            .unwrap_or(source)
    };
    let canonical = fs::canonicalize(&candidate).unwrap_or(candidate);
    canonical
        .strip_prefix(project_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| canonical.to_string_lossy().replace('\\', "/"))
}

fn sort_coverage_reports(reports: &mut [Value]) {
    reports.sort_by(|left, right| {
        let left_file = left.get("file").and_then(Value::as_str).unwrap_or("");
        let right_file = right.get("file").and_then(Value::as_str).unwrap_or("");
        left_file.cmp(right_file)
    });
}

fn coverage_percent(covered: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        (covered as f64 / total as f64) * 100.0
    }
}
