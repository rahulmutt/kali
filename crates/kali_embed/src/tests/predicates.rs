use super::*;

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
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "capability: effects.console"));
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "resource: stdout"));
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
    assert!(diagnostic
        .notes
        .iter()
        .any(|note| note == "capability: resources.maxThreads"));
    assert!(diagnostic.notes.iter().any(|note| note == "resource: 1"));
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
    assert!(diagnostic
        .message
        .contains("host-registered predicate 'second'"));
    assert_eq!(*log.lock().expect("log mutex"), vec!["first", "second"]);
}

#[test]
fn embedding_predicate_registration_availability_can_be_queried() {
    let enabled = EmbeddingCtx::new();
    let disabled = EmbeddingCtx::with_predicate_registration_enabled(false);

    assert!(enabled.predicate_registration_enabled());
    assert!(!disabled.predicate_registration_enabled());
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
