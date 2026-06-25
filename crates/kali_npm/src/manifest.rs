use crate::*;

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
            schema_version: crate::MANIFEST_SCHEMA,
            ..Self::default()
        }
    }

    pub fn is_minimal(&self) -> bool {
        self.schema_version == crate::MANIFEST_SCHEMA
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
            version: crate::LOCK_VERSION,
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
    if manifest.schema_version != crate::MANIFEST_SCHEMA {
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
    if lock.version != crate::LOCK_VERSION {
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

pub(crate) fn manifest_registry_package_keys(manifest: &ProjectManifest) -> Vec<String> {
    manifest
        .dependencies
        .iter()
        .chain(manifest.dev_dependencies.iter())
        .map(|(name, version)| package_key(name, version))
        .collect()
}

pub(crate) fn split_package_key(key: &str) -> Option<(&str, &str)> {
    key.rsplit_once('@')
}

pub(crate) fn validate_manifest_registry_collisions(
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
#[path = "manifest_tests.rs"]
mod manifest_tests;
