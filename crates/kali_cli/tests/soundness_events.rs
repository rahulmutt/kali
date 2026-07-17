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
// Task 9 C-1 FINAL — DEFAULT-DENY over an allowlist at the shared
// deferred-callback choke point. A deferred callback restores captures through
// the OWNER's env-record pointer, but the owner frame and its arena are gone
// when the callback fires: only a BY-VALUE promoted scalar cell (a depth-1 i64
// stored inline in the record) survives. Every other capture reads a
// placeholder in the deferred lane while node computes a real value. The
// earlier scalar-only DENYLIST leaked three whole classes — captured OBJECTS
// (repr I64 or `Object`), scalars laundered THROUGH an object field, and
// param-ALIAS locals — so the deny was flipped to an ALLOWLIST: everything is
// denied UNLESS it is a by-value scalar OR a provable zero-placeholder
// construct (`new AbortController()`). All four registration surfaces
// (setTimeout / setInterval / queueMicrotask / addEventListener) inherit it.
// The five param/string/float scalar pins below are now SUBSUMED by the
// default (they were the original denylist entries); the five object/alias
// pins that follow are the holes the allowlist flip closed (Task 9 C-1 final
// verification probes b2/b7/b2b/b5/b3).
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

// --- Allowlist-flip closures (Task 9 C-1 final verification probes) ---------
// Each RUNS in node (a real value) and printed a placeholder 0 in kali before
// the flip; the scalar-only denylist ALLOWED every one of them. The allowlist
// now denies them E5506 (reject-don't-miscompile).

/// Probe b2: a setTimeout callback captures a local OBJECT and reads a field.
/// node: "x=4"; kali pre-flip: "x=0" (the object pointer is not restored — it
/// aims into the owner's reclaimed arena). The scalar-only form allowed this
/// because the object was `is_scalar == false` and not a param.
#[test]
fn deferred_settimeout_captured_object_read_fails_closed() {
    assert_e5506_capture_class(
        "function m(){ const o = { x: 4 }; setTimeout(function(){ console.log(\"x=\" + o.x); }, 0); }\nm();\n",
        "local",
    );
}

/// Probe b7: the SAME program reads the object field synchronously (`sync=4`,
/// correct) and then in the deferred callback (`x=0`) — kali self-contradicts,
/// which DISPROVES the falsified rationale that a captured object's in-callback
/// read equals its out-of-callback read. The materialized sync value is exactly
/// what the deferred lane loses. Deny E5506.
#[test]
fn deferred_settimeout_captured_object_self_contradiction_fails_closed() {
    assert_e5506_capture_class(
        "function m(){ const o = { x: 4 }; console.log(\"sync=\" + o.x); setTimeout(function(){ console.log(\"x=\" + o.x); }, 0); }\nm();\n",
        "local",
    );
}

/// Probe b2b: a captured object field is MUTATED inside the deferred callback
/// (`o.x = o.x + 1`). node: "x=5"/"x2=5"; kali pre-flip: "x=0"/"x2=0". Deny
/// E5506 (a mutation through a non-restored pointer writes reclaimed memory).
#[test]
fn deferred_settimeout_captured_object_mutation_fails_closed() {
    assert_e5506_capture_class(
        "function m(){ const o = { x: 4 }; setTimeout(function(){ o.x = o.x + 1; console.log(\"x=\" + o.x); }, 0); }\nm();\n",
        "local",
    );
}

/// Probe b5: a scalar param is laundered INTO an object field (`o.x = i`) and
/// the field is read in the callback. node: "x=9"; kali pre-flip: "x=0". The
/// object here even earns an `Object` repr (so it passed the OLD `if lowered`
/// early-out) yet still reads 0 deferred — the allowlist requires a by-value
/// SCALAR, not merely a `cell_is_promotable` object. Deny E5506 (object class).
#[test]
fn deferred_settimeout_scalar_laundered_into_object_fails_closed() {
    assert_e5506_capture_class(
        "function m(i){ const o = { x: 0 }; o.x = i; setTimeout(function(){ console.log(\"x=\" + o.x); }, 0); }\nm(9);\n",
        "object",
    );
}

/// Probe b3 (=p36b): a param is aliased into a `let` (`let a = i`) and the alias
/// is captured. node: "a=1"; kali pre-flip: "a=0". The alias is `is_scalar ==
/// false` with a NON-param name, so the old `function_param_names` consult (which
/// only caught DIRECT param captures) missed it. Deny E5506.
#[test]
fn deferred_settimeout_param_alias_capture_fails_closed() {
    assert_e5506_capture_class(
        "function m(i){ let a = i; setTimeout(function(){ console.log(\"a=\" + a); }, 0); }\nm(1);\n",
        "local",
    );
}

/// KEEP-ALLOWED residual pin (mirrors the webBaselineSmoke build invariant).
/// A listener that captures a promotable I64 scalar (`count`, a by-value cell —
/// the FIRST allowlist entry, restorable) AND a provable zero-placeholder
/// construct (`controller`, a `new AbortController()` that lowers to the
/// drop-and-push-0 placeholder — the SECOND allowlist entry) must still BUILD
/// and RUN. This is the ONE class the default-deny allowlist keeps: `controller`
/// reads the same placeholder 0 in the owner's own body, so the deferred read of
/// 0 introduces NO divergence (unlike a real object — see the b2/b7 pins above).
/// Denying it would be a spurious hard error breaking the "unsupported
/// constructs must still build (warn, not error)" contract. Lifted into the
/// constrained set when Stage P3 gives AbortController an `Object` repr. node
/// runs it (the listener fires once).
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

// --- Task 9 rider: hand-mirrored exclusion-list tripwires -------------------
// `declarator_init_is_placeholder_construct`'s Array/Uint8Array/EventTarget
// exclusion list (crates/kali_codegen/src/lower.rs) is a HAND-MIRRORED NAME
// LIST: `unlowered_capture_denied`'s allowlist branch 2 admits a captured
// `new X()` binding only because its lowering is drop-and-push-0 TODAY. See
// the §8.6 inventory bullet in docs/superpowers/followups/stageD-triage.md.

/// Task 9 rider probe c1 — DELIBERATE TRIPWIRE, NOT a correctness pin. `Set`
/// is not in the exclusion list, so kali currently treats a bound `new
/// Set(...)` as a zero-placeholder construct and ALLOWS the deferred
/// capture under `unlowered_capture_denied`'s allowlist branch 2 — sound
/// TODAY only because `new Set(...)` itself still lowers to the
/// drop-and-push-0 placeholder (same class as `new AbortController()`).
/// node v26.5.0 prints the REAL values `sync=3`/`cb=3` (a `Set` with 3
/// elements, unchanged across the callback); kali (build succeeds, no
/// warnings) prints `sync=0`/`cb=0` — both sides read the SAME placeholder,
/// so there is no cross-callback DIVERGENCE today (hence "sound", not
/// "correct"). This test pins kali's CURRENT behavior exactly. The day
/// `new Set` gains a real lowering, this assertion goes RED instead of the
/// allowlist silently starting to leak a real (now-divergent) value —
/// signaling that `declarator_init_is_placeholder_construct`'s exclusion
/// list must gain `Set` alongside `Array`/`Uint8Array`/`EventTarget`.
#[test]
fn deferred_capture_of_bound_set_placeholder_tripwire() {
    let out = run_kali(
        "function m(){ const s = new Set([1,2,3]); console.log(\"sync=\" + s.size); setTimeout(function(){ console.log(\"cb=\" + s.size); }, 0); }\nm();\n",
    );
    assert!(
        out.status.success(),
        "expected build+run success (deliberate tripwire — the current \
         allowlist admits a bound `new Set(...)` capture); stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "sync=0\ncb=0\n",
        "kali's current (sound-by-coincidence) output for a captured bound \
         `new Set(...)` diverged from the pinned same-0-both-sides value — if \
         `Set` now prints something other than the placeholder 0 (e.g. a real \
         size), `Set` must be added to the exclusion list in \
         `declarator_init_is_placeholder_construct` (crates/kali_codegen/src/lower.rs) \
         before this pin is updated, or the allowlist has started leaking a \
         real value through a class it assumes is a zero placeholder"
    );
}

/// Task 9 rider (reviewer probe c3) — pins the `is_function_like` walk-stop
/// in `binding_is_placeholder_construct` (crates/kali_codegen/src/intrinsics/host.rs):
/// a nested function's OWN `new AbortController()` binding must not be
/// attributed to an OUTER binding of the same name when the choke point
/// checks whether the OUTER capture is a zero-placeholder construct. Shape:
/// `outer` binds a REAL object `const c = { x: 4 }` and registers a
/// `setTimeout` callback reading `c.x`; a nested function `inner`, defined
/// inside `outer`, separately shadows the name with its own
/// `const c = new AbortController()` (never itself captured — it exists
/// only as a same-name decoy). Without the walk-stop, the search over
/// `outer`'s body would wrongly descend into `inner`, find ITS placeholder
/// declarator, and WRONG-ALLOW outer's real-object capture — a silent
/// value-losing miscompile (outer's deferred `c.x` would read 0 instead of
/// node's real 4). node v26.5.0 prints `x=4`. On HEAD the walk-stop keeps
/// this denied: E5506, capture class "local" (same class as probe b2's
/// plain captured-object pin). This test makes
/// `binding_is_placeholder_construct`'s walk-stop safety claim
/// self-sustaining — a regression here is a silent value-losing miscompile,
/// not a mere behavior change.
#[test]
fn deferred_capture_nested_shadow_placeholder_denies() {
    assert_e5506_capture_class(
        "function outer(){\n  const c = { x: 4 };\n  function inner(){ const c = new AbortController(); }\n  inner();\n  setTimeout(function(){ console.log(\"x=\" + c.x); }, 0);\n}\nouter();\n",
        "local",
    );
}
