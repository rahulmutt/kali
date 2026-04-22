use super::*;
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tempfile::tempdir;

fn permissive_policy() -> kali_sandbox::SandboxPolicy {
    kali_sandbox::SandboxPolicy {
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
                max_connections: None,
            },
            process: kali_sandbox::ProcessPolicy {
                spawn: kali_sandbox::AccessRule::Deny(false),
                env_read: kali_sandbox::AccessRule::Deny(false),
                env_write: kali_sandbox::AccessRule::Deny(false),
            },
            timer: kali_sandbox::TimerPolicy {
                schedule: false,
                max_timeout_ms: None,
                max_active_timers: None,
            },
            eval: false,
            random: false,
            console: true,
        },
        resources: kali_sandbox::ResourceLimits {
            max_memory_mb: None,
            max_cpu_time_ms: None,
            max_open_files: None,
            max_spawned_processes: None,
            max_threads: None,
        },
        base_dir: std::path::PathBuf::from("."),
        serialized_source: None,
    }
}

#[test]
fn compiles_standalone_artifacts_in_memory() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let compiler = KaliCompiler::new(CompilerConfig::default());
    let artifact = compiler.compile_file(&source_path).expect("compile file");

    assert!(!artifact.wasm_bytes().is_empty());
    assert_eq!(artifact.metadata().artifact_kind, "executable");
    assert_eq!(
        artifact.metadata().entrypoint,
        source_path.display().to_string()
    );
}

#[test]
fn compiles_library_artifacts_with_wit_sidecars() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let compiler = KaliCompiler::new(CompilerConfig::default());
    let artifact = compiler.compile_lib(&source_path).expect("compile lib");

    assert!(!artifact.wasm_bytes().is_empty());
    assert!(artifact.wit().contains("package kali:embed;"));
    assert!(artifact.wit().contains("export add: func();"));
    assert_eq!(artifact.metadata().artifact_kind, "lib");
    assert_eq!(artifact.metadata().exports.as_ref().unwrap().len(), 1);
}

#[test]
fn compile_lib_reports_missing_export_surfaces() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("lib.ts");
    fs::write(&source_path, "const value = 1;").expect("write source");

    let compiler = KaliCompiler::new(CompilerConfig::default());
    let error = compiler
        .compile_lib(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_EXPORT_SURFACE as u32)),
        "expected E5011 diagnostic: {error}"
    );
}

#[test]
fn temporary_source_paths_are_unique_across_calls() {
    let first = temporary_source_path("first-module");
    let second = temporary_source_path("second/module");

    assert_ne!(first, second);
    assert!(first.display().to_string().contains("kali-embed-"));
    assert!(second.display().to_string().contains("kali-embed-"));
    assert!(first.display().to_string().contains("first-module"));
    assert!(second.display().to_string().contains("second_module"));
}

#[test]
fn compile_lib_from_raw_source_uses_a_stable_module_name() {
    let compiler = KaliCompiler::new(CompilerConfig::default());
    let artifact = compiler
        .compile_lib_source(
            "math/embedded",
            "export function add(a, b) { return a + b; }",
        )
        .expect("compile lib source");

    assert!(!artifact.wasm_bytes().is_empty());
    assert_eq!(artifact.metadata().artifact_kind, "lib");
    assert_eq!(artifact.metadata().entrypoint, "math/embedded");
    assert!(artifact.wit().contains("// module: math/embedded"));
    assert!(artifact.wit().contains("export add: func();"));
}

#[test]
fn embedding_context_uses_the_stable_compiler_api() {
    let ctx = EmbeddingCtx::new();
    let wasm_bytes = ctx.build_library("export function add(a, b) { return a + b; }");

    assert!(!wasm_bytes.is_empty());
}

#[test]
fn embedding_layer_reexports_the_host_predicate_context() {
    let operation = kali_sandbox::HostOperation::Console;
    let context = PolicyPredicateContext::from_operation(&operation);

    assert_eq!(context.capability, "effects.console");
    assert_eq!(context.subject, "stdout");
    assert_eq!(context.operation, operation);
    assert!(context.details.is_empty());

    let mut registry = PolicyPredicateRegistry::enabled();
    registry.register("effects.console", "deny-stdout", |ctx| {
        ctx.subject != "stdout"
    });
}

#[test]
fn embedding_operation_context_uses_the_resource_alias_and_details() {
    let operation = HostOperation::ThreadSpawn { active_threads: 5 };
    let context = OperationContext::from_operation(&operation);

    assert_eq!(context.capability, "resources.maxThreads");
    assert_eq!(context.resource, "5");
    assert_eq!(context.operation, operation);
    assert_eq!(
        context.details.get("activeThreads").map(String::as_str),
        Some("5")
    );
}

#[test]
fn embedding_predicates_can_deny_with_a_host_specific_reason() {
    let policy = permissive_policy();
    let mut ctx = EmbeddingCtx::new();
    ctx.register_sandbox_predicate("effects.console", "deny-console", |_| {
        PredicateDecision::deny("console output is forbidden")
    })
    .expect("predicate registration should succeed");

    let diagnostic = ctx
        .check_operation_with_policy(&policy, HostOperation::Console)
        .expect_err("predicate should narrow console access");

    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::EFFECT_NOT_PERMITTED as u32)
    );
    assert!(diagnostic
        .message
        .contains("host-registered predicate 'deny-console'"));
    assert!(diagnostic.message.contains("console output is forbidden"));
}

#[test]
fn embedding_predicates_do_not_override_declarative_denials() {
    let mut policy = permissive_policy();
    policy.effects.console = false;

    let mut ctx = EmbeddingCtx::new();
    ctx.register_sandbox_predicate("effects.console", "allow-all", |_| {
        PredicateDecision::allow()
    })
    .expect("predicate registration should succeed");

    let diagnostic = ctx
        .check_operation_with_policy(&policy, HostOperation::Console)
        .expect_err("declarative deny should win");

    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::EFFECT_NOT_PERMITTED as u32)
    );
    assert!(diagnostic.message.contains("Console output is not allowed"));
    assert!(!diagnostic.message.contains("host-registered predicate"));
}

#[test]
fn embedding_predicates_can_inspect_thread_budget_context_details() {
    let mut policy = permissive_policy();
    policy.resources.max_threads = Some(3);

    let seen_details = Arc::new(Mutex::new(Vec::<Option<String>>::new()));
    let seen_details_clone = Arc::clone(&seen_details);
    let mut ctx = EmbeddingCtx::new();
    ctx.register_sandbox_predicate(
        "resources.maxThreads",
        "deny-busy-thread-budget",
        move |context| {
            seen_details_clone
                .lock()
                .expect("details mutex")
                .push(context.details.get("activeThreads").cloned());
            context
                .details
                .get("activeThreads")
                .is_some_and(|count| count == "0")
                .into()
        },
    )
    .expect("predicate registration should succeed");

    let diagnostic = ctx
        .check_operation_with_policy(&policy, HostOperation::ThreadSpawn { active_threads: 1 })
        .expect_err("predicate should narrow thread creation");

    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::EFFECT_NOT_PERMITTED as u32)
    );
    assert!(diagnostic
        .message
        .contains("host-registered predicate 'deny-busy-thread-budget'"));
    assert_eq!(
        *seen_details.lock().expect("details mutex"),
        vec![Some(String::from("1"))]
    );
}

#[test]
fn embedding_predicates_run_in_registration_order() {
    let policy = permissive_policy();
    let log = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let mut ctx = EmbeddingCtx::new();

    let first_log = Arc::clone(&log);
    ctx.register_sandbox_predicate("effects.console", "first", move |_| {
        first_log.lock().expect("log mutex").push("first");
        PredicateDecision::allow()
    })
    .expect("first predicate registration should succeed");

    let second_log = Arc::clone(&log);
    ctx.register_sandbox_predicate("effects.console", "second", move |_| {
        second_log.lock().expect("log mutex").push("second");
        PredicateDecision::deny("console output is forbidden")
    })
    .expect("second predicate registration should succeed");

    let diagnostic = ctx
        .check_operation_with_policy(&policy, HostOperation::Console)
        .expect_err("second predicate should reject after first passes");

    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e4::EFFECT_NOT_PERMITTED as u32)
    );
    assert!(diagnostic.message.contains("host-registered predicate 'second'"));
    assert_eq!(*log.lock().expect("log mutex"), vec!["first", "second"]);
}

#[test]
fn embedding_predicate_registration_rejects_when_disabled() {
    let mut ctx = EmbeddingCtx::with_predicate_registration_enabled(false);
    let error = ctx
        .register_sandbox_predicate("effects.console", "deny-console", |_| {
            PredicateDecision::deny("console output is forbidden")
        })
        .err()
        .expect("disabled predicate support should reject registration");

    assert_eq!(
        error.code,
        Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(error
        .message
        .contains("host-registered sandbox predicates are unavailable"));
}

#[test]
fn compiler_rejects_threaded_runtime_profiles_in_the_current_phase() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let mut config = CompilerConfig::default();
    config.runtime_profiles = vec!["wasm-threads".to_string()];
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected E5006 diagnostic: {error}"
    );
}

#[test]
fn compiler_rejects_duplicate_runtime_profiles_before_phase_gating() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let mut config = CompilerConfig::default();
    config.runtime_profiles = vec!["wasm-threads".to_string(), "wasm-threads".to_string()];
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)),
        "expected E5009 diagnostic: {error}"
    );
}

#[test]
fn compiler_rejects_unknown_runtime_profiles_before_phase_gating() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export function add(a, b) { return a + b; }").expect("write source");

    let mut config = CompilerConfig::default();
    config.runtime_profiles = vec!["fiber-threads".to_string()];
    let compiler = KaliCompiler::new(config);
    let error = compiler
        .compile_file(&source_path)
        .expect_err("compile should fail");

    assert!(
        error.diagnostics().iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)),
        "expected E5009 diagnostic: {error}"
    );
}
