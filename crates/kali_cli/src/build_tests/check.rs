use super::*;

#[test]
fn check_source_file_supports_number_predicates_in_deno_and_browser_ts_js_jsx_and_tsx_input() {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["ts", "js", "jsx", "tsx"] {
            assert_check_source_file_supports_number_predicates_in_input(api_surface, extension);
        }
    }
}

#[test]
fn check_source_file_supports_set_constructor_iteration_in_deno_and_browser_js_ts_jsx_and_tsx_input(
) {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["js", "ts", "jsx", "tsx"] {
            assert_check_source_file_supports_set_constructor_iteration_in_input(
                api_surface,
                extension,
            );
        }
    }
}

#[test]
fn check_source_file_supports_map_constructor_iteration_in_deno_and_browser_js_ts_jsx_and_tsx_input(
) {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["js", "ts", "jsx", "tsx"] {
            assert_check_source_file_supports_map_constructor_iteration_in_input(
                api_surface,
                extension,
            );
        }
    }
}

#[test]
fn check_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_ts_input(
) {
    assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_jsx_input(
) {
    assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_tsx_input(
) {
    assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_check_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn check_source_file_supports_object_helper_nullish_logical_iterator_slices_in_browser_api_surface_in_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_check_source_file_supports_object_helper_nullish_logical_iterator_slices_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn check_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_browser_api_surface_in_ts_jsx_and_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        assert_check_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn check_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_deno_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_check_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
            ApiSurface::Deno,
            extension,
        );
    }
}

#[test]
fn check_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_check_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn check_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_browser_api_surface_in_ts_jsx_and_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        assert_check_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn check_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_deno_js_and_ts_input()
{
    for extension in ["js", "ts"] {
        assert_check_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_input(
            ApiSurface::Deno,
            extension,
        );
    }
}

#[test]
fn check_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_browser_api_surface_in_js_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_check_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn check_source_file_supports_for_of_object_keys_const_bound_iterable_in_browser_api_surface_in_js_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_check_source_file_supports_for_of_object_keys_const_bound_iterable_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn check_source_file_rejects_array_callback_iteration_in_ts_input() {
    assert_check_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn check_source_file_rejects_array_callback_iteration_in_js_input() {
    assert_check_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Deno, "js");
}

#[test]
fn check_source_file_rejects_array_callback_iteration_in_browser_ts_input() {
    assert_check_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn check_source_file_rejects_array_callback_iteration_in_browser_js_input() {
    assert_check_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Browser, "js");
}

#[test]
fn check_source_file_rejects_array_callback_iteration_in_browser_jsx_input() {
    assert_check_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn check_source_file_rejects_array_callback_iteration_in_browser_tsx_input() {
    assert_check_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_browser_js_input() {
    assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_browser_ts_input() {
    assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_browser_jsx_input() {
    assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_browser_tsx_input() {
    assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn check_source_file_rejects_generator_functions_in_ts_input() {
    assert_check_source_file_rejects_generator_lowering_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn check_source_file_rejects_generator_functions_in_js_input() {
    assert_check_source_file_rejects_generator_lowering_in_input(ApiSurface::Deno, "js");
}

#[test]
fn check_source_file_rejects_generator_functions_in_jsx_input() {
    assert_check_source_file_rejects_generator_lowering_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn check_source_file_rejects_generator_functions_in_tsx_input() {
    assert_check_source_file_rejects_generator_lowering_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn check_source_file_rejects_async_generator_functions_in_ts_input() {
    assert_check_source_file_rejects_async_generator_lowering_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn check_source_file_rejects_async_generator_functions_in_js_input() {
    assert_check_source_file_rejects_async_generator_lowering_in_input(ApiSurface::Deno, "js");
}

#[test]
fn check_source_file_rejects_async_generator_functions_in_jsx_input() {
    assert_check_source_file_rejects_async_generator_lowering_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn check_source_file_rejects_async_generator_functions_in_tsx_input() {
    assert_check_source_file_rejects_async_generator_lowering_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn check_source_file_rejects_generator_functions_in_browser_ts_input() {
    assert_check_source_file_rejects_generator_lowering_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn check_source_file_rejects_generator_functions_in_browser_js_input() {
    assert_check_source_file_rejects_generator_lowering_in_input(ApiSurface::Browser, "js");
}

#[test]
fn check_source_file_rejects_generator_functions_in_browser_jsx_input() {
    assert_check_source_file_rejects_generator_lowering_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn check_source_file_rejects_generator_functions_in_browser_tsx_input() {
    assert_check_source_file_rejects_generator_lowering_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn check_source_file_rejects_async_generator_functions_in_browser_ts_input() {
    assert_check_source_file_rejects_async_generator_lowering_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn check_source_file_rejects_async_generator_functions_in_browser_js_input() {
    assert_check_source_file_rejects_async_generator_lowering_in_input(ApiSurface::Browser, "js");
}

#[test]
fn check_source_file_rejects_async_generator_functions_in_browser_jsx_input() {
    assert_check_source_file_rejects_async_generator_lowering_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn check_source_file_rejects_async_generator_functions_in_browser_tsx_input() {
    assert_check_source_file_rejects_async_generator_lowering_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn check_source_file_rejects_class_generator_methods_in_deno_and_browser_input() {
    assert_check_source_file_rejects_class_generator_methods_in_input(ApiSurface::Deno);
    assert_check_source_file_rejects_class_generator_methods_in_input(ApiSurface::Browser);
}
