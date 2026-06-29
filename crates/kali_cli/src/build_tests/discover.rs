use super::*;

#[test]
fn discover_dynamic_import_targets_ignores_comment_and_string_substrings() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let ghost_path = dir.path().join("ghost.ts");
    let lazy_path = dir.path().join("lazy.ts");
    fs::write(&ghost_path, "export const ghost = true;").expect("write ghost chunk");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const comment = \"import('./ghost.ts')\";\n/* import('./ghost.ts') */\nconst lazy = import('./lazy.ts');\n",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.ts");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_directory_index_chunks() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.ts"), "export const lazy = true;").expect("write lazy index");
    fs::write(&source_path, "const lazy = import('./lazy');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.ts")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_template_literal_dynamic_import_chunks() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let lazy_path = dir.path().join("lazy.ts");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; const lazy = import(`./${name}`);",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.ts");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_literal_dynamic_import_chunks_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let lazy_path = dir.path().join("lazy.js");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(&source_path, "const lazy = import('./lazy.js');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.js");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_directory_index_chunks_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.js"), "export const lazy = true;").expect("write lazy index");
    fs::write(&source_path, "const lazy = import('./lazy');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.js")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_directory_index_chunks_in_jsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.jsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.jsx"), "export const lazy = true;").expect("write lazy index");
    fs::write(&source_path, "const lazy = import('./lazy');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.jsx")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_directory_index_chunks_in_tsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.tsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.tsx"), "export const lazy = true;").expect("write lazy index");
    fs::write(&source_path, "const lazy = import('./lazy');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.tsx")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_parenthesized_dynamic_import_targets_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.js"), "export const lazy = true;").expect("write lazy index");
    fs::write(
        &source_path,
        "const name = 'lazy'; const root = './'; const lazy = import((root + name));",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.js")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_parenthesized_dynamic_import_targets_in_jsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.jsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.jsx"), "export const lazy = true;").expect("write lazy index");
    fs::write(
        &source_path,
        "const name = 'lazy'; const root = './'; const lazy = import((root + name));",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.jsx")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_parenthesized_dynamic_import_targets_in_tsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.tsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.tsx"), "export const lazy = true;").expect("write lazy index");
    fs::write(
        &source_path,
        "const name = 'lazy'; const root = './'; const lazy = import((root + name));",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.tsx")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_ignores_comment_and_string_substrings_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let ghost_path = dir.path().join("ghost.js");
    let lazy_path = dir.path().join("lazy.js");
    fs::write(&ghost_path, "export const ghost = true;").expect("write ghost chunk");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const comment = \"import('./ghost.js')\";\n/* import('./ghost.js') */\nconst lazy = import('./lazy.js');\n",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.js");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_ignores_comment_and_string_substrings_in_jsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.jsx");
    let ghost_path = dir.path().join("ghost.jsx");
    let lazy_path = dir.path().join("lazy.jsx");
    fs::write(&ghost_path, "export const ghost = true;").expect("write ghost chunk");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const comment = \"import('./ghost.jsx')\";\n/* import('./ghost.jsx') */\nconst lazy = import('./lazy.jsx');\n",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.jsx");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_ignores_comment_and_string_substrings_in_tsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.tsx");
    let ghost_path = dir.path().join("ghost.tsx");
    let lazy_path = dir.path().join("lazy.tsx");
    fs::write(&ghost_path, "export const ghost = true;").expect("write ghost chunk");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const comment = \"import('./ghost.tsx')\";\n/* import('./ghost.tsx') */\nconst lazy = import('./lazy.tsx');\n",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.tsx");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_parenthesized_dynamic_import_targets_in_ts_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.ts"), "export const lazy = true;").expect("write lazy index");
    fs::write(
        &source_path,
        "const name = 'lazy'; const root = './'; const lazy = import((root + name));",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.ts")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_nullish_wrapped_dynamic_import_targets_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let lazy_path = dir.path().join("lazy.js");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(&source_path, "const lazy = import((null ?? './lazy.js'));").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.js");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_logical_wrapped_dynamic_import_targets_in_ts_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let lazy_path = dir.path().join("lazy.ts");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(&source_path, "const lazy = import((false || './lazy.ts'));").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.ts");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}
