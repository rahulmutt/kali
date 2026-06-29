use super::*;

#[test]
fn test_supports_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("test", "smoke.test.js", false);
}

#[test]
fn test_supports_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested_frozen_set_map_constructor_result("test", filename, false);
    }
}

#[test]
fn json_test_supports_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested_frozen_set_map_constructor_result("test", filename, true);
    }
}

#[test]
fn test_supports_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("test", "smoke.test.js", false);
}

#[test]
fn test_supports_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested_frozen_object_helper_iteration_targets("test", filename, false);
    }
}

#[test]
fn json_test_supports_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested_frozen_object_helper_iteration_targets("test", filename, true);
    }
}

#[test]
fn test_supports_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
            "test", filename, false,
        );
    }
}

#[test]
fn json_test_supports_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
            "test", filename, true,
        );
    }
}
