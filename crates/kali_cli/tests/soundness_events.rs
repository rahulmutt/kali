//! Stage D event-surface lane (Task 3) — `new EventTarget()` construction lane
//! + handle-escape discipline.
//!
//! The construction lane emits an opaque i64 host handle for a declarator-bound
//! `new EventTarget()`, records the binding's provenance, and fails closed on
//! every read that would leak the raw handle (spec §2.4). The
//! addEventListener/dispatchEvent emit arms land in Task 4; until then a
//! `t.addEventListener(...)` still takes the Stage C backstop.

use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_kali(source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// Stage D event lane: `new EventTarget()` in a declarator compiles and the
/// program runs (the handle is opaque; nothing observable yet).
/// node v26.5.0: "done\n".
#[test]
fn event_target_construction_in_declarator_compiles_and_runs() {
    let out = run_kali(
        r#"const t = new EventTarget();
console.log("done");
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "done\n");
}

/// Handle-escape discipline (spec §2.4): an EventTarget binding read outside
/// the lane's allowed positions fails closed — the handle must never leak as
/// a number (node prints `EventTarget {}`; kali would print the raw handle).
#[test]
fn event_target_handle_escape_fails_closed() {
    let out = run_kali(
        r#"const t = new EventTarget();
console.log(t);
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Reassigned target bindings lose stable provenance — every later lane use
/// fails closed (the unstable_provenance_names rule).
#[test]
fn event_target_reassigned_binding_fails_closed() {
    let out = run_kali(
        r#"let t = new EventTarget();
t = new EventTarget();
t.addEventListener("tick", function () {});
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `new EventTarget()` OUTSIDE a declarator has no recordable provenance —
/// fail closed, don't emit an untracked handle.
#[test]
fn event_target_non_declarator_construction_fails_closed() {
    let out = run_kali(
        r#"new EventTarget();
console.log("done");
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ---------------------------------------------------------------------------
// Task 4: addEventListener + dispatchEvent emit arms — end-to-end runs and the
// fail-closed envelope. Every expected stdout below was verified against
// node v26.5.0 before being asserted.
// ---------------------------------------------------------------------------

/// Assert a fixture fails closed with E5506 (the shared envelope-deny shape).
fn assert_e5506(source: &str) {
    let out = run_kali(source);
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Happy path, function scope, capturing listener: dispatch is SYNCHRONOUS
/// (the mutation is visible on the line after dispatchEvent) and the return
/// value is true. node v26.5.0: "before=0\nlistener n=1\nafter=1\ndispatched\n".
#[test]
fn event_dispatch_runs_capturing_listener_synchronously() {
    let out = run_kali(
        r#"function owner() {
  const t = new EventTarget();
  let n = 0;
  t.addEventListener("tick", () => {
    n += 1;
    console.log("listener n=" + n);
  });
  console.log("before=" + n);
  const ok = t.dispatchEvent(new CustomEvent("tick"));
  console.log("after=" + n);
  if (ok) {
    console.log("dispatched");
  }
}
owner();
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "before=0\nlistener n=1\nafter=1\ndispatched\n"
    );
}

/// Two listeners fire in registration order. node v26.5.0: "a\nb\n".
#[test]
fn event_listeners_fire_in_registration_order() {
    let out = run_kali(
        r#"const t = new EventTarget();
t.addEventListener("tick", function () { console.log("a"); });
t.addEventListener("tick", function () { console.log("b"); });
t.dispatchEvent(new CustomEvent("tick"));
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a\nb\n");
}

/// The same function registered twice fires once per dispatch (identity
/// dedup). node v26.5.0: "hit\n".
#[test]
fn event_duplicate_listener_dedups() {
    let out = run_kali(
        r#"const t = new EventTarget();
function onTick() {
  console.log("hit");
}
t.addEventListener("tick", onTick);
t.addEventListener("tick", onTick);
t.dispatchEvent(new CustomEvent("tick"));
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hit\n");
}

/// Dispatch with zero listeners returns true. node v26.5.0: "ok\n".
#[test]
fn event_dispatch_with_no_listeners_returns_true() {
    let out = run_kali(
        r#"const t = new EventTarget();
if (t.dispatchEvent(new CustomEvent("none"))) {
  console.log("ok");
}
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok\n");
}

/// A fn-alias declarator (`const g = f`) resolves through STABLE provenance to
/// the compiled function, so the listener registers and RUNS (the VERIFY note
/// on the plan's `event_alias_callback` pin: the resolver soundly produces
/// Resolved, so this is in-lane, not a deny). node v26.5.0: "x\n".
#[test]
fn event_alias_callback_resolves_and_runs() {
    let out = run_kali(
        r#"const t = new EventTarget();
function f() { console.log("x"); }
const g = f;
t.addEventListener("tick", g);
t.dispatchEvent(new CustomEvent("tick"));
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x\n");
}

// --- Envelope deny pins (each fails closed E5506) ---------------------------

/// A non-literal event type has no lowering (only string literals resolve).
#[test]
fn event_non_literal_event_name_fails_closed() {
    assert_e5506(
        "const t = new EventTarget(); let name = \"tick\"; t.addEventListener(name, function () {});\n",
    );
}

/// A listener declaring a parameter has no Event-object lowering yet.
#[test]
fn event_listener_with_parameter_fails_closed() {
    assert_e5506(
        "const t = new EventTarget(); t.addEventListener(\"tick\", (e) => { console.log(\"x\"); });\n",
    );
}

/// An options/capture third argument has no lowering.
#[test]
fn event_listener_options_arg_fails_closed() {
    assert_e5506("const t = new EventTarget(); t.addEventListener(\"tick\", function () {}, true);\n");
}

/// OUT-OF-LANE PRESERVATION (spec §2): a `dispatchEvent` argument that is not
/// an inline `new CustomEvent(<literal>)` — here a CustomEvent with a `detail`
/// (extra ctor arg) — on an in-lane receiver keeps PRE-LANE behavior: the
/// dispatch falls through to the backstop and is silently dropped, the build
/// SUCCEEDS (the browser web-baseline corpus relies on this — an in-lane target
/// with out-of-lane dispatches must stay deployable). The empty-body listener
/// makes the drop node-observationally inert here; the general silent-drop is
/// the inventoried Stage-P3 residual. node v26.5.0: "" (no output).
#[test]
fn event_custom_event_with_detail_out_of_lane_builds() {
    let out = run_kali(
        "const t = new EventTarget(); t.addEventListener(\"tick\", function () {}); t.dispatchEvent(new CustomEvent(\"tick\", { detail: 1 }));\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

/// OUT-OF-LANE PRESERVATION (spec §2): a bound (non-inline) event argument on
/// an in-lane receiver falls through to the backstop and BUILDS (same rationale
/// as `event_custom_event_with_detail_out_of_lane_builds`). No listener is
/// registered here, so the drop is node-observationally inert. node v26.5.0: "".
#[test]
fn event_bound_event_argument_out_of_lane_builds() {
    let out = run_kali(
        "const t = new EventTarget(); const ev = new CustomEvent(\"tick\"); t.dispatchEvent(ev);\n",
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

/// `removeEventListener` has no lowering; the handle-escape discipline forbids
/// any EventTarget method other than addEventListener/dispatchEvent — fail
/// closed rather than silently drop (a later dispatch would then diverge).
#[test]
fn event_remove_event_listener_fails_closed() {
    assert_e5506(
        "const t = new EventTarget(); function f() {} t.addEventListener(\"tick\", f); t.removeEventListener(\"tick\", f);\n",
    );
}

/// A captured (cross-function) EventTarget receiver has no provable provenance
/// in the inner function — its dispatch would be silently dropped (a proven
/// divergence: node fires the outer-registered listener). Fail closed.
#[test]
fn event_captured_receiver_fails_closed() {
    assert_e5506(
        "function outer() { const t = new EventTarget(); function inner() { t.dispatchEvent(new CustomEvent(\"tick\")); } inner(); } outer();\n",
    );
}

/// Out-of-lane preservation (spec §2 out-of-lane): a member `addEventListener`
/// on a NON-EventTarget receiver with a non-capturing listener keeps its
/// pre-lane behavior — an E3100 warning + zero-placeholder fallback, build
/// SUCCEEDS (a pre-existing residual inventoried for Stage P3; do NOT convert
/// to a deny here). Mirrors the browser package corpus `signal.addEventListener`
/// shape (an AbortSignal-like unknown receiver) minus its object-shape read.
/// The load-bearing property: the backstop lane did not widen.
#[test]
fn event_unknown_receiver_non_capturing_listener_still_builds() {
    let out = run_kali(
        r#"function attach(signal) {
  signal.addEventListener("abort", () => {});
}
console.log("built");
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "built\n");
}
