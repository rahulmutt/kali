use super::*;

#[test]
fn indexed_db_stub_persists_values() {
    let db = IndexedDB::open("browser-cache");
    assert_eq!(db.name(), "browser-cache");

    db.put("sessions", "alpha", Value::String("1".to_string()));
    db.put("sessions", "beta", Value::String("2".to_string()));
    assert_eq!(db.store_names(), vec!["sessions".to_string()]);
    assert_eq!(
        db.get("sessions", "alpha"),
        Some(Value::String("1".to_string()))
    );
    assert_eq!(
        db.delete("sessions", "alpha"),
        Some(Value::String("1".to_string()))
    );
    assert_eq!(db.get("sessions", "alpha"), None);

    db.clear_store("sessions");
    assert!(db.store_names().is_empty());

    let alias = IndexedDb::open("browser-cache-alias");
    assert_eq!(alias.name(), "browser-cache-alias");
}

#[test]
fn indexed_db_stub_exposes_deterministic_snapshots() {
    let db = IndexedDB::open("browser-cache");
    db.put("sessions", "beta", Value::String("2".to_string()));
    db.put("sessions", "alpha", Value::String("1".to_string()));
    db.put("settings", "theme", Value::String("dark".to_string()));

    let snapshot = db.snapshot();
    assert_eq!(
        snapshot.keys().cloned().collect::<Vec<_>>(),
        vec!["sessions".to_string(), "settings".to_string()]
    );
    assert_eq!(
        snapshot["sessions"].keys().cloned().collect::<Vec<_>>(),
        vec!["alpha".to_string(), "beta".to_string()]
    );
    assert_eq!(
        snapshot["settings"].keys().cloned().collect::<Vec<_>>(),
        vec!["theme".to_string()]
    );
    assert_eq!(
        db.snapshot_value(),
        serde_json::json!({
            "sessions": {"alpha": "1", "beta": "2"},
            "settings": {"theme": "dark"}
        })
    );
    assert_eq!(db.snapshot_json_value(), db.snapshot_value());
    assert_eq!(db.snapshot_object_value(), db.snapshot_value());

    let alias = IndexedDb::open("browser-cache-alias");
    assert!(alias.snapshot().is_empty());
    assert_eq!(alias.snapshot_value(), serde_json::json!({}));
}
