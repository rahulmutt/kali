//! Unresolved-member-read spec §3.4 through the real `analyze_source_file`
//! wiring: the read mirror's refusal on a class-instance receiver defers to
//! the class-instance rewrite (Task 7 ruling 2).

use super::*;
use tempfile::TempDir;

const UNRES_READ: &str = "no lowering for that read";

fn analyze_errors(source: &str) -> Vec<String> {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("main.js");
    std::fs::write(&path, source).expect("write main.js");
    match analyze_source_file(&path, ApiSurface::Node, &[], false, false) {
        Ok(analyzed) => analyzed
            .diagnostics
            .into_iter()
            .filter(|d| d.is_error())
            .map(|d| d.message)
            .collect(),
        Err(diagnostics) => diagnostics.into_iter().map(|d| d.message).collect(),
    }
}

#[test]
fn an_undeclared_class_field_read_reports_the_class_message_only() {
    let errors = analyze_errors(
        "class C{ constructor(){ this.n=1; } } const c=new C(); console.log(c.zz);\n",
    );
    assert!(
        errors
            .iter()
            .any(|m| m.contains("is not declared on class `C`")),
        "{errors:?}"
    );
    assert!(!errors.iter().any(|m| m.contains(UNRES_READ)), "{errors:?}");
}

#[test]
fn the_held_read_refusal_speaks_when_the_class_rewrite_stays_quiet() {
    // A stateless `extends` chain is not rewritten and not refused, so the
    // mirror's held refusal is the one that fails the program.
    let errors = analyze_errors(
        "class A{f(){return 1;}} class B extends A{g(){return 2;}} const b=new B(); console.log(\"z=\"+b.z);\n",
    );
    assert_eq!(
        errors.iter().filter(|m| m.contains(UNRES_READ)).count(),
        1,
        "{errors:?}"
    );
}
