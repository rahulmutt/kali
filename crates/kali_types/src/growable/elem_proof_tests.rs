use kali_ast::{Expression, Statement};

use super::{elem_proof, is_scalar_global_function, ElemProof};
use crate::growable::flow::GrowNode;

/// The initializer of the last `const x = <expr>;` in `src`.
fn init_of(src: &str) -> Expression {
    let statements = crate::test_support::parse_statements(src);
    statements
        .iter()
        .rev()
        .find_map(|statement| match statement {
            Statement::VariableDeclaration(d) => d.declarations.first()?.init.clone(),
            _ => None,
        })
        .expect("a declarator with an initializer")
}

fn proof(expr: &str) -> ElemProof {
    elem_proof("f", &init_of(&format!("const x = {expr};")))
}

fn binding(name: &str) -> ElemProof {
    ElemProof::Binding {
        func: "f".to_string(),
        name: name.to_string(),
    }
}

#[test]
fn numbers_and_number_valued_operators_are_yes() {
    for expr in ["1", "1.5", "-1", "(2)", "1 + 2 * 3", "i++", "--i", "-(1)"] {
        assert_eq!(proof(expr), ElemProof::Yes, "{expr}");
    }
}

#[test]
fn strings_templates_and_typeof_are_str() {
    for expr in ["\"s\"", "`t`", "typeof y", "\"ab\".slice(1)"] {
        assert_eq!(proof(expr), ElemProof::Str, "{expr}");
    }
    // A string method on a string is a string and its receiver one.
    assert_eq!(
        proof("\"a\".toUpperCase()"),
        ElemProof::All(vec![ElemProof::Str, ElemProof::Str])
    );
}

#[test]
fn values_that_may_be_no_number_or_string_are_no() {
    for expr in [
        "true",
        "null",
        "1 < 2",
        "a === b",
        "!y",
        "-1n",
        "{ a: 1 }",
        "[1]",
        "() => 1",
        "o.field",
        "new Array(3)",
        "a[i][j]",
        "o.m()",
        "(0, f)()()",
    ] {
        assert_eq!(proof(expr), ElemProof::No, "{expr}");
    }
}

#[test]
fn a_bare_identifier_is_a_binding_obligation() {
    assert_eq!(proof("y"), binding("y"));
    // Undeclared names are bindings too: the binding check refuses them.
    assert_eq!(proof("undefined"), binding("undefined"));
    assert_eq!(proof("+y"), binding("y"));
    assert_eq!(proof("y = z"), binding("z"));
}

#[test]
fn an_operator_conjoins_its_operands() {
    assert_eq!(
        proof("y + z"),
        ElemProof::All(vec![binding("y"), binding("z")])
    );
    assert_eq!(proof("c ? y : 1"), binding("y"));
    assert_eq!(proof("(1, y)"), binding("y"));
    // The parser gives `&&`, `||` and `??` as binary expressions, which the
    // proof does not admit: refused, fail-closed.
    for expr in ["y && z", "y || z", "y ?? z"] {
        assert_eq!(proof(expr), ElemProof::No, "{expr}");
    }
}

#[test]
fn an_add_assignment_conjoins_its_target_and_value() {
    // `s += "x"` is a string when `s` is: not a number.
    assert_eq!(
        proof("y += z"),
        ElemProof::All(vec![binding("y"), binding("z")])
    );
    assert_eq!(proof("y -= z"), ElemProof::Yes);
}

#[test]
fn a_bare_call_is_a_call_obligation() {
    assert_eq!(
        proof("g(1)"),
        ElemProof::Call {
            caller: "f".to_string(),
            callee: "g".to_string(),
        }
    );
    assert!(is_scalar_global_function("parseInt"));
    assert!(!is_scalar_global_function("g"));
    // An allocation call is an array.
    assert_eq!(proof("Array(3)"), ElemProof::No);
}

#[test]
fn element_reads_are_element_obligations() {
    let elements = ElemProof::Elements {
        func: "f".to_string(),
        name: "xs".to_string(),
    };
    assert_eq!(proof("xs[i]"), elements);
    assert_eq!(proof("xs.pop()"), elements);
    assert_eq!(
        proof("ws[i].length"),
        ElemProof::StringElements {
            func: "f".to_string(),
            name: "ws".to_string(),
        }
    );
    assert_eq!(
        proof("s.slice(1)"),
        ElemProof::StringBinding {
            func: "f".to_string(),
            name: "s".to_string(),
        }
    );
}

#[test]
fn built_in_results_lean_on_their_receiver() {
    let array_or_string = ElemProof::ArrayOrString {
        func: "f".to_string(),
        name: "xs".to_string(),
    };
    assert_eq!(proof("xs.length"), array_or_string);
    assert_eq!(proof("xs.indexOf(1)"), array_or_string);
    assert_eq!(proof("xs.slice(1).indexOf(1)"), array_or_string);
    assert_eq!(
        proof("xs.join()"),
        ElemProof::All(vec![array_or_string, ElemProof::Str])
    );
    assert_eq!(
        proof("Math.floor(y)"),
        ElemProof::Global {
            func: "f".to_string(),
            name: "Math".to_string(),
        }
    );
    assert_eq!(proof("s.charCodeAt(0)"), binding("s"));
}

#[test]
fn all_simplifies_and_no_absorbs() {
    assert_eq!(ElemProof::all(vec![]), ElemProof::Yes);
    assert_eq!(
        ElemProof::all(vec![ElemProof::Yes, binding("y")]),
        binding("y")
    );
    assert_eq!(
        ElemProof::all(vec![binding("y"), ElemProof::No]),
        ElemProof::No
    );
    assert_eq!(
        ElemProof::all(vec![
            ElemProof::All(vec![binding("y"), binding("z")]),
            ElemProof::Str
        ]),
        ElemProof::All(vec![binding("y"), binding("z"), ElemProof::Str])
    );
}

#[test]
fn substitute_replaces_a_parameter_read_and_refuses_other_uses() {
    let with = ElemProof::GrowElements(GrowNode::Binding("main".to_string(), "xs".to_string()));
    assert_eq!(binding("p").substitute("f", "p", &with), with);
    assert_eq!(binding("q").substitute("f", "p", &with), binding("q"));
    let elements = ElemProof::Elements {
        func: "f".to_string(),
        name: "p".to_string(),
    };
    assert_eq!(elements.substitute("f", "p", &with), ElemProof::No);
    assert_eq!(
        ElemProof::All(vec![binding("p"), binding("q")]).substitute("f", "p", &ElemProof::Yes),
        binding("q")
    );
}

#[test]
fn mentions_binding_looks_through_conjunctions() {
    let p = ElemProof::All(vec![binding("y"), ElemProof::Str]);
    assert!(p.mentions_binding("y"));
    assert!(!p.mentions_binding("z"));
    assert!(!ElemProof::Yes.mentions_binding("y"));
}
