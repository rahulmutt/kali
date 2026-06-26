use crate::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

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
