use super::*;
use std::sync::atomic::AtomicUsize;

#[test]
fn performance_now_is_monotonic_and_non_negative() {
    let first = performance_now();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let second = performance_now();

    assert!(first >= 0.0, "first timestamp: {first}");
    assert!(
        second >= first,
        "timestamps should not go backwards: {first} -> {second}"
    );
}

#[test]
fn random_fill_populates_the_requested_buffer() {
    let mut buffer = [0u8; 16];
    fill_random_values(&mut buffer).expect("random fill");
    assert_eq!(buffer.len(), 16);
}

#[test]
fn random_uuid_has_the_expected_shape() {
    let uuid = random_uuid().expect("random uuid");

    assert_eq!(uuid.len(), 36);
    assert_eq!(uuid.chars().nth(8), Some('-'));
    assert_eq!(uuid.chars().nth(13), Some('-'));
    assert_eq!(uuid.chars().nth(18), Some('-'));
    assert_eq!(uuid.chars().nth(23), Some('-'));
    assert_eq!(uuid.chars().nth(14), Some('4'));
    assert!(matches!(uuid.chars().nth(19), Some('8' | '9' | 'a' | 'b')));
}

#[test]
fn crypto_facade_reuses_the_shared_randomness_helpers() {
    let crypto = crypto();
    let mut buffer = [0u8; 8];

    crypto
        .get_random_values(&mut buffer)
        .expect("crypto.getRandomValues");
    assert_eq!(buffer.len(), 8);

    let uuid = crypto.random_uuid().expect("crypto.randomUUID");
    assert_eq!(uuid.len(), 36);
    assert_eq!(uuid.chars().nth(14), Some('4'));
}

#[test]
fn crypto_subtle_digest_supports_sha256() {
    let digest = crypto()
        .subtle()
        .digest(" sha-256 ", b"abc")
        .expect("sha-256 digest");

    assert_eq!(
        digest,
        vec![
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ]
    );
}

#[test]
fn crypto_subtle_digest_rejects_unknown_algorithms() {
    let error = crypto()
        .subtle()
        .digest("sha-999", b"abc")
        .expect_err("unknown algorithms should be rejected");

    assert!(
        error
            .to_string()
            .contains("unsupported Web Crypto digest algorithm 'sha-999'"),
        "error: {error}"
    );
}

#[test]
fn text_codec_round_trips_unicode() {
    let input = "héllo 🌍";
    let encoded = text_encode(input);
    assert_eq!(encoded, input.as_bytes());
    let decoded = text_decode(&encoded).expect("valid utf-8");
    assert_eq!(decoded, input);
}

#[test]
fn base64_helpers_round_trip_binary_strings() {
    assert_eq!(btoa("hello").expect("encode"), "aGVsbG8=");
    assert_eq!(atob("aGVs bG8=").expect("decode"), "hello");
    assert_eq!(atob("aGVsbG8").expect("unpadded decode"), "hello");
}

#[test]
fn base64_helpers_reject_out_of_range_input() {
    assert!(btoa("€").is_err());
}

#[test]
fn structured_clone_copies_values() {
    let original = vec![1, 2, 3];
    let cloned = structured_clone(&original);
    assert_eq!(cloned, original);
}

#[test]
fn blob_collects_bytes_and_text() {
    let blob = Blob::new(
        ["hello ".as_bytes(), "world".as_bytes()],
        Some("text/plain".to_string()),
    );
    assert_eq!(blob.size(), 11);
    assert_eq!(blob.mime_type(), Some("text/plain"));
    assert_eq!(blob.bytes(), b"hello world");
    assert_eq!(blob.text().expect("blob text"), "hello world");
}

#[test]
fn file_wraps_blob_metadata() {
    let file = File::new(
        "report.txt",
        ["hello ".as_bytes(), "world".as_bytes()],
        Some("text/plain".to_string()),
        42,
    );
    assert_eq!(file.name(), "report.txt");
    assert_eq!(file.last_modified(), 42);
    assert_eq!(file.size(), 11);
    assert_eq!(file.bytes(), b"hello world");
    assert_eq!(file.blob().mime_type(), Some("text/plain"));
    assert_eq!(file.text().expect("file text"), "hello world");
}

#[test]
fn file_reader_reads_blob_and_file_payloads() {
    let blob = Blob::new(
        ["reader payload".as_bytes()],
        Some("text/plain".to_string()),
    );
    let file = File::new("reader.txt", ["reader payload".as_bytes()], None, 7);

    let mut reader = FileReader::new();
    assert_eq!(reader.ready_state(), FileReaderState::Empty);

    assert_eq!(
        reader.read_as_text(&blob).expect("blob text"),
        "reader payload"
    );
    assert_eq!(reader.ready_state(), FileReaderState::Done);
    assert_eq!(reader.result_bytes(), Some(b"reader payload".as_slice()));

    reader.clear();
    assert_eq!(reader.ready_state(), FileReaderState::Empty);
    assert!(reader.result_bytes().is_none());

    assert_eq!(reader.read_file_as_bytes(&file), b"reader payload");
    assert_eq!(reader.ready_state(), FileReaderState::Done);
    assert_eq!(
        reader.read_file_as_text(&file).expect("file text"),
        "reader payload"
    );
}

#[test]
fn form_data_records_entries_and_preserves_order() {
    let blob = Blob::new(["form payload".as_bytes()], Some("text/plain".to_string()));
    let file = File::new("form.txt", ["file payload".as_bytes()], None, 13);
    let form = FormData::new();

    form.append("alpha", "1");
    form.append("beta", blob.clone());
    form.append("beta", file.clone());

    assert!(form.has("alpha"));
    assert_eq!(
        form.get("alpha").expect("alpha entry").value(),
        &FormDataValue::Text("1".to_string())
    );
    assert_eq!(form.get_all("beta").len(), 2);
    assert_eq!(
        form.get_all("beta")[0].value(),
        &FormDataValue::Blob(blob.clone())
    );
    assert_eq!(
        form.get_all("beta")[1].value(),
        &FormDataValue::File(file.clone())
    );

    form.set("beta", "replacement");
    assert_eq!(form.get_all("beta").len(), 1);
    assert_eq!(
        form.get("beta").expect("beta entry").value(),
        &FormDataValue::Text("replacement".to_string())
    );

    form.delete("alpha");
    assert!(!form.has("alpha"));
    assert_eq!(form.entries().len(), 1);
}

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
fn storage_round_trips_values_and_stays_ordered() {
    let storage = Storage::new();
    storage.set_item("alpha", "1");
    storage.set_item("beta", "2");

    assert_eq!(storage.length(), 2);
    assert_eq!(storage.get_item("alpha").as_deref(), Some("1"));
    assert_eq!(storage.key(0).as_deref(), Some("alpha"));
    assert_eq!(storage.key(1).as_deref(), Some("beta"));
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

#[test]
fn navigator_baseline_exposes_stable_metadata() {
    let navigator = navigator();
    assert_eq!(navigator.user_agent(), "Kali/1.0 (Web)");
    assert_eq!(navigator.language(), "en-US");
    assert_eq!(navigator.languages(), &[String::from("en-US")]);
    assert!(navigator.on_line());
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
    assert_eq!(socket.sent_text_messages(), vec!["hello", "world"]);

    socket.close();
    assert_eq!(socket.ready_state(), WebSocketReadyState::Closed);
}

#[test]
fn worker_stub_records_posted_messages() {
    let worker = Worker::new("https://example.com/worker.js").expect("worker url");
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
fn shared_array_buffer_clones_share_mutations() {
    let buffer = SharedArrayBuffer::from_bytes([1, 2, 3, 4]);
    let clone = buffer.clone();

    assert_eq!(buffer.byte_length(), 4);
    assert_eq!(buffer.snapshot(), vec![1, 2, 3, 4]);
    assert_eq!(Atomics::load(&clone, 1), Some(2));
    assert_eq!(Atomics::store(&clone, 1, 9), Some(2));
    assert_eq!(Atomics::add(&buffer, 0, 4), Some(1));
    assert_eq!(Atomics::sub(&buffer, 2, 1), Some(3));
    assert_eq!(Atomics::compare_exchange(&buffer, 3, 4, 7), Some(Ok(4)));
    assert_eq!(Atomics::exchange(&clone, 0, 6), Some(5));
    assert_eq!(buffer.snapshot(), vec![6, 9, 2, 7]);
    assert_eq!(clone.snapshot(), vec![6, 9, 2, 7]);
}

#[test]
fn shared_array_buffer_supports_zero_length_buffers() {
    let buffer = SharedArrayBuffer::new(0);
    assert!(buffer.is_empty());
    assert!(Atomics::load(&buffer, 0).is_none());
    assert!(Atomics::store(&buffer, 0, 1).is_none());
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
