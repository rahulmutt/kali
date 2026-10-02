use super::*;
use kali_ast::{Expression, Statement};
use std::collections::{BTreeMap, BTreeSet};

fn parse(src: &str) -> Vec<Statement> {
    crate::test_support::parse_statements(src)
}

/// The argument of the first `return` in the first function of `src`.
fn first_return_arg(src: &str) -> Option<Expression> {
    for stmt in parse(src) {
        if let Statement::FunctionDeclaration(decl) = stmt {
            for s in &decl.body.body {
                if let Statement::ReturnStatement(r) = s {
                    return r.argument.clone();
                }
            }
        }
    }
    panic!("no return in {src}");
}

fn classify(src: &str) -> ReturnArg {
    let arg = first_return_arg(src);
    classify_return_arg(arg.as_ref(), &|n| n == "lit", &|n| n == "letlit")
}

#[test]
fn integer_literal_is_literal() {
    assert_eq!(
        classify("function f() { return [1, 2, 3]; }"),
        ReturnArg::Literal(None)
    );
}

#[test]
fn empty_literal_is_literal() {
    assert_eq!(
        classify("function f() { return []; }"),
        ReturnArg::Literal(None)
    );
}

#[test]
fn computed_integer_elements_are_literal() {
    assert_eq!(
        classify("function f(x) { return [x, x * 2, -x]; }"),
        ReturnArg::Literal(None)
    );
}

#[test]
fn boolean_string_nested_and_spread_elements_are_bad() {
    for src in [
        "function f() { return [true]; }",
        "function f() { return [\"a\"]; }",
        "function f() { return [[1]]; }",
        "function f(a) { return [...a]; }",
        "function f(x) { return [x > 1]; }",
        "function f() { return [null]; }",
    ] {
        assert_eq!(
            classify(src),
            ReturnArg::BadArray(kali_common::ARRAY_RETURN_ELEMENT),
            "{src}"
        );
    }
}

#[test]
fn const_literal_identifier_is_literal_and_let_literal_is_bad() {
    assert_eq!(
        classify("function f() { return lit; }"),
        ReturnArg::Literal(Some("lit".into()))
    );
    assert_eq!(
        classify("function f() { return letlit; }"),
        ReturnArg::BadArray(kali_common::ARRAY_RETURN_LET_LITERAL)
    );
}

#[test]
fn other_identifier_is_binding() {
    assert_eq!(
        classify("function f(x) { return x; }"),
        ReturnArg::Binding("x".into())
    );
}

#[test]
fn allocations_are_allocation() {
    assert_eq!(
        classify("function f() { return new Array(3); }"),
        ReturnArg::Allocation
    );
    assert_eq!(
        classify("function f() { return new Array(3).fill(2); }"),
        ReturnArg::Allocation
    );
    assert_eq!(
        classify("function f() { return (new Array(3)); }"),
        ReturnArg::Allocation
    );
}

#[test]
fn bare_call_is_call() {
    assert_eq!(
        classify("function f() { return g(); }"),
        ReturnArg::Call("g".into())
    );
}

#[test]
fn scalars_and_bare_return_are_non_array() {
    assert_eq!(classify("function f() { return 0; }"), ReturnArg::NonArray);
    assert_eq!(
        classify("function f() { return {a: 1}; }"),
        ReturnArg::NonArray
    );
    assert_eq!(classify("function f() { return; }"), ReturnArg::NonArray);
}

fn falls_off(src: &str) -> bool {
    for stmt in parse(src) {
        if let Statement::FunctionDeclaration(decl) = stmt {
            return body_falls_off_end(&decl.body.body);
        }
    }
    unreachable!()
}

#[test]
fn falls_off_end_detection() {
    assert!(!falls_off("function f() { return [1]; }"));
    assert!(falls_off("function f(c) { if (c) { return [1]; } }"));
    assert!(!falls_off(
        "function f(c) { if (c) { return [1]; } else { return [2]; } }"
    ));
    assert!(!falls_off("function f() { throw 1; }"));
    assert!(falls_off("function f() { }"));
}

#[test]
fn if_else_both_returning_does_not_fall_off() {
    assert!(!falls_off(
        "function f(c) { const x = 1; if (c) { return [x]; } else { if (c) { return [2]; } else { return [3]; } } }"
    ));
}

fn facts_one(name: &str, returns: Vec<ReturnArg>) -> ArrayReturnFacts {
    let mut f = ArrayReturnFacts::default();
    f.declaration_counts.insert(name.into(), 1);
    f.candidate_forms.insert(name.into());
    f.returns.insert(name.into(), returns);
    // Called by default, so taint behaviour is the called-function one; the
    // R8 tests clear it.
    f.called.insert(name.into());
    f
}

fn no_base(_: &str, _: &str) -> bool {
    false
}

#[test]
fn literal_only_function_is_array_returning() {
    let facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.contains("f"));
    assert!(s.tainted.is_empty());
}

#[test]
fn mixed_function_is_tainted_mixed() {
    let facts = facts_one("f", vec![ReturnArg::Literal(None), ReturnArg::NonArray]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(!s.array_returning.contains("f"));
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_MIXED));
}

#[test]
fn falling_off_the_end_taints_mixed() {
    let mut facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    facts.falls_off_end.insert("f".into());
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_MIXED));
}

#[test]
fn bad_array_reason_wins() {
    let facts = facts_one(
        "f",
        vec![
            ReturnArg::Literal(None),
            ReturnArg::BadArray(kali_common::ARRAY_RETURN_ELEMENT),
        ],
    );
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_ELEMENT));
}

#[test]
fn non_candidate_form_with_array_return_is_tainted_form() {
    // A named non-candidate (a named function expression) taints FORM; an
    // anonymous `__kali_fn_N` one is exempt (Task 4 step 10 narrowing).
    let mut facts = facts_one("g", vec![ReturnArg::Literal(None)]);
    facts.candidate_forms.clear();
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("g"), Some(&kali_common::ARRAY_RETURN_FORM));

    let mut facts = facts_one("__kali_fn_0", vec![ReturnArg::Literal(None)]);
    facts.candidate_forms.clear();
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("__kali_fn_0"), None);
}

#[test]
fn repeated_name_is_tainted_form() {
    let mut facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    facts.declaration_counts.insert("f".into(), 2);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_FORM));
}

#[test]
fn recursion_is_array_returning() {
    let facts = facts_one(
        "f",
        vec![ReturnArg::Literal(None), ReturnArg::Call("f".into())],
    );
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.contains("f"));
}

#[test]
fn mutual_recursion_is_array_returning() {
    let mut facts = facts_one(
        "f",
        vec![ReturnArg::Literal(None), ReturnArg::Call("g".into())],
    );
    facts.declaration_counts.insert("g".into(), 1);
    facts.candidate_forms.insert("g".into());
    facts
        .returns
        .insert("g".into(), vec![ReturnArg::Call("f".into())]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.contains("f"));
    assert!(s.array_returning.contains("g"));
}

#[test]
fn call_to_non_array_function_is_not_array_shaped() {
    let mut facts = facts_one("f", vec![ReturnArg::Call("g".into())]);
    facts.declaration_counts.insert("g".into(), 1);
    facts.candidate_forms.insert("g".into());
    facts.returns.insert("g".into(), vec![ReturnArg::NonArray]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.is_empty());
    assert!(
        s.tainted.is_empty(),
        "a scalar-returning call chain is untouched"
    );
}

#[test]
fn call_bound_binding_makes_return_admitted() {
    let mut facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    facts.declaration_counts.insert("g".into(), 1);
    facts.candidate_forms.insert("g".into());
    facts
        .returns
        .insert("g".into(), vec![ReturnArg::Binding("a".into())]);
    facts.call_bound.push(("g".into(), "a".into(), "f".into()));
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.call_bound.contains(&("g".to_string(), "a".to_string())));
    assert!(s.array_returning.contains("g"));
}

#[test]
fn param_fed_only_arrays_is_array_fed() {
    let facts = facts_one("f", vec![ReturnArg::Binding("x".into())]);
    let mut params = BTreeMap::new();
    params.insert("f".to_string(), vec!["x".to_string()]);
    let feeds = vec![Feed {
        caller: "_start".into(),
        callee: "f".into(),
        index: 0,
        shape: ArgShape::Allocation,
    }];
    let s = solve(&facts, &feeds, &params, &BTreeSet::new(), &no_base);
    assert!(s
        .array_fed_params
        .contains(&("f".to_string(), "x".to_string())));
    assert!(s.array_returning.contains("f"));
}

#[test]
fn param_with_any_scalar_feed_is_not_array_fed() {
    let facts = facts_one("f", vec![ReturnArg::Binding("x".into())]);
    let mut params = BTreeMap::new();
    params.insert("f".to_string(), vec!["x".to_string()]);
    let feeds = vec![
        Feed {
            caller: "_start".into(),
            callee: "f".into(),
            index: 0,
            shape: ArgShape::Allocation,
        },
        Feed {
            caller: "_start".into(),
            callee: "f".into(),
            index: 0,
            shape: ArgShape::Other,
        },
    ];
    let s = solve(&facts, &feeds, &params, &BTreeSet::new(), &no_base);
    assert!(s.array_fed_params.is_empty());
    assert!(s.array_returning.is_empty());
}

#[test]
fn escaping_function_params_are_never_array_fed() {
    let facts = facts_one("f", vec![ReturnArg::Binding("x".into())]);
    let mut params = BTreeMap::new();
    params.insert("f".to_string(), vec!["x".to_string()]);
    let feeds = vec![Feed {
        caller: "_start".into(),
        callee: "f".into(),
        index: 0,
        shape: ArgShape::Allocation,
    }];
    let escaping: BTreeSet<String> = ["f".to_string()].into();
    let s = solve(&facts, &feeds, &params, &escaping, &no_base);
    assert!(s.array_fed_params.is_empty());
}

#[test]
fn base_runtime_array_binding_is_admitted() {
    let facts = facts_one("f", vec![ReturnArg::Binding("a".into())]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &|f, n| {
        f == "f" && n == "a"
    });
    assert!(s.array_returning.contains("f"));
}

#[test]
fn non_array_binding_return_is_untouched() {
    let facts = facts_one("f", vec![ReturnArg::Binding("n".into())]);
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.is_empty());
    assert!(s.tainted.is_empty());
}

#[test]
fn classify_init_kinds() {
    let init = |src: &str| first_return_arg(src).expect("return argument");
    assert_eq!(
        classify_init(&init("function f() { return [1, 2]; }")),
        InitKind::ArrayLiteral
    );
    assert_eq!(
        classify_init(&init("function f() { return []; }")),
        InitKind::ArrayLiteral
    );
    assert_eq!(
        classify_init(&init("function f() { return new Array(3); }")),
        InitKind::Allocation
    );
    assert_eq!(
        classify_init(&init("function f() { return new Array(3).fill(0); }")),
        InitKind::Allocation
    );
    assert_eq!(
        classify_init(&init("function f() { return g(); }")),
        InitKind::Call("g".to_string())
    );
    assert_eq!(
        classify_init(&init("function f() { return (g)(1); }")),
        InitKind::Call("g".to_string())
    );
    assert_eq!(
        classify_init(&init("function f(o) { return o.g(); }")),
        InitKind::Other
    );
    assert_eq!(
        classify_init(&init("function f() { return 1; }")),
        InitKind::Other
    );
}

#[test]
fn fill_value_finds_the_value() {
    let arg = first_return_arg("function f() { return new Array(3).fill(4); }").unwrap();
    assert!(matches!(
        fill_value(&arg),
        Some(Expression::Literal(kali_ast::LiteralValue::Number(n))) if *n == 4.0
    ));
    let arg = first_return_arg("function f() { return new Array(3); }").unwrap();
    assert!(fill_value(&arg).is_none());
}

// Ruling R8: an uncalled, non-escaping function is never tainted; admission is
// unchanged.
#[test]
fn uncalled_function_is_not_tainted_but_a_called_one_is() {
    let returns = vec![ReturnArg::Literal(None), ReturnArg::NonArray];
    let mut facts = facts_one("f", returns.clone());
    facts.called.clear();
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), None);
    assert!(s.taint_exempt.contains("f"));

    let mut facts = facts_one(
        "f",
        vec![ReturnArg::BadArray(kali_common::ARRAY_RETURN_ELEMENT)],
    );
    facts.called.clear();
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), None);

    // Same function, called: tainted exactly as before.
    let facts = facts_one("f", returns.clone());
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_MIXED));
    assert!(!s.taint_exempt.contains("f"));

    // Uncalled but escaping: tainted.
    let mut facts = facts_one("f", returns);
    facts.called.clear();
    let escaping: BTreeSet<String> = ["f".to_string()].into();
    let s = solve(&facts, &[], &BTreeMap::new(), &escaping, &no_base);
    assert_eq!(s.tainted.get("f"), Some(&kali_common::ARRAY_RETURN_MIXED));
}

#[test]
fn uncalled_admissible_function_stays_array_returning() {
    let mut facts = facts_one("f", vec![ReturnArg::Literal(None)]);
    facts.called.clear();
    let s = solve(&facts, &[], &BTreeMap::new(), &BTreeSet::new(), &no_base);
    assert!(s.array_returning.contains("f"));
}
