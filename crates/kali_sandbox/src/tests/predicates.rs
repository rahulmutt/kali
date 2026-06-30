use super::*;

#[test]
fn predicate_registry_rejects_when_disabled() {
    let policy = valid_policy();
    let registry = PolicyPredicateRegistry::disabled();

    let diagnostic = policy
        .check_operation_with_predicates(HostOperation::Console, &registry)
        .expect_err("disabled predicate registry should reject evaluation");

    assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(diagnostic
        .message
        .contains("host-registered sandbox predicates"));
}

#[test]
fn registered_predicates_run_after_declarative_allowance() {
    let policy = valid_policy();
    let mut registry = PolicyPredicateRegistry::enabled();
    registry.register("effects.console", "deny-stdout", |context| {
        context.subject != "stdout"
    });

    let diagnostic = policy
        .check_operation_with_predicates(HostOperation::Console, &registry)
        .expect_err("predicate should narrow console access");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert!(diagnostic
        .message
        .contains("host-registered predicate 'deny-stdout'"));
    assert!(diagnostic.message.contains("effects.console"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "capability: effects.console"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "subject: stdout"));
}

#[test]
fn predicate_context_records_process_spawn_details() {
    let operation = HostOperation::ProcessSpawn {
        executable: "deno".to_string(),
    };
    let context = PolicyPredicateContext::from_operation(&operation);

    assert_eq!(context.capability, "effects.process.spawn");
    assert_eq!(context.subject, "deno");
    assert_eq!(context.operation, operation);
    assert_eq!(
        context.details.get("executable").map(String::as_str),
        Some("deno")
    );
}

#[test]
fn predicate_context_records_file_network_and_env_details() {
    let file_read = PolicyPredicateContext::from_operation(&HostOperation::FileRead {
        path: PathBuf::from("/workspace/input.txt"),
    });
    assert_eq!(file_read.capability, "effects.fileSystem.read");
    assert_eq!(file_read.subject, "/workspace/input.txt");
    assert_eq!(
        file_read.details.get("path").map(String::as_str),
        Some("/workspace/input.txt")
    );

    let network_fetch = PolicyPredicateContext::from_operation(&HostOperation::NetworkFetch {
        url: "https://example.com/api".to_string(),
    });
    assert_eq!(network_fetch.capability, "effects.network.fetch");
    assert_eq!(network_fetch.subject, "https://example.com/api");
    assert_eq!(
        network_fetch.details.get("url").map(String::as_str),
        Some("https://example.com/api")
    );

    let env_write = PolicyPredicateContext::from_operation(&HostOperation::EnvironmentWrite {
        key: "KALI_FLAG".to_string(),
    });
    assert_eq!(env_write.capability, "effects.process.envWrite");
    assert_eq!(env_write.subject, "KALI_FLAG");
    assert_eq!(
        env_write.details.get("key").map(String::as_str),
        Some("KALI_FLAG")
    );

    let process_env_write =
        PolicyPredicateContext::from_operation(&HostOperation::ProcessEnvWrite {
            key: "KALI_FLAG".to_string(),
        });
    assert_eq!(process_env_write.capability, "effects.process.envWrite");
    assert_eq!(process_env_write.subject, "KALI_FLAG");
    assert_eq!(
        process_env_write.details.get("key").map(String::as_str),
        Some("KALI_FLAG")
    );
}

#[test]
fn predicate_context_records_remaining_host_specific_details() {
    let file_write = PolicyPredicateContext::from_operation(&HostOperation::FileWrite {
        path: PathBuf::from("/workspace/output.txt"),
    });
    assert_eq!(file_write.capability, "effects.fileSystem.write");
    assert_eq!(file_write.subject, "/workspace/output.txt");
    assert_eq!(
        file_write.details.get("path").map(String::as_str),
        Some("/workspace/output.txt")
    );

    let network_connect = PolicyPredicateContext::from_operation(&HostOperation::NetworkConnect {
        target: "127.0.0.1:80".to_string(),
    });
    assert_eq!(network_connect.capability, "effects.network.connect");
    assert_eq!(network_connect.subject, "127.0.0.1:80");
    assert_eq!(
        network_connect.details.get("target").map(String::as_str),
        Some("127.0.0.1:80")
    );

    let network_listen = PolicyPredicateContext::from_operation(&HostOperation::NetworkListen {
        target: "127.0.0.1:0".to_string(),
    });
    assert_eq!(network_listen.capability, "effects.network.listen");
    assert_eq!(network_listen.subject, "127.0.0.1:0");
    assert_eq!(
        network_listen.details.get("target").map(String::as_str),
        Some("127.0.0.1:0")
    );

    let environment_read =
        PolicyPredicateContext::from_operation(&HostOperation::EnvironmentRead {
            key: "PATH".to_string(),
        });
    assert_eq!(environment_read.capability, "effects.process.envRead");
    assert_eq!(environment_read.subject, "PATH");
    assert_eq!(
        environment_read.details.get("key").map(String::as_str),
        Some("PATH")
    );

    let timer_schedule = PolicyPredicateContext::from_operation(&HostOperation::TimerSchedule {
        delay_ms: 250,
        active_timers: 2,
    });
    assert_eq!(timer_schedule.capability, "effects.timer.schedule");
    assert_eq!(timer_schedule.subject, "250");
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

    let console = PolicyPredicateContext::from_operation(&HostOperation::Console);
    assert_eq!(console.capability, "effects.console");
    assert_eq!(console.subject, "stdout");
    assert!(console.details.is_empty());

    let random = PolicyPredicateContext::from_operation(&HostOperation::Random);
    assert_eq!(random.capability, "effects.random");
    assert_eq!(random.subject, "random");
    assert!(random.details.is_empty());

    let eval = PolicyPredicateContext::from_operation(&HostOperation::Eval);
    assert_eq!(eval.capability, "effects.eval");
    assert_eq!(eval.subject, "eval");
    assert!(eval.details.is_empty());
}

#[test]
fn predicate_context_records_late_process_control_details() {
    let process_pid =
        PolicyPredicateContext::from_operation(&HostOperation::ProcessPid { pid: 42 });
    assert_eq!(process_pid.capability, "effects.process.pid");
    assert_eq!(process_pid.subject, "42");
    assert_eq!(process_pid.operation, HostOperation::ProcessPid { pid: 42 });
    assert_eq!(
        process_pid.details.get("pid").map(String::as_str),
        Some("42")
    );

    let process_cwd = PolicyPredicateContext::from_operation(&HostOperation::ProcessCwd {
        cwd: PathBuf::from("/workspace/project"),
    });
    assert_eq!(process_cwd.capability, "effects.process.cwd");
    assert_eq!(process_cwd.subject, "/workspace/project");
    assert_eq!(
        process_cwd.operation,
        HostOperation::ProcessCwd {
            cwd: PathBuf::from("/workspace/project"),
        }
    );
    assert_eq!(
        process_cwd.details.get("cwd").map(String::as_str),
        Some("/workspace/project")
    );

    let process_chdir = PolicyPredicateContext::from_operation(&HostOperation::ProcessChdir {
        path: PathBuf::from("/workspace/project/nested"),
    });
    assert_eq!(process_chdir.capability, "effects.process.chdir");
    assert_eq!(process_chdir.subject, "/workspace/project/nested");
    assert_eq!(
        process_chdir.operation,
        HostOperation::ProcessChdir {
            path: PathBuf::from("/workspace/project/nested"),
        }
    );
    assert_eq!(
        process_chdir.details.get("path").map(String::as_str),
        Some("/workspace/project/nested")
    );

    let process_exit =
        PolicyPredicateContext::from_operation(&HostOperation::ProcessExit { code: 3 });
    assert_eq!(process_exit.capability, "effects.process.exit");
    assert_eq!(process_exit.subject, "3");
    assert_eq!(
        process_exit.operation,
        HostOperation::ProcessExit { code: 3 }
    );
    assert_eq!(
        process_exit.details.get("code").map(String::as_str),
        Some("3")
    );
}

#[test]
fn predicate_context_records_thread_spawn_details() {
    let operation = HostOperation::ThreadSpawn { active_threads: 3 };
    let context = PolicyPredicateContext::from_operation(&operation);

    assert_eq!(context.capability, "resources.maxThreads");
    assert_eq!(context.subject, "3");
    assert_eq!(context.operation, operation);
    assert_eq!(
        context.details.get("activeThreads").map(String::as_str),
        Some("3")
    );
}

#[test]
fn late_process_control_operations_remain_feature_gated() {
    let policy = valid_policy();

    let pid = policy
        .check_operation(HostOperation::ProcessPid { pid: 1234 })
        .expect_err("process pid should remain gated in the current phase");
    assert_eq!(pid.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(pid.message.contains("effects.process.pid"));

    let cwd = policy
        .check_operation(HostOperation::ProcessCwd {
            cwd: PathBuf::from("/workspace/project"),
        })
        .expect_err("process cwd should remain gated in the current phase");
    assert_eq!(cwd.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(cwd.message.contains("effects.process.cwd"));

    let chdir = policy
        .check_operation(HostOperation::ProcessChdir {
            path: PathBuf::from("/workspace/project/nested"),
        })
        .expect_err("process chdir should remain gated in the current phase");
    assert_eq!(chdir.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(chdir.message.contains("effects.process.chdir"));

    let exit = policy
        .check_operation(HostOperation::ProcessExit { code: 3 })
        .expect_err("process exit should remain gated in the current phase");
    assert_eq!(exit.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(exit.message.contains("effects.process.exit"));
}

#[test]
fn registered_predicates_can_inspect_host_specific_context_details() {
    let mut policy = valid_policy();
    policy.resources.max_threads = Some(2);

    let seen_details = Arc::new(Mutex::new(Vec::<Option<String>>::new()));
    let mut registry = PolicyPredicateRegistry::enabled();
    let seen_details_clone = Arc::clone(&seen_details);
    registry.register(
        "resources.maxThreads",
        "deny-nonzero-threads",
        move |context| {
            seen_details_clone
                .lock()
                .expect("details mutex")
                .push(context.details.get("activeThreads").cloned());
            context
                .details
                .get("activeThreads")
                .is_some_and(|count| count == "0")
        },
    );

    let diagnostic = policy
        .check_operation_with_predicates(
            HostOperation::ThreadSpawn { active_threads: 1 },
            &registry,
        )
        .expect_err("predicate should narrow thread creation");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert!(diagnostic
        .message
        .contains("host-registered predicate 'deny-nonzero-threads'"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "capability: resources.maxThreads"));
    assert!(diagnostic.notes.iter().any(|note| note == "subject: 1"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "detail activeThreads: 1"));
    assert_eq!(
        *seen_details.lock().expect("details mutex"),
        vec![Some(String::from("1"))]
    );
}

#[test]
fn declarative_denials_stay_primary_over_predicates() {
    let mut policy = valid_policy();
    policy.effects.console = false;

    let mut registry = PolicyPredicateRegistry::enabled();
    registry.register("effects.console", "allow-all", |_| true);

    let diagnostic = policy
        .check_operation_with_predicates(HostOperation::Console, &registry)
        .expect_err("declarative deny should win");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert!(diagnostic.message.contains("Console output is not allowed"));
    assert!(!diagnostic.message.contains("host-registered predicate"));
}

#[test]
fn registered_predicates_run_in_registration_order() {
    let policy = valid_policy();
    let log = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let mut registry = PolicyPredicateRegistry::enabled();

    let first_log = Arc::clone(&log);
    registry.register("effects.console", "first", move |_| {
        first_log.lock().expect("log mutex").push("first");
        true
    });

    let second_log = Arc::clone(&log);
    registry.register("effects.console", "second", move |_| {
        second_log.lock().expect("log mutex").push("second");
        false
    });

    let diagnostic = policy
        .check_operation_with_predicates(HostOperation::Console, &registry)
        .expect_err("second predicate should reject after first passes");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert_eq!(*log.lock().expect("log mutex"), vec!["first", "second"]);
}

#[test]
fn access_rules_match_globs() {
    let policy = valid_policy();
    assert!(policy
        .effects
        .file_system
        .read
        .allows_path(Path::new("/data/input.txt"), &policy.base_dir));
    assert!(!policy
        .effects
        .file_system
        .read
        .allows_path(Path::new("/secret/input.txt"), &policy.base_dir));
    assert!(policy.effects.network.fetch.allows_candidate(
        "https://example.com/a/b",
        &policy.base_dir,
        PatternKind::Url
    ));
}
