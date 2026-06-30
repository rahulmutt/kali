use super::*;

#[test]
fn policy_validates_and_serializes() {
    let policy = valid_policy();
    assert!(policy.validate().is_ok());
    let json = policy.to_canonical_json().expect("canonical json");
    assert!(json.contains("\"schemaVersion\":1"));
    assert!(json.contains("\"effects\""));
}

#[test]
fn policy_thread_budget_helper_preserves_zero_cap_tightening() {
    let mut policy = valid_policy();
    policy.resources.max_threads = Some(4);

    assert_eq!(policy.effective_thread_budget(None), Some(4));
    assert_eq!(policy.effective_thread_budget(Some(0)), Some(0));
    assert_eq!(policy.effective_thread_budget(Some(2)), Some(2));
}

#[test]
fn policy_rejects_thread_spawn_when_no_budget_is_available() {
    let mut policy = valid_policy();
    policy.resources.max_threads = None;

    let diagnostic = policy
        .check_operation(HostOperation::ThreadSpawn { active_threads: 0 })
        .expect_err("thread creation should remain gated without a budget");

    assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(diagnostic.message.contains("resources.maxThreads"));
}

#[test]
fn policy_rejects_thread_spawn_when_the_budget_is_zero() {
    let mut policy = valid_policy();
    policy.resources.max_threads = Some(0);

    let diagnostic = policy
        .check_operation(HostOperation::ThreadSpawn { active_threads: 0 })
        .expect_err("zero-cap thread budgets should deny thread creation");

    assert_eq!(diagnostic.code, Some(e4::RESOURCE_LIMIT_EXCEEDED as u32));
    assert!(diagnostic.message.contains("active thread count 1"));
}

#[test]
fn policy_rejects_positive_spawn_budget_before_subprocess_support_exists() {
    let mut policy = valid_policy();
    policy.resources.max_spawned_processes = Some(1);

    let diagnostics = policy
        .validate()
        .expect_err("positive spawned-process budgets should remain gated in the current phase");

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.code, Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(diagnostic.message.contains("resources.maxSpawnedProcesses"));
}

#[test]
fn policy_spawn_budget_helper_combines_policy_and_override() {
    let mut policy = valid_policy();
    policy.resources.max_spawned_processes = Some(4);

    assert_eq!(policy.effective_spawn_budget(None), Some(4));
    assert_eq!(policy.effective_spawn_budget(Some(0)), Some(0));
    assert_eq!(policy.effective_spawn_budget(Some(2)), Some(2));
}

#[test]
fn policy_rejects_timer_schedule_when_scheduling_is_disabled() {
    let mut policy = valid_policy();
    policy.effects.timer.schedule = false;

    let diagnostic = policy
        .check_operation(HostOperation::TimerSchedule {
            delay_ms: 250,
            active_timers: 0,
        })
        .expect_err("timer creation should remain gated when scheduling is disabled");

    assert_eq!(diagnostic.code, Some(e4::EFFECT_NOT_PERMITTED as u32));
    assert!(diagnostic
        .message
        .contains("Timer creation is not allowed by the current policy"));
}

#[test]
fn policy_rejects_timer_schedule_when_the_delay_exceeds_the_policy_limit() {
    let mut policy = valid_policy();
    policy.effects.timer.max_timeout_ms = Some(100);

    let diagnostic = policy
        .check_operation(HostOperation::TimerSchedule {
            delay_ms: 250,
            active_timers: 0,
        })
        .expect_err("timer delays above the policy limit should be rejected");

    assert_eq!(diagnostic.code, Some(e4::RESOURCE_LIMIT_EXCEEDED as u32));
    assert!(diagnostic
        .message
        .contains("timer delay 250ms exceeds policy limit of 100ms"));
}

#[test]
fn policy_rejects_timer_schedule_when_the_active_timer_limit_is_reached() {
    let mut policy = valid_policy();
    policy.effects.timer.max_active_timers = Some(1);

    let diagnostic = policy
        .check_operation(HostOperation::TimerSchedule {
            delay_ms: 250,
            active_timers: 1,
        })
        .expect_err("timer counts above the policy limit should be rejected");

    assert_eq!(diagnostic.code, Some(e4::RESOURCE_LIMIT_EXCEEDED as u32));
    assert!(diagnostic
        .message
        .contains("active timer count 2 exceeds policy limit of 1"));
}

#[test]
fn policy_rejects_unavailable_capabilities() {
    let mut policy = valid_policy();
    policy.effects.process.env_write = AccessRule::Deny(true);
    policy.effects.network.connect = AccessRule::Deny(true);
    policy.effects.eval = true;
    policy.resources.max_threads = Some(1);

    let validation = policy.validate_policy();
    assert!(!validation.valid);
    assert!(validation
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
}
