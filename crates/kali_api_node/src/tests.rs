use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};
use tempfile::tempdir;

#[test]
fn process_context_tracks_env_and_output() {
    let mut env = BTreeMap::new();
    env.insert("HOME".to_string(), "/tmp/home".to_string());
    let mut process = NodeProcess::with_host_context(
        vec!["node".into(), "script.js".into()],
        env,
        "/workspace/project",
    );

    assert_eq!(process.argv(), &["node", "script.js"]);
    assert_eq!(process.argv0(), "node");
    assert_eq!(process.argv_len(), 2);
    assert_eq!(process.argv_at(1), Some("script.js"));
    assert_eq!(process.cwd(), Path::new("/workspace/project"));
    assert_eq!(process.pid(), std::process::id());
    assert_eq!(process.env_get("HOME"), Some("/tmp/home"));
    assert!(process.env_has("HOME"));
    assert!(process.has("HOME"));
    assert!(!process.env_has("MISSING"));
    assert!(!process.has("MISSING"));
    assert_eq!(
        process.env_snapshot(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.snapshot(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.env_to_object(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.env_snapshot_object_value(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.snapshot_object_value(),
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))])
    );
    assert_eq!(
        process.env_snapshot_value(),
        serde_json::json!({ "HOME": "/tmp/home" })
    );
    assert_eq!(
        process.env_snapshot_json_value(),
        serde_json::json!({ "HOME": "/tmp/home" })
    );
    assert_eq!(
        process.snapshot_json_value(),
        serde_json::json!({ "HOME": "/tmp/home" })
    );
    assert_eq!(process.snapshot_value(), process.env_snapshot_value());
    assert_eq!(
        process.env_to_json_value(),
        serde_json::json!({ "HOME": "/tmp/home" })
    );
    assert_eq!(process.env_set("EDITOR", "nano"), None);
    assert_eq!(process.env_remove("HOME"), Some(String::from("/tmp/home")));
    assert_eq!(process.env_delete("EDITOR"), Some(String::from("nano")));
    assert_eq!(process.env_get("HOME"), None);
    assert_eq!(process.env_get("EDITOR"), None);
    assert_eq!(process.env_snapshot(), BTreeMap::new());
    assert_eq!(process.snapshot(), BTreeMap::new());
    assert_eq!(process.env_to_object(), BTreeMap::new());
    assert_eq!(process.env_snapshot_value(), serde_json::json!({}));
    assert_eq!(process.env_to_json_value(), serde_json::json!({}));

    process.write_stdout("hello");
    process.write_stderr("oops");
    process.set_exit_code(7);

    assert_eq!(process.stdout(), "hello");
    assert_eq!(process.stderr(), "oops");
    assert_eq!(process.exit_code(), Some(7));

    process.exit(3);
    assert_eq!(process.exit_code(), Some(3));
}

#[test]
fn runtime_projection_preserves_host_argv0_projection() {
    let projection = NodeRuntimeProjection::from_host_context(
        vec!["node".into(), "script.js".into()],
        BTreeMap::new(),
        "/workspace/project",
    );

    assert_eq!(projection.process().argv0(), "node");
}

#[test]
fn runtime_projection_exposes_deterministic_env_snapshot() {
    let mut env = BTreeMap::new();
    env.insert("HOME".to_string(), "/tmp/home".to_string());
    env.insert("EDITOR".to_string(), "nano".to_string());

    let mut projection = NodeRuntimeProjection::from_host_context(
        vec!["node".into(), "script.js".into()],
        env,
        "/workspace/project",
    );

    assert_eq!(
        projection.env_snapshot(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert_eq!(
        projection.snapshot(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert_eq!(
        projection.env_to_object(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert_eq!(
        projection.env_snapshot_object_value(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert_eq!(
        projection.snapshot_object_value(),
        BTreeMap::from([
            (String::from("EDITOR"), String::from("nano")),
            (String::from("HOME"), String::from("/tmp/home")),
        ])
    );
    assert!(projection.env_has("HOME"));
    assert!(projection.has("HOME"));
    assert!(!projection.env_has("MISSING"));
    assert!(!projection.has("MISSING"));
    assert_eq!(
        projection.env_snapshot_value(),
        serde_json::json!({ "EDITOR": "nano", "HOME": "/tmp/home" })
    );
    assert_eq!(
        projection.env_snapshot_json_value(),
        serde_json::json!({ "EDITOR": "nano", "HOME": "/tmp/home" })
    );
    assert_eq!(
        projection.snapshot_json_value(),
        serde_json::json!({ "EDITOR": "nano", "HOME": "/tmp/home" })
    );
    assert_eq!(projection.snapshot_value(), projection.env_snapshot_value());
    assert_eq!(
        projection.env_delete("HOME"),
        Some(String::from("/tmp/home"))
    );
    assert_eq!(projection.env_has("HOME"), false);
    assert_eq!(
        projection.env_to_json_value(),
        serde_json::json!({ "EDITOR": "nano" })
    );

    projection.chdir("./nested/../other");
    assert_eq!(
        projection.env_snapshot(),
        BTreeMap::from([(String::from("EDITOR"), String::from("nano"))])
    );
    assert_eq!(
        projection.snapshot(),
        BTreeMap::from([(String::from("EDITOR"), String::from("nano"))])
    );
    assert_eq!(
        projection.env_snapshot_value(),
        serde_json::json!({ "EDITOR": "nano" })
    );
}

#[test]
fn default_process_context_uses_node_as_argv0() {
    let process = NodeProcess::default();

    assert_eq!(process.argv0(), "node");
    assert!(process.argv().is_empty());
}

#[test]
fn path_helpers_are_lexical_and_deterministic() {
    assert_eq!(
        normalize_path("./foo/../bar//baz"),
        PathBuf::from("bar/baz")
    );
    assert_eq!(
        join_path("/tmp", "project/src"),
        PathBuf::from("/tmp/project/src")
    );
    assert_eq!(
        resolve_path("/tmp/project", "../lib/index.js"),
        PathBuf::from("/tmp/lib/index.js")
    );
    assert_eq!(
        relative_path("/tmp/project/src", "/tmp/project/lib/index.js"),
        PathBuf::from("../lib/index.js")
    );
    assert_eq!(
        dirname("/tmp/project/src/main.ts"),
        PathBuf::from("/tmp/project/src")
    );
    assert_eq!(basename("/tmp/project/src/main.ts"), "main.ts");
    assert_eq!(extname("/tmp/project/src/main.ts"), ".ts");
}

#[test]
fn child_process_helpers_capture_command_output() {
    let (command, args): (&str, &[&str]) = if cfg!(windows) {
        ("cmd", &["/C", "echo", "child-process"])
    } else {
        ("sh", &["-lc", "printf child-process"])
    };

    let output = NodeChildProcess::spawn_sync(command, args).expect("spawn child process");
    assert_eq!(output.status(), 0);
    assert_eq!(
        String::from_utf8(output.stdout().to_vec())
            .expect("stdout")
            .trim_end(),
        "child-process"
    );
    assert!(output.stderr().is_empty(), "stderr: {:?}", output.stderr());
}

#[test]
fn crypto_helpers_produce_expected_formats() {
    assert_eq!(
        sha256_hex("hello"),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    assert_eq!(
        NodeCrypto::create_hash("sha256", "hello").expect("hash"),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    assert_eq!(
        NodeCrypto::create_hmac(
            "sha256",
            "key",
            "The quick brown fox jumps over the lazy dog"
        )
        .expect("hmac"),
        "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
    );
    assert_eq!(random_bytes(16).expect("random bytes").len(), 16);
    assert_eq!(NodeCrypto::random_bytes(8).expect("random bytes").len(), 8);

    let uuid = random_uuid_v4().expect("uuid");
    assert_eq!(uuid.len(), 36);
    assert_eq!(&uuid[14..15], "4");
    assert!(matches!(&uuid[19..20], "8" | "9" | "a" | "b"));
    assert_eq!(NodeCrypto::random_uuid_v4().expect("uuid").len(), 36);
}

#[test]
fn event_emitter_invokes_listeners_in_order() {
    use std::sync::{Arc, Mutex};

    let emitter = EventEmitter::new();
    let observed: Arc<Mutex<Vec<(String, i32)>>> = Arc::new(Mutex::new(Vec::new()));

    {
        let observed = Arc::clone(&observed);
        emitter.on("message", move |event| {
            observed
                .lock()
                .expect("observed mutex")
                .push((event.event_type().to_string(), 1));
        });
    }
    {
        let observed = Arc::clone(&observed);
        emitter.on("message", move |event| {
            observed
                .lock()
                .expect("observed mutex")
                .push((event.event_type().to_string(), 2));
        });
    }

    let event = NodeEvent::with_detail("message", "payload");
    assert_eq!(emitter.emit(&event), 2);
    assert_eq!(
        observed.lock().expect("observed mutex").clone(),
        vec![("message".to_string(), 1), ("message".to_string(), 2)]
    );
    assert_eq!(event.detail(), Some("payload"));
    assert_eq!(emitter.listener_count("message"), 2);
}

#[test]
fn buffer_and_util_helpers_round_trip() {
    let buffer = NodeBuffer::from_utf8("hello");
    assert_eq!(buffer.as_slice(), b"hello");
    assert_eq!(buffer.len(), 5);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.to_utf8().expect("utf8"), "hello");
    assert_eq!(buffer.to_base64(), "aGVsbG8=");
    assert_eq!(
        NodeBuffer::from_base64("aGVsbG8=")
            .expect("base64")
            .as_slice(),
        b"hello"
    );
    assert_eq!(buffer.to_hex(), "68656c6c6f");
    assert_eq!(
        NodeBuffer::from_hex("68656c6c6f").expect("hex").as_slice(),
        b"hello"
    );
    assert!(NodeBuffer::from_hex("abc").is_err());

    let bytes = NodeBuffer::from_bytes(vec![1, 2, 3]).into_bytes();
    assert_eq!(bytes, vec![1, 2, 3]);

    let formatted = util_format(&["node", "compat", "layer"]);
    assert_eq!(formatted, "node compat layer");
    assert_eq!(
        NodeUtil::format(&["node", "compat", "layer"]),
        "node compat layer"
    );
    assert_eq!(util_inspect(&vec![1, 2, 3]), "[1, 2, 3]");
    assert_eq!(NodeUtil::inspect(&vec![1, 2, 3]), "[1, 2, 3]");
    assert_eq!(
        util_promisify(|callback| callback(Ok::<_, String>(42))),
        Ok(42)
    );
    assert_eq!(
        NodeUtil::promisify(|callback| callback(Ok::<_, String>(21))),
        Ok(21)
    );
    assert_eq!(assert_true(true, "ok"), Ok(()));
    assert_eq!(assert_true(false, "fail"), Err("fail".to_string()));
    assert_eq!(NodeAssert::strict_equal(&4, &4, "strict"), Ok(()));
    assert_eq!(NodeAssert::not_strict_equal(&4, &5, "not strict"), Ok(()));
}

#[test]
fn assert_helpers_produce_clear_results() {
    assert_eq!(NodeAssert::ok(true, "ok"), Ok(()));
    assert_eq!(NodeAssert::ok(false, "bad"), Err("bad".to_string()));
    assert_eq!(NodeAssert::equal(&3, &3, "equal"), Ok(()));
    assert_eq!(
        NodeAssert::equal(&3, &4, "mismatch"),
        Err("mismatch: expected 4, got 3".to_string())
    );
    assert_eq!(NodeAssert::not_equal(&3, &4, "not equal"), Ok(()));
    assert_eq!(
        NodeAssert::not_equal(&3, &3, "same"),
        Err("same: value unexpectedly matched 3".to_string())
    );
    assert_eq!(
        NodeAssert::deep_equal(&vec![1, 2], &vec![1, 2], "deep"),
        Ok(())
    );
    assert_eq!(NodeAssert::fail("boom"), Err("boom".to_string()));
}

#[test]
fn fs_helpers_round_trip_files_and_directories() {
    let dir = tempdir().expect("tempdir");
    let fs = NodeFs::new(dir.path());

    fs.mkdir("nested", false).expect("mkdir");
    fs.write_text_file("nested/alpha.txt", "alpha")
        .expect("write text");
    fs.write_file("nested/beta.bin", [0, 1, 2])
        .expect("write file");
    fs.rename("nested/alpha.txt", "nested/renamed.txt")
        .expect("rename file");

    assert_eq!(
        fs.read_file("nested/beta.bin").expect("read file"),
        vec![0, 1, 2]
    );
    assert_eq!(
        fs.read_text_file("nested/renamed.txt").expect("read text"),
        "alpha"
    );
    assert_eq!(
        fs.readdir("nested").expect("readdir"),
        vec!["beta.bin".to_string(), "renamed.txt".to_string()]
    );

    let stat = fs.stat("nested/renamed.txt").expect("stat");
    assert!(stat.is_file());
    assert!(!stat.is_dir());
    assert!(!stat.is_symlink());
    assert_eq!(stat.len(), 5);

    let lstat = fs.lstat("nested/renamed.txt").expect("lstat");
    assert!(lstat.is_file());
    assert!(!lstat.is_dir());
    assert!(!lstat.is_symlink());

    fs.remove("nested/beta.bin", false).expect("remove file");
    fs.remove("nested", true).expect("remove dir");
    assert!(!fs.exists("nested"));
}

#[test]
fn fs_promises_helpers_match_sync_helpers() {
    let dir = tempdir().expect("tempdir");
    let fs = NodeFsPromises::new(dir.path());

    fs.mkdir("nested", false).expect("mkdir");
    fs.write_text_file("nested/alpha.txt", "alpha")
        .expect("write text");
    fs.write_file("nested/beta.bin", [0, 1, 2])
        .expect("write file");
    fs.rename("nested/alpha.txt", "nested/renamed.txt")
        .expect("rename file");

    assert_eq!(
        fs.read_file("nested/beta.bin").expect("read file"),
        vec![0, 1, 2]
    );
    assert_eq!(
        fs.read_text_file("nested/renamed.txt").expect("read text"),
        "alpha"
    );
    assert_eq!(
        fs.readdir("nested").expect("readdir"),
        vec!["beta.bin".to_string(), "renamed.txt".to_string()]
    );

    let stat = fs.stat("nested/renamed.txt").expect("stat");
    assert!(stat.is_file());
    assert!(!stat.is_dir());
    assert!(!stat.is_symlink());
    assert_eq!(stat.len(), 5);

    let lstat = fs.lstat("nested/renamed.txt").expect("lstat");
    assert!(lstat.is_file());
    assert!(!lstat.is_dir());
    assert!(!lstat.is_symlink());

    fs.remove("nested/beta.bin", false).expect("remove file");
    fs.remove("nested", true).expect("remove dir");
    assert!(!fs.exists("nested"));
}

#[test]
fn stream_helpers_concatenate_bytes() {
    let bytes = NodeStream::concat(b"hello ", b"world");
    assert_eq!(bytes, b"hello world");
    assert_eq!(NodeStream::from_utf8("abc"), b"abc");
    assert_eq!(NodeStream::from_bytes(vec![1, 2, 3]), vec![1, 2, 3]);
    assert_eq!(NodeStream::to_utf8(b"kali").expect("utf8"), "kali");
}

#[test]
fn http_helpers_fetch_local_response_body() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let body = "hello node http";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let server = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            let _ = stream.write_all(response.as_bytes());
        }
    });

    let url = format!("http://127.0.0.1:{}/", addr.port());
    let response = NodeHttp::get(&url).expect("http get");
    assert_eq!(response.status(), 200);
    assert_eq!(response.body(), body.as_bytes());
    assert_eq!(response.text().expect("text"), body);

    server.join().expect("server thread");
}

#[test]
fn os_and_url_helpers_expose_expected_views() {
    let os = NodeOs;
    assert!(!os.platform().is_empty());
    assert!(!os.arch().is_empty());
    assert!(matches!(os.eol(), "\n" | "\r\n"));
    assert!(os.cpus() >= 1);
    assert_eq!(os.tmpdir(), std::env::temp_dir());

    let expected_home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from));
    assert_eq!(os.home_dir(), expected_home);

    let parsed = NodeUrl::parse("https://example.com/path?query=1").expect("url");
    assert_eq!(parsed.as_str(), "https://example.com/path?query=1");

    let resolved = NodeUrl::resolve("https://example.com/base/", "../child").expect("resolve");
    assert_eq!(resolved.as_str(), "https://example.com/child");
}

#[test]
fn runtime_projection_bundles_common_node_surfaces() {
    let dir = tempdir().expect("tempdir");
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).expect("create nested dir");

    let mut projection = NodeRuntimeProjection::from_host_context(
        vec!["node".into(), "script.js".into()],
        BTreeMap::from([(String::from("HOME"), String::from("/tmp/home"))]),
        dir.path(),
    );

    assert_eq!(
        projection.process().argv(),
        &vec!["node".to_string(), "script.js".to_string()][..]
    );
    assert_eq!(projection.process().argv_len(), 2);
    assert_eq!(projection.process().env_get("HOME"), Some("/tmp/home"));
    assert_eq!(projection.fs().cwd(), dir.path());
    assert!(!projection.os().platform().is_empty());
    assert_eq!(projection.url(), NodeUrl);
    assert_eq!(projection.util(), NodeUtil);
    assert_eq!(projection.assert(), NodeAssert);
    assert_eq!(projection.child_process(), NodeChildProcess);

    projection.chdir("nested");
    assert_eq!(projection.process().cwd(), nested.as_path());
    assert_eq!(projection.fs().cwd(), nested.as_path());
    projection
        .fs()
        .write_text_file("relative.txt", "ok")
        .expect("write via chdir");
    assert_eq!(
        std::fs::read_to_string(nested.join("relative.txt")).expect("read via chdir"),
        "ok"
    );

    projection.process_mut().write_stdout("ok");
    assert_eq!(projection.process().stdout(), "ok");

    assert_eq!(
        NodePath::dirname("/tmp/project/src/main.ts"),
        PathBuf::from("/tmp/project/src")
    );
    assert_eq!(
        NodePath::relative("/tmp/project/src", "/tmp/project/lib/index.js"),
        PathBuf::from("../lib/index.js")
    );
    assert_eq!(NodePath::basename("/tmp/project/src/main.ts"), "main.ts");
    assert_eq!(NodePath::extname("/tmp/project/src/main.ts"), ".ts");
    assert_eq!(
        NodeCrypto::create_hash("sha384", "hello")
            .expect("hash")
            .len(),
        96
    );
}
