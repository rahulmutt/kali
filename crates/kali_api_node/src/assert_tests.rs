use crate::*;

#[test]
fn assert_helpers_produce_clear_results() {
    assert_eq!(NodeAssert::ok(true, "ok"), Ok(()));
    assert_eq!(NodeAssert::ok(false, "bad"), Err("bad".to_string()));
    assert_eq!(NodeAssert::equal(&3, &3, "equal"), Ok(()));
    assert_eq!(
        NodeAssert::equal(&3, &4, "mismatch"),
        Err("mismatch: expected 4, got 3".to_string())
    );
    assert_eq!(NodeAssert::not_equal(&3, &4, "not equal"), Ok(()));
    assert_eq!(
        NodeAssert::not_equal(&3, &3, "same"),
        Err("same: value unexpectedly matched 3".to_string())
    );
    assert_eq!(
        NodeAssert::deep_equal(&vec![1, 2], &vec![1, 2], "deep"),
        Ok(())
    );
    assert_eq!(NodeAssert::fail("boom"), Err("boom".to_string()));
}
