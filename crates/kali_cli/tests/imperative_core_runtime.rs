use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_js(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn relational_operators_compute_booleans() {
    assert_eq!(run_js("console.log(3 < 5);\n"), "1\n");
    assert_eq!(run_js("console.log(5 < 3);\n"), "0\n");
    assert_eq!(run_js("console.log(5 > 3);\n"), "1\n");
    assert_eq!(run_js("console.log(3 >= 3);\n"), "1\n");
    assert_eq!(run_js("console.log(2 <= 1);\n"), "0\n");
}

#[test]
fn functions_return_computed_values() {
    assert_eq!(
        run_js("function id(x) { return x; }\nconsole.log(id(42));\n"),
        "42\n"
    );
    assert_eq!(
        run_js("function add(a, b) { return a + b; }\nconsole.log(add(40, 2));\n"),
        "42\n"
    );
    assert_eq!(
        run_js("function dbl(x) { return x * 2; }\nconsole.log(dbl(21));\n"),
        "42\n"
    );
}

#[test]
fn mutable_locals_round_trip() {
    assert_eq!(run_js("let x = 5;\nconsole.log(x);\n"), "5\n");
    assert_eq!(run_js("let x = 5;\nx = x + 1;\nconsole.log(x);\n"), "6\n");
    assert_eq!(run_js("let x = 0;\nx = 9;\nconsole.log(x);\n"), "9\n");
    assert_eq!(
        run_js("function f() { let x = 5; x = x + 1; return x; }\nconsole.log(f());\n"),
        "6\n"
    );
}

#[test]
fn loops_iterate() {
    // while with a counter and accumulator
    assert_eq!(
        run_js(
            "let s = 0;\nlet i = 0;\nwhile (i < 5) { s = s + i; i = i + 1; }\nconsole.log(s);\n"
        ),
        "10\n"
    );
    // for loop
    assert_eq!(
        run_js("let s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + i; }\nconsole.log(s);\n"),
        "10\n"
    );
    // break out of while(true)
    assert_eq!(
        run_js("let i = 0;\nwhile (true) { if (i >= 3) { break; } i = i + 1; }\nconsole.log(i);\n"),
        "3\n"
    );
    // do-while runs body first
    assert_eq!(
        run_js("let i = 0;\nlet n = 0;\ndo { n = n + 1; i = i + 1; } while (i < 4);\nconsole.log(n);\n"),
        "4\n"
    );
    // recursion now terminates (relational base case + real calls)
    assert_eq!(
        run_js(
            "function s(n) { if (n < 1) { return 0; } return n + s(n - 1); }\nconsole.log(s(5));\n"
        ),
        "15\n"
    );
}

#[test]
fn integer_arrays_read_write() {
    assert_eq!(
        run_js("const a = new Array(3);\na[0] = 10;\na[1] = 20;\na[2] = a[0] + a[1];\nconsole.log(a[2]);\n"),
        "30\n"
    );
    // dynamic index from a loop variable
    assert_eq!(
        run_js("const a = new Array(5);\nfor (let i = 0; i < 5; i = i + 1) { a[i] = i * i; }\nlet s = 0;\nfor (let i = 0; i < 5; i = i + 1) { s = s + a[i]; }\nconsole.log(s);\n"),
        "30\n"
    );
    // swap via a temp (the fannkuch inner idiom)
    assert_eq!(
        run_js("const a = new Array(2);\na[0] = 7;\na[1] = 9;\nconst t = a[0];\na[0] = a[1];\na[1] = t;\nconsole.log(a[0]);\nconsole.log(a[1]);\n"),
        "9\n7\n"
    );
}

#[test]
fn strict_equality_operators_parse_and_compute() {
    // constant-fold path
    assert_eq!(run_js("console.log(4 === 4);\n"), "1\n");
    assert_eq!(run_js("console.log(4 === 5);\n"), "0\n");
    assert_eq!(run_js("console.log(4 !== 5);\n"), "1\n");
    // dynamic path
    assert_eq!(run_js("let r = 4;\nconsole.log(r === 4);\n"), "1\n");
    assert_eq!(run_js("let r = 4;\nconsole.log(r === 5);\n"), "0\n");
    // === inside a loop condition / if (the fannkuch shape)
    assert_eq!(
        run_js("let r = 0;\nwhile (r !== 4) { r = r + 1; }\nconsole.log(r);\n"),
        "4\n"
    );
    assert_eq!(
        run_js("let s = 0;\nlet i = 0;\nwhile (i < 6) { i = i + 1; if (i === 3) { continue; } s = s + i; }\nconsole.log(s);\n"),
        "18\n"
    );
}

#[test]
fn computed_array_subscripts() {
    // read with a[i+1] / a[i-1]
    assert_eq!(
        run_js("const a = new Array(3);\na[0]=10;\na[1]=20;\na[2]=30;\nlet i = 0;\nconsole.log(a[i + 1]);\n"),
        "20\n"
    );
    // write with a[r-1]
    assert_eq!(
        run_js("const a = new Array(3);\nlet r = 2;\na[r - 1] = 99;\nconsole.log(a[1]);\n"),
        "99\n"
    );
    // the fannkuch shift idiom: perm1[i] = perm1[i+1]
    assert_eq!(
        run_js("const a = new Array(4);\nfor (let i = 0; i < 4; i = i + 1) { a[i] = i; }\nlet i = 0;\nwhile (i < 3) { a[i] = a[i + 1]; i = i + 1; }\nconsole.log(a[0]);\nconsole.log(a[1]);\nconsole.log(a[2]);\n"),
        "1\n2\n3\n"
    );
    // literal-index static fold must still work (regression guard)
    assert_eq!(
        run_js("const a = [10, 20, 30];\nconsole.log(a[0] + a[1] + a[2]);\n"),
        "60\n"
    );
    // identifier index still works (Task 5 path)
    assert_eq!(
        run_js("const a = new Array(2);\na[0]=7;\na[1]=9;\nlet j=1;\nconsole.log(a[j]);\n"),
        "9\n"
    );
}
