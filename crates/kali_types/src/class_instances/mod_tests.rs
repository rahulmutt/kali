use super::classes::{ClassPlans, RewrittenClass};
use super::sweep;
use crate::test_support::parse_statements;

fn plans() -> ClassPlans {
    let mut plans = ClassPlans::default();
    plans.rewritten.insert(
        "C".into(),
        RewrittenClass {
            name: "C".into(),
            fields: vec![],
            methods: ["m".to_string()].into_iter().collect(),
            leading_run: 0,
            ctor_params: vec![],
        },
    );
    plans
}

fn sweep_messages(src: &str) -> Vec<String> {
    let mut stmts = parse_statements(src);
    sweep(&mut stmts, &plans()).into_iter().map(|d| d.message).collect()
}

fn unlowered() -> String {
    kali_common::class_construction_unavailable_message("C", kali_common::CLASS_REASON_UNLOWERED)
}

#[test]
fn a_leftover_class_reference_is_swept_once() {
    assert_eq!(sweep_messages("function f(){ return C; } const d = C;"), [unlowered()]);
}

#[test]
fn a_user_binding_shadowing_the_class_name_is_not_swept() {
    assert!(sweep_messages("function f(C){ return C; }").is_empty());
}

#[test]
fn a_leftover_new_is_swept() {
    assert_eq!(sweep_messages("const o = new C(1);"), [unlowered()]);
}

#[test]
fn a_this_in_a_generated_function_is_swept_but_not_in_a_nested_arrow_free_program() {
    assert_eq!(sweep_messages("function C__m(__this){ return this.n; }"), [unlowered()]);
    assert_eq!(sweep_messages("function main(){ return () => this; }"), Vec::<String>::new());
}
