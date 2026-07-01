use crate::*;

#[derive(Debug, Clone)]
pub enum PackageTarget {
    Registry {
        registry: String,
        name: String,
        version: Option<String>,
    },
    RawUrl(String),
}

pub(crate) fn parse_package_target(target: &str) -> Result<PackageTarget, Diagnostic> {
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

pub(crate) fn split_package_name_and_version(
    spec: &str,
) -> Result<(String, Option<String>), Diagnostic> {
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

pub(crate) fn encode_package_name(name: &str) -> String {
    urlencoding::encode(name).into_owned()
}

pub(crate) fn package_key(name: &str, version: &str) -> String {
    format!("{}@{}", name, version)
}

pub(crate) fn install_name_from_package(name: &str) -> String {
    name.trim_start_matches("jsr:").to_string()
}

pub(crate) fn jsr_compat_name(name: &str) -> String {
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

pub(crate) fn types_package_name(name: &str) -> String {
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

pub(crate) fn split_bare_package_source(source: &str) -> Option<(String, Option<&str>)> {
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
