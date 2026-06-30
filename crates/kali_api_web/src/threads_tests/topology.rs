use super::*;

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
