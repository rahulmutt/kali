use super::*;

#[test]
fn test_rejects_array_callback_iteration_from_call_expression_source_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for json_output in [false, true] {
        for filename in [
            "smoke.test.js",
            "smoke.test.ts",
            "smoke.test.jsx",
            "smoke.test.tsx",
        ] {
            assert_browser_requested_array_callback_iteration_source_rejects(
                array_callback_iteration_source(),
                filename,
                json_output,
                "test",
            );
        }
    }
}

#[test]
fn test_rejects_non_literal_set_and_map_constructor_iteration_from_call_expression_source_in_browser_api_surface_with_harness_js_input(
) {
    for source in [
        set_constructor_call_expression_source(),
        map_constructor_call_expression_source(),
    ] {
        assert_browser_requested_iterator_source_rejects(source, "smoke.test.js", false, "test");
    }
}

#[test]
fn json_test_rejects_non_literal_set_and_map_constructor_iteration_from_call_expression_source_in_browser_api_surface_with_harness_js_input(
) {
    for source in [
        set_constructor_call_expression_source(),
        map_constructor_call_expression_source(),
    ] {
        assert_browser_requested_iterator_source_rejects(source, "smoke.test.js", true, "test");
    }
}
