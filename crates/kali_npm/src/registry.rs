use crate::*;

/// The outcome of a registry package audit.
#[derive(Debug, Clone)]
pub struct RegistryPackageAudit {
    pub registry: String,
    pub name: String,
    pub version: String,
    pub findings: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedRegistryPackage {
    pub(crate) registry: String,
    pub(crate) name: String,
    pub(crate) install_name: String,
    pub(crate) version: String,
    pub(crate) resolved: String,
    pub(crate) integrity: Option<String>,
}

pub(crate) fn resolve_registry_package(
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

pub(crate) fn resolve_npm_package(
    name: &str,
    requested_version: Option<&str>,
) -> Result<ResolvedRegistryPackage, Diagnostic> {
    let metadata_url = npm_registry_metadata_url(name);
    resolve_npm_like_package("npm", name, name, &metadata_url, requested_version)
}

pub(crate) fn npm_registry_base_url() -> String {
    std::env::var("KALI_REGISTRY").unwrap_or_else(|_| crate::DEFAULT_NPM_REGISTRY.to_string())
}

pub(crate) fn npm_registry_metadata_url(name: &str) -> String {
    format!("{}/{}", npm_registry_base_url(), encode_package_name(name))
}

pub(crate) fn jsr_registry_metadata_url(name: &str) -> String {
    let raw_name = name.trim_start_matches("jsr:");
    let compat_name = jsr_compat_name(raw_name);
    format!("https://npm.jsr.io/{}", encode_package_name(&compat_name))
}

pub(crate) fn fetch_registry_metadata(
    registry: &str,
    display_name: &str,
    metadata_url: &str,
) -> Result<serde_json::Value, Diagnostic> {
    if let Some(cached) = crate::REGISTRY_METADATA_CACHE
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

    if let Ok(mut cache) = crate::REGISTRY_METADATA_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    {
        cache.insert(metadata_url.to_string(), metadata.clone());
    }

    Ok(metadata)
}

pub(crate) fn resolve_jsr_package(
    name: &str,
    requested_version: Option<&str>,
) -> Result<ResolvedRegistryPackage, Diagnostic> {
    let raw_name = name.trim_start_matches("jsr:");
    let metadata_url = jsr_registry_metadata_url(name);
    resolve_npm_like_package("jsr", name, raw_name, &metadata_url, requested_version)
}

pub(crate) fn resolve_npm_like_package(
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

pub(crate) fn audit_package_version_metadata(
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

    if version_meta
        .get("exports")
        .is_some_and(value_contains_native_addon_path)
    {
        findings.push(Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            format!(
                "package '{package_name}'@{version} in {registry} publishes a native addon exports target and falls outside the pure JS/TS package contract",
            ),
        ));
    }

    if version_meta
        .get("bin")
        .is_some_and(value_contains_native_addon_path)
    {
        findings.push(Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            format!(
                "package '{package_name}'@{version} in {registry} publishes a native addon bin entrypoint and falls outside the pure JS/TS package contract",
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

pub(crate) fn select_registry_version(
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

#[cfg(test)]
#[path = "registry_tests.rs"]
mod registry_tests;
