//! Regression (throw-fallout Stage 2 Lane C, fix wave 2): the release
//! specialization env (`specialize_layout_bindings`) must exclude MUTATED
//! bindings.
//!
//! Previously it was a third independently hand-rolled constant env with no
//! mutated-name awareness: at `--release`/`--release-advanced` it cloned the
//! PRE-MUTATION literal over every bound identifier occurrence
//! (`Object.keys(s)` → `Object.keys({a:1,b:2,c:3})`), producing a
//! literal-receiver enumeration that every downstream fold folds
//! unconditionally — so the block-nested-store, computed-store, and alias
//! stale classes all compiled to WRONG release wasm with exit 0 (stale keys
//! baked into the data section), and out-of-lane deletes were
//! literal-substituted and swallowed before codegen's default-deny arm.
//!
//! Post-fix: each stale class must REJECT at release build time (E5506-class,
//! no artifact), while the ELIGIBLE top-level delete+reinsert timeline program
//! (Probe 3) must still build AND execute byte-correctly — the ordered fold
//! runs before specialization, so the strip must not disturb folded literals.

use kali_cli::build::{compile_source_file, BuildMode};
use kali_cli::ApiSurface;
use kali_runtime::RuntimeCtx;
use std::fs;
use tempfile::tempdir;

fn compile(source: &str, mode: BuildMode) -> Result<Vec<u8>, String> {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.ts");
    fs::write(&path, source).expect("write source");
    compile_source_file(&path, mode, ApiSurface::Deno, &[], false, false)
        .map_err(|diagnostics| format!("{diagnostics:?}"))
}

fn compile_and_run(source: &str, mode: BuildMode) -> String {
    let wasm = compile(source, mode).unwrap_or_else(|err| panic!("compile failed: {err}"));
    let runtime = RuntimeCtx::new(None);
    let outcome = runtime
        .execute(&wasm)
        .unwrap_or_else(|diagnostics| panic!("execute failed under {mode:?}: {diagnostics:?}"));
    outcome.stdout
}

/// Alias of a mutated binding: `const s = r` after `r.b = 2`. Enumerating the
/// alias has no sound static shape — must reject, never fold r's stale
/// pre-store literal.
const ALIAS_OF_MUTATED: &str = "\
const r = { a: 1 };
r.b = 2;
const s = r;
console.log(Object.keys(s).join(\",\"));
";

/// Store nested inside a bare block: out-of-lane for the straight-line
/// timeline — must reject, never fold the pre-block shape.
const BLOCK_NESTED_STORE: &str = "\
const r = { a: 1, b: 2 };
{ r.b = 9; }
console.log(Object.values(r).join(\",\"));
";

/// Computed member store: out-of-lane (dot-only timeline) — must reject,
/// never fold the pre-store value.
const COMPUTED_STORE: &str = "\
const r = { a: 1 };
r[\"a\"] = 2;
console.log(Object.values(r)[0]);
";

/// ELIGIBLE top-level delete+reinsert (Probe 3): the ordered timeline fold
/// handles this soundly — release builds must keep folding it correctly.
const ELIGIBLE_DELETE_REINSERT: &str = "\
const r = { \"a\": 1, \"b\": 2, \"c\": 3 };
delete r.b;
r.b = 4;
const ks = Object.keys(r);
const vs = Object.values(r);
if (ks.length !== 3 || ks[0] !== 'a' || ks[1] !== 'c' || ks[2] !== 'b') throw new Error('keys stale');
if (vs[0] !== 1 || vs[1] !== 3 || vs[2] !== 4) throw new Error('values stale');
console.log('ok');
";

#[test]
fn release_rejects_stale_mutated_binding_enumerations() {
    for mode in [BuildMode::Release, BuildMode::ReleaseAdvanced] {
        for (name, source) in [
            ("alias-of-mutated", ALIAS_OF_MUTATED),
            ("block-nested-store", BLOCK_NESTED_STORE),
            ("computed-store", COMPUTED_STORE),
        ] {
            let result = compile(source, mode);
            assert!(
                result.is_err(),
                "{name} under {mode:?} must reject (fail-closed), not build stale wasm"
            );
        }
    }
}

#[test]
fn release_still_folds_the_eligible_timeline_program() {
    for mode in [BuildMode::Release, BuildMode::ReleaseAdvanced] {
        let stdout = compile_and_run(ELIGIBLE_DELETE_REINSERT, mode);
        assert_eq!(
            stdout, "ok\n",
            "eligible top-level delete+reinsert must keep folding correctly under {mode:?}"
        );
    }
}
