# Browser Bundle Production Gaps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the two recorded production gaps in browser bundles: give bundles a reliable entry point for top-level program code (`start()`, so bare top-level `console.log` finally routes through the glue's `console_log` import in a browser), and provide a browser-native harness-page generator in `kali_runtime` (the existing prelude is node-only).

**Architecture:** (1) The bundle glue (`generate_browser_bundle_js` in `cmd_build.rs`) gains a memoized exported `start()` helper — in both the ESM and CJS template branches — that awaits the eager `instancePromise` (memory/heap are assigned in its `.then`, which registered first and therefore runs first) and invokes the wasm `_start` export exactly once. This mirrors what the wasmtime host (`execute.rs:167-176`) and the runtime harnesses (`harness.rs` `_start()` calls) already do; the glue was the only consumer that never called it. Explicit-not-eager: running `_start` on import would double-run top-level code under the runtime harness (which calls `_start` on its own `loadWithImports` instance) and would change import-time semantics for ~98 existing bundle tests. (2) `kali_runtime` gains `browser_bundle_harness_page(bundle_dir, body)` — an HTML page with a module script that defines `bundleJs` (the same body contract as the node-only `browser_bundle_harness_script`) but emits **no** `node:` imports and **no** fetch shim: the glue's own `fetch(wasmUrl)` works once the bundle is served over HTTP. The completion-binding name becomes a shared `pub const BROWSER_HARNESS_DONE_BINDING`. (3) The gated CDP smoke test consumes both — bare-top-level + exported-function fixture, `start()` + wrapper body, generated page — proving the whole story against real Chromium.

**Tech Stack:** Rust, wasm bundle JS glue (string templates inside `format!` — braces are doubled `{{ }}`), node harness lane, headless Chromium via the existing test-only CDP driver.

**Empirically validated before planning** (in a scratch dir, by hand-patching a built glue): the exact `start()` code below prints a top-level program's `console.log(1 + 2)` output once across two `start()` calls (memoization) under the node harness shim; a mixed fixture (top-level statement + `// kali-tree-shake: smoke` marker + exported fn) keeps BOTH paths — `start()` prints `3`, `smoke(1n,2n)` prints `7`. Without the marker the export wrapper is not emitted, so the fixture must keep the marker.

## Global Constraints

- Production scope is exactly: `crates/kali_cli/src/bin/cmd_build.rs` (append-only additions to the two glue template branches + the CJS `exported` line), `crates/kali_runtime/src/browser/harness.rs` (new const + new function; nothing existing modified), `crates/kali_runtime/src/browser/harness_tests.rs`, `crates/kali_runtime/src/lib.rs` (re-exports only). Test scope: new `crates/kali_cli/tests/browser_bundle_toplevel_start.rs`, and `crates/kali_cli/tests/browser_cdp_smoke.rs` + `crates/kali_cli/tests/cdp_driver/driver.rs` (Task 3 only). Docs: `docs/superpowers/specs/2026-07-01-browser-harness-node-preference-design.md` (Task 4 only).
- Do NOT touch the four hand-mirrored `kali:rt` JS import lists (`cmd_build.rs:1534`, `cmd_build.rs:1763`, `harness.rs:148`, `harness.rs:461`) or the glue's existing helpers (`load`, `loadWithImports`, `loadDynamicImport`), wrappers, `instantiate`, or `instancePromise` lines.
- The completion-binding name stays exactly `"__kaliHarnessDone"` — Task 2 promotes it to a constant; no behavior change.
- `cargo test --workspace` is NOT a usable gate in this repo (pre-existing chromium-sandbox failures). Gates are the specific lanes named per task.
- Gated browser lane (needs Chromium; available in this environment, runs in ~1-2s): `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`.
- Repo hygiene before every commit: `cargo fmt` leaves no diff; `cargo clippy -p kali_cli -p kali_runtime --tests -- -D warnings` clean at each task's commit.
- The glue templates live inside Rust `format!` raw strings: every literal JS `{`/`}` must be written `{{`/`}}`.

---

### Task 1: Glue `start()` helper (ESM + CJS) + node-lane integration test

**Files:**
- Modify: `crates/kali_cli/src/bin/cmd_build.rs` (two template branches inside `generate_browser_bundle_js`)
- Create: `crates/kali_cli/tests/browser_bundle_toplevel_start.rs`

**Interfaces:**
- Consumes: the glue's existing `instancePromise` (ESM template ~`cmd_build.rs:1648`, CJS equivalent) and the wasm `_start` export (synthetic; holds all top-level statements; emitted by `kali_codegen/src/lower.rs:288`).
- Produces (Task 3 relies on this): every emitted bundle glue exports `async function start()` — memoized, idempotent, runs `instance.exports._start` exactly once after `wasmMemory`/`wasmHeap` are assigned, returns `undefined`, propagates a trap as a rejected (and memoized) promise. ESM: a named export. CJS: a member of `exported`/`module.exports`.

Note on collisions: a user export named `start` would collide with the synthetic helper exactly as it already would for `load`/`loadWithImports`/`loadDynamicImport` — pre-existing hazard class, same precedent, no new handling.

- [ ] **Step 1: Write the failing integration test**

Create `crates/kali_cli/tests/browser_bundle_toplevel_start.rs` (modeled on `browser_number_predicates_bundle.rs`):

```rust
//! A bare top-level program in a browser bundle runs via the glue's exported
//! `start()` helper, routing its console output through the `console_log`
//! import — and runs at most once no matter how many times `start()` is called.
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn toplevel_program_runs_once_via_glue_start_helper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, "console.log(1 + 2);\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bundle_dir = dir.path().join("app");
    let harness_path = dir.path().join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.start();
await mod.start();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3\n"), "stdout: {stdout:?}");
    assert_eq!(
        stdout.matches("3\n").count(),
        1,
        "top-level code must run exactly once across repeated start() calls; stdout: {stdout:?}"
    );
}

#[test]
fn start_helper_is_present_in_both_glue_formats() {
    for format in ["esm", "cjs"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join("app.ts");
        fs::write(&source_path, "console.log(1 + 2);\n").expect("write source");

        let mut command = Command::new(kali_bin());
        command
            .current_dir(dir.path())
            .arg("build")
            .arg("--bundle")
            .arg("--api")
            .arg("browser");
        if format == "cjs" {
            command.arg("--format").arg("cjs");
        }
        let output = command.arg(&source_path).output().expect("run kali");
        assert!(
            output.status.success(),
            "[{format}] stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let glue_file = if format == "cjs" { "app.cjs" } else { "app.js" };
        let js = fs::read_to_string(dir.path().join("app").join(glue_file)).expect("read glue");
        match format {
            "esm" => assert!(
                js.contains("export async function start()"),
                "[esm] glue: {js}"
            ),
            _ => {
                assert!(js.contains("async function start()"), "[cjs] glue: {js}");
                assert!(
                    js.contains("const exported = { load, loadWithImports, loadDynamicImport, start };"),
                    "[cjs] glue: {js}"
                );
            }
        }
        assert!(
            js.contains("instance.exports._start()"),
            "[{format}] glue must invoke the _start export: {js}"
        );
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --test browser_bundle_toplevel_start`
Expected: BOTH tests FAIL. `toplevel_program_runs_once_via_glue_start_helper` fails its `output.status.success()` assertion with stderr containing `mod.start is not a function`; `start_helper_is_present_in_both_glue_formats` fails the `export async function start()` assertion.

- [ ] **Step 3: Add `start()` to both glue template branches**

In `crates/kali_cli/src/bin/cmd_build.rs`, inside `generate_browser_bundle_js`:

3a. **ESM branch:** immediately after the `loadDynamicImport` block (currently lines ~1749-1751):

```js
export async function loadDynamicImport(specifier) {{
  return await import(resolveDynamicImportTarget(specifier).href);
}}
```

insert (still inside the same raw template string — note the doubled braces):

```js

// Run the program's top-level statements (the wasm `_start` export) exactly
// once; repeated calls await the same completion (or the same trap).
let startPromise = null;
export async function start() {{
  if (startPromise === null) {{
    startPromise = instancePromise.then((instance) => {{
      if (typeof instance.exports._start === 'function') {{
        instance.exports._start();
      }}
    }});
  }}
  return await startPromise;
}}
```

3b. **CJS branch:** immediately after the CJS `loadDynamicImport` block (currently lines ~1978-1980), insert the same helper WITHOUT the `export` keyword:

```js

// Run the program's top-level statements (the wasm `_start` export) exactly
// once; repeated calls await the same completion (or the same trap).
let startPromise = null;
async function start() {{
  if (startPromise === null) {{
    startPromise = instancePromise.then((instance) => {{
      if (typeof instance.exports._start === 'function') {{
        instance.exports._start();
      }}
    }});
  }}
  return await startPromise;
}}
```

and change the CJS exported line (currently `cmd_build.rs:1982`):

```js
const exported = {{ load, loadWithImports, loadDynamicImport }};
```

to:

```js
const exported = {{ load, loadWithImports, loadDynamicImport, start }};
```

Nothing else in either template changes.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_cli --test browser_bundle_toplevel_start`
Expected: PASS — 2/2. The execution test's stdout contains exactly one `3` line.

- [ ] **Step 5: Regression-run two existing bundle lanes**

Run: `cargo test -p kali_cli --test browser_number_predicates_bundle` and `cargo test -p kali_cli --test browser_bundle_cjs_source_classes`
Expected: PASS (8/8 each) — proves the glue additions are append-only and broke no existing consumer. (These build real bundles and, for the first, execute them under node.)

- [ ] **Step 6: Lint, format, commit**

```bash
cargo clippy -p kali_cli -p kali_runtime --tests -- -D warnings
cargo fmt
git add crates/kali_cli/src/bin/cmd_build.rs crates/kali_cli/tests/browser_bundle_toplevel_start.rs
git commit -m "feat(cli): export a memoized start() from browser bundle glue so top-level programs run"
```

Expected: clippy clean; fmt no diff.

---

### Task 2: `browser_bundle_harness_page` + `BROWSER_HARNESS_DONE_BINDING` in kali_runtime

**Files:**
- Modify: `crates/kali_runtime/src/browser/harness.rs` (new const + new function, after `browser_bundle_harness_script`)
- Modify: `crates/kali_runtime/src/browser/harness_tests.rs` (two new tests)
- Modify: `crates/kali_runtime/src/lib.rs` (extend the existing browser-harness re-export block at ~lines 35-40)

**Interfaces:**
- Consumes: nothing new (pure string generation, like its module neighbors).
- Produces (Task 3 relies on these): `pub const BROWSER_HARNESS_DONE_BINDING: &str = "__kaliHarnessDone";` and `pub fn browser_bundle_harness_page(bundle_dir: &str, body: &str) -> String`, both re-exported from the crate root (`kali_runtime::BROWSER_HARNESS_DONE_BINDING`, `kali_runtime::browser_bundle_harness_page`). Body contract: same as `browser_bundle_harness_script` — the script defines `bundleJs` and the body (conventionally `\n`-terminated) does its own `await import(bundleJs.href)`.

- [ ] **Step 1: Write the failing unit tests**

Add to `crates/kali_runtime/src/browser/harness_tests.rs`:

```rust
#[test]
fn browser_bundle_harness_page_is_browser_native() {
    let body = "const mod = await import(bundleJs.href);\nawait mod.start();\n";
    let page = browser_bundle_harness_page("app", body);
    assert!(page.starts_with("<!doctype html>"), "page: {page}");
    assert!(page.contains("<script type=\"module\">"), "page: {page}");
    assert!(
        page.contains("const bundleJs = new URL('./app/app.js', import.meta.url);"),
        "page: {page}"
    );
    assert!(page.contains(body), "page: {page}");
    assert!(
        page.contains("globalThis.__kaliHarnessDone('')"),
        "page must signal the completion binding with one string arg: {page}"
    );
    assert!(page.contains(BROWSER_HARNESS_DONE_BINDING), "page: {page}");
    assert!(
        !page.contains("node:"),
        "a browser-native page must not import node builtins: {page}"
    );
}

#[test]
fn browser_bundle_harness_page_shares_the_node_script_body_contract() {
    let body = "const mod = await import(bundleJs.href);\nawait mod.start();\n";
    let node_script = browser_bundle_harness_script("app", false, body);
    let page = browser_bundle_harness_page("app", body);
    for artifact in [node_script.as_str(), page.as_str()] {
        assert!(
            artifact.contains("const bundleJs = new URL('./app/app.js', import.meta.url);"),
            "artifact: {artifact}"
        );
        assert!(artifact.contains(body), "artifact: {artifact}");
    }
}
```

(Check the top of `harness_tests.rs` for its existing `use super::*;`-style import; if the new names are not covered by it, extend that import list.)

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_runtime harness`
Expected: FAIL to compile — ``cannot find function `browser_bundle_harness_page` `` / ``cannot find value `BROWSER_HARNESS_DONE_BINDING` ``. (Baseline for this lane before the change: 24 passed.)

- [ ] **Step 3: Implement the const and the page generator**

In `crates/kali_runtime/src/browser/harness.rs`, insert immediately after the `browser_bundle_harness_script` function (after line ~59):

```rust
/// The completion binding a browser harness page invokes once its body has
/// finished (successfully or not). A DevTools/CDP driver installs it via
/// `Runtime.addBinding`; Chromium requires binding functions to be called
/// with exactly one string argument, so pages pass `''`.
pub const BROWSER_HARNESS_DONE_BINDING: &str = "__kaliHarnessDone";

/// Build a browser-native harness page for an HTTP-served bundle. Unlike the
/// node-only prelude above, this emits no `node:` imports and installs no
/// fetch shim: the bundle glue's own `fetch(wasmUrl)` works once the bundle
/// directory is served over HTTP next to this page. The module script defines
/// `bundleJs` for the body — the same body contract as
/// [`browser_bundle_harness_script`] — reports body failures via
/// `console.error`, and always invokes [`BROWSER_HARNESS_DONE_BINDING`] when
/// a driver has installed it.
pub fn browser_bundle_harness_page(bundle_dir: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>Kali browser bundle harness</title>
<script type="module">
const bundleJs = new URL('./{bundle_dir}/{bundle_dir}.js', import.meta.url);
try {{
{body}}} catch (err) {{
  console.error('harness error: ' + (err && err.stack || err));
}}
if (globalThis.{binding}) {{ globalThis.{binding}(''); }}
</script>
"#,
        bundle_dir = bundle_dir,
        body = body,
        binding = BROWSER_HARNESS_DONE_BINDING
    )
}
```

In `crates/kali_runtime/src/lib.rs`, extend the existing browser-harness re-export block (~lines 35-40) so `browser_bundle_harness_page` and `BROWSER_HARNESS_DONE_BINDING` are exported alongside `browser_bundle_harness_script` (match the block's existing style — add the two names to the same `pub use` list(s)).

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p kali_runtime harness`
Expected: PASS — 26 passed (24 baseline + the 2 new).

- [ ] **Step 5: Lint, format, commit**

```bash
cargo clippy -p kali_cli -p kali_runtime --tests -- -D warnings
cargo fmt
git add crates/kali_runtime/src/browser/harness.rs crates/kali_runtime/src/browser/harness_tests.rs crates/kali_runtime/src/lib.rs
git commit -m "feat(runtime): browser-native bundle harness page generator + shared done-binding constant"
```

Expected: clippy clean; fmt no diff.

---

### Task 3: Rewire the gated CDP smoke test onto the production surfaces

**Files:**
- Modify: `crates/kali_cli/tests/cdp_driver/driver.rs` (`CDP_DONE_BINDING` becomes a re-export of the production constant)
- Modify: `crates/kali_cli/tests/browser_cdp_smoke.rs` (fixture + harness page + assertions + file-top doc comment)

**Interfaces:**
- Consumes: `start()` from Task 1, `browser_bundle_harness_page` + `BROWSER_HARNESS_DONE_BINDING` from Task 2, and the existing test-only CDP driver/HTTP-server mechanics (unchanged).
- Produces: the gated `real_chromium_runs_a_browser_bundle_and_captures_console` becomes the end-to-end proof, against real Chromium, that (a) a top-level program's console output routes through `console_log` via `start()`, (b) the per-export wrapper path still works, and (c) the production-generated page is genuinely browser-loadable.

- [ ] **Step 1: Single-source the completion-binding name in the driver**

In `crates/kali_cli/tests/cdp_driver/driver.rs`, replace:

```rust
/// The completion binding a harness page calls to signal it finished.
pub(crate) const CDP_DONE_BINDING: &str = "__kaliHarnessDone";
```

with:

```rust
/// The completion binding a harness page calls to signal it finished —
/// single-sourced from the production constant so driver and page generator
/// cannot drift.
pub(crate) use kali_runtime::BROWSER_HARNESS_DONE_BINDING as CDP_DONE_BINDING;
```

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — 10 passed / 5 ignored, unchanged (the value is identical; `route_event` and the unit tests keep compiling against the alias).

- [ ] **Step 2: Rewire the smoke test fixture and harness page**

In `crates/kali_cli/tests/browser_cdp_smoke.rs`:

2a. Replace the fixture source write (currently the `// kali-tree-shake: smoke` + `export async function smoke` block around lines 121-131, and its explanatory comment above) with:

```rust
    // 1. Build a browser bundle from a program with BOTH entry shapes: a bare
    //    top-level statement (runs via the glue's `start()` helper and must
    //    route console.log through the `console_log` import) and an exported
    //    function (runs via the per-export wrapper). The tree-shake marker is
    //    required for the export wrapper to be emitted.
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("app.ts");
    fs::write(
        &source,
        "// kali-tree-shake: smoke\n\
console.log(1 + 2);\n\
export async function smoke(left, right) {\n\
  console.log(6 + 1);\n\
  return left - left + right - right;\n\
}\n",
    )
    .expect("write source");
```

2b. Replace the hand-written HTML harness (currently the `let harness = "<!doctype html>..."` block and the comment above it, around lines 152-169) with the production generator:

```rust
    // 3. Generate the browser-native harness page from the production API:
    //    run the top-level program via start(), then the exported wrapper.
    let harness_path = dir.path().join("cdp-harness.html");
    let harness = kali_runtime::browser_bundle_harness_page(
        "app",
        "const mod = await import(bundleJs.href);\n\
await mod.start();\n\
await mod.smoke(1n, 2n);\n",
    );
    fs::write(&harness_path, harness).expect("write harness");
```

2c. Replace the final assertions (the `log_lines` filter + the two `assert!`s, currently ~lines 184-200) with exact-stdout assertions covering both paths in order:

```rust
    // 6. Assert the real browser produced BOTH programs' console output, in
    //    order: top-level `console.log(1 + 2)` via start(), then the export's
    //    `console.log(6 + 1)` via the wrapper.
    assert!(outcome.completed, "harness did not signal completion");
    assert_eq!(
        outcome.stdout(),
        "3\n7\n",
        "console: {:?}",
        outcome.console
    );
```

(Remove the now-unused `CdpConsoleLine` import if nothing else in the file uses it; keep `CdpPageOutcome` if still referenced.)

2d. Update the file-top module doc comment (lines 1-12): it currently explains that the in-test HTTP server exists because `browser_bundle_harness_script` is node-only. Keep the HTTP-server rationale (Chromium still blocks `fetch()` of `file://`, so serving stays necessary) but note the page itself now comes from the production `browser_bundle_harness_page` generator instead of a hand-written string.

- [ ] **Step 3: Run the unit lane**

Run: `cargo test -p kali_cli --test browser_cdp_smoke`
Expected: PASS — 10 passed / 5 ignored (compile-level proof; the rewired test is gated).

- [ ] **Step 4: Run the gated browser lane, twice**

Run: `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored` (twice)
Expected: PASS — 5/5 both times. The smoke test now proves in real Chromium: top-level output `3` (via `start()`), export output `7` (via wrapper), from a production-generated page. Also verify no leaked browsers: `pgrep -c chromium` → 0.

- [ ] **Step 5: Lint, format, commit**

```bash
cargo clippy -p kali_cli -p kali_runtime --tests -- -D warnings
cargo fmt
git add crates/kali_cli/tests/cdp_driver/driver.rs crates/kali_cli/tests/browser_cdp_smoke.rs
git commit -m "test(cli): drive the CDP smoke through start() and the production harness page"
```

Expected: clippy clean; fmt no diff.

---

### Task 4: Spec doc update + whole-plan verification

**Files:**
- Modify: `docs/superpowers/specs/2026-07-01-browser-harness-node-preference-design.md` (the "Implementation notes" paragraph, ~lines 97-105)

**Interfaces:** none — documentation only, plus the final gate.

- [ ] **Step 1: Update the Implementation notes paragraph**

In the spec, the current paragraph reads (verbatim):

```
Implementation notes: Chromium's `Runtime.addBinding` functions require exactly one
string argument, so harness pages call `globalThis.__kaliHarnessDone('')`. The
`browser_bundle_harness_script` prelude is node-only (it imports `node:fs/promises`),
so the smoke test serves the emitted bundle and an HTML module harness from an
in-test localhost HTTP server instead of reusing that helper. A bare top-level
`main()` call in a browser bundle does not route `console.log` through the
`console_log` import, so the fixture uses the repo's exported-function +
`// kali-tree-shake:` marker shape (possible production follow-up).
```

Replace it with:

```
Implementation notes: Chromium's `Runtime.addBinding` functions require exactly one
string argument, so harness pages call `globalThis.__kaliHarnessDone('')` — the
binding name is the shared `kali_runtime::BROWSER_HARNESS_DONE_BINDING` constant,
which the CDP driver re-uses. Both production follow-ups recorded here are closed
(2026-07-02): the bundle glue now exports a memoized `start()` helper that runs the
program's top-level statements (the wasm `_start` export) exactly once, so a bare
top-level program routes `console.log` through the `console_log` import in a
browser; and `kali_runtime::browser_bundle_harness_page` generates a browser-native
harness page (no `node:` imports — the glue's own `fetch` works over HTTP). The
smoke test still serves the bundle from an in-test localhost HTTP server (Chromium
blocks `fetch()` of `file://`), but its page now comes from the production
generator, and its fixture exercises both entry shapes: a bare top-level statement
via `start()` and an exported function via the per-export wrapper.
```

- [ ] **Step 2: Whole-plan verification**

Run every gate and confirm green:

```bash
cargo test -p kali_cli --test browser_bundle_toplevel_start          # 2/2
cargo test -p kali_runtime harness                                    # 26 passed
cargo test -p kali_cli --test browser_cdp_smoke                       # 10 passed / 5 ignored
cargo test -p kali_cli --test browser_cdp_smoke -- --ignored          # 5/5
cargo test -p kali_cli --test browser_number_predicates_bundle        # 8/8
cargo test -p kali_cli --test browser_bundle_cjs_source_classes       # 8/8
cargo clippy -p kali_cli -p kali_runtime --tests -- -D warnings       # clean
cargo fmt                                                             # no diff
```

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-07-01-browser-harness-node-preference-design.md
git commit -m "docs(spec): record closure of both browser-bundle production gaps (start() + harness page)"
```

---

## Verification (whole plan)

- All Task 4 Step 2 gates green; the gated lane run twice.
- `git status` clean; four commits, each formatted.
- Production diff limited to: two append-only glue template additions + one CJS `exported` line (`cmd_build.rs`), one new const + one new function + re-exports (`kali_runtime`).
