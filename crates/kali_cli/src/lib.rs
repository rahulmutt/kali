//! CLI interface for the Kali compiler.

use clap::Parser;
use std::path::PathBuf;

pub mod build;

#[derive(Parser, Debug)]
#[command(name = "kali")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    #[command(name = "check")]
    /// Type-check source files
    Check {
        /// Source files to check
        files: Vec<String>,
    },
    #[command(name = "build")]
    /// Build source files
    Build {
        /// Source files to build
        files: Vec<String>,
        /// Explicit fast build mode
        #[arg(long, conflicts_with_all = ["release", "release_advanced"])]
        fast: bool,
        /// Explicit release build mode
        #[arg(long, conflicts_with_all = ["fast", "release_advanced"])]
        release: bool,
        /// Explicit release-advanced build mode
        #[arg(long = "release-advanced", conflicts_with_all = ["fast", "release"])]
        release_advanced: bool,
        /// Output directory for artifacts
        #[arg(long)]
        out_dir: Option<PathBuf>,
    },
    #[command(name = "run")]
    /// Run source files
    Run {
        /// Source files to run
        files: Vec<String>,
    },
    #[command(name = "test")]
    /// Test source files
    Test {
        /// Source files to test
        files: Vec<String>,
    },
    #[command(name = "init")]
    /// Initialize a new Kali project
    Init,
    #[command(name = "install")]
    /// Install dependencies
    Install,
    #[command(name = "fmt")]
    /// Format source files
    Fmt {
        /// Source files to format
        files: Vec<String>,
    },
    #[command(name = "lint")]
    /// Lint source files
    Lint {
        /// Source files to lint
        files: Vec<String>,
    },
}

impl Commands {
    pub fn build_mode(&self) -> build::BuildMode {
        match self {
            Commands::Build {
                fast,
                release,
                release_advanced,
                ..
            } if *release_advanced => build::BuildMode::ReleaseAdvanced,
            Commands::Build { release, .. } if *release => build::BuildMode::Release,
            Commands::Build { fast, .. } if *fast => build::BuildMode::Fast,
            Commands::Build { .. } => build::BuildMode::Fast,
            _ => build::BuildMode::Fast,
        }
    }
}
