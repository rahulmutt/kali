use crate::*;

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
