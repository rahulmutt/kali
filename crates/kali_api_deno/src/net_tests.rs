use crate::*;
use std::io::{Read, Write};

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
