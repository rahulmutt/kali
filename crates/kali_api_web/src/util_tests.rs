use crate::*;

#[test]
fn performance_now_is_monotonic_and_non_negative() {
    let first = performance_now();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let second = performance_now();

    assert!(first >= 0.0, "first timestamp: {first}");
    assert!(
        second >= first,
        "timestamps should not go backwards: {first} -> {second}"
    );
}

#[test]
fn text_codec_round_trips_unicode() {
    let input = "héllo 🌍";
    let encoded = text_encode(input);
    assert_eq!(encoded, input.as_bytes());
    let decoded = text_decode(&encoded).expect("valid utf-8");
    assert_eq!(decoded, input);
}

#[test]
fn structured_clone_copies_values() {
    let original = vec![1, 2, 3];
    let cloned = structured_clone(&original);
    assert_eq!(cloned, original);
}
