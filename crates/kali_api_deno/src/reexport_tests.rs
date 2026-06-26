use crate::*;

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

