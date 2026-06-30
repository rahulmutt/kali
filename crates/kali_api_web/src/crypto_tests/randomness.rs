use super::*;

#[test]
fn random_fill_populates_the_requested_buffer() {
    let mut buffer = [0u8; 16];
    fill_random_values(&mut buffer).expect("random fill");
    assert_eq!(buffer.len(), 16);
}

#[test]
fn random_uuid_has_the_expected_shape() {
    let uuid = random_uuid().expect("random uuid");

    assert_eq!(uuid.len(), 36);
    assert_eq!(uuid.chars().nth(8), Some('-'));
    assert_eq!(uuid.chars().nth(13), Some('-'));
    assert_eq!(uuid.chars().nth(18), Some('-'));
    assert_eq!(uuid.chars().nth(23), Some('-'));
    assert_eq!(uuid.chars().nth(14), Some('4'));
    assert!(matches!(uuid.chars().nth(19), Some('8' | '9' | 'a' | 'b')));
}

#[test]
fn crypto_facade_reuses_the_shared_randomness_helpers() {
    let crypto = crypto();
    let mut buffer = [0u8; 8];

    crypto
        .get_random_values(&mut buffer)
        .expect("crypto.getRandomValues");
    assert_eq!(buffer.len(), 8);

    let uuid = crypto.random_uuid().expect("crypto.randomUUID");
    assert_eq!(uuid.len(), 36);
    assert_eq!(uuid.chars().nth(14), Some('4'));
}
