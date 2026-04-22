//! Package management for Kali (npm/JSR registry support).

use base64::Engine;
use flate2::read::GzDecoder;
use kali_error::{_error_codes::e5, _error_codes::e6, Diagnostic};
use reqwest::blocking::Client;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
};
use tar::Archive;

const MANIFEST_SCHEMA: u32 = 1;
const LOCK_VERSION: u32 = 1;
const DEFAULT_NPM_REGISTRY: &str = "https://registry.npmjs.org";
const NODE_ONLY_HOST_APIS: &[&str] = &[
    "fs",
    "fs/promises",
    "path",
    "os",
    "url",
    "crypto",
    "events",
    "stream",
    "http",
    "https",
    "process",
    "buffer",
    "util",
    "assert",
    "child_process",
];

static REGISTRY_METADATA_CACHE: OnceLock<Mutex<BTreeMap<String, serde_json::Value>>> =
    OnceLock::new();

/// Top-level Kali project manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct ProjectManifest {
    pub schema_version: u32,
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler_options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compat: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub include: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub exclude: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub imports: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub dev_dependencies: BTreeMap<String, String>,
}

impl ProjectManifest {
    pub fn minimal() -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA,
            ..Self::default()
        }
    }

    pub fn is_minimal(&self) -> bool {
        self.schema_version == MANIFEST_SCHEMA
            && self.schema_uri.is_none()
            && self.compiler_options.is_none()
            && self.compat.is_none()
            && self.sandbox.is_none()
            && self.include.is_empty()
            && self.exclude.is_empty()
            && self.imports.is_empty()
            && self.dependencies.is_empty()
            && self.dev_dependencies.is_empty()
    }
}

fn package_host_fit_context_for_manifest(manifest: &ProjectManifest) -> PackageHostFitContext {
    let Some(options) = manifest.compiler_options.as_ref() else {
        return PackageHostFitContext::DefaultStandalone;
    };
    let Some(options) = options.as_object() else {
        return PackageHostFitContext::DefaultStandalone;
    };

    match options.get("apiSurface").and_then(|value| value.as_str()) {
        Some("node") => PackageHostFitContext::Node,
        _ => PackageHostFitContext::DefaultStandalone,
    }
}

/// Lockfile metadata for a resolved package.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct LockFile {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema_uri: Option<String>,
    pub version: u32,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub packages: BTreeMap<String, LockedPackage>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub raw_urls: BTreeMap<String, RawUrlEntry>,
}

impl LockFile {
    pub fn minimal() -> Self {
        Self {
            version: LOCK_VERSION,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LockedPackage {
    pub registry: String,
    pub integrity: String,
    pub resolved: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RawUrlEntry {
    pub integrity: String,
    pub cached: String,
}

#[derive(Debug, Clone)]
pub enum PackageTarget {
    Registry {
        registry: String,
        name: String,
        version: Option<String>,
    },
    RawUrl(String),
}

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub target: Option<String>,
    pub dev: bool,
    pub allow_scripts: bool,
    pub suppress_script_output: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackageHostFitContext {
    DefaultStandalone,
    Node,
}

impl PackageHostFitContext {
    fn allows_node_only_host_apis(self) -> bool {
        matches!(self, Self::Node)
    }
}

#[derive(Debug, Clone)]
pub struct InstallSummary {
    pub manifest_path: Option<PathBuf>,
    pub lock_path: Option<PathBuf>,
    pub installed: Vec<String>,
}

/// The outcome of a registry package audit.
#[derive(Debug, Clone)]
pub struct RegistryPackageAudit {
    pub registry: String,
    pub name: String,
    pub version: String,
    pub findings: Vec<Diagnostic>,
}

pub fn discover_project_root(start: impl AsRef<Path>) -> Option<PathBuf> {
    let mut current = start.as_ref().canonicalize().ok()?;
    loop {
        if current.join("kali.json").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

pub fn load_manifest(root: impl AsRef<Path>) -> Result<Option<ProjectManifest>, Diagnostic> {
    let path = root.as_ref().join("kali.json");
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path).map_err(|error| {
        Diagnostic::error(
            e6::INVALID_LOCK_FILE as u32,
            format!("failed to read manifest '{}': {}", path.display(), error),
        )
    })?;
    let manifest: ProjectManifest = serde_json::from_str(&contents).map_err(|error| {
        Diagnostic::error(
            e6::INVALID_PACKAGE_SPECIFIER as u32,
            format!("failed to parse manifest '{}': {}", path.display(), error),
        )
    })?;
    if manifest.schema_version != MANIFEST_SCHEMA {
        return Err(Diagnostic::error(
            e6::INVALID_PACKAGE_SPECIFIER as u32,
            format!(
                "manifest '{}' has unsupported schemaVersion {}",
                path.display(),
                manifest.schema_version
            ),
        ));
    }
    Ok(Some(manifest))
}

pub fn save_manifest(
    root: impl AsRef<Path>,
    manifest: &ProjectManifest,
) -> Result<PathBuf, Diagnostic> {
    let path = root.as_ref().join("kali.json");
    let json = serde_json::to_string_pretty(manifest).map_err(|error| {
        Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to serialize manifest '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    fs::write(&path, json).map_err(|error| {
        Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!("failed to write manifest '{}': {}", path.display(), error),
        )
    })?;
    Ok(path)
}

pub fn load_lock(root: impl AsRef<Path>) -> Result<Option<LockFile>, Diagnostic> {
    let path = root.as_ref().join("kali.lock");
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&path).map_err(|error| {
        Diagnostic::error(
            e6::INVALID_LOCK_FILE as u32,
            format!("failed to read lock file '{}': {}", path.display(), error),
        )
    })?;
    let lock: LockFile = serde_json::from_str(&contents).map_err(|error| {
        Diagnostic::error(
            e6::INVALID_LOCK_FILE as u32,
            format!("failed to parse lock file '{}': {}", path.display(), error),
        )
    })?;
    if lock.version != LOCK_VERSION {
        return Err(Diagnostic::error(
            e6::INVALID_LOCK_FILE as u32,
            format!(
                "lock file '{}' has unsupported version {}",
                path.display(),
                lock.version
            ),
        ));
    }
    Ok(Some(lock))
}

pub fn save_lock(root: impl AsRef<Path>, lock: &LockFile) -> Result<PathBuf, Diagnostic> {
    let path = root.as_ref().join("kali.lock");
    let json = serde_json::to_string_pretty(lock).map_err(|error| {
        Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to serialize lock file '{}': {}",
                path.display(),
                error
            ),
        )
    })?;
    fs::write(&path, json).map_err(|error| {
        Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!("failed to write lock file '{}': {}", path.display(), error),
        )
    })?;
    Ok(path)
}

pub fn ensure_project_ready(root: impl AsRef<Path>) -> Result<(), Diagnostic> {
    let root = root.as_ref();
    let manifest = match load_manifest(root)? {
        Some(manifest) => manifest,
        None => ProjectManifest::minimal(),
    };

    let root_keys = manifest_registry_package_keys(&manifest);
    let declared_raw_urls = discover_install_time_raw_urls(root, &manifest)?;

    let Some(lock) = load_lock(root)? else {
        if root_keys.is_empty() && declared_raw_urls.is_empty() {
            return Ok(());
        }

        return Err(Diagnostic::error(
            e6::INSTALL_REQUIRED as u32,
            "package installation is required before this command can proceed",
        ));
    };

    let reachable = collect_reachable_registry_packages(&lock, &root_keys)?;
    if reachable.len() != lock.packages.len() {
        return Err(Diagnostic::error(
            e6::INSTALL_REQUIRED as u32,
            "package installation is required before this command can proceed",
        ));
    }

    for key in reachable {
        let Some((name, _version)) = split_package_key(&key) else {
            return Err(Diagnostic::error(
                e6::INVALID_LOCK_FILE as u32,
                format!("invalid package key '{}' in lock file", key),
            ));
        };
        let install_name = install_name_from_package(name);
        let install_dir = root.join("node_modules").join(&install_name);
        if !install_dir.exists() {
            return Err(Diagnostic::error(
                e6::INSTALL_REQUIRED as u32,
                format!(
                    "package '{}' must be installed before this command can proceed",
                    name
                ),
            ));
        }

        let cache_dir = root.join(".kali-cache").join("packages").join(&key);
        if !cache_dir.exists() {
            return Err(Diagnostic::error(
                e6::INSTALL_REQUIRED as u32,
                format!(
                    "package '{}' cache must be materialized before this command can proceed",
                    name
                ),
            ));
        }
    }

    if !declared_raw_urls.is_empty() {
        for url in &declared_raw_urls {
            match lock.raw_urls.get(url) {
                Some(entry) if Path::new(&entry.cached).exists() => {}
                _ => {
                    return Err(Diagnostic::error(
                        e6::INSTALL_REQUIRED as u32,
                        format!(
                            "raw URL '{}' must be installed before this command can proceed",
                            url
                        ),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn manifest_registry_package_keys(manifest: &ProjectManifest) -> Vec<String> {
    manifest
        .dependencies
        .iter()
        .chain(manifest.dev_dependencies.iter())
        .map(|(name, version)| package_key(name, version))
        .collect()
}

fn split_package_key(key: &str) -> Option<(&str, &str)> {
    key.rsplit_once('@')
}

fn ensure_lock_install_name_unique(
    install_names: &mut BTreeMap<String, String>,
    key: &str,
) -> Result<(), Diagnostic> {
    let Some((name, _version)) = split_package_key(key) else {
        return Err(Diagnostic::error(
            e6::INVALID_LOCK_FILE as u32,
            format!("invalid package key '{}' in lock file", key),
        ));
    };

    let install_name = install_name_from_package(name);
    match install_names.entry(install_name) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(key.to_string());
            Ok(())
        }
        std::collections::btree_map::Entry::Occupied(entry) if entry.get() == key => Ok(()),
        std::collections::btree_map::Entry::Occupied(entry) => Err(Diagnostic::error(
            e6::VERSION_MISMATCH as u32,
            format!(
                "packages '{}' and '{}' would both materialize to node_modules/{}",
                entry.get(),
                key,
                entry.key()
            ),
        )),
    }
}

fn collect_reachable_registry_packages(
    lock: &LockFile,
    root_keys: &[String],
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut reachable = BTreeSet::new();
    let mut install_names = BTreeMap::new();
    let mut stack = root_keys.to_vec();

    while let Some(key) = stack.pop() {
        if !reachable.insert(key.clone()) {
            continue;
        }

        let package = lock.packages.get(&key).ok_or_else(|| {
            Diagnostic::error(
                e6::INSTALL_REQUIRED as u32,
                format!(
                    "package '{}' must be installed before this command can proceed",
                    key
                ),
            )
        })?;

        ensure_lock_install_name_unique(&mut install_names, &key)?;

        for (dep_name, dep_version) in &package.dependencies {
            stack.push(package_key(dep_name, dep_version));
        }
    }

    Ok(reachable)
}

fn prune_unreachable_registry_packages(
    root: &Path,
    lock: &mut LockFile,
    reachable: &BTreeSet<String>,
) -> Result<(), Vec<Diagnostic>> {
    let unreachable = lock
        .packages
        .keys()
        .filter(|key| !reachable.contains(*key))
        .cloned()
        .collect::<Vec<_>>();

    if unreachable.is_empty() {
        return Ok(());
    }

    let remaining_install_names = lock
        .packages
        .keys()
        .filter(|key| reachable.contains(*key))
        .filter_map(|key| {
            split_package_key(key).map(|(name, _version)| install_name_from_package(name))
        })
        .collect::<BTreeSet<_>>();

    for key in unreachable {
        lock.packages.remove(&key);

        let cache_dir = root.join(".kali-cache").join("packages").join(&key);
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to remove stale package cache '{}': {}",
                        cache_dir.display(),
                        error
                    ),
                )]
            })?;
        }

        if let Some((name, _version)) = split_package_key(&key) {
            let install_name = install_name_from_package(name);
            if !remaining_install_names.contains(&install_name) {
                let install_path = root.join("node_modules").join(&install_name);
                if install_path.exists() {
                    fs::remove_dir_all(&install_path).map_err(|error| {
                        vec![Diagnostic::error(
                            e6::INSTALL_FAILED as u32,
                            format!(
                                "failed to remove stale install path '{}': {}",
                                install_path.display(),
                                error
                            ),
                        )]
                    })?;
                }
            }
        }
    }

    Ok(())
}

fn discover_install_source_files(root: &Path) -> Result<Vec<PathBuf>, Diagnostic> {
    let mut files = Vec::new();
    collect_install_source_files(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_install_source_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), Diagnostic> {
    let entries = fs::read_dir(current).map_err(|error| {
        Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to read directory '{}': {}",
                current.display(),
                error
            ),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!("failed to read entry in '{}': {}", current.display(), error),
            )
        })?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if path.is_dir() {
            if matches!(
                name.as_ref(),
                ".git" | "node_modules" | ".kali-cache" | "target"
            ) || name.starts_with('.')
            {
                continue;
            }
            if path != root && path.join("kali.json").exists() {
                continue;
            }
            collect_install_source_files(root, &path, files)?;
        } else if is_install_source_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn is_install_source_file(path: &Path) -> bool {
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

fn collect_source_module_specifiers(source: &str) -> BTreeSet<String> {
    let mut specifiers = BTreeSet::new();

    for quote in ['"', '\'', '`'] {
        let mut offset = 0usize;
        while let Some(start_rel) = source[offset..].find(quote) {
            let start = offset + start_rel;
            let after = &source[start + quote.len_utf8()..];
            let Some(end_rel) = after.find(quote) else {
                break;
            };
            let candidate = &after[..end_rel];
            let mut context_start = start.saturating_sub(160);
            while context_start > 0 && !source.is_char_boundary(context_start) {
                context_start -= 1;
            }
            let context = &source[context_start..start];
            if !candidate.trim().is_empty()
                && !candidate.chars().any(|ch| ch.is_whitespace())
                && (context.contains("import")
                    || context.contains("export")
                    || context.contains("from")
                    || context.contains("require("))
            {
                specifiers.insert(candidate.to_string());
            }
            offset = start + quote.len_utf8() + end_rel + quote.len_utf8();
            if offset >= source.len() {
                break;
            }
        }
    }

    specifiers
}

fn resolve_import_map_specifier(
    specifier: &str,
    imports: &BTreeMap<String, String>,
) -> Option<String> {
    let mut best: Option<(&str, &str)> = None;

    for (key, target) in imports {
        let matched = if key.ends_with('/') {
            specifier.starts_with(key)
        } else {
            specifier == key
        };

        if matched && best.is_none_or(|(best_key, _)| key.len() > best_key.len()) {
            best = Some((key.as_str(), target.as_str()));
        }
    }

    let (key, target) = best?;
    if key.ends_with('/') {
        if target.ends_with('/') {
            Some(format!("{}{}", target, &specifier[key.len()..]))
        } else {
            None
        }
    } else {
        Some(target.to_string())
    }
}

fn is_raw_url(specifier: &str) -> bool {
    specifier.starts_with("https://") || specifier.starts_with("http://")
}

fn registry_package_has_install_time_lifecycle_hooks(
    registry: &str,
    display_name: &str,
    requested_version: Option<&str>,
) -> Result<bool, Diagnostic> {
    let mut visited = BTreeSet::new();
    registry_package_has_install_time_lifecycle_hooks_inner(
        registry,
        display_name,
        requested_version,
        &mut visited,
    )
}

fn registry_package_has_install_time_lifecycle_hooks_inner(
    registry: &str,
    display_name: &str,
    requested_version: Option<&str>,
    visited: &mut BTreeSet<String>,
) -> Result<bool, Diagnostic> {
    let metadata_url = match registry {
        "npm" => npm_registry_metadata_url(display_name),
        "jsr" => jsr_registry_metadata_url(display_name),
        other => {
            return Err(Diagnostic::error(
                e6::INVALID_PACKAGE_SPECIFIER as u32,
                format!(
                    "unsupported registry '{}' for package '{}'",
                    other, display_name
                ),
            ));
        }
    };

    let metadata = fetch_registry_metadata(registry, display_name, &metadata_url)?;
    let versions = metadata
        .get("versions")
        .and_then(|value| value.as_object())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::NOT_FOUND as u32,
                format!(
                    "{} metadata for '{}' does not contain versions",
                    registry, display_name
                ),
            )
        })?;

    let version = select_registry_version(display_name, versions, requested_version)?;
    let visit_key = format!("{registry}:{display_name}@{version}");
    if !visited.insert(visit_key) {
        return Ok(false);
    }

    let version_meta = versions
        .get(&version)
        .and_then(|value| value.as_object())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::NOT_FOUND as u32,
                format!("missing metadata for '{}'@{}", display_name, version),
            )
        })?;

    if package_version_has_install_time_lifecycle_hooks(version_meta) {
        return Ok(true);
    }

    let dependency_specs = version_meta
        .get("dependencies")
        .and_then(|value| value.as_object())
        .into_iter()
        .chain(
            version_meta
                .get("optionalDependencies")
                .and_then(|value| value.as_object()),
        )
        .flat_map(|map| map.iter())
        .filter_map(|(name, value)| value.as_str().map(|spec| (name.clone(), spec.to_string())))
        .collect::<Vec<_>>();

    for (dep_name, dep_spec) in dependency_specs {
        let dep_registry = if dep_name.starts_with("jsr:") {
            "jsr"
        } else {
            "npm"
        };
        if registry_package_has_install_time_lifecycle_hooks_inner(
            dep_registry,
            &dep_name,
            Some(dep_spec.as_str()),
            visited,
        )? {
            return Ok(true);
        }
    }

    Ok(false)
}

fn package_version_has_install_time_lifecycle_hooks(
    version_meta: &serde_json::Map<String, serde_json::Value>,
) -> bool {
    version_meta
        .get("scripts")
        .and_then(|value| value.as_object())
        .map(|scripts| {
            ["preinstall", "install", "postinstall"]
                .iter()
                .any(|phase| {
                    scripts
                        .get(*phase)
                        .and_then(|value| value.as_str())
                        .map(|script| !script.trim().is_empty())
                        .unwrap_or(false)
                })
        })
        .unwrap_or(false)
}

fn has_effective_npm_scriptable_install_work(
    manifest: &ProjectManifest,
    target: Option<&PackageTarget>,
) -> Result<bool, Diagnostic> {
    match target {
        Some(PackageTarget::Registry {
            registry,
            name,
            version,
        }) if registry == "npm" => {
            registry_package_has_install_time_lifecycle_hooks(registry, name, version.as_deref())
        }
        Some(PackageTarget::Registry { registry, .. }) if registry == "jsr" => Ok(false),
        Some(PackageTarget::Registry { .. }) => Ok(false),
        Some(PackageTarget::RawUrl(_)) => Ok(false),
        None => {
            for (name, version) in manifest
                .dependencies
                .iter()
                .chain(manifest.dev_dependencies.iter())
            {
                if name.starts_with("jsr:") {
                    continue;
                }
                if registry_package_has_install_time_lifecycle_hooks(
                    "npm",
                    name,
                    Some(version.as_str()),
                )? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }
}

fn discover_install_time_raw_urls(
    root: &Path,
    manifest: &ProjectManifest,
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut urls = BTreeSet::new();

    for (key, target) in &manifest.imports {
        if !key.ends_with('/') && is_raw_url(target) {
            urls.insert(target.clone());
        }
    }

    for source_file in discover_install_source_files(root)? {
        let source = fs::read_to_string(&source_file).map_err(|error| {
            Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to read source file '{}': {}",
                    source_file.display(),
                    error
                ),
            )
        })?;

        for specifier in collect_source_module_specifiers(&source) {
            let resolved =
                resolve_import_map_specifier(&specifier, &manifest.imports).unwrap_or(specifier);
            if is_raw_url(&resolved) {
                urls.insert(resolved);
            }
        }
    }

    Ok(urls)
}

fn prune_unreachable_raw_urls(
    root: &Path,
    lock: &mut LockFile,
    reachable: &BTreeSet<String>,
) -> Result<(), Vec<Diagnostic>> {
    let unreachable = lock
        .raw_urls
        .keys()
        .filter(|url| !reachable.contains(*url))
        .cloned()
        .collect::<Vec<_>>();

    for url in unreachable {
        if let Some(entry) = lock.raw_urls.remove(&url) {
            remove_cached_raw_url_entry(root, &entry.cached)?;
        }
    }

    Ok(())
}

fn remove_cached_raw_url_entry(root: &Path, cached: &str) -> Result<(), Vec<Diagnostic>> {
    let cached_path = Path::new(cached);
    if cached_path.exists() {
        if cached_path.is_dir() {
            fs::remove_dir_all(cached_path).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to remove raw URL cache '{}': {}",
                        cached_path.display(),
                        error
                    ),
                )]
            })?;
        } else {
            fs::remove_file(cached_path).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to remove raw URL cache '{}': {}",
                        cached_path.display(),
                        error
                    ),
                )]
            })?;
        }
    }

    let raw_root = root.join(".kali-cache").join("raw");
    let mut current = cached_path.parent();
    while let Some(dir) = current {
        if dir == raw_root || !dir.starts_with(&raw_root) {
            break;
        }

        let is_empty = fs::read_dir(dir)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true);
        if !is_empty {
            break;
        }

        fs::remove_dir(dir).map_err(|error| {
            vec![Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to remove empty raw cache directory '{}': {}",
                    dir.display(),
                    error
                ),
            )]
        })?;
        current = dir.parent();
    }

    Ok(())
}

fn reconcile_raw_urls(
    root: &Path,
    lock: &mut LockFile,
    declared_raw_urls: &BTreeSet<String>,
    installed: &mut BTreeSet<String>,
) -> Result<(), Vec<Diagnostic>> {
    for url in declared_raw_urls {
        let needs_install = match lock.raw_urls.get(url) {
            Some(entry) => !Path::new(&entry.cached).exists(),
            None => true,
        };

        if needs_install {
            install_raw_url(root, lock, url, installed)?;
        } else {
            installed.insert(url.clone());
        }
    }

    prune_unreachable_raw_urls(root, lock, declared_raw_urls)
}

pub fn install_project(
    root: impl AsRef<Path>,
    options: InstallOptions,
) -> Result<InstallSummary, Vec<Diagnostic>> {
    let root = root.as_ref();
    fs::create_dir_all(root).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to create project root '{}': {}",
                root.display(),
                error
            ),
        )]
    })?;

    let mut manifest = match load_manifest(root) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => ProjectManifest::minimal(),
        Err(diagnostic) => return Err(vec![diagnostic]),
    };

    let parsed_target = match options.target.as_deref() {
        Some(target) => Some(parse_package_target(target).map_err(|diagnostic| vec![diagnostic])?),
        None => None,
    };

    if options.dev {
        match parsed_target.as_ref() {
            None => {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    "`--dev` requires an explicit registry package target",
                )]);
            }
            Some(PackageTarget::RawUrl(_)) => {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    "`--dev` is not valid for raw-URL targets",
                )]);
            }
            _ => {}
        }
    }

    if options.allow_scripts {
        match parsed_target.as_ref() {
            Some(PackageTarget::RawUrl(_)) => {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    "`--allow-scripts` is not valid for raw-URL targets",
                )]);
            }
            Some(PackageTarget::Registry { registry, .. }) if registry == "jsr" => {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    "`--allow-scripts` is not valid for JSR targets",
                )]);
            }
            _ => {}
        }

        if !has_effective_npm_scriptable_install_work(&manifest, parsed_target.as_ref())
            .map_err(|diagnostic| vec![diagnostic])?
        {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                "`--allow-scripts` requires effective npm-scriptable install work",
            )]);
        }
    }

    let mut lock = match load_lock(root) {
        Ok(Some(lock)) => lock,
        Ok(None) => LockFile::minimal(),
        Err(diagnostic) => return Err(vec![diagnostic]),
    };

    let mut installed = BTreeSet::new();
    let mut installed_paths = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let mut explicit_raw_url: Option<String> = None;
    let mut resolved_root_keys = BTreeSet::new();
    let host_fit_context = package_host_fit_context_for_manifest(&manifest);

    if let Some(target) = parsed_target {
        match target {
            PackageTarget::Registry {
                registry,
                name,
                version,
            } => {
                let resolved = match resolve_registry_package(&registry, &name, version.as_deref())
                {
                    Ok(resolved) => resolved,
                    Err(diagnostic) => return Err(vec![diagnostic]),
                };

                let dependency_version = resolved.version.clone();
                if options.dev {
                    manifest
                        .dev_dependencies
                        .insert(resolved.name.clone(), dependency_version.clone());
                } else {
                    manifest
                        .dependencies
                        .insert(resolved.name.clone(), dependency_version.clone());
                }

                validate_manifest_registry_collisions(&manifest)?;

                resolved_root_keys.insert(package_key(&resolved.name, &resolved.version));
                install_registry_package(
                    root,
                    &mut lock,
                    &resolved,
                    options.allow_scripts,
                    options.suppress_script_output,
                    host_fit_context,
                    &mut installed,
                    &mut installed_paths,
                    &mut diagnostics,
                )?;
            }
            PackageTarget::RawUrl(url) => {
                validate_manifest_registry_collisions(&manifest)?;
                explicit_raw_url = Some(url);
            }
        }
    } else if !manifest.dependencies.is_empty() || !manifest.dev_dependencies.is_empty() {
        validate_manifest_registry_collisions(&manifest)?;
        for (name, version) in manifest
            .dependencies
            .iter()
            .chain(manifest.dev_dependencies.iter())
        {
            let registry = if name.starts_with("jsr:") {
                "jsr"
            } else {
                "npm"
            };
            let resolved = resolve_registry_package(registry, name, Some(version.as_str()))
                .map_err(|diagnostic| vec![diagnostic])?;
            resolved_root_keys.insert(package_key(&resolved.name, &resolved.version));
            install_registry_package(
                root,
                &mut lock,
                &resolved,
                options.allow_scripts,
                options.suppress_script_output,
                host_fit_context,
                &mut installed,
                &mut installed_paths,
                &mut diagnostics,
            )?;
        }
    }

    let mut declared_raw_urls =
        discover_install_time_raw_urls(root, &manifest).map_err(|diagnostic| vec![diagnostic])?;
    if let Some(url) = explicit_raw_url {
        declared_raw_urls.insert(url);
    }
    reconcile_raw_urls(root, &mut lock, &declared_raw_urls, &mut installed)?;

    let root_keys = if resolved_root_keys.is_empty() {
        manifest_registry_package_keys(&manifest)
    } else {
        resolved_root_keys.into_iter().collect::<Vec<_>>()
    };
    let reachable = collect_reachable_registry_packages(&lock, &root_keys)
        .map_err(|diagnostic| vec![diagnostic])?;
    prune_unreachable_registry_packages(root, &mut lock, &reachable)?;

    let manifest_path = if manifest.is_minimal()
        && manifest.dependencies.is_empty()
        && manifest.dev_dependencies.is_empty()
    {
        None
    } else {
        Some(save_manifest(root, &manifest).map_err(|diagnostic| vec![diagnostic])?)
    };

    let lock_path = if lock.packages.is_empty() && lock.raw_urls.is_empty() {
        let path = root.join("kali.lock");
        if path.exists() {
            fs::remove_file(&path).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to remove stale lock file '{}': {}",
                        path.display(),
                        error
                    ),
                )]
            })?;
        }
        None
    } else {
        Some(save_lock(root, &lock).map_err(|diagnostic| vec![diagnostic])?)
    };

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(InstallSummary {
        manifest_path,
        lock_path,
        installed: installed.into_iter().collect(),
    })
}

fn record_install_path(
    installed_paths: &mut BTreeMap<String, String>,
    install_name: &str,
    key: &str,
) -> Result<(), Vec<Diagnostic>> {
    match installed_paths.entry(install_name.to_string()) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(key.to_string());
            Ok(())
        }
        std::collections::btree_map::Entry::Occupied(entry) if entry.get() == key => Ok(()),
        std::collections::btree_map::Entry::Occupied(entry) => Err(vec![Diagnostic::error(
            e6::VERSION_MISMATCH as u32,
            format!(
                "packages '{}' and '{}' would both materialize to node_modules/{}",
                entry.get(),
                key,
                entry.key()
            ),
        )]),
    }
}

#[allow(clippy::too_many_arguments)]
fn install_registry_package(
    root: &Path,
    lock: &mut LockFile,
    resolved: &ResolvedRegistryPackage,
    allow_scripts: bool,
    suppress_script_output: bool,
    host_fit_context: PackageHostFitContext,
    installed: &mut BTreeSet<String>,
    installed_paths: &mut BTreeMap<String, String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), Vec<Diagnostic>> {
    let _ = diagnostics;
    let key = package_key(&resolved.name, &resolved.version);
    let package_dir = root.join(".kali-cache").join("packages").join(&key);
    let node_modules_dir = root.join("node_modules");
    let install_path = node_modules_dir.join(&resolved.install_name);

    record_install_path(installed_paths, &resolved.install_name, &key)?;

    let locked = lock.packages.contains_key(&key);
    let verification_integrity = lock
        .packages
        .get(&key)
        .map(|package| package.integrity.clone())
        .or_else(|| resolved.integrity.clone());
    let package_dir_ready = package_dir.exists();
    let install_ready = install_path.exists();
    if locked && package_dir_ready && install_ready {
        let extracted_root = if package_dir.join("package").is_dir() {
            package_dir.join("package")
        } else {
            package_dir.clone()
        };
        let package_json = read_package_json(&extracted_root)?;
        validate_package_shape(&package_json, allow_scripts)?;
        validate_package_host_fit(&extracted_root, host_fit_context)
            .map_err(|diagnostic| vec![diagnostic])?;

        installed.insert(key.clone());
        if let Some(package) = lock.packages.get(&key) {
            let dependencies = package.dependencies.clone();
            for (dep_name, dep_spec) in dependencies {
                if installed.contains(&package_key(&dep_name, &dep_spec)) {
                    continue;
                }
                let dep_registry = if dep_name.starts_with("jsr:") {
                    "jsr"
                } else {
                    "npm"
                };
                let dep_resolved =
                    resolve_registry_package(dep_registry, &dep_name, Some(dep_spec.as_str()))
                        .map_err(|diagnostic| vec![diagnostic])?;
                install_registry_package(
                    root,
                    lock,
                    &dep_resolved,
                    allow_scripts,
                    suppress_script_output,
                    host_fit_context,
                    installed,
                    installed_paths,
                    diagnostics,
                )?;
            }
        }
        return Ok(());
    }

    if !package_dir_ready {
        fs::create_dir_all(&package_dir).map_err(|error| {
            vec![Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to create package cache '{}': {}",
                    package_dir.display(),
                    error
                ),
            )]
        })?;

        let tarball_bytes =
            download_bytes(&resolved.resolved).map_err(|diagnostic| vec![diagnostic])?;
        let integrity =
            verify_tarball_integrity(&tarball_bytes, verification_integrity.as_deref())?;
        extract_tarball(&tarball_bytes, &package_dir)?;

        let extracted_root = if package_dir.join("package").is_dir() {
            package_dir.join("package")
        } else {
            package_dir.clone()
        };

        let package_json = read_package_json(&extracted_root)?;
        validate_package_shape(&package_json, allow_scripts)?;
        validate_package_host_fit(&extracted_root, host_fit_context).map_err(|diagnostic| {
            let _ = fs::remove_dir_all(&package_dir);
            vec![diagnostic]
        })?;

        if let Some(parent) = install_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to create node_modules path '{}': {}",
                        parent.display(),
                        error
                    ),
                )]
            })?;
        }
        copy_tree(&extracted_root, &install_path)?;

        let dependency_specs = package_json
            .dependencies
            .iter()
            .chain(package_json.optional_dependencies.iter())
            .map(|(name, version)| (name.clone(), version.clone()))
            .collect::<Vec<_>>();
        let mut resolved_dependencies = BTreeMap::new();

        for (dep_name, dep_spec) in dependency_specs {
            if installed.contains(&package_key(&dep_name, &dep_spec)) {
                continue;
            }
            let dep_registry = if dep_name.starts_with("jsr:") {
                "jsr"
            } else {
                "npm"
            };
            let dep_resolved =
                resolve_registry_package(dep_registry, &dep_name, Some(dep_spec.as_str()))
                    .map_err(|diagnostic| vec![diagnostic])?;
            resolved_dependencies.insert(dep_name.clone(), dep_resolved.version.clone());
            install_registry_package(
                root,
                lock,
                &dep_resolved,
                allow_scripts,
                suppress_script_output,
                host_fit_context,
                installed,
                installed_paths,
                diagnostics,
            )?;
        }

        lock.packages.insert(
            key.clone(),
            LockedPackage {
                registry: resolved.registry.clone(),
                integrity,
                resolved: resolved.resolved.clone(),
                dependencies: resolved_dependencies.clone(),
            },
        );

        run_package_lifecycle_hooks(
            &install_path,
            &package_json,
            allow_scripts,
            suppress_script_output,
        )?;

        installed.insert(key.clone());

        return Ok(());
    }

    let extracted_root = if package_dir.join("package").is_dir() {
        package_dir.join("package")
    } else {
        package_dir.clone()
    };

    let package_json = read_package_json(&extracted_root)?;
    validate_package_shape(&package_json, allow_scripts)?;
    validate_package_host_fit(&extracted_root, host_fit_context).map_err(|diagnostic| {
        let _ = fs::remove_dir_all(&package_dir);
        vec![diagnostic]
    })?;

    if let Some(parent) = install_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to create node_modules path '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }
    copy_tree(&extracted_root, &install_path)?;

    let dependency_specs = package_json
        .dependencies
        .iter()
        .chain(package_json.optional_dependencies.iter())
        .map(|(name, version)| (name.clone(), version.clone()))
        .collect::<Vec<_>>();
    let mut resolved_dependencies = BTreeMap::new();
    let integrity = if !locked {
        let tarball_bytes =
            download_bytes(&resolved.resolved).map_err(|diagnostic| vec![diagnostic])?;
        Some(verify_tarball_integrity(
            &tarball_bytes,
            verification_integrity.as_deref(),
        )?)
    } else {
        None
    };

    for (dep_name, dep_spec) in dependency_specs {
        if installed.contains(&package_key(&dep_name, &dep_spec)) {
            continue;
        }
        let dep_registry = if dep_name.starts_with("jsr:") {
            "jsr"
        } else {
            "npm"
        };
        let dep_resolved =
            resolve_registry_package(dep_registry, &dep_name, Some(dep_spec.as_str()))
                .map_err(|diagnostic| vec![diagnostic])?;
        resolved_dependencies.insert(dep_name.clone(), dep_resolved.version.clone());
        install_registry_package(
            root,
            lock,
            &dep_resolved,
            allow_scripts,
            suppress_script_output,
            host_fit_context,
            installed,
            installed_paths,
            diagnostics,
        )?;
    }

    if let Some(integrity) = integrity {
        lock.packages.insert(
            key.clone(),
            LockedPackage {
                registry: resolved.registry.clone(),
                integrity,
                resolved: resolved.resolved.clone(),
                dependencies: resolved_dependencies.clone(),
            },
        );
    }

    run_package_lifecycle_hooks(
        &install_path,
        &package_json,
        allow_scripts,
        suppress_script_output,
    )?;

    installed.insert(key.clone());

    Ok(())
}

fn install_raw_url(
    root: &Path,
    lock: &mut LockFile,
    url: &str,
    installed: &mut BTreeSet<String>,
) -> Result<(), Vec<Diagnostic>> {
    let bytes = download_bytes(url).map_err(|diagnostic| vec![diagnostic])?;
    let hash = sha256_hex(&bytes);
    let cache_dir = root
        .join(".kali-cache")
        .join("raw")
        .join(format!("sha256-{}", hash));
    fs::create_dir_all(&cache_dir).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to create raw cache '{}': {}",
                cache_dir.display(),
                error
            ),
        )]
    })?;

    let file_name = raw_url_file_name(url).unwrap_or_else(|| "index.ts".to_string());
    let cached = cache_dir.join(&file_name);
    fs::write(&cached, bytes).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to write raw cache '{}': {}",
                cached.display(),
                error
            ),
        )]
    })?;

    lock.raw_urls.insert(
        url.to_string(),
        RawUrlEntry {
            integrity: format!("sha256-{}", hash),
            cached: cached.to_string_lossy().to_string(),
        },
    );
    installed.insert(url.to_string());
    Ok(())
}

#[derive(Debug, Clone)]
struct ResolvedRegistryPackage {
    registry: String,
    name: String,
    install_name: String,
    version: String,
    resolved: String,
    integrity: Option<String>,
}

fn resolve_registry_package(
    registry: &str,
    name: &str,
    requested_version: Option<&str>,
) -> Result<ResolvedRegistryPackage, Diagnostic> {
    match registry {
        "npm" => resolve_npm_package(name, requested_version),
        "jsr" => resolve_jsr_package(name, requested_version),
        other => Err(Diagnostic::error(
            e6::INVALID_PACKAGE_SPECIFIER as u32,
            format!("unsupported registry '{}' for package '{}'", other, name),
        )),
    }
}

fn resolve_npm_package(
    name: &str,
    requested_version: Option<&str>,
) -> Result<ResolvedRegistryPackage, Diagnostic> {
    let metadata_url = npm_registry_metadata_url(name);
    resolve_npm_like_package("npm", name, name, &metadata_url, requested_version)
}

fn npm_registry_base_url() -> String {
    std::env::var("KALI_REGISTRY").unwrap_or_else(|_| DEFAULT_NPM_REGISTRY.to_string())
}

fn npm_registry_metadata_url(name: &str) -> String {
    format!("{}/{}", npm_registry_base_url(), encode_package_name(name))
}

fn jsr_registry_metadata_url(name: &str) -> String {
    let raw_name = name.trim_start_matches("jsr:");
    let compat_name = jsr_compat_name(raw_name);
    format!("https://npm.jsr.io/{}", encode_package_name(&compat_name))
}

fn fetch_registry_metadata(
    registry: &str,
    display_name: &str,
    metadata_url: &str,
) -> Result<serde_json::Value, Diagnostic> {
    if let Some(cached) = REGISTRY_METADATA_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .ok()
        .and_then(|cache| cache.get(metadata_url).cloned())
    {
        return Ok(cached);
    }

    let client = Client::builder()
        .user_agent("kali/0.1.0")
        .build()
        .map_err(|error| Diagnostic::error(e6::INSTALL_FAILED as u32, error.to_string()))?;

    let response = client.get(metadata_url).send().map_err(|error| {
        Diagnostic::error(
            e6::NOT_FOUND as u32,
            format!(
                "failed to fetch {} metadata for '{}': {}",
                registry, display_name, error
            ),
        )
    })?;

    if !response.status().is_success() {
        return Err(Diagnostic::error(
            e6::NOT_FOUND as u32,
            format!(
                "package '{}' not found in {} registry (status {})",
                display_name,
                registry,
                response.status()
            ),
        ));
    }

    let metadata_text = response.text().map_err(|error| {
        Diagnostic::error(
            e6::INVALID_PACKAGE_SPECIFIER as u32,
            format!(
                "invalid {} metadata for '{}': {}",
                registry, display_name, error
            ),
        )
    })?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata_text).map_err(|error| {
        Diagnostic::error(
            e6::INVALID_PACKAGE_SPECIFIER as u32,
            format!(
                "invalid {} metadata for '{}': {}",
                registry, display_name, error
            ),
        )
    })?;

    if let Ok(mut cache) = REGISTRY_METADATA_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    {
        cache.insert(metadata_url.to_string(), metadata.clone());
    }

    Ok(metadata)
}

fn resolve_jsr_package(
    name: &str,
    requested_version: Option<&str>,
) -> Result<ResolvedRegistryPackage, Diagnostic> {
    let raw_name = name.trim_start_matches("jsr:");
    let metadata_url = jsr_registry_metadata_url(name);
    resolve_npm_like_package("jsr", name, raw_name, &metadata_url, requested_version)
}

fn resolve_npm_like_package(
    registry: &str,
    display_name: &str,
    install_name_source: &str,
    metadata_url: &str,
    requested_version: Option<&str>,
) -> Result<ResolvedRegistryPackage, Diagnostic> {
    let metadata = fetch_registry_metadata(registry, display_name, metadata_url)?;

    let versions = metadata
        .get("versions")
        .and_then(|value| value.as_object())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::NOT_FOUND as u32,
                format!(
                    "{} metadata for '{}' does not contain versions",
                    registry, display_name
                ),
            )
        })?;

    let version = select_registry_version(display_name, versions, requested_version)?;

    let version_meta = versions
        .get(&version)
        .and_then(|value| value.as_object())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::NOT_FOUND as u32,
                format!("missing metadata for '{}'@{}", display_name, version),
            )
        })?;

    let dist = version_meta
        .get("dist")
        .and_then(|value| value.as_object())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::NOT_FOUND as u32,
                format!("missing dist metadata for '{}'@{}", display_name, version),
            )
        })?;

    let tarball = dist
        .get("tarball")
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::NOT_FOUND as u32,
                format!("missing tarball URL for '{}'@{}", display_name, version),
            )
        })?;
    let integrity = dist
        .get("integrity")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    Ok(ResolvedRegistryPackage {
        registry: registry.to_string(),
        name: display_name.to_string(),
        install_name: install_name_from_package(install_name_source),
        version,
        resolved: tarball.to_string(),
        integrity,
    })
}

fn audit_package_version_metadata(
    registry: &str,
    package_name: &str,
    version: &str,
    version_meta: &serde_json::Map<String, serde_json::Value>,
) -> Vec<Diagnostic> {
    let mut findings = Vec::new();

    let mut lifecycle_phases = Vec::new();
    if let Some(scripts) = version_meta
        .get("scripts")
        .and_then(|value| value.as_object())
    {
        for phase in ["preinstall", "install", "postinstall"] {
            if let Some(script) = scripts.get(phase).and_then(|value| value.as_str()) {
                if !script.trim().is_empty() {
                    lifecycle_phases.push(phase);
                    if script.contains("node-gyp") {
                        findings.push(Diagnostic::error(
                            e6::INCOMPATIBLE_PACKAGE as u32,
                            format!(
                                "package '{package_name}'@{version} in {registry} uses a node-gyp lifecycle script and falls outside the pure JS/TS package contract",
                            ),
                        ));
                    }
                }
            }
        }
    }

    if !lifecycle_phases.is_empty() {
        findings.push(
            Diagnostic::warning(
                e6::LIFECYCLE_SCRIPT_REJECTED as u32,
                format!(
                    "package '{package_name}'@{version} in {registry} declares lifecycle scripts in {}",
                    lifecycle_phases.join(", ")
                ),
            )
            .note("package-audit treats install-time scripts as a security finding"),
        );
    }

    if version_meta
        .get("gypfile")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        findings.push(Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            format!(
                "package '{package_name}'@{version} in {registry} declares gypfile=true and falls outside the pure JS/TS package contract",
            ),
        ));
    }

    if let Some(path) = version_meta
        .get("main")
        .and_then(|value| value.as_str())
        .or_else(|| version_meta.get("module").and_then(|value| value.as_str()))
        .filter(|path| path.ends_with(".node"))
    {
        findings.push(Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            format!(
                "package '{package_name}'@{version} in {registry} publishes a native addon entrypoint '{path}' and falls outside the pure JS/TS package contract",
            ),
        ));
    }

    if let Some(path) = version_meta.get("bin").and_then(|value| match value {
        serde_json::Value::String(path) if path.ends_with(".node") => Some(path.as_str()),
        serde_json::Value::Object(map) => map
            .values()
            .find_map(|value| value.as_str().filter(|path| path.ends_with(".node"))),
        _ => None,
    }) {
        findings.push(Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            format!(
                "package '{package_name}'@{version} in {registry} publishes a native addon bin entrypoint '{path}' and falls outside the pure JS/TS package contract",
            ),
        ));
    }

    findings
}

pub fn audit_registry_package(
    registry: &str,
    name: &str,
) -> Result<RegistryPackageAudit, Diagnostic> {
    let resolved = resolve_registry_package(registry, name, None)?;
    let ResolvedRegistryPackage {
        registry: resolved_registry,
        name: resolved_name,
        version: resolved_version,
        ..
    } = resolved;
    let metadata_url = match registry {
        "npm" => npm_registry_metadata_url(name),
        "jsr" => jsr_registry_metadata_url(name),
        other => {
            return Err(Diagnostic::error(
                e6::INVALID_PACKAGE_SPECIFIER as u32,
                format!("unsupported registry '{}' for package '{}'", other, name),
            ))
        }
    };
    let metadata = fetch_registry_metadata(registry, name, &metadata_url)?;
    let versions = metadata
        .get("versions")
        .and_then(|value| value.as_object())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::NOT_FOUND as u32,
                format!(
                    "{} metadata for '{}' does not contain versions",
                    registry, name
                ),
            )
        })?;
    let version_meta = versions
        .get(&resolved_version)
        .and_then(|value| value.as_object())
        .ok_or_else(|| {
            Diagnostic::error(
                e6::NOT_FOUND as u32,
                format!("missing metadata for '{}'@{}", name, resolved_version),
            )
        })?;

    Ok(RegistryPackageAudit {
        registry: resolved_registry,
        name: resolved_name,
        version: resolved_version.clone(),
        findings: audit_package_version_metadata(registry, name, &resolved_version, version_meta),
    })
}

fn select_registry_version(
    display_name: &str,
    versions: &serde_json::Map<String, serde_json::Value>,
    requested_version: Option<&str>,
) -> Result<String, Diagnostic> {
    if let Some(requested) = requested_version {
        if versions.contains_key(requested) {
            return Ok(requested.to_string());
        }

        if Version::parse(requested).is_ok() {
            return Err(Diagnostic::error(
                e6::VERSION_MISMATCH as u32,
                format!(
                    "package '{}' does not publish version '{}'",
                    display_name, requested
                ),
            ));
        }

        let req = VersionReq::parse(requested).map_err(|_| {
            Diagnostic::error(
                e6::INVALID_PACKAGE_SPECIFIER as u32,
                format!(
                    "package '{}' has invalid version specifier '{}'",
                    display_name, requested
                ),
            )
        })?;

        let mut candidate: Option<Version> = None;
        for key in versions.keys() {
            if let Ok(version) = Version::parse(key) {
                if req.matches(&version)
                    && candidate
                        .as_ref()
                        .map(|current| &version > current)
                        .unwrap_or(true)
                {
                    candidate = Some(version);
                }
            }
        }

        let Some(latest) = candidate else {
            return Err(Diagnostic::error(
                e6::VERSION_MISMATCH as u32,
                format!(
                    "package '{}' does not publish a version matching '{}'",
                    display_name, requested
                ),
            ));
        };

        return Ok(latest.to_string());
    }

    let mut candidate: Option<Version> = None;
    for key in versions.keys() {
        if let Ok(version) = Version::parse(key) {
            if version.pre.is_empty()
                && candidate
                    .as_ref()
                    .map(|current| &version > current)
                    .unwrap_or(true)
            {
                candidate = Some(version);
            }
        }
    }
    let Some(latest) = candidate else {
        return Err(Diagnostic::error(
            e6::NOT_FOUND as u32,
            format!("package '{}' has no stable published version", display_name),
        ));
    };
    Ok(latest.to_string())
}

fn parse_package_target(target: &str) -> Result<PackageTarget, Diagnostic> {
    if target.starts_with("http://") || target.starts_with("https://") {
        return Ok(PackageTarget::RawUrl(target.to_string()));
    }

    if target.starts_with("jsr:") {
        let spec = target.trim_start_matches("jsr:");
        let (name, version) = split_package_name_and_version(spec)?;
        return Ok(PackageTarget::Registry {
            registry: "jsr".to_string(),
            name: format!("jsr:{}", name),
            version,
        });
    }

    let (name, version) = split_package_name_and_version(target)?;
    Ok(PackageTarget::Registry {
        registry: "npm".to_string(),
        name,
        version,
    })
}

fn split_package_name_and_version(spec: &str) -> Result<(String, Option<String>), Diagnostic> {
    if spec.is_empty() {
        return Err(Diagnostic::error(
            e6::INVALID_PACKAGE_SPECIFIER as u32,
            "empty package specifier is invalid",
        ));
    }

    if spec.starts_with('@') {
        let mut parts = spec.rsplitn(2, '@');
        let version = parts.next().unwrap_or_default();
        let name = parts.next().unwrap_or_default();
        if name.is_empty() || version.is_empty() {
            return Ok((spec.to_string(), None));
        }
        if version
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            return Ok((name.to_string(), Some(version.to_string())));
        }
        return Ok((spec.to_string(), None));
    }

    if let Some((name, version)) = spec.rsplit_once('@') {
        if !version.is_empty()
            && version
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            return Ok((name.to_string(), Some(version.to_string())));
        }
    }

    Ok((spec.to_string(), None))
}

fn encode_package_name(name: &str) -> String {
    urlencoding::encode(name).into_owned()
}

fn package_key(name: &str, version: &str) -> String {
    format!("{}@{}", name, version)
}

fn install_name_from_package(name: &str) -> String {
    name.trim_start_matches("jsr:").to_string()
}

fn validate_manifest_registry_collisions(
    manifest: &ProjectManifest,
) -> Result<(), Vec<Diagnostic>> {
    let mut occupied_paths = BTreeMap::new();

    for name in manifest
        .dependencies
        .keys()
        .chain(manifest.dev_dependencies.keys())
    {
        let install_path = install_name_from_package(name);
        match occupied_paths.entry(install_path) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(name.clone());
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                let previous = entry.get();
                if previous != name {
                    return Err(vec![Diagnostic::error(
                        e6::VERSION_MISMATCH as u32,
                        format!(
                            "registry identities '{}' and '{}' would both materialize to node_modules/{}",
                            previous,
                            name,
                            entry.key()
                        ),
                    )]);
                }
            }
        }
    }

    Ok(())
}

fn jsr_compat_name(name: &str) -> String {
    let raw = name.trim_start_matches("jsr:");
    let raw = raw.strip_prefix('@').unwrap_or(raw);
    let mut parts = raw.splitn(2, '/');
    let scope = parts.next().unwrap_or(raw);
    let package = parts.next().unwrap_or("");
    if package.is_empty() {
        format!("@jsr/{}", scope.replace('/', "__"))
    } else {
        format!("@jsr/{}__{}", scope, package)
    }
}

fn read_package_json(package_dir: &Path) -> Result<PackageJson, Vec<Diagnostic>> {
    let path = package_dir.join("package.json");
    let contents = fs::read_to_string(&path).map_err(|error| {
        vec![Diagnostic::error(
            e6::INVALID_LOCK_FILE as u32,
            format!(
                "failed to read package.json '{}': {}",
                path.display(),
                error
            ),
        )]
    })?;
    let package_json: PackageJson = serde_json::from_str(&contents).map_err(|error| {
        vec![Diagnostic::error(
            e6::INVALID_LOCK_FILE as u32,
            format!(
                "failed to parse package.json '{}': {}",
                path.display(),
                error
            ),
        )]
    })?;
    Ok(package_json)
}

fn validate_package_shape(
    package_json: &PackageJson,
    allow_scripts: bool,
) -> Result<(), Vec<Diagnostic>> {
    let install_time_phases = [
        (
            "preinstall",
            package_json
                .scripts
                .get("preinstall")
                .map(|script| !script.trim().is_empty())
                .unwrap_or(false),
        ),
        (
            "install",
            package_json
                .scripts
                .get("install")
                .map(|script| !script.trim().is_empty())
                .unwrap_or(false),
        ),
        (
            "postinstall",
            package_json
                .scripts
                .get("postinstall")
                .map(|script| !script.trim().is_empty())
                .unwrap_or(false),
        ),
    ]
    .into_iter()
    .filter_map(|(phase, present)| present.then_some(phase))
    .collect::<Vec<_>>();

    if !allow_scripts && !install_time_phases.is_empty() {
        return Err(vec![Diagnostic::error(
            e6::LIFECYCLE_SCRIPT_REJECTED as u32,
            "npm install-time lifecycle scripts require `--allow-scripts`",
        )]);
    }

    for phase in install_time_phases {
        if package_json
            .scripts
            .get(phase)
            .is_some_and(|script| script.contains("node-gyp"))
        {
            return Err(vec![Diagnostic::error(
                e6::INCOMPATIBLE_PACKAGE as u32,
                "package uses a node-gyp lifecycle script and falls outside the pure JS/TS package contract",
            )]);
        }
    }

    if package_json
        .main
        .as_deref()
        .or(package_json.module.as_deref())
        .map(|path| path.ends_with(".node"))
        .unwrap_or(false)
    {
        return Err(vec![Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            "package publishes a native addon entrypoint and falls outside the pure JS/TS package contract",
        )]);
    }

    if package_json.bin.as_ref().is_some_and(|value| match value {
        serde_json::Value::String(path) => path.ends_with(".node"),
        serde_json::Value::Object(map) => map
            .values()
            .filter_map(|value| value.as_str())
            .any(|path| path.ends_with(".node")),
        _ => false,
    }) {
        return Err(vec![Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            "package bin entry points to a native addon and falls outside the pure JS/TS package contract",
        )]);
    }

    Ok(())
}

fn run_package_lifecycle_hooks(
    package_dir: &Path,
    package_json: &PackageJson,
    allow_scripts: bool,
    suppress_output: bool,
) -> Result<(), Vec<Diagnostic>> {
    if !allow_scripts || package_json.scripts.is_empty() {
        return Ok(());
    }

    for phase in ["preinstall", "install", "postinstall"] {
        let Some(script) = package_json.scripts.get(phase) else {
            continue;
        };
        if script.trim().is_empty() {
            continue;
        }

        run_package_lifecycle_hook(package_dir, phase, script, suppress_output)?;
    }

    Ok(())
}

fn run_package_lifecycle_hook(
    package_dir: &Path,
    phase: &str,
    script: &str,
    suppress_output: bool,
) -> Result<(), Vec<Diagnostic>> {
    let mut command = if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.arg("/C").arg(script);
        command
    } else {
        let mut command = Command::new("sh");
        command.arg("-c").arg(script);
        command
    };

    command.current_dir(package_dir);
    if suppress_output {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    let status = command.status().map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to run npm lifecycle script '{}' in '{}': {}",
                phase,
                package_dir.display(),
                error
            ),
        )]
    })?;

    if !status.success() {
        return Err(vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "npm lifecycle script '{}' in '{}' failed with status {}",
                phase,
                package_dir.display(),
                status
            ),
        )]);
    }

    Ok(())
}

fn validate_package_host_fit(
    package_dir: &Path,
    host_fit_context: PackageHostFitContext,
) -> Result<(), Diagnostic> {
    if host_fit_context.allows_node_only_host_apis() {
        return Ok(());
    }

    if let Some((path, builtin)) = scan_for_node_only_host_api(package_dir)? {
        return Err(Diagnostic::error(
            e6::NODE_ONLY_HOST_APIS as u32,
            format!(
                "package uses Node-only host API '{}' in '{}' and falls outside the default standalone context; use the Phase-3 Node compatibility target",
                builtin,
                path.display()
            ),
        ));
    }

    Ok(())
}

fn scan_for_node_only_host_api(root: &Path) -> Result<Option<(PathBuf, &'static str)>, Diagnostic> {
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|error| {
            Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!("failed to read '{}': {}", dir.display(), error),
            )
        })? {
            let entry = entry.map_err(|error| {
                Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!("failed to read entry in '{}': {}", dir.display(), error),
                )
            })?;
            let path = entry.path();
            if path.is_dir() {
                if !should_skip_package_scan_dir(&path) {
                    stack.push(path);
                }
                continue;
            }

            if !is_scannable_package_source(&path) {
                continue;
            }

            let contents = match fs::read_to_string(&path) {
                Ok(contents) => contents,
                Err(_) => continue,
            };

            if let Some(builtin) = source_mentions_node_only_host_api(&contents) {
                return Ok(Some((path, builtin)));
            }
        }
    }

    Ok(None)
}

fn is_scannable_package_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
    )
}

fn should_skip_package_scan_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("node_modules" | ".git" | ".kali-cache" | "target")
    )
}

fn source_mentions_node_only_host_api(contents: &str) -> Option<&'static str> {
    for builtin in NODE_ONLY_HOST_APIS {
        let patterns = [
            format!("node:{}", builtin),
            format!("require(\"{}\")", builtin),
            format!("require('{}')", builtin),
            format!("from \"{}\"", builtin),
            format!("from '{}'", builtin),
            format!("import(\"{}\")", builtin),
            format!("import('{}')", builtin),
        ];

        if patterns.iter().any(|pattern| contents.contains(pattern)) {
            return Some(*builtin);
        }
    }

    None
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PackageJson {
    pub name: Option<String>,
    pub version: Option<String>,
    pub main: Option<String>,
    pub module: Option<String>,
    pub exports: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub package_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<serde_json::Value>,
    pub types: Option<String>,
    pub typings: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub scripts: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(
        rename = "optionalDependencies",
        skip_serializing_if = "BTreeMap::is_empty",
        default
    )]
    pub optional_dependencies: BTreeMap<String, String>,
}

fn download_bytes(url: &str) -> Result<Vec<u8>, Diagnostic> {
    let client = Client::builder()
        .user_agent("kali/0.1.0")
        .build()
        .map_err(|error| Diagnostic::error(e6::INSTALL_FAILED as u32, error.to_string()))?;
    let response = client.get(url).send().map_err(|error| {
        Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!("failed to download '{}': {}", url, error),
        )
    })?;
    if !response.status().is_success() {
        return Err(Diagnostic::error(
            e6::NOT_FOUND as u32,
            format!(
                "download '{}' failed with status {}",
                url,
                response.status()
            ),
        ));
    }
    let bytes = response.bytes().map_err(|error| {
        Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!("failed to read '{}': {}", url, error),
        )
    })?;
    Ok(bytes.to_vec())
}

fn verify_tarball_integrity(
    bytes: &[u8],
    integrity: Option<&str>,
) -> Result<String, Vec<Diagnostic>> {
    let actual = format_sha512(bytes);
    if let Some(expected) = integrity {
        if !integrity_matches(expected, bytes) {
            return Err(vec![Diagnostic::error(
                e6::INTEGRITY_VERIFICATION_FAILED as u32,
                format!(
                    "tarball integrity mismatch: expected {}, got sha512-{}",
                    expected, actual
                ),
            )]);
        }
    }
    Ok(format!("sha512-{}", actual))
}

fn integrity_matches(expected: &str, bytes: &[u8]) -> bool {
    if let Some(encoded) = expected.strip_prefix("sha512-") {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
            return decoded == Sha512::digest(bytes).to_vec();
        }
    }
    false
}

fn format_sha512(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    base64::engine::general_purpose::STANDARD.encode(digest)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{:x}", digest)
}

fn extract_tarball(bytes: &[u8], package_dir: &Path) -> Result<(), Vec<Diagnostic>> {
    let mut archive = Archive::new(GzDecoder::new(io::Cursor::new(bytes)));
    archive.unpack(package_dir).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to extract tarball into '{}': {}",
                package_dir.display(),
                error
            ),
        )]
    })
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), Vec<Diagnostic>> {
    if target.exists() {
        fs::remove_dir_all(target).map_err(|error| {
            vec![Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to clean install directory '{}': {}",
                    target.display(),
                    error
                ),
            )]
        })?;
    }
    fs::create_dir_all(target.parent().unwrap_or_else(|| Path::new("."))).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to prepare install directory '{}': {}",
                target.display(),
                error
            ),
        )]
    })?;
    recursive_copy(source, target)
}

fn recursive_copy(source: &Path, target: &Path) -> Result<(), Vec<Diagnostic>> {
    fs::create_dir_all(target).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!("failed to create '{}': {}", target.display(), error),
        )]
    })?;

    for entry in fs::read_dir(source).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!("failed to read '{}': {}", source.display(), error),
        )]
    })? {
        let entry = entry.map_err(|error| {
            vec![Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!("failed to read entry in '{}': {}", source.display(), error),
            )]
        })?;
        let path = entry.path();
        let target_path = target.join(entry.file_name());
        if path.is_dir() {
            recursive_copy(&path, &target_path)?;
        } else {
            fs::copy(&path, &target_path).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to copy '{}' to '{}': {}",
                        path.display(),
                        target_path.display(),
                        error
                    ),
                )]
            })?;
        }
    }
    Ok(())
}

fn raw_url_file_name(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()?
                .next_back()
                .map(|segment| segment.to_string())
        })
        .filter(|name| !name.is_empty())
}

/// Resolve a bare import against the materialized package graph.
pub fn resolve_materialized_import(root: impl AsRef<Path>, source: &str) -> Option<PathBuf> {
    resolve_materialized_import_with_browser_context(root, source, false)
}

/// Resolve a bare import against the materialized package graph with an explicit browser context.
///
/// The explicit browser flag is merged with the current manifest context so callers can opt into
/// browser package resolution before a manifest is written, while still honoring an existing
/// browser-oriented project configuration.
pub fn resolve_materialized_import_with_browser_context(
    root: impl AsRef<Path>,
    source: &str,
    browser_context: bool,
) -> Option<PathBuf> {
    let root = root.as_ref();
    let manifest = load_manifest(root).ok().flatten();
    let manifest_browser_context = manifest
        .as_ref()
        .and_then(|manifest| manifest.compiler_options.as_ref())
        .and_then(|options| options.as_object())
        .and_then(|options| options.get("apiSurface"))
        .and_then(|value| value.as_str())
        == Some("browser");
    let browser_context = browser_context || manifest_browser_context;
    let (package_name, subpath) = split_bare_package_source(source)?;
    let package_dir = root.join("node_modules").join(&package_name);
    if package_dir.exists() {
        let package_json_path = package_dir.join("package.json");
        let package_json = fs::read_to_string(package_json_path)
            .ok()
            .and_then(|contents| serde_json::from_str::<PackageJson>(&contents).ok())?;

        if let Some(subpath) = subpath {
            match resolve_package_subpath(&package_dir, &package_json, subpath, browser_context) {
                Some(PackageResolutionOutcome::Resolved(path)) => return Some(path),
                Some(PackageResolutionOutcome::BrowserBlocked) => return None,
                None => {}
            }
        } else {
            match resolve_package_entry(&package_dir, &package_json, browser_context) {
                Some(PackageResolutionOutcome::Resolved(path)) => return Some(path),
                Some(PackageResolutionOutcome::BrowserBlocked) => return None,
                None => {}
            }
            if let Some(path) = resolve_package_types_entry(&package_dir, &package_json) {
                return Some(path);
            }
        }
    }

    resolve_types_package_import(root, &package_name, subpath, browser_context)
}

fn resolve_types_package_import(
    root: &Path,
    package_name: &str,
    subpath: Option<&str>,
    browser_context: bool,
) -> Option<PathBuf> {
    let manifest = load_manifest(root).ok().flatten()?;
    let browser_context = browser_context
        || manifest
            .compiler_options
            .as_ref()
            .and_then(|options| options.as_object())
            .and_then(|options| options.get("apiSurface"))
            .and_then(|value| value.as_str())
            == Some("browser");
    let types_package_name = types_package_name(package_name);
    if !manifest.dev_dependencies.contains_key(&types_package_name) {
        return None;
    }

    let types_dir = root.join("node_modules").join(&types_package_name);
    if !types_dir.exists() {
        return None;
    }

    let package_json_path = types_dir.join("package.json");
    let package_json = fs::read_to_string(package_json_path)
        .ok()
        .and_then(|contents| serde_json::from_str::<PackageJson>(&contents).ok())?;

    if let Some(subpath) = subpath {
        match resolve_package_subpath(&types_dir, &package_json, subpath, browser_context) {
            Some(PackageResolutionOutcome::Resolved(path)) => Some(path),
            Some(PackageResolutionOutcome::BrowserBlocked) => None,
            None => None,
        }
    } else {
        match resolve_package_entry(&types_dir, &package_json, browser_context) {
            Some(PackageResolutionOutcome::Resolved(path)) => Some(path),
            Some(PackageResolutionOutcome::BrowserBlocked) => None,
            None => resolve_package_types_entry(&types_dir, &package_json),
        }
    }
}

fn resolve_package_types_entry(package_dir: &Path, package_json: &PackageJson) -> Option<PathBuf> {
    if let Some(types) = &package_json.types {
        if let Some(path) = resolve_package_file(package_dir, types) {
            return Some(path);
        }
    }

    if let Some(typings) = &package_json.typings {
        if let Some(path) = resolve_package_file(package_dir, typings) {
            return Some(path);
        }
    }

    resolve_package_file(package_dir, "index.d.ts")
        .or_else(|| resolve_package_file(package_dir, "index.d.mts"))
        .or_else(|| resolve_package_file(package_dir, "index.d.cts"))
}

fn types_package_name(name: &str) -> String {
    if let Some(rest) = name.strip_prefix('@') {
        let mut parts = rest.splitn(2, '/');
        let scope = parts.next().unwrap_or(rest);
        let package = parts.next().unwrap_or("");
        if package.is_empty() {
            return format!("@types/{}", scope);
        }
        return format!("@types/{}__{}", scope, package);
    }

    format!("@types/{}", name)
}

fn split_bare_package_source(source: &str) -> Option<(String, Option<&str>)> {
    if source.starts_with('.') || source.starts_with('/') || source.contains("://") {
        return None;
    }

    if source.starts_with('@') {
        let mut parts = source.splitn(3, '/');
        let scope = parts.next()?;
        let name = parts.next()?;
        let remainder = parts.next();
        let package = format!("{}/{}", scope, name);
        return Some((package, remainder));
    }

    let mut parts = source.splitn(2, '/');
    let package = parts.next()?.to_string();
    let remainder = parts.next();
    Some((package, remainder))
}

enum PackageResolutionOutcome {
    Resolved(PathBuf),
    BrowserBlocked,
}

fn resolve_package_entry(
    package_dir: &Path,
    package_json: &PackageJson,
    browser_context: bool,
) -> Option<PackageResolutionOutcome> {
    if let Some(main) = &package_json.main {
        if let Some(path) = resolve_package_file(package_dir, main) {
            return Some(apply_browser_rewrite(
                package_dir,
                package_json,
                path,
                true,
                browser_context,
            ));
        }
    }

    if let Some(module) = &package_json.module {
        if let Some(path) = resolve_package_file(package_dir, module) {
            return Some(apply_browser_rewrite(
                package_dir,
                package_json,
                path,
                true,
                browser_context,
            ));
        }
    }

    resolve_package_file(package_dir, "index.js")
        .or_else(|| resolve_package_file(package_dir, "index.mjs"))
        .or_else(|| resolve_package_file(package_dir, "index.ts"))
        .map(|path| apply_browser_rewrite(package_dir, package_json, path, true, browser_context))
}

fn resolve_package_subpath(
    package_dir: &Path,
    package_json: &PackageJson,
    subpath: &str,
    browser_context: bool,
) -> Option<PackageResolutionOutcome> {
    let joined = package_dir.join(subpath);
    if joined.is_file() {
        return Some(apply_browser_rewrite(
            package_dir,
            package_json,
            joined,
            false,
            browser_context,
        ));
    }
    if let Some(path) = resolve_package_file(package_dir, subpath) {
        return Some(apply_browser_rewrite(
            package_dir,
            package_json,
            path,
            false,
            browser_context,
        ));
    }

    if let Some(exports) = &package_json.exports {
        if let Some(path) = resolve_package_exports(package_dir, exports, subpath) {
            return Some(apply_browser_rewrite(
                package_dir,
                package_json,
                path,
                false,
                browser_context,
            ));
        }
    }

    None
}

fn apply_browser_rewrite(
    package_dir: &Path,
    package_json: &PackageJson,
    resolved_path: PathBuf,
    allow_browser_string: bool,
    browser_context: bool,
) -> PackageResolutionOutcome {
    if !browser_context {
        return PackageResolutionOutcome::Resolved(resolved_path);
    }

    let Some(browser) = package_json.browser.as_ref() else {
        return PackageResolutionOutcome::Resolved(resolved_path);
    };

    match browser {
        serde_json::Value::String(path) if allow_browser_string => {
            resolve_package_file(package_dir, path)
                .map(PackageResolutionOutcome::Resolved)
                .unwrap_or(PackageResolutionOutcome::BrowserBlocked)
        }
        serde_json::Value::Object(map) => {
            let Some(relative_path) = resolved_path.strip_prefix(package_dir).ok() else {
                return PackageResolutionOutcome::Resolved(resolved_path);
            };
            let key = format!("./{}", relative_path.to_string_lossy().replace('\\', "/"));
            match map.get(&key) {
                Some(serde_json::Value::Bool(false)) => PackageResolutionOutcome::BrowserBlocked,
                Some(serde_json::Value::String(path)) => resolve_package_file(package_dir, path)
                    .map(PackageResolutionOutcome::Resolved)
                    .unwrap_or(PackageResolutionOutcome::BrowserBlocked),
                Some(_) => PackageResolutionOutcome::BrowserBlocked,
                None => PackageResolutionOutcome::Resolved(resolved_path),
            }
        }
        _ => PackageResolutionOutcome::Resolved(resolved_path),
    }
}

fn resolve_package_exports(
    package_dir: &Path,
    exports: &serde_json::Value,
    subpath: &str,
) -> Option<PathBuf> {
    match exports {
        serde_json::Value::String(path) => resolve_package_file(package_dir, path),
        serde_json::Value::Object(map) => {
            let requested_key = if subpath.is_empty() {
                ".".to_string()
            } else {
                format!("./{}", subpath)
            };

            if let Some(value) = map.get(&requested_key).or_else(|| map.get(".")) {
                if let Some(path) = resolve_package_exports_target(package_dir, value, None) {
                    return Some(path);
                }
            }

            let mut pattern_matches = map
                .iter()
                .filter_map(|(key, value)| {
                    let capture = match_export_pattern(key, &requested_key)?;
                    Some((key, capture, value))
                })
                .collect::<Vec<_>>();
            pattern_matches.sort_by(
                |(left_key, left_capture, _), (right_key, right_capture, _)| {
                    right_key
                        .len()
                        .cmp(&left_key.len())
                        .then_with(|| left_capture.len().cmp(&right_capture.len()))
                        .then_with(|| left_key.cmp(right_key))
                },
            );

            for (_, capture, value) in pattern_matches {
                if let Some(path) =
                    resolve_package_exports_target(package_dir, value, Some(capture))
                {
                    return Some(path);
                }
            }

            None
        }
        _ => None,
    }
}

fn resolve_package_exports_target(
    package_dir: &Path,
    value: &serde_json::Value,
    capture: Option<&str>,
) -> Option<PathBuf> {
    match value {
        serde_json::Value::String(path) => {
            let candidate = substitute_export_pattern(path, capture);
            resolve_package_file(package_dir, &candidate)
        }
        serde_json::Value::Object(branches) => {
            for branch in ["deno", "browser", "import", "require", "default"] {
                if let Some(branch_value) = branches.get(branch) {
                    if let Some(path) = branch_value
                        .as_str()
                        .map(|path| substitute_export_pattern(path, capture))
                        .and_then(|path| resolve_package_file(package_dir, &path))
                    {
                        return Some(path);
                    }
                }
            }
            None
        }
        _ => None,
    }
}

fn substitute_export_pattern(path: &str, capture: Option<&str>) -> String {
    match capture {
        Some(capture) => path.replace('*', capture),
        None => path.to_string(),
    }
}

fn match_export_pattern<'a>(pattern: &'a str, requested_key: &'a str) -> Option<&'a str> {
    let (prefix, suffix) = pattern.split_once('*')?;
    if requested_key.len() < prefix.len() + suffix.len() {
        return None;
    }
    if !requested_key.starts_with(prefix) || !requested_key.ends_with(suffix) {
        return None;
    }

    Some(&requested_key[prefix.len()..requested_key.len() - suffix.len()])
}

fn resolve_package_file(package_dir: &Path, candidate: &str) -> Option<PathBuf> {
    let path = package_dir.join(candidate);
    if path.is_file() {
        return Some(path);
    }

    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        if matches!(
            ext,
            "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs"
        ) {
            return None;
        }
    }

    for extension in [
        "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "d.ts", "d.mts", "d.cts",
    ] {
        let with_ext = package_dir.join(format!("{}.{}", candidate, extension));
        if with_ext.is_file() {
            return Some(with_ext);
        }
    }

    None
}

/// Check whether a project root needs installation before analysis/execution.
pub fn project_requires_install(root: impl AsRef<Path>) -> bool {
    match load_manifest(root) {
        Ok(Some(manifest)) => {
            !manifest.dependencies.is_empty() || !manifest.dev_dependencies.is_empty()
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
