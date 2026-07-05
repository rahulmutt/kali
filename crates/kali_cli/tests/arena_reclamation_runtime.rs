//! Task 6 (loop arenas): reclamation-proof and behavioral-matrix tests. These
//! are the FIRST tests to genuinely exercise `__arena_reset` and page
//! recycling at runtime — Task 5 built the synthetics but never drove them
//! (`g7`, the free-list head, was provably always 0 before this task).
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Build a process-wide-unique directory name for a test fixture. Uniqueness
/// must NOT depend on wall-clock resolution: tests run multi-threaded, and on
/// platforms with a coarse `SystemTime` clock (e.g. macOS) two concurrent
/// calls can observe the same `as_nanos()` value and collide on the same temp
/// dir. A process-wide monotonic counter guarantees uniqueness independently
/// of the wall-clock's resolution.
fn unique_fixture_slug(label: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    format!(
        "kali-arena-reclamation-{label}-{unique}-{}-{seq}",
        std::process::id()
    )
}

fn write_temp_source(label: &str, source: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(unique_fixture_slug(label));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    fs::write(&path, source).expect("write source fixture");
    path
}

fn write_temp_policy_json(label: &str, policy: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(unique_fixture_slug(label));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("policy.json");
    fs::write(&path, policy).expect("write policy fixture");
    path
}

/// `maxMemoryMB: 8`, `maxCpuTimeMs: 600000` (raised so a genuinely-reclaiming
/// run's CPU cost never gets mistaken for the fuel trap E4003 — see Task 1).
const MEM8_POLICY_JSON: &str = r#"{
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
    "maxMemoryMB": 8,
    "maxCpuTimeMs": 600000,
    "maxOpenFiles": null,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}"#;

#[test]
fn per_iteration_loop_allocations_are_reclaimed() {
    // 400 iterations x ~256KB of fresh objects = ~100MB cumulative, under an
    // 8MB memory cap: passes only if iteration arenas recycle pages.
    let source = write_temp_source(
        "reclaim_loop",
        r#"function mkRow() {
  return { a: 1, b: 2, c: 3, d: 4 };
}
function main() {
  let sum = 0;
  for (let round = 0; round < 400; round = round + 1) {
    let last = 0;
    for (let i = 0; i < 8000; i = i + 1) {
      const row = mkRow();
      last = row.a + row.d;
    }
    sum = sum + last;
  }
  console.log(sum);
}
main();
"#,
    );
    let policy = write_temp_policy_json("mem8", MEM8_POLICY_JSON);
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2000\n"); // 400 * (1+4)
}

#[test]
fn mini_binary_trees_output_exact() {
    // The CLBG binary-trees shape at n=8 (minDepth=4, maxDepth=8,
    // stretchDepth=9), computed with object-literal trees (kali's supported
    // heap-object surface) instead of a `TreeNode` constructor: a stretch
    // tree built and checked before the main loop, a long-lived tree built
    // before the loop and checked AFTER it (both must survive untouched by
    // any loop-arena reset), and a nested for/for nest of per-iteration
    // trees in between (the arena'd portion).
    let source = write_temp_source(
        "mini_binary_trees",
        r#"function bottomUpTree(depth) {
  if (depth > 0) {
    return { left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) };
  }
  return { left: null, right: null };
}
function itemCheck(t) {
  if (t.left === null) {
    return 1;
  }
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}
function main() {
  const minDepth = 4;
  const maxDepth = 8;
  const stretchDepth = maxDepth + 1;
  let total = itemCheck(bottomUpTree(stretchDepth));
  const longLivedTree = bottomUpTree(maxDepth);
  for (let depth = minDepth; depth <= maxDepth; depth = depth + 2) {
    const iterations = 1 << (maxDepth - depth + minDepth);
    let check = 0;
    for (let i = 1; i <= iterations; i = i + 1) {
      check = check + itemCheck(bottomUpTree(depth));
    }
    total = total + check;
  }
  total = total + itemCheck(longLivedTree);
  console.log(total);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // stretch(9) = 1023; depth-loop checks: 256*31 + 64*127 + 16*511 = 24240;
    // longLived(8) = 511. total = 1023 + 24240 + 511 = 25774.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "25774\n");
}

#[test]
fn break_inside_arena_loop_is_sound() {
    // Loop allocs per iteration, breaks at i==50: proves the break-path
    // arena release restores state correctly (a fresh object built AFTER the
    // loop must not observe any corruption from the loop's own arena).
    let source = write_temp_source(
        "break_sound",
        r#"function mk(v) {
  return { a: v, b: v + 1 };
}
function main() {
  let sum = 0;
  for (let i = 0; i < 1000; i = i + 1) {
    const obj = mk(i);
    sum = sum + obj.a;
    if (i === 50) {
      break;
    }
  }
  const after = mk(999);
  console.log(sum + after.a + after.b);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // sum = 0+1+...+50 = 1275; after = mk(999) => a=999, b=1000.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "3274\n");
}

#[test]
fn return_from_arena_loop_is_sound() {
    // A function returns a SCALAR from inside an arena'd loop; the caller
    // continues allocating correctly afterward — proves `emit_return`'s
    // inline arena unwind (Step 4) releases every live frame before the
    // early `Instruction::Return`, not just the loop's own normal-exit path.
    let source = write_temp_source(
        "return_sound",
        r#"function mk(v) {
  return { a: v, b: v * 2 };
}
function findFirstOver(limit) {
  for (let i = 0; i < 1000; i = i + 1) {
    const obj = mk(i);
    if (obj.a + obj.b > limit) {
      return i;
    }
  }
  return -1;
}
function main() {
  const found = findFirstOver(100);
  const after = mk(found);
  console.log(found + after.a + after.b);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // 3*i > 100 first at i=34 (3*34=102); after = mk(34) => a=34, b=68.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "136\n");
}

#[test]
fn module_global_store_fails_closed() {
    // A function called inside a loop stores a fresh object into a
    // persistent, long-lived container (an array allocated BEFORE the loop)
    // reachable only by reference through a call argument — the
    // interprocedural sibling of `store_to_outer_fails_closed` below (the
    // escaping store happens inside the CALLEE, not inline in the loop's own
    // function). The gate must veto the loop's arena; the container's
    // contents must be correct after the loop.
    let source = write_temp_source(
        "module_global_store",
        r#"function mk(v) {
  return { a: v, b: v + 1 };
}
function store(arr, v) {
  arr[0] = mk(v);
}
function main() {
  const arr = [mk(0)];
  for (let i = 1; i <= 100; i = i + 1) {
    store(arr, i);
  }
  console.log(arr[0].a + arr[0].b);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "201\n"); // mk(100): 100+101
}

#[test]
fn store_to_outer_fails_closed() {
    // The fresh object escapes the iteration into an outer-declared binding:
    // the gate must veto this loop's arena; the value must survive the loop.
    // NOTE: this test's own veto genuinely fires, but its final `mk(100)`
    // would also survive under a WRONG grant (a top-of-iteration reset only
    // recycles the *previous* iteration's pages, so the *last* iteration's
    // allocation is never actually clobbered) — it does not by itself prove
    // reset-on-escape. `module_global_store_fails_closed` above is the real
    // mis-grant detector: it reads the escaped value from a container built
    // BEFORE the loop, so a wrong grant recycles it out from under a still-
    // live read.
    let source = write_temp_source(
        "store_outer",
        r#"function mk(v) {
  return { a: v, b: v + 1 };
}
function main() {
  let last = mk(0);
  for (let i = 1; i <= 100; i = i + 1) {
    last = mk(i);
  }
  console.log(last.a + last.b);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "201\n"); // 100 + 101
}

#[test]
fn nested_arena_loops_correct() {
    let source = write_temp_source(
        "nested_loops",
        r#"function mk(v) {
  return { a: v, b: 2 * v };
}
function main() {
  let total = 0;
  for (let outer = 0; outer < 50; outer = outer + 1) {
    let rowSum = 0;
    for (let inner = 0; inner < 200; inner = inner + 1) {
      const cell = mk(inner);
      rowSum = rowSum + cell.a + cell.b;
    }
    total = total + rowSum;
  }
  console.log(total);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // rowSum = 3 * (0..199 sum) = 3 * 19900 = 59700; total = 50 * 59700
    assert_eq!(String::from_utf8_lossy(&output.stdout), "2985000\n");
}

#[test]
fn nested_arena_loops_with_inner_break_is_sound() {
    // Regression pin for the break double-release bug (whole-branch review
    // finding 1): an unlabeled `break` used to emit its OWN inline arena
    // release in `emit_break_or_continue`, and that release's `Br` lands
    // exactly where `emit_loop` already emits its own unconditional
    // normal-exit release — so the same `ArenaFrame` was released twice.
    //
    // The bug is invisible with a single arena'd loop (`break_inside_arena_loop_is_sound`
    // above): with nothing enclosing it, the frame's saved page/cursor/limit
    // are all zero, so a second `__arena_reset` walking an empty page list is
    // a harmless no-op. It only corrupts state when the loop that breaks is
    // NESTED inside an outer loop that has ALREADY allocated (a non-zero
    // saved page) at the moment the inner loop opens: the inner frame's
    // saved trio is then the OUTER arena's own live page/cursor/limit. The
    // (buggy) inline release correctly recycles the inner loop's own pages
    // and restores the trio to the outer arena's values — then falling
    // through into `emit_loop`'s unconditional release runs `__arena_reset`
    // A SECOND time, this time against the now-current (just-restored)
    // OUTER arena, splicing the outer loop's still-live pages onto the free
    // list out from under it: a corrupted free list / use-after-free, not
    // merely "the inner loop's own pages released twice".
    //
    // `outerObj` is allocated directly in the OUTER loop's body, OUTSIDE the
    // inner loop, every outer iteration, so the outer arena's saved trio is
    // always non-zero by the time the inner loop opens. The inner loop
    // allocates its own `cell` objects and unconditionally breaks partway
    // through, then `outerObj` is read back immediately after the inner loop
    // closes, every outer iteration (20 of them, to give the corrupted free
    // list many chances to alias a still-live page into a later allocation
    // and clobber `outerObj`'s fields before they're read).
    let source = write_temp_source(
        "nested_inner_break",
        r#"function mk(v) {
  return { a: v, b: v + 1 };
}
function main() {
  let total = 0;
  for (let outer = 0; outer < 20; outer = outer + 1) {
    const outerObj = mk(outer);
    for (let inner = 0; inner < 30; inner = inner + 1) {
      const cell = mk(inner);
      if (inner === 10) {
        break;
      }
    }
    total = total + outerObj.a + outerObj.b;
  }
  console.log(total);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // outerObj = mk(outer) => a=outer, b=outer+1, so each iteration
    // contributes 2*outer + 1; total = sum_{outer=0}^{19} (2*outer + 1)
    // = 2*(0+1+...+19) + 20 = 2*190 + 20 = 400.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "400\n");
}

#[test]
fn spans_inside_arena_loop() {
    // Per-iteration `new Array(20000)` (~160KB, past the 65528-byte
    // single-page PAYLOAD) x 200 iterations under an 8MB memory cap: passes
    // only if multi-page SPANS — not just single pages — return to the free
    // list on reset and get reused by the next iteration's span request.
    //
    // `marker`'s object literal exists solely so `mkSpan` has a
    // `kali_mir`-recognized allocation site: a bare `new Array(n)` call is
    // (by design, see `loop_preorder_ordinals`'s doc comment in
    // `kali_codegen::lower`) invisible to the escape gate's `allocates` bit
    // on its own, since it lowers to a generic HIR `NewExpr` wrapping a
    // `CallExpr` rather than an `ArrayExpr` literal — without `marker`,
    // `mkSpan` would never become arena-eligible and this test would
    // (correctly, fail-closed) never reclaim, OOMing under the memory cap
    // regardless of Task 6's hooks. `t = tag + 0` (rather than storing `tag`
    // bare) keeps the array-element stores classified as definitely-scalar:
    // an arithmetic `BinaryExpr` is unconditionally `ValueClass::Scalar` in
    // `kali_mir`'s coarse v1 classifier, whereas a bare parameter identifier
    // is `DependsOn` an interprocedural node that, stored through a
    // member/array write, conservatively fails closed as may-heap (as it
    // should — the parameter's true type isn't known without full type
    // inference) and would veto `mkSpan`'s eligibility.
    let source = write_temp_source(
        "spans_arena_loop",
        r#"function mkSpan(tag) {
  const marker = { tag: tag };
  const arr = new Array(20000);
  const t = tag + 0;
  arr[0] = t;
  arr[19999] = t * 2;
  return arr[0] + arr[19999];
}
function main() {
  let sum = 0;
  for (let i = 0; i < 200; i = i + 1) {
    sum = sum + mkSpan(i);
  }
  console.log(sum);
}
main();
"#,
    );
    let policy = write_temp_policy_json("mem8_spans", MEM8_POLICY_JSON);
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // per-iteration value = t + 2t + marker.a - marker.a = 3*i; sum = 3 * (0..199 sum) = 59700.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "59700\n");
}

// --- Task 7: function-body arenas -----------------------------------------

#[test]
fn function_scratch_is_reclaimed() {
    // `scratchSum` builds a single ~64KB `new Array` of scratch, fills and
    // sums it, and returns a scalar — the array dies entirely inside the
    // call (never returned, never stored anywhere reachable after the call
    // returns). Crucially there is NO loop *inside* `scratchSum` that
    // allocates: the two `for` loops here only read/write the ALREADY-
    // allocated `buf`'s existing elements, so Task 6's already-shipped
    // per-loop arenas have nothing of their own to reclaim in this function
    // — the only mechanism that CAN recycle this ~64KB per call is a Task-7
    // per-call FUNCTION arena. `marker` (declared, never read again) exists
    // solely so `scratchSum` has a `kali_mir`-recognized object-literal
    // allocation site: exactly like `spans_inside_arena_loop`'s `mkSpan`
    // above, a bare `new Array(n)` is invisible to the escape gate's
    // `allocates` bit on its own, so without `marker` `scratchSum` would
    // never become arena-eligible at all and this test would (fail-closed,
    // correctly) never reclaim regardless of Task 7. (An earlier draft
    // folded `marker.tag - marker.tag` into the return value to net zero —
    // disassembly showed an earlier compiler stage constant-folding that
    // whole expression to a literal `0` and dropping `marker`'s allocation
    // entirely, silently undoing the arena-eligibility trick. Leaving
    // `marker` truly unread, exactly like `mkSpan`'s own precedent, avoids
    // that.)
    //
    // `scratchSum` is called 300 times from a loop in `main`. That loop's OWN
    // arena is explicitly vetoed by `taint`: a factory whose returned OBJECT
    // is stored into `taints`, an array allocated BEFORE the loop — the same
    // store-to-outer-container pattern `module_global_store_fails_closed`
    // uses to force a veto (confirmed by disassembly during development: with
    // `taint`'s escaping store present, `main`'s loop emits no
    // save/zero-trio prologue and no `__arena_reset` call at all around the
    // `call` loop). This is NOT optional set dressing — it is the whole
    // point of this test: a plain `for` loop calling a scalar-returning
    // function (no escaping store anywhere) was tried first and the
    // escape/arena gate correctly granted THAT loop its own per-iteration
    // arena (Task 6, interprocedural: nothing escapes the call, so the loop
    // safely reclaims `scratchSum`'s internal allocations between calls
    // without any Task-7 code running at all) — which made an early
    // version of this test pass even against the pre-Task-7 codegen,
    // defeating it as a TDD RED/GREEN pin. With the loop's own arena
    // genuinely vetoed via `taint`, NOTHING but a Task-7 per-call function
    // arena on `scratchSum` itself can reclaim its scratch, confirmed by
    // re-running this exact fixture against the pre-Task-7 codegen and
    // observing a real E4000 allocation-failure trap (see the task report).
    //
    // Cumulative: 300 calls x ~64KB ~= 18.3MB, over 2x an 8MB memory cap —
    // passes only if the per-call function arena actually reclaims between
    // calls. (The fill and sum are folded into ONE loop over `buf`, not two,
    // to keep the total instruction count in the same ballpark as the
    // already-passing `per_iteration_loop_allocations_are_reclaimed` above
    // rather than tripping the CPU-fuel guard instead of the memory cap.)
    let source = write_temp_source(
        "function_scratch",
        r#"function taint(v) {
  return { v: v };
}
function scratchSum(seed) {
  const marker = { tag: seed };
  const buf = new Array(8000);
  let sum = 0;
  for (let i = 0; i < 8000; i = i + 1) {
    buf[i] = seed + i;
    sum = sum + buf[i];
  }
  return sum;
}
function main() {
  const taints = new Array(1);
  let total = 0;
  for (let call = 0; call < 300; call = call + 1) {
    taints[0] = taint(call);
    total = total + scratchSum(call);
  }
  console.log(total + taints[0].v - taints[0].v);
}
main();
"#,
    );
    let policy = write_temp_policy_json("function_scratch_mem8", MEM8_POLICY_JSON);
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // scratchSum(seed) = sum_{i=0}^{7999} (seed+i) = 8000*seed + 31996000.
    // total = sum_{call=0}^{299} scratchSum(call)
    //       = 8000 * (0+...+299) + 300*31996000
    //       = 8000*44850 + 9598800000 = 358800000 + 9598800000
    //       = 9957600000.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "9957600000\n");
}

#[test]
fn recursive_function_arena_sound() {
    // `recurse` opens a function-body arena (its `scratch` object literal is
    // read into a scalar and then discarded — ScopeLocal fate, never
    // returned — so the gate grants it `opens_arena`, unlike a factory that
    // returns its allocation). Recursing to depth 500 means up to 501
    // function-arena frames are simultaneously OPEN (each recursive call's
    // prologue runs before the next level's, and none release until their
    // own call returns): this pins that the saved-trio locals for each stack
    // frame are genuinely separate wasm locals (one physical local slot per
    // ACTIVATION, not a single shared slot silently clobbered by the next
    // nested call) — if they aliased across frames, an inner frame's release
    // would restore the WRONG (some other frame's) saved arena state into
    // the globals, corrupting every shallower frame's still-live scratch and
    // producing a wrong sum (or a trap), not just the right one by luck.
    let source = write_temp_source(
        "recursive_function_arena",
        r#"function recurse(depth) {
  const scratch = { a: depth, b: depth * 2, c: depth * 3 };
  let sum = scratch.a + scratch.b + scratch.c;
  if (depth <= 0) {
    return sum;
  }
  return sum + recurse(depth - 1);
}
function main() {
  console.log(recurse(500));
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Per-frame sum at depth d = d + 2d + 3d = 6d. total = sum_{d=0}^{500} 6d
    // = 6 * (500*501/2) = 6*125250 = 751500.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "751500\n");
}

#[test]
fn factory_functions_get_no_arena() {
    // BINARY-TREES-CRITICAL (RISK 1): a `bottomUpTree`-shaped factory — every
    // site inside it is Returned, never ScopeLocal (its own object literal
    // IS the return value, and the two recursive calls' results are threaded
    // straight into the returned object's fields) — called from a loop in
    // `main` that itself opens a per-ITERATION arena (Task 6): the tree is
    // built and checked, then discarded before the next iteration, a
    // classic loop-arena grant.
    //
    // If `bottomUpTree` were WRONGLY granted its own per-call FUNCTION arena
    // (this task's new machinery), each recursive call's returned subtree
    // would be allocated from a per-call arena that gets `__arena_reset` the
    // instant that call returns — so by the time the PARENT call reads
    // `left`/`right` back and returns its own (parent-level) object, the
    // child calls' pages are already back on the free list and get reused
    // (and overwritten) by the very next sibling call or the parent's own
    // allocation, corrupting the tree before `itemCheck` ever walks it. That
    // failure mode produces a wrong `total`, not a crash, which is exactly
    // why this test pins an EXACT output rather than just "doesn't trap".
    //
    // This test's job is to prove the codegen consumes `opens_arena`
    // faithfully — i.e., it does NOT independently second-guess or
    // over-grant beyond what the escape gate (kali_mir, already shipped)
    // decided.
    let source = write_temp_source(
        "factory_no_arena",
        r#"function bottomUpTree(depth) {
  if (depth > 0) {
    return { left: bottomUpTree(depth - 1), right: bottomUpTree(depth - 1) };
  }
  return { left: null, right: null };
}
function itemCheck(t) {
  if (t.left === null) {
    return 1;
  }
  return 1 + itemCheck(t.left) + itemCheck(t.right);
}
function main() {
  let total = 0;
  for (let i = 0; i < 100; i = i + 1) {
    const tree = bottomUpTree(10);
    total = total + itemCheck(tree);
  }
  console.log(total);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // A full depth-10 tree has itemCheck = 2^11 - 1 = 2047 nodes;
    // 100 iterations => 100 * 2047 = 204700.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "204700\n");
}

#[test]
fn mixed_returned_and_scratch_function_is_sound() {
    // FINAL-REVIEW CRITICAL: `make` has BOTH a dying scratch object
    // (`scratch`, a 6-field object read into a scalar and then discarded —
    // ScopeLocal fate, exactly the shape `opens_arena_only_with_local_sites`
    // grants a function arena for) AND a returned fresh object (`{ v: s }` —
    // Returned fate). The old gate rule `opens_arena(f) = arena_eligible(f)
    // && has_scope_local_site` looked ONLY at the ScopeLocal bit and ignored
    // the Returned site entirely, so it wrongly granted `make` its own
    // per-call function arena. Every fresh-heap site in an arena-eligible
    // function routes into the SAME current arena, so the returned `{ v: s
    // }` was ALSO allocated into that per-call arena — and `make`'s epilogue
    // resets/releases that arena on every exit path, splicing the just
    // *returned* object's backing page onto the free list the instant the
    // call returns.
    //
    // `make` is called TWICE per iteration so the LIFO free-list clobber is
    // observable within a single iteration: `r1 = make(i)`'s returned page is
    // freed when `make` returns, and `r2 = make(i + 1000)`'s own allocations
    // (scratch + its own returned object) reuse that freed page from the free
    // list BEFORE `r1.v` is ever read back — so `r1.v` reads back corrupted
    // (overwritten by `r2`'s call), producing the wrong sum. Confirmed RED
    // against the pre-fix gate (`arena_gate.rs` with the `has_returned_site`
    // exclusion reverted): this exact fixture printed `1262400`, not the
    // correct `662400` — see the Final-review Critical section of
    // task-9-report.md for the stash/rebuild/restore transcript.
    let source = write_temp_source(
        "mixed_returned_and_scratch",
        r#"function make(seed) {
  const scratch = { a: seed, b: seed + 1, c: seed + 2, d: seed + 3, e: seed + 4, f: seed + 5 };
  const s = scratch.a + scratch.b + scratch.c + scratch.d + scratch.e + scratch.f;
  return { v: s };
}
function main() {
  let total = 0;
  for (let i = 0; i < 100; i = i + 1) {
    const r1 = make(i);
    const r2 = make(i + 1000);
    total = total + r1.v + r2.v;
  }
  console.log(total);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // scratch's fields sum to 6*seed + (0+1+2+3+4+5) = 6*seed + 15, so
    // make(seed).v = 6*seed + 15.
    // r1.v = 6*i + 15; r2.v = 6*(i+1000) + 15 = 6*i + 6015.
    // r1.v + r2.v = 12*i + 6030.
    // total = sum_{i=0}^{99} (12*i + 6030) = 12*(99*100/2) + 100*6030
    //       = 12*4950 + 603000 = 59400 + 603000 = 662400.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "662400\n");
}

#[test]
fn mixed_scratch_and_returned_call_result_is_sound() {
    // FINAL-REVIEW CRITICAL, ROUND 2: same use-after-reset hazard as
    // `mixed_returned_and_scratch_function_is_sound`, but the returned heap
    // value is a CALL-RESULT (`return build(s);`) rather than a bare or
    // name-bound object literal. Round 1's fix (`arena_gate.rs` commit
    // 7d2af1a30) only detected `has_returned_site` via two SHAPE-SPECIFIC
    // paths — `arena_is_fresh_literal` in `arena_note_return` (bare-literal
    // returns) and `binding.returned` in
    // `arena_finalize_current_function` (name-bound-literal returns) — and
    // MISSED this shape entirely: `return build(s)` is syntactically neither
    // a literal nor a fresh-heap binding of `make`, so round 1 still grants
    // `make` its own per-call function arena, which resets on exit and
    // splices the just-returned `{ v: s }` object's backing page onto the
    // free list the instant `make` returns — the exact same LIFO free-list
    // clobber as the round-1 pin, just reached through a call-result instead
    // of a literal. Round 2 generalizes the veto via a deferred
    // `push_returned_site` resolved against the escape-flow fixpoint, which
    // must resolve `return build(s)`'s class (`DependsOn(Return { build })`)
    // as may-heap because `build`'s own return is a fresh object literal.
    //
    // Confirmed RED against the round-1-only gate (this test's source
    // reproduces the same corrupted-sum failure mode as
    // `mixed_returned_and_scratch_function_is_sound` when only round 1's
    // shape-specific detection is present — see the Final-review Critical
    // round 2 section of task-9-report.md for the stash/rebuild/restore
    // transcript).
    let source = write_temp_source(
        "mixed_scratch_and_returned_call_result",
        r#"function build(v) {
  return { v: v };
}
function make(seed) {
  const scratch = { a: seed, b: seed + 1, c: seed + 2, d: seed + 3, e: seed + 4, f: seed + 5 };
  const s = scratch.a + scratch.b + scratch.c + scratch.d + scratch.e + scratch.f;
  return build(s);
}
function main() {
  let total = 0;
  for (let i = 0; i < 100; i = i + 1) {
    const r1 = make(i);
    const r2 = make(i + 1000);
    total = total + r1.v + r2.v;
  }
  console.log(total);
}
main();
"#,
    );
    let output = std::process::Command::new(kali_bin())
        .arg("run")
        .arg(&source)
        .output()
        .expect("run kali");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Identical arithmetic to `mixed_returned_and_scratch_function_is_sound`
    // (`build` just wraps `s` in `{ v: s }`, same as the literal it replaces):
    // make(seed).v = 6*seed + 15; total = sum_{i=0}^{99} (12*i + 6030) =
    // 12*4950 + 603000 = 662400.
    assert_eq!(String::from_utf8_lossy(&output.stdout), "662400\n");
}
