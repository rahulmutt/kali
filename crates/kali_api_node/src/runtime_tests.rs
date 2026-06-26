use crate::*;
use std::{collections::BTreeMap, path::PathBuf};
use tempfile::tempdir;

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
