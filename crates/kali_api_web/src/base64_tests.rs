use crate::*;

#[test]
fn base64_helpers_round_trip_binary_strings() {
    assert_eq!(btoa("hello").expect("encode"), "aGVsbG8=");
    assert_eq!(atob("aGVs bG8=").expect("decode"), "hello");
    assert_eq!(atob("aGVsbG8").expect("unpadded decode"), "hello");
}

#[test]
fn base64_helpers_reject_out_of_range_input() {
    assert!(btoa("€").is_err());
}

#[test]
fn base64_helpers_reject_malformed_input_lengths() {
    let error = atob("abcde").expect_err("malformed length");
    assert_eq!(
        error.to_string(),
        "The string to be decoded is not correctly encoded."
    );
}
