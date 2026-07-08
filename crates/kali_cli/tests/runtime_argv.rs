use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_node_source_with_args(src: &str, args: &[&str]) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-argv-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    let mut cmd = Command::new(kali_bin());
    cmd.arg("run").arg("--api").arg("node").arg(&path).arg("--");
    for a in args {
        cmd.arg(a);
    }
    cmd.output().expect("run kali")
}

#[test]
fn process_argv_element_read_yields_the_string() {
    // On the node surface argv == ["node", <src>, ...guestArgs], so argv[2] is
    // the first guest arg. Printing it must echo "hello".
    let out = run_node_source_with_args("console.log(process.argv[2]);\n", &["hello"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello\n");
}

#[test]
fn process_argv_element_length_is_the_byte_count() {
    let out = run_node_source_with_args("console.log(process.argv[2].length);\n", &["abcd"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n");
}

// --- Fail-closed pins: bounded fail-open follow-up (Spec 5 Task 5) ---
//
// `expression_is_nonneg_int_literal` (kali_types) used to accept ANY
// non-negative whole-valued `f64` literal as a static argv index, including
// values above `i64::MAX`, while codegen's `is_process_argv_element` parses
// the literal's TEXT via `str::parse::<i64>()` (which returns `None` on
// overflow). That let kali_types PROVE a runtime-string classification for
// an index codegen could never actually recognize as one — a bounded
// both-sides fail-open. The fix bounds the types-side predicate to
// `Number.MAX_SAFE_INTEGER` (2^53 - 1) so every accepted literal round-trips
// through codegen's i64 parse with no residual boundary mismatch.
//
// Observed behavior (run, then pinned — not assumed): an index literal
// above the safe-integer bound is NOT recognized by
// `is_process_argv_element` on EITHER side (never was, and still isn't —
// the codegen recognizer was always i64-parse-based). Both before and after
// the kali_types fix, codegen takes the SAME "unrecognized argv index
// shape" fallback that a negative literal (`process.argv[-1]`) or a
// variable index (`process.argv[i]`) already take: it emits the numeric
// placeholder `0` rather than a real `args_get` string handle, so
// `console.log(process.argv[<hugeLiteral>])` prints the placeholder `0`
// (never the real guest arg) and the program runs to completion rather
// than trapping. What the kali_types fix actually changes is upstream of
// this observable point (`identifier`/`+`-operand string-typedness for a
// `const` binding of such a read no longer over-claims a proven string) —
// this pin locks the externally observable contract: the program must
// NEVER succeed with a *real* string value (leaking a genuine arg or
// exercising the runtime-string `args_get`/tagged-handle machinery) for an
// index kali_types cannot actually prove codegen recognizes.
#[test]
fn process_argv_huge_literal_index_never_flows_as_a_real_string() {
    // 10_000_000_000_000_000_000 > i64::MAX (~9.22e18): codegen's
    // `parse_number_literal` overflows and returns `None`, so this index is
    // never a recognized argv element on the codegen side, before or after
    // the kali_types fix.
    let out = run_node_source_with_args(
        "console.log(process.argv[10000000000000000000]);\n",
        &["hello"],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Must NOT leak the real guest arg (that would mean the pathological
    // literal was somehow honored as a genuine argv index/string).
    assert_ne!(stdout, "hello\n");
    // Takes the same harmless numeric-placeholder fallback as every other
    // unrecognized argv index shape (negative literal, variable index).
    assert_eq!(stdout, "0\n");
}

#[test]
fn process_argv_variable_index_never_flows_as_a_real_string() {
    // A non-literal (variable) index was never claimed as a static argv
    // element by either side — this pin guards the fallback contract stays
    // the same shape as the huge-literal pin above (no reliance on a
    // dynamic value being smuggled through as a string).
    let out = run_node_source_with_args(
        "var i = 2;\nconsole.log(process.argv[i]);\n",
        &["hello"],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_ne!(stdout, "hello\n");
    assert_eq!(stdout, "0\n");
}

// --- Fail-closed pins: `.length` on an unprovable argv index (Spec 5 Task 5
//     follow-up 2) ---
//
// The bounded-literal fix above kept BOTH the `is_process_argv_element`
// recognizers (codegen i64 parse) and the kali_types string predicates in
// lockstep, so `process.argv[<huge>]` is not classified as a string on either
// side. But the `.length` gate had NO arm for "an argv element whose index is
// not the provable subset": because such a receiver is (correctly) not a
// string, `reject_unprovable_string_length` treated it as an unrelated
// non-string `.length` and stayed silent — and codegen's STATIC console.log
// render (`render_length`) then folded `process.argv[<huge>].length` to the
// argv-element node's CHILD COUNT (`2`, its `[argv, index]` children), a bogus
// number where Node THROWS (`argv[huge]` is `undefined`). A third recognizer
// lane (the static render fold) outside the string/`.length` mirror. The fix
// adds a fail-closed arm keyed on the structural `process.argv[<any index>]`
// shape MINUS the provable-element subset: any argv `.length` whose index is
// not a static non-negative integer literal that round-trips through codegen's
// i64 parse now rejects E5506 rather than miscompiling.
//
// Behavior below RUN on the rebuilt binary before pinning (not assumed).
#[test]
fn process_argv_huge_literal_index_length_fails_closed() {
    // `process.argv[<huge>].length`: Node throws `TypeError: Cannot read
    // properties of undefined (reading 'length')` and exits non-zero. Kali must
    // NOT miscompile it to the child-count `2` — it must fail closed.
    let out = run_node_source_with_args(
        "console.log(process.argv[10000000000000000000].length);\n",
        &["hello"],
    );
    assert!(
        !out.status.success(),
        "argv[huge].length must fail closed, got stdout: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506, stderr: {stderr}"
    );
    // The specific pre-fix miscompile (child-count fold) must never surface.
    assert_ne!(String::from_utf8_lossy(&out.stdout), "2\n");
}

#[test]
fn process_argv_variable_index_length_fails_closed() {
    // A non-literal (variable) argv index is likewise not a provable element:
    // `process.argv[i].length` must fail closed for the same reason (the
    // fail-closed matrix rejects a non-provably-in-range / non-integer index).
    let out = run_node_source_with_args(
        "var i = 2;\nconsole.log(process.argv[i].length);\n",
        &["hello"],
    );
    assert!(
        !out.status.success(),
        "argv[i].length must fail closed, got stdout: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "expected E5506, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn process_argv_valid_index_length_still_succeeds() {
    // Regression control for the fix above: the PROVABLE argv-element subset
    // (a static non-negative integer literal) must keep taking the runtime
    // string `.length` lane and report the correct byte count.
    let out = run_node_source_with_args("console.log(process.argv[2].length);\n", &["abcd"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n");
}
