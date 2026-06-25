use crate::*;

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

pub(crate) fn resolve_types_package_import(
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

pub(crate) fn resolve_package_types_entry(package_dir: &Path, package_json: &PackageJson) -> Option<PathBuf> {
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

pub(crate) enum PackageResolutionOutcome {
    Resolved(PathBuf),
    BrowserBlocked,
}

pub(crate) fn resolve_package_entry(
    package_dir: &Path,
    package_json: &PackageJson,
    browser_context: bool,
) -> Option<PackageResolutionOutcome> {
    if let Some(exports) = &package_json.exports {
        if let Some(path) = resolve_package_exports(package_dir, exports, "", browser_context) {
            return Some(apply_browser_rewrite(
                package_dir,
                package_json,
                path,
                true,
                browser_context,
            ));
        }
    }

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

pub(crate) fn resolve_package_subpath(
    package_dir: &Path,
    package_json: &PackageJson,
    subpath: &str,
    browser_context: bool,
) -> Option<PackageResolutionOutcome> {
    if let Some(exports) = &package_json.exports {
        if let Some(path) = resolve_package_exports(package_dir, exports, subpath, browser_context)
        {
            return Some(apply_browser_rewrite(
                package_dir,
                package_json,
                path,
                false,
                browser_context,
            ));
        }
    }

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

    None
}

pub(crate) fn apply_browser_rewrite(
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

pub(crate) fn resolve_package_exports(
    package_dir: &Path,
    exports: &serde_json::Value,
    subpath: &str,
    browser_context: bool,
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
                if let Some(path) =
                    resolve_package_exports_target(package_dir, value, None, browser_context)
                {
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
                if let Some(path) = resolve_package_exports_target(
                    package_dir,
                    value,
                    Some(capture),
                    browser_context,
                ) {
                    return Some(path);
                }
            }

            None
        }
        _ => None,
    }
}

pub(crate) fn resolve_package_exports_target(
    package_dir: &Path,
    value: &serde_json::Value,
    capture: Option<&str>,
    browser_context: bool,
) -> Option<PathBuf> {
    match value {
        serde_json::Value::String(path) => {
            let candidate = substitute_export_pattern(path, capture);
            resolve_package_file(package_dir, &candidate)
        }
        serde_json::Value::Object(branches) => {
            let branch_order: &[&str] = if browser_context {
                &["browser", "import", "require", "default"]
            } else {
                &["deno", "import", "require", "default"]
            };

            for &branch in branch_order {
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

pub(crate) fn substitute_export_pattern(path: &str, capture: Option<&str>) -> String {
    match capture {
        Some(capture) => path.replace('*', capture),
        None => path.to_string(),
    }
}

pub(crate) fn match_export_pattern<'a>(pattern: &'a str, requested_key: &'a str) -> Option<&'a str> {
    let (prefix, suffix) = pattern.split_once('*')?;
    if requested_key.len() < prefix.len() + suffix.len() {
        return None;
    }
    if !requested_key.starts_with(prefix) || !requested_key.ends_with(suffix) {
        return None;
    }

    Some(&requested_key[prefix.len()..requested_key.len() - suffix.len()])
}

pub(crate) fn resolve_package_file(package_dir: &Path, candidate: &str) -> Option<PathBuf> {
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

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod resolve_tests;
