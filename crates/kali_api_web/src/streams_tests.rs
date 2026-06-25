use crate::*;

#[test]
fn readable_stream_shares_state_and_closing_is_deterministic() {
    let stream = ReadableStream::from_chunks(["alpha".as_bytes(), "beta".as_bytes()]);
    let clone = stream.clone();

    clone.append_chunk("gamma");
    assert_eq!(stream.bytes(), b"alphabetagamma");
    assert_eq!(
        clone.chunks(),
        vec![b"alpha".to_vec(), b"beta".to_vec(), b"gamma".to_vec()]
    );

    stream.close();
    clone.append_chunk("delta");
    assert_eq!(stream.bytes(), b"alphabetagamma");
    assert!(clone.is_closed());
    assert!(stream.is_closed());
}

#[test]
fn writable_and_transform_streams_share_the_same_backing_state() {
    let writable = WritableStream::new();
    writable.write("hello ");
    writable.write_text("world");
    assert_eq!(
        writable.chunks(),
        vec![b"hello ".to_vec(), b"world".to_vec()]
    );
    assert_eq!(writable.text().expect("writable text"), "hello world");

    writable.close();
    writable.write("!");
    assert_eq!(writable.bytes(), b"hello world");
    assert!(writable.is_closed());

    let transform = TransformStream::new();
    transform.writable().write("left");
    transform.readable().append_chunk("-right");
    assert_eq!(transform.readable().bytes(), b"left-right");
    assert_eq!(transform.writable().bytes(), b"left-right");

    transform.readable().close();
    transform.writable().write("-ignored");
    assert_eq!(transform.readable().bytes(), b"left-right");
    assert!(transform.readable().is_closed());
    assert!(transform.writable().is_closed());
}

#[test]
fn text_encoder_and_decoder_streams_share_the_shared_baseline() {
    let encoder = TextEncoderStream::new();
    encoder.write_text("héllo 🌍");
    assert_eq!(encoder.readable().bytes(), "héllo 🌍".as_bytes());
    assert_eq!(encoder.writable().text().expect("encoder text"), "héllo 🌍");

    let decoder = TextDecoderStream::new();
    decoder.write("decoded ");
    decoder.writable().write_text("payload");
    assert_eq!(decoder.readable().bytes(), b"decoded payload");
    assert_eq!(
        decoder.readable().text().expect("decoder text"),
        "decoded payload"
    );
    assert_eq!(decoder.writable().bytes(), b"decoded payload");
}
