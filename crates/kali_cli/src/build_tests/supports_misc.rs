use super::*;

#[test]
fn build_source_file_supports_deno_env_get_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE'));",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE')); console.log(Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno.env[\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_get_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "console.log(Deno.env.get('KALI_ENV_GET_SMOKE')); console.log(Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno.env[\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));",
        )
        .expect("write source");

        let output = build_source_file(
            &source_path,
            BuildMode::Fast,
            ApiSurface::Deno,
            false,
            &[],
            16,
            None,
            None,
        )
        .expect("build should succeed");

        assert!(output.output_path.exists(), "extension: {extension}");
        Validator::new()
            .validate_all(&output.wasm_bytes)
            .expect("artifact should validate");
    }
}

#[test]
fn build_source_file_supports_deno_env_has_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'));",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_has_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'));",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_has_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "console.log(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'));",
        )
        .expect("write source");

        let output = build_source_file(
            &source_path,
            BuildMode::Fast,
            ApiSurface::Deno,
            false,
            &[],
            16,
            None,
            None,
        )
        .expect("build should succeed");

        assert!(output.output_path.exists(), "extension: {extension}");
        Validator::new()
            .validate_all(&output.wasm_bytes)
            .expect("artifact should validate");
    }
}

#[test]
fn build_source_file_supports_deno_env_set_and_delete_in_all_inputs() {
    for extension in ["ts", "js", "jsx", "tsx"] {
        assert_build_source_file_supports_deno_env_set_and_delete_in_input(extension);
    }
}

#[test]
fn build_source_file_supports_deno_pid_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno.pid); console.log(Deno[\"pid\"]); console.log(globalThis.Deno.pid); console.log(globalThis[\"Deno\"][\"pid\"]);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.pid); console.log(Deno[\"pid\"]); console.log(globalThis.Deno.pid); console.log(globalThis[\"Deno\"][\"pid\"]);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_pid_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno[\"pid\"]); console.log(globalThis[\"Deno\"][\"pid\"]);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_cwd_chdir_and_exit_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let nested_dir = dir.path().join("nested");
    fs::create_dir(&nested_dir).expect("create nested dir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno[\"cwd\"]()); console.log(globalThis[\"Deno\"][\"cwd\"]()); Deno[\"chdir\"]('nested'); globalThis[\"Deno\"][\"chdir\"]('nested'); Deno[\"exit\"](7); globalThis[\"Deno\"][\"exit\"](7);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_cwd_chdir_and_exit_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let nested_dir = dir.path().join("nested");
    fs::create_dir(&nested_dir).expect("create nested dir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno[\"cwd\"]()); console.log(globalThis[\"Deno\"][\"cwd\"]()); Deno[\"chdir\"]('nested'); globalThis[\"Deno\"][\"chdir\"]('nested'); Deno[\"exit\"](7); globalThis[\"Deno\"][\"exit\"](7);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno[\"pid\"]); console.log(globalThis[\"Deno\"][\"pid\"]);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_env_get_in_ts_input_direct() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'));",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_env_get_in_js_input_direct() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'));",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_env_get_in_js_input_global_this() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno.env[\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_env_get_in_ts_input_global_this() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno.env[\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_permission_query_const_bindings_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
Deno.permissions.query({ name: read_descriptor });
Deno.permissions["query"]({ name: read_descriptor });
Deno["permissions"]["query"]({ name: read_descriptor });
globalThis.Deno.permissions.query({ name: read_descriptor });
globalThis.Deno.permissions["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"].query({ name: write_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: write_descriptor });
globalThis.Deno.permissions.query({ name: write_descriptor });
globalThis.Deno.permissions["query"]({ name: write_descriptor });
Deno.permissions.query({ name: env_descriptor });
Deno.permissions["query"]({ name: env_descriptor });
Deno["permissions"]["query"]({ name: env_descriptor });
globalThis["Deno"]["permissions"].query({ name: net_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: net_descriptor });
globalThis.Deno.permissions.query({ name: net_descriptor });
globalThis.Deno.permissions["query"]({ name: net_descriptor });
"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_permission_query_const_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
Deno.permissions.query({ name: read_descriptor });
Deno.permissions["query"]({ name: read_descriptor });
Deno["permissions"]["query"]({ name: read_descriptor });
globalThis.Deno.permissions.query({ name: read_descriptor });
globalThis.Deno.permissions["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"].query({ name: write_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: write_descriptor });
globalThis.Deno.permissions.query({ name: write_descriptor });
globalThis.Deno.permissions["query"]({ name: write_descriptor });
Deno.permissions.query({ name: env_descriptor });
Deno.permissions["query"]({ name: env_descriptor });
Deno["permissions"]["query"]({ name: env_descriptor });
globalThis["Deno"]["permissions"].query({ name: net_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: net_descriptor });
globalThis.Deno.permissions.query({ name: net_descriptor });
globalThis.Deno.permissions["query"]({ name: net_descriptor });
"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_permission_query_const_bindings_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            r#"const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
Deno.permissions.query({ name: read_descriptor });
Deno.permissions["query"]({ name: read_descriptor });
Deno["permissions"]["query"]({ name: read_descriptor });
globalThis.Deno.permissions.query({ name: read_descriptor });
globalThis.Deno.permissions["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"].query({ name: write_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: write_descriptor });
globalThis.Deno.permissions.query({ name: write_descriptor });
globalThis.Deno.permissions["query"]({ name: write_descriptor });
Deno.permissions.query({ name: env_descriptor });
Deno.permissions["query"]({ name: env_descriptor });
Deno["permissions"]["query"]({ name: env_descriptor });
globalThis["Deno"]["permissions"].query({ name: net_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: net_descriptor });
globalThis.Deno.permissions.query({ name: net_descriptor });
globalThis.Deno.permissions["query"]({ name: net_descriptor });
"#,
        )
        .expect("write source");

        let output = build_source_file(
            &source_path,
            BuildMode::Fast,
            ApiSurface::Deno,
            false,
            &[],
            16,
            None,
            None,
        )
        .expect("build should succeed");

        assert!(output.output_path.exists(), "extension: {extension}");
        Validator::new()
            .validate_all(&output.wasm_bytes)
            .expect("artifact should validate");
    }
}

#[test]
fn build_source_file_supports_number_predicates_in_deno_and_browser_ts_js_jsx_and_tsx_input() {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["ts", "js", "jsx", "tsx"] {
            assert_build_source_file_supports_number_predicates_in_input(api_surface, extension);
        }
    }
}

#[test]
fn build_source_file_supports_object_is_primitive_literals_in_deno_and_browser_ts_js_jsx_and_tsx_input(
) {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["ts", "js", "jsx", "tsx"] {
            assert_build_source_file_supports_object_is_primitive_literals_in_input(
                api_surface,
                extension,
            );
        }
    }
}

#[test]
fn build_source_file_supports_object_is_same_reference_alias_chain_in_deno_and_browser_ts_js_jsx_and_tsx_input(
) {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["ts", "js", "jsx", "tsx"] {
            assert_build_source_file_supports_object_is_same_reference_alias_chain_in_input(
                api_surface,
                extension,
            );
        }
    }
}

#[test]
fn build_source_file_supports_object_has_own_call_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"Object.hasOwn({}, "a"); globalThis.Object.hasOwn({}, "a"); globalThis["Object"]["hasOwn"]({}, "a"); Object.freeze(globalThis?.Object.hasOwn)({}, "a"); Object.freeze((globalThis?.Object.hasOwn))({}, "a"); Object.freeze(globalThis?.Object["hasOwn"])({}, "a"); Object.freeze((globalThis?.Object["hasOwn"]))({}, "a"); Object.prototype.hasOwnProperty.call({}, "a"); globalThis.Object.prototype.hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({}, "a"); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)({}, "a"); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))({}, "a"); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])({}, "a"); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))({}, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_supports_object_has_own_call_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"Object.hasOwn({}, "a"); globalThis.Object.hasOwn({}, "a"); globalThis["Object"]["hasOwn"]({}, "a"); Object.freeze(globalThis.Object.hasOwn)({}, "a"); Object.freeze(globalThis["Object"].hasOwn)({}, "a"); Object.freeze((globalThis["Object"].hasOwn))({}, "a"); Object.freeze((globalThis["Object"]["hasOwn"]))({}, "a"); Object.freeze(globalThis?.Object.hasOwn)({}, "a"); Object.freeze((globalThis?.Object.hasOwn))({}, "a"); Object.freeze(globalThis?.Object["hasOwn"])({}, "a"); Object.freeze((globalThis?.Object["hasOwn"]))({}, "a"); Object.prototype.hasOwnProperty.call({}, "a"); globalThis.Object.prototype.hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({}, "a"); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)({}, "a"); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))({}, "a"); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])({}, "a"); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))({}, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_supports_object_has_own_call_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_object_has_own_call_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_object_has_own_call_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_object_has_own_call_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_object_has_own_call_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_object_has_own_call_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_supports_object_has_own_call_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_object_has_own_call_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_supports_promise_all_settled_across_input_classes() {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["ts", "js", "jsx", "tsx"] {
            assert_build_source_file_supports_promise_all_settled_in_input(api_surface, extension);
        }
    }
}

#[test]
fn build_source_file_supports_promise_any_across_input_classes() {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["ts", "js", "jsx", "tsx"] {
            assert_build_source_file_supports_promise_any_in_input(api_surface, extension);
        }
    }
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_js_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_ts_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_js_input() {
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_ts_input() {
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_browser_api_surface_in_jsx_input()
{
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_browser_api_surface_in_tsx_input()
{
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis[\"Math\"].atan2 literal build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input(
) {
    assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_ts_input(
) {
    assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math.exp(zero)); console.log(globalThis.Math.log(one)); console.log((globalThis.Math.exp)(zero)); console.log(Object.freeze((globalThis.Math.log))(one));\n",
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
    .expect("globalThis.Math.exp/log identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_bracketed_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"exp\"](zero)); console.log(globalThis[\"Math\"][\"log\"](one));\n",
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
    .expect("globalThis[\"Math\"][\"exp\"]/[\"log\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math.exp(zero)); console.log(globalThis.Math.log(one)); console.log((globalThis.Math.exp)(zero)); console.log(Object.freeze((globalThis.Math.log))(one));\n",
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
    .expect("globalThis.Math.exp/log identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_bracketed_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"exp\"](zero)); console.log(globalThis[\"Math\"][\"log\"](one));\n",
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
    .expect("globalThis[\"Math\"][\"exp\"]/[\"log\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_mixed_bracket_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math[\"exp\"](zero)); console.log(globalThis.Math[\"log\"](one));\n",
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
    .expect("globalThis.Math[\"exp\"]/[\"log\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_mixed_bracket_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math[\"exp\"](zero)); console.log(globalThis.Math[\"log\"](one));\n",
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
    .expect("globalThis.Math[\"exp\"]/[\"log\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_js_input() {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_ts_input() {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_jsx_input() {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_tsx_input() {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_js_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_ts_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_array_from_iteration_in_js_input() {
    assert_build_source_file_supports_array_from_iteration_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_array_from_iteration_in_ts_input() {
    assert_build_source_file_supports_array_from_iteration_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_array_from_iteration_in_jsx_input() {
    assert_build_source_file_supports_array_from_iteration_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_array_from_iteration_in_tsx_input() {
    assert_build_source_file_supports_array_from_iteration_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_array_from_iteration_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_array_from_iteration_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_array_from_iteration_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_array_from_iteration_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_array_from_iteration_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_array_from_iteration_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_supports_array_from_iteration_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_array_from_iteration_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_supports_array_from_new_set_and_new_map_iteration_in_js_input() {
    assert_build_source_file_supports_array_from_new_set_and_new_map_iteration_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_array_from_new_set_and_new_map_iteration_in_ts_input() {
    assert_build_source_file_supports_array_from_new_set_and_new_map_iteration_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_array_from_new_set_and_new_map_iteration_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_array_from_new_set_and_new_map_iteration_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_array_from_new_set_and_new_map_iteration_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_array_from_new_set_and_new_map_iteration_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_array_from_new_set_and_new_map_iteration_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_array_from_new_set_and_new_map_iteration_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_array_from_new_set_and_new_map_iteration_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_array_from_new_set_and_new_map_iteration_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_set_constructor_iteration_in_deno_and_browser_js_ts_jsx_and_tsx_input(
) {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["js", "ts", "jsx", "tsx"] {
            assert_build_source_file_supports_set_constructor_iteration_in_input(
                api_surface,
                extension,
            );
        }
    }
}

#[test]
fn build_source_file_supports_map_constructor_iteration_in_deno_and_browser_js_ts_jsx_and_tsx_input(
) {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["js", "ts", "jsx", "tsx"] {
            assert_build_source_file_supports_map_constructor_iteration_in_input(
                api_surface,
                extension,
            );
        }
    }
}

#[test]
fn build_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_browser_api_surface_in_ts_jsx_and_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        assert_build_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_object_helper_nullish_logical_iterator_slices_in_browser_api_surface_in_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_source_file_supports_object_helper_nullish_logical_iterator_slices_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_deno_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
            ApiSurface::Deno,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_browser_api_surface_in_ts_jsx_and_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        assert_build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_browser_api_surface_in_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_set_and_map_frozen_callable_inventory_in_deno_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_build_source_file_supports_set_and_map_frozen_callable_inventory_in_input(
            ApiSurface::Deno,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_set_and_map_frozen_callable_inventory_in_browser_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_build_source_file_supports_set_and_map_frozen_callable_inventory_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_async_class_method_in_supported_inputs() {
    for (api_surface, bundle, extension) in [
        (ApiSurface::Deno, false, "ts"),
        (ApiSurface::Deno, false, "js"),
        (ApiSurface::Deno, false, "jsx"),
        (ApiSurface::Deno, false, "tsx"),
        (ApiSurface::Browser, true, "ts"),
        (ApiSurface::Browser, true, "js"),
        (ApiSurface::Browser, true, "jsx"),
        (ApiSurface::Browser, true, "tsx"),
    ] {
        assert_build_source_file_supports_async_class_method_in_input(
            api_surface,
            bundle,
            extension,
        );
    }
}

#[test]
fn build_source_file_supports_deno_env_to_object_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("env materialization APIs should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_to_object_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("env materialization APIs should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_to_object_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("env materialization APIs should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_to_object_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("env materialization APIs should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_js_input() {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_ts_input() {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_jsx_input() {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_tsx_input() {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}
