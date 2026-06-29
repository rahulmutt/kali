use super::*;

#[test]
fn run_supports_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("run", "main.js", false);
}

#[test]
fn run_supports_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("run", "main.ts", false);
}

#[test]
fn run_supports_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_requested_frozen_set_map_constructor_result("run", filename, false);
    }
}

#[test]
fn json_run_supports_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("run", "main.js", true);
}

#[test]
fn json_run_supports_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("run", "main.ts", true);
}

#[test]
fn json_run_supports_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_requested_frozen_set_map_constructor_result("run", filename, true);
    }
}

#[test]
fn run_supports_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("run", "main.js", false);
}

#[test]
fn run_supports_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("run", "main.ts", false);
}

#[test]
fn run_supports_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_requested_frozen_object_helper_iteration_targets("run", filename, false);
    }
}

#[test]
fn json_run_supports_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("run", "main.js", true);
}

#[test]
fn json_run_supports_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("run", "main.ts", true);
}

#[test]
fn json_run_supports_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_requested_frozen_object_helper_iteration_targets("run", filename, true);
    }
}

#[test]
fn run_supports_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
            "run", filename, false,
        );
    }
}

#[test]
fn json_run_supports_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
            "run", filename, true,
        );
    }
}
