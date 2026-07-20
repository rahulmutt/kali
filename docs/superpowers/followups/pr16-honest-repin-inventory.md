# PR #16 honest re-pin inventory — REVISION 2 (Strategy B, N=708)

Supersedes the rev1 inventory (694-test, name-classified, voided by the silent-miscompile
register). Evidence re-gathered on HEAD `006dc55fc` (docs-only ahead of `b379f7ee6`) against
`target/debug/kali` built from that tree (Group-1 evidence-corrupting fixes all landed),
differentially vs Node 26.5.0. Baseline: `pr16-honest-red-baseline.txt` (N=708,
double-enumerated zero-drift, run-1 == run-3).

Method: one full `cargo test --workspace --no-fail-fast` capture (NO `--nocapture`) parsed
into per-test `---- <name> stdout ----` blocks; each block classified by observed payload
(not by name). A = fails closed / loud (E-code, runtime trap, or the fixture's own
self-check throwing — nonzero exit, never a silent wrong value). B = silent wrong value at
exit 0. 3 `browser_runtime` corpus blocks whose evidence sat inside a JSON `"stdout"` field
were hand-audited → all B (`"0 0\n0 0\n"` wrong output at `exitCode:0`).

## 1. Headline split

| class | count | share | action |
|-------|-------|-------|--------|
| **A** — fails closed / loud | **545** | 77% | **re-pin** to assert the diagnostic (`E5506` / `!success()` / observed `Uncaught` text) |
| **B** — silent wrong value | **163** | 23% | **`#[ignore]`+issue** (see §3 — no cheap Group-2 deny-lane family is present in this red set) |
| total | **708** | | |

Class-A diagnostic texture (evidence counts across 726 blocks): 192 `error[E5506]`
(compile-refuse), 110 `RuntimeError: unreachable` + 54 `error[E4000]` (runtime traps),
283 `Uncaught Error` + 20 `Uncaught exception` (the fixture's OWN assertion throwing on
wrong async/boolean behavior — a loud failure, not a silent wrong value). All three flavors
re-pin honestly; none risks blessing a falsehood.

## 2. Adjudication table (family × class)

| family | total | A (re-pin) | B (ignore+issue) | B nature | action |
|--------|-------|-----------|------------------|----------|--------|
| **object-enum** | 417 | 293 | 124 | no object-enumeration materialization model; rev1 deny lane was landed then **reverted** (`139d71189`) → architectural | A: re-pin · B: ignore+issue |
| **promise** | 128 | 128 | 0 | async combinators fail loud (trap / `Uncaught`) | re-pin only |
| **string-iter** | 56 | 48 | 8 | string→array iteration materialization (object-enum-adjacent choke) | A: re-pin · B: ignore+issue |
| **mapset** | 31 | 23 | 8 | `Set`/`Map` constructor iteration + `Array.from` (object-enum-adjacent) | A: re-pin · B: ignore+issue |
| **corpus** | 29 | 10 | 19 | real npm packages; heterogeneous silent miscompiles (`"0 0"` output) — cannot deny-lane a whole package | A: re-pin · B: ignore+issue |
| **microtask** | 22 | 20 | 2 | Stage-D `queueMicrotask` ordering; mostly fails loud | A: re-pin · B: ignore+issue |
| **other** | 10 | 10 | 0 | singletons, all fail loud | re-pin only |
| **deno** | 4 | 4 | 0 | `Deno.env`/chdir fail loud | re-pin only |
| **crypto** | 4 | 2 | 2 | bundle WebCrypto/`TextEncoder`; 2 loud, 2 silent | A: re-pin · B: ignore+issue |
| **bool-logic** | 4 | 4 | 0 | bundle boolean lowering; fails loud (`Uncaught`) | re-pin only |
| **await** | 3 | 3 | 0 | await-wrapped enumeration; fails loud | re-pin only |
| **TOTAL** | **708** | **545** | **163** | | |

Coverage ledger: 545 + 163 = 708 = baseline N. No silent remainder. (Family tokens are a
coarse grouping for wave batching; the class call per test is from its observed payload, not
its name. The object-enum/string-iter/mapset families each contain BOTH A and B members —
consistent with the register's warning that the A/B boundary is a property of the exact
syntactic shape, not the feature.)

## 3. The "deny-lane where cheap" decision (ratified 2026-07-20)

The rev2 design admitted a cheap-class-B deny lane for register **Group-2** defects (R-11
bitwise compound-assign, R-24 `Object.freeze`, R-09 `continue`, R-16 string methods, R-26
unary `+`, R-27 comma, R-22 `==`). **Measurement shows none of those defects have a test in
this 708 red set** — every class-B family here is the architectural object-enum / async /
corpus cluster (register Group 3/4 and the reverted object-enum lane). Therefore
**all 163 class-B tests take the `#[ignore]`+issue action**; the deny-lane branch does not
fire in this effort. (Cheap Group-2 fixes remain post-merge follow-ups on their own
branches, unrelated to unblocking PR #16.)

## 4. Class-B families → one tracking issue each (register mapping)

| family | B count | register defect | flip-back condition | issue |
|--------|---------|-----------------|---------------------|-------|
| object-enum | 124 | object enumeration materialization (no object model); register Group 4 | real Object.keys/values/entries/for-of enumeration lands | _(filed in Task 4)_ |
| corpus | 19 | heterogeneous — package hits an object-enum/other silent miscompile | corpus re-greens as underlying defects land | _(filed in Task 4)_ |
| string-iter | 8 | string→array iteration materialization | string-iteration lowering lands | _(filed in Task 4)_ |
| mapset | 8 | `Set`/`Map` constructor iteration + `Array.from` | iterable materialization lands | _(filed in Task 4)_ |
| crypto | 2 | bundle WebCrypto/`TextEncoder` zero-placeholder | P5 (`TextEncoder`) / WebCrypto lane | _(filed in Task 4)_ |
| microtask | 2 | Stage-D `queueMicrotask` ordering silent case | microtask ordering fix | _(filed in Task 4)_ |

## 5. Proposed wave order (largest family first)

1. object-enum (417: 293 re-pin + 124 ignore) — spans `runtime_smoke/{run,test,build}.rs`,
   `package_corpus/*`, browser suites; the mega-wave, split by binary if needed.
2. promise (128 re-pin) · 3. string-iter (56) · 4. mapset (31) · 5. corpus (29) ·
   6. microtask (22) · 7. other/deno/crypto/bool-logic/await (25).

Each wave gates on full re-enumeration vs the frozen 708 baseline: newly-red empty,
drain monotone.

## 6. Class-A re-pin idiom by diagnostic

- `error[E5506]` present → `assert!(stderr.contains("E5506"), "stderr: {stderr}");`
- runtime trap / `Uncaught` (no stable E-code) → `assert!(!output.status.success(), "must fail closed: {output:?}");`
  optionally tightened with the observed `Uncaught`/message substring where stable.

## 7. Raw artifacts (session scratchpad, not committed)

- `pr16-blocks.txt` — the full no-`--nocapture` capture (726 failure blocks).
- `pr16-triage.tsv` — per-block `name<TAB>class<TAB>evidence`.
- `pr16-name-class.tsv` — resolved per-name class (708 rows, `?`→B).
- `pr16-fam-class.tsv` — family × class cross-tab (source of §2).
