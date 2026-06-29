use super::*;

#[test]
fn test_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("test", "smoke.test.js");
}

#[test]
fn test_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("test", "smoke.test.ts");
}

#[test]
fn test_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("test", "smoke.test.jsx");
}

#[test]
fn test_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("test", "smoke.test.tsx");
}

#[test]
fn test_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.js", false);
}

#[test]
fn test_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.jsx", false);
}

#[test]
fn test_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.tsx", false);
}

#[test]
fn json_test_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.jsx", true);
}

#[test]
fn json_test_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.tsx", true);
}

#[test]
fn json_test_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("test", "smoke.test.js");
}

#[test]
fn json_test_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("test", "smoke.test.ts");
}

#[test]
fn json_test_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("test", "smoke.test.jsx");
}

#[test]
fn json_test_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("test", "smoke.test.tsx");
}
