use super::*;

#[test]
fn supported_static_string_search_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log(\"hello\".includes(\"ell\")); console.log(\"hello\".includes(\"e\", -4)); console.log(\"hello\".indexOf(\"l\", 3)); console.log(\"hello\".indexOf(\"e\", -2)); console.log(\"hello\".lastIndexOf(\"l\")); console.log(\"hello\".lastIndexOf(\"l\", 2)); console.log(\"hello\".lastIndexOf(\"l\", -1));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("i64.const 1"), "{printed}");
    assert!(printed.contains("i64.const 3"), "{printed}");
    assert!(printed.contains("i64.const 2"), "{printed}");
    assert!(printed.contains("i64.const -1"), "{printed}");
}

#[test]
fn supported_static_string_prefix_suffix_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log(\"hello\".startsWith(\"he\")); console.log(\"hello\".startsWith(\"ll\", 2)); console.log(\"hello\".endsWith(\"lo\")); console.log(\"hello\".endsWith(\"ell\", 4)); console.log(\"hello\".endsWith(\"he\", 4));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    // 4 from this program's own lowering + 3 each from the synthetic `__join`
    // (Spec 3) and its `__join_arena` twin (Spec 7 Task 4c) — both present in
    // every module with identical bodies: two loop-increment `i += 1`s and the
    // `n - 1` separator-count term all use `i64.const 1`. + 3 each from the
    // growable-join synthetics `__join_growable_i64` / `__join_growable_str`
    // (Task 5, same three `i64.const 1` sites, also always present). + 4 from
    // `__streq` (throw-fallout Stage 1, also present in every module): the
    // identical-handles return, the len==0 return, the loop-increment
    // `i += 1`, and the all-bytes-equal result.
    // + the Stage P4 Task 4 URLSearchParams scan synthetics (also present in
    // every module): `__usp_get` = 1 (the `__streq`-match `== 1` compare);
    // `__usp_has` = 2 (the compare + the `return 1` match); `__usp_getall` = 4
    // (two passes × [compare + `count += 1`]); `__usp_set` = 3 (the compare +
    // `found = 1`, PLUS `.matches` counts the `i64.const 16` grow term
    // `cap*2*8` as a SUBSTRING of "i64.const 1"). Total new = 1+2+4+3 = 10.
    assert_eq!(
        printed.matches("i64.const 1").count(),
        4 + 3 + 3 + 3 + 3 + 4 + 1 + 2 + 4 + 3,
        "{printed}"
    );
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_static_string_search_lowers_omitted_search_as_undefined() {
    let program = parse_and_lower_lir(
        "console.log('hello'.includes()); console.log('undefined value'.startsWith()); console.log('value undefined'.endsWith()); console.log('hello'.indexOf()); console.log('value undefined'.lastIndexOf());",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    // 2 from this program's own lowering + 3 each from the synthetic `__join`
    // (Spec 3) and its `__join_arena` twin (Spec 7 Task 4c) — both present in
    // every module with identical bodies: two loop-increment `i += 1`s and the
    // `n - 1` separator-count term all use `i64.const 1`. + 3 each from the
    // growable-join synthetics `__join_growable_i64` / `__join_growable_str`
    // (Task 5, same three `i64.const 1` sites, also always present). + 4 from
    // `__streq` (throw-fallout Stage 1, also present in every module): the
    // identical-handles return, the len==0 return, the loop-increment
    // `i += 1`, and the all-bytes-equal result.
    // + the Stage P4 Task 4 URLSearchParams scan synthetics (also present in
    // every module, module-invariant bodies): `__usp_get` = 1, `__usp_has` = 2,
    // `__usp_getall` = 4, `__usp_set` = 3 (incl. the `i64.const 16` grow term
    // counted as a "i64.const 1" SUBSTRING by `.matches`). Total new = 10.
    assert_eq!(
        printed.matches("i64.const 1").count(),
        2 + 3 + 3 + 3 + 3 + 4 + 1 + 2 + 4 + 3,
        "{printed}"
    );
    assert!(printed.contains("i64.const 6"), "{printed}");
    assert!(printed.contains("i64.const -1"), "{printed}");
    assert!(printed.contains("i64.const 0"), "{printed}");
}

#[test]
fn supported_static_string_length_lowers_static_string_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.length); console.log('hé'.length); console.log(Object.freeze('world').length);",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("\"5\""), "{printed}");
    assert!(printed.contains("\"2\""), "{printed}");
}

#[test]
fn supported_static_string_slice_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.slice(1)); console.log('hello'.slice(1, 4)); console.log('hello'.slice(1.5, 4.9)); console.log('hello'.slice(-4, -1)); console.log('hello'.slice(4, 1));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("ello"), "{printed}");
    assert!(printed.contains("ell"), "{printed}");
    assert!(printed.contains("el"), "{printed}");
}

#[test]
fn supported_static_string_substring_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.substring(1)); console.log('hello'.substring(1, 4)); console.log('hello'.substring(1.5, 4.9)); console.log('hello'.substring(4, 1)); console.log('hello'.substring(-2, 2));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("ello"), "{printed}");
    assert!(printed.contains("ell"), "{printed}");
    assert!(printed.contains("he"), "{printed}");
}

#[test]
fn supported_static_string_at_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.at()); console.log('hello'.at(1)); console.log('hello'.at(-1)); console.log('hello'.at(99));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("\"h\""), "{printed}");
    assert!(printed.contains("\"e\""), "{printed}");
    assert!(printed.contains("\"o\""), "{printed}");
    assert!(printed.contains("undefined"), "{printed}");
}

#[test]
fn supported_static_string_char_at_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.charAt()); console.log('hello'.charAt(1)); console.log('hello'.charAt(-1)); console.log('hello'.charAt(99));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("\"h\""), "{printed}");
    assert!(printed.contains("\"e\""), "{printed}");
    assert!(printed.contains("\"\""), "{printed}");
}

#[test]
fn supported_static_string_char_code_at_lowers_ascii_literals() {
    let program = parse_and_lower_lir(
        "console.log('hello'.charCodeAt()); console.log('hello'.charCodeAt(1)); console.log('hello'.charCodeAt(-1)); console.log('hello'.charCodeAt(99));",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("104"), "{printed}");
    assert!(printed.contains("101"), "{printed}");
    assert!(printed.contains("NaN"), "{printed}");
}
