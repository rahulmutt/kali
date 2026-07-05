use super::*;

#[test]
fn runtime_executes_modules_with_console_host_imports() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (import "kali:rt" "console_error" (func $console_error (param i64)))
                (import "kali:rt" "console_warn" (func $console_warn (param i64)))
                (import "kali:rt" "console_info" (func $console_info (param i64)))
                (import "kali:rt" "console_debug" (func $console_debug (param i64)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    call $console_log
                    i64.const 2
                    call $console_error
                    i64.const 3
                    call $console_warn
                    i64.const 4
                    call $console_info
                    i64.const 5
                    call $console_debug))
            "#,
    );

    let runtime = RuntimeCtx::default();
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.stdout, "1\n4\n5\n");
    assert_eq!(outcome.stderr, "2\n[warn] 3\n");
}

#[test]
fn runtime_exposes_arguments() {
    let runtime = RuntimeCtx::with_host_context(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        capture_env(),
        PathBuf::from("."),
    );

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "args_len" (func $args_len (result i32)))
                (func (export "_start")
                    call $args_len
                    i32.const 2
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_records_guest_process_exit_codes() {
    let runtime = RuntimeCtx::default();
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "process_exit" (func $process_exit (param i64)))
                (func (export "_start")
                    i64.const 7
                    call $process_exit))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 7);
}

#[test]
fn runtime_exposes_environment_variables() {
    let mut env = BTreeMap::new();
    env.insert("KALI_RUNTIME_TEST_ENV".to_string(), "hello".to_string());
    let runtime = RuntimeCtx::with_host_context(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        env,
        PathBuf::from("."),
    );

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "env_get" (func $env_get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "KALI_RUNTIME_TEST_ENV")
                (func (export "_start")
                    i32.const 0
                    i32.const 21
                    i32.const 128
                    i32.const 64
                    call $env_get
                    i32.const 6
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_reports_environment_variable_presence() {
    let mut env = BTreeMap::new();
    env.insert("KALI_RUNTIME_TEST_ENV".to_string(), "hello".to_string());
    let runtime = RuntimeCtx::with_host_context(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        env,
        PathBuf::from("."),
    );

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "env_has" (func $env_has (param i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "KALI_RUNTIME_TEST_ENV")
                (func (export "_start")
                    i32.const 0
                    i32.const 21
                    call $env_has
                    i32.const 1
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_reports_current_working_directory() {
    let dir = kali_test_support::fixtures::tempdir();
    let cwd = dir.path().to_path_buf();
    let expected_len = cwd.to_string_lossy().len() as i32;
    let runtime = RuntimeCtx::with_host_context(None, Vec::new(), BTreeMap::new(), cwd);

    let wasm = compile_wat(&format!(
        r#"
            (module
                (import "kali:rt" "cwd" (func $cwd (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i32.const 0
                    i32.const 0
                    i32.const 128
                    i32.const 64
                    call $cwd
                    i32.const {expected_len}
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#
    ));

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_deletes_environment_variables() {
    let mut env = BTreeMap::new();
    env.insert("KALI_RUNTIME_TEST_ENV".to_string(), "hello".to_string());
    let runtime = RuntimeCtx::with_host_context(
        None,
        vec!["alpha".to_string(), "beta".to_string()],
        env,
        PathBuf::from("."),
    );

    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "env_delete" (func $env_delete (param i32 i32 i32 i32) (result i32)))
                (import "kali:rt" "env_get" (func $env_get (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "KALI_RUNTIME_TEST_ENV")
                (func (export "_start")
                    i32.const 0
                    i32.const 21
                    i32.const 0
                    i32.const 0
                    call $env_delete
                    drop
                    i32.const 0
                    i32.const 21
                    i32.const 128
                    i32.const 64
                    call $env_get
                    i32.eqz
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
}

#[test]
fn runtime_writes_text_files() {
    let dir = kali_test_support::fixtures::tempdir();
    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), dir.path().to_path_buf());
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "fs_write_text_file" (func $write (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "./written.txt")
                (data (i32.const 64) "hello runtime")
                (func (export "_start")
                    i32.const 0
                    i32.const 13
                    i32.const 64
                    i32.const 13
                    call $write
                    i32.const 0
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);

    let written = fs::read_to_string(dir.path().join("written.txt")).expect("written file");
    assert_eq!(written, "hello runtime");
}

#[test]
fn runtime_fetches_http_responses() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let body = "hello fetch";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0u8; 1024];
        let _ = stream.read(&mut buffer);
        let _ = stream.write_all(response.as_bytes());
    });

    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let url = format!("http://127.0.0.1:{}/", addr.port());
    let wat = format!(
        r#"
            (module
                (import "kali:rt" "fetch" (func $fetch (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{}")
                (func (export "_start")
                    i32.const 0
                    i32.const {}
                    i32.const 128
                    i32.const 64
                    call $fetch
                    i32.const {}
                    i32.eq
                    if
                    else
                        unreachable
                    end))
            "#,
        url,
        url.len(),
        body.len()
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 0);
    server.join().expect("server thread");
}

#[test]
fn runtime_reports_mocked_fetch_failures() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buffer = [0u8; 1024];
        let _ = stream.read(&mut buffer);
        let _ = stream.write_all(response.as_bytes());
    });

    let runtime =
        RuntimeCtx::with_host_context(None, Vec::new(), capture_env(), PathBuf::from("."));
    let url = format!("http://127.0.0.1:{}/missing", addr.port());
    let wat = format!(
        r#"
            (module
                (import "kali:rt" "fetch" (func $fetch (param i32 i32 i32 i32) (result i32)))
                (memory (export "memory") 1)
                (data (i32.const 0) "{}")
                (func (export "_start")
                    i32.const 0
                    i32.const {}
                    i32.const 128
                    i32.const 64
                    call $fetch
                    drop))
            "#,
        url,
        url.len()
    );

    let wasm = compile_wat(&wat);
    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    let diagnostic = outcome.trap.expect("fetch should fail");
    assert_eq!(diagnostic.code, Some(e4::UNCAUGHT_ERROR as u32));
    assert!(
        diagnostic.message.contains("runtime trap"),
        "diagnostic: {:?}",
        diagnostic
    );
    server.join().expect("server thread");
}

#[test]
fn runtime_rejects_math_pow_negative_exponents_without_panicking() {
    let runtime = RuntimeCtx::default();
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "math_pow" (func $math_pow (param i64 i64) (result i64)))
                (func (export "_start")
                    i64.const 2
                    i64.const -1
                    call $math_pow
                    drop))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(outcome.exit_code, 1);
    let diagnostic = outcome
        .trap
        .expect("negative Math.pow exponents should be rejected through the host import");
    assert_eq!(diagnostic.code, Some(e4::UNCAUGHT_ERROR as u32));
    assert!(
        diagnostic.message.contains("runtime trap"),
        "diagnostic: {:?}",
        diagnostic
    );
}

#[test]
fn runtime_rejects_console_calls_when_policy_denies_them() {
    let policy = SandboxPolicy {
        schema_version: 1,
        schema_uri: None,
        effects: kali_sandbox::EffectsPolicy {
            file_system: kali_sandbox::FileSystemPolicy {
                read: kali_sandbox::AccessRule::Deny(false),
                write: kali_sandbox::AccessRule::Deny(false),
            },
            network: kali_sandbox::NetworkPolicy {
                fetch: kali_sandbox::AccessRule::Deny(false),
                connect: kali_sandbox::AccessRule::Deny(false),
                listen: kali_sandbox::AccessRule::Deny(false),
                max_connections: Some(1),
            },
            process: kali_sandbox::ProcessPolicy {
                spawn: kali_sandbox::AccessRule::Deny(false),
                env_read: kali_sandbox::AccessRule::Deny(false),
                env_write: kali_sandbox::AccessRule::Deny(false),
            },
            timer: kali_sandbox::TimerPolicy {
                schedule: true,
                max_timeout_ms: Some(1000),
                max_active_timers: Some(1),
            },
            eval: false,
            random: true,
            console: false,
        },
        resources: kali_sandbox::ResourceLimits {
            max_memory_mb: Some(256),
            max_cpu_time_ms: Some(1000),
            max_open_files: Some(8),
            max_spawned_processes: Some(0),
            max_threads: Some(0),
        },
        base_dir: PathBuf::from("."),
        serialized_source: None,
    };
    let runtime =
        RuntimeCtx::with_host_context(Some(policy), Vec::new(), capture_env(), PathBuf::from("."));
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (memory (export "memory") 1)
                (func (export "_start")
                    i64.const 1
                    call $console_log))
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("console should be denied");
    assert_eq!(diagnostics[0].code, Some(e4::EFFECT_NOT_PERMITTED as u32));
}
