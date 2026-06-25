use super::*;
use kali_common::bytewise_shared_memory_is_lock_free;

#[test]
fn worker_stub_records_posted_messages() {
    let worker = Worker::new(" https://example.com/worker.js \n").expect("worker url");
    assert_eq!(
        worker.script_url().as_str(),
        "https://example.com/worker.js"
    );
    assert!(!worker.is_terminated());

    worker.post_message(Value::String("ping".to_string()));
    assert_eq!(
        worker.posted_messages(),
        vec![Value::String("ping".to_string())]
    );

    worker.terminate();
    assert!(worker.is_terminated());
    worker.post_message(Value::String("ignored".to_string()));
    assert_eq!(
        worker.posted_messages(),
        vec![Value::String("ping".to_string())]
    );
}

#[test]
fn worker_stub_records_shared_buffers_with_shared_backing() {
    let worker = Worker::new("https://example.com/worker.js").expect("worker url");
    let buffer = SharedArrayBuffer::from_bytes([1, 2, 3]);

    worker.post_shared_buffer(buffer.clone());
    let posted = worker.posted_shared_buffers();
    assert_eq!(posted.len(), 1);
    assert_eq!(posted[0].snapshot(), vec![1, 2, 3]);

    buffer.store(1, 9);
    assert_eq!(worker.posted_shared_buffers()[0].snapshot(), vec![1, 9, 3]);
}

#[test]
fn worker_stub_trims_surrounding_whitespace_from_script_urls() {
    let worker = Worker::new(" \thttps://example.com/worker.js \n").expect("worker url");

    assert_eq!(
        worker.script_url().as_str(),
        "https://example.com/worker.js"
    );
}

#[test]
fn worker_stub_ignores_shared_buffer_posts_after_termination() {
    let worker = Worker::new("https://example.com/worker.js").expect("worker url");
    let buffer = SharedArrayBuffer::from_bytes([4, 5, 6]);

    worker.post_shared_buffer(buffer.clone());
    worker.terminate();
    worker.post_shared_buffer(SharedArrayBuffer::from_bytes([7, 8, 9]));

    assert_eq!(worker.posted_shared_buffers(), vec![buffer]);
}

#[test]
fn worker_stub_preserves_interleaved_post_order() {
    let worker = Worker::new("https://example.com/worker.js").expect("worker url");
    let buffer = SharedArrayBuffer::from_bytes([7, 8, 9]);

    worker.post_message(Value::String("alpha".to_string()));
    worker.post_shared_buffer(buffer.clone());
    worker.post_message(Value::String("omega".to_string()));

    assert_eq!(
        worker.posted_items(),
        vec![
            PostedItem::Message(Value::String("alpha".to_string())),
            PostedItem::SharedBuffer(buffer),
            PostedItem::Message(Value::String("omega".to_string())),
        ]
    );
}

#[test]
fn thread_runtime_topology_assigns_one_instance_per_worker() {
    let mut topology = ThreadRuntimeTopology::new();
    let first = topology
        .spawn_worker("https://example.com/worker-a.js")
        .expect("first worker");
    let second = topology
        .spawn_worker("https://example.com/worker-b.js")
        .expect("second worker");

    assert_ne!(first, second);
    assert_eq!(topology.total_instances(), 2);
    assert_eq!(topology.instance_ids(), vec![first, second]);
    assert!(topology.is_live(first));
    assert!(topology.is_live(second));

    assert!(topology.post_message(first, Value::String("ping".to_string())));
    assert!(topology.post_shared_buffer(second, SharedArrayBuffer::from_bytes([7, 8, 9])));
    assert!(topology.terminate(first));
    assert!(!topology.is_live(first));
    assert!(topology.is_live(second));
}

#[test]
fn thread_runtime_topology_keeps_monotonic_instance_ids_after_termination() {
    let mut topology = ThreadRuntimeTopology::new();
    let first = topology
        .spawn_worker("https://example.com/worker-a.js")
        .expect("first worker");
    let second = topology
        .spawn_worker("https://example.com/worker-b.js")
        .expect("second worker");

    assert!(topology.terminate(first));

    let third = topology
        .spawn_worker("https://example.com/worker-c.js")
        .expect("third worker");

    assert_eq!(first, 0);
    assert_eq!(second, 1);
    assert_eq!(third, 2);
    assert_eq!(topology.instance_ids(), vec![first, second, third]);

    let report = topology.snapshot();
    assert_eq!(report.total_instances, 3);
    assert_eq!(report.terminated_instances, 1);
    assert_eq!(report.live_instances.len(), 2);
    assert_eq!(
        report
            .live_instances
            .iter()
            .map(|snapshot| snapshot.instance_id)
            .collect::<Vec<_>>(),
        vec![second, third]
    );
    assert!(report
        .live_instances
        .iter()
        .all(|snapshot| !snapshot.was_terminated));
}

#[test]
fn thread_runtime_topology_snapshot_reports_live_instances_deterministically() {
    let mut topology = ThreadRuntimeTopology::new();
    let live = topology
        .spawn_worker("https://example.com/live-worker.js")
        .expect("live worker");
    let terminated = topology
        .spawn_worker("https://example.com/terminated-worker.js")
        .expect("terminated worker");

    topology.post_message(live, Value::String("hello".to_string()));
    topology.post_shared_buffer(live, SharedArrayBuffer::from_bytes([1, 2, 3]));
    topology.post_message(terminated, Value::String("goodbye".to_string()));
    topology.terminate(terminated);

    let report = topology.snapshot();
    assert_eq!(report.total_instances, 2);
    assert_eq!(report.terminated_instances, 1);
    assert_eq!(report.live_instances.len(), 1);
    assert_eq!(
        report.total_instances,
        report.terminated_instances + report.live_instances.len()
    );

    let snapshot = &report.live_instances[0];
    assert_eq!(snapshot.instance_id, live);
    assert_eq!(snapshot.script_url, "https://example.com/live-worker.js");
    assert_eq!(
        snapshot.posted_messages,
        vec![Value::String("hello".to_string())]
    );
    assert_eq!(snapshot.posted_shared_buffers, vec![vec![1, 2, 3]]);
    assert!(!snapshot.was_terminated);
    assert_eq!(
        snapshot.snapshot_value(),
        serde_json::json!({
            "instanceId": live,
            "scriptUrl": "https://example.com/live-worker.js",
            "postedMessages": ["hello"],
            "postedSharedBuffers": [[1, 2, 3]],
            "wasTerminated": false
        })
    );
    assert_eq!(snapshot.snapshot_json_value(), snapshot.snapshot_value());
    assert_eq!(snapshot.snapshot_object_value(), snapshot.snapshot_value());
    assert_eq!(snapshot.snapshot(), snapshot.clone());
    assert_eq!(snapshot.thread_topology_snapshot(), snapshot.clone());
    assert_eq!(
        snapshot.thread_topology_snapshot_object_value(),
        snapshot.snapshot_value()
    );
    assert_eq!(
        snapshot.thread_topology_snapshot_value(),
        snapshot.snapshot_value()
    );
    assert_eq!(
        snapshot.thread_topology_snapshot_json_value(),
        snapshot.snapshot_value()
    );

    assert_eq!(report.thread_topology_snapshot(), report.clone());
    assert_eq!(
        report.snapshot_value(),
        serde_json::json!({
            "totalInstances": 2,
            "terminatedInstances": 1,
            "liveInstances": [{
                "instanceId": live,
                "scriptUrl": "https://example.com/live-worker.js",
                "postedMessages": ["hello"],
                "postedSharedBuffers": [[1, 2, 3]],
                "wasTerminated": false
            }]
        })
    );
    assert_eq!(report.snapshot_json_value(), report.snapshot_value());
    assert_eq!(
        topology.thread_topology_snapshot().snapshot_json_value(),
        report.snapshot_value()
    );
    assert_eq!(report.snapshot_object_value(), report.snapshot_value());
    assert_eq!(report.snapshot(), report.clone());
    assert_eq!(
        report.thread_topology_snapshot_object_value(),
        report.snapshot_value()
    );
    assert_eq!(
        report.thread_topology_snapshot_value(),
        report.snapshot_value()
    );
    assert_eq!(
        report.thread_topology_snapshot_json_value(),
        report.snapshot_value()
    );
    assert_eq!(
        topology.thread_topology_snapshot().snapshot_value(),
        report.snapshot_value()
    );
    assert_eq!(topology.snapshot_value(), report.snapshot_value());
    assert_eq!(topology.snapshot_json_value(), report.snapshot_value());
    assert_eq!(topology.snapshot_object_value(), report.snapshot_value());
    assert_eq!(
        topology.thread_topology_snapshot_object_value(),
        report.snapshot_value()
    );
    assert_eq!(
        topology.thread_topology_snapshot_value(),
        report.snapshot_value()
    );
    assert_eq!(
        topology.thread_topology_snapshot_json_value(),
        report.snapshot_value()
    );
    assert_eq!(
        topology
            .thread_topology_snapshot()
            .thread_topology_snapshot(),
        topology.thread_topology_snapshot()
    );
}

#[test]
fn thread_runtime_topology_shutdown_reports_live_instances_deterministically() {
    let mut topology = ThreadRuntimeTopology::new();
    let live = topology
        .spawn_worker("https://example.com/live-worker.js")
        .expect("live worker");
    let terminated = topology
        .spawn_worker("https://example.com/terminated-worker.js")
        .expect("terminated worker");

    topology.post_message(live, Value::String("hello".to_string()));
    topology.post_shared_buffer(live, SharedArrayBuffer::from_bytes([1, 2, 3]));
    topology.post_message(terminated, Value::String("goodbye".to_string()));
    topology.terminate(terminated);

    let report = topology.shutdown();
    assert_eq!(report.total_instances, 2);
    assert_eq!(report.terminated_instances, 1);
    assert_eq!(report.live_instances.len(), 1);

    let snapshot = &report.live_instances[0];
    assert_eq!(snapshot.instance_id, live);
    assert_eq!(snapshot.script_url, "https://example.com/live-worker.js");
    assert_eq!(
        snapshot.posted_messages,
        vec![Value::String("hello".to_string())]
    );
    assert_eq!(snapshot.posted_shared_buffers, vec![vec![1, 2, 3]]);
    assert!(!snapshot.was_terminated);
}

#[test]
fn thread_runtime_topology_shutdown_keeps_live_instances_sorted_by_id() {
    let mut topology = ThreadRuntimeTopology::new();
    let first = topology
        .spawn_worker("https://example.com/first-worker.js")
        .expect("first worker");
    let middle = topology
        .spawn_worker("https://example.com/middle-worker.js")
        .expect("middle worker");
    let last = topology
        .spawn_worker("https://example.com/last-worker.js")
        .expect("last worker");

    topology.post_message(first, Value::String("first".to_string()));
    topology.post_shared_buffer(first, SharedArrayBuffer::from_bytes([1, 0, 0]));
    topology.post_message(middle, Value::String("middle".to_string()));
    topology.post_shared_buffer(middle, SharedArrayBuffer::from_bytes([0, 1, 0]));
    topology.post_message(last, Value::String("last".to_string()));
    topology.post_shared_buffer(last, SharedArrayBuffer::from_bytes([0, 0, 1]));

    topology.terminate(middle);

    let report = topology.shutdown();
    assert_eq!(report.total_instances, 3);
    assert_eq!(report.terminated_instances, 1);
    assert_eq!(report.live_instances.len(), 2);
    assert_eq!(
        report
            .live_instances
            .iter()
            .map(|snapshot| snapshot.instance_id)
            .collect::<Vec<_>>(),
        vec![first, last]
    );
    assert_eq!(
        report.live_instances[0].posted_messages,
        vec![Value::String("first".to_string())]
    );
    assert_eq!(
        report.live_instances[0].posted_shared_buffers,
        vec![vec![1, 0, 0]]
    );
    assert_eq!(
        report.live_instances[1].posted_messages,
        vec![Value::String("last".to_string())]
    );
    assert_eq!(
        report.live_instances[1].posted_shared_buffers,
        vec![vec![0, 0, 1]]
    );
    assert!(!report.live_instances[0].was_terminated);
    assert!(!report.live_instances[1].was_terminated);
}

#[test]
fn thread_runtime_topology_shutdown_keeps_live_instances_sorted_after_first_termination() {
    let mut topology = ThreadRuntimeTopology::new();
    let first = topology
        .spawn_worker("https://example.com/first-worker.js")
        .expect("first worker");
    let middle = topology
        .spawn_worker("https://example.com/middle-worker.js")
        .expect("middle worker");
    let last = topology
        .spawn_worker("https://example.com/last-worker.js")
        .expect("last worker");

    topology.post_message(first, Value::String("first".to_string()));
    topology.post_shared_buffer(first, SharedArrayBuffer::from_bytes([1, 0, 0]));
    topology.post_message(middle, Value::String("middle".to_string()));
    topology.post_shared_buffer(middle, SharedArrayBuffer::from_bytes([0, 1, 0]));
    topology.post_message(last, Value::String("last".to_string()));
    topology.post_shared_buffer(last, SharedArrayBuffer::from_bytes([0, 0, 1]));

    topology.terminate(first);

    let report = topology.shutdown();
    assert_eq!(report.total_instances, 3);
    assert_eq!(report.terminated_instances, 1);
    assert_eq!(report.live_instances.len(), 2);
    assert_eq!(
        report
            .live_instances
            .iter()
            .map(|snapshot| snapshot.instance_id)
            .collect::<Vec<_>>(),
        vec![middle, last]
    );
    assert_eq!(
        report.live_instances[0].posted_messages,
        vec![Value::String("middle".to_string())]
    );
    assert_eq!(
        report.live_instances[0].posted_shared_buffers,
        vec![vec![0, 1, 0]]
    );
    assert_eq!(
        report.live_instances[1].posted_messages,
        vec![Value::String("last".to_string())]
    );
    assert_eq!(
        report.live_instances[1].posted_shared_buffers,
        vec![vec![0, 0, 1]]
    );
    assert!(!report.live_instances[0].was_terminated);
    assert!(!report.live_instances[1].was_terminated);
}

#[test]
fn thread_runtime_topology_counts_multiple_terminated_instances_deterministically() {
    let mut topology = ThreadRuntimeTopology::new();
    let first = topology
        .spawn_worker("https://example.com/first-worker.js")
        .expect("first worker");
    let second = topology
        .spawn_worker("https://example.com/second-worker.js")
        .expect("second worker");
    let third = topology
        .spawn_worker("https://example.com/third-worker.js")
        .expect("third worker");

    topology.post_message(first, Value::String("first".to_string()));
    topology.post_shared_buffer(first, SharedArrayBuffer::from_bytes([1, 0, 0]));
    topology.post_message(second, Value::String("second".to_string()));
    topology.post_shared_buffer(second, SharedArrayBuffer::from_bytes([0, 1, 0]));
    topology.post_message(third, Value::String("third".to_string()));
    topology.post_shared_buffer(third, SharedArrayBuffer::from_bytes([0, 0, 1]));
    topology.terminate(first);
    topology.terminate(second);

    let report = topology.shutdown();
    assert_eq!(report.total_instances, 3);
    assert_eq!(report.terminated_instances, 2);
    assert_eq!(report.live_instances.len(), 1);
    assert_eq!(report.live_instances[0].instance_id, third);
    assert_eq!(
        report.live_instances[0].posted_messages,
        vec![Value::String("third".to_string())]
    );
    assert_eq!(
        report.live_instances[0].posted_shared_buffers,
        vec![vec![0, 0, 1]]
    );
    assert!(!report.live_instances[0].was_terminated);
}

#[test]
fn atomics_reports_lock_free_status_deterministically() {
    let first = Atomics::is_lock_free();
    let second = Atomics::is_lock_free();

    assert_eq!(first, bytewise_shared_memory_is_lock_free());
    assert_eq!(first, second);
}

#[test]
fn shared_array_buffer_clones_share_mutations() {
    let buffer = SharedArrayBuffer::from_bytes([1, 2, 3, 4]);
    let clone = buffer.clone();

    assert!(Atomics::is_lock_free());
    assert_eq!(buffer.byte_length(), 4);
    assert_eq!(buffer.snapshot(), vec![1, 2, 3, 4]);
    assert_eq!(Atomics::load(&clone, 1), Some(2));
    assert_eq!(Atomics::store(&clone, 1, 9), Some(2));
    assert_eq!(Atomics::add(&buffer, 0, 4), Some(1));
    assert_eq!(Atomics::and(&buffer, 0, 0b1111_1100), Some(5));
    assert_eq!(Atomics::or(&buffer, 1, 0b0000_0101), Some(9));
    assert_eq!(Atomics::xor(&buffer, 2, 0b0000_0110), Some(3));
    assert_eq!(Atomics::sub(&buffer, 2, 1), Some(5));
    assert_eq!(Atomics::compare_exchange(&buffer, 3, 4, 7), Some(Ok(4)));
    assert_eq!(Atomics::exchange(&clone, 0, 6), Some(4));
    assert_eq!(buffer.snapshot(), vec![6, 13, 4, 7]);
    assert_eq!(clone.snapshot(), vec![6, 13, 4, 7]);
}

#[test]
fn shared_array_buffer_compare_exchange_failure_leaves_bytes_unchanged() {
    let buffer = SharedArrayBuffer::from_bytes([10, 20]);

    assert_eq!(Atomics::compare_exchange(&buffer, 0, 11, 99), Some(Err(10)));
    assert_eq!(Atomics::snapshot(&buffer), vec![10, 20]);
}

#[test]
fn shared_array_buffer_supports_zero_length_buffers() {
    let buffer = SharedArrayBuffer::new(0);
    assert!(buffer.is_empty());
    assert!(Atomics::load(&buffer, 0).is_none());
    assert!(Atomics::store(&buffer, 0, 1).is_none());
    assert!(Atomics::and(&buffer, 0, 0xff).is_none());
    assert!(Atomics::or(&buffer, 0, 0xff).is_none());
    assert!(Atomics::xor(&buffer, 0, 0xff).is_none());
    assert!(Atomics::compare_exchange(&buffer, 0, 0, 1).is_none());
    assert!(Atomics::snapshot(&buffer).is_empty());
}

#[test]
fn broadcast_channel_stub_records_posted_messages() {
    let channel = BroadcastChannel::new("browser-corpus");
    assert_eq!(channel.name(), "browser-corpus");
    assert!(!channel.is_closed());

    channel.post_message(Value::String("ping".to_string()));
    assert_eq!(
        channel.posted_messages(),
        vec![Value::String("ping".to_string())]
    );

    channel.close();
    assert!(channel.is_closed());
    channel.post_message(Value::String("ignored".to_string()));
    assert_eq!(
        channel.posted_messages(),
        vec![Value::String("ping".to_string())]
    );
}

#[test]
fn broadcast_channel_stub_records_shared_buffers_with_shared_backing() {
    let channel = BroadcastChannel::new("browser-corpus");
    let buffer = SharedArrayBuffer::from_bytes([4, 5, 6]);

    channel.post_shared_buffer(buffer.clone());
    let posted = channel.posted_shared_buffers();
    assert_eq!(posted.len(), 1);
    assert_eq!(posted[0].snapshot(), vec![4, 5, 6]);

    buffer.add(2, 1);
    assert_eq!(channel.posted_shared_buffers()[0].snapshot(), vec![4, 5, 7]);
}

#[test]
fn broadcast_channel_stub_ignores_shared_buffer_posts_after_close() {
    let channel = BroadcastChannel::new("browser-corpus");
    let buffer = SharedArrayBuffer::from_bytes([4, 5, 6]);

    channel.post_shared_buffer(buffer.clone());
    channel.close();
    channel.post_shared_buffer(SharedArrayBuffer::from_bytes([7, 8, 9]));

    assert_eq!(channel.posted_shared_buffers(), vec![buffer]);
}

#[test]
fn broadcast_channel_stub_preserves_interleaved_post_order() {
    let channel = BroadcastChannel::new("browser-corpus");
    let buffer = SharedArrayBuffer::from_bytes([4, 5, 6]);

    channel.post_message(Value::String("alpha".to_string()));
    channel.post_shared_buffer(buffer.clone());
    channel.post_message(Value::String("omega".to_string()));

    assert_eq!(
        channel.posted_items(),
        vec![
            PostedItem::Message(Value::String("alpha".to_string())),
            PostedItem::SharedBuffer(buffer),
            PostedItem::Message(Value::String("omega".to_string())),
        ]
    );
}

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
