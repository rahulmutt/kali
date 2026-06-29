use super::*;

#[test]
fn build_emits_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_set_map_constructor_result("app.js", false);
}

#[test]
fn json_build_emits_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_set_map_constructor_result("app.js", true);
}

#[test]
fn build_emits_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_set_map_constructor_result("app.ts", false);
}

#[test]
fn json_build_emits_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_set_map_constructor_result("app.ts", true);
}

#[test]
fn build_emits_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_bundle_smoke_is_configured(
) {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_frozen_set_map_constructor_result(filename, false);
        assert_browser_bundle_frozen_set_map_constructor_result(filename, true);
    }
}

#[test]
fn build_emits_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_object_helper_iteration_targets("app.js", false);
}

#[test]
fn json_build_emits_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_object_helper_iteration_targets("app.js", true);
}

#[test]
fn build_emits_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_object_helper_iteration_targets("app.ts", false);
}

#[test]
fn json_build_emits_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_object_helper_iteration_targets("app.ts", true);
}

#[test]
fn build_emits_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_bundle_smoke_is_configured(
) {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_frozen_object_helper_iteration_targets(filename, false);
        assert_browser_bundle_frozen_object_helper_iteration_targets(filename, true);
    }
}

#[test]
fn build_emits_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_bundle_input_variants_when_configured(
) {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_parenthesized_frozen_set_map_constructor_result(filename, false);
    }
}

#[test]
fn json_build_emits_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_bundle_input_variants_when_configured(
) {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_parenthesized_frozen_set_map_constructor_result(filename, true);
    }
}
