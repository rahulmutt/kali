use super::*;

#[test]
fn build_source_file_supports_for_of_identifier_binding_in_ts_input() {
    assert_build_source_file_supports_for_of_identifier_binding_in_input("ts");
}

#[test]
fn build_source_file_supports_for_of_identifier_binding_in_js_input() {
    assert_build_source_file_supports_for_of_identifier_binding_in_input("js");
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_jsx_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_tsx_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_deno_js_and_ts_input()
{
    for extension in ["js", "ts"] {
        assert_build_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_input(
            ApiSurface::Deno,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_browser_api_surface_in_js_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_for_of_object_keys_const_bound_iterable_in_browser_api_surface_in_js_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_source_file_supports_for_of_object_keys_const_bound_iterable_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_let_binding_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_let_binding_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_let_binding_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_let_binding_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_let_binding_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_let_binding_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_let_binding_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_let_binding_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_string_concatenation_iteration_in_js_input() {
    assert_build_source_file_supports_for_await_string_concatenation_iteration_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_string_concatenation_iteration_in_ts_input() {
    assert_build_source_file_supports_for_await_string_concatenation_iteration_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_string_concatenation_iteration_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_string_concatenation_iteration_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_string_concatenation_iteration_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_string_concatenation_iteration_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_string_concatenation_iteration_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_await_string_concatenation_iteration_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_string_concatenation_iteration_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_await_string_concatenation_iteration_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_string_concatenation_iteration_in_js_input() {
    assert_build_source_file_supports_for_of_string_concatenation_iteration_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_template_literal_string_iteration_in_js_input() {
    assert_build_source_file_supports_for_of_template_literal_string_iteration_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_string_concatenation_iteration_in_ts_input() {
    assert_build_source_file_supports_for_of_string_concatenation_iteration_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_template_literal_string_iteration_in_ts_input() {
    assert_build_source_file_supports_for_of_template_literal_string_iteration_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_string_concatenation_iteration_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_string_concatenation_iteration_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_template_literal_string_iteration_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_template_literal_string_iteration_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_string_concatenation_iteration_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_string_concatenation_iteration_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_string_concatenation_iteration_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_of_string_concatenation_iteration_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_of_string_concatenation_iteration_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_of_string_concatenation_iteration_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_of_template_literal_string_iteration_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_template_literal_string_iteration_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_js_input()
{
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_ts_input()
{
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_jsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_tsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_jsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_tsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}
