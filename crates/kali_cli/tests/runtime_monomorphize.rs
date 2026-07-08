//! End-to-end acceptance for object-shape monomorphization (fasta Spec 5,
//! Task 7a-2). A function reached by two *distinct* object-param shape tuples
//! must compile — the AST clone+rename transform specializes it per shape so
//! each clone lowers monomorphically — while genuinely-ambiguous merges still
//! fail closed (E5506), never miscompile.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-mono-{}-{}-{}",
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

#[test]
fn dump_two_distinct_shapes_prints_three_then_two() {
    // The design-doc repro. Before the transform this errors E5506 (one param
    // `t` reached by two distinct shapes {a,b,c} and {x,y}); after, `dump` is
    // cloned per shape and each clone's for..in bakes its own field count.
    // node prints `3\n2`.
    let src = "function dump(t){var s=0;for(var k in t){s=s+1;}return s;}\n\
               var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0};\n\
               console.log(dump(A)); console.log(dump(B));\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n2\n");
}

#[test]
fn transitive_outer_inner_two_shapes_prints_three_then_two() {
    // Probe P4: `outer(t)` forwards `t` to `inner(t)`. Both levels must be
    // specialized transitively — the clone of `outer` for {a,b,c} must call the
    // {a,b,c} clone of `inner`. node prints `3\n2`.
    let src = "function inner(t){var s=0;for(var k in t){s=s+1;}return s;}\n\
               function outer(t){return inner(t);}\n\
               var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0};\n\
               console.log(outer(A)); console.log(outer(B));\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n2\n");
}

#[test]
fn nested_fn_decl_caller_still_rejects_cleanly() {
    // Task 7a-2 follow-up (fail-closed guard): `outer` is reached by two
    // distinct shapes but contains a nested `function helper(){...}`
    // declaration. Cloning `outer` would duplicate `helper` into two
    // same-named wasm exports, which wasm validation would reject with an
    // opaque duplicate-export error. The guard drops `outer` from
    // specializations instead, so this now fails closed with the existing
    // clean E5506 conflicting-object-shapes diagnostic (never miscompiles,
    // never surfaces the opaque wasm error).
    let src = "function outer(t){ function helper(){ return 1; } var s=0; for(var k in t){ s=s+1; } return s + helper(); }\n\
               var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0};\n\
               console.log(outer(A)); console.log(outer(B));\n";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "outer with a nested fn decl must fail closed, not miscompile; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected the clean E5506 conflicting-object-shapes diagnostic, not an \
         opaque downstream error; stderr: {stderr}"
    );
}

#[test]
fn ambiguous_conditional_merge_still_rejects() {
    // Fail-closed pin (design §4): `var o = cond ? A : B; dump(o)` merges two
    // shapes into one slot at one use site — no per-call-site partition exists,
    // so the plan is empty for `dump` and the existing E5506 conflict must still
    // fire. Reject, never miscompile.
    let src = "function dump(t){var s=0;for(var k in t){s=s+1;}return s;}\n\
               var A={a:1.0,b:2.0,c:3.0}; var B={x:1.0,y:2.0}; var cond=1.0;\n\
               var o = cond ? A : B; console.log(dump(o));\n";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "ambiguous cond ? A : B merge must fail closed (E5506); stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
