//! Main CLI binary for the Kali compiler.

use clap::Parser;
use kali_cli::{
    build, discover_source_files, discover_test_files, init, is_declaration_only_source_file,
    load_sandbox_policy, Args,
};
use kali_error::{Diagnostic, _error_codes::e5};
use kali_npm::{discover_project_root, ensure_project_ready, install_project, InstallOptions};
use kali_runtime::RuntimeCtx;
use kali_sandbox::SandboxPolicy;
use std::path::PathBuf;

fn main() {
    let args = Args::parse();

    if args.command.is_none() {
        // Default behavior: print version and exit
        println!("kali 0.1.0");
        return;
    }

    match args.command.unwrap() {
        kali_cli::Commands::Check { sandbox, files } => {
            if let Err(exit_code) = ensure_installed_or_exit() {
                std::process::exit(exit_code);
            }
            let policy = match load_policy_or_exit(sandbox) {
                Ok(policy) => policy,
                Err(exit_code) => std::process::exit(exit_code),
            };
            if let Err(exit_code) = check_files(files, policy.as_ref()) {
                std::process::exit(exit_code);
            }
        }
        kali_cli::Commands::Build {
            sandbox,
            files,
            fast,
            release,
            release_advanced,
            out_dir,
        } => {
            if files.is_empty() {
                eprintln!("Error[E5001]: build requires at least one source file");
                std::process::exit(1);
            }

            if let Err(exit_code) = ensure_installed_or_exit() {
                std::process::exit(exit_code);
            }
            let policy = match load_policy_or_exit(sandbox) {
                Ok(policy) => policy,
                Err(exit_code) => std::process::exit(exit_code),
            };

            let mode = build::build_mode_from_flags(fast, release, release_advanced);
            let out_dir_path = out_dir.as_deref();

            for file in files {
                match build::build_source_file(&file, mode, out_dir_path, policy.as_ref()) {
                    Ok(output) => {
                        println!("Built {} -> {}", file, output.output_path.display());
                    }
                    Err(diagnostics) => {
                        for diagnostic in diagnostics {
                            eprintln!("{}", diagnostic);
                        }
                        std::process::exit(1);
                    }
                }
            }
        }
        kali_cli::Commands::Run { sandbox, files } => {
            if let Err(exit_code) = ensure_installed_or_exit() {
                std::process::exit(exit_code);
            }
            let policy = match load_policy_or_exit(sandbox) {
                Ok(policy) => policy,
                Err(exit_code) => std::process::exit(exit_code),
            };
            if let Err(exit_code) = run_files(files, policy.as_ref()) {
                std::process::exit(exit_code);
            }
        }
        kali_cli::Commands::Test {
            sandbox,
            files,
            filter,
            coverage,
        } => {
            if let Err(exit_code) = ensure_installed_or_exit() {
                std::process::exit(exit_code);
            }
            let policy = match load_policy_or_exit(sandbox) {
                Ok(policy) => policy,
                Err(exit_code) => std::process::exit(exit_code),
            };
            if let Err(exit_code) = test_files(files, filter, coverage, policy.as_ref()) {
                std::process::exit(exit_code);
            }
        }
        kali_cli::Commands::Init { lib } => match init::init_current_directory(lib) {
            Ok(summary) => {
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
            Err(diagnostic) => {
                eprintln!("{}", diagnostic);
                std::process::exit(5);
            }
        },
        kali_cli::Commands::Install {
            target,
            dev,
            allow_scripts,
        } => {
            let cwd = match std::env::current_dir() {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("Error[E6100]: failed to read current directory: {}", error);
                    std::process::exit(1);
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
                    if summary.installed.is_empty() {
                        println!("Installed 0 package(s)");
                    } else {
                        println!("Installed {} package(s)", summary.installed.len());
                    }
                }
                Err(diagnostics) => {
                    print_diagnostics(&diagnostics);
                    std::process::exit(1);
                }
            }
        }
        kali_cli::Commands::Fmt { files: _files } => {
            println!("Formatting files... (stub)");
        }
        kali_cli::Commands::Lint { files: _files } => {
            println!("Linting files... (stub)");
        }
    }
}

fn ensure_installed_or_exit() -> Result<(), i32> {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("Error[E6100]: failed to read current directory: {}", error);
            return Err(1);
        }
    };
    let project_root = discover_project_root(&cwd).unwrap_or(cwd);

    match ensure_project_ready(project_root) {
        Ok(()) => Ok(()),
        Err(diagnostic) => {
            eprintln!("{}", diagnostic);
            Err(1)
        }
    }
}

fn load_policy_or_exit(sandbox: Option<PathBuf>) -> Result<Option<SandboxPolicy>, i32> {
    match sandbox {
        Some(path) => match load_sandbox_policy(&path) {
            Ok(policy) => Ok(Some(policy)),
            Err(diagnostics) => {
                print_diagnostics(&diagnostics);
                Err(5)
            }
        },
        None => Ok(None),
    }
}

fn check_files(files: Vec<String>, policy: Option<&SandboxPolicy>) -> Result<(), i32> {
    if let Some(policy) = policy {
        if let Err(diagnostics) = policy.validate() {
            print_diagnostics(&diagnostics);
            return Err(5);
        }
    }

    let selected_files = if files.is_empty() {
        discover_source_files(".")
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
    } else {
        files
    };

    if selected_files.is_empty() {
        println!("Checked 0 file(s)");
        return Ok(());
    }

    let mut checked = 0usize;
    let mut failed = 0usize;

    for file in selected_files {
        checked += 1;
        match build::check_source_file(&file) {
            Ok(()) => {}
            Err(diagnostics) => {
                failed += 1;
                print_diagnostics(&diagnostics);
            }
        }
    }

    if failed == 0 {
        println!("Checked {} file(s)", checked);
        Ok(())
    } else {
        Err(1)
    }
}

fn run_files(files: Vec<String>, policy: Option<&SandboxPolicy>) -> Result<(), i32> {
    let Some(source) = single_or_error(files, "run") else {
        return Err(1);
    };

    if let Err(diagnostic) = validate_runtime_entrypoint(&source) {
        eprintln!("{}", diagnostic);
        return Err(1);
    }

    let wasm_bytes = match build::compile_source_file(&source, build::BuildMode::Fast) {
        Ok(bytes) => bytes,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };

    let runtime = RuntimeCtx::new(policy.cloned());
    match runtime.execute(&wasm_bytes) {
        Ok(_outcome) => Ok(()),
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            Err(1)
        }
    }
}

fn test_files(
    files: Vec<String>,
    filter: Option<String>,
    coverage: bool,
    policy: Option<&SandboxPolicy>,
) -> Result<(), i32> {
    if coverage {
        eprintln!(
            "{}",
            Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "test coverage reporting is unavailable in this phase"
            )
        );
        return Err(1);
    }

    let selected_files = if files.is_empty() {
        discover_test_files(".")
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
    } else {
        files
    };

    if selected_files.is_empty() {
        println!("ok 0");
        return Ok(());
    }

    let mut valid_files = Vec::new();
    for file in selected_files {
        let source = PathBuf::from(&file);
        if let Err(diagnostic) = validate_runtime_entrypoint(&source) {
            eprintln!("{}", diagnostic);
            return Err(1);
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
        println!("ok 0");
        return Ok(());
    }

    let runtime = RuntimeCtx::new(policy.cloned());
    let mut passed = 0usize;
    let mut failed = 0usize;

    for file in filtered_files {
        let source = PathBuf::from(&file);

        let wasm_bytes = match build::compile_source_file(&source, build::BuildMode::Fast) {
            Ok(bytes) => bytes,
            Err(diagnostics) => {
                print_diagnostics(&diagnostics);
                failed += 1;
                continue;
            }
        };

        match runtime.execute_tests(&wasm_bytes) {
            Ok(outcome) => {
                passed += outcome.tests_run.saturating_sub(outcome.tests_failed);
                failed += outcome.tests_failed;
            }
            Err(diagnostics) => {
                print_diagnostics(&diagnostics);
                failed += 1;
            }
        }
    }

    if failed == 0 {
        println!("ok {}", passed);
        Ok(())
    } else {
        println!("FAILED {}", failed);
        Err(1)
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

fn single_or_error(files: Vec<String>, command: &str) -> Option<PathBuf> {
    match files.as_slice() {
        [] => {
            eprintln!(
                "Error[E5001]: {} requires at least one source file",
                command
            );
            None
        }
        [file] => Some(PathBuf::from(file)),
        _ => {
            eprintln!(
                "Error[E5008]: {} accepts only one primary source file in this stage",
                command
            );
            None
        }
    }
}

fn print_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!("{}", diagnostic);
    }
}
