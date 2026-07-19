//! End-to-end: a multi-argument `console.*` call must never silently discard
//! arguments.
//!
//! The defect: the console lowering had two lanes. The STATIC lane
//! (`render_console_call`) folds a call whose arguments are ALL compile-time
//! renderable into one interned string joined with `" "` — so all-literal calls
//! printed correctly. The DYNAMIC lane, taken as soon as ANY argument is not
//! statically renderable, emitted argument 0, called the console import, and
//! then emitted each remaining argument purely for side effects and `Drop`ped
//! it. `console.log(5, "y=" + 6)` printed `5` and exited 0 — no diagnostic.
//!
//! This is evidence-corrupting: fixtures self-check by logging several values
//! on one line, so a vanished argument makes a check pass vacuously.
//!
//! The invariant: NO SILENT OUTPUT LOSS. Every argument is printed,
//! space-separated, or the call fails closed with an E5506 naming the
//! construct. It is never dropped.
//!
//! Known, still-open sibling defect (NOT fixed here, deliberately): a COMPUTED
//! boolean renders as `1`/`0` rather than `true`/`false` — it needs a
//! `Repr::Boolean` axis, and `soundness_abort.rs` ratifies `1`/`0` rendering in
//! some paths. Tests below that involve a computed boolean pin kali's current
//! `1` rendering and say so; the point they defend is that the argument is
//! PRESENT, not how it renders.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-consolemultiarg-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// Assert that `src` runs successfully and emits `expected` as a COMPLETE line
/// on stdout.
///
/// Full-line equality is the whole point. A `contains`/prefix assertion passes
/// when the tail of the line is dropped, which is exactly how this defect hid
/// for so long: `assert!(stdout.contains("5"))` is satisfied by the broken
/// output `5` as well as the correct `5 y=6`.
fn assert_stdout_line(src: &str, expected: &str) {
    let out = run_source(src);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "expected a clean run, got exit {:?}\nstdout: {stdout}\nstderr: {stderr}",
        out.status.code()
    );
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines.contains(&expected),
        "SILENT ARGUMENT LOSS: no line equal to {expected:?}.\n\
         stdout lines: {lines:?}\nstderr: {stderr}"
    );
}

/// The controller's exact repro: a literal followed by a runtime concatenation.
/// node prints `5 y=6`; the broken lowering printed `5`.
#[test]
fn multi_arg_literal_then_runtime_concat_prints_both() {
    assert_stdout_line("console.log(5, \"y=\" + 6);\n", "5 y=6");
}

/// A non-literal in POSITION 0 followed by literals. Confirms the drop is not
/// specific to a trailing non-literal — the lane is selected by "any argument
/// is non-static", and then everything after argument 0 is lost regardless of
/// which argument made it dynamic. Broken output was `2`.
#[test]
fn multi_arg_non_literal_first_then_literals_prints_all() {
    assert_stdout_line(
        "function f(x) { return x + 1; }\nconsole.log(f(1), \"lit\", 7);\n",
        "2 lit 7",
    );
}

/// Three arguments with the non-literal in the MIDDLE. Broken output was `A`:
/// both the dynamic middle argument and the trailing literal were discarded.
#[test]
fn multi_arg_non_literal_in_middle_prints_all_three() {
    assert_stdout_line(
        "function f(x) { return x + 1; }\nconsole.log(\"A\", f(2), \"C\");\n",
        "A 3 C",
    );
}

/// Positive control on argument COUNT: five arguments, one dynamic. Any drop
/// anywhere shortens the line and fails the equality.
#[test]
fn multi_arg_five_arguments_one_dynamic_prints_all() {
    assert_stdout_line(
        "function f(x) { return x * 2; }\nconsole.log(1, \"b\", f(3), \"d\", 5);\n",
        "1 b 6 d 5",
    );
}

/// The all-literal STATIC lane must keep working byte-identically. This is the
/// regression guard on the lane that was already correct.
#[test]
fn multi_arg_all_literals_still_prints_all() {
    assert_stdout_line("console.log(1, \"two\");\n", "1 two");
    assert_stdout_line("console.log(\"a\", \"b\");\n", "a b");
    assert_stdout_line("console.log(1, 2, 3);\n", "1 2 3");
    assert_stdout_line("console.log(\"s\", true, null);\n", "s true null");
}

/// A boolean as argument 0, with a literal after it. Broken output was `1` —
/// the `"x"` was discarded entirely.
///
/// A `const`-bound comparison folds to a node the emitter shapes as `Boolean`,
/// so the shared `emit_as_string` ladder renders it `true` — matching node
/// exactly. This is not a boolean fix: it is the pre-existing `+`-concatenation
/// rendering, inherited for free by routing multi-argument console output
/// through the same ladder.
#[test]
fn multi_arg_boolean_does_not_drop_following_argument() {
    assert_stdout_line("const c = (1 === 1);\nconsole.log(c, \"x\");\n", "true x");
}

/// The KNOWN, still-open sibling defect, pinned as current truth so a later
/// `Repr::Boolean` axis has a tripwire: a boolean that reaches the console
/// through a seam carrying no `Boolean` shape — a function return, or a `let`
/// local read — renders `1` where node renders `true`.
///
/// That rendering is deliberately NOT fixed here (it needs the `Repr::Boolean`
/// axis, and `soundness_abort.rs` ratifies `1`/`0` rendering in some paths).
/// What these cases assert is the invariant that IS in scope: the trailing
/// argument is present. Under the old lowering both lines printed `1` alone.
/// When the boolean axis lands, these expectations become `true y` / `true z`.
#[test]
fn multi_arg_unshaped_boolean_renders_one_but_keeps_every_argument() {
    assert_stdout_line(
        "function g(a, b) { return a === b; }\nconsole.log(g(1, 1), \"y\");\n",
        "1 y",
    );
    assert_stdout_line("let d = (2 > 1);\nconsole.log(d, \"z\");\n", "1 z");
}

/// A runtime float argument alongside a literal — the float stringify seam has
/// its own coercion path, so it gets its own case.
#[test]
fn multi_arg_runtime_float_prints_both() {
    assert_stdout_line(
        "function f(x) { return x / 2; }\nconsole.log(\"v\", f(5));\n",
        "v 2.5",
    );
}

/// Every affected console sink shares the one lowering choke point, so each one
/// must print both arguments. `log`/`info`/`debug` go to stdout; `error`/`warn`
/// go to stderr (and `warn` carries kali's pre-existing `[warn] ` prefix, which
/// is a separate rendering convention and not what is under test). The
/// assertion is on the full two-argument payload either way, so a dropped tail
/// fails it.
#[test]
fn every_console_sink_prints_all_arguments() {
    for sink in ["log", "error", "warn", "info", "debug"] {
        let src = format!("const v = \"y=\" + 6;\nconsole.{sink}(\"S\", v);\n");
        let out = run_source(&src);
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            out.status.success(),
            "console.{sink}: expected a clean run, got exit {:?}\n\
             stdout: {stdout}\nstderr: {stderr}",
            out.status.code()
        );
        let combined = format!("{stdout}{stderr}");
        assert!(
            combined.contains("S y=6"),
            "SILENT ARGUMENT LOSS in console.{sink}: expected the full payload \
             `S y=6`.\nstdout: {stdout}\nstderr: {stderr}"
        );
    }
}

/// The no-silent-loss invariant stated directly: whatever the runtime can or
/// cannot render, a multi-argument call must either print every argument or
/// fail closed with a diagnostic. It must never exit 0 having printed a
/// strict prefix of its arguments.
#[test]
fn multi_arg_never_exits_zero_with_a_dropped_argument() {
    let src = "function f(x) { return x + 1; }\n\
               console.log(\"head\", f(1), \"tail-should-print\");\n";
    let out = run_source(&src);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if out.status.success() {
        assert!(
            stdout.lines().any(|l| l == "head 2 tail-should-print"),
            "SILENT ARGUMENT LOSS: exited 0 without printing every argument.\n\
             stdout: {stdout}\nstderr: {stderr}"
        );
    } else {
        let combined = format!("{stdout}{stderr}");
        assert!(
            combined.contains("E5506"),
            "a rejected multi-argument console call must fail closed with an \
             E5506 naming the construct.\nstdout: {stdout}\nstderr: {stderr}"
        );
    }
}
