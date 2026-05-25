use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use serde_json::Value;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("kali binary path")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn start_registry_metadata_server(
    body: &'static str,
) -> (
    String,
    Arc<AtomicUsize>,
    Arc<AtomicBool>,
    thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind registry metadata server");
    listener.set_nonblocking(true).expect("set nonblocking");
    let addr = listener.local_addr().expect("registry metadata address");
    let hits = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let hits_thread = hits.clone();
    let stop_thread = stop.clone();

    let handle = thread::spawn(move || loop {
        if stop_thread.load(Ordering::SeqCst) {
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                hits_thread.fetch_add(1, Ordering::SeqCst);
                let mut buffer = [0u8; 1024];
                let _ = stream.read(&mut buffer);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
    });

    (
        format!("http://127.0.0.1:{}", addr.port()),
        hits,
        stop,
        handle,
    )
}

#[test]
fn json_package_effects_rejects_padded_package_argument_with_normalized_context() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("package-effects")
        .arg(" lodash ")
        .output()
        .expect("run kali");

    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-effects");
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["requestedValue"], " lodash ");
    assert_eq!(errors[0]["context"]["effectiveValue"], "lodash");
}

#[test]
fn json_package_audit_rejects_padded_package_argument_with_normalized_context() {
    let (registry_url, hits, stop, handle) =
        start_registry_metadata_server(r#"{"schemaVersion":1,"packages":[]}"#);

    let output = Command::new(kali_bin())
        .env("KALI_REGISTRY", registry_url)
        .arg("--output")
        .arg("json")
        .arg("package-audit")
        .arg(" lodash ")
        .output()
        .expect("run kali");

    stop.store(true, Ordering::SeqCst);
    handle.join().expect("join registry server");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        0,
        "registry should not be queried"
    );
    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert!(!json["success"].as_bool().expect("success boolean"));
    assert_eq!(json["exitCode"], 5);
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(errors[0]["context"]["origin"], "cli");
    assert_eq!(errors[0]["context"]["requestedValue"], " lodash ");
    assert_eq!(errors[0]["context"]["effectiveValue"], "lodash");
}
