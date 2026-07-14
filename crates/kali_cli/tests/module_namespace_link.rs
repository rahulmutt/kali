//! End-to-end acceptance tests for the AST module-linking pass
//! (throw-fallout Stage 5). Unlike `kali_cli`'s in-crate `module_link.rs`
//! unit tests (which only assert the AST rename/append/order happened),
//! these tests actually RUN the built binary and assert real stdout — the
//! false-confidence gap a review finding on Task 7 identified: an existing
//! unit test (`append_linked_functions_still_renames_genuine_sibling_call_no_shadow`)
//! asserted the sibling-call rename happened but never checked the result
//! actually RESOLVED, so it stayed green over a defect where the linked
//! clones were appended in alphabetical (`BTreeMap` key) order instead of
//! dependency order — whenever a caller happened to sort before its callee,
//! the appended program failed (or, worse, silently produced a wrong value)
//! at the resolver, not at `append_linked_functions` itself.
//!
//! (Task 8 extends this file with the fuller Stage 5 acceptance suite; this
//! file starts minimal, with just the two dependency-order pins the Task 7
//! fix requires.)

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Runs `node <main_path>` with `dir` as the working directory (so relative
/// `import`/`import()` specifiers resolve the same way they do for the kali
/// invocation under test). Every fixture this helper is called on is valid,
/// unmodified ES that node can execute directly — no kali-specific rewriting
/// involved — so a straight `node` run is a faithful oracle for "what should
/// this program actually print".
fn node_output(dir: &Path, main_path: &Path) -> Output {
    Command::new("node")
        .current_dir(dir)
        .arg(main_path)
        .output()
        .expect("run node")
}

/// `helper` is declared BEFORE `f` in the linked module's source (`f` calls
/// `helper`). Node prints `inside helper`, `7`, `main loaded` (verified
/// against a real `node` run of this exact fixture). A `Number` return
/// (`return 7`, not `return 7n`/`String(...)`) and a call made DIRECTLY at
/// the `console.log` site (never bound to a `const` first) are deliberate:
/// two PRE-EXISTING, unrelated kali codegen bugs — `String(bigint)` folds to
/// `0`, and a `const` bound to a call re-evaluates it at every use,
/// duplicating side effects — would otherwise corrupt this test's expected
/// output for reasons that have nothing to do with the module-linking
/// dependency-order fix under test here.
#[test]
fn run_supports_namespace_linked_sibling_call_helper_declared_before_export() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        r#"function helper() { console.log("inside helper"); return 7; }
export function f() { return helper(); }
"#,
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as ns from "./util.js";
console.log(ns.f());
console.log("main loaded");
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "inside helper\n7\nmain loaded\n"
    );
}

/// The mirror declaration order: `f` (the export, and the caller) is
/// declared BEFORE `helper` (the private callee) in the linked module's
/// source. This is the plan's own mandated Task 4/5 fixture shape, and the
/// exact shape the pre-fix alphabetical (`BTreeMap` key) append order got
/// wrong: `"f" < "helper"` alphabetically, which happens to match THIS
/// source order too, so the pre-fix code appended the caller's clone before
/// its callee's regardless of which order the module's source actually
/// declared them in. Same expected stdout as the forward-order test above —
/// proving the fix is genuinely dependency-driven, not order-of-appearance
/// driven.
#[test]
fn run_supports_namespace_linked_sibling_call_export_declared_before_helper() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        r#"export function f() { return helper(); }
function helper() { console.log("inside helper"); return 7; }
"#,
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as ns from "./util.js";
console.log(ns.f());
console.log("main loaded");
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "inside helper\n7\nmain loaded\n"
    );
}

// ---- Task 8: distinguishable-value acceptance suite ----
//
// The two tests above (and the 32 legacy bucket-#7 tests, unmodified by this
// task) only assert BUILDABILITY or "the program doesn't crash" — none of
// them distinguish a genuine call into the linked module's body from the
// pre-stage fail-open behavior, which silently returned an integer `0`
// wherever a namespace member read/call should have resolved (see the
// stage's triage doc, "Baseline reproducers" section, for the empirical
// `0`-instead-of-`7` mirage this replaces). Every GREEN fixture below is
// deliberately built so a regression back to `0` is impossible to miss: each
// linked function has a body side effect (`console.log("inside …")`) AND
// returns a non-zero, non-default value, and the assertions check BOTH the
// side effect ran and the value it returned actually reached `console.log`.
//
// Fixture-shape note (overrides the task brief — see task-8-brief.md and the
// controller's fixture override): the brief's original fixtures used
// `console.log(String(ns.lazyValue()))` and a `const value = await
// chunk.lazyValue()` binding. Both trip PRE-EXISTING, unrelated kali codegen
// bugs (`String(<bigint>)` folds to `0`; a `const` bound to a call
// re-evaluates the call at every use, duplicating side effects) that would
// corrupt the expected output for reasons that have nothing to do with this
// stage. Every fixture below instead: (a) returns a plain `Number` (`return
// 7`, never `7n`), and (b) calls the linked function DIRECTLY at the
// `console.log` call site, never through an intermediate `const`.

/// `import * as ns from "./util.js"` + a direct `console.log(ns.lazyValue())`
/// call. Distinguishable from the pre-stage fail-open `0`: node and kali must
/// both print `inside lazyValue` (the body ran) then `7` (the real return
/// value, not a falsy placeholder).
#[test]
fn static_namespace_member_call_runs_body_and_returns_value() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        r#"export function lazyValue() { console.log("inside lazyValue"); return 7; }
"#,
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as ns from "./util.js";
if (typeof ns.lazyValue !== 'function') { throw new Error('missing lazyValue export'); }
console.log(ns.lazyValue());
console.log("main loaded");
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "inside lazyValue\n7\nmain loaded\n");

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

/// `const chunk = await import("./lazy.js")` (the const binding IS the
/// provenance-proven binding — required and fine; Bug B, the corrupted
/// fixture the brief originally specified, is about binding a CALL's
/// result, not the import expression itself) + a direct
/// `await chunk.lazyValue()` call at the `console.log` site, run through the
/// browser lane (`--api browser`, `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`)
/// since a dynamic `await import(...)` only resolves through that lane.
#[test]
fn dynamic_import_member_call_runs_body_and_returns_value() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("lazy.js"),
        r#"export function lazyValue() { console.log("inside lazyValue"); return 7; }
"#,
    )
    .expect("write lazy.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const chunk = await import("./lazy.js");
  if (typeof chunk.lazyValue !== 'function') { throw new Error('missing lazyValue export'); }
  console.log(await chunk.lazyValue());
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "inside lazyValue\n7\nmain loaded\n");

    // This fixture is plain, valid ES (no kali-specific syntax) — a direct
    // `node main.js` run (no browser lane needed) is a faithful oracle.
    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

/// Same shape as the previous test, but the specifier is a template literal
/// built from a `const` (`` `./${name}` ``) rather than a bare string
/// literal — the dominant shape in the 32-name legacy bucket (26 of 32 names
/// are template-literal-harness tests per the triage doc).
#[test]
fn dynamic_import_template_literal_specifier_variant() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("lazy.js"),
        r#"export function lazyValue() { console.log("inside lazyValue"); return 7; }
"#,
    )
    .expect("write lazy.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const name = "lazy.js";
  const chunk = await import(`./${name}`);
  if (typeof chunk.lazyValue !== 'function') { throw new Error('missing lazyValue export'); }
  console.log(await chunk.lazyValue());
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "inside lazyValue\n7\nmain loaded\n");

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

/// `typeof ns.missing` (a real, non-exported name) must fold to
/// `"undefined"`, and `typeof ns.lazyValue` (a real export) to `"function"` —
/// proving the typeof fold distinguishes members that exist from ones that
/// don't, rather than defaulting both to the same value.
#[test]
fn typeof_missing_member_is_undefined_string() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        r#"export function lazyValue() { console.log("inside lazyValue"); return 7; }
"#,
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as ns from "./util.js";
console.log(typeof ns.missing);
console.log(typeof ns.lazyValue);
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "undefined\nfunction\n");

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

/// Two DIFFERENT linked modules export a function with the SAME name
/// (`tag`). Each namespace's call must route to its OWN module's body — this
/// is the acceptance proof for per-module mangling (`__link{index}_{name}`):
/// if either mangling or the call-callee rewrite were confused about which
/// module a binding belongs to, this would call the wrong body (or the same
/// body twice) instead of `inside A`/`1` then `inside B`/`2`.
#[test]
fn two_modules_same_export_name_route_to_respective_bodies() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("a.js"),
        r#"export function tag() { console.log("inside A"); return 1; }
"#,
    )
    .expect("write a.js");
    fs::write(
        dir.path().join("b.js"),
        r#"export function tag() { console.log("inside B"); return 2; }
"#,
    )
    .expect("write b.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as a from "./a.js";
import * as b from "./b.js";
console.log(a.tag());
console.log(b.tag());
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "inside A\n1\ninside B\n2\n");

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

/// `console.log(chunk)` — leaking the raw namespace binding value itself.
/// Pre-stage, this printed the raw specifier string (a silent fail-open);
/// this pass must reject it at compile time (E5506) instead.
#[test]
fn namespace_value_leak_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("lazy.js"),
        r#"export function lazyValue() { console.log("inside lazyValue"); return 7; }
"#,
    )
    .expect("write lazy.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const chunk = await import("./lazy.js");
  console.log(chunk);
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "expected a non-zero exit; stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
}

/// `chunk.notAnExport()` — calling a member that is not a real export of the
/// linked module (node raises a `TypeError` here at runtime; this pass
/// rejects it at compile time instead).
#[test]
fn non_export_member_call_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("lazy.js"),
        r#"export function lazyValue() { console.log("inside lazyValue"); return 7; }
function notAnExport() { return 1; }
"#,
    )
    .expect("write lazy.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const chunk = await import("./lazy.js");
  chunk.notAnExport();
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "expected a non-zero exit; stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
}

/// A namespace binding whose target module has a top-level statement other
/// than a plain `export function` declaration (here, a top-level
/// `console.log` — an impure/non-function top level) must be rejected: the
/// module-linking purity gate cannot safely append (and thus cannot safely
/// prove the shape of) a module it can't fully account for.
#[test]
fn impure_target_module_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("lazy.js"),
        r#"console.log("top level side effect");
export function lazyValue() { console.log("inside lazyValue"); return 7; }
"#,
    )
    .expect("write lazy.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const chunk = await import("./lazy.js");
  console.log(await chunk.lazyValue());
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "expected a non-zero exit; stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
}

/// GREEN guard: the pre-existing "statement-form" `await import(...)` (no
/// binding at all — no provenance is even collected, so this pass is a
/// no-op) must stay green exactly as it did before this stage. This
/// documents, rather than asserts away, a pre-existing and unrelated
/// divergence: the imported chunk's top-level `console.log("lazy loaded")`
/// never actually runs under kali (node DOES run it — verified separately),
/// so this test deliberately does NOT assert "lazy loaded" appears in
/// stdout, only that the program still completes and reaches "main loaded".
#[test]
fn statement_form_side_effect_import_stays_green() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("lazy.js"),
        "console.log(\"lazy loaded\");\n",
    )
    .expect("write lazy.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  await import("./lazy.js");
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Exact, not `contains`: this pins the pre-existing chunk-never-runs
    // divergence. `lazy.js` has a top-level `console.log("lazy loaded")` that
    // node WOULD run; kali does not. Asserting the exact stdout means that if
    // kali ever starts executing a statement-form chunk's top level, this test
    // fails and the divergence is re-examined deliberately, rather than
    // silently satisfying a `contains("main loaded")` check.
    assert_eq!(stdout, "main loaded\n", "stdout: {stdout}");
}

/// C1 (final whole-branch review): the specifier fold was SCOPE-BLIND — a
/// function PARAM named the same as a module-scope `const` specifier string
/// never displaced that const in the fold's inherited map, so `await
/// import(spec)` inside `load(spec)` linked the module the OUTER `spec`
/// named, not the one the CALLER actually passed. This exact fixture printed
/// `111` under kali (a.js — the module-scope const's target) against node's
/// `222` (b.js — the argument's target): a silent WRONG-MODULE link, exit 0,
/// no diagnostic. (The review's own probe used BigInt returns; this file's
/// established convention is a plain `Number` return — see the header note —
/// because `console.log(<bigint>)` is a PRE-EXISTING, unrelated kali/node
/// formatting divergence (`2` vs `2n`) that would corrupt the oracle
/// comparison for reasons unrelated to module linking. The wrong-module class
/// under test is identical either way.)
///
/// Post-fix the fold is gated on "bound exactly once in the whole file", so
/// `spec` (bound twice: the const AND the param) is unprovable, the binding
/// earns no provenance, and C2's default-deny rejects the USE at compile time
/// (E5506). A fail-closed reject is the intended outcome here — preferred, per
/// the review, over hand-rolling lexical-scope tracking to link it "correctly".
#[test]
fn param_shadowed_dynamic_import_specifier_is_rejected_not_silently_mislinked() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("a.js"),
        "export function which() { return 111; }\n",
    )
    .expect("write a.js");
    fs::write(
        dir.path().join("b.js"),
        "export function which() { return 222; }\nexport function onlyInB() { return 1; }\n",
    )
    .expect("write b.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"const spec = "./a.js";
async function load(spec) {
  const c = await import(spec);
  return c.which();
}
async function main() { console.log(await load("./b.js")); }
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "expected a compile-time reject, not a silent link; stdout: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
    // The pre-fix wrong-module answer must not be produced under ANY exit
    // status: `111` was a.js's `which()` (the module-scope const's target),
    // where node — whose real oracle output is asserted below — says `222`.
    assert!(
        !stdout.contains("111"),
        "the wrong-module value must never be printed; stdout: {stdout}"
    );

    // node oracle: the argument (`./b.js`) wins, so the true answer is 222.
    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node.stdout), "222\n");
}

/// C2 (final whole-branch review), `let`-bound shape: `collect_namespace_
/// provenance` proves only `const` bindings, so a `let`-bound dynamic import
/// earned no provenance — and, pre-fix, an empty provenance map made the whole
/// pass early-return, falling straight through to the pre-stage fail-open. This
/// exact fixture printed `0` under kali (a raw namespace/specifier value coerced
/// to a number) against node's `42`, with exit 0 and NO diagnostic.
///
/// Post-fix, an un-proven but USED namespace-shaped binding is default-denied
/// (E5506) — the binding is namespace-shaped and its value is read, so it is
/// exactly the "would leak a raw value" class this stage exists to close.
#[test]
fn let_bound_dynamic_import_namespace_binding_is_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        "export function greet() { return 42; }\n",
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  let c = await import("./util.js");
  console.log(c.greet());
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "expected a compile-time reject, not the silent pre-stage `0`; stdout: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "a rejected build must print nothing at all; stdout: {stdout}"
    );

    // node oracle: the real answer is 42 — never the `0` kali used to print.
    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node.stdout), "42\n");
}

/// C2 scoping, GREEN guard: an un-provable namespace binding that is never
/// USED must NOT be denied (node runs it fine, and denying it would only
/// create new reds). Composes with I1: unused ⇒ neither loaded nor denied,
/// so even an impure target module is harmless here.
#[test]
fn unused_unprovable_namespace_binding_stays_green() {
    let dir = tempdir().expect("tempdir");
    // Impure (a top-level `export const` fails the purity gate) AND the
    // binding is `let`-bound (unprovable) — but nothing reads `c`.
    fs::write(dir.path().join("impure.js"), "export const VERSION = 1n;\n")
        .expect("write impure.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  let c = await import("./impure.js");
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "main loaded\n");
}

/// I1 (final whole-branch review), GREEN guard: a STATIC namespace import of
/// a module that would fail the purity gate must not be loaded (or gated) at
/// all while its binding is unused. Pre-fix this was a hard E5506 build
/// failure on a program node runs without complaint.
#[test]
fn unused_static_namespace_import_of_impure_module_stays_green() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("impure.js"), "export const VERSION = 1n;\n")
        .expect("write impure.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as ns from "./impure.js";
console.log("main loaded");
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "main loaded\n");

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

/// Minor (final whole-branch review): `deny_unrewritten_uses` used to walk the
/// SPLICED-IN clone bodies too, so a linked module's own internal local that
/// happened to share a name with an entry provenance binding was wrongly
/// E5506'd — even though the two live in different files and different scopes.
/// Node runs this fine; so must kali, byte-for-byte.
#[test]
fn linked_module_internal_local_sharing_the_binding_name_stays_green() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("lib.js"),
        "export function calc() { const ns = 2; return ns; }\n",
    )
    .expect("write lib.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"import * as ns from "./lib.js";
console.log(ns.calc());
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "2\n");

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

// ---- C2 remainder (second whole-branch review round): ImportExpression position allowlist ----
//
// The first C2 fix (`deny_unproven_namespace_binding_candidates`) is a DENYLIST OF BINDING
// SHAPES — it only records a candidate at a `VariableDeclarator.init` or a relative
// `import * as` specifier. Anything reaching a member access through some OTHER route was never
// recorded at all, so it fell straight through to the pre-stage silent `0` — exit 0, no
// diagnostic. Every fixture below prints `42` under node (real, distinguishable output — never
// the `0` the pre-fix fail-open produced) and must now be REJECTED (E5506) by
// `deny_import_expressions_outside_allowlist`'s position-based default-deny.

/// Assignment form: `c = await import(...)`, not a declarator init.
#[test]
fn assignment_form_dynamic_import_is_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        "export function greet() { return 42; }\n",
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  let c;
  c = await import("./util.js");
  console.log(c.greet());
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "expected a compile-time reject, not the silent pre-stage `0`; stdout: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "a rejected build must print nothing at all; stdout: {stdout}"
    );

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node.stdout), "42\n");
}

/// Same assignment-form binding, but read through `typeof c.greet` instead of a call — both
/// value-escape routes through the same un-tracked assignment RHS must be rejected.
#[test]
fn assignment_form_dynamic_import_typeof_is_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        "export function greet() { return 42; }\n",
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  let c;
  c = await import("./util.js");
  console.log(typeof c.greet);
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "expected a compile-time reject, not the silent pre-stage `0`; stdout: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node.stdout), "function\n");
}

/// Inline member access on the import result — no binding at all:
/// `(await import(...)).greet()`.
#[test]
fn inline_member_access_on_import_result_is_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        "export function greet() { return 42; }\n",
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  console.log((await import("./util.js")).greet());
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "expected a compile-time reject, not the silent pre-stage `0`; stdout: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "a rejected build must print nothing at all; stdout: {stdout}"
    );

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node.stdout), "42\n");
}

/// Sequence-expression init: `const c = (0, await import(...))` — the whole `await import(...)`
/// is buried inside a `SequenceExpression`, not the direct (mod-parens) declarator init
/// `as_await_import_source` recognizes.
#[test]
fn sequence_expression_init_dynamic_import_is_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        "export function greet() { return 42; }\n",
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const c = (0, await import("./util.js"));
  console.log(c.greet());
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "expected a compile-time reject, not the silent pre-stage `0`; stdout: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node.stdout), "42\n");
}

/// Member/property sink: `box.m = await import(...)`, then `box.m.greet()`.
#[test]
fn member_sink_dynamic_import_is_rejected() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        "export function greet() { return 42; }\n",
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const box = { m: null };
  box.m = await import("./util.js");
  console.log(box.m.greet());
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success(),
        "expected a compile-time reject, not the silent pre-stage `0`; stdout: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&node.stdout), "42\n");
}

// ---- C2 remainder, GREEN guards: the new allowlist gate must not over-reach ----

/// Positive control: bindingless statement-form `await import(...)` must stay green (39 tests
/// beyond this file depend on this).
#[test]
fn position_allowlist_leaves_bindingless_statement_form_green() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("lazy.js"),
        "console.log(\"lazy loaded\");\n",
    )
    .expect("write lazy.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  await import("./lazy.js");
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "main loaded\n");
}

/// Positive control: the proven `const ns = await import("./x")` lane must still link and run for
/// real, distinguishable output.
#[test]
fn position_allowlist_leaves_proven_const_lane_green() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        "export function greet() { return 42; }\n",
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const chunk = await import("./util.js");
  console.log(chunk.greet());
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "42\n");

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

/// Positive control: `Object.freeze(...)`-wrapped specifier (a drained bucket-#7 fixture shape)
/// must still link and run.
#[test]
fn position_allowlist_leaves_object_freeze_wrapped_specifier_green() {
    let dir = tempdir().expect("tempdir");
    fs::write(
        dir.path().join("util.js"),
        "export function greet() { return 42; }\n",
    )
    .expect("write util.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  const chunk = await import(Object.freeze("./util.js"));
  console.log(chunk.greet());
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "42\n");

    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        stdout,
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
}

/// Positive control: an unused, unprovable (`let`-bound) namespace binding must not be denied by
/// the new position gate — the declarator-init position is allowlisted regardless of `kind`;
/// whether it earns real provenance (and the "unused is harmless" exemption) is the pre-existing
/// pipeline's job.
#[test]
fn position_allowlist_leaves_unused_unprovable_binding_green() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("impure.js"), "export const VERSION = 1n;\n")
        .expect("write impure.js");
    let main_path = dir.path().join("main.js");
    fs::write(
        &main_path,
        r#"async function main() {
  let c = await import("./impure.js");
  console.log("main loaded");
}
main();
"#,
    )
    .expect("write main.js");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "main loaded\n");
}
