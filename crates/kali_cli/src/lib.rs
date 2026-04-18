//! CLI interface for the Kali compiler.

use clap::{Parser, ValueEnum};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

use kali_error::{_error_codes::e5, Diagnostic};
use kali_sandbox::SandboxPolicy;

pub mod build;
pub mod init;
pub mod output;

pub fn discover_source_files(root: impl AsRef<Path>) -> Vec<PathBuf> {
    discover_project_files(root.as_ref(), DiscoveryKind::Source)
}

fn discover_project_files(root: &Path, kind: DiscoveryKind) -> Vec<PathBuf> {
    let exclude_set = load_exclude_set(root);
    let root = root.to_path_buf();
    let mut discovered = Vec::new();

    let mut walk = WalkBuilder::new(&root);
    walk.hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .filter_entry(move |entry| should_descend(entry.path(), &root, &exclude_set));

    for entry in walk.build().filter_map(Result::ok) {
        let path = entry.path();
        if entry.file_type().map(|ty| ty.is_file()).unwrap_or(false)
            && matches_discovery_kind(path, kind)
        {
            discovered.push(path.to_path_buf());
        }
    }

    discovered.sort();
    discovered
}

fn is_source_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    if name.ends_with(".test.ts")
        || name.ends_with(".spec.ts")
        || name.ends_with(".test.js")
        || name.ends_with(".spec.js")
        || name.ends_with(".test.tsx")
        || name.ends_with(".spec.tsx")
        || name.ends_with(".test.mts")
        || name.ends_with(".spec.mts")
        || name.ends_with(".test.cts")
        || name.ends_with(".spec.cts")
    {
        return false;
    }

    name.ends_with(".ts")
        || name.ends_with(".tsx")
        || name.ends_with(".js")
        || name.ends_with(".jsx")
        || name.ends_with(".mts")
        || name.ends_with(".cts")
        || name.ends_with(".d.ts")
        || name.ends_with(".d.mts")
        || name.ends_with(".d.cts")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => f.write_str("text"),
            OutputFormat::Json => f.write_str("json"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ApiSurface {
    Deno,
    Node,
    Browser,
}

impl std::fmt::Display for ApiSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiSurface::Deno => f.write_str("deno"),
            ApiSurface::Node => f.write_str("node"),
            ApiSurface::Browser => f.write_str("browser"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum BundleFormat {
    Esm,
    Cjs,
}

impl std::fmt::Display for BundleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleFormat::Esm => f.write_str("esm"),
            BundleFormat::Cjs => f.write_str("cjs"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl std::fmt::Display for ColorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorChoice::Auto => f.write_str("auto"),
            ColorChoice::Always => f.write_str("always"),
            ColorChoice::Never => f.write_str("never"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "kali")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, global = true, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,

    #[arg(long, global = true)]
    pub pretty: bool,

    #[arg(long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true)]
    pub quiet: bool,

    #[arg(long, global = true, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

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
        /// Select the effective API surface
        #[arg(long, value_enum)]
        api: Option<ApiSurface>,
        /// Enable documented compatibility features
        #[arg(long = "compat", value_delimiter = ',')]
        compat: Vec<String>,
        /// Source files to check
        files: Vec<String>,
    },
    #[command(name = "build")]
    /// Build source files
    Build {
        /// Sandbox policy file to validate and embed
        #[arg(long)]
        sandbox: Option<PathBuf>,
        /// Select the effective API surface
        #[arg(long, value_enum)]
        api: Option<ApiSurface>,
        /// Enable documented compatibility features
        #[arg(long = "compat", value_delimiter = ',')]
        compat: Vec<String>,
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
        /// Override the specialization fan-out cap for this build
        #[arg(long = "max-specializations")]
        max_specializations: Option<usize>,
        /// Select the browser bundle artifact mode
        #[arg(long, conflicts_with_all = ["lib", "capi", "component"])]
        bundle: bool,
        /// Select the browser bundle output format
        #[arg(long, value_enum)]
        format: Option<BundleFormat>,
        /// Select the base library artifact mode
        #[arg(long, conflicts_with_all = ["bundle", "capi", "component"])]
        lib: bool,
        /// Select the later public C embedding artifact flow
        #[arg(long, conflicts_with_all = ["bundle", "lib", "component"])]
        capi: bool,
        /// Select the later Component Model artifact flow
        #[arg(long, conflicts_with_all = ["bundle", "lib", "capi"])]
        component: bool,
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
        /// Select the effective API surface
        #[arg(long, value_enum)]
        api: Option<ApiSurface>,
        /// Enable documented compatibility features
        #[arg(long = "compat", value_delimiter = ',')]
        compat: Vec<String>,
        /// Source files to run
        files: Vec<String>,
    },
    #[command(name = "test")]
    /// Test source files
    Test {
        /// Sandbox policy file to enforce
        #[arg(long)]
        sandbox: Option<PathBuf>,
        /// Select the effective API surface
        #[arg(long, value_enum)]
        api: Option<ApiSurface>,
        /// Enable documented compatibility features
        #[arg(long = "compat", value_delimiter = ',')]
        compat: Vec<String>,
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
    Init {
        /// Scaffold a library-oriented project template
        #[arg(long)]
        lib: bool,
    },
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
        /// Check formatting without writing files
        #[arg(long)]
        check: bool,
        /// Source files to format
        files: Vec<String>,
    },
    #[command(name = "lint")]
    /// Lint source files
    Lint {
        /// Automatically apply safe fixes
        #[arg(long)]
        fix: bool,
        /// Source files to lint
        files: Vec<String>,
    },
    #[command(name = "effects")]
    /// Analyze source-file effects
    Effects {
        /// Enable documented compatibility features
        #[arg(long = "compat", value_delimiter = ',')]
        compat: Vec<String>,
        /// Source files to analyze
        files: Vec<String>,
    },
    #[command(name = "package-effects")]
    /// Analyze registry-package effects
    PackageEffects {
        /// Registry package target to analyze
        target: String,
    },
    #[command(name = "package-audit")]
    /// Audit a registry package
    PackageAudit {
        /// Registry package target to audit
        target: String,
        /// Hidden legacy flag rejected by the command handler with E5008
        #[arg(long, hide = true)]
        preview: bool,
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
    discover_project_files(root.as_ref(), DiscoveryKind::Test)
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

fn matches_discovery_kind(path: &Path, kind: DiscoveryKind) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    match kind {
        DiscoveryKind::Source => is_source_file(path),
        DiscoveryKind::Test => {
            name.ends_with(".test.ts")
                || name.ends_with(".spec.ts")
                || name.ends_with(".test.js")
                || name.ends_with(".spec.js")
                || name.ends_with(".test.tsx")
                || name.ends_with(".spec.tsx")
                || name.ends_with(".test.mts")
                || name.ends_with(".spec.mts")
                || name.ends_with(".test.cts")
                || name.ends_with(".spec.cts")
        }
    }
}

#[derive(Clone, Copy)]
enum DiscoveryKind {
    Source,
    Test,
}

fn should_descend(path: &Path, root: &Path, exclude_set: &GlobSet) -> bool {
    if path == root {
        return true;
    }

    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return true;
    };

    if matches!(name, ".git" | "node_modules" | ".kali-cache" | "target") || name.starts_with('.') {
        return false;
    }

    if path.is_dir() && path != root && path.join("kali.json").exists() {
        return false;
    }

    let rel = path.strip_prefix(root).unwrap_or(path);
    !exclude_set.is_match(rel)
}

fn load_exclude_set(root: &Path) -> GlobSet {
    let manifest_path = root.join("kali.json");
    let Ok(raw) = fs::read_to_string(manifest_path) else {
        return GlobSetBuilder::new().build().expect("empty globset");
    };

    let Ok(manifest) = serde_json::from_str::<Value>(&raw) else {
        return GlobSetBuilder::new().build().expect("empty globset");
    };

    let Some(patterns) = manifest.get("exclude").and_then(|value| value.as_array()) else {
        return GlobSetBuilder::new().build().expect("empty globset");
    };

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let Some(pattern) = pattern.as_str() else {
            continue;
        };
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }

    builder
        .build()
        .unwrap_or_else(|_| GlobSetBuilder::new().build().expect("empty globset"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tempfile::tempdir;

    #[test]
    fn discovers_source_files_and_declaration_files() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).unwrap();
        fs::write(dir.path().join("main.ts"), "const main = 1;").unwrap();
        fs::write(dir.path().join("types.d.ts"), "declare const x: number;").unwrap();
        fs::create_dir_all(dir.path().join(".hidden")).unwrap();
        fs::write(
            dir.path().join(".hidden").join("skip.ts"),
            "const skip = 1;",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("child")).unwrap();
        fs::write(
            dir.path().join("child").join("kali.json"),
            r#"{"schemaVersion":1}"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("child").join("nested.ts"),
            "const nested = 1;",
        )
        .unwrap();

        let files = discover_source_files(dir.path());
        assert!(files.contains(&dir.path().join("main.ts")));
        assert!(files.contains(&dir.path().join("types.d.ts")));
        assert!(!files.contains(&dir.path().join(".hidden").join("skip.ts")));
        assert!(!files.contains(&dir.path().join("child").join("nested.ts")));
    }

    #[test]
    fn discover_source_files_respects_kali_json_exclude() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("kali.json"),
            r#"{"schemaVersion":1,"exclude":["dist/**"]}"#,
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("dist")).unwrap();
        fs::write(dir.path().join("dist").join("bundle.ts"), "const x = 1;").unwrap();

        let files = discover_source_files(dir.path());
        assert!(!files.contains(&dir.path().join("dist").join("bundle.ts")));
    }

    #[test]
    fn build_command_parses_max_specializations_override() {
        let args = Args::parse_from(["kali", "build", "--max-specializations", "32", "main.ts"]);
        match args.command {
            Some(Commands::Build {
                max_specializations,
                ..
            }) => {
                assert_eq!(max_specializations, Some(32));
            }
            other => panic!("expected build command, got {other:?}"),
        }
    }
}
