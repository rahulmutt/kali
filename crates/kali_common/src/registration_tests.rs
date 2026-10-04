use super::*;

#[test]
fn bare_scheduling_callees_register() {
    for name in ["queueMicrotask", "setTimeout", "setInterval"] {
        assert!(is_deferred_registration_callee(name, false), "{name}");
        assert!(
            !is_deferred_registration_callee(name, true),
            "member {name}"
        );
    }
}

#[test]
fn only_a_member_add_event_listener_registers() {
    assert!(is_deferred_registration_callee("addEventListener", true));
    assert!(!is_deferred_registration_callee("addEventListener", false));
}

#[test]
fn other_callees_do_not_register() {
    for name in ["test", "then", "forEach", "setImmediate", ""] {
        assert!(!is_deferred_registration_callee(name, false));
        assert!(!is_deferred_registration_callee(name, true));
    }
}
