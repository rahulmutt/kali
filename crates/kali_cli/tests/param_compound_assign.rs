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

// `x += 0.5` on a FLOAT parameter — the f64 compound lowering. Pins the
// float lane of the provably-scalar allowlist.
#[test]
fn param_compound_float_runs() {
    let src = "function f(x){x+=0.5;return x;} console.log(f(1.5));";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

// `s += "b"` on a STRING parameter — the string-concat compound lowering.
// Pins the string lane of the provably-scalar allowlist.
#[test]
fn param_compound_string_runs() {
    let src = "function f(s){s+=\"b\";return s;} console.log(f(\"a\"));";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ab\n");
}

// FAIL-CLOSED GUARD: compound-assign on a non-scalar (array) parameter must
// NOT miscompile — it must reject. Marking params mutable only removes the
// mutability barrier; the array repr still has no compound lowering, so this
// must still fail (never silently produce output). The param's array-ness is
// known only from the call-site flow (`g(xs)` where `xs` is an array), tracked
// via the `non_scalar_params` taint the resolve-phase allowlist consults.
#[test]
fn array_param_compound_still_rejects() {
    let src = "function g(a){a+=1;return a;} var xs=[1,2]; console.log(g(xs));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "array-param compound must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "array-param compound must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// `a++` (update expression) on an array parameter passed as a DIRECT array
// literal `f([1, 2])`. node prints `NaN`; kali must reject, not print `1`.
#[test]
fn array_param_update_increment_rejects() {
    let src = "function f(a){a++;return a;} console.log(f([1,2]));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "array-param update must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "array-param update must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// `p -= 2` on an array parameter (identifier arg). node prints `NaN`; kali
// must reject, not print `-2`.
#[test]
fn array_param_compound_minus_rejects() {
    let src = "function f(p){p-=2;return p;} var xs=[5,6]; console.log(f(xs));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "array-param compound `-=` must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "array-param compound `-=` must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// `p += 1` on an OBJECT parameter. node prints `[object Object]1`; kali
// previously leaked a raw heap address (`4105`) — it must reject. The object
// argument propagates `Repr::Object` onto the param scalar, which the
// provably-scalar allowlist rejects (it admits only I64/F64/String).
#[test]
fn object_param_compound_rejects() {
    let src = "var o={x:1}; function f(p){p+=1;return p;} console.log(f(o));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "object-param compound must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "object-param compound must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// `s++` (update expression) on a STRING parameter. node prints `NaN`; the
// compound-gate allowlist admits String (it has a compound `+=` lowering),
// but codegen's update arm (`emit_update_expression`) is I64-only with no
// string lowering — the narrower update-only gate must reject this fail
// closed rather than silently print the untouched string `a`.
#[test]
fn string_param_update_increment_rejects() {
    let src = "function f(s){s++;return s;} console.log(f(\"a\"));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "string-param update must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "string-param update must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// `x++` (update expression) on a FLOAT parameter. node prints `2.5`; codegen's
// update arm is I64-only, so pre-narrowing this reached codegen and failed
// late with an ugly E4201 WASM validation error. The narrower update-only
// gate must now reject this as a clean compile-time E5506, not E4201.
#[test]
fn float_param_update_increment_rejects() {
    let src = "function f(x){x++;return x;} console.log(f(1.5));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "float-param update must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "float-param update must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// POSITIVE-PROOF gate (fasta Spec 6 final-review follow-up). The `non_scalar_params`
// array taint only sees a DIRECT array argument (a bare-identifier array binding
// or a syntactic `[..]`/`new Array`). A param that receives an array through an
// INDIRECT call shape keeps the DEFAULT `I64` repr — indistinguishable from a
// genuine int param — and would pass a repr-only allowlist and miscompile. The
// gate now admits a compound/update param ONLY when interprocedural flow
// POSITIVELY proved it receives a scalar; every indirect array shape below is
// left unproven and rejects. Each reproducer prints a wrong scalar + exit 0 on
// the pre-fix binary (node disagrees); all must reject fail-closed.

// Indirect array via CALL RETURN: `f(g())` where `g` returns an array. node
// prints `1,21`; the pre-fix compiler printed `1`.
#[test]
fn param_compound_indirect_array_call_return_rejects() {
    let src = "function g(){return [1,2];} function f(p){p+=1;return p;} console.log(f(g()));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "indirect array (call-return) compound must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// Indirect array via a PASS-THROUGH param chain: `f(xs)` -> `h(a)` with the
// compound on `h`'s param. node prints `1,21`; the pre-fix compiler printed `1`.
#[test]
fn param_compound_indirect_array_passthrough_rejects() {
    let src = "var xs=[1,2]; function h(p){p+=1;return p;} function f(a){return h(a);} console.log(f(xs));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "indirect array (pass-through) compound must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// Same pass-through chain but with a `p++` UPDATE. node prints `NaN`; the
// pre-fix compiler printed `1`.
#[test]
fn param_update_indirect_array_passthrough_rejects() {
    let src = "var xs=[1,2]; function h(p){p++;return p;} function f(a){return h(a);} console.log(f(xs));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "indirect array (pass-through) update must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// Indirect array via a MEMBER-EXPRESSION argument: `f(o.a)` where `o.a` is an
// array. node prints `1,21`; the pre-fix compiler printed a wrong scalar.
#[test]
fn param_compound_indirect_array_member_expr_rejects() {
    let src = "var o={a:[1,2]}; function f(p){p+=1;return p;} console.log(f(o.a));";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "indirect array (member-expr) compound must reject, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

// NEVER-CALLED function: its param has NO call-site flow evidence at all, so the
// positive-proof gate cannot prove a scalar and rejects. This is acceptable and
// explicit: pre-branch, an immutable param already rejected any compound/update,
// so no working program regresses. Documents the positive-proof edge case.
#[test]
fn never_called_param_compound_rejects() {
    let src = "function f(n){n+=1;return n;} console.log(1);";
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "never-called param compound must reject (no scalar-inflow proof), got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stdout.is_empty(),
        "must produce NO stdout, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}
