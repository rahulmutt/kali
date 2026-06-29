use super::*;

#[test]
fn deno_env_get_member_calls_lower_to_runtime_env_get_import() {
    let program = parse_and_lower_lir("console.log(Deno.env.get(\"HOME\"));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"env_get\""));
    assert!(
        printed.contains("i32.const 4096"),
        "printed wasm: {printed}"
    );
}

#[test]
fn deno_env_has_member_calls_lower_to_runtime_env_has_import() {
    let program = parse_and_lower_lir("console.log(Deno.env.has(\"HOME\"));");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"env_has\""));
}

#[test]
fn deno_env_set_member_calls_lower_to_runtime_env_set_import() {
    let program =
        parse_and_lower_lir("Deno.env.set(\"KALI_ENV_SET_SMOKE\", \"hello-environment\");");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"env_set\""));
}

#[test]
fn bracketed_deno_env_set_member_calls_lower_to_runtime_env_set_import() {
    let program = parse_and_lower_lir(
        "Deno[\"env\"][\"set\"](\"KALI_ENV_SET_SMOKE\", \"hello-environment\");",
    );
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"env_set\""));
}

#[test]
fn deno_env_delete_member_calls_lower_to_runtime_env_delete_import() {
    let program = parse_and_lower_lir("Deno.env.delete(\"KALI_ENV_SET_SMOKE\");");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"env_delete\""));
}

#[test]
fn mixed_global_this_deno_env_delete_member_calls_lower_to_runtime_env_delete_import() {
    let program =
        parse_and_lower_lir("globalThis.Deno[\"env\"][\"delete\"](\"KALI_ENV_SET_SMOKE\");");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    assert!(printed.contains("import \"kali:rt\" \"env_delete\""));
}
