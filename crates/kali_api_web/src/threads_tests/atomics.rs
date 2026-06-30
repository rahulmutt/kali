use super::*;

#[test]
fn atomics_reports_lock_free_status_deterministically() {
    let first = Atomics::is_lock_free();
    let second = Atomics::is_lock_free();

    assert_eq!(first, bytewise_shared_memory_is_lock_free());
    assert_eq!(first, second);
}
