use crate::*;
use std::{collections::BTreeMap, path::Path};

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
fn default_process_context_uses_node_as_argv0() {
    let process = NodeProcess::default();

    assert_eq!(process.argv0(), "node");
    assert!(process.argv().is_empty());
}
