//! Main CLI binary for the Kali compiler.

use clap::Parser;
use kali_cli::{build, Args};

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
        kali_cli::Commands::Run { files: _files } => {
            println!("Running files... (stub)");
        }
        kali_cli::Commands::Test { files: _files } => {
            println!("Testing files... (stub)");
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
