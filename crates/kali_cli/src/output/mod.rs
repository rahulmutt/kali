//! CLI output envelope: construction, per-payload validation, and serialization.

mod artifacts_timings;
mod browser_runtime;
mod coverage;
mod diagnostic;
mod envelope;
mod options;
mod payload;
mod schema;
mod serialize;
mod thread_topology;

pub use envelope::{emit_envelope, emit_envelope_value, validate_envelope_value};
pub use options::CliOutputOptions;
pub use payload::{
    validate_check_payload_value, validate_doctor_payload_value, validate_effects_payload_value,
    validate_fmt_payload_value, validate_init_payload_value, validate_install_payload_value,
    validate_lint_payload_value, validate_package_audit_payload_value,
    validate_package_effects_payload_value, validate_run_payload_value, validate_test_payload_value,
};
pub use serialize::{diagnostic_to_json, diagnostic_to_text, json_source_path, json_string_list};
pub use thread_topology::merge_thread_topology_snapshot_values;

// Crate-internal re-exports (build.rs reaches validate_sorted_string_array_value via this path).
pub(crate) use schema::*;
