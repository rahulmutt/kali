use std::{path::PathBuf, process::Command};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/benchmarks")
        .join(name)
}

fn run_fixture_with_policy(source_name: &str, policy_name: &str, n: &str) -> std::process::Output {
    let source = fixture(source_name);
    let policy = fixture(policy_name);
    Command::new(kali_bin())
        .arg("run")
        .arg("--api")
        .arg("node")
        .arg("--sandbox")
        .arg(&policy)
        .arg(&source)
        .arg("--")
        .arg(n)
        .output()
        .expect("run kali")
}

// Bounded-peak reclamation proof (fasta Spec 7 Task 4e). `join-loop-peak.ts`
// wraps its while loop in a named function (`run`) with a per-iteration
// heap `marker` object literal, so the loop is granted its own
// per-iteration arena (Task 6) and its `line.join("") + "!"` string site is
// proven iteration-local (Task 4b), routing both the join and the `+`
// concat to the resettable `__join_arena`/`string_concat_arena` twins
// (Task 4c/4d) instead of the never-reset global allocator.
//
// Per-iteration arena churn: `line.join("")` produces a 60-byte string and
// `+ "!"` produces a 61-byte string; both round up to 64 bytes at the
// 8-byte-aligned bump allocator (`(len + 7) & !7`), so each iteration bumps
// ~128 bytes off the loop's current arena page before `__arena_reset`
// recycles it back onto the free list at the top of the NEXT iteration.
// Without reclamation (every string call hard-wired to the global,
// never-reset bump allocator) N=500,000 iterations would need roughly
// 500,000 * 128B ~= 64MB just for the join/concat garbage (before even
// counting the per-iteration `marker` object or page/header overhead) --
// this test's fixed 4MB policy budget (`join-loop-peak.policy.json`) sits
// far below that, so a large N only fits if reclamation is real: with it,
// the loop's own arena page is continuously popped off and returned to the
// shared free list every iteration, so linear memory only ever needs to
// grow to a small constant number of pages (module baseline + `line`'s own
// single-page function-body arena + the loop's one recycled page),
// independent of N.
//
// Discrimination evidence (not re-run by this test, recorded in the task
// report): the SAME 4MB policy against a MODULE-SCOPE variant of this exact
// program (the while loop hoisted out of any function, at `_start` scope --
// module-scope sites are, by design, never arena-eligible) passes at
// N=1,000 but traps E4000 (allocation failure) at N=500,000 under the
// identical budget, confirming the budget is genuinely tight enough to
// discriminate reclaimed-vs-not, not merely generous enough to always pass.
#[test]
fn join_concat_loop_has_bounded_peak() {
    for n in ["1000", "500000"] {
        let out = run_fixture_with_policy("join-loop-peak.ts", "join-loop-peak.policy.json", n);
        assert!(
            out.status.success(),
            "N={n} should fit the fixed small budget under reclamation; stderr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        let expected_n: usize = n.parse().expect("n is numeric");
        assert_eq!(
            stdout.lines().count(),
            expected_n,
            "N={n} should print exactly N lines"
        );
        assert!(
            stdout
                .lines()
                .all(|line| line.len() == 61 && line.ends_with('!')),
            "N={n}: every line should be 60 'x's followed by '!'"
        );
    }
}
