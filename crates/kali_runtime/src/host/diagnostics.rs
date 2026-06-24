//! Host-import error helpers, runtime diagnostics, and path resolution.
use crate::*;

pub(crate) fn host_import_error(name: &str, error: impl std::fmt::Display) -> Diagnostic {
    Diagnostic::error(
        e4::UNCAUGHT_ERROR as u32,
        format!("failed to register host import '{}': {}", name, error),
    )
}

pub(crate) fn runtime_error_diagnostic(error: impl std::fmt::Display) -> Diagnostic {
    let message = error.to_string();
    if message.contains("KALI_E4001") || message.contains("E4001") {
        Diagnostic::error(e4::EFFECT_NOT_PERMITTED as u32, message)
    } else if message.contains("KALI_E4003")
        || message.contains("E4003")
        || message.contains("fuel")
        || message.contains("memory limit")
        || message.contains("resource limit")
    {
        Diagnostic::error(e4::RESOURCE_LIMIT_EXCEEDED as u32, message)
    } else {
        Diagnostic::error(e4::UNCAUGHT_ERROR as u32, message)
    }
}

pub(crate) fn resolve_host_path(state: &KaliHostState, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        state.cwd.join(path)
    }
}

pub(crate) fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push("..");
                }
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
            std::path::Component::Normal(part) => normalized.push(part),
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}
