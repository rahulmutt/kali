use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    // Per-process AtomicU64 counter keeps the temp slug unique even when two
    // sources share a length (repo convention; avoids the macOS CI temp-slug
    // collision flake).
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-modglobals-{}-{}-{}",
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

/// The exact Spec 4a LCG shape: a module-scope `var` mutated (READ + WRITE)
/// from inside a function across calls. Pre-fix this hit E5506 (read) + E8001
/// (`=` write); now the persistent mutable global carries state across calls.
#[test]
fn module_var_written_and_read_from_function() {
    let out = run_source(
        "var g = 42;\nfunction bump(){ g = g + 1; return g; }\nconsole.log(bump());\nconsole.log(bump());\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "43\n44\n");
}

/// Read-only module var from a function (no write), plus a module-scope read.
#[test]
fn module_var_read_only_from_function() {
    let out = run_source(
        "var base = 100;\nfunction get(){ return base + 1; }\nconsole.log(get());\nconsole.log(base);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "101\n100\n");
}

/// The capstone's LCG with the float-division use site: an i64 module global is
/// mutated by integer arithmetic (`% 139968`) then read in a float context
/// (`(1.0 * s) / 139968`) — the repr system must convert the i64 GlobalGet to
/// f64 at the use site.
#[test]
fn module_var_lcg_float_division() {
    let src = "var s = 42;\nfunction r(){ s = (s * 3877 + 29573) % 139968; return (1.0 * s) / 139968; }\nconsole.log(r());\nconsole.log(r());\n";
    let out = run_source(src);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node -e on the same source:
    //   s = (42*3877+29573)%139968 = 192407 % 139968 = 52439 -> 52439/139968
    //   s = (52439*3877+29573)%139968 = ... -> matches node
    let expected = {
        // Compute the reference locally to avoid a hardcoded transcription error.
        let mut s: i64 = 42;
        let mut lines = String::new();
        for _ in 0..2 {
            s = (s * 3877 + 29573) % 139968;
            let v = (1.0 * s as f64) / 139968.0;
            lines.push_str(&format!("{}\n", v));
        }
        lines
    };
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected);
}

/// Compound assignment (`+=`) to a module global from a function decomposes to
/// GlobalGet/op/GlobalSet.
#[test]
fn module_var_compound_assign_from_function() {
    let out = run_source(
        "var acc = 0;\nfunction add(n){ acc += n; return acc; }\nconsole.log(add(5));\nconsole.log(add(7));\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n12\n");
}

/// FIX 1 fail-closed pin: a module OBJECT reassigned from a function but NEVER
/// member-accessed has a DEFAULT-`I64` repr (its shape is never proven), so a
/// naive "I64 minus member-bases" heuristic would promote it and store the
/// object handle in an i64 global (a memory-corruption-class fail-open, printed
/// `0` exit 0 pre-fix). The positive-scalar promotion proof requires the init
/// (`{x:1}`) to be numeric — it is not — so the binding is left unpromoted and
/// the assignment stays fail-closed (E5506).
#[test]
fn module_object_reassigned_never_member_accessed_is_rejected() {
    let out = run_source(
        "var o = { x: 1 };\nfunction f(){ o = { x: 2 }; return 0; }\nf();\nconsole.log(o);\n",
    );
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection, stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 fail-closed rejection, stderr: {stderr}"
    );
}

/// FIX 2: a local `var g` inside a function that shadows a promoted module
/// global `g` must write its OWN local slot (JS lexical scoping), NOT clobber
/// the module global. Pre-fix the declarator-init `GlobalSet` fired for the
/// local too, so the module `g` became 5 (node: 10).
#[test]
fn local_var_shadowing_module_global_does_not_clobber() {
    let out = run_source(
        "var g = 10;\nfunction f(){ var g = 5; return g + 1; }\nconsole.log(f());\nconsole.log(g);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "6\n10\n");
}

/// FIX 3: a param `n` that shadows a promoted module global `n` must read the
/// PARAM, not the global. Pre-fix the read `GlobalGet` branch sat ahead of the
/// param lookup, so `id(7)` returned the global 5 (node: 7).
#[test]
fn param_shadowing_module_global_reads_param() {
    let out = run_source(
        "var n = 5;\nfunction id(n){ return n; }\nconsole.log(id(7));\nconsole.log(n);\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n5\n");
}

/// SCOPE fail-closed pin: a mutable module-scope OBJECT mutated from a function
/// is a persistent heap root the GC-less region reclamation does not model.
/// It MUST stay rejected (E5506) — never silently lowered to a mutable global.
#[test]
fn module_object_mutation_from_function_still_rejected() {
    let out =
        run_source("var o = { x: 1 };\nfunction f(){ o.x = 2; return o.x; }\nconsole.log(f());\n");
    assert!(
        !out.status.success(),
        "expected a fail-closed rejection, stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 fail-closed rejection, stderr: {stderr}"
    );
}
