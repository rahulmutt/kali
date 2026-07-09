# fasta Spec 6 — Verbatim Vendoring + Large-N SHA-256 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Vendor the upstream `fasta-node-1` benchmark source *verbatim* (upstream operator forms) and validate it byte-for-byte against node at a small N and via SHA-256 at a large N, by adding the one missing compile primitive: compound-assign on a function parameter.

**Architecture:** Two tasks. Task 1 fixes the type-resolver so named function parameters are treated as mutable bindings (they already are in JS), which lets the fail-closed compound/update-assignment gate admit `n -= x` on a parameter and route it through the existing codegen local lane — no codegen change. Task 2 adds the verbatim fixture + metadata + sandbox policy and a two-tier acceptance test (small-N golden, large-N SHA-256).

**Tech Stack:** Rust workspace (`kali_types`, `kali_cli`/package `kali`), wasmtime-backed `kali run`, `sha2` crate (already a dev-dependency), node v26.4.0 as the reference oracle.

## Global Constraints

- The CLI package is named **`kali`** (its sources live under `crates/kali_cli/`). Integration tests run with `-p kali`.
- Verification gate (run before every commit that touches compiler crates): `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali` and `cargo fmt --check` — both clean. (Per the repo verification convention.)
- Fixtures live in `crates/kali_cli/tests/fixtures/benchmarks/`. Benchmark metadata JSON must have **exactly 5 keys**: `benchmark`, `version`, `sourceFile`, `sourceSha256`, `buildModes`, with `version: 1` and `buildModes` exactly `["--fast", "--release", "--release-advanced"]` (enforced by `schema_docs::benchmark_fixture_metadata_schema_tracks_current_fixture_contract`).
- `sourceSha256` is `"sha256-" + hex(sha256(source_file_bytes))` and is checked against the on-disk file at test time — it must match the fixture bytes exactly (trailing newline included).
- Controller discipline: re-run every reproducer on a **freshly-built** binary; trust observed behavior, not fix reports. Never trust a "fixed" claim without re-running.
- Task 1 is **additive** (a previously-rejected form now compiles). No existing fixture output may change: nbody, mandelbrot n=200, binary-trees N=21, spectral-norm, fannkuch stay byte-identical.
- Scope: **named** parameters only. Do not mark type parameters (`bind_type_params` → `bind_name_list`) mutable. Destructuring/default params are out of scope.

---

## Task 1: Compound-assign on a parameter binding

**Files:**
- Modify: `crates/kali_types/src/context.rs` — add `mark_binding_mutable` helper; mark params in `bind_function_params` (covers function expressions + arrow functions).
- Modify: `crates/kali_types/src/resolve/mod.rs:642` — mark function-declaration params mutable (fasta's path).
- Modify: `crates/kali_types/src/resolve/function.rs:65` — mark class-method params mutable.
- Test: `crates/kali_cli/tests/param_compound_assign.rs` (create).

**Interfaces:**
- Produces: `TypeContext::mark_binding_mutable(&mut self, scope_id: NodeId, name: &str)` — marks an already-bound name as mutable in `scope_id`; no-op if not bound there. Used by Task 1 only; Task 2 does not consume it.
- Behavioral contract after this task: `param op= rhs` and `param++`/`param--` on a **named** parameter compile and lower identically to the same operation on a `var` local of the same repr. A non-scalar (array/object) parameter compound-assign still rejects fail-closed.

### Background (why this is the only gap)

`Scope::bind` (`crates/kali_types/src/scope.rs:104`) inserts `mutable_bindings[name] = false` for every binding. `resolve_variable_declaration` upgrades `var`/`let` declarators to `true`, but the parameter-binding paths never do. So `binding_is_mutable` (`resolve/expression.rs:1842`) returns `false` for a parameter, and the fail-closed compound/update/nullish gate (`resolve/expression.rs:1683`, `:1786`) rejects `n -= lenOut` with `E5506`. Plain `=` on a parameter already works (different path; codegen already indexes params as locals). Marking params mutable drops them into the same lane a `var` local already uses safely.

Parameters reach the resolver through **three** value-param sites (type params, which also use `bind_name_list`, must stay untouched):
- Function **declarations**: `resolve/mod.rs:642` → `bind_name_list(params)` in `function_scope_id` — *fasta's path*.
- Function **expressions** + **arrow functions**: `function.rs:16` / `:26` → `bind_function_params(&expr.params)` in the current (pushed function) scope.
- Class **methods**: `function.rs:65` → `bind_name_list(&method.params)` in the current scope.

- [ ] **Step 1: Write the failing test file**

Create `crates/kali_cli/tests/param_compound_assign.rs`:

```rust
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Write `src` to a uniquely-named temp file and `kali run` it. The unique
/// slug (pid + atomic counter + src length) avoids the concurrent-fixture
/// collision flake documented for the mandelbrot fixture work.
fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-param-compound-{}-{}-{}",
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

// A compound `-=` targeting a PARAMETER (`n`), decremented in a loop — the
// exact shape fasta's `fastaRepeat`/`fastaRandom` use (`n -= lenOut`).
#[test]
fn param_compound_minus_equals_in_loop_runs() {
    let src = "function f(n){var t=0;while(n>0){t=t+1;n-=1;}return t;} console.log(f(4));";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n");
}

// `+=` on a parameter that is also read after (accumulate into the param).
#[test]
fn param_compound_plus_equals_runs() {
    let src = "function g(n){var i=0;while(i<3){n+=2;i=i+1;}return n;} console.log(g(10));";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "16\n");
}

// `n++` (update expression) on a parameter.
#[test]
fn param_update_increment_runs() {
    let src = "function h(n){var i=0;while(i<3){n++;i=i+1;}return n;} console.log(h(0));";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
}

// FAIL-CLOSED GUARD: compound-assign on a non-scalar (array) parameter must
// NOT miscompile — it must reject. Marking params mutable only removes the
// mutability barrier; the array repr still has no compound lowering, so this
// must still fail (never silently produce output).
#[test]
fn array_param_compound_still_rejects() {
    let src = "function g(a){a+=1;return a;} var xs=[1,2]; console.log(g(xs));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "array-param compound must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali --test param_compound_assign`
Expected: `param_compound_minus_equals_in_loop_runs`, `param_compound_plus_equals_runs`, and `param_update_increment_runs` FAIL (the binary exits non-zero with `error[E5506]: compound assignment lowering is unavailable for binding 'n' ...` / the update-expression variant). `array_param_compound_still_rejects` PASSES already (it rejects today for the mutability reason).

- [ ] **Step 3: Add the `mark_binding_mutable` helper and mark params in `bind_function_params`**

In `crates/kali_types/src/context.rs`, replace `bind_function_params` (currently at lines 326-330) with:

```rust
    pub(crate) fn bind_function_params(&mut self, params: &[FunctionParam]) {
        for param in params {
            self.bind_current_scope(param.name.clone());
        }
        // JS parameters are reassignable (mutable) — the same binding kind as a
        // `var`/`let` local. Mark them so `binding_is_mutable` reports true and
        // the fail-closed compound/update-assignment gate admits `n -= x` /
        // `n++` on a parameter, routing it through the same codegen local lane a
        // `var` local uses (fasta Spec 6 Task 1).
        if let Some(scope_id) = self.current_scope_id() {
            for param in params {
                self.mark_binding_mutable(scope_id, &param.name);
            }
        }
    }

    /// Mark an already-bound `name` as a mutable binding in `scope_id`. No-op
    /// if `name` is not bound in that scope. Used to flag function parameters
    /// mutable after they are bound (they are reassignable in JS).
    pub(crate) fn mark_binding_mutable(&mut self, scope_id: NodeId, name: &str) {
        if let Some(scope) = self.scope_mut(scope_id) {
            if scope.bindings.contains_key(name) {
                scope.mutable_bindings.insert(name.to_owned(), true);
            }
        }
    }
```

Note: `scope_mut(scope_id)` returns `Option<&mut Scope>` (it is used as `self.scope_mut(scope_id).expect("active scope exists")` at `context.rs:281`), so the `if let Some(scope)` above is correct as written.

- [ ] **Step 4: Mark function-declaration params mutable (fasta's path)**

In `crates/kali_types/src/resolve/mod.rs`, in the `Statement::FunctionDeclaration` arm, immediately after `self.bind_name_list(params);` (line 642), insert:

```rust
                // Function-declaration parameters are mutable JS bindings (see
                // context::bind_function_params) — mark them so a compound/
                // update assignment on a parameter (`n -= lenOut`) is admitted.
                for param in params {
                    self.mark_binding_mutable(function_scope_id, param);
                }
```

(`params` here is `&[String]`; `param: &String` coerces to `&str`.)

- [ ] **Step 5: Mark class-method params mutable**

In `crates/kali_types/src/resolve/function.rs`, in `resolve_class_body`, immediately after `self.bind_name_list(&method.params);` (line 65), insert:

```rust
            if let Some(scope_id) = self.current_scope_id() {
                for param in &method.params {
                    self.mark_binding_mutable(scope_id, param);
                }
            }
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p kali --test param_compound_assign`
Expected: all four tests PASS (`4\n`, `16\n`, `3\n`, and the array reject).

- [ ] **Step 7: Run the full verification gate (no regressions)**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali`
Expected: PASS. Pay attention to the existing CLBG runtime tests (nbody, mandelbrot, binary-trees, spectral-norm, fannkuch) and the for-in / for-in-key resolve tests — Task 1 must not change any of them.

Run: `cargo fmt --check`
Expected: clean (no diff). If it reports formatting, run `cargo fmt` and re-verify.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_types/src/context.rs \
        crates/kali_types/src/resolve/mod.rs \
        crates/kali_types/src/resolve/function.rs \
        crates/kali_cli/tests/param_compound_assign.rs
git commit -m "feat(types): compound/update assignment on a parameter binding (fasta Spec 6 Task 1)

Mark named function parameters as mutable bindings (they are reassignable in
JS), so the fail-closed compound/update-assignment gate admits \`n -= x\` / \`n++\`
on a parameter and routes it through the existing codegen local lane. Covers
function declarations, expressions, arrows, and methods; leaves type params
untouched. Array/object parameter compound-assign still rejects fail-closed."
```

---

## Task 2: Vendor verbatim fixture + two-tier SHA-256 validation

**Files:**
- Create: `crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.ts`
- Create: `crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.json`
- Create: `crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.policy.json`
- Modify: `crates/kali_cli/tests/schema_docs/misc.rs` — add `"fasta"` to `expected_benchmark_names` (line 2073) and `"fasta-benchmark-v1.ts"` to `expected_benchmark_sources` (line 2145).
- Test: `crates/kali_cli/tests/clbg_fasta_runtime.rs` (create).

**Interfaces:**
- Consumes: the parameter compound-assign primitive from Task 1 (the verbatim fixture uses `n -= lenOut` / `n -= line.length` and does not compile without it).
- Produces: nothing consumed downstream; this is the acceptance layer.

### Reference constants (verified against node v26.4.0, seed fixed at 42)

- Fixture source (exact bytes below, 1803 bytes incl. trailing newline): `sha256 = 66cea09e1e3e7ee23792dee729871713e6570d44fe046caf030bc4533c58b4ee`
- Tier-1 golden output at **N=8** (byte-for-byte):
  ```
  >ONE Homo sapiens alu
  GGCCGGGCGCGGTGGC
  >TWO IUB ambiguity codes
  cttBtatcatatgctaKggNcata
  >THREE Homo sapiens frequency
  aatagctaaatcttgtgcttcgttagaagtctcgactacg
  ```
- Tier-2 output SHA-256 at **N=2,000,000** (20 MB, ~1.5s, sits below the ~N≈4M leak wall): `a6b7308b4f7ea37cbaef69bdb05448c8623549978dc24d30e4e197026c1e073a`
- (For the future Spec 7 canonical pin, not used here: N=25,000,000 output SHA-256 = `6a26f1c843bebd234692ff1bd98ad517dd7df732fe93d2095845a2ddafc9ecee`.)

- [ ] **Step 1: Create the verbatim fixture source**

Create `crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.ts` with **exactly** this content (upstream `fasta-node-1` operator forms: `+=`/`-=`/`i++`, multi-declarator `var`, braceless `for..in`; a single trailing newline at end of file):

```js
var last = 42;
function rand(max) { last = (last * 3877 + 29573) % 139968; return max * last / 139968; }
function makeCumulative(table) {
  var prev = null;
  for (var c in table) {
    if (prev) table[c] += table[prev];
    prev = c;
  }
}
function fastaRepeat(n, seq) {
  var seqi = 0, lenOut = 60;
  while (n > 0) {
    if (n < lenOut) lenOut = n;
    if (seqi + lenOut < seq.length) {
      console.log(seq.substring(seqi, seqi + lenOut));
      seqi += lenOut;
    } else {
      console.log(seq.substring(seqi) + seq.substring(0, lenOut - (seq.length - seqi)));
      seqi = lenOut - (seq.length - seqi);
    }
    n -= lenOut;
  }
}
function fastaRandom(n, table) {
  var line = new Array(60);
  makeCumulative(table);
  while (n > 0) {
    if (n < line.length) line = new Array(n);
    for (var i = 0; i < line.length; i++) {
      var r = rand(1);
      for (var c in table) if (r < table[c]) break;
      line[i] = c;
    }
    console.log(line.join(""));
    n -= line.length;
  }
}
var ALU = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG" +
"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA" +
"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT" +
"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA" +
"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG" +
"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC" +
"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAAA";
var IUB = { a: 0.27, c: 0.12, g: 0.12, t: 0.27, B: 0.02, D: 0.02, H: 0.02, K: 0.02, M: 0.02, N: 0.02, R: 0.02, S: 0.02, V: 0.02, W: 0.02, Y: 0.02 };
var HomoSap = { a: 0.3029549426680, c: 0.1979883004921, g: 0.1975473066391, t: 0.3015094502008 };
var n = +process.argv[2];
console.log(">ONE Homo sapiens alu");
fastaRepeat(2 * n, ALU);
console.log(">TWO IUB ambiguity codes");
fastaRandom(3 * n, IUB);
console.log(">THREE Homo sapiens frequency");
fastaRandom(5 * n, HomoSap);
```

- [ ] **Step 2: Verify the fixture's source hash matches the reference constant**

Run: `sha256sum crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.ts`
Expected: `66cea09e1e3e7ee23792dee729871713e6570d44fe046caf030bc4533c58b4ee  ...`

If it differs, the file bytes are off (usually a missing/extra trailing newline or a smart-quote). Fix the file until the hash matches — the metadata JSON in Step 3 depends on it. (Alternatively, if you intentionally alter whitespace, recompute and use the new hash consistently in Step 3.)

- [ ] **Step 3: Create the fixture metadata JSON**

Create `crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.json` (exactly 5 keys; `sourceSha256` = `sha256-` + the hash from Step 2):

```json
{
  "benchmark": "fasta",
  "version": 1,
  "sourceFile": "fasta-benchmark-v1.ts",
  "sourceSha256": "sha256-66cea09e1e3e7ee23792dee729871713e6570d44fe046caf030bc4533c58b4ee",
  "buildModes": ["--fast", "--release", "--release-advanced"]
}
```

- [ ] **Step 4: Create the sandbox policy (fuel raised above the default runaway guard)**

Create `crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.policy.json` (memory uncapped so it does not gate the interim below-the-wall run; all effects denied except `console`; generous fuel, matching the binary-trees precedent):

```json
{
  "schemaVersion": 1,
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": null },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": null, "maxActiveTimers": null },
    "eval": false,
    "random": false,
    "console": true
  },
  "resources": {
    "maxMemoryMB": null,
    "maxCpuTimeMs": 64000000,
    "maxOpenFiles": null,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}
```

- [ ] **Step 5: Write the two-tier acceptance test**

Create `crates/kali_cli/tests/clbg_fasta_runtime.rs`:

```rust
use sha2::{Digest, Sha256};
use std::{path::PathBuf, process::Command};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

// Tier 1 — small-N golden. The verbatim upstream fasta-node-1 source (with
// `+=`/`-=`/`i++`), read from the checked-in fixture, run under `--api node`
// with N=8, must match node v26.4.0 byte-for-byte (seed fixed at 42).
#[test]
fn fasta_small_n_matches_node_golden() {
    const GOLDEN: &str = ">ONE Homo sapiens alu\nGGCCGGGCGCGGTGGC\n>TWO IUB ambiguity codes\ncttBtatcatatgctaKggNcata\n>THREE Homo sapiens frequency\naatagctaaatcttgtgcttcgttagaagtctcgactacg\n";
    let source = fixture("fasta-benchmark-v1.ts");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg(&source)
        .arg("--")
        .arg("8")
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), GOLDEN);
}

// Tier 2 — large-N SHA-256. N=2,000,000 (20 MB output, ~1.5s) sits with ~40%
// byte-headroom below the measured ~N>=4M allocation wall (E4000): the fasta
// output loops leak their per-line join/substring temporaries — there is NO
// per-line reclamation yet. Canonical N=25,000,000 (254 MB) awaits fasta
// Spec 7's arena reclamation; its node reference hash is
// 6a26f1c843bebd234692ff1bd98ad517dd7df732fe93d2095845a2ddafc9ecee. This
// interim tier proves the golden-free SHA validation harness against the
// N=2M node reference.
#[test]
fn fasta_large_n_matches_node_sha256() {
    const N: &str = "2000000";
    const NODE_SHA256: &str =
        "a6b7308b4f7ea37cbaef69bdb05448c8623549978dc24d30e4e197026c1e073a";
    let source = fixture("fasta-benchmark-v1.ts");
    let policy = fixture("fasta-benchmark-v1.policy.json");
    let output = Command::new(kali_bin())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .arg("--")
        .arg(N)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let digest = format!("{:x}", Sha256::digest(&output.stdout));
    assert_eq!(
        digest, NODE_SHA256,
        "fasta N={} output SHA-256 differs from the node v26.4.0 reference",
        N
    );
}
```

- [ ] **Step 6: Register the fixture in the benchmark-corpus allowlists**

The schema test asserts the fixture directory's benchmark set exactly equals two hardcoded allowlists. In `crates/kali_cli/tests/schema_docs/misc.rs`:

1. In `expected_benchmark_names` (the `BTreeSet<String>` array literal starting at line 2073), add a `"fasta",` entry alongside the other slugs (e.g. next to `"mandelbrot"`).
2. In `expected_benchmark_sources` (the array literal starting at line 2145), add a `"fasta-benchmark-v1.ts",` entry alongside the other `*-benchmark-v1.ts` source names.

- [ ] **Step 7: Run the fasta runtime test**

Run: `cargo test -p kali --test clbg_fasta_runtime`
Expected: both `fasta_small_n_matches_node_golden` and `fasta_large_n_matches_node_sha256` PASS. (Requires Task 1 to be present, or the fixture fails to compile with `E5506` on `n -= …`.)

- [ ] **Step 8: Run the schema/metadata test**

Run: `cargo test -p kali --test schema_docs benchmark_fixture_metadata_schema_tracks_current_fixture_contract`
Expected: PASS. If it fails with "benchmark slugs should match" or "source files should match", the allowlist edits in Step 6 are missing or misspelled. If it fails on `sourceSha256`, the JSON hash and the fixture bytes disagree (re-check Step 2).

- [ ] **Step 9: Run the full verification gate**

Run: `cargo test -p kali_lexer -p kali_common -p kali_types -p kali_codegen -p kali`
Expected: PASS (all crates, including the new fasta tests and every existing fixture unchanged).

Run: `cargo fmt --check`
Expected: clean.

- [ ] **Step 10: Commit**

```bash
git add crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.ts \
        crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.json \
        crates/kali_cli/tests/fixtures/benchmarks/fasta-benchmark-v1.policy.json \
        crates/kali_cli/tests/clbg_fasta_runtime.rs \
        crates/kali_cli/tests/schema_docs/misc.rs
git commit -m "test(cli): vendor fasta-node-1 verbatim + two-tier SHA-256 validation (fasta Spec 6 Task 2)

Checked-in verbatim upstream fixture (upstream +=/-=/i++ operator forms) plus a
small-N byte-for-byte golden and an N=2,000,000 SHA-256 tier vs the node
v26.4.0 reference. N=2M sits below the current per-line allocation wall;
canonical N=25M + reclamation is fasta Spec 7."
```

---

## Self-Review

**1. Spec coverage.**
- Spec Task 1 (parameter compound-assign) → Plan Task 1. ✓
- Spec "verbatim vendoring" (upstream operator forms) → Plan Task 2 Step 1 fixture + hash check. ✓
- Spec fail-open safety / reject-safety matrix → Plan Task 1 `array_param_compound_still_rejects` + the full-gate regression check (Step 7) covering the string-param and for-in cases via existing tests. Note: the plan pins the *array* reject explicitly and relies on the gate + fixture byte-match for the string-param and for-in claims (a string-param `+=` may legitimately concatenate rather than reject, so it is not asserted as a hard reject to avoid over-specifying unverified behavior — the gate proves no regression).
- Spec Task 2 two-tier SHA-256 (small-N golden + large-N hash) → Plan Task 2 Steps 5-8. ✓
- Spec fixture convention (metadata JSON + policy + schema allowlist) → Plan Task 2 Steps 3-4, 6. ✓
- Spec scope boundary (N=25M + reclamation deferred to Spec 7) → documented in the Tier-2 test comment and Step 5; not implemented here. ✓

**2. Placeholder scan.** No `TBD`/`TODO`/"handle edge cases"/"similar to". Every code step shows complete code; every run step shows the exact command and expected result. ✓

**3. Type consistency.** `mark_binding_mutable(&mut self, scope_id: NodeId, name: &str)` is defined in Task 1 Step 3 and called with the same signature in Steps 3-5 (`scope_id: NodeId`, `name: &str` via `&String`/`&param.name` coercion). Test helper `run_source` / `kali_bin` / `fixture` names are consistent within each test file. The Tier-2 constant `NODE_SHA256` and the Step-2 `sourceSha256` are distinct hashes (output vs source) and are not conflated. ✓

**Verification note resolved:** `scope_mut` returns `Option<&mut Scope>` (confirmed at `context.rs:281`), so `mark_binding_mutable` uses `if let Some(scope) = self.scope_mut(scope_id)` as written.
