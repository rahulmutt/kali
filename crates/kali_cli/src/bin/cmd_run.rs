//! run command handler.

use kali_cli::{
    build,
    output::{self, validate_run_payload_value, CliOutputOptions},
};
use kali_error::DiagnosticContextOrigin;
use kali_runtime::{browser_runtime_request_context, RuntimeCtx};
use serde_json::{json, Value};
use std::{collections::BTreeMap, env, fs, path::PathBuf, time::Instant};

use super::cmd_package::validate_source_effects_against_policy;
use super::config;
use super::shared;

pub(crate) fn browser_stdout_thread_topology_snapshot_value(stdout: &str) -> Option<Value> {
    stdout.lines().rev().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        let value = serde_json::from_str::<Value>(trimmed).ok()?;
        value.get("threadTopology").cloned()
    })
}

pub(crate) fn run_command(
    file: String,
    guest_args: Vec<String>,
    api: Option<kali_cli::ApiSurface>,
    compat: Vec<String>,
    wasm_threads: bool,
    max_specializations: Option<usize>,
    max_spawned_processes: Option<u64>,
    max_threads: Option<u64>,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let effective_api = match config::resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
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

    // The opt-in browser harness can execute standalone browser-requested programs,
    // but it is not a Kali-hosted runtime sandbox. Keep `run --api browser --sandbox`
    // on the browser-runtime availability gate instead of silently dropping policy
    // enforcement when a harness override is present.
    let browser_runtime_available =
        config::browser_runtime_harness_command_available() && sandbox.is_none();
    if let Err(exit_code) = config::reject_unavailable_browser_runtime(
        "run",
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
            return shared::emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) =
        config::reject_unavailable_compat_features("run", &effective_compat, output, None, None)
    {
        return Err(exit_code);
    }
    let effective_runtime_profiles = match config::resolve_effective_runtime_profiles(wasm_threads)
    {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
        }
    };
    if let Err(exit_code) = config::reject_unavailable_runtime_profiles(
        "run",
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
        "run",
        max_spawned_processes,
        output,
        None,
        None,
    ) {
        return Err(exit_code);
    }
    if let Err(exit_code) = config::reject_unavailable_zero_capable_budgets(
        "run",
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
                return shared::emit_diagnostics_and_exit("run", diagnostics, 5, output, None, None)
            }
        };
    let compat_eval = effective_compat.iter().any(|feature| feature == "eval");
    let source = PathBuf::from(file);

    if let Some(policy) = policy.as_ref() {
        if let Err(diagnostics) =
            validate_source_effects_against_policy(&source, policy, effective_api)
        {
            return shared::emit_diagnostics_and_exit(
                "run",
                diagnostics,
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
    }

    if let Err(diagnostic) = shared::validate_runtime_entrypoint(&source, effective_api) {
        return shared::emit_diagnostics_and_exit("run", vec![diagnostic], 5, output, None, None);
    }
    if let Err(diagnostics) =
        build::reject_async_and_generator_class_methods_in_runtime_entrypoint(&source)
    {
        return shared::emit_diagnostics_and_exit(
            "run",
            diagnostics,
            1,
            output,
            Some(&source),
            fs::read_to_string(&source).ok().as_deref(),
        );
    }

    let wasm_bytes = match build::compile_source_file_with_specialization_cap_and_validation(
        &source,
        build::BuildMode::Fast,
        max_specializations,
        effective_api,
        &effective_runtime_profiles,
        compat_eval,
        false,
        false,
    ) {
        Ok(bytes) => bytes,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "run",
                diagnostics,
                1,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            )
        }
    };

    let runtime_args = if effective_api == kali_cli::ApiSurface::Node {
        let mut argv = vec!["node".to_string(), source.display().to_string()];
        let mut guest_args = guest_args;
        if guest_args.first().is_some_and(|arg| arg == "--") {
            guest_args.remove(0);
        }
        argv.extend(guest_args);
        argv
    } else {
        guest_args
    };
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        policy.clone(),
        runtime_args,
        env::vars().collect::<BTreeMap<_, _>>(),
        cwd,
        effective_api.to_string(),
    )
    .with_runtime_profiles(effective_runtime_profiles.clone())
    .with_max_threads(max_threads)
    .with_max_spawned_processes(max_spawned_processes);
    let start = Instant::now();
    match runtime.execute(&wasm_bytes) {
        Ok(outcome) => {
            if output.is_json() {
                let mut thread_topology = outcome.thread_topology.thread_topology_snapshot_value();
                if matches!(effective_api, kali_cli::ApiSurface::Browser)
                    && outcome.thread_topology.total_instances == 0
                    && outcome.thread_topology.terminated_instances == 0
                    && outcome.thread_topology.live_instances.is_empty()
                {
                    if let Some(stdout_thread_topology) =
                        browser_stdout_thread_topology_snapshot_value(&outcome.stdout)
                    {
                        output::merge_thread_topology_snapshot_values(
                            &mut thread_topology,
                            &stdout_thread_topology,
                        );
                    }
                }
                let payload = json!({
                    "exitCode": outcome.exit_code,
                    "runtimeMs": start.elapsed().as_millis(),
                    "hostContract": runtime.host_contract().canonical_label(),
                    "runtimeBackend": runtime.runtime_backend().canonical_label(),
                    "threadTopology": thread_topology,
                });
                validate_run_payload_value(&payload)
                    .expect("constructed run payload must satisfy schema-v1 shape");
                shared::print_envelope(
                    "run",
                    outcome.exit_code == 0,
                    vec![],
                    vec![],
                    payload,
                    Some(outcome.stdout),
                    Some(outcome.stderr),
                    outcome.exit_code,
                    output,
                );
            } else {
                if !output.quiet {
                    if !outcome.stdout.is_empty() {
                        print!("{}", outcome.stdout);
                    }
                    if !outcome.stdout_bytes.is_empty() {
                        use std::io::Write;
                        let mut out = std::io::stdout();
                        let _ = out.write_all(&outcome.stdout_bytes);
                        let _ = out.flush();
                    }
                    if !outcome.stderr.is_empty() {
                        eprint!("{}", outcome.stderr);
                    }
                }
            }
            if outcome.exit_code == 0 {
                Ok(())
            } else {
                Err(outcome.exit_code)
            }
        }
        Err(diagnostics) => shared::emit_diagnostics_and_exit(
            "run",
            diagnostics,
            1,
            output,
            Some(&source),
            fs::read_to_string(&source).ok().as_deref(),
        ),
    }
}
