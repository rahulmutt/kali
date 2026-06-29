use super::*;

#[test]
fn check_rejects_non_literal_for_of_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.js", false, "check", false);
}

#[test]
fn json_check_rejects_non_literal_for_of_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.js", true, "check", false);
}

#[test]
fn check_rejects_non_literal_for_of_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.jsx", false, "check", false);
}

#[test]
fn json_check_rejects_non_literal_for_of_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(for_of_source(), "main.jsx", true, "check", false);
}

#[test]
fn check_rejects_non_literal_for_await_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.ts", false, "check", false);
}

#[test]
fn json_check_rejects_non_literal_for_await_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.ts", true, "check", false);
}

#[test]
fn check_rejects_non_literal_for_await_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.tsx", false, "check", false);
}

#[test]
fn json_check_rejects_non_literal_for_await_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(for_await_source(), "main.tsx", true, "check", false);
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.js", false, "check", false);
}

#[test]
fn check_rejects_non_literal_object_values_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        object_values_source(),
        "main.js",
        false,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.js",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_non_literal_object_keys_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.js", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_values_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(object_values_source(), "main.js", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_entries_iterator_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.js",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.ts", false, "check", false);
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.ts",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_non_literal_object_keys_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.ts", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_entries_iterator_source_in_ts_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.ts",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.jsx", false, "check", false);
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.jsx",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_non_literal_object_keys_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.jsx", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_entries_iterator_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.jsx",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.tsx", false, "check", false);
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.tsx",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_non_literal_object_keys_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(object_keys_source(), "main.tsx", true, "check", false);
}

#[test]
fn json_check_rejects_non_literal_object_entries_iterator_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(
        object_entries_source(),
        "main.tsx",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_array_callback_iteration_from_call_expression_source_in_browser_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        for json_output in [false, true] {
            assert_browser_array_callback_iteration_source_rejects(
                array_callback_iteration_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}

#[test]
fn check_rejects_array_callback_iteration_from_call_expression_source_under_inherited_browser_config(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        for json_output in [false, true] {
            assert_inherited_browser_array_callback_iteration_source_rejects(
                array_callback_iteration_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}

#[test]
fn check_rejects_set_constructor_iteration_from_call_expression_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.js",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_set_constructor_iteration_from_call_expression_source_in_js_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.js",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_set_constructor_iteration_from_call_expression_source_in_ts_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.ts",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_set_constructor_iteration_from_call_expression_source_in_ts_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.ts",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_set_constructor_iteration_from_call_expression_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.jsx",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_set_constructor_iteration_from_call_expression_source_in_jsx_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.jsx",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_set_constructor_iteration_from_call_expression_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.tsx",
        false,
        "check",
        false,
    );
}

#[test]
fn json_check_rejects_set_constructor_iteration_from_call_expression_source_in_tsx_input() {
    assert_browser_iterator_source_rejects(
        set_constructor_call_expression_source(),
        "main.tsx",
        true,
        "check",
        false,
    );
}

#[test]
fn check_rejects_map_constructor_iteration_from_call_expression_source_in_browser_input() {
    assert_map_constructor_iteration_from_call_expression_source_rejects("check", false);
}

#[test]
fn check_rejects_map_constructor_iteration_from_call_expression_source_under_inherited_browser_config(
) {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                map_constructor_call_expression_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}

#[test]
fn check_rejects_non_literal_object_keys_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_keys_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}

#[test]
fn check_rejects_non_literal_object_values_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_values_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}

#[test]
fn check_rejects_non_literal_object_entries_iterator_source_under_inherited_browser_config() {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                object_entries_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}

#[test]
fn check_rejects_set_constructor_iteration_from_call_expression_source_under_inherited_browser_config(
) {
    for json_output in [false, true] {
        for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            assert_inherited_browser_iterator_source_rejects(
                set_constructor_call_expression_source(),
                filename,
                json_output,
                "check",
                false,
            );
        }
    }
}
