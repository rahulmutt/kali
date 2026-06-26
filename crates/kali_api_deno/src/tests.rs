use super::*;
use std::path::Path;

#[test]
fn navigator_baseline_is_reexported() {
    let navigator = navigator();
    assert_eq!(navigator.user_agent(), "Kali/1.0 (Web)");
    assert!(navigator.on_line());
}

#[test]
fn random_uuid_is_reexported() {
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
fn crypto_facade_is_reexported() {
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

#[test]
fn initialization_drags_in_shared_web_baseline() {
    deno_api_init();
    assert!(performance_now() >= 0.0);
    assert_eq!(text_encode("deno"), b"deno");
}

#[test]
fn web_file_reader_is_reexported_through_the_deno_surface() {
    let blob = Blob::new(["deno payload".as_bytes()], None);
    let mut reader = FileReader::new();

    assert_eq!(reader.ready_state(), FileReaderState::Empty);
    assert_eq!(
        reader.read_as_text(&blob).expect("blob text"),
        "deno payload"
    );
    assert_eq!(reader.ready_state(), FileReaderState::Done);
    assert_eq!(reader.result_bytes(), Some(b"deno payload".as_slice()));
}

#[test]
fn form_data_is_reexported_through_the_deno_surface() {
    let blob = Blob::new(
        ["deno form payload".as_bytes()],
        Some("text/plain".to_string()),
    );
    let file = File::new("deno-form.txt", ["file payload".as_bytes()], None, 17);
    let form = FormData::new();

    form.append("blob", blob.clone());
    form.append("file", file.clone());
    form.set("text", "value");

    assert_eq!(
        form.get("text").expect("text entry").value(),
        &FormDataValue::Text("value".to_string())
    );
    assert_eq!(
        form.get("blob").expect("blob entry").value(),
        &FormDataValue::Blob(blob)
    );
    assert_eq!(
        form.get("file").expect("file entry").value(),
        &FormDataValue::File(file)
    );
}

#[test]
fn browser_url_is_reexported_through_the_deno_surface() {
    let mut url = URL::new("https://example.com/deno?alpha=1#fragment").expect("url");
    assert_eq!(url.as_str(), "https://example.com/deno?alpha=1#fragment");
    assert_eq!(url.pathname(), "/deno");
    assert_eq!(url.search(), "?alpha=1");
    assert_eq!(url.hash(), "#fragment");

    url.set_pathname("/bridge");
    url.set_search("?beta=2");
    url.set_hash("section");
    assert_eq!(url.as_str(), "https://example.com/bridge?beta=2#section");
}

#[test]
fn browser_stubs_are_reexported_through_the_deno_surface() {
    let socket = WebSocket::new("https://example.com/socket").expect("websocket url");
    assert_eq!(socket.ready_state(), WebSocketReadyState::Open);
    socket.send_bytes(vec![0, 1, 2, 3]);
    assert_eq!(socket.sent_binary_messages(), vec![vec![0, 1, 2, 3]]);

    let worker = Worker::new("https://example.com/worker.js").expect("worker url");
    assert_eq!(
        worker.script_url().as_str(),
        "https://example.com/worker.js"
    );

    let channel = BroadcastChannel::new("browser-cache");
    assert_eq!(channel.name(), "browser-cache");
    channel.post_message(serde_json::json!({"ok": true}));
    assert_eq!(
        channel.posted_messages(),
        vec![serde_json::json!({"ok": true})]
    );

    let params = URLSearchParams::from_query("alpha=1&beta=two+words");
    assert_eq!(params.get("alpha").as_deref(), Some("1"));
    assert_eq!(params.get_all("beta"), vec!["two words".to_string()]);

    let db = IndexedDB::open("browser-cache");
    db.put("objects", "item", serde_json::json!({"ok": true}));

    let alias = IndexedDb::open("browser-cache-alias");
    assert_eq!(alias.name(), "browser-cache-alias");
    assert_eq!(
        db.get("objects", "item"),
        Some(serde_json::json!({"ok": true}))
    );
}

#[test]
fn tcp_connect_and_listen_round_trip_bytes() {
    let listener = listen("127.0.0.1", 0).expect("listen");
    let addr = listener.local_addr().expect("listener addr");

    let server = std::thread::spawn(move || {
        let (mut connection, peer) = listener.accept().expect("accept");
        assert_eq!(peer.ip().to_string(), "127.0.0.1");
        assert_eq!(connection.peer_addr().expect("peer addr"), peer);
        assert_eq!(connection.read_to_end().expect("server read"), b"ping");
        connection.write_all(b"pong").expect("server write");
        connection.flush().expect("server flush");
        drop(connection);
    });

    let mut client = connect("127.0.0.1", addr.port()).expect("connect");
    assert_eq!(
        client
            .local_addr()
            .expect("client local addr")
            .ip()
            .to_string(),
        "127.0.0.1"
    );
    client.write_all(b"ping").expect("client write");
    client.flush().expect("client flush");
    client.shutdown_write().expect("client shutdown write");
    assert_eq!(client.read_to_end().expect("client read"), b"pong");

    server.join().expect("server join");
}

#[test]
fn serve_emits_a_basic_http_response() {
    let server = serve(
        |request| {
            assert_eq!(request.method(), "GET");
            assert_eq!(request.url().path(), "/hello");
            assert_eq!(request.headers().get("host").as_deref(), Some("127.0.0.1"));

            let headers = kali_api_web::Headers::new();
            headers.set("content-type", "text/plain; charset=utf-8");
            Response::with_parts(request.url().as_str(), 200, "OK", headers, "hello")
                .expect("response")
        },
        "127.0.0.1",
        0,
    )
    .expect("serve");

    let addr = server.local_addr();
    let mut socket = std::net::TcpStream::connect(addr).expect("connect server");
    socket
        .write_all(b"GET /hello HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
        .expect("write request");
    socket
        .shutdown(std::net::Shutdown::Write)
        .expect("shutdown write");

    let mut response = String::new();
    socket.read_to_string(&mut response).expect("read response");
    assert!(
        response.starts_with("HTTP/1.1 200 OK"),
        "response: {response}"
    );
    assert!(
        response.contains("content-type: text/plain; charset=utf-8"),
        "response: {response}"
    );
    assert!(response.ends_with("hello"), "response: {response}");

    server.join().expect("server join");
}
