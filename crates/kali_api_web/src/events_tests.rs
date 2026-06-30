use crate::*;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[path = "events_tests/abort.rs"]
mod abort;

#[path = "events_tests/event_target.rs"]
mod event_target;

#[path = "events_tests/custom_event.rs"]
mod custom_event;
