//! Package management for Kali (npm/JSR registry support).

use base64::Engine;
use flate2::read::GzDecoder;
use kali_error::{Diagnostic, _error_codes::e6};
use reqwest::blocking::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};
use tar::Archive;

const MANIFEST_SCHEMA: u32 = 1;
const LOCK_VERSION: u32 = 1;
const DEFAULT_NPM_REGISTRY: &str = "https://registry.npmjs.org";

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
}

#[derive(Debug, Clone)]
pub struct InstallSummary {
    pub manifest_path: Option<PathBuf>,
    pub lock_path: Option<PathBuf>,
    pub installed: Vec<String>,
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
        None => return Ok(()),
    };

    let requires_install =
        !manifest.dependencies.is_empty() || !manifest.dev_dependencies.is_empty();
    if !requires_install {
        return Ok(());
    }

    let lock = load_lock(root)?;
    let Some(lock) = lock else {
        return Err(Diagnostic::error(
            e6::INSTALL_REQUIRED as u32,
            "package installation is required before this command can proceed",
        ));
    };

    for (name, version) in manifest
        .dependencies
        .iter()
        .chain(manifest.dev_dependencies.iter())
    {
        let key = package_key(name, version);
        let install_dir = root
            .join("node_modules")
            .join(name.trim_start_matches("jsr:"));
        if !lock.packages.contains_key(&key) || !install_dir.exists() {
            return Err(Diagnostic::error(
                e6::INSTALL_REQUIRED as u32,
                format!(
                    "package '{}' must be installed before this command can proceed",
                    name
                ),
            ));
        }
    }

    Ok(())
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
        Ok(None) => {
            if options.target.is_some() {
                ProjectManifest::minimal()
            } else {
                return Ok(InstallSummary {
                    manifest_path: None,
                    lock_path: None,
                    installed: Vec::new(),
                });
            }
        }
        Err(diagnostic) => return Err(vec![diagnostic]),
    };

    let mut lock = match load_lock(root) {
        Ok(Some(lock)) => lock,
        Ok(None) => LockFile::minimal(),
        Err(diagnostic) => return Err(vec![diagnostic]),
    };

    let mut installed = BTreeSet::new();
    let mut diagnostics = Vec::new();

    if let Some(target) = options.target.as_deref() {
        match parse_package_target(target) {
            Ok(PackageTarget::Registry {
                registry,
                name,
                version,
            }) => {
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

                install_registry_package(
                    root,
                    &mut lock,
                    &resolved,
                    options.allow_scripts,
                    &mut installed,
                    &mut diagnostics,
                )?;
            }
            Ok(PackageTarget::RawUrl(url)) => {
                if options.allow_scripts {
                    return Err(vec![Diagnostic::error(
                        e6::RAW_URL_NOT_ALLOWED as u32,
                        "`--allow-scripts` is not valid for raw-URL targets",
                    )]);
                }
                validate_manifest_registry_collisions(&manifest)?;
                install_raw_url(root, &mut lock, &url, &mut installed, &mut diagnostics)?;
            }
            Err(diagnostic) => return Err(vec![diagnostic]),
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
            install_registry_package(
                root,
                &mut lock,
                &resolved,
                options.allow_scripts,
                &mut installed,
                &mut diagnostics,
            )?;
        }
    }

    let manifest_path = if manifest.is_minimal()
        && manifest.dependencies.is_empty()
        && manifest.dev_dependencies.is_empty()
        && options.target.is_none()
    {
        None
    } else {
        Some(save_manifest(root, &manifest).map_err(|diagnostic| vec![diagnostic])?)
    };

    let lock_path = if lock.packages.is_empty() && lock.raw_urls.is_empty() {
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

fn install_registry_package(
    root: &Path,
    lock: &mut LockFile,
    resolved: &ResolvedRegistryPackage,
    allow_scripts: bool,
    installed: &mut BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), Vec<Diagnostic>> {
    let _ = diagnostics;
    let key = package_key(&resolved.name, &resolved.version);
    if lock.packages.contains_key(&key) {
        installed.insert(key);
        return Ok(());
    }

    let package_dir = root.join(".kali-cache").join("packages").join(&key);
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
    let integrity = verify_tarball_integrity(&tarball_bytes, resolved.integrity.as_deref())?;
    extract_tarball(&tarball_bytes, &package_dir)?;

    let extracted_root = if package_dir.join("package").is_dir() {
        package_dir.join("package")
    } else {
        package_dir.clone()
    };

    let node_modules_dir = root.join("node_modules");
    let install_path = node_modules_dir.join(&resolved.install_name);
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

    let package_json = read_package_json(&install_path)?;
    validate_package_shape(&package_json, allow_scripts)?;
    let dependencies = package_json
        .dependencies
        .iter()
        .chain(package_json.optional_dependencies.iter())
        .map(|(name, version)| (name.clone(), version.clone()))
        .collect::<BTreeMap<_, _>>();

    lock.packages.insert(
        key.clone(),
        LockedPackage {
            registry: resolved.registry.clone(),
            integrity,
            resolved: resolved.resolved.clone(),
            dependencies: dependencies.clone(),
        },
    );
    installed.insert(key.clone());

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
            installed,
            diagnostics,
        )?;
    }

    Ok(())
}

fn install_raw_url(
    root: &Path,
    lock: &mut LockFile,
    url: &str,
    installed: &mut BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
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
    let _ = diagnostics;
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
    let metadata_url = format!("{}/{}", DEFAULT_NPM_REGISTRY, encode_package_name(name));
    resolve_npm_like_package("npm", name, name, &metadata_url, requested_version)
}

fn resolve_jsr_package(
    name: &str,
    requested_version: Option<&str>,
) -> Result<ResolvedRegistryPackage, Diagnostic> {
    let raw_name = name.trim_start_matches("jsr:");
    let compat_name = jsr_compat_name(raw_name);
    let metadata_url = format!("https://npm.jsr.io/{}", encode_package_name(&compat_name));
    resolve_npm_like_package("jsr", name, raw_name, &metadata_url, requested_version)
}

fn resolve_npm_like_package(
    registry: &str,
    display_name: &str,
    install_name_source: &str,
    metadata_url: &str,
    requested_version: Option<&str>,
) -> Result<ResolvedRegistryPackage, Diagnostic> {
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

    let version = if let Some(requested) = requested_version {
        if versions.contains_key(requested) {
            requested.to_string()
        } else {
            return Err(Diagnostic::error(
                e6::VERSION_MISMATCH as u32,
                format!(
                    "package '{}' does not publish version '{}'",
                    display_name, requested
                ),
            ));
        }
    } else {
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
        latest.to_string()
    };

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
    if !allow_scripts && !package_json.scripts.is_empty() {
        return Err(vec![Diagnostic::error(
            e6::LIFECYCLE_SCRIPT_REJECTED as u32,
            "npm lifecycle scripts require `--allow-scripts`",
        )]);
    }

    for (name, script) in &package_json.scripts {
        if matches!(name.as_str(), "install" | "preinstall" | "postinstall")
            && script.contains("node-gyp")
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct PackageJson {
    pub name: Option<String>,
    pub main: Option<String>,
    pub module: Option<String>,
    pub exports: Option<serde_json::Value>,
    #[serde(rename = "type")]
    pub package_type: Option<String>,
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
                .last()
                .map(|segment| segment.to_string())
        })
        .filter(|name| !name.is_empty())
}

/// Resolve a bare import against the materialized package graph.
pub fn resolve_materialized_import(root: impl AsRef<Path>, source: &str) -> Option<PathBuf> {
    let root = root.as_ref();
    let (package_name, subpath) = split_bare_package_source(source)?;
    let package_dir = root.join("node_modules").join(&package_name);
    if !package_dir.exists() {
        return None;
    }

    let package_json_path = package_dir.join("package.json");
    let package_json = fs::read_to_string(package_json_path)
        .ok()
        .and_then(|contents| serde_json::from_str::<PackageJson>(&contents).ok())?;

    if let Some(subpath) = subpath {
        return resolve_package_subpath(&package_dir, &package_json, subpath);
    }

    resolve_package_entry(&package_dir, &package_json)
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

fn resolve_package_entry(package_dir: &Path, package_json: &PackageJson) -> Option<PathBuf> {
    if let Some(main) = &package_json.main {
        if let Some(path) = resolve_package_file(package_dir, main) {
            return Some(path);
        }
    }

    if let Some(module) = &package_json.module {
        if let Some(path) = resolve_package_file(package_dir, module) {
            return Some(path);
        }
    }

    resolve_package_file(package_dir, "index.js")
        .or_else(|| resolve_package_file(package_dir, "index.mjs"))
        .or_else(|| resolve_package_file(package_dir, "index.ts"))
}

fn resolve_package_subpath(
    package_dir: &Path,
    package_json: &PackageJson,
    subpath: &str,
) -> Option<PathBuf> {
    let joined = package_dir.join(subpath);
    if joined.is_file() {
        return Some(joined);
    }
    if let Some(path) = resolve_package_file(package_dir, subpath) {
        return Some(path);
    }

    if let Some(exports) = &package_json.exports {
        if let Some(path) = resolve_package_exports(package_dir, exports, subpath) {
            return Some(path);
        }
    }

    None
}

fn resolve_package_exports(
    package_dir: &Path,
    exports: &serde_json::Value,
    subpath: &str,
) -> Option<PathBuf> {
    match exports {
        serde_json::Value::String(path) => resolve_package_file(package_dir, path),
        serde_json::Value::Object(map) => {
            let key = if subpath.is_empty() {
                ".".to_string()
            } else {
                format!("./{}", subpath)
            };
            let value = map.get(&key).or_else(|| map.get("."))?;
            match value {
                serde_json::Value::String(path) => resolve_package_file(package_dir, path),
                serde_json::Value::Object(branches) => {
                    for branch in ["deno", "browser", "import", "require", "default"] {
                        if let Some(branch_value) = branches.get(branch) {
                            if let Some(path) = branch_value
                                .as_str()
                                .and_then(|path| resolve_package_file(package_dir, path))
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
        _ => None,
    }
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

    for extension in ["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"] {
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
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn manifest_round_trip_is_deterministic() {
        let manifest = ProjectManifest {
            schema_version: MANIFEST_SCHEMA,
            dependencies: BTreeMap::from([("lodash".to_string(), "4.17.21".to_string())]),
            ..ProjectManifest::default()
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let parsed: ProjectManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.dependencies.get("lodash").unwrap(), "4.17.21");
    }

    #[test]
    fn lock_round_trip_is_deterministic() {
        let lock = LockFile {
            version: LOCK_VERSION,
            packages: BTreeMap::from([(
                "lodash@4.17.21".to_string(),
                LockedPackage {
                    registry: "npm".to_string(),
                    integrity: "sha512-demo".to_string(),
                    resolved: "https://example.com/lodash.tgz".to_string(),
                    dependencies: BTreeMap::new(),
                },
            )]),
            ..LockFile::default()
        };

        let json = serde_json::to_string_pretty(&lock).unwrap();
        let parsed: LockFile = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.packages.len(), 1);
    }

    #[test]
    fn bare_import_resolves_from_materialized_package() {
        let dir = tempdir().unwrap();
        let package_dir = dir.path().join("node_modules/lodash");
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(
            package_dir.join("package.json"),
            r#"{"name":"lodash","main":"lodash.js"}"#,
        )
        .unwrap();
        fs::write(package_dir.join("lodash.js"), "export default 1;").unwrap();

        let resolved = resolve_materialized_import(dir.path(), "lodash");
        assert_eq!(resolved.unwrap(), package_dir.join("lodash.js"));
    }

    #[test]
    fn manifest_registry_collisions_are_rejected_before_install() {
        let manifest = ProjectManifest {
            dependencies: BTreeMap::from([("@scope/name".to_string(), "1.0.0".to_string())]),
            dev_dependencies: BTreeMap::from([(
                "jsr:@scope/name".to_string(),
                "1.0.0".to_string(),
            )]),
            ..ProjectManifest::default()
        };

        let error = validate_manifest_registry_collisions(&manifest).unwrap_err();
        assert_eq!(error.len(), 1);
        let diagnostic = &error[0];
        assert_eq!(diagnostic.code, Some(e6::VERSION_MISMATCH as u32));
        assert!(diagnostic
            .message
            .contains("would both materialize to node_modules/@scope/name"));
    }

    #[test]
    fn manifest_registry_collisions_allow_identical_identity_spelling() {
        let manifest = ProjectManifest {
            dependencies: BTreeMap::from([("lodash".to_string(), "1.0.0".to_string())]),
            dev_dependencies: BTreeMap::new(),
            ..ProjectManifest::default()
        };

        validate_manifest_registry_collisions(&manifest)
            .expect("single dependency should be valid");
    }
}
