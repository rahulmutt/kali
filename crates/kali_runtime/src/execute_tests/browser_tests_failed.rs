use super::super::browser_tests_failed;

#[test]
fn honest_summary_passthrough_is_never_overridden() {
    assert_eq!(browser_tests_failed(3, true, false), 3);
}

#[test]
fn crash_lane_with_no_reported_failures_counts_as_one_failure() {
    assert_eq!(browser_tests_failed(0, true, false), 1);
}

#[test]
fn clean_pass_stays_zero() {
    assert_eq!(browser_tests_failed(0, true, true), 0);
}

#[test]
fn non_test_lane_crash_is_untouched() {
    assert_eq!(browser_tests_failed(0, false, false), 0);
}

#[test]
fn reported_failures_with_clean_exit_pass_through() {
    assert_eq!(browser_tests_failed(2, true, true), 2);
}
