use super::*;
use kali_common::bytewise_shared_memory_is_lock_free;
use std::sync::atomic::AtomicUsize;


#[test]
fn url_search_params_round_trips_values_and_serializes_deterministically() {
    let params = URLSearchParams::new();
    params.append("alpha", "1");
    params.append("beta", "two words");
    params.append("beta", "3");

    assert!(params.has("alpha"));
    assert_eq!(params.get("alpha").as_deref(), Some("1"));
    assert_eq!(
        params.get_all("beta"),
        vec!["two words".to_string(), "3".to_string()]
    );

    params.set("beta", "replacement");
    assert_eq!(params.get_all("beta"), vec!["replacement".to_string()]);
    params.delete("alpha");
    assert!(!params.has("alpha"));
    assert_eq!(
        params.entries(),
        vec![("beta".to_string(), "replacement".to_string())]
    );
    assert_eq!(params.to_string(), "beta=replacement");

    let parsed = URLSearchParams::from_query("alpha=1&beta=two+words&beta=3");
    assert_eq!(parsed.get("alpha").as_deref(), Some("1"));
    assert_eq!(
        parsed.get_all("beta"),
        vec!["two words".to_string(), "3".to_string()]
    );
}

#[test]
fn headers_request_and_response_round_trip_deterministically() {
    let headers = Headers::new();
    headers.append("Content-Type", "text/plain");
    headers.append("x-request-id", "alpha");
    headers.append("X-Request-Id", "beta");
    headers.set("Accept", "application/json");

    assert!(headers.has("content-type"));
    assert_eq!(headers.get("CONTENT-TYPE").as_deref(), Some("text/plain"));
    assert_eq!(headers.get("x-request-id").as_deref(), Some("alpha"));
    assert_eq!(
        headers.entries(),
        vec![
            ("content-type".to_string(), "text/plain".to_string()),
            ("x-request-id".to_string(), "alpha".to_string()),
            ("x-request-id".to_string(), "beta".to_string()),
            ("accept".to_string(), "application/json".to_string()),
        ]
    );

    let request = Request::with_parts(
        "https://example.com/api",
        "post",
        headers.clone(),
        "payload",
    )
    .expect("request");
    assert_eq!(request.method(), "POST");
    assert_eq!(request.url().as_str(), "https://example.com/api");
    assert_eq!(request.text().expect("request text"), "payload");
    assert_eq!(
        request.headers().get("accept").as_deref(),
        Some("application/json")
    );

    let response = Response::from_request(&request);
    assert_eq!(response.status(), 200);
    assert_eq!(response.status_text(), "OK");
    assert!(response.ok());
    assert_eq!(response.url().as_str(), "https://example.com/api");
    assert_eq!(response.text().expect("response text"), "payload");
    assert_eq!(
        response.headers().get("x-request-id").as_deref(),
        Some("alpha")
    );

    let echoed = fetch(&request);
    assert_eq!(echoed.text().expect("echo text"), "payload");
    assert_eq!(echoed.status(), 200);
}

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

#[test]
fn url_parser_can_parse_and_resolve() {
    let parsed = parse_url("https://example.com/path").expect("url");
    assert_eq!(parsed.as_str(), "https://example.com/path");

    let resolved = resolve_url("https://example.com/base/", "../child").expect("resolved");
    assert_eq!(resolved.as_str(), "https://example.com/child");
}

#[test]
fn url_object_round_trips_components() {
    let mut url = URL::new("https://example.com/base?alpha=1#fragment").expect("url");
    assert_eq!(url.as_str(), "https://example.com/base?alpha=1#fragment");
    assert_eq!(url.href(), "https://example.com/base?alpha=1#fragment");
    assert_eq!(url.protocol(), "https:");
    assert_eq!(url.pathname(), "/base");
    assert_eq!(url.search(), "?alpha=1");
    assert_eq!(url.hash(), "#fragment");
    assert_eq!(url.host(), Some("example.com"));

    url.set_pathname("/child");
    url.set_search("?beta=2");
    url.set_hash("section");
    assert_eq!(url.as_str(), "https://example.com/child?beta=2#section");
    assert_eq!(url.port(), None);
    assert_eq!(url.set_protocol("http:"), Ok(()));
    assert_eq!(url.as_str(), "http://example.com/child?beta=2#section");
    assert_eq!(url.set_host("example.org"), Ok(()));
    assert_eq!(url.set_port(Some(8080)), Ok(()));
    assert_eq!(url.as_str(), "http://example.org:8080/child?beta=2#section");

    let resolved = URL::resolve("https://example.com/base/", "../child").expect("resolved");
    assert_eq!(resolved.as_str(), "https://example.com/child");
    assert_eq!(
        resolved.clone().into_inner().as_str(),
        "https://example.com/child"
    );
}

#[test]
fn abort_controller_flips_the_signal() {
    let controller = AbortController::new();
    let signal = controller.signal();
    assert!(!signal.aborted());
    controller.abort();
    assert!(signal.aborted());
}

#[test]
fn abort_signal_dispatches_abort_events_once() {
    let controller = AbortController::new();
    let signal = controller.signal();
    let invocations = Arc::new(AtomicUsize::new(0));
    let invocations_clone = Arc::clone(&invocations);

    signal.add_event_listener("abort", move |event| {
        assert_eq!(event.event_type(), "abort");
        invocations_clone.fetch_add(1, Ordering::SeqCst);
    });

    controller.abort();
    controller.abort();

    assert!(signal.aborted());
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
    assert_eq!(signal.dispatch_event(&Event::new("abort")), 1);
}

#[test]
fn event_target_dispatches_registered_listeners() {
    let target = EventTarget::new();
    let seen = Arc::new(AtomicBool::new(false));
    let seen_clone = Arc::clone(&seen);

    target.add_event_listener("hello", move |event| {
        seen_clone.store(event.event_type() == "hello", Ordering::SeqCst);
    });

    let event = Event::new("hello");
    assert_eq!(target.dispatch_event(&event), 1);
    assert!(seen.load(Ordering::SeqCst));
}

#[test]
fn event_target_can_remove_registered_listeners() {
    let target = EventTarget::new();
    let invocations = Arc::new(AtomicUsize::new(0));
    let invocations_clone = Arc::clone(&invocations);

    let listener_id = target.add_event_listener("hello", move |_| {
        invocations_clone.fetch_add(1, Ordering::SeqCst);
    });

    let event = Event::new("hello");
    assert_eq!(target.dispatch_event(&event), 1);
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
    assert!(target.remove_event_listener("hello", listener_id));
    assert!(!target.remove_event_listener("hello", listener_id));
    assert_eq!(target.dispatch_event(&event), 0);
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
}

#[test]
fn event_target_can_remove_listeners_during_dispatch_without_deadlocking() {
    let target = EventTarget::new();
    let first_invocations = Arc::new(AtomicUsize::new(0));
    let second_invocations = Arc::new(AtomicUsize::new(0));
    let second_listener_id = Arc::new(AtomicUsize::new(usize::MAX));
    let target_for_first = target.clone();

    let first_invocations_clone = Arc::clone(&first_invocations);
    let second_listener_id_clone = Arc::clone(&second_listener_id);
    target.add_event_listener("hello", move |_| {
        first_invocations_clone.fetch_add(1, Ordering::SeqCst);
        let removed = target_for_first
            .remove_event_listener("hello", second_listener_id_clone.load(Ordering::SeqCst));
        assert!(removed, "listener removal should succeed during dispatch");
    });

    let second_invocations_clone = Arc::clone(&second_invocations);
    let second_id = target.add_event_listener("hello", move |_| {
        second_invocations_clone.fetch_add(1, Ordering::SeqCst);
    });
    second_listener_id.store(second_id, Ordering::SeqCst);

    let event = Event::new("hello");
    assert_eq!(target.dispatch_event(&event), 1);
    assert_eq!(first_invocations.load(Ordering::SeqCst), 1);
    assert_eq!(second_invocations.load(Ordering::SeqCst), 0);
    assert!(!target.remove_event_listener("hello", second_id));
}

#[test]
fn custom_event_carries_detail_payload() {
    let event = CustomEvent::new("payload", Value::String("detail".to_string()));
    assert_eq!(event.event().event_type(), "payload");
    assert_eq!(event.detail(), &Value::String("detail".to_string()));
}

#[test]
fn websocket_stub_tracks_sent_messages() {
    let mut socket = WebSocket::new("https://example.com/socket").expect("websocket url");
    assert_eq!(socket.ready_state(), WebSocketReadyState::Open);
    assert_eq!(socket.url().as_str(), "https://example.com/socket");

    socket.send_text("hello");
    socket.send_text("world");
    socket.send_bytes(vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(socket.sent_text_messages(), vec!["hello", "world"]);
    assert_eq!(
        socket.sent_binary_messages(),
        vec![vec![0xde, 0xad, 0xbe, 0xef]]
    );

    socket.close();
    assert_eq!(socket.ready_state(), WebSocketReadyState::Closed);
}

#[test]
fn websocket_stub_clones_binary_payloads_deterministically() {
    let socket = WebSocket::new("https://example.com/socket").expect("websocket url");
    let mut payload = vec![0x01, 0x02, 0x03, 0x04];

    socket.send_bytes(&payload);
    payload[0] = 0xff;
    payload[1] = 0xee;

    socket.send_bytes(payload.as_slice());
    assert_eq!(
        socket.sent_binary_messages(),
        vec![vec![0x01, 0x02, 0x03, 0x04], vec![0xff, 0xee, 0x03, 0x04]]
    );
}

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
