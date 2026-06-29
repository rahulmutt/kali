use super::*;

#[test]
fn run_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("run", "main.js");
}

#[test]
fn run_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("run", "main.ts");
}

#[test]
fn run_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("run", "main.jsx");
}

#[test]
fn run_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("run", "main.tsx");
}

#[test]
fn run_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.js", false);
}

#[test]
fn run_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.ts", false);
}

#[test]
fn run_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.jsx", false);
}

#[test]
fn run_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.tsx", false);
}

#[test]
fn json_run_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.js", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.ts", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.jsx", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.tsx", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("run", "main.js");
}

#[test]
fn json_run_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("run", "main.ts");
}

#[test]
fn json_run_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("run", "main.jsx");
}

#[test]
fn json_run_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("run", "main.tsx");
}
