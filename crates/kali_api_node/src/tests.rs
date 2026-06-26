use super::*;
use tempfile::tempdir;

#[test]
fn process_context_tracks_env_and_output() {
    let mut env = BTreeMap::new();
    env.insert("HOME".to_string(), "/tmp/home".to_string());
    let mut process = NodeProcess::with_host_context(
        vec!["node".into(), "script.js".into()],
        env,
        "/workspace/project",
    );

    assert_eq!(process.argv(), &["node", "script.js"]);
    assert_eq!(process.argv0(), "node");
    assert_eq!(process.argv_len(), 2);
    assert_eq!(process.argv_at(1), Some("script.js"));
    assert_eq!(process.cwd(), Path::new("/workspace/project"));
    assert_eq!(process.pid(), std::process::id());
    assert_eq!(process.env_get("HOME"), Some("/tmp/home"));
    assert!(process.env_has("HOME"));
    assert!(process.has("HOME"));
    assert!(!process.env_has("MISSING"));
    assert!(!process.has("MISSING"));
    assert_eq!(
        process.env_snapshot(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.snapshot(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.env_to_object(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.env_snapshot_object_value(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.snapshot_object_value(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.env_snapshot_value(),
        serde_json::json!({ "HOME": "/tmp/home" })
    );
    assert_eq!(
        process.env_snapshot_json_value(),
        serde_json::json!({ "HOME": "/tmp/home" })
    );
    assert_eq!(
        process.snapshot_json_value(),
        serde_json::json!({ "HOME": "/tmp/home" })
    );
    assert_eq!(process.snapshot_value(), process.env_snapshot_value());
    assert_eq!(
        process.env_to_json_value(),
        serde_json::json!({ "HOME": "/tmp/home" })
    );
    assert_eq!(process.env_set("EDITOR", "nano"), None);
    assert_eq!(process.env_remove("HOME"), Some(String::from("/tmp/home")));
    assert_eq!(process.env_delete("EDITOR"), Some(String::from("nano")));
    assert_eq!(process.env_get("HOME"), None);
    assert_eq!(process.env_get("EDITOR"), None);
    assert_eq!(process.env_snapshot(), BTreeMap::new());
    assert_eq!(process.snapshot(), BTreeMap::new());
    assert_eq!(process.env_to_object(), BTreeMap::new());
    assert_eq!(process.env_snapshot_value(), serde_json::json!({}));
    assert_eq!(process.env_to_json_value(), serde_json::json!({}));

    process.write_stdout("hello");
    process.write_stderr("oops");
    process.set_exit_code(7);

    assert_eq!(process.stdout(), "hello");
    assert_eq!(process.stderr(), "oops");
    assert_eq!(process.exit_code(), Some(7));

    process.exit(3);
    assert_eq!(process.exit_code(), Some(3));
}

#[test]
fn runtime_projection_preserves_host_argv0_projection() {
    let projection = NodeRuntimeProjection::from_host_context(
        vec!["node".into(), "script.js".into()],
        BTreeMap::new(),
        "/workspace/project",
    );

    assert_eq!(projection.process().argv0(), "node");
}

#[test]
fn runtime_projection_exposes_deterministic_env_snapshot() {
    let mut env = BTreeMap::new();
    env.insert("HOME".to_string(), "/tmp/home".to_string());
    env.insert("EDITOR".to_string(), "nano".to_string());

    let mut projection = NodeRuntimeProjection::from_host_context(
        vec!["node".into(), "script.js".into()],
        env,
        "/workspace/project",
    );

    assert_eq!(
        projection.env_snapshot(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert_eq!(
        projection.snapshot(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert_eq!(
        projection.env_to_object(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert_eq!(
        projection.env_snapshot_object_value(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert_eq!(
        projection.snapshot_object_value(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert!(projection.env_has("HOME"));
    assert!(projection.has("HOME"));
    assert!(!projection.env_has("MISSING"));
    assert!(!projection.has("MISSING"));
    assert_eq!(
        projection.env_snapshot_value(),
        serde_json::json!({ "EDITOR": "nano", "HOME": "/tmp/home" })
    );
    assert_eq!(
        projection.env_snapshot_json_value(),
        serde_json::json!({ "EDITOR": "nano", "HOME": "/tmp/home" })
    );
    assert_eq!(
        projection.snapshot_json_value(),
        serde_json::json!({ "EDITOR": "nano", "HOME": "/tmp/home" })
    );
    assert_eq!(projection.snapshot_value(), projection.env_snapshot_value());
    assert_eq!(
        projection.env_delete("HOME"),
        Some(String::from("/tmp/home"))
    );
    assert!(!projection.env_has("HOME"));
    assert_eq!(
        projection.env_to_json_value(),
        serde_json::json!({ "EDITOR": "nano" })
    );

    projection.chdir("./nested/../other");
    assert_eq!(
        projection.env_snapshot(),
        BTreeMap::from([(String::from("EDITOR"), String::from("nano"))])
    );
    assert_eq!(
        projection.snapshot(),
        BTreeMap::from([(String::from("EDITOR"), String::from("nano"))])
    );
    assert_eq!(
        projection.env_snapshot_value(),
        serde_json::json!({ "EDITOR": "nano" })
    );
}

#[test]
fn default_process_context_uses_node_as_argv0() {
    let process = NodeProcess::default();

    assert_eq!(process.argv0(), "node");
    assert!(process.argv().is_empty());
}

#[test]
fn path_helpers_are_lexical_and_deterministic() {
    assert_eq!(
        normalize_path("./foo/../bar//baz"),
        PathBuf::from("bar/baz")
    );
    assert_eq!(
        join_path("/tmp", "project/src"),
        PathBuf::from("/tmp/project/src")
    );
    assert_eq!(
        resolve_path("/tmp/project", "../lib/index.js"),
        PathBuf::from("/tmp/lib/index.js")
    );
    assert_eq!(
        relative_path("/tmp/project/src", "/tmp/project/lib/index.js"),
        PathBuf::from("../lib/index.js")
    );
    assert_eq!(
        dirname("/tmp/project/src/main.ts"),
        PathBuf::from("/tmp/project/src")
    );
    assert_eq!(basename("/tmp/project/src/main.ts"), "main.ts");
    assert_eq!(extname("/tmp/project/src/main.ts"), ".ts");
}

#[test]
fn fs_helpers_round_trip_files_and_directories() {
    let dir = tempdir().expect("tempdir");
    let fs = NodeFs::new(dir.path());

    fs.mkdir("nested", false).expect("mkdir");
    fs.write_text_file("nested/alpha.txt", "alpha")
        .expect("write text");
    fs.write_file("nested/beta.bin", [0, 1, 2])
        .expect("write file");
    fs.rename("nested/alpha.txt", "nested/renamed.txt")
        .expect("rename file");

    assert_eq!(
        fs.read_file("nested/beta.bin").expect("read file"),
        vec![0, 1, 2]
    );
    assert_eq!(
        fs.read_text_file("nested/renamed.txt").expect("read text"),
        "alpha"
    );
    assert_eq!(
        fs.readdir("nested").expect("readdir"),
        vec!["beta.bin".to_string(), "renamed.txt".to_string()]
    );

    let stat = fs.stat("nested/renamed.txt").expect("stat");
    assert!(stat.is_file());
    assert!(!stat.is_dir());
    assert!(!stat.is_symlink());
    assert_eq!(stat.len(), 5);

    let lstat = fs.lstat("nested/renamed.txt").expect("lstat");
    assert!(lstat.is_file());
    assert!(!lstat.is_dir());
    assert!(!lstat.is_symlink());

    fs.remove("nested/beta.bin", false).expect("remove file");
    fs.remove("nested", true).expect("remove dir");
    assert!(!fs.exists("nested"));
}

#[test]
fn fs_promises_helpers_match_sync_helpers() {
    let dir = tempdir().expect("tempdir");
    let fs = NodeFsPromises::new(dir.path());

    fs.mkdir("nested", false).expect("mkdir");
    fs.write_text_file("nested/alpha.txt", "alpha")
        .expect("write text");
    fs.write_file("nested/beta.bin", [0, 1, 2])
        .expect("write file");
    fs.rename("nested/alpha.txt", "nested/renamed.txt")
        .expect("rename file");

    assert_eq!(
        fs.read_file("nested/beta.bin").expect("read file"),
        vec![0, 1, 2]
    );
    assert_eq!(
        fs.read_text_file("nested/renamed.txt").expect("read text"),
        "alpha"
    );
    assert_eq!(
        fs.readdir("nested").expect("readdir"),
        vec!["beta.bin".to_string(), "renamed.txt".to_string()]
    );

    let stat = fs.stat("nested/renamed.txt").expect("stat");
    assert!(stat.is_file());
    assert!(!stat.is_dir());
    assert!(!stat.is_symlink());
    assert_eq!(stat.len(), 5);

    let lstat = fs.lstat("nested/renamed.txt").expect("lstat");
    assert!(lstat.is_file());
    assert!(!lstat.is_dir());
    assert!(!lstat.is_symlink());

    fs.remove("nested/beta.bin", false).expect("remove file");
    fs.remove("nested", true).expect("remove dir");
    assert!(!fs.exists("nested"));
}

#[test]
fn stream_helpers_concatenate_bytes() {
    let bytes = NodeStream::concat(b"hello ", b"world");
    assert_eq!(bytes, b"hello world");
    assert_eq!(NodeStream::from_utf8("abc"), b"abc");
    assert_eq!(NodeStream::from_bytes(vec![1, 2, 3]), vec![1, 2, 3]);
    assert_eq!(NodeStream::to_utf8(b"kali").expect("utf8"), "kali");
}


#[test]
fn runtime_projection_bundles_common_node_surfaces() {
    let dir = tempdir().expect("tempdir");
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).expect("create nested dir");

    let mut projection = NodeRuntimeProjection::from_host_context(
        vec!["node".into(), "script.js".into()],
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))]),
        dir.path(),
    );

    assert_eq!(
        projection.process().argv(),
        &vec!["node".to_string(), "script.js".to_string()][..]
    );
    assert_eq!(projection.process().argv_len(), 2);
    assert_eq!(projection.process().env_get("HOME"), Some("/tmp/home"));
    assert_eq!(projection.fs().cwd(), dir.path());
    assert!(!projection.os().platform().is_empty());
    assert_eq!(projection.url(), NodeUrl);
    assert_eq!(projection.util(), NodeUtil);
    assert_eq!(projection.assert(), NodeAssert);
    assert_eq!(projection.child_process(), NodeChildProcess);

    projection.chdir("nested");
    assert_eq!(projection.process().cwd(), nested.as_path());
    assert_eq!(projection.fs().cwd(), nested.as_path());
    projection
        .fs()
        .write_text_file("relative.txt", "ok")
        .expect("write via chdir");
    assert_eq!(
        std::fs::read_to_string(nested.join("relative.txt")).expect("read via chdir"),
        "ok"
    );

    projection.process_mut().write_stdout("ok");
    assert_eq!(projection.process().stdout(), "ok");

    assert_eq!(
        NodePath::dirname("/tmp/project/src/main.ts"),
        PathBuf::from("/tmp/project/src")
    );
    assert_eq!(
        NodePath::relative("/tmp/project/src", "/tmp/project/lib/index.js"),
        PathBuf::from("../lib/index.js")
    );
    assert_eq!(NodePath::basename("/tmp/project/src/main.ts"), "main.ts");
    assert_eq!(NodePath::extname("/tmp/project/src/main.ts"), ".ts");
    assert_eq!(
        NodeCrypto::create_hash("sha384", "hello")
            .expect("hash")
            .len(),
        96
    );
}
