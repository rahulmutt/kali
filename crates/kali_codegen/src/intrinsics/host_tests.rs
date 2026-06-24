use crate::test_support::*;
use crate::*;
use wasmparser::Validator;

#[test]
fn console_member_calls_lower_to_console_host_imports() {
    let program = parse_and_lower_lir(
        "console.log(1); console.error(2); console.warn(3); console.info(4); console.debug(5);",
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
    assert!(printed.contains("import \"kali:rt\" \"console_log\""));
    assert!(printed.contains("import \"kali:rt\" \"console_error\""));
    assert!(printed.contains("import \"kali:rt\" \"console_warn\""));
    assert!(printed.contains("import \"kali:rt\" \"console_info\""));
    assert!(printed.contains("import \"kali:rt\" \"console_debug\""));
}

#[test]
fn console_assert_member_lowering_uses_console_error_for_falsey_conditions() {
    let program = parse_and_lower_lir("console.assert(1, 'assert failed');");
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
    assert!(printed.contains("import \"kali:rt\" \"console_error\""));
    assert!(printed.contains("i64.eqz"));
    assert!(printed.contains("i32.eqz"));
}

#[test]
fn process_argv_slice_length_with_non_default_start_lowers_to_runtime_args_length_minus_start() {
    let program = parse_and_lower_lir("console.log(process.argv.slice(1).length);");
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
    assert!(printed.contains("i64.const 1"));
    assert!(printed.contains("i64.sub"));
}

#[test]
fn process_argv_length_lowers_to_runtime_args_length_import() {
    let program = parse_and_lower_lir("console.log(process.argv.length);");
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
    assert!(printed.contains("import \"kali:rt\" \"args_len\""));
}

#[test]
fn deno_args_length_lowers_to_runtime_args_length_import() {
    let program = parse_and_lower_lir("console.log(Deno.args.length);");
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
    assert!(printed.contains("import \"kali:rt\" \"args_len\""));
}

#[test]
fn deno_args_slice_length_with_non_default_start_lowers_to_runtime_args_length_minus_start() {
    let program = parse_and_lower_lir("console.log(Deno.args.slice(1).length);");
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
    assert!(printed.contains("i64.const 1"));
    assert!(printed.contains("i64.sub"));
}

#[test]
fn deno_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(Deno.pid);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn global_this_deno_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(globalThis.Deno.pid);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn bracketed_global_this_deno_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(globalThis[\"Deno\"][\"pid\"]);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn process_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(process.pid);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn global_this_process_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(globalThis.process.pid);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn bracketed_global_this_process_pid_member_calls_lower_to_runtime_pid_import() {
    let program = parse_and_lower_lir("console.log(globalThis[\"process\"][\"pid\"]);");
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
    assert!(printed.contains("import \"kali:rt\" \"process_pid\""));
}

#[test]
fn deno_cwd_member_calls_lower_to_runtime_cwd_import() {
    let program = parse_and_lower_lir("console.log(Deno.cwd());");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"cwd\""));
}

#[test]
fn deno_chdir_member_calls_lower_to_runtime_cwd_set_import() {
    let program = parse_and_lower_lir("Deno.chdir(\"nested\");");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"cwd_set\""));
}

#[test]
fn process_cwd_member_calls_lower_to_runtime_cwd_import() {
    let program = parse_and_lower_lir("console.log(process.cwd());");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"cwd\""));
}

#[test]
fn bracketed_global_this_process_cwd_member_calls_lower_to_runtime_cwd_import() {
    let program = parse_and_lower_lir("console.log(globalThis[\"process\"][\"cwd\"]());");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"cwd\""));
}

#[test]
fn process_exit_member_calls_lower_to_runtime_process_exit_import() {
    let program = parse_and_lower_lir("process.exit(7);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("import \"kali:rt\" \"process_exit\""));
}

#[test]
fn process_kill_zero_probe_lowers_to_boolean_true_without_process_exit_import() {
    let program = parse_and_lower_lir("process.kill(0);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
    Validator::new()
        .validate_all(&result.wasm_bytes)
        .expect("generated wasm should validate");

    assert!(printed.contains("i64.const 1"), "printed wasm: {printed}");
    assert!(!printed.contains("process_exit"), "printed wasm: {printed}");
}

#[test]
fn process_kill_zero_probe_lowers_through_transparent_wrappers_without_process_exit_import() {
    for source in [
        "process.kill((0));",
        "process.kill(+0);",
        "Object.freeze(process.kill)(0);",
        "Object.freeze((process.kill))(0);",
        "Object.freeze((process.kill))(+0);",
        "Object.freeze(globalThis.process.kill)(0); Object.freeze(globalThis.process.kill)(+0);",
        "Object.freeze(globalThis.process[\"kill\"])(0);",
        "Object.freeze((globalThis[\"process\"][\"kill\"]))(0); Object.freeze((globalThis[\"process\"][\"kill\"]))(+0);",
        "Object.freeze(globalThis[\"process\"].kill)(0);",
        "Object.freeze(globalThis[\"process\"][\"kill\"])(0);",
        "Object.freeze(globalThis[\"process\"][\"kill\"])(+0);",
        "Object.freeze(process)[\"kill\"](0);",
        "Object.freeze(process)[\"kill\"](+0);",
        "Object.freeze((process)[\"kill\"])(0);",
        "Object.freeze((process)[\"kill\"])(+0);",
        "Object.freeze(globalThis.process)[\"kill\"](0);",
        "Object.freeze(globalThis.process)[\"kill\"](+0);",
        "Object.freeze(globalThis[\"process\"])[\"kill\"](0);",
        "Object.freeze(globalThis[\"process\"])[\"kill\"](+0);",
        "Object.freeze((process))[\"kill\"](0);",
        "Object.freeze((process))[\"kill\"](+0);",
        "Object.freeze((globalThis.process))[\"kill\"](0);",
        "Object.freeze((globalThis.process))[\"kill\"](+0);",
        "Object.freeze((globalThis[\"process\"]))[\"kill\"](0);",
        "Object.freeze((globalThis[\"process\"]))[\"kill\"](+0);",
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.is_empty(),
            "{source:?}: {:?}",
            result.diagnostics
        );
        let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");

        assert!(
            printed.contains("i64.const 1"),
            "{source:?} printed wasm: {printed}"
        );
        assert!(
            !printed.contains("process_exit"),
            "{source:?} printed wasm: {printed}"
        );
    }
}

#[test]
fn process_kill_zero_probe_through_wrapped_process_objects_lowers_without_process_exit_import() {
    for source in [
        "((process)).kill(0);",
        "((globalThis.process)).kill(0);",
        "((process.kill))(0);",
        "((process[\"kill\"]))(0);",
        "((globalThis.process[\"kill\"]))(0);",
        "((globalThis[\"process\"][\"kill\"]))(0);",
        "((globalThis[\"process\"].kill))(0);",
        "process[\"kill\"](0);",
        "Object.freeze((process))[\"kill\"](0);",
        "Object.freeze((process))[\"kill\"](+0);",
        "Object.freeze((globalThis.process))[\"kill\"](0);",
        "Object.freeze((globalThis.process))[\"kill\"](+0);",
        "Object.freeze((globalThis[\"process\"]))[\"kill\"](0);",
        "Object.freeze((globalThis[\"process\"]))[\"kill\"](+0);",
        "globalThis.process[\"kill\"](0);",
        "globalThis[\"process\"][\"kill\"](0);",
        "globalThis[\"process\"].kill(0);",
        "globalThis[\"process\"][\"kill\"](+0);",
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.is_empty(),
            "{source:?}: {:?}",
            result.diagnostics
        );
        let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");

        assert!(
            printed.contains("i64.const 1"),
            "{source:?} printed wasm: {printed}"
        );
        assert!(
            !printed.contains("process_exit"),
            "{source:?} printed wasm: {printed}"
        );
    }
}

#[test]
fn process_kill_zero_probe_through_static_zero_aliases_lowers_without_process_exit_import() {
    for source in [
        "const zero = 0; const zeroAlias = zero; process.kill(zeroAlias);",
        "const zero = 0; const zeroAlias = zero; globalThis.process.kill(+zero);",
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.is_empty(),
            "{source:?}: {:?}",
            result.diagnostics
        );
        let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");

        assert!(
            printed.contains("i64.const 1"),
            "{source:?} printed wasm: {printed}"
        );
        assert!(
            !printed.contains("process_exit"),
            "{source:?} printed wasm: {printed}"
        );
    }
}

#[test]
fn process_kill_zero_probe_through_transparent_type_wrappers_lowers_without_process_exit_import() {
    for source in [
        "process.kill((0 satisfies number));",
        "process.kill((0 as number));",
        "process.kill((0));",
    ] {
        let program = parse_and_lower_lir(source);
        let mut ctx = CodegenCtx::new(TargetConfig {
            max_specializations: 16,
            compat_eval: false,
            coverage: false,
        });
        let result = lower_lir_to_wasm(&mut ctx, &program);

        assert!(
            result.diagnostics.is_empty(),
            "{source:?}: {:?}",
            result.diagnostics
        );
        let printed = wasmprinter::print_bytes(&result.wasm_bytes).expect("print wasm");
        Validator::new()
            .validate_all(&result.wasm_bytes)
            .expect("generated wasm should validate");

        assert!(
            printed.contains("i64.const 1"),
            "{source:?} printed wasm: {printed}"
        );
        assert!(
            !printed.contains("process_exit"),
            "{source:?} printed wasm: {printed}"
        );
    }
}

#[test]
fn process_kill_non_zero_probe_is_rejected_by_codegen() {
    let program = parse_and_lower_lir("process.kill(1);");
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    let result = lower_lir_to_wasm(&mut ctx, &program);

    assert!(
        !result.diagnostics.is_empty(),
        "expected process.kill(1) to be rejected"
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("process.kill")),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains(r#"process["kill"](0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains(r#"globalThis["process"]["kill"](0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze(globalThis.process["kill"])(0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze(globalThis.process["kill"])(+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze(process)["kill"](+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze((process)["kill"])(0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze((process)["kill"])(+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze(globalThis.process)["kill"](+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze(globalThis["process"].kill)(0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze(globalThis["process"].kill)(+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"Object.freeze(globalThis["process"]["kill"])(+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains(r#"((process["kill"]))(0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains(r#"((process["kill"]))(+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"((globalThis.process["kill"]))(+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"((globalThis.process["kill"]))(+0)"#)),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().any(|diag| diag
            .message
            .contains(r#"((globalThis["process"]["kill"]))(+0)"#)),
        "{:?}",
        result.diagnostics
    );
}

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

#[test]
fn process_argv_slice_length_lowers_to_runtime_args_length_minus_start() {
    let program = parse_and_lower_lir("console.log(process.argv.slice(2).length);");
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
    assert!(printed.contains("import \"kali:rt\" \"args_len\""));
    assert!(printed.contains("i64.const 2"));
    assert!(printed.contains("i64.sub"));
}
