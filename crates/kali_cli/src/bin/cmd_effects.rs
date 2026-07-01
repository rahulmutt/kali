//! effects command handler.

use kali_cli::output::CliOutputOptions;
use kali_error::{_error_codes::e5, Diagnostic};
use kali_sandbox::{effect_report_from_inference, infer_effects_from_roots};
use std::{fs, path::PathBuf};

use super::cmd_package::analysis_context_for_api;
use super::config;
use super::shared;

pub(crate) fn effects_command(
    api: Option<kali_cli::ApiSurface>,
    files: Vec<String>,
    compat: Vec<String>,
    wasm_threads: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if sandbox.is_some() {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`effects` does not accept `--sandbox`; use `check` or `build --sandbox` for policy validation"
                .to_string(),
        );
        return shared::emit_diagnostics_and_exit(
            "effects",
            vec![diagnostic],
            5,
            output,
            None,
            None,
        );
    }

    let Some(source) = shared::single_or_error(files, "effects", output)? else {
        return Err(1);
    };

    let effective_api = match config::resolve_effective_api_surface(api) {
        Ok(api) => api,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "effects",
                diagnostics,
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
    };
    if let Err(diagnostic) = shared::validate_runtime_entrypoint(&source, effective_api) {
        return shared::emit_diagnostics_and_exit(
            "effects",
            vec![diagnostic],
            5,
            output,
            Some(&source),
            fs::read_to_string(&source).ok().as_deref(),
        );
    }
    let effective_compat = match config::resolve_effective_compat_features(compat) {
        Ok(features) => features,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "effects",
                diagnostics,
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
    };
    if let Err(exit_code) = config::reject_unavailable_compat_features(
        "effects",
        &effective_compat,
        output,
        Some(&source),
        fs::read_to_string(&source).ok().as_deref(),
    ) {
        return Err(exit_code);
    }
    let effective_runtime_profiles = match config::resolve_effective_runtime_profiles(wasm_threads)
    {
        Ok(profiles) => profiles,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "effects",
                diagnostics,
                5,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
    };
    if let Err(exit_code) = config::reject_unavailable_runtime_profiles(
        "effects",
        &effective_runtime_profiles,
        !matches!(effective_api, kali_cli::ApiSurface::Browser),
        output,
        Some(&source),
        fs::read_to_string(&source).ok().as_deref(),
    ) {
        return Err(exit_code);
    }
    let context = analysis_context_for_api(
        effective_api,
        effective_runtime_profiles,
        effective_compat.clone(),
    );
    let inference = match infer_effects_from_roots(&[source.clone()], context.clone()) {
        Ok(inference) => inference,
        Err(diagnostics) => {
            return shared::emit_diagnostics_and_exit(
                "effects",
                diagnostics,
                1,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            );
        }
    };

    let report = effect_report_from_inference(
        vec![source.to_string_lossy().to_string()],
        context,
        inference,
    );
    shared::emit_native_json_payload("effects", &report, output)
}
