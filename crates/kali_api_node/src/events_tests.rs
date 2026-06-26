use crate::*;

#[test]
fn event_emitter_invokes_listeners_in_order() {
    use std::sync::{Arc, Mutex};

    let emitter = EventEmitter::new();
    let observed: Arc<Mutex<Vec<(String, i32)>>> = Arc::new(Mutex::new(Vec::new()));

    {
        let observed = Arc::clone(&observed);
        emitter.on("message", move |event| {
            observed
                .lock()
                .expect("observed mutex")
                .push((event.event_type().to_string(), 1));
        });
    }
    {
        let observed = Arc::clone(&observed);
        emitter.on("message", move |event| {
            observed
                .lock()
                .expect("observed mutex")
                .push((event.event_type().to_string(), 2));
        });
    }

    let event = NodeEvent::with_detail("message", "payload");
    assert_eq!(emitter.emit(&event), 2);
    assert_eq!(
        observed.lock().expect("observed mutex").clone(),
        vec![("message".to_string(), 1), ("message".to_string(), 2)]
    );
    assert_eq!(event.detail(), Some("payload"));
    assert_eq!(emitter.listener_count("message"), 2);
}
