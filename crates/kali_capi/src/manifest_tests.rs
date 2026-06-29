use crate::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn valid_binding_package_manifest() -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": 1,
        "kind": "binding-package",
        "moduleName": "sample",
        "hostAbiVersion": HOST_ABI_VERSION,
        "minHostAbiVersion": HOST_ABI_VERSION,
        "maxSpecializations": 8,
        "runtimeProfiles": ["wasm-threads", "fiber-threads", "wasm-threads"],
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "artifacts": {
            "library": "sample.capi.wasm",
            "metadata": "sample.cabi.json",
            "exportsHeader": "sample.h",
            "glue": ["z.py", "a.py", "z.py"]
        }
    })
}

#[path = "manifest_tests/parsing.rs"]
mod parsing;

#[path = "manifest_tests/helpers.rs"]
mod helpers;

#[path = "manifest_tests/summary.rs"]
mod summary;

#[path = "manifest_tests/construction.rs"]
mod construction;
