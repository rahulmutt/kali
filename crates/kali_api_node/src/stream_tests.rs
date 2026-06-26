use crate::*;

#[test]
fn stream_helpers_concatenate_bytes() {
    let bytes = NodeStream::concat(b"hello ", b"world");
    assert_eq!(bytes, b"hello world");
    assert_eq!(NodeStream::from_utf8("abc"), b"abc");
    assert_eq!(NodeStream::from_bytes(vec![1, 2, 3]), vec![1, 2, 3]);
    assert_eq!(NodeStream::to_utf8(b"kali").expect("utf8"), "kali");
}
