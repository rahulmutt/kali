use super::*;

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
