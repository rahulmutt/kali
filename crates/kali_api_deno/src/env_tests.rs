use crate::*;
use std::collections::BTreeMap;

#[test]
fn env_view_is_deterministic_and_mutable() {
    let mut env = DenoEnv::new(BTreeMap::from([
        (String::from("HOME"), String::from("/tmp/home")),
        (String::from("TERM"), String::from("xterm-256color")),
    ]));

    assert_eq!(env.get("HOME"), Some("/tmp/home"));
    assert!(env.has("HOME"));
    assert!(!env.has("MISSING"));
    assert_eq!(
        env.set("HOME", "/workspace/home"),
        Some(String::from("/tmp/home"))
    );
    assert_eq!(env.set("EDITOR", "nano"), None);
    assert_eq!(env.set("TEMP", "tmp"), None);
    assert_eq!(env.get("HOME"), Some("/workspace/home"));
    assert_eq!(env.remove("TERM"), Some(String::from("xterm-256color")));
    assert_eq!(env.delete("TEMP"), Some(String::from("tmp")));
    assert_eq!(env.get("TERM"), None);
    assert_eq!(env.get("TEMP"), None);
    assert_eq!(
        env.to_object().get("HOME"),
        Some(&String::from("/workspace/home"))
    );
    assert_eq!(
        env.env_to_object().get("HOME"),
        Some(&String::from("/workspace/home"))
    );
    assert_eq!(
        env.env_snapshot().get("HOME"),
        Some(&String::from("/workspace/home"))
    );
    assert_eq!(
        env.snapshot().get("HOME"),
        Some(&String::from("/workspace/home"))
    );
    assert_eq!(
        env.env_snapshot_object_value().get("HOME"),
        Some(&String::from("/workspace/home"))
    );
    assert_eq!(
        env.snapshot_object_value().get("HOME"),
        Some(&String::from("/workspace/home"))
    );
    assert_eq!(
        env.env_snapshot_value()
            .as_object()
            .expect("json object")
            .get("HOME"),
        Some(&serde_json::Value::String(String::from("/workspace/home")))
    );
    assert_eq!(
        env.env_to_json_value()
            .as_object()
            .expect("json object")
            .get("HOME"),
        Some(&serde_json::Value::String(String::from("/workspace/home")))
    );
    assert_eq!(
        env.env_snapshot_json_value()
            .as_object()
            .expect("json object")
            .get("HOME"),
        Some(&serde_json::Value::String(String::from("/workspace/home")))
    );
    assert_eq!(env.snapshot_value(), env.to_json_value());
    assert_eq!(env.to_object().get("EDITOR"), Some(&String::from("nano")));
    assert_eq!(env.iter().count(), 2);
}

#[test]
fn env_view_snapshot_is_sorted_and_detached_from_later_mutations() {
    let mut env = DenoEnv::new(BTreeMap::from([
        (String::from("BETA"), String::from("2")),
        (String::from("ALPHA"), String::from("1")),
    ]));

    let snapshot = env.to_object();
    assert_eq!(
        snapshot.keys().cloned().collect::<Vec<_>>(),
        vec!["ALPHA", "BETA"]
    );
    assert_eq!(snapshot.get("ALPHA"), Some(&String::from("1")));
    assert_eq!(snapshot.get("BETA"), Some(&String::from("2")));
    assert_eq!(env.env_snapshot(), snapshot);
    assert_eq!(env.snapshot(), snapshot);
    assert_eq!(env.env_to_object(), snapshot);
    assert_eq!(env.env_snapshot_object_value(), snapshot);
    assert_eq!(env.snapshot_object_value(), snapshot);

    let json_snapshot = env.to_json_value();
    assert_eq!(env.env_to_json_value(), json_snapshot);
    assert_eq!(env.env_snapshot_value(), json_snapshot);
    assert_eq!(env.env_snapshot_json_value(), json_snapshot);
    assert_eq!(env.snapshot_json_value(), json_snapshot);
    assert_eq!(env.snapshot_value(), env.to_json_value());
    let json_snapshot = json_snapshot.as_object().expect("json object");
    assert_eq!(
        json_snapshot.get("ALPHA"),
        Some(&serde_json::Value::String(String::from("1")))
    );
    assert_eq!(
        json_snapshot.get("BETA"),
        Some(&serde_json::Value::String(String::from("2")))
    );

    env.set("ALPHA", "updated");
    env.set("GAMMA", "3");
    env.remove("BETA");

    assert_eq!(snapshot.get("ALPHA"), Some(&String::from("1")));
    assert_eq!(snapshot.get("BETA"), Some(&String::from("2")));
    assert!(!snapshot.contains_key("GAMMA"));
    assert_eq!(
        json_snapshot.get("ALPHA"),
        Some(&serde_json::Value::String(String::from("1")))
    );
    assert_eq!(
        json_snapshot.get("BETA"),
        Some(&serde_json::Value::String(String::from("2")))
    );
    assert!(!json_snapshot.contains_key("GAMMA"));
}
