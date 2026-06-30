use super::*;

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
fn embedding_operation_context_uses_process_spawn_resource_alias_and_details() {
    let operation = HostOperation::ProcessSpawn {
        executable: "deno".to_string(),
    };
    let context = OperationContext::from_operation(&operation);

    assert_eq!(context.capability, "effects.process.spawn");
    assert_eq!(context.resource, "deno");
    assert_eq!(context.operation, operation);
    assert_eq!(
        context.details.get("executable").map(String::as_str),
        Some("deno")
    );
}

#[test]
fn embedding_operation_context_carries_file_network_and_env_details() {
    let file_read = OperationContext::from_operation(&HostOperation::FileRead {
        path: std::path::PathBuf::from("/workspace/input.txt"),
    });
    assert_eq!(file_read.capability, "effects.fileSystem.read");
    assert_eq!(file_read.resource, "/workspace/input.txt");
    assert_eq!(
        file_read.details.get("path").map(String::as_str),
        Some("/workspace/input.txt")
    );

    let network_fetch = OperationContext::from_operation(&HostOperation::NetworkFetch {
        url: "https://example.com/api".to_string(),
    });
    assert_eq!(network_fetch.capability, "effects.network.fetch");
    assert_eq!(network_fetch.resource, "https://example.com/api");
    assert_eq!(
        network_fetch.details.get("url").map(String::as_str),
        Some("https://example.com/api")
    );

    let env_write = OperationContext::from_operation(&HostOperation::EnvironmentWrite {
        key: "KALI_FLAG".to_string(),
    });
    assert_eq!(env_write.capability, "effects.process.envWrite");
    assert_eq!(env_write.resource, "KALI_FLAG");
    assert_eq!(
        env_write.details.get("key").map(String::as_str),
        Some("KALI_FLAG")
    );

    let process_env_write = OperationContext::from_operation(&HostOperation::ProcessEnvWrite {
        key: "KALI_FLAG".to_string(),
    });
    assert_eq!(process_env_write.capability, "effects.process.envWrite");
    assert_eq!(process_env_write.resource, "KALI_FLAG");
    assert_eq!(
        process_env_write.details.get("key").map(String::as_str),
        Some("KALI_FLAG")
    );

    let process_pid = OperationContext::from_operation(&HostOperation::ProcessPid { pid: 42 });
    assert_eq!(process_pid.capability, "effects.process.pid");
    assert_eq!(process_pid.resource, "42");
    assert_eq!(process_pid.operation, HostOperation::ProcessPid { pid: 42 });
    assert_eq!(
        process_pid.details.get("pid").map(String::as_str),
        Some("42")
    );

    let process_cwd = OperationContext::from_operation(&HostOperation::ProcessCwd {
        cwd: std::path::PathBuf::from("/workspace"),
    });
    assert_eq!(process_cwd.capability, "effects.process.cwd");
    assert_eq!(process_cwd.resource, "/workspace");
    assert_eq!(
        process_cwd.details.get("cwd").map(String::as_str),
        Some("/workspace")
    );

    let process_chdir = OperationContext::from_operation(&HostOperation::ProcessChdir {
        path: std::path::PathBuf::from("/workspace/project"),
    });
    assert_eq!(process_chdir.capability, "effects.process.chdir");
    assert_eq!(process_chdir.resource, "/workspace/project");
    assert_eq!(
        process_chdir.details.get("path").map(String::as_str),
        Some("/workspace/project")
    );

    let process_exit = OperationContext::from_operation(&HostOperation::ProcessExit { code: 3 });
    assert_eq!(process_exit.capability, "effects.process.exit");
    assert_eq!(process_exit.resource, "3");
    assert_eq!(
        process_exit.details.get("code").map(String::as_str),
        Some("3")
    );
}

#[test]
fn embedding_operation_context_carries_remaining_host_specific_details() {
    let file_write = OperationContext::from_operation(&HostOperation::FileWrite {
        path: std::path::PathBuf::from("/workspace/output.txt"),
    });
    assert_eq!(file_write.capability, "effects.fileSystem.write");
    assert_eq!(file_write.resource, "/workspace/output.txt");
    assert_eq!(
        file_write.details.get("path").map(String::as_str),
        Some("/workspace/output.txt")
    );

    let network_connect = OperationContext::from_operation(&HostOperation::NetworkConnect {
        target: "127.0.0.1:80".to_string(),
    });
    assert_eq!(network_connect.capability, "effects.network.connect");
    assert_eq!(network_connect.resource, "127.0.0.1:80");
    assert_eq!(
        network_connect.details.get("target").map(String::as_str),
        Some("127.0.0.1:80")
    );

    let network_listen = OperationContext::from_operation(&HostOperation::NetworkListen {
        target: "127.0.0.1:0".to_string(),
    });
    assert_eq!(network_listen.capability, "effects.network.listen");
    assert_eq!(network_listen.resource, "127.0.0.1:0");
    assert_eq!(
        network_listen.details.get("target").map(String::as_str),
        Some("127.0.0.1:0")
    );

    let environment_read = OperationContext::from_operation(&HostOperation::EnvironmentRead {
        key: "PATH".to_string(),
    });
    assert_eq!(environment_read.capability, "effects.process.envRead");
    assert_eq!(environment_read.resource, "PATH");
    assert_eq!(
        environment_read.details.get("key").map(String::as_str),
        Some("PATH")
    );

    let timer_schedule = OperationContext::from_operation(&HostOperation::TimerSchedule {
        delay_ms: 250,
        active_timers: 2,
    });
    assert_eq!(timer_schedule.capability, "effects.timer.schedule");
    assert_eq!(timer_schedule.resource, "250");
    assert_eq!(
        timer_schedule
            .details
            .get("activeTimers")
            .map(String::as_str),
        Some("2")
    );
    assert_eq!(
        timer_schedule.details.get("delayMs").map(String::as_str),
        Some("250")
    );

    let console = OperationContext::from_operation(&HostOperation::Console);
    assert_eq!(console.capability, "effects.console");
    assert_eq!(console.resource, "stdout");
    assert!(console.details.is_empty());

    let random = OperationContext::from_operation(&HostOperation::Random);
    assert_eq!(random.capability, "effects.random");
    assert_eq!(random.resource, "random");
    assert!(random.details.is_empty());

    let eval = OperationContext::from_operation(&HostOperation::Eval);
    assert_eq!(eval.capability, "effects.eval");
    assert_eq!(eval.resource, "eval");
    assert!(eval.details.is_empty());
}

#[test]
fn embedding_operation_context_uses_the_resource_alias_and_details_for_threads() {
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
