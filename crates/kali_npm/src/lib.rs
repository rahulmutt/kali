//! Package management for Kali (npm/JSR registry support).

/// Package metadata.
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub resolution: Resolution,
}

/// Package resolution information.
#[derive(Debug, Clone)]
pub enum Resolution {
    Npm { registry: String, specifier: String },
    Jsr { specifier: String },
    RawUrl { url: String },
}

/// Resolve package dependencies.
pub fn resolve_package(
    _name: &str,
    _specifier: &str,
) -> Result<PackageMetadata, ()> {
    Err(())
}
