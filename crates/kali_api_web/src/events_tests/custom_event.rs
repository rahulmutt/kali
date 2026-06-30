use super::*;

#[test]
fn custom_event_carries_detail_payload() {
    let event = CustomEvent::new("payload", Value::String("detail".to_string()));
    assert_eq!(event.event().event_type(), "payload");
    assert_eq!(event.detail(), &Value::String("detail".to_string()));
}
