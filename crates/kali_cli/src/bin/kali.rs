use clap::Parser;
use kali_cli::{
    build, discover_source_files, discover_test_files, init, is_declaration_only_source_file,
    load_sandbox_policy,
    output::{self, CliOutputOptions},
    Args, Commands,
};
use kali_error::{Diagnostic, _error_codes::e5};
use kali_fmt::format_source;
use kali_lint::lint_with_options;
use kali_npm::{discover_project_root, ensure_project_ready, install_project, InstallOptions};
use kali_runtime::RuntimeCtx;
use kali_sandbox::SandboxPolicy;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

fn main() {
    let args = Args::parse();
    let output = CliOutputOptions {
        format: args.output,
        pretty: args.pretty,
        verbose: args.verbose,
        quiet: args.quiet,
        color: args.color,
    };

    if output.pretty && !output.is_json() {
        eprintln!("Error[E5008]: `--pretty` is only meaningful when JSON output is active");
        std::process::exit(1);
    }

    if args.command.is_none() {
        println!("kali 0.1.0");
        return;
    }

    match args.command.unwrap() {
        Commands::Check { sandbox, files } => {
            if let Err(exit_code) = check_command(files, sandbox, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Build {
            sandbox,
            files,
            fast,
            release,
            release_advanced,
            out_dir,
        } => {
            if let Err(exit_code) = build_command(
                files,
                sandbox,
                fast,
                release,
                release_advanced,
                out_dir,
                &output,
            ) {
                std::process::exit(exit_code);
            }
        }
        Commands::Run { sandbox, files } => {
            if let Err(exit_code) = run_command(files, sandbox, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Test {
            sandbox,
            files,
            filter,
            coverage,
        } => {
            if let Err(exit_code) = test_command(files, filter, coverage, sandbox, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Init { lib } => match init::init_current_directory(lib) {
            Ok(summary) => {
                if output.is_json() {
                    let payload = json!({
                        "root": summary.root,
                        "manifestPath": summary.manifest_path,
                        "sourcePath": summary.source_path,
                        "library": summary.library,
                    });
                    print_envelope(
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
                if output.is_json() {
                    let (errors, warnings) = single_diagnostic_to_values(diagnostic, None, None);
                    print_envelope(
                        "init",
                        false,
                        errors,
                        warnings,
                        Value::Null,
                        None,
                        None,
                        1,
                        &output,
                    );
                } else {
                    eprintln!("{}", diagnostic);
                }
                std::process::exit(1);
            }
        },
        Commands::Install {
            target,
            dev,
            allow_scripts,
        } => {
            if let Err(exit_code) = install_command(target, dev, allow_scripts, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Fmt { check, files } => {
            if let Err(exit_code) = fmt_command(files, check, &output) {
                std::process::exit(exit_code);
            }
        }
        Commands::Lint { fix, files } => {
            if let Err(exit_code) = lint_command(files, fix, &output) {
                std::process::exit(exit_code);
            }
        }
    }
}

fn check_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let policy = load_policy_or_exit(sandbox, output)?;
    ensure_project_ready_or_exit(output)?;

    let selected_files = if files.is_empty() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_root = discover_project_root(&cwd).unwrap_or(cwd);
        discover_source_files(&project_root)
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
    } else {
        files
    };

    if let Some(policy) = policy.as_ref() {
        if let Err(diagnostics) = policy.validate() {
            return emit_diagnostics_and_exit("check", diagnostics, 5, output, None, None);
        }
    }

    let mut checked = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for file in selected_files {
        checked += 1;
        match build::check_source_file(&file) {
            Ok(()) => {}
            Err(diagnostics) => {
                let source = fs::read_to_string(&file).ok();
                let (file_errors, file_warnings) = split_and_convert_diagnostics(
                    &diagnostics,
                    Some(Path::new(&file)),
                    source.as_deref(),
                );
                errors.extend(file_errors);
                warnings.extend(file_warnings);
                if !output.is_json() {
                    for diagnostic in diagnostics {
                        eprintln!("{}", diagnostic);
                    }
                }
            }
        }
    }

    let success = errors.is_empty();
    if output.is_json() {
        let payload = json!({
            "filesChecked": checked,
            "errorCount": errors.len(),
            "warningCount": warnings.len(),
        });
        print_envelope(
            "check",
            success,
            errors,
            warnings,
            payload,
            None,
            None,
            if success { 0 } else { 1 },
            output,
        );
    } else if success {
        if !output.quiet {
            println!("Checked {} file(s)", checked);
        }
    }

    if success {
        Ok(())
    } else {
        Err(1)
    }
}

fn build_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    fast: bool,
    release: bool,
    release_advanced: bool,
    out_dir: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let policy = load_policy_or_exit(sandbox, output)?;
    ensure_project_ready_or_exit(output)?;

    if let Some(policy) = policy.as_ref() {
        if let Err(diagnostics) = policy.validate() {
            return emit_diagnostics_and_exit("build", diagnostics, 5, output, None, None);
        }
    }

    let selected_files = if files.is_empty() { vec![] } else { files };
    if selected_files.is_empty() {
        let diagnostic = Diagnostic::error(
            e5::MISSING_REQUIRED_ARGUMENT as u32,
            "build requires at least one source file",
        );
        if output.is_json() {
            let (errors, warnings) = single_diagnostic_to_values(diagnostic, None, None);
            print_envelope(
                "build",
                false,
                errors,
                warnings,
                Value::Null,
                None,
                None,
                1,
                output,
            );
        } else {
            eprintln!("{}", diagnostic);
        }
        return Err(1);
    }

    let mode = build::build_mode_from_flags(fast, release, release_advanced);
    let out_dir_path = out_dir.as_deref();
    let mut artifacts = Vec::new();
    let mut errors = Vec::new();

    for file in selected_files {
        match build::build_source_file(&file, mode, out_dir_path, policy.as_ref()) {
            Ok(output_file) => {
                if output.is_json() {
                    let source_hash =
                        source_hash_for_file(Path::new(&file)).unwrap_or_else(|_| "".to_string());
                    artifacts.push(json!({
                        "artifactKind": artifact_kind_for_build(out_dir_path),
                        "outputPath": output_file.output_path,
                        "sizeBytes": output_file.wasm_bytes.len(),
                        "buildMode": build_mode_name(mode),
                        "sourceHash": source_hash,
                    }));
                } else if !output.quiet {
                    println!("Built {} -> {}", file, output_file.output_path.display());
                }
            }
            Err(diagnostics) => {
                errors.extend(diagnostics.iter().cloned());
                if !output.is_json() {
                    for diagnostic in diagnostics {
                        eprintln!("{}", diagnostic);
                    }
                }
            }
        }
    }

    let success = errors.is_empty();
    if output.is_json() {
        let payload = if artifacts.len() == 1 {
            artifacts.into_iter().next().unwrap()
        } else {
            json!({ "artifacts": artifacts })
        };
        let (error_values, warning_values) = split_and_convert_diagnostics(&errors, None, None);
        print_envelope(
            "build",
            success,
            error_values,
            warning_values,
            payload,
            None,
            None,
            if success { 0 } else { 1 },
            output,
        );
    }

    if success {
        Ok(())
    } else {
        Err(1)
    }
}

fn run_command(
    files: Vec<String>,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let policy = load_policy_or_exit(sandbox, output)?;
    ensure_project_ready_or_exit(output)?;
    let Some(source) = single_or_error(files, "run", output)? else {
        return Err(1);
    };

    if let Err(diagnostic) = validate_runtime_entrypoint(&source) {
        return emit_diagnostics_and_exit("run", vec![diagnostic], 1, output, None, None);
    }

    let wasm_bytes = match build::compile_source_file(&source, build::BuildMode::Fast) {
        Ok(bytes) => bytes,
        Err(diagnostics) => {
            return emit_diagnostics_and_exit(
                "run",
                diagnostics,
                1,
                output,
                Some(&source),
                fs::read_to_string(&source).ok().as_deref(),
            )
        }
    };

    let runtime = RuntimeCtx::new(policy.clone());
    let start = Instant::now();
    match runtime.execute(&wasm_bytes) {
        Ok(outcome) => {
            if output.is_json() {
                let payload = json!({
                    "exitCode": outcome.exit_code,
                    "runtimeMs": start.elapsed().as_millis(),
                });
                print_envelope(
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
        Err(diagnostics) => emit_diagnostics_and_exit(
            "run",
            diagnostics,
            1,
            output,
            Some(&source),
            fs::read_to_string(&source).ok().as_deref(),
        ),
    }
}

fn test_command(
    files: Vec<String>,
    filter: Option<String>,
    coverage: bool,
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    if coverage {
        let diagnostic = Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "test coverage reporting is unavailable in this phase",
        );
        if output.is_json() {
            let (errors, warnings) = single_diagnostic_to_values(diagnostic, None, None);
            print_envelope(
                "test",
                false,
                errors,
                warnings,
                Value::Null,
                None,
                None,
                1,
                output,
            );
        } else {
            eprintln!("{}", diagnostic);
        }
        return Err(1);
    }

    let policy = load_policy_or_exit(sandbox, output)?;
    ensure_project_ready_or_exit(output)?;

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
        if let Err(diagnostic) = validate_runtime_entrypoint(&source) {
            return emit_diagnostics_and_exit(
                "test",
                vec![diagnostic],
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
            .filter(|file| matches_test_filter(file, pattern))
            .collect::<Vec<_>>()
    } else {
        valid_files
    };

    if filtered_files.is_empty() {
        if output.is_json() {
            let payload = json!({ "total": 0, "passed": 0, "failed": 0, "skipped": 0 });
            print_envelope(
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

    let runtime = RuntimeCtx::new(policy.clone());
    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut captured_stdout = String::new();
    let mut captured_stderr = String::new();
    let mut diagnostics = Vec::new();
    let start = Instant::now();

    for file in filtered_files {
        let source = PathBuf::from(&file);
        let wasm_bytes = match build::compile_source_file(&source, build::BuildMode::Fast) {
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

        match runtime.execute_tests(&wasm_bytes) {
            Ok(outcome) => {
                total += outcome.tests_run;
                passed += outcome.tests_run.saturating_sub(outcome.tests_failed);
                failed += outcome.tests_failed;
                captured_stdout.push_str(&outcome.stdout);
                captured_stderr.push_str(&outcome.stderr);
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

    if output.is_json() {
        let payload = json!({
            "total": total,
            "passed": passed,
            "failed": failed,
            "skipped": 0,
            "runtimeMs": start.elapsed().as_millis(),
        });
        let success = diagnostics.is_empty();
        let (errors, warnings) = split_and_convert_diagnostics(&diagnostics, None, None);
        print_envelope(
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
            println!("ok {}", passed);
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

fn fmt_command(files: Vec<String>, check: bool, output: &CliOutputOptions) -> Result<(), i32> {
    ensure_project_ready_or_exit(output)?;
    let selected_files = selected_source_files(files, true);
    if selected_files.is_empty() {
        if output.is_json() {
            let payload = json!({"filesFormatted": 0, "filesChecked": 0});
            print_envelope("fmt", true, vec![], vec![], payload, None, None, 0, output);
        } else if !output.quiet {
            println!("{} 0 file(s)", if check { "Checked" } else { "Formatted" });
        }
        return Ok(());
    }

    let mut changed = 0usize;
    let mut processed = 0usize;
    for file in selected_files {
        processed += 1;
        let path = PathBuf::from(&file);
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                let diagnostic = Diagnostic::error(
                    e5::OUTPUT_ERROR as u32,
                    format!("failed to read source file '{}': {}", path.display(), error),
                );
                return emit_diagnostics_and_exit(
                    "fmt",
                    vec![diagnostic],
                    1,
                    output,
                    Some(&path),
                    None,
                );
            }
        };
        let formatted = format_source(&source);
        if formatted != source {
            changed += 1;
            if !check {
                if let Err(error) = fs::write(&path, formatted) {
                    let diagnostic = Diagnostic::error(
                        e5::OUTPUT_ERROR as u32,
                        format!(
                            "failed to write formatted file '{}': {}",
                            path.display(),
                            error
                        ),
                    );
                    return emit_diagnostics_and_exit(
                        "fmt",
                        vec![diagnostic],
                        1,
                        output,
                        Some(&path),
                        Some(&source),
                    );
                }
            }
        }
    }

    if output.is_json() {
        let payload = json!({"filesFormatted": changed, "filesChecked": processed});
        let success = !check || changed == 0;
        print_envelope(
            "fmt",
            success,
            vec![],
            vec![],
            payload,
            None,
            None,
            if check && changed > 0 { 1 } else { 0 },
            output,
        );
    } else if !output.quiet {
        if check {
            if changed == 0 {
                println!("Checked {} file(s)", processed);
            } else {
                println!("Would format {} file(s)", changed);
            }
        } else {
            println!("Formatted {} file(s)", changed);
        }
    }

    if check && changed > 0 {
        Err(1)
    } else {
        Ok(())
    }
}

fn lint_command(files: Vec<String>, fix: bool, output: &CliOutputOptions) -> Result<(), i32> {
    ensure_project_ready_or_exit(output)?;
    let selected_files = selected_source_files(files, true);
    if selected_files.is_empty() {
        if output.is_json() {
            let payload =
                json!({"filesLinted": 0, "errorCount": 0, "warningCount": 0, "fixedCount": 0});
            print_envelope("lint", true, vec![], vec![], payload, None, None, 0, output);
        } else if !output.quiet {
            println!("Linted 0 file(s)");
        }
        return Ok(());
    }

    let mut processed = 0usize;
    let mut had_error = false;
    let mut fixed = 0usize;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for file in selected_files {
        processed += 1;
        let path = PathBuf::from(&file);
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                let diagnostic = Diagnostic::error(
                    e5::OUTPUT_ERROR as u32,
                    format!("failed to read source file '{}': {}", path.display(), error),
                );
                return emit_diagnostics_and_exit(
                    "lint",
                    vec![diagnostic],
                    1,
                    output,
                    Some(&path),
                    None,
                );
            }
        };

        let result = lint_with_options(&source, fix);
        let (file_errors, file_warnings) =
            split_and_convert_diagnostics(&result.diagnostics, Some(&path), Some(&source));
        had_error |= !file_errors.is_empty();
        errors.extend(file_errors);
        warnings.extend(file_warnings);

        if let Some(fixed_source) = result.fixed_source {
            if fix && fixed_source != source {
                if let Err(error) = fs::write(&path, fixed_source) {
                    let diagnostic = Diagnostic::error(
                        e5::OUTPUT_ERROR as u32,
                        format!("failed to write fixed file '{}': {}", path.display(), error),
                    );
                    return emit_diagnostics_and_exit(
                        "lint",
                        vec![diagnostic],
                        1,
                        output,
                        Some(&path),
                        Some(&source),
                    );
                }
                fixed += 1;
            }
        }
    }

    if output.is_json() {
        let payload = json!({
            "filesLinted": processed,
            "errorCount": errors.len(),
            "warningCount": warnings.len(),
            "fixedCount": fixed,
        });
        print_envelope(
            "lint",
            !had_error,
            errors,
            warnings,
            payload,
            None,
            None,
            if had_error { 1 } else { 0 },
            output,
        );
    } else if !output.quiet {
        if fix {
            println!("Fixed {} file(s)", fixed);
        }
        if had_error {
            println!("Linted {} file(s)", processed);
        } else {
            println!("Linted {} file(s)", processed);
        }
    }

    if had_error {
        Err(1)
    } else {
        Ok(())
    }
}

fn install_command(
    target: Option<String>,
    dev: bool,
    allow_scripts: bool,
    output: &CliOutputOptions,
) -> Result<(), i32> {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!("failed to read current directory: {}", error),
            );
            return emit_diagnostics_and_exit("install", vec![diagnostic], 1, output, None, None);
        }
    };
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    let result = install_project(
        project_root,
        InstallOptions {
            target,
            dev,
            allow_scripts,
        },
    );
    match result {
        Ok(summary) => {
            if output.is_json() {
                let payload = json!({
                    "manifestPath": summary.manifest_path,
                    "lockPath": summary.lock_path,
                    "installed": summary.installed,
                    "updated": [],
                    "removed": [],
                });
                print_envelope(
                    "install",
                    true,
                    vec![],
                    vec![],
                    payload,
                    None,
                    None,
                    0,
                    output,
                );
            } else if !output.quiet {
                println!("Installed {} package(s)", summary.installed.len());
            }
            Ok(())
        }
        Err(diagnostics) => {
            emit_diagnostics_and_exit("install", diagnostics, 1, output, None, None)
        }
    }
}

fn load_policy_or_exit(
    sandbox: Option<PathBuf>,
    output: &CliOutputOptions,
) -> Result<Option<SandboxPolicy>, i32> {
    match sandbox {
        Some(path) => match load_sandbox_policy(&path) {
            Ok(policy) => Ok(Some(policy)),
            Err(diagnostics) => {
                emit_diagnostics_and_exit("policy", diagnostics, 5, output, Some(&path), None)
                    .map(|_| None)
            }
        },
        None => Ok(None),
    }
}

fn ensure_project_ready_or_exit(output: &CliOutputOptions) -> Result<(), i32> {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!("failed to read current directory: {}", error),
            );
            return emit_diagnostics_and_exit("cli", vec![diagnostic], 1, output, None, None);
        }
    };
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);
    match ensure_project_ready(project_root) {
        Ok(()) => Ok(()),
        Err(diagnostic) => {
            emit_diagnostics_and_exit("cli", vec![diagnostic], 1, output, None, None)
        }
    }
}

fn selected_source_files(files: Vec<String>, discover: bool) -> Vec<String> {
    if files.is_empty() && discover {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_root = discover_project_root(&cwd).unwrap_or(cwd);
        discover_source_files(&project_root)
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect()
    } else {
        files
    }
}

fn single_or_error(
    files: Vec<String>,
    command: &str,
    output: &CliOutputOptions,
) -> Result<Option<PathBuf>, i32> {
    match files.as_slice() {
        [] => {
            let diagnostic = Diagnostic::error(
                e5::MISSING_REQUIRED_ARGUMENT as u32,
                format!("{} requires at least one source file", command),
            );
            emit_diagnostics_and_exit(command, vec![diagnostic], 1, output, None, None)
                .map(|_| None)
        }
        [file] => Ok(Some(PathBuf::from(file))),
        _ => {
            let diagnostic = Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                format!(
                    "{} accepts only one primary source file in this stage",
                    command
                ),
            );
            emit_diagnostics_and_exit(command, vec![diagnostic], 1, output, None, None)
                .map(|_| None)
        }
    }
}

fn validate_runtime_entrypoint(source: &PathBuf) -> Result<(), Diagnostic> {
    if is_declaration_only_source_file(source) {
        Err(Diagnostic::error(
            e5::INVALID_PRIMARY_INPUT_KIND as u32,
            format!(
                "declaration-only file '{}' cannot be used as a runtime entrypoint",
                source.display()
            ),
        )
        .with_suggestion("use `kali check` for declaration-only files"))
    } else {
        Ok(())
    }
}

fn matches_test_filter(file: &str, pattern: &str) -> bool {
    let path = PathBuf::from(file);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file);
    file.contains(pattern) || name.contains(pattern)
}

fn print_envelope(
    command: &str,
    success: bool,
    errors: Vec<Value>,
    warnings: Vec<Value>,
    payload: Value,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: i32,
    output: &CliOutputOptions,
) {
    let value = output::emit_envelope_value(
        command,
        success,
        Value::Array(errors),
        Value::Array(warnings),
        payload,
        stdout,
        stderr,
        exit_code,
    );
    if output.pretty {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("serialize json envelope")
        );
    } else {
        println!(
            "{}",
            serde_json::to_string(&value).expect("serialize json envelope")
        );
    }
}

fn emit_diagnostics_and_exit(
    command: &str,
    diagnostics: Vec<Diagnostic>,
    exit_code: i32,
    output: &CliOutputOptions,
    source_path: Option<&Path>,
    source_text: Option<&str>,
) -> Result<(), i32> {
    if output.is_json() {
        let (errors, warnings) =
            split_and_convert_diagnostics(&diagnostics, source_path, source_text);
        print_envelope(
            command,
            errors.is_empty(),
            errors,
            warnings,
            Value::Null,
            None,
            None,
            exit_code,
            output,
        );
    } else {
        for diagnostic in diagnostics {
            eprintln!("{}", diagnostic);
        }
    }
    Err(exit_code)
}

fn split_and_convert_diagnostics(
    diagnostics: &[Diagnostic],
    source_path: Option<&Path>,
    source_text: Option<&str>,
) -> (Vec<Value>, Vec<Value>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for diagnostic in diagnostics {
        let value = output::diagnostic_to_json(diagnostic, source_path, source_text, "error");
        if diagnostic.is_error() {
            errors.push(value);
        } else {
            warnings.push(value);
        }
    }
    (errors, warnings)
}

fn single_diagnostic_to_values(
    diagnostic: Diagnostic,
    source_path: Option<&Path>,
    source_text: Option<&str>,
) -> (Vec<Value>, Vec<Value>) {
    let diagnostics = vec![diagnostic];
    split_and_convert_diagnostics(&diagnostics, source_path, source_text)
}

fn build_mode_name(mode: build::BuildMode) -> &'static str {
    match mode {
        build::BuildMode::Fast => "fast",
        build::BuildMode::Release => "release",
        build::BuildMode::ReleaseAdvanced => "release-advanced",
    }
}

fn artifact_kind_for_build(out_dir: Option<&Path>) -> &'static str {
    if out_dir.is_some() {
        "executable"
    } else {
        "executable"
    }
}

fn source_hash_for_file(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("sha256-{:x}", hasher.finalize()))
}
