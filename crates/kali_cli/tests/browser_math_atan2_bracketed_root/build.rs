use super::*;

#[test]
fn build_emits_bracketed_global_this_math_atan2_zero_slice_in_js_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2("app.js", false);
}

#[test]
fn build_emits_bracketed_global_this_math_atan2_zero_slice_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2("app.ts", false);
}

#[test]
fn build_emits_bracketed_global_this_math_atan2_zero_slice_in_jsx_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2("app.jsx", false);
}

#[test]
fn build_emits_bracketed_global_this_math_atan2_zero_slice_in_tsx_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2("app.tsx", false);
}

#[test]
fn json_build_emits_bracketed_global_this_math_atan2_zero_slice_in_js_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2("app.js", true);
}

#[test]
fn json_build_emits_bracketed_global_this_math_atan2_zero_slice_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2("app.ts", true);
}

#[test]
fn json_build_emits_bracketed_global_this_math_atan2_zero_slice_in_jsx_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2("app.jsx", true);
}

#[test]
fn json_build_emits_bracketed_global_this_math_atan2_zero_slice_in_tsx_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2("app.tsx", true);
}

#[test]
fn build_emits_bracketed_global_this_math_atan2_as_const_wrapper_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2_wrapper(
        "app.ts",
        false,
        "// kali-tree-shake: bracketedGlobalThisMathAtan2AsConstWrapper\nfunction bracketedGlobalThisMathAtan2AsConstWrapper() {\n  const zero = (0 as const);\n  const one = (1 as const);\n  console.log(globalThis[\"Math\"].atan2(zero, one));\n  return globalThis[\"Math\"].atan2(zero, one);\n}\n",
        "bracketedGlobalThisMathAtan2AsConstWrapper();",
    );
}

#[test]
fn json_build_emits_bracketed_global_this_math_atan2_as_const_wrapper_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2_wrapper(
        "app.ts",
        true,
        "// kali-tree-shake: bracketedGlobalThisMathAtan2AsConstWrapper\nfunction bracketedGlobalThisMathAtan2AsConstWrapper() {\n  const zero = (0 as const);\n  const one = (1 as const);\n  console.log(globalThis[\"Math\"].atan2(zero, one));\n  return globalThis[\"Math\"].atan2(zero, one);\n}\n",
        "bracketedGlobalThisMathAtan2AsConstWrapper();",
    );
}

#[test]
fn build_emits_bracketed_global_this_math_atan2_satisfies_wrapper_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2_wrapper(
        "app.ts",
        false,
        "// kali-tree-shake: bracketedGlobalThisMathAtan2SatisfiesWrapper\nfunction bracketedGlobalThisMathAtan2SatisfiesWrapper() {\n  const zero = (0 satisfies number);\n  const one = (1 satisfies number);\n  console.log(globalThis[\"Math\"].atan2(zero, one));\n  return globalThis[\"Math\"].atan2(zero, one);\n}\n",
        "bracketedGlobalThisMathAtan2SatisfiesWrapper();",
    );
}

#[test]
fn json_build_emits_bracketed_global_this_math_atan2_satisfies_wrapper_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2_wrapper(
        "app.ts",
        true,
        "// kali-tree-shake: bracketedGlobalThisMathAtan2SatisfiesWrapper\nfunction bracketedGlobalThisMathAtan2SatisfiesWrapper() {\n  const zero = (0 satisfies number);\n  const one = (1 satisfies number);\n  console.log(globalThis[\"Math\"].atan2(zero, one));\n  return globalThis[\"Math\"].atan2(zero, one);\n}\n",
        "bracketedGlobalThisMathAtan2SatisfiesWrapper();",
    );
}

#[test]
fn build_emits_bracketed_global_this_math_atan2_bracketed_method_in_js_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2_bracketed_method("app.js", false);
}

#[test]
fn build_emits_bracketed_global_this_math_atan2_bracketed_method_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2_bracketed_method("app.ts", false);
}

#[test]
fn json_build_emits_bracketed_global_this_math_atan2_bracketed_method_in_js_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2_bracketed_method("app.js", true);
}

#[test]
fn json_build_emits_bracketed_global_this_math_atan2_bracketed_method_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_atan2_bracketed_method("app.ts", true);
}

#[test]
fn build_emits_single_quoted_global_this_math_atan2_zero_slice_in_js_input() {
    assert_browser_bundle_single_quoted_global_this_math_atan2("app.js", false);
}

#[test]
fn build_emits_single_quoted_global_this_math_atan2_zero_slice_in_ts_input() {
    assert_browser_bundle_single_quoted_global_this_math_atan2("app.ts", false);
}

#[test]
fn build_emits_single_quoted_global_this_math_atan2_zero_slice_in_jsx_input() {
    assert_browser_bundle_single_quoted_global_this_math_atan2("app.jsx", false);
}

#[test]
fn build_emits_single_quoted_global_this_math_atan2_zero_slice_in_tsx_input() {
    assert_browser_bundle_single_quoted_global_this_math_atan2("app.tsx", false);
}

#[test]
fn json_build_emits_single_quoted_global_this_math_atan2_zero_slice_in_js_input() {
    assert_browser_bundle_single_quoted_global_this_math_atan2("app.js", true);
}

#[test]
fn json_build_emits_single_quoted_global_this_math_atan2_zero_slice_in_ts_input() {
    assert_browser_bundle_single_quoted_global_this_math_atan2("app.ts", true);
}

#[test]
fn json_build_emits_single_quoted_global_this_math_atan2_zero_slice_in_jsx_input() {
    assert_browser_bundle_single_quoted_global_this_math_atan2("app.jsx", true);
}

#[test]
fn json_build_emits_single_quoted_global_this_math_atan2_zero_slice_in_tsx_input() {
    assert_browser_bundle_single_quoted_global_this_math_atan2("app.tsx", true);
}
