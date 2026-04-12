//! Main CLI binary for the Kali compiler.

use clap::Parser;
use kali_cli::{build, discover_test_files, is_declaration_only_source_file, Args};
use kali_error::{Diagnostic, _error_codes::e5};
use kali_runtime::RuntimeCtx;
use std::path::PathBuf;

fn main() {
    let args = Args::parse();

    if args.command.is_none() {
        // Default behavior: print version and exit
        println!("kali 0.1.0");
        return;
    }

    match args.command.unwrap() {
        kali_cli::Commands::Check { files: _files } => {
            println!("Checking files... (stub)");
        }
        kali_cli::Commands::Build {
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

            let mode = build::build_mode_from_flags(fast, release, release_advanced);
            let out_dir_path = out_dir.as_deref();

            for file in files {
                match build::build_source_file(&file, mode, out_dir_path) {
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
        kali_cli::Commands::Run { files } => {
            if let Err(exit_code) = run_files(files) {
                std::process::exit(exit_code);
            }
        }
        kali_cli::Commands::Test { files } => {
            if let Err(exit_code) = test_files(files) {
                std::process::exit(exit_code);
            }
        }
        kali_cli::Commands::Init => {
            println!("Initializing new project... (stub)");
        }
        kali_cli::Commands::Install => {
            println!("Installing dependencies... (stub)");
        }
        kali_cli::Commands::Fmt { files: _files } => {
            println!("Formatting files... (stub)");
        }
        kali_cli::Commands::Lint { files: _files } => {
            println!("Linting files... (stub)");
        }
    }
}

fn run_files(files: Vec<String>) -> Result<(), i32> {
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

    let runtime = RuntimeCtx::new(None);
    match runtime.execute(&wasm_bytes) {
        Ok(_outcome) => Ok(()),
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            Err(1)
        }
    }
}

fn test_files(files: Vec<String>) -> Result<(), i32> {
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

    let runtime = RuntimeCtx::new(None);
    let mut passed = 0usize;
    let mut failed = 0usize;

    for file in selected_files {
        let source = PathBuf::from(&file);
        if let Err(diagnostic) = validate_runtime_entrypoint(&source) {
            eprintln!("{}", diagnostic);
            failed += 1;
            continue;
        }

        let wasm_bytes = match build::compile_source_file(&source, build::BuildMode::Fast) {
            Ok(bytes) => bytes,
            Err(diagnostics) => {
                print_diagnostics(&diagnostics);
                failed += 1;
                continue;
            }
        };

        match runtime.execute(&wasm_bytes) {
            Ok(_outcome) => passed += 1,
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
