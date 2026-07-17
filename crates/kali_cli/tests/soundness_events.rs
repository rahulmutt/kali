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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert_e5506(
        "const t = new EventTarget(); t.addEventListener(\"tick\", function () {}, true);\n",
    );
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "built\n");
}

/// I-1 e2e pin: a stale `clearInterval(0)` issued BEFORE any timer is scheduled
/// must be a no-op, not a poison of the first interval's re-arm. Because
/// `next_timer_id` starts at 0, the `setInterval` below is allocated id 0 — the
/// exact id the stale clear names. Pre-fix, the re-arm check ate the interval
/// after its first firing (kali printed only `1`); node v26.5.0 prints
/// `1\n2\n3\n`. The interval self-clears after its 3rd tick.
#[test]
fn stale_clear_before_setinterval_does_not_poison_first_interval() {
    let out = run_kali(
        "clearInterval(0);\nlet n = 0;\nlet id = 0;\nfunction tick(){ n = n + 1; console.log(n); if (n === 3) { clearInterval(id); } }\nid = setInterval(tick, 10);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n2\n3\n");
}

/// I-2 pin: a builtin scheduling name SHADOWED by a user binding is not the
/// genuine builtin — it must take the generic user-call treatment, not the
/// builtin exemption in `reject_anonymous_function_argument`. Pre-fix, a
/// shadowed `queueMicrotask(function(){})` was a TOTAL silent no-op (the
/// exemption skipped the anonymous-argument rejection AND codegen never
/// invoked the shadow body — node prints "shadow"). The sound outcome that
/// lands: the anonymous argument fails closed E5506, exactly as the generic
/// `let foo = function(f){…}; foo(function(){})` case already does (no longer
/// a silent no-op unique to builtin names).
#[test]
fn shadowed_scheduling_builtin_with_anonymous_arg_fails_closed() {
    assert_e5506(
        "let queueMicrotask = function(f){ console.log(\"shadow\"); };\nqueueMicrotask(function(){ console.log(\"cb\"); });\n",
    );
}

// ---------------------------------------------------------------------------
// Task 9 C-1 (scalar-only deny, user-ratified) — a deferred callback that
// captures a NON-lowered SCALAR-class binding (a param / string-repr /
// float-repr) reads a placeholder 0 in the deferred lane while node computes a
// real value. All four registration surfaces (setTimeout / setInterval /
// queueMicrotask / addEventListener) inherit the deny at the shared
// scheduling-callback choke point. Each source below RUNS in node (printing a
// real value) and printed a placeholder in kali before this fix — the pins
// assert the sound reject-don't-miscompile outcome (E5506).
// ---------------------------------------------------------------------------

/// Assert E5506 AND that the diagnostic names the expected capture class.
fn assert_e5506_capture_class(source: &str, class: &str) {
    let out = run_kali(source);
    assert!(!out.status.success(), "expected E5506, got exit 0");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains(&format!("a captured {class} binding")),
        "expected capture class '{class}' in diagnostic, got: {stderr}"
    );
}

/// p36e: setTimeout callback captures a PARAM. node: "i=6"; kali pre-fix:
/// "i=0" (the param never reaches the deferred env). Deny E5506 (param class).
#[test]
fn deferred_settimeout_captured_param_fails_closed() {
    assert_e5506_capture_class(
        "function main(i){ setTimeout(function(){ console.log(\"i=\" + i); }, 5); }\nmain(6);\n",
        "param",
    );
}

/// p55: queueMicrotask callback captures a PARAM. node: "i=3"; kali pre-fix:
/// "i=0". Deny E5506 (param class) — the queueMicrotask surface inherits it.
#[test]
fn deferred_queuemicrotask_captured_param_fails_closed() {
    assert_e5506_capture_class(
        "function main(i){ queueMicrotask(function(){ console.log(\"i=\" + i); }); }\nmain(3);\n",
        "param",
    );
}

/// p54: an addEventListener listener captures a PARAM, fired by a synchronous
/// dispatchEvent. node: "p=7"; kali pre-fix: "p=0". Deny E5506 (param class) —
/// the event surface (callback at children[2]) inherits the same choke point.
#[test]
fn deferred_event_listener_captured_param_fails_closed() {
    assert_e5506_capture_class(
        "function main(p){\n  const t = new EventTarget();\n  t.addEventListener(\"e\", function(){ console.log(\"p=\" + p); });\n  t.dispatchEvent(new CustomEvent(\"e\"));\n}\nmain(7);\n",
        "param",
    );
}

/// p53b: setTimeout callback captures a STRING-repr local. node: "hi"; kali
/// pre-fix: "" (empty — the placeholder string handle is 0). Deny E5506
/// (string class).
#[test]
fn deferred_settimeout_captured_string_fails_closed() {
    assert_e5506_capture_class(
        "function main(){ let s = \"hi\"; setTimeout(function(){ console.log(s); }, 1); }\nmain();\n",
        "string",
    );
}

/// p56: setTimeout callback captures a FLOAT-repr local. node: "1.5"; kali
/// pre-fix: "" (empty). Deny E5506 (float class).
#[test]
fn deferred_settimeout_captured_float_fails_closed() {
    assert_e5506_capture_class(
        "function main(){ let a = 1.5; setTimeout(function(){ console.log(a); }, 1); }\nmain();\n",
        "float",
    );
}

/// KEEP-ALLOWED residual pin (mirrors the webBaselineSmoke build invariant).
/// A listener that captures a promotable I64 scalar (`count`, lowered — works)
/// AND a NON-scalar zero-placeholder construct (`controller`, a
/// `new AbortController()` that reaches the E3100 placeholder fallback) must
/// still BUILD and RUN — it must NOT trip the scalar-only deny. The scalar-only
/// narrowing (vs. the reverted full deny) exists precisely to preserve this:
/// `controller` has NO real value kali ever computed (its in-callback read
/// equals its out-of-callback read of the same placeholder 0), so there is
/// nothing to diverge — the deny would be a spurious hard error breaking the
/// "unsupported constructs must still build (warn, not error)" contract. This
/// residual is lifted into the constrained set when Stage P3 gives
/// AbortController an `Object` repr. node runs it (the listener fires once).
#[test]
fn deferred_listener_nonscalar_placeholder_capture_still_builds() {
    let out = run_kali(
        "function main(){\n  const controller = new AbortController();\n  const t = new EventTarget();\n  let count = 0;\n  t.addEventListener(\"e\", function(){ count += 1; controller.abort(); });\n  t.dispatchEvent(new CustomEvent(\"e\"));\n  console.log(\"count=\" + count);\n}\nmain();\n",
    );
    assert!(
        out.status.success(),
        "the non-scalar placeholder capture must still build/run; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "scalar-only deny must NOT fire on a non-scalar placeholder capture; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
