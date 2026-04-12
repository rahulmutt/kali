//! CLI interface for the Kali compiler.

use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};

use kali_error::{Diagnostic, _error_codes::e5};
use kali_sandbox::SandboxPolicy;

pub mod build;

pub fn discover_source_files(root: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut discovered = Vec::new();
    collect_source_files(root.as_ref(), &mut discovered);
    discovered
}

fn collect_source_files(dir: &Path, discovered: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if matches!(name, ".git" | "node_modules" | "target" | ".kali") || name.starts_with('.')
            {
                continue;
            }
            collect_source_files(&path, discovered);
            continue;
        }

        if is_source_file(&path) {
            discovered.push(path);
        }
    }
}

fn is_source_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    if name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts") {
        return false;
    }

    name.ends_with(".ts")
        || name.ends_with(".tsx")
        || name.ends_with(".js")
        || name.ends_with(".jsx")
        || name.ends_with(".mts")
        || name.ends_with(".cts")
}

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
        /// Sandbox policy file to validate
        #[arg(long)]
        sandbox: Option<PathBuf>,
        /// Source files to check
        files: Vec<String>,
    },
    #[command(name = "build")]
    /// Build source files
    Build {
        /// Sandbox policy file to validate and embed
        #[arg(long)]
        sandbox: Option<PathBuf>,
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
        /// Sandbox policy file to enforce
        #[arg(long)]
        sandbox: Option<PathBuf>,
        /// Source files to run
        files: Vec<String>,
    },
    #[command(name = "test")]
    /// Test source files
    Test {
        /// Sandbox policy file to enforce
        #[arg(long)]
        sandbox: Option<PathBuf>,
        /// Only run tests matching this pattern
        #[arg(long)]
        filter: Option<String>,
        /// Emit coverage data once the report contract is stabilized
        #[arg(long)]
        coverage: bool,
        /// Source files to test
        files: Vec<String>,
    },
    #[command(name = "init")]
    /// Initialize a new Kali project
    Init,
    #[command(name = "install")]
    /// Install dependencies
    Install {
        /// Package target to add or reconcile
        target: Option<String>,
        /// Add the target to devDependencies
        #[arg(long)]
        dev: bool,
        /// Allow npm lifecycle scripts during installation
        #[arg(long = "allow-scripts")]
        allow_scripts: bool,
    },
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

pub fn is_declaration_only_source_file(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(name)
            if name.ends_with(".d.ts")
                || name.ends_with(".d.mts")
                || name.ends_with(".d.cts")
    )
}

pub fn discover_test_files(root: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut discovered = Vec::new();
    collect_test_files(root.as_ref(), &mut discovered);
    discovered
}

pub fn load_sandbox_policy(path: impl AsRef<Path>) -> Result<SandboxPolicy, Vec<Diagnostic>> {
    let path = path.as_ref();
    SandboxPolicy::from_file(path)
}

pub fn sandbox_policy_diagnostics(path: impl AsRef<Path>, error: impl ToString) -> Vec<Diagnostic> {
    vec![Diagnostic::error(
        e5::INVALID_POLICY as u32,
        format!(
            "failed to load sandbox policy '{}': {}",
            path.as_ref().display(),
            error.to_string()
        ),
    )]
}

fn collect_test_files(dir: &Path, discovered: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_test_files(&path, discovered);
            continue;
        }

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if name.ends_with(".test.ts")
            || name.ends_with(".spec.ts")
            || name.ends_with(".test.js")
            || name.ends_with(".spec.js")
        {
            discovered.push(path);
        }
    }
}
