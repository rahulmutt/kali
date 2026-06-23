//! Package root detection and native addon rejection helpers.

use super::*;

pub(crate) fn package_root_for_materialized_source(source: &Path) -> Option<PathBuf> {
    for ancestor in source.ancestors() {
        if !ancestor.join("package.json").exists() {
            continue;
        }

        let parent = ancestor.parent()?;
        if parent.file_name() == Some(std::ffi::OsStr::new("node_modules")) {
            return Some(ancestor.to_path_buf());
        }

        let grandparent = parent.parent()?;
        if parent
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('@'))
            && grandparent.file_name() == Some(std::ffi::OsStr::new("node_modules"))
        {
            return Some(ancestor.to_path_buf());
        }
    }

    None
}

pub(crate) fn reject_native_addon_package_source(source: &Path) -> Option<Diagnostic> {
    let package_root = package_root_for_materialized_source(source)?;
    let package_json_path = package_root.join("package.json");
    let package_json_contents = fs::read_to_string(&package_json_path).ok()?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_contents).ok()?;
    let package_name = package_json
        .get("name")
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| package_root.display().to_string());

    let entrypoint = package_json
        .get("main")
        .and_then(native_addon_path)
        .or_else(|| package_json.get("module").and_then(native_addon_path));
    if let Some(path) = entrypoint {
        return Some(Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            format!(
                "package '{}' publishes a native addon entrypoint '{}' and falls outside the pure JS/TS package contract",
                package_name, path
            ),
        ));
    }

    if package_json
        .get("exports")
        .is_some_and(value_contains_native_addon_path)
    {
        return Some(Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            format!(
                "package '{}' publishes a native addon exports target and falls outside the pure JS/TS package contract",
                package_name
            ),
        ));
    }

    if package_json
        .get("bin")
        .is_some_and(value_contains_native_addon_path)
    {
        return Some(Diagnostic::error(
            e6::INCOMPATIBLE_PACKAGE as u32,
            format!(
                "package '{}' publishes a native addon bin entrypoint and falls outside the pure JS/TS package contract",
                package_name
            ),
        ));
    }

    None
}

pub(crate) fn value_contains_native_addon_path(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(path) => path.ends_with(".node"),
        serde_json::Value::Array(values) => values.iter().any(value_contains_native_addon_path),
        serde_json::Value::Object(map) => map.values().any(value_contains_native_addon_path),
        _ => false,
    }
}

pub(crate) fn native_addon_path(value: &serde_json::Value) -> Option<&str> {
    value.as_str().filter(|path| path.ends_with(".node"))
}
