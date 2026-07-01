use crate::*;

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub target: Option<String>,
    pub dev: bool,
    pub allow_scripts: bool,
    pub suppress_script_output: bool,
}

#[derive(Debug, Clone)]
pub struct InstallSummary {
    pub manifest_path: Option<PathBuf>,
    pub lock_path: Option<PathBuf>,
    pub installed: Vec<String>,
    pub removed: Vec<String>,
}

pub(crate) fn ensure_lock_install_name_unique(
    install_names: &mut BTreeMap<String, String>,
    key: &str,
) -> Result<(), Diagnostic> {
    let Some((name, _version)) = split_package_key(key) else {
        return Err(Diagnostic::error(
            e6::INVALID_LOCK_FILE as u32,
            format!("invalid package key '{}' in lock file", key),
        ));
    };

    let install_name = install_name_from_package(name);
    match install_names.entry(install_name) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(key.to_string());
            Ok(())
        }
        std::collections::btree_map::Entry::Occupied(entry) if entry.get() == key => Ok(()),
        std::collections::btree_map::Entry::Occupied(entry) => Err(Diagnostic::error(
            e6::VERSION_MISMATCH as u32,
            format!(
                "packages '{}' and '{}' would both materialize to node_modules/{}",
                entry.get(),
                key,
                entry.key()
            ),
        )),
    }
}

pub(crate) fn collect_reachable_registry_packages(
    lock: &LockFile,
    root_keys: &[String],
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut reachable = BTreeSet::new();
    let mut install_names = BTreeMap::new();
    let mut stack = root_keys.to_vec();

    while let Some(key) = stack.pop() {
        if !reachable.insert(key.clone()) {
            continue;
        }

        let package = lock.packages.get(&key).ok_or_else(|| {
            Diagnostic::error(
                e6::INSTALL_REQUIRED as u32,
                format!(
                    "package '{}' must be installed before this command can proceed",
                    key
                ),
            )
        })?;

        ensure_lock_install_name_unique(&mut install_names, &key)?;

        for (dep_name, dep_version) in &package.dependencies {
            stack.push(package_key(dep_name, dep_version));
        }
    }

    Ok(reachable)
}

pub(crate) fn prune_unreachable_registry_packages(
    root: &Path,
    lock: &mut LockFile,
    reachable: &BTreeSet<String>,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let unreachable = lock
        .packages
        .keys()
        .filter(|key| !reachable.contains(*key))
        .cloned()
        .collect::<Vec<_>>();

    if unreachable.is_empty() {
        return Ok(Vec::new());
    }

    let remaining_install_names = lock
        .packages
        .keys()
        .filter(|key| reachable.contains(*key))
        .filter_map(|key| {
            split_package_key(key).map(|(name, _version)| install_name_from_package(name))
        })
        .collect::<BTreeSet<_>>();

    let mut removed = Vec::new();
    for key in unreachable {
        lock.packages.remove(&key);
        removed.push(key.clone());

        let cache_dir = root.join(".kali-cache").join("packages").join(&key);
        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to remove stale package cache '{}': {}",
                        cache_dir.display(),
                        error
                    ),
                )]
            })?;
        }

        if let Some((name, _version)) = split_package_key(&key) {
            let install_name = install_name_from_package(name);
            if !remaining_install_names.contains(&install_name) {
                let install_path = root.join("node_modules").join(&install_name);
                if install_path.exists() {
                    fs::remove_dir_all(&install_path).map_err(|error| {
                        vec![Diagnostic::error(
                            e6::INSTALL_FAILED as u32,
                            format!(
                                "failed to remove stale install path '{}': {}",
                                install_path.display(),
                                error
                            ),
                        )]
                    })?;
                }
            }
        }
    }

    Ok(removed)
}

pub(crate) fn discover_install_source_files(root: &Path) -> Result<Vec<PathBuf>, Diagnostic> {
    let mut files = Vec::new();
    collect_install_source_files(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

pub(crate) fn collect_install_source_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), Diagnostic> {
    let entries = fs::read_dir(current).map_err(|error| {
        Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to read directory '{}': {}",
                current.display(),
                error
            ),
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!("failed to read entry in '{}': {}", current.display(), error),
            )
        })?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if path.is_dir() {
            if matches!(
                name.as_ref(),
                ".git" | "node_modules" | ".kali-cache" | "target"
            ) || name.starts_with('.')
            {
                continue;
            }
            if path != root && path.join("kali.json").exists() {
                continue;
            }
            collect_install_source_files(root, &path, files)?;
        } else if is_install_source_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

pub(crate) fn is_install_source_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    if name.ends_with(".test.ts")
        || name.ends_with(".spec.ts")
        || name.ends_with(".test.js")
        || name.ends_with(".spec.js")
        || name.ends_with(".test.tsx")
        || name.ends_with(".spec.tsx")
        || name.ends_with(".test.mts")
        || name.ends_with(".spec.mts")
        || name.ends_with(".test.cts")
        || name.ends_with(".spec.cts")
    {
        return false;
    }

    name.ends_with(".ts")
        || name.ends_with(".tsx")
        || name.ends_with(".js")
        || name.ends_with(".jsx")
        || name.ends_with(".mts")
        || name.ends_with(".cts")
        || name.ends_with(".d.ts")
        || name.ends_with(".d.mts")
        || name.ends_with(".d.cts")
}

pub(crate) fn collect_source_module_specifiers(source: &str) -> BTreeSet<String> {
    let mut specifiers = BTreeSet::new();

    for quote in ['"', '\'', '`'] {
        let mut offset = 0usize;
        while let Some(start_rel) = source[offset..].find(quote) {
            let start = offset + start_rel;
            let after = &source[start + quote.len_utf8()..];
            let Some(end_rel) = after.find(quote) else {
                break;
            };
            let candidate = &after[..end_rel];
            let mut context_start = start.saturating_sub(160);
            while context_start > 0 && !source.is_char_boundary(context_start) {
                context_start -= 1;
            }
            let context = &source[context_start..start];
            if !candidate.trim().is_empty()
                && !candidate.chars().any(|ch| ch.is_whitespace())
                && (context.contains("import")
                    || context.contains("export")
                    || context.contains("from")
                    || context.contains("require("))
            {
                specifiers.insert(candidate.to_string());
            }
            offset = start + quote.len_utf8() + end_rel + quote.len_utf8();
            if offset >= source.len() {
                break;
            }
        }
    }

    specifiers
}

pub(crate) fn resolve_import_map_specifier(
    specifier: &str,
    imports: &BTreeMap<String, String>,
) -> Option<String> {
    let mut best: Option<(&str, &str)> = None;

    for (key, target) in imports {
        let matched = if key.ends_with('/') {
            specifier.starts_with(key)
        } else {
            specifier == key
        };

        if matched && best.is_none_or(|(best_key, _)| key.len() > best_key.len()) {
            best = Some((key.as_str(), target.as_str()));
        }
    }

    let (key, target) = best?;
    if key.ends_with('/') {
        if target.ends_with('/') {
            Some(format!("{}{}", target, &specifier[key.len()..]))
        } else {
            None
        }
    } else {
        Some(target.to_string())
    }
}

pub(crate) fn is_raw_url(specifier: &str) -> bool {
    specifier.starts_with("https://") || specifier.starts_with("http://")
}

pub(crate) fn discover_install_time_raw_urls(
    root: &Path,
    manifest: &ProjectManifest,
) -> Result<BTreeSet<String>, Diagnostic> {
    let mut urls = BTreeSet::new();

    for (key, target) in &manifest.imports {
        if !key.ends_with('/') && is_raw_url(target) {
            urls.insert(target.clone());
        }
    }

    for source_file in discover_install_source_files(root)? {
        let source = fs::read_to_string(&source_file).map_err(|error| {
            Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to read source file '{}': {}",
                    source_file.display(),
                    error
                ),
            )
        })?;

        for specifier in collect_source_module_specifiers(&source) {
            let resolved =
                resolve_import_map_specifier(&specifier, &manifest.imports).unwrap_or(specifier);
            if is_raw_url(&resolved) {
                urls.insert(resolved);
            }
        }
    }

    Ok(urls)
}

pub(crate) fn prune_unreachable_raw_urls(
    root: &Path,
    lock: &mut LockFile,
    reachable: &BTreeSet<String>,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    let unreachable = lock
        .raw_urls
        .keys()
        .filter(|url| !reachable.contains(*url))
        .cloned()
        .collect::<Vec<_>>();

    let mut removed = Vec::new();
    for url in unreachable {
        if let Some(entry) = lock.raw_urls.remove(&url) {
            removed.push(url.clone());
            remove_cached_raw_url_entry(root, &entry.cached)?;
        }
    }

    Ok(removed)
}

pub(crate) fn remove_cached_raw_url_entry(
    root: &Path,
    cached: &str,
) -> Result<(), Vec<Diagnostic>> {
    let cached_path = Path::new(cached);
    if cached_path.exists() {
        if cached_path.is_dir() {
            fs::remove_dir_all(cached_path).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to remove raw URL cache '{}': {}",
                        cached_path.display(),
                        error
                    ),
                )]
            })?;
        } else {
            fs::remove_file(cached_path).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to remove raw URL cache '{}': {}",
                        cached_path.display(),
                        error
                    ),
                )]
            })?;
        }
    }

    let raw_root = root.join(".kali-cache").join("raw");
    let mut current = cached_path.parent();
    while let Some(dir) = current {
        if dir == raw_root || !dir.starts_with(&raw_root) {
            break;
        }

        let is_empty = fs::read_dir(dir)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true);
        if !is_empty {
            break;
        }

        fs::remove_dir(dir).map_err(|error| {
            vec![Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to remove empty raw cache directory '{}': {}",
                    dir.display(),
                    error
                ),
            )]
        })?;
        current = dir.parent();
    }

    Ok(())
}

pub(crate) fn reconcile_raw_urls(
    root: &Path,
    lock: &mut LockFile,
    declared_raw_urls: &BTreeSet<String>,
    installed: &mut BTreeSet<String>,
) -> Result<Vec<String>, Vec<Diagnostic>> {
    for url in declared_raw_urls {
        let needs_install = match lock.raw_urls.get(url) {
            Some(entry) => !Path::new(&entry.cached).exists(),
            None => true,
        };

        if needs_install {
            install_raw_url(root, lock, url, installed)?;
        } else {
            installed.insert(url.clone());
        }
    }

    prune_unreachable_raw_urls(root, lock, declared_raw_urls)
}

pub(crate) fn has_effective_npm_scriptable_install_work(
    manifest: &ProjectManifest,
    target: Option<&PackageTarget>,
) -> bool {
    match target {
        Some(PackageTarget::Registry { registry, .. }) => registry == "npm",
        Some(PackageTarget::RawUrl(_)) => false,
        None => manifest
            .dependencies
            .keys()
            .chain(manifest.dev_dependencies.keys())
            .any(|name| !name.starts_with("jsr:")),
    }
}

pub fn install_project(
    root: impl AsRef<Path>,
    options: InstallOptions,
) -> Result<InstallSummary, Vec<Diagnostic>> {
    let root = root.as_ref();
    fs::create_dir_all(root).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to create project root '{}': {}",
                root.display(),
                error
            ),
        )]
    })?;

    let mut manifest = match load_manifest(root) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => ProjectManifest::minimal(),
        Err(diagnostic) => return Err(vec![diagnostic]),
    };

    let parsed_target = match options.target.as_deref() {
        Some(target) => Some(parse_package_target(target).map_err(|diagnostic| vec![diagnostic])?),
        None => None,
    };

    if matches!(
        parsed_target.as_ref(),
        Some(PackageTarget::Registry {
            version: Some(_),
            ..
        })
    ) {
        return Err(vec![Diagnostic::error(
            e5::INVALID_CLI_USAGE as u32,
            "`kali install` accepts only registry package identifiers, not explicit versions",
        )]);
    }

    if options.dev {
        match parsed_target.as_ref() {
            None => {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    "`--dev` requires an explicit registry package target",
                )]);
            }
            Some(PackageTarget::RawUrl(_)) => {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    "`--dev` is not valid for raw-URL targets",
                )]);
            }
            _ => {}
        }
    }

    if options.allow_scripts {
        match parsed_target.as_ref() {
            Some(PackageTarget::RawUrl(_)) => {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    "`--allow-scripts` is not valid for raw-URL targets",
                )]);
            }
            Some(PackageTarget::Registry { registry, .. }) if registry == "jsr" => {
                return Err(vec![Diagnostic::error(
                    e5::INVALID_CLI_USAGE as u32,
                    "`--allow-scripts` is not valid for JSR targets",
                )]);
            }
            _ => {}
        }

        if !has_effective_npm_scriptable_install_work(&manifest, parsed_target.as_ref()) {
            return Err(vec![Diagnostic::error(
                e5::INVALID_CLI_USAGE as u32,
                "`kali install --allow-scripts` requires non-empty npm install work in the current invocation",
            )]);
        }
    }

    let mut lock = match load_lock(root) {
        Ok(Some(lock)) => lock,
        Ok(None) => LockFile::minimal(),
        Err(diagnostic) => return Err(vec![diagnostic]),
    };

    let mut installed = BTreeSet::new();
    let mut installed_paths = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let mut explicit_raw_url: Option<String> = None;
    let mut resolved_root_keys = BTreeSet::new();
    let mut removed = Vec::new();
    let host_fit_context = package_host_fit_context_for_manifest(&manifest);

    if let Some(target) = parsed_target {
        match target {
            PackageTarget::Registry {
                registry,
                name,
                version,
            } => {
                let resolved = match resolve_registry_package(&registry, &name, version.as_deref())
                {
                    Ok(resolved) => resolved,
                    Err(diagnostic) => return Err(vec![diagnostic]),
                };

                let dependency_version = resolved.version.clone();
                if options.dev {
                    manifest
                        .dev_dependencies
                        .insert(resolved.name.clone(), dependency_version.clone());
                } else {
                    manifest
                        .dependencies
                        .insert(resolved.name.clone(), dependency_version.clone());
                }

                validate_manifest_registry_collisions(&manifest)?;

                resolved_root_keys.insert(package_key(&resolved.name, &resolved.version));
                install_registry_package(
                    root,
                    &mut lock,
                    &resolved,
                    options.allow_scripts,
                    options.suppress_script_output,
                    host_fit_context,
                    &mut installed,
                    &mut installed_paths,
                    &mut diagnostics,
                )?;
            }
            PackageTarget::RawUrl(url) => {
                validate_manifest_registry_collisions(&manifest)?;
                explicit_raw_url = Some(url);
            }
        }
    } else if !manifest.dependencies.is_empty() || !manifest.dev_dependencies.is_empty() {
        validate_manifest_registry_collisions(&manifest)?;
        for (name, version) in manifest
            .dependencies
            .iter()
            .chain(manifest.dev_dependencies.iter())
        {
            let registry = if name.starts_with("jsr:") {
                "jsr"
            } else {
                "npm"
            };
            let resolved = resolve_registry_package(registry, name, Some(version.as_str()))
                .map_err(|diagnostic| vec![diagnostic])?;
            resolved_root_keys.insert(package_key(&resolved.name, &resolved.version));
            install_registry_package(
                root,
                &mut lock,
                &resolved,
                options.allow_scripts,
                options.suppress_script_output,
                host_fit_context,
                &mut installed,
                &mut installed_paths,
                &mut diagnostics,
            )?;
        }
    }

    let mut declared_raw_urls =
        discover_install_time_raw_urls(root, &manifest).map_err(|diagnostic| vec![diagnostic])?;
    if let Some(url) = explicit_raw_url {
        declared_raw_urls.insert(url);
    }
    removed.extend(reconcile_raw_urls(
        root,
        &mut lock,
        &declared_raw_urls,
        &mut installed,
    )?);

    let root_keys = if resolved_root_keys.is_empty() {
        manifest_registry_package_keys(&manifest)
    } else {
        resolved_root_keys.into_iter().collect::<Vec<_>>()
    };
    let reachable = collect_reachable_registry_packages(&lock, &root_keys)
        .map_err(|diagnostic| vec![diagnostic])?;
    removed.extend(prune_unreachable_registry_packages(
        root, &mut lock, &reachable,
    )?);

    let manifest_path = if manifest.is_minimal()
        && manifest.dependencies.is_empty()
        && manifest.dev_dependencies.is_empty()
    {
        None
    } else {
        Some(save_manifest(root, &manifest).map_err(|diagnostic| vec![diagnostic])?)
    };

    let lock_path = if lock.packages.is_empty() && lock.raw_urls.is_empty() {
        let path = root.join("kali.lock");
        if path.exists() {
            fs::remove_file(&path).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to remove stale lock file '{}': {}",
                        path.display(),
                        error
                    ),
                )]
            })?;
        }
        None
    } else {
        Some(save_lock(root, &lock).map_err(|diagnostic| vec![diagnostic])?)
    };

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    removed.sort();
    removed.dedup();

    Ok(InstallSummary {
        manifest_path,
        lock_path,
        installed: installed.into_iter().collect(),
        removed,
    })
}

pub(crate) fn record_install_path(
    installed_paths: &mut BTreeMap<String, String>,
    install_name: &str,
    key: &str,
) -> Result<(), Vec<Diagnostic>> {
    match installed_paths.entry(install_name.to_string()) {
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(key.to_string());
            Ok(())
        }
        std::collections::btree_map::Entry::Occupied(entry) if entry.get() == key => Ok(()),
        std::collections::btree_map::Entry::Occupied(entry) => Err(vec![Diagnostic::error(
            e6::VERSION_MISMATCH as u32,
            format!(
                "packages '{}' and '{}' would both materialize to node_modules/{}",
                entry.get(),
                key,
                entry.key()
            ),
        )]),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn install_registry_package(
    root: &Path,
    lock: &mut LockFile,
    resolved: &ResolvedRegistryPackage,
    allow_scripts: bool,
    suppress_script_output: bool,
    host_fit_context: PackageHostFitContext,
    installed: &mut BTreeSet<String>,
    installed_paths: &mut BTreeMap<String, String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), Vec<Diagnostic>> {
    let _ = diagnostics;
    let key = package_key(&resolved.name, &resolved.version);
    let package_dir = root.join(".kali-cache").join("packages").join(&key);
    let node_modules_dir = root.join("node_modules");
    let install_path = node_modules_dir.join(&resolved.install_name);

    record_install_path(installed_paths, &resolved.install_name, &key)?;

    let locked = lock.packages.contains_key(&key);
    let verification_integrity = lock
        .packages
        .get(&key)
        .map(|package| package.integrity.clone())
        .or_else(|| resolved.integrity.clone());
    let package_dir_ready = package_dir.exists();
    let install_ready = install_path.exists();
    if locked && package_dir_ready && install_ready {
        let extracted_root = if package_dir.join("package").is_dir() {
            package_dir.join("package")
        } else {
            package_dir.clone()
        };
        let package_json = read_package_json(&extracted_root)?;
        validate_package_shape(&package_json, allow_scripts)?;
        validate_package_host_fit(&extracted_root, host_fit_context)
            .map_err(|diagnostic| vec![diagnostic])?;

        installed.insert(key.clone());
        if let Some(package) = lock.packages.get(&key) {
            let dependencies = package.dependencies.clone();
            for (dep_name, dep_spec) in dependencies {
                if installed.contains(&package_key(&dep_name, &dep_spec)) {
                    continue;
                }
                let dep_registry = if dep_name.starts_with("jsr:") {
                    "jsr"
                } else {
                    "npm"
                };
                let dep_resolved =
                    resolve_registry_package(dep_registry, &dep_name, Some(dep_spec.as_str()))
                        .map_err(|diagnostic| vec![diagnostic])?;
                install_registry_package(
                    root,
                    lock,
                    &dep_resolved,
                    allow_scripts,
                    suppress_script_output,
                    host_fit_context,
                    installed,
                    installed_paths,
                    diagnostics,
                )?;
            }
        }
        return Ok(());
    }

    if !package_dir_ready {
        fs::create_dir_all(&package_dir).map_err(|error| {
            vec![Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to create package cache '{}': {}",
                    package_dir.display(),
                    error
                ),
            )]
        })?;

        let tarball_bytes =
            download_bytes(&resolved.resolved).map_err(|diagnostic| vec![diagnostic])?;
        let integrity =
            verify_tarball_integrity(&tarball_bytes, verification_integrity.as_deref())?;
        extract_tarball(&tarball_bytes, &package_dir)?;

        let extracted_root = if package_dir.join("package").is_dir() {
            package_dir.join("package")
        } else {
            package_dir.clone()
        };

        let package_json = read_package_json(&extracted_root)?;
        validate_package_shape(&package_json, allow_scripts)?;
        validate_package_host_fit(&extracted_root, host_fit_context).map_err(|diagnostic| {
            let _ = fs::remove_dir_all(&package_dir);
            vec![diagnostic]
        })?;

        if let Some(parent) = install_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                vec![Diagnostic::error(
                    e6::INSTALL_FAILED as u32,
                    format!(
                        "failed to create node_modules path '{}': {}",
                        parent.display(),
                        error
                    ),
                )]
            })?;
        }
        copy_tree(&extracted_root, &install_path)?;

        let dependency_specs = package_json
            .dependencies
            .iter()
            .chain(package_json.optional_dependencies.iter())
            .map(|(name, version)| (name.clone(), version.clone()))
            .collect::<Vec<_>>();
        let mut resolved_dependencies = BTreeMap::new();

        for (dep_name, dep_spec) in dependency_specs {
            if installed.contains(&package_key(&dep_name, &dep_spec)) {
                continue;
            }
            let dep_registry = if dep_name.starts_with("jsr:") {
                "jsr"
            } else {
                "npm"
            };
            let dep_resolved =
                resolve_registry_package(dep_registry, &dep_name, Some(dep_spec.as_str()))
                    .map_err(|diagnostic| vec![diagnostic])?;
            resolved_dependencies.insert(dep_name.clone(), dep_resolved.version.clone());
            install_registry_package(
                root,
                lock,
                &dep_resolved,
                allow_scripts,
                suppress_script_output,
                host_fit_context,
                installed,
                installed_paths,
                diagnostics,
            )?;
        }

        lock.packages.insert(
            key.clone(),
            LockedPackage {
                registry: resolved.registry.clone(),
                integrity,
                resolved: resolved.resolved.clone(),
                dependencies: resolved_dependencies.clone(),
            },
        );

        run_package_lifecycle_hooks(
            &install_path,
            &package_json,
            allow_scripts,
            suppress_script_output,
        )?;

        installed.insert(key.clone());

        return Ok(());
    }

    let extracted_root = if package_dir.join("package").is_dir() {
        package_dir.join("package")
    } else {
        package_dir.clone()
    };

    let package_json = read_package_json(&extracted_root)?;
    validate_package_shape(&package_json, allow_scripts)?;
    validate_package_host_fit(&extracted_root, host_fit_context).map_err(|diagnostic| {
        let _ = fs::remove_dir_all(&package_dir);
        vec![diagnostic]
    })?;

    if let Some(parent) = install_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::error(
                e6::INSTALL_FAILED as u32,
                format!(
                    "failed to create node_modules path '{}': {}",
                    parent.display(),
                    error
                ),
            )]
        })?;
    }
    copy_tree(&extracted_root, &install_path)?;

    let dependency_specs = package_json
        .dependencies
        .iter()
        .chain(package_json.optional_dependencies.iter())
        .map(|(name, version)| (name.clone(), version.clone()))
        .collect::<Vec<_>>();
    let mut resolved_dependencies = BTreeMap::new();
    let integrity = if !locked {
        let tarball_bytes =
            download_bytes(&resolved.resolved).map_err(|diagnostic| vec![diagnostic])?;
        Some(verify_tarball_integrity(
            &tarball_bytes,
            verification_integrity.as_deref(),
        )?)
    } else {
        None
    };

    for (dep_name, dep_spec) in dependency_specs {
        if installed.contains(&package_key(&dep_name, &dep_spec)) {
            continue;
        }
        let dep_registry = if dep_name.starts_with("jsr:") {
            "jsr"
        } else {
            "npm"
        };
        let dep_resolved =
            resolve_registry_package(dep_registry, &dep_name, Some(dep_spec.as_str()))
                .map_err(|diagnostic| vec![diagnostic])?;
        resolved_dependencies.insert(dep_name.clone(), dep_resolved.version.clone());
        install_registry_package(
            root,
            lock,
            &dep_resolved,
            allow_scripts,
            suppress_script_output,
            host_fit_context,
            installed,
            installed_paths,
            diagnostics,
        )?;
    }

    if let Some(integrity) = integrity {
        lock.packages.insert(
            key.clone(),
            LockedPackage {
                registry: resolved.registry.clone(),
                integrity,
                resolved: resolved.resolved.clone(),
                dependencies: resolved_dependencies.clone(),
            },
        );
    }

    run_package_lifecycle_hooks(
        &install_path,
        &package_json,
        allow_scripts,
        suppress_script_output,
    )?;

    installed.insert(key.clone());

    Ok(())
}

pub(crate) fn install_raw_url(
    root: &Path,
    lock: &mut LockFile,
    url: &str,
    installed: &mut BTreeSet<String>,
) -> Result<(), Vec<Diagnostic>> {
    let bytes = download_bytes(url).map_err(|diagnostic| vec![diagnostic])?;
    let hash = sha256_hex(&bytes);
    let cache_dir = root
        .join(".kali-cache")
        .join("raw")
        .join(format!("sha256-{}", hash));
    fs::create_dir_all(&cache_dir).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to create raw cache '{}': {}",
                cache_dir.display(),
                error
            ),
        )]
    })?;

    let file_name = raw_url_file_name(url).unwrap_or_else(|| "index.ts".to_string());
    let cached = cache_dir.join(&file_name);
    fs::write(&cached, bytes).map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to write raw cache '{}': {}",
                cached.display(),
                error
            ),
        )]
    })?;

    lock.raw_urls.insert(
        url.to_string(),
        RawUrlEntry {
            integrity: format!("sha256-{}", hash),
            cached: cached.to_string_lossy().to_string(),
        },
    );
    installed.insert(url.to_string());
    Ok(())
}

pub(crate) fn run_package_lifecycle_hooks(
    package_dir: &Path,
    package_json: &PackageJson,
    allow_scripts: bool,
    suppress_output: bool,
) -> Result<(), Vec<Diagnostic>> {
    if !allow_scripts || package_json.scripts.is_empty() {
        return Ok(());
    }

    for phase in ["preinstall", "install", "postinstall"] {
        let Some(script) = package_json.scripts.get(phase) else {
            continue;
        };
        if script.trim().is_empty() {
            continue;
        }

        run_package_lifecycle_hook(package_dir, phase, script, suppress_output)?;
    }

    Ok(())
}

pub(crate) fn run_package_lifecycle_hook(
    package_dir: &Path,
    phase: &str,
    script: &str,
    suppress_output: bool,
) -> Result<(), Vec<Diagnostic>> {
    let mut command = if cfg!(windows) {
        let mut command = Command::new("cmd");
        command.arg("/C").arg(script);
        command
    } else {
        let mut command = Command::new("sh");
        command.arg("-c").arg(script);
        command
    };

    command.current_dir(package_dir);
    if suppress_output {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    let status = command.status().map_err(|error| {
        vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "failed to run npm lifecycle script '{}' in '{}': {}",
                phase,
                package_dir.display(),
                error
            ),
        )]
    })?;

    if !status.success() {
        return Err(vec![Diagnostic::error(
            e6::INSTALL_FAILED as u32,
            format!(
                "npm lifecycle script '{}' in '{}' failed with status {}",
                phase,
                package_dir.display(),
                status
            ),
        )]);
    }

    Ok(())
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod install_tests;
