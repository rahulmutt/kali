use crate::*;

pub(crate) fn download_bytes(url: &str) -> Result<Vec<u8>, Diagnostic> {
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

pub(crate) fn verify_tarball_integrity(
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

pub(crate) fn integrity_matches(expected: &str, bytes: &[u8]) -> bool {
    if let Some(encoded) = expected.strip_prefix("sha512-") {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) {
            return decoded == Sha512::digest(bytes).to_vec();
        }
    }
    false
}

pub(crate) fn format_sha512(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    base64::engine::general_purpose::STANDARD.encode(digest)
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{:x}", digest)
}

pub(crate) fn extract_tarball(bytes: &[u8], package_dir: &Path) -> Result<(), Vec<Diagnostic>> {
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

pub(crate) fn copy_tree(source: &Path, target: &Path) -> Result<(), Vec<Diagnostic>> {
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

pub(crate) fn recursive_copy(source: &Path, target: &Path) -> Result<(), Vec<Diagnostic>> {
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

pub(crate) fn raw_url_file_name(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()
        .and_then(|parsed| {
            parsed
                .path_segments()?
                .next_back()
                .map(|segment| segment.to_string())
        })
        .filter(|name| !name.is_empty())
}
