use crate::*;
use std::sync::atomic::{AtomicBool, AtomicUsize};

#[test]
fn abort_controller_flips_the_signal() {
    let controller = AbortController::new();
    let signal = controller.signal();
    assert!(!signal.aborted());
    controller.abort();
    assert!(signal.aborted());
}

#[test]
fn abort_signal_dispatches_abort_events_once() {
    let controller = AbortController::new();
    let signal = controller.signal();
    let invocations = Arc::new(AtomicUsize::new(0));
    let invocations_clone = Arc::clone(&invocations);

    signal.add_event_listener("abort", move |event| {
        assert_eq!(event.event_type(), "abort");
        invocations_clone.fetch_add(1, Ordering::SeqCst);
    });

    controller.abort();
    controller.abort();

    assert!(signal.aborted());
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
    assert_eq!(signal.dispatch_event(&Event::new("abort")), 1);
}

#[test]
fn event_target_dispatches_registered_listeners() {
    let target = EventTarget::new();
    let seen = Arc::new(AtomicBool::new(false));
    let seen_clone = Arc::clone(&seen);

    target.add_event_listener("hello", move |event| {
        seen_clone.store(event.event_type() == "hello", Ordering::SeqCst);
    });

    let event = Event::new("hello");
    assert_eq!(target.dispatch_event(&event), 1);
    assert!(seen.load(Ordering::SeqCst));
}

#[test]
fn event_target_can_remove_registered_listeners() {
    let target = EventTarget::new();
    let invocations = Arc::new(AtomicUsize::new(0));
    let invocations_clone = Arc::clone(&invocations);

    let listener_id = target.add_event_listener("hello", move |_| {
        invocations_clone.fetch_add(1, Ordering::SeqCst);
    });

    let event = Event::new("hello");
    assert_eq!(target.dispatch_event(&event), 1);
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
    assert!(target.remove_event_listener("hello", listener_id));
    assert!(!target.remove_event_listener("hello", listener_id));
    assert_eq!(target.dispatch_event(&event), 0);
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
}

#[test]
fn event_target_can_remove_listeners_during_dispatch_without_deadlocking() {
    let target = EventTarget::new();
    let first_invocations = Arc::new(AtomicUsize::new(0));
    let second_invocations = Arc::new(AtomicUsize::new(0));
    let second_listener_id = Arc::new(AtomicUsize::new(usize::MAX));
    let target_for_first = target.clone();

    let first_invocations_clone = Arc::clone(&first_invocations);
    let second_listener_id_clone = Arc::clone(&second_listener_id);
    target.add_event_listener("hello", move |_| {
        first_invocations_clone.fetch_add(1, Ordering::SeqCst);
        let removed = target_for_first
            .remove_event_listener("hello", second_listener_id_clone.load(Ordering::SeqCst));
        assert!(removed, "listener removal should succeed during dispatch");
    });

    let second_invocations_clone = Arc::clone(&second_invocations);
    let second_id = target.add_event_listener("hello", move |_| {
        second_invocations_clone.fetch_add(1, Ordering::SeqCst);
    });
    second_listener_id.store(second_id, Ordering::SeqCst);

    let event = Event::new("hello");
    assert_eq!(target.dispatch_event(&event), 1);
    assert_eq!(first_invocations.load(Ordering::SeqCst), 1);
    assert_eq!(second_invocations.load(Ordering::SeqCst), 0);
    assert!(!target.remove_event_listener("hello", second_id));
}

#[test]
fn custom_event_carries_detail_payload() {
    let event = CustomEvent::new("payload", Value::String("detail".to_string()));
    assert_eq!(event.event().event_type(), "payload");
    assert_eq!(event.detail(), &Value::String("detail".to_string()));
}
