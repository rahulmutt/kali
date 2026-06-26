use crate::*;

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
