//! C ABI bindings for Kali (Phase 2 target).

/// Initialize the C API.
pub extern "C" fn kali_init() -> i32 {
    0
}

/// Cleanup the C API.
pub extern "C" fn kali_cleanup() {
}
