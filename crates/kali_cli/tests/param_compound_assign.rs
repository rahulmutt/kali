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
