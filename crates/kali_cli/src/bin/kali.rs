/// Main CLI binary for the Kali compiler.

use kali_cli::Args;
use clap::Parser;

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
        kali_cli::Commands::Build { files: _files, mode } => {
            println!("Building with mode: {:?} (stub)", mode);
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
