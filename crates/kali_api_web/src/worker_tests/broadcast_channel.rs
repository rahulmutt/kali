use super::*;

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
