use crate::*;

pub(crate) fn package_host_fit_context_for_manifest(
    manifest: &ProjectManifest,
) -> PackageHostFitContext {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PackageHostFitContext {
    DefaultStandalone,
    Node,
}

impl PackageHostFitContext {
    fn allows_node_only_host_apis(self) -> bool {
        matches!(self, Self::Node)
    }
}

pub(crate) fn read_package_json(package_dir: &Path) -> Result<PackageJson, Vec<Diagnostic>> {
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

pub(crate) fn value_contains_native_addon_path(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(path) => path.ends_with(".node"),
        serde_json::Value::Array(values) => values.iter().any(value_contains_native_addon_path),
        serde_json::Value::Object(map) => map.values().any(value_contains_native_addon_path),
        _ => false,
    }
}

pub(crate) fn validate_package_shape(
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
            .is_some_and(|script| script_uses_native_bootstrap_tool(script))
        {
            return Err(vec![Diagnostic::error(
                e6::INCOMPATIBLE_PACKAGE as u32,
                "package uses a native or binary bootstrap lifecycle script and falls outside the pure JS/TS package contract",
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

    if package_json
        .exports
        .as_ref()
        .is_some_and(value_contains_native_addon_path)
    {
        return Err(vec![Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            "package publishes a native addon exports target and falls outside the pure JS/TS package contract",
        )]);
    }

    if package_json
        .bin
        .as_ref()
        .is_some_and(value_contains_native_addon_path)
    {
        return Err(vec![Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            "package bin entry points to a native addon and falls outside the pure JS/TS package contract",
        )]);
    }

    Ok(())
}

pub(crate) fn script_uses_native_bootstrap_tool(script: &str) -> bool {
    let script = script.to_ascii_lowercase();
    [
        "node-gyp",
        "node-pre-gyp",
        "prebuild-install",
        "prebuildify",
        "cmake-js",
    ]
    .iter()
    .any(|needle| script.contains(needle))
}

pub(crate) fn validate_package_host_fit(
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

pub(crate) fn scan_for_node_only_host_api(
    root: &Path,
) -> Result<Option<(PathBuf, &'static str)>, Diagnostic> {
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

pub(crate) fn is_scannable_package_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
    )
}

pub(crate) fn should_skip_package_scan_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("node_modules" | ".git" | ".kali-cache" | "target")
    )
}

pub fn source_mentions_node_only_host_api(contents: &str) -> Option<&'static str> {
    for builtin in crate::NODE_ONLY_HOST_APIS {
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
pub(crate) struct PackageJson {
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

#[cfg(test)]
#[path = "validate_tests.rs"]
mod validate_tests;
