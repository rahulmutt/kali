use super::*;

#[test]
fn shared_array_buffer_clones_share_mutations() {
    let buffer = SharedArrayBuffer::from_bytes([1, 2, 3, 4]);
    let clone = buffer.clone();

    assert!(Atomics::is_lock_free());
    assert_eq!(buffer.byte_length(), 4);
    assert_eq!(buffer.snapshot(), vec![1, 2, 3, 4]);
    assert_eq!(Atomics::load(&clone, 1), Some(2));
    assert_eq!(Atomics::store(&clone, 1, 9), Some(2));
    assert_eq!(Atomics::add(&buffer, 0, 4), Some(1));
    assert_eq!(Atomics::and(&buffer, 0, 0b1111_1100), Some(5));
    assert_eq!(Atomics::or(&buffer, 1, 0b0000_0101), Some(9));
    assert_eq!(Atomics::xor(&buffer, 2, 0b0000_0110), Some(3));
    assert_eq!(Atomics::sub(&buffer, 2, 1), Some(5));
    assert_eq!(Atomics::compare_exchange(&buffer, 3, 4, 7), Some(Ok(4)));
    assert_eq!(Atomics::exchange(&clone, 0, 6), Some(4));
    assert_eq!(buffer.snapshot(), vec![6, 13, 4, 7]);
    assert_eq!(clone.snapshot(), vec![6, 13, 4, 7]);
}

#[test]
fn shared_array_buffer_compare_exchange_failure_leaves_bytes_unchanged() {
    let buffer = SharedArrayBuffer::from_bytes([10, 20]);

    assert_eq!(Atomics::compare_exchange(&buffer, 0, 11, 99), Some(Err(10)));
    assert_eq!(Atomics::snapshot(&buffer), vec![10, 20]);
}

#[test]
fn shared_array_buffer_supports_zero_length_buffers() {
    let buffer = SharedArrayBuffer::new(0);
    assert!(buffer.is_empty());
    assert!(Atomics::load(&buffer, 0).is_none());
    assert!(Atomics::store(&buffer, 0, 1).is_none());
    assert!(Atomics::and(&buffer, 0, 0xff).is_none());
    assert!(Atomics::or(&buffer, 0, 0xff).is_none());
    assert!(Atomics::xor(&buffer, 0, 0xff).is_none());
    assert!(Atomics::compare_exchange(&buffer, 0, 0, 1).is_none());
    assert!(Atomics::snapshot(&buffer).is_empty());
}
