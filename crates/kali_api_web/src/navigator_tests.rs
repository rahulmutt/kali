use crate::*;
use serde_json::Value;

#[test]
fn navigator_baseline_exposes_stable_metadata() {
    let navigator = navigator();
    assert_eq!(navigator.user_agent(), "Kali/1.0 (Web)");
    assert_eq!(navigator.language(), "en-US");
    assert_eq!(navigator.languages(), &[String::from("en-US")]);
    assert!(navigator.on_line());
}

#[test]
fn navigator_snapshot_helpers_expose_deterministic_object_and_json_views() {
    let navigator = navigator();
    let snapshot = navigator.snapshot();
    assert_eq!(
        snapshot.get("userAgent"),
        Some(&Value::String(String::from("Kali/1.0 (Web)")))
    );
    assert_eq!(
        snapshot.get("language"),
        Some(&Value::String(String::from("en-US")))
    );
    assert_eq!(
        snapshot.get("languages"),
        Some(&Value::Array(vec![Value::String(String::from("en-US"))]))
    );
    assert_eq!(snapshot.get("online"), Some(&Value::Bool(true)));
    assert_eq!(navigator.snapshot_object_value(), snapshot);

    let json_snapshot = navigator.snapshot_value();
    assert_eq!(navigator.snapshot_json_value(), json_snapshot);
    assert_eq!(json_snapshot["userAgent"], "Kali/1.0 (Web)");
    assert_eq!(json_snapshot["language"], "en-US");
    assert_eq!(
        json_snapshot["languages"],
        Value::Array(vec![Value::String(String::from("en-US"))])
    );
    assert_eq!(json_snapshot["online"], true);
}
