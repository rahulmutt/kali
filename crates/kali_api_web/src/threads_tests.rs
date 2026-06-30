use crate::*;
use kali_common::bytewise_shared_memory_is_lock_free;
use serde_json::Value;

#[path = "threads_tests/topology.rs"]
mod topology;

#[path = "threads_tests/atomics.rs"]
mod atomics;

#[path = "threads_tests/shared_array_buffer.rs"]
mod shared_array_buffer;
