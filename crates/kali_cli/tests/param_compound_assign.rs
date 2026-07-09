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
