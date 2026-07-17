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

/// Same shape as `run_kali` but drives the `kali test` harness (registers and
/// runs `Kali.test(...)` bodies) instead of `kali run`.
fn run_kali_test(source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&path)
        .output()
        .expect("run kali test")
}

/// The bug, end to end: the arrow's body must NOT execute inline.
/// Pre-stage kali prints `1` (body flattened into module scope and run).
#[test]
fn a_block_arrow_callback_body_does_not_run_inline() {
    let out = run_kali(
        r#"let ran = 0;
queueMicrotask(() => {
  ran = 1;
});
console.log("sync ran=" + ran);
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The callback is deferred to the microtask FIFO, so `ran` is still 0 here.
    // (It runs during the post-_start drain; node agrees.)
    assert!(
        String::from_utf8_lossy(&out.stdout).starts_with("sync ran=0"),
        "callback body ran inline: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// Step 6 probe requirement: `a_block_arrow_callback_body_does_not_run_inline`
/// alone CANNOT distinguish "deferred" from "dropped" — `sync ran=0` prints
/// identically whether the callback runs later or never runs at all. This
/// test closes that gap with a SECOND print, emitted from INSIDE the callback
/// body, on a line the sync `console.log` above it can never reach. If the
/// `queueMicrotask` emit arm silently drops the callback (a no-op), this is
/// the assertion that goes red — proven by the Step 6 re-mask probe.
#[test]
fn a_queued_microtask_callback_actually_runs_during_the_drain() {
    let out = run_kali(
        r#"let ran = 0;
queueMicrotask(() => {
  ran = 1;
  console.log("callback ran=" + ran);
});
console.log("sync ran=" + ran);
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout, "sync ran=0\ncallback ran=1\n",
        "the queued callback must actually run during the post-_start microtask \
         drain (deferred), not merely be registered-and-dropped: {}",
        stdout
    );
}

/// An anonymous callback handed to a consumer that CANNOT invoke it must fail
/// closed — never compile to a function nobody calls.
#[test]
fn an_anonymous_callback_to_an_uninvocable_callee_fails_closed() {
    let out = run_kali(
        r#"function takesCallback(cb) { return 1; }
takesCallback(() => {
  console.log("never");
});
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A named function expression keeps its own name; two anonymous arrows in the
/// same module get DISTINCT names. If the pre-pass reused one name, the second
/// body would overwrite the first and this prints the wrong value.
#[test]
fn anonymous_functions_get_distinct_stable_names() {
    let out = run_kali(
        r#"const a = () => 1;
const b = () => 2;
console.log(a() + b());
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}

/// The shape of the predicted regressions: a callback body doing compound
/// assignment, an update expression, and a logical (`||=`) assignment on
/// locals, plus `typeof` and `new`. Today these are MASKED because the
/// flattened body sits in module scope. Once bodies are real function
/// scopes, they must still compile and run — that is what repr-tracking
/// buys.
///
/// Deviations from the task brief's verbatim snippet, each verified against
/// a freshly built binary to confirm it reproduces IDENTICALLY at bare
/// module scope (i.e. is unrelated to function-scope repr-tracking, the
/// thing this task actually changes) before being substituted:
///
/// - `n ??= ...` / `s ??= "x"` (nullish-assignment on a plain scalar) is
///   rejected with E5506 ("null and 0 are indistinguishable for a scalar
///   value; only a for-in-key alias with a null sentinel supports `??=`") at
///   EVERY scope, always — a separate, deliberate, pre-existing restriction
///   (`resolve/expression.rs` near `for_in_key_shape`) with no connection to
///   this task. Replaced with `m ||= 5;` (logical-OR assignment), which
///   exercises the same "compound/logical-assign on a local inside a real
///   function scope" resolver path this task fixes, without that unrelated
///   restriction: confirmed RED (E5506) pre-fix / GREEN post-fix inside a
///   function-expression body.
/// - `typeof n` on a plain integer local (after `n += 2`) always prints `0`,
///   never `"number"` — `kali_codegen`'s `typeof_static_text` only
///   classifies a proven-FLOAT or literal operand as `"number"`; a plain I64
///   local is deliberately left unclassified (an I64 may be an internal
///   handle) and falls through to a placeholder. Reproduces identically at
///   module scope. Replaced with `typeof true` ("boolean" — statically
///   classified), read with a solo `console.log(t)` (see next point).
/// - The original single concatenated `console.log(n + " " + s + " " + t)`
///   is split: numeric locals (`n`, `m`) are joined with literal-rooted `+`
///   concatenation (works — codegen unconditionally number-to-string-coerces
///   an I64 operand), but a STRING VALUE COMPUTED INSIDE a function-
///   expression/arrow/class-method body (e.g. `typeof true`, or any local
///   reassigned to a string via `??`) must NOT be fed into further `+`
///   concatenation: `kali_types::repr_infer::infer_reprs` — a SEPARATE
///   analysis pass from the one this task changes, run once up front
///   (`resolve/mod.rs:350`) — has no `FunctionExpression` / `Arrow-
///   FunctionExpression` / `ClassBody` / `MethodDefinition` arms at all (grep
///   confirms zero hits), so it never seeds a String repr for any binding
///   declared inside such a body. `+`-concatenating such a value either
///   fails closed with E3200 (safe — confirmed for a `+` node whose operand
///   is DIRECTLY the unproven identifier) or, when chained behind an
///   already-proven-string `+` (`... + t` at the tail of a
///   left-associative chain), silently prints the raw i64 handle bits
///   instead of the string (confirmed: `"s=" + s` prints garbage after
///   `s = s ?? "x"` inside a callback, reproducing IDENTICALLY on the
///   pre-Task-3 binary and even at bare module scope) — a genuine,
///   pre-existing, OUT-OF-SCOPE silent-miscompile class this task does not
///   touch (see the task-3 report's "concerns" section). A solo
///   `console.log(t)` sidesteps this entirely: `console.log` tag-detects a
///   string handle at RUNTIME and does not need the static repr proof.
///   (Multi-argument `console.log(n, m, t)` was also tried as an
///   alternative to a second concatenation and was independently found to
///   print only its first argument, everywhere, including bare module
///   scope with no function involved — yet another unrelated pre-existing
///   gap, avoided by keeping every `console.log` call single-argument.)
/// - `new Box(9)` is kept exactly as the brief has it, but (like the brief)
///   its result is never read back: `this.v = v` inside a constructor does
///   not actually land the field (`b.v` prints `0`, not `9`, reproducing
///   identically at module scope) — another pre-existing, unrelated gap.
///   The assertion here is only that constructing it compiles and runs
///   without trapping inside a real function scope.
#[test]
fn a_function_expression_body_supports_compound_and_typeof_and_new() {
    let out = run_kali(
        r#"function Box(v) { this.v = v; }
const f = function () {
  let n = 1;
  n += 2;
  n++;
  let m = 0;
  m ||= 5;
  console.log(n + " " + m);
  let t = typeof true;
  console.log(t);
  let b = new Box(9);
};
f();
"#,
    );
    assert!(
        out.status.success(),
        "expected the body to compile and run, got:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4 5\nboolean\n");
}

/// Task 3 (B): an anonymous `export default function () {}` must compile and
/// run cleanly — no panic, no spurious diagnostic — now that the pre-pass
/// (`name_anon_functions.rs`) fills its empty `name` the same way
/// `kali_hir`'s fallback already does, closing the naming divergence between
/// the two crates (`kali_types::resolve_export_default` binds the function's
/// scope directly under `func.name`, with no fallback of its own — before
/// this fix it would bind under the literal empty string). This body is
/// intentionally simple (no compound-assign/typeof/new): `resolve_export_default`
/// does not push onto `current_function` (see the doc comment on
/// `TypeContext::current_function_scopes`, which explicitly defers that to a
/// later task), so it stays outside Task 3's repr-tracking scope — this test
/// is only about the NAME agreeing, not about repr-tracking this body too.
#[test]
fn anonymous_export_default_function_compiles_and_runs() {
    let out = run_kali(
        r#"export default function () {
  return 1;
};
console.log("ok");
"#,
    );
    assert!(
        out.status.success(),
        "expected an anonymous default-export function to compile and run, got:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ok\n");
    assert!(
        String::from_utf8_lossy(&out.stderr).is_empty(),
        "expected no diagnostics, got stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Stage 5 recorded `class C { run(){ return 42; } } new C().run()` → 0.
/// Class-method bodies are one of the untracked function-shaped scopes
/// (kali_types/src/context.rs:48-54), so Task 3 is the HYPOTHESISED root cause.
/// This test decides it either way.
///
/// VERDICT (Task 4, fresh binary at this commit): the hypothesis is
/// FALSIFIED. Task 3's repr-tracking did not touch this path — kali still
/// prints `0` where node prints `42`. Root cause is a SEPARATE lowering gap
/// (class-method `return` value is dropped, not a repr-tracking problem);
/// see docs/superpowers/followups/throw-fallout-stage6-triage.md section 9.
/// Left `#[ignore]`d with assertions intact per the task brief — do not
/// widen this stage to chase it.
#[test]
#[ignore = "class-method return lowering — separate root cause, see stage6 triage"]
fn class_method_bodies_return_their_value() {
    let out = run_kali(
        r#"class C {
  run() {
    return 42;
  }
}
console.log(new C().run());
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}

/// Stage-6 probe 3a pinned: a feature-rich block-arrow callback body must
/// run DEFERRED with correct ordering, not inline — the module lines print
/// FIRST (`MODULE-END-acc`, then `console.log(acc)` reads the still-`0`
/// pre-drain binding), the microtask drains after (`INSIDE-CALLBACK`, then
/// the mutated value). node v26.5.0: "MODULE-END-acc\n0\nINSIDE-CALLBACK\n15\n".
/// Re-verify with node before asserting. Pre-D3 kali inverted the order and
/// leaked the mutated `acc`.
///
/// DEVIATION from the task-7 brief's verbatim snippet, verified ORTHOGONAL to
/// the arrow un-flatten this task lands: the brief wrote the callback's module
/// write as `acc = value + b.n` (module binding `=` an expression CONTAINING a
/// member access). That specific shape — `<module-scope-let> = <expr with a
/// `.field` member read>` written from inside ANY function body — pre-existingly
/// mis-parses (emits `E8001 unsupported unary operator 'n'` + `E8001 binary
/// operator '='`, then misclassifies the WRITE as a module-binding READ →
/// `E5506`). Confirmed to reproduce IDENTICALLY with a plain SYNCHRONOUS NAMED
/// function (`function cb(){ let b=new Box(); acc = b.n; } cb();`) — i.e. it is
/// wholly unrelated to arrows / deferral / the un-flatten, and un-fixed by this
/// task. (A sibling pre-existing bug: a `.field` read inside a function lowers
/// to `0`, so `b.n` would not read `4` anyway.) The RHS member access is moved
/// into an unobserved local `probe` (still exercising `new` + `.field` read in
/// the body — feature-rich), and the module write becomes `acc = value` (no
/// member in the RHS → the working lane). The load-bearing property — a
/// feature-rich block-arrow body running DEFERRED in correct order, not inline
/// — is unchanged and node+kali agree byte-for-byte.
#[test]
fn a_feature_rich_block_arrow_callback_defers_with_correct_ordering() {
    let out = run_kali(
        r#"class Box { constructor() { this.n = 4; } }
let acc = 0;
queueMicrotask(() => {
  let value = 1;
  value += 2;
  value *= 5;
  let b = new Box();
  let probe = b.n + value;
  acc = value;
  console.log("INSIDE-CALLBACK");
  console.log(acc);
});
console.log("MODULE-END-acc");
console.log(acc);
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "MODULE-END-acc\n0\nINSIDE-CALLBACK\n15\n"
    );
}

/// Stage-6 probe 4 pinned: a block-arrow `Kali.test` callback registers a
/// REAL test (total: 1), and its body does NOT run inline at module scope.
/// Pre-D3: the body ran between the module lines and the harness printed a
/// vacuous `ok 1` with zero registered tests.
#[test]
fn a_block_arrow_kali_test_registers_a_real_test() {
    let out = run_kali_test(
        r#"console.log("A-module-start");
Kali.test('real-test', () => { console.log("B-inside-test-body"); });
console.log("C-module-end");
"#,
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The body line must come AFTER module end (deferred to the test run),
    // and the run must report a registered, passing test.
    let a = stdout.find("A-module-start").expect("module start printed");
    let c = stdout.find("C-module-end").expect("module end printed");
    let b = stdout.find("B-inside-test-body").expect("test body ran");
    assert!(
        a < c && c < b,
        "test body ran inline at module scope: {stdout}"
    );
    assert!(
        stdout.contains("ok 1"),
        "expected a passing registered test: {stdout}"
    );
}

/// Arrow spelling of deferred_queue_microtask_capturing_callback_runs_with_env —
/// the un-flatten routes `() => {…}` through the same FunctionExpression lane.
/// node v26.5.0: "sync=5\nmt=6\n".
#[test]
fn deferred_queue_microtask_capturing_block_arrow_runs_with_env() {
    let out = run_kali(
        r#"function owner() {
  let base = 5;
  queueMicrotask(() => {
    base += 1;
    console.log("mt=" + base);
  });
  console.log("sync=" + base);
}
owner();
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nmt=6\n");
}

/// Arrow spelling of the setTimeout capturing fixture — `() => {…}` routes
/// through the same FunctionExpression lane, deferred and env-threaded.
/// node v26.5.0: "sync=5\nmt=6\n".
#[test]
fn deferred_set_timeout_capturing_block_arrow_runs_with_env() {
    let out = run_kali(
        r#"function owner() {
  let base = 5;
  setTimeout(() => {
    base += 1;
    console.log("mt=" + base);
  }, 0);
  console.log("sync=" + base);
}
owner();
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nmt=6\n");
}
