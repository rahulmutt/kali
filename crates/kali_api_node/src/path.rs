//! Node.js `path` module helpers.

use std::{
    env,
    path::{Component, Path, PathBuf},
};

/// Canonicalize a path using lexical `.` / `..` resolution.
///
/// This intentionally stays filesystem-agnostic so the helper remains deterministic for tests
/// and build-time host analysis.
pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

/// Join a base path and segment.
pub fn join_path(base: impl AsRef<Path>, segment: impl AsRef<Path>) -> PathBuf {
    let mut joined = PathBuf::from(base.as_ref());
    joined.push(segment.as_ref());
    joined
}

/// Resolve a path against a base path.
pub fn resolve_path(base: impl AsRef<Path>, input: impl AsRef<Path>) -> PathBuf {
    let input = input.as_ref();
    if input.is_absolute() {
        normalize_path(input)
    } else {
        normalize_path(join_path(base, input))
    }
}

/// Compute a lexical relative path between two locations.
///
/// This keeps the helper deterministic while still mirroring the shape of
/// Node's `path.relative` API closely enough for the compatibility layer.
pub fn relative_path(from: impl AsRef<Path>, to: impl AsRef<Path>) -> PathBuf {
    let from = resolve_node_path(from);
    let to = resolve_node_path(to);

    if path_root_key(&from) != path_root_key(&to) {
        return to;
    }

    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();
    let mut shared_prefix = 0;
    while shared_prefix < from_components.len()
        && shared_prefix < to_components.len()
        && from_components[shared_prefix] == to_components[shared_prefix]
    {
        shared_prefix += 1;
    }

    let mut relative = PathBuf::new();
    for component in from_components.iter().skip(shared_prefix) {
        if matches!(
            component,
            Component::Normal(_) | Component::CurDir | Component::ParentDir
        ) {
            relative.push("..");
        }
    }

    for component in to_components.iter().skip(shared_prefix) {
        match component {
            Component::RootDir | Component::Prefix(_) => {}
            _ => relative.push(component.as_os_str()),
        }
    }

    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

fn resolve_node_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        normalize_path(path)
    } else {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        normalize_path(join_path(cwd, path))
    }
}

fn path_root_key(path: impl AsRef<Path>) -> Option<String> {
    let mut components = path.as_ref().components();
    match components.next()? {
        Component::Prefix(prefix) => Some(prefix.as_os_str().to_string_lossy().into_owned()),
        Component::RootDir => Some(String::from("/")),
        _ => None,
    }
}

/// Return the parent directory of a path, or `.` if it has no parent.
pub fn dirname(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Return the final path component as a string.
pub fn basename(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

/// Return the final extension, including the leading `.` when present.
pub fn extname(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return String::new();
    };
    let Some(dot_index) = file_name.rfind('.') else {
        return String::new();
    };
    if dot_index == 0 {
        String::new()
    } else {
        file_name[dot_index..].to_string()
    }
}

/// A namespace-style projection of path helpers used by the Node compatibility layer.
#[derive(Clone, Copy, Debug, Default)]
pub struct NodePath;

impl NodePath {
    pub fn normalize(path: impl AsRef<Path>) -> PathBuf {
        normalize_path(path)
    }

    pub fn join(base: impl AsRef<Path>, segment: impl AsRef<Path>) -> PathBuf {
        join_path(base, segment)
    }

    pub fn resolve(base: impl AsRef<Path>, input: impl AsRef<Path>) -> PathBuf {
        resolve_path(base, input)
    }

    pub fn relative(from: impl AsRef<Path>, to: impl AsRef<Path>) -> PathBuf {
        relative_path(from, to)
    }

    pub fn dirname(path: impl AsRef<Path>) -> PathBuf {
        dirname(path)
    }

    pub fn basename(path: impl AsRef<Path>) -> String {
        basename(path)
    }

    pub fn extname(path: impl AsRef<Path>) -> String {
        extname(path)
    }
}

#[cfg(test)]
#[path = "path_tests.rs"]
mod path_tests;
