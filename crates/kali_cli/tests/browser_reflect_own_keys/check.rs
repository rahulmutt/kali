use super::*;

#[test]
fn check_accepts_reflect_own_keys_in_jsx_input_on_browser_surface() {
    assert_browser_checked_reflect_own_keys("main.jsx", false);
}

#[test]
fn check_accepts_reflect_own_keys_in_tsx_input_on_browser_surface() {
    assert_browser_checked_reflect_own_keys("main.tsx", false);
}

#[test]
fn json_check_accepts_reflect_own_keys_in_jsx_input_on_browser_surface() {
    assert_browser_checked_reflect_own_keys("main.jsx", true);
}

#[test]
fn json_check_accepts_reflect_own_keys_in_tsx_input_on_browser_surface() {
    assert_browser_checked_reflect_own_keys("main.tsx", true);
}
