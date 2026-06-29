use super::*;

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_js_input() {
    assert_browser_bundle_reflect_own_keys("app.js", false);
}

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_ts_input() {
    assert_browser_bundle_reflect_own_keys("app.ts", false);
}

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_jsx_input() {
    assert_browser_bundle_reflect_own_keys("app.jsx", false);
}

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_tsx_input() {
    assert_browser_bundle_reflect_own_keys("app.tsx", false);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_js_input() {
    assert_browser_bundle_reflect_own_keys("app.js", true);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_ts_input() {
    assert_browser_bundle_reflect_own_keys("app.ts", true);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_jsx_input() {
    assert_browser_bundle_reflect_own_keys("app.jsx", true);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_tsx_input() {
    assert_browser_bundle_reflect_own_keys("app.tsx", true);
}
