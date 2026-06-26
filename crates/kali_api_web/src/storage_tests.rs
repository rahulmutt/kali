use crate::*;
use serde_json::Value;

#[test]
fn storage_round_trips_values_and_stays_ordered() {
    let storage = Storage::new();
    storage.set_item("alpha", "1");
    storage.set_item("beta", "2");

    let expected_snapshot = storage.snapshot();
    let expected_json = Value::Object(
        expected_snapshot
            .iter()
            .map(|(key, value)| (key.clone(), Value::String(value.clone())))
            .collect(),
    );

    assert_eq!(storage.length(), 2);
    assert_eq!(storage.get_item("alpha").as_deref(), Some("1"));
    assert_eq!(storage.key(0).as_deref(), Some("alpha"));
    assert_eq!(storage.key(1).as_deref(), Some("beta"));
    assert_eq!(storage.snapshot_object_value(), expected_snapshot);
    assert_eq!(storage.snapshot_value(), expected_json);
    assert_eq!(storage.snapshot_json_value(), expected_json);
    assert_eq!(storage.remove_item("alpha").as_deref(), Some("1"));
    assert_eq!(storage.length(), 1);
    storage.clear();
    assert_eq!(storage.length(), 0);
    assert!(storage.snapshot().is_empty());
}

#[test]
fn shared_browser_storage_buckets_remain_isolated() {
    let local = local_storage();
    let session = session_storage();
    local.clear();
    session.clear();

    local.set_item("mode", "local");
    session.set_item("mode", "session");

    assert_eq!(local.get_item("mode").as_deref(), Some("local"));
    assert_eq!(session.get_item("mode").as_deref(), Some("session"));
    assert_ne!(local.snapshot(), session.snapshot());

    local.clear();
    session.clear();
}
