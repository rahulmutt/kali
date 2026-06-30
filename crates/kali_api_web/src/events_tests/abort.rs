use super::*;

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
