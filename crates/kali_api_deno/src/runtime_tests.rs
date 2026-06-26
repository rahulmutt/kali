use crate::*;
use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn runtime_projection_bundles_baseline_context() {
    let mut projection = DenoRuntimeProjection::from_host_context(
        vec![String::from("kali"), String::from("run")],
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))]),
        "/workspace/project",
        DenoPermissions::open(),
    );

    assert_eq!(
        projection.args().as_slice(),
        &[String::from("kali"), String::from("run")]
    );
    assert_eq!(projection.env().get("HOME"), Some("/tmp/home"));
    assert!(projection.env_has("HOME"));
    assert!(projection.has("HOME"));
    assert!(!projection.env_has("MISSING"));
    assert!(!projection.has("MISSING"));
    assert_eq!(
        projection.env_snapshot().get("HOME"),
        Some(&String::from("/tmp/home"))
    );
    assert_eq!(
        projection.env.snapshot().get("HOME"),
        Some(&String::from("/tmp/home"))
    );
    assert_eq!(
        projection.env_to_object().get("HOME"),
        Some(&String::from("/tmp/home"))
    );
    assert_eq!(
        projection.env_snapshot_object_value(),
        projection.env_snapshot()
    );
    assert_eq!(
        projection.snapshot_object_value(),
        projection.env_snapshot()
    );
    projection.env_mut().set("HOME", "/workspace/home");
    projection.env_mut().set("EDITOR", "nano");
    assert_eq!(projection.env().get("HOME"), Some("/workspace/home"));
    assert_eq!(
        projection.env_snapshot().get("HOME"),
        Some(&String::from("/workspace/home"))
    );
    assert_eq!(
        projection.env.snapshot().get("HOME"),
        Some(&String::from("/workspace/home"))
    );
    assert_eq!(
        projection.env_snapshot().get("EDITOR"),
        Some(&String::from("nano"))
    );
    let json_snapshot = projection.env_snapshot_value();
    let json_snapshot = json_snapshot.as_object().expect("json object");
    assert_eq!(
        projection.env_snapshot_json_value(),
        serde_json::json!({ "HOME": "/workspace/home", "EDITOR": "nano" })
    );
    assert_eq!(
        projection.snapshot_json_value(),
        serde_json::json!({ "HOME": "/workspace/home", "EDITOR": "nano" })
    );
    assert_eq!(projection.snapshot_value(), projection.env_snapshot_value());
    assert_eq!(
        projection.env_to_json_value(),
        serde_json::json!({ "HOME": "/workspace/home", "EDITOR": "nano" })
    );
    assert_eq!(
        json_snapshot.get("HOME"),
        Some(&serde_json::Value::String(String::from("/workspace/home")))
    );
    assert_eq!(
        json_snapshot.get("EDITOR"),
        Some(&serde_json::Value::String(String::from("nano")))
    );
    assert_eq!(projection.fs().cwd(), Path::new("/workspace/project"));
    assert_eq!(projection.pid(), std::process::id());
    assert_eq!(projection.exit_code(), None);

    projection.chdir("/workspace/project/../workspace/./next");
    assert_eq!(
        projection.fs().cwd(),
        Path::new("/workspace/workspace/next")
    );

    projection.exit(7);
    assert_eq!(projection.exit_code(), Some(7));
    assert_eq!(
        projection.permissions().query(DenoPermissionKind::Read),
        Ok(DenoPermissionStatus::Granted)
    );
}

#[test]
fn runtime_projection_new_defaults_to_open_permissions_and_empty_views() {
    let projection = DenoRuntimeProjection::new("/workspace/project");

    assert!(projection.args().as_slice().is_empty());
    assert!(projection.env().to_object().is_empty());
    assert!(projection.env.snapshot().is_empty());
    assert!(projection.env_to_object().is_empty());
    assert!(!projection.has("MISSING"));
    assert!(projection.env_snapshot_object_value().is_empty());
    assert_eq!(
        projection.permissions().query(DenoPermissionKind::Read),
        Ok(DenoPermissionStatus::Granted)
    );
    assert_eq!(
        projection.permissions().query(DenoPermissionKind::Write),
        Ok(DenoPermissionStatus::Granted)
    );
    assert_eq!(
        projection.permissions().query(DenoPermissionKind::Net),
        Ok(DenoPermissionStatus::Granted)
    );
    assert_eq!(
        projection.permissions().query(DenoPermissionKind::Env),
        Ok(DenoPermissionStatus::Granted)
    );
}
