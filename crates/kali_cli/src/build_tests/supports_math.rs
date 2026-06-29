use super::*;

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_js_input() {
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_ts_input() {
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_browser_api_surface_in_js_input()
{
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_browser_api_surface_in_ts_input()
{
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_js_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_js_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_ts_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_jsx_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_tsx_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_ts_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_jsx_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_tsx_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_js_input() {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_ts_input() {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_browser_api_surface_in_js_input()
{
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_browser_api_surface_in_ts_input()
{
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_jsx_input() {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_tsx_input() {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_browser_api_surface_in_jsx_input()
{
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_browser_api_surface_in_tsx_input()
{
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_js_input(
) {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_ts_input(
) {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_js_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_ts_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_jsx_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_tsx_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_js_input() {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_ts_input() {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_jsx_input() {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_tsx_input() {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_js_input() {
    assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_ts_input() {
    assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_inverse_trig_identity_literals_in_js_input() {
    assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_inverse_trig_identity_literals_in_ts_input() {
    assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_inverse_trig_identity_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_inverse_trig_identity_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_as_const_wrappers_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Deno,
        "ts",
        "const zero = (0 as const); const one = (1 as const); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_satisfies_wrappers_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Deno,
        "ts",
        "const zero = (0 satisfies number); const one = (1 satisfies number); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_as_const_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Browser,
        "ts",
        "const zero = (0 as const); const one = (1 as const); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_satisfies_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Browser,
        "ts",
        "const zero = (0 satisfies number); const one = (1 satisfies number); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_await_wrappers_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Deno,
        "js",
        "async function main() {\n  const zero = await 0;\n  const one = await 1;\n  console.log(Math.atan2(zero, one));\n}\nmain();\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_await_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Browser,
        "ts",
        "async function main() {\n  const zero = await 0;\n  const one = await 1;\n  console.log(globalThis[\"Math\"].atan2(zero, one));\n}\nmain();\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_jsx_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_tsx_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_js_input() {
    assert_build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_ts_input() {
    assert_build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_js_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_ts_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_jsx_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_tsx_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_supports_math_exp_and_log_const_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_exp_and_log_const_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_exp_and_log_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_exp_and_log_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_js_input() {
    assert_build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_pow_frozen_callable_alias_inventory_in_browser_api_surface_in_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            math_pow_browser_alias_inventory_build_source(),
        )
        .expect("write source");

        let output = build_source_file(
            &source_path,
            BuildMode::Fast,
            ApiSurface::Browser,
            false,
            &[],
            16,
            None,
            None,
        )
        .expect("math.pow frozen callable alias inventory build should succeed");

        Validator::new()
            .validate_all(&output.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

#[test]
fn build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}
