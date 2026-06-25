use crate::*;

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
