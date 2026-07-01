#![allow(
    clippy::cloned_ref_to_slice_refs,
    clippy::match_like_matches_macro,
    clippy::needless_borrows_for_generic_args,
    clippy::question_mark,
    clippy::too_many_arguments
)]

use clap::Parser;
#[cfg(test)]
use kali_cli::output;
#[cfg(test)]
use kali_cli::output::validate_package_effects_payload_value;
use kali_cli::{
    init,
    output::{validate_init_payload_value, CliOutputOptions},
    Args, Commands,
};
use kali_error::{_error_codes::e5, set_verbose_diagnostics, Diagnostic};
#[cfg(test)]
use kali_sandbox::package_effects_report;
use serde_json::{json, Value};

mod cmd_build;
mod cmd_check;
mod cmd_doctor;
mod cmd_effects;
mod cmd_fmt;
mod cmd_install;
mod cmd_lint;
mod cmd_package;
mod cmd_run;
mod cmd_test;
mod config;
mod shared;

fn main() {
    let args = Args::parse();
    let output = CliOutputOptions {
        format: args.output,
        pretty: args.pretty,
        verbose: args.verbose,
        quiet: args.quiet,
        color: args.color,
    };
    set_verbose_diagnostics(output.verbose);

    let pretty_allowed_without_json =
        shared::command_allows_pretty_without_json(args.command.as_ref());
    if output.pretty && !output.is_json() && !pretty_allowed_without_json {
        let diagnostic = Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`--pretty` is only meaningful when JSON output is active",
        );
        eprintln!("{}", diagnostic);
        std::process::exit(5);
    }

    if args.command.is_none() {
        println!("kali 0.1.0");
        return;
    }

    match args.command.unwrap() {
        Commands::Check {
            sandbox,
            api,
            compat,
            wasm_threads,
            fix,
            files,
        } => {
            if let Err(exit_code) =
                cmd_check::check_command(files, sandbox, api, compat, wasm_threads, fix, &output)
            {
                std::process::exit(exit_code);
            }
        }
        Commands::Build {
            sandbox,
            api,
            compat,
            profile,
            validate_ir,
            wasm_threads,
            files,
            fast,
            release,
            release_advanced,
            max_specializations,
            bundle,
            format,
            lib,
            capi,
            component,
            out_dir,
        } => {
            if let Err(exit_code) = cmd_build::build_command(
                files,
                sandbox,
                api,
                compat,
                profile,
                validate_ir,
                wasm_threads,
                fast,
                release,
                release_advanced,
                max_specializations,
                bundle,
                format,
                lib,
                capi,
                component,
                out_dir,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
        Commands::Run {
            sandbox,
            api,
            compat,
            wasm_threads,
            max_specializations,
            max_spawned_processes,
            max_threads,
            file,
            guest_args,
        } => {
            if let Err(exit_code) = cmd_run::run_command(
                file,
                guest_args,
                api,
                compat,
                wasm_threads,
                max_specializations,
                max_spawned_processes,
                max_threads,
                sandbox,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
        Commands::Test {
            sandbox,
            api,
            compat,
            wasm_threads,
            max_specializations,
            max_spawned_processes,
            max_threads,
            files,
            filter,
            coverage,
        } => {
            if let Err(exit_code) = cmd_test::test_command(
                files,
                api,
                compat,
                wasm_threads,
                max_specializations,
                max_spawned_processes,
                max_threads,
                filter,
                coverage,
                sandbox,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
        Commands::Doctor => {
            if let Err(exit_code) = cmd_doctor::doctor_command(&output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Init { lib, api, sandbox } => {
            if let Err(exit_code) =
                shared::reject_workflow_context_flags("init", api, sandbox, &output)
            {
                std::process::exit(exit_code);
            }
            match init::init_current_directory(lib) {
                Ok(summary) => {
                    if output.is_json() {
                        let payload = json!({
                            "root": summary.root,
                            "manifestPath": summary.manifest_path,
                            "sourcePath": summary.source_path,
                            "library": summary.library,
                        });
                        validate_init_payload_value(&payload)
                            .expect("constructed init payload must satisfy schema-v1 shape");
                        shared::print_envelope(
                            "init",
                            true,
                            vec![],
                            vec![],
                            payload,
                            None,
                            None,
                            0,
                            &output,
                        );
                    } else if !output.quiet {
                        let template = if summary.library {
                            "library"
                        } else {
                            "application"
                        };
                        println!(
                            "Initialized {} scaffold at {}",
                            template,
                            summary.root.display()
                        );
                    }
                }
                Err(diagnostic) => {
                    let exit_code =
                        shared::diagnostics_exit_code(std::slice::from_ref(&diagnostic));
                    if output.is_json() {
                        let (errors, warnings) =
                            shared::single_diagnostic_to_values(diagnostic, None, None);
                        shared::print_envelope(
                            "init",
                            false,
                            errors,
                            warnings,
                            Value::Null,
                            None,
                            None,
                            exit_code,
                            &output,
                        );
                    } else {
                        eprintln!("{}", diagnostic);
                    }
                    std::process::exit(exit_code);
                }
            }
        }
        Commands::Install {
            target,
            dev,
            api,
            sandbox,
            allow_scripts,
        } => {
            if let Err(exit_code) =
                cmd_install::install_command(target, dev, api, sandbox, allow_scripts, &output)
            {
                std::process::exit(exit_code);
            }
        }
        Commands::Fmt {
            check,
            api,
            sandbox,
            files,
        } => {
            if let Err(exit_code) =
                shared::reject_workflow_context_flags("fmt", api, sandbox, &output)
            {
                std::process::exit(exit_code);
            }
            if let Err(exit_code) = cmd_fmt::fmt_command(files, check, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Lint {
            fix,
            api,
            sandbox,
            files,
        } => {
            if let Err(exit_code) =
                shared::reject_workflow_context_flags("lint", api, sandbox, &output)
            {
                std::process::exit(exit_code);
            }
            if let Err(exit_code) = cmd_lint::lint_command(files, fix, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Effects {
            api,
            compat,
            wasm_threads,
            sandbox,
            files,
        } => {
            if let Err(exit_code) =
                cmd_effects::effects_command(api, files, compat, wasm_threads, sandbox, &output)
            {
                std::process::exit(exit_code);
            }
        }
        Commands::PackageEffects {
            api,
            compat,
            wasm_threads,
            sandbox,
            target,
        } => {
            if let Err(exit_code) = cmd_package::package_effects_command(
                target,
                api,
                compat,
                wasm_threads,
                sandbox,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
        Commands::PackageAudit {
            api,
            compat,
            wasm_threads,
            sandbox,
            target,
            preview,
        } => {
            if let Err(exit_code) = cmd_package::package_audit_command(
                target,
                preview,
                api,
                compat,
                wasm_threads,
                sandbox,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::cmd_package::{
        analysis_context_for_api, package_analysis_specific_flag_context, package_audit_command,
        package_audit_preview_diagnostic, package_effects_command, sort_package_audit_findings,
        PACKAGE_AUDIT_PREVIEW_MESSAGE,
    };
    use super::config::{
        manifest_compat_features, manifest_max_specializations, manifest_runtime_profiles,
    };
    use super::shared::{command_allows_pretty_without_json, emit_native_json_payload};
    use super::{package_effects_report, CliOutputOptions};
    use kali_cli::{ColorChoice, OutputFormat};
    use kali_common::{FileId, Span};
    use kali_error::{_error_codes::e5, Diagnostic, DiagnosticContextOrigin};
    use kali_npm::ProjectManifest;
    use serde_json::json;
    use std::path::Path;

    fn diagnostic_with_span(file_id: u32, start: u32, end: u32) -> Diagnostic {
        Diagnostic::error(e5::INVALID_CLI_USAGE as u32, "shared finding").with_span(Span::new(
            FileId::new(file_id),
            start,
            end,
        ))
    }

    #[test]
    fn package_audit_findings_sort_by_span_as_final_tiebreaker() {
        let mut findings = vec![
            diagnostic_with_span(4, 20, 24),
            diagnostic_with_span(2, 10, 12),
            diagnostic_with_span(2, 8, 9),
            diagnostic_with_span(2, 10, 11),
        ];

        sort_package_audit_findings(&mut findings);

        let spans = findings
            .iter()
            .map(|diagnostic| diagnostic.span.expect("span"))
            .collect::<Vec<_>>();

        assert_eq!(
            spans,
            vec![
                Span::new(FileId::new(2), 8, 9),
                Span::new(FileId::new(2), 10, 11),
                Span::new(FileId::new(2), 10, 12),
                Span::new(FileId::new(4), 20, 24),
            ]
        );
    }

    #[test]
    fn diagnostics_exit_code_treats_feature_availability_as_usage_error() {
        let diagnostic = Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, "feature unavailable");

        assert_eq!(super::shared::diagnostics_exit_code(&[diagnostic]), 5);
    }

    #[test]
    fn pretty_without_json_is_only_allowed_for_effects_and_package_effects() {
        let effects = super::Commands::Effects {
            api: None,
            compat: Vec::new(),
            wasm_threads: false,
            sandbox: None,
            files: Vec::new(),
        };
        let package_effects = super::Commands::PackageEffects {
            api: None,
            compat: Vec::new(),
            wasm_threads: false,
            sandbox: None,
            target: Vec::new(),
        };
        let package_audit = super::Commands::PackageAudit {
            api: None,
            compat: Vec::new(),
            wasm_threads: false,
            sandbox: None,
            target: Vec::new(),
            preview: false,
        };

        assert!(command_allows_pretty_without_json(Some(&effects)));
        assert!(command_allows_pretty_without_json(Some(&package_effects)));
        assert!(!command_allows_pretty_without_json(Some(&package_audit)));
        assert!(!command_allows_pretty_without_json(None));
    }

    #[test]
    fn package_analysis_target_parser_rejects_whitespace_and_non_registry_forms() {
        for (command, target, expected_fragment) in [
            ("package-effects", " widget ", "without whitespace"),
            (
                "package-audit",
                "npm:lodash",
                "bare npm package names or `jsr:` identifiers",
            ),
            (
                "package-effects",
                "jsr:",
                "requires a package name after `jsr:`",
            ),
        ] {
            let err = super::cmd_package::parse_registry_package_target(command, target)
                .expect_err("invalid registry package target should fail");
            assert_eq!(err.code, Some(e5::INVALID_CLI_USAGE as u32));
            assert!(
                err.message.contains(expected_fragment),
                "unexpected error: {err:?}"
            );
        }
    }

    #[test]
    fn package_audit_preview_rejects_before_target_validation() {
        for (output, target) in [
            (
                CliOutputOptions {
                    format: OutputFormat::Text,
                    pretty: false,
                    verbose: false,
                    quiet: false,
                    color: ColorChoice::Auto,
                },
                Vec::<String>::new(),
            ),
            (
                CliOutputOptions {
                    format: OutputFormat::Json,
                    pretty: true,
                    verbose: false,
                    quiet: false,
                    color: ColorChoice::Auto,
                },
                vec![String::from("lodash"), String::from("react")],
            ),
            (
                CliOutputOptions {
                    format: OutputFormat::Json,
                    pretty: true,
                    verbose: false,
                    quiet: false,
                    color: ColorChoice::Auto,
                },
                vec![String::from("lodash")],
            ),
        ] {
            let exit_code =
                package_audit_command(target, true, None, Vec::new(), false, None, &output)
                    .expect_err("preview should fail before target validation");

            assert_eq!(exit_code, 5);
        }

        let diagnostic = package_audit_preview_diagnostic();
        let context = diagnostic.context.as_ref().expect("diagnostic context");
        assert_eq!(diagnostic.code, Some(e5::INVALID_CLI_USAGE as u32));
        assert_eq!(diagnostic.message, PACKAGE_AUDIT_PREVIEW_MESSAGE);
        assert_eq!(context.origin, DiagnosticContextOrigin::Cli);
        assert_eq!(context.flag.as_deref(), Some("--preview"));
        assert_eq!(context.requested_value.as_deref(), Some("true"));
        assert_eq!(context.effective_value.as_deref(), Some("true"));
    }

    #[test]
    fn package_audit_preview_diagnostic_serializes_cli_flag_context_in_json() {
        let diagnostic = package_audit_preview_diagnostic();
        let value = crate::output::diagnostic_to_json(&diagnostic, None, None, "error");

        assert_eq!(value["message"], json!(PACKAGE_AUDIT_PREVIEW_MESSAGE));
        assert_eq!(value["context"]["origin"], json!("cli"));
        assert_eq!(value["context"]["flag"], json!("--preview"));
        assert_eq!(value["context"]["requestedValue"], json!("true"));
        assert_eq!(value["context"]["effectiveValue"], json!("true"));
    }

    #[test]
    fn package_analysis_specific_flag_context_prefers_the_first_cli_flag() {
        let sandbox = Path::new("kali.policy.json");
        let context = package_analysis_specific_flag_context(
            Some(kali_cli::ApiSurface::Browser),
            &[String::from("eval")],
            true,
            Some(sandbox),
        )
        .expect("package-analysis-specific flag context");

        assert_eq!(context.origin, DiagnosticContextOrigin::Cli);
        assert_eq!(context.flag.as_deref(), Some("--api"));
        assert_eq!(context.requested_value.as_deref(), Some("browser"));
        assert_eq!(context.effective_value.as_deref(), Some("browser"));
    }

    #[test]
    fn package_analysis_specific_flag_context_records_compat_value() {
        let context =
            package_analysis_specific_flag_context(None, &[String::from("eval")], false, None)
                .expect("package-analysis-specific flag context");

        assert_eq!(context.origin, DiagnosticContextOrigin::Cli);
        assert_eq!(context.flag.as_deref(), Some("--compat"));
        assert_eq!(context.requested_value.as_deref(), Some("eval"));
        assert_eq!(context.effective_value.as_deref(), Some("eval"));
    }

    #[test]
    fn package_analysis_specific_flag_context_records_wasm_threads_value() {
        let context = package_analysis_specific_flag_context(None, &[], true, None)
            .expect("package-analysis-specific flag context");

        assert_eq!(context.origin, DiagnosticContextOrigin::Cli);
        assert_eq!(context.flag.as_deref(), Some("--wasm-threads"));
        assert_eq!(context.requested_value.as_deref(), Some("true"));
        assert_eq!(context.effective_value.as_deref(), Some("true"));
    }

    #[test]
    fn package_analysis_specific_flag_context_falls_back_to_sandbox() {
        let sandbox = Path::new("kali.policy.json");
        let context = package_analysis_specific_flag_context(None, &[], false, Some(sandbox))
            .expect("sandbox flag context");

        assert_eq!(context.origin, DiagnosticContextOrigin::Cli);
        assert_eq!(context.flag.as_deref(), Some("--sandbox"));
        assert_eq!(context.requested_value.as_deref(), Some("kali.policy.json"));
        assert_eq!(context.effective_value.as_deref(), Some("kali.policy.json"));
    }

    fn assert_package_analysis_specific_flag_rejection<F>(invoke: F)
    where
        F: Fn(&CliOutputOptions) -> Result<(), i32>,
    {
        for output in [
            CliOutputOptions {
                format: OutputFormat::Text,
                pretty: false,
                verbose: false,
                quiet: false,
                color: ColorChoice::Auto,
            },
            CliOutputOptions {
                format: OutputFormat::Json,
                pretty: true,
                verbose: false,
                quiet: false,
                color: ColorChoice::Auto,
            },
        ] {
            let exit_code = invoke(&output)
                .expect_err("package-analysis-specific flags should fail before target validation");

            assert_eq!(exit_code, 5);
        }
    }

    #[test]
    fn package_audit_rejects_package_analysis_specific_flags_before_target_validation() {
        for (api, compat, wasm_threads, sandbox) in [
            (
                Some(kali_cli::ApiSurface::Browser),
                Vec::<String>::new(),
                false,
                None,
            ),
            (None, vec![String::from("eval")], false, None),
            (None, Vec::<String>::new(), true, None),
            (
                None,
                Vec::<String>::new(),
                false,
                Some(Path::new("kali.policy.json").to_path_buf()),
            ),
        ] {
            assert_package_analysis_specific_flag_rejection(|output| {
                package_audit_command(
                    Vec::new(),
                    false,
                    api,
                    compat.clone(),
                    wasm_threads,
                    sandbox.clone(),
                    output,
                )
            });
        }
    }

    #[test]
    fn package_effects_rejects_package_analysis_specific_flags_before_target_validation() {
        for (api, compat, wasm_threads, sandbox) in [
            (
                Some(kali_cli::ApiSurface::Browser),
                Vec::<String>::new(),
                false,
                None,
            ),
            (None, vec![String::from("eval")], false, None),
            (None, Vec::<String>::new(), true, None),
            (
                None,
                Vec::<String>::new(),
                false,
                Some(Path::new("kali.policy.json").to_path_buf()),
            ),
        ] {
            assert_package_analysis_specific_flag_rejection(|output| {
                package_effects_command(
                    Vec::new(),
                    api,
                    compat.clone(),
                    wasm_threads,
                    sandbox.clone(),
                    output,
                )
            });
        }
    }

    #[test]
    fn native_json_payload_emission_validates_effects_payload_shape() {
        let output = CliOutputOptions {
            format: OutputFormat::Json,
            pretty: false,
            verbose: false,
            quiet: false,
            color: ColorChoice::Auto,
        };

        let result = std::panic::catch_unwind(|| {
            let _ = emit_native_json_payload("effects", &json!({"schemaVersion": 1}), &output);
        });

        assert!(
            result.is_err(),
            "invalid effects payload should panic before emission"
        );
    }

    #[test]
    fn native_json_payload_emission_accepts_package_audit_null_payload() {
        let output = CliOutputOptions {
            format: OutputFormat::Json,
            pretty: true,
            verbose: false,
            quiet: true,
            color: ColorChoice::Auto,
        };

        let result = emit_native_json_payload("package-audit", &serde_json::Value::Null, &output);

        assert!(
            result.is_ok(),
            "package-audit native JSON payload should validate and emit successfully"
        );
    }

    #[test]
    fn package_effects_report_carries_inherited_browser_threaded_context() {
        let context = analysis_context_for_api(
            kali_cli::ApiSurface::Browser,
            vec!["wasm-threads".to_string(), "wasm-threads".to_string()],
            Vec::new(),
        );
        let report = kali_sandbox::EffectReport {
            schema_version: 1,
            analysis_context: context.clone(),
            entry_points: vec!["widget".to_string()],
            effects: Vec::new(),
            dynamic_effects: false,
            dynamic_reasons: Vec::new(),
        };
        let payload = package_effects_report(
            kali_sandbox::PackageCoordinate {
                name: "widget".to_string(),
                version: "1.2.3".to_string(),
                registry: "npm".to_string(),
            },
            report,
        );
        let payload = serde_json::to_value(payload).expect("serialize package-effects payload");

        super::validate_package_effects_payload_value(&payload)
            .expect("browser-threaded package-effects payload should validate");
        assert_eq!(
            payload["report"]["analysisContext"]["apiSurface"],
            json!("browser")
        );
        assert_eq!(
            payload["report"]["analysisContext"]["runtimeProfiles"],
            json!(["wasm-threads"])
        );
        assert_eq!(
            payload["report"]["analysisContext"]["compatFeatures"],
            json!([])
        );
        assert_eq!(context.api_surface, "browser");
    }

    #[test]
    fn package_effects_report_rejects_whitespace_padded_package_coordinate_fields() {
        for (field, value, expected) in [
            (
                "name",
                " widget ",
                "package-effects payload package name must not have leading or trailing whitespace",
            ),
            (
                "version",
                " 1.2.3 ",
                "package-effects payload package version must not have leading or trailing whitespace",
            ),
            (
                "registry",
                " npm ",
                "package-effects payload package registry must not have leading or trailing whitespace",
            ),
        ] {
            let report = kali_sandbox::EffectReport {
                schema_version: 1,
                analysis_context: kali_sandbox::EffectAnalysisContext::new("deno"),
                entry_points: vec!["widget".to_string()],
                effects: Vec::new(),
                dynamic_effects: false,
                dynamic_reasons: Vec::new(),
            };
            let payload = package_effects_report(
                kali_sandbox::PackageCoordinate {
                    name: if field == "name" {
                        value.to_string()
                    } else {
                        "widget".to_string()
                    },
                    version: if field == "version" {
                        value.to_string()
                    } else {
                        "1.2.3".to_string()
                    },
                    registry: if field == "registry" {
                        value.to_string()
                    } else {
                        "npm".to_string()
                    },
                },
                report,
            );
            let payload = serde_json::to_value(payload).expect("serialize package-effects payload");

            let err = super::validate_package_effects_payload_value(&payload)
                .expect_err("whitespace-padded package coordinate should be rejected");

            assert_eq!(err, expected);
        }
    }

    #[test]
    fn manifest_compat_features_attach_config_context() {
        let manifest = ProjectManifest {
            compat: Some(json!({"features": ["eval", "future"]})),
            ..ProjectManifest::minimal()
        };

        let diagnostics = manifest_compat_features(&manifest)
            .expect_err("unsupported compat feature should fail manifest validation");
        let diagnostic = diagnostics.first().expect("diagnostic");
        let context = diagnostic.context.as_deref().expect("diagnostic context");

        assert_eq!(context.origin, DiagnosticContextOrigin::Config);
        assert_eq!(context.config_path.as_deref(), Some("compat.features"));
        assert_eq!(context.effective_value.as_deref(), Some("future"));
    }

    #[test]
    fn manifest_runtime_profiles_attach_config_context() {
        let manifest = ProjectManifest {
            compiler_options: Some(json!({"runtimeProfiles": ["future"]})),
            ..ProjectManifest::minimal()
        };

        let diagnostics = manifest_runtime_profiles(&manifest)
            .expect_err("unsupported runtime profile should fail manifest validation");
        let diagnostic = diagnostics.first().expect("diagnostic");
        let context = diagnostic.context.as_deref().expect("diagnostic context");

        assert_eq!(context.origin, DiagnosticContextOrigin::Config);
        assert_eq!(
            context.config_path.as_deref(),
            Some("compilerOptions.runtimeProfiles")
        );
    }

    #[test]
    fn manifest_max_specializations_accepts_zero() {
        let manifest = ProjectManifest {
            compiler_options: Some(json!({"maxSpecializations": 0})),
            ..ProjectManifest::minimal()
        };

        let max_specializations = manifest_max_specializations(&manifest)
            .expect("zero maxSpecializations should be accepted");

        assert_eq!(max_specializations, Some(0));
    }

    #[test]
    fn manifest_max_specializations_rejects_negative_values() {
        let manifest = ProjectManifest {
            compiler_options: Some(json!({"maxSpecializations": -1})),
            ..ProjectManifest::minimal()
        };

        let diagnostics = manifest_max_specializations(&manifest)
            .expect_err("negative maxSpecializations should fail manifest validation");
        let diagnostic = diagnostics.first().expect("diagnostic");

        assert_eq!(diagnostic.code, Some(e5::INVALID_CONFIG as u32));
        assert_eq!(
            diagnostic.message,
            "`compilerOptions.maxSpecializations` must be a non-negative integer"
        );
    }
}
