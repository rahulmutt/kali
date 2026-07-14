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
