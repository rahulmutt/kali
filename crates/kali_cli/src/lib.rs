//! CLI interface for the Kali compiler.

use clap::Parser;

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
        #[arg(long)]
        mode: Option<String>,
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
